//! Options the AI offers must be plays the engine will make: a removal spell under a lock (Silence) and a combo
//! already gone for this turn are not offered (each was chosen, then refused, in traced Tier 6 games).

use sim_core::ai::brain::{Situation, removal_options};
use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn removal_under_silence_is_not_offered() {
    let mut g = table(&["dack-fayden-white-stax", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    perm(&mut g, P1, "Grave Titan");
    let offered = |g: &sim_core::state::Game| {
        removal_options(g, P0, &Situation::new(g, P0)).iter().any(|o| o.label.contains("Swords to Plowshares"))
    };
    assert!(offered(&g), "Swords on Grave Titan");
    g.locks_used = true;
    g.silence = Some((g.turn_stamp(), P1));
    assert!(!offered(&g), "an opponent's Silence: no noncreature spells this turn");
}

#[test]
fn a_combo_gone_for_this_turn_is_not_offered_again() {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    let i = combos::combo_index("Oracle + Demonic Consultation").unwrap();
    let name = combos::all()[i].name;
    let offered = |g: &mut sim_core::state::Game| {
        combos::combo_options(g, P0, 0.0, Some(false)).unwrap().iter().any(|o| o.label.contains(name))
    };
    assert!(offered(&mut g));
    let turns = g.player(P0).turns;
    g.player_mut(P0).combo_turn = Some((turns, name));
    assert!(!offered(&mut g), "attempt_run refuses a second try this turn");
}
