//! Hand-written card code, one file per Python module in `cards/impl/` (t1.py is `t1.rs`, and so on). Each module
//! has a `register` that fills in the `Registry` by card name, as Python's `@on(name, event)` decorators do at
//! import; the shared functions the engine calls by name are in `cardcode.rs`, which calls into these modules.
//!
//! The modules are ported deck by deck (M5: Tier 1 and Sauron, phase 6: the rest); a card whose code isn't ported
//! yet plays with its tags and compiled abilities alone, as the Python engine would without its hooks.

use crate::cards::CardDb;
use crate::hooks::Registry;

pub mod alela;
pub mod combos;
pub mod common;
pub mod fixes;
pub mod galadriel;
pub mod jodah;
pub mod lands;
pub mod marchesa;
pub mod mine;
pub mod partials;
pub mod rules;
pub mod rules2;
pub mod t1;
pub mod t2;
pub mod t3;
pub mod t4;
pub mod t5;
pub mod topdeck;
pub mod yshtola;
pub mod zur;

/// every module's card code, registered in Python's import order (cardimpl.load: common, t1, t2, t3, t4, t5, combos,
/// topdeck, fixes, lands, partials, rules, rules2, mine, marchesa, zur, galadriel, yshtola, alela, jodah): a later module's slot
/// replaces an earlier one's for the same card and event, as a later `@on(name, event)` does in Python (partials' and
/// rules' Druid Class and Ezuri options replace t1's, rules2's Goblin Rabblemaster and Legion Warboss upkeeps replace
/// t1's ...)
pub fn registry(db: &CardDb) -> Result<Registry, String> {
    let mut r = Registry::new(db);
    common::register(&mut r, db)?;
    t1::register(&mut r, db)?;
    t2::register(&mut r, db)?;
    t3::register(&mut r, db)?;
    t4::register(&mut r, db)?;
    t5::register(&mut r, db)?;
    combos::register(&mut r, db)?;
    topdeck::register(&mut r, db)?;
    fixes::register(&mut r, db)?;
    lands::register(&mut r, db)?;
    partials::register(&mut r, db)?;
    rules::register(&mut r, db)?;
    rules2::register(&mut r, db)?;
    mine::register(&mut r, db)?;
    marchesa::register(&mut r, db)?;
    zur::register(&mut r, db)?;
    galadriel::register(&mut r, db)?;
    yshtola::register(&mut r, db)?;
    alela::register(&mut r, db)?;
    jodah::register(&mut r, db)?;
    Ok(r)
}
