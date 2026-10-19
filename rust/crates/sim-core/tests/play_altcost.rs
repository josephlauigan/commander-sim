//! Practice mode: alternative costs and graveyard / land abilities that card code keeps in the AI's `hand_options`,
//! `gy_options` and `land_options` hooks are offered to the person.

mod play_common;
use play_common::*;
use sim_core::state::DataKey;

fn cast_hand(i: usize) -> F {
    act(json!({"do": "cast", "card": i}))
}

#[test]
fn you_dash_ragavan() {
    let mut g = table(&["godo", "seph"]);
    hand(&mut g, P0, &["Ragavan, Nimble Pilferer"]);
    pool(&mut g, P0, &[('R', 2)]);
    let h = main(&mut g, P0, vec![cast_hand(0), by_text("dash")]);
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Ragavan, Nimble Pilferer: which?")), "asked how to cast it");
    let m = perms_named(&g, P0, "Ragavan, Nimble Pilferer");
    assert_eq!(m.len(), 1, "Ragavan is on the battlefield");
    assert!(g.perm(m[0]).data.truthy(DataKey::Dash), "dashed");
    assert!(!g.perm(m[0]).sick, "with haste");
    assert_eq!(pool_total(&g, P0), 0, "{{1}}{{R}} paid from your pool");
    assert!(g.player(P0).hand.is_empty());
}

#[test]
fn you_may_cast_ragavan_normally_instead() {
    let mut g = table(&["godo", "seph"]);
    hand(&mut g, P0, &["Ragavan, Nimble Pilferer"]);
    pool(&mut g, P0, &[('R', 2)]);
    main(&mut g, P0, vec![cast_hand(0), by_text("cast it")]);
    let m = perms_named(&g, P0, "Ragavan, Nimble Pilferer");
    assert_eq!(m.len(), 1);
    assert!(!g.perm(m[0]).data.truthy(DataKey::Dash), "not dashed");
    assert_eq!(pool_total(&g, P0), 1, "only {{R}} paid");
}

#[test]
fn you_escape_woe_strider() {
    let mut g = table(&["sephfs", "seph"]);
    let woe = take(&mut g, P0, "Woe Strider");
    let mut gy = vec![woe];
    for n in ["Swamp", "Sol Ring", "Mind Stone", "Arcane Signet"] {
        gy.push(take(&mut g, P0, n));
    }
    g.player_mut(P0).gy = gy;
    pool(&mut g, P0, &[('B', 5)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "zone": "gy", "card": 0}))]);
    let m = perms_named(&g, P0, "Woe Strider");
    assert_eq!(m.len(), 1, "Woe Strider escaped onto the battlefield; asked: {:?}", h.asked());
    assert_eq!(g.perm(m[0]).plus, 2, "with two +1/+1 counters");
    assert_eq!(pool_total(&g, P0), 0);
    assert!(g.player(P0).gy.is_empty(), "four other cards exiled");
}

#[test]
fn the_graveyard_list_shows_an_escape_card() {
    let mut g = table(&["sephfs", "seph"]);
    let woe = take(&mut g, P0, "Woe Strider");
    g.player_mut(P0).gy = vec![woe];
    assert!(sim_core::human::alt::gy_hook(&g, woe), "the page lists it");
}

#[test]
fn you_use_phyrexian_tower() {
    let mut g = table(&["sheoldred", "seph"]);
    let l = lands(&mut g, P0, "Phyrexian Tower", 1, false)[0];
    let t = token(&mut g, P0, 1);
    let h = main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), pick(0)]);
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Phyrexian Tower: sacrifice which")), "you chose the creature");
    assert!(!g.perm(t).on_bf, "sacrificed");
    assert!(g.land(l).tapped);
    assert_eq!(pool_total(&g, P0), 2, "{{B}}{{B}} added");
}

#[test]
fn you_kick_everflowing_chalice_twice_and_it_taps_for_two() {
    let mut g = table(&["godo", "seph"]);
    hand(&mut g, P0, &["Everflowing Chalice"]);
    pool(&mut g, P0, &[('B', 4)]);
    let h = main(&mut g, P0, vec![cast_hand(0), by_text("2 times")]);
    assert!(h.asked().iter().any(|r| r.prompt.contains("multikicker")), "asked how many kicks");
    let m = perms_named(&g, P0, "Everflowing Chalice");
    assert_eq!(m.len(), 1);
    assert_eq!(g.perm(m[0]).data.int(DataKey::Kicks), 2, "two kicks recorded");
    assert_eq!(pool_total(&g, P0), 0, "{{4}} paid");
    let src = sim_core::human::mana::sources(&g, P0);
    assert!(src.iter().any(|s| s.name.contains("Chalice") && s.amount == 2), "it taps for 2");
}

#[test]
fn you_may_not_afford_the_kicks() {
    let mut g = table(&["godo", "seph"]);
    hand(&mut g, P0, &["Everflowing Chalice"]);
    pool(&mut g, P0, &[('B', 1)]);
    let h = main(&mut g, P0, vec![cast_hand(0), pick(0)]);
    let labels = &h.asked().iter().find(|r| r.prompt.contains("multikicker")).unwrap().choices.clone();
    assert_eq!(labels.len(), 2, "no kicks, plus cancel: {labels:?}");
}

#[test]
fn you_flashback_twinned_vision() {
    let mut g = table(&["kefka", "seph"]);
    let tv = take(&mut g, P0, "Twinned Vision");
    g.player_mut(P0).gy = vec![tv];
    pool(&mut g, P0, &[('U', 3), ('R', 3), ('B', 3)]);
    hand(&mut g, P0, &["Sol Ring"]);
    let hand0 = g.player(P0).hand.len();
    main(&mut g, P0, vec![act(json!({"do": "cast", "zone": "gy", "card": 0}))]);
    assert_eq!(g.player(P0).hand.len(), hand0 - 1 + 2, "discarded one, drew two");
    assert!(!g.player(P0).gy.contains(&tv) || g.player(P0).gy.len() > 1);
}
