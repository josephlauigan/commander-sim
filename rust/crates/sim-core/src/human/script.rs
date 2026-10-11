//! `ScriptHuman`: answers requests from a list or a function instead of a person (Python's `ScriptController`),
//! for tests and the text client's scripted input.

use crate::control::{Answer, Human, Request};
use crate::ids::PlayerId;
use crate::state::Game;
use crate::sym::Sym;
use std::collections::VecDeque;
use std::sync::Mutex;

/// A runaway script ends the game instead of hanging a test (Python's `ScriptController.LIMIT`).
pub const LIMIT: usize = 20000;

type Func = Box<dyn FnMut(&Request) -> Answer + Send>;

enum Source {
    /// answers in order; an empty list answers `Cancel`
    List(VecDeque<Answer>),
    /// each answer may depend on the request
    Funcs(VecDeque<Func>),
    /// one function for every request
    Func(Func),
}

pub struct ScriptHuman {
    src: Mutex<Source>,
    asked: Mutex<Vec<Request>>,
    told: Mutex<Vec<(Sym, String)>>,
    autopass: Mutex<Option<&'static str>>,
}

impl ScriptHuman {
    /// answers from a list, in order
    pub fn new(answers: Vec<Answer>) -> ScriptHuman {
        ScriptHuman::with(Source::List(answers.into()))
    }

    /// answers from a list of functions of the request (Python lists may hold callables)
    pub fn from_funcs(answers: Vec<Func>) -> ScriptHuman {
        ScriptHuman::with(Source::Funcs(answers.into()))
    }

    /// every answer is a function of the request
    pub fn from_fn(f: impl FnMut(&Request) -> Answer + Send + 'static) -> ScriptHuman {
        ScriptHuman::with(Source::Func(Box::new(f)))
    }

    fn with(src: Source) -> ScriptHuman {
        ScriptHuman {
            src: Mutex::new(src),
            asked: Mutex::new(vec![]),
            told: Mutex::new(vec![]),
            autopass: Mutex::new(None),
        }
    }

    /// set the auto-pass mode ('respond', 'stack', 'all')
    pub fn set_autopass(&self, mode: &'static str) {
        *self.autopass.lock().unwrap() = Some(mode);
    }

    /// every request asked so far
    pub fn asked(&self) -> Vec<Request> {
        self.asked.lock().unwrap().clone()
    }

    /// the requests of one kind
    pub fn asked_kind(&self, kind: &str) -> Vec<Request> {
        self.asked.lock().unwrap().iter().filter(|r| r.kind == kind).cloned().collect()
    }

    /// everything told so far: (kind, text)
    pub fn told(&self) -> Vec<(Sym, String)> {
        self.told.lock().unwrap().clone()
    }

    /// the texts told with one kind ('invalid', 'info' ...)
    pub fn told_kind(&self, kind: &str) -> Vec<String> {
        self.told.lock().unwrap().iter().filter(|t| t.0 == kind).map(|t| t.1.clone()).collect()
    }
}

impl Human for ScriptHuman {
    fn ask(&self, _g: &Game, _seat: PlayerId, req: Request) -> Answer {
        let n = {
            let mut a = self.asked.lock().unwrap();
            a.push(req.clone());
            a.len()
        };
        if n > LIMIT {
            return Answer::Cancel;
        }
        match &mut *self.src.lock().unwrap() {
            Source::Func(f) => f(&req),
            Source::List(l) => l.pop_front().unwrap_or(Answer::Cancel),
            Source::Funcs(l) => match l.pop_front() {
                Some(mut f) => f(&req),
                None => Answer::Cancel,
            },
        }
    }

    fn tell(&self, _seat: PlayerId, kind: Sym, text: &str) {
        self.told.lock().unwrap().push((kind, text.to_string()));
    }

    fn autopass(&self) -> Option<&'static str> {
        *self.autopass.lock().unwrap()
    }
}
