//! Gamble (Kefka's tutor for the Underworld Breach line): a card found, one discarded at random. The line itself was
//! the Sauron and Kefka hand AIs', deleted on 2026-11-01 with them.

use sim_core::engine::cast::cast_card;
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Kefka against one Veyran with `opp_library` cards left (no hand), `fuel` Islands in Kefka's graveyard
fn board(swamps: usize, mountains: usize, islands: usize, opp_library: usize, fuel: usize) -> Game {
    let mut g = table(&["kefka", "veyran"]);
    let pl = g.player_mut(P1);
    pl.hand.clear();
    let rest = pl.library.split_off(opp_library.min(pl.library.len()));
    pl.gy.extend(rest);
    let island = card(&g, "Island");
    let k = g.player_mut(P0);
    k.hand.clear();
    k.gy = vec![island; fuel];
    lands(&mut g, P0, "Swamp", swamps, false);
    lands(&mut g, P0, "Mountain", mountains, false);
    lands(&mut g, P0, "Island", islands, false);
    g
}

#[test]
fn gamble_tutors_a_card_and_discards_one_at_random() {
    let mut g = board(3, 3, 1, 40, 0);
    let ub = card(&g, "Underworld Breach");
    let bf = card(&g, "Brain Freeze");
    g.player_mut(P0).library.extend([bf, ub]);
    let mut dn = g.player(P0).deck_names.to_vec();
    dn.push(bf);
    g.player_mut(P0).deck_names = dn.into();
    let h = hand(&mut g, P0, &["Gamble", "Island", "Island", "Island"]);
    assert!(cast_card(&mut g, P0, h[0], "hand", Ctx::default()).unwrap());
    let pl = g.player(P0);
    assert_eq!(pl.hand.len() + pl.gy.len(), 5, "one found, one discarded at random");
}
