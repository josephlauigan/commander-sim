//! The smaug candidate deck (decklists/JD/candidates/smaug.md): its card code and AI plan. Filled in by a candidate
//! agent (rust/agents/cand2-*.md).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// smaug's cast priority: the generic priorities for now
pub fn smaug_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
