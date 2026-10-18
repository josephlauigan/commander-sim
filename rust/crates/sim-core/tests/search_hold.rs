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
    let mut g = table(&["kefka", "veyran", "sauron", "seph"]);
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

#[test]
fn lookahead_keeps_ad_nauseams_mana_up() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 1.0);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    assert_eq!(pick.as_deref(), Some("stop (hold mana)"), "{:?}", search::LAST_SCORES.with(|l| l.borrow().clone()));
}

#[test]
fn without_the_bonus_the_lookahead_taps_out() {
    let mut g = game();
    ad_nauseam_and_a_creature(&mut g);
    set_hold_w(&mut g, 0.0);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    assert_eq!(pick.as_deref(), Some("Sheoldred, the Apocalypse"), "the old search spends the mana");
}
