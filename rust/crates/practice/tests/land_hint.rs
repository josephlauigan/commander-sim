//! The main-phase hint makes the AI's land drop first and says why: the land's points (new colours, untapped, ...)
//! or, for a Tier 6 deck, the plan's reason when the planner plays a different land; the other lands with their points;
//! and no land once the land drop is used. Asking changes nothing in the real game.

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::advisor;
use serde_json::{Value, json};
use sim_core::control::Request;
use sim_core::ids::PlayerId;
use sim_core::run::settings;
use sim_core::state::{Game, Step};

/// seat 0 is `deck` at its main phase on round 3, with these lands on the battlefield and exactly this hand
fn game_with(deck: &str, lands: &[&str], hand: &[&str]) -> (Game, PlayerId) {
    let e = engine();
    let others = ["aragorn", "sheoldred", "cloud", "thanos", "yshtola"];
    let mut seats: Vec<String> = vec![deck.to_string()];
    seats.extend(others.iter().filter(|k| **k != deck).take(3).map(|s| s.to_string()));
    let (mut g, _) =
        practice::session::setup::build_game(&e, settings("loose", "adaptive", 1.0).unwrap(), 77, &seats, &[]).unwrap();
    let p = PlayerId(0);
    let back = std::mem::take(&mut g.player_mut(p).hand);
    g.player_mut(p).library.extend(back);
    let take = |g: &mut Game, n: &str| {
        let i = g
            .player(p)
            .library
            .iter()
            .position(|&c| &*g.db.get(c).name == n)
            .unwrap_or_else(|| panic!("{deck}: no {n} in the library"));
        g.player_mut(p).library.remove(i)
    };
    for n in lands {
        let c = take(&mut g, n);
        sim_core::engine::turn::play_land_card(&mut g, p, c, "plays").unwrap();
    }
    for n in hand {
        let c = take(&mut g, n);
        g.player_mut(p).hand.push(c);
    }
    g.player_mut(p).lands_played = 0;
    g.round = 3;
    g.active = Some(p);
    g.step = Step::Main1;
    (g, p)
}

fn hint(g: &mut Game, p: PlayerId) -> Value {
    let req = Request {
        kind: sim_core::sym::intern("priority"),
        prompt: "x".into(),
        choices: vec![],
        data: json!({"stack": []}),
    };
    advisor::hint(g, p, &req)
}

fn detail(h: &Value) -> Vec<String> {
    h["detail"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
}

fn snapshot(g: &Game, p: PlayerId) -> (Vec<String>, usize, i32, u32) {
    let pl = g.player(p);
    let hand = pl.hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    (hand, pl.lands.len(), pl.life, pl.lands_played as u32)
}

#[test]
fn the_hint_plays_the_ais_land_first_and_says_why() {
    // Aragorn with an Island out: Yavimaya Coast (green, untapped) against Path of Ancestry (three new colours, tapped)
    let (mut g, p) =
        game_with("aragorn", &["Island"], &["Yavimaya Coast", "Path of Ancestry", "Counterspell", "Lightning Bolt"]);
    let before = snapshot(&g, p);
    let h = hint(&mut g, p);
    assert_eq!(snapshot(&g, p), before, "asking changes nothing: {h}");
    // the land named is the usual choice's best by points, read the same way on a copy
    let g2 = practice::advisor::copy(&mut g, p);
    let best = sim_core::engine::turn::land_reasons(&g2, p);
    let top = g2.db.get(best[0].card).name.to_string();
    let text = h["text"].as_str().unwrap();
    assert!(text.starts_with(&format!("Play {top}, then ")), "{h}");
    assert_eq!(h["land"], json!(format!("Play {top}")), "{h}");
    let d = detail(&h);
    let why = d.iter().find(|l| l.starts_with("Why:")).unwrap_or_else(|| panic!("{h}"));
    assert!(why.contains("points"), "{why}");
    assert!(why.contains("your lands don't make yet") || why.contains("no new colour"), "{why}");
    assert!(why.contains("untapped") || why.contains("tapped"), "{why}");
    let other = if top == "Path of Ancestry" { "Yavimaya Coast" } else { "Path of Ancestry" };
    let others = d.iter().find(|l| l.starts_with("Your other lands, by points:")).unwrap_or_else(|| panic!("{h}"));
    assert!(others.contains(other), "{others}");
    // after the land, the spell part: what to cast with the land in play, or pass (Path of Ancestry enters tapped)
    assert!(text.ends_with("then pass") || text.contains(", then cast "), "{h}");
}

#[test]
fn a_new_colour_beats_a_basic_of_a_colour_you_have() {
    // Y'shtola with a Swamp out: a Plains (white is new) beats a second Swamp
    let (mut g, p) = game_with("yshtola", &["Swamp"], &["Plains", "Swamp", "Rhystic Study", "Counterspell"]);
    let h = hint(&mut g, p);
    assert!(h["text"].as_str().unwrap().starts_with("Play Plains, then "), "{h}");
    let d = detail(&h);
    let why = d.iter().find(|l| l.starts_with("Why:")).unwrap();
    assert!(why.contains("makes white, which your lands don't make yet"), "{why}");
    let others = d.iter().find(|l| l.starts_with("Your other lands")).unwrap();
    assert!(others.contains("Swamp") && others.contains("no new colour"), "{others}");
}

#[test]
fn no_land_once_the_land_drop_is_used() {
    let (mut g, p) = game_with("aragorn", &["Island"], &["Yavimaya Coast", "Counterspell", "Lightning Bolt"]);
    g.player_mut(p).lands_played = 1;
    let h = hint(&mut g, p);
    assert!(!h["text"].as_str().unwrap().starts_with("Play "), "{h}");
    assert!(h.get("land").is_none(), "{h}");
    assert!(!detail(&h).iter().any(|l| l.starts_with("Land first")), "{h}");
}

#[test]
fn thrasios_plays_the_land_its_win_line_needs_and_says_so() {
    // Underground Sea and Scrubland out; Thassa's Oracle ({U}{U}) and Demonic Consultation ({B}) in hand. By points
    // Bayou wins (green is new); only Otawara gives the second blue, so the planner plays it: the line goes off now
    let (mut g, p) = game_with(
        "thrasios",
        &["Underground Sea", "Scrubland"],
        &["Bayou", "Otawara, Soaring City", "Thassa's Oracle", "Demonic Consultation"],
    );
    let before = snapshot(&g, p);
    let h = hint(&mut g, p);
    assert_eq!(snapshot(&g, p), before, "asking changes nothing: {h}");
    assert!(h["text"].as_str().unwrap().starts_with("Play Otawara, Soaring City, then "), "{h}");
    let d = detail(&h);
    let why = d.iter().find(|l| l.starts_with("Why:")).unwrap_or_else(|| panic!("{h}"));
    assert!(why.contains("the plan") && why.contains("this turn"), "{why}");
    assert!(d.iter().any(|l| l.starts_with("(By points alone the pick would be Bayou")), "{h}");
}
