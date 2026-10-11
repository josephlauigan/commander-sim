//! The gix candidate deck's cards (impls/gix.rs), on hand-built positions. Seat 0 plays the gix deck, seat 1 the seph
//! deck.

use sim_core::ai::{act, brain};
use sim_core::engine::combat::{can_block, kw};
use sim_core::engine::mana::{can_pay, pay};
use sim_core::engine::removal::legal_targets;
use sim_core::engine::values::epow;
use sim_core::engine::{cast, combat, life, turn, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["gix", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn has_option(g: &mut Game, label: &str) -> bool {
    options(g, P0, false).iter().any(|o| o.label.contains(label))
}

fn take_option(g: &mut Game, label: &str) -> bool {
    let opts = options(g, P0, false);
    let o = opts
        .iter()
        .find(|o| o.label.contains(label))
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn cast_at(g: &mut Game, name: &str, target: Option<PermId>) {
    let c = hand(g, P0, &[name])[0];
    cast::cast_card(g, P0, c, "hand", Ctx { target, ..Ctx::default() }).unwrap();
}

fn kill(g: &mut Game, m: PermId) {
    zones::die(g, m, "destroy".into()).unwrap();
}

fn size(g: &Game, m: PermId) -> (i32, i32) {
    (epow(g, m), sim_core::engine::values::etgh(g, m))
}

// ------------------------------------------------------------------ Al Bhed Salvagers
#[test]
fn salvagers_drain_when_a_creature_or_artifact_dies() {
    let mut g = game();
    perm(&mut g, P0, "Al Bhed Salvagers");
    let t = token(&mut g, P0, 1);
    kill(&mut g, t);
    assert_eq!((g.player(P0).life, g.player(P1).life), (41, 39));
    let k = perm(&mut g, P0, "Key to the Side-Door");
    kill(&mut g, k);
    assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38));
    // an opponent's creature dying does nothing
    let o = token(&mut g, P1, 1);
    kill(&mut g, o);
    assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38));
}

#[test]
fn salvagers_drain_when_they_die_and_when_a_clue_is_sacrificed() {
    let mut g = game();
    let s = perm(&mut g, P0, "Al Bhed Salvagers");
    g.player_mut(P0).clues = 1;
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(act::perform(&mut g, P0, &Action::Clue).unwrap());
    assert_eq!((g.player(P0).life, g.player(P1).life), (41, 39));
    kill(&mut g, s);
    assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38));
}

// ------------------------------------------------------------------ Necrosquito
#[test]
fn necrosquito_enters_with_two_oil_counters_and_grows() {
    let mut g = game();
    let n = perm(&mut g, P0, "Necrosquito");
    assert_eq!(size(&g, n), (2, 2));
    assert!(g.perm(n).on_bf);
    let a = token(&mut g, P0, 1);
    kill(&mut g, a);
    assert_eq!(size(&g, n), (3, 3));
    let o = token(&mut g, P1, 1);
    kill(&mut g, o);
    assert_eq!(size(&g, n), (3, 3));
    let k = perm(&mut g, P0, "Key to the Side-Door");
    kill(&mut g, k);
    assert_eq!(size(&g, n), (4, 4));
    g.player_mut(P0).clues = 1;
    lands(&mut g, P0, "Swamp", 2, false);
    act::perform(&mut g, P0, &Action::Clue).unwrap();
    assert_eq!(size(&g, n), (5, 5));
}

// ------------------------------------------------------------------ Callous Inspector, Apex Witchstalker
#[test]
fn callous_inspector_pings_you_and_makes_a_clue() {
    let mut g = game();
    let m = perm(&mut g, P0, "Callous Inspector");
    assert!(kw(&g, m, "menace"));
    kill(&mut g, m);
    assert_eq!(g.player(P0).life, 39);
    assert_eq!(g.player(P0).clues, 1);
}

#[test]
fn apex_witchstalker_gains_life_entering_and_dying() {
    let mut g = game();
    let m = perm(&mut g, P0, "Apex Witchstalker");
    assert_eq!(g.player(P0).life, 42);
    assert!(kw(&g, m, "menace"));
    kill(&mut g, m);
    assert_eq!(g.player(P0).life, 44);
}

#[test]
fn apex_witchstalker_basic_landcycles() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Apex Witchstalker"]);
    let before = g.player(P0).hand.len();
    assert!(take_option(&mut g, "basic landcycling"));
    let h = &g.player(P0).hand;
    assert_eq!(h.len(), before, "the card is swapped for a basic land");
    assert!(h.iter().any(|&c| g.db.get(c).land));
    assert!(!h.iter().any(|&c| &*g.db.get(c).name == "Apex Witchstalker"));
}

// ------------------------------------------------------------------ Corpulent Corpse
#[test]
fn corpulent_corpse_has_fear() {
    let mut g = game();
    let c = perm(&mut g, P0, "Corpulent Corpse");
    let red = perm(&mut g, P1, "Goblin Arsonist");
    let black = perm(&mut g, P1, "Burglar Rat");
    let art = perm(&mut g, P1, "Salvaged Manaworker");
    assert!(!can_block(&g, red, c));
    assert!(can_block(&g, black, c));
    assert!(can_block(&g, art, c));
}

#[test]
fn corpulent_corpse_suspends_and_comes_out_hasty_on_the_fifth_upkeep() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let c = hand(&mut g, P0, &["Corpulent Corpse"])[0];
    assert!(take_option(&mut g, "suspend Corpulent Corpse"));
    assert!(g.player(P0).exile.contains(&c));
    assert_eq!(g.player(P0).suspended, vec![(c, 5)]);
    g.step = Step::Start;
    for i in 1..=4 {
        turn::upkeep(&mut g, P0).unwrap();
        assert!(g.player(P0).perms.iter().all(|&m| &*g.perm(m).name != "Corpulent Corpse"), "upkeep {i}");
    }
    turn::upkeep(&mut g, P0).unwrap();
    let m = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == "Corpulent Corpse").expect("it is cast");
    assert!(!g.perm(m).sick, "haste");
    assert!(g.player(P0).suspended.is_empty());
}

// ------------------------------------------------------------------ Dark Matter Manipulator, Gravblade Heavy, Ulna Alley Shopkeep
#[test]
fn dark_matter_manipulator_mills_and_grows_with_the_graveyard() {
    let mut g = game();
    let m = perm(&mut g, P0, "Dark Matter Manipulator");
    assert_eq!(g.player(P0).gy.len(), 3);
    assert_eq!(size(&g, m), (1, 2));
    let more: Vec<_> = (0..4).map(|_| card(&g, "Swamp")).collect();
    g.player_mut(P0).gy.extend(more);
    assert_eq!(g.player(P0).gy.len(), 7);
    assert_eq!(size(&g, m), (3, 2));
    let more: Vec<_> = (0..7).map(|_| card(&g, "Swamp")).collect();
    g.player_mut(P0).gy.extend(more);
    assert_eq!(size(&g, m), (5, 2));
}

#[test]
fn gravblade_heavy_needs_an_artifact() {
    let mut g = game();
    let m = perm(&mut g, P0, "Gravblade Heavy");
    life::check_state(&mut g).unwrap();
    assert_eq!(size(&g, m), (3, 4));
    assert!(!g.perm(m).dt);
    g.player_mut(P0).clues = 1; // a Clue is an artifact
    life::check_state(&mut g).unwrap();
    assert_eq!(size(&g, m), (4, 4));
    assert!(g.perm(m).dt, "deathtouch");
    g.player_mut(P0).clues = 0;
    life::check_state(&mut g).unwrap();
    assert!(!g.perm(m).dt);
    assert_eq!(size(&g, m), (3, 4));
    perm(&mut g, P0, "Mask of Memory");
    assert_eq!(size(&g, m), (4, 4));
}

#[test]
fn ulna_alley_shopkeep_is_infused() {
    let mut g = game();
    let m = perm(&mut g, P0, "Ulna Alley Shopkeep");
    assert!(kw(&g, m, "menace"));
    assert_eq!(size(&g, m), (2, 3));
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(size(&g, m), (4, 3));
}

// ------------------------------------------------------------------ Gollum, Grim Wanderer
#[test]
fn gollums_meager_meal_adds_a_counter_and_two_life_and_keeps_the_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let a = perm(&mut g, P0, "Rank Rat");
    let c = hand(&mut g, P0, &[GOLLUM_NAME])[0];
    let before = size(&g, a);
    assert!(take_option(&mut g, "Meager Meal"));
    assert_eq!(size(&g, a), (before.0 + 1, before.1 + 1));
    assert_eq!(g.player(P0).life, 42);
    assert!(g.player(P0).hand.contains(&c), "the creature is cast later");
    assert!(!has_option(&mut g, "Meager Meal"), "once");
}

const GOLLUM_NAME: &str = "Gollum, Silent Slinker";

#[test]
fn grim_wanderer_waits_for_a_death_this_turn() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Grim Wanderer"]);
    assert!(!has_option(&mut g, "Grim Wanderer"));
    let t = token(&mut g, P1, 1);
    kill(&mut g, t);
    assert!(has_option(&mut g, "Grim Wanderer"));
}

// ------------------------------------------------------------------ Restless Bloodseeker
#[test]
fn restless_bloodseeker_makes_blood_flips_and_drains() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 6, false);
    let m = perm(&mut g, P0, "Restless Bloodseeker");
    assert_eq!(size(&g, m), (1, 3));
    // no life gained: no Blood
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).bloods, 0);
    life::gain(&mut g, P0, 1).unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).bloods, 1);
    assert!(!has_option(&mut g, "transform"));
    g.player_mut(P0).bloods = 2;
    assert!(take_option(&mut g, "transform"));
    assert_eq!(g.player(P0).bloods, 0);
    assert!(g.perm(m).data.truthy(DataKey::Flipped));
    assert_eq!(size(&g, m), (3, 3));
    let before = g.player(P1).life;
    assert!(take_option(&mut g, "Bloodsoaked Reveler"));
    assert_eq!((g.player(P1).life, g.player(P0).life), (before - 2, 43));
}

#[test]
fn blood_rummage_draws_for_a_junk_card() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    perm(&mut g, P0, "Restless Bloodseeker");
    g.player_mut(P0).bloods = 1;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Swamp"]);
    lands(&mut g, P0, "Swamp", 5, false);
    let lib = g.player(P0).library.len();
    assert!(take_option(&mut g, "rummage"));
    assert_eq!(g.player(P0).bloods, 0);
    assert_eq!(g.player(P0).library.len(), lib - 1);
    assert_eq!(g.player(P0).hand.len(), 1);
}

// ------------------------------------------------------------------ Salvaged Manaworker
#[test]
fn salvaged_manaworker_filters_one_mana_to_any_colour_once_a_turn() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    assert!(!can_pay(&g, P0, 0, "B", false));
    perm(&mut g, P0, "Salvaged Manaworker");
    assert!(can_pay(&g, P0, 0, "B", false));
    assert!(can_pay(&g, P0, 2, "B", false), "the filtered mana is one of the three");
    assert!(!can_pay(&g, P0, 3, "B", false), "it adds no mana");
    assert!(pay(&mut g, P0, 1, "B", false).unwrap());
    assert!(!can_pay(&g, P0, 0, "B", false), "once each turn (and two Mountains are tapped)");
}

// ------------------------------------------------------------------ Sangrophage, Deepcavern Imp
#[test]
fn sangrophage_taps_unless_you_pay_two_life() {
    let mut g = game();
    let m = perm(&mut g, P0, "Sangrophage");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 38);
    assert!(!g.perm(m).tapped);
    g.player_mut(P0).life = 10;
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 10);
    assert!(g.perm(m).tapped);
}

#[test]
fn deepcavern_imp_echo_discards_or_sacrifices() {
    let mut g = game();
    let m = perm(&mut g, P0, "Deepcavern Imp");
    assert!(kw(&g, m, "flying") && kw(&g, m, "haste"));
    hand(&mut g, P0, &["Swamp"]);
    let n = g.player(P0).hand.len();
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.perm(m).on_bf);
    assert_eq!(g.player(P0).hand.len(), n - 1, "a card discarded");
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.perm(m).on_bf, "echo is paid once");
    // no card to discard: sacrificed
    let mut g = game();
    let m = perm(&mut g, P0, "Deepcavern Imp");
    g.player_mut(P0).hand.clear();
    turn::upkeep(&mut g, P0).unwrap();
    assert!(!g.perm(m).on_bf);
}

// ------------------------------------------------------------------ Balustrade Spy
#[test]
fn balustrade_spy_mills_until_a_land() {
    let mut g = game();
    // top of the library is the end of the vector: two spells, a land, then more
    let names = ["Swamp", "Terror", "Terror"];
    let cs: Vec<_> = names.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P1).library.extend(cs);
    let lib = g.player(P1).library.len();
    let gy = g.player(P1).gy.len();
    perm(&mut g, P0, "Balustrade Spy");
    assert_eq!(g.player(P1).library.len(), lib - 3);
    assert_eq!(g.player(P1).gy.len(), gy + 3);
}

// ------------------------------------------------------------------ Key to the Side-Door
#[test]
fn key_to_the_side_door_makes_the_best_attacker_unblockable() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let k = perm(&mut g, P0, "Key to the Side-Door");
    let a = perm(&mut g, P0, "Sangrophage");
    let b = perm(&mut g, P1, "Burglar Rat");
    assert!(can_block(&g, b, a));
    assert!(take_option(&mut g, "Key to the Side-Door"));
    assert!(g.perm(k).tapped);
    assert!(!can_block(&g, b, a));
}

// ------------------------------------------------------------------ the spells
#[test]
fn overkill_kills_anything() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let t = perm(&mut g, P1, "Grave Titan");
    cast_at(&mut g, "Overkill", Some(t));
    assert!(!g.perm(t).on_bf);
}

#[test]
fn depressurize_kills_a_creature_of_power_three_or_less() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let big = perm(&mut g, P1, "Grave Titan");
    let small = perm(&mut g, P1, "Burglar Rat");
    let c = card(&g, "Depressurize");
    let tg = legal_targets(&g, P0, "destroy", "c", false, Some(c));
    assert!(tg.contains(&small) && !tg.contains(&big));
    cast_at(&mut g, "Depressurize", Some(small));
    assert!(!g.perm(small).on_bf);
    assert!(g.perm(big).on_bf);
}

#[test]
fn end_of_the_hunt_exiles_the_greatest_mana_value() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let big = perm(&mut g, P1, "Grave Titan");
    let small = perm(&mut g, P1, "Burglar Rat");
    cast_at(&mut g, "End of the Hunt", None);
    assert!(!g.perm(big).on_bf);
    assert!(g.perm(small).on_bf);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Titan"));
}

// ------------------------------------------------------------------ equipment
#[test]
fn medics_kitesail_gives_flying_a_point_of_power_and_life() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let e = perm(&mut g, P0, "Medic's Kitesail");
    let a = perm(&mut g, P0, "Sangrophage");
    let base = size(&g, a);
    assert!(sim_core::dsl::equip(&mut g, P0, e, a, 2).unwrap());
    assert_eq!(size(&g, a), (base.0 + 1, base.1));
    assert!(kw(&g, a, "flying"));
    let b = perm(&mut g, P1, "Burglar Rat");
    assert!(!can_block(&g, b, a));
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn mask_of_memory_loots_two_on_a_hit_and_prefers_an_evasive_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let e = perm(&mut g, P0, "Mask of Memory");
    let ground = perm(&mut g, P0, "Rank Rat");
    let flier = perm(&mut g, P0, "Balustrade Spy");
    let opts = sim_core::dsl::equip_options(&g, P0);
    let o = opts.iter().find(|o| o.label.contains("Mask of Memory")).expect("equip option");
    match &o.act {
        Some(Action::Equip { target, .. }) => assert_eq!(*target, flier),
        other => panic!("{other:?}"),
    }
    let _ = ground;
    assert!(sim_core::dsl::equip(&mut g, P0, e, flier, 1).unwrap());
    let (h, gy) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    combat::resolve_combat(&mut g, P0, &[flier], P1, &[]).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 1, "two drawn, one discarded");
    assert_eq!(g.player(P0).gy.len(), gy + 1);
}

// ------------------------------------------------------------------ Gix
#[test]
fn gix_draws_for_a_connecting_creature_only_with_comfortable_life() {
    let mut g = game();
    perm(&mut g, P0, "Gix, Yawgmoth Praetor");
    let a = perm(&mut g, P0, "Sangrophage");
    let h = g.player(P0).hand.len();
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (h + 1, 39));
    g.player_mut(P0).life = 12;
    g.perm_mut(a).tapped = false;
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 1, "no draw at 12 life");
}

#[test]
fn gix_ability_waits_for_seven_lands() {
    let mut g = game();
    perm(&mut g, P0, "Gix, Yawgmoth Praetor");
    lands(&mut g, P0, "Swamp", 6, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Swamp", "Swamp", "Swamp"]);
    assert!(!has_option(&mut g, "Gix (discard X)"));
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(has_option(&mut g, "Gix (discard X)"));
    let _ = Val::None;
}

// ------------------------------------------------------------------ the AI plan
#[test]
fn evasive_creatures_come_first_and_equipment_waits_for_a_creature() {
    let mut g = game();
    let spy = card(&g, "Balustrade Spy");
    let vanilla = card(&g, "Rank Rat");
    let kite = card(&g, "Medic's Kitesail");
    let p = |g: &Game, c| sim_core::ai::decks::deck_prio(g, P0, c);
    assert!(p(&g, spy) > p(&g, vanilla));
    assert_eq!(p(&g, kite), 0);
    perm(&mut g, P0, "Rank Rat");
    assert!(p(&g, kite) >= 46);
}
