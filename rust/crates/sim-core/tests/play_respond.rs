//! Practice mode: priority on other players' turns (Python's tests/test_play_respond.py). Every spell an opponent
//! casts gives you priority: counter it, cast an instant, or pass. When an opponent counters your spell you may counter
//! back. You also get priority at the end of each other turn and when attacked.

mod play_common;
use play_common::*;
use sim_core::engine::{self, combat, mana as emana, stack};
use sim_core::human::{human::respond, legal, mana};
use sim_core::state::Ctx;

fn tap_all(g: &mut Game, p: PlayerId) {
    while let Some(s) = mana::sources(g, p).first() {
        let id = s.id as i64;
        mana::tap(g, p, id, None).unwrap();
    }
}

/// opponent q casts `name` from hand, paid automatically
fn opp_casts(g: &mut Game, q: PlayerId, name: &str) -> bool {
    let c = hand(g, q, &[name])[0];
    let (generic, pips) = emana::cost_of(g, q, c);
    emana::pay(g, q, generic, &pips, false).unwrap();
    engine::cast::cast_card(g, q, c, "hand", Ctx::default()).unwrap()
}

#[test]
fn counter_an_opponents_spell() {
    let mut g = table(&["veyran", "sauron"]);
    hand(&mut g, P1, &["Counterspell"]);
    lands(&mut g, P1, "Island", 2, false);
    tap_all(&mut g, P1);
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Mountain", 1, false);
    let h = seat(&mut g, P1, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!opp_casts(&mut g, P0, "Guttersnipe"));
    assert!(!has_perm(&g, P0, "Guttersnipe"));
    let c = card(&g, "Counterspell");
    assert!(g.player(P1).gy.contains(&c));
    assert_eq!(h.asked()[0].data["stack"][0]["name"], "Guttersnipe");
    assert_eq!(h.asked()[0].data["stack"].as_array().unwrap().len(), 1);
}

#[test]
fn pass_and_it_resolves() {
    let mut g = table(&["veyran", "sauron"]);
    lands(&mut g, P0, "Mountain", 3, false);
    seat(&mut g, P1, vec![pass()]);
    assert!(opp_casts(&mut g, P0, "Guttersnipe"));
}

#[test]
fn priority_on_every_spell() {
    let mut g = table(&["veyran", "sauron"]);
    lands(&mut g, P0, "Island", 1, false);
    let h = seat(&mut g, P1, vec![pass()]);
    opp_casts(&mut g, P0, "Sol Ring"); // a spell the AI would never bother to counter
    assert_eq!(h.asked().len(), 1);
}

#[test]
fn tap_then_counter() {
    let mut g = table(&["veyran", "sauron"]);
    hand(&mut g, P1, &["Counterspell"]);
    lands(&mut g, P1, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 3, false);
    seat(
        &mut g,
        P1,
        vec![
            act(json!({"do": "tap", "source": 0})),
            act(json!({"do": "tap", "source": 0})),
            act(json!({"do": "cast", "card": 0})),
        ],
    );
    assert!(!opp_casts(&mut g, P0, "Guttersnipe"));
}

#[test]
fn wrong_kind_of_counterspell() {
    let mut g = table(&["veyran", "sauron"]);
    let fg = hand(&mut g, P1, &["Fierce Guardianship"])[0];
    lands(&mut g, P1, "Island", 3, false);
    tap_all(&mut g, P1);
    lands(&mut g, P0, "Mountain", 3, false);
    let gs = card(&g, "Guttersnipe");
    assert_eq!(
        legal::check_counter(&g, P1, fg, gs).as_deref(),
        Some("Fierce Guardianship can't counter Guttersnipe.") // noncreature spells only
    );
    let h = seat(&mut g, P1, vec![act(json!({"do": "cast", "card": 0})), pass()]);
    assert!(opp_casts(&mut g, P0, "Guttersnipe"));
    assert_eq!(h.told()[0].0, "invalid");
}

#[test]
fn no_spell_to_counter() {
    let mut g = table(&["sauron", "veyran"]);
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    lands(&mut g, P0, "Island", 2, false);
    tap_all(&mut g, P0);
    let why = legal::check_cast(&mut g, P0, c, "hand").unwrap();
    assert!(why.contains("counters a spell"), "{why}");
}

#[test]
fn counter_back_when_the_ai_counters_you() {
    let mut g = table(&["sauron", "veyran"]);
    let sheo = hand(&mut g, P0, &["Sheoldred, the Apocalypse"])[0];
    hand(&mut g, P0, &["Counterspell"]);
    lands(&mut g, P0, "Island", 2, false);
    tap_all(&mut g, P0);
    let ctr = hand(&mut g, P1, &["Counterspell"])[0];
    lands(&mut g, P1, "Island", 2, false);
    // hand: Sheoldred, Counterspell; target: their counterspell
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 1})), pick(0)]);
    let item = stack::spell_item(&mut g, P0, sheo, Ctx::default(), 7.0);
    let id = item.id;
    g.stack.push(item); // Sheoldred is on the stack; Veyran counters it
    assert!(stack::cast_counter_spell(&mut g, P1, ctr, id).unwrap());
    // (Python reads `item.countered` on the object it kept: here the stack's own copy, if it is still there)
    assert!(!g.stack.iter().find(|x| x.id == id).is_some_and(|x| x.countered)); // you countered the counterspell
    assert_eq!(g.player(P0).stats.get("counterwar_won").copied(), Some(1));
}

#[test]
fn priority_when_attacked() {
    let mut g = table(&["veyran", "sauron"]);
    perm(&mut g, P1, "Grave Titan");
    let a = perm(&mut g, P0, "Guttersnipe");
    let h = seat(&mut g, P1, vec![pass(), cancel()]);
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    let kinds: Vec<String> = h.asked().iter().map(|r| r.kind.to_string()).collect();
    assert_eq!(kinds, ["priority", "block"]);
    assert!(h.asked()[0].prompt.contains("attacks you"), "{}", h.asked()[0].prompt);
}

#[test]
fn the_priority_order_on_a_spell() {
    let mut g = table(&["veyran", "sauron", "seph"]);
    let h = seat(&mut g, P2, vec![pass()]);
    let bolt = card(&g, "Lightning Bolt");
    respond(&mut g, P2, "Veyran casts Opt", Some(bolt), Some(P0)).unwrap();
    let d = h.asked()[0].data.clone();
    let keys: Vec<&str> = d["order"].as_array().unwrap().iter().map(|x| x["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["sauron", "seph"]); // after the caster, in turn order
    assert_eq!(d["stack"], json!([{"name": "Lightning Bolt"}]));
}
