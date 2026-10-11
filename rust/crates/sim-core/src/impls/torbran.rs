//! The torbran candidate deck (decklists/JD/candidates/torbran.md): its card code and AI plan. Filled in by the
//! torbran agent (rust/agents/cand-*.md).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// torbran's cast priority: the generic priorities for now
pub fn torbran_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
