//! Plan files that replace the hand AI of cloud, yshtola and sheoldred (`audit/<deck>/plan-file-2026-11-01.md`):
//! the schema features they use (`[equip]`, `plays`, `[[stay_home]]`, `[[prio_add]]`, the `commander_chumps_to_live`
//! and `adnaus_main2` knobs; `audit/phase4/plans-design.md`) and, for each hand-AI decision a plan file now encodes,
//! a hand-built position where the generic policy (hand AI off, `force_hand(Some(false))`) makes the hand AI's choice.

use sim_core::ai::{self, plan_data, plans};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Game, Step};
use sim_core::testkit::{hand, lands, perm, table, tier};

const P: PlayerId = PlayerId(0);
const Q: PlayerId = PlayerId(1);

/// the deck in seat 0 and three pool decks of its own tier
fn seated(key: &'static str, t: &str) -> Game {
    let mut keys = vec![key];
    keys.extend(tier(t).into_iter().filter(|k| *k != key).take(3));
    table(&keys)
}

/// p's commander put onto the battlefield as if cast, ready to attack
fn commander(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let m = perm(g, p, name);
    g.perm_mut(m).is_cmd = true;
    g.perm_mut(m).sick = false;
    g.player_mut(p).cmd_in_zone = false;
    m
}

/// the generic equip option for Equipment `e`: (target, utility)
fn equip_of(g: &Game, e: PermId) -> Option<(PermId, f64)> {
    sim_core::dsl::equip_options(g, P).into_iter().find_map(|o| match o.act {
        Some(Action::Equip { src, target, .. }) if src == e => Some((target, o.utility)),
        _ => None,
    })
}

/// the card-code options of permanent `src`
fn perm_options(g: &mut Game, src: PermId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("card options");
    f(g, src, P, post).unwrap()
}

/// the hand options of card c in p's hand
fn hand_options(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, P, post).unwrap()
}

// ------------------------------------------------------------------ the default is unchanged
/// the new schema features are used by these three plan files only, and those are read only with the hand AI off:
/// with the defaults (hand AI on for the three) nothing changes for any deck
#[test]
fn only_the_three_plan_files_use_the_new_features_and_only_with_the_hand_ai_off() {
    for (k, l) in plan_data::all() {
        let p = &l.plan;
        let uses = p.equip.is_some()
            || !p.plays.is_empty()
            || !p.stay_home.is_empty()
            || !p.prio_add.is_empty()
            || p.knobs.commander_chumps_to_live.is_some()
            || p.knobs.adnaus_main2.is_some();
        // --- generic-paths (2026-11-01): sauron's plan lists Exsanguinate in `plays` (its hand AI is deleted, so its
        // plan file is always read: that changes its default play, on purpose)
        let sauron_plays = *k == "sauron" && p.plays == ["Exsanguinate"];
        assert_eq!(uses, matches!(*k, "cloud" | "yshtola" | "sheoldred") || sauron_plays, "{k}");
    }
    for (key, t) in [("cloud", "t1"), ("yshtola", "t5"), ("sheoldred", "t3")] {
        let g = seated(key, t);
        let _on = ai::force_hand(Some(true));
        assert!(plan_data::active(&g, P).is_none(), "{key}: its plan file is not read with the hand AI on");
        assert!(!plans::plays_plan(key));
        let _off = ai::force_hand(Some(false));
        assert!(plan_data::active(&g, P).is_some(), "{key}: read with the hand AI off");
    }
}

#[test]
fn the_new_features_are_validated() {
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    assert!(bad("deck = \"x\"\n[equip]\nholder = \"Thalia\"").contains("only \"commander\""));
    assert!(bad("deck = \"x\"\n[equip]\nholder = \"commander\"\nextra = { \"A\" = [11, 0] }").contains("above 10"));
    assert!(bad("deck = \"x\"\n[[stay_home]]\nwhen = []").contains("no when"));
    assert!(bad("deck = \"x\"\n[[stay_home]]\nwhen = [\"cmd-wears:gear=0\"]").contains("no set \"gear\""));
    assert!(bad("deck = \"x\"\n[[prio_add]]\nvalue = 3").contains("prio_add rule 1: it selects no card"));
    assert!(bad("deck = \"x\"\nplays = 3").contains("invalid type"));
    let ok = "deck = \"x\"\nplays = [\"A\"]\n[sets]\ngear = [\"B\"]\n[knobs]\ncommander_chumps_to_live = true\n\
              adnaus_main2 = true\n[equip]\nholder = \"commander\"\nmove = false\n[[stay_home]]\n\
              when = [\"cmd-blockers>=1 & atk-power - def-life<0 & (cmd-wears:gear=0 | cmd-killers>=1) & \
              not commander-indestructible & cmd-power<4\"]\n[[prio_add]]\ncreature = false\nvalue = 2";
    let l = plan_data::load("x", ok).unwrap();
    assert_eq!(l.plan.equip.as_ref().unwrap().moves, Some(false));
    // the plan test checks the names these name: plays, the equip extras, the prio_add cards
    let named = plan_data::named_cards(&l);
    assert!(named.contains(&"A".to_string()) && named.contains(&"B".to_string()));
}

// ------------------------------------------------------------------ cloud
/// was cloud::holder_bonus: the Equipment goes on Cloud, not on a bigger creature
#[test]
fn cloud_the_sword_goes_on_cloud() {
    let mut g = seated("cloud", "t1");
    lands(&mut g, P, "Plains", 4, false);
    let cl = commander(&mut g, P, "Cloud, Midgar Mercenary");
    perm(&mut g, P, "Archon of Absolution"); // a 3/2 flier: the generic score's pick
    let e = perm(&mut g, P, "Sword of Fire and Ice");
    let _on = ai::force_hand(Some(true));
    assert_eq!(equip_of(&g, e).map(|x| x.0), Some(cl), "the hand AI");
    let _off = ai::force_hand(Some(false));
    let (t, u) = equip_of(&g, e).expect("an equip option");
    assert_eq!(t, cl, "the plan's [equip]");
    assert!(u > 6.0, "the plan's utility is added ({u})");
}

/// was cloud::wants_move: a Sword worn by another creature moves onto Cloud once he is out
#[test]
fn cloud_worn_equipment_moves_onto_cloud() {
    let mut g = seated("cloud", "t1");
    lands(&mut g, P, "Plains", 4, false);
    let a = perm(&mut g, P, "Archon of Absolution");
    let e = perm(&mut g, P, "Sword of Hearth and Home");
    g.perm_mut(e).attached = Some(a);
    let cl = commander(&mut g, P, "Cloud, Midgar Mercenary");
    let _on = ai::force_hand(Some(true));
    assert_eq!(equip_of(&g, e).map(|x| x.0), Some(cl), "the hand AI");
    let _off = ai::force_hand(Some(false));
    assert_eq!(equip_of(&g, e).map(|x| x.0), Some(cl), "the plan's [equip] move");
}

/// was cloud_special: Champion's Helm and Sword of Feast and Famine (equip only in card code) are equipped to Cloud
#[test]
fn cloud_the_helm_and_feast_and_famine_are_equipped() {
    let mut g = seated("cloud", "t1");
    lands(&mut g, P, "Plains", 4, false);
    let cl = commander(&mut g, P, "Cloud, Midgar Mercenary");
    perm(&mut g, P, "Archon of Absolution");
    let helm = perm(&mut g, P, "Champion's Helm");
    let soff = perm(&mut g, P, "Sword of Feast and Famine");
    let _on = ai::force_hand(Some(true));
    assert!(equip_of(&g, helm).is_none(), "the hand AI equips them through cloud_special");
    let hand_ai = sim_core::impls::cloud::cloud_special(&mut g, P, false).unwrap();
    for n in ["Champion's Helm", "Sword of Feast and Famine"] {
        assert!(hand_ai.iter().any(|o| o.label == format!("equip {n} to Cloud, Midgar Mercenary")), "{n}");
    }
    let _off = ai::force_hand(Some(false));
    assert!(sim_core::impls::cloud::cloud_special(&mut g, P, false).unwrap().is_empty());
    assert_eq!(equip_of(&g, helm).map(|x| x.0), Some(cl), "the plan's equip.extra");
    let (t, u) = equip_of(&g, soff).expect("Sword of Feast and Famine");
    assert_eq!(t, cl);
    let want = 2.5 + 0.5 * 4.0 - 0.3 * 2.0 + 4.5; // cloud_special's utility
    assert!((u - want).abs() < 1e-9, "{u} vs {want}");
}

/// was keep_cloud_home: a bare Cloud stays home when he can be blocked; wearing a trigger Sword that no blocker kills
/// him through, he goes
#[test]
fn cloud_stays_home_bare_and_attacks_with_a_sword() {
    let mut g = seated("cloud", "t1");
    let cl = commander(&mut g, P, "Cloud, Midgar Mercenary");
    perm(&mut g, Q, "Ornithopter"); // 0/2 flier: blocks him, kills nothing
    let pick = |g: &mut Game| ai::attack_filter(g, P, vec![cl], Q);
    {
        let _on = ai::force_hand(Some(true));
        assert!(pick(&mut g).is_empty(), "the hand AI: nothing on him yet");
    }
    {
        let _off = ai::force_hand(Some(false));
        assert!(pick(&mut g).is_empty(), "the plan's stay_home");
    }
    let e = perm(&mut g, P, "Sword of Hearth and Home");
    g.perm_mut(e).attached = Some(cl);
    {
        let _on = ai::force_hand(Some(true));
        assert_eq!(pick(&mut g), vec![cl], "the hand AI: a Sword's trigger");
    }
    let _off = ai::force_hand(Some(false));
    assert_eq!(pick(&mut g), vec![cl], "the plan: the trigger Sword, no blocker that kills him");
    // a blocker that kills him: home again (both)
    perm(&mut g, Q, "Grave Titan"); // black: Hearth and Home gives no protection from it
    assert!(pick(&mut g).is_empty(), "the plan: a 6/6 blocker would kill the 3/2");
    let _on = ai::force_hand(Some(true));
    assert!(pick(&mut g).is_empty(), "the hand AI");
}

// ------------------------------------------------------------------ yshtola
/// was yshtola::x_option (gated on the hand AI): Exsanguinate is cast as a main-phase play for a big X
#[test]
fn yshtola_the_x_drain_is_offered() {
    let mut g = seated("yshtola", "t5");
    lands(&mut g, P, "Swamp", 10, false);
    let c = hand(&mut g, P, &["Exsanguinate"])[0];
    g.active = Some(P);
    let _on = ai::force_hand(Some(true));
    let on = hand_options(&mut g, c, Some(false));
    assert!(on.iter().any(|o| o.label == "Exsanguinate (X = 8)"), "the hand AI");
    let _off = ai::force_hand(Some(false));
    let off = hand_options(&mut g, c, Some(false));
    assert_eq!(off.iter().map(|o| &o.label).collect::<Vec<_>>(), on.iter().map(|o| &o.label).collect::<Vec<_>>());
}

/// was yshtola::helm: Champion's Helm goes on Y'shtola
#[test]
fn yshtola_the_helm_goes_on_her() {
    let mut g = seated("yshtola", "t5");
    lands(&mut g, P, "Swamp", 3, false);
    let y = commander(&mut g, P, "Y'shtola, Night's Blessed");
    perm(&mut g, P, "Restoration Angel");
    let helm = perm(&mut g, P, "Champion's Helm");
    g.active = Some(P);
    let _on = ai::force_hand(Some(true));
    let on = perm_options(&mut g, helm, Some(false));
    assert!(on.iter().any(|o| o.label == "equip Champion's Helm to Y'shtola, Night's Blessed"), "the hand AI");
    let _off = ai::force_hand(Some(false));
    assert!(perm_options(&mut g, helm, Some(false)).is_empty());
    assert_eq!(equip_of(&g, helm).map(|x| x.0), Some(y), "the plan's [equip]");
}

/// was trigger_bonus: with her out, a noncreature spell of mana value 3 or more is worth more (the more with Sanguine
/// Bond out), as the hand AI's priority says
#[test]
fn yshtola_her_trigger_raises_the_mv3_noncreature_spells() {
    let mut g = seated("yshtola", "t5");
    let c = hand(&mut g, P, &["Ill-Gotten Inheritance"])[0];
    let prio = |g: &Game| ai::decks::deck_prio(g, P, c);
    let (on0, off0) = {
        let _on = ai::force_hand(Some(true));
        let on = prio(&g);
        let _off = ai::force_hand(Some(false));
        (on, prio(&g))
    };
    assert_eq!(off0, 54, "the plan's prio rule");
    commander(&mut g, P, "Y'shtola, Night's Blessed");
    let (on1, off1) = {
        let _on = ai::force_hand(Some(true));
        let on = prio(&g);
        let _off = ai::force_hand(Some(false));
        (on, prio(&g))
    };
    assert!(on1 > on0 && off1 == off0 + 12, "hand {on0} -> {on1}, plan {off0} -> {off1}");
    perm(&mut g, P, "Sanguine Bond");
    let _off = ai::force_hand(Some(false));
    assert_eq!(prio(&g), off0 + 16, "with Sanguine Bond out");
    let wurm = g.db.id("Massacre Wurm").unwrap();
    assert_eq!(plan_data::prio_add(&g, P, wurm), 0.0, "a creature: no trigger");
}

/// was the combat.rs chump rule: she chump-blocks only to live
#[test]
fn yshtola_chumps_only_to_live() {
    let g = seated("yshtola", "t5");
    let _off = ai::force_hand(Some(false));
    assert!(plan_data::commander_chumps_to_live(&g, P));
    let g = seated("cloud", "t1");
    assert!(!plan_data::commander_chumps_to_live(&g, P), "a plan without the knob");
}

// ------------------------------------------------------------------ sheoldred
/// was sheoldred_special: Darksteel Plate onto Sheoldred ahead of the other plays, not onto a bigger creature
#[test]
fn sheoldred_the_plate_goes_on_her_first() {
    let mut g = seated("sheoldred", "t3");
    lands(&mut g, P, "Swamp", 4, false);
    let s = commander(&mut g, P, "Sheoldred, the Apocalypse");
    perm(&mut g, P, "Grave Titan");
    let e = perm(&mut g, P, "Darksteel Plate");
    g.active = Some(P);
    let _on = ai::force_hand(Some(true));
    let on = sim_core::impls::sheoldred::sheoldred_special(&mut g, P, false).unwrap();
    let hand_u = on.iter().find(|o| o.label == "equip Darksteel Plate to Sheoldred").expect("the hand AI").utility;
    let _off = ai::force_hand(Some(false));
    let (t, u) = equip_of(&g, e).expect("an equip option");
    assert_eq!(t, s, "the plan's [equip]");
    assert!((u - hand_u).abs() < 0.2, "about the hand AI's utility: {u} vs {hand_u}");
}

/// was the gate on otherworld Atlas's options: at the end of an opponent's turn it adds a counter
#[test]
fn sheoldred_the_atlas_is_used() {
    let mut g = seated("sheoldred", "t3");
    let a = perm(&mut g, P, "Otherworld Atlas");
    g.active = Some(Q);
    let _on = ai::force_hand(Some(true));
    let on: Vec<String> = perm_options(&mut g, a, None).into_iter().map(|o| o.label).collect();
    assert!(!on.is_empty(), "the hand AI");
    let _off = ai::force_hand(Some(false));
    let off: Vec<String> = perm_options(&mut g, a, None).into_iter().map(|o| o.label).collect();
    assert_eq!(off, on, "the plan's plays");
    // a deck whose plan doesn't list it: no play
    let mut g = seated("veyran", "t1");
    let a = perm(&mut g, P, "Otherworld Atlas");
    g.active = Some(Q);
    assert!(perm_options(&mut g, a, None).is_empty());
}

/// was the sheoldred arm of adnaus_cast_now: Ad Nauseam in her own second main phase
#[test]
fn sheoldred_ad_nauseam_in_her_second_main_phase() {
    let mut g = seated("sheoldred", "t3");
    g.player_mut(P).hand.clear();
    hand(&mut g, P, &["Ad Nauseam"]);
    g.active = Some(P);
    g.step = Step::Main2;
    let _on = ai::force_hand(Some(true));
    assert!(plans::adnaus_cast_now(&g, P), "the hand AI");
    let _off = ai::force_hand(Some(false));
    assert!(plans::adnaus_cast_now(&g, P), "the plan's adnaus_main2");
    let g2 = {
        let mut g = seated("yshtola", "t5");
        g.player_mut(P).hand.clear();
        g.active = Some(P);
        g.step = Step::Main2;
        g
    };
    assert!(!plans::adnaus_cast_now(&g2, P), "a plan without the knob");
}
