//! Tier 6 table rules: how a cEDH pod plays around the player who is about to win. Every entry point is gated to
//! Tier 6 players by the caller (`impls::t6_plans::is_t6`); Tier 0-5 games never reach this file.
//!
//! - archenemy: a 0-10 score per opponent for "about to win" (`archenemy_score`), used to pick counter and removal
//!   targets, whom to attack, and whether to keep mana up.
//! - spend answers on the win: `counter_verdict` (a combo step or Ad Nauseam is always countered when possible, free
//!   counters included; a lesser spell is not, while the archenemy is still to come).
//! - go when it can't be stopped: `go_ok` counts what the table can answer against what the line needs.
//! - protection first: `protect_before_go` casts Silence / Orim's Chant before the line starts.
//! - Angel's Grace: `grace_save` answers a lethal state when the card and {W} are there.

use crate::ai::brain::{open_mana, prob_holding};
use crate::cards::Types;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::free_counter;
use crate::engine::values::commander_out;
use crate::flow::Res;
use crate::ids::{CardId, PlayerId};
use crate::impls::{combos, t6_plans};
use crate::state::{Ctx, Game, StackItem};
use crate::tag::Tag;

/// an opponent at or above this score is about to win: hold up answers for it, spend them on its plays
pub const NEAR: f64 = 5.0;

/// what an AI at Tier 6 does with a counterspell
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verdict {
    /// counter it if any counter can be cast (a win attempt)
    Counter,
    /// hold the counter
    Pass,
    /// the usual roll, with this value and threshold
    Roll(f64, f64),
}

fn is_free_ctr(g: &Game, q: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if &*d.name == "Deflecting Swat" {
        return commander_out(g, q);
    }
    d.tag(Tag::Ctr)
        && (d.tag(Tag::Free) || d.tag(Tag::Pact) || d.tag(Tag::Misstep) || d.tag(Tag::Fon) || free_counter(g, q, c))
}

/// P(q can stop one spell or ability of a player going for a win right now): free counters whatever its mana, counters
/// paid from the blue it has open, and instant-speed removal for a permanent step
pub fn stop_chance(g: &Game, q: PlayerId) -> f64 {
    if g.player(q).hand.is_empty() {
        return 0.0;
    }
    let free = prob_holding(g, q, |c| is_free_ctr(g, q, c));
    let up = open_mana(g, q, Some('U'));
    let paid = prob_holding(g, q, |c| g.db.get(c).tag(Tag::Ctr))
        * if up >= 2 {
            0.9
        } else if up >= 1 {
            0.45
        } else {
            0.0
        };
    let rem = prob_holding(g, q, |c| {
        let d = g.db.get(c);
        d.instant && d.tag(Tag::Rem)
    }) * if open_mana(g, q, None) >= 1 { 0.5 } else { 0.0 };
    1.0 - (1.0 - free) * (1.0 - paid) * (1.0 - rem)
}

/// 0-10: how close q is to winning, as `viewer` can read it. Pieces of its best line on the battlefield (public), a
/// ready line, mana open with cards in hand, tutors and Ad Nauseam it may hold, and board pieces that are part of a
/// combo already.
pub fn archenemy_score(g: &Game, q: PlayerId) -> f64 {
    let pl = g.player(q);
    if !pl.alive {
        return 0.0;
    }
    let (frac, ready) = combos::line_progress(g, q);
    let mut s = 5.0 * frac * frac;
    if ready {
        s += 4.0;
    }
    let up = open_mana(g, q, None) as f64;
    if pl.hand.len() >= 2 {
        s += (up / 8.0).min(1.0);
    }
    let tut = prob_holding(g, q, |c| g.db.get(c).tag(Tag::Tut) || g.db.get(c).tag(Tag::Seal));
    s += 1.5 * tut * (up / 4.0).min(1.0);
    if prob_holding(g, q, |c| g.db.get(c).tag(Tag::Adnaus)) > 0.0 && up >= 4.0 {
        s += 1.5;
    }
    s.min(10.0)
}

/// the opponent of `viewer` closest to winning, with its score; None below 3
pub fn archenemy(g: &Game, viewer: PlayerId) -> Option<(PlayerId, f64)> {
    let mut best: Option<(PlayerId, f64)> = None;
    for q in g.opps(viewer) {
        let s = archenemy_score(g, q);
        if best.is_none_or(|b| s > b.1) {
            best = Some((q, s));
        }
    }
    best.filter(|b| b.1 >= 3.0)
}

/// the threat bonus the archenemy adds to a Tier 6 player's reading of `q`
pub fn threat_bonus(g: &Game, q: PlayerId) -> f64 {
    let s = archenemy_score(g, q);
    if s >= 3.0 { 1.2 * s } else { 0.0 }
}

/// What q does with its counterspell(s) against `top` (a spell on the stack that q may counter). `val`/`thr`/`nc` are
/// the generic inputs (value to q, threshold, counters in hand).
pub fn counter_verdict(g: &Game, q: PlayerId, top: &StackItem, val: f64, thr: f64, nc: usize) -> Verdict {
    let ctrl = top.controller;
    let adnaus = top.card.is_some_and(|c| g.db.get(c).tag(Tag::Adnaus));
    if g.t6_going == Some(ctrl) || adnaus {
        return Verdict::Counter; // the spell is (a step of) the win: every answer, free ones too
    }
    // a piece of one of the caster's lines (quiet Tier 6 lines too: Godo, Kinnan, Dack): the closer it takes the line,
    // the more it is worth stopping; the last piece is the win. Only permanents: an instant or sorcery piece matters as
    // a step of the win attempt itself (`t6_going` above)
    let piece = top.card.filter(|&c| g.db.get(c).perm).map_or(0.0, |c| combos::piece_gain(g, ctrl, &g.db.get(c).name));
    if piece >= 1.0 {
        return Verdict::Counter;
    }
    let val = if piece > 0.0 { val + 2.0 + 4.0 * piece } else { val };
    let arch = archenemy(g, q);
    let held = |extra: f64| thr + extra + if nc <= 1 { 1.0 } else { 0.0 };
    match arch {
        Some((a, s)) if a == ctrl && s >= NEAR => {
            if s >= 8.0 && val >= 3.0 {
                Verdict::Counter
            } else {
                Verdict::Roll(val + 0.3 * s, thr - 1.0)
            }
        }
        Some((_, s)) if s >= NEAR => {
            // somebody else is about to win: keep the counters for them unless this spell is a real threat
            if val >= 7.0 { Verdict::Roll(val, thr) } else { Verdict::Pass }
        }
        _ => {
            if val >= 7.0 {
                Verdict::Roll(val, thr)
            } else {
                Verdict::Roll(val, held(0.5)) // not at the first spell: the win attempt is still to come
            }
        }
    }
}

/// is `c` a protection spell we use before going for a win, and for whom
fn protection(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| matches!(&*g.db.get(c).name, "Silence" | "Orim's Chant")).collect()
}

/// P(the table stops the line `cmb` right now), with p's own protection counted
pub fn go_risk(g: &Game, p: PlayerId) -> f64 {
    let mut miss = 1.0;
    for q in g.opps(p) {
        miss *= 1.0 - stop_chance(g, q);
    }
    let mut risk = 1.0 - miss;
    let prot = protection(g, p);
    if !prot.is_empty() && can_pay(g, p, 0, "W", false) {
        risk *= 0.2; // Silence / Chant first
    }
    let mine_free = g.player(p).hand.iter().filter(|&&c| is_free_ctr(g, p, c)).count().min(2);
    risk * 0.75f64.powi(mine_free as i32)
}

/// Should p go for combo `cmb` (index into `combos::all()`) now? The plan slot decides first.
pub fn go_ok(g: &Game, p: PlayerId, combo: &str) -> bool {
    if let Some(v) = t6_plans::go_now(g, p, combo) {
        return v;
    }
    let risk = go_risk(g, p);
    let other = g.opps(p).map(|q| archenemy_score(g, q)).fold(0.0, f64::max);
    // waiting costs turns, and a cEDH pod does not wait long: the bar drops as the game goes on
    let mut thr: f64 = if other >= 6.0 {
        0.85 // somebody else is about to win: waiting is worse
    } else if other >= NEAR {
        0.75
    } else {
        0.6
    };
    if g.round >= 4 {
        thr += 0.15;
    }
    if g.round >= 6 {
        thr = 1.0;
    }
    risk <= thr
}

/// Cast Silence (two or more opponents could answer) or Orim's Chant (one) before the line starts, when the mana for
/// both is there. Returns true if a protection spell was cast.
pub fn protect_before_go(g: &mut Game, p: PlayerId, need: (u32, String)) -> Res<bool> {
    let stops: Vec<(PlayerId, f64)> = g.opps(p).map(|q| (q, stop_chance(g, q))).filter(|x| x.1 >= 0.15).collect();
    if stops.is_empty() {
        return Ok(false);
    }
    for c in protection(g, p) {
        let d = g.db.get(c);
        let silence = &*d.name == "Silence";
        if silence != (stops.len() >= 2) {
            continue;
        }
        if !can_pay(g, p, need.0 + d.generic, &format!("{}{}", need.1, d.pips), false) {
            continue;
        }
        let ctx = if silence { Ctx::default() } else { Ctx { face: Some(stops[0].0), ..Ctx::default() } };
        crate::glog!(g, "  {} protects the win with {}", g.player(p).name, d.name);
        if crate::ai::brain::do_cast_with(g, p, c, None, ctx)? {
            g.player_mut(p).stat("t6_protect", 1);
            return Ok(true);
        }
    }
    Ok(false)
}

/// p is about to lose (life, commander damage or poison): cast Angel's Grace from hand if {W} is open. Split second, so
/// nothing answers it; it is paid and resolved here without a stack window. True if p is saved this turn.
pub fn grace_save(g: &mut Game, p: PlayerId) -> bool {
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == "Angel's Grace") else {
        return false;
    };
    if !can_pay(g, p, 0, "W", false) || pay(g, p, 0, "W", false).is_err() {
        return false;
    }
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.gy.push(c);
    pl.flag_turn.insert(crate::sym::intern("stax_grace"), st);
    if pl.life < 1 {
        pl.life = 1; // damage can't take you below 1
    }
    pl.stat("t6_grace", 1);
    crate::glog!(g, "    {} casts Angel's Grace: it can't lose this turn", g.player(p).name);
    true
}

/// does the card count as a land or a mana source for the "hold up" reserve (never held back for)
pub fn is_ramp(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.land || d.tag(Tag::Rock) || d.tag(Tag::Fastmana) || d.types.has(Types::LAND)
}

/// A plan's priority at or above this on a cast is a "do it now" play (see `must_option`).
pub const MUST_PRIO: f64 = 90.0;

/// Tier 6 "must-do" plays, taken before the look-ahead (which kept voting them down: `Sisay: Mount Doom 100%` lost to
/// "stop (hold mana)", Chord over Kinnan, Dack scored and not cast). Returns the index in `opts` of the play to make:
/// the highest-priority cast whose plan priority (`t6_plans::prio`) is at least `MUST_PRIO` (commanders, fast mana,
/// the protection pre-cast), a card that would complete one of the player's lines (`combos::piece_gain` >= 1.0), or
/// Sisay's fetch. None while the plan's `hold_up` slot asks to pass with the mana up.
pub fn must_option(g: &Game, p: PlayerId, opts: &[crate::hooks::Opt]) -> Option<usize> {
    use crate::hooks::Action;
    if t6_plans::hold_up(g, p).is_some_and(|x| x > 0.0) {
        return None;
    }
    let mut best: Option<(f64, f64, usize)> = None;
    for (i, o) in opts.iter().enumerate() {
        let score = match &o.act {
            Some(Action::Cast { card, zone: None }) => {
                let pr = t6_plans::prio(g, p, *card);
                if pr.is_some_and(|x| x >= MUST_PRIO) {
                    pr.unwrap_or(0.0)
                } else if pr.is_none_or(|x| x > 0.0) && !g.db.get(*card).land && combos::piece_gain(g, p, &g.db.get(*card).name) >= 1.0 {
                    89.0
                } else {
                    continue;
                }
            }
            Some(Action::Ability { .. }) if g.player(p).key == "sisay-legends-toolbox" && o.label.starts_with("Sisay: ") => 91.0,
            _ => continue,
        };
        if best.is_none_or(|b| (score, o.utility) > (b.0, b.1)) {
            best = Some((score, o.utility, i));
        }
    }
    best.map(|b| b.2)
}
