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
    // the compiled "when it enters, gain N life" is the one path (Phase 3: the engine's land_etb_fx is gone)
    for (name, n) in [("Jungle Hollow", 1), ("Scoured Barrens", 1), ("Thornwood Falls", 1), ("Radiant Fountain", 2)] {
        let mut g = game();
        let life = g.player(P0).life;
        land_enters(&mut g, P0, name);
        assert_eq!(g.player(P0).life - life, n, "{name}");
    }
}

#[test]
fn a_lands_compiled_ability_is_offered_unless_another_path_runs_it() {
    // Phase 3: a land's compiled activated ability is offered as a permanent's is, except a fetch land's (the f tag
    // cracks it: turn::crack_fetch) and a channel ability (used from the hand by card code: Boseiju's hand_options);
    // Treasure Vault has none (dsl = []: the ability language has no {X} activation cost)
    let mut g = game();
    for name in ["Flooded Strand", "Boseiju, Who Endures", "Castle Ardenvale", "Treasure Vault"] {
        let c = take(&mut g, P0, name);
        g.add_land(P0, c, false);
    }
    lands(&mut g, P0, "Forest", 4, false);
    lands(&mut g, P0, "Plains", 2, false);
    let opts = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    let mut labels: Vec<&str> = opts.iter().map(|o| o.label.as_str()).collect();
    labels.sort();
    assert_eq!(labels, ["Castle Ardenvale ability"]);
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
