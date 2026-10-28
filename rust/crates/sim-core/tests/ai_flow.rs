//! ai-flow (AI foundations step 3): the Tier 6 card and mana flow counters (`audit/ai-foundations/step3-flow.md`).
//! --- ai-eval: step 3's fixes A and B (`engine_first`) and their tests are gone: the look-ahead casts engines and
//! mana permanents itself (`tests/ai_eval.rs`, `audit/ai-foundations/eval-fix.md`).

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
