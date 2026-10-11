//! The sheoldred deck (decklists/JD/sheoldred-mono-black.md): its card code and AI plan. Filled in by the
//! sheoldred-cards agent (rust/agents/sheoldred-cards.md).

use crate::cards::CardDb;
use crate::flow::Res;
use crate::hooks::{Opt, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// sheoldred's cast priority: the generic priorities for now
pub fn sheoldred_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

/// sheoldred's own plays (brain.special_options): none yet
pub fn sheoldred_special(_g: &mut Game, _p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// sheoldred's responses to removal aimed at its permanents: none yet
pub fn sheoldred_protect(
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
