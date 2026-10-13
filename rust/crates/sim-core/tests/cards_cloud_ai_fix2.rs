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
