//! The order of cost changes (CR 601.2f): the total cost is the mana cost or an alternative cost, plus every increase (the
//! commander tax, Disruptor Flute's {3}, Thalia and Sphere of Resistance), minus every reduction, never below {0}; only then
//! do Trinisphere's minimum, delve and the rest apply. `engine::mana::cost_of_base` sums every increase and reduction of
//! the generic part with no floor between them and floors once. Before, each step floored at 0, so a reduction larger than
//! the generic part so far was lost before a later increase (the tax, the Flute, a hook tax) was added. Oracle text and the
//! rules are the expectation throughout; each test says what the old pipeline gave where it differs.

mod play_common;
use play_common::*;
use sim_core::engine::mana::{can_pay_spell, cost_of, cost_of_x};
use sim_core::state::{DataKey, Val};
use sim_core::testkit::{set_commander, table};

const KARN: &str = "Karn, Gilded Guardian";
const UR: &str = "The Ur-Dragon";

fn karn_table() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn dragon_table() -> Game {
    let mut g = table(&["miirym-temur-dragons", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn sorted(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v.sort();
    v.into_iter().collect()
}

fn price(g: &Game, c: sim_core::ids::CardId) -> (u32, String) {
    let (gn, pips) = cost_of(g, P0, c);
    (gn, sorted(&pips))
}

fn sources(g: &mut Game, spec: &[(&str, usize)]) {
    for &(name, n) in spec {
        lands(g, P0, name, n, false);
    }
}

/// the commander has been cast `casts` times already: tax 2 each
fn taxed(g: &mut Game, c: sim_core::ids::CardId, casts: u32) {
    for _ in 0..casts {
        g.player_mut(P0).add_tax(c);
    }
}

// ======================================================== 1. Karn's recast
#[test]
fn karn_recast_with_five_colours_and_the_crystal_takes_the_one_off_the_tax() {
    // {2/W}{2/U}{2/B}{2/R}{2/G} paid with the five colours has no generic part; the commander tax {2} is added by the
    // rules before reductions, so The Fire Crystal ("red spells cost {1} less"; Karn is red) takes {1} of it: {1}+WUBRG.
    let mut g = karn_table();
    set_commander(&mut g, P0, KARN);
    let c = card(&g, KARN);
    sources(&mut g, &[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Mountain", 1), ("Forest", 1)]);
    taxed(&mut g, c, 1);
    assert_eq!(g.player(P0).tax_of(c), 2);
    assert_eq!(price(&g, c), (2, sorted("WUBRG")), "no reducer: the tax is {{2}}");
    perm(&mut g, P0, "The Fire Crystal");
    assert_eq!(price(&g, c), (1, sorted("WUBRG")), "the Crystal's {{1}} comes off the tax (old pipeline: still 2)");
    sources(&mut g, &[("Plains", 1)]);
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "six lands pay {{1}}+WUBRG");
    // a second recast: tax 4, the Crystal: 3
    taxed(&mut g, c, 1);
    assert_eq!(price(&g, c), (3, sorted("WUBRG")));
    // no recast: nothing to reduce, unchanged
    g.player_mut(P0).tax = 0;
    assert_eq!(price(&g, c), (0, sorted("WUBRG")), "tax 0: the reduction finds no generic mana");
}

// ======================================================== 2. reducers over the generic part take from the tax
#[test]
fn reducers_larger_than_the_generic_part_take_from_the_tax_and_stop_at_zero() {
    let mut g = dragon_table();
    set_commander(&mut g, P0, UR);
    let ur = card(&g, UR);
    let inc = perm(&mut g, P0, "Urza's Incubator");
    assert_eq!(g.perm(inc).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
    perm(&mut g, P0, "Dragonspeaker Shaman");
    perm(&mut g, P0, "Sapphire Medallion");
    let w = sorted("WUBRG");
    // 4 - 2 - 2 - 1 = -1 -> 0
    assert_eq!(price(&g, ur), (0, w.clone()), "no tax: the surplus {{1}} of reduction is lost");
    // tax 2: 4 + 2 - 5 = 1 (old pipeline: 0 + 2 = 2)
    taxed(&mut g, ur, 1);
    assert_eq!(price(&g, ur), (1, w.clone()), "tax 2: the reductions take {{5}} of {{6}}");
    // tax 4: 4 + 4 - 5 = 3 (old: 4)
    taxed(&mut g, ur, 1);
    assert_eq!(price(&g, ur), (3, w.clone()), "tax 4");
    // tax 6: 4 + 6 - 5 = 5 (old: 6)
    taxed(&mut g, ur, 1);
    assert_eq!(price(&g, ur), (5, w), "tax 6");
}

// ======================================================== 3. Disruptor Flute before a reduction
#[test]
fn disruptor_flute_is_an_increase_the_medallion_takes_from() {
    // Counterspell {U}{U} under Disruptor Flute naming it costs {3} more; Sapphire Medallion takes {1} of that: {2}{U}{U}
    // (old pipeline: the Medallion's {1} was lost on the empty generic part, so {3}{U}{U})
    let mut g = dragon_table();
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    let flute = perm(&mut g, P1, "Disruptor Flute");
    g.flutes.clear();
    g.flutes.push((flute, c));
    assert_eq!(price(&g, c), (3, "UU".to_string()), "the Flute alone");
    perm(&mut g, P0, "Sapphire Medallion");
    assert_eq!(price(&g, c), (2, "UU".to_string()));
}

// ======================================================== 4. across steps
#[test]
fn a_reduction_and_a_later_hook_tax_are_summed_before_the_floor() {
    // Counterspell {U}{U}: Sapphire Medallion (-1, compiled) and Sphere of Resistance (+1, a card-code hook) come to
    // nothing. Old pipeline: the Medallion's clamp came first, then the Sphere: {1}{U}{U}.
    let mut g = dragon_table();
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    perm(&mut g, P0, "Sapphire Medallion");
    assert_eq!(price(&g, c), (0, "UU".to_string()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, c), (0, "UU".to_string()), "-1 and +1");
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, c), (1, "UU".to_string()), "-1 and +2 (the sum is positive)");
}

#[test]
fn a_per_creature_reduction_and_a_later_hook_tax_are_summed_before_the_floor() {
    // Blasphemous Act {8}{R} costs {1} less for each creature on the battlefield: with ten creatures, 8 - 10 + 1 (Sphere of
    // Resistance) is below zero. Old pipeline: its own clamp came first, then the Sphere: {1}{R}.
    let mut g = dragon_table();
    let act = hand(&mut g, P0, &["Blasphemous Act"])[0];
    for _ in 0..10 {
        token(&mut g, P1, 1);
    }
    assert_eq!(price(&g, act), (0, "R".to_string()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, act), (0, "R".to_string()), "8 - 10 + 1 is below zero");
    for _ in 0..3 {
        token(&mut g, P1, 1);
    }
    perm(&mut g, P1, "Sphere of Resistance");
    perm(&mut g, P1, "Sphere of Resistance");
    // 13 creatures, three Spheres: 8 - 13 + 3 = -2
    assert_eq!(price(&g, act), (0, "R".to_string()));
}

// ======================================================== 5. Trinisphere is still last
#[test]
fn trinisphere_minimum_comes_after_every_reduction() {
    let mut g = dragon_table();
    perm(&mut g, P1, "Trinisphere");
    let sol = card(&g, "Sol Ring");
    assert_eq!(price(&g, sol), (3, String::new()), "{{1}} spell: at least three");
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    assert_eq!(price(&g, c), (1, "UU".to_string()), "{{U}}{{U}} plus {{1}} makes three symbols");
    perm(&mut g, P0, "Sapphire Medallion");
    assert_eq!(price(&g, c), (1, "UU".to_string()), "a reducer does not go below the minimum");
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P0, "Ruby Medallion");
    assert_eq!(price(&g, sol), (3, String::new()), "still three with or without reducers");
}

// ======================================================== 6. Dracogenesis
#[test]
fn dracogenesis_is_an_alternative_cost_of_zero_that_the_tax_and_reducers_adjust() {
    // The Ur-Dragon from the command zone with Dracogenesis: no mana cost, but the tax {2} is added; a Dragon reducer
    // (Dragonlord's Servant, {1} less) takes from it. Old pipeline: the reducer was cleared with the rest, so 2.
    let mut g = dragon_table();
    set_commander(&mut g, P0, UR);
    let ur = card(&g, UR);
    perm(&mut g, P0, "Dracogenesis");
    taxed(&mut g, ur, 1);
    assert_eq!(price(&g, ur), (2, String::new()), "tax only");
    perm(&mut g, P0, "Dragonlord's Servant");
    assert_eq!(price(&g, ur), (1, String::new()), "the reducer takes {{1}} of the tax (old pipeline: 2)");
    // an increase applies to the alternative cost too (601.2f): Sphere of Resistance on a free Dragon
    let mut g = dragon_table();
    perm(&mut g, P0, "Dracogenesis");
    let gold = card(&g, "Goldspan Dragon");
    assert_eq!(price(&g, gold), (0, String::new()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, gold), (1, String::new()), "Sphere's {{1}} is paid (old pipeline: lost, 0)");
}

// ======================================================== 7. Tamiyo's emblem
#[test]
fn tamiyos_emblem_is_an_alternative_cost_that_taxes_and_trinisphere_still_change() {
    let mut g = dragon_table();
    g.player_mut(P0).emblems.push("tamiyo");
    let bolt = hand(&mut g, P0, &["Lightning Bolt"])[0];
    assert_eq!(price(&g, bolt), (0, String::new()), "from hand without paying its mana cost");
    let t = perm(&mut g, P1, "Thalia, Guardian of Thraben");
    assert_eq!(price(&g, bolt), (1, String::new()), "Thalia: noncreature spells cost {{1}} more (old pipeline: 0)");
    g.perm_mut(t).on_bf = false;
    perm(&mut g, P1, "Trinisphere");
    assert_eq!(price(&g, bolt), (3, String::new()), "Trinisphere: a spell costs at least {{3}} (old pipeline: 0)");
    // not cast from hand (the command zone, say): the emblem does not apply
    g.player_mut(P0).hand.retain(|&x| x != bolt);
    assert_eq!(price(&g, bolt), (2, "R".to_string()), "printed {{R}} under Trinisphere: three symbols in all");
}

// ======================================================== 8. unchanged values pinned
#[test]
fn x_spells_spectacle_and_delve_keep_their_values() {
    let mut g = dragon_table();
    // X under Jet Medallion: Damnable Pact {X}{B}{B} with X = 3 is {3}{B}{B}, {1} less
    perm(&mut g, P0, "Jet Medallion");
    let pact = card(&g, "Damnable Pact");
    assert_eq!(cost_of_x(&g, P0, pact, 3), (2, "BB".to_string()));
    // spectacle: Light Up the Stage costs {R} after an opponent lost life this turn; a Sphere adds {1}
    let stage = card(&g, "Light Up the Stage");
    g.player_mut(P0).hit_turn = g.player(P0).turns as i32;
    assert_eq!(cost_of_x(&g, P0, stage, 0), (0, "R".to_string()));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(cost_of_x(&g, P0, stage, 0), (1, "R".to_string()));
    // delve comes after the floor: Treasure Cruise {7}{U}, three cards to exile, Sapphire Medallion: {7} - 1 - 3 = 3 (+Sphere)
    let mut g = dragon_table();
    let cruise = card(&g, "Treasure Cruise");
    assert_eq!(cost_of_x(&g, P0, cruise, 0), (7, "U".to_string()));
    let plains = card(&g, "Plains");
    g.player_mut(P0).gy.extend([plains, plains, plains]);
    assert_eq!(cost_of_x(&g, P0, cruise, 0), (4, "U".to_string()), "delve takes {{3}} of the generic mana");
    perm(&mut g, P0, "Sapphire Medallion");
    assert_eq!(cost_of_x(&g, P0, cruise, 0), (3, "U".to_string()));
}
