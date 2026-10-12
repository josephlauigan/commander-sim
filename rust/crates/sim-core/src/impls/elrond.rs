//! The elrond deck (decklists/JD/elrond-ring-tempts.md): mono-blue, the Ring tempting you through a second creature
//! entering each turn. Its creatures' card code and AI plan. The brief is rust/agents/elrond-core.md; the verdicts are
//! in data/card_overrides.toml.
//!
//! Cards modeled here (a compiled ability is kept where it is exact):
//! - **Elrond, Lord of Rivendell**: scry 1 for him and each other creature that enters under your control, on any
//!   turn (cards through `Etb`, tokens through `TokensEnter`, one trigger each); the second resolution of the turn
//!   makes the Ring tempt you, the third and later only scry (`DataKey::Resolved` / `ResolvedAt` on his permanent).
//!   The Ring-bearer the AI picks here is the best evasive attacker (`ring_pick`), never Gandalf while another
//!   creature is out; **Gandalf, Friend of the Shire** draws when the Ring tempts you and the bearer is another
//!   creature (`gandalf_draw`, called from `mine::ring_tempt`).
//! - **Cloudfin Raptor** (evolve), **Defiler of Dreams** (draw on a blue permanent spell; pay 2 life for {U} less,
//!   `defiler_discount` reads in `mana::cost_of`), **Deluge Virtuoso** (tap + stun counter; opus), **Rapier Wit**,
//!   **Matterbending Mage**, **Mindseeker Oculus**, **Mouth of the Storm** (-3/-0 until your next turn:
//!   `DataKey::Weaken`, read by `values::epow`), **Orysa** (cost), **Professor Hulk**, **Rescue, Pepper Potts**,
//!   **Starbreach Whale** and **Starwinder** (warp), **Textbook Tabulator** (increment), **The Wasp**, **Dune Mover**
//!   (toxic), **Fear of Isolation** (the returned permanent is an additional cost).
//!
//! Stun counters (`DataKey::Stun`) are removed by the untap step in `turn::step_start` instead of untapping.
//!
//! The AI plan: Elrond as soon as three mana allows; with him out, creature spells first, flash creatures held for
//! the other players' end steps (or as the second entry of a turn), creatures that return a creature (Matterbending
//! Mage, Rescue, Fear of Isolation) pick the best enter-the-battlefield creature (`best_own_etb`) unless an
//! opponent's creature is the better target, and warp when that makes a second entry or the full cost is out of
//! reach. Counterspells are kept up by the generic hold logic (`brain::hold_value`); `elrond_protect` counters the
//! removal spell or bounces Elrond and the Ring-bearer.

use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Types};
use crate::engine::cast::cast_card;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::{cast_counter, counter_window, pick_counter, trigger_window};
use crate::engine::values::{card_name, card_tag, epow, etgh, pval};
use crate::engine::zones::{bounce, draw, exile_perm, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

pub const ELROND: &str = "Elrond, Lord of Rivendell";
const GANDALF: &str = "Gandalf, Friend of the Shire";
const CLOUDFIN: &str = "Cloudfin Raptor";
const DEFILER: &str = "Defiler of Dreams";
const VIRTUOSO: &str = "Deluge Virtuoso";
const WIT: &str = "Rapier Wit";
const MAGE: &str = "Matterbending Mage";
const OCULUS: &str = "Mindseeker Oculus";
const MOUTH: &str = "Mouth of the Storm";
const ORYSA: &str = "Orysa, Tide Choreographer";
const HULK: &str = "Professor Hulk";
const RESCUE: &str = "Rescue, Pepper Potts";
const WHALE: &str = "Starbreach Whale";
const WINDER: &str = "Starwinder";
const TABULATOR: &str = "Textbook Tabulator";
const WASP: &str = "The Wasp, Winsome Avenger";
const COATL: &str = "Hope-Ender Coatl";
const MOVER: &str = "Dune Mover";
const FEAR: &str = "Fear of Isolation";

/// the flash creatures the plan holds for end steps
const FLASHERS: [&str; 5] = ["Divining Duelist", COATL, RESCUE, WASP, GANDALF];

// ================================================================== helpers
fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn my_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, m: PermId) -> &str {
    card_name(g, m).unwrap_or("")
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
}

/// p's Elrond, if he is out
pub fn elrond_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && card_name(g, m) == Some(ELROND))
}

/// how many times Elrond's ability has resolved this turn
pub fn resolved_now(g: &Game, e: PermId) -> i64 {
    let d = &g.perm(e).data;
    if d.get(DataKey::ResolvedAt) == Some(&Val::Stamp(g.turn_stamp())) { d.int(DataKey::Resolved) } else { 0 }
}

/// mana spent on the spell being cast now (what the card costs now; a free cast spent none)
fn mana_spent(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let Some((cc, ctx, _)) = &g.cur_cast else { return 0 };
    if *cc != c || ctx.paid_otherwise {
        return 0;
    }
    let (gn, pips) = cost_of(g, p, c);
    gn + pips.chars().count() as u32
}

fn is_artifact(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

fn counter_mult(g: &Game, m: PermId) -> i32 {
    crate::dsl::counter_mult(g, g.perm(m).owner)
}

// ================================================================== the Ring
/// the Ring-bearer the elrond deck chooses: the attacker that is hardest to block (flying, hexproof), then the
/// biggest; Gandalf (which never attacks) only when he is the only creature, so his own draw trigger stays on
pub fn ring_pick(g: &Game, p: PlayerId, cr: &[PermId]) -> PermId {
    let score = |m: PermId| -> f64 {
        let mut s = epow(g, m) as f64 + 0.4 * etgh(g, m) as f64;
        if g.perm(m).noatk || card_tag(g, m, Tag::Noatk) {
            s -= 100.0;
        }
        if crate::dsl::has_kw(g, m, "flying") {
            s += 6.0;
        }
        if crate::dsl::has_kw(g, m, "hexproof") {
            s += 3.0;
        }
        if g.perm(m).data.truthy(DataKey::Unblockable) {
            s += 6.0;
        }
        if g.perm(m).token {
            s -= 0.5;
        }
        if Some(m) == g.player(p).ring_bearer {
            s += 0.5; // keep the bearer it has
        }
        s
    };
    max_by(cr, score).unwrap()
}

/// Gandalf, Friend of the Shire: whenever the Ring tempts you, if you chose a creature other than Gandalf as your
/// Ring-bearer, draw a card (`mine::ring_tempt`, once the bearer is chosen)
pub fn gandalf_draw(g: &mut Game, p: PlayerId, bearer: PermId) -> Res {
    for src in my_creatures(g, p) {
        if card_name(g, src) != Some(GANDALF) || src == bearer {
            continue;
        }
        if trigger_window(g, p, Some(src), "draw a card (the Ring tempted you)", Some(3.0))? {
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

// ================================================================== Elrond
/// a creature (card or token) entered under Elrond's controller: scry 1; the second resolution this turn makes the
/// Ring tempt you
fn elrond_enters(g: &mut Game, src: Src) -> Res {
    let o = g.perm(src).owner;
    if !live(g, src) || g.perm(src).neutered {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "scry 1; the second time each turn, the Ring tempts you", Some(4.0))? {
        return Ok(());
    }
    let st = g.turn_stamp();
    let x = g.perm_mut(src);
    let n =
        if x.data.get(DataKey::ResolvedAt) == Some(&Val::Stamp(st)) { x.data.int(DataKey::Resolved) + 1 } else { 1 };
    x.data.set(DataKey::ResolvedAt, Val::Stamp(st));
    x.data.set(DataKey::Resolved, Val::Int(n));
    crate::cardcode::scry(g, o, 1, false)?;
    if n == 2 {
        g.player_mut(o).stat("ring_tempts", 1);
        let turns = g.player(o).turns;
        g.player_mut(o).milestone.entry("elrond").or_insert(turns); // the elrond deck's plan online
        crate::cardcode::ring_tempt(g, o)?;
    }
    Ok(())
}

fn elrond_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p != g.perm(src).owner || !g.is_creature(m) {
        return Ok(());
    }
    elrond_enters(g, src)
}

fn elrond_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    for &t in toks {
        if g.perm(t).on_bf && g.is_creature(t) {
            elrond_enters(g, src)?;
        }
    }
    Ok(())
}

// ================================================================== Cloudfin Raptor
/// Evolve: whenever a creature enters under your control, if it has greater power or toughness than this creature, a
/// +1/+1 counter on this creature (checked as it enters and again on resolution)
fn evolve(g: &mut Game, src: Src, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let greater = |g: &Game| epow(g, m) > epow(g, src) || etgh(g, m) > etgh(g, src);
    if m == src || !live(g, src) || !g.perm(m).on_bf || !greater(g) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "evolve: a +1/+1 counter", Some(2.0))? && live(g, src) && greater(g) {
        let k = counter_mult(g, src);
        crate::cardcode::add_counters(g, src, k);
    }
    Ok(())
}

fn cloudfin_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if p != g.perm(src).owner || !g.is_creature(m) {
        return Ok(());
    }
    evolve(g, src, m)
}

fn cloudfin_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    for &t in toks {
        evolve(g, src, t)?;
    }
    Ok(())
}

// ================================================================== Defiler of Dreams
/// c is a blue permanent spell p casts for {U} less by paying 2 life (Defiler of Dreams): the AI pays the life only
/// with life to spare, and only for a spell with a blue mana symbol in its cost
pub fn defiler_discount(g: &Game, p: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if !d.perm || d.land || !d.colors.has('U') || !d.pips.contains('U') || d.kws.iter().any(|k| &**k == "devoid") {
        return false;
    }
    if g.player(p).life < 15 || g.player(p).flag_turn.get(&intern("elrond_warp_cast")) == Some(&g.turn_stamp()) {
        return false;
    }
    g.player(p).perms.iter().any(|&m| live(g, m) && card_name(g, m) == Some(DEFILER))
}

/// Whenever you cast a blue permanent spell, draw a card; when it was cast for {U} less, the 2 life is paid with it
fn defiler_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    let blue = d.perm && !d.land && d.colors.has('U') && !d.kws.iter().any(|k| &**k == "devoid");
    if caster != o || !blue || !live(g, src) {
        return Ok(());
    }
    if defiler_discount(g, o, c) {
        lose_life(g, o, 2, Some(o), "other", None)?; // the cost paid with the spell, even if the draw is countered
    }
    if trigger_window(g, o, Some(src), "draw a card (blue permanent spell)", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Deluge Virtuoso and stun counters
/// put a stun counter on m (it is tapped by the caller)
pub fn stun(g: &mut Game, m: PermId) {
    let n = g.perm(m).data.int(DataKey::Stun);
    g.perm_mut(m).data.set(DataKey::Stun, Val::Int(n + 1));
}

/// the opposing creature worth tapping and stunning: the one that blocks or attacks best, untapped
fn stun_target(g: &Game, p: PlayerId) -> Option<PermId> {
    super::common::best_opp_creature(g, p, |m| !g.perm(m).tapped)
        .or_else(|| super::common::best_opp_creature(g, p, |_| true))
}

/// Deluge Virtuoso: when it enters, tap target creature an opponent controls and put a stun counter on it
fn virtuoso_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != g.perm(src).owner {
        return Ok(());
    }
    let Some(t) = stun_target(g, p) else { return Ok(()) };
    if trigger_window(g, p, Some(src), "tap a creature and put a stun counter on it", Some(4.0))? && live(g, t) {
        g.perm_mut(t).tapped = true;
        stun(g, t);
        crate::glog!(g, "    {} is tapped and stunned", g.perm(t).name);
    }
    Ok(())
}

/// Opus: whenever you cast an instant or sorcery spell, +1/+1 until end of turn; +2/+2 instead if five or more mana
/// was spent on it
fn virtuoso_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) || !live(g, src) {
        return Ok(());
    }
    let n = if mana_spent(g, o, c) >= 5 { 2 } else { 1 };
    if trigger_window(g, o, Some(src), "opus: +1/+1 until end of turn", Some(1.0))? && live(g, src) {
        let x = g.perm_mut(src);
        x.eot_pt = (x.eot_pt.0 + n, x.eot_pt.1 + n);
    }
    Ok(())
}

/// Rapier Wit: tap target creature; if it's your turn, a stun counter on it. Draw a card. (The target is the cast's
/// own, or the best untapped creature an opponent controls.)
fn wit_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&t| live(g, t) && g.is_creature(t) && !crate::engine::values::untargetable(g, t));
    if let Some(t) = t.or_else(|| stun_target(g, p)) {
        g.perm_mut(t).tapped = true;
        if g.active == Some(p) {
            stun(g, t);
        }
    }
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ================================================================== returning your own creatures
/// what re-entering is worth for permanent m of p (a creature that is out): its enter trigger, plus the entry itself
/// for Elrond, less what it costs to cast again
pub fn etb_score(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let Some(cd) = g.perm(m).cd else { return -9.0 };
    if g.perm(m).token {
        return -9.0;
    }
    let d = g.db.get(cd);
    let v = match &*d.name {
        "Ultimecia, Temporal Threat" => 8.0,
        ORYSA => 7.0,
        "Coral Barrier" => 6.0, // two bodies: two entries
        VIRTUOSO => {
            if g.opps(p).any(|q| g.player(q).perms.iter().any(|&x| live(g, x) && g.is_creature(x))) {
                5.0
            } else {
                1.0
            }
        }
        "Mnemonic Wall" => {
            if g.player(p).gy.iter().any(|&x| g.db.get(x).instant || g.db.get(x).sorcery) {
                5.0
            } else {
                0.5
            }
        }
        OCULUS => 4.0,
        "Divining Duelist" => 4.0,
        "Keeper of the Quiet Hour" => 3.5,
        WHALE | TABULATOR | "Quicksilver Fisher" => 3.0,
        MAGE | RESCUE | "Peerless Ropemaster" => 3.0,
        "Silver Deputy" | MOVER => 2.0,
        _ => 0.0,
    };
    let bonus = match elrond_of(g, p) {
        Some(e) if resolved_now(g, e) < 2 => 2.5, // another entry this turn
        Some(_) => 1.0,
        None => 0.0,
    };
    // the Ring-bearer and big bodies are worth more on the table
    let keep = 0.25 * epow(g, m) as f64 + if g.player(p).ring_bearer == Some(m) { 1.5 } else { 0.0 };
    v + bonus - 0.5 * d.cmc as f64 - keep
}

/// the creature (or, for `artifacts`, artifact) of p's worth returning to hand for its enter trigger, with its score
pub fn best_own_etb(g: &Game, p: PlayerId, exclude: Option<PermId>, artifacts: bool) -> Option<(PermId, f64)> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m)
                && Some(m) != exclude
                && g.perm(m).cd.is_some()
                && !g.perm(m).is_cmd
                && (g.is_creature(m) || (artifacts && is_artifact(g, m)))
        })
        .collect();
    let best = max_by(&cands, |m| etb_score(g, p, m) + if artifacts && is_artifact(g, m) { 1.5 } else { 0.0 })?;
    Some((best, etb_score(g, p, best) + if artifacts && is_artifact(g, best) { 1.5 } else { 0.0 }))
}

/// Matterbending Mage: when it enters, return up to one other target creature to its owner's hand: an opponent's
/// creature when it is the bigger threat than the enter trigger of yours is worth, else your best enter-the-battlefield
/// creature
fn mage_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != g.perm(src).owner {
        return Ok(());
    }
    let own = best_own_etb(g, p, Some(src), false);
    let opp = super::common::best_opp_creature(g, p, |x| x != src);
    let pick = match (own, opp) {
        (Some((a, s)), Some(b)) => {
            if pval(g, b) * 0.6 > s || s < 1.0 {
                b
            } else {
                a
            }
        }
        (Some((a, s)), None) if s >= 1.0 => a,
        (None, Some(b)) => b,
        _ => return Ok(()),
    };
    if trigger_window(g, p, Some(src), "return another creature to its owner's hand", Some(4.0))? && live(g, pick) {
        bounce(g, pick)?;
    }
    Ok(())
}

/// Whenever you cast a spell with {X} in its mana cost, this creature can't be blocked this turn
fn has_x(g: &Game, c: CardId) -> bool {
    g.db.get(c).tag(Tag::X) || cname(g, c) == "West Coast Expansion"
}

fn mage_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster != o || !has_x(g, c) || !live(g, src) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "can't be blocked this turn", Some(2.0))? && live(g, src) {
        let st = g.turn_stamp();
        g.perm_mut(src).data.set(DataKey::Unbl, Val::Stamp(st));
    }
    Ok(())
}

/// Rescue, Pepper Potts: when it enters, return up to one other target artifact or creature you control to its
/// owner's hand; if it was an artifact, a +1/+1 counter on Rescue
fn rescue_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != g.perm(src).owner {
        return Ok(());
    }
    let Some((t, s)) = best_own_etb(g, p, Some(src), true) else { return Ok(()) };
    if s < 0.5 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "return another artifact or creature you control to hand", Some(4.0))?
        && live(g, t)
    {
        let art = is_artifact(g, t);
        bounce(g, t)?;
        if art && live(g, src) {
            let k = counter_mult(g, src);
            crate::cardcode::add_counters(g, src, k);
        }
    }
    Ok(())
}

// ================================================================== Fear of Isolation
/// Fear of Isolation: as an additional cost, return a permanent you control to its owner's hand. The cost is taken as
/// the spell is cast: a creature worth re-entering, else a land that was tapped for the spell.
fn fear_cost(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if cname(g, c) != FEAR {
        return Ok(());
    }
    if let Some((m, s)) = best_own_etb(g, p, None, true)
        && s >= 4.0
        && live(g, m)
    {
        crate::glog!(g, "    {} returns {} (Fear of Isolation)", g.player(p).name, g.perm(m).name);
        return bounce(g, m);
    }
    // a tapped land first (its mana is already spent), then the cheapest permanent
    let lands = g.player(p).lands.clone();
    let pick = max_by(&lands, |l| {
        let x = g.land(l);
        (x.tapped as i32 * 4) as f64 - g.db.get(x.cd).tags.str(Tag::C).map_or(0, |s| s.chars().count()) as f64
    });
    if let Some(l) = pick {
        let cd = g.land(l).cd;
        crate::engine::turn::remove_land(g, p, l);
        g.player_mut(p).hand.push(cd);
        crate::glog!(g, "    {} returns {} (Fear of Isolation)", g.player(p).name, g.db.get(cd).name);
        return Ok(());
    }
    let cr: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.perm(m).cd.is_some() && !g.perm(m).is_cmd)
        .collect();
    if let Some(m) = max_by(&cr, |m| -(pval(g, m)) + etb_score(g, p, m)) {
        return bounce(g, m);
    }
    Ok(())
}

/// Fear of Isolation can't be cast without a permanent to return
fn fear_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let any = !pl.lands.is_empty() || pl.perms.iter().any(|&m| live(g, m) && g.perm(m).cd.is_some());
    // a spare land is cheap, but each return costs a land drop: only with lands to spare
    if !any || (pl.lands.len() < 4 && !my_creatures(g, p).iter().any(|&m| etb_score(g, p, m) >= 4.0)) {
        return 0;
    }
    generic_prio(g, p, c)
}

// ================================================================== the rest of the creatures
/// Mindseeker Oculus: when it enters, empower Jace 4
fn oculus_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && p == g.perm(src).owner && trigger_window(g, p, Some(src), "empower Jace 4", Some(3.0))? {
        super::alela::empower(g, p, 4);
    }
    Ok(())
}

/// the amount a creature of an opponent is weakened by Mouth of the Storm right now (`values::epow`)
pub fn weakened(g: &Game, m: PermId) -> i32 {
    if let Some(Val::List(v)) = g.perm(m).data.get(DataKey::Weaken)
        && let [Val::Player(q), Val::Int(t), Val::Int(n)] = v.as_slice()
        && g.player(*q).turns as i64 == *t
    {
        return *n as i32;
    }
    0
}

/// Mouth of the Storm: when it enters, creatures your opponents control get -3/-0 until your next turn
fn mouth_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != g.perm(src).owner {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "opponents' creatures get -3/-0 until your next turn", Some(4.0))? {
        return Ok(());
    }
    let turns = g.player(p).turns as i64;
    for q in g.opps(p).collect::<Vec<_>>() {
        for x in my_creatures(g, q) {
            let prior = weakened(g, x) as i64;
            g.perm_mut(x)
                .data
                .set(DataKey::Weaken, Val::List(vec![Val::Player(p), Val::Int(turns), Val::Int(prior + 3)]));
        }
    }
    Ok(())
}

/// Orysa costs {3} less to cast if creatures you control have total toughness 10 or greater
fn orysa_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let total: i32 = my_creatures(g, p).into_iter().map(|m| etgh(g, m).max(0)).sum();
    if total >= 10 { -3 } else { 0 }
}

/// Professor Hulk: whenever it deals combat damage to a player, draw that many cards
fn hulk_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if a != src || dmg <= 0 || p != g.perm(src).owner {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "draw cards equal to the damage", Some(5.0))? {
        draw(g, p, dmg as u32, false)?;
    }
    Ok(())
}

/// Starwinder: whenever a creature you control deals combat damage to a player, you may draw that many cards (the AI
/// does, unless that would draw its library nearly dry)
fn winder_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if dmg <= 0 || g.perm(a).owner != o || p != o || !live(g, src) {
        return Ok(());
    }
    if g.player(o).library.len() as i32 <= dmg + 3 {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "draw cards equal to the damage", Some(5.0))? {
        draw(g, o, dmg as u32, false)?;
    }
    Ok(())
}

/// Textbook Tabulator: increment, a +1/+1 counter when the mana spent on a spell you cast is greater than its power or
/// toughness (the lesser of the two)
fn tabulator_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster != o || !live(g, src) || g.perm(src).neutered {
        return Ok(());
    }
    let low = epow(g, src).min(etgh(g, src));
    if (mana_spent(g, o, c) as i32) <= low {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "increment: a +1/+1 counter", Some(2.0))? && live(g, src) {
        let k = counter_mult(g, src);
        crate::cardcode::add_counters(g, src, k);
    }
    Ok(())
}

/// The Wasp: when it enters, target Hero gains hexproof until end of turn (the Ring-bearer when it is a Hero, else
/// the Wasp)
fn wasp_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != g.perm(src).owner {
        return Ok(());
    }
    let heroes: Vec<PermId> = my_creatures(g, p)
        .into_iter()
        .filter(|&x| g.perm(x).cd.is_some_and(|c| g.db.get(c).has_subtype("hero")))
        .collect();
    let t = max_by(&heroes, |x| pval(g, x) + if g.player(p).ring_bearer == Some(x) { 5.0 } else { 0.0 }).unwrap_or(src);
    if trigger_window(g, p, Some(src), "a Hero gains hexproof until end of turn", Some(2.0))? && live(g, t) {
        super::partials::eot_kw(g, t, "hexproof");
    }
    Ok(())
}

/// The Wasp: whenever it attacks, tap target creature defending player controls
fn wasp_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) || p != g.perm(src).owner {
        return Ok(vec![]);
    }
    let cs: Vec<PermId> = my_creatures(g, d)
        .into_iter()
        .filter(|&x| !g.perm(x).tapped && !crate::engine::values::untargetable(g, x))
        .collect();
    let Some(t) = max_by(&cs, |x| pval(g, x)) else { return Ok(vec![]) };
    if trigger_window(g, p, Some(src), "tap target creature", Some(3.0))? && live(g, t) {
        g.perm_mut(t).tapped = true;
    }
    Ok(vec![])
}

/// Dune Mover: toxic 1 (a poison counter with the combat damage)
fn mover_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src && dmg > 0 && !crate::engine::values::melira(g, d) {
        g.player_mut(d).poison += 1;
        crate::glog!(g, "    {} gets a poison counter (toxic)", g.player(d).name);
    }
    Ok(())
}

// ================================================================== warp
/// the warp cost of the warp cards: (generic, pips)
fn warp_cost(name: &str) -> (u32, &'static str) {
    if name == WHALE { (1, "U") } else { (2, "UU") }
}

fn warped_key(name: &str) -> Sym {
    intern(&format!("elrond_warped_{name}"))
}

fn is_warper(g: &Game, c: CardId) -> bool {
    matches!(cname(g, c), WHALE | WINDER)
}

/// the warped creature is exiled at the beginning of the next end step; its card can be cast from exile on a later
/// turn
fn warp_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let c = g.perm(src).cd.unwrap();
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    {} (warp) is exiled", cname(g, c));
    exile_perm(g, src)?;
    let st = g.turn_stamp();
    let key = warped_key(cname(g, c));
    g.player_mut(o).flag_turn.insert(key, st);
    if !g.player(o).exile.contains(&c) {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

/// warp when the full cost is out of reach this turn (before combat), or when it makes the turn's second entry
fn warp_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) || !is_warper(g, c) {
        return Ok(vec![]);
    }
    let (wg, wp) = warp_cost(cname(g, c));
    if !can_pay(g, p, wg, wp, false) {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    let full = can_pay(g, p, cg, &cp, false);
    let second = elrond_of(g, p).is_some_and(|e| resolved_now(g, e) == 1);
    if !((!full && !post) || second) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if second { 6.0 } else { 2.5 },
        label: format!("{} (warp)", cname(g, c)),
        act: Some(Action::Plan { f: warp_go, arg: c.0 as i64 }),
    }])
}

fn warp_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (wg, wp) = warp_cost(cname(g, c));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, wg, wp, false) {
        return Ok(false);
    }
    pay(g, p, wg, wp, false)?;
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern("elrond_warp_cast"), st);
    let r = cast_card(g, p, c, "hand", Ctx::default());
    g.player_mut(p).flag_turn.shift_remove(&intern("elrond_warp_cast"));
    r?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

/// the warped creatures in exile: cast on a later turn for the mana cost
fn exile_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let mut out = vec![];
    if g.active != Some(p) {
        return out;
    }
    let st = g.turn_stamp();
    for c in g.player(p).exile.clone() {
        if !is_warper(g, c) || g.player(p).flag_turn.get(&warped_key(cname(g, c))) == Some(&st) {
            continue;
        }
        let (cg, cp) = cost_of(g, p, c);
        if can_pay(g, p, cg, &cp, false) {
            out.push(Opt {
                utility: 4.5,
                label: format!("{} (from exile)", cname(g, c)),
                act: Some(Action::Plan { f: exile_go, arg: c.0 as i64 }),
            });
        }
    }
    out
}

fn exile_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = cost_of(g, p, c);
    let Some(i) = g.player(p).exile.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    g.player_mut(p).exile.remove(i);
    let key = warped_key(cname(g, c));
    g.player_mut(p).flag_turn.shift_remove(&key);
    cast_card(g, p, c, "exile", Ctx::default())?;
    Ok(true)
}

// ================================================================== the AI plan
/// elrond's cast priority: Elrond first; with him out, creature spells before the rest and the flash creatures held
/// for the other players' end steps (or as the turn's second entry); Fear of Isolation only with something to return
pub fn elrond_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = generic_prio(g, p, c);
    let pl = g.player(p);
    if c == pl.cmd {
        if pl.turns < 2 {
            return 0;
        }
        if tempt_set(g, p).is_some_and(|s| s.contains(&c)) {
            return 98; // Elrond and the entries that tempt the Ring this turn
        }
        // alone he is exposed for a whole round: wait for a turn that tempts while another entry is in hand
        let spare = pl.hand.iter().any(|&x| entries_of(g, x) > 0 && cname(g, x) != FEAR);
        if spare && pl.turns < SOLO_TURN + 2 * pl.tax as u32 {
            return 0;
        }
        return if pl.tax == 0 { 97 } else { RECAST_PRIO };
    }
    let d = g.db.get(c);
    let name = &*d.name;
    let out = elrond_of(g, p);
    if name == FEAR {
        return fear_prio(g, p, c);
    }
    if d.creature || entries_of(g, c) > 0 {
        if let Some(s) = tempt_set(g, p)
            && s.contains(&c)
        {
            return 90;
        }
    }
    if FLASHERS.contains(&name) {
        return flash_hold(g, p, c);
    }
    let Some(e) = out else { return base };
    if !d.creature || base <= 0 {
        return base;
    }
    let n = resolved_now(g, e);
    if n < 2 && g.active == Some(p) {
        return base + 10; // creatures first: each entry counts toward the Ring
    }
    base
}

/// the flash creatures (the registry's `prio` slot, so the compiled-ability value never revives a held card): cast as
/// the second entry of a turn or at the end of another player's turn, else held; with Elrond out, a first entry on
/// another player's turn only when a second flash creature can follow
fn flash_hold(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let Some(e) = elrond_of(g, p) else { return 50 };
    let n = resolved_now(g, e);
    if g.active == Some(p) {
        return if n == 1 { 75 } else { 0 };
    }
    let spare = g.player(p).hand.iter().filter(|&&x| x != c && FLASHERS.contains(&cname(g, x))).count();
    if n >= 1 {
        80
    } else if spare > 0 {
        60
    } else {
        45
    }
}

/// elrond's own plays: warp, and the warped creatures from exile
pub fn elrond_special(g: &mut Game, p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    Ok(exile_options(g, p))
}

/// removal aimed at Elrond (or the Ring-bearer, or another creature worth 6 or more): counter the removal spell, else
/// bounce the creature with Unsummon or Whoosh! (not against bounce itself, and not against a wipe or an edict)
pub fn elrond_protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: &str,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor.is_none() || actor == Some(p) || g.perm(m).cd.is_none() || !g.is_creature(m) {
        return Ok(false);
    }
    if matches!(kind, "edict" | "wipe" | "sacrifice") {
        return Ok(false);
    }
    let is_elrond = card_name(g, m) == Some(ELROND);
    let bearer = crate::impls::mine::ring_bearer(g, p) == Some(m);
    if !is_elrond && !bearer && pval(g, m) < 6.0 {
        return Ok(false);
    }
    if let Some(s) = spell
        && let Some(ctr) = pick_counter(g, p, s)
        && cast_counter(g, p, ctr, Some(s))?
    {
        crate::glog!(g, "    {} counters {}", g.player(p).name, cname(g, s));
        return Ok(true);
    }
    if kind == "bounce" {
        return Ok(false);
    }
    for name in ["Unsummon", "Whoosh!"] {
        let Some(c) = in_hand(g, p, name) else { continue };
        let (cg, cp) = cost_of(g, p, c);
        if !can_pay(g, p, cg, &cp, false) {
            continue;
        }
        pay(g, p, cg, &cp, false)?;
        let pl = g.player_mut(p);
        let i = pl.hand.iter().position(|&x| x == c).unwrap();
        pl.hand.remove(i);
        pl.spells_this_turn += 1;
        pl.stat("spells_cast", 1);
        pl.cast_names.insert(c);
        crate::glog!(g, "  {} casts {} on {}", g.player(p).name, name, pname(g, m));
        crate::engine::cast::on_cast(g, p, c)?;
        let ok = counter_window(g, p, c, 6.0, vec![])?;
        g.player_mut(p).gy.push(c);
        if ok && live(g, m) {
            bounce(g, m)?;
        }
        return Ok(ok);
    }
    Ok(false)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, ELROND)?;
    c.etb = Some(elrond_etb);
    c.tokens_enter = Some(elrond_tokens);
    let c = r.card(db, CLOUDFIN)?;
    c.etb = Some(cloudfin_etb);
    c.tokens_enter = Some(cloudfin_tokens);
    r.card(db, DEFILER)?.cast = Some(defiler_cast);
    let c = r.card(db, VIRTUOSO)?;
    c.etb = Some(virtuoso_etb);
    c.cast = Some(virtuoso_cast);
    r.card(db, WIT)?.resolve = Some(wit_resolve);
    let c = r.card(db, MAGE)?;
    c.etb = Some(mage_etb);
    c.cast = Some(mage_cast);
    r.card(db, OCULUS)?.etb = Some(oculus_etb);
    r.card(db, MOUTH)?.etb = Some(mouth_etb);
    r.card(db, ORYSA)?.self_cost = Some(orysa_cost);
    r.card(db, HULK)?.combat_damage = Some(hulk_damage);
    r.card(db, RESCUE)?.etb = Some(rescue_etb);
    let c = r.card(db, WHALE)?;
    c.end_step = Some(warp_end);
    c.hand_options = Some(warp_hand);
    let c = r.card(db, WINDER)?;
    c.end_step = Some(warp_end);
    c.hand_options = Some(warp_hand);
    c.combat_damage = Some(winder_damage);
    for n in FLASHERS {
        r.card(db, n)?.prio = Some(flash_hold);
    }
    r.card(db, TABULATOR)?.cast = Some(tabulator_cast);
    let c = r.card(db, WASP)?;
    c.etb = Some(wasp_etb);
    c.attack = Some(wasp_attack);
    r.card(db, MOVER)?.combat_damage = Some(mover_damage);
    let c = r.card(db, FEAR)?;
    c.self_cast = Some(fear_cost);
    Ok(())
}

// ================================================================== search evaluation
/// what the look-ahead search adds to the elrond seat's position score: Elrond on the battlefield, and each Ring tempt
/// so far (capped, the Ring stops growing at level 4), so a play that makes a second entry beats one that doesn't
pub fn progress(g: &Game, q: PlayerId) -> f64 {
    let pl = g.player(q);
    let tempts = pl.stats.get("ring_tempts").copied().unwrap_or(0).min(6) as f64;
    let e = if elrond_of(g, q).is_some() { PROGRESS_ELROND } else { 0.0 };
    e + PROGRESS_TEMPT * tempts
}
const PROGRESS_ELROND: f64 = 4.0;
const PROGRESS_TEMPT: f64 = 3.0;

// ================================================================== the entry planner
/// how many creature entries (for Elrond's trigger) casting this card makes
fn entries_of(g: &Game, c: CardId) -> u32 {
    let d = g.db.get(c);
    let name = &*d.name;
    if name == "Coral Barrier" {
        return 2;
    }
    if d.creature {
        return 1;
    }
    if matches!(name, "Muse's Encouragement" | "Retrieve the Esper" | "Unfathomable Truths") {
        return 1;
    }
    0
}

/// the cheapest set of (at most three) entry cards that makes `need` entries within `mana`: Elrond (not out yet) must be
/// in it. Returns the cards.
fn entry_set(g: &Game, p: PlayerId, need: u32, mana: u32, elrond_out: bool) -> Option<Vec<CardId>> {
    let pl = g.player(p);
    let mut cards: Vec<(CardId, u32, u32)> = Vec::new();
    let mut add = |c: CardId| {
        let n = entries_of(g, c);
        if n == 0 || cname(g, c) == FEAR {
            return;
        }
        let (gn, pips) = cost_of(g, p, c);
        cards.push((c, gn + pips.chars().count() as u32, n));
    };
    for &c in &pl.hand {
        add(c);
    }
    if pl.cmd_in_zone && !elrond_out {
        add(pl.cmd);
    }
    let k = cards.len();
    let mut best: Option<(u32, Vec<CardId>)> = None;
    for mask in 1u32..(1 << k) {
        if mask.count_ones() > 3 {
            continue;
        }
        let (mut cost, mut ent, mut has_e) = (0, 0, elrond_out);
        for (i, &(c, cc, n)) in cards.iter().enumerate() {
            if mask >> i & 1 == 1 {
                cost += cc;
                ent += n;
                has_e |= c == pl.cmd;
            }
        }
        if ent < need || cost > mana || !has_e {
            continue;
        }
        if best.as_ref().is_none_or(|(bc, _)| cost < *bc) {
            best = Some((cost, cards.iter().enumerate().filter(|(i, _)| mask >> i & 1 == 1).map(|(_, x)| x.0).collect()));
        }
    }
    best.map(|b| b.1)
}

/// my turn: the plan to make the Ring tempt this turn, if mana allows
fn tempt_set(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    if g.active != Some(p) {
        return None;
    }
    let out = elrond_of(g, p);
    let n = out.map_or(0, |e| resolved_now(g, e)) as u32;
    if n >= 2 {
        return None;
    }
    entry_set(g, p, 2 - n, crate::engine::mana::total_mana(g, p, false), out.is_some())
}
const RECAST_PRIO: i32 = 45;
const SOLO_TURN: u32 = 6;
