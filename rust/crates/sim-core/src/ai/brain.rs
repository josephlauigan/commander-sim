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

use super::{act, decks, explain, gumbel_order, is_main, plans, pool, sample, search, sig, style, temp};
use crate::cards::Types;
use crate::engine::cast::{additional_cost, cast_card, castable, on_cast, pact_affordable, parse_cost};
use crate::engine::combat::double_strike;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, mana_units, pay, phyrexian_life, total_mana};
use crate::engine::removal::legal_targets;
use crate::engine::stack::free_counter;
use crate::engine::values::{commander_out, epow, etgh, has, player_hexproof, pval, shielded, threat, untargetable};
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
    if !pl.deck_names.contains(&pl.cmd) && pred(pl.cmd) {
        total += 1;
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
    pub ctr_risk: f64,
}

impl Situation {
    pub fn new(g: &Game, p: PlayerId) -> Situation {
        let pl = g.player(p);
        let opps: Vec<PlayerId> = g.opps(p).collect();
        let threat: Vec<(PlayerId, f64)> = opps.iter().map(|&q| (q, threat(g, p, q))).collect();
        let leader = max_by(&threat, |x| x.1).map(|x| x.0);
        let max_threat =
            if threat.is_empty() { 0.0 } else { threat.iter().map(|x| x.1).fold(f64::NEG_INFINITY, f64::max) };
        let mut inc = 0.0;
        for &q in &opps {
            let share = 1.0 / g.opps(q).count().max(1) as f64;
            for &m in &g.player(q).perms {
                let x = g.perm(m);
                if g.is_creature(m) && !x.noatk && !x.phased {
                    inc += epow(g, m) as f64 * if x.fly { 1.0 } else { 0.6 } * share;
                }
            }
        }
        let has_q = |q: PlayerId, t: Tag| has(g, q, t);
        let combo_near = opps.iter().any(|&q| {
            let k = g.player(q).key;
            (k == "veyran" && has_q(q, Tag::Vkitten) && has_q(q, Tag::Vfire))
                || (k == "sauron" && has_q(q, Tag::Sword) && has_q(q, Tag::Assault))
                || (k == "veyran" && has_q(q, Tag::Aether) && g.player(q).life >= 40)
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

/// brain.hold_value: the card the AI wants to keep mana up for, and how much
pub fn hold_value(g: &Game, p: PlayerId, s: &Situation) -> (Option<CardId>, f64) {
    let pl = g.player(p);
    let held: Vec<CardId> = pl
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            held(g, c) && !free_counter(g, p, c) && can_pay(g, p, d.generic, &d.pips, false)
        })
        .collect();
    // PORT(phase 6): Jodah's protection mana (CI.jodah_hold)
    if held.is_empty() {
        return (None, 0.0);
    }
    let c = min_by(&held, |c| g.db.get(c).cmc as f64).unwrap();
    let caution = style(g, pl.key).caution;
    let mut v = 1.0 + 3.0 * caution * (s.max_threat / 20.0).min(1.0) + 2.5 * s.combo_near as i32 as f64;
    if !is_main(pl.key) {
        // outside decks: three opponents will cast something worth answering this cycle
        let n = held.iter().filter(|&&x| g.db.get(x).tag(Tag::Ctr)).count();
        v = v.max(1.5 + 2.5 * caution * n.min(2) as f64);
    }
    if pl.key == "seph" && decks::bomb_on_bf(g, p) {
        v += 2.0 * removal_risk(g, p) + 1.0;
    }
    if s.turn <= 2 {
        v -= 2.0;
    }
    (Some(c), v)
}

fn reserve_penalty(g: &Game, p: PlayerId, c: CardId, hold: Option<CardId>, hold_v: f64) -> f64 {
    let Some(h) = hold.filter(|&h| h != c) else { return 0.0 };
    let (cg, cp) = cost_of(g, p, c);
    let hd = g.db.get(h);
    if can_pay(g, p, cg + hd.generic, &format!("{cp}{}", hd.pips), false) {
        return 0.0;
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
    let main = is_main(pl.key);
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
    // its own priority said no (or it's kept for responses): keep it
    let hand_written = g.registry.get(c).is_some_and(|i| i.prio.is_some()) || plans::response_only(pl.key, c);
    if base <= 0.0 && d.has_dsl() && !hand_written {
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
    if c == pl.cmd && !pl.hand.contains(&c) {
        u -= 0.35 * pl.tax as f64; // recasting gets pricier each time (no tax from hand)
    }
    if (d.instant || d.sorcery) && pl.key == "veyran" {
        // each spell fires every magecraft payoff (doubled by Veyran): casting is value in itself
        const PAY: [Tag; 7] =
            [Tag::Ping, Tag::Dragoncaller, Tag::Mystic, Tag::Spelldraw, Tag::Aether, Tag::Spelltok, Tag::Kiln];
        let n_pay = pl
            .perms
            .iter()
            .filter(|&&m| {
                let x = g.perm(m);
                !x.phased && x.cd.is_some_and(|cd| PAY.iter().any(|&k| g.db.get(cd).tag(k)))
            })
            .count();
        u += 0.9 * n_pay as f64 * if has(g, p, Tag::Veyran) { 2.0 } else { 1.0 };
    }
    if (d.bomb >= 5 || c == pl.cmd || base >= 70.0) && !d.land {
        u *= 1.0 - 0.35 * s.ctr_risk; // walking into open counter mana
    }
    if pl.key == "veyran" && c != pl.cmd {
        u += veyran_sequence(g, p, c);
    }
    Some(u)
}

/// brain.veyran_sequence: Veyran doubles every cast trigger, so a spell cast while Veyran waits in the command zone
/// wastes half its value. On your own turn: with the mana for Veyran, cast Veyran before any instant or sorcery that
/// isn't urgent; one land short, hold sorcery-speed card draw for next turn.
fn veyran_sequence(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let pl = g.player(p);
    let d = g.db.get(c);
    if g.active != Some(p) || !pl.cmd_in_zone || !(d.instant || d.sorcery) {
        return 0.0;
    }
    if pl.perms.iter().any(|&m| g.perm(m).cd == Some(pl.cmd) && !g.perm(m).phased) {
        return 0.0;
    }
    const URGENT: [Tag; 7] = [Tag::Rem, Tag::Ctr, Tag::Wipe, Tag::Prot, Tag::Lr, Tag::Rock, Tag::Fastmana];
    if URGENT.iter().any(|&k| d.tag(k)) || d.land {
        return 0.0;
    }
    let (gn, pips) = cost_of(g, p, pl.cmd);
    let (need, have) = (gn + pips.len() as u32, total_mana(g, p, false));
    if have >= need {
        return -4.0;
    }
    if have + 1 >= need && d.tag(Tag::Draw) && pl.hand.len() >= 3 && d.sorcery {
        return -2.5;
    }
    0.0
}

// ------------------------------------------------------------------ generic executors
/// cards whose casting needs deck-specific choices (targets, modes, X); never cast generically (brain.SPECIAL)
const SPECIAL: [Tag; 7] = [Tag::Rean, Tag::Fill, Tag::Yawg, Tag::Avarice, Tag::Mastery, Tag::Crackle, Tag::Xdrain];
/// graveyard fillers that are plain permanents: cast normally
const CAST_FILL: [&str; 3] = ["stitcher", "tortured", "wayfinder"];
/// a creature worth more than this (pval) isn't sacrificed to pay for a spell
const SPARE_VALUE: f64 = 3.0;

fn special(g: &Game, c: CardId) -> bool {
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
            g.is_creature(m) && !x.phased && !x.is_cmd && !x.army
        })
        .collect();
    let m = min_by(&cands, |m| pval(g, m))?;
    (pval(g, m) <= SPARE_VALUE).then_some(m)
}

/// brain.do_cast: p casts c (from hand or the command zone; zone 'gy': its flashback; 'yawg': from the graveyard
/// under Yawgmoth's Will). False if it can't now.
pub fn do_cast(g: &mut Game, p: PlayerId, c: CardId, zone: Option<Sym>) -> Res<bool> {
    let db = g.db.clone();
    let d = db.get(c);
    let main = is_main(g.player(p).key);
    if special(g, c)
        && !d.has_dsl()
        && !(d.creature && !main)
        && !d.tags.str(Tag::Fill).is_some_and(|f| CAST_FILL.contains(&f))
    {
        return Ok(false);
    }
    if d.has_dsl() && !additional_cost(g, p, c, true)? {
        return Ok(false);
    }
    let pl = g.player(p);
    let in_hand = pl.hand.contains(&c);
    if matches!(zone, None | Some("hand")) && !in_hand && !(c == pl.cmd && pl.cmd_in_zone) {
        return Ok(false);
    }
    if zone == Some("yawg") && !pl.gy.contains(&c) {
        return Ok(false);
    }
    let zone: Sym = zone.unwrap_or(if c == pl.cmd && !in_hand { "cmd" } else { "hand" });
    let cast_zone: Sym = if zone == "yawg" { "gy" } else { zone };
    if !g.hooks.is_empty() && !castable(g, p, c, cast_zone) {
        return Ok(false);
    }
    let cv = d.tag(Tag::Convoke);
    let mut fodder = None;
    if d.tag(Tag::Needsac) {
        // Diabolic Intent: sacrifice a creature as an additional cost
        fodder = spare_creature(g, p);
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
    let ok = can_pay(g, p, cg, &cp, cv);
    let paid = if ok { pay(g, p, cg, &cp, cv) } else { Ok(false) };
    g.pay_for = None;
    if !ok || !paid? {
        return Ok(false);
    }
    let mut ctx = Ctx::default();
    if d.tag(Tag::Tokx) || d.tag(Tag::Xtutor) {
        // X spells of outside decks: X = all spare mana
        let x = total_mana(g, p, cv);
        pay(g, p, x, "", cv)?;
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
    if matches!(zone, "hand" | "cmd") && !d.phyrexian.is_empty() {
        // Phyrexian symbols paid with 2 life each
        let life = phyrexian_life(g, c, &cp);
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
    let ok = cast_card(g, p, c, cast_zone, ctx)?; // from the graveyard: exiled after
    if g.player(p).key == "seph" && ok && (c == g.player(p).cmd || d.bomb >= 4) {
        super::note_bomb(g, p, c, false);
    }
    Ok(true)
}

fn swamp_land(g: &Game, p: PlayerId) -> bool {
    g.player(p).lands.iter().any(|&l| {
        let d = g.db.get(g.land(l).cd);
        d.has_subtype("swamp") || &*d.name == "Swamp"
    })
}

/// sacrifice fodder for removal costs: creatures that are tokens or no bomb
fn rem_fodder(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && (x.token || x.cd.is_none_or(|cd| g.db.get(cd).bomb == 0))
        })
        .collect()
}

/// a removal spell castable for free (Fierce Guardianship-style with your commander out; Snuff Out for 4 life)
fn free_removal(g: &Game, p: PlayerId, c: CardId) -> bool {
    let t = &g.db.get(c).tags;
    (t.has(Tag::Freecmd) && commander_out(g, p)) || (t.has(Tag::Snuff) && g.player(p).life > 12 && swamp_land(g, p))
}

fn rem_spec(g: &Game, c: CardId) -> (&str, &str, bool) {
    let t = &g.db.get(c).tags;
    (t.str(Tag::Rem).unwrap_or(""), t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4))
}

/// brain.removal_options
pub fn removal_options(g: &Game, p: PlayerId, s: &Situation) -> Vec<Opt> {
    let mut out = vec![];
    let main = is_main(g.player(p).key);
    let caution = style(g, g.player(p).key).caution;
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        let t = &d.tags;
        if !t.has(Tag::Rem) || d.creature {
            continue;
        }
        let fods = rem_fodder(g, p);
        let extra = if t.has(Tag::Sacor4) && fods.is_empty() { 4 } else { 0 };
        if !free_removal(g, p, c) && !can_pay(g, p, d.generic + extra, &d.pips, t.has(Tag::Convoke)) {
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
        let tg = legal_targets(g, p, rk, tgt, mv4, Some(c));
        let Some(best) = max_by(&tg, |m| pval(g, m)) else { continue };
        let mut v = pval(g, best);
        if Some(g.perm(best).owner) == s.leader {
            v *= 1.25;
        }
        if s.combo_near && pval(g, best) >= 8.0 {
            v += 2.0;
        }
        if !main && v < 2.5 {
            continue; // outside decks: no removal on trivial targets
        }
        let mut u = v - 3.5;
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
            && can_pay(g, p, d.generic + k as u32, &d.pips, false)
        {
            let kd = format!("dmg{}", rk[3..].parse::<i32>().unwrap_or(0) + 2); // Burst Lightning kicked: 4 damage
            let tg4 = legal_targets(g, p, &kd, tgt, mv4, Some(c));
            if let Some(b4) = max_by(&tg4, |m| pval(g, m))
                && pval(g, b4) > v + 1.0
            {
                out.push(Opt {
                    utility: pval(g, b4) - 4.5,
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
    let rem: Vec<CardId> = g
        .player(d)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let x = g.db.get(c);
            x.instant && x.tag(Tag::Rem) && !x.creature
        })
        .collect();
    // Galadriel's permanents' answers (Ballista Squad, Lawbringer, Lightbringer)
    let answers =
        if g.player(d).key == "galadriel" { crate::impls::galadriel::attack_answers(g, d, p, atk) } else { vec![] };
    if (rem.is_empty() && answers.is_empty()) || atk.is_empty() {
        return Ok(false);
    }
    let incoming: i32 = atk
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.perm(m).owner == p)
        .map(|&m| epow(g, m) * if double_strike(g, m) { 2 } else { 1 })
        .sum();
    let dl = g.player(d).life;
    let danger = incoming as f64 >= dl as f64 * 0.5
        || atk.iter().any(|&m| g.perm(m).is_cmd && g.player(d).cmd_dmg[p.index()] + epow(g, m) >= 21);
    let mut best: Option<(f64, CardId, PermId)> = None;
    for &c in &rem {
        let x = g.db.get(c);
        if !can_pay(g, d, x.generic, &x.pips, x.tag(Tag::Convoke)) {
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
            let v = pval(g, m) + if danger { 3.0 } else { 0.0 } + 0.3 * epow(g, m) as f64;
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, c, m));
            }
        }
    }
    if let Some(&(v2, a)) = answers.iter().fold(None, |b: Option<&(f64, _)>, x| match b {
        Some(y) if y.0 >= x.0 => Some(y),
        _ => Some(x),
    }) && v2 + if danger { 3.0 } else { 0.0 } - 3.0 > 0.0
        && best.is_none_or(|b| v2 >= b.0 - 1.0)
    {
        if crate::impls::galadriel::answer(g, d, a)? {
            g.player_mut(d).stat("attack_removal", 1);
            return Ok(true);
        }
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

/// brain.cast_removal: p casts removal spell c at one of tg (sampled by value), kicked for `kick` more
pub fn cast_removal(g: &mut Game, p: PlayerId, c: CardId, tg: &[PermId], kick: u32, kind: Option<Sym>) -> Res<bool> {
    let db = g.db.clone();
    let d = db.get(c);
    let cv = d.tag(Tag::Convoke);
    let fods = rem_fodder(g, p);
    let mut extra = if d.tag(Tag::Sacor4) && fods.is_empty() { 4 } else { 0 };
    let free = free_removal(g, p, c);
    extra += kick;
    if !g.player(p).hand.contains(&c) || (!free && !can_pay(g, p, d.generic + extra, &d.pips, cv)) {
        return Ok(false);
    }
    if !g.hooks.is_empty() && !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let live: Vec<(f64, PermId)> =
        tg.iter().copied().filter(|&m| g.perm(m).on_bf && !untargetable(g, m)).map(|m| (pval(g, m) / 1.5, m)).collect();
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
    if free && d.tag(Tag::Snuff) && !(d.tag(Tag::Freecmd) && commander_out(g, p)) {
        lose_life(g, p, 4, Some(p), "life", None)?;
    } else if !free {
        pay(g, p, d.generic + extra, &d.pips, cv)?;
    }
    if let Some(f) = fod {
        die(g, f, "sac")?;
    }
    cast_card(g, p, c, "hand", Ctx { target: Some(target), rem_kind: kind, ..Ctx::default() })?;
    g.player_mut(p).stat("removal_cast", 1);
    Ok(true)
}

/// brain.wipe_options. eot: the end of the turn before yours, where only instant-speed wipes (Cyclonic Rift's
/// overload) are offered; in the main phase an instant-speed wipe is held for that window unless the danger is now.
pub fn wipe_options(g: &Game, p: PlayerId, s: &Situation, eot: bool) -> Vec<Opt> {
    let mut out = vec![];
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        let Some(kind) = d.tags.str(Tag::Wipe) else { continue };
        if eot && !d.instant {
            continue;
        }
        let (cg, cp) = decks::wipe_cost(g, p, c);
        if !can_pay(g, p, cg, &cp, false) {
            continue;
        }
        let (ol, ml, victim) = decks::wipe_eval(g, p, kind);
        if ol <= 0.0 {
            continue; // hits nothing of theirs (Nibelheim Aflame with no creature of ours)
        }
        let swing = ol - 1.2 * ml;
        let mut u = swing / 2.2 - 2.5 + 2.0 * s.danger.min(1.5);
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
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    if !g.hooks.is_empty() && !castable(g, p, c, "hand") {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    cast_card(g, p, c, "hand", Ctx { victim, ..Ctx::default() })?;
    g.player_mut(p).stat("wipes_cast", 1);
    Ok(true)
}

// ------------------------------------------------------------------ deck-specific plays
/// brain.special_options: the plays your decks' own plans add. Only legal plays are listed, so the distribution is
/// over real choices.
pub fn special_options(g: &mut Game, p: PlayerId, s: &Situation, post: bool) -> Res<Vec<Opt>> {
    match g.player(p).key {
        "sauron" => decks::sauron_options(g, p, s, post),
        // PORT(phase 6): Sephiroth's (loops, reanimation, fill, tutors, hardcasts ...), Veyran's (the combo,
        // Aetherflux, Mizzix's Mastery) and Jodah's plays
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
        if !d.tag(Tag::Face) || !r.starts_with("dmg") || !can_pay(g, p, d.generic, &d.pips, false) {
            continue;
        }
        let dmg = r[3..].parse::<i32>().unwrap_or(0) + thor;
        let kick = match d.tags.int(Tag::Kick) {
            Some(k) if can_pay(g, p, d.generic + k as u32, &d.pips, false) => k as u32,
            _ => 0,
        };
        for &q in &s.opps {
            if player_hexproof(g, q) {
                continue;
            }
            let life = g.player(q).life;
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
    if g.player(p).clues > 0 && can_pay(g, p, 2, "", false) {
        o.push(Opt { utility: if post { 2.5 } else { 1.5 }, label: "crack a Clue".into(), act: Some(Action::Clue) });
    }
    o
}

// ------------------------------------------------------------------ main phase
/// your decks that also use the outside decks' generic plays (equip, Dispute, reanimation)
const GENERIC_PLAYS: [&str; 4] = ["galadriel", "yshtola", "alela", "jodah"];

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
    for c in g.player(p).hand.clone() {
        if let Some(f) = g.registry.get(c).and_then(|i| i.hand_options) {
            o.extend(f(g, c, p, post)?);
        }
    }
    let key = g.player(p).key;
    if !is_main(key) || GENERIC_PLAYS.contains(&key) {
        o.extend(pool::special_options(g, p, s, post)?);
    } else if g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| &*g.db.get(c).name == "Grave Pact")
    }) {
        o.extend(crate::cardcode::aristocrat_options(g, p, post)?); // Grave Pact: sacrifice for edicts
    }
    o.extend(crate::cardcode::land_options(g, p, post)?);
    o.extend(crate::cardcode::combo_options(g, p, s.ctr_risk, post)?);
    Ok(o)
}

/// brain.main_options: every legal main-phase play for p right now; the last is "stop" (act None)
pub fn main_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let s = Situation::new(g, p);
    let (hold_card, hold_v) = hold_value(g, p, &s);
    let mut opts = vec![];
    let key = g.player(p).key;
    let main = is_main(key);
    let rsv = if !main && !post { pool::combat_reserve(g, p) } else { None };
    let mut cands: Vec<CardId> = g.player(p).hand.clone();
    if g.player(p).cmd_in_zone {
        cands.push(g.player(p).cmd);
    }
    for c in cands {
        let d = g.db.get(c);
        if d.land {
            continue;
        }
        let Some(mut u) = card_utility(g, p, &s, c) else { continue };
        let (cg, cp) = cost_of(g, p, c);
        g.pay_for = Some(c);
        let ok = can_pay(g, p, cg, &cp, d.tag(Tag::Convoke));
        g.pay_for = None;
        if !ok {
            continue;
        }
        u -= reserve_penalty(g, p, c, hold_card, hold_v);
        if let Some((rg, rp)) = &rsv
            && !can_pay(g, p, cg + rg, &format!("{cp}{rp}"), false)
        {
            u -= 6.0; // outside decks: mana kept for combat (ninjutsu)
        }
        if d.creature && d.tag(Tag::Flash) && c != g.player(p).cmd && s.danger < 0.8 {
            u -= 1.5; // flash: better at the end of the turn before yours
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
    if g.dsl_on {
        opts.extend(crate::dsl::ability_options(g, p, true)?);
    }
    opts.extend(hook_options(g, p, &s, Some(post))?);
    let mut stop_u = if hold_card.is_some() { hold_v } else { -3.0 };
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

/// brain.main: p's main phase. The look-ahead picks when it's on; otherwise (or if its pick can't be made) the
/// heuristic samples an order and makes the first play that works.
pub fn main(g: &mut Game, p: PlayerId, post: bool) -> Res {
    for _ in 0..18 {
        if g.over || !g.player(p).alive {
            return Ok(());
        }
        g.tick()?;
        let opts = main_options(g, p, post)?;
        if opts.len() >= 2
            && search::enabled(g, p)
            && let Some(i) = search::choose(g, p, post, &opts)
        {
            let o = &opts[i];
            let lbl = format!("{} [search]", o.label);
            explain(g, p, &labels(&opts), &lbl, "decides");
            let Some(a) = &o.act else { return Ok(()) };
            if act::perform(g, p, a)? {
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
                return Ok(());
            };
            if act::perform(g, p, a)? {
                explain(g, p, &labels(&opts), &o.label, "decides");
                acted = true;
                break;
            }
        }
        if !acted {
            return Ok(());
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ attacks
/// brain.choose_defender: who p attacks: threat, a likely kill, a grudge, blockers, attack taxes, play style
pub fn choose_defender(g: &mut Game, p: PlayerId) -> PlayerId {
    let mut opps: Vec<PlayerId> = g.opps(p).filter(|&q| !shielded(g, q)).collect();
    if opps.is_empty() {
        opps = g.opps(p).collect(); // combat damage to a protected player is prevented
    }
    let my: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.noatk && !x.sick
        })
        .map(|&m| epow(g, m))
        .sum();
    let key = g.player(p).key;
    let aggr = style(g, key).aggression;
    let mut items: Vec<(f64, PlayerId)> = vec![];
    for &q in &opps {
        let mut u = 0.25 * threat(g, p, q);
        if my as f64 >= g.player(q).life as f64 * 0.8 {
            u += 4.0 + 2.0 * aggr; // go for the kill
        }
        u += 0.15 * g.player(p).grudge.get(&g.player(q).key).copied().unwrap_or(0.0); // hit back whoever hit you
        if !is_main(key) {
            u += pool::ninja_defender_bonus(g, p, q);
        }
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
    crate::engine::turn::erebos_draw(g, p)?;
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
            if !(d.instant || (gand && d.sorcery)) || t.has(Tag::Prot) || t.has(Tag::Rem) || t.has(Tag::Wipe) {
                continue;
            }
            if special(g, c) {
                continue; // cast only through their deck-specific logic
            }
            if t.has(Tag::Ctr) {
                if t.has(Tag::Eotdraw) && can_pay(g, p, d.generic, &d.pips, false) {
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
            if !can_pay(g, p, d.generic, &d.pips, cv) {
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
            if !(d.creature && d.tag(Tag::Flash)) || special(g, c) || !can_pay(g, p, d.generic, &d.pips, false) {
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
                && g.db.get(*card).instant
            {
                o.utility += 1.2 * caution;
                opts.push(o);
            }
        }
        let mut opts = overload_rift(g, p, opts);
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
    let (gn, pips, n) = (d.generic, d.pips.to_string(), d.tags.int_or_0(Tag::Eotdraw));
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
    on_cast(g, p, c)?;
    draw(g, p, n as u32, false)?;
    Ok(true)
}
