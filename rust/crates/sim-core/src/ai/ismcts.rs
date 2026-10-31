//! --- phase4-search: information-set Monte Carlo tree search on the high-stakes decisions of the generic-policy
//! decks (`audit/phase4/search-design.md`).
//!
//! - `stakes`: what is at stake in a decision (the facts the stakes test reads, and the diagnostic prints).
//! - `AI_SEARCH_DIAG=1`: one line on stderr per look-ahead decision of a generic-policy deck (`SRD ...`), with the
//!   stakes facts and whether the look-ahead's pick differs from the policy's. Stats only: play is unchanged.

use crate::engine::values::epow;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Game, SearchCost};
use crate::sym::Sym;
use std::cell::RefCell;
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

// ------------------------------------------------------------------ the switch and the knobs
enum Mode {
    Off,
    All,
    Keys(Vec<String>),
}

fn mode() -> &'static Mode {
    static M: OnceLock<Mode> = OnceLock::new();
    M.get_or_init(|| parse_mode(std::env::var("AI_SEARCH").ok().as_deref()))
}

/// `AI_SEARCH`: unset or `1` ISMCTS for every generic-policy deck, `0` today's look-ahead, `a,b` ISMCTS for those
/// keys. Stage C: ISMCTS is the default (the paired tests were level on t1-t5 in both halves at 5-24% fewer playout
/// steps a game, `audit/phase4/search-handoff.md`); `0` keeps today's for a measurement arm. JD's decks and Tier 6
/// always keep today's (`values::generic_deck`).
fn parse_mode(v: Option<&str>) -> Mode {
    match v.map(str::trim) {
        Some("0") => Mode::Off,
        None | Some("") | Some("1") => Mode::All,
        Some(list) => Mode::Keys(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()),
    }
}

/// does p decide its look-ahead decisions by ISMCTS?
pub fn on(g: &Game, p: PlayerId) -> bool {
    if let Some(f) = FORCE.with(|f| f.get()) {
        return f && super::values::generic_deck(g, p);
    }
    match mode() {
        Mode::Off => false,
        Mode::All => super::values::generic_deck(g, p),
        Mode::Keys(k) => super::values::generic_deck(g, p) && k.iter().any(|x| x == g.player(p).key),
    }
}

/// the search's knobs (env `AI_SEARCH_*`, for A/B runs; the defaults are the design's)
#[derive(Debug, Clone, Copy)]
pub struct Knobs {
    /// playout steps a decision may spend (`AI_SEARCH_BUDGET`)
    pub budget: u64,
    /// the UCB constant, in `evaluate` points (`AI_SEARCH_C`)
    pub c: f64,
    /// tree levels: the root's candidates are level 1, the decider's next choices in a playout level 2 ...
    /// (`AI_SEARCH_DEPTH`)
    pub depth: usize,
    /// progressive widening below the root: a node visited v times may try ceil(widen * sqrt(v)) of its candidates,
    /// best utility first (`AI_SEARCH_WIDEN`)
    pub widen: f64,
    /// playouts every root candidate gets before UCB (on shared seeds), and the most per candidate
    pub min_visits: u32,
    pub max_visits: u32,
    /// the stakes test (`AI_SEARCH_GATE=0`: every decision is searched)
    pub gate: bool,
    /// the stakes test's close call: the policy's top two real options within this much utility. Default: no
    /// limit, so every decision with two or more real options is searched (t1 half B, 2000 games: a limit of 0.5
    /// cost 2 points, -3.7 [-7.3, -0.1], against -1.8 [-5.5, +1.9] with none, at the same steps)
    pub close: f64,
    /// the fewest shared copies on which the search may overrule the policy's top play, or call a leader clear
    /// (`AI_SEARCH_PAIRS`; today's margin has 6 at the least)
    pub pairs: usize,
    /// --- phase4-search-b (an ablation knob): the playouts every attack plan and counter choice gets before UCB
    /// (`AI_SEARCH_ATK_MIN`; default `min_visits`)
    pub atk_min: u32,
}

fn env_or<T: std::str::FromStr>(key: &str, d: T) -> T {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| v.parse().unwrap_or_else(|_| panic!("bad {key} {v:?}")))
        .unwrap_or(d)
}

pub const BUDGET: u64 = 6_000;
pub const C: f64 = 15.0;
pub const DEPTH: usize = 3;
pub const WIDEN: f64 = 0.5;
pub const MIN_VISITS: u32 = 2;
pub const MAX_VISITS: u32 = 24;
pub const CLOSE: f64 = f64::INFINITY;
pub const PAIRS: usize = 6;

fn env_knobs() -> &'static Knobs {
    static K: OnceLock<Knobs> = OnceLock::new();
    K.get_or_init(|| Knobs {
        budget: env_or("AI_SEARCH_BUDGET", BUDGET),
        c: env_or("AI_SEARCH_C", C),
        depth: env_or("AI_SEARCH_DEPTH", DEPTH),
        widen: env_or("AI_SEARCH_WIDEN", WIDEN),
        min_visits: env_or("AI_SEARCH_MIN", MIN_VISITS),
        max_visits: env_or("AI_SEARCH_MAX", MAX_VISITS),
        gate: !matches!(std::env::var("AI_SEARCH_GATE").ok().as_deref(), Some("0")),
        close: env_or("AI_SEARCH_CLOSE", CLOSE),
        pairs: env_or("AI_SEARCH_PAIRS", PAIRS),
        atk_min: env_or("AI_SEARCH_ATK_MIN", env_or("AI_SEARCH_MIN", MIN_VISITS)),
    })
}

thread_local! {
    /// a test's overrides on this thread: ISMCTS on or off for every generic-policy deck, and the knobs
    static FORCE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static KNOBS: std::cell::Cell<Option<Knobs>> = const { std::cell::Cell::new(None) };
}

/// for tests: ISMCTS on (`Some(true)`) or off for every generic-policy deck on this thread, whatever `AI_SEARCH`
/// says; None: back to `AI_SEARCH`
pub fn force(on: Option<bool>) {
    FORCE.with(|f| f.set(on));
}

/// for tests: these knobs on this thread (None: back to the env's)
pub fn set_knobs(k: Option<Knobs>) {
    KNOBS.with(|f| f.set(k));
}

/// the knobs in force
pub fn knobs() -> Knobs {
    KNOBS.with(|k| k.get()).unwrap_or(*env_knobs())
}

// ------------------------------------------------------------------ the stakes test
/// is a main-phase decision with these stakes worth a search? (else the policy's top play is made)
pub fn high_stakes_main(s: &Stakes, k: &Knobs) -> bool {
    !k.gate
        || s.wipe
        || s.removal
        || s.lethal
        || s.danger >= 1.0
        || s.my_dist <= 1
        || s.opp_dist == 0
        || (s.nreal >= 2 && s.gap < k.close)
}

/// is an attack decision with these stakes worth a search? (else the policy's attack is made)
pub fn high_stakes_attack(s: &Stakes, k: &Knobs) -> bool {
    !k.gate || s.lethal || s.danger >= 0.5 || s.blockers > 0
}

// ------------------------------------------------------------------ the tree
/// a node: one of the decider's choices (an option's label and its occurrence), reached by the path above it
struct Node {
    key: (String, usize),
    visits: u32,
    /// the times it was available when its parent chose (ISMCTS's availability count)
    avail: u32,
    sum: f64,
    kids: Vec<usize>,
}

struct Tree {
    seat: PlayerId,
    nodes: Vec<Node>,
    /// this playout's place in the tree (None: off it, the policy plays), its path, its depth, and whether it has
    /// added its one new node
    cur: Option<usize>,
    path: Vec<usize>,
    depth: usize,
    expanded: bool,
    knobs: Knobs,
    top_k: usize,
}

thread_local! {
    static TREE: RefCell<Option<Tree>> = const { RefCell::new(None) };
}

/// the candidates of a decision: the policy's top `top_k` real options and "stop", best utility first, each as
/// (index into opts, (label, occurrence))
pub fn candidates(opts: &[Opt], top_k: usize) -> Vec<(usize, (String, usize))> {
    let mut real: Vec<usize> = (0..opts.len()).filter(|&i| opts[i].act.is_some()).collect();
    real.sort_by(|&a, &b| opts[b].utility.partial_cmp(&opts[a].utility).unwrap_or(std::cmp::Ordering::Equal));
    real.truncate(top_k);
    if let Some(s) = opts.iter().position(|o| o.act.is_none()) {
        real.push(s);
    }
    let key = |i: usize| (opts[i].label.clone(), opts[..i].iter().filter(|x| x.label == opts[i].label).count());
    real.into_iter().map(|i| (i, key(i))).collect()
}

/// inside a playout of an ISMCTS decision: the decider p's next main-phase choice from the tree (UCB over the
/// candidates this copy offers, with progressive widening), or None to let the policy play (off the tree, past its
/// depth, or after this playout's one new node)
pub fn tree_pick(g: &Game, p: PlayerId, opts: &[Opt]) -> Option<usize> {
    if !g.in_search {
        return None;
    }
    TREE.with(|t| {
        let mut t = t.borrow_mut();
        let tr = t.as_mut()?;
        if tr.seat != p {
            return None;
        }
        let cur = tr.cur?;
        if tr.depth >= tr.knobs.depth {
            tr.cur = None;
            return None;
        }
        let mut cands = candidates(opts, tr.top_k);
        cands.sort_by(|a, b| opts[b.0].utility.partial_cmp(&opts[a.0].utility).unwrap_or(std::cmp::Ordering::Equal));
        let allowed = ((tr.knobs.widen * (tr.nodes[cur].visits as f64).sqrt()).ceil() as usize).max(1);
        cands.truncate(allowed);
        let mut have: Vec<(usize, usize)> = vec![];
        let mut missing = None;
        for (i, key) in cands {
            match tr.nodes[cur].kids.iter().copied().find(|&k| tr.nodes[k].key == key) {
                Some(k) => have.push((i, k)),
                None if missing.is_none() => missing = Some((i, key)),
                None => {}
            }
        }
        for &(_, k) in &have {
            tr.nodes[k].avail += 1;
        }
        let (i, k) = if let Some((i, key)) = missing {
            if tr.expanded {
                tr.cur = None;
                return None;
            }
            let k = tr.nodes.len();
            tr.nodes.push(Node { key, visits: 0, avail: 1, sum: 0.0, kids: vec![] });
            tr.nodes[cur].kids.push(k);
            tr.expanded = true;
            (i, k)
        } else {
            let c = tr.knobs.c;
            let ucb = |k: usize| {
                let n = &tr.nodes[k];
                let v = n.visits.max(1) as f64;
                n.sum / v + c * ((n.avail.max(1) as f64).ln() / v).sqrt()
            };
            *have.iter().reduce(|a, b| if ucb(b.1) > ucb(a.1) { b } else { a })?
        };
        tr.path.push(k);
        tr.cur = Some(k);
        tr.depth += 1;
        Some(i)
    })
}

// ------------------------------------------------------------------ the root
/// per-candidate playout values (`vals[k][j]`: candidate k's j-th playout, on copy j of the decision: the same
/// determinization for every candidate, so they compare paired), and the steps they took
pub struct Searched {
    pub vals: Vec<Vec<f64>>,
    pub steps: u64,
}

/// the capped mean of a candidate's playouts
fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NEG_INFINITY } else { v.iter().map(|x| x.min(100.0)).sum::<f64>() / v.len() as f64 }
}

/// per shared copy, a's value less b's (each capped at a win)
fn paired(vals: &[Vec<f64>], a: usize, b: usize) -> Vec<f64> {
    vals[a].iter().zip(&vals[b]).map(|(x, y)| x.min(100.0) - y.min(100.0)).collect()
}

/// the leader by capped mean (ties to the earlier candidate, the higher utility)
pub fn leader(vals: &[Vec<f64>]) -> usize {
    (1..vals.len()).fold(0, |b, k| if mean(&vals[k]) > mean(&vals[b]) { k } else { b })
}

/// the search of one decision of p's over candidates `keys` (labels for the tree's root level), `ti` the policy's
/// top candidate when the pick is held to the margin against it. `play(g, k, j)` plays candidate k on copy j and
/// returns its value (bonuses in). Candidates first get `min_visits` playouts each on shared copies, then UCB
/// picks; it stops at the budget, at `max_visits` a candidate, when a candidate is clearly ahead of every other
/// (the margin holds against each), or when no candidate can be expected to beat the policy's top (`hopeless`).
pub fn search_root(
    g: &mut Game,
    p: PlayerId,
    keys: Vec<(String, usize)>,
    ti: Option<usize>,
    z: f64,
    cap: u32,
    min_visits: u32,
    mut play: impl FnMut(&mut Game, usize, u64) -> f64,
) -> Searched {
    let kn = Knobs { min_visits, ..knobs() };
    let nk = keys.len();
    let mut nodes = vec![Node { key: (String::new(), 0), visits: 0, avail: 0, sum: 0.0, kids: (1..=nk).collect() }];
    nodes.extend(keys.into_iter().map(|key| Node { key, visits: 0, avail: 0, sum: 0.0, kids: vec![] }));
    let top_k = g.settings.search.top_k;
    TREE.with(|t| {
        *t.borrow_mut() =
            Some(Tree { seat: p, nodes, cur: None, path: vec![], depth: 0, expanded: false, knobs: kn, top_k })
    });
    let w0 = g.search_work;
    let mut vals: Vec<Vec<f64>> = vec![vec![]; nk];
    loop {
        let n: Vec<u32> = vals.iter().map(|v| v.len() as u32).collect();
        let total: u32 = n.iter().sum();
        let k = if let Some(k) = (0..nk).filter(|&k| n[k] < kn.min_visits).min_by_key(|&k| n[k]) {
            k
        } else {
            let ucb = |k: usize| mean(&vals[k]) + kn.c * ((total as f64).ln() / n[k] as f64).sqrt();
            let mut k = (0..nk).filter(|&k| n[k] < cap).reduce(|a, b| if ucb(b) > ucb(a) { b } else { a });
            // the pick is held to the margin against the policy's top: keep the two compared on the same copies
            if let (Some(kk), Some(t)) = (k, ti) {
                let l = leader(&vals);
                if kk == l && l != t && n[t] < n[l] && n[t] < cap {
                    k = Some(t);
                }
            }
            match k {
                Some(k) => k,
                None => break,
            }
        };
        TREE.with(|t| {
            if let Some(tr) = t.borrow_mut().as_mut() {
                tr.cur = Some(k + 1);
                tr.path = vec![k + 1];
                tr.depth = 1;
                tr.expanded = false;
            }
        });
        let v = play(g, k, n[k] as u64);
        TREE.with(|t| {
            if let Some(tr) = t.borrow_mut().as_mut() {
                let path = std::mem::take(&mut tr.path);
                for i in path {
                    tr.nodes[i].visits += 1;
                    tr.nodes[i].sum += v.min(100.0);
                }
                tr.nodes[0].visits += 1;
                tr.cur = None;
            }
        });
        vals[k].push(v);
        if g.search_work - w0 >= kn.budget {
            break;
        }
        if vals.iter().any(|v| (v.len() as u32) < kn.min_visits) {
            continue;
        }
        let l = leader(&vals);
        let clear = (0..nk).filter(|&k| k != l).all(|k| {
            let d = paired(&vals, l, k);
            // overruling the policy's top needs `pairs` shared copies; the rest (and keeping the top) two
            let need = if ti == Some(k) { kn.pairs.max(2) } else { 2 };
            d.len() >= need && super::search::margin_holds(&d, z)
        });
        if clear {
            break;
        }
        if let Some(t) = ti
            && (0..nk).filter(|&k| k != t).all(|k| {
                let d = paired(&vals, k, t);
                d.len() as u32 >= kn.min_visits.max(3) && super::search::hopeless(&d, 0.5)
            })
        {
            break;
        }
    }
    TREE.with(|t| *t.borrow_mut() = None);
    Searched { vals, steps: g.search_work - w0 }
}

/// the pick: the leader, held to the margin against the policy's top `ti` (on their shared copies)
pub fn pick(vals: &[Vec<f64>], ti: Option<usize>, z: f64, pairs: usize) -> usize {
    let l = leader(vals);
    match ti {
        Some(t)
            if l != t && {
                let d = paired(vals, l, t);
                d.len() < pairs || !super::search::margin_holds(&d, z)
            } =>
        {
            t
        }
        _ => l,
    }
}

// ------------------------------------------------------------------ the decisions
fn policy(g: &mut Game, p: PlayerId, kind: usize) {
    g.player_mut(p).search_cost.policy[kind] += 1;
}

/// a main-phase decision of p's by ISMCTS (an index into opts), or the policy's top play when little is at stake
pub fn choose_main(g: &mut Game, p: PlayerId, post: bool, opts: &[Opt]) -> Option<usize> {
    use super::search::{WIN_FLOOR, hold_bonus, hold_credit, playout_main_kept, ramp_bonus};
    let cands = candidates(opts, g.settings.search.top_k);
    let top_u = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    let kn = knobs();
    if !high_stakes_main(&stakes(g, p, opts), &kn) {
        policy(g, p, SearchCost::MAIN);
        return Some(top_u);
    }
    let parts = format!("{}:{}:{}", g.player(p).key, g.player(p).hand.len(), g.player(p).perms.len());
    let base_seed = super::search::decision_seed(g, &parts);
    let hold = if g.settings.search.hold_kept { hold_credit(g, p) } else { None };
    let ti = cands.iter().position(|c| c.0 == top_u);
    let z = super::search::margin_z(&g.settings.search);
    let keys: Vec<(String, usize)> = cands.iter().map(|c| c.1.clone()).collect();
    let s = search_root(g, p, keys, ti, z, kn.max_visits, kn.min_visits, |g, k, r| {
        let (i, (label, n)) = (&cands[k].0, &cands[k].1);
        let (v, kept) = playout_main_kept(g, p, post, label, *n, base_seed, r, hold.map(|h| h.0));
        let extra = if v < WIN_FLOOR {
            let h = match hold {
                Some((_, w)) if kept => w,
                Some(_) => 0.0,
                None if g.settings.search.hold_kept => 0.0,
                None => hold_bonus(g, p, &opts[*i]),
            };
            ramp_bonus(g, p, &opts[*i]) + h
        } else {
            0.0
        };
        v + extra
    });
    let best = pick(&s.vals, ti, z, kn.pairs);
    super::search::LAST_SCORES.with(|l| {
        *l.borrow_mut() = cands.iter().zip(&s.vals).map(|(c, v)| (c.1.0.clone(), mean(v))).collect();
    });
    if cands[best].0 != top_u {
        super::search::stat(|x| x.changed += 1);
    }
    Some(cands[best].0)
}

/// p's attack plan by ISMCTS, or None (the policy's attack) when little is at stake
pub fn choose_attack(g: &mut Game, p: PlayerId) -> Option<(PlayerId, Sym)> {
    let cands = super::search::attack_cands(g, p);
    if cands.is_empty() {
        return None;
    }
    if !high_stakes_attack(&stakes(g, p, &[]), &knobs()) {
        policy(g, p, SearchCost::ATTACK);
        return None;
    }
    let parts = format!("{}:atk", g.player(p).key);
    let base_seed = super::search::decision_seed(g, &parts);
    let keys: Vec<(String, usize)> = cands.iter().map(|c| (format!("atk:{}:{}", c.0.0, c.1), 0)).collect();
    let z = super::search::margin_z(&g.settings.search);
    let cap = g.settings.search.rollouts; // today's count at most: an attack's plans are many and close
    let s = search_root(g, p, keys, None, z, cap, knobs().atk_min, |g, k, r| {
        super::search::playout_attack(g, p, cands[k], base_seed, r)
    });
    let means: Vec<f64> = s.vals.iter().map(|v| mean(v)).collect();
    let mut best = leader(&s.vals);
    if !super::is_main(g.player(p).key) {
        best = super::search::attack_on_near_tie(&cands, &means, 1.0, best); // --- casual speed
    }
    let (d, how) = cands[best];
    crate::glog!(g, "      [{} attack plan by search: {how} at {}]", g.player(p).name, g.player(d).name);
    Some(cands[best])
}

/// q's counter decision on p's spell c by ISMCTS: true to counter (a counter war is always worth the search)
pub fn choose_counter(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId) -> bool {
    let parts = format!("{}:ctr", g.player(q).key);
    let base_seed = super::search::decision_seed(g, &parts);
    let keys = vec![("ctr:yes".to_string(), 0), ("ctr:no".to_string(), 0)];
    let cap = g.settings.search.rollouts; // today's count at most
    let z = super::search::margin_z(&g.settings.search);
    let s = search_root(g, q, keys, None, z, cap, knobs().atk_min, |g, k, r| {
        super::search::playout_counter(g, q, p, c, k == 0, base_seed, r)
    });
    mean(&s.vals[0]) > mean(&s.vals[1])
}

// ------------------------------------------------------------------ part (b): blocks, protection, wipe responses
// --- phase4-search-b (`audit/phase4/search-design.md`, "Part (b)")

/// does p's block / protection / wipe decision here count in the survey? (a generic-policy seat in the real game of
/// a look-ahead run)
pub fn noting(g: &Game, p: PlayerId) -> bool {
    !g.in_search && g.settings.ai == crate::settings::AiMode::Lookahead && super::values::generic_deck(g, p)
}

/// count one decision of `kind` for p's survey: came up (`hot` false, `acted` false), or its stakes, or the
/// heuristic acting
pub fn note(g: &mut Game, p: PlayerId, kind: usize, hot: bool, acted: bool) {
    let c = &mut g.player_mut(p).search_cost;
    if acted {
        c.acted[kind] += 1;
    } else {
        c.seen[kind] += 1;
        c.hot[kind] += hot as u32;
    }
}

/// the damage p's attackers `atk` deal d with these blocks (double strike twice; the blocked deal none)
pub fn unblocked_damage(g: &Game, atk: &[PermId], assign: &[(PermId, PermId)]) -> i32 {
    use crate::engine::combat::double_strike;
    atk.iter()
        .filter(|&&a| g.perm(a).on_bf && !assign.iter().any(|x| x.0 == a))
        .map(|&a| epow(g, a).max(0) * if double_strike(g, a) { 2 } else { 1 })
        .sum()
}

/// the survey of d's blocks against p's attack: seen when one of d's untapped creatures can block an attacker, hot
/// when the attack reaches half d's life, acted when the heuristic's blocks still leave lethal
pub fn note_blocks(g: &mut Game, atk: &[PermId], d: PlayerId, unbl: &[PermId], assign: &[(PermId, PermId)]) {
    if !noting(g, d) {
        return;
    }
    let can = atk.iter().any(|&a| {
        !unbl.contains(&a)
            && g.perm(a).on_bf
            && g.player(d).perms.iter().any(|&b| {
                g.is_creature(b)
                    && !g.perm(b).tapped
                    && !g.perm(b).phased
                    && crate::engine::combat::can_block(g, b, a)
            })
    });
    if !can {
        return;
    }
    let life = g.player(d).life;
    let hot = 2 * unblocked_damage(g, atk, &[]) >= life;
    note(g, d, SearchCost::BLOCK, hot, false);
    if unblocked_damage(g, atk, assign) >= life {
        note(g, d, SearchCost::BLOCK, false, true);
    }
}

// ---- the switch
fn mode_b() -> &'static Mode {
    static M: OnceLock<Mode> = OnceLock::new();
    M.get_or_init(|| parse_mode_b(std::env::var("AI_SEARCH_B").ok().as_deref()))
}

/// `AI_SEARCH_B`: unset or `0` PR 5's behaviour (the heuristic blocks, protects and answers wipes), `1` the search
/// for every generic-policy deck on ISMCTS, `a,b` for those keys
fn parse_mode_b(v: Option<&str>) -> Mode {
    match v.map(str::trim) {
        None | Some("") | Some("0") => Mode::Off,
        Some("1") => Mode::All,
        Some(list) => Mode::Keys(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()),
    }
}

thread_local! {
    static FORCE_B: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// for tests: part (b)'s searches on or off on this thread, whatever `AI_SEARCH_B` says (None: back to it)
pub fn force_b(on: Option<bool>) {
    FORCE_B.with(|f| f.set(on));
}

/// does p decide its blocks, protection and wipe responses by search here? (a generic-policy deck on ISMCTS, in the
/// real game, the look-ahead on and within its per-game caps)
fn on_b(g: &mut Game, p: PlayerId) -> bool {
    let want = match FORCE_B.with(|f| f.get()) {
        Some(f) => f,
        None => match mode_b() {
            Mode::Off => false,
            Mode::All => true,
            Mode::Keys(k) => k.iter().any(|x| x == g.player(p).key),
        },
    };
    want && !g.in_search && on(g, p) && g.humans.get(p).is_none() && super::search::enabled(g, p)
}

/// the fraction of the defender's life the attack must reach (unblocked) for its blocks to be searched
/// (`AI_SEARCH_B_BLOCK`)
pub const BLOCK_STAKES: f64 = 0.5;

fn block_stakes() -> f64 {
    static B: OnceLock<f64> = OnceLock::new();
    *B.get_or_init(|| env_or("AI_SEARCH_B_BLOCK", BLOCK_STAKES))
}

/// the root search of a part (b) decision of p's: candidates `keys`, the heuristic's choice `ti`; each candidate at
/// most `rollouts` playouts (today's count, as attacks and counters), within the budget. The pick is held to the
/// margin against the heuristic's choice on `pairs` shared copies.
fn search_b(
    g: &mut Game,
    p: PlayerId,
    kind: usize,
    tag: &str,
    keys: Vec<(String, usize)>,
    ti: usize,
    mut play: impl FnMut(&mut Game, usize, u64, u64) -> f64,
) -> usize {
    let parts = format!("{}:{tag}", g.player(p).key);
    let kn = knobs();
    super::search::booked(g, p, kind, |g| {
        let base_seed = super::search::decision_seed(g, &parts);
        let z = super::search::margin_z(&g.settings.search);
        let cap = g.settings.search.rollouts;
        let s = search_root(g, p, keys, Some(ti), z, cap, kn.min_visits, |g, k, r| play(g, k, base_seed, r));
        pick(&s.vals, Some(ti), z, kn.pairs)
    })
}

// ---- blocks
/// the block plans the search tries for d against p's attack `atk` (`unbl`: can't be blocked), the heuristic's
/// first: (label, blocks). Dropped when equal to an earlier plan. Menace attackers are blocked only as the heuristic
/// blocks them; the engine has one blocker an attacker, so no double blocks.
pub fn block_plans(
    g: &Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    unbl: &[PermId],
    heur: &[(PermId, PermId)],
) -> Vec<(String, Vec<(PermId, PermId)>)> {
    use crate::engine::combat::{can_block, fiery, kw};
    use crate::engine::values::{etgh, pval};
    let blockers: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased)
        .collect();
    let (fd, fp) = (fiery(g, d), fiery(g, p));
    let kills = |b: PermId, a: PermId| epow(g, b) * fd >= etgh(g, a) || g.perm(b).dt;
    let dies = |b: PermId, a: PermId| epow(g, a) * fp >= etgh(g, b) || g.perm(a).dt;
    let mut order: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&a| g.perm(a).on_bf && !unbl.contains(&a) && !kw(g, a, "menace"))
        .collect();
    order.sort_by_key(|&a| -epow(g, a));
    let cheapest = |c: Vec<PermId>| -> Option<PermId> {
        c.into_iter().min_by(|&x, &y| pval(g, x).total_cmp(&pval(g, y)))
    };
    let free = |used: &[(PermId, PermId)], a: PermId, f: &dyn Fn(PermId) -> bool| -> Option<PermId> {
        cheapest(
            blockers
                .iter()
                .copied()
                .filter(|&b| !used.iter().any(|x| x.1 == b) && can_block(g, b, a) && f(b))
                .collect(),
        )
    };
    let blocked = |used: &[(PermId, PermId)], a: PermId| used.iter().any(|x| x.0 == a);
    // kill-and-survive blocks only
    let mut good = vec![];
    for &a in &order {
        if let Some(b) = free(&good, a, &|b| kills(b, a) && !dies(b, a)) {
            good.push((a, b));
        }
    }
    // every attacker blocked: kill-and-survive, else a trade, else a chump
    let mut all = good.clone();
    for &a in &order {
        if blocked(&all, a) || epow(g, a) <= 0 {
            continue;
        }
        if let Some(b) = free(&all, a, &|b| kills(b, a)).or_else(|| free(&all, a, &|_| true)) {
            all.push((a, b));
        }
    }
    // the heuristic's blocks, then chumps on the biggest attackers left until the rest is not lethal
    let mut live = heur.to_vec();
    let life = g.player(d).life;
    for &a in &order {
        if unblocked_damage(g, atk, &live) < life {
            break;
        }
        if blocked(&live, a) {
            continue;
        }
        if let Some(b) = free(&live, a, &|_| true) {
            live.push((a, b));
        }
    }
    let mut out: Vec<(String, Vec<(PermId, PermId)>)> = vec![];
    let norm = |v: &[(PermId, PermId)]| {
        let mut v = v.to_vec();
        v.sort_by_key(|x| (x.0.0, x.1.0));
        v
    };
    for (label, plan) in
        [("blk:heur", heur.to_vec()), ("blk:none", vec![]), ("blk:good", good), ("blk:live", live), ("blk:all", all)]
    {
        if out.iter().all(|x| norm(&x.1) != norm(&plan)) {
            out.push((label.to_string(), plan));
        }
    }
    out
}

/// d's blocks against p's attack by search (None: the heuristic's `heur` stand): searched when the attack, unblocked,
/// reaches `AI_SEARCH_B_BLOCK` of d's life and there is another plan to try
pub fn choose_blocks(
    g: &mut Game,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    unbl: &[PermId],
    heur: &[(PermId, PermId)],
) -> Option<Vec<(PermId, PermId)>> {
    if !on_b(g, d) {
        return None;
    }
    let incoming = unblocked_damage(g, atk, &[]);
    if (incoming as f64) < block_stakes() * g.player(d).life as f64 {
        policy(g, d, SearchCost::BLOCK);
        return None;
    }
    let plans = block_plans(g, p, atk, d, unbl, heur);
    if plans.len() < 2 {
        policy(g, d, SearchCost::BLOCK);
        return None;
    }
    let keys = plans.iter().map(|x| (x.0.clone(), 0)).collect();
    let atk = atk.to_vec();
    let best = search_b(g, d, SearchCost::BLOCK, "blk", keys, 0, |g, k, seed, r| {
        let plan = plans[k].1.clone();
        let atk = atk.clone();
        super::search::playout_with(g, d, seed, r, |g2| {
            let atk2: Vec<PermId> =
                atk.iter().copied().filter(|&m| g2.perm(m).on_bf && g2.perm(m).owner == p).collect();
            let mut tot = 0;
            crate::engine::combat::resolve_blocks(g2, p, atk2, d, plan, &mut tot)?;
            // the end of this combat (later defenders of a split attack and extra combats are not played)
            crate::impls::t0_zurgo::end_of_combat(g2, p)?;
            g2.in_combat.clear();
            g2.end_combat = false;
            g2.blocking.clear();
            let pl = g2.player_mut(p);
            pl.haste_all = false;
            pl.trample = false;
            pl.pump = 0;
            g2.step = crate::state::Step::Combat; // resumes at the second main phase
            Ok(true)
        })
    });
    if best != 0 {
        super::search::stat(|x| x.changed += 1);
        crate::glog!(g, "      [{} blocks by search: {}]", g.player(d).name, plans[best].0);
    }
    Some(plans[best].1.clone())
}

// ---- protection
/// owner's protection of m from actor's targeted removal `kind` by search: Some(true) protected, Some(false) let it
/// resolve, None not searched (the heuristic decides). Searched when a protector would save m.
pub fn choose_protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> crate::flow::Res<Option<bool>> {
    if actor == Some(owner) || !on_b(g, owner) {
        return Ok(None);
    }
    let opts = super::pool::protect_options(g, owner, m, kind, spell, 3);
    if opts.is_empty() {
        return Ok(None);
    }
    let heur = super::pool::protect_heuristic(g, owner, m, kind, spell);
    let ti = heur.and_then(|h| opts.iter().position(|o| (o.1, o.2) == h)).map_or(0, |i| i + 1);
    let mut keys = vec![("prot:none".to_string(), 0)];
    keys.extend(opts.iter().map(|o| (o.0.clone(), 0)));
    let best = search_b(g, owner, SearchCost::PROTECT, "prot", keys, ti, |g, k, seed, r| {
        let opt = (k > 0).then(|| (opts[k - 1].1, opts[k - 1].2));
        super::search::playout_with(g, owner, seed, r, |g2| {
            if !g2.perm(m).on_bf {
                return Ok(true);
            }
            match opt {
                Some((i, src)) => super::pool::protect_play(g2, owner, m, i, src),
                None => {
                    crate::cardcode::sac_in_response(g2, owner, m, kind)?;
                    crate::engine::removal::apply_removal_rest(g2, actor, m, kind, spell)?;
                    Ok(true)
                }
            }
        })
    });
    if best != ti {
        super::search::stat(|x| x.changed += 1);
    }
    if best == 0 {
        if heur.is_some() {
            crate::glog!(g, "      [{} holds its protection by search]", g.player(owner).name);
        }
        return Ok(Some(crate::cardcode::sac_in_response(g, owner, m, kind)?));
    }
    let (_, i, src) = opts[best - 1];
    if super::pool::protect_play(g, owner, m, i, src)? {
        return Ok(Some(true));
    }
    Ok(None) // it couldn't be used after all: the heuristic decides
}

// ---- wipe responses
/// q's answer to a wipe of `kind` by search: Some(answer) searched, None not searched (the heuristic decides).
/// Searched when a board-wide protector (or Golgari Charm's regeneration) would save q's board, inside an ordinary
/// wipe (`removal::apply_wipe_inner`; the three sweepers of `impls::sweepers` keep the heuristic).
pub fn choose_wipe(
    g: &mut Game,
    q: PlayerId,
    kind: Sym,
    caster: PlayerId,
) -> crate::flow::Res<Option<Option<Sym>>> {
    use crate::engine::removal::{WIPE_CTX, wipe_finish, wipe_victim};
    let Some(ctx) = WIPE_CTX.with(|w| w.borrow().clone()).filter(|c| c.kind == kind && c.caster == caster) else {
        return Ok(None);
    };
    if !on_b(g, q) {
        return Ok(None);
    }
    if g.player(q).perms.iter().any(|&x| g.perm(x).cd.is_some_and(|c| &*g.db.get(c).name == "Lena, Selfless Champion"))
    {
        return Ok(None); // Lena's own rule: the heuristic
    }
    let regen = kind == "destroy" && g.player(q).hand.iter().any(|&c| &*g.db.get(c).name == "Golgari Charm");
    let opts = super::pool::wipe_options(g, q, kind, 3);
    if opts.is_empty() && !regen {
        return Ok(None);
    }
    // options: 0 none, then regeneration, then the protectors
    let mut keys = vec![("wipe:none".to_string(), 0)];
    if regen {
        keys.push(("wipe:regen".to_string(), 0));
    }
    keys.extend(opts.iter().map(|o| (o.0.clone(), 0)));
    let off = 1 + regen as usize;
    let ti = if regen {
        1
    } else {
        opts.iter()
            .position(|o| super::pool::wipe_heuristic_takes(g, q, o.1))
            .map_or(0, |i| i + off)
    };
    let best = search_b(g, q, SearchCost::WIPE, "wipe", keys, ti, |g, k, seed, r| {
        let ctx = ctx.clone();
        let opt = (k >= off).then(|| (opts[k - off].1, opts[k - off].2));
        super::search::playout_with(g, q, seed, r, |g2| {
            let prot = if k == 0 {
                None
            } else if regen && k == 1 {
                if !crate::cardcode::regen_wipe(g2, q)? {
                    return Ok(false);
                }
                Some("indes")
            } else {
                let (i, src) = opt.unwrap();
                match super::pool::wipe_play(g2, q, i, src)? {
                    Some(r) => Some(r),
                    None => return Ok(false),
                }
            };
            wipe_victim(g2, caster, kind, q, prot, &ctx.modes, ctx.deluge_x)?;
            for &v in &ctx.rest {
                if g2.over {
                    return Ok(true);
                }
                let pv = super::wipe_response(g2, v, kind, caster)?;
                wipe_victim(g2, caster, kind, v, pv, &ctx.modes, ctx.deluge_x)?;
            }
            wipe_finish(g2, kind, &ctx.modes)?;
            Ok(true)
        })
    });
    if best != ti {
        super::search::stat(|x| x.changed += 1);
    }
    if best == 0 {
        if ti != 0 {
            crate::glog!(g, "      [{} holds its wipe protection by search]", g.player(q).name);
        }
        return Ok(Some(None));
    }
    if regen && best == 1 {
        if crate::cardcode::regen_wipe(g, q)? {
            return Ok(Some(Some("indes")));
        }
        return Ok(None);
    }
    let (_, i, src) = opts[best - off];
    match super::pool::wipe_play(g, q, i, src)? {
        Some(r) => Ok(Some(Some(r))),
        None => Ok(None),
    }
}
