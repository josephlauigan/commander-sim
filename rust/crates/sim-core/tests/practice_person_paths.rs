//! Practice mode: the person's own choices for Gifts Ungiven (the target opponent) and Mindbreak Trap (any number of
//! target spells), checked against the Oracle text and against the AI's play of the same card.
//!
//! Gifts Ungiven: "Search your library for up to four cards with different names and reveal them. Target opponent
//! chooses two of those cards. Put the chosen cards into your graveyard and the rest into your hand. Then shuffle."
//! (the sim's version: the opponent's choice puts two in the hand and two in the graveyard, the engine's convention
//! for the card since the AI's own pick is scored that way). With one opponent there is nothing to choose.

mod parity_common;
mod play_common;
use play_common::*;
use sim_core::engine::tutors;
use std::sync::{Arc, Mutex};

const P3: PlayerId = PlayerId(3);

/// everything the person was asked, as "kind: prompt [choices]"
type Asked = Arc<Mutex<Vec<(String, String, Vec<String>)>>>;

/// seat 0 as a person who answers the target-opponent question with the choice starting with `who` (None: the
/// first) and everything else with the first choice
fn gifts_person(g: &mut Game, who: Option<&'static str>) -> Asked {
    let log: Asked = Arc::new(Mutex::new(vec![]));
    let l2 = log.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push((r.kind.to_string(), r.prompt.clone(), r.choices.clone()));
            if r.prompt.contains("target opponent") {
                let at = who.and_then(|w| r.choices.iter().position(|c| c.starts_with(w)));
                return Answer::Pick(at.unwrap_or(0));
            }
            Answer::Pick(0)
        })),
    );
    log
}

fn target_questions(a: &Asked) -> Vec<Vec<String>> {
    a.lock().unwrap().iter().filter(|x| x.1.contains("target opponent")).map(|x| x.2.clone()).collect()
}

/// the line of the log that names who split the pile
fn bins_line(g: &Game) -> String {
    g.log.as_ref().unwrap().iter().find(|l| l.contains(" bins ")).cloned().expect("a 'bins' line in the log")
}

fn four_seat_game() -> Game {
    let mut g = table(&["seph", "cloud", "kefka", "thanos"]);
    g.log = Some(vec![]);
    g
}

/// Gifts Ungiven's "target opponent": every living opponent of the caster, and nobody else
#[test]
fn gifts_asks_the_person_which_opponent_splits_the_pile() {
    let mut g = four_seat_game();
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    // the Oracle text lists "target opponent": the three other seats, each shown as a player with their life
    let want: Vec<String> = [P1, P2, P3].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "one question, with exactly the three opponents");
}

#[test]
fn gifts_the_opponent_the_person_names_is_the_one_who_splits() {
    for who in [P1, P2, P3] {
        let mut g = four_seat_game();
        let name = g.player(who).name.to_string();
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        let _ = gifts_person(&mut g, Some(leaked));
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert!(bins_line(&g).contains(&format!("; {name} bins ")), "{name} splits: {}", bins_line(&g));
    }
}

#[test]
fn gifts_a_dead_opponent_is_not_offered() {
    let mut g = four_seat_game();
    g.player_mut(P2).alive = false;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    let want: Vec<String> = [P1, P3].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "two living opponents only");
}

#[test]
fn gifts_an_opponent_who_cannot_be_targeted_is_not_offered() {
    // The One Ring: protection from everything, so no spell can target its controller
    let mut g = four_seat_game();
    g.player_mut(P3).ring_prot = true;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    let want: Vec<String> = [P1, P2].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "the protected opponent is not a legal target");
}

#[test]
fn gifts_with_one_opponent_asks_nothing_about_the_target() {
    let mut g = table(&["seph", "cloud"]);
    g.log = Some(vec![]);
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert!(target_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(bins_line(&g).contains(&format!("; {} bins ", g.player(P1).name)));
}

#[test]
fn gifts_with_one_living_opponent_left_asks_nothing_about_the_target() {
    let mut g = four_seat_game();
    g.player_mut(P1).alive = false;
    g.player_mut(P3).alive = false;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert!(target_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(bins_line(&g).contains(&format!("; {} bins ", g.player(P2).name)));
}
