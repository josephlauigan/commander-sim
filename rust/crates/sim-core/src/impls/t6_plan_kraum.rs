//! Tier 6 plan slot for `kraum-tymna-blue-breach-midrange`: the deck agent fills these in from the deck's SPEC.md. Contract: see `t6_plans.rs`.
//! Every function starts as "no opinion" (`None`), so the shared cEDH table rules decide.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(_g: &Game, _p: PlayerId, _c: CardId) -> Option<f64> {
    None
}

/// tutor wish list, best first; None = no opinion (the closest combo's missing pieces)
pub fn wish(_g: &Game, _p: PlayerId) -> Option<Vec<CardId>> {
    None
}

/// mulligan keep rule for the current opening hand; None = no opinion (2-5 lands)
pub fn keep(_g: &Game, _p: PlayerId) -> Option<bool> {
    None
}

/// extra value of passing with mana up this turn (added to the shared hold value); None = no opinion
pub fn hold_up(_g: &Game, _p: PlayerId) -> Option<f64> {
    None
}

/// go for the combo named `combo` now (Some(true)), wait (Some(false)), or no opinion (None)
pub fn go_now(_g: &Game, _p: PlayerId, _combo: &str) -> Option<bool> {
    None
}
