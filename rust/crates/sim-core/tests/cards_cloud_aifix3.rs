//! Tests for the cloud-aifix3 task: one section per fix.

use sim_core::ids::PlayerId;
use sim_core::impls::cloud;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P1).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

// ---- F1: an equipped Cloud attacks
fn wear(g: &mut Game, gear: &str) -> sim_core::ids::PermId {
    let cl = perm(g, P0, CLOUD);
    let e = perm(g, P0, gear);
    g.perm_mut(e).attached = Some(cl);
    cl
}

#[test]
fn sworded_cloud_attacks_into_a_small_blocker() {
    let mut g = game();
    let cl = wear(&mut g, "Sword of Fire and Ice");
    token(&mut g, P1, 1);
    assert!(cloud::must_attack(&g, P0));
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert_eq!(out, vec![cl], "the Sword's trigger beats sitting home");
    let plan = sim_core::ai::search::choose_attack(&mut g, P0);
    assert!(plan.is_some_and(|x| x.1 != "none"), "{plan:?}");
}

#[test]
fn sworded_cloud_stays_home_into_a_killer_blocker() {
    let mut g = game();
    let cl = wear(&mut g, "Sword of Hearth and Home");
    token(&mut g, P1, 9);
    assert!(!cloud::must_attack(&g, P0));
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn bare_cloud_is_not_forced() {
    let mut g = game();
    let cl = perm(&mut g, P0, CLOUD);
    token(&mut g, P1, 1);
    assert!(!cloud::must_attack(&g, P0));
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn other_decks_are_not_forced() {
    let mut g = table(&["seph", "cloud"]);
    assert!(!cloud::must_attack(&g, P0));
    let _ = &mut g;
}

// ---- F2: Stormfront Riders spares an equipped Cloud
#[test]
fn riders_does_not_return_an_equipped_cloud() {
    let mut g = game();
    let cl = wear(&mut g, "Sword of Fire and Ice");
    let duelist = perm(&mut g, P0, "Kor Duelist");
    perm(&mut g, P0, "Stormfront Riders");
    assert!(g.perm(cl).on_bf, "Cloud stays");
    assert!(!equipment_on_cloud(&g).is_empty(), "with his Sword");
    assert!(!g.perm(duelist).on_bf, "the Duelist goes home instead");
}

#[test]
fn riders_still_returns_a_bare_cloud_and_other_decks_creatures() {
    let mut g = game();
    let cl = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Stormfront Riders");
    assert!(!g.perm(cl).on_bf, "a bare Cloud may be returned (rebuy of the tutor)");
}

fn equipment_on_cloud(g: &Game) -> Vec<sim_core::ids::PermId> {
    cloud::equipment_on(g, cloud::cloud_of(g, P0).unwrap())
}

// ---- F4: Sword of Feast and Famine in the tutor order
const SWORDS: [&str; 3] = ["Sword of Fire and Ice", "Sword of Hearth and Home", "Sword of Feast and Famine"];

fn pick(g: &Game) -> String {
    sim_core::ai::tutor_pick(g, P0, "ae").map(|c| g.db.get(c).name.to_string()).unwrap()
}

fn swords_library(g: &mut Game) {
    let cs: Vec<_> = SWORDS.iter().map(|n| card(g, n)).collect();
    g.player_mut(P0).library = cs;
    lands(g, P0, "Plains", 4, false);
}

#[test]
fn feast_and_famine_comes_before_hearth_and_home_once_a_sword_is_out() {
    let mut g = game();
    swords_library(&mut g);
    assert_eq!(pick(&g), "Sword of Fire and Ice", "nothing out: the usual order");
    let mut g = game();
    swords_library(&mut g);
    hand(&mut g, P0, &["Sword of Fire and Ice"]);
    assert_eq!(pick(&g), "Sword of Feast and Famine");
}

#[test]
fn feast_and_famine_first_with_a_clogged_hand() {
    let mut g = game();
    swords_library(&mut g);
    hand(&mut g, P0, &["Sol Ring", "Mind Stone", "Arcane Signet", "Swords to Plowshares"]);
    assert_eq!(pick(&g), "Sword of Feast and Famine");
}

// ---- F8: Unfinished Business is cast
fn labels(g: &mut Game) -> Vec<String> {
    sim_core::ai::brain::main_options(g, P0, false).unwrap().iter().map(|o| o.label.clone()).collect()
}

fn to_gy(g: &mut Game, names: &[&str]) {
    for n in names {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
}

#[test]
fn unfinished_business_is_cast_on_a_creature_and_its_equipment() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    to_gy(&mut g, &["Kor Duelist", "Sword of Fire and Ice"]);
    hand(&mut g, P0, &["Unfinished Business"]);
    g.player_mut(P0).turns = 5;
    let l = labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Unfinished Business")), "{l:?}");
}

#[test]
fn unfinished_business_waits_without_a_creature_card() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    to_gy(&mut g, &["Sword of Fire and Ice"]);
    hand(&mut g, P0, &["Unfinished Business"]);
    g.player_mut(P0).turns = 5;
    let l = labels(&mut g);
    assert!(!l.iter().any(|x| x.contains("Unfinished Business")), "{l:?}");
}

#[test]
fn recommission_is_cast_on_a_cheap_graveyard_creature() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    to_gy(&mut g, &["Kor Duelist"]);
    hand(&mut g, P0, &["Recommission"]);
    g.player_mut(P0).turns = 5;
    let l = labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Recommission")), "{l:?}");
}
