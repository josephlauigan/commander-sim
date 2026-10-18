//! Tier 6 batch `blue` (impls/t6_blue.rs) on hand-built positions. Seat 0 plays the Rograkh & Silas deck, seat 1
//! the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::t6_blue as t6;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-silas-turbo-breach", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ======================================================== Borne Upon a Wind
#[test]
fn borne_upon_a_wind_draws_and_grants_flash_this_turn() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Borne Upon a Wind");
    assert_eq!(g.player(P0).hand.len(), before + 1, "the card left the hand, one was drawn");
    assert!(in_gy(&g, P0, "Borne Upon a Wind"));
    assert!(t6::flash_turn_grant(&g, P0));
    assert!(!t6::flash_turn_grant(&g, P1), "only the caster");
}
