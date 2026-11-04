//! Many counter doublers (urdragon t5, ten seeds from 500196): one seat's doubler copies passed 31 and the counters
//! step's multiplier (`gather`'s `mult`, an i32 product) overflowed. The multiplier is capped at
//! `repl::COUNTER_MULT_CAP`, the modifications saturate, and one event puts at most `repl::COUNTER_CAP` counters.

use sim_core::engine::repl;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// seat 0 with `k` Doubling Seasons
fn doublers(k: usize) -> Game {
    let mut g = table(&["sauron", "seph"]);
    for _ in 0..k {
        perm(&mut g, P0, "Doubling Season");
    }
    g
}

#[test]
fn a_few_doublers_still_double() {
    let mut g = doublers(3);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(repl::counter_mult(&g, P0), 8);
    assert_eq!(repl::put_counters(&mut g, P0, elves, 2), 16);
    assert_eq!(g.perm(elves).plus, 16);
    assert_eq!(repl::counter_mult(&g, P1), 1);
}

#[test]
fn forty_doublers_put_counters_without_overflow() {
    let mut g = doublers(40);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(repl::counter_mult(&g, P0), repl::COUNTER_MULT_CAP);
    // before the cap: 2^32 overflowed the product in `gather` (a panic in the debug-asserting release build)
    let put = repl::put_counters(&mut g, P0, elves, 1);
    assert_eq!(put, repl::COUNTER_MULT_CAP);
    let put = repl::put_counters(&mut g, P0, elves, 100);
    assert_eq!(put, repl::COUNTER_CAP, "one event stops at the cap");
    assert_eq!(g.perm(elves).plus, repl::COUNTER_MULT_CAP + repl::COUNTER_CAP);
    // a huge request saturates rather than wrapping negative
    assert_eq!(repl::put_counters(&mut g, P0, elves, i32::MAX), repl::COUNTER_CAP);
}
