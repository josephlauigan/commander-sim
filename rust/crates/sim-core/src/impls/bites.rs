//! Bite and fight spells and the green creatures that go with them, keyed by card name so every deck that plays one
//! of these cards gets it (the Titania deck above all). There is no deck-specific AI here. The mechanics, once each:
//! - **bite** (`pick`, `deal`, `bite_resolve`): "target creature you control deals damage equal to its power to target
//!   creature or planeswalker you don't control". The biting creature is `Ctx::host`, the victim `Ctx::target`.
//!   Deathtouch makes any bite lethal. Bite Down, Hard-Hitting Question, Compel Brutality, Diplomatic Relations and
//!   Knockout Maneuver are bites; Chelonian Tackle is a fight (the victim hits back).
//!   The engine tracks no marked damage, so a bite that does not kill does nothing.
//! - **crime** (`crime`): targeting an opponent, their permanents or their graveyard cards (Hardbristle Bandit).
//! - **infusion** (`gained_life`): "if you gained life this turn" (`Player::gained_turn`).
//!
//! The bites hide themselves from the generic priorities (`zero_prio`) and are played by their `hand_options`: bite and
//! fight with the best deathtouch creature when one is out (any bite kills), otherwise the biggest creature, at the
//! biggest threat. The other cards: Track Down, Lashwhip Predator, Bugenhagen, Tenured Concocter, Paladin of Predation.
//! Tests: tests/cards_bites.rs.

use super::partials::eot_kw;
use crate::cards::CardDb;
use crate::engine::cast::{castable, is_casts};
use crate::engine::mana::{can_pay, cost_of};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{may_window, trigger_window};
use crate::engine::values::{epow, etgh, once_per_turn, pval};
use crate::engine::zones::{Zone, die, draw, leave, max_by, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn live_creature(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m)
}

fn deathtouch(g: &Game, m: PermId) -> bool {
    g.perm(m).dt || crate::engine::combat::kw(g, m, "deathtouch")
}

fn is_walker(g: &Game, m: PermId) -> bool {
    g.perm(m).loyalty.is_some()
}

/// infusion: p gained life this turn
pub fn gained_life(g: &Game, p: PlayerId) -> bool {
    g.player(p).gained_turn.is_some_and(|x| x.0 == g.turn_stamp() && x.1 > 0)
}

fn affordable(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false) && castable(g, p, c, "hand")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// a pump until end of turn
fn pump(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = g.perm_mut(m);
    x.eot_pt.0 += dp;
    x.eot_pt.1 += dt;
}

fn plus_counter(g: &mut Game, m: PermId, n: i32) {
    crate::cardcode::add_counters(g, m, n);
}

// ------------------------------------------------------------------ crime (Hardbristle Bandit)
/// p committed a crime: each Hardbristle Bandit of p's untaps (once each turn)
pub fn crime(g: &mut Game, p: PlayerId) -> Res {
    for m in g.player(p).perms.clone() {
        if live(g, m) && g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Hardbristle Bandit") {
            bandit_untap(g, m)?;
        }
    }
    Ok(())
}

fn bandit_untap(g: &mut Game, m: PermId) -> Res {
    let p = owner(g, m);
    if once_per_turn(g, p, intern(&format!("crime_{}", m.0)))
        && trigger_window(g, p, Some(m), "a crime: untap Hardbristle Bandit", Some(2.0))?
        && live(g, m)
    {
        g.perm_mut(m).tapped = false;
    }
    Ok(())
}

/// does the spell being cast target an opponent, an opponent's permanent or a card in their graveyard?
fn spell_is_crime(g: &Game, p: PlayerId, ctx: &Ctx) -> bool {
    ctx.target.is_some_and(|t| owner(g, t) != p) || ctx.face.is_some_and(|q| q != p)
}

/// Hardbristle Bandit: whenever you commit a crime (a spell that targets an opponent or their things), untap it
fn bandit_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    let Some((cc, ctx, _)) = g.cur_cast.clone() else { return Ok(()) };
    if cc == c && spell_is_crime(g, caster, &ctx) {
        bandit_untap(g, src)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ bite and fight
#[derive(Clone, Copy)]
struct Pick {
    src: PermId,
    tgt: PermId,
    v: f64,
}

/// the damage `src` deals as the spell resolves (`bonus`: power the spell gives it first)
fn bite_power(g: &Game, src: PermId, bonus: i32, walker_src: bool) -> i32 {
    if walker_src { g.perm(src).loyalty.unwrap_or(0) } else { (epow(g, src) + bonus).max(0) }
}

fn kills(g: &Game, src: PermId, amount: i32, t: PermId, walker_src: bool) -> bool {
    if amount <= 0 {
        return false;
    }
    if is_walker(g, t) {
        return amount >= g.perm(t).loyalty.unwrap_or(0);
    }
    (deathtouch(g, src) && !walker_src) || amount >= etgh(g, t)
}

/// the best (creature, victim) for the bite spell c: the victim is the biggest threat that some creature kills; the
/// biter is a deathtouch creature if there is one, else the most powerful one. `bonus`: power the spell gives the biter
/// first; `fight`: the victim hits back (a victim with deathtouch is left alone); `walkers`: planeswalkers are legal
/// victims; `walker_src`: a planeswalker of mine may be the source (Compel Brutality's second mode).
fn pick(g: &Game, p: PlayerId, c: CardId, bonus: i32, fight: bool, walkers: bool, walker_src: bool) -> Option<Pick> {
    let tgts = legal_targets(g, p, "destroy", if walkers { "cp" } else { "c" }, false, Some(c));
    let srcs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            if walker_src { live(g, m) && is_walker(g, m) } else { live_creature(g, m) && !untargetable_by_me(g, m) }
        })
        .collect();
    let mut best: Option<Pick> = None;
    for &t in &tgts {
        if fight && deathtouch(g, t) && epow(g, t) > 0 {
            continue;
        }
        let ok: Vec<PermId> =
            srcs.iter().copied().filter(|&s| kills(g, s, bite_power(g, s, bonus, walker_src), t, walker_src)).collect();
        let s = ok.iter().copied().find(|&s| deathtouch(g, s)).or_else(|| {
            max_by(&ok, |s| (bite_power(g, s, bonus, walker_src), pval(g, s)).0 as f64 * 100.0 - pval(g, s))
        });
        let Some(s) = s else { continue };
        let v = pval(g, t);
        if best.is_none_or(|b| v > b.v) {
            best = Some(Pick { src: s, tgt: t, v });
        }
    }
    best
}

/// hexproof or shroud on my own creature does not stop me targeting it; this is only a hook for a future rule
fn untargetable_by_me(_g: &Game, _m: PermId) -> bool {
    false
}

/// `src` deals `amount` damage to t (a creature or planeswalker)
fn deal(
    g: &mut Game,
    p: PlayerId,
    src: PermId,
    t: PermId,
    amount: i32,
    spell: Option<CardId>,
    walker_src: bool,
) -> Res {
    if !live(g, t) || amount <= 0 {
        return Ok(());
    }
    if is_walker(g, t) {
        let l = g.perm(t).loyalty.unwrap_or(0) - amount;
        g.perm_mut(t).loyalty = Some(l);
        crate::glog!(g, "    {} deals {} to {} (loyalty {})", g.perm(src).name, amount, g.perm(t).name, l);
        if l <= 0 {
            let o = owner(g, t);
            leave(g, t)?;
            to_zone_card(g, t, Zone::Gy);
            let nm = g.perm(t).name;
            *g.player_mut(o).lost_names.entry(nm).or_insert(0) += 1;
        }
        return Ok(());
    }
    g.mark_damaged(t);
    if kills(g, src, amount, t, walker_src) {
        let n = if deathtouch(g, src) && !walker_src { 100_000 } else { amount };
        apply_removal(g, Some(p), t, &format!("dmg{n}"), spell)?;
    }
    Ok(())
}

/// Bite Down, Hard-Hitting Question, Compel Brutality, Diplomatic Relations, Burrog Barrage, Knockout Maneuver and
/// Chelonian Tackle as they resolve: the spell's own effect on the biter, then the damage
fn bite_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let name = name_of(g, c).to_string();
    let walker_src = ctx.mode == Some("walker");
    let (bonus, fight, walkers) = match name.as_str() {
        "Diplomatic Relations" => (1, false, false),
        "Burrog Barrage" => (i32::from(is_casts(g, p) >= 2), false, false),
        "Knockout Maneuver" => (1, false, false),
        "Chelonian Tackle" => (0, true, false),
        "Compel Brutality" => (0, false, true),
        _ => (0, false, true), // Bite Down, Hard-Hitting Question
    };
    // a person chooses both targets; the AI (or a spell cast without them) picks now
    let (mut src, mut tgt) = (ctx.host, ctx.target);
    if src.is_none_or(|s| !live(g, s) || owner(g, s) != p) || (tgt.is_none() && name != "Burrog Barrage" && !fight) {
        if let Some(b) = pick(g, p, c, bonus, fight, walkers, walker_src) {
            (src, tgt) = (Some(b.src), Some(b.tgt));
        } else if src.is_none() {
            src =
                max_by(&g.player(p).perms.iter().copied().filter(|&m| live_creature(g, m)).collect::<Vec<_>>(), |m| {
                    epow(g, m) as f64
                });
        }
    }
    let Some(s) = src.filter(|&s| live(g, s) && owner(g, s) == p) else { return Ok("gy") };
    // the spell's effect on the creature
    match name.as_str() {
        "Diplomatic Relations" => {
            pump(g, s, 1, 0);
            eot_kw(g, s, "vigilance");
        }
        "Burrog Barrage" if is_casts(g, p) >= 2 => pump(g, s, 1, 0),
        "Knockout Maneuver" => plus_counter(g, s, 1),
        "Chelonian Tackle" => pump(g, s, 0, 10),
        _ => {}
    }
    let Some(t) = tgt.filter(|&t| live(g, t)) else { return Ok("gy") };
    let amount = bite_power(g, s, 0, walker_src);
    if fight {
        let back = epow(g, t);
        let kill_back = back > 0 && (deathtouch(g, t) || back >= etgh(g, s));
        deal(g, p, s, t, amount, Some(c), false)?;
        if kill_back && live(g, s) {
            die(g, s, "destroy")?;
        }
    } else {
        deal(g, p, s, t, amount, Some(c), walker_src)?;
    }
    Ok("gy")
}

fn bite_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some((bonus, fight, walkers)) = bite_kind(g, p, c) else { return Ok(false) };
    let name = name_of(g, c).to_string();
    let (b, mode) = match pick(g, p, c, bonus, fight, walkers, false) {
        Some(b) => (b, None),
        None if name == "Compel Brutality" => match pick(g, p, c, 0, false, true, true) {
            Some(b) => (b, Some(intern("walker"))),
            None => return Ok(false),
        },
        None => return Ok(false),
    };
    crate::ai::brain::do_cast_with(
        g,
        p,
        c,
        None,
        Ctx { target: Some(b.tgt), host: Some(b.src), mode, ..Ctx::default() },
    )
}

/// (power the spell gives the biter, is it a fight, may it hit planeswalkers) for a bite card
fn bite_kind(g: &Game, p: PlayerId, c: CardId) -> Option<(i32, bool, bool)> {
    Some(match name_of(g, c) {
        "Bite Down" | "Hard-Hitting Question" | "Compel Brutality" => (0, false, true),
        "Diplomatic Relations" => (1, false, false),
        "Burrog Barrage" => (i32::from(is_casts(g, p) >= 1), false, false),
        "Knockout Maneuver" => (1, false, false),
        "Chelonian Tackle" => (0, true, false),
        _ => return None,
    })
}

/// the main-phase play of a bite card: at the biggest threat one of my creatures can kill
fn bite_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let Some((bonus, fight, walkers)) = bite_kind(g, p, c) else { return Ok(vec![]) };
    let mut found = pick(g, p, c, bonus, fight, walkers, false);
    if found.is_none() && name_of(g, c) == "Compel Brutality" {
        found = pick(g, p, c, 0, false, true, true);
    }
    let Some(b) = found else { return Ok(vec![]) };
    if b.v < 1.2 {
        return Ok(vec![]); // a spell is not spent on a trivial creature
    }
    let d = g.db.get(c);
    let mut u = b.v - 3.5;
    if d.instant && post == Some(false) {
        u -= 0.3; // an instant: slightly better held, but a bite before combat clears a blocker
    }
    if d.name.as_ref() == "Knockout Maneuver" {
        u += 0.4; // the counter stays
    }
    Ok(vec![Opt {
        utility: u,
        label: format!("{} -> {}", d.name, g.perm(b.tgt).name),
        act: Some(Action::Plan { f: bite_plan, arg: c.0 as i64 }),
    }])
}

// ------------------------------------------------------------------ Track Down
/// Track Down: scry 3, then reveal the top card; a creature or land card is drawn
fn track_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::cardcode::scry(g, p, 3, false)?;
    if let Some(&top) = g.player(p).library.last() {
        let d = g.db.get(top);
        if d.creature || d.land {
            draw(g, p, 1, false)?;
        }
    }
    Ok("gy")
}

fn track_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).library.len() < 8 { 0 } else { 32 }
}

// ------------------------------------------------------------------ cost reductions
/// Lashwhip Predator costs {2} less if your opponents control three or more creatures
fn lashwhip_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n: usize = g.opps(p).map(|q| g.player(q).perms.iter().filter(|&&m| live_creature(g, m)).count()).sum();
    if n >= 3 { -2 } else { 0 }
}

// ------------------------------------------------------------------ Bugenhagen

/// Bugenhagen, Wise Elder: at the beginning of your upkeep, if you control a creature with power 7 or greater, draw
fn bugenhagen_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let big = g.player(p).perms.iter().any(|&m| live_creature(g, m) && epow(g, m) >= 7);
    if big && trigger_window(g, p, Some(src), "draw a card (a creature with power 7 or greater)", Some(4.0))? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true; // the engine reads this creature's size from its own rule from here on
    }
    Ok(())
}

// ------------------------------------------------------------------ Tenured Concocter
fn concocter_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if gained_life(g, p) { (2, 0) } else { (0, 0) }
}

/// m becomes the target of a spell or ability a player other than its controller controls (`engine::removal::apply_removal`)
pub fn becomes_target(g: &mut Game, m: PermId, _by: PlayerId) -> Res {
    if g.perm(m).cd.is_none_or(|c| name_of(g, c) != "Tenured Concocter") || !live(g, m) {
        return Ok(());
    }
    let o = owner(g, m);
    if (g.player(o).library.len() > 2 || (crate::human::is_human(g, o) && !g.player(o).library.is_empty()))
        && may_window(g, o, Some(m), "you may draw a card", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Paladin of Predation
/// Paladin of Predation: toxic 6 (six poison counters with the combat damage). It can't be blocked by creatures with
/// power 2 or less: `rules::evasion_blocked`.
fn paladin_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src && dmg > 0 && !crate::engine::values::melira(g, d) {
        let k = crate::cardcode::vorinclex(g, g.perm(a).owner, 6); // Vorinclex
        g.player_mut(d).poison += k as u32;
        crate::glog!(g, "    {} gets 6 poison counters (toxic)", g.player(d).name);
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // the bites and the fight: cast by their own options
    for name in [
        "Bite Down",
        "Hard-Hitting Question",
        "Compel Brutality",
        "Diplomatic Relations",
        "Knockout Maneuver",
        "Chelonian Tackle",
    ] {
        let c = r.card(db, name)?;
        c.resolve = Some(bite_resolve);
        c.hand_options = Some(bite_options);
        c.prio = Some(zero_prio);
    }
    let c = r.card(db, "Track Down")?;
    c.resolve = Some(track_resolve);
    c.prio = Some(track_prio);
    r.card(db, "Lashwhip Predator")?.self_cost = Some(lashwhip_cost);
    r.card(db, "Hardbristle Bandit")?.cast = Some(bandit_cast);
    r.card(db, "Bugenhagen, Wise Elder")?.upkeep = Some(bugenhagen_upkeep);
    let c = r.card(db, "Tenured Concocter")?;
    c.self_pt = Some(concocter_pt);
    c.etb = Some(selfpt_etb);
    r.card(db, "Paladin of Predation")?.combat_damage = Some(paladin_damage);
    Ok(())
}
