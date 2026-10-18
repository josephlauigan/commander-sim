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

// ======================================================== Mandate of Peace
#[test]
fn mandate_locks_opponents_and_ends_combat_damage() {
    let mut g = tbl();
    g.active = Some(P0);
    let atk = token(&mut g, P0, 3);
    g.in_combat = vec![atk];
    assert!(sim_core::impls::t6_stax::in_combat_now(&g));
    resolve(&mut g, P0, "Mandate of Peace");
    assert!(sim_core::impls::t6_stax::cast_lock(&g, P1));
    assert!(sim_core::impls::t6_stax::cast_lock(&g, PlayerId(2)));
    assert!(!sim_core::impls::t6_stax::cast_lock(&g, P0));
    assert!(!can_cast(&g, P1, "Lightning Bolt"));
    assert_eq!(g.fog, Some(g.turn_stamp()), "no combat damage the rest of the turn");
}

#[test]
fn mandate_is_not_castable_outside_combat() {
    let g = tbl();
    assert!(!sim_core::impls::t6_stax::in_combat_now(&g));
}

// ======================================================== Angel's Grace
#[test]
fn grace_sets_the_turn_flag_and_the_life_floor_for_its_caster_only() {
    let mut g = tbl();
    g.active = Some(P0);
    assert_eq!(sim_core::impls::t6_stax::grace_floor(&g, P0, -3), -3);
    resolve(&mut g, P0, "Angel's Grace");
    assert!(sim_core::impls::t6_stax::grace_active(&g, P0));
    assert!(!sim_core::impls::t6_stax::grace_active(&g, P1));
    assert_eq!(sim_core::impls::t6_stax::grace_floor(&g, P0, -3), 1, "damage leaves you at 1");
    assert_eq!(sim_core::impls::t6_stax::grace_floor(&g, P0, 12), 12, "above 1 is unchanged");
    assert_eq!(sim_core::impls::t6_stax::grace_floor(&g, P1, -3), -3);
}

// ======================================================== Blind Obedience
fn tapped(g: &Game, m: sim_core::ids::PermId) -> bool {
    g.perm(m).tapped
}

#[test]
fn obedience_taps_opposing_creatures_and_artifacts_as_they_enter_but_not_its_own() {
    let mut g = tbl();
    perm(&mut g, P0, "Blind Obedience");
    let theirs = token(&mut g, P1, 2);
    assert!(tapped(&g, theirs), "an opposing creature token enters tapped");
    let art = perm(&mut g, P1, "Sol Ring");
    assert!(tapped(&g, art), "an opposing artifact enters tapped");
    let own = token(&mut g, P0, 2);
    assert!(!tapped(&g, own), "its controller's own permanents are untouched");
    let own_art = perm(&mut g, P0, "Mana Vault");
    assert!(!tapped(&g, own_art));
}

#[test]
fn obedience_leaves_opposing_lands_and_other_permanents_alone() {
    let mut g = tbl();
    perm(&mut g, P0, "Blind Obedience");
    let ench = perm(&mut g, P1, "Rhystic Study");
    assert!(!tapped(&g, ench), "enchantments are not affected");
}

#[test]
fn obedience_extort_drains_each_opponent_when_its_controller_casts_with_spare_mana() {
    let mut g = tbl();
    let ob = perm(&mut g, P0, "Blind Obedience");
    lands(&mut g, P0, "Plains", 4, false);
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    let c = card(&g, "Sol Ring");
    let f = g.registry.get(card(&g, "Blind Obedience")).unwrap().cast.unwrap();
    f(&mut g, ob, P0, c).unwrap();
    assert_eq!(g.player(P1).life, l1 - 1);
    assert_eq!(g.player(PlayerId(2)).life, g.player(PlayerId(3)).life);
    assert_eq!(g.player(P0).life, l0 + 3, "gains what the three opponents lost");
}

#[test]
fn obedience_extort_does_not_trigger_on_an_opponents_spell() {
    let mut g = tbl();
    let ob = perm(&mut g, P0, "Blind Obedience");
    lands(&mut g, P0, "Plains", 4, false);
    let l1 = g.player(P1).life;
    let f = g.registry.get(card(&g, "Blind Obedience")).unwrap().cast.unwrap();
    let sol = card(&g, "Sol Ring");
    f(&mut g, ob, P1, sol).unwrap();
    assert_eq!(g.player(P1).life, l1);
}

// ======================================================== Damping Sphere
use sim_core::engine::mana::total_mana;

#[test]
fn sphere_makes_two_mana_lands_produce_one() {
    let mut g = tbl();
    lands(&mut g, P1, "Ancient Tomb", 2, false);
    lands(&mut g, P1, "Plains", 1, false);
    assert_eq!(total_mana(&g, P1, false), 5, "without the Sphere: 2 + 2 + 1");
    perm(&mut g, P0, "Damping Sphere");
    assert_eq!(total_mana(&g, P1, false), 3, "with it, every land makes one mana");
    perm(&mut g, P0, "Damping Sphere");
    assert_eq!(total_mana(&g, P1, false), 3, "a second Sphere changes nothing more");
}

#[test]
fn sphere_taxes_each_spell_by_the_spells_already_cast_this_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Damping Sphere");
    let sol = card(&g, "Sol Ring");
    assert_eq!(cost_of(&g, P1, sol).0, 1, "the first spell is not taxed");
    let st = g.turn_stamp();
    g.player_mut(P1).turn_casts = Some((st, vec![card(&g, "Mana Vault")]));
    assert_eq!(cost_of(&g, P1, sol).0, 2, "one other spell: {{1}} more");
    g.player_mut(P1).turn_casts = Some((st, vec![card(&g, "Mana Vault"), card(&g, "Lightning Bolt")]));
    assert_eq!(cost_of(&g, P1, sol).0, 3, "two other spells: {{2}} more");
    assert_eq!(cost_of(&g, P0, sol).0, 1, "counted per player");
    g.player_mut(P0).turn_casts = Some((st, vec![card(&g, "Mana Vault")]));
    assert_eq!(cost_of(&g, P0, sol).0, 2, "its own controller pays too");
}

// ======================================================== Pithing Needle
#[test]
fn needle_names_the_opposing_engine_and_stops_only_that_name() {
    let mut g = tbl();
    perm(&mut g, P1, "Tezzeret, Cruel Captain");
    let (want, _) = sim_core::ai::decks::flute_pick(&g, P0).expect("something worth naming");
    let needle = perm(&mut g, P0, "Pithing Needle");
    let named = sim_core::impls::t6_stax::needle_name(&g, needle).expect("it names a card as it enters");
    assert_eq!(named, want, "it names what Disruptor Flute would");
    let name = g.db.get(named).name.to_string();
    assert!(sim_core::impls::t6_stax::needle_stops(&g, &name));
    assert!(!sim_core::impls::t6_stax::needle_stops(&g, "Sol Ring"));
    g.perm_mut(needle).on_bf = false;
    assert!(!sim_core::impls::t6_stax::needle_stops(&g, &name), "gone with the Needle");
}

// ======================================================== Chalice of the Void
#[test]
fn chalice_enters_with_one_counter_and_costs_two() {
    let mut g = tbl();
    let ch = card(&g, "Chalice of the Void");
    assert_eq!(cost_of(&g, P0, ch).0, 2, "{{X}}{{X}} with X = 1");
    let m = perm(&mut g, P0, "Chalice of the Void");
    assert_eq!(sim_core::impls::t6_stax::chalice_counters(&g, m), 1);
}

#[test]
fn chalice_stops_spells_with_mana_value_equal_to_its_counters_for_everyone() {
    let mut g = tbl();
    assert!(can_cast(&g, P1, "Sol Ring"));
    let m = perm(&mut g, P0, "Chalice of the Void");
    assert!(!can_cast(&g, P1, "Sol Ring"), "mana value 1 is countered");
    assert!(!can_cast(&g, P0, "Sol Ring"), "its controller's too");
    assert!(!can_cast(&g, P1, "Lightning Bolt"), "any mana value 1 spell");
    let two = card(&g, "Godo, Bandit Warlord");
    assert!(sim_core::engine::cast::castable(&g, P1, two, "hand"), "mana value 6 is fine");
    g.perm_mut(m).on_bf = false;
    assert!(can_cast(&g, P1, "Sol Ring"), "gone with the Chalice");
}
