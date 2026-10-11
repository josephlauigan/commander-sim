//! Sessions: settings, running a game, Undo, saved games, replay determinism, closing. (Python's test_play_undo,
//! test_play_save and the session half of test_play_hint / test_play_review; the Hint and the comparison need the
//! person's main phase and are in session_hint.rs.)

mod common;
use common::*;
use practice::session::Session;
use serde_json::{Value, json};
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(120);

/// the final table, without the bits that legitimately differ between runs
fn final_state(evs: &[Value]) -> Value {
    let o = over(evs);
    json!({"winner": o["winner"], "how": o["how"], "view": o["view"]})
}

#[test]
fn settings_are_checked() {
    let e = engine();
    let mut o = opts("adaptive");
    o.deck = "nobody".into();
    assert!(Session::new(o).err().unwrap().starts_with("unknown deck \"nobody\": one of seph"));
    let mut o = opts("adaptive");
    o.tier = "t9".into();
    assert!(Session::new(o).err().unwrap().starts_with("unknown tier \"t9\""));
    let mut o = opts("adaptive");
    o.partner = Some("sauron".into());
    assert_eq!(Session::new(o).err().unwrap(), "the two players need different decks");
    let mut o = opts("adaptive");
    o.seat = Some(5);
    assert_eq!(Session::new(o).err().unwrap(), "seat is 1 to 4");
    let mut o = opts("adaptive");
    o.seats = Some(vec!["sauron".into(), "x".into(), "y".into(), "z".into()]);
    assert!(Session::new(o).err().unwrap().starts_with("seats: your deck and 3 of"));
    let mut o = opts("adaptive");
    o.seat = Some(2);
    let s = Session::new(o).unwrap();
    assert_eq!(s.seats[1], "sauron");
    assert_eq!(s.seats.len(), 4);
    drop(e);
}

#[test]
fn a_game_runs_to_its_end_and_the_same_seed_seats_the_same_table() {
    let a = Session::new(opts("adaptive")).unwrap();
    let b = Session::new(opts("adaptive")).unwrap();
    assert_eq!(a.seats, b.seats);
    a.start();
    let (_, evs) = drive(&a, 0, LIMIT, &|_, _| None);
    let k = kinds(&evs);
    assert!(k.contains(&"request".to_string()) && k.contains(&"log".to_string()) && k.contains(&"turn".to_string()));
    assert_eq!(evs[0]["kind"], "log");
    assert!(evs[0]["text"].as_str().unwrap().starts_with("Seat order: "));
    let first_req = evs.iter().find(|e| e["kind"] == "request").unwrap();
    assert_eq!(first_req["request"]["kind"], "mulligan");
    assert_eq!(first_req["seat"], "sauron");
    assert_eq!(first_req["request"]["data"]["names"].as_array().unwrap().len(), 7);
    let o = over(&evs);
    assert!(o["view"]["players"].as_array().unwrap().len() == 4);
    // the view hides the others' hands
    for p in o["view"]["players"].as_array().unwrap() {
        assert_eq!(p.get("hand").is_some(), p["you"] == true);
    }
    // the same game again gives the same table
    b.start();
    let (_, evs2) = drive(&b, 0, LIMIT, &|_, _| None);
    assert_eq!(final_state(&evs), final_state(&evs2));
    a.close(Duration::from_secs(10));
    b.close(Duration::from_secs(10));
}

#[test]
fn undo_replays_to_the_same_game() {
    let s = Session::new(opts("adaptive")).unwrap();
    s.start();
    let (c, evs) = drive(&s, 0, LIMIT, &|_, _| None);
    let want = final_state(&evs);
    let n_answers = s.answers_len();
    assert!(n_answers >= 1);
    assert_eq!(s.undo(0, "sauron"), Some("Nothing to undo yet.".into()));
    // Undo the only answer: back at the mulligan, then keep again
    assert_eq!(s.undo(1, "sauron"), None);
    let (c2, evs2) = drive(&s, c, LIMIT, &|_, _| None);
    assert_eq!(evs2[0]["kind"], "reset");
    assert_eq!(evs2[0]["undone"], 1);
    assert_eq!(evs2[0]["by"], "sauron");
    assert_eq!(final_state(&evs2), want);
    // a different answer: mulligan once (then keep): a different game
    assert_eq!(s.undo(1, "sauron"), None);
    let (_, evs3) = drive(&s, c2, LIMIT, &|_, i| if i == 1 { Some(json!(1)) } else { None });
    let hand = |e: &Value| e["view"]["players"].as_array().unwrap().iter().find(|p| p["you"] == true).unwrap()["hand"].clone();
    // (until the person's own turns are in the engine the game itself doesn't depend on the hand)
    assert!(hand(over(&evs3)) != hand(over(&evs)), "a different opening hand");
    s.close(Duration::from_secs(10));
}

#[test]
fn a_saved_game_loads_and_replays_to_where_it_was_saved() {
    let s = Session::new(opts("adaptive")).unwrap();
    s.start();
    let (_, evs) = drive(&s, 0, LIMIT, &|_, _| None);
    let want = final_state(&evs);
    let saved = s.saved();
    assert_eq!(saved["version"], 1);
    assert_eq!(saved["seed"], 4242);
    assert_eq!(saved["finished"], true);
    // through JSON text, as a file
    let text = serde_json::to_string(&saved).unwrap();
    let data: Value = serde_json::from_str(&text).unwrap();
    for _ in 0..2 {
        // loaded, and loaded again from the loaded game's own save
        let l = Session::load(engine(), &data, true, true).unwrap();
        l.start();
        let (_, evs) = drive(&l, 0, LIMIT, &|_, _| panic!("a loaded game asks nothing it has the answer to"));
        assert_eq!(final_state(&evs), want);
        assert!(evs.iter().filter(|e| e["kind"] == "log").all(|e| e["replay"] == true), "replayed history is marked");
        assert_eq!(l.saved()["answers"], saved["answers"]);
        l.close(Duration::from_secs(10));
    }
    let mut bad = data.clone();
    bad["version"] = json!(99);
    assert_eq!(Session::load(engine(), &bad, true, true).err().unwrap(), "a saved game from a different version");
    s.close(Duration::from_secs(10));
}

#[test]
fn the_look_ahead_tape_replays_a_game_without_searching_again() {
    let mut o = opts("lookahead");
    o.max_rounds = 4;
    let s = Session::new(o).unwrap();
    s.start();
    let (c, evs) = drive(&s, 0, Duration::from_secs(600), &|_, _| None);
    let want = final_state(&evs);
    let n = s.tape.lock().unwrap().entries.len();
    assert!(n > 0, "the look-ahead made decisions");
    let t0 = std::time::Instant::now();
    assert_eq!(s.undo(1, "sauron"), None);
    // the tape is cut at the mulligan (the AI's decisions came after it): the second run searches again and
    // records the same entries
    let (_, evs2) = drive(&s, c, Duration::from_secs(600), &|_, _| None);
    assert_eq!(final_state(&evs2), want);
    assert_eq!(s.tape.lock().unwrap().entries.len(), n);
    let _ = t0;
    // saved with its tape; loading replays it from the tape
    let data = s.saved();
    assert_eq!(data["tape"].as_array().unwrap().len(), n);
    let l = Session::load(engine(), &data, true, true).unwrap();
    let t1 = std::time::Instant::now();
    l.start();
    let (_, evs3) = drive(&l, 0, Duration::from_secs(600), &|_, _| panic!("no questions"));
    assert_eq!(final_state(&evs3), want);
    assert_eq!(l.tape.lock().unwrap().entries.len(), n, "nothing new was searched");
    eprintln!("replay from the tape took {:?}", t1.elapsed());
    s.close(Duration::from_secs(10));
    l.close(Duration::from_secs(10));
}

#[test]
fn closing_stops_every_thread() {
    let s = Session::new(opts("adaptive")).unwrap();
    s.start();
    // wait for the first decision
    let (_, evs) = s.events_after(0, Duration::from_secs(30));
    let mut got = evs;
    while !got.iter().any(|e| e["kind"] == "request") {
        let (_, more) = s.events_after(got.len() as u64, Duration::from_secs(30));
        got.extend(more);
    }
    assert!(s.threads_running());
    s.close(Duration::from_secs(10));
    assert!(!s.threads_running());
    let (_, rest) = s.events_after(got.len() as u64, Duration::from_millis(500));
    assert_eq!(rest.last().unwrap()["how"], "closed");
}

#[test]
fn hint_and_review_say_why_not() {
    let s = Session::new(opts("adaptive")).unwrap();
    assert_eq!(s.hint("sauron").unwrap_err(), "There is no decision waiting for you.");
    assert_eq!(s.review("sauron").unwrap_err(), "The review opens when the game ends.");
    s.start();
    let (_, evs) = drive(&s, 0, LIMIT, &|_, _| None);
    let _ = evs;
    let r = s.review("sauron").unwrap();
    assert_eq!(r["summary"]["answers"].as_u64().unwrap() as usize, s.answers_len());
    assert!(r["decisions"].is_array());
    assert_eq!(s.review("nobody").unwrap_err(), "You have no seat in this game.");
    s.close(Duration::from_secs(10));
}

#[test]
fn auto_pass_changes_apply_from_the_next_answer() {
    let s = Session::new(opts("adaptive")).unwrap();
    assert_eq!(s.autopass_at("sauron", None), "respond");
    assert_eq!(s.set_autopass("sauron", "bogus").unwrap(), "auto-pass: one of respond, stack, all");
    assert_eq!(s.set_autopass("nobody", "all").unwrap(), "You have no seat in this game.");
    assert_eq!(s.set_autopass("sauron", "all"), None);
    assert_eq!(s.autopass_at("sauron", Some(0)), "respond");
    assert_eq!(s.autopass_at("sauron", Some(1)), "all");
}
