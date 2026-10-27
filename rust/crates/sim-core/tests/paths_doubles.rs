//! Phase 2, the hard doubles (`cardgen audit --paths`): cards whose compiled abilities and card code both acted on
//! one event. Each test checks that the card's effect now happens exactly once, by whichever path kept it (see
//! `data/card_overrides.toml` for the overrides and the `paths_ok` splits). Heliod's Pilgrim, Tataru Taru and Daxos
//! keep their tests in cards_t1.rs, cards_t6_legends2.rs and cards_t45_heliod_zur.rs.

use sim_core::dsl;
use sim_core::engine::{cast, combat, hooks, mana::cost_of, stack, zones};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const TERGRID: &str = "tergrid-mono-black-disruption";
const TEYSA: &str = "teysa-orzhov-aristocrats";
const KRENKO: &str = "krenko-mono-red-goblins";
const ATRAXA: &str = "atraxa-infect";

fn tbl(first: &str) -> Game {
    table(&[first, TEYSA, "marwyn-mono-green-elves", "aurelia-boros-extra-combats"])
}

fn settle(g: &mut Game) {
    stack::flush_triggers(g).unwrap();
    stack::settle_stack(g).unwrap();
}

fn rule_slot(g: &Game, name: &str, f: impl Fn(&sim_core::hooks::CardImpl) -> bool) -> bool {
    g.registry.get(card(g, name)).is_some_and(f)
}

// ======================================================== cost reducers: the compiled static:cost alone
#[test]
fn the_medallions_cut_exactly_one_from_their_colour() {
    let mut g = tbl(KRENKO);
    for (medallion, spell, other) in [
        ("Ruby Medallion", "Beetleback Chief", "Damnation"),     // {2}{R}{R}
        ("Pearl Medallion", "Wrath of God", "Beetleback Chief"), // {2}{W}{W}
        ("Jet Medallion", "Damnation", "Wrath of God"),          // {2}{B}{B}
    ] {
        let (s, o) = (card(&g, spell), card(&g, other));
        assert_eq!(cost_of(&g, P0, s).0, 2, "{spell} before {medallion}");
        let m = perm(&mut g, P0, medallion);
        assert!(!rule_slot(&g, medallion, |i| i.cost.is_some()), "{medallion}: no cost hook");
        assert_eq!(cost_of(&g, P0, s).0, 1, "{medallion}: {spell} costs exactly {{1}} less");
        assert_eq!(cost_of(&g, P0, o).0, 2, "{medallion}: another colour pays full");
        assert_eq!(cost_of(&g, P1, s).0, 2, "{medallion}: only its controller's spells");
        zones::leave(&mut g, m).unwrap();
    }
}

#[test]
fn foundry_inspector_and_etherium_sculptor_cut_one_each_from_artifacts() {
    let mut g = tbl("urza-mono-blue-artifacts");
    let wurm = card(&g, "Wurmcoil Engine"); // {6}
    let elf = card(&g, "Beetleback Chief");
    perm(&mut g, P0, "Foundry Inspector");
    assert_eq!(cost_of(&g, P0, wurm).0, 5, "one reducer: {{1}} less");
    perm(&mut g, P0, "Etherium Sculptor");
    assert_eq!(cost_of(&g, P0, wurm).0, 4, "two reducers: {{2}} less, not {{4}}");
    assert_eq!(cost_of(&g, P0, elf).0, 2, "not a nonartifact spell");
    for n in ["Foundry Inspector", "Etherium Sculptor"] {
        assert!(!rule_slot(&g, n, |i| i.cost.is_some()), "{n}: no cost hook");
    }
}

#[test]
fn starfield_mystic_cuts_one_from_enchantments() {
    let mut g = tbl("light-paws-aura-voltron");
    let ring = card(&g, "Oblivion Ring"); // {2}{W}
    perm(&mut g, P0, "Starfield Mystic");
    assert_eq!(cost_of(&g, P0, ring).0, 1, "exactly {{1}} less");
    assert_eq!(cost_of(&g, P0, card(&g, "Wrath of God")).0, 2, "not a non-enchantment");
}

// ======================================================== toxic: compiled, given once by combat damage
fn hit(g: &mut Game, m: PermId) {
    combat::resolve_combat(g, P0, &[m], P1, &[m]).unwrap();
}

#[test]
fn toxic_creatures_give_their_poison_exactly_once() {
    for (name, n) in [("Dune Mover", 1), ("Paladin of Predation", 6), ("Venomous Brutalizer", 3)] {
        let mut g = tbl(TERGRID);
        let m = perm(&mut g, P0, name);
        assert!(!rule_slot(&g, name, |i| i.combat_damage.is_some()), "{name}: no combat-damage hook");
        hit(&mut g, m);
        assert_eq!(g.player(P1).poison, n, "{name}: toxic {n} once");
    }
}

#[test]
fn ribskiff_crewed_gives_two_poison_once() {
    let mut g = tbl(TERGRID);
    let m = perm(&mut g, P0, "Ribskiff");
    let crew = token(&mut g, P0, 3);
    g.perm_mut(crew).sick = true;
    hooks::fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(g.is_creature(m));
    hit(&mut g, m);
    assert_eq!(g.player(P1).poison, 2, "toxic 2 once");
}

#[test]
fn contaminant_grafter_poisons_once_then_proliferates_once() {
    let mut g = tbl(ATRAXA);
    let m = perm(&mut g, P0, "Contaminant Grafter");
    hit(&mut g, m);
    settle(&mut g);
    assert_eq!(g.player(P1).poison, 2, "toxic 1, then the proliferate trigger adds one");
}

// ======================================================== triggers kept on one path
#[test]
fn laelia_exiles_one_card_and_grows_once_per_attack() {
    let mut g = tbl("prosper-rakdos-exile-treasure");
    let l = perm(&mut g, P0, "Laelia, the Blade Reforged");
    let (hand0, lib0) = (g.player(P0).hand.len(), g.player(P0).library.len());
    combat::attack_triggers(&mut g, P0, &[l], P1).unwrap();
    settle(&mut g);
    assert_eq!(g.player(P0).library.len(), lib0 - 1, "one card off the top");
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "the exiled card (playable this turn), no extra draw");
    assert_eq!(g.player(P0).impulse.len(), 1);
    assert_eq!(g.perm(l).plus, 1, "one +1/+1 counter");
}

#[test]
fn oath_of_teferi_blinks_one_permanent_and_exiles_nothing_for_good() {
    let mut g = tbl(ATRAXA);
    let mover = perm(&mut g, P0, "Dune Mover"); // an enter trigger worth blinking (its compiled land search)
    let tok = token(&mut g, P0, 1);
    perm(&mut g, P0, "Oath of Teferi");
    settle(&mut g);
    assert!(!g.perm(mover).on_bf, "Dune Mover is exiled until the end step");
    assert_eq!(g.player(P0).oath_return.len(), 1);
    assert!(g.perm(tok).on_bf, "nothing else of ours is exiled");
}

#[test]
fn liliana_waker_of_the_dead_offers_only_the_walker_s_abilities() {
    let mut g = tbl(TERGRID);
    let li = perm(&mut g, P0, "Liliana, Waker of the Dead");
    assert!(g.db.get(card(&g, "Liliana, Waker of the Dead")).abilities.is_empty(), "no compiled +1");
    let compiled = dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(
        !compiled.iter().any(|o| o.label.starts_with("Liliana")),
        "{:?}",
        compiled.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    let walker = sim_core::impls::common::walker_options(&mut g, li, P0, Some(false)).unwrap();
    let plus: Vec<_> = walker.iter().filter(|o| o.label.ends_with("+1")).collect();
    assert_eq!(plus.len(), 1, "one +1, the walker's: {:?}", walker.iter().map(|o| &o.label).collect::<Vec<_>>());
}

#[test]
fn static_net_exiles_once_and_gains_two_life_once() {
    let mut g = table(&["yshtola", "seph"]);
    let t = perm(&mut g, P1, "Grave Titan");
    let life = g.player(P0).life;
    perm(&mut g, P0, "Static Net");
    settle(&mut g);
    assert!(!g.perm(t).on_bf, "the compiled trigger exiles the Titan");
    assert_eq!(g.player(P0).life, life + 2, "the card code gains 2 once");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).name == "Powerstone").count(), 1);
}

#[test]
fn twilight_diviner_surveils_once_and_copies_nothing_for_itself() {
    let mut g = table(&["meren-golgari-recursion", TEYSA]);
    let (lib0, gy0) = (g.player(P0).library.len(), g.player(P0).gy.len());
    perm(&mut g, P0, "Twilight Diviner");
    settle(&mut g);
    let lib = g.player(P0).library.len();
    let moved = g.player(P0).gy.len() - gy0;
    assert_eq!(lib + moved, lib0 - 1, "no card drawn (the Diviner itself left the library)");
    assert!(moved <= 2, "surveil 2 once: {moved} cards to the graveyard");
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).token), "its own entering makes no copy");
}

#[test]
fn daxos_gains_one_life_per_creature_entering_and_dying() {
    let mut g = tbl(TEYSA);
    perm(&mut g, P0, "Daxos, Blessed by the Sun");
    let life = g.player(P0).life;
    let t = token(&mut g, P0, 1);
    settle(&mut g);
    assert_eq!(g.player(P0).life, life + 1, "another creature entered: 1 life");
    zones::die(&mut g, t, "destroy").unwrap();
    settle(&mut g);
    assert_eq!(g.player(P0).life, life + 2, "it died: 1 more");
}

#[test]
fn elemental_mascot_pumps_once_and_exiles_one_card_for_a_big_spell() {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", TEYSA]);
    let m = perm(&mut g, P0, "Elemental Mascot");
    let base = sim_core::engine::values::epow(&g, m);
    let c = hand(&mut g, P0, &["Embrace the Paradox"])[0]; // mana value 5
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    settle(&mut g);
    assert_eq!(sim_core::engine::values::epow(&g, m), base + 1, "+1/+0 once");
    assert_eq!(g.player(P0).impulse_long.len(), 1, "one card exiled");
}

// ======================================================== uncounterable: the spell itself, and the others
#[test]
fn allosaurus_shepherd_and_hexing_squelcher_cover_themselves_and_your_spells() {
    let mut g = tbl(KRENKO);
    let elf = card(&g, "Elvish Mystic");
    let bolt = card(&g, "Lightning Bolt");
    assert!(g.db.get(card(&g, "Allosaurus Shepherd")).tag(sim_core::tag::Tag::Unc), "Shepherd itself");
    assert!(g.db.get(card(&g, "Hexing Squelcher")).tag(sim_core::tag::Tag::Unc), "Squelcher itself");
    assert!(!hooks::any_uncounterable(&g, P0, elf));
    perm(&mut g, P0, "Allosaurus Shepherd");
    assert!(hooks::any_uncounterable(&g, P0, elf), "your green spells");
    assert!(!hooks::any_uncounterable(&g, P0, bolt), "not a red one");
    assert!(!hooks::any_uncounterable(&g, P1, elf), "not an opponent's");
    perm(&mut g, P0, "Hexing Squelcher");
    assert!(hooks::any_uncounterable(&g, P0, bolt), "Squelcher: every spell you control");
}

// ======================================================== Snap: the compiled spell alone
#[test]
fn snap_bounces_its_target_and_untaps_exactly_two_lands() {
    let mut g = tbl("rograkh-thrasios-creature-combo");
    assert!(!rule_slot(&g, "Snap", |i| i.resolve.is_some()), "no hand-written resolve");
    lands(&mut g, P0, "Island", 4, true);
    let t = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Snap"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    settle(&mut g);
    assert!(!g.perm(t).on_bf && g.player(P1).hand.contains(&card(&g, "Grave Titan")), "bounced to its owner");
    let untapped = g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count();
    assert_eq!(untapped, 2, "two lands untapped, not four");
    assert!(g.player(P0).gy.contains(&c));
}

// ======================================================== the shadowed spells: the hand-written resolve alone
/// These keep a compiled spell for the AI's value of the card (`paths_ok`); `cast::resolve` runs the hand-written
/// resolve instead, never both.
const SHADOWED: [&str; 18] = [
    "Borne Upon a Wind",
    "Calamity's Wake",
    "Demonic Counsel",
    "Erode",
    "Finale of Revelation",
    "Heartless Act",
    "Kaya's Wrath",
    "Liliana's Triumph",
    "Mardu Charm",
    "Molten Duplication",
    "Natural Order",
    "Open the Armory",
    "Rapid Hybridization",
    "Rite of Flame",
    "Scout's Warning",
    "Sunfall",
    "Timetwister",
    "Vona's Hunger",
];

#[test]
fn the_shadowed_spells_have_a_hand_written_resolve_over_their_compiled_spell() {
    let g = tbl(TERGRID);
    for n in SHADOWED {
        let d = g.db.get(card(&g, n));
        assert!(d.abilities.iter().any(|a| matches!(a, dsl::model::Ability::Spell { .. })), "{n}: compiled spell");
        assert!(rule_slot(&g, n, |i| i.resolve.is_some()), "{n}: hand-written resolve");
    }
}

#[test]
fn kaya_s_wrath_gains_life_once_per_creature_of_yours() {
    let mut g = tbl(TEYSA);
    for _ in 0..2 {
        token(&mut g, P0, 1);
    }
    token(&mut g, P1, 1);
    let life = g.player(P0).life;
    let c = take(&mut g, P0, "Kaya's Wrath");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    settle(&mut g);
    assert!(g.player(P0).perms.iter().chain(&g.player(P1).perms).all(|&m| !g.is_creature(m) || !g.perm(m).on_bf));
    assert_eq!(g.player(P0).life, life + 2, "2 of yours destroyed: 2 life, once");
}

#[test]
fn timetwister_gives_every_player_seven_once() {
    let mut g = tbl("rograkh-silas-turbo-breach");
    hand(&mut g, P1, &["Swamp", "Swamp"]);
    let c = take(&mut g, P0, "Timetwister");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    settle(&mut g);
    for q in 0..4 {
        assert_eq!(g.player(PlayerId(q)).hand.len(), 7, "seat {q}: seven cards, not fourteen");
    }
}
