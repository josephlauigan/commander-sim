//! The Tymna partner decks (`thrasios`: Tymna the Weaver + Thrasios, Triton Hero; `kraum`: Tymna + Kraum, Ludevic's
//! Opus). The seat has two commanders (`Player::cmd` and `Player::cmd2`); the engine casts both from the command
//! zone with the generic commander priority. The decks' own AI plans are written later: for now both use the
//! default priority.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// ai/decks.rs `deck_prio` for the thrasios seat (the default until the plan is written)
pub fn thrasios_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

/// ai/decks.rs `deck_prio` for the kraum seat (the default until the plan is written)
pub fn kraum_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}
