//! Tier 6 table support: who is a Tier 6 player, and the plan slots the deck agents fill.
//!
//! SLOT CONTRACT. Each Tier 6 deck answers five slots. Since Phase 4 PR 2 they come from its plan file
//! (`data/plans/<key>.toml`, read by `ai::plan_data::t6`); the decks still in `HAND` (Dack Fayden, which the plan schema
//! cannot express, and the unseated `godo`) keep a hand file `impls/t6_plan_<name>.rs`, called through the dispatch
//! below (the only `match` on the deck key), and `AI_PLAN_DATA` picks between the two. Each returns `None` for "no
//! opinion", and then the shared Tier 6 table rules (`ai/t6_table.rs`) and the generic AI decide, exactly as before.
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

use super::{t6_plan_dack as dack, t6_plan_godo as godo};

/// the eight Tier 6 deck keys: the six pool decks of `decklists/pool/t6-cedh/`, Godo (retired 2026-10-25, a test seat)
/// and JD's Scion
pub const T6_KEYS: [&str; 8] = [
    "dack-fayden-white-stax",
    "kinnan-bonder-mana-combo",
    "kraum-tymna-blue-breach-midrange",
    "rograkh-silas-turbo-breach",
    "rograkh-thrasios-creature-combo",
    "sisay-legends-toolbox",
    "godo",
    "scion", // --- scion: JD's Scion of the Ur-Dragon turbo deck
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
            "godo" => godo::$f($g, $p $(, $a)*),
            _ => None,
        }
    };
}
/// --- phase4-cedh: the Tier 6 decks that still have a hand plan file here (`AI_PLAN_DATA` picks between it and the
/// deck's `data/plans/<key>.toml`); a deck whose hand file is deleted always plays from its plan file. Deleted after a
/// level paired test: kinnan, kraum, rograkh-silas, rograkh-thrasios, scion, and sisay's five slots (its `fetch` stays
/// in `t6_plan_sisay.rs`).
pub const HAND: [&str; 2] = ["dack-fayden-white-stax", "godo"];

pub fn has_hand(key: &str) -> bool {
    HAND.contains(&key)
}

/// --- phase4-cedh: the plan file a Tier 6 player plays from (`ai::plan_data`), when it does. Only Tier 6 decks: a
/// pool deck's plan file is read by the generic policy (`plans::config`), never through these slots.
fn plan(g: &Game, p: PlayerId) -> Option<&'static crate::ai::plan_data::Loaded> {
    if !is_t6(g, p) {
        return None;
    }
    crate::ai::plan_data::active(g, p)
}

pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    if let Some(l) = plan(g, p) {
        return crate::ai::plan_data::t6::prio(l, g, p, c);
    }
    dispatch!(g, p, prio(c))
}
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    if let Some(l) = plan(g, p) {
        return crate::ai::plan_data::t6::wish(l, g, p);
    }
    dispatch!(g, p, wish())
}
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    if let Some(l) = plan(g, p) {
        return crate::ai::plan_data::t6::keep(l, g, p);
    }
    dispatch!(g, p, keep())
}
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    if let Some(l) = plan(g, p) {
        return crate::ai::plan_data::t6::hold_up(l, g, p);
    }
    dispatch!(g, p, hold_up())
}
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    if let Some(l) = plan(g, p) {
        return crate::ai::plan_data::t6::go_now(l, g, p, combo);
    }
    dispatch!(g, p, go_now(combo))
}
