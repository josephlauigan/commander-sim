//! trostani deck (registered 2026-11-27), Batch B: the life-gain cards, populate and the order of their triggers.
//!
//! - **Trostani, Selesnya's Voice**: whenever another creature you control enters, you gain life equal to its toughness,
//!   read as the trigger resolves (its last known toughness if it has left); {1}{G}{W}, {T}: populate (`zur::populate`).
//!   Creature tokens come through `Event::TokensEnter`, which queues one of her enters triggers per token
//!   (`hooks::queue_etb_per_token`, the Haliya pattern). Her trigger is marked late (`CardImpl::late`): it resolves after
//!   the other triggers of the same entering (Emiel's counter, an Archangel of Thune gain, a Cathars' Crusade), so it reads
//!   the toughness they left (CR 603.3b: its controller orders the triggers, and this is the order that gains the most).
//! - **Verdant Sun's Avatar**: the same trigger for itself and for another creature you control.
//! - **Treebeard, Gracious Host**: two Food as it enters; whenever you gain life, that many +1/+1 counters on a Halfling or
//!   Treefolk (here the best one you control: Treebeard), one trigger per gain event.
//! - Rhox Faithmender (life gain doubled, after the "plus" effects) and Archangel of Thune are compiled; tests only.
//! - Test of Endurance (upkeep, 50 life) is `t45_heliod_zur::test_upkeep`, with Felidar Sovereign's 40.
//!
//! The AI uses Trostani's populate at the end of an opponent's turn (the end-of-turn window: the mana would be wasted
//! otherwise), and in its own main phases only when the copy pays at once (a Cathars' Crusade or an Archangel of Thune out,
//! or its life reaching the 40 of a Felidar Sovereign or the 50 of a Test of Endurance).

use crate::engine::life::gain;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{etgh, has_type};
use crate::engine::zones::make_artifact_tokens;
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

const TROSTANI: &str = "Trostani, Selesnya's Voice";
const AVATAR: &str = "Verdant Sun's Avatar";
const TREEBEARD: &str = "Treebeard, Gracious Host";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ================================================================== Trostani and the Avatar
/// "Whenever another creature you control enters, you gain life equal to that creature's toughness" (Trostani), or "this
/// creature or another" (the Avatar: `or_self`). It resolves late (see the module note), and the toughness is read then.
fn life_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId, or_self: bool) -> Res {
    if (m == src && !or_self) || p != crate::engine::hooks::trigger_controller(g, src) || !g.is_creature(m) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "gain life equal to its toughness", Some(2.0))? {
        return Ok(());
    }
    let n = etgh(g, m).max(0); // (a creature that has left: its last known toughness)
    gain(g, p, n)
}

fn trostani_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    life_etb(g, src, p, m, false)
}

fn avatar_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    life_etb(g, src, p, m, true)
}

/// creature tokens entered under p's control (a run-now hook): one of the enters triggers for each token
fn life_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let toks: Vec<PermId> = toks.iter().copied().filter(|&t| g.is_creature(t)).collect();
    crate::engine::hooks::queue_etb_per_token(g, src, p, &toks)
}

/// the enters trigger of a life-gain card: queued, resolving late, and queued once per token for tokens
fn life_card(r: &mut Registry, db: &crate::cards::CardDb, name: &str, etb: crate::hooks::EtbFn) -> Result<(), String> {
    let c = r.card(db, name)?;
    c.etb = Some(etb);
    c.tokens_enter = Some(life_tokens);
    *c = c.enter_queues(Event::TokensEnter).late(Event::Etb);
    Ok(())
}

// ================================================================== Trostani's populate
/// the AI's reasons to populate in its own main phase: the copy pays at once
fn populate_pays_now(g: &Game, p: PlayerId) -> bool {
    let named = |n: &str| g.player(p).perms.iter().any(|&m| live(g, m) && !g.perm(m).neutered && &*g.perm(m).name == n);
    let life = g.player(p).life;
    named("Cathars' Crusade")
        || named("Archangel of Thune")
        || (named("Felidar Sovereign") && life < 40 && life + 3 >= 40)
        || (named("Test of Endurance") && life < 50 && life + 3 >= 50)
}

/// {1}{G}{W}, {T}: populate. The AI does it with a creature token to copy: at an opponent's end step (`post` None), and in
/// its own main phase when the copy pays now.
fn trostani_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || !live(g, src) || !can_populate(g, src, p) {
        return Ok(vec![]);
    }
    if post.is_some() && !(g.active == Some(p) && populate_pays_now(g, p)) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if post.is_none() { 1.5 } else { 2.0 },
        label: "Trostani: populate".into(),
        act: Some(Action::Ability { src, f: trostani_go, arg: 0 }),
    }])
}

fn can_populate(g: &Game, src: PermId, p: PlayerId) -> bool {
    !g.perm(src).tapped
        && !crate::engine::combat::tap_sick(g, src)
        && crate::impls::zur::populate_source(g, p).is_some()
        && can_pay(g, p, 1, "GW", false)
}

fn trostani_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || !can_populate(g, src, p) || !pay(g, p, 1, "GW", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates Trostani: populate", g.player(p).name);
    if ability_window(g, p, Some(src), "populate", None, None)? {
        crate::impls::zur::populate(g, p)?;
    }
    Ok(true)
}

// ================================================================== Treebeard, Gracious Host
/// When Treebeard enters, create two Food tokens (token doublers apply)
fn treebeard_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "create two Food", Some(2.0))? {
        return Ok(());
    }
    make_artifact_tokens(g, p, "Food", 2)
}

/// Whenever you gain life, put that many +1/+1 counters on target Halfling or Treefolk: one trigger for each gain event,
/// n the life really gained (after the doubling). The AI aims it at Treebeard, else at its Halfling or Treefolk with the
/// most toughness.
fn treebeard_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if n <= 0 || p != crate::engine::hooks::trigger_controller(g, src) || !live(g, src) {
        return Ok(());
    }
    let target = if has_type(g, src, "treefolk") || has_type(g, src, "halfling") {
        Some(src)
    } else {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| live(g, m) && g.is_creature(m) && (has_type(g, m, "halfling") || has_type(g, m, "treefolk")))
            .max_by_key(|&m| etgh(g, m))
    };
    let Some(t) = target else { return Ok(()) };
    if !trigger_window(g, p, Some(src), "put +1/+1 counters equal to the life gained", Some(2.0))? || !live(g, t) {
        return Ok(()); // (its only target is gone)
    }
    crate::engine::repl::put_counters(g, p, t, n);
    Ok(())
}

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    life_card(r, db, TROSTANI, trostani_etb)?;
    r.card(db, TROSTANI)?.options = Some(trostani_options);
    life_card(r, db, AVATAR, avatar_etb)?;
    let c = r.card(db, TREEBEARD)?;
    c.etb = Some(treebeard_etb);
    c.gain_life = Some(treebeard_gain);
    Ok(())
}
