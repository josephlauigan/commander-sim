//! trostani deck (registered 2026-11-27): combat and AI pieces (defender, Gavony and Castle AI, Elder Gargaroth, Sunblast Angel, Teferi's Protection check).
//!
//! Empty until its batch lands (the Opus design is /tmp/handoff/trostani-opus-design.md; this module belongs to one batch).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
