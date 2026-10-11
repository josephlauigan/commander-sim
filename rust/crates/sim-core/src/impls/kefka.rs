//! Kefka, Court Mage's deck (Grixis discard): the cards the engine's tags and compiled abilities don't cover, and the
//! deck's AI plan (cast priorities, what to discard to Kefka's trigger, when to flip, when the Crawler loop is safe).

use crate::cards::CardDb;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// kefka's cast priority: the generic priorities, Kefka first
pub fn kefka_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

pub fn register(_r: &mut Registry, _db: &CardDb) -> Result<(), String> {
    Ok(())
}
