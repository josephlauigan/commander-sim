//! What the server needs from a game session (practice-session's `SessionApi` in `session/api.rs` is the real one;
//! this is the server's own copy of the shape it codes against, with a fake in the tests). The server follows
//! Python's `Hub`: it keeps the numbered event history, the pending requests and the latest views itself, fed from
//! the session's raw events (JSON shaped like Python's `Session.events`):
//!
//! * `{"kind": "request", "seat": deck, "request": {"kind","prompt","choices","data"}}` (a `view` inside `data` is
//!   the seat's table, moved out by the hub); any other event is `{"kind": ..., ...}` with an optional `view` (for
//!   the session's first human), `views` ({seat: view}), `seat` (only that seat sees it), `hide` (that seat does not).
//! * `kind: "reset"` (an Undo restarts the game: the history is cleared), `"over"` with `how`, `"log"`, `"loading"`...
//!
//! Error texts are Python's (the page shows them).

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// A game's settings, as the state route reports them.
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub deck: String,
    pub tier: String,
    pub seed: u64,
    pub seats: Vec<String>,
    pub profile: String,
    pub ai: String,
    /// the decks people play (one, or two with a partner)
    pub humans: Vec<String>,
    /// deck key -> the name shown for its seat
    pub names: Vec<(String, String)>,
    pub partner: Option<String>,
    /// the game waits on a decision now (a one-player game can be saved)
    pub waiting: bool,
    pub finished: bool,
}

impl Info {
    pub fn name_of(&self, seat: &str) -> String {
        self.names.iter().find(|(k, _)| k == seat).map(|(_, n)| n.clone()).unwrap_or_else(|| seat.to_string())
    }
}

pub trait SessionApi: Send + Sync {
    fn info(&self) -> Info;
    /// the seat's auto-pass mode once all answers are given (`autopass_of(seat, at=10**9)`)
    fn autopass_of(&self, seat: &str) -> String;
    /// raw events after `cursor`, waiting up to `timeout` for one; returns the new cursor and the events
    fn events_after(&self, cursor: u64, timeout: Duration) -> (u64, Vec<Value>);
    /// start the game (after the images are ready)
    fn start(&self);
    /// give the waiting decision of `seat` its answer
    fn answer(&self, seat: &str, value: Value);
    /// `None`, or why not
    fn undo(&self, n: i64, seat: &str) -> Option<String>;
    fn try_it(&self, n: &Value, seat: &str) -> Option<String>;
    /// where Undo of seat's last n answers rewinds to, or None
    fn undo_point(&self, n: i64, seat: &str) -> Option<usize>;
    /// whose answer each answer so far is
    fn answer_seats(&self) -> Vec<String>;
    /// the decision `n` has an AI choice to try
    fn has_try_entry(&self, n: &Value, seat: &str) -> bool;
    fn set_autopass(&self, seat: &str, mode: &Value) -> Option<String>;
    fn hint(&self, seat: &str) -> Result<Value, String>;
    fn review(&self, seat: &str) -> Result<Value, String>;
    /// the game as a save file's data; `wait`: give up if a hint holds the game that long (autosave)
    fn saved(&self, wait: Option<Duration>) -> Result<Value, String>;
    /// end the game, and wait up to `join` for its thread to finish
    fn close(&self, join: Duration);
}

/// Makes sessions; practice-session provides the real one.
pub trait SessionFactory: Send + Sync {
    /// a new game from the request's options (deck, tier, seed?, seat?, opponents?, profile?, ai?, tools?, seats?,
    /// partner?, autopass?); `Err` is the message of Python's ValueError
    fn new_session(&self, opts: &Value) -> Result<Arc<dyn SessionApi>, String>;
    /// only check the settings of a two-player game (the second person's deck is `partner`)
    fn check_two(&self, opts: &Value, partner: &str) -> Result<(), String>;
    /// a saved game, replayed to where it was saved
    fn load_session(&self, data: &Value, compare: bool) -> Result<Arc<dyn SessionApi>, String>;
    /// the card names of the decks in a game (for the images)
    fn card_names(&self, info: &Info) -> Vec<String>;
}

/// the answer to auto-pass: one of these modes
pub const AUTOPASS: [&str; 3] = ["respond", "stack", "all"];

/// the factory used until the session is wired: every game fails with a message
pub struct NoSessions;

impl SessionFactory for NoSessions {
    fn new_session(&self, _: &Value) -> Result<Arc<dyn SessionApi>, String> {
        Err("games are not available in this build yet".into())
    }
    fn check_two(&self, _: &Value, _: &str) -> Result<(), String> {
        Err("games are not available in this build yet".into())
    }
    fn load_session(&self, _: &Value, _: bool) -> Result<Arc<dyn SessionApi>, String> {
        Err("games are not available in this build yet".into())
    }
    fn card_names(&self, _: &Info) -> Vec<String> {
        vec![]
    }
}
