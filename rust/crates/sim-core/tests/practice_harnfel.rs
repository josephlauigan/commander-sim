//! Practice mode: Birgi, God of Storytelling // Harnfel, Horn of Bounty. A person can cast the back face, a legendary
//! artifact with "Discard a card: Exile the top two cards of your library. You may play those cards this turn."

mod play_common;
use play_common::*;

#[test]
fn you_cast_harnfel_and_use_its_ability() {
    let mut g = table(&["kefka", "seph"]);
    let cs = hand(&mut g, P0, &["Birgi, God of Storytelling", "Island"]);
    let birgi = cs[0];
    pool(&mut g, P0, &[('R', 5)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Harnfel")]);
    let m = perms_named(&g, P0, "Harnfel, Horn of Bounty");
    assert_eq!(m.len(), 1, "Harnfel is on the battlefield");
    assert_eq!(g.perm(m[0]).phys, Some(birgi), "the card behind it is Birgi");
    assert!(!g.player(P0).hand.contains(&birgi));
    assert_eq!(pool_total(&g, P0), 0, "{{4}}{{R}} paid");
    // its ability: discard the Island, exile the top two cards, play them this turn
    let top2: Vec<_> = g.player(P0).library.iter().rev().take(2).copied().collect();
    let i = g.player(P0).perms.iter().position(|&x| x == m[0]).unwrap();
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pick(0)]);
    assert!(g.player(P0).gy.contains(&cs[1]), "the Island was discarded");
    for c in &top2 {
        assert!(g.player(P0).hand.contains(c) && g.player(P0).impulse.contains(c), "playable this turn");
    }
    // when it dies the card that goes to the graveyard is Birgi
    sim_core::engine::zones::die(&mut g, m[0], "destroy").unwrap();
    assert!(g.player(P0).gy.contains(&birgi), "Birgi's card, not a card that doesn't exist");
}

#[test]
fn you_play_bloodsoaked_insight_as_the_land_sanguine_morass() {
    // the other modal double-faced card of the owned decks (Sauron): the back face is a land, played from the hand
    let mut g = table(&["sauron", "seph"]);
    let c = hand(&mut g, P0, &["Bloodsoaked Insight // Sanguine Morass"])[0];
    main(&mut g, P0, vec![act(json!({"do": "land", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c));
    assert!(g.player(P0).lands.iter().any(|&l| g.perm(l).cd == Some(c)), "on the battlefield as a land");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).cd != Some(c) || g.perm(l).tapped), "it enters tapped");
}
