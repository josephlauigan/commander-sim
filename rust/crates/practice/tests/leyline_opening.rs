//! A person who keeps an opening hand with Leyline of the Guildpact in it is asked, right after the mulligan, whether to
//! begin the game with it on the battlefield ("If this card is in your opening hand, you may begin the game with it on the
//! battlefield"). The Thanos deck plays it.

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::session::Session;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

/// the next request event of the session, or None if it ends first
fn next_request(s: &Arc<Session>, cursor: &mut u64) -> Option<Value> {
    let end = std::time::Instant::now() + Duration::from_secs(60);
    while std::time::Instant::now() < end {
        let (c, evs) = s.events_after(*cursor, Duration::from_millis(200));
        *cursor = c;
        for ev in evs {
            match ev["kind"].as_str().unwrap_or("") {
                "request" => return Some(ev),
                "over" | "error" => return None,
                _ => {}
            }
        }
    }
    panic!("no request in time");
}

#[test]
fn a_kept_hand_with_leyline_is_asked_whether_to_begin_with_it() {
    for seed in 1..400u64 {
        let mut o = opts("adaptive");
        o.deck = "thanos".into();
        o.tier = "t5".into();
        o.seed = Some(seed);
        o.compare = false;
        o.max_rounds = 1;
        let s = Session::new(o).unwrap();
        s.start();
        let mut cursor = 0;
        let first = next_request(&s, &mut cursor).expect("the mulligan");
        assert_eq!(first["request"]["kind"], "mulligan");
        let names = first["request"]["data"]["names"].as_array().unwrap().clone();
        if !names.iter().any(|n| n == "Leyline of the Guildpact") {
            s.close(Duration::from_secs(10));
            continue;
        }
        let seat = first["seat"].as_str().unwrap().to_string();
        s.answer(&seat, &json!(0)); // keep
        let ask = next_request(&s, &mut cursor).expect("the question");
        let prompt = ask["request"]["prompt"].as_str().unwrap();
        assert!(prompt.contains("Leyline of the Guildpact"), "asked about Leyline: {prompt}");
        assert_eq!(ask["request"]["choices"], json!(["yes", "no"]));
        s.answer(&seat, &json!(0)); // yes
        // the game goes on: the next thing asked is the game's own (a priority or attack decision), not Leyline again
        if let Some(next) = next_request(&s, &mut cursor) {
            assert!(!next["request"]["prompt"].as_str().unwrap_or("").contains("begin the game"));
        }
        s.close(Duration::from_secs(10));
        return;
    }
    panic!("no seed in 1..400 gave the Thanos seat a Leyline in its opening hand");
}
