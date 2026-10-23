//! Practice mode: a person attacking into Ghostly Prison-style taxes. The engine asks whether to pay and taps the
//! person's untapped sources for it (the pool empties between steps, so mana floated in the main phase is gone by
//! the declare-attackers step). Attackers that aren't paid for don't attack, and the log says so. Before this, the
//! tax was checked against the floating pool only and every attacker was silently dropped.

mod play_common;
use play_common::*;
use sim_core::engine::combat::combat;

// the tax prompt is a yes/no choice: pick(0) pays, pick(1) declines

fn untapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

fn logged(g: &mut Game, p: PlayerId) -> Vec<String> {
    g.log = Some(vec![]);
    combat(g, p).unwrap();
    g.log.take().unwrap_or_default()
}

#[test]
fn pays_the_tax_from_untapped_lands() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Ghostly Prison");
    perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 3, false);
    let h = seat(&mut g, P0, vec![picks(&[0]), pick(0)]);
    let log = logged(&mut g, P0);
    assert_eq!(g.player(P1).life, 40 - 6);
    assert_eq!(untapped_lands(&g, P0), 1);
    assert!(h.asked().iter().any(|r| r.prompt.contains("pay {2}")), "asked: {:?}", h.asked());
    assert!(log.iter().any(|l| l.contains("pays {2}")), "log: {log:?}");
}

#[test]
fn a_small_attacker_is_yours_to_pay_for() {
    // the AI only pays for attackers with power at least the tax; a person decides
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Ghostly Prison");
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Plains", 2, false);
    seat(&mut g, P0, vec![picks(&[0]), pick(0)]);
    combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, 40 - 1);
}

#[test]
fn declining_the_tax_is_logged() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Ghostly Prison");
    let t = perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 2, false);
    seat(&mut g, P0, vec![picks(&[0]), pick(1)]);
    let log = logged(&mut g, P0);
    assert_eq!(g.player(P1).life, 40);
    assert!(!g.perm(t).tapped);
    assert_eq!(untapped_lands(&g, P0), 2);
    assert!(log.iter().any(|l| l.contains("doesn't pay")), "log: {log:?}");
}

#[test]
fn no_mana_for_the_tax_is_logged() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Ghostly Prison");
    perm(&mut g, P0, "Grave Titan");
    let h = seat(&mut g, P0, vec![picks(&[0])]);
    let log = logged(&mut g, P0);
    assert_eq!(g.player(P1).life, 40);
    assert!(!h.asked().iter().any(|r| r.prompt.contains("pay {")), "asked: {:?}", h.asked());
    assert!(log.iter().any(|l| l.contains("can't pay")), "log: {log:?}");
}

#[test]
fn only_the_attackers_you_can_pay_for_attack() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Ghostly Prison");
    perm(&mut g, P0, "Grave Titan"); // 6/6
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Plains", 2, false); // {2}: one attacker
    let h = seat(&mut g, P0, vec![picks(&[0, 1]), pick(0)]);
    let log = logged(&mut g, P0);
    assert_eq!(g.player(P1).life, 40 - 6, "the strongest one attacks");
    assert_eq!(untapped_lands(&g, P0), 0);
    assert!(h.asked().iter().any(|r| r.prompt.contains("pay {2}")), "asked: {:?}", h.asked());
    assert!(log.iter().any(|l| l.contains("1 of 2")), "log: {log:?}");
}

#[test]
fn an_untaxed_opponent_needs_no_payment() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    perm(&mut g, P1, "Ghostly Prison");
    perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 2, false);
    let h = seat(&mut g, P0, vec![picks(&[0]), pick(1)]); // attack Sephiroth (P2), not Veyran
    combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P2).life, 40 - 6);
    assert_eq!(untapped_lands(&g, P0), 2);
    assert!(!h.asked().iter().any(|r| r.prompt.contains("pay {")));
}
