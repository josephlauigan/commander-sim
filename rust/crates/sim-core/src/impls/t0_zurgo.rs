//! Tier 0 pool deck card code: the cards of the zurgo precon (decklists/pool/t0-b2-precons/) that weren't modeled.
//! Filled in by the t0-zurgo agent (rust/agents/t0-zurgo.md).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
