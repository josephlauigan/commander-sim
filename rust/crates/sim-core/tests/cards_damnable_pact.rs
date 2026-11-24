//! Damnable Pact ({X}{B}{B}, sorcery: "Target player draws X cards and loses X life.") as the sheoldred deck's AI plays
//! it (impls/sheoldred.rs `pact_options`) and as it resolves. Seat 0 plays the sheoldred deck, seat 1 the seph deck;
//! the cards are put into hand or onto the battlefield by name, whatever the deck list holds.

use sim_core::engine::{cast, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const SHEO: &str = "Sheoldred, the Apocalypse";
const PACT: &str = "Damnable Pact";

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn game3() -> Game {
    table(&["sheoldred", "seph", "cloud"])
}

fn life(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life
}

fn cast_pact(g: &mut Game, p: PlayerId, face: Option<PlayerId>, x: i32) {
    let c = hand(g, p, &[PACT])[0];
    cast::cast_card(g, p, c, "hand", Ctx { face, x, ..Ctx::default() }).unwrap();
}

fn pact_opts(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let c = card(g, PACT);
    if !g.player(p).hand.contains(&c) {
        hand(g, p, &[PACT]);
    }
    let f = g.registry.get(c).unwrap().hand_options.expect("Damnable Pact has hand options");
    f(g, c, p, Some(false)).unwrap()
}

fn run_plan(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        _ => panic!("not a plan"),
    }
}

fn untapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

fn trim_library(g: &mut Game, p: PlayerId, n: usize) {
    g.player_mut(p).library.truncate(n);
}

fn pact_in_gy(g: &Game, p: PlayerId) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == PACT)
}

// ------------------------------------------------------------------ resolving
#[test]
fn damnable_pact_draws_x_cards_and_loses_x_life() {
    let mut g = game();
    cast_pact(&mut g, P0, Some(P1), 3);
    assert_eq!((g.player(P1).hand.len(), life(&g, P1)), (3, 37));
    assert_eq!(life(&g, P0), 40, "the caster is untouched");
    assert!(pact_in_gy(&g, P0), "a sorcery goes to its owner's graveyard");
}

#[test]
fn damnable_pact_aimed_at_the_caster_draws_and_costs_the_caster() {
    let mut g = game();
    cast_pact(&mut g, P0, Some(P0), 4);
    assert_eq!((g.player(P0).hand.len(), life(&g, P0)), (4, 36));
    let mut g = game();
    cast_pact(&mut g, P0, None, 2);
    assert_eq!((g.player(P0).hand.len(), life(&g, P0)), (2, 38), "no target chosen: the caster");
}

#[test]
fn damnable_pact_with_x_zero_does_nothing() {
    let mut g = game();
    cast_pact(&mut g, P0, Some(P1), 0);
    assert_eq!((g.player(P1).hand.len(), life(&g, P1)), (0, 40));
}

#[test]
fn damnable_pact_draws_fire_sheoldred_three_per_card_in_all() {
    let mut g = game();
    let sheo = card(&g, SHEO);
    zones::enter(&mut g, P0, sheo, zones::Enter { sick: false, ..Default::default() }).unwrap();
    cast_pact(&mut g, P0, Some(P1), 3);
    assert_eq!(g.player(P1).hand.len(), 3);
    assert_eq!(life(&g, P1), 40 - 6 - 3, "Sheoldred takes 2 per card, the spell 1 per card");
}

#[test]
fn damnable_pact_draws_fire_underworld_dreams_and_fate_unraveler() {
    let mut g = game();
    perm(&mut g, P0, "Underworld Dreams");
    perm(&mut g, P0, "Fate Unraveler");
    cast_pact(&mut g, P0, Some(P1), 3);
    assert_eq!(life(&g, P1), 40 - 3 * 2 - 3, "1 per card from each, then 3 from the spell");
}

#[test]
fn damnable_pact_on_a_target_that_runs_out_of_life_during_the_draws_ends_them() {
    let mut g = game3();
    perm(&mut g, P0, SHEO);
    g.player_mut(P1).life = 4;
    cast_pact(&mut g, P0, Some(P1), 3);
    // the spell finishes resolving before state-based actions (CR 704): the player is at 0 or less and out by then
    assert!(life(&g, P1) <= 0, "Sheoldred's two cards are enough: {}", life(&g, P1));
    assert!(!g.player(P1).alive);
    assert!(g.player(PlayerId(2)).alive && life(&g, P0) == 40, "nobody else is touched");
}

#[test]
fn damnable_pact_kills_through_its_own_life_loss() {
    let mut g = game();
    g.player_mut(P1).life = 3;
    cast_pact(&mut g, P0, Some(P1), 3);
    assert_eq!(g.player(P1).hand.len(), 3);
    assert!(!g.player(P1).alive);
}

#[test]
fn damnable_pact_on_a_dead_target_fizzles() {
    let mut g = game3();
    g.player_mut(P1).alive = false;
    cast_pact(&mut g, P0, Some(P1), 2);
    // no legal target: no draws and no life loss for anyone (it is never redirected to the caster), the card still
    // goes to the graveyard
    assert_eq!((g.player(P0).hand.len(), life(&g, P0)), (0, 40));
    assert_eq!(g.player(P1).hand.len(), 0);
    assert!(pact_in_gy(&g, P0));
}

// ------------------------------------------------------------------ the AI's options
#[test]
fn the_ai_aims_damnable_pact_at_a_kill_while_sheoldred_is_out() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P1).life = 9; // 3 cards at 3 each
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draws 3 and loses 9"), "{}", o[0].label);
    assert!(o[0].utility > 8.0, "a kill scores like the X drain's: {}", o[0].utility);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive);
    assert_eq!(untapped(&g, P0), 2, "X = 3 plus {{B}}{{B}} is five of the seven lands");
    assert!(pact_in_gy(&g, P0));
}

#[test]
fn the_ai_casts_damnable_pact_for_a_big_drain_without_a_kill() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1, "X = 5 is 15 life at 3 a card");
    assert!(o[0].label.contains("draws 5 and loses 15"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!((g.player(P1).hand.len(), life(&g, P1)), (5, 25));
    assert_eq!(untapped(&g, P0), 0, "all the spare mana went to X");
}

#[test]
fn the_ai_holds_damnable_pact_when_the_drain_is_small() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false); // X = 3: 9 life, no kill
    let o = pact_opts(&mut g, P0);
    assert!(
        o.iter().all(|x| !x.label.contains(&g.player(P1).name.to_string())),
        "no opponent aimed at: less than 10 life and no kill"
    );
    assert!(o[0].label.starts_with("Damnable Pact (draw "), "the self-draw instead: {}", o[0].label);
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false); // X = 1
    assert!(pact_opts(&mut g, P0).is_empty());
}

#[test]
fn the_ai_counts_underworld_dreams_in_the_drain() {
    let mut g = game();
    perm(&mut g, P0, "Underworld Dreams");
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P1).life = 6; // 2 a card (Dreams 1 + the spell 1): X = 3
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draws 3 and loses 6"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive);
}

#[test]
fn the_ai_does_not_aim_damnable_pact_at_an_opponent_whose_library_is_too_short() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    trim_library(&mut g, P1, 3);
    let o = pact_opts(&mut g, P0);
    assert!(
        o.iter().all(|x| !x.label.contains(&g.player(P1).name.to_string())),
        "X = 5 would deck them; no opponent qualifies"
    );
    assert!(o[0].label.starts_with("Damnable Pact (draw "), "the self-draw instead: {}", o[0].label);
}

#[test]
fn with_sheoldred_out_the_self_draw_gains_life_and_has_a_small_margin() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false); // X = 3, no opponent worth aiming at
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draw 3"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P0), 43, "three draws gain 6 under Sheoldred and the spell costs 3");
    assert_eq!(g.player(P0).hand.len(), 3);
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P0).life = 9; // 9 - 7 = 2: X = 2
    let o = pact_opts(&mut g, P0);
    assert!(o[0].label.contains("draw 2"), "{}", o[0].label);
    g.player_mut(P0).life = 8; // a margin of 1: below X = 2
    assert!(pact_opts(&mut g, P0).is_empty());
}

#[test]
fn the_ai_never_hands_an_opponent_cards_without_a_draw_punisher() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P1).life = 3;
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1, "the self-draw");
    assert!(!o[0].label.contains(&g.player(P1).name.to_string()), "{}", o[0].label);
    assert!(o[0].label.starts_with("Damnable Pact (draw "), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(g.player(P1).hand.len(), 0);
    assert_eq!(life(&g, P1), 3);
    assert_eq!(g.player(P0).hand.len(), 5, "X = 5 cards for the caster");
    assert_eq!(life(&g, P0), 35);
}

#[test]
fn the_self_draw_keeps_life_at_twelve_or_more() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 15; // X = 3 leaves 12
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draw 3"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P0), 12);
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 13; // X = 1 only: below the floor of 2
    assert!(pact_opts(&mut g, P0).is_empty());
}

#[test]
fn the_self_draw_counts_an_opposing_sheoldreds_toll() {
    // their Sheoldred costs each of our draws 2 life, the spell 1: 3 a card, and the floor stays 12
    let mut g = game();
    perm(&mut g, P1, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 18; // (18 - 12) / 3 = 2 cards
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draw 2"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P0), 12, "18 less 2 for the spell and 4 from their Sheoldred");
    let mut g = game();
    perm(&mut g, P1, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 17; // (17 - 12) / 3 = 1: below X = 2
    assert!(pact_opts(&mut g, P0).is_empty());
}

#[test]
fn an_opposing_underworld_dreams_and_bowmasters_count_too() {
    let mut g = game();
    perm(&mut g, P1, "Underworld Dreams");
    perm(&mut g, P1, "Orcish Bowmasters");
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 18; // 1 + 1 + 1 = 3 a card: 2 cards
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draw 2"), "{}", o[0].label);
}

#[test]
fn with_both_sheoldreds_out_the_floor_is_twelve_and_each_card_costs_three() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P1, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).life = 18;
    let o = pact_opts(&mut g, P0);
    assert!(
        o.iter().all(|x| !x.label.contains(&g.player(P1).name.to_string())),
        "their own Sheoldred gains them what ours takes: no drain worth aiming"
    );
    assert!(o[0].label.contains("draw 2"), "{}", o[0].label);
}

#[test]
fn an_opponents_own_sheoldred_cancels_the_toll_in_the_kill_math() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P1, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P1).life = 3; // a card costs them 1 net (2 from ours, 2 gained, 1 for the spell): X = 3
    let o = pact_opts(&mut g, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draws 3"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive, "the spell's own X life finishes them");
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P1, SHEO);
    lands(&mut g, P0, "Swamp", 7, false); // 40 life: 1 a card is nowhere near a drain of 10
    let o = pact_opts(&mut g, P0);
    assert!(o.iter().all(|x| !x.label.contains(&g.player(P1).name.to_string())), "{o:?}");
}

#[test]
fn the_self_draw_takes_at_most_half_the_library_and_six_cards() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 10, false);
    let o = pact_opts(&mut g, P0);
    assert!(o[0].label.contains("draw 6"), "capped at six: {}", o[0].label);
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    hand(&mut g, P0, &[PACT]); // out of the library first: the shuffled library may hold it among the four kept cards
    trim_library(&mut g, P0, 4);
    let o = pact_opts(&mut g, P0);
    assert!(o[0].label.contains("draw 2"), "half of a four card library: {}", o[0].label);
    trim_library(&mut g, P0, 3);
    assert!(pact_opts(&mut g, P0).is_empty(), "one card is below X = 2");
}

#[test]
fn damnable_pact_needs_the_mana_and_the_turn() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false); // X = 1
    assert!(pact_opts(&mut g, P0).is_empty());
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    g.active = Some(P1);
    assert!(pact_opts(&mut g, P0).is_empty(), "a sorcery: only in the caster's own turn");
}

#[test]
fn the_ai_plays_damnable_pact_only_for_a_deck_whose_plan_names_it() {
    let mut g = table(&["karn", "sheoldred"]);
    lands(&mut g, P0, "Swamp", 7, false);
    assert!(pact_opts(&mut g, P0).is_empty(), "the karn plan does not `plays` it");
}

#[test]
fn damnable_pact_has_card_code_and_a_cast_flow_for_a_person() {
    let g = game();
    assert!(g.registry.get(card(&g, PACT)).is_some());
    assert_eq!(
        sim_core::human::specs::spec(PACT).map(|a| a.len()),
        Some(2),
        "a person is asked for a target player, then X"
    );
}
