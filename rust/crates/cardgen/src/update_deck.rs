//! `cardgen deck KEY LIST`: replace a deck's list with a new one (Python's `update_deck.py`). Paste the 100 cards and
//! the deck file's card sections are rewritten.
//!
//! ```text
//! cardgen deck sauron new-list.txt            # from a file
//! cardgen deck sauron -                       # paste the list, then Ctrl-D
//! cardgen deck sauron new-list.txt --dry-run  # show what would change, write nothing
//! cardgen deck sauron new-list.txt --log "Why I made these swaps."
//! ```
//!
//! The deck is one of your deck keys (data/my_decks.toml), a pool deck key, or a path to a deck .md file. The list can
//! be copied from most deck sites: "1 Card Name", "1x Card Name" or just "Card Name" per line. Set codes and collector
//! numbers ("(MOM) 123 *F*") don't change the list; for your decks they are kept in data/printings/<key>.txt, so
//! practice mode shows those exact cards. Foil marks, blank lines and headers such as "Commander" or "Deck" are ignored.
//! Anything after a "Sideboard" or "Maybeboard" header is ignored. The commander must be in the list.
//!
//! What it does:
//!  1. checks the list: 100 cards, singleton, the commander present, every name found on Scryfall, colour identity,
//!     bans, and (pool decks) the tier's Game Changer rule; any problem stops it before a file is touched;
//!  2. rewrites '## Import list' and the card lines of '## Decklist by type' (keeping the file's own groups and any
//!     notes after them), and the Game Changers / Lands lines of a pool deck's header;
//!  3. with --log, adds a dated line naming the cards out and in: to decklists/CHANGELOG.md (under the deck's title) for
//!     your decks, to decklists/pool/CHANGELOG.md for a pool deck (deck files describe only the current list);
//!  4. for your decks, records the new list in tests/fixtures/my_decks_parsed.json (the deck guard test);
//!  5. reports what changed, how completely the simulator models the new cards, and where the rest of the file still
//!     mentions cards that left (strategy text isn't rewritten);
//!  6. for your decks, when the list has set codes, writes data/printings/<key>.txt (even if no card changed).

use crate::audit::{self, Audit, cut};
use crate::data::{Project, read};
use crate::decks::{self, BASICS, py_repr};
use crate::export::Inputs;
use crate::json::{self, J};
use crate::re::{findall, rmatch, search, sub};
use crate::scryfall;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use unicode_normalization::{UnicodeNormalization, char::canonical_combining_class};

/// the groups of '## Decklist by type', in the order a new group is placed
const GROUPS: [&str; 10] = [
    "Commander",
    "Creatures",
    "Planeswalkers",
    "Battles",
    "Enchantments",
    "Artifacts",
    "Instants and sorceries",
    "Sorceries",
    "Instants",
    "Lands",
];

// ------------------------------------------------------------------ reading the list
/// [(count, name)] from a pasted list
pub fn parse_list(text: &str) -> Vec<(usize, String)> {
    let mut out = vec![];
    for raw in text.lines() {
        let mut line = raw.trim().to_string();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if rmatch(r"(?i)^(sideboard|maybeboard|considering|tokens?)\b", &line).is_some() {
            break;
        }
        line = sub(r"\s+\*[A-Za-z]+\*", "", &line); // foil / etched marks
        line = sub(r"\s+\([A-Za-z0-9]{2,6}\)(\s+\S+)?\s*$", "", &line); // (SET) 123, also 73★ and 177†
        line = sub(r"\s+\[[^\]]*\]\s*$", "", &line); // [SET]
        match rmatch(r"(?i)^(\d+)\s*x?\s+(.+)$", &line) {
            Some(m) => out.push((m.gs(1).parse().unwrap_or(1), m.gs(2).trim().to_string())),
            None => {
                if rmatch(r"(?i)^(commander|commanders|deck|main ?deck|mainboard|companion|about|name)\b.*$", &line)
                    .is_some()
                {
                    continue;
                }
                out.push((1, line));
            }
        }
    }
    out
}

/// [(count, name, "set/number")] for the lines of a pasted list that name a printing ("1x Sol Ring (sld) 2093 *F*")
pub fn parse_printings(text: &str) -> Vec<(usize, String, String)> {
    let mut out = vec![];
    for raw in text.lines() {
        let line = sub(r"\s+\*[A-Za-z]+\*", "", raw.trim());
        if rmatch(r"(?i)^(sideboard|maybeboard|considering|tokens?)\b", &line).is_some() {
            break;
        }
        if let Some(m) = rmatch(r"^(\d+)\s*x?\s+(.+?)\s+\(([A-Za-z0-9]{2,6})\)\s+(\S+)\s*$", &line) {
            let print = format!("{}/{}", m.gs(3).to_lowercase(), m.gs(4));
            out.push((m.gs(1).parse().unwrap_or(1), m.gs(2).trim().to_string(), print));
        }
    }
    out
}

/// data/printings/<key>.txt: the printing of every copy, "1 Sol Ring (sld) 2093" per line, under the names the deck
/// file uses (practice mode shows these images)
pub fn printings_text(prints: &[(usize, String, String)], canon: &BTreeMap<String, String>) -> String {
    let mut lines: Vec<String> = prints
        .iter()
        .map(|(k, n, p)| {
            let (set, num) = p.split_once('/').unwrap_or((p, ""));
            format!("{k} {} ({set}) {num}", canon.get(n).unwrap_or(n))
        })
        .collect();
    lines.sort_by_key(|l| plain(l.split_once(' ').map(|x| x.1).unwrap_or(l)));
    lines.join("\n") + "\n"
}

/// for comparing and sorting names: no accents, case or punctuation ("Mauhur Uruk-hai Captain" matches)
pub fn plain(s: &str) -> String {
    let s: String = s.nfkd().filter(|c| canonical_combining_class(*c) == 0).collect::<String>().to_lowercase();
    sub(r"[^\w/ ]", " ", &s).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// the name to write for what was asked, or None when Scryfall found a different card. A two-faced card keeps the
/// form the simulator already knows it by (its hand-written rules are registered under that name); a new card gets
/// Scryfall's full name.
pub fn canonical(requested: &str, rec: &J, known: &dyn Fn(&str) -> bool) -> Option<String> {
    let full = rec.s_or("name").to_string();
    let front = full.split(" // ").next().unwrap_or("").to_string();
    let mut names = vec![full.clone(), front.clone()];
    if let Some(f) = rec.get("card_faces").and_then(|f| f.as_arr()) {
        names.extend(f.iter().map(|x| x.s_or("name").to_string()));
    }
    if !names.iter().any(|n| plain(n) == plain(requested)) {
        return None;
    }
    for n in [requested, full.as_str(), front.as_str()] {
        if known(n) {
            return Some(n.to_string());
        }
    }
    Some(full)
}

// ------------------------------------------------------------------ the deck file
/// a `**Commander (1).**` or `**Commanders (2).**` line
fn is_cmd_line(l: &str) -> bool {
    l.starts_with("**Commander (1).**") || l.starts_with("**Commanders (2).**")
}

/// the second commander: one of your two-commander decks', or a pool partner pair's (`- **Commander:** A / B`, as
/// `decks::load_pool_deck` reads it; a double-faced card's `A // B` has no spaced single slash)
pub fn commander2_of(inp: &Inputs, path: &Path, text: &str) -> Option<String> {
    let base = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    if let Some(d) = inp.my.iter().find(|d| Path::new(&d.file).file_name().is_some_and(|f| f.to_string_lossy() == base)) {
        return d.commander2.clone();
    }
    pool_commander_line(text).and_then(|l| split_pair(&l).1)
}

/// a pool deck's commander line as its first and second commander (`A / B`; `A // B` is one double-faced card)
pub fn split_pair(line: &str) -> (String, Option<String>) {
    match line.split_once(" / ") {
        Some((a, b)) => (a.trim().to_string(), Some(b.trim().to_string())),
        None => (line.trim().to_string(), None),
    }
}

/// a pool deck's `- **Commander:**` line
fn pool_commander_line(text: &str) -> Option<String> {
    search(r"(?m)^- \*\*Commander:\*\*\s*(.+?)\s*$", text).map(|m| m.gs(1).to_string())
}

pub fn commander_of(inp: &Inputs, path: &Path, text: &str) -> Result<String, String> {
    let base = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    for d in &inp.my {
        if Path::new(&d.file).file_name().is_some_and(|f| f.to_string_lossy() == base) {
            return Ok(d.commander.clone()); // your decks: the commander the simulator plays
        }
    }
    if let Some(l) = pool_commander_line(text) {
        // a partner pair's first commander (commander2_of gives the second)
        return Ok(split_pair(&l).0);
    }
    if let Some(m) = search(r"(?m)^\*\*Commander \(1\)\.\*\*\s*(.+?)\s*$", text) {
        return Ok(m.gs(1).to_string());
    }
    Err(format!("{}: no commander line found", path.display()))
}

/// the '## Decklist by type' group a card goes in, among the labels the file uses
fn group_of(rec: &J, labels: &[String]) -> String {
    let tl = rec.s("type_line").unwrap_or("").split(" // ").next().unwrap_or("").to_string();
    let has = |w: &str| tl.contains(w);
    if has("Land") {
        "Lands".into()
    } else if has("Creature") {
        "Creatures".into()
    } else if has("Planeswalker") {
        "Planeswalkers".into()
    } else if has("Battle") {
        "Battles".into()
    } else if has("Instant") || has("Sorcery") {
        if labels.iter().any(|l| l == "Instants and sorceries") {
            "Instants and sorceries".into()
        } else if has("Instant") {
            "Instants".into()
        } else {
            "Sorceries".into()
        }
    } else if has("Enchantment") {
        "Enchantments".into()
    } else if has("Artifact") {
        "Artifacts".into()
    } else {
        "Other".into()
    }
}

#[derive(Clone, Copy)]
pub enum SortKey {
    Plain0,
    Lower,
    Plain,
}

impl SortKey {
    fn key(self, s: &str) -> String {
        match self {
            SortKey::Plain0 => s.to_string(),
            SortKey::Lower => s.to_lowercase(),
            SortKey::Plain => plain(s),
        }
    }
}

/// the order the file's import list already uses (plain order, or ignoring case / punctuation)
pub fn sort_key(old_singles: &[String]) -> SortKey {
    for k in [SortKey::Plain0, SortKey::Lower, SortKey::Plain] {
        if old_singles.windows(2).all(|w| k.key(&w[0]) <= k.key(&w[1])) {
            return k;
        }
    }
    SortKey::Plain
}

fn sorted_by(mut v: Vec<String>, key: SortKey) -> Vec<String> {
    v.sort_by_key(|a| key.key(a));
    v
}

/// the new file text
#[allow(clippy::too_many_arguments)]
pub fn rewrite(
    text: &str,
    commander: &str,
    commander2: Option<&str>,
    counts: &[(String, usize)],
    recs: &BTreeMap<String, J>,
    by_type_commas: bool,
    today: &str,
    log_note: Option<&str>,
    key: SortKey,
) -> Result<String, String> {
    let count_of = |n: &str| counts.iter().find(|(x, _)| x == n).map(|(_, k)| *k).unwrap_or(0);
    let singles =
        sorted_by(counts.iter().filter(|(n, _)| !BASICS.contains(&n.as_str())).map(|(n, _)| n.clone()).collect(), key);
    let import = r"(?ms)^## Import list \(\d+\)\s*\n\s*```\n(.*?)\n```";
    let m = search(import, text);
    let was: Vec<String> = m
        .as_ref()
        .map(|m| {
            m.gs(1).split('\n').filter(|l| l.contains(' ')).map(|l| l.split_once(' ').unwrap().1.to_string()).collect()
        })
        .unwrap_or_default();
    let mut basics: Vec<String> =
        counts.iter().filter(|(n, _)| BASICS.contains(&n.as_str())).map(|(n, _)| n.clone()).collect();
    basics.sort_by_key(|n| (was.iter().position(|w| w == n).unwrap_or(was.len()), n.clone()));
    let total: usize = counts.iter().map(|(_, k)| k).sum();
    // import list
    let block: String =
        singles.iter().chain(basics.iter()).map(|n| format!("{} {n}", count_of(n))).collect::<Vec<_>>().join("\n");
    let m = search(import, text).ok_or("the file has no \"## Import list\" block")?;
    let mut text = format!("{}## Import list ({total})\n\n```\n{block}\n```{}", &text[..m.start()], &text[m.end()..]);
    // decklist by type: replace the bold group lines, keep everything else in the section
    if let Some(m) = search(r"(?ms)^## Decklist by type \(\d+\)[ \t]*\n(.*?)(?=^## )", &text) {
        let body = m.gs(1).to_string();
        let lines: Vec<&str> = body.split('\n').collect();
        let bold: Vec<usize> =
            (0..lines.len()).filter(|i| rmatch(r"^\*\*[^*]+ \(\d+\)\.\*\*", lines[*i]).is_some()).collect();
        let labels: Vec<String> =
            bold.iter().map(|i| rmatch(r"^\*\*(.+?) \(\d+\)\.\*\*", lines[*i]).unwrap().gs(1).to_string()).collect();
        let mut groups: Vec<(String, Vec<String>)> = vec![];
        for n in singles.iter().chain(basics.iter()) {
            if n == commander || Some(n.as_str()) == commander2 {
                continue;
            }
            let g = group_of(&recs[n], &labels);
            match groups.iter_mut().find(|(k, _)| *k == g) {
                Some((_, v)) => v.push(n.clone()),
                None => groups.push((g, vec![n.clone()])),
            }
        }
        let show = |n: &str| if by_type_commas { n.to_string() } else { n.replace(',', "") };
        let lands_lines =
            bold.iter().filter(|i| lines[**i].contains("Lands")).map(|i| lines[*i]).collect::<Vec<_>>().join(" ");
        let seen = findall(r"\b\d+ ([A-Z][\w-]*(?: [A-Z][\w-]*)*)", &lands_lines);
        let basic_rank = |n: &String| (seen.iter().position(|s| s == n).unwrap_or(seen.len()), n.clone());
        let has_group = |g: &str| groups.iter().any(|(k, _)| k == g);
        let mut order: Vec<String> = labels.iter().filter(|l| *l != "Commander").cloned().collect();
        order.extend(GROUPS.iter().filter(|g| has_group(g) && !labels.iter().any(|l| l == *g)).map(|g| g.to_string()));
        let mut rest_groups: Vec<String> =
            groups.iter().map(|(k, _)| k.clone()).filter(|g| !GROUPS.contains(&g.as_str())).collect();
        rest_groups.sort();
        order.extend(rest_groups);
        let cmd_line = lines.iter().find(|l| is_cmd_line(l)).map(|l| l.to_string());
        let mut new = vec![cmd_line.unwrap_or(match commander2 {
            Some(c2) => format!("**Commanders (2).** {commander}, {c2}"),
            None => format!("**Commander (1).** {commander}"),
        })]; // an existing line is kept as written
        for g in &order {
            let Some((_, cards)) = groups.iter().find(|(k, _)| k == g) else { continue };
            if cards.is_empty() {
                continue;
            }
            let ones: Vec<String> = cards.iter().filter(|n| !BASICS.contains(&n.as_str())).map(|n| show(n)).collect();
            let mut bs: Vec<&String> = cards.iter().filter(|n| BASICS.contains(&n.as_str())).collect();
            bs.sort_by_key(|n| basic_rank(n));
            let many: Vec<String> = bs.iter().map(|n| format!("{} {n}", count_of(n))).collect();
            let k: usize = cards.iter().map(|n| count_of(n)).sum();
            new.push(format!("**{g} ({k}).** {}", ones.into_iter().chain(many).collect::<Vec<_>>().join(", ")));
        }
        let (first, rest_from) = match (bold.first(), bold.last()) {
            (Some(f), Some(l)) => (*f, l + 1),
            _ => (0, 0),
        };
        let mut body2: Vec<String> = lines[..first].iter().map(|s| s.to_string()).collect();
        body2.extend(new.join("\n\n").split('\n').map(|s| s.to_string()));
        body2.extend(lines[rest_from.min(lines.len())..].iter().map(|s| s.to_string()));
        text = format!("{}## Decklist by type ({total})\n{}{}", &text[..m.start()], body2.join("\n"), &text[m.end()..]);
    }
    // a pool deck's header lines
    let gcs = sorted_by(
        counts
            .iter()
            .filter(|(n, _)| recs[n].get("game_changer").is_some_and(|g| g.truthy()))
            .map(|(n, _)| n.clone())
            .collect(),
        key,
    );
    if let Some(m) = search(r"(?m)^- \*\*Game Changers \(\d+\):\*\*\s*(.*)$", &text) {
        let claimed: usize = search(r"\((\d+)\)", m.all()).unwrap().gs(1).parse().unwrap_or(0);
        let same = claimed == gcs.len()
            && gcs.iter().all(|n| m.gs(1).contains(n.as_str()) || m.gs(1).contains(n.split(" // ").next().unwrap()));
        if !same {
            // only when the set changed
            let list = if gcs.is_empty() { "none".to_string() } else { gcs.join(", ") };
            text = format!("{}- **Game Changers ({}):** {list}{}", &text[..m.start()], gcs.len(), &text[m.end()..]);
        }
    }
    let lands: Vec<&String> = counts
        .iter()
        .map(|(n, _)| n)
        .filter(|n| recs[*n].s("type_line").unwrap_or("").split(" // ").next().unwrap_or("").contains("Land"))
        .collect();
    let nb: usize = lands.iter().filter(|n| !BASICS.contains(&n.as_str())).map(|n| count_of(n)).sum();
    let b: usize = lands.iter().filter(|n| BASICS.contains(&n.as_str())).map(|n| count_of(n)).sum();
    text = sub(r"(?m)^- \*\*Lands:\*\*.*$", &format!("- **Lands:** {} ({nb} nonbasic + {b} basic)", nb + b), &text);
    if let Some(note) = log_note {
        text = add_log(&text, today, note);
    }
    Ok(text)
}

/// an '**Updated <date>.**' line after the last one near the top of the file (or after the title)
pub fn add_log(text: &str, today: &str, line: &str) -> String {
    let entry = format!("**Updated {today}.** {line}").trim_end().to_string();
    let mut lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
    let top = lines.iter().position(|l| l.starts_with("## ")).unwrap_or(lines.len());
    let ups: Vec<usize> = (0..top).filter(|i| lines[*i].starts_with("**Updated ")).collect();
    let at = ups.last().map(|l| l + 1).unwrap_or(1).min(lines.len());
    if ups.is_empty() {
        lines.splice(at..at, [String::new(), entry]);
    } else {
        lines.insert(at, entry);
    }
    lines.join("\n")
}

/// the history of your decks, under decklists/
pub const CHANGELOG: &str = "CHANGELOG.md";

/// a '- **<date>.** <line>' bullet at the top of the deck's '### Changes' list in decklists/CHANGELOG.md (the section
/// is the deck file's title); a missing section, Changes list or file is made
pub fn add_changelog(log: &str, title: &str, key: &str, file: &str, today: &str, line: &str) -> String {
    let entry = format!("- **{today}.** {line}").trim_end().to_string();
    let mut lines: Vec<String> = log.trim_end().split('\n').map(|s| s.to_string()).collect();
    if log.trim().is_empty() {
        lines = vec!["# Deck changelog".into()];
    }
    let head = format!("## {title}");
    let Some(at) = lines.iter().position(|l| l.trim_end() == head) else {
        lines.extend([
            String::new(),
            head,
            String::new(),
            format!("Deck key `{key}`, `decklists/{file}`."),
            String::new(),
            "### Changes".into(),
            String::new(),
            entry,
        ]);
        return lines.join("\n") + "\n";
    };
    let end = (at + 1..lines.len()).find(|i| lines[*i].starts_with("## ")).unwrap_or(lines.len());
    match (at + 1..end).find(|i| lines[*i].trim_end() == "### Changes") {
        Some(c) => {
            let first = (c + 1..end).find(|i| !lines[*i].trim().is_empty()).unwrap_or(end);
            if first == end || !lines[first].starts_with("- ") {
                lines.splice(first..first, [entry, String::new()]);
            } else {
                lines.insert(first, entry);
            }
        }
        None => {
            let first = (at + 1..end).find(|i| lines[*i].starts_with("### ")).unwrap_or(end);
            lines.splice(first..first, ["### Changes".into(), String::new(), entry, String::new()]);
        }
    }
    lines.join("\n") + "\n"
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn deck_file(inp: &Inputs, which: &str) -> Result<PathBuf, String> {
    let p = Path::new(which);
    if p.exists() {
        return std::fs::canonicalize(p).map_err(|e| e.to_string());
    }
    if let Some(d) = inp.my.iter().find(|d| d.key == which) {
        return Ok(decks::deck_path(&inp.project, d));
    }
    inp.pool.iter().find(|d| d.key == which).map(|d| d.path.clone()).ok_or_else(|| {
        format!(
            "No deck {which:?}: use {}, a pool deck key, or a path to a deck file",
            inp.my.iter().map(|d| d.key.as_str()).collect::<Vec<_>>().join(", ")
        )
    })
}

/// Everything `cardgen deck` prints (stdout) and whether it succeeded; lets the tests run it on a copy.
pub struct Outcome {
    pub text: String,
    pub ok: bool,
}

pub struct Options {
    pub dry: bool,
    pub log: Option<String>,
    pub offline: bool,
}

pub fn run(project: &Project, which: &str, list_text: &str, opts: &Options) -> Result<Outcome, String> {
    let mut out = String::new();
    macro_rules! say {
        ($($a:tt)*) => {{ out.push_str(&format!($($a)*)); out.push('\n'); }};
    }
    let mut inp = Inputs::load(project)?;
    let path = deck_file(&inp, which)?;
    let text = read(&path)?;
    let commander = commander_of(&inp, &path, &text)?;
    let commander2 = commander2_of(&inp, &path, &text);
    let entries = parse_list(list_text);
    if entries.is_empty() {
        return Err("the list is empty".into());
    }
    if !opts.offline {
        let mut names: Vec<String> = entries.iter().map(|(_, n)| n.clone()).collect();
        names.push(commander.clone());
        names.extend(commander2.clone());
        scryfall::fetch(&mut inp.cache, &names, false);
        inp.cache.save()?;
    }
    let old = decks::load(&path)?;
    let built = inp.build();
    let known = |n: &str| old.iter().any(|o| o == n) || built.cards.contains_key(n);
    let mut problems = vec![];
    let mut counts: Vec<(String, usize)> = vec![];
    let mut recs: BTreeMap<String, J> = BTreeMap::new();
    let mut canon: BTreeMap<String, String> = BTreeMap::new(); // name as pasted -> name as written
    for (_, n) in &entries {
        if let Some(r) = inp.cache.get(n) {
            recs.insert(n.clone(), r.clone());
        }
    }
    if let Some(r) = inp.cache.get(&commander) {
        recs.insert(commander.clone(), r.clone());
    }
    if let Some(c2) = &commander2
        && let Some(r) = inp.cache.get(c2)
    {
        recs.insert(c2.clone(), r.clone());
    }
    for (k, n) in &entries {
        let Some(rec) = inp.cache.get(n).cloned() else {
            problems.push(format!("not found on Scryfall: {n}"));
            continue;
        };
        // the form the deck file already uses first ("Blazing Firesinger" for "Blazing Firesinger // Seething Song")
        let in_old = |x: &str| old.iter().any(|o| o == x);
        let Some(name) = canonical(n, &rec, &in_old).filter(|c| in_old(c)).or_else(|| canonical(n, &rec, &known))
        else {
            problems.push(format!(
                "{} matched a different card on Scryfall ({}): check the spelling",
                py_repr(n),
                rec.s_or("name")
            ));
            continue;
        };
        match counts.iter_mut().find(|(x, _)| *x == name) {
            Some((_, c)) => *c += k,
            None => counts.push((name.clone(), *k)),
        }
        canon.insert(n.clone(), name.clone());
        recs.insert(name, rec);
    }
    let in_pool = path.to_string_lossy().contains(&format!("{0}pool{0}", std::path::MAIN_SEPARATOR));
    let tier = if in_pool {
        path.parent()
            .and_then(|d| d.file_name())
            .map(|s| s.to_string_lossy().split('-').next().unwrap_or("").to_string())
    } else {
        None
    };
    let cards: Vec<String> = counts.iter().flat_map(|(n, k)| std::iter::repeat_n(n.clone(), *k)).collect();
    let cmds: Vec<&str> = std::iter::once(commander.as_str()).chain(commander2.as_deref()).collect();
    problems.extend(decks::structural_problems(&cards, &cmds, 100)); // every problem at once, not only the first
    problems.extend(decks::card_problems(&cards, &cmds, tier.as_deref(), None, &|n| recs.get(n).cloned()).0);
    if !problems.is_empty() {
        say!("Not updated: the new list for {} has problems:", path.file_name().unwrap().to_string_lossy());
        for p in &problems {
            say!("  - {p}");
        }
        return Ok(Outcome { text: out, ok: false });
    }
    let count_of = |n: &str| counts.iter().find(|(x, _)| x == n).map(|(_, k)| *k).unwrap_or(0);
    let old_count = |n: &str| old.iter().filter(|o| *o == n).count();
    let mut uniq_old: Vec<String> = old.clone();
    uniq_old.sort();
    uniq_old.dedup();
    let out_cards = sorted_by(uniq_old.iter().filter(|n| count_of(n) == 0).cloned().collect(), SortKey::Plain);
    let in_cards =
        sorted_by(counts.iter().filter(|(n, _)| old_count(n) == 0).map(|(n, _)| n.clone()).collect(), SortKey::Plain);
    let moved = sorted_by(
        uniq_old.iter().filter(|n| count_of(n) > 0 && old_count(n) != count_of(n)).cloned().collect(),
        SortKey::Plain,
    );
    let by_type = search(r"(?ms)^## Decklist by type.*?(?=^## )", &text);
    let typed: String = by_type
        .map(|m| m.all().split('\n').filter(|l| !l.starts_with("**Commander")).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    let commas = uniq_old.iter().any(|n| {
        n.contains(',') && typed.contains(n.as_str()) && *n != commander && Some(n.as_str()) != commander2.as_deref()
    });
    let today = today();
    let note = opts.log.as_ref().map(|l| {
        let join = |v: &Vec<String>| if v.is_empty() { "nothing".to_string() } else { v.join(", ") };
        format!(
            "Out: {}. In: {}.{}",
            join(&out_cards),
            join(&in_cards),
            if l.is_empty() { String::new() } else { format!(" {l}") }
        )
    });
    let singles_old: Vec<String> =
        old.iter().filter(|n| !BASICS.contains(&n.as_str()) && old_count(n) == 1).cloned().collect();
    let key = sort_key(&singles_old);
    // deck files describe the deck as it is: your decks keep their history in decklists/CHANGELOG.md, pool decks in
    // decklists/pool/CHANGELOG.md; only a deck file elsewhere gets its log line at its own top
    let my_deck = inp
        .my
        .iter()
        .find(|d| std::fs::canonicalize(decks::deck_path(project, d)).ok().as_deref() == Some(path.as_path()))
        .cloned();
    let file_note = if my_deck.is_some() || in_pool { None } else { note.as_deref() };
    let new_text = rewrite(&text, &commander, commander2.as_deref(), &counts, &recs, commas, &today, file_note, key)?;

    let rel = path
        .strip_prefix(&project.root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path.to_string_lossy().to_string());
    say!(
        "{rel}: {} out, {} in{}",
        out_cards.len(),
        in_cards.len(),
        if moved.is_empty() { String::new() } else { format!(", {} basic counts changed", moved.len()) }
    );
    if !out_cards.is_empty() {
        say!("  out: {}", out_cards.join(", "));
    }
    if !in_cards.is_empty() {
        say!("  in:  {}", in_cards.join(", "));
    }
    for n in &moved {
        say!("  {n}: {} -> {}", old_count(n), count_of(n));
    }
    let gcs: Vec<&String> =
        counts.iter().map(|(n, _)| n).filter(|n| recs[*n].get("game_changer").is_some_and(|g| g.truthy())).collect();
    let mut gcs_sorted = gcs.clone();
    gcs_sorted.sort();
    say!(
        "  {} cards, {} Game Changers{}",
        counts.iter().map(|(_, k)| k).sum::<usize>(),
        gcs.len(),
        if gcs.is_empty() {
            String::new()
        } else {
            format!(" ({})", gcs_sorted.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
        }
    );
    // the printings, when the list names them (your decks only): written even when the cards are unchanged
    let prints = parse_printings(list_text);
    if let (Some(d), false) = (&my_deck, prints.is_empty()) {
        let p = project.data(&format!("printings/{}.txt", d.key));
        let new_prints = printings_text(&prints, &canon);
        if std::fs::read_to_string(&p).ok().as_deref() != Some(new_prints.as_str()) {
            let copies: usize = prints.iter().map(|x| x.0).sum();
            if opts.dry {
                say!("(--dry-run: would write data/printings/{}.txt, {copies} cards with a printing)", d.key);
            } else {
                std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
                std::fs::write(&p, new_prints).map_err(|e| format!("{}: {e}", p.display()))?;
                say!("Wrote data/printings/{}.txt ({copies} cards with a printing).", d.key);
            }
        }
    }
    if new_text == text {
        say!("Nothing to change.");
        return Ok(Outcome { text: out, ok: true });
    }
    if opts.dry {
        say!("\n(--dry-run: nothing written)");
    } else {
        std::fs::write(&path, &new_text).map_err(|e| format!("{}: {e}", path.display()))?;
        say!("Wrote {rel}.");
    }
    // (changelog under decklists/, deck key, deck file under decklists/, fallback title)
    let log_to: Option<(String, String, String, String)> = match &my_deck {
        Some(d) => Some((CHANGELOG.into(), d.key.clone(), d.file.clone(), d.name.clone())),
        None if in_pool => {
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            Some((format!("pool/{CHANGELOG}"), stem.clone(), rel.trim_start_matches("decklists/").to_string(), stem))
        }
        None => None,
    };
    if let (Some((log_rel, key, file, name)), Some(note)) = (&log_to, &note) {
        let log_path = project.decklists().join(log_rel);
        let log = if log_path.exists() { read(&log_path)? } else { String::new() };
        let title = text.split('\n').find_map(|l| l.strip_prefix("# ")).unwrap_or(name).trim();
        let new_log = add_changelog(&log, title, key, file, &today, note);
        if opts.dry {
            say!("(--dry-run: would add to decklists/{log_rel}: - **{today}.** {note})");
        } else {
            std::fs::write(&log_path, new_log).map_err(|e| format!("{}: {e}", log_path.display()))?;
            say!("Logged the change in decklists/{log_rel}.");
        }
    }
    // how well the new cards are modeled, and where the file still mentions cards that left
    let inp2 = Inputs::load(project)?; // the lists as written now (the cache was saved above)
    let mine = inp2.my.iter().any(|d| Path::new(&d.file).file_name() == path.file_name());
    let real = inp2.my.iter().any(|d| decks::deck_path(project, d) == path) || project_deck(&inp2, &path);
    let built2 = crate::sources::build(&inp2.carddb, &inp2.overrides, &inp2.cache, &{
        let mut w = inp2.wanted();
        w.extra.extend(in_cards.iter().cloned()); // a --dry-run list isn't in the files yet
        w
    });
    let a = Audit::new(&inp2, &built2.cards)?;
    if !in_cards.is_empty() {
        say!("\nHow the simulator models the new cards:");
        for n in &in_cards {
            let (s, note) = a.card_status(n, mine);
            say!(
                "  {:<11} {n}{}",
                s,
                if s != "Full" && !note.is_empty() { format!(": {}", cut(&note, 110)) } else { String::new() }
            );
        }
        if in_cards.iter().any(|n| audit::rank(&a.card_status(n, mine).0) < audit::rank("Full-auto")) {
            say!(
                "  Cards below Full-auto play weaker (or stronger) than the real card; worth modeling before measuring."
            );
        }
    }
    let kept: Vec<String> = if path.exists() { decks::load(&path).unwrap_or_default() } else { vec![] };
    let cmd_line = text.split('\n').find(|l| is_cmd_line(l)).unwrap_or("").to_string();
    let mut hits: Vec<(usize, String, String)> = vec![];
    for (i, line) in text.split('\n').enumerate() {
        if rmatch(r"^(## (Import list|Decklist by type)|\*\*[^*]+ \(\d+\)\.\*\*|\d+ )", line).is_some() {
            continue;
        }
        for n in &out_cards {
            let short = n.split(" // ").next().unwrap().split(',').next().unwrap();
            // a short name ("Kefka") only counts when no card still in the deck, and not the commander, shares it
            let short_ok =
                short.chars().count() >= 6 && !cmd_line.contains(short) && !kept.iter().any(|k| k.contains(short));
            if line.contains(n.as_str()) || line.contains(&n.replace(',', "")) || (short_ok && line.contains(short)) {
                hits.push((i + 1, n.clone(), line.trim().to_string()));
            }
        }
    }
    if !hits.is_empty() {
        say!("\nThe rest of the file still mentions cards that left (strategy text is not rewritten):");
        for (i, n, line) in hits.iter().take(25) {
            say!("  line {i}: {n}: {}", cut(line, 100));
        }
        if hits.len() > 25 {
            say!("  ... and {} more", hits.len() - 25);
        }
    }
    if real && !opts.dry {
        record_fixture(&inp2)?;
        say!("\nRecorded the new list for the deck guard (tests/fixtures/my_decks_parsed.json).");
    }
    if !opts.dry {
        say!("Next: cargo test --release   then rebuild the data (cardgen export) and measure it.");
    }
    Ok(Outcome { text: out, ok: true })
}

fn project_deck(inp: &Inputs, path: &Path) -> bool {
    inp.my.iter().any(|d| std::fs::canonicalize(inp.project.decklists().join(&d.file)).ok().as_deref() == Some(path))
}

/// tests/fixtures/my_decks_parsed.json: every one of your decks' cards, sorted (the deck guard test)
pub fn fixture_json(inp: &Inputs) -> Result<String, String> {
    let mut o = crate::json::Obj::new();
    for d in &inp.my {
        let mut cards = decks::load(&decks::deck_path(&inp.project, d))?;
        cards.sort();
        o.set(&d.key, J::Arr(cards.into_iter().map(J::from).collect()));
    }
    Ok(json::dumps(&J::Obj(o), Some(0), true))
}

fn record_fixture(inp: &Inputs) -> Result<(), String> {
    let path = inp.project.root.join("tests").join("fixtures").join("my_decks_parsed.json");
    std::fs::write(&path, fixture_json(inp)?).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{add_changelog, parse_list, parse_printings, printings_text, split_pair};
    use std::collections::BTreeMap;

    #[test]
    fn a_pool_partner_line_is_two_commanders() {
        let pair = |a: &str, b: Option<&str>| (a.to_string(), b.map(String::from));
        assert_eq!(split_pair("Kraum, Ludevic's Opus / Tymna the Weaver"), pair("Kraum, Ludevic's Opus", Some("Tymna the Weaver")));
        assert_eq!(split_pair("Kinnan, Bonder Prodigy"), pair("Kinnan, Bonder Prodigy", None));
        // a double-faced card's name is one commander
        let esika = "Esika, God of the Tree // The Prismatic Bridge";
        assert_eq!(split_pair(esika), pair(esika, None));
    }

    #[test]
    fn printings_are_kept_under_the_deck_files_names() {
        let list = "1x Sol Ring (sld) 2093 *F*\n6x Island (mdd) 294\n1x Blazing Firesinger // Seething Song (sos) 109\n\
                    1x Sheoldred, Whispering One (pnph) 73★ *F*\n1x Big Score (plst) SNC-102\n1 Arcane Signet\n";
        // the list itself is names and counts only
        assert_eq!(parse_list(list)[2], (1, "Blazing Firesinger // Seething Song".to_string()));
        assert_eq!(parse_list(list)[3], (1, "Sheoldred, Whispering One".to_string())); // "73★" is a number too
        let p = parse_printings(list);
        assert_eq!(p.len(), 5); // Arcane Signet names no printing
        assert_eq!(p[0], (1, "Sol Ring".to_string(), "sld/2093".to_string()));
        assert_eq!(p[3].2, "pnph/73★");
        let canon =
            BTreeMap::from([("Blazing Firesinger // Seething Song".to_string(), "Blazing Firesinger".to_string())]);
        assert_eq!(
            printings_text(&p, &canon),
            "1 Big Score (plst) SNC-102\n1 Blazing Firesinger (sos) 109\n6 Island (mdd) 294\n\
             1 Sheoldred, Whispering One (pnph) 73★\n1 Sol Ring (sld) 2093\n"
        );
    }

    #[test]
    fn changelog_entries_go_first_in_the_decks_changes_list() {
        // no file yet: header, then the deck's section
        let a = add_changelog("", "Kefka — Discard", "kefka", "JD/k.md", "2026-10-14", "Out: A. In: B.");
        assert_eq!(
            a,
            "# Deck changelog\n\n## Kefka — Discard\n\nDeck key `kefka`, `decklists/JD/k.md`.\n\n### Changes\n\n- **2026-10-14.** Out: A. In: B.\n"
        );
        // the newest entry goes first; other decks' sections are untouched
        let b = add_changelog(&a, "Sauron — Amass", "sauron", "JD/s.md", "2026-10-14", "Out: C. In: D.");
        let c = add_changelog(&b, "Kefka — Discard", "kefka", "JD/k.md", "2026-10-15", "Out: B. In: A.");
        let kefka = c.split("## Kefka — Discard\n").nth(1).unwrap().split("\n## ").next().unwrap();
        assert!(
            kefka.contains("### Changes\n\n- **2026-10-15.** Out: B. In: A.\n- **2026-10-14.** Out: A. In: B."),
            "{c}"
        );
        assert!(c.ends_with("### Changes\n\n- **2026-10-14.** Out: C. In: D.\n"), "{c}");
        // a section without a Changes list gets one, before its other subsections
        let d = add_changelog(
            "# Deck changelog\n\n## X\n\nIntro.\n\n### Tuning notes\n\nNote.\n",
            "X",
            "x",
            "JD/x.md",
            "2026-10-15",
            "Out: E. In: F.",
        );
        assert_eq!(
            d,
            "# Deck changelog\n\n## X\n\nIntro.\n\n### Changes\n\n- **2026-10-15.** Out: E. In: F.\n\n### Tuning notes\n\nNote.\n"
        );
    }
}
