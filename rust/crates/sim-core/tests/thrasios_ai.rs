//! The `thrasios` seat's combo cards (Walking Ballista, Basalt Monolith, Rings of Brighthearth) under the generic
//! policy. The seat's hand AI (protection spells, the combat trick, the combo tutor) was deleted on 2026-11-01.

use sim_core::ai::decks::deck_prio;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    let mut g = table(&["thrasios", "seph", "veyran"]);
    g.active = Some(P0);
    g
}

// ---------------------------------------------------------------- the combo cards
fn prio(g: &Game, name: &str) -> i32 {
    deck_prio(g, P0, card(g, name))
}

#[test]
fn walking_ballista_is_cast_for_value_with_spare_mana_but_held_for_the_combo() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Walking Ballista"]);
    assert_eq!(prio(&g, "Walking Ballista"), 0, "X would be 1 with three mana");
    lands(&mut g, P0, "Forest", 2, false);
    assert!(prio(&g, "Walking Ballista") >= 45, "five mana: X = 2, even with Rings still in the library");
    // Basalt and Rings both in reach: hold the kill
    hand(&mut g, P0, &["Basalt Monolith", "Rings of Brighthearth"]);
    assert_eq!(prio(&g, "Walking Ballista"), 0);
    // ... unless the mana is huge
    lands(&mut g, P0, "Forest", 5, false);
    assert!(prio(&g, "Walking Ballista") > 0);
}

#[test]
fn basalt_is_always_ramp_and_rings_waits_for_a_basalt() {
    let mut g = tbl();
    g.player_mut(P0).turns = 9;
    hand(&mut g, P0, &["Basalt Monolith", "Rings of Brighthearth"]);
    assert!(prio(&g, "Basalt Monolith") >= 70, "cast on any turn");
    let with_basalt = prio(&g, "Rings of Brighthearth");
    let mut g2 = tbl();
    hand(&mut g2, P0, &["Rings of Brighthearth"]);
    assert!(with_basalt >= 70 && with_basalt > prio(&g2, "Rings of Brighthearth"));
}

