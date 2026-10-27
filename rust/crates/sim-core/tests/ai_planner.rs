//! ai-planner (AI foundations step 2): the line planner for the Tier 6 decks (`ai::planner`), one test per behaviour
//! of the brief (`rust/agents/ai-planner.md`).

use sim_core::ai;
use sim_core::ai::planner::{self, Dest, Step};
use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::{Game, Step as TurnStep};
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);

/// the scion seat against three inert goldfish seats (they never cast), its hand emptied, a library of Islands
fn scion_goldfish() -> Game {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P0).library = lib;
    g.log = Some(vec![]);
    g
}

fn set_planner(g: &mut Game, on: bool) {
    let mut s = (*g.settings).clone();
    s.planner = on;
    g.settings = Arc::new(s);
}

fn oracle_line() -> usize {
    combos::combo_index("Thassa's Oracle + Demonic Consultation").unwrap()
}

/// put a card into p's library (below the top card)
fn in_library(g: &mut Game, name: &str) {
    let c = card(g, name);
    g.player_mut(P0).library.insert(0, c);
}

fn log_has(g: &Game, s: &str) -> bool {
    g.log.as_ref().is_some_and(|l| l.iter().any(|x| x.contains(s)))
}

/// Turn 2, two lands, Thassa's Oracle + Demonic Consultation and Dark Ritual: the line is reached now (the ritual's
/// mana counted), so the plan is to go, and the main phase wins with it.
#[test]
fn a_turn_two_oracle_kill_through_a_ritual() {
    let mut g = scion_goldfish();
    g.player_mut(P0).turns = 2;
    lands(&mut g, P0, "Underground Sea", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Dark Ritual"]);
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert_eq!(plan, Some(vec![Step::Go(oracle_line())]));
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(g.over && g.winner == Some(P0), "the ritual pays for the line");
    assert!(log_has(&g, "[plan: combo: Thassa's Oracle"), "the trace names the plan");
}

/// Five lands, Thassa's Oracle and Demonic Tutor: Demonic Tutor finds Demonic Consultation (the missing piece), and
/// the mana left pays for the line this turn.
#[test]
fn a_tutor_finds_the_missing_piece_and_the_line_wins_this_turn() {
    let mut g = scion_goldfish();
    g.player_mut(P0).turns = 4;
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Tutor"]);
    in_library(&mut g, "Demonic Consultation");
    let (dt, consult) = (card(&g, "Demonic Tutor"), card(&g, "Demonic Consultation"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert_eq!(plan, Some(vec![Step::Tutor(dt, consult), Step::Go(oracle_line())]));
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(g.over && g.winner == Some(P0), "tutor, then the line, in one main phase");
    assert!(log_has(&g, "[plan: Demonic Tutor for Demonic Consultation -> combo: Thassa's Oracle"));
}

/// Two lands (the land drop used), Thassa's Oracle + Demonic Consultation and Demonic Tutor: the line lacks only mana
/// (UU + B from two lands). Nothing wins this turn; the tutor finds mana that makes it payable next turn (a land for
/// the next land drop, or Lotus Petal), not another piece.
#[test]
fn a_tutor_fetches_mana_when_the_line_lacks_only_mana() {
    let mut g = scion_goldfish();
    g.player_mut(P0).turns = 3;
    g.player_mut(P0).land_turn = 3;
    lands(&mut g, P0, "Underground Sea", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Demonic Tutor"]);
    in_library(&mut g, "Lotus Petal");
    in_library(&mut g, "Ad Nauseam"); // a piece of another line, which the tutor should not take
    assert_eq!(planner::plan_this_turn(&mut g, P0).unwrap(), None, "two mana can't pay for both");
    let plan = planner::plan_next_turn(&mut g, P0, false).unwrap().expect("a next-turn plan");
    let Step::Tutor(t, x) = plan[0] else { panic!("{plan:?}") };
    assert_eq!(&*g.db.get(t).name, "Demonic Tutor");
    let d = g.db.get(x);
    assert!(d.land || &*d.name == "Lotus Petal", "the tutor fetches mana: {}", d.name);
    // in the game: the main phase makes the play
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(log_has(&g, "[plan: Demonic Tutor for"), "the play is the plan's");
    assert!(!g.player(P0).hand.contains(&card(&g, "Ad Nauseam")));
}

/// The end of the turn before ours, three untapped lands, Thassa's Oracle and Vampiric Tutor: the window casts
/// Vampiric Tutor for Demonic Consultation (to the top), the turn draws it and the line wins.
#[test]
fn a_to_top_tutor_at_the_end_of_turn_then_the_kill() {
    let mut g = scion_goldfish();
    g.player_mut(P0).turns = 3;
    g.round = 3;
    g.active = Some(PlayerId(3));
    g.step = TurnStep::End;
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Vampiric Tutor"]);
    in_library(&mut g, "Demonic Consultation");
    let consult = card(&g, "Demonic Consultation");
    let (step, plan, why) = planner::eot_play(&mut g, P0).unwrap().expect("an end-of-turn plan");
    assert_eq!(why, planner::READY);
    assert_eq!(step, Step::Tutor(card(&g, "Vampiric Tutor"), consult), "{plan:?}");
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert!(log_has(&g, "end of turn: Vampiric Tutor for Demonic Consultation [plan:"));
    assert_eq!(sim_core::engine::tutors::known_top(&g, P0), vec![consult], "the card is on top, and known");
    sim_core::engine::turn::play_turn(&mut g, P0).unwrap();
    assert!(g.over && g.winner == Some(P0), "drawn, then the line");
}

/// A hand with no line in reach: the planner has no plan, and the main phase plays exactly as with the planner off
/// (the same plays, the same log, and the game's random numbers untouched)
#[test]
fn no_plan_in_a_hand_with_no_line_leaves_the_old_choice() {
    let build = || {
        let mut g = scion_goldfish();
        g.player_mut(P0).turns = 3;
        lands(&mut g, P0, "Volcanic Island", 3, false);
        hand(&mut g, P0, &["Sol Ring", "Rhystic Study", "Ragavan, Nimble Pilferer", "Island"]);
        g
    };
    let mut on = build();
    assert_eq!(planner::decide(&mut on, P0).unwrap(), planner::Decision::None);
    ai::brain::main(&mut on, P0, false).unwrap();
    let mut off = build();
    set_planner(&mut off, false);
    ai::brain::main(&mut off, P0, false).unwrap();
    assert_eq!(on.log, off.log);
    assert_eq!(on.player(P0).hand, off.player(P0).hand);
    assert_eq!(on.rng.next_u64(), off.rng.next_u64(), "the planner used none of the game's random numbers");
}

/// Enlightened Tutor finds an artifact or enchantment: the planner never offers it Thassa's Oracle, and a forced
/// Oracle is not taken (the tutor picks as usual among what it may find)
#[test]
fn the_forced_tutor_target_obeys_the_tutors_restriction() {
    let mut g = scion_goldfish();
    g.player_mut(P0).turns = 3;
    lands(&mut g, P0, "Plateau", 2, false);
    hand(&mut g, P0, &["Enlightened Tutor", "Demonic Consultation"]);
    in_library(&mut g, "Thassa's Oracle");
    in_library(&mut g, "Lotus Petal");
    let (et, oracle, petal) = (card(&g, "Enlightened Tutor"), card(&g, "Thassa's Oracle"), card(&g, "Lotus Petal"));
    let (dest, legal) = planner::tutor_targets(&g, P0, et).unwrap();
    assert_eq!(dest, Dest::Top);
    assert!(!legal.contains(&oracle) && legal.contains(&petal), "artifacts and enchantments only");
    assert!(planner::perform(&mut g, P0, Step::Tutor(et, oracle)).unwrap(), "the tutor is cast");
    let top = sim_core::engine::tutors::known_top(&g, P0);
    assert!(!top.contains(&oracle), "Enlightened Tutor can't take Thassa's Oracle: {top:?}");
    assert!(g.player(P0).library.contains(&oracle));
    assert_eq!(g.player(P0).plan_tutor, None, "the choice lasts only for its cast");
    // a legal forced card is taken
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Plateau", 2, false);
    hand(&mut g, P0, &["Enlightened Tutor"]);
    in_library(&mut g, "Lotus Petal");
    in_library(&mut g, "Mox Diamond");
    let moxd = card(&g, "Mox Diamond");
    assert!(planner::perform(&mut g, P0, Step::Tutor(et, moxd)).unwrap());
    assert_eq!(sim_core::engine::tutors::known_top(&g, P0), vec![moxd], "the planner's card, on top and known");
}

/// The planner off (the knob; env PLANNER=0) is the step 1 AI: no plan is made or logged
#[test]
fn planner_off_is_the_step_one_ai() {
    assert!(!sim_core::settings::planner_from(Some("0")));
    assert!(!sim_core::settings::planner_from(Some("false")));
    assert!(sim_core::settings::planner_from(None));
    assert!(sim_core::settings::planner_from(Some("1")));
    let build = || {
        let mut g = scion_goldfish();
        g.player_mut(P0).turns = 4;
        lands(&mut g, P0, "Underground Sea", 5, false);
        hand(&mut g, P0, &["Thassa's Oracle", "Demonic Tutor"]);
        in_library(&mut g, "Demonic Consultation");
        g
    };
    let mut off = build();
    set_planner(&mut off, false);
    assert!(!planner::applies(&off, P0));
    ai::brain::main(&mut off, P0, false).unwrap();
    assert!(!log_has(&off, "[plan:"), "no plan with the planner off");
    let mut on = build();
    assert!(planner::applies(&on, P0));
    ai::brain::main(&mut on, P0, false).unwrap();
    assert!(log_has(&on, "[plan:"));
}

/// Three real Tier 6 opponents, each with a full hand and blue mana open: the table can stop the line (`go_ok` says
/// wait), so the ready line is no plan and the main phase doesn't go for it
#[test]
fn with_a_counter_visible_and_go_ok_false_the_planner_does_not_go() {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "rograkh-silas-turbo-breach", "kinnan-bonder-mana-combo"]);
    g.log = Some(vec![]);
    for i in 1..4 {
        let q = PlayerId(i);
        let cs: Vec<_> = ["Force of Will", "Counterspell", "Swan Song", "Flusterstorm", "Mental Misstep"]
            .iter()
            .map(|n| take(&mut g, q, n))
            .collect();
        g.player_mut(q).hand = cs;
        lands(&mut g, q, "Island", 3, false);
    }
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 3;
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    assert!(!combos::plan_lines(&g, P0, false).is_empty(), "the line is ready and affordable");
    assert!(combos::plan_lines(&g, P0, true).is_empty(), "the table says wait");
    assert_eq!(planner::plan_this_turn(&mut g, P0).unwrap(), None);
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(!g.over, "no attempt into open counters");
    assert!(g.player(P0).hand.contains(&card(&g, "Thassa's Oracle")));
    assert!(!log_has(&g, "[plan: combo:"));
}

/// A card the deck's plan slot keeps in hand (`t6_plans::prio` 0) is never the planner's play: Dack Fayden waits while
/// three opponents hold counters with blue open, as the deck's plan says (casting it anyway made Dack win 41% of the
/// Tier 6 table in the first measurement)
#[test]
fn the_plan_slot_keeps_a_card_in_hand_against_the_planner() {
    let mut g = table(&[
        "dack-fayden-white-stax",
        "kraum-tymna-blue-breach-midrange",
        "rograkh-silas-turbo-breach",
        "kinnan-bonder-mana-combo",
    ]);
    for i in 1..4 {
        let q = PlayerId(i);
        let cs: Vec<_> = ["Force of Will", "Counterspell", "Swan Song", "Flusterstorm", "Mental Misstep"]
            .iter()
            .map(|n| take(&mut g, q, n))
            .collect();
        g.player_mut(q).hand = cs;
        lands(&mut g, q, "Island", 3, false);
    }
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 3;
    lands(&mut g, P0, "Plains", 6, false);
    let dack = g.player(P0).cmd;
    assert_eq!(sim_core::impls::t6_plans::prio(&g, P0, dack), Some(0.0), "the plan holds Dack");
    let d = planner::decide(&mut g, P0).unwrap();
    assert!(
        !matches!(&d, planner::Decision::Play { step: Step::Cast(c), .. } if *c == dack),
        "the planner casts Dack against the plan: {d:?}"
    );
}
