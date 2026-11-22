//! `cardgen weekly-report`: the weekly job's report (`data/knowledge/reports/YYYY-MM-DD.md`). It compares the previous
//! sync (`oracle.prev.json`) with the current one (`oracle.json`) for new cards, bans and Game Changer changes, checks
//! JD's decks (`mine` of `data/decks.json`) for legality and Game Changer changes and for Oracle errata against the
//! engine's cache (`data/scryfall_cache.json`), and lists role changes and the LLM stage's numbers. Offline and
//! read-only except for the one report file (and the pruning of old reports). It never fails because an input is
//! missing or damaged: the report says so instead (an unreadable catalog is itself an action item). See
//! `documents/card-knowledge.md`.

use crate::data::Project;
use crate::decks::gc_rules;
use crate::digest;
use crate::knowledge::{self, Card, Index, Raw, fold};
use crate::map::{self, MineDeck};
use crate::roles;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "usage: cardgen weekly-report [--date YYYY-MM-DD] [--commit TEXT] [--stages FILE] \
[--roles-before FILE] [--llm-log FILE] [--out PATH] [--keep N] [--stdout] [--data-dir DIR]";

/// reports kept in `reports/` by default
pub const KEEP: usize = 12;
/// rows a list shows before it says `(N more)`
const LIST_CAP: usize = 40;
const SET_CAP: usize = 25;
const SETS_SHOWN: usize = 30;

/// the stage names the script records (the report looks up these two)
pub const BULK_STAGE: &str = "bulk-sync";
pub const LLM_STAGE: &str = "llm";

// ------------------------------------------------------------------------------------------------ inputs

/// A catalog read with the loader `find` uses ([`Index::load`]), so a name means the same card here as everywhere
/// else: an exact full name beats a face name, and the Commander-legal card of a shared name wins.
pub struct Cat {
    pub updated_at: String,
    idx: Index,
}

impl Cat {
    pub fn load(path: &Path) -> Result<Cat, String> {
        let idx = Index::load(path)?;
        Ok(Cat { updated_at: idx.bulk_updated_at.clone(), idx })
    }

    pub fn len(&self) -> usize {
        self.idx.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.idx.cards.is_empty()
    }

    pub fn cards(&self) -> impl Iterator<Item = &Raw> {
        self.idx.cards.iter().map(|c| &c.raw)
    }

    fn id(&self, id: &str) -> Option<&Raw> {
        self.idx.by_oracle_id(id).map(|i| &self.idx.cards[i as usize].raw)
    }

    /// the cards a deck's name stands for ([`digest::resolve`], the policy of `find` and the digests)
    fn resolve(&self, name: &str) -> Vec<&Card> {
        digest::resolve(&self.idx, name).into_iter().map(|i| &self.idx.cards[i as usize]).collect()
    }
}

/// the previous sync
pub enum Prev {
    /// no `oracle.prev.json`: the first run
    Missing,
    /// there is one, but it cannot be read
    Bad(String),
    Ok(Cat),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Ok,
    Skip,
    Fail,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Skip => "skip",
            Status::Fail => "fail",
        }
    }
}

/// one stage of the job as the script recorded it
#[derive(Clone, Debug)]
pub struct Stage {
    pub name: String,
    pub status: Status,
    pub secs: u64,
    pub note: String,
}

/// `name<TAB>status<TAB>seconds<TAB>note` per line (`#` comments and blank lines ignored); lines that do not parse are
/// returned as problems, not dropped silently
pub fn parse_stages(text: &str) -> (Vec<Stage>, Vec<String>) {
    let (mut stages, mut bad) = (vec![], vec![]);
    for line in text.lines().map(str::trim_end).filter(|l| !l.trim().is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.splitn(4, '\t').collect();
        let status = match f.get(1).copied() {
            Some("ok") => Status::Ok,
            Some("skip") => Status::Skip,
            Some("fail") => Status::Fail,
            _ => {
                bad.push(line.to_string());
                continue;
            }
        };
        let Some(secs) = f.get(2).and_then(|s| s.parse().ok()) else {
            bad.push(line.to_string());
            continue;
        };
        stages.push(Stage { name: f[0].to_string(), status, secs, note: f.get(3).unwrap_or(&"").to_string() });
    }
    (stages, bad)
}

type RoleTable = BTreeMap<String, Vec<String>>;

/// everything the report is made of; each part may be missing or broken, and the report says which
pub struct Inputs {
    pub date: String,
    pub commit: String,
    pub new: Result<Cat, String>,
    pub prev: Prev,
    pub decks: Result<Vec<MineDeck>, String>,
    /// the engine's name-keyed cache (`data/scryfall_cache.json`)
    pub cache: Result<HashMap<String, Card>, String>,
    /// oracle_id -> roles, now and before this run's role stage
    pub roles: Result<RoleTable, String>,
    pub roles_before: Option<Result<RoleTable, String>>,
    pub stages: Vec<Stage>,
    pub stage_problems: Vec<String>,
    pub llm_log: Option<String>,
    /// rows of `llm_roles.jsonl` and of `llm_roles.pending.jsonl`
    pub llm_rows: (usize, usize),
    /// `reports/merge.json` of the last merge, if any
    pub merge: Option<serde_json::Value>,
}

// ------------------------------------------------------------------------------------------------ reading

fn read_roles(path: &Path) -> Result<RoleTable, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let cards =
        v.get("cards").and_then(|c| c.as_object()).ok_or_else(|| format!("{}: no cards object", path.display()))?;
    Ok(cards
        .iter()
        .map(|(id, a)| {
            let roles = a.get("roles").and_then(|r| r.as_array()).into_iter().flatten();
            (id.clone(), roles.filter_map(|r| r.as_str().map(String::from)).collect())
        })
        .collect())
}

/// the engine's cache by folded name
fn read_cache(path: &Path) -> Result<HashMap<String, Card>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let map: BTreeMap<String, Raw> =
        serde_json::from_str(&text).map_err(|e| format!("{}: cannot read it ({e})", path.display()))?;
    let mut out = HashMap::new();
    for (key, raw) in map {
        let card = Card::new(raw);
        out.insert(fold(&key), card);
    }
    Ok(out)
}

fn count_rows(path: &Path) -> usize {
    std::fs::read_to_string(path).map(|t| t.lines().filter(|l| !l.trim().is_empty()).count()).unwrap_or(0)
}

// ------------------------------------------------------------------------------------------------ the report

fn is_legal(c: &Raw) -> bool {
    c.legalities.commander.as_deref() == Some("legal")
}

fn legality(c: &Raw) -> &str {
    c.legalities.commander.as_deref().unwrap_or("unknown")
}

/// EDHREC rank ascending, no rank last, then the name
fn by_rank(a: &&Raw, b: &&Raw) -> std::cmp::Ordering {
    (a.edhrec_rank.unwrap_or(u32::MAX), &a.name).cmp(&(b.edhrec_rank.unwrap_or(u32::MAX), &b.name))
}

/// `a, b, c (N more)`
fn capped(names: &[String], cap: usize) -> String {
    let mut s = names.iter().take(cap).cloned().collect::<Vec<_>>().join(", ");
    if names.len() > cap {
        let _ = write!(s, " ({} more)", names.len() - cap);
    }
    s
}

fn short(s: &str, n: usize) -> String {
    let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= n { one } else { format!("{}...", one.chars().take(n).collect::<String>()) }
}

/// Data-derived text for the report: one line (every whitespace run, newline included, becomes one space) with the
/// characters that make Markdown structure escaped, so a card name, key, set or note can never start a heading, a
/// list, a link, a table cell or a code span of its own.
fn md(s: &str) -> String {
    md_keep(s).trim().to_string()
}

/// [`md`] without trimming: the pieces of one line keep their edge spaces
fn md_keep(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            space = true;
            continue;
        }
        if c.is_control() {
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        if matches!(c, '\\' | '`' | '[' | ']' | '|' | '#' | '<' | '>' | '*') {
            out.push('\\');
        }
        out.push(c);
    }
    if space {
        out.push(' ');
    }
    out
}

/// text for inside a backtick span: one line, no backtick
fn code(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").replace('`', "'")
}

/// a deck of `mine` with its cards resolved against the current catalog
struct DeckView<'a> {
    key: &'a str,
    ident: &'a str,
    tiers: &'a [String],
    /// the resolved cards, once each
    cards: Vec<&'a Card>,
    /// escaped for the report
    missing: Vec<String>,
}

fn deck_views<'a>(decks: &'a [MineDeck], cat: &'a Cat) -> Vec<DeckView<'a>> {
    decks
        .iter()
        .map(|d| {
            let names = d
                .deck
                .cards
                .iter()
                .map(String::as_str)
                .chain([d.deck.commander.as_str()])
                .chain(d.deck.commander2.as_deref())
                .filter(|n| !n.is_empty());
            let (mut cards, mut missing): (Vec<&Card>, Vec<String>) = (vec![], vec![]);
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for n in names {
                let found = cat.resolve(n);
                if found.is_empty() {
                    let n = md(n);
                    if !missing.contains(&n) {
                        missing.push(n);
                    }
                }
                for c in found {
                    if seen.insert(c.raw.oracle_id.as_str()) {
                        cards.push(c);
                    }
                }
            }
            DeckView { key: &d.deck.key, ident: &d.deck.ident, tiers: &d.tiers, cards, missing }
        })
        .collect()
}

/// the card's colour identity fits inside the deck's
fn fits(card: &Raw, ident: &str) -> bool {
    card.color_identity.iter().all(|c| ident.contains(c.as_str()))
}

/// the deck's Game Changer count against the rule of each tier it is simulated at
fn gc_rule_text(count: usize, tiers: &[String]) -> String {
    if tiers.is_empty() {
        return format!("{count} Game Changers now (the deck lists no tiers, so no tier rule applies)");
    }
    let rules: Vec<String> = tiers
        .iter()
        .map(|t| match gc_rules(t) {
            Some((lo, hi)) => {
                let verdict = if (lo..=hi).contains(&count) { "within" } else { "OUTSIDE" };
                format!("{t} allows {lo}-{hi}: {verdict}")
            }
            None => format!("{t} is unrestricted"),
        })
        .collect();
    format!(
        "{count} Game Changers now ({}; the rule is decks::gc_rules for the tiers in data/my_decks.toml)",
        rules.join("; ")
    )
}

/// The Oracle text of a card as the classifier reads it ([`roles::normalize`]): reminder text stripped, the card's own
/// name written `~`, whitespace collapsed. Two printings that differ only in those are the same Oracle text.
fn oracle_text(c: &Card) -> String {
    let lines: Vec<String> = c.lines().iter().map(|(_, l)| roles::normalize(c, l)).collect();
    lines
        .iter()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" / ")
}

/// the fields of a card the engine's cache and the catalog both hold, in a comparable form (whitespace collapsed)
fn oracle_fields(c: &Card) -> Vec<(&'static str, String)> {
    let r = &c.raw;
    let g = |o: &Option<String>| o.clone().unwrap_or_default();
    let faces: Vec<String> = r
        .card_faces
        .iter()
        .flatten()
        .map(|x| {
            [&x.name, &g(&x.mana_cost), &x.type_line, &g(&x.power), &g(&x.toughness), &g(&x.loyalty)]
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();
    let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    vec![
        ("mana_cost", flat(&g(&r.mana_cost))),
        ("cmc", format!("{}", r.cmc)),
        ("type_line", flat(&r.type_line)),
        ("oracle_text", oracle_text(c)),
        ("power", g(&r.power)),
        ("toughness", g(&r.toughness)),
        ("loyalty", g(&r.loyalty)),
        ("color_identity", r.color_identity.join("")),
        ("keywords", r.keywords.join(",")),
        ("card_faces", flat(&faces.join("\n"))),
    ]
}

/// Where two texts differ: the changed middle of each (the common start and end left out) with a little context,
/// `"...before[old -> new]after..."`.
fn change(a: &str, b: &str) -> String {
    const CTX: usize = 30;
    const MID: usize = 80;
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ").chars().collect::<Vec<char>>();
    let (a, b) = (norm(a), norm(b));
    let max = a.len().min(b.len());
    let start = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let end = a.iter().rev().zip(b.iter().rev()).take(max - start).take_while(|(x, y)| x == y).count();
    let text = |s: &[char]| s.iter().collect::<String>();
    let mid = |s: &[char]| {
        let m = text(s);
        if m.chars().count() > MID { format!("{}...", m.chars().take(MID).collect::<String>()) } else { m }
    };
    let (old, new) = (mid(&a[start..a.len() - end]), mid(&b[start..b.len() - end]));
    let before = &a[start.saturating_sub(CTX)..start];
    let after = &a[a.len() - end..a.len() - end + end.min(CTX)];
    let dots = |cut: bool| if cut { "..." } else { "" };
    format!(
        "{}{}[{} -> {}]{}{}",
        dots(start > CTX),
        md_keep(&text(before)),
        md(&old),
        md(&new),
        md_keep(&text(after)),
        dots(end > CTX)
    )
}

/// the fields in which the cache differs from the catalog, with the change
fn errata(cache: &Card, now: &Card) -> Vec<String> {
    let (a, b) = (oracle_fields(cache), oracle_fields(now));
    a.iter()
        .zip(&b)
        .filter(|(x, y)| x.1 != y.1)
        .map(|(x, y)| match x.0 {
            "card_faces" => "a face differs".to_string(),
            k => format!("{k}: \"{}\"", change(&x.1, &y.1)),
        })
        .collect()
}

/// what changed between two syncs, by oracle_id
#[derive(Default)]
struct Diff<'a> {
    new: Vec<&'a Raw>,
    removed: Vec<&'a Raw>,
    /// (before, after) where the commander legality differs
    legality: Vec<(&'a Raw, &'a Raw)>,
    /// (before, after) where the Game Changer flag differs
    gc: Vec<(&'a Raw, &'a Raw)>,
}

fn diff<'a>(prev: &'a Cat, now: &'a Cat) -> Diff<'a> {
    let mut d = Diff::default();
    for c in now.cards() {
        match prev.id(&c.oracle_id) {
            None => d.new.push(c),
            Some(p) => {
                if legality(p) != legality(c) {
                    d.legality.push((p, c));
                }
                if p.game_changer != c.game_changer {
                    d.gc.push((p, c));
                }
            }
        }
    }
    d.removed = prev.cards().filter(|p| now.id(&p.oracle_id).is_none()).collect();
    let key = |c: &Raw| (c.name.clone(), c.oracle_id.clone());
    d.new.sort_by_key(|c| key(c));
    d.removed.sort_by_key(|c| key(c));
    d.legality.sort_by_key(|(_, c)| key(c));
    d.gc.sort_by_key(|(_, c)| key(c));
    d
}

fn stage<'a>(stages: &'a [Stage], name: &str) -> Option<&'a Stage> {
    stages.iter().find(|s| s.name == name)
}

/// why there is no new-against-old comparison this run, if there is none
fn no_diff_reason(inp: &Inputs) -> Option<String> {
    match (&inp.new, &inp.prev, stage(&inp.stages, BULK_STAGE).map(|s| s.status)) {
        (Err(e), _, _) => Some(format!("the catalog cannot be read ({e})")),
        (_, _, Some(Status::Skip)) => {
            Some("the bulk file did not change this run, so there are no new cards and no ban changes".into())
        }
        (_, _, Some(Status::Fail)) => Some("the catalog sync failed this run, so there is nothing to compare".into()),
        (_, Prev::Missing, _) => Some("there is no previous sync (oracle.prev.json): this is the first run".into()),
        (_, Prev::Bad(e), _) => Some(format!("the previous sync cannot be read ({e})")),
        _ => None,
    }
}

/// the items of "Action needed", for JD's decks only plus the run's own failures
fn action_items(inp: &Inputs, d: Option<&Diff>, views: &[DeckView]) -> Vec<String> {
    let mut action: Vec<String> = vec![];
    for s in inp.stages.iter().filter(|s| s.status == Status::Fail) {
        let why = if s.note.is_empty() { "no reason recorded" } else { &s.note };
        action.push(format!("Stage `{}` failed: {}", code(&s.name), md(why)));
    }
    for p in &inp.stage_problems {
        action.push(format!("A line of the stage file could not be read: `{}`", code(p)));
    }
    if let Err(e) = &inp.new {
        action.push(format!("The catalog cannot be read: {}", md(e)));
    }
    if let Prev::Bad(e) = &inp.prev {
        action.push(format!("The previous sync cannot be read, so no changes were compared: {}", md(e)));
    }
    match &inp.decks {
        Err(e) => action.push(format!("JD's decks cannot be read, so no deck was checked: {}", md(e))),
        Ok(decks) if decks.is_empty() && inp.new.is_ok() => {
            action.push("`mine` in data/decks.json is empty, so no deck was checked.".into())
        }
        Ok(_) => {}
    }
    if let Err(e) = &inp.cache {
        action.push(format!("The engine's card cache cannot be read, so no errata were checked: {}", md(e)));
    }
    for v in views.iter().filter(|v| !v.missing.is_empty()) {
        action.push(format!("Deck `{}`: not found in the catalog: {}", code(v.key), capped(&v.missing, 10)));
    }
    if let Some(d) = d {
        for v in views {
            let ids: BTreeSet<&str> = v.cards.iter().map(|c| c.raw.oracle_id.as_str()).collect();
            for (p, c) in d.legality.iter().filter(|(_, c)| ids.contains(c.oracle_id.as_str())) {
                if is_legal(p) && !is_legal(c) {
                    let (now, was) = (legality(c).replace('_', " "), legality(p).replace('_', " "));
                    action.push(format!("Deck `{}`: **{}** became {now} (was {was}).", code(v.key), md(&c.name)));
                }
            }
            let changes: Vec<String> = d
                .gc
                .iter()
                .filter(|(_, c)| ids.contains(c.oracle_id.as_str()))
                .map(|(_, c)| {
                    format!("{} {} the Game Changer list", md(&c.name), if c.game_changer { "joined" } else { "left" })
                })
                .collect();
            if !changes.is_empty() {
                let count = v.cards.iter().filter(|c| c.raw.game_changer).count();
                action.push(format!(
                    "Deck `{}`: {}; {}.",
                    code(v.key),
                    changes.join(", "),
                    gc_rule_text(count, v.tiers)
                ));
            }
        }
    } else if matches!(inp.prev, Prev::Missing) {
        // no previous sync to compare: still say which deck cards are not legal now
        for v in views {
            for c in v.cards.iter().map(|c| &c.raw).filter(|c| !is_legal(c)) {
                let now = legality(c).replace('_', " ");
                action.push(format!(
                    "Deck `{}`: **{}** is {now} (no previous sync to say since when).",
                    code(v.key),
                    md(&c.name)
                ));
            }
        }
    }
    if let (Ok(cache), Ok(_)) = (&inp.cache, &inp.new) {
        for v in views {
            let stale: Vec<(&Card, Vec<String>)> = v
                .cards
                .iter()
                .filter_map(|c| cache.get(&fold(&c.raw.name)).map(|k| (*c, errata(k, c))))
                .filter(|(_, e)| !e.is_empty())
                .collect();
            if stale.is_empty() {
                continue;
            }
            action.push(format!(
                "Deck `{}`: {} card(s) differ from data/scryfall_cache.json (Oracle errata; the engine's model may be \
                 stale):",
                code(v.key),
                stale.len()
            ));
            for (c, e) in stale.iter().take(LIST_CAP) {
                action.push(format!("  - {}: {}", md(&c.raw.name), e.join("; ")));
            }
            if stale.len() > LIST_CAP {
                action.push(format!("  - ({} more)", stale.len() - LIST_CAP));
            }
        }
    }
    action
}

fn section_run(o: &mut String, inp: &Inputs, reason: &Option<String>) {
    o.push_str("\n## Run\n\n");
    let _ = writeln!(o, "- Date: {}", inp.date);
    let commit = if inp.commit.is_empty() { "not recorded".to_string() } else { md(&inp.commit) };
    let _ = writeln!(o, "- Repository: {commit}");
    match (&inp.new, &inp.prev) {
        (Ok(now), prev) => {
            let legal = now.cards().filter(|c| is_legal(c)).count();
            match prev {
                Prev::Ok(p) if p.updated_at != now.updated_at => {
                    let _ = writeln!(o, "- Bulk file updated_at: {} -> {}", md(&p.updated_at), md(&now.updated_at));
                }
                _ => {
                    let _ =
                        writeln!(o, "- Bulk file updated_at: {} (unchanged, or no previous sync)", md(&now.updated_at));
                }
            }
            match prev {
                Prev::Ok(p) => {
                    let _ = writeln!(o, "- Cards: {} -> {} ({legal} Commander-legal now)", p.len(), now.len());
                }
                _ => {
                    let _ = writeln!(o, "- Cards: {} ({legal} Commander-legal)", now.len());
                }
            }
        }
        (Err(_), _) => o.push_str("- Catalog: not readable\n"),
    }
    if let Some(r) = reason {
        let _ = writeln!(o, "- Comparison: none, {}", md(r));
    }
    if inp.stages.is_empty() {
        o.push_str("- Stages: none recorded (this report was not written by the weekly script)\n");
    } else {
        o.push_str("\n| Stage | Status | Seconds | Note |\n|---|---|---|---|\n");
        for s in &inp.stages {
            let _ = writeln!(o, "| {} | {} | {} | {} |", md(&s.name), s.status.as_str(), s.secs, md(&s.note));
        }
    }
}

fn section_new(o: &mut String, inp: &Inputs, d: Option<&Diff>, views: &[DeckView], reason: &Option<String>) {
    o.push_str("\n## New cards\n\n");
    let Some(d) = d else {
        let _ = writeln!(o, "No new-card list this run: {}.", md(reason.as_deref().unwrap_or("no comparison")));
        return;
    };
    let legal_new = d.new.iter().filter(|c| is_legal(c)).count();
    let _ = writeln!(
        o,
        "{} new Oracle cards ({legal_new} Commander-legal), {} gone from the catalog.\n",
        d.new.len(),
        d.removed.len()
    );
    let mut sets: BTreeMap<&str, Vec<&Raw>> = BTreeMap::new();
    for c in &d.new {
        sets.entry(c.set.as_str()).or_default().push(c);
    }
    let mut order: Vec<(&str, &Vec<&Raw>)> = sets.iter().map(|(s, c)| (*s, c)).collect();
    order.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    for (set, cards) in order.iter().take(SETS_SHOWN) {
        let mut legal: Vec<&Raw> = cards.iter().copied().filter(|c| is_legal(c)).collect();
        legal.sort_by(by_rank);
        let names: Vec<String> = legal.iter().map(|c| md(&c.name)).collect();
        let _ = writeln!(
            o,
            "- **{}**: {} new, {} Commander-legal{}{}",
            if set.is_empty() { "(no set)".to_string() } else { md(set) },
            cards.len(),
            legal.len(),
            if names.is_empty() { "" } else { ": " },
            capped(&names, SET_CAP)
        );
    }
    if order.len() > SETS_SHOWN {
        let rest: usize = order[SETS_SHOWN..].iter().map(|(_, c)| c.len()).sum();
        let _ = writeln!(o, "- ({} more sets, {rest} cards)", order.len() - SETS_SHOWN);
    }
    if !d.removed.is_empty() {
        let names: Vec<String> = d.removed.iter().map(|c| md(&c.name)).collect();
        let _ = writeln!(o, "\nGone from the catalog: {}", capped(&names, 15));
    }
    o.push_str("\n### New Commander-legal cards inside each of JD's identities\n\n");
    if views.is_empty() {
        o.push_str("No decks to take identities from.\n");
    }
    for v in views {
        let mut inside: Vec<&Raw> = d.new.iter().copied().filter(|c| is_legal(c) && fits(c, v.ident)).collect();
        inside.sort_by(by_rank);
        let rows: Vec<String> = inside
            .iter()
            .map(|c| match inp.roles.as_ref().ok().and_then(|r| r.get(&c.oracle_id)) {
                Some(roles) if !roles.is_empty() => format!("{} [{}]", md(&c.name), md(&roles.join(", "))),
                _ => md(&c.name),
            })
            .collect();
        let ident = if v.ident.is_empty() { "colourless".to_string() } else { md(v.ident) };
        let sep = if rows.is_empty() { "" } else { ": " };
        let _ = writeln!(o, "- `{}` ({ident}): {} new{sep}{}", code(v.key), rows.len(), capped(&rows, LIST_CAP));
    }
    if let Err(e) = &inp.roles {
        let _ = writeln!(o, "\nRoles are not shown: {}", md(&short(e, 160)));
    }
}

fn section_bans(o: &mut String, d: Option<&Diff>, reason: &Option<String>) {
    o.push_str("\n## Bans/unbans and Game Changer changes\n\n");
    let Some(d) = d else {
        let _ = writeln!(o, "No comparison this run: {}.", md(reason.as_deref().unwrap_or("no comparison")));
        return;
    };
    let line = |p: &Raw, c: &Raw| format!("{} ({} -> {})", md(&c.name), md(legality(p)), md(legality(c)));
    let of = |f: &dyn Fn(&Raw, &Raw) -> bool| -> Vec<String> {
        d.legality.iter().filter(|(p, c)| f(p, c)).map(|(p, c)| line(p, c)).collect()
    };
    let banned = of(&|_, c| legality(c) == "banned");
    let unbanned = of(&|p, _| legality(p) == "banned");
    let other = of(&|p, c| legality(p) != "banned" && legality(c) != "banned");
    let gc = |on: bool| -> Vec<String> {
        d.gc.iter().filter(|(_, c)| c.game_changer == on).map(|(_, c)| md(&c.name)).collect()
    };
    let arrive: Vec<String> = d
        .new
        .iter()
        .filter(|c| legality(c) == "banned" || c.game_changer)
        .map(|c| format!("{} ({}{})", md(&c.name), md(legality(c)), if c.game_changer { ", Game Changer" } else { "" }))
        .collect();
    for (label, rows) in [
        ("Banned", banned),
        ("Unbanned", unbanned),
        ("Other legality changes", other),
        ("Game Changer added", gc(true)),
        ("Game Changer removed", gc(false)),
        ("New cards that arrive banned or as Game Changers", arrive),
    ] {
        let text = if rows.is_empty() { "none".to_string() } else { capped(&rows, LIST_CAP) };
        let _ = writeln!(o, "- {label}: {text}");
    }
}

fn section_roles(o: &mut String, inp: &Inputs, views: &[DeckView]) {
    o.push_str("\n## Role changes and LLM stats\n\n");
    match (&inp.roles, &inp.roles_before, &inp.new) {
        (Ok(now), Some(Ok(before)), Ok(cat)) => {
            let changed: Vec<(&String, &Vec<String>, &Vec<String>)> =
                now.iter().filter_map(|(id, r)| before.get(id).filter(|b| *b != r).map(|b| (id, b, r))).collect();
            let added = now.keys().filter(|id| !before.contains_key(*id)).count();
            let gone = before.keys().filter(|id| !now.contains_key(*id)).count();
            let _ = writeln!(
                o,
                "- Role table: {} cards ({added} not in the table before, {gone} gone), {} with different roles",
                now.len(),
                changed.len()
            );
            let deck_ids: BTreeSet<&str> =
                views.iter().flat_map(|v| v.cards.iter().map(|c| c.raw.oracle_id.as_str())).collect();
            // cards in JD's decks first
            let mut rows: Vec<(bool, String)> = changed
                .iter()
                .map(|(id, b, r)| {
                    let name = cat.id(id).map(|c| c.name.as_str()).unwrap_or(id.as_str());
                    (
                        !deck_ids.contains(id.as_str()),
                        format!("{}: [{}] -> [{}]", md(name), md(&b.join(", ")), md(&r.join(", "))),
                    )
                })
                .collect();
            rows.sort();
            let _ = writeln!(o, "- Changes on cards in JD's decks: {}", rows.iter().filter(|r| !r.0).count());
            for (_, r) in rows.iter().take(LIST_CAP) {
                let _ = writeln!(o, "  - {r}");
            }
            if rows.len() > LIST_CAP {
                let _ = writeln!(o, "  - ({} more)", rows.len() - LIST_CAP);
            }
        }
        (Err(e), _, _) => {
            let _ = writeln!(o, "- Role table not readable: {}", md(&short(e, 160)));
        }
        (_, Some(Err(e)), _) => {
            let _ = writeln!(
                o,
                "- The role table from before this run is not readable, no role changes listed: {}",
                md(&short(e, 160))
            );
        }
        (_, None, _) => o.push_str("- No role table from before this run, so no role changes are listed.\n"),
        _ => o.push_str("- Role changes cannot be listed: the catalog is not readable.\n"),
    }
    let _ = writeln!(
        o,
        "- LLM cache: {} tracked rows (llm_roles.jsonl), {} pending rows (llm_roles.pending.jsonl, not merged)",
        inp.llm_rows.0, inp.llm_rows.1
    );
    if let Some(s) = stage(&inp.stages, LLM_STAGE) {
        let note = if s.note.is_empty() { String::new() } else { format!(": {}", md(&s.note)) };
        let _ = writeln!(o, "- LLM stage this run: {} ({}s){note}", s.status.as_str(), s.secs);
    }
    if let Some(m) = &inp.merge {
        let n = |k: &str| m.get(k).and_then(|v| v.as_u64());
        let parts: Vec<String> = ["routed", "with_answer", "llm_failed", "failed", "not_run", "not_legal"]
            .iter()
            .filter_map(|k| n(k).map(|v| format!("{k} {v}")))
            .collect();
        let dis: u64 = m
            .get("disagreements")
            .and_then(|d| d.as_object())
            .map(|d| d.values().filter_map(|k| k.as_object()).flat_map(|k| k.values()).filter_map(|v| v.as_u64()).sum())
            .unwrap_or(0);
        if !parts.is_empty() {
            let _ = writeln!(o, "- Last merge (reports/merge.json): {}; {dis} disagreements", parts.join(", "));
        }
    }
    if let Some(log) = &inp.llm_log {
        let lines: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
        if !lines.is_empty() {
            o.push_str("\nEnd of this run's LLM log:\n\n```\n");
            for l in &lines[lines.len().saturating_sub(12)..] {
                let _ = writeln!(o, "{}", short(l, 200).replace("```", "'''"));
            }
            o.push_str("```\n");
        }
    }
}

/// the whole report as text; pure, so the same inputs give the same bytes
pub fn render(inp: &Inputs) -> String {
    let reason = no_diff_reason(inp);
    let views = match (&inp.new, &inp.decks) {
        (Ok(now), Ok(decks)) => deck_views(decks, now),
        _ => vec![],
    };
    let d = match (&inp.new, &inp.prev, &reason) {
        (Ok(now), Prev::Ok(p), None) => Some(diff(p, now)),
        _ => None,
    };
    let action = action_items(inp, d.as_ref(), &views);
    let mut o = String::new();
    let _ = writeln!(o, "# Card knowledge: weekly report {}\n", inp.date);
    o.push_str("## Action needed\n\n");
    if action.is_empty() {
        o.push_str("Nothing needs action.\n");
    }
    for a in &action {
        if a.starts_with("  - ") {
            let _ = writeln!(o, "{a}");
        } else {
            let _ = writeln!(o, "- {a}");
        }
    }
    section_run(&mut o, inp, &reason);
    section_new(&mut o, inp, d.as_ref(), &views, &reason);
    section_bans(&mut o, d.as_ref(), &reason);
    section_roles(&mut o, inp, &views);
    o.push_str(
        "\nNothing here changed a tracked file. Pending LLM rows, errata and swap ideas wait for a reviewed session.\n",
    );
    o
}

// ------------------------------------------------------------------------------------------------ the command

#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub path: PathBuf,
    pub stdout: bool,
    pub action_items: usize,
    pub pruned: usize,
}

impl Output {
    pub fn summary(&self) -> String {
        let pruned = if self.pruned > 0 { format!(", {} old report(s) removed", self.pruned) } else { String::new() };
        format!(
            "weekly-report: {} bytes, {} action item(s) -> {}{pruned}",
            self.text.len(),
            self.action_items,
            self.path.display()
        )
    }
}

/// `YYYY-MM-DD` and a day that exists
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
        && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// The (date, sequence) of a dated report name: `YYYY-MM-DD.md` is the newest report of its day (sequence `u64::MAX`),
/// `YYYY-MM-DD.N.md` an earlier one that a later run replaced (the higher N, the later it was replaced). Sorting by
/// this key is chronological, with N compared as a number.
fn report_key(n: &str) -> Option<(String, u64)> {
    let stem = n.strip_suffix(".md")?;
    let date = stem.get(..10)?;
    let rest = &stem[10..];
    if !is_date(date) {
        return None;
    }
    if rest.is_empty() {
        return Some((date.to_string(), u64::MAX));
    }
    let seq = rest.strip_prefix('.').filter(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))?;
    Some((date.to_string(), seq.parse().ok().filter(|n| *n < u64::MAX)?))
}

/// Deletes the oldest dated reports so that `keep` stay; returns how many it removed. Other files are never touched.
pub fn prune(reports: &Path, keep: usize) -> usize {
    let Ok(rd) = std::fs::read_dir(reports) else { return 0 };
    let mut keyed: Vec<((String, u64), String)> = rd
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| report_key(&n).map(|k| (k, n)))
        .collect();
    keyed.sort();
    let extra = keyed.len().saturating_sub(keep);
    keyed.iter().take(extra).filter(|(_, n)| std::fs::remove_file(reports.join(n)).is_ok()).count()
}

/// The sequence number the next archive of `date` takes: one above the highest there is.
fn next_archive_seq(reports: &Path, date: &str) -> u64 {
    let Ok(rd) = std::fs::read_dir(reports) else { return 1 };
    rd.filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| report_key(&n))
        .filter(|(d, seq)| d == date && *seq != u64::MAX)
        .map(|(_, seq)| seq + 1)
        .max()
        .unwrap_or(1)
}

/// Publishes the report of `date` into `reports`: the whole text is written to a temporary file first; only then is
/// the report already there (if any) kept as `DATE.N.md` (a hard link, so `DATE.md` is never absent) and the new one
/// renamed over `DATE.md`. A write that fails leaves the previous report exactly where it was.
fn publish(reports: &Path, date: &str, text: &str) -> Result<PathBuf, String> {
    use std::io::Write;
    std::fs::create_dir_all(reports).map_err(|e| format!("{}: {e}", reports.display()))?;
    let path = reports.join(format!("{date}.md"));
    let tmp = reports.join(format!(".{date}.md.{}.tmp", std::process::id()));
    let written = std::fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(text.as_bytes()).and_then(|_| f.flush()))
        .map_err(|e| format!("{}: {e}", tmp.display()));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if path.exists() {
        let archive = reports.join(format!("{date}.{}.md", next_archive_seq(reports, date)));
        if let Err(e) = std::fs::hard_link(&path, &archive).or_else(|_| std::fs::copy(&path, &archive).map(|_| ())) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("{}: {e}", archive.display()));
        }
    }
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })?;
    Ok(path)
}

/// Parsed `weekly-report` arguments.
#[derive(Debug, Default)]
pub struct Args {
    pub date: String,
    pub commit: String,
    pub stages: Option<PathBuf>,
    pub roles_before: Option<PathBuf>,
    pub llm_log: Option<PathBuf>,
    pub out: Option<PathBuf>,
    pub keep: usize,
    pub stdout: bool,
    pub data_dir: Option<PathBuf>,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut a = Args { keep: KEEP, date: chrono::Local::now().format("%Y-%m-%d").to_string(), ..Args::default() };
    let mut i = 0;
    while i < args.len() {
        let f = args[i].as_str();
        i += 1;
        if f == "--stdout" {
            a.stdout = true;
            continue;
        }
        let v = args.get(i).filter(|v| !v.starts_with("--")).ok_or_else(|| format!("{f} needs a value\n{USAGE}"))?;
        i += 1;
        match f {
            "--date" if is_date(v) => a.date = v.clone(),
            "--date" => return Err(format!("--date needs YYYY-MM-DD, got '{v}'")),
            "--commit" => a.commit = v.clone(),
            "--stages" => a.stages = Some(PathBuf::from(v)),
            "--roles-before" => a.roles_before = Some(PathBuf::from(v)),
            "--llm-log" => a.llm_log = Some(PathBuf::from(v)),
            "--out" => a.out = Some(PathBuf::from(v)),
            "--keep" => {
                a.keep = v
                    .parse()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or_else(|| format!("--keep needs a number above 0, got '{v}'"))?
            }
            "--data-dir" => a.data_dir = Some(PathBuf::from(v)),
            _ => return Err(format!("unknown argument '{f}'\n{USAGE}")),
        }
    }
    Ok(a)
}

/// Reads the inputs of a report from the project and the knowledge directory.
pub fn gather(project: &Project, dir: &Path, a: &Args) -> Result<Inputs, String> {
    let (stages, stage_problems) = match &a.stages {
        Some(f) => parse_stages(&std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?),
        None => (vec![], vec![]),
    };
    let prev_path = dir.join("oracle.prev.json");
    let prev = if !prev_path.exists() {
        Prev::Missing
    } else {
        match Cat::load(&prev_path) {
            Ok(c) => Prev::Ok(c),
            Err(e) => Prev::Bad(e),
        }
    };
    let decks_path = project.data("decks.json");
    let decks = if decks_path.is_file() {
        map::load_mine(&decks_path)
    } else {
        Err(format!("{}: not found", decks_path.display()))
    };
    Ok(Inputs {
        date: a.date.clone(),
        commit: a.commit.clone(),
        new: Cat::load(&dir.join("oracle.json")),
        prev,
        decks,
        cache: read_cache(&project.data("scryfall_cache.json")),
        roles: read_roles(&dir.join("roles.json")),
        roles_before: a.roles_before.as_deref().map(read_roles),
        stages,
        stage_problems,
        llm_log: a.llm_log.as_ref().and_then(|p| std::fs::read_to_string(p).ok()),
        llm_rows: (count_rows(&dir.join("llm_roles.jsonl")), count_rows(&dir.join("llm_roles.pending.jsonl"))),
        merge: std::fs::read_to_string(dir.join("reports/merge.json")).ok().and_then(|t| serde_json::from_str(&t).ok()),
    })
}

/// `cardgen weekly-report`: writes the report (default `<knowledge>/reports/<date>.md`, published whole or not at
/// all; an earlier report of the same date becomes `<date>.N.md`) and keeps the newest `--keep` dated reports. With
/// `--out` it only writes that file. Fails only if an argument is wrong, the stage file named cannot be read, or the
/// report cannot be written.
pub fn command(project: &Project, args: &[String]) -> Result<Output, String> {
    let a = parse_args(args)?;
    let dir = knowledge::knowledge_dir(project, a.data_dir.as_deref())
        .unwrap_or_else(|_| digest::out_dir(project, a.data_dir.as_deref()));
    let text = render(&gather(project, &dir, &a)?);
    let action_items = text
        .split("## Action needed\n")
        .nth(1)
        .and_then(|s| s.split("\n## ").next())
        .map_or(0, |s| s.lines().filter(|l| l.starts_with("- ")).count());
    let reports = dir.join("reports");
    let (path, pruned) = match &a.out {
        Some(p) => {
            digest::write_file(p, &text)?;
            (p.clone(), 0)
        }
        None => {
            let path = publish(&reports, &a.date, &text)?;
            (path, prune(&reports, a.keep))
        }
    };
    Ok(Output { text, path, stdout: a.stdout, action_items, pruned })
}
