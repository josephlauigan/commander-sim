//! Adversarial phase-4 contract checks. Synthetic projects never alter repository data; real-data checks are
//! opt-in, select cards/decks by property, and use an independent Python oracle for name folding and view counts.

use cardgen::audit::{Audit, rank};
use cardgen::data::Project;
use cardgen::digest::{self, Prev, Spec};
use cardgen::export::Inputs;
use cardgen::facts::{Facts, parse_status_list};
use cardgen::knowledge::{self, FindError, Index};
use cardgen::{map, roles};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/adversarial_facts");
const CAP: [&str; 8] = ["--user", "--scope", "-q", "-p", "MemoryMax=24G", "-p", "MemorySwapMax=0", "--"];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

fn card(name: &str) -> Value {
    json!({"name": name, "oracle_id": format!("synthetic-{name}"), "mana_cost": "{1}", "cmc": 1,
        "type_line": "Artifact", "oracle_text": "Draw a card.", "color_identity": [], "keywords": [],
        "layout": "normal", "legalities": {"commander": "legal"}, "game_changer": false,
        "released_at": "2026-01-01", "set": "tst", "rarity": "common", "reserved": false, "edhrec_rank": 1})
}

struct Env(PathBuf);

impl Env {
    fn new(cards: Vec<Value>) -> Self {
        static SEQ: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "facts-adversarial-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        let e = Self(root);
        e.write(
            "data/knowledge/oracle.json",
            json!({"meta": {"schema": 1, "bulk_updated_at": "x",
            "download_uri": "x", "synced": "x", "count": cards.len()}, "cards": cards})
            .to_string(),
        );
        e.decks(json!([]));
        e
    }

    fn write(&self, path: &str, contents: impl AsRef<[u8]>) {
        let p = self.0.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, contents).unwrap();
    }

    fn decks(&self, mine: Value) {
        self.write("data/decks.json", json!({"version": 1, "mine": mine, "pool": []}).to_string());
    }

    fn engine(&self) {
        self.write("data/carddb.txt", "Test Engine Relic|A|1|rock=1:C\n");
        for p in ["card_overrides.toml", "my_decks.toml", "python_hooks.txt", "audit.toml"] {
            self.write(&format!("data/{p}"), "");
        }
        // an empty my_decks.toml still needs the format table cardgen loads with it
        self.write(
            "data/formats.toml",
            "[commander]\nname = \"Commander\"\nseats = 4\nlife = 40\nfirst_player_draws = true\nbans = \"scryfall\"\n",
        );
        self.write("data/scryfall_cache.json", "{}");
        self.write("data/cards.json", r#"{"version":1,"cards":[]}"#);
        std::fs::create_dir_all(self.0.join("decklists/pool")).unwrap();
    }

    fn project(&self) -> Project {
        Project::new(&self.0)
    }

    fn index(&self) -> Index {
        Index::load(&self.0.join("data/knowledge/oracle.json")).unwrap()
    }

    fn facts(&self) -> Facts {
        Facts::load(&self.project(), &self.index(), true, true).unwrap()
    }

    fn find(&self, args: &[&str]) -> Result<knowledge::FindOutput, FindError> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.extend(["--data-dir".into(), self.0.join("data/knowledge").display().to_string()]);
        knowledge::find_with_notes(&self.project(), &a)
    }

    fn rows(&self, args: &[&str]) -> Vec<Value> {
        let mut a = args.to_vec();
        a.push("--json");
        serde_json::from_str::<Vec<Value>>(&self.find(&a).unwrap().out).unwrap()
    }

    fn attach_roles(&self) {
        let table = roles::generate(&self.index(), "").unwrap();
        self.write("data/knowledge/roles.json", roles::bytes(&table).unwrap());
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn binary(root: &Path, args: &[&str]) -> Output {
    Command::new("systemd-run")
        .args(CAP)
        .arg(env!("CARGO_BIN_EXE_cardgen"))
        .arg("--root")
        .arg(root)
        .args(args)
        .current_dir(root)
        .env_remove("SIM_DECKS")
        .env_remove("CARDGEN_KNOWLEDGE_DIR")
        .output()
        .unwrap()
}

fn python(mode: &str, root: &Path, artifacts: &Path) -> Value {
    let out = Command::new("python3")
        .arg("-I")
        .arg(Path::new(FIXTURES).join("independent.py"))
        .arg(mode)
        .arg(root)
        .arg(artifacts)
        .output()
        .unwrap();
    assert!(out.status.success(), "independent {mode}: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    eprintln!("independent {mode}: {v}");
    v
}

fn only_id(e: &Env, name: &str) -> u32 {
    let index = e.index();
    let ids = index.lookup(name);
    assert_eq!(ids.len(), 1);
    ids[0]
}

#[test]
fn findings_matrix_crlf_boundaries_markup_faces_and_exact_metadata() {
    let names = [
        "Opt",
        "Option",
        "Ring",
        "Spring",
        "Wrath",
        "Wrathful",
        "Aether Relic",
        "Andúril, Test Blade",
        "Urza's Test",
        "Fire // Ice",
        "Sol Ring",
    ];
    let mut cards: Vec<Value> = names.iter().map(|n| card(n)).collect();
    cards[9]["layout"] = json!("split");
    cards[9]["card_faces"] = json!([{"name": "Fire", "type_line": "Instant"},
        {"name": "Ice", "type_line": "Instant"}]);
    let e = Env::new(cards);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []}]));
    // Expected names are labelled independently per line, including deliberate non-matches. A finding is a raw
    // physical line: split names/emphasis are not silently joined, and code fences do not suppress longer names.
    let lines: Vec<(&str, Vec<&str>)> = vec![
        ("Sol Ring before a heading", vec!["Sol Ring"]),
        ("# First heading", vec![]),
        ("Option Spring Wrathful", vec!["Option", "Spring", "Wrathful"]),
        ("Opt Ring Wrath", vec!["Wrath"]),
        ("**Opt** **Ring** `Fire` `Ice`", vec!["Opt", "Ring", "Fire // Ice"]),
        ("Opt => Ring", vec!["Opt", "Ring"]),
        ("**Option** **Spring** Wrathful", vec!["Option", "Spring", "Wrathful"]),
        ("Sol Rings XS​ol Ring Spring", vec!["Spring"]),
        ("sol ring SOL RING", vec![]),
        ("Anduril, Test Blade and Urza’s Test", vec!["Andúril, Test Blade", "Urza's Test"]),
        ("Æther Relic and Aether Relic", vec!["Aether Relic"]),
        ("## Sol Ring heading", vec!["Sol Ring"]),
        ("| Sol Ring | [Sol Ring](https://invalid.example/) |", vec!["Sol Ring"]),
        ("```text", vec![]),
        ("Sol Ring", vec!["Sol Ring"]),
        ("```", vec![]),
        ("Sol **Ring**", vec!["Ring"]),
        ("Sol", vec![]),
        ("Ring", vec![]),
        ("Fire // Ice", vec!["Fire // Ice"]),
        ("Sol Ring Sol Ring Sol Ring", vec!["Sol Ring"]),
    ];
    e.write("audit/alpha/FINDINGS.md", lines.iter().map(|(l, _)| *l).collect::<Vec<_>>().join("\r\n"));
    let f = e.facts();
    for name in names {
        let mut heading = "";
        let mut expected = vec![];
        for (at, (text, hits)) in lines.iter().enumerate() {
            if hits.contains(&name) {
                expected.push((at + 1, heading.to_string(), text.to_string()));
            }
            if text.starts_with('#') {
                heading = text;
            }
        }
        let actual: Vec<_> = f
            .findings_of(only_id(&e, name))
            .iter()
            .map(|h| {
                assert_eq!(h.file, "audit/alpha/FINDINGS.md");
                (h.line, h.heading.clone(), h.text.clone())
            })
            .collect();
        assert_eq!(actual, expected, "{name}");
    }
}

#[test]
fn findings_large_multibyte_lines_and_thousand_mentions_count_lines_once() {
    let e = Env::new(vec![card("Sol Ring")]);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []}]));
    let heading = format!("# {}", "é".repeat(250));
    let long = format!("{} Sol Ring", "é".repeat(524_288));
    let many = "Sol Ring ".repeat(1000);
    e.write("audit/alpha/FINDINGS.md", format!("{heading}\n{long}\n{many}\n{}", "Sol Ring\n".repeat(1000)));
    let f = e.facts();
    let hits = f.findings_of(0);
    assert_eq!(hits.len(), 1002);
    assert_eq!(hits[0].line, 2);
    assert_eq!(hits[0].text, "é".repeat(200));
    assert_eq!(hits[0].heading, heading.chars().take(200).collect::<String>());
    assert_eq!(f.context(None).findings[&0], 1002);
    assert_eq!(e.rows(&[])[0]["findings"].as_array().unwrap().len(), 1002);
}

#[test]
fn defect_double_backtick_code_spans_lose_short_names() {
    let e = Env::new(vec![card("Opt")]);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []}]));
    e.write("audit/alpha/FINDINGS.md", "`Opt`\n``Opt``\n```Opt```\n``a `quoted` Opt``\n");
    let actual: Vec<_> = e.facts().findings_of(0).iter().map(|f| f.line).collect();
    assert_eq!(actual, [1, 2, 3, 4], "all four are Markdown inline code spans");
}

#[test]
fn defect_bom_hides_first_heading() {
    let e = Env::new(vec![card("Sol Ring")]);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []}]));
    e.write("audit/alpha/FINDINGS.md", "\u{feff}# First heading\r\nSol Ring\r\n");
    assert_eq!(e.facts().findings_of(0)[0].heading, "# First heading");
}

#[test]
fn defect_finding_text_drops_markdown_trailing_spaces() {
    let e = Env::new(vec![card("Sol Ring")]);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []}]));
    e.write("audit/alpha/FINDINGS.md", "Sol Ring  \n");
    assert_eq!(e.rows(&[])[0]["findings"][0]["text"], "Sol Ring  ", "text is the line cut to 200 chars");
}

#[test]
fn findings_missing_empty_directory_invalid_utf8_and_symlink() {
    let e = Env::new(vec![card("Sol Ring")]);
    e.decks(json!([{"key":"alpha", "commander":"", "ident":"", "cards": []},
        {"key":"missing", "commander":"", "ident":"", "cards": []}]));
    assert!(e.facts().files.is_empty());
    e.write("audit/alpha/FINDINGS.md", "");
    assert!(e.facts().findings.is_empty());
    e.write("audit/alpha/FINDINGS.md", [0xff]);
    assert!(Facts::decks_and_findings(&e.project(), &e.index()).unwrap_err().contains("FINDINGS.md"));
    let p = e.0.join("audit/alpha/FINDINGS.md");
    std::fs::remove_file(&p).unwrap();
    std::fs::create_dir(&p).unwrap();
    assert!(Facts::decks_and_findings(&e.project(), &e.index()).is_err());
    std::fs::remove_dir(p).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        e.write("audit/target.md", "Sol Ring\n");
        symlink("../target.md", e.0.join("audit/alpha/FINDINGS.md")).unwrap();
        assert_eq!(e.facts().findings_of(0)[0].file, "audit/alpha/FINDINGS.md");
        let target = e.0.join("audit/target.md");
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0)).unwrap();
        assert!(Facts::decks_and_findings(&e.project(), &e.index()).is_err());
        std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[test]
fn deck_letters_collisions_digits_dashes_empty_mine_and_commander2() {
    let e = Env::new(vec![card("Sol Ring"), card("Test Commander"), card("Test Partner")]);
    let keys = ["alpha", "alpha-two", "a-3", "123", "---", "z9"];
    e.decks(json!(
        keys.iter()
            .map(|k| json!({"key": k, "commander":"Test Commander",
        "commander2":"Test Partner", "ident":"", "cards":["Sol Ring", "Sol Ring"]}))
            .collect::<Vec<_>>()
    ));
    let f = e.facts();
    assert_eq!(f.letters.len(), keys.len());
    let letters: BTreeSet<_> = f.letters.iter().map(|(l, _)| *l).collect();
    assert_eq!(letters.len(), keys.len());
    assert!(letters.iter().all(|c| c.is_ascii_uppercase()));
    assert_eq!(f.letters, e.facts().letters);
    for r in e.rows(&["--limit", "all"]) {
        assert_eq!(r["decks"], json!(keys));
    }
    assert!(f.context(Some("alpha")).legend().unwrap().contains("A=alpha"));
    e.decks(json!([]));
    let empty = e.facts();
    assert!(empty.context(None).legend().is_none());
    assert!(empty.context(None).model_legend().is_none());
    assert!(empty.deck_bit("alpha").is_err());
}

#[test]
fn synthetic_deck_membership_faces_accents_shared_names_illegal_cards_and_basics_python() {
    let mut face = card("Test Front // Test Back");
    face["card_faces"] = json!([{"name":"Test Front", "type_line":"Artifact"},
        {"name":"Test Back", "type_line":"Artifact"}]);
    let mut banned = card("Test Banned");
    banned["legalities"]["commander"] = json!("banned");
    let mut illegal = card("Test Illegal");
    illegal["legalities"]["commander"] = json!("not_legal");
    let mut shared = card("Test Shared");
    shared["oracle_id"] = json!("synthetic-shared-illegal");
    shared["legalities"]["commander"] = json!("not_legal");
    let e = Env::new(vec![
        face,
        banned,
        illegal,
        card("Test Shared"),
        shared,
        card("Æther Conduit"),
        card("Urza's Test"),
        card("Test Basic"),
        card("Andúril, Test Blade"),
    ]);
    e.decks(json!([
        {"key":"alpha", "commander":"Urza’s Test", "commander2":"Test Back", "ident":"", "cards":[
            "Test Basic", "Test Basic", "Aether Conduit", "Anduril, Test Blade", "Test Banned", "Missing Card"]},
        {"key":"beta", "commander":"", "ident":"", "cards":["Test Illegal", "Test Shared", "Test Front"]}
    ]));
    e.write("data/cards.json", r#"{"cards":[]}"#);
    e.write("rows.json", serde_json::to_vec(&e.rows(&["--legal", "any", "--meld", "--limit", "all"])).unwrap());
    for key in ["alpha", "beta"] {
        e.write(
            &format!("own-{key}.json"),
            serde_json::to_vec(&e.rows(&["--in-deck", key, "--legal", "any", "--meld", "--limit", "all"])).unwrap(),
        );
        e.write(
            &format!("safe-{key}.json"),
            serde_json::to_vec(&e.rows(&["--deck", key, "--status", "full,full-auto", "--limit", "all"])).unwrap(),
        );
    }
    python("rows", &e.0, &e.0);
    let notes = e.find(&["--deck", "alpha", "--json"]).unwrap().notes;
    assert!(notes.iter().any(|n| n.contains("Missing Card")));
}

#[test]
fn defect_more_than_sixteen_decks_silently_lose_usage_and_findings() {
    let e = Env::new(vec![card("Sol Ring")]);
    e.decks(json!(
        (0..28)
            .map(|n| json!({"key":format!("deck-{n}"), "commander":"",
        "ident":"", "cards":["Sol Ring"]}))
            .collect::<Vec<_>>()
    ));
    e.write("audit/deck-27/FINDINGS.md", "Sol Ring\n");
    let r = &e.rows(&[])[0];
    let actual = (
        r["decks"].as_array().unwrap().len(),
        r["findings"].as_array().unwrap().len(),
        e.find(&["--in-deck", "deck-27"]).is_ok(),
    );
    assert_eq!(actual, (28, 1, true), "no documented find/decks.json deck-count cap");
}

#[test]
fn defect_status_for_card_absent_from_exported_engine_database() {
    let e = Env::new(vec![card("Test Engine Relic")]);
    e.engine();
    // Sources can be newer than the engine export. The engine reads cards.json, which has no records here.
    let r = &e.rows(&[])[0];
    assert!(r.get("status").is_none(), "no exported engine record, but find says {r}");
    assert!(e.facts().context(None).model_status.is_empty());
}

/// Coordinator decision (round 2): a missing, malformed, wrong-shape or stale `data/cards.json` makes the engine facts
/// unavailable: plain `find` works with one stderr note and no `status`; `--status` is a clear error; a valid export
/// gives the status. Edited from the first version, which expected every case to fail `find --json`.
#[test]
fn defect_missing_or_malformed_cards_json_is_ignored() {
    let e = Env::new(vec![card("Test Engine Relic")]);
    e.engine();
    e.attach_roles();
    let stale_one = r#"{"version":1,"cards":[{"name":"Test Engine Relic"},{"name":"Test Extra Card"}]}"#;
    let cases: [(&str, Option<&str>, &str); 6] = [
        ("missing", None, "data/cards.json is missing"),
        ("brace", Some("{"), "not valid JSON"),
        ("empty array", Some("[]"), "is not {"),
        ("truncated", Some(r#"{"cards":[{"name":"Test Engine Relic"}"#), "not valid JSON"),
        ("empty export", Some(r#"{"version":1,"cards":[]}"#), "is stale"),
        ("stale by one card", Some(stale_one), "is stale"),
    ];
    for (what, text, why) in cases {
        match text {
            Some(t) => e.write("data/cards.json", t),
            None => {
                let _ = std::fs::remove_file(e.0.join("data/cards.json"));
            }
        }
        let out = e.find(&["--json"]).unwrap_or_else(|err| panic!("{what}: plain find must work: {err}"));
        let rows: Vec<Value> = serde_json::from_str(&out.out).unwrap();
        assert!(rows[0].get("status").is_none() && rows[0].get("status_note").is_none(), "{what}: {}", rows[0]);
        assert!(rows[0]["decks"].is_array() && rows[0]["findings"].is_array(), "{what}");
        assert_eq!(rows[0]["engine"], "unavailable", "{what}");
        // round 3: --full never says there is no record when the engine facts are unavailable
        let full = e.find(&["--full"]).unwrap().out;
        assert!(
            full.contains("engine status: unavailable (") && full.contains("; run cardgen export)"),
            "{what}: {full}"
        );
        assert!(!full.contains("NO RECORD") && !full.contains("engine status: none"), "{what}: {full}");
        let err = out.stderr();
        assert_eq!(err.lines().count(), 1, "{what}: one stderr line, got {err:?}");
        assert!(
            err.starts_with("(engine status unavailable: ")
                && err.contains(why)
                && err.trim_end().ends_with("; run cardgen export)"),
            "{what}: {err}"
        );
        match e.find(&["--status", "full"]) {
            Err(FindError::Failed(m)) => {
                assert!(m.contains("engine status unavailable") && m.contains(why), "{what}: {m}")
            }
            other => panic!("{what}: --status must fail, got {other:?}"),
        }
        assert!(matches!(e.find(&["--status", "none"]), Err(FindError::Failed(_))), "{what}");
        assert!(e.find(&[]).unwrap().stderr().is_empty(), "{what}: a table search does not need the engine");
        let facts = e.facts();
        assert!(facts.engine.status.is_empty() && facts.context(None).model_status.is_empty(), "{what}");
        assert!(facts.context(None).engine_note.unwrap().contains(why), "{what}");
        // nothing was written by the read commands
        assert_eq!(e.0.join("data/cards.json").is_file(), text.is_some(), "{what}");
    }
    // a valid export: the status is there and nothing is said
    e.write("data/cards.json", r#"{"version":1,"cards":[{"name":"Test Engine Relic"}]}"#);
    let out = e.find(&["--json"]).unwrap();
    let rows: Vec<Value> = serde_json::from_str(&out.out).unwrap();
    assert_eq!(rows[0]["status"], "Full", "{}", rows[0]);
    assert_eq!(rows[0]["engine"], "record");
    assert_eq!(out.stderr(), "");
    assert_eq!(e.find(&["--status", "full", "--count"]).unwrap().out, "1\n");
}

#[test]
fn missing_and_malformed_audit_sources_fail_cleanly() {
    for file in ["card_overrides.toml", "audit.toml"] {
        for value in [None, Some("[broken")] {
            let e = Env::new(vec![card("Test Engine Relic")]);
            e.engine();
            // round 2: the export must be valid and current for the audit sources to be read at all
            e.write("data/cards.json", r#"{"version":1,"cards":[{"name":"Test Engine Relic"}]}"#);
            if let Some(text) = value {
                e.write(&format!("data/{file}"), text);
            } else {
                std::fs::remove_file(e.0.join("data").join(file)).unwrap();
            }
            assert!(matches!(e.find(&["--json"]), Err(FindError::Failed(_))));
            assert!(e.find(&[]).is_ok(), "plain table must not load the engine");
        }
    }
}

#[test]
fn status_grammar_and_filter_cross_product_keep_counts_sorting_paging_and_notes() {
    let mut c = card("Red Candidate");
    c["color_identity"] = json!(["R"]);
    let mut gc = card("Test Draw");
    gc["game_changer"] = json!(true);
    let e = Env::new(vec![card("Sol Ring"), card("Second Relic"), gc, c]);
    e.decks(json!([{"key":"alpha", "commander":"Sol Ring", "ident":"", "cards":["Sol Ring"]},
        {"key":"beta", "commander":"", "ident":"", "cards":["Second Relic", "Test Draw"]}]));
    e.attach_roles();
    assert_eq!(e.find(&[]).unwrap().out, e.find(&["--status", "none"]).unwrap().out);
    assert_eq!(parse_status_list("FuLL, full, NONE,Full-Auto").unwrap(), ["full", "none", "full-auto"]);
    for bad in ["", ",", "full,", ",full", "none,,full", "nan", "full;none", "Full auto", "全", "\0"] {
        assert!(parse_status_list(bad).is_err(), "{bad:?}");
    }
    for status in ["full", "full-auto", "approximate", "partial", "unmodeled"] {
        assert!(e.rows(&["--status", status]).is_empty());
    }
    let filters = [
        vec![],
        vec!["--deck", "alpha"],
        vec!["--ident", "C"],
        vec!["--gc"],
        vec!["--type", "Artifact"],
        vec!["--text", "Draw"],
        vec!["--cmc", "<=1"],
        vec!["--role", "draw"],
        vec!["--name-contains", "Relic"],
        vec!["--name", "Test Draw"],
    ];
    for filter in filters {
        let mut args = vec!["--limit", "all", "--sort", "name", "--status", "NoNe,NONE"];
        args.extend(filter);
        let base = e.rows(&args);
        for flag in ["--in-deck", "--not-in-deck"] {
            let mut filtered = args.clone();
            filtered.extend([flag, "BETA"]);
            let expected: Vec<_> = base
                .iter()
                .filter(|r| r["decks"].as_array().unwrap().contains(&json!("beta")) == (flag == "--in-deck"))
                .cloned()
                .collect();
            assert_eq!(e.rows(&filtered), expected, "{filtered:?}");
        }
    }
    let all = e.rows(&["--in-deck", "beta", "--sort", "name"]);
    assert_eq!(e.rows(&["--in-deck", "beta", "--sort", "name", "--limit", "1", "--offset", "1"]), all[1..]);
    assert_eq!(e.find(&["--in-deck", "beta", "--count", "--json", "--offset", "999"]).unwrap().out, "2\n");
    assert!(e.rows(&["--in-deck", "beta", "--not-in-deck", "beta"]).is_empty());
    for flag in ["--status", "--in-deck", "--not-in-deck"] {
        assert!(matches!(e.find(&[flag]), Err(FindError::Usage(_))));
        assert!(matches!(e.find(&[flag, "unknown"]), Err(FindError::Usage(_))));
        assert!(matches!(e.find(&[flag, "none", flag, "none"]), Err(FindError::Usage(_))));
    }
    let note = e.find(&["--name", "Sol Ring", "--not-in-deck", "alpha", "--json"]).unwrap();
    assert!(note.notes_on_stderr && note.notes.iter().any(|n| n.contains("--not-in-deck")));
    let page = binary(&e.0, &["find", "--json", "--limit", "1"]);
    assert!(page.status.success());
    serde_json::from_slice::<Value>(&page.stdout).unwrap();
    assert!(String::from_utf8_lossy(&page.stderr).contains("showing 1-1"));
}

#[test]
fn odd_unicode_values_and_concurrent_readers_do_not_panic() {
    let e = Env::new(vec![card("Sol Ring")]);
    for flag in ["--status", "--in-deck", "--not-in-deck"] {
        for value in ["\0", "\n", "\r\n", "\u{feff}", "\u{202e}", "é", "💥", "--json", "../x", "a,b"] {
            assert!(e.find(&[flag, value]).is_err(), "{flag} {value:?}");
        }
        assert!(e.find(&[flag, &"x".repeat(65_536)]).is_err());
        let mut seed = 0x5eed_u64;
        for _ in 0..64 {
            let chars = ['\0', '\u{1}', ',', '-', 'é', '全', '💥', '\n', '\u{feff}', '\u{202e}'];
            let value: String = (0..32)
                .map(|_| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    chars[(seed >> 32) as usize % chars.len()]
                })
                .collect();
            assert!(e.find(&[flag, &value]).is_err());
        }
    }
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..4).map(|_| s.spawn(|| e.find(&["--json"]).unwrap().out)).collect();
        let outputs: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(outputs.windows(2).all(|w| w[0] == w[1]));
    });
}

#[cfg(unix)]
#[test]
fn defect_invalid_utf8_filter_value_panics_in_cli() {
    use std::os::unix::ffi::OsStrExt;
    let e = Env::new(vec![card("Sol Ring")]);
    let mut panics = vec![];
    for flag in ["--status", "--in-deck", "--not-in-deck"] {
        let out = Command::new("systemd-run")
            .args(CAP)
            .arg(env!("CARGO_BIN_EXE_cardgen"))
            .arg("--root")
            .arg(&e.0)
            .args(["find", flag])
            .arg(std::ffi::OsStr::from_bytes(b"\xff"))
            .output()
            .unwrap();
        if String::from_utf8_lossy(&out.stderr).contains("panicked") || out.status.code() != Some(1) {
            panics.push((flag, out.status.code(), String::from_utf8_lossy(&out.stderr).into_owned()));
        }
    }
    assert!(panics.is_empty(), "invalid filter bytes must return usage errors, not panic: {panics:?}");
}

#[test]
fn stale_or_edited_roles_fail_explicitly_and_data_dir_is_respected() {
    let e = Env::new(vec![card("Test Engine Relic")]);
    e.attach_roles();
    let dir = e.0.join("data/knowledge");
    assert!(digest::open(&e.project(), Some(&dir)).is_ok());
    let original = std::fs::read_to_string(dir.join("roles.json")).unwrap();
    let edited = original.replace("Test Engine Relic", "Forged Relic");
    // Any hand edit in the assignment bytes invalidates the table digest.
    let edited = if edited == original { original.replacen("\"roles\":", "\"roles\": \n", 1) } else { edited };
    assert_ne!(edited, original);
    e.write("data/knowledge/roles.json", edited);
    assert!(digest::open(&e.project(), Some(&dir)).is_err());
    let out = e.find(&["--json"]).unwrap();
    assert!(out.role_note.is_some());
    e.write("data/knowledge/roles.json", original);
    let mut changed = card("Test Engine Relic");
    changed["oracle_text"] = json!("Draw two cards.");
    e.write("data/knowledge/oracle.json", json!({"meta": {"schema":1,"count":1},"cards":[changed]}).to_string());
    assert!(digest::open(&e.project(), Some(&dir)).is_err());
}

#[test]
fn custom_data_dirs_unknown_identity_and_concurrent_view_writes() {
    let e = Env::new(vec![card("Sol Ring"), card("Test Draw")]);
    e.decks(json!([{"key":"alpha", "commander":"Sol Ring", "ident":"Z", "cards":[]}]));
    assert!(e.find(&["--deck", "alpha"]).is_err(), "invalid deck identity must not widen the search");
    e.decks(json!([]));
    e.attach_roles();
    for dir in ["data/knowledge".to_string(), e.0.join("data/knowledge").display().to_string()] {
        let out = binary(&e.0, &["find", "--data-dir", &dir, "--json", "--limit", "all"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let rows: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(rows.len(), 2);
    }
    let path = e.0.join("digest.txt");
    let args = vec![
        "--ident".to_string(),
        "C".to_string(),
        "--data-dir".to_string(),
        e.0.join("data/knowledge").display().to_string(),
        "--out".to_string(),
        path.display().to_string(),
    ];
    let texts = std::thread::scope(|s| {
        let handles: Vec<_> =
            (0..4).map(|_| s.spawn(|| digest::command(&e.project(), &args).unwrap().digest.text)).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
    });
    assert!(texts.windows(2).all(|w| w[0] == w[1]));
    assert_eq!(std::fs::read_to_string(path).unwrap(), texts[0]);
    assert!(!texts[0].lines().any(|l| l.starts_with("# m:") || l.starts_with("# d:")));
    assert!(texts[0].lines().filter(|l| !l.starts_with('#')).all(|l| !l.contains("m:")));
    e.decks(json!([{"key":"alpha", "commander":"Sol Ring", "ident":"", "cards":[]}]));
    e.write("audit/alpha/FINDINGS.md", "Sol Ring\n");
    let with_findings = digest::command(&e.project(), &args).unwrap().digest.text;
    assert_eq!(with_findings.lines().filter(|l| l.starts_with("# m:")).count(), 1);
    assert_eq!(with_findings.lines().filter(|l| l.starts_with("# d:")).count(), 1);
    assert!(with_findings.contains("k:1"));
}

// Real catalog: every check is data-driven. No production card name or deck key is baked into a test.

fn real_rows(args: &[&str]) -> Vec<Value> {
    let mut a: Vec<_> = args.iter().map(|s| s.to_string()).collect();
    a.extend(["--json".into(), "--data-dir".into(), repo().join("data/knowledge").display().to_string()]);
    serde_json::from_str(&knowledge::find(&Project::new(repo()), &a).unwrap()).unwrap()
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_full_catalog_json_deck_usage_and_safe_candidates_independent_python() {
    let artifacts = Env::new(vec![]);
    let rows = real_rows(&["--limit", "all", "--legal", "any", "--meld"]);
    artifacts.write("rows.json", serde_json::to_vec(&rows).unwrap());
    let decks = read_json(repo().join("data/decks.json"));
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let safe = real_rows(&["--deck", key, "--status", "full,full-auto", "--limit", "all"]);
        artifacts.write(&format!("safe-{key}.json"), serde_json::to_vec(&safe).unwrap());
        let own = real_rows(&["--in-deck", key, "--legal", "any", "--meld", "--limit", "all"]);
        artifacts.write(&format!("own-{key}.json"), serde_json::to_vec(&own).unwrap());
    }
    python("rows", &repo(), &artifacts.0);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_every_engine_record_agrees_with_audit_status_and_note_with_documented_collision_policy() {
    let p = Project::new(repo());
    let index = Index::open(&p, Some(&p.data("knowledge"))).unwrap();
    let inp = Inputs::load(&p).unwrap();
    let built = inp.build();
    let audit = Audit::new(&inp, &built.cards).unwrap();
    let mut mine: BTreeSet<_> = inp.my_cards.iter().flat_map(|(_, c)| c.iter().map(String::as_str)).collect();
    for d in &inp.my {
        mine.insert(&d.commander);
        mine.extend(d.commander2.as_deref());
    }
    let exported = read_json(p.data("cards.json"));
    let all_names: Vec<_> = exported["cards"].as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap()).collect();
    let artifacts = Env::new(vec![]);
    artifacts.write("names.txt", all_names.join("\n"));
    let cli = binary(&repo(), &["audit", "--cards", artifacts.0.join("names.txt").to_str().unwrap()]);
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let stdout = String::from_utf8(cli.stdout).unwrap();
    let mut cli_count = 0;
    for l in stdout.lines() {
        let parts: Vec<_> = l.splitn(3, " | ").collect();
        assert_eq!(parts.len(), 3);
        let expected = audit.card_status(parts[1], true);
        assert_eq!((parts[0], parts[2]), (expected.0.as_str(), expected.1.as_str()));
        cli_count += 1;
    }
    assert_eq!(cli_count, all_names.len());
    let mut expected: BTreeMap<u32, Vec<(&str, (String, String))>> = BTreeMap::new();
    let mut mine_sensitive = 0;
    for name in &all_names {
        let sn = audit.card_status(name, mine.contains(name));
        mine_sensitive += usize::from(audit.card_status(name, true) != audit.card_status(name, false));
        // Name resolution checked independently against Python, not inferred from the implementation under test.
        for &i in index.lookup(name) {
            if index.cards[i as usize].legal == knowledge::Legal::Legal
                || !index.lookup(name).iter().any(|&j| index.cards[j as usize].legal == knowledge::Legal::Legal)
            {
                expected.entry(i).or_default().push((name, sn.clone()));
            }
        }
    }
    let facts = Facts::load(&p, &index, true, true).unwrap();
    let rows = real_rows(&["--limit", "all", "--legal", "any", "--meld"]);
    let by_id: BTreeMap<_, _> = rows.iter().map(|r| (r["oracle_id"].as_str().unwrap(), r)).collect();
    let mut collisions = 0;
    let mut differing = 0;
    let mut different_status = 0;
    for (i, c) in index.cards.iter().enumerate() {
        let row = by_id[c.raw.oracle_id.as_str()];
        if let Some(candidates) = expected.get_mut(&(i as u32)) {
            candidates.sort_by_key(|(name, (s, _))| (rank(s), knowledge::fold(name) != c.key));
            let (name, (s, n)) = &candidates[0];
            collisions += usize::from(candidates.len() > 1);
            differing += usize::from(candidates.iter().any(|(_, sn)| sn != &candidates[0].1));
            different_status += usize::from(candidates.iter().any(|(_, sn)| sn.0 != candidates[0].1.0));
            assert_eq!((&row["status"], &row["status_note"]), (&json!(s), &json!(n)), "{name}");
            assert_eq!(facts.status_of(i as u32), Some((s.as_str(), n.as_str())));
        } else {
            assert!(row.get("status").is_none() && row.get("status_note").is_none());
            assert!(facts.status_of(i as u32).is_none());
        }
    }
    for status in ["none", "full", "full-auto", "approximate", "partial", "unmodeled"] {
        let hits = real_rows(&["--status", status, "--legal", "any", "--meld", "--limit", "all"]);
        let want: BTreeSet<_> = rows
            .iter()
            .filter(|r| {
                r.get("status").and_then(Value::as_str).map_or(status == "none", |s| s.eq_ignore_ascii_case(status))
            })
            .map(|r| r["oracle_id"].as_str().unwrap())
            .collect();
        let got: BTreeSet<_> = hits.iter().map(|r| r["oracle_id"].as_str().unwrap()).collect();
        assert_eq!(got, want, "{status}");
    }
    let full_args = [
        "--status",
        "full,full-auto,approximate,partial,unmodeled",
        "--limit",
        "all",
        "--legal",
        "any",
        "--meld",
        "--full",
    ]
    .map(String::from);
    let full_output = knowledge::find_with_notes(&p, &full_args).unwrap().out;
    let actual: Vec<_> = full_output.lines().filter(|l| l.starts_with("engine status:")).collect();
    let expected_full: Vec<_> = rows
        .iter()
        .filter_map(|r| {
            let s = r.get("status")?.as_str().unwrap();
            let note = r["status_note"].as_str().unwrap();
            // round 3: Full-auto is worded as what it is
            let s = if s == "Full-auto" { "Full-auto (compiled, not hand-verified)" } else { s };
            Some(format!("engine status: {s}{}", if note.is_empty() { String::new() } else { format!(" ({note})") }))
        })
        .collect();
    assert_eq!(actual, expected_full, "--full must print every recorded card's status and note exactly");
    let sample = rows.iter().find(|r| r.get("status").is_some()).unwrap();
    let name = sample["name"].as_str().unwrap();
    let full = binary(&repo(), &["find", "--name", name, "--full", "--legal", "any"]);
    assert!(full.status.success());
    assert!(
        String::from_utf8_lossy(&full.stdout)
            .contains(&format!("engine status: {}", sample["status"].as_str().unwrap()))
    );
    eprintln!(
        "{} exported records; {} resolved; {collisions} collisions, {differing} distinct verdicts \
        ({different_status} different statuses); \
        {mine_sensitive} mine-sensitive records; unresolved {:?}",
        all_names.len(),
        expected.len(),
        facts.engine.unresolved
    );
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_all_decks_and_32_identities_digest_grammar_budgets_determinism_and_map_counts_python() {
    let p = Project::new(repo());
    let artifacts = Env::new(vec![]);
    let t = Instant::now();
    let (index, table) = digest::open(&p, Some(&p.data("knowledge"))).unwrap();
    let facts = Facts::load(&p, &index, true, true).unwrap();
    let prev = Prev::load(&p.data("knowledge"));
    let decks = map::load_mine(&p.data("decks.json")).unwrap();
    for (name, deck, ident) in decks
        .iter()
        .map(|d| (d.deck.key.clone(), Some(&d.deck), None))
        .chain((0..32u8).map(|bits| (format!("ident-{bits}"), None, Some(bits))))
    {
        let spec =
            Spec { deck, ident, role: None, view: None, limit: None, offset: 0, budget_tokens: None, prev: &prev };
        let ctx = facts.context(deck.map(|d| d.key.as_str()));
        let first = digest::build_with(&index, &table, &spec, &ctx).unwrap();
        assert_eq!(first.text, digest::build_with(&index, &table, &spec, &ctx).unwrap().text, "{name}");
        artifacts.write(&format!("digest-{name}.txt"), first.text);
    }
    let spell = p.data("spellbook");
    let first = map::build_with_facts(&index, &table, Some(&spell), &decks, Some(&facts)).unwrap();
    assert_eq!(first, map::build_with_facts(&index, &table, Some(&spell), &decks, Some(&facts)).unwrap());
    artifacts.write("map.md", first);
    artifacts
        .write("rows.json", serde_json::to_vec(&real_rows(&["--legal", "any", "--meld", "--limit", "all"])).unwrap());
    python("views", &repo(), &artifacts.0);
    eprintln!("37 views and map, each built twice: {:.3}s", t.elapsed().as_secs_f64());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_cold_cli_time_and_rss() {
    let decks = read_json(repo().join("data/decks.json"));
    let d = &decks["mine"][0];
    let name = d["commander"].as_str().unwrap();
    let e = Env::new(vec![]);
    let path = e.0.join("digest.txt");
    for (label, args, bound) in [
        ("find-json", vec!["find", "--name", name, "--json"], 1.0),
        ("digest", vec!["digest", "--deck", d["key"].as_str().unwrap(), "--out", path.to_str().unwrap()], 1.5),
    ] {
        let stats = e.0.join(format!("{label}.time"));
        let out = Command::new("systemd-run")
            .args(CAP)
            .args(["/usr/bin/time", "-v", "-o"])
            .arg(&stats)
            .arg(env!("CARGO_BIN_EXE_cardgen"))
            .arg("--root")
            .arg(repo())
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let text = std::fs::read_to_string(stats).unwrap();
        eprintln!("{label}:\n{text}");
        let wall = text.lines().find(|l| l.contains("Elapsed (wall clock)")).unwrap().rsplit(": ").next().unwrap();
        let components: Vec<f64> = wall.split(':').map(|n| n.parse().unwrap()).collect();
        let secs = components.into_iter().fold(0.0, |n, c| 60.0 * n + c);
        assert!(secs < bound, "{label} cold {secs}s >= {bound}s");
        let rss: usize = text
            .lines()
            .find(|l| l.contains("Maximum resident set size"))
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        assert!(rss < 2_000_000, "{label} RSS {rss} KiB");
    }
}
