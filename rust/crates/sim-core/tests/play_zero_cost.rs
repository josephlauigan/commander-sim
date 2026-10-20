//! Practice mode: a zero-cost spell (Lotus Petal) casts from an empty mana pool, and Trinisphere's floor of three
//! applies only while Trinisphere is untapped (the refusal names it).

mod play_common;
use play_common::*;

#[test]
fn lotus_petal_casts_for_free() {
    let mut g = table(&["kefka", "seph"]);
    let c = hand(&mut g, P0, &["Lotus Petal"])[0];
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c), "{}", told_last(&h));
    assert!(has_perm(&g, P0, "Lotus Petal"));
}

#[test]
fn trinisphere_makes_lotus_petal_cost_three() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Trinisphere");
    let c = hand(&mut g, P0, &["Lotus Petal"])[0];
    pool(&mut g, P0, &[('C', 2)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    let why = told_last(&h);
    assert!(g.player(P0).hand.contains(&c));
    assert!(why.contains("It costs {3}") && why.contains("Missing {1}") && why.contains("Trinisphere"), "{why}");
}

#[test]
fn a_tapped_trinisphere_sets_no_floor() {
    let mut g = table(&["kefka", "seph"]);
    let t = perm(&mut g, P0, "Trinisphere");
    g.perm_mut(t).tapped = true;
    let c = hand(&mut g, P0, &["Lotus Petal"])[0];
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c), "{}", told_last(&h));
}
