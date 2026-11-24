//! Pins how Commander practice behaves today, so later changes to practice (Cali formats, 1v1 tables, ...) can prove
//! that Commander practice is exactly as before. Test-only: it changes no behaviour.
//!
//! Four things are recorded in `tests/fixtures/commander_practice.json` and recomputed on every run:
//!   1. `seats`: `choose_seats` for every Commander deck of decks.json `mine` (a deck whose format is not Commander,
//!      like the Cali deck, is left out; the decks are read at test time, not named here) x tiers t1..t6 x seeds
//!      1..=10, with no format given: one person; two people (each deck's `partner` is the next deck in the sorted
//!      list); a given `seat`; fixed `opponents`; an exact `seats`; and error cases (the error text is pinned);
//!   2. `sessions`: the hashes and length of the whole event stream (every log line, turn, request, view and the
//!      final table) of bot-played sessions: 2 per Commander deck (adaptive AI, compare off, max_rounds 3), a two-person
//!      one, a t6 one and a full-length one (max_rounds 30);
//!   3. `save`, `save_partner`: Commander save files (one person; two people with `answer_seats` and `partner_shadow`)
//!      written on the code that generated the fixture (a few scripted decisions), and the hash of the event stream of
//!      loading each, replaying it and playing on with the same bot. The test also checks that saving today still
//!      writes the same file; `legacy_save` is only loaded.
//!
//! A mismatch names the pin that failed.
//!
//! WHEN THIS TEST TURNS RED, AND WHAT TO DO. The `seats` pins depend only on practice's own seating code, so they fail
//! only when practice changes. The `sessions` and `save` pins play whole games, so they ALSO fail on any change to the
//! deck lists (data/decks.json), the pools, the card data or card code, or the AI (sim-core), which routine swap and
//! engine PRs make all the time (about 312 commits since 11-09). So:
//!   - Regenerate the fixture on the base commit (origin/main), in its own commit, and never in the same commit as a
//!     practice change. A practice change (Cali formats, 1v1 tables, ...) must then pass WITHOUT regenerating: that is
//!     the proof that Commander practice did not change.
//!   - Each session pin has two hashes: `game_hash` is over the events with every `view` / `views` / `data.view` key
//!     removed. If `game_hash` fails, the game itself changed; if only `hash` fails, a view or event shape changed
//!     (a PR that adds a view field regenerates and the unchanged `game_hash` shows the game is the same).
//!   - When a later regeneration is needed, `legacy_save` (the first save, kept as it was written) is carried over, never
//!     rewritten: it is only loaded, so backward compatibility with the old save format stays proven.
//!   - Run with no AI_*, PLANNER* or SEARCH_* environment variables set (sim-core's settings.rs reads them, and they
//!     change the AI and so the games).
//!
//!     UPDATE_PRACTICE_FIXTURE=1 cargo test --release -p practice --test commander_practice_fixture
//!
//! (without the variable the test only compares). Nothing here depends on time, paths or hash-map order: the seed of
//! every game is given, the hash is FNV-1a over each event's JSON text, and the JSON maps are sorted.

#[path = "session_common/mod.rs"]
mod common;
use common::engine;
use practice::session::setup::{SeatArgs, choose_seats, pool_keys};
use practice::session::{Options, Session};
use serde_json::{Map, Value, json};
use sim_core::format::DEFAULT_FORMAT;
use sim_core::run::Engine;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const TIERS: [&str; 6] = ["t1", "t2", "t3", "t4", "t5", "t6"];
const SEEDS: std::ops::RangeInclusive<u64> = 1..=10;
const LIMIT: Duration = Duration::from_secs(300);
/// the save is written after this many answers (a game in progress, waiting on the next decision)
const SAVE_AFTER: usize = 30;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/commander_practice.json")
}

/// FNV-1a, 64 bit
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf29ce484222325)
    }
    fn add(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= *b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
        self.0 ^= 0xff; // a separator, so ["ab","c"] and ["a","bc"] differ
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
}

/// the keys of decks.json `mine` that play Commander, in the file's order
fn commander_decks(e: &Engine) -> Vec<String> {
    let text = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/decks.json"))
        .expect("read data/decks.json");
    let file: Value = serde_json::from_str(&text).expect("parse data/decks.json");
    let mine = file["mine"].as_array().expect("decks.json has `mine`");
    mine.iter()
        .map(|d| d["key"].as_str().expect("a deck key").to_string())
        .filter(|k| e.format_of(k).key == DEFAULT_FORMAT)
        .collect()
}

// ---------------------------------------------------------------------------------------------- 1. the seats

fn seat_result(r: Result<Vec<String>, String>) -> Value {
    match r {
        Ok(s) => json!(s),
        Err(m) => json!({"err": m}),
    }
}

/// the partner of each deck: the next deck in the sorted list (wrapping round)
fn partner_of(decks: &[String]) -> Vec<(String, String)> {
    let mut sorted = decks.to_vec();
    sorted.sort();
    (0..sorted.len()).map(|i| (sorted[i].clone(), sorted[(i + 1) % sorted.len()].clone())).collect()
}

/// `choose_seats` with `f` changing the arguments from the plain ones
fn seats_of<'a>(e: &Engine, deck: &'a str, tier: &'a str, seed: u64, f: impl FnOnce(&mut SeatArgs<'a>)) -> Value {
    let mut a = SeatArgs { deck, tier, partner: None, seed, seat: None, opponents: None, seats: None };
    f(&mut a);
    seat_result(choose_seats(e, &a))
}

fn seats_pin(e: &Engine, decks: &[String]) -> Value {
    let partners = partner_of(decks);
    // by deck, then tier, then seed
    let grid = |f: &dyn Fn(&str, &str, &str, u64) -> Value, tiers: &[&str], seeds: Vec<u64>| {
        let mut out = Map::new();
        for deck in decks {
            let partner = &partners.iter().find(|p| &p.0 == deck).unwrap().1;
            let mut by_tier = Map::new();
            for tier in tiers {
                let by_seed: Map<String, Value> =
                    seeds.iter().map(|&seed| (seed.to_string(), f(deck, partner, tier, seed))).collect();
                by_tier.insert(tier.to_string(), Value::Object(by_seed));
            }
            out.insert(deck.clone(), Value::Object(by_tier));
        }
        Value::Object(out)
    };
    // three fixed opponents of a tier that are neither the deck nor its partner (the pool is sorted, so this is stable)
    let others = |tier: &str, n: usize, not: &[&str]| -> Vec<String> {
        pool_keys(e, tier).into_iter().filter(|k| !not.contains(&k.as_str())).take(n).collect()
    };
    let all: Vec<u64> = SEEDS.collect();
    let few: Vec<u64> = (1..=3).collect();

    // exact seats: the seats a draw gave, given back (playing the same seed again), and a wrong one
    let exact = |deck: &str, partner: Option<&str>| -> Value {
        let drawn =
            choose_seats(e, &SeatArgs { deck, tier: "t5", partner, seed: 2, seat: None, opponents: None, seats: None })
                .expect("a draw at t5");
        let mut wrong = drawn.clone();
        wrong.swap(0, 1);
        wrong[0] = "nonesuch".into();
        json!({
            "again": seats_of(e, deck, "t5", 2, |a| { a.partner = partner; a.seats = Some(&drawn) }),
            "wrong": seats_of(e, deck, "t5", 2, |a| { a.partner = partner; a.seats = Some(&wrong) }),
            "short": seats_of(e, deck, "t5", 2, |a| { a.partner = partner; a.seats = Some(&drawn[..3]) }),
        })
    };

    let mut errors = vec![];
    let (sorted_first, second) = partners[0].clone(); // the sorted-first deck and its partner
    let second = &second;
    let opp3 = others("t5", 3, &[&sorted_first, second]);
    let opp2 = others("t5", 2, &[&sorted_first, second]);
    let bad3 = vec!["nonesuch".to_string(), opp3[1].clone(), opp3[2].clone()];
    let bad2 = vec!["nonesuch".to_string(), opp2[1].clone()];
    let mut case = |name: &str, v: Value| errors.push(json!({"case": name, "result": v}));
    case("one person, a seat of 0", seats_of(e, &sorted_first, "t5", 1, |a| a.seat = Some(0)));
    case("one person, a seat of 5", seats_of(e, &sorted_first, "t5", 1, |a| a.seat = Some(5)));
    case("one person, two opponents", seats_of(e, &sorted_first, "t5", 1, |a| a.opponents = Some(&opp2)));
    case("one person, an opponent not in the tier", seats_of(e, &sorted_first, "t5", 1, |a| a.opponents = Some(&bad3)));
    case(
        "two people, a seat",
        seats_of(e, &sorted_first, "t5", 1, |a| {
            a.partner = Some(second);
            a.seat = Some(2)
        }),
    );
    case(
        "two people, three opponents",
        seats_of(e, &sorted_first, "t5", 1, |a| {
            a.partner = Some(second);
            a.opponents = Some(&opp3)
        }),
    );
    case(
        "two people, an opponent not in the tier",
        seats_of(e, &sorted_first, "t5", 1, |a| {
            a.partner = Some(second);
            a.opponents = Some(&bad2)
        }),
    );

    json!({
        "one_person": grid(&|d, _, t, s| seats_of(e, d, t, s, |_| {}), &TIERS, all.clone()),
        "two_people": grid(&|d, p, t, s| seats_of(e, d, t, s, |a| a.partner = Some(p)), &TIERS, all.clone()),
        "seat": grid(
            &|d, _, t, s| {
                let by_seat: Map<String, Value> = (1..=4usize)
                    .map(|n| (n.to_string(), seats_of(e, d, t, s, |a| a.seat = Some(n))))
                    .collect();
                Value::Object(by_seat)
            },
            &["t5"],
            (1..=5).collect(),
        ),
        "opponents": grid(
            &|d, p, t, s| {
                let o = others(t, 3, &[d, p]);
                json!({
                    "one_person": seats_of(e, d, t, s, |a| a.opponents = Some(&o)),
                    "two_people": seats_of(e, d, t, s, |a| { a.partner = Some(p); a.opponents = Some(&o[..2]) }),
                })
            },
            &["t5"],
            few,
        ),
        "exact": Value::Object(
            decks
                .iter()
                .map(|d| {
                    let p = &partners.iter().find(|p| &p.0 == d).unwrap().1;
                    (d.clone(), json!({"one_person": exact(d, None), "two_people": exact(d, Some(p))}))
                })
                .collect(),
        ),
        "errors": errors,
    })
}

// ---------------------------------------------------------------------------------------------- 2. the sessions

/// one bot-played session
struct Spec<'a> {
    deck: &'a str,
    partner: Option<&'a str>,
    tier: &'a str,
    seed: u64,
    max_rounds: u32,
}

fn options(e: &Arc<Engine>, sp: &Spec) -> Options {
    let mut o = Options::new(e.clone(), sp.deck, sp.tier);
    o.partner = sp.partner.map(str::to_string);
    o.seed = Some(sp.seed);
    o.ai = "adaptive".into();
    o.compare = false;
    o.views = true;
    o.max_rounds = sp.max_rounds;
    o
}

/// A person's answers: keep every hand, play a land and tap a mana source in the main phase, pass otherwise, no
/// attacks, the first choice otherwise (the bot of the HTTP tests, over the session's own events).
#[derive(Default)]
struct Bot {
    played: Vec<(String, i64)>,
    /// answers in a row that were not a pass (a safety stop)
    streak: usize,
}

impl Bot {
    fn answer(&mut self, req: &Value) -> Value {
        let a = self.pick(req);
        self.streak = if a["do"] == "pass" || !a.is_object() { 0 } else { self.streak + 1 };
        a
    }

    fn pick(&mut self, req: &Value) -> Value {
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

/// the event without any `view` / `views` key, at any depth (so a later PR that adds a view field leaves it alone)
fn without_views(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| *k != "view" && *k != "views")
                .map(|(k, x)| (k.clone(), without_views(x)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(without_views).collect()),
        x => x.clone(),
    }
}

/// What a played session gave: every event, hashed.
struct Played {
    events: usize,
    hash: u64,
    /// the same events with the views removed
    game_hash: u64,
    over: Value,
    /// the answers the bot gave (its live ones), in all and by seat
    given: usize,
    by_seat: Map<String, Value>,
}

/// Start `s` and answer its requests with a bot per person until the game is over (or `stop_after` answers have been
/// given: then the session is left waiting on its next decision, and `over` is null).
fn play(s: &Arc<Session>, stop_after: Option<usize>) -> Played {
    s.start();
    let end = Instant::now() + LIMIT;
    let mut bots: BTreeMap<String, Bot> = BTreeMap::new();
    let (mut cursor, mut events, mut given) = (0u64, 0usize, 0usize);
    let mut by_seat: BTreeMap<String, usize> = BTreeMap::new();
    let (mut h, mut g) = (Fnv::new(), Fnv::new());
    let done = |h: &Fnv, g: &Fnv, events, over, given, by_seat: BTreeMap<String, usize>| Played {
        events,
        hash: h.0,
        game_hash: g.0,
        over,
        given,
        by_seat: by_seat.into_iter().map(|(k, n)| (k, json!(n))).collect(),
    };
    loop {
        assert!(Instant::now() < end, "the game did not finish in {LIMIT:?}");
        let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
        cursor = c;
        for ev in evs {
            events += 1;
            h.add(ev.to_string().as_bytes());
            g.add(without_views(&ev).to_string().as_bytes());
            match ev["kind"].as_str().unwrap_or("") {
                "request" => {
                    if stop_after == Some(given) {
                        return done(&h, &g, events, Value::Null, given, by_seat);
                    }
                    let seat = ev["seat"].as_str().unwrap();
                    let a = bots.entry(seat.to_string()).or_default().answer(&ev["request"]);
                    given += 1;
                    *by_seat.entry(seat.to_string()).or_default() += 1;
                    s.answer(seat, &a);
                }
                "over" | "error" => {
                    assert_eq!(ev["kind"], "over", "the game ended in an error: {ev}");
                    let over = json!({"winner": ev["winner"], "how": ev["how"]});
                    return done(&h, &g, events, over, given, by_seat);
                }
                _ => {}
            }
        }
    }
}

fn hex(x: u64) -> String {
    format!("{x:016x}")
}

/// What the pin says of one played session
fn session_json(s: &Session, sp: &Spec, p: &Played) -> Value {
    json!({
        "deck": sp.deck, "partner": sp.partner, "tier": sp.tier, "seed": sp.seed, "max_rounds": sp.max_rounds,
        "seats": s.seats, "events": p.events, "hash": hex(p.hash), "game_hash": hex(p.game_hash), "over": p.over,
        "answers": p.given, "answers_by_seat": p.by_seat,
    })
}

/// The sessions: 2 per Commander deck (its own tiers in turn, a tier it is simulated against; seeds 1 and 2), then one
/// of two people, one against the t6 pod, and one of full length (30 rounds: the passive bot is killed).
fn sessions_pin(e: &Arc<Engine>, decks: &[String]) -> Value {
    let mut specs = vec![];
    for deck in decks {
        let tiers = &e.decks[deck].tiers;
        assert!(!tiers.is_empty(), "{deck} has no tiers in decks.json");
        for i in 0..2 {
            specs.push(Spec { deck, partner: None, tier: &tiers[i % tiers.len()], seed: i as u64 + 1, max_rounds: 3 });
        }
    }
    let (first, partner) = partner_of(decks)[0].clone();
    specs.push(Spec { deck: &first, partner: Some(&partner), tier: &e.decks[&first].tiers[0], seed: 3, max_rounds: 3 });
    specs.push(Spec { deck: &decks[0], partner: None, tier: "t6", seed: 3, max_rounds: 3 });
    specs.push(Spec { deck: &decks[0], partner: None, tier: &e.decks[&decks[0]].tiers[0], seed: 4, max_rounds: 30 });
    let mut out = vec![];
    for sp in &specs {
        let s = Session::new(options(e, sp)).unwrap();
        let p = play(&s, None);
        s.close(Duration::from_secs(10));
        out.push(session_json(&s, sp, &p));
    }
    json!(out)
}

// ---------------------------------------------------------------------------------------------- 3. the save file

/// A save written after `SAVE_AFTER` answers, and what loading it gives. The save of the committed fixture is the one
/// that gets loaded (a save from the code that made the fixture); today's save is reported next to it.
fn save_pin(e: &Arc<Engine>, sp: &Spec, committed: Option<&Value>) -> Value {
    let s = Session::new(options(e, sp)).unwrap();
    let p = play(&s, Some(SAVE_AFTER));
    assert_eq!(p.given, SAVE_AFTER, "the game ended before {SAVE_AFTER} decisions");
    let saved = s.saved();
    s.close(Duration::from_secs(10));
    let source = committed.map_or(&saved, |c| &c["saved"]);
    let l = load(e, source);
    let r = play(&l, None);
    // (the loaded game played on, so its own save has the file's answers first, then the live ones)
    let (kept, later) = (source["answers"].as_array().unwrap(), l.saved()["answers"].clone());
    assert_eq!(&later.as_array().unwrap()[..kept.len()], &kept[..], "the loaded game replays the saved answers");
    l.close(Duration::from_secs(10));
    json!({
        "deck": sp.deck, "partner": sp.partner, "tier": sp.tier, "seed": sp.seed, "answers_at_save": SAVE_AFTER,
        "saved": saved,
        "replay": {"events": r.events, "hash": hex(r.hash), "game_hash": hex(r.game_hash), "over": r.over,
                   "live_answers": r.given, "live_by_seat": r.by_seat},
    })
}

/// a save loaded the way the server does (through JSON text)
fn load(e: &Arc<Engine>, saved: &Value) -> Arc<Session> {
    let data: Value = serde_json::from_str(&serde_json::to_string(saved).unwrap()).unwrap();
    Session::load(e.clone(), &data, true, false).expect("load the committed save")
}

/// The one-person and the two-person save
fn saves_pin(e: &Arc<Engine>, decks: &[String], committed: Option<&Value>) -> Value {
    let tier = &e.decks[&decks[0]].tiers[0];
    let (first, partner) = partner_of(decks)[0].clone();
    let one = Spec { deck: &decks[0], partner: None, tier, seed: 7, max_rounds: 3 };
    let two = Spec { deck: &first, partner: Some(&partner), tier: &e.decks[&first].tiers[0], seed: 7, max_rounds: 3 };
    let save = save_pin(e, &one, committed.map(|c| &c["save"]));
    let partner_save = save_pin(e, &two, committed.map(|c| &c["save_partner"]));
    assert!(partner_save["saved"]["answer_seats"].is_array(), "a two-person save has answer_seats");
    assert!(partner_save["saved"]["partner_shadow"].is_array(), "a two-person save has partner_shadow");
    json!({"save": save, "save_partner": partner_save})
}

/// the oldest save, only loaded: it must still load (the save format stays readable)
fn legacy_loads(e: &Arc<Engine>, legacy: &Value) {
    let l = load(e, legacy);
    l.close(Duration::from_secs(10));
}

// ---------------------------------------------------------------------------------------------- the test

/// `committed`: the fixture being compared with (None when generating); `old_legacy`: the legacy save to carry over
fn compute(e: &Arc<Engine>, committed: Option<&Value>, old_legacy: Option<Value>) -> Value {
    let decks = commander_decks(e);
    assert!(!decks.is_empty(), "no Commander deck in decks.json `mine`");
    let saves = saves_pin(e, &decks, committed);
    // the first save ever recorded is kept as it was written (only loaded); the first generation starts it
    let legacy = old_legacy.unwrap_or_else(|| saves["save"]["saved"].clone());
    legacy_loads(e, &legacy);
    json!({
        "about": "Pins Commander practice (tests/commander_practice_fixture.rs); regenerate with UPDATE_PRACTICE_FIXTURE=1",
        "decks": decks,
        "seats": seats_pin(e, &decks),
        "sessions": sessions_pin(e, &decks),
        "save": saves["save"],
        "save_partner": saves["save_partner"],
        "legacy_save": legacy,
    })
}

/// Compare one part and say which pin failed
fn check(failures: &mut Vec<String>, pin: &str, want: &Value, got: &Value) {
    if want != got {
        failures.push(format!("PIN FAILED: {pin}"));
        // name the first entry that differs, when the pin is a map or a list
        let diff = |k: String, a: &Value, b: &Value| {
            if a != b {
                let (a, b) = (a.to_string(), b.to_string());
                let cut = |s: &str| s.chars().take(300).collect::<String>();
                Some(format!("    {k}: fixture {} ... now {}", cut(&a), cut(&b)))
            } else {
                None
            }
        };
        let mut lines = vec![];
        match (want, got) {
            (Value::Object(a), Value::Object(b)) => {
                for k in a.keys().chain(b.keys().filter(|k| !a.contains_key(*k))) {
                    lines.extend(diff(k.clone(), a.get(k).unwrap_or(&Value::Null), b.get(k).unwrap_or(&Value::Null)));
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                for i in 0..a.len().max(b.len()) {
                    lines.extend(diff(
                        format!("[{i}]"),
                        a.get(i).unwrap_or(&Value::Null),
                        b.get(i).unwrap_or(&Value::Null),
                    ));
                }
            }
            _ => {}
        }
        failures.extend(lines.into_iter().take(6));
    }
}

/// the session pins without their event hash: the game itself (its table, length, result and answers)
fn game_part(sessions: &Value) -> Value {
    Value::Array(
        sessions
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let mut s = s.as_object().unwrap().clone();
                s.remove("hash");
                Value::Object(s)
            })
            .collect(),
    )
}

/// the session pins' event hash (over the events as sent, views included), with which session it is
fn hash_part(sessions: &Value) -> Value {
    Value::Array(
        sessions
            .as_array()
            .unwrap()
            .iter()
            .map(|s| json!({"deck": s["deck"], "partner": s["partner"], "tier": s["tier"], "seed": s["seed"], "max_rounds": s["max_rounds"], "hash": s["hash"]}))
            .collect(),
    )
}

#[test]
fn commander_practice_is_as_recorded() {
    let e = engine();
    let path = fixture_path();
    let update = std::env::var("UPDATE_PRACTICE_FIXTURE").is_ok_and(|v| v == "1");
    let read = || -> Option<Value> { serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok() };
    if update {
        let legacy = read().and_then(|old| old.get("legacy_save").cloned());
        let now = compute(&e, None, legacy);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&now).unwrap() + "\n").unwrap();
        eprintln!("wrote {}", path.display());
        return;
    }
    let want = read().unwrap_or_else(|| {
        panic!("no readable fixture at {}; generate it with UPDATE_PRACTICE_FIXTURE=1", path.display())
    });
    let now = compute(&e, Some(&want), want.get("legacy_save").cloned());
    let mut failures = vec![];
    check(&mut failures, "the Commander decks of decks.json `mine`", &want["decks"], &now["decks"]);
    check(
        &mut failures,
        "choose_seats (one person, two people, seat, opponents, exact seats, errors)",
        &want["seats"],
        &now["seats"],
    );
    check(
        &mut failures,
        "sessions, game_hash and results: THE GAME ITSELF CHANGED (the events without their views, the length, the winner)",
        &game_part(&want["sessions"]),
        &game_part(&now["sessions"]),
    );
    check(
        &mut failures,
        "sessions, hash only: a view or event shape changed (if the game pin above passed, the game is the same)",
        &hash_part(&want["sessions"]),
        &hash_part(&now["sessions"]),
    );
    for (name, what) in [("save", "one-person"), ("save_partner", "two-person")] {
        check(
            &mut failures,
            &format!("the {what} save file written today ({name}: session.saved())"),
            &want[name]["saved"],
            &now[name]["saved"],
        );
        check(
            &mut failures,
            &format!("loading and replaying the committed {what} save ({name})"),
            &want[name]["replay"],
            &now[name]["replay"],
        );
    }
    assert!(
        failures.is_empty(),
        "Commander practice no longer behaves as recorded:\n{}\n\
         - a failing seats pin: practice's seating changed. A failing sessions / save pin may instead come from a change to \
         the decks, pools, card code or AI since the fixture was made: then regenerate the fixture on the base commit \
         (origin/main), in its own commit, and a practice change must pass WITHOUT regenerating. Never regenerate in the \
         same commit as a practice change; keep `legacy_save` (it is carried over). \
         Regenerate with UPDATE_PRACTICE_FIXTURE=1 and review the diff. Run with no AI_*, PLANNER* or SEARCH_* variables set.",
        failures.join("\n")
    );
}
