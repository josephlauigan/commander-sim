//! Smaug the Magnificent (impls/smaug.rs), on hand-built positions with its Oracle text as the expectation: flying,
//! haste; whenever Smaug attacks, he deals damage equal to the number of Treasures you control to any target; at the
//! beginning of your upkeep, create a Treasure token.

use sim_core::engine::{combat, turn};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SMAUG: &str = "Smaug the Magnificent";

fn tbl() -> Game {
    table(&["galadriel", "veyran", "seph"])
}

fn lives(g: &Game) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

#[test]
fn smaug_has_flying_and_haste() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SMAUG);
    assert!(sim_core::dsl::has_kw(&g, s, "flying") && sim_core::dsl::has_kw(&g, s, "haste"));
}

#[test]
fn smaug_makes_a_treasure_in_your_upkeep() {
    let mut g = tbl();
    perm(&mut g, P0, SMAUG);
    let t = g.player(P0).treasures;
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).treasures, t + 1);
}

#[test]
fn smaug_with_no_treasures_deals_no_damage() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SMAUG);
    g.player_mut(P0).treasures = 0;
    let before = lives(&g);
    combat::attack_triggers(&mut g, P0, &[s], P1).unwrap();
    assert_eq!(lives(&g), before);
}

#[test]
fn smaug_kills_a_player_the_treasures_can_finish() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SMAUG);
    perm(&mut g, P1, "Vampire Nighthawk");
    g.player_mut(P0).treasures = 3;
    g.player_mut(P2).life = 3;
    combat::attack_triggers(&mut g, P0, &[s], P1).unwrap();
    assert!(g.player(P2).life <= 0, "lethal beats killing a creature");
    assert_eq!(g.player(P0).treasures, 3, "the Treasures are counted, not spent");
}

#[test]
fn smaug_kills_a_flier_that_could_block_him() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SMAUG);
    let hawk = perm(&mut g, P1, "Vampire Nighthawk");
    token(&mut g, P2, 2);
    g.player_mut(P0).treasures = 3;
    let before = lives(&g);
    combat::attack_triggers(&mut g, P0, &[s], P1).unwrap();
    assert!(!g.perm(hawk).on_bf, "3 damage kills the 2/3 flier");
    assert_eq!(lives(&g), before, "no player was hit");
}

#[test]
fn smaug_goes_face_when_nothing_dies_to_it() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SMAUG);
    let hawk = perm(&mut g, P1, "Vampire Nighthawk");
    g.player_mut(P0).treasures = 2;
    g.player_mut(P2).life = 25;
    combat::attack_triggers(&mut g, P0, &[s], P1).unwrap();
    assert!(g.perm(hawk).on_bf, "2 damage doesn't kill a 2/3");
    assert_eq!(g.player(P2).life, 23, "the opponent with the least life takes it");
}

#[test]
fn another_attacker_does_not_trigger_smaug() {
    let mut g = tbl();
    perm(&mut g, P0, SMAUG);
    let other = token(&mut g, P0, 2);
    g.player_mut(P0).treasures = 5;
    g.player_mut(P2).life = 5;
    combat::attack_triggers(&mut g, P0, &[other], P1).unwrap();
    assert_eq!(g.player(P2).life, 5);
}
