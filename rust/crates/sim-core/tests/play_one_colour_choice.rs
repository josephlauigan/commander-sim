//! Practice mode: "add N mana of any one colour" (Carpet of Flowers, The Aetherspark, Photon) asks the person which
//! colour, then floats N of it. It used to float `Kind::Any`, which the person could split across colours. A source of
//! several mana the person taps (Gilded Lotus, a Karoo) gives one colour / its exact pair.

mod play_common;
use play_common::*;
use sim_core::engine::pool::Kind;
use sim_core::engine::turn::continue_turn;
use sim_core::hooks::{Call, Event};
use sim_core::human::mana;
use std::sync::Arc;

/// a five-colour person at P0 against a Sauron seat; `answer` picks the colour named in a colour question
fn five_colour_person(colour: &'static str) -> (Game, Arc<ScriptHuman>) {
    let mut g = table(&["scion", "kraum-tymna-blue-breach-midrange"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        match r.choices.iter().position(|c| c.as_str() == colour) {
            Some(i) => Answer::Pick(i),
            None => Answer::Yes,
        }
    }));
    g.humans.add(P0, h.clone());
    (g, h)
}

fn colour_questions(h: &ScriptHuman) -> Vec<Request> {
    h.asked().into_iter().filter(|r| r.choices.iter().any(|c| c == "white") && r.choices.len() == 5).collect()
}

fn floating(g: &Game, p: sim_core::ids::PlayerId) -> Vec<(char, u32)> {
    [('W', Kind::W), ('U', Kind::U), ('B', Kind::B), ('R', Kind::R), ('G', Kind::G), ('A', Kind::Any), ('C', Kind::C)]
        .into_iter()
        .map(|(c, k)| (c, g.player(p).mana.get(k)))
        .filter(|x| x.1 > 0)
        .collect()
}

#[test]
fn carpet_of_flowers_asks_the_person_for_the_colour() {
    let (mut g, h) = five_colour_person("red");
    perm(&mut g, P0, "Carpet of Flowers");
    lands(&mut g, P1, "Island", 3, false);
    sim_core::engine::hooks::fire_trigger(&mut g, Event::Main1, Call::Player { p: P0 }).unwrap();
    let qs = colour_questions(&h);
    assert_eq!(qs.len(), 1, "one question about the colour: {:?}", h.asked());
    assert_eq!(qs[0].choices, ["white", "blue", "black", "red", "green"], "the colours of the person's identity");
    assert_eq!(floating(&g, P0), vec![('R', 3)], "three Islands: three red, not three of any colour");
}

#[test]
fn the_aetherspark_asks_the_person_for_the_colour() {
    let (mut g, h) = five_colour_person("green");
    let m = perm(&mut g, P0, "The Aetherspark");
    let c = g.perm(m).card();
    let ab = g.registry.get(c).unwrap().walker.unwrap();
    (ab[2].eff)(&mut g, P0, m).unwrap();
    assert_eq!(colour_questions(&h).len(), 1);
    assert_eq!(floating(&g, P0), vec![('G', 10)]);
}

#[test]
fn a_one_colour_identity_is_not_asked() {
    let mut g = table(&["cloud", "kraum-tymna-blue-breach-midrange"]);
    g.active = Some(P0);
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Yes));
    g.humans.add(P0, h.clone());
    let m = perm(&mut g, P0, "The Aetherspark");
    let c = g.perm(m).card();
    let ab = g.registry.get(c).unwrap().walker.unwrap();
    (ab[2].eff)(&mut g, P0, m).unwrap();
    assert!(h.asked().iter().all(|r| r.choices.len() != 1), "nothing to choose between");
    assert_eq!(floating(&g, P0), vec![('W', 10)], "Cloud is white");
}

#[test]
fn the_persons_floating_mana_then_pays_one_colour_only() {
    let (mut g, _h) = five_colour_person("red");
    let m = perm(&mut g, P0, "The Aetherspark");
    let c = g.perm(m).card();
    let ab = g.registry.get(c).unwrap().walker.unwrap();
    (ab[2].eff)(&mut g, P0, m).unwrap();
    mana::gather_floating(&mut g, P0);
    assert!(mana::pool_can_pay(&g, P0, 2, "RR"));
    assert!(!mana::pool_can_pay(&g, P0, 0, "RG"), "ten red mana are not a green mana");
}

#[test]
fn tapping_gilded_lotus_gives_three_of_the_colour_chosen() {
    let (mut g, _h) = five_colour_person("red");
    perm(&mut g, P0, "Gilded Lotus");
    let id = mana::sources(&g, P0).into_iter().find(|s| s.name == "Gilded Lotus").unwrap().id as i64;
    assert_eq!(mana::tap(&mut g, P0, id, Some('B')).unwrap(), None);
    assert_eq!(pool_text(&g, P0), "{B}{B}{B}");
}

#[test]
fn tapping_a_karoo_gives_its_pair_whatever_colour_is_clicked() {
    for click in [None, Some('B'), Some('G')] {
        let (mut g, _h) = five_colour_person("red");
        lands(&mut g, P0, "Golgari Rot Farm", 1, false);
        let s = mana::sources(&g, P0).into_iter().find(|s| s.name == "Golgari Rot Farm").unwrap();
        assert!(s.pair, "the page shows one button for a pair");
        assert_eq!(mana::tap(&mut g, P0, s.id as i64, click).unwrap(), None);
        assert_eq!(g.player(P0).pool.total(), 2);
        assert!(mana::pool_can_pay(&g, P0, 0, "BG"), "{click:?}: {}", pool_text(&g, P0));
        assert!(!mana::pool_can_pay(&g, P0, 0, "BB"), "{click:?}: {}", pool_text(&g, P0));
    }
}

#[test]
fn a_dork_is_not_a_pair() {
    let (mut g, _h) = five_colour_person("red");
    perm(&mut g, P0, "Birds of Paradise");
    let s = mana::sources(&g, P0).into_iter().find(|s| s.name == "Birds of Paradise").unwrap();
    assert!(!s.pair);
    let _ = continue_turn;
}
