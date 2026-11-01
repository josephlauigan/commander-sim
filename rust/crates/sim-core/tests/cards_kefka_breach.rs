//! Kefka's discard choice (`kefka::discard_choice`, the looters' own choice) keeps the pieces of the Underworld Breach
//! line the other half is waiting for. The line itself was the Sauron and Kefka hand AIs', deleted on 2026-11-01.

use sim_core::ids::PlayerId;
use sim_core::impls::kefka;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Kefka against one Veyran whose library holds `opp_library` cards (the rest milled, no hand); `fuel` Islands in
/// Kefka's graveyard; Kefka's hand empty
fn board(islands: usize, swamps: usize, opp_library: usize, fuel: usize) -> Game {
    let mut g = table(&["kefka", "veyran"]);
    let pl = g.player_mut(P1);
    pl.hand.clear();
    let rest = pl.library.split_off(opp_library.min(pl.library.len()));
    pl.gy.extend(rest);
    let island = card(&g, "Island");
    let k = g.player_mut(P0);
    k.hand.clear();
    k.gy = vec![island; fuel];
    lands(&mut g, P0, "Island", islands, false);
    lands(&mut g, P0, "Swamp", swamps, false);
    g
}

#[test]
fn kefka_does_not_discard_the_pieces_the_line_waits_for() {
    let mut g = board(5, 4, 30, 0);
    let h = hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Forest", "Forest"]);
    let pick = kefka::discard_choice(&g, P0, &h, 0).unwrap();
    assert!(&*g.db.get(pick).name == "Forest", "discarded {}", g.db.get(pick).name);
    // Brain Freeze alone (no Breach out) is fine to bin: it is escaped from the graveyard later
    let mut g = board(5, 4, 30, 0);
    let h = hand(&mut g, P0, &["Brain Freeze", "Island", "Swamp"]);
    let _ = kefka::discard_choice(&g, P0, &h, 0).unwrap();
}

