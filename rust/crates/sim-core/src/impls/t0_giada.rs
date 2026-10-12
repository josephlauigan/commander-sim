//! Tier 0 pool deck card code: the cards of the giada precon (decklists/pool/t0-b2-precons/) that weren't modeled.
//! Filled in by the t0-giada agent (rust/agents/t0-giada.md).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
