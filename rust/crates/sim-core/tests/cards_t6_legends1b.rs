//! Tier 6 batch `legends1b` (impls/t6_legends1b.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t6_legends1b::ability_haste;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["sisay-legends-toolbox", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

// ======================================================== Ioreth of the Healing House
#[test]
fn ioreth_untaps_a_tapped_land() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Island", 2, true);
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    let o = options(&mut g, i, P0, Some(true));
    assert_eq!(o.len(), 1, "one tapped land to untap");
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(i).tapped, "{{T}} is the cost");
    assert_eq!(ls.iter().filter(|&&l| !g.perm(l).tapped).count(), 1, "exactly one land untaps");
    assert!(options(&mut g, i, P0, Some(true)).is_empty(), "Ioreth is tapped");
}

#[test]
fn ioreth_has_nothing_to_untap() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    assert!(options(&mut g, i, P0, Some(true)).is_empty(), "nothing tapped: the AI does not tap");
}

#[test]
fn ioreth_untaps_two_other_legends_not_itself() {
    let mut g = tbl();
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    let s = perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    let m = perm(&mut g, P0, "Marvin, Murderous Mimic");
    g.perm_mut(s).tapped = true;
    g.perm_mut(m).tapped = true;
    let o = options(&mut g, i, P0, Some(true));
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(s).tapped && !g.perm(m).tapped, "both legendary creatures untap");
    assert!(g.perm(i).tapped);
}

#[test]
fn ioreth_is_not_summoning_sick_proof() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 1, true);
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    g.perm_mut(i).sick = true;
    assert!(options(&mut g, i, P0, Some(true)).is_empty(), "a {{T}} ability of a summoning-sick creature");
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    assert!(ability_haste(&g, P0));
    assert_eq!(options(&mut g, i, P0, Some(true)).len(), 1, "with Shang-Chi the sick Ioreth may tap");
}

// ======================================================== Marvin, Murderous Mimic
#[test]
fn marvin_has_ioreths_ability_only_with_an_ioreth() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 1, true);
    let m = perm(&mut g, P0, "Marvin, Murderous Mimic");
    assert!(options(&mut g, m, P0, Some(true)).is_empty(), "no creature to take an ability from");
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    g.perm_mut(i).tapped = true;
    let o = options(&mut g, m, P0, Some(true));
    assert!(!o.is_empty());
    // untaps Ioreth (another permanent) and Ioreth, in turn, can untap Marvin: the DDR loop
    let o = o.iter().max_by(|a, b| a.utility.total_cmp(&b.utility)).unwrap().clone();
    assert!(run(&mut g, P0, &o));
    assert!(g.perm(m).tapped);
}

#[test]
fn marvin_and_ioreth_loop_with_a_legend_that_taps_for_mana() {
    let mut g = tbl();
    let i = perm(&mut g, P0, "Ioreth of the Healing House");
    let m = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let s = perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    g.perm_mut(s).tapped = true; // tapped for mana
    let o = options(&mut g, i, P0, Some(true));
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(s).tapped, "Ioreth untapped the legend");
    g.perm_mut(s).tapped = true;
    let o = options(&mut g, m, P0, Some(true));
    assert!(!o.is_empty(), "Marvin repeats Ioreth's ability in the same turn");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(s).tapped && !g.perm(i).tapped, "legend and Ioreth untapped by Marvin");
}

// ======================================================== Shang-Chi, Master of Kung Fu
#[test]
fn shang_chi_gives_abilities_haste() {
    let mut g = tbl();
    assert!(!ability_haste(&g, P0));
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    assert!(ability_haste(&g, P0));
    assert!(!ability_haste(&g, P1), "only its controller");
}

// ======================================================== Tyvar, Jubilant Brawler
#[test]
fn tyvar_plus_untaps_a_creature() {
    let mut g = tbl();
    let ty = perm(&mut g, P0, "Tyvar, Jubilant Brawler");
    assert!(ability_haste(&g, P0), "Tyvar gives abilities haste too");
    let s = perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    g.perm_mut(s).tapped = true;
    let o = options(&mut g, ty, P0, Some(true));
    let plus = o.iter().find(|x| x.label.contains("+1")).expect("+1 offered");
    assert!(run(&mut g, P0, plus));
    assert!(!g.perm(s).tapped);
    assert_eq!(g.perm(ty).loyalty, Some(4));
}

#[test]
fn tyvar_minus_mills_three_and_returns_a_cheap_creature() {
    let mut g = tbl();
    let ty = perm(&mut g, P0, "Tyvar, Jubilant Brawler");
    let c = card(&g, "Shang-Chi, Master of Kung Fu");
    g.player_mut(P0).gy.push(c);
    let before = g.player(P0).library.len();
    let o = options(&mut g, ty, P0, Some(true));
    let minus = o.iter().find(|x| x.label.contains("-2")).expect("-2 offered");
    assert!(run(&mut g, P0, minus));
    assert_eq!(g.player(P0).library.len(), before - 3, "three cards milled");
    assert!(g.player(P0).perms.iter().any(|&m| on(&g, m) && &*g.perm(m).name == "Shang-Chi, Master of Kung Fu"));
    assert!(!g.player(P0).gy.contains(&c));
    assert_eq!(g.perm(ty).loyalty, Some(1));
}

// ======================================================== Loki's Scepter
#[test]
fn loki_steals_the_best_creature_until_end_of_turn() {
    let mut g = tbl();
    let t = perm(&mut g, P1, "Craterhoof Behemoth");
    assert!(sim_core::engine::values::pval(&g, t) >= 3.0);
    g.perm_mut(t).tapped = true;
    perm(&mut g, P0, "Loki's Scepter");
    assert_eq!(g.perm(t).owner, P0, "control changes");
    assert!(!g.perm(t).tapped, "untapped");
    assert!(!g.perm(t).sick, "haste");
    assert!(g.player(P0).borrowed.contains(&t), "returns at end of turn");
}

#[test]
fn loki_leaves_a_weak_creature_alone() {
    let mut g = tbl();
    let t = token(&mut g, P1, 1);
    perm(&mut g, P0, "Loki's Scepter");
    assert_eq!(g.perm(t).owner, P1);
}
