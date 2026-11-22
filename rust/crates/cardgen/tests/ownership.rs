//! `cardgen ownership --check`: `data/ownership.toml` against `data/decks.json` (`mine`) and the catalog. A tiny
//! catalog (`tests/fixtures/ownership_oracle.json`: accents, an Æ, a face name, a shared name) and decks written per
//! test; the last tests read the repository's real `data/ownership.toml` and `data/decks.json` without a catalog.
//! Nothing uses the network.

use cardgen::data::Project;
use cardgen::ownership::{self, Level, Options, Report};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const DECKS: &str = r#"{"version": 1, "mine": [
 {"key": "alpha", "commander": "Sol Ring", "cards": ["Sol Ring", "Lightning Bolt", "Palantír of Orthanc",
   "Leech Collector // Bloodletting", "Island", "Island"]},
 {"key": "beta", "commander": "Sol Ring", "cards": ["Sol Ring", "Urza’s Saga", "Æther Vial", "Barad-dûr", "Plains"]}
], "pool": []}"#;

/// two decks, Sol Ring in both; beta unconfirmed
const GOOD: &str = r#"
version = 1
updated = "2026-11-22"
[deck.alpha]
confirmed = "2026-11-22"
have = [{ card = "Sol Ring", copies = 2 }, "Lightning Bolt", "Palantír of Orthanc"]
proxy = ["Leech Collector // Bloodletting"]
missing = []
[deck.beta]
have = [{ card = "Sol Ring", copies = 2 }, "Urza's Saga", "Æther Vial"]
missing = ["Barad-dûr"]
[spare]
cards = []
"#;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// a temporary project root with `data/decks.json`, `data/knowledge/oracle.json` and an empty `data/none` (a
/// `--data-dir` without a catalog); removed on drop
struct Env(PathBuf);

impl Env {
    fn new() -> Env {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p =
            std::env::temp_dir().join(format!("own-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(p.join("data/knowledge")).unwrap();
        std::fs::create_dir_all(p.join("data/none")).unwrap();
        std::fs::copy(fixture("ownership_oracle.json"), p.join("data/knowledge/oracle.json")).unwrap();
        std::fs::write(p.join("data/decks.json"), DECKS).unwrap();
        Env(p)
    }

    fn write(&self, toml: &str) -> PathBuf {
        let f = self.0.join("own.toml");
        std::fs::write(&f, toml).unwrap();
        f
    }

    /// with the catalog
    fn check(&self, toml: &str) -> Report {
        self.run(toml, "data/knowledge", false)
    }

    /// without a catalog: names compared by spelling
    fn check_plain(&self, toml: &str) -> Report {
        self.run(toml, "data/none", false)
    }

    fn run(&self, toml: &str, dir: &str, strict: bool) -> Report {
        let opts = Options { file: Some(self.write(toml)), data_dir: Some(self.0.join(dir)), strict };
        ownership::check(&Project::new(&self.0), &opts).unwrap()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn has(r: &Report, level: Level, needles: &[&str]) -> bool {
    r.problems.iter().any(|p| {
        let t = format!("{} | {}", p.deck.clone().unwrap_or_default(), p.card.clone().unwrap_or_default());
        p.level == level && needles.iter().all(|n| p.message.contains(n) || t.contains(n))
    })
}

fn dump(r: &Report) -> String {
    r.text()
}

#[test]
fn a_valid_file_passes_with_the_unconfirmed_warning_only() {
    let env = Env::new();
    for r in [env.check(GOOD), env.check_plain(GOOD)] {
        assert!(r.ok(), "{}", dump(&r));
        assert_eq!((r.errors(), r.infos()), (0, 0), "{}", dump(&r));
        // beta is unconfirmed; Sol Ring's two copies are confirmed in both decks (copies = 2)
        assert_eq!(r.warnings(), if r.catalog { 1 } else { 2 }, "{}", dump(&r));
        assert!(has(&r, Level::Warning, &["beta", "unconfirmed"]), "{}", dump(&r));
        assert!(!has(&r, Level::Warning, &["alpha", "unconfirmed"]));
        assert!(!has(&r, Level::Warning, &["Sol Ring"]), "confirmed copies silence the warning: {}", dump(&r));
        let (a, b) = (&r.decks[0], &r.decks[1]);
        assert_eq!((a.key.as_str(), a.have, a.proxy, a.missing), ("alpha", 3, 1, 0));
        assert_eq!((b.key.as_str(), b.have, b.proxy, b.missing), ("beta", 3, 0, 1));
        assert_eq!(a.confirmed.as_deref(), Some("2026-11-22"));
        assert_eq!(b.confirmed, None);
        assert_eq!(r.spare, 0);
    }
}

#[test]
fn the_summary_and_json_name_every_deck() {
    let env = Env::new();
    let r = env.check(GOOD);
    let t = r.text();
    assert!(t.contains("alpha: have 3, proxy 1, missing 0; confirmed 2026-11-22"), "{t}");
    assert!(t.contains("beta: have 3, proxy 0, missing 1; unconfirmed"), "{t}");
    assert!(t.contains("spare: 0 cards") && t.contains("0 errors, 1 warnings"), "{t}");
    let j: serde_json::Value = serde_json::from_str(&r.json()).unwrap();
    assert_eq!(j["ok"], true);
    assert_eq!(j["decks"].as_array().unwrap().len(), 2);
    assert_eq!(j["problems"][0]["level"], "warning");
}

#[test]
fn names_fold_like_find_name() {
    let env = Env::new();
    // accents, case, a face name (either face), apostrophe style, Æ spelt out
    let t = r#"
version = 1
updated = "2026-11-22"
[deck.alpha]
have = ["SOL RING", "lightning bolt", "palantir of orthanc", "bloodletting"]
[deck.beta]
have = ["Sol Ring", "Urza’s Saga", "AETHER VIAL", "barad-dur"]
"#;
    let r = env.check(t);
    assert!(r.ok(), "{}", dump(&r));
    // the same without a catalog, except that a face name counts only for a deck card that has it
    let r = env.check_plain(t);
    assert!(r.ok(), "{}", dump(&r));
    let t2 = t.replace("bloodletting", "leech collector");
    assert!(env.check(&t2).ok() && env.check_plain(&t2).ok());
}

#[test]
fn a_name_not_in_the_catalog_is_an_error_but_not_without_one() {
    let env = Env::new();
    let t = GOOD.replace("\"Lightning Bolt\"", "\"Lightning Bolte\"");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "Lightning Bolte", "not found in the card catalog"]), "{}", dump(&r));
    // it is also a card the list does not have, and the list's card is missing from the file
    assert!(has(&r, Level::Error, &["Lightning Bolte", "not in the deck list"]), "{}", dump(&r));
    assert!(has(&r, Level::Error, &["Lightning Bolt", "not in the ownership file"]), "{}", dump(&r));
    assert!(!r.ok());
}

#[test]
fn without_a_catalog_resolution_is_skipped_with_a_warning_and_strict_fails() {
    let env = Env::new();
    let r = env.check_plain(GOOD);
    assert!(r.ok() && !r.catalog, "{}", dump(&r));
    assert!(has(&r, Level::Warning, &["name resolution skipped"]), "{}", dump(&r));
    let r = env.run(GOOD, "data/none", true);
    assert!(!r.ok() && has(&r, Level::Error, &["--strict"]), "{}", dump(&r));
}

#[test]
fn a_deck_card_missing_from_the_file_and_a_file_card_not_in_the_deck() {
    let env = Env::new();
    let t = GOOD
        .replace(", \"Palantír of Orthanc\"", "")
        .replace("missing = [\"Barad-dûr\"]", "missing = [\"Barad-dûr\", \"Extra Card\"]");
    for r in [env.check(&t), env.check_plain(&t)] {
        assert!(has(&r, Level::Error, &["alpha", "Palantír of Orthanc", "not in the ownership file"]), "{}", dump(&r));
        assert!(has(&r, Level::Error, &["beta", "Extra Card", "not in the deck list"]), "{}", dump(&r));
        assert_eq!(r.errors(), 2, "{}", dump(&r));
    }
}

#[test]
fn basic_lands_are_left_out_of_the_comparison_and_never_listed() {
    let env = Env::new();
    // Island and Plains are in the decks' lists and not in the file: fine; listing one is an error
    assert!(env.check(GOOD).ok());
    let t = GOOD.replace("missing = []", "missing = [\"Island\"]");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "Island", "basic land"]), "{}", dump(&r));
    assert_eq!(r.errors(), 1, "{}", dump(&r));
    let t = GOOD.replace("cards = []", "cards = [\"Plains\"]");
    assert!(has(&env.check(&t), Level::Error, &["Plains", "basic land"]));
}

#[test]
fn a_card_with_two_statuses_or_listed_twice_in_a_deck() {
    let env = Env::new();
    let t = GOOD.replace("missing = []", "missing = [\"Lightning Bolt\"]");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "Lightning Bolt", "two statuses", "have and missing"]), "{}", dump(&r));
    let t = GOOD.replace("\"Lightning Bolt\"", "\"Lightning Bolt\", \"lightning bolt\"");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "Lightning Bolt", "listed 2 times in have"]), "{}", dump(&r));
    // two spellings of one card are one card, also by a face name
    let t = GOOD
        .replace("proxy = [\"Leech Collector // Bloodletting\"]", "proxy = [\"Leech Collector\", \"Bloodletting\"]");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "listed 2 times in proxy"]), "{}", dump(&r));
}

#[test]
fn a_deck_without_a_section_and_a_deck_that_does_not_exist() {
    let env = Env::new();
    let r = env.check("version = 1\nupdated = \"2026-11-22\"\n[deck.alpha]\nhave = [\"Sol Ring\", \"Lightning Bolt\", \"Palantír of Orthanc\", \"Leech Collector // Bloodletting\"]\n[deck.gamma]\nhave = []\n");
    assert!(has(&r, Level::Error, &["beta", "no [deck.KEY] section"]), "{}", dump(&r));
    assert!(has(&r, Level::Error, &["gamma", "no such deck"]), "{}", dump(&r));
    assert!(r.decks.iter().any(|d| d.key == "beta" && !d.listed));
    assert!(r.text().contains("beta: have 0, proxy 0, missing 0; no section"));
}

#[test]
fn unknown_keys_bad_dates_bad_version_and_malformed_toml() {
    let env = Env::new();
    let r = env.check(
        &GOOD.replace("version = 1\n", "version = 1\ncolour = 1\n").replace("cards = []", "cards = []\nnote = 1"),
    );
    assert!(has(&r, Level::Error, &["unknown top-level key", "colour"]), "{}", dump(&r));
    assert!(has(&r, Level::Error, &["unknown key", "note", "[spare]"]), "{}", dump(&r));
    let r = env.check(&GOOD.replace("proxy =", "owned = [\"x\"]\nproxy ="));
    assert!(has(&r, Level::Error, &["alpha", "unknown key", "owned"]), "{}", dump(&r));
    let r = env.check(&GOOD.replace("{ card = \"Sol Ring\", copies = 2 }", "{ card = \"Sol Ring\", note = 1 }"));
    assert!(has(&r, Level::Error, &["unknown key", "note"]), "{}", dump(&r));
    for bad in ["22-11-2026", "2026-13-01", "2026-02-30", "2026/11/22", "20261122", ""] {
        let r = env.check(&GOOD.replace("updated = \"2026-11-22\"", &format!("updated = \"{bad}\"")));
        assert!(has(&r, Level::Error, &["updated", "YYYY-MM-DD"]), "{bad}: {}", dump(&r));
        let r = env.check(&GOOD.replace("confirmed = \"2026-11-22\"", &format!("confirmed = \"{bad}\"")));
        assert!(has(&r, Level::Error, &["alpha", "confirmed", "YYYY-MM-DD"]), "{bad}: {}", dump(&r));
    }
    // an unquoted TOML date is not the documented string
    let r = env.check(&GOOD.replace("updated = \"2026-11-22\"", "updated = 2026-11-22"));
    assert!(has(&r, Level::Error, &["updated", "quoted"]), "{}", dump(&r));
    assert!(has(&env.check(&GOOD.replace("version = 1", "version = 2")), Level::Error, &["version must be 1"]));
    assert!(has(&env.check(&GOOD.replace("version = 1\n", "")), Level::Error, &["version"]));
    assert!(has(
        &env.check(&GOOD.replace("updated = \"2026-11-22\"\n[deck.alpha]", "[deck.alpha]")),
        Level::Error,
        &["updated"]
    ));
    let r = env.check("version = 1\n[deck.alpha\nhave = [");
    assert!(has(&r, Level::Error, &["malformed TOML"]) && r.decks.is_empty(), "{}", dump(&r));
    // a duplicate table or key is malformed TOML too
    assert!(has(&env.check(&format!("{GOOD}\n[deck.alpha]\n")), Level::Error, &["malformed TOML"]));
    let r = env.check("version = 1\nupdated = \"2026-11-22\"\ndeck = 3\n");
    assert!(has(&r, Level::Error, &["deck must be a table"]), "{}", dump(&r));
    let r = env.check(&GOOD.replacen("have = [", "have = \"Sol Ring\"\nmissing_ = [", 1));
    assert!(!r.ok(), "a have that is not a list: {}", dump(&r));
}

#[test]
fn a_missing_file_is_an_error() {
    let env = Env::new();
    let opts =
        Options { file: Some(env.0.join("nope.toml")), data_dir: Some(env.0.join("data/knowledge")), strict: false };
    let r = ownership::check(&Project::new(&env.0), &opts).unwrap();
    assert!(!r.ok() && r.errors() == 1, "{}", dump(&r));
}

#[test]
fn bad_copies_are_errors() {
    let env = Env::new();
    for bad in ["0", "-1", "1.5", "\"2\"", "true"] {
        let t = GOOD.replace("\"Lightning Bolt\"", &format!("{{ card = \"Lightning Bolt\", copies = {bad} }}"));
        let r = env.check(&t);
        assert!(
            has(&r, Level::Error, &["Lightning Bolt", "copies must be a whole number of 1 or more"]),
            "{bad}: {}",
            dump(&r)
        );
    }
}

#[test]
fn a_copies_table_counts_in_spare_and_in_a_deck() {
    let env = Env::new();
    let t =
        GOOD.replace("cards = []", "cards = [{ card = \"Extra Card\", copies = 3 }, { card = \"Pick Your Poison\" }]");
    let r = env.check(&t);
    assert!(r.ok(), "{}", dump(&r));
    assert_eq!(r.spare, 4);
    assert!(r.text().contains("spare: 4 cards"));
}

// ------------------------------------------------------------------------------------------------ spare

#[test]
fn a_spare_card_that_a_deck_plays_is_a_second_copy_info_not_an_error() {
    let env = Env::new();
    let t = GOOD.replace("cards = []", "cards = [\"lightning bolt\", \"Leech Collector\", \"Extra Card\"]");
    for r in [env.check(&t), env.check_plain(&t)] {
        assert!(r.ok(), "{}", dump(&r));
        assert_eq!(r.infos(), 2, "{}", dump(&r));
        assert!(has(&r, Level::Info, &["lightning bolt", "second copy", "alpha"]), "{}", dump(&r));
        assert!(has(&r, Level::Info, &["Leech Collector", "second copy", "alpha"]), "{}", dump(&r));
        assert!(r.text().contains("INFO"));
        assert_eq!(r.spare, 3);
        // info is not counted as a warning
        assert_eq!(r.warnings(), if r.catalog { 1 } else { 2 });
    }
}

#[test]
fn spare_duplicates_unknown_cards_and_bad_shapes_are_errors() {
    let env = Env::new();
    let r = env.check(&GOOD.replace("cards = []", "cards = [\"Extra Card\", \"EXTRA CARD\"]"));
    assert!(has(&r, Level::Error, &["listed twice in spare"]), "{}", dump(&r));
    let r = env.check(&GOOD.replace("cards = []", "cards = [\"Nonesuch\"]"));
    assert!(has(&r, Level::Error, &["Nonesuch", "not found in the card catalog (spare)"]), "{}", dump(&r));
    let r = env.check(&GOOD.replace("cards = []", "cards = \"Extra Card\""));
    assert!(has(&r, Level::Error, &["spare.cards must be a list"]), "{}", dump(&r));
    let r = env.check(&GOOD.replace("[spare]\ncards = []", "").replace("version = 1\n", "version = 1\nspare = 3\n"));
    assert!(has(&r, Level::Error, &["spare must be a table"]), "{}", dump(&r));
}

// ------------------------------------------------------------------------------------------------ copies

#[test]
fn a_have_card_in_two_decks_warns_until_every_entry_says_copies_equal_to_the_decks() {
    let env = Env::new();
    // no copies anywhere: warning
    let plain = GOOD.replace("{ card = \"Sol Ring\", copies = 2 }", "\"Sol Ring\"");
    let r = env.check(&plain);
    assert!(r.ok(), "{}", dump(&r));
    assert!(has(&r, Level::Warning, &["Sol Ring", "2 decks", "alpha, beta", "confirm 2 copies"]), "{}", dump(&r));
    // confirmed in one deck only: still a warning
    let half = GOOD.replacen("{ card = \"Sol Ring\", copies = 2 }", "\"Sol Ring\"", 1);
    let r = env.check(&half);
    assert!(r.ok() && has(&r, Level::Warning, &["Sol Ring", "confirm 2 copies"]), "{}", dump(&r));
    // confirmed in both: no warning
    assert!(!has(&env.check(GOOD), Level::Warning, &["Sol Ring"]));
}

#[test]
fn copies_that_differ_from_the_number_of_decks_are_errors() {
    let env = Env::new();
    for (bad, who) in [("3", "alpha"), ("1", "beta")] {
        let t = if who == "alpha" {
            GOOD.replacen("copies = 2", &format!("copies = {bad}"), 1)
        } else {
            let i = GOOD.rfind("copies = 2").unwrap();
            format!("{}copies = {bad}{}", &GOOD[..i], &GOOD[i + "copies = 2".len()..])
        };
        let r = env.check(&t);
        assert!(!r.ok(), "{}", dump(&r));
        assert!(
            has(&r, Level::Error, &[who, "Sol Ring", &format!("copies = {bad}"), "2 decks have it"]),
            "{}",
            dump(&r)
        );
    }
}

#[test]
fn copies_on_a_card_held_in_a_single_deck_must_be_one_or_absent() {
    let env = Env::new();
    let t = GOOD.replace("\"Lightning Bolt\"", "{ card = \"Lightning Bolt\", copies = 2 }");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["alpha", "Lightning Bolt", "copies = 2", "only this deck has it"]), "{}", dump(&r));
    assert_eq!(r.errors(), 1, "{}", dump(&r));
    // one is fine, written or not
    let t = GOOD.replace("\"Lightning Bolt\"", "{ card = \"Lightning Bolt\", copies = 1 }");
    assert!(env.check(&t).ok());
    // proxy and missing cards take no copies either
    let t = GOOD.replace("missing = [\"Barad-dûr\"]", "missing = [{ card = \"Barad-dûr\", copies = 2 }]");
    let r = env.check(&t);
    assert!(has(&r, Level::Error, &["beta", "Barad-dûr", "copies on a missing card"]), "{}", dump(&r));
    let t = GOOD.replace(
        "proxy = [\"Leech Collector // Bloodletting\"]",
        "proxy = [{ card = \"Leech Collector // Bloodletting\", copies = 2 }]",
    );
    assert!(has(&env.check(&t), Level::Error, &["alpha", "copies on a proxy card"]));
}

// ------------------------------------------------------------------------------------------------ command

#[test]
fn the_command_reports_the_pass_flag_and_takes_its_options() {
    let env = Env::new();
    let f = env.write(GOOD);
    let args = |extra: &[&str]| -> Vec<String> {
        ["--check", "--file", f.to_str().unwrap(), "--data-dir", env.0.join("data/knowledge").to_str().unwrap()]
            .iter()
            .chain(extra)
            .map(|s| s.to_string())
            .collect()
    };
    let p = Project::new(&env.0);
    let (out, ok) = ownership::command(&p, &args(&[])).unwrap();
    assert!(ok && out.contains("0 errors"), "{out}");
    let (out, ok) = ownership::command(&p, &args(&["--json"])).unwrap();
    assert!(ok && serde_json::from_str::<serde_json::Value>(&out).unwrap()["ok"] == true);
    std::fs::write(&f, "version = 1").unwrap();
    assert!(!ownership::command(&p, &args(&[])).unwrap().1);
    assert!(ownership::command(&p, &["--file".into(), "x".into()]).is_err(), "--check is required");
    assert!(ownership::command(&p, &["--check".into(), "--bogus".into()]).is_err());
    assert!(ownership::command(&p, &["--check".into(), "--file".into()]).is_err());
}

// ------------------------------------------------------------------------------------------------ the real files

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// a directory with no oracle.json, so the check falls back to the spelling of decks.json's names
fn no_catalog() -> PathBuf {
    let d = std::env::temp_dir().join(format!("own-nocat-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_real_ownership_file_passes_against_the_real_decks() {
    let dir = no_catalog();
    let opts = Options { file: None, data_dir: Some(dir.clone()), strict: false };
    let r = ownership::check(&Project::new(repo()), &opts).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        r.ok(),
        "data/ownership.toml does not match data/decks.json: a deck list changed. In [deck.KEY], add each card reported \
         'not in the ownership file' under the status JD gives (have/proxy/missing; ask JD, don't assume have), and \
         remove each card reported 'not in the deck list' (move it to [spare] if JD keeps it). See \
         rust/crates/cardgen/README.md, Ownership.\n{}",
        r.text()
    );
    assert!(Path::new(&repo()).join("data/ownership.toml").is_file());
    // every deck of mine has a section
    assert!(r.decks.iter().all(|d| d.listed), "{}", r.text());
    assert!(!r.decks.is_empty());
}
