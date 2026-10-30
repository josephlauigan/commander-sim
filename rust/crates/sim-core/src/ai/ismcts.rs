//! --- phase4-search: information-set Monte Carlo tree search on the high-stakes decisions of the generic-policy
//! decks (`audit/phase4/search-design.md`).
//!
//! - `stakes`: what is at stake in a decision (the facts the stakes test reads, and the diagnostic prints).
//! - `AI_SEARCH_DIAG=1`: one line on stderr per look-ahead decision of a generic-policy deck (`SRD ...`), with the
//!   stakes facts and whether the look-ahead's pick differs from the policy's. Stats only: play is unchanged.

use crate::engine::values::epow;
use crate::hooks::{Action, Opt};
use crate::ids::PlayerId;
use crate::state::Game;
use std::sync::OnceLock;

// ------------------------------------------------------------------ what is at stake
/// the facts of a decision the stakes test reads
#[derive(Debug, Clone, Copy, Default)]
pub struct Stakes {
    /// real options (not "stop")
    pub nreal: usize,
    /// the policy's top real option's utility less the second's (infinite with one)
    pub gap: f64,
    /// the policy's top option is "stop"
    pub top_stop: bool,
    /// a board wipe or a removal spell among the candidates
    pub wipe: bool,
    pub removal: bool,
    /// combo distance (pieces missing, `planner::distance`, capped at 6): the decider's, the nearest opponent's
    pub my_dist: usize,
    pub opp_dist: usize,
    /// the decider's creatures' power against the lowest opponent life: at or above it (`lethal`), and the
    /// table's power against the decider's life (`danger`, `brain::Situation`)
    pub lethal: bool,
    pub danger: f64,
    /// opponents' untapped creatures (blockers) and the decider's creatures able to attack
    pub blockers: usize,
    pub attackers: usize,
    pub round: u32,
}

/// the stakes of p's decision among `opts` (empty for an attack or a counter)
pub fn stakes(g: &Game, p: PlayerId, opts: &[Opt]) -> Stakes {
    let mut u: Vec<f64> = opts.iter().filter(|o| o.act.is_some()).map(|o| o.utility).collect();
    u.sort_by(|a, b| b.total_cmp(a));
    let top = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    let wipe = opts.iter().any(|o| matches!(o.act, Some(Action::Wipe { .. })));
    let removal = opts.iter().any(|o| matches!(o.act, Some(Action::Removal { .. })));
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let dist = |q: PlayerId| super::planner::distance(g, q).min(6);
    let creatures = |q: PlayerId| g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m)).collect::<Vec<_>>();
    let mine = creatures(p);
    let power: i32 = mine.iter().map(|&m| epow(g, m).max(0)).sum();
    let min_life = opps.iter().map(|&q| g.player(q).life).min().unwrap_or(i32::MAX);
    Stakes {
        nreal: u.len(),
        gap: if u.len() >= 2 { u[0] - u[1] } else { f64::INFINITY },
        top_stop: !opts.is_empty() && opts[top].act.is_none(),
        wipe,
        removal,
        my_dist: dist(p),
        opp_dist: opps.iter().map(|&q| dist(q)).min().unwrap_or(6),
        lethal: power >= min_life,
        danger: super::brain::Situation::new(g, p).danger,
        blockers: opps.iter().map(|&q| creatures(q).iter().filter(|&&m| !g.perm(m).tapped).count()).sum(),
        attackers: mine.iter().filter(|&&m| !g.perm(m).tapped && !g.perm(m).sick).count(),
        round: g.round,
    }
}

// ------------------------------------------------------------------ the diagnostic
/// `AI_SEARCH_DIAG=1`: the per-decision diagnostic is on
pub fn diag_on() -> bool {
    static D: OnceLock<bool> = OnceLock::new();
    *D.get_or_init(|| matches!(std::env::var("AI_SEARCH_DIAG").ok().as_deref(), Some("1")))
}

/// one diagnostic line: `SRD kind changed steps nreal gap top_stop wipe removal my_dist opp_dist lethal danger
/// blockers attackers round key`
pub fn diag(g: &Game, p: PlayerId, kind: &str, s: &Stakes, changed: bool, steps: u64) {
    eprintln!(
        "SRD {kind} {} {steps} {} {:.2} {} {} {} {} {} {} {:.2} {} {} {} {}",
        changed as u8,
        s.nreal,
        if s.gap.is_finite() { s.gap } else { 99.0 },
        s.top_stop as u8,
        s.wipe as u8,
        s.removal as u8,
        s.my_dist,
        s.opp_dist,
        s.lethal as u8,
        s.danger,
        s.blockers,
        s.attackers,
        s.round,
        g.player(p).key
    );
}
