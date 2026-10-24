//! Playing the games: the card data and deck lists loaded once, seats drawn as pools.draw_seats draws them, games
//! spread over worker threads, and each finished game boiled down to what the reports read (`GameOut`).

use crate::decks::{self, PoolDeck};
use crate::progress::Progress;
use crate::pyrand::draw_seats;
use sim_core::engine::turn::Seat;
use sim_core::export::RawCard;
use sim_core::run::Engine;
use sim_core::settings::{AiMode, Profile, Settings};
use sim_core::state::Game;
use sim_core::sym::Sym;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, mpsc};

/// How one player's game went.
#[derive(Debug, Clone)]
pub struct PlayerOut {
    pub key: Sym,
    pub turns: u32,
    pub alive: bool,
    /// who eliminated this player (an index into GameOut::players)
    pub killer: Option<usize>,
    pub stats: HashMap<Sym, i64>,
    pub milestone: HashMap<Sym, u32>,
    pub first_bomb: Option<u32>,
    pub death_kind: Sym,
    /// cards seen (drawn, tutored, the opening hand) and cast, by name; permanents removed by opponents
    pub seen: Vec<String>,
    pub cast: Vec<String>,
    pub lost: Vec<(String, u32)>,
    /// the own-turn count at which each cast card was first cast (with the card lists)
    pub cast_turns: Vec<(String, u32)>,
}

impl PlayerOut {
    /// p.stats[k] with Python's Counter default
    pub fn stat(&self, k: &str) -> i64 {
        self.stats.get(k).copied().unwrap_or(0)
    }
}

/// How a game went (the fields poolmode.py and compare.py read from a finished game).
#[derive(Debug, Clone)]
pub struct GameOut {
    pub winner: Option<usize>,
    pub wintype: Option<Sym>,
    pub round: u32,
    pub players: Vec<PlayerOut>,
    pub log: Option<Vec<String>>,
}

impl GameOut {
    pub fn from_game(g: &Game, with_cards: bool) -> GameOut {
        let names = |ids: &mut dyn Iterator<Item = sim_core::ids::CardId>| -> Vec<String> {
            ids.map(|c| g.db.get(c).name.to_string()).collect()
        };
        GameOut {
            winner: g.winner.map(|p| p.0 as usize),
            wintype: g.wintype,
            round: g.round,
            players: g
                .players
                .iter()
                .map(|p| PlayerOut {
                    key: p.key,
                    turns: p.turns,
                    alive: p.alive,
                    killer: p.killer.map(|k| k.0 as usize),
                    stats: p.stats.iter().map(|(k, v)| (*k, *v)).collect(),
                    milestone: p.milestone.iter().map(|(k, v)| (*k, *v)).collect(),
                    first_bomb: p.first_bomb,
                    death_kind: p.death_kind,
                    seen: if with_cards { names(&mut p.seen.iter().copied()) } else { vec![] },
                    cast: if with_cards { names(&mut p.cast_names.iter().copied()) } else { vec![] },
                    lost: if with_cards {
                        p.lost_names.iter().map(|(k, v)| (k.to_string(), *v)).collect()
                    } else {
                        vec![]
                    },
                    cast_turns: if with_cards {
                        let t = p.turns;
                        let rest = p.cast_names.iter().skip(p.cast_logged).map(|c| (*c, t));
                        p.cast_turns.iter().copied().chain(rest).map(|(c, t)| (g.db.get(c).name.to_string(), t)).collect()
                    } else {
                        vec![]
                    },
                })
                .collect(),
            log: g.log.clone(),
        }
    }

    pub fn player(&self, key: &str) -> Option<usize> {
        self.players.iter().position(|p| p.key == key)
    }

    /// g.wintype or 'damage'
    pub fn wintype_or_damage(&self) -> &str {
        match self.wintype {
            Some(w) if !w.is_empty() => w,
            _ => "damage",
        }
    }
}

/// The AI and the run-wide options (compare.py's globals AI, TEMP, JOBS).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunOpts {
    pub ai: AiMode,
    pub temp: f64,
    pub jobs: usize,
}

pub fn ai_name(ai: AiMode) -> &'static str {
    match ai {
        AiMode::Lookahead => "lookahead",
        AiMode::Adaptive => "adaptive",
    }
}

pub fn profile(name: &str) -> Result<Profile, String> {
    match name {
        "loose" => Ok(Profile::Loose),
        "conservative" => Ok(Profile::Conservative),
        _ => Err(format!("no profile {name:?} (conservative or loose)")),
    }
}

/// A game that couldn't be played: its seed and what went wrong.
#[derive(Debug, Clone)]
pub struct GameError {
    pub seed: u64,
    pub what: String,
}

/// Everything a run needs: the engine, the deck lists, the options, the progress bar and the errors so far.
pub struct Sim {
    pub engine: Engine,
    pub mine: BTreeMap<String, Vec<String>>,
    pub pool: Vec<PoolDeck>,
    pub opts: RunOpts,
    pub prog: Mutex<Progress>,
    pub errors: Mutex<Vec<GameError>>,
    /// --knobs FILE: applied to every run's settings (`settings`)
    pub knobs: Option<crate::knobs::Knobs>,
    seats: HashMap<String, Seat>,
    /// the exported card records (data/cards.json), by name
    raw: HashMap<String, RawCard>,
}

impl Sim {
    pub fn load(opts: RunOpts) -> Result<Sim, String> {
        let root = decks::root()?;
        let data = decks::data_dir(&root);
        let engine = Engine::load(&data).map_err(|e| format!("loading {}: {e}", data.display()))?;
        let mine = decks::load_mine(&root)?;
        let pool = decks::load_pool(&root)?;
        let raw = sim_core::export::load_cards(&data.join("cards.json")).map_err(|e| format!("data/cards.json: {e}"))?;
        let raw = raw.into_iter().map(|c| (c.name.clone(), c)).collect();
        let mut sim = Sim {
            engine,
            mine,
            pool,
            opts,
            prog: Mutex::new(Progress::new()),
            errors: Mutex::new(vec![]),
            knobs: None,
            seats: HashMap::new(),
            raw,
        };
        let mut seats = HashMap::new();
        for (k, cards) in &sim.mine {
            seats.insert(k.clone(), sim.seat(k, cards)?);
        }
        for d in &sim.pool {
            seats.insert(d.key.clone(), sim.seat(&d.key, &d.cards)?);
        }
        sim.seats = seats;
        sim.warn_stale_export();
        Ok(sim)
    }

    /// the commander of a deck: yours from data/decks.json (ais.CMDS), a pool deck's from its file
    pub fn commander(&self, key: &str) -> Option<String> {
        if self.mine.contains_key(key) {
            return self.engine.decks.get(key).map(|d| d.commander.clone());
        }
        self.pool.iter().find(|d| d.key == key).map(|d| d.commander.clone())
    }

    /// a seat for a deck key with these cards (its list, or a variant)
    pub fn seat(&self, key: &str, cards: &[String]) -> Result<Seat, String> {
        let cmd = self.commander(key).ok_or_else(|| format!("no deck {key:?} (not in data/decks.json?)"))?;
        self.engine.seat(key, cards, &cmd)
    }

    /// a card's exported record
    pub fn raw_card(&self, name: &str) -> Option<&RawCard> {
        self.raw.get(name)
    }

    /// a double-faced card typed by its front face alone ("Kuja, Genome Sorcerer"): the full "A // B" name it is stored under
    pub fn full_face_name(&self, name: &str) -> Option<String> {
        let pre = format!("{name} // ");
        self.raw.keys().find(|k| k.starts_with(&pre)).map(|k| k.to_string())
    }

    /// poolmode.pool_keys: one tier's deck keys, in file order
    pub fn pool_keys(&self, tier: &str) -> Vec<String> {
        self.pool.iter().filter(|d| d.tier == tier).map(|d| d.key.clone()).collect()
    }

    /// poolmode.name_of: 'Sauron' for your decks, the commander's short name for a pool deck
    pub fn name_of(&self, key: &str) -> String {
        if let Some(d) = self.engine.decks.get(key)
            && self.mine.contains_key(key)
        {
            return d.name.clone();
        }
        // a pool deck: the exported name (decks.json), "Rograkh/Silas" for a partner pair
        match (self.engine.decks.get(key), self.pool.iter().find(|d| d.key == key)) {
            (Some(d), Some(_)) => d.name.clone(),
            (None, Some(d)) => d.short_name(),
            _ => key.to_string(),
        }
    }

    pub fn settings(&self, profile: Profile) -> Settings {
        let mut st = Settings::new(profile, self.opts.ai, self.opts.temp);
        if let Some(k) = &self.knobs {
            k.apply(&mut st);
        }
        st
    }

    /// every deck key a knobs file may name: your decks and the pool's
    pub fn deck_keys(&self) -> Vec<&str> {
        self.mine.keys().map(|k| k.as_str()).chain(self.pool.iter().map(|d| d.key.as_str())).collect()
    }

    /// the cards of a deck file that differ from data/decks.json: the export is out of date (a warning only; the
    /// games use the deck files, and every card has to be in data/cards.json)
    fn warn_stale_export(&self) {
        let lists = self.mine.iter().map(|(k, v)| (k.as_str(), v)).chain(self.pool.iter().map(|d| (d.key.as_str(), &d.cards)));
        let stale: Vec<&str> = lists
            .filter(|(k, v)| {
                let mut a: Vec<&String> = v.iter().collect();
                a.sort();
                self.engine.decks.get(*k).is_none_or(|d| {
                    let mut b: Vec<&String> = d.cards.iter().collect();
                    b.sort();
                    a != b
                })
            })
            .map(|(k, _)| k)
            .collect();
        if !stale.is_empty() {
            eprintln!(
                "note: data/decks.json differs from the deck files for {} (the games use the deck files; re-export \
                 the cards after changing a list)",
                stale.join(", ")
            );
        }
    }

    /// poolmode.play: the seats for this seed (me and k opponents drawn from opp_keys, or k pool decks when me is
    /// None), my seat with my_seat's cards
    pub fn seats_for(&self, seed: u64, me: Option<&str>, my_seat: Option<&Seat>, opp_keys: &[String], k: usize) -> Vec<Seat> {
        draw_seats(seed, opp_keys, me, k)
            .iter()
            .map(|s| match (me, my_seat) {
                (Some(m), Some(seat)) if m == s => seat.clone(),
                _ => self.seats[s].clone(),
            })
            .collect()
    }

    pub fn play(&self, seed: u64, seats: &[Seat], settings: &Settings, trace: bool) -> Game {
        self.engine.play(seed, seats, settings.clone(), 20, trace)
    }

    /// Play f(seed) for every seed on the worker threads, the progress bar moving as games finish. Results come back
    /// in seed order; a game that panicked is left out and recorded (compare.report_errors lists them at the end).
    pub fn run_seeds<T: Send>(&self, seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<(u64, T)> {
        let jobs = self.opts.jobs.max(1).min(seeds.len().max(1));
        let next = AtomicUsize::new(0);
        let mut out: Vec<Option<Result<T, String>>> = (0..seeds.len()).map(|_| None).collect();
        let (tx, rx) = mpsc::channel::<(usize, Result<T, String>)>();
        std::thread::scope(|sc| {
            for _ in 0..jobs {
                let tx = tx.clone();
                let (next, f) = (&next, &f);
                std::thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(sc, move || {
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            if i >= seeds.len() {
                                break;
                            }
                            let r = catch_game(|| f(seeds[i]));
                            if tx.send((i, r)).is_err() {
                                break;
                            }
                        }
                    })
                    .expect("spawn a worker thread");
            }
            drop(tx);
            for (i, r) in rx {
                out[i] = Some(r);
                self.prog.lock().unwrap().update(1);
            }
        });
        let mut res = Vec::with_capacity(seeds.len());
        for (i, r) in out.into_iter().enumerate() {
            match r {
                Some(Ok(t)) => res.push((seeds[i], t)),
                Some(Err(what)) => self.errors.lock().unwrap().push(GameError { seed: seeds[i], what }),
                None => self.errors.lock().unwrap().push(GameError { seed: seeds[i], what: "not played".into() }),
            }
        }
        res
    }
}

thread_local! {
    static IN_GAME: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The panic hook: quiet inside a game (the error is reported at the end of the run), the usual message elsewhere.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !IN_GAME.get() {
            default(info);
        }
    }));
}

/// run one game, turning a panic into an error message ("panic: ..." with where it happened)
pub fn catch_game<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    IN_GAME.set(true);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    IN_GAME.set(false);
    r.map_err(|e| {
        let msg = e
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| e.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        format!("panic: {msg}")
    })
}
