//! Phase 4 deck plans (`data/plans/<deck>.toml`, `ai/plan_data.rs`): every plan file loads, names only cards of its
//! deck (`data/decks.json`, or a retired deck's import list), names catalogue combos the deck can assemble, and
//! matches the Commander Spellbook variants it cites. A deck list change that strands a plan line fails here.
//!
//! Combo data by Commander Spellbook, https://commanderspellbook.com.

use serde_json::Value;
use sim_core::ai::plan_data;
use sim_core::testkit::data_dir;
use std::collections::BTreeSet;
use std::fs;

/// the deck's cards (commanders included): decks.json, or for a retired deck its import list
fn deck_cards(key: &str, retired: bool) -> BTreeSet<String> {
    if retired {
        let p = data_dir().join(format!("../decklists/pool/retired/{key}.md"));
        let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let list = text.split("## Import list").nth(1).unwrap_or_else(|| panic!("{key}: no import list"));
        return list
            .lines()
            .filter_map(|l| l.trim().split_once(' ').filter(|(n, _)| n.parse::<u32>().is_ok()).map(|x| x.1))
            .map(|n| n.trim().to_string())
            .collect();
    }
    let v: Value = serde_json::from_str(&fs::read_to_string(data_dir().join("decks.json")).unwrap()).unwrap();
    let d = v["pool"]
        .as_array()
        .unwrap()
        .iter()
        .chain(v["mine"].as_array().into_iter().flatten())
        .find(|d| d["key"] == key)
        .unwrap_or_else(|| panic!("plan file for {key:?}, which is not in data/decks.json (retired = true?)"));
    let mut s: BTreeSet<String> = d["cards"].as_array().unwrap().iter().map(|c| c.as_str().unwrap().into()).collect();
    match &d["commander"] {
        Value::String(c) => {
            s.insert(c.clone());
        }
        Value::Array(cs) => s.extend(cs.iter().map(|c| c.as_str().unwrap().to_string())),
        _ => {}
    }
    s
}

/// a Spellbook variant's card names, from the deck's Spellbook file (any list)
fn spellbook_cards(key: &str, id: &str) -> Option<BTreeSet<String>> {
    let v: Value =
        serde_json::from_str(&fs::read_to_string(data_dir().join(format!("spellbook/{key}.json"))).ok()?).ok()?;
    v.as_object()?
        .values()
        .filter_map(|x| x.as_array())
        .flatten()
        .find(|x| x["id"] == id)
        .map(|x| x["cards"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect())
}

#[test]
fn every_plan_file_fits_its_deck() {
    let all = plan_data::all();
    assert!(!all.is_empty(), "no plan files embedded");
    let mut errs = vec![];
    for (key, l) in all.iter() {
        let deck = deck_cards(key, l.plan.retired);
        for n in plan_data::named_cards(l) {
            if !deck.contains(&n) {
                errs.push(format!("{key}: {n:?} is not in the deck"));
            }
        }
        for (x, (groups, _)) in l.plan.line.iter().zip(&l.lines) {
            for grp in groups {
                if !grp.iter().any(|n| deck.contains(n)) {
                    errs.push(format!("{key}: line {:?}: no card of {grp:?} is in the deck", x.name));
                }
            }
            if let Some(id) = &x.spellbook {
                let want: BTreeSet<String> = x.pieces.iter().flatten().cloned().collect();
                match spellbook_cards(key, id) {
                    None => errs.push(format!("{key}: line {:?}: no Spellbook variant {id}", x.name)),
                    Some(s) if !s.is_subset(&want) => {
                        errs.push(format!("{key}: line {:?}: Spellbook {id} is {s:?}", x.name))
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(errs.is_empty(), "{}", errs.join("\n"));
}

#[test]
fn a_bad_plan_file_is_refused_with_its_reason() {
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    assert!(bad("deck = \"y\"").contains("named for"));
    assert!(bad("deck = \"x\"\nfoo = 1").contains("unknown field"));
    assert!(bad("deck = \"x\"\n[[line]]\nname = \"a\"\ncombo = \"no such combo\"").contains("no catalogue combo"));
    assert!(bad("deck = \"x\"\n[[prio]]\nvalue = 3").contains("selects no card"));
    assert!(bad("deck = \"x\"\n[[prio]]\ncommander = true\nwhen = [\"moon > 3\"]\nvalue = 3").contains("unknown"));
    assert!(
        plan_data::load("x", "deck = \"x\"\n[[prio]]\ncards = [\"A\"]\nwhen = [\"creatures>=6 & lands<3\"]\nvalue = 1")
            .is_ok()
    );
}

/// `cargo test --release --test plans -- --ignored --nocapture catalogue_lines` (PLAN_DECKS=key,...): the catalogue
/// combos each deck can assemble, the `[[line]] combo = ...` candidates for its plan file
#[test]
#[ignore]
fn catalogue_lines_report() {
    let keys = std::env::var("PLAN_DECKS").unwrap_or_default();
    for key in keys.split(',').filter(|k| !k.is_empty()) {
        let deck = deck_cards(key, false);
        println!("{key}:");
        for c in sim_core::impls::combos::all() {
            if c.groups.iter().all(|g| g.iter().any(|n| deck.contains(*n))) {
                println!("  {:?}", c.name);
            }
        }
    }
}
