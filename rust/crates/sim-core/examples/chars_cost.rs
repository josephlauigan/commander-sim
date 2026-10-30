//! How long reading a permanent's characteristics takes (Phase 3, the characteristics pass): every reader
//! (`is_creature`, power, toughness, `has_type`, `subtypes`, `colors_of`, `has_kw`) over every permanent of a
//! mid-game table (the `clone_cost` board: four players, 8 lands and 15 permanents each, an anthem in play, the
//! ability language on). Not a sim: one fixed board, read again and again.
//!     cargo run --release -p sim-core --example chars_cost

use sim_core::dsl;
use sim_core::engine::values;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Val};
use sim_core::sym::intern;
use sim_core::testkit;
use std::hint::black_box;
use std::time::Instant;

fn main() {
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
            if k % 4 == 0 {
                g.perm_mut(m).data.set(DataKey::Counters, Val::Int(2));
                g.perm_mut(m).plus = 1;
            }
            g.perm_mut(m).on_bf = true;
            g.player_mut(p).perms.push(m);
        }
    }
    for name in ["Glorious Anthem", "Maskwood Nexus", "Elesh Norn, Grand Cenobite"] {
        if let Some(c) = g.db.id(name) {
            let m = g.new_perm(PlayerId(0), Some(c), intern("Token"), 1, 1);
            g.perm_mut(m).on_bf = true;
            g.player_mut(PlayerId(0)).perms.push(m);
        }
    }
    g.dsl_on = true;
    let all: Vec<PermId> = g.players.iter().flat_map(|q| q.perms.iter().chain(q.lands.iter()).copied()).collect();
    let n = 2_000;
    let reads = (n * all.len()) as f64;
    let time = |label: &str, f: &dyn Fn(PermId) -> i64| {
        let t = Instant::now();
        let mut keep = 0i64;
        for _ in 0..n {
            for &m in &all {
                keep += f(black_box(m));
            }
        }
        println!("{label:12} {:7.1} ns per permanent ({keep})", t.elapsed().as_secs_f64() / reads * 1e9);
    };
    time("is_creature", &|m| g.is_creature(m) as i64);
    time("power", &|m| values::epow(&g, m) as i64);
    time("toughness", &|m| values::etgh(&g, m) as i64);
    time("has_type", &|m| values::has_type(&g, m, "elf") as i64);
    time("subtypes", &|m| values::subtypes(&g, m).len() as i64);
    time("colors_of", &|m| values::colors_of(&g, m).count() as i64);
    time("has_kw", &|m| dsl::has_kw(&g, m, "flying") as i64);
    time("all readers", &|m| {
        g.is_creature(m) as i64
            + values::epow(&g, m) as i64
            + values::etgh(&g, m) as i64
            + values::has_type(&g, m, "elf") as i64
            + values::subtypes(&g, m).len() as i64
            + values::colors_of(&g, m).count() as i64
            + dsl::has_kw(&g, m, "flying") as i64
    });
    time("chars()", &|m| sim_core::engine::chars::chars(&g, m).power as i64);
    println!("{} permanents and lands", all.len());
}
