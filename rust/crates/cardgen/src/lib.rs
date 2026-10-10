//! Card data and deck lists without Python: builds `data/cards.json` and `data/decks.json` from the hand-edited data
//! files (`data/carddb.txt`, `data/card_overrides.toml`, `data/my_decks.toml`), the Scryfall cache and the deck lists
//! in `decklists/`. See README.md.

pub mod json;
pub mod py;
pub mod re;
pub mod audit;
pub mod autotag;
pub mod cardinfo;
pub mod cd;
pub mod data;
pub mod decks;
pub mod dsl;
pub mod dsl_parse;
pub mod export;
pub mod scryfall;
pub mod sources;
pub mod update_deck;
