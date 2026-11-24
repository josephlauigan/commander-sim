//! Two people at a two-seat format (no AI seat): the table, Undo and Try it from either seat against a fresh session
//! given the same answers, and the save. The Cali deck is the deck of yours whose own format has 2 seats; the guest is
//! the first other deck that format admits. No deck is named.

#[path = "session_common/mod.rs"]
mod sess;

use practice::session::human::answer_to_json;
use practice::session::{MY_DECKS, Options, Session};
use serde_json::{Value, json};
use sim_core::run::Engine;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(300);

fn engine() -> Arc<Engine> {
    sess::engine()
}

/// (the Cali deck, the guest's deck)
fn pair(e: &Engine) -> (String, String) {
    let deck = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck in my decks").to_string();
    let fmt = e.format_of(&deck).key.clone();
    let guest = MY_DECKS
        .iter()
        .find(|k| **k != deck && e.admits(k, &fmt).is_ok())
        .expect("another deck that the format admits")
        .to_string();
    (deck, guest)
}

/// a two-person table with no tier
fn options(e: &Arc<Engine>, deck: &str, guest: &str, seed: u64) -> Options {
    let mut o = Options::new(e.clone(), deck, "");
    o.partner = Some(guest.into());
    o.seed = Some(seed);
    o.ai = "adaptive".into();
    o.compare = false;
    o.views = true;
    o.max_rounds = 3;
    o
}

/// the first seed from `from` at which `first` takes the first seat
fn seed_with_first(e: &Arc<Engine>, deck: &str, guest: &str, first: &str, from: u64) -> u64 {
    (from..from + 200)
        .find(|&s| Session::new(options(e, deck, guest, s)).unwrap().seats[0] == first)
        .expect("a seed with that seat order")
}

/// keep every hand, decline yes/no, play a land and tap a mana source in your main phase, otherwise pass
#[derive(Default)]
struct Bot {
    played: Vec<(String, i64)>,
    streak: usize,
}

impl Bot {
    fn answer(&mut self, req: &Value) -> Value {
        let a = self.pick(req);
        self.streak = if a["do"] == "pass" || !a.is_object() { 0 } else { self.streak + 1 };
        a
    }

    fn pick(&mut self, req: &Value) -> Value {
        if req["choices"] == json!(["yes", "no"]) {
            return json!(1);
        }
        match req["kind"].as_str().unwrap_or("") {
            "priority" => {}
            "mulligan" => return json!("keep"),
            "attack" => return json!([]),
            "choose" | "target" | "block" | "search" => return json!(0),
            _ => return json!(true),
        }
        let pass = json!({"do": "pass"});
        if self.streak >= 25 {
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
        }
        pass
    }
}

fn is_guard_warning(t: &str) -> bool {
    t.starts_with("The replay of this game went a different way at decision ")
}

/// A session driven by hand: the events seen, the request waiting, and every answer given (seat, answer)
struct Table {
    s: Arc<Session>,
    cursor: u64,
    evs: Vec<Value>,
    bot: Bot,
    given: Vec<(String, Value)>,
    /// the request waiting, or the `over` / `error` event that ended the game
    at: Value,
}

impl Table {
    fn open(s: Arc<Session>) -> Table {
        s.start();
        let mut t = Table { s, cursor: 0, evs: vec![], bot: Bot::default(), given: vec![], at: Value::Null };
        t.at = t.wait();
        t
    }

    /// the next request (or the end of the game) after what has been read
    fn wait(&mut self) -> Value {
        let end = Instant::now() + LIMIT;
        loop {
            assert!(
                Instant::now() < end,
                "no request in {LIMIT:?}; last events: {:?}",
                &self.evs[self.evs.len().saturating_sub(4)..]
            );
            let (c, evs) = self.s.events_after(self.cursor, Duration::from_millis(200));
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

    /// answer the waiting request: from `script` (the answer for this position, which must be by the same seat), or
    /// the bot's
    fn step(&mut self, script: Option<&[(String, Value)]>) {
        assert!(self.live(), "the game is over: {}", self.at);
        let seat = self.seat();
        let a = match script {
            Some(sc) => {
                let (s, a) = &sc[self.given.len()];
                assert_eq!(*s, seat, "answer {}: the script's seat is not the one the game asks", self.given.len());
                a.clone()
            }
            None => self.bot.answer(&self.at["request"]),
        };
        self.given.push((seat.clone(), a.clone()));
        self.s.answer(&seat, &a);
        self.at = self.wait();
    }

    /// answer until `n` answers are given (or the game ends first)
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

    /// how many times the replay's seat guard stopped a replay (it must never, on a game nobody edited)
    fn guard_warns(&self) -> usize {
        self.evs.iter().filter(|e| e["kind"] == "warn" && e["text"].as_str().is_some_and(is_guard_warning)).count()
    }

    /// the log texts after the last reset (all of them, with none)
    fn logs_after_reset(&self) -> Vec<String> {
        let from = self.evs.iter().rposition(|e| e["kind"] == "reset").map_or(0, |i| i + 1);
        self.evs[from..]
            .iter()
            .filter(|e| e["kind"] == "log")
            .map(|e| e["text"].as_str().unwrap().to_string())
            .collect()
    }
}

/// where a game stands: the logs since the reset, the waiting decision (the request with its view, and its seat), the
/// answers' seats, and the save it would write now
#[derive(Debug, PartialEq)]
struct Point {
    logs: Vec<String>,
    request: Value,
    seat: Value,
    answer_seats: Vec<String>,
    saved: Value,
}

fn point(t: &Table) -> Point {
    let saved = t.s.saved();
    let seats = saved["answer_seats"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    Point {
        logs: t.logs_after_reset(),
        request: t.at["request"].clone(),
        seat: t.at["seat"].clone(),
        answer_seats: seats,
        saved,
    }
}

fn tape_len(s: &Session) -> usize {
    s.tape.lock().unwrap().entries.len()
}

/// the save without the comparison's entries (workers finish them when they finish: not part of the game's state)
fn without_shadows(mut v: Value) -> Value {
    let m = v.as_object_mut().unwrap();
    m.remove("shadow");
    m.remove("partner_shadow");
    v
}

fn masked(p: Point) -> Point {
    Point { saved: without_shadows(p.saved), ..p }
}

// ------------------------------------------------------------------------------------------------ the table

#[test]
fn a_two_person_table_of_a_two_seat_format_has_no_ai_seat() {
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut orders = std::collections::BTreeSet::new();
    for seed in 1..=12u64 {
        let s = Session::new(options(&e, &deck, &guest, seed)).unwrap();
        assert_eq!(s.seats.len(), 2, "seed {seed}");
        let mut sorted = s.seats.clone();
        sorted.sort();
        let mut want = vec![deck.clone(), guest.clone()];
        want.sort();
        assert_eq!(sorted, want, "seed {seed}: both people, nobody else");
        assert_eq!(s.humans, vec![deck.clone(), guest.clone()]);
        assert_eq!(s.tier, None);
        assert_eq!(s.format, e.format_of(&deck).key);
        assert_eq!(s.saved().get("tier"), None, "a table of people only has no tier in its save");
        orders.insert(s.seats.clone());
    }
    assert_eq!(orders.len(), 2, "both seat orders come up across seeds: {orders:?}");

    // a tier given is ignored (the people's decks are not drawn from it); a wrong one is still refused
    let mut o = options(&e, &deck, &guest, 3);
    o.tier = e.decks[&deck].tiers[0].clone();
    assert_eq!(Session::new(o).unwrap().tier, None);
    let mut o = options(&e, &deck, &guest, 3);
    o.tier = "t9".into();
    assert!(Session::new(o).err().unwrap().starts_with("unknown tier \"t9\": one of t0"));

    // no AI opponents to pick, and the seats are drawn
    let mut o = options(&e, &deck, &guest, 3);
    o.opponents = Some(vec![guest.clone()]);
    let why = Session::new(o).err().unwrap();
    assert!(why.ends_with("for two people has no AI opponents"), "{why}");
    let mut o = options(&e, &deck, &guest, 3);
    o.opponents = Some(vec![]);
    assert!(Session::new(o).is_ok());
    let mut o = options(&e, &deck, &guest, 3);
    o.seat = Some(1);
    assert_eq!(Session::new(o).err().unwrap(), "with two players the seats are drawn at random");
    // seats given in full: both orders
    for seats in [vec![deck.clone(), guest.clone()], vec![guest.clone(), deck.clone()]] {
        let mut o = options(&e, &deck, &guest, 3);
        o.seats = Some(seats.clone());
        assert_eq!(Session::new(o).unwrap().seats, seats);
    }
    for bad in [vec![deck.clone()], vec![deck.clone(), deck.clone()], vec![deck.clone(), guest.clone(), deck.clone()]] {
        let mut o = options(&e, &deck, &guest, 3);
        o.seats = Some(bad.clone());
        assert_eq!(Session::new(o).err().unwrap(), "seats: your two decks, in turn order", "{bad:?}");
    }
    // the same deck twice is still refused
    let mut o = options(&e, &deck, &deck, 3);
    o.partner = Some(deck.clone());
    assert_eq!(Session::new(o).err().unwrap(), "the two players need different decks");

    // an AI table needs a tier: one person here, and two people at a four-seat format
    let mut o = options(&e, &deck, &guest, 3);
    o.partner = None;
    assert_eq!(Session::new(o).err().unwrap(), "unknown tier None");
    let c = MY_DECKS.iter().find(|k| e.format_of(k).seats == 4).expect("a four-seat deck in my decks");
    let cg = MY_DECKS.iter().find(|k| *k != c && e.admits(k, &e.format_of(c).key).is_ok()).expect("a second one");
    let mut o = options(&e, c, cg, 3);
    assert_eq!(Session::new(o.clone()).err().unwrap(), "unknown tier None");
    o.tier = e.decks[*c].tiers[0].clone();
    let s = Session::new(o).unwrap();
    assert_eq!((s.seats.len(), s.tier.is_some()), (4, true));
}

// ------------------------------------------------------------------------------------------------ Undo

/// One Undo case: play `stop` answers with the bot, take back `n` of `seat`'s answers, and compare the game that asks
/// again with a fresh session given the same answers up to there (and `more` answers on from there). Returns the record.
fn undo_case(
    e: &Arc<Engine>,
    deck: &str,
    guest: &str,
    seed: u64,
    stop: usize,
    seat: &str,
    n: i64,
    more: usize,
) -> Option<String> {
    let mut t = Table::open(Session::new(options(e, deck, guest, seed)).unwrap());
    t.run_to(stop, None);
    if !t.live() {
        return None; // the game ended first
    }
    let to = t.s.undo_point(n, seat)?;
    let first = t.s.seats[0].clone();
    let original = t.given.clone();
    let seats_before = point(&t).answer_seats;
    let tape_before = tape_len(&t.s);
    assert!(t.s.undo(n, seat).is_none());
    t.at = t.wait();
    t.given.truncate(to);
    let after = point(&t);
    assert_eq!(
        after.answer_seats,
        seats_before[..to].to_vec(),
        "the answers kept are the first {to}, with their seats"
    );
    let dropped = original.len() - to;

    let mut f = Table::open(Session::new(options(e, deck, guest, seed)).unwrap());
    f.run_to(to, Some(&original));
    let fresh = point(&f);
    assert_eq!(after.logs, fresh.logs, "seed {seed} stop {stop} {seat} n {n}: the log after the reset");
    assert_eq!(after.seat, fresh.seat);
    assert_eq!(after.request, fresh.request, "seed {seed} stop {stop} {seat} n {n}: the waiting request, view and all");
    assert_eq!(after.answer_seats, fresh.answer_seats);
    assert_eq!(after.saved, fresh.saved, "seed {seed} stop {stop} {seat} n {n}: the save");
    assert_eq!(after, fresh);

    // and on from there, the same bot (fresh in both) answering: the same game
    t.bot = Bot::default();
    f.bot = Bot::default();
    t.run_to(to + more, None);
    f.run_to(to + more, None);
    assert_eq!(point(&t), point(&f), "seed {seed} stop {stop} {seat} n {n}: {more} answers on");
    assert_eq!(t.given, f.given);
    let on = f.given.len() - to;
    assert_eq!(tape_len(&t.s) + tape_len(&f.s), 0);
    assert_eq!(t.guard_warns() + f.guard_warns(), 0, "the seat guard fired on a game nobody edited");
    t.s.close(Duration::from_secs(10));
    f.s.close(Duration::from_secs(10));
    Some(format!(
        "seed {seed}, first seat {} ({}), stopped at decision {stop}, Undo {n} of seat {seat} ({}): back to decision {to} \
         (kept {to}, dropped {dropped}), {on} answers on; tape length before {tape_before} after {} fresh {}; the replay matched a fresh session",
        if first == deck { "the Cali deck" } else { "the guest" },
        first,
        if seat == deck { "the Cali deck" } else { "the guest" },
        tape_len(&t.s),
        tape_len(&f.s),
    ))
}

/// how many answers a whole game takes with the bot
fn game_length(e: &Arc<Engine>, deck: &str, guest: &str, seed: u64) -> usize {
    let mut t = Table::open(Session::new(options(e, deck, guest, seed)).unwrap());
    t.run_out();
    assert_eq!(t.at["kind"], "over", "{}", t.at);
    let n = t.given.len();
    t.s.close(Duration::from_secs(10));
    n
}

#[test]
fn undo_from_either_seat_replays_to_the_game_a_fresh_session_reaches() {
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut cases = 0;
    for first in [&deck, &guest] {
        let seed = seed_with_first(&e, &deck, &guest, first, 1);
        let len = game_length(&e, &deck, &guest, seed);
        eprintln!("RECORD seed {seed}: first seat {first}, a game of {len} answers with the bot, max_rounds 3");
        assert!(len > 12, "a game of {len} answers is too short to test mid-game");
        // the mulligan (both people have answered once: decision 2 waits), then mid-game
        for stop in [2, len / 2] {
            for who in [&deck, &guest] {
                for n in [1, 3] {
                    if let Some(r) = undo_case(&e, &deck, &guest, seed, stop, who, n, 10) {
                        eprintln!("RECORD {r}");
                        cases += 1;
                    }
                }
            }
        }
    }
    assert!(cases >= 12, "{cases} cases ran");
}

#[test]
fn undo_takes_back_the_other_persons_later_answers() {
    let e = engine();
    let (deck, guest) = pair(&e);
    for first in [&deck, &guest] {
        let seed = seed_with_first(&e, &deck, &guest, first, 1);
        let other = if first == &deck { &guest } else { &deck };
        let mut t = Table::open(Session::new(options(&e, &deck, &guest, seed)).unwrap());
        t.run_to(2, None); // both people have made their mulligan
        assert_eq!(t.given.iter().map(|g| g.0.clone()).collect::<Vec<_>>(), vec![first.clone(), other.clone()]);
        // the first seat takes its mulligan back: the other person's answer, made after it, goes too
        assert!(t.s.undo(1, first).is_none());
        t.at = t.wait();
        assert_eq!(t.s.answers_len(), 0, "nothing is kept");
        assert_eq!(t.seat(), *first, "the first seat is asked again");
        t.given.clear();
        t.step(None); // the same keep
        // the other person is asked again live, not answered from the old record
        assert!(t.live() && t.seat() == *other);
        assert!(t.evs.last().unwrap().get("replay").is_none(), "{}", t.evs.last().unwrap());
        assert_eq!(point(&t).answer_seats, vec![first.clone()]);
        assert_eq!(t.guard_warns(), 0);
        eprintln!(
            "RECORD seed {seed}: first seat {first}: Undo of the first seat's mulligan took back the other person's too; the other was asked again live"
        );
        t.s.close(Duration::from_secs(10));
    }
}

// ------------------------------------------------------------------------------------------------ Try it

#[test]
fn try_it_from_the_second_seat_applies_the_ai_choice() {
    let e = engine();
    let (deck, guest) = pair(&e);
    // a decision of the guest where the AI's choice is one action
    let mut found = None;
    for seed in 1..=8u64 {
        let mut o = options(&e, &deck, &guest, seed);
        o.compare = true;
        let mut t = Table::open(Session::new(o).unwrap());
        t.run_out();
        assert_eq!(t.at["kind"], "over", "{}", t.at);
        let review = t.s.review(&guest).unwrap();
        let hit = review["decisions"].as_array().unwrap().iter().find_map(|d| {
            let n = d["n"].as_u64()? as usize;
            let en = t.s.try_entry(&json!(n), &guest)?;
            // one that is not what the guest answered there, so that applying it changes the game
            en.ai_answer()
                .filter(|a| t.given.get(n).is_some_and(|g| g.1 != answer_to_json(a)))
                .map(|a| (n, a, d["differs"].clone(), en.ai_text()))
        });
        if let Some(h) = hit {
            found = Some((seed, t, h));
            break;
        }
        t.s.close(Duration::from_secs(10));
    }
    let (seed, mut t, (n, then, differs, ai_text)) = found.expect("a guest decision whose AI choice is one action");
    let original = t.given.clone();
    assert_eq!(original[n].0, guest, "decision {n} is the guest's");
    let seats_before = point(&t).answer_seats;
    assert!(t.s.try_it(&json!(n), &guest).is_none());
    t.at = t.wait();
    t.given.truncate(n);
    t.given.push((guest.clone(), answer_to_json(&then)));
    let after = masked(point(&t));
    assert_eq!(after.answer_seats[..n], seats_before[..n]);
    assert_eq!(after.answer_seats.len(), n + 1, "the AI's answer stands as the guest's answer {n}");
    assert_eq!(after.answer_seats[n], guest);
    let applied = t.evs.iter().rev().find(|e| e["kind"] == "reset").unwrap()["tryit"].clone();
    assert_eq!((applied["applied"].clone(), applied["by"].clone()), (json!(true), json!(guest)));

    // a fresh session given the first n answers, then the AI's, is at the same point
    let mut o = options(&e, &deck, &guest, seed);
    o.compare = true;
    let mut f = Table::open(Session::new(o).unwrap());
    let mut script = original[..n].to_vec();
    script.push((guest.clone(), answer_to_json(&then)));
    f.run_to(n + 1, Some(&script));
    let fresh = masked(point(&f));
    assert_eq!(after.logs, fresh.logs);
    assert_eq!(after.request, fresh.request);
    assert_eq!(after.seat, fresh.seat);
    assert_eq!(after.answer_seats, fresh.answer_seats);
    assert_eq!(after.saved, fresh.saved);
    assert_eq!(tape_len(&t.s) + tape_len(&f.s), 0);
    assert_eq!(t.guard_warns() + f.guard_warns(), 0, "the seat guard fired on a game nobody edited");
    eprintln!(
        "RECORD seed {seed}: Try it from the guest at decision {n} (AI: {ai_text:?}, differs {differs}): applied, the game is at decision {} \
         as a fresh session given the same answers is; tape length {} / {}",
        n + 1,
        tape_len(&t.s),
        tape_len(&f.s)
    );
    t.s.close(Duration::from_secs(10));
    f.s.close(Duration::from_secs(10));
}

// ------------------------------------------------------------------------------------------------ the save

#[test]
fn a_two_person_cali_save_has_no_tier_and_loads_to_the_same_point() {
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &guest, 1);
    let len = game_length(&e, &deck, &guest, seed);
    let mut t = Table::open(Session::new(options(&e, &deck, &guest, seed)).unwrap());
    t.run_to(len / 2, None);
    assert!(t.live());
    let saved = t.s.saved();
    for k in ["format", "partner", "answer_seats", "partner_shadow", "seats", "answers", "tape"] {
        assert!(saved.get(k).is_some(), "the save has {k}");
    }
    assert_eq!(saved.get("tier"), None);
    assert_eq!(saved["version"], 1);
    assert_eq!(saved["format"], e.format_of(&deck).key.as_str());
    assert_eq!(saved["tape"], json!([]));
    assert_eq!(saved["answer_seats"].as_array().unwrap().len(), len / 2);
    let text = serde_json::to_string(&saved).unwrap();
    let here = point(&t);
    t.s.close(Duration::from_secs(10));

    // it loads to the decision it was saved at
    let l = Session::load(e.clone(), &serde_json::from_str(&text).unwrap(), true, false).unwrap();
    assert_eq!(
        (l.tier.clone(), l.partner.clone(), l.humans.clone()),
        (None, Some(guest.clone()), vec![deck.clone(), guest.clone()])
    );
    let lt = Table::open(l);
    assert_eq!(point(&lt), here);
    assert_eq!(lt.guard_warns(), 0);
    lt.s.close(Duration::from_secs(10));

    // a tier in it is ignored (a table of people only has none); null or not a string is damaged; one person needs one
    let edit = |f: &dyn Fn(&mut Value)| {
        let mut v: Value = serde_json::from_str(&text).unwrap();
        f(&mut v);
        Session::load(e.clone(), &v, true, false)
    };
    let given = edit(&|v| v["tier"] = json!(e.decks[&deck].tiers[0])).unwrap();
    assert_eq!(given.tier, None);
    assert_eq!(given.saved().get("tier"), None);
    for bad in [Value::Null, json!(5), json!(["t1"])] {
        let why = edit(&|v| v["tier"] = bad.clone()).err().unwrap();
        assert_eq!(why, "a damaged saved game", "tier {bad}");
    }
    // the same file for one person (no partner): the AI seats have no tier
    let why = edit(&|v| {
        let m = v.as_object_mut().unwrap();
        m.remove("partner");
        m.remove("partner_shadow");
        m.remove("answer_seats");
    })
    .err()
    .unwrap();
    assert_eq!(why, "a damaged saved game");
}

// ------------------------------------------------------------------------------------------------ the tape

#[test]
fn the_tape_stays_empty_without_ai_seats() {
    let e = engine();
    let (deck, guest) = pair(&e);
    let mut o = options(&e, &deck, &guest, 5);
    o.compare = true;
    let mut t = Table::open(Session::new(o).unwrap());
    let mut hints = 0;
    while t.live() {
        if hints < 4 && t.at["request"]["kind"] == "priority" {
            t.s.hint(&t.seat()).unwrap();
            hints += 1;
        }
        t.step(None);
        assert_eq!(tape_len(&t.s), 0);
    }
    assert_eq!(t.at["kind"], "over", "{}", t.at);
    assert_eq!(t.s.saved()["tape"], json!([]));
    assert!(hints > 0);
    eprintln!("RECORD seed 5: a whole game with compare on and {hints} hints: tape length {}", tape_len(&t.s));
    t.s.close(Duration::from_secs(30));
}

// ------------------------------------------------------------------------------------------------ the seat guard

#[test]
fn the_seat_guard_stops_a_replay_whose_seats_do_not_match() {
    let e = engine();
    let (deck, guest) = pair(&e);
    let seed = seed_with_first(&e, &deck, &guest, &deck, 1);
    let mut t = Table::open(Session::new(options(&e, &deck, &guest, seed)).unwrap());
    t.run_to(8, None);
    assert!(t.live());
    let saved = t.s.saved();
    let seats: Vec<String> =
        saved["answer_seats"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    assert_eq!(seats.len(), 8);
    t.s.close(Duration::from_secs(10));
    let other = |k: &str| if k == deck { guest.clone() } else { deck.clone() };
    let load = |f: &dyn Fn(&mut Value)| {
        let mut v = saved.clone();
        f(&mut v);
        Session::load(e.clone(), &v, true, false)
    };
    let same = |a: &Table, b: &Table| {
        assert_eq!((a.at["request"].clone(), a.at["seat"].clone()), (b.at["request"].clone(), b.at["seat"].clone()));
    };

    // an untouched save: the guard does not fire
    let ok = Table::open(load(&|_| {}).unwrap());
    assert_eq!((ok.guard_warns(), ok.s.answers_len()), (0, 8));

    // every seat swapped: the first decision is asked of one seat, its recorded answer is the other's: a live ask at
    // decision 0, one seat-less warning, nothing replayed
    let swapped: Vec<String> = seats.iter().map(|k| other(k)).collect();
    let g = Table::open(load(&|v| v["answer_seats"] = json!(swapped)).unwrap());
    assert!(g.live());
    assert_eq!(g.guard_warns(), 1, "{:?}", g.evs.iter().filter(|e| e["kind"] == "warn").collect::<Vec<_>>());
    let w = g.evs.iter().find(|e| e["kind"] == "warn").unwrap();
    assert_eq!(w["text"], "The replay of this game went a different way at decision 0: it goes on from here.");
    assert!(w.get("seat").is_none() && w.get("replay").is_none(), "{w}");
    assert_eq!(g.s.answers_len(), 0);
    assert_eq!(g.seat(), seats[0], "the first decision is live, asked of its own seat");
    // it is the decision the unedited game starts with
    let fresh = Table::open(Session::new(options(&e, &deck, &guest, seed)).unwrap());
    same(&g, &fresh);
    eprintln!("RECORD guard: all seats swapped: 1 warning at decision 0, live ask of {}", g.seat());
    g.s.close(Duration::from_secs(10));
    fresh.s.close(Duration::from_secs(10));

    // two answers' seats swapped in the middle: the replay runs to the first and stops there
    let at = (1..seats.len()).find(|&i| seats[i] != seats[i - 1]).unwrap() + 2;
    let at = at.min(seats.len() - 2);
    let k = (at..seats.len() - 1).find(|&i| seats[i] != seats[i + 1]).expect("two answers by different seats");
    let mut mid = seats.clone();
    mid.swap(k, k + 1);
    let g = Table::open(load(&|v| v["answer_seats"] = json!(mid)).unwrap());
    assert!(g.live());
    assert_eq!(g.guard_warns(), 1);
    assert!(
        g.evs.iter().any(|e| e["text"]
            == format!("The replay of this game went a different way at decision {k}: it goes on from here."))
    );
    assert_eq!(g.s.answers_len(), k);
    assert_eq!(point(&g).answer_seats, seats[..k].to_vec());
    eprintln!(
        "RECORD guard: seats of answers {k} and {} swapped: 1 warning at decision {k}, {k} answers kept, live from there",
        k + 1
    );
    g.s.close(Duration::from_secs(10));

    // answers the game never asks: left over at the end, one warning
    let mut t = Table::open(Session::new(options(&e, &deck, &guest, seed)).unwrap());
    t.run_out();
    assert_eq!(t.at["kind"], "over");
    let done = t.s.saved();
    let n = t.given.len();
    t.s.close(Duration::from_secs(10));
    let mut v = done.clone();
    v["answers"].as_array_mut().unwrap().extend([json!({"do": "pass"}), json!({"do": "pass"})]);
    v["answer_seats"].as_array_mut().unwrap().extend([json!(deck), json!(guest)]);
    let g = Table::open(Session::load(e.clone(), &v, true, false).unwrap());
    assert_eq!(g.at["kind"], "over", "{}", g.at);
    assert_eq!(g.guard_warns(), 1);
    assert!(
        g.evs.iter().any(|e| e["text"]
            == format!("The replay of this game went a different way at decision {n}: it goes on from here."))
    );
    // the untouched finished save: no warning
    let g2 = Table::open(Session::load(e.clone(), &done, true, false).unwrap());
    assert_eq!((g2.at["kind"].clone(), g2.guard_warns()), (json!("over"), 0));
    eprintln!(
        "RECORD guard: 2 left-over answers after a finished game of {n}: 1 warning at decision {n}; the untouched finished save: 0"
    );

    // a save whose answer_seats cannot be right is damaged: a non-player, the wrong length, not a list, not a string
    for (what, bad) in [
        ("a non-player", {
            let mut b = seats.clone();
            b[3] = "nobody".into();
            json!(b)
        }),
        ("too short", json!(seats[..7].to_vec())),
        ("too long", json!([seats.clone(), vec![deck.clone()]].concat())),
        ("null", Value::Null),
        ("a number", json!(5)),
        ("a number in the list", {
            let mut b: Vec<Value> = seats.iter().map(|s| json!(s)).collect();
            b[2] = json!(7);
            json!(b)
        }),
    ] {
        let why = load(&|v| v["answer_seats"] = bad.clone()).err().unwrap_or_else(|| panic!("{what}: loaded"));
        assert_eq!(why, "a damaged saved game", "{what}");
    }
    // a two-player save without answer_seats is replayed unchecked
    let g = Table::open(
        load(&|v| {
            v.as_object_mut().unwrap().remove("answer_seats");
        })
        .unwrap(),
    );
    assert_eq!((g.guard_warns(), g.s.answers_len()), (0, 8));
    g.s.close(Duration::from_secs(10));
    ok.s.close(Duration::from_secs(10));
    g2.s.close(Duration::from_secs(10));
}
