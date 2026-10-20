//! The Cruelty of Gix (Seph FS, seat 0 sephfs vs sauron) and K'rrik, Son of Yawgmoth (Sheoldred, seat 0 sheoldred vs
//! sephfs). Each expectation comes from the card's Oracle text.

mod play_common;
use play_common::*;
use sim_core::ai::brain;
use sim_core::engine::{cast, turn, values};
use sim_core::human::legal;
use sim_core::impls::{krrik, sephfs_cards};
use sim_core::state::{Ctx, DataKey};

const GIX: &str = "The Cruelty of Gix";
const KRRIK: &str = "K'rrik, Son of Yawgmoth";

fn gix_game() -> Game {
    table(&["sephfs", "sauron"])
}

fn cast_gix(g: &mut Game) {
    let c = hand(g, P0, &[GIX])[0];
    cast::cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}

fn bury(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
}

fn lore(g: &Game) -> Option<i64> {
    perms_named(g, P0, GIX).first().map(|&m| g.perm(m).data.int(DataKey::Lore))
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a person who answers prompts starting with a key by that pick, and declines the rest
fn person(g: &mut Game, p: PlayerId, picks: Vec<(&'static str, usize)>) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        match picks.iter().find(|(s, _)| r.prompt.starts_with(s)) {
            Some(&(_, i)) => Answer::Pick(i),
            None => Answer::Decline,
        }
    }));
    g.humans.add(p, h.clone());
    h
}

// ================================================================== The Cruelty of Gix: the AI's chapter
#[test]
fn gix_ai_reads_ahead_to_three_when_an_opponents_graveyard_holds_a_good_creature() {
    let mut g = gix_game();
    bury(&mut g, P1, "Massacre Wurm"); // after a wipe: the opponent's graveyard
    assert_eq!(sephfs_cards::gix_start(&g, P0), 3);
    cast_gix(&mut g);
    assert!(has_perm(&g, P0, "Massacre Wurm"), "chapter III: under my control");
    assert!(!in_gy(&g, P1, "Massacre Wurm"));
    assert!(!has_perm(&g, P0, GIX), "sacrificed after III");
}

#[test]
fn gix_ai_reads_ahead_to_two_when_the_deck_wants_a_piece_and_life_allows() {
    let mut g = gix_game();
    lands(&mut g, P0, "Swamp", 1, false); // short of mana: the deck's tutor wants Sol Ring
    let sol = card(&g, "Sol Ring");
    take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).library.push(sol);
    let db = g.db.clone();
    g.player_mut(P0).hand.retain(|&c| !db.get(c).land);
    assert_eq!(sephfs_cards::gix_start(&g, P0), 2);
    let life = g.player(P0).life;
    let hand0 = g.player(P0).hand.len();
    cast_gix(&mut g);
    assert_eq!(lore(&g), Some(2), "chapter II is on the Saga");
    assert_eq!(g.player(P0).life, life - 3, "II: lose 3 life");
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "II: a card from the library (the Saga left the hand)");
    // the Saga goes on to III next turn and is sacrificed
    turn::upkeep(&mut g, P0).unwrap();
    assert!(!has_perm(&g, P0, GIX));
}

#[test]
fn gix_ai_reads_ahead_to_one_with_low_life_or_no_need_and_takes_the_best_card() {
    let mut g = gix_game();
    g.player_mut(P0).life = 10; // too low to pay 3 for the tutor
    assert_eq!(sephfs_cards::gix_start(&g, P0), 1);
    let slaver = hand(&mut g, P1, &["Dread Slaver"])[0];
    hand(&mut g, P1, &["Sol Ring"]);
    cast_gix(&mut g);
    assert_eq!(lore(&g), Some(1));
    assert!(in_gy(&g, P1, "Dread Slaver") && g.player(P1).gy.contains(&slaver), "I: the creature card is discarded");
    assert!(g.player(P1).hand.iter().any(|&c| &*g.db.get(c).name == "Sol Ring"), "the non-creature stays");
    // next turn chapter II (life 10 -> 7 after the tutor), then III the turn after
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(lore(&g), Some(2));
    assert_eq!(g.player(P0).life, 7);
    turn::upkeep(&mut g, P0).unwrap();
    assert!(!has_perm(&g, P0, GIX), "sacrificed after III");
}

// ================================================================== The Cruelty of Gix: a person
#[test]
fn gix_person_picks_the_chapter_and_the_cards() {
    // chapter I
    let mut g = gix_game();
    hand(&mut g, P1, &["Dread Slaver", "Sol Ring"]);
    let _h = person(&mut g, P0, vec![("The Cruelty of Gix (read ahead)", 0), ("The Cruelty of Gix: that player", 0)]);
    cast_gix(&mut g);
    assert_eq!(lore(&g), Some(1));
    assert!(in_gy(&g, P1, "Dread Slaver"), "I: the person chose the only creature or planeswalker card");
    // chapter II
    let mut g = gix_game();
    let life = g.player(P0).life;
    let h = person(&mut g, P0, vec![("The Cruelty of Gix (read ahead)", 1), ("Search your library", 0)]);
    cast_gix(&mut g);
    assert_eq!(lore(&g), Some(2));
    assert_eq!(g.player(P0).life, life - 3);
    assert!(!h.asked_kind("search").is_empty(), "II: the person searched");
    // chapter III, from an opponent's graveyard
    let mut g = gix_game();
    bury(&mut g, P0, "Vampire Nighthawk");
    bury(&mut g, P1, "Massacre Wurm");
    let h = person(&mut g, P0, vec![("The Cruelty of Gix (read ahead)", 2), ("The Cruelty of Gix: put which", 1)]);
    cast_gix(&mut g);
    let q = h.asked().into_iter().find(|r| r.prompt.starts_with("The Cruelty of Gix: put which")).unwrap();
    assert_eq!(q.choices.len(), 2, "both graveyards are offered: {:?}", q.choices);
    assert!(has_perm(&g, P0, "Vampire Nighthawk") || has_perm(&g, P0, "Massacre Wurm"));
    assert!(!has_perm(&g, P0, GIX), "sacrificed after III");
}

// ================================================================== K'rrik
fn krrik_game(life: i32) -> Game {
    let mut g = table(&["sheoldred", "sephfs"]);
    g.active = Some(P0);
    g.player_mut(P0).life = life;
    perm(&mut g, P0, KRRIK);
    g
}

#[test]
fn krrik_grows_on_each_black_spell_only() {
    let mut g = krrik_game(40);
    let k = perms_named(&g, P0, KRRIK)[0];
    let base = values::epow(&g, k);
    lands(&mut g, P0, "Swamp", 5, false);
    let c = hand(&mut g, P0, &["Dread Slaver"])[0];
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(values::epow(&g, k), base + 1, "a black spell: one +1/+1 counter");
    lands(&mut g, P0, "Plains", 1, false);
    let s = hand(&mut g, P0, &["Sol Ring"])[0];
    assert!(brain::do_cast(&mut g, P0, s, None).unwrap());
    assert_eq!(values::epow(&g, k), base + 1, "a colourless spell: nothing");
}

#[test]
fn krrik_ai_pays_life_for_black_pips_above_the_floor() {
    let mut g = krrik_game(19);
    lands(&mut g, P0, "Swamp", 3, false); // Dread Slaver costs {3}{B}{B}: the mana pays only the {3}
    let c = hand(&mut g, P0, &["Dread Slaver"])[0];
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(g.player(P0).life, 19 - 4, "two black pips paid with 2 life each: 15, the floor");
}

#[test]
fn krrik_ai_keeps_the_floor() {
    let mut g = krrik_game(18); // paying both would leave 14
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Dread Slaver"])[0];
    assert!(!brain::do_cast(&mut g, P0, c, None).unwrap(), "can't pay {{3}}{{B}}{{B}} with 3 lands and 2 life only");
    assert_eq!(g.player(P0).life, 18);
    assert!(g.player(P0).hand.contains(&c));
    assert_eq!(krrik::LIFE_FLOOR, 15);
}

#[test]
fn krrik_pays_nothing_when_the_mana_is_there() {
    let mut g = krrik_game(40);
    lands(&mut g, P0, "Swamp", 5, false);
    let c = hand(&mut g, P0, &["Dread Slaver"])[0];
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn krrik_own_phyrexian_cost_can_be_paid_with_life() {
    let mut g = table(&["sheoldred", "sephfs"]);
    lands(&mut g, P0, "Swamp", 4, false); // {4}{B/P}{B/P}{B/P}: three pips with life
    let c = hand(&mut g, P0, &[KRRIK])[0];
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert!(has_perm(&g, P0, KRRIK));
    assert_eq!(g.player(P0).life, 34);
}

#[test]
fn krrik_gives_a_person_the_choice_for_every_black_pip() {
    let mut g = krrik_game(40);
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Dread Slaver"])[0];
    assert_eq!(legal::phyrexian(&g, P0, c), "BB");
    let ways = legal::phyrexian_ways(&g, P0, c, 3, "BB");
    assert!(ways.iter().any(|(l, left)| *l == 4 && left.is_empty()), "{ways:?}");
    // without K'rrik the card has no Phyrexian mana
    let g2 = table(&["sheoldred", "sephfs"]);
    let c2 = g2.db.id("Dread Slaver").unwrap();
    assert_eq!(legal::phyrexian(&g2, P0, c2), "");
}
