//! t5-hold: the held-mana counters (`holdstats`) are trace-only, and they count what they say.
//!
//! - a seeded game is played to the same result, log and counters with them on and off;
//! - a held counterspell that counters a spell before its holder's next turn is a `used` hold with the spell's class.

use sim_core::engine::stack::{self};
use sim_core::engine::turn::{run_rounds, setup_game};
use sim_core::holdstats;
use sim_core::ids::PlayerId;
use sim_core::settings::{AiMode, Profile, Settings};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game(keys: &[&str], seed: u64, ai: AiMode, trace: bool) -> Game {
    let seats: Vec<_> = keys.iter().map(|k| seat_spec(k)).collect();
    let settings = Arc::new(Settings::new(Profile::Loose, ai, 1.0));
    let mut g = setup_game(db(), registry(), settings, seed, &seats, trace);
    run_rounds(&mut g, 20);
    g
}

fn game_off(keys: &[&str], seed: u64, ai: AiMode, trace: bool) -> Game {
    let seats: Vec<_> = keys.iter().map(|k| seat_spec(k)).collect();
    let mut s = Settings::new(Profile::Loose, ai, 1.0);
    s.hold_trace = false;
    let mut g = setup_game(db(), registry(), Arc::new(s), seed, &seats, trace);
    run_rounds(&mut g, 20);
    g
}

fn plain_stats(g: &Game, p: usize) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> =
        g.players[p].stats.iter().filter(|(k, _)| !k.starts_with("hd|")).map(|(k, v)| (k.to_string(), *v)).collect();
    v.sort();
    v
}

#[test]
fn a_seeded_game_is_played_the_same_with_the_counters_on() {
    let pod = tier("t5");
    assert!(pod.len() >= 4, "Tier 5 has decks");
    for (seed, ai) in [(900_001u64, AiMode::Adaptive), (900_002, AiMode::Adaptive), (900_003, AiMode::Lookahead)] {
        let keys: Vec<&str> = (0..4).map(|i| pod[(seed as usize + i * 3) % pod.len()]).collect();
        let (on, off) = (game(&keys, seed, ai, true), game_off(&keys, seed, ai, true));
        assert_eq!(on.winner, off.winner, "seed {seed} {keys:?}: winner");
        assert_eq!(on.round, off.round, "seed {seed} {keys:?}: rounds");
        assert_eq!(on.log, off.log, "seed {seed} {keys:?}: the game log");
        for p in 0..4 {
            assert_eq!(plain_stats(&on, p), plain_stats(&off, p), "seed {seed}: seat {p}'s other counters");
            assert!(!off.players[p].stats.keys().any(|k| k.starts_with("hd|")), "off: no counters kept");
        }
        let n: i64 = on.players.iter().map(|p| p.stats.get("hd|all|turns").copied().unwrap_or(0)).sum();
        assert!(n > 0, "seed {seed}: on, the turns were counted");
        for p in &on.players {
            let s = |k: &str| p.stats.get(k).copied().unwrap_or(0);
            for gp in ["ctr", "prot", "rem", "adn", "oth", "idle"] {
                let done: i64 =
                    ["used", "castnh", "else", "unused"].iter().map(|o| s(&format!("hd|all|{gp}|{o}"))).sum();
                assert!(done <= s(&format!("hd|all|{gp}|n")), "seed {seed}: {gp}: more outcomes than holds");
            }
            assert!(s("hd|mid|turns") <= s("hd|all|turns"));
        }
    }
}

#[test]
fn a_held_counterspell_that_counters_a_spell_is_a_used_hold() {
    let mut g = table(&["sauron", "cloud"]);
    hand(&mut g, P1, &["Counterspell"]);
    lands(&mut g, P1, "Island", 2, false);
    g.player_mut(P1).turns = 6;
    holdstats::snapshot(&mut g, P1);
    let st = |g: &Game, k: &str| g.player(P1).stats.get(k).copied().unwrap_or(0);
    assert_eq!(st(&g, "hd|all|turns"), 1);
    assert_eq!(st(&g, "hd|mid|ctr|n"), 1, "a counterspell is held on turn 6");
    assert_eq!(st(&g, "hd|mid|ctr|mana"), 2);
    // P0 casts a creature; the holder counters it
    let sheo = card(&g, "Sheoldred, the Apocalypse");
    let it = stack::spell_item(&mut g, P0, sheo, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    let ctr = *g.player(P1).hand.last().unwrap();
    assert!(stack::cast_counter_spell(&mut g, P1, ctr, id).unwrap());
    holdstats::turn_start(&mut g, P1); // its next turn
    assert_eq!(st(&g, "hd|mid|ctr|used"), 1);
    assert_eq!(st(&g, "hd|mid|ctr|cast"), 1);
    assert_eq!(st(&g, "hd|mid|ctr|hit|creature"), 1, "Sheoldred is a creature");
    assert_eq!(st(&g, "hd|mid|ctr|spent"), 2, "the two mana of Counterspell");
    assert_eq!(st(&g, "hd|mid|ctr|unused"), 0);
    // the next turn ends with the mana up and nothing cast: an unused hold
    hand(&mut g, P1, &["Counterspell"]);
    g.player_mut(P1).turns = 7;
    for l in g.player(P1).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    holdstats::snapshot(&mut g, P1);
    holdstats::turn_start(&mut g, P1);
    assert_eq!(st(&g, "hd|mid|ctr|n"), 2);
    assert_eq!(st(&g, "hd|mid|ctr|unused"), 1);
}
