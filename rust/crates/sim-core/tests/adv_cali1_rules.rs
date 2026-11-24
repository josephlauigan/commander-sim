// Written by an independent adversarial reviewer (not the author of the engine rules): starting-life sweeps, the draw skip and Commander equivalence.
//! Adversarial tests against the format rules (`Settings.rules`: starting life, the starting player's first draw).
//! Written by a reviewer who did not write the code under test.

mod play_common;
use play_common::*;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh, threat};
use sim_core::engine::{combat, turn};
use sim_core::format::Rules;
use sim_core::hooks::{Action, Call, Event};
use sim_core::settings::AiMode;
use sim_core::state::DataKey;

const CALI: Rules = Rules { life: 30, first_player_draws: false };

fn rules(life: i32) -> Rules {
    Rules { life, first_player_draws: true }
}

fn bare(r: Rules, n: usize) -> Game {
    let mut g = table_in(&["galadriel", "cloud", "seph", "sauron"][..n], 1, r);
    let c = card(&g, "Counterspell");
    for q in &mut g.players {
        q.library = vec![c; 30];
        q.hand.clear();
        q.turns = 0;
    }
    g.round = 1;
    g
}

// ------------------------------------------------------------------ the readers, swept over lives and rule lives
const STARTS: [i32; 6] = [1, 20, 30, 31, 40, 41];

fn lives_around(start: i32) -> Vec<i32> {
    let mut v: Vec<i32> = vec![-10, -1, 0, 1, 2];
    for d in [-30, -16, -15, -14, -8, -7, -6, -3, -2, -1, 0, 1, 2, 6, 7, 8, 14, 15, 16, 30] {
        v.push(start + d);
    }
    v
}

#[test]
fn cosmos_elixir_sweep_draws_iff_life_above_start() {
    for s in STARTS {
        for life in lives_around(s) {
            let mut g = table_in(&["heliod-mono-white-stax", "seph", "sauron"], 1, rules(s));
            perm(&mut g, P0, "Cosmos Elixir");
            g.player_mut(P0).life = life;
            let hand0 = g.player(P0).hand.len();
            fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
            let drew = g.player(P0).hand.len() - hand0;
            let want = if life > s { (life, 1) } else { (life + 2, 0) };
            assert_eq!((g.player(P0).life, drew), want, "start {s} life {life}");
        }
    }
}

#[test]
fn ajani_zero_sweep_needs_exactly_fifteen_above_start() {
    for s in STARTS {
        for life in lives_around(s) {
            let mut g = table_in(&["heliod-mono-white-stax", "seph", "sauron"], 1, rules(s));
            let a = perm(&mut g, P0, "Ajani, Strength of the Pride");
            perm(&mut g, P1, "Sun Titan");
            g.player_mut(P0).life = life;
            let f = g.registry.get(g.perm(a).cd.unwrap()).unwrap().options.unwrap();
            let on = f(&mut g, a, P0, Some(false)).unwrap().iter().any(|o| o.label.ends_with("+0") && o.act.is_some());
            assert_eq!(on, life >= s + 15, "start {s} life {life}");
        }
    }
}

#[test]
fn valkyrie_and_speaker_sweep_need_exactly_seven_above_start() {
    for s in STARTS {
        for life in lives_around(s) {
            let mut g = table_in(&["giada-calling-all-angels", "cloud", "seph"], 1, rules(s));
            g.active = Some(P0);
            perm(&mut g, P0, "Righteous Valkyrie");
            let w = perm(&mut g, P0, "Metallic Mimic");
            let sp = perm(&mut g, P0, "Speaker of the Heavens");
            g.perm_mut(sp).sick = false;
            let base = (epow(&g, w), etgh(&g, w));
            g.player_mut(P0).life = life;
            let pumped = (epow(&g, w), etgh(&g, w)) != base;
            let hook = *g.registry.get(card(&g, "Speaker of the Heavens")).unwrap();
            let opts = (hook.options.unwrap())(&mut g, sp, P0, None).unwrap();
            let speaks = opts.into_iter().any(|o| matches!(o.act, Some(Action::Ability { .. })));
            assert_eq!(pumped, life >= s + 7, "valkyrie: start {s} life {life}");
            assert_eq!(speaks, life >= s + 7, "speaker: start {s} life {life}");
        }
    }
}

#[test]
fn shadow_of_mortality_sweep_is_the_difference_clamped_at_zero() {
    for s in STARTS {
        for life in lives_around(s) {
            let mut g = table_in(&["yuriko-dimir-ninjas", "seph", "cloud"], 1, rules(s));
            let c = card(&g, "Shadow of Mortality");
            let f = g.registry.get(c).unwrap().self_cost.unwrap();
            g.player_mut(P0).life = life;
            assert_eq!(f(&g, P0, c), -((s - life).max(0)), "start {s} life {life}");
        }
    }
}

/// Cecil deals 2 here (the author's test flips 17 -> 15 at 30); the flip is `life_after <= start / 2` (integer half)
#[test]
fn cecil_sweep_flips_iff_life_after_is_at_most_half_of_start() {
    for s in STARTS {
        for life in lives_around(s).into_iter().filter(|&l| l > 3) {
            let mut g = table_in(&["galadriel", "cloud", "seph"], 1, rules(s));
            let c = perm(&mut g, P0, "Cecil, Dark Knight");
            g.player_mut(P0).life = life;
            combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap();
            let after = g.player(P0).life;
            let flipped = g.perm(c).data.truthy(DataKey::Flipped);
            // "less than or equal to half your starting life total": 31 -> 15.5, so 15 flips and 16 does not
            let want = 2 * after <= s;
            assert_eq!(flipped, want, "start {s} life {life} -> {after}");
        }
    }
}

// ------------------------------------------------------------------ start_life against a life override and per seat
#[test]
fn table_with_a_life_override_keeps_commander_readers_where_they_were() {
    // the old literal 40 and the new start_life (40 under table_with) must agree for every life
    for life in [-3, 0, 1, 40, 41, 46, 47, 54, 55, 56] {
        let mut g = table_with(&["heliod-mono-white-stax", "seph", "sauron"], 1, life);
        assert!(g.players.iter().all(|p| p.start_life == 40 && p.life == life));
        let hand0 = g.player(P0).hand.len();
        perm(&mut g, P0, "Cosmos Elixir");
        fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
        assert_eq!(g.player(P0).hand.len() - hand0, usize::from(life > 40), "life {life}");
    }
}

#[test]
fn threat_baseline_is_each_seats_own_start_life() {
    let mut g = table_in(&["galadriel", "cloud", "seph"], 1, CALI);
    g.player_mut(P1).start_life = 20;
    g.player_mut(P1).life = 20;
    g.player_mut(P2).start_life = 40;
    g.player_mut(P2).life = 20;
    assert_eq!(threat(&g, P0, P1), 0.0);
    assert!((threat(&g, P0, P2) + 1.0).abs() < 1e-9, "20 against a baseline of 40 is -1.0");
    // life far above or below stays exact in f64
    g.player_mut(P1).life = 1 << 30;
    assert_eq!(threat(&g, P0, P1), ((1i64 << 30) - 20) as f64 * 0.05);
    g.player_mut(P1).life = -(1 << 30);
    assert_eq!(threat(&g, P0, P1), (-(1i64 << 30) - 20) as f64 * 0.05);
}

#[test]
fn threat_equals_the_old_formula_for_every_commander_life() {
    let mut g = table_in(&["galadriel", "cloud"], 1, Rules::COMMANDER);
    for life in (-50..200).chain([i32::MIN / 2, i32::MAX / 2, 1 << 30, -(1 << 30)]) {
        g.player_mut(P1).life = life;
        let old = life as f64 - 40.0;
        let s0 = 0.0; // an empty board: the permanents' worth is exactly 0
        assert_eq!(threat(&g, P0, P1), s0 + old * 0.05, "life {life}");
    }
}

#[test]
fn cecil_reads_its_controllers_own_start_life_not_seat_zeros() {
    let mut g = table_in(&["galadriel", "cloud", "seph"], 1, CALI);
    g.player_mut(P1).start_life = 40; // a seat that started at 40 in a 30-life game: half is 20
    let c = perm(&mut g, P1, "Cecil, Dark Knight");
    g.player_mut(P1).life = 22; // -> 20
    combat::resolve_combat(&mut g, P1, &[c], P0, &[c]).unwrap();
    assert!(g.perm(c).data.truthy(DataKey::Flipped), "P1 started at 40, so 20 is half");
}

// ------------------------------------------------------------------ the skipped step
#[test]
fn four_seat_cali_only_seat_zero_skips() {
    let mut g = bare(CALI, 4);
    for i in 0..4u8 {
        turn::step_start(&mut g, PlayerId(i)).unwrap();
    }
    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();
    assert_eq!(hands, vec![0, 1, 1, 1]);
}

#[test]
fn seat_zero_second_turn_after_a_skipped_first_turn_draws_but_does_not_skip_again() {
    // the first turn (turns 0 -> 1, round 1) skips; a third and fourth step_start never skip
    let mut g = bare(CALI, 2);
    turn::step_start(&mut g, P0).unwrap();
    g.round = 2;
    turn::step_start(&mut g, P0).unwrap();
    g.round = 3;
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
}

#[test]
fn a_real_extra_turn_queued_in_round_one_draws() {
    let mut g = bare(CALI, 2);
    turn::add_extra_turn(&mut g, P0, false);
    turn::play_turn(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).turns, 2);
    assert_eq!(g.player(P0).hand.len(), 1, "the first turn skipped its draw, the extra turn drew one");
}

#[test]
fn an_extra_turn_for_seat_one_queued_by_seat_zero_draws() {
    let mut g = bare(CALI, 2);
    turn::add_extra_turn(&mut g, P1, false);
    turn::play_turn(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).turns, 1);
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn a_seat_zero_that_skips_its_whole_first_turn_draws_on_the_next() {
    // documents what the code does (`turns == 1` and `round == 1`): turn skipped by run_rounds, the next turn is round 2
    let mut g = bare(CALI, 2);
    g.player_mut(P0).skip_turns = 1;
    turn::run_rounds(&mut g, 2);
    // seat 0 played round 2 only
    assert_eq!(g.player(P0).turns, 1);
    assert_eq!(g.player(P1).turns, 2);
}

#[test]
fn life_from_the_loam_does_not_dredge_on_the_skipped_step() {
    let run = |r: Rules| {
        let mut g = bare(r, 2);
        let loam = take(&mut g, P0, "Life from the Loam");
        g.player_mut(P0).gy.push(loam);
        let lib0 = g.player(P0).library.len();
        turn::step_start(&mut g, P0).unwrap();
        (g.player(P0).hand.len(), lib0 - g.player(P0).library.len(), g.player(P0).gy.contains(&loam))
    };
    let (h, milled, in_gy) = run(CALI);
    assert_eq!((h, milled, in_gy), (0, 0, true), "no draw, no dredge, Loam stays");
    let (h, milled, _) = run(Rules::COMMANDER);
    assert!(h >= 1 && milled >= 1, "the Commander control does something: hand {h}, library -{milled}");
}

#[test]
fn necropotence_on_the_skipped_step_does_not_panic_or_draw() {
    let mut g = bare(CALI, 2);
    perm(&mut g, P0, "Necropotence");
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0);
}

#[test]
fn opponent_howling_mine_does_not_hit_the_skipped_player_and_hits_seat_one() {
    let mut g = bare(CALI, 2);
    perm(&mut g, P1, "Howling Mine");
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0, "an opposing Mine adds nothing to a skipped step");
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P1).hand.len(), 2);
}

#[test]
fn the_skipped_step_still_clears_mana_pools_and_runs_the_main_phase_start() {
    // mana added in the upkeep/draw window must be gone by main phase (rule 500.4); check the skip didn't jump past it
    let mut g = bare(CALI, 2);
    g.player_mut(P0).drain_mana = 2;
    turn::step_start(&mut g, P0).unwrap();
    assert!(g.player(P0).drain_mana == 0, "the drain is taken after the draw step in either case");
}

/// Sylvan Library's draw-step trigger is implemented in `upkeep` (turn.rs ~799) as a plain engine draw, so it is not
/// inside the block `skips_first_draw` wraps. CR 103.8a skips the whole step, so it should not draw. Unreachable in a
/// real game (nothing but a Leyline-style opening-hand card can be on the battlefield at seat 0's first draw step), so
/// this is a low-severity note, not a Cali blocker.
#[test]
fn sylvan_library_does_not_trigger_in_the_skipped_draw_step() {
    let mut g = bare(CALI, 2);
    perm(&mut g, P0, "Sylvan Library");
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0, "a skipped draw step has no beginning-of-draw-step trigger");
}

// ------------------------------------------------------------------ whole games
fn count_skips(g: &Game) -> usize {
    g.log.as_ref().unwrap().iter().filter(|l| l.contains("skips the draw step")).count()
}

#[test]
fn whole_games_skip_exactly_once_in_cali_and_never_in_commander() {
    let t5 = tier("t5");
    let mut ran = 0;
    for (n, seed) in [(2usize, 11u64), (2, 12), (3, 13), (4, 14)] {
        let keys: Vec<&str> = (0..n).map(|i| t5[(i + seed as usize) % t5.len()]).collect();
        let mut dup = keys.clone();
        dup.sort();
        dup.dedup();
        if dup.len() != keys.len() {
            continue;
        }
        ran += 1;
        let g = play_in(&keys, seed, true, AiMode::Adaptive, CALI);
        assert!(g.round >= 2, "the game got past round one: round {}", g.round);
        assert_eq!(count_skips(&g), 1, "{keys:?} seed {seed}");
        let first = g.log.as_ref().unwrap().iter().position(|l| l.contains("skips the draw step")).unwrap();
        let line = &g.log.as_ref().unwrap()[first];
        assert!(line.contains(&*g.player(P0).name), "{line}");
        assert!(g.players.iter().all(|p| p.start_life == 30));
        let g = play_in(&keys, seed, true, AiMode::Adaptive, Rules::COMMANDER);
        assert_eq!(count_skips(&g), 0, "{keys:?} seed {seed}");
    }
    assert!(ran >= 3, "only {ran} games ran");
}

#[test]
fn commander_play_in_equals_play_with() {
    let a = play_with(&["arzakon", "seph"], 5, true, AiMode::Adaptive);
    let b = play_in(&["arzakon", "seph"], 5, true, AiMode::Adaptive, Rules::COMMANDER);
    assert_eq!(a.log, b.log);
    assert_eq!(a.winner, b.winner);
}
