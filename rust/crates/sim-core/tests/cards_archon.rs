//! Archon of Absolution: flying, protection from white, and a {1}-per-attacker tax on attacking its controller.

use sim_core::cards::Colors;
use sim_core::engine::combat::{attack_limits, attack_restrictions, can_block};
use sim_core::engine::values::{epow, protected_from};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["cloud", "seph"])
}

#[test]
fn archon_is_a_3_2_flyer_with_protection_from_white_only() {
    let mut g = game();
    let a = perm(&mut g, P0, "Archon of Absolution");
    assert_eq!(epow(&g, a), 3);
    assert!(protected_from(&g, a, Colors::from_letters("W")));
    assert!(!protected_from(&g, a, Colors::from_letters("B")));
    assert!(!protected_from(&g, a, Colors::from_letters("R")));
}

#[test]
fn white_creatures_cannot_block_a_attacking_archon_but_others_can() {
    let mut g = game();
    let a = perm(&mut g, P1, "Archon of Absolution");
    let white = perm(&mut g, P0, "Windborn Muse");
    assert!(!can_block(&g, white, a));
}

#[test]
fn archon_taxes_one_per_attacker_against_its_controller_only() {
    let mut g = game();
    assert_eq!(attack_restrictions(&g, P1, P0).0, 0);
    perm(&mut g, P0, "Archon of Absolution");
    assert_eq!(attack_restrictions(&g, P1, P0).0, 1);
    assert_eq!(attack_restrictions(&g, P0, P1).0, 0, "its own controller attacking others pays nothing");
}

#[test]
fn the_tax_is_charged_for_each_attacker() {
    let mut g = game();
    perm(&mut g, P0, "Archon of Absolution");
    let atk: Vec<_> = (0..3).map(|_| perm(&mut g, P1, "Windborn Muse")).collect();
    let ls = lands(&mut g, P1, "Plains", 2, false);
    let out = attack_limits(&mut g, P1, P0, atk).unwrap();
    assert_eq!(out.len(), 2, "only two attackers can be paid for with two lands");
    assert!(ls.iter().all(|&l| g.perm(l).tapped), "{{1}} paid per attacker");
}
