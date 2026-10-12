//! Card code for the white cards of the cloud deck (decklists/JD/cloud-midgar-equipment.md), and face-down (morph, disguise) machinery.
//! Filled in by the white-rest agent (rust/agents/white-rest.md).

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
