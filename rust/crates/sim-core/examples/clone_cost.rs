//! How long copying a game takes (the look-ahead copies one per playout; Python's deepcopy took about 4 ms under
//! the profiler). A mid-game table: four players with full zones, 8 lands and 15 permanents each, some with state.
//!     cargo run --release -p sim-core --example clone_cost

use sim_core::cards::CardDb;
use sim_core::export::load_decks;
use sim_core::ids::{CardId, PlayerId};
use sim_core::rng::Rng;
use sim_core::settings::{AiMode, Profile, Settings};
use sim_core::state::{DataKey, Game, Land, PermData, Player, Val};
use sim_core::sym::intern;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let db = Arc::new(CardDb::load(&data.join("cards.json")).unwrap());
    let decks = load_decks(&data.join("decks.json")).unwrap();
    let seats: Vec<_> = decks.pool.iter().filter(|d| d.tier.as_deref() == Some("t4")).take(4).collect();
    let players = seats
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let ids: Arc<[CardId]> = d.cards.iter().map(|n| db.id(n).unwrap()).collect();
            let cmd = db.id(&d.commander).unwrap();
            Player::new(PlayerId(i as u8), intern(&d.key), Default::default(), cmd, ids)
        })
        .collect();
    let settings = Arc::new(Settings::new(Profile::Loose, AiMode::Lookahead, 1.0));
    let mut g = Game::new(db.clone(), settings, players, Rng::named("play:1"));
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
