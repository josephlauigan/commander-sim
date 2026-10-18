//! Kinnan (mana combo) plan slots: mulligan, tutor wish list, cast priority, go/wait
//! (audit/t6/kinnan-bonder-mana-combo/SPEC.md decision scenarios 3, 11, 12, 14).

use sim_core::ids::PlayerId;
use sim_core::impls::{t6_plan_kinnan as plan, t6_plans};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const KINNAN: &str = "kinnan-bonder-mana-combo";

fn tbl() -> Game {
    let mut g = table(&[KINNAN, "seph", "sauron", "kefka"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn name(g: &Game, c: sim_core::ids::CardId) -> String {
    g.db.get(c).name.to_string()
}

#[test]
fn keeps_dorks_with_kinnan_and_fast_mana_with_an_engine() {
    // scenario 11: two lands, two dorks, a free counter, an engine
    let mut g = tbl();
    hand(
        &mut g,
        P0,
        &["Island", "Forest", "Birds of Paradise", "Elvish Mystic", "Force of Will", "Consecrated Sphinx"],
    );
    hand(&mut g, P0, &["Dispel"]);
    assert_eq!(plan::keep(&g, P0), Some(true));
    // scenario 14: Sol Ring, Talisman, Remora, two lands
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &["Sol Ring", "Talisman of Curiosity", "Mystic Remora", "Island", "Forest", "Dispel", "Dizzy Spell"],
    );
    assert_eq!(plan::keep(&g, P0), Some(true));
}

#[test]
fn ships_one_land_hands_and_hands_with_nothing_to_do() {
    // scenario 12: one land (Ancient Tomb), Mox Diamond (needs a land to discard), Study, Halfling, two spells
    let mut g = tbl();
    hand(
        &mut g,
        P0,
        &["Ancient Tomb", "Mox Diamond", "Rhystic Study", "Delighted Halfling", "Commandeer", "Swan Song"],
    );
    hand(&mut g, P0, &["Dispel"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
    // no land at all
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &["Sol Ring", "Mana Vault", "Rhystic Study", "Force of Will", "Birds of Paradise", "Dispel", "Dizzy Spell"],
    );
    assert_eq!(plan::keep(&g, P0), Some(false));
    // six lands
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Island", "Forest", "Island", "Forest", "Island", "Forest", "Sol Ring"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
}

#[test]
fn two_lands_and_nothing_castable_is_a_mulligan_but_three_lands_and_an_engine_is_not() {
    let mut g = tbl();
    hand(
        &mut g,
        P0,
        &["Island", "Forest", "Dispel", "Dizzy Spell", "Finale of Devastation", "Chain of Vapor", "Snapback"],
    );
    assert_eq!(plan::keep(&g, P0), Some(false));
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &["Island", "Forest", "Breeding Pool", "Rhystic Study", "Dispel", "Dizzy Spell", "Chain of Vapor"],
    );
    assert_eq!(plan::keep(&g, P0), Some(true));
}

#[test]
fn kinnan_comes_before_the_rocks_and_the_counters_are_held() {
    let mut g = tbl();
    let cs = hand(&mut g, P0, &["Force of Will", "Rhystic Study", "Arcane Signet"]);
    let k = g.player(P0).cmd;
    assert!(plan::prio(&g, P0, k).unwrap() > 88.0);
    assert_eq!(plan::prio(&g, P0, cs[0]), Some(0.0));
    assert!(plan::prio(&g, P0, cs[1]).unwrap() > 80.0);
    assert_eq!(plan::prio(&g, P0, cs[2]), None);
}

#[test]
fn wish_list_asks_for_the_monolith_then_an_outlet() {
    let mut g = tbl();
    let v: Vec<_> = [
        "Rhystic Study",
        "Thrasios, Triton Hero",
        "Basalt Monolith",
        "Transmute Artifact",
        "Emrakul, the Promised End",
    ]
    .iter()
    .map(|n| card(&g, n))
    .collect();
    g.player_mut(P0).library = v;
    let w = t6_plans::wish(&g, P0).unwrap();
    let names: Vec<String> = w.iter().map(|&c| name(&g, c)).collect();
    assert_eq!(names[0], "Basalt Monolith");
    assert_eq!(names[1], "Transmute Artifact");
    assert!(!names.contains(&"Emrakul, the Promised End".to_string()));
    // Monolith in hand: the outlet comes first
    hand(&mut g, P0, &["Basalt Monolith"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(name(&g, w[0]), "Thrasios, Triton Hero");
}

#[test]
fn waits_on_the_loop_while_stax_sits_on_the_table() {
    // scenario 3 reason: Collector Ouphe stops the rocks and the Monolith
    let mut g = tbl();
    perm(&mut g, P1, "Collector Ouphe");
    assert_eq!(plan::go_now(&g, P0, "Kinnan + Basalt Monolith"), Some(false));
    assert_eq!(plan::go_now(&g, P0, "Thassa's Oracle + Demonic Consultation / Tainted Pact"), None);
}

#[test]
fn other_decks_do_not_reach_the_kinnan_plan() {
    let mut g = table(&["seph", KINNAN, "sauron", "kefka"]);
    g.active = Some(P0);
    assert_eq!(t6_plans::keep(&g, P0), None);
}
