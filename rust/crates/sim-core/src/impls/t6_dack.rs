//! Tier 6 batch `dack`: Dack Fayden's white stax pile (rust/agents/t6-batches.md, audit/t6/dack-fayden-white-stax).
//! Cards only: no engine file is touched. What needs the engine is listed in rust/agents/engine-needs-dack.md.
//!
//! - Fiend Hunter, All-Fates Stalker: banishers built on `common::oring_exile` / `oring_return`.
//! - Touch the Spirit Realm: the same, plus Channel (`touch_channel`).
//!
//!
//! The banishers' enter hooks read "you" from the enter call's `p` (the player the creature entered under), not from
//! the permanent's current controller: Dack puts creatures onto the battlefield under its controller and then
//! donates them, so by the time the queued enter trigger runs the permanent belongs to an opponent but the trigger is
//! still its original controller's (CR 603.3a).

use crate::cards::{CardDb, Types};
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::trigger_window;
use crate::engine::removal::apply_removal;
use crate::engine::values::{pval, untargetable};
use crate::engine::zones::{discard_cards, exile_perm, max_by};
use crate::impls::t2::flicker_worth;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{oring_exile, oring_return};
use crate::state::{Ctx, DataKey, Game, Val};


// ================================================================== Fiend Hunter
/// Full: "you may exile another target creature"; the exiled card returns under its owner's control when Fiend
/// Hunter leaves (a leaves trigger). The AI takes the best opposing creature worth taking (pval >= 1).
fn hunter_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "exile another target creature", Some(5.0))? && g.perm(src).on_bf {
        oring_exile(g, src, p, |g, x| x != src && g.is_creature(x), false, true)?;
    }
    Ok(())
}

fn hunter_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

// ================================================================== All-Fates Stalker
fn has_sub(g: &Game, m: PermId, s: &str) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype(s))
}

/// the ETB: exile up to one target non-Assassin creature until Stalker leaves the battlefield
fn stalker_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src
        && trigger_window(g, p, Some(src), "exile a non-Assassin creature", Some(5.0))?
        && g.perm(src).on_bf
    {
        oring_exile(g, src, p, |g, x| !has_sub(g, x, "assassin"), false, true)?;
    }
    Ok(())
}

/// "until this creature leaves": an effect ending, so the card returns at once
fn stalker_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

/// Warp {1}{W}: cast from hand for {1}{W}; exile it at the beginning of the next end step (its exiled creature comes
/// back); the card may be cast from exile on a later turn. Not offered to the AI: casting from exile needs an
/// exile-zone options hook the engine lacks (engine-needs-dack.md), so a warped Stalker would be stranded.
const WARP_RECAST_SUPPORTED: bool = false;

fn stalker_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !WARP_RECAST_SUPPORTED || post != Some(false) || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    // warp pays when the full cost can't be paid this turn
    let (cg, cp) = cost_of(g, p, c);
    if can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2,
        label: "All-Fates Stalker (warp)".into(),
        act: Some(Action::Plan { f: stalker_warp, arg: c.0 as i64 }),
    }])
}

fn stalker_warp(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

/// a warped Stalker is exiled at the beginning of the next end step
fn stalker_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    All-Fates Stalker (warp) is exiled");
    exile_perm(g, src)
}

// ================================================================== Touch the Spirit Realm
fn is_art_or_creature(g: &Game, m: PermId) -> bool {
    g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

/// Full (both halves): the enchantment's ETB exiles up to one target artifact or creature until it leaves (the AI
/// takes the best opposing one); Channel is `touch_channel`.
fn touch_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "exile an artifact or creature", Some(5.0))? && g.perm(src).on_bf {
        oring_exile(g, src, p, is_art_or_creature, false, false)?;
    }
    Ok(())
}

fn touch_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

fn touch_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let best = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !untargetable(g, m) && is_art_or_creature(g, m))
        .map(|m| pval(g, m))
        .fold(0.0, f64::max);
    if best >= 2.0 { 45 + 30.min((6.0 * best) as i32) } else { 0 }
}

/// Channel -- {1}{W}, discard Touch the Spirit Realm: exile target artifact or creature; it returns to the
/// battlefield under its owner's control at the beginning of the next end step (the engine's `eot_returns` list,
/// which Eerie Interlude uses). A token is gone for good. Returns false when it can't be done.
pub fn touch_channel(g: &mut Game, p: PlayerId, c: CardId, target: PermId) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    let x = g.perm(target);
    if !x.on_bf || x.phased || !is_art_or_creature(g, target) || untargetable(g, target) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    discard_cards(g, p, &[c])?;
    crate::glog!(g, "    {} channels Touch the Spirit Realm on {}", g.player(p).name, g.perm(target).name);
    let (cd, orig, tok) = (g.perm(target).phys.or(g.perm(target).cd), g.perm(target).orig, g.perm(target).token);
    apply_removal(g, Some(p), target, "exile", Some(c))?;
    if !tok
        && !g.perm(target).on_bf
        && let Some(cd) = cd
        && !g.player(orig).is_commander(cd)
        && g.player(orig).exile.contains(&cd)
    {
        g.eot_returns.push((orig, cd));
    }
    Ok(true)
}

fn touch_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, m) = (CardId((arg >> 32) as u16), PermId((arg & 0xffff_ffff) as u32));
    touch_channel(g, p, c, m)
}

/// the AI channels it on its own turn to flicker its best enters-the-battlefield creature (it comes back at the end
/// step, summoning sick but untapped), or to clear a good opposing blocker for the turn
fn touch_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    let arg = |m: PermId| ((c.0 as i64) << 32) | m.0 as i64;
    let mut out = vec![];
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && x.owner == p && x.orig == p && !x.token && g.is_creature(m)
        })
        .collect();
    if let Some(m) = max_by(&mine, |m| flicker_worth(g, p, m))
        && flicker_worth(g, p, m) >= 3.0
    {
        out.push(Opt {
            utility: flicker_worth(g, p, m).min(6.0),
            label: format!("Touch the Spirit Realm (channel) on {}", g.perm(m).name),
            act: Some(Action::Plan { f: touch_go, arg: arg(m) }),
        });
    }
    Ok(out)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Fiend Hunter")?;
    c.etb = Some(hunter_etb);
    c.leaves = Some(hunter_leaves);
    let c = r.card(db, "All-Fates Stalker")?;
    c.etb = Some(stalker_etb);
    c.leaves = Some(stalker_leaves);
    crate::impls::partials::at_once(c, crate::hooks::Event::Leaves, true); // an "until" duration, not a trigger
    c.end_step = Some(stalker_end);
    c.hand_options = Some(stalker_hand);
    let c = r.card(db, "Touch the Spirit Realm")?;
    c.etb = Some(touch_etb);
    c.leaves = Some(touch_leaves);
    crate::impls::partials::at_once(c, crate::hooks::Event::Leaves, true);
    c.prio = Some(touch_prio);
    c.hand_options = Some(touch_hand);
    Ok(())
}
