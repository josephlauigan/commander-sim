//! Card data and deck lists without Python: builds `data/cards.json` and `data/decks.json` from the hand-edited data
//! files (`data/carddb.txt`, `data/card_overrides.toml`, `data/my_decks.toml`), the Scryfall cache and the deck lists
//! in `decklists/`. See README.md.

pub mod audit;
pub mod autotag;
pub mod bulk;
pub mod cardinfo;
pub mod cd;
pub mod data;
pub mod decks;
pub mod digest;
pub mod dsl;
pub mod dsl_parse;
pub mod export;
pub mod facts;
pub mod json;
pub mod knowledge;
pub mod llm;
pub mod map;
pub mod ownership;
pub mod paths;
pub mod py;
pub mod re;
pub mod roles;
pub mod scryfall;
pub mod sources;
pub mod update_deck;
pub mod weekly;
