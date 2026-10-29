//! `cardgen audit`: how faithfully the simulator models every card in the opponent pools and in your decks, and the
//! status of each pool deck's key engines (Python's `pool_audit.py`). The tables it works from are in data/audit.toml;
//! a card's own verdict is the `status` / `note` of its entry in data/card_overrides.toml.
//!
//! Statuses:
//!   Full         hand-verified against Oracle text (an override's status, documents/card-audit.md, [card_notes])
//!   Full-auto    every Oracle line compiled by the ability language and no ability keyword dropped; not hand-verified
//!   Approximate  modeled with a simplification (regex tags, or a note)
//!   Partial      main effect in, some text or keywords ignored
//!   Unmodeled    the sim never casts it, or it does nothing when cast
//!
//! `cardgen audit --hooks` also lists what is left to port: events the Python code handled for a card
//! (data/python_hooks.txt) that sim-core's card registry doesn't handle.

use crate::autotag::autotag;
use crate::cd::Cd;
use crate::data::{Project, read};
use crate::decks::PoolDeck;
use crate::export::Inputs;
use crate::json::J;
use crate::re::{rmatch, sub};
use std::collections::{BTreeMap, BTreeSet};

pub const STATUSES: [&str; 5] = ["Full", "Full-auto", "Approximate", "Partial", "Unmodeled"];

pub fn rank(s: &str) -> i32 {
    match s {
        "Full" => 4,
        "Full-auto" => 3,
        "Approximate" => 2,
        "Partial" => 1,
        _ => 0,
    }
}

#[derive(Debug, Default)]
pub struct Engine {
    pub label: String,
    pub cards: Vec<String>,
    pub mechs: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AuditData {
    pub unsupported_keywords: BTreeSet<String>,
    pub stat_tags: BTreeSet<String>,
    pub deck_only_tags: BTreeSet<String>,
    pub pool_ok: Vec<(String, Vec<String>)>,
    pub card_notes: BTreeMap<String, (String, String)>,
    pub mechanics: BTreeMap<String, (String, String)>,
    pub engines: Vec<(String, Vec<Engine>)>,
}

fn strs(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn status_note(v: &toml::Value) -> (String, String) {
    let g = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    (g("status"), g("note"))
}

pub fn load_audit_data(p: &Project) -> Result<AuditData, String> {
    let path = p.data("audit.toml");
    let t: toml::Table = read(&path)?.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut d = AuditData {
        unsupported_keywords: strs(t.get("unsupported_keywords")).into_iter().collect(),
        stat_tags: strs(t.get("stat_tags")).into_iter().collect(),
        deck_only_tags: strs(t.get("deck_only_tags")).into_iter().collect(),
        ..Default::default()
    };
    if let Some(po) = t.get("pool_ok").and_then(|v| v.as_table()) {
        d.pool_ok = po.iter().map(|(k, v)| (k.clone(), strs(Some(v)))).collect();
    }
    if let Some(cn) = t.get("card_notes").and_then(|v| v.as_table()) {
        d.card_notes = cn.iter().map(|(k, v)| (k.clone(), status_note(v))).collect();
    }
    if let Some(m) = t.get("mechanics").and_then(|v| v.as_table()) {
        d.mechanics = m.iter().map(|(k, v)| (k.clone(), status_note(v))).collect();
    }
    if let Some(e) = t.get("engines").and_then(|v| v.as_table()) {
        for (deck, list) in e {
            let engines = list
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|x| Engine {
                            label: x.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            cards: strs(x.get("cards")),
                            mechs: strs(x.get("mechs")),
                        })
                        .collect()
                })
                .unwrap_or_default();
            d.engines.push((deck.clone(), engines));
        }
    }
    Ok(d)
}

/// documents/card-audit.md: the hand-verified status of the cards your decks play (name -> status, note)
fn main_audit(p: &Project) -> BTreeMap<String, (String, String)> {
    let mut st = BTreeMap::new();
    if let Ok(text) = read(&p.root.join("documents").join("card-audit.md")) {
        for line in text.lines() {
            if let Some(m) =
                rmatch(r"^\| (.+?) \| (Modeled|Approximate|Partial|Not modeled|Unverified) \| (.*?) \|", line)
            {
                st.entry(m.gs(1).to_string()).or_insert((m.gs(2).to_string(), m.gs(3).to_string()));
            }
        }
    }
    st
}

/// does any value in this compiled ability equal the bare amount "X"?
fn has_bare_x(j: &J) -> bool {
    match j {
        J::Str(s) => s == "X",
        J::Arr(v) => v.iter().any(has_bare_x),
        J::Obj(o) => o.iter().any(|(_, v)| has_bare_x(v)),
        _ => false,
    }
}

/// `s[:n]` (code points)
pub fn cut(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

const SIMPLE_KW: [&str; 9] = ["flying", "deathtouch", "vigilance", "lifelink", "haste", "trample", "reach", "flash", "infect"];

/// Oracle text minus reminder text and lines made only of keywords the engine models
pub fn rules_text(oracle: &str) -> String {
    let t = sub(r"\([^)]*\)", "", oracle);
    let keep: Vec<&str> = t
        .split('\n')
        .filter(|l| {
            !l.trim().is_empty() && !l.split(',').all(|w| SIMPLE_KW.contains(&w.trim().to_lowercase().as_str()))
        })
        .collect();
    keep.join(" / ")
}

pub struct Audit<'a> {
    pub data: AuditData,
    pub cards: &'a BTreeMap<String, Cd>,
    pub inp: &'a Inputs,
    pub overrides: BTreeMap<&'a str, (&'a str, &'a str)>,
    main: BTreeMap<String, (String, String)>,
}

impl<'a> Audit<'a> {
    pub fn new(inp: &'a Inputs, cards: &'a BTreeMap<String, Cd>) -> Result<Audit<'a>, String> {
        let overrides = inp
            .overrides
            .iter()
            .filter_map(|(n, e)| match (&e.status, &e.note) {
                (Some(s), Some(t)) => Some((n.as_str(), (s.as_str(), t.as_str()))),
                _ => None,
            })
            .collect();
        Ok(Audit { data: load_audit_data(&inp.project)?, cards, inp, overrides, main: main_audit(&inp.project) })
    }

    fn deck_only(&self, cd: &Cd) -> Vec<String> {
        let mut bad: Vec<String> =
            cd.tags.keys().filter(|k| self.data.deck_only_tags.contains(*k)).map(|k| k.to_string()).collect();
        for (k, ok) in &self.data.pool_ok {
            if let Some(v) = cd.tags.get(k) {
                let v = v.py_str();
                if !ok.contains(&v) {
                    bad.push(format!("{k}={v}"));
                }
            }
        }
        bad
    }

    /// (status, note) for one card as an outside deck plays it; `mine`: as one of your decks plays it (your decks'
    /// AI acts on the hand tags in carddb.txt)
    pub fn card_status(&self, name: &str, mine: bool) -> (String, String) {
        if let Some((s, n)) = self.data.card_notes.get(name) {
            return (s.clone(), n.clone());
        }
        if let Some((s, n)) = self.overrides.get(name) {
            return (s.to_string(), n.to_string());
        }
        self.auto_status(name, mine)
    }

    /// (status, note) from the card data alone, without a hand verdict (audit.toml [card_notes], an override's status):
    /// "Full-auto" here means every Oracle line compiled, nothing unparsed and no ability keyword dropped
    pub fn auto_status(&self, name: &str, mine: bool) -> (String, String) {
        let Some(cd) = self.cards.get(name) else { return ("Unmodeled".into(), "not in the card database".into()) };
        if ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"].contains(&name) {
            return ("Full".into(), "basic land".into());
        }
        let rec = self.inp.cache.get(name);
        let kws: BTreeSet<String> = rec
            .map(|r| {
                r.strs("keywords")
                    .iter()
                    .map(|k| k.to_lowercase())
                    .filter(|k| self.data.unsupported_keywords.contains(k))
                    .collect()
            })
            .unwrap_or_default();
        let kwnote = if kws.is_empty() {
            String::new()
        } else {
            format!("keywords not modeled: {}", kws.iter().cloned().collect::<Vec<_>>().join(", "))
        };
        if cd.source == "manual" {
            let bad = self.deck_only(cd);
            if !bad.is_empty() && !mine {
                return (
                    "Unmodeled".into(),
                    format!("hand tags ({}) are only acted on by the main decks' AI", bad.join(", ")),
                );
            }
            let (s, note) = self.main.get(name).cloned().unwrap_or(("Modeled".into(), "hand-tagged".into()));
            let s = match s.as_str() {
                "Modeled" => "Full".to_string(),
                "Not modeled" => "Unmodeled".to_string(),
                "Unverified" => "Approximate".to_string(),
                _ => s,
            };
            return (s, note);
        }
        if cd.land() {
            let util: Vec<String> = rec.map(|r| autotag(r).unparsed).unwrap_or_default();
            if !util.is_empty() {
                return (
                    "Partial".into(),
                    format!("land utility ignored: {}", util.iter().map(|u| cut(u, 70)).collect::<Vec<_>>().join("; ")),
                );
            }
            return ("Full".into(), "mana only".into());
        }
        let mut functional: Vec<&str> = cd.tags.keys().filter(|k| !self.data.stat_tags.contains(*k)).collect();
        functional.sort();
        let abil_empty = cd.dsl.as_ref().is_none_or(|d| {
            d.iter().all(|a| {
                a.get("static").and_then(|s| s.as_str()) == Some("note")
                    || a.get("type").and_then(|s| s.as_str()) == Some("additional_cost")
            })
        });
        let unparsed = &cd.unparsed;
        let oracle = {
            let o = rec.and_then(|r| r.s("oracle_text")).unwrap_or("").to_string();
            if !o.is_empty() {
                o
            } else {
                rec.and_then(|r| r.get("card_faces"))
                    .and_then(|f| f.as_arr())
                    .map(|f| {
                        f.iter().map(|x| x.s("oracle_text").unwrap_or("").to_string()).collect::<Vec<_>>().join(" ")
                    })
                    .unwrap_or_default()
            }
        };
        let first3 = |n: usize| unparsed.iter().take(3).map(|u| cut(u, n)).collect::<Vec<_>>().join("; ");
        if abil_empty && functional.is_empty() {
            if cd.creature() {
                if rules_text(&oracle).is_empty() {
                    return ("Full".into(), "vanilla creature (keywords only)".into());
                }
                let what = if unparsed.is_empty() { cut(&rules_text(&oracle), 120) } else { first3(70) };
                let tail = if kwnote.is_empty() { String::new() } else { format!("; {kwnote}") };
                return ("Partial".into(), format!("body only, abilities not modeled: {what}{tail}"));
            }
            return ("Unmodeled".into(), if unparsed.is_empty() { "no effect recognised".into() } else { first3(70) });
        }
        let mut notes: Vec<String> = unparsed.iter().take(3).map(|u| cut(u, 80)).collect();
        if !kwnote.is_empty() {
            notes.push(kwnote);
        }
        if !unparsed.is_empty() || !kws.is_empty() {
            return ("Partial".into(), notes.join("; "));
        }
        // a bare "X" amount on a card with no {X} (a "that many" or counting phrase the compiler could not name) runs as
        // a stand-in 3 at run time: never "compiled from Oracle text"
        if cd.dsl.as_ref().is_some_and(|d| d.iter().any(|a| has_bare_x(&J::Obj(a.clone()))))
            && !oracle.contains("{X}")
            && !rec.and_then(|r| r.s("mana_cost")).unwrap_or("").contains("{X}")
        {
            return (
                "Partial".into(),
                format!("an amount the compiler could not name runs as 3: {}", cut(&rules_text(&oracle), 90)),
            );
        }
        if cd.source == "scryfall+dsl" {
            return ("Full-auto".into(), "compiled from Oracle text".into());
        }
        ("Approximate".into(), format!("regex tags: {}", functional.join(" ")))
    }

    fn mech(&self, m: &str) -> (String, String) {
        self.data
            .mechanics
            .get(m)
            .cloned()
            .unwrap_or(("Unmodeled".into(), format!("mechanic {m} is not in data/audit.toml")))
    }

    pub fn engine_status(&self, deck: &PoolDeck, cards: &[String], mechs: &[String]) -> (String, Vec<String>) {
        let mut worst = "Full".to_string();
        let mut notes = vec![];
        for c in cards {
            if !deck.cards.contains(c) {
                continue;
            }
            let (s, _) = self.card_status(c, false);
            if rank(&s) < rank(&worst) {
                worst = s.clone();
            }
            if rank(&s) < rank("Full-auto") {
                notes.push(format!("{c}: {s}"));
            }
        }
        for m in mechs {
            let (s, n) = self.mech(m);
            if rank(&s) < rank(&worst) {
                worst = s.clone();
            }
            if rank(&s) < rank("Full-auto") {
                notes.push(format!("{m}: {s}{}", if n.is_empty() { String::new() } else { format!(" ({n})") }));
            }
        }
        (worst, notes)
    }
}

pub struct Row<'a> {
    pub deck: &'a PoolDeck,
    pub per: BTreeMap<String, (String, String)>,
    pub cnt: BTreeMap<String, usize>,
    pub eng: Vec<(String, String, Vec<String>)>,
    pub missing: Vec<String>,
}

pub fn audit_pool<'a>(a: &Audit, decks: &'a [PoolDeck]) -> Vec<Row<'a>> {
    let mut out = vec![];
    for d in decks {
        let mut uniq: Vec<&String> = d.cards.iter().collect();
        uniq.sort();
        uniq.dedup();
        let per: BTreeMap<String, (String, String)> =
            uniq.iter().map(|n| ((*n).clone(), a.card_status(n, false))).collect();
        let mut cnt: BTreeMap<String, usize> = BTreeMap::new();
        for (s, _) in per.values() {
            *cnt.entry(s.clone()).or_default() += 1;
        }
        let engines: &[Engine] =
            a.data.engines.iter().find(|(k, _)| *k == d.key).map(|(_, e)| e.as_slice()).unwrap_or(&[]);
        let eng = engines
            .iter()
            .map(|e| {
                let (s, n) = a.engine_status(d, &e.cards, &e.mechs);
                (e.label.clone(), s, n)
            })
            .collect();
        let missing = engines.iter().flat_map(|e| e.cards.iter()).filter(|c| !d.cards.contains(c)).cloned().collect();
        out.push(Row { deck: d, per, cnt, eng, missing });
    }
    out
}

pub fn markdown(rows: &[Row]) -> String {
    let mut l = vec![
        format!("| Tier | Deck | {} | Engines: full / approx. / partial / unmodeled |", STATUSES.join(" | ")),
        format!("|{}", "---|".repeat(STATUSES.len() + 3)),
    ];
    for r in rows {
        let mut e: BTreeMap<String, usize> = BTreeMap::new();
        for (_, s, _) in &r.eng {
            *e.entry(if rank(s) >= rank("Full-auto") { "full".to_string() } else { s.to_lowercase() }).or_default() +=
                1;
        }
        let g = |k: &str| e.get(k).copied().unwrap_or(0);
        l.push(format!(
            "| {} | {} | {} | {} / {} / {} / {} |",
            r.deck.tier.chars().nth(1).unwrap_or('?'),
            crate::decks::short_name(&r.deck.commander),
            STATUSES.iter().map(|s| r.cnt.get(*s).copied().unwrap_or(0).to_string()).collect::<Vec<_>>().join(" | "),
            g("full"),
            g("approximate"),
            g("partial"),
            g("unmodeled")
        ));
    }
    l.push(String::new());
    for r in rows {
        l.push(format!("**{}** (Tier {})", r.deck.title, r.deck.tier.chars().nth(1).unwrap_or('?')));
        l.push(String::new());
        l.push("| Engine | Status | What is missing |".into());
        l.push("|---|---|---|".into());
        for (label, s, notes) in &r.eng {
            l.push(format!(
                "| {label} | {s} | {} |",
                if notes.is_empty() { "-".to_string() } else { notes.join("; ") }
            ));
        }
        l.push(String::new());
    }
    l.join("\n")
}

/// the summary table (default `cardgen audit` output)
pub fn summary(rows: &[Row]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<34} {}   engines ok\n",
        "deck",
        STATUSES.iter().map(|s| format!("{:>9}", cut(s, 9))).collect::<Vec<_>>().join(" ")
    ));
    let mut tot: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        let ok = r.eng.iter().filter(|(_, s, _)| rank(s) >= rank("Full-auto")).count();
        out.push_str(&format!(
            "{:<34} {}   {ok}/{}\n",
            cut(&r.deck.key, 34),
            STATUSES.iter().map(|s| format!("{:9}", r.cnt.get(*s).copied().unwrap_or(0))).collect::<Vec<_>>().join(" "),
            r.eng.len()
        ));
        if !r.missing.is_empty() {
            out.push_str(&format!("    (engine cards not in this list: {})\n", r.missing.join(", ")));
        }
        for (s, _) in r.per.values() {
            *tot.entry(STATUSES.iter().find(|x| *x == s).copied().unwrap_or("Unmodeled")).or_default() += 1;
        }
    }
    out.push_str(&format!(
        "all decks (card-slots, unique per deck): {}\n",
        STATUSES.iter().map(|s| format!("{s} {}", tot.get(s).copied().unwrap_or(0))).collect::<Vec<_>>().join(", ")
    ));
    out
}

/// one pool deck's cards by status (`--deck KEY`)
pub fn deck_report(rows: &[Row], key: &str) -> String {
    let mut out = String::new();
    for r in rows.iter().filter(|r| r.deck.key.starts_with(key)) {
        out.push_str(&format!("== {}\n", r.deck.title));
        let mut names: Vec<&String> = r.per.keys().collect();
        names.sort_by_key(|n| (rank(&r.per[*n].0), (*n).clone()));
        for n in names {
            out.push_str(&format!("  {:<11} {:<40} {}\n", r.per[n].0, cut(n, 40), cut(&r.per[n].1, 150)));
        }
    }
    out
}

/// your decks: every card that is not Full, as your decks play it (`--mine`)
pub fn mine_report(a: &Audit) -> String {
    let mut out = String::new();
    for (k, cards) in &a.inp.my_cards {
        let uniq: BTreeSet<&String> = cards.iter().collect();
        let per: Vec<(&String, (String, String))> = uniq.iter().map(|n| (*n, a.card_status(n, true))).collect();
        let mut bad: Vec<(&String, &(String, String))> =
            per.iter().filter(|(_, sn)| rank(&sn.0) < rank("Full-auto")).map(|(n, sn)| (*n, sn)).collect();
        out.push_str(&format!("== {k}: {} unique cards, {} not Full\n", per.len(), bad.len()));
        bad.sort_by_key(|(n, sn)| (rank(&sn.0), (*n).clone()));
        for (n, (s, note)) in bad {
            out.push_str(&format!("  {:<11} {:<40} {}\n", s, cut(n, 40), cut(note, 120)));
        }
    }
    out
}

/// What is left to port: events the Python code handled for a card that sim-core's registry doesn't handle, for the
/// cards the decks play (all of them with `all`).
/// The engine's card database and card-code registry, built in memory from the inputs as `cardgen export` would
/// write them (no file is read back): the real registry, so every way a card gets code is counted.
pub fn engine_db(inp: &Inputs) -> Result<(sim_core::cards::CardDb, sim_core::hooks::Registry), String> {
    use sim_core::export::RawCard;
    #[derive(serde::Deserialize)]
    struct File {
        cards: Vec<RawCard>,
    }
    let exported = crate::export::export(inp)?;
    let raw: File = serde_json::from_str(&exported.cards_json).map_err(|e| e.to_string())?;
    let db = sim_core::cards::CardDb::from_raw(&raw.cards)?;
    let reg = sim_core::impls::registry(&db)?;
    Ok((db, reg))
}

pub fn hooks_report(inp: &Inputs, built: &BTreeMap<String, Cd>, all: bool) -> Result<String, String> {
    use sim_core::hooks::Event;
    let (db, reg) = engine_db(inp)?;
    let w = inp.wanted();
    let mut out = String::new();
    let (mut total, mut done) = (0, 0);
    let mut gaps = vec![];
    for (name, h) in &inp.hooks {
        if h.events.is_empty() && !h.prio {
            continue;
        }
        if !built.contains_key(name) || (!all && !w.played.contains(name)) {
            continue;
        }
        total += 1;
        let imp = db.id(name).and_then(|id| reg.get(id));
        let missing: Vec<String> = h
            .events
            .iter()
            .filter(|ev| match Event::from_name(ev) {
                Some(e) => !imp.is_some_and(|i| i.handles(e)),
                None => true,
            })
            .cloned()
            .collect();
        if missing.is_empty() && (!h.prio || imp.is_some_and(|i| i.prio.is_some())) {
            done += 1;
        } else {
            let mut m = missing;
            if h.prio && !imp.is_some_and(|i| i.prio.is_some()) {
                m.push("prio".into());
            }
            gaps.push(format!("  {name}: {}", m.join(" ")));
        }
    }
    out.push_str(&format!(
        "Rust card code vs the Python hooks ({}): {done} of {total} cards handle every event the Python code did\n",
        if all { "every card" } else { "cards the decks play" }
    ));
    for g in gaps {
        out.push_str(&g);
        out.push('\n');
    }
    let _ = J::Null;
    Ok(out)
}

/// `cardgen audit --md FILE` may write only the model-status table: never a decklist (anything under a `decklists`
/// directory) and never an existing file that is not such a table. The deck docs have to stay loadable.
pub fn check_md_target(path: &str) -> Result<(), String> {
    check_md_target_for(path, "| Tier | Deck |")
}

/// `check_md_target` for a report whose files start with `header`
pub fn check_md_target_for(path: &str, header: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    if p.components().chain(abs.components()).any(|c| c.as_os_str() == "decklists") {
        return Err(format!("{path}: cardgen audit never writes into decklists/ (deck docs must keep their import list)"));
    }
    if let Ok(text) = std::fs::read_to_string(p)
        && !text.trim().is_empty()
        && !text.starts_with(header)
    {
        return Err(format!("{path}: exists and is not an audit table; refusing to overwrite it"));
    }
    Ok(())
}
