//! Integer overflows in Tier 5 games (1-3 of every 2000 panicked in release builds, where overflow is checked): a
//! Thrakkus-style doubling leaves creature power near `i32::MAX`, and the damage, power-sum and life arithmetic around
//! it wrapped. Sites: a `.sum()` of attackers' power in `combat::ai_blocks` / `split_attack` (std's accum.rs:206),
//! `lose_life`'s `n *= fiery(..)` (Fiery Emancipation), and `threat`'s `life - 40` on a total saturated to `i32::MIN`.

use sim_core::engine::chars::POWER_CAP;
use sim_core::engine::combat::{fiery, resolve_combat, split_attack};
use sim_core::engine::life::{LIFE_CAP, check_state, gain, lose_life};
use sim_core::engine::values::{board_power, threat};
use sim_core::ids::{PermId, PlayerId};
use sim_core::satsum::SatSum;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn huge_attackers(g: &mut Game, p: PlayerId, k: usize) -> Vec<PermId> {
    (0..k).map(|_| token(g, p, i32::MAX - 1)).collect()
}

#[test]
fn power_sums_saturate() {
    let mut g = table(&["sauron", "cloud", "seph"]);
    let atk = huge_attackers(&mut g, P0, 3);
    // a power past the cap reads as the cap, so a board of them sums in range; the sum helper itself saturates
    assert_eq!(sim_core::engine::values::epow(&g, atk[0]), POWER_CAP);
    assert_eq!(board_power(&g, P0), 3 * POWER_CAP, "three near-MAX powers: a plain sum wrapped");
    assert_eq!([i32::MAX, i32::MAX, 5].into_iter().ssum(), i32::MAX);
    assert_eq!([i32::MIN, -7].into_iter().ssum(), i32::MIN);
    assert_eq!([3, 4, 5].into_iter().ssum(), 12);
    // the d_dies / free sums in split_attack (the accum.rs panics of seeds 501675, 501270, 501060, 500464)
    let split = split_attack(&g, P0, P1, &atk, &[]);
    assert!(!split.is_empty());
}

#[test]
fn a_huge_attack_resolves_without_a_panic() {
    let mut g = table(&["sauron", "cloud", "seph"]);
    let atk = huge_attackers(&mut g, P0, 3);
    perm(&mut g, P1, "Grave Titan"); // a blocker, so ai_blocks sums `incoming` and weighs the trade
    let connected = resolve_combat(&mut g, P0, &atk, P1, &atk).unwrap();
    assert!(!connected.is_empty(), "the unblocked attackers connected");
    assert!(!g.player(P1).alive, "lethal");
    assert!(g.player(P1).life <= 0 && g.player(P1).life >= -LIFE_CAP);
}

#[test]
fn fiery_emancipation_triples_and_saturates() {
    let mut g = table(&["sauron", "cloud"]);
    assert_eq!(fiery(&g, P0), 1);
    perm(&mut g, P0, "Fiery Emancipation");
    assert_eq!(fiery(&g, P0), 3);
    lose_life(&mut g, P1, 5, Some(P0), intern("burn"), None).unwrap();
    assert_eq!(g.player(P1).life, 40 - 15, "normal games play as before");
    // seed 501342: a huge hit through the multiplier overflowed `n *= fiery`
    lose_life(&mut g, P1, i32::MAX / 2, Some(P0), intern("burn"), None).unwrap();
    assert_eq!(g.player(P1).life, -LIFE_CAP);
    check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

#[test]
fn a_stack_of_multipliers_saturates() {
    let mut g = table(&["sauron", "cloud"]);
    for _ in 0..24 {
        perm(&mut g, P0, "Fiery Emancipation"); // 3^24 does not fit an i32
    }
    assert_eq!(fiery(&g, P0), i32::MAX);
    lose_life(&mut g, P1, 7, Some(P0), intern("burn"), None).unwrap();
    check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

#[test]
fn life_stays_in_range_and_threat_does_not_overflow() {
    let mut g = table(&["sauron", "cloud"]);
    // seed 500224: life saturated to i32::MIN, then `pl.life - 40` in `threat` subtracted past it
    g.player_mut(P1).life = i32::MIN;
    let t = threat(&g, P0, P1);
    assert!(t < -1.0e7, "a dead seat is no threat: {t}");
    lose_life(&mut g, P1, i32::MAX, Some(P0), intern("drain"), None).unwrap();
    g.player_mut(P1).life = 40;
    lose_life(&mut g, P1, i32::MAX, Some(P0), intern("drain"), None).unwrap();
    assert_eq!(g.player(P1).life, -LIFE_CAP, "clamped, so `life - cost` checks around the AI stay in range");
    let _ = threat(&g, P0, P1);
    // life gain clamps too
    g.player_mut(P0).life = LIFE_CAP - 1;
    gain(&mut g, P0, i32::MAX).unwrap();
    assert_eq!(g.player(P0).life, LIFE_CAP);
}
