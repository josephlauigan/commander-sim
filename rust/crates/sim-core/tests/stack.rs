//! The stack and triggered abilities: Python's tests/test_stack.py and tests/test_triggers.py (the tests that don't
//! need a person at the table or a card's own code; those come with phase 9 and M5).

use sim_core::engine::{life, stack, zones};
use sim_core::hooks::{TrigAct, Trigger};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game, StackKind};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn drained(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).filter(|&q| g.player(q).life < 40).collect()
}

// ------------------------------------------------------------------ test_stack.Stack
#[test]
fn a_counter_war_between_three_players() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let (s, v) = (P0, P1);
    let sheo = card(&g, "Sheoldred, the Apocalypse");
    hand(&mut g, v, &["Counterspell"]);
    lands(&mut g, v, "Island", 2, false);
    hand(&mut g, s, &["Counterspell"]);
    lands(&mut g, s, "Island", 2, false);
    let it = stack::spell_item(&mut g, s, sheo, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    let ctr = *g.player(v).hand.last().unwrap();
    assert!(stack::cast_counter_spell(&mut g, v, ctr, id).unwrap()); // Veyran counters Sheoldred ...
    assert!(!g.stack.iter().find(|x| x.id == id).unwrap().countered); // ... Sauron counters that back
    assert_eq!(g.player(s).stats["counterwar_won"], 1);
    let cs = card(&g, "Counterspell");
    let n = g.player(s).gy.iter().chain(&g.player(v).gy).filter(|&&c| c == cs).count();
    assert_eq!(n, 2);
}

#[test]
fn a_countered_counterspell_does_nothing() {
    let mut g = table(&["sauron", "veyran"]);
    let (s, v) = (P0, P1);
    let sheo = card(&g, "Sheoldred, the Apocalypse");
    let cs = card(&g, "Counterspell");
    let it = stack::spell_item(&mut g, s, sheo, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    let mut ctr_item = stack::spell_item(&mut g, v, cs, Ctx { counter: Some(id), ..Ctx::default() }, 7.0);
    ctr_item.generic = false;
    let ctr_id = ctr_item.id;
    g.stack.push(ctr_item);
    stack::resolve_counter(&mut g, s, cs, ctr_id).unwrap(); // Sauron's counter resolves first
    assert!(g.stack.iter().find(|x| x.id == ctr_id).unwrap().countered);
    assert!(!g.stack.iter().find(|x| x.id == id).unwrap().countered);
}

#[test]
fn the_stack_copies_with_the_game() {
    let mut g = table(&["veyran", "sauron"]);
    let snipe = card(&g, "Guttersnipe");
    let it = stack::spell_item(&mut g, P0, snipe, Ctx::default(), 5.0);
    g.stack.push(it);
    let mut g2 = g.clone();
    assert_eq!(g2.stack.len(), 1);
    assert_eq!(g2.stack[0].controller, P0);
    stack::settle_stack(&mut g2).unwrap();
    assert!(g2.stack.is_empty());
    assert!(g2.player(P0).perms.iter().any(|&m| g2.perm(m).name == "Guttersnipe"));
    assert_eq!(g.stack.len(), 1); // the original is untouched
}

// ------------------------------------------------------------------ test_stack.Abilities
#[test]
fn no_window_when_nobody_could_answer() {
    let mut g = table(&["veyran", "sauron"]);
    let src = perm(&mut g, P0, "Triskelion");
    let n = g.stack_pushes;
    assert!(stack::ability_window(&mut g, P0, Some(src), "1 damage", Some(9.0), None).unwrap());
    assert_eq!(g.stack_pushes, n);
}

// ------------------------------------------------------------------ test_triggers.Triggers
#[test]
fn an_enters_trigger_waits_for_the_spell_to_finish() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    g.resolving = 1; // a spell is resolving
    perm(&mut g, P0, "Archon of Cruelty");
    assert!(drained(&g, P0).is_empty());
    assert_eq!(g.trig_queue.len(), 1);
    g.resolving = 0;
    stack::flush_triggers(&mut g).unwrap(); // it has finished: the trigger goes on the stack
    assert_eq!(drained(&g, P0).len(), 1);
}

#[test]
fn the_active_players_triggers_go_on_the_stack_first() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    g.active = Some(P1);
    let t = |q| Trigger {
        controller: q,
        src: None,
        act: TrigAct::EtbOnce { p: q, m: PermId(0) },
        name: None,
        known: true,
        imp: None,
        cast_etb: false,
    };
    let order: Vec<PlayerId> = stack::apnap(&g, vec![t(P0), t(P2), t(P1)]).iter().map(|t| t.controller).collect();
    assert_eq!(order, [P1, P2, P0]); // so they resolve last
}

#[test]
fn a_copied_game_keeps_its_waiting_triggers() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    g.resolving = 1;
    perm(&mut g, P0, "Archon of Cruelty"); // its trigger waits for the spell resolving
    let mut g2 = g.clone();
    stack::settle_stack(&mut g2).unwrap(); // the look-ahead finishes the copy's stack
    assert_eq!(drained(&g2, P0).len(), 1);
    assert!(drained(&g, P0).is_empty()); // the real game is untouched
    assert_eq!(g.trig_queue.len(), 1);
}

#[test]
fn a_countered_trigger_moves_no_card() {
    let mut g = table(&["seph", "veyran"]);
    let m = perm(&mut g, P0, "Grave Titan");
    let cd = g.perm(m).cd.unwrap();
    let mut it = stack::spell_item(&mut g, P0, cd, Ctx { source: Some(m), ..Ctx::default() }, 3.0);
    it.kind = StackKind::Trigger;
    it.generic = false;
    it.countered = true;
    g.stack.push(it);
    stack::settle_stack(&mut g).unwrap();
    assert!(g.player(P0).perms.contains(&m));
    assert!(!g.player(P0).gy.contains(&cd));
}

#[test]
fn archons_trigger_drains_the_biggest_threat() {
    // not in the Python suite: the engine's own enters trigger (etb_once) end to end
    let mut g = table(&["seph", "veyran", "sauron"]);
    perm(&mut g, P2, "Grave Titan"); // Sauron's board is the threat
    let before = g.player(P2).life;
    let hand0 = g.player(P0).hand.len();
    perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!(g.player(P2).life, before - 3);
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    life::check_state(&mut g).unwrap();
    let _ = zones::TOKEN_CAP;
}
