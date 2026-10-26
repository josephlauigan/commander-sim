//! The casual combos of `teysa-orzhov-aristocrats` (impls/cc_teysa.rs) on hand-built positions. Each expectation
//! follows the cards' Oracle text: Teysa Karlov doubles Pitiless Plunderer's death trigger, so a free sacrifice of
//! Reassembling Skeleton makes the two Treasures that pay its {1}{B}; Phyrexian Altar plus Pawn of Ulamog's (or Sifter
//! of Skulls's) token make the same two mana.

use sim_core::ids::PlayerId;
use sim_core::impls::{cc_teysa, combos, common};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const DOUBLED: &str = "Teysa Karlov / Drivnod + Pitiless Plunderer";
const SPAWN: &str = "Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog";

fn teysa() -> Game {
    let mut g = table(&[cc_teysa::DECK, "cloud"]);
    g.player_mut(P0).cmd_in_zone = false;
    g
}

fn idx(part: &str) -> usize {
    combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"))
}

#[test]
fn teysa_plunderer_skeleton_and_a_free_outlet_loop_into_a_payoff() {
    let mut g = teysa();
    let i = idx(DOUBLED);
    perm(&mut g, P0, "Pitiless Plunderer");
    perm(&mut g, P0, "Reassembling Skeleton");
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Blood Artist");
    assert!(!combos::ready(&g, P0, i), "one Treasure a death does not pay {{1}}{{B}}: no doubler");
    perm(&mut g, P0, "Teysa Karlov");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn drivnod_doubles_as_teysa_does_and_a_payoff_is_needed() {
    let mut g = teysa();
    let i = idx(DOUBLED);
    for n in ["Drivnod, Carnage Dominus", "Pitiless Plunderer", "Reassembling Skeleton", "Carrion Feeder"] {
        perm(&mut g, P0, n);
    }
    assert!(!combos::ready(&g, P0, i), "no payoff");
    perm(&mut g, P0, "Zulaport Cutthroat");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn the_skeleton_may_start_in_the_graveyard_with_its_return_open() {
    let mut g = teysa();
    let i = idx(DOUBLED);
    for n in ["Teysa Karlov", "Pitiless Plunderer", "Yahenni, Undying Partisan", "Cruel Celebrant"] {
        perm(&mut g, P0, n);
    }
    let skel = card(&g, "Reassembling Skeleton");
    g.player_mut(P0).gy.push(skel);
    assert!(!combos::ready(&g, P0, i), "no mana for {{1}}{{B}}");
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn phyrexian_altar_skeleton_and_a_spawner_loop_into_a_payoff() {
    let mut g = teysa();
    let i = idx(SPAWN);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Reassembling Skeleton");
    perm(&mut g, P0, "Bastion of Remembrance");
    assert!(!combos::ready(&g, P0, i), "no Pawn or Sifter: one mana a lap");
    perm(&mut g, P0, "Sifter of Skulls");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn ashnods_altar_makes_no_black_so_it_is_no_spawner_loop() {
    let mut g = teysa();
    let i = idx(SPAWN);
    for n in ["Ashnod's Altar", "Reassembling Skeleton", "Pawn of Ulamog", "Blood Artist"] {
        perm(&mut g, P0, n);
    }
    assert!(!combos::ready(&g, P0, i));
}

#[test]
fn the_lines_are_this_decks_only() {
    // sephfs runs the Pawn loop through its own plan; yawgmoth keeps its own lines
    let mut g = table(&["yawgmoth-mono-black-aristocrats", "cloud"]);
    for n in ["Phyrexian Altar", "Reassembling Skeleton", "Pawn of Ulamog", "Blood Artist"] {
        perm(&mut g, P0, n);
    }
    assert!(!combos::ready(&g, P0, idx(SPAWN)));
}

#[test]
fn an_unanswered_loop_wins() {
    let mut g = teysa();
    let i = idx(SPAWN);
    for n in ["Phyrexian Altar", "Reassembling Skeleton", "Pawn of Ulamog", "Blood Artist"] {
        perm(&mut g, P0, n);
    }
    g.active = Some(P0);
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!(g.winner, Some(P0));
    assert_eq!(g.wintype, Some("combo"));
}

#[test]
fn teysa_keeps_her_payoffs_and_the_plunderer_out_of_the_fodder() {
    // a fresh Blood Artist was sacrificed for its own drain: the loop then has no payoff
    let mut g = teysa();
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Pitiless Plunderer");
    perm(&mut g, P0, "Zulaport Cutthroat");
    assert!(common::aristocrat_options(&mut g, P0, Some(true)).unwrap().is_empty(), "no spare fodder");
    let t = token(&mut g, P0, 1);
    let o = common::aristocrat_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1);
    assert!(o[0].label.starts_with(&format!("sacrifice {}", g.perm(t).name)), "{}", o[0].label);
}

#[test]
fn other_pool_decks_still_sacrifice_payoffs() {
    let mut g = table(&["yawgmoth-mono-black-aristocrats", "cloud"]);
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Zulaport Cutthroat");
    assert!(!common::aristocrat_options(&mut g, P0, Some(true)).unwrap().is_empty());
}
