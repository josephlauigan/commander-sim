//! JD's `thanos` deck: Thanos, the Mad Titan's power-up (impls/t6_thanos.rs) and the three spells that are free while
//! you control a commander (Fierce Guardianship, Deflecting Swat, Deadly Rollick): the commander must be on the
//! battlefield, not in the command zone.

use sim_core::ai::brain;
use sim_core::engine::mana::can_pay;
use sim_core::engine::removal;
use sim_core::engine::stack::free_counter;
use sim_core::engine::values::commander_out;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t6_thanos::{self, THANOS};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["thanos", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

/// Thanos on the battlefield as P0's commander; `fresh`: he entered this turn
fn thanos(g: &mut Game, fresh: bool) -> PermId {
    let t = perm(g, P0, THANOS);
    g.perm_mut(t).is_cmd = true;
    g.player_mut(P0).cmd_in_zone = false;
    if !fresh {
        g.turn_enter = g.perm(t).born; // he was here before this turn began
    }
    assert_eq!(t6_thanos::entered_this_turn(g, t), fresh);
    t
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

#[test]
fn even_destroys_mana_value_two_and_zero_and_spares_thanos() {
    let mut g = tbl();
    let t = thanos(&mut g, false);
    let two = perm(&mut g, P1, "Grand Abolisher"); // 2
    let zero = token(&mut g, P1, 1); // a token: 0, even
    let one = perm(&mut g, P1, "Birds of Paradise"); // 1
    let three = perm(&mut g, P1, "Opposition Agent"); // 3
    let mine = perm(&mut g, P0, "Noble Hierarch"); // 1
    t6_thanos::resolve(&mut g, P0, t, true).unwrap();
    assert!(!live(&g, two) && !live(&g, zero), "even: 2 and 0 are destroyed");
    assert!(live(&g, one) && live(&g, three) && live(&g, mine), "odd ones survive");
    assert!(live(&g, t), "Thanos (mana value 3, but 'each other creature') survives");
    assert_eq!(g.perm(t).plus, 2, "two +1/+1 counters");
}

#[test]
fn odd_destroys_mana_value_one_and_three() {
    let mut g = tbl();
    let t = thanos(&mut g, false);
    let two = perm(&mut g, P1, "Grand Abolisher");
    let zero = token(&mut g, P1, 1);
    let one = perm(&mut g, P1, "Birds of Paradise");
    let three = perm(&mut g, P1, "Opposition Agent");
    t6_thanos::resolve(&mut g, P0, t, false).unwrap();
    assert!(!live(&g, one) && !live(&g, three));
    assert!(live(&g, two) && live(&g, zero) && live(&g, t));
    assert_eq!(g.perm(t).plus, 2);
}

#[test]
fn the_parity_is_chosen_from_the_board_not_fixed() {
    // the opponents' creatures are odd, ours even: odd
    let mut g = tbl();
    let t = thanos(&mut g, false);
    perm(&mut g, P1, "Opposition Agent");
    perm(&mut g, P1, "Spellseeker"); // 3
    perm(&mut g, P0, "Orcish Bowmasters"); // 2
    assert!(!t6_thanos::best_parity(&g, P0, t).0, "odd");
    // the opponents' creatures are even, ours odd: even
    let mut g = tbl();
    let t = thanos(&mut g, false);
    perm(&mut g, P1, "Grand Abolisher");
    perm(&mut g, P1, "Collector Ouphe");
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Noble Hierarch");
    assert!(t6_thanos::best_parity(&g, P0, t).0, "even");
}

#[test]
fn the_cost_is_cug_the_turn_he_entered_and_c_needs_colourless_mana() {
    let mut g = tbl();
    let t = thanos(&mut g, true);
    assert_eq!(t6_thanos::cost(&g, t), (0, "CUG"));
    lands(&mut g, P0, "Tropical Island", 2, false);
    assert!(!can_pay(&g, P0, 0, "CUG", false), "no colourless source: {{C}} isn't generic");
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    assert!(can_pay(&g, P0, 0, "CUG", false));
    // the next turn: the full cost
    let mut g = tbl();
    let t = thanos(&mut g, false);
    assert_eq!(t6_thanos::cost(&g, t), (0, "CWUBRG"));
    lands(&mut g, P0, "City of Brass", 6, false);
    assert!(!can_pay(&g, P0, 0, "CWUBRG", false), "City of Brass makes only coloured mana");
}

#[test]
fn the_power_up_is_used_once_per_game() {
    let mut g = tbl();
    let t = thanos(&mut g, true);
    lands(&mut g, P0, "Ancient Tomb", 2, false);
    lands(&mut g, P0, "Tropical Island", 4, false);
    let ab = perm(&mut g, P1, "Grand Abolisher");
    assert!(!t6_thanos::used(&g, P0));
    assert!(t6_thanos::thanos_go(&mut g, t, P0, 0).unwrap());
    assert!(t6_thanos::used(&g, P0));
    assert!(!live(&g, ab));
    assert_eq!(g.perm(t).plus, 2);
    assert!(!t6_thanos::thanos_go(&mut g, t, P0, 0).unwrap(), "a second activation is refused");
    assert_eq!(g.perm(t).plus, 2);
}

// ======================================================== free while you control a commander
#[test]
fn fierce_guardianship_is_free_only_with_the_commander_on_the_battlefield() {
    let mut g = tbl();
    let fg = card(&g, "Fierce Guardianship");
    assert!(!commander_out(&g, P0) && !free_counter(&g, P0, fg), "Thanos in the command zone: not free");
    let t = perm(&mut g, P0, THANOS);
    assert!(!free_counter(&g, P0, fg), "a Thanos that isn't the commander doesn't count");
    g.perm_mut(t).is_cmd = true;
    assert!(commander_out(&g, P0) && free_counter(&g, P0, fg));
}

#[test]
fn deadly_rollick_is_offered_free_with_the_commander_out() {
    let mut g = tbl();
    hand(&mut g, P0, &["Deadly Rollick"]);
    perm(&mut g, P1, "Hellkite Tyrant");
    let t = perm(&mut g, P0, THANOS); // no mana at all
    let rollick = |g: &Game| {
        brain::removal_options(g, P0, &brain::Situation::new(g, P0)).iter().any(|o| o.label.contains("Deadly Rollick"))
    };
    assert!(!rollick(&g), "not the commander: Rollick costs {{3}}{{B}}");
    g.perm_mut(t).is_cmd = true;
    assert!(rollick(&g), "with the commander out it is free");
}

#[test]
fn deflecting_swat_is_free_with_the_commander_out() {
    let mut g = tbl();
    let t = thanos(&mut g, false); // no mana at all
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Deflecting Swat"]);
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), t, "exile", Some(stp)).unwrap();
    assert!(live(&g, t), "Thanos survives");
    assert!(!live(&g, theirs), "the Swords hit the caster's own creature");
    assert!(g.player(P0).gy.contains(&card(&g, "Deflecting Swat")));
}
