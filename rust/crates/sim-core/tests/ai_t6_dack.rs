//! Dack's plan slots (impls/t6_plan_dack.rs) on the decision scenarios of audit/t6/dack-fayden-white-stax/SPEC.md.

use sim_core::ai::decks::deck_prio_f;
use sim_core::ids::PlayerId;
use sim_core::impls::t6_plans;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const DACK: &str = "dack-fayden-white-stax";

fn tbl() -> Game {
    let mut g = table(&[DACK, "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    g.round = 3;
    g
}

fn opening(names: &[&str]) -> Game {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, names);
    g
}

fn dack(g: &Game) -> sim_core::ids::CardId {
    g.player(P0).cmd
}

/// six mana with WW for Dack: Plains x4 and Sol Ring and a Mox
fn six_mana(g: &mut Game) {
    lands(g, P0, "Plains", 4, false);
    perm(g, P0, "Sol Ring");
}

#[test]
fn mulligan_follows_the_mana() {
    // two lands and fast mana with something to do
    let g = opening(&[
        "Plains",
        "Ancient Tomb",
        "Sol Ring",
        "Chalice of the Void",
        "Silence",
        "Swords to Plowshares",
        "Cursed Totem",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
    // no white source and no way to make it
    let g = opening(&[
        "Mishra's Workshop",
        "Urza's Cave",
        "Sol Ring",
        "Mana Vault",
        "Silence",
        "Trinisphere",
        "Cursed Totem",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
    // all lands
    let g = opening(&["Plains", "Plains", "Plains", "Plains", "Command Tower", "Ancient Den", "Emergence Zone"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
    // only free mana, nothing else to spend it on
    let g = opening(&[
        "Plains",
        "Lotus Petal",
        "Lion's Eye Diamond",
        "Chalice of the Void",
        "Silence",
        "Trinisphere",
        "Defense Grid",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
}

#[test]
fn silence_goes_first_then_dack() {
    // scenario 1: 6 mana with WW + 1, Silence and Chant in hand, a blue opponent with open mana
    let mut g = opening(&["Silence", "Orim's Chant", "Reprieve"]);
    lands(&mut g, P0, "Plains", 5, false);
    perm(&mut g, P0, "Sol Ring");
    lands(&mut g, PlayerId(1), "Island", 3, false);
    hand(&mut g, PlayerId(1), &["Force of Will", "Counterspell"]);
    let s = card(&g, "Silence");
    let o = card(&g, "Orim's Chant");
    assert_eq!(deck_prio_f(&g, P0, dack(&g)), 0.0, "protection comes before the cast");
    assert!(deck_prio_f(&g, P0, s).max(deck_prio_f(&g, P0, o)) >= 90.0);
}

#[test]
fn no_protection_and_open_blue_mana_means_a_stax_piece() {
    // scenario 2: two opponents with open blue mana, no protection: Dack waits, Defense Grid is cast
    let mut g = opening(&["Defense Grid", "Trinisphere"]);
    six_mana(&mut g);
    for i in [1, 2] {
        lands(&mut g, PlayerId(i), "Island", 3, false);
        hand(&mut g, PlayerId(i), &["Force of Will", "Counterspell", "Mana Drain"]);
    }
    assert_eq!(deck_prio_f(&g, P0, dack(&g)), 0.0);
    assert!(deck_prio_f(&g, P0, card(&g, "Defense Grid")) > 60.0);
    // a tapped-out table: Dack goes
    let mut g = opening(&["Defense Grid"]);
    six_mana(&mut g);
    assert!(deck_prio_f(&g, P0, dack(&g)) >= 90.0);
}

#[test]
fn trinisphere_and_bauble_wait_for_the_fast_mana() {
    // scenario 3 and 12
    let g = opening(&["Trinisphere", "Mox Opal", "Lotus Petal", "Sol Ring", "Vexing Bauble"]);
    assert!(deck_prio_f(&g, P0, card(&g, "Trinisphere")) <= 0.0);
    assert!(deck_prio_f(&g, P0, card(&g, "Vexing Bauble")) <= 0.0);
    assert!(deck_prio_f(&g, P0, card(&g, "Sol Ring")) >= 90.0);
    let g = opening(&["Trinisphere"]);
    assert!(deck_prio_f(&g, P0, card(&g, "Trinisphere")) > 0.0);
}

#[test]
fn banishers_are_held_without_a_threat() {
    let g = opening(&["Fiend Hunter", "All-Fates Stalker"]);
    assert_eq!(deck_prio_f(&g, P0, card(&g, "Fiend Hunter")), 0.0);
}

#[test]
fn tutor_looks_for_mana_when_short_and_a_lock_piece_otherwise() {
    let mut g = opening(&["Plains"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(g.db.get(w[0]).name.as_ref(), "Sol Ring", "short of six mana: the rock first");
    six_mana(&mut g);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_ne!(g.db.get(w[0]).name.as_ref(), "Sol Ring");
}

#[test]
fn the_flip_goes_without_waiting_and_the_early_turns_develop() {
    let mut g = tbl();
    assert_eq!(t6_plans::go_now(&g, P0, "Dack Fayden: the first flip wins the turn (35 piles)"), Some(true));
    assert_eq!(t6_plans::go_now(&g, P0, "something else"), None);
    g.player_mut(P0).turns = 2;
    assert!(t6_plans::hold_up(&g, P0).is_some_and(|u| u < 0.0));
}
