//! Test support for practice-mode person paths: does a person who answers with the AI's own picks get the AI's
//! outcome? (Not `testkit.rs`, which another change owns.)
//!
//! `person_matches_ai` runs one scenario twice from identical games: once with the AI deciding at the seat, and once
//! with a scripted person at the seat whose answers a small per-card adapter reads off the AI's finished game. It then
//! compares a state fingerprint (zones, life, permanents with tapped and counters, the stack, the mana pools) and the
//! public action lines (what the table could read; not the person's prompts, not the secret half of a `log_secret`
//! line, not the AI's `[X decides: ...]` lines).
#![allow(dead_code)]

use sim_core::control::{Answer, LogSink, Request};
use sim_core::human::script::ScriptHuman;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use std::sync::{Arc, Mutex};

/// the public text of every log line, as the table would read it
#[derive(Default)]
pub struct PublicLines(Mutex<Vec<String>>);

impl LogSink for PublicLines {
    fn line(&self, _g: &Game, line: &str, _secret: Option<(PlayerId, &str)>) {
        // the AI's own reasoning is not an action
        if !line.contains(" decides: ") {
            self.0.lock().unwrap().push(line.to_string());
        }
    }
}

impl PublicLines {
    pub fn lines(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// start collecting the public lines of `g`
pub fn watch(g: &mut Game) -> Arc<PublicLines> {
    let sink = Arc::new(PublicLines::default());
    g.link.sink = Some(sink.clone());
    sink
}

fn sorted_names(g: &Game, cs: impl IntoIterator<Item = sim_core::ids::CardId>) -> Vec<String> {
    let mut v: Vec<String> = cs.into_iter().map(|c| g.db.get(c).name.to_string()).collect();
    v.sort();
    v
}

/// the observable state of the game, one line per fact: for each seat the zones (as multisets of card names: a
/// shuffle's order is not a decision), life, permanents with tapped and counters, the mana pools; then the stack
pub fn fingerprint(g: &Game) -> Vec<String> {
    let mut out = vec![];
    for pl in &g.players {
        let p = pl.id;
        out.push(format!("P{} alive={} life={} poison={}", p.index(), pl.alive, pl.life, pl.poison));
        out.push(format!("P{} hand={:?}", p.index(), sorted_names(g, pl.hand.iter().copied())));
        out.push(format!("P{} gy={:?}", p.index(), sorted_names(g, pl.gy.iter().copied())));
        out.push(format!("P{} exile={:?}", p.index(), sorted_names(g, pl.exile.iter().copied())));
        out.push(format!(
            "P{} library={}:{:?}",
            p.index(),
            pl.library.len(),
            sorted_names(g, pl.library.iter().copied())
        ));
        let mut perms: Vec<String> = pl
            .lands
            .iter()
            .chain(pl.perms.iter())
            .map(|&m| {
                let x = g.perm(m);
                format!(
                    "{}{} +{} loyalty={:?} {}/{}",
                    x.name,
                    if x.tapped { " (tapped)" } else { "" },
                    x.plus,
                    x.loyalty,
                    x.pow,
                    x.tgh
                )
            })
            .collect();
        perms.sort();
        out.push(format!("P{} permanents={:?}", p.index(), perms));
        out.push(format!(
            "P{} tokens treasures={} clues={} bloods={} foods={}",
            p.index(),
            pl.treasures,
            pl.clues,
            pl.bloods,
            pl.foods
        ));
        out.push(format!("P{} pool={} engine_mana={:?}", p.index(), pl.pool.text(), pl.mana));
    }
    for (i, it) in g.stack.iter().enumerate() {
        out.push(format!(
            "stack[{i}] {} of P{} countered={} by={:?} exile_after={} counter_target={:?}",
            it.name,
            it.controller.index(),
            it.countered,
            it.countered_by.map(|c| g.db.get(c).name.to_string()),
            it.ctx.exile_after,
            it.ctx.counter
        ));
    }
    out
}

/// the first difference between two lists of lines, as a readable message
fn first_difference(what: &str, ai: &[String], person: &[String]) -> Option<String> {
    let n = ai.len().max(person.len());
    for i in 0..n {
        if ai.get(i) != person.get(i) {
            return Some(format!(
                "{what} differ at #{i}:\n  AI:     {:?}\n  person: {:?}\n--- AI ---\n{}\n--- person ---\n{}",
                ai.get(i),
                person.get(i),
                ai.join("\n"),
                person.join("\n")
            ));
        }
    }
    None
}

/// one scenario to run with the AI and with a person
pub struct Scenario<'a> {
    /// the seat that decides
    pub seat: PlayerId,
    /// a fresh, identical game each call (no person seated)
    pub build: &'a dyn Fn() -> Game,
    /// the engine entry where the seat decides, with the AI at the seat
    pub ai: &'a dyn Fn(&mut Game),
    /// what a person does first that the AI does inside its decision (tapping lands for mana, say): run before the
    /// public lines are collected, since the AI makes no such separate action
    pub person_prep: &'a dyn Fn(&mut Game),
    /// the same entry with a person at the seat (often the same as `ai`)
    pub person: &'a dyn Fn(&mut Game),
    /// read the AI's picks off the games (before the act, after the AI's act) and the AI's public lines, and make the
    /// person's answers
    pub adapter: &'a dyn Fn(&Game, &Game, &[String]) -> Box<dyn FnMut(&Request) -> Answer + Send>,
}

/// what a run left behind
pub struct Outcome {
    pub fingerprint: Vec<String>,
    pub lines: Vec<String>,
}

pub struct Parity {
    pub ai: Outcome,
    pub person: Outcome,
    /// everything the person was asked
    pub asked: Vec<Request>,
}

/// Run `s` with the AI, then with a scripted person who answers what the AI picked; panic with the first difference if
/// the state fingerprints or the public action lines differ. Returns both outcomes and the person's questions, so a
/// test can also check what was offered.
pub fn person_matches_ai(s: &Scenario) -> Parity {
    let before = (s.build)();
    let mut g_ai = (s.build)();
    let lines_ai = watch(&mut g_ai);
    (s.ai)(&mut g_ai);
    let ai = Outcome { fingerprint: fingerprint(&g_ai), lines: lines_ai.lines() };

    let mut g_p = (s.build)();
    let h = Arc::new(ScriptHuman::from_fn((s.adapter)(&before, &g_ai, &ai.lines)));
    g_p.humans.add(s.seat, h.clone());
    (s.person_prep)(&mut g_p);
    let lines_p = watch(&mut g_p);
    (s.person)(&mut g_p);
    let person = Outcome { fingerprint: fingerprint(&g_p), lines: lines_p.lines() };

    if let Some(d) = first_difference("state fingerprints", &ai.fingerprint, &person.fingerprint) {
        panic!("a person answering the AI's picks did not get the AI's state: {d}");
    }
    if let Some(d) = first_difference("public action lines", &ai.lines, &person.lines) {
        panic!("a person answering the AI's picks did not get the AI's action lines: {d}");
    }
    Parity { ai, person, asked: h.asked() }
}

/// the names of the cards that p has in `hand` + `gy` + `exile` + `library`-less zones after but not before (the cards
/// that moved to p's hand or graveyard), as a sorted list: for adapters that read a tutor's picks
pub fn gained(before: &Game, after: &Game, p: PlayerId) -> Vec<String> {
    let names = |g: &Game| {
        let pl = g.player(p);
        let mut v = sorted_names(g, pl.hand.iter().copied());
        v.extend(sorted_names(g, pl.gy.iter().copied()));
        v
    };
    let mut left = names(before);
    let mut out = vec![];
    for n in names(after) {
        match left.iter().position(|x| *x == n) {
            Some(i) => {
                left.remove(i);
            }
            None => out.push(n),
        }
    }
    out.sort();
    out
}

/// the index of the choice that is the card called `name` (a label is the name, then the cost: "Sol Ring {1}"; a
/// stack label "Name (Controller)"), or None
pub fn choice_of(r: &Request, name: &str) -> Option<usize> {
    r.choices
        .iter()
        .position(|c| c == name || c.starts_with(&format!("{name} {{")) || c.starts_with(&format!("{name} (")))
}
