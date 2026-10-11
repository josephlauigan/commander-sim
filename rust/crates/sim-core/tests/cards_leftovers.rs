//! The last placeholders of the port: Necropotence's payment and delivery, Braids' upkeep sacrifice, the prowess the
//! card data doesn't show (Eris), and Kylox's free copies as real casts.

use sim_core::cardcode;
use sim_core::engine::combat;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn necropotence_pays_life_down_to_the_floor() {
    let mut g = table(&["yshtola", "sauron"]);
    perm(&mut g, P0, "Necropotence");
    g.player_mut(P0).hand.clear();
    let (life, lib) = (g.player(P0).life, g.player(P0).library.len());
    cardcode::necro_pay(&mut g, P0).unwrap();
    // nothing on the table threatens: the floor is 10, and the hand holds 8
    let n = ((life - 10).min(8) as usize).min(lib);
    assert_eq!(g.player(P0).hand.len(), n);
    assert_eq!(g.player(P0).life, life - n as i32);
    assert_eq!(g.player(P0).library.len(), lib - n);
    assert_eq!(g.player(P0).stats["necro_cards"], n as i64);
}

#[test]
fn necropotence_delivers_a_persons_cards_at_the_end_step() {
    let mut g = table(&["yshtola", "sauron"]);
    let turns = g.player(P0).turns;
    let (a, b) = (card(&g, "Sol Ring"), card(&g, "Brainstorm"));
    g.player_mut(P0).necro_wait = vec![(a, turns), (b, turns + 1)];
    let hand = g.player(P0).hand.len();
    cardcode::necro_deliver(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert_eq!(*g.player(P0).hand.last().unwrap(), a);
    assert!(g.player(P0).seen.contains(&a));
    assert_eq!(g.player(P0).necro_wait, vec![(b, turns + 1)]);
    cardcode::necro_deliver(&mut g, P0).unwrap(); // nothing more is due
    assert_eq!(g.player(P0).hand.len(), hand + 1);
}

#[test]
fn braids_makes_the_cheapest_permanent_go() {
    let mut g = table(&["yawgmoth-mono-black-aristocrats", "sauron"]);
    perm(&mut g, P1, "Braids, Cabal Minion");
    lands(&mut g, P0, "Swamp", 2, false);
    let tok = token(&mut g, P0, 1);
    cardcode::braids_sacrifice(&mut g, P0).unwrap(); // a 1/1 token (0.55) is cheaper than a land (6)
    assert!(!g.perm(tok).on_bf);
    assert_eq!(g.player(P0).lands.len(), 2);
    let gy = g.player(P0).gy.len();
    cardcode::braids_sacrifice(&mut g, P0).unwrap(); // only lands are left
    assert_eq!(g.player(P0).lands.len(), 1);
    assert_eq!(g.player(P0).gy.len(), gy + 1);
    assert_eq!(g.player(P0).stats["braids_sacs"], 2);
}

#[test]
fn eris_has_prowess_without_the_keyword() {
    let mut g = table(&["veyran", "sauron"]);
    let eris = perm(&mut g, P0, "Eris, Roar of the Storm");
    assert!(cardcode::is_prowess_card(&g, g.perm(eris).cd.unwrap()));
    sim_core::engine::cast::prowess(&mut g, P0);
    assert_eq!(g.perm(eris).eot_pt, (1, 1));
}

#[test]
fn kylox_casts_its_copies_for_real() {
    let mut g = table(&["seph", "sauron"]);
    let kylox = perm(&mut g, P0, "Kylox, Visionary Inventor");
    let tok = token(&mut g, P0, 2);
    let bolt = card(&g, "Lightning Bolt");
    g.player_mut(P0).library.push(bolt);
    let spells = g.player(P0).spells_this_turn;
    combat::attack_triggers(&mut g, P0, &[kylox], P1).unwrap();
    assert!(!g.perm(tok).on_bf);
    assert!(g.player(P0).exile.contains(&bolt));
    assert_eq!(g.player(P0).spells_this_turn, spells + 1);
}
