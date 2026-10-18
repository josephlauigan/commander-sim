//! Tier 6 batch `green2` (impls/t6_green2.rs) on hand-built positions. Seat 0 plays the Rograkh & Thrasios deck,
//! seat 1 the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-thrasios-creature-combo", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn squirrels(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).pow == 1).count()
}

// ======================================================== Chatterstorm
#[test]
fn chatterstorm_makes_a_squirrel_and_a_copy_for_each_earlier_spell() {
    let mut g = game();
    cast_spell(&mut g, P0, "Opt");
    cast_spell(&mut g, P0, "Preordain");
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 3, "the Squirrel and two copies' Squirrels");
    assert!(in_gy(&g, P0, "Chatterstorm"));
}

#[test]
fn chatterstorm_alone_makes_one_squirrel() {
    let mut g = game();
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 1);
}

// ======================================================== Cloud of Faeries
fn untapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

#[test]
fn cloud_of_faeries_untaps_two_lands_when_it_enters() {
    let mut g = game();
    lands(&mut g, P0, "Island", 4, false);
    cast_spell(&mut g, P0, "Cloud of Faeries");
    let cloud = on_bf(&g, P0, "Cloud of Faeries").expect("on the battlefield");
    assert!(g.perm(cloud).fly, "flying");
    // {1}{U} was paid, then two lands untapped: 4 - 2 + 2
    assert_eq!(untapped(&g, P0), 4);
}

#[test]
fn cloud_of_faeries_untaps_only_the_tapped_lands_it_has() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Forest", 1, true);
    perm(&mut g, P0, "Cloud of Faeries"); // not cast: its enters trigger still runs
    assert_eq!(untapped(&g, P0), 3, "the one tapped land untaps");
}

#[test]
fn cloud_of_faeries_cycles_for_two() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Cloud of Faeries"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("cycling option");
    let opts = f(&mut g, c, P0, None).unwrap();
    assert_eq!(opts.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert!(in_gy(&g, P0, "Cloud of Faeries"));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 1, "discarded it and drew a card");
    assert_eq!(untapped(&g, P0), 1, "{{2}} paid");
}

#[test]
fn cloud_of_faeries_cannot_cycle_without_two_mana() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 1, false);
    let c = hand(&mut g, P0, &["Cloud of Faeries"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    assert!(f(&mut g, c, P0, None).unwrap().is_empty());
}
