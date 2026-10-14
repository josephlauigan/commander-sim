//! The Sephiroth FS deck's plan (`sephfs`, mono-black: Sephiroth, Fabled SOLDIER; wipe the board, drain on every death
//! and take the dead). Sephiroth himself is in `sheoldred.rs` (shared with the sheoldred deck); the deck's own cards
//! are in `sephfs_cards.rs`. This module holds what the deck decides:
//!
//! - **Cast order** (`sephfs_prio`): Sephiroth first, the free sacrifice outlets and death payoffs with him; the edicts
//!   (Plaguecrafter, Fleshbag Marauder, Accursed Marauder) when the deaths they cause reach the fourth drain of the
//!   turn (the flip), otherwise held; Massacre Girl as a wipe.
//! - **Wipes** (`wipe_utility`): valued with the drain for every creature that dies; an unflipped Sephiroth is spared
//!   unless the wipe's own deaths flip him first; Extinction Event picks its parity (`sephfs_cards::parity_choice`);
//!   Living Death by what each graveyard gives back.
//! - **Steals**: Reanimate, Animate Dead and the prepared Reanimate take the best creature card from any graveyard
//!   (`rean_targets`).
//! - Stats for `--analyze`: `sf_*` counters (flips, wipes, steals by card, sacrifices, cards drawn by Sephiroth).

use super::sephfs_cards::{all_creatures, cmc_of, flipped, is_seph, parity_choice, steal_targets};
use crate::ai::pool::generic_prio;
use crate::cards::CardDb;
use crate::engine::values::pval;
use crate::hooks::Registry;
use crate::ids::{CardId, CardId as C, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::tag::Tag;

pub const KEY: &str = "sephfs";

fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p's Sephiroth on the battlefield and how many drains have resolved this turn (4 flips him)
fn seph_state(g: &Game, p: PlayerId) -> Option<(PermId, i64, bool)> {
    let m = g
        .player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && !g.perm(m).phased && is_seph(g, m) && g.perm(m).owner == p)?;
    let st = g.turn_stamp();
    let n = match g.perm(m).data.get(DataKey::SephTimes) {
        Some(Val::List(l)) if matches!(l.first(), Some(Val::Stamp(s)) if *s == st) => l.get(1).map_or(0, Val::int),
        _ => 0,
    };
    Some((m, n, flipped(g, m)))
}

/// how many creatures opponents of p control
fn opp_creatures(g: &Game, p: PlayerId) -> usize {
    all_creatures(g).into_iter().filter(|&(q, _)| q != p).count()
}

/// how many opponents control at least one creature
fn opps_with_creature(g: &Game, p: PlayerId) -> usize {
    g.opps(p).filter(|&q| all_creatures(g).iter().any(|&(x, _)| x == q)).count()
}

/// creatures p could sacrifice for nothing worse than a drain: not Sephiroth himself, not the commander
fn own_fodder(g: &Game, p: PlayerId) -> usize {
    all_creatures(g).into_iter().filter(|&(q, m)| q == p && !is_seph(g, m) && !g.perm(m).is_cmd).count()
}

/// free sacrifice outlets p has on the battlefield
fn outlets(g: &Game, p: PlayerId) -> usize {
    super::common::outlets(g, p).len()
}

/// the edicts: each player sacrifices a creature
const EDICTS: [&str; 3] = ["Plaguecrafter", "Fleshbag Marauder", "Accursed Marauder"];

/// the deck's cast priority (0-90)
pub fn sephfs_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = generic_prio(g, p, c);
    let d = g.db.get(c);
    let n = &*d.name;
    let seph = seph_state(g, p);
    if EDICTS.contains(&n) {
        let Some((_, times, fl)) = seph else {
            return if base > 0 { 25 } else { 0 }; // Sephiroth is not out: hold the edict
        };
        if fl {
            // the emblem is out: an edict is a drain per death, worth it when opponents have creatures
            return if opps_with_creature(g, p) >= 2 { 66 } else { 30 };
        }
        // deaths this edict causes: one per opponent with a creature, ours (the cheapest), plus outlets' fodder
        let deaths = opps_with_creature(g, p) as i64 + 1 + (outlets(g, p).min(own_fodder(g, p))) as i64;
        return if times + deaths >= 4 { 92 } else { 20 };
    }
    match n {
        "Rabid Attack" => return 0, // cast through `options`, ahead of a sweeper
        "Massacre Girl" => return massacre_prio(g, p, base),
        "Pestilence" => {
            return if seph.is_some() || g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.is_creature(m)) {
                58
            } else {
                36
            };
        }
        "Grave Betrayal" => return if opp_creatures(g, p) >= 2 { 62 } else { 25 },
        "Kaya's Ghostform" => {
            return match seph {
                Some((_, _, false)) => 86, // protect him before he flips
                _ if g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.is_creature(m)) => 28,
                _ => 0,
            };
        }
        "Phyrexian Altar" | "Viscera Seer" | "Woe Strider" | "Carrion Feeder" | "Yahenni, Undying Partisan" => {
            return if seph.is_some() { base.max(72) } else { base.max(55) };
        }
        "Blood Artist"
        | "Zulaport Cutthroat"
        | "Bastion of Remembrance"
        | "Butcher of Malakir"
        | "Pitiless Plunderer" => {
            return base.max(56);
        }
        _ => {}
    }
    base
}

/// Massacre Girl: a wipe on a body. Cast it when the board is the opponents'.
fn massacre_prio(g: &Game, p: PlayerId, base: i32) -> i32 {
    let theirs: f64 = all_creatures(g).iter().filter(|&&(q, _)| q != p).map(|&(_, m)| pval(g, m)).sum();
    let ours: f64 = all_creatures(g).iter().filter(|&&(q, _)| q == p).map(|&(_, m)| pval(g, m)).sum();
    if opp_creatures(g, p) >= 3 && theirs > 1.2 * ours + 3.0 { 88 } else { base.min(20) }
}

/// the utility of casting wipe card c of kind `kind` (None: don't), on brain.wipe_options' scale
pub fn wipe_utility(g: &Game, p: PlayerId, c: CardId, kind: &str, danger: f64) -> Option<f64> {
    let dv = super::common::death_value(g, p, None);
    let all = all_creatures(g);
    let n_own = all.iter().filter(|&&(q, _)| q == p).count();
    let n_opp = all.len() - n_own;
    let seph = seph_state(g, p).filter(|s| !s.2);
    let (mut gain, mut loss, deaths): (f64, f64, usize) = match kind {
        "parity" => {
            let (par, ol, ml) = parity_choice(g, p);
            let _ = par;
            (ol, ml, 0) // exiled: nobody dies, no drain
        }
        "living" => {
            let back = |q: PlayerId| -> f64 {
                g.player(q).gy.iter().map(|&x| 0.8 * super::sephfs_cards::steal_value(g, x)).sum::<f64>()
            };
            let board = |q: PlayerId, mine: bool| -> f64 {
                all.iter().filter(|&&(x, _)| (x == p) == mine && (mine || x == q)).map(|&(_, m)| pval(g, m)).sum()
            };
            let mut ol = 0.0;
            for q in g.opps(p) {
                ol += board(q, false) - back(q);
            }
            let ml = board(p, true) - back(p);
            (ol.max(0.0), ml.max(0.0), all.len())
        }
        _ => {
            let (ol, ml, _) = crate::ai::decks::wipe_eval(g, p, kind);
            (ol, ml, n_opp + n_own)
        }
    };
    gain += dv * deaths as f64;
    if let Some((m, times, _)) = seph
        && deaths > 0
        && kind != "parity"
    {
        // the wipe kills him too: unless the others' deaths reach the fourth drain first, that is the flip lost
        let flips = times + (deaths as i64 - 1) >= 4;
        let own_cost = pval(g, m);
        if flips {
            gain += 3.0; // the emblem
        } else {
            loss += own_cost + 6.0;
        }
    }
    if gain <= 0.0 {
        return None;
    }
    let swing = gain - 1.2 * loss;
    let mut u = swing / 2.2 - 2.5 + 2.0 * danger.min(1.5);
    if name(g, c) == "Living Death" && n_opp < 2 {
        u -= 2.0;
    }
    Some(u)
}

/// ais.rean_targets for the deck: the best creature card in any graveyard (our own included)
pub fn rean_targets(g: &Game, p: PlayerId, kind: &str) -> Vec<(f64, C, PlayerId)> {
    let _ = kind;
    steal_targets(g, p)
}

/// cmc, for tests of the plan
pub fn cmc(g: &Game, m: PermId) -> u32 {
    cmc_of(g, m)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    let _ = Tag::Wipe;
    Ok(())
}

/// the sweepers p could cast this turn after Rabid Attack ({1}{B}): the cheapest cost among wipes in hand, plus
/// Pestilence on the battlefield (it needs {B} per activation)
fn follow_up_cost(g: &Game, p: PlayerId) -> Option<u32> {
    let mut best: Option<u32> = None;
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        if d.tags.str(Tag::Wipe).is_some() || EDICTS.contains(&&*d.name) {
            let cost = d.generic + d.pips.len() as u32;
            best = Some(best.map_or(cost, |b| b.min(cost)));
        }
    }
    if g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == "Pestilence") {
        best = Some(best.map_or(3, |b| b.min(3)));
    }
    best
}

/// the plays the deck adds: Rabid Attack (+1/+0 and "draw a card when this dies" for our creatures) ahead of a
/// wipe, an edict or Pestilence that kills several of them, with the mana for both
pub fn options(g: &mut Game, p: PlayerId, post: bool) -> crate::flow::Res<Vec<crate::hooks::Opt>> {
    use crate::hooks::Opt;
    let _ = post;
    let Some(&c) = g.player(p).hand.iter().find(|&&c| &*g.db.get(c).name == "Rabid Attack") else { return Ok(vec![]) };
    if g.active != Some(p) || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let d = g.db.get(c);
    let Some(follow) = follow_up_cost(g, p) else { return Ok(vec![]) };
    let mine = all_creatures(g).iter().filter(|&&(q, m)| q == p && !is_seph(g, m)).count();
    if mine < 2 || !crate::engine::mana::can_pay(g, p, d.generic + follow, &d.pips, false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 7.0 + mine as f64 * 0.5,
        label: format!("Rabid Attack ({mine} creatures, ahead of a sweep)"),
        act: Some(crate::hooks::Action::Cast { card: c, zone: None }),
    }])
}
