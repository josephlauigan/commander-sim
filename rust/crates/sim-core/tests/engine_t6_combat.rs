//! Tier 6 engine pass, combat (plan row 8): no cap on additional combat phases, a beginning-of-combat event for every
//! player's combat, and "end the combat phase".

use sim_core::engine::combat::combat;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["godo", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

#[test]
fn godo_with_helm_runs_past_four_combats_and_kills_three_goldfish() {
    let mut g = tbl();
    perm(&mut g, P0, "Helm of the Host");
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let helm = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Helm of the Host").unwrap();
    g.perm_mut(helm).attached = Some(godo);
    for q in 1..4 {
        g.player_mut(PlayerId(q)).life = 40;
    }
    combat(&mut g, P0).unwrap();
    assert!(g.player(P0).combat_no > 4, "more than four combats ran: {}", g.player(P0).combat_no);
    for q in 1..4 {
        assert!(!g.player(PlayerId(q)).alive, "opponent {q} is dead");
    }
}

#[test]
fn each_combat_fires_for_an_opponents_combat_and_when_nobody_attacks() {
    let mut g = tbl();
    let t = token(&mut g, P1, 2);
    let s = perm(&mut g, P1, "Sting, the Glinting Dagger");
    g.perm_mut(s).attached = Some(t);
    g.perm_mut(t).tapped = true;
    // P0's combat with no creatures: nobody attacks, yet Sting's controller untaps the equipped creature
    combat(&mut g, P0).unwrap();
    assert!(!g.perm(t).tapped, "Sting untapped at the beginning of an opponent's combat");
}

#[test]
fn end_the_combat_phase_skips_damage_and_removes_attackers_but_later_combats_happen() {
    let mut g = tbl();
    let a = token(&mut g, P0, 5);
    // Mandate of Peace resolved in the declare attackers step: the rest of this combat is skipped
    g.end_combat = true;
    let r = sim_core::engine::combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert!(r.is_empty());
    assert_eq!(g.player(P1).life, 40, "no combat damage");
    // the flag is for that one combat: the next combat phase deals damage
    g.end_combat = false;
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 35);
    // and combat() clears a stale flag when it starts a phase
    g.end_combat = true;
    g.player_mut(P0).extra_combats = 0;
    combat(&mut g, P0).unwrap();
    assert!(!g.end_combat);
}

#[test]
fn karlach_with_isshin_gives_at_most_two_extra_combats() {
    // Karlach's extra combat is "if it's the first combat phase of the turn"; Isshin doubles it: 3 combats at most.
    // Its compiled ability had no first-combat condition and, uncapped, chained to 200 combats (fixed 2026-10-18).
    let mut g = table(&["isshin-mardu-attack-triggers", "lathril-golgari-elves", "tatyova-simic-landfall", "teysa-orzhov-aristocrats"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    perm(&mut g, P0, "Karlach, Fury of Avernus");
    for _ in 0..3 {
        token(&mut g, P0, 3);
    }
    for q in 1..4 {
        g.player_mut(PlayerId(q)).life = 200;
    }
    combat(&mut g, P0).unwrap();
    let n = g.player(P0).combat_no;
    assert!(n >= 2 && n <= 3, "Karlach + Isshin: 2 or 3 combats, got {n}");
}
