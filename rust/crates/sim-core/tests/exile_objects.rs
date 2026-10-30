//! Phase 3 PR 5: exiled cards are objects (`engine/exile.rs`, design in `audit/phase3/exile-design.md`). A card that
//! leaves exile and is exiled again is a new object (rule 400.7): "until ~ leaves" returns only the object it exiled,
//! a silver counter stays with its object, and "you may play it this turn" ends with that turn.

use sim_core::cardcode;
use sim_core::engine::exile::{ExileId, ExileZone};
use sim_core::engine::turn;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).name == name)
}

#[test]
fn every_card_put_into_exile_is_a_new_object() {
    let (a, b) = (CardId(1), CardId(2));
    let mut z = ExileZone::default();
    z.push(a);
    z.push(b);
    let first = z.newest(a).unwrap();
    assert!(z.take(a));
    assert_eq!(z.find(first), None, "it left exile: the object is gone");
    z.push(a);
    let second = z.newest(a).unwrap();
    assert_ne!(first, second, "the same card exiled again is a new object");
    assert_eq!(z, vec![b, a]);
    assert_eq!(z.objs().iter().map(|o| o.id).collect::<Vec<_>>(), [ExileId(1), second]);
}

#[test]
fn until_leaves_returns_the_object_it_exiled() {
    let mut g = table(&["brago-azorius-blink-control", "kaalia-mardu-creature-cheat"]);
    perm(&mut g, P1, "Baneslayer Angel");
    let j = perm(&mut g, P0, "Palace Jailer");
    let c = card(&g, "Baneslayer Angel");
    assert!(g.player(P1).exile.contains(&c));
    let i = g.player(P1).exile.iter().position(|&x| x == c).unwrap();
    assert_eq!(g.player(P1).exile.obj(i).until, Some(j), "linked to the Jailer");
    let Some(Val::List(held)) = g.perm(j).data.get(DataKey::Oring) else { panic!("the Jailer holds it") };
    assert!(matches!(&held[0], Val::List(e) if e.len() == 3), "[card, owner, exile id]");
    // the card leaves exile (cast from exile, say) and something else exiles it again
    assert!(g.player_mut(P1).exile.take(c));
    g.player_mut(P1).exile.push(c);
    cardcode::become_monarch(&mut g, P1).unwrap();
    assert!(named(&g, P1, "Baneslayer Angel").is_none(), "a new object: the Jailer's exile ended with the old one");
    assert!(g.player(P1).exile.contains(&c));
}

#[test]
fn until_leaves_still_returns_the_card_it_exiled() {
    let mut g = table(&["brago-azorius-blink-control", "kaalia-mardu-creature-cheat"]);
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P0, "Palace Jailer");
    // another copy of the card exiled after it doesn't change which one returns
    let c = card(&g, "Baneslayer Angel");
    g.player_mut(P1).exile.push(c);
    cardcode::become_monarch(&mut g, P1).unwrap();
    assert!(named(&g, P1, "Baneslayer Angel").is_some(), "it returns");
    assert_eq!(g.player(P1).exile, vec![c], "the later one stays");
    assert_eq!(g.player(P1).exile.obj(0).until, None);
}

fn karn_opts(g: &mut Game, k: PermId) -> Vec<Opt> {
    let c = g.perm(k).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, k, P0, Some(false)).unwrap()
}

#[test]
fn a_silver_counter_stays_with_its_object() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    let c = take(&mut g, P0, "Wurmcoil Engine");
    g.player_mut(P0).exile.push(c);
    g.player_mut(P0).exile.last_mut().unwrap().silver = true;
    assert_eq!(g.player(P0).exile.silver(), [c]);
    assert!(karn_opts(&mut g, k).iter().any(|o| o.label.contains("-1")), "-1 has a silver card");
    // it leaves exile and comes back: a new object, without the counter
    assert!(g.player_mut(P0).exile.take(c));
    g.player_mut(P0).exile.push(c);
    assert!(g.player(P0).exile.silver().is_empty());
    assert!(karn_opts(&mut g, k).iter().all(|o| !o.label.contains("-1")), "nothing to return");
}

#[test]
fn karn_minus_one_takes_the_object_with_the_counter() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    let c = take(&mut g, P0, "Wurmcoil Engine");
    g.player_mut(P0).exile.push(c); // an older copy without a counter
    g.player_mut(P0).exile.push(c);
    g.player_mut(P0).exile.last_mut().unwrap().silver = true;
    let m1 = karn_opts(&mut g, k).into_iter().find(|o| o.label.contains("-1")).expect("-1");
    assert!(sim_core::ai::act::perform(&mut g, P0, m1.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).exile, vec![c], "the copy without a counter stays");
    assert!(g.player(P0).exile.silver().is_empty());
    assert_eq!(g.player(P0).hand.last(), Some(&c));
}

#[test]
fn impulse_draw_ends_with_the_turn_it_was_given_in() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    // P1 exiles a card it may play this turn, on P0's turn
    let c = take(&mut g, P1, "Swamp");
    g.player_mut(P1).hand.push(c);
    g.player_mut(P1).impulse.push(c);
    turn::end_step(&mut g, P0).unwrap();
    assert!(!g.player(P1).hand.contains(&c), "this turn is over: it can't be played on P1's turn");
    assert!(g.player(P1).impulse.is_empty());
    assert!(g.player(P1).exile.contains(&c));
}
