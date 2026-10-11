//! How long copying a game takes (the look-ahead copies one per playout; Python's deepcopy took about 4 ms under
//! the profiler). A mid-game table: four players with full zones, 8 lands and 15 permanents each, some with state.
//!     cargo run --release -p sim-core --example clone_cost

use sim_core::ids::PlayerId;
use sim_core::state::{DataKey, Land, PermData, Val};
use sim_core::sym::intern;
use sim_core::testkit;
use std::time::Instant;

fn main() {
    let t4: Vec<&str> =
        testkit::decks().iter().filter(|d| d.tier.as_deref() == Some("t4")).take(4).map(|d| &*d.key).collect();
    let mut g = testkit::table(&t4);
    for i in 0..4 {
        let p = PlayerId(i);
        for _ in 0..7 {
            let c = g.player_mut(p).library.pop().unwrap();
            g.player_mut(p).hand.push(c);
        }
        for _ in 0..10 {
            let c = g.player_mut(p).library.pop().unwrap();
            g.player_mut(p).gy.push(c);
        }
        for _ in 0..8 {
            let c = g.player_mut(p).library.pop().unwrap();
            let id = sim_core::ids::LandId(g.lands.len() as u32);
            g.lands.push(Land { id, cd: c, owner: p, tapped: false, data: PermData::default(), on_bf: true });
            g.player_mut(p).lands.push(id);
        }
        for k in 0..15 {
            let c = g.player(p).library[k];
            let m = g.new_perm(p, Some(c), intern("Token"), 1, 1);
            if k % 4 == 0 {
                g.perm_mut(m).data.set(DataKey::Counters, Val::Int(2));
            }
            g.perm_mut(m).on_bf = true;
            g.player_mut(p).perms.push(m);
        }
        for s in ["casts", "draws", "flicker Conjurer's Closet", "land_miss"] {
            g.player_mut(p).stat(intern(s), 1);
        }
    }
    let n = 100_000;
    let t = Instant::now();
    let mut keep = 0usize;
    for _ in 0..n {
        let c = g.clone();
        keep += c.perms.len();
    }
    let per = t.elapsed().as_secs_f64() / n as f64;
    println!("{} permanents, {} lands: {:.2} µs per copy ({keep})", g.perms.len(), g.lands.len(), per * 1e6);
}
