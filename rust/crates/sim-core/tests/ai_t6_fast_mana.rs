//! Tier 6 fast mana on turns 1-3 (ai/t6_table.rs `must_option`): Chrome Mox with a card to imprint is cast before the
//! look-ahead, which voted "stop" over it (the two cards in hand score above a mox on the battlefield).

use sim_core::ai::t6_table;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KINNAN: &str = "kinnan-bonder-mana-combo";
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";

fn turn_one(hand_cards: &[&str]) -> Game {
    let mut g = table(&[KINNAN, KRAUM]);
    g.active = Some(P0);
    g.player_mut(P0).turns = 1;
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, hand_cards);
    g
}

fn must_label(g: &mut Game) -> Option<String> {
    let opts = sim_core::ai::brain::main_options(g, P0, false).unwrap();
    t6_table::must_option(g, P0, &opts).map(|i| opts[i].label.clone())
}

#[test]
fn chrome_mox_with_a_spare_card_is_cast_on_turn_one() {
    let mut g = turn_one(&["Chrome Mox", "Brainstorm"]);
    assert_eq!(must_label(&mut g).as_deref(), Some("Chrome Mox"));
}

#[test]
fn chrome_mox_with_nothing_to_imprint_is_not_forced() {
    let mut g = turn_one(&["Chrome Mox", "Forest"]);
    assert_eq!(must_label(&mut g), None);
}

#[test]
fn chrome_mox_is_not_forced_when_the_only_spare_card_is_a_line_piece() {
    let mut g = turn_one(&["Chrome Mox", "Hullbreaker Horror"]);
    assert!(sim_core::impls::t6_pieces::is_line_piece(&g, P0, "Hullbreaker Horror"), "a line piece of Kinnan's");
    assert_eq!(must_label(&mut g), None);
}

#[test]
fn chrome_mox_is_not_forced_after_turn_three() {
    let mut g = turn_one(&["Chrome Mox", "Brainstorm", "Mystical Tutor"]);
    g.player_mut(P0).turns = 4;
    assert_ne!(must_label(&mut g).as_deref(), Some("Chrome Mox"));
}
