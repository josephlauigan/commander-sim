//! Why casual games ran long (rust/agents/casual-speed.md): the fixes, each on a hand-built position.

use sim_core::engine::turn::play_land;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn land_names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// Selesnya Sanctuary: "When this land enters, return a land you control to its owner's hand." With no other land
/// it returns itself, so a first land drop of a bounceland is lost (and was lost every turn: the AI chose it again
/// over the Forest in hand, 0 lands on turn 5 in 13 of 100 Tier 0 games). The Forest goes first.
#[test]
fn bounceland_waits_for_another_land() {
    let mut g = table(&["anikthea-enduring-enchantments", "giada-calling-all-angels"]);
    hand(&mut g, P0, &["Selesnya Sanctuary", "Forest"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Forest"]);
    assert!(in_hand(&g, P0, "Selesnya Sanctuary"));
}

/// with another land out the bounceland is played as before: it returns that land
#[test]
fn bounceland_after_a_land() {
    let mut g = table(&["anikthea-enduring-enchantments", "giada-calling-all-angels"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Selesnya Sanctuary"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Selesnya Sanctuary"]);
    assert!(in_hand(&g, P0, "Plains"));
}

/// Lotus Field: "When Lotus Field enters, sacrifice two untapped lands." With no other land it sacrifices itself:
/// the Swamp goes first
#[test]
fn lotus_field_waits_for_two_lands() {
    let mut g = table(&["lord-windgrace-jund-lands", "giada-calling-all-angels"]);
    hand(&mut g, P0, &["Lotus Field", "Swamp"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Swamp"]);
    assert!(in_hand(&g, P0, "Lotus Field"));
}
