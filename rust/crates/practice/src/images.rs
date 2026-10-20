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
    /// set while a background download runs (shared by clones), so a second one never starts
    fetching: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Images {
    pub fn new(dir: impl Into<PathBuf>) -> Images {
        Images { dir: dir.into(), fetching: Default::default() }
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
        // a temp file of its own per save: the background commander fetch and a game's image download can save at once
        static SAVES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = SAVES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = self.dir.join(format!("index.json.{}.{n}.tmp", std::process::id()));
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
        self.prepare_printed(names, &Printings::new(), progress, lookup, download).map(|r| r.0)
    }

    /// `prepare`, plus the exact printings of the decks that list them: those are looked up by set and collector
    /// number (index key `print:<set>/<number>`), and a card with a printing needs no lookup by name. Also returns
    /// `{deck key: {card name: [image files of each copy]}}`, for the copies whose images are on disk.
    pub fn prepare_printed(
        &self,
        names: &[String],
        printed: &Printings,
        progress: &mut dyn FnMut(usize, usize),
        lookup: &dyn Fn(Vec<Value>) -> Vec<Value>,
        download: &dyn Fn(&str, &Path) -> Result<(), String>,
    ) -> Result<(BTreeMap<String, Vec<String>>, SeatImages), String> {
        let mut idx = self.load_index();
        let prints: BTreeSet<&String> = printed.values().flat_map(|m| m.values().flatten()).collect();
        let missing: Vec<&&String> = prints.iter().filter(|p| !idx.contains_key(&print_key(p))).collect();
        for batch in missing.chunks(75) {
            let ids = batch
                .iter()
                .filter_map(|p| p.split_once('/'))
                .map(|(set, num)| json!({"set": set, "collector_number": num}))
                .collect();
            for card in lookup(ids) {
                if let (Some(set), Some(num)) = (card["set"].as_str(), card["collector_number"].as_str()) {
                    idx.insert(print_key(&format!("{set}/{num}")), entry(&card));
                }
            }
        }
        // a card with a printing needs no lookup by name: its first printing stands in for it
        for m in printed.values() {
            for (n, ps) in m {
                if !idx.contains_key(&key(n))
                    && let Some(e) = ps.iter().find_map(|p| idx.get(&print_key(p))).cloned()
                {
                    idx.insert(key(n), e);
                }
            }
        }
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
        let printed_entries: Vec<&Value> = prints.iter().filter_map(|p| idx.get(&print_key(p))).collect();
        let mut jobs: Vec<(String, String)> = vec![];
        for e in entries.iter().map(|(_, e)| *e).chain(printed_entries) {
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
        Ok((out, self.cached_printed_in(&idx, printed)))
    }

    /// `prepare_printed` against Scryfall (or `PRACTICE_SCRYFALL`)
    pub fn prepare_online(
        &self,
        names: &[String],
        printed: &Printings,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(BTreeMap<String, Vec<String>>, SeatImages), String> {
        self.prepare_printed(names, printed, progress, &scryfall_lookup, &scryfall_download)
    }

    /// Download `names` and the `printed` pictures in a background thread, without blocking the caller; fails
    /// quietly (offline: nothing is cached and the page draws the cards from their names). Returns None, doing
    /// nothing, when a background download already runs.
    pub fn fetch_in_background(
        &self,
        names: Vec<String>,
        printed: Printings,
        lookup: std::sync::Arc<dyn Fn(Vec<Value>) -> Vec<Value> + Send + Sync>,
        download: std::sync::Arc<dyn Fn(&str, &Path) -> Result<(), String> + Send + Sync>,
    ) -> Option<std::thread::JoinHandle<()>> {
        use std::sync::atomic::Ordering;
        if self.fetching.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return None;
        }
        struct Done(std::sync::Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Done {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let (me, done) = (self.clone(), Done(self.fetching.clone()));
        let spawned = std::thread::Builder::new().name("practice-images".into()).spawn(move || {
            let _done = done;
            let _ = me.prepare_printed(&names, &printed, &mut |_, _| {}, &*lookup, &*download);
        });
        match spawned {
            Ok(h) => Some(h),
            Err(_) => {
                self.fetching.store(false, Ordering::SeqCst);
                None
            }
        }
    }

    /// `fetch_in_background` against Scryfall (or `PRACTICE_SCRYFALL`)
    pub fn fetch_online_in_background(
        &self,
        names: Vec<String>,
        printed: Printings,
    ) -> Option<std::thread::JoinHandle<()>> {
        self.fetch_in_background(
            names,
            printed,
            std::sync::Arc::new(scryfall_lookup),
            std::sync::Arc::new(scryfall_download),
        )
    }

    /// `{deck key: {card name: [image files of each copy]}}` for the printings already on disk (no network)
    pub fn cached_printed(&self, printed: &Printings) -> SeatImages {
        self.cached_printed_in(&self.load_index(), printed)
    }

    fn cached_printed_in(&self, idx: &serde_json::Map<String, Value>, printed: &Printings) -> SeatImages {
        let mut out = SeatImages::new();
        for (deck, m) in printed {
            for (n, ps) in m {
                let copies: Vec<Vec<String>> = ps
                    .iter()
                    .filter_map(|p| idx.get(&print_key(p)))
                    .map(|e| files_of(Some(e)).into_iter().filter(|f| self.have(f)).collect::<Vec<_>>())
                    .filter(|f| !f.is_empty())
                    .collect();
                if !copies.is_empty() {
                    out.entry(deck.clone()).or_default().insert(n.clone(), copies);
                }
            }
        }
        out
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

/// deck key -> card name -> the printing of each copy ("set/number"), from `data/printings/<key>.txt`
pub type Printings = BTreeMap<String, BTreeMap<String, Vec<String>>>;
/// deck key -> card name -> the image files of each copy
pub type SeatImages = BTreeMap<String, BTreeMap<String, Vec<Vec<String>>>>;

fn print_key(p: &str) -> String {
    format!("print:{}", p.to_lowercase())
}

/// the printings of these decks: `data/printings/<key>.txt`, lines like "6 Island (mdd) 294" (written by
/// `cardgen deck` from a list with set codes); a deck without the file shows Scryfall's usual printing
pub fn load_printings(data: &Path, keys: &[String]) -> Printings {
    let mut out = Printings::new();
    for k in keys {
        let Ok(text) = std::fs::read_to_string(data.join("printings").join(format!("{k}.txt"))) else { continue };
        let deck = out.entry(k.clone()).or_default();
        for line in text.lines() {
            let Some((count, rest)) = line.trim().split_once(' ') else { continue };
            let Some(at) = rest.rfind(" (") else { continue };
            let Some((set, num)) = rest[at + 2..].split_once(") ") else { continue };
            let copies = deck.entry(rest[..at].to_string()).or_default();
            for _ in 0..count.parse::<usize>().unwrap_or(1) {
                copies.push(format!("{set}/{}", num.trim()));
            }
        }
    }
    out
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
    fn background_fetch_runs_once_at_a_time_and_fails_quietly_offline() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let im = Images::new(temp("bg"));
        let names = vec!["Sol Ring".to_string()];
        // offline: nothing found, nothing cached, no panic
        let h = im
            .fetch_in_background(
                names.clone(),
                Printings::new(),
                Arc::new(|_| vec![]),
                Arc::new(|_, _| Err("offline".into())),
            )
            .unwrap();
        h.join().unwrap();
        assert!(im.cached(&names).is_empty());
        // a download that is still running blocks a second one
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let rx = std::sync::Mutex::new(rx);
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = n.clone();
        let slow = Arc::new(move |ids: Vec<Value>| -> Vec<Value> {
            if n2.fetch_add(1, Ordering::SeqCst) > 0 {
                return vec![]; // the token lookup, after the first
            }
            let _ = rx.lock().unwrap().recv();
            ids.iter().filter_map(|i| i["name"].as_str()).map(|nm| card(nm, "id9")).collect()
        });
        let h = im
            .fetch_in_background(
                names.clone(),
                Printings::new(),
                slow.clone(),
                Arc::new(|_, p| std::fs::write(p, b"x").map_err(|e| e.to_string())),
            )
            .unwrap();
        assert!(im.fetch_in_background(names.clone(), Printings::new(), slow, Arc::new(|_, _| Ok(()))).is_none());
        tx.send(()).unwrap();
        h.join().unwrap();
        assert!(n.load(Ordering::SeqCst) >= 1);
        assert_eq!(im.cached(&names).get("Sol Ring").map(Vec::len), Some(1));
        // and once it finished another may start
        assert!(
            im.fetch_in_background(vec![], Printings::new(), Arc::new(|_| vec![]), Arc::new(|_, _| Ok(()))).is_some()
        );
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
    fn printings_are_fetched_by_set_and_number_per_deck() {
        let im = Images::new(temp("p").join("images"));
        let data = im.dir.parent().unwrap().to_path_buf();
        std::fs::create_dir_all(data.join("printings")).unwrap();
        std::fs::write(data.join("printings/a.txt"), "1 Sol Ring (sld) 2093\n2 Plains (bro) 268\n1 Plains (dom) 252\n")
            .unwrap();
        std::fs::write(data.join("printings/b.txt"), "1 Sol Ring (fic) 356\n").unwrap();
        let printed = load_printings(&data, &["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(printed["a"]["Plains"], ["bro/268", "bro/268", "dom/252"]);
        assert!(!printed.contains_key("c"));
        let by_name = Mutex::new(vec![]);
        let lookup = |ids: Vec<Value>| -> Vec<Value> {
            ids.iter()
                .filter_map(|i| {
                    if let Some(n) = i["name"].as_str() {
                        by_name.lock().unwrap().push(n.to_string());
                        return (n == "Island").then(|| card("Island", "isl"));
                    }
                    let (set, num) = (i["set"].as_str()?, i["collector_number"].as_str()?);
                    let id = format!("{set}{num}");
                    let mut c = card(if set == "sld" || set == "fic" { "Sol Ring" } else { "Plains" }, &id);
                    c["set"] = set.into();
                    c["collector_number"] = num.into();
                    c["all_parts"] = json!([]);
                    Some(c)
                })
                .collect()
        };
        let download = |_: &str, p: &Path| std::fs::write(p, b"jpg").map_err(|e| e.to_string());
        let names: Vec<String> = ["Sol Ring", "Plains", "Island"].map(String::from).to_vec();
        let (got, seats) = im.prepare_printed(&names, &printed, &mut |_, _| {}, &lookup, &download).unwrap();
        // only the card without a printing is looked up by name
        assert_eq!(*by_name.lock().unwrap(), ["Island"]);
        assert_eq!(seats["a"]["Sol Ring"], [["sld2093.jpg"]]);
        assert_eq!(seats["b"]["Sol Ring"], [["fic356.jpg"]]);
        assert_eq!(seats["a"]["Plains"], [["bro268.jpg"], ["bro268.jpg"], ["dom252.jpg"]]);
        assert_eq!(got["Island"], ["isl.jpg"]);
        assert!(got.contains_key("Sol Ring")); // a printing stands in for the name
        assert_eq!(im.cached_printed(&printed), seats);
        let _ = std::fs::remove_dir_all(&data);
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

// ------------------------------------------------------------------ naming the tokens the engine doesn't name
static GUESS_IMAGES: Mutex<Option<Images>> = Mutex::new(None);

/// Register `view.guess_token` with the engine (`legal::token_name`): the image index of `images` names the tokens.
pub fn register_token_guesser(images: &Images) {
    *GUESS_IMAGES.lock().unwrap_or_else(|e| e.into_inner()) = Some(images.clone());
    sim_core::human::legal::set_token_guesser(guess_token);
}

/// Python's `view.guess_token`: many cards make tokens the engine doesn't name (Krenko's Goblins): the token its
/// owner's deck makes with the same power, toughness, colour and flying (then any deck at the table's, for tokens
/// given to you: Beast Within)
fn guess_token(g: &sim_core::state::Game, m: &sim_core::state::Perm) -> Option<String> {
    let images = GUESS_IMAGES.lock().unwrap_or_else(|e| e.into_inner()).clone()?;
    let names_of = |ps: &[sim_core::ids::PlayerId]| -> Vec<String> {
        ps.iter().flat_map(|&q| g.player(q).deck_names.iter().map(|&c| g.db.get(c).name.to_string())).collect()
    };
    let all: Vec<_> = g.players.iter().map(|q| q.id).collect();
    for names in [names_of(&[m.orig]), names_of(&all)] {
        if let Some(n) = pick_token(&images.token_kinds(&names), m.pow, m.tgh, m.colors, m.fly) {
            return Some(n);
        }
    }
    None
}

/// the best token among `kinds` with this size: colour counts only when the permanent has one (the engine leaves
/// many tokens colourless), then flying, then how many of the deck's cards make it
fn pick_token(kinds: &[Value], pw: i32, tg: i32, colors: sim_core::cards::Colors, fly: bool) -> Option<String> {
    let kinds: Vec<&Value> = kinds.iter().filter(|t| t["power"] == pw && t["toughness"] == tg).collect();
    let count = |n: &Value| kinds.iter().filter(|t| &t["name"] == n).count() as f64;
    let score = |t: &Value| {
        let col = sim_core::cards::Colors::from_letters(t["colors"].as_str().unwrap_or(""));
        (if colors.is_empty() { 0.0 } else { f64::from(u8::from(col == colors) * 2) })
            + f64::from(u8::from(t["flying"].as_bool().unwrap_or(false) == fly))
            + count(&t["name"]) / 100.0
    };
    let mut best: Option<(&Value, f64)> = None;
    for t in &kinds {
        let s = score(t);
        if best.is_none_or(|b| s > b.1) {
            best = Some((t, s)); // the first of equals, as Python's max
        }
    }
    best.and_then(|b| b.0["name"].as_str().map(str::to_string))
}

#[cfg(test)]
mod guess_tests {
    use super::*;
    use sim_core::cards::Colors;

    #[test]
    fn the_token_with_the_same_size_colour_and_flying() {
        let k = vec![
            json!({"name": "Goblin", "power": 1, "toughness": 1, "colors": "R", "flying": false}),
            json!({"name": "Soldier", "power": 1, "toughness": 1, "colors": "W", "flying": false}),
            json!({"name": "Spirit", "power": 1, "toughness": 1, "colors": "W", "flying": true}),
            json!({"name": "Beast", "power": 3, "toughness": 3, "colors": "G", "flying": false}),
        ];
        assert_eq!(pick_token(&k, 1, 1, Colors::from_letters("R"), false).as_deref(), Some("Goblin"));
        assert_eq!(pick_token(&k, 1, 1, Colors::from_letters("W"), true).as_deref(), Some("Spirit"));
        // colourless: flying decides, then the first
        assert_eq!(pick_token(&k, 1, 1, Colors::NONE, true).as_deref(), Some("Spirit"));
        assert_eq!(pick_token(&k, 1, 1, Colors::NONE, false).as_deref(), Some("Goblin"));
        assert_eq!(pick_token(&k, 9, 9, Colors::NONE, false), None);
    }
}
