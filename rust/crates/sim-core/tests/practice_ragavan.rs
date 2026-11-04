//! Practice mode: a card Ragavan, Nimble Pilferer exiles is in the person's hand for the turn ("you may cast it this
//! turn"), and an instant or sorcery can be cast like any card of the hand.

mod play_common;
use play_common::*;
use sim_core::engine::hooks::run_hook;
use sim_core::hooks::{Call, Event};

#[test]
fn you_cast_a_sorcery_or_instant_ragavan_exiled() {
    let mut g = table(&["sauron", "seph"]);
    let rag = perm(&mut g, P0, "Ragavan, Nimble Pilferer");
    let c = card(&g, "Think Twice");
    g.player_mut(P1).library.retain(|&x| x != c);
    g.player_mut(P1).library.push(c);
    run_hook(&mut g, rag, Event::CombatDamage, &Call::CombatDamage { p: P0, a: rag, d: P1, dmg: 2 }).unwrap();
    let i = g.player(P0).hand.iter().position(|&x| x == c).expect("in your hand for the turn");
    pool(&mut g, P0, &[('U', 2)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": i}))]);
    assert!(!g.player(P0).hand.contains(&c), "cast");
    assert!(g.player(P1).gy.contains(&c), "it goes to its owner's graveyard");
    assert_eq!(pool_total(&g, P0), 0, "{{1}}{{U}} paid from your pool");
}
