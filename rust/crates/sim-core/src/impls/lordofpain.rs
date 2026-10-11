//! The lordofpain candidate deck (decklists/JD/candidates/lordofpain.md): its card code and AI plan. Filled in by a candidate
//! agent (rust/agents/cand2-*.md).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// lordofpain's cast priority: the generic priorities for now
pub fn lordofpain_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
