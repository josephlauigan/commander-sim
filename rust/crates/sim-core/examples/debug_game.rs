//! Look at one game's end state (development aid).
//!     cargo run --release -p sim-core --example debug_game <deck> <tier> <seed>

use sim_core::testkit;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (me, t, seed) = (&a[1], &a[2], a[3].parse::<u64>().unwrap());
    let pool = testkit::tier(t);
    let mine: Vec<&str> = testkit::decks().iter().filter(|d| d.tier.is_none()).map(|d| &*d.key).collect();
    let i = mine.iter().position(|k| k == me).unwrap();
    let g = testkit::play(&[me, pool[i % 5], pool[(i + 1) % 5], pool[(i + 2) % 5]], seed, true);
    let log = g.log.as_ref().unwrap();
    for line in &log[log.len().saturating_sub(25)..] {
        println!("{line}");
    }
    println!(
        "\nover {} stopped {} round {} work {} wintype {:?} stack {} queue {} dsl_depth {}",
        g.over,
        g.stopped,
        g.round,
        g.work,
        g.wintype,
        g.stack.len(),
        g.trig_queue.len(),
        g.dsl_depth
    );
    for it in &g.stack {
        println!("  stack: {}", it.name);
    }
    for t in &g.trig_queue {
        println!("  queued: {:?}", t.act);
    }
}
