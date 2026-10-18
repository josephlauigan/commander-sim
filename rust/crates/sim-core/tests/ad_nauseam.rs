//! Ad Nauseam's shared AI (ai/plans.rs adnaus_*, engine/tutors.rs ad_nauseam, ai/brain.rs card_utility): cast at the
//! end of the turn before yours, flip only while the life stays above a floor that counts more than one opponent, and
//! never past a hand you can use. Every deck that plays the card uses these rules.

use sim_core::ai::{brain, plans};
use sim_core::engine::tutors;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

fn tbl() -> Game {
    table(&["kefka", "veyran", "sauron", "seph"])
}

/// p's library: these cards, the last on top
fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let lib: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = lib;
}

#[test]
fn the_floor_counts_more_than_the_biggest_board() {
    let mut g = tbl();
    assert_eq!(plans::adnaus_floor(&g, P0), 10, "an empty table: the minimum");
    token(&mut g, P1, 6);
    token(&mut g, P2, 4);
    token(&mut g, P3, 2);
    // 6 + (4 + 2) / 2 + 6
    assert_eq!(plans::adnaus_floor(&g, P0), 15);
}

#[test]
fn the_life_per_card_is_the_librarys_average_mana_value() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Swamp", "Swamp", "Sol Ring", "Massacre Wurm"]);
    // (0 + 0 + 1 + 6) / 4
    assert!((plans::adnaus_life_per_card(&g, P0) - 1.75).abs() < 1e-9);
}

#[test]
fn it_stops_at_a_hand_you_can_use() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 5, false);
    hand(&mut g, P0, &["Island", "Island", "Island"]);
    set_library(&mut g, P0, &["Swamp"; 30]);
    let cap = plans::adnaus_hand_cap(&g, P0);
    assert_eq!(cap, 9, "seven plus (5 lands + a land drop) / 3");
    tutors::ad_nauseam(&mut g, P0, None).unwrap();
    assert_eq!(g.player(P0).hand.len(), cap, "free lands: it flips exactly to the cap");
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn it_stops_above_the_floor() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Massacre Wurm"; 30]);
    g.player_mut(P0).life = 30;
    tutors::ad_nauseam(&mut g, P0, None).unwrap();
    // floor 10, room for a 7-drop: it flips while life - 7 > 10, six life a card: 30 -> 24 -> 18 -> 12 (stops)
    assert_eq!(g.player(P0).life, 12);
    assert_eq!(g.player(P0).hand.len(), 3);
}

#[test]
fn it_waits_for_the_end_of_the_turn_before_yours() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Ad Nauseam");
    g.player_mut(P0).hand.push(c);
    set_library(&mut g, P0, &["Swamp"; 30]);
    g.active = Some(P0);
    let s = brain::Situation::new(&g, P0);
    assert_eq!(brain::card_utility(&g, P0, &s, c), None, "not in your own main phase");
    g.active = Some(P3);
    let s = brain::Situation::new(&g, P0);
    assert!(brain::card_utility(&g, P0, &s, c).is_some(), "at the end of the turn before yours");
}

#[test]
fn a_full_hand_holds_it() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Ad Nauseam");
    g.player_mut(P0).hand.push(c);
    hand(&mut g, P0, &["Island"; 7]);
    set_library(&mut g, P0, &["Swamp"; 30]);
    // cap 7 + 1 / 3 = 7: room for one card after the cast, not four. It is still worth keeping (not discarded).
    g.active = Some(P3);
    assert!(!plans::adnaus_cast_now(&g, P0));
    assert!(plans::adnaus_prio(&g, P0, c) > 0, "its value doesn't depend on the hand");
}

#[test]
fn your_main_phase_keeps_its_mana_up() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 6, false);
    let c = take(&mut g, P0, "Ad Nauseam");
    g.player_mut(P0).hand.push(c);
    set_library(&mut g, P0, &["Swamp"; 30]);
    g.active = Some(P0);
    let s = brain::Situation::new(&g, P0);
    let (hc, hv) = brain::hold_value(&g, P0, &s);
    assert_eq!(hc, Some(c), "the mana is held for Ad Nauseam");
    assert!(hv > 3.0, "{hv}");
}
