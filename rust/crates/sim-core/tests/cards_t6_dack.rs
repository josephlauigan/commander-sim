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

// ======================================================== Touch the Spirit Realm
#[test]
fn touch_exiles_an_opposing_creature_until_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let touch = perm(&mut g, P0, "Touch the Spirit Realm");
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, touch, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some());
}

#[test]
fn touch_channel_returns_the_target_at_the_end_step_under_its_owner() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let titan = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Touch the Spirit Realm"])[0];
    assert!(sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, titan).unwrap());
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    assert!(g.player(P0).gy.contains(&c), "the card was discarded as the cost");
    sim_core::impls::galadriel::eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns under its owner's control at the end step");
}

#[test]
fn touch_channel_needs_its_mana_and_a_legal_target() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Touch the Spirit Realm"])[0];
    assert!(!sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, titan).unwrap(), "no mana");
    lands(&mut g, P0, "Plains", 2, false);
    let land_like = token(&mut g, P1, 1);
    sim_core::engine::zones::die(&mut g, land_like, "test").unwrap();
    assert!(!sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, land_like).unwrap(), "target is gone");
}
