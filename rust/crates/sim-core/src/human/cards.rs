//! Python's play/cards.py. Ported by the practice-cards task.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// What card c needs before the person can cast it that `legal` doesn't know (Python's `cards.needs`): the reason
/// it can't be cast, or None. PORT(practice-cards): stub, never blocks.
pub fn needs(_g: &Game, _p: PlayerId, _c: CardId) -> Option<String> {
    None
}

/// Does p's Flashback (the card) give graveyard card c flashback (Python's `cards.granted_flashback`).
/// PORT(practice-cards): stub.
pub fn granted_flashback(_g: &Game, _p: PlayerId, _c: CardId) -> bool {
    false
}
