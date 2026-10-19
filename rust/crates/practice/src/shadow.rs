//! The AI comparison log (practice mode's shadow log; Python's `play/shadow.py`): at each decision the AI would
//! weigh, what you did, what the AI would have done, and how the two score when the game is played on from there.
//!
//! When you answer a decision worth comparing (a play in your main phase, or your attack), the game is copied
//! before your answer takes effect. The comparison is worked out on that copy by worker threads (Python did it on
//! the game's own thread while it waited for your answer; Rust copies are independent, so workers run beside the
//! game). Whatever is left when the game ends is finished before the review opens.
//!
//! Scores are the look-ahead's: the mean over `rollouts` playouts of where the position stands at the end of your
//! next turn, from -100 (lost) to +100 (won). The same futures are used for every option of a decision. A decision
//! with one sensible option, or one the AI doesn't weigh this way (tapping mana, playing a land, an ability), is
//! recorded without a score.

use crate::view::{seam::pool_total, step_name};
use serde_json::{Value, json};
use sim_core::ai::{brain, search};
use sim_core::control::{Answer, Request};
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};
use sim_core::sym::intern;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// options scoring within this of each other count as the same choice (no difference shown)
pub const SAME: f64 = 0.1;

enum Job {
    Main { snap: Game, sp: PlayerId, post: bool, label: String },
    Attack { snap: Game, sp: PlayerId, label: Option<(usize, String)> },
}

#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub n: usize,
    pub round: u32,
    pub step: String,
    pub kind: String,
    pub situation: String,
    pub yours: String,
    /// the AI's choice (a label)
    pub ai: Option<String>,
    pub scores: Vec<(String, f64)>,
    pub scored: bool,
    pub note: String,
    /// your choice as the AI's label for it (the key into scores)
    pub yours_key: Option<String>,
    /// your hand and commander (if in the command zone) at the decision
    pub hand: Vec<String>,
    pub cmd: Option<String>,
    /// the second commander (Partner) if it is in the command zone
    pub cmd2: Option<String>,
    pub ai_plan: Option<(usize, String)>,
    pub n_choices: usize,
    /// a mulligan's one-line AI verdict with its reason
    pub ai_note: String,
}

impl Entry {
    fn score(&self, k: &str) -> Option<f64> {
        self.scores.iter().find(|(l, _)| l == k).map(|x| x.1)
    }

    /// for a saved game (the comparison already worked out; no copy of the game)
    pub fn saved(&self) -> Value {
        json!({
            "n": self.n, "round": self.round, "step": self.step, "kind": self.kind, "situation": self.situation,
            "yours": self.yours, "ai": self.ai, "scores": self.scores.iter().map(|(k, v)| json!([k, v])).collect::<Vec<_>>(),
            "scored": self.scored, "note": self.note, "yours_key": self.yours_key, "hand": self.hand, "cmd": self.cmd, "cmd2": self.cmd2,
            "ai_plan": self.ai_plan.as_ref().map(|(i, m)| json!([i, m])), "n_choices": self.n_choices, "ai_note": self.ai_note,
        })
    }

    pub fn from_saved(d: &Value) -> Result<Entry, String> {
        let bad = || "a saved game with a damaged comparison log".to_string();
        let s = |k: &str| d.get(k).and_then(Value::as_str).map(str::to_string);
        let strs = |k: &str| -> Vec<String> {
            d.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
        };
        Ok(Entry {
            n: d.get("n").and_then(Value::as_u64).ok_or_else(bad)? as usize,
            round: d.get("round").and_then(Value::as_u64).unwrap_or(0) as u32,
            step: s("step").unwrap_or_default(),
            kind: s("kind").ok_or_else(bad)?,
            situation: s("situation").unwrap_or_default(),
            yours: s("yours").unwrap_or_default(),
            ai: s("ai"),
            scores: d
                .get("scores")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|p| Some((p.get(0)?.as_str()?.to_string(), p.get(1)?.as_f64()?))).collect())
                .unwrap_or_default(),
            scored: d.get("scored").and_then(Value::as_bool).unwrap_or(false),
            note: s("note").unwrap_or_default(),
            yours_key: s("yours_key"),
            hand: strs("hand"),
            cmd: s("cmd"),
            cmd2: s("cmd2"),
            ai_plan: d.get("ai_plan").and_then(|p| Some((p.get(0)?.as_u64()? as usize, p.get(1)?.as_str()?.to_string()))),
            n_choices: d.get("n_choices").and_then(Value::as_u64).unwrap_or(0) as usize,
            ai_note: s("ai_note").unwrap_or_default(),
        })
    }

    /// the AI's choice as your answer at this decision, or None when it isn't a single action
    pub fn ai_answer(&self) -> Option<Answer> {
        let ai = self.ai.as_deref()?;
        if self.kind == "main" {
            if ai.starts_with("stop") {
                return Some(Answer::Action(json!({"do": "pass"})));
            }
            let name = ai.split(" -> ").next().unwrap_or("");
            if let Some(i) = self.hand.iter().position(|h| h == name) {
                return Some(Answer::Action(json!({"do": "cast", "card": i})));
            }
            if self.cmd.as_deref() == Some(name) || self.cmd2.as_deref() == Some(name) {
                return Some(Answer::Action(json!({"do": "cast", "zone": "cmd", "cmd": name})));
            }
            return None;
        }
        if self.kind == "attack" {
            match self.ai_plan.as_ref().map(|p| p.1.as_str()) {
                Some("none") => return Some(Answer::Picks(vec![])),
                Some("all") => return Some(Answer::Picks((0..self.n_choices).collect())),
                _ => {}
            }
        }
        None
    }

    pub fn ai_text(&self) -> Option<String> {
        if self.kind == "mulligan" {
            return Some(self.ai_note.clone());
        }
        let ai = self.ai.as_deref()?;
        if self.kind == "main" {
            if ai.starts_with("stop") {
                return Some("Pass (nothing more this phase)".into());
            }
            let (name, _, tgt) = match ai.split_once(" -> ") {
                Some((a, b)) => (a, " -> ", b),
                None => (ai, "", ""),
            };
            return Some(format!("Cast {name}{}", if tgt.is_empty() { String::new() } else { format!(" on {tgt}") }));
        }
        Some(ai.to_string())
    }

    pub fn as_dict(&self) -> Value {
        let step_text = match self.step.as_str() {
            "main1" => "Main 1".to_string(),
            "main2" => "Main 2".to_string(),
            "combat" => "Combat".to_string(),
            o => o.to_string(),
        };
        let yk = self.yours_key.as_deref();
        let score_yours = if self.scored { yk.and_then(|k| self.score(k)) } else { None };
        let score_ai = if self.scored { self.ai.as_deref().and_then(|k| self.score(k)) } else { None };
        let differs = if self.kind == "mulligan" {
            self.ai.as_deref() != yk
        } else {
            self.scored
            && self.ai.as_deref() != yk
            && yk.is_some_and(|k| self.score(k).is_some())
            && match (score_ai, score_yours) {
                (Some(a), Some(y)) => a - y >= SAME,
                _ => false,
            }
        };
        json!({
            "n": self.n, "round": self.round, "step": self.step, "kind": self.kind, "situation": self.situation,
            "key": self.yours_key, "ai_text": self.ai_text(), "can_try": self.scored && self.ai.is_some(),
            "yours": self.yours, "ai": self.ai, "score_yours": score_yours, "score_ai": score_ai, "scored": self.scored,
            "note": self.note, "scores": self.scores.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>(),
            "step_text": step_text, "differs": differs,
        })
    }
}

struct Slot {
    e: Entry,
    job: Option<Job>,
    running: bool,
}

impl Slot {
    fn pending(&self) -> bool {
        self.job.is_some() || self.running
    }
}

#[derive(Default)]
struct Inner {
    slots: Vec<Slot>,
}

/// One person's comparison log, with its worker threads.
#[derive(Default)]
pub struct Shadow {
    inner: Mutex<Inner>,
    cv: Condvar,
    stop: AtomicBool,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

fn situation(g: &Game, p: PlayerId) -> String {
    let opps: Vec<String> = g.opps(p).map(|q| format!("{} {}", g.player(q).name, g.player(q).life)).collect();
    let pl = g.player(p);
    format!(
        "{} life, {} in hand, {} lands, {} creatures; opponents: {}",
        pl.life,
        pl.hand.len(),
        pl.lands.len(),
        pl.perms.iter().filter(|&&m| g.is_creature(m)).count(),
        opps.join(", ")
    )
}

/// a copy of the real game g for the comparison: the person's floating mana counts as mana of any colour
fn snapshot(g: &Game, p: PlayerId) -> Game {
    let mut g2 = g.clone();
    g2.cur_cast = None;
    g2.resolving = 0;
    g2.trig_probe = false;
    g2.trig_current = None;
    g2.player_mut(p).floating.any += pool_total(g, p);
    g2.player_mut(p).pool = Default::default();
    g2.search_n = 0;
    g2
}

fn n_workers() -> usize {
    if cfg!(target_os = "ios") {
        return 2;
    }
    let cpus = std::thread::available_parallelism().map_or(2, |n| n.get());
    cpus.saturating_sub(1).clamp(1, 4)
}

impl Shadow {
    pub fn new() -> Shadow {
        Shadow::default()
    }

    // -------------------------------------------------------------- recording (game thread, before your answer applies)
    /// decision n: request req answered with ans. Copies the game if the decision is one to compare.
    pub fn record(self: &Arc<Self>, g: &Game, p: PlayerId, req: &Request, ans: &Answer, n: usize) {
        let slot = match &*req.kind {
            "priority" => {
                let Answer::Action(a) = ans else { return };
                let stack_empty = req.data.get("stack").and_then(Value::as_array).is_none_or(|s| s.is_empty());
                if g.active != Some(p) || !matches!(g.step, Step::Main1 | Step::Main2) || !stack_empty {
                    return;
                }
                let doing = a.get("do").and_then(Value::as_str).unwrap_or("");
                if doing == "tap" {
                    return; // getting mana ready: not a decision of its own
                }
                let label = main_label(g, p, a);
                let pl = g.player(p);
                let mut e = Entry {
                    n,
                    round: g.round,
                    step: step_name(g.step).into(),
                    kind: "main".into(),
                    situation: situation(g, p),
                    yours: yours_text(g, p, a, label.as_deref()),
                    hand: pl.hand.iter().map(|&c| g.db.get(c).name.to_string()).collect(),
                    cmd: pl.cmd_in_zone.then(|| g.db.get(pl.cmd).name.to_string()),
                    cmd2: pl.cmd2.filter(|_| pl.cmd2_in_zone).map(|c| g.db.get(c).name.to_string()),
                    ..Default::default()
                };
                match label {
                    None => {
                        e.note = "not compared: the AI weighs spells and passing here, not this".into();
                        Slot { e, job: None, running: false }
                    }
                    Some(l) => {
                        e.yours_key = Some(l.clone());
                        let job = Job::Main { snap: snapshot(g, p), sp: p, post: g.step == Step::Main2, label: l };
                        Slot { e, job: Some(job), running: false }
                    }
                }
            }
            "attack" => {
                let label = attack_label(g, p, req, ans);
                let mut e = Entry {
                    n,
                    round: g.round,
                    step: "combat".into(),
                    kind: "attack".into(),
                    situation: situation(g, p),
                    yours: attack_text(req, ans),
                    n_choices: req.choices.len(),
                    ..Default::default()
                };
                e.yours_key = None;
                Slot { e, job: Some(Job::Attack { snap: snapshot(g, p), sp: p, label }), running: false }
            }
            "mulligan" => {
                let (ai_keeps, why) = crate::advisor::mulligan::verdict(g, p, &req.prompt);
                let you_keep = matches!(ans, Answer::Pick(0) | Answer::Yes);
                let word = |k: bool| if k { "keep" } else { "mulligan" };
                let hand: Vec<String> = g.player(p).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
                let lands = g.player(p).hand.iter().filter(|&&c| g.db.get(c).land).count();
                let e = Entry {
                    n,
                    round: g.round,
                    step: "opening hand".into(),
                    kind: "mulligan".into(),
                    situation: format!("{} cards, {lands} lands", hand.len()),
                    yours: if you_keep { "Keep" } else { "Mulligan" }.into(),
                    ai: Some(word(ai_keeps).into()),
                    yours_key: Some(word(you_keep).into()),
                    ai_note: why,
                    note: "the AI's keep rule, not a playout".into(),
                    hand,
                    ..Default::default()
                };
                Slot { e, job: None, running: false }
            }
            _ => return,
        };
        self.inner.lock().unwrap().slots.push(slot);
        self.spawn_workers();
        self.cv.notify_all();
    }

    /// Undo: the decisions from n on are gone
    pub fn forget_from(&self, n: usize) {
        self.inner.lock().unwrap().slots.retain(|s| s.e.n < n);
    }

    /// the entries (finished or not), for the review and Try it
    pub fn entries(&self) -> Vec<Entry> {
        self.inner.lock().unwrap().slots.iter().map(|s| s.e.clone()).collect()
    }

    pub fn entry(&self, n: usize) -> Option<Entry> {
        self.inner.lock().unwrap().slots.iter().find(|s| s.e.n == n).map(|s| s.e.clone())
    }

    /// entries with their comparison worked out (unfinished ones don't save)
    pub fn saved(&self) -> Vec<Value> {
        self.inner.lock().unwrap().slots.iter().filter(|s| !s.pending()).map(|s| s.e.saved()).collect()
    }

    pub fn load(&self, entries: Vec<Entry>) {
        self.inner.lock().unwrap().slots = entries.into_iter().map(|e| Slot { e, job: None, running: false }).collect();
    }

    pub fn pending(&self) -> usize {
        self.inner.lock().unwrap().slots.iter().filter(|s| s.pending()).count()
    }

    pub fn review(&self) -> Vec<Value> {
        self.inner.lock().unwrap().slots.iter().map(|s| s.e.as_dict()).collect()
    }

    // -------------------------------------------------------------- working
    fn spawn_workers(self: &Arc<Self>) {
        let mut w = self.workers.lock().unwrap();
        if self.stop.load(Ordering::Relaxed) || !w.is_empty() {
            return;
        }
        for _ in 0..n_workers() {
            let me = self.clone();
            let h = std::thread::Builder::new()
                .name("practice-shadow".into())
                .stack_size(64 << 20)
                .spawn(move || me.work())
                .expect("spawn shadow worker");
            w.push(h);
        }
    }

    fn work(&self) {
        loop {
            let (n, job) = {
                let mut g = self.inner.lock().unwrap();
                loop {
                    if self.stop.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Some(s) = g.slots.iter_mut().find(|s| s.job.is_some()) {
                        s.running = true;
                        break (s.e.n, s.job.take().unwrap());
                    }
                    g = self.cv.wait_timeout(g, Duration::from_millis(200)).unwrap().0;
                }
            };
            let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_job(n, job)))
                .unwrap_or_else(|_| Done { note: Some("not compared: the comparison failed".into()), ..Done::default() });
            let mut g = self.inner.lock().unwrap();
            if let Some(s) = g.slots.iter_mut().find(|s| s.e.n == n && s.running) {
                out.apply(&mut s.e);
                s.running = false;
            }
            drop(g);
            self.cv.notify_all();
        }
    }

    /// wait until every comparison is worked out; `progress(done, total)` after each. Gives up if `abort` is set.
    pub fn finish(&self, progress: &mut dyn FnMut(usize, usize), abort: &AtomicBool) {
        let total = self.pending();
        let mut last = 0;
        loop {
            let mut g = self.inner.lock().unwrap();
            let left = g.slots.iter().filter(|s| s.pending()).count();
            let done = total.saturating_sub(left);
            if done != last {
                last = done;
                drop(g);
                progress(done, total);
                g = self.inner.lock().unwrap();
            }
            if left == 0 || abort.load(Ordering::Relaxed) {
                return;
            }
            let _ = self.cv.wait_timeout(g, Duration::from_millis(100)).unwrap();
        }
    }

    /// stop and join the worker threads (nothing is left running)
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cv.notify_all();
        let hs: Vec<_> = self.workers.lock().unwrap().drain(..).collect();
        for h in hs {
            let _ = h.join();
        }
    }

    pub fn workers_running(&self) -> usize {
        self.workers.lock().unwrap().iter().filter(|h| !h.is_finished()).count()
    }
}

#[derive(Default)]
struct Done {
    ai: Option<String>,
    scores: Option<Vec<(String, f64)>>,
    scored: Option<bool>,
    note: Option<String>,
    yours_key: Option<String>,
    ai_plan: Option<(usize, String)>,
}

impl Done {
    fn apply(self, e: &mut Entry) {
        if let Some(a) = self.ai {
            e.ai = Some(a);
        }
        if let Some(s) = self.scores {
            e.scores = s;
        }
        if let Some(s) = self.scored {
            e.scored = s;
        }
        if let Some(n) = self.note {
            e.note = n;
        }
        if let Some(k) = self.yours_key {
            e.yours_key = Some(k);
        }
        if self.ai_plan.is_some() {
            e.ai_plan = self.ai_plan;
        }
    }
}

fn run_job(n: usize, job: Job) -> Done {
    match job {
        Job::Main { mut snap, sp, post, label } => {
            let mut d = Done::default();
            let Ok(opts) = brain::main_options(&mut snap, sp, post) else {
                d.note = Some("not compared: the AI found no way to make this play from here".into());
                return d;
            };
            let mut real: Vec<usize> = (0..opts.len()).filter(|&i| opts[i].act.is_some()).collect();
            let Some(stop) = opts.iter().position(|o| o.act.is_none()) else {
                d.note = Some("not compared: the AI found no way to make this play from here".into());
                return d;
            };
            real.sort_by(|&a, &b| opts[b].utility.partial_cmp(&opts[a].utility).unwrap_or(std::cmp::Ordering::Equal));
            real.truncate(snap.settings.search.top_k);
            let mut cands = real.clone();
            cands.push(stop);
            let ai_set: Vec<String> = cands.iter().map(|&i| opts[i].label.clone()).collect();
            let mut labels = ai_set.clone();
            let mine = if label == "stop" { opts[stop].label.clone() } else { label.clone() };
            d.yours_key = Some(mine.clone());
            if !labels.contains(&mine) {
                if !opts.iter().any(|o| o.label == mine) {
                    d.note = Some("not compared: the AI found no way to make this play from here".into());
                    d.scored = Some(false);
                    return d;
                }
                labels.push(mine);
            }
            if labels.len() < 2 {
                d.ai = Some(labels[0].clone());
                d.note = Some("forced: only one option".into());
                return d;
            }
            let base = 0x5EED + n as u64;
            let rollouts = snap.settings.search.rollouts as u64;
            let mut tot = vec![0.0; labels.len()];
            for r in 0..rollouts {
                for (k, lb) in labels.iter().enumerate() {
                    tot[k] += search::playout_main(&mut snap, sp, post, lb, 0, base, r);
                }
            }
            let scores: Vec<(String, f64)> = labels.iter().zip(&tot).map(|(l, t)| (l.clone(), t / rollouts as f64)).collect();
            let sc = |l: &str| scores.iter().find(|x| x.0 == l).map_or(f64::NEG_INFINITY, |x| x.1);
            let mut best = &ai_set[0];
            for l in &ai_set {
                if sc(l) > sc(best) {
                    best = l;
                }
            }
            d.ai = Some(best.clone());
            d.scores = Some(scores);
            d.scored = Some(true);
            d
        }
        Job::Attack { mut snap, sp, label } => {
            let mut d = Done::default();
            let opps: Vec<PlayerId> = snap.opps(sp).collect();
            let Some(label) = label.filter(|_| !opps.is_empty()) else {
                d.note = Some("not compared".into());
                return d;
            };
            let mut cands: Vec<(PlayerId, &'static str)> = vec![];
            for &i in &opps {
                cands.push((i, "filtered"));
                cands.push((i, "all"));
            }
            cands.push((opps[0], "none"));
            let base = 0xA77 + n as u64;
            let rollouts = snap.settings.search.rollouts as u64;
            let mut tot = vec![0.0; cands.len()];
            for r in 0..rollouts {
                for (k, &c) in cands.iter().enumerate() {
                    tot[k] += search::playout_attack(&mut snap, sp, c, base, r);
                }
            }
            let name = |c: (PlayerId, &str)| attack_name(&snap, c);
            d.scores = Some(cands.iter().zip(&tot).map(|(&c, t)| (name(c), t / rollouts as f64)).collect());
            let mut b = 0;
            for k in 1..cands.len() {
                if tot[k] > tot[b] {
                    b = k;
                }
            }
            d.ai = Some(name(cands[b]));
            d.ai_plan = Some((cands[b].0.index(), cands[b].1.to_string()));
            let mode = intern(&label.1);
            let pid = PlayerId(label.0 as u8);
            d.yours_key = Some(if cands.contains(&(pid, mode)) { name((pid, mode)) } else { name((opps[0], mode)) });
            d.scored = Some(true);
            d
        }
    }
}

// ------------------------------------------------------------------ main phase
/// the AI's label for your play: a card's name, or 'stop' for passing; None for something it doesn't weigh
fn main_label(g: &Game, p: PlayerId, ans: &Value) -> Option<String> {
    match ans.get("do").and_then(Value::as_str)? {
        "pass" => Some("stop".into()),
        "cast" => {
            let pl = g.player(p);
            if ans.get("zone").and_then(Value::as_str) == Some("cmd") {
                let c = commander_cast(g, p, ans);
                return Some(g.db.get(c).name.to_string());
            }
            let i = ans.get("card").and_then(Value::as_u64)? as usize;
            let zone = if ans.get("zone").and_then(Value::as_str) == Some("gy") { &pl.gy } else { &pl.hand };
            zone.get(i).map(|&c| g.db.get(c).name.to_string())
        }
        _ => None,
    }
}

fn yours_text(g: &Game, p: PlayerId, ans: &Value, label: Option<&str>) -> String {
    let doing = ans.get("do").and_then(Value::as_str).unwrap_or("None");
    if doing == "pass" {
        return "Pass (nothing more this phase)".into();
    }
    if let Some(l) = label {
        return format!("Cast {l}");
    }
    let pl = g.player(p);
    match doing {
        "land" => {
            let zone = if ans.get("zone").and_then(Value::as_str) == Some("gy") { &pl.gy } else { &pl.hand };
            match ans.get("card").and_then(Value::as_u64).and_then(|i| zone.get(i as usize)) {
                Some(&c) => format!("Play {}", g.db.get(c).name),
                None => "Play a land".into(),
            }
        }
        "use" => {
            if ans.get("land").is_some() {
                return "Use a land ability".into();
            }
            match ans.get("perm").and_then(Value::as_u64).and_then(|i| pl.perms.get(i as usize)) {
                Some(&m) => format!("Use {}", g.perm(m).name),
                None => "Use an ability".into(),
            }
        }
        o => o.to_string(),
    }
}

// ------------------------------------------------------------------ attacks
/// your declaration as the look-ahead's plans see attacks: (defender index, 'none' | 'all' | 'filtered')
fn attack_label(g: &Game, p: PlayerId, req: &Request, ans: &Answer) -> Option<(usize, String)> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let first = opps.first()?.index();
    let picked: &[usize] = match ans {
        Answer::Picks(v) => v,
        _ => &[],
    };
    if picked.is_empty() {
        return Some((first, "none".into()));
    }
    let uniq: HashSet<_> = picked.iter().collect();
    let mode = if uniq.len() == req.choices.len() { "all" } else { "filtered" };
    Some((first, mode.into()))
}

fn attack_text(req: &Request, ans: &Answer) -> String {
    let picked: &[usize] = match ans {
        Answer::Picks(v) => v,
        _ => &[],
    };
    if picked.is_empty() {
        return "No attack".into();
    }
    let names: Vec<&str> = picked.iter().filter_map(|&i| req.choices.get(i)).map(|c| c.split(" (").next().unwrap_or("")).collect();
    format!("Attack with {}", names.join(", "))
}

fn attack_name(g: &Game, c: (PlayerId, &str)) -> String {
    let who = &g.player(c.0).name;
    match c.1 {
        "none" => "No attack".into(),
        "all" => format!("Attack {who} with everything"),
        _ => format!("Attack {who} (safe attackers)"),
    }
}

/// the commander a `cast from the command zone` answer means (the one named, else the first in the zone)
fn commander_cast(g: &Game, p: PlayerId, ans: &Value) -> sim_core::ids::CardId {
    let pl = g.player(p);
    let named = match ans.get("cmd") {
        Some(Value::Number(n)) => n.as_u64().and_then(|i| pl.commanders().nth(i as usize)),
        Some(Value::String(s)) => pl.commanders().find(|&c| g.db.get(c).name.eq_ignore_ascii_case(s)),
        _ => None,
    };
    named.or_else(|| pl.cmds_in_zone().next()).unwrap_or(pl.cmd)
}
