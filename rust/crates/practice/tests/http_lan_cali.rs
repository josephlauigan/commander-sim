//! Two people at a two-seat format (Cali) over HTTP: the host opens the table the way the page does, a friend joins
//! from another address, each has their own seats and decisions, Undo asks the other person, a saved game has no tier
//! and waits for the friend; a deck the table's format does not admit is refused before the lobby is taken. No deck is
//! named: the Cali deck is the catalog's deck whose own format has two seats, the guest the first other deck that
//! format admits. Like http_lan.rs these need a server with --lan and a non-loopback address (the in-process one has
//! both when this computer has a LAN address), otherwise they say SKIPPED.
mod common;
use common::*;
use practice::catalog::MY_DECKS;
use practice::server::api::{Info, SessionApi, SessionFactory};
use practice::server::hub::Hub;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};

macro_rules! lan {
    () => {{
        let Some(s) = server() else { return };
        if !s.lan || s.remote_addr.is_none() {
            eprintln!("SKIPPED: needs a server started with --lan and a non-loopback address for this computer");
            return;
        }
        s
    }};
}

/// what the catalog says about the decks
struct Decks {
    /// the Cali deck (its own format has two seats), its format's key and name
    cali: String,
    format: String,
    format_name: String,
    /// the first other deck that format admits
    guest: String,
    /// a deck of the four-seat format (Commander), and another one it admits
    commander: String,
    commander2: String,
    /// a deck the Cali format refuses, if there is one
    refused_at_cali: Option<String>,
    /// a tier key (the page sends one at every format)
    tier: String,
}

fn decks(c: &mut Client) -> Decks {
    let o = c.get("/api/options").json();
    let formats = o["formats"].as_array().unwrap();
    let seats_of = |k: &str| formats.iter().find(|f| f["key"] == k).map_or(4, |f| f["seats"].as_u64().unwrap());
    let ds = o["decks"].as_array().unwrap();
    let key = |d: &Value| d["key"].as_str().unwrap().to_string();
    let cali = ds.iter().find(|d| seats_of(d["format"].as_str().unwrap_or("commander")) == 2).expect("a Cali deck");
    let format = cali["format"].as_str().unwrap().to_string();
    let format_name = formats.iter().find(|f| f["key"] == format.as_str()).unwrap()["name"].as_str().unwrap().into();
    let guest = ds
        .iter()
        .find(|d| d["key"] != cali["key"] && d["formats"].as_array().is_some_and(|a| a.iter().any(|k| *k == format)))
        .expect("a second deck the format admits");
    let admits = |d: &Value, f: &str| d["formats"].as_array().is_none_or(|a| a.iter().any(|k| k == f));
    let commander = ds
        .iter()
        .find(|d| seats_of(d["format"].as_str().unwrap_or("commander")) == 4)
        .expect("a deck of the four-seat format");
    let commander2 = ds
        .iter()
        .find(|d| d["key"] != commander["key"] && admits(d, "commander"))
        .expect("a second deck the four-seat format admits");
    let refused_at_cali = ds.iter().find(|d| !admits(d, &format)).map(key);
    Decks {
        cali: key(cali),
        format,
        format_name,
        guest: key(guest),
        commander: key(commander),
        commander2: key(commander2),
        refused_at_cali,
        tier: o["tiers"][0]["key"].as_str().unwrap().to_string(),
    }
}

/// the new-game request the page sends to open a table for two at the deck's own format
fn page_body(d: &Decks, deck: &str, format: Option<&str>, opponents: Value, seed: u64) -> Value {
    let mut b = json!({
        "deck": deck, "tier": d.tier, "ai": "adaptive", "profile": "loose", "autopass": "respond", "images": false,
        "tools": {"hint": true, "undo": true, "compare": false}, "opponents": opponents, "two": true, "seed": seed,
    });
    if let Some(f) = format {
        b["format"] = json!(f);
    }
    b
}

/// one person's browser, answering what is asked of their seat
struct Person {
    c: Client,
    played: Vec<(String, i64)>,
    answered: usize,
}

impl Person {
    fn new(c: Client) -> Person {
        Person { c, played: vec![], answered: 0 }
    }

    fn state(&mut self) -> Value {
        self.c.get("/api/state").json()
    }

    /// answer this seat's waiting decision with the bot, if there is one (false: nothing waits, or it was answered in
    /// the meantime)
    fn answer_waiting(&mut self) -> bool {
        let st = self.state();
        let p = &st["pending"];
        if !p.is_object() {
            return false;
        }
        let seat = st["game"]["you"].as_str().unwrap_or("").to_string();
        let mut ev = p.clone();
        ev["view"] = p["views"][&seat].clone();
        let a = bot(&ev, &mut self.played);
        if self.c.answer(p["id"].as_u64().unwrap(), a).status == 200 {
            self.answered += 1;
            return true;
        }
        false
    }
}

/// open the table (host on this computer, friend on the other address), join it, and return both people and the seed
fn table(s: &Server, d: &Decks, seed: u64) -> (Person, Person) {
    let mut host = Person::new(s.client());
    let mut friend = Person::new(s.remote_client().unwrap());
    host.c.get("/");
    friend.c.get("/");
    let r = host.c.post("/api/new", page_body(d, &d.cali, Some(&d.format), json!([]), seed));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let code = r.json()["code"].as_str().unwrap().to_string();
    let r = friend.c.post("/api/join", json!({"code": code, "deck": d.guest}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    (host, friend)
}

/// both people answer what is asked until each has answered `n` decisions
fn play_until(host: &mut Person, friend: &mut Person, n: usize) {
    let end = Instant::now() + Duration::from_secs(300);
    while host.answered.min(friend.answered) < n {
        assert!(Instant::now() < end, "answered {} and {} of {n}", host.answered, friend.answered);
        let moved = host.answer_waiting() | friend.answer_waiting();
        if !moved {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn wait_until(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    f()
}

#[test]
fn a_cali_table_for_two_opens_and_a_friend_joins() {
    let s = lan!();
    let d = decks(&mut s.client());
    let mut host = Person::new(s.client());
    let mut friend = Person::new(s.remote_client().unwrap());
    host.c.get("/");
    friend.c.get("/");
    // the page's own request: the deck's format, no opponents, a tier (the page keeps sending one)
    let r = host.c.post("/api/new", page_body(&d, &d.cali, Some(&d.format), json!([]), 10));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let code = r.json()["code"].as_str().unwrap().to_string();
    // the lobby: the format, and no tier; the friend does not see the code
    let lb = host.state()["lobby"].clone();
    assert_eq!(lb["format"], d.format.as_str());
    assert_eq!(lb["tier"], Value::Null);
    assert_eq!((lb["deck"].as_str(), lb["code"].as_str()), (Some(d.cali.as_str()), Some(code.as_str())));
    let fl = friend.state()["lobby"].clone();
    assert_eq!((fl["format"].as_str(), fl["deck"].as_str()), (Some(d.format.as_str()), Some(d.cali.as_str())));
    assert!(fl.get("code").is_none());
    let r = friend.c.post("/api/join", json!({"code": code, "deck": d.guest}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let (a, b) = (host.state(), friend.state());
    assert_eq!(a["game"]["you"], d.cali.as_str());
    assert_eq!(b["game"]["you"], d.guest.as_str());
    for st in [&a, &b] {
        assert_eq!(st["game"]["tier"], Value::Null, "a table of people only has no tier");
        assert_eq!(st["game"]["format"], d.format.as_str());
        assert_eq!(st["game"]["seats"].as_array().unwrap().len(), 2, "no AI seat");
        let mut humans: Vec<&str> =
            st["game"]["humans"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
        humans.sort();
        let mut want = vec![d.cali.as_str(), d.guest.as_str()];
        want.sort();
        assert_eq!(humans, want);
    }
    // each person gets their own mulligan (one decision each, theirs), and then the game goes on
    let mut seen = [false, false];
    let end = Instant::now() + Duration::from_secs(120);
    while !(seen[0] && seen[1]) {
        assert!(Instant::now() < end, "no mulligan for {seen:?}");
        for (i, p) in [&mut host, &mut friend].into_iter().enumerate() {
            let st = p.state();
            if st["pending"]["request"]["kind"] == "mulligan" {
                assert_eq!(st["pending"]["seat"], st["game"]["you"]);
                seen[i] = true;
                p.answer_waiting();
            }
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    play_until(&mut host, &mut friend, 6);
    host.c.quit();
}

#[test]
fn a_deck_the_format_refuses_is_refused_before_the_lobby_is_taken() {
    let s = lan!();
    let d = decks(&mut s.client());
    let mut host = Person::new(s.client());
    let mut friend = Person::new(s.remote_client().unwrap());
    host.c.get("/");
    friend.c.get("/");
    // a Commander table, and the Cali deck as the guest: the format refuses it
    let mut body = page_body(&d, &d.commander, None, json!([]), 10);
    body.as_object_mut().unwrap().remove("opponents"); // the table draws two at random (the page sends the ones it showed)
    let r = host.c.post("/api/new", body);
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let code = r.json()["code"].as_str().unwrap().to_string();
    assert_eq!(host.state()["lobby"]["format"], "commander");
    let r = friend.c.post("/api/join", json!({"code": code, "deck": d.cali}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("cannot be played at a Commander table"), "{}", r.error());
    // the lobby is still there with the same code, and no game started
    let st = host.state();
    assert_eq!(st["lobby"]["code"], code.as_str());
    assert!(st["game"].is_null());
    // the same guest can pick another deck with the same code
    let r = friend.c.post("/api/join", json!({"code": code, "deck": d.commander2}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(friend.state()["game"]["seats"].as_array().unwrap().len(), 4);
    host.c.quit();
}

#[test]
fn undo_at_a_cali_table_for_two_asks_the_other_person() {
    let s = lan!();
    let d = decks(&mut s.client());
    let (mut host, mut friend) = table(&s, &d, 10);
    play_until(&mut host, &mut friend, 4);
    let last = host.state()["last_event"].as_u64().unwrap();
    let mut stream = host.c.stream(last, None);
    // the friend asks; the host is asked, and says yes; both games go back
    let r = friend.c.post("/api/undo", json!({"n": 1}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert!(!r.json()["asked"].is_null(), "the other person is asked first");
    assert!(wait_until(30, || host.state()["proposal"].is_object()));
    let prop = host.state()["proposal"].clone();
    assert!(prop["text"].as_str().unwrap().contains("take back"), "{prop}");
    assert!(friend.state()["proposal"].is_null());
    // not theirs to answer
    assert_eq!(friend.c.post("/api/approve", json!({"id": prop["id"], "yes": true})).status, 409);
    assert_eq!(host.c.post("/api/approve", json!({"id": prop["id"], "yes": true})).status, 200);
    let evs = stream.until(120, |e| e["kind"] == "reset");
    let reset = evs.last().unwrap();
    assert_eq!(reset["kind"], "reset");
    assert_eq!(reset["by"], d.guest.as_str());
    // no warning about the replay: it went the way it was recorded
    assert!(!evs.iter().any(|e| e["kind"] == "warn"), "{evs:?}");
    // both people are asked again, each their own decisions, and the game goes on
    host.answered = 0;
    friend.answered = 0;
    play_until(&mut host, &mut friend, 3);
    host.c.quit();
}

#[test]
fn a_saved_cali_game_for_two_has_no_tier_in_its_name_and_waits_for_the_friend() {
    let s = lan!();
    let d = decks(&mut s.client());
    let (mut host, mut friend) = table(&s, &d, 10);
    play_until(&mut host, &mut friend, 6);
    // a decision waits: save (retry while an answer in flight lands)
    let mut r = host.c.post("/api/save", json!({}));
    assert!(wait_until(60, || {
        if r.status == 409 {
            r = host.c.post("/api/save", json!({}));
        }
        r.status != 409
    }));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let name = r.json()["name"].as_str().unwrap().to_string();
    let seed = host.state()["game"]["seed"].as_u64().unwrap();
    assert!(name.contains(&format!("{}+{}", d.cali, d.guest)), "{name}");
    assert!(
        name.ends_with(&format!("-{}-{seed}.json", d.format)),
        "no tier segment between the format and the seed: {name}"
    );
    let file = s.save_file(&name).expect("the save file");
    assert!(file.get("tier").is_none(), "a table of people only has no tier");
    assert_eq!((file["format"].as_str(), file["partner"].as_str()), (Some(d.format.as_str()), Some(d.guest.as_str())));
    assert_eq!(file["tape"], json!([]));
    // the saves list shows it with no tier
    let listed = host.c.get("/api/saves").json();
    let x = listed["saves"].as_array().unwrap().iter().find(|x| x["name"] == name.as_str()).unwrap().clone();
    assert_eq!((x["tier"].clone(), x["format"].clone()), (Value::Null, json!(d.format)));
    host.c.quit();
    // it loads (a two-person game needs --lan) and waits in a lobby for the friend, with the guest's deck
    let r = host.c.post("/api/load", json!({"name": name, "images": false}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let code = r.json()["code"].as_str().unwrap().to_string();
    let lb = friend.state()["lobby"].clone();
    assert_eq!(
        (
            lb["deck"].as_str(),
            lb["partner"].as_str(),
            lb["loaded"].as_bool(),
            lb["format"].as_str(),
            lb["tier"].clone()
        ),
        (Some(d.cali.as_str()), Some(d.guest.as_str()), Some(true), Some(d.format.as_str()), Value::Null)
    );
    let wrong = friend.c.post("/api/join", json!({"code": code, "deck": d.cali}));
    assert_eq!(wrong.status, 409);
    let ok = friend.c.post("/api/join", json!({"code": code, "deck": d.guest}));
    assert_eq!(ok.status, 200, "{}", String::from_utf8_lossy(&ok.body));
    let st = friend.state();
    assert_eq!((st["game"]["you"].as_str(), st["game"]["tier"].clone()), (Some(d.guest.as_str()), Value::Null));
    host.c.quit();
    s.remove_save(&name);
}

#[test]
fn two_at_a_two_seat_format_needs_lan() {
    // a server without --lan (the shared one has it): the same 409 as at a four-seat format
    use practice::server::{self, Config};
    let Some(shared) = server() else { return }; // holds the lock that keeps the shared server's tests apart
    let d = decks(&mut shared.client());
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let tmp = std::env::temp_dir().join(format!("practice-nolan-{}", std::process::id()));
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
    let engine = Arc::new(sim_core::run::Engine::load(&data).expect("card data"));
    let cfg = Config {
        saves: data.join("saves"),
        data: data.clone(),
        static_dir: root.join("rust/crates/practice/static"),
        lan: false,
        app: false,
        port: 0,
    };
    let h = server::start_with(cfg, Arc::new(practice::session::api_impl::RealSessions { engine })).unwrap();
    let mut c = Client::new(&format!("127.0.0.1:{}", h.port));
    let r = c.post("/api/new", page_body(&d, &d.cali, Some(&d.format), json!([]), 3));
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert!(r.error().contains("--lan"), "{}", r.error());
    let r = c.post("/api/new", page_body(&d, &d.commander, None, json!([]), 3));
    assert_eq!((r.status, r.error().contains("--lan")), (409, true));
    h.stop();
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn a_cali_table_for_two_refuses_what_it_cannot_keep() {
    let s = lan!();
    let d = decks(&mut s.client());
    let mut c = s.client();
    let post = |c: &mut Client, f: &dyn Fn(&mut Value)| {
        let mut b = page_body(&d, &d.cali, Some(&d.format), json!([]), 3);
        f(&mut b);
        let r = c.post("/api/new", b);
        c.quit();
        r
    };
    // AI opponents at a table of people only
    let r = post(&mut c, &|b| b["opponents"] = json!([d.guest]));
    assert_eq!(r.status, 400, "{}", String::from_utf8_lossy(&r.body));
    assert!(r.error().ends_with("for two people has no AI opponents"), "{}", r.error());
    // a seat or seats are dropped (the seats are drawn): the table still opens
    for (k, v) in [("seat", json!(1)), ("seats", json!([d.cali, d.guest]))] {
        let r = post(&mut c, &|b| b[k] = v.clone());
        assert_eq!(r.status, 200, "{k}: {}", String::from_utf8_lossy(&r.body));
    }
    // the tier is still checked at every format
    let r = post(&mut c, &|b| b["tier"] = json!("t9"));
    assert_eq!((r.status, r.error().starts_with("tier: one of ")), (400, true));
    let r = post(&mut c, &|b| {
        b.as_object_mut().unwrap().remove("tier");
    });
    assert_eq!((r.status, r.error().starts_with("tier: one of ")), (400, true));
    // a deck the format refuses, as the host's
    if let Some(bad) = &d.refused_at_cali {
        let r = post(&mut c, &|b| b["deck"] = json!(bad));
        assert_eq!(r.status, 400, "{}", String::from_utf8_lossy(&r.body));
        assert!(r.error().contains(&format!("cannot be played at a {} table", d.format_name)), "{}", r.error());
    }
    // a friend needs the table's code: a partner key at a one-person two-seat table is refused
    let r = post(&mut c, &|b| {
        b.as_object_mut().unwrap().remove("two");
        b["partner"] = json!(d.guest);
    });
    assert_eq!(r.status, 400, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(r.error(), "partner: a friend joins with the table's code");
}

// ------------------------------------------------------------------------------------------------ the placeholder

/// a factory whose `check_two` refuses the partners in `refuse` (and records the partners it was asked about)
struct Checks {
    refuse: Vec<String>,
    asked: std::sync::Mutex<Vec<String>>,
}

impl SessionFactory for Checks {
    fn new_session(&self, _: &Value) -> Result<Arc<dyn SessionApi>, String> {
        Err("not used".into())
    }
    fn check_two(&self, opts: &Value, partner: &str) -> Result<(), String> {
        assert!(opts.get("partner").is_none());
        self.asked.lock().unwrap().push(partner.to_string());
        if self.refuse.iter().any(|r| r == partner) { Err(format!("{partner} refused")) } else { Ok(()) }
    }
    fn load_session(&self, _: &Value, _: bool) -> Result<Arc<dyn SessionApi>, String> {
        Err("not used".into())
    }
    fn card_names(&self, _: &Info) -> Vec<String> {
        vec![]
    }
}

fn hub_with(f: Arc<Checks>) -> (Arc<Hub>, std::path::PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("practice-ph-{}-{:?}", std::process::id(), std::thread::current().id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    (Hub::new(true, false, dir.clone(), practice::images::Images::new(dir.clone()), f), dir)
}

#[test]
fn the_placeholder_skips_a_deck_the_check_refuses() {
    let host = MY_DECKS[0];
    let others: Vec<String> = MY_DECKS.iter().filter(|k| **k != host).map(|k| k.to_string()).collect();
    assert!(others.len() >= 3);
    // the first two others refuse: the lobby opens with the third checked and accepted
    let f = Arc::new(Checks { refuse: others[..2].to_vec(), asked: Default::default() });
    let (h, dir) = hub_with(f.clone());
    let opts = json!({"deck": host, "tier": "t1", "images": false});
    assert!(h.open_lobby(&opts, "host", None).is_ok());
    assert_eq!(*f.asked.lock().unwrap(), others[..3].to_vec(), "each deck but the host's, in order, until one passes");
    h.shutdown();
    // the first one passes: only it is checked (the placeholder is what it always was)
    let f = Arc::new(Checks { refuse: vec![], asked: Default::default() });
    let (h2, _) = hub_with(f.clone());
    assert!(h2.open_lobby(&opts, "host", None).is_ok());
    assert_eq!(*f.asked.lock().unwrap(), others[..1].to_vec());
    h2.shutdown();
    // none passes: the first refusal is the error, and no lobby opens
    let f = Arc::new(Checks { refuse: others.clone(), asked: Default::default() });
    let (h3, _) = hub_with(f.clone());
    assert_eq!(h3.open_lobby(&opts, "host", None).unwrap_err(), format!("{} refused", others[0]));
    assert_eq!(*f.asked.lock().unwrap(), others);
    assert!(h3.state(Some("host"), true)["lobby"].is_null());
    h3.shutdown();
    let _ = std::fs::remove_dir_all(dir);
}
