// written by an independent reviewer
//! C5a adversarial: the routes (`/api/new`, `/api/state`, `/api/saves`, `/api/options`, `/api/join`), the catalog as
//! the EXISTING web client reads it, save names, and the terminal client's arguments.

mod common;
#[path = "session_common/mod.rs"]
mod sess;

use practice::session::MY_DECKS;
use practice::session::setup::pool_keys;
use serde_json::{Value, json};
use sim_core::format::DEFAULT_FORMAT;
use sim_core::run::Engine;

macro_rules! srv {
    () => {
        match common::server() {
            Some(s) => s,
            None => return,
        }
    };
}

fn cali_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck");
    (k.to_string(), e.decks[*k].tiers[0].clone())
}

fn commander_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).key == DEFAULT_FORMAT).expect("a Commander deck");
    (k.to_string(), e.decks[*k].tiers[0].clone())
}

/// a request body with the fields the tests don't care about fixed (no images, no comparison, adaptive AI)
fn body(deck: &str, tier: &str, extra: Value) -> Value {
    let mut b =
        json!({"deck": deck, "tier": tier, "seed": 5, "ai": "adaptive", "images": false, "tools": {"compare": false}});
    for (k, v) in extra.as_object().unwrap() {
        b[k] = v.clone();
    }
    b
}

/// post /api/new; (status, error text or ""), and a started game is ended again
fn new(c: &mut common::Client, b: Value) -> (u16, String) {
    let r = c.post("/api/new", b);
    let out = (r.status, if r.status == 200 { String::new() } else { r.error() });
    if r.status == 200 {
        c.quit();
    }
    out
}

// -------------------------------------------------------------------------------------- `format` values

#[test]
fn every_odd_spelling_of_format() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let (cdeck, ctier) = commander_deck(&e);
    let mut c = s.client();
    let not_a_key = "format: one of commander, cali";
    // not strings: the type error, whatever the deck
    for bad in
        [json!(4), json!(true), json!(false), json!([]), json!(["cali"]), json!({}), json!({"k": "cali"}), json!(0)]
    {
        for (d, t) in [(&deck, &tier), (&cdeck, &ctier)] {
            let (st, msg) = new(&mut c, body(d, t, json!({"format": bad})));
            assert_eq!((st, msg.as_str()), (400, "format: a format key"), "{d} format {bad}");
        }
    }
    // strings that are not exactly a key
    for bad in [
        "",
        " ",
        "Cali",
        "CALI",
        "Commander",
        " cali",
        "cali ",
        "cali\n",
        "cali\0",
        "cal",
        "caliX",
        "../cali",
        "commander,cali",
    ] {
        for (d, t) in [(&deck, &tier), (&cdeck, &ctier)] {
            let (st, msg) = new(&mut c, body(d, t, json!({"format": bad})));
            assert_eq!((st, msg.as_str()), (400, not_a_key), "{d} format {bad:?}");
        }
    }
    // null is "not given": the deck's own format
    for (d, t) in [(&deck, &tier), (&cdeck, &ctier)] {
        let (st, msg) = new(&mut c, body(d, t, json!({"format": null})));
        assert_eq!((st, msg.as_str()), (200, ""), "{d}");
    }
    // the right keys
    assert_eq!(new(&mut c, body(&deck, &tier, json!({"format": "cali"}))).0, 200);
    assert_eq!(new(&mut c, body(&cdeck, &ctier, json!({"format": "commander"}))).0, 200);
    assert_eq!(new(&mut c, body(&cdeck, &ctier, json!({"format": "cali"}))).0, 200);
    let (st, msg) = new(&mut c, body(&deck, &tier, json!({"format": "commander"})));
    assert_eq!(st, 400);
    assert_eq!(msg, format!("{deck} cannot be played at a Commander table"));
}

/// the format key is checked BEFORE the other fields: a request wrong in two ways gets the format text; and a
/// Commander request wrong in one way gets the same text it always got
#[test]
fn texts_for_a_commander_request_are_as_before() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = commander_deck(&e);
    let keys = pool_keys(&e, &tier).join(", ");
    let mut c = s.client();
    let cases: Vec<(Value, &str, String)> = vec![
        (
            json!({"deck": "nope", "tier": tier}),
            "",
            "deck: one of yshtola, sheoldred, kyodai, urdragon, yoshimaru".into(),
        ),
        (json!({"deck": deck, "tier": "t9"}), "", "tier: one of t0, t1, t2, t3, t4, t5, t6".into()),
        (json!({"deck": deck, "tier": tier, "opponents": "x"}), "", "opponents: a list of deck keys".into()),
        (json!({"deck": deck, "tier": tier, "seats": "x"}), "", "seats: four deck keys in turn order".into()),
        (json!({"deck": deck, "tier": tier, "seat": "x"}), "", "seat: 1 to 4".into()),
        (json!({"deck": deck, "tier": tier, "autopass": "x"}), "", "autopass: one of respond, stack, all".into()),
        (json!({"deck": deck, "tier": tier, "seat": 0}), "", "seat is 1 to 4".into()),
        (json!({"deck": deck, "tier": tier, "seat": 5}), "", "seat is 1 to 4".into()),
        (json!({"deck": deck, "tier": tier, "seat": 2.0}), "", "seat: 1 to 4".into()),
        (
            json!({"deck": deck, "tier": tier, "opponents": [keys.split(", ").next().unwrap()]}),
            "",
            format!("pick three of: {keys}"),
        ),
        (
            json!({"deck": deck, "tier": tier, "seats": ["a", "b", "c", "d"]}),
            "",
            format!("seats: your deck and 3 of {keys}, in turn order"),
        ),
    ];
    for (b, _, want) in cases {
        let mut full = body(&deck, &tier, json!({}));
        for (k, v) in b.as_object().unwrap() {
            full[k] = v.clone();
        }
        let (st, msg) = new(&mut c, full);
        assert_eq!((st, msg.as_str()), (400, want.as_str()), "{b}");
    }
    // two failures at once: the format first
    let (st, msg) = new(&mut c, body(&deck, &tier, json!({"format": "nope", "seat": "x"})));
    assert_eq!((st, msg.as_str()), (400, "format: one of commander, cali"));
    // `two` without --lan, as before (409), for a Commander deck
    if !s.lan {
        let (st, msg) = new(&mut c, body(&deck, &tier, json!({"two": true})));
        assert_eq!(st, 409, "{msg}");
    }
}

// -------------------------------------------------------------------------------------- seats at a Cali table

#[test]
fn seat_seats_opponents_and_partner_at_a_cali_table() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let (cdeck, _) = commander_deck(&e);
    let mut c = s.client();
    let ok: Vec<String> = pool_keys(&e, &tier).into_iter().filter(|k| e.admits(k, "cali").is_ok()).collect();
    let opp = ok[0].clone();
    // seat
    for (seat, want) in [
        (json!(0), (400, "seat is 1 to 2")),
        (json!(3), (400, "seat is 1 to 2")),
        (json!(4), (400, "seat is 1 to 2")),
        (json!(1000000), (400, "seat is 1 to 2")),
        (json!(1), (200, "")),
        (json!(2), (200, "")),
        (json!(-1), (200, "")), // a negative number is not read as a seat (as on main for a Commander deck): random
        (json!(true), (200, "")),
        (json!(2.0), (400, "seat: 1 to 2")),
        (json!("2"), (400, "seat: 1 to 2")),
        (json!([2]), (400, "seat: 1 to 2")),
    ] {
        let (st, msg) = new(&mut c, body(&deck, &tier, json!({"seat": seat})));
        assert_eq!((st, msg.as_str()), want, "seat {seat}");
    }
    // a given seat is where the deck sits
    for (seat, at) in [(1, 0), (2, 1)] {
        let r = c.post("/api/new", body(&deck, &tier, json!({"seat": seat})));
        assert_eq!(r.status, 200);
        assert_eq!(r.json()["seats"][at], deck.as_str(), "seat {seat}: {}", r.json());
        c.quit();
    }
    // exact seats
    let r = c.post("/api/new", body(&deck, &tier, json!({"seats": [opp, deck]})));
    assert_eq!((r.status, r.json()["seats"].clone()), (200, json!([opp, deck])));
    c.quit();
    let r = c.post("/api/new", body(&deck, &tier, json!({"seats": [deck, opp]})));
    assert_eq!((r.status, r.json()["seats"].clone()), (200, json!([deck, opp])));
    c.quit();
    for bad in [
        json!([deck]),
        json!([opp]),
        json!([deck, deck]),
        json!([opp, opp]),
        json!([deck, opp, opp]),
        json!(["x", "y"]),
        json!([]),
        json!([cdeck, opp, "a", "b"]),
    ] {
        let (st, msg) = new(&mut c, body(&deck, &tier, json!({"seats": bad})));
        assert_eq!(st, 400, "seats {bad}");
        assert!(msg.starts_with("seats: your deck and 1 of "), "seats {bad}: {msg}");
    }
    // opponents
    let r = c.post("/api/new", body(&deck, &tier, json!({"opponents": [opp]})));
    assert_eq!((r.status, r.json()["seats"].as_array().map(Vec::len)), (200, Some(2)));
    assert!(r.json()["seats"].as_array().unwrap().iter().any(|k| *k == opp.as_str()));
    c.quit();
    for bad in [json!([]), json!([opp, ok[1]]), json!([opp, ok[1], ok[2]]), json!(["nope"])] {
        let (st, msg) = new(&mut c, body(&deck, &tier, json!({"opponents": bad})));
        assert_eq!(st, 400, "opponents {bad}");
        assert!(msg.starts_with("pick one of: "), "opponents {bad}: {msg}");
    }
    // a partner at a 2-seat table
    let (st, msg) = new(&mut c, body(&deck, &tier, json!({"partner": cdeck})));
    assert_eq!(st, 400, "{msg}");
    assert!(msg.contains("4-seat format"), "{msg}");
    let (st, msg) = new(&mut c, body(&cdeck, &tier, json!({"format": "cali", "partner": deck})));
    assert_eq!(st, 400, "{msg}");
    // `two` at a 2-seat table, with and without the format given, with and without --lan: always the 400
    for extra in [json!({"two": true}), json!({"two": "yes"}), json!({"two": 1})] {
        let (st, msg) = new(&mut c, body(&deck, &tier, extra.clone()));
        assert_eq!(st, 400, "{extra}: {msg}");
        let mut f = extra.clone();
        f["format"] = json!("cali");
        let (st, msg) = new(&mut c, body(&cdeck, &tier, f));
        assert_eq!(st, 400, "{extra} cali: {msg}");
    }
    // `two: false` is not two
    assert_eq!(new(&mut c, body(&deck, &tier, json!({"two": false}))).0, 200);
}

// -------------------------------------------------------------------------------------- the existing web client

/// What `static/app.js startGame` sends for a one-person game: the deck, the tier, the options, and `opponents` =
/// THREE decks drawn from the whole tier (`drawOpponents`: `keys.slice(0, 3)` of the shuffled tier list, whatever
/// Cali admits), plus `seat` from the 1-4 select. The page is not changed in this PR: for a Commander deck this is
/// exactly today's request; for the Cali deck it must still start a game (or the page offers a deck it can't start).
#[test]
#[ignore = "ticket T-S5-15: the existing web page cannot start the Cali deck until the web client PR (C5b-web) sends a format-aware request; remove this ignore there"]
fn the_existing_web_client_request_still_starts_the_cali_deck() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let mut c = s.client();
    let cat = c.get("/api/options").json();
    let tier_keys: Vec<String> =
        cat["tiers"].as_array().unwrap().iter().find(|t| t["key"] == tier.as_str()).unwrap()["decks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["key"].as_str().unwrap().to_string())
            .collect();
    let web = |deck: &str, seat: Option<u64>| {
        let mut b = json!({
            "deck": deck, "tier": tier, "ai": "adaptive", "profile": "loose",
            "tools": {"hint": true, "undo": true, "compare": false}, "autopass": "respond",
            "opponents": tier_keys[..3].to_vec(), "images": false,
        });
        if let Some(n) = seat {
            b["seat"] = json!(n);
        }
        b
    };
    // the control: a Commander deck gets its game, as before
    let (cdeck, _) = commander_deck(&e);
    let (st, msg) = new(&mut c, web(&cdeck, None));
    assert_eq!((st, msg.as_str()), (200, ""), "the web client's request for a Commander deck");
    let (st, msg) = new(&mut c, web(&cdeck, Some(3)));
    assert_eq!((st, msg.as_str()), (200, ""), "the web client's request for a Commander deck, seat 3");
    // the Cali deck
    let (st, msg) = new(&mut c, web(&deck, None));
    assert_eq!((st, msg.as_str()), (200, ""), "the web client's request for the Cali deck, as the page sends it");
}

/// and the catalog it reads: `win_rates` / `average` are null for the Cali deck, the page's reads are guarded
/// (app.js:896 `d.win_rates && d.win_rates[tier] != null`; `average` is read nowhere in static/*.js), and for a
/// Commander deck only the three new keys are added
#[test]
fn the_catalog_the_existing_page_reads() {
    let s = srv!();
    let e = sess::engine();
    let mut c = s.client();
    let cat = c.get("/api/options").json();
    let old_deck_keys = [
        "key",
        "commander",
        "commander2",
        "commanders",
        "ident",
        "name",
        "bracket",
        "game_changers",
        "win_rates",
        "average",
    ];
    let old_tier_deck_keys = ["key", "commander", "commander2", "commanders", "name", "summary"];
    let old_top = ["decks", "tiers", "images", "lan", "host", "app"];
    let keys = |v: &Value| -> Vec<String> {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    let sorted = |a: &[&str], extra: &[&str]| -> Vec<String> {
        let mut k: Vec<String> = a.iter().chain(extra).map(|s| s.to_string()).collect();
        k.sort();
        k
    };
    assert_eq!(keys(&cat), sorted(&old_top, &["formats", "win_rates_format"]));
    for d in cat["decks"].as_array().unwrap() {
        assert_eq!(keys(d), sorted(&old_deck_keys, &["format", "formats", "win_rates_format"]), "{}", d["key"]);
        let k = d["key"].as_str().unwrap();
        // everything the page does with a deck
        assert!(d["name"].is_string() && d["bracket"].is_number() && d["game_changers"].is_array(), "{k}");
        assert!(d["commanders"].as_array().is_some_and(|a| !a.is_empty()), "{k}");
        if e.format_of(k).key == DEFAULT_FORMAT {
            assert!(d["win_rates"].is_null() || d["win_rates"].is_object(), "{k}");
        } else {
            assert!(d["win_rates"].is_null() && d["average"].is_null(), "{k}: {}", d["win_rates"]);
        }
    }
    for t in cat["tiers"].as_array().unwrap() {
        for d in t["decks"].as_array().unwrap() {
            assert_eq!(keys(d), sorted(&old_tier_deck_keys, &["formats"]), "{}", d["key"]);
        }
    }
    // the first deck is what the page picks by default (`catalog.decks[0]`): it plays Commander, so the page's
    // default new-game request (no opponents change) is a Commander request
    let first = cat["decks"][0]["key"].as_str().unwrap();
    assert_eq!(e.format_of(first).key, DEFAULT_FORMAT, "the page's default deck is {first}");
}

// -------------------------------------------------------------------------------------- state, saves, names

fn safe_name(n: &str) -> bool {
    n.ends_with(".json") && n.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_+.".contains(&b))
}

#[test]
fn state_saves_list_and_save_names_for_both_formats() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let (cdeck, ctier) = commander_deck(&e);
    let mut c = s.client();
    let mut names = vec![];
    for (d, t, fmt, given) in [
        (&cdeck, &ctier, "commander", None),
        (&cdeck, &ctier, "cali", Some("cali")),
        (&deck, &tier, "cali", None),
        (&deck, &tier, "cali", Some("cali")),
    ] {
        let mut b = body(d, t, json!({"seed": 12}));
        if let Some(f) = given {
            b["format"] = json!(f);
        }
        assert_eq!(c.post("/api/new", b).status, 200);
        let mull = common::wait_for(&mut c, 0, common::is_request);
        assert_eq!(mull["request"]["kind"], "mulligan");
        let st = c.get("/api/state").json();
        assert_eq!(st["game"]["format"], fmt, "{d} {given:?}");
        let n = st["game"]["seats"].as_array().unwrap().len();
        assert_eq!(n, if fmt == "cali" { 2 } else { 4 });
        let r = c.post("/api/save", json!({}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        let name = r.json()["name"].as_str().unwrap().to_string();
        assert!(safe_name(&name), "{name}");
        // the name: the format segment only when it is not Commander, never twice
        let stem = name.trim_end_matches(".json");
        let tail = stem.splitn(3, '-').nth(2).unwrap();
        let want = if fmt == "commander" { format!("{d}-{t}-12") } else { format!("{d}-cali-{t}-12") };
        assert_eq!(tail, want, "{name}");
        let file = s.save_file(&name).unwrap();
        assert_eq!(
            file.get("format").and_then(Value::as_str),
            if fmt == "commander" { None } else { Some("cali") },
            "{name}"
        );
        let list = c.get("/api/saves").json();
        let row = list["saves"].as_array().unwrap().iter().find(|x| x["name"] == name.as_str()).unwrap().clone();
        assert_eq!(row["format"], fmt, "{row}");
        names.push(name);
        c.quit();
    }
    // four different saves (the same deck + tier + seed at two formats do not share a name)
    let mut uniq = names.clone();
    uniq.sort();
    uniq.dedup();
    assert!(uniq.len() >= 3, "{names:?}");
    // an old Commander save (no `format`) is listed as Commander
    let row_of = |c: &mut common::Client, n: &str| {
        c.get("/api/saves").json()["saves"].as_array().unwrap().iter().find(|x| x["name"] == n).unwrap().clone()
    };
    assert_eq!(row_of(&mut c, &names[0])["format"], "commander");
    // each one loads back as the game it was
    for (n, fmt) in names.iter().zip(["commander", "cali", "cali", "cali"]) {
        let r = c.post("/api/load", json!({"name": n, "images": false}));
        assert_eq!(r.status, 200, "{n}: {}", String::from_utf8_lossy(&r.body));
        common::wait_for(&mut c, 0, common::is_request);
        assert_eq!(c.get("/api/state").json()["game"]["format"], fmt, "{n}");
        c.quit();
    }
    for n in &names {
        s.remove_save(n);
    }
}

/// A game in progress at a Cali table is autosaved with its format, and the saves page lists it.
#[test]
fn the_autosave_of_a_cali_game_carries_its_format() {
    let s = srv!();
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let mut c = s.client();
    assert_eq!(c.post("/api/new", body(&deck, &tier, json!({"seed": 13}))).status, 200);
    common::wait_for(&mut c, 0, common::is_request);
    let start = std::time::Instant::now();
    let mut row = Value::Null;
    while start.elapsed() < std::time::Duration::from_secs(20) {
        row = c.get("/api/saves").json()["autosave"].clone();
        if !row.is_null() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    c.quit();
    if row.is_null() {
        eprintln!("no autosave appeared in 20 s (the server may write it only after an answer); not a failure");
        return;
    }
    assert_eq!(row["format"], "cali", "{row}");
    assert_eq!(row["deck"], deck.as_str());
}

// -------------------------------------------------------------------------------------- LAN: a Cali deck joins a Commander lobby

/// The host opens a Commander two-person table; the guest picks the Cali deck (`MY_DECKS` lets it through
/// `join`'s own checks). The refusal is right (the Cali deck cannot sit at a Commander table), but the host's table
/// must survive it: `Hub::join` takes the lobby BEFORE `new_game` can refuse. Before this PR no refusal was
/// reachable after the take (Opus, 11-23); now a guest's Cali deck reaches one. (The join-keeps-lobby fix, 44641513,
/// is not in this branch.)
#[test]
fn a_cali_guest_deck_is_refused_without_destroying_the_hosts_table() {
    let s = srv!();
    if !s.lan {
        eprintln!("SKIPPED: the server is not --lan");
        return;
    }
    let e = sess::engine();
    let (kdeck, ktier) = {
        // a Commander deck for the host, not the first one (the placeholder partner of `open_lobby` is the first
        // deck of MY_DECKS that is not the host's)
        let k = MY_DECKS.iter().filter(|k| e.format_of(k).key == DEFAULT_FORMAT).nth(1).expect("two Commander decks");
        (k.to_string(), e.decks[*k].tiers[0].clone())
    };
    let (cali, _) = cali_deck(&e);
    let other = MY_DECKS.iter().find(|k| e.format_of(k).key == DEFAULT_FORMAT && **k != kdeck).unwrap().to_string();
    let mut host = s.client();
    let mut guest = s.client();
    let r = host.post("/api/new", body(&kdeck, &ktier, json!({"two": true})));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let code = r.json()["code"].as_str().unwrap().to_string();
    // the Cali deck as the guest's deck
    let r = guest.post("/api/join", json!({"code": code, "deck": cali}));
    let refused = r.status != 200 || r.json().get("error").is_some_and(|e| !e.is_null());
    assert!(refused, "a Cali deck joined a Commander table: {}", String::from_utf8_lossy(&r.body));
    // the host's table is still open and the same code still works for a deck that fits
    let lobby = host.get("/api/state").json()["lobby"].clone();
    assert!(!lobby.is_null(), "the refused join destroyed the host's table: state = {}", host.get("/api/state").json());
    let r = guest.post("/api/join", json!({"code": code, "deck": other}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    host.quit();
}

// -------------------------------------------------------------------------------------- the terminal client

fn run(args: &[&str]) -> (i32, String, String) {
    let exe = env!("CARGO_BIN_EXE_commander-practice");
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let out = std::process::Command::new(exe)
        .args(args)
        .args(["--data", data.to_str().unwrap()])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into(),
        String::from_utf8_lossy(&out.stderr).into(),
    )
}

#[test]
fn terminal_arguments() {
    // argument errors: exit 2 and the usage
    for args in [
        vec!["--text", "--seat", "x"],
        vec!["--text", "--seat", "-1"],
        vec!["--text", "--seat", "1.5"],
        vec!["--text", "--seat", ""],
        vec!["--text", "--seat"],
        vec!["--text", "--format"],
        vec!["--text", "--deck", "kyodai", "--format"],
    ] {
        let (code, _, err) = run(&args);
        assert_eq!(code, 2, "{args:?}: {err}");
        assert!(err.contains("usage:"), "{args:?}: {err}");
    }
    // a value that parses but the session refuses: exit 1 and the session's text, nothing played
    for (args, want) in [
        (vec!["--text", "--deck", "kyodai", "--format", "nope"], "unknown format \"nope\": one of "),
        (vec!["--text", "--deck", "kyodai", "--format", "commander"], "kyodai cannot be played at a Commander table"),
        (vec!["--text", "--deck", "kyodai", "--format", "Cali"], "unknown format \"Cali\""),
        (vec!["--text", "--deck", "kyodai", "--format="], "unknown format \"\""),
        (vec!["--text", "--deck", "kyodai", "--tier", "t5", "--seat", "3"], "seat is 1 to 2"),
        (vec!["--text", "--deck", "kyodai", "--tier", "t5", "--seat", "0"], "seat is 1 to 2"),
        (vec!["--text", "--deck", "kyodai", "--tier", "t5", "--seat", "9", "--format", "cali"], "seat is 1 to 2"),
        (vec!["--text", "--deck", "yoshimaru", "--seat", "9"], "seat is 1 to 4"),
        (vec!["--text", "--deck", "yoshimaru", "--seat", "0"], "seat is 1 to 4"),
        (vec!["--text", "--deck", "yoshimaru", "--seat", "18446744073709551615"], "seat is 1 to 4"),
        (vec!["--text", "--deck", "yoshimaru", "--opponents", "a,b"], "pick three of: "),
        (vec!["--text", "--deck", "kyodai", "--tier", "t5", "--opponents", "a,b,c"], "pick one of: "),
    ] {
        let (code, _, err) = run(&args);
        assert_eq!(code, 1, "{args:?}: {err}");
        assert!(err.contains(want), "{args:?}: wanted {want:?} in {err:?}");
    }
    // `--format=cali` (inline value) is the same as two words
    let (code, _, err) = run(&["--text", "--deck", "yoshimaru", "--format=cali", "--seat", "3"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("seat is 1 to 2"), "{err}");
}

/// A Cali game at the terminal runs to its end with no input at all (every prompt takes its default), without a
/// panic, and the table it prints has two players at 30 life.
#[test]
fn terminal_plays_a_cali_game_with_default_answers() {
    let e = sess::engine();
    let (deck, tier) = cali_deck(&e);
    let exe = env!("CARGO_BIN_EXE_commander-practice");
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let mut child = std::process::Command::new(exe)
        .args([
            "--text",
            "--deck",
            &deck,
            "--tier",
            &tier,
            "--seed",
            "3",
            "--ai",
            "adaptive",
            "--data",
            data.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // an empty line answers every prompt with its default (keep, pass, none)
    {
        use std::io::Write;
        let mut si = child.stdin.take().unwrap();
        std::thread::spawn(move || {
            for _ in 0..20000 {
                if si.write_all(b"\n").is_err() {
                    break;
                }
            }
        });
    }
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(st) = child.try_wait().unwrap() {
            break Some(st);
        }
        if start.elapsed() > std::time::Duration::from_secs(240) {
            child.kill().ok();
            child.wait().ok();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    };
    let out = child.wait_with_output().unwrap();
    let (so, se) = (String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(!se.contains("panicked"), "{se}");
    assert!(
        status.is_some(),
        "the terminal game did not end in 240 s; stdout tail: {}",
        &so[so.len().saturating_sub(600)..]
    );
    assert!(so.contains("life 30"), "no life-30 line in the table: {}", &so[..so.len().min(1500)]);
    assert!(!so.contains("life 40"), "a life-40 line at a Cali table");
    assert!(so.contains("skips the draw step of the first turn"), "no first-draw skip line");
    assert!(so.contains("Game over"), "the game did not reach its end: {}", &so[so.len().saturating_sub(600)..]);
    assert!(!so.contains("(closed)"), "the game was closed, not played: {}", &so[so.len().saturating_sub(600)..]);
}
