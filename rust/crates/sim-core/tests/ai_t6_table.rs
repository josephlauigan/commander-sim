//! Tier 6 table rules (ai/t6_table.rs): archenemy, answers spent on the win, hold up vs develop, going when it can't
//! be stopped, protection first, Angel's Grace, and that other tiers are untouched.

use sim_core::ai::t6_table::{self, Verdict};
use sim_core::ids::PlayerId;
use sim_core::impls::{combos, t6_plans};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const SISAY: &str = "sisay-legends-toolbox";
const KINNAN: &str = "kinnan-bonder-mana-combo";
const DACK: &str = "dack-fayden-white-stax";

fn tbl(keys: &[&str]) -> Game {
    let mut g = table(keys);
    g.active = Some(P0);
    g
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

const ORACLE: &str = "Oracle + Demonic Consultation";

fn oracle_table(keys: &[&str], seed: u64) -> Game {
    let mut g = table_with(keys, seed, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    g
}

fn go(g: &mut Game) {
    let i = combos::combo_index(ORACLE).unwrap();
    assert!(combos::ready(g, P0, i));
    combos::attempt(g, P0, i as i64).unwrap();
}

#[test]
fn the_gate_is_the_seven_tier_6_keys() {
    for k in t6_plans::T6_KEYS {
        assert!(t6_plans::is_t6_key(k));
    }
    assert_eq!(t6_plans::T6_KEYS.len(), 7);
    for k in ["seph", "sauron", "yshtola", "cloud", "kefka"] {
        assert!(!t6_plans::is_t6_key(k));
    }
}

#[test]
fn free_force_of_will_counters_thassas_oracle_line_every_time() {
    // three opponents; one holds Force of Will and a blue card to pitch, no mana: the win step is always countered
    for seed in 0..12u64 {
        let mut g = oracle_table(&[KRAUM, SISAY, "seph", "sauron"], seed);
        hand(&mut g, P1, &["Force of Will", "Brainstorm"]);
        go(&mut g);
        assert_eq!(g.winner, None, "seed {seed}: the line must be stopped");
        assert!(in_gy(&g, P1, "Force of Will"), "seed {seed}: Force of Will was cast");
    }
}

#[test]
fn the_same_line_wins_when_nobody_can_answer() {
    let mut g = oracle_table(&[KRAUM, SISAY, "seph", "sauron"], 3);
    go(&mut g);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_win_step_is_always_countered_but_the_first_spell_is_held_for() {
    let mut g = tbl(&[KRAUM, SISAY]);
    let c = card(&g, "Demonic Consultation");
    let it = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), 5.0);
    g.t6_going = Some(P0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 5.0, 4.0, 1), Verdict::Counter);
    g.t6_going = None;
    match t6_table::counter_verdict(&g, P1, &it, 5.0, 4.0, 1) {
        Verdict::Roll(v, t) => assert!(t > 4.0 && v <= 5.0, "a single counter is held: threshold {t}"),
        v => panic!("{v:?}"),
    }
}

#[test]
fn ad_nauseam_is_a_win_spell_to_counter() {
    let mut g = tbl(&[KRAUM, SISAY]);
    let c = card(&g, "Ad Nauseam");
    assert!(sim_core::ai::spell_importance(&g, P0, c) >= 8.0);
    let it = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), 8.0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 8.0, 4.0, 1), Verdict::Counter);
}

#[test]
fn the_archenemy_is_the_player_with_the_line_on_board() {
    let mut g = tbl(&[KINNAN, SISAY, KRAUM]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    lands(&mut g, P0, "Forest", 3, false);
    let (a, s) = t6_table::archenemy(&g, P1).expect("an archenemy");
    assert_eq!(a, P0);
    assert!(s >= t6_table::NEAR, "score {s}");
    assert!(t6_table::archenemy(&g, P0).is_none() || t6_table::archenemy_score(&g, P1) < s);
}

#[test]
fn counters_go_to_the_archenemy_not_to_a_bystander() {
    let mut g = tbl(&[KINNAN, SISAY, KRAUM]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    lands(&mut g, P0, "Forest", 3, false);
    let c = card(&g, "Brainstorm");
    let it = sim_core::engine::stack::spell_item(&mut g, P2, c, Ctx::default(), 4.0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 4.0, 4.0, 2), Verdict::Pass, "P2 is not the one to stop");
    let it = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), 4.0);
    let v = t6_table::counter_verdict(&g, P1, &it, 4.0, 4.0, 2);
    assert!(v == Verdict::Counter || matches!(v, Verdict::Roll(x, t) if x > 4.0 && t < 4.0), "{v:?}");
}

#[test]
fn a_player_with_swan_song_up_does_not_tap_out_for_a_creature_while_an_opponent_has_the_engine() {
    let mut g = tbl(&[SISAY, KINNAN, KRAUM]);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Swan Song", "Mulldrifter"]);
    perm(&mut g, P1, "Kinnan, Bonder Prodigy");
    perm(&mut g, P1, "Basalt Monolith");
    lands(&mut g, P1, "Forest", 3, false);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let stop = opts.iter().find(|o| o.act.is_none()).expect("stop").utility;
    let cast = opts.iter().filter(|o| o.label.starts_with("Mulldrifter")).map(|o| o.utility).fold(f64::MIN, f64::max);
    assert!(stop > cast, "stop {stop} vs Mulldrifter {cast}");
}

#[test]
fn going_waits_when_the_table_can_stop_it_and_goes_when_it_cannot() {
    let mut g = oracle_table(&[KRAUM, SISAY, "seph", "sauron"], 1);
    // an empty table: nobody can answer
    assert!(t6_table::go_ok(&g, P0, ORACLE));
    // three opponents with open blue mana and full hands
    lands(&mut g, P1, "Island", 4, false);
    lands(&mut g, P2, "Island", 4, false);
    hand(&mut g, P1, &["Force of Will", "Brainstorm", "Opt", "Counterspell", "Swan Song"]);
    hand(&mut g, P2, &["Brainstorm", "Opt", "Counterspell", "Swan Song", "Force of Will"]);
    let risk = t6_table::go_risk(&g, P0);
    assert!(risk > 0.35, "risk {risk}");
}

#[test]
fn silence_is_cast_before_the_line_when_opponents_could_answer() {
    let mut g = oracle_table(&[KRAUM, SISAY, "seph", "sauron"], 5);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Silence"]);
    lands(&mut g, P1, "Island", 4, false);
    hand(&mut g, P1, &["Brainstorm", "Opt", "Preordain", "Ponder", "Island"]);
    lands(&mut g, P2, "Island", 4, false);
    hand(&mut g, P2, &["Brainstorm", "Opt", "Preordain", "Ponder", "Island"]);
    go(&mut g);
    assert!(in_gy(&g, P0, "Silence"), "Silence was cast first");
    assert_eq!(g.winner, Some(P0), "with the table locked out the line goes through");
}

#[test]
fn angels_grace_saves_a_player_who_would_lose() {
    let mut g = tbl(&[DACK, SISAY]);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Angel's Grace"]);
    g.player_mut(P0).life = 0;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.player(P0).alive, "Angel's Grace");
    assert!(in_gy(&g, P0, "Angel's Grace"));
    assert_eq!(g.player(P0).life, 1);
}

#[test]
fn without_grace_the_same_player_loses_and_other_tiers_never_cast_it() {
    let mut g = tbl(&[DACK, SISAY]);
    g.player_mut(P0).life = 0;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive);
    let mut g = tbl(&["seph", SISAY]);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Angel's Grace"]);
    g.player_mut(P0).life = 0;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive, "a Tier 0-5 player does not use the Tier 6 rule");
}

#[test]
fn the_archenemy_reading_does_not_see_cards_in_hand() {
    // Thassa's Oracle and Demonic Consultation in hand are hidden: nothing of the line is public
    let g = oracle_table(&[KRAUM, SISAY, KINNAN], 1);
    assert_eq!(combos::line_progress(&g, P0), (0.0, false));
}

#[test]
fn quiet_tier_6_lines_count_for_the_archenemy_reading() {
    // Godo + Helm of the Host is a quiet line (Tier 0-5 decks don't value its pieces); the Tier 6 reading sees it
    let mut g = tbl(&["godo", SISAY, KINNAN]);
    perm(&mut g, P0, "Godo, Bandit Warlord");
    perm(&mut g, P0, "Helm of the Host");
    let (frac, _) = combos::line_progress(&g, P0);
    assert!(frac > 0.0, "Godo and Helm on the battlefield: {frac}");
}

#[test]
fn a_commander_line_is_ready_only_when_the_commander_can_be_cast() {
    // Dack's flip line is Dack alone; in the command zone with no mana it is not ready, with the mana it is
    let mut g = tbl(&[DACK, SISAY, KINNAN]);
    assert!(!combos::line_progress(&g, P0).1);
    lands(&mut g, P0, "Plains", 6, false); // Dack costs {4}{W}{W}
    assert!(combos::line_progress(&g, P0).1);
}

#[test]
fn a_spell_that_completes_a_line_is_countered_and_a_piece_is_worth_more() {
    // Kinnan cast with Basalt Monolith on the battlefield completes Kinnan + Basalt (a quiet Tier 6 line counts too)
    let mut g = tbl(&[KINNAN, KRAUM, SISAY]);
    perm(&mut g, P0, "Basalt Monolith");
    let k = card(&g, "Kinnan, Bonder Prodigy");
    let it = sim_core::engine::stack::spell_item(&mut g, P0, k, Ctx::default(), 4.0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 3.0, 4.0, 2), Verdict::Counter);
    // Godo with nothing of its lines out is a piece, not the last one: its value rises above the plain creature's
    let mut g = tbl(&["godo", KRAUM, SISAY]);
    let godo = card(&g, "Godo, Bandit Warlord");
    assert!(combos::piece_gain(&g, P0, "Godo, Bandit Warlord") > 0.0);
    let it = sim_core::engine::stack::spell_item(&mut g, P0, godo, Ctx::default(), 4.0);
    let v = t6_table::counter_verdict(&g, P1, &it, 3.0, 4.0, 2);
    assert!(matches!(v, Verdict::Counter) || matches!(v, Verdict::Roll(x, _) if x > 5.0), "{v:?}");
}

#[test]
fn pact_of_negation_is_not_picked_when_its_upkeep_cannot_be_paid() {
    // one land: Pact's {0} used to fall through to the plain "can it be paid" check, and the player lost at its upkeep
    let mut g = tbl(&[KRAUM, SISAY, KINNAN]);
    lands(&mut g, P1, "Island", 1, false);
    hand(&mut g, P1, &["Pact of Negation"]);
    let oracle = card(&g, "Thassa's Oracle");
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, oracle), None);
    lands(&mut g, P1, "Island", 4, false);
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, oracle), Some(card(&g, "Pact of Negation")));
}
