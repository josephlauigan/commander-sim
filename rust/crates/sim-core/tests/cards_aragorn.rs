//! Aragorn, the Uniter (impls/aragorn.rs) against its Oracle text:
//! "Whenever you cast a white spell, create a 1/1 white Human Soldier creature token.
//!  Whenever you cast a blue spell, scry 2.
//!  Whenever you cast a red spell, Aragorn deals 3 damage to target opponent.
//!  Whenever you cast a green spell, target creature gets +4/+4 until end of turn."

use sim_core::engine::cast;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ARAGORN: &str = "Aragorn, the Uniter";

fn soldiers(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token && g.is_creature(m)).count()
}

/// Aragorn on P0's battlefield, ready to attack, at a two-seat table
fn setup() -> (Game, PermId) {
    let mut g = table(&["aragorn", "cloud"]);
    let a = perm(&mut g, P0, ARAGORN);
    g.perm_mut(a).sick = false;
    (g, a)
}

fn cast_spell(g: &mut Game, name: &str) {
    let c = card(g, name);
    cast::on_cast(g, P0, c).unwrap();
}

#[test]
fn a_white_spell_makes_a_1_1_white_human_soldier() {
    let (mut g, _) = setup();
    let life = g.player(P1).life;
    cast_spell(&mut g, "Swords to Plowshares");
    assert_eq!(soldiers(&g, P0), 1);
    let t = *g.player(P0).perms.iter().find(|&&m| g.perm(m).token).unwrap();
    assert_eq!((g.perm(t).pow, g.perm(t).tgh), (1, 1));
    assert_eq!(g.player(P1).life, life, "white only: no damage");
}

#[test]
fn a_red_spell_deals_3_damage_to_an_opponent() {
    let (mut g, _) = setup();
    let life = g.player(P1).life;
    cast_spell(&mut g, "Lightning Bolt");
    assert_eq!(g.player(P1).life, life - 3);
    assert_eq!(soldiers(&g, P0), 0);
    assert!(g.player(P0).milestone.contains_key("aragorn"));
}

#[test]
fn the_ai_aims_the_damage_at_the_lowest_life_total() {
    let mut g = table(&["aragorn", "cloud", "sheoldred"]);
    let a = perm(&mut g, P0, ARAGORN);
    g.perm_mut(a).sick = false;
    g.player_mut(PlayerId(2)).life = 5;
    let l1 = g.player(P1).life;
    cast_spell(&mut g, "Lightning Bolt");
    assert_eq!((g.player(P1).life, g.player(PlayerId(2)).life), (l1, 2));
}

#[test]
fn a_green_spell_pumps_a_creature_4_4_until_end_of_turn() {
    let (mut g, a) = setup();
    cast_spell(&mut g, "Nature's Lore");
    assert_eq!(g.perm(a).eot_pt, (4, 4), "the AI pumps its own ready attacker, Aragorn");
}

#[test]
fn a_blue_spell_scries_and_does_nothing_else() {
    let (mut g, a) = setup();
    let (life, lib) = (g.player(P1).life, g.player(P0).library.len());
    cast_spell(&mut g, "Counterspell");
    assert_eq!(g.player(P0).library.len(), lib, "scry moves cards within the library (or none to the bottom)");
    assert_eq!((g.player(P1).life, soldiers(&g, P0), g.perm(a).eot_pt), (life, 0, (0, 0)));
}

#[test]
fn a_multicolour_spell_triggers_each_of_its_colours() {
    let (mut g, _) = setup();
    let life = g.player(P1).life;
    cast_spell(&mut g, "Expressive Iteration"); // {U}{R}: scry 2 and 3 damage
    assert_eq!(g.player(P1).life, life - 3);
    assert_eq!(soldiers(&g, P0), 0);
}

#[test]
fn a_colourless_spell_triggers_nothing() {
    let (mut g, a) = setup();
    let life = g.player(P1).life;
    cast_spell(&mut g, "Sol Ring");
    assert_eq!((g.player(P1).life, soldiers(&g, P0), g.perm(a).eot_pt), (life, 0, (0, 0)));
}

#[test]
fn veyran_doubles_the_triggers_of_instants_and_sorceries_only() {
    let (mut g, _) = setup();
    perm(&mut g, P0, "Veyran, Voice of Duality");
    let life = g.player(P1).life;
    cast_spell(&mut g, "Lightning Bolt");
    assert_eq!(g.player(P1).life, life - 6, "an instant: twice");
    cast_spell(&mut g, "Guttersnipe"); // a red creature spell: once
    assert_eq!(g.player(P1).life, life - 9);
}

#[test]
fn fiery_emancipation_triples_the_damage() {
    let (mut g, _) = setup();
    perm(&mut g, P0, "Fiery Emancipation");
    let life = g.player(P1).life;
    cast_spell(&mut g, "Lightning Bolt");
    assert_eq!(g.player(P1).life, life - 9);
}

#[test]
fn an_opponents_spell_or_aragorn_off_the_battlefield_triggers_nothing() {
    let mut g = table(&["aragorn", "cloud"]);
    let life = g.player(P1).life;
    let c = card(&g, "Lightning Bolt");
    cast::on_cast(&mut g, P0, c).unwrap(); // Aragorn still in the command zone
    assert_eq!(g.player(P1).life, life);
    let a = perm(&mut g, P0, ARAGORN);
    g.perm_mut(a).sick = false;
    let l0 = g.player(P0).life;
    cast::on_cast(&mut g, P1, c).unwrap(); // the opponent casts it
    assert_eq!(g.player(P0).life, l0);
}

#[test]
fn the_plan_casts_aura_shards() {
    // the 11/06 analysis: Aura Shards was never cast (no rule ranked it); the plan file now does
    let (mut g, _) = setup();
    g.player_mut(P0).turns = 4;
    let c = hand(&mut g, P0, &["Aura Shards"])[0];
    assert!(sim_core::ai::decks::deck_prio(&g, P0, c) > 0);
}

// ------------------------------------------------------------------ the AI's value of the triggers
use sim_core::impls::aragorn as ara;

fn main_labels(g: &mut Game) -> Vec<String> {
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    sim_core::ai::brain::main_options(g, P0, false).unwrap().into_iter().map(|o| o.label).collect()
}

#[test]
fn the_trigger_value_needs_aragorn_out_and_follows_the_colours() {
    let mut g = table(&["aragorn", "cloud"]);
    let bolt = card(&g, "Lightning Bolt");
    assert_eq!(ara::cast_value(&g, P0, bolt), 0.0, "no Aragorn out: nothing");
    let a = perm(&mut g, P0, ARAGORN);
    g.perm_mut(a).sick = false;
    assert_eq!(ara::red_damage(&g, P0, bolt), 3);
    assert!(ara::cast_value(&g, P0, bolt) > 0.0);
    assert_eq!(ara::red_damage(&g, P0, card(&g, "Sol Ring")), 0, "colourless: no trigger");
    perm(&mut g, P0, "Veyran, Voice of Duality");
    assert_eq!(ara::red_damage(&g, P0, bolt), 6, "Veyran: an instant triggers twice");
    assert_eq!(ara::red_damage(&g, P0, card(&g, "Guttersnipe")), 3, "a creature spell: once");
    perm(&mut g, P0, "Fiery Emancipation");
    assert_eq!(ara::red_damage(&g, P0, bolt), 18, "Fiery Emancipation triples it");
}

#[test]
fn the_damage_that_kills_is_worth_much_more() {
    let (mut g, _) = setup();
    let bolt = card(&g, "Lightning Bolt");
    let plain = ara::cast_value(&g, P0, bolt);
    g.player_mut(P1).life = 3;
    assert!(ara::cast_value(&g, P0, bolt) > plain + 3.0);
}

#[test]
fn with_aragorn_out_lightning_bolt_goes_to_the_face() {
    let (mut g, _) = setup();
    lands(&mut g, P0, "Mountain", 1, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Lightning Bolt"]);
    let labels = main_labels(&mut g);
    assert!(labels.iter().any(|l| l.starts_with("Lightning Bolt to the face")), "{labels:?}");
}

#[test]
fn without_aragorn_bolt_is_not_thrown_at_a_full_life_total() {
    let mut g = table(&["aragorn", "cloud"]);
    lands(&mut g, P0, "Mountain", 1, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Lightning Bolt"]);
    let labels = main_labels(&mut g);
    assert!(!labels.iter().any(|l| l.starts_with("Lightning Bolt to the face")), "{labels:?}");
}

#[test]
fn bolt_plus_aragorn_is_lethal_at_six() {
    let (mut g, _) = setup();
    lands(&mut g, P0, "Mountain", 1, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Lightning Bolt"]);
    g.player_mut(P1).life = 6;
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let face = opts.iter().find(|o| o.label.starts_with("Lightning Bolt to the face")).expect("a face option");
    assert_eq!(face.utility, 9.0, "the lethal burn's utility: Bolt 3 + Aragorn 3 = 6");
}

#[test]
fn counterspells_are_still_held_with_aragorn_out() {
    let (mut g, _) = setup();
    lands(&mut g, P0, "Island", 2, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Counterspell"]);
    let labels = main_labels(&mut g);
    assert!(!labels.iter().any(|l| l.starts_with("Counterspell")), "{labels:?}");
}
