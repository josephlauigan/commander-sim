//! Dockside Extortionist is banned in Commander (Scryfall) and legal in Cali Commander (the format's own ban list in
//! data/formats.toml replaces Scryfall's): cardgen's `legality_problems` refuses it at a Commander table and passes it at
//! a Cali table, and sim-core's reading of the same two formats agrees. Real data only (no network).

use cardgen::data::{Project, load_formats};
use cardgen::decks::{Illegal, legality_problems};
use cardgen::scryfall::Cache;
use sim_core::export::{RawDeck, load_decks};
use sim_core::format::Format;
use std::path::PathBuf;

const NAME: &str = "Dockside Extortionist";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn commander_refuses_it_and_cali_passes_it() {
    let all = load_formats(&Project::new(root())).unwrap();
    let get = |k: &str| all.iter().find(|f| f.key == k).unwrap().clone();
    let cache = Cache::load(&root().join("data/scryfall_cache.json")).unwrap();
    let r = cache.get(NAME).expect("the Scryfall cache has the card");
    assert_eq!(legality_problems(&get("commander"), NAME, Some(r)), [Illegal::Banned], "banned in Commander");
    assert!(legality_problems(&get("cali"), NAME, Some(r)).is_empty(), "legal in Cali Commander");
}

#[test]
fn sim_core_seats_it_at_a_cali_table_and_a_cali_deck_never_at_a_commander_table() {
    let mut cali = load_decks(&root().join("data/decks.json")).unwrap().formats.remove("cali").unwrap();
    cali.key = "cali".into();
    let mut cmd = Format::commander();
    cmd.key = "commander".into();
    let deck = |format: Option<&str>| RawDeck {
        tier: None,
        key: "probe".into(),
        name: "probe".into(),
        commander: "X".into(),
        commander2: None,
        ident: None,
        tiers: vec![],
        format: format.map(String::from),
        cards: vec![NAME.to_string()],
    };
    assert!(cali.admits(&deck(None), &cmd).is_ok(), "a Commander-format deck holding it may sit at a Cali table");
    assert!(cali.admits(&deck(Some("cali")), &cali).is_ok());
    assert!(cmd.admits(&deck(Some("cali")), &cali).is_err(), "a Commander table refuses a Cali deck");
}
