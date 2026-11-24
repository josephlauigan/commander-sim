// written by an independent reviewer
//! Adversarial tests for C4 (`TableAi`): the default profile is Commander everywhere, the gate-only cover change,
//! the duel cover (at most one protection spell), `t6_going` cleanup on an error, the old `go_risk` formula for card
//! code, counters that touch nothing but stats, TABLE_AI parsing and 1v1 thresholds.

use sim_core::ai::{learned, plan_data, t6_table};
use sim_core::format::{Format, GoRules, TableAi};
use sim_core::ids::PlayerId;
use sim_core::impls::{combos, doomsday, t6_plans};
use sim_core::settings::{AiMode, Profile, Settings};
use sim_core::state::Game;
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ORACLE: &str = "Oracle + Demonic Consultation";
const T6: &str = "kraum-tymna-blue-breach-midrange";

const CALI: TableAi = TableAi { learned_eval: false, go_rules: GoRules::All, duel: true };
const GO_ALL: TableAi = TableAi { go_rules: GoRules::All, ..TableAi::COMMANDER };

fn all_keys() -> Vec<String> {
    let mut keys: Vec<String> = decks().iter().map(|d| d.key.clone()).collect();
    keys.sort();
    keys.dedup();
    keys
}

fn stat(g: &Game, p: PlayerId, k: &str) -> i64 {
    g.player(p).stats.get(k).copied().unwrap_or(0)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn offered(g: &mut Game, p: PlayerId) -> bool {
    combos::combo_options(g, p, 0.0, Some(false))
        .unwrap()
        .iter()
        .any(|o| o.label.starts_with("combo: ") && o.label.contains(ORACLE))
}

/// key at P0 with Oracle + Consultation ready, `extra` in hand besides; others armed with counters when `armed`.
/// `plains`: Plains in play (white mana for Silence / Chant).
fn position(key: &str, extra: &[&str], armed: bool, seats: usize, plains: usize) -> Game {
    let keys = ["sisay-legends-toolbox", "yshtola", "sauron"];
    let mut ks = vec![key];
    ks.extend(&keys[..seats - 1]);
    let mut g = table_with(&ks, 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    if plains > 0 {
        lands(&mut g, P0, "Plains", plains, false);
    }
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

// ---------------------------------------------------------------------------------------------------------------
// (1) go_rules == is_t6 for every key, every seat index, every Settings constructor path
#[test]
fn adv1_default_profile_go_rules_is_is_t6_for_every_key_seat_and_constructor() {
    let db = db();
    let keys = all_keys();
    let others = ["sisay-legends-toolbox", "yshtola", "sauron"];
    let settings: Vec<(String, Settings)> = vec![
        ("new/cons/adaptive".into(), Settings::new(Profile::Conservative, AiMode::Adaptive, 1.0)),
        ("new/cons/lookahead".into(), Settings::new(Profile::Conservative, AiMode::Lookahead, 0.5)),
        ("new/loose/adaptive".into(), Settings::new(Profile::Loose, AiMode::Adaptive, 2.0)),
        ("new/loose/lookahead".into(), Settings::new(Profile::Loose, AiMode::Lookahead, 1.0)),
        ("run::settings loose/lookahead".into(), sim_core::run::settings("loose", "lookahead", 1.0).unwrap()),
        ("run::settings cons/adaptive".into(), sim_core::run::settings("conservative", "adaptive", 1.0).unwrap()),
    ];
    for (name, st) in &settings {
        assert_eq!(st.table, TableAi::COMMANDER, "{name}");
        let st = Arc::new(st.clone());
        for k in &keys {
            for at in 0..4usize {
                let mut ks: Vec<&str> = others.to_vec();
                ks.insert(at, k.as_str());
                let players = ks.iter().enumerate().map(|(i, key)| seat(&db, i, key, 1)).collect();
                let g = Game::new(db.clone(), registry(), st.clone(), players, sim_core::rng::Rng::named("adv4"));
                for i in 0..4u8 {
                    let p = PlayerId(i);
                    assert_eq!(t6_plans::go_rules(&g, p), t6_plans::is_t6(&g, p), "{name}: {k} at {at}, seat {i}");
                }
                assert!(!g.settings.table.duel);
            }
        }
    }
    // the testkit constructors too
    for g in [table(&["yshtola", "sauron"]), table_with(&["yshtola"], 2, 30), play(&["sauron", "yshtola"], 4, false)] {
        assert_eq!(g.settings.table, TableAi::COMMANDER);
    }
}

#[test]
fn adv1b_real_data_every_deck_plays_commanders_profile_except_the_cali_decks() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let e = sim_core::run::Engine::load(&dir).unwrap();
    assert!(!e.decks.is_empty());
    let mut cali_seen = 0;
    for k in e.decks.keys() {
        let f = e.format_of(k);
        if f.key == "cali" {
            cali_seen += 1;
            assert_eq!(f.ai, CALI, "{k}");
        } else {
            assert_eq!(f.key, "commander", "{k}");
            assert_eq!(f.ai, TableAi::COMMANDER, "{k}");
        }
    }
    assert!(cali_seen >= 1, "a deck plays cali");
    // a key that is no deck: Commander
    assert_eq!(e.format_of("no-such-deck").ai, TableAi::COMMANDER);
    assert_eq!(Format::commander().ai, TableAi::COMMANDER);
}

// ---------------------------------------------------------------------------------------------------------------
// (4) duel cover: at most one protection spell; Silence only / Chant only / both / none; 2 and 4 seats; every profile
fn casts(key: &str, hold: &[&str], table: TableAi, seats: usize, armed: bool) -> (bool, bool) {
    let mut g = position(key, hold, armed, seats, 3);
    set_table(&mut g, table);
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    combos::attempt(&mut g, P0, i).unwrap();
    assert_eq!(g.t6_going, None);
    (in_gy(&g, P0, "Silence"), in_gy(&g, P0, "Orim's Chant"))
}

#[test]
fn adv4_at_most_one_protection_spell_in_every_combination() {
    let holds: [&[&str]; 4] = [&["Silence"], &["Orim's Chant"], &["Silence", "Orim's Chant"], &[]];
    let tables = [
        TableAi::COMMANDER,
        GO_ALL,
        CALI,
        TableAi { duel: true, ..TableAi::COMMANDER },
        TableAi { learned_eval: false, ..TableAi::COMMANDER },
    ];
    for key in [T6, "yoshimaru", "sauron", "yshtola"] {
        for seats in [2usize, 4] {
            for armed in [true, false] {
                for t in tables {
                    for h in holds {
                        let (s, c) = casts(key, h, t, seats, armed);
                        assert!(!(s && c), "{key} seats {seats} armed {armed} {t:?} hold {h:?}: both cast");
                        if !armed {
                            assert!(!s && !c, "no stopper: nothing to cover ({key} {t:?} {h:?})");
                        }
                        if !h.contains(&"Silence") {
                            assert!(!s);
                        }
                        if !h.contains(&"Orim's Chant") {
                            assert!(!c);
                        }
                        // a generic seat without the knob never casts protection
                        if key == "sauron" || key == "yshtola" {
                            assert!(!s && !c, "{key} {t:?} {h:?}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn adv4b_duel_cover_matrix_for_tier_6() {
    // one live opponent at a duel table
    assert_eq!(casts(T6, &["Silence"], CALI, 2, true), (true, false));
    assert_eq!(casts(T6, &["Orim's Chant"], CALI, 2, true), (false, true));
    assert_eq!(casts(T6, &["Silence", "Orim's Chant"], CALI, 2, true), (false, true));
    assert_eq!(casts(T6, &[], CALI, 2, true), (false, false));
    // four live opponents-ish seats under the duel flag (a hand-built oddity): two stoppers want Silence even when
    // Chant is held
    assert_eq!(casts(T6, &["Silence", "Orim's Chant"], CALI, 4, true), (true, false));
    // Chant only with two stoppers: no cast at all (Chant covers one opponent)
    assert_eq!(casts(T6, &["Orim's Chant"], CALI, 4, true), (false, false));
    // Commander profile, one stopper: Chant only (today)
    assert_eq!(casts(T6, &["Silence"], TableAi::COMMANDER, 2, true), (false, false));
    // no white mana for either: nothing
    let mut g = position(T6, &["Silence", "Orim's Chant"], true, 2, 0);
    set_table(&mut g, CALI);
    combos::attempt(&mut g, P0, combos::combo_index(ORACLE).unwrap() as i64).unwrap();
    assert!(!in_gy(&g, P0, "Silence") && !in_gy(&g, P0, "Orim's Chant"));
}

// ---------------------------------------------------------------------------------------------------------------
// (3) yoshimaru (opt-in knob, not Tier 6)
#[test]
fn adv3_yoshimaru_opt_in_path_under_both_profiles() {
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    for (table, go) in [(TableAi::COMMANDER, false), (GO_ALL, true), (CALI, true)] {
        let mut g = position("yoshimaru", &["Silence"], true, 2, 3);
        set_table(&mut g, table);
        assert!(plan_data::protect_before_go(&g, P0) && !t6_plans::is_t6(&g, P0));
        assert_eq!(t6_plans::go_rules(&g, P0), go);
        combos::attempt(&mut g, P0, i).unwrap();
        assert!(in_gy(&g, P0, "Silence"), "the opt-in reprice path still casts Silence ({table:?})");
        assert_eq!(g.t6_going, None);
        assert_eq!(stat(&g, P0, "go_went") > 0, go && stat(&g, P0, "combo_attempt") > 0, "{table:?}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// (5) the gate: no false cover for a generic seat; card code (Doomsday) keeps the old x0.2 for every deck
#[test]
fn adv5_the_gate_end_to_end_cover_only_for_a_seat_that_casts_it() {
    // armed table: risk is over the bar. With false cover (x0.2) the generic seat would be offered the line.
    for hold in [&["Silence"][..], &["Orim's Chant"][..], &["Silence", "Orim's Chant"][..]] {
        let mut g = position("sauron", hold, true, 4, 3);
        set_table(&mut g, GO_ALL);
        assert!(!offered(&mut g, P0), "generic seat holding {hold:?} gets no cover at the gate");
        let mut g = position("sauron", hold, true, 2, 3);
        set_table(&mut g, CALI);
        // (the 2-seat armed position is under the bar even bare: risk 0.526 < 0.6, so compare the gate's risk instead)
        let mut bare = position("sauron", &[], true, 2, 3);
        set_table(&mut bare, CALI);
        assert_eq!(
            t6_table::go_gate_risk(&g, P0),
            t6_table::go_gate_risk(&bare, P0),
            "generic seat holding {hold:?} (duel) gets no cover at the gate"
        );
        // a Tier 6 seat and the knob seat do cast it: covered, offered
        for key in [T6, "yoshimaru"] {
            let mut g = position(key, hold, true, 4, 3);
            set_table(&mut g, GO_ALL);
            assert!(offered(&mut g, P0), "{key} holding {hold:?} is covered");
        }
    }
    // Tier 6 seat that cannot pay for the protection: no cover (no Plains)
    let mut g = position(T6, &["Silence"], true, 4, 0);
    assert!(!offered(&mut g, P0));
}

/// the body of `go_risk_covered` on origin/main (f5dd0a30), re-implemented from public pieces
fn old_go_risk_covered(g: &Game, p: PlayerId, covered: bool) -> f64 {
    use sim_core::engine::mana::can_pay_spell;
    use sim_core::tag::Tag;
    let mut miss = 1.0;
    for q in g.opps(p) {
        miss *= 1.0 - t6_table::stop_chance(g, q);
    }
    let mut risk = 1.0 - miss;
    let prot: Vec<_> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| matches!(&*g.db.get(c).name, "Silence" | "Orim's Chant"))
        .collect();
    if prot.iter().any(|&c| can_pay_spell(g, p, c, 0, "W", false)) || covered {
        risk *= 0.2;
    }
    let free = |c| {
        let d = g.db.get(c);
        if &*d.name == "Deflecting Swat" {
            return sim_core::engine::values::commander_out(g, p);
        }
        d.tag(Tag::Ctr)
            && (d.tag(Tag::Free)
                || d.tag(Tag::Pact)
                || d.tag(Tag::Misstep)
                || d.tag(Tag::Fon)
                || sim_core::engine::stack::free_counter(g, p, c))
    };
    let mine_free = g.player(p).hand.iter().filter(|&&c| free(c)).count().min(2);
    risk * 0.75f64.powi(mine_free as i32)
}

#[test]
fn adv5b_card_code_risk_is_the_old_formula_for_every_deck_and_every_profile() {
    let holds: [&[&str]; 6] = [
        &[],
        &["Silence"],
        &["Orim's Chant"],
        &["Silence", "Orim's Chant"],
        &["Force of Will", "Silence"],
        &["Force of Will", "Pact of Negation", "Orim's Chant"],
    ];
    let mut n = 0;
    for key in all_keys() {
        for h in holds {
            for armed in [true, false] {
                for plains in [0usize, 2] {
                    for table in [TableAi::COMMANDER, GO_ALL, CALI] {
                        let mut g = position(&key, h, armed, 4, plains);
                        set_table(&mut g, table);
                        for covered in [false, true] {
                            let (new, old) =
                                (t6_table::go_risk_covered(&g, P0, covered), old_go_risk_covered(&g, P0, covered));
                            assert!(
                                (new - old).abs() < 1e-12,
                                "{key} {h:?} armed {armed} plains {plains} {table:?}: {new} vs {old}"
                            );
                            n += 1;
                        }
                        assert_eq!(t6_table::go_risk(&g, P0), old_go_risk_covered(&g, P0, false));
                        // Doomsday's check is `go_risk_covered <= go_threshold`
                        for covered in [false, true] {
                            assert_eq!(
                                doomsday::risk_ok(&g, P0, covered),
                                old_go_risk_covered(&g, P0, covered) <= t6_table::go_threshold(&g, P0),
                                "{key} {h:?}"
                            );
                        }
                        // the gate's risk is the old risk for a seat that casts it, and ignores hand cover otherwise
                        let gate = t6_table::go_gate_risk(&g, P0);
                        if t6_table::casts_cover(&g, P0) {
                            assert!((gate - old_go_risk_covered(&g, P0, false)).abs() < 1e-12, "{key}");
                        } else {
                            let mut bare = position(&key, &[], armed, 4, plains);
                            set_table(&mut bare, table);
                            // same hand minus the protection: free counters in hand are the only other difference
                            let want = {
                                let mut x = position(
                                    &key,
                                    &h.iter()
                                        .copied()
                                        .filter(|n| !matches!(*n, "Silence" | "Orim's Chant"))
                                        .collect::<Vec<_>>(),
                                    armed,
                                    4,
                                    plains,
                                );
                                set_table(&mut x, table);
                                old_go_risk_covered(&x, P0, false)
                            };
                            assert!((gate - want).abs() < 1e-12, "{key} {h:?}: gate {gate} vs {want}");
                        }
                    }
                }
            }
        }
    }
    assert!(n > 1000);
}

// ---------------------------------------------------------------------------------------------------------------
// (6) t6_going cleared on an error path (OutOfWork inside the line); counters stay consistent
#[test]
fn adv6_t6_going_is_cleared_when_the_line_errors() {
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    let mut errs = 0;
    for (key, table) in
        [(T6, TableAi::COMMANDER), ("sauron", GO_ALL), ("yoshimaru", GO_ALL), ("sauron", CALI), (T6, CALI)]
    {
        for extra in 0..80u64 {
            let mut g = position(key, &["Silence", "Orim's Chant"], true, 2, 3);
            set_table(&mut g, table);
            g.work_cap = g.work + extra;
            let r = combos::attempt(&mut g, P0, i);
            if r.is_err() {
                errs += 1;
            }
            assert_eq!(g.t6_going, None, "{key} {table:?} cap +{extra}: t6_going left set after {r:?}");
        }
    }
    assert!(errs > 0, "the budget never cut a line short: the test did not reach an error path");
}

// ---------------------------------------------------------------------------------------------------------------
// (11) counters: stats only; an all-Tier-6 table under go_rules = all is the same game as under Commander
#[test]
fn adv11_an_all_tier_6_table_plays_identically_under_go_rules_all() {
    let t6s = tier("t6");
    assert!(t6s.len() >= 4, "{t6s:?}");
    let ks = [t6s[0], t6s[1], t6s[2], t6s[3]];
    let mut compared = 0;
    for seed in 1..=3u64 {
        let rules = sim_core::format::Rules::COMMANDER;
        let a = play_at(&ks, seed, true, AiMode::Adaptive, rules, TableAi::COMMANDER);
        let b = play_at(&ks, seed, true, AiMode::Adaptive, rules, GO_ALL);
        assert_eq!(a.log, b.log, "seed {seed}: log differs");
        assert_eq!((a.round, a.rng.clone()), (b.round, b.rng.clone()), "seed {seed}");
        for (x, y) in a.players.iter().zip(&b.players) {
            let strip = |m: &indexmap::IndexMap<sim_core::sym::Sym, i64>| {
                m.iter()
                    .filter(|(k, _)| !matches!(&*k.to_string(), "go_held" | "go_went"))
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect::<Vec<_>>()
            };
            assert_eq!(strip(&x.stats), strip(&y.stats), "seed {seed}");
            assert_eq!(stat(&a, x.id, "go_held") + stat(&a, x.id, "go_went"), 0, "Commander games carry no new stat");
            assert_eq!(x.life, y.life);
            assert_eq!(x.alive, y.alive);
        }
        compared += 1;
    }
    assert_eq!(compared, 3);
}

#[test]
fn adv11b_commander_games_carry_no_new_stat_or_turn_flag() {
    for (keys, seed) in [
        (vec!["sauron", "yshtola", "sisay-legends-toolbox", "yoshimaru"], 3u64),
        (vec![T6, "sauron", "yshtola", "yoshimaru"], 4),
        (vec!["yshtola", "sauron"], 5),
    ] {
        let g = play(&keys, seed, false);
        for p in &g.players {
            for k in ["go_held", "go_went"] {
                assert!(!p.stats.keys().any(|s| &*s.to_string() == k), "{keys:?}: {k} appeared in a Commander game");
            }
            assert!(!p.flag_turn.keys().any(|s| &*s.to_string() == "go_held_turn"));
        }
    }
}

#[test]
fn adv11c_a_cali_game_counts_and_stays_deterministic() {
    let rules = sim_core::format::Rules { life: 30, first_player_draws: false };
    let keys = [T6, "sauron"];
    let a = play_at(&keys, 7, true, AiMode::Adaptive, rules, CALI);
    let b = play_at(&keys, 7, true, AiMode::Adaptive, rules, CALI);
    assert_eq!(a.log, b.log);
    assert!(a.round > 0);
    // go_held / go_went never exceed the turns a seat had
    for p in &a.players {
        assert!(p.stats.get("go_held").copied().unwrap_or(0) <= p.turns as i64 + 1);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// (7) learned::on is false at every Cali seat; in-process only checks the AI_EVAL-unset case, the child checks the env
#[test]
fn adv7_learned_off_at_every_cali_seat_even_with_ai_eval_listing_keys() {
    if std::env::var("ADV_CALI4_CHILD").is_ok() {
        let positive = std::env::var("ADV_CALI4_CHILD").unwrap() == "positive";
        for k in all_keys() {
            let mut g = table_with(&[&k, "sisay-legends-toolbox"], 1, 30);
            let commander_on = (0..2).any(|i| learned::on(&g, PlayerId(i)));
            set_table(&mut g, CALI);
            for i in 0..2 {
                assert!(!learned::on(&g, PlayerId(i)), "{k} seat {i}: learned on at a Cali table");
            }
            set_table(&mut g, TableAi { learned_eval: false, ..TableAi::COMMANDER });
            for i in 0..2 {
                assert!(!learned::on(&g, PlayerId(i)));
            }
            if positive && std::env::var("AI_EVAL").map(|v| v != "0").unwrap_or(true) && commander_on {
                return; // control: the Commander profile did have it on for some generic seat
            }
        }
        if positive {
            panic!("control failed: no generic deck had the learned evaluation on under the Commander profile");
        }
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let all = all_keys().join(",");
    for (env, mode) in [
        (None, "positive"),
        (Some("1"), "positive"),
        (Some("0"), "none"),
        (Some(all.as_str()), "positive"),
        (Some("sauron,yshtola,sisay-legends-toolbox"), "positive"),
        (Some("bogus-key"), "none"),
    ] {
        let mut c = std::process::Command::new(&exe);
        c.args(["adv7_learned_off_at_every_cali_seat_even_with_ai_eval_listing_keys", "--exact", "--nocapture"])
            .env("ADV_CALI4_CHILD", mode)
            .env_remove("AI_EVAL");
        if let Some(v) = env {
            c.env("AI_EVAL", v);
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success(),
            "AI_EVAL={env:?}: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------
// (8) 1v1 thresholds and risks: finite, ordered, not blocking forever
#[test]
fn adv8_one_v_one_threshold_and_risk_are_finite_and_relax_with_the_round() {
    for key in [T6, "sauron", "yshtola"] {
        let mut prev = 0.0;
        for round in 1..=12u32 {
            let mut g = position(key, &[], true, 2, 3);
            set_table(&mut g, CALI);
            g.round = round as _;
            let thr = t6_table::go_threshold(&g, P0);
            let risk = t6_table::go_risk(&g, P0);
            assert!(thr.is_finite() && risk.is_finite() && (0.0..=1.0).contains(&risk), "{key} r{round}: {thr} {risk}");
            assert!(thr >= 0.6 && thr <= 1.0 && thr >= prev, "{key} r{round}: threshold {thr} after {prev}");
            prev = thr;
            if round >= 6 {
                assert_eq!(thr, 1.0);
                assert!(t6_table::go_gate_risk(&g, P0) <= thr);
            }
        }
    }
    // the opponent dead / hand empty: no NaN, no division by zero
    let mut g = position(T6, &[], true, 2, 3);
    g.player_mut(P1).alive = false;
    let (thr, risk) = (t6_table::go_threshold(&g, P0), t6_table::go_risk(&g, P0));
    assert!(thr.is_finite() && risk.is_finite(), "{thr} {risk}");
    let mut g = position(T6, &[], true, 2, 3);
    g.player_mut(P1).hand.clear();
    assert_eq!(t6_table::go_risk(&g, P0), 0.0);
    assert_eq!(t6_table::archenemy_score(&g, P1) >= 0.0, true);
}

// ---------------------------------------------------------------------------------------------------------------
// (12) TABLE_AI parsing (the core half: `TableAi::turned_off`); the CLI half is in sim-cli's adv_cali4 test
#[test]
fn adv12_turned_off_parsing() {
    let ok = |s: &str| CALI.turned_off(s);
    assert_eq!(ok("").unwrap(), CALI);
    assert_eq!(ok("   ").unwrap(), CALI);
    assert_eq!(ok("commander").unwrap(), TableAi::COMMANDER);
    assert_eq!(ok(" commander ").unwrap(), TableAi::COMMANDER);
    assert_eq!(ok("go_rules,learned_eval").unwrap(), TableAi { go_rules: GoRules::T6, learned_eval: true, duel: true });
    assert_eq!(ok("go_rules, learned_eval , duel").unwrap(), TableAi::COMMANDER);
    assert_eq!(ok("learned_eval,learned_eval").unwrap(), TableAi { learned_eval: true, ..CALI });
    // refused
    for bad in [
        "Commander",
        "COMMANDER",
        "go_rules,",
        ",go_rules",
        "commander,go_rules",
        "all",
        "go_rules;learned_eval",
        "none",
        "t6",
        "speed",
    ] {
        let e = ok(bad).unwrap_err();
        assert!(e.contains("bad TABLE_AI"), "{bad:?}: {e}");
    }
    // a Commander table: any valid spec changes nothing
    for s in ["commander", "go_rules", "learned_eval", "duel", "go_rules,learned_eval,duel"] {
        assert_eq!(TableAi::COMMANDER.turned_off(s).unwrap(), TableAi::COMMANDER, "{s}");
    }
    // every combination describes without panicking, and names its fields
    let mut seen = std::collections::BTreeSet::new();
    for le in [false, true] {
        for go in [GoRules::T6, GoRules::All] {
            for duel in [false, true] {
                let t = TableAi { learned_eval: le, go_rules: go, duel };
                let d = t.describe();
                assert!(seen.insert(d.clone()) || t == TableAi::COMMANDER, "{d} is not unique");
                if t != TableAi::COMMANDER {
                    assert!(d.contains("learned eval") && d.contains("go rules") && d.contains("duel cover"), "{d}");
                    assert_eq!(d.contains("learned eval on"), le);
                    assert_eq!(d.contains("every seat"), go == GoRules::All);
                    assert_eq!(d.contains("duel cover on"), duel);
                }
            }
        }
    }
    assert_eq!(seen.len(), 8);
}

/// diagnostic for adv5's duel failure: is the 2-seat armed position over the bar without any cover?
#[test]
fn adv5c_diagnose_duel_gate_numbers() {
    for hold in [&[][..], &["Silence"][..]] {
        let mut g = position("sauron", hold, true, 2, 3);
        set_table(&mut g, CALI);
        println!(
            "hold {hold:?}: gate_risk {:.3} go_risk {:.3} thr {:.3} offered {}",
            t6_table::go_gate_risk(&g, P0),
            t6_table::go_risk(&g, P0),
            t6_table::go_threshold(&g, P0),
            offered(&mut g, P0)
        );
    }
}
