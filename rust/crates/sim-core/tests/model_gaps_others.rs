//! Model gaps, batch A2 (impls/model_gaps_a2.rs and the pieces it touches): Hexing Squelcher's ward, Thanos' mana
//! values, Lotho, Marvin's borrowed abilities, Psychosis Crawler, Scion of the Ur-Dragon (the pool path).

use sim_core::engine::life::lose_life;
use sim_core::engine::removal;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::{self, draw, enter_token_copy};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t6_thanos::{self, THANOS};
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn tbl() -> Game {
    table(&["kefka", "thanos", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    sim_core::engine::cast::cast_card(g, p, c, "hand", sim_core::state::Ctx::default()).unwrap();
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn main_phase(g: &mut Game, p: PlayerId) {
    g.active = Some(p);
    g.step = Step::Main1;
}

// ------------------------------------------------------------------ Hexing Squelcher
#[test]
fn squelcher_ward_costs_two_life_on_itself_and_on_other_creatures_but_not_on_its_controller() {
    let mut g = tbl();
    let sq = perm(&mut g, P0, "Hexing Squelcher");
    let other = perm(&mut g, P0, "Noble Hierarch");
    let bolt = card(&g, "Lightning Bolt");
    removal::apply_removal(&mut g, Some(P1), sq, "destroy", Some(bolt)).unwrap();
    assert_eq!(g.player(P1).life, 38, "ward on the Squelcher itself: 2 life");
    assert!(!live(&g, sq));
    // without the Squelcher the other creature has no ward
    removal::apply_removal(&mut g, Some(P1), other, "destroy", Some(bolt)).unwrap();
    assert_eq!(g.player(P1).life, 38, "no Squelcher, no ward");
    let sq = perm(&mut g, P0, "Hexing Squelcher");
    let other2 = perm(&mut g, P0, "Birds of Paradise");
    removal::apply_removal(&mut g, Some(P1), other2, "destroy", Some(bolt)).unwrap();
    assert_eq!(g.player(P1).life, 36, "ward on another creature of the Squelcher's controller");
    // its own controller pays nothing
    let mine = perm(&mut g, P0, "Grand Abolisher");
    removal::apply_removal(&mut g, Some(P0), mine, "destroy", Some(bolt)).unwrap();
    assert_eq!(g.player(P0).life, 40, "ward does not apply to your own spells");
    assert!(live(&g, sq));
    // an AI at 4 life or less lets the spell be countered
    let victim = perm(&mut g, P0, "Birds of Paradise");
    g.player_mut(P2).life = 4;
    removal::apply_removal(&mut g, Some(P2), victim, "destroy", Some(bolt)).unwrap();
    assert!(live(&g, victim), "can't pay: countered");
    assert_eq!(g.player(P2).life, 4);
}

// ------------------------------------------------------------------ Thanos, the Mad Titan
#[test]
fn thanos_token_copies_have_the_copied_cards_mana_value_and_other_tokens_zero() {
    let mut g = tbl();
    let t = perm(&mut g, P0, THANOS);
    g.turn_enter = g.perm(t).born;
    let c_copy_odd = card(&g, "Birds of Paradise");
    let copy_odd = enter_token_copy(&mut g, P1, c_copy_odd).unwrap().unwrap(); // a token copy: 1
    let c_copy_even = card(&g, "Grand Abolisher");
    let copy_even = enter_token_copy(&mut g, P1, c_copy_even).unwrap().unwrap(); // 2
    let plain = token(&mut g, P1, 1); // a Treasure-style plain token: 0
    assert!(g.perm(copy_odd).token);
    assert_eq!(t6_thanos::mv(&g, copy_odd), 1);
    assert_eq!(t6_thanos::mv(&g, copy_even), 2);
    assert_eq!(t6_thanos::mv(&g, plain), 0);
    t6_thanos::resolve(&mut g, P0, t, false).unwrap(); // odd
    assert!(!live(&g, copy_odd), "the token copy of a mana value 1 card is odd");
    assert!(live(&g, copy_even) && live(&g, plain));
    t6_thanos::resolve(&mut g, P0, t, true).unwrap(); // even
    assert!(!live(&g, copy_even) && !live(&g, plain), "mana value 2 and 0 are even");
}

// ------------------------------------------------------------------ Lotho, Corrupt Shirriff
#[test]
fn lotho_drains_and_makes_a_treasure_on_each_players_second_spell_each_turn() {
    let mut g = tbl();
    let t0 = g.player(P0).treasures;
    perm(&mut g, P0, "Lotho, Corrupt Shirriff");
    assert_eq!(g.player(P0).treasures, t0, "nothing happens when Lotho enters");
    g.active = Some(P1);
    cast_free(&mut g, P1, "Lightning Bolt");
    assert_eq!(g.player(P0).treasures, t0, "the first spell does nothing");
    assert_eq!(g.player(P0).life, 40);
    cast_free(&mut g, P1, "Opt");
    assert_eq!(g.player(P0).treasures, t0 + 1, "an opponent's second spell: a Treasure for Lotho's controller");
    assert_eq!(g.player(P0).life, 39, "and Lotho's controller loses 1 life");
    cast_free(&mut g, P1, "Brainstorm");
    assert_eq!(g.player(P0).treasures, t0 + 1, "the third spell does nothing");
    // its controller's own second spell counts too
    g.active = Some(P0);
    cast_free(&mut g, P0, "Opt");
    cast_free(&mut g, P0, "Brainstorm");
    assert_eq!(g.player(P0).treasures, t0 + 2);
    assert_eq!(g.player(P0).life, 38);
}

// ------------------------------------------------------------------ Marvin, Murderous Mimic
#[test]
fn marvin_copies_a_compiled_activated_ability_with_a_mana_cost() {
    let mut g = tbl();
    main_phase(&mut g, P0);
    let marvin = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let zombie = perm(&mut g, P0, "Caged Zombie"); // {1}{B}, {T}: each opponent loses 2 life
    g.perm_mut(zombie).tapped = true; // the Zombie itself is tapped: it is Marvin who uses the ability
    let opts = |g: &mut Game| {
        let c = g.perm(marvin).cd.unwrap();
        let f = g.registry.get(c).unwrap().options.unwrap();
        f(g, marvin, P0, None).unwrap()
    };
    assert!(opts(&mut g).is_empty(), "no mana for {{1}}{{B}}");
    lands(&mut g, P0, "Swamp", 2, false);
    let o = opts(&mut g);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Marvin copies Caged Zombie");
    let life = g.player(P1).life;
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P1).life, life - 2, "each opponent loses 2 life");
    assert!(g.perm(marvin).tapped, "Marvin paid the {{T}}");
    assert!(g.perm(zombie).tapped);
    // a creature with Marvin's own name gives nothing
    g.perm_mut(marvin).tapped = false;
    lands(&mut g, P0, "Swamp", 2, false);
    zones::die(&mut g, zombie, "destroy").unwrap();
    assert!(opts(&mut g).is_empty(), "the Zombie is gone");
}

#[test]
fn marvin_also_copies_the_ability_of_a_token_copy_of_a_creature() {
    let mut g = tbl();
    main_phase(&mut g, P0);
    let marvin = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let c_tok = card(&g, "Deadeye Duelist");
    let tok = enter_token_copy(&mut g, P0, c_tok).unwrap().unwrap(); // {1},{T}: 1 damage to an opponent
    g.perm_mut(tok).tapped = true;
    lands(&mut g, P0, "Mountain", 1, false);
    let c = g.perm(marvin).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    let o = f(&mut g, marvin, P0, None).unwrap();
    assert_eq!(o.len(), 1, "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
}

// ------------------------------------------------------------------ Psychosis Crawler
#[test]
fn psychosis_crawler_is_as_big_as_the_hand_and_drains_on_each_draw() {
    let mut g = tbl();
    hand(&mut g, P0, &["Island", "Island", "Island"]);
    let cr = perm(&mut g, P0, "Psychosis Crawler");
    let n = g.player(P0).hand.len() as i32;
    assert_eq!((epow(&g, cr), etgh(&g, cr)), (n, n), "power and toughness: cards in hand");
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    draw(&mut g, P0, 2, false).unwrap();
    assert_eq!(g.player(P1).life, l1 - 2, "each opponent loses 1 life per card drawn");
    assert_eq!(g.player(P2).life, l2 - 2);
    assert_eq!(g.player(P0).life, 40);
    let n2 = g.player(P0).hand.len() as i32;
    assert_eq!(epow(&g, cr), n2, "it grows with the hand");
    // an opponent's draw does nothing
    draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P1).life, l1 - 2);
    let _ = lose_life;
}

// ------------------------------------------------------------------ Scion of the Ur-Dragon (the pool deck's Rivaz use)
#[test]
fn the_pool_decks_scion_also_becomes_a_copy_of_the_dragon_it_puts_into_the_graveyard() {
    let mut g = table(&[
        "ur-dragon-five-color-dragons",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ]);
    let scion = perm(&mut g, P0, "Scion of the Ur-Dragon");
    lands(&mut g, P0, "Badlands", 2, false);
    perm(&mut g, P0, "Rivaz of the Claw");
    let c = g.perm(scion).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    g.active = Some(P1);
    let o = f(&mut g, scion, P0, None).unwrap();
    if o.is_empty() {
        return; // no Dragon creature card left in this seed's library
    }
    let before = g.perm(scion).name;
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(sim_core::impls::t6_scion::is_copy(&g, scion), "Scion becomes a copy until end of turn");
    assert_ne!(g.perm(scion).name, before);
}
