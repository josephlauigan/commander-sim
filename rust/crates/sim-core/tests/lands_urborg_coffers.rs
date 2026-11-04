//! Urborg, Tomb of Yawgmoth ("Each land is a Swamp in addition to its other land types": so each land can also tap
//! for {B}) and Cabal Coffers ("{2}, {T}: Add {B} for each Swamp you control", modelled as its net mana). Seat 0 plays
//! JD's Kefka (Grixis, with Urborg), seat 1 the pool's mono-black Tergrid (Coffers, Urborg, Crypt Ghast).

use sim_core::engine::mana::{self, Source};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["kefka", "tergrid-mono-black-disruption"])
}

/// the unit land l contributes to p's mana: (amount, can make {B})
fn unit(g: &Game, p: PlayerId, l: PermId) -> (u32, bool) {
    let u = mana::mana_units(g, p, false).into_iter().find(|u| matches!(u.src, Source::Land(x) if x == l));
    u.map_or((0, false), |u| (u.amt, u.cols.has('B')))
}

#[test]
fn urborg_lets_every_land_tap_for_black() {
    let mut g = game();
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    let mountain = lands(&mut g, P0, "Mountain", 1, false)[0];
    assert_eq!(unit(&g, P0, island), (1, false));
    assert_eq!(unit(&g, P0, mountain), (1, false));
    lands(&mut g, P0, "Urborg, Tomb of Yawgmoth", 1, false);
    assert_eq!(unit(&g, P0, island), (1, true), "an Island is a Swamp too");
    assert_eq!(unit(&g, P0, mountain), (1, true));
    assert!(mana::land_colors_now(&g, P0, island).has('B'), "Fellwar Stone's read agrees");
    // any player's Urborg: the opponent's lands too
    let opp_island = lands(&mut g, P1, "Island", 1, false)[0];
    assert!(unit(&g, P1, opp_island).1);
}

#[test]
fn urborg_does_nothing_under_blood_moon_and_two_mana_lands_keep_their_ability() {
    let mut g = game();
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    let tomb = lands(&mut g, P0, "Ancient Tomb", 1, false)[0];
    lands(&mut g, P0, "Urborg, Tomb of Yawgmoth", 1, false);
    assert_eq!(unit(&g, P0, tomb), (2, false), "Ancient Tomb stays {{C}}{{C}}, never {{B}}{{B}}");
    perm(&mut g, P0, "Blood Moon");
    assert_eq!(unit(&g, P0, island), (1, false), "Urborg is a plain Mountain: no Swamps");
}

#[test]
fn urborg_lands_get_crypt_ghasts_extra_and_keep_black() {
    let mut g = game();
    let island = lands(&mut g, P1, "Island", 1, false)[0];
    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
    perm(&mut g, P1, "Crypt Ghast");
    let (amt, b) = unit(&g, P1, island);
    assert!(b, "still taps for {{B}} with Ghast's extra mana");
    assert_eq!(amt, 2, "an Island is a Swamp: Ghast adds {{B}}");
}

#[test]
fn cabal_coffers_nets_black_per_swamp_minus_two() {
    for (swamps, want) in [(0, 0), (1, 0), (2, 0), (3, 1), (4, 2), (6, 4)] {
        let mut g = game();
        let c = lands(&mut g, P1, "Cabal Coffers", 1, false)[0];
        lands(&mut g, P1, "Swamp", swamps, false);
        assert_eq!(unit(&g, P1, c).0, want, "{swamps} Swamps");
    }
}

#[test]
fn cabal_coffers_counts_every_land_under_urborg_and_needs_two_lands_to_pay() {
    let mut g = game();
    let c = lands(&mut g, P1, "Cabal Coffers", 1, false)[0];
    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
    lands(&mut g, P1, "Myriad Landscape", 1, false);
    lands(&mut g, P1, "Swamp", 1, false);
    // four Swamps (Coffers, Urborg and Landscape are Swamps too): {B}x4 for the {2}
    assert_eq!(unit(&g, P1, c).0, 2);
    let total: u32 = mana::mana_units(&g, P1, false).iter().map(|u| u.amt).sum();
    assert_eq!(total, 5, "three lands' mana, two of it paying for Coffers' four");
    // the other lands already tapped: nothing pays the {2}
    let mut g = game();
    let c = lands(&mut g, P1, "Cabal Coffers", 1, false)[0];
    lands(&mut g, P1, "Swamp", 4, true);
    assert_eq!(unit(&g, P1, c).0, 0);
}
