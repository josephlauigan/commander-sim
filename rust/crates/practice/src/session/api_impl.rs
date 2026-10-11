//! The server's view of a session: `SessionApi` for a `Session`, and the factory the server makes games with.

use super::{Options, Session};
use crate::server::api::{Info, SessionApi, SessionFactory};
use serde_json::Value;
use sim_core::run::Engine;
use std::sync::Arc;
use std::time::Duration;

pub struct SessionHandle(pub Arc<Session>);

impl SessionApi for SessionHandle {
    fn info(&self) -> Info {
        let s = &self.0;
        let st = s.st.lock().unwrap();
        Info {
            deck: s.deck.clone(),
            tier: s.tier.clone(),
            seed: s.seed,
            seats: s.seats.clone(),
            profile: s.profile.clone(),
            ai: s.ai.clone(),
            humans: s.humans.clone(),
            names: s.names.clone(),
            partner: s.partner.clone(),
            waiting: st.current.is_some(),
            finished: st.finished,
        }
    }
    fn autopass_of(&self, seat: &str) -> String {
        self.0.autopass_at(seat, Some(usize::MAX / 2))
    }
    fn events_after(&self, cursor: u64, timeout: Duration) -> (u64, Vec<Value>) {
        self.0.events_after(cursor, timeout)
    }
    fn start(&self) {
        self.0.start();
    }
    fn answer(&self, seat: &str, value: Value) {
        self.0.answer(seat, &value);
    }
    fn undo(&self, n: i64, seat: &str) -> Option<String> {
        self.0.undo(n, seat)
    }
    fn try_it(&self, n: &Value, seat: &str) -> Option<String> {
        self.0.try_it(n, seat)
    }
    fn undo_point(&self, n: i64, seat: &str) -> Option<usize> {
        self.0.undo_point(n, seat)
    }
    fn answer_seats(&self) -> Vec<String> {
        self.0.st.lock().unwrap().answer_seats.clone()
    }
    fn has_try_entry(&self, n: &Value, seat: &str) -> bool {
        self.0.try_entry(n, seat).is_some()
    }
    fn set_autopass(&self, seat: &str, mode: &Value) -> Option<String> {
        match mode.as_str() {
            Some(m) => self.0.set_autopass(seat, m),
            None => Some(format!("auto-pass: one of {}", super::AUTOPASS.join(", "))),
        }
    }
    fn hint(&self, seat: &str) -> Result<Value, String> {
        self.0.hint(seat)
    }
    fn review(&self, seat: &str) -> Result<Value, String> {
        self.0.review(seat)
    }
    fn saved(&self, _wait: Option<Duration>) -> Result<Value, String> {
        // a hint works on a copy of the game, so nothing holds the game still: the save never waits
        Ok(self.0.saved())
    }
    fn close(&self, join: Duration) {
        self.0.close(join);
    }
}

/// makes real sessions on the loaded engine
pub struct RealSessions {
    pub engine: Arc<Engine>,
}

fn opt_str(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn opt_list(v: &Value, k: &str) -> Option<Vec<String>> {
    v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
}

impl RealSessions {
    fn options(&self, opts: &Value) -> Result<Options, String> {
        let deck = opt_str(opts, "deck").ok_or("unknown deck None")?;
        let tier = opt_str(opts, "tier").ok_or("unknown tier None")?;
        let mut o = Options::new(self.engine.clone(), &deck, &tier);
        o.seed = opts.get("seed").and_then(Value::as_u64);
        o.seat = opts.get("seat").and_then(Value::as_u64).map(|n| n as usize);
        o.opponents = opt_list(opts, "opponents");
        if let Some(p) = opt_str(opts, "profile") {
            o.profile = p;
        }
        if let Some(a) = opt_str(opts, "ai") {
            o.ai = a;
        }
        o.views = true;
        o.compare = opts.get("tools").and_then(|t| t.get("compare")).and_then(Value::as_bool).unwrap_or(true);
        o.seats = opt_list(opts, "seats");
        o.partner = opt_str(opts, "partner");
        if let Some(a) = opt_str(opts, "autopass") {
            o.autopass = a;
        }
        Ok(o)
    }
}

impl SessionFactory for RealSessions {
    fn new_session(&self, opts: &Value) -> Result<Arc<dyn SessionApi>, String> {
        Ok(Arc::new(SessionHandle(Session::new(self.options(opts)?)?)))
    }
    fn check_two(&self, opts: &Value, partner: &str) -> Result<(), String> {
        let mut o = self.options(opts)?;
        o.partner = Some(partner.to_string());
        Session::new(o).map(|_| ())
    }
    fn load_session(&self, data: &Value, compare: bool) -> Result<Arc<dyn SessionApi>, String> {
        Ok(Arc::new(SessionHandle(Session::load(self.engine.clone(), data, true, compare)?)))
    }
    fn card_names(&self, info: &Info) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut out = vec![];
        for k in &info.seats {
            if let Some(d) = self.engine.decks.get(k) {
                for n in d.cards.iter().chain(std::iter::once(&d.commander)) {
                    if seen.insert(n.clone()) {
                        out.push(n.clone());
                    }
                }
            }
        }
        out
    }
}
