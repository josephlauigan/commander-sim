//! The Hint and the AI comparison (Python's test_play_hint / test_play_shadow / test_play_review) on a game held at
//! the person's main phase, built directly (the session drives these through the person's own turns, which the
//! engine gets with practice-core).

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::advisor;
use practice::session::setup::build_game;
use practice::shadow::{Entry, Shadow};
use serde_json::json;
use sim_core::control::{Answer, Request};
use sim_core::ids::PlayerId;
use sim_core::run::settings;
use sim_core::state::{Game, Step};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn main_phase_game(ai: &str) -> (Game, PlayerId) {
    let e = engine();
    let seats: Vec<String> = ["sauron", "ghave", "veyran", "galadriel"].iter().map(|s| s.to_string()).collect();
    let (mut g, _) = build_game(&e, settings("loose", ai, 1.0).unwrap(), 77, &seats, &[]).unwrap();
    let p = PlayerId(0);
    g.round = 3;
    g.active = Some(p);
    g.step = Step::Main1;
    // some lands to work with
    (g, p)
}

fn req(kind: &str) -> Request {
    Request { kind: sim_core::sym::intern(kind), prompt: "x".into(), choices: vec![], data: json!({"stack": []}) }
}

#[test]
fn the_hint_names_a_play() {
    for ai in ["adaptive", "lookahead"] {
        let (mut g, p) = main_phase_game(ai);
        let before = g.player(p).hand.clone();
        let h = advisor::hint(&mut g, p, &req("priority"));
        assert!(h["text"].is_string(), "{h}");
        assert!(h["detail"].is_array());
        assert_eq!(g.player(p).hand, before, "asking changes nothing");
        assert_eq!(g.search_n, 0, "the real game's look-ahead counters stay put");
    }
    let (mut g, p) = main_phase_game("adaptive");
    let h = advisor::hint(&mut g, p, &req("mulligan"));
    assert!(h["text"].as_str().unwrap().starts_with("Keep") || h["text"].as_str().unwrap().starts_with("Mulligan"), "{h}");
    g.step = Step::Combat;
    let h = advisor::hint(&mut g, p, &req("priority"));
    assert_eq!(h["text"], "Pass, unless you have an instant you want to use now.");
}

#[test]
fn the_comparison_scores_what_you_did_against_the_ai() {
    let (g, p) = main_phase_game("lookahead");
    let sh = Arc::new(Shadow::new());
    sh.record(&g, p, &req("priority"), &Answer::Action(json!({"do": "pass"})), 5);
    sh.record(&g, p, &req("priority"), &Answer::Action(json!({"do": "tap", "source": 0})), 6); // not a decision
    let mut attack = req("attack");
    attack.choices = vec!["A (1/1)".into(), "B (2/2)".into()];
    sh.record(&g, p, &attack, &Answer::Picks(vec![0, 1]), 7);
    assert_eq!(sh.entries().len(), 2);
    let mut seen = vec![];
    sh.finish(&mut |d, t| seen.push((d, t)), &AtomicBool::new(false));
    assert_eq!(sh.pending(), 0);
    let es = sh.entries();
    let main = &es[0];
    assert_eq!((main.n, main.kind.as_str(), main.step.as_str()), (5, "main", "main1"));
    assert_eq!(main.yours, "Pass (nothing more this phase)");
    assert!(main.scored || !main.note.is_empty(), "{main:?}");
    if main.scored {
        assert!(main.ai.is_some() && main.scores.len() >= 2);
        let d = main.as_dict();
        assert_eq!(d["can_try"], true);
        assert!(d["score_yours"].is_number() && d["score_ai"].is_number());
    }
    let atk = &es[1];
    assert_eq!((atk.kind.as_str(), atk.yours.as_str()), ("attack", "Attack with A, B"));
    assert!(atk.scored && atk.ai_plan.is_some(), "{atk:?}");
    assert_eq!(atk.scores.len(), 7); // filtered + all against each of three opponents, and no attack
    assert!(atk.yours_key.as_deref().unwrap().contains("with everything"));
    // a saved comparison loads back the same
    for e in &es {
        let back = Entry::from_saved(&e.saved()).unwrap();
        assert_eq!(back.as_dict(), e.as_dict());
        assert_eq!(back.ai_answer(), e.ai_answer());
    }
    // Undo forgets the decisions from n on
    sh.forget_from(7);
    assert_eq!(sh.entries().len(), 1);
    sh.shutdown();
    assert_eq!(sh.workers_running(), 0);
}

#[test]
fn the_ais_choice_becomes_your_answer_when_it_is_one_action() {
    let mut e = Entry { n: 3, kind: "main".into(), hand: vec!["Sol Ring".into(), "Island".into()], ..Default::default() };
    e.ai = Some("Island -> x".into());
    assert_eq!(e.ai_answer(), Some(Answer::Action(json!({"do": "cast", "card": 1}))));
    e.ai = Some("stop (hold mana)".into());
    assert_eq!(e.ai_answer(), Some(Answer::Action(json!({"do": "pass"}))));
    assert_eq!(e.ai_text().unwrap(), "Pass (nothing more this phase)");
    e.ai = Some("Mystery".into());
    assert_eq!(e.ai_answer(), None);
    let mut a = Entry { kind: "attack".into(), n_choices: 3, ..Default::default() };
    a.ai = Some("Attack X with everything".into());
    a.ai_plan = Some((1, "all".into()));
    assert_eq!(a.ai_answer(), Some(Answer::Picks(vec![0, 1, 2])));
    a.ai_plan = Some((1, "none".into()));
    assert_eq!(a.ai_answer(), Some(Answer::Picks(vec![])));
    a.ai_plan = Some((1, "filtered".into()));
    assert_eq!(a.ai_answer(), None);
}

#[test]
fn the_review_compares_your_mulligans_with_the_ais() {
    let (mut g, p) = main_phase_game("adaptive");
    // a hand of seven lands: the AI mulligans it
    let pl = g.player_mut(p);
    let all: Vec<_> = pl.hand.drain(..).chain(pl.library.drain(..)).collect();
    let (lands, spells): (Vec<_>, Vec<_>) = all.into_iter().partition(|&c| g.db.get(c).land);
    let pl = g.player_mut(p);
    pl.hand = lands.iter().copied().take(7).collect();
    pl.library = lands.into_iter().skip(7).chain(spells).collect();
    let mut r = req("mulligan");
    r.prompt = "Your opening hand (7 to keep).".into();
    let sh = Arc::new(Shadow::new());
    sh.record(&g, p, &r, &Answer::Pick(0), 1); // you keep
    sh.record(&g, p, &r, &Answer::Pick(1), 2); // you mulligan
    let es = sh.entries();
    assert_eq!((es[0].kind.as_str(), es[0].yours.as_str()), ("mulligan", "Keep"));
    let (a, b) = (es[0].as_dict(), es[1].as_dict());
    assert_eq!(a["differs"], true, "{a}");
    assert!(a["ai_text"].as_str().unwrap().starts_with("Mulligan (7 lands"), "{a}");
    assert_eq!(b["differs"], false, "{b}");
    let back = Entry::from_saved(&es[0].saved()).unwrap();
    assert_eq!(back.as_dict(), a, "a saved game keeps the mulligan comparison");
}
