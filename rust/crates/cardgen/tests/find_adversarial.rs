//! Offline adversarial checks for phase 2. Deliberate failures are named `defect_*`.
//! Real-catalog checks are opt-in; all writes are to disposable temporary projects.
use cardgen::data::Project;
use cardgen::knowledge::{self, FindError, Index, Query};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "find-adversarial-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Env(Temp);
impl Env {
    fn new(cards: Vec<Value>) -> Self {
        let e = Self(Temp::new());
        fs::create_dir_all(e.dir()).unwrap();
        e.write_catalog(json!({"meta":{"schema":1,"count":cards.len()},"cards":cards}));
        e.decks(json!({"mine":[],"pool":[]}));
        e
    }
    fn fixture() -> Self {
        let v: Value = serde_json::from_str(include_str!("fixtures/find_oracle.json")).unwrap();
        Self::new(v["cards"].as_array().unwrap().clone())
    }
    fn root(&self) -> &Path {
        &self.0.0
    }
    fn dir(&self) -> PathBuf {
        self.root().join("data/knowledge")
    }
    fn write_catalog(&self, v: Value) {
        fs::write(self.dir().join("oracle.json"), serde_json::to_vec(&v).unwrap()).unwrap();
    }
    fn decks(&self, v: Value) {
        fs::write(self.root().join("data/decks.json"), serde_json::to_vec(&v).unwrap()).unwrap();
    }
    fn run(&self, args: &[&str]) -> Result<String, FindError> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.extend(["--data-dir".into(), self.dir().display().to_string()]);
        knowledge::find(&Project::new(self.root()), &a)
    }
    fn ok(&self, args: &[&str]) -> String {
        self.run(args).unwrap_or_else(|e| panic!("{args:?}: {e}"))
    }
    fn rows(&self, args: &[&str]) -> Vec<Value> {
        let mut a = args.to_vec();
        a.push("--json");
        serde_json::from_str(&self.ok(&a)).expect("valid JSON array")
    }
    fn names(&self, args: &[&str]) -> Vec<String> {
        self.rows(args).iter().map(|c| c["name"].as_str().unwrap().into()).collect()
    }
    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = binary();
        c.arg("--root").arg(self.root()).arg("find").args(args);
        c
    }
}
fn card(name: &str) -> Value {
    json!({"name":name,"cmc":1,"type_line":"Artifact","color_identity":[],
        "legalities":{"commander":"legal"},"oracle_id":name,"mana_cost":"{1}","oracle_text":"Draw a card."})
}
fn parse(args: &[&str]) -> Result<Query, FindError> {
    Query::parse(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}
fn binary() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cardgen"));
    c.env_remove("CARDGEN_KNOWLEDGE_DIR").env_remove("COMMANDER_SIM_ROOT");
    c
}
/// Files avoid pipe-buffer deadlocks on full-catalog output; a stuck child is killed and reaped.
fn bounded(mut cmd: Command) -> Output {
    let t = Temp::new();
    let stdout = t.0.join("stdout");
    let stderr = t.0.join("stderr");
    let mut child = cmd.stdout(File::create(&stdout).unwrap()).stderr(File::create(&stderr).unwrap()).spawn().unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if started.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("find did not finish within 15 s: {}", fs::read_to_string(stderr).unwrap());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    Output { status, stdout: fs::read(stdout).unwrap(), stderr: fs::read(stderr).unwrap() }
}
fn clean_error(o: &Output) {
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success(), "unexpected success: {}", String::from_utf8_lossy(&o.stdout));
    assert!(o.stdout.is_empty(), "stdout on failure");
    assert!(err.contains("cardgen:") && !err.contains("panicked") && !err.contains("stack overflow"), "{err}");
}
fn fold(s: &str) -> String {
    s.replace('’', "'")
        .replace('Æ', "AE")
        .replace('æ', "ae")
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[test]
fn defect_ascii_exact_name() {
    let e = Env::new(vec![card("Andúril, Narsil Reforged")]);
    assert_eq!(e.names(&["--name", "Anduril, Narsil Reforged"]), ["Andúril, Narsil Reforged"]);
}
#[test]
fn defect_ascii_name_contains() {
    let e = Env::new(vec![card("Gríma Wormtongue")]);
    assert_eq!(e.names(&["--name-contains", "grima"]), ["Gríma Wormtongue"]);
}
#[test]
fn defect_apostrophe_and_ligature_names() {
    let e = Env::new(vec![card("Lim-Dûl's Vault"), card("Aether Vial")]);
    let mut failures = vec![];
    for (q, expected) in [("Lim-Dûl’s Vault", "Lim-Dûl's Vault"), ("Æther Vial", "Aether Vial")] {
        if e.names(&["--name", q]) != [expected] {
            failures.push(q);
        }
    }
    assert!(failures.is_empty(), "unresolved spelling variants: {failures:?}");
}
#[test]
fn defect_deck_spelling_exclusion() {
    let e = Env::new(vec![card("Andúril, Narsil Reforged"), card("Urza's Bauble"), card("Aether Vial")]);
    e.decks(json!({"mine":[{"key":"test","ident":"C","cards":[
        "Anduril, Narsil Reforged","Urza’s Bauble","Æther Vial"]}]}));
    assert!(e.names(&["--deck", "test"]).is_empty(), "already-in-deck cards leaked: {}", e.ok(&["--deck", "test"]));
}
#[test]
fn name_case_outer_space_empty_and_filter_intersection() {
    let e = Env::fixture();
    assert_eq!(e.names(&["--name", " \tSoL rInG\n"]), ["Sol Ring"]);
    assert!(e.names(&["--name", ""]).is_empty());
    assert!(e.names(&["--name", " Sol  Ring "]).is_empty(), "exact matching currently preserves inner spaces");
    let keep = [
        "--name",
        "Sol Ring",
        "--name-contains",
        "ring",
        "--ident",
        "C",
        "--legal",
        "legal",
        "--cmc",
        "<=1",
        "--type",
        "artifact",
        "--text",
        "add",
        "--sort",
        "usd",
        "--limit",
        "1",
        "--offset",
        "0",
        "--full",
        "--meld",
    ];
    assert_eq!(e.names(&keep), ["Sol Ring"]);
    for extra in [
        vec!["--gc"],
        vec!["--cmc", ">1"],
        vec!["--ident", "W"],
        vec!["--type", "land"],
        vec!["--text", "nonexistent"],
        vec!["--name-contains", "other"],
        vec!["--legal", "banned"],
    ] {
        let mut a = vec!["--name", "Sol Ring"];
        a.extend(extra);
        // A colourless card is admitted by W, so this one must remain.
        let expected = if a.contains(&"W") { 1 } else { 0 };
        assert_eq!(e.rows(&a).len(), expected, "{a:?}");
    }
}
#[test]
fn parser_missing_repeated_unknown_and_numeric_edges() {
    let flags = [
        "--name",
        "--name-contains",
        "--ident",
        "--legal",
        "--cmc",
        "--type",
        "--text",
        "--deck",
        "--sort",
        "--limit",
        "--offset",
        "--data-dir",
    ];
    for flag in flags {
        assert!(matches!(parse(&[flag]), Err(FindError::Usage(_))), "{flag}");
    }
    for (flag, value) in [
        ("--name", "x"),
        ("--name-contains", "x"),
        ("--ident", "C"),
        ("--legal", "any"),
        ("--cmc", "1"),
        ("--deck", "x"),
        ("--sort", "name"),
        ("--limit", "1"),
        ("--offset", "0"),
        ("--data-dir", "x"),
    ] {
        assert!(matches!(parse(&[flag, value, flag, value]), Err(FindError::Usage(_))), "{flag}");
    }
    for flag in ["--role", "--digest", "--map"] {
        assert!(parse(&[flag]).is_err());
    }
    for value in ["<=", "=>2", "==2", "<", ">", "NaN", "inf", "-inf", "1e1000", "∞"] {
        assert!(parse(&["--cmc", value]).is_err(), "{value}");
    }
    for value in ["<=2", ">=2", "1e3", ".5", "-1", " 2 ", "< .5", ">-1"] {
        assert!(parse(&["--cmc", value]).is_ok(), "{value}");
    }
    assert!(parse(&["--limit", "0"]).is_err(), "--limit 0 is a usage error");
    for flag in ["--limit", "--offset"] {
        assert!(flag == "--limit" || parse(&[flag, "0"]).is_ok());
        assert!(parse(&[flag, &usize::MAX.to_string()]).is_ok());
        for value in ["18446744073709551616", "99999999999999999999999999999", "-1", "NaN", "1e3", " 2 "] {
            assert!(parse(&[flag, value]).is_err(), "{flag} {value}");
        }
    }
    for value in ["wu", "WWuu", "CC", ""] {
        assert!(parse(&["--ident", value]).is_ok());
    }
    for value in ["wubrgx", "colorless", " WU "] {
        assert!(parse(&["--ident", value]).is_err());
    }
    for value in ["legal", "any", "banned", "not_legal", "restricted"] {
        assert!(parse(&["--legal", value]).is_ok());
    }
    for value in ["unknown", "LEGAL", ""] {
        assert!(parse(&["--legal", value]).is_err());
    }
    for value in ["edhrec", "name", "cmc", "usd"] {
        assert!(parse(&["--sort", value]).is_ok());
    }
    for value in ["price", "Name", ""] {
        assert!(parse(&["--sort", value]).is_err());
    }
}
#[test]
fn defect_flags_consumed_as_values() {
    let mut swallowed = vec![];
    for flag in ["--name", "--name-contains", "--type", "--text", "--deck", "--data-dir"] {
        if parse(&[flag, "--json"]).is_ok() {
            swallowed.push(flag);
        }
    }
    assert!(swallowed.is_empty(), "flags missing values accepted and --json silently consumed: {swallowed:?}");
}
#[test]
fn defect_unknown_flag_hidden_in_value() {
    let e = Env::fixture();
    let o = bounded(e.cmd(&["--name", "--role"]));
    clean_error(&o);
    assert!(String::from_utf8_lossy(&o.stderr).contains("usage:"));
}
#[test]
fn type_multiword_subtype_supertype_and_empty_identity() {
    let e = Env::fixture();
    assert_eq!(e.names(&["--type", " basic LAND ", "--sort", "name"]), ["Island", "Plains"]);
    assert_eq!(e.names(&["--type", "elf druid"]), ["Llanowar Elves"]);
    assert!(e.names(&["--type", "elfish"]).is_empty());
    for id in ["", "C", "CC", "c"] {
        assert!(e.rows(&["--ident", id, "--limit", "100000"]).iter().all(|c| c["identity"] == ""));
    }
}
#[test]
fn regex_flags_alternation_unicode_anchors_and_joiner() {
    let mut a = card("Faces // Back");
    a["oracle_text"] = Value::Null; // optional text can be null, just as absent
    a["card_faces"] = json!([
        {"name":"Faces","oracle_text":"Draw a card.\nSéance — Æther."},
        {"name":"Back","oracle_text":"COUNTER target spell."}]);
    let e = Env::new(vec![a]);
    for pattern in [
        r"draw|counter",
        r"draw(?-i) a card",
        r"(?i:séance)",
        r"(?-i)COUNTER",
        r"(?m)^COUNTER",
        r"(?s)^Draw.*spell\.$",
        r"(?m)^//$",
        r"(?x) Draw \s+ a \s+ card ",
        r"(?<=Draw )a",
        r"(Draw)\s+\1|COUNTER",
    ] {
        assert_eq!(e.names(&["--text", pattern]).len(), 1, "{pattern}");
    }
    for pattern in [r"(?-i)counter", r"^COUNTER", r"^Draw.*spell\.$", r"^Faces$"] {
        assert!(e.names(&["--text", pattern]).is_empty(), "{pattern}");
    }
    // The joiner is searchable by design; repeated filters may match different faces.
    assert_eq!(e.names(&["--text", "Draw", "--text", "COUNTER"]).len(), 1);
}
#[test]
fn pathological_and_invalid_regexes_are_bounded() {
    let mut a = card("Long text");
    a["oracle_text"] = json!(format!("{}!", "a".repeat(6000)));
    let e = Env::new(vec![a]);
    for pattern in [
        r"^(a+)+$",
        r"^(?=(a+)+$).*$",
        r"^(a|aa)+\1$",
        r"(?<=a)a{1,3000}(?!a)",
        r"(",
        r"(?<=a+)",
        r"(?z)a",
        r"a{9999999999999999999999}",
    ] {
        let o = bounded(e.cmd(&["--text", pattern, "--count"]));
        if o.status.success() {
            assert!(String::from_utf8_lossy(&o.stdout).trim().parse::<usize>().is_ok());
        } else {
            clean_error(&o);
        }
    }
}
#[test]
fn legality_unknown_absent_null_and_restricted() {
    let mut cards = vec![];
    for value in
        [json!("legal"), json!("banned"), json!("not_legal"), json!("restricted"), json!("future"), Value::Null]
    {
        let mut c = card(&value.to_string());
        c["legalities"]["commander"] = value;
        cards.push(c);
    }
    let mut c = card("Absent legalities");
    c.as_object_mut().unwrap().remove("legalities");
    cards.push(c);
    let e = Env::new(cards);
    assert_eq!(e.rows(&[]).len(), 1);
    assert_eq!(e.rows(&["--legal", "any"]).len(), 7);
    for legal in ["banned", "not_legal", "restricted"] {
        assert_eq!(e.rows(&["--legal", legal]).len(), 1);
    }
    assert_eq!(e.rows(&["--legal", "any"]).iter().filter(|c| c["legal"] == "unknown").count(), 3);
}
#[test]
fn decks_priority_commanders_faces_unknown_names_and_override() {
    let e = Env::fixture();
    e.decks(json!({"mine":[{"key":"same","ident":"WU","commander":"Isamaru, Hound of Konda",
        "commander2":"Petty Theft","cards":["Ice","Island","Island","Not in Catalog"]}],
        "pool":[{"key":"same","ident":"R","cards":["Sol Ring"]}]}));
    let rows = e.rows(&["--deck", "SAME", "--limit", "100000"]);
    let names: Vec<_> = rows.iter().map(|c| c["name"].as_str().unwrap()).collect();
    for excluded in ["Isamaru, Hound of Konda", "Brazen Borrower // Petty Theft", "Fire // Ice", "Island"] {
        assert!(!names.contains(&excluded), "{excluded}");
    }
    assert!(names.contains(&"Sol Ring"));
    assert!(
        e.rows(&["--deck", "same", "--limit", "100000"]).iter().all(|c| c["identity"]
            .as_str()
            .unwrap()
            .chars()
            .all(|x| "WU".contains(x)))
    );
    for ident in ["", "C"] {
        e.decks(json!({"mine":[{"key":"test","ident":ident}]}));
        assert!(e.rows(&["--deck", "test"]).iter().all(|c| c["identity"] == ""));
    }
    e.decks(json!({"mine":[{"key":"test","ident":"X"}]}));
    assert!(matches!(e.run(&["--deck", "test"]), Err(FindError::Failed(_))));
    assert!(matches!(e.run(&["--deck", "test", "--ident", "C"]), Err(FindError::Failed(_))));
    assert!(e.run(&["--deck", "unknown"]).unwrap_err().to_string().contains("test"));
}
#[test]
fn malformed_missing_deck_files_are_clean_errors() {
    let e = Env::fixture();
    for contents in ["{", r#"{"mine":null}"#, r#"{"mine":[{"key":5}]}"#, r#"{"mine":[{"key":"x","cards":[null]}]}"#] {
        fs::write(e.root().join("data/decks.json"), contents).unwrap();
        clean_error(&bounded(e.cmd(&["--deck", "x"])));
    }
    fs::remove_file(e.root().join("data/decks.json")).unwrap();
    clean_error(&bounded(e.cmd(&["--deck", "x"])));
    assert!(!e.rows(&["--name", "Sol Ring"]).is_empty(), "decks.json is unnecessary without --deck");
}
#[test]
fn output_json_controls_unicode_full_and_empty_modes() {
    let name = "Quote \" slash \\ tab\t newline\n null\0 astral 🐉";
    let text = "Text\r\n\t\u{0008}\u{000c}\0 \"\\ é 🐉";
    let mut c = card(name);
    c["oracle_text"] = json!(text);
    c["power"] = json!("1");
    c["toughness"] = json!("2");
    c["keywords"] = json!(["Flying"]);
    c["produced_mana"] = json!(["C"]);
    c["reserved"] = json!(true);
    let e = Env::new(vec![c]);
    for full in [vec![], vec!["--full"]] {
        let rows = e.rows(&full);
        assert_eq!(rows[0]["name"], name);
        assert_eq!(rows[0]["oracle_text"], text);
        if !full.is_empty() {
            assert_eq!(rows[0]["power"], "1");
            assert_eq!(rows[0]["reserved"], true);
        }
    }
    let text_out = e.ok(&["--full"]);
    assert!(text_out.contains(text) && text_out.contains("P/T 1/2"));
    for mode in [
        vec![],
        vec!["--full"],
        vec!["--json"],
        vec!["--full", "--json"],
        vec!["--count"],
        vec!["--count", "--json", "--full"],
    ] {
        let mut args = vec!["--name", "absent"];
        args.extend(mode);
        let s = e.ok(&args);
        if args.contains(&"--count") {
            assert_eq!(s, "0\n");
        } else if args.contains(&"--json") {
            assert_eq!(serde_json::from_str::<Value>(&s).unwrap(), json!([]));
        } else {
            assert_eq!(s, "no matches\n");
        }
    }
}
#[test]
fn defect_wide_table_alignment() {
    let e = Env::new(vec![card("長い名前"), card("Séance")]);
    let out = e.ok(&["--sort", "name"]);
    // These fixed CJK ideographs each occupy two terminal cells; é occupies one.
    let display_width = |s: &str| s.chars().map(|c| if "長い名前".contains(c) { 2 } else { 1 }).sum::<usize>();
    let columns: Vec<_> = out.lines().skip(1).map(|l| display_width(&l[..l.find("{1}").unwrap()])).collect();
    assert_eq!(columns[0], columns[1], "cost column has different terminal positions: {columns:?}\n{out}");
}
#[test]
fn paging_extremes_count_notes_and_all_sort_ties() {
    let mut cards = vec![];
    for i in 0..43 {
        let mut c = card(if i % 2 == 0 { "Same" } else { "same" });
        c["oracle_id"] = json!(format!("id-{i:03}"));
        if i % 3 != 0 {
            c["edhrec_rank"] = json!(i % 4);
            c["usd"] = json!((i % 4).to_string());
        }
        c["cmc"] = json!(i % 3);
        cards.push(c);
    }
    let e = Env::new(cards);
    for sort in ["edhrec", "name", "cmc", "usd"] {
        let base = ["--legal", "any", "--full", "--sort", sort];
        let mut args = base.to_vec();
        args.extend(["--limit", "100000"]);
        let all = e.rows(&args);
        assert_eq!(e.rows(&args), all, "determinism: {sort}");
        let mut pages = vec![];
        for offset in (0..43).step_by(7) {
            let o = offset.to_string();
            let mut a = base.to_vec();
            a.extend(["--limit", "7", "--offset", &o]);
            pages.extend(e.rows(&a));
        }
        assert_eq!(pages, all, "paging: {sort}");
        assert_eq!(pages.iter().map(|c| c["oracle_id"].as_str().unwrap()).collect::<BTreeSet<_>>().len(), 43);
    }
    assert_eq!(e.ok(&["--count", "--json", "--full", "--limit", "1", "--offset", &usize::MAX.to_string()]), "43\n");
    assert!(e.rows(&["--offset", &usize::MAX.to_string(), "--limit", &usize::MAX.to_string()]).is_empty());
    assert!(matches!(e.run(&["--limit", "0"]), Err(FindError::Usage(_))));
    assert_eq!(e.ok(&["--offset", "43"]), "no matches in this page\n");
    assert!(e.ok(&["--limit", "7", "--offset", "7"]).contains("showing 8-14 of 43; --offset 14"));
}
#[test]
fn malformed_catalogs_never_panic() {
    let e = Env::new(vec![card("Test")]);
    for bad in [
        r#"{"meta":{"schema":1},"cards":["#,
        r#"{"meta":{"schema":0},"cards":[]}"#,
        r#"{"meta":{"schema":2},"cards":[]}"#,
        r#"{"meta":{"count":0},"cards":[]}"#,
        r#"{"meta":{"schema":1,"count":2},"cards":[]}"#,
        r#"{"meta":{"schema":1},"cards":[{"cmc":"1"}]}"#,
        r#"{"meta":{"schema":1},"cards":[{"color_identity":null}]}"#,
        r#"{"meta":{"schema":1},"cards":[{"legalities":null}]}"#,
        r#"{"meta":{"schema":1},"cards":[{"card_faces":false}]}"#,
    ] {
        fs::write(e.dir().join("oracle.json"), bad).unwrap();
        clean_error(&bounded(e.cmd(&["--json"])));
    }
}
#[test]
fn nullable_faces_empty_catalog_unknown_fields_and_optional_count() {
    let e = Env::new(vec![]);
    for meta in [json!({"schema":1}), json!({"schema":1,"count":0})] {
        e.write_catalog(json!({"meta":meta,"cards":[]}));
        assert_eq!(e.ok(&["--count"]), "0\n");
    }
    let mut c = card("Test");
    c["card_faces"] = Value::Null;
    c["future"] = json!({"any":[1,null]});
    e.write_catalog(json!({"meta":{"schema":1},"cards":[c]}));
    assert_eq!(e.names(&[]), ["Test"]);
}
#[test]
fn defect_empty_catalog_card_name() {
    let e = Env::new(vec![card("")]);
    clean_error(&bounded(e.cmd(&["--json"])));
}
#[test]
fn defect_duplicate_oracle_ids() {
    let c = card("Same card");
    let e = Env::new(vec![c.clone(), c]);
    let o = bounded(e.cmd(&["--json"]));
    // Corrupt duplicates may be rejected or deduplicated, but must not count as two cards.
    if o.status.success() {
        let rows: Vec<Value> = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(rows.len(), 1, "duplicate oracle_id inflated result and count");
    } else {
        clean_error(&o);
    }
}
#[test]
fn hundred_megabyte_catalog_load_is_bounded() {
    let e = Env::new(vec![]);
    let path = e.dir().join("oracle.json");
    let mut out = BufWriter::new(File::create(&path).unwrap());
    out.write_all(b"{\"meta\":{\"schema\":1,\"count\":1024},\"cards\":[").unwrap();
    let text = "a".repeat(102400);
    for i in 0..1024 {
        if i != 0 {
            out.write_all(b",").unwrap();
        }
        let mut c = card(&format!("Large {i}"));
        c["oracle_text"] = json!(text);
        serde_json::to_writer(&mut out, &c).unwrap();
    }
    out.write_all(b"]}").unwrap();
    out.flush().unwrap();
    drop(out);
    assert!(fs::metadata(&path).unwrap().len() > 100_000_000);
    let mut cmd = Command::new("/usr/bin/time");
    cmd.env_remove("CARDGEN_KNOWLEDGE_DIR")
        .args(["-v", env!("CARGO_BIN_EXE_cardgen"), "--root"])
        .arg(e.root())
        .args(["find", "--count"]);
    let o = bounded(cmd);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(o.stdout, b"1024\n");
    eprintln!("100 MB catalog: {}", String::from_utf8_lossy(&o.stderr));
    assert_rss(&o.stderr);
}
fn assert_rss(stderr: &[u8]) {
    let s = String::from_utf8_lossy(stderr);
    let kb: usize = s
        .lines()
        .find_map(|l| l.trim().strip_prefix("Maximum resident set size (kbytes):").map(|v| v.trim().parse().unwrap()))
        .expect("/usr/bin/time -v peak RSS");
    assert!(kb < 1_000_000, "peak RSS {kb} KB exceeds 1 GB");
}
#[test]
fn directory_resolution_empty_env_relative_override_symlink_and_missing_git() {
    let e = Env::new(vec![card("Local")]);
    let mut c = e.cmd(&["--count"]);
    c.env("CARDGEN_KNOWLEDGE_DIR", "");
    assert_eq!(bounded(c).stdout, b"1\n");
    let alt = e.root().join("alt");
    fs::create_dir(&alt).unwrap();
    fs::write(alt.join("oracle.json"), r#"{"meta":{"schema":1},"cards":[]}"#).unwrap();
    let mut c = e.cmd(&["--count", "--data-dir", "alt"]);
    c.current_dir(e.root()).env("CARDGEN_KNOWLEDGE_DIR", "missing");
    assert_eq!(bounded(c).stdout, b"0\n");
    let mut c = e.cmd(&["--count"]);
    c.env("CARDGEN_KNOWLEDGE_DIR", &alt);
    assert_eq!(bounded(c).stdout, b"0\n");
    let mut c = e.cmd(&["--count", "--data-dir", "data/knowledge"]);
    c.current_dir(e.root()).env("CARDGEN_KNOWLEDGE_DIR", &alt);
    assert_eq!(bounded(c).stdout, b"1\n");
    let mut c = e.cmd(&["--count"]);
    c.env("PATH", "/missing");
    assert_eq!(bounded(c).stdout, b"1\n");
    #[cfg(unix)]
    {
        let link = e.root().join("link");
        std::os::unix::fs::symlink(e.root(), &link).unwrap();
        let mut c = binary();
        c.arg("--root").arg(&link).args(["find", "--count"]);
        assert_eq!(bounded(c).stdout, b"1\n");
    }
    fs::remove_file(e.dir().join("oracle.json")).unwrap();
    fs::create_dir(e.dir().join("oracle.json")).unwrap();
    let mut c = e.cmd(&["--count"]);
    c.env("PATH", "/missing");
    clean_error(&bounded(c));
}
#[test]
fn worktree_fallback_and_local_precedence_without_commits() {
    let e = Env::new(vec![card("Main catalog")]);
    // A minimal linked-worktree git layout. No commits, refs, or worktrees in the real repository change.
    let common = e.root().join(".git");
    fs::create_dir_all(common.join("objects")).unwrap();
    fs::create_dir_all(common.join("refs")).unwrap();
    fs::write(common.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::write(common.join("config"), "[core]\nrepositoryformatversion = 0\nbare = false\n").unwrap();
    let linked = common.join("worktrees/linked");
    fs::create_dir_all(&linked).unwrap();
    fs::write(linked.join("HEAD"), "ref: refs/heads/linked\n").unwrap();
    fs::write(linked.join("commondir"), "../..\n").unwrap();
    let wt = e.root().join("linked");
    fs::create_dir_all(wt.join("data/knowledge")).unwrap();
    fs::write(wt.join(".git"), format!("gitdir: {}\n", linked.display())).unwrap();
    fs::write(linked.join("gitdir"), wt.join(".git").display().to_string()).unwrap();
    fs::write(wt.join("data/knowledge/roles.toml"), "tracked marker").unwrap();
    let run = || {
        let mut c = binary();
        c.arg("--root").arg(&wt).args(["find", "--name", "Main catalog", "--count"]);
        bounded(c)
    };
    let o = run();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(o.stdout, b"1\n");
    let mut c = binary();
    c.arg("--root").arg(&wt).args(["find", "--count"]).env("PATH", "/missing");
    clean_error(&bounded(c));
    #[cfg(unix)]
    {
        let link = e.root().join("symlinked-linked");
        std::os::unix::fs::symlink(&wt, &link).unwrap();
        let mut c = binary();
        c.arg("--root").arg(link).args(["find", "--count"]);
        assert_eq!(bounded(c).stdout, b"1\n");
    }
    fs::write(wt.join("data/knowledge/oracle.json"), r#"{"meta":{"schema":1},"cards":[]}"#).unwrap();
    assert_eq!(run().stdout, b"0\n");
}

#[cfg(unix)]
#[test]
fn unreadable_file_fails_cleanly() {
    use std::os::unix::fs::PermissionsExt;
    let e = Env::new(vec![card("Test")]);
    let p = e.dir().join("oracle.json");
    fs::set_permissions(&p, fs::Permissions::from_mode(0)).unwrap();
    let readable = File::open(&p).is_ok();
    let o = bounded(e.cmd(&["--count"]));
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    if readable {
        eprintln!("privileged user can read mode 000; permission-denial check unavailable");
    } else {
        clean_error(&o);
    }
}
#[test]
fn closed_stdout_pipe_does_not_panic() {
    let mut cards = vec![];
    for i in 0..1000 {
        cards.push(card(&format!("{}-{i}", "x".repeat(200))));
    }
    let e = Env::new(cards);
    let t = Temp::new();
    let err = t.0.join("err");
    let mut child =
        e.cmd(&["--limit", "100000"]).stdout(Stdio::piped()).stderr(File::create(&err).unwrap()).spawn().unwrap();
    let reader = child.stdout.take().unwrap();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    drop(reader);
    let deadline = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if deadline.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("closed pipe hung");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let err = fs::read_to_string(err).unwrap();
    assert!(status.success(), "{status}: {err}");
    assert!(!err.contains("panic"), "{err}");
}
#[cfg(unix)]
#[test]
fn defect_stdout_write_failure_is_success() {
    let e = Env::fixture();
    let out = File::options().write(true).open("/dev/full").unwrap();
    let o = e.cmd(&["--json", "--limit", "100000"]).stdout(out).stderr(Stdio::piped()).output().unwrap();
    assert!(!o.status.success(), "find reported success after stdout failed with ENOSPC");
    assert!(!String::from_utf8_lossy(&o.stderr).contains("panicked"));
}

fn real_project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}
fn real_dir() -> PathBuf {
    knowledge::knowledge_dir(&real_project(), None).expect("local catalog; never download")
}
fn real_rows(args: &[&str]) -> Vec<Value> {
    let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    a.extend([
        "--data-dir".into(),
        real_dir().display().to_string(),
        "--json".into(),
        "--limit".into(),
        "100000".into(),
    ]);
    serde_json::from_str(&knowledge::find(&real_project(), &a).unwrap()).unwrap()
}
#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn defect_real_ascii_names_and_substrings() {
    let cases = [
        ("Anduril, Narsil Reforged", "Andúril, Narsil Reforged", "Anduril"),
        ("Barad-dur", "Barad-dûr", "Barad-dur"),
        ("Grima Wormtongue", "Gríma Wormtongue", "Grima"),
        ("Lim-Dul's Vault", "Lim-Dûl's Vault", "Lim-Dul"),
        ("Jotun Grunt", "Jötun Grunt", "Jotun"),
        ("Seance", "Séance", "Seance"),
        ("Urza’s Saga", "Urza's Saga", "Urza’s"),
        ("Æther Vial", "Aether Vial", "Æther"),
    ];
    let mut failures = vec![];
    for (query, expected, part) in cases {
        assert!(real_rows(&["--name", expected]).iter().any(|c| c["name"] == expected), "baseline {expected}");
        for (flag, q) in [("--name", query), ("--name-contains", part)] {
            let hits = real_rows(&[flag, q]);
            eprintln!("real {flag} {q:?}: {}", json!(hits.iter().map(|c| &c["name"]).collect::<Vec<_>>()));
            if !hits.iter().any(|c| c["name"] == expected) {
                failures.push(format!("{flag} {q:?} misses {expected}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn defect_real_deck_ascii_spelling_exclusion() {
    let idx = Index::load(&real_dir().join("oracle.json")).unwrap();
    let names = [
        "Andúril, Narsil Reforged",
        "Barad-dûr",
        "Gríma Wormtongue",
        "Lim-Dûl's Vault",
        "Jötun Grunt",
        "Séance",
        "Aether Vial",
        "Urza's Saga",
    ];
    let wanted: BTreeSet<_> = names.iter().map(|n| fold(n)).collect();
    let all: Value = serde_json::from_reader(File::open(real_dir().join("oracle.json")).unwrap()).unwrap();
    let cards: Vec<_> = all["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| wanted.contains(&fold(c["name"].as_str().unwrap())))
        .cloned()
        .collect();
    assert_eq!(cards.len(), names.len());
    for n in names {
        assert!(!idx.lookup(n).is_empty());
    }
    // Only deck spellings are synthesized; catalog records are copied unchanged from the real file.
    let e = Env::new(cards);
    e.decks(json!({"mine":[{"key":"ascii","ident":"WUBRG","cards":[
        "Anduril, Narsil Reforged", "Barad-dur", "Grima Wormtongue", "Lim-Dul's Vault",
        "Jotun Grunt", "Seance", "Æther Vial", "Urza’s Saga"]}]}));
    let leaked = e.names(&["--deck", "ascii", "--limit", "100000"]);
    assert!(leaked.is_empty(), "real catalog cards already in the deck leaked: {leaked:?}");
}
#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn real_mine_decks_have_no_accent_folded_overlap() {
    let decks: Value = serde_json::from_reader(File::open(real_project().data("decks.json")).unwrap()).unwrap();
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let ident = d["ident"].as_str().unwrap();
        let have: BTreeSet<_> = d["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| fold(s.as_str().unwrap()))
            .chain([fold(d["commander"].as_str().unwrap()), fold(d["commander2"].as_str().unwrap_or(""))])
            .collect();
        let rows = real_rows(&["--deck", key]);
        eprintln!("mine {key}: {} candidates, accent-folded overlaps checked", rows.len());
        for c in rows {
            assert!(!have.contains(&fold(c["name"].as_str().unwrap())), "{key}: leaked {}", c["name"]);
            assert!(c["identity"].as_str().unwrap().chars().all(|ch| ident.contains(ch)), "{key}: {c}");
        }
    }
}
#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn real_identity_mana_indicators_back_faces_and_shared_names() {
    for (name, identity) in [
        ("Command Tower", ""),
        ("Spectral Procession", "W"),
        ("Dismember", "B"),
        ("Archangel Avacyn // Avacyn, the Purifier", "WR"),
        ("Westvale Abbey // Ormendahl, Profane Prince", "B"),
        ("Transguild Courier", "WUBRG"),
        ("Dryad Arbor", "G"),
    ] {
        let c = real_rows(&["--name", name]);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0]["identity"], identity);
        assert_eq!(real_rows(&["--name", name, "--ident", identity]).len(), 1);
        for letter in identity.chars() {
            let missing = identity.replace(letter, "");
            assert!(real_rows(&["--name", name, "--ident", &missing]).is_empty(), "{name} missing {letter}");
        }
    }
    for name in ["Lightning Bolt", "Pick Your Poison", "Red Herring"] {
        let rows = real_rows(&["--name", name, "--legal", "any"]);
        assert_eq!(rows.len(), if name == "Lightning Bolt" { 1 } else { 2 });
        assert_eq!(rows[0]["legal"], "legal");
    }
    assert!(real_rows(&["--name", "Little Girl"]).is_empty());
    assert_eq!(real_rows(&["--name", "Little Girl", "--legal", "not_legal"])[0]["cmc"], 0.5);
    assert_eq!(real_rows(&["--name", "Brisela, Voice of Nightmares"]).len(), 1, "an exact name shows a meld result");
    assert_eq!(real_rows(&["--name", "Brisela, Voice of Nightmares", "--meld"]).len(), 1);
}
#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn real_catalog_output_and_scan_performance() {
    let dir = real_dir();
    let start = Instant::now();
    let idx = Index::load(&dir.join("oracle.json")).unwrap();
    eprintln!("real load: {:?}, {} cards", start.elapsed(), idx.cards.len());
    let q = parse(&[
        "--legal",
        "any",
        "--meld",
        "--text",
        r"(?<!un)counter target (?!noncreature)\w+",
        "--text",
        r"\bdraw\b",
    ])
    .unwrap();
    let regexes: Vec<_> = q.texts.iter().map(|s| fancy_regex::Regex::new(&format!("(?i){s}")).unwrap()).collect();
    let start = Instant::now();
    let hits = idx.matches(&q, None, &[], &regexes).unwrap();
    let elapsed = start.elapsed();
    eprintln!("real regex scan: {elapsed:?}, {} matches", hits.len());
    assert!(elapsed < Duration::from_secs(1));
    for args in [
        vec!["--count", "--legal", "any", "--meld"],
        vec!["--legal", "any", "--text", r"(?<!un)counter target (?!noncreature)\w+", "--text", r"\bdraw\b", "--count"],
        vec!["--legal", "any", "--meld", "--text", r"(?s)\A(?:(?!zzzzzzzzzzzzzzzz).)*\z", "--count"],
        vec!["--legal", "any", "--meld", "--limit", "100000", "--json"],
    ] {
        let mut c = Command::new("/usr/bin/time");
        c.env_remove("CARDGEN_KNOWLEDGE_DIR")
            .args(["-v", env!("CARGO_BIN_EXE_cardgen"), "--root"])
            .arg(&real_project().root)
            .arg("find")
            .args(&args)
            .arg("--data-dir")
            .arg(&dir);
        let o = bounded(c);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        eprintln!("real {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        assert_rss(&o.stderr);
        if args.contains(&"--json") {
            let rows: Vec<Value> = serde_json::from_slice(&o.stdout).expect("all-catalog JSON is valid");
            assert_eq!(rows.len(), idx.cards.len());
            assert_eq!(rows.len(), 35113);
        }
    }
}
