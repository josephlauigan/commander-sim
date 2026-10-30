//! Practice mode: the human seat's mana pool (Python's tests/test_play_mana.py). Tapping a source adds its mana (with
//! the source's side effects), costs are paid from the pool, and the pool empties between steps.

mod play_common;
use play_common::*;
use sim_core::engine::turn::continue_turn;
use sim_core::human::{legal, mana};
use sim_core::state::Step;

fn src(g: &Game, p: PlayerId, name: &str) -> i64 {
    mana::sources(g, p).into_iter().find(|x| x.name == name).unwrap_or_else(|| panic!("no source {name}")).id as i64
}

fn pool_pay(g: &mut Game, p: PlayerId, generic: u32, pips: &str) -> bool {
    let mut pl = std::mem::take(&mut g.player_mut(p).pool);
    let ok = pl.pay(g, p, generic, pips);
    g.player_mut(p).pool = pl;
    ok
}

#[test]
fn colours_and_generic() {
    let mut g = table(&["sauron", "veyran"]);
    pool(&mut g, P0, &[('B', 2)]);
    let pl = g.player(P0).pool.clone();
    assert!(!pl.can_pay(&g, P0, 2, "BB"));
    assert_eq!(pl.missing(2, "BB"), "{2}");
    pool(&mut g, P0, &[('C', 2)]);
    assert!(pool_pay(&mut g, P0, 2, "BB"));
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn generic_spends_colourless_before_colours() {
    let mut g = table(&["sauron", "veyran"]);
    pool(&mut g, P0, &[('C', 1), ('U', 1), ('B', 1)]);
    assert!(pool_pay(&mut g, P0, 1, "B"));
    assert_eq!(pool_text(&g, P0), "{U}");
}

#[test]
fn any_colour_mana_pays_a_pip() {
    let mut g = table(&["sauron", "veyran"]);
    pool(&mut g, P0, &[('A', 1)]);
    assert!(pool_pay(&mut g, P0, 0, "U"));
}

#[test]
fn a_land_taps_into_the_pool() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let id = src(&g, P0, "Swamp");
    assert_eq!(mana::tap(&mut g, P0, id, None).unwrap(), None);
    assert_eq!(pool_text(&g, P0), "{B}");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count(), 1);
    assert_eq!(mana::sources(&g, P0).len(), 1);
}

#[test]
fn wrong_colour_is_refused() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 1, false);
    let id = src(&g, P0, "Swamp");
    let why = mana::tap(&mut g, P0, id, Some('U')).unwrap().unwrap();
    assert!(why.contains("can't make blue"), "{why}");
    let l = g.player(P0).lands[0];
    assert!(!g.perm(l).tapped);
}

#[test]
fn talisman_hurts_only_for_colour() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Talisman of Dominance");
    let id = src(&g, P0, "Talisman of Dominance");
    assert_eq!(mana::tap(&mut g, P0, id, Some('C')).unwrap(), None);
    assert_eq!(g.player(P0).life, 40);
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Talisman of Dominance");
    let id = src(&g, P0, "Talisman of Dominance");
    mana::tap(&mut g, P0, id, Some('U')).unwrap();
    assert_eq!(g.player(P0).life, 39);
}

#[test]
fn sol_ring_and_treasure() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sol Ring");
    g.player_mut(P0).treasures = 1;
    let id = src(&g, P0, "Sol Ring");
    mana::tap(&mut g, P0, id, None).unwrap();
    let id = src(&g, P0, "Treasure");
    mana::tap(&mut g, P0, id, Some('R')).unwrap();
    assert_eq!(g.player(P0).treasures, 0);
    assert_eq!(pool_text(&g, P0), "{R}{C}{C}");
}

#[test]
fn paying_from_the_pool_and_the_reason_when_short() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    for _ in 0..2 {
        let id = mana::sources(&g, P0)[0].id as i64;
        mana::tap(&mut g, P0, id, None).unwrap();
    }
    let why = mana::pay_from_pool(&mut g, P0, 2, "BB");
    assert_eq!(why.as_deref(), Some("It costs {2}{B}{B}; your mana pool has {B}{B}. Missing {2}."));
    assert_eq!(mana::pay_from_pool(&mut g, P0, 0, "BB"), None);
}

#[test]
fn ritual_mana_joins_the_pool() {
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).floating.any = 3; // Dark Ritual's mana, as the engine adds it
    assert_eq!(mana::pay_from_pool(&mut g, P0, 2, "B"), None);
    assert_eq!(g.player(P0).floating.any, 0);
}

#[test]
fn the_pool_empties_between_steps() {
    let mut g = table(&["sauron", "veyran"]);
    seat(&mut g, P0, vec![]);
    pool(&mut g, P0, &[('B', 2)]);
    let _ = continue_turn(&mut g, P0, Step::End);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn sources_name_their_land_permanent_or_treasure() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Island", 2, false);
    let rock = perm(&mut g, P0, "Sol Ring");
    g.player_mut(P0).treasures = 1;
    let srcs = mana::sources(&g, P0);
    let mut ls: Vec<usize> = srcs.iter().filter_map(|x| x.land).collect();
    ls.sort();
    assert_eq!(ls, vec![0, 1]);
    let idx = g.player(P0).perms.iter().position(|&m| m == rock).unwrap();
    assert_eq!(srcs.iter().filter_map(|x| x.perm).collect::<Vec<_>>(), vec![idx]);
    assert_eq!(srcs.iter().filter(|x| x.treasure).count(), 1);
}

const VRASKA: &str = "Vraska, Betrayal's Sting";

fn cast_vraska(answers: Vec<F>, mana: &[(char, u32)]) -> (Game, std::sync::Arc<ScriptHuman>) {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &[VRASKA]);
    pool(&mut g, P0, mana);
    let mut a = vec![act(json!({"do": "cast", "card": 0}))];
    a.extend(answers);
    let h = main(&mut g, P0, a);
    (g, h)
}

#[test]
fn pay_with_life() {
    let (g, h) = cast_vraska(vec![pick(1)], &[('B', 2), ('C', 4)]); // {4}{B}{B/P}: both ways work
    assert_eq!(h.asked()[1].choices[..2], ["{B}".to_string(), "2 life".to_string()]);
    assert!(has_perm(&g, P0, VRASKA));
    assert_eq!((g.player(P0).life, pool_total(&g, P0)), (38, 1)); // the spare {B} is left over
}

#[test]
fn pay_with_mana() {
    let (g, _h) = cast_vraska(vec![pick(0)], &[('B', 2), ('C', 4)]);
    assert_eq!((g.player(P0).life, pool_total(&g, P0)), (40, 0));
}

#[test]
fn only_life_works() {
    let (g, _h) = cast_vraska(vec![], &[('B', 1), ('C', 4)]); // one {B}: the Phyrexian symbol must be life
    assert!(has_perm(&g, P0, VRASKA));
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn rules_check_counts_life() {
    let mut g = table(&["sauron", "veyran"]);
    let c = hand(&mut g, P0, &[VRASKA])[0];
    pool(&mut g, P0, &[('B', 1), ('C', 4)]);
    assert_eq!(legal::check_cast(&mut g, P0, c, "hand"), None);
    g.player_mut(P0).life = 1;
    let why = legal::check_cast(&mut g, P0, c, "hand").unwrap();
    assert!(why.contains("2 life each"), "{why}");
}
