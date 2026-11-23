// Written by an independent reviewer (the yoshimaru-adv adversarial pass), adopted unchanged except where noted.
//! Adversarial tests for branch yoshimaru-ai: report.rs `resolved_by_turn`, the `yoshimaru` arm of `plan_turn` /
//! `plan_text`, and how `Analysis::add` feeds them. Declared from report.rs under #[cfg(test)] (the crate is a bin, so
//! its private functions cannot be reached from tests/).

use super::*;
use crate::play::{GameOut, PlayerOut};
use crate::stats::Counter;
use std::collections::HashMap;

fn player(deck: &'static str, ms: &[(&'static str, u32)]) -> PlayerOut {
    PlayerOut {
        key: deck,
        turns: 10,
        alive: true,
        killer: None,
        stats: HashMap::new(),
        milestone: ms.iter().copied().collect(),
        death_kind: "",
        seen: vec![],
        cast: vec![],
        lost: vec![],
        cast_turns: vec![],
        search_cost: Default::default(),
        generic: false,
    }
}

fn game(p: PlayerOut) -> GameOut {
    GameOut { winner: None, wintype: None, round: 10, stopped: false, players: vec![p], log: None }
}

/// the pre-change loop, verbatim, for comparison
fn old_resolved_by_turn(cmd: &Counter<u32>, n: i64) -> Vec<String> {
    let n = n as f64;
    let mut cum = 0;
    let mut line = vec![];
    for t in 2..9u32 {
        cum += cmd.get(&t);
        if t >= 4 {
            line.push(format!("T{t} {:.0}%", (100 * cum) as f64 / n));
        }
    }
    line
}

#[test]
fn resolved_by_turn_empty_input_is_all_zero() {
    let c = Counter::<u32>::default();
    assert_eq!(resolved_by_turn(&c, 100), ["T4 0%", "T5 0%", "T6 0%", "T7 0%", "T8 0%"]);
}

#[test]
fn resolved_by_turn_all_turn_zero_counts_everywhere() {
    let mut c = Counter::<u32>::default();
    c.add(0, 100);
    assert_eq!(resolved_by_turn(&c, 100), ["T4 100%", "T5 100%", "T6 100%", "T7 100%", "T8 100%"]);
}

#[test]
fn resolved_by_turn_ignores_turns_beyond_the_printed_range() {
    let mut c = Counter::<u32>::default();
    c.add(9, 50);
    c.add(12, 50);
    assert_eq!(resolved_by_turn(&c, 100), ["T4 0%", "T5 0%", "T6 0%", "T7 0%", "T8 0%"]);
}

#[test]
fn resolved_by_turn_is_cumulative_and_monotone_and_denominator_is_all_games() {
    let mut c = Counter::<u32>::default();
    // 60 of 200 games resolve the commander at turns 1, 3, 4, 4, 5, 8 (sum of counts 60); 140 never do
    for (t, k) in [(1, 10), (3, 10), (4, 10), (5, 10), (6, 10), (8, 10)] {
        c.add(t, k);
    }
    let l = resolved_by_turn(&c, 200);
    assert_eq!(l, ["T4 15%", "T5 20%", "T6 25%", "T7 25%", "T8 30%"]);
    let pct: Vec<f64> = l.iter().map(|s| s.split(' ').nth(1).unwrap().trim_end_matches('%').parse().unwrap()).collect();
    assert!(pct.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn resolved_by_turn_matches_the_old_output_when_no_turn_zero_or_one_casts_exist() {
    let mut c = Counter::<u32>::default();
    for (t, k) in [(2, 7), (3, 11), (4, 13), (6, 5), (7, 2), (9, 3)] {
        c.add(t, k);
    }
    assert_eq!(resolved_by_turn(&c, 137), old_resolved_by_turn(&c, 137));
}

#[test]
fn resolved_by_turn_with_zero_games_does_not_panic() {
    let c = Counter::<u32>::default();
    let _ = resolved_by_turn(&c, 0); // NaN% as before (the old loop divided by n as f64 too)
}

#[test]
fn plan_turn_yoshimaru_takes_the_earlier_of_tymna_and_combo() {
    let t = |ms: &[(&'static str, u32)]| plan_turn(&player("yoshimaru", ms), "yoshimaru");
    assert_eq!(t(&[("tymna", 3), ("combo", 6)]), Some(3), "Tymna first");
    assert_eq!(t(&[("tymna", 6), ("combo", 4)]), Some(4), "combo first");
    assert_eq!(t(&[("tymna", 5), ("combo", 5)]), Some(5), "same turn");
    assert_eq!(t(&[]), None, "neither");
    assert_eq!(t(&[("tymna", 4)]), Some(4));
    assert_eq!(t(&[("combo", 7)]), Some(7));
    assert_eq!(t(&[("tymna", 0), ("combo", 7)]), Some(7), "a zero milestone is no milestone");
    assert_eq!(t(&[("tymna", 0), ("combo", 0)]), None);
    // the deck's other milestones are no plan
    assert_eq!(t(&[("cmd_cast", 1), ("yshtola", 2), ("act", 3)]), None);
}

#[test]
fn plan_turn_for_the_other_decks_is_unchanged() {
    for deck in ["thrasios", "kraum"] {
        let p = player(deck, &[("tymna", 6), ("combo", 4)]);
        assert_eq!(plan_turn(&p, deck), Some(4), "{deck}");
    }
    // a deck that is not listed falls to `combo`, so the yoshimaru arm cannot leak: Tymna's draw does not count there
    let p = player("sheoldred", &[("tymna", 2)]);
    assert_eq!(plan_turn(&p, "kyodai"), None);
    assert_eq!(plan_turn(&p, "yshtola"), None);
    assert_eq!(plan_turn(&player("urdragon", &[("combo", 5), ("tymna", 2)]), "urdragon"), Some(5));
}

#[test]
fn plan_text_names_the_yoshimaru_plan_and_only_for_that_deck() {
    assert!(!plan_text("yoshimaru").is_empty());
    assert_eq!(plan_text("thrasios"), plan_text("kraum"));
    assert_ne!(plan_text("yoshimaru"), plan_text("thrasios"));
    assert_eq!(plan_text("no-such-deck"), "");
}

#[test]
fn analysis_add_feeds_plan_and_cmd_with_turn_one_events_and_never_resolved_games_count_in_the_denominator() {
    let mut a = Analysis::default();
    a.add(&game(player("yoshimaru", &[("cmd_cast", 1), ("tymna", 1)])), "yoshimaru"); // turn 1 plan and cast
    a.add(&game(player("yoshimaru", &[("cmd_cast", 3), ("combo", 5), ("tymna", 4)])), "yoshimaru");
    a.add(&game(player("yoshimaru", &[])), "yoshimaru"); // never resolved, no plan
    assert_eq!(a.n, 3);
    assert_eq!(a.cmd.total(), 2);
    assert_eq!(a.plan.total(), 2);
    assert_eq!(a.plan.get(&1), 1);
    assert_eq!(a.plan.get(&4), 1);
    let l = resolved_by_turn(&a.cmd, a.n);
    // 2 of 3 games have resolved the commander by T4: the denominator is games, not resolved games
    assert_eq!(l[0], "T4 67%");
}

/// the cmd milestone can be a turn 0 value (a cast before the first turn counter ticks); `Analysis::add` keeps it
#[test]
fn analysis_add_keeps_a_turn_zero_cast() {
    let mut a = Analysis::default();
    a.add(&game(player("yoshimaru", &[("cmd_cast", 0)])), "yoshimaru");
    assert_eq!(resolved_by_turn(&a.cmd, a.n)[0], "T4 100%");
}
