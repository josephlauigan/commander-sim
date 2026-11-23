//! Resolve attack pronouns on real event objects, including copies, tokens and an object that leaves.
mod play_common;
use play_common::*;
use sim_core::dsl::{activate, has_kw};
use sim_core::engine::{combat, stack, turn, zones};

#[test]
fn atarka_itself_changelings_and_dragon_tokens_gain_double_strike_only_when_attacking() {
    let mut g = table(&["seph", "cloud"]);
    let atarka = perm(&mut g, P0, "Atarka, World Render");
    let changeling = perm(&mut g, P0, "Changeling Outcast");
    let mut spec = zones::Tokens::new(1, 4);
    spec.types = vec!["dragon"];
    let dragon = zones::make_tokens(&mut g, P0, spec).unwrap()[0];
    let ordinary = token(&mut g, P0, 2);
    let idle = perm(&mut g, P0, "Wrathful Red Dragon");
    combat::attack_triggers(&mut g, P0, &[atarka, changeling, dragon, ordinary], P1).unwrap();
    for m in [atarka, changeling, dragon] {
        assert!(has_kw(&g, m, "double strike"), "{} should gain double strike", g.perm(m).name);
    }
    for m in [ordinary, idle] {
        assert!(!has_kw(&g, m, "double strike"));
    }
    turn::step_start(&mut g, P1).unwrap();
    for m in [atarka, changeling, dragon] {
        assert!(!has_kw(&g, m, "double strike"), "grant expired before the next turn's actions");
    }
}

#[test]
fn two_atarkas_each_trigger_for_the_attacking_dragon() {
    let mut g = table(&["seph", "cloud"]);
    let a = perm(&mut g, P0, "Atarka, World Render");
    g.perm_mut(a).exc.not_legendary = true; // a legal nonlegendary copy, as with Helm of the Host
    let b = perm(&mut g, P0, "Atarka, World Render");
    let d = perm(&mut g, P0, "Wrathful Red Dragon");
    g.resolving += 1;
    combat::attack_triggers(&mut g, P0, &[d], P1).unwrap();
    g.resolving -= 1;
    assert_eq!(g.trig_queue.iter().filter(|t| t.src == Some(a) || t.src == Some(b)).count(), 2);
    stack::flush_triggers(&mut g).unwrap();
    assert!(has_kw(&g, d, "double strike"));
    assert!(!has_kw(&g, a, "double strike") && !has_kw(&g, b, "double strike"));
}

#[test]
fn departed_attacker_does_not_redirect_its_grant_to_a_new_object_or_other_dragon() {
    let mut g = table(&["seph", "cloud"]);
    perm(&mut g, P0, "Atarka, World Render");
    let d = perm(&mut g, P0, "Wrathful Red Dragon");
    let idle = perm(&mut g, P0, "Dragonlord Ojutai");
    g.resolving += 1;
    combat::attack_triggers(&mut g, P0, &[d], P1).unwrap();
    zones::exile_perm(&mut g, d).unwrap();
    let new = perm(&mut g, P0, "Wrathful Red Dragon");
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert!(!has_kw(&g, new, "double strike"), "a returned card is a new object");
    assert!(!has_kw(&g, idle, "double strike"));
}

#[test]
fn hallowblade_can_activate_while_tapped_and_its_self_grant_expires() {
    let mut g = table(&["seph", "cloud"]);
    let b = perm(&mut g, P0, "Seasoned Hallowblade");
    let other = token(&mut g, P0, 2);
    g.perm_mut(b).tapped = true;
    hand(&mut g, P0, &["Plains"]);
    assert!(activate(&mut g, P0, b, 0).unwrap());
    assert!(has_kw(&g, b, "indestructible"));
    assert!(!has_kw(&g, other, "indestructible"));
    assert!(g.player(P0).hand.is_empty());
    turn::step_start(&mut g, P1).unwrap();
    assert!(!has_kw(&g, b, "indestructible"));
}
