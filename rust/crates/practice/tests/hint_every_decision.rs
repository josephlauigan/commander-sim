//! A hint at every decision: several seeded games with the person's seat, a hint asked at each request (mulligans,
//! bottoming, priority, attacks, blocks, choices ...), each hint with a real `text`, a `detail` array and a
//! `choice`. Also the mulligan hint itself and each kind of request, built by hand.

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::advisor;
use practice::session::Session;
use serde_json::{Value, json};
use sim_core::control::Request;
use sim_core::ids::PlayerId;
use sim_core::run::settings;
use sim_core::state::Game;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

fn check(h: &Value, what: &str) {
    let t = h["text"].as_str().unwrap_or_else(|| panic!("{what}: text is not a string: {h}"));
    assert!(!t.trim().is_empty() && t != "null" && !t.contains("No hint"), "{what}: bad text {t:?}");
    assert!(h["detail"].is_array(), "{what}: detail is not an array: {h}");
    assert!(h.get("choice").is_some(), "{what}: no choice key: {h}");
}

#[test]
fn every_decision_of_several_games_has_a_hint() {
    let seen = RefCell::new(BTreeMap::<String, usize>::new());
    let mut mulligan_texts = vec![];
    for (i, (deck, seed)) in
        [("sauron", 4242u64), ("ghave", 7), ("cloud", 11), ("kefka", 3), ("yshtola", 5)].into_iter().enumerate()
    {
        let mut o = opts("adaptive");
        o.deck = deck.into();
        o.seed = Some(seed);
        o.max_rounds = 8;
        let s = Session::new(o).unwrap();
        s.start();
        let texts = RefCell::new(vec![]);
        let policy = |req: &Value, asked: usize| -> Option<Value> {
            let kind = req["kind"].as_str().unwrap().to_string();
            let h = s.hint(deck).unwrap_or_else(|e| panic!("{deck}: hint for a {kind} request failed: {e}"));
            check(&h, &format!("{deck} {kind} {}", req["prompt"]));
            *seen.borrow_mut().entry(kind.clone()).or_default() += 1;
            if kind == "mulligan" {
                texts.borrow_mut().push(h["text"].as_str().unwrap().to_string());
                if i < 2 && asked <= 2 {
                    return Some(json!(1)); // mulligan twice: reaches the bottoming choice
                }
                return Some(json!(0));
            }
            if kind == "priority" && req["data"]["stack"].as_array().is_none_or(|s| s.is_empty()) {
                return Some(match asked % 6 {
                    0 => json!({"do": "land", "card": 0}),
                    1 => json!({"do": "land", "card": 1}),
                    2 => json!({"do": "cast", "zone": "cmd"}),
                    3 => json!({"do": "cast", "card": 0}),
                    _ => json!({"do": "pass"}),
                });
            }
            if kind == "attack" {
                return Some(json!([0]));
            }
            None
        };
        drive(&s, 0, Duration::from_secs(240), &policy);
        mulligan_texts.extend(texts.into_inner());
    }
    let seen = seen.into_inner();
    for k in ["mulligan", "priority", "choose"] {
        assert!(seen.contains_key(k), "no {k} request seen: {seen:?}");
    }
    assert!(mulligan_texts.iter().all(|t| t.starts_with("Keep") || t.starts_with("Mulligan")), "{mulligan_texts:?}");
    eprintln!("kinds hinted: {seen:?}");
}

fn game() -> (Game, PlayerId) {
    let e = engine();
    let seats: Vec<String> = ["sauron", "ghave", "veyran", "galadriel"].iter().map(|s| s.to_string()).collect();
    let (g, _) =
        practice::session::setup::build_game(&e, settings("loose", "adaptive", 1.0).unwrap(), 77, &seats, &[]).unwrap();
    (g, PlayerId(0))
}

fn req(kind: &str, prompt: &str, choices: &[&str]) -> Request {
    Request {
        kind: sim_core::sym::intern(kind),
        prompt: prompt.into(),
        choices: choices.iter().map(|s| s.to_string()).collect(),
        data: json!({}),
    }
}

#[test]
fn the_mulligan_hint_explains_the_ais_rule() {
    let (mut g, p) = game();
    // draw a seven for the person's seat, as the mulligan does
    let pl = g.player_mut(p);
    let hand = std::mem::take(&mut pl.hand);
    pl.library.extend(hand);
    for _ in 0..7 {
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
    }
    let h = advisor::hint(&mut g, p, &req("mulligan", "Your opening hand (7 to keep).", &["keep", "mulligan"]));
    check(&h, "mulligan");
    let t = h["text"].as_str().unwrap();
    assert!(t.starts_with("Keep") || t.starts_with("Mulligan"), "{t}");
    let d: Vec<&str> = h["detail"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(d[0].contains("2 to 5 lands") || d[0].contains("3 to 5 lands"), "{d:?}");
    assert!(d.iter().any(|l| l.starts_with("Colours:")) && d.iter().any(|l| l.starts_with("Commander ")), "{d:?}");
    assert_eq!(h["choice"], if t.starts_with("Keep") { "keep" } else { "mulligan" });
    // with cards to bottom the hint names the AI's bottom card
    let h = advisor::hint(
        &mut g,
        p,
        &req(
            "mulligan",
            "Your opening hand (5 to keep). If you keep, you put 2 card(s) on the bottom.",
            &["keep", "mulligan"],
        ),
    );
    assert!(h["detail"].as_array().unwrap().iter().any(|l| l.as_str().unwrap().contains("on the bottom")), "{h}");
    let labels: Vec<String> = g.player(p).hand.iter().map(|&c| sim_core::human::choices::card_label(&g, c)).collect();
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    let b = advisor::hint(&mut g, p, &req("choose", "Put a card on the bottom of your library (1 of 2)", &refs));
    check(&b, "bottom");
    assert!(b["choice"].is_u64(), "{b}");
}

#[test]
fn every_kind_of_request_gets_a_sentence_never_null() {
    let (mut g, p) = game();
    let hand: Vec<String> = g.player(p).hand.iter().map(|&c| sim_core::human::choices::card_label(&g, c)).collect();
    let hr: Vec<&str> = hand.iter().map(String::as_str).collect();
    let cases: Vec<Request> = vec![
        req("mulligan", "Your opening hand (7 to keep).", &["keep", "mulligan"]),
        req("mulligan", "", &[]),
        req("attack", "Declare attackers", &[]),
        req("block", "X attacks you (4 damage in all). Block Y with?", &["A (1/1)", "no block"]),
        req("block", "", &[]),
        req("target", "Doom Blade: choose a target", &["Opponent", "none"]),
        req("target", "", &[]),
        req("search", "Search for a land", &["Swamp", "find nothing"]),
        req("search", "", &[]),
        req("choose", "End of turn: discard down to 7", &hr),
        req("choose", "Discard a card", &[]),
        req("choose", "Scry: Nobody Known {2}{B}", &["keep on top", "put on the bottom"]),
        req("choose", "Surveil: Swamp", &["keep on top", "put on your graveyard"]),
        req("choose", "Sacrifice which creature?", &["Nothing (1/1)"]),
        req("choose", "Put a card on the bottom of your library (1 of 1)", &[]),
        req("choose", "Pay with?", &["yes", "no"]),
        req("choose", "Prismari Charm: choose one", &["deal damage 2", "draw cards"]),
        req("choose", "Barad-dur: choose X (it costs)", &["1", "2"]),
        req("choose", "Something odd", &[]),
        req("continue", "Round 2", &[]),
        req("priority", "Main 1", &[]),
        req("who-knows", "Strange", &[]),
        req("who-knows", "", &[]),
    ];
    for r in cases {
        let h = advisor::hint(&mut g, p, &r);
        check(&h, &format!("{} / {}", r.kind, r.prompt));
    }
}
