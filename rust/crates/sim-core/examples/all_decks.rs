//! Every deck at every tier, one game each, with the AI given: a check that nothing panics or hangs.
//!     cargo run --release -p sim-core --example all_decks [adaptive|lookahead] [games per pairing]

use sim_core::settings::AiMode;
use sim_core::testkit::*;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ai = if args.get(1).map(|s| s.as_str()) == Some("lookahead") { AiMode::Lookahead } else { AiMode::Adaptive };
    let n: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mine: Vec<&str> = decks().iter().filter(|d| d.tier.is_none()).map(|d| &*d.key).collect();
    let t0 = Instant::now();
    let (mut games, mut timeouts, mut stopped, mut rounds) = (0, 0, 0, 0);
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let pool = tier(t);
        for (i, me) in mine.iter().enumerate() {
            for k in 0..n {
                let seed = 910_000 + 100 * i as u64 + k;
                if std::env::var_os("ALL_DECKS_VERBOSE").is_some() {
                    eprintln!("{t} {me} seed {seed}");
                }
                let g = play_with(&[me, pool[i % 5], pool[(i + 1) % 5], pool[(i + 2) % 5]], seed, false, ai);
                games += 1;
                rounds += g.round;
                timeouts += (g.wintype == Some("timeout")) as u32;
                stopped += g.stopped as u32;
            }
        }
    }
    println!(
        "{games} games ({ai:?}) in {:.1}s: {timeouts} timeouts ({stopped} out of engine steps), {:.1} rounds a game",
        t0.elapsed().as_secs_f64(),
        rounds as f64 / games as f64
    );
}
