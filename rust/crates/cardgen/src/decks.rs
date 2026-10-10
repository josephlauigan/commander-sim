//! Deck lists: your decks (`decklists/<owner>/<deck>.md`, named in data/my_decks.toml) and the outside decks
//! (`decklists/pool/<tier folder>/<deck>.md`, the five power tiers). A deck file's cards are its `## Import list`
//! block ("1 Sol Ring" lines); a pool deck's commander is its `**Commander:**` line.

use crate::data::{MyDeck, Project, read};
use crate::re::{rmatch, search};
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
    Ok(PoolDeck { path: path.to_path_buf(), tier, key, commander, cards: load(path)? })
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
