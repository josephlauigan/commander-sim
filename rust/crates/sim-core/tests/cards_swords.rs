//! The Swords whose triggers card_overrides.toml completes with the ability language (Forge and Frontier: `impulse`,
//! `extra_land`; Once and Future: `free_gy_spell`; Body and Mind; War and Peace: `event_player_hand`), on hand-built
//! positions.

use sim_core::dsl;
use sim_core::engine::values::{epow, etgh};
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

#[test]
fn forge_and_frontier_cards_not_played_are_exiled_at_the_end_step() {
    let (mut g, a) = equipped("Sword of Forge and Frontier");
    let mut lib = vec![card(&g, "Kitchen Finks"); 20];
    lib.extend([card(&g, "Swords to Plowshares"), card(&g, "Kitchen Finks")]);
    g.player_mut(P0).library = lib;
    g.player_mut(P0).hand.clear();
    hit(&mut g, a);
    assert_eq!(g.player(P0).hand.len(), 2, "both exiled cards are playable this turn");
    assert_eq!(g.player(P0).impulse.len(), 2);
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).hand.is_empty(), "the ones not played go to exile");
    assert_eq!(g.player(P0).exile.len(), 2);
}

#[test]
fn body_and_mind_a_wolf_and_ten_milled() {
    let (mut g, a) = equipped("Sword of Body and Mind");
    let lib = vec![card(&g, "Plains"); 30];
    g.player_mut(P1).library = lib;
    let gy = g.player(P1).gy.len();
    let toks = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    hit(&mut g, a);
    assert_eq!(g.player(P1).library.len(), 20);
    assert_eq!(g.player(P1).gy.len(), gy + 10);
    let wolves: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(wolves.len(), toks + 1);
    let w = *wolves.last().unwrap();
    assert_eq!((epow(&g, w), etgh(&g, w)), (2, 2));
}

#[test]
fn war_and_peace_their_hand_hurts_yours_heals() {
    let (mut g, a) = equipped("Sword of War and Peace");
    let theirs = vec![card(&g, "Plains"); 5];
    g.player_mut(P1).hand = theirs;
    let mine = vec![card(&g, "Plains"); 3];
    g.player_mut(P0).hand = mine;
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    hit(&mut g, a);
    assert_eq!(g.player(P1).life, l1 - 5);
    assert_eq!(g.player(P0).life, l0 + 3);
}

#[test]
fn sinew_and_steel_destroys_a_planeswalker_and_an_artifact() {
    let (mut g, a) = equipped("Sword of Sinew and Steel");
    let rock = perm(&mut g, P1, "Sol Ring");
    let mine = perm(&mut g, P0, "Sol Ring");
    hit(&mut g, a);
    assert!(!g.perm(rock).on_bf, "the opponent's artifact is destroyed");
    assert!(g.perm(mine).on_bf, "not your own");
}
