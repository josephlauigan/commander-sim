//! The Tier 3 Atraxa deck's hand-written cards (impls/t3_atraxa.rs): Ezuri, Tainted Observer, Contaminant Grafter,
//! Brokers Ascendancy, Contagion Engine, Cankerbloom, Phyresis Outbreak, Drown in Ichor.

use sim_core::ai::brain::do_cast;
use sim_core::engine::turn;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["atraxa-superfriends", "seph"]);
    g.active = Some(P0);
    g
}

fn cast(g: &mut Game, name: &str) {
    hand(g, P0, &[name]);
    let c = card(g, name);
    assert!(do_cast(g, P0, c, None).unwrap(), "{name} is cast");
}

#[test]
fn ezuri_pays_three_to_proliferate_twice_and_draws_for_each() {
    let mut g = tbl();
    g.player_mut(P1).poison = 2;
    lands(&mut g, P0, "Forest", 4, false);
    lands(&mut g, P0, "Island", 3, false);
    let n = g.player(P0).hand.len();
    cast(&mut g, "Ezuri, Stalker of Spheres");
    assert_eq!(g.player(P1).poison, 4, "proliferate twice");
    assert_eq!(g.player(P0).hand.len(), n + 2, "Ezuri draws for each proliferate");
}

#[test]
fn tainted_observer_proliferates_when_another_creature_enters() {
    let mut g = tbl();
    g.player_mut(P1).poison = 1;
    perm(&mut g, P0, "Tainted Observer");
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Blightbelly Rat");
    assert_eq!(g.player(P1).poison, 2);
}

#[test]
fn contaminant_grafter_draws_and_puts_a_land_down_at_its_corrupted_end_step() {
    let mut g = tbl();
    perm(&mut g, P0, "Contaminant Grafter");
    g.player_mut(P1).poison = 3;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Forest"]);
    let lands_before = g.player(P0).lands.len();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).lands.len(), lands_before + 1, "the Forest from hand");
    assert_eq!(g.player(P0).hand.len(), 1, "one card drawn");
}

#[test]
fn brokers_ascendancy_puts_counters_at_your_end_step() {
    let mut g = tbl();
    perm(&mut g, P0, "Brokers Ascendancy");
    let t = token(&mut g, P0, 1);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(t).plus, 1);
}

#[test]
fn contagion_engine_shrinks_the_creatures_of_the_opponent_it_hurts_most() {
    let mut g = tbl();
    let small = token(&mut g, P1, 1);
    let big = token(&mut g, P1, 3);
    perm(&mut g, P0, "Contagion Engine");
    assert!(!g.perm(small).on_bf, "the 1/1 dies to its -1/-1 counter");
    assert_eq!(g.perm(big).plus, -1);
}

#[test]
fn phyresis_outbreak_poisons_then_kills_by_poison_count() {
    let mut g = tbl();
    g.player_mut(P1).poison = 1;
    let two = token(&mut g, P1, 2);
    let three = token(&mut g, P1, 3);
    lands(&mut g, P0, "Swamp", 3, false);
    cast(&mut g, "Phyresis Outbreak");
    assert_eq!(g.player(P1).poison, 2);
    assert!(!g.perm(two).on_bf, "-2/-2 kills the 2/2");
    assert!(g.perm(three).on_bf);
}

#[test]
fn drown_in_ichor_kills_and_proliferates() {
    let mut g = tbl();
    g.player_mut(P1).poison = 1;
    let t = token(&mut g, P1, 4);
    lands(&mut g, P0, "Swamp", 2, false);
    cast(&mut g, "Drown in Ichor");
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).poison, 2);
}

#[test]
fn cankerbloom_sacrifices_to_proliferate() {
    let mut g = tbl();
    g.player_mut(P1).poison = 2;
    let cb = perm(&mut g, P0, "Cankerbloom");
    lands(&mut g, P0, "Forest", 1, false);
    let cd = g.perm(cb).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).unwrap();
    let o = f(&mut g, cb, P0, Some(false)).unwrap();
    let act = o.into_iter().find_map(|o| o.act).expect("the sacrifice option");
    assert!(sim_core::ai::act::perform(&mut g, P0, &act).unwrap());
    assert!(!g.perm(cb).on_bf);
    assert_eq!(g.player(P1).poison, 3);
}

#[test]
fn infectious_bite_kills_with_your_biggest_creature_and_poisons() {
    let mut g = tbl();
    token(&mut g, P0, 4);
    let t = token(&mut g, P1, 3);
    lands(&mut g, P0, "Forest", 2, false);
    cast(&mut g, "Infectious Bite");
    assert!(!g.perm(t).on_bf, "4 damage kills the 3/3");
    assert_eq!(g.player(P1).poison, 1);
}
