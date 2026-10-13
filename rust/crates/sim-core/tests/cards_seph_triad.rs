//! Sephiroth, Fabled SOLDIER // One-Winged Angel (impls/sheoldred.rs) and Triple Triad (impls/mine.rs) on hand-built
//! positions. Seat 0 plays the sheoldred deck, seat 1 the seph deck.

use sim_core::engine::{turn, values, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

fn life(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// a Seph on the battlefield with nothing else to sacrifice
fn seph(g: &mut Game) -> PermId {
    perm(g, P0, SEPH)
}

#[test]
fn enters_and_sacrifices_a_token_to_draw_and_drain() {
    let mut g = table(&["sheoldred", "seph"]);
    let t = token(&mut g, P0, 1);
    let (l0, h0) = (life(&g, P1), g.player(P0).hand.len());
    let s = seph(&mut g);
    assert!(!g.perm(t).on_bf, "the token was sacrificed");
    assert_eq!(g.player(P0).hand.len(), h0 + 1, "and a card drawn");
    assert_eq!((life(&g, P1), life(&g, P0)), (l0 - 1, 40 + 1), "its death is a drain of 1");
    assert!(!flipped(&g, s));
    assert_eq!(values::epow(&g, s), 3);
}

#[test]
fn does_not_sacrifice_the_commander_or_a_valuable_creature() {
    let mut g = table(&["sheoldred", "seph"]);
    let sheo = perm(&mut g, P0, "Sheoldred, the Apocalypse");
    let h0 = g.player(P0).hand.len();
    seph(&mut g);
    assert!(g.perm(sheo).on_bf);
    assert_eq!(g.player(P0).hand.len(), h0, "nothing sacrificed, nothing drawn");
}

#[test]
fn fourth_drain_in_a_turn_transforms_it_and_gives_an_emblem() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = seph(&mut g);
    let l0 = life(&g, P1);
    for i in 0..4 {
        let t = token(&mut g, P1, 1); // any creature dying counts
        assert!(!flipped(&g, s), "not yet after {i} drains");
        zones::die(&mut g, t, "sba".into()).unwrap();
    }
    assert!(flipped(&g, s));
    assert_eq!(life(&g, P1), l0 - 4);
    assert_eq!((values::epow(&g, s), values::etgh(&g, s)), (5, 5));
    assert!(g.player(P0).emblems.contains(&"seph"));
    // the emblem: a further death drains once (the back face has no drain of its own)
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!(life(&g, P1), l0 - 5);
    // and it outlives Sephiroth
    zones::die(&mut g, s, "destroy".into()).unwrap();
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!(life(&g, P1), l0 - 7, "Sephiroth's own death, then the next one");
}

#[test]
fn drains_spread_over_turns_do_not_transform_it() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = seph(&mut g);
    for _ in 0..3 {
        let t = token(&mut g, P1, 1);
        zones::die(&mut g, t, "sba".into()).unwrap();
    }
    g.round += 1; // next turn
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert!(!flipped(&g, s));
}

#[test]
fn its_own_death_does_not_drain() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = seph(&mut g);
    let l0 = life(&g, P1);
    zones::die(&mut g, s, "destroy".into()).unwrap();
    assert_eq!(life(&g, P1), l0);
}

#[test]
fn one_winged_angel_attack_sacrifices_all_the_fodder_and_draws_that_many() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = seph(&mut g);
    g.perm_mut(s).data.set(DataKey::Flipped, sim_core::state::Val::Bool(true));
    g.selfpt = true;
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let h0 = g.player(P0).hand.len();
    let f = g.registry.get(card(&g, SEPH)).unwrap().attack.unwrap();
    f(&mut g, s, P0, &[s], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), h0 + 3);
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count(), 1);
}

#[test]
fn triple_triad_plays_your_card_and_the_cheaper_ones_free_and_keeps_ownership() {
    let mut g = table(&["sheoldred", "seph", "veyran"]);
    perm(&mut g, P0, "Triple Triad");
    let (sheo, ring, hoof) =
        (card(&g, "Sheoldred, the Apocalypse"), card(&g, "Sol Ring"), card(&g, "Craterhoof Behemoth"));
    for (p, c) in [(P0, sheo), (P1, ring), (P2, hoof)] {
        g.player_mut(p).library.retain(|&x| x != c);
        g.player_mut(p).library.push(c);
    }
    turn::upkeep(&mut g, P0).unwrap();
    let bf =
        |g: &Game, p: PlayerId, n: &str| g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == n);
    assert!(bf(&g, P0, "Sheoldred, the Apocalypse"), "your own card is played free");
    assert!(bf(&g, P0, "Sol Ring"), "a cheaper opposing card too");
    assert!(g.player(P2).exile.contains(&hoof), "an equal or greater one stays in exile");
    assert!(!bf(&g, P0, "Craterhoof Behemoth"));
    let ring_perm = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Sol Ring").unwrap();
    assert_eq!(g.perm(ring_perm).orig, P1, "Sol Ring is still P1's card");
    zones::die(&mut g, ring_perm, "destroy".into()).unwrap();
    assert!(g.player(P1).gy.contains(&ring), "and goes to P1's graveyard");
}
