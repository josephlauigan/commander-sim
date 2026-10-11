//! The Torbran candidate deck (impls/torbran.rs and the engine's Torbran replacement effect), on hand-built positions.
//! Seat 0 plays the torbran deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::{cast, life, removal};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["torbran", "seph"])
}

fn attack(g: &mut Game, src: PermId, atk: &[PermId]) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().attack.unwrap();
    f(g, src, P0, atk, P1).unwrap();
}

fn blocks(g: &mut Game, src: PermId, assign: &mut Vec<(PermId, PermId)>) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().blocks.unwrap();
    f(g, src, P0, &[], P1, assign).unwrap();
}

fn options(g: &mut Game, post: bool) -> Vec<Opt> {
    brain::main_options(g, P0, post).unwrap()
}

// ------------------------------------------------------------------ Torbran
#[test]
fn torbran_adds_two_to_damage_to_an_opponent() {
    let mut g = game();
    life::lose_life(&mut g, P1, 3, Some(P0), "burn".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 37);
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    life::lose_life(&mut g, P1, 3, Some(P0), "burn".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 32); // 3 + 2
    life::lose_life(&mut g, P1, 1, Some(P0), "combat".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 29); // combat damage too
    life::lose_life(&mut g, P1, 2, Some(P0), "drain".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 27); // life loss is not damage
}

#[test]
fn torbran_does_not_change_damage_to_you_or_the_opponents_sources() {
    let mut g = game();
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    life::lose_life(&mut g, P0, 3, Some(P0), "other".into(), Some(true)).unwrap();
    assert_eq!(g.player(P0).life, 37); // damage to yourself
    life::lose_life(&mut g, P0, 3, Some(P1), "burn".into(), None).unwrap();
    assert_eq!(g.player(P0).life, 34); // an opponent's source: Torbran is not theirs
}

#[test]
fn two_torbrans_add_four() {
    let mut g = game();
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    life::lose_life(&mut g, P1, 1, Some(P0), "burn".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 35);
}

#[test]
fn torbran_lets_one_damage_kill_a_three_toughness_creature() {
    let mut g = game();
    let big = perm(&mut g, P1, "Grave Titan"); // 6/6
    // the targeting counts the +2
    let t = token(&mut g, P1, 1);
    g.perm_mut(t).tgh = 3;
    assert!(!removal::legal_targets(&g, P0, "dmg1", "c", false, None).contains(&t));
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    assert!(removal::legal_targets(&g, P0, "dmg1", "c", false, None).contains(&t));
    assert!(!removal::legal_targets(&g, P0, "dmg1", "c", false, None).contains(&big));
}

#[test]
fn torbran_makes_lightning_strike_lethal_sooner() {
    let mut g = game();
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    g.player_mut(P1).life = 5;
    lands(&mut g, P0, "Mountain", 2, false);
    let c = hand(&mut g, P0, &["Lightning Strike"])[0];
    let opts = options(&mut g, false);
    assert!(
        opts.iter().any(|o| o.label.starts_with("Lightning Strike to the face")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive);
}

#[test]
fn torbran_pingers_go_for_the_face() {
    let mut g = game();
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    let _small = perm(&mut g, P1, "Llanowar Elves");
    cast::bolt_something(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P1).life, 37); // 1 + 2 to the face, the Elves are not a threat worth a ping
}

// ------------------------------------------------------------------ creatures
#[test]
fn borderland_marauder_gets_plus_two_when_it_attacks() {
    let mut g = game();
    let m = perm(&mut g, P0, "Borderland Marauder");
    let base = sim_core::engine::values::epow(&g, m);
    attack(&mut g, m, &[m]);
    assert_eq!(sim_core::engine::values::epow(&g, m), base + 2);
}

#[test]
fn goblin_medics_pings_when_it_attacks() {
    let mut g = game();
    let m = perm(&mut g, P0, "Goblin Medics");
    g.perm_mut(m).tapped = true;
    attack(&mut g, m, &[m]);
    assert_eq!(g.player(P1).life, 39);
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    attack(&mut g, m, &[m]);
    assert_eq!(g.player(P1).life, 36); // 1 + 2
}

#[test]
fn karplusan_wolverine_pings_when_blocked_only() {
    let mut g = game();
    let w = perm(&mut g, P0, "Karplusan Wolverine");
    let b = perm(&mut g, P1, "Grave Titan");
    blocks(&mut g, w, &mut vec![]);
    assert_eq!(g.player(P1).life, 40);
    blocks(&mut g, w, &mut vec![(w, b)]);
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn godos_irregulars_pay_red_to_kill_the_blocker() {
    let mut g = game();
    let i = perm(&mut g, P0, "Godo's Irregulars");
    let b = token(&mut g, P1, 3);
    g.perm_mut(b).tgh = 1;
    lands(&mut g, P0, "Mountain", 2, false);
    blocks(&mut g, i, &mut vec![(i, b)]);
    assert!(!g.perm(b).on_bf, "a 3/1 blocker dies to one activation");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 1);
}

#[test]
fn godos_irregulars_need_several_activations_without_torbran() {
    let mut g = game();
    let i = perm(&mut g, P0, "Godo's Irregulars");
    let b = token(&mut g, P1, 3);
    g.perm_mut(b).tgh = 3;
    lands(&mut g, P0, "Mountain", 3, false);
    blocks(&mut g, i, &mut vec![(i, b)]);
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3);
    // with Torbran one activation is 3 damage
    let b2 = token(&mut g, P1, 3);
    g.perm_mut(b2).tgh = 3;
    for &l in &g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    perm(&mut g, P0, "Torbran, Thane of Red Fell");
    blocks(&mut g, i, &mut vec![(i, b2)]);
    assert!(!g.perm(b2).on_bf);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 1);
}

// ------------------------------------------------------------------ spells
#[test]
fn impractical_joke_stops_prevention_and_deals_three() {
    let mut g = game();
    g.player_mut(P1).ring_prot = true; // The One Ring: damage from opponents' sources is prevented
    life::lose_life(&mut g, P1, 3, Some(P0), "burn".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 40);
    let t = token(&mut g, P1, 2);
    g.perm_mut(t).tgh = 3;
    lands(&mut g, P0, "Mountain", 1, false);
    let c = hand(&mut g, P0, &["Impractical Joke"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert!(!g.perm(t).on_bf);
    assert!(g.unpreventable());
    life::lose_life(&mut g, P1, 3, Some(P0), "burn".into(), None).unwrap();
    assert_eq!(g.player(P1).life, 37);
}

#[test]
fn rift_bolt_suspends_and_is_cast_at_the_next_upkeep() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 1, false);
    let c = hand(&mut g, P0, &["Rift Bolt"])[0];
    let opts = options(&mut g, true);
    let o = opts.iter().find(|o| o.label == "suspend Rift Bolt").expect("suspend option");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).suspended.contains(&(c, 1)));
    assert!(g.player(P0).exile.contains(&c));
    assert!(!g.player(P0).hand.contains(&c));
    sim_core::cardcode::suspend_upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).gy.contains(&c));
    assert_eq!(g.player(P1).life, 37);
}
