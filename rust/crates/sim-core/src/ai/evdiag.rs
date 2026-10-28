//! --- ai-eval: Part 1's diagnostic (stats only, no play change). With env AI_EVAL_DIAG=1, every main-phase
//! look-ahead decision whose heuristic top option is a card engine, a commander cast from the command zone or a mana
//! permanent is written to stderr, one `EVD` line per option the look-ahead tried: the rollout average split into
//! `search::strength`'s terms for the decider and the opponents' term, the hold and ramp bonuses, the share of
//! playouts won, lost or failed, and two facts about the open playouts: whether the top option's card had been cast
//! by the end, and whether the held card's mana was still up when the decider's turn ended. In a traced game
//! (`--trace`) the same goes into the log.
//!
//! Nothing here reads the game's random numbers or changes a choice: it only reads the copies the look-ahead already
//! played, so the games are identical with it on or off.

use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PlayerId};
use crate::state::{Game, Step};
use crate::tag::Tag;
use std::cell::{Cell, RefCell};
use std::sync::OnceLock;

/// is the diagnostic on (env AI_EVAL_DIAG=1)?
pub fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| matches!(std::env::var("AI_EVAL_DIAG").ok().as_deref(), Some("1")))
}

thread_local! {
    static SEED: Cell<u64> = const { Cell::new(0) };
    static CUR: RefCell<Option<Acc>> = const { RefCell::new(None) };
}

/// the seed of the game this thread is playing (set by `run::Engine::play`), to find a decision again with --trace
pub fn set_seed(s: u64) {
    SEED.with(|x| x.set(s));
}

/// the seed `set_seed` recorded
pub fn seed() -> u64 {
    SEED.with(|x| x.get())
}

/// the terms of `search::strength` (decider's side) and of `evaluate` (the opponents' and the dead), as named
pub const TERMS: [&str; 9] = ["life", "board", "power", "hand", "lands", "treasures", "combo", "opp", "dead"];

#[derive(Default, Clone)]
struct OptAcc {
    n: u32,
    win: u32,
    loss: u32,
    open: u32,
    /// open playouts where the top option's card had left the hand or command zone by the end (cast now or later)
    cast: u32,
    /// open playouts where the held card was still in hand and payable when the decider's turn ended
    held: u32,
    eval_open: f64,
    terms: [f64; 9],
}

struct Acc {
    cur: usize,
    top: Option<CardId>,
    hold: Option<CardId>,
    held_now: bool,
    opts: Vec<OptAcc>,
}

/// what kind of play the heuristic's top option is: "commander", "engine", "mana" (None: something else)
pub fn class(g: &Game, p: PlayerId, c: CardId) -> Option<&'static str> {
    let pl = g.player(p);
    if pl.is_commander(c) && !pl.hand.contains(&c) {
        return Some("commander");
    }
    let d = g.db.get(c);
    if super::t6_table::CARD_ENGINES.contains(&&*d.name)
        || d.tag(Tag::Rhystic)
        || d.tag(Tag::Eng)
        || d.tag(Tag::Onering)
        || d.tag(Tag::Sphinx)
        || d.tag(Tag::Necro)
    {
        return Some("engine");
    }
    super::t6_table::mana_permanent(g, c).then_some("mana")
}

/// the heuristic's top option (by utility) and its class, if it is one this diagnostic records
pub fn top_class(g: &Game, p: PlayerId, opts: &[Opt]) -> Option<(usize, &'static str)> {
    let top = (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b });
    let Some(Action::Cast { card, .. }) = opts[top].act else { return None };
    class(g, p, card).map(|k| (top, k))
}

/// a decision starts: k options will be played out; `top` is the heuristic's top card, `hold` the card mana is held
/// for (`brain::hold_value`)
pub fn begin(k: usize, top: Option<CardId>, hold: Option<CardId>) {
    let a = Acc { cur: 0, top, hold, held_now: false, opts: vec![OptAcc::default(); k] };
    CUR.with(|c| *c.borrow_mut() = Some(a));
}

/// the next playouts are of option k
pub fn option(k: usize) {
    CUR.with(|c| {
        if let Some(a) = c.borrow_mut().as_mut() {
            a.cur = k;
            a.held_now = false;
        }
    });
}

/// the decider p's turn has ended in a playout: is the held card still in hand, with its mana up?
pub fn turn_end(g2: &Game, p: PlayerId) {
    CUR.with(|c| {
        let mut b = c.borrow_mut();
        let Some(a) = b.as_mut() else { return };
        a.held_now = a.hold.is_some_and(|h| {
            let (cg, cp) = crate::engine::mana::cost_of(g2, p, h);
            g2.player(p).hand.contains(&h) && crate::engine::mana::can_pay(g2, p, cg, &cp, false)
        });
    });
}

/// a playout of the current option ended at g2, scored v for p
pub fn playout(g2: &Game, p: PlayerId, v: f64) {
    CUR.with(|c| {
        let mut b = c.borrow_mut();
        let Some(a) = b.as_mut() else { return };
        let (top, held_now) = (a.top, a.held_now);
        let o = &mut a.opts[a.cur];
        o.n += 1;
        if v >= super::search::WIN_FLOOR {
            o.win += 1;
        } else if v <= -100.0 {
            o.loss += 1;
        } else {
            o.open += 1;
            o.eval_open += v;
            let pl = g2.player(p);
            if top.is_some_and(|t| !pl.hand.contains(&t) && !pl.cmds_in_zone().any(|x| x == t)) {
                o.cast += 1;
            }
            if held_now {
                o.held += 1;
            }
            let t = terms(g2, p);
            for i in 0..t.len() {
                o.terms[i] += t[i];
            }
        }
    });
}

/// `search::strength`'s terms for p, then `evaluate`'s opponents' and dead terms (raw units, before the tanh)
pub fn terms(g: &Game, p: PlayerId) -> [f64; 9] {
    use super::search::strength;
    use crate::engine::values::{epow, pval};
    let pl = g.player(p);
    let live = || pl.perms.iter().copied().filter(|&m| !g.perm(m).phased);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let mut so: Vec<f64> = opps.iter().map(|&q| strength(g, q)).collect();
    so.sort_by(|a, b| b.total_cmp(a));
    let opp = if so.is_empty() { 0.0 } else { -0.6 * so[0] - 0.4 * so.iter().sum::<f64>() / so.len() as f64 };
    let dead = g.players.iter().filter(|q| q.id != p && !q.alive).count() as f64;
    [
        0.25 * pl.life.clamp(0, 60) as f64,
        live().map(|m| pval(g, m)).sum(),
        0.35 * live().filter(|&m| g.is_creature(m)).map(|m| epow(g, m) as f64).sum::<f64>(),
        0.9 * pl.hand.len() as f64,
        0.6 * pl.lands.len() as f64,
        0.4 * pl.treasures as f64,
        g.settings.search.combo_w * crate::cardcode::combo_progress(g, p),
        opp,
        12.0 * dead,
    ]
}

/// One option's line for `end`.
pub struct Row<'a> {
    pub label: &'a str,
    pub utility: f64,
    pub score: f64,
    pub ramp: f64,
    pub hold: f64,
}

/// the decision is made: write it out (stderr, and the log of a traced game)
pub fn end(g: &mut Game, p: PlayerId, class: &str, top: &str, chosen: usize, rows: &[Row]) {
    let Some(acc) = CUR.with(|c| c.borrow_mut().take()) else { return };
    let pl = g.player(p);
    let step = match g.step {
        Step::Main1 => "main1",
        Step::Main2 => "main2",
        _ => "other",
    };
    let head = format!(
        "EVD\t{}\t{}\t{}\t{}\t{}\t{}\t{step}\t{class}\t{top}\t{}\t{}",
        SEED.with(|s| s.get()),
        g.search_n,
        pl.key,
        u8::from(pl.t6),
        g.round,
        pl.turns,
        rows[chosen].label,
        u8::from(rows[chosen].label != top),
    );
    let mut log = vec![];
    for (k, (r, o)) in rows.iter().zip(&acc.opts).enumerate() {
        let role = if r.label == top {
            "top"
        } else if r.label.starts_with("stop") {
            "stop"
        } else {
            "other"
        };
        let open = o.open.max(1) as f64;
        let t: Vec<String> = o.terms.iter().map(|x| format!("{:.3}", x / open)).collect();
        eprintln!(
            "{head}\t{}\t{role}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{}\t{:.3}\t{:.3}\t{}\t{}",
            r.label,
            u8::from(k == chosen),
            r.utility,
            o.n,
            o.win,
            o.loss,
            o.open,
            (o.n as i64 - (o.win + o.loss + o.open) as i64),
            r.score,
            o.eval_open / open,
            t.join("\t"),
            r.ramp,
            r.hold,
            o.cast,
            o.held,
        );
        if g.log.is_some() {
            let tt: Vec<String> = TERMS.iter().zip(&o.terms).map(|(n, x)| format!("{n} {:.2}", x / open)).collect();
            log.push(format!(
                "        [eval {}{}: u {:.2}, score {:.2}, won {}/{} lost {}, open eval {:.2} ({}), ramp {:.2}, hold \
                 {:.2}, top card cast in {}/{} open, held mana up at turn end in {}]",
                r.label,
                if k == chosen { " (chosen)" } else { "" },
                r.utility,
                r.score,
                o.win,
                o.n,
                o.loss,
                o.eval_open / open,
                tt.join(", "),
                r.ramp,
                r.hold,
                o.cast,
                o.open,
                o.held
            ));
        }
    }
    for l in log {
        crate::glog!(g, "{l}");
    }
}
