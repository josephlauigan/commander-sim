//! Tutors that reveal: the log names the card for everyone (`tutors::REVEALS`, `log_found`), and every tutor line
//! says where the card went. The list is checked against the Scryfall cache and the decks.

use serde_json::Value;
use sim_core::control::LogSink;
use sim_core::engine::{tutors, zones};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;
use std::sync::{Arc, Mutex};

const P0: PlayerId = PlayerId(0);

fn json(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

/// a sentence of the Oracle text searches a library and reveals what it finds
fn says_reveal(oracle: &str) -> bool {
    let t = oracle.to_lowercase();
    t.split(['.', '\n']).any(|s| s.contains("search") && s.contains("library for") && s.contains("reveal"))
}

#[test]
fn the_reveal_list_matches_the_oracle_text() {
    let cache = json("scryfall_cache.json");
    let oracle = |name: &str| -> Option<String> {
        cache
            .as_object()
            .unwrap()
            .values()
            .find(|r| r["name"] == name)
            .and_then(|r| r["oracle_text"].as_str().map(String::from))
    };
    for &n in tutors::REVEALS {
        let t = oracle(n).unwrap_or_else(|| panic!("{n}: not in the Scryfall cache"));
        assert!(says_reveal(&t), "{n} is in REVEALS but its Oracle text doesn't reveal: {t}");
    }
    let decks = json("decks.json");
    let mut missing = vec![];
    for d in ["mine", "pool"].iter().flat_map(|k| decks[k].as_array().unwrap()) {
        for c in d["cards"].as_array().unwrap().iter().filter_map(Value::as_str) {
            if !tutors::reveals(c) && oracle(c).is_some_and(|t| says_reveal(&t)) && !missing.contains(&c) {
                missing.push(c);
            }
        }
    }
    assert!(missing.is_empty(), "tutors that reveal, missing from tutors::REVEALS: {missing:?}");
}

/// the lines the table sees (the public half of a secret line)
#[derive(Default)]
struct Public(Mutex<Vec<String>>);
impl LogSink for Public {
    fn line(&self, _g: &Game, line: &str, _secret: Option<(PlayerId, &str)>) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

fn traced(g: &mut Game) -> Arc<Public> {
    let s = Arc::new(Public::default());
    g.link.sink = Some(s.clone());
    s
}

fn last(s: &Public) -> String {
    s.0.lock().unwrap().last().cloned().unwrap_or_default()
}

#[test]
fn a_hidden_tutor_says_where_the_card_went_but_not_what_it_was() {
    let mut g = table(&["seph", "cloud"]);
    let s = traced(&mut g);
    tutors::tutor(&mut g, P0, "any", "Demonic Tutor").unwrap();
    let line = last(&s);
    assert!(line.ends_with("tutors a card into their hand"), "{line}");
    tutors::tutor_to_top(&mut g, P0, "any", 2, "Vampiric Tutor").unwrap();
    let line = last(&s);
    assert!(line.ends_with("puts a card on top of their library"), "{line}");
}

#[test]
fn a_revealing_tutor_names_the_card_for_everyone() {
    let mut g = table(&["seph", "cloud"]);
    let s = traced(&mut g);
    tutors::tutor_to_top(&mut g, P0, "any", 0, "Mystical Tutor").unwrap();
    let top = *g.player(P0).library.last().unwrap();
    let line = last(&s);
    assert!(line.ends_with(&format!("reveals {} and puts it on top of their library", g.db.get(top).name)), "{line}");
    tutors::tutor(&mut g, P0, "any", "Fabricate").unwrap();
    let got = *g.player(P0).hand.last().unwrap();
    let line = last(&s);
    assert!(line.ends_with(&format!("reveals {} and puts it into their hand", g.db.get(got).name)), "{line}");
}

#[test]
fn a_basic_land_to_hand_is_revealed() {
    let mut g = table(&["seph", "cloud"]);
    let s = traced(&mut g);
    zones::land_to_hand(&mut g, P0).unwrap();
    let got = *g.player(P0).hand.last().unwrap();
    let line = last(&s);
    assert!(line.ends_with(&format!("reveals {} and puts it into their hand", g.db.get(got).name)), "{line}");
}
