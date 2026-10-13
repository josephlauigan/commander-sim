//! Deck lists: your decks (`decklists/<owner>/<deck>.md`, named in data/my_decks.toml) and the outside decks
//! (`decklists/pool/<tier folder>/<deck>.md`, the five power tiers). A deck file's cards are its `## Import list`
//! block ("1 Sol Ring" lines); a pool deck's commander is its `**Commander:**` line.

use crate::data::{MyDeck, Project, read};
use crate::json::J;
use crate::py;
use crate::re::{rmatch, search};
use crate::scryfall::Cache;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// the cards of a deck file (each listed name repeated by its count)
pub fn load(path: &Path) -> Result<Vec<String>, String> {
    let txt = read(path)?;
    let bad = || format!("{}: no '## Import list' block", path.display());
    let after = txt.split("## Import list").nth(1).ok_or_else(bad)?;
    let block = after.split("```").nth(1).ok_or_else(bad)?;
    let mut out = vec![];
    for line in block.trim().lines() {
        let (n, name) = line.split_once(' ').ok_or(format!("{}: bad line {line:?}", path.display()))?;
        let n: usize = n.parse().map_err(|_| format!("{}: bad count in {line:?}", path.display()))?;
        for _ in 0..n {
            out.push(name.trim().to_string());
        }
    }
    Ok(out)
}

/// where one of your deck files is: $SIM_DECKS (flat, or with the owner folders), then decklists/
pub fn deck_path(p: &Project, d: &MyDeck) -> PathBuf {
    let file = Path::new(&d.file);
    let base = file.file_name().map(PathBuf::from).unwrap_or_default();
    if let Ok(env) = std::env::var("SIM_DECKS") {
        if !env.is_empty() {
            for c in [Path::new(&env).join(&base), Path::new(&env).join(file)] {
                if c.exists() {
                    return c;
                }
            }
        }
    }
    p.decklists().join(file)
}

#[derive(Debug, Clone)]
pub struct PoolDeck {
    pub path: PathBuf,
    pub tier: String,
    pub key: String,
    pub commander: String,
    pub cards: Vec<String>,
    /// the file's first line without its '#'
    pub title: String,
    /// the Game Changer count the file's header claims
    pub gc_claimed: Option<usize>,
}

/// all pool decks, sorted by path (tier, then file name)
pub fn load_pool(p: &Project) -> Result<Vec<PoolDeck>, String> {
    let mut paths = vec![];
    let dir = p.decklists().join("pool");
    for tier in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let tier = tier.map_err(|e| e.to_string())?.path();
        let fname = tier.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if !tier.is_dir() || !fname.starts_with('t') {
            continue;
        }
        for f in std::fs::read_dir(&tier).map_err(|e| e.to_string())? {
            let f = f.map_err(|e| e.to_string())?.path();
            if f.extension().is_some_and(|e| e == "md") {
                paths.push(f);
            }
        }
    }
    paths.sort_by_key(|a| a.to_string_lossy().to_string());
    paths.iter().map(|f| load_pool_deck(f)).collect()
}

pub fn load_pool_deck(path: &Path) -> Result<PoolDeck, String> {
    let folder = path.parent().and_then(|d| d.file_name()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let tier = folder.split('-').next().unwrap_or("").to_string();
    let key = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let txt = read(path)?;
    let commander = search(r"(?m)^- \*\*Commander:\*\*\s*(.+?)\s*$", &txt)
        .map(|m| m.gs(1).to_string())
        .ok_or(format!("{}: no **Commander:** line", path.display()))?;
    let title = py::strip(txt.lines().next().unwrap_or(""), "# ").to_string();
    let gc_claimed = search(r"\*\*Game Changers \((\d+)\)", &txt).and_then(|m| m.gs(1).parse().ok());
    Ok(PoolDeck { path: path.to_path_buf(), tier, key, commander, cards: load(path)?, title, gc_claimed })
}

/// the name logs use for a pool deck: its commander's first name part
pub fn short_name(commander: &str) -> String {
    commander.split(" // ").next().unwrap_or("").split(',').next().unwrap_or("").to_string()
}

/// `n Card Name` -> (n, name)
pub fn parse_line(line: &str) -> Option<(usize, String)> {
    let m = rmatch(r"^(\d+)\s+(.+?)\s*$", line.trim())?;
    Some((m.gs(1).parse().ok()?, m.gs(2).to_string()))
}

pub const BASICS: [&str; 11] = [
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
];

/// Game Changer rules per tier (decklists/pool/README.md): (min, max); None = unrestricted
pub fn gc_rules(tier: &str) -> Option<(usize, usize)> {
    match tier {
        "t0" | "t1" => Some((0, 0)),
        "t2" => Some((1, 2)),
        "t3" => Some((3, 3)),
        _ => None,
    }
}

/// Checks that need no card data: size, singleton, commander present.
pub fn structural_problems(cards: &[String], commander: &str, size: usize) -> Vec<String> {
    let mut probs = vec![];
    if cards.len() != size {
        probs.push(format!("{} cards, expected {size}", cards.len()));
    }
    if commander.is_empty() {
        probs.push("no **Commander:** line".into());
    } else if !cards.iter().any(|c| c == commander) {
        probs.push(format!("commander {} is not in the import list", py_repr(commander)));
    }
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *count.entry(c).or_default() += 1;
    }
    let dups: Vec<&str> = count.iter().filter(|(n, k)| **k > 1 && !BASICS.contains(n)).map(|(n, _)| *n).collect();
    if !dups.is_empty() {
        probs.push(format!("not singleton: {}", dups.join(", ")));
    }
    probs
}

/// Python's `repr()` of a card name (single quotes unless the name has one and no double quote)
pub fn py_repr(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// Did Scryfall return the card we asked for (not a fuzzy near-miss)?
pub fn resolved_name_ok(requested: &str, rec: &J) -> bool {
    let key = requested.trim().to_lowercase();
    let mut names = vec![rec.s_or("name").to_lowercase()];
    if let Some(f) = rec.get("card_faces").and_then(|f| f.as_arr()) {
        names.extend(f.iter().map(|x| x.s_or("name").to_lowercase()));
    }
    names.contains(&key)
}

/// what `card_problems` found out besides the problems
pub struct DeckInfo {
    pub gcs: Vec<String>,
    pub identity: String,
}

/// Checks that need Scryfall data. `recs`: requested name -> record (Python's slim records).
pub fn card_problems(
    cards: &[String],
    commander: &str,
    tier: Option<&str>,
    gc_claimed: Option<usize>,
    recs: &dyn Fn(&str) -> Option<J>,
) -> (Vec<String>, DeckInfo) {
    let mut probs = vec![];
    let mut names: Vec<&String> = cards.iter().collect();
    names.sort();
    names.dedup();
    let rec_of: Vec<(&String, Option<J>)> = names.iter().map(|n| (*n, recs(n))).collect();
    let missing: Vec<&str> = rec_of.iter().filter(|(_, r)| r.is_none()).map(|(n, _)| n.as_str()).collect();
    if !missing.is_empty() {
        probs.push(format!("not found on Scryfall: {}", missing.join(", ")));
    }
    let fuzzy: Vec<String> = rec_of
        .iter()
        .filter_map(|(n, r)| {
            r.as_ref().filter(|r| !resolved_name_ok(n, r)).map(|r| format!("{n} -> {}", r.s_or("name")))
        })
        .collect();
    if !fuzzy.is_empty() {
        probs.push(format!("resolved only by fuzzy match (check spelling): {}", fuzzy.join(", ")));
    }
    let cmd = recs(commander);
    let ident: Option<Vec<String>> =
        cmd.as_ref().map(|c| c.strs("color_identity").iter().map(|s| s.to_string()).collect());
    if let Some(c) = &cmd {
        let tl = c.s("type_line").unwrap_or("");
        let mut text = c.s("oracle_text").unwrap_or("").to_string();
        text.push(' ');
        text.push_str(
            &c.get("card_faces")
                .and_then(|f| f.as_arr())
                .map(|f| f.iter().map(|x| x.s("oracle_text").unwrap_or("").to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
        );
        if !(tl.contains("Legendary") && tl.contains("Creature")) && !text.contains("can be your commander") {
            probs.push(format!("commander {commander} is not a legendary creature"));
        }
    }
    let identity = match &ident {
        Some(i) => {
            let mut v: Vec<&str> = i.iter().map(|s| s.as_str()).collect();
            v.sort();
            let id = v.concat();
            let off: Vec<&str> = rec_of
                .iter()
                .filter(|(_, r)| {
                    r.as_ref().is_some_and(|r| !r.strs("color_identity").iter().all(|c| i.iter().any(|x| x == c)))
                })
                .map(|(n, _)| n.as_str())
                .collect();
            if !off.is_empty() {
                probs.push(format!(
                    "outside colour identity {}: {}",
                    if id.is_empty() { "C" } else { &id },
                    off.join(", ")
                ));
            }
            id
        }
        None => "?".to_string(),
    };
    let banned: Vec<&str> = rec_of
        .iter()
        .filter(|(_, r)| {
            r.as_ref().is_some_and(|r| {
                r.get("legalities").and_then(|l| l.get("commander")).and_then(|c| c.as_str()) == Some("banned")
            })
        })
        .map(|(n, _)| n.as_str())
        .collect();
    if !banned.is_empty() {
        probs.push(format!("banned in Commander: {}", banned.join(", ")));
    }
    let gcs: Vec<String> = rec_of
        .iter()
        .filter(|(_, r)| r.as_ref().is_some_and(|r| r.get("game_changer").is_some_and(|g| g.truthy())))
        .map(|(n, _)| n.to_string())
        .collect();
    if let Some((lo, hi)) = tier.and_then(gc_rules) {
        if !(lo <= gcs.len() && gcs.len() <= hi) {
            probs.push(format!("{} Game Changers, tier {} allows {lo}-{hi}", gcs.len(), tier.unwrap()));
        }
    }
    if let Some(c) = gc_claimed {
        if c != gcs.len() {
            probs.push(format!("file says {c} Game Changers, Scryfall says {}", gcs.len()));
        }
    }
    (probs, DeckInfo { gcs, identity })
}

/// `cardgen validate`: every pool deck's problems (Python's `pools --validate`)
pub fn validate_all(p: &Project, cache: &Cache) -> Result<Vec<(PoolDeck, Vec<String>, DeckInfo)>, String> {
    let mut out = vec![];
    for d in load_pool(p)? {
        let mut probs = structural_problems(&d.cards, &d.commander, 100);
        let (p2, info) = card_problems(&d.cards, &d.commander, Some(&d.tier), d.gc_claimed, &|n| cache.get(n).cloned());
        probs.extend(p2);
        out.push((d, probs, info));
    }
    Ok(out)
}
