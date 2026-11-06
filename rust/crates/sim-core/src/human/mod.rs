//! Practice mode's rules for a seat played by a person (Python's `play/choices.py`, `play/human.py`, `play/legal.py`,
//! `play/mana.py`, `play/combat.py`, `play/cards.py`, `play/abilities.py`), built on `control::Human::ask`.
//!
//! Inert in simulations: every entry point first asks `human_of(g, p)`, and a copied game has no humans, so the
//! look-ahead and the sim paths are unchanged. The engine puts an `if let Some(h) = human_of(g, p)` in front of the
//! AI's code at each decision site (Python's `hc = human_choice(g, p); if hc: ...`).
//!
//! The page's `view` (Python adds `build_view(g, p.key)` to priority and choice requests) is NOT built here:
//! `sim-core` cannot see the `practice` crate. A `Human` whose `live()` is true adds `data["view"]` itself when it
//! gets the request (it is handed `&Game`).

pub mod abilities;
pub mod alt;
pub mod cards;
pub mod gaps_a;
pub mod marvin_hooks;
pub mod model_gaps_a4;
pub mod choices;
pub mod cloud;
pub mod combos;
pub mod confluence;
pub mod combat;
pub mod fetch;
pub mod human;
pub mod karn;
pub mod legal;
pub mod mana;
pub mod protect;
pub mod script;
pub mod specs;
pub mod thanos;
pub mod gaps_c;
pub mod gaps_t2;
pub mod triggers_t1;
pub mod urdragon;
pub mod walkers;

use crate::control::Human;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use std::sync::Arc;

/// The person playing seat p, if any (Python's `human_choice(g, p)` / `is_human`). Cloned out of the game so the
/// caller can keep using `g` mutably while it asks.
pub fn human_of(g: &Game, p: PlayerId) -> Option<Arc<dyn Human>> {
    g.humans.get(p).cloned()
}

/// Tell the person at p that the engine decided something for them (the log's "(automatic)" line): for a choice the
/// card's code still makes by the AI's rules for a person. No-op for a seat without a person.
pub fn auto_note(g: &Game, p: PlayerId, text: &str) {
    if let Some(h) = human_of(g, p) {
        h.tell(p, crate::sym::intern("auto"), text);
    }
}

/// Is p played by a person? (Python's `is_human(g, p)` / `human_seat`)
pub fn is_human(g: &Game, p: PlayerId) -> bool {
    g.humans.get(p).is_some()
}

/// The human seats still in the game (Python's `human.humans(g)`).
pub fn humans(g: &Game) -> Vec<PlayerId> {
    g.players.iter().filter(|q| q.alive && is_human(g, q.id)).map(|q| q.id).collect()
}

/// A target: Python mixes `Perm` and `Player` in one list; here they are told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Perm(PermId),
    Player(PlayerId),
}
