//! Every registry hook on a leave event (Leaves, Dies, SelfDies, SelfExiled, ExiledFromBf, CreatureToGy, LandGy,
//! Sacrifice), classified by hand against the card's Oracle text. Teysa Karlov, Drivnod and Gandalf the White add a
//! triggered ability caused by a creature dying, or by a legendary permanent or an artifact leaving the battlefield;
//! they add nothing to what merely happens as a permanent leaves (an effect that ends, bookkeeping, a delayed return),
//! and not to a trigger the leave did not cause ("whenever you sacrifice a permanent": a sacrifice is not a leaving).
//! The dispatcher (`engine::hooks::leave_hook_is_trigger`) decides from the hook's bits; this table is the reviewed
//! answer, so a new hook cannot slip in with a wrong default.
//!
//! `tests/leaves_inventory.txt`, one row per line: `card | event | class | behaviour test | note`.
//!   T  the whole hook is a leave-caused triggered ability: copied
//!   S  not a trigger: an effect ending, bookkeeping, a delayed return (runs now, never queued)
//!   M  mixed: setup that runs its trigger part `leave_trigger_times` times (the 4th column names its test)
//!   N  not caused by the leave: a sacrifice trigger, a trigger of another event. A queued hook that is not T is N.
//! `LEAVES_INVENTORY_PRINT=<file>` writes the registry's rows with the dispatcher's class there, to start a new row.

use sim_core::engine::hooks::leave_hook_is_trigger;
use sim_core::hooks::Event;
use sim_core::testkit::*;
use std::collections::{BTreeMap, BTreeSet};

const EVENTS: [Event; 8] = [
    Event::Leaves,
    Event::Dies,
    Event::SelfDies,
    Event::SelfExiled,
    Event::ExiledFromBf,
    Event::CreatureToGy,
    Event::LandGy,
    Event::Sacrifice,
];

/// (card, event) -> (dispatcher says trigger, both bits set, runs now, explicit trigger bit)
fn registry_rows() -> BTreeMap<(String, String), (bool, bool, bool, bool)> {
    let g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    let mut out = BTreeMap::new();
    for d in g.db.iter() {
        let Some(c) = g.db.id(&d.name) else { continue };
        let Some(i) = g.registry.get(c) else { continue };
        for e in EVENTS {
            if i.handles(e) {
                let now = i.runs_at_once(e) || !e.is_trigger();
                let both = i.is_enter_trigger(e) && i.is_enter_setup(e);
                out.insert(
                    (d.name.to_string(), format!("{e:?}")),
                    (leave_hook_is_trigger(i, e), both, now, i.is_enter_trigger(e)),
                );
            }
        }
    }
    out
}

fn table_rows() -> BTreeMap<(String, String), (String, String)> {
    let text = include_str!("leaves_inventory.txt");
    let mut out = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('|').map(str::trim).collect();
        assert!(f.len() >= 3, "line {}: card | event | class [| test]", n + 1);
        let key = (f[0].to_string(), f[1].to_string());
        let val = (f[2].to_string(), f.get(3).unwrap_or(&"").to_string());
        assert!(out.insert(key, val).is_none(), "line {}: a second row for {} {}", n + 1, f[0], f[1]);
    }
    out
}

#[test]
fn print_registry_rows() {
    let Ok(path) = std::env::var("LEAVES_INVENTORY_PRINT") else { return };
    let mut out = String::new();
    for ((card, e), (trig, _, now, _)) in registry_rows() {
        out += &format!("ROW|{card}|{e}|{}|{}\n", if trig { "T" } else { "S" }, if now { "now" } else { "queued" });
    }
    std::fs::write(path, out).unwrap();
}

#[test]
fn every_leave_hook_is_classified_and_the_dispatcher_agrees() {
    let reg = registry_rows();
    let tab = table_rows();
    let mut bad = vec![];
    for (k, (trig, both, now, _)) in &reg {
        let Some((class, _)) = tab.get(k) else {
            bad.push(format!("no row for {} {}", k.0, k.1));
            continue;
        };
        if !matches!(class.as_str(), "T" | "S" | "M" | "N") {
            bad.push(format!("{} {}: class {class:?} is not T, S, M or N", k.0, k.1));
        } else if *both {
            bad.push(format!("{} {}: marked both enter_trigger and enter_setup", k.0, k.1));
        } else if class == "S" && !now {
            bad.push(format!(
                "{} {}: an effect ending or bookkeeping runs as the permanent leaves, never from the queue",
                k.0, k.1
            ));
        } else if class == "M" && !now {
            bad.push(format!("{} {}: a mixed hook decides at the event, so it runs now", k.0, k.1));
        } else if class == "M" && *trig {
            bad.push(format!(
                "{} {}: a mixed hook owns its helper loop; the dispatcher must not multiply it too",
                k.0, k.1
            ));
        } else if (class == "T") != *trig {
            bad.push(format!(
                "{} {}: row says {class}, the dispatcher treats it as {}",
                k.0,
                k.1,
                if *trig { "T" } else { "not T" }
            ));
        }
    }
    for k in tab.keys() {
        if !reg.contains_key(k) {
            bad.push(format!("stale row: {} {}", k.0, k.1));
        }
    }
    assert!(bad.is_empty(), "{} problem(s):\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn every_mixed_leave_hook_names_a_behaviour_test_that_exists() {
    let mut names = BTreeSet::new();
    for f in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")).unwrap() {
        let p = f.unwrap().path();
        if p.extension().is_some_and(|x| x == "rs") {
            let s = std::fs::read_to_string(&p).unwrap();
            for l in s.lines() {
                if let Some(rest) = l.trim_start().strip_prefix("fn ") {
                    names.insert(rest.split('(').next().unwrap().to_string());
                }
            }
        }
    }
    let mut bad = vec![];
    for (k, (class, test)) in table_rows() {
        if class == "M" && (test.is_empty() || !test.split_whitespace().all(|t| names.contains(t))) {
            bad.push(format!("{} {}: mixed row needs existing test name(s), has {test:?}", k.0, k.1));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
