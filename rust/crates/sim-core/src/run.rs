//! Playing pool games for the Python run harness (`python3 -m commander_sim ... --engine rust`): the card data and
//! card code loaded once per process, a game played from explicit seats (Python's `ais.play_pool_game`), and a
//! summary of how it went for poolmode's tallies.

use crate::cards::{CardDb, Colors};
use crate::engine::turn::{Seat, run_rounds, setup_game};
use crate::export::{RawDeck, load_decks};
use crate::hooks::Registry;
use crate::ids::CardId;
use crate::settings::{AiMode, Profile, Settings};
use crate::state::Game;
use crate::sym::intern;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// The card database, the card code and the deck lists (data/cards.json, data/decks.json).
pub struct Engine {
    pub db: Arc<CardDb>,
    pub registry: Arc<Registry>,
    pub decks: HashMap<String, RawDeck>,
}

impl Engine {
    pub fn load(data: &Path) -> Result<Engine, String> {
        let db = Arc::new(CardDb::load(&data.join("cards.json")).map_err(|e| e.to_string())?);
        let registry = Arc::new(crate::impls::registry(&db)?);
        let f = load_decks(&data.join("decks.json")).map_err(|e| e.to_string())?;
        let decks = f.mine.into_iter().chain(f.pool).map(|d| (d.key.clone(), d)).collect();
        Ok(Engine { db, registry, decks })
    }

    /// a seat from a deck key, its cards (a deck list, possibly with swaps) and its commander
    pub fn seat(&self, key: &str, cards: &[String], commander: &str) -> Result<Seat, String> {
        let card =
            |n: &str| self.db.id(n).ok_or_else(|| format!("{n:?} isn't in data/cards.json (re-export the cards)"));
        let commander = card(commander)?;
        let ident = match self.decks.get(key).and_then(|d| d.ident.as_deref()) {
            Some(s) => Colors::from_letters(s),
            None => self.db.get(commander).identity.unwrap_or_default(),
        };
        let name = self.decks.get(key).map_or(key, |d| d.name.as_str());
        let cards: Arc<[CardId]> = cards.iter().map(|n| card(n)).collect::<Result<_, _>>()?;
        Ok(Seat { key: intern(key), name: intern(name), commander, ident, cards })
    }

    /// ais.play_pool_game: a game with this seat order, from seed: setup, mulligans and up to max_rounds rounds
    pub fn play(&self, seed: u64, seats: &[Seat], settings: Settings, max_rounds: u32, trace: bool) -> Game {
        let mut g = setup_game(self.db.clone(), self.registry.clone(), Arc::new(settings), seed, seats, trace);
        run_rounds(&mut g, max_rounds);
        g
    }
}

/// the run settings from the Python harness's names ('loose', 'lookahead', the --temp scale)
pub fn settings(profile: &str, ai: &str, temp: f64) -> Result<Settings, String> {
    let profile = match profile {
        "loose" => Profile::Loose,
        "conservative" => Profile::Conservative,
        _ => return Err(format!("no profile {profile:?}")),
    };
    let ai = match ai {
        "lookahead" => AiMode::Lookahead,
        "adaptive" => AiMode::Adaptive,
        _ => return Err(format!("the Rust engine has no {ai:?} AI")),
    };
    Ok(Settings::new(profile, ai, temp))
}

/// How a game went, for poolmode's tallies.
#[derive(Debug, Serialize)]
pub struct Summary {
    pub winner: Option<String>,
    pub wintype: Option<String>,
    pub round: u32,
    /// the game hit its step budget (a runaway loop) and ended as a timeout
    pub stopped: bool,
    pub players: Vec<PlayerSummary>,
    /// look-ahead decisions made, and the engine steps their playouts took
    pub search_n: u32,
    pub search_work: u64,
    pub log: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct PlayerSummary {
    pub key: String,
    pub turns: u32,
    pub alive: bool,
    pub life: i32,
    /// who eliminated this player
    pub killer: Option<String>,
    pub stats: HashMap<String, i64>,
    pub milestone: HashMap<String, u32>,
}

pub fn summary(g: &Game) -> Summary {
    let key = |p: crate::ids::PlayerId| g.player(p).key.to_string();
    Summary {
        winner: g.winner.map(key),
        wintype: g.wintype.map(|w| w.to_string()),
        round: g.round,
        stopped: g.stopped,
        players: g
            .players
            .iter()
            .map(|p| PlayerSummary {
                key: p.key.to_string(),
                turns: p.turns,
                alive: p.alive,
                life: p.life,
                killer: p.killer.map(key),
                stats: p.stats.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                milestone: p.milestone.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            })
            .collect(),
        search_n: g.search_n,
        search_work: g.search_work,
        log: g.log.clone(),
    }
}
