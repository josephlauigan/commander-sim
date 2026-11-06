//! c1-t6-lines (`audit/realism/c1-2026-11-06/REPORT.md`): one scripted position per fix. In each the old code held or
//! failed and the new code acts.

use sim_core::ai;
use sim_core::ai::planner::{self, Step};
use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// seat 0 is `key`; the other three seats are inert goldfish (empty hand, a library of Wastes), p0's hand is empty and
/// its library is Islands
fn goldfish(key: &str) -> Game {
    let mut g = table(&[key, "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
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

fn in_library(g: &mut Game, name: &str) {
    let c = card(g, name);
    g.player_mut(P0).library.insert(0, c);
}

fn line(part: &str) -> usize {
    combos::combo_index(part).unwrap()
}

/// Fix 1: Diabolic Intent's only fodder is the commander. The old AI would not sacrifice a commander, so the tutor
/// was held for rounds (Kraum/Tymna: "Diabolic Intent 249" tutors held). The planner may eat it when the tutor
/// finishes the line this turn.
#[test]
fn a_sacrifice_tutor_eats_the_commander_when_it_wins_the_turn() {
    let mut g = goldfish("kraum-tymna-blue-breach-midrange");
    g.player_mut(P0).turns = 4;
    lands(&mut g, P0, "Underground Sea", 5, false);
    let tymna = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(tymna).is_cmd = true;
    hand(&mut g, P0, &["Demonic Consultation", "Diabolic Intent"]);
    in_library(&mut g, "Thassa's Oracle");
    let (di, oracle) = (card(&g, "Diabolic Intent"), card(&g, "Thassa's Oracle"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert_eq!(plan, Some(vec![Step::Tutor(di, oracle), Step::Go(line("Thassa's Oracle + Demonic Consultation"))]));
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(g.over && g.winner == Some(P0), "Diabolic Intent eats Tymna, finds the Oracle, the line wins");
}

/// Fix 1b: no creature on the battlefield: the planner casts the cheapest creature first as fodder.
#[test]
fn a_creature_is_cast_to_feed_a_sacrifice_tutor() {
    let mut g = goldfish("kraum-tymna-blue-breach-midrange");
    g.player_mut(P0).turns = 4;
    lands(&mut g, P0, "Underground Sea", 4, false);
    lands(&mut g, P0, "Badlands", 2, false);
    hand(&mut g, P0, &["Demonic Consultation", "Diabolic Intent", "Ragavan, Nimble Pilferer"]);
    in_library(&mut g, "Thassa's Oracle");
    let (di, oracle, rag) =
        (card(&g, "Diabolic Intent"), card(&g, "Thassa's Oracle"), card(&g, "Ragavan, Nimble Pilferer"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap();
    assert_eq!(
        plan,
        Some(vec![Step::Cast(rag), Step::Tutor(di, oracle), Step::Go(line("Thassa's Oracle + Demonic Consultation"))])
    );
}

/// Fix 2: Transmute Artifact is a tutor for the planner: sacrifice a cheap artifact, take Basalt Monolith onto the
/// battlefield, and Kinnan + Basalt Monolith (with Urza as the mana sink) is ready.
#[test]
fn transmute_artifact_fetches_the_monolith_for_kinnan() {
    let mut g = goldfish("kinnan-bonder-mana-combo");
    g.round = 6; // the plan file holds Kinnan's loop before round 4 without cover
    g.player_mut(P0).turns = 5;
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P0, "Forest", 4, false);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Urza, Lord High Artificer");
    hand(&mut g, P0, &["Transmute Artifact"]);
    in_library(&mut g, "Basalt Monolith");
    let (ta, mono) = (card(&g, "Transmute Artifact"), card(&g, "Basalt Monolith"));
    let plan = planner::plan_this_turn(&mut g, P0).unwrap().expect("a plan");
    assert_eq!(plan[0], Step::Tutor(ta, mono), "{plan:?}");
}

/// Fix 3: a Sisay line's fetch honours Cursed Totem (creatures' activated abilities). It was called directly by the
/// combo step and never asked.
#[test]
fn sisay_cannot_fetch_under_cursed_totem() {
    let mut g = goldfish("sisay-legends-toolbox");
    lands(&mut g, P0, "Command Tower", 5, false);
    perm(&mut g, P0, "Sisay, Weatherlight Captain");
    in_library(&mut g, "Gaea's Cradle");
    let cradle = card(&g, "Gaea's Cradle");
    perm(&mut g, PlayerId(1), "Cursed Totem");
    assert!(!sim_core::impls::legends::sisay_fetch_card(&mut g, P0, cradle).unwrap(), "Totem stops the ability");
    assert!(g.player(P0).library.contains(&cradle));
}

/// Fix 3b: the same fetch, no lock: it works (the check does not break the line).
#[test]
fn sisay_fetches_without_a_lock() {
    let mut g = goldfish("sisay-legends-toolbox");
    lands(&mut g, P0, "Command Tower", 5, false);
    perm(&mut g, P0, "Sisay, Weatherlight Captain");
    in_library(&mut g, "Gaea's Cradle");
    let cradle = card(&g, "Gaea's Cradle");
    assert!(sim_core::impls::legends::sisay_fetch_card(&mut g, P0, cradle).unwrap());
    assert!(!g.player(P0).library.contains(&cradle));
}

/// Fix 4: Kinnan + Basalt Monolith is not ready under Collector Ouphe (Monolith's untap is an artifact ability).
#[test]
fn kinnan_and_monolith_are_stopped_by_collector_ouphe() {
    let mut g = goldfish("kinnan-bonder-mana-combo");
    g.player_mut(P0).turns = 5;
    lands(&mut g, P0, "Forest", 4, false);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Urza, Lord High Artificer");
    let i = line("Kinnan + Basalt Monolith");
    assert!((combos::all()[i].ready)(&g, P0).0, "ready with nothing against it");
    perm(&mut g, PlayerId(1), "Collector Ouphe");
    assert!(!(combos::all()[i].ready)(&g, P0).0, "Collector Ouphe stops the Monolith's untap");
}
