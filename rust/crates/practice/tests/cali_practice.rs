//! Practice mode at a Cali Commander table (2 seats, 30 life, the starting player skips its first draw, the Cali AI
//! profile): one person against one AI opponent, through the session API and the routes. The decks are found in
//! decks.json by their format, never by name.

mod common;
#[path = "session_common/mod.rs"]
mod sess;

use practice::session::setup::{SeatArgs, choose_seats, choose_seats_in, pool_keys};
use practice::session::{MY_DECKS, Options, Session};
use serde_json::{Value, json};
use sim_core::format::{DEFAULT_FORMAT, Rules, TableAi};
use sim_core::run::Engine;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(300);
const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

fn engine() -> Arc<Engine> {
    sess::engine()
}

/// a deck of yours whose own format has 2 seats, and a tier it is simulated against
fn cali_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).seats == 2).expect("a 2-seat deck in my decks");
    (k.to_string(), e.decks[*k].tiers[0].clone())
}

/// a deck of yours that plays Commander, and a tier it is simulated against
fn commander_deck(e: &Engine) -> (String, String) {
    let k = MY_DECKS.iter().find(|k| e.format_of(k).key == DEFAULT_FORMAT).expect("a Commander deck in my decks");
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

/// A person's answers: keep every hand, decline yes/no questions (no Leyline on the battlefield), play a land and tap a
/// mana source in the main phase, pass otherwise, no attacks, the first choice otherwise.
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

/// Start `s` and answer its requests with the bot until the game is over, or until `stop_after` answers have been given
/// (then the next request is the last event returned, unanswered). Every event seen.
fn play(s: &Arc<Session>, stop_after: Option<usize>) -> Vec<Value> {
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
                    if stop_after == Some(given) {
                        return all;
                    }
                    let a = bot.answer(&ev["request"]);
                    given += 1;
                    s.answer(ev["seat"].as_str().unwrap(), &a);
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

fn me(req: &Value) -> &Value {
    req["data"]["view"]["players"].as_array().unwrap().iter().find(|p| p["you"] == true).unwrap()
}

fn is_priority(ev: &Value) -> bool {
    ev["kind"] == "request" && ev["request"]["kind"] == "priority"
}

// ------------------------------------------------------------------------------------------------ the game

#[test]
fn a_cali_session_is_two_seats_under_the_cali_rules() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 1)).unwrap();
    assert_eq!((s.format.as_str(), s.seats.len()), ("cali", 2));
    assert!(s.seats.contains(&deck));
    let opp = s.seats.iter().find(|k| **k != deck).unwrap();
    assert!(e.admits(opp, "cali").is_ok(), "{opp} is not a Cali deck");
    let evs = play(&s, None);
    s.close(Duration::from_secs(10));
    // the table when turn one begins: 30 life each
    let first = evs.iter().find(|ev| ev["kind"] == "turn").expect("a turn event");
    let players = first["views"][&deck]["players"].as_array().unwrap();
    assert_eq!(players.len(), 2);
    assert!(players.iter().all(|p| p["life"] == 30), "{players:?}");
    let over = evs.last().unwrap();
    assert_eq!(over["kind"], "over");
    assert_eq!(over["view"]["players"].as_array().unwrap().len(), 2);
}

#[test]
fn the_game_is_built_with_the_formats_rules_and_ai_profile() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 1)).unwrap();
    let st = s.game_settings();
    let fmt = &e.formats["cali"];
    assert_eq!(st.rules, Rules { life: 30, first_player_draws: false });
    assert_eq!(st.table, fmt.ai);
    assert!(st.table.duel && !st.table.learned_eval, "the Cali profile: {:?}", st.table);
    // a Commander game is as it always was
    let (cdeck, ctier) = commander_deck(&e);
    let c = Session::new(options(&e, &cdeck, &ctier, 1)).unwrap();
    assert_eq!(c.format, DEFAULT_FORMAT);
    let st = c.game_settings();
    assert_eq!((st.rules, st.table), (Rules::COMMANDER, TableAi::COMMANDER));
}

/// the starting player skips its first draw, and the person is that player when seated first; seated second they draw
#[test]
fn the_person_skips_the_first_draw_on_the_play_and_draws_on_the_draw() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    // the deck's cards without its commander(s), which start in the command zone
    let d = &e.decks[&deck];
    let cards = d.cards.iter().filter(|c| **c != d.commander && Some(*c) != d.commander2.as_ref()).count() as u64;
    for (seat, hand) in [(1usize, 7u64), (2, 8)] {
        let mut o = options(&e, &deck, &tier, 5);
        o.seat = Some(seat);
        let s = Session::new(o).unwrap();
        assert_eq!(s.seats.iter().position(|k| *k == deck), Some(seat - 1));
        let evs = play(&s, None);
        s.close(Duration::from_secs(10));
        // my first look at my own first main phase
        let req = evs
            .iter()
            .filter(|ev| is_priority(ev))
            .map(|ev| &ev["request"])
            .find(|r| {
                let v = &r["data"]["view"];
                v["round"] == 1 && v["active"] == deck.as_str() && v["step"] == "main1"
            })
            .unwrap_or_else(|| panic!("seat {seat}: no first main phase of mine"));
        let m = me(req);
        assert_eq!(m["hand_count"], hand, "seat {seat}: cards in hand at my first main phase");
        assert_eq!(m["library"], cards - hand, "seat {seat}: cards in the library");
        assert_eq!(m["life"], 30);
        // and nothing before it (the draw step) asked me anything with a different hand
        let early: Vec<&Value> = evs
            .iter()
            .filter(|ev| is_priority(ev))
            .map(|ev| &ev["request"])
            .take_while(|r| *r != req)
            .filter(|r| r["data"]["view"]["round"] == 1 && r["data"]["view"]["active"] == deck.as_str())
            .collect();
        assert!(early.iter().all(|r| me(r)["hand_count"] == hand), "seat {seat}: a request before the main phase");
    }
}

#[test]
fn the_persons_mulligan_is_london_with_a_free_first_mulligan() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 3)).unwrap();
    s.start();
    let mut cursor = 0;
    let mut next = |s: &Arc<Session>| -> Value {
        let end = Instant::now() + Duration::from_secs(60);
        while Instant::now() < end {
            let (c, evs) = s.events_after(cursor, Duration::from_millis(200));
            cursor = c;
            if let Some(ev) = evs.into_iter().find(|ev| ev["kind"] == "request") {
                return ev;
            }
        }
        panic!("no request in time");
    };
    let r0 = next(&s);
    assert_eq!(r0["request"]["kind"], "mulligan");
    assert_eq!(r0["request"]["data"]["mulligans"], 0);
    s.answer(&deck, &json!("mulligan"));
    let r1 = next(&s);
    assert_eq!(r1["request"]["data"]["mulligans"], 1);
    let p1 = r1["request"]["prompt"].as_str().unwrap();
    assert!(p1.contains("(7 to keep)") && !p1.contains("bottom"), "the first mulligan is free: {p1}");
    s.answer(&deck, &json!("mulligan"));
    let r2 = next(&s);
    let p2 = r2["request"]["prompt"].as_str().unwrap();
    assert!(p2.contains("(6 to keep)") && p2.contains("1 card(s) on the bottom"), "the second costs a card: {p2}");
    s.close(Duration::from_secs(10));
}

// ------------------------------------------------------------------------------------------------ the table

#[test]
fn the_seats_of_a_cali_deck_are_two_and_admitted() {
    let e = engine();
    let (deck, _) = cali_deck(&e);
    let mut refused_seen = false;
    let mut on_the_play = [0u32; 2];
    for tier in TIERS {
        let refused: Vec<String> = pool_keys(&e, tier).into_iter().filter(|k| e.admits(k, "cali").is_err()).collect();
        refused_seen |= !refused.is_empty();
        for seed in 1..=40u64 {
            let a = SeatArgs { deck: &deck, tier, partner: None, seed, seat: None, opponents: None, seats: None };
            let seats = choose_seats(&e, &a).unwrap();
            assert_eq!(seats.len(), 2, "{tier} {seed}: {seats:?}");
            let opp = seats.iter().find(|k| **k != deck).expect("an opponent");
            assert_eq!(seats.iter().filter(|k| **k == deck).count(), 1);
            assert!(pool_keys(&e, tier).contains(opp) && e.admits(opp, "cali").is_ok(), "{tier} {seed}: {opp}");
            assert!(!refused.contains(opp));
            assert_eq!(choose_seats(&e, &a).unwrap(), seats, "{tier} {seed}: not deterministic");
            on_the_play[seats.iter().position(|k| *k == deck).unwrap()] += 1;
        }
    }
    assert!(refused_seen, "no pool deck is refused by Cali: the exclusion goes untested");
    assert!(on_the_play.iter().all(|&n| n > 40), "the seat should be random: {on_the_play:?}");
}

#[test]
fn a_cali_table_seat_pick_opponents_and_exact_seats() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let args =
        |seat, opponents, seats| SeatArgs { deck: &deck, tier: &tier, partner: None, seed: 9, seat, opponents, seats };
    let natural = choose_seats(&e, &args(None, None, None)).unwrap();
    let opp = natural.iter().find(|k| **k != deck).unwrap().clone();
    assert_eq!(choose_seats(&e, &args(Some(1), None, None)).unwrap(), [deck.clone(), opp.clone()]);
    assert_eq!(choose_seats(&e, &args(Some(2), None, None)).unwrap(), [opp.clone(), deck.clone()]);
    assert_eq!(choose_seats(&e, &args(Some(3), None, None)).unwrap_err(), "seat is 1 to 2");
    let one = [opp.clone()];
    assert!(choose_seats(&e, &args(None, Some(&one), None)).unwrap().contains(&opp));
    let three = vec![opp.clone(), "a".to_string(), "b".to_string()];
    assert!(choose_seats(&e, &args(None, Some(&three), None)).unwrap_err().starts_with("pick one of: "));
    let exact = [opp.clone(), deck.clone()];
    assert_eq!(choose_seats(&e, &args(None, None, Some(&exact))).unwrap(), exact);
    let four = vec![opp.clone(), deck.clone(), "a".to_string(), "b".to_string()];
    assert!(choose_seats(&e, &args(None, None, Some(&four))).unwrap_err().starts_with("seats: your deck and 1 of "));
}

#[test]
fn a_four_seat_table_is_chosen_as_before() {
    let e = engine();
    let (deck, tier) = commander_deck(&e);
    for seed in 1..=10u64 {
        let a = SeatArgs { deck: &deck, tier: &tier, partner: None, seed, seat: None, opponents: None, seats: None };
        let seats = choose_seats(&e, &a).unwrap();
        assert_eq!(seats.len(), 4);
        assert_eq!(choose_seats_in(&e, &e.formats[DEFAULT_FORMAT], &a).unwrap(), seats);
        let seat = SeatArgs { seat: Some(4), ..a };
        assert_eq!(choose_seats(&e, &seat).unwrap()[3], deck);
        let seat = SeatArgs { seat: Some(5), ..seat };
        assert_eq!(choose_seats(&e, &seat).unwrap_err(), "seat is 1 to 4");
    }
}

#[test]
fn the_format_is_checked() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let (cdeck, ctier) = commander_deck(&e);
    // unknown format
    let mut o = options(&e, &deck, &tier, 1);
    o.format = Some("nope".into());
    assert!(Session::new(o).err().unwrap().starts_with("unknown format \"nope\": one of "));
    // a Cali deck is refused at a Commander table, with the reason
    let mut o = options(&e, &deck, &tier, 1);
    o.format = Some(DEFAULT_FORMAT.into());
    let why = Session::new(o).err().unwrap();
    assert!(why.contains("cannot be played at a Commander table"), "{why}");
    // a Commander deck may sit at a Cali table (its list holds nothing Cali bans): 2 seats
    if e.admits(&cdeck, "cali").is_ok() {
        let mut o = options(&e, &cdeck, &ctier, 1);
        o.format = Some("cali".into());
        let s = Session::new(o).unwrap();
        assert_eq!((s.format.as_str(), s.seats.len()), ("cali", 2));
        assert_eq!(s.game_settings().rules.life, 30);
    }
    // a partner is refused at a 2-seat table
    let mut o = options(&e, &deck, &tier, 1);
    o.partner = Some(cdeck.clone());
    let why = Session::new(o).err().unwrap();
    assert!(why.contains("4-seat format"), "{why}");
    let mut o = options(&e, &cdeck, &ctier, 1);
    o.format = Some("cali".into());
    o.partner = Some(deck.clone());
    assert!(Session::new(o).err().unwrap().contains("4-seat format"));
}

// ------------------------------------------------------------------------------------------------ saves

/// the last request of the events: where the game waits
fn waiting(evs: &[Value]) -> &Value {
    evs.iter().rev().find(|ev| ev["kind"] == "request").expect("a request")
}

#[test]
fn a_cali_save_carries_its_format_and_reloads_to_the_same_decision() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 7)).unwrap();
    let evs = play(&s, Some(20));
    let saved = s.saved();
    s.close(Duration::from_secs(10));
    assert_eq!(saved["format"], "cali");
    assert_eq!(saved["version"], 1);
    let text = serde_json::to_string(&saved).unwrap();
    let l = Session::load(e.clone(), &serde_json::from_str(&text).unwrap(), true, false).unwrap();
    assert_eq!((l.format.as_str(), l.seats.clone()), ("cali", s.seats.clone()));
    let again = play(&l, Some(0));
    l.close(Duration::from_secs(10));
    assert_eq!(waiting(&again)["request"], waiting(&evs)["request"], "the loaded game waits on the same decision");
    assert_eq!(waiting(&again)["seat"], waiting(&evs)["seat"]);
}

#[test]
fn a_commander_save_has_no_format_and_an_old_save_loads_as_commander() {
    let e = engine();
    let (deck, tier) = commander_deck(&e);
    let s = Session::new(options(&e, &deck, &tier, 7)).unwrap();
    let evs = play(&s, Some(12));
    let saved = s.saved();
    s.close(Duration::from_secs(10));
    assert!(saved.get("format").is_none(), "a Commander save is written as it always was");
    let l = Session::load(e.clone(), &saved, true, false).unwrap();
    assert_eq!((l.format.as_str(), l.seats.len()), (DEFAULT_FORMAT, 4));
    let again = play(&l, Some(0));
    l.close(Duration::from_secs(10));
    assert_eq!(waiting(&again)["request"], waiting(&evs)["request"]);
}

#[test]
fn a_save_of_a_cali_deck_without_a_format_is_refused_not_converted() {
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    // what a save made before formats looked like for this deck: a four-seat table, no `format`
    let (cdeck, _) = commander_deck(&e);
    let pool = pool_keys(&e, &tier);
    let mut seats: Vec<String> = pool.iter().take(3).cloned().collect();
    seats.push(deck.clone());
    let mut old = json!({
        "version": 1, "deck": deck, "tier": tier, "seed": 3, "seats": seats, "profile": "loose", "ai": "adaptive",
        "max_rounds": 3, "answers": [], "tape": [], "shadow": [], "finished": false, "round": 0,
        "autopass": {}, "autopass_log": [],
    });
    let why = Session::load(e.clone(), &old, true, false).err().expect("refused");
    assert!(why.contains("no format") && why.contains("Commander") && why.contains("cannot be played"), "{why}");
    // a format that does not exist
    old["format"] = "nope".into();
    assert!(Session::load(e.clone(), &old, true, false).err().unwrap().starts_with("unknown format"));
    // and the same save of a Commander deck loads
    let mut ok = old.clone();
    ok.as_object_mut().unwrap().remove("format");
    ok["deck"] = cdeck.clone().into();
    let tier = e.decks[&cdeck].tiers[0].clone();
    ok["tier"] = tier.clone().into();
    let mut seats: Vec<String> = pool_keys(&e, &tier).into_iter().take(3).collect();
    seats.push(cdeck.clone());
    ok["seats"] = json!(seats);
    assert_eq!(Session::load(e.clone(), &ok, true, false).unwrap().format, DEFAULT_FORMAT);
}

// ------------------------------------------------------------------------------------------------ the catalog

#[test]
fn the_catalog_lists_the_formats_and_what_each_deck_can_play() {
    let e = engine();
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let cat = practice::catalog::catalog(&data).unwrap();
    let keys: Vec<&str> = cat["formats"].as_array().unwrap().iter().map(|f| f["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["commander", "cali"]);
    let cali = &cat["formats"][1];
    assert_eq!(
        (cali["seats"].as_u64(), cali["life"].as_u64(), cali["first_player_draws"].as_bool()),
        (Some(2), Some(30), Some(false))
    );
    assert_eq!(cat["formats"][0]["seats"], 4);
    // every deck: its own format, and the formats that admit it, as the engine says
    let expect = |key: &str| -> Vec<String> {
        ["commander", "cali"].iter().filter(|f| e.admits(key, f).is_ok()).map(|f| f.to_string()).collect()
    };
    let list =
        |v: &Value| -> Vec<String> { v.as_array().unwrap().iter().map(|k| k.as_str().unwrap().to_string()).collect() };
    for d in cat["decks"].as_array().unwrap() {
        let k = d["key"].as_str().unwrap();
        assert_eq!(d["format"], e.format_of(k).key.as_str(), "{k}");
        assert_eq!(list(&d["formats"]), expect(k), "{k}");
        // the win rates are four-player numbers: marked, and none for a deck that does not play Commander
        assert_eq!(d["win_rates_format"], "commander");
        if e.format_of(k).key != DEFAULT_FORMAT {
            assert!(d["win_rates"].is_null() && d["average"].is_null(), "{k}");
        }
    }
    assert_eq!(cat["win_rates_format"], "commander");
    let (deck, _) = cali_deck(&e);
    let d = cat["decks"].as_array().unwrap().iter().find(|d| d["key"] == deck.as_str()).unwrap();
    assert_eq!(list(&d["formats"]), ["cali"]);
    let (cdeck, _) = commander_deck(&e);
    let d = cat["decks"].as_array().unwrap().iter().find(|d| d["key"] == cdeck.as_str()).unwrap();
    assert!(list(&d["formats"]).contains(&"commander".to_string()));
    // the tier decks: Cali lists the ones it admits, and not the one holding a banned card
    let mut refused = 0;
    for t in cat["tiers"].as_array().unwrap() {
        for d in t["decks"].as_array().unwrap() {
            let k = d["key"].as_str().unwrap();
            assert_eq!(list(&d["formats"]), expect(k), "{k}");
            refused += usize::from(!list(&d["formats"]).contains(&"cali".to_string()));
        }
    }
    assert!(refused > 0, "no tier deck is refused by Cali");
}

// ------------------------------------------------------------------------------------------------ routes

macro_rules! srv {
    () => {
        match common::server() {
            Some(s) => s,
            None => return,
        }
    };
}

#[test]
fn the_routes_check_the_format() {
    let s = srv!();
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let (cdeck, ctier) = commander_deck(&e);
    let mut c = s.client();
    let new = |c: &mut common::Client, body: Value| {
        let r = c.post("/api/new", body);
        (r.status, r.error())
    };
    // two people at a 2-seat table: not here
    let (st, msg) = new(&mut c, json!({"deck": deck, "tier": tier, "two": true}));
    assert_eq!(st, 400, "{msg}");
    assert!(msg.contains("2 seats"), "{msg}");
    let (st, msg) = new(&mut c, json!({"deck": cdeck, "tier": ctier, "format": "cali", "two": true}));
    assert_eq!(st, 400, "{msg}");
    // unknown, not a string, and one that does not admit the deck
    let (st, msg) = new(&mut c, json!({"deck": deck, "tier": tier, "format": "nope"}));
    assert_eq!((st, msg.as_str()), (400, "format: one of commander, cali"));
    assert_eq!(new(&mut c, json!({"deck": deck, "tier": tier, "format": 4})).0, 400);
    let (st, msg) = new(&mut c, json!({"deck": deck, "tier": tier, "format": "commander"}));
    assert_eq!(st, 400);
    assert!(msg.contains("cannot be played at a Commander table"), "{msg}");
    // the texts name the table's seats
    let (st, msg) = new(&mut c, json!({"deck": deck, "tier": tier, "seat": "2"}));
    assert_eq!((st, msg.as_str()), (400, "seat: 1 to 2"));
    let (st, msg) = new(&mut c, json!({"deck": deck, "tier": tier, "seats": [1]}));
    assert_eq!((st, msg.as_str()), (400, "seats: two deck keys in turn order"));
    // a Commander deck at a Cali table says 2 too
    let (st, msg) = new(&mut c, json!({"deck": cdeck, "tier": ctier, "format": "cali", "seat": "x"}));
    assert_eq!((st, msg.as_str()), (400, "seat: 1 to 2"));
    // while a Commander deck's texts are as they were
    let (st, msg) = new(&mut c, json!({"deck": cdeck, "tier": ctier, "seats": [1, 2]}));
    assert_eq!((st, msg.as_str()), (400, "seats: four deck keys in turn order"));
    let (st, msg) = new(&mut c, json!({"deck": cdeck, "tier": ctier, "seat": "x"}));
    assert_eq!((st, msg.as_str()), (400, "seat: 1 to 4"));
}

#[test]
fn a_cali_game_through_the_routes_is_named_by_its_format() {
    let s = srv!();
    let e = engine();
    let (deck, tier) = cali_deck(&e);
    let mut c = s.client();
    let r = c.post(
        "/api/new",
        json!({"deck": deck, "tier": tier, "seed": 11, "ai": "adaptive", "images": false, "tools": {"compare": false}}),
    );
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(r.json()["seats"].as_array().unwrap().len(), 2);
    let mull = common::wait_for(&mut c, 0, common::is_request);
    assert_eq!(mull["request"]["kind"], "mulligan");
    let state = c.get("/api/state").json();
    assert_eq!(
        (state["game"]["format"].as_str(), state["game"]["seats"].as_array().map(Vec::len)),
        (Some("cali"), Some(2))
    );
    let r = c.post("/api/save", json!({}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let name = r.json()["name"].as_str().unwrap().to_string();
    assert!(name.contains(&format!("-{deck}-cali-{tier}-11")), "{name}");
    let file = s.save_file(&name).expect("the save file");
    assert_eq!(file["format"], "cali");
    let list = c.get("/api/saves").json();
    let row = list["saves"].as_array().unwrap().iter().find(|x| x["name"] == name.as_str()).expect("listed");
    assert_eq!(row["format"], "cali");
    // it loads again as a Cali game
    let r = c.post("/api/load", json!({"name": name, "images": false}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    common::wait_for(&mut c, 0, common::is_request);
    assert_eq!(c.get("/api/state").json()["game"]["format"], "cali");
    c.quit();
    s.remove_save(&name);
}

// ------------------------------------------------------------------------------------------------ the terminal client

/// `--format cali` is an argument the terminal client accepts and hands to the session (no game is played: the usage
/// text and the argument loop are what is run)
#[test]
fn the_terminal_client_takes_format() {
    let exe = env!("CARGO_BIN_EXE_commander-practice");
    let help = std::process::Command::new(exe).arg("--help").output().unwrap();
    assert!(String::from_utf8_lossy(&help.stdout).contains("--format KEY"));
    // a seat beyond 4 is no longer refused by the argument check (the session checks it against the table): it gets as
    // far as loading the card data, which a missing --data dir stops
    let out = std::process::Command::new(exe)
        .args(["--text", "--format", "cali", "--seat", "2", "--data", "/nonexistent-practice-data"])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("bad value") && !err.contains("unrecognized argument"), "{err}");
    let out = std::process::Command::new(exe).args(["--text", "--format"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stderr).contains("--format needs a value"));
}
