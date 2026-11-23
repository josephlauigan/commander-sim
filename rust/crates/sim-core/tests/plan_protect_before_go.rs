//! Stage 4: the opt-in plan knob `knobs.protect_before_go`. A plan deck that sets it casts Silence / Orim's Chant from
//! hand before a line goes, the step `t6_table::protect_before_go` gives a Tier 6 player (`combos::attempt`). Off, every
//! deck plays as before; the Tier 6 slots (`go_now`, `hold_up`) stay Tier 6 only.

use sim_core::ai::{plan_data, t6_table};
use sim_core::ids::PlayerId;
use sim_core::impls::{combos, t6_plans};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ORACLE: &str = "Oracle + Demonic Consultation";

/// p0's deck with a line ready (Thassa's Oracle + Demonic Consultation, {1}{U}{U}{B} up), Silence and Plains in hand
/// range, and two blue opponents whose hands could answer (stop chance 0.25 each, so the table needs two of them)
fn position(key: &str, opps_hold: bool) -> Game {
    let mut g = table_with(&[key, "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Silence"]);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    if opps_hold {
        for q in [P1, P2] {
            lands(&mut g, q, "Island", 4, false);
            hand(&mut g, q, &["Brainstorm", "Opt", "Preordain", "Ponder", "Island"]);
        }
    }
    g
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn go(g: &mut Game) {
    let i = combos::combo_index(ORACLE).unwrap();
    assert!(combos::ready(g, P0, i));
    combos::attempt(g, P0, i as i64).unwrap();
}

#[test]
fn the_knob_parses_and_a_typo_is_rejected() {
    let l = plan_data::load("x", "deck = \"x\"\n[knobs]\nprotect_before_go = true").unwrap();
    assert_eq!(l.plan.knobs.protect_before_go, Some(true));
    let l = plan_data::load("x", "deck = \"x\"").unwrap();
    assert_eq!(l.plan.knobs.protect_before_go, None);
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_befor_go = true").contains("unknown field"));
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_before_go_now = true").contains("unknown field"));
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_before_go = 3").contains("invalid type"));
}

#[test]
fn only_the_yoshimaru_plan_turns_it_on() {
    for (k, l) in plan_data::all() {
        assert_eq!(l.plan.knobs.protect_before_go.is_some(), *k == "yoshimaru", "{k}");
    }
    assert_eq!(plan_data::get("yoshimaru").unwrap().plan.knobs.protect_before_go, Some(true));
}

#[test]
fn with_the_knob_silence_is_cast_before_the_line() {
    let mut g = position("yoshimaru", true);
    assert!(plan_data::protect_before_go(&g, P0));
    assert!(!t6_plans::is_t6(&g, P0), "the knob opens one step, not the Tier 6 gate");
    assert!(t6_plans::go_now(&g, P0, ORACLE).is_none(), "go_now stays closed to a non-Tier-6 deck");
    assert!(t6_table::go_risk(&g, P0) > 0.0);
    go(&mut g);
    assert!(in_gy(&g, P0, "Silence"), "Silence was cast first");
    assert_eq!(g.winner, Some(P0), "with the table locked out the line goes through");
    assert_eq!(g.t6_going, None);
}

#[test]
fn with_the_knob_but_nothing_to_fear_silence_stays_in_hand() {
    let mut g = position("yoshimaru", false);
    go(&mut g);
    assert!(!in_gy(&g, P0, "Silence"), "the same conditions as Tier 6: no opponent could answer");
}

#[test]
fn without_the_knob_the_same_position_casts_no_silence() {
    // decks that play from a plan or not, none of them sets the knob: today's play
    for key in ["kyodai", "sheoldred", "urdragon", "cloud"] {
        let mut g = position(key, true);
        assert!(!plan_data::protect_before_go(&g, P0), "{key}");
        go(&mut g);
        assert!(!in_gy(&g, P0, "Silence"), "{key}: Silence stays in hand");
    }
    // a pool seat with no plan file at all
    let mut g = position("sauron", true);
    assert!(!plan_data::protect_before_go(&g, P0));
    go(&mut g);
    assert!(!in_gy(&g, P0, "Silence"), "sauron");
}

#[test]
fn a_tier_6_seat_still_casts_silence_whatever_its_plan_says() {
    for key in ["kraum-tymna-blue-breach-midrange", "kinnan-bonder-mana-combo"] {
        let mut g = position(key, true);
        assert!(t6_plans::is_t6(&g, P0));
        assert!(!plan_data::protect_before_go(&g, P0), "{key} has no knob");
        go(&mut g);
        assert!(in_gy(&g, P0, "Silence"), "{key}: Tier 6 casts it first");
    }
}

// ------------------------------------------------------------------ round 2: the knob path's two changes stay off Tier 6
const HELIOD: &str = "Heliod, Sun-Crowned + Walking Ballista";

/// `key` at P0 with Heliod out, Walking Ballista and Silence (no Orim's Chant) in hand, `plains` Plains and `swamps`
/// Swamps; `opps` of P1/P2 hold four Islands and a hand that could answer
fn heliod(key: &str, plains: usize, swamps: usize, opps: usize) -> Game {
    let mut g = table_with(&[key, "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", plains, false);
    lands(&mut g, P0, "Swamp", swamps, false);
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    hand(&mut g, P0, &["Walking Ballista", "Silence"]);
    for q in [P1, P2].into_iter().take(opps) {
        lands(&mut g, q, "Island", 4, false);
        hand(&mut g, q, &["Brainstorm", "Opt", "Preordain", "Ponder", "Island"]);
    }
    g
}

fn go_heliod(g: &mut Game) {
    let i = combos::combo_index(HELIOD).unwrap();
    combos::attempt(g, P0, i as i64).unwrap();
}

#[test]
fn tier_6_with_one_stopper_and_only_silence_casts_nothing() {
    let mut g = heliod("kraum-tymna-blue-breach-midrange", 2, 5, 1);
    assert!(t6_plans::is_t6(&g, P0));
    let stops = [P1, P2].iter().filter(|&&q| t6_table::stop_chance(&g, q) >= 0.15).count();
    assert_eq!(stops, 1);
    go_heliod(&mut g);
    assert!(!in_gy(&g, P0, "Silence"), "Tier 6 keeps Silence for two stoppers, as before");
    // the knob deck, same position, casts it
    let mut g = heliod("yoshimaru", 2, 5, 1);
    go_heliod(&mut g);
    assert!(in_gy(&g, P0, "Silence"));
}

#[test]
fn tier_6_under_damping_sphere_is_unchanged_and_a_knob_deck_keeps_silence() {
    // 5 lands: the line is {3}{W}, Silence {W}; with Silence counted the Sphere adds {1} to the line
    let mut g = heliod("kraum-tymna-blue-breach-midrange", 2, 3, 2);
    perm(&mut g, P1, "Damping Sphere");
    go_heliod(&mut g);
    assert!(in_gy(&g, P0, "Silence"), "Tier 6 still casts Silence first");
    assert_ne!(g.winner, Some(P0), "and the line is not played: the known Tier 6 flaw, a ticket, not changed here");
    let mut g = heliod("yoshimaru", 2, 3, 2);
    perm(&mut g, P1, "Damping Sphere");
    go_heliod(&mut g);
    assert!(in_hand(&g, P0, "Silence"), "the knob deck sees the tax and keeps Silence");
    assert_eq!(g.winner, Some(P0));
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}
