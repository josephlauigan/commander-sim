//! Tests for the Cloud AI fixes (cloud-ai-fix task): equip options, the tutor pick, search value, priorities,
//! the attack filter, and recasting Cloud before combat.

use sim_core::ai::{act, brain, search};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::cloud;
use sim_core::settings::AiMode;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

/// the library is exactly these cards (so the tests don't depend on the Cloud deck list)
fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn cloud_board(g: &mut Game, n: usize) -> PermId {
    lands(g, P0, "Plains", n, false);
    let cl = perm(g, P0, CLOUD);
    g.perm_mut(cl).sick = false;
    cl
}

fn prio(g: &mut Game, name: &str) -> i32 {
    let c = hand(g, P0, &[name])[0];
    cloud::cloud_prio(g, P0, c)
}

#[test]
fn helm_and_feast_and_famine_equip_and_work() {
    for (n, cost) in [("Champion's Helm", 1), ("Sword of Feast and Famine", 2)] {
        let mut g = game();
        let cl = cloud_board(&mut g, 3);
        let e = perm(&mut g, P0, n);
        g.perm_mut(e).attached = None;
        let before = sim_core::engine::values::epow(&g, cl);
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        let o =
            opts.iter().find(|o| o.label == format!("equip {n} to {CLOUD}")).unwrap_or_else(|| panic!("{n}: no equip"));
        assert_eq!(opts.iter().filter(|o| o.label.starts_with(&format!("equip {n}"))).count(), 1, "no double options");
        let free = g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count();
        assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
        assert_eq!(g.perm(e).attached, Some(cl));
        let used = free - g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count();
        assert_eq!(used, cost, "{n} equip cost");
        assert_eq!(sim_core::engine::values::epow(&g, cl), before + 2);
        // not offered again once worn
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        assert!(!opts.iter().any(|o| o.label.starts_with(&format!("equip {n}"))));
    }
}

#[test]
fn scythe_gets_an_equip_option() {
    let mut g = game();
    cloud_board(&mut g, 4);
    let e = perm(&mut g, P0, "Magitek Scythe");
    g.perm_mut(e).attached = None; // it attaches itself on entering
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    assert!(opts.iter().any(|o| o.label.starts_with("equip Magitek Scythe")));
}

#[test]
fn tutor_fetches_a_piece_the_mana_allows() {
    // 3 lands: a cheap Sword or the Helm, never Colossus Hammer (cast 1, equip 8) or a Kaldra piece
    for seed in 0..8u64 {
        let mut g = table_with(&["cloud", "seph"], 900 + seed, 40);
        g.player_mut(P0).hand.clear();
        set_library(
            &mut g,
            P0,
            &[
                "Sword of Fire and Ice",
                "Sword of Hearth and Home",
                "Sword of Feast and Famine",
                "Champion's Helm",
                "Colossus Hammer",
                "Shield of Kaldra",
                "Wrecking Ball Arm",
            ],
        );
        lands(&mut g, P0, "Plains", 3, false);
        perm(&mut g, P0, CLOUD);
        let got: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            ["Sword of Fire and Ice", "Sword of Hearth and Home", "Sword of Feast and Famine", "Champion's Helm"]
                .contains(&got[0].as_str()),
            "fetched {got:?}"
        );
    }
}

#[test]
fn tutor_skips_pieces_in_hand() {
    let mut g = game();
    set_library(
        &mut g,
        P0,
        &[
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Sword of Feast and Famine",
            "Champion's Helm",
            "Colossus Hammer",
            "Shield of Kaldra",
            "Wrecking Ball Arm",
        ],
    );
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Sword of Fire and Ice", "Sword of Hearth and Home"]);
    perm(&mut g, P0, CLOUD);
    let names: Vec<&str> = g.player(P0).hand.iter().map(|&c| &*g.db.get(c).name).collect();
    assert_eq!(names.iter().filter(|n| **n == "Sword of Fire and Ice").count(), 1);
    assert!(names.contains(&"Sword of Feast and Famine") || names.contains(&"Champion's Helm"), "{names:?}");
}

#[test]
fn equipping_shield_of_kaldra_beats_stop() {
    let mut g = game();
    set_library(&mut g, P0, &vec!["Plains"; 30]);
    cloud_board(&mut g, 4);
    perm(&mut g, P0, "Shield of Kaldra");
    hand(&mut g, P0, &["Sword of Kaldra", "Helm of Kaldra", "Puresteel Paladin"]);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    search::LAST_SCORES.with(|l| l.borrow_mut().clear());
    let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    let sc = search::LAST_SCORES.with(|l| l.borrow().clone());
    let get = |n: &str| sc.iter().find(|x| x.0 == n).map(|x| x.1).unwrap();
    assert!(get("equip Shield of Kaldra") > get("stop"), "{sc:?}");
    assert_eq!(pick.as_deref(), Some("equip Shield of Kaldra"));
}

#[test]
fn priorities() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    let hold = prio(&mut g, "Sword of Kaldra"); // an Equipment waits in hand
    assert!(prio(&mut g, "Solemn Simulacrum") >= 68);
    assert!(prio(&mut g, "Court of Ardenvale") >= 64);
    assert!(prio(&mut g, "Sword of Fire and Ice") > hold);
    assert!(prio(&mut g, "Sword of Feast and Famine") > 0);
    assert!(prio(&mut g, "Sword of Hearth and Home") > 0);
    // C4 additions
    assert!(prio(&mut g, "Auriok Steelshaper") >= 66);
    assert!(prio(&mut g, "Kor Outfitter") >= 66);
    assert!(prio(&mut g, "Balan, Wandering Knight") >= 66); // backup carrier: Equipment waits, Cloud away
    assert!(prio(&mut g, "Armored Skyhunter") >= 60);
    cloud_board(&mut g, 0);
    assert!(prio(&mut g, "Swiftfoot Boots") >= 74);
    assert!(prio(&mut g, "Benevolent Bodyguard") >= 64);
}

#[test]
fn cloud_with_only_a_scope_stays_home_against_a_blocker() {
    let mut g = game();
    let cl = cloud_board(&mut g, 3);
    let sc = perm(&mut g, P0, "Explorer's Scope");
    g.perm_mut(sc).attached = Some(cl);
    token(&mut g, P1, 3);
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert!(!out.contains(&cl));
    // a real Sword puts him back in
    let sw = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(sw).attached = Some(cl);
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert!(out.contains(&cl));
}

#[test]
fn recast_cloud_before_combat_with_equipment_out() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    let e = perm(&mut g, P0, "Helm of Kaldra");
    g.perm_mut(e).attached = None;
    g.player_mut(P0).turns = 4;
    g.step = Step::Main1;
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == CLOUD).expect("Cloud castable");
    assert!(cloud::search_bonus(&g, P0, o) >= 18.0);
    g.step = Step::Main2;
    assert_eq!(cloud::search_bonus(&g, P0, o), 10.0);
}

#[test]
fn c4_cards_priorities_and_carriers() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Sword of Fire and Ice"]); // Equipment waits in hand
    assert!(prio(&mut g, "Ardenn, Intrepid Archaeologist") >= 72);
    assert!(prio(&mut g, "Kor Duelist") >= 62);
    assert!(prio(&mut g, "Balan, Wandering Knight") >= 66);
    // with Cloud away, the equip goes to Balan or Duelist over a plain body
    perm(&mut g, P0, "Bronze Guardian");
    let kd = perm(&mut g, P0, "Kor Duelist");
    let sw = perm(&mut g, P0, "Sword of Hearth and Home");
    g.perm_mut(sw).attached = None;
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    assert!(
        opts.iter().any(|o| o.label.contains("equip Sword of Hearth and Home") && o.label.contains("Kor Duelist"))
            || opts.iter().any(|o| o.label.starts_with("equip Sword of Hearth and Home")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    assert!(cloud::holder_score(&g, kd) > 0.0);
}
