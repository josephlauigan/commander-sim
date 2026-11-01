//! Phase 4 PR 7: each deck's hand-written AI behind `AI_HAND` (`audit/phase4/hand-off-design.md`). With the switch
//! off (the default for every switchable deck outside `ai::HAND_KEEP` since the recalibration's stage 1) a switchable
//! deck decides through the generic policy and its plan file; with it on, its hand AI's entry points answer as before.
//! The hand AI of the eleven decks whose generic policy was level or better was deleted on 2026-11-01 ("Deleted" in the
//! design): they play the generic policy and their plan file whatever the switch says. JD's decks are seated with
//! their own tiers only.

use sim_core::ai::{self, decks, plans, pool, values};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::{lands, table, tier};

const P: PlayerId = PlayerId(0);

/// the deck in seat 0 and three pool decks of its own tier
fn seated(key: &'static str, t: &str) -> Game {
    let mut keys = vec![key];
    keys.extend(tier(t).into_iter().filter(|k| *k != key).take(3));
    table(&keys)
}

fn cards(g: &Game) -> Vec<CardId> {
    let pl = g.player(P);
    pl.library.iter().chain(&pl.hand).copied().collect()
}

type Prio = fn(&Game, PlayerId, CardId) -> f64;

/// the generic policy: a generic-policy deck that plays from its plan file, `deck_prio` the pool's generic priority
fn check_generic(g: &Game, key: &str) {
    assert!(!ai::hand_ai(g, P), "{key}: off");
    assert_eq!(ai::hand_key(g, P), "", "{key}");
    assert!(values::generic_deck(g, P), "{key}: a generic-policy deck with the hand AI off");
    assert!(plans::plays_plan(key), "{key}: plays from data/plans/{key}.toml with the hand AI off");
    for c in cards(g) {
        let generic = pool::generic_prio(g, P, c);
        if generic != 0 {
            assert_eq!(decks::deck_prio(g, P, c), generic, "{key}: {} off", g.db.get(c).name);
        }
    }
}

/// off: the generic policy (the hand priority function is not reached); on: none of that, and `deck_prio` is the hand
/// function's
fn check_jd_deck(key: &'static str, t: &str, hand_prio: Prio) {
    let g = seated(key, t);
    {
        let _off = ai::force_hand(Some(false));
        check_generic(&g, key);
    }
    let _on = ai::force_hand(Some(true));
    assert!(ai::hand_ai(&g, P));
    assert_eq!(ai::hand_key(&g, P), key);
    assert!(!values::generic_deck(&g, P), "{key}: its hand AI on");
    assert!(!plans::plays_plan(key), "{key}: the new plan file is not read with the hand AI on");
    let mut differs = false;
    for c in cards(&g) {
        let h = hand_prio(&g, P, c);
        if h != 0.0 {
            assert_eq!(decks::deck_prio_f(&g, P, c), h, "{key}: {} on", g.db.get(c).name);
        }
        differs |= h != pool::generic_prio(&g, P, c) as f64;
    }
    assert!(differs, "{key}: the hand priority differs from the generic one somewhere");
}

#[test]
fn the_switch_defaults_to_the_kept_decks_and_leaves_other_decks_alone() {
    // the stage 1 defaults (audit/recal/pairs-2026-10-31.md section 4): unset or empty, the hand AI on for exactly
    // HAND_KEEP; 0 off for every switchable deck; 1 all; a key list those keys
    assert_eq!(ai::HAND_KEEP, ["yshtola", "sheoldred", "cloud"]);
    assert_eq!(ai::HAND_DECKS, ai::HAND_KEEP, "the switchable decks are the kept ones since the deletion");
    for k in ai::HAND_DECKS {
        let keep = ai::HAND_KEEP.contains(&k);
        assert_eq!(ai::hand_switch_wants(None, k), keep, "unset: {k}");
        assert_eq!(ai::hand_switch_wants(Some(""), k), keep, "empty: {k}");
        assert!(!ai::hand_switch_wants(Some("0"), k), "0: {k} off");
        assert!(ai::hand_switch_wants(Some("1"), k), "1: {k} on");
        assert_eq!(ai::hand_switch_wants(Some("cloud, karn"), k), k == "cloud", "a list: {k}");
    }
    // no AI_HAND in the test environment: the kept decks play their hand AI
    if std::env::var("AI_HAND").is_err() {
        for k in ai::HAND_DECKS {
            assert!(ai::hand_ai_key(k), "{k}");
        }
    }
    let _on = ai::force_hand(Some(true));
    assert!(ai::HAND_DECKS.iter().all(|k| ai::hand_ai_key(k)));
    assert!(ai::HAND_DELETED.iter().all(|k| !ai::hand_ai_key(k)), "a deleted hand AI can't be switched on");
    {
        let _off = ai::force_hand(Some(false));
        assert!(ai::HAND_DECKS.iter().all(|k| !ai::hand_ai_key(k)));
        // decks with nothing to switch, and the retired test seats, keep what they have
        for k in ["chulane-bant-value-combo", "ghave", "urdragon", "thanos", "sephfs", "godo", "scion"] {
            assert!(ai::hand_ai_key(k), "{k}");
        }
    }
    assert!(ai::hand_ai_key("cloud"), "the guard puts the previous value back");
}

/// with nothing forced, a kept deck is not a generic-policy deck and its plan file is not read (values::generic_deck
/// and plans::plays_plan follow hand_ai_key)
#[test]
fn the_default_reaches_generic_deck_and_the_plan_files() {
    if std::env::var("AI_HAND").is_ok() {
        return;
    }
    let g = seated("cloud", "t1");
    assert!(ai::hand_ai(&g, P) && !values::generic_deck(&g, P), "cloud: its hand AI by default");
    assert!(!plans::plays_plan("cloud"));
}

#[test]
fn every_switchable_and_deleted_deck_has_a_plan_file() {
    let pool = ["brago-azorius-blink-control", "aurelia-boros-extra-combats", "dack-fayden-white-stax"];
    for k in ai::HAND_DECKS.iter().chain(&ai::HAND_DELETED).chain(&pool) {
        assert!(ai::plan_data::get(k).is_some(), "data/plans/{k}.toml");
    }
}

#[test]
fn yshtola_switch() {
    check_jd_deck("yshtola", "t5", sim_core::impls::yshtola::yshtola_prio);
}

#[test]
fn sheoldred_switch() {
    check_jd_deck("sheoldred", "t3", |g, p, c| sim_core::impls::sheoldred::sheoldred_prio(g, p, c) as f64);
}

#[test]
fn cloud_switch() {
    check_jd_deck("cloud", "t1", |g, p, c| sim_core::impls::cloud::cloud_prio(g, p, c) as f64);
}

/// JD's (and Avery's) decks whose hand AI was deleted: the generic policy and the plan file, even with the hand AI
/// forced on
#[test]
fn deleted_jd_decks_play_the_generic_policy() {
    let _on = ai::force_hand(Some(true));
    for (key, t) in [
        ("veyran", "t1"),
        ("sauron", "t3"),
        ("galadriel", "t1"),
        ("alela", "t1"),
        ("kefka", "t3"),
        ("karn", "t4"),
        ("thrasios", "t5"),
    ] {
        assert!(ai::HAND_DELETED.contains(&key), "{key}");
        check_generic(&seated(key, t), key);
    }
}

#[test]
fn deleted_pool_decks_play_their_plan_files() {
    let _on = ai::force_hand(Some(true));
    for key in ["brago-azorius-blink-control", "aurelia-boros-extra-combats"] {
        assert!(plans::plays_plan(key), "{key}");
    }
}

fn noop(_: &mut Game, _: PermId, _: PlayerId, _: i64) -> sim_core::flow::Res<bool> {
    Ok(false)
}

#[test]
fn sisay_has_no_sisay_only_rule() {
    let _on = ai::force_hand(Some(true));
    let mut keys = vec!["sisay-legends-toolbox"];
    keys.extend(tier("t6").into_iter().filter(|k| !k.starts_with("sisay")).take(3));
    let mut g = table(&keys);
    let src = lands(&mut g, P, "Forest", 1, false)[0];
    let opts =
        vec![Opt { utility: 1.0, label: "Sisay: a fetch".into(), act: Some(Action::Ability { src, f: noop, arg: 0 }) }];
    assert_eq!(ai::t6_table::must_option(&g, P, &opts), None);
}

#[test]
fn dack_plays_its_plan_file() {
    let _on = ai::force_hand(Some(true));
    let key = "dack-fayden-white-stax";
    let mut keys = vec![key];
    keys.extend(tier("t6").into_iter().filter(|k| *k != key).take(3));
    let g = table(&keys);
    let plan = ai::plan_data::get(key).expect("dack's plan file");
    assert!(plans::plays_plan(key));
    for c in cards(&g) {
        assert_eq!(
            sim_core::impls::t6_plans::prio(&g, P, c),
            ai::plan_data::t6::prio(plan, &g, P, c),
            "{}",
            g.db.get(c).name
        );
    }
}
