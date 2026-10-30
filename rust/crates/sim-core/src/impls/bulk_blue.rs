//! Card code for the blue spells (keyed by card name; no deck owns it).
//! Written by the blue-spells agent (rust/agents/archive/blue-spells.md).
//!
//! - Jace, Reality Sculptor (walker framework + the empower Jace token of `alela.rs`).
//! - Token copies: Relm's Sketching, Echocasting Symposium (paradigm through the engine's rebound step).
//! - Spree, kicker, modes: Phantom Interference, Whoosh!, Amazing Acrobatics, Icy Reception.
//! - Top or bottom: Sudden Setback, Vanish from Sight (the owner's choice: `top_or_bottom`).
//! - Kasmina's Transmutation (a zur.rs lock Aura), Rousing Read (an Aura), Retrieve the Esper, Unfathomable Truths,
//!   West Coast Expansion, Sleight of Hand.
//!
//! Engine hooks into this file (each a one-line call): stack.rs `spree_extra`/`spirit_token`/`acrobatics_tap`,
//! cast.rs `spell_to_library`, removal.rs `top_or_bottom`, zur.rs `kasmina_attach`, common.rs `kasmina_detach`.

use super::common::{AURA, AuraSpec, WalkerAb, aura, eot, walker};
use super::galadriel::blink_value;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, cast_spell_copy, castable, discard_worst, on_cast};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{card_name, epow, pval, untargetable};
use crate::engine::zones::{Tokens, Zone, draw, enter_token_copy, leave, make_tokens, max_by, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Opt, PlanFn, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

pub const JACE_RS: &str = "Jace, Reality Sculptor";

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}
// ================================================================== Jace, Reality Sculptor
/// the Jace planeswalkers p controls (the Jace token and Jace, Reality Sculptor)
fn jaces(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| !g.perm(m).phased && g.perm(m).on_bf && card_name(g, m).is_some_and(|n| n.starts_with("Jace")))
        .collect()
}

/// loyalty counters among the Jaces p controls
fn jace_loyalty(g: &Game, p: PlayerId) -> i32 {
    jaces(g, p).iter().map(|&m| g.perm(m).loyalty.unwrap_or(0)).sum()
}

/// Islands p controls (basic Islands and anything with the Island type)
fn islands(g: &Game, p: PlayerId) -> i32 {
    g.player(p)
        .lands
        .iter()
        .filter(|&&l| {
            let d = g.db.get(g.land(l).card());
            &*d.name == "Island" || d.has_subtype("island")
        })
        .count() as i32
}

/// +1: empower Jace X, X = Islands you control (the Jace token gets X loyalty counters)
fn jace_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let x = islands(g, p);
    Some(1.5 + (x as f64 / 3.0).min(3.0))
}

fn jace_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let x = islands(g, p);
    if x > 0 {
        super::alela::empower(g, p, x);
    }
    Ok(())
}

/// -3: until your next turn, creatures attacking you get -5/-0. Worth it when an opponent has a real attack and
/// there is loyalty to spare.
fn jace_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if g.perm(src).loyalty.unwrap_or(0) <= 3 {
        return None;
    }
    let big: i32 = g
        .opps(p)
        .map(|q| {
            g.player(q)
                .perms
                .iter()
                .filter(|&&m| g.is_creature(m) && !g.perm(m).phased)
                .map(|&m| crate::engine::values::epow(g, m).min(5))
                .sum::<i32>()
        })
        .sum();
    if big < 8 {
        return None;
    }
    Some(1.0 + big as f64 / 8.0)
}

fn jace_minus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    g.perm_mut(src).data.set(DataKey::Used, Val::Bool(true));
    crate::glog!(g, "    until {}'s next turn, creatures attacking them get -5/-0", g.player(p).name);
    Ok(())
}

/// 0: exile all but the bottom card of each opponent's library; only with 25 or more loyalty counters among your Jaces
fn jace_zero_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    (jace_loyalty(g, p) >= 25).then_some(30.0)
}

fn jace_zero(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if jace_loyalty(g, p) < 25 {
        return Ok(()); // the activation condition is checked again as it resolves
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let pl = g.player_mut(q);
        let rest: Vec<_> = pl.library.drain(1.min(pl.library.len())..).collect();
        crate::glog!(g, "    exiles {} cards from {}'s library", rest.len(), g.player(q).name);
        g.player_mut(q).exile.extend(rest);
    }
    Ok(())
}

static JACE_RS_ABILITIES: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "empower Jace X (X = Islands)", val: jace_plus_val, eff: jace_plus },
    WalkerAb { delta: -3, label: "attackers get -5/-0 until your next turn", val: jace_minus_val, eff: jace_minus },
    WalkerAb {
        delta: 0,
        label: "exile all but the bottom card of each opponent's library",
        val: jace_zero_val,
        eff: jace_zero,
    },
];

/// Jace's -3 lasts until your next turn
fn jace_upkeep(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if g.perm(src).owner == p {
        g.perm_mut(src).data.set(DataKey::Used, Val::Bool(false));
    }
    Ok(())
}

/// the -3: a creature attacking the Jace's controller gets -5/-0 until end of turn
fn jace_attack(g: &mut Game, src: PermId, _p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == d && g.perm(src).on_bf && g.perm(src).data.truthy(DataKey::Used) {
        for &a in atk {
            if g.perm(a).on_bf {
                super::common::eot(g, a, -5, 0);
            }
        }
    }
    Ok(vec![])
}

// ================================================================== helpers
fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn plan(utility: f64, label: String, f: PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// (card, permanent) in one plan argument; 0xFFFF_FFFF: no permanent
const NO_PERM: u32 = u32::MAX;

fn pack(c: CardId, m: Option<PermId>) -> i64 {
    ((c.0 as i64) << 32) | m.map_or(NO_PERM, |m| m.0) as i64
}

fn unpack(a: i64) -> (CardId, Option<PermId>) {
    let m = (a & 0xFFFF_FFFF) as u32;
    (CardId((a >> 32) as u16), (m != NO_PERM).then_some(PermId(m)))
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// pay for card c (cost plus `extra` generic), take it out of the hand and cast it with ctx: false if it can't be
/// done now (the shape of act.rs's `face`)
fn cast_paid(g: &mut Game, p: PlayerId, c: CardId, extra: u32, extra_pips: &str, ctx: Ctx) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    let pips = format!("{pips}{extra_pips}");
    if !can_pay(g, p, gn + extra, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + extra, &pips, false)?;
    cast_card(g, p, c, "hand", ctx)?;
    Ok(true)
}

/// does the owner of card c want it back on top of the library (a spell it can cast soon) rather than on the bottom?
fn wants_back(g: &Game, owner: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if d.land {
        return g.player(owner).lands.len() < 5;
    }
    let lands = g.player(owner).lands.len() as u32;
    d.cmc <= lands + 2 && card_worth(g, owner, c, false) >= 45.0
}

/// the card goes into its owner's library: on top, or (choose: its owner decides) on the bottom
pub fn spell_to_library(g: &mut Game, o: PlayerId, c: CardId, choose: bool) {
    let top = !choose || wants_back(g, o, c);
    if top {
        g.player_mut(o).library.push(c);
    } else {
        g.player_mut(o).library.insert(0, c);
    }
}

/// "the owner puts it on their choice of the top or bottom of their library" (a permanent): tokens cease to exist and
/// a commander goes to the command zone. The AI owner takes the top for a card it wants back and the bottom
/// otherwise (a land only while it is short of them): a tuck that costs the owner a draw step for a bad card is
/// the owner's best answer.
pub fn top_or_bottom(g: &mut Game, m: PermId) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    let owner = g.perm(m).orig;
    let top = g.perm(m).cd.is_some_and(|c| !g.perm(m).token && wants_back(g, owner, c));
    leave(g, m)?;
    let before = g.player(owner).library.len();
    to_zone_card(g, m, Zone::Top);
    if !top && g.player(owner).library.len() > before {
        let c = g.player_mut(owner).library.pop().unwrap();
        g.player_mut(owner).library.insert(0, c);
    }
    crate::glog!(g, "    {} goes on the {} of its owner's library", g.perm(m).name, if top { "top" } else { "bottom" });
    Ok(())
}

/// a 2/2 white Spirit creature token with flying
pub fn spirit_token(g: &mut Game, p: PlayerId) -> Res {
    let spec = Tokens {
        tgh: Some(2),
        fly: true,
        color: Some(Colors::from_letters("W")),
        types: vec!["spirit"],
        ..Tokens::new(1, 2)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// Phantom Interference cast as a counterspell ({1}{U}) with {3} to spare: the Spirit mode too (spree). Pays the
/// extra cost; true if it did.
pub fn spree_extra(g: &mut Game, q: PlayerId, ctr: CardId) -> Res<bool> {
    if &*g.db.get(ctr).name != "Phantom Interference" || !can_pay(g, q, 3, "", false) {
        return Ok(false);
    }
    pay(g, q, 3, "", false)?;
    crate::glog!(g, "    {} also chooses the Spirit mode", pname(g, q));
    Ok(true)
}

/// Amazing Acrobatics countered a spell: its other mode taps one or two target creatures (opposing ones: the biggest
/// that are untapped)
pub fn acrobatics_tap(g: &mut Game, q: PlayerId) -> Res {
    tap_best(g, q, 2);
    Ok(())
}

fn tap_best(g: &mut Game, q: PlayerId, n: usize) -> usize {
    let mut cs: Vec<PermId> = g
        .opps(q)
        .flat_map(|o| g.player(o).perms.iter().copied().collect::<Vec<_>>())
        .filter(|&m| g.is_creature(m) && on_bf(g, m) && !g.perm(m).tapped && !untargetable(g, m))
        .collect();
    cs.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    cs.truncate(n);
    for &m in &cs {
        g.perm_mut(m).tapped = true;
        crate::glog!(g, "    {} is tapped", g.perm(m).name);
    }
    cs.len()
}

// ================================================================== Kasmina's Transmutation
/// Kasmina's Transmutation `a` enchants `host` (zur::lock_attach calls this): it loses all abilities and has base
/// power and toughness 1/1 (counters and other effects still count). The creature's old state is kept on the Aura.
pub fn kasmina_attach(g: &mut Game, a: PermId, host: PermId) {
    let x = g.perm(host);
    let saved = vec![
        Val::Int(x.pow as i64),
        Val::Int(x.tgh as i64),
        Val::Bool(x.fly),
        Val::Bool(x.dt),
        Val::Bool(x.vig),
        Val::Bool(x.lifelink),
        Val::Bool(x.neutered),
    ];
    g.perm_mut(a).data.set(DataKey::Held, Val::List(saved));
    let x = g.perm_mut(host);
    (x.pow, x.tgh) = (1, 1);
    (x.fly, x.dt, x.vig, x.lifelink, x.neutered) = (false, false, false, false, true);
    g.bf_ver += 1;
}

/// Kasmina's Transmutation `a` left the battlefield: its creature is itself again
pub fn kasmina_detach(g: &mut Game, a: PermId) {
    if g.perm(a).cd.is_none_or(|c| &*g.db.get(c).name != "Kasmina's Transmutation") {
        return;
    }
    let Some(Val::List(v)) = g.perm(a).data.get(DataKey::Held).cloned() else { return };
    g.perm_mut(a).data.set(DataKey::Held, Val::None);
    let Some(host) = g.perm(a).attached.filter(|&h| g.perm(h).on_bf) else { return };
    let int = |i: usize| if let Some(Val::Int(n)) = v.get(i) { *n as i32 } else { 0 };
    let flag = |i: usize| matches!(v.get(i), Some(Val::Bool(true)));
    let x = g.perm_mut(host);
    (x.pow, x.tgh) = (int(0), int(1));
    (x.fly, x.dt, x.vig, x.lifelink, x.neutered) = (flag(2), flag(3), flag(4), flag(5), flag(6));
    g.bf_ver += 1;
}

// ================================================================== token copies
/// how much a token copy of permanent m is worth to p: a creature by its value and its enter trigger (the Ring
/// tempts on the second creature entering in a turn), an artifact by its value
fn copy_value(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let Some(cd) = g.perm(m).cd else { return 0.0 };
    let d = g.db.get(cd);
    if super::garland::is_garland(g, m) && g.perm(m).owner == p && g.player(p).key == super::garland::KEY {
        return super::garland::recrown_value(g, p) + 1.0; // a copy of Garland enters and crowns again; the legend rule keeps one
    }
    if d.tag(Tag::Leg) || !on_bf(g, m) || (g.perm(m).owner != p && untargetable(g, m)) {
        return 0.0; // the legend rule would put one of them straight back
    }
    if g.is_creature(m) {
        pval(g, m) + 0.5 * crate::ai::card_etb_value(g, p, cd) + 1.0
    } else if d.types.has(Types::ARTIFACT) {
        pval(g, m).min(5.0)
    } else {
        0.0
    }
}

/// the best permanent for a token copy: (permanent, value); own_creatures: Echocasting Symposium's target
fn best_copy(g: &Game, p: PlayerId, own_creatures: bool) -> Option<(PermId, f64)> {
    let mut best: Option<(PermId, f64)> = None;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id) {
        if own_creatures && q != p {
            continue;
        }
        for &m in &g.player(q).perms {
            if own_creatures && !g.is_creature(m) {
                continue;
            }
            let v = copy_value(g, p, m);
            if v > 0.0 && best.is_none_or(|b| v > b.1) {
                best = Some((m, v));
            }
        }
    }
    best
}

/// the land worth copying (a basic or any land of p's own): the extra mana is worth about a card early on
fn land_copy_value(g: &Game, p: PlayerId) -> f64 {
    let n = g.player(p).lands.len();
    if n == 0 || n >= 9 { 0.0 } else { 3.0 }
}

fn token_copy(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    crate::glog!(g, "    {} creates a token copy of {}", pname(g, p), g.perm(m).name);
    let tok = enter_token_copy(g, p, cd)?;
    if let Some(t) = tok.filter(|&t| super::garland::is_my_garland(g, t) && super::garland::garland_of(g, p).is_some_and(|o| o != t)) {
        // the legend rule: one Garland stays (the card); his enter trigger has been put on the stack by the copy
        crate::engine::zones::leave(g, t)?;
    }
    Ok(())
}

/// Relm's Sketching: a token copy of target artifact, creature or land (the target chosen as it resolves when none was
/// named: the best creature or artifact on the table, else one of our lands)
fn relm_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let named = ctx.target.filter(|&t| copy_value(g, p, t) > 0.0);
    match named.map(|t| (t, 1.0)).or_else(|| best_copy(g, p, false)) {
        Some((t, v)) if v >= land_copy_value(g, p) || named.is_some() => token_copy(g, p, t)?,
        _ => {
            let ls: Vec<_> = g.player(p).lands.clone();
            if let Some(&l) = ls.last() {
                let cd = g.land(l).card();
                crate::glog!(g, "    {} creates a token copy of {}", pname(g, p), g.db.get(cd).name);
                g.add_land(p, cd, false);
            }
        }
    }
    Ok("gy")
}

fn relm_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let v = best_copy(g, p, false).map_or(0.0, |b| b.1).max(land_copy_value(g, p) - 0.5);
    if v >= 3.0 { 55 } else { 0 }
}

/// Echocasting Symposium: target player (the caster) creates a token copy of target creature they control. Paradigm:
/// the spell is exiled as it resolves (once), and at the start of each of its controller's turns (the engine's
/// rebound step, a little before the first main phase) a copy of it is cast for free.
fn echo_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let para = ctx.mode == Some("para");
    let who = ctx.face.filter(|&q| g.player(q).alive).unwrap_or(p);
    let named = ctx.target.filter(|&t| g.perm(t).owner == p && g.is_creature(t) && copy_value(g, p, t) > 0.0);
    if let Some(t) = named.or_else(|| best_copy(g, p, true).map(|b| b.0)) {
        token_copy(g, who, t)?;
    }
    if para {
        return Ok("gy"); // a copy: no card moves
    }
    g.player_mut(p).rebound.push(c);
    crate::glog!(g, "    {} exiles Echocasting Symposium (paradigm)", pname(g, p));
    Ok("exile")
}

fn echo_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if best_copy(g, p, true).is_some_and(|b| b.1 >= 4.0) { 54 } else { 0 }
}

/// the paradigm: cast a copy of the exiled spell for free (it stays in exile and comes back next turn)
fn echo_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    g.player_mut(p).rebound.push(c);
    let Some((t, v)) = best_copy(g, p, true) else { return Ok(()) };
    if v < 2.0 {
        return Ok(()); // "you may": nothing worth copying
    }
    crate::glog!(g, "  {} casts a copy of Echocasting Symposium (paradigm)", pname(g, p));
    cast_spell_copy(g, p, c, Some(&Ctx { target: Some(t), mode: Some("para"), face: Some(p), ..Ctx::default() }))?;
    Ok(())
}

// ================================================================== spree, kicker and modes
/// Phantom Interference: the Spirit mode ({3}); its counter mode ({1}, unless the controller pays {2}) is cast through
/// the engine's counterspell code (`ctr` tag; stack.rs `counter_cost` and `spree_extra` add both modes in response)
fn phantom_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    spirit_token(g, p)?;
    Ok("gy")
}

/// Phantom Interference's Spirit mode at the end of an opponent's turn: {3}{U} for a 2/2 flier (a second creature for
/// Elrond's trigger when another entered)
fn phantom_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (gn, pips) = cost_of(g, p, c);
    if post.is_some() || !can_pay(g, p, gn + 3, &pips, false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(2.6, "Phantom Interference (Spirit)".into(), phantom_go, pack(c, None))])
}

fn phantom_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, _) = unpack(arg);
    cast_paid(g, p, c, 3, "", Ctx { mode: Some("spirit"), ..Ctx::default() })
}

/// Whoosh!: return target nonland permanent to its owner's hand; kicked ({1}{U}), draw a card
fn whoosh_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "bounce", Some(c))?;
        if ctx.mode == Some("kick") {
            draw(g, p, 1, false)?;
        }
    }
    Ok("gy")
}

/// Whoosh!'s own plays: at the end of an opponent's turn, bounce our creature with the best enter trigger (and draw if
/// the mana is there to kick it); with spare mana after combat, the kicked spell on the best opposing permanent
fn whoosh_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (gn, pips) = cost_of(g, p, c);
    let mut out = vec![];
    let kick_ok = can_pay(g, p, gn + 1, &format!("{pips}U"), false);
    if post.is_none() && g.active != Some(p) && can_pay(g, p, gn, &pips, false) {
        let mine: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && on_bf(g, m) && !g.perm(m).token)
            .collect();
        if let Some(t) = max_by(&mine, |m| blink_value(g, p, m))
            && blink_value(g, p, t) >= 3.0
        {
            let v = blink_value(g, p, t) - 1.0 + if kick_ok { 1.5 } else { 0.0 };
            out.push(plan(v, format!("Whoosh! (bounce {})", g.perm(t).name), whoosh_go, pack(c, Some(t))));
        }
    }
    if post == Some(true)
        && can_pay(g, p, gn, &pips, false)
        && let Some(m) = super::garland::own_turn_recrown(g, p, gn + pips.len() as u32)
    {
        // Garland's crown has stalled: bounce him after combat and cast him again (no kicker: the mana is for Garland)
        out.push(plan(super::garland::recrown_value(g, p), "Whoosh! (bounce Garland, crown stalled)".into(), whoosh_go, pack(c, Some(m))));
    }
    if post == Some(true) && kick_ok {
        let tg = legal_targets(g, p, "bounce", "nl", false, Some(c));
        if let Some(b) = max_by(&tg, |m| pval(g, m))
            && pval(g, b) >= 4.0
        {
            out.push(plan(
                pval(g, b) - 2.5,
                format!("Whoosh! (kicked) -> {}", g.perm(b).name),
                whoosh_go,
                pack(c, Some(b)),
            ));
        }
    }
    Ok(out)
}

fn whoosh_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let Some(t) = t.filter(|&t| g.perm(t).on_bf && (g.perm(t).owner == p || !untargetable(g, t))) else {
        return Ok(false);
    };
    let (gn, pips) = cost_of(g, p, c);
    let spare = g.active == Some(p) && super::garland::own_turn_recrown(g, p, gn + 2 + pips.len() as u32).is_none() && g.player(p).key == super::garland::KEY && g.perm(t).owner == p && super::garland::is_my_garland(g, t);
    let kick = !spare && can_pay(g, p, gn + 1, &format!("{pips}U"), false);
    let ctx = Ctx { target: Some(t), mode: kick.then_some("kick"), ..Ctx::default() };
    cast_paid(g, p, c, if kick { 1 } else { 0 }, if kick { "U" } else { "" }, ctx)
}

/// Amazing Acrobatics' tap mode as the attacker: tap the defender's two best untapped blockers when the attack is big
/// enough to matter. (The counter mode: the `ctr` tag; with a spell countered, `acrobatics_tap` taps two creatures
/// too.)
fn acrobatics_attack(g: &mut Game, c: CardId, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res {
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(());
    }
    let power: i32 = atk.iter().filter(|&&a| on_bf(g, a)).map(|&a| epow(g, a)).sum();
    let blockers: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && on_bf(g, m) && !g.perm(m).tapped && !untargetable(g, m))
        .collect();
    let lethal = power >= g.player(d).life;
    if blockers.is_empty() || (!lethal && (power < 8 || blockers.len() < 2)) {
        return Ok(());
    }
    pay(g, p, gn, &pips, false)?;
    remove_card(&mut g.player_mut(p).hand, c);
    crate::glog!(g, "  {} casts Amazing Acrobatics: taps blockers", pname(g, p));
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 4.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(());
    }
    let mut bs = blockers;
    bs.sort_by(|&a, &b| pval(g, b).total_cmp(&pval(g, a)));
    for &m in bs.iter().take(2) {
        if g.perm(m).on_bf {
            g.perm_mut(m).tapped = true;
            crate::glog!(g, "    {} is tapped", g.perm(m).name);
        }
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}

/// Icy Reception's second mode as the defender: -5/-0 on an unblocked attacker whose damage matters
fn icy_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let open: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&a| on_bf(g, a) && g.perm(a).owner == p && !assign.iter().any(|x| x.0 == a))
        .collect();
    let dmg: i32 = open.iter().map(|&a| epow(g, a)).sum();
    let Some(big) = max_by(&open, |a| epow(g, a) as f64) else { return Ok(()) };
    let saved = epow(g, big).min(5);
    let life = g.player(d).life;
    let worth = (dmg >= life && dmg - saved < life) || (saved >= 5 && dmg * 3 >= life);
    if !worth {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    remove_card(&mut g.player_mut(d).hand, c);
    crate::glog!(g, "  {} casts Icy Reception: {} gets -5/-0", pname(g, d), g.perm(big).name);
    on_cast(g, d, c)?;
    if !counter_window(g, d, c, 4.0, vec![])? {
        g.player_mut(d).gy.push(c);
        return Ok(());
    }
    if g.perm(big).on_bf {
        eot(g, big, -5, 0);
    }
    g.player_mut(d).gy.push(c);
    Ok(())
}

// ================================================================== top or bottom
/// Vanish from Sight: target nonland permanent's owner puts it on their choice of the top or bottom of their library,
/// then surveil 1. (Sudden Setback needs no code of its own: `rem=topbot` for the permanent, and `ctr`/`lapse` for a
/// spell: stack.rs/cast.rs ask `spell_to_library`.)
fn vanish_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") }; // no legal target: it fizzles
    apply_removal(g, Some(p), t, "topbot", Some(c))?;
    crate::cardcode::scry(g, p, 1, true)?;
    Ok("gy")
}

// ================================================================== the rest
/// Retrieve the Esper: a 3/3 blue Robot Warrior artifact creature token; cast from a graveyard, two +1/+1 counters
/// on it
fn retrieve_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let mut data = PermData::default();
    data.set(DataKey::Artifact, Val::Bool(true));
    let spec = Tokens {
        tgh: Some(3),
        warrior: true,
        color: Some(Colors::from_letters("U")),
        types: vec!["robot", "warrior"],
        data,
        ..Tokens::new(1, 3)
    };
    for t in make_tokens(g, p, spec)? {
        if ctx.from_gy {
            crate::cardcode::put_plus(g, t, 2);
        }
    }
    Ok("gy")
}

fn retrieve_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).turns >= 2 { 58 } else { 0 }
}

/// Rousing Read: when the Aura enters, draw two cards then discard a card (the +1/+1 and flying are its AuraSpec)
fn rousing_etb(g: &mut Game, p: PlayerId, a: PermId, _host: PermId) -> Res {
    if trigger_window(g, p, Some(a), "draw two, discard one", None)? {
        draw(g, p, 2, false)?;
        discard_worst(g, p, 1)?;
    }
    Ok(())
}

static ROUSING: AuraSpec = AuraSpec { pow: 1, tgh: 1, kws: &["flying"], on_etb: Some(rousing_etb), ..AURA };

/// Unfathomable Truths (devoid: colourless, which an instant never shows on the battlefield or to anything the
/// decks run): draw three cards and create a 0/1 colourless Eldrazi Spawn token
fn truths_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    draw(g, p, 3, false)?;
    let spec = Tokens { tgh: Some(1), color: Some(Colors::NONE), types: vec!["eldrazi", "spawn"], ..Tokens::new(1, 0) };
    make_tokens(g, p, spec)?;
    Ok("gy")
}

/// West Coast Expansion: draw X cards; with X 5 or more, cast the best Hero spell from the hand free
fn west_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    draw(g, p, ctx.x.max(0) as u32, false)?;
    if ctx.x >= 5 {
        let heroes: Vec<CardId> = g
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|&h| g.db.get(h).has_subtype("hero") && !g.db.get(h).land && castable(g, p, h, "hand"))
            .collect();
        if let Some(h) = max_by(&heroes, |h| g.db.get(h).cmc as f64 * 10.0 + card_worth(g, p, h, false) / 10.0) {
            crate::glog!(g, "    West Coast Expansion casts {} free", g.db.get(h).name);
            cast_card(g, p, h, "hand", Ctx::default())?;
        }
    }
    Ok("gy")
}

/// West Coast Expansion's X: all the spare mana in the main phase once X is worth it (three cards, or two on a
/// nearly empty hand; a Hero in the hand pulls X = 5 within reach)
fn west_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) {
        return Ok(vec![]);
    }
    let total = total_mana(g, p, false) as i32;
    let x = total - 2;
    let hand = g.player(p).hand.len() as i32;
    if x < 2 || (x < 3 && hand > 2) || !can_pay(g, p, x as u32, "UU", false) {
        return Ok(vec![]);
    }
    let hero = g.player(p).hand.iter().any(|&h| g.db.get(h).has_subtype("hero") && !g.db.get(h).land);
    let bonus = if x >= 5 && hero { 2.0 } else { 0.0 };
    Ok(vec![plan(
        1.5 + 0.7 * x as f64 + bonus,
        format!("West Coast Expansion X={x}"),
        west_go,
        pack(c, None) | (x as i64) << 48,
    )])
}

fn west_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let x = (arg >> 48) as i32;
    let (c, _) = unpack(arg & 0xFFFF_FFFF_FFFF);
    if x < 0 || !g.player(p).hand.contains(&c) || !can_pay(g, p, x as u32, "UU", false) {
        return Ok(false);
    }
    cast_paid(g, p, c, x as u32, "", Ctx { x, ..Ctx::default() })
}

/// Sleight of Hand: look at the top two cards of the library, the better one into the hand, the other to the bottom
fn sleight_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.player(p).library.len();
    if n == 0 {
        return Ok("gy");
    }
    if n == 1 {
        draw(g, p, 1, false)?;
        return Ok("gy");
    }
    let (a, b) = (g.player(p).library[n - 1], g.player(p).library[n - 2]);
    let keep = if card_worth(g, p, b, false) > card_worth(g, p, a, false) { b } else { a };
    let other = if keep == a { b } else { a };
    g.player_mut(p).library.truncate(n - 2);
    g.player_mut(p).library.insert(0, other);
    g.player_mut(p).hand.push(keep);
    crate::glog!(g, "    {} keeps {} and bottoms the other", pname(g, p), g.db.get(keep).name);
    Ok("gy")
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    walker(r, db, JACE_RS, &JACE_RS_ABILITIES)?;
    let c = r.card(db, JACE_RS)?;
    c.upkeep = Some(jace_upkeep);
    c.attack = Some(jace_attack);
    let c = r.card(db, "Relm's Sketching")?;
    c.resolve = Some(relm_resolve);
    c.prio = Some(relm_prio);
    let c = r.card(db, "Echocasting Symposium")?;
    c.resolve = Some(echo_resolve);
    c.prio = Some(echo_prio);
    c.rebound = Some(echo_rebound);
    let c = r.card(db, "Phantom Interference")?;
    c.resolve = Some(phantom_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(phantom_options);
    let c = r.card(db, "Whoosh!")?;
    c.resolve = Some(whoosh_resolve);
    c.hand_options = Some(whoosh_options);
    r.card(db, "Amazing Acrobatics")?.hand_attack = Some(acrobatics_attack);
    r.card(db, "Icy Reception")?.hand_defend = Some(icy_defend);
    r.card(db, "Vanish from Sight")?.resolve = Some(vanish_resolve);
    let c = r.card(db, "Retrieve the Esper")?;
    c.resolve = Some(retrieve_resolve);
    c.prio = Some(retrieve_prio);
    aura(r, db, "Rousing Read", &ROUSING)?;
    r.card(db, "Unfathomable Truths")?.resolve = Some(truths_resolve);
    let c = r.card(db, "West Coast Expansion")?;
    c.resolve = Some(west_resolve);
    c.prio = Some(zero_prio);
    c.hand_options = Some(west_options);
    r.card(db, "Sleight of Hand")?.resolve = Some(sleight_resolve);
    Ok(())
}
