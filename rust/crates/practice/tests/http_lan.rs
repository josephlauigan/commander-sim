//! Port of the server half of tests/test_play_lan.py (TwoPlayerServer), plus the HTTP-reachable parts of its
//! TwoPlayerSession tests (the hint is for the seat deciding): the lobby and join code, each browser's seat (a
//! cookie), each seat's own stream with the other's hand left out, an Undo the other person must agree to, a saved
//! two-player game. The second person is a client that reaches the server from this computer's LAN address (a
//! non-loopback source), so the server sees "the other computer"; set PRACTICE_URL_HOST2 to use another address.
//! These need a server started with --lan and an address other than loopback; otherwise they say SKIPPED.
mod common;
use common::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// one person's browser: its own cookie, and (when listening) its own event stream answered by the bot
struct Browser {
    c: Client,
    events: Arc<Mutex<Vec<Value>>>,
    requests: Option<Receiver<Value>>,
    auto: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

impl Browser {
    fn new(c: Client) -> Browser {
        Browser {
            c,
            events: Arc::default(),
            requests: None,
            auto: Arc::new(AtomicBool::new(true)),
            stop: Arc::default(),
        }
    }
    fn call(&mut self, method: &str, path: &str, body: Option<Value>) -> Resp {
        self.c.call(method, path, body)
    }
    fn post(&mut self, path: &str, body: Value) -> Resp {
        self.c.post(path, body)
    }
    fn get(&mut self, path: &str) -> Value {
        self.c.get(path).json()
    }
    fn count(&self, kind: &str) -> usize {
        self.events.lock().unwrap().iter().filter(|e| e["kind"] == kind).count()
    }
    fn has(&self, kind: &str) -> bool {
        self.count(kind) > 0
    }
    /// read this seat's stream in the background; answer its requests with the bot while `auto` is on, else queue them
    fn listen(&mut self, played: Arc<Mutex<Vec<(String, i64)>>>) {
        let (tx, rx) = channel();
        self.requests = Some(rx);
        let mut c = Client { addr: self.c.addr.clone(), cookie: self.c.cookie.clone() };
        let (events, auto, stop) = (self.events.clone(), self.auto.clone(), self.stop.clone());
        thread::spawn(move || {
            let mut st = c.stream(0, None);
            while !stop.load(Ordering::Relaxed) {
                let Some(ev) = st.next(Duration::from_millis(500)) else { continue };
                events.lock().unwrap().push(ev.clone());
                if ev["kind"] == "request" {
                    if auto.load(Ordering::Relaxed) {
                        if ev["view"].is_null() && ev["request"]["kind"] == "priority" {
                            continue;
                        }
                        let a = bot(&ev, &mut played.lock().unwrap());
                        c.answer(ev["id"].as_u64().unwrap(), a);
                    } else {
                        let _ = tx.send(ev);
                    }
                }
            }
        });
    }
    fn next_request(&self, secs: u64) -> Option<Value> {
        self.requests.as_ref().unwrap().recv_timeout(Duration::from_secs(secs)).ok()
    }
}

fn wait_until(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    f()
}

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

/// the host opens a two-player table; returns (host, friend, join code)
fn open_table(s: &Server) -> (Browser, Browser, String) {
    let mut host = Browser::new(s.client());
    let mut friend = Browser::new(s.remote_client().unwrap());
    host.call("GET", "/", None);
    friend.call("GET", "/", None);
    let r = host.post(
        "/api/new",
        // seed 8: with the 10-13 Sauron and Seph lists, seed 7 kills both human seats early and the AIs finish the game
        // before the test can pause the bots (the save test then waits for a decision that never comes)
        json!({"deck": "sauron", "tier": "t2", "seed": 8, "ai": "adaptive", "images": false, "two": true}),
    );
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let j = r.json();
    assert!(j["addresses"].is_array());
    (host, friend, j["code"].as_str().unwrap().to_string())
}

fn join(friend: &mut Browser, code: &str, deck: &str) -> u16 {
    friend.post("/api/join", json!({"code": code, "deck": deck})).status
}

#[test]
fn test_two_needs_a_server_started_with_lan() {
    let Some(s) = server() else { return };
    if s.lan {
        eprintln!("SKIPPED: the server was started with --lan");
        return;
    }
    let mut c = s.client();
    let r = c.post("/api/new", json!({"deck": "sauron", "tier": "t2", "two": true}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("--lan"), "{}", r.error());
}

#[test]
fn test_the_lobby_and_joining() {
    let s = lan!();
    let (mut host, mut friend, code) = open_table(&s);
    let st = host.get("/api/state");
    assert_eq!(st["lobby"]["code"], code.as_str());
    assert!(st["lobby"]["addresses"].is_array());
    let st = friend.get("/api/state");
    assert!(st["lobby"].get("code").is_none()); // the friend needs it from the host
    assert_eq!(st["lobby"]["deck"], "sauron");
    let wrong = if code != "000000" { "000000" } else { "111111" };
    assert_eq!(join(&mut friend, wrong, "seph"), 409);
    assert_eq!(join(&mut friend, &code, "sauron"), 409);
    assert_eq!(join(&mut host, &code, "seph"), 409);
    assert_eq!(join(&mut friend, &code, "seph"), 200);
    let a = host.get("/api/state");
    let b = friend.get("/api/state");
    assert_eq!((a["game"]["you"].as_str(), b["game"]["you"].as_str()), (Some("sauron"), Some("seph")));
    let mut humans: Vec<&str> = a["game"]["humans"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    humans.sort();
    assert_eq!(humans, ["sauron", "seph"]);
    assert_eq!(friend.post("/api/new", json!({"deck": "veyran", "tier": "t2"})).status, 403);
    assert_eq!(friend.post("/api/quit", json!({})).status, 403);
    assert_eq!(friend.post("/api/save", json!({})).status, 403);
    assert_eq!(friend.c.get("/api/saves").status, 403);
    let mut stranger = Browser::new(s.remote_client().unwrap());
    stranger.call("GET", "/", None);
    let st = stranger.get("/api/state");
    assert!(st["game"].is_null());
    assert_eq!(st["busy"], true);
    host.post("/api/quit", json!({}));
}

#[test]
fn test_joining_when_nothing_waits() {
    let s = lan!();
    let mut host = Browser::new(s.client());
    let mut friend = Browser::new(s.remote_client().unwrap());
    host.post("/api/quit", json!({}));
    let r = friend.post("/api/join", json!({"code": "123456", "deck": "seph"}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "There is no game waiting for a second player.");
}

#[test]
fn test_each_seat_sees_its_own_game() {
    let s = lan!();
    let (mut host, mut friend, code) = open_table(&s);
    join(&mut friend, &code, "seph");
    let played = Arc::new(Mutex::new(Vec::new()));
    host.listen(played.clone());
    friend.listen(played);
    let round3 = |b: &Browser| {
        b.events.lock().unwrap().iter().any(|e| e["kind"] == "turn" && e["view"]["round"].as_i64().unwrap_or(0) >= 3)
    };
    assert!(wait_until(300, || round3(&host)), "no turn 3");
    host.stop.store(true, Ordering::Relaxed);
    friend.stop.store(true, Ordering::Relaxed);
    host.post("/api/quit", json!({}));
    for (b, me) in [(&host, "sauron"), (&friend, "seph")] {
        let evs = b.events.lock().unwrap();
        let reqs: Vec<&Value> = evs.iter().filter(|e| e["kind"] == "request").collect();
        assert!(!reqs.is_empty());
        assert!(evs.iter().any(|e| e["kind"] == "waiting")); // the other person's decisions
        for e in evs.iter() {
            let v = &e["view"];
            if v.is_null() {
                continue;
            }
            let players = v["players"].as_array().unwrap();
            let you: Vec<&Value> = players.iter().filter(|p| p["you"] == true).collect();
            assert_eq!(you.len(), 1);
            assert_eq!(you[0]["key"], me);
            assert!(players.iter().filter(|p| p["you"] != true).all(|p| p.get("hand").is_none()));
        }
        assert!(!evs.iter().any(|e| e.get("seat").is_some() || e.get("views").is_some() || e.get("hide").is_some()));
        assert_eq!(reqs.iter().filter(|e| e["request"]["kind"] == "mulligan").count(), 1);
        assert!(evs.windows(2).all(|w| w[0]["id"].as_u64() < w[1]["id"].as_u64()));
    }
}

#[test]
fn test_hint_is_for_the_seat_deciding() {
    let s = lan!();
    let (mut host, mut friend, code) = open_table(&s);
    join(&mut friend, &code, "seph");
    host.auto.store(false, Ordering::Relaxed);
    friend.auto.store(false, Ordering::Relaxed);
    let played = Arc::new(Mutex::new(Vec::new()));
    host.listen(played.clone());
    friend.listen(played);
    // the mulligans come one after the other, in seating order (random with two players): while one person's waits,
    // the other has no decision
    let end = Instant::now() + Duration::from_secs(120);
    let (mut first, mut second, req) = loop {
        assert!(Instant::now() < end, "no mulligan came");
        if let Some(r) = host.requests.as_ref().unwrap().recv_timeout(Duration::from_millis(300)).ok() {
            break (host, friend, r);
        }
        if let Some(r) = friend.requests.as_ref().unwrap().recv_timeout(Duration::from_millis(300)).ok() {
            break (friend, host, r);
        }
    };
    let r = second.post("/api/hint", json!({}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("no decision"), "{}", r.error());
    assert_eq!(first.post("/api/hint", json!({})).status, 200);
    assert_eq!(first.post("/api/answer", json!({"id": req["id"], "answer": "keep"})).status, 200);
    let f = second.next_request(120).expect("the second person's mulligan");
    let r = first.post("/api/hint", json!({}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("no decision"), "{}", r.error());
    assert_eq!(second.post("/api/hint", json!({})).status, 200);
    second.post("/api/answer", json!({"id": f["id"], "answer": "keep"}));
    first.stop.store(true, Ordering::Relaxed);
    second.stop.store(true, Ordering::Relaxed);
    first.post("/api/quit", json!({}));
}

#[test]
fn test_undo_needs_the_other_persons_ok() {
    let s = lan!();
    let (mut host, mut friend, code) = open_table(&s);
    join(&mut friend, &code, "seph");
    let played = Arc::new(Mutex::new(Vec::new()));
    host.auto.store(false, Ordering::Relaxed);
    friend.auto.store(false, Ordering::Relaxed);
    host.listen(played.clone());
    friend.listen(played.clone());
    let mut answered = [0usize; 2];
    let end = Instant::now() + Duration::from_secs(300);
    while answered.iter().min().unwrap() < &3 && Instant::now() < end {
        // both mulligans and a few decisions each
        for (i, b) in [&mut host, &mut friend].into_iter().enumerate() {
            if let Some(ev) = b.requests.as_ref().unwrap().recv_timeout(Duration::from_millis(500)).ok() {
                let a = bot(&ev, &mut played.lock().unwrap());
                let r = b.post("/api/answer", json!({"id": ev["id"], "answer": a}));
                assert_eq!(r.status, 200, "browser {i} answering {ev}: {}", String::from_utf8_lossy(&r.body));
                answered[i] += 1;
            }
        }
    }
    assert!(answered.iter().min().unwrap() >= &3, "{answered:?}");
    let r = friend.post("/api/undo", json!({"n": 1}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert!(!r.json()["asked"].is_null());
    assert_eq!(friend.post("/api/undo", json!({"n": 1})).status, 409); // one request at a time
    assert!(wait_until(30, || host.has("proposal")));
    let prop = host.events.lock().unwrap().iter().find(|e| e["kind"] == "proposal").unwrap()["proposal"].clone();
    assert!(prop["text"].as_str().unwrap().contains("take back"));
    assert!(!friend.has("proposal"));
    assert_eq!(host.get("/api/state")["proposal"]["id"], prop["id"]);
    assert_eq!(friend.post("/api/approve", json!({"id": prop["id"], "yes": true})).status, 409); // not theirs
    assert_eq!(host.post("/api/approve", json!({"id": prop["id"], "yes": false})).status, 200);
    assert!(wait_until(30, || friend.has("declined")));
    // asking again is allowed once it was answered
    let r = friend.post("/api/undo", json!({"n": 1}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert!(wait_until(30, || host.count("proposal") >= 2));
    let prop =
        host.events.lock().unwrap().iter().filter(|e| e["kind"] == "proposal").last().unwrap()["proposal"].clone();
    assert_eq!(host.post("/api/approve", json!({"id": prop["id"], "yes": true})).status, 200);
    assert!(wait_until(120, || host.has("reset") && friend.has("reset")));
    host.stop.store(true, Ordering::Relaxed);
    friend.stop.store(true, Ordering::Relaxed);
    host.post("/api/quit", json!({}));
}

#[test]
fn test_a_saved_two_player_game_waits_for_the_friend() {
    let s = lan!();
    let (mut host, mut friend, code) = open_table(&s);
    join(&mut friend, &code, "seph");
    let played = Arc::new(Mutex::new(Vec::new()));
    host.listen(played.clone());
    friend.listen(played);
    assert!(wait_until(120, || host.count("decided") >= 12));
    host.auto.store(false, Ordering::Relaxed);
    friend.auto.store(false, Ordering::Relaxed);
    let (hq, fq) = (host.requests.take().unwrap(), friend.requests.take().unwrap());
    // a decision is waiting, unanswered (the bot may have answered one in flight, so wait for a queued request)
    assert!(wait_until(120, || host.get("/api/state")["pending"].is_object()
        || friend.get("/api/state")["pending"].is_object()));
    let r = host.post("/api/save", json!({}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let name = r.json()["name"].as_str().unwrap().to_string();
    assert!(name.contains("+seph"), "{name}");
    host.stop.store(true, Ordering::Relaxed);
    friend.stop.store(true, Ordering::Relaxed);
    drop((hq, fq));
    let r2 = host.post("/api/load", json!({"name": name, "images": false}));
    assert_eq!(r2.status, 200, "{}", String::from_utf8_lossy(&r2.body));
    let code2 = r2.json()["code"].as_str().unwrap().to_string();
    assert!(!code2.is_empty());
    let lb = friend.get("/api/state")["lobby"].clone();
    assert_eq!(
        (lb["deck"].as_str(), lb["partner"].as_str(), lb["loaded"].as_bool()),
        (Some("sauron"), Some("seph"), Some(true))
    );
    assert_eq!(join(&mut friend, &code2, "veyran"), 409);
    assert_eq!(join(&mut friend, &code2, "seph"), 200);
    assert_eq!(friend.get("/api/state")["game"]["you"], "seph");
    host.post("/api/quit", json!({}));
    s.remove_save(&name);
}

#[test]
fn test_a_saved_two_player_game_needs_lan() {
    // only meaningful on a server without --lan; with it, the load above is the test
    let Some(s) = server() else { return };
    if s.lan {
        eprintln!("SKIPPED: the server was started with --lan");
    }
}

#[test]
fn test_one_player_undo_is_at_once() {
    let Some(s) = server() else { return };
    let mut host = Browser::new(s.client());
    host.call("GET", "/", None);
    host.post("/api/new", json!({"deck": "sauron", "tier": "t2", "seed": 4, "ai": "adaptive", "images": false}));
    host.auto.store(false, Ordering::Relaxed);
    host.listen(Arc::default());
    let ev = host.next_request(120).unwrap();
    host.post("/api/answer", json!({"id": ev["id"], "answer": "keep"}));
    host.next_request(120).unwrap();
    let r = host.post("/api/undo", json!({"n": 1}));
    assert_eq!(r.status, 200);
    assert!(r.json().get("asked").is_none());
    host.stop.store(true, Ordering::Relaxed);
    host.post("/api/quit", json!({}));
}
