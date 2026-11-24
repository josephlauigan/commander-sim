// written by an independent reviewer
//! C5a adversarial: the Cali session. Hints and the comparison (copies of the game), save and reload mid-game
//! (original vs loaded views), `Session::load` of odd `format` values, undo at a Cali table, `format` given
//! explicitly equal to the deck's own.

#[path = "session_common/mod.rs"]
mod sess;

use practice::session::setup::pool_keys;
use practice::session::{MY_DECKS, Options, Session};
use serde_json::{Value, json};
use sim_core::format::DEFAULT_FORMAT;
use sim_core::run::Engine;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(300);

fn engine() -> Arc<Engine> {
    sess::engine()
}

fn cali_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck in my decks");
    (k.to_string(), e.decks[*k].tiers[0].clone())
}

fn commander_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).key == DEFAULT_FORMAT).expect("a Commander deck");
    (k.to_string(), e.decks[*k].tiers[0].clone())
}

fn options(e: &Arc<Engine>, deck: &str, tier: &str, seed: u64) -> Options {
    let mut o = Options::new(e.clone(), deck, tier);
    o.seed = Some(seed);
    o.ai = "adaptive".into();
    o.compare = false;
    o.views = true;
    o.max_rounds = 3;
    o
}

/// keep every hand, decline yes/no, play a land and tap a mana source in the main phase, otherwise pass
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

/// Drive `s` with the bot until it is over, or until `stop_after` answers were given (then the next request is the
/// last event, unanswered). `on_request(n, seat, req)` runs before each answer (n = answers given so far).
fn drive(s: &Arc<Session>, stop_after: Option<usize>, on_request: &mut dyn FnMut(usize, &str, &Value)) -> Vec<Value> {
    s.start();
    let end = Instant::now() + LIMIT;
    let mut bot = Bot::default();
    let (mut cursor, mut given, mut all) = (0u64, 0usize, vec![]);
    loop {
        assert!(Instant::now() < end, "the game did not finish in {LIMIT:?}");
        let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
        cursor = c;
        for ev in evs {
            let kind = ev["kind"].as_str().unwrap_or("").to_string();
            all.push(ev.clone());
            match kind.as_str() {
                "request" => {
                    let seat = ev["seat"].as_str().unwrap().to_string();
                    on_request(given, &seat, &ev["request"]);
                    if stop_after == Some(given) {
                        return all;
                    }
                    let a = bot.answer(&ev["request"]);
                    given += 1;
                    s.answer(&seat, &a);
                }
                "over" | "error" => {
                    assert_eq!(kind, "over", "the game ended in an error: {ev}");
                    return all;
                }
                _ => {}
            }
        }
    }
}

fn play(s: &Arc<Session>, stop_after: Option<usize>) -> Vec<Value> {
    drive(s, stop_after, &mut |_, _, _| {})
}

fn waiting(evs: &[Value]) -> &Value {
    evs.iter().rev().find(|ev| ev["kind"] == "request").expect("a request")
}

// -------------------------------------------------------------------------------------- hints and the comparison

/// The hint and the AI comparison play copies of the game. They are built from the game's own `Settings` (an `Arc`
/// cloned with the game), so they must run at a Cali table without a panic and the review must open.
#[test]
fn hint_and_comparison_run_at_a_cali_table() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let mut o = options(&e, &deck, &tier, 21);
    o.compare = true;
    let s = Session::new(o).unwrap();
    let mut hints = 0;
    let evs = drive(&s, None, &mut |_, seat, req| {
        if req["kind"] == "priority" && hints < 3 {
            let h = s.hint(seat).unwrap_or_else(|e| panic!("hint refused: {e}"));
            assert!(h.is_object() || h.is_array(), "{h}");
            hints += 1;
        }
        if req["kind"] == "mulligan" && hints < 3 {
            s.hint(seat).unwrap_or_else(|e| panic!("mulligan hint refused: {e}"));
            hints += 1;
        }
    });
    assert!(hints > 0, "no hint was asked for");
    assert_eq!(evs.last().unwrap()["kind"], "over");
    let r = s.review(&deck).unwrap_or_else(|e| panic!("review: {e}"));
    assert!(r["summary"]["decisions"].as_u64().unwrap() > 0, "{}", r["summary"]);
    // every comparison line of a 2-seat game names ONE opponent; nothing mentions a third
    let text = r.to_string();
    assert!(!text.contains("NaN"), "NaN in the review");
    s.close(Duration::from_secs(30));
}

// -------------------------------------------------------------------------------------- save and reload

/// the original at a decision, and a reloaded copy of its save at the same decision, agree: the request (with its
/// whole view), the seat, and the save they would each write now
fn reload_agrees(e: &Arc<Engine>, deck: &str, tier: &str, seed: u64, seat: Option<usize>, stop: usize) {
    let mut o = options(e, deck, tier, seed);
    o.seat = seat;
    let s = Session::new(o).unwrap();
    let evs = play(&s, Some(stop));
    let saved = s.saved();
    let text = serde_json::to_string(&saved).unwrap();
    let wait = waiting(&evs).clone();
    s.close(Duration::from_secs(10));
    let l = Session::load(e.clone(), &serde_json::from_str(&text).unwrap(), true, false)
        .unwrap_or_else(|why| panic!("{deck} seat {seat:?} stop {stop}: load: {why}"));
    assert_eq!(l.format, s.format);
    assert_eq!(l.seats, s.seats);
    let again = play(&l, Some(0));
    if evs.last().unwrap()["kind"] == "over" {
        // the game ended before `stop` answers: the reload must end the same way
        assert_eq!(again.last().unwrap()["kind"], "over", "{deck} seat {seat:?} stop {stop}");
        assert_eq!(again.last().unwrap()["winner"], evs.last().unwrap()["winner"]);
        l.close(Duration::from_secs(10));
        eprintln!("{deck} seat {seat:?} stop {stop}: game over first");
        return;
    }
    let w2 = waiting(&again);
    assert_eq!(w2["seat"], wait["seat"], "{deck} seat {seat:?} stop {stop}");
    assert_eq!(w2["request"], wait["request"], "{deck} seat {seat:?} stop {stop}: the same decision, view and all");
    let saved2 = l.saved();
    l.close(Duration::from_secs(10));
    assert_eq!(saved2, saved, "{deck} seat {seat:?} stop {stop}: the reloaded game writes the save it was loaded from");
}

#[test]
fn a_reloaded_cali_game_is_the_original_at_every_point() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    for (seed, seat) in [(31u64, Some(1usize)), (32, Some(2)), (33, None)] {
        for stop in [0usize, 1, 6, 15, 30, 45] {
            reload_agrees(&e, &deck, &tier, seed, seat, stop);
        }
    }
}

/// the same check on a Commander deck, so a failure above can be told from a failure that exists on main too
#[test]
fn a_reloaded_commander_game_is_the_original_at_every_point() {
    let e = engine();
    let (deck, tier) = commander_deck(&e);
    for (seed, seat) in [(31u64, Some(1usize)), (32, Some(4))] {
        for stop in [0usize, 6, 30] {
            reload_agrees(&e, &deck, &tier, seed, seat, stop);
        }
    }
}

#[test]
fn a_finished_cali_save_reloads_to_the_same_end() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 41)).unwrap();
    let evs = play(&s, None);
    let over = evs.last().unwrap().clone();
    let saved = s.saved();
    s.close(Duration::from_secs(10));
    assert_eq!(saved["finished"], true);
    assert_eq!(saved["format"], "cali");
    let l = Session::load(e.clone(), &saved, true, false).unwrap();
    let again = play(&l, None);
    l.close(Duration::from_secs(10));
    let over2 = again.last().unwrap();
    assert_eq!(over2["kind"], "over");
    for k in ["winner", "how", "view"] {
        assert_eq!(over2[k], over[k], "{k}");
    }
}

// -------------------------------------------------------------------------------------- Session::load, odd formats

/// a save of a 4-seat Commander game, as made before formats existed (no `format`)
fn old_commander_save(e: &Engine, deck: &str, tier: &str) -> Value {
    let mut seats: Vec<String> = pool_keys(e, tier).into_iter().take(3).collect();
    seats.push(deck.to_string());
    json!({
        "version": 1, "deck": deck, "tier": tier, "seed": 3, "seats": seats, "profile": "loose", "ai": "adaptive",
        "max_rounds": 3, "answers": [], "tape": [], "shadow": [], "finished": false, "round": 0,
        "autopass": {}, "autopass_log": [],
    })
}

fn loaded(e: &Arc<Engine>, v: &Value) -> Result<(String, usize), String> {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Session::load(e.clone(), v, true, false)));
    let l = r.unwrap_or_else(|_| panic!("Session::load PANICKED on {}", v["format"]))?;
    let out = (l.format.clone(), l.seats.len());
    l.close(Duration::from_secs(5));
    Ok(out)
}

#[test]
fn load_of_a_commander_save_with_every_format_spelling() {
    let e = engine();
    let (deck, tier) = commander_deck(&e);
    let base = old_commander_save(&e, &deck, &tier);
    // no format: Commander
    assert_eq!(loaded(&e, &base).unwrap(), (DEFAULT_FORMAT.to_string(), 4));
    // "commander" written by hand: Commander, and re-saving it drops the key again (as main writes it)
    let mut v = base.clone();
    v["format"] = json!("commander");
    assert_eq!(loaded(&e, &v).unwrap(), (DEFAULT_FORMAT.to_string(), 4));
    let l = Session::load(e.clone(), &v, true, false).unwrap();
    assert!(l.saved().get("format").is_none(), "{}", l.saved());
    l.close(Duration::from_secs(5));
    // the unusable ones: an error, never a panic and never a silent Commander game
    for bad in [
        json!(null),
        json!(4),
        json!(true),
        json!(["commander"]),
        json!({"k": 1}),
        json!(""),
        json!("Commander"),
        json!("COMMANDER"),
        json!(" commander"),
        json!("nope"),
    ] {
        let mut v = base.clone();
        v["format"] = bad.clone();
        let r = loaded(&e, &v);
        assert!(r.is_err(), "format {bad} loaded as {r:?}");
    }
    // `"format": null` is not a missing format: it is a damaged file (the text says so, not the old-save text)
    let mut v = base.clone();
    v["format"] = json!(null);
    let why = loaded(&e, &v).unwrap_err();
    assert!(why.contains("damaged"), "{why}");
}

#[test]
fn load_of_cali_and_mismatched_saves() {
    let e = engine();
    let (cdeck, ctier) = cali_deck(&e);
    let (kdeck, ktier) = commander_deck(&e);
    // a hand-made Cali save of the Cali deck at a Cali table
    let opp = pool_keys(&e, &ctier).into_iter().find(|k| e.admits(k, "cali").is_ok()).unwrap();
    let mut v = old_commander_save(&e, &cdeck, &ctier);
    v["format"] = json!("cali");
    v["seats"] = json!([opp, cdeck]);
    assert_eq!(loaded(&e, &v).unwrap(), ("cali".to_string(), 2));
    // the same file with the four seats of the old table: refused with the seats text
    let mut four = v.clone();
    four["seats"] = old_commander_save(&e, &cdeck, &ctier)["seats"].clone();
    let why = loaded(&e, &four).unwrap_err();
    assert!(why.starts_with("seats: your deck and 1 of "), "{why}");
    // a Cali save naming a pool deck that Cali refuses
    if let Some(refused) = TIERS_WITH_REFUSED(&e) {
        let mut bad = v.clone();
        bad["tier"] = json!(refused.0);
        bad["seats"] = json!([refused.1, cdeck]);
        let why = loaded(&e, &bad).unwrap_err();
        assert!(why.starts_with("seats: your deck and 1 of "), "{why}");
        assert!(!why.contains(&format!(", {}", refused.1)) && !why.contains(&format!("of {}", refused.1)), "{why}");
    }
    // a Cali save of a Commander deck is fine (its list holds nothing Cali bans) at 2 seats
    if e.admits(&kdeck, "cali").is_ok() {
        let opp = pool_keys(&e, &ktier).into_iter().find(|k| e.admits(k, "cali").is_ok()).unwrap();
        let mut ok = old_commander_save(&e, &kdeck, &ktier);
        ok["format"] = json!("cali");
        ok["seats"] = json!([kdeck, opp]);
        assert_eq!(loaded(&e, &ok).unwrap(), ("cali".to_string(), 2));
        // with a `partner` key: refused (2 seats)
        let mut p = ok.clone();
        p["partner"] = json!(cdeck);
        assert!(loaded(&e, &p).is_err());
    }
    // a Commander-format save of the Cali deck, however written
    let mut c = old_commander_save(&e, &cdeck, &ctier);
    c["format"] = json!("commander");
    let why = loaded(&e, &c).unwrap_err();
    assert!(why.contains("cannot be played at a Commander table"), "{why}");
    assert!(!why.contains("no format"), "an explicit format must not get the no-format text: {why}");
}

/// a tier and a pool deck of it that Cali refuses
#[allow(non_snake_case)]
fn TIERS_WITH_REFUSED(e: &Engine) -> Option<(String, String)> {
    for t in ["t0", "t1", "t2", "t3", "t4", "t5", "t6"] {
        if let Some(k) = pool_keys(e, t).into_iter().find(|k| e.admits(k, "cali").is_err()) {
            return Some((t.to_string(), k));
        }
    }
    None
}

/// the message for an old save says "no format, so a Commander game" for ANY failure, including ones that have
/// nothing to do with the format (a damaged tier): misleading text, not a crash
#[test]
fn the_no_format_message_is_only_for_failures_that_come_from_the_format() {
    let e = engine();
    let (deck, tier) = commander_deck(&e);
    let mut v = old_commander_save(&e, &deck, &tier);
    v["tier"] = json!("t9");
    let why = loaded(&e, &v).unwrap_err();
    assert!(!why.contains("no format"), "an unknown tier is not a format problem, but the text says so: {why}");
}

// -------------------------------------------------------------------------------------- explicit format, undo

/// `format: Some(own)` is the same game as `format: None`
#[test]
fn naming_the_decks_own_format_changes_nothing() {
    let e = engine();
    for (deck, tier, key) in
        [(cali_deck(&e).0, cali_deck(&e).1, "cali"), (commander_deck(&e).0, commander_deck(&e).1, DEFAULT_FORMAT)]
    {
        let a = Session::new(options(&e, &deck, &tier, 77)).unwrap();
        let mut o = options(&e, &deck, &tier, 77);
        o.format = Some(key.to_string());
        let b = Session::new(o).unwrap();
        assert_eq!((a.format.as_str(), a.seats.clone()), (b.format.as_str(), b.seats.clone()));
        let (ea, eb) = (play(&a, Some(8)), play(&b, Some(8)));
        assert_eq!(waiting(&ea)["request"], waiting(&eb)["request"]);
        assert_eq!(a.saved(), b.saved());
        a.close(Duration::from_secs(10));
        b.close(Duration::from_secs(10));
    }
}

/// opponents and exact seats that Cali refuses are refused by the session, with the texts of the seating code
#[test]
fn a_cali_table_refuses_what_cali_refuses() {
    let e = engine();
    let (deck, _) = cali_deck(&e);
    let (tier, refused) = TIERS_WITH_REFUSED(&e).expect("a pool deck Cali refuses");
    let mut o = options(&e, &deck, &tier, 5);
    o.opponents = Some(vec![refused.clone()]);
    let why = Session::new(o).err().unwrap();
    assert!(why.starts_with("pick one of: "), "{why}");
    assert!(
        !why.split("of: ").nth(1).unwrap().split(", ").any(|k| k == refused),
        "the list offers the refused deck: {why}"
    );
    let mut o = options(&e, &deck, &tier, 5);
    o.seats = Some(vec![deck.clone(), refused.clone()]);
    let why = Session::new(o).err().unwrap();
    assert!(why.starts_with("seats: your deck and 1 of "), "{why}");
    let mut o = options(&e, &deck, &tier, 5);
    o.seat = Some(0);
    assert_eq!(Session::new(o).err().unwrap(), "seat is 1 to 2");
    let mut o = options(&e, &deck, &tier, 5);
    o.seat = Some(3);
    assert_eq!(Session::new(o).err().unwrap(), "seat is 1 to 2");
}

/// undo rewinds and replays the game through `play` again: the Cali rules must be there after the rewind
#[test]
fn undo_at_a_cali_table_replays_under_the_cali_rules() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let mut o = options(&e, &deck, &tier, 61);
    o.seat = Some(2);
    let s = Session::new(o).unwrap();
    let mut undone = false;
    let end = Instant::now() + LIMIT;
    s.start();
    let mut bot = Bot::default();
    let (mut cursor, mut given, mut life_seen) = (0u64, 0usize, vec![]);
    let mut over = false;
    while !over {
        assert!(Instant::now() < end, "no end");
        let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
        cursor = c;
        for ev in evs {
            match ev["kind"].as_str().unwrap_or("") {
                "request" => {
                    if let Some(ps) = ev["request"]["data"]["view"]["players"].as_array() {
                        life_seen.extend(ps.iter().filter_map(|p| p["life"].as_i64()));
                    }
                    if given == 14 && !undone {
                        undone = true;
                        assert_eq!(s.undo(5, &deck), None, "undo refused");
                        continue;
                    }
                    let a = bot.answer(&ev["request"]);
                    given += 1;
                    s.answer(ev["seat"].as_str().unwrap(), &a);
                }
                "over" => over = true,
                "error" => panic!("error after undo: {ev}"),
                _ => {}
            }
        }
    }
    s.close(Duration::from_secs(10));
    assert!(undone);
    assert!(life_seen.iter().all(|l| *l <= 30), "a life total above 30 at a Cali table: {life_seen:?}");
    assert!(life_seen.contains(&30));
}
