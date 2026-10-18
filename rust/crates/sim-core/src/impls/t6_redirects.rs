//! Tier 6 batch `redirects`: the redirect spells, the damage / removal instants and the small white and green
//! instants (rust/agents/t6-batches.md). Cards only: no engine file is touched, so what needs the engine (answering a
//! spell or ability on the stack, alternative costs, "can't cast" locks, flash for a creature card) is written as a
//! plain `pub fn` the engine can call later, and listed in rust/agents/engine-needs-redirects.md.
//!
//! - Redirect Lightning, Ricochet Trap, Untimely Malfunction (redirect mode): `redirect_removal`, the same response
//!   as Deflecting Swat / Return the Favor in `mine.rs`, plus `redirect_spell` for a spell on the stack.

use crate::engine::cast::on_cast;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::values::{pval, untargetable};
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::Sym;
use crate::cards::CardDb;

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

/// p casts `card` from the hand as a response: the cast is counted (storm, magecraft, cast triggers) and the card
/// goes to the graveyard; the cost was paid by the caller
fn cast_from_hand(g: &mut Game, p: PlayerId, card: CardId) -> Res {
    remove_card(&mut g.player_mut(p).hand, card);
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(card);
    pl.stat("spells_cast", 1);
    on_cast(g, p, card)?;
    g.player_mut(p).gy.push(card);
    Ok(())
}

// ================================================================== redirecting removal
/// the permanents of `actor` a redirected spell of this kind may be aimed at (a damage spell needs a creature)
fn redirect_targets(g: &Game, actor: PlayerId, kind: &str, not: PermId) -> Vec<PermId> {
    g.player(actor)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            x != not
                && g.perm(x).on_bf
                && !g.perm(x).phased
                && !untargetable(g, x)
                && (g.is_creature(x) || !kind.starts_with("dmg"))
        })
        .collect()
}

/// Redirect Lightning's additional cost: how p pays "5 life or {2}". True = the life. The AI pays the mana when it
/// has it ({R} plus {2}), else 5 life when that leaves it above 10; None if neither can be paid.
pub fn redirect_lightning_life(g: &Game, p: PlayerId) -> Option<bool> {
    if can_pay(g, p, 2, "R", false) {
        Some(false)
    } else if can_pay(g, p, 0, "R", false) && g.player(p).life > 10 {
        Some(true)
    } else {
        None
    }
}

/// the permanent the AI redirects a targeted spell of `kind` to: the best of the caster's own (not `m`)
pub fn redirect_to(g: &Game, actor: PlayerId, kind: &str, m: PermId) -> Option<PermId> {
    max_by(&redirect_targets(g, actor, kind, m), |x| pval(g, x))
}

/// Redirect Lightning / Untimely Malfunction / Ricochet Trap answering removal aimed at the owner's permanent `m` (the
/// response `ai::protect_response` would call, like Deflecting Swat in `mine.rs`): the best card the owner holds pays
/// its cost, and the spell is aimed at the best of the caster's own permanents instead. Redirect Lightning costs {R}
/// and {2} or 5 life, Untimely Malfunction {1}{R}, Ricochet Trap {R} after a blue spell by an opponent or {3}{R}.
/// True if the removal was redirected (and so has been applied to the new target).
pub fn redirect_removal(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    let (Some(actor), Some(spell)) = (actor, spell) else { return Ok(false) };
    if actor == owner || matches!(&*kind, "edict" | "wipe") || (pval(g, m) < 4.0 && !g.perm(m).is_cmd) {
        return Ok(false);
    }
    let Some(t) = redirect_to(g, actor, &kind, m) else { return Ok(false) };
    for name in ["Redirect Lightning", "Untimely Malfunction", "Ricochet Trap"] {
        let Some(c) = g.db.id(name).filter(|c| g.player(owner).hand.contains(c)) else { continue };
        match name {
            "Redirect Lightning" => {
                let Some(life) = redirect_lightning_life(g, owner) else { continue };
                if life {
                    pay(g, owner, 0, "R", false)?;
                    lose_life(g, owner, 5, Some(owner), "other", None)?;
                } else {
                    pay(g, owner, 2, "R", false)?;
                }
            }
            "Untimely Malfunction" => {
                if !can_pay(g, owner, 1, "R", false) {
                    continue;
                }
                pay(g, owner, 1, "R", false)?;
            }
            _ => {
                // Ricochet Trap: change the target of a spell; {R} if an opponent cast a blue spell this turn
                let (gn, pips) = if blue_spell_this_turn(g, owner) { (0, "R") } else { (3, "R") };
                if !can_pay(g, owner, gn, pips, false) {
                    continue;
                }
                pay(g, owner, gn, pips, false)?;
            }
        }
        cast_from_hand(g, owner, c)?;
        crate::glog!(g, "    {} casts {}: {} now targets {}", pname(g, owner), name, cname(g, spell), g.perm(t).name);
        apply_removal(g, Some(actor), t, &kind, Some(spell))?;
        return Ok(true);
    }
    Ok(false)
}

/// Ricochet Trap's condition: an opponent of p cast a blue spell this turn
pub fn blue_spell_this_turn(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| match &g.player(q).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.iter().any(|&c| g.db.get(c).pips.contains('U')),
        _ => false,
    })
}

// ================================================================== registration
pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
