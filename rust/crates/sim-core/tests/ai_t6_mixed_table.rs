//! A Tier 6 deck at a table of another tier (JD, 2026-11-01): the cEDH pod's rules (every win step countered, Ad
//! Nauseam a win spell, the quiet lines' progress) are how Tier 6 decks read each other. A deck of any other tier reads
//! a Tier 6 player by the generic rules, as it reads anyone.

use sim_core::ai::t6_table::{self, Verdict};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const SISAY: &str = "sisay-legends-toolbox";
const URZA: &str = "urza-mono-blue-artifacts"; // Tier 5
const ZUR: &str = "zur-esper-enchantment-control"; // Tier 5

fn tbl(keys: &[&str]) -> Game {
    let mut g = table(keys);
    g.active = Some(P0);
    g
}

#[test]
fn a_tier_6_win_step_is_countered_by_the_tier_6_table_and_rolled_by_anyone_else() {
    // P0 (Kraum/Tymna) is going for its line; P1 plays Tier 6, P2 Tier 5
    let mut g = tbl(&[KRAUM, SISAY, URZA]);
    let c = card(&g, "Demonic Consultation");
    let it = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), 5.0);
    g.t6_going = Some(P0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 5.0, 4.0, 1), Verdict::Counter);
    assert_eq!(t6_table::counter_verdict(&g, P2, &it, 5.0, 4.0, 1), Verdict::Roll(5.0, 4.0));
}

#[test]
fn ad_nauseam_is_a_win_spell_only_to_the_tier_6_table() {
    let mut g = tbl(&[KRAUM, SISAY, URZA]);
    let c = card(&g, "Ad Nauseam");
    let imp = sim_core::ai::spell_importance(&g, P0, c);
    assert_eq!(imp, 8.0, "a Tier 6 caster's Ad Nauseam keeps its importance");
    let it = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), imp);
    // the Tier 6 counterer: the win spell, always countered
    assert_eq!(sim_core::engine::stack::imp_seen_by(&g, P1, &it), 8.0);
    assert_eq!(sim_core::engine::stack::value_seen_by(&g, P1, &it), 8.0);
    assert_eq!(t6_table::counter_verdict(&g, P1, &it, 8.0, 4.0, 1), Verdict::Counter);
    // the Tier 5 counterer: the importance the generic rules give the same spell
    let generic = sim_core::ai::combo_imp(&g, P0, c).max(sim_core::ai::pool::spell_importance_generic(&g, P0, c));
    assert!(generic < 8.0, "generic importance {generic}");
    assert_eq!(sim_core::engine::stack::imp_seen_by(&g, P2, &it), generic);
    assert_eq!(sim_core::engine::stack::value_seen_by(&g, P2, &it), generic);
    assert!(matches!(t6_table::counter_verdict(&g, P2, &it, generic, 4.0, 1), Verdict::Roll(..)));
}

#[test]
fn a_tier_5_reading_is_the_same_whoever_casts_the_spell() {
    // Brainstorm from a Tier 6 or a Tier 5 player: a Tier 5 counterer reads its plain importance either way
    let mut g = tbl(&[KRAUM, URZA, ZUR]);
    let c = card(&g, "Brainstorm");
    let from_t6 = sim_core::engine::stack::spell_item(&mut g, P0, c, Ctx::default(), 3.0);
    let from_t5 = sim_core::engine::stack::spell_item(&mut g, P2, c, Ctx::default(), 3.0);
    assert_eq!(sim_core::engine::stack::imp_seen_by(&g, P1, &from_t6), 3.0);
    assert_eq!(sim_core::engine::stack::imp_seen_by(&g, P1, &from_t5), 3.0);
}

#[test]
fn a_tier_6_players_quiet_line_counts_to_tier_6_viewers_only() {
    // Ioreth + Marvin + a legend + Mount Doom (Sisay) is a quiet line: the Tier 6 reading sees it, the generic one doesn't
    let mut g = tbl(&[SISAY, KRAUM, URZA]);
    perm(&mut g, P0, "Ioreth of the Healing House");
    perm(&mut g, P0, "Marvin, Murderous Mimic");
    let own = sim_core::cardcode::combo_progress(&g, P0);
    let by_t6 = sim_core::cardcode::combo_progress_seen(&g, P1, P0);
    let by_t5 = sim_core::cardcode::combo_progress_seen(&g, P2, P0);
    assert!(own > 0.0, "Sisay reads its own quiet line: {own}");
    assert_eq!(by_t6, own, "a Tier 6 viewer reads it as Godo does");
    assert!(by_t5 < by_t6, "a Tier 5 viewer reads it by the generic rule: {by_t5} against {by_t6}");
    // and the look-ahead's strength follows the combo term
    let s6 = sim_core::ai::search::strength_seen(&g, P1, P0);
    let s5 = sim_core::ai::search::strength_seen(&g, P2, P0);
    assert_eq!(sim_core::ai::search::strength(&g, P0), s6);
    assert!(s5 < s6, "{s5} against {s6}");
}

#[test]
fn readings_between_tier_5_players_and_of_tier_5_players_are_unchanged() {
    // nothing Tier 6 is read: the viewer doesn't matter
    let mut g = tbl(&[URZA, ZUR, KRAUM]);
    perm(&mut g, P0, "Urza, Lord High Artificer");
    for viewer in [P0, P1, P2] {
        assert_eq!(
            sim_core::cardcode::combo_progress_seen(&g, viewer, P0),
            sim_core::cardcode::combo_progress(&g, P0),
            "viewer {viewer:?}"
        );
        assert_eq!(sim_core::ai::search::strength_seen(&g, viewer, P0), sim_core::ai::search::strength(&g, P0));
    }
}
