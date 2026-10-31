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
use crate::state::{Game, SearchCost, Step};
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

pub(crate) fn stat(f: impl FnOnce(&mut Stats)) {
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
pub(crate) fn decision_seed(g: &mut Game, parts: &str) -> u64 {
    stat(|x| x.decisions += 1);
    g.search_n += 1;
    let name = format!("search:{}:{}:{parts}", g.search_n, g.round);
    Rng::named(&name).next_u64() >> 16
}

// --- phase4-search
/// `f` is one look-ahead decision of p's (of `kind`): its decision and the playout steps it took go to p's
/// `SearchCost` (only read by the reports: play is unchanged)
pub(crate) fn booked<T>(g: &mut Game, p: PlayerId, kind: usize, f: impl FnOnce(&mut Game) -> T) -> T {
    let (n0, w0, k0) = (g.search_n, g.search_work, stats().playouts);
    let r = f(g);
    let (dn, dw) = (g.search_n > n0, g.search_work - w0);
    let c = &mut g.player_mut(p).search_cost;
    c.n[kind] += dn as u32;
    c.steps[kind] += dw;
    c.playouts[kind] += stats().playouts - k0;
    r
}

/// --- phase4-search: the per-decision diagnostic (`AI_SEARCH_DIAG`) is on for p's decisions
fn diag_for(g: &Game, p: PlayerId) -> bool {
    super::ismcts::diag_on() && super::values::generic_deck(g, p)
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
    // --- ai-fix: I: the cards `me` put on top of its own library itself stay there (Vampiric Tutor's card was shuffled
    // away, so no playout saw the line it completes next turn)
    let known = crate::engine::tutors::known_top(g2, me).len();
    for i in 0..g2.players.len() {
        let q = &mut g2.players[i];
        if q.id == me {
            let n = q.library.len() - known; // `known_top` holds only while they are the library's top cards
            rng.shuffle(&mut q.library[..n]);
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
    g2.search_from = g.player(me).turns; // --- ai-fix: E: a win is scored by when it comes
    determinize(&mut g2, me, &mut rng);
    g2.rng = Rng::from_u64(rng.next_u64());
    let w = g2.work;
    (g2, w)
}

// ------------------------------------------------------------------ playing a copy forward
/// search.play_on: finish the active player's turn (p2's by default) from the current step, then everyone's turns
/// until the end of p2's next turn (`horizon` of p2's turns)
pub fn play_on(g2: &mut Game, p2: PlayerId, active: Option<PlayerId>) -> Res {
    play_on_with(g2, p2, active, |_| {})
}

/// `play_on`, calling `turn_end` once the active player's turn (and its extra turns) is over
fn play_on_with(g2: &mut Game, p2: PlayerId, active: Option<PlayerId>, turn_end: impl FnOnce(&Game)) -> Res {
    let act = active.unwrap_or(p2);
    if !g2.stack.is_empty() || !g2.trig_queue.is_empty() {
        crate::engine::stack::settle_stack(g2)?; // copied mid-stack: finish it first
    }
    if g2.player(act).alive && !g2.over {
        let step = g2.step;
        crate::engine::turn::continue_turn(g2, act, step)?;
        crate::engine::turn::run_extra_turns(g2)?;
    }
    turn_end(g2); // --- ai-eval
    play_rounds(g2, p2, act)
}

/// --- phase4-search-b: a copy whose decision came inside a step of the active player's turn (a block, a
/// protection, a wipe response): settle the stack, finish the turn from the step after the one in progress where
/// re-running it would repeat it (the draw, the combat, the end step), then everyone's turns as `play_on`
pub(crate) fn play_on_resume(g2: &mut Game, p2: PlayerId) -> Res {
    let act = g2.active.unwrap_or(p2);
    if !g2.stack.is_empty() || !g2.trig_queue.is_empty() {
        crate::engine::stack::settle_stack(g2)?;
    }
    let next = match g2.step {
        Step::Start | Step::Main1 => Some(Step::Main1),
        Step::Combat | Step::Main2 => Some(Step::Main2),
        Step::End => None,
    };
    if let Some(st) = next
        && g2.player(act).alive
        && !g2.over
    {
        crate::engine::turn::continue_turn(g2, act, st)?;
        crate::engine::turn::run_extra_turns(g2)?;
    }
    play_rounds(g2, p2, act)
}

/// everyone's turns after act's until the end of p2's next turn (`horizon` of p2's turns)
fn play_rounds(g2: &mut Game, p2: PlayerId, act: PlayerId) -> Res {
    const STOP_ROUNDS: u32 = 20;
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
        if g2.over || (q == p2 && g2.settings.search.short) {
            break; // --- ai-eval: a short playout ends with the turn cycle, before p2's next turn (`Search::short`)
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
        return if g.winner == Some(p) { win_score(g, p) } else { -100.0 };
    }
    if !g.player(p).alive {
        return -100.0;
    }
    if g.opps(p).next().is_none() {
        return win_score(g, p);
    }
    if super::learned::on(g, p) {
        return super::learned::value(g, p); // --- phase4-eval: the learned evaluation (`AI_EVAL`)
    }
    95.0 * (raw_term(g, p) / 100.0).tanh()
}

/// `evaluate`'s term before the tanh, for a position short of a win or a loss: p's strength against the strongest
/// opponents (and the mean of them all), plus 12 for each opponent dead
pub fn raw_term(g: &Game, p: PlayerId) -> f64 {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let dead = g.players.iter().filter(|q| q.id != p && !q.alive).count() as f64;
    let s = strength(g, p);
    if opps.is_empty() {
        return s + 12.0 * dead;
    }
    let mut so: Vec<f64> = opps.iter().map(|&q| strength(g, q)).collect();
    so.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    s - 0.6 * so[0] - 0.4 * (so.iter().copied().psum() / so.len() as f64) + 12.0 * dead
}

// --- ai-fix: E
/// what a win loses for each of the winner's own turns it comes after the decision: winning this turn beats a play
/// whose line still wins next turn (both scored 100, and since the sure-win cap the tie went to the higher utility)
pub const WIN_TURN_COST: f64 = 4.0;
/// a win never scores below this: above every position short of a win (`evaluate` keeps those under 95)
pub const WIN_FLOOR: f64 = 96.0;

// --- ai-fix: E
/// a win for p, scored by when it came: 100 on the turn of the decision (or before p's next turn), less
/// `WIN_TURN_COST` for each own turn of p's after it, never below `WIN_FLOOR`. The playout's game ended on p's
/// own-turn count `g.player(p).turns` (frozen once the game is over); `g.search_from` is the count at the decision.
/// Outside the look-ahead a win is 100.
pub fn win_score(g: &Game, p: PlayerId) -> f64 {
    if !g.in_search {
        return 100.0;
    }
    let later = g.player(p).turns.saturating_sub(g.search_from);
    (100.0 - WIN_TURN_COST * later as f64).max(WIN_FLOOR)
}

// --- ai-eval: the board valued for what it will produce
//
// The unit is `strength`'s own: a card in hand is `HAND_W` (0.9). A permanent that makes cards or mana every turn is
// worth what it will make over `OUTPUT_TURNS`, the two own turns after the playout ends (the second at half weight:
// by then it may be gone or the game over), in cards:
//   output_value = HAND_W * (1 + 0.5) * (cards a turn cycle + MANA_CARD * mana a turn)
// One rate converts mana to cards: MANA_CARD = 0.4 / 0.9, a Treasure's value in `strength` (one mana, once) over a
// card's. With it a permanent making one mana a turn is worth 0.6 over the window, what `strength` already gives a
// land, so rocks, dorks and lands share one scale. Cards a turn cycle come from the card itself:
// - `eng=N` (it draws N at upkeep), else the small table `OUTPUT_CARDS`, the rates measured while the engine was out
//   at the Tier 6 table (audit/ai-foundations/step3-flow.md). The tax engines (Rhystic Study, Mystic Remora, Esper
//   Sentinel, Kraum, Faerie Mastermind) draw off opponents' spells: their rate is for three opponents who cast, and
//   scales with the live opponents who can (none in a goldfish, whose seats never cast).
// - mana a turn: `rock=N:...` (N), a mana creature (`dork`: 1), Chrome Mox (1). A rock that doesn't untap (Mana Vault,
//   Grim Monolith) or that is sacrificed for its mana (Lotus Petal, Lion's Eye Diamond) makes nothing past the turn.
// A card whose `pval` already prices its output (Rhystic Study 4, an `eng` card 2, The One Ring 5, Consecrated Sphinx
// 6, Smothering Tithe 4) counts the larger of the two, not both. Everything else adds the output to its `pval`.

/// a card in hand, in `strength`
pub const HAND_W: f64 = 0.9;
/// the weights of the own turns after the playout over which a permanent's output counts
pub const OUTPUT_TURNS: [f64; 2] = [1.0, 0.5];
/// cards one mana is worth: a Treasure's 0.4 over a card's 0.9
pub const MANA_CARD: f64 = 0.4 / HAND_W;

/// cards a turn cycle while out, measured at the Tier 6 table (step3-flow.md), and whether the card draws off
/// opponents' spells (a tax engine: the rate is for three opponents who cast)
pub const OUTPUT_CARDS: [(&str, f64, bool); 9] = [
    ("Rhystic Study", 3.0, true),
    ("Mystic Remora", 2.7, true),
    ("Esper Sentinel", 1.7, true),
    ("Kraum, Ludevic's Opus", 1.2, true),
    ("Faerie Mastermind", 0.7, true),
    ("Necropotence", 4.0, false),
    ("The One Ring", 2.0, false),
    ("Tymna the Weaver", 1.0, false),
    ("Thrasios, Triton Hero", 0.3, false),
];

/// what permanent m will produce in a turn cycle, in cards (mana converted at `MANA_CARD`)
pub fn output_cards(g: &Game, m: crate::ids::PermId) -> f64 {
    let x = g.perm(m);
    let Some(c) = x.cd else { return 0.0 };
    let d = g.db.get(c);
    let t = &d.tags;
    let cards = if let Some(n) = t.int(Tag::Eng)
        && !t.has(Tag::Remora)
    {
        n.max(0) as f64
    } else if let Some(&(_, n, tax)) = OUTPUT_CARDS.iter().find(|e| e.0 == &*d.name) {
        if tax {
            let casters = g.opps(x.owner).filter(|&q| !crate::goldfish::is_inert_key(g.player(q).key)).count();
            n * casters.min(3) as f64 / 3.0
        } else {
            n
        }
    } else {
        0.0
    };
    let one_shot = t.has(Tag::Nountap) || matches!(&*d.name, "Lotus Petal" | "Lion's Eye Diamond");
    let mana = if one_shot {
        0
    } else if let Some(r) = t.str(Tag::Rock) {
        r.split_once(':').map_or(r, |x| x.0).parse().unwrap_or(1)
    } else if t.has(Tag::Dork) || t.has(Tag::Chromemox) {
        1
    } else {
        0
    };
    cards + MANA_CARD * mana as f64
}

/// permanent m's output over the window, in `strength`'s unit
pub fn output_value(g: &Game, m: crate::ids::PermId) -> f64 {
    HAND_W * OUTPUT_TURNS.iter().sum::<f64>() * output_cards(g, m)
}

/// a permanent's value in `strength`: its `pval` plus its output (the larger of the two when `pval` already prices
/// the output)
pub(crate) fn board_value(g: &Game, m: crate::ids::PermId) -> f64 {
    use crate::engine::values::pval;
    let v = pval(g, m);
    if !g.settings.search.output {
        return v;
    }
    let out = output_value(g, m);
    if out == 0.0 {
        return v;
    }
    let t = &g.db.get(g.perm(m).cd.expect("output_value is 0 for a card-less permanent")).tags;
    let priced =
        t.has(Tag::Rhystic) || t.has(Tag::Tithe) || t.has(Tag::Eng) || t.has(Tag::Onering) || t.has(Tag::Sphinx);
    if priced { v.max(out) } else { v + out }
}

/// search.strength
pub fn strength(g: &Game, q: PlayerId) -> f64 {
    use crate::engine::values::epow;
    let pl = g.player(q);
    let life = pl.life.clamp(0, 60) as f64;
    let live = || pl.perms.iter().copied().filter(|&m| !g.perm(m).phased);
    let board = live().map(|m| board_value(g, m)).psum(); // --- ai-eval: was pval
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

// --- phase4-search-b
/// one playout of a decision of me's made inside a step (a block, a protection, a wipe response): `make` makes the
/// option in the copy (false: it can't be made there, -50), then the copy plays on (`play_on_resume`) and is scored
pub(crate) fn playout_with(
    g: &mut Game,
    me: PlayerId,
    base_seed: u64,
    r: u64,
    make: impl FnOnce(&mut Game) -> Res<bool>,
) -> f64 {
    let (mut g2, w0) = playout_copy(g, me, base_seed, r);
    let res = (|| -> Res<bool> {
        if !make(&mut g2)? {
            return Ok(false);
        }
        if !g2.over {
            play_on_resume(&mut g2, me)?;
        }
        Ok(true)
    })();
    let cut = match res {
        Ok(false) => {
            done(g, &g2, w0, false);
            return -50.0;
        }
        Ok(true) => false,
        Err(Stop::OutOfWork) => true,
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
        Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
    };
    let v = evaluate(&g2, me);
    done(g, &g2, w0, cut);
    v
}

// --- ai-eval
/// Does the look-ahead's pick beat the heuristic's top play by more than the noise? `d` holds, per rollout, the
/// pick's value less the top play's (the same seeds, bonuses in, each capped at a win). The pick stands when the mean
/// difference is above `z` standard errors of it. Six rollouts of a Tier 6 table differ by 12-18 points from a win or
/// a loss in one copy, while casting an engine now or a phase later differs by 0-3 (the stop line casts the same card
/// before the horizon in 70-97% of its playouts: eval-fix.md, Part 1), so the bare argmax picked among near-equal
/// lines at random and passed over the heuristic's 95-100% play turn after turn.
pub fn margin_holds(d: &[f64], z: f64) -> bool {
    let n = d.len() as f64;
    if n < 2.0 {
        return d.iter().sum::<f64>() > 0.0;
    }
    let m = d.iter().sum::<f64>() / n;
    let var = d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    m > z * (var / n).sqrt()
}

// --- ai-rollouts
/// The adaptive rollouts' early stop: more rollouts can't be expected to carry the pick past the margin, because its
/// mean gain over the top play is at or below `stop` standard errors of it (a mean at or below 0 always stops).
pub fn hopeless(d: &[f64], stop: f64) -> bool {
    let n = d.len() as f64;
    if n < 2.0 {
        return d.iter().sum::<f64>() <= 0.0;
    }
    let m = d.iter().sum::<f64>() / n;
    let var = d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    m <= 0.0 || m <= stop * (var / n).sqrt()
}

// --- ai-rollouts
/// The pick from per-rollout values (`vals[k][r]`, bonuses in) and each option's deck bonus `ex[k]` (Karn's and
/// Cloud's `search_bonus`, per rollout): (the bare argmax of the capped sums, ties to the earlier option; the pick
/// after the margin against the heuristic's top play `ti`). `choose_inner`'s rule, on any number of rollouts.
pub fn pick_with(vals: &[Vec<f64>], ex: &[f64], ti: usize, z: f64) -> (usize, usize) {
    let s: Vec<f64> = vals
        .iter()
        .zip(ex)
        .map(|(v, e)| (v.iter().sum::<f64>() + e * v.len() as f64).min(100.0 * v.len() as f64))
        .collect();
    let mut b = 0;
    for k in 1..s.len() {
        if s[k] > s[b] {
            b = k;
        }
    }
    let d = paired(vals, ex, b, ti);
    (b, if b != ti && !margin_holds(&d, z) { ti } else { b })
}

/// --- ai-rollouts: per rollout, option a's value less option b's (deck bonuses in, each capped at a win)
fn paired(vals: &[Vec<f64>], ex: &[f64], a: usize, b: usize) -> Vec<f64> {
    vals[a].iter().zip(&vals[b]).map(|(x, y)| (x + ex[a]).min(100.0) - (y + ex[b]).min(100.0)).collect()
}

// --- ai-rollouts
/// The adaptive rollouts' knobs (`Search::adapt_*`, and the margin's z)
#[derive(Debug, Clone, Copy)]
pub struct Adapt {
    pub batch: u32,
    pub cap: u32,
    pub stop: f64,
    pub z: f64,
}

// --- ai-rollouts
/// Adaptive rollouts. `(bare, best)` is the decision on the rollouts in `vals` so far. When the margin hands it back
/// (the bare pick isn't the top play `ti`, and the pick is), `more(vals, r0..r1)` appends rollouts r0..r1 of every
/// option (new seeds, the same for every option), `a.batch` at a time, and the pick is made again on all of them. It
/// stops when the pick stands (the margin holds), when the look-ahead's best is the top play itself, when the
/// pick's gain is `hopeless` (at or below `a.stop` standard errors), or at `a.cap` rollouts per option. Returns the
/// new (bare, best).
pub fn adapt_rollouts(
    vals: &mut Vec<Vec<f64>>,
    ex: &[f64],
    ti: usize,
    (mut bare, mut best): (usize, usize),
    a: &Adapt,
    mut more: impl FnMut(&mut Vec<Vec<f64>>, std::ops::Range<u64>),
) -> (usize, usize) {
    while bare != ti && best == ti && !hopeless(&paired(vals, ex, bare, ti), a.stop) {
        let used = vals[0].len() as u32;
        if used >= a.cap || a.batch == 0 {
            break;
        }
        let next = (used + a.batch).min(a.cap);
        more(vals, used as u64..next as u64);
        (bare, best) = pick_with(vals, ex, ti, a.z);
    }
    (bare, best)
}

/// --- ai-rollouts: rollouts `rs` (seeds (base_seed, r)) of every option, appended to `vals` with their bonuses, as
/// `choose_inner` adds them. `diag`: the Part 1 diagnostic is open, and records these playouts too.
#[allow(clippy::too_many_arguments)]
fn rollout_vals(
    g: &mut Game,
    p: PlayerId,
    post: bool,
    opts: &[Opt],
    keyed: &[(usize, &str, usize)],
    base_seed: u64,
    rs: std::ops::Range<u64>,
    hold: Option<(CardId, f64)>,
    vals: &mut [Vec<f64>],
    diag: bool,
) {
    for r in rs {
        for (k, &(i, label, n)) in keyed.iter().enumerate() {
            if diag {
                super::evdiag::option(k);
            }
            let (v, kept) = playout_main_kept(g, p, post, label, n, base_seed, r, hold.map(|h| h.0));
            let extra = if v < WIN_FLOOR {
                let h = match hold {
                    Some((_, w)) if kept => w,
                    Some(_) => 0.0,
                    None if g.settings.search.hold_kept => 0.0,
                    None => hold_bonus(g, p, &opts[i]),
                };
                ramp_bonus(g, p, &opts[i]) + h
            } else {
                0.0
            };
            vals[k].push(v + extra);
        }
    }
}

/// --- ai-rollouts: each option's deck bonus per rollout (Karn's and Cloud's `search_bonus`; 0 for other decks)
fn deck_extras(g: &Game, p: PlayerId, opts: &[Opt], keyed: &[(usize, &str, usize)]) -> Vec<f64> {
    keyed
        .iter()
        .map(|kk| match g.player(p).key {
            "karn" => crate::impls::karn::search_bonus(g, p, &opts[kk.0]),
            "cloud" => crate::impls::cloud::search_bonus(g, p, &opts[kk.0]),
            _ => 0.0,
        })
        .collect()
}

/// --- ai-rollouts: the knobs of the adaptive rollouts in g's settings (None: off)
fn adapt_of(g: &Game) -> Option<Adapt> {
    let s = &g.settings.search;
    s.adapt.then_some(Adapt { batch: s.adapt_batch, cap: s.adapt_cap, stop: s.adapt_stop, z: margin_z(s) })
}

/// --- ai-rollouts: the margin's z for this decision: `adapt_z` at every look when the adaptive rollouts are on and can
/// look more than once (a test repeated after each batch at z = 2 lets through more noise than one test does),
/// `margin` otherwise (a cap at `rollouts` is one look). 0 stays 0 (the bare argmax).
pub fn margin_z(s: &crate::settings::Search) -> f64 {
    if s.adapt && s.adapt_cap > s.rollouts && s.margin > 0.0 { s.adapt_z } else { s.margin }
}

// --- ai-eval
/// the held card and the bonus (`hold_w` times the heuristic's hold value) each open playout that keeps its mana up
/// through p's turn earns (`Search::hold_kept`); None when nothing is held or `hold_w` is 0
pub fn hold_credit(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    let w = g.settings.search.hold_w;
    if w == 0.0 {
        return None;
    }
    let (c, v) = brain::hold_value(g, p, &brain::Situation::new(g, p));
    c.filter(|_| v > 0.0).map(|c| (c, w * v))
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
        let st = diag_for(g, p).then(|| super::ismcts::stakes(g, p, opts)); // --- phase4-search: stats only
        let w0 = g.search_work;
        let r = booked(g, p, SearchCost::MAIN, |g| choose_inner(g, p, post, opts));
        if let (Some(s), Some(i)) = (st, r) {
            let top = (0..opts.len()).fold(0, |b, k| if opts[k].utility > opts[b].utility { k } else { b });
            super::ismcts::diag(g, p, "main", &s, i != top, g.search_work - w0);
        }
        return r;
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
    if super::ismcts::on(g, p) {
        return super::ismcts::choose_main(g, p, post, opts); // --- phase4-search
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
    // --- ai-fix: E: the playouts that did not end in p's win: the ramp and hold bonuses value what pays off after the
    // horizon, and a playout already won has nothing left to pay off (a held line that wins next turn at 96 plus a big
    // hold value would otherwise tie a win this turn at the cap)
    let mut open = vec![0.0; keyed.len()];
    // --- ai-eval: the Part 1 diagnostic (stats only): a decision whose heuristic top is an engine, a commander or mana
    let diag = if super::evdiag::on() { super::evdiag::top_class(g, p, opts) } else { None };
    if let Some((ti, _)) = diag {
        let top = match opts[ti].act {
            Some(Action::Cast { card, .. }) => Some(card),
            _ => None,
        };
        let hold = brain::hold_value(g, p, &brain::Situation::new(g, p)).0;
        super::evdiag::begin(keyed.len(), top, hold);
    }
    // --- ai-eval: the hold bonus goes to each open playout that ended p's turn with the held card's mana still up,
    // whatever the option: casting an engine that leaves the counter payable keeps the mana up too, and "stop" in main
    // 1 often doesn't (the heuristic taps out in main 2). The "stop (hold mana)" label used to get it every time
    // (Part 1: the label's line kept the mana up in 64-70% of main-1 playouts, a cast's in 60-66%).
    let hold = if g.settings.search.hold_kept { hold_credit(g, p) } else { None };
    let mut held = vec![0.0; keyed.len()];
    // --- ai-eval: each playout's value with its bonuses, for the paired margin test below
    let hold_w1 = hold.map(|h| h.1);
    let mut vals = vec![vec![]; keyed.len()];
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, &(_, label, n)) in keyed.iter().enumerate() {
            if diag.is_some() {
                super::evdiag::option(k); // --- ai-eval
            }
            let (v, kept) = playout_main_kept(g, p, post, label, n, base_seed, r, hold.map(|h| h.0));
            scores[k] += v;
            if v < WIN_FLOOR {
                open[k] += 1.0;
                if kept {
                    held[k] += 1.0; // --- ai-eval
                }
            }
            let extra = if v < WIN_FLOOR {
                let h = match hold_w1 {
                    Some(w) if kept => w,
                    Some(_) => 0.0,
                    None if g.settings.search.hold_kept => 0.0,
                    None => hold_bonus(g, p, &opts[keyed[k].0]),
                };
                ramp_bonus(g, p, &opts[keyed[k].0]) + h
            } else {
                0.0
            };
            vals[k].push(v + extra); // --- ai-eval
        }
    }
    // the hold bonus each option earned in all
    let holds: Vec<f64> = (0..keyed.len())
        .map(|k| match hold {
            Some((_, v)) => v * held[k], // --- ai-eval
            None if g.settings.search.hold_kept => 0.0,
            None => hold_bonus(g, p, &opts[keyed[k].0]) * open[k],
        })
        .collect();
    for (k, kk) in keyed.iter().enumerate() {
        scores[k] += ramp_bonus(g, p, &opts[kk.0]) * open[k];
        scores[k] += holds[k];
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
    let bare = best; // --- ai-eval: the pick before the margin (the noise diagnostic)
    // --- ai-eval: the look-ahead departs from the heuristic's top play only on evidence (`margin_holds`)
    let z = margin_z(&g.settings.search); // --- ai-rollouts: `adapt_z` when adaptive
    let top_u = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    if z > 0.0
        && keyed[best].0 != top_u
        && let Some(ti) = keyed.iter().position(|kk| kk.0 == top_u)
    {
        let extra = |k: usize| match g.player(p).key {
            "karn" => crate::impls::karn::search_bonus(g, p, &opts[keyed[k].0]),
            "cloud" => crate::impls::cloud::search_bonus(g, p, &opts[keyed[k].0]),
            _ => 0.0,
        };
        let (eb, et) = (extra(best), extra(ti));
        let d: Vec<f64> =
            vals[best].iter().zip(&vals[ti]).map(|(a, b)| (a + eb).min(100.0) - (b + et).min(100.0)).collect();
        if !margin_holds(&d, z) {
            best = ti;
        }
    }
    // --- ai-rollouts: a decision the margin hands back gets more rollouts of the same options (`adapt_rollouts`)
    let hb6 = bare != best; // handed back on the first rollouts
    let mut bare = bare;
    if let Some(a) = adapt_of(g)
        && z > 0.0
        && bare != best
        && let Some(ti) = keyed.iter().position(|kk| kk.0 == top_u)
    {
        let ex = deck_extras(g, p, opts, &keyed);
        let on = diag.is_some();
        (bare, best) = adapt_rollouts(&mut vals, &ex, ti, (bare, best), &a, |v, rs| {
            rollout_vals(g, p, post, opts, &keyed, base_seed, rs, hold, v, on)
        });
        let m = vals[0].len() as f64;
        if m > n {
            for (k, v) in vals.iter().enumerate() {
                scores[k] = (v.iter().sum::<f64>() + ex[k] * m).min(100.0 * m) * n / m; // per `n` rollouts, as before
            }
        }
    }
    if super::evdiag::fallback_on() {
        fallback_diag(g, p, post, opts, &keyed, &vals, (bare, best, top_u), base_seed, hold, hb6); // --- ai-eval
    }
    LAST_SCORES.with(|l| {
        *l.borrow_mut() = keyed.iter().zip(&scores).map(|(k, s)| (k.1.to_string(), s / n)).collect();
    });
    let top = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    if let Some((ti, class)) = diag {
        // --- ai-eval
        let rows: Vec<super::evdiag::Row> = keyed
            .iter()
            .enumerate()
            .map(|(k, kk)| super::evdiag::Row {
                label: kk.1,
                utility: opts[kk.0].utility,
                score: scores[k] / n,
                ramp: ramp_bonus(g, p, &opts[kk.0]),
                hold: holds[k] / open[k].max(1.0), // per open playout
            })
            .collect();
        let top_label = opts[ti].label.clone();
        super::evdiag::end(g, p, class, &top_label, best, &rows);
        noise_diag(g, p, post, opts, &keyed, &vals, (bare, best, top_u), base_seed, hold, class);
    }
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
    playout_main_kept(g, p, post, label, n, base_seed, r, None).0
}

/// --- ai-eval: `playout_main`, and whether the playout kept the mana up for `hold` (`kept_up`) when p's turn ended
#[allow(clippy::too_many_arguments)]
pub(crate) fn playout_main_kept(
    g: &mut Game,
    p: PlayerId,
    post: bool,
    label: &str,
    n: usize,
    base_seed: u64,
    r: u64,
    hold: Option<CardId>,
) -> (f64, bool) {
    let (mut g2, w0) = playout_copy(g, p, base_seed, r);
    let mut kept = false;
    let res = (|| -> Res<Option<f64>> {
        let opts = brain::main_options(&mut g2, p, post)?;
        let Some(i) = find(&opts, label, n) else { return Ok(Some(-50.0)) };
        if let Some(a) = &opts[i].act {
            if !act::perform(&mut g2, p, a)? {
                return Ok(Some(-50.0));
            }
            brain::main(&mut g2, p, post)?; // the rest of this phase, heuristically
        }
        play_on_after_phase(&mut g2, p, hold, &mut kept)?;
        Ok(None)
    })();
    let cut = match res {
        Ok(Some(v)) => return (v, false),
        Ok(None) => false,
        Err(Stop::OutOfWork) => true,
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
        Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
    };
    let v = evaluate(&g2, p);
    super::evdiag::playout(&g2, p, v); // --- ai-eval: stats only (nothing unless a diagnosed decision is open)
    done(g, &g2, w0, cut);
    (v, kept)
}

// --- ai-eval
/// p kept the mana up for the held card h: it is still in hand and p can pay for it now
pub fn kept_up(g: &Game, p: PlayerId, h: CardId) -> bool {
    let (cg, cp) = crate::engine::mana::cost_of(g, p, h);
    g.player(p).hand.contains(&h) && crate::engine::mana::can_pay(g, p, cg, &cp, false)
}

/// search.play_on_after_phase: the chosen play is made and the phase finished: go on from the next step. `kept` is
/// set when p's turn ends with the mana up for `hold`.
fn play_on_after_phase(g2: &mut Game, p2: PlayerId, hold: Option<CardId>, kept: &mut bool) -> Res {
    g2.step = match g2.step {
        Step::Main1 => Step::Combat,
        _ => Step::End,
    };
    play_on_with(g2, p2, None, |g| {
        super::evdiag::turn_end(g, p2); // --- ai-eval: the diagnostic reads the turn end
        *kept = hold.is_some_and(|h| kept_up(g, p2, h)); // --- ai-eval
    })
}

// ------------------------------------------------------------------ attacks
/// search.choose_attack: at the first combat of p's turn: (defender, 'filtered' | 'all' | 'none') by look-ahead
pub fn choose_attack(g: &mut Game, p: PlayerId) -> Option<(PlayerId, Sym)> {
    if g.search_tape.0.is_none() {
        let st = diag_for(g, p).then(|| (super::ismcts::stakes(g, p, &[]), brain::choose_defender(&mut clone(g), p)));
        let w0 = g.search_work;
        let r = booked(g, p, SearchCost::ATTACK, |g| choose_attack_inner(g, p));
        if let (Some((s, d)), Some(plan)) = (st, r) {
            super::ismcts::diag(g, p, "attack", &s, plan != (d, "filtered"), g.search_work - w0); // --- phase4-search
        }
        return r;
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

/// the attack plans the look-ahead tries: filtered and all-out at each opponent it may attack, and none (empty: no
/// opponent to attack)
pub(crate) fn attack_cands(g: &mut Game, p: PlayerId) -> Vec<(PlayerId, Sym)> {
    let mut opps: Vec<PlayerId> = g.opps(p).collect();
    if g.player(p).key == crate::impls::garland::KEY {
        opps = crate::impls::garland::attack_targets(g, p, opps); // never the monarch while we hold a steal
    }
    if opps.is_empty() {
        return vec![];
    }
    let mut cands: Vec<(PlayerId, Sym)> = vec![];
    for &q in &opps {
        cands.push((q, "filtered"));
        cands.push((q, "all"));
    }
    cands.push((opps[0], "none"));
    cands
}

fn choose_attack_inner(g: &mut Game, p: PlayerId) -> Option<(PlayerId, Sym)> {
    if super::ismcts::on(g, p) {
        return super::ismcts::choose_attack(g, p); // --- phase4-search
    }
    let cands = attack_cands(g, p);
    if cands.is_empty() {
        return None;
    }
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
        let st = diag_for(g, q).then(|| super::ismcts::stakes(g, q, &[]));
        let w0 = g.search_work;
        let r = booked(g, q, SearchCost::COUNTER, |g| choose_counter_inner(g, q, p, c));
        if let Some(s) = st {
            super::ismcts::diag(g, q, "counter", &s, r, g.search_work - w0); // --- phase4-search: "changed" = countered
        }
        return r;
    }
    let (res, replayed) = taped(g, "counter", |g| Taped::Counter(choose_counter_inner(g, q, p, c)));
    if replayed {
        let parts = format!("{}:ctr", g.player(q).key);
        decision_seed(g, &parts);
    }
    matches!(res, Taped::Counter(true))
}

fn choose_counter_inner(g: &mut Game, q: PlayerId, p: PlayerId, c: CardId) -> bool {
    if super::ismcts::on(g, q) {
        return super::ismcts::choose_counter(g, q, p, c); // --- phase4-search
    }
    let parts = format!("{}:ctr", g.player(q).key);
    let base_seed = decision_seed(g, &parts);
    let mut scores = [0.0, 0.0]; // [counter, don't]
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, counter) in [true, false].into_iter().enumerate() {
            scores[k] += playout_counter(g, q, p, c, counter, base_seed, r);
        }
    }
    scores[0] > scores[1]
}

/// one playout of q's counter decision on p's spell c (the top of the stack): countered or not. The score of the
/// position it reaches (a counter that can't be cast: -50).
pub(crate) fn playout_counter(
    g: &mut Game,
    q: PlayerId,
    p: PlayerId,
    c: CardId,
    counter: bool,
    base_seed: u64,
    r: u64,
) -> f64 {
    use crate::engine::stack::{cast_counter, counter_side_effects, pick_counter, settle_stack};
    let idx = g.stack.len().checked_sub(1); // the spell in question is on top of the stack
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
        Ok(false) => return -50.0,
        Ok(true) => false,
        Err(Stop::OutOfWork) => true,
        Err(Stop::Probe(pr)) => panic!("a trigger probe escaped a playout: {pr:?}"),
        Err(Stop::Cancelled) => panic!("a person was asked inside a playout"),
    };
    let v = evaluate(&g2, q);
    done(g, &g2, w0, cut);
    v
}

// --- ai-eval
/// the pick from per-rollout values (`vals[k][r]`, bonuses in): the bare argmax of the capped sums (ties to the
/// earlier option, the higher utility), and the pick after the margin against option `ti` (the heuristic's top)
fn pick_of(vals: &[Vec<f64>], ti: usize, z: f64) -> (usize, usize) {
    let s: Vec<f64> = vals.iter().map(|v| v.iter().sum::<f64>().min(100.0 * v.len() as f64)).collect();
    let mut b = 0;
    for k in 1..s.len() {
        if s[k] > s[b] {
            b = k;
        }
    }
    let d: Vec<f64> = vals[b].iter().zip(&vals[ti]).map(|(a, t)| a.min(100.0) - t.min(100.0)).collect();
    (b, if b != ti && !margin_holds(&d, z) { ti } else { b })
}

// --- ai-eval
/// Part 2's noise diagnostic (env AI_EVAL_DIAG=1, decisions `evdiag` records): one `EVN` line with the paired
/// top-minus-stop difference (mean, its standard error, the spread of each line), and the picks with and without the
/// margin (z = 1) on this decision's seeds and on a second set of seeds. The second playouts spend no game budget
/// (`search_work` is restored) and read no random numbers of the game, so play is unchanged.
#[allow(clippy::too_many_arguments)]
fn noise_diag(
    g: &mut Game,
    p: PlayerId,
    post: bool,
    opts: &[Opt],
    keyed: &[(usize, &str, usize)],
    vals: &[Vec<f64>],
    (bare, best, top_u): (usize, usize, usize),
    base_seed: u64,
    hold: Option<(CardId, f64)>,
    class: &str,
) {
    let Some(ti) = keyed.iter().position(|kk| kk.0 == top_u) else { return };
    let Some(si) = keyed.iter().position(|kk| opts[kk.0].act.is_none()) else { return };
    let sd = |x: &[f64]| {
        let n = x.len() as f64;
        let m = x.iter().sum::<f64>() / n;
        (m, (x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt())
    };
    let cap = |v: &[f64]| v.iter().map(|x| x.min(100.0)).collect::<Vec<f64>>();
    let d: Vec<f64> = cap(&vals[ti]).iter().zip(cap(&vals[si])).map(|(a, b)| a - b).collect();
    let (md, sdd) = sd(&d);
    let (_, sdt) = sd(&cap(&vals[ti]));
    let (_, sds) = sd(&cap(&vals[si]));
    let (_, m1) = pick_of(vals, ti, 1.0);
    let work = g.search_work;
    let seed2 = base_seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut alt = vec![vec![]; keyed.len()];
    for r in 0..g.settings.search.rollouts as u64 {
        for (k, &(i, label, n)) in keyed.iter().enumerate() {
            let (v, kept) = playout_main_kept(g, p, post, label, n, seed2, r, hold.map(|h| h.0));
            let extra = if v < WIN_FLOOR {
                ramp_bonus(g, p, &opts[i]) + if kept { hold.map_or(0.0, |h| h.1) } else { 0.0 }
            } else {
                0.0
            };
            alt[k].push(v + extra);
        }
    }
    g.search_work = work;
    let (a0, a1) = pick_of(&alt, ti, 1.0);
    let pl = g.player(p);
    let step = if g.step == Step::Main1 { "main1" } else { "main2" };
    eprintln!(
        "EVN\t{}\t{}\t{}\t{class}\t{step}\t{}\t{md:.3}\t{:.3}\t{sdt:.3}\t{sds:.3}\t{sdd:.3}\t{bare}\t{m1}\t{best}\t{a0}\t{a1}\t{ti}\t{si}\t{}",
        super::evdiag::seed(),
        g.search_n,
        pl.key,
        u8::from(hold.is_some()),
        sdd / (d.len() as f64).sqrt(),
        keyed.len(),
    );
}

// --- ai-eval
/// The fallback diagnostic (env AI_EVAL_FALLBACK=1), for every main-phase look-ahead decision: one `EVF` line with
/// the kinds of the heuristic's top play and of the look-ahead's bare pick, whether the margin handed the decision
/// back to the heuristic, the paired standard error of the top play against its best alternative, and whether the
/// bare and the final pick change on a second set of seeds. The second playouts spend no game budget (`search_work`
/// is restored) and read none of the game's random numbers, so play is unchanged.
/// --- ai-rollouts: then the rollouts per option the decision used, whether the margin handed it back on its first
/// `rollouts`, and the rollouts the second set used. Both sets decide by the same rule, adaptive rollouts included,
/// and "bare" is the argmax over all the rollouts a set used.
#[allow(clippy::too_many_arguments)]
fn fallback_diag(
    g: &mut Game,
    p: PlayerId,
    post: bool,
    opts: &[Opt],
    keyed: &[(usize, &str, usize)],
    vals: &[Vec<f64>],
    (bare, best, top_u): (usize, usize, usize),
    base_seed: u64,
    hold: Option<(CardId, f64)>,
    hb6: bool,
) {
    let Some(ti) = keyed.iter().position(|kk| kk.0 == top_u) else { return };
    let z = margin_z(&g.settings.search); // --- ai-rollouts
    let sum = |v: &Vec<f64>| v.iter().map(|x| x.min(100.0)).sum::<f64>();
    let alt = if bare != ti {
        bare
    } else {
        match (0..keyed.len()).filter(|&k| k != ti).max_by(|&a, &b| sum(&vals[a]).total_cmp(&sum(&vals[b]))) {
            Some(k) => k,
            None => return,
        }
    };
    let d: Vec<f64> = vals[ti].iter().zip(&vals[alt]).map(|(a, b)| a.min(100.0) - b.min(100.0)).collect();
    let n = d.len() as f64;
    let m = d.iter().sum::<f64>() / n;
    let se = (d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0).max(1.0) / n).sqrt();
    let work = g.search_work;
    let seed2 = base_seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut alt_vals = vec![vec![]; keyed.len()];
    // --- ai-rollouts: the second set of seeds is decided by the same rule, adaptive rollouts included
    rollout_vals(g, p, post, opts, keyed, seed2, 0..g.settings.search.rollouts as u64, hold, &mut alt_vals, false);
    let ex = deck_extras(g, p, opts, keyed);
    let (mut a0, mut a1) = pick_with(&alt_vals, &ex, ti, z);
    if let Some(a) = adapt_of(g)
        && z > 0.0
    {
        (a0, a1) = adapt_rollouts(&mut alt_vals, &ex, ti, (a0, a1), &a, |v, rs| {
            rollout_vals(g, p, post, opts, keyed, seed2, rs, hold, v, false)
        });
    }
    g.search_work = work;
    let pl = g.player(p);
    eprintln!(
        "EVF\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{m:.3}\t{se:.3}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        super::evdiag::seed(),
        pl.key,
        u8::from(pl.t6),
        if g.step == Step::Main1 { "main1" } else { "main2" },
        super::evdiag::kind(g, p, &opts[top_u]),
        super::evdiag::kind(g, p, &opts[keyed[bare].0]),
        keyed.len(),
        u8::from(bare != ti && best == ti),
        u8::from(bare != a0),
        u8::from(best != a1),
        u8::from(bare == ti),
        u8::from(best == ti),
        vals[0].len(),
        u8::from(hb6),
        alt_vals[0].len(),
    );
}
