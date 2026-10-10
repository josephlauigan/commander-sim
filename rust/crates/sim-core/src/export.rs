//! Loading the Python export (`commander_sim/tools/export_cards.py`): `data/cards.json` and `data/decks.json`.
//!
//! These are the raw records, field for field as Python writes them. The engine's own card type (`CardDef`, M1)
//! is built from them; nothing here interprets tags or abilities.

use indexmap::IndexMap;
use serde::Deserialize;
use std::{fs, io, path::Path};

/// A tag's value: `fly` is a bare tag (true), `pow=7` has a value (always a string in the export).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TagValue {
    Flag(bool),
    Value(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Derived {
    pub cmc: u32,
    pub land: bool,
    pub creature: bool,
    pub instant: bool,
    pub sorcery: bool,
    pub perm: bool,
}

/// One card definition as Python's `engine.CD` has it.
#[derive(Debug, Clone, Deserialize)]
pub struct RawCard {
    pub name: String,
    /// One-letter type codes: L land, C creature, I instant, S sorcery, A artifact, E enchantment, P planeswalker.
    pub types: String,
    pub generic: u32,
    pub pips: String,
    pub tags: IndexMap<String, TagValue>,
    pub pow: i32,
    pub tgh: i32,
    pub bomb: i32,
    /// Compiled abilities, kept as JSON until the ability language is typed (M3).
    pub dsl: Option<serde_json::Value>,
    pub start_loyalty: Option<String>,
    pub kws: Vec<String>,
    pub subtypes: Vec<String>,
    pub protfrom: String,
    pub ward: u32,
    pub identity: Option<String>,
    pub game_changer: Option<bool>,
    pub source: String,
    pub unparsed: Vec<String>,
    pub derived: Derived,
    /// the colours of its Phyrexian mana symbols ("B" for {B/P})
    pub phyrexian: String,
    /// a land's "when this land enters" effects: ("scry", n) or ("gain", n)
    pub land_etb_fx: Vec<(String, i32)>,
    /// Events the card's Python implementation handles: what has to be ported by hand.
    pub python_hooks: Vec<String>,
    pub spell_prio: bool,
}

impl RawCard {
    /// What Python's `CD.__init__` works out from the types and the cost.
    pub fn compute_derived(&self) -> Derived {
        let has = |c: char| self.types.contains(c);
        let (land, creature, instant, sorcery) = (has('L'), has('C'), has('I'), has('S'));
        Derived {
            cmc: self.generic + self.pips.chars().count() as u32,
            land,
            creature,
            instant,
            sorcery,
            perm: !(instant || sorcery || land),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CardsFile {
    version: u32,
    cards: Vec<RawCard>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawDeck {
    #[serde(default)]
    pub tier: Option<String>,
    pub key: String,
    /// the name shown in logs ('Sauron', 'Krenko')
    pub name: String,
    pub commander: String,
    pub cards: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecksFile {
    pub version: u32,
    pub mine: Vec<RawDeck>,
    pub pool: Vec<RawDeck>,
}

fn invalid(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}

pub fn load_cards(path: &Path) -> io::Result<Vec<RawCard>> {
    let f: CardsFile = serde_json::from_str(&fs::read_to_string(path)?).map_err(invalid)?;
    if f.version != 1 {
        return Err(invalid(format!("cards.json version {} (expected 1)", f.version)));
    }
    Ok(f.cards)
}

pub fn load_decks(path: &Path) -> io::Result<DecksFile> {
    let f: DecksFile = serde_json::from_str(&fs::read_to_string(path)?).map_err(invalid)?;
    if f.version != 1 {
        return Err(invalid(format!("decks.json version {} (expected 1)", f.version)));
    }
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn data(f: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data").join(f)
    }

    #[test]
    fn every_card_loads_and_derives_like_python() {
        let cards = load_cards(&data("cards.json")).unwrap();
        assert!(cards.len() > 1200, "{} cards", cards.len());
        for c in &cards {
            assert_eq!(c.compute_derived(), c.derived, "{}", c.name);
        }
        let names: Vec<&str> = cards.iter().map(|c| c.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "cards.json is sorted by name");
    }

    #[test]
    fn every_deck_card_has_a_definition() {
        let cards = load_cards(&data("cards.json")).unwrap();
        let names: HashSet<&str> = cards.iter().map(|c| c.name.as_str()).collect();
        let decks = load_decks(&data("decks.json")).unwrap();
        assert_eq!(decks.pool.len(), 25);
        for d in decks.mine.iter().chain(&decks.pool) {
            for n in d.cards.iter().chain(std::iter::once(&d.commander)) {
                assert!(names.contains(n.as_str()), "{}: {} has no definition", d.key, n);
            }
        }
    }
}
