//! Ghave, Guru of Spores (JD's `ghave` deck, impls/ghave.rs and the ability language) on hand-built positions. The
//! expectations follow the Oracle text: "Ghave enters with five +1/+1 counters on it. {1}, Remove a +1/+1 counter
//! from a creature you control: Create a 1/1 green Saproling creature token. {1}, Sacrifice a creature: Put a +1/+1
//! counter on target creature." The loops (Ghave + an altar + Cathars' Crusade or Doubling Season, Mikaeus +
//! Triskelion, Kitchen Finks + Cathars' Crusade) run through the combo catalogue as their result.

use sim_core::engine::zones::{self, Enter};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{combos, common};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const GHAVE: &str = "Ghave, Guru of Spores";
const SAPROLING: u16 = 1; // the compiled abilities: [0] the enters note, [1] the Saproling, [2] the sacrifice
const SACRIFICE: u16 = 2;
const DRAIN: &str = "Ghave + an altar + Cathars' Crusade / Doubling Season + a death payoff";

fn ghave_table() -> Game {
    let mut g = table(&["ghave", "sheoldred"]);
    g.player_mut(P0).cmd_in_zone = false;
    g.active = Some(P0);
    g
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn saprolings(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.is_creature(m)).count()
}

fn idx(part: &str) -> usize {
    combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"))
}

#[test]
fn ghave_enters_with_five_counters_and_ten_with_doubling_season() {
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    assert_eq!(g.perm(gh).plus, 5, "a 0/0 that enters with five +1/+1 counters");
    assert!(g.perm(gh).on_bf, "it survives as a 5/5");

    let mut g = ghave_table();
    perm(&mut g, P0, "Doubling Season");
    let gh = perm(&mut g, P0, GHAVE);
    assert_eq!(g.perm(gh).plus, 10, "Doubling Season doubles the counters it enters with");
}

#[test]
fn ghave_enters_with_his_counters_when_reanimated_too() {
    let mut g = ghave_table();
    let c = take(&mut g, P0, GHAVE);
    let gh = zones::enter(&mut g, P0, c, Enter { sick: false, ..Default::default() }).unwrap();
    assert_eq!(g.perm(gh).plus, 5);
}

#[test]
fn one_mana_and_a_counter_make_a_saproling() {
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(sim_core::dsl::activate(&mut g, P0, gh, SAPROLING).unwrap());
    assert_eq!(g.perm(gh).plus, 4, "the counter came off Ghave");
    assert_eq!(saprolings(&g, P0), 1, "a 1/1 green Saproling");
    assert!(!sim_core::dsl::activate(&mut g, P0, gh, SAPROLING).unwrap(), "no mana left for the {{1}}");
}

#[test]
fn the_counter_comes_from_any_creature_you_control() {
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    g.perm_mut(gh).plus = 1; // Ghave at 1/1: removing his last counter would kill him
    let t = perm(&mut g, P0, "Triskelion"); // three counters
    lands(&mut g, P0, "Forest", 1, false);
    assert!(sim_core::dsl::activate(&mut g, P0, gh, SAPROLING).unwrap());
    assert_eq!((g.perm(gh).plus, g.perm(t).plus), (1, 2), "the counter came off Triskelion, which spares it");
    assert_eq!(saprolings(&g, P0), 1);
}

#[test]
fn no_counter_no_saproling() {
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    g.perm_mut(gh).plus = 1;
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!sim_core::dsl::activate(&mut g, P0, gh, SAPROLING).unwrap(), "Ghave's last counter would kill him");
    assert_eq!(saprolings(&g, P0), 0);
}

#[test]
fn one_mana_and_a_sacrifice_put_a_counter_on_a_creature() {
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    let sap = token(&mut g, P0, 1);
    perm(&mut g, P0, "Blood Artist");
    lands(&mut g, P0, "Forest", 1, false);
    let life = g.player(P1).life;
    let total =
        |g: &Game| g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| g.perm(m).plus).sum::<i32>();
    let before = total(&g);
    assert!(sim_core::dsl::activate(&mut g, P0, gh, SACRIFICE).unwrap());
    assert!(!g.perm(sap).on_bf, "the token was the sacrifice");
    assert_eq!(total(&g), before + 1, "a +1/+1 counter on a creature");
    assert_eq!(g.player(P1).life, life - 1, "Blood Artist drained for the death");
}

#[test]
fn ghave_sacrifices_a_creature_in_response_to_exile() {
    // the deck's plan knob `paid_outlets`: Ghave's {1} outlet answers removal, as a free outlet would
    let mut g = ghave_table();
    let gh = perm(&mut g, P0, GHAVE);
    let titan = perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(common::paid_outlets_on(&g, P0));
    assert!(common::sac_in_response(&mut g, P0, titan, "exile").unwrap());
    assert!(!g.perm(titan).on_bf && g.player(P0).gy.contains(&card(&g, "Grave Titan")), "it died instead");
    assert!(g.perm(gh).on_bf);
}

#[test]
fn a_pool_deck_does_not_use_paid_outlets_in_response() {
    let g = table(&["sheoldred", "ghave"]);
    assert!(!common::paid_outlets_on(&g, P0), "only a deck whose plan asks for it");
}

#[test]
fn ghave_an_altar_and_cathars_crusade_need_a_payoff_then_win() {
    let mut g = ghave_table();
    let i = idx(DRAIN);
    perm(&mut g, P0, GHAVE);
    perm(&mut g, P0, "Ashnod's Altar");
    perm(&mut g, P0, "Cathars' Crusade");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!combos::ready(&g, P0, i), "unbounded Saprolings, but no payoff");
    perm(&mut g, P0, "Blood Artist");
    assert!(combos::ready(&g, P0, i));
    // the loop runs as its result (no step by step activation, so it cannot hang) and Blood Artist drains the table
    assert!(combos::attempt(&mut g, P0, i as i64).unwrap());
    assert_eq!(g.winner, Some(P0));
    assert_eq!(g.wintype, Some("combo"));
}

#[test]
fn phyrexian_altar_doubling_season_and_sephiroth_are_the_same_line() {
    let mut g = ghave_table();
    let i = idx(DRAIN);
    for n in [GHAVE, "Phyrexian Altar", "Doubling Season", "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel"] {
        perm(&mut g, P0, n);
    }
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn the_ghave_line_needs_a_counter_and_working_abilities() {
    let mut g = ghave_table();
    let i = idx(DRAIN);
    let gh = perm(&mut g, P0, GHAVE);
    for n in ["Ashnod's Altar", "Cathars' Crusade", "Zulaport Cutthroat"] {
        perm(&mut g, P0, n);
    }
    assert!(combos::ready(&g, P0, i));
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).plus = 0; // (as if shrunk: Ghave would be dead, but the check is the counters; Crusade put
        // one on Zulaport too)
    }
    assert!(!combos::ready(&g, P0, i), "no +1/+1 counter to remove for the first Saproling");
    g.perm_mut(gh).plus = 3;
    perm(&mut g, P1, "Rest in Peace");
    assert!(!combos::ready(&g, P0, i), "Rest in Peace: nothing dies, the payoffs never trigger");
}

#[test]
fn ashnods_mana_and_triskelion_win_without_a_drain() {
    let mut g = ghave_table();
    let i = idx("Ghave + Ashnod's Altar + Cathars' Crusade / Doubling Season + Triskelion");
    for n in [GHAVE, "Phyrexian Altar", "Cathars' Crusade", "Triskelion"] {
        perm(&mut g, P0, n);
    }
    assert!(!combos::ready(&g, P0, i), "Phyrexian Altar nets no mana: no counters for Triskelion");
    perm(&mut g, P0, "Ashnod's Altar");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn mikaeus_and_triskelion_kill_alone() {
    let mut g = ghave_table();
    let i = idx("Mikaeus + Triskelion");
    perm(&mut g, P0, "Mikaeus, the Unhallowed");
    assert!(!combos::ready(&g, P0, i));
    perm(&mut g, P0, "Triskelion");
    assert!(combos::ready(&g, P0, i));
    assert!(combos::attempt(&mut g, P0, i as i64).unwrap());
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn kitchen_finks_cathars_crusade_and_a_free_outlet_loop_into_a_payoff() {
    let mut g = ghave_table();
    let i = idx("Kitchen Finks + Cathars' Crusade + a free outlet");
    for n in ["Kitchen Finks", "Cathars' Crusade", "Viscera Seer"] {
        perm(&mut g, P0, n);
    }
    assert!(!combos::ready(&g, P0, i), "unbounded life and deaths, no payoff");
    perm(&mut g, P0, "Cruel Celebrant");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn the_combos_are_the_decks_own() {
    // only the ghave list holds these pieces: deck_combos finds them for it and for no pool deck of its tiers
    let g = table(&["ghave", "sheoldred"]);
    let mine = combos::deck_combos(&g, P0);
    for part in [DRAIN, "Mikaeus + Triskelion", "Nim Deathmantle + Ashnod's Altar + Grave Titan"] {
        assert!(mine.contains(&idx(part)), "{part}");
    }
    for t in ["t3", "t4"] {
        for key in tier(t) {
            let g = table(&[key, "ghave"]);
            let theirs = combos::deck_combos(&g, P0);
            assert!(!theirs.contains(&idx(DRAIN)) && !theirs.contains(&idx("Mikaeus + Triskelion")), "{key}");
        }
    }
    assert!(named(&g, P0, GHAVE).is_empty());
}
