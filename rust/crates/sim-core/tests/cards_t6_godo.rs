//! Tier 6 "godo" batch (impls/t6_godo.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["godo", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

#[allow(dead_code)]
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn on(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
}

fn attack(g: &mut Game, src: PermId, p: PlayerId, atk: &[PermId], d: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().attack.unwrap();
    f(g, src, p, atk, d).unwrap();
}

// ======================================================== Godo, Bandit Warlord
#[test]
fn godo_enters_and_fetches_helm_onto_the_battlefield() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Helm of the Host"), "Equipment put onto the battlefield, not the hand");
    assert!(!g.player(P0).hand.contains(&card(&g, "Helm of the Host")));
    assert!(!g.player(P0).library.contains(&card(&g, "Helm of the Host")));
    let _ = godo;
}

#[test]
fn godo_fetches_hammer_first_with_a_copier_in_hand() {
    let mut g = tbl();
    hand(&mut g, P0, &["Twinflame"]);
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Hammer of Nazahn"));
    assert!(!on(&g, P0, "Helm of the Host"));
}

#[test]
fn godo_then_fetches_hammer_once_helm_is_out() {
    let mut g = tbl();
    perm(&mut g, P0, "Helm of the Host");
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Hammer of Nazahn"), "Helm already out: the next Equipment is Hammer");
}

#[test]
fn godo_first_attack_untaps_and_adds_one_combat_once_a_turn() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(godo).tapped = true; // attacking tapped it
    assert_eq!(g.player(P0).extra_combats, 0);
    attack(&mut g, godo, P0, &[godo], P1);
    assert!(!g.perm(godo).tapped, "untaps");
    assert_eq!(g.player(P0).extra_combats, 1, "one additional combat phase");
    g.perm_mut(godo).tapped = true;
    attack(&mut g, godo, P0, &[godo], P1);
    assert_eq!(g.player(P0).extra_combats, 1, "only the first attack each turn");
    assert!(g.perm(godo).tapped);
}

#[test]
fn godo_attack_trigger_needs_godo_to_attack() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let other = token(&mut g, P0, 2);
    attack(&mut g, godo, P0, &[other], P1);
    assert_eq!(g.player(P0).extra_combats, 0);
}
