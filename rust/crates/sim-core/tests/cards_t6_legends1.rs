//! Tier 6 "legends1" batch (impls/t6_legends1.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["sisay-legends-toolbox", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn tapped(g: &Game, m: PermId) -> bool {
    g.perm(m).tapped
}

// ======================================================== Derevi, Empyrial Tactician
#[test]
fn derevi_enters_and_untaps_a_tapped_mana_creature() {
    let mut g = tbl();
    let dork = perm(&mut g, P0, "Bloom Tender");
    g.perm_mut(dork).tapped = true;
    let d = perm(&mut g, P0, "Derevi, Empyrial Tactician");
    assert!(g.perm(d).on_bf);
    assert!(!tapped(&g, dork), "enters: untap target permanent");
}

#[test]
fn derevi_taps_an_opposing_blocker_when_nothing_of_ours_needs_untapping() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    assert!(tapped(&g, big), "on our turn Derevi taps the best blocker");
}

#[test]
fn derevi_untaps_when_a_creature_deals_combat_damage_to_a_player() {
    let mut g = tbl();
    let d = perm(&mut g, P0, "Derevi, Empyrial Tactician");
    let dork = perm(&mut g, P0, "Bloom Tender");
    g.perm_mut(dork).tapped = true;
    let hitter = token(&mut g, P0, 3);
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::CombatDamage,
        sim_core::hooks::Call::CombatDamage { p: P0, a: hitter, d: P1, dmg: 3 },
    )
    .unwrap();
    assert!(!tapped(&g, dork));
    let _ = d;
}

#[test]
fn derevi_ignores_damage_by_an_opponents_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    let dork = perm(&mut g, P0, "Bloom Tender");
    g.perm_mut(dork).tapped = true;
    let hitter = token(&mut g, P1, 3);
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::CombatDamage,
        sim_core::hooks::Call::CombatDamage { p: P1, a: hitter, d: P0, dmg: 3 },
    )
    .unwrap();
    assert!(tapped(&g, dork));
}

// ======================================================== Emiel the Blessed
#[test]
fn emiel_pays_for_a_counter_on_an_entering_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Emiel the Blessed");
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = true;
    }
    lands(&mut g, P0, "Plains", 1, false);
    let c = perm(&mut g, P0, "Bloom Tender");
    assert_eq!(g.perm(c).plus, 1, "pay W: +1/+1 counter");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "the Plains paid");
}

#[test]
fn emiel_puts_two_counters_on_a_unicorn_and_none_without_mana() {
    let mut g = tbl();
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = true;
    }
    perm(&mut g, P0, "Emiel the Blessed");
    let none = perm(&mut g, P0, "Baneslayer Angel");
    assert_eq!(g.perm(none).plus, 0, "no mana, no counter");
    lands(&mut g, P0, "Forest", 1, false);
    g.perm_mut(none).tapped = false;
    let e2 = perm(&mut g, P0, "Emiel the Blessed");
    assert_eq!(g.perm(e2).plus, 2, "the first Emiel pays G: a Unicorn gets two");
}

// ======================================================== Selvala, Heart of the Wilds
#[test]
fn selvala_draws_for_the_biggest_creature_entering() {
    let mut g = tbl();
    perm(&mut g, P0, "Selvala, Heart of the Wilds");
    perm(&mut g, P1, "Bloom Tender");
    let h = g.player(P0).hand.len();
    perm(&mut g, P0, "Baneslayer Angel");
    assert_eq!(g.player(P0).hand.len(), h + 1, "5 power beats every other creature");
    let h = g.player(P1).hand.len();
    perm(&mut g, P1, "Bloom Tender");
    assert_eq!(g.player(P1).hand.len(), h, "1 power does not");
}

#[test]
fn selvala_taps_for_greatest_power_less_the_g() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Selvala, Heart of the Wilds");
    g.perm_mut(s).sick = false;
    perm(&mut g, P0, "Baneslayer Angel");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, s), Some(4), "X = 5, minus the {{G}}");
}
