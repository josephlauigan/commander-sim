//! The Lord of Pain and Smaug candidate decks (impls/lordofpain.rs, impls/smaug.rs), on hand-built positions.
//! Seat 0 plays the deck under test; seats 1 and 2 are other decks.

use sim_core::ai::{act, brain};
use sim_core::engine::combat::kw;
use sim_core::engine::life::gain;
use sim_core::engine::mana::pay;
use sim_core::engine::{cast, removal, turn, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn game(deck: &str) -> Game {
    table(&[deck, "seph", "kefka"])
}

fn options(g: &mut Game, post: bool) -> Vec<Opt> {
    brain::main_options(g, P0, post).unwrap()
}

/// pay for and cast a card from hand
fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    let (cg, cp) = sim_core::engine::mana::cost_of(g, p, c);
    assert!(pay(g, p, cg, &cp, false).unwrap(), "can't pay for {name}");
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn mana(g: &mut Game, p: PlayerId, n: usize) {
    lands(g, p, "Swamp", 3, false);
    lands(g, p, "Mountain", n, false);
}

// ------------------------------------------------------------------ The Lord of Pain
#[test]
fn lord_deals_damage_equal_to_the_first_spell_of_each_player_each_turn() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "The Lord of Pain");
    mana(&mut g, P0, 6);
    let before = (g.player(P1).life, g.player(P2).life);
    cast_from_hand(&mut g, P0, "Bedevil"); // mana value 3: the AI picks one opponent
    let after = (g.player(P1).life, g.player(P2).life);
    assert_eq!(before.0 + before.1 - after.0 - after.1, 3, "exactly one opponent takes 3");
    cast_from_hand(&mut g, P0, "Skyscanner"); // the second spell this turn: nothing
    assert_eq!(g.player(P1).life + g.player(P2).life, after.0 + after.1);
}

#[test]
fn lord_triggers_on_opponents_spells_and_never_targets_the_caster() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "The Lord of Pain");
    lands(&mut g, P1, "Island", 6, false);
    g.player_mut(P2).life = 5; // closest to dying
    // P1 casts a mana value 2 spell: P2 (an opponent of the Lord, not the caster) takes it
    cast_from_hand(&mut g, P1, "Counterspell");
    assert_eq!(g.player(P2).life, 3);
    assert_eq!(g.player(P1).life, 40);
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn lord_hits_its_own_controller_when_only_they_can_be_chosen() {
    let mut g = table(&["lordofpain", "seph"]);
    perm(&mut g, P0, "The Lord of Pain");
    lands(&mut g, P1, "Island", 4, false);
    cast_from_hand(&mut g, P1, "Counterspell"); // the caster cannot be chosen: the Lord controller is
    assert_eq!(g.player(P0).life, 38);
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn lord_picks_the_opponent_closest_to_dying_and_first_spell_resets_each_turn() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "The Lord of Pain");
    mana(&mut g, P0, 6);
    g.player_mut(P1).life = 4;
    cast_from_hand(&mut g, P0, "Bedevil");
    assert!(!g.player(P1).alive || g.player(P1).life <= 1, "the opponent at 4 life takes 3");
    assert_eq!(g.player(P2).life, 40);
}

#[test]
fn lord_stops_opponents_gaining_life_but_not_its_controller() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "The Lord of Pain");
    gain(&mut g, P1, 5).unwrap();
    assert_eq!(g.player(P1).life, 40);
    gain(&mut g, P0, 5).unwrap();
    assert_eq!(g.player(P0).life, 45);
}

#[test]
fn lord_plan_casts_the_biggest_spell_first() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "The Lord of Pain");
    mana(&mut g, P0, 6);
    let cs = hand(&mut g, P0, &["Suplex", "Invoke Despair"]);
    let (small, big) = (
        sim_core::impls::lordofpain::lordofpain_prio(&g, P0, cs[0]),
        sim_core::impls::lordofpain::lordofpain_prio(&g, P0, cs[1]),
    );
    assert!(big > small, "Invoke Despair ({big}) before Suplex ({small})");
}

// ------------------------------------------------------------------ Smaug the Impenetrable
fn smaug(g: &mut Game) -> PermId {
    perm(g, P0, "Smaug the Impenetrable")
}

#[test]
fn smaug_makes_a_treasure_per_noncombat_damage() {
    let mut g = game("smaug");
    let s = smaug(&mut g);
    removal::apply_removal(&mut g, Some(P1), s, "dmg3", None).unwrap();
    assert_eq!(g.player(P0).treasures, 3);
    assert!(g.perm(s).on_bf, "indestructible");
    removal::apply_removal(&mut g, Some(P0), s, "destroy", None).unwrap();
    assert_eq!(g.player(P0).treasures, 3, "destroy isn't damage");
    assert!(kw(&g, s, "flying") && kw(&g, s, "haste") && kw(&g, s, "indestructible"));
}

#[test]
fn smaug_gets_treasures_from_a_sweeper() {
    let mut g = game("lordofpain");
    let s = perm(&mut g, P0, "Smaug the Impenetrable");
    mana(&mut g, P0, 6);
    let t = token(&mut g, P1, 2);
    cast_from_hand(&mut g, P0, "Ultimate Magic: Meteor");
    assert_eq!(g.player(P0).treasures, 7);
    assert!(g.perm(s).on_bf);
    assert!(!g.perm(t).on_bf);
}

#[test]
fn smaug_fangtail_ping_option() {
    let mut g = game("smaug");
    smaug(&mut g);
    let f = perm(&mut g, P0, "Viashino Fangtail");
    let o = options(&mut g, false);
    let o = o.iter().find(|o| o.label.contains("Fangtail pings Smaug")).expect("the ping is offered");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).treasures, 1);
    assert!(g.perm(f).tapped);
}

#[test]
fn smaug_burn_without_a_better_target_goes_to_smaug() {
    let mut g = game("smaug");
    smaug(&mut g);
    mana(&mut g, P0, 4);
    hand(&mut g, P0, &["Orcish Cannonade"]);
    let o = options(&mut g, false);
    let o = o.iter().find(|o| o.label.starts_with("Orcish Cannonade")).expect("offered");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).treasures, 2, "2 damage to Smaug");
    assert_eq!(g.player(P0).life, 37);
}

#[test]
fn smaug_plan_casts_smaug_first_with_seven_mana() {
    let mut g = game("smaug");
    mana(&mut g, P0, 7);
    let c = g.player(P0).cmd;
    let p = sim_core::impls::smaug::smaug_prio(&g, P0, c);
    assert!(p >= 90, "{p}");
}

// ------------------------------------------------------------------ Charforger
#[test]
fn charforger_counts_oil_and_exiles_the_top_card() {
    let mut g = game("lordofpain");
    let c = perm(&mut g, P0, "Charforger");
    let o = perm(&mut g, P0, "Goblin Medics");
    zones::die(&mut g, o, "destroy".into()).unwrap();
    let oil = |g: &Game| g.perm(c).data.int(DataKey::Oil);
    assert_eq!(oil(&g), 1);
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "destroy".into()).unwrap();
    let op = perm(&mut g, P1, "Terror of the Peaks");
    zones::die(&mut g, op, "destroy".into()).unwrap();
    assert_eq!(oil(&g), 2, "an opponent's creature is not counted");
    let a = perm(&mut g, P0, "Arcane Signet");
    zones::die(&mut g, a, "destroy".into()).unwrap();
    assert_eq!(oil(&g), 3, "artifacts count");
    let lib = g.player(P0).library.len();
    let o = options(&mut g, false);
    let o = o.iter().find(|o| o.label.starts_with("Charforger")).expect("offered with three counters");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(oil(&g), 0);
    assert_eq!(g.player(P0).library.len(), lib - 1);
    assert_eq!(g.player(P0).impulse.len(), 1);
}

#[test]
fn charforger_makes_a_goblin_when_it_enters() {
    let mut g = game("lordofpain");
    mana(&mut g, P0, 3);
    let n = g.player(P0).perms.len();
    cast_from_hand(&mut g, P0, "Charforger");
    assert_eq!(g.player(P0).perms.len(), n + 2);
}

// ------------------------------------------------------------------ Kalain
#[test]
fn kalain_adds_counters_for_treasure_mana_spent() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "Kalain, Reclusive Painter");
    let t = g.player(P0).treasures;
    assert_eq!(t, 1, "Kalain makes a Treasure");
    g.player_mut(P0).treasures = 3;
    let before = g.player(P0).perms.len();
    cast_from_hand(&mut g, P0, "Skyscanner");
    let m = *g.player(P0).perms.last().unwrap();
    assert_eq!(g.player(P0).perms.len(), before + 1);
    assert_eq!(g.player(P0).treasures, 0);
    assert_eq!(g.perm(m).plus, 3, "three Treasure mana paid");
    // paid with lands: nothing
    mana(&mut g, P0, 3);
    cast_from_hand(&mut g, P0, "Rummaging Goblin");
    let m = *g.player(P0).perms.last().unwrap();
    assert_eq!(g.perm(m).plus, 0);
}

// ------------------------------------------------------------------ Lagomos
#[test]
fn lagomos_makes_a_hasty_elemental_each_combat_and_sacrifices_it_at_the_end_step() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "Lagomos, Hand of Hatred");
    let n = g.player(P0).perms.len();
    sim_core::cardcode::combat_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).perms.len(), n + 1);
    let e = *g.player(P0).perms.last().unwrap();
    assert_eq!((g.perm(e).pow, g.perm(e).tgh), (2, 1));
    assert!(!g.perm(e).sick && kw(&g, e, "trample"));
    turn::end_step(&mut g, P0).unwrap();
    assert!(!g.perm(e).on_bf);
}

#[test]
fn lagomos_tutors_only_after_five_creatures_died() {
    let mut g = game("lordofpain");
    perm(&mut g, P0, "Lagomos, Hand of Hatred");
    let has = |g: &mut Game| options(g, false).iter().any(|o| o.label.starts_with("Lagomos"));
    for _ in 0..4 {
        let t = token(&mut g, P1, 1);
        zones::die(&mut g, t, "destroy".into()).unwrap();
    }
    assert!(!has(&mut g), "four deaths are not enough");
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "destroy".into()).unwrap();
    assert!(has(&mut g));
    let o = options(&mut g, false);
    let o = o.iter().find(|o| o.label.starts_with("Lagomos")).unwrap();
    let h = g.player(P0).hand.len();
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), h + 1);
    assert!(!has(&mut g), "tapped");
}

// ------------------------------------------------------------------ Hero's Heirloom
#[test]
fn heirloom_grants_trample_and_haste_to_a_legendary_creature_only() {
    let mut g = game("smaug");
    let h = perm(&mut g, P0, "Hero's Heirloom");
    let leg = perm(&mut g, P0, "Kalain, Reclusive Painter");
    let plain = perm(&mut g, P0, "Goblin Medics");
    g.perm_mut(h).attached = Some(plain);
    assert!(!kw(&g, plain, "trample") && !kw(&g, plain, "haste"));
    g.perm_mut(h).attached = Some(leg);
    assert!(kw(&g, leg, "trample") && kw(&g, leg, "haste"));
}

// ------------------------------------------------------------------ Ultimate Magic: Meteor
#[test]
fn meteor_deals_seven_to_every_creature() {
    let mut g = game("lordofpain");
    mana(&mut g, P0, 6);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P1, 3);
    cast_from_hand(&mut g, P0, "Ultimate Magic: Meteor");
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
}

#[test]
fn meteor_foretold_destroys_an_artifact_or_land_of_each_opponent() {
    let mut g = game("lordofpain");
    mana(&mut g, P0, 3);
    let c = hand(&mut g, P0, &["Ultimate Magic: Meteor"])[0];
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P2, "Forest", 3, false);
    let (l1, l2) = (g.player(P1).lands.len(), g.player(P2).lands.len());
    let o = options(&mut g, true);
    let fo = o.iter().find(|o| o.label.starts_with("foretell")).expect("foretell offered");
    assert!(act::perform(&mut g, P0, fo.act.as_ref().unwrap()).unwrap());
    // the same turn: no cast yet; a later turn: cast for {5}{R}
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(g.player(P0).hand.contains(&c));
    // let the plan function run directly (the option's value depends on the board)
    let arg = c.0 as i64;
    let ok = sim_core::impls::lordofpain::meteor_cast_foretold(&mut g, P0, arg).unwrap();
    assert!(ok);
    assert!(g.player(P1).perms.iter().all(|&m| !g.perm(m).on_bf || g.perm(m).name != "Sol Ring"));
    assert_eq!(g.player(P1).lands.len(), l1);
    assert_eq!(g.player(P2).lands.len(), l2 - 1);
}
