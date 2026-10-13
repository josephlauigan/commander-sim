//! Comet Storm (src/impls/threats.rs) against its Oracle text: {X}{R}{R}, multikicker {1}; X damage to each of
//! 1 + kicks targets.

use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

fn game() -> Game {
    let mut g = table(&["sauron", "veyran", "seph", "kefka"]);
    g.active = Some(P0);
    for p in [P1, P2, P3] {
        g.player_mut(p).life = 40;
    }
    g
}

fn storm(g: &mut Game, x: i32, kicks: u32) {
    let c = hand(g, P0, &["Comet Storm"])[0];
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, P0, c, &Ctx { x, kicks, ..Ctx::default() }).unwrap();
}

fn opts(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, P0, post).unwrap()
}

fn run(g: &mut Game, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Plan { f, arg }) => f(g, P0, arg).unwrap(),
        _ => panic!("not a plan"),
    }
}

#[test]
fn unkicked_hits_one_target_for_x() {
    let mut g = game();
    storm(&mut g, 5, 0);
    let lost: Vec<i32> = [P1, P2, P3].iter().map(|&p| 40 - g.player(p).life).collect();
    assert_eq!(lost.iter().sum::<i32>(), 5, "one target, X damage");
}

#[test]
fn each_kick_adds_a_target_taking_x() {
    let mut g = game();
    storm(&mut g, 4, 2);
    let lost: Vec<i32> = [P1, P2, P3].iter().map(|&p| 40 - g.player(p).life).collect();
    assert_eq!(lost.iter().filter(|&&l| l == 4).count(), 3, "three targets (1 + 2 kicks) each take 4: {lost:?}");
}

#[test]
fn damage_goes_to_creatures_it_kills_and_planeswalkers() {
    let mut g = game();
    let t = token(&mut g, P1, 6);
    let pw = perm(&mut g, P2, "Jace, the Mind Sculptor");
    let l = g.perm(pw).loyalty.unwrap_or(0);
    assert!(l > 0);
    storm(&mut g, l.max(6), 3);
    assert!(!g.perm(t).on_bf, "the 6/6 dies");
    assert!(!g.perm(pw).on_bf, "the planeswalker loses all its loyalty");
}

#[test]
fn a_creature_that_survives_is_not_a_target_worth_taking() {
    let mut g = game();
    let t = token(&mut g, P1, 12);
    storm(&mut g, 4, 0);
    assert!(g.perm(t).on_bf);
}

#[test]
fn the_ai_kills_every_opponent_it_can_with_the_kicks_and_x() {
    let mut g = game();
    for p in [P1, P2, P3] {
        g.player_mut(p).life = 4;
    }
    lands(&mut g, P0, "Mountain", 9, false);
    let c = hand(&mut g, P0, &["Comet Storm"])[0];
    let o = opts(&mut g, c, Some(true));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("X = 5, 2 kick"), "{}", o[0].label);
}

#[test]
fn the_ai_pays_x_and_the_kicks_and_the_spell_resolves() {
    let mut g = game();
    for p in [P1, P2, P3] {
        g.player_mut(p).life = 5;
    }
    lands(&mut g, P0, "Mountain", 9, false);
    let c = hand(&mut g, P0, &["Comet Storm"])[0];
    let o = opts(&mut g, c, Some(true));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0]));
    assert!(g.over || [P1, P2, P3].iter().all(|&p| !g.player(p).alive), "all three opponents die");
}

#[test]
fn the_ai_does_not_cast_it_without_a_kill_early() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 5, false);
    let c = hand(&mut g, P0, &["Comet Storm"])[0];
    assert!(opts(&mut g, c, Some(true)).is_empty());
}

#[test]
fn the_end_of_turn_window_needs_a_kill() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Mountain", 6, false);
    g.player_mut(P2).life = 3;
    let c = hand(&mut g, P0, &["Comet Storm"])[0];
    assert_eq!(opts(&mut g, c, None).len(), 1, "lethal at an opponent's end of turn");
    g.player_mut(P2).life = 40;
    assert!(opts(&mut g, c, None).is_empty());
}
