//! One source that makes several mana pays only the colours it can make. Before, a unit was a colour SET and an
//! amount, so Gilded Lotus's three `Any` mana paid {W}{U}{B}, and a Karoo's {B}{G} paid {B}{B}.
//!
//! * "N mana of any one colour" (Gilded Lotus, Lotus Field, Black Lotus, Sanctum Weaver, a Treasure under Goldspan
//!   Dragon): all N are ONE colour, chosen as the source is tapped. Tag `oneclr` in the card data.
//! * a fixed set (Golgari Rot Farm and the other Karoos, Golgari Signet, Faeburrow Elder, Bloom Tender): exactly one
//!   mana of each colour in the set.
//! What a payment leaves over floats as the colour it was made in, never as "any colour".

use sim_core::engine::mana::{can_pay, pay, total_mana};
use sim_core::engine::pool::Kind;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// a five-colour seat (scion) against three seats, its hand emptied
fn five_colours() -> Game {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

/// what floats, by colour letter ('A' is any colour)
fn floating(g: &Game) -> Vec<(char, u32)> {
    [('W', Kind::W), ('U', Kind::U), ('B', Kind::B), ('R', Kind::R), ('G', Kind::G), ('A', Kind::Any), ('C', Kind::C)]
        .into_iter()
        .map(|(c, k)| (c, g.player(P0).mana.get(k)))
        .filter(|x| x.1 > 0)
        .collect()
}

#[test]
fn gilded_lotus_pays_one_colour_only() {
    let mut g = five_colours();
    perm(&mut g, P0, "Gilded Lotus");
    assert!(!can_pay(&g, P0, 0, "WUB", false), "three mana of one colour are not {{W}}{{U}}{{B}}");
    assert!(!can_pay(&g, P0, 0, "UUB", false));
    assert!(can_pay(&g, P0, 0, "UUU", false));
    assert!(can_pay(&g, P0, 2, "U", false), "{{2}}{{U}}");
    assert!(can_pay(&g, P0, 3, "", false), "generic takes any");
}

#[test]
fn gilded_lotus_with_other_sources_covers_the_other_colours() {
    let mut g = five_colours();
    perm(&mut g, P0, "Gilded Lotus");
    lands(&mut g, P0, "Plains", 1, false);
    assert!(!can_pay(&g, P0, 0, "WUB", false), "the Plains is white, the Lotus one of blue and black");
    assert!(can_pay(&g, P0, 0, "WUU", false), "Plains {{W}}, Lotus {{U}}{{U}}");
    lands(&mut g, P0, "Island", 1, false);
    assert!(can_pay(&g, P0, 0, "WUB", false), "Plains {{W}}, Island {{U}}, Lotus {{B}}");
    assert!(!can_pay(&g, P0, 0, "WURG", false), "the Lotus makes one colour only");
}

#[test]
fn two_lotuses_make_two_colours() {
    let mut g = five_colours();
    perm(&mut g, P0, "Gilded Lotus");
    assert!(!can_pay(&g, P0, 0, "UUUBBB", false));
    perm(&mut g, P0, "Gilded Lotus");
    assert!(can_pay(&g, P0, 0, "UUUBBB", false), "one Lotus each");
    assert!(!can_pay(&g, P0, 0, "UUUBBR", false));
}

#[test]
fn lotus_field_and_black_lotus_are_one_colour_too() {
    for (name, land) in [("Lotus Field", true), ("Black Lotus", false)] {
        let mut g = five_colours();
        if land {
            lands(&mut g, P0, name, 1, false);
        } else {
            perm(&mut g, P0, name);
        }
        assert!(!can_pay(&g, P0, 0, "RGW", false), "{name}");
        assert!(can_pay(&g, P0, 0, "RRR", false), "{name}");
        assert!(can_pay(&g, P0, 1, "GG", false), "{name}");
    }
}

#[test]
fn a_lotus_floats_what_the_payment_left_as_the_colour_it_paid() {
    let mut g = five_colours();
    perm(&mut g, P0, "Gilded Lotus");
    assert!(pay(&mut g, P0, 0, "U", false).unwrap());
    assert_eq!(floating(&g), vec![('U', 2)], "tapped for {{U}}{{U}}{{U}}: two blue left");
    // and what is left pays blue only
    assert!(can_pay(&g, P0, 0, "UU", false));
    assert!(!can_pay(&g, P0, 0, "UB", false));
}

#[test]
fn a_lotus_paying_generic_only_floats_one_colour() {
    let mut g = five_colours();
    perm(&mut g, P0, "Gilded Lotus");
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    let f = floating(&g);
    assert_eq!(f.len(), 1, "one colour, never split: {f:?}");
    assert_eq!(f[0].1, 2);
    assert_ne!(f[0].0, 'A', "not any-colour mana");
}

#[test]
fn a_karoo_pays_its_pair_not_the_same_colour_twice() {
    let mut g = five_colours();
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    assert!(can_pay(&g, P0, 0, "BG", false));
    assert!(!can_pay(&g, P0, 0, "BB", false), "{{B}}{{G}}, not {{B}}{{B}}");
    assert!(!can_pay(&g, P0, 0, "GG", false));
    assert!(!can_pay(&g, P0, 0, "BR", false));
    assert!(can_pay(&g, P0, 2, "", false));
    assert!(can_pay(&g, P0, 1, "B", false));
    assert!(can_pay(&g, P0, 1, "G", false));
}

#[test]
fn every_karoo_pays_its_own_pair() {
    for (name, a, b) in [
        ("Azorius Chancery", 'W', 'U'),
        ("Boros Garrison", 'W', 'R'),
        ("Gruul Turf", 'R', 'G'),
        ("Izzet Boilerworks", 'U', 'R'),
        ("Orzhov Basilica", 'W', 'B'),
        ("Rakdos Carnarium", 'B', 'R'),
        ("Selesnya Sanctuary", 'G', 'W'),
        ("Simic Growth Chamber", 'U', 'G'),
    ] {
        let mut g = five_colours();
        lands(&mut g, P0, name, 1, false);
        let pair: String = [a, b].iter().collect();
        assert!(can_pay(&g, P0, 0, &pair, false), "{name} pays {pair}");
        assert!(!can_pay(&g, P0, 0, &format!("{a}{a}"), false), "{name}: {a}{a}");
        assert!(!can_pay(&g, P0, 0, &format!("{b}{b}"), false), "{name}: {b}{b}");
    }
}

#[test]
fn a_karoo_floats_the_other_colour_of_its_pair() {
    let mut g = five_colours();
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    assert!(pay(&mut g, P0, 0, "B", false).unwrap());
    assert_eq!(floating(&g), vec![('G', 1)], "tapped for {{B}}{{G}}, paid {{B}}: the {{G}} floats");
}

#[test]
fn a_karoo_paying_generic_floats_nothing() {
    let mut g = five_colours();
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    assert!(pay(&mut g, P0, 2, "", false).unwrap());
    assert!(floating(&g).is_empty());
}

#[test]
fn golgari_signet_pays_its_pair_not_the_same_colour_twice() {
    let mut g = five_colours();
    perm(&mut g, P0, "Golgari Signet");
    assert_eq!(total_mana(&g, P0, false), 2);
    assert!(can_pay(&g, P0, 0, "BG", false));
    assert!(!can_pay(&g, P0, 0, "BB", false));
    assert!(!can_pay(&g, P0, 0, "GG", false));
    assert!(pay(&mut g, P0, 0, "G", false).unwrap());
    assert_eq!(floating(&g), vec![('B', 1)]);
}

#[test]
fn a_karoo_and_a_signet_together_make_two_pairs() {
    let mut g = five_colours();
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    perm(&mut g, P0, "Golgari Signet");
    assert!(can_pay(&g, P0, 0, "BBGG", false), "a pair from each");
    assert!(!can_pay(&g, P0, 0, "BBBG", false));
}

#[test]
fn colourless_sources_of_several_mana_stay_colourless() {
    let mut g = five_colours();
    perm(&mut g, P0, "Sol Ring");
    assert!(can_pay(&g, P0, 2, "", false));
    assert!(!can_pay(&g, P0, 0, "U", false));
}

#[test]
fn faeburrow_elder_and_bloom_tender_make_one_of_each_colour_and_tap_once() {
    let mut g = five_colours();
    let e = perm(&mut g, P0, "Faeburrow Elder"); // green and white
    perm(&mut g, P0, "Cuboid Colony"); // green and blue
    assert!(can_pay(&g, P0, 0, "GWU", false));
    assert!(!can_pay(&g, P0, 0, "GGW", false));
    assert!(pay(&mut g, P0, 0, "GWU", false).unwrap());
    assert!(g.perm(e).tapped);

    let mut g = five_colours();
    perm(&mut g, P0, "Bloom Tender"); // green
    perm(&mut g, P0, "Cuboid Colony"); // green and blue
    assert!(can_pay(&g, P0, 0, "GU", false), "Bloom Tender: one mana of each colour among your permanents");
    assert!(!can_pay(&g, P0, 0, "UU", false));
    assert!(!can_pay(&g, P0, 0, "GR", false), "red is not among them");
}

#[test]
fn a_treasure_under_goldspan_dragon_makes_two_of_one_colour() {
    let mut g = five_colours();
    perm(&mut g, P0, "Goldspan Dragon");
    g.player_mut(P0).treasures = 1;
    assert!(can_pay(&g, P0, 0, "UU", false));
    assert!(!can_pay(&g, P0, 0, "UR", false), "two mana of any ONE colour");
}

#[test]
fn a_dork_that_makes_one_mana_is_unchanged() {
    let mut g = five_colours();
    perm(&mut g, P0, "Birds of Paradise");
    lands(&mut g, P1, "Island", 1, false);
    assert!(can_pay(&g, P0, 0, "R", false));
    assert!(can_pay(&g, P0, 0, "W", false));
    assert!(!can_pay(&g, P0, 0, "WU", false));
}
