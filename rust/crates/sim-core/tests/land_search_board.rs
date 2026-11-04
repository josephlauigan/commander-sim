//! Urza's Cave and Expedition Map choose a land by the board (`impls/karn_engines.rs` board_land_want, 2026-11-04):
//! Cabal Coffers and Urborg, Tomb of Yawgmoth as a pair, a colour the hand needs, and nothing when no land is worth
//! the sacrifice. Y'shtola's list holds Urborg, Coffers and Serra's Sanctum.

use sim_core::engine::mana::land_colors_now;
use sim_core::hooks::Opt;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    let mut g = table(&["yshtola", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn lands_named(g: &Game, n: &str) -> usize {
    g.player(P0).lands.iter().filter(|&&l| &*g.db.get(g.perm(l).card()).name == n).count()
}

fn all_opts(g: &mut Game) -> Vec<Opt> {
    let s = sim_core::ai::brain::Situation::new(g, P0);
    let mut o = sim_core::dsl::ability_options(g, P0, true).unwrap();
    o.extend(sim_core::ai::brain::hook_options(g, P0, &s, Some(true)).unwrap());
    o.extend(sim_core::ai::brain::hook_options(g, P0, &s, Some(false)).unwrap());
    o
}

fn has_opt(g: &mut Game, word: &str) -> bool {
    all_opts(g).iter().any(|o| o.label.contains(word))
}

fn run(g: &mut Game, word: &str) -> bool {
    let o = all_opts(g);
    let best = o.iter().filter(|o| o.label.contains(word)).max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap());
    let Some(x) = best else { panic!("no option {word:?} in {:?}", o.iter().map(|o| &o.label).collect::<Vec<_>>()) };
    sim_core::ai::act::perform(g, P0, x.act.as_ref().unwrap()).unwrap()
}

#[test]
fn urzas_cave_fetches_urborg_when_cabal_coffers_is_out() {
    let mut g = tbl();
    lands(&mut g, P0, "Cabal Coffers", 1, false);
    lands(&mut g, P0, "Plains", 5, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert!(run(&mut g, "Urza's Cave"));
    assert_eq!(lands_named(&g, "Urborg, Tomb of Yawgmoth"), 1);
    assert_eq!(lands_named(&g, "Urza's Cave"), 0, "sacrificed");
}

#[test]
fn urzas_cave_fetches_cabal_coffers_when_urborg_is_out() {
    let mut g = tbl();
    lands(&mut g, P0, "Urborg, Tomb of Yawgmoth", 1, false);
    lands(&mut g, P0, "Plains", 5, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert!(run(&mut g, "Urza's Cave"));
    assert_eq!(lands_named(&g, "Cabal Coffers"), 1);
}

#[test]
fn urzas_cave_fetches_the_colour_the_hand_needs_without_the_pair() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    hand(&mut g, P0, &["Rhystic Study"]);
    assert!(run(&mut g, "Urza's Cave"));
    assert_eq!(lands_named(&g, "Urborg, Tomb of Yawgmoth") + lands_named(&g, "Cabal Coffers"), 0);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(land_colors_now(&g, P0, l).has('U'), "a blue source for Rhystic Study");
}

#[test]
fn urzas_cave_stays_a_land_when_no_land_is_worth_it() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert!(!has_opt(&mut g, "Urza's Cave"), "no Coffers or Urborg out, no colour missing");
}

fn opt_utility(g: &mut Game, word: &str) -> Option<f64> {
    all_opts(g).iter().filter(|o| o.label.contains(word)).map(|o| o.utility).reduce(f64::max)
}

#[test]
fn urzas_cave_for_urborg_beside_coffers_is_worth_more_than_a_mid_value_play() {
    let mut g = tbl();
    lands(&mut g, P0, "Cabal Coffers", 1, false);
    lands(&mut g, P0, "Plains", 7, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    let u = opt_utility(&mut g, "Urza's Cave").expect("the Cave is offered");
    assert!(u >= 5.0, "Urborg turns on seven more Swamps for Coffers: utility {u}");
}

#[test]
fn urzas_cave_is_not_offered_once_the_pair_is_out() {
    let mut g = tbl();
    lands(&mut g, P0, "Cabal Coffers", 1, false);
    lands(&mut g, P0, "Urborg, Tomb of Yawgmoth", 1, false);
    lands(&mut g, P0, "Plains", 3, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert!(!has_opt(&mut g, "Urza's Cave"));
}

#[test]
fn expedition_map_finds_urborg_for_cabal_coffers() {
    let mut g = tbl();
    lands(&mut g, P0, "Cabal Coffers", 1, false);
    lands(&mut g, P0, "Plains", 4, false);
    perm(&mut g, P0, "Expedition Map");
    assert!(run(&mut g, "Expedition Map"));
    let last = *g.player(P0).hand.last().unwrap();
    assert_eq!(&*g.db.get(last).name, "Urborg, Tomb of Yawgmoth");
}
