//! Tier 6 batch `legends1b`: the Sisay deck's untap loop and its haste enablers (Ioreth of the Healing House, Marvin,
//! Murderous Mimic, Shang-Chi, Master of Kung Fu, Tyvar, Jubilant Brawler, Loki's Scepter). Cards only: no engine
//! file is touched; what the engine still has to do is in rust/agents/engine-needs-legends1b.md.

use super::common::{WalkerAb, walker};
use super::common::best_opp_nonland;
use crate::cards::CardDb;
use crate::engine::hooks as eh;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::pval;
use crate::engine::zones::{Enter, enter, max_by, mill};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased && !g.perm(m).neutered
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

const SHANG: &str = "Shang-Chi, Master of Kung Fu";
const TYVAR: &str = "Tyvar, Jubilant Brawler";
const IORETH: &str = "Ioreth of the Healing House";
const MARVIN: &str = "Marvin, Murderous Mimic";

/// "You may activate abilities of creatures you control as though those creatures had haste": p controls Shang-Chi or
/// Tyvar. (Called by the mana-source collection in engine/mana.rs and by the card `can_tap` checks.)
pub fn ability_haste(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && matches!(name_of(g, m), SHANG | TYVAR))
}

/// can permanent m's {T} ability be used now? (untapped; a creature must not be summoning sick unless it has haste or
/// its controller has Shang-Chi / Tyvar)
fn can_tap(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m)
        && !x.tapped
        && (!x.sick || !g.is_creature(m) || crate::dsl::has_kw(g, m, "haste") || ability_haste(g, x.owner))
}

fn land_yield(g: &Game, p: PlayerId, l: LandId) -> u32 {
    let base = crate::cardcode::dyn_mana_land(g, p, l)
        .map_or_else(|| g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1).max(0) as u32, |n| n);
    (base as i32 + eh::total_land_mana(g, p, l)).max(0) as u32
}

fn is_leg_creature(g: &Game, m: PermId) -> bool {
    g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
}

/// what untapping the tapped permanent m is worth to p, in mana: a mana creature or rock, or Ioreth / Marvin when the
/// other one is out (each untaps the other's source)
fn perm_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    if !live(g, m) || !x.tapped || x.owner != p {
        return 0.0;
    }
    let Some(c) = x.cd else { return 0.0 };
    let t = &g.db.get(c).tags;
    let mut v = if t.has(Tag::Rock) || t.has(Tag::Dork) { 1.0 } else { 0.0 };
    if matches!(name_of(g, m), IORETH | MARVIN) {
        v += 1.0;
    }
    v
}

// ================================================================== Ioreth / Marvin
/// uses of the untap ability by p this turn (all sources): capped so an Ioreth + Marvin pair can't loop forever
const UNTAP_CAP: usize = 6;

fn uses(g: &Game, p: PlayerId) -> usize {
    let stamp = g.turn_stamp();
    (0..UNTAP_CAP).filter(|n| g.player(p).flag_turn.get(&intern(&format!("untap_use_{n}"))) == Some(&stamp)).count()
}

fn bump(g: &mut Game, p: PlayerId) {
    let n = uses(g, p);
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(&format!("untap_use_{n}")), stamp);
}

/// the best use of "{T}: untap another target permanent" / "{T}: untap two other target legendary creatures" for src:
/// (arg, worth). arg: bits 60-61 the kind (0 one permanent, 1 one land, 2 two legendary creatures), then two ids.
fn untap_plan(g: &Game, p: PlayerId, src: PermId) -> Option<(i64, f64)> {
    let mut best: Option<(i64, f64)> = None;
    let mut take = |arg: i64, w: f64| {
        if w >= 1.0 && best.is_none_or(|b| w > b.1) {
            best = Some((arg, w));
        }
    };
    for &l in &g.player(p).lands {
        if g.land(l).on_bf && g.land(l).tapped {
            take((1 << 60) | l.0 as i64, land_yield(g, p, l) as f64);
        }
    }
    let others: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| m != src && g.perm(m).tapped).collect();
    for &m in &others {
        take(m.0 as i64, perm_worth(g, p, m));
    }
    let legs: Vec<PermId> = others.iter().copied().filter(|&m| is_leg_creature(g, m)).collect();
    let mut ranked: Vec<(f64, PermId)> = legs.iter().map(|&m| (perm_worth(g, p, m), m)).collect();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    if ranked.len() >= 2 {
        take((2 << 60) | ((ranked[0].1.0 as i64) << 20) | ranked[1].1.0 as i64, ranked[0].0 + ranked[1].0);
    }
    best
}

/// Marvin has Ioreth's abilities only while p controls an Ioreth (its own name differs from Marvin's)
fn has_ability(g: &Game, p: PlayerId, src: PermId) -> bool {
    match name_of(g, src) {
        IORETH => true,
        MARVIN => g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m) == IORETH),
        _ => false,
    }
}

/// Marvin has two jobs: Ioreth's untap (Sisay) and copying Kefka's creatures' {T} abilities (newcards_b); both option lists apply
pub(crate) fn marvin_both(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut v = untap_options(g, src, p, post)?;
    v.extend(super::newcards_b::marvin_options(g, src, p, post)?);
    Ok(v)
}

fn untap_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !can_tap(g, src) || uses(g, p) >= UNTAP_CAP || !has_ability(g, p, src) {
        return Ok(vec![]);
    }
    let Some((arg, w)) = untap_plan(g, p, src) else { return Ok(vec![]) };
    let label = format!("untap ({})", name_of(g, src));
    Ok(vec![Opt {
        utility: 1.5 + 0.5 * w,
        label,
        act: Some(Action::Ability { src, f: untap_go, arg }),
    }])
}

/// **Ioreth of the Healing House** ({2}{U}, 1/4): "{T}: Untap another target permanent." and "{T}: Untap two other
/// target legendary creatures." **Marvin, Murderous Mimic** ({2}, 2/2) has the activated abilities of the other
/// creatures its controller has: modelled for Ioreth's (the untap loop with a legend that taps for mana; Marvin and
/// Ioreth untap each other). The AI taps when it untaps a tapped land or mana permanent (or two legends), at most
/// six times a turn.
fn untap_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if owner(g, src) != p || !can_tap(g, src) || uses(g, p) >= UNTAP_CAP || !has_ability(g, p, src) {
        return Ok(false);
    }
    let kind = arg >> 60;
    let mask = (1i64 << 20) - 1;
    let targets: Vec<PermId> = match kind {
        0 => vec![PermId(arg as _)],
        2 => vec![PermId(((arg >> 20) & mask) as _), PermId((arg & mask) as _)],
        _ => vec![],
    };
    let land = (kind == 1).then(|| LandId((arg & mask) as _));
    let ok = |m: &PermId| *m != src && live(g, *m) && g.perm(*m).owner == p && g.perm(*m).tapped;
    if !targets.iter().all(ok) || land.is_some_and(|l| !(g.land(l).on_bf && g.land(l).tapped)) {
        return Ok(false);
    }
    bump(g, p);
    g.perm_mut(src).tapped = true; // the cost
    super::t0_captain::became_tapped(g, src);
    if !ability_window(g, p, Some(src), "untap", None, None)? {
        return Ok(true);
    }
    for m in targets {
        if live(g, m) {
            g.perm_mut(m).tapped = false;
            crate::glog!(g, "    {} untaps {}", g.perm(src).name, g.perm(m).name);
        }
    }
    if let Some(l) = land {
        if g.land(l).on_bf {
            g.land_mut(l).tapped = false;
            crate::glog!(g, "    {} untaps {}", g.perm(src).name, g.db.get(g.land(l).cd).name);
        }
    }
    Ok(true)
}

// ================================================================== Tyvar, Jubilant Brawler
fn tyvar_plus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let tapped = g.player(p).perms.iter().any(|&m| g.is_creature(m) && g.perm(m).tapped && live(g, m));
    let _ = src;
    Some(if tapped { 3.0 } else { 1.5 })
}

/// +1: untap up to one target creature (the best tapped one: a mana creature, then the highest-valued)
fn tyvar_plus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let cands: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && g.perm(m).tapped && live(g, m)).collect();
    let Some(t) = max_by(&cands, |m| perm_worth(g, p, m) * 10.0 + pval(g, m)) else { return Ok(()) };
    if !ability_window(g, p, Some(src), "untap a creature", None, None)? || !live(g, t) {
        return Ok(());
    }
    g.perm_mut(t).tapped = false;
    crate::glog!(g, "    Tyvar untaps {}", g.perm(t).name);
    Ok(())
}

/// the creature card with mana value 2 or less in p's graveyard that -2 would return: the best by power
fn tyvar_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let cands: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= 2).collect();
    max_by(&cands, |c| g.db.get(c).pow as f64 + 0.1 * g.db.get(c).cmc as f64)
}

fn tyvar_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let l = g.perm(src).loyalty.unwrap_or(3);
    (l >= 3 && tyvar_pick(g, p).is_some() && g.player(p).library.len() > 8).then_some(3.5)
}

/// -2: mill three cards, then you may return a creature card with mana value 2 or less from your graveyard to the
/// battlefield (Marvin, Ioreth's targets, or a mana creature that is then free of summoning sickness anyway)
fn tyvar_minus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    if !ability_window(g, p, Some(src), "mill three, return a creature", None, None)? {
        return Ok(());
    }
    mill(g, p, 3)?;
    let Some(c) = tyvar_pick(g, p) else { return Ok(()) };
    g.player_mut(p).gy.retain(|&x| x != c);
    enter(g, p, c, Enter::default())?;
    crate::glog!(g, "    Tyvar returns {}", g.db.get(c).name);
    Ok(())
}

static TYVAR_AB: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "untap a creature", val: tyvar_plus_val, eff: tyvar_plus },
    WalkerAb { delta: -2, label: "mill 3, return a creature", val: tyvar_minus_val, eff: tyvar_minus },
];

// ================================================================== Loki's Scepter
/// **Loki's Scepter** ({2}{R}, legendary artifact): when it enters, gain control of target creature until end of turn,
/// untap it, it gains haste (it also becomes a Villain: no effect here); "{T}: Add one mana of any color" is the
/// card's `rock` tag. The AI steals the best opposing creature worth at least 3 (not a commander), as Conscripts does.
fn loki_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(t) = best_opp_nonland(g, o, |x| g.is_creature(x)) else { return Ok(()) };
    if pval(g, t) < 3.0 || g.perm(t).is_cmd {
        return Ok(());
    }
    let label = format!("gain control of {}", g.perm(t).name);
    if !trigger_window(g, o, Some(src), &label, Some(6.0))? || !g.perm(t).on_bf {
        return Ok(());
    }
    let (q, cd) = (owner(g, t), g.perm(src).cd);
    if crate::ai::protect_response(g, q, t, "steal", Some(o), cd)? || !g.perm(t).on_bf {
        return Ok(());
    }
    crate::engine::control::gain_control(g, o, t, crate::engine::control::Duration::EndOfTurn);
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, IORETH)?.options = Some(untap_options);
    r.card(db, MARVIN)?.options = Some(marvin_both);
    walker(r, db, TYVAR, &TYVAR_AB)?;
    r.card(db, "Loki's Scepter")?.etb = Some(loki_etb);
    Ok(())
}
