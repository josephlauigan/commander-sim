//! The person's seat in a running game (Python's `RecordingController` + `HumanController`): `ask` posts a request
//! event and blocks the game thread on a channel until the page answers; while the session replays (Undo, a saved
//! game) the recorded answers are given back without asking.

use super::Session;
use crate::view::build_view;
use serde_json::{Value, json};
use sim_core::control::{Answer, Human, Request};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::sym::Sym;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// what the page sends as an answer, as an `Answer`: a number picks, a list picks several, true/false is yes/no,
/// 'cancel' (or null) is the page's way out of a choice, an object is an action on your own turn
pub fn answer_from_json(v: &Value) -> Answer {
    match v {
        Value::Number(n) => match n.as_u64() {
            Some(i) => Answer::Pick(i as usize),
            None => Answer::Decline,
        },
        Value::Array(a) => Answer::Picks(a.iter().filter_map(|x| x.as_u64().map(|i| i as usize)).collect()),
        Value::Bool(true) => Answer::Yes,
        Value::Bool(false) => Answer::No,
        Value::Object(_) => Answer::Action(v.clone()),
        Value::String(s) => match s.as_str() {
            "keep" | "yes" => Answer::Pick(0),
            "mulligan" | "no" => Answer::Pick(1),
            _ => Answer::Decline,
        },
        Value::Null => Answer::Decline,
    }
}

/// an `Answer` as the page (and a save file) has it
pub fn answer_to_json(a: &Answer) -> Value {
    match a {
        Answer::Pick(i) => json!(i),
        Answer::Picks(v) => json!(v),
        Answer::Yes => json!(true),
        Answer::No => json!(false),
        Answer::Decline | Answer::Cancel => json!("cancel"),
        Answer::Action(v) => v.clone(),
    }
}

pub fn request_json(req: &Request) -> Value {
    json!({"kind": req.kind, "prompt": req.prompt, "choices": req.choices, "data": req.data})
}

pub struct SessionHuman {
    pub session: Arc<Session>,
    /// the deck key of the seat
    pub seat: String,
    pub rx: Mutex<Receiver<Answer>>,
}

impl Human for SessionHuman {
    fn ask(&self, g: &Game, seat: PlayerId, mut req: Request) -> Answer {
        let s = &self.session;
        let mark = s.tape.lock().unwrap().pos;
        // the recorded answers of a replay
        {
            let mut st = s.st.lock().unwrap();
            st.marks.push(mark);
            if let Some(a) = st.replay.pop_front() {
                st.answers.push(a.clone());
                st.answer_seats.push(self.seat.clone());
                return a;
            }
            st.replaying = false; // caught up: this decision is live
            if st.run.aborted() {
                return Answer::Cancel;
            }
        }
        // the table as this seat sees it now (Python adds it to priority, attack and choice requests)
        if !matches!(&*req.kind, "mulligan" | "continue") {
            if !req.data.is_object() {
                req.data = json!({});
            }
            if req.data.get("view").is_none() {
                req.data["view"] = build_view(g, &self.seat);
            }
        }
        let snap = {
            let mut c = g.clone();
            c.cur_cast = g.cur_cast.clone();
            Arc::new(Mutex::new(c))
        };
        {
            let mut st = s.st.lock().unwrap();
            st.current = Some(super::Current { seat: self.seat.clone(), req: req.clone(), snap });
            st.round = g.round;
            st.events.push(json!({"kind": "request", "request": request_json(&req), "seat": self.seat}));
        }
        s.cv.notify_all();
        let ans = self.rx.lock().unwrap().recv().unwrap_or(Answer::Cancel);
        let mut st = s.st.lock().unwrap();
        st.current = None;
        if ans == Answer::Cancel {
            return Answer::Cancel;
        }
        let n = st.answers.len();
        if s.compare
            && let Some(sh) = s.shadows.get(&self.seat)
        {
            sh.record(g, seat, &req, &ans, n); // the AI comparison: copy the game before the answer applies
        }
        st.answers.push(ans.clone());
        st.answer_seats.push(self.seat.clone());
        ans
    }

    fn tell(&self, _seat: PlayerId, kind: Sym, text: &str) {
        if self.session.st.lock().unwrap().replaying {
            return;
        }
        self.session.emit(json!({"kind": kind, "text": text, "seat": self.seat}));
    }

    fn live(&self) -> bool {
        true
    }

    fn autopass(&self) -> Option<&'static str> {
        Some(match self.session.autopass_at(&self.seat, None).as_str() {
            "stack" => "stack",
            "all" => "all",
            _ => "respond",
        })
    }
}
