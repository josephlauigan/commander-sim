//! ai-rollouts: adaptive rollouts for the close calls the margin hands back (`audit/ai-foundations/rollouts.md`).
//! A main-phase decision whose look-ahead pick doesn't clear the margin over the heuristic's top play gets more
//! rollouts of the same options, a batch at a time, until the margin holds, it clearly can't, or the cap.

use sim_core::ai::{brain, search};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Step;
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);

const A: search::Adapt = search::Adapt { batch: 6, cap: 24, stop: 0.5, z: 2.0 };

/// two options: 0 the heuristic's top play (always 0), 1 the look-ahead's pick, whose value per rollout r is
/// `pattern[r % len]`; returns (bare, best, rollouts per option used, batches asked for)
fn run(pattern: &[f64], a: &search::Adapt) -> (usize, usize, usize, usize) {
    let f = |r: u64| pattern[r as usize % pattern.len()];
    let mut vals = vec![vec![0.0; 6], (0..6).map(f).collect::<Vec<f64>>()];
    let ex = [0.0, 0.0];
    let first = search::pick_with(&vals, &ex, 0, a.z);
    let mut asked = 0;
    let (b, best) = search::adapt_rollouts(&mut vals, &ex, 0, first, a, |v, rs| {
        asked += 1;
        for r in rs {
            v[0].push(0.0);
            v[1].push(f(r));
        }
    });
    (b, best, vals[0].len(), asked)
}

/// A real but noisy gain (mean 2, standard error 1.8 at six rollouts) is handed back at six, and stands once more
/// rollouts show it: at 18 it clears two standard errors
#[test]
fn a_noisy_real_gain_stands_after_more_rollouts() {
    assert_eq!(search::pick_with(&[vec![0.0; 6], vec![6.0, -2.0, 6.0, -2.0, 6.0, -2.0]], &[0.0, 0.0], 0, 2.0), (1, 0));
    assert_eq!(run(&[6.0, -2.0], &A), (1, 1, 18, 2));
}

/// A gain inside the noise at every count (about 0.3 standard deviations a rollout) runs to the cap and goes back
/// to the heuristic; a cap that isn't a multiple of the batch is kept
#[test]
fn a_gain_inside_the_noise_goes_back_to_the_heuristic_at_the_cap() {
    assert_eq!(run(&[10.0, -5.0], &A), (1, 0, 24, 3));
    assert_eq!(run(&[10.0, -5.0], &search::Adapt { cap: 20, ..A }), (1, 0, 20, 3));
    assert_eq!(run(&[10.0, -5.0], &search::Adapt { cap: 6, ..A }), (1, 0, 6, 0), "a cap of six: the old rule");
}

/// The early stop: a gain at or below `stop` standard errors gets no more rollouts
#[test]
fn a_hopeless_gain_gets_no_more_rollouts() {
    assert!(search::hopeless(&[-1.0, 0.5, -2.0, 1.0, 0.0, -0.5], 0.5), "a mean below 0");
    assert!(search::hopeless(&[0.0; 6], 0.5), "a tie");
    assert!(search::hopeless(&[20.0, -18.0, 5.0, -10.0, 15.0, -9.0], 0.5), "0.1 standard errors");
    assert!(!search::hopeless(&[10.0, -5.0, 10.0, -5.0, 10.0, -5.0], 0.5), "0.75 standard errors");
    assert_eq!(run(&[2.0, -40.0, 30.0, 1.0, 2.0, 7.0], &A), (1, 0, 6, 0), "0.07 standard errors: stops at six");
}

/// More rollouts that put the top play first end the extension with the top play, not a hand-back
#[test]
fn the_top_play_first_on_more_rollouts_ends_it() {
    // +5 on the first six rollouts (0.6 standard errors), then -10 on every new one
    let mut vals = vec![vec![0.0; 6], vec![20.0, -10.0, 20.0, -10.0, 20.0, -10.0]];
    let ex = [0.0, 0.0];
    let first = search::pick_with(&vals, &ex, 0, 2.0);
    assert_eq!(first, (1, 0));
    let r = search::adapt_rollouts(&mut vals, &ex, 0, first, &A, |v, rs| {
        for _ in rs {
            v[0].push(0.0);
            v[1].push(-10.0);
        }
    });
    assert_eq!((r, vals[0].len()), ((0, 0), 12));
}

/// A decision the margin doesn't hand back (the pick stands, or the top play is the pick) gets no more rollouts
#[test]
fn only_a_handed_back_decision_gets_more_rollouts() {
    assert_eq!(run(&[3.0, 2.5, 4.0, 3.5, 2.0, 3.0], &A), (1, 1, 6, 0), "the pick stands at six");
    assert_eq!(run(&[-3.0], &A), (0, 0, 6, 0), "the top play is the pick");
}

fn kraum_rhystic(adapt: Option<(u32, u32)>) -> (Option<String>, u64) {
    let mut g = table(&[
        "kraum-tymna-blue-breach-midrange",
        "sisay-legends-toolbox",
        "dack-fayden-white-stax",
        "kinnan-bonder-mana-combo",
    ]);
    set_ai(&mut g, AiMode::Lookahead);
    let mut s = (*g.settings).clone();
    s.search.adapt = adapt.is_some();
    if let Some((cap, batch)) = adapt {
        s.search.adapt_cap = cap;
        s.search.adapt_batch = batch;
    }
    g.settings = Arc::new(s);
    let lib: Vec<_> = (0..40).map(|i| card(&g, if i % 2 == 0 { "Island" } else { "Brainstorm" })).collect();
    g.player_mut(P0).library = lib;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Rhystic Study", "Sol Ring", "Brainstorm", "Arcane Signet"]);
    lands(&mut g, P0, "Island", 4, false);
    g.player_mut(P0).turns = 3;
    g.active = Some(P0);
    g.step = Step::Main2;
    let opts = brain::main_options(&mut g, P0, true).unwrap();
    let before = search::stats().playouts;
    let i = search::choose(&mut g, P0, true, &opts);
    (i.map(|i| opts[i].label.clone()), search::stats().playouts - before)
}

/// In a game: a cap of six rollouts is the old rule exactly (the same pick on the same playouts), and a cap of 24
/// never plays an option more than 24 times
#[test]
fn in_a_game_the_cap_bounds_the_rollouts_and_six_is_the_old_rule() {
    let off = kraum_rhystic(None);
    assert_eq!(kraum_rhystic(Some((6, 6))), off);
    let (_, n_off) = off;
    let (pick, n) = kraum_rhystic(Some((24, 6)));
    assert!(pick.is_some());
    // the options tried: up to `top_k` plays and "stop"; six rollouts each with the switch off
    assert!(n >= n_off && n <= n_off * 4, "{n} playouts against {n_off}");
}

/// The margin's z: Pocock's 2.36 at every look when the adaptive rollouts can look more than once, so testing again
/// after each batch lets through no more noise than one test at z = 2; the plain margin when they're off or capped at
/// one look, and 0 (the bare argmax) stays 0
#[test]
fn repeated_looks_use_the_stricter_z() {
    let mut s = sim_core::settings::Search::default();
    s.margin = 2.0;
    s.adapt_z = 2.36;
    s.rollouts = 6;
    s.adapt = true;
    s.adapt_cap = 24;
    assert_eq!(search::margin_z(&s), 2.36);
    s.adapt_cap = 6;
    assert_eq!(search::margin_z(&s), 2.0);
    s.adapt_cap = 24;
    s.adapt = false;
    assert_eq!(search::margin_z(&s), 2.0);
    s.adapt = true;
    s.margin = 0.0;
    assert_eq!(search::margin_z(&s), 0.0);
}
