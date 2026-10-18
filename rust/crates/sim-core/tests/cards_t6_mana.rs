//! Tier 6 "mana" batch (impls/t6_mana.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["rograkh-silas-turbo-breach", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.player(p).perms.contains(&m)
}

// ======================================================== Chromatic Star
#[test]
fn chromatic_star_is_not_a_free_rock_and_cracks_for_a_card() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let s = perm(&mut g, P0, "Chromatic Star");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "Star adds no mana by itself");
    assert!(options(&mut g, s, P0, Some(false)).is_empty(), "not before combat");
    let o = options(&mut g, s, P0, Some(true));
    assert_eq!(o.len(), 1);
    let before = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert!(!on(&g, P0, s), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Chromatic Star")));
    assert_eq!(g.player(P0).hand.len(), before + 1, "drew a card");
    assert_eq!(g.player(P0).floating.any, 1, "one mana of any colour");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2 - 1 + 1, "paid {{1}}, got one mana back");
}

#[test]
fn chromatic_star_destroyed_still_draws() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Chromatic Star");
    let before = g.player(P0).hand.len();
    sim_core::engine::zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1);
    let _ = P1;
}

// ======================================================== Desperate Ritual
#[test]
fn desperate_ritual_adds_rrr_for_two_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let c = hand(&mut g, P0, &["Desperate Ritual"])[0];
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "R", false).unwrap(), "{{1}}{{R}}");
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap());
    assert_eq!(g.player(P0).floating.r, 3, "RRR");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 3, "both Mountains were tapped for the cost");
    assert!(g.player(P0).gy.contains(&c), "an instant goes to the graveyard");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "RRR", false));
}
