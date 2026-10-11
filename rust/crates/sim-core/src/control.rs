//! Who decides for a seat: the AI, or (practice mode) a person.
//!
//! The AI's decisions are made inline by the engine and AI code, as in Python. A seat played by a person has a
//! `Human`, and at each of the person's decisions the engine asks it (Python's `human_choice(g, p)`, which returns
//! the choices module, and the controller's `ask`). The practice server's `Human` turns a `Request` into a message
//! to the browser and blocks the engine thread until the answer comes back.
//!
//! A copied game never has people in it: the look-ahead plays every seat with the AI (Python's `search.clone` drops
//! `g.controllers`). Here that is automatic, because cloning `Humans` gives no humans.

use crate::ids::PlayerId;
use crate::state::{Game, MAX_SEATS};
use crate::sym::Sym;
use std::sync::Arc;

/// One decision a person must make (Python's `controller.Request`): what kind ('mulligan', 'target', 'choose' ...),
/// the prompt, the legal choices, and anything else the page needs to show it.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub kind: Sym,
    pub prompt: String,
    pub choices: Vec<String>,
    pub data: serde_json::Value,
}

/// A person's answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// the index of a choice
    Pick(usize),
    /// several choices (cards to discard, modes)
    Picks(Vec<usize>),
    Yes,
    No,
    Cancel,
    /// the page's 'cancel' answer (a way out of a choice that has one); not the same as `Cancel`
    Decline,
    /// an action on the human's own turn (tap, land, cast, use, pass), as the page sends it
    Action(serde_json::Value),
}

pub trait Human: Send + Sync {
    /// Wait for the person's answer to `req`. `g` is the game as it stands (for building their view).
    fn ask(&self, g: &Game, seat: PlayerId, req: Request) -> Answer;
    /// Tell the person something that needs no answer ('info', 'warn' ...).
    fn tell(&self, seat: PlayerId, kind: Sym, text: &str);
    /// Is a person watching (the browser, the terminal)? Python's `ctl.notify is not None`: only then do choices
    /// carry `refs` (where each choice sits on the table) and the `view`. A `ScriptHuman` is not live.
    fn live(&self) -> bool {
        false
    }
    /// The seat's auto-pass setting: 'respond' (default), 'stack' or 'all' (Python's `ctl.autopass`).
    fn autopass(&self) -> Option<&'static str> {
        None
    }
}

/// Where the game's log lines go as they happen (practice mode's event stream). `secret` is set for a line only one
/// seat may read (Python's `EventLog.secret`): the sink is handed `public` as `line` and the full line in `secret`.
pub trait LogSink: Send + Sync {
    fn line(&self, g: &Game, line: &str, secret: Option<(PlayerId, &str)>);
}

/// A running practice game's links to its session: the log sink and the abort flag (set when the session is closed
/// or rewound: the next engine step stops the game with `Stop::Cancelled`). Cloning gives no links, like `Humans`:
/// a look-ahead copy neither reports nor aborts.
#[derive(Default)]
pub struct Link {
    pub sink: Option<Arc<dyn LogSink>>,
    pub abort: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl Link {
    pub fn aborted(&self) -> bool {
        self.abort.as_ref().is_some_and(|a| a.load(std::sync::atomic::Ordering::Relaxed))
    }
}

impl Clone for Link {
    fn clone(&self) -> Link {
        Link::default()
    }
}

impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Link")
    }
}

impl Game {
    /// A log line naming a card only seat `p` may see (a card tutored to hand): the game's log keeps `line` whole;
    /// `p` is shown `line`, the others `public` (Python's `g.log.secret(key, public, line)`).
    pub fn log_secret(&mut self, p: PlayerId, public: &str, line: &str) {
        let full = format!("R{:<2} {}", self.round, line);
        if let Some(log) = self.log.as_mut() {
            log.push(full.clone());
        }
        if let Some(s) = self.link.sink.clone() {
            let public = format!("R{:<2} {}", self.round, public);
            s.line(self, &public, Some((p, &full)));
        }
    }
}

type Seats = [Option<Arc<dyn Human>>; MAX_SEATS];

/// The people at a table, by seat. Cloning gives an empty table (see the module comment).
#[derive(Default)]
pub struct Humans {
    seats: Option<Arc<Seats>>,
}

impl Humans {
    pub fn none() -> Humans {
        Humans { seats: None }
    }

    pub fn seat(p: PlayerId, h: Arc<dyn Human>) -> Humans {
        let mut v: Seats = Default::default();
        v[p.index()] = Some(h);
        Humans { seats: Some(Arc::new(v)) }
    }

    pub fn add(&mut self, p: PlayerId, h: Arc<dyn Human>) {
        let mut v: Seats = match &self.seats {
            Some(s) => (**s).clone(),
            None => Default::default(),
        };
        v[p.index()] = Some(h);
        self.seats = Some(Arc::new(v));
    }

    /// The person playing seat p, if any (Python's `human_choice(g, p) is not None`).
    pub fn get(&self, p: PlayerId) -> Option<&Arc<dyn Human>> {
        self.seats.as_ref()?.get(p.index())?.as_ref()
    }

    pub fn any(&self) -> bool {
        self.seats.as_ref().is_some_and(|s| s.iter().any(Option::is_some))
    }
}

impl Clone for Humans {
    fn clone(&self) -> Humans {
        Humans::none()
    }
}

impl std::fmt::Debug for Humans {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seats: Vec<usize> =
            self.seats.iter().flat_map(|s| s.iter().enumerate().filter(|x| x.1.is_some()).map(|x| x.0)).collect();
        write!(f, "Humans{seats:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Always;
    impl Human for Always {
        fn ask(&self, _: &Game, _: PlayerId, _: Request) -> Answer {
            Answer::Yes
        }
        fn tell(&self, _: PlayerId, _: Sym, _: &str) {}
    }

    #[test]
    fn a_copy_of_the_table_has_no_people() {
        let mut h = Humans::seat(PlayerId(1), Arc::new(Always));
        h.add(PlayerId(3), Arc::new(Always));
        assert!(h.get(PlayerId(1)).is_some() && h.get(PlayerId(3)).is_some() && h.get(PlayerId(0)).is_none());
        assert!(h.any());
        let c = h.clone();
        assert!(!c.any() && c.get(PlayerId(1)).is_none());
    }
}
