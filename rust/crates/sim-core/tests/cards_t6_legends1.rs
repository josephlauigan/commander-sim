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
        sim_core::engine::hooks::Call::CombatDamage { p: P0, a: hitter, d: P1, dmg: 3 },
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
        sim_core::engine::hooks::Call::CombatDamage { p: P1, a: hitter, d: P0, dmg: 3 },
    )
    .unwrap();
    assert!(tapped(&g, dork));
}
