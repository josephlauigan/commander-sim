//! Tier 6 table support: who is a Tier 6 player, and the plan slots the deck agents fill.
//!
//! SLOT CONTRACT. Each of the seven Tier 6 decks (the six pool keys and `godo`) has one file
//! `impls/t6_plan_<name>.rs` with five functions. The shared AI calls them through the dispatch below (the only
//! `match` on the deck key); a deck agent edits ONLY its own plan file. Each returns `None` for "no opinion", and
//! then the shared Tier 6 table rules (`ai/t6_table.rs`) and the generic AI decide, exactly as before.
//!
//! - `prio(g, p, c) -> Option<f64>`: cast priority of card `c` on `deck_prio`'s 0-100 scale; `Some(0.0)` or less
//!   keeps the card in hand. Replaces the deck's generic priority in `card_utility`.
//! - `wish(g, p) -> Option<Vec<CardId>>`: tutor targets, best first, replacing the closest combo's missing pieces.
//! - `keep(g, p) -> Option<bool>`: keep (true) or mulligan the 7 cards in `p`'s hand now (it is called per draw; after
//!   two mulligans the shared bottom-cards rule still applies and the hand is kept regardless on the last draw).
//! - `hold_up(g, p) -> Option<f64>`: extra utility for "stop (hold mana)" in the main phase (positive: pass with the
//!   mana up; negative: develop). Added to the shared hold value, so it works on top of the archenemy logic.
//! - `go_now(g, p, combo_name) -> Option<bool>`: whether to go for the named combo (`combos::all()` name) right now;
//!   `Some(false)` blocks it this decision, `Some(true)` forces it past the shared "can it be stopped" check,
//!   `None` leaves the check to `ai/t6_table.rs`.
//!
//! Every slot is only ever called for a Tier 6 player (`is_t6_key`), so Tier 0-5 games cannot reach a plan.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

use super::{
    t6_plan_dack as dack, t6_plan_godo as godo, t6_plan_kinnan as kinnan, t6_plan_kraum as kraum,
    t6_plan_rograkh_silas as rsilas, t6_plan_rograkh_thrasios as rthras, t6_plan_sisay as sisay,
};

/// the seven Tier 6 deck keys: the six pool decks of `decklists/pool/t6-cedh/` and JD's Godo
pub const T6_KEYS: [&str; 7] = [
    "dack-fayden-white-stax",
    "kinnan-bonder-mana-combo",
    "kraum-tymna-blue-breach-midrange",
    "rograkh-silas-turbo-breach",
    "rograkh-thrasios-creature-combo",
    "sisay-legends-toolbox",
    "godo",
];

/// the one gate: is this deck key a Tier 6 deck?
pub fn is_t6_key(key: &str) -> bool {
    T6_KEYS.contains(&key)
}

/// the one gate for every Tier 6 AI rule: is player `p` playing a Tier 6 deck?
pub fn is_t6(g: &Game, p: PlayerId) -> bool {
    is_t6_key(g.player(p).key)
}

macro_rules! dispatch {
    ($g:expr, $p:expr, $f:ident($($a:expr),*)) => {
        match $g.player($p).key {
            "dack-fayden-white-stax" => dack::$f($g, $p $(, $a)*),
            "kinnan-bonder-mana-combo" => kinnan::$f($g, $p $(, $a)*),
            "kraum-tymna-blue-breach-midrange" => kraum::$f($g, $p $(, $a)*),
            "rograkh-silas-turbo-breach" => rsilas::$f($g, $p $(, $a)*),
            "rograkh-thrasios-creature-combo" => rthras::$f($g, $p $(, $a)*),
            "sisay-legends-toolbox" => sisay::$f($g, $p $(, $a)*),
            "godo" => godo::$f($g, $p $(, $a)*),
            _ => None,
        }
    };
}

pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    dispatch!(g, p, prio(c))
}
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    dispatch!(g, p, wish())
}
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    dispatch!(g, p, keep())
}
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    dispatch!(g, p, hold_up())
}
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    dispatch!(g, p, go_now(combo))
}
