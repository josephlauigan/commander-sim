//! The AI plans of the Tymna decks (impls/tymna_partners.rs): spreading the attack so each opponent is hit, and the
//! cast priorities.

use sim_core::ai::decks::deck_prio;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::tymna_partners::spread;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

fn tbl(key: &str) -> Game {
    table(&[key, "veyran", "seph", "galadriel"])
}

/// the opponents each attacker goes to, in the order of `atk`
fn dest(groups: &[(PlayerId, Vec<PermId>)], a: PermId) -> Option<PlayerId> {
    groups.iter().find(|x| x.1.contains(&a)).map(|x| x.0)
}

fn fliers(g: &mut Game) -> Vec<PermId> {
    ["Falcon and Redwing", "Aven Mindcensor", "Nightwind Glider"].iter().map(|n| perm(g, P0, n)).collect()
}

#[test]
fn each_opponent_gets_a_flier_while_tymna_is_out() {
    let mut g = tbl("thrasios");
    perm(&mut g, P0, "Tymna the Weaver");
    let f = fliers(&mut g);
    for q in [P1, P2, P3] {
        perm(&mut g, q, "Bastion Protector"); // a ground blocker
    }
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    let mut to: Vec<PlayerId> = f.iter().map(|&a| dest(&groups, a).unwrap()).collect();
    to.sort();
    assert_eq!(to, vec![P1, P2, P3], "one flier at each opponent: {groups:?}");
}

#[test]
fn the_cheapest_attacker_is_sent_and_a_ground_creature_stays() {
    let mut g = tbl("kraum");
    perm(&mut g, P0, "Tymna the Weaver");
    let f = fliers(&mut g);
    let big = perm(&mut g, P0, "Lyra Dawnbringer"); // a flier too, but the most valuable
    for q in [P1, P2, P3] {
        perm(&mut g, q, "Bastion Protector");
    }
    let mut all = f.clone();
    all.push(big);
    let groups = spread(&g, P0, vec![(P1, all.clone())], &all);
    assert_eq!(dest(&groups, big), Some(P1), "the best creature stays on the first target: {groups:?}");
    assert!(groups.len() == 3, "{groups:?}");
}

#[test]
fn an_opponent_with_a_flying_blocker_gets_no_attacker() {
    let mut g = tbl("thrasios");
    perm(&mut g, P0, "Tymna the Weaver");
    let f = fliers(&mut g);
    perm(&mut g, P2, "Falcon and Redwing"); // can block fliers
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    assert!(groups.iter().all(|x| x.0 != P2), "P2 can block every flier: {groups:?}");
    assert!(groups.iter().any(|x| x.0 == P3), "{groups:?}");
}

#[test]
fn a_lethal_attack_is_kept_whole() {
    let mut g = tbl("thrasios");
    perm(&mut g, P0, "Tymna the Weaver");
    let f = fliers(&mut g);
    g.player_mut(P1).life = 5; // Falcon 1 + Mindcensor 2 + Glider 2: all three needed
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    assert_eq!(groups.len(), 1, "{groups:?}");
}

#[test]
fn nothing_changes_without_tymna_or_for_other_decks() {
    let mut g = tbl("thrasios");
    let f = fliers(&mut g);
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    assert_eq!(groups, vec![(P1, f.clone())], "no Tymna, no change");
    let mut g = tbl("sauron");
    perm(&mut g, P0, "Tymna the Weaver");
    let f = fliers(&mut g);
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    assert_eq!(groups, vec![(P1, f)], "another deck's attack is untouched");
}

#[test]
fn reconnaissance_mission_also_spreads() {
    let mut g = tbl("kraum");
    perm(&mut g, P0, "Reconnaissance Mission");
    let f = fliers(&mut g);
    let groups = spread(&g, P0, vec![(P1, f.clone())], &f);
    assert_eq!(groups.len(), 3, "{groups:?}");
}

#[test]
fn priorities_follow_the_plan() {
    let mut g = tbl("thrasios");
    let h = hand(&mut g, P0, &["Falcon and Redwing", "Lyra Dawnbringer", "Bastion Protector", "Birds of Paradise"]);
    let (falcon, lyra, bastion, birds) = (h[0], h[1], h[2], h[3]);
    g.player_mut(P0).turns = 3;
    let tymna = g.player(P0).cmd;
    let p = |g: &Game, c| deck_prio(g, P0, c);
    assert!(p(&g, falcon) >= 78, "a cheap flier while no opponent is covered");
    assert!(p(&g, birds) > p(&g, falcon), "mana first");
    assert!(p(&g, lyra) < p(&g, falcon));
    assert!(p(&g, bastion) <= 40, "Bastion Protector waits for both commanders");
    assert!(p(&g, tymna) >= 85 || g.db.get(tymna).name.as_ref() != "Tymna the Weaver");
    // three fliers out: the next one is an ordinary creature
    fliers(&mut g);
    assert!(p(&g, falcon) < 78);
    // both commanders out: Bastion Protector comes down
    perm(&mut g, P0, "Tymna the Weaver");
    perm(&mut g, P0, "Thrasios, Triton Hero");
    assert!(p(&g, bastion) >= 70);
}
