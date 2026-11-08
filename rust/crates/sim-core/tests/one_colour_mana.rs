//! "Add N mana of any one color" makes N mana of ONE colour, chosen to suit what the player holds, never split across
//! colours (Carpet of Flowers, The Aetherspark). Both used to float `Kind::Any`, which could pay {U}{R}{G}... at once.

use sim_core::engine::mana::can_pay;
use sim_core::engine::pool::Kind;
use sim_core::engine::{hooks, mana};
use sim_core::hooks::{Call, Event};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// a five-colour seat (scion) against three seats, its hand emptied
fn five_colours() -> Game {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn colours_floating(g: &Game) -> Vec<(char, u32)> {
    [('W', Kind::W), ('U', Kind::U), ('B', Kind::B), ('R', Kind::R), ('G', Kind::G), ('A', Kind::Any), ('C', Kind::C)]
        .into_iter()
        .map(|(c, k)| (c, g.player(P0).mana.get(k)))
        .filter(|x| x.1 > 0)
        .collect()
}

fn aetherspark_ultimate(g: &mut Game) {
    let m = perm(g, P0, "The Aetherspark");
    let c = g.perm(m).card();
    let ab = g.registry.get(c).unwrap().walker.unwrap();
    (ab[2].eff)(g, P0, m).unwrap();
}

#[test]
fn the_aetherspark_adds_ten_mana_of_one_colour_the_hand_asks_for() {
    for (spell, letter) in [("Counterspell", 'U'), ("Lightning Bolt", 'R'), ("Cultivate", 'G')] {
        let mut g = five_colours();
        hand(&mut g, P0, &[spell]);
        aetherspark_ultimate(&mut g);
        assert_eq!(colours_floating(&g), vec![(letter, 10)], "{spell}: ten mana of one colour, the one it needs");
    }
}

#[test]
fn the_aetherspark_mana_cannot_pay_two_colours() {
    let mut g = five_colours();
    hand(&mut g, P0, &["Counterspell"]);
    aetherspark_ultimate(&mut g);
    assert!(can_pay(&g, P0, 0, "UUUU", false));
    assert!(!can_pay(&g, P0, 0, "UR", false), "ten blue mana are not a red mana");
}

#[test]
fn carpet_of_flowers_adds_x_mana_of_one_colour() {
    let mut g = five_colours();
    perm(&mut g, P0, "Carpet of Flowers");
    lands(&mut g, P1, "Island", 3, false);
    hand(&mut g, P0, &["Lightning Bolt"]);
    hooks::fire_trigger(&mut g, Event::Main1, Call::Player { p: P0 }).unwrap();
    assert_eq!(colours_floating(&g), vec![('R', 3)], "three Islands: three mana, all red for the Bolt");
    assert!(!can_pay(&g, P0, 0, "RU", false));
}

#[test]
fn the_choice_follows_what_cannot_be_paid_now() {
    // a Mountain already pays the Bolt, so the blue Counterspell decides the colour
    let mut g = five_colours();
    lands(&mut g, P0, "Mountain", 1, false);
    hand(&mut g, P0, &["Lightning Bolt", "Counterspell"]);
    assert_eq!(mana::choose_one_colour(&g, P0, 3), Kind::U);
}
