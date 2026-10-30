//! How long asking the replacement effects takes (Phase 3, the replacement step): counters placed, poison given,
//! tokens made, damage prevented and life gained, over every permanent of the `chars_cost` mid-game table (four
//! players, 8 lands and 15 permanents each, the ability language on), first with no replacement effect in play, then
//! with Doubling Season, Hardened Scales and Vorinclex for player 0. Not a sim: one fixed board, asked again and again.
//!     cargo run --release -p sim-core --example repl_cost

use sim_core::cardcode;
use sim_core::dsl;
use sim_core::engine::values;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit;
use std::hint::black_box;
use std::time::Instant;

fn board() -> Game {
    let t4: Vec<&str> =
        testkit::decks().iter().filter(|d| d.tier.as_deref() == Some("t4")).take(4).map(|d| &*d.key).collect();
    let mut g = testkit::table(&t4);
    for i in 0..4 {
        let p = PlayerId(i);
        for _ in 0..8 {
            let c = g.player_mut(p).library.pop().unwrap();
            g.add_land(p, c, false);
        }
        for k in 0..15 {
            let c = g.player(p).library[k];
            let m = g.new_perm(p, Some(c), intern("Token"), 1, 1);
            g.perm_mut(m).on_bf = true;
            g.player_mut(p).perms.push(m);
        }
    }
    g.dsl_on = true;
    g
}

fn add(g: &mut Game, name: &str) {
    if let Some(c) = g.db.id(name) {
        let m = g.new_perm(PlayerId(0), Some(c), intern("Token"), 1, 1);
        g.perm_mut(m).on_bf = true;
        g.player_mut(PlayerId(0)).perms.push(m);
    }
}

fn run(g: &Game, label: &str) {
    let all: Vec<PermId> = g.players.iter().flat_map(|q| q.perms.iter().copied()).collect();
    let n = 2_000;
    let reads = (n * all.len()) as f64;
    let time = |what: &str, f: &dyn Fn(PermId) -> i64| {
        let t = Instant::now();
        let mut keep = 0i64;
        for _ in 0..n {
            for &m in &all {
                keep += f(black_box(m));
            }
        }
        println!("{label:8} {what:14} {:7.1} ns per ask ({keep})", t.elapsed().as_secs_f64() / reads * 1e9);
    };
    let p0 = PlayerId(0);
    time("+1/+1 counters", &|m| {
        let o = g.perm(m).owner;
        let k = 2 * dsl::counter_mult(g, o);
        let k = k + if g.is_creature(m) { cardcode::scales(g, o) * dsl::counter_mult(g, o) } else { 0 };
        cardcode::vorinclex(g, p0, k) as i64
    });
    time("-1/-1 counters", &|m| {
        let o = g.perm(m).owner;
        if values::melira(g, o) { 0 } else { cardcode::vorinclex(g, p0, -1) as i64 }
    });
    time("poison", &|m| {
        let o = g.perm(m).owner;
        if values::melira(g, o) { 0 } else { cardcode::vorinclex(g, p0, 1) as i64 }
    });
    time("tokens", &|m| dsl::token_mult(g, g.perm(m).owner) as i64);
    time("damage", &|m| values::no_damage(g, m) as i64);
    time("life gain", &|m| dsl::lifegain_mod(g, g.perm(m).owner, 3) as i64);
}

fn main() {
    let mut g = board();
    run(&g, "empty");
    for name in ["Doubling Season", "Hardened Scales", "Vorinclex, Monstrous Raider"] {
        add(&mut g, name);
    }
    run(&g, "doublers");
    println!("{} permanents", g.players.iter().map(|q| q.perms.len()).sum::<usize>());
}
