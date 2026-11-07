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
