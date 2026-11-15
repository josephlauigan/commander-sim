//! The page prints a playable graveyard card as "Name (how)": `how` names the card that allows the cast, not an
//! internal code.

use practice::view::seam::graveyard_playable;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn hows(g: &sim_core::state::Game) -> Vec<(String, String)> {
    let v = graveyard_playable(g, P0);
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| (x["name"].as_str().unwrap().to_string(), x["how"].as_str().unwrap().to_string()))
        .collect()
}

#[test]
fn a_card_cast_with_yawgmoths_will_says_yawgmoths_will() {
    let mut g = table(&["seph", "cloud"]);
    let c = card(&g, "Entomb");
    g.player_mut(P0).gy.push(c);
    assert!(hows(&g).is_empty(), "nothing allows it yet");
    g.player_mut(P0).yawg = true;
    g.player_mut(P0).yawg_gy = vec![c];
    assert_eq!(hows(&g), vec![("Entomb".to_string(), "Yawgmoth's Will".to_string())]);
}

#[test]
fn a_flashback_card_still_says_flashback() {
    let mut g = table(&["seph", "cloud"]);
    let c = card(&g, "Deep Analysis");
    g.player_mut(P0).gy.push(c);
    assert_eq!(hows(&g), vec![("Deep Analysis".to_string(), "flashback".to_string())]);
}
