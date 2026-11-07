//! The heuristic AI (Python's `ai/brain.py`): probabilistic, board-reading decisions.
//!
//! How a decision is made:
//! 1. read the board (`Situation`): turn, mana, hand, how much damage is pointed at me, who is leading, whether an
//!    opponent is close to a combo, what my opponents probably hold (from public information only);
//! 2. list every legal play with a utility (a score on a ~0-10 scale);
//! 3. turn the utilities into a probability distribution (softmax with a temperature) and sample a play: higher
//!    scores are more likely, not certain;
//! 4. make the play, re-read the board, and repeat until the AI samples "stop here" (hold up mana) or nothing legal
//!    is left.
//!
//! The temperature sets how human the AI is: low is near-optimal and predictable, high is looser. Each deck also has
//! a play style (aggression, caution) that shifts the utilities.

use super::{hand_key, jd_deck}; // --- phase4-hand-off
use super::{act, decks, explain, gumbel_order, is_main, pool, sample, search, sig, style, temp};
use crate::cards::Types;
use crate::engine::altcost;
use crate::engine::cast::{additional_cost, cast_card, castable, on_cast, pact_affordable, parse_cost};
use crate::engine::combat::double_strike;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, mana_units, pay, phyrexian_life, seq_cost, total_mana};
use crate::engine::removal::legal_targets;
use crate::engine::stack::free_counter;
use crate::engine::values::{commander_out, epow, etgh, has, player_hexproof_from, pval, shielded, threat, untargetable};
use crate::engine::zones::{die, draw, max_by, min_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ opponent model
/// brain.prob_holding: P(opponent q holds at least one card matching pred), from public information only: their
/// decklist minus the copies already seen (graveyard, exile, battlefield).
pub fn prob_holding(g: &Game, q: PlayerId, pred: impl Fn(CardId) -> bool) -> f64 {
    let pl = g.player(q);
    if pl.hand.is_empty() {
        return 0.0;
    }
    let mut total = pl.deck_names.iter().filter(|&&c| pred(c)).count();
    for c in pl.commanders() {
        if !pl.deck_names.contains(&c) && pred(c) {
            total += 1;
        }
    }
    let seen = pl.gy.iter().chain(pl.exile.iter()).filter(|&&c| pred(c)).count()
        + pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(&pred)).count();
    let left = total.saturating_sub(seen);
    let unknown = pl.library.len() + pl.hand.len();
    if left == 0 || unknown == 0 {
        return 0.0;
    }
    let d = (left as f64 / unknown as f64).min(1.0);
    1.0 - (1.0 - d).powi(pl.hand.len() as i32)
}

/// brain.open_mana: q's untapped mana (of one colour: all of it if any source makes that colour, else 0)
pub fn open_mana(g: &Game, q: PlayerId, colour: Option<char>) -> u32 {
    let us = mana_units(g, q, false);
    let tot: u32 = us.iter().map(|u| u.amt).sum();
    match colour {
        None => tot,
        Some(ch) if us.iter().any(|u| u.cols.has(ch)) => tot,
        Some(_) => 0,
    }
}

fn is_counter(g: &Game, c: CardId) -> bool {
    g.db.get(c).tag(Tag::Ctr)
}

fn is_instant_removal(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant && d.tag(Tag::Rem)
}

/// brain.counter_risk: P(my next important spell gets countered)
pub fn counter_risk(g: &Game, p: PlayerId) -> f64 {
    let mut miss = 1.0;
    for q in g.opps(p) {
        let ph = prob_holding(g, q, |c| is_counter(g, c));
        if ph == 0.0 {
            continue;
        }
        let up = open_mana(g, q, Some('U'));
        let mut can = if up >= 2 {
            1.0
        } else if up >= 1 {
            0.4
        } else {
            0.0
        };
        if g.player(q).deck_names.iter().any(|&c| is_counter(g, c) && free_counter(g, q, c)) {
            can = f64::max(can, 0.25);
        }
        miss *= 1.0 - ph * can * 0.85;
    }
    1.0 - miss
}

/// brain.removal_risk: P(an opponent answers my best threat at instant speed this cycle)
pub fn removal_risk(g: &Game, p: PlayerId) -> f64 {
    let mut miss = 1.0;
    for q in g.opps(p) {
        let ph = prob_holding(g, q, |c| is_instant_removal(g, c));
        if ph > 0.0 && open_mana(g, q, None) >= 2 {
            miss *= 1.0 - ph * 0.8;
        }
    }
    1.0 - miss
}

// ------------------------------------------------------------------ board reading
/// brain.Situation: the board as one player reads it
#[derive(Debug, Clone)]
pub struct Situation {
    pub turn: u32,
    pub opps: Vec<PlayerId>,
    pub mana: u32,
    pub hand: usize,
    pub lands_in_hand: usize,
    pub threat: Vec<(PlayerId, f64)>,
    pub leader: Option<PlayerId>,
    pub max_threat: f64,
    /// damage the table could swing at me (fliers in full, ground creatures at 0.6, shared among its targets)
    pub incoming: f64,
    /// incoming against my life
    pub danger: f64,
    pub combo_near: bool,
    /// an opponent's commander on the battlefield is a piece of a catalogued combo (Heliod with Walking Ballista, Kinnan,
    /// Urza): the table keeps an answer up, as real tables do, without seeing the hand
    pub cmdr_combo: bool,
    pub ctr_risk: f64,
}

impl Situation {
    pub fn new(g: &Game, p: PlayerId) -> Situation {
        let pl = g.player(p);
        let opps: Vec<PlayerId> = g.opps(p).collect();
        let t6 = crate::impls::t6_plans::is_t6(g, p);
        let threat: Vec<(PlayerId, f64)> = opps
            .iter()
            .map(|&q| (q, threat(g, p, q) + if t6 { super::t6_table::threat_bonus(g, q) } else { 0.0 }))
            .collect();
        let leader = max_by(&threat, |x| x.1).map(|x| x.0);
        let max_threat =
            if threat.is_empty() { 0.0 } else { threat.iter().map(|x| x.1).fold(f64::NEG_INFINITY, f64::max) };
        let mut inc = 0.0;
        for &q in &opps {
            let share = 1.0 / g.opps(q).count().max(1) as f64;
            for &m in &g.player(q).perms {
                let x = g.perm(m);
                if g.is_creature(m) && !x.noatk && !x.phased {
                    let reach = if x.fly { 1.0 } else { 0.6 } * share;
                    let ph = crate::engine::combat::poison_hit(g, m);
                    // --- c3: infect damage is poison: each counter is worth the share of life that ten kill
                    let life_pow = if crate::engine::combat::kw(g, m, "infect") { 0 } else { epow(g, m) };
                    inc += life_pow as f64 * reach;
                    if ph > 0 && crate::engine::repl::poison_ok(g, p) {
                        inc += ph as f64 * pl.life.max(1) as f64 / (10 - pl.poison as i32).max(1) as f64 * reach;
                    }
                }
            }
        }
        let has_q = |q: PlayerId, t: Tag| has(g, q, t);
        let combo_near = t6 && super::t6_table::archenemy(g, p).is_some_and(|a| a.1 >= super::t6_table::NEAR)
            || opps.iter().any(|&q| {
                let k = g.player(q).key;
                k == "sauron" && has_q(q, Tag::Sword) && has_q(q, Tag::Assault)
            });
        let cmdr_combo = opps.iter().any(|&q| {
            g.player(q).perms.iter().any(|&m| {
                let x = g.perm(m);
                x.is_cmd && !x.phased && x.cd.is_some_and(|c| crate::cardcode::is_combo_piece(g, c))
            })
        });
        Situation {
            turn: pl.turns,
            mana: total_mana(g, p, false),
            hand: pl.hand.len(),
            lands_in_hand: pl.hand.iter().filter(|&&c| g.db.get(c).land).count(),
            leader,
            max_threat,
            incoming: inc,
            danger: inc / pl.life.max(1) as f64,
            combo_near,
            cmdr_combo,
            ctr_risk: counter_risk(g, p),
            threat,
            opps,
        }
    }
}

/// brain.HELD: cards worth keeping mana up for (counters that cost mana, protection)
fn held(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    (d.tag(Tag::Ctr) && !d.tag(Tag::Free)) || matches!(d.tags.str(Tag::Prot), Some("hi" | "phase" | "indes" | "blink"))
}

/// removal that can answer a combo piece in response (`combos::interrupted` casts it): an instant, or one with flash
fn instant_removal(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.tags.str(Tag::Rem).is_some() && (d.instant || d.tag(Tag::Flash))
}

/// p can pay what casting c costs now (`cost_of`: taxes such as Thorn of Amethyst included), not only its printed cost
fn can_pay_now(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false)
}

/// brain.hold_value: the card the AI wants to keep mana up for, and how much
pub fn hold_value(g: &Game, p: PlayerId, s: &Situation) -> (Option<CardId>, f64) {
    let pl = g.player(p);
    // --- t6-ai-b: instant removal is held for an opposing commander that is a combo piece, but a Tier 6 table always
    // has Kinnan, Thrasios or Tymna out: a Tier 6 deck reads the archenemy (`combo_near`) alone, as the rest of its
    // `near` does
    let cmdr_removal = if crate::impls::t6_plans::is_t6(g, p) { s.combo_near } else { s.cmdr_combo };
    let held: Vec<CardId> = pl
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            (held(g, c) || (cmdr_removal && instant_removal(g, c))) && !free_counter(g, p, c) && can_pay_now(g, p, c)
        })
        .collect();
    let (mut hc, mut hv) = if held.is_empty() { (None, 0.0) } else { hold_of(g, p, s, &held) };
    // Ad Nauseam is cast at the end of the turn before yours: keep its mana up when it's worth casting
    for &c in &pl.hand {
        // --- c1b-engines: Tier 6: mana is held for Ad Nauseam only while there is room to cast it (four cards under the
        // hand cap): with a full hand it is not cast (`adnaus_cast_now`), and the hold only kept the hand full
        // (--- t6-ai: JD's Sheoldred too: her list plays Ad Nauseam, and no other deck outside Tier 6 does)
        if g.db.get(c).tag(Tag::Adnaus)
            && can_pay_now(g, p, c)
            && (!(crate::impls::t6_plans::is_t6(g, p) || g.player(p).key == "sheoldred")
                || super::plans::adnaus_room(g, p) >= 4)
        {
            let v = super::plans::adnaus_prio(g, p, c) as f64 / 10.0 - 1.0;
            if v > hv {
                (hc, hv) = (Some(c), v);
            }
        }
    }
    (hc, hv)
}

/// t5-hold-b: P(an opponent answers my worthy permanent at instant speed) below which a pool deck keeps no mana up for
/// protection (`removal_risk`: an opponent that may hold instant removal and has two mana open)
const PROT_RISK_MIN: f64 = 0.25;

/// t5-hold-b: a pool deck (Tiers 1-5 outside decks): not one of JD's or Avery's decks (`is_main`, `jd_deck`) and not a
/// Tier 6 deck, whose hold rules stay as they were
fn pool_deck(g: &Game, p: PlayerId) -> bool {
    let k = g.player(p).key;
    !is_main(k) && !jd_deck(k) && !crate::impls::t6_plans::is_t6(g, p) && !g.player(p).t6
}

/// t5-hold-b: what mana kept up for protection alone is worth to a pool deck: nothing (None) unless it has a commander,
/// a combo piece or a plan file's key card on the battlefield to protect and an opponent can plausibly answer it at
/// instant speed (`removal_risk`); then it grows with that risk. The old floor held 4.8 mana on 2% of Tier 5 turns
/// (Miirym mostly) and the protection was used 5% of the time (audit/realism/t5-hold-2026-11-06).
fn protection_hold(g: &Game, p: PlayerId, caution: f64) -> Option<f64> {
    let worth = g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && !g.perm(m).phased && pool::worth_holding_for(g, p, m));
    if !worth {
        return None;
    }
    let risk = removal_risk(g, p);
    (risk >= PROT_RISK_MIN).then(|| 1.0 + 3.0 * caution * risk)
}

/// brain._hold: the cheapest held card, and how much keeping its mana up is worth
fn hold_of(g: &Game, p: PlayerId, s: &Situation, held: &[CardId]) -> (Option<CardId>, f64) {
    let pl = g.player(p);
    let c = min_by(held, |c| g.db.get(c).cmc as f64).unwrap();
    let caution = style(g, pl.key).caution;
    // --- t6-ai: a Tier 6 deck reads the archenemy alone (`combo_near`): a combo-piece commander on the table is every
    // cEDH pod's normal board, not a threat that is close to winning
    let near = if crate::impls::t6_plans::is_t6(g, p) { s.combo_near } else { s.combo_near || s.cmdr_combo };
    let mut v = 1.0 + 3.0 * caution * (s.max_threat / 20.0).min(1.0) + 2.5 * near as i32 as f64;
    if !is_main(hand_key(g, p)) {
        // outside decks: three opponents will cast something worth answering this cycle
        let n = held.iter().filter(|&&x| g.db.get(x).tag(Tag::Ctr)).count();
        let protection_only = held.iter().all(|&x| g.db.get(x).tag(Tag::Prot));
        if n == 0 && protection_only && pool_deck(g, p) {
            // t5-hold-b: protection alone has its own term, not the counter floor (instant removal keeps the floor: it
            // was cast on 32% of the turns it was held, against 5% for protection)
            let Some(pv) = protection_hold(g, p, caution) else { return (None, 0.0) };
            return (Some(c), pv - if s.turn <= 2 { 2.0 } else { 0.0 });
        }
        v = v.max(1.5 + 2.5 * caution * n.min(2) as f64);
    }
    if s.turn <= 2 {
        v -= 2.0;
    }
    (Some(c), v)
}

fn reserve_penalty(g: &Game, p: PlayerId, c: CardId, hold: Option<CardId>, hold_v: f64, near: bool) -> f64 {
    let Some(h) = hold.filter(|&h| h != c) else { return 0.0 };
    let (cg, cp) = seq_cost(g, p, &[c, h]); // c now, the held card after it
    if can_pay(g, p, cg, &cp, false) {
        return 0.0;
    }
    if near && !super::t6_table::is_ramp(g, c) {
        return 8.0f64.max(0.7 * hold_v); // an opponent is close (Tier 6, or a combo commander): no tapping out for value
    }
    0.7 * hold_v
}

// ------------------------------------------------------------------ card utilities
/// brain.draws_cards: a card that draws for its caster
pub fn draws_cards(g: &Game, c: CardId) -> bool {
    use crate::dsl::model::Ability;
    let d = g.db.get(c);
    if d.tag(Tag::Draw) {
        return true;
    }
    d.abilities.iter().any(|a| match a {
        Ability::Spell { effects } | Ability::Triggered { effects, .. } => {
            effects.iter().any(|e| e.kind == "draw" && e.who.as_deref().unwrap_or("you") == "you")
        }
        _ => false,
    })
}

/// brain.card_utility: how much p wants to cast c now (None: not now)
pub fn card_utility(g: &Game, p: PlayerId, s: &Situation, c: CardId) -> Option<f64> {
    let pl = g.player(p);
    let d = g.db.get(c);
    let main = is_main(hand_key(g, p));
    let mut base = decks::deck_prio_f(g, p, c);
    if !main && pl.library.len() < 8 && draws_cards(g, c) {
        return None; // don't draw yourself out
    }
    if !main && d.tag(Tag::Ctr) && !d.creature {
        return None; // counters wait for a spell to counter
    }
    if (d.tag(Tag::Top) || d.tag(Tag::Seal))
        && d.instant
        && g.active == Some(p)
        && !pl.hand.iter().any(|&x| x != c && g.db.get(x).tag(Tag::Draw))
    {
        return None; // a card put on top on your own turn waits a turn: tutor at the end of theirs
    }
    if d.tag(Tag::Adnaus) && !super::plans::adnaus_cast_now(g, p) {
        return None; // Ad Nauseam waits for the end of the turn before yours, with room in the hand
    }
    if base <= 0.0 && crate::impls::t6_plans::prio(g, p, c).is_some() {
        return None; // a Tier 6 deck plan set this card to 0: it waits (no DSL-value fallback)
    }
    // --- ai-fix: C: a ritual the deck has no priority for is worth the play its mana makes possible this turn. Not on
    // a Yawgmoth's Will turn: every spell is storm for the line then, and the old value stands
    if base <= 0.0 && !pl.yawg && crate::engine::mana::ritual_out(g, p, c).is_some() {
        return ritual_utility(g, p, s, c);
    }
    // its own priority said no (or it's kept for responses): keep it
    let hand_written = g.registry.get(c).is_some_and(|i| i.prio.is_some());
    // JD's decks: a 0 from the deck's own priority is deliberate for an instant (held for responses) and for removal,
    // wipes, counters and protection (cast by the response and removal logic); the rest still gets the ability value
    // (a ritual is no answer: Dark Ritual is cast by the storm line, and from the graveyard under Yawgmoth's Will)
    let ritual = d.abilities.iter().any(
        |a| matches!(a, crate::dsl::model::Ability::Spell { effects } if effects.iter().any(|e| e.kind == "add_mana")),
    );
    let held_kind =
        !ritual && (d.instant || d.tag(Tag::Rem) || d.tag(Tag::Wipe) || d.tag(Tag::Ctr) || d.tag(Tag::Prot));
    if base <= 0.0 && d.has_dsl() && !hand_written && !(super::jd_deck(hand_key(g, p)) && held_kind) {
        base = crate::dsl::card_value(g, p, c) * 10.0;
    }
    if base <= 0.0 {
        return None;
    }
    let mut u = base / 10.0; // deck knowledge as a prior (0-9)
    let t = &d.tags;
    if t.has(Tag::Rock) || t.has(Tag::Dork) || t.has(Tag::Lr) {
        if s.lands_in_hand == 0 && s.turn <= 6 && !t.has(Tag::Moxd) {
            u += 1.5; // (Mox Diamond needs a land)
        }
        u -= 0.35 * (s.turn as f64 - 5.0).max(0.0);
    }
    if t.has(Tag::Draw) || t.has(Tag::Eng) {
        if s.hand <= 2 {
            u += 1.2;
        }
        if s.danger > 0.8 {
            u -= 0.8; // no time to durdle
        }
    }
    if d.creature && d.pow >= 2 && s.danger > 0.5 {
        u += 0.8; // bodies to block
    }
    if pl.is_commander(c) && !pl.hand.contains(&c) {
        u -= 0.35 * pl.tax_of(c) as f64; // recasting gets pricier each time (no tax from hand)
        // --- ai-table-fixes (AI_CMDRISK): and into open removal it is the other threats first
        if super::table_fixes::cmdrisk_on() && pl.tax_of(c) > 0 {
            let other = pl.hand.iter().any(|&x| x != c && !g.db.get(x).land && can_pay_now(g, p, x));
            u -= super::table_fixes::cmd_recast_penalty(g, p, c, s.danger, other);
        }
    }
    // --- c3: an outside deck casts its commander the turn it can pay for it, as players do (a first cast: no tax).
    // The sampled order left it for a later turn about two decisions in five (Tergrid 54%, Kinnan 52% of the
    // decisions that offered it chose it), so commanders came down a turn late.
    if !main && pl.is_commander(c) && !pl.hand.contains(&c) && pl.tax_of(c) == 0 && !crate::impls::t6_plans::is_t6(g, p)
    {
        u += COMMANDER_FIRST;
    }
    // --- aragorn: every spell cast with Aragorn out triggers him (3 damage per red spell, a Soldier, a scry, a pump)
    u += crate::impls::aragorn::cast_value(g, p, c);
    if (super::values::bomb_of(g, p, c) >= 5.0 || pl.is_commander(c) || base >= 70.0) && !d.land {
        u *= 1.0 - 0.35 * s.ctr_risk; // walking into open counter mana
    }
    Some(u)
}

/// --- c3: how much more an outside deck wants to cast its commander for the first time (utility points)
pub const COMMANDER_FIRST: f64 = 1.5;

// --- ai-fix: C
/// a spell a ritual makes castable is worth casting the ritual for when it has at least this utility (tutors, the
/// commander and a line count whatever their utility)
pub const RITUAL_GOOD_SPELL: f64 = 5.0;
/// how much less a ritual is worth than the play it makes possible
pub const RITUAL_DISCOUNT: f64 = 0.25;

// --- ai-fix: C
/// A ritual's utility: worth casting when the mana it nets lets p cast something this turn that it can't pay for now:
/// a catalogued line (`combos::line_enabled_by`), a tutor, its commander, or a good spell (`RITUAL_GOOD_SPELL`). It is
/// worth that play's utility less `RITUAL_DISCOUNT`, and nothing (None) otherwise: mana with nothing to spend it on is
/// lost at the end of the step, the reason Cabal Ritual, Rite of Flame and Desperate Ritual had priority 0.
pub fn ritual_utility(g: &Game, p: PlayerId, s: &Situation, c: CardId) -> Option<f64> {
    use crate::engine::mana::{mana_units, plan_pay, ritual_net, units_after_ritual};
    let u = mana_units(g, p, false);
    let after = units_after_ritual(g, p, c, &u)?;
    let mut best = crate::impls::combos::line_enabled_by(g, p, c, &after);
    let pl = g.player(p);
    let mut cands: Vec<(CardId, &str)> = pl
        .hand
        .iter()
        .copied()
        .filter(|&x| x != c && !g.db.get(x).land && ritual_net(g, p, x) <= 0)
        .map(|x| (x, "hand"))
        .collect();
    cands.extend(pl.cmds_in_zone().map(|x| (x, "cmd")));
    for (x, zone) in cands {
        let (xg, xp) = cost_of(g, p, x);
        if plan_pay(g, &u, xg, &xp).is_some() || plan_pay(g, &after, xg, &xp).is_none() || !castable(g, p, x, zone) {
            continue; // payable without the ritual, or not even with it
        }
        let d = g.db.get(x);
        let tutor = [Tag::Tut, Tag::Xtutor, Tag::Seal, Tag::Top].iter().any(|&t| d.tag(t));
        let Some(v) = card_utility(g, p, s, x) else { continue };
        if tutor || pl.is_commander(x) || v >= RITUAL_GOOD_SPELL {
            best = Some(best.map_or(v, |b: f64| b.max(v)));
        }
    }
    best.map(|v| v - RITUAL_DISCOUNT)
}

// ------------------------------------------------------------------ generic executors
/// cards whose casting needs deck-specific choices (targets, modes, X); never cast generically (brain.SPECIAL)
const SPECIAL: [Tag; 7] = [Tag::Rean, Tag::Fill, Tag::Yawg, Tag::Avarice, Tag::Mastery, Tag::Crackle, Tag::Xdrain];
/// graveyard fillers that are plain permanents: cast normally
const CAST_FILL: [&str; 3] = ["stitcher", "tortured", "wayfinder"];
/// a creature worth more than this (pval) isn't sacrificed to pay for a spell
const SPARE_VALUE: f64 = 3.0;

pub(crate) fn special(g: &Game, c: CardId) -> bool {
    SPECIAL.iter().any(|&k| g.db.get(c).tag(k))
}

/// brain.spare_creature: the creature p would sacrifice to pay a spell's cost: the cheapest, never its commander or
/// the Army, and nothing worth more than a utility body; None: wait for a better time
pub fn spare_creature(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.phased && (!x.is_cmd || free_commander(g, m)) && !x.army
        })
        .collect();
    let worth = |m: PermId| if free_commander(g, m) { 0.0 } else { super::values::pval_for(g, p, m) };
    let m = min_by(&cands, worth)?;
    (worth(m) <= SPARE_VALUE).then_some(m)
}

/// --- c1-t6-lines: the creature a planned tutor sacrifices when `spare_creature` finds none worth giving. The line planner
/// chose this tutor (`Player::plan_tutor` is set only around its cast) because the cards it finds finish a line, so any
/// creature will do, the commander too (it is recast for the tax): a pilot one tutor from the win sacrifices Tymna to
/// Diabolic Intent. The cheapest non-commander first.
pub fn planned_fodder(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).plan_tutor?;
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).army)
        .collect();
    let worth = |m: PermId| super::values::pval_for(g, p, m) + if g.perm(m).is_cmd { 100.0 } else { 0.0 };
    min_by(&cands, worth)
}

/// a commander with mana value 0 (Rograkh, Son of Rohgahh): it is recast for the commander tax alone, so real decks
/// sacrifice it freely (Diabolic Intent's fodder in Rograkh/Silas lists), where a costed commander stays protected
fn free_commander(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.is_cmd && x.cd.is_some_and(|c| g.db.get(c).cmc == 0)
}

/// brain.do_cast: p casts c (from hand or the command zone; zone 'gy': its flashback; 'yawg': from the graveyard
/// under Yawgmoth's Will). False if it can't now.
pub fn do_cast(g: &mut Game, p: PlayerId, c: CardId, zone: Option<Sym>) -> Res<bool> {
    do_cast_with(g, p, c, zone, Ctx::default())
}

/// do_cast with the spell's choices made as it is cast (a bite names its creature and target here)
pub fn do_cast_with(g: &mut Game, p: PlayerId, c: CardId, zone: Option<Sym>, ctx0: Ctx) -> Res<bool> {
    let db = g.db.clone();
    let d = db.get(c);
    let main = is_main(hand_key(g, p));
    // Seph FS has no reanimation plan of its own: it casts a hand-tagged reanimation spell (Dread Return) like any
    // other by its priority, and the spell resolves through seph_rean_resolve (the best creature card)
    let sephfs_rean = g.player(p).key == "sephfs" && d.tag(Tag::Rean);
    if special(g, c)
        && !d.has_dsl()
        && !(d.creature && !main)
        && !d.tags.str(Tag::Fill).is_some_and(|f| CAST_FILL.contains(&f))
        && !sephfs_rean
    {
        return Ok(false);
    }
    if d.has_dsl() && !additional_cost(g, p, c, true)? {
        return Ok(false);
    }
    let pl = g.player(p);
    let in_hand = pl.hand.contains(&c);
    if matches!(zone, None | Some("hand")) && !in_hand && !pl.cmd_zone_of(c) {
        return Ok(false);
    }
    if zone == Some("yawg") && !pl.gy.contains(&c) {
        return Ok(false);
    }
    let zone: Sym = zone.unwrap_or(if pl.is_commander(c) && !in_hand { "cmd" } else { "hand" });
    let cast_zone: Sym = if zone == "yawg" { "gy" } else { zone };
    if g.cast_checks() && !castable(g, p, c, cast_zone) {
        return Ok(false);
    }
    let cv = d.tag(Tag::Convoke);
    let mut fodder = None;
    if d.tag(Tag::Needsac) {
        // Diabolic Intent: sacrifice a creature as an additional cost
        fodder = spare_creature(g, p).or_else(|| planned_fodder(g, p));
        if fodder.is_none() {
            return Ok(false);
        }
    }
    let fblife = d.tags.int(Tag::Fblife);
    let (cg, cp) = if zone == "gy" {
        if let Some(l) = fblife
            && g.player(p).life <= l + 5
        {
            return Ok(false);
        }
        parse_cost(d.tags.str(Tag::Fb).unwrap_or(""))
    } else {
        cost_of(g, p, c)
    };
    g.pay_for = Some(c);
    // --- t6-xtutor: the planner pays X = its target's mana value (set only around its cast), not all the spare mana;
    // the X is paid with the cost
    let plan_x = g.player(p).plan_x.filter(|_| d.tag(Tag::Xtutor)).unwrap_or(0);
    let ok = can_pay(g, p, cg + plan_x, &cp, cv);
    let paid = if ok { pay(g, p, cg + plan_x, &cp, cv) } else { Ok(false) };
    g.pay_for = None;
    if !ok || !paid? {
        return Ok(false);
    }
    let mut ctx = ctx0;
    if d.tag(Tag::Tokx) || d.tag(Tag::Xtutor) {
        // X spells of outside decks: X = all spare mana
        let x = match g.player(p).plan_x.filter(|_| d.tag(Tag::Xtutor)) {
            Some(x) => x, // --- t6-xtutor: paid with the cost
            None => {
                let x = total_mana(g, p, cv);
                pay(g, p, x, "", cv)?;
                x
            }
        };
        ctx.x = x as i32;
        if d.tag(Tag::Xtutor) {
            g.last_x = x as i32;
        }
    }
    if d.has_dsl() {
        additional_cost(g, p, c, false)?;
    }
    if zone == "gy"
        && let Some(l) = fblife
    {
        lose_life(g, p, l, Some(p), "life", None)?;
    }
    if matches!(zone, "hand" | "cmd") && !crate::impls::krrik::life_pips(g, p, c).is_empty() {
        // Phyrexian symbols paid with 2 life each
        let life = phyrexian_life(g, p, c, &cp);
        if life > 0 {
            lose_life(g, p, life, Some(p), "life", None)?;
        }
    }
    if let Some(f) = fodder {
        if !g.perm(f).on_bf || g.perm(f).owner != p {
            return Ok(false);
        }
        die(g, f, "sac")?;
    }
    cast_card(g, p, c, cast_zone, ctx)?; // from the graveyard: exiled after
    Ok(true)
}

fn swamp_land(g: &Game, p: PlayerId) -> bool {
    g.player(p).lands.iter().any(|&l| {
        let d = g.db.get(g.perm(l).card());
        d.has_subtype("swamp") || &*d.name == "Swamp"
    })
}

/// sacrifice fodder for removal costs: creatures that are tokens or no bomb
fn rem_fodder(g: &Game, p: PlayerId) -> Vec<PermId> {
    let fodder_max = if super::values::on(g, p) { super::values::BOMB } else { f64::MIN_POSITIVE }; // --- phase4-values
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && (x.token || x.cd.is_none_or(|cd| super::values::bomb_of(g, p, cd) < fodder_max))
        })
        .collect()
}

/// a removal spell castable for free (Fierce Guardianship-style with your commander out; Snuff Out for 4 life)
/// what a removal spell costs: (generic, pips, life), from `cost_of` (taxes included). For a card with Phyrexian symbols
/// (Dismember) `cost_of` drops a {X/P} pip the mana can't pay, and the life is 2 per dropped pip.
fn rem_cost(g: &Game, p: PlayerId, c: CardId) -> (u32, String, i32) {
    let (gn, pips) = cost_of(g, p, c);
    if g.db.get(c).phyrexian.is_empty() {
        return (gn, pips, 0);
    }
    let life = phyrexian_life(g, p, c, &pips);
    (gn, pips, life)
}

fn free_removal(g: &Game, p: PlayerId, c: CardId) -> bool {
    let t = &g.db.get(c).tags;
    ((t.has(Tag::Freecmd) && commander_out(g, p)) || (t.has(Tag::Snuff) && g.player(p).life > 12 && swamp_land(g, p)))
        && altcost::tax_payable(g, p, c) // a free cast still pays the cost increases (Defense Grid)
}

fn rem_spec(g: &Game, c: CardId) -> (&str, &str, bool) {
    let t = &g.db.get(c).tags;
    (t.str(Tag::Rem).unwrap_or(""), t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4))
}

/// the cards p may cast as a spell now, with the zone: the hand, and the top card of the library under a Mystic Forge
/// (an artifact or colourless card), which the removal and wipe options and their executors treat like a hand card
fn spell_sources(g: &Game, p: PlayerId) -> Vec<(CardId, &'static str)> {
    let mut v: Vec<(CardId, &'static str)> = g.player(p).hand.iter().map(|&c| (c, "hand")).collect();
    if let Some(t) = crate::impls::karn_engines::forge_top_card(g, p) {
        v.push((t, "lib"));
    }
    v
}

/// where c is cast from for removal / wipe code: the hand, or the top of the library under a Mystic Forge
fn source_zone(g: &Game, p: PlayerId, c: CardId) -> Option<&'static str> {
    if g.player(p).hand.contains(&c) {
        Some("hand")
    } else {
        (crate::impls::karn_engines::forge_top_card(g, p) == Some(c)).then_some("lib")
    }
}

/// brain.removal_options
pub fn removal_options(g: &Game, p: PlayerId, s: &Situation) -> Vec<Opt> {
    let mut out = vec![];
    let main = is_main(hand_key(g, p));
    let caution = style(g, g.player(p).key).caution;
    for (c, zone) in spell_sources(g, p) {
        let d = g.db.get(c);
        let t = &d.tags;
        if !t.has(Tag::Rem) || d.creature {
            continue;
        }
        if g.cast_checks() && !castable(g, p, c, zone) {
            continue; // a lock, or Vexing Bauble / Chalice of the Void: cast_removal would refuse it
        }
        let fods = rem_fodder(g, p);
        let extra = if t.has(Tag::Sacor4) && fods.is_empty() { 4 } else { 0 };
        let (rg, rp, _) = rem_cost(g, p, c);
        // an alternative cost (Snapback, Pyrokinesis: cards exiled from hand) when the mana can't be paid
        let free = free_removal(g, p, c);
        let alt = !free && zone == "hand" && altcost::use_alt(g, p, c, None, rg + extra, &rp);
        if !free && !alt && !can_pay(g, p, rg + extra, &rp, t.has(Tag::Convoke)) {
            continue;
        }
        if t.has(Tag::Needart3)
            && g.player(p)
                .perms
                .iter()
                .filter(|&&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).types.has(Types::ARTIFACT)))
                .count()
                < 3
        {
            continue;
        }
        let (rk, tgt, mv4) = rem_spec(g, c);
        let mut tg = legal_targets(g, p, rk, tgt, mv4, Some(c));
        if !main && s.combo_near && crate::impls::t6_plans::is_t6(g, p) {
            // Tier 6: with an opponent close to winning, the answer goes to its pieces
            let on_arch: Vec<PermId> = tg.iter().copied().filter(|&m| Some(g.perm(m).owner) == s.leader).collect();
            if !on_arch.is_empty() {
                tg = on_arch;
            }
        }
        let Some(best) = max_by(&tg, |m| super::values::pval_for(g, p, m)) else { continue };
        let mut v = super::values::pval_for(g, p, best);
        if Some(g.perm(best).owner) == s.leader {
            v *= 1.25;
        }
        if !main && crate::impls::t6_plans::is_t6(g, p) {
            if s.combo_near && Some(g.perm(best).owner) == s.leader {
                v += 3.0; // Tier 6: answers are spent on the archenemy's engine
            } else if d.instant && !s.combo_near && v < 4.5 {
                continue; // Tier 6: held for the win attempt, not fired at the first creature
            }
        }
        if s.combo_near && super::values::pval_for(g, p, best) >= 8.0 {
            v += 2.0;
        }
        if !main && v < 2.5 {
            continue; // outside decks: no removal on trivial targets
        }
        let mut u = v - 3.5;
        if alt {
            u -= 1.5 * altcost::lost(g, p, c, None); // cards given up instead of mana
        }
        if d.instant {
            u -= 1.2 * caution * g.settings.instant_extra as f64 / 2.0; // instants are worth holding
        }
        if t.has(Tag::Needsac) && fods.is_empty() {
            continue;
        }
        if let Some(pc) = t.str(Tag::Pactpay)
            && !pact_affordable(g, p, pc)
        {
            continue; // Slaughter Pact: or lose the game
        }
        out.push(Opt {
            utility: u,
            label: format!("{} -> {}", d.name, g.perm(best).name),
            act: Some(Action::Removal { card: c, targets: tg.clone(), kick: 0, kind: None }),
        });
        if let Some(k) = t.int(Tag::Kick)
            && rk.starts_with("dmg")
            && can_pay(g, p, rg + k as u32, &rp, false)
        {
            let kd = format!("dmg{}", rk[3..].parse::<i32>().unwrap_or(0) + 2); // Burst Lightning kicked: 4 damage
            let tg4 = legal_targets(g, p, &kd, tgt, mv4, Some(c));
            if let Some(b4) = max_by(&tg4, |m| super::values::pval_for(g, p, m))
                && super::values::pval_for(g, p, b4) > v + 1.0
            {
                out.push(Opt {
                    utility: super::values::pval_for(g, p, b4) - 4.5,
                    label: format!("{} (kicked) -> {}", d.name, g.perm(b4).name),
                    act: Some(Action::Removal { card: c, targets: tg4, kick: k as u32, kind: Some(intern(&kd)) }),
                });
            }
        }
    }
    out
}

/// brain.attack_response: d (the AI) is attacked: with priority in the declare attackers step, kill an attacker with
/// instant removal when it's worth the card (a valuable creature, or the attack is dangerous)
pub fn attack_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    // --- generic-paths: Mishra's Factory and Foundry animate to block a dangerous attack (any deck)
    crate::impls::karn::block_response(g, d, p, atk)?;
    if crate::impls::titania::is_deck(g.player(d).key) && crate::impls::titania::attack_fog(g, d, p, atk)? {
        return Ok(true);
    }
    if crate::impls::t3_marwyn::ambush_response(g, d, p, atk)? {
        return Ok(true); // --- Tier 3 (marwyn): Galadhrim Ambush
    }
    let gand = has(g, d, Tag::Gandalf); // Gandalf: sorceries at instant speed
    if gand && gandalf_wipe_response(g, d, p, atk)? {
        return Ok(true);
    }
    let rem: Vec<CardId> = g
        .player(d)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let x = g.db.get(c);
            (x.instant || (gand && x.sorcery))
                && x.tag(Tag::Rem)
                && !x.creature
                && !(g.cast_checks() && !castable(g, d, c, "hand"))
        })
        .collect();
    // --- generic-paths: the answers d's permanents give (Ballista Squad, Lawbringer, Lightbringer)
    let answers = crate::impls::galadriel::attack_answers(g, d, p, atk);
    if (rem.is_empty() && answers.is_empty()) || atk.is_empty() {
        return Ok(false);
    }
    let incoming: i32 = atk
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.perm(m).owner == p)
        .map(|&m| epow(g, m) * if double_strike(g, m) { 2 } else { 1 })
        .sum();
    let dl = g.player(d).life;
    let mine: Vec<PermId> = atk.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).owner == p).collect();
    let danger = incoming as f64 >= dl as f64 * 0.5
        || crate::engine::combat::poison_share(g, d, &mine) >= 0.5 // --- c3: infect and toxic: ten counters kill
        || atk.iter().any(|&m| g.perm(m).is_cmd && crate::engine::values::cmd_dmg_by(g, d, p, m) + epow(g, m) >= 21);
    let mut best: Option<(f64, CardId, PermId)> = None;
    for &c in &rem {
        let x = g.db.get(c);
        let (xg, xp, _) = rem_cost(g, d, c);
        let alt = altcost::use_alt(g, d, c, None, xg, &xp);
        if !alt && !can_pay(g, d, xg, &xp, x.tag(Tag::Convoke)) {
            continue;
        }
        if x.tag(Tag::Needsac) || x.tag(Tag::Pactpay) {
            continue;
        }
        let (rk, tgt, mv4) = rem_spec(g, c);
        for m in legal_targets(g, d, rk, tgt, mv4, Some(c)) {
            if !atk.contains(&m) {
                continue;
            }
            let pen = if alt { 1.5 * altcost::lost(g, d, c, None) } else { 0.0 };
            let v = pval(g, m) + if danger { 3.0 } else { 0.0 } + 0.3 * epow(g, m) as f64 - pen;
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, c, m));
            }
        }
    }
    // --- generic-paths: a permanent's answer, when it is worth it and about as good as the best removal spell
    if let Some(&(v2, a)) = answers.iter().fold(None, |b: Option<&(f64, _)>, x| match b {
        Some(y) if y.0 >= x.0 => Some(y),
        _ => Some(x),
    }) && v2 + if danger { 3.0 } else { 0.0 } - 3.0 > 0.0
        && best.is_none_or(|b| v2 >= b.0 - 1.0)
        && crate::impls::galadriel::answer(g, d, a)?
    {
        g.player_mut(d).stat("attack_removal", 1);
        return Ok(true);
    }
    let Some((v, c, m)) = best else { return Ok(false) };
    let caution = style(g, g.player(d).key).caution;
    if v - 3.5 - 1.2 * caution * g.settings.instant_extra as f64 / 2.0 <= 0.0 {
        return Ok(false);
    }
    crate::glog!(g, "  {} answers the attack: {} -> {}", g.player(d).name, g.db.get(c).name, g.perm(m).name);
    if !cast_removal(g, d, c, &[m], 0, None)? {
        return Ok(false);
    }
    g.player_mut(d).stat("attack_removal", 1);
    Ok(true)
}

/// can_pay with spell c being paid for (Karn's mana is not for nonartifact spells: `Game::pay_for`)
fn can_pay_for(g: &mut Game, p: PlayerId, c: CardId, gn: u32, pips: &str, cv: bool) -> bool {
    g.pay_for = Some(c);
    let ok = can_pay(g, p, gn, pips, cv);
    g.pay_for = None;
    ok
}

fn pay_for(g: &mut Game, p: PlayerId, c: CardId, gn: u32, pips: &str, cv: bool) -> Res<bool> {
    g.pay_for = Some(c);
    let r = pay(g, p, gn, pips, cv);
    g.pay_for = None;
    r
}

/// brain.cast_removal: p casts removal spell c at one of tg (sampled by value), kicked for `kick` more
pub fn cast_removal(g: &mut Game, p: PlayerId, c: CardId, tg: &[PermId], kick: u32, kind: Option<Sym>) -> Res<bool> {
    let db = g.db.clone();
    let d = db.get(c);
    let cv = d.tag(Tag::Convoke);
    let fods = rem_fodder(g, p);
    let mut extra = if d.tag(Tag::Sacor4) && fods.is_empty() { 4 } else { 0 };
    let free = free_removal(g, p, c);
    extra += kick;
    let (rg, rp, rlife) = rem_cost(g, p, c);
    let Some(zone) = source_zone(g, p, c) else { return Ok(false) };
    let alt = !free && kick == 0 && zone == "hand" && altcost::use_alt(g, p, c, None, rg + extra, &rp);
    if !free && !alt && !can_pay_for(g, p, c, rg + extra, &rp, cv) {
        return Ok(false);
    }
    if g.cast_checks() && !castable(g, p, c, zone) {
        return Ok(false);
    }
    let live: Vec<(f64, PermId)> = tg
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !untargetable(g, m))
        .map(|m| (super::values::pval_for(g, p, m) / 1.5, m))
        .collect();
    if live.is_empty() {
        return Ok(false);
    }
    let t = temp(g, p);
    let target = sample(&mut g.rng, &live, t).unwrap();
    let mut fod = None;
    if d.tag(Tag::Needsac) || (d.tag(Tag::Sacor4) && !fods.is_empty()) {
        if fods.is_empty() {
            return Ok(false);
        }
        fod = min_by(&fods, |x| sac_worth(g, x));
    }
    if alt {
        if !altcost::pay_alt(g, p, c, None)? {
            return Ok(false);
        }
    } else if free {
        if !altcost::pay_tax(g, p, c)? {
            return Ok(false);
        }
        if d.tag(Tag::Snuff) && !(d.tag(Tag::Freecmd) && commander_out(g, p)) {
            lose_life(g, p, 4, Some(p), "life", None)?;
        }
    } else {
        pay_for(g, p, c, rg + extra, &rp, cv)?;
        if rlife > 0 {
            lose_life(g, p, rlife, Some(p), "life", None)?; // {B/P} paid with 2 life (Dismember)
        }
    }
    if let Some(f) = fod {
        die(g, f, "sac")?;
    }
    if zone == "lib" {
        g.player_mut(p).library.pop(); // Mystic Forge: cast from the top
    }
    cast_card(g, p, c, zone, Ctx { target: Some(target), rem_kind: kind, ..Ctx::default() })?;
    g.player_mut(p).stat("removal_cast", 1);
    Ok(true)
}

/// brain.wipe_options. eot: the end of the turn before yours, where only instant-speed wipes (Cyclonic Rift's
/// overload) are offered; in the main phase an instant-speed wipe is held for that window unless the danger is now.
pub fn wipe_options(g: &Game, p: PlayerId, s: &Situation, eot: bool) -> Vec<Opt> {
    let mut out = vec![];
    for (c, zone) in spell_sources(g, p) {
        let d = g.db.get(c);
        let Some(kind) = d.tags.str(Tag::Wipe) else { continue };
        if eot
            && !d.instant
            && !(d.sorcery && (has(g, p, Tag::Gandalf) || crate::impls::karn_cards::liberator_flash(g, p, c)))
        // (Liberator: All Is Dust)
        {
            continue;
        }
        let (cg, cp) = decks::wipe_cost(g, p, c);
        let alt = zone == "hand" && altcost::use_alt(g, p, c, None, cg, &cp); // Sunscour: two white cards instead of {5}{W}{W}
        if (!alt && !can_pay(g, p, cg, &cp, false)) || (g.cast_checks() && !castable(g, p, c, zone)) {
            continue;
        }
        let (ol, ml, victim) = decks::wipe_eval(g, p, kind);
        if ol <= 0.0 {
            continue; // hits nothing of theirs (Nibelheim Aflame with no creature of ours)
        }
        // --- ai-table-fixes (AI_WIPEVALUE): a wipe that costs the caster as much as the table is held unless it is
        // about to lose
        if super::table_fixes::wipevalue_on() && !matches!(kind, "rift" | "rebuke" | "vandal") {
            let theirs: Vec<f64> =
                g.opps(p).filter(|&q| g.player(q).alive).map(|q| decks::wipe_loss(g, q, kind, p)).collect();
            if super::table_fixes::wipe_blocked(decks::wipe_loss(g, p, kind, p), &theirs, s.danger) {
                continue;
            }
        }
        let mut u = (ol - 1.2 * ml) / 2.2 - 2.5 + 2.0 * s.danger.min(1.5);
        if g.player(p).key == "sephfs" {
            match crate::impls::sephfs::wipe_utility(g, p, c, kind, s.danger) {
                Some(v) => u = v,
                None => continue,
            }
        }
        if alt {
            u -= 1.5 * altcost::lost(g, p, c, None); // cards given up instead of mana
        }
        if d.instant && !eot && s.danger < 0.8 {
            u -= 1.5; // held: at the end of the turn before yours it sticks
        }
        out.push(Opt { utility: u, label: d.name.to_string(), act: Some(Action::Wipe { card: c, victim }) });
    }
    out
}

/// the executor of a wipe option
pub fn cast_wipe(g: &mut Game, p: PlayerId, c: CardId, victim: Option<PlayerId>) -> Res<bool> {
    let (cg, cp) = decks::wipe_cost(g, p, c);
    let Some(zone) = source_zone(g, p, c) else { return Ok(false) };
    let alt = zone == "hand" && altcost::use_alt(g, p, c, None, cg, &cp);
    if !alt && !can_pay_for(g, p, c, cg, &cp, false) {
        return Ok(false);
    }
    if g.cast_checks() && !castable(g, p, c, zone) {
        return Ok(false);
    }
    if alt {
        if !altcost::pay_alt(g, p, c, None)? {
            return Ok(false);
        }
    } else {
        pay_for(g, p, c, cg, &cp, false)?;
    }
    if zone == "lib" {
        g.player_mut(p).library.pop(); // Mystic Forge: cast from the top
    }
    cast_card(g, p, c, zone, Ctx { victim, ..Ctx::default() })?;
    g.player_mut(p).stat("wipes_cast", 1);
    Ok(true)
}

// ------------------------------------------------------------------ deck-specific plays
/// brain.special_options: the plays your decks' own plans add. Only legal plays are listed, so the distribution is
/// over real choices.
pub fn special_options(g: &mut Game, p: PlayerId, _s: &Situation, post: bool) -> Res<Vec<Opt>> {
    match hand_key(g, p) {
        "omnath" => crate::impls::titania::titania_special(g, p, post),
        "kraum" => crate::impls::tymna_cards::kraum_special(g, p, post),
        "sephfs" => crate::impls::sephfs::options(g, p, post),
        _ => Ok(vec![]),
    }
}

/// brain.extra_options: Disruptor Flute, flashback, Crackle with Power, burn to the face, Clues
pub fn extra_options(g: &mut Game, p: PlayerId, s: &Situation, post: bool, sorcery_ok: bool) -> Vec<Opt> {
    let mut o = vec![];
    // Disruptor Flute (flash): worth casting when there's a name that really hurts an opponent
    for &c in &g.player(p).hand {
        if !g.db.get(c).tag(Tag::Flute) {
            continue;
        }
        let (cg, cp) = cost_of(g, p, c);
        if !can_pay(g, p, cg, &cp, false) {
            continue;
        }
        if let Some((name, score)) = decks::flute_pick(g, p)
            && score >= 3.0
        {
            o.push(Opt {
                utility: score * 0.8 - 1.0,
                label: format!("Disruptor Flute (naming {})", g.db.get(name).name),
                act: Some(Action::Cast { card: c, zone: None }),
            });
        }
        break;
    }
    // flashback from the graveyard
    for c in g.player(p).gy.clone() {
        let d = g.db.get(c);
        let Some(fb) = d.tags.str(Tag::Fb) else { continue };
        if d.sorcery && !sorcery_ok {
            continue;
        }
        let (cg, cp) = parse_cost(fb);
        if !can_pay(g, p, cg, &cp, false) {
            continue;
        }
        let mut u = card_utility(g, p, s, c);
        if d.tag(Tag::Fbnib) {
            let (ol, ml, _) = decks::wipe_eval(g, p, "nib");
            u = Some((ol - 1.2 * ml) / 2.2 - 1.0 + 1.5);
        }
        let Some(u) = u else { continue };
        o.push(Opt {
            utility: u - 0.5,
            label: format!("{} (flashback)", d.name),
            act: Some(Action::Cast { card: c, zone: Some("gy") }),
        });
    }
    // Crackle with Power: X = (mana - 2) / 3
    for &c in &g.player(p).hand {
        if !g.db.get(c).tag(Tag::Crackle) || !(sorcery_ok || has(g, p, Tag::Gandalf)) {
            continue;
        }
        let x = (total_mana(g, p, false) as i32 - 2).div_euclid(3);
        if x < 1 {
            continue;
        }
        let lethal = s.opps.iter().filter(|&&q| g.player(q).life <= 5 * x).count();
        o.push(Opt {
            utility: 2.0 + 3.0 * lethal as f64 + x as f64,
            label: format!("Crackle with Power X={x}"),
            act: Some(Action::Crackle { card: c, x: x as u32 }),
        });
    }
    // burn to the face when it kills
    let thor = has(g, p, Tag::Thor) as i32;
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        let r = d.tags.str(Tag::Rem).unwrap_or("");
        if !d.tag(Tag::Face)
            || !r.starts_with("dmg")
            || !can_pay_now(g, p, c)
            || (g.cast_checks() && !castable(g, p, c, "hand"))
        {
            continue;
        }
        let dmg = r[3..].parse::<i32>().unwrap_or(0) + thor;
        let kick = match d.tags.int(Tag::Kick) {
            Some(k)
                if {
                    let (cg, cp) = cost_of(g, p, c);
                    can_pay(g, p, cg + k as u32, &cp, false)
                } =>
            {
                k as u32
            }
            _ => 0,
        };
        // --- aragorn: with Aragorn out a red burn spell also triggers his 3 damage (to the lowest-life opponent) and
        // Fiery Emancipation triples both; the burn is worth casting at the face then, lethal or not
        let aragorn = crate::impls::aragorn::aragorn_out(g, p);
        let low_q = s.opps.iter().copied().filter(|&q| g.player(q).life > 0).min_by_key(|&q| g.player(q).life);
        for &q in &s.opps {
            if player_hexproof_from(g, q, d.colors) {
                continue;
            }
            let life = g.player(q).life;
            let total = if aragorn {
                let extra = if Some(q) == low_q { crate::impls::aragorn::red_damage(g, p, c) } else { 0 };
                dmg.saturating_mul(crate::engine::combat::fiery(g, p)).saturating_add(extra)
            } else {
                dmg
            };
            if aragorn && life > total && Some(q) == low_q {
                o.push(Opt {
                    utility: (0.3 * total as f64).min(6.0),
                    label: format!("{} to the face ({})", d.name, g.player(q).name),
                    act: Some(Action::Face { card: c, q, kick: 0 }),
                });
                continue;
            }
            let dmg = total;
            if life <= dmg {
                o.push(Opt {
                    utility: 9.0,
                    label: format!("{} to the face ({})", d.name, g.player(q).name),
                    act: Some(Action::Face { card: c, q, kick: 0 }),
                });
            } else if kick > 0 && life <= dmg + 2 {
                o.push(Opt {
                    utility: 9.0,
                    label: format!("{} kicked to the face ({})", d.name, g.player(q).name),
                    act: Some(Action::Face { card: c, q, kick }),
                });
            }
        }
    }
    // Clues
    // --- Tiers 4/5 (urza-yawgmoth): Forensic Gadgeteer makes it {1}
    if g.player(p).clues > 0 && can_pay(g, p, crate::impls::t45_urza_yawgmoth::clue_cost(g, p), "", false) {
        o.push(Opt { utility: if post { 2.5 } else { 1.5 }, label: "crack a Clue".into(), act: Some(Action::Clue) });
    }
    o
}

// ------------------------------------------------------------------ main phase
/// brain.hook_options: activated abilities of card code (battlefield, graveyard and hand), the outside decks' generic
/// plays, land abilities and modeled combos. post None: the end-of-turn window.
pub fn hook_options(g: &mut Game, p: PlayerId, s: &Situation, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    if !g.hooks.is_empty() {
        for (src, imp) in crate::engine::hooks::hooked(g, Event::Options) {
            if g.perm(src).owner == p && !crate::cardcode::ability_locked(g, src, p) {
                o.extend((imp.options.unwrap())(g, src, p, post)?);
            }
        }
    }
    for c in g.player(p).gy.clone() {
        if let Some(f) = g.registry.get(c).and_then(|i| i.gy_options) {
            o.extend(f(g, c, p, post)?);
        }
    }
    o.extend(crate::impls::t12_windgrace::emblem_retrace_options(g, p, post)?); // --- urdragon: Wrenn and Six's emblem
    o.extend(crate::impls::bulk_white::face_up_options(g, p, post)?); // face-down creatures turned face up
    o.extend(crate::impls::generic_paths::exile_options(g, p, post)?); // --- generic-paths: a warped Voidborn
    for c in g.player(p).hand.clone() {
        if let Some(f) = g.registry.get(c).and_then(|i| i.hand_options) {
            o.extend(f(g, c, p, post)?);
        }
    }
    // Omnath also uses the outside decks' generic plays (equip, Dispute, reanimation)
    if !is_main(hand_key(g, p)) || hand_key(g, p) == "omnath" {
        o.extend(pool::special_options(g, p, s, post)?);
    } else if g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| &*g.db.get(c).name == "Grave Pact")
    }) || !crate::cardcode::outlets_of(g, p).is_empty()
    {
        o.extend(crate::cardcode::aristocrat_options(g, p, post)?); // Grave Pact: sacrifice for edicts
    }
    o.extend(crate::cardcode::land_options(g, p, post)?);
    o.extend(crate::cardcode::combo_options(g, p, s.ctr_risk, post)?);
    Ok(o)
}

/// Gandalf, Friend of the Shire: flexible sorceries (card draw, tutors, token makers, targeted removal) with no
/// main-phase reason to be cast now wait for the end-of-turn window (`end_of_turn_window`), which casts sorceries at
/// instant speed, and the mana stays open for instants and flash meanwhile. Not held: ramp, creatures, wipes,
/// protection, removal when the board is dangerous, and draw or tutors while a land drop is still needed.
pub fn gandalf_hold(g: &Game, p: PlayerId, s: &Situation, opts: &mut [Opt]) {
    if !has(g, p, Tag::Gandalf) {
        return;
    }
    for o in opts.iter_mut() {
        let (Some(Action::Cast { card, .. }) | Some(Action::Removal { card, .. })) = &o.act else { continue };
        let d = g.db.get(*card);
        let t = &d.tags;
        if !d.sorcery || d.creature || d.land || t.has(Tag::Wipe) || t.has(Tag::Prot) {
            continue;
        }
        let rem = t.has(Tag::Rem);
        let flex =
            rem || t.has(Tag::Draw) || t.has(Tag::Tokx) || t.has(Tag::Tut) || t.has(Tag::Xtutor) || t.has(Tag::Top);
        if !flex || t.has(Tag::Rock) || t.has(Tag::Dork) || t.has(Tag::Lr) || special(g, *card) {
            continue;
        }
        let needs_land = s.lands_in_hand == 0 && s.turn <= 5;
        if (rem && s.danger >= 0.8) || (!rem && needs_land) {
            continue;
        }
        o.utility -= 3.0;
    }
}

/// brain.main_options: every legal main-phase play for p right now; the last is "stop" (act None)
pub fn main_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let s = Situation::new(g, p);
    let (hold_card, hold_v) = hold_value(g, p, &s);
    // --- c1b-engines: what "stop" is worth, as the end of this function computes it (the engine rule reads it)
    let stop_est =
        (if hold_card.is_some() { hold_v } else { -3.0 }) + crate::impls::t6_plans::hold_up(g, p).unwrap_or(0.0);
    let mut opts = vec![];
    let main = is_main(hand_key(g, p));
    let rsv = if !main && !post { pool::combat_reserve(g, p) } else { None };
    let mut cands: Vec<CardId> = g.player(p).hand.clone();
    let mut zoned: Vec<CardId> = g.player(p).cmds_in_zone().collect();
    zoned.sort_by_key(|&c| g.db.get(c).cmc); // two commanders: the cheaper first
    cands.extend(zoned);
    for c in cands {
        // what do_cast would refuse is not offered: an additional cost that can't be paid (Culling the Weak with no
        // creature), a lock, or a free spell Vexing Bauble or Chalice of the Void would counter
        if g.db.get(c).has_dsl() && !g.db.get(c).land && !additional_cost(g, p, c, true)? {
            continue;
        }
        let d = g.db.get(c);
        if d.land {
            continue;
        }
        let zone = if g.player(p).hand.contains(&c) { "hand" } else { "cmd" };
        if g.cast_checks() && !castable(g, p, c, zone) {
            continue;
        }
        if d.tag(Tag::Needsac) && spare_creature(g, p).is_none() {
            continue; // Diabolic Intent with nothing to sacrifice: do_cast would refuse it
        }
        let Some(mut u) = card_utility(g, p, &s, c) else { continue };
        let (cg, cp) = cost_of(g, p, c);
        g.pay_for = Some(c);
        let ok = can_pay(g, p, cg, &cp, d.tag(Tag::Convoke));
        g.pay_for = None;
        if !ok {
            continue;
        }
        // Tier 6 near a win, or an opposing commander one card from its combo: the answer stays up
        let near = if crate::impls::t6_plans::is_t6(g, p) { s.combo_near } else { s.cmdr_combo };
        let mut engine_floor = None;
        if crate::engine::mana::ritual_net(g, p, c) <= 0 {
            let mut pen = reserve_penalty(g, p, c, hold_card, hold_v, near); // --- ai-fix: C: a ritual adds more than it spends
            // --- c1b-engines: a Tier 6 draw engine is cast early whatever it leaves for the held answer; and it is
            // never outvoted by "stop" while the held answer stays payable (see `engine_rule`). `near` there is the
            // archenemy reading only: an opposing commander that is a combo piece (`cmdr_combo`) is on the table of
            // every cEDH pod, Kinnan or Thrasios or Tymna alone proves nothing

            if let Some(engine) = engine_rule(g, p, c, &s, s.combo_near) {
                if engine.early {
                    pen = 0.0;
                }
                if pen == 0.0 {
                    engine_floor = Some((stop_est + ENGINE_MARGIN, engine.early));
                }
            }
            u -= pen;
        }
        if let Some((rg, rp)) = &rsv
            && !can_pay(g, p, cg + rg, &format!("{cp}{rp}"), false)
        {
            u -= 6.0; // outside decks: mana kept for combat (ninjutsu)
        }
        if d.creature
            && (d.tag(Tag::Flash)
                || (crate::impls::t6_blue::static_flash(g, p, true) && !crate::impls::t6_blue::flash_turn_grant(g, p)))
            && !g.player(p).is_commander(c)
            && s.danger < 0.8
        {
            u -= 1.5; // flash: better at the end of the turn before yours
        }
        if let Some((floor, early)) = engine_floor
            && (early || !d.creature || !d.tag(Tag::Flash))
        {
            u = u.max(floor); // --- c1b-engines: the engine is cast before "stop"; a flash creature only on curve
        }
        opts.push(Opt { utility: u, label: d.name.to_string(), act: Some(Action::Cast { card: c, zone: None }) });
    }
    if g.player(p).yawg {
        // Yawgmoth's Will: spells from the graveyard (reanimation has its own path)
        for c in g.player(p).gy.clone() {
            let d = g.db.get(c);
            if d.land || !g.player(p).yawg_gy.contains(&c) || special(g, c) {
                continue;
            }
            let Some(u) = card_utility(g, p, &s, c) else { continue };
            let (cg, cp) = cost_of(g, p, c);
            if !can_pay(g, p, cg, &cp, false) {
                continue;
            }
            opts.push(Opt {
                utility: u,
                label: format!("{} (Yawgmoth's Will)", d.name),
                act: Some(Action::Cast { card: c, zone: Some("yawg") }),
            });
        }
    }
    opts.extend(removal_options(g, p, &s));
    opts.extend(wipe_options(g, p, &s, false));
    opts.extend(special_options(g, p, &s, post)?);
    opts.extend(extra_options(g, p, &s, post, true));
    opts.extend(crate::cardcode::breach_gc_options(g, p)?);
    opts.extend(crate::impls::breach_line::breach_options(g, p, post)?); // the whole Breach line, when it finishes the table
    if g.dsl_on {
        opts.extend(crate::dsl::ability_options(g, p, true)?);
    }
    opts.extend(hook_options(g, p, &s, Some(post))?);
    super::rules::filter_options(g, p, &mut opts)?; // --- phase4-rules
    gandalf_hold(g, p, &s, &mut opts);
    let mut stop_u = if hold_card.is_some() { hold_v } else { -3.0 };
    if let Some(u) = crate::impls::t6_plans::hold_up(g, p) {
        stop_u += u; // a Tier 6 deck's plan slot: pass with the mana up (or develop)
    }
    if rsv.is_some() {
        stop_u = stop_u.max(4.5); // ninjutsu window: go to combat, cast after
    }
    let label = if hold_card.is_some() { "stop (hold mana)" } else { "stop" };
    opts.push(Opt { utility: stop_u, label: label.into(), act: None });
    Ok(opts)
}

fn labels(opts: &[Opt]) -> Vec<(f64, String)> {
    opts.iter().map(|o| (o.utility, o.label.clone())).collect()
}

// ------------------------------------------------------------------ c1b-engines: Tier 6 draw engines
/// --- c1b-engines: how much more an engine cast is worth than "stop" when the held answer stays payable. Passing with
/// mana that nothing needs is not a play: the 400-game traces (seed 900000) had Rhystic Study, Necropotence and the
/// others offered at 6 and lost to "stop (hold mana)" at 5.5-8 with a counter or Ad Nauseam in hand.
pub const ENGINE_MARGIN: f64 = 1.0;
/// --- c1b-engines: through this own turn an engine is cast even when it taps the mana of the held answer (real pilots
/// play Rhystic Study, Mystic Remora, Esper Sentinel and The One Ring on turns 1-3 into open mana)
pub const ENGINE_EARLY_TURN: u32 = 4;
/// --- c1b-engines: the draw and tax engines a Tier 6 deck casts early (the brief's list: `t6stats::FLOW_ENGINES` less
/// Ad Nauseam, a finisher, plus the creature engines)
pub const T6_ENGINES: [&str; 9] = [
    "Rhystic Study",
    "Mystic Remora",
    "Esper Sentinel",
    "Sylvan Library",
    "The One Ring",
    "Necropotence",
    "Faerie Mastermind",
    "Thrasios, Triton Hero",
    "Consecrated Sphinx",
];

/// --- c1b-engines: what `engine_rule` grants
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineRule {
    /// own turn 1-4: the reserve for a held answer does not count against the cast
    pub early: bool,
}

/// --- c1b-engines: a Tier 6 player's draw engine (not its commander) that no opponent is about to win against and
/// that does not take the mana of a line of its own that is ready: it is cast before "stop" while the held answer
/// stays payable, and early (`ENGINE_EARLY_TURN`) even when it does not. A plan that sets the card's priority to 0
/// holds it (`card_utility` is None, the rule is never asked).
pub fn engine_rule(g: &Game, p: PlayerId, c: CardId, s: &Situation, near: bool) -> Option<EngineRule> {
    if near || !crate::impls::t6_plans::is_t6(g, p) || g.player(p).is_commander(c) {
        return None;
    }
    if !T6_ENGINES.contains(&&*g.db.get(c).name) {
        return None;
    }
    if !crate::impls::combos::ready_lines(g, p).is_empty() {
        return None; // a line is ready: its mana comes first
    }
    Some(EngineRule { early: s.turn <= ENGINE_EARLY_TURN })
}

/// --- c1b-engines: trace only. A Tier 6 deck stopped in its main phase: print the nonland cards it could pay for and
/// what the cast logic made of each (its utility, or why it was not offered), with the stop's own utility. Reads the
/// game, never changes it, draws no random numbers.
fn stop_diag(g: &mut Game, p: PlayerId, opts: &[Opt], post: bool) {
    if g.log.is_none() || !(crate::impls::t6_plans::is_t6(g, p) || std::env::var_os("STOP_DIAG").is_some()) {
        return;
    }
    let s = Situation::new(g, p);
    let mut cands: Vec<CardId> = g.player(p).hand.clone();
    cands.extend(g.player(p).cmds_in_zone());
    let mut out = vec![];
    for c in cands {
        let d = g.db.get(c);
        if d.land {
            continue;
        }
        let (cg, cp) = cost_of(g, p, c);
        g.pay_for = Some(c);
        let ok = can_pay(g, p, cg, &cp, d.tag(Tag::Convoke));
        g.pay_for = None;
        if !ok {
            continue;
        }
        let zone = if g.player(p).hand.contains(&c) { "hand" } else { "cmd" };
        let why = if !castable(g, p, c, zone) {
            "not castable".to_string()
        } else {
            let prio = decks::deck_prio_f(g, p, c);
            match card_utility(g, p, &s, c) {
                Some(u) => match opts.iter().find(|o| o.label == *d.name) {
                    Some(o) => format!("u {u:.1} (offered {:.1}, prio {prio})", o.utility),
                    None => format!("u {u:.1} (not offered, prio {prio})"),
                },
                None => format!("no utility (prio {prio})"),
            }
        };
        out.push(format!("{}: {why}", d.name));
    }
    if out.is_empty() {
        return;
    }
    let stop_u = opts.iter().find(|o| o.act.is_none()).map_or(0.0, |o| o.utility);
    let n = g.player(p).name;
    let (hc, hv) = hold_value(g, p, &s);
    let hold = hc.map_or("none".to_string(), |c| format!("{} {hv:.1}", g.db.get(c).name));
    let plan = crate::impls::t6_plans::hold_up(g, p).unwrap_or(0.0);
    let near = s.combo_near;
    let (mana, hand) = (s.mana, s.hand);
    crate::glog!(
        g,
        "      [{n} stops{} (u {stop_u:.1}; hold {hold}, plan {plan:.1}, near {near}), mana {mana}, hand {hand}; payable: {}]",
        if post { " in main 2" } else { "" },
        out.join("; ")
    );
}

/// brain.main: p's main phase. The look-ahead picks when it's on; otherwise (or if its pick can't be made) the
/// heuristic samples an order and makes the first play that works.
pub fn main(g: &mut Game, p: PlayerId, post: bool) -> Res {
    for _ in 0..18 {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        g.tick()?;
        let mut opts = main_options(g, p, post)?;
        crate::gf_audit::decision(g, p, &opts); // --- ai-audit
        // --- ai-planner: the line planner comes first for a Tier 6 deck: the first play of a plan that wins this turn,
        // else of one that leaves a line ready next turn, is made now, and the plan is made again at the next decision.
        // A next-turn plan that starts with an instant tutor keeps the tutor's mana up for the end-of-turn window.
        if super::planner::applies(g, p) {
            use super::planner::{Decision, describe, label};
            match super::planner::decide(g, p)? {
                Decision::Play { step, plan, why: when } => {
                    let lbl = label(g, step);
                    let (n, seq) = (g.player(p).name, describe(g, &plan));
                    crate::glog!(g, "      [{n} decides: {lbl} [plan: {seq}; {when}]]");
                    let done = super::planner::perform(g, p, step)?;
                    let k = match (when == super::planner::WINS, done) {
                        (true, true) => "plan|this_turn_played",
                        (true, false) => "plan|this_turn_play_failed",
                        (false, true) => "plan|next_turn_played",
                        (false, false) => "plan|next_turn_play_failed",
                    };
                    super::planner::bump(g, p, k);
                    if done {
                        crate::t6stats::speed_decided(g, p, &lbl);
                        crate::gf_audit::acted(g, p, &lbl); // --- ai-audit
                        continue;
                    }
                }
                Decision::Reserve { tutor, plan } => {
                    let (n, t, seq) = (g.player(p).name, g.db.get(tutor).name.to_string(), describe(g, &plan));
                    crate::glog!(g, "      [{n} keeps mana for {t} [plan: {seq}; at the end of the turn before ours]]");
                    opts = super::planner::reserve(g, p, tutor, super::planner::pay_x(g, p, plan[0]), opts);
                    super::planner::bump(g, p, "plan|reserve");
                }
                Decision::None => {}
            }
        }
        if crate::impls::t6_plans::is_t6(g, p)
            && let Some(i) = super::t6_table::must_option(g, p, &opts)
            && let Some(a) = &opts[i].act
        {
            // Tier 6: the plan's must-do play comes before the look-ahead
            let lbl = format!("{} [must]", opts[i].label);
            if act::perform(g, p, a)? {
                explain(g, p, &labels(&opts), &lbl, "decides");
                crate::gf_audit::acted(g, p, &lbl); // --- ai-audit
                continue;
            }
        }
        // --- phase4-search: inside a playout of an ISMCTS decision, the decider's own choices follow the tree
        if opts.len() >= 2
            && let Some(i) = if search::enabled(g, p) {
                search::choose(g, p, post, &opts)
            } else {
                super::ismcts::tree_pick(g, p, &opts)
            }
        {
            let o = &opts[i];
            let lbl = format!("{} [search]", o.label);
            explain(g, p, &labels(&opts), &lbl, "decides");
            let Some(a) = &o.act else {
                crate::gf_audit::stop(g, p, &opts, i, true); // --- ai-audit
                stop_diag(g, p, &opts, post);
                return Ok(());
            };
            if act::perform(g, p, a)? {
                crate::gf_audit::acted(g, p, &lbl); // --- ai-audit
                continue;
            }
        }
        let us: Vec<f64> = opts.iter().map(|o| o.utility).collect();
        let t = temp(g, p);
        let order = gumbel_order(&mut g.rng, &us, t);
        let mut acted = false;
        for i in order {
            let o = &opts[i];
            let Some(a) = &o.act else {
                explain(g, p, &labels(&opts), &o.label, "decides");
                crate::gf_audit::stop(g, p, &opts, i, false); // --- ai-audit
                stop_diag(g, p, &opts, post);
                return Ok(());
            };
            if act::perform(g, p, a)? {
                explain(g, p, &labels(&opts), &o.label, "decides");
                crate::gf_audit::acted(g, p, &o.label); // --- ai-audit
                acted = true;
                break;
            }
            // offered, but the engine couldn't make it (the next option in the order is tried)
            crate::glog!(g, "      ({} couldn't play {})", g.player(p).name, o.label);
        }
        if !acted {
            return Ok(());
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ attacks
/// how much more an opponent with Tergrid, God of Fright out draws attacks (a planeswalker about to ultimate: 3)
const TERGRID_ATTACK: f64 = 3.0;

/// brain.choose_defender: who p attacks: threat, a likely kill, a grudge, blockers, attack taxes, play style
pub fn choose_defender(g: &mut Game, p: PlayerId) -> PlayerId {
    let mut opps: Vec<PlayerId> = g.opps(p).filter(|&q| !shielded(g, q)).collect();
    if opps.is_empty() {
        opps = g.opps(p).collect(); // combat damage to a protected player is prevented
    }
    if g.player(p).key == crate::impls::garland::KEY {
        opps = crate::impls::garland::attack_targets(g, p, opps); // never the monarch: the hit would end the steals
    }
    let able: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.noatk && !x.sick
        })
        .collect();
    // --- c3: infect damage is poison, not life; a tax or cap on q cuts what can be sent there
    let pows: Vec<i32> =
        able.iter().map(|&m| if crate::engine::combat::kw(g, m, "infect") { 0 } else { epow(g, m) }).collect();
    let key = g.player(p).key;
    let aggr = style(g, key).aggression;
    let mut items: Vec<(f64, PlayerId)> = vec![];
    for &q in &opps {
        let mut u = 0.25 * threat(g, p, q);
        if crate::impls::t6_plans::is_t6(g, p) {
            u += 0.25 * super::t6_table::threat_bonus(g, q); // Tier 6: hit whoever is about to win
        }
        let my = crate::engine::combat::sendable_power(g, p, q, &pows);
        let pshare = crate::engine::combat::poison_share(g, q, &able);
        if my as f64 >= g.player(q).life as f64 * 0.8 || pshare >= 0.8 {
            u += 4.0 + 2.0 * aggr; // go for the kill (ten poison counters are as lethal as the life)
        } else {
            u += 2.0 * pshare; // --- c3: and infect hits go where they bring a player nearer ten counters
        }
        u += 0.15 * g.player(p).grudge.get(&g.player(q).key).copied().unwrap_or(0.0); // hit back whoever hit you
        if !is_main(hand_key(g, p)) {
            u += pool::ninja_defender_bonus(g, p, q);
        }
        u += crate::impls::newcards_b::stagger_bonus(g, p, q); // Lightning, Army of One
        // planeswalkers about to ultimate draw attacks
        u += g
            .player(q)
            .perms
            .iter()
            .filter_map(|&m| {
                let x = g.perm(m);
                let pw = x.cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
                if !pw || x.loyalty.unwrap_or(0) == 0 {
                    return None;
                }
                let up = crate::cardcode::ult_pressure(g, m);
                (up >= 0.6).then(|| 3.0 * up.min(1.0))
            })
            .psum();
        if has(g, q, Tag::Tergrid) {
            u += TERGRID_ATTACK; // Tergrid, God of Fright keeps every edict and discard: the table races it
        }
        let blockers = g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).tapped).count();
        u -= 0.15 * blockers as f64 * (1.0 - aggr);
        if !g.hooks.is_empty() {
            // attack taxes and caps on q
            let (tax, cap) = crate::engine::combat::attack_restrictions(g, p, q);
            u -= 1.2 * tax as f64 + if cap.is_some_and(|c| c <= 2) { 1.5 } else { 0.0 };
        }
        items.push((u, q));
    }
    let t = temp(g, p);
    let d = sample(&mut g.rng, &items, t).unwrap();
    let ex: Vec<(f64, String)> = items.iter().map(|(u, q)| (*u, g.player(*q).name.to_string())).collect();
    let dn = g.player(d).name;
    explain(g, p, &ex, dn, "attacks");
    d
}

/// brain.filter_attackers: keep some ground creatures home when the table threatens a lot of damage
pub fn filter_attackers(g: &mut Game, p: PlayerId, atk: Vec<PermId>) -> Vec<PermId> {
    let s = Situation::new(g, p);
    let st = style(g, g.player(p).key);
    if s.danger < 0.25 {
        return atk;
    }
    let keep_p = sig((s.danger * st.caution - 0.35 * st.aggression - 0.15) / 0.12);
    let naj = g.player(p).key == "najeela";
    let mut out = vec![];
    for &m in &atk {
        let x = g.perm(m);
        if x.fly || x.vig || x.army || (x.token && naj) {
            out.push(m);
            continue;
        }
        let f = 0.5 + 0.5 * (etgh(g, m) as f64 / 4.0).min(1.0);
        if g.rng.random() < keep_p * f {
            continue;
        }
        out.push(m);
    }
    if out.len() < atk.len() {
        crate::glog!(g, "      [{} keeps {} creature(s) home as blockers]", g.player(p).name, atk.len() - out.len());
    }
    out
}

// ------------------------------------------------------------------ end-of-turn window
/// brain._overload_rift: Cyclonic Rift at the end of the turn before yours: mana untaps next, so the overload costs
/// nothing extra. When it bounces clearly more than the best single target (1.5 times), drop the single-target
/// choices and rank the overload above them.
fn overload_rift(g: &Game, p: PlayerId, opts: Vec<Opt>) -> Vec<Opt> {
    let Some(rift) = g.player(p).hand.iter().copied().find(|&c| {
        let d = g.db.get(c);
        d.tags.str(Tag::Wipe) == Some("rift") && d.instant
    }) else {
        return opts;
    };
    let rname = g.db.get(rift).name.to_string();
    let Some(over) = opts.iter().find(|o| o.label == rname).cloned() else { return opts };
    let ol = decks::wipe_eval(g, p, "rift").0;
    let best = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased)
        .map(|m| pval(g, m) * if g.perm(m).token { 1.0 } else { 0.5 })
        .fold(0.0, f64::max);
    if ol < 1.5 * best {
        return opts;
    }
    let pre = format!("{rname} -> ");
    let top = opts
        .iter()
        .filter(|o| o.label.starts_with(&pre))
        .map(|o| o.utility)
        .chain(std::iter::once(over.utility))
        .fold(f64::NEG_INFINITY, f64::max);
    let mut out: Vec<Opt> = opts.into_iter().filter(|o| !o.label.starts_with(&pre) && o.label != rname).collect();
    out.push(Opt { utility: top + 0.5, label: format!("{} (overload)", over.label), act: over.act });
    out
}

/// brain.end_of_turn_window: at the end of the turn before yours, spend mana you held up but didn't need:
/// instant-speed draw, removal and token spells. Mana untaps next, so holding has no value.
pub fn end_of_turn_window(g: &mut Game, p: PlayerId) -> Res {
    crate::t6stats::eot_window(g, p);
    crate::engine::turn::erebos_draw(g, p)?;
    // --- ai-planner: the planner's instant tutor (or the instant ritual that pays for it) for a line ready next turn
    if super::planner::applies(g, p) {
        for _ in 0..3 {
            if g.over || !g.player(p).alive {
                return Ok(());
            }
            let Some((step, plan, why)) = super::planner::eot_play(g, p)? else { break };
            let lbl = super::planner::label(g, step);
            let seq = super::planner::describe(g, &plan);
            crate::glog!(g, "      [{} end of turn: {lbl} [plan: {seq}; {why}]]", g.player(p).name);
            if !super::planner::perform(g, p, step)? {
                super::planner::bump(g, p, "plan|eot_play_failed");
                break;
            }
            super::planner::bump(g, p, "plan|eot_played");
            g.player_mut(p).stat("eot_casts", 1);
        }
    }
    for _ in 0..6 {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        let s = Situation::new(g, p);
        let mut opts = vec![];
        let gand = has(g, p, Tag::Gandalf); // Gandalf: sorceries at instant speed
        for c in g.player(p).hand.clone() {
            let d = g.db.get(c);
            let t = &d.tags;
            let flashed = crate::engine::cast::grants_flash(g, p, c); // Borne, Trickster, Floodcaller, Quicksilver ...
            if !(d.instant || ((gand || flashed) && d.sorcery))
                || t.has(Tag::Prot)
                || t.has(Tag::Rem)
                || t.has(Tag::Wipe)
            {
                continue;
            }
            if special(g, c) {
                continue; // cast only through their deck-specific logic
            }
            if t.has(Tag::Ctr) {
                if t.has(Tag::Eotdraw) && can_pay_now(g, p, c) {
                    // Mystic Confluence: draw three
                    opts.push(Opt {
                        utility: 2.5,
                        label: format!("{} (draw three)", d.name),
                        act: Some(Action::Plan { f: confluence_draw, arg: c.0 as i64 }),
                    });
                }
                continue;
            }
            let cv = t.has(Tag::Convoke);
            let (cg, cp) = cost_of(g, p, c);
            if !can_pay(g, p, cg, &cp, cv) || (g.cast_checks() && !castable(g, p, c, "hand")) {
                continue;
            }
            const EOT: [Tag; 8] =
                [Tag::Draw, Tag::Tokx, Tag::Treas, Tag::Gifts, Tag::Intuition, Tag::Seal, Tag::Adnaus, Tag::Top];
            if EOT.iter().any(|&k| t.has(k)) || d.sorcery {
                let mut u = card_utility(g, p, &s, c);
                if u.is_none() && !t.has(Tag::Tokx) {
                    continue; // the deck's AI said no (Ad Nauseam at low life)
                }
                if t.has(Tag::Tokx) {
                    u = Some(3.0 + total_mana(g, p, cv) as f64 / 2.0);
                }
                opts.push(Opt {
                    utility: u.unwrap(),
                    label: d.name.to_string(),
                    act: Some(Action::Cast { card: c, zone: None }),
                });
            }
        }
        opts.extend(wipe_options(g, p, &s, true)); // Cyclonic Rift's overload at instant speed
        for c in g.player(p).hand.clone() {
            // flash creatures: in before the next turn's draws
            let d = g.db.get(c);
            if !((d.creature && d.tag(Tag::Flash)) || (d.perm && !d.land && crate::engine::cast::grants_flash(g, p, c)))
                || special(g, c)
                || !can_pay_now(g, p, c)
                || (g.cast_checks() && !castable(g, p, c, "hand"))
            {
                continue;
            }
            if let Some(u) = card_utility(g, p, &s, c) {
                opts.push(Opt {
                    utility: u + 1.0,
                    label: d.name.to_string(),
                    act: Some(Action::Cast { card: c, zone: None }),
                });
            }
        }
        opts.extend(extra_options(g, p, &s, true, gand));
        if g.dsl_on {
            opts.extend(crate::dsl::ability_options(g, p, false)?);
        }
        opts.extend(hook_options(g, p, &s, None)?);
        opts.extend(decks::monolith_untap_options(g, p));
        let caution = style(g, g.player(p).key).caution;
        for mut o in removal_options(g, p, &s) {
            if let Some(Action::Removal { card, kick: 0, .. }) = &o.act
                && (g.db.get(*card).instant
                    || (g.db.get(*card).sorcery && (gand || crate::impls::karn_cards::liberator_flash(g, p, *card))))
            {
                o.utility += 1.2 * caution;
                opts.push(o);
            }
        }
        let mut opts = overload_rift(g, p, opts);
        super::rules::filter_options(g, p, &mut opts)?; // --- phase4-rules
        if opts.is_empty() {
            return Ok(());
        }
        opts.push(Opt { utility: -1.5, label: "pass".into(), act: None });
        let us: Vec<f64> = opts.iter().map(|o| o.utility).collect();
        let t = temp(g, p);
        let order = gumbel_order(&mut g.rng, &us, t);
        let mut done = false;
        for i in order {
            let Some(a) = &opts[i].act else { return Ok(()) };
            if act::perform(g, p, a)? {
                g.player_mut(p).stat("eot_casts", 1);
                let lbl = opts[i].label.clone();
                explain(g, p, &labels(&opts), &lbl, "end of turn");
                done = true;
                break;
            }
        }
        if !done {
            return Ok(());
        }
    }
    Ok(())
}

/// Mystic Confluence at the end of the turn before yours: its draw-three mode
fn confluence_draw(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let d = g.db.get(c);
    let n = d.tags.int_or_0(Tag::Eotdraw);
    let (gn, pips) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let i = pl.hand.iter().position(|&x| x == c).unwrap();
    pl.hand.remove(i);
    pay(g, p, gn, &pips, false)?;
    let pl = g.player_mut(p);
    pl.gy.push(c);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c); // the card report counts a card as cast from this set
    on_cast(g, p, c)?;
    draw(g, p, n as u32, false)?;
    Ok(true)
}

/// Gandalf, Friend of the Shire, defend step: a wipe sorcery cast after attackers are declared when it kills more
/// opposing value (the attackers included) than ours and the attack would cost us a lot (a big share of our life, or
/// commander damage). Returns whether it was cast.
fn gandalf_wipe_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    let incoming: i32 = atk
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.perm(m).owner == p)
        .map(|&m| epow(g, m) * if double_strike(g, m) { 2 } else { 1 })
        .sum();
    if incoming <= 0 || (incoming as f64) < g.player(d).life as f64 * 0.3 {
        return Ok(false); // the attack is no big deal
    }
    let s = Situation::new(g, d);
    let mut best: Option<(f64, CardId, Option<PlayerId>)> = None;
    for &c in &g.player(d).hand {
        let x = g.db.get(c);
        let Some(kind) = x.tags.str(Tag::Wipe) else { continue };
        if !x.sorcery {
            continue; // instants already have their own paths
        }
        let (cg, cp) = decks::wipe_cost(g, d, c);
        if !can_pay(g, d, cg, &cp, false) {
            continue;
        }
        let (ol, ml, victim) = decks::wipe_eval(g, d, kind);
        // wipe_eval counts the attackers as board value; the attack we avoid is worth extra
        let swing = ol - 1.2 * ml + 0.5 * incoming as f64;
        if ol > ml && swing / 2.2 - 2.5 + 2.0 * s.danger.min(1.5) > 0.0 && best.is_none_or(|b| swing > b.0) {
            best = Some((swing, c, victim));
        }
    }
    let Some((_, c, victim)) = best else { return Ok(false) };
    crate::glog!(g, "  {} wipes the board after the attack: {}", g.player(d).name, g.db.get(c).name);
    if !cast_wipe(g, d, c, victim)? {
        return Ok(false);
    }
    g.player_mut(d).stat("attack_removal", 1);
    Ok(true)
}
