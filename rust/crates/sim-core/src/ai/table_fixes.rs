//! Three generic AI fixes found in the pool rebalance's traces (2026-11-02). Each has its own switch, read like
//! `AI_SEARCH_B` / `AI_RULES` / `AI_T6STOPS`: unset, empty or anything but `0` is on (the default); `=0` is off. All
//! three off: today's play, byte for byte.
//!
//! - `AI_SACSTEAL`: a sacrifice (an edict, a sacrifice outlet's cost, Torment of Hailfire's choice, Korvold's trigger)
//!   counts what the sacrificed nontoken permanent is worth to an opponent whose Tergrid, God of Fright takes it.
//!   Tokens are never taken, so they stay first in line.
//! - `AI_CMDRISK`: recasting a commander from the command zone weighs the chance an opponent removes it again
//!   (`cmd_recast_penalty`), unless the table is about to kill us or the commander is the only play.
//! - `AI_WIPEVALUE`: a board wipe is cast only when the caster is better off than the average opponent, or the
//!   table's leader loses much more than the caster, or the caster is in danger (`wipe_blocked`).

use std::cell::Cell;
use std::sync::OnceLock;

use crate::engine::values::{has, pval};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

/// a switch's value from its environment variable: `0` off, anything else on
pub fn parse(v: Option<&str>) -> bool {
    !matches!(v.map(str::trim), Some("0"))
}

macro_rules! switch {
    ($force:ident, $on:ident, $cell:ident, $env:literal, $doc:literal) => {
        thread_local! {
            static $cell: Cell<Option<bool>> = const { Cell::new(None) };
        }
        #[doc = concat!("for tests: ", $doc, " on or off on this thread, whatever `", $env, "` says (None: back to it)")]
        pub fn $force(on: Option<bool>) {
            $cell.with(|f| f.set(on));
        }
        #[doc = concat!("is ", $doc, " on? (`", $env, "`)")]
        pub fn $on() -> bool {
            if let Some(f) = $cell.with(|f| f.get()) {
                return f;
            }
            static M: OnceLock<bool> = OnceLock::new();
            *M.get_or_init(|| parse(std::env::var($env).ok().as_deref()))
        }
    };
}

switch!(force_sacsteal, sacsteal_on, FORCE_SACSTEAL, "AI_SACSTEAL", "the steal-on-sacrifice cost");
switch!(force_cmdrisk, cmdrisk_on, FORCE_CMDRISK, "AI_CMDRISK", "the commander recast risk");
switch!(force_wipevalue, wipevalue_on, FORCE_WIPEVALUE, "AI_WIPEVALUE", "the relative wipe value");

// ------------------------------------------------------------------ fix 1: sacrifice and Tergrid
/// does an opponent of p control a permanent that takes p's sacrificed nontoken permanents (Tergrid, God of Fright)?
pub fn steal_on_sacrifice(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| g.player(q).alive && has(g, q, Tag::Tergrid))
}

/// what it costs p, beyond losing it, to sacrifice permanent m with such an opponent out: what m is worth to them (a
/// land about one mana, anything else its value, at least half a point). Nothing for a token, which is not taken, and
/// nothing with the switch off or no such opponent.
pub fn steal_cost(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if !sacsteal_on() || !steal_on_sacrifice(g, p) {
        return 0.0;
    }
    let x = g.perm(m);
    let Some(c) = x.cd else { return 0.0 };
    if x.token {
        return 0.0;
    }
    if g.db.get(c).land { 1.0 } else { pval(g, m).max(0.5) }
}

/// the same for a permanent card p would discard from hand (Tergrid takes discarded permanent cards too); `worth` is
/// what p rates the card
pub fn steal_cost_card(g: &Game, p: PlayerId, c: CardId, worth: f64) -> f64 {
    if !sacsteal_on() || !steal_on_sacrifice(g, p) {
        return 0.0;
    }
    let d = g.db.get(c);
    if d.land {
        1.0
    } else if d.perm {
        worth.max(0.5)
    } else {
        0.0
    }
}

// ------------------------------------------------------------------ fix 2: recasting a commander
/// per earlier cast from the command zone, how much utility a fully likely removal takes off a recast
const RECAST_STEP: f64 = 1.0;
/// the least removal risk a player assumes against opponents who hold cards (the public lists undercount: the pool's
/// hands hold the removal their decks are built around)
const RISK_FLOOR: f64 = 0.2;
/// the most a recast loses to the removal risk
const RECAST_CAP: f64 = 2.8;

/// P(an opponent removes a commander I cast now before it does anything): removal held by an opponent with mana up
/// (instant speed, as `brain::removal_risk`), or any removal or wipe in a hand on their turn at half weight. From the
/// public decklists only.
pub fn cmd_removal_risk(g: &Game, p: PlayerId) -> f64 {
    let mut miss = 1.0;
    for q in g.opps(p) {
        let any = crate::ai::brain::prob_holding(g, q, |c| {
            let d = g.db.get(c);
            d.tag(Tag::Rem) || d.tag(Tag::Wipe)
        });
        if any == 0.0 {
            continue;
        }
        let inst = crate::ai::brain::prob_holding(g, q, |c| {
            let d = g.db.get(c);
            d.instant && d.tag(Tag::Rem)
        });
        let up = crate::ai::brain::open_mana(g, q, None) >= 2;
        let hit = f64::max(0.5 * any, if up { 0.8 * inst } else { 0.0 });
        miss *= 1.0 - hit;
    }
    let risk = 1.0 - miss;
    if g.opps(p).any(|q| !g.player(q).hand.is_empty()) { risk.max(RISK_FLOOR) } else { risk }
}

/// the utility a recast of commander `c` loses to removal risk: 0 for a first cast (tax 0), when `danger` (damage
/// pointed at me against my life) is 0.8 or more, when the commander has haste and can finish an opponent, or with
/// nothing else in hand to cast instead (`other_play`)
pub fn cmd_recast_penalty(g: &Game, p: PlayerId, c: CardId, danger: f64, other_play: bool) -> f64 {
    if !cmdrisk_on() || !other_play || danger >= 0.8 {
        return 0.0;
    }
    let casts = g.player(p).tax_of(c) / 2;
    if casts == 0 {
        return 0.0;
    }
    let d = g.db.get(c);
    if d.tag(Tag::Haste) && g.opps(p).any(|q| g.player(q).life <= d.pow) {
        return 0.0; // it wins now
    }
    (RECAST_STEP * casts as f64 * cmd_removal_risk(g, p)).min(RECAST_CAP)
}

// ------------------------------------------------------------------ fix 3: wipes that cost the caster as much
/// a wipe whose caster loses `mine` while the opponents lose `theirs` (one entry each, in the same units) is held:
/// the caster would lose at least as much as the average opponent, and no opponent loses much more than the caster
/// (a leader to take down), and the caster is not in danger. A wipe that costs the caster nothing is never held.
pub fn wipe_blocked(mine: f64, theirs: &[f64], danger: f64) -> bool {
    if !wipevalue_on() || mine <= 0.0 || theirs.is_empty() || danger >= 1.0 {
        return false;
    }
    let mean = theirs.iter().sum::<f64>() / theirs.len() as f64;
    let leader = theirs.iter().copied().fold(0.0, f64::max);
    mean <= mine && leader < 1.5 * mine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switches_read_like_the_others() {
        assert!(parse(None));
        assert!(parse(Some("")));
        assert!(parse(Some("1")));
        assert!(!parse(Some("0")));
        assert!(!parse(Some(" 0 ")));
    }

    #[test]
    fn a_symmetric_wipe_is_blocked_and_a_lopsided_one_is_not() {
        force_wipevalue(Some(true));
        assert!(wipe_blocked(2.0, &[1.0, 2.0, 2.0], 0.0)); // Brago's Farewell, t2-2 round 12
        assert!(!wipe_blocked(2.0, &[1.0, 6.0, 2.0], 0.0)); // a leader loses three times what we do
        assert!(!wipe_blocked(2.0, &[3.0, 3.0, 3.0], 0.0)); // each opponent loses more than we do
        assert!(!wipe_blocked(0.0, &[0.0, 0.0], 0.0)); // costs us nothing
        assert!(!wipe_blocked(2.0, &[1.0, 2.0, 2.0], 1.2)); // we are about to lose
        force_wipevalue(Some(false));
        assert!(!wipe_blocked(2.0, &[1.0, 2.0, 2.0], 0.0));
        force_wipevalue(None);
    }
}
