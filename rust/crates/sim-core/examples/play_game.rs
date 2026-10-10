//! A whole game in Rust with its play-by-play: Sauron against three Tier 1 decks.
//!     cargo run --release -p sim-core --example play_game [seed]

use sim_core::testkit;

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(500001);
    let t1 = testkit::tier("t1");
    let g = testkit::play(&["sauron", t1[0], t1[1], t1[2]], seed, true);
    for line in g.log.as_ref().unwrap() {
        println!("{line}");
    }
    let winner = g.winner.map_or("nobody", |w| g.player(w).name);
    println!("\nwinner: {winner} ({}), round {}, {} engine steps", g.wintype.unwrap_or("?"), g.round, g.work);
}
