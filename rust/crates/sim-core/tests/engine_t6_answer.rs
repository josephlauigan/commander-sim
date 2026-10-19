//! Engine feature `answer`: the AI counters an opponent's activated or triggered ability on the stack with a card that
//! can (Ertai Resurrected, Louisoix's Sacrifice), and Peter Parker's Camera copies a triggered ability on the stack.
//! Seat 0 holds the answers; seat 1 plays the Godo deck.

use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game_godo() -> Game {
    let mut g = table(&["godo", "seph"]);
    g.active = Some(P0);
    g
}

fn game() -> Game {
    let mut g = table(&["seph", "godo"]);
    g.active = Some(P1);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn blue_black(g: &mut Game, p: PlayerId) {
    lands(g, p, "Island", 2, false);
    lands(g, p, "Swamp", 2, false);
}

#[test]
fn ertai_counters_godos_enter_trigger_through_ai_respond() {
    let mut g = game();
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    let h = g.player(P1).hand.len();
    let godo = perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(g.perm(godo).on_bf);
    assert!(on_bf(&g, P0, "Ertai Resurrected").is_some(), "the AI flashed Ertai in");
    assert!(on_bf(&g, P1, "Helm of the Host").is_none(), "the enter trigger was countered: no Equipment");
    assert!(on_bf(&g, P1, "Hammer of Nazahn").is_none());
    assert_eq!(g.player(P1).hand.len(), h + 1, "Godo's controller drew a card");
}

#[test]
fn without_ertai_godo_fetches_helm() {
    let mut g = game();
    blue_black(&mut g, P0);
    perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(on_bf(&g, P1, "Helm of the Host").is_some());
}

#[test]
fn ertai_without_mana_cannot_answer() {
    let mut g = game();
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(on_bf(&g, P1, "Helm of the Host").is_some());
    assert!(on_bf(&g, P0, "Ertai Resurrected").is_none());
}

#[test]
fn louisoix_counters_godos_enter_trigger_paying_two_extra() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Louisoix's Sacrifice"]);
    perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(on_bf(&g, P1, "Helm of the Host").is_none(), "countered");
    assert!(in_gy(&g, P0, "Louisoix's Sacrifice"));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "{{U}} plus {{2}}");
}

#[test]
fn louisoix_sacrifices_a_legendary_creature_when_it_cannot_pay_two() {
    let mut g = game();
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Louisoix's Sacrifice"]);
    let legend = perm(&mut g, P0, "Adelbert Steiner"); // a legendary creature worth less than the trigger
    perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(on_bf(&g, P1, "Helm of the Host").is_none(), "countered");
    assert!(!g.perm(legend).on_bf, "sacrificed for the additional cost");
}

#[test]
fn camera_copies_godos_enter_trigger_copy_fetches_hammer_original_helm() {
    let mut g = game_godo();
    lands(&mut g, P0, "Island", 2, false);
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    // the table's seat 0 is `seph`: give it Godo's library
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let _ = 0;
    assert!(on_bf(&g, P0, "Hammer of Nazahn").is_some(), "the copy fetched Hammer of Nazahn");
    assert!(on_bf(&g, P0, "Helm of the Host").is_some(), "the original fetched Helm of the Host");
    assert!(g.perm(cam).tapped);
    assert_eq!(g.perm(cam).data.int(DataKey::Counters), 2, "one film counter removed");
    let helm = on_bf(&g, P0, "Helm of the Host").unwrap();
    assert_eq!(g.perm(helm).attached, Some(godo), "Hammer of Nazahn attached Helm to Godo");
}

#[test]
fn camera_needs_its_mana_and_a_counter() {
    let mut g = game_godo();
    let cam = perm(&mut g, P0, "Peter Parker's Camera"); // no lands: {2} unpayable
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(!g.perm(cam).tapped);
    assert_eq!(g.perm(cam).data.int(DataKey::Counters), 3);
    assert!(on_bf(&g, P0, "Hammer of Nazahn").is_none() != on_bf(&g, P0, "Helm of the Host").is_none(), "one fetch");
}

#[test]
fn ertai_counters_the_copy_and_the_original_still_resolves() {
    let mut g = game();
    g.active = Some(P1);
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    lands(&mut g, P1, "Mountain", 2, false);
    let cam = perm(&mut g, P1, "Peter Parker's Camera");
    perm(&mut g, P1, "Godo, Bandit Warlord");
    assert!(on_bf(&g, P0, "Ertai Resurrected").is_some(), "Ertai answered the copy on top of the stack");
    assert!(g.perm(cam).tapped);
    let n = ["Helm of the Host", "Hammer of Nazahn"].iter().filter(|n| on_bf(&g, P1, n).is_some()).count();
    assert_eq!(n, 1, "the copy was countered; the original fetched one Equipment");
}

#[test]
fn the_trigger_copy_is_counted_off_when_it_resolves() {
    // engine-level: the mechanism leaves no stale copy records behind
    let mut g = game_godo();
    lands(&mut g, P0, "Island", 2, false);
    perm(&mut g, P0, "Peter Parker's Camera");
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(g.trig_copies.is_empty());
    assert!(g.stack.is_empty());
}

#[test]
fn ertai_counters_helms_beginning_of_combat_trigger() {
    let mut g = game();
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    let helm = perm(&mut g, P1, "Helm of the Host");
    let host = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(helm).attached = Some(host);
    let n = g.player(P1).perms.len();
    let f = g.registry.get(g.perm(helm).cd.unwrap()).unwrap().combat_start.unwrap();
    let toks = f(&mut g, helm, P1).unwrap();
    assert!(toks.is_empty(), "the trigger was countered: no token copy");
    assert!(on_bf(&g, P0, "Ertai Resurrected").is_some());
    assert!(g.player(P1).perms.len() >= n, "(Grave Titan's own tokens aside) no Helm copy of it");
    assert_eq!(g.player(P1).perms.iter().filter(|&&m| &*g.perm(m).name == "Grave Titan").count(), 1);
}

#[test]
fn ertai_counters_thassas_oracle_win_trigger() {
    let mut g = game();
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    g.player_mut(P1).library.clear();
    perm(&mut g, P1, "Thassa's Oracle");
    assert!(!g.over, "the trigger was countered: no win");
    assert!(on_bf(&g, P0, "Ertai Resurrected").is_some());
}

#[test]
fn thassas_oracle_wins_when_nothing_answers() {
    let mut g = game();
    g.player_mut(P1).library.clear();
    perm(&mut g, P1, "Thassa's Oracle");
    assert!(g.over, "an empty library: Oracle wins");
}
