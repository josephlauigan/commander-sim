//! ai-flow (AI foundations step 3): the Tier 6 card and mana flow fixes and their counters
//! (`audit/ai-foundations/step3-flow.md`).

use sim_core::ai::t6_table::engine_first;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn kraum_table() -> Game {
    table(&[
        "kraum-tymna-blue-breach-midrange",
        "sisay-legends-toolbox",
        "dack-fayden-white-stax",
        "kinnan-bonder-mana-combo",
    ])
}

fn cast(g: &Game, name: &str, u: f64) -> Opt {
    Opt { utility: u, label: name.into(), act: Some(Action::Cast { card: card(g, name), zone: None }) }
}

fn stop(u: f64) -> Opt {
    Opt { utility: u, label: "stop (hold mana)".into(), act: None }
}

/// The heuristic rates Rhystic Study above passing: it is cast before the look-ahead (which voted "stop")
#[test]
fn a_draw_engine_the_heuristic_rates_best_comes_before_the_look_ahead() {
    let mut g = kraum_table();
    hand(&mut g, P0, &["Rhystic Study", "Sol Ring"]);
    let opts = vec![cast(&g, "Sol Ring", 4.0), cast(&g, "Rhystic Study", 6.0), stop(1.0)];
    assert_eq!(engine_first(&g, P0, &opts), Some(1));
}

/// Passing rated higher (a counter held, the plan's hold_up), or a best play that is not an engine (Demonic Tutor):
/// the look-ahead decides as before
#[test]
fn passing_rated_higher_or_a_plain_best_play_leaves_it_to_the_look_ahead() {
    let mut g = kraum_table();
    hand(&mut g, P0, &["Rhystic Study", "Sol Ring"]);
    let opts = vec![cast(&g, "Rhystic Study", 3.0), stop(4.0)];
    assert_eq!(engine_first(&g, P0, &opts), None);
    let opts = vec![cast(&g, "Demonic Tutor", 7.0), cast(&g, "Rhystic Study", 6.0), stop(1.0)];
    assert_eq!(engine_first(&g, P0, &opts), None);
}

/// Tymna from the command zone is an engine (her draws come after the horizon); a deck outside Tier 6 is untouched
#[test]
fn the_commander_from_the_command_zone_counts_and_other_tiers_are_untouched() {
    let g = kraum_table();
    let tymna = card(&g, "Tymna the Weaver");
    assert!(g.player(P0).is_commander(tymna));
    let opts = vec![cast(&g, "Tymna the Weaver", 5.0), stop(0.0)];
    assert_eq!(engine_first(&g, P0, &opts), Some(0));
    let mut g =
        table(&["atraxa-infect", "korvold-jund-sacrifice", "marwyn-mono-green-elves", "tergrid-mono-black-disruption"]);
    hand(&mut g, P0, &["Rhystic Study"]);
    let opts = vec![cast(&g, "Rhystic Study", 6.0), stop(1.0)];
    assert_eq!(engine_first(&g, P0, &opts), None);
}

/// The flow counters credit a draw to the source resolving for the player: the draw step, then an engine
#[test]
fn a_draw_is_credited_to_its_source() {
    let mut g = kraum_table();
    let lib: Vec<_> = (0..10).map(|_| card(&g, "Island")).collect();
    g.player_mut(P0).library = lib;
    g.player_mut(P0).turns = 2;
    g.active = Some(P0);
    sim_core::engine::zones::draw(&mut g, P0, 1, true).unwrap();
    let rhystic = card(&g, "Rhystic Study");
    sim_core::t6stats::flow_source(&mut g, P0, Some(rhystic));
    sim_core::engine::zones::draw(&mut g, P0, 2, false).unwrap();
    let st = |k: &str| g.player(P0).stats.get(k).copied().unwrap_or(0);
    assert_eq!(st("fl|d|step|T2"), 1);
    assert_eq!(st("fl|d|engine|T2"), 2);
    assert_eq!(st("fl|dc|Rhystic Study"), 2);
}

/// A rock or mox the heuristic rates best is made before the look-ahead too; Lotus Petal (one-shot) is not
#[test]
fn a_mana_permanent_comes_before_the_look_ahead_but_not_a_one_shot() {
    let mut g = kraum_table();
    hand(&mut g, P0, &["Chrome Mox", "Lotus Petal", "Force of Will"]);
    let opts = vec![cast(&g, "Chrome Mox", 6.0), stop(1.0)];
    assert_eq!(engine_first(&g, P0, &opts), Some(0));
    let opts = vec![cast(&g, "Lotus Petal", 6.0), stop(1.0)];
    assert_eq!(engine_first(&g, P0, &opts), None);
}
