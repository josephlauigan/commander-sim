// Written by an independent reviewer (the yoshimaru-adv adversarial pass), adopted unchanged except where noted.
//! Adversarial tests for branch yoshimaru-ai: explain.rs `by_turn` (declared from explain.rs under #[cfg(test)]).

use super::*;
use crate::stats::Counter;

fn old_by_turn(c: &Counter<u32>, t: u32, n: i64) -> f64 {
    (2..=t).map(|x| c.get(&x)).sum::<i64>() as f64 * 100.0 / n.max(1) as f64
}

#[test]
fn by_turn_empty_and_zero_games() {
    let c = Counter::<u32>::default();
    for t in [0, 1, 4, 12] {
        assert_eq!(by_turn(&c, t, 100), 0.0);
        assert_eq!(by_turn(&c, t, 0), 0.0, "n = 0 is clamped to 1, no NaN");
    }
}

#[test]
fn by_turn_turn_zero_and_beyond_range() {
    let mut c = Counter::<u32>::default();
    c.add(0, 10);
    c.add(1, 10);
    c.add(20, 10);
    assert_eq!(by_turn(&c, 0, 100), 10.0);
    assert_eq!(by_turn(&c, 1, 100), 20.0);
    assert_eq!(by_turn(&c, 12, 100), 20.0, "a turn past the asked one is not counted");
    assert_eq!(by_turn(&c, 30, 100), 30.0);
}

#[test]
fn by_turn_is_cumulative_and_monotone() {
    let mut c = Counter::<u32>::default();
    for (t, k) in [(1, 3), (2, 4), (5, 9), (6, 1), (9, 2)] {
        c.add(t, k);
    }
    let v: Vec<f64> = (0..=12).map(|t| by_turn(&c, t, 50)).collect();
    assert!(v.windows(2).all(|w| w[0] <= w[1]), "{v:?}");
    assert_eq!(v[12], 38.0);
}

#[test]
fn by_turn_equals_the_old_value_when_no_turn_zero_or_one_events_exist() {
    let mut c = Counter::<u32>::default();
    for (t, k) in [(2, 4), (3, 6), (4, 1), (7, 5), (11, 2)] {
        c.add(t, k);
    }
    for t in [2, 4, 6, 8, 12] {
        assert_eq!(by_turn(&c, t, 77), old_by_turn(&c, t, 77), "turn {t}");
    }
}

/// the denominator is the games run, never the games that had the event
#[test]
fn by_turn_denominator_is_all_games() {
    let mut c = Counter::<u32>::default();
    c.add(3, 25);
    assert_eq!(by_turn(&c, 4, 100), 25.0);
    assert_eq!(by_turn(&c, 4, 25), 100.0);
}
