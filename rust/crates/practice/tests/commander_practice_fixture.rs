//! Pins how Commander practice behaves today, so later changes to practice (Cali formats, 1v1 tables, ...) can prove
//! that Commander practice is exactly as before. Test-only: it changes no behaviour.
//!
//! Three things are recorded in `tests/fixtures/commander_practice.json` and recomputed on every run:
//!   1. `seats`: `choose_seats` for every Commander deck of decks.json `mine` (a deck whose format is not Commander,
//!      like the Cali deck, is left out; the decks are read at test time, not named here) x tiers t1..t6 x seeds
//!      1..=10, with no format given;
//!   2. `sessions`: the hash and length of the whole event stream (every log line, turn, request, view and the
//!      final table) of 2 bot-played sessions per Commander deck (adaptive AI, compare off, max_rounds 3);
//!   3. `save`: a Commander save file written on the code that generated the fixture (one person, a few scripted
//!      decisions), and the hash of the event stream of loading it, replaying it and playing on with the same bot.
//!      The test also checks that saving today still writes the same file.
//!
//! A mismatch names the pin that failed. To accept a deliberate change, regenerate the fixture and review the diff:
//!
//!     UPDATE_PRACTICE_FIXTURE=1 cargo test --release -p practice --test commander_practice_fixture
//!
//! (without the variable the test only compares). Nothing here depends on time, paths or hash-map order: the seed of
//! every game is given, the hash is FNV-1a over each event's JSON text, and the JSON maps are sorted.

#[path = "session_common/mod.rs"]
mod common;
use common::engine;
use practice::session::setup::{SeatArgs, choose_seats};
use practice::session::{Options, Session};
use serde_json::{Map, Value, json};
use sim_core::format::DEFAULT_FORMAT;
use sim_core::run::Engine;
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

fn seats_pin(e: &Engine, decks: &[String]) -> Value {
    let mut out = Map::new();
    for deck in decks {
        let mut by_tier = Map::new();
        for tier in TIERS {
            let mut by_seed = Map::new();
            for seed in SEEDS {
                let r = choose_seats(
                    e,
                    &SeatArgs { deck, tier, partner: None, seed, seat: None, opponents: None, seats: None },
                );
                by_seed.insert(
                    seed.to_string(),
                    match r {
                        Ok(s) => json!(s),
                        Err(m) => json!({"err": m}),
                    },
                );
            }
            by_tier.insert(tier.to_string(), Value::Object(by_seed));
        }
        out.insert(deck.clone(), Value::Object(by_tier));
    }
    Value::Object(out)
}

// ---------------------------------------------------------------------------------------------- 2. the sessions

fn options(e: &Arc<Engine>, deck: &str, tier: &str, seed: u64) -> Options {
    let mut o = Options::new(e.clone(), deck, tier);
    o.seed = Some(seed);
    o.ai = "adaptive".into();
    o.compare = false;
    o.views = true;
    o.max_rounds = 3;
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

/// What a played session gave: every event, hashed.
struct Played {
    events: usize,
    hash: u64,
    over: Value,
    /// the answers the bot gave (its live ones)
    given: usize,
}

/// Start `s` and answer its requests with the bot until the game is over (or `stop_after` answers have been given:
/// then the session is left waiting on its next decision, and `over` is null).
fn play(s: &Arc<Session>, stop_after: Option<usize>) -> Played {
    s.start();
    let end = Instant::now() + LIMIT;
    let mut bot = Bot::default();
    let (mut cursor, mut events, mut given) = (0u64, 0usize, 0usize);
    let mut h = Fnv::new();
    loop {
        assert!(Instant::now() < end, "the game did not finish in {LIMIT:?}");
        let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
        cursor = c;
        for ev in evs {
            events += 1;
            h.add(ev.to_string().as_bytes());
            match ev["kind"].as_str().unwrap_or("") {
                "request" => {
                    if stop_after == Some(given) {
                        return Played { events, hash: h.0, over: Value::Null, given };
                    }
                    let a = bot.answer(&ev["request"]);
                    given += 1;
                    s.answer(ev["seat"].as_str().unwrap(), &a);
                }
                "over" | "error" => {
                    assert_eq!(ev["kind"], "over", "the game ended in an error: {ev}");
                    return Played {
                        events,
                        hash: h.0,
                        over: json!({"winner": ev["winner"], "how": ev["how"]}),
                        given,
                    };
                }
                _ => {}
            }
        }
    }
}

fn hex(x: u64) -> String {
    format!("{x:016x}")
}

/// 2 sessions per Commander deck: its own tiers in turn (a tier it is simulated against), seeds 1 and 2
fn sessions_pin(e: &Arc<Engine>, decks: &[String]) -> Value {
    let mut out = vec![];
    for deck in decks {
        let tiers = &e.decks[deck].tiers;
        assert!(!tiers.is_empty(), "{deck} has no tiers in decks.json");
        for i in 0..2 {
            let (tier, seed) = (&tiers[i % tiers.len()], i as u64 + 1);
            let s = Session::new(options(e, deck, tier, seed)).unwrap();
            let p = play(&s, None);
            s.close(Duration::from_secs(10));
            out.push(json!({
                "deck": deck, "tier": tier, "seed": seed, "seats": s.seats,
                "events": p.events, "hash": hex(p.hash), "over": p.over, "answers": p.given,
            }));
        }
    }
    json!(out)
}

// ---------------------------------------------------------------------------------------------- 3. the save file

/// the deck and tier of the save: the first Commander deck's first tier
fn save_pin(e: &Arc<Engine>, decks: &[String], committed: Option<&Value>) -> Value {
    let deck = &decks[0];
    let tier = &e.decks[deck].tiers[0];
    let s = Session::new(options(e, deck, tier, 7)).unwrap();
    let p = play(&s, Some(SAVE_AFTER));
    assert_eq!(p.given, SAVE_AFTER, "the game ended before {SAVE_AFTER} decisions");
    let saved = s.saved();
    s.close(Duration::from_secs(10));
    // the committed save is the one that gets loaded (a save from the code that made the fixture); today's save is
    // reported next to it
    let source = committed.map_or(&saved, |c| &c["saved"]);
    let text = serde_json::to_string(source).unwrap();
    let data: Value = serde_json::from_str(&text).unwrap();
    let l = Session::load(e.clone(), &data, true, false).expect("load the committed save");
    let r = play(&l, None);
    // (the loaded game played on, so its own save has the file's answers first, then the live ones)
    let (kept, later) = (data["answers"].as_array().unwrap(), l.saved()["answers"].clone());
    assert_eq!(&later.as_array().unwrap()[..kept.len()], &kept[..], "the loaded game replays the saved answers");
    l.close(Duration::from_secs(10));
    json!({
        "deck": deck, "tier": tier, "seed": 7, "answers_at_save": SAVE_AFTER, "saved": saved,
        "replay": {"events": r.events, "hash": hex(r.hash), "over": r.over, "live_answers": r.given},
    })
}

// ---------------------------------------------------------------------------------------------- the test

fn compute(e: &Arc<Engine>, committed: Option<&Value>) -> Value {
    let decks = commander_decks(e);
    assert!(!decks.is_empty(), "no Commander deck in decks.json `mine`");
    json!({
        "about": "Pins Commander practice (tests/commander_practice_fixture.rs); regenerate with UPDATE_PRACTICE_FIXTURE=1",
        "decks": decks,
        "seats": seats_pin(e, &decks),
        "sessions": sessions_pin(e, &decks),
        "save": save_pin(e, &decks, committed.map(|c| &c["save"])),
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

#[test]
fn commander_practice_is_as_recorded() {
    let e = engine();
    let path = fixture_path();
    let update = std::env::var("UPDATE_PRACTICE_FIXTURE").is_ok_and(|v| v == "1");
    if update {
        let now = compute(&e, None);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&now).unwrap() + "\n").unwrap();
        eprintln!("wrote {}", path.display());
        return;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("no fixture at {} ({err}); generate it with UPDATE_PRACTICE_FIXTURE=1", path.display())
    });
    let want: Value = serde_json::from_str(&text).expect("the fixture is JSON");
    let now = compute(&e, Some(&want));
    let mut failures = vec![];
    check(&mut failures, "the Commander decks of decks.json `mine`", &want["decks"], &now["decks"]);
    check(&mut failures, "choose_seats (decks x tiers t1..t6 x seeds 1..=10)", &want["seats"], &now["seats"]);
    check(&mut failures, "the event streams of the bot-played sessions", &want["sessions"], &now["sessions"]);
    check(
        &mut failures,
        "the save file written today (session.saved())",
        &want["save"]["saved"],
        &now["save"]["saved"],
    );
    check(&mut failures, "loading and replaying the committed save", &want["save"]["replay"], &now["save"]["replay"]);
    assert!(
        failures.is_empty(),
        "Commander practice no longer behaves as recorded:\n{}\n(an intended change: regenerate with UPDATE_PRACTICE_FIXTURE=1 and review the diff)",
        failures.join("\n")
    );
}
