//! AI audit phase 2 (suffix "ysh"): the kraum seat's fixes. Dark Endurance, Underworld Breach, Baird, the Diabolic Tutor
//! pick, Firebird and Lightning.

use sim_core::engine::{life, removal, values, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn kraum_tbl() -> Game {
    table(&["thrasios", "galadriel", "veyran", "seph"])
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

// ---------------------------------------------------------------- Dark Endurance
#[test]
fn dark_endurance_is_not_cast_against_exile_or_for_a_small_creature() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Dark Endurance"]);
    let swords = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), t, "exile", Some(swords)).unwrap();
    assert!(!on(&g, t), "exile isn't stopped by indestructible");
    assert_eq!(g.player(P0).hand.len(), 1, "the card was kept");
    let tok = token(&mut g, P0, 1);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), tok, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, tok));
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn dark_endurance_is_only_for_the_tymna_partner_seats() {
    let mut g = table(&["galadriel", "veyran", "seph", "sauron"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Dark Endurance"]);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, t));
}

// ---------------------------------------------------------------- Underworld Breach
// (dropped 2026-10-19 with the Kraum deck: breach_candidates' wipe rule is keyed to the `kraum` seat, which no longer exists)

// ---------------------------------------------------------------- Baird
// ---------------------------------------------------------------- Diabolic Tutor
// ---------------------------------------------------------------- Firebird, Lightning
#[test]
fn firebird_pumps_the_other_attackers() {
    let mut g = kraum_tbl();
    g.active = Some(P0);
    let f = perm(&mut g, P0, "Firebird, Blazing Ranger");
    let a = perm(&mut g, P0, "Aven Mindcensor");
    let before = values::epow(&g, a);
    let imp = g.registry.get(g.perm(f).cd.unwrap()).unwrap().attack.unwrap();
    imp(&mut g, f, P0, &[f, a], P1).unwrap();
    assert_eq!(values::epow(&g, a), before + values::epow(&g, f));
    assert_eq!(values::epow(&g, f), 1, "Firebird itself isn't pumped");
}

#[test]
fn lightning_exiles_a_card_playable_while_it_stays_and_loses_them_when_it_leaves() {
    let mut g = kraum_tbl();
    g.active = Some(P0);
    let l = perm(&mut g, P0, "Lightning, Security Sergeant");
    let c = card(&g, "Sol Ring");
    g.player_mut(P0).library.push(c);
    let imp = g.registry.get(g.perm(l).cd.unwrap()).unwrap().combat_damage.unwrap();
    imp(&mut g, l, P0, l, P1, 2).unwrap();
    assert!(g.player(P0).hand.contains(&c), "waits in the hand");
    assert!(!g.player(P0).library.contains(&c));
    // survives end-of-turn cleanup of impulse cards
    sim_core::engine::turn::end_step(&mut g, P0).ok();
    assert!(g.player(P0).hand.contains(&c), "still playable on later turns");
    zones::die(&mut g, l, "destroy").unwrap();
    assert!(!g.player(P0).hand.contains(&c));
    assert!(g.player(P0).exile.contains(&c), "exiled for good");
}

// ================================================================== Y'shtola
use sim_core::ai::brain::{self, Situation};
use sim_core::hooks::Action;
use sim_core::impls::yshtola as ysh_mod;

fn ysh_tbl() -> Game {
    table(&["yshtola", "veyran", "sauron"])
}

fn ysh_out(g: &mut Game, p: PlayerId) -> PermId {
    let cmd = g.player(p).cmd;
    let m = zones::enter(g, p, cmd, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

#[test]
fn spot_removal_needs_a_worthwhile_target_and_a_three_mana_spell_counts_her_trigger() {
    let mut g = ysh_tbl();
    let path = card(&g, "Path to Exile");
    let unmaking = card(&g, "Anguished Unmaking");
    assert_eq!(ysh_mod::removal_adjust(&g, P0, path, 1.5), None, "Path on a mana-rock sized target: held");
    assert_eq!(ysh_mod::removal_adjust(&g, P0, path, 5.0), Some(0.0));
    // without her out the MV3 instant needs a real target too
    assert_eq!(ysh_mod::removal_adjust(&g, P0, unmaking, 1.5), None);
    ysh_out(&mut g, P0);
    let b = ysh_mod::removal_adjust(&g, P0, unmaking, 1.5).expect("with her out the trigger is the value");
    assert!(b > 0.0, "{b}");
    assert_eq!(ysh_mod::removal_adjust(&g, P0, path, 1.5), None, "Path is MV1: no trigger");
}

#[test]
fn a_trigger_that_makes_four_is_worth_more() {
    let mut g = ysh_tbl();
    ysh_out(&mut g, P0);
    let c = card(&g, "Anguished Unmaking");
    let before = ysh_mod::trigger_bonus(&g, P0, c);
    life::lose_life(&mut g, P1, 2, Some(P0), "triggers", None).unwrap();
    assert!(ysh_mod::trigger_bonus(&g, P0, c) > before, "an opponent has lost 2 this turn");
}

#[test]
fn farewell_is_cast_with_teferis_protection_held_up_behind_it() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = ysh_tbl();
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 9, false);
    let y = ysh_out(&mut g, P0);
    let sb = perm(&mut g, P0, "Sanguine Bond");
    let far = hand(&mut g, P0, &["Farewell", "Teferi's Protection"]);
    let bane = perm(&mut g, P1, "Baneslayer Angel");
    assert!(ysh_mod::teferis_first(&g, P0, far[0], "farewell"), "9 mana: Farewell and Teferi's");
    let s = Situation::new(&g, P0);
    let u = brain::wipe_options(&g, P0, &s, false);
    assert!(u.iter().any(|o| o.label == "Farewell"));
    assert!(matches!(u[0].act, Some(Action::Wipe { .. })));
    brain::cast_wipe(&mut g, P0, far[0], None).unwrap();
    assert!(!g.perm(bane).on_bf, "the Angel is gone");
    assert!(g.perm(y).on_bf && g.perm(y).phased && g.perm(sb).on_bf && g.perm(sb).phased, "hers phased out");
    assert!(g.player(P0).life_locked);
    assert!(!g.player(P0).hand.contains(&far[1]));
}

#[test]
fn farewell_without_the_mana_for_teferis_is_not_planned_around_it() {
    let mut g = ysh_tbl();
    lands(&mut g, P0, "Plains", 6, false);
    ysh_out(&mut g, P0);
    let far = hand(&mut g, P0, &["Farewell", "Teferi's Protection"]);
    perm(&mut g, P0, "Sanguine Bond");
    perm(&mut g, P1, "Baneslayer Angel");
    assert!(!ysh_mod::teferis_first(&g, P0, far[0], "farewell"));
}

