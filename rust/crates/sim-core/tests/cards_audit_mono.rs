//! AI audit phase 2, sheoldred and titania: Rooftop Assassin's damage condition, Seizan, Dark Deal, Necrosquito's
//! priority, Fblthp's {X}{G}{G} and Silhana Starfletcher's green mana.

use sim_core::engine::{cast, mana};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::sheoldred::sheoldred_prio;
use sim_core::state::{Ctx, Game};
use sim_core::tag::Tag;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn cast_hand(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

#[test]
fn rooftop_assassin_needs_a_damaged_creature() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    lands(&mut g, P0, "Swamp", 4, false);
    cast_hand(&mut g, P0, "Rooftop Assassin");
    assert!(g.perm(victim).on_bf, "an undamaged creature survives the Assassin");
    assert!(on_bf(&g, P0, "Rooftop Assassin").is_some());
}

#[test]
fn rooftop_assassin_kills_a_creature_dealt_damage_this_turn() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    let other = perm(&mut g, P1, "Llanowar Elves");
    g.mark_damaged(victim);
    lands(&mut g, P0, "Swamp", 4, false);
    cast_hand(&mut g, P0, "Rooftop Assassin");
    assert!(!g.perm(victim).on_bf, "the damaged creature is destroyed");
    assert!(g.perm(other).on_bf, "the undamaged one is not");
}

#[test]
fn rooftop_assassin_damage_mark_expires_with_the_turn() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    g.mark_damaged(victim);
    assert!(g.was_damaged_now(victim));
    g.round += 1;
    assert!(!g.was_damaged_now(victim));
}

#[test]
fn rooftop_assassin_is_cast_for_a_damaged_target() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Rooftop Assassin"])[0];
    let victim = perm(&mut g, P1, "Atraxa, Grand Unifier");
    let idle = sheoldred_prio(&g, P0, c);
    g.mark_damaged(victim);
    assert!(sheoldred_prio(&g, P0, c) > idle + 20);
}

#[test]
fn seizan_comes_down_for_underworld_dreams_without_sheoldred() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Seizan, Perverter of Truth"])[0];
    assert!(sheoldred_prio(&g, P0, c) < 60);
    perm(&mut g, P0, "Underworld Dreams");
    assert_eq!(sheoldred_prio(&g, P0, c), 80);
}

#[test]
fn necrosquito_is_a_better_cast_with_sacrifice_fodder() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Necrosquito"])[0];
    let base = sheoldred_prio(&g, P0, c);
    assert!(base >= 55, "not last among the four-drops, got {base}");
    perm(&mut g, P0, "Carnage Altar");
    token(&mut g, P0, 1);
    assert!(sheoldred_prio(&g, P0, c) > base + 5);
}

#[test]
fn silhana_starfletcher_taps_for_green() {
    let g = table(&["titania", "seph"]);
    let d = g.db.get(card(&g, "Silhana Starfletcher"));
    assert_eq!(d.tags.str(Tag::Dork), Some("G"));
}

#[test]
fn fblthp_costs_x_gg_and_fetches_one_forest() {
    let mut g = table(&["titania", "seph"]);
    let f = card(&g, "Fblthp, Knows the Way");
    assert_eq!(g.db.get(f).cmc, 3, "{{1}}{{G}}{{G}}: X = 1");
    lands(&mut g, P0, "Forest", 2, false);
    let c = hand(&mut g, P0, &["Fblthp, Knows the Way"])[0];
    assert!(!mana::can_pay(&g, P0, 1, "GG", false), "two lands can't pay X=1");
    let _ = c;
    lands(&mut g, P0, "Forest", 1, false);
    let before = g.player(P0).hand.iter().filter(|&&x| &*g.db.get(x).name == "Forest").count();
    cast_hand_existing(&mut g, c);
    let after = g.player(P0).hand.iter().filter(|&&x| &*g.db.get(x).name == "Forest").count();
    assert_eq!(after, before + 1);
}

fn cast_hand_existing(g: &mut Game, c: CardId) {
    cast::cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}
