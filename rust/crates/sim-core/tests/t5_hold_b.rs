//! t5-hold-b: a pool deck keeps mana up for protection only when it has something worth protecting and an opponent can
//! answer it at instant speed; the counter hold is unchanged; and two cards the deck never cast (Garruk's Uprising,
//! Welding Jar) are cast.

use sim_core::ai::{brain, decks, search, style};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const MIIRYM: &str = "miirym-temur-dragons";
const ZUR: &str = "zur-esper-enchantment-control";

/// Miirym on turn 7 with eight lands (Forest and Island untapped) and Heroic Intervention in hand; P1 is Zur's deck
fn miirym_table() -> Game {
    let mut g = table(&[MIIRYM, ZUR, "urza-mono-blue-artifacts", "kinnan-simic-mana-combo"]);
    g.player_mut(P0).turns = 7;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Forest", 4, false);
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Heroic Intervention"]);
    g
}

/// P1 can answer at instant speed: a hand, two mana open, and twenty instant removal spells among the cards it has not
/// yet seen
fn arm_p1(g: &mut Game) {
    lands(g, P1, "Plains", 4, false);
    g.player_mut(P1).hand.clear();
    hand(g, P1, &["Brainstorm", "Ponder", "Preordain", "Opt", "Counterspell", "Forest", "Island"]);
    let swords = card(g, "Swords to Plowshares");
    let mut names = g.player(P1).deck_names.to_vec();
    names.extend(std::iter::repeat_n(swords, 20));
    g.player_mut(P1).deck_names = names.into();
}

fn miirym_out(g: &mut Game) {
    let m = perm(g, P0, "Miirym, Sentinel Wyrm");
    g.perm_mut(m).is_cmd = true;
}

fn held(g: &Game) -> (Option<sim_core::ids::CardId>, f64) {
    let s = brain::Situation::new(g, P0);
    brain::hold_value(g, P0, &s)
}

#[test]
fn nothing_worth_protecting_no_hold() {
    // the commander is in the command zone, an opponent could answer: the old floor still held 1.5
    let mut g = miirym_table();
    arm_p1(&mut g);
    assert!(brain::removal_risk(&g, P0) >= 0.25, "the opponent can plausibly answer");
    assert!(held(&g).0.is_none(), "nothing on the battlefield to protect: no mana is kept up");
}

#[test]
fn nobody_can_answer_no_hold() {
    // the commander is out, the opponents have no cards in hand
    let mut g = miirym_table();
    miirym_out(&mut g);
    for q in 1..4 {
        g.player_mut(PlayerId(q)).hand.clear();
    }
    assert_eq!(brain::removal_risk(&g, P0), 0.0);
    assert!(held(&g).0.is_none(), "no opponent holds a card: the protection is not held");
}

#[test]
fn a_commander_out_and_an_opponent_who_can_answer_hold() {
    let mut g = miirym_table();
    miirym_out(&mut g);
    arm_p1(&mut g);
    let risk = brain::removal_risk(&g, P0);
    assert!(risk >= 0.25, "risk {risk}");
    let (c, v) = held(&g);
    assert_eq!(c, Some(card(&g, "Heroic Intervention")), "Heroic Intervention is held");
    let caution = style(&g, g.player(P0).key).caution;
    assert!((v - (1.0 + 3.0 * caution * risk)).abs() < 1e-9, "the value grows with the risk: {v}");
}

#[test]
fn the_look_ahead_mirrors_it() {
    let mut g = miirym_table();
    set_ai(&mut g, AiMode::Lookahead);
    let mut s = (*g.settings).clone();
    s.search.hold_w = 1.0;
    g.settings = Arc::new(s);
    arm_p1(&mut g);
    assert!(search::hold_credit(&g, P0).is_none(), "nothing to protect: the look-ahead credits no hold");
    let opts = brain::main_options(&mut g, P0, true).unwrap();
    assert!(opts.iter().all(|o| o.label != "stop (hold mana)"), "and the stop is a plain stop");
    miirym_out(&mut g);
    let (c, v) = search::hold_credit(&g, P0).expect("the commander is out and answerable: a hold credit");
    assert_eq!(c, card(&g, "Heroic Intervention"));
    assert!(v > 0.0);
    let opts = brain::main_options(&mut g, P0, true).unwrap();
    let stop = opts.iter().find(|o| o.label == "stop (hold mana)").expect("the hold is offered");
    assert_eq!(search::hold_bonus(&g, P0, stop), v);
}

#[test]
fn a_held_counter_is_unchanged() {
    // no commander out, nobody to answer: only the counter floor holds Counterspell. Adding a protection spell
    // changes nothing (the counter's term is exactly the old one)
    let mut g = miirym_table();
    hand(&mut g, P0, &["Counterspell"]);
    let (c, v) = held(&g);
    assert!(c.is_some(), "a held card is named");
    let s = brain::Situation::new(&g, P0);
    let caution = style(&g, g.player(P0).key).caution;
    let near = s.combo_near || s.cmdr_combo;
    let base = 1.0 + 3.0 * caution * (s.max_threat / 20.0).min(1.0) + 2.5 * near as i32 as f64;
    let floor = 1.5 + 2.5 * caution;
    assert!((v - base.max(floor)).abs() < 1e-9, "counter hold {v} = max({base}, {floor})");
    // the protection spell on its own gets no hold at all
    let mut h = miirym_table();
    assert!(held(&h).0.is_none());
    // with both, the value is the counter's
    hand(&mut h, P0, &["Counterspell"]);
    assert_eq!(held(&h).1, v);
}

#[test]
fn jd_and_tier_six_decks_keep_the_old_floor() {
    // sheoldred (JD's), cloud (JD's) and a Tier 6 deck hold Heroic Intervention with nothing out and nobody to answer
    for key in ["sheoldred", "cloud", "kinnan-bonder-mana-combo", "thrasios"] {
        let mut g = table(&[key, ZUR, "urza-mono-blue-artifacts", "kinnan-simic-mana-combo"]);
        g.player_mut(P0).turns = 7;
        g.player_mut(P0).hand.clear();
        lands(&mut g, P0, "Forest", 4, false);
        lands(&mut g, P0, "Island", 4, false);
        hand(&mut g, P0, &["Heroic Intervention"]);
        let (c, v) = held(&g);
        assert!(c.is_some() && v >= 1.5, "{key}: held {c:?} {v}");
    }
}

#[test]
fn garruks_uprising_is_cast_with_a_big_creature_out() {
    let mut g = miirym_table();
    hand(&mut g, P0, &["Garruk's Uprising"]);
    let up = card(&g, "Garruk's Uprising");
    assert_eq!(decks::deck_prio(&g, P0, up), 0, "no creature of power 4 or more out yet");
    perm(&mut g, P0, "Goldspan Dragon"); // a 4/4
    assert!(decks::deck_prio(&g, P0, up) >= 50);
    let s = brain::Situation::new(&g, P0);
    assert!(brain::card_utility(&g, P0, &s, up).is_some_and(|u| u > 0.0), "and it is wanted");
}

#[test]
fn welding_jar_is_cast_once_an_artifact_is_out() {
    let mut g = table(&["urza-mono-blue-artifacts", ZUR, MIIRYM, "kinnan-simic-mana-combo"]);
    g.player_mut(P0).turns = 7;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Island", 6, false);
    hand(&mut g, P0, &["Welding Jar"]);
    let jar = card(&g, "Welding Jar");
    assert_eq!(decks::deck_prio(&g, P0, jar), 0, "no artifact worth saving");
    perm(&mut g, P0, "Mana Vault");
    assert!(decks::deck_prio(&g, P0, jar) > 0, "a Mana Vault to regenerate");
    let s = brain::Situation::new(&g, P0);
    assert!(brain::card_utility(&g, P0, &s, jar).is_some_and(|u| u > 0.0));
}

#[test]
fn a_held_instant_removal_spell_keeps_the_old_floor() {
    // the protection term is for protection alone: with an instant removal spell held too (no counter), the old
    // outside-deck floor applies as before, even with nothing of ours on the battlefield to protect
    // (instant removal is a held card only with an opposing combo-piece commander out: Kinnan here)
    let mut g = miirym_table();
    let k = perm(&mut g, PlayerId(3), "Kinnan, Bonder Prodigy");
    g.perm_mut(k).is_cmd = true;
    assert!(brain::Situation::new(&g, P0).cmdr_combo, "Kinnan out: an opposing combo commander");
    hand(&mut g, P0, &["Beast Within"]);
    let (c, v) = held(&g);
    assert!(c.is_some(), "the removal spell's mana is still kept up");
    let caution = style(&g, g.player(P0).key).caution;
    assert!(v >= 1.5 - 1e-9 && v >= (1.5 + 2.5 * caution * 0.0) - 1e-9, "the outside-deck floor: {v}");
}
