//! Engine feature `combo3`: the declared steps of Thassa's Oracle + Consultation / Pact and Kinnan + Basalt Monolith, the
//! missing cEDH lines (Breach loop, Ad Nauseam, Hullbreaker Horror, the Kinnan outlets) and the Godo leftovers
//! (impls/t6_combos_extra.rs). Seat 0 plays the combo, seat 1 holds the answers.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl(keys: &[&str]) -> Game {
    let mut g = table(keys);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn go(g: &mut Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
    assert!(combos::ready(g, P0, i), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap()
}

fn not_ready(g: &Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap();
    !combos::ready(g, P0, i)
}

fn blue_black(g: &mut Game, p: PlayerId) {
    lands(g, p, "Island", 2, false);
    lands(g, p, "Swamp", 2, false);
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ============================================================ Thassa's Oracle + Consultation / Pact
const ORACLE: &str = "Oracle + Demonic Consultation";

fn oracle_table(lib: &[&str]) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let lib: Vec<_> = lib.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    g
}

#[test]
fn oracle_with_consultation_has_three_declared_steps_and_wins() {
    let i = combos::combo_index(ORACLE).unwrap();
    assert_eq!(combos::all()[i].steps.len(), 3);
    let mut g = oracle_table(&["Sol Ring", "Counterspell", "Brainstorm"]);
    go(&mut g, ORACLE);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P0).library.is_empty(), "Consultation exiled the library");
}

#[test]
fn a_tidebinder_on_the_oracle_trigger_stops_the_line_after_the_library_is_gone() {
    let mut g = oracle_table(&["Sol Ring", "Counterspell", "Brainstorm"]);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, ORACLE);
    assert_eq!(g.winner, None, "the trigger was countered");
    assert!(on_bf(&g, P0, "Thassa's Oracle").is_some());
    assert!(g.player(P0).library.is_empty());
}

#[test]
fn a_counterspell_on_consultation_leaves_the_library_alone() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["kraum-tymna-blue-breach-midrange", "seph"], seed, 40);
        g.active = Some(P0);
        blue_black(&mut g, P0);
        hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
        let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
        g.player_mut(P0).library = lib;
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, ORACLE);
        if g.winner.is_none() {
            countered += 1;
            assert_eq!(g.player(P0).library.len(), 2, "a counter stopped the line before the exile or on the Oracle");
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

// ============================================================ Kinnan + Basalt Monolith and its outlets
const KB: &str = "Kinnan + Basalt Monolith";

fn kinnan_table() -> Game {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    g
}

#[test]
fn kinnan_basalt_has_steps_and_wins_with_the_outlet_paid() {
    let i = combos::combo_index(KB).unwrap();
    assert_eq!(combos::all()[i].steps.len(), 2);
    let mut g = kinnan_table();
    assert!(not_ready(&g, KB), "Kinnan's ability needs {{U}}{{G}} from a real source");
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    go(&mut g, KB);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_guildmage_on_kinnans_activation_stops_the_loop() {
    let mut g = kinnan_table();
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, KB);
    assert_eq!(g.winner, None, "the outlet was countered");
}

#[test]
fn thrasios_draws_the_deck_when_kinnans_ability_cant_be_paid() {
    let mut g = kinnan_table();
    assert!(not_ready(&g, "Thrasios (draw"), "no Thrasios");
    perm(&mut g, P0, "Thrasios, Triton Hero");
    go(&mut g, "Thrasios (draw");
    assert_eq!(g.winner, Some(P0));
    let mut g = kinnan_table();
    perm(&mut g, P0, "Thrasios, Triton Hero");
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "Thrasios (draw");
    assert_eq!(g.winner, None, "Thrasios's ability was countered");
}

#[test]
fn mirage_mirror_turns_the_loop_into_blue_green() {
    let mut g = kinnan_table();
    perm(&mut g, P0, "Mirage Mirror");
    go(&mut g, "Mirage Mirror");
    assert_eq!(g.winner, Some(P0));
    let mut g = kinnan_table();
    perm(&mut g, P0, "Mirage Mirror");
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "Mirage Mirror");
    assert_eq!(g.winner, None, "the Mirror's ability was countered");
}

// ============================================================ Underworld Breach + LED + Brain Freeze
const BREACH_O: &str = "Brain Freeze (Thassa";
const BREACH_T: &str = "Brain Freeze (at the table)";

fn breach_table(oracle: bool) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Brainstorm", "Ponder", "Preordain"]);
    if oracle {
        hand(&mut g, P0, &["Thassa's Oracle"]);
    }
    g
}

#[test]
fn the_breach_loop_empties_the_library_and_escapes_the_oracle() {
    let mut g = breach_table(true);
    assert!(not_ready(&g, BREACH_T), "the Oracle is in hand: that line is the other one");
    go(&mut g, BREACH_O);
    assert_eq!(g.winner, Some(P0), "an escaped Thassa's Oracle on an empty library");
    assert!(g.player(P0).library.is_empty());
    assert_eq!(g.wintype, Some("combo"));
}

#[test]
fn the_breach_loop_without_the_oracle_brain_freezes_the_table() {
    let mut g = breach_table(false);
    assert!(not_ready(&g, BREACH_O));
    go(&mut g, BREACH_T);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P1).library.is_empty(), "every copy past the fuel milled the opponent");
}

#[test]
fn the_breach_loop_needs_fuel_and_red() {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Brainstorm", "Ponder", "Preordain"]);
    assert!(not_ready(&g, BREACH_T), "no {{R}} for Breach");
    let mut g = breach_table(false);
    let db = g.db.clone();
    g.player_mut(P0).hand.retain(|&c| !["Brainstorm", "Ponder", "Preordain"].contains(&&*db.get(c).name));
    assert!(not_ready(&g, BREACH_T), "three cards to exile are needed to escape");
}

#[test]
fn a_counterspell_on_a_loop_spell_stops_the_breach_line() {
    let mut stopped = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["kraum-tymna-blue-breach-midrange", "seph"], seed, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Mountain", 1, false);
        lands(&mut g, P0, "Island", 1, false);
        hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Brainstorm", "Ponder", "Preordain", "Thassa's Oracle"]);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, BREACH_O);
        if g.winner.is_none() {
            stopped += 1;
            assert!(!g.player(P0).library.is_empty() || on_bf(&g, P0, "Thassa's Oracle").is_none());
        }
    }
    assert!(stopped >= 1, "the AI counters a loop spell sometimes");
}

// ============================================================ Ad Nauseam
const AD: &str = "Ad Nauseam into a win";

fn ad_table(top: &[&str]) -> Game {
    ad_table_with(top, &[])
}

fn ad_table_with(top: &[&str], bottom: &[&str]) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Ad Nauseam"]);
    let tops: Vec<_> = top.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.extend(tops); // the last one is the top
    let bots: Vec<_> = bottom.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.splice(0..0, bots);
    g
}

#[test]
fn ad_nauseam_draws_into_the_oracle_line_and_wins() {
    let mut g = ad_table(&["Thassa's Oracle", "Demonic Consultation"]);
    go(&mut g, AD);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P0).life < 40, "Ad Nauseam cost life");
}

#[test]
fn ad_nauseam_that_finds_nothing_ends_without_a_win() {
    let mut g = ad_table_with(&[], &["Thassa's Oracle", "Demonic Consultation"]); // at the very bottom: out of reach
    go(&mut g, AD);
    assert_eq!(g.winner, None);
    assert!(in_gy(&g, P0, "Ad Nauseam"));
}

#[test]
fn ad_nauseam_needs_life_and_a_line_in_the_deck() {
    let mut g = ad_table(&["Thassa's Oracle", "Demonic Consultation"]);
    g.player_mut(P0).life = 25;
    assert!(not_ready(&g, AD), "above 25 life");
}

#[test]
fn a_counterspell_on_ad_nauseam_stops_the_line() {
    let mut stopped = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["kraum-tymna-blue-breach-midrange", "seph"], seed, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Swamp", 3, false);
        lands(&mut g, P0, "Island", 2, false);
        lands(&mut g, P0, "Mountain", 3, false);
        hand(&mut g, P0, &["Ad Nauseam", "Thassa's Oracle"]); // with the Oracle in hand the table sees what Ad Nauseam finds
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, AD);
        if g.winner.is_none() && g.player(P0).life == 40 {
            stopped += 1;
        }
    }
    assert!(stopped >= 1, "countered before it drew anything");
}

// ============================================================ Hullbreaker Horror
const HORROR: &str = "Hullbreaker Horror + Kinnan";

fn horror_table() -> Game {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Hullbreaker Horror");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Mox Amber"]);
    g
}

#[test]
fn horror_with_sol_ring_out_and_mox_amber_in_hand_loops() {
    let mut g = horror_table();
    go(&mut g, HORROR);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn horror_can_be_flashed_in_and_lotus_petal_is_a_partner() {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Mox Amber");
    hand(&mut g, P0, &["Hullbreaker Horror", "Lotus Petal"]);
    lands(&mut g, P0, "Island", 7, false);
    go(&mut g, HORROR);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Hullbreaker Horror").is_some());
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Hullbreaker Horror", "Mox Amber"]);
    lands(&mut g, P0, "Island", 2, false);
    assert!(not_ready(&g, HORROR), "Horror costs seven; two lands and a Kinnan-doubled Sol Ring make six");
}

#[test]
fn a_tidebinder_on_the_horror_trigger_stops_the_loop() {
    let mut g = horror_table();
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, HORROR);
    assert_eq!(g.winner, None);
}

#[test]
fn two_rocks_that_dont_pair_are_no_loop() {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Hullbreaker Horror");
    perm(&mut g, P0, "Lotus Petal");
    hand(&mut g, P0, &["Fellwar Stone"]);
    assert!(not_ready(&g, HORROR), "Petal and Stone are both in the second list");
}
