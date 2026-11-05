//! Garth One-Eye (JD's `urdragon` deck). Oracle: "{T}: Choose a card name that hasn't been chosen from among
//! Disenchant, Braingeyser, Terror, Shivan Dragon, Regrowth, and Black Lotus. Create a copy of the card with the chosen
//! name. You may cast the copy. (You still pay its costs.)"
//!
//! - The names chosen are kept on the Garth permanent (`DataKey::Garth`, a bit for each name), so a Garth that left
//!   the battlefield and came back (a new permanent) can choose all six again.
//! - The copy is not free: its mana cost is paid through `cost_of` (The Ur-Dragon's eminence takes {1} off Shivan
//!   Dragon) and it is cast with `cast_card` from the zone "copy": a real cast (cast triggers, counterspells, the
//!   stack), no card moves, and a permanent copy enters as a token. A copy that is countered or never cast ceases to
//!   exist. The sorcery-speed names (Braingeyser, Shivan Dragon, Regrowth, Black Lotus) are only offered in a main
//!   phase; Disenchant and Terror also at the end of the turn before Garth's controller's (the instant-speed window).
//! - Black Lotus is only ever a copy: a token artifact that taps for three mana of one colour and is sacrificed
//!   (`rules::petal_sac`).
//! - AI (`options`): Garth is tapped only for a copy that can be cast and used now. Terror / Disenchant: an opposing
//!   target worth removing; Shivan Dragon: the mana is there; Regrowth: a graveyard card worth having; Braingeyser:
//!   X of at least 3 with the spare mana; Black Lotus: when its three mana make a spell castable this turn (else the
//!   last resort). Left out: Braingeyser always targets its
//!   caster; "can't be regenerated" on Terror.

use crate::cards::CardDb;
use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::legal_targets;
use crate::engine::stack::ability_window;
use crate::engine::values::card_name;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::tag::Tag;

pub const GARTH: &str = "Garth One-Eye";
pub const LOTUS: &str = "Black Lotus";

/// the six names, in the bit order of `DataKey::Garth`
pub const NAMES: [&str; 6] = ["Disenchant", "Braingeyser", "Terror", "Shivan Dragon", "Regrowth", LOTUS];
const DISENCHANT: usize = 0;
const BRAINGEYSER: usize = 1;
const TERROR: usize = 2;
const SHIVAN: usize = 3;
const REGROWTH: usize = 4;
const BLACK_LOTUS: usize = 5;

/// Braingeyser draws at most this many (and never leaves fewer than eight cards in the library)
const MAX_X: u32 = 8;

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// has this Garth chosen name `i` already
pub fn chosen(g: &Game, garth: PermId, i: usize) -> bool {
    g.perm(garth).data.int(DataKey::Garth) & (1 << i) != 0
}

/// the names this Garth may still choose
pub fn unchosen(g: &Game, garth: PermId) -> Vec<usize> {
    (0..NAMES.len()).filter(|&i| !chosen(g, garth, i)).collect()
}

/// can Garth's {T} be paid now (untapped, no summoning sickness unless it has haste)
fn can_tap(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m) && !x.tapped && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
}

fn card_of(g: &Game, i: usize) -> Option<CardId> {
    g.db.id(NAMES[i])
}

/// the mana p needs for card c as cost_of reads it, plus `x` more generic
fn affordable(g: &Game, p: PlayerId, c: CardId, x: u32) -> bool {
    let (gn, pips) = cost_of(g, p, c);
    can_pay(g, p, gn + x, &pips, false)
}

// ================================================================== the ability
/// A plan for one name: the copy's card, the target or X it is cast with. `arg` packs the name (low 8 bits) and the
/// target (a permanent id) or X above them.
fn pack(i: usize, extra: u32) -> i64 {
    i as i64 | (extra as i64) << 8
}

fn unpack(arg: i64) -> (usize, u32) {
    ((arg & 0xff) as usize, (arg >> 8) as u32)
}

/// Garth's ability with name `i`: validates the plan again (the state may differ from when the option was made) and
/// returns what the copy is cast with: (card, target, X).
fn plan_for(g: &Game, p: PlayerId, i: usize, extra: u32) -> Option<(CardId, Option<PermId>, u32)> {
    let c = card_of(g, i)?;
    match i {
        TERROR | DISENCHANT => {
            let t = PermId(extra);
            let d = g.db.get(c);
            let kind = d.tags.str(Tag::Rem)?;
            let legal = legal_targets(g, p, kind, d.tags.str(Tag::Tgt)?, false, Some(c));
            (legal.contains(&t) && affordable(g, p, c, 0)).then_some((c, Some(t), 0))
        }
        BRAINGEYSER => (extra >= 1 && affordable(g, p, c, extra)).then_some((c, None, extra)),
        REGROWTH => (!g.player(p).gy.is_empty() && affordable(g, p, c, 0)).then_some((c, None, 0)),
        _ => affordable(g, p, c, 0).then_some((c, None, 0)),
    }
}

/// {T}: choose a name, create the copy, cast it (paying its costs). False if it can't be done now (Garth stays untapped).
fn garth_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (i, extra) = unpack(arg);
    if i >= NAMES.len() || !can_tap(g, src) || g.perm(src).owner != p || chosen(g, src, i) {
        return Ok(false);
    }
    let Some((c, target, x)) = plan_for(g, p, i, extra) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    let mask = g.perm(src).data.int(DataKey::Garth) | (1 << i);
    g.perm_mut(src).data.set(DataKey::Garth, Val::Int(mask));
    crate::glog!(g, "  {} chooses {}: a copy of it", g.perm(src).name, NAMES[i]);
    if !ability_window(g, p, Some(src), &format!("create a copy of {}", NAMES[i]), None, target)? {
        return Ok(true); // countered: no copy
    }
    cast_copy(g, p, c, target, x)?;
    Ok(true)
}

/// "You may cast the copy": pay its costs (and X) and cast it. If it can't be cast, or its target is gone, it
/// ceases to exist. Whatever happens to the copy, no card moves.
pub fn cast_copy(g: &mut Game, p: PlayerId, c: CardId, target: Option<PermId>, x: u32) -> Res<bool> {
    cast_copy_at(g, p, c, target, None, x)
}

/// `cast_copy` with a player target too (a person's Braingeyser: "target player draws X cards")
pub fn cast_copy_at(
    g: &mut Game,
    p: PlayerId,
    c: CardId,
    target: Option<PermId>,
    face: Option<PlayerId>,
    x: u32,
) -> Res<bool> {
    if g.over || !g.player(p).alive || !castable(g, p, c, "copy") {
        return Ok(false);
    }
    if target.is_some_and(|t| !g.perm(t).on_bf) {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn + x, &pips, false) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let paid = pay(g, p, gn + x, &pips, false);
    g.pay_for = None;
    if !paid? {
        return Ok(false);
    }
    let (n_gy, n_ex) = (g.player(p).gy.len(), g.player(p).exile.len());
    let ctx = Ctx { target, face, x: x as i32, ..Ctx::default() };
    let ok = cast_card(g, p, c, "copy", ctx)?;
    // a countered copy ceases to exist instead of going to the graveyard
    let pl = g.player_mut(p);
    if let Some(k) = (n_gy..pl.gy.len()).rev().find(|&k| pl.gy[k] == c) {
        pl.gy.remove(k);
    }
    if let Some(k) = (n_ex..pl.exile.len()).rev().find(|&k| pl.exile[k] == c) {
        pl.exile.remove(k);
    }
    Ok(ok)
}

// ================================================================== the AI
/// the opposing permanent worth the most to remove with `c` (Terror, Disenchant), and its value
fn best_target(g: &Game, p: PlayerId, c: CardId) -> Option<(PermId, f64)> {
    let d = g.db.get(c);
    let legal = legal_targets(g, p, d.tags.str(Tag::Rem)?, d.tags.str(Tag::Tgt)?, false, Some(c));
    let best = crate::engine::zones::max_by(&legal, |m| crate::ai::values::pval_for(g, p, m))?;
    Some((best, crate::ai::values::pval_for(g, p, best)))
}

/// the most X Braingeyser can be cast for with the mana p has (UU paid too), at most MAX_X and what the library lets
fn braingeyser_x(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let (gn, pips) = cost_of(g, p, c);
    let room = (g.player(p).library.len() as u32).saturating_sub(8).min(MAX_X);
    (0..=room.min(total_mana(g, p, false))).rev().find(|&x| can_pay(g, p, gn + x, &pips, false)).unwrap_or(0)
}

/// would Black Lotus's three mana make a spell castable that is not castable now (a card in hand or the commander)
fn lotus_enables(g: &Game, p: PlayerId) -> bool {
    let have = total_mana(g, p, false);
    let pl = g.player(p);
    let cands = pl.hand.iter().copied().filter(|&c| !g.db.get(c).land).chain(pl.cmds_in_zone());
    cands.into_iter().any(|c| {
        let (gn, pips) = cost_of(g, p, c);
        let need = gn + pips.chars().count() as u32;
        need > have && need <= have + 3 && castable(g, p, c, "hand")
    })
}

/// The AI's use of Garth. `post` None is the end-of-turn window (instants only); a main phase also gets the sorcery-
/// speed names. Only a copy that can be cast now and is worth casting is offered, so Garth is never tapped for nothing.
fn options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_tap(g, src) || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let sorcery_ok = post.is_some() && g.stack.is_empty();
    let mut out = vec![];
    let s = crate::ai::brain::Situation::new(g, p);
    for i in unchosen(g, src) {
        let Some(c) = card_of(g, i) else { continue };
        let timing_ok = sorcery_ok || matches!(i, TERROR | DISENCHANT);
        if !timing_ok || !castable(g, p, c, "copy") {
            continue;
        }
        let opt = |utility: f64, what: String, extra: u32| Opt {
            utility,
            label: format!("Garth One-Eye: {what}"),
            act: Some(Action::Ability { src, f: garth_go, arg: pack(i, extra) }),
        };
        match i {
            TERROR | DISENCHANT => {
                let Some((t, v)) = best_target(g, p, c) else { continue };
                let worth = if i == TERROR { 2.5 } else { 2.0 };
                if v < worth || !affordable(g, p, c, 0) {
                    continue;
                }
                // a free removal spell: worth the target, a little more than a card from the hand
                out.push(opt(v - 1.5, format!("{} ({})", NAMES[i], g.perm(t).name), t.0));
            }
            BRAINGEYSER => {
                let x = braingeyser_x(g, p, c);
                if x >= 3 {
                    out.push(opt(2.0 + 0.4 * x as f64, format!("Braingeyser for {x}"), x));
                }
            }
            SHIVAN => {
                if !affordable(g, p, c, 0) {
                    continue;
                }
                let u = crate::ai::brain::card_utility(g, p, &s, c).unwrap_or(3.0);
                out.push(opt(u.max(3.0) + 1.0, "Shivan Dragon".into(), 0));
            }
            REGROWTH => {
                if !affordable(g, p, c, 0) {
                    continue;
                }
                // the card the compiled "regrow" returns: the best by (bomb, mana value)
                let pick = g.player(p).gy.iter().copied().max_by_key(|&x| (g.db.get(x).bomb, g.db.get(x).cmc));
                if let Some(x) = pick.filter(|&x| {
                    let d = g.db.get(x);
                    !d.land && (d.bomb >= 3 || d.cmc >= 3)
                }) {
                    let d = g.db.get(x);
                    out.push(opt(2.0 + 0.4 * d.cmc.min(6) as f64, format!("Regrowth ({})", d.name), 0));
                }
            }
            BLACK_LOTUS => {
                // Black Lotus: a token that taps for three mana of one colour (and goes). Costs nothing to make; it
                // is worth the most when it lets a spell be cast this turn
                out.push(opt(if lotus_enables(g, p) { 6.0 } else { 0.5 }, "Black Lotus".into(), 0));
            }
            _ => {}
        }
    }
    Ok(out)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, GARTH)?.options = Some(options);
    r.card(db, LOTUS)?.on_tap_perm = Some(super::rules::petal_sac);
    Ok(())
}

/// the Garth permanents p controls
pub fn garths(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && card_name(g, m) == Some(GARTH)).collect()
}

/// Garth's ability with name `NAMES[i]` (a target's permanent id, or Braingeyser's X, as `extra`), as the AI's option
/// runs it; false if it can't be done now and Garth stays untapped. For tests and card code that activates it.
pub fn activate(g: &mut Game, garth: PermId, p: PlayerId, i: usize, extra: u32) -> Res<bool> {
    garth_go(g, garth, p, pack(i, extra))
}
