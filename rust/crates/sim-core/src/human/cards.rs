//! Python's play/cards.py. Ported by the practice-cards task.
//! The per-card choices of the people's decks (targets, modes, tutors, pay-or-not, ordering), and the helpers the
//! `HUMAN(phase 9)` sites in `impls/*` share.

use super::{Target, choices, legal};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
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
