//! Phase 2, PR 5a: the competing and shadowed doubles of `cardgen audit --paths` (a compiled activated ability beside
//! a card-code `options` / `land_options` entry; a compiled spell under a hand-written `resolve`). Each test checks
//! that the card now runs through one path and does its effect exactly once, and that the AI can still use it. The
//! overrides and the `paths_ok` splits are in `data/card_overrides.toml`. Tests the cards already had (Slayers'
//! Stronghold and Sunhome: cards_t3_aurelia.rs; Vault of the Archangel and Silent Clearing: cards_lands.rs; Karn's
//! Bastion: cards_common2.rs; Buried Ruin: cards_t45_heliod_zur.rs; Mount Doom: cards_t6_lands.rs; Dragon's Hoard:
//! cards_t45_urdragon.rs; Staff of Compleation's destroy: cards_t6_dack.rs; Demon's cast for life:
//! cards_t0_anikthea.rs; Damn's target mode: cards_t45_edgar.rs) still run the card code they test. The last section is
//! the nine split lines PR 5b found to be doubles (an enters tag beside card code; Doubling Season's loyalty).

use sim_core::ai::act::perform;
use sim_core::cardcode;
use sim_core::dsl::{self, model::Ability};
use sim_core::engine::{cast, stack};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn tbl(first: &str) -> Game {
    table(&[first, "teysa-orzhov-aristocrats", "marwyn-mono-green-elves", "aurelia-boros-extra-combats"])
}

fn go(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn land_opts(g: &mut Game, p: PlayerId, post: Option<bool>, prefix: &str) -> Vec<Opt> {
    cardcode::land_options(g, p, post).unwrap().into_iter().filter(|o| o.label.starts_with(prefix)).collect()
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("an options slot");
    f(g, m, p, post).unwrap()
}

/// the compiled activations dsl::ability_options offers for permanent m
fn compiled_opts(g: &mut Game, p: PlayerId, m: PermId) -> Vec<Opt> {
    dsl::ability_options(g, p, true)
        .unwrap()
        .into_iter()
        .filter(|o| matches!(o.act, Some(Action::Dsl { src, .. }) if src == m))
        .collect()
}

fn has_compiled(g: &Game, name: &str) -> bool {
    !g.db.get(card(g, name)).abilities.is_empty()
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

// ======================================================== the competing lands: land_options alone
/// Lands aren't permanents in `perms`, so `dsl::ability_options` never offered their compiled activated ability:
/// their card code was the only path. The compiled ability is gone (`dsl = []`); the land code stays.
const LANDS: [&str; 15] = [
    "Boiling Rock Prison",
    "Buried Ruin",
    "Castle Embereth",
    "Castle Locthwain",
    "Ghost Quarter",
    "Horizon Canopy",
    "Karn's Bastion",
    "Mount Doom",
    "Silent Clearing",
    "Slayers' Stronghold",
    "Sunbaked Canyon",
    "Sunhome, Fortress of the Legion",
    "Tectonic Edge",
    "Vault of the Archangel",
    "Waterlogged Grove",
];

#[test]
fn the_competing_lands_keep_only_their_land_code() {
    let mut g = tbl("aurelia-boros-extra-combats");
    for n in LANDS {
        assert!(!has_compiled(&g, n), "{n}: no compiled ability");
        assert!(g.registry.get(card(&g, n)).is_some_and(|i| i.land_options.is_some()), "{n}: land_options");
        lands(&mut g, P0, n, 1, false);
    }
    lands(&mut g, P0, "Plateau", 8, false);
    let offered = dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(offered.iter().all(|o| !matches!(o.act, Some(Action::Dsl { .. }))), "no compiled land activation");
}

#[test]
fn ghost_quarter_destroys_the_land_and_its_controller_finds_one_basic_once() {
    let mut g = tbl("alela");
    let gq = lands(&mut g, P0, "Ghost Quarter", 1, false)[0];
    let cradle = lands(&mut g, P1, "Gaea's Cradle", 1, false)[0];
    lands(&mut g, P1, "Forest", 2, false);
    set_library(&mut g, P1, &["Forest", "Forest", "Llanowar Elves"]);
    let o = land_opts(&mut g, P0, Some(false), "Ghost Quarter");
    assert_eq!(o.len(), 1, "aimed at the Cradle");
    assert!(o[0].label.ends_with("Gaea's Cradle"));
    let (n1, lib1) = (g.player(P1).lands.len(), g.player(P1).library.len());
    assert!(go(&mut g, P0, &o[0]));
    assert!(!g.player(P1).lands.contains(&cradle), "the Cradle is destroyed");
    assert!(g.player(P1).gy.contains(&card(&g, "Gaea's Cradle")));
    assert_eq!(g.player(P1).lands.len(), n1, "one land lost, one basic found");
    assert_eq!(g.player(P1).library.len(), lib1 - 1, "exactly one basic from the library");
    assert!(!g.player(P0).lands.contains(&gq) && g.player(P0).gy.contains(&card(&g, "Ghost Quarter")), "sacrificed");
    assert!(land_opts(&mut g, P0, Some(false), "Ghost Quarter").is_empty(), "once: it is gone");
}

#[test]
fn tectonic_edge_needs_four_opposing_lands_and_finds_no_basic() {
    let mut g = tbl("teysa-orzhov-aristocrats");
    lands(&mut g, P0, "Tectonic Edge", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let coffers = lands(&mut g, P1, "Cabal Coffers", 1, false)[0];
    lands(&mut g, P1, "Swamp", 2, false);
    assert!(land_opts(&mut g, P0, Some(false), "Tectonic Edge").is_empty(), "three lands: not yet");
    lands(&mut g, P1, "Swamp", 1, false);
    set_library(&mut g, P1, &["Swamp", "Swamp"]); // after `lands`, which takes its cards from the library
    let o = land_opts(&mut g, P0, Some(false), "Tectonic Edge");
    assert_eq!(o.len(), 1);
    assert!(go(&mut g, P0, &o[0]));
    assert!(!g.player(P1).lands.contains(&coffers));
    assert_eq!((g.player(P1).lands.len(), g.player(P1).library.len()), (3, 2), "no basic for its controller");
}

#[test]
fn the_sacrifice_lands_draw_exactly_one_card() {
    for n in ["Horizon Canopy", "Silent Clearing", "Sunbaked Canyon", "Waterlogged Grove", "Boiling Rock Prison"] {
        let mut g = tbl("aurelia-boros-extra-combats");
        let l = lands(&mut g, P0, n, 1, false)[0];
        lands(&mut g, P0, "Plateau", 7, false);
        let o = land_opts(&mut g, P0, None, n);
        assert_eq!(o.len(), 1, "{n}: offered at the end of the turn, flooded");
        let (hand, lib) = (g.player(P0).hand.len(), g.player(P0).library.len());
        assert!(go(&mut g, P0, &o[0]));
        assert_eq!(g.player(P0).hand.len(), hand + 1, "{n}: one card");
        assert_eq!(g.player(P0).library.len(), lib - 1, "{n}: one draw");
        assert!(!g.player(P0).lands.contains(&l) && g.player(P0).gy.contains(&card(&g, n)), "{n}: sacrificed");
    }
}

#[test]
fn castle_locthwain_draws_once_and_costs_the_cards_in_hand() {
    let mut g = tbl(TERGRID);
    let l = lands(&mut g, P0, "Castle Locthwain", 1, false)[0];
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Swamp"]);
    let life = g.player(P0).life;
    let o = land_opts(&mut g, P0, None, "Castle Locthwain");
    assert_eq!(o.len(), 1);
    assert!(go(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).life, life - 2, "two cards in hand after the draw");
    assert!(g.land(l).tapped);
    assert!(land_opts(&mut g, P0, None, "Castle Locthwain").is_empty(), "tapped: once");
}

const TERGRID: &str = "tergrid-mono-black-disruption";

#[test]
fn castle_embereth_pumps_the_team_once() {
    let mut g = tbl("krenko-mono-red-goblins");
    lands(&mut g, P0, "Castle Embereth", 1, false);
    lands(&mut g, P0, "Mountain", 3, false);
    let ts: Vec<PermId> = (0..4).map(|_| token(&mut g, P0, 1)).collect();
    let o = land_opts(&mut g, P0, Some(false), "Castle Embereth");
    assert_eq!(o.len(), 1, "four ready attackers");
    assert!(go(&mut g, P0, &o[0]));
    assert!(ts.iter().all(|&m| g.perm(m).eot_pt == (1, 0)), "+1/+0 each, once");
    assert!(land_opts(&mut g, P0, Some(false), "Castle Embereth").is_empty());
}

// ======================================================== the competing nonland cards
#[test]
fn dragons_hoard_has_no_compiled_draw_and_draws_once_per_counter() {
    let mut g = tbl("miirym-temur-dragons");
    assert!(!has_compiled(&g, "Dragon's Hoard"), "the compiled draw's gold-counter cost was never payable");
    let h = perm(&mut g, P0, "Dragon's Hoard");
    g.perm_mut(h).data.set(DataKey::T45Dragon, Val::Int(1));
    assert!(compiled_opts(&mut g, P0, h).is_empty());
    let o = perm_opts(&mut g, h, P0, None);
    assert_eq!(o.len(), 1);
    let hand = g.player(P0).hand.len();
    assert!(go(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    g.perm_mut(h).tapped = false;
    assert!(perm_opts(&mut g, h, P0, None).is_empty(), "no counter left");
}

#[test]
fn weathered_wayfarer_searches_once_and_only_when_behind_on_lands() {
    let mut g = tbl("teysa-orzhov-aristocrats");
    assert!(!has_compiled(&g, "Weathered Wayfarer"), "the compiled search had no activation condition");
    let w = perm(&mut g, P0, "Weathered Wayfarer");
    g.perm_mut(w).sick = false;
    lands(&mut g, P0, "Plains", 2, false);
    lands(&mut g, P1, "Swamp", 2, false);
    set_library(&mut g, P0, &["Scrubland", "Llanowar Elves"]);
    assert!(compiled_opts(&mut g, P0, w).is_empty());
    assert!(perm_opts(&mut g, w, P0, Some(false)).is_empty(), "no opponent has more lands");
    lands(&mut g, P1, "Swamp", 1, false);
    let o = perm_opts(&mut g, w, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(go(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand, vec![card(&g, "Scrubland")], "one land, any land");
    assert!(g.perm(w).tapped);
    assert!(perm_opts(&mut g, w, P0, Some(false)).is_empty(), "tapped: once");
}

#[test]
fn gingerbrute_s_two_paths_are_two_abilities() {
    let mut g = tbl("yuriko-dimir-ninjas");
    g.active = Some(P0);
    g.step = Step::Main1;
    lands(&mut g, P0, "Island", 3, false);
    let gb = perm(&mut g, P0, "Gingerbrute");
    token(&mut g, P1, 2);
    let life = g.player(P0).life;
    let ev = perm_opts(&mut g, gb, P0, Some(false));
    assert_eq!(ev.len(), 1, "the {{1}} evasion (card code)");
    assert!(go(&mut g, P0, &ev[0]));
    assert_eq!(g.perm(gb).eot_kw.iter().filter(|&&k| k == "haste_blockers").count(), 1);
    assert_eq!(g.player(P0).life, life, "the evasion gains no life");
    let sac = compiled_opts(&mut g, P0, gb);
    assert_eq!(sac.len(), 1, "the compiled {{2}}, {{T}}, sacrifice: gain 3 life");
    assert!(go(&mut g, P0, &sac[0]));
    assert_eq!(g.player(P0).life, life + 3, "3 life, once");
    assert!(!g.perm(gb).on_bf);
}

#[test]
fn demon_of_fates_design_pumps_by_the_sacrificed_enchantment_once() {
    let mut g = tbl("anikthea-enduring-enchantments");
    g.active = Some(P0);
    lands(&mut g, P0, "Swamp", 3, false);
    let d = perm(&mut g, P0, "Demon of Fate's Design");
    let e = perm(&mut g, P0, "Sigil of the Empty Throne"); // mana value 5
    let life = g.player(P0).life;
    let o = compiled_opts(&mut g, P0, d);
    assert_eq!(o.len(), 1, "the compiled {{2}}{{B}} sacrifice");
    assert!(go(&mut g, P0, &o[0]));
    assert!(g.perm(d).on_bf && !g.perm(e).on_bf, "another enchantment is sacrificed, not the Demon");
    assert_eq!(g.perm(d).eot_pt.0, 5, "+X/+0 with X = 5, once");
    assert_eq!(g.player(P0).life, life, "the pump costs no life (the life cast is the card code's)");
}

#[test]
fn staff_of_domination_activates_through_the_compiled_abilities_and_shortcuts_only_with_infinite_mana() {
    let mut g = tbl("karn");
    lands(&mut g, P0, "Wastes", 5, false);
    let s = perm(&mut g, P0, "Staff of Domination");
    assert!(perm_opts(&mut g, s, P0, Some(false)).is_empty(), "no infinite mana: no shortcut");
    let o = compiled_opts(&mut g, P0, s);
    let draw = o.iter().find(|o| matches!(o.act, Some(Action::Dsl { idx: 4, .. }))).expect("{5}, {T}: draw");
    let hand = g.player(P0).hand.len();
    assert!(go(&mut g, P0, draw));
    assert_eq!(g.player(P0).hand.len(), hand + 1, "one card");
    assert!(g.perm(s).tapped);
}

#[test]
fn staff_of_compleation_draws_once_through_its_compiled_ability() {
    let mut g = tbl("dack-fayden-white-stax");
    let s = perm(&mut g, P0, "Staff of Compleation");
    let life = g.player(P0).life;
    let o = compiled_opts(&mut g, P0, s);
    let draw = o.iter().find(|o| matches!(o.act, Some(Action::Dsl { idx: 1, .. }))).expect("{T}, pay 4 life: draw");
    let hand = g.player(P0).hand.len();
    assert!(go(&mut g, P0, draw));
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (hand + 1, life - 4));
    assert!(perm_opts(&mut g, s, P0, Some(true)).is_empty(), "tapped: the destroy waits");
}

// ======================================================== the shadowed spells
/// The hand-written resolve does each card; the compiled spell stays as the AI's value of the card (`paths_ok`).
const SHADOWED: [&str; 24] = [
    "Damn",
    "Drown in Ichor",
    "Eldrazi Confluence",
    "Elemental Eruption",
    "Emerald Charm",
    "Entish Restoration",
    "Flare of Malice",
    "Galadhrim Ambush",
    "Get Lost",
    "Heroic Return",
    "Ignite the Future",
    "Infectious Bite",
    "Kozilek's Command",
    "Lock and Load",
    "Malakir Rebirth // Malakir Mire",
    "Mnemonic Betrayal",
    "Phyresis Outbreak",
    "Pox",
    "Pyretic Charge",
    "Return Upon the Tide",
    "Soul Partition",
    "Sundering Eruption // Volcanic Fissure",
    "Time Wipe",
    "Untimely Malfunction",
];

#[test]
fn the_shadowed_spells_keep_their_resolve_and_a_compiled_value() {
    let g = tbl(TERGRID);
    for n in SHADOWED {
        let d = g.db.get(card(&g, n));
        assert!(d.abilities.iter().any(|a| matches!(a, Ability::Spell { .. })), "{n}: compiled spell");
        assert!(g.registry.get(card(&g, n)).is_some_and(|i| i.resolve.is_some()), "{n}: hand-written resolve");
    }
}

#[test]
fn damn_overloaded_is_a_wrath_and_targeted_destroys_one() {
    let mut g = tbl("teysa-orzhov-aristocrats");
    let a = token(&mut g, P1, 3);
    let b = token(&mut g, P2, 3);
    let c = take(&mut g, P0, "Damn");
    cast::resolve(&mut g, P0, c, &Ctx { target: Some(a), ..Ctx::default() }, "hand").unwrap();
    assert!(!g.perm(a).on_bf && g.perm(b).on_bf, "not overloaded: the compiled destroy, one creature");
    let mut g = tbl("teysa-orzhov-aristocrats");
    let ms = [token(&mut g, P0, 1), token(&mut g, P1, 3), token(&mut g, P2, 5)];
    let c = take(&mut g, P0, "Damn");
    cast::resolve(&mut g, P0, c, &Ctx { overload: true, ..Ctx::default() }, "hand").unwrap();
    stack::flush_triggers(&mut g).unwrap();
    assert!(ms.iter().all(|&m| !g.perm(m).on_bf), "overloaded: each creature");
    assert!(g.player(P0).gy.contains(&c), "Damn to the graveyard, once");
}

#[test]
fn desperate_ritual_adds_exactly_three_red() {
    let mut g = tbl("krenko-mono-red-goblins");
    assert!(!has_compiled(&g, "Desperate Ritual"), "the compiled spell added any colour");
    let (r, any) = (g.player(P0).floating.r, g.player(P0).floating.any);
    let c = take(&mut g, P0, "Desperate Ritual");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!((g.player(P0).floating.r, g.player(P0).floating.any), (r + 3, any), "{{R}}{{R}}{{R}}, once");
}

// ======================================================== split lines that were doubles (found by the splits pass)
/// p's tokens on the battlefield
fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

#[test]
fn the_dies_token_makers_make_nothing_as_they_enter_and_one_token_per_death() {
    for n in ["Requiem Angel", "Sifter of Skulls", "Pawn of Ulamog"] {
        let mut g = tbl("teysa-orzhov-aristocrats");
        perm(&mut g, P0, n);
        assert!(tokens(&g, P0).is_empty(), "{n}: no token as it enters (the tok tag is gone)");
        let bear = perm(&mut g, P0, "Llanowar Elves");
        sim_core::engine::zones::die(&mut g, bear, "destroy").unwrap();
        stack::flush_triggers(&mut g).unwrap();
        stack::settle_stack(&mut g).unwrap();
        assert_eq!(tokens(&g, P0).len(), 1, "{n}: one token for the creature that died");
    }
}

#[test]
fn springheart_nantuko_makes_its_insect_on_landfall_only() {
    let mut g = tbl("rograkh-thrasios-creature-combo");
    let m = perm(&mut g, P0, "Springheart Nantuko");
    assert!(tokens(&g, P0).is_empty(), "no 1/1 as it enters");
    let f = g.registry.get(card(&g, "Springheart Nantuko")).and_then(|i| i.landfall).expect("landfall");
    f(&mut g, m, P0).unwrap();
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(tokens(&g, P0).len(), 1, "one Insect per land");
}

#[test]
fn battle_angels_of_tyr_makes_a_treasure_only_on_combat_damage() {
    let mut g = tbl("aurelia-boros-extra-combats");
    let m = perm(&mut g, P0, "Battle Angels of Tyr");
    assert!(tokens(&g, P0).is_empty(), "no Treasure as it enters (the Scryfall treas tag is gone)");
    lands(&mut g, P1, "Plains", 3, false);
    let f = g.registry.get(card(&g, "Battle Angels of Tyr")).and_then(|i| i.combat_damage).expect("combat_damage");
    f(&mut g, m, P0, m, P1, 4).unwrap();
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(tokens(&g, P0).len(), 1, "one Treasure: that player has the most lands");
}

#[test]
fn karn_scion_of_urza_enters_without_a_construct_and_doubling_season_doubles_its_loyalty_once() {
    let mut g = tbl("karn");
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    assert!(tokens(&g, P0).is_empty(), "no 0/0 Construct as it enters (the -2 makes it)");
    assert_eq!(g.perm(k).loyalty, Some(5));
    let mut g = tbl("atraxa-infect");
    perm(&mut g, P0, "Doubling Season");
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    assert_eq!(g.perm(k).loyalty, Some(10), "5 doubled once, not 20");
}

#[test]
fn sakura_tribe_elder_finds_one_basic_when_sacrificed_and_none_as_it_resolves() {
    let mut g = tbl("tatyova-simic-landfall");
    set_library(&mut g, P0, &["Forest", "Forest", "Island"]);
    let c = take(&mut g, P0, "Sakura-Tribe Elder");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert!(g.player(P0).lands.is_empty(), "no land as it resolves (the lr tag is gone)");
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    let o = perm_opts(&mut g, m, P0, None);
    assert_eq!(o.len(), 1, "sacrificed at the end of an opponent's turn");
    assert!(go(&mut g, P0, &o[0]));
    assert!(!g.perm(m).on_bf);
    assert_eq!(g.player(P0).lands.len(), 1, "one basic land");
    assert_eq!(g.player(P0).library.len(), 2);
}

#[test]
fn sowing_mycospawn_puts_one_land_onto_the_battlefield() {
    let mut g = tbl("rograkh-thrasios-creature-combo");
    lands(&mut g, P0, "Forest", 4, false);
    set_library(&mut g, P0, &["Forest", "Island", "Mountain", "Plains"]);
    let c = hand(&mut g, P0, &["Sowing Mycospawn"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    stack::flush_triggers(&mut g).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(c) && g.perm(m).on_bf), "it resolved");
    assert_eq!(g.player(P0).lands.len(), 5, "the cast trigger's land, and no second one from the lr tag");
    assert_eq!(g.player(P0).library.len(), 3);
}
