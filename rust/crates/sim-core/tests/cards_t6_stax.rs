//! Tier 6 "stax" batch (impls/t6_stax.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::engine::mana::cost_of;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["dack-fayden-white-stax", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox", "godo"])
}

// ======================================================== Disruptor Flute
#[test]
fn flute_taxes_the_named_card_by_three_for_everyone_and_stops_its_abilities() {
    let mut g = tbl();
    let flute = perm(&mut g, P0, "Disruptor Flute");
    let sol = card(&g, "Sol Ring");
    g.flutes.clear();
    g.flutes.push((flute, card(&g, "Sol Ring")));
    assert_eq!(cost_of(&g, P0, sol).0, 4, "Sol Ring costs {{3}} more for its controller");
    assert_eq!(cost_of(&g, P1, sol).0, 4, "and for opponents");
    assert!(sim_core::engine::values::stopped(&g, "Sol Ring"), "its activated abilities are off");
    assert!(!sim_core::engine::values::stopped(&g, "Mana Vault"));
    g.perm_mut(flute).on_bf = false;
    assert_eq!(cost_of(&g, P1, sol).0, 1, "gone with the Flute");
}
