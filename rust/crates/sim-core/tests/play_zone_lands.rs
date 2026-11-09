//! Practice mode, playing lands from other zones: Crucible of Worlds (and Ramunap Excavator, Ancient Greenwarden ...)
//! lets the person play a land from their graveyard as their land drop; Oracle of Mul Daya and Courser of Kruphix let
//! them play one from the top of their library; a warped card (Perigee Beckoner) is cast from exile on a later turn.

mod play_common;
use play_common::*;
use sim_core::human::human::{abilities_of, human_main};
use sim_core::human::legal;
use sim_core::ids::CardId;
use sim_core::impls::generic_paths::warped_in_exile;
use sim_core::state::{DataKey, Val};

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
fn play_top(g: &mut Game, p: PlayerId) -> Option<String> {
    sim_core::human::human::apply(g, p, &json!({"do": "land", "zone": "top"})).unwrap()
}

#[test]
fn oracle_of_mul_daya_plays_a_land_from_the_top() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Oracle of Mul Daya");
    let f = on_top(&mut g, P0, "Forest");
    assert!(legal::top_visible(&g, P0), "Mul Daya reveals the top card");
    let h = seat(&mut g, P0, vec![act(json!({"do": "land", "zone": "top"})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(land_names(&g, P0).contains(&"Forest".to_string()), "{:?}", h.told());
    assert!(!g.player(P0).library.contains(&f));
    assert_eq!(g.player(P0).lands_played, 1);
    // Mul Daya's extra land drop: a second land from the top is still allowed, a third is not
    let f2 = on_top(&mut g, P0, "Island");
    assert_eq!(legal::check_land_top(&g, P0, f2), None);
    g.player_mut(P0).lands_played = 2;
    assert!(legal::check_land_top(&g, P0, f2).unwrap().contains("already played a land"));
    assert!(play_top(&mut g, P0).unwrap().contains("already played a land"));
}

#[test]
fn mul_daya_is_not_an_activated_ability() {
    // playing a land is a special action, not an activated ability: an opposing Cursed Totem doesn't stop it, and
    // the Oracle offers nothing under "Activate" (no extra priority stop in combat)
    let mut g = table(&["thanos", "cloud"]);
    let o = perm(&mut g, P0, "Oracle of Mul Daya");
    perm(&mut g, P1, "Cursed Totem");
    on_top(&mut g, P0, "Forest");
    let labels: Vec<String> = abilities_of(&mut g, P0, o).unwrap().into_iter().map(|a| a.0).collect();
    assert!(!labels.iter().any(|l| l.contains("from the top")), "{labels:?}");
    assert_eq!(play_top(&mut g, P0), None);
    assert!(land_names(&g, P0).contains(&"Forest".to_string()));
}

#[test]
fn courser_of_kruphix_plays_a_land_from_the_top() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Courser of Kruphix");
    on_top(&mut g, P0, "Forest");
    assert_eq!(play_top(&mut g, P0), None);
    assert!(land_names(&g, P0).contains(&"Forest".to_string()));
}

#[test]
fn a_nonland_top_is_seen_but_not_played() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Oracle of Mul Daya");
    let s = on_top(&mut g, P0, "Sol Ring");
    assert!(legal::top_visible(&g, P0));
    assert!(legal::check_land_top(&g, P0, s).unwrap().contains("isn't a land"));
    assert!(play_top(&mut g, P0).is_some());
    assert_eq!(g.player(P0).library.last(), Some(&s));
}

#[test]
fn no_top_lands_without_mul_daya() {
    let mut g = table(&["thanos", "cloud"]);
    let f = on_top(&mut g, P0, "Forest");
    assert!(!legal::top_visible(&g, P0), "the top card stays hidden");
    assert!(legal::check_land_top(&g, P0, f).unwrap().contains("can't play lands from the top"));
}

#[test]
fn top_lands_only_at_sorcery_speed() {
    let mut g = table(&["thanos", "cloud"]);
    perm(&mut g, P0, "Oracle of Mul Daya");
    let f = on_top(&mut g, P0, "Forest");
    g.active = Some(P1); // another player's turn
    assert!(legal::check_land_top(&g, P0, f).is_some());
}

// ------------------------------------------------------------------ warp: the exiled card on a later turn
/// Perigee Beckoner cast for its warp cost, then exiled by its end-step trigger
fn warped_beckoner(g: &mut Game) -> CardId {
    let m = perm(g, P0, "Perigee Beckoner");
    g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().end_step.unwrap();
    f(g, m, P0).unwrap();
    assert!(g.player(P0).exile.contains(&c), "the warped Beckoner is exiled at the end step");
    c
}

#[test]
fn a_warped_beckoner_waits_for_a_later_turn() {
    let mut g = table(&["sheoldred", "cloud"]);
    let c = warped_beckoner(&mut g);
    assert!(warped_in_exile(&g, P0).is_empty(), "not on the turn it was exiled");
    assert!(legal::check_cast_warp(&g, P0, c).unwrap().contains("can't cast"));
}

#[test]
fn a_warped_beckoner_is_cast_from_exile_for_its_mana_cost() {
    let mut g = table(&["sheoldred", "cloud"]);
    let c = warped_beckoner(&mut g);
    g.round += 1; // a later turn of P0's
    assert_eq!(warped_in_exile(&g, P0), vec![c]);
    assert!(legal::check_cast_warp(&g, P0, c).unwrap().contains("Can't cast Perigee Beckoner from exile"), "no mana");
    pool(&mut g, P0, &[('C', 4), ('B', 1)]);
    assert_eq!(legal::check_cast_warp(&g, P0, c), None);
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "zone": "warp", "card": c.0})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_perm(&g, P0, "Perigee Beckoner"), "{:?}", h.told());
    assert!(!g.player(P0).exile.contains(&c));
    assert_eq!(pool_total(&g, P0), 0, "paid {{4}}{{B}}");
    assert!(warped_in_exile(&g, P0).is_empty());
}

#[test]
fn an_exiled_beckoner_that_was_not_warped_stays_put() {
    let mut g = table(&["sheoldred", "cloud"]);
    let c = take(&mut g, P0, "Perigee Beckoner");
    g.player_mut(P0).exile.push(c);
    g.round += 1;
    assert!(warped_in_exile(&g, P0).is_empty());
}
