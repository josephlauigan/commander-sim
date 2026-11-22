//! Offline adversarial review. Defect tests intentionally stay red; real-catalog tests are opt-in.
use cardgen::data::Project;
use cardgen::digest::{self, Digest, Prev, Spec, Table};
use cardgen::knowledge::{self, Deck, Index, Legal};
use cardgen::{llm, map, roles};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

const MINE: [&str; 6] = ["cloud", "sheoldred", "thanos", "thrasios", "urdragon", "yshtola"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "digest-adversarial-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn card(name: &str, typ: &str, text: &str, ci: &[&str], rank: Option<u32>) -> Value {
    let mut c = json!({"oracle_id": name, "name": name, "type_line": typ, "oracle_text": text,
        "mana_cost": "{1}", "cmc": 1, "color_identity": ci, "keywords": [], "layout": "normal",
        "game_changer": false, "legalities": {"commander": "legal"}});
    if let Some(r) = rank {
        c["edhrec_rank"] = json!(r);
    }
    c
}

struct Env {
    scratch: Scratch,
    index: Index,
    table: Table,
    role_map: roles::RoleMap,
}
impl Env {
    fn new(cards: Vec<Value>) -> Self {
        let scratch = Scratch::new();
        let dir = scratch.0.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("roles_overrides.toml"), "").unwrap();
        std::fs::write(
            dir.join("oracle.json"),
            json!({"meta": {"schema": 1, "count": cards.len()}, "cards": cards}).to_string(),
        )
        .unwrap();
        let mut index = Index::load(&dir.join("oracle.json")).unwrap();
        let role_map = roles::generate(&index, "").unwrap();
        std::fs::write(dir.join("roles.json"), roles::bytes(&role_map).unwrap()).unwrap();
        index.attach_roles(&role_map).unwrap();
        let table = Table::of(&role_map.meta);
        let env = Self { scratch, index, table, role_map };
        std::fs::write(
            env.scratch.0.join("data/decks.json"),
            json!({"version":1,"mine":[{
            "key":"tiny","name":"Tiny","commander":"Leader","ident":"WUBRG","cards":["Leader"],"tiers":["t1"]
        }],"pool":[]})
            .to_string(),
        )
        .unwrap();
        env
    }
    fn tiny() -> Self {
        Self::new(vec![
            card("Leader", "Legendary Creature — Human", "Flying", &[], Some(1)),
            card("Study Spell", "Sorcery", "Draw two cards.", &["U"], Some(2)),
            card("Other Draw", "Instant", "Draw two cards.", &["W"], Some(2)),
            card("Unranked Draw", "Sorcery", "Draw three cards.", &[], None),
            card("Vanilla", "Creature — Spirit", "", &[], Some(3)),
        ])
    }
    fn project(&self) -> Project {
        Project::new(&self.scratch.0)
    }
    fn dir(&self) -> PathBuf {
        self.scratch.0.join("data/knowledge")
    }
    fn deck(&self) -> Deck {
        knowledge::load_deck(&self.project(), "tiny").unwrap()
    }
    fn reattach(&mut self) {
        self.index.attach_roles(&self.role_map).unwrap();
        self.table = Table::of(&self.role_map.meta);
        std::fs::write(self.dir().join("roles.json"), roles::bytes(&self.role_map).unwrap()).unwrap();
    }
    fn cli(&self, args: &[&str]) -> Output {
        Command::new("systemd-run")
            .args(["--user", "--scope", "-q", "-p", "MemoryMax=24G", "-p", "MemorySwapMax=0"])
            .arg(env!("CARGO_BIN_EXE_cardgen"))
            .args(args)
            .arg("--root")
            .arg(&self.scratch.0)
            .arg("--data-dir")
            .arg(self.dir())
            .env_remove("CARDGEN_KNOWLEDGE_DIR")
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .unwrap()
    }
}

fn spec<'a>(deck: Option<&'a Deck>, prev: &'a Prev) -> Spec<'a> {
    Spec {
        deck,
        ident: deck.is_none().then_some(31),
        role: None,
        view: None,
        limit: None,
        offset: 0,
        budget_tokens: None,
        prev,
    }
}
fn lines(text: &str) -> Vec<&str> {
    text.lines().filter(|s| !s.is_empty() && !s.starts_with('#')).collect()
}
fn section(text: &str, id: char) -> Vec<&str> {
    let mut active = false;
    text.lines()
        .filter(|s| {
            if s.starts_with("## ") {
                active = s.starts_with(&format!("## {id} "));
                false
            } else {
                active && !s.starts_with('#') && !s.is_empty()
            }
        })
        .collect()
}
fn names(text: &str) -> Vec<String> {
    lines(text).iter().map(|l| l.split('|').next().unwrap().to_string()).collect()
}
fn line_checks(line: &str) {
    let f: Vec<_> = line.split('|').collect();
    assert_eq!(f.len(), 6, "{line}");
    assert!(line.chars().count() <= 160, "{} chars: {line}", line.chars().count());
    assert!(!line.contains(['\t', '\r', '\n']), "{line:?}");
    assert!(f.iter().all(|s| !s.is_empty() && s.trim() == *s), "{line:?}");
    let flags: Vec<_> = f[5].split_whitespace().collect();
    assert!(
        flags == ["-"]
            || flags.iter().all(|s| *s == "*"
                || ["GC", "BAN", "NL", "?"].contains(s)
                || s.strip_prefix('#').is_some_and(|n| n.parse::<u32>().is_ok())
                || s.strip_prefix('$').is_some_and(|n| n.parse::<u64>().is_ok())),
        "{line}"
    );
}

#[test]
fn cli_validation_output_paths_and_atomic_success() {
    let e = Env::tiny();
    for args in [
        vec![],
        vec!["--ident", "X"],
        vec!["--ident", "W", "--role", "bogus"],
        vec!["--deck", "unknown"],
        vec!["--ident", "W", "--limit", "1"],
        vec!["--ident", "W", "--budget-tokens", "0"],
        vec!["--ident", "W", "--budget-tokens", "-1"],
        vec!["--ident", "W", "--budget-tokens", "NaN"],
        vec!["--ident", "W", "--role", "draw", "--limit", "0"],
        vec!["--ident", "W", "--role", "draw", "--offset", "-1"],
        vec!["--ident", "W", "--role", "draw", "--limit", "abc"],
        vec!["--ident", "W", "--role", "draw", "--offset", "abc"],
    ] {
        let args: Vec<_> = ["digest"].into_iter().chain(args).collect();
        let out = e.cli(&args);
        assert!(!out.status.success(), "{args:?}");
        assert!(!out.stderr.is_empty(), "{args:?}");
        assert!(!e.dir().join("digests").exists(), "error wrote output: {args:?}");
    }
    for ident in ["wu", "WWU", "C", ""] {
        let out = e.cli(&["digest", "--ident", ident, "--stdout"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert!(out.stdout.starts_with(b"# digest:"));
        assert!(String::from_utf8_lossy(&out.stderr).contains("digest:"));
    }
    let out = e.cli(&["digest", "--deck", "TINY", "--ident", "W", "--stdout"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = e.cli(&["map"]);
    assert!(out.status.success());
    assert!(e.dir().join("map.md").is_file());
    let path = e.scratch.0.join("missing/nested/view.txt");
    for cmd in ["map", "digest"] {
        let args = if cmd == "map" {
            vec![cmd, "--stdout", "--out", path.to_str().unwrap()]
        } else {
            vec![cmd, "--ident", "WUBRG", "--stdout", "--out", path.to_str().unwrap()]
        };
        let out = e.cli(&args);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(out.stdout, std::fs::read(&path).unwrap());
        assert!(!path.with_extension("txt.tmp").exists());
    }
    let empty = e.scratch.0.join("empty");
    std::fs::create_dir(&empty).unwrap();
    let args = vec!["--ident".into(), "W".into(), "--data-dir".into(), empty.display().to_string()];
    assert!(format!("{:?}", digest::command(&e.project(), &args).unwrap_err()).contains("cardgen bulk-sync"));
    let ignored = Command::new("git")
        .current_dir(root())
        .args(["check-ignore", "data/knowledge/map.md", "data/knowledge/digests/test.txt"])
        .output()
        .unwrap();
    assert!(ignored.status.success());
}

#[test]
fn cli_file_permissions_directory_errors_and_extreme_pages() {
    let e = Env::tiny();
    let path = e.scratch.0.join("readonly.txt");
    std::fs::write(&path, "previous output").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    }
    // A writable parent permits atomic rename over a read-only file on Unix. A clean refusal is also safe.
    let out = e.cli(&["map", "--out", path.to_str().unwrap(), "--stdout"]);
    if out.status.success() {
        assert_eq!(std::fs::read(&path).unwrap(), out.stdout);
    } else {
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "previous output");
    }
    assert!(!e.scratch.0.join("readonly.txt.tmp").exists());
    let dir = e.scratch.0.join("out-directory");
    std::fs::create_dir(&dir).unwrap();
    let out = e.cli(&["map", "--out", dir.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(dir.is_dir());
    assert!(String::from_utf8_lossy(&out.stderr).contains("out-directory"));
    let huge = usize::MAX.to_string();
    let out = e.cli(&["digest", "--ident", "WUBRG", "--role", "draw", "--limit", &huge, "--offset", &huge, "--stdout"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("no cards at offset"));
    let decks_path = e.scratch.0.join("data/decks.json");
    let mut decks: Value = serde_json::from_slice(&std::fs::read(&decks_path).unwrap()).unwrap();
    decks["mine"][0]["ident"] = json!("W");
    std::fs::write(&decks_path, decks.to_string()).unwrap();
    let out = e.cli(&["digest", "--deck", "tiny", "--ident", "U"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("outside deck tiny"));
}

#[test]
fn role_tables_rules_merged_failed_stale_missing_and_corrupt() {
    let mut e = Env::tiny();
    let i = e.index.lookup("Study Spell")[0];
    assert!(!e.table.merged);
    assert!(
        digest::build(&e.index, &e.table, &spec(None, &Prev::Missing)).unwrap().text.contains("role table: rules-only")
    );
    // A guarded, cited LLM role, a failed routed row, a roleless final row, and an override final row.
    e.role_map.meta.prompt_fnv =
        llm::prompt_fnv(&roles::Taxonomy::parse(include_str!("../../../../data/knowledge/roles.toml")).unwrap());
    e.role_map.meta.model = "offline-test".into();
    for a in e.role_map.values_mut() {
        if a.needs_llm {
            a.llm_failed = true;
        }
    }
    let a = e.role_map.get_mut("Study Spell").unwrap();
    a.roles = vec!["draw".into()];
    a.primary = "draw".into();
    a.needs_llm = true;
    a.llm_failed = false;
    a.evidence = BTreeMap::from([(
        "draw".into(),
        roles::Evidence { source: roles::Source::Llm, certain: false, rule: "llm".into(), face: None, line: Some(0) },
    )]);
    e.role_map.get_mut("Vanilla").unwrap().needs_llm = false;
    let failed = e.role_map.get_mut("Unranked Draw").unwrap();
    failed.needs_llm = true;
    failed.llm_failed = true;
    let other = e.role_map.get_mut("Other Draw").unwrap();
    other.needs_llm = false;
    for ev in other.evidence.values_mut() {
        ev.source = roles::Source::Override;
        ev.line = None;
        ev.face = None;
        ev.certain = false;
    }
    e.reattach();
    let l = digest::Lines::new(&e.index, &e.table);
    // only an unreviewed card carries a flag for it
    let status = |name: &str| {
        let i = e.index.lookup(name)[0];
        (l.review(i), l.line(i, false).ends_with('?'))
    };
    assert_eq!((l.review(i), l.line(i, false).ends_with('?')), (digest::Review::Llm, false));
    assert_eq!(status("Vanilla"), (digest::Review::None, false));
    assert_eq!(status("Other Draw"), (digest::Review::Rules, false));
    assert_eq!(status("Unranked Draw"), (digest::Review::Uncertain, true));
    assert!(
        digest::build(&e.index, &e.table, &spec(None, &Prev::Missing))
            .unwrap()
            .text
            .contains("merged (model offline-test)")
    );
    e.role_map.meta.count += 1;
    std::fs::write(e.dir().join("roles.json"), roles::bytes(&e.role_map).unwrap()).unwrap();
    assert!(digest::open(&e.project(), Some(&e.dir())).err().unwrap().contains("cardgen roles --rules"));
    std::fs::write(e.dir().join("roles.json"), "{bad").unwrap();
    assert!(digest::open(&e.project(), Some(&e.dir())).is_err());
    std::fs::remove_file(e.dir().join("roles.json")).unwrap();
    assert!(digest::open(&e.project(), Some(&e.dir())).err().unwrap().contains("cardgen roles --rules"));
}

#[test]
fn section_a_resolves_folded_names_partners_basics_banned_and_unknown() {
    let mut banned = card("Banned", "Sorcery", "Draw a card.", &[], Some(5));
    banned["legalities"]["commander"] = json!("banned");
    let e = Env::new(vec![
        card("Leader", "Creature — Human", "", &[], Some(1)),
        card("Andúril's Friend", "Creature — Spirit", "", &[], Some(2)),
        card("Plains", "Basic Land — Plains", "({T}: Add {W}.)", &[], Some(3)),
        banned,
    ]);
    let mut deck = e.deck();
    deck.commander2 = Some("Anduril’s Friend".into());
    deck.cards = vec!["Plains".into(), "Plains".into(), "Banned".into(), "Unknown".into()];
    let d = digest::build(&e.index, &e.table, &spec(Some(&deck), &Prev::Missing)).unwrap();
    assert_eq!(section(&d.text, 'A').len(), 4); // documented one line per name, including basics
    assert!(d.text.contains("Unknown") && d.text.contains("does not list as Commander-legal (1): Banned"));
    assert!(section(&d.text, 'A').iter().all(|l| l.split('|').nth(5).unwrap().starts_with('*')));
}

#[test]
fn every_role_pages_by_one_without_gaps_and_empty_roles_terminate() {
    let e = Env::tiny();
    for role in roles::IDS {
        let expected: Vec<_> = e
            .index
            .cards
            .iter()
            .enumerate()
            .filter(|(i, _)| e.index.roles(*i as u32).unwrap().primary == role)
            .map(|(_, c)| c.raw.name.clone())
            .collect();
        let mut got = vec![];
        let mut offset = 0;
        loop {
            let d = digest::build(
                &e.index,
                &e.table,
                &Spec { role: Some(role), limit: Some(1), offset, ..spec(None, &Prev::Missing) },
            )
            .unwrap();
            got.extend(names(&d.text));
            if let Some(next) = d.next_offset {
                assert!(next > offset);
                offset = next;
            } else {
                break;
            }
        }
        assert_eq!(got.iter().collect::<BTreeSet<_>>(), expected.iter().collect());
        assert_eq!(got.len(), expected.len());
        let end = digest::build(
            &e.index,
            &e.table,
            &Spec { role: Some(role), offset: usize::MAX, ..spec(None, &Prev::Missing) },
        )
        .unwrap();
        assert_eq!(end.lines, 0);
        assert!(end.next_offset.is_none());
    }
    let mut deck = e.deck();
    deck.cards.extend(["Study Spell".into(), "Other Draw".into(), "Unranked Draw".into()]);
    let d =
        digest::build(&e.index, &e.table, &Spec { role: Some("draw"), ..spec(Some(&deck), &Prev::Missing) }).unwrap();
    assert_eq!(d.lines, 0);
}

#[test]
fn full_budget_trimming_is_monotonic_and_reports_impossible_budgets() {
    let e = Env::tiny();
    let mut previous = 0;
    for budget in [1, 100, 400, 500, 600, 700, 800, 1000, 10_000, usize::MAX] {
        let d = digest::build(&e.index, &e.table, &Spec { budget_tokens: Some(budget), ..spec(None, &Prev::Missing) })
            .unwrap();
        assert!(digest::tokens(&d.text) <= budget || d.over_budget);
        assert!(d.lines >= previous, "larger budget lost cards: {budget}");
        previous = d.lines;
    }
}

#[test]
fn previous_sync_valid_empty_same_disjoint_missing_and_malformed() {
    let e = Env::tiny();
    assert!(matches!(Prev::load(&e.dir()), Prev::Missing));
    for value in ["", "broken", "[]", "{}"] {
        std::fs::write(e.dir().join("oracle.prev.json"), value).unwrap();
        assert!(matches!(Prev::load(&e.dir()), Prev::Unreadable(_)));
    }
    for (value, count) in [
        (json!({"cards":[]}), 5),
        (json!({"cards":[{"oracle_id":"disjoint"}]}), 5),
        (json!({"cards":e.index.cards.iter().map(|c|json!({"oracle_id":c.raw.oracle_id})).collect::<Vec<_>>()}), 0),
    ] {
        std::fs::write(e.dir().join("oracle.prev.json"), value.to_string()).unwrap();
        let prev = Prev::load(&e.dir());
        let d = digest::build(&e.index, &e.table, &spec(None, &prev)).unwrap();
        assert_eq!(section(&d.text, 'D').len(), count);
    }
}

#[test]
fn defect_previous_sync_accepts_wrong_schema() {
    let e = Env::tiny();
    let value = json!({"meta":{"schema":999},"cards":[{"oracle_id":"Leader"}]});
    std::fs::write(e.dir().join("oracle.prev.json"), value.to_string()).unwrap();
    assert!(matches!(Prev::load(&e.dir()), Prev::Unreadable(_)), "accepted invalid previous sync: {value}");
}

#[test]
fn defect_previous_sync_accepts_missing_ids() {
    let e = Env::tiny();
    std::fs::write(e.dir().join("oracle.prev.json"), json!({"cards":[{"name":"Leader"}]}).to_string()).unwrap();
    assert!(matches!(Prev::load(&e.dir()), Prev::Unreadable(_)), "missing oracle_id silently became an empty string");
}

#[test]
fn defect_cut_splits_a_word_without_a_space() {
    let source = "Vigilance";
    let gist = digest::cut(source, 5);
    let body = gist.trim_end_matches('…');
    assert!(body.is_empty() || source[body.len()..].starts_with(' '), "not a word boundary: {gist:?} from {source:?}");
}

#[test]
fn defect_cut_splits_a_mana_symbol_without_a_space() {
    let gist = digest::cut("{W}{U}{B}{R}{G}: Draw a card.", 6);
    assert_eq!(gist.matches('{').count(), gist.matches('}').count(), "partial symbol: {gist}");
}

#[test]
fn defect_cut_leaves_an_unterminated_quoted_ability() {
    let e = Env::new(vec![card(
        "Grant",
        "Enchantment",
        "Creatures you control have \"Whenever this creature attacks, draw a card, then discard a card, create a \
         Treasure token, and gain 3 life.\"",
        &[],
        Some(1),
    )]);
    let line = digest::Lines::new(&e.index, &e.table).line(0, false);
    let gist = line.split('|').nth(4).unwrap();
    assert_eq!(gist.matches('"').count() % 2, 0, "unterminated quoted ability: {line}");
}

#[test]
fn defect_line_limit_fails_when_fixed_fields_alone_are_too_long() {
    // A real catalog name (not legal, but legalities do not exclude a deck's section A).
    let name = "Our Market Research Shows That Players Like Really Long Card Names So We Made this Card to Have the \
                Absolute Longest Card Name Ever Elemental";
    let mut c = card(name, "Creature — Elemental", "Trample", &["G"], Some(1));
    c["mana_cost"] = json!("{1}{G}{G}");
    c["legalities"]["commander"] = json!("not_legal");
    let e = Env::new(vec![c]);
    let mut deck = e.deck();
    deck.commander = name.into();
    deck.cards = vec![name.into()];
    let d = digest::build(&e.index, &e.table, &spec(Some(&deck), &Prev::Missing)).unwrap();
    for line in lines(&d.text) {
        line_checks(line);
    }
}

#[test]
fn defect_one_character_gist_room_produces_an_empty_field() {
    let mut c = card(&"N".repeat(143), "Creature", "Flying", &[], None);
    // 143 name + 3 cost + 1 type + 1 role + 6 flag + 5 separators = 159.
    c["mana_cost"] = json!("{1}");
    let e = Env::new(vec![c]);
    let line = digest::Lines::new(&e.index, &e.table).line(0, false);
    assert!(!line.split('|').nth(4).unwrap().is_empty(), "empty gist: {line}");
}

#[test]
fn defect_creature_type_map_counts_noncreature_face_subtypes() {
    let mut c = card("Giant // Journey", "Creature — Giant // Sorcery — Adventure", "", &[], Some(1));
    c["card_faces"] = json!([{"name":"Giant","type_line":"Creature — Giant","oracle_text":"Flying"},
        {"name":"Journey","type_line":"Sorcery — Adventure","oracle_text":"Draw a card."}]);
    let e = Env::new(vec![c]);
    let text = map::build(&e.index, &e.table, None).unwrap();
    let creature = text.lines().skip_while(|l| !l.starts_with("## Creature types")).nth(2).unwrap();
    assert_eq!(creature, "Giant 1", "noncreature subtypes in creature-type summary: {creature}");
}

#[test]
fn defect_synergy_treats_a_quoted_card_name_as_a_subtype() {
    let e = Env::new(vec![
        card(
            "Leader",
            "Legendary Creature — Human",
            "Search your library for a card named Spirit Guide, reveal it, put it into your hand, then shuffle.",
            &[],
            Some(1),
        ),
        card("Spirit Guide", "Creature — Elf", "", &[], Some(2)),
        card("Unrelated", "Creature — Spirit", "Flying", &[], Some(3)),
    ]);
    let deck = e.deck();
    let subs = digest::commander_subtypes(&e.index, &deck);
    assert!(!subs.contains(&"Spirit".to_string()), "a card name became tribal synergy: {subs:?}");
}

#[test]
fn defect_role_page_silently_exceeds_an_impossible_budget() {
    let e = Env::tiny();
    for budget in [1, 100] {
        let result = digest::build(
            &e.index,
            &e.table,
            &Spec { role: Some("draw"), budget_tokens: Some(budget), ..spec(None, &Prev::Missing) },
        );
        if let Ok(d) = result {
            assert!(
                digest::tokens(&d.text) <= budget || d.over_budget,
                "{} tokens > {budget}, {} lines, over_budget={}",
                digest::tokens(&d.text),
                d.lines,
                d.over_budget
            );
        }
    }
}

#[test]
fn defect_failed_rename_leaves_a_partial_output_file() {
    let e = Scratch::new();
    let target = e.0.join("directory");
    std::fs::create_dir(&target).unwrap();
    assert!(digest::write_file(&target, "new output").is_err());
    assert!(!e.0.join("directory.tmp").exists(), "failed write left directory.tmp containing output");
}

#[test]
fn defect_concurrent_writers_share_the_same_temporary_file() {
    let e = Scratch::new();
    let target = e.0.join("same.txt");
    let barrier = Arc::new(Barrier::new(16));
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|i| {
                let b = barrier.clone();
                let p = &target;
                scope.spawn(move || {
                    let text = format!("writer-{i}\n").repeat(32_768);
                    b.wait();
                    digest::write_file(p, &text)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
    });
    let errors: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();
    assert!(errors.is_empty(), "concurrent atomic writes failed: {errors:?}");
    let actual = std::fs::read_to_string(target).unwrap();
    assert!((0..16).any(|i| actual == format!("writer-{i}\n").repeat(32_768)), "mixed/truncated output");
}

#[test]
fn synthetic_map_is_small_deterministic_and_spellbook_errors_are_explicit() {
    let mut cards = vec![];
    for i in 0..1000 {
        let mut c = card(&format!("Creature {i}"), &format!("Creature — Subtype{i}"), "Flying", &[], Some(i));
        c["keywords"] = json!([format!("Keyword{i}"), format!("Keyword{i}")]);
        cards.push(c);
    }
    let e = Env::new(cards);
    let text = map::build(&e.index, &e.table, None).unwrap();
    assert!(text.len() < 32 * 1024);
    assert_eq!(text, map::build(&e.index, &e.table, None).unwrap());
    let dir = e.scratch.0.join("spellbook");
    std::fs::create_dir(&dir).unwrap();
    assert!(map::build(&e.index, &e.table, Some(&dir)).unwrap().contains("No deck files"));
    std::fs::write(dir.join("empty.json"), "").unwrap();
    assert!(map::build(&e.index, &e.table, Some(&dir)).unwrap_err().contains("empty.json"));
    std::fs::write(dir.join("empty.json"), "{bad").unwrap();
    assert!(map::build(&e.index, &e.table, Some(&dir)).unwrap_err().contains("empty.json"));
}

fn real() -> (Index, Table) {
    digest::open(&Project::new(root()), Some(&root().join("data/knowledge"))).unwrap()
}
fn independent() -> Value {
    let out = Command::new("python3")
        .arg("-I")
        .arg(root().join("rust/crates/cardgen/tests/fixtures/adversarial_digest/oracle_audit.py"))
        .arg(root())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap()
}
fn normalised_sources(index: &Index, i: u32) -> Vec<String> {
    let c = &index.cards[i as usize];
    c.lines().iter().map(|(_, s)| roles::normalize(c, s).replace('|', "/").replace(['\t', '\r'], " ")).collect()
}
fn check_digest(index: &Index, d: &Digest, ident: u8) {
    assert_eq!(lines(&d.text).len(), d.lines);
    let mut section_id = ' ';
    let mut role = "";
    let mut seen = BTreeSet::new();
    for line in d.text.lines() {
        if line.starts_with("## ") {
            section_id = line.chars().nth(3).unwrap();
            role = if section_id == 'B' { line.split_whitespace().nth(3).unwrap().trim_end_matches(':') } else { "" };
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        line_checks(line);
        let f: Vec<_> = line.split('|').collect();
        let found = index.lookup(f[0]);
        let i = found.iter().copied().find(|&i| index.cards[i as usize].legal == Legal::Legal).unwrap_or(found[0]);
        let c = &index.cards[i as usize];
        if section_id != 'A' {
            assert_eq!(c.legal, Legal::Legal);
            assert!(!c.is_meld());
            assert!(roles::is_deck_card(c));
            assert_eq!(c.ident & !ident, 0, "{line}");
        }
        assert_eq!(f[5].starts_with('*'), section_id == 'A');
        if section_id == 'B' {
            assert_eq!(index.roles(i).unwrap().primary, role);
            assert!(seen.insert(f[0]));
        }
        if f[4] != "-" {
            let body = f[4].trim_end_matches('…');
            assert!(normalised_sources(index, i).iter().any(|s| s.starts_with(body)), "invented gist: {line}");
        }
    }
    for id in ['A', 'B', 'C', 'D', 'E'] {
        assert_eq!(
            section(&d.text, id).len(),
            d.sections.iter().filter(|s| s.id == id).map(|s| s.lines).sum::<usize>()
        );
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn whole_catalog_lines_and_all_32_identities_and_six_decks() {
    let (idx, table) = real();
    let l = digest::Lines::new(&idx, &table);
    for (i, c) in
        idx.cards.iter().enumerate().filter(|(_, c)| c.legal == Legal::Legal && !c.is_meld() && roles::is_deck_card(c))
    {
        let line = l.line(i as u32, false);
        line_checks(&line);
        let f: Vec<_> = line.split('|').collect();
        if f[4] != "-" {
            assert!(
                normalised_sources(&idx, i as u32).iter().any(|s| s.starts_with(f[4].trim_end_matches('…'))),
                "{}: {line}",
                c.raw.name
            );
        }
    }
    for ident in 0..32 {
        let d = digest::build(&idx, &table, &Spec { ident: Some(ident), ..spec(None, &Prev::Missing) }).unwrap();
        check_digest(&idx, &d, ident);
        assert!(digest::tokens(&d.text) <= 50_000);
        assert_eq!(
            d.text,
            digest::build(&idx, &table, &Spec { ident: Some(ident), ..spec(None, &Prev::Missing) }).unwrap().text
        );
    }
    for key in MINE {
        let deck = knowledge::load_deck(&Project::new(root()), key).unwrap();
        let d = digest::build(&idx, &table, &spec(Some(&deck), &Prev::Missing)).unwrap();
        check_digest(&idx, &d, knowledge::deck_ident(&deck).unwrap());
        assert!(d.text.len() < 200_000);
        assert!(digest::tokens(&d.text) <= 50_000);
        println!("{key}: {} bytes, {} lines", d.text.len(), d.lines);
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn defect_real_catalog_gists_have_unterminated_quotes() {
    let (idx, table) = real();
    let l = digest::Lines::new(&idx, &table);
    let mut failures = vec![];
    for (i, _c) in
        idx.cards.iter().enumerate().filter(|(_, c)| c.legal == Legal::Legal && !c.is_meld() && roles::is_deck_card(c))
    {
        let line = l.line(i as u32, false);
        let gist = line.split('|').nth(4).unwrap();
        if gist.matches('"').count() % 2 != 0 {
            failures.push(line);
        }
    }
    assert!(
        failures.is_empty(),
        "{} unterminated quotes; examples:\n{}",
        failures.len(),
        failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn real_all_role_pages_match_independent_raw_json_and_chain_in_order() {
    let (idx, table) = real();
    let audit = independent();
    for key in ["cloud", "thanos", "thrasios", "lathril-golgari-elves"] {
        let deck = knowledge::load_deck(&Project::new(root()), key).unwrap();
        for role in roles::IDS {
            let expected: Vec<String> = audit["candidates"][key]["roles"][role]
                .as_array()
                .map(|a| a.iter().map(|s| s.as_str().unwrap().to_string()).collect())
                .unwrap_or_default();
            let mut got = vec![];
            let mut offset = 0;
            loop {
                let d = digest::build(
                    &idx,
                    &table,
                    &Spec { role: Some(role), limit: Some(137), offset, ..spec(Some(&deck), &Prev::Missing) },
                )
                .unwrap();
                assert!(digest::tokens(&d.text) <= 30_000);
                got.extend(names(&d.text));
                if let Some(next) = d.next_offset {
                    assert!(next > offset);
                    offset = next;
                } else {
                    break;
                }
            }
            assert_eq!(got, expected, "{key}/{role}");
        }
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn defect_candidate_union_cannot_reach_roleless_cards() {
    let (idx, table) = real();
    let audit = independent();
    let mut failures = vec![];
    for key in ["cloud", "thanos", "thrasios", "lathril-golgari-elves"] {
        let deck = knowledge::load_deck(&Project::new(root()), key).unwrap();
        let mut got = BTreeSet::new();
        let d =
            digest::build(&idx, &table, &Spec { budget_tokens: Some(usize::MAX), ..spec(Some(&deck), &Prev::Missing) })
                .unwrap();
        for id in ['B', 'C', 'E'] {
            got.extend(section(&d.text, id).iter().map(|l| l.split('|').next().unwrap().to_string()));
        }
        for role in roles::IDS {
            let mut offset = 0;
            loop {
                let page = digest::build(
                    &idx,
                    &table,
                    &Spec { role: Some(role), offset, ..spec(Some(&deck), &Prev::Missing) },
                )
                .unwrap();
                got.extend(names(&page.text));
                if let Some(next) = page.next_offset {
                    offset = next;
                } else {
                    break;
                }
            }
        }
        // the complete roleless view, in pages
        let mut offset = 0;
        let mut roleless = vec![];
        loop {
            let page = digest::build(
                &idx,
                &table,
                &Spec { view: Some(digest::View::Roleless), offset, ..spec(Some(&deck), &Prev::Missing) },
            )
            .unwrap();
            roleless.extend(names(&page.text));
            if let Some(next) = page.next_offset {
                offset = next;
            } else {
                break;
            }
        }
        let want_roleless: Vec<String> = audit["candidates"][key]["roleless"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(roleless, want_roleless, "{key}: the roleless view");
        got.extend(roleless);
        let expected: BTreeSet<String> = audit["candidates"][key]["all"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        let missing: Vec<_> = expected.difference(&got).collect();
        if !missing.is_empty() {
            failures.push(format!(
                "{key}: {} of {} candidates unreachable; first {:?}",
                missing.len(),
                expected.len(),
                missing.iter().take(5).collect::<Vec<_>>()
            ));
        }
        assert!(got.is_subset(&expected), "off-identity/illegal/deck card appeared for {key}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn num(n: u64) -> String {
    let s = n.to_string();
    s.chars()
        .enumerate()
        .map(|(i, c)| if i > 0 && (s.len() - i) % 3 == 0 { format!(",{c}") } else { c.to_string() })
        .collect()
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn real_map_counts_match_independent_python() {
    let (idx, table) = real();
    let a = independent();
    let text = map::build(&idx, &table, Some(&root().join("data/spellbook"))).unwrap();
    assert!(text.len() < 32 * 1024);
    assert_eq!(text, map::build(&idx, &table, Some(&root().join("data/spellbook"))).unwrap());
    for (key, label) in [("legal", "Commander-legal"), ("banned", "banned"), ("not_legal", "not legal")] {
        assert!(text.contains(&format!("{} {label}", num(a["legalities"][key].as_u64().unwrap()))));
    }
    assert!(text.contains(&format!("cover {} legal cards", num(a["covered"].as_u64().unwrap()))));
    for (ident, n) in a["identities"].as_object().unwrap() {
        assert!(text.contains(&format!("| {ident} | {} |", num(n.as_u64().unwrap()))));
    }
    for (r, row) in a["grid"].as_object().unwrap() {
        assert!(text.contains(&format!(
            "| {r} | {} |",
            row.as_array().unwrap().iter().map(Value::to_string).collect::<Vec<_>>().join(" | ")
        )));
    }
    for (key, header) in [("banned", "## Banned in Commander"), ("game_changers", "## Game Changers")] {
        let expected = a[key].as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>().join("; ");
        assert_eq!(text.lines().skip_while(|s| !s.starts_with(header)).nth(2).unwrap(), expected);
    }
    let kws = a["keywords"]
        .as_array()
        .unwrap()
        .iter()
        .take(15)
        .map(|p| format!("{} {}", p[0].as_str().unwrap(), p[1]))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(text.lines().skip_while(|s| !s.starts_with("## Keywords")).nth(2).unwrap(), kws);
    let subs = a["subtypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| format!("{} {}", p[0].as_str().unwrap(), p[1]))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(text.lines().skip_while(|s| !s.starts_with("## Subtypes of every kind")).nth(2).unwrap(), subs);
    assert!(text.contains(&format!("## Subtypes of every kind ({} in all", a["subtype_vocabulary"])));
    assert!(text.contains(&format!(
        "{} of {} cards have at least one role",
        num(a["with_role"].as_u64().unwrap()),
        num(a["covered"].as_u64().unwrap())
    )));
    assert!(text.contains(&format!("{} cards need the LLM", num(a["routed"].as_u64().unwrap()))));
    // the combos of JD's six decks (only those), with their counts
    for row in a["spellbook"].as_array().unwrap() {
        let row: Vec<&str> = row.as_array().unwrap().iter().map(|s| s.as_str().unwrap()).collect();
        let shown = text.contains(&format!("- {} (fetched {}): {} in the list", row[0], row[2], row[3]));
        assert_eq!(shown, MINE.contains(&row[0]), "spellbook row {row:?}");
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn defect_real_map_creature_vocabulary_includes_noncreature_faces() {
    let (idx, table) = real();
    let a = independent();
    let text = map::build(&idx, &table, None).unwrap();
    let header = text.lines().find(|s| s.starts_with("## Creature types")).unwrap();
    let count: u64 = header.split('(').nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    // Even counting every subtype of a creature-bearing face (including Equipment etc.) is an upper bound.
    let upper = a["creature_face_subtype_vocabulary"].as_u64().unwrap();
    assert!(count <= upper, "{count} creature types claimed, but at most {upper} occur on creature faces");
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn real_type_abbreviations_match_every_card_and_face_type_head() {
    let (idx, _) = real();
    let raw: Value =
        serde_json::from_slice(&std::fs::read(root().join("data/knowledge/oracle.json")).unwrap()).unwrap();
    for c in raw["cards"].as_array().unwrap() {
        // one abbreviation per face (the faces of the type line, else the card_faces), written once when all agree
        let mut faces: Vec<&str> = c["type_line"].as_str().unwrap().split(" // ").collect();
        if faces.len() == 1 {
            let from: Vec<&str> =
                c["card_faces"].as_array().into_iter().flatten().filter_map(|f| f["type_line"].as_str()).collect();
            if from.len() > 1 {
                faces = from;
            }
        }
        let mut parts: Vec<String> = faces
            .iter()
            .map(|face| {
                let heads: BTreeSet<String> =
                    face.split('—').next().unwrap().split_whitespace().map(str::to_lowercase).collect();
                let mut expected = if heads.contains("legendary") { "Lg" } else { "" }.to_string();
                let types: String = [
                    ("Artifact", 'A'),
                    ("Enchantment", 'E'),
                    ("Land", 'L'),
                    ("Creature", 'C'),
                    ("Planeswalker", 'P'),
                    ("Battle", 'B'),
                    ("Instant", 'I'),
                    ("Sorcery", 'S'),
                ]
                .iter()
                .filter(|(t, _)| heads.contains(&t.to_lowercase()))
                .map(|(_, a)| a)
                .collect();
                expected.push_str(if types.is_empty() { "O" } else { &types });
                expected
            })
            .collect();
        parts.dedup();
        if parts.iter().all(|p| *p == parts[0]) {
            parts.truncate(1);
        }
        let i = idx.by_oracle_id(c["oracle_id"].as_str().unwrap()).unwrap();
        assert_eq!(digest::type_abbrev(&idx.cards[i as usize]), parts.join("/"), "{}", c["name"]);
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json and roles.json"]
fn defect_disjoint_previous_sync_exceeds_default_five_colour_budget() {
    let (idx, table) = real();
    let deck = knowledge::load_deck(&Project::new(root()), "thanos").unwrap();
    let prev = Prev::Ids(Default::default());
    let d = digest::build(&idx, &table, &spec(Some(&deck), &prev)).unwrap();
    assert!(
        d.text.len() < 200_000,
        "{} bytes, {} tokens, {} D lines, over_budget={}",
        d.text.len(),
        digest::tokens(&d.text),
        section(&d.text, 'D').len(),
        d.over_budget
    );
}
