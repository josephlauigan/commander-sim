//! The mulligan hint for Godo, Bandit Warlord (a Tier 6 deck with its own keep rule, `t6_plan_godo::keep`): the
//! reasons describe that rule (fast mana, a path to victory), not the generic "2 to 5 lands" one, and they agree with
//! the verdict.

use practice::advisor::mulligan;
use sim_core::engine::turn::keeps_hand;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn godo_hand(names: &[&str]) -> sim_core::state::Game {
    let mut g = table(&["godo", "sauron"]);
    let old: Vec<_> = std::mem::take(&mut g.player_mut(P0).hand);
    g.player_mut(P0).library.extend(old);
    hand(&mut g, P0, names);
    g
}

#[test]
fn godo_keeps_mana_with_a_path_to_victory_and_says_why() {
    let mut g = godo_hand(&["Mountain", "Mountain", "Ancient Tomb", "Sol Ring", "Helm of the Host", "Goblin Welder", "Grim Monolith"]);
    assert!(keeps_hand(&g, P0));
    let v = mulligan::hint(&mut g, P0, "Your opening hand (7 to keep).", 0);
    assert_eq!(v["choice"], "keep", "{v}");
    let detail = v["detail"].to_string();
    assert!(detail.contains("Godo's rule"), "{detail}");
    assert!(detail.contains("Path to victory: 2"), "{detail}");
    assert!(!detail.contains("2 to 5 lands"), "no generic rule: {detail}");
    assert!(!detail.contains("does not look at key cards"), "{detail}");
}

#[test]
fn godo_ships_ramp_without_a_path_to_victory_and_says_why() {
    let mut g = godo_hand(&["Mountain", "Mountain", "Mountain", "Ancient Tomb", "Sol Ring", "Mana Vault", "Grim Monolith"]);
    assert!(!keeps_hand(&g, P0), "the engine's rule ships it");
    let v = mulligan::hint(&mut g, P0, "Your opening hand (7 to keep).", 0);
    assert_eq!(v["choice"], "mulligan", "{v}");
    assert!(v["text"].as_str().unwrap().contains("no path to victory"), "{v}");
    let (keep, why) = mulligan::verdict(&g, P0, "Your opening hand (7 to keep).");
    assert!(!keep && why.contains("Godo's rule"), "{why}");
}

#[test]
fn the_draw_number_comes_from_the_request() {
    let data = serde_json::json!({"mulligans": 1});
    assert_eq!(mulligan::tries_of(&data, "Your opening hand (7 to keep)."), 1);
    assert_eq!(mulligan::tries_of(&serde_json::json!({}), "Your opening hand (6 to keep)."), 2);
    assert_eq!(mulligan::tries_of(&serde_json::json!({}), "Your opening hand (7 to keep)."), 0);
}
