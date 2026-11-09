//! Practice mode, playing lands from other zones: Crucible of Worlds (and Ramunap Excavator, Ancient Greenwarden ...)
//! lets the person play a land from their graveyard as their land drop; Oracle of Mul Daya lets them play one from the
//! top of their library.

mod play_common;
use play_common::*;
use sim_core::human::human::{abilities_of, human_main};
use sim_core::human::legal;
use sim_core::ids::CardId;

fn gy_push(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn on_top(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
    c
}

fn land_names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.perm(l).card()).name.to_string()).collect()
}

// ------------------------------------------------------------------ the graveyard
#[test]
fn crucible_plays_strip_mine_from_the_graveyard() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Crucible of Worlds");
    let sm = gy_push(&mut g, P0, "Strip Mine");
    assert_eq!(legal::check_land_gy(&g, P0, sm), None);
    let i = g.player(P0).gy.iter().position(|&c| c == sm).unwrap();
    let h = seat(&mut g, P0, vec![act(json!({"do": "land", "zone": "gy", "card": i})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(land_names(&g, P0).contains(&"Strip Mine".to_string()), "{:?}", h.told());
    assert!(!g.player(P0).gy.contains(&sm));
    assert_eq!(g.player(P0).lands_played, 1);
}

#[test]
fn crucible_still_one_land_a_turn() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Crucible of Worlds");
    let sm = gy_push(&mut g, P0, "Strip Mine");
    g.player_mut(P0).lands_played = 1;
    assert!(legal::check_land_gy(&g, P0, sm).unwrap().contains("already played a land"));
}

#[test]
fn no_graveyard_lands_without_crucible() {
    let mut g = table(&["thanos", "cloud"]);
    let sm = gy_push(&mut g, P0, "Strip Mine");
    assert!(legal::check_land_gy(&g, P0, sm).unwrap().contains("can't play lands from your graveyard"));
}

#[test]
fn an_opponents_crucible_does_not_help() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P1, "Crucible of Worlds");
    let sm = gy_push(&mut g, P0, "Strip Mine");
    assert!(legal::check_land_gy(&g, P0, sm).unwrap().contains("can't play lands from your graveyard"));
}

#[test]
fn crucible_leaves_muldrothas_land_unused() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Crucible of Worlds");
    perm(&mut g, P0, "Muldrotha, the Gravetide");
    let sm = gy_push(&mut g, P0, "Strip Mine");
    let i = g.player(P0).gy.iter().position(|&c| c == sm).unwrap();
    seat(&mut g, P0, vec![act(json!({"do": "land", "zone": "gy", "card": i})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(land_names(&g, P0).contains(&"Strip Mine".to_string()));
    let used = g.player(P0).muld_used.as_ref().map(|(_, v)| v.clone()).unwrap_or_default();
    assert!(!used.contains(&"L"), "Crucible let the land be played, so Muldrotha's land is still unused: {used:?}");
}

// ------------------------------------------------------------------ the top of the library
#[test]
fn oracle_of_mul_daya_plays_a_land_from_the_top() {
    let mut g = table(&["thanos", "cloud"]);
    let o = perm(&mut g, P0, "Oracle of Mul Daya");
    let f = on_top(&mut g, P0, "Forest");
    let labels: Vec<String> = abilities_of(&mut g, P0, o).unwrap().into_iter().map(|a| a.0).collect();
    assert!(labels.iter().any(|l| l.contains("play Forest from the top")), "{labels:?}");
    let i = g.player(P0).perms.iter().position(|&m| m == o).unwrap();
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), by_text("top of your library"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(land_names(&g, P0).contains(&"Forest".to_string()), "{:?}", h.told());
    assert!(!g.player(P0).library.contains(&f));
    assert_eq!(g.player(P0).lands_played, 1);
    // Mul Daya's extra land drop: a second land from the top is still allowed, a third is not
    let f2 = on_top(&mut g, P0, "Island");
    assert_eq!(legal::check_land_top(&g, P0, f2), None);
    g.player_mut(P0).lands_played = 2;
    assert!(legal::check_land_top(&g, P0, f2).unwrap().contains("already played a land"));
}

#[test]
fn mul_daya_offers_nothing_for_a_nonland_top() {
    let mut g = table(&["thanos", "cloud"]);
    let o = perm(&mut g, P0, "Oracle of Mul Daya");
    on_top(&mut g, P0, "Sol Ring");
    let labels: Vec<String> = abilities_of(&mut g, P0, o).unwrap().into_iter().map(|a| a.0).collect();
    assert!(!labels.iter().any(|l| l.contains("from the top")), "{labels:?}");
}

#[test]
fn no_top_lands_without_mul_daya() {
    let mut g = table(&["thanos", "cloud"]);
    let f = on_top(&mut g, P0, "Forest");
    assert!(legal::check_land_top(&g, P0, f).unwrap().contains("can't play lands from the top"));
}
