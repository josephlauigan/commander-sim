//! The Tier 1 light-paws-aura-voltron cards (impls/t12_lightpaws.rs and the compiled patterns under
//! `// --- Tiers 1/2 (lightpaws)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::cardcode;
use sim_core::cards::Colors;
use sim_core::dsl::{cost_delta, has_kw};
use sim_core::engine::hooks::{fire_trigger, total_cost};
use sim_core::engine::values::{epow, etgh, indestructible, protected_from, untargetable};
use sim_core::engine::{cast, life, removal, zones};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const LP: &str = "light-paws-aura-voltron";
const PAWS: &str = "Light-Paws, Emperor's Voice";

fn game() -> Game {
    table(&[LP, "seph"])
}

/// Light-Paws on the battlefield as the commander
fn paws(g: &mut Game) -> PermId {
    let m = perm(g, P0, PAWS);
    g.perm_mut(m).is_cmd = true;
    m
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn on(g: &Game, m: PermId) -> usize {
    g.auras.iter().filter(|&&a| g.perm(a).on_bf && g.perm(a).attached == Some(m)).count()
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

/// removal of this kind from actor, by the named spell
fn hit(g: &mut Game, actor: PlayerId, m: PermId, kind: &'static str, spell: &str) {
    let s = card(g, spell);
    removal::apply_removal(g, Some(actor), m, kind, Some(s)).unwrap();
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, p, c, &Ctx::default()).unwrap();
}

// ------------------------------------------------------------------ static Auras
#[test]
fn armored_ascension_counts_plains_and_gives_flying() {
    let mut g = game();
    let m = paws(&mut g);
    lands(&mut g, P0, "Plains", 3, false);
    let p0 = epow(&g, m);
    let a = perm(&mut g, P0, "Armored Ascension");
    assert_eq!(g.perm(a).attached, Some(m));
    assert_eq!(epow(&g, m), p0 + 3);
    assert!(has_kw(&g, m, "flying"));
}

#[test]
fn angelic_gift_draws_once_and_gives_flying() {
    let mut g = game();
    let m = paws(&mut g);
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Angelic Gift");
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(has_kw(&g, m, "flying"));
}

#[test]
fn mask_and_shield_give_protection_from_their_colours() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Mask of Law and Grace");
    assert!(protected_from(&g, m, Colors::from_letters("B")) && protected_from(&g, m, Colors::from_letters("R")));
    assert!(!protected_from(&g, m, Colors::from_letters("G")));
    perm(&mut g, P0, "Shield of Duty and Reason");
    assert!(protected_from(&g, m, Colors::from_letters("G")) && protected_from(&g, m, Colors::from_letters("U")));
    assert!(!protected_from(&g, m, Colors::from_letters("W")));
}

#[test]
fn face_of_divinity_needs_another_aura_for_first_strike_and_lifelink() {
    let mut g = game();
    let m = paws(&mut g);
    let (p0, t0) = (epow(&g, m), etgh(&g, m));
    perm(&mut g, P0, "Face of Divinity");
    assert_eq!((epow(&g, m), etgh(&g, m)), (p0 + 2, t0 + 2));
    assert!(!has_kw(&g, m, "first strike") && !has_kw(&g, m, "lifelink"));
    perm(&mut g, P0, "Spirit Link");
    assert!(has_kw(&g, m, "first strike") && has_kw(&g, m, "lifelink"));
}

#[test]
fn holy_mantle_twinblade_and_stoneskin() {
    let mut g = game();
    let m = paws(&mut g);
    let (p0, t0) = (epow(&g, m), etgh(&g, m));
    perm(&mut g, P0, "Holy Mantle");
    let t = token(&mut g, P1, 3);
    assert!(cardcode::prot_vs(&g, m, t), "protection from creatures");
    assert_eq!((epow(&g, m), etgh(&g, m)), (p0 + 2, t0 + 2));
    perm(&mut g, P0, "Twinblade Blessing");
    assert!(has_kw(&g, m, "double strike"));
    perm(&mut g, P0, "Stoneskin");
    assert_eq!((epow(&g, m), etgh(&g, m)), (p0 + 2, t0 + 12));
}

// ------------------------------------------------------------------ triggered Auras
#[test]
fn idolized_pumps_a_creature_attacking_alone() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Idolized");
    perm(&mut g, P0, "Sol Ring");
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![m], d: P1 }).unwrap();
    assert_eq!(g.perm(m).eot_pt, (3, 3)); // Light-Paws, Idolized, Sol Ring
    g.perm_mut(m).eot_pt = (0, 0);
    let t = token(&mut g, P0, 1);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![m, t], d: P1 }).unwrap();
    assert_eq!(g.perm(m).eot_pt, (0, 0), "not alone");
}

#[test]
fn light_of_promise_puts_counters_for_life_gained() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Light of Promise");
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.perm(m).plus, 3);
    life::gain(&mut g, P1, 2).unwrap(); // an opponent's gain: nothing
    assert_eq!(g.perm(m).plus, 3);
}

#[test]
fn chains_of_custody_exiles_until_it_leaves_and_gives_ward() {
    let mut g = game();
    let m = paws(&mut g);
    let h = perm(&mut g, P1, "Hero of Bladehold");
    let a = perm(&mut g, P0, "Chains of Custody");
    assert_eq!(g.perm(a).attached, Some(m));
    assert!(!g.perm(h).on_bf, "the best opposing nonland permanent is exiled");
    assert_eq!(cardcode::aura_ward(&g, m), 2);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(named(&g, P1, "Hero of Bladehold").len(), 1, "it returns when Chains leaves");
}

#[test]
fn sheltered_by_ghosts_returns_its_exile_when_it_leaves() {
    let mut g = game();
    paws(&mut g);
    perm(&mut g, P1, "Hero of Bladehold");
    let a = perm(&mut g, P0, "Sheltered by Ghosts");
    assert!(named(&g, P1, "Hero of Bladehold").is_empty());
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(named(&g, P1, "Hero of Bladehold").len(), 1);
}

#[test]
fn mantle_of_the_ancients_returns_auras_and_equipment_and_counts_them() {
    let mut g = game();
    let m = paws(&mut g);
    to_gy(&mut g, P0, "Battle Mastery");
    to_gy(&mut g, P0, "Swiftfoot Boots");
    let p0 = epow(&g, m);
    perm(&mut g, P0, "Mantle of the Ancients");
    assert_eq!(on(&g, m), 2, "Mantle and Battle Mastery");
    let boots = named(&g, P0, "Swiftfoot Boots");
    assert_eq!(boots.len(), 1);
    assert_eq!(g.perm(boots[0]).attached, Some(m));
    assert_eq!(epow(&g, m), p0 + 3, "two Auras and an Equipment");
    assert!(has_kw(&g, m, "double strike"));
}

#[test]
fn pariah_takes_the_damage_and_dies_to_enough_of_it() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Stoneskin"); // toughness 13
    let a = perm(&mut g, P0, "Pariah");
    assert_eq!(g.perm(a).attached, Some(m));
    let life0 = g.player(P0).life;
    life::lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life0, "dealt to the creature instead");
    assert!(g.perm(m).on_bf);
    life::lose_life(&mut g, P0, 2, Some(P1), "drain", None).unwrap(); // life loss is not damage
    assert_eq!(g.player(P0).life, life0 - 2);
    life::lose_life(&mut g, P0, 8, Some(P1), "burn", None).unwrap();
    assert!(!g.perm(m).on_bf, "13 damage this turn kills it");
}

#[test]
fn pariah_only_goes_on_a_creature_that_survives_it() {
    let mut g = game();
    paws(&mut g); // 3/3, not indestructible
    let c = card(&g, "Pariah");
    let f = g.registry.get(c).unwrap().prio.unwrap();
    assert_eq!(f(&g, P0, c), 0);
    perm(&mut g, P0, "Timely Ward"); // indestructible
    assert!(f(&g, P0, c) > 0);
}

#[test]
fn umbra_mystic_gives_auras_umbra_armor() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Battle Mastery");
    perm(&mut g, P0, "Umbra Mystic");
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.perm(m).on_bf, "Battle Mastery is destroyed instead");
    assert_eq!(on(&g, m), 0);
}

#[test]
fn thran_power_suit_counts_auras_and_equipment_and_has_ward() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Battle Mastery");
    let p0 = epow(&g, m);
    let s = perm(&mut g, P0, "Thran Power Suit");
    assert!(sim_core::impls::cloud::attach(&mut g, s, m));
    assert_eq!(epow(&g, m), p0 + 2, "Battle Mastery and the Suit");
    assert_eq!(cardcode::aura_ward(&g, m), 2);
}

#[test]
fn reprobation_makes_a_0_1_without_abilities() {
    let mut g = game();
    paws(&mut g);
    let h = perm(&mut g, P1, "Hero of Bladehold");
    hit(&mut g, P0, h, "coward", "Reprobation");
    assert!(g.perm(h).on_bf && g.perm(h).neutered);
    assert_eq!((epow(&g, h), etgh(&g, h)), (0, 1));
    assert!(!indestructible(&g, h), "a Coward, not Darksteel");
}

// ------------------------------------------------------------------ cost reducers and Pearl-Ear
#[test]
fn transcendent_envoy_and_starnheim_courser_discount_their_spells() {
    let mut g = game();
    perm(&mut g, P0, "Transcendent Envoy");
    let (aura, ring, guy) = (card(&g, "Battle Mastery"), card(&g, "Sol Ring"), card(&g, "Umbra Mystic"));
    assert_eq!(cost_delta(&g, P0, aura), -1);
    assert_eq!(cost_delta(&g, P0, ring), 0);
    perm(&mut g, P0, "Starnheim Courser");
    assert_eq!(cost_delta(&g, P0, aura), -2);
    assert_eq!(cost_delta(&g, P0, ring), -1);
    assert_eq!(cost_delta(&g, P0, guy), 0);
    assert_eq!(cost_delta(&g, P1, ring), 0, "only its controller's spells");
}

#[test]
fn hero_of_iroas_discounts_auras_once() {
    let mut g = game();
    perm(&mut g, P0, "Hero of Iroas");
    let aura = card(&g, "Battle Mastery");
    assert_eq!(cost_delta(&g, P0, aura) + total_cost(&g, P0, aura), -1);
}

#[test]
fn pearl_ear_has_affinity_for_auras_and_draws_on_auras_for_modified_permanents() {
    let mut g = game();
    let pe = perm(&mut g, P0, "Pearl-Ear, Imperial Advisor");
    let armor = card(&g, "Ethereal Armor");
    assert_eq!(total_cost(&g, P0, armor), 0);
    let n = g.player(P0).hand.len();
    cast::on_cast(&mut g, P0, armor).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "Pearl-Ear is not modified yet");
    perm(&mut g, P0, "Battle Mastery");
    perm(&mut g, P0, "Spirit Link");
    assert_eq!(on(&g, pe), 2);
    assert_eq!(total_cost(&g, P0, armor), -2);
    assert_eq!(total_cost(&g, P0, card(&g, "Sol Ring")), 0);
    cast::on_cast(&mut g, P0, armor).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

// ------------------------------------------------------------------ sorceries
#[test]
fn retether_returns_each_aura_card_onto_a_creature() {
    let mut g = game();
    let m = paws(&mut g);
    to_gy(&mut g, P0, "Battle Mastery");
    to_gy(&mut g, P0, "Ethereal Armor");
    let c = card(&g, "Retether");
    let prio = g.registry.get(c).unwrap().prio.unwrap();
    assert!(prio(&g, P0, c) > 0);
    resolve(&mut g, P0, "Retether");
    assert_eq!(on(&g, m), 2);
    assert!(!g.player(P0).gy.iter().any(|&x| g.db.get(x).has_subtype("aura")));
}

#[test]
fn single_combat_leaves_one_each_and_locks_creature_spells() {
    let mut g = game();
    let m = paws(&mut g);
    token(&mut g, P0, 1);
    let big = perm(&mut g, P1, "Hero of Bladehold");
    for _ in 0..3 {
        token(&mut g, P1, 2);
    }
    resolve(&mut g, P0, "Single Combat");
    assert!(g.perm(m).on_bf && g.perm(big).on_bf);
    let creatures = |g: &Game, p: PlayerId| g.player(p).perms.iter().filter(|&&x| g.is_creature(x)).count();
    assert_eq!((creatures(&g, P0), creatures(&g, P1)), (1, 1));
    let guy = take(&mut g, P1, "Umbra Mystic");
    g.player_mut(P1).hand.push(guy);
    assert!(!cast::castable(&g, P1, guy, "hand"));
    let ring = take(&mut g, P1, "Sol Ring");
    g.player_mut(P1).hand.push(ring);
    assert!(cast::castable(&g, P1, ring, "hand"));
}

// ------------------------------------------------------------------ answers to removal
#[test]
fn blacksmiths_skill_answers_removal() {
    let mut g = game();
    let m = paws(&mut g);
    hand(&mut g, P0, &["Blacksmith's Skill"]);
    lands(&mut g, P0, "Plains", 1, false);
    hit(&mut g, P1, m, "destroy", "Swords to Plowshares");
    assert!(g.perm(m).on_bf);
    assert!(untargetable(&g, m) && indestructible(&g, m));
}

#[test]
fn karametras_blessing_saves_an_enchanted_creature() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Battle Mastery");
    hand(&mut g, P0, &["Karametra's Blessing"]);
    lands(&mut g, P0, "Plains", 1, false);
    hit(&mut g, P1, m, "exile", "Swords to Plowshares");
    assert!(g.perm(m).on_bf);
    assert_eq!(g.perm(m).eot_pt, (2, 2));
}

#[test]
fn benevolent_blessing_answers_a_coloured_spell() {
    let mut g = game();
    let m = paws(&mut g);
    hand(&mut g, P0, &["Benevolent Blessing"]);
    lands(&mut g, P0, "Plains", 2, false);
    hit(&mut g, P1, m, "exile", "Swords to Plowshares");
    assert!(g.perm(m).on_bf);
    assert!(protected_from(&g, m, Colors::from_letters("W")));
}

#[test]
fn stoneskin_answers_damage_it_survives() {
    let mut g = game();
    let m = paws(&mut g);
    hand(&mut g, P0, &["Stoneskin"]);
    lands(&mut g, P0, "Plains", 3, false);
    hit(&mut g, P1, m, "dmg4", "Lightning Bolt");
    assert!(g.perm(m).on_bf);
    assert!(etgh(&g, m) > 10);
}

#[test]
fn flickerform_saves_the_creature_and_its_auras_until_the_end_step() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Flickerform");
    perm(&mut g, P0, "Battle Mastery");
    lands(&mut g, P0, "Plains", 4, false);
    hit(&mut g, P1, m, "exile", "Swords to Plowshares");
    assert!(!g.perm(m).on_bf);
    assert!(named(&g, P0, PAWS).is_empty() && g.auras.is_empty());
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    let back = named(&g, P0, PAWS);
    assert_eq!(back.len(), 1);
    assert!(g.perm(back[0]).is_cmd);
    assert_eq!(on(&g, back[0]), 2, "Flickerform and Battle Mastery return attached");
    assert!(has_kw(&g, back[0], "double strike"));
}

#[test]
fn flickerform_dodges_a_wipe() {
    let mut g = game();
    let m = paws(&mut g);
    perm(&mut g, P0, "Flickerform");
    perm(&mut g, P0, "Ethereal Armor");
    lands(&mut g, P0, "Plains", 4, false);
    sim_core::ai::decks::wipe_response(&mut g, P0, "destroy", P1).unwrap();
    assert!(!g.perm(m).on_bf);
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert_eq!(named(&g, P0, PAWS).len(), 1);
}
