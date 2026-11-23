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

/// a hand-tagged card has no Game Changer flag or colour identity of its own: the export takes Scryfall's (by full
/// name, or by front face for a double-faced card), and a card Scryfall does not flag stays false
#[test]
fn hand_tagged_cards_take_their_game_changer_flag_and_identity_from_scryfall() {
    let p = project();
    let e = export(&Inputs::load(&p).unwrap()).unwrap();
    let cards = json::parse(&e.cards_json).unwrap();
    let card = |name: &str| {
        let c = cards.get("cards").unwrap().as_arr().unwrap().iter().find(|c| c.s_or("name") == name).unwrap();
        assert_eq!(c.s_or("source"), "manual", "{name} is a hand-tagged card");
        c.clone()
    };
    for (name, id) in [("Gifts Ungiven", "U"), ("Demonic Tutor", "B"), ("Rhystic Study", "U")] {
        let c = card(name);
        assert_eq!(c.get("game_changer"), Some(&J::from(true)), "{name} is a Game Changer");
        assert_eq!(c.s_or("identity"), id, "{name}'s colour identity");
    }
    let tergrid = card("Tergrid, God of Fright // Tergrid's Lantern");
    assert_eq!(tergrid.get("game_changer"), Some(&J::from(true)), "a double-faced card by its front face");
    assert_eq!(tergrid.s_or("identity"), "B");
    let ring = card("Sol Ring");
    assert_eq!(ring.get("game_changer"), Some(&J::from(false)), "not a Game Changer");
    assert_eq!(ring.s_or("identity"), "");
}

/// every exported card that Scryfall knows agrees with it on the Game Changer flag and the colour identity; the cards it
/// does not know have neither
#[test]
fn every_exported_card_matches_scryfall_on_game_changer_and_identity() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    let e = export(&inp).unwrap();
    let cards = json::parse(&e.cards_json).unwrap();
    let (mut known, mut unknown) = (0, vec![]);
    for c in cards.get("cards").unwrap().as_arr().unwrap() {
        let name = c.s_or("name");
        let rec = inp.cache.get(name).or_else(|| inp.cache.get(name.split(" // ").next().unwrap()));
        match rec {
            Some(r) => {
                known += 1;
                assert_eq!(
                    c.get("game_changer"),
                    Some(&J::from(r.get("game_changer").is_some_and(|g| g.truthy()))),
                    "{name}: game_changer"
                );
                assert_eq!(c.s_or("identity"), r.strs("color_identity").concat(), "{name}: identity");
            }
            None => unknown.push(name.to_string()),
        }
    }
    assert!(known > 3000, "{known} cards checked");
    for n in &unknown {
        let c = cards.get("cards").unwrap().as_arr().unwrap().iter().find(|c| c.s_or("name") == n).unwrap();
        assert!(
            c.get("game_changer").is_some_and(|g| g.is_null()) && c.get("identity").is_some_and(|g| g.is_null()),
            "{n}"
        );
    }
    eprintln!("not in the Scryfall cache, so null: {unknown:?}");
}

/// the deck check counts a hand-tagged Game Changer (it reads Scryfall's record, as the export now does)
#[test]
fn the_deck_check_counts_hand_tagged_game_changers() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    let cards: Vec<String> = ["Gifts Ungiven", "Demonic Tutor", "Sol Ring", "Swamp"].map(String::from).to_vec();
    let recs = |n: &str| inp.cache.get(n).cloned();
    let (probs, info) = cardgen::decks::card_problems(&cards, &[], Some("t1"), None, &recs);
    assert_eq!(info.gcs, ["Demonic Tutor", "Gifts Ungiven"]);
    assert!(probs.iter().any(|p| p.contains("2 Game Changers, tier t1 allows 0-0")), "{probs:?}");
}
