//! Zada, Hedron Grinder (your mono-red go-wide deck, key `zada`): Zada's copy ability, the spells it copies, the spell
//! payoffs and the deck's AI plan.
//!
//! - **Zada**: whenever you cast an instant or sorcery spell that targets only Zada, the spell is copied for each other
//!   creature you control that it could target; each copy targets a different one. Copies are not cast (no cast
//!   triggers, no storm count) and resolve before the original, each with its own target. The hook reads the spell's
//!   target from `g.cur_cast`, so a spell must be cast with its target chosen (`do_cast_with`, `cast_card`).
//! - **The Zada spells**: Ancestral Anger, Monstrous Rage (the Monster Role is `DataKey::Role`: +1/+1 and trample, and
//!   a second Role of the same kind changes nothing), Bulk Up, Act of Treason, Into the Fray, Blazing Crescendo and
//!   Twinflame have `resolve` slots that take the target from the spell's choices. Sure Strike, Furious Bellow,
//!   Smaug's Fury and Crowd's Favor are compiled from their text and already take a chosen target.
//! - **Return the Favor** copies Zada's trigger (every creature gets two copies) when the AI holds {1}{R}{R} for it;
//!   its other modes are in mine.rs (Veyran's deck).
//! - **Payoffs**: Speed, Young Avenger; Expressive Firedancer (the double strike; the +1/+1 is compiled); Burning
//!   Prophet; Dreadhorde Arcanist; Empty the Warrens (storm); Firebird, Blazing Ranger.
//! - **The AI** (`zada_options`, `zada_prio`, the attack hook): go wide first; cast a Zada spell on Zada once he and
//!   two other creatures are out; draw spells and Twinflame before combat, the pump spells when attacking.

use crate::ai::brain::do_cast_with;
use crate::ai::pool::generic_prio;
use crate::cards::CardDb;
use crate::engine::cast::{cast_card, castable, copy_spell, magecraft, on_cast, resolve};
use crate::engine::combat::has_haste;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::values::{card_name, epow, pval};
use crate::engine::zones::{Tokens, draw, enter_token_copy, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

pub const ZADA: &str = "Zada, Hedron Grinder";
const ANGER: &str = "Ancestral Anger";
const RAGE: &str = "Monstrous Rage";
const BULK_UP: &str = "Bulk Up";
const TREASON: &str = "Act of Treason";
const FRAY: &str = "Into the Fray";
const CRESCENDO: &str = "Blazing Crescendo";
const TWINFLAME: &str = "Twinflame";
const RTF: &str = "Return the Favor";
const WARRENS: &str = "Empty the Warrens";

/// the twelve spells built for Zada: the AI casts them only through the plan below
const SPELLS: [&str; 12] = [
    ANGER,
    CRESCENDO,
    TWINFLAME,
    RAGE,
    "Sure Strike",
    "Furious Bellow",
    "Smaug's Fury",
    "Crowd's Favor",
    BULK_UP,
    TREASON,
    FRAY,
    RTF,
];

pub fn is_zada_spell(name: &str) -> bool {
    SPELLS.contains(&name)
}

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

/// p's Zada, if he's out
fn zada_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && card_name(g, m) == Some(ZADA)
    })
}

fn live_creature(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased && g.is_creature(m)
}

/// the creatures p controls besides z
fn others(g: &Game, p: PlayerId, z: PermId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| m != z && live_creature(g, m)).collect()
}

/// Zada is out with at least `n` other creatures
fn ready(g: &Game, p: PlayerId, n: usize) -> Option<(PermId, usize)> {
    let z = zada_of(g, p)?;
    let k = others(g, p, z).len();
    (k >= n).then_some((z, k))
}

fn hand_card(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| name_of(g, c) == name)
}

/// the creature a spell hits: the one it was cast at, else p's best creature (cast without a plan); None: the
/// target is gone and the spell does nothing
fn target_of(g: &Game, p: PlayerId, ctx: &Ctx) -> Option<PermId> {
    match ctx.target {
        Some(t) => live_creature(g, t).then_some(t),
        None => best_creature(g, p),
    }
}

fn best_creature(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live_creature(g, m)).collect();
    max_by(&cs, |m| pval(g, m) + 0.1 * epow(g, m) as f64)
}

fn eot_kw(g: &mut Game, m: PermId, kw: Sym) {
    let x = g.perm_mut(m);
    if !x.eot_kw.contains(&kw) {
        x.eot_kw.push(kw);
    }
}

fn pump(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 + dp, x.eot_pt.1 + dt);
}

// ------------------------------------------------------------------ the spells
/// +X/+0 and trample, X = 1 plus the Ancestral Angers in your graveyard; draw a card
fn anger(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        let x = 1 + g.player(p).gy.iter().filter(|&&y| name_of(g, y) == ANGER).count() as i32;
        pump(g, t, x, 0);
        eot_kw(g, t, "trample");
        draw(g, p, 1, false)?;
    }
    Ok("gy")
}

/// +2/+0 until end of turn and a Monster Role (+1/+1 and trample; a Role you control replaces another)
fn rage(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        pump(g, t, 2, 0);
        g.perm_mut(t).data.set(DataKey::Role, Val::Bool(true));
    }
    Ok("gy")
}

/// double the target's power until end of turn
fn bulk_up(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        let n = epow(g, t);
        pump(g, t, n, 0);
    }
    Ok("gy")
}

/// gain control until end of turn, untap, haste
fn treason(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        if g.perm(t).owner != p {
            super::marchesa::steal(g, p, t, true);
        }
        let x = g.perm_mut(t);
        x.tapped = false;
        x.sick = false;
        eot_kw(g, t, "haste");
    }
    Ok("gy")
}

/// the target attacks this turn if able. Splice onto Arcane: the deck has no other Arcane spell, so Into the Fray is
/// never spliced and the clause does nothing here.
fn fray(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        let now = Val::Stamp(g.turn_stamp());
        g.perm_mut(t).data.set(DataKey::MustAttack, now);
    }
    Ok("gy")
}

/// +3/+1 until end of turn; exile the top card of your library, you may play it until the end of your next turn
fn crescendo(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = target_of(g, p, ctx) {
        pump(g, t, 3, 1);
        if let Some(x) = g.player_mut(p).library.pop() {
            let pl = g.player_mut(p);
            pl.hand.push(x);
            pl.seen.insert(x);
            let until = pl.turns + 1;
            pl.impulse_long.push((x, until));
            crate::glog!(
                g,
                "    {} exiles {} (playable until the end of next turn)",
                g.player(p).name,
                g.db.get(x).name
            );
        }
    }
    Ok("gy")
}

/// a token copy of creature m with haste, exiled at the next end step (a token of a token copies what it is)
fn copy_creature(g: &mut Game, p: PlayerId, m: PermId) -> Res<Option<PermId>> {
    let x = g.perm(m);
    let cp = match x.cd {
        Some(cd) => enter_token_copy(g, p, cd)?,
        None => {
            let mut data = x.data.clone();
            data.remove(DataKey::Role);
            data.remove(DataKey::MustAttack);
            let spec = Tokens {
                tgh: Some(x.tgh),
                fly: x.fly,
                warrior: x.warrior,
                lifelink: x.lifelink,
                dt: x.dt,
                color: Some(x.colors),
                types: x.ttypes.clone(),
                sick: false,
                data,
                ..Tokens::new(1, x.pow)
            };
            make_tokens(g, p, spec)?.into_iter().next()
        }
    };
    if let Some(cp) = cp {
        let y = g.perm_mut(cp);
        y.sick = false;
        y.temp = true;
    }
    Ok(cp)
}

/// Twinflame's effect for one target: a hasty token copy, exiled at the next end step. A legendary target's copy would
/// go to the graveyard at once (the legend rule), so nothing is made. Strive's extra targets are never chosen: the AI
/// casts Twinflame at one creature, and Zada's copies make the rest.
fn twinflame_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| live_creature(g, t))
        && !crate::cardcode::is_legendary(g, t)
    {
        copy_creature(g, p, t)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Zada
/// Whenever you cast an instant or sorcery that targets only Zada, copy it for each other creature you control
fn zada_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || !(d.instant || d.sorcery) || g.perm(src).phased {
        return Ok(());
    }
    let Some((cc, ctx, _)) = g.cur_cast.clone() else { return Ok(()) };
    if cc != c || ctx.target != Some(src) || ctx.face.is_some() {
        return Ok(());
    }
    let others = others(g, p, src);
    if others.is_empty() {
        return Ok(());
    }
    let name = d.name.to_string();
    if !trigger_window(g, p, Some(src), &format!("copy {name} for {} creatures", others.len()), Some(6.0))? {
        return Ok(());
    }
    let turns = g.player(p).turns;
    g.player_mut(p).milestone.entry("zada").or_insert(turns);
    let rounds = 1 + return_the_favor(g, p, others.len())?;
    crate::glog!(
        g,
        "    Zada copies {name} for {} creature(s){}",
        others.len(),
        if rounds > 1 { " twice" } else { "" }
    );
    for _ in 0..rounds {
        for &t in &others {
            if g.over || !live_creature(g, t) || g.perm(t).owner != p {
                continue;
            }
            let mut cctx = ctx.clone();
            cctx.target = Some(t);
            magecraft(g, p, Some(c), true)?;
            resolve(g, p, c, &cctx, "copy")?;
        }
    }
    Ok(())
}

/// Return the Favor on Zada's trigger ({1}{R}{R}: copy the triggered ability): 1 extra round of copies, or 0
fn return_the_favor(g: &mut Game, p: PlayerId, n: usize) -> Res<u32> {
    let Some(r) = hand_card(g, p, RTF) else { return Ok(0) };
    if n < 3 || human(g, p) || !can_pay(g, p, 1, "RR", false) || !castable(g, p, r, "hand") {
        return Ok(0);
    }
    g.player_mut(p).hand.retain(|&x| x != r);
    pay(g, p, 1, "RR", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(r);
    let prev = std::mem::replace(&mut g.in_rtf, true);
    let res = (|| -> Res<u32> {
        on_cast(g, p, r)?;
        crate::glog!(g, "  {} casts Return the Favor: copy Zada's trigger", g.player(p).name);
        Ok(counter_window(g, p, r, 5.0, vec![])? as u32)
    })();
    g.in_rtf = prev;
    g.player_mut(p).gy.push(r);
    res
}

/// Return the Favor in this deck copies Zada's trigger (above); the Veyran deck's copy-a-spell mode isn't used here
fn rtf_hand_cast(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    if g.player(p).key == "zada" {
        return Ok(());
    }
    super::mine::rtf_copy(g, c, p, spell)
}

// ------------------------------------------------------------------ payoffs
/// Burning Prophet: whenever you cast a noncreature spell, +1/+0 until end of turn, then scry 1
fn prophet(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    if caster != p || g.db.get(c).creature || g.perm(src).phased {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "+1/+0 and scry 1", None)? {
        pump(g, src, 1, 0);
        crate::cardcode::scry(g, p, 1, false)?;
    }
    Ok(())
}

/// mana spent on the spell being cast now (a flashback cost, or what the card costs now; a free cast spent none)
fn mana_spent(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let Some((cc, ctx, zone)) = &g.cur_cast else { return 0 };
    if *cc != c || ctx.paid_otherwise {
        return 0;
    }
    if *zone == "gy" {
        let fb = g.db.get(c).tags.str(Tag::Fb).unwrap_or("");
        let (n, rest) = crate::engine::cast::parse_cost(fb);
        return n + rest.chars().count() as u32;
    }
    let (gn, pips) = cost_of(g, p, c);
    gn + pips.chars().count() as u32
}

/// Expressive Firedancer, opus: five or more mana spent: double strike too (the +1/+1 is the compiled trigger)
fn firedancer(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || !(d.instant || d.sorcery) || g.perm(src).phased || mana_spent(g, p, c) < 5 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "double strike", None)? {
        eot_kw(g, src, "double strike");
    }
    Ok(())
}

/// Speed, Young Avenger: whenever you cast a noncreature spell, you may pay {1}; when you do, target creature with haste
/// can't be blocked this turn except by creatures with haste. The AI pays for its best attacker with haste.
fn speed(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    if caster != p || g.db.get(c).creature || g.perm(src).phased || !can_pay(g, p, 1, "", false) {
        return Ok(());
    }
    let hasty = |g: &Game| -> Vec<PermId> {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| live_creature(g, m) && has_haste(g, m) && epow(g, m) > 0 && !g.perm(m).tapped)
            .collect()
    };
    if g.active != Some(p) || hasty(g).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "pay {1}: a creature with haste is blocked only by haste", None)? {
        return Ok(());
    }
    if let Some(t) = max_by(&hasty(g), |m| epow(g, m) as f64)
        && can_pay(g, p, 1, "", false)
    {
        pay(g, p, 1, "", false)?;
        eot_kw(g, t, "haste_blockers");
        crate::glog!(g, "    Speed: {} can be blocked only by creatures with haste", g.perm(t).name);
    }
    Ok(())
}

/// Firebird: whenever it attacks, other attacking creatures get +X/+0 until end of turn, X = its power
fn firebird(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) || atk.len() < 2 {
        return Ok(vec![]);
    }
    if trigger_window(g, p, Some(src), "other attackers get +X/+0", None)? {
        let x = epow(g, src);
        for &m in atk {
            if m != src && live_creature(g, m) {
                pump(g, m, x, 0);
            }
        }
    }
    Ok(vec![])
}

/// spells Dreadhorde Arcanist casts and how much the AI wants each now, with its target; None: never
fn arcanist_value(g: &Game, p: PlayerId, c: CardId) -> Option<(f64, Option<PermId>)> {
    let name = name_of(g, c);
    let zr = ready(g, p, 1);
    let n = zr.map_or(0, |x| x.1) as f64;
    let at_zada = zr.map(|x| x.0);
    let best = best_creature(g, p);
    let attackers = g.player(p).perms.iter().filter(|&&m| live_creature(g, m) && g.perm(m).tapped).count() as f64;
    let (v, t) = match name {
        ANGER | CRESCENDO => match at_zada {
            Some(z) => (4.0 + n, Some(z)),
            None => (3.0, best),
        },
        RAGE => match at_zada {
            Some(z) => (3.0 + 0.6 * n, Some(z)),
            None => (2.5, best),
        },
        TWINFLAME => (at_zada.map(|_| 3.0 + 0.6 * n)?, at_zada),
        BULK_UP | "Sure Strike" | "Furious Bellow" | "Smaug's Fury" | "Crowd's Favor" => {
            (1.0 + 0.5 * attackers.max(1.0) * if at_zada.is_some() { 1.0 } else { 0.3 }, at_zada.or(best))
        }
        WARRENS | "Glimpse the Impossible" | "Seize the Spoils" | "Command the Stage" => (2.0, None),
        _ => return None,
    };
    Some((v, t))
}

/// Dreadhorde Arcanist: whenever it attacks, cast an instant or sorcery card with mana value up to its power from your
/// graveyard without paying its mana cost; it's exiled instead of going to the graveyard
fn arcanist(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) || human(g, p) {
        return Ok(vec![]);
    }
    let power = epow(g, src);
    let pick = |g: &Game| -> Option<(CardId, Option<PermId>)> {
        let cs: Vec<CardId> = g
            .player(p)
            .gy
            .iter()
            .copied()
            .filter(|&c| {
                let d = g.db.get(c);
                (d.instant || d.sorcery) && d.cmc as i32 <= power && arcanist_value(g, p, c).is_some()
            })
            .collect();
        let c = max_by(&cs, |c| arcanist_value(g, p, c).map_or(0.0, |x| x.0))?;
        arcanist_value(g, p, c).filter(|x| x.0 >= 2.0).map(|x| (c, x.1))
    };
    let Some((c, _)) = pick(g) else { return Ok(vec![]) };
    let name = name_of(g, c).to_string();
    if trigger_window(g, p, Some(src), &format!("cast {name} from the graveyard for free"), Some(4.0))? {
        let Some((c, t)) = pick(g) else { return Ok(vec![]) };
        crate::glog!(g, "    Dreadhorde Arcanist casts {} from the graveyard", g.db.get(c).name);
        cast_card(g, p, c, "gy", Ctx { target: t, paid_otherwise: true, ..Ctx::default() })?;
    }
    Ok(vec![])
}

/// Storm: when you cast it, copy it for each spell cast before it this turn
fn warrens_storm(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let n = super::mine::storm_count(g);
    if n > 0 {
        crate::glog!(g, "    Storm: {} cop{} of Empty the Warrens", n, if n == 1 { "y" } else { "ies" });
    }
    for _ in 0..n {
        copy_spell(g, p, c, None, false)?;
        if g.over {
            break;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ the AI
/// the Zada spells have no cast priority of their own (the plan casts them); the rest of the deck uses the outside
/// decks' generic priorities, with Empty the Warrens waiting for a bigger storm
pub fn zada_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let name = name_of(g, c);
    if is_zada_spell(name) {
        return 0;
    }
    if name == WARRENS {
        let n = g.player(p).spells_this_turn as i32;
        return (40 + 10 * n).min(88);
    }
    generic_prio(g, p, c)
}

fn spell_cost(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let (gn, pips) = cost_of(g, p, c);
    gn + pips.chars().count() as u32
}

/// is there a creature (or token maker) in hand that's cheap enough to cast along with a spell costing `cost`
fn go_wide_first(g: &Game, p: PlayerId, cost: u32) -> bool {
    let mana = total_mana(g, p, false);
    g.player(p).hand.iter().any(|&x| {
        let d = g.db.get(x);
        (d.creature || name_of(g, x) == WARRENS) && !d.land && spell_cost(g, p, x) + cost <= mana
    })
}

const CAST_ON_ZADA: u8 = 0;
const CAST_ON: u8 = 1;
const FLASHBACK_ON_ZADA: u8 = 2;

fn plan_arg(c: CardId, mode: u8, t: Option<PermId>) -> i64 {
    c.0 as i64 | (mode as i64) << 16 | (t.map_or(0, |t| t.0 as i64)) << 24
}

/// cast the spell with its target chosen now (Zada, or a creature)
fn cast_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let mode = ((arg >> 16) & 0xff) as u8;
    let t = PermId((arg >> 24) as u32);
    let target = match mode {
        CAST_ON => Some(t),
        _ => zada_of(g, p),
    };
    let Some(target) = target.filter(|&t| live_creature(g, t)) else { return Ok(false) };
    let zone = (mode == FLASHBACK_ON_ZADA).then_some("gy");
    do_cast_with(g, p, c, zone, Ctx { target: Some(target), ..Ctx::default() })
}

/// the main-phase plays: a Zada spell on Zada once he has company, the cards and tokens first; the cantrips on the best
/// creature when nothing better is left in hand
pub fn zada_options(g: &Game, p: PlayerId, post: bool) -> Vec<Opt> {
    let mut o = vec![];
    if human(g, p) || g.active != Some(p) {
        return o;
    }
    let zr = ready(g, p, 2);
    let hand = g.player(p).hand.clone();
    for &c in &hand {
        let name = name_of(g, c);
        if !is_zada_spell(name) || name == RTF {
            continue;
        }
        let (gn, pips) = cost_of(g, p, c);
        let convoke = g.db.get(c).tag(Tag::Convoke);
        if !can_pay(g, p, gn, &pips, convoke) || !castable(g, p, c, "hand") {
            continue;
        }
        let cost = gn + pips.chars().count() as u32;
        let wide = if go_wide_first(g, p, cost) { 4.0 } else { 0.0 };
        if let Some((z, k)) = zr {
            let k = k as f64;
            let v = match name {
                ANGER => Some(4.5 + 0.9 * (k + 1.0)),
                CRESCENDO => Some(4.0 + 0.8 * (k + 1.0)),
                TWINFLAME if !post => Some(4.0 + 0.6 * k),
                RAGE => Some(3.5 + 0.5 * k),
                TREASON if !post => {
                    let fresh = others(g, p, z)
                        .into_iter()
                        .chain([z])
                        .filter(|&m| (g.perm(m).sick || g.perm(m).tapped) && !has_haste(g, m))
                        .count() as f64;
                    (fresh >= 2.0).then_some(3.0 + 0.7 * fresh)
                }
                FRAY if !post => {
                    let payoffs = g
                        .player(p)
                        .perms
                        .iter()
                        .filter(|&&m| {
                            matches!(
                                card_name(g, m),
                                Some("Speed, Young Avenger" | "Expressive Firedancer" | "Burning Prophet")
                            )
                        })
                        .count() as f64;
                    (payoffs > 0.0).then_some(2.5 + 0.8 * payoffs)
                }
                _ => None, // the pump spells wait for the attack (the attack hook)
            };
            if let Some(v) = v {
                o.push(Opt {
                    utility: v + 2.0 - wide,
                    label: format!("{name} on Zada ({} copies)", k as usize),
                    act: Some(Action::Plan { f: cast_plan, arg: plan_arg(c, CAST_ON_ZADA, None) }),
                });
            }
        } else if matches!(name, ANGER | CRESCENDO) {
            // the cantrips, when nothing else in hand is worth the mana
            let mana = total_mana(g, p, false);
            let better = hand.iter().any(|&x| {
                x != c && !g.db.get(x).land && !is_zada_spell(name_of(g, x)) && spell_cost(g, p, x) + cost <= mana
            });
            if better {
                continue;
            }
            if let Some(t) = best_creature(g, p) {
                o.push(Opt {
                    utility: 2.5,
                    label: format!("{name} (cantrip)"),
                    act: Some(Action::Plan { f: cast_plan, arg: plan_arg(c, CAST_ON, Some(t)) }),
                });
            }
        } else if name == TWINFLAME && !post {
            // one good creature to copy
            let best = g
                .player(p)
                .perms
                .iter()
                .copied()
                .map(|m| (super::mine::twinflame_value(g, p, m, post, Some(0)), m))
                .filter(|x| x.0 >= 1.5)
                .max_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((v, t)) = best {
                o.push(Opt {
                    utility: 1.0 + 0.6 * v,
                    label: "Twinflame (one creature)".into(),
                    act: Some(Action::Plan { f: cast_plan, arg: plan_arg(c, CAST_ON, Some(t)) }),
                });
            }
        }
    }
    o
}

/// the pump spell to cast on Zada as the attack is declared: (card, from the graveyard)
fn best_pump(g: &Game, p: PlayerId, z: PermId, atk: &[PermId]) -> Option<(CardId, bool)> {
    let live: Vec<PermId> = atk.iter().copied().filter(|&m| live_creature(g, m) && g.perm(m).owner == p).collect();
    if live.len() < 2 || others(g, p, z).is_empty() {
        return None;
    }
    let n = live.len() as f64;
    let power: f64 = live.iter().map(|&m| epow(g, m) as f64).sum();
    let mut best: Option<(f64, CardId, bool)> = None;
    let mut consider = |c: CardId, gain: f64, fb: bool| {
        let (gn, pips, conv) = if fb {
            let (n, rest) = crate::engine::cast::parse_cost(g.db.get(c).tags.str(Tag::Fb).unwrap_or(""));
            (n, rest, false)
        } else {
            let (gn, pips) = cost_of(g, p, c);
            (gn, pips, g.db.get(c).tag(Tag::Convoke))
        };
        if gain >= 5.0 && can_pay(g, p, gn, &pips, conv) && best.is_none_or(|b| gain > b.0) {
            best = Some((gain, c, fb));
        }
    };
    for &c in &g.player(p).hand {
        if !castable(g, p, c, "hand") {
            continue;
        }
        match name_of(g, c) {
            "Sure Strike" | "Furious Bellow" | "Smaug's Fury" => consider(c, 3.0 * n, false),
            "Crowd's Favor" => consider(c, n, false),
            RAGE | CRESCENDO => consider(c, 3.0 * n + 2.0, false),
            BULK_UP => consider(c, power, false),
            _ => {}
        }
    }
    for &c in &g.player(p).gy {
        if name_of(g, c) == BULK_UP && castable(g, p, c, "gy") {
            consider(c, power - 2.0, true);
        }
    }
    best.map(|b| (b.1, b.2))
}

/// Zada's own attack hook (no trigger of his): the pump spells go on Zada as attackers are declared
fn zada_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || g.perm(src).phased || human(g, p) {
        return Ok(vec![]);
    }
    for _ in 0..4 {
        let Some((c, fb)) = best_pump(g, p, src, atk) else { break };
        let zone = fb.then_some("gy");
        if !do_cast_with(g, p, c, zone, Ctx { target: Some(src), ..Ctx::default() })?
            || g.over
            || !live_creature(g, src)
        {
            break;
        }
    }
    Ok(vec![])
}

/// Twinflame: the Zada deck casts it from `zada_options`; other decks use mine.rs's
fn twinflame_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).key != "zada" {
        return super::mine::twinflame(g, c, p, post);
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let z = r.card(db, ZADA)?;
    z.cast = Some(zada_cast);
    z.attack = Some(zada_attack);
    *z = z.at_once(Event::Attack);
    r.card(db, ANGER)?.resolve = Some(anger);
    r.card(db, RAGE)?.resolve = Some(rage);
    r.card(db, BULK_UP)?.resolve = Some(bulk_up);
    r.card(db, TREASON)?.resolve = Some(treason);
    r.card(db, FRAY)?.resolve = Some(fray);
    r.card(db, CRESCENDO)?.resolve = Some(crescendo);
    let t = r.card(db, TWINFLAME)?;
    t.resolve = Some(twinflame_resolve);
    t.hand_options = Some(twinflame_options);
    r.card(db, RTF)?.hand_cast = Some(rtf_hand_cast);
    r.card(db, "Burning Prophet")?.cast = Some(prophet);
    r.card(db, "Expressive Firedancer")?.cast = Some(firedancer);
    r.card(db, "Speed, Young Avenger")?.cast = Some(speed);
    r.card(db, "Firebird, Blazing Ranger")?.attack = Some(firebird);
    r.card(db, "Dreadhorde Arcanist")?.attack = Some(arcanist);
    r.card(db, WARRENS)?.self_cast = Some(warrens_storm);
    Ok(())
}
