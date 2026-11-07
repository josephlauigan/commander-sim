//! The Underworld Breach line (impls/breach_line.rs) on hand-built positions: offered only when the dry run finishes
//! the table, and played for real then; any deck holding Breach and a storm card runs it (Aragorn's red rituals and a
//! pool deck's black ones).

use sim_core::ai::act;
use sim_core::ids::PlayerId;
use sim_core::impls::breach_line;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// the line's own cards: never used as plain fuel
const LINE: [&str; 11] = ["Brain Freeze", "Grapeshot", "Lotus Petal", "Dark Ritual", "Cabal Ritual", "Seething Song",
    "Desperate Ritual", "Rite of Flame", "Lion's Eye Diamond", "Jeska's Will", "Underworld Breach"];

/// P0 in its main phase with Breach out, `hand`, `n_fuel` other cards in its graveyard and `lands` untapped Volcanic
/// Islands; each opponent has `opp_lib` cards left in its library
fn position(key: &str, hand_names: &[&str], n_fuel: usize, lands_n: usize, opp_lib: usize) -> Game {
    let mut g = table(&[key, "cloud", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).turns = 6;
    g.player_mut(P0).hand.clear();
    perm(&mut g, P0, "Underworld Breach");
    lands(&mut g, P0, "Volcanic Island", lands_n, false);
    hand(&mut g, P0, hand_names);
    // fuel: plain cards off the top of the library (no line card among them)
    let line = LINE;
    let mut fuel = vec![];
    while fuel.len() < n_fuel {
        let c = g.player_mut(P0).library.pop().unwrap();
        if !line.contains(&&*g.db.get(c).name) {
            fuel.push(c);
        }
    }
    g.player_mut(P0).gy.extend(fuel);
    for q in 1..4 {
        let pl = g.player_mut(PlayerId(q));
        pl.library.truncate(opp_lib);
    }
    g
}

fn table_finished(g: &Game) -> bool {
    g.winner == Some(P0) || g.opps(P0).all(|q| !g.player(q).alive || g.player(q).library.is_empty())
}

#[test]
fn aragorn_takes_the_line_when_it_finishes_the_table() {
    let mut g = position(
        "aragorn",
        &["Brain Freeze", "Grapeshot", "Seething Song", "Desperate Ritual", "Rite of Flame", "Lotus Petal"],
        12,
        4,
        9,
    );
    let opts = breach_line::breach_options(&mut g, P0, false).unwrap();
    assert_eq!(opts.len(), 1, "the dry run finishes the table, so the line is offered");
    assert_eq!(opts[0].label, "Underworld Breach line");
    act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap();
    assert!(table_finished(&g), "every opponent dead or milled out");
    assert_eq!(g.player(P0).stats.get("breach_line").copied(), Some(1));
}

#[test]
fn no_line_without_fuel_or_mana() {
    // two cards in the graveyard and one land: no escape, too little mana
    let mut g = position("aragorn", &["Brain Freeze", "Grapeshot", "Rite of Flame"], 2, 1, 40);
    assert!(breach_line::breach_options(&mut g, P0, false).unwrap().is_empty());
}

#[test]
fn no_line_when_it_would_not_finish_the_table() {
    // full libraries and 40 life: the pieces are there but the line falls short, so it is not offered
    let mut g = position("aragorn", &["Brain Freeze", "Rite of Flame"], 6, 3, 90);
    // no line card left in the library either: a self-milling Brain Freeze would otherwise dig up Lotus Petal and
    // Seething Song for Breach to escape (it did after the 2026-11-06 Aragorn list change), and the line would finish
    let lib: Vec<_> = g.player(P0).library.iter().copied().filter(|&c| !LINE.contains(&&*g.db.get(c).name)).collect();
    g.player_mut(P0).library = lib;
    assert!(breach_line::breach_options(&mut g, P0, false).unwrap().is_empty());
}

#[test]
fn a_pool_deck_with_the_pieces_runs_it_too() {
    let mut g = position(
        "rograkh-silas-turbo-breach",
        &["Brain Freeze", "Dark Ritual", "Cabal Ritual", "Lotus Petal", "Rite of Flame"],
        12,
        4,
        9,
    );
    let opts = breach_line::breach_options(&mut g, P0, false).unwrap();
    assert_eq!(opts.len(), 1, "any deck holding Breach and a storm card");
    act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap();
    assert!(table_finished(&g));
}
