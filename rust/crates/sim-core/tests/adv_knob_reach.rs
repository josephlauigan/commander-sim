// Written by an independent reviewer (not the author of the knob), adopted unchanged.
//! Adversarial tests for the opt-in knob `protect_before_go` (branch stage4-protect-knob): reach (which of Yoshimaru's
//! lines run through `combos::attempt`), the protect step's edges for a non-Tier-6 deck, the default path, and Tier 6.

use sim_core::ai::{plan_data, t6_table};
use sim_core::hooks::Action;
use sim_core::ids::PlayerId;
use sim_core::impls::{combos, t6_plans};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const HELIOD: &str = "Heliod, Sun-Crowned + Walking Ballista";
const VITO: &str = "Vito / Enduring Tenacity";

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// the opponents P1 and P2 (both) could answer: four open Islands and a hand; `n` of them (0, 1 or 2)
fn arm(g: &mut Game, n: usize) {
    for q in [P1, P2].into_iter().take(n) {
        lands(g, q, "Island", 4, false);
        hand(g, q, &["Brainstorm", "Opt", "Preordain", "Ponder", "Island"]);
    }
}

/// yoshimaru at P0, Heliod out, Walking Ballista and Silence in hand, `plains` Plains and `swamps` Swamps untapped
fn heliod_position(plains: usize, swamps: usize, opps: usize) -> Game {
    let mut g = table_with(&["yoshimaru", "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", plains, false);
    lands(&mut g, P0, "Swamp", swamps, false);
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    hand(&mut g, P0, &["Walking Ballista", "Silence"]);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    arm(&mut g, opps);
    g
}

fn line(part: &str) -> i64 {
    combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}")) as i64
}

// ------------------------------------------------------------------ reach
#[test]
fn both_yoshimaru_plan_lines_are_catalogue_combos() {
    // plan lines name catalogue combos; a name that is not one would never reach `attempt`
    let l = plan_data::get("yoshimaru").unwrap();
    assert_eq!(l.plan.knobs.protect_before_go, Some(true));
    assert!(combos::combo_index(HELIOD).is_some());
    assert!(combos::combo_index(VITO).is_some());
}

#[test]
fn heliod_line_through_attempt_casts_silence_first() {
    // {3}{W} for the line (mana_need), {W} for Silence: 7 lands, two Plains
    let mut g = heliod_position(2, 5, 2);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    assert!(combos::ready(&g, P0, i as usize));
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_gy(&g, P0, "Silence"), "Silence cast before the line");
    assert_eq!(g.winner, Some(P0), "and the line went");
    assert_eq!(g.t6_going, None);
}

#[test]
fn heliod_line_with_no_spare_white_keeps_silence_and_still_goes() {
    // 7 lands but one white source: the line needs {W}, Silence needs another {W}: the combined cost can't be paid
    let mut g = heliod_position(1, 6, 2);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_hand(&g, P0, "Silence"), "Silence stays in hand: its {{W}} is the line's");
    assert_eq!(g.winner, Some(P0), "the line is not fizzled by a Silence that cannot be afforded");
}

#[test]
fn heliod_line_with_exactly_the_line_mana_keeps_silence() {
    // 4 lands: the line alone (mana_need = {3}{W})
    let mut g = heliod_position(2, 2, 2);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_hand(&g, P0, "Silence"));
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn vito_and_blood_on_the_battlefield_cast_silence() {
    let mut g = table_with(&["yoshimaru", "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 1, false);
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    perm(&mut g, P0, "Exquisite Blood");
    hand(&mut g, P0, &["Silence"]);
    arm(&mut g, 2);
    let i = line("Vito / Enduring Tenacity");
    assert!(combos::ready(&g, P0, i as usize));
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_gy(&g, P0, "Silence"), "casts: none, need (0,\"\"): Silence is still cast first");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn vito_with_blood_in_hand_casts_silence_then_blood() {
    let mut g = table_with(&["yoshimaru", "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 2, false);
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    hand(&mut g, P0, &["Exquisite Blood", "Silence"]);
    arm(&mut g, 2);
    let i = line("Vito / Enduring Tenacity");
    assert!(combos::ready(&g, P0, i as usize));
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_gy(&g, P0, "Silence"));
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn combo_options_offers_the_heliod_line_and_its_action_is_attempt() {
    let mut g = heliod_position(2, 5, 2);
    let opts = combos::combo_options(&mut g, P0, 0.0, Some(false)).unwrap();
    let o = opts.iter().find(|o| o.label.contains("Heliod, Sun-Crowned")).expect("the Heliod line is offered");
    match &o.act {
        Some(Action::Plan { f, arg }) => {
            assert_eq!(*f as usize, combos::attempt as *const () as usize, "the option runs `attempt`");
            f(&mut g, P0, *arg).unwrap();
        }
        _ => panic!("not a Plan action"),
    }
    assert!(in_gy(&g, P0, "Silence"));
}

#[test]
fn the_brain_main_phase_runs_the_line_with_silence() {
    let mut g = heliod_position(2, 5, 2);
    g.log = Some(vec![]);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    let log = g.log.clone().unwrap().join("\n");
    assert_eq!(g.winner, Some(P0), "the main phase goes for the line:\n{log}");
    assert!(in_gy(&g, P0, "Silence"), "and protects it first:\n{log}");
}

// ------------------------------------------------------------------ the protect step's guards
#[test]
fn a_line_not_ready_casts_no_silence() {
    let mut g = heliod_position(2, 5, 2);
    // Ballista gone: the line is not ready
    let b = g.player(P0).hand.iter().position(|&c| &*g.db.get(c).name == "Walking Ballista").unwrap();
    g.player_mut(P0).hand.remove(b);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    assert!(!combos::ready(&g, P0, i as usize));
    assert!(!combos::attempt(&mut g, P0, i).unwrap());
    assert!(in_hand(&g, P0, "Silence"));
}

#[test]
fn an_unaffordable_line_casts_no_silence() {
    // 3 lands: the line ({3}{W}) can't be paid, so Silence must not be spent on nothing
    let mut g = heliod_position(2, 1, 2);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_hand(&g, P0, "Silence"), "no Silence for a line that cannot go");
}

#[test]
fn a_line_already_gone_this_turn_casts_no_second_silence() {
    let mut g = heliod_position(2, 5, 2);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    let turns = g.player(P0).turns;
    g.player_mut(P0).combo_turn = Some((turns, combos::all()[i as usize].name));
    assert!(!combos::attempt(&mut g, P0, i).unwrap());
    assert!(in_hand(&g, P0, "Silence"), "combo_turn guard");
}

#[test]
fn nobody_can_answer_keeps_silence_in_hand() {
    let mut g = heliod_position(2, 5, 0);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_hand(&g, P0, "Silence"));
    assert_eq!(g.winner, Some(P0));
}

/// FINDING candidate: with ONE opponent who could answer, the table's rule casts only Orim's Chant ("silence !=
/// (stops >= 2)"); Yoshimaru's list has no Chant, so the knob does nothing against a single stopper. This asserts the
/// useful behaviour (Silence is better than nothing against one opponent); it fails if the Tier 6 rule is reused as is.
#[test]
fn one_opponent_who_could_answer_silence_is_cast_for_a_deck_without_chant() {
    let mut g = heliod_position(2, 5, 1);
    let stops = [P1, P2].iter().filter(|&&q| t6_table::stop_chance(&g, q) >= 0.15).count();
    assert_eq!(stops, 1, "setup: exactly one opponent could respond");
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(in_gy(&g, P0, "Silence"), "Silence was not cast against a single possible answer");
}

#[test]
fn silence_uncastable_for_colour_does_not_spend_line_mana() {
    // no white source beyond the line's own, plus an extra colourless land: the line is payable, Silence is not
    let mut g = heliod_position(1, 5, 2);
    let n = g.player(P0).lands.len();
    assert_eq!(n, 6);
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g, P0, i).unwrap();
    assert_eq!(g.winner, Some(P0));
    assert!(in_hand(&g, P0, "Silence"));
}

// ------------------------------------------------------------------ copies (look-ahead playouts)
#[test]
fn a_look_ahead_copy_spends_its_own_silence_only() {
    let mut g = heliod_position(2, 5, 2);
    let mut g2 = sim_core::ai::search::clone(&mut g);
    g2.in_search = true;
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    combos::attempt(&mut g2, P0, i).unwrap();
    assert!(in_gy(&g2, P0, "Silence"));
    assert!(in_hand(&g, P0, "Silence"), "the real game keeps its Silence");
    assert_eq!(g.winner, None);
}

// ------------------------------------------------------------------ Tier 6 and the default path
#[test]
fn tier_6_seats_clear_t6_going_on_every_path() {
    for key in ["kraum-tymna-blue-breach-midrange", "kinnan-bonder-mana-combo"] {
        // ready, protected line
        let mut g = table_with(&[key, "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Island", 2, false);
        lands(&mut g, P0, "Swamp", 2, false);
        lands(&mut g, P0, "Plains", 2, false);
        hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Silence"]);
        arm(&mut g, 2);
        let i = combos::combo_index("Oracle + Demonic Consultation").unwrap() as i64;
        assert!(t6_plans::is_t6(&g, P0));
        combos::attempt(&mut g, P0, i).unwrap();
        assert_eq!(g.t6_going, None, "{key}: after a ready line");
        // not ready (cards gone)
        let mut g = table_with(&[key, "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
        g.active = Some(P0);
        assert!(!combos::attempt(&mut g, P0, i).unwrap());
        assert_eq!(g.t6_going, None, "{key}: not ready");
        // the same line a second time in a turn
        let mut g = table_with(&[key, "sisay-legends-toolbox", "yshtola", "sauron"], 5, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Island", 2, false);
        lands(&mut g, P0, "Swamp", 2, false);
        hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
        let turns = g.player(P0).turns;
        g.player_mut(P0).combo_turn = Some((turns, combos::all()[i as usize].name));
        assert!(!combos::attempt(&mut g, P0, i).unwrap());
        assert_eq!(g.t6_going, None, "{key}: gone this turn");
    }
}

#[test]
fn the_knob_is_off_for_every_deck_key_but_yoshimaru() {
    let mut keys: Vec<&str> = decks().iter().map(|d| d.key.as_str()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let g = table_with(&[k, "sisay-legends-toolbox"], 1, 40);
        assert_eq!(plan_data::protect_before_go(&g, P0), k == "yoshimaru", "{k}");
        if t6_plans::is_t6(&g, P0) {
            assert!(!plan_data::protect_before_go(&g, P0), "{k}: a Tier 6 key never sets the knob");
        }
    }
}

#[test]
fn knob_parsing_edges() {
    let ok = |t: &str| plan_data::load("x", t).unwrap();
    assert_eq!(ok("deck = \"x\"\n[knobs]\nprotect_before_go = false").plan.knobs.protect_before_go, Some(false));
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_before_go = \"yes\"").contains("invalid type"));
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_before_go = 1").contains("invalid type"));
    assert!(bad("deck = \"x\"\n[knobs]\nprotect_before_go = true\nprotect_before_go = false").contains("duplicate"));
    // the knob outside [knobs] is an unknown field
    assert!(bad("deck = \"x\"\nprotect_before_go = true").contains("unknown field"));
}

// ------------------------------------------------------------------ the protect step must not take mana the line needs
/// Damping Sphere (an opponent's): a spell costs {1} more per OTHER spell its caster cast this turn. `mana_need` prices
/// the line's casts from zero spells cast, then Silence is cast first and the Ballista costs {1} more than `need` said.
#[test]
fn silence_does_not_make_the_line_unaffordable_under_damping_sphere() {
    let mut g = heliod_position(2, 3, 2); // 5 lands: need 4 + Silence 1 passes the protect check
    perm(&mut g, P1, "Damping Sphere");
    let i = line("Heliod, Sun-Crowned + Walking Ballista");
    assert!(combos::ready_to_go(&g, P0, i as usize), "the line is payable now");
    combos::attempt(&mut g, P0, i).unwrap();
    assert!(
        g.winner == Some(P0) || in_hand(&g, P0, "Silence"),
        "Silence was cast (gy: {}) and the line did not go: protection spent the mana of the line",
        in_gy(&g, P0, "Silence")
    );
}

/// a counterspell-holding table: whatever happens to Silence, the line must not be left half-paid and `attempt` must
/// say true when the turn state changed
#[test]
fn a_countered_silence_leaves_a_consistent_state() {
    for seed in 1..40u64 {
        let mut g = table_with(&["yoshimaru", "sisay-legends-toolbox", "yshtola", "sauron"], seed, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Plains", 2, false);
        lands(&mut g, P0, "Swamp", 4, false);
        perm(&mut g, P0, "Heliod, Sun-Crowned");
        hand(&mut g, P0, &["Walking Ballista", "Silence"]);
        for q in [P1, P2] {
            lands(&mut g, q, "Island", 4, false);
            hand(&mut g, q, &["Counterspell", "Mana Drain", "Force of Will"]);
        }
        let i = line("Heliod, Sun-Crowned + Walking Ballista");
        let r = combos::attempt(&mut g, P0, i).unwrap();
        let silence_cast = !in_hand(&g, P0, "Silence");
        if silence_cast && g.winner != Some(P0) {
            assert!(r, "seed {seed}: Silence was cast (state changed) but attempt returned false");
        }
        assert_eq!(g.t6_going, None);
    }
}

