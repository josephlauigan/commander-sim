//! The sacrifice package for Sephiroth in Sheoldred: Phyrexian Tower, the free outlets (Phyrexian Altar, Carrion
//! Feeder), Carnage Altar, Common Crook, Callous Inspector and the AI rules for using them. Seat 0 plays the
//! sheoldred deck, seat 1 the seph deck.

use sim_core::ai::decks;
use sim_core::engine::pool::Kind;
use sim_core::cardcode;
use sim_core::dsl;
use sim_core::engine::mana;
use sim_core::engine::zones::{self, die};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::common;
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

/// three opponents, the usual pod: a drain is worth three
fn game4() -> Game {
    table(&["sheoldred", "seph", "veyran", "sauron"])
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        _ => panic!("not a plan"),
    }
}

fn sacrificed(g: &Game, p: PlayerId) -> i64 {
    g.player(p).stats.get("creatures_sacrificed").copied().unwrap_or(0)
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

fn tokens(g: &mut Game, p: PlayerId, n: usize) -> Vec<PermId> {
    (0..n).map(|_| token(g, p, 1)).collect()
}

fn tower_opts(g: &mut Game) -> Vec<Opt> {
    cardcode::land_options(g, P0, Some(true))
        .unwrap()
        .into_iter()
        .filter(|o| o.label.contains("Phyrexian Tower"))
        .collect()
}

// ------------------------------------------------------------------ Phyrexian Tower
#[test]
fn the_tower_taps_for_one_colourless_not_two_black() {
    let mut g = game();
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 1);
    assert_eq!(u[0].amt, 1);
    assert!(u[0].cols.is_empty(), "the Tower taps for {{C}}");
    assert!(!mana::can_pay(&g, P0, 0, "B", false));
}

#[test]
fn the_tower_sacrifices_for_bb_when_a_death_payoff_is_out() {
    let mut g = game();
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Blood Artist");
    let t = token(&mut g, P0, 1);
    let o = tower_opts(&mut g);
    assert_eq!(o.len(), 1, "Sephiroth and Blood Artist make the sacrifice worth it");
    let before = g.player(P1).life;
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P0).mana.get(Kind::B), 2);
    assert_eq!(g.player(P1).life, before - 2, "Sephiroth and Blood Artist each drain 1");
    assert!(g.perm(g.player(P0).lands[0]).tapped);
    assert_eq!(sacrificed(&g, P0), 1);
}

#[test]
fn the_tower_is_left_alone_without_a_payoff_or_a_spell_to_cast() {
    let mut g = game();
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    token(&mut g, P0, 1);
    assert!(tower_opts(&mut g).is_empty());
}

#[test]
fn the_tower_sacrifices_for_mana_that_puts_a_spell_in_reach() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    let t = token(&mut g, P0, 1);
    hand(&mut g, P0, &["Crypt Ghast"]); // {3}{B}: four mana, three on hand
    let o = tower_opts(&mut g);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(t).on_bf);
    // two Swamps (BB), the Tower's own BB floating: Sheoldred is castable
    assert!(mana::can_pay(&g, P0, 2, "BB", false));
}

#[test]
fn the_tower_never_sacrifices_the_commander_a_payoff_or_sephiroth() {
    let mut g = game();
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Blood Artist");
    let c = token(&mut g, P0, 1);
    g.perm_mut(c).is_cmd = true;
    assert!(tower_opts(&mut g).is_empty());
}

// ------------------------------------------------------------------ the free outlets
#[test]
fn phyrexian_altar_sacrifices_spare_fodder_for_mana_and_the_drain() {
    let mut g = game4();
    perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Phyrexian Altar");
    let t = token(&mut g, P0, 1);
    let o = common::aristocrat_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1, "a death payoff is out");
    let before = g.player(P1).life;
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P0).mana.get(Kind::Any), 1);
    assert_eq!(g.player(P1).life, before - 1, "Sephiroth drains");
}

#[test]
fn carrion_feeder_grows_with_each_sacrifice() {
    let mut g = game();
    let f = perm(&mut g, P0, "Carrion Feeder");
    let ts = tokens(&mut g, P0, 2);
    common::sac_through(&mut g, P0, f, ts[0]).unwrap();
    common::sac_through(&mut g, P0, f, ts[1]).unwrap();
    assert_eq!(g.perm(f).plus, 2);
}

#[test]
fn no_outlet_use_without_a_payoff_or_a_reason() {
    let mut g = game();
    perm(&mut g, P0, "Phyrexian Altar");
    token(&mut g, P0, 1);
    assert!(common::aristocrat_options(&mut g, P0, Some(true)).unwrap().is_empty());
}

#[test]
fn the_commander_and_sephiroth_are_never_fodder() {
    let mut g = game();
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Blood Artist");
    let c = token(&mut g, P0, 1);
    g.perm_mut(c).is_cmd = true;
    assert!(common::aristocrat_options(&mut g, P0, Some(true)).unwrap().is_empty());
}

#[test]
fn four_sacrifices_in_one_turn_flip_sephiroth() {
    let mut g = game();
    let s = perm(&mut g, P0, SEPH);
    let altar = perm(&mut g, P0, "Phyrexian Altar");
    let ts = tokens(&mut g, P0, 4);
    let before = g.player(P1).life;
    for t in ts {
        common::sac_through(&mut g, P0, altar, t).unwrap();
    }
    assert!(flipped(&g, s));
    assert_eq!(g.player(P0).emblems.iter().filter(|&&e| e == "seph").count(), 1);
    assert_eq!(g.player(P1).life, before - 4);
    assert_eq!(g.player(P0).stats.get("seph_flips").copied(), Some(1));
    assert_eq!(sacrificed(&g, P0), 4);
}

#[test]
fn the_ai_sacrifices_its_spare_fodder_until_sephiroth_flips() {
    let mut g = game4();
    let s = perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Carrion Feeder");
    tokens(&mut g, P0, 5);
    for _ in 0..8 {
        let o = common::aristocrat_options(&mut g, P0, Some(true)).unwrap();
        let Some(o) = o.first() else { break };
        run(&mut g, P0, o);
    }
    assert!(flipped(&g, s), "four spare bodies went to the outlets");
    assert!(sacrificed(&g, P0) >= 4);
}

// ------------------------------------------------------------------ a creature that dies anyway
#[test]
fn a_creature_that_is_about_to_die_goes_to_an_outlet_that_gains_something() {
    let mut g = game();
    let f = perm(&mut g, P0, "Carrion Feeder");
    let c = perm(&mut g, P0, "Common Crook");
    assert!(cardcode::sac_in_response(&mut g, P0, c, "destroy").unwrap());
    assert!(!g.perm(c).on_bf);
    assert_eq!(g.perm(f).plus, 1);
    assert_eq!(g.player(P0).treasures, 1, "Common Crook's Treasure");
}

#[test]
fn the_commander_is_not_sacrificed_in_response() {
    let mut g = game();
    perm(&mut g, P0, "Carrion Feeder");
    let c = token(&mut g, P0, 1);
    g.perm_mut(c).is_cmd = true;
    assert!(!cardcode::sac_in_response(&mut g, P0, c, "destroy").unwrap());
    assert!(g.perm(c).on_bf);
}

#[test]
fn altar_mana_is_only_worth_a_sacrifice_in_response_when_a_spell_waits_on_it() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    perm(&mut g, P0, "Phyrexian Altar");
    let a = token(&mut g, P0, 1);
    assert!(!cardcode::sac_in_response(&mut g, P0, a, "destroy").unwrap(), "nothing to spend the mana on");
    hand(&mut g, P0, &["Crypt Ghast"]);
    assert!(cardcode::sac_in_response(&mut g, P0, a, "destroy").unwrap());
}

#[test]
fn a_wipe_finds_the_outlets_sacrificing_what_it_kills() {
    let mut g = game4();
    perm(&mut g, P0, SEPH);
    let f = perm(&mut g, P0, "Carrion Feeder");
    let ts = tokens(&mut g, P0, 2);
    let before = g.player(P1).life;
    let r = decks::wipe_response(&mut g, P0, "destroy", P1).unwrap();
    assert_eq!(r, None);
    assert!(ts.iter().all(|&t| !g.perm(t).on_bf));
    assert!(g.perm(f).plus >= 2 || !g.perm(f).on_bf);
    assert!(g.player(P1).life < before, "Sephiroth drained on the way out");
}

// ------------------------------------------------------------------ Carnage Altar, Common Crook, Callous Inspector
#[test]
fn carnage_altar_pays_three_and_a_creature_for_a_card() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let altar = perm(&mut g, P0, "Carnage Altar");
    let t = token(&mut g, P0, 1);
    let o = dsl::ability_options(&mut g, P0, true).unwrap();
    let o = o.iter().find(|o| o.label.contains("Carnage Altar")).expect("offered");
    let hand_before = g.player(P0).hand.len();
    let Some(Action::Dsl { src, idx }) = o.act.clone() else { panic!() };
    assert_eq!(src, altar);
    assert!(dsl::activate(&mut g, P0, src, idx).unwrap());
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P0).hand.len(), hand_before + 1);
    assert_eq!(mana::total_mana(&g, P0, false), 0);
}

#[test]
fn carnage_altar_never_eats_the_commander_or_a_payoff() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    perm(&mut g, P0, "Carnage Altar");
    perm(&mut g, P0, "Blood Artist");
    let c = token(&mut g, P0, 1);
    g.perm_mut(c).is_cmd = true;
    let o = dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(!o.iter().any(|o| o.label.contains("Carnage Altar")));
}

#[test]
fn common_crook_leaves_a_treasure() {
    let mut g = game();
    let c = perm(&mut g, P0, "Common Crook");
    die(&mut g, c, "sac").unwrap();
    assert_eq!(g.player(P0).treasures, 1);
}

#[test]
fn callous_inspector_hurts_you_and_leaves_a_clue() {
    let mut g = game();
    let c = perm(&mut g, P0, "Callous Inspector");
    let life = g.player(P0).life;
    die(&mut g, c, "sac").unwrap();
    assert_eq!(g.player(P0).life, life - 1);
    assert_eq!(g.player(P0).clues, 1);
    let _ = zones::sac_worth;
}
