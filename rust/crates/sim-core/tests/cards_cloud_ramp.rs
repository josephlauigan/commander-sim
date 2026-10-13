//! The cloud deck's ramp and equipment candidates (impls/bulk_white.rs catch-up ramp, impls/cloud.rs helpers, the
//! rocks). Seat 0 plays the cloud deck, seat 1 the seph deck, seat 2 (three-seat tests) the sauron deck.

use sim_core::ai::act;
use sim_core::engine::mana::total_mana;
use sim_core::engine::values::epow;
use sim_core::engine::{cast, hooks, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::cloud;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn game() -> Game {
    table(&["cloud", "seph"])
}

fn count_in_hand(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).hand.iter().filter(|&&c| &*g.db.get(c).name == name).count()
}

fn plains_in_library(g: &Game, p: PlayerId) -> usize {
    g.player(p).library.iter().filter(|&&c| &*g.db.get(c).name == "Plains").count()
}

/// p controls n lands (Swamps for the opponents, so Plains in the library are the deck's own)
fn set_lands(g: &mut Game, p: PlayerId, n: usize) {
    for l in std::mem::take(&mut g.player_mut(p).lands) {
        g.land_mut(l).on_bf = false;
    }
    lands(g, p, if p == P0 { "Plains" } else { "Swamp" }, n, false);
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn bf_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.len()
}

// ------------------------------------------------------------------ Loyal Warhound
#[test]
fn warhound_fetches_a_tapped_plains_only_when_an_opponent_has_more_lands() {
    let mut g = game();
    set_lands(&mut g, P0, 2);
    set_lands(&mut g, P1, 3);
    let lib = plains_in_library(&g, P0);
    assert!(lib > 2);
    perm(&mut g, P0, "Loyal Warhound");
    assert_eq!(bf_lands(&g, P0), 3, "one Plains onto the battlefield");
    assert!(g.land(*g.player(P0).lands.last().unwrap()).tapped, "tapped");
    assert_eq!(plains_in_library(&g, P0), lib - 1);
    // level on lands: nothing
    let mut g = game();
    set_lands(&mut g, P0, 3);
    set_lands(&mut g, P1, 3);
    perm(&mut g, P0, "Loyal Warhound");
    assert_eq!(bf_lands(&g, P0), 3, "an opponent with the same number of lands is not more");
}

#[test]
fn warhound_has_no_regex_ramp_tag() {
    let g = game();
    let d = g.db.get(card(&g, "Loyal Warhound"));
    assert!(!d.tag(sim_core::tag::Tag::Lr) && !d.tag(sim_core::tag::Tag::Lrt));
}

// ------------------------------------------------------------------ Oreskos Explorer
#[test]
fn oreskos_explorer_fetches_one_plains_per_opponent_with_more_lands() {
    let mut g = table(&["cloud", "seph", "sauron"]);
    set_lands(&mut g, P0, 3);
    set_lands(&mut g, P1, 5);
    set_lands(&mut g, P2, 4);
    let before = count_in_hand(&g, P0, "Plains");
    perm(&mut g, P0, "Oreskos Explorer");
    assert_eq!(count_in_hand(&g, P0, "Plains"), before + 2, "two opponents have more lands: X = 2");
    // one of them level: X = 1
    let mut g = table(&["cloud", "seph", "sauron"]);
    set_lands(&mut g, P0, 4);
    set_lands(&mut g, P1, 5);
    set_lands(&mut g, P2, 4);
    perm(&mut g, P0, "Oreskos Explorer");
    assert_eq!(count_in_hand(&g, P0, "Plains"), 1, "X = 1");
    // nobody ahead: X = 0
    let mut g = game();
    set_lands(&mut g, P0, 4);
    set_lands(&mut g, P1, 4);
    perm(&mut g, P0, "Oreskos Explorer");
    assert_eq!(count_in_hand(&g, P0, "Plains"), 0);
}

// ------------------------------------------------------------------ Gift of Estates
#[test]
fn gift_of_estates_fetches_three_plains_when_behind() {
    let mut g = game();
    set_lands(&mut g, P0, 3);
    set_lands(&mut g, P1, 4);
    let c: CardId = hand(&mut g, P0, &["Gift of Estates"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(count_in_hand(&g, P0, "Plains"), 3);
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn gift_of_estates_does_nothing_when_not_behind() {
    let mut g = game();
    set_lands(&mut g, P0, 4);
    set_lands(&mut g, P1, 4);
    let c: CardId = hand(&mut g, P0, &["Gift of Estates"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(count_in_hand(&g, P0, "Plains"), 0);
}

// ------------------------------------------------------------------ Keeper of the Accord
fn end_step(g: &mut Game, p: PlayerId) {
    hooks::fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

#[test]
fn keeper_of_the_accord_makes_a_soldier_at_an_opponent_s_end_step_when_outnumbered() {
    let mut g = game();
    set_lands(&mut g, P0, 3);
    set_lands(&mut g, P1, 3);
    perm(&mut g, P0, "Keeper of the Accord");
    token(&mut g, P1, 2);
    token(&mut g, P1, 2);
    let mine = g.player(P0).perms.iter().filter(|&&m| g.is_creature(m)).count();
    end_step(&mut g, P0); // its own end step: nothing
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.is_creature(m)).count(), mine);
    end_step(&mut g, P1);
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 1, "a 1/1 Soldier");
    assert_eq!(epow(&g, toks[0]), 1);
    assert!(g.perm(toks[0]).ttypes.contains(&"soldier"));
    assert_eq!(bf_lands(&g, P0), 3, "lands are level: no search");
    // 1 creature vs 2 and then 2 vs 2: no more tokens
    end_step(&mut g, P1);
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 1);
}

#[test]
fn keeper_of_the_accord_fetches_a_tapped_plains_when_an_opponent_has_more_lands() {
    let mut g = game();
    set_lands(&mut g, P0, 3);
    set_lands(&mut g, P1, 4);
    perm(&mut g, P0, "Keeper of the Accord");
    end_step(&mut g, P1);
    assert_eq!(bf_lands(&g, P0), 4);
    assert!(g.land(*g.player(P0).lands.last().unwrap()).tapped);
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token), "creatures: 1 against 0 or equal, no Soldier");
    end_step(&mut g, P1); // level now
    assert_eq!(bf_lands(&g, P0), 4);
}

// ------------------------------------------------------------------ Archaeomancer's Map
#[test]
fn archaeomancer_s_map_fetches_two_basic_plains_to_hand() {
    let mut g = game();
    perm(&mut g, P0, "Archaeomancer's Map");
    assert_eq!(count_in_hand(&g, P0, "Plains"), 2);
}

#[test]
fn archaeomancer_s_map_puts_a_land_onto_the_battlefield_when_an_opponent_ahead_plays_one() {
    let mut g = game();
    set_lands(&mut g, P0, 2);
    set_lands(&mut g, P1, 2);
    perm(&mut g, P0, "Archaeomancer's Map");
    let before = count_in_hand(&g, P0, "Plains");
    // an opponent's land enters: now 3 against 2
    lands(&mut g, P1, "Swamp", 1, false);
    zones::landfall(&mut g, P1).unwrap();
    assert_eq!(bf_lands(&g, P0), 3, "a land from the hand onto the battlefield");
    assert_eq!(count_in_hand(&g, P0, "Plains"), before - 1);
    assert!(!g.land(*g.player(P0).lands.last().unwrap()).tapped, "untapped");
    // level now (3 against 3): the next land of an opponent that is not ahead does nothing
    zones::landfall(&mut g, P1).unwrap();
    assert_eq!(bf_lands(&g, P0), 3);
    // my own landfall never triggers it
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(bf_lands(&g, P0), 3);
}

// ------------------------------------------------------------------ the rocks
#[test]
fn thran_dynamo_taps_for_three_colorless() {
    let mut g = game();
    let before = total_mana(&g, P0, false);
    perm(&mut g, P0, "Thran Dynamo");
    assert_eq!(total_mana(&g, P0, false), before + 3);
}

#[test]
fn prismatic_lens_taps_for_one_colorless() {
    let mut g = game();
    let before = total_mana(&g, P0, false);
    perm(&mut g, P0, "Prismatic Lens");
    assert_eq!(total_mana(&g, P0, false), before + 1);
}

// ------------------------------------------------------------------ Kor Outfitter
#[test]
fn kor_outfitter_attaches_a_loose_equipment_to_cloud() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = None;
    perm(&mut g, P0, "Kor Outfitter");
    assert_eq!(g.perm(e).attached, Some(cl));
}

#[test]
fn kor_outfitter_moves_equipment_off_a_worse_wearer_but_leaves_cloud_s() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let t = token(&mut g, P0, 1);
    let worn = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(worn).attached = Some(t);
    let on_cloud = perm(&mut g, P0, "Explorer's Scope");
    g.perm_mut(on_cloud).attached = Some(cl);
    perm(&mut g, P0, "Kor Outfitter");
    assert_eq!(g.perm(worn).attached, Some(cl), "moved from the token");
    assert_eq!(g.perm(on_cloud).attached, Some(cl));
}

// ------------------------------------------------------------------ Auriok Steelshaper
#[test]
fn auriok_steelshaper_makes_equip_cost_one_less_but_not_a_colored_cost() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let scope = perm(&mut g, P0, "Explorer's Scope");
    let pb = perm(&mut g, P0, "Veteran's Powerblade");
    let before = cloud::equip_cost(&g, P0, scope, cl, 3);
    perm(&mut g, P0, "Auriok Steelshaper");
    assert_eq!(cloud::equip_cost(&g, P0, scope, cl, 3), before - 1);
    assert_eq!(cloud::equip_cost(&g, P0, scope, cl, 0), 0, "never below zero");
    assert_eq!(cloud::equip_cost(&g, P0, pb, cl, 2), 1, "Equip Soldier {{W}} has no generic part");
    perm(&mut g, P0, "Auriok Steelshaper");
    assert_eq!(cloud::equip_cost(&g, P0, scope, cl, 3), before - 2, "two Steelshapers");
}

#[test]
fn auriok_steelshaper_pumps_soldiers_and_knights_only_while_equipped() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let st = perm(&mut g, P0, "Auriok Steelshaper");
    let other = token(&mut g, P0, 2);
    let (pc, ps, po) = (epow(&g, cl), epow(&g, st), epow(&g, other));
    let e = perm(&mut g, P0, "Explorer's Scope");
    g.perm_mut(e).attached = Some(st);
    assert_eq!(epow(&g, cl), pc + 1, "Cloud is a Soldier");
    assert_eq!(epow(&g, st), ps + 1, "Steelshaper is a Soldier itself (Scope gives no power)");
    assert_eq!(epow(&g, other), po, "a plain token is neither");
    g.perm_mut(e).attached = None;
    assert_eq!(epow(&g, cl), pc, "unequipped: no bonus");
}

// ------------------------------------------------------------------ Brass Squire
#[test]
fn brass_squire_attaches_an_equipment_to_cloud_without_sorcery_timing() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = None;
    let sq = perm(&mut g, P0, "Brass Squire");
    g.active = Some(P0);
    let os = options(&mut g, sq, P0, Some(false));
    assert_eq!(os.len(), 1);
    assert!(act::perform(&mut g, P0, os[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.perm(e).attached, Some(cl));
    assert!(g.perm(sq).tapped, "{{T}}");
    assert!(options(&mut g, sq, P0, Some(false)).is_empty(), "tapped: no more");
}

#[test]
fn brass_squire_does_nothing_while_summoning_sick_or_with_nothing_to_move() {
    let mut g = game();
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = None;
    let sq = perm(&mut g, P0, "Brass Squire");
    g.active = Some(P0);
    g.perm_mut(sq).sick = true;
    assert!(options(&mut g, sq, P0, Some(false)).is_empty());
    g.perm_mut(sq).sick = false;
    g.perm_mut(e).attached = on_bf(&g, P0, "Cloud, Midgar Mercenary");
    assert!(options(&mut g, sq, P0, Some(false)).is_empty(), "already on Cloud");
}
