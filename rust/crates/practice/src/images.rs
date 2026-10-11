//! Card images for the browser table (Python's `play/images.py`): found on Scryfall and downloaded once into
//! `<data>/images/` (not in git), only for the cards of the decks in this game and the tokens those cards make.
//!
//! `index.json` maps a card name (lower case) to `{id, name, urls, files, tokens, type_line, ...}` and `token:<id>` to
//! a token's entry, as Python's does, so images already downloaded keep working. The card cache is left alone.
//! Requests follow Scryfall's guidelines: a User-Agent, 100 ms between requests, lookups of 75 cards per request, then
//! one request per image not yet on disk. A card whose image can't be had is drawn by the page from its name; the
//! game starts either way. The base URL is `PRACTICE_SCRYFALL` when set (the tests run against a local fake).

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const SIZE: &str = "normal"; // 488x680
const USER_AGENT: &str = "commander-pod-sim/1.0 (personal deck testing)";

fn key(name: &str) -> String {
    name.trim().to_lowercase()
}

fn base_url() -> String {
    std::env::var("PRACTICE_SCRYFALL").unwrap_or_else(|_| "https://api.scryfall.com".into())
}

/// one request every 100 ms at most, across threads
fn wait() {
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(t) = *last {
        let dt = t.elapsed();
        if dt < Duration::from_millis(100) {
            std::thread::sleep(Duration::from_millis(100) - dt);
        }
    }
    *last = Some(Instant::now());
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build()
}

/// The image cache: a folder with `index.json` and the pictures.
#[derive(Clone, Debug)]
pub struct Images {
    pub dir: PathBuf,
}

impl Images {
    pub fn new(dir: impl Into<PathBuf>) -> Images {
        Images { dir: dir.into() }
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    pub fn load_index(&self) -> serde_json::Map<String, Value> {
        std::fs::read_to_string(self.index_path())
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| if let Value::Object(m) = v { Some(m) } else { None })
            .unwrap_or_default()
    }

    pub fn save_index(&self, idx: &serde_json::Map<String, Value>) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join("index.json.tmp");
        std::fs::write(&tmp, serde_json::to_string(idx).unwrap_or_default())?;
        std::fs::rename(tmp, self.index_path())
    }

    fn have(&self, file: &str) -> bool {
        self.dir.join(file).is_file()
    }

    /// `{card name: image files}` for the cards already on disk (no network)
    pub fn cached(&self, names: &[String]) -> BTreeMap<String, Vec<String>> {
        let idx = self.load_index();
        let mut out = BTreeMap::new();
        for n in names {
            let have: Vec<String> = files_of(idx.get(&key(n))).into_iter().filter(|f| self.have(f)).collect();
            if !have.is_empty() {
                out.insert(n.clone(), have);
            }
        }
        out
    }

    /// Make sure every card in `names` (and the tokens they make) has its images on disk. `progress(done, total)`
    /// is called as it goes. Returns `{card name: image files}` for the cards it has images for, tokens under their
    /// name. `lookup` and `download` default to Scryfall (see `scryfall_lookup`, `scryfall_download`).
    pub fn prepare(
        &self,
        names: &[String],
        progress: &mut dyn FnMut(usize, usize),
        lookup: &dyn Fn(Vec<Value>) -> Vec<Value>,
        download: &dyn Fn(&str, &Path) -> Result<(), String>,
    ) -> Result<BTreeMap<String, Vec<String>>, String> {
        let mut idx = self.load_index();
        let want: Vec<&String> = names.iter().filter(|n| !idx.contains_key(&key(n))).collect();
        for batch in want.chunks(75) {
            let ids = batch.iter().map(|n| json!({"name": n.split(" // ").next().unwrap_or("")})).collect();
            let mut found: BTreeMap<String, Value> = BTreeMap::new();
            for card in lookup(ids) {
                if let Some(n) = card["name"].as_str() {
                    found.insert(key(n), card.clone());
                }
                if let Some(n) = card["card_faces"][0]["name"].as_str() {
                    found.insert(key(n), card.clone());
                }
            }
            for n in batch {
                let card = found.get(&key(n)).or_else(|| found.get(&key(n.split(" // ").next().unwrap_or(""))));
                if let Some(card) = card {
                    idx.insert(key(n), entry(card));
                }
            }
        }
        let mut tokens: Vec<String> = vec![];
        for n in names {
            for t in strings(idx.get(&key(n)).map(|e| &e["tokens"])) {
                if !tokens.contains(&t) {
                    tokens.push(t);
                }
            }
        }
        // new, or cached before power/toughness were kept
        let missing: Vec<&String> = tokens
            .iter()
            .filter(|t| idx.get(&format!("token:{t}")).is_none_or(|e| e.get("type_line").is_none()))
            .collect();
        for batch in missing.chunks(75) {
            for card in lookup(batch.iter().map(|t| json!({"id": t})).collect()) {
                if let Some(id) = card["id"].as_str() {
                    idx.insert(format!("token:{id}"), entry(&card));
                }
            }
        }
        self.save_index(&idx).map_err(|e| e.to_string())?;
        let mut entries: Vec<(String, &Value)> =
            names.iter().filter_map(|n| idx.get(&key(n)).map(|e| (n.clone(), e))).collect();
        for t in &tokens {
            if let Some(e) = idx.get(&format!("token:{t}")) {
                entries.push((e["name"].as_str().unwrap_or("").to_string(), e));
            }
        }
        let mut jobs: Vec<(String, String)> = vec![];
        for (_, e) in &entries {
            for (u, f) in strings(Some(&e["urls"])).into_iter().zip(files_of(Some(e))) {
                if !self.have(&f) && !jobs.iter().any(|j| j.1 == f) {
                    jobs.push((u, f));
                }
            }
        }
        let total = jobs.len();
        progress(0, total);
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        for (i, (url, f)) in jobs.iter().enumerate() {
            let _ = download(url, &self.dir.join(f)); // drawn from its name in the page
            progress(i + 1, total);
        }
        let mut out = BTreeMap::new();
        for (name, e) in entries {
            let have: Vec<String> = files_of(Some(e)).into_iter().filter(|f| self.have(f)).collect();
            if !have.is_empty() {
                out.entry(name).or_insert(have);
            }
        }
        Ok(out)
    }

    /// `prepare` against Scryfall (or `PRACTICE_SCRYFALL`)
    pub fn prepare_online(
        &self,
        names: &[String],
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<BTreeMap<String, Vec<String>>, String> {
        self.prepare(names, progress, &scryfall_lookup, &scryfall_download)
    }

    /// the creature tokens the cards in `names` can make, from the image index (empty when the images haven't been
    /// fetched): `[{name, power, toughness, colors, flying}]`
    pub fn token_kinds(&self, names: &[String]) -> Vec<Value> {
        let idx = self.load_index();
        let (mut out, mut seen) = (vec![], BTreeSet::new());
        let sorted: BTreeSet<&String> = names.iter().collect();
        for n in sorted {
            for t in strings(idx.get(&key(n)).map(|e| &e["tokens"])) {
                let Some(e) = idx.get(&format!("token:{t}")) else { continue };
                if seen.contains(&t) || !e["type_line"].as_str().unwrap_or("").contains("Creature") {
                    continue;
                }
                seen.insert(t);
                let num = |k: &str| match &e[k] {
                    Value::Null => Some(0),
                    v => v.as_str().and_then(|s| s.parse::<i64>().ok()),
                };
                let (Some(pw), Some(tg)) = (num("power"), num("toughness")) else { continue }; // */* tokens
                out.push(json!({"name": e["name"], "power": pw, "toughness": tg,
                    "colors": e["colors"].as_str().unwrap_or(""),
                    "flying": strings(Some(&e["keywords"])).iter().any(|k| k == "flying")}));
            }
        }
        out
    }
}

/// every card name in the decks of a game: `(key, cards, commander)` seat specs
pub fn game_names(specs: &[(String, Vec<String>, Option<String>)]) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    for (_, cards, cmd) in specs {
        for n in cards.iter().chain(cmd.iter()) {
            if !names.contains(n) {
                names.push(n.clone());
            }
        }
    }
    names
}

fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

fn files_of(e: Option<&Value>) -> Vec<String> {
    strings(e.map(|e| &e["files"]))
}

/// the index entry for a Scryfall card object
pub fn entry(card: &Value) -> Value {
    let mut urls: Vec<String> = if card["image_uris"].is_object() {
        vec![card["image_uris"][SIZE].as_str().unwrap_or("").to_string()]
    } else {
        card["card_faces"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| f["image_uris"][SIZE].as_str().unwrap_or("").to_string())
            .collect()
    };
    urls.retain(|u| !u.is_empty());
    let id = card["id"].as_str().unwrap_or("");
    let files: Vec<String> =
        (0..urls.len()).map(|i| if i == 0 { format!("{id}.jpg") } else { format!("{id}-{i}.jpg") }).collect();
    let tokens: Vec<&str> = card["all_parts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| p["component"] == "token")
        .filter_map(|p| p["id"].as_str())
        .collect();
    let keywords: Vec<String> = strings(Some(&card["keywords"])).iter().map(|k| k.to_lowercase()).collect();
    json!({"id": id, "name": card["name"], "urls": urls, "files": files, "tokens": tokens,
        "type_line": card["type_line"].as_str().unwrap_or(""), "power": card["power"], "toughness": card["toughness"],
        "colors": strings(Some(&card["colors"])).join(""), "keywords": keywords})
}

/// Scryfall `/cards/collection` for up to 75 identifiers; empty if offline
pub fn scryfall_lookup(identifiers: Vec<Value>) -> Vec<Value> {
    wait();
    let r = agent()
        .post(&format!("{}/cards/collection", base_url()))
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .send_json(json!({"identifiers": identifiers}));
    match r.ok().and_then(|r| r.into_json::<Value>().ok()) {
        Some(v) => v["data"].as_array().cloned().unwrap_or_default(),
        None => vec![],
    }
}

pub fn scryfall_download(url: &str, path: &Path) -> Result<(), String> {
    wait();
    let r = agent().get(url).set("User-Agent", USER_AGENT).call().map_err(|e| e.to_string())?;
    let mut data = vec![];
    std::io::Read::read_to_end(&mut r.into_reader(), &mut data).map_err(|e| e.to_string())?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("practice-img-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn card(name: &str, id: &str) -> Value {
        json!({"id": id, "name": name, "image_uris": {"normal": format!("http://x/{id}.jpg")}, "type_line": "Creature",
               "all_parts": [{"component": "token", "id": "tok1"}]})
    }

    #[test]
    fn prepare_fetches_once_and_caches() {
        let im = Images::new(temp("a"));
        let names = vec!["Sol Ring".to_string(), "Nowhere".to_string()];
        let looked = Mutex::new(0);
        let lookup = |ids: Vec<Value>| -> Vec<Value> {
            *looked.lock().unwrap() += 1;
            ids.iter()
                .filter_map(|i| match (i["name"].as_str(), i["id"].as_str()) {
                    (Some("Sol Ring"), _) => Some(card("Sol Ring", "sr")),
                    (_, Some("tok1")) => Some(json!({"id": "tok1", "name": "Soldier", "type_line": "Token Creature", "power": "1",
                        "toughness": "1", "colors": ["W"], "keywords": ["Flying"], "image_uris": {"normal": "http://x/tok1.jpg"}})),
                    _ => None,
                })
                .collect()
        };
        let download = |_: &str, p: &Path| std::fs::write(p, b"jpg").map_err(|e| e.to_string());
        let mut seen = vec![];
        let got = im.prepare(&names, &mut |d, t| seen.push((d, t)), &lookup, &download).unwrap();
        assert_eq!(got["Sol Ring"], vec!["sr.jpg"]);
        assert_eq!(got["Soldier"], vec!["tok1.jpg"]);
        assert!(!got.contains_key("Nowhere"));
        assert_eq!(seen, vec![(0, 2), (1, 2), (2, 2)]);
        assert_eq!(im.cached(&names)["Sol Ring"], vec!["sr.jpg"]);
        // a second call needs no downloads, and the unknown card is looked up again
        let mut seen = vec![];
        im.prepare(&names, &mut |d, t| seen.push((d, t)), &lookup, &download).unwrap();
        assert_eq!(seen, vec![(0, 0)]);
        let kinds = im.token_kinds(&names);
        assert_eq!(kinds, vec![json!({"name": "Soldier", "power": 1, "toughness": 1, "colors": "W", "flying": true})]);
        let _ = std::fs::remove_dir_all(&im.dir);
    }

    #[test]
    fn a_failed_download_is_skipped() {
        let im = Images::new(temp("b"));
        let lookup = |_: Vec<Value>| vec![card("Sol Ring", "sr")];
        let download = |_: &str, _: &Path| Err("offline".to_string());
        let got = im.prepare(&["Sol Ring".to_string()], &mut |_, _| {}, &lookup, &download).unwrap();
        assert!(got.is_empty());
        let _ = std::fs::remove_dir_all(&im.dir);
    }

    #[test]
    fn double_faced_cards_have_a_file_per_face() {
        let c = json!({"id": "dfc", "name": "A // B", "card_faces": [
            {"name": "A", "image_uris": {"normal": "u0"}}, {"name": "B", "image_uris": {"normal": "u1"}}]});
        assert_eq!(entry(&c)["files"], json!(["dfc.jpg", "dfc-1.jpg"]));
    }
}
