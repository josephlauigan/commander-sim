//! What the setup screen shows (Python's `play/catalog.py`): your decks (commander, bracket, Game Changers, how they
//! do in the simulations) and the five tiers (their decks). Read from the card data folder (`decks.json`,
//! `my_decks.toml`, `scryfall_cache.json`) and, when the repository's `decklists/` folder sits next to it, from the
//! deck files (the bracket line, the commander's display name, the tier folders' names) and the look-ahead results
//! table `decklists/pool/pool-results.md` (section 0d). Without `decklists/` (the iPad app) the bracket is guessed
//! from the Game Changer count, the name is the commander and there are no win rates.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const TIERS: [&str; 6] = ["t0", "t1", "t2", "t3", "t4", "t5"];
/// the decks the setup screen offers, in the order of Python's `poolmode.MINE` (error messages list them so)
pub const MY_DECKS: [&str; 7] = ["seph", "veyran", "sauron", "galadriel", "yshtola", "alela", "jodah"];

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// `(key, deck file under decklists/)` of your decks in the order of `my_decks.toml` (Python's `decks.FILES`)
fn my_deck_files(data: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = vec![];
    for line in read(&data.join("my_decks.toml")).unwrap_or_default().lines() {
        let line = line.trim();
        if let Some(k) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            out.push((k.to_string(), String::new()));
        } else if let Some(rest) = line.strip_prefix("file")
            && let Some(v) = rest.trim_start().strip_prefix('=')
            && let Some(last) = out.last_mut()
        {
            last.1 = v.trim().trim_matches('"').to_string();
        }
    }
    out
}

/// the commander as the deck file names it: `**Commander (1).** Name (anything)`
pub fn display_name(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("**Commander (1).**") {
            let mut s = rest.trim();
            if s.ends_with(')')
                && let Some(i) = s.find('(')
            {
                s = s[..i].trim_end();
            }
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// the bracket the deck file states (`Bracket 3`, possibly inside `**`)
pub fn stated_bracket(text: &str) -> Option<u8> {
    text.lines().find_map(|l| {
        let rest = l.trim_start_matches('*').strip_prefix("Bracket ")?;
        let d = rest.chars().next()?;
        d.to_digit(10).map(|d| d as u8)
    })
}

/// the first `12.3%` in a table cell
fn percent(cell: &str) -> Option<f64> {
    let b = cell.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if c == b'%' {
            let mut j = i;
            while j > 0 && (b[j - 1].is_ascii_digit() || b[j - 1] == b'.') {
                j -= 1;
            }
            if j < i {
                return cell[j..i].parse().ok();
            }
        }
    }
    None
}

/// `{deck key: [win % per tier]}` from section 0d of pool-results.md; the first row of a deck wins
pub fn win_rates(text: &str) -> BTreeMap<String, Vec<f64>> {
    let mut out = BTreeMap::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("### 0d.") {
            inside = true;
            continue;
        }
        if inside && line.starts_with('#') {
            break;
        }
        if !inside {
            continue;
        }
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').map(str::trim).collect();
        let key = match cells[0].to_lowercase().as_str() {
            "sephiroth" => "seph",
            "veyran" => "veyran",
            "sauron" => "sauron",
            "galadriel" => "galadriel",
            "y'shtola" | "yshtola" => "yshtola",
            "alela" => "alela",
            "jodah" => "jodah",
            "kefka" => "kefka",
            "titania" => "titania",
            "sheoldred" => "sheoldred",
            "cloud" => "cloud",
            _ => continue,
        };
        if cells.len() < 6 {
            continue;
        }
        let rates: Vec<f64> = cells[1..6].iter().filter_map(|c| percent(c)).collect();
        if rates.len() == 5 && !out.contains_key(key) {
            out.insert(key.to_string(), rates);
        }
    }
    out
}

/// `T1 · High B2 / Low B3` from the tier's folder name (`t1-high-b2-low-b3`)
pub fn tier_label(tier: &str, folder: Option<&str>) -> String {
    let words: Vec<String> = folder
        .map(|f| {
            f.split('-')
                .skip(1)
                .map(|w| {
                    let b = w.as_bytes();
                    if b.len() == 2 && b[0] == b'b' && b[1].is_ascii_digit() {
                        w.to_uppercase()
                    } else {
                        let mut c = w.chars();
                        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    format!("{} · {}", tier.to_uppercase(), words.join(" ").replace(" Low", " / Low"))
}

fn short_name(commander: &str) -> &str {
    commander.split(" // ").next().unwrap_or("").split(',').next().unwrap_or("")
}

/// `round(x, 1)` the way Python prints it
fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Game Changer card names (lower case) from the card cache
fn game_changer_set(data: &Path) -> BTreeSet<String> {
    let Some(v) = read(&data.join("scryfall_cache.json")).and_then(|t| serde_json::from_str::<Value>(&t).ok()) else {
        return BTreeSet::new();
    };
    v.as_object()
        .map(|m| m.iter().filter(|(_, c)| c["game_changer"].as_bool() == Some(true)).map(|(k, _)| k.clone()).collect())
        .unwrap_or_default()
}

/// the whole catalog: `{decks: [...], tiers: [...]}`, the same JSON as Python's `catalog.catalog()`
pub fn catalog(data: &Path) -> Result<Value, String> {
    let root: PathBuf = data.join("..");
    let decks_file: Value = serde_json::from_str(&read(&data.join("decks.json")).ok_or("data/decks.json is missing")?)
        .map_err(|e| format!("data/decks.json: {e}"))?;
    let by_key =
        |list: &str| -> Vec<&Value> { decks_file[list].as_array().map(|a| a.iter().collect()).unwrap_or_default() };
    let mine = by_key("mine");
    let gc = game_changer_set(data);
    let rates = read(&root.join("decklists/pool/pool-results.md")).map(|t| win_rates(&t)).unwrap_or_default();
    let mut decks = vec![];
    for (key, file) in my_deck_files(data) {
        let Some(d) = mine.iter().find(|d| d["key"] == key.as_str()) else { continue };
        let cards: BTreeSet<&str> = d["cards"].as_array().into_iter().flatten().filter_map(|c| c.as_str()).collect();
        let gcs: Vec<&str> = cards.iter().copied().filter(|c| gc.contains(&c.trim().to_lowercase())).collect();
        let text = read(&root.join("decklists").join(&file)).unwrap_or_default();
        let commander = d["commander"].as_str().unwrap_or("");
        let bracket = stated_bracket(&text).unwrap_or(if gcs.len() > 3 { 4 } else { 3 });
        let r = rates.get(&key);
        decks.push(json!({
            "key": key, "commander": commander, "name": display_name(&text).unwrap_or(commander.to_string()),
            "bracket": bracket, "game_changers": gcs,
            "win_rates": r.map(|r| TIERS[1..].iter().zip(r).map(|(t, v)| (t.to_string(), json!(v))).collect::<serde_json::Map<_, _>>()),
            "average": r.map(|r| round1(r.iter().sum::<f64>() / r.len() as f64)),
        }));
    }
    let pool = by_key("pool");
    let folders: Vec<String> = std::fs::read_dir(root.join("decklists/pool"))
        .map(|rd| rd.flatten().filter_map(|e| e.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    let tiers: Vec<Value> = TIERS
        .iter()
        .map(|&t| {
            let mut ds: Vec<&&Value> = pool.iter().filter(|d| d["tier"] == t).collect();
            ds.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
            let folder = folders.iter().find(|f| f.starts_with(&format!("{t}-")));
            json!({
                "key": t, "label": tier_label(t, folder.map(String::as_str)),
                "decks": ds.iter().map(|d| json!({
                    "key": d["key"], "commander": d["commander"],
                    "name": short_name(d["commander"].as_str().unwrap_or("")),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(json!({"decks": decks, "tiers": tiers}))
}

/// every commander on the setup screen (for their images)
pub fn commanders(cat: &Value) -> Vec<String> {
    let mut out: Vec<String> = cat["decks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| d["commander"].as_str())
        .map(String::from)
        .collect();
    for t in cat["tiers"].as_array().into_iter().flatten() {
        out.extend(
            t["decks"].as_array().into_iter().flatten().filter_map(|d| d["commander"].as_str()).map(String::from),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(tier_label("t1", Some("t1-high-b2-low-b3")), "T1 · High B2 / Low B3");
        assert_eq!(tier_label("t2", Some("t2-mid-b3")), "T2 · Mid B3");
        assert_eq!(tier_label("t2", None), "T2 · ");
    }

    #[test]
    fn deck_file_lines() {
        assert_eq!(
            display_name("x\n**Commander (1).** Sephiroth, the Savior (Atraxa)\n").unwrap(),
            "Sephiroth, the Savior"
        );
        assert_eq!(stated_bracket("a\n**Bracket 4 (optimized)**\n"), Some(4));
        assert_eq!(stated_bracket("no bracket here"), None);
    }

    #[test]
    fn results_table() {
        let t = "### 0d. Look-ahead\n| Deck | T1 | T2 | T3 | T4 | T5 |\n| Sephiroth | 39.7% (a) | 51.2% | 38.5% | 33.9% | 22.2% |\n\
                 | Sephiroth | 1% | 1% | 1% | 1% | 1% |\n| Alela | 10% | 20% | x | 1% | 2% |\n## next\n| Jodah | 1% | 1% | 1% | 1% | 1% |\n";
        let r = win_rates(t);
        assert_eq!(r["seph"], vec![39.7, 51.2, 38.5, 33.9, 22.2]);
        assert!(!r.contains_key("alela") && !r.contains_key("jodah"));
    }
}
