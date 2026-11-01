//! Many token doublers (thrasios t5, seed 500193): the Miirym deck's Astral Dragon made token copies of Parallel
//! Lives until one seat had 39. 2^39 saturated the multiplier to u32::MAX, which `as i32` read as -1: one Treasure
//! became u32::MAX Treasures and the second creation that turn overflowed `note_tokens`. The multiplier is capped
//! at `repl::MULT_CAP` (the battlefield's `TOKEN_CAP`).

use sim_core::engine::repl;
use sim_core::engine::zones::{self, TOKEN_CAP, Tokens};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// seat 0 with `k` Parallel Lives (the copies Astral Dragon makes are the card's own doubler)
fn doublers(k: usize) -> Game {
    let mut g = table(&["sauron", "seph"]);
    g.dsl_on = true;
    for _ in 0..k {
        perm(&mut g, P0, "Parallel Lives");
    }
    g
}

#[test]
fn a_few_doublers_still_double() {
    let g = doublers(3);
    assert_eq!(repl::token_mult(&g, P0), 8);
    assert_eq!(repl::tokens(&g, P0, 3), 24);
    assert_eq!(repl::token_mult(&g, P1), 1);
}

#[test]
fn the_multiplier_stops_at_the_cap() {
    let g = doublers(8); // 256: just past it
    assert_eq!(repl::token_mult(&g, P0), repl::MULT_CAP);
    let g = doublers(40);
    assert_eq!(repl::MULT_CAP as usize, TOKEN_CAP);
    assert_eq!(repl::token_mult(&g, P0), repl::MULT_CAP);
    assert_eq!(repl::tokens(&g, P0, u32::MAX), u32::MAX, "saturates, doesn't overflow");
}

#[test]
fn forty_doublers_make_treasures_without_overflow() {
    let mut g = doublers(40);
    let before = g.player(P0).treasures;
    // before the cap: the first made u32::MAX Treasures (-1 read as u32), the second panicked in note_tokens
    zones::add_treasure(&mut g, P0, 1).unwrap();
    zones::add_treasure(&mut g, P0, 1).unwrap();
    zones::make_artifact_tokens(&mut g, P0, "Clue", 2).unwrap();
    let cap = repl::MULT_CAP;
    assert_eq!(g.player(P0).treasures, before + 2 * cap);
    assert_eq!(g.player(P0).clues, 2 * cap);
    assert_eq!(g.player(P0).tok_made.1, 4 * cap);
    // creature tokens: the battlefield's cap, as before
    let made = zones::make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap();
    assert!(!made.is_empty() && g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count() <= TOKEN_CAP);
}
