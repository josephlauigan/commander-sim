//! Alternative costs for hand spells (engine/altcost.rs): the mechanism, the AI's choice to pay them, the cost
//! increases that still apply (Defense Grid), and the cards that use the slot.

use sim_core::ai::brain;
use sim_core::engine::{altcost, stack};
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::{Ctx, Game, StackItem, StackKind};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P1); // the opponent's turn: P0 answers
    g
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a spell of p's on the stack (cast: out of the hand)
fn on_stack(g: &mut Game, p: PlayerId, name: &str) -> (u32, CardId) {
    let c = hand(g, p, &[name])[0];
    let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(p).hand.remove(i);
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    g.stack.push(StackItem {
        id,
        controller: p,
        card: Some(c),
        ctx: Ctx::default(),
        zone: "hand",
        imp: 6.0,
        aff: vec![],
        generic: true,
        kind: StackKind::Spell,
        name: name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    (id, c)
}

fn countered(g: &Game, id: u32) -> bool {
    g.stack.iter().find(|x| x.id == id).unwrap().countered
}

// ======================================================== the mechanism
#[test]
fn the_slot_is_read_through_the_engine_module() {
    let mut g = game();
    let daze = hand(&mut g, P0, &["Daze"])[0];
    let bolt = hand(&mut g, P0, &["Lightning Bolt"])[0];
    assert!(altcost::slot(&g, daze).is_some());
    assert!(altcost::slot(&g, bolt).is_none());
    assert!(!altcost::available(&g, P0, daze, None), "no Island");
    assert!(!altcost::available(&g, P0, bolt, None), "no slot");
    lands(&mut g, P0, "Island", 1, true);
    assert!(altcost::available(&g, P0, daze, None));
    assert_eq!(altcost::tax(&g, P0, daze), 0);
    assert!(altcost::lost(&g, P0, daze, None) < 1.0, "a replayed Island is cheap");
    // the mana cost {1}{U} can't be paid (the only land is tapped): use the alternative
    assert!(altcost::use_alt(&g, P0, daze, None, 1, "U"));
    lands(&mut g, P0, "Island", 2, false);
    assert!(!altcost::use_alt(&g, P0, daze, None, 1, "U"), "mana available: keep the Island");
}

// ======================================================== Daze and Flare of Denial (counters)
#[test]
fn daze_counters_for_an_island_when_the_ai_has_no_mana() {
    let mut g = game();
    lands(&mut g, P0, "Island", 1, true); // tapped: no mana
    let daze = hand(&mut g, P0, &["Daze"])[0];
    let (id, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(daze));
    assert!(stack::cast_counter_spell(&mut g, P0, daze, id).unwrap());
    assert!(countered(&g, id), "P1 has no {{1}} to pay");
    assert!(in_hand(&g, P0, "Island"), "the Island went back to hand");
    assert!(g.player(P0).lands.is_empty());
}

#[test]
fn flare_of_denial_sacrifices_a_blue_creature() {
    let mut g = game();
    let flare = hand(&mut g, P0, &["Flare of Denial"])[0];
    let m = perm(&mut g, P0, "Mulldrifter");
    let (id, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(flare));
    assert!(stack::cast_counter_spell(&mut g, P0, flare, id).unwrap());
    assert!(countered(&g, id));
    assert!(!g.perm(m).on_bf, "Mulldrifter was sacrificed");
}

#[test]
fn no_alternative_cost_to_pay_means_no_counter() {
    let mut g = game();
    hand(&mut g, P0, &["Flare of Denial", "Daze"]);
    let (_, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(stack::pick_counter(&g, P0, spell), None);
}

#[test]
fn a_cheap_pitch_clears_a_lower_bar_than_a_costly_one() {
    // the threshold bump is 1.5 per card given up: a Daze Island is a fraction of a card, a Commandeer two
    let mut g = game();
    let daze = hand(&mut g, P0, &["Daze"])[0];
    let com = hand(&mut g, P0, &["Commandeer"])[0];
    assert!(altcost::lost(&g, P0, daze, None) < altcost::lost(&g, P0, com, None));
}

// ======================================================== Disrupting Shoal (mana value equals X)
#[test]
fn disrupting_shoal_needs_a_blue_card_of_the_spells_mana_value() {
    let mut g = game();
    let shoal = hand(&mut g, P0, &["Disrupting Shoal", "Counterspell"])[0]; // Counterspell: mana value 2
    let (_, two) = on_stack(&mut g, P1, "Phantasmal Image");
    assert_eq!(stack::pick_counter(&g, P0, two), Some(shoal), "mana value 2: pitched for free");
    let (_, three) = on_stack(&mut g, P1, "Rhystic Study");
    assert_eq!(stack::pick_counter(&g, P0, three), None, "mana value 3: nothing to pitch, no mana");
    // hard cast: {X}{U}{U} with X the spell's mana value
    assert_eq!(stack::counter_cost(&g, shoal, Some(three), Some(P0)), (3, "UU".to_string()));
    lands(&mut g, P0, "Island", 5, false);
    assert_eq!(stack::pick_counter(&g, P0, three), Some(shoal), "five mana pays {{3}}{{U}}{{U}}");
}

#[test]
fn disrupting_shoal_counters_and_exiles_the_pitched_card() {
    let mut g = game();
    let shoal = hand(&mut g, P0, &["Disrupting Shoal", "Counterspell"])[0];
    let (id, _) = on_stack(&mut g, P1, "Phantasmal Image");
    assert!(stack::cast_counter_spell(&mut g, P0, shoal, id).unwrap());
    assert!(countered(&g, id));
    assert!(in_exile(&g, P0, "Counterspell"));
}

// ======================================================== Mindbreak Trap (free mode, any number of spells)
#[test]
fn mindbreak_trap_is_free_after_three_spells_and_exiles_all_of_them() {
    let mut g = game();
    let trap = hand(&mut g, P0, &["Mindbreak Trap"])[0];
    let (a, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    let (b, _) = on_stack(&mut g, P1, "Brainstorm");
    let (c, _) = on_stack(&mut g, P1, "Counterspell");
    assert_eq!(stack::pick_counter(&g, P0, spell), None, "no mana, fewer than three spells counted");
    let st = g.turn_stamp();
    let cs = vec![card(&g, "Lightning Bolt"), card(&g, "Brainstorm"), card(&g, "Counterspell")];
    g.player_mut(P1).turn_casts = Some((st, cs));
    assert!(stack::free_counter(&g, P0, trap));
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(trap));
    assert!(stack::cast_counter_spell(&mut g, P0, trap, a).unwrap());
    assert!(countered(&g, a) && countered(&g, b) && countered(&g, c), "all three are exiled");
}

// ======================================================== Snapback, Pyrokinesis, Sunscour (the AI's own plays)
#[test]
fn snapback_bounces_for_a_blue_card_when_the_ai_has_no_mana() {
    let mut g = game();
    g.active = Some(P0);
    let snap = hand(&mut g, P0, &["Snapback", "Counterspell"])[0];
    let big = perm(&mut g, P1, "Hellkite Tyrant");
    let s = brain::Situation::new(&g, P0);
    let opts = brain::removal_options(&g, P0, &s);
    let o = opts.iter().find(|o| o.label.starts_with("Snapback")).expect("Snapback is offered at no mana");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(big).on_bf);
    assert!(in_exile(&g, P0, "Counterspell"));
    assert!(g.player(P0).gy.contains(&snap));
}

#[test]
fn pyrokinesis_kills_for_a_red_card() {
    let mut g = game();
    g.active = Some(P0);
    hand(&mut g, P0, &["Pyrokinesis", "Lightning Bolt"]);
    let a = perm(&mut g, P1, "Mulldrifter");
    let s = brain::Situation::new(&g, P0);
    let opts = brain::removal_options(&g, P0, &s);
    let o = opts.iter().find(|o| o.label.starts_with("Pyrokinesis")).expect("offered");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(a).on_bf);
    assert!(in_exile(&g, P0, "Lightning Bolt"));
}

#[test]
fn removal_is_not_pitched_when_the_mana_is_there() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 6, false);
    let p = hand(&mut g, P0, &["Pyrokinesis", "Lightning Bolt"])[0];
    assert!(!altcost::use_alt(&g, P0, p, None, 4, "RR"));
    perm(&mut g, P1, "Mulldrifter");
    let s = brain::Situation::new(&g, P0);
    let opts = brain::removal_options(&g, P0, &s);
    let o = opts.iter().find(|o| o.label.starts_with("Pyrokinesis")).expect("offered");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(in_hand(&g, P0, "Lightning Bolt"), "paid {{4}}{{R}}{{R}}, nothing exiled");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "all six Mountains paid the mana cost");
}

#[test]
fn sunscour_exiles_two_white_cards_and_destroys_the_creatures() {
    let mut g = game();
    g.active = Some(P0);
    let sc = hand(&mut g, P0, &["Sunscour", "Swords to Plowshares", "Path to Exile"])[0];
    for _ in 0..3 {
        perm(&mut g, P1, "Hellkite Tyrant");
    }
    let s = brain::Situation::new(&g, P0);
    let opts = brain::wipe_options(&g, P0, &s, false);
    let o = opts.iter().find(|o| o.label == "Sunscour").expect("offered at no mana");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).exile.len(), 2);
    assert!(g.player(P0).gy.contains(&sc));
    assert!(g.player(P1).perms.iter().all(|&m| !g.perm(m).on_bf || !g.is_creature(m)));
}

// ======================================================== Defense Grid taxes alternative and free casts
#[test]
fn defense_grid_taxes_alternative_cost_casts() {
    let mut g = game(); // P1's turn: P0's spells cost {3} more
    perm(&mut g, P0, "Defense Grid");
    lands(&mut g, P0, "Island", 1, true);
    let daze = hand(&mut g, P0, &["Daze"])[0];
    let (id, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(altcost::tax(&g, P0, daze), 3);
    assert!(!altcost::available(&g, P0, daze, None), "the Island is tapped and {{3}} can't be paid");
    assert_eq!(stack::pick_counter(&g, P0, spell), None);
    // three untapped lands pay the tax, and the Island still goes back
    lands(&mut g, P0, "Swamp", 3, false);
    assert!(altcost::available(&g, P0, daze, None));
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(daze));
    assert!(stack::cast_counter_spell(&mut g, P0, daze, id).unwrap());
    assert!(countered(&g, id));
    assert!(in_hand(&g, P0, "Island"));
    let tapped = g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count();
    assert_eq!(tapped, 3, "the three Swamps paid the Defense Grid tax");
}

#[test]
fn defense_grid_taxes_a_force_of_will_pitch() {
    let mut g = game();
    perm(&mut g, P0, "Defense Grid");
    let fow = hand(&mut g, P0, &["Force of Will", "Counterspell"])[0];
    let (_, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(stack::pick_counter(&g, P0, spell), None, "the pitch needs {{3}} for the Grid");
    lands(&mut g, P0, "Swamp", 3, false);
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(fow));
    assert!(stack::pay_counter(&mut g, P0, fow, Some(spell)).unwrap());
    assert!(in_exile(&g, P0, "Counterspell"));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

#[test]
fn without_defense_grid_a_pitch_costs_no_mana() {
    let mut g = game();
    let fow = hand(&mut g, P0, &["Force of Will", "Counterspell"])[0];
    let (_, spell) = on_stack(&mut g, P1, "Lightning Bolt");
    assert_eq!(altcost::tax(&g, P0, fow), 0);
    assert_eq!(stack::pick_counter(&g, P0, spell), Some(fow));
}
