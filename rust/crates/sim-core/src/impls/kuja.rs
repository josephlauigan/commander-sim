//! Kuja, Genome Sorcerer // Trance Kuja, Fate Defied (Kefka's deck; bought 10/19, tested with `--swap`).
//!
//! - **Kuja** (3/4): at the beginning of its controller's end step, a tapped 0/1 black Wizard token is created, then if
//!   the controller has four or more Wizards (Kefka and Niv-Mizzet count, and so do the tokens) Kuja transforms.
//!   Transformed (`DataKey::Flipped`, kept) it is Trance Kuja, a 4/6.
//! - **The token** carries `DataKey::KujaWiz`: whenever its controller casts a noncreature spell it deals 1 damage to
//!   each opponent (`wizard_pings`, called by `engine::cast` next to prowess). It lives on without Kuja.
//! - **Flare Star** (Trance Kuja): damage a Wizard its controller controls would deal is doubled: `flare_star` is the
//!   multiplier for a source permanent. Read by the token pings, by combat damage (`engine::combat`: to blockers and to
//!   the player, trample excess too) and by the DSL damage effect (Niv-Mizzet's draw trigger). Damage a Wizard deals
//!   through other card code (none in Kefka's list) is not doubled.

use crate::cards::{CardDb, Colors};
use crate::engine::life::lose_life;
use crate::engine::stack::trigger_window;
use crate::engine::values::{card_name, has_type};
use crate::engine::zones::{Tokens, make_tokens};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

pub const KUJA: &str = "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied";

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// Kuja has transformed into Trance Kuja
pub fn trance(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped) && card_name(g, m) == Some(KUJA)
}

fn wizards(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m) && has_type(g, m, "wizard")).count()
}

/// p controls a Trance Kuja
pub fn trance_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && trance(g, m))
}

/// Flare Star: the damage multiplier for damage that source `src` (a permanent, if there is one) deals: 2 when a Wizard
/// its controller controls would deal it and its controller has a Trance Kuja, else 1
pub fn flare_star(g: &Game, src: Option<PermId>) -> i32 {
    let Some(m) = src else { return 1 };
    if !g.perm(m).on_bf {
        return 1;
    }
    let o = g.perm(m).owner;
    if g.is_creature(m) && has_type(g, m, "wizard") && trance_out(g, o) { 2 } else { 1 }
}

/// at the beginning of your end step: a tapped 0/1 Wizard, then transform with four or more Wizards
fn kuja_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || !live(g, src) {
        return Ok(());
    }
    if trance(g, src) {
        // Trance Kuja has the same trigger on its front face only: no more tokens once transformed
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "create a tapped 0/1 Wizard", None)? {
        return Ok(());
    }
    let spec = Tokens { color: Some(Colors::from_letters("B")), types: vec!["wizard"], ..Tokens::new(1, 0) };
    let spec = Tokens { tgh: Some(1), ..spec };
    for m in make_tokens(g, p, spec)? {
        let x = g.perm_mut(m);
        x.tapped = true;
        x.data.set(DataKey::KujaWiz, Val::Bool(true));
    }
    if live(g, src) && wizards(g, p) >= 4 {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Flipped, Val::Bool(true));
        x.pow = 4;
        x.tgh = 6;
        crate::glog!(g, "    Kuja transforms into Trance Kuja, Fate Defied (4/6)");
    }
    Ok(())
}

/// p cast a noncreature spell: each of their Kuja Wizard tokens deals 1 damage to each opponent (doubled under Flare Star)
pub fn wizard_pings(g: &mut Game, p: PlayerId) -> Res {
    let toks: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.perm(m).token && g.perm(m).data.truthy(DataKey::KujaWiz))
        .collect();
    for m in toks {
        if !live(g, m) {
            continue;
        }
        let dmg = flare_star(g, Some(m));
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, dmg, Some(p), "burn", None)?;
        }
    }
    Ok(())
}

/// Kuja's cast priority in Kefka's deck: a 3/4 that makes a pinger every turn; sooner when Wizards are out already
pub fn kuja_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    if &*g.db.get(c).name != KUJA {
        return None;
    }
    Some(if wizards(g, p) >= 2 { 70 } else { 62 })
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(KUJA).is_some() {
        r.card(db, KUJA)?.end_step = Some(kuja_end);
    }
    Ok(())
}
