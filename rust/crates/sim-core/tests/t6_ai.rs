//! t6-ai (rust/agents/t6-ai.md): Tier 6 planner and hold fixes, one scripted test per fix.
//!
//! 1. Finale of Devastation / Green Sun's Zenith for Eternal Witness (Rograkh/Thrasios): the deck's plan file holds
//!    both as finishers (prio 0), and the planner skipped any card the plan slot holds, so it never cast them as tutors.
//! 2. Whir of Invention for an artifact piece (Kinnan): an X tutor for artifacts, with improvise counted in its cost.
//! 4. "An opponent is close to winning": a Tier 6 deck reads the archenemy alone, not any combo-piece commander.
//! 5. Ad Nauseam's hold for Sheoldred needs room for four cards in hand, as for Tier 6.

use sim_core::ai::planner::{self, Step};
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step as TurnStep};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const RT: &str = "rograkh-thrasios-creature-combo";
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const KINNAN: &str = "kinnan-bonder-mana-combo";

fn log_has(g: &Game, s: &str) -> bool {
    g.log.as_ref().is_some_and(|l| l.iter().any(|x| x.contains(s)))
}

// ------------------------------------------------------------------ 1. Finale / GSZ for Eternal Witness
/// Rograkh/Thrasios, turn 6: Gaea's Cradle and five creatures out, Snap and the tutor in hand, eight lands, Eternal
/// Witness in the library (the position of seed 900000 game 139, R6-R7, with the fifth creature the line needs)
fn witness_table(tutor: &str) -> Game {
    let mut g = table(&[RT, KRAUM, "dack-fayden-white-stax", "rograkh-silas-turbo-breach"]);
    g.active = Some(P0);
    g.step = TurnStep::Main1;
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 6;
    g.round = 6;
    for i in 1..4 {
        g.player_mut(PlayerId(i)).hand.clear();
    }
    g.log = Some(vec![]);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lands(&mut g, P0, "Forest", 4, false);
    lands(&mut g, P0, "Island", 3, false);
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    hand(&mut g, P0, &[tutor, "Snap"]);
    assert!(g.player(P0).library.contains(&card(&g, "Eternal Witness")));
    g
}

fn plan_for_witness(tutor: &str) {
    let mut g = witness_table(tutor);
    let (t, w) = (card(&g, tutor), card(&g, "Eternal Witness"));
    let plan =
        planner::plan_this_turn(&mut g, P0).unwrap().unwrap_or_else(|| panic!("{tutor}: no plan for Eternal Witness"));
    assert_eq!(plan.len(), 2, "{tutor}: {plan:?}");
    assert_eq!(plan[0], Step::Tutor(t, w), "{tutor}: {plan:?}");
    assert!(matches!(plan[1], Step::Go(_)));
    assert_eq!(planner::tutor_x(&g, plan[0]), 3, "X is Witness's mana value");
}

#[test]
fn finale_of_devastation_is_cast_for_eternal_witness() {
    plan_for_witness("Finale of Devastation");
}

#[test]
fn green_suns_zenith_finds_eternal_witness_too() {
    // control: the line itself casts GSZ (`x_tutor`), so the plan is the line alone
    let mut g = witness_table("Green Sun's Zenith");
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().expect("a plan");
    assert!(matches!(plan.as_slice(), [Step::Go(_)]), "{plan:?}");
}

/// the main phase makes the play, the line runs, and the loop's last Witness returns the Finale it was found with
#[test]
fn the_main_phase_goes_for_witness_with_finale_and_gets_finale_back() {
    let mut g = witness_table("Finale of Devastation");
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(log_has(&g, "[plan: Finale of Devastation for Eternal Witness (X=3)"), "{:?}", g.log);
    assert!(log_has(&g, "Eternal Witness + Snap + Gaea's Cradle"), "the line ran: {:?}", g.log);
}

// ------------------------------------------------------------------ 2. Whir of Invention
const KINNAN_SEATS: [&str; 4] = [KINNAN, KRAUM, "dack-fayden-white-stax", "rograkh-silas-turbo-breach"];

/// Kinnan and Thrasios out, Whir of Invention in hand, Basalt Monolith in the library, `lands` Islands and Forests
fn whir_table(islands: usize, forests: usize, artifacts: usize) -> Game {
    let mut g = table(&KINNAN_SEATS);
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
    g.active = Some(P0);
    g.step = TurnStep::Main1;
    g.log = Some(vec![]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Thrasios, Triton Hero");
    token(&mut g, P0, 1); // the third attacker Kinnan + Monolith + Thrasios needs (Finale pumps one per opponent)
    lands(&mut g, P0, "Island", islands, false);
    lands(&mut g, P0, "Forest", forests, false);
    for _ in 0..artifacts {
        perm(&mut g, P0, "Mirage Mirror"); // an untapped artifact that is no mana rock: Whir's improvise taps it
    }
    hand(&mut g, P0, &["Whir of Invention"]);
    assert!(g.player(P0).library.contains(&card(&g, "Basalt Monolith")));
    g
}

/// Whir ({X}{U}{U}{U}) for Basalt Monolith (X = 3) needs six mana, and {G}{G} stays for the line (Kinnan + Monolith +
/// Thrasios, then Finale)
#[test]
fn whir_of_invention_is_cast_for_basalt_monolith() {
    let mut g = whir_table(3, 5, 0);
    let (whir, basalt) = (card(&g, "Whir of Invention"), card(&g, "Basalt Monolith"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().expect("a plan with Whir");
    assert_eq!(plan.len(), 2, "{plan:?}");
    assert_eq!(plan[0], Step::Tutor(whir, basalt), "{plan:?}");
    assert!(matches!(plan[1], Step::Go(_)));
    assert_eq!((planner::tutor_x(&g, plan[0]), planner::pay_x(&g, P0, plan[0])), (3, 3));
    // the real cast puts the planned artifact onto the battlefield
    assert!(planner::perform(&mut g, P0, plan[0]).unwrap());
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(basalt)), "Basalt Monolith is out");
}

/// improvise: two untapped artifacts pay for two of the three, so four lands (UUU + 1) pay for Whir, not six; two Forests stay for the line
#[test]
fn whir_counts_the_artifacts_it_taps_to_improvise() {
    let mut poor = whir_table(3, 0, 0);
    assert_eq!(planner::plan_this_turn(&mut poor, P0).unwrap(), None, "three Islands pay for UUU only");
    let mut g = whir_table(3, 3, 2);
    let (whir, basalt) = (card(&g, "Whir of Invention"), card(&g, "Basalt Monolith"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().expect("a plan with Whir");
    assert_eq!(plan[0], Step::Tutor(whir, basalt), "{plan:?}");
    assert_eq!((planner::tutor_x(&g, plan[0]), planner::pay_x(&g, P0, plan[0])), (3, 1), "two artifacts improvise");
    assert!(planner::perform(&mut g, P0, plan[0]).unwrap());
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(basalt)), "Basalt Monolith is out");
    let tapped = g
        .player(P0)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Mirage Mirror") && g.perm(m).tapped)
        .count();
    assert_eq!(tapped, 2, "both artifacts were tapped to improvise");
}

// ------------------------------------------------------------------ 4. "an opponent is close to winning"
use sim_core::ai::brain::{self, Situation};

/// a held counter and the mana for it; P1 plays Kinnan, whose commander (a catalogued combo piece) is on the table.
/// The hold value with the commander flagged as a commander minus the same board with it as an ordinary permanent
/// isolates `cmdr_combo`.
fn hold_gap(seat: &str) -> f64 {
    let mut g = table(&[seat, KINNAN, "dack-fayden-white-stax", "rograkh-silas-turbo-breach"]);
    g.active = Some(P0);
    g.step = TurnStep::Main1;
    g.player_mut(P0).turns = 5;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Counterspell"]);
    let k = perm(&mut g, PlayerId(1), "Kinnan, Bonder Prodigy");
    g.perm_mut(k).is_cmd = true;
    let mut s = Situation::new(&g, P0);
    assert!(s.cmdr_combo, "Kinnan is a commander on the battlefield and a combo piece");
    assert!(!s.combo_near, "no archenemy is close");
    let mut hv = |cmdr: bool| {
        s.cmdr_combo = cmdr;
        let (held, v) = brain::hold_value(&g, P0, &s);
        assert!(held.is_some(), "{seat}: the counter is held");
        v
    };
    let with = hv(true);
    with - hv(false)
}

/// a Tier 6 deck is not told an opponent is close to winning by a combo-piece commander alone
#[test]
fn a_tier_6_deck_ignores_a_combo_piece_commander_for_its_hold() {
    assert_eq!(hold_gap(KRAUM), 0.0, "Kraum/Tymna");
    assert_eq!(hold_gap("thrasios"), 0.0, "JD's Tymna/Thrasios (a Tier 6 key)");
}

/// ... and every other deck still is (up to +2.5 on the hold value: a deck outside the main set has a floor under it)
#[test]
fn a_deck_outside_tier_6_still_reads_the_commander() {
    let gap = hold_gap("sheoldred");
    assert!(gap > 0.0, "JD's Sheoldred still reads the commander: {gap}");
}

// ------------------------------------------------------------------ 5. Ad Nauseam's hold for Sheoldred
fn adnaus_hold(seat: &str, hand_size: usize) -> f64 {
    let mut g = table(&[seat, KINNAN, "dack-fayden-white-stax", "rograkh-silas-turbo-breach"]);
    g.active = Some(PlayerId(1)); // the end of the turn before ours
    g.step = TurnStep::End;
    g.player_mut(P0).turns = 5;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Ad Nauseam"]);
    let filler: Vec<&str> = vec!["Island"; hand_size - 1];
    hand(&mut g, P0, &filler);
    let s = Situation::new(&g, P0);
    brain::hold_value(&g, P0, &s).1
}

/// with a full hand Ad Nauseam is not cast, so no mana is held for it: Sheoldred as Tier 6
#[test]
fn sheoldred_holds_no_mana_for_ad_nauseam_with_a_full_hand() {
    assert!(adnaus_hold("sheoldred", 3) > 0.0, "room for four: the mana is held");
    assert_eq!(adnaus_hold("sheoldred", 10), 0.0, "a full hand: nothing is held");
}

/// Trophy Mage (a creature whose enter trigger fetches an artifact with mana value 3) is a planner tutor for the Monolith:
/// the planner casts it, then casts the Monolith it found, then goes (Kinnan + Monolith + Thrasios)
#[test]
fn trophy_mage_is_cast_for_basalt_monolith() {
    let mut g = whir_table(1, 7, 0); // {2}{U} for the Mage, {3} for the Monolith, {G}{G} left for the line
    hand(&mut g, P0, &["Trophy Mage"]);
    let whir = card(&g, "Whir of Invention");
    g.player_mut(P0).hand.retain(|&c| c != whir);
    let (mage, basalt) = (card(&g, "Trophy Mage"), card(&g, "Basalt Monolith"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().expect("a plan with Trophy Mage");
    assert_eq!(plan.len(), 3, "{plan:?}");
    assert_eq!(plan[0], Step::Tutor(mage, basalt), "{plan:?}");
    assert_eq!(plan[1], Step::Cast(basalt), "{plan:?}");
    assert!(matches!(plan[2], Step::Go(_)));
}
