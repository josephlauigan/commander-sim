//! The counter-on-cast hook (`Event::CastCounter`, engine/cast.rs `cast_card`) and the record of the mana spent on a
//! cast (`Ctx::mana_spent`, `Ctx::mana_colored`): Vexing Bauble, Chalice of the Void, Void Mirror.

use sim_core::engine::mana::{cost_of, pay};
use sim_core::engine::cast;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["dack-fayden-white-stax", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox", "godo"]);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// p pays the card's cost from its lands and casts it from hand
fn cast_paid(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    let (gn, pips) = cost_of(g, p, c);
    assert!(pay(g, p, gn, &pips, false).unwrap(), "{name} is paid");
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

/// p casts the card without paying anything (a Mox, a free alternative cost)
fn cast_free(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn chalice(g: &mut Game, p: PlayerId, x: i64) {
    let m = perm(g, p, "Chalice of the Void");
    g.perm_mut(m).data.set(sim_core::state::DataKey::Charge, sim_core::state::Val::Int(x));
}

// ======================================================== the mana spent record
#[test]
fn the_cast_records_the_mana_spent_and_whether_any_was_coloured() {
    let mut g = tbl();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    let (gn, pips) = cost_of(&g, P0, card(&g, "Sol Ring"));
    assert!(pay(&mut g, P0, gn, &pips, false).unwrap());
    assert_eq!((g.mana_spent, g.mana_colored), (1, false), "one of Ancient Tomb's {{C}}{{C}} pays {{1}}: colourless");
    let mut g = tbl();
    lands(&mut g, P0, "Island", 1, false);
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!((g.mana_spent, g.mana_colored), (1, true), "an Island's mana is coloured even for a generic cost");
}

#[test]
fn a_free_cast_spends_no_mana_and_an_alternative_cost_keeps_only_the_mana_it_spends() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    assert!(pay(&mut g, P0, 2, "", false).unwrap()); // an earlier payment (an ability) leaves a record
    assert_eq!(g.mana_spent, 2);
    let c = hand(&mut g, P0, &["Mox Amber"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(pay(&mut g, P0, gn, &pips, false).unwrap());
    assert_eq!(g.mana_spent, 0, "paying {{0}} starts the record over");
}

// ======================================================== Vexing Bauble
#[test]
fn bauble_counters_a_free_spell_it_was_cast_and_goes_to_the_graveyard() {
    let mut g = tbl();
    perm(&mut g, P1, "Vexing Bauble");
    let before = g.player(P0).spells_this_turn;
    cast_free(&mut g, P0, "Mox Amber");
    assert!(!on_bf(&g, P0, "Mox Amber"), "countered");
    assert!(in_gy(&g, P0, "Mox Amber"), "in its owner's graveyard");
    assert_eq!(g.player(P0).spells_this_turn, before + 1, "it was cast: cast triggers and the storm count happened");
}

#[test]
fn bauble_counters_its_controllers_own_free_spells_too() {
    let mut g = tbl();
    perm(&mut g, P0, "Vexing Bauble");
    cast_free(&mut g, P0, "Lotus Petal");
    assert!(in_gy(&g, P0, "Lotus Petal") && !on_bf(&g, P0, "Lotus Petal"));
}

#[test]
fn bauble_lets_a_spell_paid_with_mana_resolve_and_ignores_lands() {
    let mut g = tbl();
    perm(&mut g, P1, "Vexing Bauble");
    lands(&mut g, P0, "Island", 1, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(on_bf(&g, P0, "Sol Ring"), "mana was spent");
}

#[test]
fn bauble_counters_a_free_alternative_cost_cast() {
    let mut g = tbl();
    perm(&mut g, P1, "Vexing Bauble");
    // an alternative cost that spends no mana: the pay record is zero
    let c = hand(&mut g, P0, &["Snuff Out"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { paid_otherwise: true, ..Ctx::default() }).unwrap();
    assert!(in_gy(&g, P0, "Snuff Out"));
}

// ======================================================== Chalice of the Void
#[test]
fn chalice_counters_a_spell_with_mana_value_equal_to_its_counters() {
    let mut g = tbl();
    chalice(&mut g, P1, 1);
    lands(&mut g, P0, "Island", 1, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(!on_bf(&g, P0, "Sol Ring") && in_gy(&g, P0, "Sol Ring"), "mana value 1 with one counter");
}

#[test]
fn chalice_at_two_counters_spares_mana_value_one_and_counters_two() {
    let mut g = tbl();
    chalice(&mut g, P1, 2);
    lands(&mut g, P0, "Island", 3, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(on_bf(&g, P0, "Sol Ring"), "mana value 1 is fine at X = 2");
    cast_paid(&mut g, P0, "Mind Stone");
    assert!(in_gy(&g, P0, "Mind Stone"), "mana value 2 is countered");
}

#[test]
fn chalice_at_zero_counters_the_free_spells_only() {
    let mut g = tbl();
    chalice(&mut g, P1, 0);
    cast_free(&mut g, P0, "Mox Amber");
    assert!(in_gy(&g, P0, "Mox Amber"));
    lands(&mut g, P0, "Island", 1, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(on_bf(&g, P0, "Sol Ring"));
}

#[test]
fn chalice_enters_with_half_the_mana_paid_and_costs_twice_the_x_the_ai_chose() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 6, false);
    let c = card(&g, "Chalice of the Void");
    let x = sim_core::impls::t6_stax::chalice_x(&g, P0).expect("six lands pay for it");
    assert!((0..=3).contains(&x));
    assert_eq!(cost_of(&g, P0, c).0 as i32, 2 * x);
    cast_paid(&mut g, P0, "Chalice of the Void");
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    assert_eq!(sim_core::impls::t6_stax::chalice_counters(&g, m), x, "X counters for {{X}}{{X}}");
}

#[test]
fn chalice_with_one_land_is_not_cast() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 1, false);
    assert_eq!(sim_core::impls::t6_stax::chalice_x(&g, P0), None);
    let c = hand(&mut g, P0, &["Chalice of the Void"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(!sim_core::engine::mana::can_pay(&g, P0, gn, &pips, false), "{{X}}{{X}} with X = 1 needs two mana");
}

// ======================================================== Void Mirror
#[test]
fn mirror_counters_a_spell_paid_with_colourless_mana_only() {
    let mut g = tbl();
    perm(&mut g, P1, "Void Mirror");
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(in_gy(&g, P0, "Sol Ring"), "no coloured mana was spent");
}

#[test]
fn mirror_counters_free_spells_and_lets_coloured_payments_resolve() {
    let mut g = tbl();
    perm(&mut g, P1, "Void Mirror");
    cast_free(&mut g, P0, "Mox Amber");
    assert!(in_gy(&g, P0, "Mox Amber"));
    lands(&mut g, P0, "Plains", 1, false);
    cast_paid(&mut g, P0, "Sol Ring");
    assert!(on_bf(&g, P0, "Sol Ring"), "a Plains paid for it");
}

// ======================================================== the AI does not cast into a cast-counter
#[test]
fn the_ai_does_not_cast_a_mox_into_a_bauble_but_casts_sol_ring() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 1, false);
    let mox = hand(&mut g, P0, &["Mox Amber"])[0];
    let sol = hand(&mut g, P0, &["Sol Ring"])[0];
    assert!(cast::castable(&g, P0, mox, "hand") && cast::castable(&g, P0, sol, "hand"));
    perm(&mut g, P1, "Vexing Bauble");
    assert!(!cast::castable(&g, P0, mox, "hand"), "it would be countered as it is cast");
    assert!(cast::castable(&g, P0, sol, "hand"), "mana is spent on Sol Ring");
}
