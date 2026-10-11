//! The Scryfall card cache (`data/scryfall_cache.json`, Python's format: lower-case card name -> slim record) and the
//! client that fills it. Follows Scryfall's API guidelines: a descriptive User-Agent and Accept header, at most ~10
//! requests per second, batch lookups through /cards/collection (75 names per request), and cache everything.
//! Tests and `cardgen export --offline` never touch the network.

use crate::json::{self, J, Obj};
use crate::{jo, obj};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const API: &str = "https://api.scryfall.com";

pub fn key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[derive(Debug, Default)]
pub struct Cache {
    pub path: PathBuf,
    order: Vec<String>,
    map: HashMap<String, J>,
    dirty: bool,
}

impl Cache {
    pub fn load(path: &Path) -> Result<Cache, String> {
        let mut c = Cache { path: path.to_path_buf(), ..Default::default() };
        if path.exists() {
            let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            match json::parse(&text)? {
                J::Obj(o) => {
                    for (k, v) in o.0 {
                        c.order.push(k.clone());
                        c.map.insert(k, v);
                    }
                }
                _ => return Err(format!("{}: not a JSON object", path.display())),
            }
        }
        Ok(c)
    }

    /// the record for a card name (`cache.get(name.lower())`; Python's callers differ on stripping, names have none)
    pub fn get(&self, name: &str) -> Option<&J> {
        self.map.get(&key(name))
    }

    pub fn get_raw(&self, lower: &str) -> Option<&J> {
        self.map.get(lower)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(&key(name))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// every cached card's name (the record's own `name`)
    pub fn names(&self) -> Vec<String> {
        self.order.iter().filter_map(|k| self.map.get(k).and_then(|v| v.s("name")).map(|s| s.to_string())).collect()
    }

    pub fn insert(&mut self, k: String, rec: J) {
        if !self.map.contains_key(&k) {
            self.order.push(k.clone());
        }
        self.map.insert(k, rec);
        self.dirty = true;
    }

    /// write the cache back (only if something was added), in Python's `json.dump` form
    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let o = Obj(self.order.iter().map(|k| (k.clone(), self.map[k].clone())).collect());
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, json::dumps(&J::Obj(o), None, true)).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &self.path).map_err(|e| e.to_string())?;
        self.dirty = false;
        Ok(())
    }
}

/// keep only what the simulator uses
pub fn slim(card: &J) -> J {
    let mut out = Obj::new();
    for k in [
        "name",
        "mana_cost",
        "cmc",
        "type_line",
        "oracle_text",
        "power",
        "toughness",
        "loyalty",
        "colors",
        "color_identity",
        "keywords",
        "produced_mana",
        "layout",
        "game_changer",
        "legalities",
    ] {
        if let Some(v) = card.get(k) {
            out.set(k, v.clone());
        }
    }
    if out.contains("legalities") {
        let c = card.get("legalities").and_then(|l| l.get("commander")).cloned().unwrap_or(J::Null);
        out.set("legalities", obj! {"commander" => c});
    }
    if let Some(faces) = card.get("card_faces").and_then(|f| f.as_arr()).filter(|f| !f.is_empty()) {
        let fs: Vec<J> = faces
            .iter()
            .map(|f| {
                let mut o = Obj::new();
                for k in ["name", "mana_cost", "type_line", "oracle_text", "power", "toughness", "loyalty"] {
                    if let Some(v) = f.get(k) {
                        o.set(k, v.clone());
                    }
                }
                J::Obj(o)
            })
            .collect();
        out.set("card_faces", fs);
    }
    J::Obj(out)
}

struct Client {
    last: Instant,
}

impl Client {
    fn wait(&mut self) {
        let dt = self.last.elapsed();
        if dt < Duration::from_millis(100) {
            std::thread::sleep(Duration::from_millis(100) - dt);
        }
        self.last = Instant::now();
    }

    fn request(&mut self, url: &str, body: Option<&J>) -> Result<J, String> {
        self.wait();
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build();
        let req = match body {
            Some(_) => agent.post(url),
            None => agent.get(url),
        }
        .set("User-Agent", "commander-pod-sim/1.0 (personal deck testing)")
        .set("Accept", "application/json");
        let resp = match body {
            Some(b) => req.set("Content-Type", "application/json").send_string(&json::dumps(b, None, true)),
            None => req.call(),
        }
        .map_err(|e| e.to_string())?;
        json::parse(&resp.into_string().map_err(|e| e.to_string())?)
    }
}

/// Look up every name not yet cached (batches of 75) and add it to the cache; returns the names Scryfall couldn't
/// find. A failed request (offline) stops the lookups and is reported on stderr; what was fetched is kept.
pub fn fetch(cache: &mut Cache, names: &[String], verbose: bool) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let want: Vec<&String> = names.iter().filter(|n| seen.insert(key(n)) && !cache.contains(n)).collect();
    let mut missing = vec![];
    if want.is_empty() {
        return missing;
    }
    if verbose {
        eprintln!("Scryfall: fetching {} card(s)...", want.len());
    }
    let mut client = Client { last: Instant::now() - Duration::from_secs(1) };
    for batch in want.chunks(75) {
        let ids: Vec<J> = batch.iter().map(|n| jo!("name" => py_first(n, " // "))).collect();
        let res = match client.request(&format!("{API}/cards/collection"), Some(&jo!("identifiers" => ids))) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Scryfall request failed ({e}). Check your internet connection; using cached cards only.");
                missing.extend(batch.iter().map(|n| n.to_string()));
                break;
            }
        };
        let mut found: HashMap<String, J> = HashMap::new();
        for card in res.get("data").and_then(|d| d.as_arr()).cloned().unwrap_or_default() {
            if let Some(n) = card.s("name") {
                found.insert(key(n), card.clone());
            }
            if let Some(f0) = card.get("card_faces").and_then(|f| f.as_arr()).and_then(|f| f.first()) {
                if let Some(n) = f0.s("name") {
                    found.insert(key(n), card.clone());
                }
            }
        }
        for n in batch {
            let mut card = found.get(&key(n)).or_else(|| found.get(&key(py_first(n, " // ")))).cloned();
            if card.is_none() {
                // fall back to fuzzy lookup for odd spellings
                let url = format!("{API}/cards/named?fuzzy={}", urlencode(n));
                card = client.request(&url, None).ok();
            }
            match card {
                Some(c) if c.s("name").is_some() => cache.insert(key(n), slim(&c)),
                _ => {
                    if verbose {
                        eprintln!("  not found on Scryfall: {n}");
                    }
                    missing.push(n.to_string());
                }
            }
        }
        if let Err(e) = cache.save() {
            eprintln!("could not save the cache: {e}");
        }
    }
    missing
}

fn py_first<'a>(s: &'a str, sep: &str) -> &'a str {
    s.split(sep).next().unwrap_or("")
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
