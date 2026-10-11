//! One practice game (Python's `play/session.py`): seats the table, runs the engine on its own thread, and streams
//! what happens as events (JSON shaped like Python's `Session.events`; contract in `server/api.rs`).
//!
//! One or two people play (two: each on their own computer, through the server; `partner` is the second person's
//! deck). Each person's seat has its own hint and AI comparison; their answers go into one list in the order the
//! game asked for them, so Undo and saved games replay both.
//!
//! Events: `{"kind":"log","text"}`, `{"kind":"turn","player","view","views"}`, `{"kind":"request","request":{kind,
//! prompt,choices,data},"seat"}`, `{"kind":"over","winner","how","view","views"}`, `{"kind":"error","text"}`,
//! `reset`, `reviewing`, and the messages a person is told (`info`, `warn`, ...). An event meant for one seat only
//! has `seat`; one every seat but that one sees has `hide`; one shown at once rather than played back has `replay`.

pub mod api_impl;
pub mod human;
pub mod setup;

use crate::advisor;
use crate::shadow::{Entry, Shadow};
use crate::view::build_view;
use human::{SessionHuman, answer_from_json, answer_to_json};
use serde_json::{Value, json};
use sim_core::ai::tape::{Tape, TapeEntry, Taped};
use sim_core::control::{Answer, LogSink, Request};
use sim_core::ids::PlayerId;
use sim_core::run::{Engine, settings};
use sim_core::state::Game;
use sim_core::sym::intern;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub use setup::{MY_DECKS, TIERS};

pub const SAVE_VERSION: u64 = 1;
/// the review counts a decision "clearly worse" when the AI's choice scored this much higher
pub const WORSE: f64 = 1.0;
pub const AUTOPASS: [&str; 3] = ["respond", "stack", "all"];

/// the AI's reasoning traces ('[Tergrid decides: ...]') name cards still in an opponent's hand: not shown in play
pub fn is_private(line: &str) -> bool {
    let body = if line.starts_with('R') { line.get(4..).unwrap_or("") } else { line };
    body.trim_start().starts_with('[')
}

/// a top-level action ('R4    Tergrid casts Jet Medallion'), not one of its effects (indented further) or a header
pub fn is_action(line: &str) -> bool {
    let body = if line.starts_with('R') { line.get(4..).unwrap_or("") } else { line };
    body.starts_with("  ") && !body.starts_with("   ")
}

#[derive(Clone)]
pub struct Options {
    pub engine: Arc<Engine>,
    pub deck: String,
    pub tier: String,
    pub seed: Option<u64>,
    pub seat: Option<usize>,
    pub opponents: Option<Vec<String>>,
    pub profile: String,
    pub ai: String,
    pub step: bool,
    pub max_rounds: u32,
    pub views: bool,
    pub compare: bool,
    pub seats: Option<Vec<String>>,
    pub partner: Option<String>,
    pub autopass: String,
}

impl Options {
    pub fn new(engine: Arc<Engine>, deck: &str, tier: &str) -> Options {
        Options {
            engine,
            deck: deck.into(),
            tier: tier.into(),
            seed: None,
            seat: None,
            opponents: None,
            profile: "loose".into(),
            ai: "lookahead".into(),
            step: false,
            max_rounds: 30,
            views: false,
            compare: false,
            seats: None,
            partner: None,
            autopass: "respond".into(),
        }
    }
}

/// the decision waiting for a person
pub(crate) struct Current {
    pub seat: String,
    pub req: Request,
    /// the game as it stands (a copy), for the Hint
    pub snap: Arc<Mutex<Game>>,
}

/// one run of the game thread: its abort flag and the queues its people answer on
pub(crate) struct Gen {
    pub abort: Arc<AtomicBool>,
    pub tx: HashMap<String, Sender<Answer>>,
}

impl Gen {
    pub fn aborted(&self) -> bool {
        self.abort.load(Ordering::Relaxed)
    }
    fn close(&self) {
        self.abort.store(true, Ordering::Relaxed);
        for t in self.tx.values() {
            let _ = t.send(Answer::Cancel);
        }
    }
}

pub(crate) struct Outcome {
    pub winner_key: Option<String>,
    pub winner_name: Option<String>,
    pub how: String,
    pub round: u32,
}

pub(crate) struct State {
    pub events: Vec<Value>,
    pub answers: Vec<Answer>,
    pub marks: Vec<usize>,
    pub answer_seats: Vec<String>,
    pub replay: VecDeque<Answer>,
    pub replaying: bool,
    pub current: Option<Current>,
    pub finished: bool,
    pub restarting: bool,
    pub round: u32,
    pub autopass_start: Vec<(String, String)>,
    /// changes during the game: (from answer number, seat, mode)
    pub autopass_log: Vec<(usize, String, String)>,
    pub run: Gen,
    pub thread: Option<JoinHandle<()>>,
    pub outcome: Option<Outcome>,
}

pub struct Session {
    pub engine: Arc<Engine>,
    pub deck: String,
    pub tier: String,
    pub profile: String,
    pub ai: String,
    pub step: bool,
    pub max_rounds: u32,
    pub partner: Option<String>,
    /// the seats people play
    pub humans: Vec<String>,
    pub views: bool,
    pub compare: bool,
    pub seed: u64,
    pub seats: Vec<String>,
    /// deck key -> the name the table shows
    pub names: Vec<(String, String)>,
    pub(crate) st: Mutex<State>,
    pub(crate) cv: Condvar,
    pub tape: Arc<Mutex<Tape>>,
    pub shadows: HashMap<String, Arc<Shadow>>,
}

struct Sink(Arc<Session>);

impl LogSink for Sink {
    fn line(&self, g: &Game, line: &str, secret: Option<(PlayerId, &str)>) {
        match secret {
            None => self.0.on_log(g, line, None, None),
            Some((p, full)) => {
                let key = g.player(p).key.to_string();
                self.0.on_log(g, line, None, Some(&key));
                if self.0.humans.contains(&key) {
                    self.0.on_log(g, full, Some(&key), None);
                }
            }
        }
    }
}

fn panic_text(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "panic".into())
}

impl Session {
    pub fn new(o: Options) -> Result<Arc<Session>, String> {
        let e = &o.engine;
        if !MY_DECKS.contains(&o.deck.as_str()) {
            return Err(format!("unknown deck {:?}: one of {}", o.deck, MY_DECKS.join(", ")));
        }
        if !TIERS.contains(&o.tier.as_str()) {
            return Err(format!("unknown tier {:?}: one of {}", o.tier, TIERS.join(", ")));
        }
        if let Some(p) = &o.partner {
            if !MY_DECKS.contains(&p.as_str()) {
                return Err(format!("unknown deck {p:?}: one of {}", MY_DECKS.join(", ")));
            }
            if *p == o.deck {
                return Err("the two players need different decks".into());
            }
        }
        settings(&o.profile, &o.ai, 1.0)?;
        if !e.decks.contains_key(&o.deck) {
            return Err(format!("{:?} isn't in decks.json", o.deck));
        }
        let humans: Vec<String> = std::iter::once(o.deck.clone()).chain(o.partner.clone()).collect();
        let seed = o.seed.unwrap_or_else(|| {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
            1 + sim_core::rng::Rng::from_u64(t).below(999_999)
        });
        let seats = setup::choose_seats(
            e,
            &setup::SeatArgs {
                deck: &o.deck,
                tier: &o.tier,
                partner: o.partner.as_deref(),
                seed,
                seat: o.seat,
                opponents: o.opponents.as_deref(),
                seats: o.seats.as_deref(),
            },
        )?;
        let names = seats.iter().map(|k| (k.clone(), e.decks.get(k).map_or(k.clone(), |d| d.name.clone()))).collect();
        let shadows = humans.iter().map(|k| (k.clone(), Arc::new(Shadow::new()))).collect();
        Ok(Arc::new(Session {
            engine: o.engine.clone(),
            deck: o.deck,
            tier: o.tier,
            profile: o.profile,
            ai: o.ai,
            step: o.step,
            max_rounds: o.max_rounds,
            partner: o.partner,
            views: o.views,
            compare: o.compare,
            seed,
            seats,
            names,
            st: Mutex::new(State {
                events: vec![],
                answers: vec![],
                marks: vec![],
                answer_seats: vec![],
                replay: VecDeque::new(),
                replaying: false,
                current: None,
                finished: false,
                restarting: false,
                round: 0,
                autopass_start: humans.iter().map(|k| (k.clone(), o.autopass.clone())).collect(),
                autopass_log: vec![],
                run: Gen { abort: Arc::new(AtomicBool::new(false)), tx: HashMap::new() },
                thread: None,
                outcome: None,
            }),
            humans,
            cv: Condvar::new(),
            tape: Arc::new(Mutex::new(Tape::default())),
            shadows,
        }))
    }

    pub fn answers_len(&self) -> usize {
        self.st.lock().unwrap().answers.len()
    }

    pub fn name_of(&self, key: &str) -> String {
        self.names.iter().find(|(k, _)| k == key).map_or(key.to_string(), |x| x.1.clone())
    }

    // ------------------------------------------------------------------ events
    /// one event for whoever reads the stream (shown at once, not played back, while the session replays)
    pub fn emit(&self, mut ev: Value) {
        let mut st = self.st.lock().unwrap();
        if st.replaying {
            ev["replay"] = true.into();
        }
        st.events.push(ev);
        drop(st);
        self.cv.notify_all();
    }

    /// raw events after `cursor`, waiting up to `timeout` for one; returns the new cursor and the events
    pub fn events_after(&self, cursor: u64, timeout: Duration) -> (u64, Vec<Value>) {
        let end = Instant::now() + timeout;
        let mut st = self.st.lock().unwrap();
        loop {
            let c = cursor as usize;
            if st.events.len() > c {
                return (st.events.len() as u64, st.events[c..].to_vec());
            }
            let now = Instant::now();
            if now >= end {
                return (cursor, vec![]);
            }
            st = self.cv.wait_timeout(st, end - now).unwrap().0;
        }
    }

    fn with_views(&self, ev: &mut Value, g: &Game) {
        let views: serde_json::Map<String, Value> = self.humans.iter().map(|k| (k.clone(), build_view(g, k))).collect();
        ev["view"] = views[&self.deck].clone();
        ev["views"] = Value::Object(views);
    }

    fn on_log(&self, g: &Game, line: &str, seat: Option<&str>, hide: Option<&str>) {
        if is_private(line) {
            return; // never shown in play
        }
        let mut ev = json!({"kind": "log", "text": line});
        if let Some(s) = seat {
            ev["seat"] = s.into();
        }
        if let Some(h) = hide {
            ev["hide"] = h.into();
        }
        if is_action(line) && self.views && !self.st.lock().unwrap().replaying {
            self.with_views(&mut ev, g); // the table after each action, for the browser's playback
        }
        self.emit(ev);
        if line.contains("--- ")
            && line.contains(" turn ")
            && let Some(a) = g.active
        {
            let mut t = json!({"kind": "turn", "player": &*g.player(a).key});
            self.with_views(&mut t, g);
            self.emit(t);
            if self.step
                && let Some(me) = g.players.iter().find(|q| q.key == self.deck.as_str()).map(|q| q.id)
                && let Some(h) = g.humans.get(me).cloned()
            {
                let req = Request {
                    kind: intern("continue"),
                    prompt: format!("{}: turn {}", g.player(a).name, g.player(a).turns),
                    choices: vec![],
                    data: json!({}),
                };
                h.ask(g, me, req);
            }
        }
    }

    // ------------------------------------------------------------------ running
    /// start the game on its own thread
    pub fn start(self: &Arc<Self>) {
        let abort = Arc::new(AtomicBool::new(false));
        let mut tx = HashMap::new();
        let mut rxs = vec![];
        for k in &self.humans {
            let (t, r) = channel();
            tx.insert(k.clone(), t);
            rxs.push((k.clone(), r));
        }
        let me = self.clone();
        let flag = abort.clone();
        let h = std::thread::Builder::new()
            .name("practice-game".into())
            .stack_size(64 << 20)
            .spawn(move || {
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| me.clone().play(flag, rxs)));
                if let Err(p) = r {
                    me.emit(json!({"kind": "error", "text": panic_text(&*p)}));
                }
            })
            .expect("spawn game thread");
        let mut st = self.st.lock().unwrap();
        st.run = Gen { abort, tx };
        st.thread = Some(h);
    }

    fn closed(&self) {
        if !self.st.lock().unwrap().restarting {
            self.emit(json!({"kind": "over", "view": null, "winner": null, "how": "closed"}));
        }
    }

    fn play(self: Arc<Self>, abort: Arc<AtomicBool>, rxs: Vec<(String, std::sync::mpsc::Receiver<Answer>)>) {
        let st = settings(&self.profile, &self.ai, 1.0).expect("checked in new");
        let (mut g, mull) = match setup::build_game(&self.engine, st, self.seed, &self.seats, &self.humans) {
            Ok(x) => x,
            Err(e) => return self.emit(json!({"kind": "error", "text": e})),
        };
        for (k, rx) in rxs {
            let pid = g.players.iter().find(|q| q.key == k.as_str()).unwrap().id;
            let h = SessionHuman { session: self.clone(), seat: k, rx: Mutex::new(rx) };
            g.humans.add(pid, Arc::new(h));
        }
        g.link.sink = Some(Arc::new(Sink(self.clone())));
        g.link.abort = Some(abort.clone());
        g.search_tape = sim_core::ai::tape::TapeSlot(Some(self.tape.clone()));
        let order: Vec<String> = g.players.iter().map(|p| p.name.to_string()).collect();
        self.on_log(&g, &format!("Seat order: {}", order.join(", ")), None, None);
        for (p, mut r) in mull {
            if sim_core::human::choices::mulligan(&mut g, p, &mut r).is_err() {
                return self.closed();
            }
        }
        sim_core::engine::turn::run_rounds(&mut g, self.max_rounds);
        if abort.load(Ordering::Relaxed) {
            return self.closed();
        }
        if self.compare {
            // the comparisons not yet worked out, before the review
            let jobs: Vec<&Arc<Shadow>> = self.shadows.values().filter(|s| s.pending() > 0).collect();
            let total: usize = jobs.iter().map(|s| s.pending()).sum();
            let mut done = 0;
            for sh in jobs {
                let k = sh.pending();
                let base = done;
                sh.finish(
                    &mut |d, _| self.emit(json!({"kind": "reviewing", "done": base + d, "total": total})),
                    &abort,
                );
                done += k;
            }
            if abort.load(Ordering::Relaxed) {
                return self.closed();
            }
        }
        let winner = g.winner.map(|w| g.player(w));
        {
            let mut s = self.st.lock().unwrap();
            s.finished = true;
            s.round = g.round;
            s.outcome = Some(Outcome {
                winner_key: winner.map(|w| w.key.to_string()),
                winner_name: winner.map(|w| w.name.to_string()),
                how: g.wintype.unwrap_or("").to_string(),
                round: g.round,
            });
        }
        let mut ev = json!({"kind": "over", "winner": winner.map(|w| w.key.to_string()), "how": g.wintype.unwrap_or("")});
        self.with_views(&mut ev, &g);
        self.emit(ev);
    }

    // ------------------------------------------------------------------ auto-pass
    /// seat's auto-pass setting once `at` answers have been given (now, by default). A change made while the game
    /// waits on decision n applies from answer n + 1, so a replay (Undo, a saved game) stops where it did
    pub fn autopass_at(&self, seat: &str, at: Option<usize>) -> String {
        let st = self.st.lock().unwrap();
        autopass_of(&st, seat, at)
    }

    pub fn set_autopass(&self, seat: &str, mode: &str) -> Option<String> {
        if !AUTOPASS.contains(&mode) {
            return Some(format!("auto-pass: one of {}", AUTOPASS.join(", ")));
        }
        if !self.humans.iter().any(|h| h == seat) {
            return Some("You have no seat in this game.".into());
        }
        let mut st = self.st.lock().unwrap();
        let n = st.answers.len() + 1;
        st.autopass_log.push((n, seat.into(), mode.into()));
        None
    }

    // ------------------------------------------------------------------ Undo, Try it
    /// where Undo of seat's last n answers rewinds to (an index into answers), or None
    pub fn undo_point(&self, n: i64, seat: &str) -> Option<usize> {
        let st = self.st.lock().unwrap();
        let mine: Vec<usize> = (0..st.answer_seats.len()).filter(|&i| st.answer_seats[i] == seat).collect();
        if n < 1 || (mine.len() as i64) < n {
            return None;
        }
        Some(mine[mine.len() - n as usize])
    }

    /// take back your last n answers (with two players, any of the other person's made since go too): the game
    /// restarts from its seed and replays the rest, then asks the decision before them again. None, or why not
    pub fn undo(self: &Arc<Self>, n: i64, seat: &str) -> Option<String> {
        let Some(to) = self.undo_point(n, seat) else { return Some("Nothing to undo yet.".into()) };
        self.rewind(Some(to), None, json!({"undone": n, "by": seat}))
    }

    pub fn try_entry(&self, n: &Value, seat: &str) -> Option<Entry> {
        let n = n.as_u64()? as usize;
        let e = self.shadows.get(seat)?.entry(n)?;
        (e.scored && e.ai.is_some()).then_some(e)
    }

    /// the review's "Try it": back to decision n, with the AI's choice there instead of yours (when it maps onto
    /// one action; otherwise you're back at the decision and the AI's choice is shown for you to make)
    pub fn try_it(self: &Arc<Self>, n: &Value, seat: &str) -> Option<String> {
        let Some(e) = self.try_entry(n, seat) else { return Some("That decision has no AI choice to try.".into()) };
        let then = e.ai_answer();
        let info = json!({"tryit": {"n": e.n, "ai": e.ai_text(), "applied": then.is_some(), "by": seat}});
        self.rewind(Some(e.n), then, info)
    }

    /// a rewind to answer `to`: the replay keeps the settings it had; the setting now applies from there on
    fn autopass_rewind(&self, st: &mut State, to: usize) {
        let latest: Vec<(String, String)> = self.humans.iter().map(|k| (k.clone(), autopass_of(st, k, Some(usize::MAX / 2)))).collect();
        st.autopass_log.retain(|e| e.0 <= to);
        for (k, m) in latest {
            if autopass_of(st, &k, Some(to + 1)) != m {
                st.autopass_log.push((to + 1, k, m));
            }
        }
    }

    /// restart the game and replay your first `to` answers (then `then`, if given, as your answer to decision `to`);
    /// the decisions after are live again. None, or why not
    pub fn rewind(self: &Arc<Self>, to: Option<usize>, then: Option<Answer>, info: Value) -> Option<String> {
        let (to, handle) = {
            let mut st = self.st.lock().unwrap();
            let to = match to {
                Some(t) if t <= st.answers.len() => t,
                _ => return Some("No such decision.".into()),
            };
            self.autopass_rewind(&mut st, to);
            st.restarting = true;
            st.run.close();
            (to, st.thread.take())
        };
        if let Some(h) = handle {
            let end = Instant::now() + Duration::from_secs(30);
            while !h.is_finished() && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(5));
            }
            if !h.is_finished() {
                let mut st = self.st.lock().unwrap();
                st.thread = Some(h);
                st.restarting = false;
                return Some("The game is busy; try again in a moment.".into());
            }
            let _ = h.join();
        }
        {
            let mut st = self.st.lock().unwrap();
            let keep: Vec<Answer> = st.answers[..to].to_vec();
            let cut = st.marks.get(to).copied().unwrap_or_else(|| self.tape.lock().unwrap().entries.len());
            self.tape.lock().unwrap().cut(cut);
            for sh in self.shadows.values() {
                sh.forget_from(keep.len());
            }
            st.answers.clear();
            st.marks.clear();
            st.answer_seats.clear();
            st.replay = keep.into_iter().chain(then).collect();
            st.finished = false;
            st.outcome = None;
            st.current = None;
            st.replaying = false;
            let mut ev = json!({"kind": "reset", "undone": 0});
            if let Value::Object(m) = info {
                for (k, v) in m {
                    ev[k] = v;
                }
            }
            st.events.push(ev);
            st.replaying = !st.replay.is_empty();
            st.restarting = false;
        }
        self.cv.notify_all();
        self.start();
        None
    }

    // ------------------------------------------------------------------ answers, hint, review
    pub fn answer(&self, seat: &str, value: &Value) {
        let tx = self.st.lock().unwrap().run.tx.get(seat).cloned();
        if let Some(t) = tx {
            let _ = t.send(answer_from_json(value));
        }
    }

    pub fn hint(&self, seat: &str) -> Result<Value, String> {
        let (req, snap) = {
            let st = self.st.lock().unwrap();
            match &st.current {
                Some(c) if c.seat == seat => (c.req.clone(), c.snap.clone()),
                _ => return Err("There is no decision waiting for you.".into()),
            }
        };
        let mut g = snap.lock().unwrap();
        let Some(p) = g.players.iter().find(|q| q.key == seat).map(|q| q.id) else {
            return Err("There is no decision waiting for you.".into());
        };
        Ok(advisor::hint(&mut g, p, &req))
    }

    /// the AI comparison for a seat (yours), once the game is over (it stays hidden while you play)
    pub fn review(&self, seat: &str) -> Result<Value, String> {
        let st = self.st.lock().unwrap();
        let (Some(o), true) = (&st.outcome, st.finished) else { return Err("The review opens when the game ends.".into()) };
        let Some(sh) = self.shadows.get(seat) else { return Err("You have no seat in this game.".into()) };
        let ds = sh.review();
        let scored: Vec<&Value> = ds.iter().filter(|d| d["scored"] == true).collect();
        let f = |d: &Value, k: &str| d[k].as_f64().unwrap_or(0.0);
        let summary = json!({
            "won": o.winner_key.as_deref() == Some(seat), "winner": o.winner_name, "how": o.how, "rounds": o.round,
            "answers": st.answer_seats.iter().filter(|s| *s == seat).count(), "decisions": ds.len(),
            "compared": scored.len(), "matched": scored.iter().filter(|d| d["differs"] == false).count(),
            "worse": scored.iter().filter(|d| f(d, "score_ai") - f(d, "score_yours") >= WORSE).count(),
            "worse_margin": WORSE,
        });
        Ok(json!({"summary": summary, "decisions": ds}))
    }

    // ------------------------------------------------------------------ saving and loading
    /// the game as a file: the seed and table, your answers, the AI's look-ahead decisions and the comparison.
    /// Loading it replays the same game (on the same code: a change to card rules or the AI can change a game).
    pub fn saved(&self) -> Value {
        let st = self.st.lock().unwrap();
        let tape: Vec<Value> = self.tape.lock().unwrap().entries.iter().map(tape_json).collect();
        let mut d = json!({
            "version": SAVE_VERSION, "deck": self.deck, "tier": self.tier, "seed": self.seed, "seats": self.seats,
            "profile": self.profile, "ai": self.ai, "max_rounds": self.max_rounds,
            "answers": st.answers.iter().map(answer_to_json).collect::<Vec<_>>(), "tape": tape,
            "shadow": self.shadows[&self.deck].saved(), "finished": st.finished, "round": st.round,
            "autopass": st.autopass_start.iter().cloned().collect::<HashMap<_, _>>(),
            "autopass_log": st.autopass_log.iter().map(|(n, k, m)| json!([n, k, m])).collect::<Vec<_>>(),
        });
        if let Some(p) = &self.partner {
            d["partner"] = p.clone().into();
            d["answer_seats"] = json!(st.answer_seats);
            d["partner_shadow"] = self.shadows[p].saved().into();
        }
        d
    }

    /// a session that replays a saved game to where it was saved (a finished game to its end)
    pub fn load(engine: Arc<Engine>, data: &Value, views: bool, compare: bool) -> Result<Arc<Session>, String> {
        if data.get("version").and_then(Value::as_u64) != Some(SAVE_VERSION) {
            return Err("a saved game from a different version".into());
        }
        let damaged = || "a damaged saved game".to_string();
        let s = |k: &str| data.get(k).and_then(Value::as_str).map(str::to_string).ok_or_else(damaged);
        let mut o = Options::new(engine, &s("deck")?, &s("tier")?);
        o.seed = Some(data.get("seed").and_then(Value::as_u64).ok_or_else(damaged)?);
        o.profile = s("profile")?;
        o.ai = s("ai")?;
        o.max_rounds = data.get("max_rounds").and_then(Value::as_u64).unwrap_or(30) as u32;
        o.views = views;
        o.compare = compare;
        o.seats = Some(
            data.get("seats").and_then(Value::as_array).ok_or_else(damaged)?.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        );
        o.partner = data.get("partner").and_then(Value::as_str).map(str::to_string);
        let sess = Session::new(o)?;
        {
            let mut t = sess.tape.lock().unwrap();
            for e in data.get("tape").and_then(Value::as_array).ok_or_else(damaged)? {
                t.entries.push(tape_from_json(e).ok_or_else(damaged)?);
            }
        }
        let entries = |k: &str| -> Result<Vec<Entry>, String> {
            data.get(k).and_then(Value::as_array).map(|a| a.iter().map(Entry::from_saved).collect()).unwrap_or(Ok(vec![]))
        };
        sess.shadows[&sess.deck].load(entries("shadow")?);
        if let Some(p) = &sess.partner {
            sess.shadows[p].load(entries("partner_shadow")?);
        }
        let mut st = sess.st.lock().unwrap();
        if let Some(m) = data.get("autopass").and_then(Value::as_object) {
            for (k, v) in m {
                if let (Some(v), Some(e)) = (v.as_str(), st.autopass_start.iter_mut().find(|e| &e.0 == k)) {
                    e.1 = v.to_string();
                }
            }
        }
        st.autopass_log = data
            .get("autopass_log")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|e| Some((e.get(0)?.as_u64()? as usize, e.get(1)?.as_str()?.to_string(), e.get(2)?.as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        st.replay = data
            .get("answers")
            .and_then(Value::as_array)
            .ok_or_else(damaged)?
            .iter()
            .map(answer_from_json)
            .collect();
        if let Some(a) = data.get("answer_seats").and_then(Value::as_array) {
            // (a two-player save knows whose answer each one is; replay re-derives it, so this only checks the shape)
            if a.len() != st.replay.len() {
                return Err(damaged());
            }
        }
        st.replaying = !st.replay.is_empty();
        drop(st);
        Ok(sess)
    }

    // ------------------------------------------------------------------ closing
    /// end the game and wait up to `join` for its thread (and the comparison workers) to finish
    pub fn close(&self, join: Duration) {
        let h = {
            let mut st = self.st.lock().unwrap();
            st.run.close();
            st.thread.take()
        };
        for sh in self.shadows.values() {
            sh.shutdown();
        }
        if let Some(h) = h {
            let end = Instant::now() + join;
            while !h.is_finished() && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(5));
            }
            if h.is_finished() {
                let _ = h.join();
            } else {
                self.st.lock().unwrap().thread = Some(h);
            }
        }
    }

    /// is the game thread (or a comparison worker) still alive?
    pub fn threads_running(&self) -> bool {
        let st = self.st.lock().unwrap();
        st.thread.as_ref().is_some_and(|h| !h.is_finished()) || self.shadows.values().any(|s| s.workers_running() > 0)
    }
}

fn autopass_of(st: &State, seat: &str, at: Option<usize>) -> String {
    let n = at.unwrap_or(st.answers.len());
    let mut mode = st.autopass_start.iter().find(|e| e.0 == seat).map_or("respond", |e| e.1.as_str());
    for (i, k, m) in &st.autopass_log {
        if k == seat && *i <= n {
            mode = m;
        }
    }
    mode.to_string()
}

fn tape_json(e: &TapeEntry) -> Value {
    let r = match &e.result {
        Taped::Main(x) => x.as_ref().map_or(Value::Null, |(l, n)| json!([l, n])),
        Taped::Attack(x) => x.map_or(Value::Null, |(p, m)| json!([p.index(), m])),
        Taped::Counter(b) => json!(b),
    };
    json!([e.result.kind(), r, e.work])
}

fn tape_from_json(v: &Value) -> Option<TapeEntry> {
    let kind = v.get(0)?.as_str()?;
    let r = v.get(1)?;
    let work = v.get(2)?.as_u64()?;
    let result = match kind {
        "main" => Taped::Main(if r.is_null() { None } else { Some((r.get(0)?.as_str()?.to_string(), r.get(1)?.as_u64()? as usize)) }),
        "attack" => Taped::Attack(if r.is_null() {
            None
        } else {
            Some((PlayerId(r.get(0)?.as_u64()? as u8), intern(r.get(1)?.as_str()?)))
        }),
        "counter" => Taped::Counter(r.as_bool()?),
        _ => return None,
    };
    Some(TapeEntry { result, work })
}
