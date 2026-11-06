//! c1b-engines (`audit/realism/c1b-2026-11-06/REPORT.md`): Tier 6 decks hold their draw engines and wheels. One
//! scripted position per fix: the old policy offered the engine below "stop" (or at 0), the new one offers it above.
//! The main-phase menu (`brain::main_options`) is read, not sampled, so the checks are deterministic.

use sim_core::ai::{self, brain};
use sim_core::hooks::Opt;
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// seat 0 is `key`; the other three seats are inert goldfish (empty hand, a library of Wastes); p0 starts with an empty
/// hand and a library of Islands, on its own main phase, `turns` turns in
fn goldfish(key: &str, turns: u32) -> Game {
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
    g.player_mut(P0).turns = turns;
    g.round = turns;
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn util(opts: &[Opt], label: &str) -> f64 {
    opts.iter()
        .find(|o| o.label == label)
        .unwrap_or_else(|| panic!("{label} is not offered: {:?}", labels(opts)))
        .utility
}

fn stop(opts: &[Opt]) -> f64 {
    opts.iter().find(|o| o.act.is_none()).expect("stop is always offered").utility
}

fn labels(opts: &[Opt]) -> Vec<(String, f64)> {
    opts.iter().map(|o| (o.label.clone(), (o.utility * 10.0).round() / 10.0)).collect()
}

fn menu(g: &mut Game) -> Vec<Opt> {
    brain::main_options(g, P0, false).unwrap()
}

/// the engine is offered at least a margin above "stop"
fn beats_stop(opts: &[Opt], engine: &str) {
    let (e, s) = (util(opts, engine), stop(opts));
    assert!(
        e >= s + brain::ENGINE_MARGIN - 1e-9,
        "{engine} {e:.2} must beat stop {s:.2} by the margin: {:?}",
        labels(opts)
    );
}

/// Fix 1 (the main cause): Rhystic Study on turn 3 with Ad Nauseam and a counter in hand and five lands. Casting the
/// Study (3) leaves too little for Ad Nauseam (5), so the old reserve penalty (0.7 x the 7.0 of holding for it) took
/// the Study to 3.5 against "stop (hold mana)" at 7.0. Real pilots cast the Study on turn 3; the new rule does not
/// charge the reserve through turn 4 when no opponent is about to win.
#[test]
fn rhystic_study_is_cast_on_turn_three_with_ad_nauseam_held() {
    let mut g = goldfish("kinnan-bonder-mana-combo", 3);
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(&mut g, P0, &["Rhystic Study", "Ad Nauseam", "Swan Song"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 5.0, "mana is held for Ad Nauseam: {:?}", labels(&opts));
    beats_stop(&opts, "Rhystic Study");
}

/// Fix 1b: later, with the mana for both the engine and the held card, "stop" no longer outvotes the engine (the old
/// cast was offered at the plan's 55 against a stop worth 7.0).
#[test]
fn a_late_engine_is_not_outvoted_by_stop_while_the_held_card_stays_payable() {
    let mut g = goldfish("kinnan-bonder-mana-combo", 7);
    lands(&mut g, P0, "Underground Sea", 8, false);
    hand(&mut g, P0, &["Rhystic Study", "Ad Nauseam"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 5.0, "{:?}", labels(&opts));
    beats_stop(&opts, "Rhystic Study");
}

/// Fix 1c: Necropotence on turn 3 (Rograkh/Silas: hand_payable 269 by turn 4, cast 4%).
#[test]
fn necropotence_is_cast_on_turn_three_with_ad_nauseam_held() {
    let mut g = goldfish("rograkh-silas-turbo-breach", 3);
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(&mut g, P0, &["Necropotence", "Ad Nauseam", "Swan Song"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 5.0, "{:?}", labels(&opts));
    beats_stop(&opts, "Necropotence");
}

/// Fix 1d: Thrasios on curve in the Kinnan deck (hand_payable 279 by turn 4).
#[test]
fn thrasios_is_cast_on_curve_with_a_counter_held() {
    let mut g = goldfish("kinnan-bonder-mana-combo", 3);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Thrasios, Triton Hero", "Swan Song", "Flusterstorm"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 1.0, "{:?}", labels(&opts));
    beats_stop(&opts, "Thrasios, Triton Hero");
}

/// Fix 1e: Faerie Mastermind on curve (a flash creature: the old policy waited for the end of the turn, and held it
/// there as well: hand_payable 233 by turn 4).
#[test]
fn faerie_mastermind_is_cast_on_curve_with_a_counter_held() {
    let mut g = goldfish("kraum-tymna-blue-breach-midrange", 3);
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Faerie Mastermind", "Swan Song", "Flusterstorm"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 1.0, "{:?}", labels(&opts));
    beats_stop(&opts, "Faerie Mastermind");
}

/// Fix 1f: the rule is for Tier 6 draw engines only: a rock is not one, and another tier's deck is not touched.
#[test]
fn the_engine_rule_is_for_tier_six_engines_only() {
    let mut g = goldfish("kinnan-bonder-mana-combo", 2);
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Rhystic Study", "Fellwar Stone"]);
    let s = brain::Situation::new(&g, P0);
    let (rhystic, stone) = (card(&g, "Rhystic Study"), card(&g, "Fellwar Stone"));
    assert!(brain::engine_rule(&g, P0, rhystic, &s, false).is_some_and(|r| r.early));
    assert!(brain::engine_rule(&g, P0, rhystic, &s, true).is_none(), "an opponent about to win: the answer stays up");
    assert!(brain::engine_rule(&g, P0, stone, &s, false).is_none(), "a rock is not an engine");
    g.round = 7;
    g.player_mut(P0).turns = 7;
    let s = brain::Situation::new(&g, P0);
    assert!(brain::engine_rule(&g, P0, rhystic, &s, false).is_some_and(|r| !r.early), "late: no reserve waiver");
    let mut other = goldfish("yshtola", 2);
    hand(&mut other, P0, &["Rhystic Study"]);
    let s = brain::Situation::new(&other, P0);
    let c = card(&other, "Rhystic Study");
    assert!(brain::engine_rule(&other, P0, c, &s, false).is_none(), "Yshtola is not a Tier 6 deck");
}

/// Fix 2: mana is held for Ad Nauseam only with room in the hand to cast it (four cards under the cap). With a full
/// hand the old hold kept "stop" at 7.0 and the hand full (Rograkh/Silas: Ad Nauseam hand_payable 172, cast 1%).
#[test]
fn mana_is_not_held_for_an_ad_nauseam_that_cannot_be_cast() {
    let mut g = goldfish("kraum-tymna-blue-breach-midrange", 5);
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(&mut g, P0, &["Ad Nauseam"]);
    let s = brain::Situation::new(&g, P0);
    assert!(brain::hold_value(&g, P0, &s).0.is_some(), "room for the draw: the mana is held");
    hand(
        &mut g,
        P0,
        &[
            "Consecrated Sphinx",
            "Terror of the Peaks",
            "Atraxa, Praetors' Voice",
            "Elesh Norn, Grand Cenobite",
            "Karn, the Great Creator",
            "Thassa's Oracle",
        ],
    );
    let s = brain::Situation::new(&g, P0);
    assert_eq!(
        brain::hold_value(&g, P0, &s).0,
        None,
        "seven cards in hand: Ad Nauseam is not cast, so nothing is held"
    );
    // JD's Tier 5 and Tier 6 decks outside the pool are not changed by this: only Tier 6 keys read the room (the rule
    // is gated on `is_t6`; Sheoldred's own Ad Nauseam hold is unchanged, see the report)
}

/// Fix 3: a stuck hand wheels. Rograkh/Silas with five lands and a hand of expensive cards it cannot pay for casts
/// Wheel of Fortune, Windfall and Timetwister; with castable cards in hand, or a combo piece, it does not.
#[test]
fn a_stuck_hand_wheels() {
    for wheel in ["Wheel of Fortune", "Windfall", "Timetwister"] {
        let mut g = goldfish("rograkh-silas-turbo-breach", 4);
        lands(&mut g, P0, "Island", 3, false);
        lands(&mut g, P0, "Mountain", 2, false);
        hand(
            &mut g,
            P0,
            &[
                wheel,
                "Consecrated Sphinx",
                "Elesh Norn, Grand Cenobite",
                "Terror of the Peaks",
                "Atraxa, Praetors' Voice",
            ],
        );
        let w = card(&g, wheel);
        assert!(ai::decks::deck_prio_f(&g, P0, w) > 0.0, "{wheel}: nothing castable besides it, so it wheels");
        let opts = menu(&mut g);
        assert!(opts.iter().any(|o| o.label == wheel), "{wheel} is offered");
    }
}

#[test]
fn a_hand_with_plays_in_it_does_not_wheel() {
    for wheel in ["Wheel of Fortune", "Windfall", "Timetwister"] {
        let mut g = goldfish("rograkh-silas-turbo-breach", 4);
        lands(&mut g, P0, "Island", 3, false);
        lands(&mut g, P0, "Mountain", 2, false);
        hand(&mut g, P0, &[wheel, "Mystic Remora", "Sol Ring", "Consecrated Sphinx", "Elesh Norn, Grand Cenobite"]);
        let w = card(&g, wheel);
        assert_eq!(ai::decks::deck_prio_f(&g, P0, w), 0.0, "{wheel}: two castable cards in hand, no wheel");
    }
}

/// Fix 3b: a wheel that would leave no mana to use the seven new cards waits.
#[test]
fn a_wheel_with_no_mana_left_waits() {
    let mut g = goldfish("rograkh-silas-turbo-breach", 4);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 1, false);
    hand(
        &mut g,
        P0,
        &[
            "Wheel of Fortune",
            "Consecrated Sphinx",
            "Terror of the Peaks",
            "Atraxa, Praetors' Voice",
            "Elesh Norn, Grand Cenobite",
        ],
    );
    let w = card(&g, "Wheel of Fortune");
    assert_eq!(ai::decks::deck_prio_f(&g, P0, w), 0.0);
}

/// Fix 1g: an opposing commander that is a modeled combo piece (Kinnan on the table) made the old policy treat every
/// Tier 6 turn as "an opponent is close to winning" (`cmdr_combo`: reserve 8.0 on every cast); that alone no longer
/// keeps the engine in hand.
#[test]
fn an_opposing_combo_commander_alone_does_not_hold_the_engine() {
    let mut g = goldfish("rograkh-thrasios-creature-combo", 3);
    let kinnan = perm(&mut g, PlayerId(3), "Kinnan, Bonder Prodigy");
    g.perm_mut(kinnan).is_cmd = true;
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Rhystic Study", "Swan Song", "Flusterstorm"]);
    let opts = menu(&mut g);
    assert!(stop(&opts) > 1.0, "{:?}", labels(&opts));
    beats_stop(&opts, "Rhystic Study");
}
