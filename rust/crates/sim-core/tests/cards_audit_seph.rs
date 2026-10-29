//! AI audit phase 2, Seph/Atraxa (seph): Hornet Queen and Muldrotha rated as bombs, one threshold, token blink value,
//! Entomb for a loop piece, Momentary Blink as protection, Grand Abolisher timing, Tireless Tracker before the land
//! drop, the late tutor wish list.

use sim_core::ai::{self, decks, seph};
use sim_core::engine::{removal, turn, values};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::mine;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| values::card_name(g, m) == Some(name))
}

fn to_gy(g: &mut Game, name: &str) -> CardId {
    let c = take(g, P0, name);
    g.player_mut(P0).gy.push(c);
    c
}

#[test]
fn hornet_queen_and_muldrotha_are_bombs_for_seph() {
    let g = table(&["seph", "veyran"]);
    assert_eq!(seph::sbomb(&g, card(&g, "Hornet Queen")), 7);
    assert_eq!(seph::sbomb(&g, card(&g, "Muldrotha, the Gravetide")), 6);
    assert_eq!(seph::sbomb(&g, card(&g, "Archon of Cruelty")), 9);
    assert_eq!(g.db.get(card(&g, "Hornet Queen")).bomb, 0); // the shared card line is unchanged (pool decks)
}

#[test]
fn a_hornet_queen_in_the_yard_is_a_reanimation_target_and_counts_as_a_bomb() {
    let mut g = table(&["seph", "veyran"]);
    let hq = to_gy(&mut g, "Hornet Queen");
    assert!(seph::own_bomb_in_gy(&g, P0));
    assert_eq!(decks::rean_targets(&g, P0, "animate")[0].1, hq);
}

#[test]
fn mikaeus_in_the_yard_counts_as_a_bomb_in_the_yard() {
    let mut g = table(&["seph", "veyran"]);
    assert!(!seph::own_bomb_in_gy(&g, P0));
    to_gy(&mut g, "Mikaeus, the Unhallowed");
    assert!(seph::own_bomb_in_gy(&g, P0));
}

#[test]
fn hornet_queen_is_hardcast_and_a_bomb_on_the_board_stops_the_tutoring() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Forest", 7, false);
    let hq = hand(&mut g, P0, &["Hornet Queen"])[0];
    assert_eq!(seph::seph_prio(&g, P0, hq), 0); // not a plain creature any more: the hardcast plan casts her
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(opts.iter().any(|o| o.label == "hardcast bomb"));
    perm(&mut g, P0, "Hornet Queen");
    assert!(decks::bomb_on_bf(&g, P0));
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
fn entomb_bins_the_missing_loop_piece_before_the_biggest_bomb() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Mikaeus, the Unhallowed");
    perm(&mut g, P0, "Viscera Seer");
    hand(&mut g, P0, &["Animate Dead"]);
    lands(&mut g, P0, "Swamp", 3, false);
    let tri = card(&g, "Triskelion");
    assert!(g.player(P0).library.contains(&tri));
    assert!(seph::fill_pieces(&g, P0).contains(&tri));
    hand(&mut g, P0, &["Entomb"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(opts.iter().any(|o| o.label == "fill graveyard"));
    ai::seph_fill_resolve(&mut g, P0, "entomb".into(), &Ctx::default()).unwrap();
    let binned = g.player(P0).gy.clone();
    assert!(binned.len() == 1 && (binned[0] == tri || seph::sbomb(&g, binned[0]) == 0), "a loop piece, not Archon: {binned:?}");
}

#[test]
fn momentary_blink_saves_a_bomb_from_removal_and_flashbacks() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Momentary Blink"]);
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), archon, "exile", Some(stp)).unwrap();
    assert!(named(&g, P0, "Archon of Cruelty").is_some()); // blinked: the Swords fizzled
    assert_eq!(g.player(P0).stats.get("momentary_blink_protection").copied(), Some(1));
    let mb = card(&g, "Momentary Blink");
    assert!(g.player(P0).gy.contains(&mb));
    // the flashback ({3}{U}) is the second save
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Island", 4, false);
    to_gy(&mut g, "Momentary Blink");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    removal::apply_removal(&mut g, Some(P1), archon, "exile", Some(stp)).unwrap();
    assert!(named(&g, P0, "Archon of Cruelty").is_some());
    assert!(g.player(P0).exile.contains(&mb));
}

#[test]
fn grand_abolisher_waits_for_the_combo_turn() {
    let mut g = table(&["seph", "veyran"]);
    let gab = hand(&mut g, P0, &["Grand Abolisher"])[0];
    assert_eq!(seph::seph_prio(&g, P0, gab), 35);
    perm(&mut g, P0, "Mikaeus, the Unhallowed");
    perm(&mut g, P0, "Viscera Seer"); // one piece (Triskelion) from a loop
    assert_eq!(seph::seph_prio(&g, P0, gab), 70);
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

#[test]
fn tracker_is_cast_before_the_land_drop() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Tireless Tracker", "Forest"]);
    turn::play_land(&mut g, P0).unwrap();
    assert!(named(&g, P0, "Tireless Tracker").is_some());
    assert_eq!(g.player(P0).clues, 1); // the land drop made a Clue
    assert_eq!(g.player(P0).lands.len(), 4);
}

#[test]
fn tracker_waits_when_the_mana_reanimates() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Tireless Tracker", "Swamp", "Animate Dead"]);
    to_gy(&mut g, "Archon of Cruelty");
    turn::play_land(&mut g, P0).unwrap();
    assert!(named(&g, P0, "Tireless Tracker").is_none());
}

#[test]
fn late_tutors_find_the_wipe_under_pressure_and_muldrotha_in_a_long_game() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Archon of Cruelty"); // a bomb is out
    g.player_mut(P0).turns = 3;
    for _ in 0..4 {
        token(&mut g, P1, 3);
    }
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Toxic Deluge"));
    g.player_mut(P0).life = 6;
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Austere Command"));
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P0).turns = 9;
    perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Muldrotha, the Gravetide")); // the long game
    g.player_mut(P0).turns = 4;
    // the tutor looks in the library: keep Swan Song there whatever the opening hand dealt
    let sw = card(&g, "Swan Song");
    g.player_mut(P0).hand.retain(|&c| c != sw);
    if !g.player(P0).library.contains(&sw) {
        g.player_mut(P0).library.push(sw);
    }
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Swan Song"));
}
