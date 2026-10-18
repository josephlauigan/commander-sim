//! Tier 6 batch `copies` (impls/t6_copies.rs) on hand-built positions, each Oracle text as the expectation.
//! Seat 0 plays a stand-in deck (cards are keyed by name), seat 1 the seph deck.

use sim_core::engine::cast;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).name == name).collect()
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn cast_hand(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

/// the clone that entered as a copy: the permanent named like the target, whose physical card is the clone
fn copy_of(g: &Game, p: PlayerId, target: &str, clone: CardId) -> Option<PermId> {
    on_bf(g, p, target).into_iter().find(|&m| g.perm(m).phys == Some(clone))
}

#[test]
fn clever_impersonator_copies_the_best_nonland_permanent() {
    let mut g = game();
    token(&mut g, P1, 1);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P1, "Sol Ring");
    let c = cast_hand(&mut g, P0, "Clever Impersonator");
    assert!(on_bf(&g, P0, "Clever Impersonator").is_empty());
    let m = copy_of(&g, P0, "Grave Titan", c).expect("copy of Grave Titan");
    assert!(g.is_creature(m));
    // it dies as the clone card
    sim_core::engine::zones::die(&mut g, m, "dies").unwrap();
    assert!(gy_has(&g, P0, "Clever Impersonator"));
}

#[test]
fn clever_impersonator_copy_has_the_enters_effect() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    let before = g.player(P0).perms.len();
    cast_hand(&mut g, P0, "Clever Impersonator");
    // Grave Titan's enters trigger makes two Zombies: the copy, plus tokens
    assert!(g.player(P0).perms.len() > before + 1, "copy + zombie tokens");
}

#[test]
fn flesh_duplicate_and_phantasmal_image_copy_creatures_only() {
    for name in ["Flesh Duplicate", "Phantasmal Image"] {
        let mut g = game();
        perm(&mut g, P1, "Sol Ring");
        perm(&mut g, P1, "Mulldrifter");
        let c = cast_hand(&mut g, P0, name);
        assert!(copy_of(&g, P0, "Mulldrifter", c).is_some(), "{name} copies the creature");
        assert!(on_bf(&g, P0, "Sol Ring").is_empty());
    }
}

#[test]
fn copy_artifact_copies_an_artifact_not_a_creature() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P1, "Basalt Monolith");
    let c = cast_hand(&mut g, P0, "Copy Artifact");
    assert!(copy_of(&g, P0, "Basalt Monolith", c).is_some());
    assert!(on_bf(&g, P0, "Grave Titan").is_empty());
}

#[test]
fn sculpting_steel_copies_an_artifact() {
    let mut g = game();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Basalt Monolith");
    let c = cast_hand(&mut g, P0, "Sculpting Steel");
    assert!(copy_of(&g, P0, "Basalt Monolith", c).is_some());
}

#[test]
fn mirrormade_copies_an_enchantment_or_artifact() {
    let mut g = game();
    perm(&mut g, P1, "Rhystic Study");
    let c = cast_hand(&mut g, P0, "Mirrormade");
    assert!(copy_of(&g, P0, "Rhystic Study", c).is_some());
}

#[test]
fn a_clone_with_nothing_to_copy_is_not_cast_on_purpose() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Clever Impersonator"])[0];
    assert_eq!(sim_core::ai::pool::generic_prio(&g, P0, c), 0);
    perm(&mut g, P1, "Grave Titan");
    assert!(sim_core::ai::pool::generic_prio(&g, P0, c) > 0);
}

#[test]
fn mockingbird_copies_a_creature_within_the_mana_spent_and_flies() {
    let mut g = game();
    perm(&mut g, P1, "Mulldrifter"); // mana value 5
    perm(&mut g, P1, "Grave Titan"); // mana value 6: out of reach with 3 mana
    lands(&mut g, P0, "Island", 5, false);
    let c = hand(&mut g, P0, &["Mockingbird"])[0];
    let opts = opts_for(&mut g, c);
    assert!(!opts.is_empty(), "Mockingbird offers a cast");
    run_plan(&mut g, &opts[0]);
    let m = copy_of(&g, P0, "Mulldrifter", c).expect("a copy of Mulldrifter");
    assert!(g.perm(m).fly, "with flying");
    assert!(on_bf(&g, P0, "Grave Titan").is_empty(), "mana value 6 is more than the 5 mana spent");
}

fn opts_for(g: &mut Game, c: CardId) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    g.active = Some(P0);
    f(g, c, P0, Some(true)).unwrap()
}

fn run_plan(g: &mut Game, o: &Opt) {
    match &o.act {
        Some(Action::Plan { f, arg }) => assert!(f(g, P0, *arg).unwrap()),
        _ => panic!("not a plan"),
    }
}

#[test]
fn heat_shimmer_makes_a_hasty_token_copy_exiled_at_the_end_step() {
    let mut g = game();
    perm(&mut g, P1, "Mulldrifter");
    let before = g.player(P0).perms.len();
    cast_hand(&mut g, P0, "Heat Shimmer");
    let toks = on_bf(&g, P0, "Mulldrifter");
    assert_eq!(toks.len(), 1);
    let t = g.perm(toks[0]);
    assert!(t.token && !t.sick && t.temp, "a hasty token that is exiled at the end step");
    assert!(g.player(P0).perms.len() > before);
    assert!(gy_has(&g, P0, "Heat Shimmer"));
}

#[test]
fn molten_duplication_copies_only_your_artifact_or_creature_and_sacrifices_it() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    cast_hand(&mut g, P0, "Molten Duplication");
    assert!(on_bf(&g, P0, "Grave Titan").is_empty(), "an opponent's creature is not a legal target");
    let mut g = game();
    perm(&mut g, P0, "Mulldrifter");
    cast_hand(&mut g, P0, "Molten Duplication");
    let toks: Vec<PermId> = on_bf(&g, P0, "Mulldrifter").into_iter().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 1);
    assert!(!g.perm(toks[0]).sick);
    assert!(g.perm(toks[0]).data.truthy(DataKey::EotSac), "sacrificed at the end step");
}

#[test]
fn electroduplicate_has_flashback_and_copies_your_creature() {
    let mut g = game();
    let c = card(&g, "Electroduplicate");
    assert_eq!(g.db.get(c).tags.str(sim_core::tag::Tag::Fb), Some("2RR"));
    perm(&mut g, P0, "Mulldrifter");
    cast_hand(&mut g, P0, "Electroduplicate");
    assert_eq!(on_bf(&g, P0, "Mulldrifter").len(), 2);
}

#[test]
fn flash_photography_copies_any_permanent_and_stays() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    let c = card(&g, "Flash Photography");
    assert_eq!(g.db.get(c).tags.str(sim_core::tag::Tag::Fb), Some("4UU"));
    cast_hand(&mut g, P0, "Flash Photography");
    let toks = on_bf(&g, P0, "Grave Titan");
    assert_eq!(toks.len(), 1);
    assert!(g.perm(toks[0]).token && !g.perm(toks[0]).temp);
}

#[test]
fn flash_photography_has_flash_on_your_own_permanent() {
    let mut g = game();
    perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Island", 4, false);
    let c = hand(&mut g, P0, &["Flash Photography"])[0];
    g.active = Some(P1);
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let opts = f(&mut g, c, P0, None).unwrap();
    assert_eq!(opts.len(), 1);
    run_plan(&mut g, &opts[0]);
    assert_eq!(on_bf(&g, P0, "Grave Titan").len(), 2);
}

#[test]
fn twincast_copies_a_worthwhile_spell_in_response() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Twincast"]);
    let bolt = hand(&mut g, P0, &["Harmonize"])[0];
    cast::cast_card(&mut g, P0, bolt, "hand", Ctx::default()).unwrap();
    assert!(gy_has(&g, P0, "Twincast"), "Twincast was cast");
    assert!(g.player(P0).spells_this_turn >= 2);
}

#[test]
fn flare_of_duplication_can_sacrifice_a_red_creature() {
    let mut g = game();
    perm(&mut g, P1, "Grave Titan");
    let fodder = perm(&mut g, P0, "Goblin Guide");
    hand(&mut g, P0, &["Flare of Duplication"]);
    let bolt = hand(&mut g, P0, &["Harmonize"])[0];
    cast::cast_card(&mut g, P0, bolt, "hand", Ctx::default()).unwrap();
    assert!(gy_has(&g, P0, "Flare of Duplication"));
    assert!(!g.perm(fodder).on_bf, "the red creature was sacrificed");
}

#[test]
fn volatile_stormdrake_exchanges_control_and_pays_energy() {
    let mut g = game();
    perm(&mut g, P1, "Ravenous Chupacabra"); // mana value 4: the four {E} pay for it
    let d = perm(&mut g, P0, "Volatile Stormdrake");
    assert_eq!(g.perm(d).owner, P1, "the opponent has the Drake");
    assert_eq!(on_bf(&g, P0, "Ravenous Chupacabra").len(), 1, "and we have the creature");
    assert_eq!(g.player(P0).energy, 0, "four energy paid for mana value 4");
}

#[test]
fn volatile_stormdrake_leaves_a_creature_it_cannot_pay_for() {
    let mut g = game();
    perm(&mut g, P1, "Mulldrifter"); // mana value 5 > 4 energy
    let d = perm(&mut g, P0, "Volatile Stormdrake");
    assert_eq!(g.perm(d).owner, P0, "no exchange");
    assert_eq!(on_bf(&g, P1, "Mulldrifter").len(), 1);
}

#[test]
fn peter_parkers_camera_enters_with_three_film_counters() {
    let mut g = game();
    let m = perm(&mut g, P0, "Peter Parker's Camera");
    assert_eq!(g.perm(m).data.int(DataKey::Counters), 3);
}
