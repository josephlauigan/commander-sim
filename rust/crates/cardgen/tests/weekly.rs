//! `cardgen weekly-report` on a tiny synthetic catalog (`tests/fixtures/weekly/`): an old and a new `oracle.json` with
//! new cards in two sets, a ban, an unban, a Game Changer added and one removed, an Oracle errata against the engine's
//! cache, a card gone from the catalog, three decks of `mine` (two affected, one not) and a pool deck that must be
//! ignored. Nothing uses the network, the sim or a server.

use cardgen::data::Project;
use cardgen::weekly;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/weekly").join(name)
}

/// a project root with `data/decks.json`, `data/scryfall_cache.json` and a knowledge directory holding the new catalog,
/// the old one as `oracle.prev.json` and a role table; removed on drop
struct Env(PathBuf);

impl Env {
    fn new() -> Env {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "weekly-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(p.join("data/knowledge")).unwrap();
        let e = Env(p);
        e.copy("decks.json", "data/decks.json");
        e.copy("scryfall_cache.json", "data/scryfall_cache.json");
        e.copy("new_oracle.json", "data/knowledge/oracle.json");
        e.copy("old_oracle.json", "data/knowledge/oracle.prev.json");
        e.copy("roles_now.json", "data/knowledge/roles.json");
        e
    }

    fn copy(&self, from: &str, to: &str) {
        std::fs::copy(fixture(from), self.0.join(to)).unwrap();
    }

    fn dir(&self) -> PathBuf {
        self.0.join("data/knowledge")
    }

    fn report(&self, extra: &[&str]) -> Result<weekly::Output, String> {
        let mut a: Vec<String> =
            ["--date", "2026-11-22", "--commit", "abc1234", "--data-dir"].map(String::from).to_vec();
        a.push(self.dir().display().to_string());
        a.extend(extra.iter().map(|s| s.to_string()));
        weekly::command(&Project::new(&self.0), &a)
    }

    fn text(&self, extra: &[&str]) -> String {
        self.report(extra).unwrap_or_else(|e| panic!("weekly-report {extra:?}: {e}")).text
    }

    fn write(&self, rel: &str, text: &str) -> String {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p.display().to_string()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// the lines of one `## ` section
fn section<'a>(text: &'a str, title: &str) -> &'a str {
    let head = format!("## {title}\n");
    let start = text.find(&head).unwrap_or_else(|| panic!("no section {title}:\n{text}")) + head.len();
    let rest = &text[start..];
    &rest[..rest.find("\n## ").unwrap_or(rest.len())]
}

#[test]
fn the_sections_come_in_the_agreed_order() {
    let e = Env::new();
    let t = e.text(&[]);
    let at = |h: &str| t.find(&format!("\n## {h}\n")).unwrap_or_else(|| panic!("no {h}:\n{t}"));
    let order =
        ["Action needed", "Run", "New cards", "Bans/unbans and Game Changer changes", "Role changes and LLM stats"];
    let pos: Vec<usize> = order.iter().map(|h| at(h)).collect();
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "{pos:?}");
    assert!(t.starts_with("# Card knowledge: weekly report 2026-11-22\n"), "{t}");
    assert!(t.contains("- Repository: abc1234"));
}

#[test]
fn action_needed_lists_only_jds_decks_with_legality_game_changers_errata_and_unknown_cards() {
    let e = Env::new();
    let t = e.text(&[]);
    let a = section(&t, "Action needed");
    // a deck card that became banned
    assert!(a.contains("Deck `kx`: **Dark Tutor** became banned (was legal)."), "{a}");
    // a Game Changer added, with the deck's new count against each tier it is simulated at
    assert!(a.contains("Deck `kx`: Glass Knife joined the Game Changer list; 1 Game Changers now"), "{a}");
    assert!(a.contains("t3 allows 3-3: OUTSIDE") && a.contains("t4 is unrestricted"), "{a}");
    // and one removed, in a deck whose tier allows none
    assert!(
        a.contains("Deck `ky`: Gone Gift left the Game Changer list; 0 Game Changers now (t1 allows 0-0: within"),
        "{a}"
    );
    // errata against the engine's cache
    assert!(a.contains("Old Bear: oracle_text: \"[Old -> New] text of the bear.\""), "{a}");
    // a change in the middle of a long text shows the changed part with its surroundings
    let wide = a.lines().find(|l| l.contains("Wide Change: oracle_text")).unwrap_or_else(|| panic!("{a}"));
    assert!(wide.contains("\"...") && wide.contains("[3 -> 4]") && wide.contains("life and then"), "{wide}");
    // a deck card the catalog does not know
    assert!(a.contains("Deck `kx`: not found in the catalog: Missing Thing"), "{a}");
    // the untouched deck, the pool deck and a card that merely got unbanned raise nothing
    for quiet in ["kz", "Red Burn", "Fresh Ban", "Unban Me", "`pp`"] {
        assert!(!a.contains(quiet), "{quiet} should not be an action item: {a}");
    }
    // the cache matches the catalog for every other deck card: Sky Counter has no errata
    assert!(!a.contains("Sky Counter"), "{a}");
}

#[test]
fn new_cards_by_set_and_inside_each_identity_with_their_roles() {
    let e = Env::new();
    let t = e.text(&[]);
    let n = section(&t, "New cards");
    assert!(n.contains("7 new Oracle cards (5 Commander-legal), 1 gone from the catalog."), "{n}");
    // sets: the one with more cards first; legal cards by EDHREC rank (rank 5, 50, 100, 200)
    assert!(
        n.contains("- **nws**: 4 new, 4 Commander-legal: Fresh Colourless, Fresh Bolt, Fresh Rift, Fresh Vow"),
        "{n}"
    );
    assert!(n.contains("- **oth**: 3 new, 1 Commander-legal: Fresh Mud"), "{n}");
    assert!(n.contains("Gone from the catalog: Retired Card"), "{n}");
    // inside the identities: colourless cards fit every deck; roles in brackets come from the role table
    assert!(n.contains("- `kx` (B): 2 new: Fresh Colourless, Fresh Mud [removal]"), "{n}");
    assert!(n.contains("- `ky` (WU): 3 new: Fresh Colourless, Fresh Rift [counter, draw], Fresh Vow"), "{n}");
    assert!(n.contains("- `kz` (R): 2 new: Fresh Colourless, Fresh Bolt [removal]"), "{n}");
    // a banned or not-legal newcomer is not offered
    assert!(!n.contains("Fresh Ban") && !n.contains("Fresh Silver"), "{n}");
}

#[test]
fn bans_unbans_and_game_changers_catalog_wide() {
    let e = Env::new();
    let t = e.text(&[]);
    let b = section(&t, "Bans/unbans and Game Changer changes");
    assert!(b.contains("- Banned: Dark Tutor (legal -> banned)"), "{b}");
    assert!(b.contains("- Unbanned: Unban Me (banned -> legal)"), "{b}");
    assert!(b.contains("- Other legality changes: none"), "{b}");
    assert!(b.contains("- Game Changer added: Glass Knife"), "{b}");
    assert!(b.contains("- Game Changer removed: Gone Gift"), "{b}");
    assert!(b.contains("- New cards that arrive banned or as Game Changers: Fresh Ban (banned, Game Changer)"), "{b}");
}

#[test]
fn role_changes_need_the_table_from_before_the_run_and_lead_with_jds_cards() {
    let e = Env::new();
    let t = e.text(&[]);
    assert!(section(&t, "Role changes and LLM stats").contains("No role table from before this run"), "{t}");
    let before = fixture("roles_before.json").display().to_string();
    let t = e.text(&["--roles-before", &before]);
    let r = section(&t, "Role changes and LLM stats");
    assert!(r.contains("- Role table: 18 cards (7 not in the table before, 1 gone), 1 with different roles"), "{r}");
    assert!(
        r.contains("- Changes on cards in JD's decks: 1")
            && r.contains("  - Sky Counter: [counter] -> [counter, draw]"),
        "{r}"
    );
    // an unreadable table from before is said, not guessed
    let t = e.text(&["--roles-before", "/nonexistent/roles.json"]);
    assert!(section(&t, "Role changes and LLM stats").contains("not readable"), "{t}");
}

#[test]
fn llm_numbers_come_from_the_cache_files_the_merge_report_and_the_log() {
    let e = Env::new();
    e.write("data/knowledge/llm_roles.jsonl", "{}\n{}\n{}\n");
    e.write("data/knowledge/llm_roles.pending.jsonl", "{}\n\n{}\n");
    e.write(
        "data/knowledge/reports/merge.json",
        r#"{"routed":10,"with_answer":7,"llm_failed":3,"disagreements":{"RiskyRole":{"veto":2,"drop":1}}}"#,
    );
    let log = e.write("log.txt", "line one\nclassified 20 cards\n");
    let stages = e.write("stages.tsv", "llm\tok\t90\t20 cards classified\n");
    let t = e.text(&["--llm-log", &log, "--stages", &stages]);
    let r = section(&t, "Role changes and LLM stats");
    assert!(
        r.contains(
            "- LLM cache: 3 tracked rows (llm_roles.jsonl), 2 pending rows (llm_roles.pending.jsonl, not merged)"
        ),
        "{r}"
    );
    assert!(r.contains("- LLM stage this run: ok (90s): 20 cards classified"), "{r}");
    assert!(
        r.contains("- Last merge (reports/merge.json): routed 10, with_answer 7, llm_failed 3; 3 disagreements"),
        "{r}"
    );
    assert!(r.contains("classified 20 cards"), "{r}");
}

#[test]
fn a_first_run_has_no_diff_and_still_names_deck_cards_that_are_not_legal() {
    let e = Env::new();
    std::fs::remove_file(e.dir().join("oracle.prev.json")).unwrap();
    let t = e.text(&[]);
    assert!(section(&t, "Run").contains("Comparison: none, there is no previous sync"), "{t}");
    assert!(section(&t, "New cards").contains("No new-card list this run"), "{t}");
    assert!(section(&t, "Bans/unbans and Game Changer changes").contains("No comparison this run"), "{t}");
    let a = section(&t, "Action needed");
    assert!(a.contains("Deck `kx`: **Dark Tutor** is banned (no previous sync to say since when)."), "{a}");
    // the state checks do not need a previous sync
    assert!(a.contains("Old Bear: oracle_text") && a.contains("not found in the catalog: Missing Thing"), "{a}");
}

#[test]
fn a_malformed_previous_sync_is_reported_not_fatal() {
    let e = Env::new();
    for bad in ["{oops", "", "{\"meta\": {\"schema\": 9}, \"cards\": []}", "[1,2,3]"] {
        e.write("data/knowledge/oracle.prev.json", bad);
        let t = e.text(&[]);
        let a = section(&t, "Action needed");
        assert!(a.contains("The previous sync cannot be read, so no changes were compared"), "{bad:?}: {a}");
        assert!(section(&t, "New cards").contains("No new-card list this run"), "{bad:?}");
        assert!(a.contains("Old Bear"), "the state checks still run: {a}");
    }
}

#[test]
fn an_unreadable_or_missing_catalog_still_writes_a_report_with_the_failure_first() {
    let e = Env::new();
    e.write("data/knowledge/oracle.json", "{oops");
    let t = e.text(&[]);
    let a = section(&t, "Action needed");
    assert!(a.trim_start().starts_with("- The catalog cannot be read"), "{a}");
    assert!(section(&t, "Run").contains("Catalog: not readable"), "{t}");
    std::fs::remove_file(e.dir().join("oracle.json")).unwrap();
    let out = e.report(&[]).unwrap();
    assert!(out.text.contains("The catalog cannot be read"), "{}", out.text);
    assert!(out.path.is_file());
}

#[test]
fn an_empty_mine_and_a_missing_decks_file_are_said() {
    let e = Env::new();
    e.write("data/decks.json", r#"{"version":1,"mine":[],"pool":[]}"#);
    let t = e.text(&[]);
    let a = section(&t, "Action needed");
    assert!(a.contains("`mine` in data/decks.json is empty, so no deck was checked."), "{a}");
    assert!(!a.contains("Deck `"), "{a}");
    // the catalog-wide sections still work
    assert!(section(&t, "Bans/unbans and Game Changer changes").contains("- Banned: Dark Tutor"), "{t}");
    assert!(section(&t, "New cards").contains("No decks to take identities from."), "{t}");
    std::fs::remove_file(e.0.join("data/decks.json")).unwrap();
    let a = e.text(&[]);
    assert!(a.contains("JD's decks cannot be read, so no deck was checked"), "{a}");
}

#[test]
fn a_skipped_or_failed_sync_shows_no_comparison_and_failures_lead_the_action_list() {
    let e = Env::new();
    let skip = e.write(
        "skip.tsv",
        "fetch-build\tok\t30\tbuilt abc1234\nbulk-sync\tskip\t2\tup to date\nllm\tskip\t0\tGPU busy\n",
    );
    let t = e.text(&["--stages", &skip]);
    assert!(section(&t, "Run").contains("Comparison: none, the bulk file did not change this run"), "{t}");
    assert!(section(&t, "New cards").contains("No new-card list this run"), "{t}");
    assert!(section(&t, "Run").contains("| llm | skip | 0 | GPU busy |"), "{t}");
    assert!(!section(&t, "Action needed").contains("Dark Tutor"), "no diff, no ban: {t}");
    let fail = e.write("fail.tsv", "bulk-sync\tfail\t5\tno network\nroles\tfail\t1\tcardgen exited 1\nbogus line\n");
    let t = e.text(&["--stages", &fail]);
    let a = section(&t, "Action needed");
    let first = a.trim_start().lines().next().unwrap();
    assert_eq!(first, "- Stage `bulk-sync` failed: no network", "{a}");
    assert!(
        a.contains("Stage `roles` failed: cardgen exited 1")
            && a.contains("A line of the stage file could not be read: `bogus line`"),
        "{a}"
    );
    assert!(section(&t, "Run").contains("the catalog sync failed this run"), "{t}");
    assert!(e.report(&["--stages", "/nonexistent/stages.tsv"]).is_err(), "a stage file that is named must exist");
}

#[test]
fn stage_lines_parse_and_bad_ones_are_returned() {
    let (s, bad) = weekly::parse_stages("# c\n\na\tok\t3\tfine\tstill note\nb\tskip\t0\nc\tweird\t1\td\nd\tok\tx\ty\n");
    assert_eq!(s.len(), 2);
    assert_eq!((s[0].name.as_str(), s[0].secs, s[0].note.as_str()), ("a", 3, "fine\tstill note"));
    assert_eq!(s[1].note, "");
    assert_eq!(bad, ["c\tweird\t1\td", "d\tok\tx\ty"]);
}

#[test]
fn the_same_inputs_give_the_same_bytes_and_the_report_is_the_only_file_written() {
    let e = Env::new();
    let before = snapshot(&e.0);
    let (a, b) = (e.0.join("a.md"), e.0.join("b.md"));
    let first = e.text(&["--out", a.to_str().unwrap()]);
    let second = e.text(&["--out", b.to_str().unwrap()]);
    assert_eq!(first, second);
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    let mut after = snapshot(&e.0);
    after.retain(|(p, _)| p != "a.md" && p != "b.md");
    assert_eq!(before, after, "no input changed");
    // the default place and name
    let out = e.report(&[]).unwrap();
    assert_eq!(out.path, e.dir().join("reports/2026-11-22.md"));
    assert_eq!(std::fs::read_to_string(&out.path).unwrap(), first);
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = vec![];
    fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                out.push((p.strip_prefix(base).unwrap().display().to_string(), std::fs::read(&p).unwrap()));
            }
        }
    }
    walk(root, root, &mut out);
    out
}

#[test]
fn twelve_reports_are_kept_a_rerun_keeps_the_earlier_one_and_other_files_are_left_alone() {
    let e = Env::new();
    let reports = e.dir().join("reports");
    std::fs::create_dir_all(&reports).unwrap();
    for day in 1..=14 {
        std::fs::write(reports.join(format!("2026-10-{day:02}.md")), "old").unwrap();
    }
    for other in ["merge.json", "roles-disagreements.txt", "notes.md", "2026-10-xx.md"] {
        std::fs::write(reports.join(other), "keep").unwrap();
    }
    let out = e.report(&[]).unwrap();
    let mut dated: Vec<String> = std::fs::read_dir(&reports)
        .unwrap()
        .filter_map(|f| f.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with("2026-") && n[..10].bytes().all(|b| b.is_ascii_digit() || b == b'-'))
        .collect();
    dated.sort();
    assert_eq!(dated.len(), 12, "twelve dated reports (2026-10-xx.md is not one): {dated:?}");
    assert_eq!(dated.last().unwrap(), "2026-11-22.md");
    assert!(!dated.contains(&"2026-10-01.md".to_string()) && !dated.contains(&"2026-10-03.md".to_string()));
    assert!(dated.contains(&"2026-10-04.md".to_string()));
    assert_eq!(out.pruned, 3);
    for other in ["merge.json", "roles-disagreements.txt", "notes.md", "2026-10-xx.md"] {
        assert!(reports.join(other).is_file(), "{other} was kept");
    }
    // a second report on the same day: the first is kept as DATE.1.md
    let first = std::fs::read_to_string(reports.join("2026-11-22.md")).unwrap();
    let again = e.text(&["--commit", "second"]);
    assert_eq!(std::fs::read_to_string(reports.join("2026-11-22.1.md")).unwrap(), first);
    assert!(
        again.contains("Repository: second")
            && std::fs::read_to_string(reports.join("2026-11-22.md")).unwrap() == again
    );
    // --keep narrows it
    let out = e.report(&["--keep", "2"]).unwrap();
    assert!(out.pruned >= 10);
    assert!(reports.join("2026-11-22.md").is_file() && reports.join("2026-11-22.2.md").is_file());
}

#[test]
fn a_bad_argument_is_an_error_and_writes_nothing() {
    let e = Env::new();
    let before = snapshot(&e.0);
    for args in [&["--date", "22/11/2026"][..], &["--keep", "0"], &["--nope", "x"], &["--commit"], &["--out"]] {
        assert!(e.report(args).is_err(), "{args:?}");
    }
    assert_eq!(before, snapshot(&e.0));
}

#[test]
fn a_deck_card_may_be_spelled_either_way_and_a_synthetic_pool_deck_is_not_checked() {
    let e = Env::new();
    // the deck lists the cards in another case and with a stray accent: names are matched folded
    let decks = std::fs::read_to_string(e.0.join("data/decks.json")).unwrap().replace("Dark Tutor", "dark tútor");
    e.write("data/decks.json", &decks);
    let a = section(&e.text(&[]), "Action needed").to_string();
    assert!(a.contains("**Dark Tutor** became banned"), "{a}");
    assert!(!a.contains("`pp`"), "{a}");
}

#[test]
fn markup_in_names_notes_and_keys_is_escaped_and_flattened() {
    let e = Env::new();
    let nasty = "Bolt # 1 | [click](https://example.invalid) *x* `y`\\n\\n## Forged";
    let oracle = std::fs::read_to_string(fixture("new_oracle.json")).unwrap().replace("Fresh Bolt", nasty);
    e.write("data/knowledge/oracle.json", &oracle);
    let decks = std::fs::read_to_string(fixture("decks.json")).unwrap().replace("\"kz\"", "\"k`z|\"");
    e.write("data/decks.json", &decks);
    let stages = e.write("stages.tsv", "llm\tfail\t1\tnote | with ## and `ticks` [x](y)\\n\n");
    let t = e.text(&["--commit", "abc | `d` # e", "--stages", &stages]);
    // escaped, on one line, and no new section
    let name = "Bolt \\# 1 \\| \\[click\\](https://example.invalid) \\*x\\* \\`y\\` \\#\\# Forged";
    assert!(t.contains(name), "{t}");
    assert!(!t.contains("\n## Forged") && t.matches("\n## ").count() == 5, "{t}");
    assert!(t.contains("- Repository: abc \\| \\`d\\` \\# e"), "{t}");
    assert!(t.contains("| llm | fail | 1 | note \\| with \\#\\# and \\`ticks\\` \\[x\\](y)\\\\n |"), "{t}");
    // a backtick or bar in a deck key cannot end its code span
    assert!(t.contains("- `k'z|` (R)"), "{t}");
}
