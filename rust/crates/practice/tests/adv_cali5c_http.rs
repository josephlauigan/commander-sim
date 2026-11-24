// written by an independent reviewer
//! C5c adversarial tests over HTTP: the lobby, join, `two` and `partner` at a two-seat format, the Undo proposal flow with
//! two browsers, tampered saves loaded through /api/load, hint and review per seat, the Commander two-person JSON, and a
//! lock-order stress. No deck is named: the Cali deck is the catalog's deck whose own format has two seats, the guest the
//! first other deck that format admits. Needs a --lan server at a non-loopback address (tests/common/mod.rs), else SKIPPED.
mod common;
use common::*;
use practice::catalog::MY_DECKS;
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, OnceLock};
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

// ------------------------------------------------------------------------------------------------ what the catalog says

struct Cat {
    /// the Cali deck, its format's key and name, the first other deck that format admits
    cali: String,
    format: String,
    format_name: String,
    guest: String,
    /// a four-seat deck and a second one it admits
    cmdr: String,
    cmdr2: String,
    /// every deck of mine the Cali format refuses
    refused_at_cali: Vec<String>,
    tier: String,
    /// (key, own format, formats it is admitted at)
    all: Vec<(String, String, Option<Vec<String>>)>,
    seats: Vec<(String, u64)>,
}

impl Cat {
    fn admits(&self, deck: &str, format: &str) -> bool {
        self.all
            .iter()
            .find(|d| d.0 == deck)
            .is_some_and(|d| d.2.as_ref().is_none_or(|f| f.iter().any(|k| k == format)))
    }
}

fn cat(c: &mut Client) -> Cat {
    let o = c.get("/api/options").json();
    let seats: Vec<(String, u64)> = o["formats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["key"].as_str().unwrap().to_string(), f["seats"].as_u64().unwrap_or(4)))
        .collect();
    let seats_of = |k: &str| seats.iter().find(|f| f.0 == k).map_or(4, |f| f.1);
    let all: Vec<(String, String, Option<Vec<String>>)> = o["decks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["key"].as_str().unwrap().to_string(),
                d["format"].as_str().unwrap_or("commander").to_string(),
                d["formats"].as_array().map(|a| a.iter().map(|k| k.as_str().unwrap().to_string()).collect()),
            )
        })
        .collect();
    let cali = all.iter().find(|d| seats_of(&d.1) == 2).expect("a Cali deck").clone();
    let format = cali.1.clone();
    let format_name = o["formats"].as_array().unwrap().iter().find(|f| f["key"] == format.as_str()).unwrap()["name"]
        .as_str()
        .unwrap()
        .into();
    let adm =
        |d: &(String, String, Option<Vec<String>>), f: &str| d.2.as_ref().is_none_or(|a| a.iter().any(|k| k == f));
    let guest = all.iter().find(|d| d.0 != cali.0 && adm(d, &format)).expect("a guest").0.clone();
    let cmdr = all.iter().find(|d| seats_of(&d.1) == 4).expect("a four-seat deck").clone();
    let cmdr2 = all.iter().find(|d| d.0 != cmdr.0 && adm(d, &cmdr.1)).expect("a second four-seat deck").0.clone();
    let refused_at_cali = all.iter().filter(|d| !adm(d, &format)).map(|d| d.0.clone()).collect();
    Cat {
        cali: cali.0,
        format,
        format_name,
        guest,
        cmdr: cmdr.0,
        cmdr2,
        refused_at_cali,
        tier: o["tiers"][0]["key"].as_str().unwrap().to_string(),
        all,
        seats,
    }
}

/// the new-game body the page sends for a table of two (a Cali one has no opponents; a Commander one two)
fn two_body(c: &Cat, deck: &str, format: Option<&str>, opponents: Value, seed: u64) -> Value {
    let mut b = json!({
        "deck": deck, "tier": c.tier, "ai": "adaptive", "profile": "loose", "autopass": "respond", "images": false,
        "tools": {"hint": true, "undo": true, "compare": false}, "opponents": opponents, "two": true, "seed": seed,
    });
    if let Some(f) = format {
        b["format"] = json!(f);
    }
    b
}

fn solo_body(c: &Cat, deck: &str, format: Option<&str>, opponents: Value, seed: u64) -> Value {
    let mut b = two_body(c, deck, format, opponents, seed);
    b.as_object_mut().unwrap().remove("two");
    b
}

fn post(host: &mut Client, rep: &mut Vec<String>, name: &str, body: Value, want: u16, text: &str) -> Resp {
    let r = host.post("/api/new", body);
    rep.push(format!("{name}: {} {}", r.status, r.error()));
    assert_eq!(r.status, want, "{name}: {}", String::from_utf8_lossy(&r.body));
    if !text.is_empty() {
        assert!(r.error().contains(text), "{name}: wanted {text:?} in {:?}", r.error());
    }
    r
}

fn lobby_of(c: &mut Client) -> Value {
    c.get("/api/state").json()["lobby"].clone()
}

fn code_of(r: &Resp) -> String {
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    r.json()["code"].as_str().unwrap().to_string()
}

fn open_cali(s: &Server, c: &Cat, host: &mut Client, seed: u64) -> String {
    let _ = s;
    let r = host.post("/api/new", two_body(c, &c.cali, Some(&c.format), json!([]), seed));
    code_of(&r)
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

/// how many threads of the practice crate (game, pump, images, shadow workers) are alive
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

fn engine() -> Arc<Engine> {
    static E: OnceLock<Arc<Engine>> = OnceLock::new();
    E.get_or_init(|| {
        let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
        Arc::new(Engine::load(&data).expect("load the card data"))
    })
    .clone()
}

/// one person: a browser that answers what its seat is asked
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
    fn pending(&mut self) -> Option<Value> {
        let st = self.state();
        st["pending"].is_object().then(|| st["pending"].clone())
    }
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

/// the table at the Cali format: host here, friend at the other address, joined
fn table(s: &Server, c: &Cat, seed: u64) -> (Person, Person) {
    let mut host = Person::new(s.client());
    let mut friend = Person::new(s.remote_client().unwrap());
    host.c.get("/");
    friend.c.get("/");
    let code = open_cali(s, c, &mut host.c, seed);
    let r = friend.c.post("/api/join", json!({"code": code, "deck": c.guest}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    (host, friend)
}

fn play_until(host: &mut Person, friend: &mut Person, n: usize) {
    let end = Instant::now() + Duration::from_secs(300);
    while host.answered.min(friend.answered) < n {
        assert!(Instant::now() < end, "answered {} and {} of {n}", host.answered, friend.answered);
        let moved = host.answer_waiting() | friend.answer_waiting();
        if !moved {
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

/// is it `a` who is asked now (else `b`)
fn a_decides(a: &mut Person, b: &mut Person) -> bool {
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        if a.pending().is_some() {
            return true;
        }
        if b.pending().is_some() {
            return false;
        }
        assert!(Instant::now() < end, "nobody is asked anything");
        std::thread::sleep(Duration::from_millis(30));
    }
}

/// (the deciding person, the other) given who `a_decides`
fn split<'a>(a: &'a mut Person, b: &'a mut Person, a_decides: bool) -> (&'a mut Person, &'a mut Person) {
    if a_decides { (a, b) } else { (b, a) }
}

// ------------------------------------------------------------------------------------------------ join

#[test]
fn every_join_refusal_at_a_cali_lobby_keeps_the_lobby() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    let mut friend = s.remote_client().unwrap();
    host.get("/");
    friend.get("/");
    let code = open_cali(&s, &c, &mut host, 21);
    let wrong = if code != "000000" { "000000" } else { "111111" };
    let kept = |host: &mut Client| {
        let lb = lobby_of(host);
        assert_eq!(lb["code"], code.as_str(), "the lobby (and its code) must survive: {lb}");
        assert_eq!(lb["format"], c.format.as_str());
        assert_eq!(lb["tier"], Value::Null);
    };
    let mut rep = vec![];
    let mut try_join = |friend: &mut Client, host: &mut Client, name: &str, body: Value, want: &str| {
        let r = friend.post("/api/join", body);
        rep.push(format!("{name}: {} {}", r.status, r.error()));
        assert_eq!(r.status, 409, "{name}: {}", String::from_utf8_lossy(&r.body));
        assert!(r.error().contains(want), "{name}: wanted {want:?}, got {:?}", r.error());
        kept(host);
    };
    try_join(&mut friend, &mut host, "wrong code", json!({"code": wrong, "deck": c.guest}), "That code is not right");
    try_join(&mut friend, &mut host, "no code", json!({"deck": c.guest}), "That code is not right");
    try_join(&mut friend, &mut host, "null code", json!({"code": null, "deck": c.guest}), "That code is not right");
    try_join(&mut friend, &mut host, "list code", json!({"code": [code], "deck": c.guest}), "That code is not right");
    try_join(&mut friend, &mut host, "empty body", json!({}), "That code is not right");
    try_join(&mut friend, &mut host, "no deck", json!({"code": code}), "deck: one of");
    for (name, v) in [
        ("null", json!(null)),
        ("a number", json!(5)),
        ("a list", json!([c.guest])),
        ("an object", json!({})),
        ("empty", json!("")),
        ("unknown", json!("zzz")),
    ] {
        try_join(&mut friend, &mut host, &format!("deck {name}"), json!({"code": code, "deck": v}), "deck: one of");
    }
    try_join(&mut friend, &mut host, "a mirror", json!({"code": code, "deck": c.cali}), "The host plays that deck");
    // the host's own browser (it has the code), with a good deck
    try_join(
        &mut host.clone_jar(),
        &mut host,
        "the host's browser",
        json!({"code": code, "deck": c.guest}),
        "This browser is the host's",
    );
    // a deck the Cali format refuses (every one of mine that it refuses)
    for d in &c.refused_at_cali {
        try_join(
            &mut friend,
            &mut host,
            &format!("refused deck {d}"),
            json!({"code": code, "deck": d}),
            "cannot be played at a",
        );
    }
    // the code with blanks around it works, once
    let r = friend.post("/api/join", json!({"code": format!("  {code}\n"), "deck": c.guest}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    // and a second one is too late
    let mut late = s.remote_client().unwrap();
    late.get("/");
    let r = late.post("/api/join", json!({"code": code, "deck": c.cmdr}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("There is no game waiting"), "{}", r.error());
    eprintln!(
        "RECORD join refusals at a Cali lobby ({} deck(s) of mine refused at Cali): {rep:#?}",
        c.refused_at_cali.len()
    );
    host.quit();
}

trait JarCopy {
    fn clone_jar(&self) -> Client;
}
impl JarCopy for Client {
    fn clone_jar(&self) -> Client {
        Client { addr: self.addr.clone(), cookie: self.cookie.clone() }
    }
}

#[test]
fn a_commander_lobby_refuses_the_cali_deck_and_keeps_its_code() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    let mut friend = s.remote_client().unwrap();
    host.get("/");
    friend.get("/");
    // the page's Commander request: no format key, two opponents drawn from the tier
    let r = host.post("/api/new", two_body(&c, &c.cmdr, None, json!([]), 22));
    // (the page sends two opponents at a Commander table; none is refused: record what the server says)
    eprintln!("RECORD Commander pair with opponents [] : {} {}", r.status, r.error());
    host.quit();
    let o = host.get("/api/options").json();
    let tier = o["tiers"][0]["decks"].as_array().unwrap();
    let opp: Vec<String> = tier
        .iter()
        .filter(|d| d["formats"].as_array().is_none_or(|a| a.iter().any(|k| k == "commander")))
        .map(|d| d["key"].as_str().unwrap().to_string())
        .filter(|k| *k != c.cmdr && *k != c.cmdr2)
        .take(2)
        .collect();
    let r = host.post("/api/new", two_body(&c, &c.cmdr, None, json!(opp), 22));
    let code = code_of(&r);
    let lb = lobby_of(&mut host);
    assert_eq!(lb["format"], "commander");
    assert_eq!(lb["tier"], c.tier.as_str());
    // the Cali deck as the guest of a Commander lobby, in the page's own words
    let r = friend.post("/api/join", json!({"code": code, "deck": c.cali}));
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    eprintln!("RECORD Cali deck into a Commander lobby: {}", r.error());
    assert!(r.error().contains("cannot be played at a Commander table"), "{}", r.error());
    assert_eq!(lobby_of(&mut host)["code"], code.as_str(), "the lobby must survive the refusal");
    // a legal guest still gets in with the same code
    let r = friend.post("/api/join", json!({"code": code, "deck": c.cmdr2}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let st = host.get("/api/state").json();
    assert_eq!(st["game"]["seats"].as_array().unwrap().len(), 4);
    assert_eq!(st["game"]["tier"], c.tier.as_str());
    host.quit();
}

/// every host deck x every guest deck at the host's own format and at the other formats: refused exactly when the
/// catalog says the format does not admit the guest (or it is the host's own deck), and a refused guest leaves the lobby
/// as it was
#[test]
fn host_guest_matrix_follows_the_catalog_and_never_loses_the_lobby() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    let mut friend = s.remote_client().unwrap();
    host.get("/");
    friend.get("/");
    let mut lines = vec![];
    let mut cells = 0;
    let formats: Vec<String> = c.seats.iter().map(|f| f.0.clone()).collect();
    for h in MY_DECKS.iter() {
        for format in &formats {
            let two = c.seats.iter().find(|f| &f.0 == format).unwrap().1 == 2;
            let mut body = two_body(&c, h, Some(format), json!([]), 23);
            if !two {
                body.as_object_mut().unwrap().remove("opponents");
            }
            for g in MY_DECKS.iter() {
                let r = host.post("/api/new", body.clone());
                if !c.admits(h, format) {
                    assert_eq!(r.status, 400, "{h} at {format}: {}", String::from_utf8_lossy(&r.body));
                    assert!(r.error().contains("cannot be played at a"), "{}", r.error());
                    lines.push(format!("{h} @ {format}: host refused: {}", r.error()));
                    break;
                }
                let code = code_of(&r);
                assert_eq!(lobby_of(&mut host)["format"], format.as_str());
                let r = friend.post("/api/join", json!({"code": code, "deck": g}));
                cells += 1;
                if g != h && c.admits(g, format) {
                    assert_eq!(
                        r.status,
                        200,
                        "{h} hosting @ {format}, {g} joins: {}",
                        String::from_utf8_lossy(&r.body)
                    );
                    assert!(wait_until(30, || friend.get("/api/state").json()["game"].is_object()));
                } else {
                    assert_eq!(
                        r.status,
                        409,
                        "{h} hosting @ {format}, {g} joins: {}",
                        String::from_utf8_lossy(&r.body)
                    );
                    let lb = lobby_of(&mut host);
                    assert_eq!(
                        lb["code"],
                        code.as_str(),
                        "{h} @ {format}, {g} refused ({}): the lobby is gone",
                        r.error()
                    );
                    lines.push(format!("{h} @ {format}, {g}: {}", r.error()));
                }
                host.quit();
            }
        }
    }
    eprintln!("RECORD matrix: {cells} join attempts; refusals: {lines:#?}");
}

#[test]
fn two_guests_racing_for_a_cali_lobby_one_wins() {
    let s = lan!();
    let c = Arc::new(cat(&mut s.client()));
    let mut host = s.client();
    host.get("/");
    let mut wins_total = 0;
    for round in 0..6 {
        let code = open_cali(&s, &c, &mut host, 30 + round);
        let n = 6;
        let bar = Arc::new(std::sync::Barrier::new(n));
        let (tx, rx) = channel();
        let mut browsers = vec![];
        for i in 0..n {
            let mut b = s.remote_client().unwrap();
            b.get("/");
            let (bar, tx, code, c2) = (bar.clone(), tx.clone(), code.clone(), c.clone());
            // half of the guests bring a deck the format refuses (when there is one), or the host's own deck
            let deck = if i % 3 == 2 {
                c2.refused_at_cali.first().cloned().unwrap_or_else(|| c2.cali.clone())
            } else {
                c2.guest.clone()
            };
            browsers.push(std::thread::spawn(move || {
                bar.wait();
                let r = b.post("/api/join", json!({"code": code, "deck": deck}));
                let _ = tx.send((i, r.status, r.error()));
                b
            }));
        }
        drop(tx);
        let mut got = vec![];
        let end = Instant::now() + Duration::from_secs(120);
        while got.len() < n {
            let left = end.saturating_duration_since(Instant::now());
            match rx.recv_timeout(left) {
                Ok(x) => got.push(x),
                Err(_) => panic!("HANG: guests racing for the lobby: {} of {n} answered", got.len()),
            }
        }
        let wins: Vec<_> = got.iter().filter(|g| g.1 == 200).collect();
        assert_eq!(wins.len(), 1, "round {round}: exactly one guest gets the seat: {got:?}");
        wins_total += 1;
        for g in got.iter().filter(|g| g.1 != 200) {
            assert_eq!(g.1, 409, "{g:?}");
        }
        let browsers: Vec<Client> = browsers.into_iter().map(|h| h.join().unwrap()).collect();
        // the game has the host and the winner, nobody else; the losers have no seat
        let st = host.get("/api/state").json();
        assert!(st["game"].is_object(), "{st}");
        let mut with_seat = 0;
        for mut b in browsers {
            let st = b.get("/api/state").json();
            if st["game"].is_object() {
                with_seat += 1;
            }
        }
        assert_eq!(with_seat, 1, "round {round}: only the winner has a seat");
        host.quit();
    }
    eprintln!("RECORD guests racing: {wins_total} rounds with exactly one winner");
}

#[test]
fn join_after_quit_after_another_lobby_and_after_a_solo_game() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    let mut friend = s.remote_client().unwrap();
    host.get("/");
    friend.get("/");
    let none = "There is no game waiting for a second player.";
    // quit
    let code1 = open_cali(&s, &c, &mut host, 40);
    host.quit();
    let r = friend.post("/api/join", json!({"code": code1, "deck": c.guest}));
    assert_eq!((r.status, r.error().as_str()), (409, none));
    // a second lobby replaces the first: the old code is not right any more
    let code1 = open_cali(&s, &c, &mut host, 41);
    let mut code2 = open_cali(&s, &c, &mut host, 42);
    while code2 == code1 {
        code2 = open_cali(&s, &c, &mut host, 42);
    }
    let r = friend.post("/api/join", json!({"code": code1, "deck": c.guest}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("That code is not right"), "{}", r.error());
    // a solo game replaces the lobby
    let r = host.post("/api/new", solo_body(&c, &c.cali, Some(&c.format), json!([c.guest.clone()]), 43));
    // (the solo opponent must be a tier deck: whatever the server says, the lobby must be gone)
    eprintln!("RECORD solo Cali game while a lobby waits: {} {}", r.status, r.error());
    let r = friend.post("/api/join", json!({"code": code2, "deck": c.guest}));
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert!(
        r.error().contains(none) || r.error().contains("That code is not right"),
        "a lobby replaced by a game cannot be joined: {}",
        r.error()
    );
    host.quit();
    // a lobby then Cancel (quit) then the guest's page: no game and no lobby, in the guest's state
    let code = open_cali(&s, &c, &mut host, 44);
    host.quit();
    let st = friend.get("/api/state").json();
    assert!(st["lobby"].is_null() && st["game"].is_null(), "{st}");
    let r = friend.post("/api/join", json!({"code": code, "deck": c.guest}));
    assert_eq!(r.status, 409);
}

// ------------------------------------------------------------------------------------------------ `two` and `partner`

#[test]
fn two_at_cali_with_every_odd_body() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    host.get("/");
    let mut rep: Vec<String> = vec![];
    let base = || two_body(&c, &c.cali, Some(&c.format), json!([]), 50);
    // a valid lobby first: refused requests must not destroy it
    let code = code_of(&host.post("/api/new", base()));
    let lobby_kept = |host: &mut Client, name: &str| {
        let lb = lobby_of(host);
        assert_eq!(lb["code"], code.as_str(), "{name}: a refused request must keep the waiting lobby: {lb}");
    };
    // opponents
    let mut b = base();
    b["opponents"] = json!([c.guest]);
    post(&mut host, &mut rep, "opponents [x]", b, 400, "no AI opponents");
    lobby_kept(&mut host, "opponents [x]");
    let mut b = base();
    b["opponents"] = json!([c.guest, c.cali]);
    post(&mut host, &mut rep, "opponents [x, y]", b, 400, "no AI opponents");
    lobby_kept(&mut host, "opponents [x,y]");
    let mut b = base();
    b["opponents"] = json!("x");
    post(&mut host, &mut rep, "opponents a string", b, 400, "opponents: a list of deck keys");
    lobby_kept(&mut host, "opponents string");
    let mut b = base();
    b["opponents"] = json!([5]);
    post(&mut host, &mut rep, "opponents [5]", b, 400, "opponents: a list");
    // tier
    for (name, t) in [
        ("missing", None),
        ("null", Some(json!(null))),
        ("t9", Some(json!("t9"))),
        ("5", Some(json!(5))),
        ("empty", Some(json!(""))),
    ] {
        let mut b = base();
        match t {
            Some(t) => b["tier"] = t,
            None => {
                b.as_object_mut().unwrap().remove("tier");
            }
        }
        post(&mut host, &mut rep, &format!("tier {name}"), b, 400, "tier: one of");
        lobby_kept(&mut host, &format!("tier {name}"));
    }
    // the format
    let mut b = base();
    b["format"] = json!("commander");
    post(&mut host, &mut rep, "format commander for the Cali deck", b, 400, "cannot be played at a Commander table");
    lobby_kept(&mut host, "format commander");
    let mut b = base();
    b["format"] = json!(5);
    post(&mut host, &mut rep, "format 5", b, 400, "format: a format key");
    let mut b = base();
    b["format"] = json!("zzz");
    post(&mut host, &mut rep, "format zzz", b, 400, "format: one of");
    let mut b = base();
    b.as_object_mut().unwrap().remove("format");
    let r = post(&mut host, &mut rep, "no format key (the deck's own)", b, 200, "");
    let code2 = code_of(&r);
    assert_eq!(lobby_of(&mut host)["format"], c.format.as_str(), "the deck's own format is written into the lobby");
    // seat / seats are dropped at a pair (as at Commander), not refused
    for (name, k, v) in [
        ("seat", "seat", json!(1)),
        ("seat true", "seat", json!(true)),
        ("seats", "seats", json!([c.cali, c.guest])),
        ("seats wrong", "seats", json!(["x"])),
    ] {
        let mut b = base();
        b[k] = v;
        let r = host.post("/api/new", b);
        rep.push(format!("{name}: {} {}", r.status, r.error()));
        assert!(r.status == 200 || r.status == 400, "{name}: {}", r.status);
    }
    // seat of the wrong type is a 400 whatever the table
    let mut b = base();
    b["seat"] = json!("x");
    post(&mut host, &mut rep, "seat a string", b, 400, "seat: 1 to");
    // a partner key beside `two`: ignored by the lobby
    let mut b = base();
    b["partner"] = json!(c.guest);
    let r = host.post("/api/new", b);
    rep.push(format!("two + partner key: {} {}", r.status, r.error()));
    assert!(r.status == 200 || r.status == 400);
    if r.status == 200 {
        let code = code_of(&r);
        let lb = lobby_of(&mut host);
        assert_eq!(lb["partner"], Value::Null, "a partner key must not pre-seat anyone in a lobby: {lb}");
        let mut friend = s.remote_client().unwrap();
        friend.get("/");
        // joining with a different deck than the "partner" key still works
        let other = MY_DECKS.iter().find(|k| **k != c.cali && c.admits(k, &c.format) && **k != c.guest);
        let deck = other.map(|d| d.to_string()).unwrap_or(c.guest.clone());
        let r = friend.post("/api/join", json!({"code": code, "deck": deck}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        host.quit();
    }
    // truthiness of `two`: the page sends true; python-style values behave like the other formats
    for (name, v, pair) in [
        ("1", json!(1), true),
        ("yes", json!("yes"), true),
        ("[1]", json!([1]), true),
        ("false", json!(false), false),
        ("0", json!(0), false),
        ("empty", json!(""), false),
        ("null", json!(null), false),
    ] {
        let mut b = solo_body(&c, &c.cali, Some(&c.format), json!([]), 51);
        b["two"] = v;
        let r = host.post("/api/new", b);
        rep.push(format!("two = {name}: {} {} {}", r.status, r.error(), r.json()["code"]));
        if pair {
            assert_eq!(r.status, 200, "two = {name}: {}", String::from_utf8_lossy(&r.body));
            assert!(r.json()["code"].is_string(), "two = {name} must open a lobby");
        } else {
            // a one-person game at Cali: one opponent drawn from a tier
            assert!(
                r.json().get("code").is_none(),
                "two = {name} must not open a lobby: {}",
                String::from_utf8_lossy(&r.body)
            );
        }
    }
    host.quit();
    let _ = code2;
    eprintln!("RECORD two at Cali: {rep:#?}");
}

#[test]
fn a_partner_key_on_a_one_person_game_at_cali_is_refused_and_at_commander_unchanged() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    host.get("/");
    let opp = {
        let o = host.get("/api/options").json();
        let ds = o["tiers"][0]["decks"].as_array().unwrap().clone();
        let keys = |f: &str| -> Vec<String> {
            ds.iter()
                .filter(|d| d["formats"].as_array().is_none_or(|a| a.iter().any(|k| k == f)))
                .map(|d| d["key"].as_str().unwrap().to_string())
                .collect()
        };
        (keys(&c.format), keys("commander"))
    };
    // Cali: a partner key of any shape is a 400 with the text, and nothing starts
    for p in [json!(c.guest), json!(c.cali), json!(5), json!([]), json!({}), json!("zzz"), json!("")] {
        let mut b = solo_body(&c, &c.cali, Some(&c.format), json!(opp.0.iter().take(1).collect::<Vec<_>>()), 52);
        b["partner"] = p.clone();
        let r = host.post("/api/new", b);
        assert_eq!(r.status, 400, "partner {p}: {}", String::from_utf8_lossy(&r.body));
        assert_eq!(r.error(), "partner: a friend joins with the table's code", "partner {p}");
        let st = host.get("/api/state").json();
        assert!(st["game"].is_null() && st["lobby"].is_null(), "partner {p}: something started: {st}");
    }
    // `partner: null` is the same as no key
    let mut b = solo_body(&c, &c.cali, Some(&c.format), json!(opp.0.iter().take(1).collect::<Vec<_>>()), 52);
    b["partner"] = json!(null);
    let r = host.post("/api/new", b);
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    host.quit();
    // the same bad key with the Cali deck but without the format key (the deck's own format): the same
    let mut b = solo_body(&c, &c.cali, None, json!(opp.0.iter().take(1).collect::<Vec<_>>()), 52);
    b["partner"] = json!(c.guest);
    let r = host.post("/api/new", b);
    assert_eq!(r.status, 400);
    assert_eq!(r.error(), "partner: a friend joins with the table's code");
    // Commander: unchanged (a partner key seats a second person nobody plays; accepted, the game starts)
    let mut b = solo_body(
        &c,
        &c.cmdr,
        None,
        json!(opp.1.iter().filter(|k| **k != c.cmdr && **k != c.cmdr2).take(2).collect::<Vec<_>>()),
        53,
    );
    b["partner"] = json!(c.cmdr2);
    let r = host.post("/api/new", b);
    eprintln!("RECORD Commander with a partner key: {} {}", r.status, r.error());
    assert_eq!(r.status, 200, "Commander must accept a partner key as before: {}", String::from_utf8_lossy(&r.body));
    let st = host.get("/api/state").json();
    assert_eq!(st["game"]["humans"].as_array().map(|a| a.len()), Some(2));
    host.quit();
}

/// every deck of mine as the host, at every format, in the order of MY_DECKS: a lobby opens exactly when the format
/// admits the deck, including the host that is the first of MY_DECKS and the one that is the only deck of its format
#[test]
fn the_placeholder_partner_never_blocks_a_legal_host() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    host.get("/");
    let mut rep = vec![];
    for (i, h) in MY_DECKS.iter().enumerate() {
        for format in c.seats.iter().map(|f| f.0.clone()) {
            let two = c.seats.iter().find(|f| f.0 == format).unwrap().1 == 2;
            let mut b = two_body(&c, h, Some(&format), if two { json!([]) } else { json!([]) }, 60);
            if !two {
                b.as_object_mut().unwrap().remove("opponents");
            }
            let r = host.post("/api/new", b);
            let want = c.admits(h, &format);
            rep.push(format!("#{i} {h} @ {format}: {} {}", r.status, r.error()));
            if want {
                assert_eq!(r.status, 200, "{h} at {format} should open a lobby: {}", String::from_utf8_lossy(&r.body));
            } else {
                assert_eq!(r.status, 400, "{h} at {format} should be refused: {}", String::from_utf8_lossy(&r.body));
            }
            host.quit();
        }
    }
    let n_cali = MY_DECKS.iter().filter(|k| c.admits(k, &c.format)).count();
    eprintln!(
        "RECORD {} of {} decks are admitted at {}; placeholder results: {rep:#?}",
        n_cali,
        MY_DECKS.len(),
        c.format_name
    );
}

/// design 5: "Commander picks the same deck with the same texts as today". The placeholder was the first other deck of
/// mine; if the opponent list makes THAT deck unusable (the session refuses it), origin/main refused the lobby, and the
/// loop that tries the next deck would open one: a Commander-visible change that design 7 does not list
#[test]
fn a_commander_lobby_whose_opponents_spoil_the_first_placeholder_is_refused_as_before() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    host.get("/");
    let h = c.cmdr.clone();
    let first_other = MY_DECKS.iter().find(|k| **k != h.as_str()).unwrap().to_string();
    let o = host.get("/api/options").json();
    let tier_keys: Vec<String> = o["tiers"][0]["decks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["formats"].as_array().is_none_or(|a| a.iter().any(|k| k == "commander")))
        .map(|d| d["key"].as_str().unwrap().to_string())
        .collect();
    let mut checked = 0;
    for opp in [
        vec![first_other.clone(), tier_keys.iter().find(|k| **k != first_other && **k != h).cloned().unwrap()],
        tier_keys.iter().filter(|k| **k != h).take(2).cloned().collect::<Vec<_>>(),
        vec![h.clone(), tier_keys.iter().find(|k| **k != h).cloned().unwrap()],
        vec![tier_keys[0].clone(), tier_keys[0].clone()],
    ] {
        // what origin/main said: its placeholder was the first other deck, and one Session::new decided
        let e = engine();
        let mut o = practice::session::Options::new(e, &h, &c.tier);
        o.partner = Some(first_other.clone());
        o.opponents = Some(opp.clone());
        o.ai = "adaptive".into();
        let main_says = practice::session::Session::new(o).err();
        let r = host.post("/api/new", two_body(&c, &h, None, json!(opp), 61));
        eprintln!(
            "RECORD Commander host {h}, opponents {opp:?}: origin/main's placeholder {first_other} gives {:?}; now {} {}",
            main_says,
            r.status,
            r.error()
        );
        match main_says {
            Some(why) => {
                assert_eq!(
                    r.status,
                    400,
                    "origin/main refused this lobby ({why}); a Commander lobby must be unchanged: now {} {}",
                    r.status,
                    r.error()
                );
                assert_eq!(r.error(), why, "same text as origin/main");
            }
            None => assert_eq!(r.status, 200, "{}", r.error()),
        }
        checked += 1;
        host.quit();
    }
    assert_eq!(checked, 4);
}

// ------------------------------------------------------------------------------------------------ Undo with two browsers

#[test]
fn the_undo_proposal_flow_with_two_browsers() {
    let s = lan!();
    let c = cat(&mut s.client());
    let (mut host, mut friend) = table(&s, &c, 70);
    let mut third = s.remote_client().unwrap();
    third.get("/");
    play_until(&mut host, &mut friend, 3);
    let mut rep = vec![];

    // (a) an undo of more than the asker has answered: refused, no proposal for the other person
    for n in [json!(999), json!(0), json!(-4), json!("abc"), json!(1.5), json!(null), json!([]), json!(u64::MAX)] {
        for who in [0, 1] {
            let (a, b) = if who == 0 { (&mut host, &mut friend) } else { (&mut friend, &mut host) };
            let r = a.c.post("/api/undo", json!({"n": n}));
            rep.push(format!("undo n={n} by {who}: {} {}", r.status, r.error()));
            assert!(
                r.status == 200 || r.status == 409,
                "undo n={n}: {} {}",
                r.status,
                String::from_utf8_lossy(&r.body)
            );
            if r.status == 200 {
                // a count that reads as 1: a proposal stands; clear it by declining
                let p = b.state()["proposal"].clone();
                assert!(p.is_object(), "{p}");
                let r = b.c.post("/api/approve", json!({"id": p["id"], "yes": false}));
                assert_eq!(r.status, 200);
            } else if n == json!(999) || n == json!(0) || n == json!(-4) {
                assert!(r.error().contains("Nothing to undo yet"), "{}", r.error());
                assert!(b.state()["proposal"].is_null(), "a refused undo must not ask the other person");
            }
        }
    }

    // (b) the person who is NOT deciding asks: the other person's decision is waiting
    let hd = a_decides(&mut host, &mut friend);
    let (before, id) = {
        let (decider, waiter) = split(&mut host, &mut friend, hd);
        let before = decider.pending().unwrap();
        let r = waiter.c.post("/api/undo", json!({"n": 1}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let p = decider.state()["proposal"].clone();
        assert!(p.is_object(), "the deciding person is asked: {p}");
        assert!(waiter.state()["proposal"].is_null(), "the asker is not asked");
        // (c) a second proposal while one stands (from either person)
        let r2 = waiter.c.post("/api/undo", json!({"n": 1}));
        rep.push(format!("second proposal by the asker: {} {}", r2.status, r2.error()));
        assert_eq!(r2.status, 409);
        assert!(r2.error().contains("Still waiting for"), "{}", r2.error());
        let r3 = decider.c.post("/api/undo", json!({"n": 1}));
        rep.push(format!("a proposal by the one who was asked: {} {}", r3.status, r3.error()));
        assert_eq!(r3.status, 409, "{}", String::from_utf8_lossy(&r3.body));
        let r4 = waiter.c.post("/api/tryit", json!({"n": 0}));
        rep.push(format!("a Try it while one stands: {} {}", r4.status, r4.error()));
        assert_eq!(r4.status, 409);
        // (d) who may approve: not the asker, not a third browser, not a stale id
        let id = p["id"].clone();
        for (name, who) in [("the asker", 0), ("a third browser", 1)] {
            let r = if who == 0 {
                waiter.c.post("/api/approve", json!({"id": id, "yes": true}))
            } else {
                third.post("/api/approve", json!({"id": id, "yes": true}))
            };
            rep.push(format!("approve by {name}: {} {}", r.status, r.error()));
            assert_eq!(r.status, 409, "{name}: {}", String::from_utf8_lossy(&r.body));
            assert!(r.error().contains("no longer open"), "{}", r.error());
        }
        let r = decider.c.post("/api/approve", json!({"id": 999999, "yes": true}));
        assert_eq!(r.status, 409);
        for (name, v) in [("id string", json!("1")), ("no id", Value::Null)] {
            let mut body = json!({"yes": true});
            if !v.is_null() {
                body["id"] = v;
            }
            let r = decider.c.post("/api/approve", body);
            assert_eq!(r.status, 409, "{name}");
        }
        assert!(decider.state()["proposal"].is_object(), "bad approvals must not clear the proposal");
        assert!(third.get("/api/state").json()["proposal"].is_null());
        let r = third.post("/api/undo", json!({"n": 1}));
        assert_eq!((r.status, r.error().as_str()), (409, "You have no seat in this game."));
        // (e) a decline: nothing changes for the decider, the asker is told
        let since = waiter.state()["last_event"].as_u64().unwrap();
        let mut asker_stream = waiter.c.stream(since, None);
        let r = decider.c.post("/api/approve", json!({"id": id, "yes": false}));
        assert_eq!(r.status, 200);
        let evs = asker_stream.until(30, |e| e["kind"] == "declined");
        let declined = evs.last().unwrap();
        assert!(declined["text"].as_str().unwrap().contains("said no to the Undo"), "{declined}");
        assert!(decider.state()["proposal"].is_null());
        let after = decider.pending().unwrap();
        assert_eq!(after["id"], before["id"], "a declined Undo must leave the waiting decision as it was");
        // (f) an approved one
        let r = waiter.c.post("/api/undo", json!({"n": 1}));
        assert_eq!(r.status, 200);
        let p = decider.state()["proposal"].clone();
        let r = decider.c.post("/api/approve", json!({"id": p["id"], "yes": true}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        (before, id)
    };
    let _ = id;
    assert!(wait_until(60, || host.pending().is_some() || friend.pending().is_some()));
    // the answer to the decision before the Undo is stale now
    {
        let (decider, _) = split(&mut host, &mut friend, hd);
        let r = decider.c.answer(before["id"].as_u64().unwrap(), json!({"do": "pass"}));
        rep.push(format!("an answer to the decision before the Undo: {} {}", r.status, r.error()));
        assert_eq!(r.status, 409);
        assert!(r.error().contains("already been answered"), "{}", r.error());
    }
    // exactly one person is asked after the replay, and nobody holds a request that is not theirs
    std::thread::sleep(Duration::from_millis(300));
    let (hp, fp) = (host.pending(), friend.pending());
    assert!(hp.is_some() ^ fp.is_some(), "exactly one person is asked: host {hp:?} friend {fp:?}");
    // the game keeps going for both
    play_until(&mut host, &mut friend, 6);
    // (g) hint and review per seat
    let hd = a_decides(&mut host, &mut friend);
    {
        let (decider, waiter) = split(&mut host, &mut friend, hd);
        let r = decider.c.post("/api/hint", json!({}));
        rep.push(format!("hint by the deciding seat: {}", r.status));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let r = waiter.c.post("/api/hint", json!({}));
        assert_eq!((r.status, r.error().as_str()), (409, "There is no decision waiting for you."));
        let r = third.post("/api/hint", json!({}));
        assert_eq!((r.status, r.error().as_str()), (409, "You have no seat in this game."));
        for p in [&mut *decider, &mut *waiter] {
            let r = p.c.get("/api/review");
            assert_eq!((r.status, r.error().as_str()), (409, "The review opens when the game ends."));
        }
        let r = third.get("/api/review");
        assert_eq!((r.status, r.error().as_str()), (409, "You have no seat in this game."));
    }
    eprintln!("RECORD undo proposal flow: {rep:#?}");
    host.c.quit();
}

// ------------------------------------------------------------------------------------------------ saves, warn, JSON

/// a save of a two-person table at `k` answers, from a session driven in this process (no game is started on the server)
fn make_save(c: &Cat, k: usize) -> Value {
    let e = engine();
    let mut o = practice::session::Options::new(e, &c.cali, "");
    o.partner = Some(c.guest.clone());
    o.seed = Some(7);
    o.ai = "adaptive".into();
    o.compare = false;
    o.views = true;
    o.max_rounds = 3;
    let s = practice::session::Session::new(o).unwrap();
    s.start();
    let mut cursor = 0;
    let mut played = vec![];
    let mut given = 0;
    let end = Instant::now() + Duration::from_secs(120);
    while given < k {
        assert!(Instant::now() < end);
        let (cu, evs) = s.events_after(cursor, Duration::from_millis(100));
        cursor = cu;
        if let Some(ev) = evs.iter().rev().find(|e| e["kind"] == "request") {
            let seat = ev["seat"].as_str().unwrap().to_string();
            let mut e2 = json!({"request": ev["request"].clone()});
            e2["view"] = ev["request"]["data"]["view"].clone();
            let a = bot(&e2, &mut played);
            s.answer(&seat, &a);
            given += 1;
        }
    }
    // wait for the decision after the last answer
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        assert!(Instant::now() < end);
        let (cu, evs) = s.events_after(cursor, Duration::from_millis(100));
        cursor = cu;
        if evs.iter().any(|e| e["kind"] == "request") {
            break;
        }
    }
    let v = s.saved();
    s.close(Duration::from_secs(10));
    v
}

#[test]
fn tampered_saves_through_the_load_route_reach_both_people_as_a_warning_or_a_clear_refusal() {
    let s = lan!();
    let c = cat(&mut s.client());
    let v = make_save(&c, 8);
    let name = |tag: &str| format!("29990101-000000-adv-{tag}.json");
    let write = |tag: &str, v: &Value| {
        std::fs::create_dir_all(&s.saves).unwrap();
        std::fs::write(s.saves.join(name(tag)), v.to_string()).unwrap();
    };
    let mut host = s.client();
    host.get("/");
    let mut rep = vec![];
    // (1) as written: the saves list shows no tier; loading opens a lobby of the format and the pinned partner
    write("good", &v);
    let list = host.get("/api/saves").json();
    let row = list["saves"].as_array().unwrap().iter().find(|x| x["name"] == name("good").as_str()).cloned().unwrap();
    assert_eq!(row["tier"], Value::Null, "{row}");
    assert_eq!(row["format"], c.format.as_str());
    let r = host.post("/api/load", json!({"name": name("good"), "images": false}));
    let code = code_of(&r);
    let lb = lobby_of(&mut host);
    assert_eq!(
        (lb["loaded"].clone(), lb["format"].clone(), lb["tier"].clone()),
        (json!(true), json!(c.format), Value::Null)
    );
    assert_eq!(lb["deck"], c.cali.as_str());
    let partner = lb["partner"].as_str().unwrap().to_string();
    // a guest with another deck, with the host's deck, a refused deck: all 409, the lobby stays
    let mut friend = s.remote_client().unwrap();
    friend.get("/");
    for d in MY_DECKS.iter().filter(|k| **k != partner) {
        let r = friend.post("/api/join", json!({"code": code, "deck": d}));
        assert_eq!(r.status, 409, "{d}");
        assert!(
            r.error().contains("This saved game goes on with the") || r.error().contains("deck:"),
            "{d}: {}",
            r.error()
        );
        assert_eq!(lobby_of(&mut host)["code"], code.as_str());
    }
    let r = friend.post("/api/join", json!({"code": code, "deck": partner}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let mut hs = host.stream(0, None);
    let evs = hs.until(60, |e| e["kind"] == "request" || e["kind"] == "waiting");
    assert!(evs.iter().all(|e| e["kind"] != "warn"), "an untouched save must not warn");
    host.quit();
    // (2) answer_seats swapped: both people get the warning, the game waits live
    let mut sw = v.clone();
    let seats: Vec<Value> = v["answer_seats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| if x == &json!(c.cali) { json!(c.guest) } else { json!(c.cali) })
        .collect();
    sw["answer_seats"] = json!(seats);
    write("swapped", &sw);
    let r = host.post("/api/load", json!({"name": name("swapped"), "images": false}));
    let code = code_of(&r);
    let mut friend = s.remote_client().unwrap();
    friend.get("/");
    let r = friend.post("/api/join", json!({"code": code, "deck": partner}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let mut warns = vec![];
    for (who, cl) in [("host", &mut host), ("friend", &mut friend)] {
        let mut st = cl.stream(0, None);
        let evs = st.until(60, |e| e["kind"] == "request" || e["kind"] == "waiting");
        let w: Vec<String> =
            evs.iter().filter(|e| e["kind"] == "warn").map(|e| e["text"].as_str().unwrap().to_string()).collect();
        assert_eq!(
            w.len(),
            1,
            "{who} must see exactly one warn event: {:?}",
            evs.iter().map(|e| e["kind"].clone()).collect::<Vec<_>>()
        );
        assert!(w[0].starts_with("The replay of this game went a different way at decision 1"), "{}", w[0]);
        warns.push(w[0].clone());
    }
    rep.push(format!("swapped: both got {:?}", warns[0]));
    host.quit();
    // (3) refusals through the route
    let mut bad = |tag: &str, f: &dyn Fn(&mut Value)| {
        let mut x = v.clone();
        f(&mut x);
        write(tag, &x);
        let r = host.post("/api/load", json!({"name": name(tag), "images": false}));
        rep.push(format!("{tag}: {} {}", r.status, r.error()));
        assert_eq!(r.status, 409, "{tag}: {}", String::from_utf8_lossy(&r.body));
        assert!(r.error().starts_with("That saved game can't be loaded: "), "{tag}: {}", r.error());
        std::fs::remove_file(s.saves.join(name(tag))).ok();
    };
    bad("as-null", &|x| x["answer_seats"] = json!(null));
    bad("as-short", &|x| x["answer_seats"] = json!([c.cali]));
    bad("as-nobody", &|x| x["answer_seats"] = json!(vec!["nobody"; 8]));
    bad("tier-null", &|x| x["tier"] = json!(null));
    bad("tier-5", &|x| x["tier"] = json!(5));
    bad("partner-null", &|x| x["partner"] = json!(null));
    bad("partner-5", &|x| x["partner"] = json!(5));
    bad("format-5", &|x| x["format"] = json!(5));
    bad("old-commander", &|x| {
        x.as_object_mut().unwrap().remove("format");
        x.as_object_mut().unwrap().remove("partner");
        x.as_object_mut().unwrap().remove("answer_seats");
        x["tier"] = json!(c.tier);
    });
    // a one-person save with no tier, and one with a tier at the Cali deck
    bad("one-person-no-tier", &|x| {
        x.as_object_mut().unwrap().remove("partner");
        x.as_object_mut().unwrap().remove("answer_seats");
    });
    let r = host.post("/api/load", json!({"name": "../../etc/passwd"}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "No such saved game.");
    for n in [json!(null), json!(5), json!(""), json!([]), json!("a/b.json")] {
        let r = host.post("/api/load", json!({"name": n}));
        assert_eq!(r.status, 409, "name {n}");
    }
    for t in ["good", "swapped"] {
        std::fs::remove_file(s.saves.join(name(t))).ok();
    }
    eprintln!("RECORD tampered saves through /api/load: {rep:#?}");
}

#[test]
fn the_commander_pair_json_has_the_keys_it_always_had_plus_the_lobby_format() {
    let s = lan!();
    let c = cat(&mut s.client());
    let mut host = s.client();
    let mut friend = s.remote_client().unwrap();
    host.get("/");
    friend.get("/");
    let o = host.get("/api/options").json();
    let opp: Vec<String> = o["tiers"][0]["decks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["formats"].as_array().is_none_or(|a| a.iter().any(|k| k == "commander")))
        .map(|d| d["key"].as_str().unwrap().to_string())
        .filter(|k| *k != c.cmdr && *k != c.cmdr2)
        .take(2)
        .collect();
    let r = host.post("/api/new", two_body(&c, &c.cmdr, None, json!(opp), 80));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let keys = |v: &Value| -> Vec<String> {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&r.json()), vec!["addresses", "code", "ok"], "/api/new two");
    let code = r.json()["code"].as_str().unwrap().to_string();
    let lb = lobby_of(&mut host);
    assert_eq!(
        keys(&lb),
        vec!["addresses", "code", "deck", "format", "loaded", "partner", "tier"],
        "host's lobby JSON"
    );
    assert_eq!(lb["format"], "commander");
    assert_eq!(lb["tier"], c.tier.as_str());
    assert_eq!(lb["partner"], Value::Null);
    let fl = lobby_of(&mut friend);
    assert_eq!(keys(&fl), vec!["deck", "format", "loaded", "partner", "tier"], "guest's lobby JSON");
    let r = friend.post("/api/join", json!({"code": code, "deck": c.cmdr2}));
    assert_eq!(r.status, 200);
    assert_eq!(keys(&r.json()), vec!["ok"], "/api/join");
    let st = host.get("/api/state").json();
    assert_eq!(
        keys(&st["game"]),
        vec!["ai", "autopass", "deck", "format", "humans", "names", "profile", "seats", "seed", "tier", "tools", "you"],
        "/api/state game"
    );
    assert_eq!(st["game"]["tier"], c.tier.as_str());
    assert_eq!(st["game"]["seats"].as_array().unwrap().len(), 4);
    assert_eq!(st["game"]["format"], "commander");
    assert_eq!(
        keys(&st),
        vec!["busy", "game", "host", "images", "lan", "last_event", "lobby", "pending", "printed", "proposal", "view"],
        "/api/state"
    );
    // the saves list row for a Commander pair has a tier
    host.quit();
}

// ------------------------------------------------------------------------------------------------ threads, stress

#[test]
fn quit_with_both_people_waiting_leaves_no_game_threads() {
    let s = lan!();
    let c = cat(&mut s.client());
    // whatever earlier tests left has been told to quit by server(); give it a moment
    wait_until(20, || practice_threads() == 0);
    let base = practice_threads();
    let mut leaks = vec![];
    for round in 0..6 {
        let (mut host, mut friend) = table(&s, &c, 90 + round);
        play_until(&mut host, &mut friend, 2 + round as usize);
        // each person's stream open (the pages are there), compare on for the second half
        let mut hs = host.c.stream(0, None);
        let mut fs = friend.c.stream(0, None);
        let _ = (hs.next(Duration::from_millis(200)), fs.next(Duration::from_millis(200)));
        match round % 3 {
            0 => {
                host.c.quit();
            }
            1 => {
                // the host starts another table: the first is replaced
                let r = host.c.post("/api/new", two_body(&c, &c.cali, Some(&c.format), json!([]), 91));
                assert_eq!(r.status, 200);
                host.c.quit();
            }
            _ => {
                // the guest's page asks for things while the host quits
                let _ = friend.c.post("/api/undo", json!({"n": 1}));
                host.c.quit();
            }
        }
        let gone = wait_until(30, || practice_threads() <= base);
        if !gone {
            leaks.push(format!(
                "round {round}: {} practice thread(s) alive 30 s after Quit (base {base})",
                practice_threads()
            ));
        }
        let st = friend.c.get("/api/state").json();
        assert!(st["game"].is_null(), "round {round}: the guest still has a game after Quit: {}", st["game"]);
        let r = friend.c.post("/api/answer", json!({"id": 1, "answer": true}));
        assert_eq!(r.status, 409, "round {round}: an answer after Quit");
    }
    eprintln!("RECORD quit with both waiting: leaks {leaks:?}");
    assert!(leaks.is_empty(), "{leaks:?}");
}

/// run `f` on a thread; the channel gets "name ok" or "name PANIC text" when it ends (so a hang and a panic differ)
fn guarded(
    tx: std::sync::mpsc::Sender<String>,
    name: String,
    f: impl FnOnce() + Send + 'static,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        let _ = tx.send(match r {
            Ok(()) => format!("{name} ok"),
            Err(p) => format!(
                "{name} PANIC {}",
                p.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default()
            ),
        });
    })
}

#[test]
fn lobby_and_join_stress_with_a_watchdog_never_hangs() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
    let s = lan!();
    let c = Arc::new(cat(&mut s.client()));
    wait_until(20, || practice_threads() == 0);
    let base = practice_threads();
    let code: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let opened = Arc::new(AtomicUsize::new(0));
    let joined = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = channel::<String>();
    let mut hs = vec![];
    // the host: open a Cali table, sometimes a Commander one, sometimes Quit
    {
        let (mut host, code, stop, opened, c) = (s.client(), code.clone(), stop.clone(), opened.clone(), c.clone());
        hs.push(guarded(tx.clone(), "host".into(), move || {
            host.get("/");
            for i in 0..60u64 {
                let r = match i % 4 {
                    0 | 1 => host.post("/api/new", two_body(&c, &c.cali, Some(&c.format), json!([]), 100 + i)),
                    2 => {
                        let mut b = two_body(&c, &c.cmdr, None, json!([]), 100 + i);
                        b.as_object_mut().unwrap().remove("opponents");
                        host.post("/api/new", b)
                    }
                    _ => host.quit(),
                };
                assert!(r.status < 500, "host: {} {}", r.status, String::from_utf8_lossy(&r.body));
                if r.status == 200 {
                    if let Some(cd) = r.json()["code"].as_str() {
                        *code.lock().unwrap() = Some(cd.to_string());
                        opened.fetch_add(1, SeqCst);
                    }
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            stop.store(true, SeqCst);
        }));
    }
    // joiners: each its own browser; good decks, a Commander deck, the host's deck, an unknown one
    for j in 0..6usize {
        let (mut b, code, stop, joined, c) =
            (s.remote_client().unwrap(), code.clone(), stop.clone(), joined.clone(), c.clone());
        hs.push(guarded(tx.clone(), format!("joiner {j}"), move || {
            b.get("/");
            let decks = [c.guest.clone(), c.cmdr2.clone(), c.cali.clone(), c.cmdr.clone(), "zzz".to_string()];
            let mut i = j;
            while !stop.load(SeqCst) {
                let cd = code.lock().unwrap().clone().unwrap_or_else(|| "000000".into());
                let r = b.post("/api/join", json!({"code": cd, "deck": decks[i % decks.len()]}));
                assert!(r.status == 200 || r.status == 409, "join: {} {}", r.status, String::from_utf8_lossy(&r.body));
                if r.status == 200 {
                    joined.fetch_add(1, SeqCst);
                }
                i += 1;
            }
        }));
    }
    // pollers
    for p in 0..3usize {
        let (mut b, stop) = (if p == 0 { s.client() } else { s.remote_client().unwrap() }, stop.clone());
        hs.push(guarded(tx.clone(), format!("poller {p}"), move || {
            while !stop.load(SeqCst) {
                for path in ["/api/state", "/api/saves", "/api/review"] {
                    let r = b.get(path);
                    assert!(r.status < 500, "{path}: {}", r.status);
                }
            }
        }));
    }
    drop(tx);
    let want = hs.len();
    let end = Instant::now() + Duration::from_secs(150);
    let mut done = vec![];
    while done.len() < want {
        let left = end.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok(m) => done.push(m),
            Err(_) => panic!(
                "HANG: after 150 s only {done:?} of {want} threads ended (join under the hub lock vs open_lobby/quit?)"
            ),
        }
    }
    let (o, j) = (opened.load(SeqCst), joined.load(SeqCst));
    eprintln!("RECORD stress: {o} lobbies opened, {j} joins accepted; {done:?}");
    assert!(done.iter().all(|m| m.ends_with(" ok")), "{done:?}");
    assert!(j <= o, "more joins accepted ({j}) than lobbies opened ({o}): a lobby was taken twice");
    s.client().quit();
    assert!(
        wait_until(30, || practice_threads() <= base),
        "{} practice threads left after the stress (base {base})",
        practice_threads()
    );
}

// ------------------------------------------------------------------------------------------------ a pre-existing race, reached over HTTP

/// Two Undo requests at the same moment in a ONE-person game (a double click, two tabs): `Session::rewind` runs unlocked
/// between its two critical sections (origin/main has the same code), so the second can slice `answers[..to]` after the
/// first has cleared it. Not a C5c change: recorded because a table of two people reaches the same code with two
/// approvals in a row. The test fails when the server's handler panics or the game never asks again.
#[test]
fn two_undo_requests_at_once_leave_the_game_playable() {
    let s = lan!();
    let c = cat(&mut s.client());
    let o = s.client().get("/api/options").json();
    let opp = o["tiers"].as_array().unwrap().iter().find(|t| t["key"] == c.tier.as_str()).unwrap()["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["formats"].as_array().is_none_or(|a| a.iter().any(|k| *k == c.format)))
        .map(|x| x["key"].as_str().unwrap().to_string())
        .unwrap();
    let mut failures = vec![];
    for round in 0..8 {
        let mut host = Person::new(s.client());
        host.c.get("/");
        let r = host.c.post("/api/new", solo_body(&c, &c.cali, Some(&c.format), json!([opp]), 120 + round));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let end = Instant::now() + Duration::from_secs(120);
        while host.answered < 6 {
            assert!(Instant::now() < end);
            if !host.answer_waiting() {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        let bar = Arc::new(std::sync::Barrier::new(2));
        let (tx, rx) = channel();
        for _ in 0..2 {
            let (mut cl, bar, tx) = (host.c.clone_jar(), bar.clone(), tx.clone());
            guarded(tx, "undo".into(), move || {
                bar.wait();
                let r = cl.post("/api/undo", json!({"n": 1}));
                assert!(r.status == 200 || r.status == 409, "undo: {}", r.status);
            });
        }
        let mut res = vec![];
        let end = Instant::now() + Duration::from_secs(60);
        while res.len() < 2 {
            match rx.recv_timeout(end.saturating_duration_since(Instant::now())) {
                Ok(m) => res.push(m),
                Err(_) => {
                    res.push("HANG".into());
                    break;
                }
            }
        }
        // the game must ask again (the seat's request), and the state route must answer
        let asked = wait_until(30, || {
            let st = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| host.pending())).ok().flatten();
            st.is_some()
        });
        if !asked || res.iter().any(|m| !m.ends_with(" ok")) {
            failures.push(format!("round {round}: requests {res:?}, asked again: {asked}"));
        }
        host.c.quit();
    }
    eprintln!("RECORD double undo in a one-person game: failures {failures:#?}");
    assert!(failures.is_empty(), "{failures:#?}");
}
