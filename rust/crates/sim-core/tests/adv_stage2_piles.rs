//! Pile split edge cases, LAN-style person seats, malformed answers and zone conservation.
mod play_common;
use play_common::*;
use sim_core::engine::tutors;
use sim_core::flow::Stop;
use sim_core::ids::CardId;
use std::sync::Arc;

const NAMES: [&str; 4] = ["Sol Ring", "Mana Crypt", "Brainstorm", "Lightning Bolt"];

fn game(n: usize) -> Game {
    let mut g = table(&["seph", "cloud"]);
    g.player_mut(P0).library = NAMES[..n].iter().map(|n| card(&g, n)).collect();
    g
}

fn first(g: &mut Game, p: PlayerId) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Pick(0)));
    g.humans.add(p, h.clone());
    h
}

fn owned(g: &Game) -> Vec<CardId> {
    let p = g.player(P0);
    let mut v: Vec<_> = p.library.iter().chain(&p.hand).chain(&p.gy).copied().collect();
    v.sort();
    v
}

#[test]
fn every_pile_size_zero_through_four_ai_person_and_lan_conserves_cards() {
    for gifts in [true, false] {
        for n in 0..=4 {
            for seats in 0..4 {
                let mut g = game(n);
                let before = owned(&g);
                if seats & 1 != 0 {
                    first(&mut g, P0);
                }
                let opponent = (seats & 2 != 0).then(|| first(&mut g, P1));
                tutors::pile_tutor(&mut g, P0, if gifts { 4 } else { 3 }, if gifts { 2 } else { 1 }).unwrap();
                let pile = n.min(if gifts { 4 } else { 3 });
                let hn = if gifts { pile.saturating_sub(2) } else { pile.min(1) };
                assert_eq!(g.player(P0).hand.len(), hn, "gifts={gifts}, n={n}, people={seats}");
                assert_eq!(g.player(P0).gy.len(), pile - hn);
                assert_eq!(owned(&g), before);
                if let Some(h) = opponent {
                    let expected = if hn > 0 && hn < pile { if gifts { 2 } else { 1 } } else { 0 };
                    assert_eq!(h.asked_kind("choose").len(), expected, "gifts={gifts}, n={n}, people={seats}");
                }
            }
        }
    }
}

#[test]
fn gifts_searches_different_names_even_when_library_has_duplicates() {
    for human in [false, true] {
        let mut g = game(2);
        let c = card(&g, NAMES[0]);
        g.player_mut(P0).library.extend([c, c]);
        let before = owned(&g);
        if human {
            first(&mut g, P0);
        }
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert!(g.player(P0).hand.is_empty());
        assert_eq!(g.player(P0).gy.len(), 2);
        assert_eq!(g.player(P0).library, [c, c]);
        assert_eq!(owned(&g), before);
    }
}

fn intuition_same_name(human: bool) {
    // Unlike Gifts Ungiven, Intuition does not require different names.
    let mut g = game(0);
    let c = card(&g, "Island");
    g.player_mut(P0).library = vec![c; 3];
    if human {
        first(&mut g, P0);
    }
    tutors::pile_tutor(&mut g, P0, 3, 1).unwrap();
    assert_eq!(g.player(P0).gy.len(), 2, "person={human}: two copies go to the graveyard");
    assert_eq!(g.player(P0).hand, [c]);
    assert!(g.player(P0).library.is_empty());
}

#[test]
#[ignore = "ticket: Intuition search requires different names"]
fn intuition_ai_may_find_three_cards_with_the_same_name() {
    intuition_same_name(false);
}

#[test]
#[ignore = "ticket: Intuition search requires different names"]
fn intuition_person_may_find_three_cards_with_the_same_name() {
    intuition_same_name(true);
}

#[test]
fn opposition_agent_takes_the_entire_pile_with_ai_searcher() {
    let mut g = game(4);
    perm(&mut g, P1, "Opposition Agent");
    let h = first(&mut g, P1);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert_eq!(g.player(P1).agent_ids.len(), 4);
    assert!(g.player(P0).hand.is_empty() && g.player(P0).gy.is_empty() && g.player(P0).library.is_empty());
    assert!(h.asked_kind("choose").is_empty(), "no split: the entire pile was exiled");
}

#[test]
#[ignore = "ticket: Opposition Agent does not take the pile on the person-searcher path"]
fn opposition_agent_also_takes_the_entire_pile_with_person_searcher() {
    let mut g = game(4);
    perm(&mut g, P1, "Opposition Agent");
    first(&mut g, P0);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert_eq!(g.player(P1).agent_ids.len(), 4, "a person searching still has Opposition Agent's replacement");
    assert!(g.player(P0).hand.is_empty() && g.player(P0).gy.is_empty());
}

#[test]
fn person_caster_can_choose_the_person_among_multiple_opponents() {
    let mut g = table(&["seph", "cloud", "kefka"]);
    g.player_mut(P0).library = NAMES.iter().map(|n| card(&g, n)).collect();
    seat(&mut g, P0, vec![pick(1), pick(0), pick(0), pick(0), pick(0)]);
    let h = first(&mut g, P2);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert_eq!(h.asked_kind("choose").len(), 2);
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).gy.len(), 2);
}

#[test]
fn malformed_split_answers_retry_then_fallback_without_panics_or_card_loss() {
    for answer in [
        Answer::Pick(usize::MAX),
        Answer::Picks(vec![]),
        Answer::Picks(vec![0, 0]),
        Answer::Picks(vec![0, 1, 2]),
        Answer::Action(json!({"card":"not in the pile"})),
        Answer::Decline,
    ] {
        let mut g = game(4);
        let before = owned(&g);
        let h = Arc::new(ScriptHuman::from_fn(move |_| answer.clone()));
        g.humans.add(P1, h.clone());
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert_eq!(g.player(P0).hand.len(), 2);
        assert_eq!(g.player(P0).gy.len(), 2);
        assert_eq!(owned(&g), before);
        assert_eq!(h.told_kind("invalid").len(), 10);
    }
}

#[test]
fn cancelling_an_ai_casters_opponent_split_preserves_the_library() {
    let mut g = game(4);
    let before = owned(&g);
    seat(&mut g, P1, vec![ans(Answer::Cancel)]);
    assert!(matches!(tutors::pile_tutor(&mut g, P0, 4, 2), Err(Stop::Cancelled)));
    assert_eq!(owned(&g), before);
}

#[test]
#[ignore = "ticket: cancelling the opponent split loses the pile"]
fn cancelling_a_person_casters_opponent_split_preserves_the_pile() {
    let mut g = game(4);
    let before = owned(&g);
    first(&mut g, P0);
    seat(&mut g, P1, vec![ans(Answer::Cancel)]);
    assert!(matches!(tutors::pile_tutor(&mut g, P0, 4, 2), Err(Stop::Cancelled)));
    assert_eq!(owned(&g), before, "a cancelled opponent choice must not strand the searched cards outside every zone");
}
