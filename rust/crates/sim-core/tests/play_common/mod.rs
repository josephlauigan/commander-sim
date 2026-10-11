//! Shared helpers for the play_*.rs tests: a ScriptHuman at a seat (Python's `ScriptController` lists, where an entry
//! may be a function of the request), and a few position builders.
#![allow(dead_code)]

pub use serde_json::{Value, json};
pub use sim_core::control::{Answer, Request};
pub use sim_core::human::script::ScriptHuman;
pub use sim_core::ids::{PermId, PlayerId};
pub use sim_core::state::Game;
pub use sim_core::testkit::*;
use std::sync::Arc;

pub const P0: PlayerId = PlayerId(0);
pub const P1: PlayerId = PlayerId(1);
pub const P2: PlayerId = PlayerId(2);

pub type F = Box<dyn FnMut(&Request) -> Answer + Send>;

/// a fixed answer
pub fn ans(a: Answer) -> F {
    Box::new(move |_| a.clone())
}
/// `{'do': ...}` action
pub fn act(v: Value) -> F {
    ans(Answer::Action(v))
}
pub fn pass() -> F {
    act(json!({"do": "pass"}))
}
pub fn pick(i: usize) -> F {
    ans(Answer::Pick(i))
}
pub fn picks(v: &[usize]) -> F {
    ans(Answer::Picks(v.to_vec()))
}
/// the page's 'cancel'
pub fn cancel() -> F {
    ans(Answer::Decline)
}
pub fn yes() -> F {
    ans(Answer::Yes)
}
pub fn no() -> F {
    ans(Answer::No)
}
/// pick the first choice whose text contains `word`
pub fn by_text(word: &'static str) -> F {
    Box::new(move |r| {
        let i = r.choices.iter().position(|x| x.contains(word)).unwrap_or_else(|| panic!("no choice with {word:?} in {:?}", r.choices));
        Answer::Pick(i)
    })
}

/// seat a ScriptHuman with these answers at p
pub fn seat(g: &mut Game, p: PlayerId, answers: Vec<F>) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_funcs(answers));
    g.humans.add(p, h.clone());
    h
}

/// add mana to p's pool
pub fn pool(g: &mut Game, p: PlayerId, mana: &[(char, u32)]) {
    for &(c, n) in mana {
        g.player_mut(p).pool.add(c, n);
    }
}

pub fn pool_text(g: &Game, p: PlayerId) -> String {
    g.player(p).pool.text()
}

pub fn pool_total(g: &Game, p: PlayerId) -> u32 {
    g.player(p).pool.total()
}

/// the main phase with these answers (and a final pass); returns the script
pub fn main(g: &mut Game, p: PlayerId, mut answers: Vec<F>) -> Arc<ScriptHuman> {
    answers.push(pass());
    let h = seat(g, p, answers);
    sim_core::human::human::human_main(g, p, false).unwrap();
    h
}

pub fn perms_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

pub fn has_perm(g: &Game, p: PlayerId, name: &str) -> bool {
    !perms_named(g, p, name).is_empty()
}

pub fn told_last(h: &ScriptHuman) -> String {
    h.told().last().map(|t| t.1.clone()).unwrap_or_default()
}
