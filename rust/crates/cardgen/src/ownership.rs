//! `cardgen ownership --check`: validates `data/ownership.toml`, JD's record of which cards of his decks he has, plays
//! as a proxy or is missing, against the deck lists in `data/decks.json` (`mine`) and, when there is one, the card
//! catalog (`data/knowledge/oracle.json`). Read-only: it writes nothing. See README.md ("Ownership").
//!
//! The file (TOML, version 1):
//!
//! ```toml
//! version = 1
//! updated = "2026-11-22"
//! [deck.yshtola]
//! confirmed = "2026-11-22"          # optional: absent = JD has not confirmed the deck yet (treat it as unknown)
//! have = ["Sol Ring", { card = "Lightning Bolt", copies = 2 }]
//! proxy = []
//! missing = []
//! [spare]
//! cards = []                        # owned cards that are in no deck
//! ```

use crate::data::Project;
use crate::knowledge::{Index, fold, knowledge_dir};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

pub const USAGE: &str = "usage: cardgen ownership --check [--file PATH] [--data-dir DIR] [--json] [--strict]";

/// the only statuses a card of a deck can have
const STATUSES: [&str; 3] = ["have", "proxy", "missing"];

/// basic lands are never listed (and are left out of the deck lists they are compared with)
const BASICS: [&str; 12] = [
    "Plains",
    "Island",
    "Swamp",
    "Mountain",
    "Forest",
    "Wastes",
    "Snow-Covered Plains",
    "Snow-Covered Island",
    "Snow-Covered Swamp",
    "Snow-Covered Mountain",
    "Snow-Covered Forest",
    "Snow-Covered Wastes",
];

fn is_basic(name: &str) -> bool {
    let f = fold(name);
    BASICS.iter().any(|b| fold(b) == f)
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// the ownership file (default `data/ownership.toml`)
    pub file: Option<PathBuf>,
    /// where `oracle.json` lives (default: see `knowledge::knowledge_dir`)
    pub data_dir: Option<PathBuf>,
    /// a missing catalog is an error (otherwise name resolution is skipped with a warning)
    pub strict: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    /// not a problem: something to confirm that is not counted in the summary's errors and warnings
    Info,
}

#[derive(Debug, Clone)]
pub struct Problem {
    pub level: Level,
    pub deck: Option<String>,
    pub card: Option<String>,
    pub message: String,
}

impl Problem {
    fn text(&self) -> String {
        let mut s = String::from(match self.level {
            Level::Error => "ERROR",
            Level::Warning => "WARNING",
            Level::Info => "INFO",
        });
        if let Some(d) = &self.deck {
            s += &format!(" deck {d}:");
        } else {
            s += ":";
        }
        if let Some(c) = &self.card {
            s += &format!(" card {c:?}:");
        }
        format!("{s} {}", self.message)
    }
}

/// one deck's line of the summary
#[derive(Debug, Clone)]
pub struct DeckLine {
    pub key: String,
    /// the deck has a section in the file
    pub listed: bool,
    pub have: usize,
    pub proxy: usize,
    pub missing: usize,
    pub confirmed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub file: PathBuf,
    pub problems: Vec<Problem>,
    pub decks: Vec<DeckLine>,
    /// the sum of the copies of the `spare` cards
    pub spare: u64,
    /// names were resolved against the catalog (otherwise by their folded spelling)
    pub catalog: bool,
}

impl Report {
    pub fn errors(&self) -> usize {
        self.problems.iter().filter(|p| p.level == Level::Error).count()
    }

    pub fn warnings(&self) -> usize {
        self.problems.iter().filter(|p| p.level == Level::Warning).count()
    }

    pub fn infos(&self) -> usize {
        self.problems.iter().filter(|p| p.level == Level::Info).count()
    }

    pub fn ok(&self) -> bool {
        self.errors() == 0
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for p in &self.problems {
            out += &p.text();
            out += "\n";
        }
        for d in &self.decks {
            out += &format!(
                "{}: have {}, proxy {}, missing {}; {}\n",
                d.key,
                d.have,
                d.proxy,
                d.missing,
                match (&d.confirmed, d.listed) {
                    (Some(c), _) => format!("confirmed {c}"),
                    (None, true) => "unconfirmed".to_string(),
                    (None, false) => "no section".to_string(),
                }
            );
        }
        out += &format!("spare: {} cards\n", self.spare);
        out += &format!(
            "{}: {} errors, {} warnings, {} info{}\n",
            self.file.display(),
            self.errors(),
            self.warnings(),
            self.infos(),
            if self.catalog { "" } else { " (names not checked against the catalog)" }
        );
        out
    }

    pub fn json(&self) -> String {
        let level = |l: Level| match l {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Info => "info",
        };
        let v = json!({
            "file": self.file.display().to_string(),
            "ok": self.ok(),
            "errors": self.errors(),
            "warnings": self.warnings(),
            "infos": self.infos(),
            "catalog": self.catalog,
            "problems": self.problems.iter().map(|p| json!({
                "level": level(p.level), "deck": p.deck, "card": p.card, "message": p.message,
            })).collect::<Vec<_>>(),
            "decks": self.decks.iter().map(|d| json!({
                "key": d.key, "listed": d.listed, "have": d.have, "proxy": d.proxy, "missing": d.missing,
                "confirmed": d.confirmed,
            })).collect::<Vec<_>>(),
            "spare": self.spare,
        });
        serde_json::to_string_pretty(&v).unwrap_or_default() + "\n"
    }
}

#[derive(Default)]
struct Sink(Vec<Problem>);

impl Sink {
    fn add(&mut self, level: Level, deck: Option<&str>, card: Option<&str>, message: impl Into<String>) {
        self.0.push(Problem {
            level,
            deck: deck.map(String::from),
            card: card.map(String::from),
            message: message.into(),
        });
    }

    fn err(&mut self, deck: Option<&str>, card: Option<&str>, message: impl Into<String>) {
        self.add(Level::Error, deck, card, message);
    }

    fn warn(&mut self, deck: Option<&str>, card: Option<&str>, message: impl Into<String>) {
        self.add(Level::Warning, deck, card, message);
    }

    fn info(&mut self, deck: Option<&str>, card: Option<&str>, message: impl Into<String>) {
        self.add(Level::Info, deck, card, message);
    }
}

/// `YYYY-MM-DD`, a real calendar date
pub fn valid_date(s: &str) -> bool {
    if !s.is_ascii() || s.len() != 10 || &s[4..5] != "-" || &s[7..8] != "-" {
        return false;
    }
    if s.bytes().enumerate().any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit()) {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s[r].parse::<u32>().unwrap_or(0);
    let (y, m, d) = (num(0..4), num(5..7), num(8..10));
    if y < 1000 || !(1..=12).contains(&m) {
        return false;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days).contains(&d)
}

/// the name comparisons: through the catalog when there is one (case, accents, apostrophes and face names fold like
/// `find --name`), else by the folded spelling, a face name standing for the full name of a deck card that has it
struct Resolver {
    index: Option<Index>,
}

impl Resolver {
    fn new(index: Option<Index>) -> Resolver {
        Resolver { index }
    }

    /// The comparison key of a name and whether it resolved (always true without a catalog). `scope` is the card
    /// list a name is read against when there is no catalog: a full name of the list wins over a face name (so
    /// `Lightning Bolt` and `Emeritus of Conflict // Lightning Bolt` stay two cards, as in the catalog), else the
    /// list's card that has that face (the smallest folded name when several do: no dependence on list order).
    fn key(&self, name: &str, scope: &[String]) -> (String, bool) {
        match &self.index {
            Some(ix) => match ix.lookup(name).iter().map(|&i| ix.cards[i as usize].raw.oracle_id.as_str()).min() {
                Some(id) => (format!("id:{id}"), true),
                None => (fold(name), false),
            },
            None => {
                let f = fold(name);
                if scope.iter().any(|n| fold(n) == f) {
                    return (f, true);
                }
                let with_face = scope.iter().filter(|n| n.split(" // ").any(|face| fold(face) == f));
                (with_face.map(|n| fold(n)).min().unwrap_or(f), true)
            }
        }
    }
}

struct Entry {
    name: String,
    /// `copies` as written (None: absent, which means 1)
    copies: Option<u64>,
}

fn entries(v: Option<&toml::Value>, deck: Option<&str>, field: &str, sink: &mut Sink) -> Vec<Entry> {
    let Some(v) = v else { return vec![] };
    let place = match deck {
        Some(d) => format!("deck.{d}.{field}"),
        None => field.to_string(),
    };
    let Some(items) = v.as_array() else {
        sink.err(deck, None, format!("{place} must be a list of card names"));
        return vec![];
    };
    let mut out = vec![];
    for item in items {
        let (name, copies) = match item {
            toml::Value::String(s) => (s.clone(), None),
            toml::Value::Table(t) => {
                for k in t.keys().filter(|k| *k != "card" && *k != "copies") {
                    sink.err(deck, None, format!("{place}: unknown key {k:?} in a card table (only card and copies)"));
                }
                let Some(name) = t.get("card").and_then(|c| c.as_str()).map(String::from) else {
                    sink.err(deck, None, format!("{place}: a card table needs card = \"Name\""));
                    continue;
                };
                let copies = match t.get("copies") {
                    None => None,
                    Some(toml::Value::Integer(n)) if *n >= 1 => Some(*n as u64),
                    Some(other) => {
                        sink.err(
                            deck,
                            Some(&name),
                            format!("{place}: copies must be a whole number of 1 or more, got {other}"),
                        );
                        None
                    }
                };
                (name, copies)
            }
            other => {
                sink.err(deck, None, format!("{place}: an entry must be a name or {{ card, copies }}, got {other}"));
                continue;
            }
        };
        if name.trim().is_empty() {
            sink.err(deck, None, format!("{place}: an entry has an empty card name"));
            continue;
        }
        out.push(Entry { name, copies });
    }
    out
}

/// The `mine` decks of `data/decks.json`: key, and the names of the cards (what the file must match).
fn mine_decks(project: &Project) -> Result<Vec<(String, Vec<String>)>, String> {
    let path = project.data("decks.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mine = v["mine"].as_array().ok_or_else(|| format!("{}: no `mine` list", path.display()))?;
    mine.iter()
        .map(|d| {
            let key = d["key"].as_str().ok_or_else(|| format!("{}: a deck without a key", path.display()))?;
            let bad = |what: String| format!("{}: deck {key}: {what}", path.display());
            let cards = d["cards"].as_array().ok_or_else(|| bad("`cards` is missing or not a list".into()))?;
            let names = cards
                .iter()
                .enumerate()
                .map(|(i, c)| c.as_str().map(String::from).ok_or_else(|| bad(format!("cards[{i}] is not a string"))))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((key.to_string(), names))
        })
        .collect()
}

pub fn check(project: &Project, opts: &Options) -> Result<Report, String> {
    let path = opts.file.clone().unwrap_or_else(|| project.data("ownership.toml"));
    let decks = mine_decks(project)?;
    let mut sink = Sink::default();

    // the catalog: absent (or --data-dir without one) skips name resolution; one that cannot be read is an error
    let mut index = None;
    match knowledge_dir(project, opts.data_dir.as_deref()) {
        Ok(dir) => match Index::load(&dir.join("oracle.json")) {
            Ok(ix) => index = Some(ix),
            Err(e) => sink.err(None, None, format!("the card catalog cannot be read: {e}")),
        },
        Err(e) => {
            if opts.strict {
                sink.err(None, None, format!("--strict: no card catalog, names cannot be checked: {e}"));
            } else {
                sink.warn(
                    None,
                    None,
                    format!("name resolution skipped (names are compared by spelling against decks.json): {e}"),
                );
            }
        }
    }
    let catalog = index.is_some();
    let all_names: Vec<String> = decks.iter().flat_map(|(_, c)| c.iter().cloned()).collect();
    let res = Resolver::new(index);

    // the deck lists, as comparison key -> name, basics left out
    let lists: Vec<(&str, BTreeMap<String, String>)> = decks
        .iter()
        .map(|(k, cards)| {
            let m = cards.iter().filter(|c| !is_basic(c)).map(|c| (res.key(c, cards).0, c.clone())).collect();
            (k.as_str(), m)
        })
        .collect();

    let mut report = Report { file: path.clone(), problems: vec![], decks: vec![], spare: 0, catalog };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            sink.err(None, None, format!("{}: {e}", path.display()));
            report.problems = sink.0;
            return Ok(report);
        }
    };
    let root: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            sink.err(None, None, format!("{}: malformed TOML: {e}", path.display()));
            report.problems = sink.0;
            return Ok(report);
        }
    };

    for k in root.keys().filter(|k| !["version", "updated", "deck", "spare"].contains(&k.as_str())) {
        sink.err(None, None, format!("unknown top-level key {k:?} (version, updated, deck, spare)"));
    }
    match root.get("version") {
        Some(toml::Value::Integer(1)) => {}
        Some(other) => sink.err(None, None, format!("version must be 1, got {other}")),
        None => sink.err(None, None, "version = 1 is missing"),
    }
    match root.get("updated") {
        Some(toml::Value::String(s)) if valid_date(s) => {}
        Some(toml::Value::String(s)) => sink.err(None, None, format!("updated {s:?} is not a YYYY-MM-DD date")),
        Some(other) => sink.err(None, None, format!("updated must be a quoted \"YYYY-MM-DD\" string, got {other}")),
        None => sink.err(None, None, "updated = \"YYYY-MM-DD\" is missing"),
    }

    let empty = toml::Table::new();
    let sections = match root.get("deck") {
        None => &empty,
        Some(toml::Value::Table(t)) => t,
        Some(other) => {
            sink.err(None, None, format!("deck must be a table of [deck.KEY] sections, got {other}"));
            &empty
        }
    };
    for key in sections.keys().filter(|k| !decks.iter().any(|(d, _)| d == *k)) {
        sink.err(Some(key), None, "no such deck in data/decks.json `mine`");
    }

    // card key -> its name and, per deck that has it, the `copies` written there (for the second-copies checks)
    type HaveIn = BTreeMap<String, (String, Vec<(String, Option<u64>)>)>;
    let mut have_in: HaveIn = BTreeMap::new();
    for ((key, list), (_, deck_names)) in lists.iter().zip(&decks) {
        let mut line = DeckLine { key: key.to_string(), listed: false, have: 0, proxy: 0, missing: 0, confirmed: None };
        let Some(section) = sections.get(*key) else {
            sink.err(Some(key), None, "the deck is in data/decks.json `mine` but has no [deck.KEY] section");
            report.decks.push(line);
            continue;
        };
        let Some(section) = section.as_table() else {
            sink.err(Some(key), None, format!("deck.{key} must be a table, got {section}"));
            report.decks.push(line);
            continue;
        };
        line.listed = true;
        for k in section.keys().filter(|k| *k != "confirmed" && !STATUSES.contains(&k.as_str())) {
            sink.err(Some(key), None, format!("unknown key {k:?} in deck.{key} (confirmed, have, proxy, missing)"));
        }
        match section.get("confirmed") {
            None => {}
            Some(toml::Value::String(s)) if valid_date(s) => line.confirmed = Some(s.clone()),
            Some(toml::Value::String(s)) => {
                sink.err(Some(key), None, format!("confirmed {s:?} is not a YYYY-MM-DD date"))
            }
            Some(other) => {
                sink.err(Some(key), None, format!("confirmed must be a quoted \"YYYY-MM-DD\" string, got {other}"))
            }
        }
        if line.confirmed.is_none() && !section.contains_key("confirmed") {
            sink.warn(Some(key), None, "unconfirmed: treat as unknown (JD has not confirmed this deck)");
        }

        // key -> (status, name) for every entry of the deck
        let mut seen: BTreeMap<String, Vec<(&str, String)>> = BTreeMap::new();
        for status in STATUSES {
            let es = entries(section.get(status), Some(key), status, &mut sink);
            match status {
                "have" => line.have = es.len(),
                "proxy" => line.proxy = es.len(),
                _ => line.missing = es.len(),
            }
            for e in es {
                if is_basic(&e.name) {
                    sink.err(Some(key), Some(&e.name), format!("a basic land is never listed ({status})"));
                    continue;
                }
                let (k, resolved) = res.key(&e.name, deck_names);
                if !resolved {
                    sink.err(Some(key), Some(&e.name), format!("not found in the card catalog ({status})"));
                }
                if status == "have" {
                    let slot = have_in.entry(k.clone()).or_insert_with(|| (e.name.clone(), vec![]));
                    if !slot.1.iter().any(|(d, _)| d == key) {
                        slot.1.push((key.to_string(), e.copies));
                    }
                } else if e.copies.is_some_and(|c| c > 1) {
                    sink.err(
                        Some(key),
                        Some(&e.name),
                        format!("copies on a {status} card must be 1 or absent (copies only confirm a have card held in several decks)"),
                    );
                }
                seen.entry(k).or_default().push((status, e.name));
            }
        }
        for (_, uses) in &seen {
            let name = &uses[0].1;
            let kinds: BTreeSet<&str> = uses.iter().map(|(s, _)| *s).collect();
            for s in &kinds {
                let n = uses.iter().filter(|(t, _)| t == s).count();
                if n > 1 {
                    sink.err(Some(key), Some(name), format!("listed {n} times in {s} (use copies for more than one)"));
                }
            }
            if kinds.len() > 1 {
                let all = kinds.iter().copied().collect::<Vec<_>>().join(" and ");
                sink.err(Some(key), Some(name), format!("has two statuses ({all}); a card has one status per deck"));
            }
        }
        for (k, name) in list {
            if !seen.contains_key(k) {
                sink.err(Some(key), Some(name), "in the deck list (data/decks.json) but not in the ownership file");
            }
        }
        for (k, uses) in &seen {
            if !list.contains_key(k) {
                sink.err(
                    Some(key),
                    Some(&uses[0].1),
                    "in the ownership file but not in the deck list (data/decks.json)",
                );
            }
        }
        report.decks.push(line);
    }

    // a have card in k decks needs k copies: confirmed when every deck's entry says copies = k
    for (name, uses) in have_in.values() {
        let k = uses.len() as u64;
        let names = uses.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>().join(", ");
        for (d, c) in uses {
            match c {
                Some(c) if (k == 1 && *c > 1) || (k > 1 && *c != k) => {
                    let msg = if k == 1 {
                        format!("copies = {c} but only this deck has it (copies must be 1 or absent)")
                    } else {
                        format!("copies = {c} but {k} decks have it ({names}): copies must equal {k}")
                    };
                    sink.err(Some(d), Some(name), msg);
                }
                _ => {}
            }
        }
        if k > 1 && !uses.iter().all(|(_, c)| *c == Some(k)) {
            sink.warn(
                None,
                Some(name),
                format!("has status have in {k} decks ({names}): confirm {k} copies (write copies = {k} in each)"),
            );
        }
    }

    // spare: owned cards in no deck
    match root.get("spare") {
        None => {}
        Some(toml::Value::Table(t)) => {
            for k in t.keys().filter(|k| *k != "cards") {
                sink.err(None, None, format!("unknown key {k:?} in [spare] (only cards)"));
            }
            let mut seen: BTreeSet<String> = BTreeSet::new();
            let mut spare_overflow = false;
            for e in entries(t.get("cards"), None, "spare.cards", &mut sink) {
                match report.spare.checked_add(e.copies.unwrap_or(1)) {
                    Some(total) => report.spare = total,
                    None => {
                        report.spare = u64::MAX;
                        if !spare_overflow {
                            spare_overflow = true;
                            sink.err(None, Some(&e.name), "the spare copies add up to more than 2^64 - 1");
                        }
                    }
                }
                if is_basic(&e.name) {
                    sink.err(None, Some(&e.name), "a basic land is never listed (spare)");
                    continue;
                }
                let (k, resolved) = res.key(&e.name, &all_names);
                if !resolved {
                    sink.err(None, Some(&e.name), "not found in the card catalog (spare)");
                }
                if !seen.insert(k.clone()) {
                    sink.err(None, Some(&e.name), "listed twice in spare (use copies for more than one)");
                }
                let owners: Vec<&str> = lists.iter().filter(|(_, l)| l.contains_key(&k)).map(|(d, _)| *d).collect();
                if !owners.is_empty() {
                    sink.info(
                        None,
                        Some(&e.name),
                        format!("second copy: also in the deck list of {}; confirm", owners.join(", ")),
                    );
                }
            }
        }
        Some(other) => sink.err(None, None, format!("spare must be a table with cards = [...], got {other}")),
    }

    report.problems = sink.0;
    Ok(report)
}

/// `cardgen ownership ...`: the text to print and whether the check passed
pub fn command(project: &Project, args: &[String]) -> Result<(String, bool), String> {
    let mut opts = Options::default();
    let (mut check_flag, mut json_flag) = (false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value\n{USAGE}"));
        match a.as_str() {
            "--check" => check_flag = true,
            "--json" => json_flag = true,
            "--strict" => opts.strict = true,
            "--file" => opts.file = Some(PathBuf::from(value("--file")?)),
            "--data-dir" => opts.data_dir = Some(PathBuf::from(value("--data-dir")?)),
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    if !check_flag {
        return Err(USAGE.to_string());
    }
    let r = check(project, &opts)?;
    Ok((if json_flag { r.json() } else { r.text() }, r.ok()))
}
