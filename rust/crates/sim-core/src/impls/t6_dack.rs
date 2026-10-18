//! Tier 6 batch `dack`: Dack Fayden's white stax pile (rust/agents/t6-batches.md, audit/t6/dack-fayden-white-stax).
//! Cards only: no engine file is touched. What needs the engine is listed in rust/agents/engine-needs-dack.md.
//!
//! - Fiend Hunter, All-Fates Stalker: banishers built on `common::oring_exile` / `oring_return`.
//!
//!
//! The banishers' enter hooks read "you" from the enter call's `p` (the player the creature entered under), not from
//! the permanent's current controller: Dack puts creatures onto the battlefield under its controller and then
//! donates them, so by the time the queued enter trigger runs the permanent belongs to an opponent but the trigger is
//! still its original controller's (CR 603.3a).

use crate::cards::CardDb;
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::trigger_window;
use crate::engine::zones::exile_perm;
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
    Ok(())
}
