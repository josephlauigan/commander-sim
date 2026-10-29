//! `cardgen audit --paths`: every path a card's behaviour reaches the sim by, and the cards where two paths run the
//! same thing (Phase 2, "one card model"). A report only: nothing here changes what the sim does.
//!
//! The paths it reads, from the engine's own card database and card-code registry (built in memory, as the engine
//! would load them):
//! - the ability language: a card's compiled abilities (`CardDef.abilities`, run by `sim-core/src/dsl`);
//! - executing tags: tags an engine executor acts on (`sim_core::exec_tags`; only those that run for the card);
//! - card code: the registry's `CardImpl` slots, split into rule slots and AI slots (`sim_core::hooks::SLOT_CLASS`);
//! - card-name special cases in `engine/`, `ai/` and `cardcode.rs`: an inventory of string literals only.
//!
//! Double classes, worst first:
//! - **hard**: two executors do the same game action for one event (a compiled `triggered:E` and a code slot for E;
//!   a compiled `static:cost` and a `cost` / `self_cost` slot; compiled toxic and a `combat_damage` slot; compiled
//!   loyalty and `walker`; an executing enters tag and `etb`; uncounterable twice; a self-cost tag and `self_cost`;
//!   compiled `enters_tapped` / `no_lifegain` statics and the same slot);
//! - **shadowed**: a compiled spell under a code `resolve` (the compiled spell never runs);
//! - **competing**: a compiled activated ability and a code `options` / `land_options` (both offer plays);
//! - **split**: compiled abilities or executing tags and code rule slots for other events. Undeclared, it counts as
//!   a double; an override's `paths_ok = "why"` declares any of the classes above a split on purpose.
//!
//! Rule-code groups, for each card with code rule slots:
//! - **dsl-covers**: the card compiles fully (Full-auto, no hand verdict) and the compiled abilities cover every
//!   event its rule slots handle: the code is a deletion candidate, to verify by hand;
//! - **dsl-partial**: some compiles but not all, or some rule slots have no compiled counterpart;
//! - **dsl-none**: nothing compiles, or the abilities were removed (`dsl = []`).

use crate::audit::{Audit, cut};
use crate::export::Inputs;
use sim_core::cards::{CardDb, CardDef};
use sim_core::dsl::model::Ability;
use sim_core::exec_tags::{self, Exec};
use sim_core::hooks::{CardImpl, Registry, SlotClass};
use std::collections::{BTreeMap, BTreeSet};

/// the first line of a `--paths --md` file (`--md` refuses to overwrite a file that starts otherwise)
pub const MD_HEADER: &str = "# Card paths (cardgen audit --paths)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Hard,
    Shadowed,
    Competing,
    /// two paths for different parts, not declared
    Split,
    /// any double an override declares with `paths_ok`
    SplitOk,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Hard => "hard",
            Class::Shadowed => "shadowed",
            Class::Competing => "competing",
            Class::Split => "split",
            Class::SplitOk => "split (paths_ok)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Covers,
    Partial,
    None,
}

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Covers => "dsl-covers",
            Group::Partial => "dsl-partial",
            Group::None => "dsl-none",
        }
    }
}

/// One card's paths.
#[derive(Debug, Clone)]
pub struct CardPaths {
    pub name: String,
    /// the card data's source (manual, pool, scryfall, scryfall+dsl)
    pub source: String,
    /// compiled abilities by type: `triggered:etb`, `static:cost`, `activated`, `spell`, `loyalty`, `uncounterable`
    pub dsl: Vec<String>,
    /// executing tags that run for it, `tag(executor)`
    pub tags: Vec<String>,
    pub rule: Vec<&'static str>,
    pub ai: Vec<&'static str>,
    /// every double found, (class, what), before `paths_ok`
    pub doubles: Vec<(Class, String)>,
    /// the card's class: the worst double, or SplitOk when declared
    pub class: Option<Class>,
    pub paths_ok: Option<String>,
    /// for cards with rule slots
    pub group: Option<Group>,
    /// the rule slots no compiled ability covers
    pub uncovered: Vec<&'static str>,
    /// the audit's status from the card data alone
    pub auto: String,
}

/// the code slots a compiled triggered event's executor duplicates
fn trigger_slots(event: &str) -> &'static [&'static str] {
    match event {
        "etb" => &["etb"],
        "dies" => &["dies", "self_dies"],
        "dies-other" => &["dies"],
        "attack" => &["attack"],
        "cast" => &["cast"],
        "combat_damage" => &["combat_damage"],
        "landfall" => &["landfall"],
        "upkeep" => &["upkeep"],
        "draw" => &["draw"],
        "end_step" => &["end_step"],
        "gain_life" => &["gain_life"],
        _ => &[],
    }
}

/// does a compiled ability (by its label) cover a rule slot?
fn covered(slot: &str, dsl: &BTreeSet<String>) -> bool {
    let has = |l: &str| dsl.contains(l);
    match slot {
        "etb" => has("triggered:etb"),
        "dies" => has("triggered:dies") || has("triggered:dies-other"),
        "self_dies" => has("triggered:dies"),
        "attack" | "cast" | "upkeep" | "end_step" | "landfall" | "draw" | "gain_life" => {
            has(&format!("triggered:{slot}"))
        }
        "combat_damage" => has("triggered:combat_damage") || has("static:toxic"),
        "resolve" => has("spell"),
        "cost" | "self_cost" => has("static:cost"),
        "walker" => has("loyalty"),
        "uncounterable" => has("uncounterable"),
        "enters_tapped" => has("static:enters_tapped"),
        "no_lifegain" => has("static:no_lifegain"),
        _ => false,
    }
}

/// a compiled ability's label (None: not a rule: a note, an additional-cost reminder)
fn label(a: &Ability) -> Option<String> {
    Some(match a {
        Ability::Spell { .. } => "spell".into(),
        // "when this dies" (`self_dies` code) and "whenever another creature dies" (`dies` code) are apart
        Ability::Triggered { event, source, .. } if event == "dies" && source.as_deref() != Some("self") => {
            "triggered:dies-other".into()
        }
        Ability::Triggered { event, .. } => format!("triggered:{event}"),
        Ability::Activated { .. } => "activated".into(),
        Ability::Loyalty { .. } => "loyalty".into(),
        Ability::Static { kind, .. } if kind == "note" => return None,
        Ability::Static { kind, .. } if kind == "uncounterable" => "uncounterable".into(),
        Ability::Static { kind, .. } => format!("static:{kind}"),
        Ability::Replacement { replace, .. } => format!("replacement:{replace}"),
        Ability::AdditionalCost { .. } => return None,
    })
}

/// The paths of one card. `auto`: the audit's status from the card data alone; `removed`: an override set `dsl = []`.
pub fn card_paths(d: &CardDef, imp: Option<&CardImpl>, auto: &str, removed: bool, paths_ok: Option<&str>) -> CardPaths {
    let mut dsl: Vec<String> = d.abilities.iter().filter_map(label).collect();
    dsl.sort();
    dsl.dedup();
    let set: BTreeSet<String> = dsl.iter().cloned().collect();
    let rule = imp.map(|i| i.slots_of(SlotClass::Rule)).unwrap_or_default();
    let ai = imp.map(|i| i.slots_of(SlotClass::Ai)).unwrap_or_default();
    let live = exec_tags::live(d, imp.is_some_and(|i| i.resolve.is_some()));
    let tags: Vec<String> = live.iter().map(|(t, e)| format!("{}({})", t.name(), e.name())).collect();
    let has_rule = |s: &str| rule.contains(&s);
    let mut doubles: Vec<(Class, String)> = vec![];
    let mut hard = |w: String| doubles.push((Class::Hard, w));
    for l in &dsl {
        if let Some(ev) = l.strip_prefix("triggered:") {
            for s in trigger_slots(ev).iter().filter(|s| has_rule(s)) {
                hard(format!("trigger: {l} + {s}"));
            }
        }
    }
    for s in ["cost", "self_cost"].into_iter().filter(|s| set.contains("static:cost") && has_rule(s)) {
        hard(format!("cost: static:cost + {s}"));
    }
    if set.contains("static:toxic") && has_rule("combat_damage") {
        hard("toxic: static:toxic + combat_damage".into());
    }
    if set.contains("loyalty") && has_rule("walker") {
        hard("loyalty: loyalty + walker".into());
    }
    for k in ["enters_tapped", "no_lifegain"] {
        if set.contains(&format!("static:{k}")) && has_rule(k) {
            hard(format!("static: static:{k} + {k}"));
        }
    }
    if has_rule("etb") {
        for (t, _) in live.iter().filter(|(_, e)| matches!(e, Exec::EnterTrigger | Exec::EnterStatic)) {
            hard(format!("enter: tag {} + etb", t.name()));
        }
    }
    let self_unc = set.contains("uncounterable") || live.iter().any(|(_, e)| *e == Exec::Cast);
    if self_unc && has_rule("uncounterable") {
        let by = if set.contains("uncounterable") { "uncounterable" } else { "tag unc" };
        hard(format!("uncounterable: {by} + uncounterable"));
    }
    if has_rule("self_cost") {
        for (t, _) in live.iter().filter(|(_, e)| *e == Exec::SelfCost) {
            hard(format!("cost: tag {} + self_cost", t.name()));
        }
    }
    if set.contains("spell") && !d.perm && has_rule("resolve") {
        doubles.push((Class::Shadowed, "spell + resolve (the compiled spell never runs)".into()));
    }
    if set.contains("activated") {
        for s in ["options", "land_options"].into_iter().filter(|s| ai.contains(s)) {
            doubles.push((Class::Competing, format!("activated + {s}")));
        }
    }
    let other: Vec<String> = dsl
        .iter()
        .cloned()
        .chain(live.iter().filter(|(_, e)| *e != Exec::Mana).map(|(t, _)| format!("tag {}", t.name())))
        .collect();
    if doubles.is_empty() && !rule.is_empty() && !other.is_empty() {
        doubles.push((Class::Split, format!("{} + code {}", other.join(" "), rule.join(" "))));
    }
    let worst = doubles.iter().map(|(c, _)| *c).min();
    let class = match (worst, paths_ok) {
        (Some(_), Some(_)) => Some(Class::SplitOk),
        (w, _) => w,
    };
    let uncovered: Vec<&'static str> = rule.iter().copied().filter(|s| !covered(s, &set)).collect();
    let group = if rule.is_empty() {
        None
    } else if dsl.is_empty() || removed {
        Some(Group::None)
    } else if auto == "Full-auto" && uncovered.is_empty() {
        Some(Group::Covers)
    } else {
        Some(Group::Partial)
    };
    CardPaths {
        name: d.name.to_string(),
        source: d.source.to_string(),
        dsl,
        tags,
        rule,
        ai,
        doubles,
        class,
        paths_ok: paths_ok.map(String::from),
        group,
        uncovered,
        auto: auto.to_string(),
    }
}

/// the `paths_ok` reasons in card_overrides.toml, by card
pub fn paths_ok(inp: &Inputs) -> BTreeMap<String, String> {
    inp.overrides.iter().filter_map(|(n, e)| e.paths_ok.clone().map(|w| (n.clone(), w))).collect()
}

/// Every card's paths, by name (cards with no path at all are left out: vanilla or unmodeled cards).
pub fn all_paths(
    db: &CardDb,
    reg: &Registry,
    au: &Audit,
    inp: &Inputs,
    ok: &BTreeMap<String, String>,
) -> BTreeMap<String, CardPaths> {
    let removed: BTreeSet<&str> = inp
        .overrides
        .iter()
        .filter(|(_, e)| e.dsl.as_ref().is_some_and(|v| v.is_empty()))
        .map(|(n, _)| n.as_str())
        .collect();
    let mut out = BTreeMap::new();
    for d in db.iter() {
        let imp = reg.get(d.id);
        let name: &str = &d.name;
        let auto = au.auto_status(name, false).0;
        let cp = card_paths(d, imp, &auto, removed.contains(name), ok.get(name).map(String::as_str));
        if cp.dsl.is_empty() && cp.tags.is_empty() && cp.rule.is_empty() && cp.ai.is_empty() {
            continue;
        }
        out.insert(name.to_string(), cp);
    }
    out
}

/// Path 5's inventory: string literals in sim-core's `engine/`, `ai/` and `cardcode.rs` (test modules left out) that
/// are a card's exact name. (file, literals, distinct names) per file.
pub fn name_literals(inp: &Inputs, db: &CardDb) -> Result<Vec<(String, usize, BTreeSet<String>)>, String> {
    let src = inp.project.root.join("rust/crates/sim-core/src");
    let mut files = vec![src.join("cardcode.rs")];
    for dir in ["engine", "ai"] {
        let rd = std::fs::read_dir(src.join(dir)).map_err(|e| format!("{}: {e}", src.join(dir).display()))?;
        let mut v: Vec<_> =
            rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
        v.sort();
        files.extend(v);
    }
    let mut out = vec![];
    for f in files {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let code = text.split("\n#[cfg(test)]").next().unwrap_or("");
        let mut n = 0;
        let mut names = BTreeSet::new();
        for lit in string_literals(code) {
            if db.id(&lit).is_some() {
                n += 1;
                names.insert(lit);
            }
        }
        let rel = f.strip_prefix(&src).unwrap_or(&f).display().to_string();
        out.push((rel, n, names));
    }
    Ok(out)
}

/// the contents of a Rust source's plain string literals (comments skipped; escapes kept as written)
pub fn string_literals(src: &str) -> Vec<String> {
    let b: Vec<char> = src.chars().collect();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            '/' if b.get(i + 1) == Some(&'/') => {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
            }
            '/' if b.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            '\'' => {
                // a char literal ('"', '\'', 'x'), else a lifetime
                if b.get(i + 1) == Some(&'\\') {
                    i += 2;
                    while i < b.len() && b[i] != '\'' {
                        i += 1;
                    }
                    i += 1;
                } else if b.get(i + 2) == Some(&'\'') {
                    i += 3;
                } else {
                    i += 1;
                }
            }
            '"' => {
                let mut s = String::new();
                i += 1;
                while i < b.len() && b[i] != '"' {
                    if b[i] == '\\' && i + 1 < b.len() {
                        s.push(b[i]);
                        i += 1;
                    }
                    s.push(b[i]);
                    i += 1;
                }
                i += 1;
                out.push(s);
            }
            _ => i += 1,
        }
    }
    out
}

/// The cards of a deck key: a pool deck whose key starts with `key`, or one of your decks by key.
pub fn deck_cards(inp: &Inputs, key: &str) -> Result<(String, BTreeSet<String>), String> {
    let mut cards = BTreeSet::new();
    let mut names = vec![];
    for d in inp.pool.iter().filter(|d| d.key.starts_with(key)) {
        names.push(d.key.clone());
        cards.extend(d.cards.iter().cloned());
        cards.insert(d.commander.clone());
        cards.extend(d.commander2.clone());
    }
    for (k, cs) in inp.my_cards.iter().filter(|(k, _)| k == key || k.starts_with(key)) {
        names.push(k.clone());
        cards.extend(cs.iter().cloned());
        if let Some(m) = inp.my.iter().find(|m| &m.key == k) {
            cards.insert(m.commander.clone());
            cards.extend(m.commander2.clone());
        }
    }
    if names.is_empty() {
        return Err(format!("--deck {key}: no pool deck or deck of yours has that key"));
    }
    Ok((names.join(", "), cards))
}

/// The counts: (label, value) lines for a set of cards.
fn counts<'a>(cards: impl Iterator<Item = &'a CardPaths>) -> Vec<(String, usize)> {
    let v: Vec<&CardPaths> = cards.collect();
    let n = |f: &dyn Fn(&CardPaths) -> bool| v.iter().filter(|c| f(c)).count();
    let mut out = vec![
        ("cards with a path".to_string(), v.len()),
        ("with compiled abilities".into(), n(&|c| !c.dsl.is_empty())),
        ("with executing tags".into(), n(&|c| !c.tags.is_empty())),
        ("with code rule slots".into(), n(&|c| !c.rule.is_empty())),
        ("with code AI slots only".into(), n(&|c| c.rule.is_empty() && !c.ai.is_empty())),
    ];
    for cl in [Class::Hard, Class::Shadowed, Class::Competing, Class::Split, Class::SplitOk] {
        out.push((format!("double: {}", cl.name()), n(&|c| c.class == Some(cl))));
    }
    for g in [Group::Covers, Group::Partial, Group::None] {
        out.push((format!("rule code: {}", g.name()), n(&|c| c.group == Some(g))));
    }
    out
}

/// The report: summary counts (every card, the cards the decks play, `deck`), the hard / shadowed / competing table,
/// the undeclared and declared splits, the dsl-covers list and the card-name literal inventory. `md`: as Markdown.
pub fn report(
    all: &BTreeMap<String, CardPaths>,
    played: &BTreeSet<String>,
    deck: Option<&(String, BTreeSet<String>)>,
    literals: &[(String, usize, BTreeSet<String>)],
    md: bool,
) -> String {
    let mut o = String::new();
    let h = |o: &mut String, t: &str| {
        if md {
            o.push_str(&format!("\n## {t}\n\n"));
        } else {
            o.push_str(&format!("\n== {t}\n"));
        }
    };
    if md {
        o.push_str(&format!("{MD_HEADER}\n\n"));
        o.push_str(
            "Every path a card's behaviour reaches the sim by, and the cards where two paths run the same thing. \
             A report only; see `rust/crates/cardgen/src/paths.rs` for the classes and groups.\n",
        );
    } else {
        o.push_str("Card paths (cardgen audit --paths)\n");
    }
    h(&mut o, "Summary");
    let mut cols: Vec<(String, Vec<(String, usize)>)> = vec![
        ("all cards".into(), counts(all.values())),
        ("cards the decks play".into(), counts(all.values().filter(|c| played.contains(&c.name)))),
    ];
    if let Some((k, cs)) = deck {
        cols.push((format!("--deck {k}"), counts(all.values().filter(|c| cs.contains(&c.name)))));
    }
    if md {
        o.push_str(&format!(
            "| | {} |\n|---|{}\n",
            cols.iter().map(|c| c.0.as_str()).collect::<Vec<_>>().join(" | "),
            "---|".repeat(cols.len())
        ));
        for (i, (label, _)) in cols[0].1.iter().enumerate() {
            o.push_str(&format!(
                "| {label} | {} |\n",
                cols.iter().map(|c| c.1[i].1.to_string()).collect::<Vec<_>>().join(" | ")
            ));
        }
    } else {
        o.push_str(&format!(
            "{:<32}{}\n",
            "",
            cols.iter().map(|c| format!("{:>22}", cut(&c.0, 22))).collect::<String>()
        ));
        for (i, (label, _)) in cols[0].1.iter().enumerate() {
            o.push_str(&format!(
                "{label:<32}{}\n",
                cols.iter().map(|c| format!("{:>22}", c.1[i].1)).collect::<String>()
            ));
        }
    }
    let row = |c: &CardPaths, why: &str| -> String {
        let j = |v: &[String]| if v.is_empty() { "-".to_string() } else { v.join(" ") };
        let js = |v: &[&str]| if v.is_empty() { "-".to_string() } else { v.join(" ") };
        let g = c.group.map_or("-", |g| g.name());
        if md {
            format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                c.name,
                c.class.map_or("-", |x| x.name()),
                why,
                j(&c.dsl),
                j(&c.tags),
                js(&c.rule),
                js(&c.ai),
                g
            )
        } else {
            format!(
                "  {:<9} {:<36} {}\n            dsl: {} | tags: {} | rule: {} | ai: {} | {g}\n",
                c.class.map_or("-", |x| x.name()),
                cut(&c.name, 36),
                why,
                j(&c.dsl),
                j(&c.tags),
                js(&c.rule),
                js(&c.ai)
            )
        }
    };
    let head = "| Card | Class | Why | DSL | Executing tags | Rule slots | AI slots | Rule code |\n|---|---|---|---|---|---|---|---|\n";
    let section = |o: &mut String, title: &str, f: &dyn Fn(&CardPaths) -> bool| {
        h(o, title);
        let mut cs: Vec<&CardPaths> = all.values().filter(|c| f(c)).collect();
        cs.sort_by_key(|c| (c.class, !played.contains(&c.name), c.name.clone()));
        if cs.is_empty() {
            o.push_str("none\n");
            return;
        }
        if md {
            o.push_str(head);
        }
        for c in cs {
            let mut why: Vec<String> = c.doubles.iter().map(|(_, w)| w.clone()).collect();
            if let Some(ok) = &c.paths_ok {
                why.push(format!("paths_ok: {ok}"));
            }
            if !played.contains(&c.name) {
                why.push("(no deck plays it)".into());
            }
            o.push_str(&row(c, &why.join("; ")));
        }
    };
    section(&mut o, "Hard, shadowed and competing doubles", &|c| {
        matches!(c.class, Some(Class::Hard | Class::Shadowed | Class::Competing))
    });
    section(&mut o, "Declared splits (paths_ok)", &|c| c.class == Some(Class::SplitOk));
    section(&mut o, "Undeclared splits (counted as doubles until an override declares them)", &|c| {
        c.class == Some(Class::Split)
    });
    h(&mut o, "dsl-covers: card code the compiled abilities may replace (verify by hand)");
    let covers: Vec<&CardPaths> = all.values().filter(|c| c.group == Some(Group::Covers)).collect();
    if covers.is_empty() {
        o.push_str("none\n");
    }
    for c in covers {
        let tail = if played.contains(&c.name) { "" } else { " (no deck plays it)" };
        if md {
            o.push_str(&format!("- {}: code {}; compiled {}{tail}\n", c.name, c.rule.join(" "), c.dsl.join(" ")));
        } else {
            o.push_str(&format!(
                "  {:<36} code {} | compiled {}{tail}\n",
                cut(&c.name, 36),
                c.rule.join(" "),
                c.dsl.join(" ")
            ));
        }
    }
    h(&mut o, "Path 5: card names as string literals in engine/, ai/ and cardcode.rs (inventory)");
    let total: usize = literals.iter().map(|x| x.1).sum();
    let distinct: BTreeSet<&String> = literals.iter().flat_map(|x| x.2.iter()).collect();
    o.push_str(&format!(
        "{total} literals naming {} distinct cards (test modules left out){}\n",
        distinct.len(),
        if md { "\n" } else { "" }
    ));
    let mut by: Vec<&(String, usize, BTreeSet<String>)> = literals.iter().filter(|x| x.1 > 0).collect();
    by.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0.clone()));
    if md {
        o.push_str("| File | Literals | Distinct cards |\n|---|---|---|\n");
    }
    for (f, n, names) in by {
        if md {
            o.push_str(&format!("| {f} | {n} | {} |\n", names.len()));
        } else {
            o.push_str(&format!("  {f:<28} {n:>5} literals, {:>4} cards\n", names.len()));
        }
    }
    o
}
