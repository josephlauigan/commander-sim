//! Godo's plan slots (impls/t6_plan_godo.rs) on the decision scenarios of audit/t6/godo-bandit-warlord/SPEC.md.

use sim_core::ai::decks::deck_prio_f;
use sim_core::ids::PlayerId;
use sim_core::impls::t6_plans;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const SISAY: &str = "sisay-legends-toolbox";

fn tbl() -> Game {
    let mut g = table(&["godo", KRAUM, SISAY, "seph"]);
    g.active = Some(P0);
    g
}

fn opening(names: &[&str]) -> Game {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, names);
    g
}

fn lib_names(g: &Game, ids: &[sim_core::ids::CardId]) -> Vec<String> {
    ids.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

#[test]
fn ramp_without_a_path_is_shipped() {
    // scenario 16
    let g = opening(&[
        "Mountain",
        "Sol Ring",
        "Chrome Mox",
        "Lotus Petal",
        "Rite of Flame",
        "Mana Vault",
        "Lion's Eye Diamond",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false), "five sources and a ritual, but no tutor and no win piece");
    let g = opening(&[
        "Mountain",
        "Sol Ring",
        "Lion's Eye Diamond",
        "Rite of Flame",
        "Desperate Ritual",
        "Defense Grid",
        "Blood Moon",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false), "two sources and no path to Godo");
}

#[test]
fn an_engine_with_permanent_mana_is_kept() {
    // scenario 17
    let g = opening(&[
        "The One Ring",
        "Basalt Monolith",
        "Voltaic Key",
        "Mountain",
        "Mountain",
        "Blood Moon",
        "Trinisphere",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
}

#[test]
fn a_tutor_with_fast_mana_is_kept_and_all_lands_or_no_lands_are_not() {
    // scenario 18 and the "ship" list
    let g = opening(&["Treasonous Ogre", "Sol Ring", "Chrome Mox", "Mountain", "Gamble", "Defense Grid"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
    let g = opening(&["Mountain", "Mountain", "Mountain", "Mountain", "Mountain", "Mountain", "Mountain"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
    let g = opening(&[
        "Gamble",
        "Imperial Recruiter",
        "Goblin Engineer",
        "Blood Moon",
        "Trinisphere",
        "Defense Grid",
        "Wheel of Fortune",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false), "no mana at all");
    let g = opening(&[
        "Mountain",
        "Mountain",
        "Helm of the Host",
        "Hammer of Nazahn",
        "Blood Moon",
        "Trinisphere",
        "Defense Grid",
    ]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true), "two lands and the pieces");
}

#[test]
fn zero_drops_come_before_vexing_bauble_and_trinisphere() {
    let g = opening(&["Vexing Bauble", "Chrome Mox", "Mountain", "Trinisphere"]);
    let b = card(&g, "Vexing Bauble");
    let t = card(&g, "Trinisphere");
    assert!(deck_prio_f(&g, P0, b) <= 5.0, "Bauble waits for the Moxes");
    assert!(deck_prio_f(&g, P0, t) <= 0.0, "Trinisphere waits for the cheap pieces");
    let g = opening(&["Vexing Bauble", "Mountain"]);
    let b = card(&g, "Vexing Bauble");
    assert!(deck_prio_f(&g, P0, b) > 5.0, "with no zero-drop left it is cast as usual");
}

#[test]
fn tutors_look_for_hammer_first_and_then_protection() {
    // scenario 15 (Hammer first with a copier) and the notes' tutor targets
    let mut g = opening(&["Twinflame", "Mountain"]);
    let w = t6_plans::wish(&g, P0).expect("a wish list");
    let n = lib_names(&g, &w);
    assert_eq!(n[0], "Hammer of Nazahn");
    assert!(n.iter().any(|x| x == "Dualcaster Mage"), "a copier in hand wants Dualcaster too: {n:?}");
    // Godo on the battlefield: Deflecting Swat is now a live protection piece
    perm(&mut g, P0, "Hammer of Nazahn");
    g.player_mut(P0).hand.clear();
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_ne!(lib_names(&g, &w)[0], "Hammer of Nazahn", "Hammer is out");
}

#[test]
fn an_early_turn_develops_instead_of_holding() {
    // a Mox in hand and Godo not out: the plan says develop
    let g = opening(&["Chrome Mox", "Mountain"]);
    assert!(t6_plans::hold_up(&g, P0).is_some_and(|u| u < 0.0));
    let g = opening(&["Defense Grid"]);
    assert_eq!(t6_plans::hold_up(&g, P0), None);
}

#[test]
fn a_turn_two_line_goes_even_into_some_risk() {
    // scenario 10: waiting loses the race; the plan answers instead of leaving it to the shared bar
    let mut g = opening(&["Pyroblast"]);
    g.round = 2;
    assert!(t6_plans::go_now(&g, P0, "Godo + Helm of the Host (equip {5})").is_some());
    // nobody has cards: no risk, so it goes
    for q in 1..4 {
        g.player_mut(PlayerId(q)).hand.clear();
    }
    assert_eq!(t6_plans::go_now(&g, P0, "Godo + Helm of the Host (equip {5})"), Some(true));
}
