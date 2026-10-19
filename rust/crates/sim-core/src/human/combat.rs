//! Combat for the human seat: declaring attackers (and which player they attack) and declaring blockers (Python's
//! `play/combat.py`). The engine still resolves combat (damage, first strike, trample, triggers); only the
//! declarations are the person's. Your attackers may attack different players: each player's blocks and damage are
//! worked out in turn. Your attackers may also attack an opponent's planeswalkers. Limitations: each attacker is
//! blocked by at most one creature (menace needs two, and only the first deals damage).

use super::{Target, choices, human_of, legal};
use crate::cards::Types;
use crate::control::{Answer, Request};
use crate::engine::combat::{can_block, has_haste, kw};
use crate::engine::values::{chasm, epow};
use crate::flow::{Res, Stop};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;
use serde_json::{Value, json};

/// could creature m attack now (by the rules, not by whether it should)
pub fn can_attack(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    if !g.is_creature(m) || x.tapped || x.phased || !g.player(p).perms.contains(&m) {
        return false;
    }
    if x.sick && !(g.player(p).haste_all || has_haste(g, m)) {
        return false;
    }
    if kw(g, m, "defender") {
        return false;
    }
    if !g.auras.is_empty() && crate::cardcode::locked(g, m, "pacify") {
        return false; // Arrest, Luminous Bonds
    }
    true
}

pub fn attack_candidates(g: &Game, p: PlayerId) -> Vec<PermId> {
    if chasm(g, p) {
        return vec![]; // Glacial Chasm: your creatures can't attack
    }
    g.player(p).perms.iter().copied().filter(|&m| can_attack(g, p, m)).collect()
}

pub const SPLIT: &str = "split them: choose a player for each creature";

/// the planeswalkers an opponent of p controls that p's creatures may attack (on the battlefield, not phased out)
pub fn attackable_walkers(g: &Game, p: PlayerId) -> Vec<(PermId, PlayerId)> {
    let mut out = vec![];
    for q in g.opps(p).filter(|&q| g.player(q).alive) {
        for &m in &g.player(q).perms {
            let x = g.perm(m);
            if x.on_bf
                && !x.phased
                && x.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))
                && x.loyalty.is_some()
            {
                out.push((m, q));
            }
        }
    }
    out
}

/// "Teferi, Hero of Dominaria (Alice) (planeswalker, 4 loyalty)": describe_target's text, so the page can find it on the table
fn walker_label(g: &Game, p: PlayerId, w: PermId) -> String {
    format!(
        "{} (planeswalker, {} loyalty)",
        legal::describe_target(g, p, Target::Perm(w)),
        g.perm(w).loyalty.unwrap_or(0)
    )
}

/// where one attacker goes
#[derive(Clone, Copy)]
enum Dest {
    Player(PlayerId),
    Walker(PermId, PlayerId),
}

/// ask which creatures attack and whom: [(defending player, attackers)], one entry per player attacked (in turn
/// order), or None for no attack. Creatures that attack a planeswalker are in the group of its controller, and
/// `g.human_walker_atk` names their planeswalker (the engine deals their damage to its loyalty).
pub fn human_attack(g: &mut Game, p: PlayerId, ncomb: u32) -> Res<Option<Vec<(PlayerId, Vec<PermId>)>>> {
    g.human_walker_atk.clear();
    let cands = attack_candidates(g, p);
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive).collect();
    if cands.is_empty() || opps.is_empty() {
        return Ok(None);
    }
    let Some(h) = human_of(g, p) else { return Ok(None) };
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    let extra = if ncomb == 1 { String::new() } else { format!(" (combat {ncomb})") };
    let data = if h.live() { json!({ "refs": choices::table_refs(g, p, &labels) }) } else { json!({}) };
    for _ in 0..5 {
        let req = Request {
            kind: intern("attack"),
            prompt: format!("Declare attackers{extra}: pick any of your creatures, or none"),
            choices: labels.clone(),
            data: data.clone(),
        };
        let picks = match h.ask(g, p, req) {
            Answer::Cancel => return Err(Stop::Cancelled),
            Answer::No | Answer::Decline => return Ok(None),
            Answer::Picks(v) if v.is_empty() => return Ok(None),
            Answer::Picks(v) if v.iter().all(|&i| i < cands.len()) => v,
            _ => {
                h.tell(p, intern("invalid"), "Pick attackers by their numbers, or none.");
                continue;
            }
        };
        let mut atk: Vec<PermId> = vec![];
        for i in picks {
            if !atk.contains(&cands[i]) {
                atk.push(cands[i]);
            }
        }
        let walkers = attackable_walkers(g, p);
        if opps.len() == 1 && walkers.is_empty() {
            return Ok(Some(vec![(opps[0], atk)]));
        }
        // everything that can be attacked: the players, then their planeswalkers
        let mut dests: Vec<Dest> = opps.iter().map(|&q| Dest::Player(q)).collect();
        dests.extend(walkers.iter().map(|&(w, q)| Dest::Walker(w, q)));
        let who: Vec<String> = dests
            .iter()
            .map(|d| match *d {
                Dest::Player(q) => legal::describe_target(g, p, Target::Player(q)),
                Dest::Walker(w, _) => walker_label(g, p, w),
            })
            .collect();
        let mut with_split = who.clone();
        if atk.len() > 1 {
            with_split.push(SPLIT.to_string());
        }
        let prompt = if walkers.is_empty() {
            format!("Attack which player with {} creature(s)?", atk.len())
        } else {
            format!("Attack which player or planeswalker with {} creature(s)?", atk.len())
        };
        let Some(k) = choices::choose(g, p, "target", &prompt, &with_split, Some("cancel"), None)? else {
            continue; // changed their mind: declare again
        };
        let mut to: Vec<(PermId, Dest)> = vec![];
        if k < dests.len() {
            to.extend(atk.iter().map(|&m| (m, dests[k])));
        } else {
            for &m in &atk {
                // split: each creature's player or planeswalker
                let prompt = format!(
                    "{} attacks which {}?",
                    legal::describe_target(g, p, Target::Perm(m)),
                    if walkers.is_empty() { "player" } else { "player or planeswalker" }
                );
                let Some(j) = choices::choose(g, p, "target", &prompt, &who, Some("cancel"), None)? else { break };
                to.push((m, dests[j]));
            }
            if to.len() < atk.len() {
                continue; // cancelled part way: declare again
            }
        }
        let ctl = |d: Dest| match d {
            Dest::Player(q) | Dest::Walker(_, q) => q,
        };
        for &(m, d) in &to {
            if let Dest::Walker(w, _) = d {
                g.human_walker_atk.push((m, w));
            }
        }
        return Ok(Some(
            opps.iter()
                .filter(|&&q| to.iter().any(|x| ctl(x.1) == q))
                .map(|&q| (q, to.iter().filter(|x| ctl(x.1) == q).map(|x| x.0).collect()))
                .collect(),
        ));
    }
    Ok(None)
}

/// ask the defending person which creature blocks each attacker; returns the (attacker, blocker) pairs
pub fn human_blocks(
    g: &mut Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    unbl: &[PermId],
) -> Res<Vec<(PermId, PermId)>> {
    let mut assign: Vec<(PermId, PermId)> = vec![];
    let mut used: Vec<PermId> = vec![];
    let blockers: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased)
        .collect();
    let total: i32 = atk.iter().filter(|&&a| g.player(p).perms.contains(&a)).map(|&a| epow(g, a)).sum();
    let Some(h) = human_of(g, d) else { return Ok(vec![]) };
    let show = h.live();
    let attackers: Vec<Value> = if show {
        let labels: Vec<String> = atk.iter().map(|&a| legal::describe_target(g, d, Target::Perm(a))).collect();
        choices::table_refs(g, d, &labels)
    } else {
        vec![]
    };
    let mut order = atk.to_vec();
    order.sort_by_key(|&m| -epow(g, m)); // stable
    for a in order {
        if unbl.contains(&a) || !g.player(p).perms.contains(&a) {
            continue;
        }
        let cands: Vec<PermId> =
            blockers.iter().copied().filter(|&b| !used.contains(&b) && can_block(g, b, a)).collect();
        if cands.is_empty() {
            continue;
        }
        let menace = kw(g, a, "menace");
        if menace && cands.len() < 2 {
            continue;
        }
        let aname = g.perm(a).name.to_string();
        let what = format!(
            "{}{}",
            legal::describe_target(g, d, Target::Perm(a)),
            if menace { " (menace: needs two blockers)" } else { "" }
        );
        let extra = if show {
            let one = choices::table_refs(g, d, &[legal::describe_target(g, d, Target::Perm(a))]);
            Some(json!({ "attackers": attackers, "attacker": one[0] }))
        } else {
            None
        };
        let labels: Vec<String> = cands.iter().map(|&b| legal::describe_target(g, d, Target::Perm(b))).collect();
        let prompt = format!("{} attacks you ({total} damage in all). Block {what} with?", g.player(p).name);
        let Some(k) = choices::choose(g, d, "block", &prompt, &labels, Some("no block"), extra)? else { continue };
        let b = cands[k];
        if menace {
            let rest: Vec<PermId> = cands.iter().copied().filter(|&x| x != b).collect();
            let labels: Vec<String> = rest.iter().map(|&x| legal::describe_target(g, d, Target::Perm(x))).collect();
            let prompt = format!("Second blocker for {aname} (menace)?");
            let Some(k2) = choices::choose(g, d, "block", &prompt, &labels, Some("no block"), None)? else {
                h.tell(
                    d,
                    intern("invalid"),
                    &format!("{aname} has menace: it needs two blockers, so it is unblocked."),
                );
                continue;
            };
            used.push(rest[k2]);
        }
        assign.push((a, b));
        used.push(b);
        crate::glog!(g, "  {} blocks {} with {}", g.player(d).name, aname, g.perm(b).name);
    }
    Ok(assign)
}
