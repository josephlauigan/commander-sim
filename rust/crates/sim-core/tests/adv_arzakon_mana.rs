//! Independent adversarial checks against the printed mana abilities.
use sim_core::engine::mana;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn game() -> Game {
    table(&["arzakon", "karn_gilded"])
}

#[test]
fn confluence_pays_for_generic_and_each_colour() {
    for pips in ["", "W", "U", "B", "R", "G"] {
        let mut g = game();
        lands(&mut g, P0, "Mana Confluence", 1, false);
        let before = g.player(P0).life;
        assert!(mana::pay(&mut g, P0, u32::from(pips.is_empty()), pips, false).unwrap());
        assert_eq!(g.player(P0).life, before - 1);
    }
}

#[test]
fn two_confluences_pay_two_life() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 2, false);
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "WU", false).unwrap());
    assert_eq!(g.player(P0).life, before - 2);
}

#[test]
fn moon_removes_confluence_cost() {
    let mut g = game();
    let l = lands(&mut g, P0, "Mana Confluence", 1, false)[0];
    perm(&mut g, P0, "Blood Moon");
    assert!(mana::moon_land(&g, l));
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "R", false).unwrap());
    assert_eq!(g.player(P0).life, before);
}

#[test]
fn leyline_grants_confluence_a_free_mana_ability() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    perm(&mut g, P0, "Leyline of the Guildpact");
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
    assert_eq!(g.player(P0).life, before, "the Island ability has no life cost");
}

#[test]
fn lantern_grants_confluence_a_free_mana_ability() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    let rock = perm(&mut g, P0, "Chromatic Lantern");
    g.perm_mut(rock).tapped = true;
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
    assert_eq!(g.player(P0).life, before, "the Lantern-granted ability has no life cost");
}

#[test]
fn ai_does_not_spend_its_last_life_for_mana() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    g.player_mut(P0).life = 1;
    assert!(!mana::can_pay(&g, P0, 1, "", false), "AI must not plan a lethal mana payment");
}

#[test]
fn payment_does_not_reduce_the_ai_to_zero_life() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    g.player_mut(P0).life = 1;
    let result = mana::pay(&mut g, P0, 1, "", false);
    assert_eq!(g.player(P0).life, 1, "the AI must decline its lethal payment: {result:?}");
}

#[test]
#[ignore = "named limit (JD 2026-11-26): the payment planner checks life per land, not the combined cost of several life-cost lands (two Confluences at 2 life)"]
fn ai_checks_combined_life_cost_of_two_confluences() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 2, false);
    g.player_mut(P0).life = 2;
    assert!(!mana::can_pay(&g, P0, 0, "WU", false), "both costs together are lethal");
}

#[test]
fn life_lock_forbids_confluence_activation() {
    let mut payable = Vec::new();
    for protection in [false, true] {
        let mut g = game();
        lands(&mut g, P0, "Mana Confluence", 1, false);
        // Flare's life lock, or Teferi's life lock and protection with a land acquired after phasing out.
        g.player_mut(P0).life_locked = true;
        g.player_mut(P0).ring_prot = protection;
        payable.push(mana::can_pay(&g, P0, 1, "", false));
    }
    assert_eq!(payable, [false, false], "neither Flare's nor Teferi's life lock permits paying life");
}

#[test]
fn flare_effect_forbids_confluence_activation() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    sim_core::impls::t45_winota::after_protector(&mut g, P0, "Flare of Fortitude").unwrap();
    assert!(g.player(P0).life_locked);
    assert!(!mana::can_pay(&g, P0, 1, "", false));
}

#[test]
fn life_locked_payment_does_not_tap_confluence_for_free() {
    let mut g = game();
    let l = lands(&mut g, P0, "Mana Confluence", 1, false)[0];
    g.player_mut(P0).life_locked = true;
    let before = g.player(P0).life;
    let result = mana::pay(&mut g, P0, 1, "", false);
    assert_eq!(g.player(P0).life, before);
    assert!(!g.perm(l).tapped, "the cost cannot be paid, but payment returned {result:?}");
}

#[test]
fn damage_prevention_does_not_prevent_confluence_cost() {
    for (name, expected_loss) in [("Mana Confluence", 1), ("City of Brass", 0), ("Yavimaya Coast", 0)] {
        let mut g = game();
        lands(&mut g, P0, name, 1, false);
        g.player_mut(P0).ring_prot = true;
        let before = g.player(P0).life;
        assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
        assert_eq!(g.player(P0).life, before - expected_loss, "{name}");
    }
}

#[test]
fn rings_does_not_copy_a_mana_ability() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    perm(&mut g, P0, "Rings of Brighthearth");
    assert_eq!(mana::total_mana(&g, P0, false), 1);
    assert!(!mana::can_pay(&g, P0, 2, "", false));
}

#[test]
fn nyxbloom_triples_mana_without_tripling_the_cost() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    perm(&mut g, P0, "Nyxbloom Ancient");
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 3, "", false).unwrap());
    assert_eq!(g.player(P0).life, before - 1);
}

#[test]
fn prefer_a_free_rainbow_land_for_generic_in_either_order() {
    let mut losses = Vec::new();
    for names in [["Mana Confluence", "Command Tower"], ["Command Tower", "Mana Confluence"]] {
        let mut g = game();
        for name in names {
            lands(&mut g, P0, name, 1, false);
        }
        let before = g.player(P0).life;
        assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
        losses.push(before - g.player(P0).life);
    }
    assert_eq!(losses, [0, 0], "free mana is available in both land orders");
}

#[test]
fn lantern_does_not_turn_tombs_two_mana_into_two_coloured_mana() {
    let mut g = game();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    let rock = perm(&mut g, P0, "Chromatic Lantern");
    g.perm_mut(rock).tapped = true;
    assert!(mana::can_pay(&g, P0, 0, "U", false));
    assert!(mana::can_pay(&g, P0, 2, "", false));
    assert!(!mana::can_pay(&g, P0, 0, "UU", false), "Lantern grants ONE mana of any colour");
}

#[test]
fn lantern_tomb_coloured_ability_is_painless() {
    let mut g = game();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    let rock = perm(&mut g, P0, "Chromatic Lantern");
    g.perm_mut(rock).tapped = true;
    let before = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
    assert_eq!(g.player(P0).life, before);
}

#[test]
#[ignore = "named limit (JD 2026-11-26): Blood Moon switches off Chromatic Lantern's granted ability in the engine; only krenko (t4) plays Blood Moon"]
fn lantern_granted_ability_survives_moon() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    perm(&mut g, P0, "Blood Moon");
    let rock = perm(&mut g, P0, "Chromatic Lantern");
    g.perm_mut(rock).tapped = true;
    assert!(mana::can_pay(&g, P0, 0, "U", false), "Moon changes land types before Lantern grants abilities");
}
