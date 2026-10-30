//! Sisay (legends toolbox) plan slots: mulligan, Sisay's fetch, tutor wish list (audit/t6/sisay-legends-toolbox/SPEC.md).

use sim_core::ids::PlayerId;
use sim_core::impls::{legends, t6_plans as plan, t6_plans}; // --- phase4-cedh: the slots read the plan file
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const SISAY: &str = "sisay-legends-toolbox";

fn tbl() -> Game {
    let mut g = table(&[SISAY, "seph", "sauron", "kefka"]);
    g.active = Some(P0);
    g
}

fn lib(g: &mut Game, names: &[&str]) {
    let v: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(P0).library = v;
}

fn name(g: &Game, c: sim_core::ids::CardId) -> String {
    g.db.get(c).name.to_string()
}

#[test]
fn keeps_mana_hands_and_ships_empty_ones() {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &["Tundra", "Bayou", "Sol Ring", "Birds of Paradise", "Rhystic Study", "Lavinia, Azorius Renegade", "Silence"],
    );
    assert_eq!(plan::keep(&g, P0), Some(true));
    // one land with two mana makers is still a keep
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &["Tundra", "Sol Ring", "Noble Hierarch", "Birds of Paradise", "Demonic Tutor", "Silence", "Mental Misstep"],
    );
    assert_eq!(plan::keep(&g, P0), Some(true));
    // no land, no play
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &[
            "Sol Ring",
            "Mana Vault",
            "Demonic Tutor",
            "Silence",
            "Mental Misstep",
            "Derevi, Empyrial Tactician",
            "Teferi, Time Raveler",
        ],
    );
    assert_eq!(plan::keep(&g, P0), Some(false));
    // lands only
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Tundra", "Bayou", "Taiga", "Plateau", "Savannah", "Command Tower", "Scalding Tarn"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
    // two lands and nothing that makes mana or finds it
    g.player_mut(P0).hand.clear();
    hand(
        &mut g,
        P0,
        &[
            "Tundra",
            "Bayou",
            "Derevi, Empyrial Tactician",
            "Teferi, Time Raveler",
            "Silence",
            "Esper Sentinel",
            "Dismember",
        ],
    );
    assert_eq!(plan::keep(&g, P0), Some(false));
}

#[test]
fn power_two_sisay_fetches_mount_doom_the_only_legend_she_can_reach() {
    let mut g = tbl();
    lib(&mut g, &["Derevi, Empyrial Tactician", "Mount Doom", "Kinnan, Bonder Prodigy", "Emiel the Blessed"]);
    let c = legends::sisay_pick(&g, P0, 2).unwrap();
    assert_eq!(name(&g, c), "Mount Doom");
}

#[test]
fn power_five_with_the_loop_pieces_out_fetches_the_missing_finisher() {
    let mut g = tbl();
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    perm(&mut g, P0, "Emiel the Blessed");
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lib(&mut g, &["Lavinia, Azorius Renegade", "Kinnan, Bonder Prodigy", "Mount Doom"]);
    let c = legends::sisay_pick(&g, P0, 5).unwrap();
    assert_eq!(name(&g, c), "Mount Doom");
}

#[test]
fn power_five_with_nothing_out_fetches_a_loop_piece_not_a_value_legend() {
    let mut g = tbl();
    lib(&mut g, &["Lavinia, Azorius Renegade", "Esper Sentinel", "Emiel the Blessed", "Derevi, Empyrial Tactician"]);
    let c = legends::sisay_pick(&g, P0, 5).unwrap();
    assert!(["Derevi, Empyrial Tactician", "Emiel the Blessed"].contains(&name(&g, c).as_str()));
}

#[test]
fn the_wish_list_asks_tutors_for_the_line_and_skips_cards_in_hand() {
    let mut g = tbl();
    hand(&mut g, P0, &["Derevi, Empyrial Tactician"]);
    lib(&mut g, &["Lavinia, Azorius Renegade", "Emiel the Blessed", "Bloom Tender", "Mount Doom"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    let names: Vec<String> = w.iter().map(|&c| name(&g, c)).collect();
    assert!(!names.contains(&"Derevi, Empyrial Tactician".to_string()));
    assert_eq!(names[0], "Emiel the Blessed");
    assert!(names.contains(&"Bloom Tender".to_string()) && names.contains(&"Mount Doom".to_string()));
}

#[test]
fn other_decks_do_not_use_the_sisay_pick() {
    // a non-Tier-6 player keeps the old pick (the dearest legendary creature, no lands)
    let mut g = table(&["seph", SISAY, "sauron", "kefka"]);
    g.active = Some(P0);
    let v: Vec<_> = ["Mount Doom", "Lavinia, Azorius Renegade"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = v;
    let c = legends::sisay_pick(&g, P0, 5).unwrap();
    assert_eq!(name(&g, c), "Lavinia, Azorius Renegade");
}
