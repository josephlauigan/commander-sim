//! Tier 6 "stax" batch (impls/t6_stax.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::engine::mana::cost_of;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["dack-fayden-white-stax", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox", "godo"])
}

// ======================================================== Disruptor Flute
#[test]
fn flute_taxes_the_named_card_by_three_for_everyone_and_stops_its_abilities() {
    let mut g = tbl();
    let flute = perm(&mut g, P0, "Disruptor Flute");
    let sol = card(&g, "Sol Ring");
    g.flutes.clear();
    g.flutes.push((flute, card(&g, "Sol Ring")));
    assert_eq!(cost_of(&g, P0, sol).0, 4, "Sol Ring costs {{3}} more for its controller");
    assert_eq!(cost_of(&g, P1, sol).0, 4, "and for opponents");
    assert!(sim_core::engine::values::stopped(&g, "Sol Ring"), "its activated abilities are off");
    assert!(!sim_core::engine::values::stopped(&g, "Mana Vault"));
    g.perm_mut(flute).on_bf = false;
    assert_eq!(cost_of(&g, P1, sol).0, 1, "gone with the Flute");
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    let ctx = sim_core::state::Ctx::default();
    f(g, p, c, &ctx).unwrap();
}

fn can_cast(g: &Game, p: PlayerId, name: &str) -> bool {
    sim_core::engine::cast::castable(g, p, card(g, name), "hand")
}

// ======================================================== Silence
#[test]
fn silence_locks_every_opponent_for_the_turn_but_not_its_caster() {
    let mut g = tbl();
    g.active = Some(P0);
    assert!(can_cast(&g, P1, "Lightning Bolt"));
    resolve(&mut g, P0, "Silence");
    assert!(sim_core::impls::t6_stax::cast_lock(&g, P1));
    assert!(sim_core::impls::t6_stax::cast_lock(&g, PlayerId(2)));
    assert!(!sim_core::impls::t6_stax::cast_lock(&g, P0), "your opponents, not you");
    assert!(!can_cast(&g, P1, "Lightning Bolt"), "an opponent's instant is refused");
    assert!(can_cast(&g, P0, "Lightning Bolt"));
}

#[test]
fn silence_ends_with_the_turn() {
    let mut g = tbl();
    g.active = Some(P0);
    resolve(&mut g, P0, "Silence");
    g.player_mut(P0).turns += 1; // the next turn of the same player
    assert!(can_cast(&g, P1, "Lightning Bolt"), "next turn it is over");
}

// ======================================================== Orim's Chant
fn resolve_at(g: &mut Game, p: PlayerId, name: &str, ctx: &sim_core::state::Ctx) {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, p, c, ctx).unwrap();
}

#[test]
fn chant_locks_only_its_target() {
    let mut g = tbl();
    g.active = Some(P0);
    let ctx = sim_core::state::Ctx { face: Some(P1), ..Default::default() };
    resolve_at(&mut g, P0, "Orim's Chant", &ctx);
    assert!(!can_cast(&g, P1, "Lightning Bolt"), "the target can't cast");
    assert!(can_cast(&g, PlayerId(2), "Lightning Bolt"), "the other opponents can");
    assert!(can_cast(&g, P0, "Lightning Bolt"));
    assert!(!sim_core::impls::t6_stax::attack_lock(&g), "unkicked: attacks are allowed");
}

#[test]
fn kicked_chant_also_stops_attacks() {
    let mut g = tbl();
    g.active = Some(P0);
    let ctx = sim_core::state::Ctx { face: Some(P1), kicks: 1, ..Default::default() };
    resolve_at(&mut g, P0, "Orim's Chant", &ctx);
    assert!(sim_core::impls::t6_stax::attack_lock(&g));
}

#[test]
fn chant_without_a_target_picks_the_opponent_with_most_cards() {
    let mut g = tbl();
    g.active = Some(P0);
    hand(&mut g, PlayerId(2), &["Sol Ring", "Mana Vault", "Lightning Bolt"]);
    resolve_at(&mut g, P0, "Orim's Chant", &Default::default());
    assert!(sim_core::impls::t6_stax::cast_lock(&g, PlayerId(2)));
}
