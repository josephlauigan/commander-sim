//! The Tymna partner decks (`thrasios`: Tymna the Weaver + Thrasios, Triton Hero; `kraum`: Tymna + Kraum, Ludevic's
//! Opus). The seat has two commanders (`Player::cmd` and `Player::cmd2`); the engine casts both from the command
//! zone with the generic commander priority. The decks' own AI plans are written later: for now both use the
//! default priority.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// ai/decks.rs `deck_prio` for the thrasios seat (the default until the plan is written)
pub fn thrasios_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

/// ai/decks.rs `deck_prio` for the kraum seat (the default until the plan is written)
pub fn kraum_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

// ------------------------------------------------------------------ measurement
/// `ty_tt_<name>` / `ty_tn_<name>`: the first turn (of the owner's own turn count) a partner-deck commander or Tymna came
/// down, summed over games, and the number of games it did (the analysis report prints the mean).
fn landed(g: &mut Game, src: crate::hooks::Src, _p: PlayerId, m: crate::ids::PermId) -> crate::flow::Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let name = crate::engine::values::card_name(g, src).unwrap_or("?").split(',').next().unwrap_or("?").replace(' ', "_");
    let turns = g.player(o).turns as i64;
    let pl = g.player_mut(o);
    let key = crate::sym::intern(&format!("ty_tn_{name}"));
    if pl.stats.contains_key(&key) {
        pl.stat(crate::sym::intern(&format!("ty_re_{name}")), 1); // cast again after it died
        return Ok(());
    }
    pl.stat(crate::sym::intern(&format!("ty_tt_{name}")), turns);
    pl.stat(key, 1);
    Ok(())
}

pub fn register(r: &mut crate::hooks::Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    for name in ["Tymna the Weaver", "Thrasios, Triton Hero", "Kraum, Ludevic's Opus"] {
        if db.id(name).is_some() {
            r.card(db, name)?.etb = Some(landed);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ spreading the attack
use crate::engine::combat::{attack_restrictions, can_block, double_strike, fiery, kw};
use crate::engine::values::{card_name, epow, pval, shielded};
use crate::ids::PermId;

/// Tymna's draw (or Reconnaissance Mission's / The Aetherspark's) is waiting on the battlefield
fn hit_engine_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && matches!(card_name(g, m), Some("Tymna the Weaver" | "Reconnaissance Mission" | "The Aetherspark"))
    })
}

/// `a` would connect with player q: none of q's untapped creatures can block it (menace: not even one blocker)
fn connects(g: &Game, q: PlayerId, a: PermId) -> bool {
    let n = g
        .player(q)
        .perms
        .iter()
        .filter(|&&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased && can_block(g, b, a))
        .count();
    n == 0 || (n == 1 && kw(g, a, "menace"))
}

/// combat.rs, after the normal split: while Tymna (or the other hit-engines) is out, make sure every living
/// opponent who can be hit takes at least one attacker that will connect, the cheapest such attacker first, without
/// breaking an attack that is lethal. `pool` is every creature that could attack. Only the thrasios and kraum seats.
pub fn spread(
    g: &Game,
    p: PlayerId,
    groups: Vec<(PlayerId, Vec<PermId>)>,
    pool: &[PermId],
) -> Vec<(PlayerId, Vec<PermId>)> {
    if !matches!(g.player(p).key, "thrasios" | "kraum") || !hit_engine_out(g, p) {
        return groups;
    }
    let hit = |a: PermId| epow(g, a) * if double_strike(g, a) { 2 } else { 1 } * fiery(g, p);
    let mut groups = groups;
    let opps: Vec<PlayerId> =
        g.opps(p).filter(|&q| g.player(q).alive && !shielded(g, q) && attack_restrictions(g, p, q).0 == 0).collect();
    let assigned = |gr: &[(PlayerId, Vec<PermId>)], a: PermId| gr.iter().position(|x| x.1.contains(&a));
    // lethal groups: the damage that can't be blocked reaches the life total
    let lethal = |gr: &[(PlayerId, Vec<PermId>)], i: usize| {
        let (q, xs) = &gr[i];
        xs.iter().filter(|&&a| connects(g, *q, a)).map(|&a| hit(a)).sum::<i32>() >= g.player(*q).life
    };
    let covered = |gr: &[(PlayerId, Vec<PermId>)], q: PlayerId| {
        gr.iter().any(|x| x.0 == q && x.1.iter().any(|&a| epow(g, a) > 0 && connects(g, q, a)))
    };
    // the opponents that can be hit by the fewest creatures first
    let cands = |q: PlayerId| -> Vec<PermId> {
        pool.iter().copied().filter(|&a| epow(g, a) > 0 && connects(g, q, a)).collect()
    };
    let mut order: Vec<PlayerId> = opps.into_iter().filter(|&q| !covered(&groups, q)).collect();
    order.sort_by_key(|&q| cands(q).len());
    for q in order {
        let mut cs: Vec<PermId> = cands(q);
        cs.sort_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
        for a in cs {
            let from = assigned(&groups, a);
            if from.is_none() && (g.perm(a).is_cmd || card_name(g, a) == Some("Tymna the Weaver")) {
                continue; // a held-back commander stays home
            }
            if let Some(i) = from {
                let (gq, xs) = (groups[i].0, &groups[i].1);
                if gq == q {
                    continue;
                }
                // not the only attacker that hits its opponent, nor part of a lethal attack
                let other_hit = xs.iter().any(|&x| x != a && epow(g, x) > 0 && connects(g, gq, x));
                if connects(g, gq, a) && !other_hit {
                    continue;
                }
                if lethal(&groups, i) {
                    let mut t = groups.clone();
                    t[i].1.retain(|&x| x != a);
                    if !lethal(&t, i) {
                        continue;
                    }
                }
                groups[i].1.retain(|&x| x != a);
            }
            match groups.iter().position(|x| x.0 == q) {
                Some(j) => groups[j].1.push(a),
                None => groups.push((q, vec![a])),
            }
            break;
        }
    }
    groups.retain(|x| !x.1.is_empty());
    groups
}
