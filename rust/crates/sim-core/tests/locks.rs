//! Engine-level test of the player cast locks and split second (engine/cast.rs: `add_cast_lock`, `spell_locked`,
//! `abilities_locked`, `split_second_up`).

use sim_core::engine::cast::{self, add_cast_lock, castable};
use sim_core::ids::PlayerId;
use sim_core::state::{LockEnd, LockKind};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn each_lock_kind_stops_what_it_says_until_its_end() {
    let mut g = table(&["sheoldred", "seph"]);
    let (bolt, titan, ring) = (card(&g, "Lightning Bolt"), card(&g, "Grave Titan"), card(&g, "Sol Ring"));
    assert!(castable(&g, P1, bolt, "hand"));
    cast::lock_this_turn(&mut g, P1, LockKind::Noncreature);
    assert!(!castable(&g, P1, bolt, "hand") && !castable(&g, P1, ring, "hand"));
    assert!(castable(&g, P1, titan, "hand"));
    assert!(castable(&g, P0, bolt, "hand"), "other players are not locked");
    cast::lock_this_turn(&mut g, P1, LockKind::AllSpells);
    assert!(!castable(&g, P1, titan, "hand"));
    assert!(!cast::abilities_locked(&g, P1), "only Abeyance's kind stops abilities");
    g.round += 1;
    assert!(castable(&g, P1, titan, "hand") && castable(&g, P1, bolt, "hand"), "the turn ended");
    // until a player's next turn (Hope of Ghirapur)
    let t = g.player(P0).turns;
    add_cast_lock(&mut g, P1, LockKind::Noncreature, LockEnd::TurnOf(P0, t));
    g.round += 1;
    assert!(!castable(&g, P1, bolt, "hand"), "still locked on a later turn");
    g.player_mut(P0).turns += 1;
    assert!(castable(&g, P1, bolt, "hand"), "P0's next turn begins");
}

#[test]
fn locks_off_means_the_cheap_gate_stays_closed() {
    let g = table(&["sheoldred", "seph"]);
    assert!(!g.locks_used);
}
