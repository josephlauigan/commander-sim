//! Adversarial review of the phase-3 role classifier (rules, overrides, side table, validation, audit, CLI).
//!
//! Offline. Tests named `defect_*` fail on purpose: each pins a real defect found by this review and stays
//! un-ignored. Tests that read the full local catalog are `#[ignore]`d (run with `-- --ignored`).
use cardgen::data::Project;
use cardgen::knowledge::{self, FindError, Index, Legal, Query, SCHEMA};
use cardgen::roles::{self, Assignment, Classifier, Evidence, IDS, RULES, Taxonomy};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, mpsc};
use std::time::{Duration, Instant};

const TAXONOMY: &str = include_str!("../../../../data/knowledge/roles.toml");
const NEEDS_CATALOG: &str = "needs data/knowledge/oracle.json (run cardgen bulk-sync)";
/// Roles whose rule hits always route to the LLM stage (mirrors the classifier's `ambiguous` list).
const ALWAYS_ROUTED: [&str; 9] =
    ["selection", "recursion", "gy-hate", "finisher", "stax", "drain", "copy", "evasion", "protection"];

// ------------------------------------------------------------------------------------------------ helpers

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn repo_root() -> PathBuf {
    manifest().join("../../..").canonicalize().unwrap()
}
fn fixture(name: &str) -> PathBuf {
    manifest().join("tests/fixtures").join(name)
}

struct Tmp(PathBuf);
impl Tmp {
    fn new(tag: &str) -> Tmp {
        let p = std::env::temp_dir().join(format!(
            "roles-adv-{}-{}-{tag}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rec(name: &str, type_line: &str, text: &str) -> Value {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut v = json!({
        "name": name, "cmc": 1.0, "type_line": type_line, "color_identity": [], "keywords": [],
        "layout": "normal", "game_changer": false, "legalities": {"commander": "legal"},
        "oracle_id": format!("00000000-0000-4000-8000-{n:012}"),
        "released_at": "2026-01-01", "set": "tst", "rarity": "common", "reserved": false,
    });
    if !text.is_empty() {
        v["oracle_text"] = json!(text);
    }
    v
}
fn face(name: &str, type_line: &str, text: &str, loyalty: Option<&str>) -> Value {
    let mut f = json!({"name": name, "type_line": type_line, "mana_cost": ""});
    if !text.is_empty() {
        f["oracle_text"] = json!(text);
    }
    if let Some(l) = loyalty {
        f["loyalty"] = json!(l);
    }
    f
}
fn multi(name: &str, layout: &str, type_line: &str, faces: Vec<Value>) -> Value {
    let mut v = rec(name, type_line, "");
    v["layout"] = json!(layout);
    v["card_faces"] = json!(faces);
    v
}
fn with(mut v: Value, key: &str, value: Value) -> Value {
    v[key] = value;
    v
}
fn catalog_json(records: &[Value]) -> Value {
    json!({"meta": {"schema": SCHEMA, "bulk_updated_at": "adversarial", "count": records.len()}, "cards": records})
}
fn write_catalog(path: &Path, records: &[Value]) {
    std::fs::write(path, serde_json::to_vec(&catalog_json(records)).unwrap()).unwrap();
}
fn index_of(records: &[Value]) -> Index {
    let t = Tmp::new("idx");
    let p = t.0.join("oracle.json");
    write_catalog(&p, records);
    Index::load(&p).unwrap()
}
fn clf() -> &'static Classifier {
    static C: OnceLock<Classifier> = OnceLock::new();
    C.get_or_init(|| Classifier::new().unwrap())
}
fn classify_record(v: Value) -> Assignment {
    let i = index_of(&[v]);
    clf().classify(&i.cards[0]).unwrap()
}
fn classify_text(type_line: &str, text: &str) -> Assignment {
    classify_record(rec("Test Subject", type_line, text))
}
fn named(i: &Index, name: &str) -> usize {
    let hits = i.lookup(name);
    assert_eq!(hits.len(), 1, "lookup {name}");
    hits[0] as usize
}
fn classify_named(i: &Index, name: &str) -> Assignment {
    clf().classify(&i.cards[named(i, name)]).unwrap()
}
fn has(a: &Assignment, role: &str) -> bool {
    a.roles.iter().any(|r| r == role)
}
fn roles_sorted(a: &Assignment) -> Vec<&str> {
    let mut v: Vec<&str> = a.roles.iter().map(String::as_str).collect();
    v.sort();
    v
}
fn fixture_records(name: &str) -> Vec<Value> {
    let v: Value = serde_json::from_slice(&std::fs::read(fixture(name)).unwrap()).unwrap();
    v["cards"].as_array().unwrap().clone()
}
fn roles_fixture() -> Index {
    Index::load(&fixture("roles_oracle.json")).unwrap()
}
fn audit_cards() -> Index {
    Index::load(&fixture("adversarial_roles/audit_cards.json")).unwrap()
}

enum Run<T> {
    Done(T, Duration),
    Panicked,
    TimedOut,
}
/// Run `f` on its own thread; a runaway is abandoned after `secs` (the process exit reaps it).
fn timeboxed<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> Run<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let t0 = Instant::now();
        let r = std::panic::catch_unwind(AssertUnwindSafe(f));
        let _ = tx.send(r.map(|v| (v, t0.elapsed())));
    });
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(Ok((v, d))) => Run::Done(v, d),
        Ok(Err(_)) => Run::Panicked,
        Err(_) => Run::TimedOut,
    }
}

/// A temporary project root laid out like the repository (`data/knowledge/...`, `data/decks.json`, ...).
struct Env(PathBuf);
impl Env {
    fn blank() -> Env {
        let tmp = Tmp::new("env");
        let root = tmp.0.clone();
        std::mem::forget(tmp); // Env::drop removes it
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("roles.toml"), TAXONOMY).unwrap();
        std::fs::write(dir.join("roles_overrides.toml"), "").unwrap();
        std::fs::write(dir.join("roles_gold.toml"), "").unwrap();
        std::fs::write(root.join("data/decks.json"), r#"{"mine":[]}"#).unwrap();
        std::fs::write(root.join("data/cards.json"), r#"{"cards":[]}"#).unwrap();
        Env(root)
    }
    fn fixture() -> Env {
        let e = Env::blank();
        std::fs::copy(fixture("roles_oracle.json"), e.kdir().join("oracle.json")).unwrap();
        e.set_deck("Counterspell", &["Counterspell"], &[("Counterspell", json!({"ctr": "any"}))]);
        e
    }
    fn with_records(records: &[Value]) -> Env {
        let e = Env::blank();
        write_catalog(&e.kdir().join("oracle.json"), records);
        e
    }
    fn kdir(&self) -> PathBuf {
        self.0.join("data/knowledge")
    }
    fn set_deck(&self, commander: &str, cards: &[&str], tags: &[(&str, Value)]) {
        std::fs::write(
            self.0.join("data/decks.json"),
            json!({"mine": [{"key": "test", "ident": "WUBRG", "commander": commander, "cards": cards}]}).to_string(),
        )
        .unwrap();
        let cards: Vec<Value> = tags.iter().map(|(n, t)| json!({"name": n, "source": "manual", "tags": t})).collect();
        std::fs::write(self.0.join("data/cards.json"), json!({"cards": cards}).to_string()).unwrap();
    }
    fn project(&self) -> Project {
        Project::new(&self.0)
    }
    fn args(&self, args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).chain(["--data-dir".into(), self.kdir().display().to_string()]).collect()
    }
    fn command(&self, args: &[&str]) -> Result<(String, bool), String> {
        roles::command(&self.project(), &self.args(args))
    }
    fn find(&self, args: &[&str]) -> Result<String, FindError> {
        knowledge::find(&self.project(), &self.args(args))
    }
    fn write(&self, rel: &str, text: &str) {
        std::fs::write(self.kdir().join(rel), text).unwrap();
    }
    fn read(&self, rel: &str) -> Vec<u8> {
        std::fs::read(self.kdir().join(rel)).unwrap()
    }
    fn rules(&self) {
        self.command(&["--rules"]).unwrap();
    }
    /// every file under data/ with its bytes, for "this command wrote nothing" checks
    fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(&p, root, out);
                } else {
                    out.insert(p.strip_prefix(root).unwrap().display().to_string(), std::fs::read(&p).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.0, &self.0, &mut out);
        out
    }
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Everything the classifier promises about one assignment, as readable problems (empty = fine).
fn problems(c: &knowledge::Card, a: &Assignment) -> Vec<String> {
    let mut bad = vec![];
    let name = &c.raw.name;
    if a.roles.len() > 4 {
        bad.push(format!("{name}: {} roles", a.roles.len()));
    }
    let unique: BTreeSet<_> = a.roles.iter().collect();
    if unique.len() != a.roles.len() {
        bad.push(format!("{name}: repeated role {:?}", a.roles));
    }
    if a.primary != a.roles.first().cloned().unwrap_or_default() {
        bad.push(format!("{name}: primary {:?} is not the first of {:?}", a.primary, a.roles));
    }
    let keys: BTreeSet<_> = a.evidence.keys().collect();
    if keys != unique {
        bad.push(format!("{name}: evidence keys {keys:?} differ from roles {:?}", a.roles));
    }
    let lines = c.lines();
    for (role, e) in &a.evidence {
        if !IDS.contains(&role.as_str()) {
            bad.push(format!("{name}: unknown role {role}"));
        }
        if e.source != "rule" {
            bad.push(format!("{name}: {role} has source {}", e.source));
            continue;
        }
        match e.line.and_then(|i| lines.get(i)) {
            None => bad.push(format!("{name}: {role} cites missing line {:?}", e.line)),
            Some((f, _)) if *f != e.face => bad.push(format!("{name}: {role} face {:?} vs line face {f:?}", e.face)),
            _ => {}
        }
        if !RULES.iter().any(|r| r.id == e.rule && r.role == role) {
            bad.push(format!("{name}: {role} cites rule {} of another role", e.rule));
        }
    }
    if a.roles.iter().any(|r| r == "finisher") && a.roles[0] != "finisher" {
        bad.push(format!("{name}: finisher is not first in {:?}", a.roles));
    }
    if a.roles.iter().any(|r| r == "lifegain") && a.roles.last().unwrap() != "lifegain" {
        bad.push(format!("{name}: lifegain is not last in {:?}", a.roles));
    }
    if !a.needs_llm && a.roles.iter().any(|r| ALWAYS_ROUTED.contains(&r.as_str())) {
        bad.push(format!("{name}: risky role {:?} but needs_llm=false", a.roles));
    }
    if !a.needs_llm && !a.roles.is_empty() && a.roles.iter().all(|r| roles::role_src(r) != "rules") {
        bad.push(format!("{name}: only LLM-domain roles {:?} but needs_llm=false", a.roles));
    }
    bad
}

// ------------------------------------------------------------------------------------------------ A. robustness

#[test]
fn degenerate_texts_never_panic_and_keep_invariants() {
    let texts = [
        "",
        " ",
        "\n",
        "\n\n\n",
        "\t \u{a0}\u{3000}",
        "()",
        "((((",
        "))))",
        "(a(b)c",
        "a)b(c",
        ")(",
        "~",
        "~ ~ ~",
        "//",
        "\n//\n",
        "• • •",
        "•",
        "−",
        "−:",
        "−3:",
        "+:",
        "0:",
        ":",
        "::::",
        "{T}:",
        "{",
        "}",
        "\u{0}",
        "{T}: Add",
        "Add",
        "add {",
        "Counter target",
        "Search your library",
        "Draw",
        "Choose one —\n•",
        "Choose one —\n•\n•",
        "Whenever",
        "Whenever ~",
        "At the beginning",
        "Create a token",
        "Return from graveyard to the battlefield",
        "−1:",
        "-1: Draw a card.",
        "− 3: Draw a card.",
        "−99999999999999999999999: Draw a card.",
    ];
    for kind in ["Creature — Test", "Instant", "Land", "Legendary Planeswalker — Test", "", "Artifact — Equipment"]
    {
        for text in texts {
            let v = with(rec("Test Subject", kind, text), "loyalty", json!("4"));
            let i = index_of(&[v]);
            let a = clf().classify(&i.cards[0]).unwrap_or_else(|e| panic!("{kind:?} {text:?}: {e}"));
            let bad = problems(&i.cards[0], &a);
            assert!(bad.is_empty(), "{kind:?} {text:?}: {bad:?}");
        }
    }
}

#[test]
fn empty_and_whitespace_cards_have_no_roles_and_do_not_route() {
    for text in ["", " ", "\n", " \n \n\t", "\u{a0}", "\u{3000}\n\u{2003}"] {
        let a = classify_text("Creature — Test", text);
        assert!(a.roles.is_empty() && !a.needs_llm, "{text:?}: {a:?}");
    }
    // a card with no oracle_text field at all, and one with only empty faces
    let a = classify_record(multi(
        "Face // Off",
        "transform",
        "Creature // Creature",
        vec![face("Face", "Creature", "", None), face("Off", "Creature", "", None)],
    ));
    assert!(a.roles.is_empty() && !a.needs_llm, "{a:?}");
}

#[test]
fn records_with_missing_fields_load_and_classify() {
    let bare = json!({"name": "Bare", "oracle_id": "bare-1"});
    let no_faces = with(rec("NoFaces", "Creature // Instant", "Draw a card."), "card_faces", json!([]));
    let sparse_face = with(
        rec("Sparse", "", ""),
        "card_faces",
        json!([{"name": "A"}, {"name": "B", "oracle_text": "Draw two cards."}]),
    );
    let null_text = json!({"name": "NullText", "oracle_id": "bare-2", "oracle_text": null, "type_line": "Instant"});
    let i = index_of(&[bare, no_faces, sparse_face, null_text]);
    for c in &i.cards {
        let a = clf().classify(c).unwrap_or_else(|e| panic!("{}: {e}", c.raw.name));
        assert!(problems(c, &a).is_empty(), "{}: {a:?}", c.raw.name);
    }
    // the sparse face's second face still counts
    assert!(has(&classify_named(&i, "Sparse"), "draw"));
}

#[test]
fn unicode_and_control_text_never_panics() {
    let long_kelvin: String = "K".repeat(400);
    let texts = [
        "Draw a card. 🙂🙂🙂 and 👨\u{200d}👩\u{200d}👧 families".to_string(),
        "Dra\u{301}w a card.".to_string(),
        "DRAW A CARD. İstanbul ß ǅ ǆ ΑΣ Σ ς".to_string(),
        "\u{202e}Draw a card.\u{202c}".to_string(),
        "Counter target spell.\r\nDraw a card.\r\n".to_string(),
        "Draw a card.\u{0085}Destroy target creature.\u{2028}Exile all creatures.".to_string(),
        format!("{long_kelvin} {long_kelvin} target"),
        "{T}: Add {G}.\u{0}\u{1}\u{7f}".to_string(),
        "ＤＲＡＷ ＡＣＡＲＤ".to_string(),
        "ⓓⓡⓐⓦ ⓐ ⓒⓐⓡⓓ".to_string(),
        "−3: Draw a card. ― – — −".to_string(),
    ];
    for t in &texts {
        for name in ["Test", "İstanbul", "Ætherling", "Ünïcode Ñame, The 🙂", "ǅungla"] {
            let i = index_of(&[rec(name, "Creature — 🙂", t)]);
            let a = clf().classify(&i.cards[0]).unwrap_or_else(|e| panic!("{name} {t:?}: {e}"));
            assert!(problems(&i.cards[0], &a).is_empty());
        }
    }
}

#[test]
fn unbalanced_and_nested_reminder_parentheses_are_stripped_predictably() {
    let i = index_of(&[rec("Test Subject", "Instant", "")]);
    let c = &i.cards[0];
    for (input, want) in [
        ("Draw a card. (reminder (nested) text) Done.", "Draw a card.  Done."),
        ("Draw a card. ((((", "Draw a card."),
        ("Draw a card.))))", "Draw a card.))))"),
        ("a (b (c) d) e", "a  e"),
        ("a (b (c d e", "a"),
        (")x(", ")x"),
        ("()", ""),
        ("(", ""),
        ("", ""),
        ("  spaced  ", "spaced"),
    ] {
        assert_eq!(roles::normalize(c, input), want, "{input:?}");
    }
    // a half-million nesting levels cannot overflow the stack (iterative depth counter)
    let deep = format!("{}x{}", "(".repeat(500_000), ")".repeat(500_000));
    assert_eq!(roles::normalize(c, &format!("keep {deep} this")), "keep  this");
}

#[test]
fn tilde_and_double_slash_already_in_oracle_text() {
    let a = classify_record(rec("Foo", "Creature — Test", "~ deals 3 damage to any target.\n// not a joiner\n~~ ~"));
    assert!(has(&a, "removal"), "{a:?}");
    assert!(a.needs_llm, "the stray '// not a joiner' line is unparsed and must route");
    // a face name containing // and a self name that is the tilde itself
    let i = index_of(&[
        rec("~", "Instant", "~ deals 3 damage to any target."),
        rec("A // B", "Instant", "Draw two cards."),
    ]);
    for c in &i.cards {
        assert!(clf().classify(c).is_ok());
    }
    assert!(has(&classify_named(&i, "A // B"), "draw"));
    let c = &i.cards[named(&i, "~")];
    assert_eq!(roles::normalize(c, "~ ~~ x"), "~ ~~ x");
}

fn flood(unit: &str, bytes: usize) -> String {
    unit.repeat(bytes / unit.len() + 1)
}

#[test]
fn one_megabyte_texts_finish_without_panic_or_hang() {
    // An Err (regex backtrack limit) is tolerated here and pinned separately by the `defect_long_line_*` test.
    let mb = 1_000_000;
    let cases: Vec<(&str, String)> = vec![
        ("one long drain line", flood("Whenever a creature dies, each opponent loses 1 life. ", mb)),
        ("opponent loses flood, no life", flood("opponent loses ", mb)),
        ("target flood", flood("target ", mb)),
        ("destroy target flood", flood("destroy target ", mb)),
        ("search flood", flood("search your library for a card, ", mb)),
        ("many short lines", flood("Draw a card.\n", mb)),
        ("many drain lines", flood("Each opponent loses 1 life.\n", mb)),
        ("exile/return flood", flood("exile you control return ", mb)),
        ("deals damage flood", flood("deals damage to target creature, ", mb)),
        ("gains flood", flood("gains hexproof until end of turn. ", mb)),
        ("add/mana flood", flood("{T}: add {G}. add one mana ", mb)),
        ("no punctuation word", "a".repeat(mb)),
        ("brace flood", flood("{", mb)),
        ("paren flood", flood("(", mb)),
        ("emoji flood", flood("🙂", mb)),
        ("colon flood", flood(": ", mb)),
    ];
    let mut report = vec![];
    for (label, text) in cases {
        let record = rec("Test Subject", "Creature — Test", &text);
        match timeboxed(60, move || {
            let i = index_of(&[record]);
            let a = clf().classify(&i.cards[0]);
            a.map(|a| (a.roles.len(), a.needs_llm))
        }) {
            Run::Done(Ok(_), d) => {
                assert!(d < Duration::from_secs(20), "{label}: {d:?}");
                report.push(format!("{label}: {d:?}"));
            }
            Run::Done(Err(_), d) => {
                assert!(d < Duration::from_secs(20), "{label}: {d:?}");
                report.push(format!("{label}: Err after {d:?}"));
            }
            Run::Panicked => panic!("{label}: panicked"),
            Run::TimedOut => panic!("{label}: still running after 60s (catastrophic backtracking)"),
        }
    }
    for r in &report {
        eprintln!("{r}");
    }
}

#[test]
fn every_rule_survives_a_keyword_flood_of_its_own_pattern_words_without_hanging() {
    // Words taken from each rule's pattern, repeated with the separators that defeat `[^.]*` (period) or feed it.
    let mut checked = 0;
    for rule in RULES {
        let words: Vec<&str> = rule
            .pattern
            .split(|c: char| !c.is_ascii_alphabetic())
            .chain(rule.negative.split(|c: char| !c.is_ascii_alphabetic()))
            .filter(|w| w.len() >= 3 && !matches!(*w, "the" | "and"))
            .collect();
        assert!(!words.is_empty(), "{}", rule.id);
        let joined = words.join(" ");
        for sep in [" ", ", ", ". ", ": ", "\n", " you control ", " target "] {
            let text = flood(&format!("{joined}{sep}"), 100_000);
            let id = rule.id;
            match timeboxed(60, move || {
                let i = index_of(&[rec("Test Subject", "Creature — Test", &text)]);
                clf().classify(&i.cards[0]).map(|a| a.roles.len())
            }) {
                Run::Done(Ok(_), d) => assert!(d < Duration::from_secs(20), "{id} sep {sep:?}: {d:?}"),
                Run::Done(Err(_), d) => assert!(d < Duration::from_secs(20), "{id} sep {sep:?}: {d:?}"),
                Run::Panicked => panic!("{id} sep {sep:?}: panic"),
                Run::TimedOut => panic!("{id} sep {sep:?}: > 60 s"),
            }
            checked += 1;
        }
    }
    assert_eq!(checked, RULES.len() * 7);
}

#[test]
fn defect_long_line_exhausts_the_regex_backtrack_limit_and_aborts_classification() {
    // classify() propagates any fancy_regex runtime error, and generate() propagates that, so one long line makes
    // `cardgen roles --rules` fail for the whole catalog. Measured thresholds on one line: ~2,000 bytes of one repeated
    // word, ~5,000 bytes of "Draw a card. " sentences, ~10,000 bytes of ordinary words. The longest real Oracle line
    // today is 645 bytes (Rules Lawyer), so the margin is 3x. A regex failure should degrade to needs_llm=true.
    for text in ["K".repeat(2_500), "opponent loses ".repeat(200), "Draw a card. ".repeat(500)] {
        let i = index_of(&[rec("Test Subject", "Creature — Test", &text)]);
        let r = clf().classify(&i.cards[0]);
        assert!(r.is_ok(), "{} bytes: {:?}", text.len(), r.err().map(|e| e.chars().take(110).collect::<String>()));
    }
}

// ------------------------------------------------------------------------------------------------ B. self names

#[test]
fn self_name_substitution_respects_word_boundaries_and_case() {
    let one = |name: &str, line: &str| {
        let i = index_of(&[rec(name, "Creature — Test", "")]);
        roles::normalize(&i.cards[0], line)
    };
    // short names that are also ordinary words, and names inside other words
    assert_eq!(one("Sol, the Sun", "Solemn Sol Sols Sol's Sol-Ring sol"), "Solemn ~ Sols ~'s ~-Ring sol");
    assert_eq!(one("Ally", "Ally Allyship Rally Ally."), "~ Allyship Rally ~.");
    assert_eq!(one("Time, Lord", "Time Warp: Time, Timelord Time."), "~ Warp: ~, Timelord ~.");
    // commas, apostrophes, accents, and the full name winning over the short name
    assert_eq!(one("Ghave, Guru of Spores", "Ghave, Guru of Spores and Ghave"), "~ and ~");
    assert_eq!(one("Óin the Brave", "Óin the Brave Oin the Brave"), "~ Oin the Brave");
    assert_eq!(one("K'rrik, Son of Yawgmoth", "K'rrik's K'rrik"), "~'s ~");
    assert_eq!(one("Urza's Saga", "Urza's Saga and Urza"), "~ and Urza");
    // names with regex metacharacters, underscores, digits and brackets are literal
    assert_eq!(one("A.B*C+? [x] ^$", "A.B*C+? [x] ^$ wins; AxBxC"), "~ wins; AxBxC");
    assert_eq!(one("_____ Goblin", "_____ Goblin gets +1/+1."), "~ gets +1/+1.");
    assert_eq!(one("R2-D2", "R2-D2 and R2-D2s"), "~ and R2-D2s");
    // a name that is a substring of its own text a hundred times over
    assert_eq!(one("aa", "aa aaaa aa"), "~ aaaa ~");
}

#[test]
fn double_faced_and_split_names_replace_every_face_name() {
    let v = multi(
        "Arlinn Kord // Arlinn, Embraced by the Moon",
        "transform",
        "Legendary Planeswalker — Arlinn // Legendary Planeswalker — Arlinn",
        vec![
            face(
                "Arlinn Kord",
                "Legendary Planeswalker — Arlinn",
                "0: Create a 2/2 green Wolf creature token. Transform Arlinn Kord.",
                Some("3"),
            ),
            face(
                "Arlinn, Embraced by the Moon",
                "Legendary Planeswalker — Arlinn",
                "−1: Arlinn deals 3 damage to any target. Transform Arlinn, Embraced by the Moon.",
                Some("3"),
            ),
        ],
    );
    let i = index_of(&[v]);
    let c = &i.cards[0];
    assert_eq!(
        roles::normalize(c, "Arlinn Kord / Arlinn, Embraced by the Moon / Arlinn"),
        "~ / ~ / ~",
        "full name, face names and the short name before the comma all become ~"
    );
    assert_eq!(
        roles::normalize(c, "Arlinn Kord // Arlinn, Embraced by the Moon"),
        "~",
        "the full card name is a self name too, and wins over its face names"
    );
}

#[test]
fn defect_verb_named_card_erases_its_own_effect() {
    // Real card: the self name `Exile` is also the first word of the sentence, so normalize() turns
    // "Exile target nonwhite attacking creature." into "~ target nonwhite attacking creature." and the removal vanishes
    // while the card still reports needs_llm=false (a silent miss).
    let i = audit_cards();
    let a = classify_named(&i, "Exile");
    assert!(has(&a, "removal") || a.needs_llm, "Exile (Exile target nonwhite attacking creature): {a:?}");
}

// ------------------------------------------------------------------------------------------------ C. planeswalkers

fn walker(loyalty: &str, lines: &[&str]) -> Assignment {
    classify_record(with(
        rec("Test Walker", "Legendary Planeswalker — Test", &lines.join("\n")),
        "loyalty",
        json!(loyalty),
    ))
}

#[test]
fn planeswalker_minus_ability_at_or_above_starting_loyalty_gives_no_role() {
    // each ability maps to a unique role so the result shows exactly which lines were read
    let a = walker(
        "4",
        &[
            "+1: Draw two cards.",
            "0: Create a 1/1 white Soldier creature token.",
            "−3: Counter target spell.",
            "−4: Destroy target creature.",
            "−5: Take an extra turn after this one.",
            "−8: Each opponent loses 5 life.",
        ],
    );
    assert_eq!(roles_sorted(&a), ["counter", "draw", "tokens"], "{a:?}");
    // ASCII hyphen-minus is accepted as a minus too
    let a = walker("3", &["-3: Destroy target creature.", "-2: Counter target spell."]);
    assert_eq!(roles_sorted(&a), ["counter"], "{a:?}");
    // a loyalty of 0 never makes a cost 0 line an ultimate ("0:" is not a minus), and -0 is
    let a = walker("0", &["0: Draw two cards.", "+1: Counter target spell."]);
    assert_eq!(roles_sorted(&a), ["counter", "draw"], "{a:?}");
}

#[test]
fn planeswalker_x_and_plus_costs_are_never_ultimates() {
    // -X costs are ordinary (27 real planeswalkers have one); +N and +X are not minus abilities at all
    let a = walker("2", &["−X: Destroy all creatures.", "+7: Draw two cards.", "+X: Counter target spell."]);
    assert_eq!(roles_sorted(&a), ["counter", "draw", "wipe"], "{a:?}");
}

#[test]
fn planeswalker_faces_use_their_own_loyalty() {
    let v = multi(
        "Front // Back",
        "transform",
        "Legendary Planeswalker — A // Legendary Planeswalker — B",
        vec![
            face("Front", "Legendary Planeswalker — A", "−3: Destroy target creature.", Some("3")),
            face(
                "Back",
                "Legendary Planeswalker — B",
                "−3: Counter target spell.\n−5: Take an extra turn after this one.",
                Some("5"),
            ),
        ],
    );
    let a = classify_record(v);
    // front -3 of 3 is an ultimate; back -3 of 5 counts; back -5 of 5 is an ultimate
    assert_eq!(roles_sorted(&a), ["counter"], "{a:?}");
    assert_eq!(a.evidence["counter"].face, Some(1));
}

#[test]
fn planeswalker_odd_loyalty_formats_do_not_panic() {
    for loyalty in ["", "X", "*", "1d4+1", "-1", "+3", "٣", "99999999999999999999", " 4 ", "4.5", "∞"] {
        let a = walker(loyalty, &["−3: Destroy target creature.", "−X: Draw a card.", "+1: Counter target spell."]);
        assert!(a.roles.len() <= 4);
        assert!(has(&a, "counter"), "{loyalty:?}: plus abilities are always read: {a:?}");
    }
    // no loyalty field at all: the ultimate cannot be recognised, but nothing panics
    let v = rec("No Loyalty", "Legendary Planeswalker — Test", "−7: Destroy target creature.");
    assert!(clf().classify(&index_of(&[v]).cards[0]).is_ok());
    // en dash and em dash are not minus signs in Oracle text; they must at least not crash or invent a cost
    let a = walker("3", &["–3: Destroy target creature."]);
    assert!(a.roles.len() <= 1);
}

// ------------------------------------------------------------------------------------------------ D. MDFC / prepare

#[test]
fn mdfc_land_back_adds_no_roles_but_a_spell_back_does() {
    let spell_land = multi(
        "Spell // Land",
        "modal_dfc",
        "Sorcery // Land",
        vec![
            face("Spell", "Sorcery", "Draw three cards.", None),
            face(
                "Land",
                "Land",
                concat!(
                    "{T}, Sacrifice this land: Destroy target creature.\n{T}: Add {W} or {U}.\n",
                    "Whenever you gain life, create a 1/1 creature token."
                ),
                None,
            ),
        ],
    );
    let a = classify_record(spell_land);
    assert_eq!(roles_sorted(&a), ["draw"], "land back contributes nothing: {a:?}");
    assert_eq!(a.evidence["draw"].face, Some(0));
    let spell_spell = multi(
        "Spell // Spell",
        "modal_dfc",
        "Sorcery // Instant",
        vec![
            face("Spell", "Sorcery", "Draw three cards.", None),
            face("Other", "Instant", "Counter target spell.", None),
        ],
    );
    assert_eq!(roles_sorted(&classify_record(spell_spell)), ["counter", "draw"]);
    // a non-modal layout with a Land second face (transform lands such as Westvale Abbey) is NOT exempt
    let transform = multi(
        "Front // Land",
        "transform",
        "Creature // Land",
        vec![
            face("Front", "Creature", "Flying", None),
            face("Land", "Land", "{T}, Sacrifice this land: Draw three cards.", None),
        ],
    );
    assert!(has(&classify_record(transform), "draw"));
    // two land faces: the back is skipped, the front still counts
    let pathway = multi(
        "A // B",
        "modal_dfc",
        "Land // Land",
        vec![
            face("A", "Land", "{T}: Add {R} or {G}. {T}: Add {R}{R}.", None),
            face("B", "Land", "{T}: Draw three cards.", None),
        ],
    );
    let a = classify_record(pathway);
    assert!(!has(&a, "draw"), "{a:?}");
}

#[test]
fn prepare_cards_use_only_the_spell_face_and_never_credit_the_copy_reminder() {
    let v = multi(
        "Wizard // Bolt",
        "prepare",
        "Creature — Wizard // Instant",
        vec![
            face(
                "Wizard",
                "Creature — Wizard",
                concat!(
                    "Whenever ~ attacks, draw a card.\nWhenever you cast your third spell each turn, ",
                    "this creature becomes prepared. (While it's prepared, you may cast a copy of its spell.)"
                ),
                None,
            ),
            face("Bolt", "Instant", "Bolt deals 3 damage to any target.", None),
        ],
    );
    let a = classify_record(v);
    assert!(has(&a, "removal"), "{a:?}");
    assert!(!has(&a, "draw"), "the creature body is not classified: {a:?}");
    assert_eq!(a.evidence["removal"].face, Some(1));
    let sorcery = multi(
        "Body // Spell",
        "prepare",
        "Creature // Sorcery",
        vec![face("Body", "Creature", "Flying", None), face("Spell", "Sorcery", "Each opponent loses 3 life.", None)],
    );
    assert!(has(&classify_record(sorcery), "drain"));
}

// ------------------------------------------------------------------------------------------------ E. exclusion

#[test]
fn stickers_and_attractions_are_not_deck_cards() {
    let i = index_of(&[
        rec("Sticker Sheet", "Stickers", "Draw a card."),
        rec("Ride", "Artifact — Attraction", "Draw a card."),
        rec("Ride Two", "Artifact Attraction", "Draw a card."),
        rec("Ordinary", "Artifact — Equipment", "Draw a card."),
        rec("Attractive", "Creature — Attraction Goblin", "Draw a card."),
        rec("Mentions", "Creature — Goblin", "Attraction Stickers Attraction"),
    ]);
    let flags: Vec<bool> = i.cards.iter().map(roles::is_deck_card).collect();
    assert_eq!(flags, [false, false, false, true, false, true]);
    let map = roles::generate(&i, "").unwrap();
    assert_eq!(map.len(), 2);
    assert!(map.values().all(|a| has(a, "draw") || a.roles.is_empty()));
}

#[test]
fn excluded_cards_resist_overrides_show_find_and_the_side_table() {
    let mut records = fixture_records("roles_oracle.json");
    records.extend(
        fixture_records("adversarial_roles/audit_cards.json")
            .into_iter()
            .filter(|c| matches!(c["name"].as_str(), Some("Spinny Ride" | "Unhinged Beast Hunt"))),
    );
    let mut seen = BTreeSet::new();
    records.retain(|c| seen.insert(c["oracle_id"].as_str().unwrap().to_string()));
    let e = Env::with_records(&records);
    let index = index_of(&records);
    let excluded: Vec<&str> =
        index.cards.iter().filter(|c| !roles::is_deck_card(c)).map(|c| c.raw.name.as_str()).collect();
    assert!(excluded.len() >= 2, "{excluded:?}");
    for name in excluded.iter().copied() {
        // overrides in every spelling of the name
        for spelling in [name.to_string(), name.to_uppercase(), name.to_lowercase()] {
            let text = format!("[[card]]\nname = {spelling:?}\nroles = []\nnote = 'bypass'\n");
            assert!(roles::generate(&index, &text).is_err(), "override for excluded {spelling}");
        }
    }
    e.rules();
    for name in excluded.iter().copied() {
        let shown = e.command(&["--show", name]).unwrap().0;
        assert!(shown.contains("excluded from roles"), "{shown}");
        for role in IDS {
            let json = e.find(&["--name", name, "--role", role, "--legal", "any", "--json"]).unwrap();
            assert_eq!(json.trim(), "[]", "{name} must never match --role {role}");
        }
        let row: Value = serde_json::from_str(&e.find(&["--name", name, "--legal", "any", "--json"]).unwrap()).unwrap();
        assert_eq!(row[0]["roles"], json!([]));
        assert_eq!(row[0]["primary"], "");
    }
    for role in IDS {
        let all = e.find(&["--role", role, "--legal", "any", "--limit", "all", "--json"]).unwrap();
        for name in excluded.iter().copied() {
            assert!(!all.contains(&format!("\"name\":{name:?}")), "{role} leaked {name}");
        }
    }
    // roles.json that smuggles a row for an excluded card is rejected on attach, even with the count balanced
    let mut map = roles::load(&e.kdir().join("roles.json")).unwrap();
    assert!(map.len() < index.cards.len());
    let smuggled = index.cards.iter().find(|c| !roles::is_deck_card(c)).unwrap().raw.oracle_id.clone();
    map.insert(smuggled, Assignment::default());
    let victim = index.cards.iter().find(|c| roles::is_deck_card(c)).unwrap().raw.oracle_id.clone();
    map.remove(&victim);
    let mut fresh = index_of(&records);
    assert!(fresh.attach_roles(&map).is_err());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_excluded_cards_have_no_rows_and_never_surface() {
    let idx = real_index();
    let excluded: Vec<_> = idx.cards.iter().filter(|c| !roles::is_deck_card(c)).collect();
    assert_eq!(excluded.len(), 99, "Stickers + Attractions in the 2026-11-21 catalog");
    for c in &excluded {
        assert!(c.raw.type_line.split_whitespace().any(|t| matches!(t, "Stickers" | "Attraction")), "{}", c.raw.name);
    }
    let map = real_map();
    for c in &excluded {
        assert!(!map.contains_key(&c.raw.oracle_id), "{}", c.raw.name);
    }
    // nothing deck-shaped is wrongly excluded: no excluded card has another type word besides sticker/attraction types
    for c in idx.cards.iter().filter(|c| roles::is_deck_card(c)) {
        assert!(map.contains_key(&c.raw.oracle_id), "{}", c.raw.name);
    }
    let mut attached = real_index_fresh();
    attached.attach_roles(map).unwrap();
    for role in IDS {
        let q = Query { roles: vec![role.to_string()], legal: knowledge::LegalFilter::Any, ..Query::default() };
        let (hits, _) = attached.search(&q, None).unwrap();
        for h in hits {
            assert!(
                roles::is_deck_card(&attached.cards[h as usize]),
                "{role}: {}",
                attached.cards[h as usize].raw.name
            );
        }
    }
}

// ------------------------------------------------------------------------------------------------ F. invariants

#[test]
fn fixture_assignments_satisfy_every_structural_invariant() {
    let i = roles_fixture();
    let mut bad = vec![];
    for c in &i.cards {
        let a = clf().classify(c).unwrap();
        bad.extend(problems(c, &a));
    }
    assert!(bad.is_empty(), "{bad:#?}");
    let i = audit_cards();
    for c in &i.cards {
        assert!(problems(c, &clf().classify(c).unwrap()).is_empty());
    }
}

#[test]
fn generation_is_byte_deterministic_and_independent_of_catalog_order() {
    let records = fixture_records("roles_oracle.json");
    let forward = roles::bytes(&roles::generate(&index_of(&records), "").unwrap()).unwrap();
    assert_eq!(forward, roles::bytes(&roles::generate(&index_of(&records), "").unwrap()).unwrap());
    let mut reversed = records.clone();
    reversed.reverse();
    assert_eq!(forward, roles::bytes(&roles::generate(&index_of(&reversed), "").unwrap()).unwrap(), "reversed");
    let mut rotated = records.clone();
    rotated.rotate_left(137);
    assert_eq!(forward, roles::bytes(&roles::generate(&index_of(&rotated), "").unwrap()).unwrap(), "rotated");
    // classification of one card does not depend on any other card: classify each alone
    let all = roles::generate(&index_of(&records), "").unwrap();
    for r in records.iter().step_by(7).filter(|r| all.contains_key(r["oracle_id"].as_str().unwrap())) {
        let alone = index_of(std::slice::from_ref(r));
        let a = clf().classify(&alone.cards[0]).unwrap();
        assert_eq!(&a, &all[r["oracle_id"].as_str().unwrap()], "{}", r["name"]);
    }
}

fn real_index() -> &'static Index {
    static I: OnceLock<Index> = OnceLock::new();
    I.get_or_init(|| Index::load(&repo_root().join("data/knowledge/oracle.json")).expect(NEEDS_CATALOG))
}
fn real_index_fresh() -> Index {
    Index::load(&repo_root().join("data/knowledge/oracle.json")).expect(NEEDS_CATALOG)
}
fn real_map() -> &'static roles::RoleMap {
    static M: OnceLock<roles::RoleMap> = OnceLock::new();
    M.get_or_init(|| roles::generate(real_index(), "").unwrap())
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_every_card_classifies_without_panic_and_holds_invariants() {
    let idx = real_index();
    assert!(idx.cards.len() > 35_000, "{}", idx.cards.len());
    let mut bad = vec![];
    let mut worst = (Duration::ZERO, String::new());
    let t0 = Instant::now();
    for c in &idx.cards {
        let t = Instant::now();
        let r = std::panic::catch_unwind(AssertUnwindSafe(|| clf().classify(c)));
        let el = t.elapsed();
        if el > worst.0 {
            worst = (el, c.raw.name.clone());
        }
        match r {
            Err(_) => bad.push(format!("{}: PANIC", c.raw.name)),
            Ok(Err(e)) => bad.push(format!("{}: {e}", c.raw.name)),
            Ok(Ok(a)) => {
                if roles::is_deck_card(c) {
                    bad.extend(problems(c, &a));
                }
            }
        }
    }
    eprintln!("classified {} cards in {:?}; slowest {:?} ({})", idx.cards.len(), t0.elapsed(), worst.0, worst.1);
    assert!(bad.is_empty(), "{} problems, first 25: {:#?}", bad.len(), &bad[..bad.len().min(25)]);
    assert!(worst.0 < Duration::from_secs(1), "slowest card {:?} took {:?}", worst.1, worst.0);
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_generation_is_independent_of_catalog_order() {
    let src: Value =
        serde_json::from_slice(&std::fs::read(repo_root().join("data/knowledge/oracle.json")).unwrap()).unwrap();
    let mut cards = src["cards"].as_array().unwrap().clone();
    cards.reverse();
    cards.rotate_left(12_345);
    let t = Tmp::new("shuffled");
    let p = t.0.join("oracle.json");
    let mut doc = src.clone();
    doc["cards"] = json!(cards);
    std::fs::write(&p, serde_json::to_vec(&doc).unwrap()).unwrap();
    let shuffled = Index::load(&p).unwrap();
    let a = roles::bytes(real_map()).unwrap();
    let b = roles::bytes(&roles::generate(&shuffled, "").unwrap()).unwrap();
    assert!(a == b, "role table depends on catalog order");
    // and a second pass over the same index is identical
    assert!(a == roles::bytes(&roles::generate(real_index(), "").unwrap()).unwrap());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_saved_roles_json_matches_a_fresh_generation() {
    let root = repo_root();
    let overrides = std::fs::read_to_string(root.join("data/knowledge/roles_overrides.toml")).unwrap();
    let fresh = roles::bytes(&roles::generate(real_index(), &overrides).unwrap()).unwrap();
    let saved = std::fs::read(root.join("data/knowledge/roles.json")).expect("run cardgen roles --rules");
    assert!(saved == fresh, "data/knowledge/roles.json is stale against the current classifier");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_evidence_never_comes_from_land_backs_prepare_bodies_or_ultimates() {
    let idx = real_index();
    let map = real_map();
    let mut bad = vec![];
    for c in idx.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let a = &map[&c.raw.oracle_id];
        let lines = c.lines();
        for (role, e) in &a.evidence {
            let Some(f) = e.face.and_then(|i| c.raw.card_faces.as_ref().and_then(|fs| fs.get(i))) else { continue };
            if c.raw.layout == "modal_dfc" && e.face.unwrap() > 0 && f.type_line.contains("Land") {
                bad.push(format!("{}: {role} cites an MDFC land back", c.raw.name));
            }
            if c.raw.layout == "prepare" && !f.type_line.contains("Instant") && !f.type_line.contains("Sorcery") {
                bad.push(format!("{}: {role} cites a prepare body", c.raw.name));
            }
        }
        for (role, e) in &a.evidence {
            let (face, line) = lines[e.line.unwrap()];
            let loyalty = face
                .and_then(|i| c.raw.card_faces.as_ref()?.get(i)?.loyalty.as_deref())
                .or(c.raw.loyalty.as_deref())
                .and_then(|l| l.parse::<u32>().ok());
            let cost =
                line.strip_prefix(['−', '-']).and_then(|s| s.split_once(':')).and_then(|(c, _)| c.parse::<u32>().ok());
            if let (Some(l), Some(cost)) = (loyalty, cost) {
                if cost >= l {
                    bad.push(format!("{}: {role} cites ultimate {line:?} (loyalty {l})", c.raw.name));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn real_catalog_routing_share_stays_under_the_stated_cap() {
    let s = roles::stats(real_index(), real_map());
    eprintln!("{}", s.text());
    let (all, llm) =
        s.routing.iter().filter(|(g, _)| g.starts_with("legal / ")).fold((0, 0), |a, (_, n)| (a.0 + n.0, a.1 + n.1));
    let share = llm as f64 / all as f64;
    assert!(share < 0.55, "legal routing share {share:.4} is above the 55% cap");
    assert_eq!(s.total + s.skipped, real_index().cards.len());
    assert_eq!(s.skipped, 99);
    // no card is silently dropped: roles are empty for a large share, and every role is used
    for (r, n) in &s.per_role {
        assert!(*n > 0, "role {r} is never assigned");
    }
}

// ------------------------------------------------------------------------------------------------ G. overrides

#[test]
fn malformed_override_files_are_errors_not_partial_tables() {
    let i = roles_fixture();
    for text in [
        "[[card]]\nname='Counterspell'\nroles=['draw'",
        "[[card]]\nname='Counterspell'\nroles=['draw']\n",
        "[[card]]\nname='Counterspell'\nroles=['draw']\nnote='   '\n",
        "[[card]]\nname='Counterspell'\nroles=['draw']\nnote=''\n",
        "[[card]]\nname='Counterspell'\nroles=['nope']\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nroles=['RAMP']\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nroles=['draw','draw']\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nroles=['ramp','draw','tokens','anthem','blink']\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nroles=['draw']\nprimary='draw'\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nroles='draw'\nnote='x'\n",
        "[[card]]\nroles=['draw']\nnote='x'\n",
        "[[card]]\nname='Counterspell'\nnote='x'\n",
        "[[card]]\nname='Not A Card In The Catalog'\nroles=[]\nnote='x'\n",
        "[[card]]\nname=''\nroles=[]\nnote='x'\n",
        concat!(
            "[[card]]\nname='Counterspell'\nroles=['draw']\nnote='x'\n",
            "[[card]]\nname='Counterspell'\nroles=['ramp']\nnote='y'\n"
        ),
        "[[card]]\nname='Counterspell'\nroles=['draw']\nnote='x'\n[[card]]\nname='COUNTERSPELL'\nroles=[]\nnote='y'\n",
        "stray = 1\n",
        "[card]\nname='Counterspell'\nroles=[]\nnote='x'\n",
    ] {
        assert!(roles::generate(&i, text).is_err(), "must reject: {text:?}");
    }
    // an empty file, comments only, and an empty [[card]]-less file are fine and change nothing
    let base = roles::bytes(&roles::generate(&i, "").unwrap()).unwrap();
    for ok in ["", "# comment only\n", "\n\n"] {
        assert_eq!(base, roles::bytes(&roles::generate(&i, ok).unwrap()).unwrap(), "{ok:?}");
    }
}

#[test]
fn override_names_resolve_folded_by_face_and_never_ambiguously() {
    let i = roles_fixture();
    let ids = |text: &str| roles::generate(&i, text);
    // case, accent and apostrophe folding reach the card; so does a face name of a split/modal card
    for name in ["counterspell", "COUNTERSPELL", "Ashnod’s Altar", "ashnod's altar", "Óin the Brave", "Oin the Brave"]
    {
        let text = format!("[[card]]\nname={name:?}\nroles=['wipe']\nnote='folded name'\n");
        let map = ids(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(map.values().filter(|a| a.evidence.values().any(|e| e.source == "override")).count(), 1);
    }
    // a face name resolves to the whole card; the exact full name beats a face name of another card
    let t = "[[card]]\nname='Agadeem, the Undercrypt'\nroles=['wipe']\nnote='face name'\n";
    let map = ids(t).unwrap();
    let mdfc = &i.cards[named(&i, "Agadeem's Awakening // Agadeem, the Undercrypt")].raw.oracle_id;
    assert_eq!(map[mdfc].roles, ["wipe"]);
    let t = "[[card]]\nname='Lightning Bolt'\nroles=['wipe']\nnote='full name beats the prepare face'\n";
    let map = ids(t).unwrap();
    let bolt = &i.cards[named(&i, "Lightning Bolt")].raw.oracle_id;
    let prepare = &i.cards[named(&i, "Emeritus of Conflict // Lightning Bolt")].raw.oracle_id;
    assert_eq!(map[bolt].roles, ["wipe"]);
    assert_ne!(map[prepare].roles, ["wipe"], "the prepare card must not be hit by its spell's name");
    // names shared by several cards cannot be overridden
    let shared = audit_cards();
    assert!(!shared.shared_names().is_empty(), "fixture must carry a shared name");
    let (name, _) = shared.shared_names()[0];
    let text = format!("[[card]]\nname={name:?}\nroles=[]\nnote='ambiguous'\n");
    let err = roles::generate(&shared, &text).unwrap_err();
    assert!(err.contains("resolves to 2 cards"), "{err}");
}

#[test]
fn override_replaces_everything_and_round_trips_through_the_side_table() {
    let e = Env::fixture();
    e.write(
        "roles_overrides.toml",
        "[[card]]\nname='Counterspell'\nroles=['finisher','stax','draw','ramp']\nnote='four unusual roles'\n\
         [[card]]\nname=\"Ashnod's Altar\"\nroles=[]\nnote='cleared'\n",
    );
    e.rules();
    let rows: Value = serde_json::from_str(&e.find(&["--name", "Counterspell", "--json"]).unwrap()).unwrap();
    assert_eq!(rows[0]["roles"], json!(["finisher", "stax", "draw", "ramp"]));
    assert_eq!(rows[0]["primary"], "finisher");
    let altar: Value = serde_json::from_str(&e.find(&["--name", "Ashnod's Altar", "--json"]).unwrap()).unwrap();
    assert_eq!(altar[0]["roles"], json!([]));
    assert_eq!(altar[0]["primary"], "");
    let shown = e.command(&["--show", "Counterspell"]).unwrap().0;
    assert!(shown.contains("needs_llm: false") && shown.contains("override"), "{shown}");
    // overrides never improve validation: score against the rules, not the override
    e.set_deck("Counterspell", &["Counterspell"], &[("Counterspell", json!({"ctr": "any"}))]);
    let fx = roles_fixture();
    let id = fx.cards[named(&fx, "Counterspell")].raw.oracle_id.clone();
    let gold = e.0.join("gold.toml");
    std::fs::write(
        &gold,
        format!("[[card]]\nname='Counterspell'\noracle_id='{id}'\nroles=['counter']\nprimary='counter'\n"),
    )
    .unwrap();
    let (out, passed) = e.command(&["--validate", "--gold", gold.to_str().unwrap(), "--json"]).unwrap();
    assert!(passed, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["hand_micro"], json!({"tp": 1, "fp": 0, "missed": 0}));
}

#[test]
fn failed_generation_leaves_the_existing_role_table_untouched_and_no_temp_files() {
    let e = Env::fixture();
    e.rules();
    let before = e.read("roles.json");
    for bad in [
        "[[card]]\nname='Nope'\nroles=[]\nnote='x'\n",
        "not toml [",
        "[[card]]\nname='Counterspell'\nroles=['zzz']\nnote='x'\n",
    ] {
        e.write("roles_overrides.toml", bad);
        assert!(e.command(&["--rules"]).is_err());
        assert!(before == e.read("roles.json"), "roles.json changed after a failed run");
    }
    e.write("roles_overrides.toml", "");
    e.rules();
    let tmp: Vec<_> = std::fs::read_dir(e.kdir())
        .unwrap()
        .map(|f| f.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(tmp.is_empty(), "{tmp:?}");
    assert!(before == e.read("roles.json"));
}

// ------------------------------------------------------------------------------------------------ H. taxonomy

fn without_role(id: &str) -> String {
    let parts: Vec<&str> = TAXONOMY.split("\n[[role]]\n").collect();
    let kept: Vec<&str> = parts.iter().copied().filter(|p| !p.starts_with(&format!("id = \"{id}\""))).collect();
    assert_eq!(kept.len(), parts.len() - 1, "{id}");
    kept.join("\n[[role]]\n")
}

#[test]
fn taxonomy_parse_rejects_missing_duplicate_and_inconsistent_roles() {
    assert!(Taxonomy::parse(TAXONOMY).is_ok());
    for id in IDS {
        assert!(Taxonomy::parse(&without_role(id)).is_err(), "role {id} removed");
    }
    let dup = TAXONOMY.replacen("id = \"fixing\"", "id = \"ramp\"", 1);
    assert!(Taxonomy::parse(&dup).is_err(), "duplicate id");
    let extra = format!("{TAXONOMY}\n[[role]]\nid = \"ramp\"\ndefinition = \"x\"\nsrc = \"rules\"\nguard = \"x\"\n");
    assert!(Taxonomy::parse(&extra).is_err(), "27 roles");
    let unknown = TAXONOMY.replacen("id = \"ramp\"", "id = \"ramp2\"", 1);
    assert!(Taxonomy::parse(&unknown).is_err(), "unknown id");
    // a changed source (the classifier and the taxonomy must agree on who owns a role)
    assert!(Taxonomy::parse(&TAXONOMY.replacen("src = \"rules\"", "src = \"llm\"", 1)).is_err());
    assert!(Taxonomy::parse(&TAXONOMY.replacen("src = \"llm\"", "src = \"rules\"", 1)).is_err());
    assert!(Taxonomy::parse(&TAXONOMY.replacen("src = \"rules+llm\"", "src = \"rules\"", 1)).is_err());
    // an unparsable guard, an empty definition, a missing field, wrong types, no version
    assert!(
        Taxonomy::parse(&TAXONOMY.replacen("guard = '(?i)add|mana of any", "guard = '(?i)(add|mana of any", 1))
            .is_err()
    );
    assert!(
        Taxonomy::parse(&TAXONOMY.replacen(
            "(?i)add|mana of any|search .*library|landcycling|enchanted land is",
            "(",
            1
        ))
        .is_err()
    );
    assert!(Taxonomy::parse(&TAXONOMY.replace("version = 1", "")).is_err());
    assert!(Taxonomy::parse(&TAXONOMY.replace("version = 1", "version = \"1\"")).is_err());
    assert!(Taxonomy::parse("").is_err());
    assert!(Taxonomy::parse("version = 1\nrole = []\n").is_err());
    assert!(Taxonomy::parse(&TAXONOMY.replacen("guard = ", "guardx = ", 1)).is_err());
    // empty definition
    let parts: Vec<&str> = TAXONOMY.split("\n[[role]]\n").collect();
    let emptied: Vec<String> = parts
        .iter()
        .map(|p| {
            if p.starts_with("id = \"extra-phase\"") {
                "id = \"extra-phase\"\ndefinition = \"\"\nsrc = \"rules\"\nguard = \"x\"\n".to_string()
            } else {
                p.to_string()
            }
        })
        .collect();
    assert!(Taxonomy::parse(&emptied.join("\n[[role]]\n")).is_err(), "empty definition");
}

#[test]
fn a_broken_taxonomy_stops_every_roles_command() {
    let e = Env::fixture();
    e.rules();
    let (no_ramp, v2) = (without_role("ramp"), TAXONOMY.replace("version = 1", "version = 2"));
    for bad in ["", "garbage {", no_ramp.as_str(), v2.as_str()] {
        e.write("roles.toml", bad);
        for args in
            [&["--rules"][..], &["--stats"], &["--show", "Counterspell"], &["--validate"], &["--audit", "--n", "3"]]
        {
            assert!(e.command(args).is_err(), "{args:?} with a broken roles.toml");
        }
    }
    let missing = e.kdir().join("roles.toml");
    std::fs::remove_file(&missing).unwrap();
    assert!(e.command(&["--stats"]).is_err());
}

// ------------------------------------------------------------------------------------------------ I. roles.json

fn good_map() -> (Index, roles::RoleMap) {
    let i = roles_fixture();
    let m = roles::generate(&i, "").unwrap();
    (i, m)
}
fn first_with_roles(i: &Index, m: &roles::RoleMap, at_least: usize) -> (usize, String) {
    let (n, c) = i
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| roles::is_deck_card(c) && m[&c.raw.oracle_id].roles.len() >= at_least)
        .unwrap();
    (n, c.raw.oracle_id.clone())
}

#[test]
fn hand_edited_roles_tables_are_rejected_by_attach() {
    let (i, map) = good_map();
    let (n, id) = first_with_roles(&i, &map, 2);
    let c = &i.cards[n];
    let mut tests: Vec<(&str, Box<dyn Fn(&mut roles::RoleMap)>)> = vec![];
    let id1 = id.clone();
    tests.push((
        "row removed",
        Box::new(move |m| {
            m.remove(&id1);
        }),
    ));
    tests.push((
        "unknown oracle id added",
        Box::new(|m| {
            m.insert("ffffffff-0000-0000-0000-000000000000".into(), Assignment::default());
        }),
    ));
    let id2 = id.clone();
    tests.push((
        "five roles",
        Box::new(move |m| {
            let a = m.get_mut(&id2).unwrap();
            a.roles = vec!["ramp", "fixing", "draw", "tokens", "anthem"].into_iter().map(String::from).collect();
        }),
    ));
    let id3 = id.clone();
    tests.push((
        "primary mismatch",
        Box::new(move |m| {
            m.get_mut(&id3).unwrap().primary = "zzz".into();
        }),
    ));
    let id4 = id.clone();
    tests.push((
        "primary dropped",
        Box::new(move |m| {
            m.get_mut(&id4).unwrap().primary.clear();
        }),
    ));
    let id5 = id.clone();
    tests.push((
        "evidence removed",
        Box::new(move |m| {
            let a = m.get_mut(&id5).unwrap();
            let k = a.roles[0].clone();
            a.evidence.remove(&k);
        }),
    ));
    let id6 = id.clone();
    tests.push((
        "extra evidence key",
        Box::new(move |m| {
            m.get_mut(&id6).unwrap().evidence.insert(
                "ghost".into(),
                Evidence { source: "rule".into(), certain: false, rule: "mana".into(), face: None, line: Some(0) },
            );
        }),
    ));
    let id7 = id.clone();
    tests.push((
        "line out of range",
        Box::new(move |m| {
            let a = m.get_mut(&id7).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().line = Some(9999);
        }),
    ));
    let id8 = id.clone();
    tests.push((
        "line missing",
        Box::new(move |m| {
            let a = m.get_mut(&id8).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().line = None;
        }),
    ));
    let id9 = id.clone();
    tests.push((
        "face wrong",
        Box::new(move |m| {
            let a = m.get_mut(&id9).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().face = Some(3);
        }),
    ));
    let id10 = id.clone();
    tests.push((
        "rule of another role",
        Box::new(move |m| {
            let a = m.get_mut(&id10).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().rule = "restriction".into();
        }),
    ));
    let id11 = id.clone();
    tests.push((
        "unknown rule",
        Box::new(move |m| {
            let a = m.get_mut(&id11).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().rule = "nope".into();
        }),
    ));
    let id12 = id.clone();
    tests.push((
        "source llm",
        Box::new(move |m| {
            let a = m.get_mut(&id12).unwrap();
            let k = a.roles[0].clone();
            a.evidence.get_mut(&k).unwrap().source = "llm".into();
        }),
    ));
    let id13 = id.clone();
    tests.push((
        "duplicate role",
        Box::new(move |m| {
            let a = m.get_mut(&id13).unwrap();
            let r = a.roles[0].clone();
            a.roles.push(r);
        }),
    ));
    let id14 = id.clone();
    tests.push((
        "role id invented",
        Box::new(move |m| {
            let a = m.get_mut(&id14).unwrap();
            a.roles[0] = "ramp2".into();
            a.primary = "ramp2".into();
        }),
    ));
    let id15 = id.clone();
    tests.push((
        "override with a line",
        Box::new(move |m| {
            let a = m.get_mut(&id15).unwrap();
            let k = a.roles[0].clone();
            let e = a.evidence.get_mut(&k).unwrap();
            e.source = "override".into();
        }),
    ));
    let id16 = id.clone();
    tests.push((
        "override marked certain",
        Box::new(move |m| {
            let a = m.get_mut(&id16).unwrap();
            let k = a.roles[0].clone();
            let e = a.evidence.get_mut(&k).unwrap();
            e.source = "override".into();
            e.line = None;
            e.face = None;
            e.certain = true;
        }),
    ));
    for (label, edit) in tests {
        let mut m = map.clone();
        edit(&mut m);
        let mut fresh = roles_fixture();
        assert!(fresh.attach_roles(&m).is_err(), "{label} on {} must be rejected", c.raw.name);
    }
    // a well-formed hand override (source override, no line, no face) is accepted
    let mut m = map.clone();
    let a = m.get_mut(&id).unwrap();
    for e in a.evidence.values_mut() {
        *e = Evidence { source: "override".into(), certain: false, rule: "by hand".into(), face: None, line: None };
    }
    assert!(roles_fixture().attach_roles(&m).is_ok());
}

#[test]
fn damaged_roles_json_files_give_clean_errors_in_every_consumer() {
    let e = Env::fixture();
    e.rules();
    let good = e.read("roles.json");
    let mut truncated = good.clone();
    truncated.truncate(good.len() / 2);
    let mut bad_utf8 = good.clone();
    bad_utf8[10] = 0xff;
    let mut extra_field = String::from_utf8(good.clone()).unwrap();
    extra_field = extra_field.replacen("\"needs_llm\"", "\"surprise\": 1,\n    \"needs_llm\"", 1);
    for (label, bytes) in [
        ("empty", b"".to_vec()),
        ("null", b"null\n".to_vec()),
        ("array", b"[]\n".to_vec()),
        ("empty object", b"{}\n".to_vec()),
        ("truncated", truncated),
        ("invalid utf-8", bad_utf8),
        ("unknown field", extra_field.into_bytes()),
        ("text", b"not json".to_vec()),
    ] {
        std::fs::write(e.kdir().join("roles.json"), &bytes).unwrap();
        for args in [&["--stats"][..], &["--show", "Counterspell"], &["--validate"]] {
            assert!(e.command(args).is_err(), "{label}: roles {args:?} must fail cleanly");
        }
        assert!(e.find(&["--name", "Counterspell"]).is_err(), "{label}: find reads the side table when present");
    }
    // restoring the file restores every consumer; --rules repairs a damaged file
    std::fs::write(e.kdir().join("roles.json"), b"garbage").unwrap();
    e.rules();
    assert!(good == e.read("roles.json"));
    assert!(e.find(&["--role", "counter"]).is_ok());
}

#[test]
fn defect_stale_oracle_text_is_not_detected_by_attach_roles() {
    // roles.json is keyed by oracle id and checked only for shape, so a catalog whose text changed (same ids, same
    // card count) silently keeps the old roles: --role counter returns a card whose text now says "Draw a card."
    let mut records = fixture_records("roles_oracle.json");
    let map = roles::generate(&index_of(&records), "").unwrap();
    for r in records.iter_mut().filter(|r| r["name"] == "Counterspell") {
        r["oracle_text"] = json!("Draw a card. Draw another card. Draw a third card.");
    }
    let mut changed = index_of(&records);
    let verdict = changed.attach_roles(&map);
    assert!(verdict.is_err(), "a stale table attached to different Oracle text without complaint");
}

#[test]
fn defect_stale_roles_json_breaks_plain_find_without_role_filters() {
    // After bulk-sync adds a card, `find --name X` (no --role anywhere) fails until `roles --rules` is rerun.
    let e = Env::fixture();
    e.rules();
    assert!(e.find(&["--name", "Counterspell"]).is_ok());
    let mut records = fixture_records("roles_oracle.json");
    records.push(rec("Brand New Card", "Instant", "Draw two cards."));
    write_catalog(&e.kdir().join("oracle.json"), &records);
    let found = e.find(&["--name", "Counterspell"]);
    assert!(found.is_ok(), "a stale derived roles.json must not take plain find down: {found:?}");
}

// ------------------------------------------------------------------------------------------------ J. find --role

fn names(json_out: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(json_out).unwrap();
    v.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect()
}

#[test]
fn find_role_combines_with_every_other_flag() {
    let e = Env::fixture();
    e.rules();
    let base = names(&e.find(&["--role", "ramp", "--json", "--limit", "all", "--legal", "any"]).unwrap());
    assert!(base.len() > 5, "{base:?}");
    let sub = |args: &[&str]| -> Vec<String> {
        let mut a = vec!["--role", "ramp", "--json"];
        if !args.contains(&"--limit") {
            a.extend(["--limit", "all"]);
        }
        a.extend_from_slice(args);
        let got = names(&e.find(&a).unwrap_or_else(|err| panic!("{args:?}: {err}")));
        for n in &got {
            assert!(base.contains(n), "{args:?} returned {n}, which is not a ramp card");
        }
        got
    };
    assert_eq!(sub(&["--role", "ramp"]).len(), sub(&[]).len(), "repeating a role changes nothing");
    assert!(sub(&["--name", "Ashnod's Altar"]) == ["Ashnod's Altar"]);
    assert!(sub(&["--name", "Counterspell"]).is_empty());
    assert!(sub(&["--name-contains", "altar"]).iter().all(|n| n.to_lowercase().contains("altar")));
    sub(&["--ident", "C"]);
    sub(&["--ident", "WUBRG"]);
    sub(&["--legal", "banned"]);
    sub(&["--legal", "any"]);
    sub(&["--cmc", "<=2"]);
    sub(&["--cmc", ">=4"]);
    assert!(!sub(&["--type", "artifact"]).is_empty());
    assert!(sub(&["--type", "artifact", "--type", "creature"]).len() <= sub(&["--type", "artifact"]).len());
    sub(&["--text", "add"]);
    assert!(sub(&["--text", "zzzzzz-no-such-text"]).is_empty());
    sub(&["--gc"]);
    sub(&["--meld"]);
    for sort in ["edhrec", "name", "cmc", "usd"] {
        sub(&["--sort", sort]);
    }
    assert!(sub(&["--limit", "1"]).len() <= 1);
    sub(&["--offset", "1", "--limit", "2"]);
    sub(&["--deck", "test"]);
    // --count agrees with the listing, and --full / table forms run
    let count: usize = e.find(&["--role", "ramp", "--count", "--legal", "any"]).unwrap().trim().parse().unwrap();
    assert_eq!(count, base.len());
    assert!(e.find(&["--role", "ramp", "--full", "--limit", "3"]).is_ok());
    assert!(e.find(&["--role", "ramp", "--role", "sac-outlet", "--limit", "all"]).unwrap().contains("Ashnod's Altar"));
    // roles AND: a card with ramp must not appear under a role it lacks
    let both =
        names(&e.find(&["--role", "ramp", "--role", "counter", "--json", "--limit", "all", "--legal", "any"]).unwrap());
    let counters = names(&e.find(&["--role", "counter", "--json", "--limit", "all", "--legal", "any"]).unwrap());
    assert!(both.iter().all(|n| base.contains(n) && counters.contains(n)));
    assert!(both.contains(&"Mana Drain".to_string()), "{both:?}");
    // every flag is still usable on its own after a role query (no state leaks)
    assert!(e.find(&["--role", "ramp", "--data-dir", "/nonexistent"]).is_err());
}

#[test]
fn find_role_rejects_bad_values_as_usage_errors() {
    let e = Env::fixture();
    e.rules();
    for bad in [
        "",
        "RAMP",
        "Ramp",
        " ramp",
        "ramp ",
        "gy_hate",
        "sac_outlet",
        "extra_phase",
        "self mill",
        "ramp,draw",
        "none",
        "0",
        "all",
        "--json",
        "-1",
        "ramp\n",
    ] {
        let err = e.find(&["--role", bad]);
        assert!(matches!(err, Err(FindError::Usage(_))), "{bad:?}: {err:?}");
    }
    for args in [&["--role"][..], &["--role", "--json"], &["--role=ramp"], &["--roles", "ramp"], &["role", "ramp"]] {
        assert!(matches!(e.find(args), Err(FindError::Usage(_))), "{args:?}");
    }
    // the message lists every id, and a missing roles.json names the command that creates it
    let msg = e.find(&["--role", "x"]).unwrap_err().to_string();
    for id in IDS {
        assert!(msg.contains(id), "{id} missing from {msg}");
    }
    std::fs::remove_file(e.kdir().join("roles.json")).unwrap();
    let msg = e.find(&["--role", "ramp"]).unwrap_err().to_string();
    assert!(msg.contains("roles --rules"), "{msg}");
}

#[test]
fn find_json_rows_match_the_side_table_exactly() {
    let e = Env::fixture();
    e.rules();
    let all: Value =
        serde_json::from_str(&e.find(&["--json", "--limit", "all", "--legal", "any", "--meld"]).unwrap()).unwrap();
    let map: roles::RoleMap = roles::load(&e.kdir().join("roles.json")).unwrap();
    assert_eq!(all.as_array().unwrap().len(), 364);
    for row in all.as_array().unwrap() {
        let id = row["oracle_id"].as_str().unwrap();
        match map.get(id) {
            Some(a) => {
                assert_eq!(row["roles"], json!(a.roles), "{}", row["name"]);
                assert_eq!(row["primary"], json!(a.primary));
            }
            None => assert_eq!(row["roles"], json!([])),
        }
    }
}

// ------------------------------------------------------------------------------------------------ K. validation

fn crafted_validation_env() -> (Env, Vec<Value>) {
    let records = vec![
        rec("Alpha Counter", "Instant", "Counter target spell."),
        rec("Beta Extra", "Sorcery", "Target player takes an extra turn after this one."),
        rec("Gamma Vanilla", "Creature — Test", ""),
        rec("Delta Draw", "Sorcery", "Draw two cards."),
        rec("Outsider", "Sorcery", "Search your library for a card, put it into your hand, then shuffle."),
    ];
    let e = Env::with_records(&records);
    e.set_deck(
        "Alpha Counter",
        &["Alpha Counter", "Beta Extra", "Gamma Vanilla", "Delta Draw"],
        &[
            ("Alpha Counter", json!({"ctr": "any"})),
            ("Delta Draw", json!({"draw": true})),
            ("Gamma Vanilla", json!({"rem": true})),
            ("Outsider", json!({"tut": true})), // not in the deck: must not count
        ],
    );
    (e, records)
}
fn gold_toml(records: &[Value], cards: &[(&str, &[&str], &str, &[&str])]) -> String {
    let mut out = String::new();
    for (name, roles, primary, not_roles) in cards {
        let id =
            records.iter().find(|r| r["name"] == *name).map(|r| r["oracle_id"].as_str().unwrap().to_string()).unwrap();
        let list = |v: &[&str]| v.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(",");
        out.push_str(&format!(
            "[[card]]\nname='{name}'\noracle_id='{id}'\nroles=[{}]\nprimary='{primary}'\nnot_roles=[{}]\n",
            list(roles),
            list(not_roles)
        ));
    }
    out
}
fn validate_json(e: &Env, gold: &str) -> (Value, bool) {
    let p = e.0.join("gold-under-test.toml");
    std::fs::write(&p, gold).unwrap();
    let (out, passed) = e.command(&["--validate", "--gold", p.to_str().unwrap(), "--json"]).unwrap();
    (serde_json::from_str(&out).unwrap(), passed)
}
fn score(tp: u64, fp: u64, missed: u64) -> Value {
    json!({"tp": tp, "fp": fp, "missed": missed})
}

#[test]
fn validation_arithmetic_matches_a_hand_computation() {
    let (e, records) = crafted_validation_env();
    e.rules();
    let rules = roles::generate(&index_of(&records), "").unwrap();
    // sanity: the crafted cards classify as designed, all rules-final
    let id = |n: &str| records.iter().find(|r| r["name"] == n).unwrap()["oracle_id"].as_str().unwrap().to_string();
    assert_eq!(rules[&id("Alpha Counter")].roles, ["counter"]);
    assert_eq!(rules[&id("Beta Extra")].roles, ["extra-phase"]);
    assert_eq!(rules[&id("Gamma Vanilla")].roles, Vec::<String>::new());
    assert_eq!(rules[&id("Delta Draw")].roles, ["draw"]);
    assert!(rules.values().all(|a| !a.needs_llm));
    assert!(rules[&id("Alpha Counter")].evidence["counter"].certain);
    assert!(rules[&id("Delta Draw")].evidence["draw"].certain);
    assert!(!rules[&id("Beta Extra")].evidence["extra-phase"].certain);

    let gold = gold_toml(
        &records,
        &[
            ("Alpha Counter", &["counter", "draw"], "counter", &["removal"]),
            ("Beta Extra", &[], "", &["counter", "extra-phase"]),
            ("Gamma Vanilla", &[], "", &[]),
            ("Delta Draw", &["draw"], "draw", &[]),
        ],
    );
    let (v, passed) = validate_json(&e, &gold);
    assert!(!passed);
    // per role, all predictions
    assert_eq!(v["gold_all"]["counter"], score(1, 0, 0));
    assert_eq!(v["gold_all"]["draw"], score(1, 0, 1));
    assert_eq!(v["gold_all"]["extra-phase"], score(0, 1, 0));
    assert_eq!(v["gold_all"]["ramp"], score(0, 0, 0));
    // certain predictions only: extra-phase is not certain so it is neither a fp nor a tp
    assert_eq!(v["gold_certain"]["counter"], score(1, 0, 0));
    assert_eq!(v["gold_certain"]["draw"], score(1, 0, 1));
    assert_eq!(v["gold_certain"]["extra-phase"], score(0, 0, 0));
    // micro sums: counter, draw and extra-phase are all `rules` roles
    assert_eq!(v["gold_rules_micro"], score(2, 1, 1));
    assert_eq!(v["gold_other_micro"], score(0, 0, 0));
    assert_eq!(v["certain_micro"], score(2, 0, 1));
    assert_eq!(v["rules_final_cards"], 4);
    assert_eq!(v["rules_final_micro"], score(2, 1, 1));
    assert_eq!(v["not_roles"], json!(["Beta Extra -> extra-phase"]));
    assert_eq!(v["certain_not_roles"], json!([]));
    assert_eq!(
        v["gold_differences"],
        json!(["Alpha Counter -> draw: missing", "Beta Extra -> extra-phase: unexpected"])
    );
    // hand tags: counter (Alpha) hit, draw (Delta) hit, removal (Gamma) missed; Outsider is not in a deck
    assert_eq!(v["hand"]["counter"], score(1, 0, 0));
    assert_eq!(v["hand"]["draw"], score(1, 0, 0));
    assert_eq!(v["hand"]["removal"], score(0, 0, 1));
    assert!(v["hand"].get("tutor").is_none(), "tags of a card outside every deck must not count");
    assert_eq!(v["hand_micro"], score(2, 0, 1));
    assert_eq!(v["hand_misses"], json!(["Gamma Vanilla -> removal"]));
    assert_eq!(
        v["failures"],
        json!([
            "hand micro recall < 0.95",
            "gold rules precision < 0.90",
            "gold rules recall < 0.85",
            "rules-final precision < 0.93",
            "rules-final recall < 0.85"
        ])
    );
    // the text report agrees with the JSON
    let p = e.0.join("gold-under-test.toml");
    let (text, passed) = e.command(&["--validate", "--gold", p.to_str().unwrap()]).unwrap();
    assert!(!passed && text.contains("FAIL: hand micro recall < 0.95; gold rules precision < 0.90"), "{text}");
    assert!(text.contains("RULES-FINAL SUBSET (4 cards; all roles)"), "{text}");
    assert!(text.contains("not_roles violations: 1 all, 0 certain"), "{text}");
}

#[test]
fn validation_a_certain_forbidden_role_fails_the_run_and_a_clean_gold_passes() {
    let (e, records) = crafted_validation_env();
    e.rules();
    // Alpha's certain counter is forbidden: certain_not_roles fires and the failure is named
    let gold = gold_toml(
        &records,
        &[
            ("Alpha Counter", &[], "", &["counter"]),
            ("Beta Extra", &["extra-phase"], "extra-phase", &[]),
            ("Delta Draw", &["draw"], "draw", &[]),
        ],
    );
    let (v, passed) = validate_json(&e, &gold);
    assert!(!passed);
    assert_eq!(v["certain_not_roles"], json!(["Alpha Counter -> counter"]));
    assert!(v["failures"].as_array().unwrap().contains(&json!("certain not_roles violations")));
    // a perfect gold set with a passing hand section passes with an empty failure list
    e.set_deck(
        "Alpha Counter",
        &["Alpha Counter", "Beta Extra", "Delta Draw"],
        &[("Alpha Counter", json!({"ctr": "any"})), ("Delta Draw", json!({"draw": true}))],
    );
    let gold = gold_toml(
        &records,
        &[
            ("Alpha Counter", &["counter"], "counter", &[]),
            ("Beta Extra", &["extra-phase"], "extra-phase", &[]),
            ("Delta Draw", &["draw"], "draw", &[]),
        ],
    );
    let (v, passed) = validate_json(&e, &gold);
    assert!(passed, "{v}");
    assert_eq!(v["failures"], json!([]));
    assert_eq!(v["gold_rules_micro"], score(3, 0, 0));
    assert_eq!(v["rules_final_micro"], score(3, 0, 0));
}

fn boundary_env(with_text: usize, vanilla: usize, expected_in_gold: usize) -> (Env, String) {
    let mut records = vec![];
    for n in 0..with_text {
        records.push(rec(&format!("Extra {n:02}"), "Sorcery", "Target player takes an extra turn after this one."));
    }
    for n in 0..vanilla {
        records.push(rec(&format!("Plain {n:02}"), "Creature — Test", ""));
    }
    let e = Env::with_records(&records);
    e.set_deck("Extra 00", &["Extra 00"], &[("Extra 00", json!({"assault": true}))]);
    e.rules();
    let mut gold = String::new();
    for r in records.iter().take(expected_in_gold.max(0)) {
        let id = r["oracle_id"].as_str().unwrap();
        gold.push_str(&format!(
            "[[card]]\nname='{}'\noracle_id='{id}'\nroles=['extra-phase']\nprimary='extra-phase'\n",
            r["name"].as_str().unwrap()
        ));
    }
    for r in records.iter().skip(expected_in_gold) {
        let id = r["oracle_id"].as_str().unwrap();
        gold.push_str(&format!(
            "[[card]]\nname='{}'\noracle_id='{id}'\nroles=[]\nprimary=''\n",
            r["name"].as_str().unwrap()
        ));
    }
    (e, gold)
}

#[test]
fn validation_thresholds_are_inclusive_at_exactly_point_nine_and_point_eight_five() {
    let fails = |v: &Value, what: &str| v["failures"].as_array().unwrap().iter().any(|f| f == what);
    // precision 9/10 = 0.90 exactly: 10 extra-turn cards, gold expects the role on 9 of them
    let (e, gold) = boundary_env(10, 0, 9);
    let (v, _) = validate_json(&e, &gold);
    assert_eq!(v["gold_rules_micro"], score(9, 1, 0));
    assert!(!fails(&v, "gold rules precision < 0.90"), "{v}");
    assert!(fails(&v, "rules-final precision < 0.93"), "{v}");
    // 9/11 < 0.90
    let (e, gold) = boundary_env(11, 0, 9);
    let (v, _) = validate_json(&e, &gold);
    assert!(fails(&v, "gold rules precision < 0.90"), "{v}");
    // recall 17/20 = 0.85 exactly: gold expects the role on 20 cards, 17 have the text
    let (e, gold) = boundary_env(17, 3, 20);
    assert!(gold.matches("[[card]]").count() == 20);
    let (v, _) = validate_json(&e, &gold);
    assert_eq!(v["gold_rules_micro"], score(17, 0, 3));
    assert!(!fails(&v, "gold rules recall < 0.85"), "{v}");
    assert!(!fails(&v, "rules-final recall < 0.85"), "{v}");
    // 16/20 fails
    let (e, gold) = boundary_env(16, 4, 20);
    let (v, passed) = validate_json(&e, &gold);
    assert!(!passed && fails(&v, "gold rules recall < 0.85") && fails(&v, "rules-final recall < 0.85"), "{v}");
}

#[test]
fn validation_rejects_every_malformed_gold_file_with_an_error() {
    let (e, records) = crafted_validation_env();
    e.rules();
    let id = records[0]["oracle_id"].as_str().unwrap().to_string();
    let card = |extra: &str| format!("[[card]]\nname='Alpha Counter'\noracle_id='{id}'\n{extra}");
    let ok = card("roles=['counter']\nprimary='counter'\n");
    let attempts: Vec<(&str, String)> = vec![
        ("empty file", "".into()),
        ("empty card list", "card=[]\n".into()),
        ("not toml", "[[card".into()),
        ("duplicate id", format!("{ok}{ok}")),
        ("unknown role", card("roles=['bogus']\nprimary='bogus'\n")),
        ("unknown not_role", card("roles=[]\nprimary=''\nnot_roles=['bogus']\n")),
        ("role also forbidden", card("roles=['counter']\nprimary='counter'\nnot_roles=['counter']\n")),
        ("primary is not first", card("roles=['counter','draw']\nprimary='draw'\n")),
        ("primary with no roles", card("roles=[]\nprimary='counter'\n")),
        ("roles with empty primary", card("roles=['counter']\nprimary=''\n")),
        ("five roles", card("roles=['ramp','draw','tokens','anthem','blink']\nprimary='ramp'\n")),
        ("unknown oracle id", ok.replace(&id, "ffffffff-ffff-4fff-8fff-ffffffffffff")),
        ("missing roles key", card("primary='counter'\n")),
        ("missing primary key", card("roles=['counter']\n")),
        ("roles wrong type", card("roles='counter'\nprimary='counter'\n")),
    ];
    for (label, gold) in attempts {
        let p = e.0.join("gold.toml");
        std::fs::write(&p, gold).unwrap();
        let r = e.command(&["--validate", "--gold", p.to_str().unwrap(), "--json"]);
        assert!(r.is_err(), "{label}: a malformed gold file must be an error, got {:?}", r.map(|x| x.1));
    }
    // a missing gold file and an unreadable default are errors too
    assert!(e.command(&["--validate", "--gold", "/nonexistent/gold.toml"]).is_err());
    // and a gold file naming a non-deck card (a Sticker/Attraction) cannot be scored
    let e2 = Env::with_records(&[
        rec("Ride", "Artifact — Attraction", "Draw a card."),
        rec("Real", "Instant", "Counter target spell."),
    ]);
    e2.set_deck("Real", &["Real"], &[("Real", json!({"ctr": true}))]);
    e2.rules();
    let ride_id = serde_json::from_slice::<Value>(&e2.read("oracle.json")).unwrap()["cards"][0]["oracle_id"]
        .as_str()
        .unwrap()
        .to_string();
    let p = e2.0.join("g.toml");
    std::fs::write(&p, format!("[[card]]\nname='Ride'\noracle_id='{ride_id}'\nroles=[]\nprimary=''\n")).unwrap();
    assert!(e2.command(&["--validate", "--gold", p.to_str().unwrap()]).is_err());
}

#[test]
fn validation_needs_at_least_one_mapped_hand_pair() {
    let records = vec![rec("Alpha Counter", "Instant", "Counter target spell.")];
    let e = Env::with_records(&records);
    e.set_deck("Alpha Counter", &["Alpha Counter"], &[]);
    e.rules();
    let gold = gold_toml(&records, &[("Alpha Counter", &["counter"], "counter", &[])]);
    let p = e.0.join("g.toml");
    std::fs::write(&p, gold).unwrap();
    let err = e.command(&["--validate", "--gold", p.to_str().unwrap()]).unwrap_err();
    assert!(err.contains("no mapped pairs"), "{err}");
    // manual/pool cards only: an auto card with a mapped tag is ignored
    std::fs::write(
        e.0.join("data/cards.json"),
        r#"{"cards":[{"name":"Alpha Counter","source":"auto","tags":{"ctr":1}}]}"#,
    )
    .unwrap();
    assert!(e.command(&["--validate", "--gold", p.to_str().unwrap()]).is_err());
}

#[test]
fn defect_gold_duplicate_roles_are_accepted() {
    // Overrides reject a repeated role; the gold validator accepts ['draw','draw'] silently (a labelling typo).
    let (e, records) = crafted_validation_env();
    e.rules();
    let gold = gold_toml(&records, &[("Delta Draw", &["draw", "draw"], "draw", &[])]);
    let p = e.0.join("g.toml");
    std::fs::write(&p, gold).unwrap();
    let r = e.command(&["--validate", "--gold", p.to_str().unwrap(), "--json"]);
    assert!(r.is_err(), "a gold card listing the same role twice must be rejected");
}

// ------------------------------------------------------------------------------------------------ L. audit

fn audit_catalog() -> Vec<Value> {
    let mut r = vec![];
    for n in 0..60 {
        r.push(rec(
            &format!("Legal {n:02}"),
            "Sorcery",
            if n % 3 == 0 { "Draw two cards." } else { "Counter target spell." },
        ));
    }
    for n in 0..5 {
        r.push(with(
            rec(&format!("Banned {n}"), "Sorcery", "Draw two cards."),
            "legalities",
            json!({"commander": "banned"}),
        ));
        r.push(with(
            rec(&format!("Illegal {n}"), "Sorcery", "Draw two cards."),
            "legalities",
            json!({"commander": "not_legal"}),
        ));
        r.push(with(rec(&format!("Unknown {n}"), "Sorcery", "Draw two cards."), "legalities", json!({})));
    }
    r.push(rec("Ride A", "Artifact — Attraction", "Draw two cards."));
    r.push(rec("Sticker B", "Stickers", "Draw two cards."));
    r
}
fn audit_ids(out: &str) -> Vec<String> {
    out.lines().filter(|l| l.starts_with('[')).map(|l| l.split(" | ").nth(1).unwrap().to_string()).collect()
}
fn audit_names(out: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.starts_with('['))
        .map(|l| l.split(" | ").next().unwrap().split("] ").nth(1).unwrap().to_string())
        .collect()
}

#[test]
fn audit_is_deterministic_without_replacement_and_independent_of_catalog_order() {
    let records = audit_catalog();
    let i = index_of(&records);
    let a = roles::audit(&i, "card=[]", 7001, 25, None, false).unwrap();
    assert_eq!(a, roles::audit(&i, "card=[]", 7001, 25, None, false).unwrap());
    let mut rev = records.clone();
    rev.reverse();
    assert_eq!(
        a,
        roles::audit(&index_of(&rev), "card=[]", 7001, 25, None, false).unwrap(),
        "catalog order leaked into the sample"
    );
    let ids = audit_ids(&a);
    assert_eq!(ids.len(), 25);
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), 25, "sampled with replacement");
    for n in audit_names(&a) {
        assert!(n.starts_with("Legal"), "only Commander-legal cards are sampled: {n}");
    }
    // different seeds give different samples; a prefix of a larger request is the smaller request
    let b = roles::audit(&i, "card=[]", 7002, 25, None, false).unwrap();
    assert_ne!(audit_ids(&a), audit_ids(&b));
    let small = roles::audit(&i, "card=[]", 7001, 5, None, false).unwrap();
    assert_eq!(audit_ids(&small), ids[..5].to_vec());
    // every seed in a sweep yields a valid sample
    let mut distinct = BTreeSet::new();
    for seed in 0..40u64 {
        let s = roles::audit(&i, "card=[]", seed, 3, None, false).unwrap();
        assert_eq!(audit_ids(&s).len(), 3, "seed {seed}");
        distinct.insert(audit_ids(&s));
    }
    assert!(distinct.len() > 20, "seeds barely change the sample: {}", distinct.len());
}

#[test]
fn audit_excludes_gold_ids_and_never_shows_non_legal_or_non_deck_cards() {
    let records = audit_catalog();
    let i = index_of(&records);
    let gold_ids: Vec<String> = records.iter().take(40).map(|r| r["oracle_id"].as_str().unwrap().to_string()).collect();
    let gold: String =
        gold_ids.iter().map(|id| format!("[[card]]\nname='x'\noracle_id='{id}'\nroles=[]\nprimary=''\n")).collect();
    let out = roles::audit(&i, &gold, 1, 1000, None, false).unwrap();
    let shown = audit_ids(&out);
    assert_eq!(shown.len(), 20, "60 legal minus 40 gold, nothing else qualifies: {shown:?}");
    assert!(shown.iter().all(|id| !gold_ids.contains(id)));
    assert!(out.contains("SHOWN 20 cards"));
    // an id in the gold file that is not in the catalog is harmless; a gold file with different shapes is checked
    let other = format!("{gold}[[card]]\nname='ghost'\noracle_id='nope'\nroles=[]\nprimary=''\n");
    assert_eq!(audit_ids(&roles::audit(&i, &other, 1, 1000, None, false).unwrap()).len(), 20);
}

#[test]
fn audit_handles_degenerate_sizes_seeds_and_pools() {
    let i = index_of(&audit_catalog());
    assert!(roles::audit(&i, "card=[]", 0, 0, None, false).is_err(), "n = 0");
    for seed in [0, 1, u64::MAX, u64::MAX - 1, 1 << 63] {
        let out = roles::audit(&i, "card=[]", seed, 1, None, false).unwrap();
        assert_eq!(audit_ids(&out).len(), 1, "seed {seed}");
    }
    let all = roles::audit(&i, "card=[]", 3, usize::MAX, None, false).unwrap();
    assert_eq!(audit_ids(&all).len(), 60, "n larger than the pool shows the whole pool once");
    let empty = index_of(&[with(rec("Banned Only", "Sorcery", "x"), "legalities", json!({"commander": "banned"}))]);
    let none = roles::audit(&empty, "card=[]", 3, 5, None, false).unwrap();
    assert!(none.contains("SHOWN 0 cards, 0 assignments; rules-final 0 cards, 0 assignments"), "{none}");
    // a gold file that is not valid TOML, or lacks the card list, is an error rather than "exclude nothing"
    assert!(roles::audit(&i, "[[card", 0, 3, None, false).is_err());
    assert!(roles::audit(&i, "", 0, 3, None, false).is_err());
    assert!(roles::audit(&i, "[[card]]\nname='x'\n", 0, 3, None, false).is_err());
    assert!(roles::audit(&i, "card=[]", 0, 3, Some("not-a-rule"), false).is_err());
}

#[test]
fn audit_rule_and_certain_filters_apply_to_the_same_evidence_and_footer_counts_add_up() {
    let i = index_of(&audit_catalog());
    let out = roles::audit(&i, "card=[]", 5, 1000, Some("card-advantage"), false).unwrap();
    assert_eq!(audit_ids(&out).len(), 20, "only the Draw two cards cards");
    assert!(out.matches("draw: card-advantage certain=true").count() == 20);
    let certain = roles::audit(&i, "card=[]", 5, 1000, Some("counter-spell"), true).unwrap();
    assert_eq!(audit_ids(&certain).len(), 40);
    let none = roles::audit(&i, "card=[]", 5, 1000, Some("mana"), true).unwrap();
    assert!(none.contains("SHOWN 0 cards, 0 assignments"));
    // footer: assignments equals the sum of the roles lines; rules-final equals the needs_llm=false cards
    let all = roles::audit(&i, "card=[]", 9, 1000, None, false).unwrap();
    let mut assignments = 0;
    let (mut cards, mut finals, mut final_assignments) = (0, 0, 0);
    for l in all.lines().filter(|l| l.trim_start().starts_with("roles:")) {
        let (r, llm) = l.trim_start().trim_start_matches("roles:").split_once("| needs_llm=").unwrap();
        let n = r.split(',').filter(|s| !s.trim().is_empty()).count();
        assignments += n;
        cards += 1;
        if llm == "false" {
            finals += 1;
            final_assignments += n;
        }
    }
    let footer = format!(
        "SHOWN {cards} cards, {assignments} assignments; rules-final {finals} cards, {final_assignments} assignments"
    );
    assert!(all.contains(&footer), "{all}");
}

// ------------------------------------------------------------------------------------------------ M. CLI

#[test]
fn roles_cli_flag_matrix() {
    let e = Env::fixture();
    e.rules();
    let gold = e.0.join("gold.toml");
    std::fs::write(&gold, "card=[]\n").unwrap();
    e.write("roles_gold.toml", "card=[]\n");
    let g = gold.to_str().unwrap();
    let errors: Vec<Vec<&str>> = vec![
        vec![],
        vec!["--bogus"],
        vec!["rules"],
        vec!["-rules"],
        vec!["--RULES"],
        vec!["--rules", "--validate"],
        vec!["--rules", "--rules"],
        vec!["--stats", "--stats"],
        vec!["--rules", "--stats", "--show", "x"],
        vec!["--show"],
        vec!["--show", "--stats"],
        vec!["--show", "Counterspell", "--rules"],
        vec!["--show", "Counterspell", "--seed", "1"],
        vec!["--show", "Counterspell", "--json"],
        vec!["--show", "No Such Card Anywhere"],
        vec!["--audit", "--seed"],
        vec!["--audit", "--seed", "x"],
        vec!["--audit", "--seed", "-1"],
        vec!["--audit", "--seed", "1.5"],
        vec!["--audit", "--seed", "18446744073709551616"],
        vec!["--audit", "--seed", ""],
        vec!["--audit", "--n"],
        vec!["--audit", "--n", "0"],
        vec!["--audit", "--n", "-3"],
        vec!["--audit", "--n", "1.5"],
        vec!["--audit", "--n", "1e3"],
        vec!["--audit", "--n", "99999999999999999999999"],
        vec!["--audit", "--rule"],
        vec!["--audit", "--rule", "ramp"],
        vec!["--audit", "--rule", "MANA"],
        vec!["--audit", "--rule", "mana", "--rule", "fixing"],
        vec!["--audit", "--seed", "1", "--seed", "2"],
        vec!["--audit", "--n", "1", "--n", "2"],
        vec!["--audit", "--certain", "--certain"],
        vec!["--audit", "--json"],
        vec!["--audit", "--gold", g],
        vec!["--seed", "1"],
        vec!["--n", "3"],
        vec!["--rule", "mana"],
        vec!["--certain"],
        vec!["--rules", "--certain"],
        vec!["--stats", "--n", "1"],
        vec!["--validate", "--json", "--json"],
        vec!["--validate", "--gold"],
        vec!["--validate", "--gold", "--json"],
        vec!["--validate", "--gold", g, "--gold", g],
        vec!["--validate", "--seed", "1"],
        vec!["--stats", "--gold", g],
        vec!["--rules", "--json"],
        vec!["--rules", "--data-dir"],
        vec!["--audit", "--data-dir", "--n", "3"],
        vec!["--audit", "extra"],
        vec!["--audit", "--n", "3", "x"],
    ];
    for args in errors {
        let r = e.command(&args);
        assert!(r.is_err(), "{args:?} must be rejected, got {:?}", r.map(|x| x.0.len()));
    }
    // repeated --data-dir (Env appends its own)
    let twice = roles::command(
        &e.project(),
        &[
            "--stats".into(),
            "--data-dir".into(),
            e.kdir().display().to_string(),
            "--data-dir".into(),
            e.kdir().display().to_string(),
        ],
    );
    assert!(twice.is_err());
    // flags in any order are fine; every mode works
    let oks: Vec<Vec<&str>> = vec![
        vec!["--rules"],
        vec!["--stats"],
        vec!["--show", "Counterspell"],
        vec!["--show", "agadeem, the undercrypt"],
        vec!["--audit"],
        vec!["--audit", "--seed", "18446744073709551615", "--n", "3"],
        vec!["--n", "3", "--audit", "--seed", "0"],
        vec!["--audit", "--rule", "mana", "--certain", "--n", "5", "--seed", "1"],
        vec!["--certain", "--audit"],
    ];
    for args in oks {
        assert!(e.command(&args).is_ok(), "{args:?}");
    }
    // option values may not look like flags
    assert!(e.command(&["--audit", "--rule", "--certain"]).is_err());
}

#[test]
fn read_only_modes_write_nothing() {
    let e = Env::fixture();
    e.rules();
    e.write("roles_gold.toml", "card=[]\n");
    let before = e.snapshot();
    e.command(&["--stats"]).unwrap();
    e.command(&["--show", "Counterspell"]).unwrap();
    e.command(&["--show", "Spinny Ride"]).ok();
    e.command(&["--audit", "--seed", "7001", "--n", "50"]).unwrap();
    e.command(&["--audit", "--rule", "mana", "--certain"]).unwrap();
    e.find(&["--role", "ramp"]).unwrap();
    e.find(&["--name", "Counterspell", "--json"]).unwrap();
    let after = e.snapshot();
    assert!(
        before == after,
        "a read-only command changed {:?}",
        before.keys().filter(|k| before.get(*k) != after.get(*k)).collect::<Vec<_>>()
    );
    // --audit needs no roles.json, and does not create one
    std::fs::remove_file(e.kdir().join("roles.json")).unwrap();
    e.command(&["--audit", "--n", "3"]).unwrap();
    assert!(!e.kdir().join("roles.json").exists());
}

#[test]
fn show_resolves_shared_face_and_prepare_names() {
    let shared_a = rec("Same Name", "Instant", "Counter target spell.");
    let shared_b = rec("Same Name", "Sorcery", "Draw two cards.");
    let bolt = rec("Lightning Bolt", "Instant", "Lightning Bolt deals 3 damage to any target.");
    let prep = multi(
        "Body // Lightning Bolt",
        "prepare",
        "Creature // Instant",
        vec![
            face("Body", "Creature", "Flying", None),
            face("Lightning Bolt", "Instant", "Lightning Bolt deals 3 damage to any target.", None),
        ],
    );
    let e = Env::with_records(&[shared_a, shared_b, bolt, prep]);
    e.rules();
    let out = e.command(&["--show", "Same Name"]).unwrap().0;
    assert!(out.contains("counter") && out.contains("draw"), "both cards of a shared name are shown: {out}");
    let out = e.command(&["--show", "Lightning Bolt"]).unwrap().0;
    assert_eq!(out.matches("Lightning Bolt\nroles:").count(), 1, "the exact full name beats the prepare face: {out}");
    let out = e.command(&["--show", "Body"]).unwrap().0;
    assert!(out.contains("Body // Lightning Bolt"));
    assert!(e.command(&["--show", ""]).is_err());
}

// ------------------------------------------------------------------------------------------------ N. routing

#[test]
fn an_unparseable_line_routes_even_when_everything_else_is_certain() {
    let a = classify_text("Instant", "Counter target spell.");
    assert!(!a.needs_llm && a.evidence["counter"].certain);
    for junk in [
        "Frobnicate the quux with a flibbertigibbet.",
        "Zork: Blorp.",
        "12345",
        "Mirror the board horizontally.",
        "Each player rolls a die and the highest roll wins a prize.",
    ] {
        let a = classify_text("Instant", &format!("Counter target spell.\n{junk}"));
        assert!(a.needs_llm, "{junk}: {a:?}");
        assert!(has(&a, "counter"), "the recognised role is kept");
    }
    assert!(classify_text("Creature — Test", "Frobnicate the quux.").needs_llm);
}

#[test]
fn boilerplate_only_cards_do_not_route() {
    for text in [
        "Flying",
        "Flying\nVigilance\nTrample",
        "First strike, deathtouch, haste",
        "Defender\nReach",
        "Hexproof",
        "Ward {2}",
        "Protection from red",
        "Menace\nFlash",
        "Equip {2}",
        "Enchant creature\nEnchanted creature gets +1/+1.",
        "Kicker {2}\nFlying",
        "Cycling {2}",
        "Flashback {3}{R}",
        "Crew 2",
        "This creature enters tapped.",
        "As this creature enters, choose a color.",
        "Scry 1.",
        "Partner",
        "Companion — each nonland card in your starting deck has a different name.",
        "Equipped creature gets +2/+2.\nEquip {3}",
        "Trample\nInfect",
        "Islandwalk",
    ] {
        let a = classify_text("Creature — Test", text);
        assert!(!a.needs_llm, "{text:?} is boilerplate and must not route: {a:?}");
    }
}

#[test]
fn every_risky_domain_role_routes_even_beside_an_easy_rules_role() {
    for (text, role) in [
        ("Creatures you control have flying.", "evasion"),
        ("Creatures you control gain hexproof until end of turn.", "protection"),
        ("Whenever you cast an instant spell, copy it.", "copy"),
        ("Whenever a creature dies, each opponent loses 1 life.", "drain"),
        ("Destroy target creature. Then draw a card.", "removal"),
        ("Return target card from your graveyard to your hand.", "recursion"),
        ("Exile target player's graveyard.", "gy-hate"),
        ("Scry 3.", "selection"),
    ] {
        let a = classify_text("Enchantment", &format!("{text}\n{{T}}: Add {{G}}."));
        assert!(has(&a, role), "{text}: {a:?}");
        assert!(a.needs_llm, "{role} must route: {a:?}");
    }
}

#[test]
fn five_functions_truncate_to_four_roles_in_prominence_order_and_route() {
    let a = classify_text(
        "Artifact",
        concat!(
            "{T}: Add {G}.\n{1}, Sacrifice a creature: Create a 1/1 white Soldier creature token.\n",
            "Whenever a creature dies, you draw a card.\nCreatures you control get +1/+1.\n",
            "Whenever a creature dies, each opponent loses 1 life.\n",
            "At the beginning of your upkeep, each player discards a card.\n",
            "When this enters, destroy target creature."
        ),
    );
    assert_eq!(a.roles.len(), 4, "{a:?}");
    assert!(a.needs_llm, "overflow must route");
    assert_eq!(a.primary, a.roles[0]);
    let kept: BTreeSet<_> = a.roles.iter().collect();
    assert_eq!(kept.len(), 4);
}

#[test]
fn defect_repeatable_damage_to_target_opponent_is_not_drain_and_not_routed() {
    // Real cards Raphael, Tough Turtle and Casey Jones, Back Alley Brute: a repeatable trigger dealing damage to
    // "target opponent". The drain pattern lists each/that/target player but not "target opponent", and the line is
    // absorbed by some other pattern, so the card is rules-final with no role (audit seed 7001/7002).
    let i = audit_cards();
    for name in ["Raphael, Tough Turtle", "Casey Jones, Back Alley Brute"] {
        let a = classify_named(&i, name);
        assert!(has(&a, "drain") || a.needs_llm, "{name}: {a:?}");
    }
}

#[test]
fn defect_third_person_loot_is_filtering_not_draw() {
    // Real card Cephalid Broker: "Target player draws two cards, then discards two cards." The loot exclusions are
    // spelled in the second person ("draw two cards, then discard two"), so the third-person text becomes draw.
    let i = audit_cards();
    let a = classify_named(&i, "Cephalid Broker");
    assert!(!has(&a, "draw"), "a draw-two/discard-two loot nets no cards: {a:?}");
}

#[test]
fn defect_target_player_gains_life_activation_is_lifegain() {
    // Real cards Mournful Zombie and Tonic Peddler: "{W}, {T}: Target player gains N life." (repeatable activation) and
    // Witch's Clinic ("Target commander gains lifelink until end of turn" by a repeatable activation). The lifegain
    // check only recognises "you gain"/"its controller gains", and the lifelink grant list lacks "target commander".
    let i = audit_cards();
    for name in ["Mournful Zombie", "Tonic Peddler", "Witch's Clinic"] {
        let a = classify_named(&i, name);
        assert!(has(&a, "lifegain") || a.needs_llm, "{name}: {a:?}");
    }
}

#[test]
fn defect_bounce_of_two_other_target_permanents_is_removal() {
    // Real card Marang River Regent: "return up to two other target nonland permanents to their owners' hands".
    // The removal pattern accepts "up to two target" but not "up to two other target".
    let i = audit_cards();
    let a = classify_named(&i, "Marang River Regent // Coil and Catch");
    assert!(has(&a, "removal") || a.needs_llm, "{a:?}");
}

#[test]
fn defect_variable_you_control_clause_blocks_negative_toughness_removal() {
    // Real card Lys Alana Scarblade: "{T}, Discard an Elf card: Target creature gets -X/-X until end of turn,
    // where X is the number of Elves you control." The removal check treats the "you control" inside the X clause as
    // ownership of the target and rejects the line, while nothing routes it.
    let i = audit_cards();
    let a = classify_named(&i, "Lys Alana Scarblade");
    assert!(has(&a, "removal") || a.needs_llm, "{a:?}");
}

#[test]
fn defect_quoted_ability_colon_corrupts_the_removal_check() {
    // Real card Grave Birthing: "Target opponent exiles a card from their graveyard. You create a 1/1 ... Scion token.
    // It has \"Sacrifice this token: Add {C}.\"" The graveyard-exile clause is not removal, but the removal check
    // splits the line at the colon inside the quoted ability, loses the clause, and accepts it.
    let i = audit_cards();
    let a = classify_named(&i, "Grave Birthing");
    assert!(!has(&a, "removal"), "exiling a card from a graveyard is not permanent removal: {a:?}");
}

#[test]
fn defect_mentioning_cascade_is_not_cascading() {
    // Real card The First Doctor: "Whenever you cast a spell with cascade, put a +1/+1 counter on target artifact or
    // creature." The draw check treats any line containing "cascade" as card advantage.
    let i = audit_cards();
    let a = classify_named(&i, "The First Doctor");
    assert!(!has(&a, "draw"), "{a:?}");
}

#[test]
fn defect_corpse_counter_is_bookkeeping_not_counters() {
    // Real card From the Catacombs: "...with a corpse counter on it." Bookkeeping markers (ki, void, finality, time...)
    // are excluded by name; corpse is not in that list, so the reanimation spell is also `counters`.
    let i = audit_cards();
    let a = classify_named(&i, "From the Catacombs");
    assert!(!has(&a, "counters"), "{a:?}");
}

// ------------------------------------------------------------------------------------------------ O. misc

#[test]
fn taxonomy_ids_match_the_classifier_and_every_rule_names_a_taxonomy_role() {
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    let ids: Vec<_> = t.role.iter().map(|r| r.id.as_str()).collect();
    for id in IDS {
        assert!(ids.contains(&id), "{id}");
    }
    for r in RULES {
        assert!(IDS.contains(&r.role), "rule {} names unknown role {}", r.id, r.role);
    }
    let rule_ids: BTreeSet<_> = RULES.iter().map(|r| r.id).collect();
    assert_eq!(rule_ids.len(), RULES.len(), "rule ids must be unique");
    // every rule's role has at least one rule and roles.json evidence can only cite these ids
    for id in IDS {
        assert!(RULES.iter().any(|r| r.role == id), "role {id} has no rule (stax/finisher etc. are rule-backed too)");
    }
    assert!(roles::check_role("ramp").is_ok());
    for bad in ["", "RAMP", "ramp ", "Ramp"] {
        assert!(roles::check_role(bad).is_err());
    }
    // Legal enum sanity used by the audit
    assert_eq!(Legal::Legal.as_str(), "legal");
}
