//! The simulator's engine, AI and cards: a port of the Python package `commander_sim`
//! (documents/rust-rewrite-scope.md; the design is in documents/rust-design.md).

pub mod ai;
pub mod cardcode;
pub mod cards;
pub mod control;
pub mod dsl;
pub mod engine;
pub mod export;
pub mod flow;
pub mod hooks;
pub mod human;
pub mod ids;
pub mod impls;
pub mod pysum;
pub mod rng;
pub mod run;
pub mod settings;
pub mod state;
pub mod sym;
pub mod tag;
pub mod testkit;
