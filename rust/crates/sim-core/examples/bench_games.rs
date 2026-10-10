//! Whole games in a row, timed: Sauron against three decks of a tier, with the heuristic AI or the look-ahead.
//!     cargo run --release -p sim-core --example bench_games [n] [adaptive|lookahead] [tier]

use sim_core::settings::AiMode;
use sim_core::testkit;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);
    let ai = match args.get(2).map(|s| s.as_str()) {
        Some("lookahead") => AiMode::Lookahead,
        _ => AiMode::Adaptive,
    };
    let tier = args.get(3).map(|s| s.as_str()).unwrap_or("t1");
    let decks = testkit::tier(tier);
    let mut wins: std::collections::BTreeMap<String, u32> = Default::default();
    let mut rounds = 0;
    let t0 = Instant::now();
    for i in 0..n {
        let k = (i as usize * 3) % decks.len();
        let opp = [decks[k], decks[(k + 1) % decks.len()], decks[(k + 2) % decks.len()]];
        let g = testkit::play_with(&["sauron", opp[0], opp[1], opp[2]], 700000 + i, false, ai);
        let w = g.winner.map_or("nobody".to_string(), |w| g.player(w).key.to_string());
        *wins.entry(w).or_default() += 1;
        rounds += g.round;
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("{n} games ({ai:?}, {tier}) in {dt:.2}s: {:.1} games/s, {:.1} rounds a game", n as f64 / dt, rounds as f64 / n as f64);
    for (k, v) in wins {
        println!("  {k}: {v}");
    }
    let s = sim_core::ai::search::stats();
    println!("look-ahead: {} decisions, {} playouts, {} changed, {} cut", s.decisions, s.playouts, s.changed, s.cut);
}
