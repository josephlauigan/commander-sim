//! Practice mode: the "you may" effects of cards compiled from the DSL ask the person (cardgen sets `optional` on
//! the effect of a "you may" sentence); the AI ignores the flag.

mod play_common;
use play_common::*;

/// answers every yes/no question of `card` with `say`, picks the first of any other choice; records the prompts
fn script(card: &'static str, say: bool) -> ScriptHuman {
    ScriptHuman::from_fn(move |r| {
        if r.prompt.starts_with(card) && r.choices.len() == 2 && r.choices[0] == "yes" {
            Answer::Pick(if say { 0 } else { 1 })
        } else if r.prompt.starts_with(card) || r.kind == "search" {
            Answer::Pick(0)
        } else {
            Answer::Decline
        }
    })
}

fn lands_of(g: &Game) -> usize {
    g.player(P0).lands.len()
}

#[test]
fn solemn_simulacrum_asks_before_searching_for_a_basic_land() {
    for say in [true, false] {
        let mut g = table(&["cloud", "seph"]);
        let h = std::sync::Arc::new(script("Solemn Simulacrum", say));
        g.humans.add(P0, h.clone());
        let before = lands_of(&g);
        perm(&mut g, P0, "Solemn Simulacrum");
        let asked = h.asked();
        assert!(
            asked
                .iter()
                .any(|r| r.prompt.starts_with("Solemn Simulacrum") && r.choices.len() == 2 && r.choices[0] == "yes"),
            "asked yes/no: {:?}",
            asked.iter().map(|r| r.prompt.clone()).collect::<Vec<_>>()
        );
        assert_eq!(
            lands_of(&g),
            before + usize::from(say),
            "say yes = {say}: {:?}",
            asked.iter().map(|r| (r.kind, r.prompt.clone(), r.choices.clone())).collect::<Vec<_>>()
        );
    }
}

#[test]
fn gravedigger_asks_before_returning_a_creature_card() {
    for say in [true, false] {
        let mut g = table(&["cloud", "seph"]);
        let h = std::sync::Arc::new(script("Gravedigger", say));
        g.humans.add(P0, h.clone());
        let c = take(&mut g, P0, "Esper Sentinel");
        g.player_mut(P0).gy.push(c);
        perm(&mut g, P0, "Gravedigger");
        assert!(h.asked().iter().any(|r| r.prompt.starts_with("Gravedigger") && r.choices[0] == "yes"));
        assert_eq!(g.player(P0).hand.contains(&c), say, "say yes = {say}");
        assert_eq!(g.player(P0).gy.contains(&c), !say);
    }
}

#[test]
fn reclamation_sage_asks_before_destroying_an_artifact() {
    for say in [true, false] {
        let mut g = table(&["cloud", "seph"]);
        let h = std::sync::Arc::new(script("Reclamation Sage", say));
        g.humans.add(P0, h.clone());
        let target = perm(&mut g, P1, "Sol Ring");
        perm(&mut g, P0, "Reclamation Sage");
        assert!(h.asked().iter().any(|r| r.prompt.starts_with("Reclamation Sage") && r.choices[0] == "yes"));
        assert_eq!(g.perm(target).on_bf, !say, "say yes = {say}");
    }
}

#[test]
fn the_ai_is_not_asked_and_still_searches() {
    let mut g = table(&["cloud", "seph"]);
    let before = lands_of(&g);
    perm(&mut g, P0, "Solemn Simulacrum");
    assert_eq!(lands_of(&g), before + 1, "the AI fetches the land as before");
}

/// Burning-Tree Vandal's attack trigger: "you may discard a card. If you do, draw a card."
fn vandal(say: bool) -> (usize, usize, usize) {
    let mut g = table(&["cloud", "seph"]);
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.choices.len() == 2 && r.choices[0] == "yes" {
            Answer::Pick(if say { 0 } else { 1 })
        } else {
            Answer::Pick(0)
        }
    }));
    g.humans.add(P0, h);
    hand(&mut g, P0, &["Sol Ring", "Bonesplitter"]);
    let m = perm(&mut g, P0, "Burning-Tree Vandal");
    let (hand0, gy0, lib0) = (g.player(P0).hand.len(), g.player(P0).gy.len(), g.player(P0).library.len());
    let cd = g.perm(m).cd.unwrap();
    sim_core::dsl::run_trigger(&mut g, P0, m, cd, 0, &sim_core::dsl::DslCtx::base()).unwrap();
    let (hand1, gy1, lib1) = (g.player(P0).hand.len(), g.player(P0).gy.len(), g.player(P0).library.len());
    (hand1.wrapping_sub(hand0), gy1.wrapping_sub(gy0), lib0 - lib1)
}

#[test]
fn burning_tree_vandal_yes_discards_and_draws() {
    let (hand, gy, drawn) = vandal(true);
    assert_eq!((gy, drawn, hand), (1, 1, 0), "discard one, draw one");
}

#[test]
fn burning_tree_vandal_no_means_no_discard_and_no_draw() {
    assert_eq!(vandal(false), (0, 0, 0));
}
