//! Independent adversarial tests (adv2) for Dockside Extortionist (impls/dockside.rs) and the Mox Opal metalcraft change
//! (impls/rules.rs `moxopal` -> karn::artifacts). Expected values come from the Oracle text and the comprehensive rules.

mod play_common;
use play_common::*;
use sim_core::cards::Types;
use sim_core::engine::mana::{self, total_mana};
use sim_core::engine::zones;
use sim_core::impls::dockside::opposing_artifacts_and_enchantments as x_of;
use sim_core::state::CopyExc;

const DOCKSIDE: &str = "Dockside Extortionist";

fn tbl() -> Game {
    table(&["arzakon", "yshtola"])
}

fn treasures(g: &Game, p: PlayerId) -> u32 {
    g.player(p).treasures
}

// ------------------------------------------------------------------ Dockside
#[test]
fn adv2_enchantment_land_counts() {
    let mut g = tbl();
    lands(&mut g, P1, "Urza's Saga", 1, false);
    assert_eq!(x_of(&g, P0), 1, "Urza's Saga is an enchantment land");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
}

#[test]
fn adv2_teferis_protection_phases_out_the_opponents_everything() {
    // CR 702.26: a phased-out permanent is treated as though it does not exist. Teferi's Protection phases out all of
    // the caster's permanents, including its Treasure tokens and artifact lands, until its next turn.
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P1, "Darksteel Citadel", 1, false);
    zones::add_treasure(&mut g, P1, 2).unwrap();
    zones::teferis_protection(&mut g, P1);
    assert!(g.perm(perm_named(&g, P1, "Sol Ring")).phased, "the engine phases the permanents");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(
        treasures(&g, P0),
        0,
        "everything the opponent has is phased out: X = 0 (Sol Ring, Citadel, 2 Treasures)"
    );
}

fn perm_named(g: &Game, p: PlayerId, n: &str) -> PermId {
    perms_named(g, p, n)[0]
}

#[test]
fn adv2_own_elesh_norn_adds_a_second_trigger() {
    // Elesh Norn, Mother of Machines: "Permanents entering the battlefield cause abilities of permanents you control to
    // trigger an additional time." Dockside's controller's own Elesh: two triggers, each counting the one Sol Ring.
    let mut g = tbl();
    perm(&mut g, P0, "Elesh Norn, Mother of Machines");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 2);
}

#[test]
fn adv2_hushbringer_blanks_the_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, "Hushbringer");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 0, "creatures entering don't cause abilities to trigger");
}

#[test]
fn adv2_blink_counts_again() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    let d = perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
    sim_core::impls::t2::blink(&mut g, P0, d).unwrap();
    assert_eq!(treasures(&g, P0), 2, "Ephemerate: it enters again");
    assert_eq!(perms_named(&g, P0, DOCKSIDE).len(), 1);
}

#[test]
fn adv2_a_stolen_dockside_counts_for_the_new_controllers_opponents() {
    // P1 controls P0's Dockside when it enters: X counts P0's artifacts (P1's opponents), not P1's own
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P1, DOCKSIDE);
    assert_eq!(treasures(&g, P1), 1);
    assert_eq!(treasures(&g, P0), 0);
}

// ------------------------------------------------------------------ Mox Opal
fn mana_of(g: &Game, p: PlayerId) -> u32 {
    total_mana(g, p, false)
}

#[test]
fn adv2_opal_with_two_artifacts_and_a_phased_third_is_off() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Sol Ring");
    let s = perm(&mut g, P0, "Arcane Signet");
    g.perm_mut(s).phased = true;
    assert_eq!(mana_of(&g, P0), 2, "Sol Ring only: a phased-out artifact is not counted");
}

#[test]
fn adv2_opal_sol_ring_and_a_treasure_is_on() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(mana_of(&g, P0), 2);
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(mana_of(&g, P0), 2 + 1 + 1, "Sol Ring, the Treasure, Opal (Opal + Sol + Treasure = 3)");
}

#[test]
fn adv2_opal_alone_with_one_treasure_is_off() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(mana_of(&g, P0), 1, "Opal + Treasure = two artifacts: only the Treasure's mana");
}

#[test]
fn adv2_opal_with_an_artifact_land_and_one_artifact() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Sol Ring");
    lands(&mut g, P0, "Seat of the Synod", 1, false);
    // Seat 1 (U) + Sol Ring 2 + Opal 1 (Opal, Sol Ring, Seat = three artifacts)
    assert_eq!(mana_of(&g, P0), 1 + 2 + 1);
}

#[test]
fn adv2_opal_with_a_food_and_a_clue() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    zones::make_artifact_tokens(&mut g, P0, "Food", 1).unwrap();
    assert_eq!(mana_of(&g, P0), 0, "Opal + Food: two artifacts, no mana");
    zones::make_artifact_tokens(&mut g, P0, "Clue", 1).unwrap();
    assert_eq!(mana_of(&g, P0), 1, "Opal + Food + Clue: metalcraft, Opal makes one");
}

#[test]
fn adv2_opal_with_an_artifact_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Academy Manufactor");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(mana_of(&g, P0), 2 + 1);
}

#[test]
fn adv2_opal_paying_with_the_treasure_in_the_same_payment() {
    // Opal + Sol Ring + Treasure: {2}{U}{U} = Opal U, Treasure U, Sol Ring {2}. Opal is activated before the Treasure
    // is sacrificed, which the rules allow (601.2g: mana abilities first).
    let mut g = tbl();
    let opal = perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Sol Ring");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert!(mana::can_pay(&g, P0, 2, "UU", false));
    assert!(mana::pay(&mut g, P0, 2, "UU", false).unwrap());
    assert_eq!(treasures(&g, P0), 0, "the Treasure was spent");
    assert!(g.perm(opal).tapped, "and Opal's mana was used");
    assert_eq!(mana_of(&g, P0), 0);
}

#[test]
fn adv2_opal_and_one_animated_artifact_land_are_two_artifacts() {
    // Astral Dragon: a token copy of an artifact land that is also a creature (listed in `lands` and in `perms`).
    // Opal + that one land are TWO artifacts (the land token is one permanent): no metalcraft.
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    let cit = card(&g, "Darksteel Citadel");
    let exc = CopyExc { add_types: Types::CREATURE, pt: Some((3, 3)), ..CopyExc::default() };
    let l = sim_core::engine::turn::enter_token_land_copy(&mut g, P0, cit, exc).unwrap().unwrap();
    g.perm_mut(l).sick = false; // so its own {T}: Add {C} is available
    assert!(g.player(P0).lands.contains(&l) && g.player(P0).perms.contains(&l), "listed in both");
    assert_eq!(mana_of(&g, P0), 1, "the Citadel's {{C}} only: Opal is off with two artifacts (two permanents)");
}

#[test]
fn adv2_karn_artifacts_counts_a_token_land_copy_once() {
    let mut g = tbl();
    let cit = card(&g, "Darksteel Citadel");
    let exc = CopyExc { add_types: Types::CREATURE, pt: Some((3, 3)), ..CopyExc::default() };
    sim_core::engine::turn::enter_token_land_copy(&mut g, P0, cit, exc).unwrap().unwrap();
    assert_eq!(sim_core::impls::karn::artifacts(&g, P0), 1, "one permanent, one artifact");
}

#[test]
fn adv2_other_karn_artifacts_users_unchanged_by_the_diff() {
    // moxopal's diff touched only rules.rs; karn::artifacts itself is the same function. A sanity value check.
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    lands(&mut g, P0, "Darksteel Citadel", 1, false);
    assert_eq!(sim_core::impls::karn::artifacts(&g, P0), 3);
}
