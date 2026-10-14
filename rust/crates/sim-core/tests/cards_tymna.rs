//! The Tymna decks' new cards (impls/tymna_cards.rs): each card on a hand-built position, with its Oracle text as the
//! expectation. Tymna the Weaver herself is tested in cards_legends.rs.

use sim_core::ai::{act, decks};
use sim_core::cards::Colors;
use sim_core::engine::{cast, combat, removal, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
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
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
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
    assert_eq!(g.player(P0).floating.r, 2, "2 damage: RR");
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
