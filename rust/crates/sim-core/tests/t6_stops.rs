//! `AI_T6STOPS` (Tier 6 fix 2, `audit/t6-stops/trace-2026-11-02.md`): attempts that stopped with no card to blame.
//! Seat 0 is the Tier 6 deck (Kraum/Tymna), seat 1 the table.

use sim_core::ai::t6_stops;
use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ORACLE: &str = "Oracle + Demonic Consultation";
const AD: &str = "Ad Nauseam into a win";

fn tbl() -> Game {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    g
}

fn oracle_hand(g: &mut Game) {
    hand(g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
}

fn idx(part: &str) -> usize {
    combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"))
}

fn stops(g: &Game) -> Vec<String> {
    g.player(P0).stats.keys().filter(|k| k.starts_with("stop|")).map(|k| k.to_string()).collect()
}

// ------------------------------------------------------------ a lock on the first spell
#[test]
fn a_void_winnower_makes_an_even_oracle_unplayable_so_the_line_is_not_ready() {
    let mut g = tbl();
    oracle_hand(&mut g);
    perm(&mut g, P1, "Void Winnower"); // opponents can't cast spells with an even mana value; the Oracle is 2
    t6_stops::force(Some(true));
    assert!(combos::ready(&g, P0, idx(ORACLE)), "the pieces and the mana are there");
    assert!(!combos::ready_to_go(&g, P0, idx(ORACLE)), "but the Oracle cannot be cast");
    t6_stops::force(Some(false));
    assert!(combos::ready_to_go(&g, P0, idx(ORACLE)), "AI_T6STOPS=0: the old test calls it ready");
    t6_stops::force(None);
}

// ------------------------------------------------------------ a lock on the second spell
#[test]
fn rule_of_law_stops_the_second_spell_of_the_line_so_the_line_is_not_ready() {
    let mut g = tbl();
    oracle_hand(&mut g);
    perm(&mut g, P1, "Rule of Law"); // one spell each turn
    t6_stops::force(Some(true));
    assert!(!combos::ready_to_go(&g, P0, idx(ORACLE)), "the Oracle is the one spell; Consultation is the second");
    t6_stops::force(Some(false));
    assert!(combos::ready_to_go(&g, P0, idx(ORACLE)));
    t6_stops::force(None);
}

#[test]
fn without_a_lock_the_line_is_ready_either_way() {
    let mut g = tbl();
    oracle_hand(&mut g);
    for on in [true, false] {
        t6_stops::force(Some(on));
        assert!(combos::ready_to_go(&g, P0, idx(ORACLE)), "switch {on}");
    }
    t6_stops::force(None);
}

// ------------------------------------------------------------ the stop names where the line failed
fn yes(_g: &mut Game, _p: PlayerId) -> sim_core::flow::Res<bool> {
    Ok(true)
}

fn refuse(_g: &mut Game, _p: PlayerId) -> sim_core::flow::Res<bool> {
    Ok(false)
}

fn no_src(_g: &Game, _p: PlayerId) -> Option<sim_core::ids::PermId> {
    None
}

fn no_keys(_g: &Game, _p: PlayerId) -> Vec<sim_core::ids::PermId> {
    vec![]
}

fn two_steps() -> combos::Combo {
    use combos::{Step, StepKind};
    let st: &'static [Step] = Box::leak(Box::new([
        Step { label: "first", kind: StepKind::Engine, src: no_src, pay: (0, ""), run: yes, keys: no_keys },
        Step { label: "second", kind: StepKind::Engine, src: no_src, pay: (0, ""), run: refuse, keys: no_keys },
    ]));
    combos::Combo {
        name: "synthetic",
        groups: &[],
        ready: |_, _| (true, vec![], vec![]),
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: st,
    }
}

#[test]
fn a_stop_with_no_card_says_which_step_failed() {
    let mut g = tbl();
    t6_stops::force(Some(true));
    combos::run_steps(&mut g, P0, &two_steps()).unwrap();
    assert_eq!(stops(&g), ["stop|synthetic|no card: step 2 (second)"]);
    t6_stops::force(Some(false));
    let mut g = tbl();
    combos::run_steps(&mut g, P0, &two_steps()).unwrap();
    assert_eq!(stops(&g), ["stop|synthetic|?"], "AI_T6STOPS=0: the old label");
    t6_stops::force(None);
}
// ------------------------------------------------------------ Ad Nauseam is a dig
fn ad_hand(g: &mut Game, lib: &[&str]) {
    lands(g, P0, "Mountain", 3, false); // seven lands: Ad Nauseam and the Oracle line
    lands(g, P0, "Swamp", 1, false);
    hand(g, P0, &["Ad Nauseam"]);
    let lib: Vec<_> = lib.iter().map(|n| card(g, n)).collect();
    g.player_mut(P0).library = lib;
}

#[test]
fn ad_nauseam_with_the_line_far_from_hand_is_not_ready() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    ad_hand(&mut g, &[]);
    // sixty cards, the Oracle, one exile spell and nothing else of use among them
    let mut lib = vec![card(&g, "Thassa's Oracle"), card(&g, "Demonic Consultation")];
    lib.extend(std::iter::repeat_n(card(&g, "Plains"), 58));
    g.player_mut(P0).library = lib;
    t6_stops::force(Some(false));
    assert!(combos::ready_to_go(&g, P0, idx(AD)), "AI_T6STOPS=0: pieces in the library are enough");
    t6_stops::force(Some(true));
    assert!(!combos::ready_to_go(&g, P0, idx(AD)), "two cards in sixty, nine draws: under the odds");
    t6_stops::force(None);
}

#[test]
fn ad_nauseam_with_one_piece_in_hand_and_the_other_likely_is_ready() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    ad_hand(&mut g, &[]);
    hand(&mut g, P0, &["Ad Nauseam", "Thassa's Oracle"]);
    // ten cards in the library, one exile spell among them: nine draws find it
    let mut lib = vec![card(&g, "Demonic Consultation")];
    lib.extend(std::iter::repeat_n(card(&g, "Plains"), 9));
    g.player_mut(P0).library = lib;
    t6_stops::force(Some(true));
    assert!(combos::ready_to_go(&g, P0, idx(AD)));
    t6_stops::force(None);
}

#[test]
fn ad_nauseam_with_a_piece_gone_from_the_deck_is_not_ready() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    ad_hand(&mut g, &["Demonic Consultation", "Plains", "Plains"]); // no Oracle anywhere, no Breach line
    t6_stops::force(Some(true));
    assert!(!combos::ready_to_go(&g, P0, idx(AD)));
    t6_stops::force(None);
}

#[test]
fn the_odds_of_finding_the_pieces() {
    use sim_core::impls::t6_combos_extra::find_all;
    assert!((find_all(10, 10, &[1]) - 1.0).abs() < 1e-9);
    assert_eq!(find_all(10, 0, &[1]), 0.0);
    assert_eq!(find_all(10, 5, &[0]), 0.0, "a piece that is not in the library");
    assert!((find_all(4, 2, &[1, 1]) - 1.0 / 6.0).abs() < 1e-9, "both of two named cards in two of four");
    assert!((find_all(10, 5, &[]) - 1.0).abs() < 1e-9, "nothing to find");
    assert!((find_all(10, 5, &[1]) - 0.5).abs() < 1e-9);
    assert!((find_all(10, 5, &[10]) - 1.0).abs() < 1e-9);
}
