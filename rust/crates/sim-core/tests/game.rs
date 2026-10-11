//! Whole games in Rust: every deck at every tier plays to the end, and a seed always plays the same game
//! (Undo and saved games depend on it).

use sim_core::testkit::*;

#[test]
fn every_deck_finishes_games_at_every_tier() {
    let mine: Vec<&str> = decks().iter().filter(|d| d.tier.is_none()).map(|d| &*d.key).collect();
    let mut played = 0;
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let pool = tier(t);
        for (i, me) in mine.iter().enumerate() {
            let seed = 900_000 + i as u64;
            let g = play(&[me, pool[i % 5], pool[(i + 1) % 5], pool[(i + 2) % 5]], seed, false);
            assert!(g.winner.is_some() || g.players.iter().all(|q| !q.alive), "{me} vs {t}: no result");
            assert!(g.wintype.is_some());
            assert!(g.stack.is_empty(), "{me} vs {t}: the game ended mid-stack");
            // triggers still waiting are fine once the game is over (Python stops resolving them too)
            assert!(g.over || g.trig_queue.is_empty(), "{me} vs {t}: triggers left waiting in a game still on");
            played += 1;
        }
    }
    assert_eq!(played, 35);
}

#[test]
fn a_seed_plays_the_same_game() {
    let t1 = tier("t1");
    let keys = ["sauron", t1[0], t1[1], t1[2]];
    let a = play(&keys, 500_001, true);
    let b = play(&keys, 500_001, true);
    assert_eq!(a.log, b.log);
    let c = play(&keys, 500_002, true);
    assert_ne!(a.log, c.log);
}

#[test]
fn a_copied_game_plays_on_independently() {
    use sim_core::engine::turn::{run_rounds, setup_game};
    let t2 = tier("t2");
    let seats: Vec<_> = ["seph", t2[0], t2[1], t2[2]].iter().map(|k| seat_spec(k)).collect();
    let settings = std::sync::Arc::new(sim_core::settings::Settings::new(
        sim_core::settings::Profile::Loose,
        sim_core::settings::AiMode::Adaptive,
        1.0,
    ));
    let g = setup_game(db(), registry(), settings, 7, &seats, false);
    let mut a = g.clone();
    let mut b = g.clone();
    run_rounds(&mut a, 20);
    run_rounds(&mut b, 20);
    assert_eq!((a.winner, a.round, a.work), (b.winner, b.round, b.work)); // the same start plays the same game
    assert_eq!(g.round, 0); // and the original is untouched
}
