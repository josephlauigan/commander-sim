//! Building game positions for tests and examples: Python's `tests/table.py`.
//!
//! ```ignore
//! let mut g = table(&["veyran", "seph"]);          // two seats, before turn one, the first seat active in main 1
//! let (me, opp) = (PlayerId(0), PlayerId(1));
//! lands(&mut g, me, "Island", 2); hand(&mut g, me, &["Counterspell"]); perm(&mut g, opp, "Grave Titan");
//! ```
//! Games use the heuristic AI and the conservative profile, as the Python tests do.

use crate::cards::CardDb;
use crate::export::{RawDeck, load_decks};
use crate::hooks::Registry;
use crate::ids::{CardId, LandId, PlayerId};
use crate::rng::Rng;
use crate::settings::{AiMode, Profile, Settings};
use crate::state::{Game, Player, Step};
use crate::sym::intern;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

pub fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data")
}

/// The card database, loaded once per process.
pub fn db() -> Arc<CardDb> {
    static DB: OnceLock<Arc<CardDb>> = OnceLock::new();
    DB.get_or_init(|| Arc::new(CardDb::load(&data_dir().join("cards.json")).unwrap())).clone()
}

/// Every deck in decks.json (yours, then the pools), loaded once.
pub fn decks() -> &'static [RawDeck] {
    static D: OnceLock<Vec<RawDeck>> = OnceLock::new();
    D.get_or_init(|| {
        let f = load_decks(&data_dir().join("decks.json")).unwrap();
        f.mine.into_iter().chain(f.pool).collect()
    })
}

pub fn deck(key: &str) -> &'static RawDeck {
    decks().iter().find(|d| d.key == key).unwrap_or_else(|| panic!("no deck {key:?}"))
}

/// The card code. PORT(M5): fills as cards are ported; empty until then.
pub fn registry() -> Arc<Registry> {
    static R: OnceLock<Arc<Registry>> = OnceLock::new();
    R.get_or_init(|| Arc::new(Registry::new(&db()))).clone()
}

/// A player for deck `key` in seat `i`, its library shuffled from the seat's own stream.
pub fn seat(db: &CardDb, i: usize, key: &str, seed: u64) -> Player {
    let d = deck(key);
    let ids: Arc<[CardId]> = d.cards.iter().map(|n| db.id(n).unwrap()).collect();
    let cmd = db.id(&d.commander).unwrap();
    let mut p = Player::new(
        PlayerId(i as u8),
        intern(&d.key),
        intern(&d.name),
        db.get(cmd).identity.unwrap_or_default(),
        cmd,
        ids,
    );
    Rng::named(&format!("lib:{seed}:{key}")).shuffle(&mut p.library);
    p
}

/// A game with these decks seated in this order: hands empty, the first seat active in its first main phase.
pub fn table(keys: &[&str]) -> Game {
    table_with(keys, 1, 40)
}

pub fn table_with(keys: &[&str], seed: u64, life: i32) -> Game {
    let db = db();
    let players = keys.iter().enumerate().map(|(i, k)| seat(&db, i, k, seed)).collect();
    let settings = Arc::new(Settings::new(Profile::Conservative, AiMode::Adaptive, 1.0));
    let mut g = Game::new(db, registry(), settings, players, Rng::named(&format!("play:{seed}")));
    for p in &mut g.players {
        p.life = life;
        p.turns = 1;
    }
    g.round = 1;
    g.active = Some(PlayerId(0));
    g.step = Step::Main1;
    g
}

/// The card out of p's library (its definition if the deck doesn't run it): Python's `take`.
pub fn take(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = g.db.id(name).unwrap_or_else(|| panic!("{name:?} isn't in data/cards.json"));
    let lib = &mut g.player_mut(p).library;
    if let Some(i) = lib.iter().position(|&x| x == c) {
        lib.remove(i);
    }
    c
}

pub fn hand(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).hand.extend(&cs);
    cs
}

/// n copies of a land on p's battlefield (no landfall, as in the Python)
pub fn lands(g: &mut Game, p: PlayerId, name: &str, n: usize, tapped: bool) -> Vec<LandId> {
    (0..n)
        .map(|_| {
            let c = take(g, p, name);
            g.add_land(p, c, tapped)
        })
        .collect()
}
