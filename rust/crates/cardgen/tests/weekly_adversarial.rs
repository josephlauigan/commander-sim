//! Offline adversarial review of phase 6. Defect tests intentionally remain red until production is fixed.
use cardgen::{bulk, data::Project, llm, weekly};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}
fn capped(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut c = Command::new("systemd-run");
    // Pass shell code literally: systemd-run expansion turns $$ into $, corrupting signal targets.
    c.arg("--expand-environment=no");
    c.args(["--user", "--scope", "-q", "-p", "MemoryMax=24G", "-p", "MemorySwapMax=0"]).arg(program);
    c
}
fn successful(o: Output) -> String {
    assert!(
        o.status.success(),
        "{:?}\n{}\n{}",
        o.status,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout).unwrap()
}
// User-manager scope creation can fail when many very short nested scopes race.
// Serialize sandbox lifetimes; explicit concurrency tests still launch their own children.
struct Sandbox(PathBuf, #[allow(dead_code)] std::sync::MutexGuard<'static, ()>);
impl Sandbox {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        static SCOPES: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = SCOPES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let s = Self(
            std::env::temp_dir().join(format!(
                "weekly-adversarial-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::SeqCst)
            )),
            guard,
        );
        for p in ["home", "bin", "out", "clone/data", "main", "xdg"] {
            std::fs::create_dir_all(s.p(p)).unwrap();
        }
        s.write("clone/data/decks.json", r#"{"mine":[],"pool":[]}"#);
        std::fs::copy(repo().join("data/knowledge/roles.toml"), s.p("out/roles.toml")).unwrap();
        s.stub("git", "exit 97"); // fail closed: no network or destructive real git calls
        for name in ["curl", "ss", "systemctl", "notify-send"] {
            s.stub(name, "printf '%s\\n' \"$0 $*\" >> \"$TEST_ROOT/calls\"; exit 97");
        }
        s.stub("fake-cardgen", "exit 97");
        s
    }
    fn p(&self, p: &str) -> PathBuf {
        self.0.join(p)
    }
    fn write(&self, p: &str, text: &str) {
        let p = self.p(p);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    fn read(&self, p: &str) -> String {
        std::fs::read_to_string(self.p(p)).unwrap()
    }
    fn stub(&self, name: &str, body: &str) {
        let p = self.p(&format!("bin/{name}"));
        std::fs::write(&p, format!("#!/bin/bash\n{body}\n")).unwrap();
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn env(&self) -> BTreeMap<String, String> {
        let mut e = BTreeMap::new();
        for (k, p) in [
            ("HOME", "home"),
            ("XDG_CACHE_HOME", "xdg"),
            ("KNOWLEDGE_MAIN", "main"),
            ("KNOWLEDGE_OUT", "out"),
            ("KNOWLEDGE_CACHE", "cache"),
            ("KNOWLEDGE_REPO", "clone"),
            ("KNOWLEDGE_CARDGEN", "bin/fake-cardgen"),
            ("CARGO_TARGET_DIR", "target"),
            ("KNOWLEDGE_PROC_ROOT", "proc"),
            ("TEST_ROOT", ""),
        ] {
            e.insert(k.into(), self.p(p).display().to_string());
        }
        for (k, v) in [
            ("KNOWLEDGE_SKIP_FETCH", "1"),
            ("KNOWLEDGE_SKIP_LLM", "1"),
            ("KNOWLEDGE_NO_SCOPE", "1"),
            ("KNOWLEDGE_DATE", "2026-11-22"),
            // inside the night window, whatever the clock says
            ("KNOWLEDGE_NOW_HOUR", "3"),
        ] {
            e.insert(k.into(), v.into());
        }
        e.insert("PATH".into(), format!("{}:/usr/bin:/bin", self.p("bin").display()));
        e
    }
    fn shell(&self, code: &str) -> Command {
        let mut c = capped("bash");
        c.args(["-c", code, "review", repo().join("scripts/knowledge-weekly.sh").to_str().unwrap()])
            .env_clear()
            .envs(
                std::env::vars().filter(|(k, _)| matches!(k.as_str(), "XDG_RUNTIME_DIR" | "DBUS_SESSION_BUS_ADDRESS")),
            )
            .envs(self.env());
        c
    }
    fn run(&self, args: &[&str]) -> Output {
        self.shell("exec bash \"$1\" \"${@:2}\"").args(args).output().unwrap()
    }
    fn fixture(&self) {
        for (from, to) in [
            ("new_oracle.json", "out/oracle.json"),
            ("old_oracle.json", "out/oracle.prev.json"),
            ("roles_now.json", "out/roles.json"),
            ("decks.json", "clone/data/decks.json"),
            ("scryfall_cache.json", "clone/data/scryfall_cache.json"),
        ] {
            std::fs::copy(repo().join("rust/crates/cardgen/tests/fixtures/weekly").join(from), self.p(to)).unwrap();
        }
    }
    fn report(&self, extra: &[&str]) -> Result<weekly::Output, String> {
        let mut a =
            vec!["--date".into(), "2026-11-22".into(), "--data-dir".into(), self.p("out").display().to_string()];
        a.extend(extra.iter().map(|s| (*s).into()));
        weekly::command(&Project::new(self.p("clone")), &a)
    }
    fn real_binary(&self) {
        self.stub("fake-cardgen", "exec \"$REAL_CARDGEN\" \"$@\"");
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn section<'a>(t: &'a str, title: &str) -> &'a str {
    let rest = t.split_once(&format!("## {title}\n")).unwrap().1;
    rest.split("\n## ").next().unwrap()
}

#[test]
fn defect_llm_stage_uses_valid_real_cardgen_arguments() {
    let s = Sandbox::new();
    s.fixture();
    s.real_binary();
    let o = s
        .shell("exec bash \"$1\" --stage 4")
        .env("KNOWLEDGE_SKIP_LLM", "0")
        .env("REAL_CARDGEN", env!("CARGO_BIN_EXE_cardgen"))
        .output()
        .unwrap();
    successful(o);
    let stages = s.read("cache/runs/2026-11-22/stages.tsv");
    assert!(!stages.contains("\tfail\t"), "stage must reach the stubbed GPU decision, without any server: {stages}");
}

#[test]
fn defect_failed_merge_preserves_previous_merged_roles() {
    let s = Sandbox::new();
    s.fixture();
    s.real_binary();
    s.write("out/llm_roles.jsonl", "malformed reviewed cache\n");
    let before = s.read("out/roles.json");
    successful(
        s.shell("exec bash \"$1\" --stage 3").env("REAL_CARDGEN", env!("CARGO_BIN_EXE_cardgen")).output().unwrap(),
    );
    assert!(s.read("cache/runs/2026-11-22/stages.tsv").contains("roles\tfail"));
    let after = s.read("out/roles.json");
    assert!(
        before == after,
        "failed stage replaced the previous table; before {} bytes, after {} bytes",
        before.len(),
        after.len()
    );
}

#[test]
fn defect_map_digest_failure_preserves_all_previous_outputs() {
    let s = Sandbox::new();
    s.fixture();
    s.write("out/map.md", "previous map");
    s.stub(
        "fake-cardgen",
        "case \" $* \" in *' map '*) echo replacement > \"$CARDGEN_KNOWLEDGE_DIR/map.md\"; exit 0;; *) exit 1;; esac",
    );
    successful(s.run(&["--stage", "5"]));
    assert!(s.read("cache/runs/2026-11-22/stages.tsv").contains("\tfail\t"));
    assert_eq!(s.read("out/map.md"), "previous map", "later digest failure must preserve the stage's previous outputs");
}

#[test]
fn defect_bad_decks_json_is_a_failed_map_digest_stage() {
    let s = Sandbox::new();
    s.fixture();
    s.write("clone/data/decks.json", "{");
    s.stub("fake-cardgen", "exit 0");
    successful(s.run(&["--stage", "5"]));
    let stages = s.read("cache/runs/2026-11-22/stages.tsv");
    assert!(stages.contains("map-digest\tfail"), "jq parse failure was swallowed: {stages}");
}

#[test]
fn defect_report_write_failure_keeps_previous_report_at_its_path() {
    let s = Sandbox::new();
    s.fixture();
    s.write("out/reports/2026-11-22.md", "previous report");
    // A file-size limit fails the replacement write but permits the preceding rename.
    // Ignore SIGXFSZ so Rust reports EFBIG; never change the test runner's limits.
    let o = s
        .shell(concat!(
            "trap '' XFSZ; ulimit -f 1; exec \"$REAL_CARDGEN\" --root \"$KNOWLEDGE_REPO\" ",
            "weekly-report --date 2026-11-22 --data-dir \"$KNOWLEDGE_OUT\""
        ))
        .env("REAL_CARDGEN", env!("CARGO_BIN_EXE_cardgen"))
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert_eq!(s.read("out/reports/2026-11-22.md"), "previous report");
}

#[test]
fn defect_report_rejects_impossible_calendar_dates() {
    let s = Sandbox::new();
    s.fixture();
    for date in ["2026-00-22", "2026-13-22", "2026-02-30", "2026-11-00"] {
        assert!(s.report(&["--date", date]).is_err(), "accepted impossible date {date}");
    }
}

#[test]
fn date_path_traversal_is_rejected_without_writing() {
    let s = Sandbox::new();
    s.fixture();
    for date in ["../escaped", "../../x", "2026-11-22/../../x", "2026-11-2\n"] {
        assert!(s.report(&["--date", date]).is_err());
    }
    assert!(!s.p("out/reports").exists());
}

#[test]
fn defect_whitespace_only_oracle_difference_is_not_errata() {
    let s = Sandbox::new();
    s.fixture();
    let mut cache: Value = serde_json::from_str(&s.read("clone/data/scryfall_cache.json")).unwrap();
    let now: Value = serde_json::from_str(&s.read("out/oracle.json")).unwrap();
    let c = now["cards"].as_array().unwrap().iter().find(|c| c["name"] == "Sky Counter").unwrap();
    cache["Sky Counter"] = c.clone();
    cache["Sky Counter"]["oracle_text"] = json!("Counter  target\nspell.");
    s.write("clone/data/scryfall_cache.json", &cache.to_string());
    let t = s.report(&[]).unwrap().text;
    assert!(!section(&t, "Action needed").contains("Sky Counter: oracle_text"), "{t}");
}

#[test]
fn defect_same_name_illegal_variant_does_not_shadow_legal_deck_card() {
    let s = Sandbox::new();
    s.fixture();
    let mut now: Value = serde_json::from_str(&s.read("out/oracle.json")).unwrap();
    let mut duplicate = now["cards"].as_array().unwrap().iter().find(|c| c["name"] == "Sky Counter").unwrap().clone();
    duplicate["oracle_id"] = json!("illegal-variant");
    duplicate["legalities"]["commander"] = json!("not_legal");
    now["cards"].as_array_mut().unwrap().insert(0, duplicate);
    now["meta"]["count"] = json!(now["cards"].as_array().unwrap().len());
    s.write("out/oracle.json", &now.to_string());
    std::fs::remove_file(s.p("out/oracle.prev.json")).unwrap();
    let t = s.report(&[]).unwrap().text;
    assert!(!section(&t, "Action needed").contains("**Sky Counter** is not legal"), "{t}");
}

#[test]
fn defect_markdown_in_card_name_cannot_create_report_sections() {
    let s = Sandbox::new();
    s.fixture();
    let mut now: Value = serde_json::from_str(&s.read("out/oracle.json")).unwrap();
    let cards = now["cards"].as_array_mut().unwrap();
    let c = cards.iter_mut().find(|c| c["name"] == "Fresh Bolt").unwrap();
    c["name"] = json!("Card\n\n## Forged section\n\n[click](https://example.invalid)");
    s.write("out/oracle.json", &now.to_string());
    let t = s.report(&[]).unwrap().text;
    assert!(!t.contains("\n## Forged section\n"), "unescaped name injected Markdown structure");
}

#[test]
fn defect_retention_keeps_most_recent_same_day_reports() {
    let s = Sandbox::new();
    s.fixture();
    for n in 0..15 {
        s.report(&["--commit", &n.to_string(), "--keep", "3"]).unwrap();
    }
    let texts: Vec<_> = std::fs::read_dir(s.p("out/reports"))
        .unwrap()
        .map(|p| std::fs::read_to_string(p.unwrap().path()).unwrap())
        .collect();
    for n in [12, 13, 14] {
        assert!(texts.iter().any(|t| t.contains(&format!("- Repository: {n}\n"))), "lost recent run {n}");
    }
}

#[test]
fn defect_fallback_report_retention_keeps_twelve_reports() {
    let s = Sandbox::new();
    for day in 1..=13 {
        successful(
            s.shell("exec bash \"$1\" --stage 6").env("KNOWLEDGE_DATE", format!("2026-11-{day:02}")).output().unwrap(),
        );
    }
    assert_eq!(
        std::fs::read_dir(s.p("out/reports")).unwrap().count(),
        12,
        "the short-report path must apply the same retention policy"
    );
}

#[test]
fn pending_cache_is_opt_in_even_when_corrupt_or_overlapping() {
    let s = Sandbox::new();
    let row = |primary: &str| {
        json!({"key":"k", "name":"Fixture", "roles":[primary], "primary":primary,
        "evidence":{primary:0}, "model":"m", "taxonomy":"t", "prompt":"p"})
        .to_string()
    };
    s.write("out/llm_roles.jsonl", &(row("draw") + "\n"));
    s.write("out/llm_roles.pending.jsonl", "malformed\n");
    assert_eq!(llm::load_cache(&s.p("out")).unwrap()["k"].primary.as_deref(), Some("draw"));
    assert!(llm::load_cache_with(&s.p("out"), true).is_err());
    s.write("out/llm_roles.pending.jsonl", &(row("draw") + "\n" + &row("removal") + "\n"));
    assert_eq!(llm::load_cache(&s.p("out")).unwrap()["k"].primary.as_deref(), Some("draw"));
    assert_eq!(llm::load_cache_with(&s.p("out"), true).unwrap()["k"].primary.as_deref(), Some("removal"));
}

#[test]
fn missing_lock_reconstruction_preserves_prev_and_truncated_bulk_is_rejected() {
    let s = Sandbox::new();
    let dir = s.p("out");
    let input = repo().join("rust/crates/cardgen/tests/fixtures/bulk_fixture.json");
    let mut opts = bulk::Options { dir: dir.clone(), from_file: None, min_count: 1, synced: "fixture".into() };
    let mut remote = bulk::LocalRemote { updated_at: "old".into(), path: input.clone() };
    bulk::run(&opts, &mut remote).unwrap();
    remote.updated_at = "current".into();
    bulk::run(&opts, &mut remote).unwrap();
    let before = (s.read("out/oracle.json"), s.read("out/oracle.prev.json"));
    std::fs::remove_file(dir.join("sync.lock")).unwrap();
    assert!(bulk::run(&opts, &mut remote).unwrap().up_to_date);
    assert_eq!((s.read("out/oracle.json"), s.read("out/oracle.prev.json")), before);
    opts.min_count = 25000;
    remote.updated_at = "newer".into();
    assert!(bulk::run(&opts, &mut remote).is_err());
    assert_eq!((s.read("out/oracle.json"), s.read("out/oracle.prev.json")), before);
    opts.min_count = 1;
    remote.updated_at = "current".into();
    let count = bulk::read_meta(&dir.join("oracle.json")).unwrap().count;
    let lying = before.0.replacen(&format!("\"count\": {count}"), &format!("\"count\": {}", count + 1), 1);
    assert_ne!(lying, before.0, "the damage must actually be applied");
    s.write("out/oracle.json", &lying);
    std::fs::remove_file(dir.join("sync.lock")).unwrap();
    assert!(!bulk::run(&opts, &mut remote).unwrap().up_to_date);
    for lock in ["corrupt\n", "schema=2\nbulk_updated_at=current\ncount=38\nfnv1a64=0000000000000000\n"] {
        s.write("out/sync.lock", lock);
        assert!(!bulk::run(&opts, &mut remote).unwrap().up_to_date);
    }
}

#[test]
fn script_default_count_floor_rejects_a_small_bulk_without_changing_catalog() {
    let s = Sandbox::new();
    s.fixture();
    s.real_binary();
    let before = (s.read("out/oracle.json"), s.read("out/oracle.prev.json"));
    successful(
        s.shell("exec bash \"$1\" --stage 2")
            .env("REAL_CARDGEN", env!("CARGO_BIN_EXE_cardgen"))
            .env("KNOWLEDGE_BULK_FILE", repo().join("rust/crates/cardgen/tests/fixtures/bulk_fixture.json"))
            .env("KNOWLEDGE_BULK_UPDATED_AT", "new")
            .output()
            .unwrap(),
    );
    assert!(s.read("cache/runs/2026-11-22/stages.tsv").contains("bulk-sync\tfail"));
    assert!(s.read("cache/runs/2026-11-22/bulk-sync.log").contains("minimum 25000"));
    assert_eq!((s.read("out/oracle.json"), s.read("out/oracle.prev.json")), before);
}

#[test]
fn roles_merge_uses_tracked_only_cache_and_dense_model_id() {
    let s = Sandbox::new();
    s.fixture();
    s.write("out/llm_roles.jsonl", "tracked cache has rows\n");
    s.write("out/llm_roles.pending.jsonl", "malformed pending rows stay out\n");
    s.stub("fake-cardgen", "printf '%s\\n' \"$*\" >> \"$TEST_ROOT/arguments\"; exit 0");
    successful(s.run(&["--stage", "3"]));
    let calls = s.read("arguments");
    assert!(calls.contains("roles --merge --no-think --model Qwen3.6-27B-Q5_K_M.gguf"), "{calls}");
    assert!(!calls.contains("--pending"), "{calls}");
}

fn snapshot(root: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime, Option<Vec<u8>>)> {
    fn walk(p: &Path, v: &mut Vec<(PathBuf, u64, std::time::SystemTime, Option<Vec<u8>>)>) {
        let m = std::fs::symlink_metadata(p).unwrap();
        v.push((
            p.into(),
            m.len(),
            m.modified().unwrap(),
            if m.is_file() { Some(std::fs::read(p).unwrap()) } else { None },
        ));
        if m.is_dir() {
            for f in std::fs::read_dir(p).unwrap() {
                walk(&f.unwrap().path(), v);
            }
        }
    }
    let mut v = vec![];
    walk(root, &mut v);
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

#[test]
fn dry_runs_are_byte_and_metadata_read_only_with_hostile_paths_and_refs() {
    let s = Sandbox::new();
    let before = snapshot(&s.0);
    for args in [
        vec!["--dry-run"],
        vec!["--dry-run", "--stage", "1"],
        vec!["--only", "roles", "--dry-run"],
        vec!["--dry-run", "--stage", "3", "--only", "report"],
        vec!["--dry-run", "--only", "llm", "--only", "map-digest"],
    ] {
        successful(
            s.shell("exec bash \"$1\" \"${@:2}\"")
                .args(args)
                .env("KNOWLEDGE_OUT", s.p("space ' quote \" $(touch INJECTED)\nnewline"))
                .env("KNOWLEDGE_REF", "origin/main; $(touch INJECTED); `touch INJECTED`")
                .output()
                .unwrap(),
        );
        assert_eq!(snapshot(&s.0), before);
    }
}

#[test]
fn fallback_report_survives_failed_notification_and_paths_with_newlines() {
    let s = Sandbox::new();
    let real_store = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/commander-sim-knowledge"));
    let real_before = real_store.as_ref().filter(|p| p.exists()).map(|p| snapshot(p));
    let out = s.p("output with ' \" quotes\nand newline");
    successful(s.shell("exec bash \"$1\"").env("KNOWLEDGE_OUT", &out).output().unwrap());
    assert!(out.join("reports/2026-11-22.md").is_file());
    assert!(
        std::fs::read_dir(out.join("reports")).unwrap().all(|p| !p
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".fallback."))
    );
    assert!(s.p("calls").is_file()); // notify-send was attempted and failed; report still won
    assert!(!s.p("home/.cache/commander-sim-knowledge").exists());
    assert_eq!(
        real_store.as_ref().filter(|p| p.exists()).map(|p| snapshot(p)),
        real_before,
        "the real HOME store must remain untouched"
    );
}

#[test]
fn stale_flock_file_is_not_an_active_lock_and_unknown_options_write_nothing() {
    let s = Sandbox::new();
    s.write("cache/lock", "stale");
    successful(s.run(&["--stage", "6"]));
    assert!(s.p("out/reports/2026-11-22.md").is_file());
    let before = snapshot(&s.0);
    for args in [vec!["--stage"], vec!["--stage", "0"], vec!["--only", "unknown"], vec!["--unknown"]] {
        assert!(!s.run(&args).status.success());
        assert_eq!(snapshot(&s.0), before);
    }
}

#[test]
fn defect_readonly_cache_still_writes_a_failure_report() {
    let s = Sandbox::new();
    std::fs::create_dir(s.p("cache")).unwrap();
    std::fs::set_permissions(s.p("cache"), std::fs::Permissions::from_mode(0o555)).unwrap();
    let o = s.run(&[]);
    // Restore permissions before Sandbox::drop, including when the assertion fails.
    std::fs::set_permissions(s.p("cache"), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        o.status.success() && s.p("out/reports/2026-11-22.md").is_file(),
        "writable output, read-only state: {:?}: {}",
        o.status,
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn report_failure_exits_nonzero_and_a_borrowed_server_is_never_stopped() {
    let s = Sandbox::new();
    s.write("blocked", "not a directory");
    assert!(!s.shell("exec bash \"$1\"").env("KNOWLEDGE_OUT", s.p("blocked")).output().unwrap().status.success());
    llm_stubs(&s);
    s.write("server-up", "borrowed");
    s.stub("fake-cardgen", "case \" $* \" in *' --dry-run '*) echo '1 to classify'; exit 0;; esac; exit 1");
    successful(
        s.shell("exec bash \"$1\" --stage 4")
            .env("KNOWLEDGE_SKIP_LLM", "0")
            .env("KNOWLEDGE_GPU_DEVICE", s.p("gpu"))
            .output()
            .unwrap(),
    );
    assert_eq!(s.read("server-up"), "borrowed");
    assert!(!std::fs::read_to_string(s.p("calls")).unwrap_or_default().contains(" stop "));
}

#[test]
fn defect_sigkill_at_roles_merge_boundary_preserves_previous_table() {
    let s = Sandbox::new();
    s.fixture();
    let before = s.read("out/roles.json");
    s.write("out/llm_roles.jsonl", "reviewed rows exist\n");
    s.stub(
        "fake-cardgen",
        r#"
case " $* " in
  *' --rules '*) echo replacement > "$CARDGEN_KNOWLEDGE_DIR/roles.json";;
  *' --merge '*) touch "$TEST_ROOT/boundary"; exec sleep 30;;
esac
"#,
    );
    let mut child = s
        .shell("exec setsid bash -c 'echo $$ > \"$TEST_ROOT/pid\"; exec bash \"$1\" --stage 3' review \"$1\"")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let wait = Instant::now();
    while !s.p("boundary").exists() && wait.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let reached = s.p("boundary").exists();
    let pid = s.read("pid").trim().to_string();
    let _ = Command::new("kill").args(["-s", "KILL", "--", &format!("-{pid}")]).status();
    let _ = child.wait();
    assert!(reached, "merge boundary never reached");
    assert!(s.read("out/roles.json") == before, "SIGKILL between rules and merge lost previous merged roles");
}

#[test]
fn defect_term_and_kill_at_each_stage_boundary_preserve_published_outputs() {
    // Run every case before asserting, so one failure cannot hide the other stage boundaries.
    let mut failures = vec![];
    for stage in 1..=6 {
        for signal in ["TERM", "KILL"] {
            let s = Sandbox::new();
            llm_stubs(&s);
            std::fs::create_dir(s.p("clone/.git")).unwrap();
            s.write("server-up", "borrowed server");
            s.write("out/map.md", "previous map");
            s.write("out/reports/2026-11-22.md", "previous report");
            s.write("out/llm_roles.jsonl", "reviewed rows\n");
            let before: BTreeMap<_, _> = snapshot(&s.p("out"))
                .into_iter()
                .filter_map(|(p, _, _, bytes)| bytes.map(|b| (p.strip_prefix(s.p("out")).unwrap().to_path_buf(), b)))
                .collect();
            s.stub(
                "git",
                "case \" $* \" in *' fetch '*) touch \"$TEST_ROOT/boundary\"; exec sleep 30;; *) echo fixture;; esac",
            );
            s.stub(
                "fake-cardgen",
                r#"
case " $* " in
  *' --dry-run '*) echo '1 to classify'; exit 0;;
esac
case "$TEST_STAGE" in
  2) echo staged > "$CARDGEN_KNOWLEDGE_DIR/oracle.staged.tmp";;
  3) case " $* " in *' --rules '*) echo replacement > "$CARDGEN_KNOWLEDGE_DIR/roles.json"; exit 0;; esac;;
  4) echo staged > "$CARDGEN_KNOWLEDGE_DIR/llm_roles.pending.staged.tmp";;
  5) case " $* " in *' map '*) echo replacement > "$CARDGEN_KNOWLEDGE_DIR/map.md"; exit 0;; esac;;
  6) echo staged > "$CARDGEN_KNOWLEDGE_DIR/reports/report.staged.tmp";;
esac
touch "$TEST_ROOT/boundary"
exec sleep 30
"#,
            );
            let mut child = s
                .shell(concat!(
                    "exec setsid bash -c 'echo $$ > \"$TEST_ROOT/pid\"; ",
                    "exec bash \"$1\" --stage \"$TEST_STAGE\"' review \"$1\""
                ))
                .env("TEST_STAGE", stage.to_string())
                .env("KNOWLEDGE_SKIP_LLM", "0")
                .env("KNOWLEDGE_SKIP_FETCH", if stage == 1 { "0" } else { "1" })
                .env("KNOWLEDGE_GPU_DEVICE", s.p("gpu"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let wait = Instant::now();
            while !s.p("boundary").exists() && wait.elapsed() < Duration::from_secs(10) {
                std::thread::sleep(Duration::from_millis(20));
            }
            let reached = s.p("boundary").exists();
            let pid = s.read("pid").trim().to_string();
            assert!(Command::new("kill").args(["-s", signal, "--", &format!("-{pid}")]).status().unwrap().success());
            let _ = child.wait();
            assert!(reached, "stage {stage} boundary not reached");
            let after: BTreeMap<_, _> = snapshot(&s.p("out"))
                .into_iter()
                .filter_map(|(p, _, _, bytes)| bytes.map(|b| (p.strip_prefix(s.p("out")).unwrap().to_path_buf(), b)))
                .collect();
            if before != after {
                let changed: Vec<_> =
                    before.keys().chain(after.keys()).filter(|k| before.get(*k) != after.get(*k)).collect();
                failures.push(format!("stage {stage} {signal}: changed/partial {changed:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn defect_unset_home_dry_run_is_handled_without_unbound_variable() {
    let s = Sandbox::new();
    let o = s
        .shell("exec bash \"$1\" --dry-run")
        .env_remove("HOME")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("KNOWLEDGE_MAIN")
        .env_remove("KNOWLEDGE_CACHE")
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&o.stderr).contains("unbound variable"), "{}", String::from_utf8_lossy(&o.stderr));
}

/// a fake /proc: desktop-like holders of the `gpu` file that use no video memory ("free"), one that holds 21 GiB
/// ("busy"), or one whose fdinfo has no `drm-resident-vram0` ("unknown")
fn gpu_tree(s: &Sandbox, kind: &str) {
    let gpu = s.p("gpu");
    let _ = std::fs::remove_dir_all(s.p("proc"));
    let holders: Vec<(&str, String)> = match kind {
        "busy" => vec![("30/3", "drm-client-id:\t1\ndrm-resident-vram0:\t21982932 KiB\n".into())],
        "unknown" => vec![("30/3", "drm-client-id:\t1\n".into())],
        _ => (0..3).map(|i| ("30/3", format!("drm-client-id:\t{i}\ndrm-resident-vram0:\t0 KiB\n"))).collect(),
    };
    for (i, (_, text)) in holders.iter().enumerate() {
        let (pid, fd) = (30 + i, 3 + i);
        std::fs::create_dir_all(s.p(&format!("proc/{pid}/fd"))).unwrap();
        std::fs::create_dir_all(s.p(&format!("proc/{pid}/fdinfo"))).unwrap();
        std::os::unix::fs::symlink(&gpu, s.p(&format!("proc/{pid}/fd/{fd}"))).unwrap();
        s.write(&format!("proc/{pid}/fdinfo/{fd}"), text);
    }
}

fn llm_stubs(s: &Sandbox) {
    s.fixture();
    s.write("gpu", "");
    s.stub(
        "fake-cardgen",
        r#"
case " $* " in *' --dry-run '*) echo '1 to classify'; exit 0;; esac
touch "$TEST_ROOT/in-flight"
exec sleep 30
"#,
    );
    s.stub(
        "curl",
        r#"
case "$*" in
  *chat/completions*) echo '{"choices":[{"message":{"content":"OK"}}]}';;
  *) [ -f "$TEST_ROOT/server-up" ] || exit 7; echo '{"data":[{"id":"Qwen3.6-27B-Q5_K_M.gguf"}]}';;
esac
"#,
    );
    gpu_tree(s, "free");
    s.stub("ss", "exit 0");
    s.stub(
        "systemctl",
        r#"
echo "$*" >> "$TEST_ROOT/calls"
case "$2" in start) touch "$TEST_ROOT/server-up";; stop) rm -f "$TEST_ROOT/server-up";; esac
exit 0
"#,
    );
}

fn interrupted_llm(signal: &str, group: bool) -> (bool, String) {
    let s = Sandbox::new();
    llm_stubs(&s);
    let mut child = s
        .shell("exec setsid bash -c 'echo $$ > \"$TEST_ROOT/pid\"; exec bash \"$1\" --stage 4' review \"$1\"")
        .env("KNOWLEDGE_SKIP_LLM", "0")
        .env("KNOWLEDGE_GPU_DEVICE", s.p("gpu"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let wait = Instant::now();
    while !s.p("in-flight").exists() && wait.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(s.p("in-flight").exists(), "stub stage did not start");
    let pid = s.read("pid").trim().to_string();
    let target = if group { format!("-{pid}") } else { pid.clone() };
    assert!(Command::new("kill").args(["-s", signal, "--", &target]).status().unwrap().success());
    let wait = Instant::now();
    while s.p("server-up").exists() && wait.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let left_running = s.p("server-up").exists();
    let calls = s.read("calls");
    // Always reap the hermetic fake job, even for the expected failing assertion in the caller.
    let _ = Command::new("kill").args(["-s", "KILL", "--", &format!("-{pid}")]).status();
    let _ = child.wait();
    (left_running, calls)
}

#[test]
fn term_to_process_group_cleans_up_owned_server() {
    let (running, calls) = interrupted_llm("TERM", true);
    assert!(!running, "{calls}");
}

#[test]
fn defect_another_starter_between_probe_and_start_keeps_its_server() {
    let s = Sandbox::new();
    llm_stubs(&s);
    s.stub("fake-cardgen", "case \" $* \" in *' --dry-run '*) echo '1 to classify';; esac; exit 0");
    // The port/GPU probes have already finished when the unit lookup occurs.
    // Another actor now activates the named unit; systemctl start on an active unit is a no-op.
    s.stub(
        "systemctl",
        r#"
echo "$*" >> "$TEST_ROOT/calls"
case "$2" in cat) touch "$TEST_ROOT/server-up";; stop) rm -f "$TEST_ROOT/server-up";; esac
exit 0
"#,
    );
    successful(
        s.shell("exec bash \"$1\" --stage 4")
            .env("KNOWLEDGE_SKIP_LLM", "0")
            .env("KNOWLEDGE_GPU_DEVICE", s.p("gpu"))
            .output()
            .unwrap(),
    );
    assert!(s.p("server-up").exists(), "stopped the other actor's server: {}", s.read("calls"));
}

#[test]
fn defect_failed_gpu_probe_does_not_authorize_server_start() {
    let s = Sandbox::new();
    llm_stubs(&s);
    // a holder of the render node whose fdinfo does not report video memory: the state cannot be established
    gpu_tree(&s, "unknown");
    let decision =
        successful(s.shell("source \"$1\"; llm_decide").env("KNOWLEDGE_GPU_DEVICE", s.p("gpu")).output().unwrap());
    assert_ne!(decision.trim(), "start", "an unreadable GPU state cannot establish that the GPU is free");
}

#[test]
fn defect_term_to_job_pid_promptly_cleans_up_owned_server() {
    let (running, calls) = interrupted_llm("TERM", false);
    assert!(!running, "TERM to job PID left the owned server running: {calls}");
}

#[test]
fn defect_sigkill_has_unit_level_owned_server_cleanup() {
    let (running, calls) = interrupted_llm("KILL", true);
    // The stubbed separately managed server represents a unit outside the weekly cgroup.
    let unit = std::fs::read_to_string(repo().join("scripts/systemd/knowledge-weekly.service")).unwrap();
    assert!(
        !running || unit.lines().any(|l| l.starts_with("ExecStopPost=")),
        "SIGKILL left a separate server unit running; weekly has no manager-side cleanup: {calls}"
    );
}

#[test]
fn independent_python_fixture_report_recomputation_and_determinism() {
    let s = Sandbox::new();
    s.fixture();
    let t = s.report(&[]).unwrap().text;
    s.write("report.md", &t);
    successful(
        capped("python3")
            .arg("-I")
            .arg(repo().join("rust/crates/cardgen/tests/fixtures/weekly_adversarial/recompute.py"))
            .arg(&s.0)
            .output()
            .unwrap(),
    );
    assert_eq!(s.report(&[]).unwrap().text, t);
}

#[test]
fn units_verify_with_overlay_home_and_timer_is_sunday_four_am() {
    let s = Sandbox::new();
    let home = s.p("home");
    for p in ["projects", "llama.cpp/build-sycl/bin", ".config"] {
        std::fs::create_dir_all(home.join(p)).unwrap();
    }
    std::os::unix::fs::symlink(repo(), home.join("projects/commander-sim")).unwrap();
    std::os::unix::fs::symlink("/bin/true", home.join("llama.cpp/build-sycl/bin/llama-server")).unwrap();
    s.write("home/.config/llama-server.env", "");
    let units = repo().join("scripts/systemd");
    successful(
        capped("systemd-analyze")
            .args(["--user", "verify"])
            .args([
                units.join("knowledge-weekly.service"),
                units.join("knowledge-weekly.timer"),
                units.join("llama-27b.service"),
            ])
            .env("HOME", &home)
            .output()
            .unwrap(),
    );
    let calendar = successful(capped("systemd-analyze").args(["calendar", "Sun *-*-* 04:00:00"]).output().unwrap());
    assert!(calendar.contains("Sun *-*-* 04:00:00"));
    let service = std::fs::read_to_string(units.join("knowledge-weekly.service")).unwrap();
    for line in ["Type=exec", "RuntimeMaxSec=4h", "MemoryMax=24G", "MemorySwapMax=0", "WorkingDirectory=%h"] {
        assert!(service.lines().any(|l| l == line), "missing {line}");
    }
    let timer = std::fs::read_to_string(units.join("knowledge-weekly.timer")).unwrap();
    for line in ["OnCalendar=Sun *-*-* 04:00:00", "Persistent=true", "RandomizedDelaySec=10min"] {
        assert!(timer.lines().any(|l| l == line));
    }
    let server = std::fs::read_to_string(units.join("llama-27b.service")).unwrap();
    assert!(!server.contains("[Install]"));
    assert!(!server.lines().any(|l| l.starts_with("Restart=")));
}

#[test]
fn unchanged_catalog_is_quiet_and_huge_new_card_lists_are_capped() {
    let s = Sandbox::new();
    s.fixture();
    let now = s.read("out/oracle.json");
    s.write("out/oracle.prev.json", &now);
    let t = s.report(&[]).unwrap().text;
    assert!(section(&t, "New cards").contains("0 new Oracle cards (0 Commander-legal), 0 gone"));
    let mut value: Value = serde_json::from_str(&now).unwrap();
    let base = value["cards"][0].clone();
    let cards = value["cards"].as_array_mut().unwrap();
    for i in 0..2000 {
        let mut c = base.clone();
        c["oracle_id"] = json!(format!("fresh-{i}"));
        c["name"] = json!(format!("New card {i:04}"));
        c["set"] = json!("test-set");
        cards.push(c);
    }
    value["meta"]["count"] = json!(cards.len());
    s.write("out/oracle.json", &value.to_string());
    let t = s.report(&[]).unwrap().text;
    assert!(t.contains("2000 new Oracle cards"));
    assert!(t.contains("(1975 more)") && t.contains("(1960 more)"));
    assert!(t.len() < 20000, "uncapped huge report: {} bytes", t.len());
}

#[test]
fn renamed_missing_and_double_faced_deck_cards_are_reported_or_resolved() {
    let s = Sandbox::new();
    s.fixture();
    let mut now: Value = serde_json::from_str(&s.read("out/oracle.json")).unwrap();
    let cards = now["cards"].as_array_mut().unwrap();
    cards.iter_mut().find(|c| c["name"] == "Sky Counter").unwrap()["name"] = json!("Renamed Counter");
    let face = cards.iter_mut().find(|c| c["name"] == "Red Burn").unwrap();
    face["name"] = json!("Red Burn // Blue Face");
    face["layout"] = json!("transform");
    face["card_faces"] = json!([{"name":"Red Burn", "oracle_text":"Deal 3 damage."},
        {"name":"Blue Face", "oracle_text":"Draw a card."}]);
    s.write("out/oracle.json", &now.to_string());
    let t = s.report(&[]).unwrap().text;
    let a = section(&t, "Action needed");
    assert!(a.contains("not found in the catalog: Sky Counter"));
    assert!(!a.contains("not found in the catalog: Red Burn"));
}

#[test]
#[ignore = "needs data/knowledge/oracle.json"]
fn real_catalog_independent_python_recomputation() {
    let s = Sandbox::new();
    for (from, to) in [
        ("data/knowledge/oracle.json", "out/oracle.json"),
        ("data/decks.json", "clone/data/decks.json"),
        ("data/scryfall_cache.json", "clone/data/scryfall_cache.json"),
        ("data/knowledge/roles.json", "out/roles.json"),
    ] {
        std::fs::copy(repo().join(from), s.p(to)).unwrap();
    }
    let prepare = r#"
import json, pathlib, sys
p=pathlib.Path(sys.argv[1]); cur=json.loads((p/'out/oracle.json').read_text())
prev=json.loads(json.dumps(cur)); decks=json.loads((p/'clone/data/decks.json').read_text())['mine']
names={n for d in decks for n in d['cards']+[d['commander']]+([d['commander2']] if d.get('commander2') else [])}
chosen=[i for i,c in enumerate(prev['cards']) if c['name'] in names and c['legalities']['commander']=='legal'][:6]
assert len(chosen)==6
cur['cards'][chosen[0]]['legalities']['commander']='banned'
cur['cards'][chosen[1]]['legalities']['commander']='not_legal'
prev['cards'][chosen[2]]['legalities']['commander']='banned'
cur['cards'][chosen[3]]['game_changer']=True; prev['cards'][chosen[3]]['game_changer']=False
cur['cards'][chosen[4]]['game_changer']=False; prev['cards'][chosen[4]]['game_changer']=True
prev['cards'][chosen[5]]['oracle_id']='synthetic-previous-id'
prev['cards'][chosen[5]]['set']='synthetic-set'
cur['cards'][chosen[5]]['oracle_text']='Synthetic changed Oracle text'
(p/'out/oracle.json').write_text(json.dumps(cur))
(p/'out/oracle.prev.json').write_text(json.dumps(prev))
"#;
    successful(capped("python3").args(["-I", "-c", prepare]).arg(&s.0).output().unwrap());
    s.write("report.md", &s.report(&[]).unwrap().text);
    successful(
        capped("python3")
            .arg("-I")
            .arg(repo().join("rust/crates/cardgen/tests/fixtures/weekly_adversarial/recompute.py"))
            .arg(&s.0)
            .output()
            .unwrap(),
    );
}
