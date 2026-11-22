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

// ------------------------------------------------------------------ a doubled trigger (Panharmonicon)
/// Panharmonicon makes the trigger happen twice for a token; each instance pays its own {1}
#[test]
fn a_doubler_makes_the_token_draw_twice() {
    let mut g = board(2);
    perm(&mut g, P0, "Panharmonicon");
    token(&mut g, P0, 1);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 2);
}

/// The AI pays only while its hand holds six or fewer (`mentor_meek`): with six cards in hand the first instance
/// draws and the second finds seven, so it declines. This is why the doubling census fixture (hand of 6) sees one draw.
#[test]
fn the_ai_stops_paying_at_a_full_hand() {
    let mut g = board(2);
    hand(
        &mut g,
        P0,
        &["Plains", "Forest", "Serra Angel", "Mask of Memory", "Brainstorm", "Captain America, Team Leader"],
    );
    perm(&mut g, P0, "Panharmonicon");
    token(&mut g, P0, 1);
    settle_stack(&mut g).unwrap();
    assert_eq!(hand_size(&g), 7, "the second instance declines at seven cards");
}
