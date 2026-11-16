//! Syr Vondam, Sunstar Exemplar (Yoshimaru/Tymna deck): {W}{B}, 2/2, vigilance, menace.
//!
//! - "Whenever another creature you control dies or is put into exile, put a +1/+1 counter on Syr Vondam and you gain
//!   1 life": the `dies` and `exiled_from_bf` hooks. Both look back in time (CR 603.10a): the creature counts as it last
//!   existed on the battlefield (a token too, and whoever controlled it when it left). A creature that leaves in the
//!   same wipe as Syr Vondam still triggers him: `left_with` finds him (he left in the same batch) and queues his
//!   trigger for the creatures that leave after him; the counter then has nowhere to go and the life is gained.
//! - "When Syr Vondam dies or is put into exile while its power is 4 or greater, destroy up to one target nonland
//!   permanent": the `self_dies` and `self_exiled` hooks. The power is the last known one, taken as he leaves
//!   (`leaving`, called by `zones::leave` before anything about him changes: an Equipment still attached counts), and is
//!   not checked again as the ability resolves. The target is chosen as it resolves (the AI: the best opposing
//!   permanent that dies to it, or none; a person: any nonland permanent they may target, or none).

use crate::cards::{CardDb, Types};
use crate::engine::chars::card_types;
use crate::engine::life::gain;
use crate::engine::removal::apply_removal;
use crate::engine::repl;
use crate::engine::stack::{queue_triggers, trigger_window};
use crate::engine::values::{colors_of, epow, indestructible, protected_from, pval, untargetable};
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::hooks::{Call, Event, Registry, Src, TrigAct, Trigger};
use crate::human::cards::pick_perm;
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;

pub const VONDAM: &str = "Syr Vondam, Sunstar Exemplar";

// ------------------------------------------------------------------ last known information
/// `zones::leave`, before the permanent is taken off the battlefield (or off an Equipment): Syr Vondam's power as he
/// leaves, and the wipe (`Game::cur_batch`, 0 outside one) he leaves in. Kept on the permanent that left.
pub fn leaving(g: &mut Game, m: PermId) {
    if g.perm(m).name != VONDAM {
        return;
    }
    let (pow, batch) = (epow(g, m), g.cur_batch.unwrap_or(0));
    g.perm_mut(m).data.set(DataKey::Counters, Val::List(vec![Val::Int(pow as i64), Val::Int(batch as i64)]));
}

/// (the power, the wipe) recorded as the permanent left, if it did
fn last_known(g: &Game, m: PermId) -> Option<(i32, u32)> {
    match g.perm(m).data.get(DataKey::Counters) {
        Some(Val::List(v)) if v.len() == 2 => Some((v[0].int() as i32, v[1].int() as u32)),
        _ => None,
    }
}

// ------------------------------------------------------------------ another creature you control
/// the creature m left the battlefield under `o`'s control: does Syr Vondam (src, controlled by o) see it?
fn sees(g: &Game, src: PermId, m: PermId) -> bool {
    m != src && g.perm(m).owner == g.perm(src).owner && g.is_creature(m)
}

fn another(g: &mut Game, src: Src, m: PermId) -> Res {
    if !sees(g, src, m) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "a +1/+1 counter and 1 life", Some(2.0))? {
        return Ok(());
    }
    if g.perm(src).on_bf && !g.perm(src).phased {
        repl::put_plus(g, src, 1); // (not when he left in the same wipe: the counter has nowhere to go)
    }
    gain(g, o, 1)
}

fn another_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    another(g, src, m)
}

fn another_exiled(g: &mut Game, src: Src, m: PermId) -> Res {
    another(g, src, m)
}

/// `die_triggers` (dying) and `exiled_from_bf` (exiled): the creature m left during a wipe; each Syr Vondam of its
/// controller that left earlier in the same wipe triggers for it too (they left at the same time)
pub fn left_with(g: &mut Game, m: PermId, dying: Option<Sym>) -> Res {
    let Some(b) = g.cur_batch else { return Ok(()) };
    let mut ts = vec![];
    for i in 0..g.perms.len() {
        let v = PermId(i as u32);
        let x = g.perm(v);
        if x.on_bf
            || x.neutered // he had lost his abilities as he left
            || x.name != VONDAM
            || v == m
            || last_known(g, v).is_none_or(|(_, vb)| vb != b)
            || !sees(g, v, m)
        {
            continue;
        }
        let (event, call) = match dying {
            Some(cause) => (Event::Dies, Call::Dies { m, cause }),
            None => (Event::ExiledFromBf, Call::Leaves { m }),
        };
        ts.push(Trigger {
            controller: x.owner,
            src: Some(v),
            act: TrigAct::Hook { event, call },
            name: None,
            known: false,
            imp: None,
            cast_etb: false,
        });
    }
    queue_triggers(g, ts)
}

// ------------------------------------------------------------------ Syr Vondam himself
/// what the destroy can hit: a nonland permanent on the battlefield that `o` may target with Syr Vondam's colours
fn targets(g: &Game, o: PlayerId, src: PermId) -> Vec<PermId> {
    let cols = colors_of(g, src);
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &t in &q.perms {
            let x = g.perm(t);
            if x.on_bf
                && !x.phased
                && !card_types(g, t).has(Types::LAND)
                && (q.id == o || !untargetable(g, t))
                && !protected_from(g, t, cols)
            {
                out.push(t);
            }
        }
    }
    out
}

/// the AI's pick: the opposing permanent most worth destroying that dies to it, if any is worth it
fn ai_target(g: &Game, o: PlayerId, src: PermId) -> Option<PermId> {
    let cs: Vec<PermId> =
        targets(g, o, src).into_iter().filter(|&t| g.perm(t).owner != o && !indestructible(g, t)).collect();
    max_by(&cs, |t| pval(g, t)).filter(|&t| pval(g, t) >= 2.0)
}

/// Syr Vondam (m) left the battlefield (died or was exiled) with power 4 or more
fn destroy(g: &mut Game, m: PermId) -> Res {
    if g.perm(m).neutered || last_known(g, m).is_none_or(|(pow, _)| pow < 4) {
        return Ok(());
    }
    let o = g.perm(m).owner;
    if !trigger_window(g, o, Some(m), "destroy up to one target nonland permanent", Some(5.0))? {
        return Ok(());
    }
    let t = if crate::human::is_human(g, o) {
        let cs = targets(g, o, m);
        if cs.is_empty() {
            return Ok(());
        }
        pick_perm(g, o, "target", "Syr Vondam: destroy which nonland permanent?", &cs, Some("no target"))?
    } else {
        ai_target(g, o, m)
    };
    if let Some(t) = t {
        apply_removal(g, Some(o), t, "destroy", None)?;
    }
    Ok(())
}

fn self_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    destroy(g, m)
}

fn self_exiled(g: &mut Game, _src: Src, m: PermId) -> Res {
    destroy(g, m)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, VONDAM)?;
    c.dies = Some(another_dies);
    c.exiled_from_bf = Some(another_exiled);
    c.self_dies = Some(self_dies);
    c.self_exiled = Some(self_exiled);
    Ok(())
}
