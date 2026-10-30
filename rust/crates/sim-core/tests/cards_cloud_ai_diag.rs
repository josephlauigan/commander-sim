//! Diagnostic probes for the Cloud AI (cloud-ai-equip task): print the main-phase options with their heuristic
//! utility and the look-ahead scores on hand-built positions. Run with `--nocapture`. They assert nothing except
//! where marked; failing demonstrations are #[ignore].

use sim_core::ai::{brain, search};
use sim_core::ids::{PermId, PlayerId};
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

fn dump(g: &mut Game, title: &str) -> Vec<(String, f64)> {
    let opts = brain::main_options(g, P0, false).unwrap();
    println!("== {title}");
    let mut v: Vec<(String, f64)> = opts.iter().map(|o| (o.label.clone(), o.utility)).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (l, u) in &v {
        println!("   heuristic {u:6.2}  {l}");
    }
    search::LAST_SCORES.with(|l| l.borrow_mut().clear());
    let pick = search::choose(g, P0, false, &opts);
    let sc = search::LAST_SCORES.with(|l| l.borrow().clone());
    for (l, s) in &sc {
        println!("   search    {s:6.2}  {l}");
    }
    println!("   -> {:?}", pick.map(|i| opts[i].label.clone()));
    v
}

fn cloud_board(g: &mut Game, lands_n: usize) -> PermId {
    lands(g, P0, "Plains", lands_n, false);
    let cl = perm(g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    cl
}

#[test]
fn probe_a_loose_arm_five_lands() {
    // Cloud out, Wrecking Ball Arm loose, 5 lands, Equipment and bodies in hand
    let mut g = game();
    cloud_board(&mut g, 5);
    perm(&mut g, P0, "Wrecking Ball Arm");
    hand(
        &mut g,
        P0,
        &["Champion's Helm", "Solemn Simulacrum", "Court of Ardenvale", "Sword of Hearth and Home", "Mask of Memory"],
    );
    dump(&mut g, "A: 5 lands, Cloud + loose Arm; hand Helm, Solemn, Court, SoHH, Mask");
}

#[test]
fn probe_b_four_lands_arm_and_kaldra() {
    let mut g = game();
    cloud_board(&mut g, 4);
    perm(&mut g, P0, "Wrecking Ball Arm");
    hand(
        &mut g,
        P0,
        &["Champion's Helm", "Sword of Kaldra", "Shield of Kaldra", "Solemn Simulacrum", "Court of Ardenvale"],
    );
    dump(&mut g, "B: 4 lands, Cloud + loose Arm; hand Helm, SoK, ShoK, Solemn, Court");
}

#[test]
fn probe_c_no_cloud_bronze_guardian() {
    // Cloud dead (in the command zone), Bronze Guardian and Exalted Angel out, a loose Arm and Sword of Kaldra
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    perm(&mut g, P0, "Bronze Guardian");
    perm(&mut g, P0, "Exalted Angel");
    perm(&mut g, P0, "Wrecking Ball Arm");
    perm(&mut g, P0, "Sword of Hearth and Home");
    hand(&mut g, P0, &["Solemn Simulacrum", "Court of Ardenvale"]);
    dump(&mut g, "C: no Cloud (command zone), 6 lands, Guardian+Angel, loose Arm + SoHH");
}

#[test]
fn probe_d_cloud_back_equipment_on_other() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    let bg = perm(&mut g, P0, "Bronze Guardian");
    let a = perm(&mut g, P0, "Wrecking Ball Arm");
    let s = perm(&mut g, P0, "Sword of Hearth and Home");
    g.perm_mut(a).attached = Some(bg);
    g.perm_mut(s).attached = Some(bg);
    cloud_board(&mut g, 0);
    dump(&mut g, "D: Cloud back, Arm + SoHH on Bronze Guardian, 6 lands");
}

#[test]
fn probe_e_game4_round5() {
    // trace game 4 (t1, loose), R5: 4 lands, Cloud wearing Sword of Fire and Ice + Patriot; the search chose "stop"
    let mut g = game();
    let cl = cloud_board(&mut g, 4);
    let s = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(s).attached = Some(cl);
    perm(&mut g, P0, "Patriot, Shield Wielder");
    hand(
        &mut g,
        P0,
        &["Puresteel Paladin", "Sword of Kaldra", "Mask of Memory", "Court of Ardenvale", "Solemn Simulacrum"],
    );
    dump(&mut g, "E: 4 lands, Cloud+SoFI, Patriot; hand Paladin, SoK, Mask, Court, Solemn");
}

#[test]
fn probe_f_only_helm_and_solemn() {
    let mut g = game();
    cloud_board(&mut g, 4);
    hand(&mut g, P0, &["Champion's Helm", "Solemn Simulacrum"]);
    dump(&mut g, "F: 4 lands, bare Cloud; hand Helm, Solemn");
    let mut g = game();
    cloud_board(&mut g, 4);
    hand(&mut g, P0, &["Court of Ardenvale", "Shield of Kaldra"]);
    dump(&mut g, "F2: 4 lands, bare Cloud; hand Court, Shield of Kaldra");
}

#[test]
fn probe_g_pval_of_the_cards() {
    // what the look-ahead's position score (search::strength: board pval + 0.35 per power + 0.9 per card in hand)
    // counts for each permanent of the deck once it is on the battlefield, against the 0.9 a card in hand is worth
    use sim_core::engine::values::{epow, pval};
    let mut g = game();
    let cl = cloud_board(&mut g, 4);
    println!("Cloud: pval {:.2} epow {}", pval(&g, cl), epow(&g, cl));
    for n in [
        "Champion's Helm",
        "Wrecking Ball Arm",
        "Sword of Kaldra",
        "Shield of Kaldra",
        "Helm of Kaldra",
        "Mask of Memory",
        "Sword of Hearth and Home",
        "Sword of Fire and Ice",
        "Conformer Shuriken",
        "Court of Ardenvale",
        "Solemn Simulacrum",
        "Veteran's Powerblade",
        "Colossus Hammer",
        "Explorer's Scope",
        "Trailblazer's Boots",
        "Bronze Guardian",
        "Exalted Angel",
    ] {
        let m = perm(&mut g, P0, n);
        println!("{n}: pval {:.2}", pval(&g, m));
    }
}

/// the look-ahead's pick over 40 shuffles (seeds) of the same position: if the scores were a signal the pick would
/// be stable; it is a coin flip between the cast options and "stop"
fn tally(title: &str, build: impl Fn(&mut Game)) {
    use std::collections::BTreeMap;
    let mut picks: BTreeMap<String, usize> = BTreeMap::new();
    let mut sums: BTreeMap<String, (f64, f64, usize)> = BTreeMap::new();
    for seed in 1..=40u64 {
        let mut g = table_with(&["cloud", "seph"], seed, 40);
        g.player_mut(P0).hand.clear();
        set_ai(&mut g, AiMode::Lookahead);
        build(&mut g);
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone()).unwrap_or("none".into());
        *picks.entry(pick).or_default() += 1;
        for (l, s) in search::LAST_SCORES.with(|l| l.borrow().clone()) {
            let e = sums.entry(l).or_default();
            e.0 += s;
            e.1 += s * s;
            e.2 += 1;
        }
    }
    println!("== {title}: picks over 40 seeds");
    for (l, n) in &picks {
        println!("   picked {n:2}x  {l}");
    }
    for (l, (s, s2, n)) in &sums {
        let m = s / *n as f64;
        println!("   score {l}: mean {m:.2} sd {:.2}", (s2 / *n as f64 - m * m).max(0.0).sqrt());
    }
}

#[test]
fn probe_h_pick_stability() {
    tally("F (Helm + Solemn, bare Cloud, 4 lands)", |g| {
        cloud_board(g, 4);
        hand(g, P0, &["Champion's Helm", "Solemn Simulacrum"]);
    });
    tally("B (loose Arm, Sword/Shield of Kaldra, Helm, Solemn, Court; 4 lands)", |g| {
        cloud_board(g, 4);
        perm(g, P0, "Wrecking Ball Arm");
        hand(
            g,
            P0,
            &["Champion's Helm", "Sword of Kaldra", "Shield of Kaldra", "Solemn Simulacrum", "Court of Ardenvale"],
        );
    });
}

/// the position score right after one play (and after the rest of the phase, heuristically), before any opponent
/// turn: shows what the look-ahead's evaluation sees in each option
fn immediate(title: &str, build: impl Fn(&mut Game), labels: &[&str]) {
    use sim_core::ai::act;
    println!("== {title}");
    for &lbl in labels {
        let mut g = table_with(&["cloud", "seph"], 1, 40);
        g.player_mut(P0).hand.clear();
        set_ai(&mut g, AiMode::Lookahead);
        build(&mut g);
        let base = search::evaluate(&g, P0);
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        let Some(o) = opts.iter().find(|o| o.label == lbl) else {
            println!("   {lbl}: not offered");
            continue;
        };
        if let Some(a) = &o.act {
            act::perform(&mut g, P0, a).unwrap();
        }
        let one = search::evaluate(&g, P0);
        let s1 = search::strength(&g, P0);
        g.in_search = true;
        brain::main(&mut g, P0, false).unwrap();
        let all = search::evaluate(&g, P0);
        let s2 = search::strength(&g, P0);
        let hand: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
        println!(
            "   {lbl:28} eval before {base:6.2} | after play {one:6.2} (strength {s1:5.2}) | after phase {all:6.2} (strength {s2:5.2}) hand left {hand:?}"
        );
    }
}

#[test]
fn probe_i_immediate_scores() {
    immediate(
        "F: Helm, Solemn, bare Cloud, 4 lands",
        |g| {
            cloud_board(g, 4);
            hand(g, P0, &["Champion's Helm", "Solemn Simulacrum"]);
        },
        &["stop", "Champion's Helm", "Solemn Simulacrum", "Wrecking Ball Arm"],
    );
    immediate(
        "B: loose Arm, SoK, ShoK, Helm, Solemn, Court; 4 lands",
        |g| {
            cloud_board(g, 4);
            perm(g, P0, "Wrecking Ball Arm");
            hand(
                g,
                P0,
                &["Champion's Helm", "Sword of Kaldra", "Shield of Kaldra", "Solemn Simulacrum", "Court of Ardenvale"],
            );
        },
        &[
            "stop",
            "equip Wrecking Ball Arm",
            "Sword of Kaldra",
            "Champion's Helm",
            "Solemn Simulacrum",
            "Court of Ardenvale",
        ],
    );
}

#[test]
fn probe_j_options_after_helm() {
    use sim_core::ai::act;
    let mut g = game();
    cloud_board(&mut g, 4);
    hand(&mut g, P0, &["Champion's Helm", "Solemn Simulacrum"]);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == "Champion's Helm").unwrap();
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    println!("lands untapped: {}", g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count());
    for o in brain::main_options(&mut g, P0, false).unwrap() {
        println!("   after Helm: {:6.2} {}", o.utility, o.label);
    }
    println!("{:?}", g.db.get(sim_core::testkit::card(&g, "Champion's Helm")).abilities);
}

#[test]
fn probe_k_which_equipment_can_be_equipped() {
    // every Equipment of the deck on the battlefield, loose, 9 lands: which get an "equip" option?
    let names = [
        "Champion's Helm",
        "Wrecking Ball Arm",
        "Sword of Kaldra",
        "Shield of Kaldra",
        "Helm of Kaldra",
        "Mask of Memory",
        "Sword of Hearth and Home",
        "Sword of Fire and Ice",
        "Sword of Feast and Famine",
        "Conformer Shuriken",
        "Veteran's Powerblade",
        "Colossus Hammer",
        "Explorer's Scope",
        "Trailblazer's Boots",
        "White Mage's Staff",
        "Nim Deathmantle",
        "Hero's Blade",
        "Magitek Scythe",
    ];
    for n in names {
        let mut g = game();
        cloud_board(&mut g, 9);
        perm(&mut g, P0, n);
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        let eq: Vec<String> = opts
            .iter()
            .filter(|o| o.label.starts_with("equip"))
            .map(|o| format!("{} ({:.1})", o.label, o.utility))
            .collect();
        println!("{n:28} -> {eq:?}");
    }
}

#[test]
fn probe_l_cast_another_kaldra_or_equip_the_one_out() {
    // game 1 R6/R7: Shield of Kaldra loose on the battlefield, 4 lands, Sword and Helm of Kaldra in hand
    let mut g = game();
    cloud_board(&mut g, 4);
    perm(&mut g, P0, "Shield of Kaldra");
    hand(&mut g, P0, &["Sword of Kaldra", "Helm of Kaldra", "Puresteel Paladin"]);
    dump(&mut g, "L: 4 lands, Cloud + loose Shield of Kaldra; hand SoK, HoK, Paladin");
    let mut g = game();
    cloud_board(&mut g, 3);
    perm(&mut g, P0, "Sword of Fire and Ice");
    hand(&mut g, P0, &["Ennis, Debate Moderator", "Cid, Freeflier Pilot", "Armory Automaton"]);
    dump(&mut g, "L2: 3 lands, Cloud + loose SoFI; hand Ennis, Cid, Armory Automaton");
}

#[test]
fn probe_m_immediate_shield() {
    immediate(
        "L: 4 lands, Cloud + loose Shield of Kaldra; hand SoK, HoK, Paladin",
        |g| {
            cloud_board(g, 4);
            perm(g, P0, "Shield of Kaldra");
            hand(g, P0, &["Sword of Kaldra", "Helm of Kaldra", "Puresteel Paladin"]);
        },
        &["stop", "equip Shield of Kaldra", "Sword of Kaldra", "Puresteel Paladin"],
    );
}

/// (fixed in cloud-ai-fix) these three Equipment of the deck get no equip option at all, so once
/// cast they never attach (Champion's Helm and Sword of Feast and Famine only have the Sauron deck's hand-written
/// equip plan; Magitek Scythe attaches on entering only, its Equip {2} is not modeled).
#[test]
fn bug_helm_feast_and_famine_and_scythe_have_no_equip_option() {
    for n in ["Champion's Helm", "Sword of Feast and Famine", "Magitek Scythe"] {
        let mut g = game();
        cloud_board(&mut g, 9);
        let e = perm(&mut g, P0, n);
        g.perm_mut(e).attached = None; // Scythe attaches on entering; the probe wants it loose
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        assert!(opts.iter().any(|o| o.label.starts_with("equip") && o.label.contains(n)), "{n}: no equip option");
    }
}
