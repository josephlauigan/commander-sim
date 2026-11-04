//! Engine batch E (2026-11-04): Ragavan casting what it exiles (any card type, this turn), and Goldspan Dragon's
//! "becomes the target of a spell" Treasure. Each expectation follows the card's Oracle text.

use sim_core::engine::hooks::run_hook;
use sim_core::engine::mana::cost_of;
use sim_core::hooks::{Call, Event};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn hit_with_ragavan(g: &mut Game, top: &str) -> sim_core::ids::CardId {
    let rag = perm(g, P0, "Ragavan, Nimble Pilferer");
    let c = card(g, top);
    g.player_mut(P1).library.retain(|&x| x != c);
    g.player_mut(P1).library.push(c);
    run_hook(g, rag, Event::CombatDamage, &Call::CombatDamage { p: P0, a: rag, d: P1, dmg: 2 }).unwrap();
    c
}

#[test]
fn ragavan_lets_you_cast_an_exiled_instant_this_turn() {
    let mut g = table(&["sauron", "seph"]);
    let treasures = g.player(P0).treasures;
    let c = hit_with_ragavan(&mut g, "Lightning Bolt");
    assert_eq!(g.player(P0).treasures, treasures + 1, "the Treasure");
    assert!(!g.player(P1).library.contains(&c), "exiled from the top of their library");
    assert!(g.player(P0).hand.contains(&c) && g.player(P0).impulse.contains(&c), "castable this turn");
    assert_eq!(g.player(P0).stolen.get(&c), Some(&P1), "still their card");
    // ordinary mana, not "mana of any type" (Opposition Agent)
    assert!(!g.player(P0).agent_ids.contains(&c));
    assert_eq!(cost_of(&g, P0, c), (0, "R".to_string()));
    // cast: it resolves and goes to its owner's graveyard
    let life = g.player(P1).life;
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    sim_core::engine::stack::settle_stack(&mut g).unwrap();
    assert!(!g.player(P0).hand.contains(&c));
    assert!(g.player(P1).gy.contains(&c) && !g.player(P0).gy.contains(&c), "back to its owner's graveyard");
    assert!(g.player(P1).life <= life, "it was cast");
}

#[test]
fn ragavan_exiled_card_not_cast_goes_back_to_its_owners_exile_at_end_of_turn() {
    let mut g = table(&["sauron", "seph"]);
    let c = hit_with_ragavan(&mut g, "Sol Ring");
    assert!(g.player(P0).hand.contains(&c));
    g.active = Some(P0);
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(!g.player(P0).hand.contains(&c), "no longer castable");
    assert!(g.player(P1).exile.contains(&c) && !g.player(P0).exile.contains(&c), "in its owner's exile");
    assert!(!g.player(P0).stolen.contains_key(&c));
}

#[test]
fn ragavan_exiles_a_land_and_it_cannot_be_cast() {
    let mut g = table(&["sauron", "seph"]);
    let c = hit_with_ragavan(&mut g, "Island");
    assert!(g.player(P1).exile.contains(&c));
    assert!(!g.player(P0).hand.contains(&c));
}

#[test]
fn goldspan_dragon_makes_a_treasure_when_a_spell_targets_it() {
    let mut g = table(&["aurelia-boros-extra-combats", "sauron"]);
    let d = perm(&mut g, P0, "Goldspan Dragon");
    let other = token(&mut g, P0, 2);
    let t0 = g.player(P0).treasures;
    sim_core::impls::card_models3::spell_targeted(&mut g, &Ctx { target: Some(other), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).treasures, t0, "another permanent");
    sim_core::impls::card_models3::spell_targeted(&mut g, &Ctx { target: Some(d), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
    // through a real cast: an opponent's removal spell aimed at the Dragon makes its owner a Treasure
    lands(&mut g, P1, "Plains", 1, false);
    let stp = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    sim_core::engine::cast::cast_card(&mut g, P1, stp, "hand", Ctx { target: Some(d), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 2, "once for that spell");
}
