//! The deck lists, read from the deck files as the Python harness read them (decks.py, deck_files.py, pools.py):
//! your decks from decklists/<owner>/, the opponent pools from decklists/pool/<tier folder>/. Cards are kept in the
//! file's order, because a seat's library is shuffled from that order.
//!
//! The commanders, display names and colour identities come from data/decks.json (the card export), as the engine
//! seats them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// your decks: key -> (owner folder in decklists/, file), as deck_files.FILES
pub const FILES: [(&str, &str, &str); 15] = [
    ("seph", "JD", "sephiroth-phyrexian-reanimator.md"),
    ("veyran", "JD", "veyran-izzet-spellslinger.md"),
    ("sauron", "JD", "sauron-grixis-amass.md"),
    ("yshtola", "JD", "yshtola-esper-drain.md"),
    ("galadriel", "Avery", "galadriel-bant-rebels.md"),
    ("alela", "Avery", "alela-esper-faeries.md"),
    ("kefka", "JD", "kefka-grixis-discard.md"),
    ("omnath", "JD", "omnath-landfall.md"),
    ("sheoldred", "JD", "sheoldred-mono-black.md"),
    ("cloud", "JD", "cloud-midgar-equipment.md"),
    ("thrasios", "JD", "tymna-thrasios-partners.md"),
    ("sephfs", "JD", "sephiroth-fs-wipe-steal.md"),
    ("godo", "JD", "godo-cedh-equipment.md"),
    ("karn", "JD", "karn-colorless-artifacts.md"),
    ("yshtola-cedh", "JD", "yshtola-cedh.md"),
];

/// compare.KEYS / poolmode.MINE: your decks, in the order --all-decks runs them
pub const MINE: [&str; 15] = ["seph", "veyran", "sauron", "galadriel", "yshtola", "alela", "kefka", "omnath", "sheoldred", "cloud", "thrasios", "sephfs", "godo", "karn", "yshtola-cedh"];

pub const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

pub fn tier_label(t: &str) -> &str {
    match t {
        "t0" => "Tier 0 (Bracket 2 precons)",
        "t1" => "Tier 1 (High B2 / Low B3)",
        "t2" => "Tier 2 (Mid B3)",
        "t3" => "Tier 3 (High B3)",
        "t4" => "Tier 4 (Low B4)",
        "t5" => "Tier 5 (High B4)",
        "t6" => "Tier 6 (cEDH, B5)",
        _ => t,
    }
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

/// The repository root: $COMMANDER_SIM_ROOT, else the first folder holding decklists/ and data/ above the current
/// folder, else above the program itself (rust/target/release/commander-sim).
pub fn root() -> Result<PathBuf, String> {
    if let Ok(r) = std::env::var("COMMANDER_SIM_ROOT")
        && !r.is_empty()
    {
        return Ok(PathBuf::from(r));
    }
    let is_root = |p: &Path| p.join("decklists").is_dir() && p.join("data").is_dir();
    let mut starts = vec![];
    if let Ok(d) = std::env::current_dir() {
        starts.push(d);
    }
    if let Ok(e) = std::env::current_exe() {
        starts.push(e.canonicalize().unwrap_or(e));
    }
    for s in starts {
        if let Some(p) = s.ancestors().find(|p| is_root(p)) {
            return Ok(p.to_path_buf());
        }
    }
    Err("can't find the repository (a folder with decklists/ and data/): run from inside it or set COMMANDER_SIM_ROOT"
        .into())
}

/// data/ (or $COMMANDER_SIM_DATA): cards.json and decks.json
pub fn data_dir(root: &Path) -> PathBuf {
    match std::env::var("COMMANDER_SIM_DATA") {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => root.join("data"),
    }
}

/// deck_files.deck_path: $SIM_DECKS (flat or by owner), then decklists/<owner>/, then /mnt/project
pub fn deck_path(root: &Path, key: &str) -> Option<PathBuf> {
    let &(_, owner, f) = FILES.iter().find(|x| x.0 == key)?;
    let repo = root.join("decklists").join(owner).join(f);
    let mut cands = vec![];
    if let Ok(env) = std::env::var("SIM_DECKS")
        && !env.is_empty()
    {
        cands.push(Path::new(&env).join(f));
        cands.push(Path::new(&env).join(owner).join(f));
    }
    cands.push(repo.clone());
    cands.push(Path::new("/mnt/project").join(f));
    Some(cands.into_iter().find(|c| c.exists()).unwrap_or(repo))
}

/// decks.load: the cards of a deck file's '## Import list' block ("1 Sol Ring" lines), in order
pub fn parse_import_list(txt: &str) -> Result<Vec<String>, String> {
    let after = txt.split_once("## Import list").ok_or("no '## Import list' section")?.1;
    let block = after.split("```").nth(1).ok_or("no ``` block after '## Import list'")?;
    let mut out = vec![];
    for line in block.trim().lines() {
        let (n, name) = line.split_once(' ').ok_or_else(|| format!("bad import line {line:?}"))?;
        let n: usize = n.trim().parse().map_err(|_| format!("bad count in import line {line:?}"))?;
        out.extend(std::iter::repeat_n(name.trim().to_string(), n));
    }
    Ok(out)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// your decks' lists (decks.DECKS), by key
pub fn load_mine(root: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut out = BTreeMap::new();
    for (key, ..) in FILES {
        let path = deck_path(root, key).unwrap();
        let cards = parse_import_list(&read(&path)?).map_err(|e| format!("{}: {e}", path.display()))?;
        out.insert(key.to_string(), cards);
    }
    Ok(out)
}

/// one opponent-pool deck (pools.PoolDeck)
#[derive(Debug, Clone)]
pub struct PoolDeck {
    pub tier: String,
    pub key: String,
    pub commander: String,
    pub cards: Vec<String>,
}

impl PoolDeck {
    pub fn parse(path: &Path) -> Result<PoolDeck, String> {
        let txt = read(path)?;
        let folder = path.parent().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().to_string());
        let folder = folder.unwrap_or_default();
        let tier = folder.split('-').next().unwrap_or("").to_string();
        let key = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let commander = txt
            .lines()
            .find_map(|l| l.strip_prefix("- **Commander:**"))
            .map(|c| c.split_once(" / ").map_or(c, |(a, _)| a).trim().to_string()) // partner pair: the first (decks.json has both)
            .filter(|c| !c.is_empty())
            .ok_or_else(|| format!("{}: no '- **Commander:**' line", path.display()))?;
        let cards = parse_import_list(&txt).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(PoolDeck { tier, key, commander, cards })
    }

    /// pools.short_name: 'Teysa Karlov', 'Isshin'
    pub fn short_name(&self) -> String {
        short_name(&self.commander)
    }
}

pub fn short_name(commander: &str) -> String {
    commander.split(" // ").next().unwrap_or("").split(',').next().unwrap_or("").to_string()
}

/// pools.load_pool(): every deck under decklists/pool/t*/, sorted by path (tier, then file name)
pub fn load_pool(root: &Path) -> Result<Vec<PoolDeck>, String> {
    let dir = root.join("decklists").join("pool");
    let mut paths = vec![];
    let tiers = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for t in tiers.flatten() {
        let tp = t.path();
        let name = t.file_name().to_string_lossy().to_string();
        if !name.starts_with('t') || !tp.is_dir() {
            continue;
        }
        for f in std::fs::read_dir(&tp).map_err(|e| format!("{}: {e}", tp.display()))?.flatten() {
            let fp = f.path();
            if fp.extension().is_some_and(|x| x == "md") && !f.file_name().to_string_lossy().starts_with('.') {
                paths.push(fp);
            }
        }
    }
    paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    paths.iter().map(|p| PoolDeck::parse(p)).collect()
}

/// pools.structural_problems: size, singleton, commander present
pub fn structural_problems(cards: &[String], commander: &str, size: usize) -> Vec<String> {
    let mut probs = vec![];
    if cards.len() != size {
        probs.push(format!("{} cards, expected {size}", cards.len()));
    }
    if commander.is_empty() {
        probs.push("no **Commander:** line".into());
    } else if !cards.iter().any(|c| c == commander) {
        probs.push(format!("commander '{commander}' is not in the import list"));
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *counts.entry(c).or_default() += 1;
    }
    let dups: Vec<&str> = counts.iter().filter(|(n, k)| **k > 1 && !BASICS.contains(n)).map(|(n, _)| *n).collect();
    if !dups.is_empty() {
        probs.push(format!("not singleton: {}", dups.join(", ")));
    }
    probs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_list_keeps_order_and_counts() {
        let txt = "# X\n\n## Import list\n\n```\n1 Sol Ring\n3 Island\n1 Counterspell\n```\n\n## Notes\n```\n1 Nope\n```";
        assert_eq!(parse_import_list(txt).unwrap(), ["Sol Ring", "Island", "Island", "Island", "Counterspell"]);
    }

    #[test]
    fn short_names() {
        assert_eq!(short_name("Teysa Karlov"), "Teysa Karlov");
        assert_eq!(short_name("Isshin, Two Heavens as One"), "Isshin");
        assert_eq!(short_name("Esika, God of the Tree // The Prismatic Bridge"), "Esika");
    }

    #[test]
    fn the_repository_decks_load() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mine = load_mine(&root).unwrap();
        assert_eq!(mine.len(), 15);
        for (k, v) in &mine {
            assert_eq!(v.len(), 100, "{k}");
        }
        let pool = load_pool(&root).unwrap();
        assert_eq!(pool.len(), 36);
        for t in TIERS {
            assert_eq!(pool.iter().filter(|d| d.tier == t).count(), if t == "t6" { 6 } else { 5 }, "{t}");
        }
        for d in &pool {
            assert!(structural_problems(&d.cards, &d.commander, 100).is_empty(), "{}", d.key);
        }
    }
}
