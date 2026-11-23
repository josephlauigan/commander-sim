//! Gifts Ungiven and Intuition split the pile as the opponent chooses, from the Oracle text.
//!
//! Gifts Ungiven: "Search your library for up to four cards with different names and reveal them. Target opponent
//! chooses two of those cards. Put the chosen cards into your graveyard and the rest into your hand. Then shuffle."
//! So a pile of 1, 2, 3 or 4 puts min(2, n) cards into the graveyard.
//! Intuition: "Search your library for three cards and reveal them. Target opponent chooses one of them. Put that card
//! into your hand and the rest into your graveyard. Then shuffle."

mod play_common;
use play_common::*;
use sim_core::engine::tutors;
use sim_core::ids::CardId;
use std::sync::{Arc, Mutex};

const NAMES: [&str; 4] = ["Sol Ring", "Mana Crypt", "Brainstorm", "Lightning Bolt"];

/// a two-seat game where seat 0's library holds exactly the first `n` of NAMES (and nothing else), empty hand and
/// graveyard
fn game(n: usize) -> Game {
    let mut g = table(&["seph", "cloud"]);
    let lib: Vec<CardId> = NAMES[..n].iter().map(|&c| card(&g, c)).collect();
    let pl = g.player_mut(P0);
    pl.library = lib;
    pl.hand.clear();
    pl.gy.clear();
    g
}

fn names(g: &Game, cs: &[CardId]) -> Vec<String> {
    let mut v: Vec<String> = cs.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    v.sort();
    v
}

#[test]
fn gifts_puts_two_or_all_of_a_short_pile_in_the_graveyard() {
    for (n, gy, hand) in [(1, 1, 0), (2, 2, 0), (3, 2, 1), (4, 2, 2)] {
        let mut g = game(n);
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert_eq!(g.player(P0).gy.len(), gy, "Gifts, pile of {n}: graveyard");
        assert_eq!(g.player(P0).hand.len(), hand, "Gifts, pile of {n}: hand");
        assert!(g.player(P0).library.is_empty(), "the whole pile left the library");
    }
}

#[test]
fn intuition_puts_one_in_the_hand_and_the_rest_in_the_graveyard() {
    for (n, gy, hand) in [(1, 0, 1), (2, 1, 1), (3, 2, 1)] {
        let mut g = game(n);
        tutors::pile_tutor(&mut g, P0, 3, 1).unwrap();
        assert_eq!(g.player(P0).hand.len(), hand, "Intuition, pile of {n}: hand");
        assert_eq!(g.player(P0).gy.len(), gy, "Intuition, pile of {n}: graveyard");
    }
}

/// seat 0 as a person who takes every card offered by the search (the first choice, each time)
fn searcher(g: &mut Game) {
    g.humans.add(P0, Arc::new(ScriptHuman::from_fn(|_: &Request| Answer::Pick(0))));
}

#[test]
fn gifts_with_a_person_searching_puts_two_or_all_of_a_short_pile_in_the_graveyard() {
    for (n, gy, hand) in [(1, 1, 0), (2, 2, 0), (3, 2, 1), (4, 2, 2)] {
        let mut g = game(n);
        searcher(&mut g);
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert_eq!(g.player(P0).gy.len(), gy, "Gifts, person, pile of {n}: graveyard");
        assert_eq!(g.player(P0).hand.len(), hand, "Gifts, person, pile of {n}: hand");
    }
}

#[test]
fn intuition_with_a_person_searching_puts_one_in_the_hand() {
    for (n, gy, hand) in [(1, 0, 1), (2, 1, 1), (3, 2, 1)] {
        let mut g = game(n);
        searcher(&mut g);
        tutors::pile_tutor(&mut g, P0, 3, 1).unwrap();
        assert_eq!(g.player(P0).hand.len(), hand, "Intuition, person, pile of {n}: hand");
        assert_eq!(g.player(P0).gy.len(), gy, "Intuition, person, pile of {n}: graveyard");
    }
}

type Asked = Arc<Mutex<Vec<(String, Vec<String>)>>>;

/// seat 1 as the opponent who answers each question with choice `at`, recording the prompts and choices
fn opponent(g: &mut Game, at: usize) -> Asked {
    let log: Asked = Arc::new(Mutex::new(vec![]));
    let l2 = log.clone();
    g.humans.add(
        P1,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push((r.prompt.clone(), r.choices.clone()));
            Answer::Pick(at)
        })),
    );
    log
}

#[test]
fn gifts_asks_a_person_opponent_which_two_cards_go_to_the_graveyard() {
    let mut g = game(4);
    let asked = opponent(&mut g, 0);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    let a = asked.lock().unwrap();
    assert_eq!(a.len(), 2, "two cards chosen, one question each: {a:?}");
    assert!(a[0].0.contains("graveyard"), "{}", a[0].0);
    assert_eq!(a[0].1.len(), 4, "the opponent sees all four cards");
    assert_eq!(a[1].1.len(), 3, "and then the three left");
    // the first choice each time: the first card offered, then the first of the rest
    let mut chosen = vec![a[0].1[0].clone(), a[1].1[0].clone()];
    chosen.sort();
    let gy = names(&g, &g.player(P0).gy);
    assert_eq!(gy.len(), 2);
    for c in &chosen {
        assert!(gy.iter().any(|n| c.contains(n.as_str())), "{c} is in the graveyard: {gy:?}");
    }
    assert_eq!(g.player(P0).hand.len(), 2, "the other two are in the hand");
}

#[test]
fn gifts_asks_a_person_opponent_nothing_when_the_pile_has_no_choice() {
    for n in [1, 2] {
        let mut g = game(n);
        let asked = opponent(&mut g, 0);
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert!(asked.lock().unwrap().is_empty(), "a pile of {n} all goes to the graveyard");
        assert_eq!(g.player(P0).gy.len(), n);
    }
}

#[test]
fn intuition_asks_a_person_opponent_which_card_goes_to_the_hand() {
    let mut g = game(3);
    let asked = opponent(&mut g, 1);
    tutors::pile_tutor(&mut g, P0, 3, 1).unwrap();
    let a = asked.lock().unwrap();
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].0.contains("hand"), "{}", a[0].0);
    let hand = names(&g, &g.player(P0).hand);
    assert_eq!(hand.len(), 1);
    assert!(a[0].1[1].contains(hand[0].as_str()), "the second card offered, {}, is the one in hand", a[0].1[1]);
    assert_eq!(g.player(P0).gy.len(), 2);
}
