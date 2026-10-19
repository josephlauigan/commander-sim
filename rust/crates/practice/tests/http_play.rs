//! Ports of the HTTP-reachable parts of tests/test_play_{undo,save,hint,review,shadow}.py: Undo restarts the game
//! from its seed and asks the same decision again with the same table; a saved game comes back decision for
//! decision; asking for a hint changes nothing; the AI comparison changes nothing and the review waits for the end.
//! Python tests drive a Session directly; these drive the same game through the browser's API with the same bot.
mod common;
use common::*;
use serde_json::{Value, json};

macro_rules! srv {
    () => {
        match server() {
            Some(s) => s,
            None => return,
        }
    };
}

fn start(c: &mut Client, deck: &str, seed: u64, ai: &str, tools: Value) {
    let r = c
        .post("/api/new", json!({"deck": deck, "tier": "t2", "seed": seed, "ai": ai, "images": false, "tools": tools}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
}

/// the events of the (restarted) game's stream up to its next request
fn until_request(c: &mut Client) -> Vec<Value> {
    c.stream(0, None).until(300, is_request)
}

/// after an Undo: the events from its reset (newer than event `since`) to the next request. The Undo answers before the
/// game restarts, so reading from the start at once can still see the old game's events
fn after_reset(c: &mut Client, since: u64) -> Vec<Value> {
    let mut st = c.stream(0, None);
    let reset = st.until(300, |e| e["kind"] == "reset" && e["id"].as_u64() > Some(since)).pop().unwrap();
    let mut evs = vec![reset];
    evs.extend(st.until(300, is_request));
    evs
}

#[test]
fn test_undo_asks_the_last_decision_again_with_the_same_table() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({}));
    let (seen, last) = play(&mut c, 40, false);
    assert_eq!(seen.len(), 41);
    let r = c.post("/api/undo", json!({"n": 3}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let evs = after_reset(&mut c, last);
    assert_eq!(frozen(evs.last().unwrap()), seen[40 - 3]);
    c.quit();
}

#[test]
fn test_replayed_history_is_marked() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({}));
    let (_, last) = play(&mut c, 15, false);
    assert_eq!(c.post("/api/undo", json!({"n": 1})).status, 200);
    // card images arrive from a background thread whenever they are fetched: not part of the replay
    let evs: Vec<Value> = after_reset(&mut c, last).into_iter().filter(|e| e["kind"] != "images").collect();
    assert_eq!(evs[0]["kind"], "reset");
    let (mid, last) = (&evs[1..evs.len() - 1], evs.last().unwrap());
    assert!(mid.iter().filter(|e| e["kind"] == "log").all(|e| e["replay"] == true));
    assert!(mid.iter().any(|e| e["kind"] == "log"));
    assert!(last.get("replay").is_none());
    assert!(evs.windows(2).all(|w| w[0]["id"].as_u64() < w[1]["id"].as_u64()));
    c.quit();
}

#[test]
fn test_nothing_to_undo() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({}));
    wait_for(&mut c, 0, is_request);
    let r = c.post("/api/undo", json!({}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "Nothing to undo yet.");
    c.quit();
}

#[test]
fn test_undo_switched_off() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({"undo": false}));
    let mull = wait_for(&mut c, 0, is_request);
    c.answer(mull["id"].as_u64().unwrap(), json!("keep"));
    let r = c.post("/api/undo", json!({}));
    assert_eq!(r.status, 409);
    assert_eq!(r.error(), "Undo is switched off for this game.");
    c.quit();
}

#[test]
fn test_a_saved_game_comes_back_the_same() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "alela", 500000, "lookahead", json!({"compare": true}));
    let p = play_with(&mut c, 50, false);
    let here = p.seen.last().unwrap().clone();
    let r = c.post("/api/save", json!({}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let name = r.json()["name"].as_str().unwrap().to_string();
    let data = s.save_file(&name).expect("the save file");
    assert_eq!(data["answers"].as_array().unwrap().len(), 50);
    assert_eq!(data["seed"], 500000);
    assert!(data["version"].is_number());
    let r = c.post("/api/load", json!({"name": name, "images": false}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let again = until_request(&mut c);
    assert_eq!(frozen(again.last().unwrap()), here);
    // saved again after the load, it is the same game: the same answers
    let name2 = c.post("/api/save", json!({})).json()["name"].as_str().unwrap().to_string();
    let data2 = s.save_file(&name2).unwrap();
    assert_eq!(data2["answers"], data["answers"]);
    c.quit();
    s.remove_save(&name);
    s.remove_save(&name2);
}

#[test]
fn test_hints_change_nothing() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "lookahead", json!({"compare": false}));
    let with = play_with(&mut c, 60, true);
    start(&mut c, "sauron", 11, "lookahead", json!({"compare": false}));
    let without = play_with(&mut c, 60, false);
    assert_eq!(with.seen, without.seen);
    let main: Vec<&Value> = with
        .hints
        .iter()
        .filter(|(k, step, _)| k == "priority" && (step == "main1" || step == "main2"))
        .map(|h| &h.2)
        .collect();
    assert!(!main.is_empty());
    assert!(main.iter().all(|h| h["text"].as_str().is_some_and(|t| !t.is_empty())));
    assert!(main.iter().any(|h| h["detail"][0].as_str().is_some_and(|d| d.starts_with("Look-ahead"))));
    for (_, _, h) in &with.hints {
        assert!(h.get("text").is_some() && h.get("detail").is_some() && h.get("choice").is_some());
    }
    c.quit();
}

#[test]
fn test_comparisons_change_nothing_and_the_review_waits() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "lookahead", json!({"compare": true}));
    let with = play_with(&mut c, 70, false);
    let r = c.get("/api/review");
    assert_eq!(r.status, 409); // hidden until the game ends
    assert!(r.error().contains("when the game ends"), "{}", r.error());
    start(&mut c, "sauron", 11, "lookahead", json!({"compare": false}));
    let without = play_with(&mut c, 70, false);
    assert_eq!(with.seen, without.seen); // the game is the same with the comparison on
    c.quit();
}

#[test]
fn test_review_and_try_it() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({"compare": true}));
    let (end, n) = play_out(&mut c, 900);
    assert_eq!(end["kind"], "over", "{end}");
    assert!(n > 0);
    let r = c.get("/api/review");
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let rv = r.json();
    let sm = &rv["summary"];
    assert!(sm["decisions"].as_u64().unwrap() > 0);
    assert!(sm["matched"].as_u64() <= sm["compared"].as_u64());
    let decs = rv["decisions"].as_array().unwrap();
    for d in decs {
        for k in ["n", "can_try", "differs", "ai"] {
            assert!(d.get(k).is_some(), "decision lacks {k}");
        }
    }
    if let Some(d) = decs
        .iter()
        .find(|d| d["can_try"] == true && d["differs"] == true && !d["ai"].as_str().unwrap_or("").starts_with("stop"))
    {
        let r = c.post("/api/tryit", json!({"n": d["n"]}));
        assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
        // the try-it game's reset (the old game's events can still be there for a moment), then on to a request
        let mut st = c.stream(0, None);
        let mut evs = st.until(300, |e| e["kind"] == "reset" && e.get("tryit").is_some()).split_off(0);
        evs.drain(..evs.len() - 1);
        evs.extend(st.until(300, |e| matches!(e["kind"].as_str(), Some("request" | "over" | "error"))));
        assert_eq!(evs[0]["kind"], "reset");
        assert_eq!(evs[0]["tryit"]["n"], d["n"]);
        assert!(evs[0]["tryit"].get("applied").is_some());
    }
    c.quit();
}

#[test]
fn test_try_it_needs_a_compared_decision() {
    let s = srv!();
    let mut c = s.client();
    start(&mut c, "sauron", 11, "adaptive", json!({"compare": true}));
    wait_for(&mut c, 0, is_request);
    let r = c.post("/api/tryit", json!({"n": 5}));
    assert_eq!(r.status, 409);
    assert!(r.error().contains("no AI choice"), "{}", r.error());
    c.quit();
}
