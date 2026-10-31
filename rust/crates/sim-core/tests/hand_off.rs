//! Phase 4 PR 7: each deck's hand-written AI behind `AI_HAND` (`audit/phase4/hand-off-design.md`). With the switch
//! off (the default for every switchable deck outside `ai::HAND_KEEP` since the recalibration's stage 1) a switchable
//! deck decides through the generic policy and its plan file; with it on, its hand AI's entry points answer as before.
//! JD's decks are seated with their own tiers only.

use sim_core::ai::{self, decks, plans, pool, values};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::{hand, lands, perm, table, tier};

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

/// off: the deck is a generic-policy deck, plays from its plan file, and `deck_prio` is the pool's generic priority
/// (the hand priority function is not reached); on: none of that, and `deck_prio` is the hand function's
fn check_jd_deck(key: &'static str, t: &str, hand_prio: Prio) {
    let g = seated(key, t);
    {
        let _off = ai::force_hand(Some(false));
        assert!(!ai::hand_ai(&g, P), "{key}: off");
        assert_eq!(ai::hand_key(&g, P), "", "{key}");
        assert!(values::generic_deck(&g, P), "{key}: a generic-policy deck with the hand AI off");
        assert!(plans::plays_plan(key), "{key}: plays from data/plans/{key}.toml with the hand AI off");
        for c in cards(&g) {
            let generic = pool::generic_prio(&g, P, c);
            if generic != 0 {
                assert_eq!(decks::deck_prio(&g, P, c), generic, "{key}: {} off", g.db.get(c).name);
            }
        }
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
    assert_eq!(ai::HAND_KEEP, ["yshtola", "sheoldred", "cloud", "dack-fayden-white-stax"]);
    for k in ai::HAND_DECKS {
        let keep = ai::HAND_KEEP.contains(&k);
        assert_eq!(ai::hand_switch_wants(None, k), keep, "unset: {k}");
        assert_eq!(ai::hand_switch_wants(Some(""), k), keep, "empty: {k}");
        assert!(!ai::hand_switch_wants(Some("0"), k), "0: {k} off");
        assert!(ai::hand_switch_wants(Some("1"), k), "1: {k} on");
        assert_eq!(ai::hand_switch_wants(Some("veyran, karn"), k), matches!(k, "veyran" | "karn"), "a list: {k}");
    }
    // no AI_HAND in the test environment: the kept decks play their hand AI, every other switchable deck is off
    if std::env::var("AI_HAND").is_err() {
        for k in ai::HAND_DECKS {
            assert_eq!(ai::hand_ai_key(k), ai::HAND_KEEP.contains(&k), "{k}");
        }
    }
    let _on = ai::force_hand(Some(true));
    assert!(ai::HAND_DECKS.iter().all(|k| ai::hand_ai_key(k)));
    {
        let _off = ai::force_hand(Some(false));
        assert!(ai::HAND_DECKS.iter().all(|k| !ai::hand_ai_key(k)));
        // decks with nothing to switch, and the retired test seats, keep what they have
        for k in ["chulane-bant-value-combo", "ghave", "urdragon", "thanos", "sephfs", "godo", "scion"] {
            assert!(ai::hand_ai_key(k), "{k}");
        }
    }
    assert!(ai::hand_ai_key("veyran"), "the guard puts the previous value back");
}

/// with nothing forced, a kept deck is not a generic-policy deck and its plan file is not read; a deck switched off
/// by default is one and reads its plan file (values::generic_deck and plans::plays_plan follow hand_ai_key)
#[test]
fn the_default_reaches_generic_deck_and_the_plan_files() {
    if std::env::var("AI_HAND").is_ok() {
        return;
    }
    let g = seated("cloud", "t1");
    assert!(ai::hand_ai(&g, P) && !values::generic_deck(&g, P), "cloud: its hand AI by default");
    assert!(!plans::plays_plan("cloud"));
    let g = seated("veyran", "t1");
    assert!(!ai::hand_ai(&g, P) && values::generic_deck(&g, P), "veyran: the generic policy by default");
    assert!(plans::plays_plan("veyran"));
}

#[test]
fn every_switchable_jd_deck_has_a_plan_file() {
    for k in ai::HAND_OFF_PLANS {
        assert!(ai::plan_data::get(k).is_some(), "data/plans/{k}.toml");
    }
}

#[test]
fn veyran_switch() {
    check_jd_deck("veyran", "t1", |g, p, c| ai::veyran::veyran_prio(g, p, c) as f64);
}

#[test]
fn sauron_switch() {
    check_jd_deck("sauron", "t3", |g, p, c| decks::sauron_prio(g, p, c) as f64);
}

#[test]
fn yshtola_switch() {
    check_jd_deck("yshtola", "t5", sim_core::impls::yshtola::yshtola_prio);
}

#[test]
fn galadriel_switch() {
    check_jd_deck("galadriel", "t1", |g, p, c| sim_core::impls::galadriel::galadriel_prio(g, p, c) as f64);
}

#[test]
fn alela_switch() {
    check_jd_deck("alela", "t1", |g, p, c| sim_core::impls::alela::alela_prio(g, p, c) as f64);
}

#[test]
fn kefka_switch() {
    check_jd_deck("kefka", "t3", |g, p, c| sim_core::impls::kefka::kefka_prio(g, p, c) as f64);
}

#[test]
fn sheoldred_switch() {
    check_jd_deck("sheoldred", "t3", |g, p, c| sim_core::impls::sheoldred::sheoldred_prio(g, p, c) as f64);
}

#[test]
fn cloud_switch() {
    check_jd_deck("cloud", "t1", |g, p, c| sim_core::impls::cloud::cloud_prio(g, p, c) as f64);
}

#[test]
fn karn_switch() {
    check_jd_deck("karn", "t4", |g, p, c| sim_core::impls::karn::karn_prio(g, p, c) as f64);
    // the Tron land preference is the hand AI's
    let g = seated("karn", "t4");
    let tron = g.db.id("Urza's Tower").expect("Urza's Tower");
    {
        let _off = ai::force_hand(Some(false));
        assert_eq!(sim_core::impls::karn::land_adjust(&g, P, tron), 0.0);
    }
    let _on = ai::force_hand(Some(true));
    assert!(sim_core::impls::karn::land_adjust(&g, P, tron) > 0.0);
}

#[test]
fn thrasios_switch() {
    check_jd_deck("thrasios", "t5", |g, p, c| sim_core::impls::tymna_partners::thrasios_prio(g, p, c) as f64);
}

#[test]
fn brago_switch() {
    let key = "brago-azorius-blink-control";
    let mut g = seated(key, "t2");
    let c = hand(&mut g, P, &["Cloudshift"])[0];
    {
        let _off = ai::force_hand(Some(false));
        assert!(!sim_core::impls::cc_brago::hold(&g, P, c), "off: the generic priority decides");
        assert!(plans::plays_plan(key));
    }
    let _on = ai::force_hand(Some(true));
    // no creature worth blinking: Cloudshift waits
    assert!(sim_core::impls::cc_brago::hold(&g, P, c));
    assert!(!plans::plays_plan(key));
}

#[test]
fn aurelia_switch() {
    let key = "aurelia-boros-extra-combats";
    let mut g = seated(key, "t3");
    perm(&mut g, P, "Aurelia, the Warleader");
    let helm = perm(&mut g, P, "Helm of the Host");
    lands(&mut g, P, "Mountain", 8, false);
    g.active = Some(P);
    let f = g.registry.get(g.perm(helm).card()).and_then(|i| i.options).expect("Helm's options");
    {
        let _off = ai::force_hand(Some(false));
        assert!(f(&mut g, helm, P, Some(false)).unwrap().is_empty(), "off: the generic equip choice");
        assert!(plans::plays_plan(key));
    }
    let _on = ai::force_hand(Some(true));
    assert!(!f(&mut g, helm, P, Some(false)).unwrap().is_empty(), "on: Helm onto Aurelia before combat");
    assert!(!plans::plays_plan(key));
}

fn noop(_: &mut Game, _: PermId, _: PlayerId, _: i64) -> sim_core::flow::Res<bool> {
    Ok(false)
}

#[test]
fn sisay_switch() {
    let mut keys = vec!["sisay-legends-toolbox"];
    keys.extend(tier("t6").into_iter().filter(|k| !k.starts_with("sisay")).take(3));
    let mut g = table(&keys);
    let src = lands(&mut g, P, "Forest", 1, false)[0];
    let opts =
        vec![Opt { utility: 1.0, label: "Sisay: a fetch".into(), act: Some(Action::Ability { src, f: noop, arg: 0 }) }];
    {
        let _off = ai::force_hand(Some(false));
        assert_eq!(ai::t6_table::must_option(&g, P, &opts), None, "off: no Sisay-only rule");
    }
    let _on = ai::force_hand(Some(true));
    assert_eq!(ai::t6_table::must_option(&g, P, &opts), Some(0), "on: the Sisay fetch is a must-do");
}

#[test]
fn dack_switch() {
    let key = "dack-fayden-white-stax";
    let mut keys = vec![key];
    keys.extend(tier("t6").into_iter().filter(|k| *k != key).take(3));
    let g = table(&keys);
    let plan = ai::plan_data::get(key).expect("dack's plan file");
    {
        let _off = ai::force_hand(Some(false));
        assert!(plans::plays_plan(key));
        for c in cards(&g) {
            assert_eq!(
                sim_core::impls::t6_plans::prio(&g, P, c),
                ai::plan_data::t6::prio(plan, &g, P, c),
                "off: {}",
                g.db.get(c).name
            );
        }
    }
    let _on = ai::force_hand(Some(true));
    assert!(!plans::plays_plan(key));
    for c in cards(&g) {
        assert_eq!(
            sim_core::impls::t6_plans::prio(&g, P, c),
            sim_core::impls::t6_plan_dack::prio(&g, P, c),
            "on: {}",
            g.db.get(c).name
        );
    }
}
