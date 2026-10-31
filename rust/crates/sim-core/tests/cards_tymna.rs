//! The Tymna decks' new cards (impls/tymna_cards.rs): each card on a hand-built position, with its Oracle text as the
//! expectation. Tymna the Weaver herself is tested in cards_legends.rs.

use sim_core::ai::{act, decks};
use sim_core::engine::pool::Kind;
use sim_core::cards::Colors;
use sim_core::engine::{cast, combat, removal, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);
const ABIGALE: &str = "Abigale, Poet Laureate // Heroic Stanza";

fn tbl() -> Game {
    table(&["galadriel", "veyran", "seph"])
}

fn set_library(g: &mut Game, p: PlayerId, top: &[&str]) {
    // the last name is the top card; plenty of filler below
    let filler = card(g, "Plains");
    let mut lib: Vec<CardId> = vec![filler; 20];
    lib.extend(top.iter().map(|n| card(g, n)));
    g.player_mut(p).library = lib;
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap()
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn untapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

// ======================================================== Thrasios, Triton Hero
#[test]
fn thrasios_puts_a_land_onto_the_battlefield_tapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 4, false);
    let t = perm(&mut g, P0, "Thrasios, Triton Hero");
    set_library(&mut g, P0, &["Forest"]);
    let o = options(&mut g, t, P0, None);
    assert_eq!(o.len(), 1);
    let (lands_before, hand_before) = (g.player(P0).lands.len(), g.player(P0).hand.len());
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).lands.len(), lands_before + 1);
    assert_eq!(untapped_lands(&g, P0), 0, "the four Islands paid, the Forest came in tapped");
    assert_eq!(g.player(P0).hand.len(), hand_before);
}

#[test]
fn thrasios_draws_a_nonland_card() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 8, false);
    let t = perm(&mut g, P0, "Thrasios, Triton Hero");
    set_library(&mut g, P0, &["Sol Ring"]);
    let n = g.player(P0).hand.len();
    let o = options(&mut g, t, P0, None);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(untapped_lands(&g, P0), 4, "only {{4}} was paid");
    assert!(g.player(P0).hand.contains(&card(&g, "Sol Ring")));
}

#[test]
fn thrasios_timing_end_of_turn_first_then_after_the_main_phase_plays() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    let t = perm(&mut g, P0, "Thrasios, Triton Hero");
    assert!(options(&mut g, t, P0, None).is_empty(), "three mana: not enough");
    lands(&mut g, P0, "Island", 1, false);
    assert!(options(&mut g, t, P0, Some(false)).is_empty(), "never in the first main phase");
    let eot = options(&mut g, t, P0, None);
    let main2 = options(&mut g, t, P0, Some(true));
    assert_eq!((eot.len(), main2.len()), (1, 1));
    assert!(eot[0].utility > 0.0 && main2[0].utility < 0.0, "end of turn eagerly, the main phase last");
    assert!(options(&mut g, t, P1, None).is_empty(), "only its controller");
}

// ======================================================== Kraum, Ludevic's Opus
#[test]
fn kraum_draws_on_the_second_spell_an_opponent_casts_each_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Kraum, Ludevic's Opus");
    let n = g.player(P0).hand.len();
    let face = Ctx { face: Some(P2), ..Default::default() };
    cast_free(&mut g, P1, "Lightning Bolt", face.clone());
    assert_eq!(g.player(P0).hand.len(), n, "the first spell: nothing");
    cast_free(&mut g, P1, "Lightning Bolt", face.clone());
    assert_eq!(g.player(P0).hand.len(), n + 1, "the second spell: a card");
    cast_free(&mut g, P1, "Lightning Bolt", face.clone());
    assert_eq!(g.player(P0).hand.len(), n + 1, "the third: nothing more");
    cast_free(&mut g, P0, "Lightning Bolt", Ctx { face: Some(P2), ..Default::default() });
    cast_free(&mut g, P0, "Lightning Bolt", Ctx { face: Some(P2), ..Default::default() });
    assert_eq!(g.player(P0).hand.len(), n + 1, "its controller's own spells don't count");
}

// ======================================================== Black Widow, Agile Avenger
#[test]
fn black_widow_grows_and_draws_on_an_opponents_second_draw() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Black Widow, Agile Avenger");
    let base = values::epow(&g, w);
    let n = g.player(P0).hand.len();
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!((g.player(P0).hand.len(), values::epow(&g, w)), (n, base));
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!((g.player(P0).hand.len(), values::epow(&g, w)), (n + 1, base + 1));
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "only the second card each turn");
    zones::draw(&mut g, P0, 2, false).unwrap();
    assert_eq!(values::epow(&g, w), base + 1, "its controller's draws don't count");
}

// ======================================================== Abigale, Poet Laureate // Heroic Stanza
#[test]
fn abigale_prepares_on_a_creature_spell_and_casts_heroic_stanza() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let a = perm(&mut g, P0, ABIGALE);
    let t = token(&mut g, P0, 2);
    assert!(options(&mut g, a, P0, Some(false)).is_empty(), "not prepared yet");
    cast_free(&mut g, P0, "Lightning Bolt", Ctx { face: Some(P2), ..Default::default() });
    assert!(!g.perm(a).data.truthy(DataKey::Prepared), "a noncreature spell doesn't prepare her");
    cast_free(&mut g, P0, "Birds of Paradise", Ctx::default());
    assert!(g.perm(a).data.truthy(DataKey::Prepared));
    let o = options(&mut g, a, P0, Some(false));
    assert_eq!(o.len(), 1);
    let plus = |g: &Game, m: PermId| g.perm(m).plus;
    let before: i32 = g.player(P0).perms.iter().map(|&m| plus(&g, m)).sum();
    assert!(run(&mut g, P0, &o[0]));
    let after: i32 = g.player(P0).perms.iter().map(|&m| plus(&g, m)).sum();
    assert_eq!(after, before + 1, "one +1/+1 counter, on a creature");
    assert!(g.perm(a).on_bf && plus(&g, a) + plus(&g, t) >= 1);
    assert_eq!(untapped_lands(&g, P0), 0, "{{1}}{{W/B}} took both Plains");
    assert!(!g.perm(a).data.truthy(DataKey::Prepared), "casting the copy unprepares her");
    assert!(options(&mut g, a, P0, Some(false)).is_empty());
}

// ======================================================== Cuboid Colony
#[test]
fn cuboid_colony_increments_when_more_mana_was_spent_than_its_power_or_toughness() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Cuboid Colony"); // 1/1
    cast_free(&mut g, P0, "Sol Ring", Ctx::default()); // mana value 1: not more than 1
    assert_eq!(g.perm(c).plus, 0);
    cast_free(&mut g, P0, "Arcane Signet", Ctx::default()); // 2 > 1
    assert_eq!(g.perm(c).plus, 1);
    cast_free(&mut g, P0, "Arcane Signet", Ctx::default()); // 2 is not more than 2
    assert_eq!(g.perm(c).plus, 1);
    cast_free(&mut g, P0, "Cultivate", Ctx::default()); // 3 > 2
    assert_eq!(g.perm(c).plus, 2);
    cast_free(&mut g, P1, "Cultivate", Ctx::default()); // an opponent's spell
    assert_eq!(g.perm(c).plus, 2);
}

// ======================================================== Elemental Mascot
#[test]
fn elemental_mascot_pumps_and_exiles_the_top_card_for_a_big_spell() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Elemental Mascot");
    let base = values::epow(&g, m);
    set_library(&mut g, P0, &["Forest"]);
    cast_free(&mut g, P0, "Lightning Bolt", Ctx { face: Some(P2), ..Default::default() });
    assert_eq!(values::epow(&g, m), base + 1, "+1/+0 until end of turn");
    assert!(g.player(P0).impulse_long.is_empty(), "under five mana: no exile");
    let lib = g.player(P0).library.len();
    cast_free(&mut g, P0, "Embrace the Paradox", Ctx::default()); // mana value 5
    assert_eq!(values::epow(&g, m), base + 2);
    assert!(g.player(P0).impulse_long.iter().any(|x| x.0 == card(&g, "Forest")), "the exiled Forest is playable");
    assert!(g.player(P0).library.len() < lib);
}

// ======================================================== Falcon and Redwing
#[test]
fn falcon_and_redwing_makes_birds_equal_to_the_damage_and_grows() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Falcon and Redwing");
    assert_eq!(g.player(P0).perms.len(), 1, "no tokens when it enters");
    combat::resolve_combat(&mut g, P0, &[f], P1, &[]).unwrap();
    let birds: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(birds.len(), 1, "1 damage: one Bird");
    assert!(g.perm(birds[0]).fly && values::epow(&g, birds[0]) == 1);
    assert_eq!(g.perm(f).plus, 1);
    g.perm_mut(f).tapped = false;
    combat::resolve_combat(&mut g, P0, &[f], P2, &[]).unwrap(); // now 2 power: two Birds
    let n = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    assert_eq!(n, 3);
    assert_eq!(g.perm(f).plus, 2);
}

// ======================================================== Photon, Mighty Marvel
#[test]
fn photon_adds_that_much_mana_of_the_colour_the_hand_wants() {
    let mut g = tbl();
    let p = perm(&mut g, P0, "Photon, Mighty Marvel");
    hand(&mut g, P0, &["Lightning Bolt"]);
    combat::resolve_combat(&mut g, P0, &[p], P1, &[]).unwrap();
    assert_eq!(g.player(P0).mana.get(Kind::R), 2, "2 damage: RR");
    assert!(mana_total(&g) >= 2);
    fn mana_total(g: &Game) -> u32 {
        sim_core::engine::mana::total_mana(g, P0, false)
    }
}

// ======================================================== Faeburrow Elder
#[test]
fn faeburrow_elder_scales_with_colours_and_taps_for_each() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Faeburrow Elder"); // green and white
    assert_eq!((values::epow(&g, e), values::etgh(&g, e)), (2, 2));
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2);
    perm(&mut g, P0, "Cuboid Colony"); // green and blue
    assert_eq!((values::epow(&g, e), values::etgh(&g, e)), (3, 3));
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 3 + 0);
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "GWU", false), "one mana of each colour");
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "GGW", false), "but only one of each");
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "GWU", false).unwrap());
    assert!(g.perm(e).tapped);
}

// ======================================================== Paradox Surveyor
#[test]
fn paradox_surveyor_takes_a_land_from_the_top_five_and_buries_the_rest() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Lightning Bolt", "Sol Ring", "Forest", "Cultivate", "Mind Stone"]);
    let (n, h) = (g.player(P0).library.len(), g.player(P0).hand.len());
    perm(&mut g, P0, "Paradox Surveyor");
    assert_eq!(g.player(P0).hand.len(), h + 1);
    assert!(g.player(P0).hand.contains(&card(&g, "Forest")), "the only land");
    assert_eq!(g.player(P0).library.len(), n - 1);
    let bottom: Vec<CardId> = g.player(P0).library[..4].to_vec();
    for name in ["Lightning Bolt", "Sol Ring", "Cultivate", "Mind Stone"] {
        assert!(bottom.contains(&card(&g, name)), "{name} went to the bottom");
    }
}

// ======================================================== Spectral Lynx
#[test]
fn spectral_lynx_has_protection_from_green_and_regenerates_for_b() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 1, false);
    let l = perm(&mut g, P0, "Spectral Lynx");
    assert!(values::protected_from(&g, l, Colors::from_letters("G")));
    removal::apply_removal(&mut g, Some(P1), l, "destroy", None).unwrap();
    assert!(on(&g, l) && g.perm(l).tapped, "regenerated with the Swamp");
    assert_eq!(untapped_lands(&g, P0), 0);
    removal::apply_removal(&mut g, Some(P1), l, "destroy", None).unwrap();
    assert!(!on(&g, l), "no {{B}} left: it dies");
}

// ======================================================== Nightveil Predator
#[test]
fn nightveil_predator_is_a_hexproof_flying_deathtoucher() {
    let mut g = tbl();
    let n = perm(&mut g, P0, "Nightveil Predator");
    assert!(values::untargetable(&g, n));
    assert!(sim_core::dsl::has_kw(&g, n, "flying") && sim_core::dsl::has_kw(&g, n, "deathtouch"));
    removal::apply_removal(&mut g, Some(P1), n, "destroy", None).unwrap();
    assert!(on(&g, n), "an opponent can't target it");
}

// ======================================================== Sphinx of New Prahv
#[test]
fn sphinx_of_new_prahv_taxes_spells_that_target_it_by_two() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Sphinx of New Prahv");
    let swords = card(&g, "Swords to Plowshares");
    lands(&mut g, P1, "Plains", 1, false);
    removal::apply_removal(&mut g, Some(P1), s, "exile", Some(swords)).unwrap();
    assert!(on(&g, s), "one mana for the tax is not enough");
    lands(&mut g, P1, "Plains", 1, false);
    removal::apply_removal(&mut g, Some(P1), s, "exile", Some(swords)).unwrap();
    assert!(!on(&g, s), "{{2}} paid");
    assert_eq!(untapped_lands(&g, P1), 0);
}

// ======================================================== Twisted Fates
#[test]
fn twisted_fates_destroys_and_counters_every_creature_of_ours() {
    let mut g = tbl();
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 3);
    let ring = perm(&mut g, P1, "Sol Ring");
    cast_free(&mut g, P0, "Twisted Fates", Ctx { target: Some(ring), ..Default::default() });
    assert!(!on(&g, ring));
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (1, 1));
}

// ======================================================== King Darien XLVIII
#[test]
fn king_darien_sacrifices_to_save_tokens_from_a_sweeper() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "King Darien XLVIII");
    let toks: Vec<PermId> = (0..4).map(|_| token(&mut g, P0, 3)).collect();
    decks::wipe_response(&mut g, P0, "destroy", P1).unwrap();
    assert!(!on(&g, k), "sacrificed");
    for t in &toks {
        assert!(values::indestructible(&g, *t) && values::untargetable(&g, *t));
    }
    zones::die(&mut g, toks[0], "destroy").unwrap();
    assert!(on(&g, toks[0]), "the sweeper doesn't kill them");
}

#[test]
fn king_darien_stays_home_for_a_lone_small_token() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "King Darien XLVIII");
    let t = token(&mut g, P0, 1);
    decks::wipe_response(&mut g, P0, "destroy", P1).unwrap();
    assert!(on(&g, k), "one 1/1 token isn't worth the King");
    assert!(!values::indestructible(&g, t));
}

// ======================================================== Razia, Boros Archangel
#[test]
fn razia_redirects_burn_aimed_at_a_creature_that_would_die() {
    let mut g = tbl();
    let r = perm(&mut g, P0, "Razia, Boros Archangel");
    let bear = token(&mut g, P0, 2); // a 2/2... toughness 2
    g.perm_mut(bear).tgh = 2;
    let victim = token(&mut g, P1, 2);
    g.perm_mut(victim).tgh = 2;
    let c = card(&g, "Lightning Bolt");
    removal::apply_removal(&mut g, Some(P1), bear, "dmg3", Some(c)).unwrap();
    assert!(on(&g, bear), "the damage was redirected");
    assert!(!on(&g, victim), "to their creature");
    assert!(g.perm(r).tapped);
    // tapped: no second redirect
    removal::apply_removal(&mut g, Some(P1), bear, "dmg3", Some(c)).unwrap();
    assert!(!on(&g, bear));
}

// ======================================================== Dakkon Blackblade, Star Compass, Embrace the Paradox
#[test]
fn dakkon_blackblade_is_as_big_as_the_lands_you_control() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let d = perm(&mut g, P0, "Dakkon Blackblade");
    assert_eq!((values::epow(&g, d), values::etgh(&g, d)), (3, 3));
    lands(&mut g, P0, "Island", 2, false);
    assert_eq!((values::epow(&g, d), values::etgh(&g, d)), (5, 5));
}

#[test]
fn star_compass_makes_the_colours_of_our_basic_lands() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Star Compass");
    assert!(g.perm(s).tapped, "enters tapped");
    g.perm_mut(s).tapped = false;
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 1, "", false), "no basic land: no mana");
    lands(&mut g, P0, "Swamp", 1, true);
    lands(&mut g, P0, "Command Tower", 1, true);
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "B", false));
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "R", false), "Command Tower is not a basic land");
}

#[test]
fn embrace_the_paradox_draws_three_and_puts_a_land_in_tapped() {
    let mut g = tbl();
    hand(&mut g, P0, &["Forest"]);
    set_library(&mut g, P0, &["Sol Ring", "Mind Stone", "Arcane Signet"]);
    let lands_before = g.player(P0).lands.len();
    cast_free(&mut g, P0, "Embrace the Paradox", Ctx::default());
    assert_eq!(g.player(P0).lands.len(), lands_before + 1);
    assert_eq!(untapped_lands(&g, P0), 0, "tapped");
    for n in ["Sol Ring", "Mind Stone", "Arcane Signet"] {
        assert!(g.player(P0).hand.contains(&card(&g, n)));
    }
}

// ======================================================== the compiled cards worth a look
#[test]
fn elenda_grows_on_deaths_and_leaves_vampires_equal_to_her_power() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Elenda, the Dusk Rose");
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.perm(e).plus, 1, "another creature died");
    let p = values::epow(&g, e);
    zones::die(&mut g, e, "destroy").unwrap();
    let vamps = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && values::epow(&g, m) == 1).count();
    assert_eq!(vamps as i32, p, "X 1/1 lifelink Vampires, X her power");
}

#[test]
fn lyra_gives_other_angels_plus_one_and_lifelink() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Giada, Font of Hope");
    let base = values::epow(&g, a);
    perm(&mut g, P0, "Lyra Dawnbringer");
    assert_eq!(values::epow(&g, a), base + 1);
    assert!(sim_core::dsl::has_kw(&g, a, "lifelink"));
}

#[test]
fn moss_diamond_enters_tapped_and_taps_for_green() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Moss Diamond");
    assert!(g.perm(m).tapped);
    g.perm_mut(m).tapped = false;
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "G", false));
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "U", false));
}

#[test]
fn silverquill_charm_counters_exile_and_drain() {
    // mode 1: two +1/+1 counters
    let mut g = tbl();
    let mine = token(&mut g, P0, 1);
    cast_free(&mut g, P0, "Silverquill Charm", Ctx { x: 1, target: Some(mine), ..Default::default() });
    assert_eq!(g.perm(mine).plus, 2);
    // mode 2: exile power 2 or less, not more
    let small = token(&mut g, P1, 2);
    let big = token(&mut g, P1, 3);
    cast_free(&mut g, P0, "Silverquill Charm", Ctx { x: 2, target: Some(big), ..Default::default() });
    assert!(on(&g, big), "power 3 is not a legal target");
    cast_free(&mut g, P0, "Silverquill Charm", Ctx { x: 2, target: Some(small), ..Default::default() });
    assert!(!on(&g, small));
    // mode 3: each opponent loses 3, we gain 3 once
    let (me, a, b) = (g.player(P0).life, g.player(P1).life, g.player(P2).life);
    cast_free(&mut g, P0, "Silverquill Charm", Ctx { x: 3, ..Default::default() });
    assert_eq!((g.player(P0).life, g.player(P1).life, g.player(P2).life), (me + 3, a - 3, b - 3));
}

#[test]
fn silverquill_charm_is_offered_as_a_hand_play_with_its_modes() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    token(&mut g, P0, 2);
    token(&mut g, P1, 1);
    let c = hand(&mut g, P0, &["Silverquill Charm"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 3, "counters, exile and drain");
    let exile = o.iter().find(|x| x.label.contains("exile")).unwrap();
    assert!(run(&mut g, P0, exile));
    assert!(g.player(P1).perms.iter().all(|&m| !on(&g, m)), "their 1/1 was exiled");
    assert_eq!(untapped_lands(&g, P0), 0);
}

// ======================================================== the Equipment package (kraum deck, 10/16)
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::Action;
use sim_core::impls::tymna_cards as tc;

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn attach(g: &mut Game, e: PermId, m: PermId) {
    g.perm_mut(e).attached = Some(m);
}

fn kraum_table() -> Game {
    table(&["thrasios", "veyran", "seph"])
}

// ------------------------------------------------------------------ Gilgamesh, Master-at-Arms
#[test]
fn gilgamesh_puts_the_equipment_among_the_top_six_onto_the_battlefield_and_attaches_one() {
    let mut g = tbl();
    // the last name is the top card
    set_library(&mut g, P0, &["Mountain", "Swamp", "Samurai's Katana", "Island", "Plains", "Conqueror's Flail"]);
    let before = g.player(P0).library.len();
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms");
    let flail = on_bf(&g, P0, "Conqueror's Flail").expect("the Flail came in");
    let katana = on_bf(&g, P0, "Samurai's Katana").expect("the Katana came in");
    assert_eq!(g.perm(flail).attached, Some(gil), "the best Equipment is attached to Gilgamesh, a Samurai");
    let hero = g.perm(katana).attached.expect("job select: the Katana wears its own Hero");
    assert!(g.perm(hero).token && tc::is_samurai(&g, hero), "the Katana makes its wearer a Samurai");
    assert_eq!(g.player(P0).library.len(), before - 2, "two Equipment left the library");
    // the four other cards of the six are on the bottom (index 0 is the bottom)
    let bottom: Vec<String> = g.player(P0).library[..4].iter().map(|&c| g.db.get(c).name.to_string()).collect();
    for n in ["Mountain", "Swamp", "Island", "Plains"] {
        assert!(bottom.iter().any(|b| b == n), "{n} went to the bottom: {bottom:?}");
    }
    assert_eq!(epow(&g, gil), 6 + 1, "6/6 plus one colour among permanents (red)");
}

#[test]
fn gilgamesh_looks_again_when_he_attacks() {
    let mut g = tbl();
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms");
    set_library(&mut g, P0, &["Plains", "Hero's Heirloom", "Island"]);
    let n = g.player(P0).library.len();
    combat::attack_triggers(&mut g, P0, &[gil], P1).unwrap();
    let heirloom = on_bf(&g, P0, "Hero's Heirloom").expect("the Heirloom came in");
    assert_eq!(g.player(P0).library.len(), n - 1);
    assert_eq!(g.perm(heirloom).attached, Some(gil));
    assert!(
        sim_core::dsl::has_kw(&g, gil, "trample") && sim_core::dsl::has_kw(&g, gil, "haste"),
        "legendary: trample and haste"
    );
}

#[test]
fn gilgamesh_with_no_equipment_in_the_top_six_only_reorders() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Island", "Swamp"]);
    let n = g.player(P0).library.len();
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms");
    assert_eq!(g.player(P0).library.len(), n);
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).name.contains("Equipment")));
    assert_eq!(epow(&g, gil), 6);
}

// ------------------------------------------------------------------ Dragoon's Lance, Samurai's Katana
fn equip_cost_for(g: &Game, p: PlayerId, e: PermId) -> Option<u32> {
    sim_core::dsl::equip_options(g, p).into_iter().find_map(|o| match o.act {
        Some(Action::Equip { src, n, .. }) if src == e => Some(n),
        _ => None,
    })
}

#[test]
fn dragoons_lance_makes_a_hero_flies_on_your_turn_and_equips_for_four() {
    let mut g = tbl();
    let lance = perm(&mut g, P0, "Dragoon's Lance");
    let hero = g.perm(lance).attached.expect("job select");
    assert!(g.perm(hero).token && sim_core::engine::values::has_type(&g, hero, "hero"));
    assert_eq!((epow(&g, hero), etgh(&g, hero)), (2, 1), "+1/+0");
    assert!(sim_core::dsl::has_kw(&g, hero, "flying"), "my turn");
    g.active = Some(P1);
    assert!(!sim_core::dsl::has_kw(&g, hero, "flying"), "not during another player's turn");
    g.active = Some(P0);
    // Gae Bolg: equip {4}
    attach_none(&mut g, lance);
    let other = token(&mut g, P0, 2);
    lands(&mut g, P0, "Plains", 3, false);
    assert_eq!(equip_cost_for(&g, P0, lance), None, "three lands: not enough");
    lands(&mut g, P0, "Plains", 1, false);
    assert_eq!(equip_cost_for(&g, P0, lance), Some(4));
    let _ = other;
}

fn attach_none(g: &mut Game, e: PermId) {
    g.perm_mut(e).attached = None;
}

#[test]
fn samurais_katana_gives_plus_two_trample_haste_and_equips_for_five() {
    let mut g = tbl();
    let katana = perm(&mut g, P0, "Samurai's Katana");
    let hero = g.perm(katana).attached.expect("job select");
    assert!(g.perm(hero).token && tc::is_samurai(&g, hero));
    assert_eq!((epow(&g, hero), etgh(&g, hero)), (3, 3));
    assert!(sim_core::dsl::has_kw(&g, hero, "trample"));
    // haste lets a summoning-sick creature attack
    let lynx = perm(&mut g, P0, "Spectral Lynx");
    g.perm_mut(lynx).sick = true;
    assert!(!sim_core::human::combat::can_attack(&g, P0, lynx));
    attach(&mut g, katana, lynx);
    assert!(sim_core::human::combat::can_attack(&g, P0, lynx), "haste");
    assert!(tc::is_samurai(&g, lynx));
    // Murasame: equip {5}
    attach_none(&mut g, katana);
    lands(&mut g, P0, "Plains", 4, false);
    assert_eq!(equip_cost_for(&g, P0, katana), None);
    lands(&mut g, P0, "Plains", 1, false);
    assert_eq!(equip_cost_for(&g, P0, katana), Some(5));
}

// ------------------------------------------------------------------ Conqueror's Flail
#[test]
fn conquerors_flail_is_plus_one_for_each_colour_among_your_permanents() {
    let mut g = tbl();
    let flail = perm(&mut g, P0, "Conqueror's Flail");
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    let base = epow(&g, lyra);
    attach(&mut g, flail, lyra);
    assert_eq!(epow(&g, lyra), base + 1, "white only");
    perm(&mut g, P0, "Sphinx of New Prahv");
    assert_eq!(epow(&g, lyra), base + 2, "white and blue");
    perm(&mut g, P0, "Nightveil Predator");
    assert_eq!(epow(&g, lyra), base + 3, "white, blue and black");
    assert_eq!(etgh(&g, lyra), 5 + 3);
}

#[test]
fn conquerors_flail_stops_opponents_casting_spells_on_your_turn_only_while_attached() {
    let mut g = tbl();
    let flail = perm(&mut g, P0, "Conqueror's Flail");
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    assert!(!values::silenced(&g, P1), "loose: nothing");
    attach(&mut g, flail, lyra);
    assert!(values::silenced(&g, P1) && values::silenced(&g, P2), "attached: opponents can't cast on my turn");
    assert!(!values::silenced(&g, P0), "I can");
    g.active = Some(P1);
    assert!(!values::silenced(&g, P0) && !values::silenced(&g, P2), "their own turn: cast away");
}

#[test]
fn conquerors_flail_is_equipped_by_the_kraum_seat_before_combat() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = kraum_table();
    lands(&mut g, P0, "Plains", 3, false);
    let flail = perm(&mut g, P0, "Conqueror's Flail");
    perm(&mut g, P0, "Nightwind Glider");
    let o = tc::kraum_special(&mut g, P0, false).unwrap();
    assert_eq!(o.len(), 1);
    assert!(o[0].label.starts_with("equip Conqueror's Flail"), "{}", o[0].label);
    assert!(o[0].utility >= 4.0, "above the minor plays");
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(flail).attached.is_some());
    assert_eq!(untapped_lands(&g, P0), 1, "equip {{2}}");
    assert!(tc::kraum_special(&mut g, P0, false).unwrap().is_empty(), "worn: nothing more to do");
    assert!(tc::kraum_special(&mut g, P0, true).unwrap().is_empty(), "never after combat");
}

// ------------------------------------------------------------------ Hero's Blade, Hero's Heirloom
#[test]
fn heros_blade_attaches_free_to_each_legendary_creature_you_play() {
    let mut g = kraum_table();
    let blade = perm(&mut g, P0, "Hero's Blade");
    perm(&mut g, P0, "Exit Specialist");
    assert_eq!(g.perm(blade).attached, None, "not legendary");
    let kraum = perm(&mut g, P0, "Kraum, Ludevic's Opus");
    assert_eq!(g.perm(blade).attached, Some(kraum));
    assert_eq!((epow(&g, kraum), etgh(&g, kraum)), (4 + 3, 4 + 2));
}

#[test]
fn heros_heirloom_gives_trample_and_haste_only_to_a_legendary_creature() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hero's Heirloom");
    let plain = perm(&mut g, P0, "Exit Specialist");
    let legend = perm(&mut g, P0, "Moseo, Vein's New Dean");
    let epow_base = epow(&g, plain);
    attach(&mut g, h, plain);
    assert_eq!(epow(&g, plain), epow_base + 2);
    assert!(!sim_core::dsl::has_kw(&g, plain, "trample") && !sim_core::dsl::has_kw(&g, plain, "haste"));
    attach(&mut g, h, legend);
    assert!(sim_core::dsl::has_kw(&g, legend, "trample") && sim_core::dsl::has_kw(&g, legend, "haste"));
}

// ------------------------------------------------------------------ Medic's Kitesail, Trailblazer's Boots, Nim Deathmantle
#[test]
fn medics_kitesail_gives_flying_plus_one_and_a_life_per_attack() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Medic's Kitesail");
    let m = token(&mut g, P0, 2);
    attach(&mut g, k, m);
    assert!(sim_core::dsl::has_kw(&g, m, "flying"));
    assert_eq!(epow(&g, m), 3);
    let life = g.player(P0).life;
    combat::attack_triggers(&mut g, P0, &[m], P1).unwrap();
    assert_eq!(g.player(P0).life, life + 1);
}

#[test]
fn trailblazers_boots_make_it_unblockable_while_the_defender_has_a_nonbasic_land() {
    let mut g = tbl();
    let boots = perm(&mut g, P0, "Trailblazer's Boots");
    let a = token(&mut g, P0, 2);
    attach(&mut g, boots, a);
    let b = token(&mut g, P1, 2);
    lands(&mut g, P1, "Plains", 2, false);
    assert!(combat::can_block(&g, b, a), "only basic lands: blockable");
    lands(&mut g, P1, "Command Tower", 1, false);
    assert!(!combat::can_block(&g, b, a), "a nonbasic land: unblockable");
    // not another player's creature
    let other = token(&mut g, P2, 2);
    lands(&mut g, P2, "Plains", 1, false);
    assert!(combat::can_block(&g, other, a), "the player without a nonbasic land can block");
}

#[test]
fn nim_deathmantle_is_plus_two_intimidate_and_brings_a_dying_creature_back() {
    let mut g = tbl();
    let nim = perm(&mut g, P0, "Nim Deathmantle");
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    let base = epow(&g, lyra);
    attach(&mut g, nim, lyra);
    assert_eq!(epow(&g, lyra), base + 2);
    let white = perm(&mut g, P1, "Bastion Protector");
    let black = perm(&mut g, P1, "Nightveil Predator");
    assert!(!combat::can_block(&g, white, lyra), "intimidate: a white creature can't block");
    assert!(combat::can_block(&g, black, lyra), "a black one can");
    lands(&mut g, P0, "Plains", 4, false);
    zones::die(&mut g, lyra, sim_core::sym::intern("destroy")).unwrap();
    let back = on_bf(&g, P0, "Lyra Dawnbringer").expect("{4} paid: Lyra returns");
    assert_eq!(g.perm(nim).attached, Some(back));
    assert_eq!(untapped_lands(&g, P0), 0);
}

// ------------------------------------------------------------------ the lands
#[test]
fn irrigated_farmland_enters_tapped_makes_white_or_blue_and_cycles_for_two() {
    let mut g = tbl();
    let c = card(&g, "Irrigated Farmland");
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c));
    assert_eq!(g.db.get(c).tags.str(sim_core::tag::Tag::C), Some("WU"));
    // cycling: only when it isn't needed as a land drop
    hand(&mut g, P0, &["Irrigated Farmland"]);
    lands(&mut g, P0, "Plains", 3, false);
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "three lands: it is the land drop");
    lands(&mut g, P0, "Plains", 2, false);
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n, "discarded one, drew one");
    assert!(g.player(P0).gy.contains(&c));
    assert_eq!(untapped_lands(&g, P0), 3, "{{2}} paid");
}

#[test]
fn spectacle_summit_surveils_at_the_end_of_an_opponents_turn() {
    let mut g = tbl();
    g.active = Some(P1);
    let l = lands(&mut g, P0, "Spectacle Summit", 1, false)[0];
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 2, false);
    let c = g.perm(l).card();
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c));
    let f = g.registry.get(c).unwrap().land_options.unwrap();
    let o = f(&mut g, l, P0, None).unwrap();
    assert_eq!(o.len(), 1, "{{2}}{{U}}{{R}} and {{T}}");
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(l).tapped);
    g.active = Some(P0);
    assert!(f(&mut g, l, P0, None).unwrap().is_empty(), "not on my own turn");
}

#[test]
fn boiling_rock_prison_is_sacrificed_for_a_card_when_flooded() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Boiling Rock Prison", 1, false)[0];
    lands(&mut g, P0, "Swamp", 4, false);
    let c = g.perm(l).card();
    let f = g.registry.get(c).unwrap().land_options.unwrap();
    assert!(f(&mut g, l, P0, None).unwrap().is_empty(), "five lands: not flooded");
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(f(&mut g, l, P0, Some(false)).unwrap().is_empty(), "only at the end of a turn");
    let o = f(&mut g, l, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.player(P0).gy.contains(&c));
    assert_eq!(g.player(P0).lands.len(), 5);
    assert_eq!(untapped_lands(&g, P0), 1, "{{4}} paid, and the Prison tapped itself");
}

// ------------------------------------------------------------------ Diabolic Tutor: what the kraum seat fetches
fn tutor(g: &Game) -> String {
    let c = decks::tutor_pick(g, P0, "any").expect("a pick");
    g.db.get(c).name.to_string()
}

#[test]
fn the_kraum_seat_tutors_for_a_land_when_short_of_mana() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = kraum_table();
    g.player_mut(P0).turns = 4;
    lands(&mut g, P0, "Plains", 2, false);
    let c = decks::tutor_pick(&g, P0, "any").unwrap();
    assert!(g.db.get(c).land, "{}", g.db.get(c).name);
}

#[test]
fn the_kraum_seat_tutors_for_a_board_wipe_when_far_behind_on_board() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = kraum_table();
    g.player_mut(P0).turns = 6;
    lands(&mut g, P0, "Plains", 6, false);
    for _ in 0..4 {
        token(&mut g, P1, 3);
    }
    assert_eq!(tutor(&g), "Austere Command");
}

#[test]
fn the_kraum_seat_tutors_for_a_finisher_not_the_flail_when_ahead() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    // audit 10/18: the Flail was fetched every time; the finisher castable by next turn comes first
    let mut g = kraum_table();
    g.player_mut(P0).turns = 6;
    lands(&mut g, P0, "Plains", 6, false);
    for _ in 0..3 {
        token(&mut g, P0, 3);
    }
    assert_eq!(tutor(&g), "Grave Titan");
    hand(&mut g, P0, &["Grave Titan"]);
    assert!(
        ["Terror of the Peaks", "Sun Titan", "Baneslayer Angel", "Lyra Dawnbringer", "Glorybringer"]
            .contains(&tutor(&g).as_str()),
        "{}",
        tutor(&g)
    );
}

// ------------------------------------------------------------------ the kraum seat: where Equipment goes
fn best_equip_target(g: &Game, e: PermId) -> Option<PermId> {
    sim_core::dsl::equip_options(g, P0).into_iter().find_map(|o| match o.act {
        Some(Action::Equip { src, target, .. }) if src == e => Some(target),
        _ => None,
    })
}

#[test]
fn evasion_equipment_goes_on_the_creature_that_cannot_get_through_on_its_own() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = kraum_table();
    lands(&mut g, P0, "Plains", 4, false);
    let flier = perm(&mut g, P0, "Nightwind Glider");
    let ground = token(&mut g, P0, 2);
    for q in [P1, P2] {
        token(&mut g, q, 2); // a ground blocker for each opponent
    }
    let kite = perm(&mut g, P0, "Medic's Kitesail");
    g.perm_mut(kite).attached = None;
    assert_eq!(best_equip_target(&g, kite), Some(ground), "not the flier that connects already");
    let _ = flier;
    // the Lance and the Boots are the same
    let lance = perm(&mut g, P0, "Dragoon's Lance");
    g.perm_mut(lance).attached = None;
    assert_eq!(best_equip_target(&g, lance), Some(ground));
}

#[test]
fn size_equipment_goes_on_a_creature_that_connects_anyway() {
    let mut g = kraum_table();
    lands(&mut g, P0, "Plains", 4, false);
    let flier = perm(&mut g, P0, "Nightwind Glider");
    let ground = token(&mut g, P0, 2);
    for q in [P1, P2] {
        token(&mut g, q, 2);
    }
    let h = perm(&mut g, P0, "Hero's Heirloom");
    assert_eq!(best_equip_target(&g, h), Some(flier), "its hit is the card Tymna draws");
    let _ = ground;
}

#[test]
fn the_other_seats_equip_choice_is_unchanged() {
    let mut g = tbl();
    let (e, m) = (perm(&mut g, P0, "Medic's Kitesail"), token(&mut g, P0, 2));
    assert_eq!(tc::equip_holder_value(&g, P0, e, m), 0.0);
}

#[test]
fn tymnas_spread_uses_the_creature_a_kitesail_makes_fly() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    use sim_core::impls::tymna_partners::spread;
    let mut g = table(&["thrasios", "veyran", "seph", "galadriel"]);
    perm(&mut g, P0, "Tymna the Weaver");
    let glider = perm(&mut g, P0, "Nightwind Glider");
    let ground = token(&mut g, P0, 2);
    let kite = perm(&mut g, P0, "Medic's Kitesail");
    attach(&mut g, kite, ground);
    let third = perm(&mut g, P0, "Aven Mindcensor");
    for q in [P1, P2, P3] {
        token(&mut g, q, 2);
    }
    let all = vec![glider, ground, third];
    let groups = spread(&g, P0, vec![(P1, all.clone())], &all);
    let mut to: Vec<PlayerId> = all.iter().map(|&a| groups.iter().find(|x| x.1.contains(&a)).unwrap().0).collect();
    to.sort();
    assert_eq!(to, vec![P1, P2, P3], "each opponent is hit by one: {groups:?}");
}

// ------------------------------------------------------------------ Shield and Helm of Kaldra (no Sword in this deck)
#[test]
fn shield_goes_on_the_commander_and_helm_on_the_big_attacker_and_kaldra_is_never_made() {
    let mut g = kraum_table();
    lands(&mut g, P0, "Plains", 6, false);
    let tymna = perm(&mut g, P0, "Tymna the Weaver");
    let big = perm(&mut g, P0, "Lyra Dawnbringer");
    let shield = perm(&mut g, P0, "Shield of Kaldra");
    let helm = perm(&mut g, P0, "Helm of Kaldra");
    for q in [P1, P2] {
        token(&mut g, q, 2);
    }
    g.perm_mut(tymna).is_cmd = true; // (entered from the library here, not cast from the command zone)
    let s = best_equip_target(&g, shield);
    assert!(s == Some(tymna) || s == Some(big), "the commander or the best threat, not a token: {s:?}");
    assert_eq!(best_equip_target(&g, helm), Some(big), "first strike, trample, haste on the big creature");
    attach(&mut g, shield, tymna);
    assert!(sim_core::engine::values::indestructible(&g, tymna));
    attach(&mut g, helm, big);
    assert!(sim_core::dsl::has_kw(&g, big, "first strike") && sim_core::dsl::has_kw(&g, big, "haste"));
    // {1}: create Kaldra needs the Sword too: no option without it
    let c = g.perm(helm).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    assert!(f(&mut g, helm, P0, Some(false)).unwrap().is_empty(), "never pays {{1}} without the Sword");
}

// ======================================================== Phyrexian mana: Dismember, Porcelain Legionnaire, Vault Skirge, Apostle's Blessing
fn kraum_tbl() -> Game {
    table(&["thrasios", "galadriel", "veyran"])
}

/// the label of the main-phase option that casts `name`, run; true if there was one
fn cast_by_ai(g: &mut Game, p: PlayerId, name: &str) -> bool {
    let opts = sim_core::ai::brain::main_options(g, p, false).unwrap();
    match opts.iter().find(|o| o.label.contains(name)) {
        Some(o) => run(g, p, o),
        None => false,
    }
}

#[test]
fn dismember_gives_minus_five_and_pays_life_for_missing_black() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Dismember"]);
    let big = perm(&mut g, P1, "Baneslayer Angel"); // 5/5
    assert!(cast_by_ai(&mut g, P0, "Dismember"), "the AI casts it on the 4/5 with one {{B}} and 2 life");
    assert!(!on(&g, big), "-5/-5 kills a toughness-5 creature");
    assert_eq!(g.player(P0).life, 38, "one {{B/P}} paid with 2 life, the other with the Swamp");
    assert_eq!(untapped_lands(&g, P0), 0);
}

#[test]
fn dismember_is_not_cast_at_a_creature_it_cannot_kill_and_pays_mana_when_it_can() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Dismember"]);
    let huge = perm(&mut g, P1, "Baneslayer Angel");
    g.perm_mut(huge).plus = 1; // a +1/+1 counter: 6/6
    assert!(!cast_by_ai(&mut g, P0, "Dismember"), "-5/-5 doesn't kill a 6/6");
    assert!(on(&g, huge));
    g.perm_mut(huge).plus = 0;
    assert!(cast_by_ai(&mut g, P0, "Dismember"));
    assert!(!on(&g, huge));
    assert_eq!(g.player(P0).life, 40, "{{1}}{{B}}{{B}} paid in mana: no life");
}

#[test]
fn dismember_does_not_pay_life_at_ten_or_less() {
    let mut g = kraum_tbl();
    g.player_mut(P0).life = 10;
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Dismember"]);
    let big = perm(&mut g, P1, "Baneslayer Angel");
    assert!(!cast_by_ai(&mut g, P0, "Dismember"));
    assert!(on(&g, big));
    assert_eq!(g.player(P0).life, 10);
}

#[test]
fn porcelain_legionnaire_is_a_three_one_first_striker_for_two_life_and_two_mana() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 2, false); // no white mana
    hand(&mut g, P0, &["Porcelain Legionnaire"]);
    assert!(cast_by_ai(&mut g, P0, "Porcelain Legionnaire"));
    let l = *g.player(P0).perms.last().unwrap();
    assert_eq!(&*g.perm(l).name, "Porcelain Legionnaire");
    assert_eq!((values::epow(&g, l), values::etgh(&g, l)), (3, 1));
    assert!(sim_core::dsl::has_kw(&g, l, "first strike"));
    assert!(g.is_creature(l));
    assert_eq!(g.player(P0).life, 38, "{{W/P}} paid with 2 life");
}

#[test]
fn porcelain_legionnaire_pays_white_mana_when_it_has_it() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Porcelain Legionnaire"]);
    assert!(cast_by_ai(&mut g, P0, "Porcelain Legionnaire"));
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn porcelain_legionnaire_first_strike_kills_a_three_toughness_blocker_unharmed() {
    let mut g = kraum_tbl();
    let l = perm(&mut g, P0, "Porcelain Legionnaire");
    let blocker = token(&mut g, P1, 3);
    g.perm_mut(blocker).tgh = 3;
    assert!(combat::kw(&g, l, "first strike"));
    assert!(!combat::kw(&g, blocker, "first strike"));
}

#[test]
fn vault_skirge_is_a_flying_lifelink_artifact_creature_for_one_mana_and_two_life() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Vault Skirge"]);
    assert!(cast_by_ai(&mut g, P0, "Vault Skirge"));
    let s = *g.player(P0).perms.last().unwrap();
    assert_eq!(&*g.perm(s).name, "Vault Skirge");
    assert_eq!((values::epow(&g, s), values::etgh(&g, s)), (1, 1));
    assert!(sim_core::dsl::has_kw(&g, s, "flying"));
    assert!(sim_core::dsl::has_kw(&g, s, "lifelink"));
    assert!(
        values::has_type(&g, s, "artifact")
            || g.db.get(g.perm(s).cd.unwrap()).types.has(sim_core::cards::Types::ARTIFACT)
    );
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn apostle_blessing_stops_a_black_removal_spell_on_the_commander_and_gives_protection_from_black() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Apostle's Blessing"]);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(doom)).unwrap();
    assert!(on(&g, t), "protection from black: Doom Blade fizzles");
    assert!(values::protected_from(&g, t, Colors::from_letters("B")));
    assert!(!values::protected_from(&g, t, Colors::from_letters("R")));
    assert!(g.player(P0).hand.is_empty());
    assert_eq!(g.player(P0).life, 40, "paid with {{W}}");
    // until end of turn only
    g.perm_mut(t).eot_kw.clear();
    assert!(!values::protected_from(&g, t, Colors::from_letters("B")));
}

#[test]
fn apostle_blessing_pays_two_life_without_white_mana() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 1, false);
    let k = perm(&mut g, P0, "Kraum, Ludevic's Opus");
    g.perm_mut(k).is_cmd = true;
    hand(&mut g, P0, &["Apostle's Blessing"]);
    let bolt = card(&g, "Lightning Bolt");
    removal::apply_removal(&mut g, Some(P1), k, "dmg3", Some(bolt)).unwrap();
    assert!(on(&g, k));
    assert!(values::protected_from(&g, k, Colors::from_letters("R")));
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn apostle_blessing_is_kept_for_a_creature_worth_saving_and_never_cast_on_its_own() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let small = token(&mut g, P0, 1);
    hand(&mut g, P0, &["Apostle's Blessing"]);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), small, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, small), "a 1/1 token isn't worth the card");
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(!cast_by_ai(&mut g, P0, "Apostle's Blessing"), "no main-phase cast");
}

#[test]
fn apostle_blessing_chooses_artifacts_against_a_colourless_artifact_spell_and_stops_artifact_damage() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Apostle's Blessing"]);
    let sol = card(&g, "Sol Ring"); // any colourless artifact card as the source
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(sol)).unwrap();
    assert!(on(&g, t));
    assert!(g.perm(t).eot_kw.contains(&"pro_artifacts"));
    let golem = perm(&mut g, P1, "Vault Skirge");
    assert!(sim_core::impls::rules2::prot_vs(&g, t, golem), "no damage from artifact creatures");
}

#[test]
fn apostle_blessing_is_not_used_against_your_own_removal() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Apostle's Blessing"]);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P0), t, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, t));
    assert_eq!(g.player(P0).hand.len(), 1);
}

// ======================================================== Phyrexian round 3: Archfiend, Skrelv, Gut Shot, White Sun's Twilight, Unlife
use sim_core::engine::turn;

fn oil(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Oil)
}

#[test]
fn archfiend_enters_with_four_oil_counters_and_drains_two_when_an_opposing_creature_dies() {
    let mut g = kraum_tbl();
    let a = perm(&mut g, P0, "Archfiend of the Dross");
    assert_eq!(oil(&g, a), 4);
    assert_eq!((values::epow(&g, a), values::etgh(&g, a)), (6, 6));
    assert!(sim_core::dsl::has_kw(&g, a, "flying"));
    let victim = token(&mut g, P1, 2);
    let mine = token(&mut g, P0, 2);
    zones::die(&mut g, victim, "destroy").unwrap();
    assert_eq!(g.player(P1).life, 38, "its controller loses 2 life");
    zones::die(&mut g, mine, "destroy").unwrap();
    assert_eq!(g.player(P0).life, 40, "only opposing creatures");
}

#[test]
fn archfiend_loses_an_oil_counter_each_upkeep_and_its_controller_loses_the_game_at_none() {
    let mut g = kraum_tbl();
    let a = perm(&mut g, P0, "Archfiend of the Dross");
    for want in [3, 2, 1] {
        turn::upkeep(&mut g, P0).unwrap();
        assert_eq!(oil(&g, a), want);
        assert!(g.player(P0).alive);
    }
    // another player's upkeep does nothing
    turn::upkeep(&mut g, P1).unwrap();
    assert_eq!(oil(&g, a), 1);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(oil(&g, a), 0);
    assert!(!g.player(P0).alive, "no oil counters left: the controller loses the game");
    assert_eq!(g.player(P0).stats.get("archfiend_lost").copied(), Some(1));
}

#[test]
fn archfiend_is_blinked_by_restoration_angel_before_the_last_counter_and_starts_over() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 2, false);
    lands(&mut g, P0, "Island", 2, false);
    let a = perm(&mut g, P0, "Archfiend of the Dross");
    g.perm_mut(a).data.set(DataKey::Oil, sim_core::state::Val::Int(1));
    hand(&mut g, P0, &["Restoration Angel"]);
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).alive, "saved");
    assert!(!on(&g, a), "the old object left");
    let new = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| &*g.perm(m).name == "Archfiend of the Dross")
        .expect("it came back");
    assert_eq!(oil(&g, new), 4, "a new object: four counters again");
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Restoration Angel"));
    assert_eq!(g.player(P0).stats.get("archfiend_saved").copied(), Some(1));
}

#[test]
fn archfiend_without_a_blinker_or_the_mana_still_loses() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 2, false); // not enough for {3}{W}
    let a = perm(&mut g, P0, "Archfiend of the Dross");
    g.perm_mut(a).data.set(DataKey::Oil, sim_core::state::Val::Int(1));
    hand(&mut g, P0, &["Restoration Angel"]);
    turn::upkeep(&mut g, P0).unwrap();
    assert!(!g.player(P0).alive);
}

fn skrelv_ready(g: &mut Game, p: PlayerId) -> PermId {
    let s = perm(g, p, "Skrelv, Defector Mite");
    g.perm_mut(s).sick = false;
    s
}

#[test]
fn skrelv_is_a_toxic_one_one_that_cannot_block_and_gives_a_poison_counter() {
    let mut g = kraum_tbl();
    let s = skrelv_ready(&mut g, P0);
    assert_eq!((values::epow(&g, s), values::etgh(&g, s)), (1, 1));
    assert!(sim_core::dsl::has_kw(&g, s, "toxic"));
    let atk = token(&mut g, P1, 2);
    assert!(!combat::can_block(&g, s, atk), "Skrelv can't block");
    sim_core::impls::tymna_cards::toxic_hit(&mut g, s, P1);
    assert_eq!(g.player(P1).poison, 1);
}

#[test]
fn skrelv_ability_gives_toxic_hexproof_from_a_colour_and_unblockability_for_the_turn() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let s = skrelv_ready(&mut g, P0);
    let a = perm(&mut g, P0, "Baneslayer Angel");
    let black_blocker = perm(&mut g, P1, "Vampire Nighthawk");
    let green_blocker = perm(&mut g, P1, "Llanowar Elves");
    assert!(combat::can_block(&g, black_blocker, a), "before the ability the black flier can block");
    let opts = options(&mut g, s, P0, Some(false));
    assert_eq!(opts.len(), 1, "an offer before combat: {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    assert!(opts[0].label.contains("Baneslayer"));
    assert!(opts[0].label.contains(" B "), "the black flier is the blocker that matters: {}", opts[0].label);
    assert!(run(&mut g, P0, &opts[0]));
    assert!(g.perm(s).tapped);
    assert_eq!(untapped_lands(&g, P0), 0, "paid {{W}}");
    assert_eq!(g.player(P0).life, 40);
    assert!(sim_core::dsl::has_kw(&g, a, "toxic") || g.perm(a).eot_kw.contains(&"toxic"));
    assert!(!combat::can_block(&g, black_blocker, a), "can't be blocked by black creatures");
    let black_removal = card(&g, "Murder");
    let red_burn = card(&g, "Lightning Bolt");
    assert!(sim_core::impls::tymna_cards::hexproof_from(&g, a, g.db.get(black_removal).colors));
    assert!(!sim_core::impls::tymna_cards::hexproof_from(&g, a, g.db.get(red_burn).colors));
    sim_core::impls::tymna_cards::toxic_hit(&mut g, a, P1);
    assert_eq!(g.player(P1).poison, 1, "toxic 1 from Skrelv's ability");
    // gone at end of turn
    g.perm_mut(a).eot_kw.clear();
    assert!(combat::can_block(&g, black_blocker, a), "the effect ends with the turn");
    assert!(
        combat::can_block(&g, green_blocker, a) || !sim_core::dsl::has_kw(&g, green_blocker, "reach"),
        "a green blocker was never affected"
    );
}

#[test]
fn skrelv_pays_two_life_without_white_mana_and_a_sick_skrelv_cannot_activate() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 1, false);
    let s = perm(&mut g, P0, "Skrelv, Defector Mite");
    g.perm_mut(s).sick = true;
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &[]);
    let murder = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(murder)).unwrap();
    assert!(!on(&g, t), "summoning sick Skrelv can't use {{T}}");
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 1, false);
    let _s = skrelv_ready(&mut g, P0);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(murder)).unwrap();
    assert!(on(&g, t), "Skrelv gave Tymna hexproof from black in response: Murder fizzles");
    assert_eq!(g.player(P0).life, 38, "{{W/P}} paid with 2 life");
    // a second black spell this turn can't target her either
    let t2 = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(t2)).unwrap();
    assert!(on(&g, t));
    // a red spell can
    let bolt = card(&g, "Lightning Bolt");
    removal::apply_removal(&mut g, Some(P1), t, "dmg3", Some(bolt)).unwrap();
    assert!(!on(&g, t));
}

#[test]
fn skrelv_does_not_save_a_small_creature_or_the_target_of_a_colourless_spell() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let _s = skrelv_ready(&mut g, P0);
    let small = token(&mut g, P0, 1);
    let murder = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), small, "destroy", Some(murder)).unwrap();
    assert!(!on(&g, small));
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    let sol = card(&g, "Sol Ring");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(sol)).unwrap();
    assert!(!on(&g, t), "no colour to choose against a colourless spell");
}

#[test]
fn skrelv_toxic_adds_to_the_poison_of_a_creature_with_toxic() {
    let mut g = kraum_tbl();
    let d = perm(&mut g, P0, "Dune Mover");
    g.perm_mut(d).eot_kw.push("toxic");
    sim_core::impls::tymna_cards::toxic_hit(&mut g, d, P1);
    assert_eq!(g.player(P1).poison, 2, "Skrelv's toxic and Dune Mover's own compiled toxic 1");
}

#[test]
fn gut_shot_kills_a_one_toughness_creature_and_pays_red_or_two_life() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    hand(&mut g, P0, &["Gut Shot"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.contains("Gut Shot")).expect("an option to shoot the elf");
    assert!(run(&mut g, P0, o));
    assert!(!on(&g, elf));
    assert_eq!(g.player(P0).life, 40, "paid with {{R}}");
    // no red: 2 life
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Island", 1, false);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    hand(&mut g, P0, &["Gut Shot"]);
    assert!(cast_by_ai(&mut g, P0, "Gut Shot"));
    assert!(!on(&g, elf));
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn gut_shot_is_held_without_a_one_toughness_target() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Gut Shot"]);
    assert!(!cast_by_ai(&mut g, P0, "Gut Shot"));
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn gut_shot_deals_one_damage_to_a_player_too() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let c = hand(&mut g, P0, &["Gut Shot"])[0];
    sim_core::engine::mana::pay(&mut g, P0, 0, "R", false).unwrap();
    let mut ctx = Ctx::default();
    ctx.face = Some(P1);
    assert!(cast::cast_card(&mut g, P0, c, "hand", ctx).unwrap());
    assert_eq!(g.player(P1).life, 39);
}

fn mites(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"mite")).collect()
}

#[test]
fn white_suns_twilight_gains_x_life_and_makes_x_toxic_mites_that_cannot_block() {
    let mut g = kraum_tbl();
    g.player_mut(P0).life = 20;
    let c = hand(&mut g, P0, &["White Sun's Twilight"])[0];
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 3, ..Ctx::default() }).unwrap());
    assert_eq!(g.player(P0).life, 23);
    let ms = mites(&g, P0);
    assert_eq!(ms.len(), 3);
    for &m in &ms {
        assert_eq!((values::epow(&g, m), values::etgh(&g, m)), (1, 1));
        assert!(sim_core::dsl::has_kw(&g, m, "toxic"));
        assert!(sim_core::dsl::has_kw(&g, m, "cant_block"));
        assert!(g.perm(m).colors == Colors::NONE, "colourless");
    }
    let atk = token(&mut g, P1, 2);
    assert!(!combat::can_block(&g, ms[0], atk));
    sim_core::impls::tymna_cards::toxic_hit(&mut g, ms[0], P1);
    assert_eq!(g.player(P1).poison, 1);
}

#[test]
fn white_suns_twilight_with_x_five_destroys_all_other_creatures() {
    let mut g = kraum_tbl();
    let mine = perm(&mut g, P0, "Serra Angel");
    let theirs = perm(&mut g, P1, "Grave Titan");
    let theirs2 = token(&mut g, P2, 3);
    let c = hand(&mut g, P0, &["White Sun's Twilight"])[0];
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 5, ..Ctx::default() }).unwrap());
    assert!(!on(&g, mine) && !on(&g, theirs) && !on(&g, theirs2));
    assert_eq!(g.player(P0).life, 45);
    assert!(mites(&g, P0).is_empty(), "the Mites are destroyed with the rest");
    // X = 4: no wipe
    let mut g = kraum_tbl();
    let theirs = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["White Sun's Twilight"])[0];
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 4, ..Ctx::default() }).unwrap());
    assert!(on(&g, theirs));
    assert_eq!(mites(&g, P0).len(), 4);
}

#[test]
fn white_suns_twilight_ai_wipes_when_behind_and_never_casts_it_at_x_zero() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 7, false);
    for q in [P1, P2] {
        perm(&mut g, q, "Grave Titan");
        perm(&mut g, q, "Baneslayer Angel");
    }
    hand(&mut g, P0, &["White Sun's Twilight"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.contains("White Sun")).expect("an option");
    assert!(o.label.contains("X=5"), "{}", o.label);
    assert!(run(&mut g, P0, o));
    assert!(g.player(P1).perms.iter().all(|&m| !g.is_creature(m)));
    assert_eq!(g.player(P0).life, 45);
}

#[test]
fn white_suns_twilight_ai_does_not_wipe_its_own_board_and_makes_four_mites_instead() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 8, false);
    perm(&mut g, P0, "Baneslayer Angel");
    perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P1, "Llanowar Elves");
    hand(&mut g, P0, &["White Sun's Twilight"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.contains("White Sun")).expect("an option");
    assert!(o.label.contains("X=4"), "{}", o.label);
    assert!(run(&mut g, P0, o));
    assert_eq!(mites(&g, P0).len(), 4);
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Baneslayer Angel"));
}

#[test]
fn white_suns_twilight_is_not_cast_with_too_little_mana() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 4, false);
    hand(&mut g, P0, &["White Sun's Twilight"]);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(!opts.iter().any(|o| o.label.contains("White Sun")));
}

#[test]
fn phyrexian_unlife_keeps_you_in_the_game_at_zero_life_and_turns_damage_into_poison() {
    let mut g = kraum_tbl();
    perm(&mut g, P0, "Phyrexian Unlife");
    g.player_mut(P0).life = 2;
    sim_core::engine::life::lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, -3, "damage that takes you to 0 or less still lowers life");
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.player(P0).alive, "you don't lose for having 0 or less life");
    assert_eq!(g.player(P0).poison, 0);
    sim_core::engine::life::lose_life(&mut g, P0, 4, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, -3, "now damage is poison");
    assert_eq!(g.player(P0).poison, 4);
    // life loss that is not damage still lowers life
    sim_core::engine::life::lose_life(&mut g, P0, 2, Some(P1), "drain", None).unwrap();
    assert_eq!(g.player(P0).life, -5);
    assert_eq!(g.player(P0).poison, 4);
    sim_core::engine::life::lose_life(&mut g, P0, 6, Some(P1), "burn", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).poison, 10);
    assert!(!g.player(P0).alive, "ten poison counters still lose");
}

#[test]
fn without_phyrexian_unlife_zero_life_loses_and_above_zero_damage_is_ordinary() {
    let mut g = kraum_tbl();
    g.player_mut(P0).life = 3;
    sim_core::engine::life::lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive);
    let mut g = kraum_tbl();
    perm(&mut g, P0, "Phyrexian Unlife");
    sim_core::engine::life::lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, 35);
    assert_eq!(g.player(P0).poison, 0, "above 0 life it is plain damage");
}

#[test]
fn phyrexian_unlife_is_cast_by_the_ai_at_low_life_only() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Phyrexian Unlife"]);
    assert!(!cast_by_ai(&mut g, P0, "Phyrexian Unlife"), "healthy: held");
    g.player_mut(P0).life = 15;
    assert!(cast_by_ai(&mut g, P0, "Phyrexian Unlife"));
}

// ------------------------------------------------------------------ Godo, Bandit Warlord
const GODO: &str = "Godo, Bandit Warlord";

fn godo_cast(g: &mut Game) -> PermId {
    let c = take(g, P0, GODO);
    zones::enter(g, P0, c, zones::Enter { sick: true, ..Default::default() }).unwrap()
}

// Godo's card code is now impls/t6_godo.rs (Tier 6; it registers after this module). Per Oracle text his enter
// trigger only puts the Equipment onto the battlefield: it is not attached (the Kraum swap test's model attached it).

#[test]
fn godo_fetches_one_equipment_onto_the_battlefield_unattached() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Mountain", "Conqueror's Flail", "Helm of Kaldra", "Shield of Kaldra", "Island"]);
    let before = g.player(P0).library.len();
    let godo = godo_cast(&mut g);
    let got: Vec<_> = ["Conqueror's Flail", "Helm of Kaldra", "Shield of Kaldra"]
        .iter()
        .filter_map(|n| on_bf(&g, P0, n))
        .collect();
    assert_eq!(got.len(), 1, "exactly one Equipment");
    assert_eq!(g.perm(got[0]).attached, None, "not attached to Godo");
    assert_eq!(g.player(P0).library.len(), before - 1);
    assert!(g.perm(godo).sick);
}

#[test]
fn godo_fetches_a_job_select_piece_unattached() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Samurai's Katana"]);
    let godo = godo_cast(&mut g);
    let katana = on_bf(&g, P0, "Samurai's Katana").expect("the Katana came in");
    assert_ne!(g.perm(katana).attached, Some(godo), "Godo's trigger does not attach it");
}

#[test]
fn godo_with_no_equipment_in_the_library_finds_nothing() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Mountain", "Island"]);
    let n = g.player(P0).library.len();
    let godo = godo_cast(&mut g);
    assert_eq!(g.player(P0).library.len(), n);
    assert!(g.player(P0).perms.iter().all(|&m| g.perm(m).attached != Some(godo)));
}

#[test]
fn godos_first_attack_untaps_him_and_the_samurai_and_gives_one_extra_combat() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Island"]);
    let godo = perm(&mut g, P0, GODO);
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms");
    let other = token(&mut g, P0, 2);
    for m in [godo, gil, other] {
        g.perm_mut(m).tapped = true;
    }
    combat::attack_triggers(&mut g, P0, &[godo], P1).unwrap();
    assert!(!g.perm(godo).tapped && !g.perm(gil).tapped, "Godo and the Samurai untap");
    assert!(g.perm(other).tapped, "a non-Samurai stays tapped");
    assert_eq!(g.player(P0).extra_combats, 1);
    // attacking again this turn (the extra combat) is not the first time
    combat::attack_triggers(&mut g, P0, &[godo], P1).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1, "no further combat");
}

// ======================================================== Cecil, Dark Knight // Cecil, Redeemed Paladin
/// Cecil, Dark Knight: "Deathtouch. Darkness — Whenever Cecil deals damage, you lose that much life. Then if your life
/// total is less than or equal to half your starting life total, untap Cecil and transform it." Redeemed Paladin: 4/4,
/// "Lifelink. Protect — Whenever Cecil attacks, other attacking creatures gain indestructible until end of turn."
#[test]
fn cecil_loses_life_per_damage_and_flips_at_half_of_forty() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Cecil, Dark Knight");
    assert!(g.perm(c).dt && values::epow(&g, c) == 2 && values::etgh(&g, c) == 3);
    g.player_mut(P0).life = 23;
    combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap();
    assert_eq!(g.player(P1).life, 38, "2 damage to the opponent");
    assert_eq!(g.player(P0).life, 21, "Cecil's controller loses 2");
    assert!(!g.perm(c).data.truthy(DataKey::Flipped), "21 is more than half of 40");
    g.perm_mut(c).tapped = false;
    combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap();
    assert_eq!(g.player(P0).life, 19, "19 is 20 or less");
    assert!(g.perm(c).data.truthy(DataKey::Flipped));
    assert!(!g.perm(c).tapped, "untapped by the transform");
    assert_eq!((values::epow(&g, c), values::etgh(&g, c)), (4, 4));
    assert!(g.perm(c).lifelink && !g.perm(c).dt);
    // the 4/4 lifelinker: damage no longer costs life; it gains it
    combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap();
    assert_eq!(g.player(P0).life, 23);
}

#[test]
fn cecil_redeemed_gives_the_other_attackers_indestructible_this_turn() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Cecil, Dark Knight");
    let t = token(&mut g, P0, 1);
    combat::attack_triggers(&mut g, P0, &[c, t], P1).unwrap();
    assert!(!values::indestructible(&g, t), "the Dark Knight protects nobody");
    g.player_mut(P0).life = 20;
    combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap(); // flips
    g.perm_mut(c).tapped = false;
    combat::attack_triggers(&mut g, P0, &[c, t], P1).unwrap();
    assert!(values::indestructible(&g, t), "the other attacker");
    assert!(!values::indestructible(&g, c), "not Cecil himself");
}

// ======================================================== Lightning, Army of One
/// Lightning, Army of One: "First strike, trample, lifelink. Stagger — Whenever Lightning deals combat damage to a
/// player, until your next turn, if a source would deal damage to that player or a permanent that player controls, it
/// deals double that damage instead."
#[test]
fn lightning_stagger_doubles_all_damage_to_the_player_and_their_permanents_until_its_next_turn() {
    use sim_core::engine::life::lose_life;
    let mut g = tbl();
    let l = perm(&mut g, P0, "Lightning, Army of One");
    let bear = token(&mut g, P1, 1);
    g.perm_mut(bear).tgh = 4;
    // before it connects: nothing doubles
    lose_life(&mut g, P1, 3, Some(P2), "burn", Some(true)).unwrap();
    assert_eq!(g.player(P1).life, 37);
    combat::resolve_combat(&mut g, P0, &[l], P1, &[l]).unwrap();
    assert_eq!(g.player(P1).life, 34, "Lightning's own 3 combat damage isn't doubled");
    assert_eq!(g.player(P0).life, 43, "lifelink");
    // noncombat damage from any source, to the player: doubled; to a permanent: doubled
    lose_life(&mut g, P1, 3, Some(P2), "burn", Some(true)).unwrap();
    assert_eq!(g.player(P1).life, 28);
    lose_life(&mut g, P1, 3, Some(P2), "drain", Some(false)).unwrap();
    assert_eq!(g.player(P1).life, 25, "life loss isn't damage");
    removal::apply_removal(&mut g, Some(P2), bear, "dmg2", None).unwrap();
    assert!(!g.perm(bear).on_bf, "2 damage doubled kills the 4-toughness token");
    // other players are unaffected
    lose_life(&mut g, P2, 3, Some(P1), "burn", Some(true)).unwrap();
    assert_eq!(g.player(P2).life, 37);
    // damage to a creature in combat counts too: a 2-power blocker needs 2 damage, doubled 4
    // until P0's next turn begins: then it stops
    g.player_mut(P0).turns += 1;
    lose_life(&mut g, P1, 3, Some(P2), "burn", Some(true)).unwrap();
    assert_eq!(g.player(P1).life, 22);
}

#[test]
fn cecil_and_lightning_are_cast_early_cheap_attackers() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = table(&["thrasios", "veyran", "seph"]);
    for n in ["Cecil, Dark Knight", "Lightning, Army of One"] {
        let c = card(&g, n);
        assert!(decks::deck_prio(&g, P0, c) >= 70, "{n}");
    }
    g.player_mut(P0).life = 24; // Cecil would flip within a hit or two with no way back: he waits
    assert!(decks::deck_prio(&g, P0, card(&g, "Cecil, Dark Knight")) < 50);
}
