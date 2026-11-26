//! The order of cost changes (CR 601.2f): the total cost is the mana cost or an alternative cost, plus every increase (the
//! commander tax, Disruptor Flute's {3}, Thalia and Sphere of Resistance), minus every reduction, never below {0}; only then
//! do Trinisphere's minimum, delve and the rest apply. `engine::mana::cost_of_base` sums every increase and reduction of
//! the generic part with no floor between them and floors once. Before, each step floored at 0, so a reduction larger than
//! the generic part so far was lost before a later increase (the tax, the Flute, a hook tax) was added. Oracle text and the
//! rules are the expectation throughout; each test says what the old pipeline gave where it differs.

mod play_common;
use play_common::*;
use sim_core::engine::mana::{can_pay_spell, cost_of, cost_of_base_probed, cost_of_free_with_extra, cost_of_x};
use sim_core::hooks::Opt;
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

// ======================================================== 9. each reducer on its own, then a later increase
/// instants and sorceries of distinct mana values 1..: enough of them in the graveyard for Eris to take `2 * k` off
fn distinct_spells(g: &mut Game, k: usize) {
    let names =
        ["Mystical Tutor", "Counterspell", "Cultivate", "Cryptic Command", "Treasure Cruise", "Blasphemous Act"];
    for n in &names[..k] {
        let c = card(g, n);
        g.player_mut(P0).gy.push(c);
    }
}

#[test]
fn eris_dawning_eminence_and_kaza_are_summed_with_a_later_increase() {
    // each reduction alone drives the running total below zero; a hook tax (Sphere of Resistance, +1) after it must still
    // be taken from the same sum, so the price is 0. A floor after any one of these steps would give {1}.
    // Eris: -2 for each distinct mana value among the instants and sorceries in the graveyard
    let mut g = dragon_table();
    let eris = card(&g, "Eris, Roar of the Storm");
    let generic = g.db.get(eris).generic as usize;
    let k = (generic + 1).div_ceil(2);
    assert!(k <= 6, "{generic}");
    distinct_spells(&mut g, k);
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, eris), (0, sorted(&g.db.get(eris).pips)), "{generic} - {} + 1", 2 * k);
    // Dawning: -1 for each instant and sorcery card in the graveyard
    let mut g = dragon_table();
    let dawning = card(&g, "The Dawning Archaic");
    let bolt = card(&g, "Lightning Bolt");
    let generic = g.db.get(dawning).generic as usize;
    g.player_mut(P0).gy.extend(vec![bolt; generic + 1]);
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, dawning), (0, sorted(&g.db.get(dawning).pips)), "{generic} - {} + 1", generic + 1);
    // the Ur-Dragon's eminence, from the command zone and from a copy on the battlefield: -2 on Scaled Nurturer {1}{G}
    let mut g = dragon_table();
    set_commander(&mut g, P0, UR);
    perm(&mut g, P0, UR);
    let nurturer = hand(&mut g, P0, &["Scaled Nurturer"])[0];
    assert_eq!(sim_core::impls::t45_urdragon::eminence(&g, P0, nurturer), -2);
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, nurturer), (0, "G".to_string()), "1 - 2 + 1");
    // Kaza, Roil Chaser: instants and sorceries cost {1} less (once): Counterspell {U}{U} with Sphere's +1
    let mut g = dragon_table();
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    g.player_mut(P0).t0s.kaza = Some((g.turn_stamp(), 1));
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(price(&g, c), (0, "UU".to_string()), "-1 and +1");
}

#[test]
fn disruptor_flute_on_a_commander_is_summed_with_the_tax_and_the_reducers() {
    // The Ur-Dragon {4}{W}{U}{B}{R}{G}: Flute +3, tax +2 (cast once before), Incubator -2, Shaman -2, Sapphire Medallion -1
    let mut g = dragon_table();
    set_commander(&mut g, P0, UR);
    let ur = card(&g, UR);
    taxed(&mut g, ur, 1);
    let flute = perm(&mut g, P1, "Disruptor Flute");
    g.flutes.clear();
    g.flutes.push((flute, ur));
    let w = sorted("WUBRG");
    assert_eq!(price(&g, ur), (9, w.clone()), "4 + 3 + 2");
    perm(&mut g, P0, "Urza's Incubator");
    perm(&mut g, P0, "Dragonspeaker Shaman");
    perm(&mut g, P0, "Sapphire Medallion");
    assert_eq!(price(&g, ur), (4, w), "4 + 3 + 2 - 5 (old pipeline: the reducers stopped at 4, then 3 + 2 = 5)");
}

#[test]
fn tamiyos_emblem_with_disruptor_flute_pays_the_flutes_three() {
    let mut g = dragon_table();
    g.player_mut(P0).emblems.push("tamiyo");
    let bolt = hand(&mut g, P0, &["Lightning Bolt"])[0];
    let flute = perm(&mut g, P1, "Disruptor Flute");
    g.flutes.clear();
    g.flutes.push((flute, bolt));
    assert_eq!(price(&g, bolt), (3, String::new()), "the emblem's {{0}} plus the Flute's {{3}} (old pipeline: 0)");
}

#[test]
fn delve_comes_after_trinisphere_with_seven_cards_in_the_graveyard() {
    // Treasure Cruise {7}{U}: Trinisphere's minimum of three is met by the printed cost, then delve exiles seven cards: {U}
    // alone. (Delve inside the running total would give {2}{U} from the minimum.)
    let mut g = dragon_table();
    perm(&mut g, P1, "Trinisphere");
    let cruise = card(&g, "Treasure Cruise");
    let plains = card(&g, "Plains");
    g.player_mut(P0).gy.extend(vec![plains; 7]);
    assert_eq!(price(&g, cruise), (0, "U".to_string()));
}

// ======================================================== 10. Twinflame under the emblem, and the probe
fn n_targets(o: &Opt) -> usize {
    o.label.split(|ch: char| !ch.is_ascii_digit()).find(|s| !s.is_empty()).unwrap().parse().unwrap()
}

#[test]
fn twinflame_under_tamiyos_emblem_pays_strive_and_the_taxes() {
    // one target: Twinflame is free, Thalia makes the noncreature spell {1}: cost_of and the AI's own price agree
    let mut g = dragon_table();
    g.player_mut(P0).emblems.push("tamiyo");
    let tf = hand(&mut g, P0, &["Twinflame"])[0];
    perm(&mut g, P1, "Thalia, Guardian of Thraben");
    assert_eq!(cost_of(&g, P0, tf), (1, String::new()));
    assert_eq!(cost_of_free_with_extra(&g, P0, tf, 0, ""), cost_of(&g, P0, tf), "no extra: the same as cost_of");
    // two extra targets: strive {2}{R} each on top; Trinisphere's minimum counts the strive symbols too
    assert_eq!(
        cost_of_free_with_extra(&g, P0, tf, 4, "RR"),
        (5, "RR".to_string()),
        "{{1}} + {{2}}{{2}} and {{R}}{{R}}"
    );
    perm(&mut g, P1, "Trinisphere");
    assert_eq!(cost_of_free_with_extra(&g, P0, tf, 0, ""), (3, String::new()));
    assert_eq!(
        cost_of_free_with_extra(&g, P0, tf, 2, "R"),
        (3, "R".to_string()),
        "min 3 over {{3}}{{R}}: the sum is 4 symbols"
    );
    // the AI's plan pays the same: three Grave Titans' worth of targets, Thalia on the table
    let mut g = dragon_table();
    g.player_mut(P0).emblems.push("tamiyo");
    lands(&mut g, P0, "Mountain", 12, false);
    for _ in 0..3 {
        perm(&mut g, P0, "Grave Titan");
    }
    let tf = hand(&mut g, P0, &["Twinflame"])[0];
    perm(&mut g, P1, "Thalia, Guardian of Thraben");
    let oo = g.registry.get(tf).unwrap().hand_options.unwrap()(&mut g, tf, P0, Some(false)).unwrap();
    let n = n_targets(&oo[0]);
    assert!(n >= 2, "{n}");
    let before = g.player(P0).lands.iter().filter(|&&m| g.perm(m).tapped).count();
    assert!(sim_core::ai::act::perform(&mut g, P0, oo[0].act.as_ref().unwrap()).unwrap());
    let tapped = g.player(P0).lands.iter().filter(|&&m| g.perm(m).tapped).count() - before;
    assert_eq!(tapped, 3 * (n - 1) + 1, "{n} targets: {{2}}{{R}} for each extra and Thalia's {{1}}");
}

#[test]
fn the_shadow_probe_changes_nothing_but_stderr() {
    // Leyline of the Guildpact {G/W}{G/U}{B/G}{R/G} (a card with two-colour hybrid symbols) with a Medallion and a Sphere:
    // the old and new prices differ, and the price and the colours `hybrid::resolve` records are the same with the probe
    // on and off
    let build = || {
        let mut g = dragon_table();
        for (n, c) in [("Forest", 1), ("Island", 1), ("Swamp", 1), ("Mountain", 1)] {
            lands(&mut g, P0, n, c, false);
        }
        perm(&mut g, P0, "Emerald Medallion");
        perm(&mut g, P1, "Sphere of Resistance");
        let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
        (g, c)
    };
    let (g1, c1) = build();
    let (g2, c2) = build();
    let off = cost_of_base_probed(&g1, P0, c1, 0, &g1.db.get(c1).pips, false);
    let on = cost_of_base_probed(&g2, P0, c2, 0, &g2.db.get(c2).pips, true);
    assert_eq!(off, on);
    assert_eq!(sim_core::engine::hybrid::priced(&g1, c1), sim_core::engine::hybrid::priced(&g2, c2));
}
