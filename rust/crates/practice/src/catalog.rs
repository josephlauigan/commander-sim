//! What the setup screen shows (Python's `play/catalog.py`): your decks (commander, bracket, Game Changers, how they
//! do in the simulations) and the five tiers (their decks). Read from the card data folder (`decks.json`,
//! `my_decks.toml`, `scryfall_cache.json`) and, when the repository's `decklists/` folder sits next to it, from the
//! deck files (the bracket line, the commander's display name, the tier folders' names) and the look-ahead results
//! table `decklists/pool/pool-results.md` (section 0d). Without `decklists/` (the iPad app) the bracket is guessed
//! from the Game Changer count, the name is the commander and there are no win rates.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];
/// the decks the setup screen offers, in the order of data/my_decks.toml (error messages list them so)
pub const MY_DECKS: [&str; 11] = [
    "seph",
    "veyran",
    "sauron",
    "yshtola",
    "galadriel",
    "alela",
    "kefka",
    "sheoldred",
    "cloud",
    "thrasios",
    "karn",
];

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

/// the commander as the deck file names it: `**Commander (1).** Name (anything)`; a Partner deck names both,
/// `**Commanders (2).** Name, Name2` (shown as "Name and Name2", the short names when it is long)
pub fn display_name(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("**Commanders (2).**") {
            let s = rest.trim();
            if let Some((a, b)) = s.split_once(", ")
                && !a.is_empty()
            {
                // "Tymna the Weaver, Kraum, Ludevic's Opus": the first name has no comma of its own here
                return Some(format!("{} and {}", a, short_name(b)));
            }
        }
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
            "kefka" => "kefka",
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

/// a pool deck's name: "Rograkh", or "Rograkh and Silas Renn" for a partner pair
fn pool_deck_name(c1: &str, c2: Option<&str>) -> String {
    match c2 {
        Some(b) => format!("{} and {}", short_name(c1), short_name(b)),
        None => short_name(c1).to_string(),
    }
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
            "key": key, "commander": commander, "commander2": d.get("commander2").cloned().unwrap_or(Value::Null),
            "commanders": std::iter::once(commander).chain(d["commander2"].as_str()).collect::<Vec<_>>(),
            "ident": d.get("ident").cloned().unwrap_or(Value::Null), "name": display_name(&text).unwrap_or(commander.to_string()),
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
                "decks": ds.iter().map(|d| {
                    let file = folder.map(|f| root.join("decklists/pool").join(f).join(format!("{}.md", d["key"].as_str().unwrap_or(""))));
                    let c1 = d["commander"].as_str().unwrap_or("");
                    let c2 = d["commander2"].as_str();
                    json!({
                        "key": d["key"], "commander": d["commander"], "commander2": d.get("commander2").cloned().unwrap_or(Value::Null),
                        "commanders": std::iter::once(c1).chain(c2).collect::<Vec<_>>(),
                        "name": pool_deck_name(c1, c2),
                        "summary": file.and_then(|f| read(&f)).map(|t| pool_summary(&t)),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(json!({"decks": decks, "tiers": tiers}))
}

/// the text of a `## <heading>` section, up to the next `## `
fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let start = text.find(&format!("\n## {heading}\n"))? + heading.len() + 5;
    let rest = &text[start..];
    Some(rest[..rest.find("\n## ").unwrap_or(rest.len())].trim())
}

/// a pool deck's box on the setup screen, from its file: the title, its axis, Game Changers and combos (header
/// lines), and the four "At the table" bullets (plan, wins by, watch out for, interaction) as `[label, text]`; a deck
/// without that section gives the first paragraph of its Strategy instead
pub fn pool_summary(text: &str) -> Value {
    let title = text.lines().next().and_then(|l| l.strip_prefix("# ")).unwrap_or("").trim();
    let axis = text.lines().find_map(|l| l.split_once("Axis: ").map(|(_, a)| a.trim().trim_end_matches('.')));
    let bullet = |label: &str| {
        text.lines().find_map(|l| l.strip_prefix(&format!("- **{label}")).map(|r| r.trim_start_matches(':').trim()))
    };
    let gcs = bullet("Game Changers").map(|g| g.split_once(":** ").map(|x| x.1).unwrap_or(g).trim());
    let combos = bullet("Combos:**").map(str::trim);
    let table: Vec<Value> = section(text, "At the table")
        .map(|s| {
            s.lines()
                .filter_map(|l| l.strip_prefix("- **")?.split_once(":** "))
                .map(|(k, v)| json!([k, v.trim()]))
                .collect()
        })
        .unwrap_or_default();
    let strategy = if table.is_empty() {
        section(text, "Strategy").and_then(|s| s.split("\n\n").next()).map(str::trim)
    } else {
        None
    };
    json!({"title": title, "axis": axis, "game_changers": gcs, "combos": combos, "table": table, "strategy": strategy})
}

/// every commander on the setup screen (for their images)
pub fn commanders(cat: &Value) -> Vec<String> {
    let mut out: Vec<String> = cat["decks"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|d| d["commanders"].as_array().into_iter().flatten().filter_map(|c| c.as_str()))
        .map(String::from)
        .collect();
    for t in cat["tiers"].as_array().into_iter().flatten() {
        out.extend(
            t["decks"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|d| d["commanders"].as_array().into_iter().flatten().filter_map(|c| c.as_str()))
                .map(String::from),
        );
    }
    let mut seen = BTreeSet::new();
    out.retain(|n| seen.insert(n.clone())); // partners share a commander
    out
}

/// the printings of your decks' commanders (data/printings/<key>.txt), for the setup screen's pictures
pub fn commander_printings(data: &Path, cat: &Value) -> crate::images::Printings {
    let decks: Vec<&Value> = cat["decks"].as_array().into_iter().flatten().collect();
    let keys: Vec<String> = decks.iter().filter_map(|d| d["key"].as_str()).map(String::from).collect();
    let mut all = crate::images::load_printings(data, &keys);
    for d in decks {
        let cmds: Vec<&str> = d["commanders"].as_array().into_iter().flatten().filter_map(|c| c.as_str()).collect();
        if let Some(m) = d["key"].as_str().and_then(|k| all.get_mut(k)) {
            m.retain(|n, _| cmds.contains(&n.as_str()));
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pool_decks_box_comes_from_its_file() {
        let head = "# Atraxa — Superfriends\n\nOpponent pool deck — **Tier 3**. Axis: Planeswalkers / proliferate.\n\n\
                    - **Commander:** Atraxa\n- **Game Changers (2):** Rhystic Study, Smothering Tithe\n- **Combos:** None.\n\n\
                    ## Role in the pool\n\nPressure.\n\n";
        let s = pool_summary(&format!(
            "{head}## At the table\n\n- **Plan:** Walkers behind taxes.\n- **Wins by:** Ultimates.\n\n\
             ## Strategy\n\nOld text.\n\nMore.\n\n## Import list (100)\n\n1 Atraxa\n"
        ));
        assert_eq!(s["title"], "Atraxa — Superfriends");
        assert_eq!(s["axis"], "Planeswalkers / proliferate");
        assert_eq!(s["game_changers"], "Rhystic Study, Smothering Tithe");
        assert_eq!(s["combos"], "None.");
        assert_eq!(s["table"], json!([["Plan", "Walkers behind taxes."], ["Wins by", "Ultimates."]]));
        assert_eq!(s["strategy"], Value::Null);
        // no "At the table" yet: the Strategy's first paragraph
        let s = pool_summary(&format!("{head}## Strategy\n\nOld text.\n\nMore.\n\n## Import list (100)\n"));
        assert_eq!((s["table"].clone(), s["strategy"].clone()), (json!([]), json!("Old text.")));
    }

    #[test]
    fn partner_pool_decks_are_named_and_pictured_with_both_commanders() {
        assert_eq!(
            pool_deck_name("Rograkh, Son of Rohgahh", Some("Silas Renn, Seeker Adept")),
            "Rograkh and Silas Renn"
        );
        assert_eq!(pool_deck_name("Kraum, Ludevic's Opus", Some("Tymna the Weaver")), "Kraum and Tymna the Weaver");
        assert_eq!(pool_deck_name("Atraxa, Grand Unifier", None), "Atraxa");
        let cat = json!({"decks": [{"commanders": ["A", "B"]}], "tiers": [{"decks": [{"commanders": ["Rograkh, Son of Rohgahh", "Thrasios, Triton Hero"]}, {"commanders": ["C"]}]}]});
        assert_eq!(commanders(&cat), ["A", "B", "Rograkh, Son of Rohgahh", "Thrasios, Triton Hero", "C"]);
    }

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
                 | Sephiroth | 1% | 1% | 1% | 1% | 1% |\n| Alela | 10% | 20% | x | 1% | 2% |\n## next\n| Najeela | 1% | 1% | 1% | 1% | 1% |\n";
        let r = win_rates(t);
        assert_eq!(r["seph"], vec![39.7, 51.2, 38.5, 33.9, 22.2]);
        assert!(!r.contains_key("alela") && !r.contains_key("najeela"));
    }
}
