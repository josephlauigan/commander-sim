//! C4 of the Cali Commander work: the AI reads a small format profile (`TableAi`). These tests pin the half that must
//! not move: under the Commander profile every deck plays exactly as before (`go_rules == is_t6`, the learned
//! evaluation unchanged, no duel cover, no new stats), and the new rules apply only when the profile says so. No test
//! names the Cali deck: a table's profile is set on the game directly.

use sim_core::ai::{learned, plan_data, t6_table, values};
use sim_core::format::{Format, GoRules, TableAi};
use sim_core::ids::PlayerId;
use sim_core::impls::{combos, t6_plans};
use sim_core::settings::{AiMode, Profile, Settings};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);
const ORACLE: &str = "Oracle + Demonic Consultation";

/// the Cali profile as `[cali.ai]` will say it (the data lands with the format record; the tests set it directly)
const CALI: TableAi = TableAi { learned_eval: false, go_rules: GoRules::All, duel: true };
/// go rules for every seat, nothing else changed
const GO_ALL: TableAi = TableAi { go_rules: GoRules::All, ..TableAi::COMMANDER };

fn all_keys() -> Vec<String> {
    let mut keys: Vec<String> = decks().iter().map(|d| d.key.clone()).collect();
    keys.sort();
    keys.dedup();
    keys
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn stat(g: &Game, p: PlayerId, k: &str) -> i64 {
    g.player(p).stats.get(k).copied().unwrap_or(0)
}

fn offered(g: &mut Game, p: PlayerId) -> bool {
    combos::combo_options(g, p, 0.0, Some(false))
        .unwrap()
        .iter()
        .any(|o| o.label.starts_with("combo: ") && o.label.contains(ORACLE))
}

/// `key` at P0 with Thassa's Oracle + Demonic Consultation ready ({1}{U}{U}{B} up, spare Plains) and `extra` in hand
/// besides; the other seats hold open blue mana, counters and cantrips when `armed`. `seats` 2 or 4.
fn position(key: &str, extra: &[&str], armed: bool, seats: usize) -> Game {
    let keys = ["sisay-legends-toolbox", "yshtola", "sauron"];
    let mut ks = vec![key];
    ks.extend(&keys[..seats - 1]);
    let mut g = table_with(&ks, 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Plains", 2, false);
    let mut h = vec!["Thassa's Oracle", "Demonic Consultation"];
    h.extend(extra);
    hand(&mut g, P0, &h);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    if armed {
        for q in (1..seats).map(|i| PlayerId(i as u8)) {
            lands(&mut g, q, "Island", 4, false);
            hand(&mut g, q, &["Force of Will", "Counterspell", "Swan Song", "Brainstorm", "Opt"]);
        }
    }
    g
}

// ------------------------------------------------------------------ the default profile is Commander
#[test]
fn every_settings_constructor_starts_on_the_commander_profile() {
    for profile in [Profile::Conservative, Profile::Loose] {
        for ai in [AiMode::Adaptive, AiMode::Lookahead] {
            assert_eq!(Settings::new(profile, ai, 1.0).table, TableAi::COMMANDER);
        }
    }
    assert_eq!(TableAi::default(), TableAi::COMMANDER);
    assert_eq!(Format::commander().ai, TableAi::COMMANDER);
    assert_eq!(Format::commander().ai, Settings::new(Profile::Loose, AiMode::Adaptive, 1.0).table);
    assert_eq!(TableAi::COMMANDER, TableAi { learned_eval: true, go_rules: GoRules::T6, duel: false });
    // the testkit tables (every test game) are Commander's
    for g in [
        table(&["yshtola", "sauron"]),
        table_with(&["yshtola", "sauron"], 3, 30),
        play(&["sauron", "yshtola"], 2, false),
    ] {
        assert_eq!(g.settings.table, TableAi::COMMANDER);
    }
}

#[test]
fn under_the_default_profile_go_rules_is_is_t6_and_the_learned_eval_is_unchanged_for_every_deck() {
    let ai_eval_set = std::env::var("AI_EVAL").ok().is_some_and(|v| !v.trim().is_empty());
    for k in all_keys() {
        let g = table_with(&[&k, "sisay-legends-toolbox", "yshtola", "sauron"], 1, 40);
        for i in 0..4 {
            let p = PlayerId(i);
            assert_eq!(t6_plans::go_rules(&g, p), t6_plans::is_t6(&g, p), "{k} at seat {i}");
            assert!(!g.settings.table.duel, "{k}");
            if !ai_eval_set {
                // `AI_EVAL` unset: the learned evaluation is on for exactly the generic-policy decks, as before
                assert_eq!(learned::on(&g, p), values::generic_deck(&g, p), "{k} at seat {i}");
            }
        }
    }
}

#[test]
fn the_cali_profile_switches_the_learned_eval_off_everywhere_and_go_rules_on_everywhere() {
    for k in all_keys() {
        let mut g = table_with(&[&k, "sisay-legends-toolbox"], 1, 30);
        set_table(&mut g, CALI);
        for i in 0..2 {
            let p = PlayerId(i);
            assert!(!learned::on(&g, p), "{k} at seat {i}: learned off");
            assert!(t6_plans::go_rules(&g, p), "{k} at seat {i}: every seat plays the go rules");
        }
        // the reverse: only the go rules, the learned evaluation as it was
        set_table(&mut g, GO_ALL);
        let ai_eval_set = std::env::var("AI_EVAL").ok().is_some_and(|v| !v.trim().is_empty());
        if !ai_eval_set {
            assert_eq!(learned::on(&g, P0), values::generic_deck(&g, P0), "{k}");
        }
    }
}

#[test]
fn a_two_seat_commander_game_keeps_duel_off() {
    // the live player count never decides it: a 2-seat game at a Commander table (the format's seats are 4)
    let g = play(&["sauron", "yshtola"], 3, false);
    assert!(!g.settings.table.duel && g.players.len() == 2);
    assert_eq!(g.settings.table, TableAi::COMMANDER);
}

// ------------------------------------------------------------------ the profile's data side
#[test]
fn turned_off_names_fields_and_commander_resets_all() {
    assert_eq!(CALI.turned_off("").unwrap(), CALI);
    assert_eq!(CALI.turned_off("commander").unwrap(), TableAi::COMMANDER);
    assert_eq!(CALI.turned_off("learned_eval").unwrap(), TableAi { learned_eval: true, ..CALI });
    assert_eq!(CALI.turned_off("go_rules, duel").unwrap(), TableAi { go_rules: GoRules::T6, duel: false, ..CALI });
    let e = CALI.turned_off("go_rules,speed").unwrap_err();
    assert!(e.contains("bad TABLE_AI") && e.contains("learned_eval"), "{e}");
    assert_eq!(TableAi::COMMANDER.describe(), "Commander's AI profile");
    assert_eq!(CALI.describe(), "learned_eval off, go_rules all, duel cover on");
}

#[test]
fn the_ai_record_is_strict_and_defaults_to_commander() {
    let f = |ai: &str| {
        serde_json::from_str::<Format>(&format!(r#"{{"name":"X","seats":2,"life":30,"first_player_draws":false{ai}}}"#))
    };
    assert_eq!(f("").unwrap().ai, TableAi::COMMANDER);
    let ok = f(r#","ai":{"learned_eval":false,"go_rules":"all"}"#).unwrap();
    assert_eq!((ok.ai.learned_eval, ok.ai.go_rules), (false, GoRules::All));
    assert_eq!(f(r#","ai":{}"#).unwrap().ai, TableAi::COMMANDER);
    for (bad, why) in [
        (r#","ai":{"go_rules":"everyone"}"#, "unknown variant"),
        (r#","ai":{"go_rules":true}"#, "expected"),
        (r#","ai":{"learned":false}"#, "unknown field"),
        (r#","ai":{"duel":true}"#, "unknown field"), // derived from the format's seats, never data
        (r#","ai":{"learned_eval":"no"}"#, "invalid type"),
    ] {
        let e = f(bad).unwrap_err().to_string();
        assert!(e.contains(why), "{bad}: {e}");
    }
}

#[test]
fn the_loader_derives_duel_from_the_seats_and_commander_cannot_change() {
    use std::collections::BTreeMap;
    let rec = |seats: usize, ai: TableAi| Format {
        key: String::new(),
        name: "X".into(),
        seats,
        life: 30,
        first_player_draws: false,
        bans: sim_core::format::Bans::Scryfall,
        banned: vec![],
        ai,
    };
    let mut raw = BTreeMap::new();
    raw.insert("duelish".to_string(), rec(2, TableAi { duel: false, ..CALI }));
    raw.insert("pod".to_string(), rec(4, TableAi { duel: true, ..GO_ALL }));
    let t = sim_core::format::formats_table(raw, []).unwrap();
    assert!(t["duelish"].ai.duel, "2 seats: duel");
    assert!(!t["pod"].ai.duel, "4 seats: no duel, whatever the record said");
    assert_eq!(t["commander"].ai, TableAi::COMMANDER);
    // [commander] must stay Commander's profile, whichever field moves
    for bad in [
        TableAi { learned_eval: false, ..TableAi::COMMANDER },
        TableAi { go_rules: GoRules::All, ..TableAi::COMMANDER },
    ] {
        let mut raw = BTreeMap::new();
        raw.insert("commander".to_string(), Format { ai: bad, ..Format::commander() });
        let e = sim_core::format::formats_table(raw, []).unwrap_err();
        assert!(e.contains("must stay 4 seats, 40 life") && e.contains("AI profile"), "{e}");
    }
}

// ------------------------------------------------------------------ go_risk: cover only when it will be cast
#[test]
fn go_risk_counts_silence_only_for_a_seat_that_will_cast_it() {
    // the same position with and without Silence in hand, Plains open
    let risk = |key: &str, silence: bool| {
        let mut g = position(key, if silence { &["Silence"] } else { &[] }, true, 4);
        set_table(&mut g, GO_ALL);
        t6_table::go_risk(&g, P0)
    };
    // a generic seat (no knob) holding Silence gets no cover, so the table's go rules cannot be fooled by it
    assert!(!plan_data::protect_before_go(&position("sauron", &[], false, 4), P0));
    let (with, without) = (risk("sauron", true), risk("sauron", false));
    assert!(without > 0.0, "the position carries risk");
    assert_eq!(with, without, "no cover for a seat that casts no Silence");
    // a Tier 6 seat casts it: covered (x0.2); so does a plan deck with the opt-in knob
    for key in ["kraum-tymna-blue-breach-midrange", "yoshimaru"] {
        let (with, without) = (risk(key, true), risk(key, false));
        assert!(with < without, "{key}: {with} against {without}");
    }
}

// ------------------------------------------------------------------ the go gates for every seat
#[test]
fn a_generic_seat_waits_for_the_table_under_go_rules_all_and_not_under_the_default() {
    let mut g = position("sauron", &[], true, 4);
    assert!(!t6_plans::is_t6(&g, P0));
    let (risk, thr) = (t6_table::go_risk(&g, P0), t6_table::go_threshold(&g, P0));
    assert!(risk > thr, "an armed table stops it: risk {risk}, bar {thr}");
    // the default profile: a generic seat offers the line whatever the table holds, as before
    assert!(offered(&mut g, P0));
    assert_eq!(stat(&g, P0, "go_held"), 0, "no new stat under the Commander profile");
    // go rules for everyone: held
    set_table(&mut g, GO_ALL);
    assert!(!offered(&mut g, P0), "the table can stop it: wait");
    assert_eq!(stat(&g, P0, "go_held"), 1);
    // asked again in the same turn: still one held turn
    assert!(!offered(&mut g, P0));
    assert_eq!(stat(&g, P0, "go_held"), 1, "once a turn");
    // nobody could answer: offered under the same rules
    let mut g = position("sauron", &[], false, 4);
    set_table(&mut g, GO_ALL);
    assert!(offered(&mut g, P0));
    assert_eq!(stat(&g, P0, "go_held"), 0);
}

#[test]
fn a_tier_6_seat_is_gated_under_the_default_and_keeps_its_stats() {
    let mut g = position("kraum-tymna-blue-breach-midrange", &[], true, 4);
    assert!(t6_plans::is_t6(&g, P0));
    assert!(!offered(&mut g, P0), "Tier 6 waits for the table: today's rule");
    assert_eq!(stat(&g, P0, "go_held"), 0, "counted only under go_rules = all: a Commander game's stats are as before");
}

#[test]
fn attempt_sets_the_win_flag_for_the_go_rules_and_counts_the_line_gone() {
    // a generic seat under go rules all: it goes (nobody can answer), counted, with no protection (no knob) and
    // `t6_going` cleared afterwards
    let mut g = position("sauron", &["Silence"], false, 4);
    set_table(&mut g, GO_ALL);
    let i = combos::combo_index(ORACLE).unwrap();
    assert!(combos::ready(&g, P0, i));
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!(g.t6_going, None);
    assert_eq!(stat(&g, P0, "go_went"), 1);
    assert_eq!(stat(&g, P0, "combo_attempt"), 1);
    assert!(!in_gy(&g, P0, "Silence"), "go_rules = all does not widen the protection step");
    // the default profile: the same attempt, no new stat
    let mut g = position("sauron", &["Silence"], false, 4);
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!((stat(&g, P0, "go_went"), stat(&g, P0, "combo_attempt")), (0, 1));
    // not ready: nothing counted, flag clear
    let mut g = position("sauron", &[], false, 4);
    set_table(&mut g, GO_ALL);
    g.player_mut(P0).hand.clear();
    assert!(!combos::attempt(&mut g, P0, i as i64).unwrap());
    assert_eq!((g.t6_going, stat(&g, P0, "go_went")), (None, 0));
}

#[test]
fn the_opt_in_deck_keeps_its_protection_path_and_sets_no_win_flag_under_the_default() {
    let mut g = position("yoshimaru", &["Silence"], true, 4);
    assert!(plan_data::protect_before_go(&g, P0) && !t6_plans::is_t6(&g, P0) && !t6_plans::go_rules(&g, P0));
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_gy(&g, P0, "Silence"), "the opt-in step still casts it");
    assert_eq!((g.t6_going, stat(&g, P0, "go_went")), (None, 0));
    // under go rules all it also counts as gone for the line
    let mut g = position("yoshimaru", &["Silence"], false, 4);
    set_table(&mut g, GO_ALL);
    combos::attempt(&mut g, P0, i).unwrap();
    assert_eq!(stat(&g, P0, "go_went"), 1);
}

// ------------------------------------------------------------------ the duel cover
/// a Tier 6 or opt-in seat with a line ready, one blue opponent who could answer, and the given protection in hand
fn duel_cast(key: &str, hold: &[&str], table: TableAi) -> (bool, bool) {
    let mut g = position(key, hold, true, 2);
    set_table(&mut g, table);
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    combos::attempt(&mut g, P0, i).unwrap();
    (in_gy(&g, P0, "Silence"), in_gy(&g, P0, "Orim's Chant"))
}

#[test]
fn a_duel_table_casts_silence_against_one_opponent_when_it_holds_no_chant() {
    let t6 = "kraum-tymna-blue-breach-midrange";
    // today: one opponent, Silence is never cast first by a Tier 6 seat
    assert_eq!(duel_cast(t6, &["Silence"], TableAi::COMMANDER), (false, false));
    assert_eq!(duel_cast(t6, &["Silence"], GO_ALL), (false, false), "go rules alone do not change the cover");
    // the duel table: Silence covers the one opponent
    let duel = TableAi { duel: true, ..TableAi::COMMANDER };
    assert_eq!(duel_cast(t6, &["Silence"], duel), (true, false));
    // Chant only: cast as today, both: Chant only (at most one protection spell), neither: none
    assert_eq!(duel_cast(t6, &["Orim's Chant"], duel), (false, true));
    assert_eq!(duel_cast(t6, &["Orim's Chant"], TableAi::COMMANDER), (false, true));
    assert_eq!(duel_cast(t6, &["Silence", "Orim's Chant"], duel), (false, true));
    assert_eq!(duel_cast(t6, &[], duel), (false, false));
    // the whole Cali profile
    assert_eq!(duel_cast(t6, &["Silence"], CALI), (true, false));
    // the opt-in deck casts Silence against one opponent under any profile (unchanged)
    assert_eq!(duel_cast("yoshimaru", &["Silence"], TableAi::COMMANDER), (true, false));
    assert_eq!(duel_cast("yoshimaru", &["Silence"], CALI), (true, false));
    // a generic seat casts none: the protection step is Tier 6's or the knob's, whatever the profile
    assert_eq!(duel_cast("sauron", &["Silence"], CALI), (false, false));
}

#[test]
fn four_player_endgames_keep_silence_for_two_stoppers_whatever_the_live_player_count() {
    // the cover follows the format's profile, not the number of live opponents: a Commander table with one opponent
    // left (the others dead) plays as it always did
    let mut g = position("kraum-tymna-blue-breach-midrange", &["Silence"], true, 4);
    for q in [P2, P3] {
        g.player_mut(q).alive = false;
    }
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(!in_gy(&g, P0, "Silence"));
    assert!(g.player(P1).alive);
}

// ------------------------------------------------------------------ the counters are stats only
#[test]
fn the_counters_draw_no_random_number_and_write_no_log_line() {
    let mut g = position("sauron", &[], true, 4);
    g.log = Some(vec![]);
    set_table(&mut g, GO_ALL);
    let (rng, log) = (g.rng.clone(), g.log.clone());
    t6_table::note_go_held(&mut g, P0);
    assert!(!offered(&mut g, P0));
    assert_eq!(stat(&g, P0, "go_held"), 1);
    assert_eq!(g.rng, rng, "no random number drawn");
    assert_eq!(g.log, log, "no log line written");
    // and the line going: the same two checks across `attempt` against the same game without the counters' stats
    let mut a = position("sauron", &[], false, 4);
    a.log = Some(vec![]);
    set_table(&mut a, GO_ALL);
    let mut b = a.clone();
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    combos::attempt(&mut a, P0, i).unwrap();
    // `b` runs the line without the go rules' bookkeeping (a Commander table): same play, same log
    set_table(&mut b, TableAi::COMMANDER);
    combos::attempt(&mut b, P0, i).unwrap();
    assert_eq!(a.rng, b.rng);
    assert_eq!(a.log, b.log);
    assert_eq!(stat(&a, P0, "go_went"), 1);
    assert_eq!(stat(&b, P0, "go_went"), 0);
}

// ------------------------------------------------------------------ whole games
#[test]
fn a_two_seat_game_under_the_cali_profile_plays_to_the_end() {
    let t5 = tier("t5");
    assert!(t5.len() >= 2);
    let rules = sim_core::format::Rules { life: 30, first_player_draws: false };
    for ai in [AiMode::Adaptive, AiMode::Lookahead] {
        let g = play_at(&[t5[0], t5[1]], 4, false, ai, rules, CALI);
        assert!(g.over || g.round > 0, "{ai:?}");
        assert!(g.settings.table.duel && !learned::on(&g, P0) && !learned::on(&g, P1));
    }
}
