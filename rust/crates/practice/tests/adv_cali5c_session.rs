// written by an independent reviewer
//! C5c adversarial tests at the session level (two people at a two-seat format, no AI seat). No deck is named: the Cali
//! deck is the deck of MY_DECKS whose own format has 2 seats, the guest is the first other deck the format admits.
//!
//! Attack list items covered here: (1) Undo / Try it from either seat at every decision of a short game with a policy the
//! author did not use (attack with everything, the LAST option everywhere, pass on every stack item, mulligan twice),
//! n larger than the seat's own answers, an answer racing a rewind, two rewinds racing; (2) tampered saves; (3) the
//! format's refusals of a deck; (6) no game thread left; (7) hint and review per seat.

use practice::session::{MY_DECKS, Options, Session};
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(240);

static SERIAL: Mutex<()> = Mutex::new(());

/// the tests of this file run one at a time (the thread counts and the cores are the whole machine's)
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn engine() -> Arc<Engine> {
    static E: OnceLock<Arc<Engine>> = OnceLock::new();
    E.get_or_init(|| {
        let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
        Arc::new(Engine::load(&data).expect("load the card data"))
    })
    .clone()
}

/// (the Cali deck, the guest's deck)
fn pair(e: &Engine) -> (String, String) {
    let deck = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck").to_string();
    let fmt = e.format_of(&deck).key.clone();
    let guest = MY_DECKS
        .iter()
        .find(|k| **k != deck && e.admits(k, &fmt).is_ok())
        .expect("another deck the format admits")
        .to_string();
    (deck, guest)
}

fn commander_deck(e: &Engine) -> String {
    MY_DECKS.iter().find(|k| e.format_of(k).seats == 4).expect("a 4-seat deck").to_string()
}

fn opts(e: &Arc<Engine>, deck: &str, guest: &str, seed: u64, rounds: u32, compare: bool) -> Options {
    let mut o = Options::new(e.clone(), deck, "");
    o.partner = Some(guest.into());
    o.seed = Some(seed);
    o.ai = "adaptive".into();
    o.compare = compare;
    o.views = true;
    o.max_rounds = rounds;
    o
}

fn seed_with_first(e: &Arc<Engine>, deck: &str, guest: &str, first: &str) -> u64 {
    (1..400u64)
        .find(|&s| Session::new(opts(e, deck, guest, s, 3, false)).unwrap().seats[0] == first)
        .expect("a seed with that seat order")
}

// ------------------------------------------------------------------------------------------------ the policies

/// The policy the author did not use: attack with every creature, take the LAST option of every choice, target, search,
/// block or yes/no, pass priority at once whenever something is on the stack, and (MullTwice) mulligan until two
/// mulligans each. In your own main phase: a land, tap the sources, then cast each card of the hand once a turn.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Style {
    Aggro,
    MullTwice,
}

#[derive(Clone)]
struct Pol {
    style: Style,
    played: Vec<(String, i64)>,
    tried: Vec<(i64, String, String)>,
    streak: usize,
}

impl Pol {
    fn new(style: Style) -> Pol {
        Pol { style, played: vec![], tried: vec![], streak: 0 }
    }

    fn pick(&mut self, seat: &str, req: &Value) -> Value {
        let a = self.pick_(seat, req);
        self.streak = if a["do"] == "pass" || !a.is_object() { 0 } else { self.streak + 1 };
        a
    }

    fn pick_(&mut self, seat: &str, req: &Value) -> Value {
        let n = req["choices"].as_array().map_or(0, |c| c.len());
        match req["kind"].as_str().unwrap_or("") {
            "mulligan" => {
                if self.style == Style::MullTwice && req["data"]["mulligans"].as_i64().unwrap_or(0) < 2 {
                    return json!("mulligan");
                }
                return json!("keep");
            }
            "attack" => return json!((0..n).collect::<Vec<_>>()),
            "priority" => {}
            _ if n > 0 => return json!(n - 1),
            _ => return json!(true),
        }
        let pass = json!({"do": "pass"});
        if req["data"]["stack"].as_array().is_some_and(|s| !s.is_empty()) || self.streak >= 25 {
            return pass;
        }
        let view = &req["data"]["view"];
        let Some(me) = view["players"].as_array().and_then(|ps| ps.iter().find(|p| p["you"] == true)) else {
            return pass;
        };
        if view["active"] == me["key"] && view["step"] == "main1" {
            let key = me["key"].as_str().unwrap_or("").to_string();
            let round = view["round"].as_i64().unwrap_or(0);
            let land = me["hand_land"]
                .as_array()
                .and_then(|a| a.iter().position(|x| x.as_bool().unwrap_or(false) || x.as_i64().unwrap_or(0) != 0));
            if let Some(l) = land
                && !self.played.contains(&(key.clone(), round))
            {
                self.played.push((key, round));
                return json!({"do": "land", "card": l});
            }
            if let Some(src) = me["mana_sources"].as_array().and_then(|a| a.first()) {
                return json!({"do": "tap", "source": src["id"]});
            }
            let names = me["hand"].as_array().cloned().unwrap_or_default();
            let lands = me["hand_land"].as_array().cloned().unwrap_or_default();
            for (i, nm) in names.iter().enumerate().rev() {
                let t = (round, key.clone(), nm.as_str().unwrap_or("").to_string());
                if lands.get(i) == Some(&json!(false)) && !self.tried.contains(&t) {
                    self.tried.push(t);
                    return json!({"do": "cast", "card": i});
                }
            }
        }
        let _ = seat;
        pass
    }
}

// ------------------------------------------------------------------------------------------------ the rig

/// a session driven by hand
struct Rig {
    s: Arc<Session>,
    cursor: u64,
    evs: Vec<Value>,
    /// the request waiting, or the `over` / `error` event
    at: Value,
    given: Vec<(String, Value)>,
    pol: Pol,
}

impl Rig {
    fn open(s: Arc<Session>, style: Style) -> Rig {
        s.start();
        let mut r = Rig { s, cursor: 0, evs: vec![], at: Value::Null, given: vec![], pol: Pol::new(style) };
        r.at = r.wait();
        r
    }

    fn wait(&mut self) -> Value {
        let end = Instant::now() + LIMIT;
        loop {
            assert!(
                Instant::now() < end,
                "no request in {LIMIT:?}; last events {:?}",
                &self.evs[self.evs.len().saturating_sub(4)..]
            );
            let (c, evs) = self.s.events_after(self.cursor, Duration::from_millis(100));
            self.cursor = c;
            for ev in evs {
                self.evs.push(ev.clone());
                if matches!(ev["kind"].as_str(), Some("request" | "over" | "error")) {
                    return ev;
                }
            }
        }
    }

    fn live(&self) -> bool {
        self.at["kind"] == "request"
    }

    fn seat(&self) -> String {
        self.at["seat"].as_str().unwrap().to_string()
    }

    fn give(&mut self, a: Value) {
        let seat = self.seat();
        self.given.push((seat.clone(), a.clone()));
        self.s.answer(&seat, &a);
        self.at = self.wait();
    }

    fn step(&mut self, script: Option<&[(String, Value)]>) {
        let seat = self.seat();
        let a = match script {
            Some(sc) => {
                let (s, a) = &sc[self.given.len()];
                assert_eq!(*s, seat, "answer {}: the script's seat is not the one the game asks", self.given.len());
                a.clone()
            }
            None => self.pol.pick(&seat, &self.at["request"]),
        };
        self.give(a);
    }

    fn run_to(&mut self, n: usize, script: Option<&[(String, Value)]>) {
        while self.given.len() < n && self.live() {
            self.step(script);
        }
    }

    fn run_out(&mut self) {
        while self.live() {
            self.step(None);
        }
    }

    fn warns(&self) -> Vec<String> {
        self.evs.iter().filter(|e| e["kind"] == "warn").map(|e| e["text"].as_str().unwrap_or("").to_string()).collect()
    }

    fn logs_after_reset(&self) -> Vec<String> {
        let from = self.evs.iter().rposition(|e| e["kind"] == "reset").map_or(0, |i| i + 1);
        self.evs[from..]
            .iter()
            .filter(|e| e["kind"] == "log")
            .map(|e| e["text"].as_str().unwrap().to_string())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
struct Point {
    logs: Vec<String>,
    request: Value,
    seat: Value,
    answer_seats: Value,
    saved: Value,
}

fn point(r: &Rig) -> Point {
    let mut saved = r.s.saved();
    let m = saved.as_object_mut().unwrap();
    m.remove("shadow");
    m.remove("partner_shadow");
    Point {
        logs: r.logs_after_reset(),
        request: r.at["request"].clone(),
        seat: r.at["seat"].clone(),
        answer_seats: saved["answer_seats"].clone(),
        saved,
    }
}

/// the whole game by the policy: the answers (seat, answer) it takes, and whether it ended
fn script_of(e: &Arc<Engine>, deck: &str, guest: &str, seed: u64, rounds: u32, style: Style) -> Vec<(String, Value)> {
    let mut r = Rig::open(Session::new(opts(e, deck, guest, seed, rounds, false)).unwrap(), style);
    r.run_out();
    assert_eq!(r.at["kind"], "over", "{}", r.at);
    let g = r.given.clone();
    r.s.close(Duration::from_secs(10));
    g
}

/// how many threads of the practice crate are alive (game, pump, images, shadow workers)
fn practice_threads() -> usize {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir("/proc/self/task") {
        for t in rd.flatten() {
            if let Ok(c) = std::fs::read_to_string(t.path().join("comm")) {
                let c = c.trim();
                if c.starts_with("practice-") && c != "practice-server" && c != "practice-conn" {
                    n += 1;
                }
            }
        }
    }
    n
}

/// close, even a session whose lock a panic poisoned
fn quiet_close(s: &Arc<Session>) {
    let s = s.clone();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || s.close(Duration::from_secs(10))));
}

fn wait_threads_gone(secs: u64) -> usize {
    let end = Instant::now() + Duration::from_secs(secs);
    loop {
        let n = practice_threads();
        if n == 0 || Instant::now() >= end {
            return n;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// run `f(i)` for every i in 0..n on up to 12 threads
fn par<F: Fn(usize) + Sync>(n: usize, f: F) {
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..12 {
            sc.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if i >= n {
                        return;
                    }
                    f(i);
                }
            });
        }
    });
}

// ------------------------------------------------------------------------------------------------ (1) Undo at every decision

struct Undone {
    /// the seat that undid was not the seat the game was waiting on (the other person's decision was waiting)
    other_was_waiting: bool,
    /// the Undo took back an answer of the other person too
    dropped_other: bool,
}

/// Undo `n` of `seat`'s answers after `k` answers of the script, and compare with a fresh session given the answers up to
/// the undo point (then `more` answers on from both with a fresh policy). None when the seat has fewer than n answers.
fn undo_case(
    e: &Arc<Engine>,
    deck: &str,
    guest: &str,
    seed: u64,
    rounds: u32,
    style: Style,
    script: &[(String, Value)],
    k: usize,
    seat: &str,
    n: i64,
) -> Option<Undone> {
    let mut t = Rig::open(Session::new(opts(e, deck, guest, seed, rounds, false)).unwrap(), style);
    t.run_to(k, Some(script));
    assert_eq!(t.given.len(), k);
    let waiting_seat = t.live().then(|| t.seat());
    let to = t.s.undo_point(n, seat);
    let Some(to) = to else {
        assert_eq!(t.s.undo(n, seat).as_deref(), Some("Nothing to undo yet."));
        t.s.close(Duration::from_secs(10));
        return None;
    };
    let dropped_other = t.given[to..].iter().any(|(s, _)| s != seat);
    assert!(t.s.undo(n, seat).is_none(), "seed {seed} k {k} {seat} n {n}");
    t.at = t.wait();
    t.given.truncate(to);

    let mut f = Rig::open(Session::new(opts(e, deck, guest, seed, rounds, false)).unwrap(), style);
    f.run_to(to, Some(script));
    let (a, b) = (point(&t), point(&f));
    let ctx = format!("seed {seed} {style:?} k {k} seat {seat} n {n} to {to}");
    assert_eq!(a.logs, b.logs, "{ctx}: logs");
    assert_eq!(a.request, b.request, "{ctx}: the waiting request");
    assert_eq!(a.seat, b.seat, "{ctx}: its seat");
    assert_eq!(a.answer_seats, b.answer_seats, "{ctx}: answer_seats");
    assert_eq!(a.saved, b.saved, "{ctx}: saved()");
    assert!(
        t.warns().is_empty() && f.warns().is_empty(),
        "{ctx}: the guard fired on an untouched game: {:?}",
        t.warns()
    );
    // on from there with a fresh policy: still the same game
    t.pol = Pol::new(style);
    f.pol = Pol::new(style);
    for _ in 0..8 {
        if !t.live() || !f.live() {
            break;
        }
        t.step(None);
        f.step(None);
    }
    let (a, b) = (point(&t), point(&f));
    assert_eq!(a.request, b.request, "{ctx}: after 8 more answers: the waiting request");
    assert_eq!(a.answer_seats, b.answer_seats, "{ctx}: after 8 more answers: answer_seats");
    assert_eq!(a.saved, b.saved, "{ctx}: after 8 more answers: saved()");
    assert!(t.warns().is_empty(), "{ctx}: the guard fired after more answers: {:?}", t.warns());
    t.s.close(Duration::from_secs(10));
    f.s.close(Duration::from_secs(10));
    Some(Undone { other_was_waiting: waiting_seat.is_some_and(|w| w != seat), dropped_other })
}

fn every_decision(style: Style, first_is_cali: bool, ns: &[i64]) {
    every_decision_cfg(style, first_is_cali, ns, 3, 1, false);
}

/// `rounds` rounds, every `stride`-th decision; `need_attacks`: the game must contain an attack with creatures
fn every_decision_cfg(style: Style, first_is_cali: bool, ns: &[i64], rounds: u32, stride: usize, need_attacks: bool) {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, if first_is_cali { &deck } else { &guest });
    let script = script_of(&e, &deck, &guest, seed, rounds, style);
    let len = script.len();
    assert!(len > 6, "a game of {len} answers");
    let attacks = script.iter().filter(|a| a.1.as_array().is_some_and(|x| !x.is_empty())).count();
    eprintln!("RECORD game of {len} answers at {rounds} rounds, seed {seed}: {attacks} attacks with creatures");
    if need_attacks {
        assert!(attacks > 0, "the game never attacked; pick more rounds");
    }
    let seats = [deck.clone(), guest.clone()];
    let cases: Vec<(usize, String, i64)> = (0..=len)
        .filter(|k| k % stride == 0 || *k == len)
        .flat_map(|k| seats.iter().flat_map(move |s| ns.iter().map(move |n| (k, s.clone(), *n))).collect::<Vec<_>>())
        .collect();
    let stats = Mutex::new((0usize, 0usize, 0usize));
    par(cases.len(), |i| {
        let (k, seat, n) = &cases[i];
        let r = undo_case(&e, &deck, &guest, seed, rounds, style, &script, *k, seat, *n);
        let mut s = stats.lock().unwrap();
        if let Some(u) = r {
            s.0 += 1;
            s.1 += u.other_was_waiting as usize;
            s.2 += u.dropped_other as usize;
        }
    });
    let (done, other_waiting, dropped) = *stats.lock().unwrap();
    eprintln!(
        "RECORD {style:?}: seed {seed}, Cali deck first: {first_is_cali}, game of {len} answers, n in {ns:?}: {done} Undo cases equal a fresh session; in {other_waiting} the OTHER person's decision was waiting; in {dropped} the undo also took back the other person's answers"
    );
    assert!(done > len / stride, "too few cases ran: {done}");
    assert!(
        other_waiting > 0 && dropped > 0,
        "the cases must include the other person's decision waiting and cross-seat drops"
    );
}

#[test]
fn undo_at_every_decision_aggro_policy_cali_deck_first() {
    every_decision(Style::Aggro, true, &[1, 2, 3]);
}

#[test]
fn undo_at_every_decision_aggro_policy_guest_first() {
    every_decision(Style::Aggro, false, &[1, 2, 3]);
}

/// longer games, so that creatures exist and attack: every 3rd decision of six rounds
#[test]
fn undo_over_six_rounds_cali_deck_first() {
    every_decision_cfg(Style::Aggro, true, &[1, 2], 6, 3, false);
}

#[test]
fn undo_over_six_rounds_guest_first() {
    every_decision_cfg(Style::Aggro, false, &[1, 2], 6, 3, false);
}

#[test]
fn undo_at_every_decision_when_both_mulligan_twice_cali_deck_first() {
    every_decision(Style::MullTwice, true, &[1, 2, 3, 4]);
}

#[test]
fn undo_at_every_decision_when_both_mulligan_twice_guest_first() {
    every_decision(Style::MullTwice, false, &[1, 2, 3, 4]);
}

// ------------------------------------------------------------------------------------------------ Try it

/// Try it at the decision n of `seat` (compare on) from the end of the game: equals a fresh session given the answers up
/// to n and then the AI's answer
fn try_case(
    e: &Arc<Engine>,
    deck: &str,
    guest: &str,
    seed: u64,
    style: Style,
    script: &[(String, Value)],
    seat: &str,
    n: usize,
) -> bool {
    let mut t = Rig::open(Session::new(opts(e, deck, guest, seed, 3, true)).unwrap(), style);
    t.run_to(script.len(), Some(script));
    assert_eq!(t.at["kind"], "over", "{}", t.at);
    let Some(en) = t.s.try_entry(&json!(n), seat) else {
        t.s.close(Duration::from_secs(10));
        return false;
    };
    let Some(then) = en.ai_answer() else {
        t.s.close(Duration::from_secs(10));
        return false;
    };
    let then = practice::session::human::answer_to_json(&then);
    assert!(t.s.try_it(&json!(n), seat).is_none(), "seed {seed} seat {seat} n {n}");
    t.at = t.wait();
    t.given.truncate(n);
    t.given.push((seat.to_string(), then.clone()));
    let reset = t.evs.iter().rev().find(|e| e["kind"] == "reset").unwrap().clone();
    assert_eq!(reset["tryit"]["by"], json!(seat));
    let mut f = Rig::open(Session::new(opts(e, deck, guest, seed, 3, true)).unwrap(), style);
    let mut sc: Vec<(String, Value)> = script[..n].to_vec();
    sc.push((seat.to_string(), then));
    f.run_to(n + 1, Some(&sc));
    let (a, b) = (point(&t), point(&f));
    let ctx = format!("seed {seed} {style:?} Try it by {seat} at decision {n}");
    assert_eq!(a.logs, b.logs, "{ctx}: logs");
    assert_eq!(a.request, b.request, "{ctx}: request");
    assert_eq!(a.seat, b.seat, "{ctx}: seat");
    assert_eq!(a.answer_seats, b.answer_seats, "{ctx}: answer_seats");
    assert_eq!(a.answer_seats.as_array().unwrap().len(), n + 1, "{ctx}");
    assert!(t.warns().is_empty() && f.warns().is_empty(), "{ctx}: guard fired: {:?}", t.warns());
    t.s.close(Duration::from_secs(10));
    f.s.close(Duration::from_secs(10));
    true
}

/// The decisions (index, seat) of the reference game of `seed` that have a one-action AI choice, with its answers.
fn try_choices(
    e: &Arc<Engine>,
    deck: &str,
    guest: &str,
    seed: u64,
    style: Style,
) -> (Vec<(String, Value)>, Vec<(usize, String)>) {
    let script = script_of(e, deck, guest, seed, 3, style);
    // which decisions have an AI choice at all: one reference run
    let mut r = Rig::open(Session::new(opts(e, deck, guest, seed, 3, true)).unwrap(), style);
    r.run_to(script.len(), Some(&script));
    assert_eq!(r.at["kind"], "over");
    let mut cases = vec![];
    for (n, (seat, _)) in script.iter().enumerate() {
        if r.s.try_entry(&json!(n), seat).is_some_and(|en| en.ai_answer().is_some()) {
            cases.push((n, seat.clone()));
        }
    }
    r.s.close(Duration::from_secs(10));
    (script, cases)
}

fn try_everywhere(first_is_cali: bool) {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let first = if first_is_cali { &deck } else { &guest };
    let style = Style::Aggro;
    // The reference game depends on the Cali deck's list (and the guest's), so a deck change can leave a game with
    // no one-action AI choice for one seat. Search upward from the first seed with this seat order, deterministically
    // and with a bound, for a reference game where each seat has one; the checks on that game are unchanged.
    const SEARCH: usize = 12;
    let mut tried = vec![];
    let mut found = None;
    for seed in (1..400u64).filter(|&s| Session::new(opts(&e, &deck, &guest, s, 3, false)).unwrap().seats[0] == *first)
    {
        if tried.len() == SEARCH {
            break;
        }
        let (script, cases) = try_choices(&e, &deck, &guest, seed, style);
        let by_seat = |s: &str| cases.iter().filter(|c| c.1 == s).count();
        eprintln!(
            "RECORD Try it: seed {seed}, Cali deck first {first_is_cali}, game of {} answers: {} decisions have a one-action AI choice ({} the Cali deck's, {} the guest's)",
            script.len(),
            cases.len(),
            by_seat(&deck),
            by_seat(&guest)
        );
        tried.push(seed);
        if by_seat(&deck) > 0 && by_seat(&guest) > 0 {
            found = Some((seed, script, cases));
            break;
        }
    }
    let Some((seed, script, cases)) = found else {
        panic!("need an AI choice for each seat: seeds tried {tried:?} (Cali deck first {first_is_cali}) have none");
    };
    let ok = std::sync::atomic::AtomicUsize::new(0);
    par(cases.len(), |i| {
        let (n, seat) = &cases[i];
        if try_case(&e, &deck, &guest, seed, style, &script, seat, *n) {
            ok.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    });
    let ok = ok.load(std::sync::atomic::Ordering::SeqCst);
    eprintln!("RECORD Try it: {ok} of {} cases equal a fresh session", cases.len());
    assert_eq!(ok, cases.len());
}

#[test]
fn try_it_at_every_decision_with_an_ai_choice_cali_deck_first() {
    try_everywhere(true);
}

#[test]
fn try_it_at_every_decision_with_an_ai_choice_guest_first() {
    try_everywhere(false);
}

/// Try it from the seat that is NOT the one the game waits on, in the middle of the game (not from the end)
#[test]
fn try_it_mid_game_while_the_other_persons_decision_waits() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut done = 0;
    for first_is_cali in [true, false] {
        let seed = seed_with_first(&e, &deck, &guest, if first_is_cali { &deck } else { &guest });
        let script = script_of(&e, &deck, &guest, seed, 3, Style::Aggro);
        let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, true)).unwrap(), Style::Aggro);
        let mid = script.len() * 2 / 3;
        t.run_to(mid, Some(&script));
        assert!(t.live());
        let waiting = t.seat();
        let other = if waiting == deck { guest.clone() } else { deck.clone() };
        // wait for the other person's earlier decisions to be compared (workers finish them on their own time)
        let end = Instant::now() + Duration::from_secs(120);
        let mut hit = None;
        while hit.is_none() && Instant::now() < end {
            hit = (0..mid).find(|&n| {
                script[n].0 == other && t.s.try_entry(&json!(n), &other).is_some_and(|en| en.ai_answer().is_some())
            });
            if hit.is_none() {
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        let Some(n) = hit else {
            eprintln!("RECORD no compared decision of the other person yet at answer {mid} (seed {seed}); skipped");
            t.s.close(Duration::from_secs(10));
            continue;
        };
        let then =
            practice::session::human::answer_to_json(&t.s.try_entry(&json!(n), &other).unwrap().ai_answer().unwrap());
        assert!(t.s.try_it(&json!(n), &other).is_none());
        t.at = t.wait();
        let mut f = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, true)).unwrap(), Style::Aggro);
        let mut sc: Vec<(String, Value)> = script[..n].to_vec();
        sc.push((other.clone(), then));
        f.run_to(n + 1, Some(&sc));
        let (a, b) = (point(&t), point(&f));
        assert_eq!(a.request, b.request, "seed {seed} Try it by {other} at {n} while {waiting} waited");
        assert_eq!(a.answer_seats, b.answer_seats);
        assert!(t.warns().is_empty(), "{:?}", t.warns());
        done += 1;
        t.s.close(Duration::from_secs(10));
        f.s.close(Duration::from_secs(10));
    }
    eprintln!("RECORD Try it mid-game from the seat that was not waiting: {done} cases");
}

// ------------------------------------------------------------------------------------------------ n larger than the seat's answers

#[test]
fn undo_and_try_it_with_nonsense_counts_change_nothing() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &deck);
    let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, true)).unwrap(), Style::Aggro);
    for stage in 0..3 {
        // stage 0: no answers at all; 1: a few; 2: many
        if stage == 1 {
            t.run_to(6, None);
        }
        if stage == 2 {
            t.run_to(24, None);
        }
        assert!(t.live());
        for seat in [&deck, &guest, &"nobody".to_string(), &String::new()] {
            let mine = t.given.iter().filter(|g| &g.0 == seat).count() as i64;
            let before = (t.s.answers_len(), point(&t).request.clone(), t.evs.len());
            for n in [mine + 1, mine + 1000, i64::MAX, i64::MAX - 1, 0, -1, i64::MIN] {
                assert_eq!(t.s.undo_point(n, seat), None, "stage {stage} {seat} n {n}");
                assert_eq!(t.s.undo(n, seat).as_deref(), Some("Nothing to undo yet."), "stage {stage} {seat} n {n}");
            }
            assert_eq!((t.s.answers_len(), point(&t).request.clone()), (before.0, before.1));
            // no reset event came
            let (_, late) = t.s.events_after(t.cursor, Duration::from_millis(50));
            assert!(late.iter().all(|e| e["kind"] != "reset"), "{late:?}");
        }
        for n in [json!(99999), json!(-1), json!(null), json!("x"), json!(1.5), json!(u64::MAX), json!([1]), json!({})]
        {
            for seat in [&deck, &guest] {
                if let Some(why) = t.s.try_it(&n, seat) {
                    assert_eq!(why, "That decision has no AI choice to try.", "{n}");
                } else {
                    // allowed only when that decision really has an AI choice: then the game has been rewound: stop the stage
                    panic!("try_it({n}) was accepted at stage {stage} before any comparison could exist");
                }
            }
        }
    }
    // and the game is still playable
    t.run_to(40, None);
    assert!(t.warns().is_empty());
    t.s.close(Duration::from_secs(10));
}

// ------------------------------------------------------------------------------------------------ (1) races

/// wait until the game is waiting again after a reset and nothing new arrives for a moment; panics on a hang
fn settle(t: &mut Rig, what: &str) {
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        assert!(
            Instant::now() < end,
            "HANG: {what}: the session never asked again; last events {:?}",
            &t.evs[t.evs.len().saturating_sub(4)..]
        );
        let (c, evs) = t.s.events_after(t.cursor, Duration::from_millis(300));
        t.cursor = c;
        let quiet = evs.is_empty();
        t.evs.extend(evs);
        if quiet {
            let reset = t.evs.iter().rposition(|e| e["kind"] == "reset");
            let last = t.evs.iter().rposition(|e| matches!(e["kind"].as_str(), Some("request" | "over" | "error")));
            if let Some(l) = last
                && reset.is_none_or(|r| l > r)
            {
                t.at = t.evs[l].clone();
                return;
            }
        }
    }
}

#[test]
fn an_answer_racing_an_undo_never_hangs_panics_or_sticks() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut cases = 0;
    let mut stale_taken = 0;
    for first_is_cali in [true, false] {
        let seed = seed_with_first(&e, &deck, &guest, if first_is_cali { &deck } else { &guest });
        for k in [2usize, 3, 4, 5, 8, 13, 20] {
            for undoer_is_waiting in [true, false] {
                for _round in 0..4 {
                    let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, false)).unwrap(), Style::Aggro);
                    t.run_to(k, None);
                    if !t.live() {
                        t.s.close(Duration::from_secs(10));
                        continue;
                    }
                    let waiting = t.seat();
                    let other = if waiting == deck { guest.clone() } else { deck.clone() };
                    let undoer = if undoer_is_waiting { waiting.clone() } else { other };
                    let n = 1;
                    if t.s.undo_point(n, &undoer).is_none() {
                        t.s.close(Duration::from_secs(10));
                        continue;
                    }
                    let a = t.pol.pick(&waiting, &t.at["request"]);
                    let before_len = t.s.answers_len();
                    let s1 = t.s.clone();
                    let s2 = t.s.clone();
                    let bar = Arc::new(std::sync::Barrier::new(2));
                    let (b1, b2) = (bar.clone(), bar.clone());
                    let w2 = waiting.clone();
                    let u2 = undoer.clone();
                    let h1 = std::thread::spawn(move || {
                        b1.wait();
                        s1.answer(&w2, &a);
                    });
                    let h2 = std::thread::spawn(move || {
                        b2.wait();
                        s2.undo(n, &u2)
                    });
                    let (tx, rx) = channel();
                    std::thread::spawn(move || {
                        let _ = h1.join();
                        let r = h2.join();
                        let _ = tx.send(r.is_ok());
                    });
                    let joined = rx
                        .recv_timeout(Duration::from_secs(90))
                        .unwrap_or_else(|_| panic!("HANG: answer vs undo (seed {seed} k {k} undoer {undoer})"));
                    assert!(joined, "a thread PANICKED: answer vs undo (seed {seed} k {k} undoer {undoer})");
                    settle(&mut t, &format!("after answer vs undo (seed {seed} k {k})"));
                    cases += 1;
                    // the session is usable: the lock is not poisoned, the books agree, and play goes on
                    let s = t.s.clone();
                    let saved =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.saved())).unwrap_or_else(|_| {
                            panic!("the session lock is POISONED after answer vs undo (seed {seed} k {k})")
                        });
                    assert_eq!(
                        saved["answers"].as_array().unwrap().len(),
                        saved["answer_seats"].as_array().unwrap().len()
                    );
                    if t.s.answers_len() > before_len {
                        stale_taken += 1;
                    }
                    t.pol = Pol::new(Style::Aggro);
                    let at0 = t.s.answers_len();
                    for _ in 0..6 {
                        if !t.live() {
                            break;
                        }
                        let seat = t.seat();
                        let a = t.pol.pick(&seat, &t.at["request"]);
                        t.s.answer(&seat, &a);
                        settle_after_answer(&mut t, &format!("seed {seed} k {k}"));
                    }
                    assert!(
                        t.s.answers_len() > at0 || !t.live(),
                        "STUCK: six answers after the race moved nothing (seed {seed} k {k})"
                    );
                    t.s.close(Duration::from_secs(10));
                    let left = wait_threads_gone(10);
                    assert_eq!(
                        left, 0,
                        "{left} practice thread(s) left after closing (seed {seed} k {k}, undoer {undoer}): a second game thread?"
                    );
                }
            }
        }
    }
    eprintln!(
        "RECORD answer vs undo: {cases} races, none hung, panicked or stuck; the answer was counted before the rewind in {stale_taken}"
    );
    assert!(cases >= 20);
}

/// after an answer: wait for the next request or the end (the answer count moved)
fn settle_after_answer(t: &mut Rig, what: &str) {
    let end = Instant::now() + Duration::from_secs(60);
    let was = t.at["id"].clone();
    let n0 = t.evs.len();
    loop {
        assert!(Instant::now() < end, "HANG: {what}: no new request after an answer");
        let (c, evs) = t.s.events_after(t.cursor, Duration::from_millis(200));
        t.cursor = c;
        for ev in evs {
            t.evs.push(ev.clone());
            if matches!(ev["kind"].as_str(), Some("request" | "over" | "error")) {
                t.at = ev;
                return;
            }
        }
        let _ = (&was, n0);
    }
}

#[test]
fn two_undos_racing_never_poison_the_session_or_leave_two_game_threads() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &deck);
    let mut poisoned = 0;
    let mut total = 0;
    let mut panics = vec![];
    for k in [6usize, 9, 12, 16, 20] {
        for _round in 0..4 {
            let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, false)).unwrap(), Style::Aggro);
            t.run_to(k, None);
            if !t.live() {
                t.s.close(Duration::from_secs(10));
                continue;
            }
            let bar = Arc::new(std::sync::Barrier::new(2));
            let mut hs = vec![];
            for (seat, n) in [(deck.clone(), 1i64), (guest.clone(), 1i64)] {
                let (s, b) = (t.s.clone(), bar.clone());
                hs.push(std::thread::spawn(move || {
                    b.wait();
                    s.undo(n, &seat)
                }));
            }
            let (tx, rx) = channel();
            std::thread::spawn(move || {
                let r: Vec<bool> = hs.into_iter().map(|h| h.join().is_ok()).collect();
                let _ = tx.send(r);
            });
            let oks = rx.recv_timeout(Duration::from_secs(90)).unwrap_or_else(|_| panic!("HANG: two undos (k {k})"));
            total += 1;
            if oks.iter().any(|ok| !ok) {
                panics.push(format!("k {k}: a thread panicked inside undo"));
            }
            let s = t.s.clone();
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.saved())).is_err() {
                poisoned += 1;
            } else {
                settle(&mut t, &format!("after two undos (k {k})"));
            }
            quiet_close(&t.s);
            let left = wait_threads_gone(10);
            if left != 0 {
                panics.push(format!("k {k}: {left} practice thread(s) left after close"));
            }
        }
    }
    eprintln!(
        "RECORD two undos racing: {total} races, {poisoned} left the session lock poisoned, problems: {panics:?}"
    );
    assert!(poisoned == 0 && panics.is_empty(), "two Undos racing: {poisoned} poisoned of {total}; {panics:?}");
}

/// L13 (known, pre-existing): an answer for a decision that a rewind has dropped can reach the NEW game's next decision
/// of that seat. This only records what happens (it asserts nothing but liveness): after an Undo, the answer meant for
/// the old decision is sent, and the game is compared with a fresh session given the same answers
#[test]
fn a_stale_answer_after_an_undo_is_recorded_not_asserted() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &deck);
    let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, false)).unwrap(), Style::Aggro);
    t.run_to(10, None);
    let waiting = t.seat();
    let u = if waiting == deck { guest.clone() } else { deck.clone() };
    let Some(to) = t.s.undo_point(1, &u) else { return };
    assert!(t.s.undo(1, &u).is_none());
    t.s.answer(&waiting, &json!(true)); // the page's answer for the old decision
    t.at = t.wait();
    let script: Vec<(String, Value)> = t.given[..to].to_vec();
    let mut f = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, false)).unwrap(), Style::Aggro);
    f.run_to(to, Some(&script));
    let same = point(&t).request == point(&f).request;
    eprintln!(
        "RECORD L13 stale answer after Undo (to {to}): the game is at {} answers, a fresh game at {}; the same request: {same}",
        t.s.answers_len(),
        f.s.answers_len()
    );
    assert!(t.live() || t.at["kind"] == "over");
    t.s.close(Duration::from_secs(10));
    f.s.close(Duration::from_secs(10));
}

// ------------------------------------------------------------------------------------------------ (2) tampered saves

/// a two-person save at `k` answers (waiting), the Cali deck first or not
fn a_save(e: &Arc<Engine>, first_is_cali: bool, k: usize) -> (Value, String, String) {
    let (deck, guest) = pair(e);
    let seed = seed_with_first(e, &deck, &guest, if first_is_cali { &deck } else { &guest });
    let mut t = Rig::open(Session::new(opts(e, &deck, &guest, seed, 3, false)).unwrap(), Style::Aggro);
    t.run_to(k, None);
    assert!(t.live());
    let v = t.s.saved();
    t.s.close(Duration::from_secs(10));
    (v, deck, guest)
}

fn load(e: &Arc<Engine>, v: &Value) -> Result<Arc<Session>, String> {
    Session::load(e.clone(), v, true, false)
}

fn refused(e: &Arc<Engine>, v: &Value) -> String {
    match load(e, v) {
        Ok(s) => {
            s.close(Duration::from_secs(5));
            "<loaded>".into()
        }
        Err(m) => m,
    }
}

#[test]
fn tampered_answer_seats_are_refused_or_stopped_never_followed_blindly() {
    let _g = serial();
    let e = engine();
    let (v, deck, guest) = a_save(&e, true, 10);
    let seats = v["answer_seats"].as_array().unwrap().clone();
    assert_eq!(seats.len(), 10);
    let with = |x: Value| {
        let mut c = v.clone();
        c["answer_seats"] = x;
        c
    };
    let damaged = "a damaged saved game";
    let cat = |a: &[Value], b: &[Value]| -> Vec<Value> { a.iter().chain(b.iter()).cloned().collect() };
    let mut report = vec![];
    // shapes that are not the right list
    for (name, x) in [
        ("null", json!(null)),
        ("a string", json!("x")),
        ("a number", json!(5)),
        ("true", json!(true)),
        ("an object", json!({})),
        ("empty", json!([])),
        ("one short", json!(seats[..9].to_vec())),
        ("one long", json!(cat(&seats, &[json!(deck)]))),
        ("unknown key", json!(cat(&[json!("nobody")], &seats[1..]))),
        ("a number inside", json!(cat(&[json!(1)], &seats[1..]))),
        ("null inside", json!(cat(&[json!(null)], &seats[1..]))),
        ("empty string inside", json!(cat(&[json!("")], &seats[1..]))),
        ("a deck of neither person", json!(cat(&seats[..9], &[json!("zzz")]))),
        ("a list inside", json!(cat(&[json!([])], &seats[1..]))),
    ] {
        let m = refused(&e, &with(x));
        report.push(format!("{name}: {m}"));
        assert_eq!(m, damaged, "answer_seats {name}");
    }
    // swapped: every entry, and a few
    let swap = |s: &Value| if s == &json!(deck) { json!(guest) } else { json!(deck) };
    let all: Vec<Value> = seats.iter().map(swap).collect();
    for (name, x) in [
        ("all swapped", all.clone()),
        ("first swapped", {
            let mut c = seats.clone();
            c[0] = swap(&c[0]);
            c
        }),
        ("last swapped", {
            let mut c = seats.clone();
            c[9] = swap(&c[9]);
            c
        }),
    ] {
        let s =
            load(&e, &with(json!(x))).unwrap_or_else(|m| panic!("{name}: should load and stop at the guard, got {m}"));
        let mut t = Rig::open(s, Style::Aggro);
        let w = t.warns();
        let guard: Vec<&String> =
            w.iter().filter(|x| x.starts_with("The replay of this game went a different way")).collect();
        assert_eq!(guard.len(), 1, "{name}: {w:?}");
        assert!(t.live(), "{name}: the game must wait live after the guard, got {}", t.at);
        // and the game goes on from there without panics for a few answers
        t.run_to(t.given.len() + 6, None);
        assert!(
            t.evs.iter().all(|e| e["kind"] != "error"),
            "{name}: {:?}",
            t.evs.iter().find(|e| e["kind"] == "error")
        );
        report.push(format!("{name}: loaded, guard text {:?}", guard[0]));
        t.s.close(Duration::from_secs(10));
    }
    // all one seat (both are people, so the shape is fine; the guard stops it at the first foreign answer)
    let s = load(&e, &with(json!(vec![json!(deck); 10]))).expect("all-one-seat loads");
    let mut t = Rig::open(s, Style::Aggro);
    assert!(t.live());
    assert!(t.warns().iter().any(|w| w.starts_with("The replay of this game went a different way")), "{:?}", t.warns());
    t.s.close(Duration::from_secs(10));
    // absent: a partner save without answer_seats is replayed unchecked (older saves), and replays to the same point
    let mut c = v.clone();
    c.as_object_mut().unwrap().remove("answer_seats");
    let s = load(&e, &c).expect("no answer_seats: loads");
    let mut t = Rig::open(s, Style::Aggro);
    assert!(t.warns().is_empty());
    assert_eq!(t.s.answers_len(), 10);
    assert_eq!(t.at["request"], {
        let s0 = load(&e, &v).unwrap();
        let mut t0 = Rig::open(s0, Style::Aggro);
        let r = t0.at["request"].clone();
        t0.s.close(Duration::from_secs(10));
        r
    });
    t.s.close(Duration::from_secs(10));
    eprintln!("RECORD tampered answer_seats: {report:#?}");
}

#[test]
fn tampered_tier_partner_and_format_in_a_two_person_cali_save() {
    let _g = serial();
    let e = engine();
    let (v, deck, guest) = a_save(&e, false, 8);
    assert!(v.get("tier").is_none(), "a zero-AI save has no tier: {}", v["tier"]);
    let tier = e.decks[&deck].tiers[0].clone();
    let with = |k: &str, x: Option<Value>| {
        let mut c = v.clone();
        match x {
            Some(x) => c[k] = x,
            None => {
                c.as_object_mut().unwrap().remove(k);
            }
        }
        c
    };
    let mut rep = vec![];
    // tier at a zero-AI table: ignored when a real tier or empty; refused when not a string / not a tier
    for (name, x, want_ok) in [
        ("absent", None, true),
        ("a real tier", Some(json!(tier)), true),
        ("empty", Some(json!("")), true),
        ("null", Some(json!(null)), false),
        ("a number", Some(json!(5)), false),
        ("a list", Some(json!(["t1"])), false),
        ("an unknown tier", Some(json!("t9")), false),
    ] {
        let m = refused(&e, &with("tier", x));
        rep.push(format!("tier {name}: {m}"));
        assert_eq!(m != "<loaded>", !want_ok, "tier {name}: {m}");
        if want_ok {
            let s = load(&e, &with("tier", None)).unwrap();
            assert_eq!(s.tier, None);
            assert_eq!(s.saved().get("tier"), None);
            s.close(Duration::from_secs(5));
        }
    }
    // partner tampering
    for (name, x, want) in [
        ("null", Some(json!(null)), Some("a damaged saved game")),
        ("a number", Some(json!(5)), Some("a damaged saved game")),
        ("a list", Some(json!([guest])), Some("a damaged saved game")),
        ("absent", None, Some("a damaged saved game")),
        ("the same deck", Some(json!(deck)), None),
    ] {
        let m = refused(&e, &with("partner", x));
        rep.push(format!("partner {name}: {m}"));
        match want {
            Some(w) => assert_eq!(m, w, "partner {name}"),
            None => assert_ne!(m, "<loaded>", "partner {name}"),
        }
    }
    // a third deck as the partner: the saved seats no longer match the two people
    let third = MY_DECKS.iter().find(|k| **k != deck && **k != guest).unwrap().to_string();
    let m = refused(&e, &with("partner", Some(json!(third))));
    rep.push(format!("partner = a third deck: {m}"));
    assert_ne!(m, "<loaded>");
    // format tampering
    for (name, x) in [
        ("null", json!(null)),
        ("a number", json!(5)),
        ("commander", json!("commander")),
        ("unknown", json!("zzz")),
        ("empty", json!("")),
    ] {
        let m = refused(&e, &with("format", Some(x)));
        rep.push(format!("format {name}: {m}"));
        assert_ne!(m, "<loaded>", "format {name}");
    }
    // seats tampering: the zero-AI table's seats are exactly the two people
    for (name, x) in [
        ("empty", json!([])),
        ("one", json!([deck])),
        ("three", json!([deck, guest, deck])),
        ("a stranger", json!([deck, "nobody"])),
        ("not a list", json!("x")),
        ("numbers", json!([1, 2])),
    ] {
        let m = refused(&e, &with("seats", Some(x)));
        rep.push(format!("seats {name}: {m}"));
        assert_ne!(m, "<loaded>", "seats {name}");
    }
    // answers tampering
    for (name, x) in [("null", json!(null)), ("a string", json!("x")), ("an object", json!({}))] {
        assert_eq!(refused(&e, &with("answers", Some(x))), "a damaged saved game", "answers {name}");
    }
    // version and tape
    assert_eq!(refused(&e, &with("version", Some(json!(2)))), "a saved game from a different version");
    assert_eq!(refused(&e, &with("tape", Some(json!(null)))), "a damaged saved game");
    eprintln!("RECORD tampered two-person Cali save: {rep:#?}");
}

#[test]
fn a_one_person_save_with_no_usable_tier_is_refused() {
    let _g = serial();
    let e = engine();
    let c = commander_deck(&e);
    let tier = e.decks[&c].tiers[0].clone();
    let mut o = Options::new(e.clone(), &c, &tier);
    o.seed = Some(5);
    o.ai = "adaptive".into();
    o.max_rounds = 2;
    o.views = true;
    let s = Session::new(o).unwrap();
    let v = s.saved();
    assert_eq!(v["tier"], json!(tier));
    let with = |x: Option<Value>| {
        let mut c = v.clone();
        match x {
            Some(x) => c["tier"] = x,
            None => {
                c.as_object_mut().unwrap().remove("tier");
            }
        }
        c
    };
    let mut rep = vec![];
    for (name, x) in [
        ("absent", None),
        ("empty", Some(json!(""))),
        ("null", Some(json!(null))),
        ("a number", Some(json!(5))),
        ("a list", Some(json!([]))),
        ("a bool", Some(json!(true))),
        ("an unknown tier", Some(json!("t9"))),
    ] {
        let m = refused(&e, &with(x));
        rep.push(format!("one-person Commander save, tier {name}: {m}"));
        assert_ne!(m, "<loaded>", "one-person save with tier {name} was LOADED");
    }
    // the real one loads
    assert_eq!(refused(&e, &v), "<loaded>");
    // `partner` null / a number with a good tier: still a one-person save
    for x in [json!(null), json!(5)] {
        let mut c = v.clone();
        c["partner"] = x.clone();
        let m = refused(&e, &c);
        rep.push(format!("one-person save with partner {x}: {m}"));
    }
    // a two-person Commander save: tier empty / absent is refused with a message about the tier, not a panic
    let g = MY_DECKS.iter().find(|k| **k != c && e.admits(k, &e.format_of(&c).key).is_ok()).unwrap().to_string();
    let mut o = Options::new(e.clone(), &c, &tier);
    o.partner = Some(g);
    o.seed = Some(5);
    o.ai = "adaptive".into();
    o.views = true;
    o.max_rounds = 2;
    let s2 = Session::new(o).unwrap();
    let v2 = s2.saved();
    for (name, x) in [("absent", None), ("empty", Some(json!(""))), ("null", Some(json!(null)))] {
        let mut c = v2.clone();
        match x {
            Some(x) => c["tier"] = x,
            None => {
                c.as_object_mut().unwrap().remove("tier");
            }
        }
        let m = refused(&e, &c);
        rep.push(format!("two-person Commander save, tier {name}: {m}"));
        assert_ne!(m, "<loaded>", "two-person Commander save with tier {name} was LOADED");
    }
    // answer_seats naming a non-player at a Commander pair is damaged (design 7d)
    let mut c = v2.clone();
    c["answer_seats"] = json!([]);
    assert_eq!(refused(&e, &c), "<loaded>");
    c["answer_seats"] = json!(["nobody"]);
    assert_eq!(refused(&e, &c), "a damaged saved game");
    eprintln!("RECORD tier tampering: {rep:#?}");
}

#[test]
fn a_save_from_before_the_format_with_the_cali_deck_is_refused_clearly() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let c = commander_deck(&e);
    let tier = e.decks[&c].tiers[0].clone();
    let mut o = Options::new(e.clone(), &c, &tier);
    o.seed = Some(7);
    o.ai = "adaptive".into();
    o.views = true;
    o.max_rounds = 2;
    let v = Session::new(o).unwrap().saved();
    assert!(v.get("format").is_none(), "a Commander save has no format key");
    let mut old = v.clone();
    old["deck"] = json!(deck);
    // a save with the Cali deck and no format: a four-player Commander game it cannot be
    let m = refused(&e, &old);
    eprintln!("RECORD old save of the Cali deck: {m}");
    assert!(m.contains("this saved game has no format, so it is a four-player Commander game"), "{m}");
    assert!(m.contains(&deck) || m.contains("cannot be played at a"), "{m}");
    // the same with the format written down as commander
    let mut explicit = old.clone();
    explicit["format"] = json!("commander");
    let m2 = refused(&e, &explicit);
    eprintln!("RECORD explicit format commander, Cali deck: {m2}");
    assert!(m2.contains("cannot be played at a Commander table"), "{m2}");
    // and as a two-person save (partner = the Cali deck's guest) with no format
    let mut pair_old = old.clone();
    pair_old["partner"] = json!(guest);
    pair_old["answer_seats"] = json!([]);
    let m3 = refused(&e, &pair_old);
    eprintln!("RECORD old two-person save with the Cali deck: {m3}");
    assert_ne!(m3, "<loaded>");
    // a Cali deck as the PARTNER of a Commander two-person save
    let mate = MY_DECKS
        .iter()
        .find(|k| **k != c.as_str() && **k != deck.as_str() && e.admits(k, "commander").is_ok())
        .unwrap()
        .to_string();
    let mut o = Options::new(e.clone(), &c, &tier);
    o.partner = Some(mate);
    o.seed = Some(7);
    o.ai = "adaptive".into();
    o.max_rounds = 2;
    let v2 = Session::new(o).unwrap().saved();
    let mut as_partner = v2.clone();
    as_partner["partner"] = json!(deck);
    let m4 = refused(&e, &as_partner);
    eprintln!("RECORD Cali deck as the partner of a Commander save: {m4}");
    assert_ne!(m4, "<loaded>");
}

// ------------------------------------------------------------------------------------------------ (3) the format refuses a deck

#[test]
fn the_format_refuses_decks_both_ways_at_session_new() {
    let _g = serial();
    let e = engine();
    let (deck, _guest) = pair(&e);
    let cali = e.format_of(&deck).key.clone();
    let c = commander_deck(&e);
    // a Cali lobby with a four-seat deck that the Cali format bans, if there is one (formats.toml's ban list)
    let banned: Vec<&&str> = MY_DECKS.iter().filter(|k| **k != deck && e.admits(k, &cali).is_err()).collect();
    eprintln!(
        "RECORD decks of mine the Cali format refuses: {banned:?}; the Cali format admits {} of {} decks",
        MY_DECKS.len() - banned.len() - 1,
        MY_DECKS.len() - 1
    );
    for b in &banned {
        let mut o = opts(&e, &deck, b, 3, 3, false);
        o.format = Some(cali.clone());
        let m = Session::new(o).err().unwrap_or_else(|| panic!("{b} was accepted at a Cali table"));
        assert!(m.contains("cannot be played at a"), "{m}");
        // the other direction: the Cali host's deck is the banned one's host
        let mut o = opts(&e, b, &deck, 3, 3, false);
        o.format = Some(cali.clone());
        assert!(Session::new(o).is_err(), "{b} hosting a Cali table");
    }
    // the Cali deck at a Commander table, either seat
    let mut o = opts(&e, &c, &deck, 3, 3, false);
    o.tier = e.decks[&c].tiers[0].clone();
    o.format = Some("commander".into());
    let m = Session::new(o).err().expect("the Cali deck as the guest of a Commander table");
    assert!(m.contains("cannot be played at a Commander table"), "{m}");
    let mut o = opts(&e, &deck, &c, 3, 3, false);
    o.tier = e.decks[&c].tiers[0].clone();
    o.format = Some("commander".into());
    assert!(Session::new(o).is_err(), "the Cali deck hosting a Commander table");
    // a mirror is refused with the same words whatever the format
    for (d, f) in [(&deck, None), (&c, Some("commander"))] {
        let mut o = opts(&e, d, d, 3, 3, false);
        o.partner = Some(d.clone());
        o.format = f.map(String::from);
        o.tier = e.decks[d].tiers[0].clone();
        assert_eq!(Session::new(o).err().unwrap(), "the two players need different decks");
    }
}

// ------------------------------------------------------------------------------------------------ (6) threads, (7) hint and review

#[test]
fn closing_and_replacing_sessions_leaves_no_game_thread() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let base = practice_threads();
    // a checked and dropped session (a lobby's pre-check) starts nothing
    for seed in 1..40u64 {
        let _ = Session::new(opts(&e, &deck, &guest, seed, 3, true)).unwrap();
    }
    assert_eq!(practice_threads(), base, "Session::new alone must not start a thread");
    // started, waiting on a person, then closed (both seats waiting is the same: one game thread asks one seat)
    for i in 0..6u64 {
        let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, i + 1, 3, true)).unwrap(), Style::Aggro);
        t.run_to(2 + i as usize * 3, None);
        let during = practice_threads();
        assert!(during >= 1);
        t.s.close(Duration::from_secs(10));
        let left = wait_threads_gone(10);
        assert_eq!(left, 0, "after close at answer {}: {left} thread(s) still alive", t.given.len());
    }
    // a session that was undone repeatedly, then closed
    let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, 4, 3, true)).unwrap(), Style::Aggro);
    for _ in 0..5 {
        t.run_to(t.given.len() + 6, None);
        let seat = t.seat();
        let other = if seat == deck { guest.clone() } else { deck.clone() };
        for who in [seat, other] {
            if t.s.undo_point(1, &who).is_some() {
                assert!(t.s.undo(1, &who).is_none());
                t.at = t.wait();
                let n = t.s.answers_len();
                t.given.truncate(n);
                break;
            }
        }
    }
    t.s.close(Duration::from_secs(10));
    assert_eq!(wait_threads_gone(10), 0, "threads left after repeated Undo and close");
}

#[test]
fn hint_and_review_work_per_seat_at_a_table_with_no_ai() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &guest);
    let mut t = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, true)).unwrap(), Style::Aggro);
    t.run_to(14, None);
    let waiting = t.seat();
    let other = if waiting == deck { guest.clone() } else { deck.clone() };
    // the hint is for the seat that is deciding
    for _ in 0..3 {
        let h = t.s.hint(&waiting);
        assert!(h.is_ok(), "hint for the waiting seat: {h:?}");
        assert_eq!(t.s.hint(&other).err().as_deref(), Some("There is no decision waiting for you."));
        assert_eq!(t.s.hint("nobody").err().as_deref(), Some("There is no decision waiting for you."));
    }
    // the review opens when the game ends, for each person
    assert_eq!(t.s.review(&waiting).err().as_deref(), Some("The review opens when the game ends."));
    t.run_out();
    assert_eq!(t.at["kind"], "over");
    let a = t.s.review(&deck).expect("review of the Cali deck's seat");
    let b = t.s.review(&guest).expect("review of the guest's seat");
    let answers = |s: &str| t.given.iter().filter(|g| g.0 == s).count() as u64;
    assert_eq!(a["summary"]["answers"].as_u64(), Some(answers(&deck)));
    assert_eq!(b["summary"]["answers"].as_u64(), Some(answers(&guest)));
    // each review holds only that person's decisions, not the other's
    for (r, s) in [(&a, &deck), (&b, &guest)] {
        for d in r["decisions"].as_array().unwrap() {
            let n = d["n"].as_u64().unwrap() as usize;
            assert_eq!(&t.given[n].0, s, "review of {s} lists decision {n}, which {} made", t.given[n].0);
        }
    }
    assert!(t.s.review("nobody").is_err());
    t.s.close(Duration::from_secs(10));
}

// ------------------------------------------------------------------------------------------------ what the policies reach

/// Records what the two policies actually exercise in the games the Undo tests replay (so the report can say what was
/// covered): the kinds of decisions asked, attacks declared with creatures, targets and choices taken as the last option,
/// priority passed with something on the stack, mulligans
#[test]
fn the_policies_reach_attacks_targets_stack_passes_and_mulligans() {
    let _g = serial();
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut total_attack_nonempty = 0;
    let mut total_stack_pass = 0;
    let mut total_choice = 0;
    for style in [Style::Aggro, Style::MullTwice] {
        for first_is_cali in [true, false] {
            let seed = seed_with_first(&e, &deck, &guest, if first_is_cali { &deck } else { &guest });
            let mut r = Rig::open(Session::new(opts(&e, &deck, &guest, seed, 3, false)).unwrap(), style);
            r.run_out();
            let reqs: Vec<&Value> = r.evs.iter().filter(|e| e["kind"] == "request").collect();
            assert_eq!(reqs.len(), r.given.len());
            let mut kinds: std::collections::BTreeMap<String, usize> = Default::default();
            let (mut atk, mut stack_pass, mut choice, mut mull) = (0, 0, 0, 0);
            for (rq, (_, a)) in reqs.iter().zip(r.given.iter()) {
                let k = rq["request"]["kind"].as_str().unwrap_or("").to_string();
                *kinds.entry(k.clone()).or_default() += 1;
                match k.as_str() {
                    "attack" if a.as_array().is_some_and(|x| !x.is_empty()) => atk += 1,
                    "priority" if rq["request"]["data"]["stack"].as_array().is_some_and(|s| !s.is_empty()) => {
                        stack_pass += 1
                    }
                    "mulligan" if *a == json!("mulligan") => mull += 1,
                    "choose" | "target" | "search" | "mode" | "block" => choice += 1,
                    _ => {}
                }
            }
            eprintln!(
                "RECORD policy {style:?}, Cali deck first {first_is_cali}: {} answers; attacks with creatures {atk}; priority passed with a stack item {stack_pass}; choices/targets/searches {choice}; mulligans taken {mull}; kinds {kinds:?}",
                r.given.len()
            );
            total_attack_nonempty += atk;
            total_stack_pass += stack_pass;
            total_choice += choice;
            r.s.close(Duration::from_secs(10));
        }
    }
    eprintln!(
        "RECORD policy totals: attacks {total_attack_nonempty}, stack passes {total_stack_pass}, choices {total_choice}"
    );
}
