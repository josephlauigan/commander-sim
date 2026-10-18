//! AI audit phase 2, Seph FS (sephfs): the tutor wish list, Gray Merchant's priority, the Altar + Skeleton + Pawn loop,
//! the plan milestone, Citadel's gate. Seat 0 plays sephfs, seat 1 sauron.

use sim_core::ai::{brain, decks};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::sephfs;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sephfs", "sauron"])
}

fn seph_out(g: &mut Game) -> PermId {
    perm(g, P0, "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel")
}

#[test]
fn tutors_fetch_the_deck_plan_not_the_generic_best() {
    // no Sephiroth, no outlet: the outlet
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Swamp"]);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Phyrexian Altar")));
    // Sephiroth out, three opposing creatures: the wipe
    let mut g = game();
    lands(&mut g, P0, "Swamp", 5, false);
    seph_out(&mut g);
    for _ in 0..3 {
        token(&mut g, P1, 2);
    }
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Massacre Girl")));
    // seven lands: the finisher
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    seph_out(&mut g);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Exsanguinate")));
}

#[test]
fn a_tutor_on_turn_two_without_lands_fetches_mana() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Sol Ring")));
    let g = game();
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Swamp")));
}

#[test]
fn gray_merchant_is_cast_for_the_drain_it_makes() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 5, false);
    let gm = hand(&mut g, P0, &["Gray Merchant of Asphodel"])[0];
    let alone = sephfs::sephfs_prio(&g, P0, gm);
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Zulaport Cutthroat");
    perm(&mut g, P0, "Butcher of Malakir");
    assert!(sephfs::sephfs_prio(&g, P0, gm) > alone + 8);
    g.player_mut(P1).life = 4;
    assert_eq!(sephfs::sephfs_prio(&g, P0, gm), 90); // lethal
}

#[test]
fn the_altar_skeleton_pawn_loop_is_run_and_wins_with_a_drain() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Reassembling Skeleton");
    perm(&mut g, P0, "Pawn of Ulamog");
    assert!(sephfs::altar_loop(&g, P0).is_none()); // no drain yet
    assert_eq!(sephfs::altar_loop_need(&g, P0).len(), 3); // Blood Artist, Zulaport, Bastion
    perm(&mut g, P0, "Blood Artist");
    assert!(sephfs::altar_loop(&g, P0).is_some());
    let o = brain::main_options(&mut g, P0, false).unwrap().into_iter().find(|o| o.label == "Altar + Skeleton + Pawn loop");
    let o = o.expect("the loop is offered");
    assert!(o.utility >= 8.0);
    sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert!(g.over);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn the_last_loop_piece_is_cast_first_and_tutored() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    seph_out(&mut g);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Reassembling Skeleton");
    let pawn = hand(&mut g, P0, &["Pawn of Ulamog"])[0];
    assert_eq!(sephfs::sephfs_prio(&g, P0, pawn), 88);
    g.player_mut(P0).hand.clear();
    let c = take(&mut g, P0, "Pawn of Ulamog");
    g.player_mut(P0).library.push(c);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(c));
}

#[test]
fn the_skeleton_waits_in_the_graveyard_for_its_return() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    seph_out(&mut g);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Pawn of Ulamog");
    let sk = take(&mut g, P0, "Reassembling Skeleton");
    g.player_mut(P0).gy.push(sk);
    assert!(sephfs::altar_loop(&g, P0).is_some()); // {1}{B} brings it back
}

#[test]
fn bolas_citadel_waits_for_an_engine() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 5, false);
    let c = hand(&mut g, P0, &["Bolas's Citadel"])[0];
    assert!(sephfs::sephfs_prio(&g, P0, c) <= 30);
    seph_out(&mut g);
    assert!(sephfs::sephfs_prio(&g, P0, c) >= 0);
}

#[test]
fn the_plan_milestone_is_the_turn_of_the_first_flip() {
    let mut g = game();
    let m = seph_out(&mut g);
    g.player_mut(P0).turns = 5;
    for _ in 0..4 {
        let t = token(&mut g, P0, 1);
        sim_core::engine::zones::die(&mut g, t, "sac").unwrap();
    }
    assert!(g.perm(m).data.truthy(sim_core::state::DataKey::Flipped));
    assert_eq!(g.player(P0).milestone.get("sephfs").copied(), Some(5));
}
