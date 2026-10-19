//! Smaug the Magnificent ({2}{R}{R}, 4/3 flying, haste), tested in Kefka and Tymna/Kraum with `--swap`.
//!
//! - *At the beginning of your upkeep, create a Treasure token* is the compiled upkeep ability (dsl).
//! - *Whenever Smaug attacks, he deals damage equal to the number of Treasures you control to any target* is
//!   `smaug_attack`: X is counted as the trigger resolves (before blockers). The target: a player it kills (the one with
//!   the least life), else the most valuable creature or planeswalker X damage kills (an opposing flier or reach
//!   creature that could block Smaug counts extra), else the opponent with the least life. X = 0 does nothing.

use crate::cards::{CardDb, Types};
use crate::dsl::has_kw;
use crate::engine::life::{check_state, lose_life};
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::values::{etgh, indestructible, no_damage, pval, untargetable};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

pub const SMAUG: &str = "Smaug the Magnificent";

/// the permanent Smaug's X damage should kill, if any: (it, the damage that kills it)
pub fn smaug_target(g: &Game, p: PlayerId, x: i32) -> Option<(PermId, i32)> {
    let mut best: Option<(PermId, i32, f64)> = None;
    for q in g.opps(p) {
        for &m in &g.player(q).perms {
            let pm = g.perm(m);
            if !pm.on_bf || pm.phased || untargetable(g, m) || no_damage(g, m) || indestructible(g, m) {
                continue;
            }
            let walker = pm.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
            let (cost, mut val) = if g.is_creature(m) {
                (etgh(g, m).max(1), pval(g, m))
            } else if walker {
                (pm.loyalty.unwrap_or(0).max(1), pval(g, m))
            } else {
                continue;
            };
            if cost > x {
                continue;
            }
            // a creature that can block a flier keeps Smaug home
            if g.is_creature(m) && (has_kw(g, m, "flying") || has_kw(g, m, "reach")) {
                val += 2.0;
            }
            if val >= 1.0 && best.is_none_or(|b| val > b.2) {
                best = Some((m, cost, val));
            }
        }
    }
    best.map(|(m, c, _)| (m, c))
}

fn smaug_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) || !g.perm(src).on_bf {
        return Ok(vec![]);
    }
    if g.player(p).treasures == 0 || g.opps(p).next().is_none() {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "damage equal to your Treasures", Some(4.0))? {
        return Ok(vec![]);
    }
    // X is the Treasures you control as the trigger resolves
    let x = g.player(p).treasures as i32;
    if x == 0 {
        return Ok(vec![]);
    }
    if crate::human::is_human(g, p) {
        crate::human::choices::deal_damage(g, p, x, "Smaug", "burn")?;
        return Ok(vec![]);
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if let Some(q) = opps.iter().copied().filter(|&q| g.player(q).life <= x).min_by_key(|&q| g.player(q).life) {
        crate::glog!(g, "    Smaug deals {x} damage to {}", g.player(q).name);
        lose_life(g, q, x, Some(p), "burn", None)?;
        return check_state(g).map(|_| vec![]);
    }
    if let Some((m, cost)) = smaug_target(g, p, x) {
        crate::glog!(g, "    Smaug deals {x} damage to {}", g.perm(m).name);
        if g.perm(m).loyalty.is_some() && !g.is_creature(m) {
            let l = g.perm(m).loyalty.unwrap_or(0) - x;
            g.perm_mut(m).loyalty = Some(l);
            if l <= 0 {
                apply_removal(g, Some(p), m, &format!("dmg{x}"), None)?;
            }
        } else {
            apply_removal(g, Some(p), m, &format!("dmg{}", cost.max(x)), None)?;
        }
    } else if let Some(q) = opps.iter().copied().min_by_key(|&q| g.player(q).life) {
        crate::glog!(g, "    Smaug deals {x} damage to {}", g.player(q).name);
        lose_life(g, q, x, Some(p), "burn", None)?;
    }
    check_state(g).map(|_| vec![])
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, SMAUG)?.attack = Some(smaug_attack);
    Ok(())
}
