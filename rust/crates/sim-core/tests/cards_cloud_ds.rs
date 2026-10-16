//! Cloud + double strike: combat damage and Sword triggers per combat-damage step.

use sim_core::engine::combat;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Cloud + Sword of Fire and Ice (optionally double strike), attacks the open player 1; returns (life lost, cards drawn, commander damage)
fn swing(ds: bool) -> (i32, usize, i32) {
    let mut g: Game = table(&["cloud", "seph"]);
    for p in [P0, P1] {
        let cs: Vec<_> = (0..20).map(|_| card(&g, "Plains")).collect();
        g.player_mut(p).library = cs;
    }
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    g.perm_mut(cl).is_cmd = true;
    let sw = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(sw).attached = Some(cl);
    if ds {
        g.perm_mut(cl).eot_kw.push("double strike");
    }
    g.player_mut(P1).life = 100;
    let hand0 = g.player(P0).hand.len();
    let lib0 = g.player(P0).library.len();
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    let _ = hand0;
    let drawn = lib0 - g.player(P0).library.len();
    let cmd = sim_core::engine::values::cmd_dmg_by(&g, P1, P0, cl);
    (100 - g.player(P1).life, drawn, cmd)
}

#[test]
fn single_strike_baseline() {
    let (life, drawn, cmd) = swing(false);
    println!("baseline: life lost {life}, cards {drawn}, cmd {cmd}");
}

#[test]
fn double_strike_cloud_sword_triggers_four_times() {
    let (life, drawn, cmd) = swing(true);
    println!("ds: life lost {life}, cards {drawn}, cmd {cmd}");
    let (l1, d1, c1) = swing(false);
    // power 2+2 = 4 per hit; SoFI 2 damage per trigger
    assert_eq!(cmd, 2 * c1, "commander damage doubles");
    assert_eq!(drawn, 2 * d1, "Sword draws: 4 triggers vs 2 ({d1})");
    assert!(life > l1);
}

fn setup() -> (Game, sim_core::ids::PermId) {
    let mut g: Game = table(&["cloud", "seph"]);
    for p in [P0, P1] {
        let cs: Vec<_> = (0..20).map(|_| card(&g, "Plains")).collect();
        g.player_mut(p).library = cs;
    }
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    g.perm_mut(cl).is_cmd = true;
    g.player_mut(P1).life = 100;
    (g, cl)
}

#[test]
fn fireshrieker_gives_double_strike() {
    let (mut g, cl) = setup();
    let fs = perm(&mut g, P0, "Fireshrieker");
    assert!(!combat::double_strike(&g, cl));
    g.perm_mut(fs).attached = Some(cl);
    assert!(combat::double_strike(&g, cl));
}

#[test]
fn duelist_heritage_gives_cloud_double_strike_on_attack() {
    use sim_core::hooks::{Call, Event};
    let (mut g, cl) = setup();
    perm(&mut g, P0, "Duelist's Heritage");
    let other = perm(&mut g, P0, "Mirran Crusader");
    g.perm_mut(other).sick = false;
    g.perm_mut(other).eot_pt.0 += 3;
    sim_core::engine::hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![cl, other], d: P1 })
        .unwrap();
    // Mirran Crusader already has double strike; Cloud is the target
    assert!(combat::double_strike(&g, cl));
}

#[test]
fn akroma_will_is_cast_for_a_sworded_cloud_and_grants_both_modes() {
    let (mut g, cl) = setup();
    let sw = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(sw).attached = Some(cl);
    lands(&mut g, P0, "Plains", 4, false);
    hand(&mut g, P0, &["Akroma's Will"]);
    let before = g.player(P0).hand.len();
    let mut assign = vec![];
    let c = g.player(P0).hand[0];
    sim_core::cardcode::defend_hooks(&mut g, P0, &mut vec![cl], P1, &mut assign).unwrap();
    assert_eq!(g.player(P0).hand.len(), before - 1, "the Will was cast");
    assert!(combat::double_strike(&g, cl));
    assert!(sim_core::dsl::has_kw(&g, cl, "flying") && sim_core::dsl::has_kw(&g, cl, "lifelink"));
    assert!(sim_core::engine::values::protected_from(&g, cl, sim_core::cards::Colors::from_letters("R")));
    let _ = c;
    let lib0 = g.player(P0).library.len();
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    assert_eq!(lib0 - g.player(P0).library.len(), 4, "four Sword draws");
}
