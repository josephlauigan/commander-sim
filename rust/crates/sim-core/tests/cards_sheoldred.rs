//! The sheoldred deck's cards (impls/sheoldred.rs) on hand-built positions. Seat 0 plays the sheoldred deck, seat 1
//! the seph deck.

use sim_core::engine::life::gain;
use sim_core::engine::{cast, turn, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
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

// ------------------------------------------------------------------ the last eight cards
#[test]
fn locust_spray_kills_a_one_toughness_creature_and_otherwise_cycles() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Locust Spray"])[0];
    let t = perm(&mut g, P1, "Moseo, Vein's New Dean"); // a 2/1 flier
    let o = card_opts(&mut g, c, P0, Some(false));
    assert!(o[0].label.contains("Locust Spray ->"), "{:?}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.perm(t).on_bf, "the 1/1 died to -1/-1");
    assert!(in_gy(&g, P0, "Locust Spray"));
    // a 3/3 is not worth it: the spray is cycled after combat
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Locust Spray"])[0];
    token(&mut g, P1, 3);
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "no cycling before the plays are made");
    let o = card_opts(&mut g, c, P0, Some(true));
    assert!(o[0].label.contains("cycling"));
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(in_gy(&g, P0, "Locust Spray"));
    assert_eq!(g.player(P0).hand.len(), n, "discarded one, drew one");
}

#[test]
fn locust_spray_shrinks_a_big_creature_until_end_of_turn() {
    let mut g = game();
    let t = token(&mut g, P1, 3);
    cast_spell(&mut g, P0, "Locust Spray", Ctx { target: Some(t), ..Ctx::default() });
    assert_eq!(values::epow(&g, t), 2);
    assert!(g.perm(t).on_bf);
}

#[test]
fn ribskiff_draws_crews_with_idle_creatures_and_gives_toxic() {
    use sim_core::hooks::{Call, Event};
    let mut g = game();
    let n = g.player(P0).hand.len();
    let m = perm(&mut g, P0, "Ribskiff");
    assert_eq!(g.player(P0).hand.len(), n + 1, "draws a card entering");
    assert!(!g.is_creature(m), "a Vehicle is not a creature until crewed");
    // a creature that is summoning sick (power 3) crews it
    let crew = token(&mut g, P0, 3);
    g.perm_mut(crew).sick = true;
    sim_core::engine::hooks::fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(g.is_creature(m), "crewed: an artifact creature");
    assert!(g.perm(crew).tapped);
    assert_eq!((values::epow(&g, m), values::etgh(&g, m)), (4, 4));
    // toxic 2 (compiled), with the combat damage
    sim_core::impls::tymna_cards::toxic_hit(&mut g, m, P1);
    assert_eq!(g.player(P1).poison, 2);
}

#[test]
fn ribskiff_never_crews_with_sheoldred() {
    use sim_core::hooks::{Call, Event};
    let mut g = game();
    let m = perm(&mut g, P0, "Ribskiff");
    let s = perm(&mut g, P0, SHEO);
    g.perm_mut(s).sick = true;
    sim_core::engine::hooks::fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(!g.is_creature(m));
    assert!(!g.perm(s).tapped);
}

#[test]
fn tome_of_legends_pages_and_draws() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let tome = perm(&mut g, P0, "Tome of Legends");
    let pages = |g: &Game| g.perm(tome).data.int(sim_core::state::DataKey::Counters);
    assert_eq!(pages(&g), 1, "enters with a page counter");
    // a non-commander entering adds nothing; a commander entering or attacking does
    perm(&mut g, P0, "Hullcarver");
    assert_eq!(pages(&g), 1);
    let cmd = perm(&mut g, P0, SHEO);
    g.perm_mut(cmd).is_cmd = true;
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Attack,
        sim_core::hooks::Call::Attack { p: P0, atk: vec![cmd], d: P1 },
    )
    .unwrap();
    assert_eq!(pages(&g), 2, "the commander attacked");
    let o = perm_opts(&mut g, tome, P0, Some(false));
    assert!(!o.is_empty());
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(pages(&g), 1);
    assert!(g.perm(tome).tapped);
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn diary_of_dreams_counts_spells_and_gets_cheaper() {
    let mut g = game();
    let diary = perm(&mut g, P0, "Diary of Dreams");
    let pages = |g: &Game| g.perm(diary).data.int(sim_core::state::DataKey::Counters);
    let tg = token(&mut g, P1, 3);
    cast_spell(&mut g, P0, "Locust Spray", Ctx { target: Some(tg), ..Ctx::default() });
    assert_eq!(pages(&g), 1, "an instant adds a page");
    perm(&mut g, P0, "Hullcarver");
    assert_eq!(pages(&g), 1, "a creature spell does not");
    assert!(perm_opts(&mut g, diary, P0, Some(false)).is_empty(), "no mana: the draw costs 4");
    lands(&mut g, P0, "Swamp", 4, false);
    let o = perm_opts(&mut g, diary, P0, Some(false));
    assert_eq!(o.len(), 1, "{{5}} minus one page is 4");
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.perm(diary).tapped);
}

#[test]
fn leech_collector_prepares_on_the_first_lifegain_and_bloodletting_drains() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let m = perm(&mut g, P0, "Leech Collector // Bloodletting");
    let prepared = |g: &Game| g.perm(m).data.truthy(sim_core::state::DataKey::Prepared);
    assert!(!prepared(&g));
    assert!(perm_opts(&mut g, m, P0, Some(false)).is_empty());
    gain(&mut g, P0, 1).unwrap();
    assert!(prepared(&g));
    let o = perm_opts(&mut g, m, P0, Some(false));
    assert_eq!(o.len(), 1);
    let b = life(&g, P1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P1), b - 2);
    assert!(!prepared(&g), "casting the copy unprepares it");
    gain(&mut g, P0, 1).unwrap();
    assert!(!prepared(&g), "only the first lifegain each turn prepares it");
}

#[test]
fn moseo_returns_a_creature_up_to_the_life_gained() {
    let mut g = game();
    perm(&mut g, P0, "Moseo, Vein's New Dean");
    let big = take(&mut g, P0, "Gravblade Heavy"); // mana value 4
    g.player_mut(P0).gy.push(big);
    turn::end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Gravblade Heavy").is_none(), "no lifegain, no return");
    gain(&mut g, P0, 2).unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Gravblade Heavy").is_none(), "2 life does not reach mana value 4");
    gain(&mut g, P0, 2).unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Gravblade Heavy").is_some(), "4 life returns it");
}

#[test]
fn lethal_throwdown_pays_its_sacrifice_and_draws_for_a_modified_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Lethal Throwdown"])[0];
    let target = perm(&mut g, P1, "Gravblade Heavy");
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "nothing to sacrifice");
    let fodder = token(&mut g, P0, 1);
    g.perm_mut(fodder).plus = 1; // a +1/+1 counter: a modified creature
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "{:?}", o.iter().map(|o| &o.label).collect::<Vec<_>>());
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.perm(fodder).on_bf, "the sacrifice is paid");
    assert!(!g.perm(target).on_bf, "the target is destroyed");
    assert_eq!(g.player(P0).hand.len(), n, "cast one, drew one for the modified creature");
}

#[test]
fn lethal_throwdown_with_an_unmodified_creature_draws_nothing() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Lethal Throwdown"])[0];
    let target = perm(&mut g, P1, "Gravblade Heavy");
    let fodder = token(&mut g, P0, 1);
    let o = card_opts(&mut g, c, P0, Some(false));
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.perm(fodder).on_bf && !g.perm(target).on_bf);
    assert_eq!(g.player(P0).hand.len(), n - 1);
}

// ------------------------------------------------------------------ cards handled by tags: they really work in a game
fn removal(g: &mut Game, name: &str, target: PermId) -> bool {
    let c = hand(g, P0, &[name])[0];
    sim_core::ai::brain::cast_removal(g, P0, c, &[target], 0, None).unwrap()
}

#[test]
fn hullcarver_has_deathtouch() {
    let mut g = game();
    let m = perm(&mut g, P0, "Hullcarver");
    assert!(sim_core::dsl::has_kw(&g, m, "deathtouch") || g.perm(m).dt);
}

#[test]
fn multiply_by_zero_kills_a_creature_without_counters() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P1, "Gravblade Heavy");
    assert!(removal(&mut g, "Multiply by Zero", t));
    assert!(!g.perm(t).on_bf, "base 0/0 and no counters: it dies");
    assert!(in_gy(&g, P0, "Multiply by Zero"));
}

#[test]
fn depressurize_kills_a_creature_of_power_three_or_less_only() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let small = token(&mut g, P1, 3);
    assert!(removal(&mut g, "Depressurize", small));
    assert!(!g.perm(small).on_bf, "power 3 - 3 = 0: destroyed");
    let big = perm(&mut g, P1, "Apex Witchstalker");
    let c = hand(&mut g, P0, &["Depressurize"])[0];
    assert!(
        !sim_core::engine::removal::legal_targets(&g, P0, "destroy", "c", false, Some(c)).contains(&big),
        "a power-4 creature is not a target worth the spell"
    );
}

#[test]
fn overkill_kills_any_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let t = perm(&mut g, P1, "Gravblade Heavy");
    assert!(removal(&mut g, "Overkill", t));
    assert!(!g.perm(t).on_bf);
}

#[test]
fn lash_of_the_balrog_sacrifices_a_creature_or_pays_four() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let t = perm(&mut g, P1, "Gravblade Heavy");
    let f = token(&mut g, P0, 1);
    assert!(removal(&mut g, "Lash of the Balrog", t));
    assert!(!g.perm(t).on_bf, "destroyed");
    assert!(!g.perm(f).on_bf, "the creature was sacrificed to pay");
    // no creature to sacrifice: it needs {4} more, and one Swamp cannot pay
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let t = perm(&mut g, P1, "Gravblade Heavy");
    assert!(!removal(&mut g, "Lash of the Balrog", t));
    assert!(g.perm(t).on_bf);
    lands(&mut g, P0, "Swamp", 4, false);
    assert!(removal(&mut g, "Lash of the Balrog", t));
    assert!(!g.perm(t).on_bf);
}

#[test]
fn deadly_dispute_sacrifices_draws_two_and_makes_a_treasure() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    let f = token(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Deadly Dispute"])[0];
    let s = sim_core::ai::brain::Situation::new(&g, P0);
    let o = sim_core::ai::pool::spell_options(&g, P0, &s, Some(false));
    let o = o.iter().find(|o| o.label == "Deadly Dispute").expect("the AI offers Deadly Dispute");
    let n = g.player(P0).hand.len();
    assert!(run_plan(&mut g, P0, o));
    assert!(!g.perm(f).on_bf);
    assert!(!in_hand(&g, P0, "Deadly Dispute") && in_gy(&g, P0, "Deadly Dispute"));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2, "drew two");
    assert_eq!(g.player(P0).treasures, 1);
    let _ = c;
}

// ------------------------------------------------------------------ Part 2: Memory Jar in a sheoldred game
#[test]
fn memory_jar_is_cracked_at_once_with_sheoldred_out_and_by_its_own_rule_without() {
    let mut g = game();
    g.active = Some(P0);
    let jar = perm(&mut g, P0, "Memory Jar");
    hand(&mut g, P0, &["Hullcarver", "Locust Spray", "Ribskiff", "Diary of Dreams", "Tome of Legends"]);
    assert!(perm_opts(&mut g, jar, P0, Some(false)).is_empty(), "five cards and no mana: Memory Jar's own rule says no");
    perm(&mut g, P0, SHEO);
    let o = perm_opts(&mut g, jar, P0, Some(false));
    assert_eq!(o.len(), 1, "with Sheoldred out it is cracked whatever the hand");
    let (a, b) = (life(&g, P0), life(&g, P1));
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.perm(jar).on_bf);
    assert_eq!(life(&g, P1), b - 14, "seven cards at 2 life each");
    assert_eq!(life(&g, P0), a + 14);
}

#[test]
fn toxic_deluge_takes_x_so_sheoldred_survives_and_pays_the_life() {
    let mut g = game();
    let sheo = perm(&mut g, P0, SHEO);
    let big = perm(&mut g, P1, "Gravblade Heavy");
    g.perm_mut(big).tgh = 9;
    let small = perm(&mut g, P1, "Hullcarver");
    let a = life(&g, P0);
    cast_spell(&mut g, P0, "Toxic Deluge", Ctx::default());
    assert!(g.perm(sheo).on_bf, "Sheoldred (toughness 5) survives X = 4");
    assert!(!g.perm(small).on_bf, "the small creature dies");
    assert!(g.perm(big).on_bf, "the 9-toughness one is out of reach at X = 4");
    assert_eq!(life(&g, P0), a - 4, "X life is paid");
}

#[test]
fn toxic_deluge_without_sheoldred_still_takes_the_biggest_toughness() {
    let mut g = game();
    let big = perm(&mut g, P1, "Gravblade Heavy");
    let a = life(&g, P0);
    cast_spell(&mut g, P0, "Toxic Deluge", Ctx::default());
    assert!(!g.perm(big).on_bf);
    assert_eq!(life(&g, P0), a - 4);
}

#[test]
fn bitter_triumph_destroys_and_costs_three_life() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P1, "Gravblade Heavy");
    let a = life(&g, P0);
    assert!(removal(&mut g, "Bitter Triumph", t));
    assert!(!g.perm(t).on_bf);
    assert_eq!(life(&g, P0), a - 3);
}

#[test]
fn korlash_is_as_big_as_your_swamps_and_regenerates() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    let k = perm(&mut g, P0, "Korlash, Heir to Blackblade");
    assert_eq!((values::epow(&g, k), values::etgh(&g, k)), (4, 4));
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), k, "destroy", None).unwrap();
    assert!(g.perm(k).on_bf, "regenerated with {{1}}{{B}} open");
    assert!(g.perm(k).tapped);
}

// ------------------------------------------------------------------ the AI plan
#[test]
fn the_ai_equips_the_helm_and_the_plate_on_sheoldred_first() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Swamp", 3, false);
    let sheo = perm(&mut g, P0, SHEO);
    let helm = perm(&mut g, P0, "Champion's Helm");
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    let eq = o.iter().find(|o| o.label.contains("Champion's Helm")).expect("equip offered");
    assert!(eq.utility >= 8.0, "ahead of everything else");
    assert!(run_plan(&mut g, P0, eq));
    assert_eq!(g.perm(helm).attached, Some(sheo));
    assert!(values::untargetable(&g, sheo), "legendary: hexproof from the Helm");
    let plate = perm(&mut g, P0, "Darksteel Plate");
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    let eq = o.iter().find(|o| o.label.contains("Darksteel Plate")).expect("plate offered");
    assert!(run_plan(&mut g, P0, eq));
    assert_eq!(g.perm(plate).attached, Some(sheo));
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), sheo, "destroy", None).unwrap();
    assert!(g.perm(sheo).on_bf, "indestructible with the Plate");
}

#[test]
fn a_protection_instant_answers_destroy_removal_aimed_at_sheoldred_but_not_exile() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let sheo = perm(&mut g, P0, SHEO);
    hand(&mut g, P0, &["Dark Endurance"]);
    let ex = sim_core::impls::sheoldred::sheoldred_protect(&mut g, P0, sheo, "exile", Some(P1), None).unwrap();
    assert!(!ex, "indestructible does not stop exile");
    assert!(in_hand(&g, P0, "Dark Endurance"));
    let ok = sim_core::impls::sheoldred::sheoldred_protect(&mut g, P0, sheo, "destroy", Some(P1), None).unwrap();
    assert!(ok);
    assert!(in_gy(&g, P0, "Dark Endurance"));
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), sheo, "destroy", None).unwrap();
    assert!(g.perm(sheo).on_bf, "indestructible until end of turn");
    assert_eq!(values::epow(&g, sheo), 6, "Dark Endurance gives +2/+0");
}

#[test]
fn a_real_destroy_spell_at_sheoldred_is_answered_in_play() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let sheo = perm(&mut g, P0, SHEO);
    hand(&mut g, P0, &["Masterful Flourish"]);
    lands(&mut g, P1, "Swamp", 2, false);
    let c = hand(&mut g, P1, &["Terror"])[0];
    cast::cast_card(&mut g, P1, c, "hand", Ctx { target: Some(sheo), ..Ctx::default() }).unwrap();
    assert!(g.perm(sheo).on_bf, "Masterful Flourish saved her");
    assert!(in_gy(&g, P0, "Masterful Flourish"));
}

#[test]
fn the_ai_holds_mana_for_a_protection_instant_and_rocks_come_before_creatures() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Battle-Rage Blessing"]);
    let (c, v) = sim_core::impls::sheoldred::sheoldred_hold(&g, P0);
    assert!(c.is_some() && v > 1.0);
    let mut g = game();
    let sig = hand(&mut g, P0, &["Arcane Signet", "Hullcarver"]);
    let p = |g: &Game, c| sim_core::impls::sheoldred::sheoldred_prio(g, P0, c);
    assert!(p(&g, sig[0]) > p(&g, sig[1]), "the rock goes first while Sheoldred is not out");
}

// ------------------------------------------------------------------ the upgrade package (swap-tested)
const SEIZAN: &str = "Seizan, Perverter of Truth";

#[test]
fn seizan_makes_each_player_lose_two_and_draw_two_at_their_upkeep() {
    let mut g = game();
    perm(&mut g, P0, SEIZAN);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!((life(&g, P0), g.player(P0).hand.len()), (38, 2), "the owner too");
    turn::upkeep(&mut g, P1).unwrap();
    assert_eq!((life(&g, P1), g.player(P1).hand.len()), (38, 2));
}

#[test]
fn seizans_draws_fire_sheoldred_for_both_sides_and_underworld_dreams() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, SEIZAN);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(life(&g, P0), 40 - 2 + 4, "loses 2 to Seizan, gains 2 per draw from her own Sheoldred");
    turn::upkeep(&mut g, P1).unwrap();
    assert_eq!(life(&g, P1), 40 - 2 - 4, "Seizan's 2 and Sheoldred's 2 per card drawn");

    let mut g = game();
    perm(&mut g, P0, "Underworld Dreams");
    perm(&mut g, P0, SEIZAN);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(life(&g, P0), 38, "the owner's own draws are not an opponent's");
    turn::upkeep(&mut g, P1).unwrap();
    assert_eq!(life(&g, P1), 40 - 2 - 2, "Seizan's 2 and 1 damage per card");
}

#[test]
fn underworld_dreams_pings_an_opponent_per_draw_but_not_its_owner() {
    let mut g = game();
    perm(&mut g, P0, "Underworld Dreams");
    zones::draw(&mut g, P0, 3, false).unwrap();
    assert_eq!(life(&g, P0), 40);
    zones::draw(&mut g, P1, 3, false).unwrap();
    assert_eq!(life(&g, P1), 37);
    assert_eq!(g.player(P1).last_kind, "triggers", "damage, so prevention can stop it");
    g.player_mut(P1).life = 40;
    g.player_mut(P1).ring_prot = true; // The One Ring: no damage from opponents' sources
    zones::draw(&mut g, P1, 2, false).unwrap();
    assert_eq!(life(&g, P1), 40, "prevented like damage");
}

#[test]
fn dark_deal_discards_every_hand_and_draws_one_fewer() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let d = hand(&mut g, P0, &["Dark Deal", "Terror", "Terror", "Terror"])[0];
    hand(&mut g, P1, &["Terror", "Terror", "Terror", "Terror", "Terror"]);
    hand(&mut g, P1, &["Terror"]);
    cast::cast_card(&mut g, P0, d, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2, "3 cards left in hand, draws 3 - 1");
    assert_eq!(g.player(P1).hand.len(), 5, "6 cards, draws 5");
    assert!(in_gy(&g, P0, "Dark Deal"));
    assert_eq!(g.player(P0).gy.len(), 3 + 1 - 1 + 1, "the discarded Terrors and the spell");
}

#[test]
fn dark_deal_with_one_card_or_none_draws_nothing_and_feeds_sheoldred_otherwise() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false);
    let d = hand(&mut g, P0, &["Dark Deal"])[0];
    hand(&mut g, P1, &["Terror"]);
    cast::cast_card(&mut g, P0, d, "hand", Ctx::default()).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len()), (0, 0));
    assert_eq!(life(&g, P1), 40);
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false);
    let d = hand(&mut g, P0, &["Dark Deal"])[0];
    hand(&mut g, P1, &["Terror", "Terror", "Terror", "Terror"]);
    cast::cast_card(&mut g, P0, d, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P1).hand.len(), 3);
    assert_eq!(life(&g, P1), 40 - 6, "Sheoldred drains 2 per redrawn card");
}

#[test]
fn painful_lesson_draws_two_then_loses_two() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Painful Lesson"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    assert_eq!((g.player(P1).hand.len(), life(&g, P1)), (2, 38));
    assert_eq!(life(&g, P0), 40);
}

#[test]
fn the_ai_aims_painful_lesson_at_the_weakest_opponent_with_sheoldred_and_at_itself_without() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Painful Lesson"])[0];
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    run_plan(&mut g, P0, &o[0]);
    assert_eq!((life(&g, P0), life(&g, P1)), (38, 40));

    let mut g = game3();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 3, false);
    g.player_mut(P2).life = 20;
    let c = hand(&mut g, P0, &["Painful Lesson"])[0];
    let o = card_opts(&mut g, c, P0, Some(false));
    run_plan(&mut g, P0, &o[0]);
    assert_eq!(life(&g, P2), 20 - 4 - 2, "two Sheoldred triggers and the spell");
    assert_eq!(life(&g, P1), 40);
}

#[test]
fn the_ai_casts_dark_deal_only_with_sheoldred_out_and_many_opposing_cards() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let c = hand(&mut g, P0, &["Dark Deal"])[0];
    hand(&mut g, P1, &["Terror", "Terror", "Terror", "Terror", "Terror"]);
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "no Sheoldred: held");
    perm(&mut g, P0, SHEO);
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    g.player_mut(P1).hand.truncate(3);
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "too few opposing cards to matter");
}

#[test]
fn the_ai_casts_underworld_dreams_early_and_seizan_only_with_sheoldred() {
    let mut g = game();
    let h = hand(&mut g, P0, &["Underworld Dreams", SEIZAN, "Sheoldred, the Apocalypse"]);
    let p = |g: &Game, c| sim_core::impls::sheoldred::sheoldred_prio(g, P0, c);
    assert!(p(&g, h[0]) > 80 && p(&g, h[0]) < p(&g, h[2]), "a high priority after Sheoldred");
    assert_eq!(p(&g, h[1]), 0, "Seizan waits for Sheoldred");
    perm(&mut g, P0, SHEO);
    assert!(p(&g, h[1]) > 60);
    let mut g = game();
    let z = hand(&mut g, P0, &[SEIZAN])[0];
    hand(&mut g, P1, &["Terror", "Terror", "Terror", "Terror", "Terror"]);
    assert!(p(&g, z) > 0, "high life and far behind on cards");
    g.player_mut(P0).life = 20;
    assert_eq!(p(&g, z), 0);
}

#[test]
fn champions_helm_goes_on_seizan_once_the_plate_protects_sheoldred() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Swamp", 4, false);
    let sheo = perm(&mut g, P0, SHEO);
    let z = perm(&mut g, P0, SEIZAN);
    let helm = perm(&mut g, P0, "Champion's Helm");
    let plate = perm(&mut g, P0, "Darksteel Plate");
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    assert!(o.iter().any(|o| o.label.contains("Champion's Helm to Sheoldred")), "no Plate yet: Helm on her");
    let eq = o.iter().find(|o| o.label.contains("Darksteel Plate")).unwrap();
    assert!(run_plan(&mut g, P0, eq));
    assert_eq!(g.perm(plate).attached, Some(sheo));
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    let eq = o.iter().find(|o| o.label.contains("Champion's Helm")).expect("helm offered");
    assert!(eq.label.contains("Seizan"));
    assert!(run_plan(&mut g, P0, eq));
    assert_eq!(g.perm(helm).attached, Some(z));
    assert!(values::untargetable(&g, z));
}

#[test]
fn the_four_upgrade_cards_resolve_from_the_card_data_not_a_bare_scryfall_body() {
    let g = game();
    for n in [SEIZAN, "Underworld Dreams", "Dark Deal", "Painful Lesson"] {
        let c = card(&g, n);
        assert!(g.registry.get(c).is_some(), "{n} has card code");
    }
}

// ------------------------------------------------------------------ the second upgrade package
const MINE: &str = "Howling Mine";
const BELL: &str = "Temple Bell";
const ATLAS: &str = "Otherworld Atlas";
const UNRAV: &str = "Fate Unraveler";
const PEER: &str = "Peer into the Abyss";
const GEIER: &str = "Geier Reach Sanitarium";
const MIKOKORO: &str = "Mikokoro, Center of the Sea";

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn trim_library(g: &mut Game, p: PlayerId, n: usize) {
    g.player_mut(p).library.truncate(n);
}

#[test]
fn howling_mine_draws_an_extra_card_in_every_players_draw_step_while_untapped() {
    // (the seph seat dredges instead of drawing, so the opponent here is veyran)
    let game = || table(&["sheoldred", "veyran"]);
    let mut g = game();
    let mine = perm(&mut g, P0, MINE);
    g.active = Some(P1);
    let h = hand_len(&g, P1);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(hand_len(&g, P1), h + 2, "the opponent's draw step: the normal draw and the Mine's");
    g.perm_mut(mine).tapped = true;
    let mut g2 = game();
    let mine2 = perm(&mut g2, P0, MINE);
    g2.perm_mut(mine2).tapped = true;
    g2.active = Some(P1);
    let h = hand_len(&g2, P1);
    turn::step_start(&mut g2, P1).unwrap();
    assert_eq!(hand_len(&g2, P1), h + 1, "a tapped Mine draws nothing");
}

#[test]
fn every_opponent_draw_the_new_cards_cause_fires_sheoldred_dreams_and_fate_unraveler() {
    let mut g = table(&["sheoldred", "veyran"]);
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, "Underworld Dreams");
    perm(&mut g, P0, UNRAV);
    perm(&mut g, P0, MINE);
    g.active = Some(P1);
    let before = life(&g, P1);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(before - life(&g, P1), 8, "two draws, each costing 2 + 1 + 1");
    // Temple Bell
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, "Underworld Dreams");
    perm(&mut g, P0, UNRAV);
    let bell = perm(&mut g, P0, BELL);
    let o = perm_opts(&mut g, bell, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P1), 36, "the Bell's draw cost the opponent 4");
    assert_eq!(life(&g, P0), 42, "and our own draw gained 2");
    // Geier Reach Sanitarium
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, UNRAV);
    let l = lands(&mut g, P0, "Swamp", 2, false);
    let sani = lands(&mut g, P0, GEIER, 1, false)[0];
    let _ = l;
    let f = g.registry.get(g.perm(sani).card()).unwrap().land_options.unwrap();
    let o = f(&mut g, sani, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(life(&g, P1), 37, "Sheoldred 2 + Fate Unraveler 1");
    // Peer into the Abyss on the opponent: 20 draws
    let mut g = game();
    perm(&mut g, P0, SHEO);
    trim_library(&mut g, P1, 10);
    g.player_mut(P1).life = 40;
    cast_spell(&mut g, P0, PEER, Ctx { face: Some(P1), ..Ctx::default() });
    assert_eq!(life(&g, P1), 15, "five draws at 2 each: 30, then half of it");
}

#[test]
fn fate_unraveler_pings_only_opponents_that_draw() {
    let mut g = game3();
    perm(&mut g, P0, UNRAV);
    zones::draw(&mut g, P1, 1, false).unwrap();
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((life(&g, P0), life(&g, P1), life(&g, P2)), (40, 39, 40));
    let u = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == UNRAV).unwrap();
    assert_eq!((values::epow(&g, u), values::etgh(&g, u)), (3, 4));
}

#[test]
fn temple_bell_draws_for_every_player_and_the_ai_taps_it_only_with_a_punisher() {
    let mut g = game3();
    let bell = perm(&mut g, P0, BELL);
    assert!(perm_opts(&mut g, bell, P0, None).is_empty(), "no draw punisher: leave it");
    let (h0, h2) = (hand_len(&g, P0), hand_len(&g, P2));
    // do it by hand anyway: the ability itself
    let f = g.registry.get(g.perm(bell).cd.unwrap()).unwrap().options.unwrap();
    perm(&mut g, P0, "Underworld Dreams");
    let o = f(&mut g, bell, P0, None).unwrap();
    assert_eq!(o.len(), 1, "with Underworld Dreams out: tap it at the end of an opponent's turn");
    assert!(f(&mut g, bell, P0, Some(false)).unwrap().is_empty(), "main phase: only when it kills");
    g.player_mut(P1).life = 1;
    assert_eq!(f(&mut g, bell, P0, Some(false)).unwrap().len(), 1, "and now it kills");
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(g.perm(bell).tapped);
    assert_eq!(hand_len(&g, P0), h0 + 1);
    assert!(life(&g, P1) <= 0, "the draw killed the opponent at 1 life");
    assert_eq!(hand_len(&g, P2), h2 + 1);
    assert!(f(&mut g, bell, P0, None).unwrap().is_empty(), "tapped");
}

#[test]
fn peer_into_the_abyss_draws_half_the_library_and_loses_half_the_life_rounding_up() {
    let mut g = game();
    trim_library(&mut g, P1, 11);
    g.player_mut(P1).life = 21;
    cast_spell(&mut g, P0, PEER, Ctx { face: Some(P1), ..Ctx::default() });
    assert_eq!(hand_len(&g, P1), 6, "half of 11, rounded up");
    assert_eq!(g.player(P1).library.len(), 5);
    assert_eq!(life(&g, P1), 10, "21 - 11");
    // yourself
    let mut g = game();
    trim_library(&mut g, P0, 20);
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, PEER, Ctx::default());
    assert_eq!(hand_len(&g, P0), h + 10);
    assert_eq!(life(&g, P0), 20);
}

#[test]
fn the_ai_aims_peer_into_the_abyss_at_the_opponent_it_kills_and_otherwise_mostly_holds_it() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 8, false);
    let c = hand(&mut g, P0, &[PEER])[0];
    trim_library(&mut g, P0, 10);
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "no Sheoldred, a small library: hold it");
    let mut g = game();
    lands(&mut g, P0, "Swamp", 8, false);
    let c = hand(&mut g, P0, &[PEER])[0];
    assert_eq!(card_opts(&mut g, c, P0, Some(false)).len(), 1, "a big library and high life: draw our own");
    perm(&mut g, P0, SHEO);
    trim_library(&mut g, P1, 10);
    g.player_mut(P1).life = 10;
    let o = card_opts(&mut g, c, P0, Some(false));
    assert!(o[0].label.contains("dies"), "Sheoldred: 5 draws cost 10, so it kills: {}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive);
    assert!(in_gy(&g, P0, PEER));
}

#[test]
fn geier_reach_sanitarium_taps_for_colourless_and_loots_everyone() {
    let mut g = game();
    let sani = lands(&mut g, P0, GEIER, 1, false)[0];
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1);
    lands(&mut g, P0, "Swamp", 2, false);
    let f = g.registry.get(g.perm(sani).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, sani, P0, None).unwrap().is_empty(), "no draw punisher: no activation");
    perm(&mut g, P0, "Underworld Dreams");
    // counted after Underworld Dreams is put out: it is in the deck list, so `perm` takes it from the library
    let (h0, h1) = (hand_len(&g, P0), hand_len(&g, P1));
    let (l0, l1) = (g.player(P0).library.len(), g.player(P1).library.len());
    let o = f(&mut g, sani, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(g.perm(sani).tapped);
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1)), (h0, h1), "each drew and discarded one");
    assert_eq!((g.player(P0).library.len(), g.player(P1).library.len()), (l0 - 1, l1 - 1));
    assert_eq!(g.player(P0).gy.len(), 1);
    assert_eq!(g.player(P1).gy.len(), 1);
    assert_eq!(life(&g, P1), 39, "Underworld Dreams");
    let mut g = game();
    let sani = lands(&mut g, P0, GEIER, 1, false)[0];
    perm(&mut g, P0, UNRAV);
    assert!(f(&mut g, sani, P0, None).unwrap().is_empty(), "no spare mana for the {{2}}");
}

fn charge(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Charge)
}

#[test]
fn otherworld_atlas_stores_counters_then_makes_each_player_draw_that_many() {
    // no punisher: a counter at the end of an opponent's turn, never in our own turn
    let mut g = game();
    let atlas = perm(&mut g, P0, ATLAS);
    g.active = Some(P0);
    assert!(perm_opts(&mut g, atlas, P0, None).is_empty(), "nothing in our own turn");
    g.active = Some(P1);
    let o = perm_opts(&mut g, atlas, P0, None);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("charge counter"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(g.perm(atlas).tapped);
    assert_eq!(charge(&g, atlas), 1);
    // Sheoldred out, one counter, no kill: grow it instead
    g.perm_mut(atlas).tapped = false;
    perm(&mut g, P0, SHEO);
    let o = perm_opts(&mut g, atlas, P0, None);
    assert!(o[0].label.contains("charge counter"), "{}", o[0].label);
    // two counters: each player draws two
    g.perm_mut(atlas).data.set(DataKey::Charge, Val::Int(2));
    let (h0, h1) = (hand_len(&g, P0), hand_len(&g, P1));
    let o = perm_opts(&mut g, atlas, P0, None);
    assert!(o[0].label.contains("draws 2"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1)), (h0 + 2, h1 + 2));
    assert_eq!(life(&g, P1), 36, "Sheoldred: 2 for each of the opponent's draws");
    assert_eq!(life(&g, P0), 44, "and 2 back for each of ours");
    assert_eq!(charge(&g, atlas), 2, "the counters stay");
    // one counter that kills: draw in our own main phase
    let mut g = game();
    let atlas = perm(&mut g, P0, ATLAS);
    perm(&mut g, P0, SHEO);
    g.perm_mut(atlas).data.set(DataKey::Charge, Val::Int(1));
    g.player_mut(P1).life = 2;
    g.active = Some(P0);
    let o = perm_opts(&mut g, atlas, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("draws 1"), "{}", o[0].label);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(life(&g, P1) <= 0, "Sheoldred: 2 for the draw");
}

#[test]
fn mikokoro_taps_for_colourless_and_makes_each_player_draw() {
    let mut g = game();
    let miko = lands(&mut g, P0, MIKOKORO, 1, false)[0];
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1);
    lands(&mut g, P0, "Swamp", 2, false);
    let f = g.registry.get(g.perm(miko).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, miko, P0, None).unwrap().is_empty(), "no draw punisher: no activation");
    perm(&mut g, P0, SHEO);
    let (h0, h1) = (hand_len(&g, P0), hand_len(&g, P1));
    let o = f(&mut g, miko, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(g.perm(miko).tapped);
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1)), (h0 + 1, h1 + 1), "each drew one and kept it");
    assert!(g.player(P0).gy.is_empty() && g.player(P1).gy.is_empty(), "no discards");
    assert_eq!(life(&g, P1), 38, "Sheoldred: 2 for the opponent's draw");
    assert_eq!(life(&g, P0), 42, "and 2 back for ours");
    let mut g = game();
    let miko = lands(&mut g, P0, MIKOKORO, 1, false)[0];
    perm(&mut g, P0, SHEO);
    assert!(f(&mut g, miko, P0, None).unwrap().is_empty(), "no spare mana for the {{2}}");
}

#[test]
fn ambitions_cost_draws_three_and_loses_three() {
    let mut g = game();
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Ambition's Cost", Ctx::default());
    assert_eq!(hand_len(&g, P0), h + 3);
    assert_eq!(life(&g, P0), 37);
    perm(&mut g, P0, SHEO);
    cast_spell(&mut g, P0, "Ambition's Cost", Ctx::default());
    assert_eq!(life(&g, P0), 37 - 3 + 6, "Sheoldred gains 2 per card");
    let c = hand(&mut g, P0, &["Ambition's Cost"])[0];
    g.player_mut(P0).life = 5;
    let p = sim_core::impls::sheoldred::sheoldred_prio(&g, P0, c);
    assert_eq!(p, 0, "not at 5 life");
}

#[test]
fn syphon_mind_makes_each_other_player_discard_and_draws_for_each() {
    let mut g = game3();
    hand(&mut g, P1, &["Terror", "Terror"]);
    hand(&mut g, P2, &["Terror"]);
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Syphon Mind", Ctx::default());
    assert_eq!((hand_len(&g, P1), hand_len(&g, P2)), (1, 0));
    assert_eq!(hand_len(&g, P0), h + 2);
    // an opponent with no hand discards nothing, so we draw nothing for them
    let mut g = game3();
    hand(&mut g, P1, &["Terror"]);
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Syphon Mind", Ctx::default());
    assert_eq!(hand_len(&g, P0), h + 1);
}

#[test]
fn howling_mine_is_held_until_a_draw_punisher_is_out() {
    let mut g = game();
    let m = hand(&mut g, P0, &[MINE])[0];
    let p = |g: &Game| sim_core::impls::sheoldred::sheoldred_prio(g, P0, m);
    assert_eq!(p(&g), 0, "no punisher: hold it");
    perm(&mut g, P0, "Underworld Dreams");
    assert!(p(&g) > 50);
    let mut g = game();
    let m = hand(&mut g, P0, &[MINE])[0];
    perm(&mut g, P0, SHEO);
    assert!(sim_core::impls::sheoldred::sheoldred_prio(&g, P0, m) > 50);
    let mut g = game();
    let (m, u) = {
        let h = hand(&mut g, P0, &[MINE, UNRAV]);
        (h[0], h[1])
    };
    assert_eq!(sim_core::impls::sheoldred::sheoldred_prio(&g, P0, m), 0);
    assert!(sim_core::impls::sheoldred::sheoldred_prio(&g, P0, u) >= 80, "the Unraveler is cast at once");
    perm(&mut g, P0, UNRAV);
    assert!(sim_core::impls::sheoldred::sheoldred_prio(&g, P0, m) > 50, "Fate Unraveler is a punisher too");
}

// the older cards the package leans on: each proven with a play in a sheoldred game
#[test]
fn mind_stone_commanders_sphere_and_charcoal_diamond_make_mana() {
    for (name, tapped) in [("Mind Stone", false), ("Commander's Sphere", false), ("Charcoal Diamond", true)] {
        let mut g = game();
        let c = hand(&mut g, P0, &[name])[0];
        cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
        let m = on_bf(&g, P0, name).unwrap_or_else(|| panic!("{name} is on the battlefield"));
        assert_eq!(g.perm(m).tapped, tapped, "{name} entering tapped?");
        g.perm_mut(m).tapped = false;
        assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1, "{name} taps for one mana");
    }
    let mut g = game();
    let d = perm(&mut g, P0, "Charcoal Diamond");
    g.perm_mut(d).tapped = false;
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "B", false), "Charcoal Diamond makes black");
    let mut g = game();
    perm(&mut g, P0, "Commander's Sphere");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "B", false), "the Sphere makes the commander's colour");
}

#[test]
fn mind_stone_and_commanders_sphere_cash_in_for_a_card() {
    let mut g = game();
    let m = perm(&mut g, P0, "Mind Stone");
    lands(&mut g, P0, "Swamp", 6, false);
    let h = hand_len(&g, P0);
    let o = perm_opts(&mut g, m, P0, Some(true));
    assert_eq!(o.len(), 1, "crack Mind Stone with six lands");
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(hand_len(&g, P0), h + 1);
    assert!(on_bf(&g, P0, "Mind Stone").is_none());
    // Commander's Sphere: sacrifice itself to draw (the ability language)
    let mut g = game();
    let s = perm(&mut g, P0, "Commander's Sphere");
    let h = hand_len(&g, P0);
    let o = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    let o = o.iter().find(|o| o.label.contains("Commander's Sphere")).expect("Sphere's sacrifice is offered");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(hand_len(&g, P0), h + 1);
    assert!(!g.perm(s).on_bf);
}

#[test]
fn castle_locthwain_draws_and_loses_life_for_the_hand() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    let castle = lands(&mut g, P0, "Castle Locthwain", 1, false)[0];
    hand(&mut g, P0, &["Terror"]);
    let f = g.registry.get(g.perm(castle).card()).unwrap().land_options.unwrap();
    let o = f(&mut g, castle, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, P0, &o[0]));
    assert_eq!(hand_len(&g, P0), 2);
    assert_eq!(life(&g, P0), 38, "lost life equal to the two cards in hand");
}

#[test]
fn phyrexian_arena_draws_in_upkeep_and_costs_a_life() {
    let mut g = game();
    perm(&mut g, P0, "Phyrexian Arena");
    let h = hand_len(&g, P0);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1);
    assert_eq!(life(&g, P0), 39);
}

#[test]
fn read_the_bones_scries_draws_two_and_loses_two() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Read the Bones", Ctx::default());
    assert_eq!(hand_len(&g, P0), h + 2);
    assert_eq!(life(&g, P0), 40 - 2 + 4, "Sheoldred gains 4 for the two cards");
    assert!(in_gy(&g, P0, "Read the Bones"));
}

#[test]
fn exsanguinate_drains_x_and_the_sheoldred_ai_casts_it_to_finish() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 5, false);
    let c = hand(&mut g, P0, &["Exsanguinate"])[0];
    g.player_mut(P1).life = 20;
    assert!(card_opts(&mut g, c, P0, Some(false)).is_empty(), "X = 3 does not kill and it is early");
    g.player_mut(P1).life = 3;
    let o = card_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "X = 3 kills");
    g.player_mut(P0).life = 10;
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive);
    assert!(g.player(P0).life >= 13, "gained the life lost: {}", g.player(P0).life);
}

#[test]
fn gray_merchant_counts_the_black_mana_symbols_of_your_permanents() {
    let mut g = game();
    perm(&mut g, P0, SHEO); // {2}{B}{B}: 2
    perm(&mut g, P0, "Arcane Signet"); // colourless: 0
    perm(&mut g, P0, "Gray Merchant of Asphodel"); // {3}{B}{B}: 2
    assert_eq!(life(&g, P1), 36, "devotion 4");
    assert_eq!(life(&g, P0), 44);
}

#[test]
fn swiftfoot_boots_go_on_sheoldred_unless_the_helm_comes_first() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    let sheo = perm(&mut g, P0, SHEO);
    let boots = perm(&mut g, P0, "Swiftfoot Boots");
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    let e = o.iter().find(|o| o.label.contains("Swiftfoot Boots")).expect("Boots offered");
    assert!(run_plan(&mut g, P0, e));
    assert_eq!(g.perm(boots).attached, Some(sheo));
    assert!(values::untargetable(&g, sheo));
    // with the Helm in hand, the Helm goes first and the Boots wait
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, "Swiftfoot Boots");
    hand(&mut g, P0, &["Champion's Helm"]);
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    assert!(!o.iter().any(|o| o.label.contains("Swiftfoot Boots")));
    // with the Helm on the table, the Helm is chosen over the Boots
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, "Swiftfoot Boots");
    perm(&mut g, P0, "Champion's Helm");
    let o = sim_core::impls::sheoldred::sheoldred_special(&mut g, P0, false).unwrap();
    let best = o.iter().max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap()).unwrap();
    assert!(best.label.contains("Champion's Helm"));
}

#[test]
fn the_second_package_has_card_code() {
    let g = game();
    for n in [MINE, BELL, UNRAV, PEER, GEIER, "Ambition's Cost", "Syphon Mind"] {
        let c = card(&g, n);
        assert!(g.registry.get(c).is_some() || n == MINE, "{n} has card code");
    }
}

fn creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count()
}

#[test]
fn vona_s_hunger_one_creature_each_then_half_with_the_city_s_blessing() {
    let mut g = game3();
    lands(&mut g, P0, "Swamp", 3, false);
    for _ in 0..3 {
        token(&mut g, P1, 2);
    }
    token(&mut g, P2, 2);
    cast_spell(&mut g, P0, "Vona's Hunger", Ctx::default());
    assert_eq!((creatures(&g, P1), creatures(&g, P2)), (2, 0), "no blessing (3 permanents): one each");

    let mut g = game3();
    lands(&mut g, P0, "Swamp", 10, false);
    for _ in 0..5 {
        token(&mut g, P1, 2);
    }
    token(&mut g, P2, 2);
    cast_spell(&mut g, P0, "Vona's Hunger", Ctx::default());
    assert_eq!((creatures(&g, P1), creatures(&g, P2)), (2, 0), "the city's blessing: half of five rounded up is three");
}
