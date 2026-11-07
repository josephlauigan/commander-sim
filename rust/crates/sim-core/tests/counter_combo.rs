//! engine-followups (2026-11-06): counterspells against a table near a combo. With an opponent near a catalogued combo
//! (half its pieces public, or a combo-piece commander out), an outside-Tier-6 deck keeps its last two counters for the
//! combo: no counter for a spell that is no combo piece, no tutor and not important. The 11/06 Aragorn traces: Mana
//! Drain on a Goblin Ringleader, Mystic Confluence on an Esper Sentinel while Urza assembled Basalt Monolith +
//! Forensic Gadgeteer.

use sim_core::ai::brain;
use sim_core::engine::cast;
use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const KINNAN: &str = "kinnan-bonder-mana-combo";

/// P0 (Aragorn) holds one Counterspell with two Islands up; P1 (Kinnan's deck) is in its main phase with four lands
fn table_vs_kinnan() -> Game {
    let mut g = table(&["aragorn", KINNAN]);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Counterspell"]);
    lands(&mut g, P0, "Island", 2, false);
    g.player_mut(P1).hand.clear();
    lands(&mut g, P1, "Forest", 4, false);
    g.active = Some(P1);
    g.step = Step::Main1;
    g
}

fn kinnan_out(g: &mut Game) {
    let k = perm(g, P1, "Kinnan, Bonder Prodigy");
    g.perm_mut(k).is_cmd = true;
    g.player_mut(P1).cmd_in_zone = false;
}

#[test]
fn half_a_combo_on_the_table_is_near() {
    let mut g = table_vs_kinnan();
    assert!(!brain::near_combo_table(&g, P0), "nothing of Kinnan's combo out");
    perm(&mut g, P1, "Basalt Monolith");
    assert!(combos::public_progress(&g, P1) >= combos::PUBLIC_NEAR);
    assert!(brain::near_combo_table(&g, P0));
}

#[test]
fn the_last_counter_waits_for_the_combo() {
    let mut g = table_vs_kinnan();
    kinnan_out(&mut g);
    assert!(brain::combo_commander_out(&g, P0));
    let c = hand(&mut g, P1, &["Elvish Mystic"])[0];
    cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    assert!(
        g.player(P0).hand.iter().any(|&x| &*g.db.get(x).name == "Counterspell"),
        "Counterspell kept for the combo, not spent on an Elvish Mystic"
    );
    assert!(g.player(P1).perms.iter().any(|&m| g.perm(m).cd == Some(c)), "the Mystic resolved");
}

#[test]
fn a_combo_piece_is_still_worth_the_counter_check() {
    // the rule only lets ordinary spells through: a combo piece goes on to the usual decision (which may counter it)
    let mut g = table_vs_kinnan();
    kinnan_out(&mut g);
    let bm = card(&g, "Basalt Monolith");
    assert!(sim_core::cardcode::is_combo_piece(&g, bm));
}
