// written by an independent reviewer
//! C5c adversarial tests that need servers of their own: a server without --lan (`two` at either format, joining,
//! loading a two-person save), shutdown with a table of two waiting (no game, pump or comparison thread left), and the
//! page files (the ids the script uses exist in the page; nothing writes HTML from event text). No deck is named.
mod common;
use common::Client;
use practice::catalog::MY_DECKS;
use practice::server::{self, Config, ServerHandle};
use practice::session::{Options, Session};
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn engine() -> Arc<Engine> {
    static E: OnceLock<Arc<Engine>> = OnceLock::new();
    E.get_or_init(|| Arc::new(Engine::load(&root().join("data")).expect("card data"))).clone()
}

struct Srv {
    h: ServerHandle,
    saves: PathBuf,
    tmp: PathBuf,
}

impl Srv {
    fn client(&self) -> Client {
        let mut c = Client::new(&format!("127.0.0.1:{}", self.h.port));
        c.get("/");
        c
    }
}

impl Drop for Srv {
    fn drop(&mut self) {
        self.h.stop();
        let _ = std::fs::remove_dir_all(&self.tmp);
    }
}

fn start(lan: bool, tag: &str) -> Srv {
    // SAFETY: set before any server of this test binary reads it
    unsafe { std::env::set_var("PRACTICE_NO_FETCH", "1") };
    let root = root();
    let tmp = std::env::temp_dir().join(format!("practice-adv-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let data = tmp.join("data");
    std::fs::create_dir_all(data.join("images")).unwrap();
    std::fs::create_dir_all(data.join("saves")).unwrap();
    std::os::unix::fs::symlink(root.join("decklists"), tmp.join("decklists")).unwrap();
    for e in std::fs::read_dir(root.join("data")).unwrap().flatten() {
        let n = e.file_name();
        if n != "images" && n != "saves" {
            std::os::unix::fs::symlink(e.path(), data.join(&n)).unwrap();
        }
    }
    let cfg = Config {
        saves: data.join("saves"),
        data: data.clone(),
        static_dir: root.join("rust/crates/practice/static"),
        lan,
        app: false,
        port: 0,
    };
    let h = server::start_with(cfg, Arc::new(practice::session::api_impl::RealSessions { engine: engine() })).unwrap();
    Srv { h, saves: data.join("saves"), tmp }
}

fn practice_threads() -> usize {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir("/proc/self/task") {
        for t in rd.flatten() {
            if let Ok(c) = std::fs::read_to_string(t.path().join("comm")) {
                let c = c.trim();
                if c.starts_with("practice-") && c != "practice-server" && c != "practice-conn" {
                    n += 1;
                }
            }
        }
    }
    n
}

fn wait_until(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

struct Decks {
    cali: String,
    format: String,
    guest: String,
    cmdr: String,
    cmdr2: String,
    tier: String,
}

fn decks() -> Decks {
    let e = engine();
    let cali = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).unwrap().to_string();
    let format = e.format_of(&cali).key.clone();
    let guest = MY_DECKS.iter().find(|k| **k != cali && e.admits(k, &format).is_ok()).unwrap().to_string();
    let cmdr = MY_DECKS.iter().find(|k| e.format_of(k).seats == 4).unwrap().to_string();
    let cmdr2 = MY_DECKS.iter().find(|k| **k != cmdr && e.admits(k, "commander").is_ok()).unwrap().to_string();
    let tier = e.decks[&cmdr].tiers[0].clone();
    Decks { cali, format, guest, cmdr, cmdr2, tier }
}

fn body(d: &Decks, deck: &str, format: Option<&str>, two: bool) -> Value {
    let mut b = json!({"deck": deck, "tier": d.tier, "ai": "adaptive", "images": false, "seed": 5,
        "tools": {"compare": false}});
    if let Some(f) = format {
        b["format"] = json!(f);
        b["opponents"] = json!([]);
    }
    if two {
        b["two"] = json!(true);
    }
    b
}

#[test]
fn a_server_without_lan_refuses_two_at_either_format_and_every_join() {
    let _g = serial();
    let d = decks();
    let s = start(false, "nolan");
    let mut c = s.client();
    let r = c.post("/api/new", body(&d, &d.cali, Some(&d.format), true));
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert!(r.error().contains("--lan"), "{}", r.error());
    let mut b = body(&d, &d.cmdr, None, true);
    b["opponents"] = json!([]);
    let r = c.post("/api/new", b);
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert!(r.error().contains("--lan"));
    // nothing opened
    let st = c.get("/api/state").json();
    assert!(st["lobby"].is_null() && st["game"].is_null(), "{st}");
    // a join has nothing to join; with every kind of body
    for b in [json!({}), json!({"code": "123456", "deck": d.guest}), json!({"code": null, "deck": null})] {
        let mut g = s.client();
        let r = g.post("/api/join", b);
        assert_eq!(r.status, 409);
    }
    // a partner key at Cali is still the route's 400, and the lan check does not come first for it
    let mut b = body(&d, &d.cali, Some(&d.format), false);
    b["partner"] = json!(d.guest);
    let r = c.post("/api/new", b);
    assert_eq!((r.status, r.error().as_str()), (400, "partner: a friend joins with the table's code"));
}

/// a two-person Cali save as a file (from a session that was never started: no answers, empty answer_seats)
fn write_cali_pair_save(s: &Srv, d: &Decks) -> String {
    let e = engine();
    let mut o = Options::new(e, &d.cali, "");
    o.partner = Some(d.guest.clone());
    o.seed = Some(9);
    o.ai = "adaptive".into();
    o.views = true;
    o.max_rounds = 3;
    let v = Session::new(o).unwrap().saved();
    assert!(v.get("tier").is_none());
    let name = "29990101-000000-adv-pair.json".to_string();
    std::fs::write(s.saves.join(&name), v.to_string()).unwrap();
    name
}

#[test]
fn loading_a_two_person_cali_save_without_lan_is_refused_and_keeps_the_running_game() {
    let _g = serial();
    let d = decks();
    let s = start(false, "nolan2");
    let name = write_cali_pair_save(&s, &d);
    let mut c = s.client();
    // a running one-person game at the Cali deck (an opponent of the tier that the format admits)
    let o = c.get("/api/options").json();
    let opp = o["tiers"].as_array().unwrap().iter().find(|t| t["key"] == d.tier.as_str()).unwrap()["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["formats"].as_array().is_none_or(|a| a.iter().any(|k| *k == d.format)))
        .map(|x| x["key"].as_str().unwrap().to_string())
        .unwrap();
    let mut b = body(&d, &d.cali, Some(&d.format), false);
    b["opponents"] = json!([opp]);
    let r = c.post("/api/new", b);
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let before = c.get("/api/state").json();
    assert!(before["game"].is_object());
    let r = c.post("/api/load", json!({"name": name, "images": false}));
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert!(r.error().contains("start the server with --lan"), "{}", r.error());
    let after = c.get("/api/state").json();
    assert_eq!(after["game"]["seed"], before["game"]["seed"], "a refused load must not end the running game: {after}");
    assert!(after["lobby"].is_null());
    // the saves list shows the pair save without a tier
    let list = c.get("/api/saves").json();
    let row = list["saves"].as_array().unwrap().iter().find(|x| x["name"] == name.as_str()).unwrap().clone();
    assert_eq!(row["tier"], Value::Null);
    assert_eq!(row["format"], d.format.as_str());
    assert_eq!(row["partner"], d.guest.as_str());
    c.quit();
}

#[test]
fn shutdown_with_a_table_of_two_waiting_leaves_no_thread() {
    let _g = serial();
    let d = decks();
    wait_until(30, || practice_threads() == 0);
    let base = practice_threads();
    assert_eq!(base, 0, "threads of an earlier test");
    for round in 0..3 {
        let s = start(true, &format!("down{round}"));
        let mut host = s.client();
        let mut friend = Client::new(&format!("127.0.0.1:{}", s.h.port));
        friend.get("/");
        let mut b = body(&d, &d.cali, Some(&d.format), true);
        b["tools"] = json!({"compare": true});
        let r = host.post("/api/new", b);
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let code = r.json()["code"].as_str().unwrap().to_string();
        let r = friend.post("/api/join", json!({"code": code, "deck": d.guest}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        // both streams open; answer a handful of decisions with the common bot
        let mut hs = host.stream(0, None);
        let mut fs = friend.stream(0, None);
        let mut played = vec![];
        let mut answered = 0;
        let end = Instant::now() + Duration::from_secs(120);
        while answered < 8 + 3 * round && Instant::now() < end {
            for (st, cl) in [(&mut hs, &mut host), (&mut fs, &mut friend)] {
                if let Some(ev) = st.next(Duration::from_millis(100))
                    && ev["kind"] == "request"
                {
                    let a = common::bot(&ev, &mut played);
                    if cl.answer(ev["id"].as_u64().unwrap(), a).status == 200 {
                        answered += 1;
                    }
                }
            }
        }
        assert!(answered >= 8, "answered {answered}");
        assert!(practice_threads() >= 2, "a game is running: {}", practice_threads());
        // the server stops while both people wait on a decision
        s.h.stop();
        let ok = wait_until(40, || practice_threads() == 0);
        assert!(ok, "round {round}: {} practice thread(s) left 40 s after shutdown", practice_threads());
        // the streams end
        let mut ended = false;
        let end = Instant::now() + Duration::from_secs(40);
        while Instant::now() < end {
            if hs.next(Duration::from_millis(500)).is_none() && fs.next(Duration::from_millis(500)).is_none() {
                ended = true;
                break;
            }
        }
        let _ = ended; // an idle stream also returns None; the thread count above is the check
        drop(s);
    }
}

// ------------------------------------------------------------------------------------------------ the page

#[test]
fn the_pages_ids_and_texts_agree_and_nothing_writes_html_from_events() {
    let dir = root().join("rust/crates/practice/static");
    let js = std::fs::read_to_string(dir.join("app.js")).unwrap();
    let html = std::fs::read_to_string(dir.join("index.html")).unwrap();
    let table = std::fs::read_to_string(dir.join("table.js")).unwrap();
    // every id the script looks up with $('#x') is in the page, or made by the script itself
    let mut missing = vec![];
    let mut rest = js.as_str();
    while let Some(i) = rest.find("$('#") {
        rest = &rest[i + 4..];
        let id: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
        let in_html = html.contains(&format!("id=\"{id}\""));
        let made = js.contains(&format!("id: '{id}'")) || js.contains(&format!("id=\"{id}\""));
        if !in_html && !made && !missing.contains(&id) {
            missing.push(id);
        }
    }
    assert!(missing.is_empty(), "ids used by app.js that index.html does not have: {missing:?}");
    for id in ["two-opp", "ai-table", "lan-note", "opponents", "tiers", "solo-n", "solo-s"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "index.html lacks #{id}");
    }
    assert!(
        !html.contains("format-note") && !js.contains("format-note"),
        "the removed #format-note is still referenced"
    );
    // no HTML from event or deck text: innerHTML only on the table icons (fixed strings), no insertAdjacentHTML/outerHTML/document.write
    for f in [&js, &table, &html] {
        for bad in ["insertAdjacentHTML", "outerHTML", "document.write(", "dangerouslySetInnerHTML"] {
            assert!(!f.contains(bad), "{bad} found");
        }
    }
    assert!(!js.contains("innerHTML"), "app.js writes innerHTML");
    assert_eq!(table.matches("innerHTML").count(), 1, "table.js: only the icon's fixed string");
    // the warn branch goes through the same text-only helpers as the invalid branch
    let warn = js.lines().find(|l| l.contains("ev.kind === 'warn'")).expect("a warn branch");
    assert!(warn.contains("logLine(ev.text") && warn.contains("toast(ev.text"), "{warn}");
    // a pair's page texts for a table with no AI seat: no tier or AI wording on the join and lobby texts of a 2-seat format
    let join = js
        .lines()
        .find(|l| l.contains("just the two of you, no AI players"))
        .expect("the join text for a two-seat table");
    assert!(!join.to_lowercase().contains("tier"), "{join}");
    // the friend option reads without AI wording until the script adds it for a table that has AI seats
    let opt = html.lines().find(|l| l.contains("value=\"two\"")).unwrap();
    assert!(opt.contains("Me and a friend on another computer"), "{opt}");
}
