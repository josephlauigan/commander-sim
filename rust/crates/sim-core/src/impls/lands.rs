//! Python's `cards/impl/lands.py`.

use crate::cards::CardDb;
use crate::hooks::Registry;

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}

/// PORT(M5): lands.revert_animated: at the start of a turn, animated lands stop being creatures (or went to the
/// graveyard if the creature died)
pub fn revert_animated(_g: &mut crate::state::Game) -> crate::flow::Res {
    Ok(())
}
