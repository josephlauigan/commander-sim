//! Tier 6 batch `green2` (impls/t6_green2.rs) on hand-built positions. Seat 0 plays the Rograkh & Thrasios deck,
//! seat 1 the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t6_green2 as t6;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-thrasios-creature-combo", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn squirrels(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).pow == 1).count()
}

// ======================================================== Chatterstorm
#[test]
fn chatterstorm_makes_a_squirrel_and_a_copy_for_each_earlier_spell() {
    let mut g = game();
    cast_spell(&mut g, P0, "Opt");
    cast_spell(&mut g, P0, "Preordain");
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 3, "the Squirrel and two copies' Squirrels");
    assert!(in_gy(&g, P0, "Chatterstorm"));
}

#[test]
fn chatterstorm_alone_makes_one_squirrel() {
    let mut g = game();
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 1);
}

// ======================================================== Cloud of Faeries
fn untapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

#[test]
fn cloud_of_faeries_untaps_two_lands_when_it_enters() {
    let mut g = game();
    lands(&mut g, P0, "Island", 4, false);
    cast_spell(&mut g, P0, "Cloud of Faeries");
    let cloud = on_bf(&g, P0, "Cloud of Faeries").expect("on the battlefield");
    assert!(g.perm(cloud).fly, "flying");
    // {1}{U} was paid, then two lands untapped: 4 - 2 + 2
    assert_eq!(untapped(&g, P0), 4);
}

#[test]
fn cloud_of_faeries_untaps_only_the_tapped_lands_it_has() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Forest", 1, true);
    perm(&mut g, P0, "Cloud of Faeries"); // not cast: its enters trigger still runs
    assert_eq!(untapped(&g, P0), 3, "the one tapped land untaps");
}

#[test]
fn cloud_of_faeries_cycles_for_two() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Cloud of Faeries"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("cycling option");
    let opts = f(&mut g, c, P0, None).unwrap();
    assert_eq!(opts.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert!(in_gy(&g, P0, "Cloud of Faeries"));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 1, "discarded it and drew a card");
    assert_eq!(untapped(&g, P0), 1, "{{2}} paid");
}

#[test]
fn cloud_of_faeries_cannot_cycle_without_two_mana() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 1, false);
    let c = hand(&mut g, P0, &["Cloud of Faeries"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    assert!(f(&mut g, c, P0, None).unwrap().is_empty());
}

// ======================================================== Noxious Revival
fn top_is(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).library.last().is_some_and(|&c| &*g.db.get(c).name == name)
}

#[test]
fn noxious_revival_puts_the_best_graveyard_card_on_top_of_the_library() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Oboro Breezecaller", "Noxious Revival"]);
    sim_core::engine::zones::discard_cards(&mut g, P0, &[c[0]]).unwrap();
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, c[1], "hand", Ctx::default()).unwrap();
    assert!(top_is(&g, P0, "Oboro Breezecaller"), "on top of its owner's library");
    assert_eq!(g.player(P0).library.len(), lib + 1);
    assert!(!in_gy(&g, P0, "Oboro Breezecaller"));
    assert!(in_gy(&g, P0, "Noxious Revival"));
}

#[test]
fn noxious_revival_costs_g_or_two_life() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Noxious Revival"])[0];
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c).0, 0);
    assert_eq!(sim_core::engine::mana::phyrexian_life(&g, P0, c, ""), 2, "unpaid {{G/P}} is 2 life");
    assert_eq!(sim_core::engine::mana::phyrexian_life(&g, P0, c, "G"), 0);
}

#[test]
fn noxious_revival_with_an_empty_graveyard_just_goes_to_the_graveyard() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Noxious Revival"])[0];
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).library.len(), lib);
    assert!(in_gy(&g, P0, "Noxious Revival"));
}

#[test]
fn noxious_revival_never_picks_itself_so_valuing_it_in_the_graveyard_ends() {
    // its priority asks what it would pick; a Noxious Revival in the graveyard used to value itself through that same
    // question without end (a stack overflow in Tier 6 games)
    let mut g = game();
    let c = hand(&mut g, P0, &["Noxious Revival"]);
    sim_core::engine::zones::discard_cards(&mut g, P0, &c).unwrap();
    assert_eq!(t6::revival_pick(&g, P0), None);
    let _ = sim_core::engine::tutors::card_worth(&g, P0, c[0], true);
}

// ======================================================== Sowing Mycospawn
fn lib_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).library.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn mycospawn_cast_fetches_a_land_onto_the_battlefield() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Sowing Mycospawn"])[0];
    let lib_lands = g.player(P0).library.iter().filter(|&&x| g.db.get(x).land).count();
    assert!(lib_lands > 0);
    let n = g.player(P0).lands.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).lands.len() > n, "a land came from the library");
    assert!(g.player(P0).library.iter().filter(|&&x| g.db.get(x).land).count() < lib_lands);
    assert!(on_bf(&g, P0, "Sowing Mycospawn").is_some(), "the 3/3 resolves too");
}

#[test]
fn mycospawn_prefers_gaea_s_cradle_with_creatures_out() {
    let mut g = game();
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Birds of Paradise");
    if !lib_has(&g, P0, "Gaea's Cradle") {
        let c = card(&g, "Gaea's Cradle");
        g.player_mut(P0).library.push(c);
    }
    let cr = card(&g, "Gaea's Cradle");
    assert_eq!(t6::mycospawn_pick(&g, P0), Some(cr));
}

#[test]
fn mycospawn_kicked_exiles_the_best_opposing_land() {
    let mut g = game();
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P1, "Command Tower", 1, false);
    t6::mycospawn_kicked(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).lands.len(), 2);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Command Tower"));
}

// ======================================================== Endurance
fn fill_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for c in hand(g, p, names) {
        sim_core::engine::zones::discard_cards(g, p, &[c]).unwrap();
    }
}

#[test]
fn endurance_is_a_flash_reach_body() {
    let g = game();
    let d = g.db.get(card(&g, "Endurance"));
    assert!(d.tag(sim_core::tag::Tag::Flash) && d.tag(sim_core::tag::Tag::Reach));
    assert_eq!((d.pow, d.tgh), (3, 4));
}

#[test]
fn endurance_puts_an_opponent_graveyard_on_the_bottom_of_their_library() {
    let mut g = game();
    fill_gy(&mut g, P1, &["Swords to Plowshares", "Opt", "Ponder", "Brainstorm", "Preordain"]);
    let c = hand(&mut g, P0, &["Endurance"])[0];
    let lib = g.player(P1).library.len();
    let bottom: Vec<_> = g.player(P1).library.iter().take(2).copied().collect();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(on_bf(&g, P0, "Endurance").is_some());
    assert!(g.player(P1).gy.is_empty(), "graveyard emptied");
    assert_eq!(g.player(P1).library.len(), lib + 5);
    assert_eq!(&g.player(P1).library[5..7], &bottom[..], "the old library sits above the five cards");
}

#[test]
fn endurance_aims_at_its_own_graveyard_with_faerie_mastermind() {
    let mut g = game();
    perm(&mut g, P0, "Faerie Mastermind");
    fill_gy(&mut g, P0, &["Opt", "Ponder"]);
    fill_gy(&mut g, P1, &["Opt", "Ponder", "Brainstorm", "Preordain", "Swords to Plowshares"]);
    let c = hand(&mut g, P0, &["Endurance"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).gy.is_empty());
    assert_eq!(g.player(P1).gy.len(), 5, "the opponent's stays");
}

#[test]
fn endurance_with_small_graveyards_does_nothing() {
    let mut g = game();
    fill_gy(&mut g, P1, &["Opt"]);
    let c = hand(&mut g, P0, &["Endurance"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P1).gy.len(), 1);
}

#[test]
fn endurance_evoke_exiles_a_green_card_and_is_sacrificed() {
    let mut g = game();
    fill_gy(&mut g, P1, &["Swords to Plowshares", "Opt", "Ponder", "Brainstorm", "Preordain"]);
    let h = hand(&mut g, P0, &["Endurance", "Noxious Revival"]);
    let f = g.registry.get(h[0]).and_then(|i| i.hand_options).expect("evoke option");
    let opts = f(&mut g, h[0], P0, None).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Noxious Revival"));
    assert!(on_bf(&g, P0, "Endurance").is_none(), "sacrificed");
    assert!(in_gy(&g, P0, "Endurance"));
    assert!(g.player(P1).gy.is_empty(), "the enters effect still happened");
}

// ======================================================== Enduring Vitality
fn mana_units(g: &Game, p: PlayerId) -> Vec<sim_core::engine::mana::Unit> {
    sim_core::engine::mana::mana_units(g, p, false)
}

#[test]
fn vitality_lets_each_creature_tap_for_any_colour() {
    let mut g = game();
    let v = perm(&mut g, P0, "Enduring Vitality");
    let bird = perm(&mut g, P0, "Eternal Witness"); // no mana ability of its own
    g.perm_mut(v).sick = false;
    g.perm_mut(bird).sick = false;
    let u = mana_units(&g, P0);
    let from = |m: PermId| u.iter().find(|w| w.src == sim_core::engine::mana::Source::Perm(m));
    assert!(from(bird).is_some(), "Eternal Witness taps for mana");
    assert!(from(v).is_some(), "so does Vitality itself");
    assert!(from(bird).unwrap().cols.has('W') && from(bird).unwrap().cols.has('U'), "any colour");
    g.perm_mut(bird).sick = true;
    assert!(mana_units(&g, P0).iter().all(|w| w.src != sim_core::engine::mana::Source::Perm(bird)), "summoning sick");
}

#[test]
fn vitality_pays_for_a_spell() {
    let mut g = game();
    let v = perm(&mut g, P0, "Enduring Vitality");
    g.perm_mut(v).sick = false;
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "U", false), "a blue pip from the creature's mana");
}

#[test]
fn vitality_returns_as_an_enchantment_when_it_dies_and_still_gives_the_mana() {
    let mut g = game();
    let v = perm(&mut g, P0, "Enduring Vitality");
    let w = perm(&mut g, P0, "Eternal Witness");
    g.perm_mut(w).sick = false;
    sim_core::engine::zones::die(&mut g, v, "destroy").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    let n = on_bf(&g, P0, "Enduring Vitality").expect("it came back");
    g.perm_mut(n).sick = false; // even unsick, a non-creature cannot tap for mana
    assert_ne!(n, v);
    assert!(!in_gy(&g, P0, "Enduring Vitality"));
    assert!(g.perm(n).data.truthy(sim_core::state::DataKey::Final), "marked as the enchantment");
    let u = mana_units(&g, P0);
    assert!(u.iter().any(|x| x.src == sim_core::engine::mana::Source::Perm(w)), "creatures still tap for mana");
    assert!(u.iter().all(|x| x.src != sim_core::engine::mana::Source::Perm(n)), "the enchantment is not a creature");
    // it does not return a second time
    sim_core::engine::zones::die(&mut g, n, "destroy").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(on_bf(&g, P0, "Enduring Vitality").is_none());
    assert!(in_gy(&g, P0, "Enduring Vitality"));
}

// ======================================================== The Cabbage Merchant
#[test]
fn merchant_makes_a_food_when_an_opponent_casts_a_noncreature_spell() {
    let mut g = game();
    perm(&mut g, P0, "The Cabbage Merchant");
    cast_spell(&mut g, P1, "Opt");
    assert_eq!(g.player(P0).foods, 1);
    cast_spell(&mut g, P1, "Grave Titan"); // a creature spell: no Food
    assert_eq!(g.player(P0).foods, 1);
    cast_spell(&mut g, P0, "Ponder"); // our own spell: no Food
    assert_eq!(g.player(P0).foods, 1);
}

#[test]
fn merchant_sacrifices_a_food_when_a_creature_deals_combat_damage_to_us() {
    let mut g = game();
    let m = perm(&mut g, P0, "The Cabbage Merchant");
    g.player_mut(P0).foods = 2;
    let a = perm(&mut g, P1, "Grave Titan");
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.combat_damage).unwrap();
    f(&mut g, m, P1, a, P0, 6).unwrap();
    assert_eq!(g.player(P0).foods, 1);
    f(&mut g, m, P1, a, P1, 6).unwrap(); // damage to someone else: nothing
    assert_eq!(g.player(P0).foods, 1);
}

#[test]
fn merchant_taps_two_foods_for_one_mana_of_any_colour() {
    let mut g = game();
    let m = perm(&mut g, P0, "The Cabbage Merchant");
    g.player_mut(P0).foods = 1;
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "U", false), "one Food is not enough");
    g.player_mut(P0).foods = 4;
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "U", false));
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "UW", false), "two pairs: two mana");
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "UWB", false));
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "UW", false).unwrap());
    assert_eq!(g.player(P0).foods, 4, "tapped, not sacrificed");
    assert!(!g.perm(m).tapped, "the ability does not tap the Merchant");
    assert_eq!(t6::foods_untapped(&g, m), 0);
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "U", false), "no untapped Foods left");
}

#[test]
fn merchant_foods_untap_at_its_controllers_upkeep() {
    let mut g = game();
    let m = perm(&mut g, P0, "The Cabbage Merchant");
    g.player_mut(P0).foods = 2;
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert_eq!(t6::foods_untapped(&g, m), 0);
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.upkeep).unwrap();
    f(&mut g, m, P1).unwrap();
    assert_eq!(t6::foods_untapped(&g, m), 0, "the opponent's upkeep does not untap them");
    f(&mut g, m, P0).unwrap();
    assert_eq!(t6::foods_untapped(&g, m), 2);
}

// ======================================================== Shifting Woodland
use sim_core::ids::LandId;

fn woodland(g: &mut Game, p: PlayerId) -> LandId {
    let c = card(g, "Shifting Woodland");
    g.add_land(p, c, false)
}

#[test]
fn woodland_enters_tapped_unless_you_control_a_forest() {
    let mut g = game();
    let c = card(&g, "Shifting Woodland");
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c), "no Forest: tapped");
    lands(&mut g, P0, "Island", 2, false);
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c), "Islands don't count");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!sim_core::engine::turn::land_enters_tapped(&g, P0, c), "with a Forest: untapped");
}

#[test]
fn woodland_taps_for_green() {
    let mut g = game();
    woodland(&mut g, P0);
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "G", false));
}

#[test]
fn delirium_needs_four_card_types_in_the_graveyard() {
    let mut g = game();
    fill_gy(&mut g, P0, &["Opt", "Eternal Witness", "Sol Ring"]);
    assert!(!t6::delirium(&g, P0), "instant, creature, artifact");
    fill_gy(&mut g, P0, &["Forest"]);
    assert!(t6::delirium(&g, P0), "plus a land");
}

#[test]
fn woodland_becomes_a_copy_of_oboro_with_delirium() {
    let mut g = game();
    let l = woodland(&mut g, P0);
    lands(&mut g, P0, "Forest", 4, false);
    fill_gy(&mut g, P0, &["Oboro Breezecaller", "Opt", "Sol Ring", "Forest"]);
    let f = g.registry.get(g.land(l).cd).and_then(|i| i.land_options).expect("land option");
    let opts = f(&mut g, l, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    let copy = on_bf(&g, P0, "Oboro Breezecaller").expect("a copy of Oboro");
    assert!(g.perm(copy).temp && !g.perm(copy).sick, "until end of turn, and not summoning sick");
    assert!(g.land(l).tapped, "the land is the creature: no mana from it as a land");
    assert!(in_gy(&g, P0, "Oboro Breezecaller"), "the card stays in the graveyard");
}

#[test]
fn woodland_does_nothing_without_delirium() {
    let mut g = game();
    let l = woodland(&mut g, P0);
    lands(&mut g, P0, "Forest", 4, false);
    fill_gy(&mut g, P0, &["Oboro Breezecaller", "Opt"]);
    let f = g.registry.get(g.land(l).cd).and_then(|i| i.land_options).unwrap();
    assert!(f(&mut g, l, P0, Some(false)).unwrap().is_empty());
}

// ======================================================== Invasion of Ikoria // Zilortha, Apex of Ikoria
const IKORIA: &str = "Invasion of Ikoria // Zilortha, Apex of Ikoria";

fn ikoria_cast(g: &mut Game, x: i32) {
    let c = hand(g, P0, &[IKORIA])[0];
    cast::cast_card(g, P0, c, "hand", Ctx { x, ..Ctx::default() }).unwrap();
}

/// a library with no creature cards in it
fn no_creatures(g: &mut Game) {
    let lib = std::mem::take(&mut g.player_mut(P0).library);
    let keep: Vec<_> = lib.into_iter().filter(|&c| !g.db.get(c).creature).collect();
    g.player_mut(P0).library = keep;
}

#[test]
fn ikoria_finds_a_non_human_creature_with_mana_value_x_or_less() {
    let mut g = game();
    no_creatures(&mut g);
    let h = hand(&mut g, P0, &["Oboro Breezecaller", "Eternal Witness", "Sol Ring"]);
    g.player_mut(P0).library.extend(&h[..2]);
    let ik = hand(&mut g, P0, &[IKORIA])[0];
    cast::cast_card(&mut g, P0, ik, "hand", Ctx { x: 3, ..Ctx::default() }).unwrap();
    assert!(on_bf(&g, P0, "Eternal Witness").is_none(), "a Human is not found");
    assert!(on_bf(&g, P0, "Oboro Breezecaller").is_some(), "the non-Human creature is");
    assert!(!g.player(P0).library.contains(&h[0]));
    assert!(g.player(P0).library.contains(&h[1]));
}

#[test]
fn ikoria_respects_the_mana_value_limit() {
    let mut g = game();
    no_creatures(&mut g);
    let h = hand(&mut g, P0, &["Oboro Breezecaller"]);
    g.player_mut(P0).library.extend(&h);
    ikoria_cast(&mut g, 1); // Oboro is mana value 2
    assert!(on_bf(&g, P0, "Oboro Breezecaller").is_none());
    assert!(in_gy(&g, P0, IKORIA));
}

#[test]
fn ikoria_can_take_the_creature_from_the_graveyard() {
    let mut g = game();
    no_creatures(&mut g);
    let gy = hand(&mut g, P0, &["Oboro Breezecaller"])[0];
    sim_core::engine::zones::discard_cards(&mut g, P0, &[gy]).unwrap();
    ikoria_cast(&mut g, 2);
    assert!(on_bf(&g, P0, "Oboro Breezecaller").is_some());
    assert!(!in_gy(&g, P0, "Oboro Breezecaller"));
    assert!(in_gy(&g, P0, IKORIA), "the battle is not modelled: the card goes to the graveyard");
}
