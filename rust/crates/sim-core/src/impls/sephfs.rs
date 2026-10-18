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
//! - **Tutors** (`tutor_pick`): board-state pick (finisher that kills, sweeper when behind, outlet/Ghostform with
//!   Sephiroth out, mana only when short), else the generic pick. A fixed wish list lost 0.8-1.7 points over 60,000 games.
//! - **Steals**: Zombify (own graveyard), Animate Dead and the prepared Reanimate take the best creature card from any graveyard
//!   (`rean_targets`).
//! - Stats for `--analyze`: `sf_*` counters (flips, wipes, steals by card, sacrifices, cards drawn by Sephiroth).

use super::sephfs_cards::{all_creatures, cmc_of, flipped, is_seph, parity_choice, steal_targets, steal_value};
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
    if altar_loop_need(g, p).contains(&n) {
        return 88; // the last piece of the Altar + Skeleton + Pawn loop
    }
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
        "Presumed Dead" => return 0, // cast through `options` or in response to removal
        "Rewrite Regrets" => {
            return match super::sephfs_cards::regrets_target(g, p) {
                Some(t) if steal_value(g, t) >= 3.0 => 64,
                _ => 0,
            };
        }
        "Zombify" => {
            return if g.player(p).gy.iter().any(|&c| steal_value(g, c) >= 3.0) { 64 } else { 0 };
        }
        "Gray Merchant of Asphodel" => return gray_merchant_prio(g, p),
        "Bolas's Citadel" => {
            // paying life for cards is only a plan with the drains and outlets to refill it
            let engine = seph.is_some() || outlets(g, p) > 0;
            return if engine { base } else { base.min(30) };
        }
        "Call of the Ring" => return 50,
        "Braids's Frightful Return" => return 52,
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
    let mut out: Vec<Opt> = vec![];
    out.extend(super::sephfs_cards::presumed_option(g, p));
    out.extend(exsanguinate_option(g, p));
    out.extend(rabid_option(g, p));
    out.extend(altar_loop_option(g, p));
    Ok(out)
}

/// Exsanguinate with all spare mana: when the drain is lethal (`x_drain_option` scores a kill), or when X is large (4+)
/// and the opposing boards are no threat to us (their total power under a third of our life)
fn exsanguinate_option(g: &Game, p: PlayerId) -> Option<crate::hooks::Opt> {
    if g.active != Some(p) {
        return None;
    }
    let c = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == "Exsanguinate")?;
    if !crate::engine::cast::castable(g, p, c, "hand") {
        return None;
    }
    let (u, label) = super::yshtola::x_drain_option(g, p, c)?;
    let threat: i32 = g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| g.perm(m).on_bf && g.is_creature(m)).map(|m| crate::engine::values::epow(g, m)).sum();
    if u < 7.0 && threat * 3 >= g.player(p).life {
        return None;
    }
    Some(crate::hooks::Opt { utility: u, label, act: Some(crate::hooks::Action::Plan { f: super::yshtola::x_drain_go, arg: c.0 as i64 }) })
}

fn rabid_option(g: &mut Game, p: PlayerId) -> Option<crate::hooks::Opt> {
    use crate::hooks::Opt;
    let c = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == "Rabid Attack")?;
    if g.active != Some(p) || !crate::engine::cast::castable(g, p, c, "hand") {
        return None;
    }
    let d = g.db.get(c);
    let follow = follow_up_cost(g, p)?;
    let mine = all_creatures(g).iter().filter(|&&(q, m)| q == p && !is_seph(g, m)).count();
    if mine < 2 || !crate::engine::mana::can_pay(g, p, d.generic + follow, &d.pips, false) {
        return None;
    }
    Some(Opt {
        utility: 7.0 + mine as f64 * 0.5,
        label: format!("Rabid Attack ({mine} creatures, ahead of a sweep)"),
        act: Some(crate::hooks::Action::Cast { card: c, zone: None }),
    })
}

// ------------------------------------------------------------------ Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog
/// the three pieces of the deck's named loop
const LOOP_PIECES: [&str; 3] = ["Phyrexian Altar", "Reassembling Skeleton", "Pawn of Ulamog"];
/// what turns the endless deaths into a win (Sephiroth himself, flipped or not, is the fourth)
const LOOP_DRAINS: [&str; 3] = ["Blood Artist", "Zulaport Cutthroat", "Bastion of Remembrance"];

fn bf_named(g: &Game, p: PlayerId, n: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| &*g.db.get(c).name == n)
    })
}

/// a source of "whenever a creature dies, each opponent loses life" on p's battlefield
fn loop_drain(g: &Game, p: PlayerId) -> bool {
    seph_state(g, p).is_some() || LOOP_DRAINS.iter().any(|n| bf_named(g, p, n).is_some())
}

/// Phyrexian Altar: sacrifice Reassembling Skeleton ({1} of mana), Pawn of Ulamog makes an Eldrazi Spawn, sacrifice
/// that too ({1} more) and return the Skeleton for {1}{B}: the mana pays for itself and every lap is two deaths. The
/// pieces on the battlefield (the Skeleton may wait in the graveyard when {1}{B} is there to return it) plus a drain.
/// The Altar and Pawn are the permanents the loop needs to survive.
pub fn altar_loop(g: &Game, p: PlayerId) -> Option<Vec<PermId>> {
    if super::mine::loops_blocked(g, p) || !loop_drain(g, p) {
        return None;
    }
    let altar = bf_named(g, p, LOOP_PIECES[0])?;
    let pawn = bf_named(g, p, LOOP_PIECES[2])?;
    let skel = match bf_named(g, p, LOOP_PIECES[1]) {
        Some(m) => m,
        None => {
            let in_gy = g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == LOOP_PIECES[1]);
            if !in_gy || !crate::engine::mana::can_pay(g, p, 1, "B", false) {
                return None;
            }
            altar // (the Skeleton is no permanent: the Altar stands in as a key piece)
        }
    };
    Some(vec![altar, skel, pawn])
}

/// the one card that would complete the loop (cast at 88, tutored for): nothing when two or more are missing
pub fn altar_loop_need(g: &Game, p: PlayerId) -> Vec<&'static str> {
    if super::mine::loops_blocked(g, p) {
        return vec![];
    }
    let have = |n: &str| bf_named(g, p, n).is_some();
    let mut missing: Vec<&'static [&'static str]> = vec![];
    for (i, n) in LOOP_PIECES.iter().enumerate() {
        if !have(n) {
            missing.push(std::slice::from_ref(&LOOP_PIECES[i]));
        }
    }
    if !loop_drain(g, p) {
        missing.push(&LOOP_DRAINS);
    }
    if missing.len() == 1 { missing[0].to_vec() } else { vec![] }
}

fn altar_loop_go(g: &mut Game, p: PlayerId, _arg: i64) -> crate::flow::Res<bool> {
    let Some(keys) = altar_loop(g, p) else { return Ok(false) };
    let turns = g.player(p).turns;
    let pl = g.player_mut(p);
    pl.stat("combo_attempt", 1);
    pl.milestone.entry("combo").or_insert(turns);
    pl.loop_turn = Some(turns);
    crate::glog!(g, "  {} goes for the Altar + Skeleton + Pawn loop", g.player(p).name);
    if crate::ai::combo_interrupted(g, p, "sephfs", &keys)? {
        g.player_mut(p).stat("combo_stopped", 1);
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(true);
    }
    crate::ai::win(g, p, "combo", Some(true))?;
    Ok(true)
}

fn altar_loop_option(g: &Game, p: PlayerId) -> Option<crate::hooks::Opt> {
    if g.player(p).loop_turn == Some(g.player(p).turns) || altar_loop(g, p).is_none() {
        return None;
    }
    Some(crate::hooks::Opt {
        utility: 14.0 - 6.0 * crate::ai::brain::removal_risk(g, p),
        label: "Altar + Skeleton + Pawn loop".into(),
        act: Some(crate::hooks::Action::Plan { f: altar_loop_go, arg: 0 }),
    })
}

// ------------------------------------------------------------------ Gray Merchant
/// devotion to black: the {B} in the mana costs of p's permanents, plus Gray Merchant's own two
fn devotion(g: &Game, p: PlayerId) -> i32 {
    2 + g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf)
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| g.db.get(c).pips.matches('B').count() as i32)
        .sum::<i32>()
}

/// Gray Merchant of Asphodel: each opponent loses its devotion in life, you gain the total. Lethal on one opponent: cast
/// it at once; otherwise it climbs with the drain it will make
fn gray_merchant_prio(g: &Game, p: PlayerId) -> i32 {
    let dev = devotion(g, p);
    if g.opps(p).any(|q| g.player(q).life <= dev) {
        return 90;
    }
    (38 + 4 * dev).min(84)
}


// ------------------------------------------------------------------ tutors (Demonic Tutor, Vampiric Tutor, Rune-Scarred Demon)
/// the tutor's pick from board state, or None for the generic pick (best priority early, biggest body late):
/// the finisher when it kills; a sweeper when the opponents are ahead on board; an outlet or Kaya's Ghostform when
/// Sephiroth is out with fodder; mana only when short of it
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str, okn: &[CardId]) -> Option<CardId> {
    if kind != "any" {
        return None;
    }
    let first = |ns: &[&str]| ns.iter().filter_map(|n| g.db.id(n)).find(|c| okn.contains(c));
    let pl = g.player(p);
    let lands = pl.lands.len() as i32;
    let held = |n: &str| {
        pl.hand.iter().any(|&c| name(g, c) == n)
            || pl.perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| name(g, c) == n))
    };
    // 1. the finisher that kills: X = lands - 2 for Exsanguinate, the devotion for Gray Merchant
    let low = g.opps(p).map(|q| g.player(q).life).min().unwrap_or(40);
    if low <= lands - 2 && let Some(c) = first(&["Exsanguinate"]) {
        return Some(c);
    }
    if low <= devotion(g, p) && let Some(c) = first(&["Gray Merchant of Asphodel"]) {
        return Some(c);
    }
    // 2. the opponents are ahead on board
    let theirs: f64 = all_creatures(g).iter().filter(|&&(q, _)| q != p).map(|&(_, m)| pval(g, m)).sum();
    let ours: f64 = all_creatures(g).iter().filter(|&&(q, _)| q == p).map(|&(_, m)| pval(g, m)).sum();
    if opp_creatures(g, p) >= 3 && theirs > 1.2 * ours + 3.0 && lands >= 4 {
        if let Some(c) = first(&["Massacre Girl", "Pestilence", "Crux of Fate"]) {
            return Some(c);
        }
    }
    // 3. Sephiroth out with fodder
    if let Some((_, _, fl)) = seph_state(g, p) {
        if own_fodder(g, p) >= 1 {
            if outlets(g, p) == 0
                && !pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Sac))
                && let Some(c) = first(&["Phyrexian Altar", "Viscera Seer"])
            {
                return Some(c);
            }
            if !fl && !held("Kaya's Ghostform") && let Some(c) = first(&["Kaya's Ghostform"]) {
                return Some(c);
            }
        }
    }
    // 4. short of mana
    if lands < 3 && !pl.hand.iter().any(|&c| g.db.get(c).land) {
        if lands >= 1 && let Some(c) = first(&["Sol Ring"]) {
            return Some(c);
        }
        return okn.iter().copied().find(|&c| g.db.get(c).land && name(g, c) == "Swamp");
    }
    None
}
