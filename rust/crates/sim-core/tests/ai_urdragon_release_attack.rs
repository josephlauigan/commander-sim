//! The Ur-Dragon seat's Twinflame lethal release commits the turn's first combat: after the cast, the AI attacks with
//! everything at the opponent the engine check proved, and not at whoever the defender heuristic would pick.
//! The table: seat 0 (urdragon) with Twinflame in hand, three Mountains and an untapped Ancient Silver Dragon (8/8
//! flier), so the cast makes one hasty copy and 16 flying damage. Seat 1 (seph) is at 16 life: the check proves it.
//! Seat 2 (aragorn) is at 20 life: the heuristic's kill bonus reaches it too (16 is at least 80% of 20), and a huge
//! grudge makes it the defender the heuristic prefers. The look-ahead is off (the test game's default).
mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::{combat, turn};
use sim_core::state::Step;

const GARTH: &str = "Garth One-Eye";
const SPARK: &str = "Spark Double";
const SEASON: &str = "Doubling Season";
const TWIN: &str = "Twinflame";
const BIG: &str = "Ancient Silver Dragon";
const Q: PlayerId = P1; // the opponent the check proves
const OTHER: PlayerId = P2; // the opponent the heuristic prefers

/// seat 0 = urdragon with the Garth line alive (the three pieces in the library), Twinflame in hand, three Mountains
/// and the big flier; seats 1 and 2 at 16 life, seat 2 the heuristic's favourite
fn base(seed: u64) -> Game {
    let mut g = table_with(&["urdragon", "seph", "aragorn"], seed, 40);
    for p in &mut g.players {
        p.hand.clear();
        p.gy.clear();
    }
    for name in [GARTH, SPARK, SEASON] {
        let c = take(&mut g, P0, name);
        g.player_mut(P0).library.push(c);
    }
    hand(&mut g, P0, &[TWIN]);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, BIG);
    g.player_mut(Q).life = 16;
    g.player_mut(OTHER).life = 16;
    g.player_mut(P0).grudge.insert("aragorn", 1000.0);
    g
}

/// the Twinflame option the AI is shown in its first main phase
fn twin_option(g: &mut Game) -> sim_core::hooks::Opt {
    let opts = sim_core::ai::brain::main_options(g, P0, false).unwrap();
    opts.into_iter().find(|o| o.label.starts_with("Twinflame (")).expect("the release option")
}

/// the real cast of the offered option
fn cast_release(g: &mut Game) {
    let o = twin_option(g);
    assert!(perform(g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).gy.contains(&card(g, TWIN)), "Twinflame resolved");
}

fn dead(g: &Game, q: PlayerId) -> bool {
    !g.player(q).alive || g.player(q).life <= 0
}

/// (a) After a real release cast the combat attacks the proven opponent with everything: the one the heuristic would
/// pick (a grudge) must stay alive, and the proven one must die.
#[test]
fn release_cast_then_combat_attacks_the_proven_opponent_and_kills_it() {
    for seed in 1..=6 {
        let mut g = base(seed);
        cast_release(&mut g);
        turn::end_step_pools(&mut g);
        g.step = Step::Combat;
        combat::combat(&mut g, P0).unwrap();
        assert!(dead(&g, Q), "seed {seed}: the proven opponent survived the combat (life {})", g.player(Q).life);
        assert!(!dead(&g, OTHER), "seed {seed}: the other opponent was hit instead");
    }
}

/// (b) `ai::choose_attack` hands the first combat the proven plan right after the cast.
#[test]
fn choose_attack_returns_all_at_the_proven_opponent_after_a_release_cast() {
    let mut g = base(1);
    cast_release(&mut g);
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), Some((Q, "all")));
}

/// (c) End to end: a whole turn of the AI from the first main phase. The release is cast and the proven opponent is
/// dead before the turn is over.
#[test]
fn a_whole_turn_with_the_release_kills_the_proven_opponent() {
    for seed in 1..=6 {
        let mut g = base(seed);
        turn::continue_turn(&mut g, P0, Step::Main1).unwrap();
        assert!(g.player(P0).gy.contains(&card(&g, TWIN)), "seed {seed}: Twinflame was cast");
        assert!(dead(&g, Q), "seed {seed}: the proven opponent is alive at the end of the turn");
    }
}
