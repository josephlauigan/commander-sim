//! ai-eval: the look-ahead's evaluation fixed at the cause (`audit/ai-foundations/eval-fix.md`): the board valued
//! for what it will produce, the hold bonus to the playouts that kept the mana up, and the margin before the
//! look-ahead departs from the heuristic's top play. Step 3's A and B (engines and mana before the look-ahead) are
//! gone, so these decisions are the look-ahead's own.

use sim_core::ai::{brain, search};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn kraum_table() -> Game {
    let mut g = table(&[
        "kraum-tymna-blue-breach-midrange",
        "sisay-legends-toolbox",
        "dack-fayden-white-stax",
        "kinnan-bonder-mana-combo",
    ]);
    set_ai(&mut g, AiMode::Lookahead);
    g
}

/// Rhystic Study and Sol Ring in hand, four lands, nothing else on the board, no counterspell to hold mana for: the
/// look-ahead casts both (it voted "stop" over them before step 3's workarounds)
#[test]
fn the_look_ahead_casts_rhystic_study_and_sol_ring_with_nothing_to_hold() {
    let mut g = kraum_table();
    let lib: Vec<_> = (0..40).map(|i| card(&g, if i % 2 == 0 { "Island" } else { "Brainstorm" })).collect();
    g.player_mut(P0).library = lib;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Rhystic Study", "Sol Ring"]);
    lands(&mut g, P0, "Island", 4, false);
    g.player_mut(P0).turns = 3;
    g.active = Some(P0);
    g.step = Step::Main2;
    let s = brain::Situation::new(&g, P0);
    assert!(brain::hold_value(&g, P0, &s).0.is_none(), "nothing to hold mana for");
    let mut cast = vec![];
    for _ in 0..2 {
        let opts = brain::main_options(&mut g, P0, true).unwrap();
        let i = search::choose(&mut g, P0, true, &opts).expect("the look-ahead decides");
        let scores = search::LAST_SCORES.with(|l| l.borrow().clone());
        let a = opts[i].act.clone().unwrap_or_else(|| panic!("passed: {scores:?}"));
        cast.push(opts[i].label.clone());
        assert!(sim_core::ai::act::perform(&mut g, P0, &a).unwrap(), "{} could not be cast", opts[i].label);
    }
    cast.sort();
    assert_eq!(cast, ["Rhystic Study", "Sol Ring"]);
}

/// The output model: Rhystic Study at its measured cards a turn cycle (none in a goldfish: its seats never cast),
/// Sol Ring at its two mana converted at one rate, a one-shot (Lotus Petal) and a rock that doesn't untap at nothing
#[test]
fn a_permanent_is_valued_for_what_it_will_produce() {
    let mut g = kraum_table();
    let rhystic = perm(&mut g, P0, "Rhystic Study");
    let ring = perm(&mut g, P0, "Sol Ring");
    let petal = perm(&mut g, P0, "Lotus Petal");
    let vault = perm(&mut g, P0, "Mana Vault");
    let window: f64 = search::OUTPUT_TURNS.iter().sum();
    assert_eq!(search::output_cards(&g, rhystic), 3.0);
    assert!((search::output_cards(&g, ring) - 2.0 * search::MANA_CARD).abs() < 1e-9);
    assert!((search::output_value(&g, ring) - search::HAND_W * window * 2.0 * search::MANA_CARD).abs() < 1e-9);
    // one mana a turn over the window is worth a land in `strength` (0.6), one mana once a Treasure (0.4)
    assert!((search::HAND_W * window * search::MANA_CARD - 0.6).abs() < 1e-9);
    assert_eq!(search::output_cards(&g, petal), 0.0);
    assert_eq!(search::output_cards(&g, vault), 0.0);
}

/// The hold bonus goes to the line that kept the counter's mana up through the turn, a cast included
#[test]
fn the_held_card_is_kept_up_when_it_is_in_hand_and_payable() {
    let mut g = kraum_table();
    g.player_mut(P0).hand.clear();
    let fow = hand(&mut g, P0, &["Swan Song"])[0];
    assert!(!search::kept_up(&g, P0, fow), "no mana");
    lands(&mut g, P0, "Island", 1, false);
    assert!(search::kept_up(&g, P0, fow), "Swan Song's U is up");
}

/// The margin: a pick that beats the top play by less than its noise doesn't stand; a clear one does
#[test]
fn the_look_ahead_departs_from_the_top_play_only_on_evidence() {
    assert!(!search::margin_holds(&[20.0, -18.0, 5.0, -10.0, 15.0, -9.0], 1.0), "noise");
    assert!(search::margin_holds(&[3.0, 2.5, 4.0, 3.5, 2.0, 3.0], 1.0), "a steady gain");
    assert!(!search::margin_holds(&[0.0; 6], 1.0), "a tie keeps the top play");
}
