//! Practice mode: Sephiroth, Fabled SOLDIER's enter/attack trigger for the human seat. "You may sacrifice another
//! creature. If you do, draw a card." (any number once it is One-Winged Angel). The AI's choice is in cards_seph_triad.rs.

mod play_common;
use play_common::*;

const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

#[test]
fn you_choose_a_creature_to_sacrifice_and_draw() {
    let mut g = table(&["sephfs", "seph"]);
    let t = token(&mut g, P0, 1);
    let h = seat(&mut g, P0, vec![pick(0)]);
    let (hand0, l1) = (g.player(P0).hand.len(), g.player(P1).life);
    perm(&mut g, P0, SEPH);
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Sephiroth: you may sacrifice")), "you were asked");
    assert!(!g.perm(t).on_bf, "the token was sacrificed");
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "and a card drawn");
    assert_eq!(g.player(P1).life, l1 - 1, "its death drains 1");
}

#[test]
fn you_may_sacrifice_nothing() {
    let mut g = table(&["sephfs", "seph"]);
    let t = token(&mut g, P0, 1);
    seat(&mut g, P0, vec![pick(1)]); // the last choice: don't sacrifice
    let hand0 = g.player(P0).hand.len();
    perm(&mut g, P0, SEPH);
    assert!(g.perm(t).on_bf);
    assert_eq!(g.player(P0).hand.len(), hand0);
}

#[test]
fn you_may_sacrifice_your_commander_or_a_good_creature() {
    // the AI never would; the person decides
    let mut g = table(&["sephfs", "seph"]);
    let sheo = perm(&mut g, P0, "Sheoldred, the Apocalypse");
    seat(&mut g, P0, vec![pick(0)]);
    let hand0 = g.player(P0).hand.len();
    perm(&mut g, P0, SEPH);
    assert!(!g.perm(sheo).on_bf);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(sim_core::state::DataKey::Flipped)
}

#[test]
fn your_sephiroth_transforms_when_the_fourth_other_creature_dies() {
    let mut g = table(&["sephfs", "seph"]);
    seat(&mut g, P0, vec![]); // nothing to sacrifice when he enters: no question
    let s = perm(&mut g, P0, SEPH);
    let toks: Vec<PermId> = (0..4).map(|_| token(&mut g, P1, 1)).collect();
    for (i, &t) in toks.iter().enumerate() {
        assert!(!flipped(&g, s), "not yet, after {i} deaths");
        sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    }
    assert!(flipped(&g, s), "the fourth death transforms him");
    assert!(g.player(P0).emblems.iter().any(|&e| e == "seph"), "and you get the emblem");
}

#[test]
fn your_sephiroth_transforms_after_a_board_wipe_that_he_survives() {
    let mut g = table(&["sephfs", "seph"]);
    let h = seat(&mut g, P0, vec![]); // no question: the four drains are the same trigger, so no ordering
    let s = perm(&mut g, P0, SEPH);
    g.perm_mut(s).data.set(sim_core::state::DataKey::Indestr, sim_core::state::Val::Bool(true));
    for _ in 0..4 {
        token(&mut g, P1, 1);
    }
    sim_core::engine::removal::apply_wipe(&mut g, P1, "destroy".into(), None, &Default::default()).unwrap();
    assert!(g.perm(s).on_bf);
    assert!(h.asked().is_empty(), "not asked to order identical triggers");
    assert!(flipped(&g, s), "four creatures died together");
}
