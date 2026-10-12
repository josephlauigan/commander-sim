//! The cloud deck (decklists/JD/cloud-midgar-equipment.md): its card code and AI plan. Filled in by the
//! cloud-cards agent (rust/agents/cloud-cards.md).

use crate::cards::CardDb;
use crate::flow::Res;
use crate::hooks::{Opt, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// cloud's cast priority: the generic priorities for now
pub fn cloud_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

/// cloud's own plays (brain.special_options): none yet
pub fn cloud_special(_g: &mut Game, _p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// cloud's responses to removal aimed at its permanents: none yet
pub fn cloud_protect(
    _g: &mut Game,
    _p: PlayerId,
    _m: PermId,
    _kind: &str,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    Ok(false)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
