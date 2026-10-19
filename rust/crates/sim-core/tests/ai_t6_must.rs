//! Tier 6 must-do plays (ai/t6_table.rs `must_option`): the plan's top plays are taken before the look-ahead.

use sim_core::ai::t6_table;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KINNAN: &str = "kinnan-bonder-mana-combo";
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";

fn tbl(keys: &[&str]) -> Game {
    let mut g = table(keys);
    g.active = Some(P0);
    g
}

#[test]
fn the_commander_is_cast_ahead_of_a_tutor_when_the_plan_says_so() {
    let mut g = tbl(&[KINNAN, KRAUM]);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Chord of Calling", "Sol Ring"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let i = t6_table::must_option(&g, P0, &opts).expect("a must play");
    assert!(opts[i].label.starts_with("Kinnan"), "got {}", opts[i].label);
}

#[test]
fn nothing_is_forced_when_no_play_is_a_must() {
    let mut g = tbl(&[KINNAN, KRAUM]);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Brainstorm"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(t6_table::must_option(&g, P0, &opts).is_none());
}
