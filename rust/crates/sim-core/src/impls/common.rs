//! Python's `cards/impl/common.py`.

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}

/// PORT(M5): common.saga_step: p's Sagas get a lore counter and their chapter abilities
pub fn saga_step(_g: &mut crate::state::Game, _p: crate::ids::PlayerId) -> crate::flow::Res {
    Ok(())
}
