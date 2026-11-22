//! Offline adversarial coverage for the ownership checker: inputs built to break it (names, copies, TOML edge cases,
//! the CLI, random and mutated files). No production code or real data is changed.

use cardgen::data::Project;
use cardgen::ownership::{self, Level, Options, Report};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const HEADER: &str = "version = 1\nupdated = \"2026-11-22\"\n";
const GOOD: &str = "version = 1\nupdated = \"2026-11-22\"\n[deck.alpha]\nconfirmed = \"2026-11-22\"\nhave = [\"Sol Ring\"]\n[spare]\ncards = []\n";
const PREPARE: &str = "Emeritus of Conflict // Lightning Bolt";

struct Env(PathBuf);

impl Env {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ownership-adv-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("data/knowledge")).unwrap();
        std::fs::create_dir_all(root.join("data/none")).unwrap();
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ownership_oracle.json");
        std::fs::copy(fixture, root.join("data/knowledge/oracle.json")).unwrap();
        let env = Self(root);
        env.decks(json!({"mine": [{"key": "alpha", "cards": ["Sol Ring"]}], "pool": []}));
        env
    }

    fn decks(&self, decks: Value) {
        std::fs::write(self.0.join("data/decks.json"), serde_json::to_vec(&decks).unwrap()).unwrap();
    }

    fn cards(&self, cards: &[&str]) {
        self.decks(json!({"mine": [{"key": "alpha", "cards": cards}], "pool": []}));
    }

    fn catalog_card(&self, name: &str, id: &str, faces: &[&str]) {
        let path = self.0.join("data/knowledge/oracle.json");
        let mut catalog: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let mut card = catalog["cards"][0].clone();
        card["name"] = json!(name);
        card["oracle_id"] = json!(id);
        if !faces.is_empty() {
            card["card_faces"] = json!(faces.iter().map(|name| json!({"name": name})).collect::<Vec<_>>());
            card["layout"] = json!("split");
        }
        catalog["cards"].as_array_mut().unwrap().push(card);
        catalog["meta"]["count"] = json!(catalog["cards"].as_array().unwrap().len());
        std::fs::write(path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    }

    fn opts(&self, catalog: bool) -> Options {
        Options {
            file: Some(self.0.join("own.toml")),
            data_dir: Some(self.0.join(if catalog { "data/knowledge" } else { "data/none" })),
            strict: false,
        }
    }

    fn check(&self, text: &str, catalog: bool) -> Report {
        std::fs::write(self.0.join("own.toml"), text).unwrap();
        ownership::check(&Project::new(&self.0), &self.opts(catalog)).unwrap()
    }

    fn cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cardgen"))
            .current_dir(&self.0)
            .arg("--root")
            .arg(&self.0)
            .args(["ownership", "--check", "--file", "own.toml", "--data-dir", "data/knowledge"])
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn problem(r: &Report, level: Level, deck: Option<&str>, card: Option<&str>, message: &str) -> bool {
    r.problems.iter().any(|p| {
        p.level == level
            && deck.is_none_or(|d| p.deck.as_deref() == Some(d))
            && card.is_none_or(|c| p.card.as_deref() == Some(c))
            && p.message.contains(message)
    })
}

fn document(have: &str, proxy: &str, missing: &str, spare: &str) -> String {
    format!(
        "{HEADER}[deck.alpha]\nconfirmed = \"2026-11-22\"\nhave = {have}\nproxy = {proxy}\nmissing = {missing}\n[spare]\ncards = {spare}\n"
    )
}

#[test]
fn names_with_unicode_punctuation_and_outer_whitespace_match_in_both_modes() {
    let env = Env::new();
    env.catalog_card("Gix, Yawgmoth Praetor", "adv-gix", &[]);
    env.cards(&["Palantír of Orthanc", "Barad-dûr", "Æther Vial", "Urza's Saga", "Gix, Yawgmoth Praetor"]);
    let text = document(
        r#"["  PALANTI\u0301R OF ORTHANC  ", "barad-du\u0302r", "aether vial", "Urza’s Saga", "Gix, Yawgmoth Praetor"]"#,
        "[]",
        "[]",
        "[]",
    );
    for catalog in [false, true] {
        let r = env.check(&text, catalog);
        assert!(r.ok(), "{}", r.text());
        assert_eq!(r.decks[0].have, 5);
        assert_eq!(r.warnings(), usize::from(!catalog));
    }
}

#[test]
fn internal_whitespace_is_not_an_alias_under_find_name_folding() {
    let env = Env::new();
    for catalog in [false, true] {
        let r = env.check(&GOOD.replace("Sol Ring", "Sol  Ring"), catalog);
        assert!(problem(&r, Level::Error, Some("alpha"), Some("Sol Ring"), "not in the ownership file"));
        assert!(problem(&r, Level::Error, Some("alpha"), Some("Sol  Ring"), "not in the deck list"));
    }
}

#[test]
fn aliases_are_duplicates_within_every_status_and_spare() {
    let env = Env::new();
    env.cards(&["Leech Collector // Bloodletting"]);
    let aliases = r#"["Leech Collector", "  BLOODLETTING  ", "Leech Collector // Bloodletting"]"#;
    for catalog in [false, true] {
        for status in ["have", "proxy", "missing"] {
            let text = format!("{HEADER}[deck.alpha]\n{status} = {aliases}\n");
            let r = env.check(&text, catalog);
            assert!(
                problem(&r, Level::Error, Some("alpha"), None, &format!("listed 3 times in {status}")),
                "{}",
                r.text()
            );
        }
        let text = document(r#"["Leech Collector"]"#, "[]", "[]", aliases);
        let r = env.check(&text, catalog);
        assert_eq!(r.problems.iter().filter(|p| p.message.contains("listed twice in spare")).count(), 2);
    }
}

#[test]
fn aliases_cannot_hide_conflicts_between_any_two_statuses() {
    let env = Env::new();
    env.cards(&["Palantír of Orthanc"]);
    for catalog in [false, true] {
        for (first, second) in [("have", "proxy"), ("have", "missing"), ("proxy", "missing")] {
            let text = format!(
                "{HEADER}[deck.alpha]\n{first} = [\"Palantír of Orthanc\"]\n{second} = [\"palanti\\u0301r of orthanc\"]\n"
            );
            let r = env.check(&text, catalog);
            assert!(problem(&r, Level::Error, Some("alpha"), None, "two statuses"), "{}", r.text());
        }
    }
}

#[test]
fn full_names_beat_prepare_faces_with_a_catalog() {
    let env = Env::new();
    env.catalog_card(PREPARE, "adv-prepare", &["Emeritus of Conflict", "Lightning Bolt"]);
    env.cards(&["Lightning Bolt", PREPARE]);
    let r = env.check(&document(r#"["Lightning Bolt", "Emeritus of Conflict"]"#, "[]", "[]", "[]"), true);
    assert!(r.ok(), "{}", r.text());
    assert_eq!(r.decks[0].have, 2);
    // Cutting the distinct instant must be detected even though a prepare face has that name.
    env.cards(&[PREPARE]);
    let r = env.check(&document(r#"["Lightning Bolt"]"#, "[]", "[]", "[]"), true);
    assert!(problem(&r, Level::Error, Some("alpha"), Some(PREPARE), "not in the ownership file"), "{}", r.text());
    assert!(problem(&r, Level::Error, Some("alpha"), Some("Lightning Bolt"), "not in the deck list"), "{}", r.text());
}

#[test]
fn fallback_preserves_distinct_full_names_that_collide_with_faces() {
    let env = Env::new();
    env.catalog_card(PREPARE, "adv-prepare", &["Emeritus of Conflict", "Lightning Bolt"]);
    env.cards(&["Lightning Bolt", PREPARE]);
    let text = document(r#"["Lightning Bolt", "Emeritus of Conflict // Lightning Bolt"]"#, "[]", "[]", "[]");
    assert!(env.check(&text, true).ok());
    let r = env.check(&text, false);
    assert!(r.ok(), "Distinct full deck names must remain distinct without a catalog:\n{}", r.text());
}

#[test]
fn fallback_cannot_hide_an_omitted_standalone_card() {
    let env = Env::new();
    env.catalog_card(PREPARE, "adv-prepare", &["Emeritus of Conflict", "Lightning Bolt"]);
    env.cards(&["Lightning Bolt", PREPARE]);
    let text = document(r#"["Emeritus of Conflict // Lightning Bolt"]"#, "[]", "[]", "[]");
    assert!(!env.check(&text, true).ok());
    let r = env.check(&text, false);
    assert!(
        problem(&r, Level::Error, Some("alpha"), Some("Lightning Bolt"), "not in the ownership file"),
        "Missing instant was accepted:\n{}",
        r.text()
    );
}

#[test]
fn fallback_shared_faces_resolve_in_the_current_deck() {
    let env = Env::new();
    env.decks(json!({"mine": [
        {"key": "alpha", "cards": ["First // Shared"]},
        {"key": "beta", "cards": ["Second // Shared"]}
    ]}));
    let text = format!("{HEADER}[deck.alpha]\nhave = [\"First // Shared\"]\n[deck.beta]\nhave = [\"Shared\"]\n");
    let r = env.check(&text, false);
    assert!(r.ok(), "A face name must match the only card with that face in its deck:\n{}", r.text());
}

#[test]
fn copy_rules_count_have_decks_only_and_spares_do_not_change_k() {
    let env = Env::new();
    env.decks(json!({"mine": [
        {"key": "alpha", "cards": ["Sol Ring"]},
        {"key": "beta", "cards": ["Sol Ring"]},
        {"key": "gamma", "cards": ["Sol Ring"]}
    ]}));
    let statuses = ["have", "proxy", "missing"];
    for a in statuses {
        for b in statuses {
            for c in statuses {
                let assignment = [a, b, c];
                let k = assignment.iter().filter(|&&s| s == "have").count();
                for copies in [None, Some(1), Some(2), Some(3)] {
                    let mut text = HEADER.to_owned();
                    for (key, status) in ["alpha", "beta", "gamma"].into_iter().zip(assignment) {
                        let card = match copies {
                            None => "\"Sol Ring\"".to_owned(),
                            Some(n) => format!("{{ card = \"Sol Ring\", copies = {n} }}"),
                        };
                        text += &format!("[deck.{key}]\nconfirmed = \"2026-11-22\"\n{status} = [{card}]\n");
                    }
                    text += "[spare]\ncards = [{ card = \"Sol Ring\", copies = 5 }]\n";
                    let r = env.check(&text, true);
                    let errors = assignment
                        .iter()
                        .filter(|&&status| copies.is_some_and(|n| if status == "have" { n != k } else { n > 1 }))
                        .count();
                    assert_eq!(r.errors(), errors, "{assignment:?}, copies={copies:?}:\n{}", r.text());
                    assert_eq!(r.warnings(), usize::from(k >= 2 && copies != Some(k)), "{}", r.text());
                    assert_eq!((r.spare, r.infos()), (5, 1));
                }
            }
        }
    }
}

#[test]
fn every_have_deck_must_confirm_k_not_just_one() {
    let env = Env::new();
    env.decks(json!({"mine": [
        {"key": "alpha", "cards": ["Sol Ring"]},
        {"key": "beta", "cards": ["Sol Ring"]},
        {"key": "gamma", "cards": ["Sol Ring"]}
    ]}));
    for mask in 0..8 {
        let mut text = HEADER.to_owned();
        for (i, key) in ["alpha", "beta", "gamma"].into_iter().enumerate() {
            let entry = if mask & (1 << i) != 0 { "{ card = \"Sol Ring\", copies = 3 }" } else { "\"Sol Ring\"" };
            text += &format!("[deck.{key}]\nconfirmed = \"2026-11-22\"\nhave = [{entry}]\n");
        }
        let r = env.check(&text, true);
        assert!(r.ok(), "{}", r.text());
        assert_eq!(r.warnings(), usize::from(mask != 7), "mask {mask}: {}", r.text());
    }
}

#[test]
fn invalid_copy_types_are_rejected_in_all_four_places() {
    let env = Env::new();
    let bad = [
        "0",
        "-1",
        "-9223372036854775808",
        "1.0",
        "1.5",
        "nan",
        "inf",
        "\"2\"",
        "true",
        "false",
        "[]",
        "{}",
        "9223372036854775808",
        "18446744073709551615",
    ];
    for catalog in [false, true] {
        for place in 0..4 {
            for copies in bad {
                let entry = format!("[{{ card = \"Sol Ring\", copies = {copies} }}]");
                let mut lists = ["[]", "[]", "[]", "[]"];
                lists[place] = &entry;
                if place == 3 {
                    lists[0] = "[\"Sol Ring\"]";
                }
                let r = env.check(&document(lists[0], lists[1], lists[2], lists[3]), catalog);
                assert!(!r.ok(), "place={place} copies={copies} catalog={catalog}: {}", r.text());
                assert!(r.text().contains("copies") || r.text().contains("malformed TOML"), "{}", r.text());
            }
        }
    }
}

#[test]
fn spare_total_near_u64_max_is_exact() {
    let env = Env::new();
    let text = document(
        r#"["Sol Ring"]"#,
        "[]",
        "[]",
        r#"[{ card = "Extra Card", copies = 9223372036854775807 }, { card = "Lightning Bolt", copies = 9223372036854775807 }, "Æther Vial"]"#,
    );
    let r = env.check(&text, true);
    assert!(r.ok(), "{}", r.text());
    assert_eq!(r.spare, u64::MAX);
    assert_eq!(serde_json::from_str::<Value>(&r.json()).unwrap()["spare"].as_u64(), Some(u64::MAX));
}

#[test]
fn overflowing_spare_total_reports_an_error_instead_of_panicking() {
    let env = Env::new();
    let text = document(
        r#"["Sol Ring"]"#,
        "[]",
        "[]",
        r#"[{ card = "Extra Card", copies = 9223372036854775807 }, { card = "Lightning Bolt", copies = 9223372036854775807 }, { card = "Æther Vial", copies = 2 }]"#,
    );
    let r = env.check(&text, true);
    assert!(!r.ok(), "An unrepresentable spare total must produce an ERROR:\n{}", r.text());
}

#[test]
fn cli_overflow_returns_error_exit_and_valid_json() {
    let env = Env::new();
    let text = document(
        r#"["Sol Ring"]"#,
        "[]",
        "[]",
        r#"[{ card = "Extra Card", copies = 9223372036854775807 }, { card = "Lightning Bolt", copies = 9223372036854775807 }, { card = "Æther Vial", copies = 2 }]"#,
    );
    std::fs::write(env.0.join("own.toml"), text).unwrap();
    let out = env.cli(&["--json"]);
    assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
    let parsed: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(parsed["ok"], false);
    assert!(parsed["errors"].as_u64().unwrap() > 0);
}

#[test]
fn invalid_dates_types_versions_and_empty_documents_are_errors() {
    let env = Env::new();
    for field in ["updated", "confirmed"] {
        for value in [
            "2026-11-22",
            "2026-11-22T00:00:00Z",
            "20261122",
            "true",
            "[]",
            "{}",
            "\"2026-02-30\"",
            "\"2100-02-29\"",
            "\"2026-00-01\"",
            "\"2026-01-00\"",
            "\"2026-1-01\"",
            "\"２０２６-１１-２２\"",
        ] {
            let text = GOOD.replace(&format!("{field} = \"2026-11-22\""), &format!("{field} = {value}"));
            let r = env.check(&text, true);
            assert!(problem(&r, Level::Error, None, None, field), "{field}={value}: {}", r.text());
        }
    }
    for text in [
        String::new(),
        GOOD.replace("version = 1\n", ""),
        GOOD.replace("version = 1", "version = 2"),
        GOOD.replace("version = 1", "version = \"1\""),
        GOOD.replace("updated = \"2026-11-22\"\n", ""),
    ] {
        assert!(!env.check(&text, true).ok(), "{text}");
    }
    assert!(env.check(&GOOD.replace("2026-11-22", "2024-02-29"), true).ok());
    assert!(env.check(&GOOD.replace("2026-11-22", "2000-02-29"), true).ok());
}

#[test]
fn valid_iso_dates_before_2000_are_not_bad_date_formats() {
    let env = Env::new();
    let r = env.check(&GOOD.replace("2026-11-22", "1999-12-31"), true);
    assert!(r.ok(), "1999-12-31 is a real YYYY-MM-DD date:\n{}", r.text());
}

#[test]
fn duplicate_keys_tables_unknown_fields_and_wrong_types_are_errors() {
    let env = Env::new();
    let cases = [
        format!("{GOOD}\n[deck.alpha]\nhave = []\n"),
        GOOD.replace("version = 1", "version = 1\nversion = 1"),
        GOOD.replace("have = [\"Sol Ring\"]", "have = [\"Sol Ring\"]\nhave = []"),
        GOOD.replace("[spare]", "[deck.alpha.extra]\nx = 1\n[spare]"),
        GOOD.replace("version = 1", "version = 1\nnote = \"hi\""),
        GOOD.replace("cards = []", "cards = []\nnote = \"hi\""),
        GOOD.replace("\"Sol Ring\"", "{ card = \"Sol Ring\", typo = 1 }"),
        GOOD.replace("have = [\"Sol Ring\"]", "have = \"Sol Ring\""),
        GOOD.replace("have = [\"Sol Ring\"]", "have = {}"),
        GOOD.replace("cards = []", "cards = \"Sol Ring\""),
        GOOD.replace("\"Sol Ring\"", "{ card = true }"),
        GOOD.replace("\"Sol Ring\"", "{ copies = 1 }"),
        GOOD.replace("\"Sol Ring\"", "17"),
        format!("{HEADER}deck = []\n"),
        format!("{HEADER}deck.alpha = false\n"),
        format!("{HEADER}spare = true\n[deck.alpha]\nhave = [\"Sol Ring\"]\n"),
    ];
    for (i, text) in cases.iter().enumerate() {
        let r = env.check(text, true);
        assert!(!r.ok(), "case {i} accepted:\n{text}\n{}", r.text());
    }
}

#[test]
fn legal_dotted_keys_inline_tables_source_comments_bom_and_crlf_parse() {
    let env = Env::new();
    let dotted =
        format!("{HEADER}deck.alpha.confirmed = \"2026-11-22\"\ndeck.alpha.have = [\"Sol Ring\"]\nspare.cards = []\n");
    let inline = format!(
        "{HEADER}deck = {{ alpha = {{ confirmed = \"2026-11-22\", have = [{{ card = \"Sol Ring\", copies = 1 }}] }} }}\nspare = {{ cards = [] }}\n"
    );
    for text in [
        dotted,
        inline,
        format!("\u{feff}{GOOD}"),
        GOOD.replace('\n', "\r\n"),
        GOOD.replace(
            "have = [\"Sol Ring\"]",
            "have = [\n\"Sol Ring\", # source: audit/example.md:17\n] # source: decklists/CHANGELOG.md:1",
        ),
    ] {
        let r = env.check(&text, true);
        assert!(r.ok(), "{}", r.text());
    }
}

#[test]
fn empty_whitespace_and_long_card_names_are_handled_without_panics() {
    let env = Env::new();
    for name in ["".to_owned(), " \t\r\n".to_owned(), "\u{a0}".to_owned(), "X".repeat(65536)] {
        let text = GOOD.replace("\"Sol Ring\"", &serde_json::to_string(&name).unwrap());
        let r = env.check(&text, true);
        assert!(!r.ok(), "unexpectedly accepted {} byte name", name.len());
        serde_json::from_str::<Value>(&r.json()).unwrap();
    }
    let huge = format!("{}\n{GOOD}", "# source: local fixture\n".repeat(50000));
    assert!(env.check(&huge, true).ok());
}

#[test]
fn basics_are_excluded_from_reference_lists_and_rejected_in_every_inventory_place() {
    let env = Env::new();
    let basics = [
        "Plains",
        "Island",
        "Swamp",
        "Mountain",
        "Forest",
        "Wastes",
        "Snow-Covered Plains",
        "Snow-Covered Island",
        "Snow-Covered Swamp",
        "Snow-Covered Mountain",
        "Snow-Covered Forest",
    ];
    let mut cards = vec!["Sol Ring"];
    cards.extend(basics);
    cards.extend(basics);
    env.cards(&cards);
    for catalog in [false, true] {
        assert!(env.check(GOOD, catalog).ok());
        for basic in basics {
            for place in 0..4 {
                let entry = format!("[\"  {}  \"]", basic.to_uppercase());
                let mut lists = ["[]", "[]", "[]", "[]"];
                lists[place] = &entry;
                let r = env.check(&document(lists[0], lists[1], lists[2], lists[3]), catalog);
                assert!(problem(&r, Level::Error, None, None, "basic land"), "{}", r.text());
            }
        }
    }
}

#[test]
fn pool_sections_are_not_required_and_commanders_follow_cards_only() {
    let env = Env::new();
    env.decks(json!({"mine": [{"key": "alpha", "commander": "Lightning Bolt", "cards": ["Sol Ring"]}], "pool": [{"key": "enemy", "cards": ["Extra Card"]}]}));
    assert!(env.check(GOOD, true).ok());
    env.decks(json!({"mine": [{"key": "alpha", "commander": "Lightning Bolt", "cards": ["Sol Ring", "Lightning Bolt"]}], "pool": []}));
    let r = env.check(GOOD, true);
    assert!(problem(&r, Level::Error, Some("alpha"), Some("Lightning Bolt"), "not in the ownership file"));
    let complete = GOOD.replace("[\"Sol Ring\"]", "[\"Sol Ring\", \"Lightning Bolt\"]");
    assert!(env.check(&complete, true).ok());
    let r = env.check(&format!("{complete}\n[deck.enemy]\nhave = []\n"), true);
    assert!(problem(&r, Level::Error, Some("enemy"), None, "no such deck"));
}

#[test]
fn real_seed_comments_parse_and_every_owned_deck_addition_or_cut_is_detected() {
    let env = Env::new();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let decks: Value = serde_json::from_slice(&std::fs::read(root.join("data/decks.json")).unwrap()).unwrap();
    let seed = std::fs::read_to_string(root.join("data/ownership.toml")).unwrap();
    assert!(seed.contains("# source:"));
    env.decks(decks.clone());
    assert!(env.check(&seed, false).ok());
    for i in 0..decks["mine"].as_array().unwrap().len() {
        let key = decks["mine"][i]["key"].as_str().unwrap();
        let mut added = decks.clone();
        added["mine"][i]["cards"].as_array_mut().unwrap().push(json!("Ownership Adversarial Added Card"));
        env.decks(added);
        let r = env.check(&seed, false);
        assert!(
            problem(&r, Level::Error, Some(key), Some("Ownership Adversarial Added Card"), "not in the ownership file"),
            "{}",
            r.text()
        );
        let mut cut = decks.clone();
        // Each real deck's first entry is a nonbasic; verify that it actually has an inventory entry first.
        let card = cut["mine"][i]["cards"].as_array_mut().unwrap().remove(0);
        let card = card.as_str().unwrap();
        assert!(seed.contains(card), "choose a nonbasic seed entry, got {card}");
        env.decks(cut);
        let r = env.check(&seed, false);
        assert!(problem(&r, Level::Error, Some(key), Some(card), "not in the deck list"), "{}", r.text());
    }
}

#[test]
fn malformed_reference_card_lists_must_not_pass_validation() {
    let env = Env::new();
    let mut accepted = vec![];
    for deck in [
        json!({"key": "alpha"}),
        json!({"key": "alpha", "cards": "Sol Ring"}),
        json!({"key": "alpha", "cards": null}),
        json!({"key": "alpha", "cards": {"name": "Sol Ring"}}),
    ] {
        env.decks(json!({"mine": [deck]}));
        std::fs::write(env.0.join("own.toml"), format!("{HEADER}[deck.alpha]\nhave = []\n")).unwrap();
        if let Ok(r) = ownership::check(&Project::new(&env.0), &env.opts(true)) {
            if r.ok() {
                accepted.push(format!("{deck}: {}", r.text()));
            }
        }
    }
    assert!(accepted.is_empty(), "Malformed reference card lists accepted:\n{}", accepted.join("\n"));
}

#[test]
fn non_string_reference_cards_must_not_silently_disappear() {
    let env = Env::new();
    let mut accepted = vec![];
    for card in [Value::Null, json!(42), json!(false), json!({"name": "Lightning Bolt"}), json!(["Lightning Bolt"])] {
        env.decks(json!({"mine": [{"key": "alpha", "cards": ["Sol Ring", card]}]}));
        std::fs::write(env.0.join("own.toml"), GOOD).unwrap();
        if let Ok(r) = ownership::check(&Project::new(&env.0), &env.opts(true)) {
            if r.ok() {
                accepted.push(format!("{card}: {}", r.text()));
            }
        }
    }
    assert!(accepted.is_empty(), "Invalid reference entries were dropped:\n{}", accepted.join("\n"));
}

#[test]
fn broken_reference_json_and_catalogs_are_errors() {
    let env = Env::new();
    for data in ["", "{", "{}", "{\"mine\": {}}", "{\"mine\": [{}]}"] {
        std::fs::write(env.0.join("data/decks.json"), data).unwrap();
        assert!(ownership::check(&Project::new(&env.0), &env.opts(true)).is_err(), "{data}");
    }
    env.cards(&["Sol Ring"]);
    for data in ["", "{", "{}", "{\"meta\": {\"schema\": 2}, \"cards\": []}"] {
        std::fs::write(env.0.join("data/knowledge/oracle.json"), data).unwrap();
        let r = env.check(GOOD, true);
        assert!(problem(&r, Level::Error, None, None, "catalog cannot be read"), "{}", r.text());
    }
}

#[test]
fn cli_exit_codes_and_json_match_errors_warnings_and_info() {
    let env = Env::new();
    let cases = [
        (GOOD.to_owned(), true),
        (GOOD.replace("confirmed = \"2026-11-22\"\n", ""), true),
        (GOOD.replace("cards = []", "cards = [\"Sol Ring\"]"), true),
        (GOOD.replace("\"Sol Ring\"", "\"Lightning Bolt\""), false),
        ("version = [".to_owned(), false),
    ];
    for (text, ok) in cases {
        let r = env.check(&text, true);
        let text_out = env.cli(&[]);
        assert_eq!(text_out.status.code(), Some(i32::from(!ok)));
        assert!(text_out.stderr.is_empty(), "{}", String::from_utf8_lossy(&text_out.stderr));
        assert_eq!(
            String::from_utf8(text_out.stdout).unwrap(),
            r.text().replace(env.0.join("own.toml").to_str().unwrap(), "own.toml")
        );
        let json_out = env.cli(&["--json"]);
        assert_eq!(json_out.status.code(), Some(i32::from(!ok)));
        let parsed: Value = serde_json::from_slice(&json_out.stdout).unwrap();
        // CLI uses a relative filename; reports otherwise contain the same data.
        let expected: Value =
            serde_json::from_str(&r.json().replace(env.0.join("own.toml").to_str().unwrap(), "own.toml")).unwrap();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn cli_paths_strict_unknown_and_repeated_flags_have_predictable_results() {
    let env = Env::new();
    env.check(GOOD, true);
    let relative = env.cli(&["--file", "own.toml", "--check", "--check", "--json", "--json"]);
    assert_eq!(relative.status.code(), Some(0));
    assert_eq!(serde_json::from_slice::<Value>(&relative.stdout).unwrap()["ok"], true);
    for args in [
        vec!["--file", "missing.toml"],
        vec!["--file", "data"],
        vec!["--strict", "--data-dir", "nonexistent"],
        vec!["--strict", "--data-dir", "own.toml"],
        vec!["--unknown"],
        vec!["--file"],
        vec!["--data-dir"],
    ] {
        let out = env.cli(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
    // Explicit directories with no oracle.json use the documented fallback, even if nonexistent or a file.
    for dir in ["nonexistent", "own.toml", "data/none"] {
        let out = env.cli(&["--data-dir", dir, "--json"]);
        assert_eq!(out.status.code(), Some(0));
        let json: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(json["catalog"], false);
        assert_eq!(json["warnings"], 1);
    }
    let out = env.cli(&["--strict", "--strict", "--data-dir", "data/none", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(serde_json::from_slice::<Value>(&out.stdout).unwrap()["errors"], 1);
}

#[test]
fn deeply_nested_toml_is_rejected_without_crashing_the_cli() {
    let env = Env::new();
    let mut failures = vec![];
    for depth in [128, 512, 2048] {
        let inputs = [
            format!("{HEADER}[deck.alpha]\nhave = {}0{}\n", "[".repeat(depth), "]".repeat(depth)),
            format!("{HEADER}junk = {}0{}\n", "{ x = ".repeat(depth), "}".repeat(depth)),
            format!("{HEADER}[deck.alpha.{}]\nhave = []\n", vec!["x"; depth].join(".")),
        ];
        for (shape, text) in inputs.iter().enumerate() {
            std::fs::write(env.0.join("own.toml"), text).unwrap();
            let out = env.cli(&["--json"]);
            if out.status.code() != Some(1) {
                failures.push(format!(
                    "depth={depth} shape={shape} status={:?}: {}",
                    out.status,
                    String::from_utf8_lossy(&out.stderr)
                ));
            } else {
                let parsed: Value = serde_json::from_slice(&out.stdout).unwrap();
                assert_eq!(parsed["ok"], false);
            }
        }
    }
    assert!(failures.is_empty(), "Nested TOML crashed or was accepted:\n{}", failures.join("\n"));
}

#[test]
fn seeded_random_and_mutated_inputs_never_panic_and_always_render_valid_json() {
    let env = Env::new();
    let mut seed = 0x6f77_6e65_7273_6869_u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let alphabet: Vec<char> =
        "versionupdateddecksparehaveproxymissingcardcopiesSol Ring\n\r\t []{}=,#\"'.01239-+é’Æ\u{feff}\0"
            .chars()
            .collect();
    for i in 0..512 {
        let text = if i % 2 == 0 {
            let len = (next() % 512) as usize;
            (0..len).map(|_| alphabet[next() as usize % alphabet.len()]).collect::<String>()
        } else {
            let mut chars: Vec<char> = GOOD.chars().collect();
            for _ in 0..1 + next() % 12 {
                let pos = next() as usize % chars.len();
                chars[pos] = alphabet[next() as usize % alphabet.len()];
            }
            chars.into_iter().collect()
        };
        let result = std::panic::catch_unwind(|| env.check(&text, i % 4 < 2));
        let r = result.unwrap_or_else(|_| panic!("checker panicked on deterministic case {i}: {text:?}"));
        let parsed: Value = serde_json::from_str(&r.json()).unwrap();
        assert_eq!(parsed["ok"], r.ok());
        assert_eq!(parsed["errors"], r.errors());
        let _ = r.text();
    }
    // Invalid UTF-8 is an input error, not a panic or lossy name match.
    std::fs::write(env.0.join("own.toml"), [0xff, 0xfe, 0x00]).unwrap();
    let r = ownership::check(&Project::new(&env.0), &env.opts(true)).unwrap();
    assert!(!r.ok());
}
