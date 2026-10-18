//! Tier 6 lands batch (impls/t6_lands.rs) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::{mana, turn};
use sim_core::ids::PlayerId;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn pod() -> Game {
    table(&["lathril-golgari-elves", "veyran"])
}

fn play_land(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    turn::play_land_card(g, p, c, "plays").unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect()
}

// ------------------------------------------------------------------ City of Brass
#[test]
fn city_of_brass_deals_one_damage_each_time_it_is_tapped() {
    let mut g = pod();
    lands(&mut g, P0, "City of Brass", 1, false);
    let life = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).life, life - 1, "1 damage when it becomes tapped");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

// ------------------------------------------------------------------ City of Traitors
#[test]
fn city_of_traitors_is_sacrificed_when_you_play_another_land() {
    let mut g = pod();
    lands(&mut g, P0, "City of Traitors", 1, false);
    assert_eq!(mana::total_mana(&g, P0, false), 2, "{{T}}: {{C}}{{C}}");
    play_land(&mut g, P0, "Forest");
    assert_eq!(names(&g, P0), ["Forest"]);
    assert!(g.player(P0).gy.contains(&card(&g, "City of Traitors")));
    // the City itself being played does not sacrifice it, another land afterwards does; the opponent's lands do not
    play_land(&mut g, P0, "City of Traitors");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
    play_land(&mut g, P1, "Island");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
}

// ------------------------------------------------------------------ Gemstone Caverns
#[test]
fn gemstone_caverns_makes_colourless_unless_it_has_a_luck_counter() {
    let mut g = pod();
    let l = lands(&mut g, P0, "Gemstone Caverns", 1, false)[0];
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 1);
    assert!(u[0].cols.is_empty(), "{{T}}: Add {{C}}");
    g.land_mut(l).data.set(DataKey::Counters, Val::Int(1)); // a luck counter
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u[0].cols, g.player(P0).ident, "with a luck counter: one mana of any colour");
    assert_eq!(u[0].amt, 1);
}
