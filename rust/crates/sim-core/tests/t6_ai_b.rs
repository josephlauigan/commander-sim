//! t6-ai-b (rust/agents/t6-ai-b.md): Tier 6 AI follow-ups, one scripted test per fix.
//!
//! 1. Lion's Eye Diamond: cast early only with a card it fuels in hand (plan files), cracked as a wheel is cast, never
//!    into an empty plan, and not counted as mana by the held statistic when no use is open.
//! 2. Instant removal held for an opposing combo-piece commander: a Tier 6 deck reads the archenemy alone.
//! 3. Urza's Saga, Tezzeret and Mana Vault (see the report: what was found).

use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step as TurnStep};
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const SILAS: &str = "rograkh-silas-turbo-breach";
const DACK: &str = "dack-fayden-white-stax";
const KINNAN: &str = "kinnan-bonder-mana-combo";

fn log_has(g: &Game, s: &str) -> bool {
    g.log.as_ref().is_some_and(|l| l.iter().any(|x| x.contains(s)))
}

/// `seat` at P0 on its own turn with an empty hand; the three other seats are goldfish with no cards in hand
fn solo(seat: &str, turn: u32) -> Game {
    let mut g = table(&[seat, KINNAN, DACK, SILAS]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = turn;
    g.round = turn;
    g.active = Some(P0);
    g.step = TurnStep::Main1;
    g.log = Some(vec![]);
    g
}

fn prio(g: &Game, name: &str) -> Option<f64> {
    sim_core::impls::t6_plans::prio(g, P0, card(g, name))
}

// ------------------------------------------------------------------ 1. Lion's Eye Diamond
use sim_core::hooks::Opt;

/// the AI's options for P0 now (the choice among them is sampled, so the tests read the options and make a play)
fn options(g: &mut Game) -> Vec<Opt> {
    sim_core::ai::brain::main_options(g, P0, false).unwrap()
}

fn find_opt<'a>(o: &'a [Opt], label: &str) -> Option<&'a Opt> {
    o.iter().find(|x| x.label == label)
}

/// the plan files: LED is cast (value 90) with a card it fuels in hand, held (0) without
#[test]
fn led_has_a_cast_priority_only_with_a_card_it_fuels() {
    for (seat, fuel) in [
        (KRAUM, vec!["Underworld Breach"]),
        (SILAS, vec!["Underworld Breach", "Wheel of Fortune", "Windfall", "Timetwister"]),
    ] {
        let mut g = solo(seat, 1);
        hand(&mut g, P0, &["Lion's Eye Diamond", "Mystic Remora"]);
        assert_eq!(prio(&g, "Lion's Eye Diamond"), Some(0.0), "{seat}: nothing to fuel: held");
        for f in fuel {
            let mut g = solo(seat, 1);
            hand(&mut g, P0, &["Lion's Eye Diamond", f]);
            assert_eq!(prio(&g, "Lion's Eye Diamond"), Some(90.0), "{seat}: {f} in hand");
        }
    }
}

/// the main phase on turn 1 offers the cast with a wheel in hand, and keeps it in hand without one
#[test]
fn led_is_offered_on_turn_one_with_a_wheel_and_not_without() {
    let mut g = solo(SILAS, 1);
    hand(&mut g, P0, &["Lion's Eye Diamond", "Windfall", "Mystic Remora", "Rhystic Study", "Gamble"]);
    let o = options(&mut g);
    let led = find_opt(&o, "Lion's Eye Diamond").expect("LED is offered with Windfall in hand");
    assert!(led.utility > 0.0, "{}", led.utility);

    let mut g = solo(SILAS, 1);
    hand(&mut g, P0, &["Lion's Eye Diamond", "Mystic Remora", "Rhystic Study", "Gamble"]);
    let o = options(&mut g);
    assert!(find_opt(&o, "Lion's Eye Diamond").is_none(), "no card to fuel: it waits in hand");
}

/// the option "cast wheel with the diamond" when the table is `lands` Islands, an opponent holding `opp` cards, the
/// library of lands only (nothing else is cast afterwards); returns the game and the option if there is one
fn wheel_table(wheel: &str, lands_n: usize, opp: usize, rest: &[&str]) -> (Game, Option<Opt>) {
    let mut g = solo(SILAS, 3);
    let w2 = card(&g, "Wastes");
    g.player_mut(PlayerId(1)).hand = vec![w2; opp];
    g.player_mut(P0).library = (0..60).map(|_| card(&g, "Wastes")).collect();
    perm(&mut g, P0, "Lion's Eye Diamond");
    lands(&mut g, P0, "Island", lands_n, false);
    let mut h = vec![wheel];
    h.extend(rest);
    hand(&mut g, P0, &h);
    let o = options(&mut g);
    let opt = o.into_iter().find(|x| x.label == format!("{wheel} (Lion's Eye Diamond)"));
    (g, opt)
}

/// LED on the table, a wheel and one other card in hand, no mana at all, an opponent holding two cards: the crack pays
/// for the wheel ({2}{U} / {2}{R}; the other card goes with the hand the wheel would discard) and the new cards arrive
/// (seven; Windfall draws as many as the largest hand discarded to it: the opponent's two)
#[test]
fn the_diamond_is_cracked_to_cast_a_wheel_from_no_mana() {
    for (wheel, mana, drawn) in [("Windfall", "UUU", 2), ("Wheel of Fortune", "RRR", 7), ("Timetwister", "UUU", 7)] {
        let (mut g, opt) = wheel_table(wheel, 0, 2, &["Mystic Remora"]);
        let opt = opt.unwrap_or_else(|| panic!("{wheel}: the crack is offered"));
        assert!(sim_core::ai::act::perform(&mut g, P0, opt.act.as_ref().unwrap()).unwrap());
        assert!(log_has(&g, &format!("cracks Lion's Eye Diamond for {mana} to cast {wheel}")), "{wheel}: {:?}", g.log);
        if wheel != "Timetwister" {
            // (Timetwister shuffles the graveyard into the library)
            let gy: Vec<String> = g.player(P0).gy.iter().map(|&c| g.db.get(c).name.to_string()).collect();
            assert!(gy.contains(&"Lion's Eye Diamond".to_string()), "{wheel}: the diamond was sacrificed: {gy:?}");
            assert!(gy.contains(&"Mystic Remora".to_string()), "{wheel}: the rest of the hand was discarded: {gy:?}");
        }
        assert_eq!(g.player(P0).hand.len(), drawn, "{wheel}: new cards");
    }
}

/// with the lands to pay for the wheel, the crack still pays for it (the option beats the plain cast) and leaves the
/// mana for the new cards
#[test]
fn the_crack_leaves_the_lands_untapped() {
    let (mut g, opt) = wheel_table("Timetwister", 3, 0, &["Mystic Remora"]);
    let opt = opt.expect("the crack is offered");
    let o = options(&mut g);
    let plain = find_opt(&o, "Timetwister").expect("the plain cast is offered too");
    assert!(opt.utility > plain.utility, "{} vs {}", opt.utility, plain.utility);
    assert!(sim_core::ai::act::perform(&mut g, P0, opt.act.as_ref().unwrap()).unwrap());
    assert!(log_has(&g, "cracks Lion's Eye Diamond for UUU to cast Timetwister"), "{:?}", g.log);
    // the three mana are still there this phase, in the pool or as untapped lands (the payment may take either)
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 3);
    assert_eq!(g.player(P0).hand.len(), 7);
}

/// never into an empty plan: a hand the deck would not wheel away (a combo piece in it), a Windfall that would draw
/// nothing (the diamond's discard does not count for it), no wheel at all: no crack is offered
#[test]
fn the_diamond_is_not_cracked_into_an_empty_plan() {
    // a combo piece in hand: wheel_prio is 0, so the wheel has no utility
    let (_, opt) = wheel_table("Timetwister", 0, 0, &["Thassa's Oracle"]);
    assert!(opt.is_none(), "a combo piece would be wheeled away");
    let (_, opt) = wheel_table("Timetwister", 0, 0, &["Mystic Remora", "Rhystic Study", "Gamble", "Imperial Seal"]);
    assert!(opt.is_none(), "a full hand is not wheeled");
    // Windfall with the opponents' hands empty draws nothing after the crack
    let (_, opt) = wheel_table("Windfall", 0, 0, &["Mystic Remora"]);
    assert!(opt.is_none(), "Windfall would draw nothing");
    // no wheel and no use: spells in hand, the diamond stays
    let mut g = solo(SILAS, 2);
    perm(&mut g, P0, "Lion's Eye Diamond");
    hand(&mut g, P0, &["Mystic Remora", "Rhystic Study", "Gamble"]);
    let o = options(&mut g);
    assert!(
        !o.iter().any(|x| x.label.contains("Lion's Eye Diamond")),
        "{:?}",
        o.iter().map(|x| &x.label).collect::<Vec<_>>()
    );
}

/// the held statistic counts the diamond only when a crack has a use
#[test]
fn the_held_statistic_counts_the_diamond_only_with_a_use() {
    let held = |cards: &[&str]| {
        let mut g = solo(SILAS, 1);
        hand(&mut g, P0, cards);
        sim_core::t6stats::mana_held(&mut g, P0);
        *g.player(P0).stats.get(&intern("mn|heldr|T1|Lion's Eye Diamond")).unwrap_or(&0)
    };
    assert_eq!(held(&["Lion's Eye Diamond", "Mystic Remora", "Gamble"]), 0, "no use: not mana");
    assert_eq!(held(&["Lion's Eye Diamond", "Timetwister", "Gamble"]), 1, "a wheel it pays for");
    assert_eq!(held(&["Lion's Eye Diamond", "Island", "Island"]), 0, "lands only, but no play for the mana");
}

// ------------------------------------------------------------------ 2. instant removal held for a combo commander
/// `seat` holds an instant removal spell with the mana for it while Kinnan sits on P1's battlefield as a commander
fn removal_held(seat: &str, removal: &str, land: &str) -> bool {
    let mut g = table(&[seat, KINNAN, DACK, SILAS]);
    g.active = Some(P0);
    g.step = TurnStep::Main1;
    g.player_mut(P0).turns = 5;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, land, 4, false);
    hand(&mut g, P0, &[removal]);
    let k = perm(&mut g, PlayerId(1), "Kinnan, Bonder Prodigy");
    g.perm_mut(k).is_cmd = true;
    let s = sim_core::ai::brain::Situation::new(&g, P0);
    assert!(s.cmdr_combo && !s.combo_near, "Kinnan is a combo-piece commander and nobody is close");
    sim_core::ai::brain::hold_value(&g, P0, &s).0.is_some()
}

#[test]
fn a_tier_6_deck_does_not_hold_instant_removal_for_a_combo_commander_alone() {
    assert!(!removal_held(DACK, "Swords to Plowshares", "Plains"), "Dack (Tier 6)");
    assert!(!removal_held("thrasios", "Swords to Plowshares", "Plains"), "JD's Tymna/Thrasios (a Tier 6 key)");
}

#[test]
fn a_deck_outside_tier_6_still_holds_instant_removal_for_it() {
    assert!(removal_held("sheoldred", "Go for the Throat", "Swamp"), "JD's Sheoldred holds it as before");
}

// ------------------------------------------------------------------ 3. Urza's Saga and Tezzeret as tutors
/// The planner (`tutor_rule`) looks for a tutor *spell* for a missing piece of a line. Urza's Saga (chapter III, after
/// two more turns) and Tezzeret, Cruel Captain (-3) fetch an artifact with mana value 0 or 1. The Tier 6 lists that play
/// them are Dack's only; this pins down that none of Dack's catalogued lines has such an artifact as a piece group
/// member that the deck needs fetched, i.e. there is nothing for the planner to plan with them.
#[test]
fn dack_has_no_line_piece_that_the_saga_or_tezzeret_could_fetch() {
    let g = table(&[DACK, KRAUM, KINNAN, SILAS]);
    let lines = sim_core::impls::combos::audit_lines(&g, P0);
    assert!(!lines.is_empty(), "Dack has catalogued lines");
    let mut cheap_artifact_pieces = vec![];
    for i in lines {
        for grp in sim_core::impls::combos::audit_groups(i) {
            for n in grp.iter() {
                let Some(id) = g.db.id(n) else { continue };
                let d = g.db.get(id);
                if d.types.has(sim_core::cards::Types::ARTIFACT) && !d.land && d.cmc <= 1 {
                    cheap_artifact_pieces.push(format!("{}: {n}", sim_core::impls::combos::audit_name(i)));
                }
            }
        }
    }
    println!("cheap artifact pieces in Dack's lines: {cheap_artifact_pieces:?}");
    assert!(cheap_artifact_pieces.is_empty(), "{cheap_artifact_pieces:?}");
}
