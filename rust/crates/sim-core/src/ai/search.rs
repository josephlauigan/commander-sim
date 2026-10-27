//! The look-ahead (Python's `ai/search.py`): main-phase plays, attack plans and counterspells are tried on copies of
//! the game, and the choice that leads to the best position is kept.
//!
//! For a decision of player p:
//! 1. the candidates are the heuristic AI's top plays (by its own utility) plus "stop";
//! 2. each candidate is tried in `rollouts` copies of the game. Each copy hides what p can't see: every opponent's
//!    hand is re-dealt from their hand and library, and p's own library is shuffled;
//! 3. in the copy the play is made, and the game goes on with the heuristic AI until the end of p's next turn;
//! 4. the position reached is scored from p's point of view (`evaluate`). All candidates share the copies' random
//!    seeds, so they are compared on the same futures.
//!
//! A copy is `Game::clone` (about 2 µs, where Python's deepcopy took 4 ms). Practice mode's tape (Undo replays a
//! game's look-ahead decisions instead of searching again) comes with phase 9.

pub use super::tape::{LAST_SCORES, Tape, TapeEntry, TapeSlot};
use super::tape::{Taped, taped};
use super::{act, brain};
use crate::flow::{Res, Stop};
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PlayerId};
use crate::pysum::PySum;
use crate::rng::Rng;
use crate::settings::AiMode;
use crate::state::{Game, Step};
use crate::sym::Sym;
use crate::tag::Tag;
use std::cell::RefCell;

/// run counters (search.STATS): decisions, playouts, decisions the look-ahead changed, playouts cut short
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub decisions: u64,
    pub playouts: u64,
    pub changed: u64,
    pub cut: u64,
    pub max_work: u64,
    pub capped: u64,
    pub big_board: u64,
}

thread_local! {
    static STATS: RefCell<Stats> = RefCell::new(Stats::default());
}

/// this thread's look-ahead counters so far
pub fn stats() -> Stats {
    STATS.with(|s| s.borrow().clone())
}

fn stat(f: impl FnOnce(&mut Stats)) {
    STATS.with(|s| f(&mut s.borrow_mut()));
}

/// search.enabled: does p decide by look-ahead here?
pub fn enabled(g: &mut Game, p: PlayerId) -> bool {
    let _ = p;
    if g.in_search || g.settings.ai != AiMode::Lookahead {
        return false;
    }
    let s = &g.settings.search;
    if g.players.iter().map(|q| q.perms.len()).sum::<usize>() > s.board_limit {
        stat(|x| x.big_board += 1);
        return false;
    }
    if g.search_work > s.game_search_work {
        stat(|x| x.capped += 1);
        return false;
    }
    if g.search_n >= s.game_decisions {
        if g.search_n == s.game_decisions {
            stat(|x| x.capped += 1);
            g.search_n += 1;
        }
        return false;
    }
    true
}

/// search._decision_seed: a new decision in game g, and its random seed
fn decision_seed(g: &mut Game, parts: &str) -> u64 {
    stat(|x| x.decisions += 1);
    g.search_n += 1;
    let name = format!("search:{}:{}:{parts}", g.search_n, g.round);
    Rng::named(&name).next_u64() >> 16
}

// ------------------------------------------------------------------ copying a game
/// search.clone: a copy of g to play on (the trace log isn't copied; a copy never has people in it)
pub fn clone(g: &mut Game) -> Game {
    let log = g.log.take();
    let audit = g.audit.take(); // --- ai-audit: a copy keeps no loss accounting
    let mut g2 = g.clone();
    g.log = log;
    g.audit = audit; // --- ai-audit
    g2.cur_cast = None;
    g2.resolving = 0;
    g2.trig_probe = false;
    g2.trig_current = None; // pending triggers come along (play_on settles them)
    g2
}

/// search.determinize: hide what `me` can't know: re-deal opponents' hands from hand and library, shuffle my library
pub fn determinize(g2: &mut Game, me: PlayerId, rng: &mut Rng) {
    for i in 0..g2.players.len() {
        let q = &mut g2.players[i];
        if q.id == me {
            rng.shuffle(&mut q.library);
        } else if q.alive {
            let (cmd, cmd2) = (q.cmd, q.cmd2);
            // a commander put into hand (Command Beacon) is known
            let keep: Vec<CardId> = q.hand.iter().copied().filter(|&c| c == cmd || Some(c) == cmd2).collect();
            let mut pool: Vec<CardId> = q.hand.iter().copied().filter(|&c| c != cmd && Some(c) != cmd2).collect();
            pool.append(&mut q.library);
            rng.shuffle(&mut pool);
            let n = q.hand.len() - keep.len();
            let lib = pool.split_off(n);
            q.hand = keep;
            q.hand.extend(pool);
            q.library = lib;
        }
    }
}

/// a copy of the real game g begins a playout: its step budget (its work is booked to g)
fn start(g2: &mut Game, g: &Game) {
    g2.in_search = true;
    g2.work_cap = g2.work + g.settings.search.playout_work;
    g2.board_cap = Some(g.settings.search.board_limit);
}

/// book-keeping after a playout: its work goes to the real game's budget
fn done(g: &mut Game, g2: &Game, start_work: u64, cut: bool) {
    let w = g2.work - start_work;
    g.search_work += w;
    stat(|x| {
        x.playouts += 1;
        x.max_work = x.max_work.max(w);
        if cut {
            x.cut += 1;
        }
    });
}

/// a playout's copy, ready to play: copied, budgeted, determinized for `me`, with its own random numbers
fn playout_copy(g: &mut Game, me: PlayerId, base_seed: u64, r: u64) -> (Game, u64) {
    let mut rng = Rng::from_u64(base_seed.wrapping_mul(31).wrapping_add(r));
    let mut g2 = clone(g);
    start(&mut g2, g);
    determinize(&mut g2, me, &mut rng);
    g2.rng = Rng::from_u64(rng.next_u64());
    let w = g2.work;
    (g2, w)
}

// ------------------------------------------------------------------ playing a copy forward
/// search.play_on: finish the active player's turn (p2's by default) from the current step, then everyone's turns
/// until the end of p2's next turn (`horizon` of p2's turns)
pub fn play_on(g2: &mut Game, p2: PlayerId, active: Option<PlayerId>) -> Res {
    const STOP_ROUNDS: u32 = 20;
    let act = active.unwrap_or(p2);
    if !g2.stack.is_empty() || !g2.trig_queue.is_empty() {
        crate::engine::stack::settle_stack(g2)?; // copied mid-stack: finish it first
    }
    if g2.player(act).alive && !g2.over {
        let step = g2.step;
        crate::engine::turn::continue_turn(g2, act, step)?;
        crate::engine::turn::run_extra_turns(g2)?;
    }
    let n = g2.players.len();
    let mut k = act.index();
    let mut left = g2.settings.search.horizon;
    while !g2.over && g2.player(p2).alive {
        k = (k + 1) % n;
        if k == 0 {
            g2.round += 1;
        }
        if g2.round > STOP_ROUNDS {
            break;
        }
        let q = PlayerId(k as u8);
        if !g2.player(q).alive {
            continue;
        }
        if g2.player(q).skip_turns > 0 {
            g2.player_mut(q).skip_turns -= 1; // Ral Zarek -7
            continue;
        }
        brain::end_of_turn_window(g2, q)?;
        if g2.over {
            break;
        }
        if g2.player(q).alive {
            crate::engine::turn::play_turn(g2, q)?;
        }
        if q == p2 {
            left = left.saturating_sub(1);
            if left == 0 {
                break;
            }
        }
    }
    Ok(())
}

/// search.evaluate: how good the position is for p: a win or a loss, else p's strength against the strongest
/// opponents (always below a win, however big the board)
pub fn evaluate(g: &Game, p: PlayerId) -> f64 {
    if g.over {
        return if g.winner == Some(p) { 100.0 } else { -100.0 };
    }
    if !g.player(p).alive {
        return -100.0;
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return 100.0;
    }
    let dead = g.players.iter().filter(|q| q.id != p && !q.alive).count() as f64;
    let s = strength(g, p);
    let mut so: Vec<f64> = opps.iter().map(|&q| strength(g, q)).collect();
    so.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let raw = s - 0.6 * so[0] - 0.4 * (so.iter().copied().psum() / so.len() as f64) + 12.0 * dead;
    95.0 * (raw / 100.0).tanh()
}

/// search.strength
pub fn strength(g: &Game, q: PlayerId) -> f64 {
    use crate::engine::values::{epow, pval};
    let pl = g.player(q);
    let life = pl.life.clamp(0, 60) as f64;
    let live = || pl.perms.iter().copied().filter(|&m| !g.perm(m).phased);
    let board = live().map(|m| pval(g, m)).psum();
    let power = live().filter(|&m| g.is_creature(m)).map(|m| epow(g, m) as f64).sum::<f64>(); // whole numbers
    0.25 * life
        + board
        + 0.35 * power
        + 0.9 * pl.hand.len() as f64
        + 0.6 * pl.lands.len() as f64
        + 0.4 * pl.treasures as f64
        + g.settings.search.combo_w * crate::cardcode::combo_progress(g, q)
}

/// own turns over which ramp still pays off (`ramp_bonus`)
pub const RAMP_TURNS: u32 = 8;

/// search.ramp_bonus: the look-ahead plays only to the end of p's next turn, and on the "hold it" line the heuristic
/// casts the rock next turn anyway, so both lines end on the same board and a creature that attacks a turn sooner wins
/// (report-cloud-sequencing: rocks held 2+ turns in half of Cloud's games). Ramp cast a turn earlier pays every turn
/// after that: a rock, dork or land-ramp spell cast on p's own turn t gets `ramp_w` per mana it makes per turn left
/// before `RAMP_TURNS`, added to the option's rollout average. `ramp_w` 0: no bonus (the old search).
/// Off by default: at `ramp_w` 1 Cloud T1 +0.1, Titania T3 -1.3, Sheoldred T4 -1.9 (1,000 games each, noise about
/// +-2): casting ramp sooner doesn't win more in these games (audit/ramp/2026-10-15).
pub fn ramp_bonus(g: &Game, p: PlayerId, o: &Opt) -> f64 {
    let w = g.settings.search.ramp_w;
    let Some(Action::Cast { card, .. }) = &o.act else { return 0.0 };
    if w == 0.0 {
        return 0.0;
    }
    let t = &g.db.get(*card).tags;
    let mana = if let Some(r) = t.str(Tag::Rock) {
        r.split_once(':').map_or(r, |x| x.0).parse().unwrap_or(1)
    } else if t.has(Tag::Dork) {
        1
    } else {
        t.int(Tag::Lr).unwrap_or(0).max(0)
    };
    w * mana as f64 * RAMP_TURNS.saturating_sub(g.player(p).turns) as f64
}

/// search.hold_bonus: the look-ahead plays only to the end of p's next turn and scores a permanent on the battlefield
/// above cards in hand, so the "stop (hold mana)" line (mana kept up for a counterspell, protection or Ad Nauseam at
/// the end of the turn before yours) loses to tapping out even when the heuristic rates holding far higher: what the
/// held card protects or finds pays off after the horizon. The hold line gets `hold_w` times the heuristic's own hold
/// value (`brain::hold_value`), added to the option's rollout average. `hold_w` 0: no bonus (the old search).
pub fn hold_bonus(g: &Game, p: PlayerId, o: &Opt) -> f64 {
    let w = g.settings.search.hold_w;
    if w == 0.0 || o.act.is_some() || o.label != "stop (hold mana)" {
        return 0.0;
    }
    let s = brain::Situation::new(g, p);
    w * brain::hold_value(g, p, &s).1.max(0.0)
}

/// the n-th option labelled `label`
fn find(opts: &[Opt], label: &str, n: usize) -> Option<usize> {
    opts.iter().enumerate().filter(|(_, o)| o.label == label).nth(n).map(|(i, _)| i)
}

// ------------------------------------------------------------------ main phase
/// search.choose: pick a main-phase play by look-ahead (an index into opts); None to let the heuristic choose.
/// In a practice game the result is recorded on the game's tape, or taken off it when the game is being replayed.
pub fn choose(g: &mut Game, p: PlayerId, post: bool, opts: &[Opt]) -> Option<usize> {
    if g.search_tape.0.is_none() {
        return choose_inner(g, p, post, opts);
    }
    if !opts.iter().any(|o| o.act.is_some()) {
        return None;
    }
    let parts = format!("{}:{}:{}", g.player(p).key, g.player(p).hand.len(), g.player(p).perms.len());
    let (res, replayed) = taped(g, "main", |g| {
        Taped::Main(
            choose_inner(g, p, post, opts)
                .map(|i| (opts[i].label.clone(), opts[..i].iter().filter(|x| x.label == opts[i].label).count())),
        )
    });
    if replayed {
        decision_seed(g, &parts);
    }
    match res {
        Taped::Main(Some((l, n))) => find(opts, &l, n),
        _ => None,
    }
}

fn choose_inner(g: &mut Game, p: PlayerId, post: bool, opts: &[Opt]) -> Option<usize> {
    let mut real: Vec<usize> = (0..opts.len()).filter(|&i| opts[i].act.is_some()).collect();
    if real.is_empty() {
        return None;
    }
    if let Some(i) = crate::impls::tymna_partners::commander_first(g, p, opts) {
        return Some(i); // thrasios, kraum: the commander comes down when it is the best play (no search)
    }
    let sink = crate::impls::tymna_partners::thrasios_sink(g, p, opts);
    if let Some(i) = sink {
        real.retain(|&x| x != i); // made after the search's best play when that is "stop"
        if real.is_empty() {
            return Some(i);
        }
    }
    real.sort_by(|&a, &b| opts[b].utility.partial_cmp(&opts[a].utility).unwrap_or(std::cmp::Ordering::Equal));
    real.truncate(g.settings.search.top_k);
    let stop = opts.iter().position(|o| o.act.is_none())?;
    real.push(stop);
    // (label, occurrence) identifies an option in a copy
    let keyed: Vec<(usize, &str, usize)> = real
        .iter()
        .map(|&i| (i, opts[i].label.as_str(), opts[..i].iter().filter(|x| x.label == opts[i].label).count()))
        .collect();
    let parts = format!("{}:{}:{}", g.player(p).key, g.player(p).hand.len(), g.player(p).perms.len());
    let base_seed = decision_seed(g, &parts);
    let mut scores = vec![0.0; keyed.len()];
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, &(_, label, n)) in keyed.iter().enumerate() {
            scores[k] += playout_main(g, p, post, label, n, base_seed, r);
        }
    }
    for (k, kk) in keyed.iter().enumerate() {
        scores[k] += ramp_bonus(g, p, &opts[kk.0]) * g.settings.search.rollouts as f64;
        scores[k] += hold_bonus(g, p, &opts[kk.0]) * g.settings.search.rollouts as f64;
    }
    let mut best = 0;
    for k in 1..keyed.len() {
        if scores[k] > scores[best] {
            best = k;
        }
    }
    if g.player(p).key == "karn" {
        for (k, kk) in keyed.iter().enumerate() {
            scores[k] += crate::impls::karn::search_bonus(g, p, &opts[kk.0]) * g.settings.search.rollouts as f64;
        }
        best = 0;
        for k in 1..keyed.len() {
            if scores[k] > scores[best] {
                best = k;
            }
        }
    }
    if g.player(p).key == "cloud" {
        for (k, kk) in keyed.iter().enumerate() {
            scores[k] += crate::impls::cloud::search_bonus(g, p, &opts[kk.0]) * g.settings.search.rollouts as f64;
        }
        best = 0;
        for k in 1..keyed.len() {
            if scores[k] > scores[best] {
                best = k;
            }
        }
    }
    let n = g.settings.search.rollouts as f64;
    // a bonus never lifts a line above a sure win: when every playout of the best play wins, a held line that also wins
    // (the heuristic combos next turn) scored 100 plus its hold bonus and beat it, so a ready Thassa's Oracle waited
    // turn after turn in goldfish games. Capped lines tie, and a tie goes to the play the heuristic rates higher.
    let uncapped = best; // --- ai-audit
    for s in &mut scores {
        *s = s.min(100.0 * n);
    }
    best = 0;
    for k in 1..keyed.len() {
        if scores[k] > scores[best] {
            best = k;
        }
    }
    if keyed[uncapped].0 != keyed[best].0 {
        crate::gf_audit::cap_changed(g); // --- ai-audit: the sure-win cap changed the pick
    }
    LAST_SCORES.with(|l| {
        *l.borrow_mut() = keyed.iter().zip(&scores).map(|(k, s)| (k.1.to_string(), s / n)).collect();
    });
    let top = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    if keyed[best].0 != top {
        stat(|x| x.changed += 1);
    }
    if let Some(i) = sink
        && opts[keyed[best].0].act.is_none()
    {
        return Some(i);
    }
    Some(keyed[best].0)
}

/// search.playout_main: one playout of main-phase option (label, n-th of that label) for p: copy r of the decision
/// seeded base_seed. The score of the position it reaches (an option that can't be played: -50).
pub fn playout_main(g: &mut Game, p: PlayerId, post: bool, label: &str, n: usize, base_seed: u64, r: u64) -> f64 {
    let (mut g2, w0) = playout_copy(g, p, base_seed, r);
    let res = (|| -> Res<Option<f64>> {
        let opts = brain::main_options(&mut g2, p, post)?;
        let Some(i) = find(&opts, label, n) else { return Ok(Some(-50.0)) };
        if let Some(a) = &opts[i].act {
            if !act::perform(&mut g2, p, a)? {
                return Ok(Some(-50.0));
            }
            brain::main(&mut g2, p, post)?; // the rest of this phase, heuristically
        }
        play_on_after_phase(&mut g2, p)?;
        Ok(None)
    })();
    let cut = match res {
        Ok(Some(v)) => return v,
        Ok(None) => false,
        Err(Stop::OutOfWork) => true,
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
        Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
    };
    let v = evaluate(&g2, p);
    done(g, &g2, w0, cut);
    v
}

/// search.play_on_after_phase: the chosen play is made and the phase finished: go on from the next step
fn play_on_after_phase(g2: &mut Game, p2: PlayerId) -> Res {
    g2.step = match g2.step {
        Step::Main1 => Step::Combat,
        _ => Step::End,
    };
    play_on(g2, p2, None)
}

// ------------------------------------------------------------------ attacks
/// search.choose_attack: at the first combat of p's turn: (defender, 'filtered' | 'all' | 'none') by look-ahead
pub fn choose_attack(g: &mut Game, p: PlayerId) -> Option<(PlayerId, Sym)> {
    if g.search_tape.0.is_none() {
        return choose_attack_inner(g, p);
    }
    if g.opps(p).next().is_none() {
        return None;
    }
    let (res, replayed) = taped(g, "attack", |g| Taped::Attack(choose_attack_inner(g, p)));
    let Taped::Attack(res) = res else { unreachable!() };
    if replayed {
        let parts = format!("{}:atk", g.player(p).key);
        decision_seed(g, &parts);
        if let Some((d, how)) = res {
            crate::glog!(g, "      [{} attack plan by search: {how} at {}]", g.player(p).name, g.player(d).name);
        }
    }
    res
}

/// --- casual speed: how much better (in `evaluate` points, a rollout average) "none" must score than the best attack
/// before an outside deck holds every attacker back. Six rollouts can't tell apart plans closer than about this: 57% of
/// the "none" plans in 100 traced Tier 0 games won by less than half a point, often with nothing at stake (one 1/1)
/// or unblockable damage on offer. A near tie goes to the attack, as at a casual table. JD's decks are unchanged.
pub const HOLD_MARGIN: f64 = 1.0;

/// --- casual speed: the chosen plan `best` (scores summed over `rollouts`), or the best attack when `best` is
/// "none" and wins by less than `HOLD_MARGIN`
pub fn attack_on_near_tie(cands: &[(PlayerId, Sym)], scores: &[f64], rollouts: f64, best: usize) -> usize {
    if cands[best].1 != "none" {
        return best;
    }
    let atk = (0..cands.len()).filter(|&k| cands[k].1 != "none").max_by(|&a, &b| scores[a].total_cmp(&scores[b]));
    match atk {
        Some(k) if (scores[best] - scores[k]) / rollouts < HOLD_MARGIN => k,
        _ => best,
    }
}

fn choose_attack_inner(g: &mut Game, p: PlayerId) -> Option<(PlayerId, Sym)> {
    let mut opps: Vec<PlayerId> = g.opps(p).collect();
    if g.player(p).key == crate::impls::garland::KEY {
        opps = crate::impls::garland::attack_targets(g, p, opps); // never the monarch while we hold a steal
    }
    if opps.is_empty() {
        return None;
    }
    let mut cands: Vec<(PlayerId, Sym)> = vec![];
    for &q in &opps {
        cands.push((q, "filtered"));
        cands.push((q, "all"));
    }
    cands.push((opps[0], "none"));
    let parts = format!("{}:atk", g.player(p).key);
    let base_seed = decision_seed(g, &parts);
    let mut scores = vec![0.0; cands.len()];
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, &c) in cands.iter().enumerate() {
            scores[k] += playout_attack(g, p, c, base_seed, r);
        }
    }
    if crate::impls::cloud::must_attack(g, p) {
        // an equipped Cloud attacks (the position score can't see a Sword's trigger pair): "none" is out
        for (k, c) in cands.iter().enumerate() {
            if c.1 == "none" {
                scores[k] = f64::NEG_INFINITY;
            }
        }
    }
    let mut best = 0;
    for k in 1..cands.len() {
        if scores[k] > scores[best] {
            best = k;
        }
    }
    if !super::is_main(g.player(p).key) {
        best = attack_on_near_tie(&cands, &scores, g.settings.search.rollouts as f64, best); // --- casual speed
    }
    let (d, how) = cands[best];
    crate::glog!(g, "      [{} attack plan by search: {how} at {}]", g.player(p).name, g.player(d).name);
    Some(cands[best])
}

/// search.playout_attack: one playout of attack plan c for p
pub fn playout_attack(g: &mut Game, p: PlayerId, c: (PlayerId, Sym), base_seed: u64, r: u64) -> f64 {
    let (mut g2, w0) = playout_copy(g, p, base_seed, r);
    g2.forced_attack = Some(c);
    g2.step = Step::Combat;
    let cut = match play_on(&mut g2, p, None) {
        Ok(()) => false,
        Err(Stop::OutOfWork) => true,
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
        Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
    };
    let v = evaluate(&g2, p);
    done(g, &g2, w0, cut);
    v
}

// ------------------------------------------------------------------ counterspells
/// search.choose_counter: q may counter p's spell c (the top of the stack, cast in p's main phase): true to counter,
/// by look-ahead to the end of q's next turn
pub fn choose_counter(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId) -> bool {
    if g.search_tape.0.is_none() {
        return choose_counter_inner(g, q, p, c);
    }
    let (res, replayed) = taped(g, "counter", |g| Taped::Counter(choose_counter_inner(g, q, p, c)));
    if replayed {
        let parts = format!("{}:ctr", g.player(q).key);
        decision_seed(g, &parts);
    }
    matches!(res, Taped::Counter(true))
}

fn choose_counter_inner(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId) -> bool {
    use crate::engine::stack::{cast_counter, counter_side_effects, pick_counter, settle_stack};
    let parts = format!("{}:ctr", g.player(q).key);
    let base_seed = decision_seed(g, &parts);
    let mut scores = [0.0, 0.0]; // [counter, don't]
    let idx = g.stack.len().checked_sub(1); // the spell in question is on top of the stack
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, counter) in [true, false].into_iter().enumerate() {
            let (mut g2, w0) = playout_copy(g, q, base_seed, r);
            let res = (|| -> Res<bool> {
                let it2 = idx.filter(|&i| i < g2.stack.len()).map(|i| g2.stack[i].id);
                if counter {
                    let Some(ctr) = pick_counter(&g2, q, c) else { return Ok(false) };
                    if !cast_counter(&mut g2, q, ctr, Some(c))? {
                        return Ok(false);
                    }
                    counter_side_effects(&mut g2, q, p, ctr)?;
                    if &*g2.db.get(ctr).name == "Mana Drain" {
                        let n = g2.db.get(c).cmc;
                        g2.player_mut(q).drain_mana += n;
                    }
                    if let Some(id) = it2
                        && let Some(it) = g2.stack.iter_mut().find(|it| it.id == id)
                    {
                        it.countered = true;
                    }
                }
                settle_stack(&mut g2)?; // the rest of the stack resolves, no more responses
                if !g2.over {
                    play_on(&mut g2, q, Some(p))?;
                }
                Ok(true)
            })();
            let cut = match res {
                Ok(false) => {
                    scores[k] -= 50.0;
                    continue;
                }
                Ok(true) => false,
                Err(Stop::OutOfWork) => true,
                Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
                Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
            };
            scores[k] += evaluate(&g2, q);
            done(g, &g2, w0, cut);
        }
    }
    scores[0] > scores[1]
}
