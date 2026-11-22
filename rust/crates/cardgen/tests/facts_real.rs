//! Phase 4 against the real catalog and the repository's real engine data: `find` shows the model status, the decks
//! that play a card and the `FINDINGS.md` lines that name it; the digest and the map carry them. Run with
//! `cargo test --release -p cardgen --test facts_real -- --ignored` after `cardgen bulk-sync` and
//! `cardgen roles --rules`. JD's decks change, so nothing here names one of his decks or cards: decks come from
//! `data/decks.json` `mine` and cards are picked by a property.

mod common;

use cardgen::data::Project;
use cardgen::digest;
use cardgen::facts::Facts;
use cardgen::knowledge::{self, Index};
use cardgen::map;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn project() -> Project {
    Project::new(root())
}

fn find_json(args: &[&str]) -> Vec<Value> {
    let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    a.push("--json".into());
    let out = knowledge::find(&project(), &a).unwrap_or_else(|e| panic!("find {args:?}: {e}"));
    serde_json::from_str::<Value>(&out).unwrap().as_array().unwrap().clone()
}

/// `(key, card names including the commanders)` of every deck of `mine`
fn mine() -> Vec<(String, Vec<String>)> {
    let decks = common::decks(&root());
    decks["mine"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            let mut names: Vec<String> =
                d["cards"].as_array().unwrap().iter().map(|c| c.as_str().unwrap().into()).collect();
            names.extend(["commander", "commander2"].iter().filter_map(|k| d.get(*k)?.as_str().map(String::from)));
            names.sort();
            names.dedup();
            (d["key"].as_str().unwrap().to_string(), names)
        })
        .collect()
}

fn word_in(line: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric();
    line.match_indices(name).any(|(at, _)| {
        !line[..at].chars().next_back().is_some_and(word) && !line[at + name.len()..].chars().next().is_some_and(word)
    })
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn a_full_card_of_a_deck_with_findings_shows_its_status_deck_and_finding_line() {
    let mut tried = 0;
    for (key, names) in mine() {
        let rel = format!("audit/{key}/FINDINGS.md");
        let Ok(text) = std::fs::read_to_string(root().join(&rel)) else { continue };
        let lines: Vec<&str> = text.lines().collect();
        for name in names.iter().filter(|n| n.len() >= 6 && n.is_ascii() && !n.contains(" // ")) {
            // an independent scan: the first line that has the name as a whole word, and the heading above it
            let Some(at) = lines.iter().position(|l| word_in(l, name)) else { continue };
            let heading = lines[..at].iter().rev().find(|l| l.starts_with('#')).map_or("", |l| *l);
            let rows = find_json(&["--name", name, "--legal", "any"]);
            let Some(row) = rows.first() else { continue };
            if row["status"] != "Full" {
                continue;
            }
            tried += 1;
            assert!(row["decks"].as_array().unwrap().iter().any(|d| *d == key.as_str()), "{name}: {}", row["decks"]);
            let f = row["findings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["file"] == rel.as_str() && f["line"] == at + 1)
                .unwrap_or_else(|| panic!("{name}: no finding at {rel}:{} in {}", at + 1, row["findings"]));
            assert_eq!(f["heading"].as_str().unwrap(), heading.chars().take(200).collect::<String>(), "{name}");
            assert_eq!(f["text"].as_str().unwrap(), lines[at].chars().take(200).collect::<String>());
            assert!(row["status_note"].is_string());
            eprintln!("{key}: {name} is Full, in {:?}, finding {rel}:{} under {heading:?}", row["decks"], at + 1);
            if tried >= 3 {
                return;
            }
        }
    }
    assert!(tried > 0, "no deck of mine has a Full card named in its FINDINGS.md");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn only_cards_with_a_cards_json_record_have_a_status_and_none_of_the_rest_is_ever_full() {
    let project = project();
    let index = Index::open(&project, None).unwrap();
    let facts = Facts::load(&project, &index, true, true).unwrap();
    let cards: Value = serde_json::from_str(&std::fs::read_to_string(project.data("cards.json")).unwrap()).unwrap();
    let recorded: BTreeSet<u32> = cards["cards"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| digest::resolve(&index, c["name"].as_str().unwrap()))
        .collect();
    let with_status: BTreeSet<u32> = facts.engine.status.keys().copied().collect();
    assert_eq!(with_status, recorded, "a status exactly for the cards that have a record");
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for (s, _) in facts.engine.status.values() {
        *by.entry(s.as_str()).or_default() += 1;
    }
    eprintln!("catalog cards with a status: {} {by:?}", with_status.len());
    eprintln!(
        "engine records: {}; resolve to no catalog card ({}): {:?}",
        facts.engine.records,
        facts.engine.unresolved.len(),
        facts.engine.unresolved
    );
    eprintln!("two engine records for one catalog card: {:?}", facts.engine.conflicts);
    assert!(with_status.len() > 3_000);
    // a card without a record: no `status` key
    let name = index
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| !with_status.contains(&(*i as u32)) && c.legal == knowledge::Legal::Legal && !c.is_meld())
        .map(|(_, c)| c.raw.name.clone())
        .unwrap();
    let row = &find_json(&["--name", &name])[0];
    assert!(row.get("status").is_none() && row.get("status_note").is_none(), "{row}");
    // and across a whole search, no row without a record says Full
    for r in find_json(&["--status", "none", "--limit", "300"]) {
        assert!(r.get("status").is_none(), "{r}");
    }
}

/// the 20 cards of `mine` the status is compared on: a fixed-seed shuffle of every distinct name
fn sample() -> Vec<String> {
    let mut names: Vec<String> = mine().into_iter().flat_map(|(_, n)| n).collect();
    names.sort();
    names.dedup();
    let mut state: u64 = 0x5eed_cafe;
    let mut out = vec![];
    while out.len() < 20 && !names.is_empty() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        out.push(names.remove((state >> 33) as usize % names.len()));
    }
    out
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn the_status_agrees_with_cardgen_audit_on_20_sampled_cards_of_mine() {
    let names = sample();
    assert_eq!(names.len(), 20);
    let file = std::env::temp_dir().join(format!("facts-sample-{}.txt", std::process::id()));
    std::fs::write(&file, names.join("\n")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cardgen"))
        .args(["audit", "--cards"])
        .arg(&file)
        .current_dir(root())
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&file);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let audit: BTreeMap<String, (String, String)> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            let mut p = l.splitn(3, " | ");
            let (s, n, note) = (p.next().unwrap(), p.next().unwrap(), p.next().unwrap_or(""));
            (n.to_string(), (s.to_string(), note.to_string()))
        })
        .collect();
    assert_eq!(audit.len(), 20, "{audit:?}");
    let index = Index::open(&project(), None).unwrap();
    let facts = Facts::load(&project(), &index, true, false).unwrap();
    let mut compared = 0;
    for n in &names {
        let found = digest::resolve(&index, n);
        let Some(&i) = found.first() else {
            eprintln!("{n}: not in the catalog");
            continue;
        };
        let (s, note) = facts.status_of(i).unwrap_or_else(|| panic!("{n} has no status"));
        assert_eq!((s.to_string(), note.to_string()), audit[n], "{n}");
        compared += 1;
    }
    eprintln!("compared {compared} of 20 sampled cards: {names:?}");
    assert!(compared >= 18);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn a_deck_listing_filtered_by_status_lists_only_such_cards_and_in_deck_agrees_with_the_deck_bits() {
    for (key, names) in mine() {
        let all = find_json(&[
            "--deck",
            &key,
            "--status",
            "none,full,full-auto,approximate,partial,unmodeled",
            "--limit",
            "all",
        ]);
        let below = find_json(&["--deck", &key, "--status", "unmodeled,partial", "--limit", "all"]);
        assert!(below.iter().all(|r| ["Unmodeled", "Partial"].contains(&r["status"].as_str().unwrap_or(""))), "{key}");
        let expect: Vec<&Value> =
            all.iter().filter(|r| ["Unmodeled", "Partial"].contains(&r["status"].as_str().unwrap_or(""))).collect();
        assert_eq!(below.len(), expect.len(), "{key}");
        let ordered = |v: &[&Value]| v.iter().map(|r| r["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(ordered(&below.iter().collect::<Vec<_>>()), ordered(&expect), "{key}");
        eprintln!("{key}: {} candidates in its colours, {} Partial or Unmodeled", all.len(), below.len());
        // --in-deck lists the deck's own cards, each with the key in `decks`
        let own = find_json(&["--in-deck", &key, "--limit", "all", "--legal", "any"]);
        assert!(own.len() > 20 && own.len() <= names.len() + 3, "{key}: {}", own.len());
        assert!(own.iter().all(|r| r["decks"].as_array().unwrap().iter().any(|d| *d == key.as_str())));
        let out = find_json(&["--not-in-deck", &key, "--limit", "40"]);
        assert!(out.iter().all(|r| !r["decks"].as_array().unwrap().iter().any(|d| *d == key.as_str())));
    }
}

fn bytes_and_tokens(text: &str) -> (usize, usize) {
    (text.len(), digest::tokens(text))
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn every_deck_of_mine_has_a_digest_in_budget_with_the_flags_in_order() {
    let tmp = std::env::temp_dir().join(format!("facts-digest-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    for (key, _) in mine() {
        let out = tmp.join(format!("{key}.txt"));
        let t = Instant::now();
        let run =
            digest::command(&project(), &["--deck".into(), key.clone(), "--out".into(), out.display().to_string()])
                .unwrap();
        let secs = t.elapsed().as_secs_f64();
        let text = std::fs::read_to_string(&out).unwrap();
        assert_eq!(text, run.digest.text);
        let (bytes, tokens) = bytes_and_tokens(&text);
        let cards: Vec<&str> = text.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect();
        eprintln!("digest {key}: {} lines, {bytes} bytes, ~{tokens} tokens, {secs:.2} s", cards.len());
        assert!(tokens <= digest::DEFAULT_BUDGET_TOKENS && bytes < 200_000, "{key}: {bytes} bytes, {tokens} tokens");
        // the letter of this deck, from the header legend
        let legend = text.lines().find(|l| l.starts_with("# d: letters")).expect("the d: legend");
        let own = legend
            .split("card: ")
            .nth(1)
            .unwrap()
            .split(' ')
            .find_map(|p| p.split_once('=').filter(|(_, k)| *k == key).map(|(c, _)| c.to_string()))
            .unwrap();
        assert!(legend.ends_with(&format!("(this digest: {own})")), "{legend}");
        assert!(text.lines().any(|l| l.starts_with("# m: ")));
        let mut in_a = false;
        let mut with_m = 0;
        for l in text.lines() {
            if l.starts_with("## ") {
                in_a = l.starts_with("## A ");
                continue;
            }
            if l.starts_with('#') || l.is_empty() {
                continue;
            }
            let fields: Vec<&str> = l.split('|').collect();
            assert_eq!(fields.len(), 6, "{l}");
            assert!(fields.iter().all(|f| !f.is_empty()), "{l}");
            assert!(l.chars().count() <= 160, "{l}");
            let rank = |f: &str| match f {
                "*" => 0,
                "BAN" | "NL" => 1,
                "GC" => 2,
                "?" => 8,
                f if f.starts_with('#') => 3,
                f if f.starts_with('$') => 4,
                f if f.starts_with("m:") => 5,
                f if f.starts_with("d:") => 6,
                f if f.starts_with("k:") => 7,
                f => panic!("unknown flag {f} in {l}"),
            };
            let toks: Vec<&str> = fields[5].split(' ').filter(|t| *t != "-").collect();
            assert!(toks.windows(2).all(|w| rank(w[0]) < rank(w[1])), "{l}");
            for t in &toks {
                if let Some(m) = t.strip_prefix("m:") {
                    assert!(["F", "f", "A", "P", "U", "N"].contains(&m), "{l}");
                    with_m += 1;
                }
                if let Some(d) = t.strip_prefix("d:") {
                    assert!(!d.is_empty() && d.chars().all(|c| c.is_ascii_uppercase()), "{l}");
                    assert!(!in_a || !d.contains(own.as_str()), "{l}: d: lists this deck's own letter in section A");
                }
                if let Some(k) = t.strip_prefix("k:") {
                    assert!(k.parse::<usize>().unwrap() > 0, "{l}");
                }
            }
            assert!(!toks.contains(&"?") || toks.last() == Some(&"?"), "{l}");
            assert!(
                toks.iter().any(|t| t.starts_with("m:")),
                "every line has an m: flag when the engine facts exist: {l}"
            );
        }
        assert!(with_m > 0, "{key}: no card carries an m: flag");
        // a second run is byte-identical
        let again =
            digest::command(&project(), &["--deck".into(), key.clone(), "--out".into(), out.display().to_string()])
                .unwrap();
        assert_eq!(again.digest.text, text);
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn the_map_has_the_engine_coverage_section_and_stays_small_and_deterministic() {
    let tmp = std::env::temp_dir().join(format!("facts-map-{}.md", std::process::id()));
    let args = ["--out".to_string(), tmp.display().to_string()];
    let (first, _) = map::command(&project(), &args).unwrap();
    let (second, _) = map::command(&project(), &args).unwrap();
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(first.text, second.text);
    assert!(first.text.len() < 32 * 1024, "{} bytes", first.text.len());
    let section =
        first.text.split("## Engine model coverage").nth(1).expect("the section").split("\n## ").next().unwrap();
    eprintln!("## Engine model coverage{section}");
    for s in ["Full", "Full-auto", "Approximate", "Partial", "Unmodeled", "no record"] {
        assert!(section.contains(&format!("| {s} |")), "{s}");
    }
    assert!(first.text.contains("| below Full (f/A/P/U) | f A P U | no record | FINDINGS mentions |"));
    eprintln!("map: {} bytes", first.text.len());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn a_cold_find_with_its_json_facts_and_a_digest_take_about_a_second_or_less() {
    // each in a fresh process: nothing is cached between runs
    let time = |args: &[&str]| {
        let t = Instant::now();
        let out = Command::new(env!("CARGO_BIN_EXE_cardgen")).args(args).current_dir(root()).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
        t.elapsed().as_secs_f64()
    };
    let (key, _) = mine().remove(0);
    let json = time(&["find", "--name", "Sol Ring", "--json"]);
    let status = time(&["find", "--status", "partial", "--limit", "5"]);
    let table = time(&["find", "--name", "Sol Ring"]);
    let dig = time(&[
        "digest",
        "--deck",
        &key,
        "--out",
        &std::env::temp_dir().join("facts-timing.txt").display().to_string(),
    ]);
    eprintln!("cold find --json {json:.2} s; find --status {status:.2} s; plain find {table:.2} s; digest {dig:.2} s");
    for (what, secs) in [("find --json", json), ("find --status", status), ("plain find", table)] {
        assert!(secs < 1.5, "{what} took {secs:.2} s");
    }
    assert!(dig < 3.0, "digest took {dig:.2} s");
}
