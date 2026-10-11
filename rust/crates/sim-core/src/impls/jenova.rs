//! The jenova candidate deck (decklists/JD/candidates/jenova.md): its card code and AI plan. Filled in by the
//! jenova agent (rust/agents/cand-*.md).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// jenova's cast priority: the generic priorities for now
pub fn jenova_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
