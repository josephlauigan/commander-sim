//! Scion of the Ur-Dragon's dragon line for the `scion` deck (impls/t6_scion.rs, impls/t6_combos_scion.rs): the copy
//! and its end, the lost ability, stacked activations, Prismari's storm and ward, the copied Dragons' card code, and
//! the lines the seat finds.

use sim_core::engine::cast::cast_card;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::impls::t6_scion::{self, COPPER, KORVOLD, PRISMARI, SCION};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn in_lib(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).library.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// Scion on the battlefield (not summoning sick) with `mana` untapped Badlands
fn scion(g: &mut Game, mana: usize) -> PermId {
    lands(g, P0, "Badlands", mana, false);
    let s = perm(g, P0, SCION);
    g.perm_mut(s).is_cmd = true;
    s
}

#[test]
fn scion_becomes_a_copy_and_the_copy_ends_with_the_turn() {
    let mut g = tbl();
    let s = scion(&mut g, 2);
    assert!(in_lib(&g, P0, COPPER));
    assert!(t6_scion::activate(&mut g, s, P0, &[COPPER]).unwrap());
    assert_eq!(&*g.perm(s).name, COPPER);
    assert_eq!((g.perm(s).pow, g.perm(s).tgh), (6, 5));
    assert!(in_gy(&g, P0, COPPER) && !in_lib(&g, P0, COPPER), "the Dragon went from the library to the graveyard");
    assert!(g.perm(s).is_cmd, "still the commander");
    // the next turn begins: the copy ended at the cleanup
    sim_core::engine::turn::step_start(&mut g, P1).unwrap();
    assert_eq!(&*g.perm(s).name, SCION);
    assert_eq!((g.perm(s).pow, g.perm(s).tgh), (4, 4));
    assert!(g.perm(s).phys.is_none());
    assert!(in_gy(&g, P0, COPPER), "the card stays in the graveyard");
}

#[test]
fn a_copy_has_lost_scions_ability() {
    let mut g = tbl();
    let s = scion(&mut g, 4);
    assert!(t6_scion::activate(&mut g, s, P0, &[COPPER]).unwrap());
    assert!(t6_scion::scion_of(&g, P0).is_none(), "no Scion with its ability on the battlefield");
    assert!(!t6_scion::activate(&mut g, s, P0, &[KORVOLD]).unwrap(), "the copy can't activate");
    assert!(in_lib(&g, P0, KORVOLD), "nothing was searched");
    // and the AI offers no activation either: the card code is the copied Dragon's now
    let opts = t6_scion::scion_options(&mut g, s, P0, Some(false)).unwrap();
    assert!(opts.is_empty());
}

#[test]
fn stacked_activations_all_resolve_and_the_last_to_resolve_wins() {
    let mut g = tbl();
    let s = scion(&mut g, 4);
    // two activations on the stack before either resolves: Korvold's resolves first, then Copper's
    assert!(t6_scion::activate(&mut g, s, P0, &[COPPER, KORVOLD]).unwrap());
    assert!(in_gy(&g, P0, COPPER) && in_gy(&g, P0, KORVOLD), "both resolved");
    assert_eq!(&*g.perm(s).name, COPPER, "the newest copy effect wins");
    assert_eq!(g.perm(s).phys, Some(card(&g, SCION)), "Scion's card under both copies");
    sim_core::engine::turn::step_start(&mut g, P1).unwrap();
    assert_eq!(&*g.perm(s).name, SCION);
}

#[test]
fn the_ai_turns_scion_into_copper_dragon_before_combat() {
    let mut g = tbl();
    let s = scion(&mut g, 2);
    let opts = t6_scion::scion_options(&mut g, s, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.contains(COPPER), "{}", opts[0].label);
    // not on another player's turn, not after combat, not while summoning sick
    assert!(t6_scion::scion_options(&mut g, s, P0, Some(true)).unwrap().is_empty());
    g.perm_mut(s).sick = true;
    assert!(t6_scion::scion_options(&mut g, s, P0, Some(false)).unwrap().is_empty());
}

/// spells cast before the next one this turn (the storm count), as if they had been cast
fn cast_before(g: &mut Game, p: PlayerId, n: usize) {
    let c = card(g, "Lotus Petal");
    let st = g.turn_stamp();
    g.player_mut(p).turn_casts = Some((st, vec![c; n]));
}

fn floating(g: &Game, p: PlayerId) -> u32 {
    g.player(p).mana.get(sim_core::engine::pool::Kind::W) + g.player(p).mana.plain_but_white()
}

fn ritual(prismari: bool) -> u32 {
    let mut g = tbl();
    if prismari {
        perm(&mut g, P0, PRISMARI);
    }
    cast_before(&mut g, P0, 2);
    let c = hand(&mut g, P0, &["Dark Ritual"])[0];
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    floating(&g, P0)
}

#[test]
fn prismaris_storm_copies_a_ritual() {
    assert_eq!(ritual(false), 3, "Dark Ritual alone: BBB");
    assert_eq!(ritual(true), 9, "two spells before it: two storm copies, BBB each");
}

fn freeze(prismari: bool) -> usize {
    let mut g = tbl();
    if prismari {
        perm(&mut g, P0, PRISMARI);
    }
    cast_before(&mut g, P0, 2);
    let n0 = g.player(P1).library.len();
    let c = hand(&mut g, P0, &["Brain Freeze"])[0];
    assert!(cast_card(&mut g, P0, c, "hand", Ctx { mill: vec![P1; 20], ..Ctx::default() }).unwrap());
    n0 - g.player(P1).library.len()
}

#[test]
fn brain_freeze_gets_a_second_storm_trigger_from_prismari() {
    assert_eq!(freeze(false), 9, "its own storm: three copies of mill three");
    assert_eq!(freeze(true), 15, "and Prismari's: two more copies");
}

#[test]
fn scion_as_prismari_gives_storm_too() {
    let mut g = tbl();
    let s = scion(&mut g, 2);
    assert!(t6_scion::activate(&mut g, s, P0, &[PRISMARI]).unwrap());
    assert!(t6_scion::prismari_on(&g, P0));
    cast_before(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Dark Ritual"])[0];
    let before = floating(&g, P0);
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(floating(&g, P0) - before, 6);
}

#[test]
fn copper_dragons_treasures_when_scion_is_the_copy() {
    let mut g = tbl();
    let s = scion(&mut g, 2);
    assert!(t6_scion::activate(&mut g, s, P0, &[COPPER]).unwrap());
    let t0 = g.player(P0).treasures;
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[s], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 34, "six flying damage");
    assert!(g.player(P0).treasures > t0, "a d20 of Treasures");
}

#[test]
fn korvolds_draws_when_scion_is_the_copy() {
    let mut g = tbl();
    let s = scion(&mut g, 2);
    assert!(t6_scion::activate(&mut g, s, P0, &[KORVOLD]).unwrap());
    g.player_mut(P0).treasures = 1;
    let h0 = g.player(P0).hand.len();
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[s], P1).unwrap();
    assert_eq!(g.player(P0).treasures, 0, "the attack trigger sacrificed the Treasure");
    assert_eq!(g.player(P0).hand.len(), h0 + 1, "a card for the sacrifice");
    assert_eq!(g.perm(s).plus, 1, "and a +1/+1 counter");
}

#[test]
fn prismaris_ward_costs_five_life() {
    let mut g = tbl();
    let m = perm(&mut g, P0, PRISMARI);
    let spell = card(&g, "Swords to Plowshares");
    g.player_mut(P1).life = 5;
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), m, "exile", Some(spell)).unwrap();
    assert!(g.perm(m).on_bf, "at 5 life the ward can't be paid: the removal is countered");
    g.player_mut(P1).life = 40;
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), m, "exile", Some(spell)).unwrap();
    assert_eq!(g.player(P1).life, 35, "five life paid");
    assert!(!g.perm(m).on_bf, "then the removal resolves");
}

#[test]
fn the_prismari_breach_line_mills_the_table() {
    let mut g = tbl();
    scion(&mut g, 6);
    hand(&mut g, P0, &["Underworld Breach", "Manamorphose", "Brain Freeze"]);
    let i = combos::combo_index("Scion as Prismari + Underworld Breach").unwrap();
    assert!(combos::ready(&g, P0, i), "the line is ready");
    g.log = Some(vec![]);
    assert!(combos::attempt(&mut g, P0, i as i64).unwrap());
    let log = g.log.clone().unwrap().join("\n");
    assert!(g.over && g.winner == Some(P0), "the opponents' libraries are milled:\n{log}");
    for q in 1..4 {
        assert!(g.player(PlayerId(q)).library.is_empty());
    }
    assert!(g.player(P0).alive && !g.player(P0).library.is_empty());
}

#[test]
fn the_prismari_line_needs_scion_with_prismari_in_the_library() {
    let mut g = tbl();
    lands(&mut g, P0, "Badlands", 6, false);
    hand(&mut g, P0, &["Underworld Breach", "Manamorphose", "Brain Freeze"]);
    let i = combos::combo_index("Scion as Prismari + Underworld Breach").unwrap();
    assert!(!combos::ready(&g, P0, i), "no Scion, no Prismari");
}

#[test]
fn the_scion_seat_finds_the_oracle_and_breach_lines() {
    let mut g = tbl();
    lands(&mut g, P0, "Underground Sea", 4, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let i = combos::combo_index("Thassa's Oracle + Demonic Consultation").unwrap();
    assert!(combos::ready(&g, P0, i));
    let mut g = tbl();
    lands(&mut g, P0, "Volcanic Island", 2, false);
    hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Gamble", "Silence", "Swan Song"]);
    let i = combos::combo_index("Underworld Breach + Lion's Eye Diamond + Brain Freeze (at the table)").unwrap();
    assert!(combos::ready(&g, P0, i));
}
