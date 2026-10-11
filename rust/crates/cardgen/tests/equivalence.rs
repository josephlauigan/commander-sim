//! `cardgen export` reproduces the committed data/cards.json and data/decks.json (which the Python exporter wrote
//! when the port started): card by card, field by field. If you edit the data files or the deck lists, re-run
//! `cardgen export` and commit the new JSON; this test then checks the files are current.

use cardgen::data::Project;
use cardgen::export::{Inputs, export};
use cardgen::json::{self, J};
use std::path::PathBuf;

fn project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

fn read_json(p: &Project, f: &str) -> J {
    json::parse(&std::fs::read_to_string(p.data(f)).unwrap()).unwrap()
}

#[test]
fn export_matches_committed_files() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    let e = export(&inp).unwrap();
    let got_cards = json::parse(&e.cards_json).unwrap();
    let want_cards = read_json(&p, "cards.json");
    let (g, w) =
        (got_cards.get("cards").unwrap().as_arr().unwrap(), want_cards.get("cards").unwrap().as_arr().unwrap());
    let mut diffs = vec![];
    let names = |v: &Vec<J>| v.iter().map(|c| c.s_or("name").to_string()).collect::<Vec<_>>();
    if names(g) != names(w) {
        let (gn, wn) = (names(g), names(w));
        diffs.push(format!(
            "card list differs: only in export {:?}, only in file {:?}",
            gn.iter().filter(|n| !wn.contains(n)).take(10).collect::<Vec<_>>(),
            wn.iter().filter(|n| !gn.contains(n)).take(10).collect::<Vec<_>>()
        ));
    }
    for (a, b) in g.iter().zip(w) {
        let (ao, bo) = (a.as_obj().unwrap(), b.as_obj().unwrap());
        for (k, v) in bo.iter() {
            if ao.get(k) != Some(v) {
                diffs.push(format!("{} .{k}: export {:?} / file {:?}", b.s_or("name"), ao.get(k), v));
            }
        }
        if ao.keys().ne(bo.keys()) {
            diffs.push(format!("{}: fields differ", b.s_or("name")));
        }
    }
    assert!(
        diffs.is_empty(),
        "{} difference(s), e.g.:\n{}",
        diffs.len(),
        diffs.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
    );
    assert_eq!(json::parse(&e.decks_json).unwrap(), read_json(&p, "decks.json"), "decks.json differs");
    // and byte for byte (key order, number formatting)
    assert_eq!(e.cards_json, std::fs::read_to_string(p.data("cards.json")).unwrap());
    assert_eq!(e.decks_json, std::fs::read_to_string(p.data("decks.json")).unwrap());
}

#[test]
fn every_list_card_has_data() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    assert!(inp.uncached().is_empty(), "cards missing from the Scryfall cache: {:?}", inp.uncached());
}

#[test]
fn deck_guard_fixture_is_current() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    let want = std::fs::read_to_string(p.root.join("tests/fixtures/my_decks_parsed.json")).unwrap();
    assert_eq!(cardgen::update_deck::fixture_json(&inp).unwrap(), want);
}
