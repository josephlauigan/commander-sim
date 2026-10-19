//! Practice mode: the person's creatures can attack an opponent's planeswalker (alone, or split with an attack at a
//! player), and the engine deals their damage to its loyalty.

mod play_common;
use play_common::*;
use sim_core::engine::combat::combat;
use sim_core::human::combat::{SPLIT, attack_candidates, attackable_walkers};

fn setup() -> (Game, PermId) {
    let mut g = table(&["sauron", "veyran"]);
    let vr = perm(&mut g, P1, "Vraska, Betrayal's Sting");
    (g, vr)
}

#[test]
fn attack_a_planeswalker() {
    let (mut g, vr) = setup();
    let t = token(&mut g, P0, 2);
    let start = g.perm(vr).loyalty.unwrap();
    let h = seat(&mut g, P0, vec![picks(&[0]), pick(1)]); // the creature, then the walker
    combat(&mut g, P0).unwrap();
    assert_eq!(g.perm(vr).loyalty, Some(start - 2));
    assert_eq!(g.player(P1).life, 40, "the player takes nothing");
    assert!(g.perm(t).tapped);
    let asked = h.asked();
    assert_eq!(asked[1].kind.to_string(), "target");
    assert!(asked[1].choices[1].contains("Vraska") && asked[1].choices[1].contains(&format!("{start} loyalty")));
}

#[test]
fn attack_the_player_when_a_walker_is_there() {
    let (mut g, vr) = setup();
    token(&mut g, P0, 2);
    let start = g.perm(vr).loyalty;
    seat(&mut g, P0, vec![picks(&[0]), pick(0)]);
    combat(&mut g, P0).unwrap();
    assert_eq!((g.player(P1).life, g.perm(vr).loyalty), (38, start));
}

#[test]
fn split_between_player_and_walker() {
    let (mut g, vr) = setup();
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 3);
    let start = g.perm(vr).loyalty.unwrap();
    let c = attack_candidates(&g, P0);
    let (ia, ib) = (c.iter().position(|&x| x == a).unwrap(), c.iter().position(|&x| x == b).unwrap());
    let h = seat(&mut g, P0, vec![picks(&[ia, ib]), pick(2), pick(0), pick(1)]); // a -> player, b -> walker
    combat(&mut g, P0).unwrap();
    assert_eq!((g.player(P1).life, g.perm(vr).loyalty), (38, Some(start - 3)));
    assert!(h.asked()[1].choices.iter().any(|x| x == SPLIT));
}

#[test]
fn a_dead_walker_goes_to_the_graveyard_and_the_player_is_spared() {
    let (mut g, vr) = setup();
    token(&mut g, P0, 8);
    seat(&mut g, P0, vec![picks(&[0]), pick(1)]);
    combat(&mut g, P0).unwrap();
    assert!(!g.player(P1).perms.contains(&vr));
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn no_walkers_no_extra_question() {
    let mut g = table(&["sauron", "veyran"]);
    token(&mut g, P0, 2);
    let h = seat(&mut g, P0, vec![picks(&[0])]);
    assert!(attackable_walkers(&g, P0).is_empty());
    combat(&mut g, P0).unwrap();
    assert_eq!(h.asked().len(), 1);
    assert_eq!(g.player(P1).life, 38);
}
