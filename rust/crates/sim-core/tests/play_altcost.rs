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
    assert!(
        h.asked().iter().any(|r| r.prompt.starts_with("Phyrexian Tower: sacrifice which")),
        "you chose the creature"
    );
    assert!(!g.perm(t).on_bf, "sacrificed");
    assert!(g.perm(l).tapped);
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

/// Diagnostic (run with --ignored --nocapture): for every card of JD's decks with a hand / gy / land hook, how many
/// ways the person is offered with plenty of mana.
#[test]
#[ignore]
fn probe_jd_cards() {
    use sim_core::human::alt;
    let keys: Vec<String> = sim_core::testkit::decks().iter().take(14).map(|d| d.key.clone()).collect();
    let mut seen = std::collections::BTreeSet::new();
    for key in keys {
        for name in sim_core::testkit::deck(&key).cards.clone() {
            if !seen.insert(name.clone()) {
                continue;
            }
            for kind in ["hand", "gy", "land"] {
                let mut g = table(&[key.as_str(), "seph"]);
                let Some(cid) = g.db.id(&name) else { continue };
                let has = match kind {
                    "hand" => alt::hand_hook(&g, cid),
                    "gy" => g.registry.get(cid).is_some_and(|i| i.gy_options.is_some()),
                    _ => g.registry.get(cid).is_some_and(|i| i.land_options.is_some()),
                };
                if !has {
                    continue;
                }
                seat(&mut g, P0, vec![]);
                pool(&mut g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6)]);
                token(&mut g, P0, 1);
                token(&mut g, P0, 1);
                lands(&mut g, P0, "Swamp", 4, false);
                let n = match kind {
                    "hand" => {
                        let c = take(&mut g, P0, &name);
                        g.player_mut(P0).hand.push(c);
                        alt::hand_alts(&mut g, P0, c).unwrap().len()
                    }
                    "gy" => {
                        let c = take(&mut g, P0, &name);
                        let mut gy = vec![c];
                        for n in ["Swamp", "Sol Ring", "Mind Stone", "Arcane Signet", "Island"] {
                            gy.push(take(&mut g, P0, n));
                        }
                        g.player_mut(P0).gy = gy;
                        hand(&mut g, P0, &["Sol Ring"]);
                        alt::gy_alts(&mut g, P0, c).unwrap().len()
                    }
                    _ => {
                        let l = lands(&mut g, P0, &name, 1, false)[0];
                        alt::land_alts(&mut g, P0, l).unwrap().len()
                    }
                };
                println!("PROBE {key:10} {kind:5} {name:40} {n}");
            }
        }
    }
}

#[test]
fn you_use_ghost_quarter_on_a_land_you_choose() {
    let mut g = table(&["alela", "seph"]);
    lands(&mut g, P0, "Ghost Quarter", 1, false);
    lands(&mut g, P1, "Island", 2, false);
    let h = main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), by_text("Island")]);
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Ghost Quarter: destroy which")), "you chose the land");
    assert!(g.player(P0).lands.is_empty(), "Ghost Quarter was sacrificed");
}

#[test]
fn you_use_geier_reach_sanitarium_with_the_mana_you_tapped() {
    let mut g = table(&["sheoldred", "seph"]);
    let l = lands(&mut g, P0, "Geier Reach Sanitarium", 1, false)[0];
    pool(&mut g, P0, &[('B', 2)]);
    main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), pick(0)]); // the ability, then the discard
    assert!(g.perm(l).tapped);
    assert_eq!(pool_total(&g, P0), 0, "{{2}} paid from your pool");
}

#[test]
fn you_return_reassembling_skeleton() {
    let mut g = table(&["sephfs", "seph"]);
    let c = take(&mut g, P0, "Reassembling Skeleton");
    g.player_mut(P0).gy = vec![c];
    pool(&mut g, P0, &[('B', 2)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "zone": "gy", "card": 0}))]);
    let m = perms_named(&g, P0, "Reassembling Skeleton");
    assert_eq!(m.len(), 1, "it came back");
    assert!(g.perm(m[0]).tapped, "tapped");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn you_cast_demonic_embrace_from_the_graveyard_and_choose_the_discard() {
    let mut g = table(&["kefka", "seph"]);
    let c = take(&mut g, P0, "Demonic Embrace");
    g.player_mut(P0).gy = vec![c];
    token(&mut g, P0, 1);
    hand(&mut g, P0, &["Sol Ring", "Mind Stone"]);
    pool(&mut g, P0, &[('B', 3)]);
    let life = g.player(P0).life;
    let h = main(
        &mut g,
        P0,
        vec![act(json!({"do": "cast", "zone": "gy", "card": 0})), by_text("Mind Stone"), pick(0), pick(0)],
    );
    println!("{:?}", h.asked().iter().map(|r| r.prompt.clone()).collect::<Vec<_>>());
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Demonic Embrace: discard")), "you chose the discard");
    assert_eq!(g.player(P0).life, life - 3);
    assert!(
        g.player(P0).gy.iter().any(|&x| &*g.db.get(x).name == "Mind Stone"),
        "the card you chose went to the graveyard"
    );
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(!g.player(P0).gy.contains(&c));
}

#[test]
fn you_cast_exalted_angel_face_down_and_turn_it_up() {
    let mut g = table(&["cloud", "seph"]);
    hand(&mut g, P0, &["Exalted Angel"]);
    pool(&mut g, P0, &[('W', 3)]);
    main(&mut g, P0, vec![cast_hand(0), by_text("face down")]);
    let m = perms_named(&g, P0, "Exalted Angel");
    assert_eq!(m.len(), 1, "on the battlefield face down");
    assert!(g.perm(m[0]).data.truthy(DataKey::Facedown));
    pool(&mut g, P0, &[('W', 4)]);
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": 0})), pick(0)]);
    assert!(!g.perm(m[0]).data.truthy(DataKey::Facedown), "turned face up");
}
