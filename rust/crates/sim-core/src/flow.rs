//! How the engine stops partway through a call. Python does this with exceptions: `OutOfWork` (a runaway game or
//! look-ahead playout used up its step budget) and `TriggerProbe` (while finding out whether a card's hook really
//! triggers, `trigger_window` raises at the hook's commit point). Here every engine function that can be cut short
//! returns `Res<T>`, and `?` carries the stop up to whoever handles it. A plain return value, so a probe (which
//! happens at every converted trigger) costs nothing like an exception or a panic would.

use crate::ids::{PermId, PlayerId};

#[derive(Debug, Clone, PartialEq)]
pub enum Stop {
    /// The game (or a look-ahead playout) used up its engine steps (Python's `E.OutOfWork`).
    OutOfWork,
    /// A hook reached its commit point while the engine was only probing whether it triggers (Python's
    /// `TriggerProbe`): who controls the trigger, its source, what it does and how much it matters.
    Probe(Probe),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub controller: PlayerId,
    pub src: Option<PermId>,
    pub name: String,
    pub imp: f64,
}

pub type Res<T = ()> = Result<T, Stop>;
