//! Phase 4 PR 7: the decks' hand-written AI, deleted (`audit/phase4/hand-off-design.md`, "Deleted"). The nine decks in
//! `ai::HAND_DELETED` (JD's and Avery's: sauron, galadriel, yshtola, alela, kefka, sheoldred, cloud, karn, thrasios)
//! and the pool decks whose hand code went (brago, aurelia, sisay, dack) decide through the generic policy and their
//! plan file; no switch brings the hand AI back. JD's decks are seated with their own tiers only.

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

/// the generic policy: a generic-policy deck that plays from its plan file, `deck_prio` the pool's generic priority
fn check_generic(g: &Game, key: &str) {
    assert!(!ai::hand_ai_key(key), "{key}: no hand AI");
    assert_eq!(ai::hand_key(g, P), "", "{key}");
    assert!(values::generic_deck(g, P), "{key}: a generic-policy deck");
    assert!(plans::plays_plan(key), "{key}: plays from data/plans/{key}.toml");
    for c in cards(g) {
        let generic = pool::generic_prio(g, P, c);
        if generic != 0 {
            assert_eq!(decks::deck_prio(g, P, c), generic, "{key}: {}", g.db.get(c).name);
        }
    }
}

/// JD's (and Avery's) decks: the generic policy and the plan file
#[test]
fn the_deleted_jd_decks_play_the_generic_policy() {
    assert_eq!(ai::HAND_DELETED.len(), 9);
    for (key, t) in [
        ("sauron", "t3"),
        ("galadriel", "t1"),
        ("yshtola", "t5"),
        ("alela", "t1"),
        ("kefka", "t3"),
        ("sheoldred", "t3"),
        ("cloud", "t1"),
        ("karn", "t4"),
        ("thrasios", "t5"),
    ] {
        assert!(ai::HAND_DELETED.contains(&key), "{key}");
        check_generic(&seated(key, t), key);
    }
}

#[test]
fn decks_with_nothing_to_switch_and_the_retired_test_seats_keep_what_they_have() {
    for k in ["chulane-bant-value-combo", "ghave", "urdragon", "thanos", "sephfs", "godo", "scion"] {
        assert!(ai::hand_ai_key(k), "{k}");
    }
}

#[test]
fn every_deleted_deck_has_a_plan_file() {
    let pool = ["brago-azorius-blink-control", "aurelia-boros-extra-combats", "dack-fayden-white-stax"];
    for k in ai::HAND_DELETED.iter().chain(&pool) {
        assert!(ai::plan_data::get(k).is_some(), "data/plans/{k}.toml");
    }
}

#[test]
fn deleted_pool_decks_play_their_plan_files() {
    for key in ["brago-azorius-blink-control", "aurelia-boros-extra-combats"] {
        assert!(plans::plays_plan(key), "{key}");
    }
}

fn noop(_: &mut Game, _: PermId, _: PlayerId, _: i64) -> sim_core::flow::Res<bool> {
    Ok(false)
}

#[test]
fn sisay_has_no_sisay_only_rule() {
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
