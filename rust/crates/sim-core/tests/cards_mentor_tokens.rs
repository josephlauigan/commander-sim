//! Mentor of the Meek (impls/t1.rs `mentor_meek`): "Whenever another creature with power 2 or less enters under your
//! control, you may pay {1}. If you do, draw a card." A creature token is an entering creature too, so it triggers it
//! the way a creature card does (the AI path: the person's path runs the same hook).

use sim_core::engine::stack::settle_stack;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// Mentor out, `plains` untapped Plains, a stocked library and an empty hand
fn board(plains: usize) -> Game {
    let mut g = table(&["galadriel", "cloud"]);
    perm(&mut g, P0, "Mentor of the Meek");
    lands(&mut g, P0, "Plains", plains, false);
    g.player_mut(P0).hand.clear();
    assert!(g.player(P0).library.len() > 5);
    g
}

fn hand_size(g: &Game) -> usize {
    g.player(P0).hand.len()
}

#[test]
fn a_creature_card_with_power_one_draws() {
    let mut g = board(1);
    perm(&mut g, P0, "Llanowar Elves");
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 1, "the control: a creature card entering draws");
}

#[test]
fn a_one_one_token_draws() {
    let mut g = board(1);
    token(&mut g, P0, 1);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 1, "a 1/1 creature token entering: pay {{1}}, draw");
}

#[test]
fn a_power_two_token_draws() {
    let mut g = board(1);
    token(&mut g, P0, 2);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 1);
}

#[test]
fn a_power_three_token_does_not_draw() {
    let mut g = board(1);
    token(&mut g, P0, 3);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 0);
}

#[test]
fn no_mana_open_means_no_draw() {
    let mut g = board(0);
    token(&mut g, P0, 1);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 0);
}

#[test]
fn an_opponents_token_does_not_draw() {
    let mut g = board(1);
    token(&mut g, PlayerId(1), 1);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 0);
}
