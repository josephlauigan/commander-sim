//! Kefka's helper cards for the Underworld Breach line: Grapeshot (storm card), Cabal Ritual and Jeska's Will (mana),
//! Gamble (tutor for Breach).

use sim_core::ai::{act, brain, decks};
use sim_core::engine::cast::cast_card;
use sim_core::ids::PlayerId;
use sim_core::impls::mine;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Kefka against one Veyran with `opp_library` cards left (no hand), `fuel` Islands in Kefka's graveyard
fn board(swamps: usize, mountains: usize, islands: usize, opp_library: usize, fuel: usize) -> Game {
    let mut g = table(&["kefka", "veyran"]);
    let pl = g.player_mut(P1);
    pl.hand.clear();
    let rest = pl.library.split_off(opp_library.min(pl.library.len()));
    pl.gy.extend(rest);
    let island = card(&g, "Island");
    let k = g.player_mut(P0);
    k.hand.clear();
    k.gy = vec![island; fuel];
    lands(&mut g, P0, "Swamp", swamps, false);
    lands(&mut g, P0, "Mountain", mountains, false);
    lands(&mut g, P0, "Island", islands, false);
    g
}

fn run_line(g: &mut Game) {
    assert!(mine::breach_line(g, P0, false).unwrap(), "the dry run finishes the table");
    let opts = brain::main_options(g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == "Underworld Breach line").expect("line offered");
    act::perform(g, P0, o.act.as_ref().unwrap()).unwrap();
}

#[test]
fn grapeshot_burns_the_table_out_in_the_line() {
    let mut g = board(3, 3, 1, 40, 8);
    g.player_mut(P1).life = 4;
    hand(&mut g, P0, &["Underworld Breach", "Grapeshot", "Dark Ritual", "Lotus Petal"]);
    run_line(&mut g);
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive, "life {}", g.player(P1).life);
}

#[test]
fn cabal_ritual_pays_for_the_line() {
    // Cabal Ritual's {B}{B}{B} pays Brain Freeze's generic cost and adds storm
    let mut g = board(1, 1, 3, 9, 8);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Cabal Ritual"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap(), "line with Cabal Ritual");
    run_line(&mut g);
    assert!(g.player(P1).library.is_empty());
    let cr = card(&g, "Cabal Ritual");
    assert!(g.player(P0).cast_names.contains(&cr));
    let mut g = board(1, 1, 3, 9, 8);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap(), "without the ritual it is too slow");
}

#[test]
fn jeskas_will_adds_red_for_the_opponents_hand_in_the_line() {
    let mut g = board(1, 2, 3, 9, 8);
    let island = card(&g, "Island");
    g.player_mut(P1).hand = vec![island; 6];
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Jeska's Will"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap(), "line with Jeska's Will");
    run_line(&mut g);
    assert!(g.player(P1).library.is_empty());
    let jw = card(&g, "Jeska's Will");
    assert!(g.player(P0).cast_names.contains(&jw));
}

#[test]
fn gamble_tutors_breach_and_is_cast_for_it() {
    let mut g = board(3, 3, 1, 40, 0);
    let ub = card(&g, "Underworld Breach");
    let bf = card(&g, "Brain Freeze");
    g.player_mut(P0).library.extend([bf, ub]);
    let mut dn = g.player(P0).deck_names.to_vec();
    dn.push(bf);
    g.player_mut(P0).deck_names = dn.into();
    let h = hand(&mut g, P0, &["Gamble", "Island", "Island", "Island"]);
    assert!(decks::deck_prio(&g, P0, h[0]) > 0, "Gamble has a cast priority in Kefka");
    assert_eq!(&*g.db.get(decks::tutor_pick(&g, P0, "any").unwrap()).name, "Underworld Breach");
    assert!(cast_card(&mut g, P0, h[0], "hand", Ctx::default()).unwrap());
    let pl = g.player(P0);
    assert!(pl.hand.contains(&ub) || pl.gy.contains(&ub), "Breach was found (then maybe discarded at random)");
    assert_eq!(pl.hand.len() + pl.gy.len(), 5, "one found, one discarded at random");
}
