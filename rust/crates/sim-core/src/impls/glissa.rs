//! The glissa candidate deck (decklists/JD/candidates/glissa.md): its card code and AI plan. Filled in by the
//! glissa agent (rust/agents/cand-*.md).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// glissa's cast priority: the generic priorities for now
pub fn glissa_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
