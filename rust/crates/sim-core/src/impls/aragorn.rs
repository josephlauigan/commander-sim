//! Aragorn, the Uniter (JD's Aragorn deck, key `aragorn`, 2026-11-06). Its four cast triggers, each checked against
//! the Oracle text in `tests/cards_aragorn.rs`:
//!
//! "Whenever you cast a white spell, create a 1/1 white Human Soldier creature token.
//!  Whenever you cast a blue spell, scry 2.
//!  Whenever you cast a red spell, Aragorn deals 3 damage to target opponent.
//!  Whenever you cast a green spell, target creature gets +4/+4 until end of turn."
//!
//! A spell of several colours triggers each matching ability (a red-white spell: a Soldier and 3 damage). Veyran,
//! Voice of Duality makes each one trigger an additional time when the spell is an instant or sorcery. The damage is
//! damage (Fiery Emancipation triples it; prevention stops it).

use crate::cards::CardDb;
use crate::engine::life::lose_life;
use crate::engine::stack::trigger_window;
use crate::engine::zones::{Tokens, make_tokens};
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;
use crate::tag::Tag;

use super::partials::at_once;

pub const ARAGORN: &str = "Aragorn, the Uniter";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

/// how many times each of Aragorn's abilities triggers for spell c: once, twice with Veyran on an instant or sorcery
pub fn trigger_count(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let d = g.db.get(c);
    let veyran = (d.instant || d.sorcery) && crate::engine::values::has(g, p, Tag::Veyran);
    1 + veyran as u32
}

/// "Whenever you cast a ... spell": Aragorn's controller cast c
fn aragorn_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if !live(g, src) || g.perm(src).owner != caster {
        return Ok(());
    }
    let cols = g.db.get(c).colors;
    let n = trigger_count(g, caster, c);
    for _ in 0..n {
        if cols.has('W') && !g.over && trigger_window(g, caster, Some(src), "a 1/1 Human Soldier", None)? {
            soldier(g, caster)?;
        }
        if cols.has('U') && !g.over && trigger_window(g, caster, Some(src), "scry 2", None)? {
            crate::cardcode::scry(g, caster, 2, false)?;
        }
        if cols.has('R') && !g.over && trigger_window(g, caster, Some(src), "3 damage to target opponent", None)? {
            burn(g, caster)?;
        }
        if cols.has('G') && !g.over && trigger_window(g, caster, Some(src), "target creature gets +4/+4", None)? {
            pump(g, caster)?;
        }
    }
    Ok(())
}

/// "create a 1/1 white Human Soldier creature token"
fn soldier(g: &mut Game, p: PlayerId) -> Res {
    let spec = Tokens {
        color: Some(crate::cards::Colors::from_letters("W")),
        types: vec![intern("human"), intern("soldier")],
        tgh: Some(1),
        ..Tokens::new(1, 1)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// "Aragorn deals 3 damage to target opponent": the person picks the opponent; the AI hits the opponent it can bring
/// lowest (a kill first), as Kaervek's face damage does
fn burn(g: &mut Game, p: PlayerId) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(());
    }
    let q = if crate::human::is_human(g, p) {
        let labels: Vec<String> =
            opps.iter().map(|&q| format!("{} ({} life)", g.player(q).name, g.player(q).life)).collect();
        let i = crate::human::choices::choose_one(g, p, "target", "Aragorn: 3 damage to target opponent", &labels)?;
        opps[i.min(opps.len() - 1)]
    } else {
        // an opponent the damage kills (the lowest such life), else the most threatening board (the combo player)
        let dmg = 3 * crate::engine::combat::fiery(g, p);
        let live: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life > 0).collect();
        let killable = live.iter().copied().filter(|&q| g.player(q).life <= dmg).min_by_key(|&q| g.player(q).life);
        killable.unwrap_or_else(|| {
            *live
                .iter()
                .max_by(|&&a, &&b| {
                    crate::engine::values::threat(g, p, a).total_cmp(&crate::engine::values::threat(g, p, b))
                })
                .unwrap_or(&opps[0])
        })
    };
    let t = g.player(p).turns;
    g.player_mut(p).milestone.entry("aragorn").or_insert(t);
    lose_life(g, q, 3, Some(p), intern("triggers"), Some(true))?;
    crate::engine::life::check_state(g)?;
    Ok(())
}

/// "target creature gets +4/+4 until end of turn": the person picks any creature; the AI pumps its own best attacker
/// (untapped and able to attack this turn, highest power: usually Aragorn), else its best creature. With none of its
/// own it must still target a creature if one exists, and picks the opposing creature that matters least.
fn pump(g: &mut Game, p: PlayerId) -> Res {
    let all: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|pl| pl.perms.iter().copied())
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m) && targetable(g, m, p))
        .collect();
    if all.is_empty() {
        return Ok(());
    }
    let m = if crate::human::is_human(g, p) {
        let labels: Vec<String> = all
            .iter()
            .map(|&m| format!("{} ({})", g.perm(m).name, g.player(g.perm(m).owner).name))
            .collect();
        let i = crate::human::choices::choose_one(g, p, "target", "Aragorn: target creature gets +4/+4", &labels)?;
        all[i.min(all.len() - 1)]
    } else {
        let mine: Vec<PermId> = all.iter().copied().filter(|&m| g.perm(m).owner == p).collect();
        let ready: Vec<PermId> = mine.iter().copied().filter(|&m| crate::human::combat::can_attack(g, p, m)).collect();
        let pick = |v: &[PermId]| v.iter().copied().max_by_key(|&m| crate::engine::values::epow(g, m));
        // an opponent's combat: our blocker (the pump saves it or kills the attacker); our own turn before combat: the
        // best ready attacker; otherwise our best creature
        let blockers: Vec<PermId> = mine.iter().copied().filter(|m| g.blocking.contains(m)).collect();
        let theirs_combat = g.active != Some(p) && matches!(g.step, crate::state::Step::Combat);
        let first = if theirs_combat { pick(&blockers) } else { pick(&ready) };
        match first.or_else(|| pick(&mine)) {
            Some(m) => m,
            None => all
                .iter()
                .copied()
                .min_by(|&a, &b| crate::engine::values::pval(g, a).total_cmp(&crate::engine::values::pval(g, b)))
                .unwrap(),
        }
    };
    super::common::eot(g, m, 4, 4);
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, ARAGORN)?;
    c.cast = Some(aragorn_cast);
    at_once(c, Event::Cast, false);
    Ok(())
}

/// can p target creature m: an opponent's creature not hexproof or shrouded (`untargetable`); p's own creature unless
/// it has shroud (hexproof only stops opponents)
fn targetable(g: &Game, m: PermId, p: PlayerId) -> bool {
    if g.perm(m).owner == p {
        !g.perm(m).phased && !crate::dsl::has_kw(g, m, "shroud")
    } else {
        !crate::engine::values::untargetable(g, m)
    }
}

// ------------------------------------------------------------------ the AI's value of Aragorn's triggers
/// p's Aragorn on the battlefield (not phased out)
pub fn aragorn_out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && &*g.perm(m).name == ARAGORN)
}

/// the damage Aragorn's red trigger deals for spell c (0 for a spell with no red, or with no Aragorn out): 3 per
/// trigger, Veyran's extra trigger on an instant or sorcery, Fiery Emancipation's tripling
pub fn red_damage(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if !aragorn_out(g, p) || !g.db.get(c).colors.has('R') {
        return 0;
    }
    3 * trigger_count(g, p, c) as i32 * crate::engine::combat::fiery(g, p)
}

/// what casting spell c is worth for Aragorn's triggers (AI utility points, added to the spell's own worth by
/// `ai::brain::card_utility`; 0 with no Aragorn out). Red: 3 damage to the lowest-life opponent, worth much more when
/// it kills; white: a 1/1 Soldier; blue: scry 2; green: +4/+4 to an attacker, only before combat on our turn
pub fn cast_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    if !aragorn_out(g, p) {
        return 0.0;
    }
    let d = g.db.get(c);
    if d.land {
        return 0.0;
    }
    let n = trigger_count(g, p, c) as f64;
    let cols = d.colors;
    let mut v = 0.0;
    if cols.has('R') {
        let dmg = red_damage(g, p, c);
        let low = g.opps(p).map(|q| g.player(q).life).filter(|&l| l > 0).min().unwrap_or(i32::MAX);
        v += if dmg >= low { 4.0 } else { 0.12 * dmg as f64 };
    }
    if cols.has('W') {
        v += 0.4 * n;
    }
    if cols.has('U') {
        v += 0.25 * n;
    }
    if cols.has('G') {
        let precombat = g.active == Some(p) && matches!(g.step, crate::state::Step::Main1);
        let ready = g.player(p).perms.iter().any(|&m| crate::human::combat::can_attack(g, p, m));
        if precombat && ready {
            v += 0.5 * n;
        }
    }
    v
}
