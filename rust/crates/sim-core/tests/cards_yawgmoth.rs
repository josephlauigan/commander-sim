//! The yawgmoth candidate deck's cards (impls/yawgmoth.rs), on hand-built positions. Seat 0 plays the yawgmoth deck,
//! seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::cardcode;
use sim_core::engine::combat::{can_block, kw};
use sim_core::engine::mana::total_mana;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, hooks, life, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::alela;
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["yawgmoth", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn find_option(g: &mut Game, p: PlayerId, post: bool, label: &str) -> Option<Opt> {
    options(g, p, post).into_iter().find(|o| o.label.starts_with(label))
}

fn take_option(g: &mut Game, p: PlayerId, post: bool, label: &str) -> bool {
    let opts = options(g, p, post);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(label))
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn to_gy(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).gy.extend(&cs);
    cs
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn fire(g: &mut Game, ev: Event, p: PlayerId) {
    hooks::fire_trigger(g, ev, Call::Player { p }).unwrap();
}

fn mana(g: &mut Game, p: PlayerId, n: usize) {
    lands(g, p, "Swamp", n, false);
}

fn tokens_named(g: &Game, p: PlayerId, ty: &'static str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).collect()
}

/// an opposing creature with the given size (a token, not summoning sick)
fn foe(g: &mut Game, pow: i32, tgh: i32) -> PermId {
    let spec = zones::Tokens { sick: false, tgh: Some(tgh), ..zones::Tokens::new(1, pow) };
    zones::make_tokens(g, P1, spec).unwrap()[0]
}

// ------------------------------------------------------------------ Yawgmoth, Thran Physician
#[test]
fn yawgmoth_sacrifices_for_a_counter_that_kills_and_a_card() {
    let mut g = game();
    let y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let tok = token(&mut g, P0, 1);
    let x1 = foe(&mut g, 3, 1);
    let hand_before = g.player(P0).hand.len();
    assert!(take_option(&mut g, P0, false, "Yawgmoth: sacrifice"));
    assert!(!g.perm(tok).on_bf, "the token was the fodder");
    assert!(!g.perm(x1).on_bf, "the -1/-1 counter killed the X/1");
    assert_eq!(g.player(P0).life, 39, "1 life");
    assert_eq!(g.player(P0).hand.len(), hand_before + 1, "a card");
    assert!(g.perm(y).on_bf, "'another creature'");
}

#[test]
fn yawgmoth_counter_goes_on_a_creature_it_shrinks_and_stays() {
    let mut g = game();
    let _y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let _tok = token(&mut g, P0, 1);
    let big = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    let (p0, t0) = (epow(&g, big), etgh(&g, big));
    assert!(take_option(&mut g, P0, false, "Yawgmoth: sacrifice"));
    assert_eq!(g.perm(big).plus, -1);
    assert_eq!((epow(&g, big), etgh(&g, big)), (p0 - 1, t0 - 1));
}

#[test]
fn yawgmoth_keeps_life_above_ten_and_does_not_sacrifice_real_creatures() {
    let mut g = game();
    let _y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let _tok = token(&mut g, P0, 1);
    let _x1 = foe(&mut g, 3, 1);
    g.player_mut(P0).life = 10;
    assert!(find_option(&mut g, P0, false, "Yawgmoth: sacrifice").is_none(), "life 10 is the floor");
    g.player_mut(P0).life = 40;
    let mut g2 = game();
    let _y = perm(&mut g2, P0, "Yawgmoth, Thran Physician");
    let _bomb = perm(&mut g2, P0, "Phyrexian Delver");
    assert!(find_option(&mut g2, P0, false, "Yawgmoth: sacrifice").is_none(), "a 3/2 body is not fodder");
}

#[test]
fn yawgmoth_proliferates_minus_counters_to_kill_and_plus_counters_to_grow() {
    let mut g = game();
    let _y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let mine = perm(&mut g, P0, "Blood Artist");
    g.perm_mut(mine).plus = 1;
    let foe2 = foe(&mut g, 3, 2);
    g.perm_mut(foe2).plus = -1; // a 2/1
    mana(&mut g, P0, 2);
    hand(&mut g, P0, &["Swamp", "Arcane Signet"]);
    assert!(take_option(&mut g, P0, true, "Yawgmoth: proliferate"));
    assert!(!g.perm(foe2).on_bf, "the second -1/-1 counter kills the X/1");
    assert_eq!(g.perm(mine).plus, 2);
    assert_eq!(g.player(P0).hand.len(), 1, "a card discarded");
}

#[test]
fn yawgmoth_has_protection_from_humans() {
    let mut g = game();
    let y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let human = perm(&mut g, P1, "Thalia, Guardian of Thraben");
    let beast = foe(&mut g, 2, 2);
    assert!(cardcode::prot_vs(&g, y, human), "no damage from Humans");
    assert!(!cardcode::prot_vs(&g, y, beast));
    assert!(!can_block(&g, human, y), "Humans can't block it");
    assert!(can_block(&g, beast, y));
}

#[test]
fn yawgmoth_sacrifices_a_dying_blocker_after_blocks() {
    let mut g = game();
    let y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let blocker = token(&mut g, P0, 1);
    let atk = foe(&mut g, 3, 1);
    let mut assign = vec![(atk, blocker)];
    cardcode::defend_hooks(&mut g, P1, &mut vec![atk], P0, &mut assign).unwrap();
    assert!(!g.perm(blocker).on_bf);
    assert!(!g.perm(atk).on_bf, "its counter killed the 3/1 attacker");
    assert!(g.perm(y).on_bf);
}

// ------------------------------------------------------------------ Braids's Frightful Return
#[test]
fn braids_reads_ahead_to_chapter_two_for_a_good_graveyard_creature() {
    let mut g = game();
    to_gy(&mut g, P0, &["Phyrexian Delver"]);
    let b = perm(&mut g, P0, "Braids's Frightful Return");
    assert!(in_hand(&g, P0, "Phyrexian Delver"), "chapter II returned it");
    assert!(g.perm(b).on_bf, "chapter III is still to come");
    assert_eq!(g.perm(b).data.int(DataKey::Lore), 2);
    // chapter III after the next draw step: the opponent sacrifices a spare permanent or loses 2 and you draw
    let life = g.player(P1).life;
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::Main1, P0);
    assert!(on_bf(&g, P0, "Braids's Frightful Return").is_none(), "sacrificed after III");
    let lost = g.player(P1).life == life - 2 && g.player(P0).hand.len() == before + 1;
    let sac = g.player(P1).perms.is_empty() || g.player(P1).life == life;
    assert!(lost || sac);
}

#[test]
fn braids_chapter_one_sacrifices_a_token_for_a_discard() {
    let mut g = game();
    let tok = token(&mut g, P0, 1);
    hand(&mut g, P1, &["Swamp", "Swamp"]);
    let h0 = g.player(P1).hand.len();
    let b = perm(&mut g, P0, "Braids's Frightful Return");
    assert_eq!(g.perm(b).data.int(DataKey::Lore), 1);
    assert!(!g.perm(tok).on_bf);
    assert_eq!(g.player(P1).hand.len(), h0 - 1);
    // II, then III, then it is sacrificed
    fire(&mut g, Event::Main1, P0);
    assert_eq!(g.perm(b).data.int(DataKey::Lore), 2);
    fire(&mut g, Event::Main1, P0);
    assert!(!g.perm(b).on_bf);
}

#[test]
fn braids_chapter_three_makes_a_rich_opponent_pay_life() {
    let mut g = game();
    g.player_mut(P1).perms.clear();
    let big = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    let life = g.player(P1).life;
    let hand_n = g.player(P0).hand.len();
    let b = perm(&mut g, P0, "Braids's Frightful Return");
    g.perm_mut(b).data.set(DataKey::Lore, Val::Int(2));
    fire(&mut g, Event::Main1, P0);
    assert!(g.perm(big).on_bf, "worth more than a card");
    assert_eq!(g.player(P1).life, life - 2);
    assert_eq!(g.player(P0).hand.len(), hand_n + 1);
    assert!(!g.perm(b).on_bf);
}

#[test]
fn braids_gains_a_lore_counter_from_proliferate() {
    let mut g = game();
    let b = perm(&mut g, P0, "Braids's Frightful Return");
    let n = g.perm(b).data.int(DataKey::Lore);
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert!(g.perm(b).data.int(DataKey::Lore) > n || !g.perm(b).on_bf);
}

// ------------------------------------------------------------------ Presumed Dead, It Doesn't Add Up
#[test]
fn presumed_dead_pumps_and_brings_the_creature_back_suspected() {
    let mut g = game();
    let m = perm(&mut g, P0, "Phyrexian Rager");
    let c = hand(&mut g, P0, &["Presumed Dead"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(m), ..Ctx::default() }).unwrap();
    assert_eq!(epow(&g, m), 2 + 2);
    zones::die(&mut g, m, intern("destroy")).unwrap();
    let back = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&x| &*g.perm(x).name == "Phyrexian Rager")
        .expect("it returned to the battlefield");
    assert!(g.perm(back).data.truthy(DataKey::Suspect));
    assert!(kw(&g, back, "menace") && kw(&g, back, "cant_block"));
    assert!(!in_gy(&g, P0, "Phyrexian Rager"));
    // a new object: it has no grant now
    zones::die(&mut g, back, intern("destroy")).unwrap();
    assert!(in_gy(&g, P0, "Phyrexian Rager"));
}

#[test]
fn presumed_dead_is_cast_before_combat_on_a_good_attacker() {
    let mut g = game();
    let _m = perm(&mut g, P0, "Phyrexian Delver");
    mana(&mut g, P0, 2);
    hand(&mut g, P0, &["Presumed Dead"]);
    assert!(take_option(&mut g, P0, false, "Presumed Dead"));
    assert!(in_gy(&g, P0, "Presumed Dead"));
}

#[test]
fn it_doesnt_add_up_reanimates_the_best_creature_and_suspects_it() {
    let mut g = game();
    to_gy(&mut g, P0, &["Festering Goblin", "Dread Drone"]);
    cast_spell(&mut g, P0, "It Doesn't Add Up");
    let m = on_bf(&g, P0, "Dread Drone").expect("the best creature card returned");
    assert!(kw(&g, m, "menace") && kw(&g, m, "cant_block"), "suspected");
    assert!(in_gy(&g, P0, "Festering Goblin"));
    assert!(in_gy(&g, P0, "It Doesn't Add Up"));
}

// ------------------------------------------------------------------ Rewrite Regrets
#[test]
fn rewrite_regrets_returns_a_cheap_creature_and_empowers_jace() {
    let mut g = game();
    to_gy(&mut g, P0, &["Phyrexian Delver", "Corpulent Corpse"]);
    cast_spell(&mut g, P0, "Rewrite Regrets");
    assert!(on_bf(&g, P0, "Phyrexian Delver").is_some());
    let j = alela::jace_token(&g, P0).expect("Empower Jace made the Jace token");
    assert_eq!(g.perm(j).loyalty, Some(2));
    cast_spell(&mut g, P0, "Rewrite Regrets");
    assert_eq!(g.perm(j).loyalty, Some(4), "Empower Jace 2 adds two loyalty counters to the token you control");
}

#[test]
fn rewrite_regrets_does_not_return_a_creature_above_mana_value_six() {
    let mut g = game();
    to_gy(&mut g, P0, &["Sheoldred, Whispering One"]);
    assert!(g.db.get(g.db.id("Sheoldred, Whispering One").unwrap()).cmc > 6);
    cast_spell(&mut g, P0, "Rewrite Regrets");
    assert!(on_bf(&g, P0, "Sheoldred, Whispering One").is_none());
}

// ------------------------------------------------------------------ Soul Enervation
#[test]
fn soul_enervation_gives_minus_four_and_drains_when_creature_cards_leave() {
    let mut g = game();
    let victim = foe(&mut g, 3, 4);
    let keeper = foe(&mut g, 2, 6);
    to_gy(&mut g, P0, &["Festering Goblin"]);
    cast_spell(&mut g, P0, "Soul Enervation");
    assert!(!g.perm(victim).on_bf, "the 3/4 died");
    assert_eq!(etgh(&g, keeper), 6, "the 2/6 wasn't the pick");
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    // a creature card leaves the graveyard (Raise Dead style): each opponent loses 1, you gain 1
    let c = g.player(P0).gy.iter().copied().find(|&c| &*g.db.get(c).name == "Festering Goblin").unwrap();
    let i = g.player(P0).gy.iter().position(|&x| x == c).unwrap();
    g.player_mut(P0).gy.remove(i);
    g.player_mut(P0).hand.push(c);
    life::check_state(&mut g).unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life), (l0 + 1, l1 - 1));
    // nothing more leaves: no more drain
    life::check_state(&mut g).unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life), (l0 + 1, l1 - 1));
}

// ------------------------------------------------------------------ the creatures
#[test]
fn arnim_zola_makes_a_tapped_villain_with_two_creature_cards_in_the_graveyard() {
    let mut g = game();
    let z = perm(&mut g, P0, "Arnim Zola, Bio-Fanatic");
    mana(&mut g, P0, 3);
    to_gy(&mut g, P0, &["Festering Goblin"]);
    assert!(find_option(&mut g, P0, true, "Arnim Zola").is_none(), "only one creature card");
    to_gy(&mut g, P0, &["Burglar Rat"]);
    assert!(take_option(&mut g, P0, true, "Arnim Zola"));
    let v = tokens_named(&g, P0, "villain");
    assert_eq!(v.len(), 1);
    assert!(g.perm(v[0]).tapped);
    assert_eq!((epow(&g, v[0]), etgh(&g, v[0])), (2, 1));
    assert!(kw(&g, v[0], "menace"));
    assert!(g.perm(z).tapped, "{{T}}");
}

#[test]
fn bloodsworn_squire_transforms_into_a_knight_as_big_as_the_graveyard() {
    let mut g = game();
    let s = perm(&mut g, P0, "Bloodsworn Squire");
    mana(&mut g, P0, 2);
    to_gy(&mut g, P0, &["Festering Goblin", "Burglar Rat", "Blood Artist", "Dread Drone"]);
    hand(&mut g, P0, &["Swamp"]);
    assert_eq!((epow(&g, s), etgh(&g, s)), (3, 3));
    assert!(take_option(&mut g, P0, true, "Bloodsworn Squire"));
    assert!(g.perm(s).tapped);
    assert!(sim_core::engine::values::indestructible(&g, s));
    assert!(g.perm(s).data.truthy(DataKey::Flipped), "four creature cards: transformed");
    assert_eq!((epow(&g, s), etgh(&g, s)), (4, 4), "four creature cards in the graveyard");
    zones::discard_cards(&mut g, P0, &[]).unwrap();
    to_gy(&mut g, P0, &["Rank Rat"]);
    assert_eq!(epow(&g, s), 5);
}

#[test]
fn bloodsworn_squire_does_not_transform_with_three_creature_cards() {
    let mut g = game();
    let s = perm(&mut g, P0, "Bloodsworn Squire");
    mana(&mut g, P0, 2);
    to_gy(&mut g, P0, &["Festering Goblin", "Burglar Rat", "Blood Artist"]);
    hand(&mut g, P0, &["Swamp"]);
    // the discard is a land: the count stays three, the ability still works if used
    let src = s;
    let f = |g: &mut Game| {
        let o = options(g, P0, true);
        o.iter().any(|o| o.label.starts_with("Bloodsworn Squire"))
    };
    assert!(!f(&mut g), "the AI waits for four");
    let _ = src;
}

#[test]
fn catapult_fodder_transforms_with_three_creatures_that_have_toughness_over_power() {
    let mut g = game();
    let f = perm(&mut g, P0, "Catapult Fodder");
    let a = zones::make_tokens(&mut g, P0, zones::Tokens { tgh: Some(1), ..zones::Tokens::new(1, 0) }).unwrap()[0];
    cardcode::combat_start(&mut g, P0).unwrap();
    assert!(!g.perm(f).data.truthy(DataKey::Flipped), "two such creatures");
    let _b = zones::make_tokens(&mut g, P0, zones::Tokens { tgh: Some(1), ..zones::Tokens::new(1, 0) }).unwrap()[0];
    assert_eq!((epow(&g, f), etgh(&g, f)), (1, 5));
    cardcode::combat_start(&mut g, P0).unwrap();
    assert!(g.perm(f).data.truthy(DataKey::Flipped));
    assert_eq!((epow(&g, f), etgh(&g, f)), (2, 6), "Catapult Captain");
    let _ = a;
}

#[test]
fn catapult_captain_drains_for_the_toughness_of_the_sacrificed_creature() {
    let mut g = game();
    let f = perm(&mut g, P0, "Catapult Fodder");
    g.perm_mut(f).data.set(DataKey::Flipped, Val::Bool(true));
    g.selfpt = true;
    let victim = zones::make_tokens(&mut g, P0, zones::Tokens { tgh: Some(5), ..zones::Tokens::new(1, 0) }).unwrap()[0];
    mana(&mut g, P0, 3);
    g.player_mut(P1).life = 5;
    assert!(take_option(&mut g, P0, true, "Catapult Captain"));
    assert!(!g.perm(victim).on_bf);
    assert!(!g.player(P1).alive || g.player(P1).life <= 0, "5 toughness = 5 life");
    assert!(g.perm(f).tapped);
}

#[test]
fn cheerful_osteomancer_enters_prepared_and_casts_raise_dead_once() {
    let mut g = game();
    to_gy(&mut g, P0, &["Phyrexian Delver", "Burglar Rat"]);
    let o = perm(&mut g, P0, "Cheerful Osteomancer");
    assert!(g.perm(o).data.truthy(DataKey::Prepared));
    mana(&mut g, P0, 1);
    assert!(take_option(&mut g, P0, false, "Raise Dead"));
    assert!(in_hand(&g, P0, "Phyrexian Delver"));
    assert!(!g.perm(o).data.truthy(DataKey::Prepared));
    assert!(find_option(&mut g, P0, false, "Raise Dead").is_none());
}

#[test]
fn dread_drone_makes_two_spawn_that_sacrifice_for_mana() {
    let mut g = game();
    let before = total_mana(&g, P0, false);
    cast_spell(&mut g, P0, "Dread Drone");
    let sp = tokens_named(&g, P0, "spawn");
    assert_eq!(sp.len(), 2);
    assert_eq!((epow(&g, sp[0]), etgh(&g, sp[0])), (0, 1));
    assert_eq!(total_mana(&g, P0, false), before + 2, "'Sacrifice this token: Add {{C}}'");
}

#[test]
fn festering_goblin_shrinks_a_creature_when_it_dies() {
    let mut g = game();
    let gob = perm(&mut g, P0, "Festering Goblin");
    let x1 = foe(&mut g, 2, 1);
    let big = foe(&mut g, 4, 4);
    zones::die(&mut g, gob, intern("destroy")).unwrap();
    assert!(!g.perm(x1).on_bf, "the X/1 it kills");
    assert_eq!(etgh(&g, big), 4);
    // with nothing to kill: -1/-1 on the best one, until end of turn
    let gob2 = perm(&mut g, P0, "Festering Goblin");
    zones::die(&mut g, gob2, intern("destroy")).unwrap();
    assert_eq!((epow(&g, big), etgh(&g, big)), (3, 3));
    assert_eq!(g.perm(big).plus, 0, "not a counter");
}

#[test]
fn perigee_beckoner_buffs_a_creature_that_returns_tapped_and_can_warp() {
    let mut g = game();
    let a = perm(&mut g, P0, "Festering Goblin");
    let _ = perm(&mut g, P0, "Perigee Beckoner");
    assert_eq!(epow(&g, a), 3);
    zones::die(&mut g, a, intern("destroy")).unwrap();
    let back = on_bf(&g, P0, "Festering Goblin").expect("returned");
    assert!(g.perm(back).tapped);
    assert_eq!(epow(&g, back), 1, "a new object: no +2/+0");
}

#[test]
fn perigee_beckoner_warp_is_exiled_at_the_end_step_and_cast_later_from_exile() {
    let mut g = game();
    let _a = perm(&mut g, P0, "Blood Artist");
    mana(&mut g, P0, 2);
    hand(&mut g, P0, &["Perigee Beckoner"]);
    assert!(take_option(&mut g, P0, false, "Perigee Beckoner (warp)"));
    let b = on_bf(&g, P0, "Perigee Beckoner").expect("warped in for {1}{B}");
    fire(&mut g, Event::EndStep, P0);
    assert!(!g.perm(b).on_bf);
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Perigee Beckoner"));
    // not on the same turn; next turn with five mana
    assert!(find_option(&mut g, P0, false, "Perigee Beckoner (from exile)").is_none());
    g.player_mut(P0).flag_turn.clear();
    mana(&mut g, P0, 5);
    assert!(take_option(&mut g, P0, false, "Perigee Beckoner (from exile)"));
    assert!(on_bf(&g, P0, "Perigee Beckoner").is_some());
}

#[test]
fn phyrexian_delver_returns_the_best_creature_and_costs_its_mana_value_in_life() {
    let mut g = game();
    to_gy(&mut g, P0, &["Festering Goblin", "Dread Drone"]);
    let life = g.player(P0).life;
    let _d = perm(&mut g, P0, "Phyrexian Delver");
    assert!(on_bf(&g, P0, "Dread Drone").is_some());
    assert_eq!(g.player(P0).life, life - 5, "Dread Drone has mana value 5");
}

#[test]
fn phyrexian_delver_does_not_pay_life_it_cannot_spare() {
    let mut g = game();
    to_gy(&mut g, P0, &["Dread Drone"]);
    g.player_mut(P0).life = 8;
    let _d = perm(&mut g, P0, "Phyrexian Delver");
    assert!(on_bf(&g, P0, "Dread Drone").is_none());
    assert_eq!(g.player(P0).life, 8);
}

#[test]
fn send_in_the_pest_makes_them_discard_and_makes_a_pest_that_gains_life() {
    let mut g = game();
    hand(&mut g, P1, &["Swamp", "Swamp"]);
    let h = g.player(P1).hand.len();
    cast_spell(&mut g, P0, "Send in the Pest");
    assert_eq!(g.player(P1).hand.len(), h - 1);
    let p = tokens_named(&g, P0, "pest");
    assert_eq!(p.len(), 1);
    assert_eq!((epow(&g, p[0]), etgh(&g, p[0])), (1, 1));
    let life = g.player(P0).life;
    cardcode::keyword_attack(&mut g, P0, &p, P1).unwrap();
    assert_eq!(g.player(P0).life, life + 1);
}

// ------------------------------------------------------------------ the plan
#[test]
fn the_recursion_spells_wait_for_a_worthwhile_graveyard_card() {
    let mut g = game();
    mana(&mut g, P0, 6);
    let c = hand(&mut g, P0, &["Rewrite Regrets"])[0];
    assert_eq!(sim_core::impls::yawgmoth::yawgmoth_prio(&g, P0, c), 0);
    to_gy(&mut g, P0, &["Phyrexian Delver"]);
    assert!(sim_core::impls::yawgmoth::yawgmoth_prio(&g, P0, c) > 50);
}

#[test]
fn yawgmoth_is_cast_first() {
    let g = game();
    let cmd = g.player(P0).cmd;
    assert!(sim_core::impls::yawgmoth::yawgmoth_prio(&g, P0, cmd) >= 85);
}
