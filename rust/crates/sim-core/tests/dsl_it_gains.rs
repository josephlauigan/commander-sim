//! "It gains ... until end of turn" in a compiled ability is about the creature the text names, not every creature
//! you control. Each expectation follows the card's Oracle text.

mod play_common;
use play_common::*;
use sim_core::dsl::{activate, has_kw};

/// Seasoned Hallowblade: "Discard a card: Tap this creature. It gains indestructible until end of turn."
#[test]
fn seasoned_hallowblade_gives_indestructible_to_itself_only() {
    let mut g = table(&["seph", "cloud"]);
    let blade = perm(&mut g, P0, "Seasoned Hallowblade");
    let other = token(&mut g, P0, 2);
    hand(&mut g, P0, &["Plains"]);
    assert!(activate(&mut g, P0, blade, 0).unwrap(), "the ability resolves");
    assert!(g.perm(blade).tapped, "it taps");
    assert!(has_kw(&g, blade, "indestructible"), "the Hallowblade gains indestructible");
    assert!(!has_kw(&g, other, "indestructible"), "no other creature does");
}
