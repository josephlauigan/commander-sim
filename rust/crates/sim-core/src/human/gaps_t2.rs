//! Practice mode (2026-11-04, Agent T2, rust/agents/practice-followups.md): triggers later in a game whose target or
//! card the AI picked for the person. These helpers only ask; each card's own file calls them from a
//! `crate::human::is_human(g, p)` branch, so the AI's path is untouched.

use super::{Target, choices, legal};
use crate::engine::values::player_hexproof;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// The person's target opponent for `source`'s trigger ("target opponent loses 1 life ..."). Opponents that can't be
/// targeted are left out (all of them when none can be); with one candidate there is nothing to ask.
pub fn pick_opponent(g: &Game, p: PlayerId, source: &str) -> Res<Option<PlayerId>> {
    let all: Vec<PlayerId> = g.opps(p).collect();
    let ok: Vec<PlayerId> =
        all.iter().copied().filter(|&q| legal::player_targetable(g, q) && !player_hexproof(g, q)).collect();
    let cands = if ok.is_empty() { all } else { ok };
    pick_from(g, p, source, &cands, "target opponent")
}

/// The person's target player for `source`'s trigger (Blood Artist: "target player loses 1 life"): any player in the
/// game, themselves included.
pub fn pick_player(g: &Game, p: PlayerId, source: &str) -> Res<Option<PlayerId>> {
    let all: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    let ok: Vec<PlayerId> =
        all.iter().copied().filter(|&q| q == p || (legal::player_targetable(g, q) && !player_hexproof(g, q))).collect();
    let cands = if ok.is_empty() { all } else { ok };
    // you are always a candidate, so there are two or more whenever anyone else is alive
    pick_from(g, p, source, &cands, "target player")
}

fn pick_from(g: &Game, p: PlayerId, source: &str, cands: &[PlayerId], what: &str) -> Res<Option<PlayerId>> {
    match cands {
        [] => Ok(None),
        [q] => Ok(Some(*q)),
        _ => {
            let labels: Vec<String> = cands.iter().map(|&q| legal::describe_target(g, p, Target::Player(q))).collect();
            let k = choices::choose_one(g, p, "target", &format!("{source}: {what}?"), &labels)?;
            Ok(Some(cands[k]))
        }
    }
}

/// The person picks one of `cands` (a target card in a graveyard, ...). `cancel` is the label of the way out for a
/// "you may"; None is a forced choice. A single candidate of a forced choice is not asked.
pub fn pick_card(g: &Game, p: PlayerId, cands: &[CardId], prompt: &str, cancel: Option<&str>) -> Res<Option<CardId>> {
    if cands.is_empty() {
        return Ok(None);
    }
    if cands.len() == 1 && cancel.is_none() {
        return Ok(Some(cands[0]));
    }
    let labels: Vec<String> = cands.iter().map(|&c| choices::card_label(g, c)).collect();
    Ok(choices::choose(g, p, "target", prompt, &labels, cancel, None)?.map(|k| cands[k]))
}

/// The person picks one of their permanents (or any in `cands`) as a trigger's target. A single candidate of a forced
/// choice is not asked.
pub fn pick_target(g: &Game, p: PlayerId, cands: &[PermId], prompt: &str, cancel: Option<&str>) -> Res<Option<PermId>> {
    if cands.is_empty() {
        return Ok(None);
    }
    if cands.len() == 1 && cancel.is_none() {
        return Ok(Some(cands[0]));
    }
    super::cards::pick_perm(g, p, "target", prompt, cands, cancel)
}

/// Adeline, Resplendent Cathar: "Whenever you attack, for each opponent, create a 1/1 white Human creature token
/// that's tapped and attacking that player or a planeswalker they control." The engine sends every token at the main
/// defender; the person's tokens go one to each opponent (a doubled batch gets the same number each), and the person
/// picks the player or one of their planeswalkers when the opponent has any.
pub fn route_adeline(g: &mut Game, p: PlayerId, tokens: &[PermId]) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() || tokens.is_empty() {
        return Ok(());
    }
    // where each opponent's tokens go: the player, or one of their planeswalkers
    let mut dest: Vec<Option<PermId>> = vec![];
    for &q in &opps {
        let walkers: Vec<PermId> =
            super::combat::attackable_walkers(g, p).into_iter().filter(|&(_, c)| c == q).map(|(w, _)| w).collect();
        if walkers.is_empty() {
            dest.push(None);
            continue;
        }
        let mut labels = vec![legal::describe_target(g, p, Target::Player(q))];
        labels.extend(walkers.iter().map(|&w| {
            format!(
                "{} (planeswalker, {} loyalty)",
                legal::describe_target(g, p, Target::Perm(w)),
                g.perm(w).loyalty.unwrap_or(0)
            )
        }));
        let prompt = format!("Adeline: the token for {} attacks the player or a planeswalker?", g.player(q).name);
        let k = choices::choose_one(g, p, "target", &prompt, &labels)?;
        dest.push(if k == 0 { None } else { Some(walkers[k - 1]) });
    }
    for (j, &t) in tokens.iter().enumerate() {
        let i = j % opps.len();
        g.atk_targets.push((t, opps[i]));
        if let Some(w) = dest[i] {
            g.human_walker_atk.push((t, w));
        }
    }
    Ok(())
}
