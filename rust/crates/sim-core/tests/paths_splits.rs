//! Phase 2 (PR 5b): the declared splits (`paths_ok` in data/card_overrides.toml) where both paths could act on the
//! same event: the effect happens once. Seat 0 plays the titania deck, seat 1 the seph deck.

use sim_core::engine::{mana, zones};
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn game() -> sim_core::state::Game {
    table(&["omnath", "seph"])
}

/// a land card enters under p's control the way a land drop does (its enters effects, then landfall)
fn land_enters(g: &mut sim_core::state::Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.add_land_entering(p, c, true);
    zones::landfall(g, p).unwrap();
}

#[test]
fn a_gain_land_gains_its_life_once() {
    // compiled "when it enters, gain N life" and the engine's land_etb_fx: only the second runs for a land
    for (name, n) in [("Jungle Hollow", 1), ("Scoured Barrens", 1), ("Thornwood Falls", 1), ("Radiant Fountain", 2)] {
        let mut g = game();
        let life = g.player(P0).life;
        land_enters(&mut g, P0, name);
        assert_eq!(g.player(P0).life - life, n, "{name}");
    }
}

#[test]
fn a_lands_compiled_ability_is_never_offered() {
    // a land is no permanent in the sim: its compiled activated ability doesn't compete with the engine's handling
    let mut g = game();
    for name in ["Flooded Strand", "Boseiju, Who Endures", "Castle Ardenvale", "Treasure Vault"] {
        let c = take(&mut g, P0, name);
        g.add_land(P0, c, false);
    }
    lands(&mut g, P0, "Forest", 4, false);
    let opts = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(opts.is_empty(), "{:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
}

#[test]
fn cryogen_relic_draws_once_as_it_enters() {
    // the draw tag on entering; the card code's draw is for leaving
    let mut g = game();
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Cryogen Relic");
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn canopy_tactician_taps_for_three() {
    // the dork tag makes one mana unit; the card code's dyn_mana_perm sets its amount (not a second unit)
    let mut g = game();
    perm(&mut g, P0, "Canopy Tactician");
    assert_eq!(mana::total_mana(&g, P0, false), 3);
}
