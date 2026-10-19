//! The omnath deck (shares impls/titania.rs with the titania deck; Titania is a card in the 99). Seat 0 plays the omnath
//! deck, seat 1 the seph deck.

use sim_core::engine::turn::crack_fetch;
use sim_core::engine::values::{epow, etgh};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::titania;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn game() -> Game {
    table(&["omnath", "seph"])
}

fn tokens(g: &Game, p: PlayerId, pw: i32, th: i32) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && epow(g, m) == pw && etgh(g, m) == th)
        .collect()
}

#[test]
fn omnath_and_titania_both_make_elementals_when_a_fetch_land_is_cracked() {
    let mut g = game();
    perm(&mut g, P0, "Omnath, Locus of Rage");
    perm(&mut g, P0, "Titania, Protector of Argoth");
    let l = lands(&mut g, P0, "Evolving Wilds", 1, false)[0];
    crack_fetch(&mut g, P0, l).unwrap();
    assert_eq!(tokens(&g, P0, 5, 3).len(), 1, "Titania: a 5/3 for the land in the graveyard");
    assert_eq!(tokens(&g, P0, 5, 5).len(), 1, "Omnath: a 5/5 for the basic that entered");
}

#[test]
fn the_omnath_deck_shares_the_titania_plan() {
    assert!(titania::is_deck("omnath") && titania::is_deck("titania") && !titania::is_deck("seph"));
    // Titania is in hand, five mana: a fetch land is held (she can be cast next turn without it)
    let mut g = game();
    g.player_mut(P0).turns = 5;
    lands(&mut g, P0, "Forest", 5, false);
    hand(&mut g, P0, &["Titania, Protector of Argoth"]);
    let fetch = hand(&mut g, P0, &["Evolving Wilds"])[0];
    assert!(titania::hold_fetch(&g, P0));
    assert!(titania::land_adjust(&g, P0, fetch) > 0.0);
    // with Omnath out every crack is a landfall: no holding
    perm(&mut g, P0, "Omnath, Locus of Rage");
    assert!(!titania::hold_fetch(&g, P0));
}
