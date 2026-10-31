//! AI audit phase 2, Seph/Atraxa (seph): Hornet Queen and Muldrotha rated as bombs, one threshold, token blink value,
//! Entomb for a loop piece, Momentary Blink as protection, Grand Abolisher timing, Tireless Tracker before the land
//! drop, the late tutor wish list.

use sim_core::ai::{self, decks};
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::mine;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn to_gy(g: &mut Game, name: &str) -> CardId {
    let c = take(g, P0, name);
    g.player_mut(P0).gy.push(c);
    c
}

#[test]
fn hornet_queen_and_muldrotha_are_bombs_for_seph() {
    let g = table(&["seph", "veyran"]);
    assert_eq!(ai::reanimate::sbomb(&g, card(&g, "Hornet Queen")), 7);
    assert_eq!(ai::reanimate::sbomb(&g, card(&g, "Muldrotha, the Gravetide")), 6);
    assert_eq!(ai::reanimate::sbomb(&g, card(&g, "Archon of Cruelty")), 9);
    assert_eq!(g.db.get(card(&g, "Hornet Queen")).bomb, 0); // the shared card line is unchanged (pool decks)
}

#[test]
fn a_hornet_queen_in_the_yard_is_a_reanimation_target_and_counts_as_a_bomb() {
    let mut g = table(&["seph", "veyran"]);
    let hq = to_gy(&mut g, "Hornet Queen");
    assert!(ai::reanimate::own_bomb_in_gy(&g, P0));
    assert_eq!(decks::rean_targets(&g, P0, "animate")[0].1, hq);
}

#[test]
fn mikaeus_in_the_yard_counts_as_a_bomb_in_the_yard() {
    let mut g = table(&["seph", "veyran"]);
    assert!(!ai::reanimate::own_bomb_in_gy(&g, P0));
    to_gy(&mut g, "Mikaeus, the Unhallowed");
    assert!(ai::reanimate::own_bomb_in_gy(&g, P0));
}

#[test]
fn blinking_hornet_queen_is_worth_four_insects() {
    let mut g = table(&["seph", "veyran"]);
    let hq = perm(&mut g, P0, "Hornet Queen");
    assert!(mine::etb_value(&g, P0, hq) >= 4.0);
    assert!(mine::flicker_worth(&g, P0, hq) >= 3.0);
    // the pool decks that play her are unchanged
    let mut g2 = table(&["veyran", "seph"]);
    let hq2 = perm(&mut g2, P0, "Hornet Queen");
    assert!(mine::etb_value(&g2, P0, hq2) < 1.0);
}

#[test]
fn grand_abolisher_stops_an_opponents_creature_ability_on_your_turn_only() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Grand Abolisher");
    let seer = perm(&mut g, P1, "Viscera Seer");
    assert!(sim_core::cardcode::ability_locked(&g, seer, P1));
    g.active = Some(P1);
    assert!(!sim_core::cardcode::ability_locked(&g, seer, P1));
}

