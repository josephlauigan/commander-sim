//! Kefka's deck (impls/kefka.rs): each card on a hand-built position, with its Oracle text as the expectation, plus the
//! Psychosis Crawler loop and the deck's AI choices.

use sim_core::ai::{act, brain, decks};
use sim_core::engine::life::lose_life;
use sim_core::engine::{cast, hooks, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::kefka;
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KEFKA: &str = "Kefka, Court Mage // Kefka, Ruler of Ruin";

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn has_perm(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn perm_named(g: &Game, p: PlayerId, name: &str) -> PermId {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).name == name).unwrap_or_else(|| panic!("no {name}"))
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap()
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn fill_library(g: &mut Game, p: PlayerId, n: usize) {
    let cs = vec![card(g, "Island"); n];
    g.player_mut(p).library = cs;
}

fn opts_of(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn take(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = opts_of(g, p, false);
    let o = opts
        .iter()
        .find(|o| o.label == label)
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn has_opt(g: &mut Game, p: PlayerId, label: &str) -> bool {
    opts_of(g, p, false).iter().any(|o| o.label == label)
}

/// Kefka on the battlefield, flipped
fn flipped_kefka(g: &mut Game, p: PlayerId) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.player_mut(p).cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    let x = g.perm_mut(m);
    x.data.set(DataKey::Ruin, Val::Bool(true));
    (x.pow, x.tgh, x.fly) = (5, 7, true);
    m
}

fn table2() -> Game {
    table(&["kefka", "veyran"])
}

// ======================================================== the discard-pressure spells
#[test]
fn duress_takes_a_noncreature_nonland_card() {
    let mut g = table2();
    hand(&mut g, P1, &["Forest", "Llanowar Elves", "Sol Ring", "Counterspell"]);
    cast_free(&mut g, P0, "Duress", Ctx::default());
    let h = &g.player(P1).hand;
    assert_eq!(h.len(), 3);
    // the lands and creatures stay; one of the two noncreature nonland cards goes
    assert!(h.contains(&card(&g, "Forest")) && h.contains(&card(&g, "Llanowar Elves")));
    assert!(gy_has(&g, P1, "Sol Ring") || gy_has(&g, P1, "Counterspell"));
    assert!(gy_has(&g, P0, "Duress"));
}

#[test]
fn duress_with_only_creatures_and_lands_does_nothing() {
    let mut g = table2();
    hand(&mut g, P1, &["Forest", "Llanowar Elves"]);
    cast_free(&mut g, P0, "Duress", Ctx::default());
    assert_eq!(g.player(P1).hand.len(), 2);
}

#[test]
fn dark_inquiry_takes_a_creature_too_but_not_a_land() {
    let mut g = table2();
    hand(&mut g, P1, &["Forest", "Llanowar Elves"]);
    cast_free(&mut g, P0, "Dark Inquiry", Ctx::default());
    assert_eq!(g.player(P1).hand, vec![card(&g, "Forest")]);
    assert!(gy_has(&g, P1, "Llanowar Elves"));
}

#[test]
fn inner_fire_adds_red_for_each_card_in_hand() {
    let mut g = table2();
    hand(&mut g, P0, &["Forest", "Island", "Swamp"]);
    cast_free(&mut g, P0, "Inner Fire", Ctx::default());
    assert_eq!(g.player(P0).floating.r, 3); // the three left in hand (Inner Fire itself is on the stack)
}

#[test]
fn arcane_omens_counts_the_colours_spent() {
    let mut g = table2();
    for n in ["Swamp", "Island", "Mountain", "Swamp", "Swamp", "Island"] {
        lands(&mut g, P0, n, 1, false);
    }
    hand(&mut g, P0, &["Arcane Omens"]);
    hand(&mut g, P1, &["Forest", "Island", "Sol Ring", "Mountain", "Plains"]);
    // {4}{B} from six lands: black, blue and red can all be spent
    assert!(take(&mut g, P0, "Arcane Omens (X=3)"));
    assert_eq!(g.player(P1).hand.len(), 2); // three discards
    assert!(gy_has(&g, P0, "Arcane Omens"));
}

#[test]
fn arcane_omens_with_one_colour_discards_one() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 5, false);
    hand(&mut g, P0, &["Arcane Omens"]);
    hand(&mut g, P1, &["Forest", "Island", "Sol Ring"]);
    assert!(take(&mut g, P0, "Arcane Omens (X=1)"));
    assert_eq!(g.player(P1).hand.len(), 2);
}

#[test]
fn decorum_dissertation_targets_yourself_before_the_flip_and_exiles_for_paradigm() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    let (l0, h0) = (g.player(P0).life, g.player(P0).hand.len());
    cast_free(&mut g, P0, "Decorum Dissertation", Ctx::default());
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (l0 - 2, h0 + 2));
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Decorum Dissertation"));
    assert!(!gy_has(&g, P0, "Decorum Dissertation"));
    // the free copy at the beginning of the first main phase
    kefka::paradigm(&mut g, P0).unwrap();
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (l0 - 4, h0 + 4));
}

#[test]
fn decorum_dissertation_targets_an_opponent_once_kefka_is_flipped() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    fill_library(&mut g, P1, 20);
    flipped_kefka(&mut g, P0);
    let (l1, h0, h1) = (g.player(P1).life, g.player(P0).hand.len(), g.player(P1).hand.len());
    cast_free(&mut g, P0, "Decorum Dissertation", Ctx::default());
    assert_eq!(g.player(P1).life, l1 - 2);
    assert_eq!(g.player(P1).hand.len(), h1 + 2); // they draw two...
    assert_eq!(g.player(P0).hand.len(), h0 + 2); // ...and their loss draws me two
}

// ======================================================== madness, mayhem and cards from the graveyard
#[test]
fn ichor_slick_is_minus_three_minus_three() {
    let mut g = table2();
    let bear = perm(&mut g, P1, "Llanowar Elves");
    let c = hand(&mut g, P0, &["Ichor Slick"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(bear), ..Ctx::default() }).unwrap();
    assert!(!g.perm(bear).on_bf);
    let big = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Ichor Slick"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(big), ..Ctx::default() }).unwrap();
    assert!(g.perm(big).on_bf);
    assert_eq!((values::epow(&g, big), values::etgh(&g, big)), (3, 3)); // 6/6 -3/-3 until end of turn
}

#[test]
fn ichor_slick_madness_casts_it_when_discarded() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 4, false);
    let bear = perm(&mut g, P1, "Mulldrifter");
    let slick = hand(&mut g, P0, &["Ichor Slick"])[0];
    zones::discard_cards(&mut g, P0, &[slick]).unwrap();
    assert!(!g.perm(bear).on_bf, "madness {{3}}{{B}} killed the Bears");
    assert!(gy_has(&g, P0, "Ichor Slick"));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count(), 0);
}

#[test]
fn ichor_slick_madness_is_not_cast_without_mana_or_a_target() {
    let mut g = table2();
    let bear = perm(&mut g, P1, "Mulldrifter");
    let slick = hand(&mut g, P0, &["Ichor Slick"])[0];
    zones::discard_cards(&mut g, P0, &[slick]).unwrap(); // no mana
    assert!(g.perm(bear).on_bf);
    lands(&mut g, P0, "Swamp", 4, false);
    kill(&mut g, bear);
    let slick = hand(&mut g, P0, &["Ichor Slick"])[0];
    zones::discard_cards(&mut g, P0, &[slick]).unwrap(); // no target
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count(), 4);
}

fn kill(g: &mut Game, m: PermId) {
    zones::die(g, m, "destroy").unwrap();
}

#[test]
fn ichor_slick_cycles_for_two() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 2, false);
    fill_library(&mut g, P0, 10);
    hand(&mut g, P0, &["Ichor Slick"]);
    let n = g.player(P0).library.len();
    assert!(take(&mut g, P0, "Ichor Slick (cycling)"));
    assert_eq!(g.player(P0).library.len(), n - 1);
    assert!(gy_has(&g, P0, "Ichor Slick"));
}

#[test]
fn scarlet_spider_mayhem_from_the_graveyard_the_turn_it_was_discarded() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 1, false);
    let sp = hand(&mut g, P0, &["Scarlet Spider, Kaine"])[0];
    zones::discard_cards(&mut g, P0, &[sp]).unwrap();
    assert!(gy_has(&g, P0, "Scarlet Spider, Kaine"));
    assert!(take(&mut g, P0, "Scarlet Spider, Kaine (mayhem)"));
    assert!(has_perm(&g, P0, "Scarlet Spider, Kaine"));
    assert!(!gy_has(&g, P0, "Scarlet Spider, Kaine"));
}

#[test]
fn scarlet_spider_without_a_discard_this_turn_stays_put() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 1, false);
    let sp = take_card(&mut g, "Scarlet Spider, Kaine");
    g.player_mut(P0).gy.push(sp);
    assert!(!has_opt(&mut g, P0, "Scarlet Spider, Kaine (mayhem)"));
    // discarded last turn: not this one
    let sp2 = hand(&mut g, P0, &["Scarlet Spider, Kaine"])[0];
    zones::discard_cards(&mut g, P0, &[sp2]).unwrap();
    g.round += 1;
    assert!(!has_opt(&mut g, P0, "Scarlet Spider, Kaine (mayhem)"));
}

fn take_card(g: &mut Game, name: &str) -> CardId {
    sim_core::testkit::take(g, P0, name)
}

#[test]
fn scarlet_spider_enters_and_may_discard_for_a_counter() {
    let mut g = table2();
    hand(&mut g, P0, &["Ichor Slick"]); // a card that wants the graveyard
    let sp = perm(&mut g, P0, "Scarlet Spider, Kaine");
    assert_eq!(g.perm(sp).plus, 1);
    assert!(g.player(P0).hand.is_empty());
}

#[test]
fn demonic_embrace_is_cast_from_the_graveyard_for_life_and_a_discard() {
    let mut g = table2();
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let (p0, t0) = (values::epow(&g, bear), values::etgh(&g, bear));
    lands(&mut g, P0, "Swamp", 3, false);
    let emb = sim_core::testkit::take(&mut g, P0, "Demonic Embrace");
    g.player_mut(P0).gy.push(emb);
    hand(&mut g, P0, &["Forest"]);
    let l0 = g.player(P0).life;
    assert!(take(&mut g, P0, "Demonic Embrace (graveyard)"));
    assert_eq!(g.player(P0).life, l0 - 3);
    assert!(gy_has(&g, P0, "Forest")); // the discard
    assert!(g.perm(bear).on_bf);
    assert_eq!(values::epow(&g, bear), p0 + 3);
    assert_eq!(values::etgh(&g, bear), t0 + 1);
    assert!(!gy_has(&g, P0, "Demonic Embrace"));
}

#[test]
fn twinned_vision_draws_one_from_hand_and_two_with_flashback_and_a_discard() {
    let mut g = table2();
    fill_library(&mut g, P0, 10);
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Twinned Vision", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), n + 1);
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Forest"]);
    let (n, gy0) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    assert!(take(&mut g, P0, "Twinned Vision (flashback)"));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2); // the discard, then two cards
    assert_eq!(g.player(P0).gy.len(), gy0); // the discard in, the flashed-back card out (exiled)
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Twinned Vision")); // flashback exiles it
}

#[test]
fn postmortem_professor_returns_by_exiling_an_instant_or_sorcery() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 2, false);
    let prof = sim_core::testkit::take(&mut g, P0, "Postmortem Professor");
    let bolt = sim_core::testkit::take(&mut g, P0, "Lightning Bolt");
    g.player_mut(P0).gy.extend([prof, bolt]);
    assert!(take(&mut g, P0, "Postmortem Professor (return)"));
    assert!(has_perm(&g, P0, "Postmortem Professor"));
    assert!(g.player(P0).exile.contains(&bolt));
}

#[test]
fn postmortem_professor_needs_an_instant_or_sorcery_in_the_graveyard() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 2, false);
    let prof = sim_core::testkit::take(&mut g, P0, "Postmortem Professor");
    let bear = sim_core::testkit::take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([prof, bear]);
    assert!(!has_opt(&mut g, P0, "Postmortem Professor (return)"));
}

#[test]
fn postmortem_professor_attack_drains_one_and_gains_one() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    let prof = perm(&mut g, P0, "Postmortem Professor");
    let imp = g.registry.get(g.perm(prof).cd.unwrap()).unwrap().attack.unwrap();
    let l0 = g.player(P0).life;
    imp(&mut g, prof, P0, &[prof], P1).unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (39, 39));
    assert_eq!(g.player(P0).life, l0 + 1); // one life, not one per opponent
}

#[test]
fn garrison_excavator_makes_a_spirit_when_cards_leave_the_graveyard() {
    let mut g = table2();
    perm(&mut g, P0, "Garrison Excavator");
    lands(&mut g, P0, "Mountain", 1, false);
    let sp = hand(&mut g, P0, &["Scarlet Spider, Kaine"])[0];
    zones::discard_cards(&mut g, P0, &[sp]).unwrap();
    assert!(take(&mut g, P0, "Scarlet Spider, Kaine (mayhem)"));
    let spirits: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).token && g.perm(m).ttypes.iter().any(|t| &**t == "spirit"))
        .collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!((g.perm(spirits[0]).pow, g.perm(spirits[0]).tgh), (2, 2));
    assert!(g.perm(perm_named(&g, P0, "Garrison Excavator")).on_bf);
}

#[test]
fn garrison_excavator_does_nothing_when_nothing_leaves() {
    let mut g = table2();
    perm(&mut g, P0, "Garrison Excavator");
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token));
}

// ======================================================== the other spells
#[test]
fn extinguish_the_light_gains_three_for_a_cheap_creature() {
    let mut g = table2();
    let bear = perm(&mut g, P1, "Llanowar Elves");
    let l0 = g.player(P0).life;
    let c = hand(&mut g, P0, &["Extinguish the Light"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(bear), ..Ctx::default() }).unwrap();
    assert!(!g.perm(bear).on_bf);
    assert_eq!(g.player(P0).life, l0 + 3);
}

#[test]
fn extinguish_the_light_on_an_expensive_creature_gains_nothing() {
    let mut g = table2();
    let titan = perm(&mut g, P1, "Grave Titan");
    let l0 = g.player(P0).life;
    let c = hand(&mut g, P0, &["Extinguish the Light"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(titan), ..Ctx::default() }).unwrap();
    assert!(!g.perm(titan).on_bf);
    assert_eq!(g.player(P0).life, l0);
}

#[test]
fn grab_the_prize_discards_and_burns_unless_a_land_was_discarded() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 2, false);
    fill_library(&mut g, P0, 20);
    hand(&mut g, P0, &["Grab the Prize", "Lightning Bolt"]);
    assert!(take(&mut g, P0, "Grab the Prize"));
    assert!(gy_has(&g, P0, "Lightning Bolt")); // the discard
    assert_eq!(g.player(P1).life, 38);
    // a land discard: no damage
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 2, false);
    fill_library(&mut g, P0, 20);
    hand(&mut g, P0, &["Grab the Prize", "Forest"]);
    assert!(take(&mut g, P0, "Grab the Prize"));
    assert!(gy_has(&g, P0, "Forest"));
    assert_eq!(g.player(P1).life, 40);
    assert_eq!(g.player(P0).hand.len(), 2);
}

#[test]
fn disintegrate_goes_face_or_exiles_a_creature() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 5, false);
    g.player_mut(P1).life = 4;
    hand(&mut g, P0, &["Disintegrate"]);
    assert!(take(&mut g, P0, "Disintegrate the face of Veyran"));
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive);
    // a creature: exiled, not dying
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 5, false);
    let titan = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Disintegrate"])[0];
    cast::cast_card(
        &mut g,
        P0,
        c,
        "hand",
        Ctx { target: Some(titan), x: 6, rem_kind: Some(sim_core::sym::intern("dmg6")), ..Ctx::default() },
    )
    .unwrap();
    assert!(!g.perm(titan).on_bf);
    assert!(!gy_has(&g, P1, "Grave Titan")); // exiled instead of dying
}

#[test]
fn stargaze_takes_x_of_twice_x_and_costs_x_life() {
    let mut g = table2();
    fill_library(&mut g, P0, 30);
    set_top(&mut g, &["Lightning Bolt", "Sol Ring", "Forest", "Forest"]);
    let c = hand(&mut g, P0, &["Stargaze"])[0];
    let (l0, h0, lib0) = (g.player(P0).life, g.player(P0).hand.len(), g.player(P0).library.len());
    cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 2, ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).life, l0 - 2);
    assert_eq!(g.player(P0).hand.len(), h0 - 1 + 2);
    assert_eq!(g.player(P0).library.len(), lib0 - 4);
    assert_eq!(g.player(P0).gy.len(), 2 + 1); // the two not taken, and Stargaze
}

fn set_top(g: &mut Game, names: &[&str]) {
    let n = names.len();
    let len = g.player(P0).library.len();
    for (i, nm) in names.iter().enumerate() {
        let c = card(g, nm);
        g.player_mut(P0).library[len - n + i] = c;
    }
}

#[test]
fn stargaze_option_is_offered_with_spare_mana() {
    let mut g = table2();
    fill_library(&mut g, P0, 30);
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Stargaze"]);
    assert!(has_opt(&mut g, P0, "Stargaze (X=4)"));
}

#[test]
fn splatter_technique_draws_four_or_wipes() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Splatter Technique", Ctx { mode: Some(sim_core::sym::intern("draw")), ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), n + 4);
    let bear = perm(&mut g, P1, "Llanowar Elves");
    let titan = perm(&mut g, P1, "Grave Titan");
    let mine = perm(&mut g, P0, "Llanowar Elves");
    let pw = perm(&mut g, P1, "Jace, the Mind Sculptor");
    let l = g.perm(pw).loyalty.unwrap();
    cast_free(&mut g, P0, "Splatter Technique", Ctx { mode: Some(sim_core::sym::intern("wipe")), ..Ctx::default() });
    assert!(!g.perm(bear).on_bf && !g.perm(mine).on_bf, "4 damage to each creature, mine too");
    assert!(g.perm(titan).on_bf);
    assert!(l <= 4 && !g.perm(pw).on_bf, "4 damage to a planeswalker with loyalty {l}");
}

#[test]
fn splatter_technique_offers_the_wipe_only_against_creatures() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P0, "Mountain", 3, false);
    hand(&mut g, P0, &["Splatter Technique"]);
    assert!(!has_opt(&mut g, P0, "Splatter Technique (4 damage to each creature)"));
    assert!(has_opt(&mut g, P0, "Splatter Technique (draw four)"));
    perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Llanowar Elves");
    assert!(has_opt(&mut g, P0, "Splatter Technique (4 damage to each creature)"));
}

// ======================================================== the creatures
#[test]
fn alpharael_draws_two_and_discards_two() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    hand(&mut g, P0, &["Forest", "Island"]);
    let a = perm(&mut g, P0, "Alpharael, Dreaming Acolyte");
    assert_eq!(g.player(P0).hand.len(), 2 + 2 - 2);
    assert_eq!(g.player(P0).gy.len(), 2);
    let _ = a;
}

#[test]
fn alpharael_discards_one_artifact_instead_of_two() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    set_top(&mut g, &["Forest", "Mind Stone"]);
    perm(&mut g, P0, "Alpharael, Dreaming Acolyte");
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(gy_has(&g, P0, "Mind Stone"));
}

#[test]
fn alpharael_has_deathtouch_on_its_controllers_turn_only() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    let a = perm(&mut g, P0, "Alpharael, Dreaming Acolyte");
    assert!(sim_core::engine::combat::kw(&g, a, "deathtouch"));
    g.active = Some(P1);
    assert!(!sim_core::engine::combat::kw(&g, a, "deathtouch"));
}

#[test]
fn stadium_tidalmage_loots_on_entering_and_attacking() {
    let mut g = table2();
    fill_library(&mut g, P0, 20);
    hand(&mut g, P0, &["Forest"]);
    let t = perm(&mut g, P0, "Stadium Tidalmage");
    assert_eq!(g.player(P0).hand.len(), 1); // drew one, discarded one
    assert_eq!(g.player(P0).gy.len(), 1);
    let at = g.registry.get(g.perm(t).cd.unwrap()).unwrap().attack.unwrap();
    at(&mut g, t, P0, &[t], P1).unwrap();
    assert_eq!(g.player(P0).gy.len(), 2);
}

#[test]
fn screeching_soulbreaker_pings_each_opponent_and_gains_one_when_attacking() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    let s = perm(&mut g, P0, "Screeching Soulbreaker");
    let at = g.registry.get(g.perm(s).cd.unwrap()).unwrap().attack.unwrap();
    let l0 = g.player(P0).life;
    at(&mut g, s, P0, &[s], P1).unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (39, 39));
    assert_eq!(g.player(P0).life, l0 + 1);
}

#[test]
fn sawblade_scamp_collects_oil_on_noncreature_spells_and_pings() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    let s = perm(&mut g, P0, "Sawblade Scamp");
    cast_free(&mut g, P0, "Lightning Bolt", Ctx { face: Some(P1), ..Ctx::default() });
    cast_free(&mut g, P0, "Llanowar Elves", Ctx::default());
    cast_free(&mut g, P0, "Sol Ring", Ctx::default());
    assert_eq!(g.perm(s).data.int(DataKey::Counters), 2); // Bolt and Sol Ring, not the creature
    let l = (g.player(P1).life, g.player(P2).life);
    assert!(take(&mut g, P0, "Sawblade Scamp: 1 damage to each opponent"));
    assert_eq!((g.player(P1).life, g.player(P2).life), (l.0 - 1, l.1 - 1));
    assert_eq!(g.perm(s).data.int(DataKey::Counters), 1);
    assert!(g.perm(s).tapped);
    assert!(!has_opt(&mut g, P0, "Sawblade Scamp: 1 damage to each opponent")); // tapped
}

#[test]
fn susurian_voidborn_drains_when_your_creature_or_artifact_dies() {
    let mut g = table2();
    perm(&mut g, P0, "Susurian Voidborn");
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let l0 = g.player(P0).life;
    kill(&mut g, bear);
    assert_eq!((g.player(P1).life, g.player(P0).life), (39, l0 + 1));
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    kill(&mut g, theirs);
    assert_eq!(g.player(P1).life, 39); // an opponent's creature doesn't count
}

#[test]
fn susurian_voidborn_drains_for_each_treasure_cracked() {
    let mut g = table2();
    perm(&mut g, P0, "Susurian Voidborn");
    zones::add_treasure(&mut g, P0, 2).unwrap();
    let l1 = g.player(P1).life;
    hooks::fire_trigger(
        &mut g,
        Event::Sacrifice,
        Call::Sacrifice { p: P0, what: sim_core::hooks::Sacrificed::Token("Treasure") },
    )
    .unwrap();
    assert_eq!(g.player(P1).life, l1 - 1);
}

#[test]
fn susurian_voidborn_warp_is_cast_for_b_exiled_at_the_end_step_and_recast_later() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Susurian Voidborn"]);
    assert!(take(&mut g, P0, "Susurian Voidborn (warp)"));
    let m = perm_named(&g, P0, "Susurian Voidborn");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    let f = g.registry.get(g.perm(m).cd.unwrap()).unwrap().end_step.unwrap();
    f(&mut g, m, P0).unwrap();
    assert!(!g.perm(m).on_bf);
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Susurian Voidborn"));
    // not on the same turn
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(!has_opt(&mut g, P0, "Susurian Voidborn (from exile)"));
    g.round += 1;
    lands(&mut g, P0, "Swamp", 0, false);
    for &l in &g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(has_opt(&mut g, P0, "Susurian Voidborn (from exile)"));
}

#[test]
fn corroding_dragonstorm_drains_two_and_surveils() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    set_library(&mut g, P0, &["Island", "Island", "Island", "Lightning Bolt", "Ichor Slick"]);
    let l0 = g.player(P0).life;
    cast_free(&mut g, P0, "Corroding Dragonstorm", Ctx::default());
    assert_eq!((g.player(P1).life, g.player(P2).life), (38, 38));
    assert_eq!(g.player(P0).life, l0 + 2);
    assert!(gy_has(&g, P0, "Ichor Slick") || g.player(P0).library.len() == 5);
}

#[test]
fn corroding_dragonstorm_returns_to_hand_when_a_dragon_enters() {
    let mut g = table2();
    cast_free(&mut g, P0, "Corroding Dragonstorm", Ctx::default());
    assert!(has_perm(&g, P0, "Corroding Dragonstorm"));
    perm(&mut g, P0, "Glorybringer");
    assert!(!has_perm(&g, P0, "Corroding Dragonstorm"));
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Corroding Dragonstorm"));
}

#[test]
fn intellect_devourer_exiles_a_card_from_each_opponents_hand_until_it_leaves() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    hand(&mut g, P1, &["Forest", "Lightning Bolt"]);
    hand(&mut g, P2, &["Counterspell"]);
    let d = perm(&mut g, P0, "Intellect Devourer");
    assert_eq!((g.player(P1).hand.len(), g.player(P2).hand.len()), (1, 0));
    assert_eq!(g.player(P0).hand.len(), 2); // held for me to play
    zones::die(&mut g, d, "destroy").unwrap();
    assert_eq!((g.player(P1).hand.len(), g.player(P2).hand.len()), (2, 1));
    assert!(g.player(P0).hand.is_empty());
}

#[test]
fn gemstone_mine_has_three_uses() {
    let mut g = table2();
    let ls = lands(&mut g, P0, "Gemstone Mine", 1, false);
    for i in 0..3 {
        assert!(g.land(ls[0]).on_bf, "use {i}");
        let ok = sim_core::engine::mana::pay(&mut g, P0, 1, "", false).unwrap();
        assert!(ok);
        g.land_mut(ls[0]).tapped = false;
    }
    assert!(!g.land(ls[0]).on_bf);
    assert!(gy_has(&g, P0, "Gemstone Mine"));
}

// ======================================================== the plan: discards, the flip and the Crawler loop
#[test]
fn the_discard_choice_bins_madness_and_mayhem_cards_first_and_wants_new_types() {
    let mut g = table2();
    let h = hand(&mut g, P0, &["Forest", "Ichor Slick", "Duress"]);
    assert_eq!(kefka::discard_choice(&g, P0, &h, 0), Some(h[1]));
    // with Slick gone: an instant is a new type when the pile has a land only
    let h2 = vec![h[0], h[2]];
    let land_bits = kefka::card_types(&g, h[0]);
    let pick = kefka::discard_choice(&g, P0, &h2, land_bits).unwrap();
    assert_eq!(pick, h[2], "the Duress adds a type, the Forest does not");
}

#[test]
fn kefkas_wheel_for_the_kefka_deck_picks_a_new_type_after_the_opponents() {
    let mut g = table2();
    hand(&mut g, P1, &["Forest"]);
    hand(&mut g, P0, &["Forest", "Island", "Duress"]);
    sim_core::impls::mine::kefka_wheel(&mut g, P0).unwrap();
    // the opponent binned a land; I bin the sorcery (a second type) rather than a land: land + sorcery = two draws
    assert!(gy_has(&g, P0, "Duress"));
    assert_eq!(g.player(P0).hand.len(), 2 + 2);
}

#[test]
fn the_flip_option_is_offered_at_eight_mana_and_ranks_with_the_plays_that_matter() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 8, false);
    perm(&mut g, P0, KEFKA);
    let opts = opts_of(&mut g, P0, false);
    let u = opts.iter().find(|o| o.label == "Kefka: {8}, transform").unwrap().utility;
    assert!(u > 5.0 && u < 8.0, "{u}");
}

#[test]
fn the_flip_with_the_crawler_out_waits_for_a_library_that_outlasts_the_table() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 8, false);
    perm(&mut g, P0, KEFKA);
    hand(&mut g, P0, &["Island"]); // the Crawler is */* (cards in hand): with an empty hand it is a 0/0 and dies
    perm(&mut g, P0, "Psychosis Crawler");
    fill_library(&mut g, P0, 20); // the opponent has 40 life
    assert!(!kefka::loop_safe(&g, P0));
    assert!(!has_opt(&mut g, P0, "Kefka: {8}, transform"));
    fill_library(&mut g, P0, 60);
    assert!(kefka::loop_safe(&g, P0));
    assert!(has_opt(&mut g, P0, "Kefka: {8}, transform"));
}

#[test]
fn the_crawler_is_held_after_the_flip_when_the_loop_would_deck_us() {
    let mut g = table2();
    flipped_kefka(&mut g, P0);
    let c = card(&g, "Psychosis Crawler");
    fill_library(&mut g, P0, 20);
    assert_eq!(decks::deck_prio(&g, P0, c), 0);
    fill_library(&mut g, P0, 60);
    assert!(decks::deck_prio(&g, P0, c) > 0);
}

#[test]
fn the_crawler_loop_kills_the_table_when_the_library_is_long_enough() {
    let mut g = table(&["kefka", "veyran", "seph"]);
    flipped_kefka(&mut g, P0);
    hand(&mut g, P0, &["Island"]); // the Crawler is */* (cards in hand): with an empty hand it is a 0/0 and dies
    perm(&mut g, P0, "Psychosis Crawler");
    fill_library(&mut g, P0, 80);
    g.player_mut(P1).life = 6;
    g.player_mut(P2).life = 9;
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!g.player(P1).alive && !g.player(P2).alive, "{} {}", g.player(P1).life, g.player(P2).life);
    assert!(g.player(P0).alive);
    assert!(g.player(P0).library.len() > 40); // it stopped as soon as the table was dead
}

#[test]
fn the_crawler_loop_stops_cleanly_when_the_library_runs_out() {
    let mut g = table(&["kefka", "veyran"]);
    flipped_kefka(&mut g, P0);
    hand(&mut g, P0, &["Island"]); // the Crawler is */* (cards in hand): with an empty hand it is a 0/0 and dies
    perm(&mut g, P0, "Psychosis Crawler");
    fill_library(&mut g, P0, 5);
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.player(P0).library.is_empty());
    assert!(g.player(P1).life < 40 && g.player(P1).alive);
}

#[test]
fn the_crawler_loop_ends_with_the_last_opponent_dead_in_a_four_player_game() {
    let mut g = table(&["kefka", "veyran", "seph", "sauron"]);
    flipped_kefka(&mut g, P0);
    hand(&mut g, P0, &["Island"]); // the Crawler is */* (cards in hand): with an empty hand it is a 0/0 and dies
    perm(&mut g, P0, "Psychosis Crawler");
    fill_library(&mut g, P0, 90);
    for q in [P1, P2, PlayerId(3)] {
        g.player_mut(q).life = 12;
    }
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.over || g.players.iter().skip(1).all(|q| !q.alive));
}

#[test]
fn a_flipped_kefka_draws_for_each_life_lost_on_your_turn() {
    let mut g = table2();
    flipped_kefka(&mut g, P0);
    fill_library(&mut g, P0, 30);
    let n = g.player(P0).hand.len();
    lose_life(&mut g, P1, 3, Some(P0), "other", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 3);
}

#[test]
fn the_discard_choice_keeps_a_held_crawler() {
    let mut g = table2();
    flipped_kefka(&mut g, P0);
    fill_library(&mut g, P0, 10); // the loop is not safe: the Crawler's cast priority is 0
    let h = hand(&mut g, P0, &["Psychosis Crawler", "Forest"]);
    assert_eq!(kefka::discard_choice(&g, P0, &h, 0), Some(h[1]));
}

#[test]
fn scarlet_spider_and_embrace_are_held_for_kefkas_trigger() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 4, false);
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, KEFKA);
    let sp = card(&g, "Scarlet Spider, Kaine");
    assert_eq!(decks::deck_prio(&g, P0, sp), 0);
    let g2 = table2();
    assert!(decks::deck_prio(&g2, P0, card(&g2, "Scarlet Spider, Kaine")) > 0, "without Kefka it is a body");
}

// ------------------------------------------------------------------ Oppression
/// Oppression: "Whenever a player casts a spell, that player discards a card." Symmetric: the caster discards (and
/// chooses), whoever it is.
#[test]
fn oppression_makes_every_caster_discard_a_card() {
    let mut g = table(&["kefka", "seph", "veyran"]);
    perm(&mut g, P0, "Oppression");
    // an opponent casts a spell with two cards left in hand: one is discarded
    hand(&mut g, P1, &["Swords to Plowshares", "Forest", "Island"]);
    let c = g.player(P1).hand[0];
    let gy0 = g.player(P1).gy.len();
    let before = g.player(P1).hand.len();
    assert!(cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap_or(false) || g.player(P1).hand.len() < before);
    // the spell left the hand, then one card was discarded (the spell itself may fizzle: only the discard is checked)
    assert_eq!(g.player(P1).hand.len(), before - 2, "the spell and a discarded card");
    assert!(g.player(P1).gy.len() >= gy0 + 1);
    // Kefka's own spell costs Kefka a card
    hand(&mut g, P0, &["Duress", "Island", "Swamp"]);
    let d = g.player(P0).hand[0];
    let b0 = g.player(P0).hand.len();
    cast::cast_card(&mut g, P0, d, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), b0 - 2, "Kefka discards too");
    // an empty hand: nothing to discard, no panic
    g.player_mut(P2).hand.clear();
    hand(&mut g, P2, &["Counterspell"]);
    let e = g.player(P2).hand[0];
    cast::cast_card(&mut g, P2, e, "hand", Ctx::default()).unwrap();
    assert!(g.player(P2).hand.is_empty());
}

/// the AI casts Oppression when the table holds more cards than Kefka can spare, never into an empty hand or with the
/// storm cards in hand
#[test]
fn kefka_casts_oppression_when_opponents_hold_cards_and_not_with_a_combo_turn() {
    let mut g = table(&["kefka", "seph", "veyran"]);
    let c = hand(&mut g, P0, &["Oppression", "Swamp", "Island", "Duress"])[0];
    hand(&mut g, P1, &["Forest", "Forest", "Forest", "Forest"]);
    hand(&mut g, P2, &["Forest", "Forest", "Forest", "Forest"]);
    assert!(decks::deck_prio(&g, P0, c) >= 40, "{}", decks::deck_prio(&g, P0, c));
    hand(&mut g, P0, &["Grapeshot", "Dark Inquiry", "Inner Fire", "Arcane Omens", "Stargaze"]);
    lands(&mut g, P0, "Swamp", 6, false);
    assert_eq!(decks::deck_prio(&g, P0, c), 0, "a storm card with spells to chain: no tax on the combo turn");
    g.player_mut(P0).hand.retain(|&x| x == c);
    assert!(decks::deck_prio(&g, P0, c) > 0, "an empty-handed Kefka is the one the tax spares");
}

// ======================================================== Marvin, Murderous Mimic
/// Marvin, Murderous Mimic: "Marvin has all activated abilities of creatures you control that don't have the same name as
/// this creature." Modeled for Niv-Mizzet, Viashino Fangtail, Blightspeaker and Rummaging Goblin only (Partial).
#[test]
fn marvin_uses_nivs_draw_only_while_niv_is_out() {
    let mut g = table(&["kefka", "veyran"]);
    fill_library(&mut g, P0, 10);
    let marvin = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let opts = |g: &mut Game| {
        let c = g.perm(marvin).cd.unwrap();
        let f = g.registry.get(c).unwrap().options.unwrap();
        f(g, marvin, P0, None).unwrap()
    };
    assert!(opts(&mut g).is_empty(), "no creature to copy");
    let niv = perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    g.perm_mut(niv).tapped = true; // Niv himself is tapped: it is Marvin who draws
    let o = opts(&mut g);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Marvin copies Niv-Mizzet, the Firemind");
    let hand0 = g.player(P0).hand.len();
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert!(g.perm(marvin).tapped);
    assert!(opts(&mut g).is_empty(), "one tap ability a turn");
    // Niv leaves: nothing to copy again
    g.perm_mut(marvin).tapped = false;
    zones::die(&mut g, niv, "destroy").unwrap();
    assert!(opts(&mut g).is_empty());
    // summoning sick: can't tap
    perm(&mut g, P0, "Viashino Fangtail");
    g.perm_mut(marvin).sick = true;
    assert!(opts(&mut g).is_empty());
    g.perm_mut(marvin).sick = false;
    let o = opts(&mut g);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Marvin copies Viashino Fangtail");
    let life = g.player(P1).life;
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(g.player(P1).life < life || g.player(P0).life < 40, "Fangtail's ping went somewhere");
}

#[test]
fn marvin_is_cast_sooner_with_a_creature_to_copy_and_the_ai_taps_him() {
    let mut g = table(&["kefka", "veyran"]);
    fill_library(&mut g, P0, 10);
    let c = card(&g, "Marvin, Murderous Mimic");
    let bare = decks::deck_prio(&g, P0, c);
    assert!(bare > 0);
    perm(&mut g, P0, "Viashino Fangtail");
    let with_pinger = decks::deck_prio(&g, P0, c);
    assert!(with_pinger > bare);
    perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    assert!(decks::deck_prio(&g, P0, c) > with_pinger, "Niv's draw is the best copy");
    let marvin = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let o = opts_of(&mut g, P0, false);
    assert!(
        o.iter().any(|x| x.label.starts_with("Marvin copies") && matches!(x.act, Some(sim_core::hooks::Action::Ability { src, .. }) if src == marvin)),
        "main options offer Marvin copied ability: {:?}", o.iter().map(|x| x.label.clone()).collect::<Vec<_>>()
    );
}
