//! Tests for the cloud-ai-fix2 task: one section per fix.

use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

const LIB: [&str; 7] = [
    "Sword of Fire and Ice",
    "Sword of Hearth and Home",
    "Sword of Feast and Famine",
    "Champion's Helm",
    "Colossus Hammer",
    "Sword of Kaldra",
    "Wrecking Ball Arm",
];

fn fetched(g: &Game) -> Vec<String> {
    g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

// ---- fix 1: engine-aware tutor
#[test]
fn tutor_takes_a_big_piece_with_puresteel_metalcraft() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, "Puresteel Paladin");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

#[test]
fn tutor_takes_a_cheap_sword_without_an_engine() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, CLOUD);
    assert_eq!(fetched(&g), vec!["Sword of Fire and Ice".to_string()]);
}

#[test]
fn tutor_takes_a_big_piece_with_ardenn_out() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

#[test]
fn enlightened_tutor_uses_the_same_pick() {
    let mut g = game();
    set_library(
        &mut g,
        P0,
        &["Sword of Fire and Ice", "Sol Ring", "Sword of Feast and Famine", "Swords to Plowshares"],
    );
    lands(&mut g, P0, "Plains", 3, false);
    let want = sim_core::ai::tutor_pick(&g, P0, "ae").map(|c| g.db.get(c).name.to_string());
    assert_eq!(want.as_deref(), Some("Sword of Fire and Ice"));
    perm(&mut g, P0, "Sword of Fire and Ice");
    let want = sim_core::ai::tutor_pick(&g, P0, "ae").map(|c| g.db.get(c).name.to_string());
    assert_eq!(want.as_deref(), Some("Sword of Feast and Famine"), "skips the piece on the battlefield");
}

// ---- fix 2: don't strand equipment
fn cast_labels(g: &mut Game) -> Vec<String> {
    sim_core::ai::brain::main_options(g, P0, false).unwrap().iter().map(|o| o.label.clone()).collect()
}

#[test]
fn equipment_is_held_without_a_carrier() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    // Cloud (2) + the Sword (3) don't fit in 3 mana: the Sword waits
    let l = cast_labels(&mut g);
    assert!(!l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    assert!(l.iter().any(|x| x.contains("Mind Stone")), "{l:?}");
}

#[test]
fn equipment_is_cast_with_a_carrier_or_as_the_only_use() {
    // Cloud and the Sword fit together
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    // a creature is out
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Kor Duelist");
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    // nothing else to do with the mana
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Sword of Fire and Ice"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
}

// ---- fix 3: protection
const P1: PlayerId = PlayerId(1);

#[test]
fn plaza_of_heroes_protects_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let pl = lands(&mut g, P0, "Plaza of Heroes", 1, false)[0];
    let cl = perm(&mut g, P0, CLOUD);
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), None).unwrap();
    assert!(saved);
    assert!(sim_core::engine::values::untargetable(&g, cl));
    assert!(sim_core::engine::values::indestructible(&g, cl));
    assert!(!g.land(pl).on_bf && !g.player(P0).lands.contains(&pl));
    assert!(g.player(P0).exile.contains(&g.land(pl).cd));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "paid {{3}} with the other lands");
}

#[test]
fn plaza_needs_three_other_mana() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let pl = lands(&mut g, P0, "Plaza of Heroes", 1, false)[0];
    let cl = perm(&mut g, P0, CLOUD);
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), None).unwrap();
    assert!(!saved);
    assert!(g.land(pl).on_bf);
}

#[test]
fn formation_answers_a_destroy_wipe() {
    for n in ["Unbreakable Formation", "Rootborn Defenses"] {
        let mut g = game();
        lands(&mut g, P0, "Plains", 3, false);
        let cl = perm(&mut g, P0, CLOUD);
        let k = perm(&mut g, P0, "Kor Duelist");
        let e = perm(&mut g, P0, "Sword of Fire and Ice");
        g.perm_mut(e).attached = Some(k);
        hand(&mut g, P0, &[n]);
        let r = sim_core::ai::wipe_response(&mut g, P0, "destroy".into(), P1).unwrap();
        assert_eq!(r.as_deref(), Some("indes"), "{n}");
        assert!(sim_core::engine::values::indestructible(&g, cl), "{n}");
        assert!(!g.player(P0).hand.iter().any(|&c| *g.db.get(c).name == *n), "{n} was cast");
        // an exile wipe is not answered by indestructible
        let mut g = game();
        lands(&mut g, P0, "Plains", 3, false);
        perm(&mut g, P0, CLOUD);
        hand(&mut g, P0, &[n]);
        assert!(sim_core::ai::wipe_response(&mut g, P0, "exile".into(), P1).unwrap().is_none());
    }
}

#[test]
fn giver_of_runes_answers_removal_on_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let cl = perm(&mut g, P0, CLOUD);
    g.perm_mut(cl).is_cmd = true;
    let gv = perm(&mut g, P0, "Giver of Runes");
    let sp = card(&g, "Swords to Plowshares");
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), Some(sp)).unwrap();
    assert!(saved, "Giver of Runes should answer");
    assert!(g.perm(gv).tapped);
}
