//! search-hold: with `hold_w` on, the look-ahead credits the "stop (hold mana)" line with the heuristic's hold value
//! (`search::hold_bonus`), so mana kept up for a counterspell or Ad Nauseam isn't spent on a creature because the
//! playout ends before the held card pays off.

use sim_core::ai::{brain, search};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);

fn game() -> Game {
    let mut g = table(&["sheoldred", "veyran", "sauron", "seph"]); // Ad Nauseam's own deck, no counterspells to hold mana for
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    let cs: Vec<_> = (0..30).map(|_| card(&g, "Swamp")).collect();
    g.player_mut(P0).library = cs;
    g
}

fn set_hold_w(g: &mut Game, w: f64) {
    let mut s = (*g.settings).clone();
    s.search.hold_w = w;
    g.settings = Arc::new(s);
}

fn ad_nauseam_and_a_creature(g: &mut Game) {
    // a library like a real one: lands, cheap spells and a few big ones (about 2.3 life a card)
    let names = ["Swamp", "Night's Whisper", "Swamp", "Massacre Wurm", "Swamp", "Lightning Bolt", "Sol Ring"];
    let cs: Vec<_> = (0..35).map(|i| card(g, names[i % names.len()])).collect();
    g.player_mut(P0).library = cs;
    lands(g, P0, "Swamp", 5, false);
    hand(g, P0, &["Ad Nauseam", "Sheoldred, the Apocalypse"]);
    g.player_mut(P0).turns = 5;
}

#[test]
fn only_the_hold_line_gets_the_bonus() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 1.0);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let hold = opts.iter().find(|o| o.label == "stop (hold mana)").expect("mana held for Ad Nauseam");
    let s = brain::Situation::new(&g, P0);
    let hv = brain::hold_value(&g, P0, &s).1;
    assert!(hv > 0.0);
    assert_eq!(search::hold_bonus(&g, P0, hold), hv);
    let hawk = opts.iter().find(|o| o.label == "Sheoldred, the Apocalypse").expect("the creature is castable");
    assert_eq!(search::hold_bonus(&g, P0, hawk), 0.0, "a cast gets nothing");
    set_hold_w(&mut g, 0.0);
    assert_eq!(search::hold_bonus(&g, P0, hold), 0.0, "hold_w 0: the old search");
}

/// --- ai-rollouts: with the adaptive rollouts (the default) the look-ahead finds casting Sheoldred ahead of holding
/// five mana for Ad Nauseam (16.5 against 15.7 a playout) past the margin at z = 2.36, and casts it: the deck has no
/// counterspell to hold for, and a resolved Sheoldred is the stronger play (JD accepted it, 2026-10-29, rollouts.md)
#[test]
fn with_adaptive_rollouts_the_lookahead_casts_sheoldred_over_holding() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 1.0);
    assert!(g.settings.search.adapt, "adaptive rollouts are the default");
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    let scores = search::LAST_SCORES.with(|l| l.borrow().clone());
    assert_eq!(pick.as_deref(), Some("Sheoldred, the Apocalypse"), "{scores:?}");
}

#[test]
fn at_six_rollouts_the_lookahead_keeps_ad_nauseams_mana_up() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 1.0);
    // --- ai-rollouts: the six-rollout rule (adaptive rollouts off): the margin hands this close call back to the
    // heuristic, which holds the mana
    let mut s = (*g.settings).clone();
    s.search.adapt = false;
    g.settings = Arc::new(s);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    assert_eq!(pick.as_deref(), Some("stop (hold mana)"), "{:?}", search::LAST_SCORES.with(|l| l.borrow().clone()));
}

#[test]
fn without_the_bonus_the_lookahead_taps_out() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 0.0);
    // --- ai-eval: the old search is also the bare argmax (margin 0): with the margin the heuristic's top play
    // (holding for Ad Nauseam) stands, since Sheoldred's lead is inside the noise of six rollouts
    let mut s = (*g.settings).clone();
    s.search.margin = 0.0;
    g.settings = Arc::new(s);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    assert_eq!(pick.as_deref(), Some("Sheoldred, the Apocalypse"), "the old search spends the mana");
}

/// the scion seat with a ready Thassa's Oracle line and a counterspell to hold mana for, against three inert goldfish
/// seats: every playout of both the combo and the held line wins
fn oracle_ready_in_a_goldfish() -> Game {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    set_ai(&mut g, AiMode::Lookahead);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P0).library = lib;
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Flusterstorm"]);
    g.player_mut(P0).turns = 4;
    g
}

#[test]
fn a_hold_bonus_never_beats_a_sure_win() {
    let mut g = oracle_ready_in_a_goldfish();
    assert!(sim_core::goldfish::is_goldfish(&g));
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    assert!(opts.iter().any(|o| o.label == "stop (hold mana)"), "mana held for Flusterstorm");
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    let scores = search::LAST_SCORES.with(|l| l.borrow().clone());
    assert!(pick.as_deref().is_some_and(|l| l.starts_with("combo: Thassa's Oracle")), "{pick:?} {scores:?}");
    assert!(scores.iter().all(|(_, s)| *s <= 100.0), "{scores:?}");
}
