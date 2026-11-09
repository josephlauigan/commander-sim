//! Regression tests for when Mox Jasper counts as mana (it needs a Dragon the hand can cast), written adversarially
//! against the first fix: sequencing, Petal, double-mana lands, dorks, zero-cost Moxen, and that hands without Jasper
//! are untouched.

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::advisor;
use serde_json::{Value, json};
use sim_core::control::Request;
use sim_core::ids::PlayerId;
use sim_core::impls::t6_keep;
use sim_core::run::settings;
use sim_core::state::Game;

/// a game whose seat 0 is `deck`, with exactly this hand (taken from the library; the rest stays shuffled)
fn game_with(deck: &str, hand: &[&str]) -> (Game, PlayerId) {
    let e = engine();
    let others = ["aragorn", "sheoldred", "cloud", "thanos", "yshtola"];
    let mut seats: Vec<String> = vec![deck.to_string()];
    seats.extend(others.iter().filter(|k| **k != deck).take(3).map(|s| s.to_string()));
    let (mut g, _) =
        practice::session::setup::build_game(&e, settings("loose", "adaptive", 1.0).unwrap(), 77, &seats, &[]).unwrap();
    let p = PlayerId(0);
    let back = std::mem::take(&mut g.player_mut(p).hand);
    g.player_mut(p).library.extend(back);
    for n in hand {
        let i = g
            .player(p)
            .library
            .iter()
            .position(|&c| &*g.db.get(c).name == *n)
            .unwrap_or_else(|| panic!("{deck}: no {n} in the library"));
        let c = g.player_mut(p).library.remove(i);
        g.player_mut(p).hand.push(c);
    }
    (g, p)
}

fn req(prompt: &str) -> Request {
    Request {
        kind: sim_core::sym::intern("mulligan"),
        prompt: prompt.into(),
        choices: vec!["keep".into(), "mulligan".into()],
        data: json!({}),
    }
}

fn hint(g: &mut Game, p: PlayerId) -> Value {
    advisor::hint(g, p, &req("Your opening hand (7 to keep)."))
}

fn lines(h: &Value) -> Vec<String> {
    h["detail"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
}

fn has(h: &Value, needle: &str) -> bool {
    lines(h).iter().any(|l| l.contains(needle)) || h["text"].as_str().unwrap().contains(needle)
}

// Synthetic hands use real exported CardIds, including cards outside the chosen deck and duplicate Jaspers.
// This deliberately tests the shared estimator independently of singleton/deck-list validation.
fn fixture(hand: &[&str]) -> (Game, PlayerId) {
    let (mut g, p) = game_with("miirym-temur-dragons", &[]);
    let ids = hand.iter().map(|n| g.db.id(n).unwrap_or_else(|| panic!("missing {n}"))).collect();
    g.player_mut(p).hand = ids;
    (g, p)
}

fn jasper_values(g: &Game, p: PlayerId) -> Vec<f64> {
    t6_keep::piece_values(g, p)
        .into_iter()
        .filter(|(c, _)| g.db.get(*c).name.as_ref() == "Mox Jasper")
        .map(|(_, v)| v)
        .collect()
}

fn mana_line(g: &mut Game, p: PlayerId) -> String {
    lines(&hint(g, p)).into_iter().find(|l| l.starts_with("Mana on turn 2:")).unwrap()
}

#[test]
fn miirym_exact_cost_dragon_activates_jasper() {
    let (g, p) = game_with("miirym-temur-dragons", &["Forest", "Island", "Mox Jasper", "Korlessa, Scale Singer"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
    let m = t6_keep::mana(&g, p);
    assert_eq!(m.eff, 3.0);
    assert_eq!(m.n_mana, 1);
}

#[test]
fn creature_changeling_counts_as_dragon() {
    let (g, p) = fixture(&["Island", "Mox Jasper", "Mothdust Changeling"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
}

#[test]
fn maskwood_nexus_is_not_itself_a_dragon() {
    let (mut g, p) = fixture(&["Forest", "Island", "Sol Ring", "Mox Jasper", "Maskwood Nexus"]);
    assert!(!g.db.get(g.db.id("Maskwood Nexus").unwrap()).creature);
    assert_eq!(jasper_values(&g, p), vec![0.0]);
    assert!(has(&hint(&mut g, p), "Mox Jasper (needs a Dragon you can cast first)"));
}

#[test]
fn noncreature_dragon_back_face_does_not_activate_jasper() {
    let (g, p) =
        fixture(&["Forest", "Island", "Sol Ring", "Mox Jasper", "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen"]);
    let d = g.db.get(g.db.id("Nicol Bolas, the Ravager // Nicol Bolas, the Arisen").unwrap());
    assert!(!d.creature);
    assert!(d.has_subtype("dragon"));
    assert_eq!(jasper_values(&g, p), vec![0.0]);
}

#[test]
fn dragon_cannot_use_jasper_to_activate_itself() {
    let (g, p) = fixture(&["Forest", "Island", "Mox Jasper", "Realmwalker"]);
    assert_eq!(jasper_values(&g, p), vec![0.0]);
}

#[test]
fn zero_lands_and_no_other_sources_leave_jasper_dead() {
    let (g, p) = fixture(&["Mox Jasper", "Mothdust Changeling"]);
    assert_eq!(jasper_values(&g, p), vec![0.0]);
    assert_eq!(t6_keep::mana(&g, p).n_mana, 0);
}

#[test]
fn two_jaspers_cannot_bootstrap_a_dragon() {
    let (g, p) = fixture(&["Forest", "Island", "Mox Jasper", "Mox Jasper", "Realmwalker"]);
    assert_eq!(jasper_values(&g, p), vec![0.0, 0.0]);
}

#[test]
fn lotus_petal_can_cast_one_mana_dragon_without_lands() {
    // Petal pays U for Mothdust; Jasper then taps. Discounting Petal's value must not change castability.
    let (g, p) = fixture(&["Mox Jasper", "Lotus Petal", "Mothdust Changeling"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
}

#[test]
fn land_plus_petal_can_cast_exactly_two_mana_dragon() {
    // Forest + Petal's U casts Korlessa on turn 1, with no draw needed.
    let (g, p) = fixture(&["Forest", "Mox Jasper", "Lotus Petal", "Korlessa, Scale Singer"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
}

#[test]
fn dragon_cannot_borrow_turn_three_mind_stone_mana() {
    // Two lands cannot cast Mind Stone AND Realmwalker by turn 2; Stone first makes mana on turn 3.
    let (g, p) = fixture(&["Forest", "Island", "Mox Jasper", "Mind Stone", "Realmwalker"]);
    assert_eq!(jasper_values(&g, p), vec![0.0]);
}

#[test]
fn conditional_jasper_must_not_cast_mind_stone_before_dragon() {
    let (mut g, p) = fixture(&["Forest", "Island", "Mox Jasper", "Mind Stone", "Korlessa, Scale Singer"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
    let line = mana_line(&mut g, p);
    // Only one land is available on turn 1 and Korlessa costs 2: Jasper cannot power a turn-1 Stone.
    assert!(!line.contains("Mind Stone 1 (cast turn 1)"), "{line}");
}

#[test]
fn conditional_jasper_must_not_cast_sol_ring_without_a_starter() {
    let (mut g, p) = fixture(&["Mox Jasper", "Lotus Petal", "Sol Ring", "Realmwalker"]);
    // Spend Petal on Ring on turn 1. Ring makes only 2, insufficient for Realmwalker on either turn.
    assert_eq!(jasper_values(&g, p), vec![0.0]);
    let line = mana_line(&mut g, p);
    assert!(!line.contains("Mox Jasper 1"), "{line}");
}

#[test]
fn petal_ring_and_jasper_are_hand_order_independent() {
    let hand = ["Forest", "Island", "Mox Jasper", "Lotus Petal", "Sol Ring", "Dragonborn Champion"];
    let (g, p) = fixture(&hand);
    let expected = jasper_values(&g, p);
    assert_eq!(expected, vec![1.0]);
    for shift in 0..hand.len() {
        let mut perm = hand;
        perm.rotate_left(shift);
        let (mut g, p) = fixture(&perm);
        assert_eq!(jasper_values(&g, p), expected);
        let m = t6_keep::mana(&g, p);
        assert_eq!(m.n_mana, 3);
        assert!((m.eff - 5.7).abs() < 1e-9); // the keep rule counts Jasper; the hint has it make mana from turn 3
        assert!(mana_line(&mut g, p).starts_with("Mana on turn 2: 4.7 ="));
    }
}

#[test]
fn double_mana_land_is_available_to_cast_dragon() {
    // Turn 1 Tomb; turn 2 Forest; 2 generic + G casts Realmwalker without Jasper.
    let (g, p) = fixture(&["Ancient Tomb", "Forest", "Mox Jasper", "Realmwalker"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
}

#[test]
fn active_jasper_is_not_a_free_piece() {
    let (g, p) = fixture(&["Island", "Mox Jasper", "Mothdust Changeling"]);
    assert_eq!(jasper_values(&g, p), vec![1.0]);
    // An active Jasper is not "mana without a land" any more than Mox Amber or Mox Opal are, and nothing reads `free`.
    assert_eq!(t6_keep::mana(&g, p).free, 0);
}

/// when Jasper switches on, hand by hand (the Dragon's turn, and so the first turn Jasper makes mana is the next)
#[test]
fn jasper_switches_on_the_turn_after_the_dragon_can_be_cast() {
    let hands: [(&[&str], Option<usize>); 5] = [
        (&["Forest", "Island", "Mox Jasper", "Mind Stone", "Korlessa, Scale Singer"], Some(2)),
        (&["Forest", "Island", "Mox Jasper", "Mind Stone", "Realmwalker"], None),
        (&["Forest", "Island", "Mox Jasper", "Sol Ring", "Realmwalker"], Some(2)),
        (&["Island", "Mox Jasper", "Mothdust Changeling"], Some(1)),
        (&["Forest", "Island", "Mox Jasper", "Mind Stone", "Mothdust Changeling"], Some(1)),
    ];
    for (hand, expect) in hands {
        let (mut g, p) = fixture(hand);
        let lands = hand.iter().filter(|n| matches!(**n, "Forest" | "Island")).count();
        let turn = t6_keep::dragon_cast(&g, p, lands).map(|x| x.0);
        // Realmwalker (cost 3) with only a Mind Stone cast turn 2 is turn 3: past turn 2, so Jasper is worth 0
        match expect {
            Some(_) => assert_eq!(turn, expect, "{hand:?}"),
            None => assert!(turn.is_none_or(|t| t > 2), "{hand:?}: {turn:?}"),
        }
        assert_eq!(jasper_values(&g, p), vec![if expect.is_some() { 1.0 } else { 0.0 }], "{hand:?}");
        let h = hint(&mut g, p);
        match expect {
            Some(t) => assert!(has(&h, &format!("is cast turn {t}")) && has(&h, "Mox Jasper"), "{hand:?}: {h}"),
            None => assert!(!has(&h, "Mox Jasper 1"), "{hand:?}: {h}"),
        }
    }
}

/// a Dragon in hand with no Mox Jasper changes nothing in the hint: the Signet is still cast on turn 2
#[test]
fn a_dragon_without_jasper_does_not_delay_rocks_in_the_hint() {
    let (mut g, p) = fixture(&["Forest", "Island", "Arcane Signet", "Korlessa, Scale Singer"]);
    let h = hint(&mut g, p);
    assert!(has(&h, "Arcane Signet (cast turn 2, makes mana from turn 3)"), "{h}");
}

/// the Dragon is paid with a zero-cost Mox's mana first: Forest + Chrome Mox cast Korlessa (2) on turn 1, so Sol Ring
/// waits for turn 2 and turn 2 has 1 land + Chrome Mox + Jasper = 3 (the bug showed Sol Ring cast turn 1 and 5)
#[test]
fn a_zero_cost_mox_pays_for_the_dragon_before_a_rock_is_cast() {
    let (mut g, p) = fixture(&[
        "Forest",
        "Chrome Mox",
        "Korlessa, Scale Singer",
        "Mox Jasper",
        "Sol Ring",
        "Counterspell",
        "Brainstorm",
    ]);
    let h = hint(&mut g, p);
    assert!(mana_line(&mut g, p).starts_with("Mana on turn 2: 3 ="), "{h}");
    assert!(has(&h, "Sol Ring (cast turn 2"), "{h}");
}

/// a dork cast turn 1 makes a whole mana on turn 2 for casting the Dragon (summoning sickness is the next-turn rule)
#[test]
fn a_mana_dork_counts_as_a_whole_mana_for_the_dragon() {
    let (g, p) = fixture(&["Forest", "Island", "Birds of Paradise", "Mox Jasper", "Realmwalker"]);
    assert_eq!(t6_keep::dragon_cast(&g, p, 2).map(|x| x.0), Some(2));
    assert_eq!(jasper_values(&g, p), vec![1.0]);
}
