//! Shared helpers for the session tests: the engine, and a driver that plays a person's side with simple answers.

#![allow(dead_code)]

use practice::session::{Options, Session};
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

pub fn engine() -> Arc<Engine> {
    static E: OnceLock<Arc<Engine>> = OnceLock::new();
    E.get_or_init(|| {
        let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
        Arc::new(Engine::load(&data).expect("load the card data"))
    })
    .clone()
}

pub fn opts(ai: &str) -> Options {
    let mut o = Options::new(engine(), "sauron", "t1");
    o.seed = Some(4242);
    o.ai = ai.into();
    o.views = true;
    o.compare = true;
    o.max_rounds = 6;
    o
}

/// the answer a simple player gives to a request: keep, pass, no attack, the first choice
pub fn simple_answer(req: &Value) -> Value {
    match req["kind"].as_str().unwrap_or("") {
        "priority" => json!({"do": "pass"}),
        "attack" => json!([]),
        "mulligan" => json!(0),
        _ => json!(0),
    }
}

/// Play the session's event stream until the game is over (or an error): answers each request with `policy`.
/// Returns every event seen. Panics after `limit`.
pub fn drive(s: &Arc<Session>, mut cursor: u64, limit: Duration, policy: &dyn Fn(&Value, usize) -> Option<Value>) -> (u64, Vec<Value>) {
    let end = Instant::now() + limit;
    let mut all = vec![];
    let mut asked = 0;
    loop {
        assert!(Instant::now() < end, "the game didn't finish in time; last events: {:?}", &all[all.len().saturating_sub(5)..]);
        let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
        cursor = c;
        for ev in evs {
            match ev["kind"].as_str().unwrap_or("") {
                "request" => {
                    let seat = ev["seat"].as_str().unwrap().to_string();
                    asked += 1;
                    let a = policy(&ev["request"], asked).unwrap_or_else(|| simple_answer(&ev["request"]));
                    all.push(ev);
                    s.answer(&seat, &a);
                }
                "over" | "error" => {
                    let done = true;
                    all.push(ev);
                    if done {
                        return (cursor, all);
                    }
                }
                _ => all.push(ev),
            }
        }
    }
}

pub fn over(evs: &[Value]) -> &Value {
    evs.last().filter(|e| e["kind"] == "over").unwrap_or_else(|| panic!("no over event; last: {:?}", evs.last()))
}

pub fn kinds(evs: &[Value]) -> Vec<String> {
    evs.iter().map(|e| e["kind"].as_str().unwrap_or("?").to_string()).collect()
}
