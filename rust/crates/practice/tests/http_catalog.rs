//! Port of tests/test_play_catalog.py at the HTTP level (/api/options): your decks with commander, bracket, Game
//! Changers and simulation win rates, and the five tiers with their decks. (Python's win-rate parser test feeds it
//! a temp markdown file; here the shape of what the table produced is checked.)
mod common;
use common::*;

macro_rules! srv {
    () => {
        match server() {
            Some(s) => s,
            None => return,
        }
    };
}

#[test]
fn test_your_decks() {
    let s = srv!();
    let o = s.client().get("/api/options").json();
    let d: std::collections::BTreeMap<&str, &serde_json::Value> =
        o["decks"].as_array().unwrap().iter().map(|x| (x["key"].as_str().unwrap(), x)).collect();
    assert_eq!(
        d.keys().copied().collect::<Vec<_>>(),
        ["alela", "galadriel", "jodah", "kefka", "sauron", "seph", "veyran", "yshtola", "zada"]
    );
    assert_eq!(d["seph"]["name"], "Sephiroth, the Savior"); // the deck file's name for Atraxa
    assert_eq!(d["seph"]["commander"], "Atraxa, Grand Unifier"); // the card the engine (and image) uses
    assert_eq!(d["seph"]["bracket"], 4);
    assert_eq!(d["veyran"]["bracket"], 3);
    assert!(d["seph"]["game_changers"].as_array().unwrap().len() > 3);
    for x in d.values() {
        for k in ["key", "commander", "name", "bracket", "game_changers", "win_rates"] {
            assert!(x.get(k).is_some(), "{} lacks {k}", x["key"]);
        }
    }
}

#[test]
fn test_win_rates_from_the_results_table() {
    let s = srv!();
    let o = s.client().get("/api/options").json();
    let sauron = o["decks"].as_array().unwrap().iter().find(|x| x["key"] == "sauron").unwrap();
    let w = sauron["win_rates"].as_object().unwrap();
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let v = w[t].as_f64().unwrap_or_else(|| panic!("no win rate for {t}"));
        assert!((0.0..=100.0).contains(&v), "{t}: {v}");
    }
}

#[test]
fn test_tiers() {
    let s = srv!();
    let o = s.client().get("/api/options").json();
    let tiers = o["tiers"].as_array().unwrap();
    assert_eq!(tiers.iter().map(|t| t["key"].as_str().unwrap()).collect::<Vec<_>>(), ["t1", "t2", "t3", "t4", "t5"]);
    assert!(tiers.iter().all(|t| t["decks"].as_array().unwrap().len() == 5));
    assert_eq!(tiers[0]["label"], "T1 · High B2 / Low B3");
    for t in tiers {
        for d in t["decks"].as_array().unwrap() {
            for k in ["key", "commander", "name"] {
                assert!(d[k].is_string(), "{k}");
            }
        }
    }
}
