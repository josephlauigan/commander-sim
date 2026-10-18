//! Rograkh/Silas turbo plan (impls/t6_plan_rograkh_silas.rs): the spec's mulligan and decision scenarios.

use sim_core::ids::PlayerId;
use sim_core::impls::t6_plans;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const RS: &str = "rograkh-silas-turbo-breach";
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";

fn tbl() -> Game {
    let mut g = table(&[RS, KRAUM]);
    g.active = Some(P0);
    g
}

fn keeps(names: &[&str]) -> bool {
    let mut g = tbl();
    hand(&mut g, P0, names);
    t6_plans::keep(&g, P0).unwrap()
}

#[test]
fn keeps_fast_mana_and_a_tutor_with_one_land() {
    // Natux: fast mana + a payoff or tutor; two lands are not required
    assert!(keeps(&["Underground Sea", "Lotus Petal", "Sol Ring", "Demonic Tutor", "Daze", "Flusterstorm", "Dark Ritual"]));
}

#[test]
fn spec_scenario_10_four_lands_sol_ring_tutor_is_a_keep_and_without_the_tutor_a_ship() {
    let lands = ["Underground Sea", "Watery Grave", "Badlands", "Volcanic Island"];
    let mut h = lands.to_vec();
    h.extend(["Rhystic Study", "Sol Ring", "Demonic Tutor"]);
    assert!(keeps(&h));
    let mut h = lands.to_vec();
    h.extend(["Rhystic Study", "Chain of Vapor", "Daze"]);
    assert!(!keeps(&h));
}

#[test]
fn spec_scenario_11_ad_nauseam_hand_is_a_keep() {
    assert!(keeps(&["Ad Nauseam", "Brain Freeze", "Chrome Mox", "Command Tower", "Culling the Weak", "Pact of Negation", "Wishclaw Talisman"]));
}

#[test]
fn spec_scenario_15_engines_without_fast_mana_or_payoff_are_shipped() {
    assert!(!keeps(&["Mystic Remora", "Rhystic Study", "Underground Sea", "Watery Grave", "Badlands", "Fierce Guardianship", "Pyroblast"]));
}

#[test]
fn all_land_and_no_land_hands_are_shipped() {
    assert!(!keeps(&["Underground Sea", "Watery Grave", "Badlands", "Volcanic Island", "Blood Crypt", "Command Tower", "Scalding Tarn"]));
    assert!(!keeps(&["Lotus Petal", "Sol Ring", "Dark Ritual", "Demonic Tutor", "Ad Nauseam", "Force of Will", "Daze"]));
}

#[test]
fn rograkh_is_cast_first_and_silas_is_not() {
    let mut g = tbl();
    let rog = g.player(P0).cmd;
    let silas = g.player(P0).cmd2.expect("partner");
    let (rog, silas) = if &*g.db.get(rog).name == "Rograkh, Son of Rohgahh" { (rog, silas) } else { (silas, rog) };
    g.player_mut(P0).turns = 1;
    assert_eq!(t6_plans::prio(&g, P0, rog), Some(100.0));
    assert_eq!(t6_plans::prio(&g, P0, silas), Some(0.0));
}

#[test]
fn mox_amber_waits_for_a_legend_and_rituals_are_held() {
    let mut g = tbl();
    let ids = hand(&mut g, P0, &["Mox Amber", "Dark Ritual", "Sol Ring"]);
    assert_eq!(t6_plans::prio(&g, P0, ids[0]), Some(0.0));
    assert_eq!(t6_plans::prio(&g, P0, ids[1]), Some(0.0));
    assert!(t6_plans::prio(&g, P0, ids[2]).unwrap() > 90.0);
    perm(&mut g, P0, "Rograkh, Son of Rohgahh");
    assert!(t6_plans::prio(&g, P0, ids[0]).unwrap() > 90.0);
}

#[test]
fn tutors_look_for_the_missing_half_of_the_oracle_line() {
    let mut g = tbl();
    hand(&mut g, P0, &["Thassa's Oracle"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(&*g.db.get(w[0]).name, "Demonic Consultation");
    let mut g = tbl();
    hand(&mut g, P0, &["Tainted Pact"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(&*g.db.get(w[0]).name, "Thassa's Oracle");
}
