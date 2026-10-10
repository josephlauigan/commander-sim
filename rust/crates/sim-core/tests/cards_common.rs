//! common.py's card code (`impls/common.rs`): Auras (attaching, bonuses, falling off, umbra armor), the Oblivion
//! Ring family, the aristocrat sacrifice options, Rest in Peace and the shared death-trigger creatures. The Python
//! suite has no tests of these cards; each expectation comes from the card's Oracle text as the Python models it.

use sim_core::cardcode;
use sim_core::dsl;
use sim_core::engine::{cast, hooks, mana, removal, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::common;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const LP: &str = "light-paws-aura-voltron";
const TEYSA: &str = "teysa-orzhov-aristocrats";
const ISSHIN: &str = "isshin-mardu-attack-triggers";

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a permanent's own options hook (Python: CI.HOOKS[name]['options'](g, src, p, s, post))
fn opts(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, Some(true)).unwrap()
}

fn combat_damage(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) {
    hooks::fire_trigger(g, Event::CombatDamage, Call::CombatDamage { p, a, d, dmg }).unwrap();
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Auras
#[test]
fn an_aura_enchants_your_creature_and_gives_its_bonus() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel"); // 1/1
    let (p0, t0) = (values::epow(&g, host), values::etgh(&g, host));
    let a = perm(&mut g, P0, "Angelic Destiny");
    assert_eq!(g.perm(a).attached, Some(host));
    assert_eq!(common::auras_on(&g, host), vec![a]);
    assert_eq!((values::epow(&g, host), values::etgh(&g, host)), (p0 + 4, t0 + 4));
    assert!(dsl::has_kw(&g, host, "flying"));
    assert!(dsl::has_kw(&g, host, "first strike"));
    assert!(!dsl::has_kw(&g, host, "lifelink"));
    assert_eq!(cardcode::own_auras_on(&g, host), 1);
}

#[test]
fn with_no_creature_the_aura_goes_to_the_graveyard() {
    let mut g = table(&[LP, TEYSA]);
    let a = perm(&mut g, P0, "Battle Mastery");
    assert!(!g.perm(a).on_bf);
    assert!(in_gy(&g, P0, "Battle Mastery"));
    assert!(g.auras.is_empty());
}

#[test]
fn light_paws_puts_auras_on_the_commander() {
    let mut g = table(&[LP, TEYSA]);
    let cmd = perm(&mut g, P0, "Light-Paws, Emperor's Voice");
    g.perm_mut(cmd).is_cmd = true;
    perm(&mut g, P0, "Hero of Bladehold"); // a bigger creature
    let a = perm(&mut g, P0, "Hyena Umbra");
    assert_eq!(g.perm(a).attached, Some(cmd));
}

#[test]
fn daybreak_coronet_needs_an_enchanted_creature() {
    let mut g = table(&[ISSHIN, TEYSA]); // no commander preference: the best creature
    let bare = perm(&mut g, P0, "Hero of Bladehold");
    let small = perm(&mut g, P0, "Esper Sentinel");
    let w = perm(&mut g, P0, "Spirit Mantle");
    assert_eq!(g.perm(w).attached, Some(bare));
    g.perm_mut(w).attached = Some(small); // moved for the test: only the Sentinel is enchanted
    let c = perm(&mut g, P0, "Daybreak Coronet");
    assert_eq!(g.perm(c).attached, Some(small));
}

#[test]
fn the_aura_falls_off_when_its_creature_leaves() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    let a = perm(&mut g, P0, "Battle Mastery");
    removal::apply_removal(&mut g, Some(P1), host, "exile", None).unwrap();
    assert!(!g.perm(a).on_bf);
    assert!(in_gy(&g, P0, "Battle Mastery"));
    assert!(g.auras.is_empty());
}

#[test]
fn angelic_destiny_returns_to_hand_when_its_creature_dies() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Angelic Destiny");
    zones::die(&mut g, host, "destroy").unwrap();
    assert!(in_hand(&g, P0, "Angelic Destiny"));
    assert!(!in_gy(&g, P0, "Angelic Destiny"));
}

#[test]
fn umbra_armor_destroys_the_umbra_instead() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    let u = perm(&mut g, P0, "Mammoth Umbra");
    zones::die(&mut g, host, "destroy").unwrap();
    assert!(g.perm(host).on_bf);
    assert!(!g.perm(u).on_bf);
    assert!(in_gy(&g, P0, "Mammoth Umbra"));
    // a sacrifice isn't a destroy: no umbra left anyway, the creature goes
    zones::die(&mut g, host, "sac").unwrap();
    assert!(!g.perm(host).on_bf);
}

#[test]
fn scaling_auras_count_your_enchantments_and_auras() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    let p0 = values::epow(&g, host);
    perm(&mut g, P0, "Ethereal Armor"); // +1/+1 per enchantment: itself
    assert_eq!(values::epow(&g, host), p0 + 1);
    perm(&mut g, P0, "Rest in Peace"); // another enchantment
    assert_eq!(values::epow(&g, host), p0 + 2);
    perm(&mut g, P0, "Sage's Reverie"); // +1 per Aura (two), and Ethereal Armor's third enchantment
    assert_eq!(values::epow(&g, host), p0 + 3 + 2);
    assert!(dsl::has_kw(&g, host, "first strike"));
}

#[test]
fn sages_reverie_draws_a_card_per_aura() {
    let mut g = table(&[LP, TEYSA]);
    perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Battle Mastery");
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Sage's Reverie");
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn spectra_ward_gives_protection_from_every_colour() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Spectra Ward");
    let black = sim_core::cards::Colors::from_letters("B");
    assert!(values::protected_from(&g, host, black));
    assert_eq!(values::epow(&g, host), 1 + 2);
}

#[test]
fn flickering_ward_protects_from_black() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Flickering Ward");
    assert!(values::protected_from(&g, host, sim_core::cards::Colors::from_letters("B")));
    assert!(!values::protected_from(&g, host, sim_core::cards::Colors::from_letters("R")));
}

#[test]
fn cartouche_makes_a_warrior() {
    let mut g = table(&[LP, TEYSA]);
    perm(&mut g, P0, "Esper Sentinel");
    let n = g.player(P0).perms.len();
    perm(&mut g, P0, "Cartouche of Solidarity");
    assert_eq!(g.player(P0).perms.len(), n + 2); // the Aura and a 1/1 Warrior
}

#[test]
fn spirit_link_gains_the_combat_damage() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Spirit Link");
    combat_damage(&mut g, P0, host, P1, 3);
    assert_eq!(g.player(P0).life, 43);
}

#[test]
fn celestial_mantle_doubles_your_life() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Celestial Mantle");
    combat_damage(&mut g, P0, host, P1, 4);
    assert_eq!(g.player(P0).life, 80);
}

// ------------------------------------------------------------------ the Oblivion Ring family
#[test]
fn oblivion_ring_exiles_the_best_threat_until_it_leaves() {
    let mut g = table(&[LP, TEYSA]);
    let t = perm(&mut g, P1, "Grave Pact");
    let o = perm(&mut g, P0, "Oblivion Ring");
    assert!(!g.perm(t).on_bf);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Pact"));
    zones::die(&mut g, o, "destroy").unwrap();
    assert_eq!(named(&g, P1, "Grave Pact").len(), 1); // back
    assert!(!g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Pact"));
}

#[test]
fn journey_to_nowhere_takes_only_creatures() {
    let mut g = table(&[LP, TEYSA]);
    let pact = perm(&mut g, P1, "Grave Pact"); // the bigger threat, but not a creature
    let c = perm(&mut g, P1, "Cruel Celebrant");
    perm(&mut g, P0, "Journey to Nowhere");
    assert!(g.perm(pact).on_bf);
    assert!(!g.perm(c).on_bf);
}

#[test]
fn cast_out_skips_what_is_protected() {
    let mut g = table(&[LP, TEYSA]);
    let host = perm(&mut g, P1, "Teysa Karlov");
    // Spectra Ward on the opponent's creature (it's theirs): protection from white stops the O-Ring
    let w = perm(&mut g, P1, "Spectra Ward");
    assert_eq!(g.perm(w).attached, Some(host));
    perm(&mut g, P0, "Cast Out");
    assert!(g.perm(host).on_bf);
}

// ------------------------------------------------------------------ sacrifice outlets and death payoffs
#[test]
fn aristocrats_sacrifice_a_token_for_lethal_drains() {
    let mut g = table_with(&[TEYSA, LP, ISSHIN], 1, 1); // the opponents are at 1
    g.player_mut(P0).life = 40;
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Zulaport Cutthroat");
    let t = token(&mut g, P0, 1);
    let o = cardcode::aristocrat_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].utility, 9.0);
    assert!(o[0].label.contains("Viscera Seer"));
    let act = o[0].act.clone().unwrap();
    assert!(sim_core::ai::act::perform(&mut g, P0, &act).unwrap());
    assert!(!g.perm(t).on_bf);
}

#[test]
fn no_outlet_no_aristocrat_play() {
    let mut g = table(&[TEYSA, LP]);
    perm(&mut g, P0, "Zulaport Cutthroat");
    token(&mut g, P0, 1);
    assert!(cardcode::aristocrat_options(&mut g, P0, Some(true)).unwrap().is_empty());
    assert!(cardcode::is_sac_outlet("Viscera Seer"));
    assert!(cardcode::is_sac_outlet("Phyrexian Altar"));
    assert!(!cardcode::is_sac_outlet("Zulaport Cutthroat"));
}

#[test]
fn a_creature_about_to_be_exiled_is_sacrificed_instead() {
    let mut g = table(&[TEYSA, LP]);
    perm(&mut g, P0, "Phyrexian Altar");
    let c = perm(&mut g, P0, "Cruel Celebrant");
    assert!(cardcode::sac_in_response(&mut g, P0, c, "exile").unwrap());
    assert!(!g.perm(c).on_bf);
    assert!(in_gy(&g, P0, "Cruel Celebrant"));
    assert_eq!(g.player(P0).floating.any, 1); // the Altar's mana
    let d = perm(&mut g, P0, "Vindictive Vampire");
    assert!(!cardcode::sac_in_response(&mut g, P0, d, "destroy").unwrap()); // it dies anyway
}

#[test]
fn cruel_celebrant_drains_on_each_death() {
    let mut g = table(&[TEYSA, LP, ISSHIN]);
    perm(&mut g, P0, "Cruel Celebrant");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sac").unwrap();
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P2).life, 39);
    assert_eq!(g.player(P0).life, 42);
}

#[test]
fn grim_haruspex_draws_for_nontoken_deaths_only() {
    let mut g = table(&[TEYSA, LP]);
    perm(&mut g, P0, "Grim Haruspex");
    let n = g.player(P0).hand.len();
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sac").unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
    let c = perm(&mut g, P0, "Vindictive Vampire");
    zones::die(&mut g, c, "sac").unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn revel_in_riches_makes_treasure_from_opponents_deaths() {
    let mut g = table(&[TEYSA, LP]);
    perm(&mut g, P0, "Revel in Riches");
    let mine = token(&mut g, P0, 1);
    zones::die(&mut g, mine, "sac").unwrap();
    assert_eq!(g.player(P0).treasures, 0);
    let theirs = token(&mut g, P1, 1);
    zones::die(&mut g, theirs, "sac").unwrap();
    assert_eq!(g.player(P0).treasures, 1);
}

#[test]
fn priest_of_forgotten_gods_needs_two_spare_creatures() {
    let mut g = table(&[TEYSA, LP]);
    let pr = perm(&mut g, P0, "Priest of Forgotten Gods");
    token(&mut g, P0, 1);
    assert!(opts(&mut g, pr, P0).is_empty());
    token(&mut g, P0, 1);
    perm(&mut g, P1, "Esper Sentinel");
    let o = opts(&mut g, pr, P0);
    let o = o.iter().find(|o| o.label == "Priest of Forgotten Gods").expect("the Priest's option");
    assert_eq!(o.utility, 4.0);
    let n = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.perm(pr).tapped);
    assert_eq!(g.player(P1).life, 38);
    assert!(named(&g, P1, "Esper Sentinel").is_empty()); // the edict
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

// ------------------------------------------------------------------ staples
#[test]
fn rest_in_peace_empties_the_graveyards() {
    let mut g = table(&[LP, TEYSA]);
    let c = take(&mut g, P1, "Cruel Celebrant");
    g.player_mut(P1).gy.push(c);
    perm(&mut g, P0, "Rest in Peace");
    assert!(g.player(P1).gy.is_empty());
    assert!(g.player(P1).exile.contains(&c));
    // from now on what dies is exiled
    let v = perm(&mut g, P1, "Vindictive Vampire");
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(!in_gy(&g, P1, "Vindictive Vampire"));
}

#[test]
fn mind_stone_is_cracked_late() {
    let mut g = table(&[LP, TEYSA]);
    let ms = perm(&mut g, P0, "Mind Stone");
    lands(&mut g, P0, "Plains", 5, false);
    assert!(opts(&mut g, ms, P0).is_empty()); // five lands
    lands(&mut g, P0, "Plains", 1, false);
    let n = g.player(P0).hand.len();
    let o = opts(&mut g, ms, P0);
    let o = o.iter().find(|o| o.label == "crack Mind Stone").unwrap();
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(ms).on_bf);
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn aura_discounts() {
    let mut g = table(&[LP, TEYSA]);
    let aura = card(&g, "Hyena Umbra");
    let other = card(&g, "Rest in Peace");
    let white = card(&g, "Esper Sentinel");
    assert_eq!(hooks::total_cost(&g, P0, aura), 0);
    perm(&mut g, P0, "Danitha Capashen, Paragon");
    perm(&mut g, P0, "Starfield Mystic");
    assert_eq!(hooks::total_cost(&g, P0, aura), -2); // an Aura and an enchantment
    assert_eq!(hooks::total_cost(&g, P0, other), -1);
    assert_eq!(hooks::total_cost(&g, P1, aura), 0); // only yours
    perm(&mut g, P0, "Pearl Medallion");
    assert_eq!(hooks::total_cost(&g, P0, white), -1);
    let _ = mana::cost_of(&g, P0, aura);
}

#[test]
fn esper_sentinel_draws_on_an_opponents_first_noncreature_spell() {
    let mut g = table(&[LP, TEYSA]);
    perm(&mut g, P0, "Esper Sentinel");
    let n = g.player(P0).hand.len();
    let c = take(&mut g, P1, "Sol Ring");
    cast::on_cast(&mut g, P1, c).unwrap(); // no mana to pay the X
    assert_eq!(g.player(P0).hand.len(), n + 1);
    let c2 = take(&mut g, P1, "Arcane Signet");
    cast::on_cast(&mut g, P1, c2).unwrap(); // the second one: no trigger
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn hero_of_bladehold_makes_attacking_soldiers() {
    let mut g = table(&[ISSHIN, TEYSA]);
    let h = perm(&mut g, P0, "Hero of Bladehold");
    g.new_attackers.clear();
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![h], d: P1 }).unwrap();
    let new = g.new_attackers.clone();
    assert_eq!(new.len(), 2);
    assert!(new.iter().all(|&m| g.perm(m).tapped && !g.perm(m).sick));
}

#[test]
fn allosaurus_shepherd_makes_green_spells_uncounterable() {
    let mut g = table(&["lathril-golgari-elves", TEYSA]);
    let green = card(&g, "Llanowar Elves");
    assert!(!hooks::any_uncounterable(&g, P0, green));
    perm(&mut g, P0, "Allosaurus Shepherd");
    assert!(hooks::any_uncounterable(&g, P0, green));
    assert!(!hooks::any_uncounterable(&g, P1, green));
}
