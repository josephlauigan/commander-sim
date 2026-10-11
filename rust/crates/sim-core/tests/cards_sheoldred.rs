//! The sheoldred deck's cards (impls/sheoldred.rs) on hand-built positions. Seat 0 plays the sheoldred deck, seat 1
//! the seph deck.

use sim_core::engine::life::gain;
use sim_core::engine::{cast, turn, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SHEO: &str = "Sheoldred, the Apocalypse";

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn game3() -> Game {
    table(&["sheoldred", "seph", "veyran"])
}

fn life(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn card_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn perm_opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run_plan(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        _ => panic!("not a plan"),
    }
}

// ------------------------------------------------------------------ Sheoldred herself
#[test]
fn sheoldred_drains_an_opponent_on_their_draw_step_draw() {
    let mut g = game3();
    perm(&mut g, P0, SHEO);
    zones::draw(&mut g, P1, 1, true).unwrap();
    assert_eq!(life(&g, P1), 38);
    assert_eq!(life(&g, P2), 40, "only the player who drew");
    assert_eq!(life(&g, P0), 40, "her controller gains only on their own draws");
}

#[test]
fn sheoldred_drains_on_every_extra_draw_of_an_opponent() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    zones::draw(&mut g, P1, 3, false).unwrap();
    assert_eq!(life(&g, P1), 34, "three cards, 2 each");
    // a wheel: seven more
    zones::draw(&mut g, P1, 7, false).unwrap();
    assert_eq!(life(&g, P1), 20);
}

#[test]
fn sheoldred_gains_two_for_every_card_her_controller_draws() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    zones::draw(&mut g, P0, 1, true).unwrap();
    assert_eq!(life(&g, P0), 42, "the draw step");
    zones::draw(&mut g, P0, 3, false).unwrap();
    assert_eq!(life(&g, P0), 48, "three extra draws");
    assert_eq!(life(&g, P1), 40);
}

#[test]
fn sheoldred_triggers_on_the_real_draw_step_of_an_opponent() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    g.active = Some(P1);
    turn::step_start(&mut g, P1).unwrap();
    assert!(life(&g, P1) <= 38, "the draw step drained them: {}", life(&g, P1));
}

#[test]
fn without_sheoldred_nobody_loses_life_to_a_draw() {
    let mut g = game();
    zones::draw(&mut g, P1, 2, false).unwrap();
    zones::draw(&mut g, P0, 2, false).unwrap();
    assert_eq!((life(&g, P0), life(&g, P1)), (40, 40));
}

// ------------------------------------------------------------------ Cost of Brilliance, Vault Plunderer
#[test]
fn cost_of_brilliance_draws_two_and_puts_a_counter() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let t = token(&mut g, P0, 1);
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Cost of Brilliance", Ctx { face: Some(P0), host: Some(t), ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), n + 2);
    assert_eq!(life(&g, P0), 38);
    assert_eq!(values::epow(&g, t), 2, "a +1/+1 counter");
}

#[test]
fn cost_of_brilliance_aimed_at_an_opponent_with_sheoldred_drains_six() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false);
    cast_spell(&mut g, P0, "Cost of Brilliance", Ctx { face: Some(P1), ..Ctx::default() });
    assert_eq!(life(&g, P1), 34, "two draws (4) and the spell's 2");
    assert_eq!(g.player(P1).hand.len(), 2);
}

#[test]
fn the_ai_aims_cost_of_brilliance_at_an_opponent_while_sheoldred_is_out_and_at_itself_otherwise() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    token(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Cost of Brilliance"])[0];
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    run_plan(&mut g, P0, &o[0]);
    assert_eq!(life(&g, P0), 38, "no Sheoldred: it draws itself");
    assert_eq!(life(&g, P1), 40);

    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false);
    token(&mut g, P0, 1);
    g.player_mut(P1).life = 30;
    let c = hand(&mut g, P0, &["Cost of Brilliance"])[0];
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    run_plan(&mut g, P0, &o[0]);
    assert_eq!(life(&g, P1), 24, "with Sheoldred it hands an opponent the cards");
    assert_eq!(life(&g, P0), 40);
}

#[test]
fn vault_plunderer_targets_you_normally_and_an_opponent_with_sheoldred() {
    let mut g = game();
    perm(&mut g, P0, "Vault Plunderer");
    assert_eq!(g.player(P0).hand.len(), 1, "you draw");
    assert_eq!(life(&g, P0), 39);

    let mut g = game3();
    perm(&mut g, P0, SHEO);
    g.player_mut(P2).life = 25;
    perm(&mut g, P0, "Vault Plunderer");
    assert_eq!(g.player(P2).hand.len(), 1, "the lowest-life opponent draws");
    assert_eq!(life(&g, P2), 22, "2 from Sheoldred, 1 from the Plunderer");
    assert_eq!(g.player(P0).hand.len(), 0);
    assert_eq!(life(&g, P1), 40);
}

// ------------------------------------------------------------------ the cards that came with the retired decks
#[test]
fn melancholic_poet_drains_for_a_spell_that_targets_a_creature() {
    let mut g = game();
    perm(&mut g, P0, "Melancholic Poet");
    lands(&mut g, P0, "Swamp", 3, false);
    let t = token(&mut g, P1, 2);
    cast_spell(&mut g, P0, "Terror", Ctx { target: Some(t), ..Ctx::default() });
    assert_eq!(life(&g, P1), 39);
    assert_eq!(life(&g, P0), 41);
}

#[test]
fn foolish_fate_destroys_and_infusion_costs_three_life() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 6, false);
    let t = token(&mut g, P1, 2);
    cast_spell(&mut g, P0, "Foolish Fate", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(life(&g, P1), 40, "no lifegain this turn");
    gain(&mut g, P0, 1).unwrap();
    let t = token(&mut g, P1, 2);
    cast_spell(&mut g, P0, "Foolish Fate", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(life(&g, P1), 37, "infusion");
}

#[test]
fn canyon_crawler_makes_a_food_and_swampcycles() {
    let mut g = game();
    perm(&mut g, P0, "Canyon Crawler");
    assert_eq!(g.player(P0).foods, 1);
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Canyon Crawler"])[0];
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "no land in hand and few lands out: cycle it");
    run_plan(&mut g, P0, &o[0]);
    assert!(in_gy(&g, P0, "Canyon Crawler"));
    assert!(in_hand(&g, P0, "Swamp"));
}

#[test]
fn mabel_removes_counters_from_the_best_target() {
    let mut g = game();
    let t = token(&mut g, P1, 2);
    g.perm_mut(t).plus = 3;
    perm(&mut g, P0, "Mabel, Bitter Recluse");
    assert_eq!(g.perm(t).plus, 0, "up to three counters removed");
}

#[test]
fn seymour_flux_pays_life_to_draw_and_grow() {
    let mut g = game();
    let s = perm(&mut g, P0, "Seymour Flux");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(life(&g, P0), 39);
    assert_eq!(g.player(P0).hand.len(), 1);
    assert_eq!(values::epow(&g, s), 6);
}

#[test]
fn al_bhed_salvagers_drain_when_your_creatures_and_artifacts_die() {
    let mut g = game();
    perm(&mut g, P0, "Al Bhed Salvagers");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!((life(&g, P1), life(&g, P0)), (39, 41));
    let s = on_bf(&g, P0, "Al Bhed Salvagers").unwrap();
    zones::die(&mut g, s, "sba".into()).unwrap();
    assert_eq!((life(&g, P1), life(&g, P0)), (38, 42), "its own death too");
}

#[test]
fn necrosquito_grows_when_another_creature_of_yours_dies() {
    let mut g = game();
    let n = perm(&mut g, P0, "Necrosquito");
    assert_eq!(values::epow(&g, n), 2, "two oil counters");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!(values::epow(&g, n), 3);
    let theirs = token(&mut g, P1, 1);
    zones::die(&mut g, theirs, "sba".into()).unwrap();
    assert_eq!(values::epow(&g, n), 3, "only your own");
}

#[test]
fn callous_inspector_pings_you_and_leaves_a_clue() {
    let mut g = game();
    let m = perm(&mut g, P0, "Callous Inspector");
    zones::die(&mut g, m, "sba".into()).unwrap();
    assert_eq!(life(&g, P0), 39);
    assert_eq!(g.player(P0).clues, 1);
}

#[test]
fn apex_witchstalker_gains_two_entering_and_dying() {
    let mut g = game();
    let m = perm(&mut g, P0, "Apex Witchstalker");
    assert_eq!(life(&g, P0), 42);
    zones::die(&mut g, m, "sba".into()).unwrap();
    assert_eq!(life(&g, P0), 44);
}

#[test]
fn gravblade_heavy_is_a_4_4_deathtouch_with_an_artifact() {
    let mut g = game();
    let m = perm(&mut g, P0, "Gravblade Heavy");
    assert_eq!(values::epow(&g, m), 3);
    assert!(!sim_core::engine::combat::kw(&g, m, "deathtouch"));
    perm(&mut g, P0, "Arcane Signet");
    assert_eq!(values::epow(&g, m), 4);
    assert!(sim_core::engine::combat::kw(&g, m, "deathtouch"));
}

#[test]
fn ulna_alley_shopkeep_gets_plus_two_after_lifegain() {
    let mut g = game();
    let m = perm(&mut g, P0, "Ulna Alley Shopkeep");
    let base = values::epow(&g, m);
    gain(&mut g, P0, 1).unwrap();
    assert_eq!(values::epow(&g, m), base + 2);
}

#[test]
fn restless_bloodseeker_makes_blood_flips_and_drains() {
    let mut g = game();
    let m = perm(&mut g, P0, "Restless Bloodseeker // Bloodsoaked Reveler");
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).bloods, 0, "no lifegain, no Blood");
    gain(&mut g, P0, 1).unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).bloods, 1);
    g.player_mut(P0).bloods = 2;
    let o = perm_opts(&mut g, m, P0, Some(false));
    let flip = o.iter().find(|o| o.label.contains("transform")).expect("transform offered");
    run_plan(&mut g, P0, flip);
    assert_eq!(g.player(P0).bloods, 0, "two Blood sacrificed");
    assert_eq!(values::epow(&g, m), 3, "Bloodsoaked Reveler is 3/3");
    lands(&mut g, P0, "Swamp", 5, false);
    let o = perm_opts(&mut g, m, P0, Some(true));
    let drain = o.iter().find(|o| o.label.contains("drain")).expect("the {4}{B} drain");
    let (a, b) = (life(&g, P0), life(&g, P1));
    run_plan(&mut g, P0, drain);
    assert_eq!((life(&g, P0), life(&g, P1)), (a + 2, b - 2));
}

#[test]
fn end_of_the_hunt_exiles_the_opponents_greatest_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let big = perm(&mut g, P1, "Seymour Flux");
    let small = token(&mut g, P1, 1);
    cast_spell(&mut g, P0, "End of the Hunt", Ctx::default());
    assert!(!g.perm(big).on_bf);
    assert!(g.perm(small).on_bf);
}

#[test]
fn phyrexian_delver_returns_the_best_creature_card_and_costs_its_mana_value() {
    let mut g = game();
    let c = take(&mut g, P0, "Seymour Flux");
    g.player_mut(P0).gy.push(c);
    perm(&mut g, P0, "Phyrexian Delver");
    assert!(on_bf(&g, P0, "Seymour Flux").is_some());
    assert_eq!(life(&g, P0), 35, "Seymour Flux has mana value 5");
}

#[test]
fn soul_enervation_shrinks_a_creature_and_drains_when_creature_cards_leave_the_graveyard() {
    let mut g = game();
    let c = take(&mut g, P0, "Burglar Rat");
    g.player_mut(P0).gy.push(c);
    let victim = perm(&mut g, P1, "Apex Witchstalker");
    lands(&mut g, P0, "Swamp", 4, false);
    cast_spell(&mut g, P0, "Soul Enervation", Ctx::default());
    assert!(!g.perm(victim).on_bf, "-4/-4 kills a 3/3");
    let before = (life(&g, P0), life(&g, P1));
    let rat = g.player(P0).gy.iter().position(|&x| x == c).unwrap();
    g.player_mut(P0).gy.remove(rat);
    let _ = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    sim_core::engine::hooks::fire_sba(&mut g).unwrap();
    assert_eq!((life(&g, P0), life(&g, P1)), (before.0 + 1, before.1 - 1));
}

#[test]
fn perigee_beckoner_pumps_another_creature_and_brings_it_back_when_it_dies() {
    let mut g = game();
    let t = perm(&mut g, P0, "Gravblade Heavy");
    let before = values::epow(&g, t);
    perm(&mut g, P0, "Perigee Beckoner");
    assert_eq!(values::epow(&g, t), before + 2);
    zones::die(&mut g, t, "sba".into()).unwrap();
    let back = on_bf(&g, P0, "Gravblade Heavy").expect("it returned");
    assert!(g.perm(back).tapped, "tapped");
    assert!(!in_gy(&g, P0, "Gravblade Heavy"));
}
