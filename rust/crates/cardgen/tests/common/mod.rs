//! Deck selection for the real-catalog tests. Which decks JD owns changes (decks are retired and added), so a test
//! never names a deck: it reads `data/decks.json` at test time and picks one by property (its number of colours),
//! from `mine` first and from `pool` when no deck of `mine` has the property.
#![allow(dead_code)]

use serde_json::Value;
use std::path::Path;

pub const COLOURS: &str = "WUBRG";

pub fn decks(root: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root.join("data/decks.json")).unwrap()).unwrap()
}

fn group(root: &Path, name: &str) -> Vec<(String, String)> {
    decks(root)[name]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["key"].as_str().unwrap().to_string(), d["ident"].as_str().unwrap().to_string()))
        .collect()
}

pub fn colour_count(ident: &str) -> usize {
    ident.chars().filter(|c| COLOURS.contains(*c)).count()
}

/// the keys of every deck of `mine`
pub fn mine_keys(root: &Path) -> Vec<String> {
    group(root, "mine").into_iter().map(|(k, _)| k).collect()
}

/// `(key, ident)` of the first deck of `mine` whose number of colours satisfies `want`; else of `pool`
pub fn deck_where(root: &Path, what: &str, want: impl Fn(usize) -> bool) -> (String, String) {
    for g in ["mine", "pool"] {
        if let Some(d) = group(root, g).into_iter().find(|(_, ident)| want(colour_count(ident))) {
            eprintln!("{what}: {} ({}, from {g})", d.0, d.1);
            return d;
        }
    }
    panic!("no deck of mine or pool has {what}");
}

/// the first deck of `pool` that is not a deck of `mine` and whose number of colours satisfies `want`; else any deck
/// of `pool` that satisfies it; else any deck of `pool`
pub fn pool_deck_where(root: &Path, what: &str, want: impl Fn(usize) -> bool) -> String {
    let mine = mine_keys(root);
    let pool = group(root, "pool");
    let d = pool
        .iter()
        .find(|(k, i)| !mine.contains(k) && want(colour_count(i)))
        .or_else(|| pool.iter().find(|(_, i)| want(colour_count(i))))
        .or_else(|| pool.first())
        .unwrap_or_else(|| panic!("no pool deck for {what}"));
    eprintln!("{what}: {} ({}, from pool)", d.0, d.1);
    d.0.clone()
}

/// a colour letter that is not in `ident`, with its bit in the `W U B R G` order of an identity mask
pub fn colour_outside(ident: &str) -> (char, u8) {
    let (i, c) = COLOURS
        .chars()
        .enumerate()
        .find(|(_, c)| !ident.contains(*c))
        .unwrap_or_else(|| panic!("{ident} has every colour"));
    (c, 1 << i)
}
