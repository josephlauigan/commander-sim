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
