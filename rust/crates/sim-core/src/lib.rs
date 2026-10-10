//! The simulator's engine, AI and cards: a port of the Python package `commander_sim`
//! (documents/rust-rewrite-scope.md; the design is in documents/rust-design.md).

pub mod cards;
pub mod control;
pub mod export;
pub mod flow;
pub mod hooks;
pub mod ids;
pub mod rng;
pub mod settings;
pub mod state;
pub mod sym;
pub mod tag;
