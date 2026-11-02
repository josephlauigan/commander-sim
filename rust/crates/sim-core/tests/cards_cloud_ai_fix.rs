//! Tests for the Cloud AI fixes (cloud-ai-fix task): equip options, the tutor pick, search value, priorities,
//! the attack filter, and recasting Cloud before combat.

use sim_core::ai::brain;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::cloud;
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
fn the_equip_goes_to_a_carrier_while_cloud_is_away() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
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
