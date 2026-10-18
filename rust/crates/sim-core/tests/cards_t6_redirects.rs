//! Tier 6 batch `redirects` (impls/t6_redirects.rs) on hand-built positions. Seat 0 plays a stand-in deck (the cards
//! are keyed by name), seat 1 the seph deck.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t6_redirects as t6;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P1);
    g
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn untapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

// ======================================================== Redirect Lightning
#[test]
fn redirect_lightning_sends_removal_back_at_the_caster_paying_mana() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    let life = g.player(P0).life;
    let done = t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap();
    assert!(done);
    assert!(g.perm(mine).on_bf, "our creature survives");
    assert!(!g.perm(theirs).on_bf, "the caster's creature takes the removal");
    assert!(in_gy(&g, P0, "Redirect Lightning"));
    assert_eq!(untapped(&g, P0), 0, "{{R}} and {{2}} were paid");
    assert_eq!(g.player(P0).life, life);
}

#[test]
fn redirect_lightning_pays_five_life_when_short_of_mana() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    let life = g.player(P0).life;
    assert!(t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(g.perm(mine).on_bf && !g.perm(theirs).on_bf);
    assert_eq!(g.player(P0).life, life - 5);
}

#[test]
fn redirect_lightning_needs_a_target_and_a_cost_and_ignores_own_spells() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    // the caster controls nothing to aim it at
    assert!(!t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(in_hand(&g, P0, "Redirect Lightning"));
    // low on life and mana: no way to pay
    let _t = perm(&mut g, P1, "Grave Titan");
    g.player_mut(P0).life = 8;
    assert!(!t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(on_bf(&g, P0, "Grave Titan").is_some());
}
