//! `mana::cost_of_x`: the cost of a spell cast with X = x, through the same reductions, taxes, floors and minimums as
//! `cost_of` (a generic reducer takes its {1} off the generic part that includes X; every increase and reduction is summed
//! first and the generic cost is floored at 0 once, CR 601.2f). Damnable Pact ({X}{B}{B}) is the X spell; Jet Medallion is the reducer; Sphere of
//! Resistance (all spells cost {1} more) and Trinisphere (a spell costs at least three) are the taxes.
//! Damping Sphere is not used: it taxes by the spells already cast this turn, which `cost_of_x` does not change.

use sim_core::engine::mana::{cost_of, cost_of_x};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const PACT: &str = "Damnable Pact";

fn game() -> Game {
    table(&["sheoldred", "seph", "cloud"])
}

fn x_cost(g: &Game, x: i32) -> (u32, String) {
    cost_of_x(g, P0, card(g, PACT), x)
}

#[test]
fn plain_pact_costs_x_plus_two_black() {
    let g = game();
    assert_eq!(x_cost(&g, 0), (0, "BB".into()));
    assert_eq!(x_cost(&g, 1), (1, "BB".into()));
    assert_eq!(x_cost(&g, 5), (5, "BB".into()));
}

#[test]
fn jet_medallion_alone_takes_one_off_the_generic_part_that_includes_x() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    assert_eq!(x_cost(&g, 0), (0, "BB".into()), "nothing to take off at X = 0");
    assert_eq!(x_cost(&g, 1), (0, "BB".into()));
    assert_eq!(x_cost(&g, 3), (2, "BB".into()));
}

#[test]
fn a_tax_alone_is_added_to_x() {
    let mut g = game();
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(x_cost(&g, 0), (1, "BB".into()));
    assert_eq!(x_cost(&g, 3), (4, "BB".into()));
}

#[test]
fn jet_medallion_then_a_later_tax() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(x_cost(&g, 0), (0, "BB".into()), "-1 and +1 are summed before the floor (601.2f)");
    assert_eq!(x_cost(&g, 3), (3, "BB".into()), "3 - 1 + 1");
}

#[test]
fn a_reduction_larger_than_the_generic_part_is_summed_with_the_tax_and_floors_once() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P0, "Jet Medallion");
    assert_eq!(x_cost(&g, 1), (0, "BB".into()), "1 - 2 floors at 0");
    assert_eq!(x_cost(&g, 4), (2, "BB".into()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(x_cost(&g, 1), (0, "BB".into()), "1 - 2 + 1 = 0: the tax is taken from by the reduction (601.2f)");
    assert_eq!(x_cost(&g, 4), (3, "BB".into()));
}

#[test]
fn a_minimum_cost_applies_after_the_reduction() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Trinisphere");
    // at least three mana in all: {B}{B} plus one generic
    assert_eq!(x_cost(&g, 0), (1, "BB".into()));
    assert_eq!(x_cost(&g, 1), (1, "BB".into()));
    assert_eq!(x_cost(&g, 4), (3, "BB".into()), "4 - 1 = 3 is above the floor");
}

#[test]
fn x_zero_equals_cost_of_for_every_spell_with_no_x() {
    let mut g = game();
    for name in ["Sol Ring", "Lightning Bolt", "Terror", PACT, "Painful Lesson", "Sheoldred, the Apocalypse"] {
        let c = card(&g, name);
        assert_eq!(cost_of_x(&g, P0, c, 0), cost_of(&g, P0, c), "{name} bare");
    }
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    for name in ["Sol Ring", "Lightning Bolt", "Terror", PACT, "Painful Lesson", "Sheoldred, the Apocalypse"] {
        let c = card(&g, name);
        assert_eq!(cost_of_x(&g, P0, c, 0), cost_of(&g, P0, c), "{name} with a reducer and a tax");
    }
}

#[test]
fn cost_of_prices_a_card_whose_x_the_ai_picks_at_that_x_and_cost_of_x_never_does() {
    // Hangarback Walker and Chalice of the Void ({X}{X}): `cost_of` is the price at the X the AI casts for
    // (`CardImpl::auto_x`), `cost_of_x` the price at the X it is given (so x = 0 stays "X = 0", and nothing recurses)
    for name in ["Hangarback Walker", "Chalice of the Void"] {
        let mut g = game();
        let c = card(&g, name);
        assert_eq!(sim_core::cardcode::auto_x(&g, P0, c), Some(None), "{name}: no lands, it can pay for no X");
        assert_eq!(cost_of_x(&g, P0, c, 0), (0, String::new()), "{name}: X = 0 is free");
        assert_eq!(cost_of(&g, P0, c), (99, String::new()), "{name}: so it is not cast: priced out of reach");
        lands(&mut g, P0, "Wastes", 6, false);
        let x = sim_core::cardcode::auto_x(&g, P0, c).unwrap().expect("six lands pay for an X");
        assert_eq!(cost_of(&g, P0, c), cost_of_x(&g, P0, c, 2 * x), "{name}");
    }
}

#[test]
fn a_spell_with_no_x_still_prices_as_before() {
    let mut g = game();
    let lesson = card(&g, "Painful Lesson"); // {2}{B}
    assert_eq!(cost_of(&g, P0, lesson), (2, "B".into()));
    perm(&mut g, P0, "Jet Medallion");
    assert_eq!(cost_of(&g, P0, lesson), (1, "B".into()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(cost_of(&g, P0, lesson), (2, "B".into()));
}

fn cast_pact_plan(g: &mut Game, x: i32) -> bool {
    let c = hand(g, P0, &[PACT])[0];
    let opts = g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1, "the AI offers the Pact");
    let Some(sim_core::hooks::Action::Plan { f, arg }) = opts[0].act else { panic!("expected a plan") };
    assert_eq!(arg >> 32, x as i64, "the plan carries the X the mana allows");
    f(g, P0, arg).unwrap()
}

#[test]
fn pact_spends_exactly_the_x_cost_with_a_reducer_and_a_later_tax() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    let ls = lands(&mut g, P0, "Swamp", 5, false);
    // X = 3 costs {3}{B}{B} - 1 + 1 = five mana
    assert!(cast_pact_plan(&mut g, 3));
    assert_eq!(ls.iter().filter(|&&m| g.perm(m).tapped).count(), 5);
    assert_eq!(g.player(P0).hand.len(), 3, "drew three");
}

#[test]
fn pact_with_one_less_land_gets_one_less_x_under_the_same_reducer_and_tax() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    let ls = lands(&mut g, P0, "Swamp", 4, false);
    assert!(cast_pact_plan(&mut g, 2));
    assert_eq!(ls.iter().filter(|&&m| g.perm(m).tapped).count(), 4);
}

#[test]
fn krrik_pays_life_only_for_a_pip_the_reduced_cost_cannot_pay_in_mana() {
    // Gray Merchant of Asphodel {3}{B}{B} with four Swamps: K'rrik drops one {B} for 2 life (3 generic + {B})
    let mut g = game();
    perm(&mut g, P0, "K'rrik, Son of Yawgmoth");
    lands(&mut g, P0, "Swamp", 4, false);
    let merchant = card(&g, "Gray Merchant of Asphodel");
    assert_eq!(cost_of(&g, P0, merchant), (3, "B".into()), "no reducer: one {{B}} is paid with life");
    // with Jet Medallion the cost is {2}{B}{B}, four mana: payable, so no life is paid (the reducer used to be
    // looked at only after K'rrik had already dropped the pip)
    perm(&mut g, P0, "Jet Medallion");
    assert_eq!(cost_of(&g, P0, merchant), (2, "BB".into()), "the reduced cost is payable in mana: no life");
}

#[test]
fn trinisphere_counts_a_pip_that_is_paid_with_life_and_delve_comes_after_it() {
    // Damnable Pact X = 1 {1}{B}{B} under Trinisphere with K'rrik out and only two Swamps: the minimum of three counts both
    // {B} symbols, so the generic part is 1; K'rrik then pays one {B} with 2 life: {1}{B} and 2 life (two mana, which the
    // Swamps pay), not {2}{B}
    let mut g = game();
    perm(&mut g, P0, "K'rrik, Son of Yawgmoth");
    perm(&mut g, P1, "Trinisphere");
    lands(&mut g, P0, "Swamp", 2, false);
    assert_eq!(x_cost(&g, 1), (1, "B".into()), "{{1}}{{B}} plus 2 life, not {{2}}{{B}}");
    // Dig Through Time {6}{U}{U} with six cards in the graveyard: delve pays the whole generic part after the minimum, so
    // the cost is {U}{U}, not raised to three
    let mut g = game();
    perm(&mut g, P1, "Trinisphere");
    for _ in 0..6 {
        let c = take(&mut g, P0, "Swamp");
        g.player_mut(P0).gy.push(c);
    }
    let dig = card(&g, "Dig Through Time");
    assert_eq!(cost_of(&g, P0, dig), (0, "UU".into()), "delve is not a reduction: the minimum does not apply to it");
}
