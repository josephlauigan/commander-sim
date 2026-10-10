//! Python's `cards/impl/rules2.py`.

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}

/// PORT(M5): rules2.uncrew: vehicles crewed on an earlier turn stop being creatures
pub fn uncrew(_g: &mut crate::state::Game, _p: crate::ids::PlayerId) {}
