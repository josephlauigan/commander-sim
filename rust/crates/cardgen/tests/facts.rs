//! Phase 4: the engine model status, the decks that play a card and the `FINDINGS.md` lines that name it, joined to the
//! catalog (`cardgen::facts`) and shown by `find`. A hand-built catalog and project root cover deck membership,
//! findings matching and the `find` additions; the engine status runs the real `Audit` over the repository's own data
//! with a catalog built from the engine's card names (so nothing here names a card or a deck of JD's: the cards are
//! picked by their status). Nothing uses the network.

use cardgen::audit::{Audit, rank};
use cardgen::data::Project;
use cardgen::export::Inputs;
use cardgen::facts::{Facts, status_code};
use cardgen::knowledge::{self, FindError, Index};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn rec(name: &str, type_line: &str, text: &str, legal: &str) -> Value {
    json!({
        "name": name, "mana_cost": "{1}", "cmc": 1.0, "type_line": type_line, "oracle_text": text,
        "color_identity": [], "keywords": [], "layout": "normal", "game_changer": false,
        "legalities": {"commander": legal}, "oracle_id": format!("t-{name}-{legal}"),
        "released_at": "2026-01-01", "set": "tst", "rarity": "common", "reserved": false, "edhrec_rank": 100
    })
}

/// a multi-face record: the full name is the faces joined by ` // `
fn faces(faces: &[&str], layout: &str) -> Value {
    let mut c = rec(&faces.join(" // "), "Creature", "", "legal");
    c["layout"] = json!(layout);
    c["card_faces"] = json!(faces.iter().map(|f| json!({"name": f, "type_line": "Creature"})).collect::<Vec<_>>());
    c
}

/// a temporary project root with a catalog, `data/decks.json` and `audit/<key>/FINDINGS.md` files
struct Env {
    root: PathBuf,
}

impl Env {
    fn new(cards: Vec<Value>) -> Env {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "facts-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(root.join("data/knowledge")).unwrap();
        let meta =
            json!({"schema": 1, "bulk_updated_at": "x", "download_uri": "x", "synced": "x", "count": cards.len()});
        std::fs::write(
            root.join("data/knowledge/oracle.json"),
            serde_json::to_string(&json!({"meta": meta, "cards": cards})).unwrap(),
        )
        .unwrap();
        Env { root }
    }

    fn decks(&self, mine: Value) {
        std::fs::write(self.root.join("data/decks.json"), json!({"version": 1, "mine": mine, "pool": []}).to_string())
            .unwrap();
    }

    fn findings(&self, key: &str, text: &str) {
        std::fs::create_dir_all(self.root.join("audit").join(key)).unwrap();
        std::fs::write(self.root.join("audit").join(key).join("FINDINGS.md"), text).unwrap();
    }

    fn project(&self) -> Project {
        Project::new(&self.root)
    }

    fn index(&self) -> Index {
        Index::load(&self.root.join("data/knowledge/oracle.json")).unwrap()
    }

    fn facts(&self) -> (Index, Facts) {
        let index = self.index();
        let facts = Facts::decks_and_findings(&self.project(), &index).unwrap();
        (index, facts)
    }

    fn find(&self, args: &[&str]) -> Result<String, FindError> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.push("--data-dir".into());
        a.push(self.root.join("data/knowledge").display().to_string());
        knowledge::find(&self.project(), &a)
    }

    fn ok(&self, args: &[&str]) -> String {
        self.find(args).unwrap_or_else(|e| panic!("find {args:?}: {e}"))
    }

    fn json(&self, args: &[&str]) -> Vec<Value> {
        let mut a = args.to_vec();
        a.push("--json");
        serde_json::from_str::<Value>(&self.ok(&a)).unwrap().as_array().unwrap().clone()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn usage(r: Result<String, FindError>) -> String {
    match r {
        Err(FindError::Usage(m)) => m,
        other => panic!("expected a usage error, got {other:?}"),
    }
}

fn names(rows: &[Value]) -> Vec<String> {
    rows.iter().map(|r| r["name"].as_str().unwrap().to_string()).collect()
}

// ------------------------------------------------------------------------------------------------ findings

fn findings_catalog() -> Vec<Value> {
    vec![
        rec("Opt", "Instant", "Scry 1.", "legal"),
        rec("Option", "Instant", "Draw.", "legal"),
        rec("Sol Ring", "Artifact", "{T}: Add {C}{C}.", "legal"),
        rec("Wrath of God", "Sorcery", "Destroy all creatures.", "legal"),
        faces(&["Fire", "Ice"], "split"),
        faces(&["Bonecrusher Giant", "Stomp"], "adventure"),
        faces(&["Delver of Secrets", "Insectile Aberration"], "transform"),
        faces(&["Emeritus of Conflict", "Lightning Bolt"], "prepare"),
        rec("Lightning Bolt", "Instant", "3 damage.", "legal"),
        rec("Andúril, Narsil Reforged", "Legendary Artifact", "", "legal"),
        rec("Urza's Saga", "Land", "", "legal"),
        rec("Pick Your Poison", "Sorcery", "Shared.", "legal"),
        rec("Pick Your Poison", "Sorcery", "Shared.", "not_legal"),
        rec("Island", "Basic Land — Island", "", "legal"),
        rec("Plains", "Basic Land — Plains", "", "legal"),
    ]
}

/// (line of alpha's FINDINGS.md, the catalog names that line mentions)
fn alpha_lines() -> Vec<(String, Vec<&'static str>)> {
    let long = format!("{} Sol Ring", "x".repeat(300));
    let rows: Vec<(String, Vec<&str>)> = vec![
        ("# Alpha findings".into(), vec![]),
        ("".into(), vec![]),
        ("## Swaps".into(), vec![]),
        ("Option is not Opt.".into(), vec!["Option"]),
        ("**Opt** and `Option`".into(), vec!["Opt", "Option"]),
        ("Cut Opt => Consider".into(), vec!["Opt"]),
        ("Sol Ring's text, not sol ring, not Sol Rings, not SolRing".into(), vec!["Sol Ring"]),
        ("sol ring only".into(), vec![]),
        ("Wrath of Godzilla".into(), vec![]),
        ("(Wrath of God)".into(), vec!["Wrath of God"]),
        ("### Fire // Ice and Fire".into(), vec!["Fire // Ice"]),
        ("Plain Ice and Fire".into(), vec![]),
        ("a **Ice** card".into(), vec!["Fire // Ice"]),
        ("Stomp the board".into(), vec!["Bonecrusher Giant // Stomp"]),
        ("Bonecrusher Giant is big".into(), vec!["Bonecrusher Giant // Stomp"]),
        ("Anduril, Narsil Reforged shines".into(), vec!["Andúril, Narsil Reforged"]),
        ("Urza’s Saga is a land".into(), vec!["Urza's Saga"]),
        ("Lightning Bolt hits".into(), vec!["Lightning Bolt"]),
        // the longest name wins: the `Lightning Bolt` inside it is not a second mention
        ("Emeritus of Conflict // Lightning Bolt is a creature".into(), vec!["Emeritus of Conflict // Lightning Bolt"]),
        ("Delver of Secrets flips".into(), vec!["Delver of Secrets // Insectile Aberration"]),
        ("Insectile Aberration".into(), vec!["Delver of Secrets // Insectile Aberration"]),
        ("Island and Plains".into(), vec!["Island", "Plains"]),
        (long, vec!["Sol Ring"]),
        ("Sol Ring Sol Ring Sol Ring".into(), vec!["Sol Ring"]),
    ];
    rows
}

fn alpha_text() -> String {
    alpha_lines().iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n") + "\n"
}

const BETA: &str = "Sol Ring before any heading\n## Beta heading\n## Sol Ring => Mox Opal\nPick Your Poison and Opt\n";

fn findings_env() -> Env {
    let env = Env::new(findings_catalog());
    env.decks(json!([
        {"key": "alpha", "commander": "Wrath of God", "ident": "", "cards": ["Island"]},
        {"key": "beta", "commander": "Wrath of God", "ident": "", "cards": ["Island"]},
        {"key": "nofile", "commander": "Wrath of God", "ident": "", "cards": ["Island"]},
    ]));
    env.findings("alpha", &alpha_text());
    env.findings("beta", BETA);
    env
}

fn finding_lines(facts: &Facts, index: &Index, name: &str) -> Vec<(String, usize)> {
    let ids = index.lookup(name);
    assert_eq!(ids.len(), 1, "{name}");
    facts.findings_of(ids[0]).iter().map(|f| (f.file.clone(), f.line)).collect()
}

#[test]
fn a_line_mentions_a_card_by_a_word_bounded_case_sensitive_name_and_short_names_need_markup() {
    let env = findings_env();
    let (index, facts) = env.facts();
    let alpha = "audit/alpha/FINDINGS.md".to_string();
    for c in &index.cards {
        let name = c.raw.name.as_str();
        if name == "Pick Your Poison" {
            continue;
        }
        let want: Vec<(String, usize)> = alpha_lines()
            .iter()
            .enumerate()
            .filter(|(_, (_, who))| who.contains(&name))
            .map(|(n, _)| (alpha.clone(), n + 1))
            .collect();
        let got: Vec<(String, usize)> =
            finding_lines(&facts, &index, name).into_iter().filter(|(f, _)| *f == alpha).collect();
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn opt_never_matches_option_and_a_name_is_counted_once_per_line() {
    let env = findings_env();
    let (index, facts) = env.facts();
    // line 4 is `Option is not Opt.`: Option, not Opt; line 5 has both in markup
    let opt: Vec<usize> = finding_lines(&facts, &index, "Opt").iter().map(|(_, l)| *l).collect();
    assert!(!opt.contains(&4) && opt.contains(&5) && opt.contains(&6), "{opt:?}");
    // `Sol Ring Sol Ring Sol Ring` is one finding, and the long line one, whatever the distance
    let sol = finding_lines(&facts, &index, "Sol Ring");
    assert_eq!(sol.iter().filter(|(f, l)| f.contains("alpha") && *l == 24).count(), 1, "{sol:?}");
    // the text is cut to 200 characters; the mention past the cut still counts
    let long = facts.findings_of(index.lookup("Sol Ring")[0]).iter().find(|f| f.line == 23).unwrap();
    assert_eq!(long.text.chars().count(), 200);
}

#[test]
fn findings_carry_file_line_and_the_nearest_preceding_heading_and_k_counts_distinct_lines_across_decks() {
    let env = findings_env();
    let (index, facts) = env.facts();
    assert_eq!(facts.files, ["audit/alpha/FINDINGS.md", "audit/beta/FINDINGS.md"], "a deck with no file is skipped");
    let sol = facts.findings_of(index.lookup("Sol Ring")[0]);
    let first = &sol[0];
    assert_eq!((first.file.as_str(), first.line), ("audit/alpha/FINDINGS.md", 7));
    assert_eq!(first.heading, "## Swaps");
    assert!(first.text.starts_with("Sol Ring's text"));
    // beta: a mention before any heading has an empty heading; a heading line that mentions the card names the one
    // before it, not itself
    let in_beta: Vec<_> = sol.iter().filter(|f| f.file.contains("beta")).collect();
    assert_eq!(
        in_beta.iter().map(|f| (f.line, f.heading.as_str())).collect::<Vec<_>>(),
        [(1, ""), (3, "## Beta heading")]
    );
    // k:N is the number of distinct lines over every file
    assert_eq!(sol.len(), 3 + 2);
    assert_eq!(facts.context(None).findings[&index.lookup("Sol Ring")[0]], sol.len());
    // a card no line names has no entry
    assert_eq!(facts.findings_of(index.lookup("Option")[0]).len(), 2);
    let ctx = facts.context(None);
    assert_eq!(ctx.findings[&index.lookup("Plains")[0]], 1);
    assert!(ctx.findings.contains_key(&index.lookup("Sol Ring")[0]));
}

#[test]
fn a_shared_name_goes_to_the_legal_card_and_an_exact_name_beats_a_face_name() {
    let env = findings_env();
    let (index, facts) = env.facts();
    let shared = index.lookup("Pick Your Poison");
    assert_eq!(shared.len(), 2);
    let legal = *shared.iter().find(|&&i| index.cards[i as usize].raw.oracle_id.ends_with("-legal")).unwrap();
    let other = *shared.iter().find(|&&i| i != legal).unwrap();
    assert_eq!(facts.findings_of(legal).len(), 1);
    assert!(facts.findings_of(other).is_empty());
    // `Lightning Bolt` is the instant; the prepare card that has it as a face is named only by its full or other name
    let prepare = index.lookup("Emeritus of Conflict")[0];
    let lines: Vec<usize> =
        facts.findings_of(prepare).iter().filter(|f| f.file.contains("alpha")).map(|f| f.line).collect();
    assert_eq!(lines, [19], "only its own full name");
}

#[test]
fn a_project_with_no_findings_files_has_none_and_a_non_utf8_file_is_an_error() {
    let env = Env::new(findings_catalog());
    env.decks(json!([{"key": "alpha", "commander": "Opt", "ident": "", "cards": []}]));
    let (_, facts) = env.facts();
    assert!(facts.findings.is_empty() && facts.files.is_empty());
    std::fs::create_dir_all(env.root.join("audit/alpha")).unwrap();
    std::fs::write(env.root.join("audit/alpha/FINDINGS.md"), [0xff, 0xfe, 0x41]).unwrap();
    assert!(Facts::decks_and_findings(&env.project(), &env.index()).unwrap_err().contains("FINDINGS.md"));
}

// ------------------------------------------------------------------------------------------------ decks

fn deck_env() -> Env {
    let env = Env::new(findings_catalog());
    env.decks(json!([
        {"key": "alpha", "commander": "Urza’s Saga", "ident": "", "cards": [
            "Island", "Island", "Anduril, Narsil Reforged", "Delver of Secrets"]},
        {"key": "beta", "commander": "Wrath of God", "commander2": "Opt", "ident": "",
         "cards": ["Island", "Lightning Bolt", "Pick Your Poison", "Not In Catalog"]},
        {"key": "gamma", "commander": "Plains", "ident": "", "cards": []},
    ]));
    env
}

#[test]
fn deck_membership_is_a_bit_per_deck_by_folded_name_face_name_commander_and_basic_land() {
    let env = deck_env();
    let (index, facts) = env.facts();
    assert_eq!(facts.keys, ["alpha", "beta", "gamma"]);
    assert_eq!(facts.letters.iter().map(|(c, _)| *c).collect::<String>(), "ABG");
    let bits = |name: &str| facts.deck_bits[index.lookup(name)[0] as usize];
    assert_eq!(bits("Island"), 0b011, "a basic land, twice in a list");
    assert_eq!(bits("Andúril, Narsil Reforged"), 0b001, "a spelling without the accent");
    assert_eq!(bits("Delver of Secrets // Insectile Aberration"), 0b001, "a front-face name");
    assert_eq!(bits("Urza's Saga"), 0b001, "a commander, curly apostrophe");
    assert_eq!(bits("Opt"), 0b010, "the second commander");
    assert_eq!(bits("Wrath of God"), 0b010);
    assert_eq!(bits("Plains"), 0b100);
    assert_eq!(bits("Sol Ring"), 0);
    // a shared name marks the Commander-legal card only
    let shared = index.lookup("Pick Your Poison");
    let by_legal: Vec<(bool, u64)> = shared
        .iter()
        .map(|&i| (index.cards[i as usize].raw.oracle_id.ends_with("-legal"), facts.deck_bits[i as usize]))
        .collect();
    assert!(by_legal.contains(&(true, 0b010)) && by_legal.contains(&(false, 0)), "{by_legal:?}");
    // the exact name beats a face name: the deck's `Lightning Bolt` is the instant, not the prepare card
    assert_eq!(bits("Lightning Bolt"), 0b010);
    assert_eq!(bits("Emeritus of Conflict // Lightning Bolt"), 0);
    assert_eq!(facts.decks_of(index.lookup("Island")[0]), ["alpha", "beta"]);
}

#[test]
fn the_context_leaves_the_current_deck_out_and_knows_nothing_of_a_pool_deck() {
    let env = deck_env();
    let (index, facts) = env.facts();
    let island = index.lookup("Island")[0];
    let ctx = facts.context(Some("beta"));
    assert_eq!(ctx.current, Some(0b010));
    assert_eq!(ctx.deck_letters, facts.letters);
    assert_eq!(ctx.deck_bits[island as usize], 0b011);
    assert!(ctx.model_status.is_empty(), "no engine data in this project: no status");
    assert_eq!(facts.context(Some("not-a-deck")).current, None);
    assert_eq!(facts.context(None).current, None);
    // the flag of Island in beta's own digest: the other deck only
    let card = &index.cards[island as usize];
    let f = cardgen::digest::flags(island, card, cardgen::digest::Review::None, true, &ctx);
    assert!(f.contains(&"d:A".to_string()), "{f:?}");
}

#[test]
fn an_unknown_deck_key_lists_the_keys_and_matching_ignores_case() {
    let env = deck_env();
    let (_, facts) = env.facts();
    assert_eq!(facts.deck_bit("BETA").unwrap(), 0b010);
    match facts.deck_bit("nope") {
        Err(FindError::Usage(m)) => assert!(m.contains("alpha, beta, gamma"), "{m}"),
        other => panic!("{other:?}"),
    }
}

// ------------------------------------------------------------------------------------------------ find

#[test]
fn every_json_row_has_decks_and_findings_arrays_and_no_status_without_an_engine_record() {
    let env = findings_env();
    let rows = env.json(&["--legal", "any", "--limit", "all"]);
    assert_eq!(rows.len(), findings_catalog().len());
    for r in &rows {
        assert!(r["decks"].is_array() && r["findings"].is_array(), "{r}");
        assert!(r.get("status").is_none() && r.get("status_note").is_none(), "{r}");
        assert_eq!(r["engine"], "no-record", "{r}");
    }
    let island = rows.iter().find(|r| r["name"] == "Island").unwrap();
    assert_eq!(island["decks"], json!(["alpha", "beta", "nofile"]));
    assert_eq!(island["findings"].as_array().unwrap().len(), 1);
    let f = &island["findings"][0];
    assert_eq!((f["file"].as_str().unwrap(), f["line"].as_u64().unwrap()), ("audit/alpha/FINDINGS.md", 22));
    assert_eq!(f["heading"], "### Fire // Ice and Fire", "a heading line above, whatever its level");
    assert_eq!(f["text"], "Island and Plains");
    // the old keys are all still there
    for k in ["name", "oracle_id", "layout", "mana_cost", "cmc", "type_line", "identity", "legal", "gc", "oracle_text"]
    {
        assert!(island.get(k).is_some(), "{k}");
    }
}

#[test]
fn full_prints_the_status_decks_and_each_finding() {
    let env = findings_env();
    let out = env.ok(&["--name", "Sol Ring", "--full"]);
    assert!(out.contains("engine status: NO RECORD (not in data/cards.json: unknown, treat as below Full)"), "{out}");
    assert!(out.contains("decks: none"), "{out}");
    assert!(
        out.contains("findings (5):") && out.contains("audit/alpha/FINDINGS.md:7  [## Swaps]  Sol Ring's text"),
        "{out}"
    );
    let out = env.ok(&["--name", "Island", "--full"]);
    assert!(out.contains("decks: alpha, beta, nofile"), "{out}");
    assert!(env.ok(&["--name", "Option", "--full"]).contains("findings (2):"));
    let none = env.ok(&["--name", "Plains", "--full"]);
    assert!(none.contains("findings (1):"), "{none}");
}

#[test]
fn in_deck_and_not_in_deck_filter_by_a_deck_of_mine() {
    let env = deck_env();
    let set = |args: &[&str]| -> BTreeSet<String> { names(&env.json(args)).into_iter().collect() };
    let alpha = set(&["--in-deck", "alpha", "--limit", "all"]);
    assert_eq!(
        alpha,
        ["Island", "Andúril, Narsil Reforged", "Delver of Secrets // Insectile Aberration", "Urza's Saga"]
            .map(String::from)
            .into_iter()
            .collect()
    );
    assert_eq!(set(&["--in-deck", "ALPHA", "--limit", "all"]), alpha, "the key ignores case");
    let not_alpha = set(&["--not-in-deck", "alpha", "--limit", "all"]);
    assert!(not_alpha.is_disjoint(&alpha) && not_alpha.contains("Sol Ring") && not_alpha.contains("Opt"));
    assert_eq!(set(&["--in-deck", "beta", "--not-in-deck", "alpha", "--limit", "all"]).contains("Island"), false);
    assert!(set(&["--in-deck", "beta", "--not-in-deck", "alpha", "--limit", "all"]).contains("Opt"));
    assert_eq!(env.ok(&["--in-deck", "alpha", "--count"]).trim(), "4");
    // an unknown key, a pool deck and a missing value are usage errors that name the keys
    let e = usage(env.find(&["--in-deck", "nope"]));
    assert!(e.contains("alpha, beta, gamma"), "{e}");
    assert!(usage(env.find(&["--not-in-deck", "nope", "--count"])).contains("alpha, beta, gamma"));
    assert!(usage(env.find(&["--in-deck"])).contains("needs a value"));
    assert!(usage(env.find(&["--in-deck", "alpha", "--in-deck", "beta"])).contains("given twice"));
    // a named card the filter removed is explained
    let out = env.ok(&["--name", "Island", "--not-in-deck", "alpha"]);
    assert!(out.contains("no matches") && out.contains("in deck alpha (--not-in-deck)"), "{out}");
    let out = env.ok(&["--name", "Sol Ring", "--in-deck", "beta"]);
    assert!(out.contains("not in deck beta"), "{out}");
}

#[test]
fn status_takes_a_comma_separated_list_of_known_words_and_none_means_no_engine_record() {
    let env = deck_env();
    let all = findings_catalog().len();
    // no engine data in this project: every card has no record
    assert_eq!(env.ok(&["--status", "none", "--legal", "any", "--count"]).trim(), (all).to_string());
    assert_eq!(env.ok(&["--status", "no-record", "--legal", "any", "--count"]).trim(), (all).to_string());
    assert_eq!(env.ok(&["--status", "No-Record,full", "--legal", "any", "--count"]).trim(), (all).to_string());
    assert_eq!(env.ok(&["--status", "NONE, full", "--legal", "any", "--count"]).trim(), (all).to_string());
    for word in ["full", "full-auto", "approximate", "partial", "unmodeled", "Full,Full-Auto", "partial,unmodeled"] {
        assert_eq!(env.ok(&["--status", word, "--legal", "any", "--count"]).trim(), "0", "{word}");
    }
    for bad in ["bogus", "", "full,,none", "full;none", "modeled"] {
        let e = usage(env.find(&["--status", bad]));
        assert!(
            e.contains("--status takes") && e.contains("full-auto") && e.contains("no-record") && e.contains("none"),
            "{bad}: {e}"
        );
    }
    assert!(usage(env.find(&["--status"])).contains("needs a value"));
    assert!(usage(env.find(&["--status", "full", "--status", "none"])).contains("given twice"));
    let out = env.ok(&["--name", "Sol Ring", "--status", "full"]);
    assert!(out.contains("engine status no-record"), "{out}");
}

// ------------------------------------------------------------------------------------------------ the engine

/// (status, note) of every engine card as `cardgen audit` computes it, and the names in a deck of `mine`
fn engine() -> (BTreeMap<String, (String, String)>, BTreeSet<String>) {
    let inp = Inputs::load(&Project::new(repo())).unwrap();
    let built = inp.build();
    let audit = Audit::new(&inp, &built.cards).unwrap();
    let mut mine: BTreeSet<String> = inp.my_cards.iter().flat_map(|(_, c)| c.iter().cloned()).collect();
    for d in &inp.my {
        mine.insert(d.commander.clone());
        mine.extend(d.commander2.clone());
    }
    let status = built.cards.keys().map(|n| (n.clone(), audit.card_status(n, mine.contains(n)))).collect();
    (status, mine)
}

/// the first plain-ASCII name of each status, in name order, that has no " // " and contains an `e`
fn picks(status: &BTreeMap<String, (String, String)>) -> BTreeMap<&'static str, String> {
    let mut out = BTreeMap::new();
    for want in ["Full", "Full-auto", "Approximate", "Partial", "Unmodeled"] {
        if let Some((n, _)) = status
            .iter()
            .find(|(n, (s, _))| s == want && n.is_ascii() && !n.contains(" // ") && n.contains('e') && !n.contains('"'))
        {
            out.insert(want, n.clone());
        }
    }
    out
}

fn engine_env(status: &BTreeMap<String, (String, String)>, picked: &BTreeMap<&'static str, String>) -> Env {
    let mut cards: Vec<Value> = picked.values().map(|n| rec(n, "Artifact", "x", "legal")).collect();
    cards.push(rec("Zzz Not In The Engine", "Artifact", "x", "legal"));
    assert!(!status.contains_key("Zzz Not In The Engine"));
    Env::new(cards)
}

#[test]
fn the_status_is_the_audits_and_a_card_with_no_record_has_none() {
    let (status, _) = engine();
    let picked = picks(&status);
    for need in ["Full", "Partial", "Unmodeled"] {
        assert!(picked.contains_key(need), "the engine data has no {need} card to test with: {picked:?}");
    }
    let env = engine_env(&status, &picked);
    let index = env.index();
    // the real project (its audit and decks) with the fixture catalog
    let facts = Facts::load(&Project::new(repo()), &index, true, false).unwrap();
    for (want, name) in &picked {
        let i = index.lookup(name)[0];
        let (s, note) = facts.status_of(i).unwrap_or_else(|| panic!("{name} has no status"));
        assert_eq!((s, note), (status[name].0.as_str(), status[name].1.as_str()), "{name}");
        assert_eq!(s, *want);
    }
    let none = index.lookup("Zzz Not In The Engine")[0];
    assert!(facts.status_of(none).is_none() && facts.context(None).model_status.get(&none).is_none());
    assert!(facts.engine.unresolved.len() + picked.len() <= facts.engine.records);
    assert!(!facts.engine.unresolved.iter().any(|n| picked.values().any(|p| p == n)));
    // the digest letters
    for (want, name) in &picked {
        let code = facts.context(None).model_status[&index.lookup(name)[0]].clone();
        assert_eq!(code, status_code(want));
        assert_eq!(code.len(), 1);
    }
    // never `Full` for a card with no record: no status at all, in the JSON row and in `--status`
    let find = |args: &[&str]| -> Vec<Value> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.extend(["--json".into(), "--data-dir".into(), env.root.join("data/knowledge").display().to_string()]);
        serde_json::from_str::<Value>(&knowledge::find(&Project::new(repo()), &a).unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .clone()
    };
    let row = &find(&["--name", "Zzz Not In The Engine"])[0];
    assert!(row.get("status").is_none() && row.get("status_note").is_none(), "{row}");
    assert_eq!(names(&find(&["--status", "none"])), ["Zzz Not In The Engine"]);
    for (want, name) in &picked {
        let row = &find(&["--name", name])[0];
        assert_eq!(row["status"], *want, "{row}");
        assert_eq!(row["status_note"], status[name].1.as_str(), "{row}");
        // `--status` takes the word in any case and keeps exactly the cards of that status
        let hits = names(&find(&["--status", &want.to_uppercase()]));
        assert_eq!(hits, [name.clone()], "{want}");
    }
    // full,full-auto is the safe-to-propose set
    let safe: BTreeSet<String> = names(&find(&["--status", "full,full-auto"])).into_iter().collect();
    let want: BTreeSet<String> = ["Full", "Full-auto"].iter().filter_map(|k| picked.get(k).cloned()).collect();
    assert_eq!(safe, want);
    let below: BTreeSet<String> = names(&find(&["--status", "approximate,partial,unmodeled"])).into_iter().collect();
    assert!(below.is_disjoint(&safe) && !below.contains("Zzz Not In The Engine"));
}

#[test]
fn an_engine_name_resolves_by_folded_name_face_name_and_the_legal_card_of_a_shared_name() {
    let (status, _) = engine();
    let picked = picks(&status);
    let [a, b, c] =
        ["Full", "Partial", "Unmodeled"].map(|k| picked.get(k).unwrap_or_else(|| panic!("no {k} card")).clone());
    // accent: the catalog spells the first `e` as `é`; DFC: the engine name is the front face; shared: a legal and a
    // not-legal card with one name
    let accented = a.replacen('e', "é", 1);
    let dfc_name = format!("{b} // Zzz Reverse Face");
    let dfc = {
        let mut d = rec(&dfc_name, "Creature", "", "legal");
        d["layout"] = json!("transform");
        d["card_faces"] =
            json!([{"name": b, "type_line": "Creature"}, {"name": "Zzz Reverse Face", "type_line": "Creature"}]);
        d
    };
    let env = Env::new(vec![rec(&accented, "Artifact", "x", "legal"), dfc, rec(&c, "Artifact", "x", "not_legal"), {
        let mut r = rec(&c, "Artifact", "x", "legal");
        r["oracle_id"] = json!("t-shared-legal-card");
        r
    }]);
    let index = env.index();
    let facts = Facts::load(&Project::new(repo()), &index, true, false).unwrap();
    let st = |i: u32| facts.status_of(i).map(|(s, n)| (s.to_string(), n.to_string()));
    assert_eq!(st(index.lookup(&a)[0]), Some(status[&a].clone()), "accent {accented}");
    assert_eq!(st(index.lookup(&dfc_name)[0]), Some(status[&b].clone()), "front face of {dfc_name}");
    let shared = index.lookup(&c);
    assert_eq!(shared.len(), 2);
    for &i in shared {
        let legal = index.cards[i as usize].raw.legalities.commander.as_deref() == Some("legal");
        assert_eq!(st(i).is_some(), legal, "only the legal card of a shared name gets the status");
    }
    // the names with no catalog card are reported, never matched by guesswork
    assert!(facts.engine.unresolved.len() > 100 && !facts.engine.unresolved.contains(&a));
}

#[test]
fn a_card_of_a_deck_of_mine_is_audited_as_mine() {
    let (status, mine) = engine();
    // a card whose verdict depends on the flag, when there is one
    let inp = Inputs::load(&Project::new(repo())).unwrap();
    let built = inp.build();
    let audit = Audit::new(&inp, &built.cards).unwrap();
    let differs: Vec<&String> = mine
        .iter()
        .filter(|n| built.cards.contains_key(*n) && audit.card_status(n, true) != audit.card_status(n, false))
        .collect();
    eprintln!("cards of mine whose status depends on the flag: {}", differs.len());
    let Some(name) = differs.first() else { return };
    let env = Env::new(vec![rec(name, "Artifact", "x", "legal")]);
    let index = env.index();
    let facts = Facts::load(&Project::new(repo()), &index, true, false).unwrap();
    assert_eq!(
        facts.status_of(index.lookup(name)[0]).map(|(s, n)| (s.to_string(), n.to_string())),
        Some(status[*name].clone())
    );
    assert_ne!(status[*name], audit.card_status(name, false));
}

#[test]
fn two_engine_records_for_one_card_show_the_lower_status_and_the_conflict_is_listed() {
    let (status, _) = engine();
    // a double-faced card the engine has under both its full name and its front face, with different statuses
    let pair = status.keys().find_map(|full| {
        let (front, back) = full.split_once(" // ")?;
        let other = status.get(front)?;
        (rank(&status[full].0) != rank(&other.0)).then(|| (full.clone(), front.to_string(), back.to_string()))
    });
    let Some((full, front, back)) = pair else {
        eprintln!("no double-faced card has two engine records with different statuses today");
        return;
    };
    let env = Env::new(vec![faces(&[front.as_str(), back.as_str()], "transform")]);
    let index = env.index();
    assert_eq!(index.cards[0].raw.name, full);
    let facts = Facts::load(&Project::new(repo()), &index, true, false).unwrap();
    let lower = if rank(&status[&full].0) < rank(&status[&front].0) { &status[&full] } else { &status[&front] };
    assert_eq!(facts.status_of(0).map(|(s, n)| (s.to_string(), n.to_string())), Some(lower.clone()));
    assert!(
        facts.engine.conflicts.iter().any(|c| c.starts_with(&format!("{full}: kept "))),
        "{:?}",
        facts.engine.conflicts
    );
}

// ------------------------------------------------------------------------------------------------ many decks

fn many_decks(n: usize) -> Env {
    let env = Env::new(vec![rec("Sol Ring", "Artifact", "x", "legal")]);
    env.decks(json!(
        (0..n)
            .map(|i| json!({"key": format!("deck-{i}"), "commander": "", "ident": "", "cards": ["Sol Ring"]}))
            .collect::<Vec<_>>()
    ));
    env.findings(&format!("deck-{}", n - 1), "Sol Ring in the last deck\n");
    env
}

#[test]
fn twenty_eight_decks_keep_every_bit_finding_filter_and_a_distinct_letter() {
    let env = many_decks(28);
    let (index, facts) = env.facts();
    assert_eq!(facts.keys.len(), 28);
    let letters: BTreeSet<char> = facts.letters.iter().map(|(c, _)| *c).collect();
    assert_eq!(letters.len(), 28, "distinct letters");
    assert!(letters.iter().all(|c| c.is_ascii_alphabetic()) && letters.iter().any(|c| c.is_ascii_lowercase()));
    assert_eq!(facts.letters, env.facts().1.letters, "deterministic");
    let sol = index.lookup("Sol Ring")[0];
    assert_eq!(facts.decks_of(sol).len(), 28);
    assert_eq!(facts.findings_of(sol).len(), 1);
    let rows = env.json(&[]);
    assert_eq!(rows[0]["decks"].as_array().unwrap().len(), 28);
    assert_eq!(rows[0]["findings"].as_array().unwrap().len(), 1);
    assert_eq!(env.ok(&["--in-deck", "deck-27", "--count"]).trim(), "1");
    assert_eq!(env.ok(&["--not-in-deck", "deck-27", "--count"]).trim(), "0");
    // the digest flag names all 28 decks
    let ctx = facts.context_checked(None).unwrap();
    let f = cardgen::digest::flags(sol, &index.cards[sol as usize], cardgen::digest::Review::None, false, &ctx);
    let d = f.iter().find(|x| x.starts_with("d:")).unwrap();
    assert_eq!(d.len() - 2, 28, "{d}");
}

#[test]
fn fifty_three_decks_work_in_find_and_are_a_clear_error_for_the_digest_letters() {
    let env = many_decks(53);
    let (index, facts) = env.facts();
    assert_eq!(facts.keys.len(), 53);
    let sol = index.lookup("Sol Ring")[0];
    assert_eq!(facts.decks_of(sol).len(), 53);
    assert_eq!(facts.findings_of(sol).len(), 1);
    assert_eq!(env.json(&[])[0]["decks"].as_array().unwrap().len(), 53);
    assert_eq!(env.ok(&["--in-deck", "deck-52", "--count"]).trim(), "1");
    match facts.context_checked(None) {
        Err(FindError::Failed(m)) => assert!(m.contains("53 decks") && m.contains("52"), "{m}"),
        other => panic!("{other:?}"),
    }
    // 52 decks are the limit of the letters: fine
    let (_, f52) = many_decks(52).facts();
    let letters: BTreeSet<char> = f52.letters.iter().map(|(c, _)| *c).collect();
    assert_eq!(letters.len(), 52);
    assert!(f52.context_checked(None).is_ok());
}

#[test]
fn sixty_four_decks_are_the_limit_of_the_bitmask() {
    let env = many_decks(64);
    let (index, facts) = env.facts();
    assert_eq!(facts.decks_of(index.lookup("Sol Ring")[0]).len(), 64);
    assert_eq!(env.ok(&["--in-deck", "deck-63", "--count"]).trim(), "1");
    let over = many_decks(65);
    let e = Facts::decks_and_findings(&over.project(), &over.index()).unwrap_err();
    assert!(e.contains("65 decks") && e.contains("64"), "{e}");
}

#[test]
fn a_backtick_run_closes_at_a_run_of_the_same_length_a_bom_is_skipped_and_text_keeps_its_spaces() {
    let env = Env::new(vec![rec("Opt", "Instant", "x", "legal"), rec("Sol Ring", "Artifact", "x", "legal")]);
    env.decks(json!([{"key": "alpha", "commander": "", "ident": "", "cards": []}]));
    env.findings(
        "alpha",
        "\u{feff}# Top\n`Opt` a\n``Opt`` b\n```Opt``` c\n``a `q` Opt`` d\n`` Opt ` e\n``Opt` f\nSol Ring  \n",
    );
    let (index, facts) = env.facts();
    let lines: Vec<usize> = facts.findings_of(index.lookup("Opt")[0]).iter().map(|f| f.line).collect();
    // the last two have a run with no closing run of its length: no code span, so a short name does not count
    assert_eq!(lines, [2, 3, 4, 5], "{lines:?}");
    let sol = &facts.findings_of(index.lookup("Sol Ring")[0])[0];
    assert_eq!((sol.line, sol.heading.as_str(), sol.text.as_str()), (8, "# Top", "Sol Ring  "));
}

#[test]
fn the_longest_name_wins_at_a_position_so_a_longer_name_does_not_also_mention_a_face() {
    let env = Env::new(vec![
        rec("Sheoldred, the Apocalypse", "Creature", "x", "legal"),
        rec("Sheoldred's Edict", "Instant", "x", "legal"),
        faces(&["Sheoldred", "The True Scriptures"], "transform"),
        rec("Silence", "Instant", "x", "legal"),
        rec("Bound in Silence", "Enchantment", "x", "legal"),
    ]);
    env.decks(json!([{"key": "alpha", "commander": "", "ident": "", "cards": []}]));
    env.findings(
        "alpha",
        "Sheoldred, the Apocalypse is great\nSheoldred's Edict hits\nSheoldred alone\nBound in Silence and Silence\n",
    );
    let (index, facts) = env.facts();
    let lines =
        |name: &str| -> Vec<usize> { facts.findings_of(index.lookup(name)[0]).iter().map(|f| f.line).collect() };
    assert_eq!(lines("Sheoldred, the Apocalypse"), [1]);
    assert_eq!(lines("Sheoldred's Edict"), [2]);
    assert_eq!(lines("Sheoldred // The True Scriptures"), [3], "a bare name still mentions the double-faced card");
    assert_eq!(lines("Bound in Silence"), [4]);
    assert_eq!(lines("Silence"), [4], "the second, separate Silence on line 4 counts once");
}

#[test]
fn a_row_says_record_no_record_or_unavailable_and_full_says_unavailable_not_none() {
    let (status, _) = engine();
    let picked = picks(&status);
    let env = engine_env(&status, &picked);
    let find = |args: &[&str]| -> Vec<Value> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.extend(["--json".into(), "--data-dir".into(), env.root.join("data/knowledge").display().to_string()]);
        serde_json::from_str::<Value>(&knowledge::find(&Project::new(repo()), &a).unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .clone()
    };
    assert_eq!(find(&["--name", "Zzz Not In The Engine"])[0]["engine"], "no-record");
    assert_eq!(find(&["--name", &picked["Full"]])[0]["engine"], "record");
    // Full-auto is worded as what it is
    if let Some(n) = picked.get("Full-auto") {
        let mut a: Vec<String> = ["--name", n, "--full", "--data-dir"].iter().map(|s| s.to_string()).collect();
        a.push(env.root.join("data/knowledge").display().to_string());
        let out = knowledge::find(&Project::new(repo()), &a).unwrap();
        assert!(out.contains("engine status: Full-auto (compiled, not hand-verified)"), "{out}");
    }
}
