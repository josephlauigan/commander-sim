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
    /// an action on the human's own turn (tap, land, cast, use, pass), as the page sends it
    Action(serde_json::Value),
}

pub trait Human: Send + Sync {
    /// Wait for the person's answer to `req`. `g` is the game as it stands (for building their view).
    fn ask(&self, g: &Game, seat: PlayerId, req: Request) -> Answer;
    /// Tell the person something that needs no answer ('info', 'warn' ...).
    fn tell(&self, seat: PlayerId, kind: Sym, text: &str);
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
