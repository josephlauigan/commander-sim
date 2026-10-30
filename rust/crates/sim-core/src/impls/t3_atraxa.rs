//! Cards of the Tier 3 Atraxa deck (`atraxa-infect`, EDHREC's bracket-3 average deck: infect and proliferate)
//! that the ability language doesn't compile in full. Poison counters, toxic, infect and proliferate are the engine's
//! (`dsl` "poison", `tymna_cards::toxic_hit`, `engine::combat`, `common::proliferate`); these hooks only add each
//! card's own clauses.
//!
//! - **Ezuri, Stalker of Spheres**: as it enters, pay {3} to proliferate twice (its "whenever you proliferate, draw a
//!   card" is compiled).
//! - **Tainted Observer**: whenever another creature you control enters, pay {2} to proliferate (toxic is compiled).
//! - **Contaminant Grafter**: once a combat, when your creatures deal combat damage to a player, proliferate;
//!   corrupted end step: with an opponent at three or more poison, draw a card, then put a land from your hand onto the
//!   battlefield.
//! - **Brokers Ascendancy**: your end step: a +1/+1 counter on each creature and a loyalty counter on each
//!   planeswalker you control.
//! - **Contagion Engine**: as it enters, a -1/-1 counter on each creature of the opponent it hurts most (its {4}, {T}:
//!   proliferate twice is compiled).
//! - **Cankerbloom**: {1}, sacrifice: proliferate when that adds poison or counters, else destroy the best opposing
//!   artifact or enchantment.
//! - **Phyresis Outbreak**: each opponent gets a poison counter, then their creatures get -X/-X (X = their
//!   controller's poison). Left out: the shrink on creatures that survive it (it ends with the turn).
//! - **Infectious Bite**: your biggest creature kills the best opposing creature its power can, then a poison counter
//!   for each opponent.
//! - **Drown in Ichor**: -4/-4 on the best opposing creature it kills, then proliferate. Left out: the shrink on a
//!   creature it doesn't kill.

use crate::engine::repl;
use crate::cards::{CardDb, Types};
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::{etgh, pval, untargetable};
use crate::engine::zones::{die, draw, landfall};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::proliferate;
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

fn name(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &*g.db.get(c).name)
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// a proliferate now adds something: an opponent with poison, or a permanent of p's with +1/+1 or loyalty counters
fn worth_proliferating(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| g.player(q).poison > 0)
        || g.player(p).perms.iter().any(|&m| live(g, m) && (g.perm(m).plus > 0 || g.perm(m).loyalty.is_some()))
}

// ================================================================== Ezuri, Stalker of Spheres
fn ezuri_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !worth_proliferating(g, o) || !can_pay(g, o, 3, "", false) {
        return Ok(());
    }
    pay(g, o, 3, "", false)?;
    crate::glog!(g, "    Ezuri: pays {{3}} and proliferates twice");
    proliferate(g, o, 2)
}

// ================================================================== Tainted Observer
fn observer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src || g.perm(m).owner != o || !g.is_creature(m) || !live(g, src) {
        return Ok(());
    }
    if !worth_proliferating(g, o) || !can_pay(g, o, 2, "", false) {
        return Ok(());
    }
    pay(g, o, 2, "", false)?;
    crate::glog!(g, "    Tainted Observer: pays {{2}} and proliferates");
    proliferate(g, o, 1)
}

// ================================================================== Contaminant Grafter
fn grafter_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if p != o || g.perm(a).owner != o || dmg <= 0 || !live(g, src) {
        return Ok(());
    }
    let key = intern(&format!("grafter{}", src.0));
    if !crate::engine::values::once_per_turn(g, o, key) {
        return Ok(()); // "one or more creatures ... one or more players": once a combat
    }
    proliferate(g, o, 1)
}

fn grafter_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if g.active != Some(o) || !live(g, src) || !g.opps(o).any(|q| g.player(q).poison >= 3) {
        return Ok(());
    }
    draw(g, o, 1, false)?;
    let land = g.player(o).hand.iter().copied().find(|&c| g.db.get(c).land);
    if let Some(c) = land {
        crate::impls::partials::remove_card(&mut g.player_mut(o).hand, c);
        let tapped = crate::engine::turn::land_enters_tapped(g, o, c);
        g.add_land(o, c, tapped);
        landfall(g, o)?;
        crate::glog!(g, "    Contaminant Grafter: draws, then puts {} onto the battlefield", g.db.get(c).name);
    }
    Ok(())
}

// ================================================================== Brokers Ascendancy
fn brokers_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if g.active != Some(o) || !live(g, src) {
        return Ok(());
    }
    for m in g.player(o).perms.clone() {
        if !live(g, m) {
            continue;
        }
        if g.is_creature(m) {
            repl::put_plus(g, m, 1);
        }
        if g.db.get(g.perm(m).cd.unwrap_or(CardId(0))).types.has(Types::PLANESWALKER)
            && let Some(l) = g.perm(m).loyalty
        {
            g.perm_mut(m).loyalty = Some(l + 1);
        }
    }
    Ok(())
}

// ================================================================== Contagion Engine
fn engine_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src {
        return Ok(());
    }
    // the opponent losing the most: creatures it would kill count most, then the rest
    let score = |g: &Game, q: PlayerId| -> f64 {
        g.player(q)
            .perms
            .iter()
            .filter(|&&x| live(g, x) && g.is_creature(x))
            .map(|&x| if etgh(g, x) <= 1 { pval(g, x) } else { 0.3 })
            .sum()
    };
    let Some(q) = g.opps(o).max_by(|&a, &b| score(g, a).total_cmp(&score(g, b))) else { return Ok(()) };
    for x in g.player(q).perms.clone() {
        if live(g, x) && g.is_creature(x) {
            repl::put_plus(g, x, -1);
            if etgh(g, x) <= 0 {
                die(g, x, intern("sba"))?;
            }
        }
    }
    crate::glog!(g, "    Contagion Engine: a -1/-1 counter on each of {}'s creatures", g.player(q).name);
    Ok(())
}

// ================================================================== Cankerbloom
/// the opposing artifact or enchantment worth destroying (value 3 or more)
fn canker_target(g: &Game, p: PlayerId) -> Option<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            let t = g.perm(m).cd.map(|c| g.db.get(c).types);
            live(g, m)
                && !g.is_creature(m)
                && t.is_some_and(|t| t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT))
                && !untargetable(g, m)
                && pval(g, m) >= 3.0
        })
        .max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

fn canker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).owner != p || !live(g, src) || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    let target = canker_target(g, p);
    if target.is_none() && !worth_proliferating(g, p) {
        return Ok(vec![]);
    }
    let utility = if target.is_some() { 1.5 + 0.3 * target.map_or(0.0, |t| pval(g, t)) } else { 1.2 };
    Ok(vec![Opt {
        utility,
        label: "sacrifice Cankerbloom".into(),
        act: Some(Action::Ability { src, f: canker_go, arg: 0 }),
    }])
}

fn canker_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).owner != p || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    let target = canker_target(g, p);
    let cd = g.perm(src).cd;
    die(g, src, intern("sac"))?;
    match target {
        Some(t) if pval(g, t) >= 4.0 || !worth_proliferating(g, p) => {
            crate::engine::removal::apply_removal(g, Some(p), t, "destroy", cd)?;
        }
        _ => proliferate(g, p, 1)?,
    }
    Ok(true)
}

// ================================================================== Phyresis Outbreak
fn outbreak_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        repl::give_poison(g, p, q, 1, repl::Path::POISON); // Melira, Vorinclex
        let x = g.player(q).poison as i32;
        for m in g.player(q).perms.clone() {
            if live(g, m) && g.is_creature(m) && etgh(g, m) <= x {
                die(g, m, intern("sba"))?;
            }
        }
    }
    crate::glog!(g, "    Phyresis Outbreak: each opponent gets a poison counter; their creatures shrink by their poison");
    crate::engine::life::check_state(g)?;
    Ok(intern("gy"))
}

// ================================================================== Drown in Ichor
/// the best opposing creature that -4/-4 kills
fn ichor_target(g: &Game, p: PlayerId) -> Option<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && etgh(g, m) <= 4 && !untargetable(g, m))
        .max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

fn ichor_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ichor_target(g, p) {
        crate::glog!(g, "    Drown in Ichor: {} gets -4/-4", name(g, t));
        die(g, t, intern("sba"))?;
    }
    proliferate(g, p, 1)?;
    Ok(intern("gy"))
}

fn ichor_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match ichor_target(g, p) {
        Some(t) if pval(g, t) >= 3.0 => 60,
        Some(_) => 30,
        None => 0,
    }
}

// ================================================================== Infectious Bite
/// your biggest creature, and the best opposing creature its power kills
fn bite_pair(g: &Game, p: PlayerId) -> Option<(PermId, PermId)> {
    let mine = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m))
        .max_by_key(|&m| crate::engine::values::epow(g, m))?;
    let pw = crate::engine::values::epow(g, mine);
    let t = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| live(g, m) && g.is_creature(m) && etgh(g, m) <= pw && !untargetable(g, m))
        .max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))?;
    Some((mine, t))
}

/// **Infectious Bite**: your creature deals damage equal to its power to an opposing creature (the best one it
/// kills), then each opponent gets a poison counter. Left out: damage to a creature it doesn't kill.
fn bite_resolve(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some((mine, t)) = bite_pair(g, p) {
        crate::glog!(g, "    {} bites {} (Infectious Bite)", name(g, mine), name(g, t));
        crate::engine::removal::apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        repl::give_poison(g, p, q, 1, repl::Path::POISON); // Melira, Vorinclex
    }
    crate::engine::life::check_state(g)?;
    Ok(intern("gy"))
}

fn bite_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match bite_pair(g, p) {
        Some((_, t)) if pval(g, t) >= 3.0 => 60,
        Some(_) => 35,
        None => 15, // the poison counters alone
    }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {

    if let Some(c) = card_if(r, db, "Ezuri, Stalker of Spheres")? {
        c.etb = Some(ezuri_etb);
    }
    if let Some(c) = card_if(r, db, "Tainted Observer")? {
        c.etb = Some(observer_etb);
    }
    if let Some(c) = card_if(r, db, "Contaminant Grafter")? {
        c.combat_damage = Some(grafter_damage);
        c.end_step = Some(grafter_end);
    }
    if let Some(c) = card_if(r, db, "Brokers Ascendancy")? {
        c.end_step = Some(brokers_end);
    }
    if let Some(c) = card_if(r, db, "Contagion Engine")? {
        c.etb = Some(engine_etb);
    }
    if let Some(c) = card_if(r, db, "Cankerbloom")? {
        c.options = Some(canker_options);
    }
    if let Some(c) = card_if(r, db, "Phyresis Outbreak")? {
        c.resolve = Some(outbreak_resolve);
    }
    if let Some(c) = card_if(r, db, "Infectious Bite")? {
        c.resolve = Some(bite_resolve);
        c.prio = Some(bite_prio);
    }
    if let Some(c) = card_if(r, db, "Drown in Ichor")? {
        c.resolve = Some(ichor_resolve);
        c.prio = Some(ichor_prio);
    }
    Ok(())
}

/// the card's hooks, when the card is in the database (no pool list may run it)
fn card_if<'a>(r: &'a mut Registry, db: &CardDb, name: &str) -> Result<Option<&'a mut crate::hooks::CardImpl>, String> {
    if db.id(name).is_some() { r.card(db, name).map(Some) } else { Ok(None) }
}
