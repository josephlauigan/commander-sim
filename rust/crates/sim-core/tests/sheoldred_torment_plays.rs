//! `data/plans/sheoldred.toml` lists Torment of Hailfire in `plays` (2026-11-26): the card's main-phase option (a finisher
//! or a late strip of every opponent's cards and permanents) is then offered to the Sheoldred seat as it already was to
//! the Tergrid and Y'shtola seats. A seat whose plan does not list it is unchanged.

use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const OPPS: [&str; 3] = ["korvold-jund-sacrifice", "marwyn-mono-green-elves", "atraxa-infect"];

fn options_for(first: &str) -> usize {
    let mut keys = vec![first];
    keys.extend(OPPS);
    let mut g: Game = table(&keys);
    for p in g.players.iter_mut() {
        p.cmd_in_zone = false;
        p.hand.clear();
    }
    let t = hand(&mut g, P0, &["Torment of Hailfire"])[0];
    lands(&mut g, P0, "Swamp", 8, false);
    for q in 1..4 {
        g.players[q].life = 9;
    }
    let f = g.registry.get(t).unwrap().hand_options.unwrap();
    f(&mut g, t, P0, Some(false)).unwrap().len()
}

#[test]
fn the_sheoldred_seat_is_offered_torment_of_hailfire_for_the_kill() {
    assert_eq!(options_for("sheoldred"), 1, "X = 6: each opponent at 9 with nothing to give dies");
}

#[test]
fn a_seat_whose_plan_does_not_list_it_is_not() {
    assert_eq!(options_for("urdragon"), 0);
}
