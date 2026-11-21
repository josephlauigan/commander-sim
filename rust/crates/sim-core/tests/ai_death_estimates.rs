//! The AI's estimates of what one creature death is worth count the death-trigger doublers (Teysa Karlov, Drivnod): they
//! ask about a creature dying, which is what those cards answer to.

use sim_core::impls::common;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: sim_core::ids::PlayerId = sim_core::ids::PlayerId(0);
const P1: sim_core::ids::PlayerId = sim_core::ids::PlayerId(1);

fn game() -> Game {
    table(&["teysa-orzhov-aristocrats", "seph"])
}

#[test]
fn a_creature_death_is_worth_twice_as_much_with_teysa_or_drivnod() {
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    let once = common::death_value(&g, P0, None);
    assert!(once > 0.0, "a drain per death");
    perm(&mut g, P0, "Teysa Karlov");
    assert_eq!(common::death_value(&g, P0, None), 2.0 * once, "Teysa: the drain happens twice");
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Drivnod, Carnage Dominus");
    assert_eq!(common::death_value(&g, P0, None), 2.0 * once, "Drivnod: the same");
}

#[test]
fn a_nonlegendary_creature_death_is_not_doubled_by_gandalf() {
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    let once = common::death_value(&g, P0, None);
    perm(&mut g, P0, "Gandalf the White");
    perm(&mut g, P0, "Mother of Runes");
    // Gandalf is legendary himself, so the cheapest creature to ask about is not: Mother of Runes
    assert_eq!(
        common::death_value(&g, P0, None),
        once,
        "the creature that would die is neither legendary nor an artifact"
    );
}

#[test]
fn the_fodder_drain_estimate_counts_teysa_so_the_outlet_is_lethal() {
    // Blood Artist is worth two per death (its drain, and its tag); Teysa doubles it; the fodder is Teysa and three tokens:
    // 16 against life 12 (8 without the doubling)
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Teysa Karlov");
    perm(&mut g, P0, "Viscera Seer");
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    g.player_mut(P1).life = 12;
    let o = common::aristocrat_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].utility, 9.0, "the sacrifices kill the opponent");
}
