//! Probes for the C4 audit of the Cloud AI (cloud-audit-c4 task). Diagnosis only: they print the options and the
//! plays of one main phase on positions rebuilt from single-game traces (`--deck cloud --pool tN --profile loose --ai
//! lookahead --trace G`) and assert nothing, except where marked. Run with
//! `cargo test --release -p sim-core --test cards_cloud_audit_c4 -- --nocapture --test-threads 1`.

use sim_core::ai::{brain, search};
use sim_core::ids::{PermId, PlayerId};
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game_with(seed: u64) -> Game {
    let mut g = table_with(&["cloud", "seph"], seed, 40);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 4; // Cloud's priority is 0 on a first turn
    set_ai(&mut g, AiMode::Lookahead);
    g
}

fn game() -> Game {
    game_with(1)
}

fn cloud_board(g: &mut Game, lands_n: usize) -> PermId {
    lands(g, P0, "Plains", lands_n, false);
    let cl = perm(g, P0, CLOUD);
    g.player_mut(P0).cmd_in_zone = false; // testkit::perm does not take him out of the command zone
    g.perm_mut(cl).sick = false;
    cl
}

/// Cloud in hand (bounced; rule 903.9b: no tax) instead of on the battlefield
fn cloud_in_hand(g: &mut Game) {
    let c = g.player(P0).cmd;
    g.player_mut(P0).cmd_in_zone = false;
    g.player_mut(P0).hand.push(c);
}

fn dump(g: &mut Game, title: &str) {
    let opts = brain::main_options(g, P0, false).unwrap();
    println!("== {title}");
    let mut v: Vec<(String, f64)> = opts.iter().map(|o| (o.label.clone(), o.utility)).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (l, u) in &v {
        println!("   heuristic {u:6.2}  {l}");
    }
    search::LAST_SCORES.with(|l| l.borrow_mut().clear());
    let pick = search::choose(g, P0, false, &opts);
    for (l, s) in search::LAST_SCORES.with(|l| l.borrow().clone()) {
        println!("   search    {s:6.2}  {l}");
    }
    println!("   -> {:?}", pick.map(|i| opts[i].label.clone()));
}

/// play the whole first main phase with the real loop (search at every step) and print what happened
fn phase(g: &mut Game, title: &str) {
    println!("== {title}");
    g.log = Some(vec![]);
    brain::main(g, P0, false).unwrap();
    for l in g.log.take().unwrap_or_default() {
        println!("   {l}");
    }
    let hand: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    let free = sim_core::engine::mana::total_mana(g, P0, false);
    println!("   hand left {hand:?}; mana still available {free}");
}

/// the pick of the search over 24 seeds of one position
fn tally(title: &str, build: impl Fn(&mut Game)) {
    use std::collections::BTreeMap;
    let mut picks: BTreeMap<String, usize> = BTreeMap::new();
    for seed in 1..=24u64 {
        let mut g = game_with(seed);
        build(&mut g);
        let opts = brain::main_options(&mut g, P0, false).unwrap();
        let pick = search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone()).unwrap_or("none".into());
        *picks.entry(pick).or_default() += 1;
    }
    println!("== {title}: picks over 24 seeds");
    for (l, n) in &picks {
        println!("   picked {n:2}x  {l}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// 1. "stop (hold mana)" with a 1-mana Equipment and 11 lands (t2 trace 1, R12/R13)
fn build_hold_mana(g: &mut Game) {
    let cl = cloud_board(g, 11);
    let s = perm(g, P0, "Sword of Feast and Famine");
    g.perm_mut(s).attached = Some(cl);
    perm(g, P0, "Bronze Guardian");
    hand(g, P0, &["Champion's Helm", "Recommission", "Loyal Warhound"]);
}

#[test]
fn probe_1_hold_mana_with_cheap_equipment() {
    let mut g = game();
    build_hold_mana(&mut g);
    dump(&mut g, "1: 11 lands, Cloud+SoFF, Bronze Guardian; hand Helm, Recommission, Warhound");
    tally("1", build_hold_mana);
    let mut g = game();
    build_hold_mana(&mut g);
    phase(&mut g, "1: the whole main phase");
}

// ---------------------------------------------------------------------------------------------------------------
// 2. Cloud bounced to hand, 7 lands: the heuristic puts him first (74-99%), the search picks a rock (t2 trace 4, R5)
fn build_bounced(g: &mut Game) {
    lands(g, P0, "Plains", 7, false);
    let p = perm(g, P0, "Patriot, Shield Wielder");
    let s = perm(g, P0, "Sword of Hearth and Home");
    g.perm_mut(s).attached = Some(p);
    cloud_in_hand(g);
    hand(g, P0, &["Coalition Relic", "Mind Stone", "Sword of Fire and Ice", "Colossus Hammer"]);
}

#[test]
fn probe_2_bounced_cloud_not_recast() {
    let mut g = game();
    build_bounced(&mut g);
    dump(&mut g, "2: Cloud in hand, 7 lands, Patriot+SoHH; hand Relic, Stone, SoFI, Hammer");
    tally("2", build_bounced);
    let mut g = game();
    build_bounced(&mut g);
    phase(&mut g, "2: the whole main phase");
}

// ---------------------------------------------------------------------------------------------------------------
// 3. Turn 2, Cloud castable (WW) with Hero's Blade in hand: the search chose the Blade (t5 trace 2, R2)
fn build_t2(g: &mut Game) {
    lands(g, P0, "Plains", 2, false);
    hand(g, P0, &["Hero's Blade", "Arcane Signet", "Kor Outfitter"]);
}

#[test]
fn probe_3_turn_two_cloud_or_blade() {
    // the commander is in the command zone at the start, as in a real game
    let mut g = game();
    build_t2(&mut g);
    // build_t2 took Cloud from the library (it is the commander: in the command zone), so no copy is in hand
    dump(&mut g, "3: 2 Plains; hand Blade, Signet, Outfitter; Cloud in the command zone");
    tally("3", build_t2);
}

// ---------------------------------------------------------------------------------------------------------------
// 4. Two Swords cast on 6 mana, nothing equipped (t2 trace 3, R6)
fn build_two_swords(g: &mut Game) {
    cloud_board(g, 5);
    perm(g, P0, "Fellwar Stone");
    hand(g, P0, &["Sword of Fire and Ice", "Sword of Hearth and Home", "Sword of Feast and Famine", "Champion's Helm"]);
}

#[test]
fn probe_4_cast_two_swords_or_cast_and_equip() {
    let mut g = game();
    build_two_swords(&mut g);
    dump(&mut g, "4: 5 lands + Fellwar Stone, bare Cloud (can attack); hand SoFI, SoHH, SoFF, Helm");
    tally("4", build_two_swords);
    let mut g = game();
    build_two_swords(&mut g);
    phase(&mut g, "4: the whole main phase");
}

// ---------------------------------------------------------------------------------------------------------------
// 5. Mask of Memory ping-pong between two carriers when Cloud is dead (t3 trace 3, R16-R19)
fn build_pingpong(g: &mut Game) {
    lands(g, P0, "Plains", 9, false);
    perm(g, P0, "Bronze Guardian");
    perm(g, P0, "Kor Duelist");
    perm(g, P0, "Mask of Memory");
}

#[test]
fn probe_5_mask_pingpong() {
    let mut g = game();
    build_pingpong(&mut g);
    phase(&mut g, "5: Cloud dead, Bronze Guardian + Kor Duelist, loose Mask of Memory, 9 lands");
    let mut g = game();
    build_pingpong(&mut g);
    brain::main(&mut g, P0, false).unwrap();
    // second pass of the same phase: does it equip again, to the other creature?
    g.log = Some(vec![]);
    brain::main(&mut g, P0, false).unwrap();
    let n = g.log.take().unwrap_or_default().iter().filter(|l| l.contains("equips Mask of Memory")).count();
    println!("   equips of the Mask on a second pass of the same phase: {n}");
}

// ---------------------------------------------------------------------------------------------------------------
// 6. Plaza of Heroes: {3},{T}, exile: a legendary creature gains hexproof and indestructible (not modeled?)
#[test]
fn probe_6_plaza_has_no_protection_ability() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    let _plaza = lands(&mut g, P0, "Plaza of Heroes", 1, false);
    cloud_board(&mut g, 0);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let labels: Vec<&str> = opts.iter().map(|o| o.label.as_str()).collect();
    println!("== 6: options with Plaza of Heroes + Cloud out: {labels:?}");
    let has = labels.iter().any(|l| l.contains("Plaza"));
    println!("   an option that uses Plaza of Heroes: {has}");
}

// ---------------------------------------------------------------------------------------------------------------
// 7. Edict (Grave Pact) with Cloud as the only creature: which permanent dies (silent in the trace, t1 trace 6, R8)
#[test]
fn probe_7_edict_takes_the_only_creature() {
    let mut g = game();
    let cl = cloud_board(&mut g, 4);
    let before = g.perm(cl).on_bf;
    sim_core::engine::zones::edict(&mut g, P0, false).unwrap();
    println!("== 7: Cloud alone, edict: still on the battlefield {} (was {before})", g.perm(cl).on_bf);
    println!("   the trace prints no line for a sacrifice, so a game where Cloud vanishes with no 'is removed' line is an edict");
}

#[test]
fn probe_2b_cloud_in_hand_alone() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 7, false);
    cloud_in_hand(&mut g);
    println!("cmd_in_zone {} hand {}", g.player(P0).cmd_in_zone, g.player(P0).hand.len());
    dump(&mut g, "2b: Cloud in hand alone, 7 lands");
    let mut g = game();
    lands(&mut g, P0, "Plains", 7, false);
    let cl = perm(&mut g, P0, CLOUD);
    // bounce him the way a Cyclonic Rift does
    sim_core::engine::zones::bounce(&mut g, cl).unwrap();
    println!("after a real bounce: cmd_in_zone {} hand {}", g.player(P0).cmd_in_zone, g.player(P0).hand.len());
    dump(&mut g, "2b: Cloud bounced for real, 7 lands");
}
