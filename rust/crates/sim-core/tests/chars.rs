//! Phase 3, the characteristics pass (`engine::chars`): `chars(g, m)` agrees with every old reader (`is_creature`,
//! `epow`, `etgh`, `has_type`, `subtypes`, `colors_of`, `has_kw`) on tricky boards, with the numbers worked out by
//! hand: anthems + counters + equipment, Elesh Norn, Maskwood Nexus, an animated land, a manland, a Blood Moon board
//! and a copy. Seat 0 plays the sauron deck, seat 1 the seph deck.

use sim_core::cards::{Colors, Types};
use sim_core::dsl;
use sim_core::engine::chars::{self, Chars, KEYWORDS};
use sim_core::engine::{mana, values, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{CopyExc, DataKey, Game, Val};
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sauron", "seph"])
}

/// the characteristics as the old readers give them
fn old(g: &Game, m: PermId) -> Chars {
    let creature = g.is_creature(m);
    Chars {
        types: values::ptypes(g, m),
        creature,
        subtypes: values::subtypes(g, m),
        all_creature_types: creature && values::has_type(g, m, "zzz-no-such-type"),
        colors: values::colors_of(g, m),
        abilities_on: !g.perm(m).neutered,
        keywords: KEYWORDS.iter().copied().filter(|k| dsl::has_kw(g, m, k)).map(intern).collect(),
        power: values::epow(g, m),
        toughness: values::etgh(g, m),
    }
}

/// `chars` agrees with the old readers on every permanent and land of the table
fn agree(g: &Game) {
    for q in &g.players {
        for &m in q.perms.iter().chain(q.lands.iter()) {
            assert_eq!(chars::chars(g, m), old(g, m), "{}", g.perm(m).name);
        }
    }
}

fn kws(g: &Game, m: PermId) -> Vec<&'static str> {
    chars::chars(g, m).keywords.iter().map(|s| -> &'static str { s }).collect()
}

fn attach(g: &mut Game, e: PermId, m: PermId) {
    g.perm_mut(e).attached = Some(m);
}

#[test]
fn anthems_counters_and_equipment_add_up() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves"); // 1/1 Elf
    g.perm_mut(elves).plus = 2; // two +1/+1 counters
    let druid = perm(&mut g, P0, "Elvish Archdruid"); // other Elves +1/+1 (compiled)
    perm(&mut g, P0, "Thornfist Striker"); // +1/+1 (tagged)
    perm(&mut g, P0, "Always Watching"); // nontoken creatures +1/+1 and vigilance (compiled)
    let sword = perm(&mut g, P0, "Sword of Feast and Famine"); // +2/+2, protection from black and green
    attach(&mut g, sword, elves);
    let c = chars::chars(&g, elves);
    // Always Watching's anthem is compiled with the filter subtype "nontoken" (a parse slip): it misses (Stage C)
    assert_eq!((c.power, c.toughness), (7, 7), "1 + 2 counters + 1 + 1 + 2");
    assert!(c.creature && c.types.has(Types::CREATURE));
    assert_eq!(kws(&g, elves), ["vigilance"]);
    assert_eq!(c.colors, Colors::from_letters("G"));
    // the Archdruid gets the other anthems but not its own
    assert_eq!((chars::power(&g, druid), chars::toughness(&g, druid)), (3, 3));
    // the Sword is not a creature: no anthem, no power from the board
    assert!(!chars::is_creature(&g, sword));
    agree(&g);
}

#[test]
fn elesh_norn_shrinks_the_other_side_and_grows_her_own() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Llanowar Elves");
    let theirs = token(&mut g, P1, 2); // a 2/2
    perm(&mut g, P1, "Elesh Norn, Grand Cenobite");
    assert_eq!((chars::power(&g, mine), chars::toughness(&g, mine)), (0, -1), "power floors at 0, toughness not");
    assert_eq!((chars::power(&g, theirs), chars::toughness(&g, theirs)), (4, 4));
    agree(&g);
}

#[test]
fn maskwood_nexus_makes_creatures_every_type_but_not_noncreature_types() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let tok = token(&mut g, P0, 1);
    assert!(!chars::has_subtype(&g, elves, "dragon"));
    let nexus = perm(&mut g, P0, "Maskwood Nexus");
    for m in [elves, tok] {
        assert!(chars::has_subtype(&g, m, "dragon") && chars::has_subtype(&g, m, "rebel"));
        assert!(!chars::has_subtype(&g, m, "equipment"));
        assert!(chars::chars(&g, m).all_creature_types);
    }
    assert!(!chars::has_subtype(&g, nexus, "dragon"), "the Nexus itself is an artifact");
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    assert!(!chars::has_subtype(&g, theirs, "dragon"), "only its controller's creatures");
    agree(&g);
}

#[test]
fn an_animated_land_is_a_creature_and_gets_the_anthems() {
    let mut g = game();
    let l = lands(&mut g, P0, "Forest", 1, false)[0];
    perm(&mut g, P0, "Thornfist Striker");
    assert!(!chars::is_creature(&g, l));
    assert_eq!(chars::power(&g, l), 0, "no anthem for a land that is not a creature");
    g.perm_mut(l).data.set(DataKey::Anim, Val::Bool(true));
    (g.perm_mut(l).pow, g.perm_mut(l).tgh) = (3, 3);
    let c = chars::chars(&g, l);
    assert!(c.creature && c.types.has(Types::LAND));
    assert_eq!((c.power, c.toughness), (4, 4), "a 3/3 land with Thornfist's +1/+1");
    agree(&g);
}

#[test]
fn a_manland_animates_as_a_creature() {
    let mut g = game();
    lands(&mut g, P0, "Mutavault", 1, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let o = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let vault = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == "Mutavault").unwrap();
    let c = chars::chars(&g, vault);
    assert!(c.creature);
    assert_eq!((c.power, c.toughness), (2, 2));
    agree(&g);
}

#[test]
fn under_blood_moon_a_nonbasic_land_has_no_abilities() {
    let mut g = game();
    let pool = lands(&mut g, P0, "Breeding Pool", 1, false)[0];
    let forest = lands(&mut g, P0, "Forest", 1, false)[0];
    assert!(!mana::moon_land(&g, pool));
    perm(&mut g, P1, "Blood Moon");
    assert!(mana::moon_land(&g, pool) && !mana::moon_land(&g, forest));
    let c = chars::chars(&g, pool);
    assert!(c.types.has(Types::LAND) && !c.creature);
    assert!(c.keywords.is_empty() && c.colors.is_empty());
    agree(&g);
}

#[test]
fn a_copy_keeps_its_exceptions() {
    let mut g = game();
    let c = take(&mut g, P0, "Llanowar Elves");
    let exc = CopyExc {
        add_types: Types::ARTIFACT,
        only_subtype: Some(intern("illusion")),
        colors: Some(Colors::from_letters("U")),
        ..CopyExc::default()
    };
    let m = zones::enter(&mut g, P0, c, zones::Enter { sick: false, exc: Some(exc), ..Default::default() }).unwrap();
    let ch = chars::chars(&g, m);
    assert!(ch.types.has(Types::ARTIFACT) && ch.types.has(Types::CREATURE));
    assert_eq!(ch.subtypes, ["illusion"]);
    assert!(!chars::has_subtype(&g, m, "elf"));
    assert_eq!(ch.colors, Colors::from_letters("U"));
    agree(&g);
}

#[test]
fn a_neutered_permanent_has_its_abilities_off() {
    let mut g = game();
    let angel = perm(&mut g, P0, "Serra Angel");
    assert_eq!(kws(&g, angel), ["flying", "vigilance"]);
    g.perm_mut(angel).neutered = true; // Darksteel Mutation, Tishana's Tidebinder ...
    let c = chars::chars(&g, angel);
    assert!(!c.abilities_on && c.keywords.is_empty());
    agree(&g);
}

// ------------------------------------------------------------------ Stage B: the table of named effects
#[test]
fn the_named_effects_are_one_table() {
    use chars::{NAMED, NamedFx, named_fx};
    assert_eq!(named_fx("Maskwood Nexus"), Some(NamedFx::AllCreatureTypes));
    assert_eq!(named_fx("Colossus Hammer"), Some(NamedFx::EquippedLoses("flying")));
    assert_eq!(named_fx("Llanowar Elves"), None);
    for n in NAMED {
        assert!(sim_core::testkit::db().id(n.name).is_some(), "{} is in cards.json", n.name);
    }
}

#[test]
fn colossus_hammer_takes_flying_and_wrecking_ball_arm_sets_base_pt() {
    let mut g = game();
    let angel = perm(&mut g, P0, "Serra Angel");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let hammer = perm(&mut g, P0, "Colossus Hammer");
    let arm = perm(&mut g, P0, "Wrecking Ball Arm");
    attach(&mut g, hammer, angel);
    attach(&mut g, arm, elves);
    assert_eq!(kws(&g, angel), ["vigilance"], "the Hammer takes flying");
    assert!(sim_core::impls::cloud::loses_flying(&g, angel));
    assert_eq!((chars::power(&g, elves), chars::toughness(&g, elves)), (7, 7), "base 7/7");
    g.perm_mut(elves).plus = 1;
    assert_eq!(chars::power(&g, elves), 8, "counters apply on top of the base");
    agree(&g);
}

#[test]
fn yavimaya_and_urborg_make_every_land_a_forest_and_a_swamp() {
    let mut g = game();
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    assert!(!sim_core::impls::t3_marwyn::is_forest(&g, island));
    assert!(!chars::lands_are(&g, 'G', true) && !chars::lands_are(&g, 'B', false));
    lands(&mut g, P1, "Yavimaya, Cradle of Growth", 1, false);
    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
    assert!(sim_core::impls::t3_marwyn::is_forest(&g, island), "any player's Yavimaya");
    assert!(chars::lands_are(&g, 'G', true) && chars::lands_are(&g, 'B', false));
    agree(&g);
}
