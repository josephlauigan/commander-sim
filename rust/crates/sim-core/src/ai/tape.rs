//! Practice mode's search tape (Python's `search.Tape` / `_taped`): the look-ahead decisions of one real game, in
//! order. Undo replays a game from its seed with the same answers; the AI's decisions then come off the tape instead
//! of being searched again (the Rust look-ahead is deterministic from the seed, so this is only for speed).

use crate::ids::PlayerId;
use crate::state::Game;
use crate::sym::Sym;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// A recorded look-ahead decision's result.
#[derive(Debug, Clone, PartialEq)]
pub enum Taped {
    /// the chosen option as (label, n-th of that label), or None (the heuristic chooses)
    Main(Option<(String, usize)>),
    Attack(Option<(PlayerId, Sym)>),
    Counter(bool),
}

impl Taped {
    pub fn kind(&self) -> &'static str {
        match self {
            Taped::Main(_) => "main",
            Taped::Attack(_) => "attack",
            Taped::Counter(_) => "counter",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TapeEntry {
    pub result: Taped,
    /// search work booked to the game by this decision
    pub work: u64,
}

#[derive(Debug, Default)]
pub struct Tape {
    pub entries: Vec<TapeEntry>,
    pub pos: usize,
}

impl Tape {
    pub fn replaying(&self) -> bool {
        self.pos < self.entries.len()
    }
    /// forget everything from entry n on (the decisions after an undone answer); the next run replays the rest
    pub fn cut(&mut self, n: usize) {
        self.entries.truncate(n);
        self.pos = 0;
    }
}

/// `Game.search_tape`: the game's tape, if it is a real practice game. A copy has none.
#[derive(Default)]
pub struct TapeSlot(pub Option<Arc<Mutex<Tape>>>);

impl Clone for TapeSlot {
    fn clone(&self) -> TapeSlot {
        TapeSlot(None)
    }
}

impl std::fmt::Debug for TapeSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TapeSlot")
    }
}

/// search._taped: run `compute` for real game g, or take its result off g's tape. Returns (result, replayed).
pub fn taped(g: &mut Game, kind: &str, compute: impl FnOnce(&mut Game) -> Taped) -> (Taped, bool) {
    let tape = g.search_tape.0.clone().expect("taped without a tape");
    {
        let mut t = tape.lock().unwrap();
        if t.replaying() {
            let e = t.entries[t.pos].clone();
            t.pos += 1;
            if e.result.kind() != kind {
                panic!("search tape out of step: {} recorded, {kind} asked", e.result.kind());
            }
            g.search_work += e.work;
            return (e.result, true);
        }
    }
    let before = g.search_work;
    let result = compute(g);
    let mut t = tape.lock().unwrap();
    t.entries.push(TapeEntry { result: result.clone(), work: g.search_work - before });
    t.pos = t.entries.len();
    (result, false)
}

thread_local! {
    /// search.LAST_SCORES: the last main-phase decision's (label, score) per candidate (the Hint shows them)
    pub static LAST_SCORES: RefCell<Vec<(String, f64)>> = const { RefCell::new(Vec::new()) };
}
