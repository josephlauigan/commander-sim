//! Port of tests/test_play_server.py (and the HTTP-reachable parts of test_play_{hint,review,save}.py): the page and
//! its files, the options, starting a game, the event stream, answering (stale answers refused), hidden
//! information, saving and the autosave. Runs against the server named by PRACTICE_URL; run with --test-threads=1.
mod common;
use common::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

macro_rules! srv {
    () => {
        match server() {
            Some(s) => s,
            None => return,
        }
    };
}

fn new_sauron(c: &mut Client, seed: u64) -> Value {
    let r = c.new_game("sauron", "t2", seed);
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    r.json()
}

#[test]
fn test_the_page_and_its_files() {
    let s = srv!();
    let mut c = s.client();
    let r = c.get("/");
    assert_eq!(r.status, 200);
    assert!(r.header("content-type").unwrap().contains("text/html"));
    assert!(String::from_utf8_lossy(&r.body).contains("app.js"));
    let r = c.get("/static/app.js");
    assert_eq!(r.status, 200);
    assert!(r.header("content-type").unwrap().contains("javascript"));
    assert_eq!(c.get("/static/../server.py").status, 404);
    assert_eq!(c.get("/static/%2e%2e/server.py").status, 404);
    assert_eq!(c.get("/nothing-here").status, 404);
}

#[test]
fn test_the_page_sets_a_seat_cookie() {
    let s = srv!();
    let mut c = s.client();
    c.get("/");
    assert!(c.cookie.as_deref().is_some_and(|k| k.starts_with("practice_seat=")), "{:?}", c.cookie);
}

#[test]
fn test_hint() {
    let s = srv!();
    let mut c = s.client();
    new_sauron(&mut c, 4);
    wait_for(&mut c, 0, is_request);
    let r = c.post("/api/hint", json!({}));
    assert_eq!(r.status, 200);
    assert!(r.json().get("text").is_some()); // mulligan: no hint for it yet
    c.post(
        "/api/new",
        json!({"deck": "sauron", "tier": "t2", "seed": 4, "ai": "adaptive", "images": false, "tools": {"hint": false}}),
    );
    wait_for(&mut c, 0, is_request);
    let r = c.post("/api/hint", json!({}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("switched off"), "{}", r.error());
    c.quit();
}

#[test]
fn test_no_decision_waiting_for_a_hint() {
    // Python's test_hint.test_no_decision_waiting (a session with nothing waiting) is, over HTTP, no game at all
    let s = srv!();
    let mut c = s.client();
    c.quit();
    let r = c.post("/api/hint", json!({}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "There is no game running.");
}

#[test]
fn test_review_waits_for_the_end() {
    let s = srv!();
    let mut c = s.client();
    new_sauron(&mut c, 4);
    wait_for(&mut c, 0, is_request);
    let r = c.get("/api/review");
    assert_eq!(r.status, 409);
    assert!(r.error().contains("when the game ends"), "{}", r.error());
    c.quit();
}

#[test]
fn test_save_list_and_load() {
    let s = srv!();
    let mut c = s.client();
    new_sauron(&mut c, 4);
    let mull = wait_for(&mut c, 0, is_request);
    c.answer(mull["id"].as_u64().unwrap(), json!("keep"));
    let nxt = wait_for(&mut c, mull["id"].as_u64().unwrap(), is_request);
    let r = c.post("/api/save", json!({}));
    assert_eq!(r.status, 200);
    let name = r.json()["name"].as_str().unwrap().to_string();
    let saves = c.get("/api/saves").json();
    let mine: Vec<&Value> = saves["saves"].as_array().unwrap().iter().filter(|x| x["name"] == name).collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0]["seed"], 4);
    assert_eq!(c.post("/api/load", json!({"name": "../x.json"})).status, 409);
    assert_eq!(c.post("/api/load", json!({"name": name, "images": false})).status, 200);
    let again = wait_for(&mut c, 0, is_request);
    assert_eq!(again["request"]["prompt"], nxt["request"]["prompt"]); // back where it was saved
    c.quit();
    s.remove_save(&name);
}

#[test]
fn test_save_and_load_errors() {
    let s = srv!();
    let mut c = s.client();
    c.quit();
    let r = c.post("/api/save", json!({}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "There is no game to save.");
    let r = c.post("/api/load", json!({"name": "no-such-save.json"}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "No such saved game.");
    // a save of another version (Python: test_play_save.test_wrong_version) is refused, and says so
    std::fs::create_dir_all(&s.saves).unwrap();
    let bad = "zz-test-bad-version.json";
    std::fs::write(s.saves.join(bad), r#"{"version": 0}"#).unwrap();
    let r = c.post("/api/load", json!({"name": bad, "images": false}));
    s.remove_save(bad);
    assert_eq!(r.status, 409);
    assert!(r.error().starts_with("That saved game can't be loaded"), "{}", r.error());
}

#[test]
fn test_the_game_saves_itself_at_each_of_your_decisions() {
    let s = srv!();
    let mut c = s.client();
    let autosaved = |answers: usize| -> Value {
        for _ in 0..100 {
            // the pump writes it just after the decision's event
            if let Some(d) = s.save_file("autosave.json") {
                if d["answers"].as_array().map(|a| a.len()) == Some(answers) {
                    return d;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("no autosave with {answers} answers");
    };
    new_sauron(&mut c, 4);
    let mull = wait_for(&mut c, 0, is_request);
    assert_eq!(autosaved(0)["seed"], 4);
    c.answer(mull["id"].as_u64().unwrap(), json!("keep"));
    let nxt = wait_for(&mut c, mull["id"].as_u64().unwrap(), is_request);
    autosaved(1);
    let listed = c.get("/api/saves").json();
    assert!(listed["saves"].as_array().unwrap().iter().all(|x| x["name"] != "autosave.json")); // not among your saved games
    assert_eq!((listed["autosave"]["deck"].as_str(), listed["autosave"]["seed"].as_u64()), (Some("sauron"), Some(4)));
    // the app closed by iOS: the game is gone, its autosave isn't (loading replaces the running game, as a restart does)
    assert!(s.save_file("autosave.json").is_some());
    assert_eq!(c.post("/api/load", json!({"name": "autosave.json", "images": false})).status, 200);
    let again = wait_for(&mut c, 0, is_request);
    assert_eq!(again["request"]["prompt"], nxt["request"]["prompt"]); // Continue: back where it was
    c.quit(); // you ended it: nothing to continue
    assert!(s.save_file("autosave.json").is_none());
    assert!(c.get("/api/saves").json()["autosave"].is_null());
    assert_eq!(c.get("/api/options").json()["app"], false);
}

#[test]
fn test_card_images_are_served() {
    let s = srv!();
    let mut c = s.client();
    std::fs::create_dir_all(&s.images).unwrap();
    let f = s.images.join("zz-test-abc.jpg");
    std::fs::write(&f, b"jpg").unwrap();
    let r = c.get("/images/zz-test-abc.jpg");
    let _ = std::fs::remove_file(&f);
    assert_eq!((r.status, r.header("content-type"), r.body.as_slice()), (200, Some("image/jpeg"), &b"jpg"[..]));
    assert_eq!(c.get("/images/../index.json").status, 404);
    assert_eq!(c.get("/images/%2e%2e/index.json").status, 404);
}

#[test]
fn test_the_loading_screen_comes_first() {
    let s = srv!();
    let mut c = s.client();
    c.post("/api/new", json!({"deck": "veyran", "tier": "t1", "seed": 3, "ai": "adaptive", "images": false}));
    let seen = c.stream(0, None).until(120, |e| matches!(e["kind"].as_str(), Some("images" | "request" | "log")));
    assert_eq!(seen.last().unwrap()["kind"], "images");
    c.quit();
}

#[test]
fn test_catching_up_sends_only_the_last_table() {
    // Python's server.catch_up unit test, through the stream: a page that connects late gets many events but only the
    // last table (one event with a "view")
    let s = srv!();
    let mut c = s.client();
    new_sauron(&mut c, 4);
    let (_, last) = play(&mut c, 6, false);
    let mut late = c.stream(0, None);
    let evs = late.until(60, |e| e["id"].as_u64() == Some(last));
    assert!(evs.len() > 20, "only {} events", evs.len());
    assert_eq!(evs.iter().filter(|e| e.get("view").is_some()).count(), 1);
    assert!(evs.windows(2).all(|w| w[0]["id"].as_u64() < w[1]["id"].as_u64())); // increasing ids
    // a reconnect with Last-Event-ID resumes after it
    let mut re = c.stream(0, Some(last));
    assert!(re.next(Duration::from_millis(1500)).is_none_or(|e| e["id"].as_u64() > Some(last)));
    c.quit();
}

#[test]
fn test_options() {
    let s = srv!();
    let mut c = s.client();
    let r = c.get("/api/options");
    assert_eq!(r.status, 200);
    let d = r.json();
    let mut decks: Vec<&str> = d["decks"].as_array().unwrap().iter().map(|x| x["key"].as_str().unwrap()).collect();
    decks.sort();
    assert_eq!(decks, ["alela", "cloud", "galadriel", "ghave", "karn", "kefka", "sauron", "sheoldred", "thanos", "thrasios", "urdragon", "veyran", "yshtola"]);
    let tiers: Vec<&str> = d["tiers"].as_array().unwrap().iter().map(|x| x["key"].as_str().unwrap()).collect();
    assert_eq!(tiers, ["t0", "t1", "t2", "t3", "t4", "t5", "t6"]);
    assert_eq!(d["tiers"][2]["decks"].as_array().unwrap().len(), 5);
    assert!(d.get("images").is_some());
    for k in ["lan", "host", "app"] {
        assert!(d[k].is_boolean(), "{k}");
    }
}

#[test]
fn test_bad_new_game() {
    let s = srv!();
    let mut c = s.client();
    let r = c.post("/api/new", json!({"deck": "nobody", "tier": "t1"}));
    assert_eq!(r.status, 400);
    assert!(r.error().starts_with("deck: one of"), "{}", r.error());
    let r = c.post("/api/new", json!({"deck": "ghave", "tier": "t9"}));
    assert_eq!((r.status, r.error().starts_with("tier: one of")), (400, true));
    let r = c.post("/api/new", json!({"deck": "ghave", "tier": "t1", "opponents": ["isshin"], "images": false}));
    assert_eq!(r.status, 400);
    assert!(r.error().contains("pick three"), "{}", r.error());
    assert_eq!(c.post("/api/new", json!({"deck": "ghave", "tier": "t1", "seat": "first"})).status, 400);
    assert_eq!(c.post("/api/new", json!({"deck": "ghave", "tier": "t1", "opponents": "isshin"})).status, 400);
    let r = c.post("/api/new", json!({"deck": "ghave", "tier": "t1", "seats": [1, 2]}));
    assert_eq!((r.status, r.error()), (400, "seats: four deck keys in turn order".to_string()));
}

#[test]
fn test_a_request_that_is_not_json() {
    let s = srv!();
    let mut st = std::net::TcpStream::connect(&s.addr).unwrap();
    use std::io::{Read, Write};
    let body = "not json";
    write!(
        st,
        "POST /api/answer HTTP/1.1\r\nHost: x\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut out = String::new();
    st.read_to_string(&mut out).unwrap();
    assert!(out.starts_with("HTTP/1.1 400"), "{out}");
    assert!(out.contains("the request body is not JSON"), "{out}");
}

#[test]
fn test_picked_opponents_seat_and_tools() {
    let s = srv!();
    let mut c = s.client();
    let o = c.get("/api/options").json();
    let t1: Vec<String> = o["tiers"][1]["decks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["key"].as_str().unwrap().to_string())
        .take(3)
        .collect();
    let r = c.post(
        "/api/new",
        json!({"deck": "alela", "tier": "t1", "opponents": t1, "seat": 2, "ai": "adaptive", "images": false,
                                      "tools": {"hint": false}}),
    );
    assert_eq!(r.status, 200);
    let seats: Vec<String> =
        r.json()["seats"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    assert_eq!(seats[1], "alela");
    let mut others: Vec<String> = seats[..1].iter().chain(&seats[2..]).cloned().collect();
    others.sort();
    let mut want = t1.clone();
    want.sort();
    assert_eq!(others, want);
    let game = c.state()["game"].clone();
    assert_eq!(game["tools"], json!({"hint": false, "undo": true, "compare": true}));
    c.quit();
}

#[test]
fn test_auto_pass() {
    let s = srv!();
    let mut c = s.client();
    assert_eq!(c.post("/api/new", json!({"deck": "yshtola", "tier": "t2", "autopass": "never"})).status, 400);
    c.post(
        "/api/new",
        json!({"deck": "yshtola", "tier": "t2", "seed": 3, "ai": "adaptive", "images": false, "autopass": "all"}),
    );
    wait_for(&mut c, 0, is_request);
    assert_eq!(c.state()["game"]["autopass"], "all");
    assert_eq!(c.post("/api/autopass", json!({"mode": "stack"})).status, 200);
    assert_eq!(c.state()["game"]["autopass"], "stack");
    assert_eq!(c.post("/api/autopass", json!({"mode": "sometimes"})).status, 409);
    c.quit();
}

#[test]
fn test_a_game_from_the_browser() {
    let s = srv!();
    let mut c = s.client();
    let r = new_sauron(&mut c, 7);
    assert!(r["seats"].as_array().unwrap().iter().any(|x| x == "sauron"));
    assert_eq!(r["seed"], 7);
    let mull = wait_for(&mut c, 0, is_request);
    assert_eq!(mull["request"]["kind"], "mulligan");
    let id = mull["id"].as_u64().unwrap();
    assert_eq!(c.answer(id + 1000, json!("keep")).status, 409); // not the waiting decision
    assert_eq!(c.answer(id, json!("keep")).status, 200);
    assert_eq!(c.answer(id, json!("keep")).status, 409); // answered
    let nxt = wait_for(&mut c, id, is_request);
    assert!(["priority", "choose", "target", "attack"].contains(&nxt["request"]["kind"].as_str().unwrap()));
    assert!(nxt["id"].as_u64() > Some(id));
    let state = c.state();
    assert_eq!(state["pending"]["id"], nxt["id"]);
    let view = &state["view"];
    let players = view["players"].as_array().unwrap();
    let me = players.iter().find(|p| p["you"] == true).unwrap();
    assert_eq!(me["key"], "sauron");
    assert!(me.get("hand").is_some());
    assert!(players.iter().filter(|p| p["you"] != true).all(|p| p.get("hand").is_none())); // their hands are counts only
    for k in ["game", "busy", "host", "lan", "lobby", "view", "pending", "images", "last_event", "proposal"] {
        assert!(state.get(k).is_some(), "state has {k}");
    }
    assert_eq!(c.quit().status, 200);
    assert!(c.state()["game"].is_null());
}

#[test]
fn test_events_have_increasing_ids_and_known_kinds() {
    let s = srv!();
    let mut c = s.client();
    new_sauron(&mut c, 7);
    let mut st = c.stream(0, None);
    assert_eq!(st.status, 200);
    assert!(st.content_type.contains("text/event-stream"), "{}", st.content_type);
    let evs = st.until(120, is_request);
    assert!(evs.windows(2).all(|w| w[0]["id"].as_u64() < w[1]["id"].as_u64()));
    assert!(evs.iter().all(|e| e["id"].is_u64() && e["kind"].is_string()));
    let req = evs.last().unwrap();
    for k in ["kind", "prompt", "choices", "data"] {
        assert!(req["request"].get(k).is_some(), "request has {k}");
    }
    // the stream from the middle resumes after `since`
    let first = evs[0]["id"].as_u64().unwrap();
    let mut later = c.stream(first, None);
    let e = later.next(Duration::from_secs(60)).unwrap();
    assert!(e["id"].as_u64().unwrap() > first);
    c.quit();
}

#[test]
fn test_unknown_routes_and_bodies() {
    let s = srv!();
    let mut c = s.client();
    assert_eq!(c.get("/api/nothing").status, 404);
    assert_eq!(c.post("/api/nothing", json!({})).status, 404);
    c.quit();
    let r = c.answer(1, json!("keep"));
    assert_eq!((r.status, r.error()), (409, "There is no game running.".to_string()));
    let r = c.post("/api/undo", json!({}));
    assert_eq!(r.status, 409);
    assert!(!r.error().is_empty());
    let t = Instant::now();
    assert!(c.get("/api/state").json()["game"].is_null());
    assert!(t.elapsed() < Duration::from_secs(5));
}
