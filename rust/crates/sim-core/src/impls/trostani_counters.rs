//! trostani deck (registered 2026-11-27): counter-doubling cards (Branching Evolution, Conclave Mentor, Cathars' Crusade,
//! Kalonian Hydra, Wildwood Scourge, The Doctor, Gavony Township). The Opus design is /tmp/handoff/trostani-opus-design.md.
//!
//! - Branching Evolution is `repl::ReplFx::Branching` (+1/+1 counters on a creature you control, twice as many). Conclave
//!   Mentor's replacement ("that many plus one") is Hardened Scales' (`repl::NAMED`); its dies trigger is here.
//! - Cathars' Crusade (`zones::creature_entered`, `make_tokens`) puts each counter through `repl::crusade_counters`: one
//!   placement for each creature and each trigger, so the doublers apply (they never did when it added to `plus` directly).
//! - Wildwood Scourge: it enters with X counters (here, as it enters); its growth is `repl::scourge_grows`, called by every
//!   counters placement (`repl::put_counters`, the enter step).
//! - Kalonian Hydra: enters with four, and its attack trigger puts as many counters again on each creature with some: a
//!   placement for each, so every doubler applies.
//! - The Doctor, EMH: the target gets a counter, then each creature you control with any counter on it gets one.
//! - Gavony Township: the AI's use of {2}{G}{W},{T} (its own mana pays for nothing: the cost taps it).

use crate::cards::CardDb;
use crate::engine::life::gain;
use crate::engine::mana::total_mana;
use crate::engine::repl;
use crate::engine::stack::{ability_window_card, trigger_window};
use crate::engine::values::{colors_of, epow, protected_from, pval, untargetable, untargetable_by_its_controller};
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::lands::{can_pay_without, pay_without};
use crate::state::{DataKey, Game};
use crate::sym::Sym;

const KALONIAN: &str = "Kalonian Hydra";
const SCOURGE: &str = "Wildwood Scourge";
const MENTOR: &str = "Conclave Mentor";
const DOCTOR: &str = "The Doctor, EMH";
const GAVONY: &str = "Gavony Township";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// p's creatures on the battlefield
fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

// ================================================================== Kalonian Hydra
/// "This creature enters with four +1/+1 counters on it": placed counters (614.1c), so every doubler applies
fn kalonian_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    repl::put_counters(g, p, m, 4);
    Ok(())
}

/// "Whenever this creature attacks, double the number of +1/+1 counters on each creature you control." To double N,
/// put N more: a placement for each creature, so Doubling Season, Branching Evolution, Hardened Scales and the rest apply
/// (N with Doubling Season makes 3N). A creature with none gets none (so Conclave Mentor adds nothing to it). The
/// numbers are read as the trigger resolves, all before any counter goes on.
fn kalonian_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) {
        // (no on-battlefield guard: the trigger exists without its source, CR 113.7a)
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "double the +1/+1 counters on each creature you control", Some(5.0))? {
        return Ok(vec![]);
    }
    let each: Vec<(PermId, i32)> =
        creatures(g, p).into_iter().map(|m| (m, g.perm(m).plus)).filter(|&(_, n)| n > 0).collect();
    for (m, n) in each {
        if g.perm(m).on_bf {
            repl::put_counters(g, p, m, n);
        }
    }
    crate::glog!(g, "    {} doubles the +1/+1 counters on {}'s creatures", g.perm(src).name, g.player(p).name);
    g.bf_ver += 1;
    Ok(vec![])
}

// ================================================================== Wildwood Scourge
/// "This creature enters with X +1/+1 counters on it": X is the X it was cast with (`Enter::x`, kept as `CastX`); a copy
/// or a card put onto the battlefield has X = 0 and dies
fn scourge_as(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let x = g.perm(m).data.int(DataKey::CastX) as i32;
    if x > 0 {
        repl::put_counters(g, p, m, x);
    }
    Ok(())
}

/// the X the AI pays for the Scourge: what it has to spare, at most five (the rest keeps mana for Trostani and the
/// activations)
fn scourge_x(_g: &Game, _p: PlayerId, _c: CardId, spare: u32) -> u32 {
    spare.min(5)
}

/// not cast for less than X = 2 (a 0/0 dies, a 1/1 is a waste of the card)
fn scourge_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 3 { 30 } else { 0 }
}

// ================================================================== Conclave Mentor
/// "When this creature dies, you gain life equal to its power." The power it last had (its counters count)
fn mentor_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if src != m {
        return Ok(());
    }
    let o = g.perm(m).owner;
    let n = epow(g, m);
    if n > 0 && trigger_window(g, o, Some(m), "gain life equal to its power", Some(2.0))? {
        gain(g, o, n)?;
    }
    Ok(())
}

// ================================================================== The Doctor, EMH
/// a creature with a counter on it (+1/+1 or -1/-1, loyalty, charge, lore, time, the engine's other counters)
fn has_counters(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.plus != 0
        || x.loyalty.is_some_and(|l| l > 0)
        || [DataKey::Charge, DataKey::Counters, DataKey::Lore, DataKey::Vanish].iter().any(|&k| x.data.int(k) > 0)
}

/// can the ability of `src` that `o` controls (CR 113.8: not whoever controls `src` now) target `m`: not hexproof or shroud against an opponent, and no protection from a colour of the
/// source (our own creature: hexproof lets us target it)
fn targetable_by(g: &Game, o: PlayerId, src: PermId, m: PermId) -> bool {
    if g.perm(m).owner == o {
        return !untargetable_by_its_controller(g, m, colors_of(g, src));
    }
    !(untargetable(g, m) || protected_from(g, m, colors_of(g, src)))
}

/// the creatures the trigger may target: any creature on the battlefield, ours or an opponent's
fn doctor_targets(g: &Game, o: PlayerId, src: PermId) -> Vec<PermId> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && targetable_by(g, o, src, m))
        .collect()
}

/// What a counter on t is worth to o as the Doctor's target. Only the target is counted twice: a creature of ours with no
/// counter gets two (its own and the second sentence's), one that already has counters would have got the second anyway.
/// So our creature with none first (the best one, `pval`), then our best creature, and an opponent's creature only when
/// nothing of ours can be the target (it must target)
fn doctor_worth(g: &Game, o: PlayerId, t: PermId) -> f64 {
    if g.perm(t).owner != o {
        return -1000.0 + pval(g, t);
    }
    pval(g, t) + if has_counters(g, t) { 0.0 } else { 100.0 }
}

/// "When The Doctor enters, put a +1/+1 counter on target creature. Then put a +1/+1 counter on each creature you control
/// with a counter on it." One target, any controller's; if it is illegal as the trigger resolves, the whole ability does
/// nothing (608.2b), the second sentence too.
fn doctor_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    let cands = doctor_targets(g, o, src);
    if cands.is_empty() {
        return Ok(());
    }
    const NAME: &str = "a +1/+1 counter on target creature, then on each creature you control with a counter";
    if g.trig_probe {
        // only whether the trigger goes on the stack: its target is chosen (and a person asked) once, as it does
        return trigger_window(g, o, Some(src), NAME, Some(5.0)).map(|_| ());
    }
    let target = if human(g, o) {
        let prompt = "The Doctor: put a +1/+1 counter on which creature?";
        crate::human::cards::pick_perm(g, o, "target", prompt, &cands, None)?.unwrap_or(cands[0])
    } else {
        max_by(&cands, |t| doctor_worth(g, o, t)).unwrap_or(cands[0])
    };
    if !trigger_window(g, o, Some(src), NAME, Some(5.0))? {
        return Ok(());
    }
    doctor_resolve(g, src, o, target)
}

/// the Doctor's ability as it resolves: nothing if `target` is no longer a legal target; else a counter on it, then one on
/// each creature `o` controls that has a counter on it (the target too, if it is `o`'s). Every placement is its own event
/// (Doubling Season, Branching Evolution ...); the creatures are fixed before any counter goes on
pub fn doctor_resolve(g: &mut Game, src: PermId, o: PlayerId, target: PermId) -> Res {
    if !live(g, target) || !g.is_creature(target) || !targetable_by(g, o, src, target) {
        return Ok(());
    }
    if g.perm(target).owner != o && !crate::engine::ward::pay_ward(g, o, target)? {
        return Ok(());
    }
    let k = repl::put_counters(g, o, target, 1);
    // the target has a counter now even if a -1/-1 counter it had was cancelled by it (SBA 704.5q comes later)
    let each: Vec<PermId> =
        creatures(g, o).into_iter().filter(|&m| has_counters(g, m) || (m == target && k > 0)).collect();
    for m in each {
        if g.perm(m).on_bf {
            repl::put_counters(g, o, m, 1);
        }
    }
    crate::glog!(g, "    {}: counters on {} and each creature with a counter", DOCTOR, g.perm(target).name);
    g.bf_ver += 1;
    Ok(())
}

// ================================================================== Gavony Township
/// (creatures p controls, the +1/+1 counters one activation would really put in all, doublers included)
fn gavony_gain(g: &Game, p: PlayerId) -> (usize, i32) {
    let cs = creatures(g, p);
    let total = cs
        .iter()
        .map(|&m| repl::counters(g, p, repl::On::Perm(m), repl::Ctr::Plus, 1, repl::Path::ALL).max(0))
        .fold(0i32, |a, k| a.saturating_add(k));
    (cs.len(), total)
}

/// {2}{G}{W}, {T}: a +1/+1 counter on each creature you control. The AI uses it with mana to spare, paid by other sources
/// (the cost taps Gavony itself), when at least two creatures grow: at an opponent's end step, worth 0.35 for each counter
/// that goes on (a doubler makes it count twice), or before its own combat in the first main phase with three or more
/// creatures, worth less (the mana it costs is the spells' first)
fn gavony_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped {
        return Ok(vec![]);
    }
    let eot = post.is_none() && g.active != Some(p);
    let main = post == Some(false) && g.active == Some(p);
    if !eot && !main {
        return Ok(vec![]);
    }
    let (n, total) = gavony_gain(g, p);
    if n < if eot { 2 } else { 3 } || !can_pay_without(g, p, l, 2, "GW") {
        return Ok(vec![]);
    }
    let u = (if eot { 0.35 } else { 0.2 }) * total as f64;
    Ok(vec![Opt {
        utility: u,
        label: format!("{GAVONY}: a +1/+1 counter on each of {n} creatures"),
        act: Some(Action::Plan { f: gavony_go, arg: l.0 as i64 }),
    }])
}

fn gavony_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !pay_without(g, p, l, 2, "GW")? {
        return Ok(false);
    }
    g.perm_mut(l).tapped = true;
    let cd = g.perm(l).card();
    crate::glog!(g, "  {} activates {}", g.player(p).name, GAVONY);
    if !ability_window_card(g, p, cd, "a +1/+1 counter on each creature you control", Some(4.0), None)? {
        return Ok(true);
    }
    for m in creatures(g, p) {
        repl::put_counters(g, p, m, 1);
    }
    g.bf_ver += 1;
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, KALONIAN)?;
    c.as_enters = Some(kalonian_as);
    c.attack = Some(kalonian_attack);
    let c = r.card(db, SCOURGE)?;
    c.as_enters = Some(scourge_as);
    c.cast_x = Some(scourge_x);
    c.prio = Some(scourge_prio);
    r.card(db, MENTOR)?.self_dies = Some(mentor_dies);
    r.card(db, DOCTOR)?.etb = Some(doctor_etb);
    r.card(db, GAVONY)?.land_options = Some(gavony_options);
    Ok(())
}
