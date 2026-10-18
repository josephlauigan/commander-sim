//! Engine feature `legend` (t6-e-legend): the legend rule (CR 704.5j) and Blood Moon / Magus of the Moon as a real
//! layer effect (nonbasic lands become plain Mountains), plus Damping Sphere's colourless override.

use sim_core::cardcode;
use sim_core::engine::life::check_state;
use sim_core::engine::mana::{can_pay, mana_units, pay, total_mana};
use sim_core::engine::zones::enter_token_copy_with;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{CopyExc, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["godo", "seph"])
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).name == name).collect()
}

// ============================================================ the legend rule
#[test]
fn two_legendary_permanents_with_one_name_leave_one() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let cd = g.perm(godo).cd.unwrap();
    let copy = enter_token_copy_with(&mut g, P0, cd, None).unwrap().unwrap();
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).attached = None; // the fetched Equipment attached itself: take it off
    }
    g.perm_mut(godo).tapped = true; // the first one has attacked: the AI keeps the untapped copy
    g.bf_ver += 1;
    check_state(&mut g).unwrap();
    let left = named(&g, P0, "Godo, Bandit Warlord");
    assert_eq!(left.len(), 1, "the legend rule leaves one");
    assert_eq!(left[0], copy, "the untapped one stays");
}

#[test]
fn the_legend_rule_keeps_the_equipped_one() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let cd = g.perm(godo).cd.unwrap();
    let copy = enter_token_copy_with(&mut g, P0, cd, None).unwrap().unwrap();
    let helm = named(&g, P0, "Helm of the Host")[0];
    g.perm_mut(helm).attached = Some(godo);
    check_state(&mut g).unwrap();
    assert_eq!(named(&g, P0, "Godo, Bandit Warlord"), vec![godo], "the equipped original stays, the token goes");
    assert!(!g.perm(copy).on_bf);
}

#[test]
fn a_copy_that_is_not_legendary_stays() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let cd = g.perm(godo).cd.unwrap();
    let exc = CopyExc { not_legendary: true, ..CopyExc::default() };
    enter_token_copy_with(&mut g, P0, cd, Some(exc)).unwrap().unwrap();
    check_state(&mut g).unwrap();
    assert_eq!(named(&g, P0, "Godo, Bandit Warlord").len(), 2, "a Helm of the Host token is not legendary");
}

#[test]
fn legend_rule_is_per_controller() {
    let mut g = tbl();
    perm(&mut g, P0, "Godo, Bandit Warlord");
    perm(&mut g, P1, "Godo, Bandit Warlord");
    check_state(&mut g).unwrap();
    assert_eq!(named(&g, P0, "Godo, Bandit Warlord").len(), 1);
    assert_eq!(named(&g, P1, "Godo, Bandit Warlord").len(), 1);
}

#[test]
fn a_godo_copy_dies_at_once_but_its_enter_trigger_still_fetches_an_equipment() {
    let mut g = tbl();
    hand(&mut g, P0, &["Twinflame"]); // the copier in hand: Hammer first
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(!named(&g, P0, "Hammer of Nazahn").is_empty() && named(&g, P0, "Helm of the Host").is_empty());
    let cd = g.perm(godo).cd.unwrap();
    let copy = enter_token_copy_with(&mut g, P0, cd, None).unwrap().unwrap();
    check_state(&mut g).unwrap();
    assert_eq!(named(&g, P0, "Godo, Bandit Warlord").len(), 1, "the copy and the original: one is put away");
    assert!(g.perm(godo).on_bf != g.perm(copy).on_bf);
    assert!(!named(&g, P0, "Helm of the Host").is_empty(), "the copy's trigger fetched the next Equipment");
}

// ============================================================ Blood Moon
#[test]
fn blood_moon_ancient_tomb_deals_no_damage_and_makes_one_red() {
    let mut g = tbl();
    lands(&mut g, P1, "Ancient Tomb", 1, false);
    perm(&mut g, P0, "Blood Moon");
    let life = g.player(P1).life;
    assert_eq!(total_mana(&g, P1, false), 1);
    assert!(can_pay(&g, P1, 0, "R", false));
    pay(&mut g, P1, 0, "R", false).unwrap();
    assert_eq!(g.player(P1).life, life, "a Mountain has no Ancient Tomb damage");
}

#[test]
fn blood_moon_ancient_tomb_at_two_life_is_usable_again() {
    let mut g = tbl();
    lands(&mut g, P1, "Ancient Tomb", 1, false);
    g.player_mut(P1).life = 2;
    assert_eq!(total_mana(&g, P1, false), 0, "its 2 damage would kill: not used");
    perm(&mut g, P0, "Blood Moon");
    assert_eq!(total_mana(&g, P1, false), 1);
}

#[test]
fn magus_of_the_moon_does_the_same() {
    let mut g = tbl();
    lands(&mut g, P1, "City of Traitors", 1, false);
    assert_eq!(total_mana(&g, P1, false), 2);
    perm(&mut g, P0, "Magus of the Moon");
    assert_eq!(total_mana(&g, P1, false), 1);
}

#[test]
fn blood_moon_urza_s_saga_loses_its_chapters_and_stays() {
    let mut g = tbl();
    let saga = lands(&mut g, P1, "Urza's Saga", 1, false)[0];
    g.player_mut(P1).sagas.push((saga, 1));
    perm(&mut g, P0, "Blood Moon");
    cardcode::land_etb(&mut g, P1, saga).unwrap();
    for _ in 0..3 {
        sim_core::impls::common::saga_step(&mut g, P1).unwrap();
    }
    assert!(g.player(P1).lands.contains(&saga), "no longer a Saga (the subtype is gone): not sacrificed");
    assert_eq!(g.player(P1).sagas, [(saga, 1)], "no lore counters are added");
    assert!(cardcode::land_options(&mut g, P1, Some(true)).unwrap().is_empty(), "no Construct ability");
    assert!(can_pay(&g, P1, 0, "R", false));
}

#[test]
fn blood_moon_removes_utility_and_enters_abilities() {
    let mut g = tbl();
    lands(&mut g, P1, "Mishra's Workshop", 1, false);
    perm(&mut g, P0, "Blood Moon");
    assert!(can_pay(&g, P1, 0, "R", false), "the Workshop is a plain Mountain");
    let tapped = sim_core::engine::turn::land_enters_tapped(&g, P1, card(&g, "Cavern of Souls"));
    assert!(!tapped);
    let bog = lands(&mut g, P1, "Bojuka Bog", 1, true)[0];
    cardcode::land_etb(&mut g, P1, bog).unwrap();
    assert!(g.player(P0).gy.is_empty() && g.player(P1).gy.is_empty());
}

#[test]
fn blood_moon_leaves_basic_lands_alone_and_ends_with_the_moon() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    let tomb = lands(&mut g, P1, "Ancient Tomb", 1, false)[0];
    let moon = perm(&mut g, P0, "Blood Moon");
    assert!(can_pay(&g, P1, 0, "U", false));
    assert!(sim_core::engine::mana::moon_land(&g, tomb));
    g.perm_mut(moon).on_bf = false;
    assert!(!sim_core::engine::mana::moon_land(&g, tomb));
    assert_eq!(total_mana(&g, P1, false), 3);
}

// ============================================================ Damping Sphere
#[test]
fn damping_sphere_makes_a_two_mana_land_one_colourless() {
    let mut g = tbl();
    lands(&mut g, P1, "Izzet Boilerworks", 1, false);
    assert!(can_pay(&g, P1, 0, "UR", false));
    perm(&mut g, P0, "Damping Sphere");
    assert_eq!(total_mana(&g, P1, false), 1);
    assert!(!can_pay(&g, P1, 0, "U", false) && !can_pay(&g, P1, 0, "R", false), "{{C}} only, never {{U}} or {{R}}");
    assert!(can_pay(&g, P1, 1, "", false));
    let u = mana_units(&g, P1, false);
    assert!(u[0].cols.is_empty());
}

#[test]
fn damping_sphere_leaves_one_mana_lands_alone() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    perm(&mut g, P0, "Damping Sphere");
    assert!(can_pay(&g, P1, 0, "U", false));
}
