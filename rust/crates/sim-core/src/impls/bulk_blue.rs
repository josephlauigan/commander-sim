//! Card code for the blue spells of the elrond deck (decklists/JD/elrond-ring-tempts.md).
//! Filled in by the blue-spells agent (rust/agents/blue-spells.md).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
