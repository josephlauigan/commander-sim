//! Cloud package B (backup carriers): the two printed-keyword creatures, plus the package cards' basic behaviour.

use sim_core::cards::Colors;
use sim_core::engine::combat::double_strike;
use sim_core::engine::values::protected_from;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn game() -> Game {
    table(&["cloud", "seph"])
}

#[test]
fn fencing_ace_has_double_strike() {
    let mut g = game();
    let a = perm(&mut g, P0, "Fencing Ace");
    assert!(double_strike(&g, a));
    assert_eq!((g.db.get(g.perm(a).cd.unwrap()).pow, g.db.get(g.perm(a).cd.unwrap()).tgh), (1, 1));
}

#[test]
fn mirran_crusader_has_double_strike_and_protection_from_black_and_green() {
    let mut g = game();
    let a = perm(&mut g, P0, "Mirran Crusader");
    assert!(double_strike(&g, a));
    assert!(protected_from(&g, a, Colors::from_letters("B")));
    assert!(protected_from(&g, a, Colors::from_letters("G")));
    assert!(!protected_from(&g, a, Colors::from_letters("R")));
    assert!(!protected_from(&g, a, Colors::from_letters("W")));
}

#[test]
fn a_double_strike_carrier_wearing_a_sword_has_the_power_to_hit_twice() {
    // the reason for the package: Fencing Ace (1/1) plus Sword of Fire and Ice is a 3-power double striker
    let mut g = game();
    let a = perm(&mut g, P0, "Fencing Ace");
    let s = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(s).attached = Some(a);
    assert!(double_strike(&g, a));
    assert_eq!(sim_core::engine::values::epow(&g, a), 3);
}

#[test]
fn wall_of_omens_draws_when_it_enters() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(g.player(P0).hand.len(), before + 1);
}
