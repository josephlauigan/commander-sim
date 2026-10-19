//! Task `windows`: an equip (or any activation, or Helm of the Host's trigger) of a Tier 6 player that works on a piece
//! of a line already on the battlefield is a win step: opponents may answer it with instant-speed removal. Seat 0
//! holds the answers; seat 1 plays the Godo deck.

use sim_core::engine::stack::equip_to;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn setup(answer: Option<&str>, mana: usize) -> (Game, PermId, PermId) {
    let mut g = table(&["seph", "godo"]);
    g.active = Some(P1);
    lands(&mut g, P0, "Plains", mana, false);
    if let Some(a) = answer {
        hand(&mut g, P0, &[a]);
    }
    lands(&mut g, P1, "Mountain", 5, false);
    let godo = perm(&mut g, P1, "Godo, Bandit Warlord");
    let helm = g.player(P1).perms.iter().copied().find(|&m| &*g.perm(m).name == "Helm of the Host").expect("Helm");
    (g, godo, helm)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn swords_in_response_to_the_equip_stops_it() {
    let (mut g, godo, helm) = setup(Some("Swords to Plowshares"), 1);
    equip_to(&mut g, P1, helm, godo, 5).unwrap();
    assert!(in_gy(&g, P0, "Swords to Plowshares"), "the answer was cast");
    assert!(!g.perm(godo).on_bf, "Godo was exiled");
    assert_eq!(g.perm(helm).attached, None, "the equip did not happen");
}

#[test]
fn no_answer_equip_goes_through() {
    let (mut g, godo, helm) = setup(None, 1);
    equip_to(&mut g, P1, helm, godo, 5).unwrap();
    assert!(g.perm(godo).on_bf);
    assert_eq!(g.perm(helm).attached, Some(godo));
}

#[test]
fn answer_without_mana_cannot_stop_it() {
    let (mut g, godo, helm) = setup(Some("Swords to Plowshares"), 0);
    equip_to(&mut g, P1, helm, godo, 5).unwrap();
    assert_eq!(g.perm(helm).attached, Some(godo));
    assert!(!in_gy(&g, P0, "Swords to Plowshares"));
}

#[test]
fn the_window_is_for_tier_6_players_only() {
    // seat 1 plays a non-Tier-6 deck: no window opens for the equip
    let mut g = table(&["seph", "seph"]);
    g.active = Some(P1);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    lands(&mut g, P1, "Mountain", 5, false);
    let c = perm(&mut g, P1, "Sakura-Tribe Elder");
    let h = perm(&mut g, P1, "Helm of the Host");
    equip_to(&mut g, P1, h, c, 5).unwrap();
    assert_eq!(g.perm(h).attached, Some(c));
    assert!(!in_gy(&g, P0, "Swords to Plowshares"));
}
