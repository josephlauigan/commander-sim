//! The simulator's engine, AI and cards: a port of the Python package `commander_sim`
//! (documents/archive/rust-rewrite-scope.md; the design is in documents/archive/rust-design.md).

pub mod ai;
pub mod cardcode;
pub mod cards;
pub mod control;
pub mod dsl;
pub mod engine;
pub mod exec_tags;
pub mod export;
pub mod flow;
pub mod gf_audit; // --- ai-audit: goldfish loss accounting (stats only)
pub mod goldfish; // --- scion: commander-sim --goldfish
pub mod holdstats;
pub mod hooks;
pub mod human;
pub mod ids;
pub mod impls;
pub mod mull_oracle; // --- c4-mulligan-oracle: trace-only setup for --mulligan-oracle
pub mod mull_oracle_tier; // --- c5-oracle-vs-tier: the real-game arm of --mulligan-oracle
pub mod pysum;
pub mod rng;
pub mod run;
pub mod satsum;
pub mod settings;
pub mod state;
pub mod sym;
pub mod t6stats;
pub mod tag;
pub mod testkit;
