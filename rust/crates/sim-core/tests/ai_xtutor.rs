//! t6-xtutor (Tier 6 fix 1, `rust/agents/t6-xtutor.md`): the line planner casts a creature X tutor toward a line, at
//! the X its target needs (`planner::xtutor_rule`, `Player::plan_x`), when the mana to cast it and then reach the line
//! exists; and plays as before with `AI_XTUTOR=0` (`Settings::xtutor` off).
//!
//! Two lines of Kinnan's deck carry the tests, the creature piece in the library:
//! - Kinnan + Basalt Monolith + Thrasios, Triton Hero (the outlet when {U}{G} can't be paid): Kinnan and Basalt on the
//!   battlefield, only Forests and Wastes (no {U}), Thrasios (MV 2, green, a non-Human) the tutor's target. Free to go.
//! - Hullbreaker Horror + Kinnan + two rocks: Kinnan and Sol Ring out, Arcane Signet ({2}) in hand, the Horror (MV 7)
//!   the target: the mana left after the tutor decides whether the line is this turn's or next turn's.

use sim_core::ai::planner::{self, Step};
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step as TurnStep};
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);
const KINNAN: &str = "kinnan-bonder-mana-combo";

/// Kinnan's seat against three inert goldfish seats (they never cast, their hands are empty), its hand emptied
fn kinnan_goldfish() -> Game {
    let mut g =
        table(&[KINNAN, "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "rograkh-silas-turbo-breach"]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 5;
    g.round = 5;
    g.log = Some(vec![]);
    g
}

fn set_xtutor(g: &mut Game, on: bool) {
    let mut s = (*g.settings).clone();
    s.xtutor = on;
    g.settings = Arc::new(s);
}

/// the line's creature must be in the library for a tutor to find it (it is, in Kinnan's deck)
fn in_lib(g: &Game, name: &str) -> bool {
    g.player(P0).library.contains(&card(g, name))
}

fn log_has(g: &Game, s: &str) -> bool {
    g.log.as_ref().is_some_and(|l| l.iter().any(|x| x.contains(s)))
}

/// Kinnan and Basalt Monolith out, Forests for mana, the tutor in hand: Thrasios in the library is the outlet
fn thrasios_table(tutor: &str, forests: usize) -> Game {
    let mut g = kinnan_goldfish();
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    lands(&mut g, P0, "Forest", forests, false);
    hand(&mut g, P0, &[tutor]);
    assert!(in_lib(&g, "Thrasios, Triton Hero"));
    g
}

/// every X tutor of the brief is cast for Thrasios at X = 2 (the card's mana value, not all the mana), the line follows
/// in the same main phase, and the trace names the X
fn cast_at_the_right_x(tutor: &str) {
    let mut g = thrasios_table(tutor, 8);
    let (t, thr) = (card(&g, tutor), card(&g, "Thrasios, Triton Hero"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().unwrap_or_else(|| panic!("{tutor}: no plan this turn"));
    assert_eq!(plan.len(), 2, "{tutor}: {plan:?}");
    assert_eq!(plan[0], Step::Tutor(t, thr), "{tutor}: {plan:?}");
    assert!(matches!(plan[1], Step::Go(_)));
    assert_eq!(planner::tutor_x(&g, plan[0]), 2);
    // the real cast: pays the tutor's cost plus X = 2, not the whole board's mana
    let untapped = |g: &Game| g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count();
    assert_eq!(untapped(&g), 8);
    assert!(planner::perform(&mut g, P0, plan[0]).unwrap(), "{tutor}: the cast");
    let (gn, pips) = sim_core::engine::mana::cost_of(&g, P0, t);
    let spent = 8 - untapped(&g);
    assert!(spent as u32 <= gn + pips.len() as u32 + 2, "{tutor}: paid {spent} lands for X = 2");
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(thr)), "{tutor}: Thrasios is on the battlefield");
    assert!(g.player(P0).plan_x.is_none() && g.player(P0).plan_tutor.is_none(), "the planner's choice is cleared");
}

#[test]
fn finale_of_devastation_is_cast_at_x_2() {
    cast_at_the_right_x("Finale of Devastation");
}

#[test]
fn green_suns_zenith_is_cast_at_x_2() {
    cast_at_the_right_x("Green Sun's Zenith");
}

#[test]
fn chord_of_calling_is_cast_at_x_2() {
    cast_at_the_right_x("Chord of Calling");
}

#[test]
fn natures_rhythm_is_cast_at_x_2() {
    cast_at_the_right_x("Nature's Rhythm");
}

#[test]
fn invasion_of_ikoria_is_cast_at_x_2() {
    cast_at_the_right_x("Invasion of Ikoria // Zilortha, Apex of Ikoria");
}

/// the main phase makes the plan's play, and the trace names the X
#[test]
fn the_main_phase_casts_finale_and_the_trace_names_x() {
    let mut g = thrasios_table("Finale of Devastation", 8);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(log_has(&g, "[plan: Finale of Devastation for Thrasios, Triton Hero (X=2)"), "{:?}", g.log);
}

/// Green Sun's Zenith finds a green creature only: a blue one (Hullbreaker Horror) is not its target
#[test]
fn green_suns_zenith_never_targets_a_blue_creature() {
    let g = thrasios_table("Green Sun's Zenith", 12);
    let (gsz, horror, thr) =
        (card(&g, "Green Sun's Zenith"), card(&g, "Hullbreaker Horror"), card(&g, "Thrasios, Triton Hero"));
    let ok = planner::xtutor_rule(&g, gsz).expect("a creature X tutor");
    assert!(!ok(horror) && ok(thr));
}

/// not enough mana for the tutor at the X its target needs (Finale at X = 2 is GG + 2 = 4 mana, with Basalt's six
/// colourless, but the {G}{G} needs two green sources): no play
#[test]
fn it_is_not_cast_when_it_cannot_be_paid_at_that_x() {
    let mut g = thrasios_table("Finale of Devastation", 1);
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert_eq!(plan, None);
    let next = planner::plan_next_turn(&mut g, P0, false).unwrap();
    assert!(next.as_ref().is_none_or(|p| p.iter().all(|s| !matches!(s, Step::Tutor(..)))), "{next:?}");
}

// ------------------------------------------------------------------ the mana left for the line
/// Kinnan and Sol Ring out, Arcane Signet in hand, Forests: the Horror (MV 7) is the piece, Chord of Calling the tutor
fn horror_table(forests: usize) -> Game {
    let mut g = kinnan_goldfish();
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Sol Ring");
    lands(&mut g, P0, "Forest", forests, false);
    hand(&mut g, P0, &["Chord of Calling", "Arcane Signet"]);
    assert!(in_lib(&g, "Hullbreaker Horror"));
    g
}

fn tutors(plan: &[Step]) -> usize {
    plan.iter().filter(|s| matches!(s, Step::Tutor(..))).count()
}

/// with the mana for Chord at X = 7 and the rock afterwards, the line is this turn's; with the tutor affordable but
/// not the rock too, the tutor is cast now and the line is next turn's (all untaps); with the tutor itself out of
/// reach (X = 7 is ten mana) nothing is cast
#[test]
fn the_tutor_is_cast_only_when_the_line_is_reachable_after_it() {
    // Chord at X = 7 is ten mana: Kinnan doubles Sol Ring, Kinnan convokes. 9 Forests pay for it and the Signet after
    // it; 6 pay for the tutor alone (the Signet then waits for the next turn); 5 can't pay for the tutor at all.
    let mut rich = horror_table(9);
    let plan = planner::plan_this_turn(&mut rich, P0).unwrap().expect("a line this turn");
    assert_eq!(tutors(&plan), 1, "{plan:?}");
    assert_eq!(planner::tutor_x(&rich, plan[0]), 7, "X is the Horror's mana value, not all the mana");
    assert!(matches!(plan[1], Step::Go(_)));

    // the tutor is affordable but the line is not, this turn: the tutor is cast now and the line is next turn's
    let mut tight = horror_table(8);
    assert_eq!(planner::plan_this_turn(&mut tight, P0).unwrap(), None);
    let next = planner::plan_next_turn(&mut tight, P0, false).unwrap().expect("a plan for next turn");
    assert_eq!((tutors(&next), planner::tutor_x(&tight, next[0])), (1, 7), "{next:?}");

    // the tutor itself is out of reach: nothing is cast
    let mut poor = horror_table(5);
    assert_eq!(planner::plan_this_turn(&mut poor, P0).unwrap(), None);
    assert_eq!(planner::plan_next_turn(&mut poor, P0, false).unwrap(), None);
}

/// Chord of Calling is an instant: at the end of the turn before ours the window casts it at X = 7 for the Horror, and
/// the turn's untap leaves the line ready
#[test]
fn chord_is_cast_in_the_end_of_turn_window_at_its_x() {
    let mut g = horror_table(8);
    g.player_mut(P0).turns = 4;
    g.active = Some(PlayerId(3));
    g.step = TurnStep::End;
    let (step, _, why) = planner::eot_play(&mut g, P0).unwrap().expect("an end-of-turn play");
    assert_eq!(why, planner::READY);
    assert_eq!(planner::tutor_x(&g, step), 7);
    sim_core::ai::end_of_turn_window(&mut g, P0).unwrap();
    assert!(log_has(&g, "end of turn: Chord of Calling for Hullbreaker Horror (X=7) [plan:"), "{:?}", g.log);
    let horror = card(&g, "Hullbreaker Horror");
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(horror)), "the Horror is on the battlefield");
}

// ------------------------------------------------------------------ the switch
/// with `AI_XTUTOR=0` the planner never plans an X tutor and the cast is as before (X = all the mana)
#[test]
fn off_is_todays_play() {
    let mut g = thrasios_table("Finale of Devastation", 8);
    set_xtutor(&mut g, false);
    let fin = card(&g, "Finale of Devastation");
    assert!(planner::xtutor_rule(&g, fin).is_none());
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert!(plan.as_ref().is_none_or(|p| tutors(p) == 0), "{plan:?}");
    g.step = TurnStep::Main1;
}

#[test]
fn the_switch_reads_like_the_planner_switch() {
    use sim_core::settings::planner_from;
    assert!(planner_from(None) && planner_from(Some("1")) && planner_from(Some("")));
    assert!(!planner_from(Some("0")) && !planner_from(Some("off")));
}
