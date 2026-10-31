//! --- phase4-rules: the three generic rule checks (`ai/rules.rs`, `audit/phase4/rules-design.md`) on hand-built
//! positions, with the switch forced on this thread; each guard also keeps today's choice with the switch off. The
//! switch is on by default since the recalibration's stage 1, so every test's baseline forces it off (`AI_RULES=0`,
//! the old play) on its thread: `game` sets it and `with` puts it back.

use sim_core::ai::rules::{self, ALL, Guard};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Tier 5 pool decks (no omnath, no godo)
const T5: [&str; 4] =
    ["urza-mono-blue-artifacts", "kinnan-simic-mana-combo", "zur-esper-enchantment-control", "miirym-temur-dragons"];

/// the table, with the switch forced off on this thread (the test's baseline)
fn game(keys: &[&str]) -> Game {
    rules::force(Some(0));
    let mut g = table(keys);
    g.round = 4;
    for pl in &mut g.players {
        pl.turns = 4;
    }
    g
}

/// runs f with these guards forced on, then puts the switch back off
fn with<T>(mask: u8, f: impl FnOnce() -> T) -> T {
    rules::force(Some(mask));
    let r = f();
    rules::force(Some(0));
    r
}

fn cast_opt(c: CardId) -> Opt {
    Opt { utility: 5.0, label: "x".into(), act: Some(Action::Cast { card: c, zone: None }) }
}

// ------------------------------------------------------------------ the switch
#[test]
fn the_switch_reads_names_and_defaults_on() {
    assert_eq!(rules::parse_mode(None), ALL, "unset: all three (the stage 1 default)");
    assert_eq!(rules::parse_mode(Some("")), ALL);
    assert_eq!(rules::parse_mode(Some("0")), 0, "0: none, the old play");
    assert_eq!(rules::parse_mode(Some("1")), ALL);
    assert_eq!(rules::parse_mode(Some("followup")), Guard::Followup as u8);
    assert_eq!(rules::parse_mode(Some("tutored, combo")), Guard::Tutored as u8 | Guard::Combo as u8);
    let g = game(&T5);
    let all_off = |g: &Game| {
        !rules::on(g, P0, Guard::Followup) && !rules::on(g, P0, Guard::Tutored) && !rules::on(g, P0, Guard::Combo)
    };
    assert!(all_off(&g), "forced off");
    // no override on the thread: with AI_RULES unset in the test run, every guard is on
    rules::force(None);
    if std::env::var("AI_RULES").is_err() {
        assert!(
            rules::on(&g, P0, Guard::Followup) && rules::on(&g, P0, Guard::Tutored) && rules::on(&g, P0, Guard::Combo)
        );
    }
    with(Guard::Combo as u8, || {
        assert!(rules::on(&g, P0, Guard::Combo));
        assert!(!rules::on(&g, P0, Guard::Followup));
    });
    assert!(all_off(&g), "with puts the switch back off");
}

#[test]
#[should_panic(expected = "bad AI_RULES guard")]
fn an_unknown_guard_name_is_refused() {
    rules::parse_mode(Some("followup,pacts"));
}

// ------------------------------------------------------------------ follow-up payments
/// Pact of Negation: "Counter target spell. At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you
/// lose the game." With two Islands p can't pay it next upkeep: the guard stops the counter; today's pick (outside
/// Tier 6 the five-land test fails and the {0} cost falls through) casts it.
#[test]
fn pact_of_negation_is_not_cast_without_its_upkeep_mana() {
    let mut g = game(&T5);
    let pact = hand(&mut g, P1, &["Pact of Negation"])[0];
    lands(&mut g, P1, "Island", 2, false);
    let spell = card(&g, "Sol Ring");
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, spell), Some(pact), "today: the Pact is cast");
    with(ALL, || {
        assert!(!rules::followup_ok(&g, P1, pact));
        assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, spell), None);
    });
    // five Islands, tapped now: they untap for the upkeep
    lands(&mut g, P1, "Island", 3, true);
    with(ALL, || {
        assert!(rules::followup_ok(&g, P1, pact));
        assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, spell), Some(pact));
    });
    // a Pact already owed: {3}{U}{U} twice needs ten mana
    g.player_mut(P1).pacts = 1;
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P1, spell), Some(pact), "today: five lands are enough");
    with(ALL, || assert!(!rules::followup_ok(&g, P1, pact)));
}

/// Slaughter Pact's debt ({2}{B}) counts with a Pact of Negation owed, and the reverse
#[test]
fn every_upkeep_debt_is_counted_together() {
    let mut g = game(&T5);
    let pact = take(&mut g, P1, "Pact of Negation");
    lands(&mut g, P1, "Island", 5, false);
    with(ALL, || assert!(rules::followup_ok(&g, P1, pact)));
    g.player_mut(P1).pact_debts.push((2, "B".into()));
    with(ALL, || assert!(!rules::followup_ok(&g, P1, pact)));
    assert!(!rules::next_upkeep_affordable(&g, P1, (0, "")), "the Slaughter Pact debt alone needs {{B}}");
}

/// Diabolic Intent ("As an additional cost to cast this spell, sacrifice a creature.") with no creature, Culling the
/// Weak (compiled: "sacrifice a creature") the same: neither is offered. The filter keeps them with the switch off.
#[test]
fn a_sacrifice_cost_spell_needs_something_to_sacrifice() {
    let mut g = game(&T5);
    let intent = hand(&mut g, P0, &["Diabolic Intent"])[0];
    let culling = hand(&mut g, P0, &["Culling the Weak"])[0];
    lands(&mut g, P0, "Swamp", 3, false);
    with(ALL, || {
        assert!(!rules::followup_ok(&g, P0, intent));
        let mut opts = vec![cast_opt(intent), cast_opt(culling), Opt { utility: 0.0, label: "stop".into(), act: None }];
        rules::filter_options(&mut g, P0, &mut opts).unwrap();
        assert_eq!(opts.len(), 1, "only stop is left");
    });
    let mut opts = vec![cast_opt(intent), cast_opt(culling)];
    rules::filter_options(&mut g, P0, &mut opts).unwrap();
    assert_eq!(opts.len(), 2, "switch off: the filter changes nothing");
    token(&mut g, P0, 1);
    with(ALL, || {
        assert!(rules::followup_ok(&g, P0, intent));
        let mut opts = vec![cast_opt(intent), cast_opt(culling)];
        rules::filter_options(&mut g, P0, &mut opts).unwrap();
        assert_eq!(opts.len(), 2);
    });
}

/// Big Score: "As an additional cost to cast this spell, discard a card." Alone in hand it has nothing to discard:
/// the menu offers it today, not with the guard on.
#[test]
fn a_discard_cost_spell_needs_another_card() {
    let mut g = game(&T5);
    g.player_mut(P0).hand.clear();
    let big = hand(&mut g, P0, &["Big Score"])[0];
    lands(&mut g, P0, "Mountain", 5, false);
    let offered = |g: &mut Game| {
        let opts = sim_core::ai::brain::main_options(g, P0, false).unwrap();
        opts.iter().any(|o| matches!(o.act, Some(Action::Cast { card, .. }) if card == big))
    };
    assert!(offered(&mut g), "today: Big Score is offered with an empty hand");
    with(ALL, || {
        assert!(!rules::followup_ok(&g, P0, big));
        assert!(!offered(&mut g));
    });
    hand(&mut g, P0, &["Mountain"]);
    with(ALL, || assert!(rules::followup_ok(&g, P0, big)));
}

// ------------------------------------------------------------------ tutored cards
fn full_hand(g: &mut Game) -> Vec<CardId> {
    g.player_mut(P0).hand.clear();
    lands(g, P0, "Island", 4, false);
    hand(
        g,
        P0,
        &[
            "Polluted Delta",
            "Misty Rainforest",
            "Scalding Tarn",
            "Brainstorm",
            "Counterspell",
            "Sol Ring",
            "Arcane Signet",
            "Mystic Remora",
            "Rhystic Study",
        ],
    )
}

/// discarding to hand size keeps the card a tutor just found (here: the card today's pick discards)
#[test]
fn discarding_to_hand_size_keeps_the_tutored_card() {
    let mut g = game(&T5);
    let h = full_hand(&mut g);
    let old = sim_core::ai::cleanup_discard(&g, P0, &h);
    with(ALL, || {
        rules::mark_tutored(&mut g, P0, old);
        assert!(rules::tutored(&g, P0, old));
        let now = sim_core::ai::cleanup_discard(&g, P0, &h);
        assert_ne!(now, old);
        assert!(h.contains(&now));
    });
    // switch off: the mark is not read, today's pick
    assert_eq!(sim_core::ai::cleanup_discard(&g, P0, &h), old);
}

/// a discard effect (`discard_worst`: looting, "discard a card") keeps it too
#[test]
fn a_discard_effect_keeps_the_tutored_card() {
    let mut g = game(&T5);
    full_hand(&mut g);
    let before = g.player(P0).hand.clone();
    let mut g0 = g.clone();
    sim_core::engine::cast::discard_worst(&mut g0, P0, 1).unwrap();
    let gone: Vec<CardId> = before.iter().copied().filter(|c| !g0.player(P0).hand.contains(c)).collect();
    assert_eq!(gone.len(), 1);
    with(ALL, || {
        rules::mark_tutored(&mut g, P0, gone[0]);
        let mut g1 = g.clone();
        sim_core::engine::cast::discard_worst(&mut g1, P0, 1).unwrap();
        assert!(g1.player(P0).hand.contains(&gone[0]), "the tutored card stays");
        assert_eq!(g1.player(P0).hand.len(), before.len() - 1);
    });
    let mut g2 = g.clone();
    sim_core::engine::cast::discard_worst(&mut g2, P0, 1).unwrap();
    assert!(!g2.player(P0).hand.contains(&gone[0]), "switch off: today's pick");
}

/// with nothing else to discard the tutored card goes: the guard only acts while another choice exists
#[test]
fn the_tutored_card_goes_when_it_is_the_only_choice() {
    let mut g = game(&T5);
    g.player_mut(P0).hand.clear();
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    with(ALL, || {
        rules::mark_tutored(&mut g, P0, c);
        assert_eq!(rules::spare_tutored(&g, P0, &[c]), None);
        assert_eq!(sim_core::ai::cleanup_discard(&g, P0, &[c]), c);
    });
}

/// a mark lasts this turn and the next, then ends
#[test]
fn a_mark_lasts_this_turn_and_last() {
    let mut g = game(&T5);
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    with(ALL, || {
        rules::mark_tutored(&mut g, P0, c);
        g.player_mut(P1).turns += 1;
        assert!(rules::tutored(&g, P0, c), "the next turn");
        g.player_mut(P0).turns += 1;
        assert!(!rules::tutored(&g, P0, c), "two turns on");
    });
}

/// the engine's tutor marks what it finds; with the switch off it marks nothing
#[test]
fn a_tutor_marks_the_card_it_finds() {
    let mut g = game(&T5);
    let mut g0 = g.clone();
    sim_core::engine::tutors::tutor(&mut g0, P0, "any").unwrap();
    assert!(g0.player(P0).tutored.is_empty());
    with(ALL, || {
        let n = g.player(P0).hand.len();
        sim_core::engine::tutors::tutor(&mut g, P0, "any").unwrap();
        assert_eq!(g.player(P0).hand.len(), n + 1);
        let found = *g.player(P0).hand.last().unwrap();
        assert_eq!(g.player(P0).tutored.len(), 1);
        assert!(rules::tutored(&g, P0, found));
    });
}

/// a card a tutor put on top is not surveilled into the graveyard
#[test]
fn surveil_keeps_a_card_tutored_to_the_top() {
    let mut g = game(&T5);
    lands(&mut g, P0, "Island", 9, false);
    let top = take(&mut g, P0, "Island");
    g.player_mut(P0).library.push(top);
    let mut g0 = g.clone();
    sim_core::ai::topdeck::scry(&mut g0, P0, 1, true).unwrap();
    assert!(g0.player(P0).gy.contains(&top), "today: a tenth land is surveilled away");
    with(ALL, || {
        rules::mark_tutored(&mut g, P0, top);
        sim_core::ai::topdeck::scry(&mut g, P0, 1, true).unwrap();
        assert_eq!(g.player(P0).library.last(), Some(&top));
    });
}

// ------------------------------------------------------------------ combo starts
const KRAUM: [&str; 4] = [
    "kraum-tymna-blue-breach-midrange",
    "urza-mono-blue-artifacts",
    "zur-esper-enchantment-control",
    "miirym-temur-dragons",
];

/// Tainted Pact is a spent piece of "Thassa's Oracle + Demonic Consultation / Tainted Pact": with Thassa's Oracle in
/// the library and no tutor in hand it is not cast; with Demonic Tutor in hand and the mana for it, it is; with the
/// Oracle in hand too.
#[test]
fn a_combo_with_an_untutorable_missing_piece_is_not_started() {
    let mut g = game(&KRAUM);
    g.player_mut(P0).hand.clear();
    let pact = hand(&mut g, P0, &["Tainted Pact"])[0];
    let oracle = card(&g, "Thassa's Oracle");
    assert!(g.player(P0).library.contains(&oracle));
    assert!(rules::combo_ok(&g, P0, pact), "switch off: today's choice");
    let mut opts = vec![cast_opt(pact)];
    rules::filter_options(&mut g, P0, &mut opts).unwrap();
    assert_eq!(opts.len(), 1);
    with(ALL, || {
        assert!(rules::spent_piece(&g, P0, pact));
        assert!(!rules::combo_ok(&g, P0, pact));
        let mut opts = vec![cast_opt(pact)];
        rules::filter_options(&mut g, P0, &mut opts).unwrap();
        assert!(opts.is_empty());
    });
    // Demonic Tutor in hand, but no mana to cast it this turn: still not
    hand(&mut g, P0, &["Demonic Tutor"]);
    with(ALL, || assert!(!rules::combo_ok(&g, P0, pact)));
    lands(&mut g, P0, "Swamp", 2, false);
    with(ALL, || assert!(rules::combo_ok(&g, P0, pact), "a tutor in hand finds the Oracle"));
}

#[test]
fn a_combo_with_its_pieces_held_is_started() {
    let mut g = game(&KRAUM);
    g.player_mut(P0).hand.clear();
    let pact = hand(&mut g, P0, &["Tainted Pact", "Thassa's Oracle"])[0];
    with(ALL, || assert!(rules::combo_ok(&g, P0, pact)));
    // a card outside every line, and a permanent piece, are never stopped
    let oracle = card(&g, "Thassa's Oracle");
    g.player_mut(P0).hand.retain(|&c| c != oracle);
    let ring = hand(&mut g, P0, &["Sol Ring"])[0];
    g.player_mut(P0).library.push(oracle);
    with(ALL, || {
        assert!(rules::combo_ok(&g, P0, ring));
        assert!(rules::combo_ok(&g, P0, oracle));
        assert!(!rules::spent_piece(&g, P0, oracle));
    });
}
