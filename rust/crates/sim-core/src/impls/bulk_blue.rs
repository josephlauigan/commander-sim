//! Card code for the blue spells of the elrond deck (decklists/JD/elrond-ring-tempts.md).
//! Filled in by the blue-spells agent (rust/agents/blue-spells.md).
//!
//! - Jace, Reality Sculptor (walker framework + the empower Jace token of `alela.rs`).

use super::common::{WalkerAb, walker};
use crate::cards::CardDb;
use crate::engine::values::card_name;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

pub const JACE_RS: &str = "Jace, Reality Sculptor";

// ================================================================== Jace, Reality Sculptor
/// the Jace planeswalkers p controls (the Jace token and Jace, Reality Sculptor)
fn jaces(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| !g.perm(m).phased && g.perm(m).on_bf && card_name(g, m).is_some_and(|n| n.starts_with("Jace")))
        .collect()
}

/// loyalty counters among the Jaces p controls
fn jace_loyalty(g: &Game, p: PlayerId) -> i32 {
    jaces(g, p).iter().map(|&m| g.perm(m).loyalty.unwrap_or(0)).sum()
}

/// Islands p controls (basic Islands and anything with the Island type)
fn islands(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .lands
        .iter()
        .filter(|&&l| {
            let d = g.db.get(g.land(l).cd);
            &*d.name == "Island" || d.has_subtype("island")
        })
        .count() as i32
}

/// +1: empower Jace X, X = Islands you control (the Jace token gets X loyalty counters)
fn jace_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let x = islands(g, p);
    Some(1.5 + (x as f64 / 3.0).min(3.0))
}

fn jace_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let x = islands(g, p);
    if x > 0 {
        super::alela::empower(g, p, x);
    }
    Ok(())
}

/// -3: until your next turn, creatures attacking you get -5/-0. Worth it when an opponent has a real attack and
/// there is loyalty to spare.
fn jace_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if g.perm(src).loyalty.unwrap_or(0) <= 3 {
        return None;
    }
    let big: i32 = g
        .opps(p)
        .map(|q| {
            g.player(q)
                .perms
                .iter()
                .filter(|&&m| g.is_creature(m) && !g.perm(m).phased)
                .map(|&m| crate::engine::values::epow(g, m).min(5))
                .sum::<i32>()
        })
        .sum();
    if big < 8 {
        return None;
    }
    Some(1.0 + big as f64 / 8.0)
}

fn jace_minus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    g.perm_mut(src).data.set(DataKey::Used, Val::Bool(true));
    crate::glog!(g, "    until {}'s next turn, creatures attacking them get -5/-0", g.player(p).name);
    Ok(())
}

/// 0: exile all but the bottom card of each opponent's library; only with 25 or more loyalty counters among your Jaces
fn jace_zero_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    (jace_loyalty(g, p) >= 25).then_some(30.0)
}

fn jace_zero(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if jace_loyalty(g, p) < 25 {
        return Ok(()); // the activation condition is checked again as it resolves
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let pl = g.player_mut(q);
        let rest: Vec<_> = pl.library.drain(1.min(pl.library.len())..).collect();
        crate::glog!(g, "    exiles {} cards from {}'s library", rest.len(), g.player(q).name);
        g.player_mut(q).exile.extend(rest);
    }
    Ok(())
}

static JACE_RS_ABILITIES: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "empower Jace X (X = Islands)", val: jace_plus_val, eff: jace_plus },
    WalkerAb { delta: -3, label: "attackers get -5/-0 until your next turn", val: jace_minus_val, eff: jace_minus },
    WalkerAb { delta: 0, label: "exile all but the bottom card of each opponent's library", val: jace_zero_val, eff: jace_zero },
];

/// Jace's -3 lasts until your next turn
fn jace_upkeep(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if g.perm(src).owner == p {
        g.perm_mut(src).data.set(DataKey::Used, Val::Bool(false));
    }
    Ok(())
}

/// the -3: a creature attacking the Jace's controller gets -5/-0 until end of turn
fn jace_attack(g: &mut Game, src: PermId, _p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == d && g.perm(src).on_bf && g.perm(src).data.truthy(DataKey::Used) {
        for &a in atk {
            if g.perm(a).on_bf {
                super::common::eot(g, a, -5, 0);
            }
        }
    }
    Ok(vec![])
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    walker(r, db, JACE_RS, &JACE_RS_ABILITIES)?;
    let c = r.card(db, JACE_RS)?;
    c.upkeep = Some(jace_upkeep);
    c.attack = Some(jace_attack);
    Ok(())
}
