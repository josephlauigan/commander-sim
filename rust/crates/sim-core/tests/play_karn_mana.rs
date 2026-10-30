//! Practice mode: Karn, Legacy Reforged's mana (artifact spells and abilities only, kept all turn) is paid from the
//! person's pool like any other mana, and refused for a nonartifact spell.

mod play_common;
use play_common::*;

#[test]
fn karns_mana_casts_an_artifact_spell() {
    let mut g = table(&["karn", "seph"]);
    g.player_mut(P0).mana.set_art(3);
    let c = hand(&mut g, P0, &["Mind Stone"])[0]; // {2}
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c), "{}", told_last(&h));
    assert!(has_perm(&g, P0, "Mind Stone"));
    assert_eq!(pool_total(&g, P0), 1, "one of Karn's three is left");
}

#[test]
fn karns_mana_does_not_cast_a_planeswalker() {
    let mut g = table(&["karn", "seph"]);
    g.player_mut(P0).mana.set_art(6);
    let c = hand(&mut g, P0, &["Karn, Scion of Urza"])[0]; // {4}
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(g.player(P0).hand.contains(&c), "the planeswalker stays in hand");
    assert!(told_last(&h).contains("It costs"), "{}", told_last(&h));
}

#[test]
fn karns_mana_survives_the_end_of_a_step_in_the_pool() {
    let mut g = table(&["karn", "seph"]);
    g.player_mut(P0).mana.set_art(2);
    sim_core::human::mana::gather_floating(&mut g, P0);
    assert!(pool_text(&g, P0).contains("2 for artifact spells and abilities"), "{}", pool_text(&g, P0));
    sim_core::human::mana::empty_pools(&mut g);
    assert_eq!(pool_total(&g, P0), 2);
}
