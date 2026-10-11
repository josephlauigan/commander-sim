//! The torbran candidate deck (decklists/JD/candidates/torbran.md): its card code and AI plan.
//!
//! - **Torbran, Thane of Red Fell** is a replacement effect in the engine, not a hook: `engine::life::torbran_plus`
//!   adds 2 per Torbran to every damage the Torbran player deals to an opponent (`lose_life`) or to an opponent's
//!   creature (`apply_removal`, combat). The deck is mono-red, so each source of the player is red: the only
//!   colourless sources (Giant's Boulder, the artifacts) deal no damage. Damage to the player's own permanents
//!   and to the player is not changed.
//! - The creature abilities below (Borderland Marauder, Goblin Medics, Karplusan Wolverine, Godo's Irregulars),
//!   Impractical Joke and Rift Bolt's suspend.
//! - **The AI plan**: the burn counts Torbran's +2 when it picks targets (`legal_targets`, `bolt_something`, the
//!   ability language's damage, the face option in `brain.rs`); with Torbran out every ping goes at a face unless a
//!   real threat (value 5) can be killed (`ping_floor`); Torbran is cast first and the equipment waits for him;
//!   burn that nearly kills goes at the face (`torbran_options`); tokens are 3-damage hits, so token makers and
//!   creatures keep their generic priorities and the deck goes wide before it attacks.

use crate::ai::brain::Situation;
use crate::cards::CardDb;
use crate::engine::cast::bolt_something;
use crate::engine::life::torbran_plus;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, etgh, pval};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;
use crate::tag::Tag;

const TORBRAN: &str = "Torbran, Thane of Red Fell";

fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// is one of p's Torbrans on the battlefield
fn torbran_out(g: &Game, p: PlayerId) -> bool {
    torbran_plus(g, p) > 0
}

/// torbran's cast priority: Torbran first (turn 4 on curve), the protection (Champion's Helm, Darksteel Plate) once he
/// is out, the rest as the generic priorities have it
pub fn torbran_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let name = name_of(g, c);
    if name == TORBRAN {
        return 92 - 3 * g.player(p).tax as i32;
    }
    if name == "Champion's Helm" || name == "Darksteel Plate" {
        return if torbran_out(g, p) { 84 } else { 30 };
    }
    crate::ai::pool::generic_prio(g, p, c)
}

// ------------------------------------------------------------------ the creatures
/// Borderland Marauder: whenever it attacks, it gets +2/+0 until end of turn
fn marauder_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && g.perm(src).on_bf
        && trigger_window(g, p, Some(src), "+2/+0 until end of turn", None)?
    {
        g.perm_mut(src).eot_pt.0 += 2;
    }
    Ok(vec![])
}

/// Goblin Medics: whenever it becomes tapped, it deals 1 damage to any target. It becomes tapped when it attacks
/// (without vigilance); the deck has nothing else that taps it, so that is the only place it is checked.
fn medics_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && g.perm(src).on_bf
        && g.perm(src).tapped
        && trigger_window(g, p, Some(src), "tapped: 1 damage to any target", None)?
    {
        bolt_something(g, p, 1)?;
    }
    Ok(vec![])
}

/// Karplusan Wolverine: whenever it becomes blocked, you may have it deal 1 damage to any target
fn wolverine_blocks(g: &mut Game, src: Src, p: PlayerId, _atk: &[PermId], _d: PlayerId, assign: &mut Assign) -> Res {
    if g.perm(src).owner == p
        && g.perm(src).on_bf
        && assign.iter().any(|x| x.0 == src)
        && trigger_window(g, p, Some(src), "blocked: 1 damage to any target", None)?
    {
        bolt_something(g, p, 1)?;
    }
    Ok(())
}

/// Godo's Irregulars: {R}: it deals 1 damage to target creature blocking it. The AI pays as many {R} as it takes to
/// kill the blocker (each activation is 1 damage, +2 with Torbran) when the blocker is worth killing or would kill
/// the Irregulars, and it can afford them all.
fn irregulars_blocks(g: &mut Game, src: Src, p: PlayerId, _atk: &[PermId], _d: PlayerId, assign: &mut Assign) -> Res {
    if g.perm(src).owner != p || !g.perm(src).on_bf {
        return Ok(());
    }
    let Some(&(_, b)) = assign.iter().find(|x| x.0 == src) else { return Ok(()) };
    if !g.perm(b).on_bf || g.perm(b).owner == p || !g.is_creature(b) {
        return Ok(());
    }
    let per = 1 + torbran_plus(g, p);
    let tough = etgh(g, b);
    let k = ((tough + per - 1) / per).max(1) as u32;
    let worth = pval(g, b) >= 2.0 || epow(g, b) >= etgh(g, src);
    if !worth || k > 4 || !can_pay(g, p, 0, &"R".repeat(k as usize), false) {
        return Ok(());
    }
    if !ability_window(g, p, Some(src), "{R}: 1 damage to the creature blocking it", Some(3.0), Some(b))? {
        return Ok(());
    }
    if !g.perm(b).on_bf || !can_pay(g, p, 0, &"R".repeat(k as usize), false) {
        return Ok(());
    }
    pay(g, p, 0, &"R".repeat(k as usize), false)?;
    // k activations of 1 (+ Torbran's 2 each) damage; apply_removal adds Torbran's 2 once more
    let total = k as i32 * per - torbran_plus(g, p);
    apply_removal(g, Some(p), b, &format!("dmg{}", total.max(1)), None)
}

// ------------------------------------------------------------------ Impractical Joke
/// Impractical Joke: damage can't be prevented this turn (`Game::no_prevent`); 3 damage to up to one target creature
/// or planeswalker (the removal tags choose the target and a creature that dies of it dies here)
fn joke_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    g.no_prevent = Some(g.turn_stamp());
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "dmg3", Some(c))?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ the plan
/// the plays torbran adds: suspend Rift Bolt with spare mana ({R}: it is cast free at the next upkeep, 3 damage
/// plus Torbran's), and burn at the face when it nearly kills (an opponent within reach of the burn plus a swing)
pub fn torbran_options(g: &mut Game, p: PlayerId, s: &Situation, post: bool) -> Res<Vec<Opt>> {
    let mut o = vec![];
    if g.active != Some(p) || crate::human::is_human(g, p) {
        return Ok(o);
    }
    let plus = torbran_plus(g, p);
    for c in g.player(p).hand.clone() {
        let d = g.db.get(c);
        if &*d.name == "Rift Bolt" && post && can_pay(g, p, 0, "R", false) {
            o.push(Opt {
                utility: 2.6 + if plus > 0 { 1.0 } else { 0.0 },
                label: "suspend Rift Bolt".into(),
                act: Some(Action::Plan { f: suspend_rift_bolt, arg: c.0 as i64 }),
            });
        }
        // burn at a face that is nearly dead: the burn plus the board's next swing finishes it
        let r = d.tags.str(Tag::Rem).unwrap_or("");
        if plus > 0 && d.tag(Tag::Face) && r.starts_with("dmg") && can_pay(g, p, d.generic, &d.pips, false) {
            let dmg = r[3..].parse::<i32>().unwrap_or(0) + plus;
            let swing: i32 = g
                .player(p)
                .perms
                .iter()
                .filter(|&&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).sick)
                .map(|&m| epow(g, m).max(0) + plus)
                .sum();
            for &q in &s.opps {
                let life = g.player(q).life;
                if life > dmg && life <= dmg + swing / 2 {
                    o.push(Opt {
                        utility: 5.0,
                        label: format!("{} to the face ({}, nearly dead)", d.name, g.player(q).name),
                        act: Some(Action::Face { card: c, q, kick: 0 }),
                    });
                }
            }
        }
    }
    Ok(o)
}

/// Rift Bolt, suspend 1—{R}: exiled with a time counter; at the next upkeep it is cast free
/// (`jodah::suspend_upkeep`, shared with Profane Tutor)
fn suspend_rift_bolt(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "R", false) {
        return Ok(false);
    }
    pay(g, p, 0, "R", false)?;
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.exile.push(c);
    pl.suspended.push((c, 1));
    crate::glog!(g, "  {} suspends Rift Bolt", g.player(p).name);
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Borderland Marauder")?.attack = Some(marauder_attack);
    r.card(db, "Goblin Medics")?.attack = Some(medics_attack);
    r.card(db, "Karplusan Wolverine")?.blocks = Some(wolverine_blocks);
    r.card(db, "Godo's Irregulars")?.blocks = Some(irregulars_blocks);
    r.card(db, "Impractical Joke")?.resolve = Some(joke_resolve);
    Ok(())
}
