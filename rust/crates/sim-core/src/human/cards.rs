//! Python's play/cards.py. Ported by the practice-cards task.
//! The per-card choices of the people's decks (targets, modes, tutors, pay-or-not, ordering), and the helpers the
//! `HUMAN(phase 9)` sites in `impls/*` share.

use super::{Target, choices, legal};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// `hc.choose(g, p, kind, prompt, [legal.describe_target(g, p, m) for m in cands], cancel=...)`: the permanent the
/// person picks from `cands`, None if they cancel (`cancel: None` is a forced choice: the first when no answer fits).
pub fn pick_perm(
    g: &Game,
    p: PlayerId,
    kind: &str,
    prompt: &str,
    cands: &[PermId],
    cancel: Option<&str>,
) -> Res<Option<PermId>> {
    let labels: Vec<String> = cands.iter().map(|&m| legal::describe_target(g, p, Target::Perm(m))).collect();
    Ok(choices::choose(g, p, kind, prompt, &labels, cancel, None)?.map(|k| cands[k]))
}

/// `hc.choose(g, p, kind, prompt, [NAME(q) for q in opps], cancel=None)`: the opponent the person picks.
pub fn pick_player(g: &Game, p: PlayerId, kind: &str, prompt: &str, opps: &[PlayerId]) -> Res<PlayerId> {
    let labels: Vec<String> = opps.iter().map(|&q| g.player(q).name.to_string()).collect();
    Ok(opps[choices::choose_one(g, p, kind, prompt, &labels)?])
}

/// cards._blink: a token is exiled for good; else t2.blink
pub fn blink(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if g.perm(m).token {
        crate::engine::zones::leave(g, m)?;
        crate::glog!(g, "    {} (a token) is exiled for good", g.perm(m).name);
        return Ok(());
    }
    crate::impls::t2::blink(g, p, m)?;
    Ok(())
}

/// cards.ephemerate_rebound: your upkeep: cast Ephemerate from exile for free, on a creature you choose; not cast, it
/// stays in exile
pub fn ephemerate_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let cre: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if cre.is_empty() {
        return Ok(());
    }
    let Some(m) = pick_perm(
        g,
        p,
        "target",
        "Rebound: cast Ephemerate again? Exile and return which creature",
        &cre,
        Some("don't cast it"),
    )?
    else {
        return Ok(());
    };
    let pl = g.player_mut(p);
    if let Some(i) = pl.exile.iter().position(|&x| x == c) {
        pl.exile.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::engine::cast::on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Ephemerate from exile (rebound) on {}", g.player(p).name, g.perm(m).name);
    if !g.over && crate::engine::stack::counter_window(g, p, c, 3.0, vec![])? && g.player(p).perms.contains(&m) {
        blink(g, p, m)?;
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}
