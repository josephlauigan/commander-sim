//! Tier 6 batch `dack` (impls/t6_dack.rs) on hand-built positions. Seat 0 plays Dack's deck, seat 1 the seph deck.

use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["dack-fayden-white-stax", "seph"]);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

// ======================================================== Fiend Hunter
#[test]
fn fiend_hunter_exiles_the_best_creature_and_returns_it_when_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let hunter = perm(&mut g, P0, "Fiend Hunter");
    assert!(!g.perm(titan).on_bf, "Grave Titan is exiled");
    assert!(in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, hunter, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns under its owner's control");
    assert!(!in_exile(&g, P1, "Grave Titan"));
}

#[test]
fn fiend_hunter_never_takes_its_own_creature() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Grave Titan");
    let _hunter = perm(&mut g, P0, "Fiend Hunter");
    assert!(g.perm(mine).on_bf);
}

// ======================================================== All-Fates Stalker
#[test]
fn stalker_exiles_a_non_assassin_and_returns_it_when_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let stalker = perm(&mut g, P0, "All-Fates Stalker");
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, stalker, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns at once, under its owner's control");
}

#[test]
fn stalker_leaves_assassins_alone() {
    let mut g = game();
    let other = perm(&mut g, P1, "All-Fates Stalker");
    let _mine = perm(&mut g, P0, "All-Fates Stalker");
    assert!(g.perm(other).on_bf, "an Assassin is not a legal target");
}
