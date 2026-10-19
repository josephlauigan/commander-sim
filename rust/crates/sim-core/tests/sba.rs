//! State-based actions (engine/life.rs `check_state`): toughness 0 or less at every check, and the decked loss.

use sim_core::engine::life::check_state;
use sim_core::engine::zones;
use sim_core::ids::PlayerId;
use sim_core::state::DataKey;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn toughness_zero_after_entering_dies_at_the_next_check() {
    let mut g = table(&["sauron", "veyran"]);
    let a = token(&mut g, P1, 3); // a 3/3
    check_state(&mut g).unwrap();
    assert!(g.perm(a).on_bf, "3 toughness lives");
    g.perm_mut(a).eot_pt = (0, -3);
    check_state(&mut g).unwrap();
    assert!(!g.perm(a).on_bf, "toughness 0 dies as a state-based action");
}

#[test]
fn indestructible_and_regeneration_do_not_save_a_zero_toughness_creature() {
    let mut g = table(&["sauron", "veyran"]);
    let a = token(&mut g, P1, 3);
    g.perm_mut(a).data.set(DataKey::Indestr, sim_core::state::Val::Int(1));
    g.player_mut(P1).regen_turn = Some(g.turn_stamp());
    g.perm_mut(a).eot_pt = (0, -3);
    check_state(&mut g).unwrap();
    assert!(!g.perm(a).on_bf);
}

#[test]
fn dismember_kills_an_indestructible_godo() {
    let mut g = table(&["sauron", "godo"]);
    let godo = perm(&mut g, P1, "Godo, Bandit Warlord"); // 3/3
    g.perm_mut(godo).data.set(DataKey::Indestr, sim_core::state::Val::Int(1)); // Hammer of Nazahn
    g.perm_mut(godo).eot_pt = (-5, -5); // Dismember
    check_state(&mut g).unwrap();
    assert!(!g.perm(godo).on_bf, "-5/-5 is not destruction");
}

#[test]
fn a_draw_from_an_empty_library_loses_at_the_next_check_not_before() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    g.player_mut(P0).library.clear();
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.player(P0).alive && g.player(P0).decked, "flagged, not yet out");
    check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive);
}

#[test]
fn an_empty_library_alone_does_not_lose() {
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).library.clear();
    check_state(&mut g).unwrap();
    assert!(g.player(P0).alive, "only an attempted draw loses");
}
