//! Sword of Forge and Frontier and Sword of Once and Future (card_overrides.toml: the ability language's
//! `extra_land` and `free_gy_spell`): their combat-damage triggers on hand-built positions.

use sim_core::dsl;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// a Kitchen Finks wearing `sword`, on P0's turn
fn equipped(sword: &str) -> (Game, PermId) {
    let mut g = table(&["cloud", "seph"]);
    let a = perm(&mut g, P0, "Kitchen Finks");
    let s = perm(&mut g, P0, sword);
    g.perm_mut(s).attached = Some(a);
    g.active = Some(P0);
    (g, a)
}

fn hit(g: &mut Game, a: PermId) {
    dsl::fire(g, "combat_damage", dsl::Fired { attacker: Some(a), defender: Some(P1), ..Default::default() }).unwrap();
}

#[test]
fn forge_and_frontier_two_cards_and_an_extra_land() {
    let (mut g, a) = equipped("Sword of Forge and Frontier");
    // a long library: optional land drops stop short of decking you (more_lands)
    let mut lib = vec![card(&g, "Kitchen Finks"); 20];
    lib.extend([card(&g, "Swords to Plowshares"), card(&g, "Plains")]); // the top is the end
    g.player_mut(P0).library = lib;
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).lands_played = 1; // the turn's land is already down
    let lands = g.player(P0).lands.len();
    hit(&mut g, a);
    assert_eq!(g.player(P0).lands.len(), lands + 1, "the extra land drop plays the Plains it found");
    assert_eq!(g.player(P0).library.len(), 20);
    let hand: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert_eq!(hand, ["Swords to Plowshares"]);
}

#[test]
fn once_and_future_casts_a_cheap_spell_from_the_graveyard_and_exiles_it() {
    let (mut g, a) = equipped("Sword of Once and Future");
    let target = perm(&mut g, P1, "Kitchen Finks");
    let stp = card(&g, "Swords to Plowshares");
    g.player_mut(P0).gy = vec![stp];
    let lib = vec![card(&g, "Plains"), card(&g, "Plains"), card(&g, "Plains")];
    g.player_mut(P0).library = lib;
    hit(&mut g, a);
    assert!(!g.perm(target).on_bf, "Swords to Plowshares, cast free, exiled the opposing creature");
    assert!(!g.player(P0).gy.contains(&stp), "the spell is exiled instead of going back to the graveyard");
    assert!(g.player(P0).exile.contains(&stp));
}
