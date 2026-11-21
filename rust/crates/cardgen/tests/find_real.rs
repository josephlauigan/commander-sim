//! `cardgen find` against the real catalog (`data/knowledge/oracle.json`, from `cardgen bulk-sync`): the phase 2
//! acceptance checks. Run with `cargo test --release -p cardgen --test find_real -- --ignored`. A worktree without its
//! own `oracle.json` reads the main checkout's.

use cardgen::data::Project;
use cardgen::knowledge::{self, Index, Query};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Instant;

const NEEDS: &str = "needs data/knowledge/oracle.json (run cardgen bulk-sync)";

fn project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

fn run(args: &[&str]) -> String {
    let a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    knowledge::find(&project(), &a).unwrap_or_else(|e| panic!("find {args:?}: {e}"))
}

fn json(args: &[&str]) -> Vec<Value> {
    let mut a = args.to_vec();
    a.push("--json");
    serde_json::from_str::<Value>(&run(&a)).unwrap().as_array().unwrap().clone()
}

fn names(args: &[&str]) -> Vec<String> {
    json(args).iter().map(|c| c["name"].as_str().unwrap().to_string()).collect()
}

fn oracle() -> Value {
    let dir = knowledge::knowledge_dir(&project(), None).expect("a catalog");
    serde_json::from_str(&std::fs::read_to_string(dir.join("oracle.json")).unwrap()).unwrap()
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn sol_ring_is_one_card() {
    let _ = NEEDS;
    assert_eq!(names(&["--name", "Sol Ring"]), ["Sol Ring"]);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn deck_cloud_stays_in_white_and_out_of_the_deck() {
    let decks: Value = serde_json::from_str(&std::fs::read_to_string(project().data("decks.json")).unwrap()).unwrap();
    let cloud = decks["mine"].as_array().unwrap().iter().find(|d| d["key"] == "cloud").unwrap();
    let in_deck: Vec<&str> = cloud["cards"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
    let all = json(&["--deck", "cloud", "--limit", "5000"]);
    assert!(all.len() > 1000);
    for c in &all {
        assert!(["", "W"].contains(&c["identity"].as_str().unwrap()), "{c}");
        assert!(!in_deck.contains(&c["name"].as_str().unwrap()), "{} is in the deck", c["name"]);
        assert_eq!(c["legal"], "legal");
    }
}

/// every deck of `mine`: no off-identity card, no card of the deck's list, matched by name or face name
#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn every_deck_of_mine_is_clean() {
    let decks: Value = serde_json::from_str(&std::fs::read_to_string(project().data("decks.json")).unwrap()).unwrap();
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let ident = d["ident"].as_str().unwrap();
        let mut have: Vec<String> =
            d["cards"].as_array().unwrap().iter().map(|c| c.as_str().unwrap().to_lowercase()).collect();
        have.extend(
            d["cards"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|c| c.as_str().unwrap().split(" // ").map(str::to_lowercase).collect::<Vec<_>>()),
        );
        for c in json(&["--deck", key, "--limit", "100000"]) {
            let id = c["identity"].as_str().unwrap();
            assert!(id.chars().all(|ch| ident.contains(ch)), "{key}: {} has identity {id}", c["name"]);
            let n = c["name"].as_str().unwrap().to_lowercase();
            assert!(!have.contains(&n), "{key}: {n} is in the deck");
        }
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn wu_cheap_counterspells_are_ranked() {
    let rows = json(&[
        "--ident",
        "WU",
        "--cmc",
        "<=2",
        "--type",
        "instant",
        "--text",
        "counter target",
        "--sort",
        "edhrec",
        "--limit",
        "5",
    ]);
    assert_eq!(rows.len(), 5);
    let ranks: Vec<u64> = rows.iter().map(|c| c["edhrec_rank"].as_u64().expect("ranked")).collect();
    assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{ranks:?}");
    for c in &rows {
        assert!(c["cmc"].as_f64().unwrap() <= 2.0);
        assert!(c["type_line"].as_str().unwrap().contains("Instant"));
        assert!(c["identity"].as_str().unwrap().chars().all(|ch| "WU".contains(ch)));
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn banned_cards_need_legal_any() {
    assert!(names(&["--name", "Primeval Titan"]).is_empty());
    let t = json(&["--legal", "any", "--name", "Primeval Titan"]);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0]["legal"], "banned");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn gc_count_equals_the_catalogs() {
    let o = oracle();
    let n = o["cards"].as_array().unwrap().iter().filter(|c| c["game_changer"] == true).count();
    assert!(n > 40, "{n}");
    assert_eq!(json(&["--gc", "--legal", "any", "--meld", "--limit", "5000"]).len(), n);
    // the default leaves out the meld results; every Game Changer is Commander-legal
    let melds = o["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["game_changer"] == true && c["all_parts_meld"] == true)
        .count();
    assert_eq!(json(&["--gc", "--limit", "5000"]).len(), n - melds);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn names_resolve_faces_and_exact_beats_face() {
    assert_eq!(names(&["--name", "Petty Theft"]), ["Brazen Borrower // Petty Theft"]);
    let bolt = json(&["--name", "Lightning Bolt"]);
    assert_eq!(bolt[0]["name"], "Lightning Bolt");
    assert_eq!(bolt.len(), 1);
    let all = names(&["--legal", "any", "--name", "Lightning Bolt"]);
    assert_eq!(all, ["Lightning Bolt"]);
    // the prepare card is still there under its own name
    assert!(!names(&["--name", "Emeritus of Conflict"]).is_empty());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn shared_names_and_no_memorabilia() {
    assert_eq!(json(&["--legal", "any", "--name", "Pick Your Poison"]).len(), 2);
    let default = json(&["--name", "Pick Your Poison"]);
    assert_eq!(default.len(), 1);
    assert_eq!(default[0]["legal"], "legal");
    for n in ["Savage Lands", "Inferno"] {
        let any = json(&["--legal", "any", "--name", n]);
        assert_eq!(any.len(), 1, "{n}: {any:?}");
        assert!(!any[0]["type_line"].as_str().unwrap().starts_with("Card"));
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn a_bad_regex_fails() {
    let a = vec!["--text".to_string(), "(".to_string()];
    let e = knowledge::find(&project(), &a).unwrap_err();
    assert!(e.to_string().contains("bad --text regex"), "{e}");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn meld_results_are_hidden_by_default() {
    let hidden = names(&["--name", "Brisela, Voice of Nightmares", "--legal", "any"]);
    assert!(hidden.is_empty());
    assert_eq!(names(&["--name", "Brisela, Voice of Nightmares", "--meld"]).len(), 1);
}

/// a full-catalog regex scan takes under a second once the catalog is loaded
#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn a_full_scan_is_fast() {
    let dir = knowledge::knowledge_dir(&project(), None).unwrap();
    let t = Instant::now();
    let idx = Index::load(&dir.join("oracle.json")).unwrap();
    let load = t.elapsed();
    assert!(idx.cards.len() > 25_000);
    let a: Vec<String> =
        ["--legal", "any", "--text", r"(?<!un)counter target (?!noncreature)\w+", "--text", r"\bdraw\b"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    let q = Query::parse(&a).unwrap();
    let re: Vec<_> = q.texts.iter().map(|t| fancy_regex::Regex::new(&format!("(?i){t}")).unwrap()).collect();
    let t = Instant::now();
    let hits = idx.matches(&q, None, &[], &re).unwrap();
    let scan = t.elapsed();
    eprintln!("load {load:?}, scan {scan:?}, {} hits", hits.len());
    assert!(scan.as_millis() < 1000, "scan took {scan:?}");
}
