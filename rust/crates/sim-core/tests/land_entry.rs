//! Lands that enter tapped unless a condition holds, by their Oracle text: Python's tests/test_land_entry.py (the
//! AI's side; a person's shock-land choice comes with practice mode).

use sim_core::engine::turn::{land_enters_tapped, play_land_card};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tapped(g: &Game, p: PlayerId, name: &str) -> bool {
    land_enters_tapped(g, p, card(g, name))
}

#[test]
fn check_land() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Mountain", 1, false);
    assert!(tapped(&g, P0, "Drowned Catacomb")); // needs an Island or a Swamp
    lands(&mut g, P0, "Island", 1, false);
    assert!(!tapped(&g, P0, "Drowned Catacomb"));
    lands(&mut g, P0, "Swamp", 3, false);
    assert!(!tapped(&g, P0, "Drowned Catacomb")); // however many lands
    assert!(tapped(&g, P0, "Sunpetal Grove")); // Forest or Plains: none
}

#[test]
fn check_land_counts_dual_types() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Steam Vents", 1, false); // Land — Island Mountain
    assert!(!tapped(&g, P0, "Drowned Catacomb"));
}

#[test]
fn fast_land() {
    let mut g = table(&["veyran", "sauron"]);
    lands(&mut g, P0, "Island", 2, false);
    assert!(!tapped(&g, P0, "Spirebluff Canal")); // two or fewer other lands
    lands(&mut g, P0, "Island", 1, false);
    assert!(tapped(&g, P0, "Spirebluff Canal"));
}

#[test]
fn slow_land() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(tapped(&g, P0, "Haunted Ridge")); // two or more other lands
    lands(&mut g, P0, "Mountain", 1, false);
    assert!(!tapped(&g, P0, "Haunted Ridge"));
}

#[test]
fn snarl_reveals_from_hand() {
    let mut g = table(&["sauron", "veyran"]);
    assert!(tapped(&g, P0, "Foreboding Ruins"));
    hand(&mut g, P0, &["Swamp"]);
    assert!(!tapped(&g, P0, "Foreboding Ruins"));
}

#[test]
fn battle_bond_and_sanctuary() {
    let mut g = table(&["sauron", "veyran", "seph", "jodah"]);
    assert!(!tapped(&g, P0, "Luxury Suite")); // three opponents
    lands(&mut g, P0, "Island", 1, false);
    assert!(tapped(&g, P0, "Sunken Hollow")); // two or more basic lands
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(!tapped(&g, P0, "Sunken Hollow"));
    assert!(tapped(&g, P0, "Mystic Sanctuary")); // three or more other Islands
    lands(&mut g, P0, "Island", 2, false);
    assert!(!tapped(&g, P0, "Mystic Sanctuary"));
}

#[test]
fn plain_tapped_and_untapped_lands() {
    let g = table(&["sauron", "veyran"]);
    assert!(tapped(&g, P0, "Crumbling Necropolis"));
    assert!(!tapped(&g, P0, "Command Tower"));
}

#[test]
fn the_ai_pays_for_a_shock_land_only_when_it_uses_the_mana() {
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).turns = 6;
    lands(&mut g, P0, "Island", 1, false);
    let cs = card(&g, "Counterspell");
    g.player_mut(P0).hand.push(cs); // two mana: needs the shock untapped
    let vents = card(&g, "Steam Vents");
    play_land_card(&mut g, P0, vents, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    assert!(!g.land(l).tapped);
    assert_eq!(g.player(P0).life, 38);
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).turns = 6;
    g.player_mut(P0).hand.clear(); // nothing to cast: save the life
    play_land_card(&mut g, P0, vents, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.land(l).tapped);
    assert_eq!(g.player(P0).life, 40);
}
