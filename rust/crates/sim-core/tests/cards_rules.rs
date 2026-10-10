//! Card code of partials.py, rules.py, rules2.py, fixes.py and t3.py (impls/partials.rs ... t3.rs) on hand-built
//! positions: Python's tests/test_rules.py DeathriteShaman, test_stack.Abilities' Azorius Guildmage and
//! test_triggers' Tidebinder and Tithe Taker, then each pilot card's own clause from its rules text.

use sim_core::ai::act::perform;
use sim_core::engine::{removal, stack, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// src's activated-ability options (its card's `options` hook), as the AI sees them in a main phase
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, post).unwrap()
}

fn option_with(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>, word: &str) -> Action {
    options(g, src, p, post).into_iter().find(|o| o.label.contains(word)).and_then(|o| o.act).expect("the option")
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

fn drained(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).filter(|&q| g.player(q).life < 40).collect()
}

// ------------------------------------------------------------------ test_rules.DeathriteShaman
#[test]
fn deathrite_cannot_pay_with_itself() {
    // {G}, {T}: the {T} is part of the cost, so Deathrite can't also tap for the {G}
    let mut g = table(&["korvold-jund-sacrifice", "seph"]);
    let (k, s) = (P0, P1);
    let d = perm(&mut g, k, "Deathrite Shaman");
    let (titan, swamp) = (card(&g, "Grave Titan"), card(&g, "Swamp"));
    g.player_mut(s).gy.extend([titan, swamp]); // a land card for its own mana
    let a = option_with(&mut g, d, k, Some(false), "exile");
    assert!(!perform(&mut g, k, &a).unwrap());
    assert!(!g.perm(d).tapped);
}

#[test]
fn deathrite_exiles_and_gains() {
    let mut g = table(&["korvold-jund-sacrifice", "seph"]);
    let (k, s) = (P0, P1);
    let d = perm(&mut g, k, "Deathrite Shaman");
    let titan = card(&g, "Grave Titan");
    g.player_mut(s).gy.push(titan);
    lands(&mut g, k, "Forest", 1, false);
    let a = option_with(&mut g, d, k, Some(false), "exile");
    assert!(perform(&mut g, k, &a).unwrap());
    assert!(g.perm(d).tapped && g.land(g.player(k).lands[0]).tapped);
    assert_eq!(g.player(s).exile, [titan]);
    assert_eq!(g.player(k).life, 42);
}

// ------------------------------------------------------------------ test_stack.Abilities: Azorius Guildmage
#[test]
fn the_ai_counters_an_important_ability_with_azorius_guildmage() {
    let mut g = table(&["veyran", "yshtola"]);
    let (v, z) = (P0, P1);
    perm(&mut g, z, "Azorius Guildmage");
    lands(&mut g, z, "Island", 3, false);
    let src = perm(&mut g, v, "Triskelion");
    assert!(!stack::ability_window(&mut g, v, Some(src), "1 damage", Some(8.0), None).unwrap()); // countered
    assert!(stack::ability_window(&mut g, v, Some(src), "1 damage", Some(2.0), None).unwrap()); // not worth the mana
}

#[test]
fn a_countered_equip_does_not_attach() {
    let mut g = table(&["sauron", "yshtola"]);
    let (s, z) = (P0, P1);
    perm(&mut g, z, "Azorius Guildmage");
    lands(&mut g, z, "Island", 3, false);
    let a = perm(&mut g, s, "Orcish Bowmasters");
    let e = perm(&mut g, s, "Lightning Greaves");
    lands(&mut g, s, "Swamp", 2, false);
    stack::equip_to(&mut g, s, e, a, 0).unwrap(); // imp 3 + Greaves' value: below the AI's bar, so it resolves
    assert_eq!(g.perm(e).attached, Some(a));
}

// ------------------------------------------------------------------ test_triggers: Tidebinder, Tithe Taker
#[test]
fn tidebinder_counters_a_trigger() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    let (s, v) = (P0, P1);
    hand(&mut g, v, &["Tishana's Tidebinder"]);
    lands(&mut g, v, "Island", 3, false);
    perm(&mut g, s, "Archon of Cruelty");
    assert_eq!(drained(&g, s), []); // the drain never happened
    assert_eq!(named(&g, v, "Tishana's Tidebinder").len(), 1);
}

#[test]
fn tithe_taker_leaves_one_spirit() {
    // afterlife 1 (test_triggers: the probe never doubles an effect)
    let mut g = table(&["yshtola", "veyran", "sauron"]);
    let z = P0;
    let m = perm(&mut g, z, "Tithe Taker");
    zones::die(&mut g, m, "destroy").unwrap();
    let toks: Vec<PermId> = g.player(z).perms.iter().copied().filter(|&x| g.perm(x).token).collect();
    assert_eq!(toks.len(), 1);
    assert!(g.perm(toks[0]).fly);
}

// ------------------------------------------------------------------ partials
#[test]
fn druid_class_level_three_animates_a_land() {
    let mut g = table(&["tatyova-simic-landfall", "sauron"]);
    lands(&mut g, P0, "Forest", 7, false);
    lands(&mut g, P0, "Command Tower", 1, false);
    let d = perm(&mut g, P0, "Druid Class");
    g.perm_mut(d).data.set(DataKey::Level, Val::Int(2));
    let a = option_with(&mut g, d, P0, Some(false), "level 3");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.perm(d).data.get(DataKey::Level), Some(&Val::Int(3)));
    assert_eq!(g.player(P0).lands.len(), 7); // a Forest became the creature (Command Tower makes more colours)
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).data.truthy(DataKey::DruidClass)).unwrap();
    assert!(g.is_creature(m) && !g.perm(m).sick);
    assert_eq!((g.perm(m).pow, g.perm(m).tgh), (8, 8)); // lands you control, itself included
    assert_eq!(g.perm(m).name, "Forest");
    lands(&mut g, P0, "Island", 2, false);
    let up = g.registry.get(card(&g, "Druid Class")).unwrap().upkeep.unwrap();
    up(&mut g, d, P0).unwrap();
    assert_eq!(g.perm(m).pow, 10); // the size follows the lands at upkeep
}

#[test]
fn rapid_hybridization_leaves_a_frog_lizard() {
    let mut g = table(&["tatyova-simic-landfall", "sauron"]);
    let t = perm(&mut g, P1, "Llanowar Elves");
    let c = card(&g, "Rapid Hybridization");
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    assert_eq!(f(&mut g, P0, c, &Ctx { target: Some(t), ..Ctx::default() }).unwrap(), "gy");
    assert!(!g.perm(t).on_bf);
    let toks: Vec<PermId> = g.player(P1).perms.iter().copied().filter(|&x| g.perm(x).token).collect();
    assert_eq!(toks.len(), 1);
    assert_eq!((g.perm(toks[0]).pow, g.perm(toks[0]).tgh), (3, 3));
    assert!(values::has_type(&g, toks[0], "frog") && values::has_type(&g, toks[0], "lizard"));
}

#[test]
fn retreat_to_coralhelm_untaps_a_mana_creature() {
    let mut g = table(&["tatyova-simic-landfall", "sauron"]);
    perm(&mut g, P0, "Retreat to Coralhelm");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(elf).tapped = true;
    zones::landfall(&mut g, P0).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert!(!g.perm(elf).tapped);
}

#[test]
fn sunfall_exiles_everything_and_incubates() {
    let mut g = table(&["light-paws-aura-voltron", "sauron"]);
    perm(&mut g, P0, "Mother of Runes");
    perm(&mut g, P1, "Grave Titan"); // and its two Zombies
    token(&mut g, P1, 2);
    let c = card(&g, "Sunfall");
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    assert!(g.players.iter().all(|q| q.perms.iter().all(|&m| !g.is_creature(m))));
    assert_eq!(g.player(P0).incubator, 5);
    lands(&mut g, P0, "Plains", 2, false);
    let o = sim_core::impls::partials::incubator_options(&mut g, P0, Some(false)).unwrap();
    assert!(perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).incubator, 0);
    let m = *g.player(P0).perms.last().unwrap();
    assert_eq!((g.perm(m).pow, g.perm(m).tgh), (5, 5));
}

#[test]
fn tajic_prevents_damage_to_the_others() {
    let mut g = table(&["isshin-mardu-attack-triggers", "sauron"]);
    perm(&mut g, P0, "Tajic, Legion's Edge");
    let m = token(&mut g, P0, 2);
    removal::apply_removal(&mut g, Some(P1), m, "dmg3", None).unwrap();
    assert!(g.perm(m).on_bf);
}

// ------------------------------------------------------------------ rules
#[test]
fn kenriths_transformation_makes_an_elk() {
    let mut g = table(&["sauron", "veyran"]);
    let t = perm(&mut g, P1, "Grave Titan");
    removal::apply_removal(&mut g, Some(P0), t, "elk", None).unwrap();
    let x = g.perm(t);
    assert!(x.on_bf && x.neutered && x.data.truthy(DataKey::Elk));
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (3, 3));
}

#[test]
fn ezuri_regenerates_a_valuable_elf() {
    let mut g = table(&["lathril-golgari-elves", "sauron"]);
    perm(&mut g, P0, "Ezuri, Renegade Leader");
    let elf = perm(&mut g, P0, "Elvish Archdruid");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(values::pval(&g, elf) >= 3.0);
    zones::die(&mut g, elf, "destroy").unwrap();
    assert!(g.perm(elf).on_bf && g.perm(elf).tapped);
    assert!(g.land(g.player(P0).lands[0]).tapped);
    let cheap = token(&mut g, P0, 1); // a token isn't an Elf: no regeneration
    zones::die(&mut g, cheap, "destroy").unwrap();
    assert!(!g.perm(cheap).on_bf);
}

#[test]
fn fact_or_fiction_splits_the_five() {
    let mut g = table(&["sauron", "veyran"]);
    let c = card(&g, "Fact or Fiction");
    let (h, gy, lib) = (g.player(P0).hand.len(), g.player(P0).gy.len(), g.player(P0).library.len());
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    let got = g.player(P0).hand.len() - h;
    assert!(got >= 1 && got <= 5);
    assert_eq!(got + g.player(P0).gy.len() - gy, 5);
    assert_eq!(g.player(P0).library.len(), lib - 5);
}

#[test]
fn allosaurus_shepherd_protects_green_spells() {
    let mut g = table(&["lathril-golgari-elves", "sauron"]);
    perm(&mut g, P0, "Allosaurus Shepherd");
    let green = card(&g, "Llanowar Elves");
    let black = card(&g, "Swamp");
    assert!(sim_core::engine::hooks::any_uncounterable(&g, P0, green));
    assert!(!sim_core::engine::hooks::any_uncounterable(&g, P1, green));
    assert!(!sim_core::engine::hooks::any_uncounterable(&g, P0, black));
}

// ------------------------------------------------------------------ rules2
#[test]
fn rabblemaster_makes_other_goblins_attack() {
    let mut g = table(&["isshin-mardu-attack-triggers", "sauron"]);
    let r = perm(&mut g, P0, "Goblin Rabblemaster");
    let gob = perm(&mut g, P0, "Goblin Instigator");
    let other = token(&mut g, P0, 2);
    let xs = sim_core::cardcode::forced_attackers(&mut g, P0, vec![], &[r, gob, other]).unwrap();
    assert_eq!(xs.iter().filter(|&&m| m == gob).count(), 1);
    assert!(!xs.contains(&r) && !xs.contains(&other));
}

#[test]
fn springbloom_druid_trades_a_land_for_two_basics() {
    let mut g = table(&["tatyova-simic-landfall", "sauron"]);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Command Tower", 1, false);
    perm(&mut g, P0, "Springbloom Druid");
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P0).lands.len(), 4);
    assert_eq!(g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Forest").count(), 1); // a basic goes first
}

// ------------------------------------------------------------------ fixes
#[test]
fn aetherize_returns_a_big_attack() {
    let mut g = table(&["sauron", "tatyova-simic-landfall"]);
    let (p, d) = (P0, P1);
    let mut atk: Vec<PermId> = (0..4).map(|_| token(&mut g, p, 3)).collect();
    hand(&mut g, d, &["Aetherize"]);
    lands(&mut g, d, "Island", 4, false);
    let mut assign = vec![];
    sim_core::cardcode::defend_hooks(&mut g, p, &mut atk, d, &mut assign).unwrap();
    assert!(atk.iter().all(|&a| !g.perm(a).on_bf));
    assert_eq!(g.player(d).gy, [card(&g, "Aetherize")]);
}

#[test]
fn crackling_doom_takes_the_greatest_power() {
    let mut g = table(&["isshin-mardu-attack-triggers", "sauron", "veyran"]);
    let big = token(&mut g, P1, 5);
    let small = token(&mut g, P1, 1);
    let c = card(&g, "Crackling Doom");
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    assert!(!g.perm(big).on_bf && g.perm(small).on_bf);
    assert_eq!((g.player(P1).life, g.player(P2).life), (38, 38));
}

// ------------------------------------------------------------------ jodah: Mother of Runes stays home for Jodah
#[test]
fn mother_of_runes_stays_home_only_for_jodah() {
    let mut g = table(&["jodah", "sauron"]);
    let m = perm(&mut g, P0, "Mother of Runes");
    assert!(g.perm(m).noatk);
    let mut g = table(&["light-paws-aura-voltron", "sauron"]);
    let m = perm(&mut g, P0, "Mother of Runes");
    assert!(!g.perm(m).noatk);
}
