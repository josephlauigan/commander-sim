//! `Hub` (Python's `server.Hub`): the current game and its event history, seats by browser cookie, the two-player
//! lobby, Undo/Try it approval, saved games and the autosave. Shared by the request handlers as an `Arc<Hub>`; the
//! lock is never held across a wait on the session.

use super::api::{Info, SessionApi, SessionFactory};
use super::http::safe_join;
use super::lan::lan_addresses;
use crate::catalog::MY_DECKS;
use crate::images::Images;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub const AUTOSAVE: &str = "autosave.json";
/// an Undo or Try it the other person hasn't answered: a new one may replace it
const PROPOSAL_WAIT: Duration = Duration::from_secs(60);

pub struct Lobby {
    pub opts: Value,
    pub code: String,
    pub host: String,
    pub session: Option<Arc<dyn SessionApi>>,
    pub deck: String,
    pub partner: Option<String>,
    pub tier: Option<String>,
}

struct Proposal {
    id: u64,
    by: String,
    to: String,
    what: &'static str,
    n: Value,
    at: Instant,
    text: String,
}

#[derive(Default)]
pub struct State {
    session: Option<Arc<dyn SessionApi>>,
    /// the history of the current game, each event with its routing (see `sse::for_seat`)
    events: Vec<Value>,
    /// seat -> the request waiting for its answer: its event
    pending: HashMap<String, Value>,
    /// seat -> the latest view of the table from that seat
    views: HashMap<String, Value>,
    images: Value,
    tools: Map<String, Value>,
    /// browser cookie -> the seat it plays in this game
    tokens: HashMap<String, String>,
    lobby: Option<Lobby>,
    proposal: Option<Proposal>,
    next_proposal: u64,
    next_id: u64,
    game_no: u64,
}

pub struct Hub {
    pub lan: bool,
    pub app: bool,
    pub saves: PathBuf,
    pub images: Images,
    pub factory: Arc<dyn SessionFactory>,
    st: Mutex<State>,
    cond: Condvar,
    stopped: AtomicBool,
}

/// `str(x)` as Python prints a JSON value
fn pystr(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        other => other.to_string(),
    }
}

/// local time formatted with strftime
pub fn now(fmt: &str) -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        let f = std::ffi::CString::new(fmt).unwrap_or_default();
        let mut buf = [0u8; 64];
        let n = libc::strftime(buf.as_mut_ptr() as *mut libc::c_char, buf.len(), f.as_ptr(), &tm);
        String::from_utf8_lossy(&buf[..n]).into_owned()
    }
}

/// `n` random bytes (the operating system's source, else the std hasher's keys)
pub fn random_bytes(n: usize) -> Vec<u8> {
    use std::io::Read;
    let mut b = vec![0u8; n];
    if std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut b)).is_ok() {
        return b;
    }
    use std::hash::{BuildHasher, Hasher};
    for (i, x) in b.iter_mut().enumerate() {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_usize(i);
        h.write_u128(
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0),
        );
        *x = h.finish() as u8;
    }
    b
}

/// a new browser cookie value (url-safe, 22 characters)
pub fn new_token() -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    random_bytes(22).iter().map(|b| A[(*b & 63) as usize] as char).collect()
}

fn obj(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

impl Hub {
    pub fn new(lan: bool, app: bool, saves: PathBuf, images: Images, factory: Arc<dyn SessionFactory>) -> Arc<Hub> {
        let st = State { images: json!({}), next_proposal: 1, next_id: 1, ..Default::default() };
        Arc::new(Hub {
            lan,
            app,
            saves,
            images,
            factory,
            st: Mutex::new(st),
            cond: Condvar::new(),
            stopped: AtomicBool::new(false),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.st.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// stop the background threads (the server is shutting down)
    pub fn shutdown(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.quit();
    }

    // ------------------------------------------------------------------ games
    pub fn new_game(
        self: &Arc<Self>,
        opts: &Value,
        sess: Option<Arc<dyn SessionApi>>,
        tokens: HashMap<String, String>,
    ) -> Result<Arc<dyn SessionApi>, String> {
        self.quit();
        let sess = match sess {
            Some(s) => s,
            None => self.factory.new_session(opts)?,
        };
        let no;
        {
            let mut st = self.lock();
            st.game_no += 1;
            st.session = Some(sess.clone());
            st.events.clear();
            st.pending.clear();
            st.views.clear();
            st.images = json!({});
            let t = &opts["tools"];
            st.tools = ["hint", "undo", "compare"]
                .iter()
                .map(|k| (k.to_string(), Value::Bool(t.get(*k).map(|v| truthy(v)).unwrap_or(true))))
                .collect();
            st.tokens = tokens;
            st.proposal = None;
            no = st.game_no;
        }
        let (h, s) = (self.clone(), sess.clone());
        std::thread::Builder::new()
            .name("practice-pump".into())
            .spawn(move || h.pump(s, no))
            .map_err(|e| e.to_string())?;
        let (h, s, want) = (self.clone(), sess.clone(), opts.get("images").map(truthy).unwrap_or(true));
        std::thread::Builder::new()
            .name("practice-images".into())
            .spawn(move || h.load_images(s, no, want))
            .map_err(|e| e.to_string())?;
        Ok(sess)
    }

    /// the loading screen: card images for this game's decks, then the game starts
    fn load_images(self: &Arc<Self>, sess: Arc<dyn SessionApi>, no: u64, want_images: bool) {
        let mut found = Default::default();
        if want_images {
            let names = self.factory.card_names(&sess.info());
            let mut progress =
                |d: usize, t: usize| self.feed(no, &sess, json!({"kind": "loading", "done": d, "total": t}));
            match self.images.prepare_online(&names, &mut progress) {
                Ok(f) => found = f,
                Err(e) => {
                    // images are a nicety: the game starts without them, with the ones already on disk
                    found = self.images.cached(&names);
                    self.feed(no, &sess, json!({"kind": "log", "text": format!("(card images unavailable: {e})")}));
                }
            }
        }
        self.feed(no, &sess, json!({"kind": "images", "images": found}));
        if self.lock().game_no != no || self.stopped.load(Ordering::SeqCst) {
            return; // replaced or quit while loading
        }
        sess.start();
    }

    /// an event of the hub's own into the history of game `no`
    fn feed(&self, no: u64, sess: &Arc<dyn SessionApi>, ev: Value) {
        let mut st = self.lock();
        if st.game_no == no {
            self.add(&mut st, ev, sess);
        }
    }

    pub fn quit(&self) {
        let sess = {
            let mut st = self.lock();
            let s = st.session.take();
            st.pending.clear();
            st.lobby = None;
            st.proposal = None;
            self.cond.notify_all();
            s
        };
        if let Some(s) = sess {
            s.close(Duration::from_secs(30)); // its engine thread may be mid-playout
        }
    }

    // ------------------------------------------------------------------ two players
    /// a two-player game waiting for the second person; returns the join code
    pub fn open_lobby(&self, opts: &Value, host: &str, sess: Option<Arc<dyn SessionApi>>) -> Result<String, String> {
        if sess.is_none() {
            // the settings are checked now, not when the second person arrives
            let deck = opts["deck"].as_str().unwrap_or("");
            let partner = MY_DECKS.iter().find(|k| **k != deck).copied().unwrap_or("");
            self.factory.check_two(opts, partner)?;
        }
        self.quit();
        let code = format!("{:06}", u32::from_le_bytes(random_bytes(4).try_into().unwrap_or([0; 4])) % 1_000_000);
        let info = sess.as_ref().map(|s| s.info());
        let mut st = self.lock();
        st.lobby = Some(Lobby {
            opts: opts.clone(),
            code: code.clone(),
            host: host.to_string(),
            deck: info
                .as_ref()
                .map(|i| i.deck.clone())
                .unwrap_or_else(|| opts["deck"].as_str().unwrap_or("").to_string()),
            partner: info.as_ref().and_then(|i| i.partner.clone()),
            tier: opts["tier"].as_str().map(String::from).or(info.map(|i| i.tier)),
            session: sess,
        });
        self.cond.notify_all();
        Ok(code)
    }

    /// the second person joins the waiting game with its code; `None`, or why not
    pub fn join(self: &Arc<Self>, token: &str, code: &Value, deck: &Value) -> Option<String> {
        let lb = {
            let mut st = self.lock();
            let Some(lb) = st.lobby.as_ref() else {
                return Some("There is no game waiting for a second player.".into());
            };
            let given = match code {
                Value::Null => String::new(),
                v => pystr(v).trim().to_string(),
            };
            if given != lb.code {
                return Some("That code is not right: the host's screen shows it.".into());
            }
            let deck_s = deck.as_str();
            if let Some(p) = &lb.partner
                && deck_s != Some(p.as_str())
            {
                return Some(format!("This saved game goes on with the {p} deck."));
            }
            let Some(deck_s) = deck_s.filter(|d| MY_DECKS.contains(d)) else {
                return Some(format!("deck: one of {}", MY_DECKS.join(", ")));
            };
            if deck_s == lb.deck {
                return Some("The host plays that deck: pick another.".into());
            }
            if token == lb.host {
                return Some("This browser is the host's: join from the other computer.".into());
            }
            st.lobby.take()?
        };
        let deck = deck.as_str().unwrap_or("").to_string();
        let tokens: HashMap<String, String> =
            [(lb.host.clone(), lb.deck.clone()), (token.to_string(), deck.clone())].into();
        let r = match lb.session {
            None => {
                let mut o = obj(&lb.opts);
                o.insert("partner".into(), json!(deck));
                self.new_game(&Value::Object(o), None, tokens)
            }
            Some(s) => {
                let o =
                    json!({"tools": lb.opts["tools"], "images": lb.opts.get("images").cloned().unwrap_or(json!(true))});
                self.new_game(&o, Some(s), tokens)
            }
        };
        r.err()
    }

    /// the waiting game as a page sees it: the host also gets the code and the address to give
    fn lobby_info(&self, lb: Option<&Lobby>, local: bool) -> Value {
        let Some(lb) = lb else { return Value::Null };
        let mut d = json!({"deck": lb.deck, "partner": lb.partner, "tier": lb.tier, "loaded": lb.session.is_some()});
        if local {
            d["code"] = json!(lb.code);
            d["addresses"] = json!(if self.lan { lan_addresses() } else { vec![] });
        }
        d
    }

    /// the seat a browser plays in the current game (a page on the host computer without one plays the first seat),
    /// or None. A game the host has ended keeps its seats until the next starts
    pub fn seat_of(&self, token: &str, local: bool) -> Option<String> {
        let st = self.lock();
        if let Some(s) = st.tokens.get(token) {
            return Some(s.clone());
        }
        if local { st.session.as_ref().map(|s| s.info().deck) } else { None }
    }

    /// Undo (`what` "undo": seat's last n answers) or Try it ("tryit": seat's decision n). One person: done now.
    /// Two: the other person is asked first. Returns (why not, the name asked)
    pub fn request_rewind(&self, seat: &str, what: &'static str, n: &Value) -> (Option<String>, Option<String>) {
        let sess;
        {
            let mut st = self.lock();
            let Some(s) = st.session.clone() else { return (Some("There is no game running.".into()), None) };
            sess = s;
            if what == "undo" && !st.tools.get("undo").map(truthy).unwrap_or(true) {
                return (Some("Undo is switched off for this game.".into()), None);
            }
            let info = sess.info();
            let other = info.humans.iter().find(|k| k.as_str() != seat).cloned();
            if let Some(other) = other {
                let (what_text, also);
                if what == "undo" {
                    let nn = n.as_i64().unwrap_or(1);
                    let Some(to) = sess.undo_point(nn, seat) else {
                        return (Some("Nothing to undo yet.".into()), None);
                    };
                    let theirs = sess.answer_seats().iter().skip(to).filter(|k| **k == other).count();
                    what_text = if nn == 1 {
                        "to take back their last action".to_string()
                    } else {
                        format!("to take back their last {nn} actions")
                    };
                    also = if theirs > 0 {
                        format!(
                            " That also takes back your last {theirs} answer{}.",
                            if theirs != 1 { "s" } else { "" }
                        )
                    } else {
                        String::new()
                    };
                } else {
                    if !sess.has_try_entry(n, seat) {
                        return (Some("That decision has no AI choice to try.".into()), None);
                    }
                    what_text = "to try the AI's choice at one of their decisions".to_string();
                    also = " The game goes back to that decision for both of you.".to_string();
                }
                if let Some(p) = &st.proposal {
                    if p.at.elapsed() < PROPOSAL_WAIT {
                        return (
                            Some(format!("Still waiting for {} to answer the last request.", info.name_of(&p.to))),
                            None,
                        );
                    }
                    let id = p.id;
                    self.push(&mut st, json!({"kind": "proposal_done", "id": id}));
                }
                let name = info.name_of(seat);
                let id = st.next_proposal;
                st.next_proposal += 1;
                let text = format!("{name} asks {what_text}.{also}");
                st.proposal = Some(Proposal {
                    id,
                    by: seat.into(),
                    to: other.clone(),
                    what,
                    n: n.clone(),
                    at: Instant::now(),
                    text: text.clone(),
                });
                self.push(&mut st, json!({"kind": "proposal", "seat": other, "proposal": {"id": id, "text": text}}));
                return (None, Some(info.name_of(&other)));
            }
        }
        (self.rewind(&sess, what, n, seat), None)
    }

    fn rewind(&self, sess: &Arc<dyn SessionApi>, what: &str, n: &Value, seat: &str) -> Option<String> {
        if what == "undo" { sess.undo(n.as_i64().unwrap_or(1), seat) } else { sess.try_it(n, seat) }
    }

    /// the other person's answer to an Undo or Try it; `None`, or why not
    pub fn approve(&self, seat: Option<&str>, pid: &Value, yes: bool) -> Option<String> {
        let (sess, what, n, by);
        {
            let mut st = self.lock();
            let ok = matches!((&st.session, &st.proposal), (Some(_), Some(p)) if pid.as_u64() == Some(p.id) && Some(p.to.as_str()) == seat);
            if !ok {
                return Some("That request is no longer open.".into());
            }
            let p = st.proposal.take()?;
            let s = st.session.clone()?;
            self.push(&mut st, json!({"kind": "proposal_done", "id": p.id}));
            if !yes {
                let who = s.info().name_of(seat.unwrap_or(""));
                let text = format!("{who} said no to the {}.", if p.what == "undo" { "Undo" } else { "Try it" });
                self.push(&mut st, json!({"kind": "declined", "seat": p.by, "text": text}));
                return None;
            }
            (sess, what, n, by) = (s, p.what, p.n, p.by);
        }
        let why = self.rewind(&sess, what, &n, &by);
        if let Some(w) = &why {
            let mut st = self.lock();
            self.push(&mut st, json!({"kind": "declined", "seat": by, "text": w}));
        }
        why
    }

    // ------------------------------------------------------------------ events
    /// the session's events into the history, until a newer game replaces it
    fn pump(self: &Arc<Self>, sess: Arc<dyn SessionApi>, no: u64) {
        let mut cursor = 0;
        loop {
            let (c, evs) = sess.events_after(cursor, Duration::from_secs(1));
            cursor = c;
            if self.stopped.load(Ordering::SeqCst) {
                return;
            }
            for ev in evs {
                {
                    let mut st = self.lock();
                    if st.game_no != no || st.session.is_none() {
                        return; // a newer game replaced this one (or it was ended)
                    }
                    self.add(&mut st, ev.clone(), &sess);
                }
                let kind = ev["kind"].as_str().unwrap_or("");
                if kind == "request" && sess.info().partner.is_none() {
                    self.autosave(&sess, no);
                } else if kind == "over" && ev["how"] != "closed" {
                    self.forget_autosave();
                }
            }
            let st = self.lock();
            if st.game_no != no || st.session.is_none() {
                return;
            }
        }
    }

    /// (holding the lock) record one session event
    fn add(&self, st: &mut State, ev: Value, sess: &Arc<dyn SessionApi>) {
        let info = sess.info();
        let kind = ev["kind"].as_str().unwrap_or("").to_string();
        if kind == "reset" {
            // Undo: the game restarts; what came before is replayed
            st.events.clear();
            st.pending.clear();
            st.views.clear();
            st.proposal = None;
        }
        let mut views: Option<Map<String, Value>> = ev.get("views").and_then(|v| v.as_object().cloned());
        if views.is_none() && !ev["view"].is_null() {
            views = Some([(info.deck.clone(), ev["view"].clone())].into_iter().collect());
        }
        let mut out;
        if kind == "request" {
            let req = &ev["request"];
            let seat = ev["seat"].as_str().filter(|s| !s.is_empty()).map(String::from).unwrap_or(info.deck.clone());
            let mut data = obj(&req["data"]);
            if let Some(v) = data.remove("view") {
                views = Some([(seat.clone(), v)].into_iter().collect());
            }
            out = obj(&json!({"kind": "request", "request": {"kind": req["kind"], "prompt": req["prompt"],
                "choices": req["choices"], "data": data}, "seat": seat, "who": info.name_of(&seat)}));
        } else {
            out = obj(&ev);
            out.remove("view");
            out.remove("views");
            if kind == "images" {
                st.images = ev["images"].clone();
            }
        }
        if let Some(views) = views.filter(|v| !v.is_empty()) {
            for (k, v) in &views {
                if !v.is_null() {
                    st.views.insert(k.clone(), v.clone());
                }
            }
            out.insert(
                "views".into(),
                Value::Object(
                    views.keys().map(|k| (k.clone(), st.views.get(k).cloned().unwrap_or(Value::Null))).collect(),
                ),
            );
        }
        let is_request = kind == "request";
        let out = self.push(st, Value::Object(out));
        if is_request {
            st.pending.insert(out["seat"].as_str().unwrap_or("").to_string(), out);
        }
    }

    /// (holding the lock) one event into the history, numbered; returns it
    fn push(&self, st: &mut State, mut out: Value) -> Value {
        out["id"] = json!(st.next_id);
        st.next_id += 1;
        st.events.push(out.clone());
        self.cond.notify_all();
        out
    }

    pub fn answer(&self, seat: Option<&str>, rid: &Value, value: Value) -> Option<String> {
        let sess;
        let seat_s;
        {
            let mut st = self.lock();
            let Some(s) = st.session.clone() else { return Some("There is no game running.".into()) };
            let p = seat.and_then(|k| st.pending.get(k));
            if p.is_none_or(|p| p["id"] != *rid || rid.is_null()) {
                return Some("That decision has already been answered.".into());
            }
            seat_s = seat?.to_string();
            st.pending.remove(&seat_s);
            let who = s.info().name_of(&seat_s);
            self.push(&mut st, json!({"kind": "decided", "by": seat_s, "who": who})); // answered: no longer waiting
            sess = s;
        }
        sess.answer(&seat_s, value);
        None
    }

    pub fn undo(&self, n: i64, seat: Option<&str>) -> Option<String> {
        let sess = self.lock().session.clone();
        let Some(sess) = sess else { return Some("There is no game running.".into()) };
        let seat = seat.map(String::from).unwrap_or_else(|| sess.info().deck);
        self.request_rewind(&seat, "undo", &json!(n)).0
    }

    // ------------------------------------------------------------------ saved games
    /// the file's name, or why not
    pub fn save(&self) -> Result<String, String> {
        let (sess, tools) = {
            let st = self.lock();
            (st.session.clone(), st.tools.clone())
        };
        let Some(sess) = sess else { return Err("There is no game to save.".into()) };
        let info = sess.info();
        if !info.waiting && !info.finished {
            return Err("Save when the game is waiting on a decision (not while the AI players play).".into());
        }
        let mut data = sess.saved(None)?;
        data["tools"] = Value::Object(tools);
        data["saved_at"] = json!(now("%Y-%m-%d %H:%M"));
        let partner = info.partner.as_ref().map(|p| format!("+{p}")).unwrap_or_default();
        let name = format!("{}-{}{}-{}-{}.json", now("%Y%m%d-%H%M%S"), info.deck, partner, info.tier, info.seed);
        std::fs::create_dir_all(&self.saves).map_err(|e| e.to_string())?;
        std::fs::write(self.saves.join(&name), data.to_string()).map_err(|e| e.to_string())?;
        Ok(name)
    }

    /// the game, waiting on your decision, into autosave.json (a failure only skips this one)
    fn autosave(&self, sess: &Arc<dyn SessionApi>, no: u64) {
        let Ok(mut data) = sess.saved(Some(Duration::from_millis(500))) else { return };
        data["saved_at"] = json!(now("%Y-%m-%d %H:%M"));
        // The state lock is held through the write: a quit, a new game or a load (which take it to end this game)
        // either come before the check, so nothing is written, or after the rename, so their cleanup removes it.
        // (Writing after the check, unlocked, could put a finished game's file back after Quit removed it.)
        let st = self.lock();
        if st.game_no != no || st.session.is_none() {
            return;
        }
        data["tools"] = Value::Object(st.tools.clone());
        if std::fs::create_dir_all(&self.saves).is_err() {
            return;
        }
        let tmp = self.saves.join(format!("{AUTOSAVE}.tmp"));
        if std::fs::write(&tmp, data.to_string()).is_ok() {
            let _ = std::fs::rename(tmp, self.saves.join(AUTOSAVE));
        }
    }

    pub fn forget_autosave(&self) {
        let _ = std::fs::remove_file(self.saves.join(AUTOSAVE));
    }

    /// the game saved by itself, as the saves list shows a game, or null
    pub fn autosaved(&self) -> Value {
        self.saves_list(true).into_iter().next().unwrap_or(Value::Null)
    }

    pub fn saves_list(&self, autosave: bool) -> Vec<Value> {
        let mut names: Vec<String> = std::fs::read_dir(&self.saves)
            .map(|rd| rd.flatten().filter_map(|e| e.file_name().into_string().ok()).collect())
            .unwrap_or_default();
        names.sort();
        names.reverse();
        let mut out = vec![];
        for name in names {
            if !name.ends_with(".json") || (name == AUTOSAVE) != autosave {
                continue;
            }
            let Some(d) = std::fs::read_to_string(self.saves.join(&name))
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            else {
                continue;
            };
            out.push(
                json!({"name": name, "deck": d["deck"], "partner": d["partner"], "tier": d["tier"], "seed": d["seed"],
                "round": d["round"], "finished": d["finished"], "saved_at": d["saved_at"]}),
            );
        }
        out
    }

    /// `Ok(None)`, `Ok(Some(code))` for a two-player game (it waits for the second person), or the reason it can't
    pub fn load(self: &Arc<Self>, name: &Value, images: bool, host: &str) -> Result<Option<String>, String> {
        let no_save = || "No such saved game.".to_string();
        let path = safe_join(&self.saves, &pystr(name)).filter(|p| p.is_file()).ok_or_else(no_save)?;
        let load = || -> Result<(Value, Arc<dyn SessionApi>), String> {
            let data: Value = serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let tools = data
                .get("tools")
                .cloned()
                .filter(|t| t.as_object().is_some_and(|o| !o.is_empty()))
                .unwrap_or(json!({}));
            let compare = tools.get("compare").map(truthy).unwrap_or(true);
            Ok((tools, self.factory.load_session(&data, compare)?))
        };
        let (tools, sess) = load().map_err(|e| format!("That saved game can't be loaded: {e}"))?;
        let info = sess.info();
        if info.partner.is_some() {
            if !self.lan {
                return Err("A two-player game: start the server with --lan to play it with your friend.".into());
            }
            return self
                .open_lobby(&json!({"tools": tools, "images": images, "tier": info.tier}), host, Some(sess))
                .map(Some);
        }
        self.new_game(&json!({"tools": tools, "images": images}), Some(sess), HashMap::new())?;
        Ok(None)
    }

    pub fn review(&self, seat: Option<&str>) -> Result<Value, String> {
        let sess = self.lock().session.clone();
        let Some(sess) = sess else { return Err("There is no game.".into()) };
        let Some(seat) = seat else { return Err("You have no seat in this game.".into()) };
        sess.review(seat)
    }

    pub fn set_autopass(&self, seat: Option<&str>, mode: &Value) -> Option<String> {
        let st = self.lock();
        let Some(sess) = &st.session else { return Some("There is no game running.".into()) };
        let Some(seat) = seat else { return Some("You have no seat in this game.".into()) };
        sess.set_autopass(seat, mode)
    }

    pub fn hint(&self, seat: Option<&str>) -> Result<Value, String> {
        let sess = {
            let st = self.lock();
            let Some(sess) = st.session.clone() else { return Err("There is no game running.".into()) };
            if seat.is_none() {
                return Err("You have no seat in this game.".into());
            }
            if !st.tools.get("hint").map(truthy).unwrap_or(true) {
                return Err("Hint is switched off for this game.".into());
            }
            sess
        };
        sess.hint(seat.unwrap_or(""))
    }

    pub fn state(&self, seat: Option<&str>, local: bool) -> Value {
        let st = self.lock();
        let sess = st.session.as_ref();
        let game = match (sess, seat) {
            (Some(s), Some(seat)) => {
                let i: Info = s.info();
                json!({"deck": i.deck, "tier": i.tier, "seed": i.seed, "seats": i.seats, "profile": i.profile, "ai": i.ai,
                    "tools": st.tools, "you": seat, "humans": i.humans,
                    "names": i.names.iter().cloned().collect::<HashMap<_, _>>(), "autopass": s.autopass_of(seat)})
            }
            _ => Value::Null,
        };
        let p =
            st.proposal.as_ref().filter(|p| Some(p.to.as_str()) == seat).map(|p| json!({"id": p.id, "text": p.text}));
        json!({"game": game, "busy": sess.is_some() && seat.is_none(), "host": local, "lan": self.lan,
            "lobby": self.lobby_info(st.lobby.as_ref(), local), "view": seat.and_then(|k| st.views.get(k)).cloned(),
            "pending": seat.and_then(|k| st.pending.get(k)).cloned(), "images": st.images,
            "last_event": st.events.last().map(|e| e["id"].clone()).unwrap_or(json!(0)),
            "proposal": p})
    }

    /// events with id > since (waits up to `timeout` for one); empty on timeout
    pub fn events_after(&self, since: u64, timeout: Duration) -> Vec<Value> {
        let mut st = self.lock();
        if !st.events.last().is_some_and(|e| e["id"].as_u64().unwrap_or(0) > since)
            && !self.stopped.load(Ordering::SeqCst)
        {
            st = self.cond.wait_timeout(st, timeout).unwrap_or_else(|e| e.into_inner()).0;
        }
        st.events.iter().filter(|e| e["id"].as_u64().unwrap_or(0) > since).cloned().collect()
    }

    /// the seat's waiting request event, if any
    pub fn pending_of(&self, seat: &str) -> Option<Value> {
        self.lock().pending.get(seat).cloned()
    }

    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    /// wake the event streams (shutdown)
    pub fn wake(&self) {
        self.cond.notify_all();
    }
}

/// Python truthiness of a JSON value
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}
