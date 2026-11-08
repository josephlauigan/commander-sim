//! Practice mode: every spell with {X} in its cost asks the person for X (2026-11-07, JD's report: Torment of Hailfire
//! cast with X = 0 with 13 mana floating). Before, only spells tagged tokx / xtutor / xdrain asked; Torment of Hailfire,
//! Hangarback Walker and Mockingbird were cast for X = 0. The card data now records the number of {X} symbols
//! (`xcount`, from the Scryfall cost).

mod play_common;
use play_common::*;
use std::sync::{Arc, Mutex};

/// answer X with `x`, recording every question asked
fn x_answer(x: u32, asked: Arc<Mutex<Vec<(String, Vec<String>)>>>) -> F {
    Box::new(move |r: &Request| {
        asked.lock().unwrap().push((r.prompt.to_string(), r.choices.iter().map(|c| c.to_string()).collect()));
        let want = format!("X = {x}");
        Answer::Pick(
            r.choices.iter().position(|c| *c == want).unwrap_or_else(|| panic!("no {want} in {:?}", r.choices)),
        )
    })
}

#[test]
fn cards_with_x_in_their_cost_say_so() {
    let g = table(&["yshtola", "cloud"]);
    for (name, n) in [("Torment of Hailfire", 1), ("Hangarback Walker", 2), ("Mockingbird", 1), ("Exsanguinate", 1)] {
        assert_eq!(g.db.get(card(&g, name)).xcount, n, "{name}");
    }
    assert_eq!(g.db.get(card(&g, "Sol Ring")).xcount, 0);
}

#[test]
fn torment_of_hailfire_asks_for_x_and_pays_it() {
    // JD's game: Cabal Coffers, Sol Ring and six lands made {B}x11 {C}{C}; {X}{B}{B} leaves 11 for X
    let mut g = table(&["yshtola", "cloud"]);
    hand(&mut g, P0, &["Torment of Hailfire"]);
    pool(&mut g, P0, &[('B', 11), ('C', 2)]);
    let asked = Arc::new(Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(11, asked.clone())]);
    let a = asked.lock().unwrap();
    assert!(a[0].0.contains("Torment of Hailfire: choose X"), "{a:?}");
    assert_eq!(a[0].1.last().map(String::as_str), Some("X = 11"), "X up to 11: {a:?}");
    assert_eq!(pool_total(&g, P0), 0, "X = 11 and {{B}}{{B}} paid");
}

#[test]
fn hangarback_walker_costs_two_per_x_and_enters_with_x_counters() {
    let mut g = table(&["yshtola", "cloud"]);
    hand(&mut g, P0, &["Hangarback Walker"]);
    pool(&mut g, P0, &[('C', 7)]);
    let asked = Arc::new(Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(3, asked.clone())]);
    let a = asked.lock().unwrap();
    assert!(a[0].0.contains("2 mana for each"), "{a:?}");
    assert_eq!(a[0].1.last().map(String::as_str), Some("X = 3"), "{{X}}{{X}} with 7 mana: X up to 3: {a:?}");
    assert_eq!(pool_total(&g, P0), 1, "6 paid for X = 3");
    let w = perms_named(&g, P0, "Hangarback Walker");
    assert_eq!(w.len(), 1, "it survives as a 3/3");
    assert_eq!(g.perm(w[0]).plus, 3, "three +1/+1 counters");
}

#[test]
fn mockingbird_asks_for_x_then_copies_a_creature_of_that_mana_value_or_less() {
    let mut g = table(&["yshtola", "cloud"]);
    hand(&mut g, P0, &["Mockingbird"]);
    perm(&mut g, P1, "Guttersnipe"); // mana value 3
    perm(&mut g, P1, "Hellkite Tyrant"); // mana value 6: too big for 3 mana spent
    pool(&mut g, P0, &[('U', 1), ('C', 2)]);
    let asked = Arc::new(Mutex::new(vec![]));
    let a2 = asked.clone();
    let pick_copy: F = Box::new(move |r: &Request| {
        a2.lock().unwrap().push((r.prompt.to_string(), r.choices.iter().map(|c| c.to_string()).collect()));
        Answer::Pick(r.choices.iter().position(|c| c.contains("Guttersnipe")).expect("Guttersnipe offered"))
    });
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(2, asked.clone()), pick_copy]);
    let a = asked.lock().unwrap();
    assert!(a[1].0.contains("Mockingbird enters as"), "{a:?}");
    assert!(!a[1].1.iter().any(|c| c.contains("Hellkite")), "mana value 6 is not offered: {a:?}");
    assert_eq!(pool_total(&g, P0), 0);
    assert!(has_perm(&g, P0, "Guttersnipe"), "Mockingbird entered as a copy of Guttersnipe: {:?}", g.log);
}

#[test]
fn mockingbird_with_x_zero_and_nothing_cheap_enough_enters_as_itself() {
    let mut g = table(&["yshtola", "cloud"]);
    hand(&mut g, P0, &["Mockingbird"]);
    perm(&mut g, P1, "Hellkite Tyrant");
    pool(&mut g, P0, &[('U', 1)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(has_perm(&g, P0, "Mockingbird"), "a 1/1 flying Bird");
}

#[test]
fn chalice_of_the_void_costs_two_per_x_and_enters_with_x_charge_counters() {
    let mut g = table(&["yshtola", "cloud"]);
    hand(&mut g, P0, &["Chalice of the Void"]);
    pool(&mut g, P0, &[('C', 4)]);
    let asked = Arc::new(Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(1, asked.clone())]);
    let a = asked.lock().unwrap();
    assert_eq!(a[0].1.last().map(String::as_str), Some("X = 2"), "{{X}}{{X}} with 4 mana: X up to 2: {a:?}");
    assert_eq!(pool_total(&g, P0), 2, "2 paid for X = 1");
    let c = perms_named(&g, P0, "Chalice of the Void");
    assert_eq!(c.len(), 1);
    assert_eq!(g.perm(c[0]).data.int(sim_core::state::DataKey::Charge), 1, "one charge counter");
}
