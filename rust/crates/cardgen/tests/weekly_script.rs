//! `scripts/knowledge-weekly.sh` without the network, a GPU or systemd: a local bulk file stands in for Scryfall, the
//! built `cardgen` is used as is, the LLM stage is skipped or driven through stubs of `curl`, `fuser`, `ss` and
//! `systemctl` placed first in PATH. The decision logic (use a running server, skip on another model or a busy GPU,
//! start and stop only what the job started) is tested as the script's own functions.

use cardgen::knowledge::Index;
use cardgen::llm::{self, Row, Variant};
use cardgen::roles::{self, Taxonomy};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// the repository this test runs in (read-only for the job: it is the clone's stand-in)
fn repo_root() -> PathBuf {
    manifest().join("../../..").canonicalize().unwrap()
}

fn script() -> PathBuf {
    repo_root().join("scripts/knowledge-weekly.sh")
}

const BULK_AT: &str = "2026-11-14T10:00:00.000+00:00";
const BULK_AT2: &str = "2026-11-21T10:00:00.000+00:00";

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new() -> Sandbox {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "weekly-script-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(root.join("home")).unwrap();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        // the knowledge directory with the tracked role files the job reads
        std::fs::create_dir_all(root.join("out")).unwrap();
        std::fs::copy(repo_root().join("data/knowledge/roles.toml"), root.join("out/roles.toml")).unwrap();
        Sandbox { root }
    }

    fn p(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn out(&self, rel: &str) -> PathBuf {
        self.root.join("out").join(rel)
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.out(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
    }

    /// environment of a smoke run: everything under the sandbox, the built cardgen, a local bulk file, no scope
    fn env(&self) -> BTreeMap<String, String> {
        let mut e = BTreeMap::new();
        let s = |p: PathBuf| p.display().to_string();
        e.insert("HOME".into(), s(self.p("home")));
        e.insert("KNOWLEDGE_MAIN".into(), s(self.p("main")));
        e.insert("KNOWLEDGE_OUT".into(), s(self.p("out")));
        e.insert("KNOWLEDGE_CACHE".into(), s(self.p("cache")));
        e.insert("KNOWLEDGE_REPO".into(), s(repo_root()));
        e.insert("KNOWLEDGE_SKIP_FETCH".into(), "1".into());
        e.insert("KNOWLEDGE_CARDGEN".into(), env!("CARGO_BIN_EXE_cardgen").into());
        e.insert("KNOWLEDGE_BULK_FILE".into(), s(manifest().join("tests/fixtures/bulk_fixture.json")));
        e.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT.into());
        e.insert("KNOWLEDGE_BULK_MIN".into(), "1".into());
        e.insert("KNOWLEDGE_NO_SCOPE".into(), "1".into());
        e.insert("KNOWLEDGE_SKIP_LLM".into(), "1".into());
        e.insert("KNOWLEDGE_DATE".into(), "2026-11-22".into());
        e.insert("CARDGEN_KNOWLEDGE_DIR".into(), String::new());
        e.insert("CARGO_TARGET_DIR".into(), s(self.p("target")));
        e
    }

    fn run(&self, args: &[&str], env: &BTreeMap<String, String>) -> Output {
        Command::new("bash").arg(script()).args(args).envs(env).stdin(Stdio::null()).output().unwrap()
    }

    fn ok(&self, args: &[&str], env: &BTreeMap<String, String>) -> String {
        let o = self.run(args, env);
        let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success(), "exit {:?}:\n{text}", o.status.code());
        text
    }

    fn stages(&self, date: &str) -> Vec<(String, String, String)> {
        let text = std::fs::read_to_string(self.p(&format!("cache/runs/{date}/stages.tsv"))).unwrap();
        text.lines()
            .map(|l| {
                let f: Vec<&str> = l.splitn(4, '\t').collect();
                (f[0].to_string(), f[1].to_string(), f.get(3).unwrap_or(&"").to_string())
            })
            .collect()
    }

    /// a stub program in the sandbox's bin directory
    fn stub(&self, name: &str, body: &str) {
        let path = self.p("bin").join(name);
        std::fs::write(&path, format!("#!/usr/bin/env bash\n{body}\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn with_stubs_first(&self, env: &mut BTreeMap<String, String>) {
        let path = std::env::var("PATH").unwrap_or_default();
        env.insert("PATH".into(), format!("{}:{path}", self.p("bin").display()));
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn git_status(root: &Path) -> Option<String> {
    let o = Command::new("git").arg("-C").arg(root).args(["status", "--porcelain"]).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

fn listing(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                out.push(p.strip_prefix(base).unwrap().display().to_string());
            }
        }
    }
    let mut v = vec![];
    walk(dir, dir, &mut v);
    v.sort();
    v
}

// ---------------------------------------------------------------------------------------- dry run and arguments

#[test]
fn dry_run_prints_every_stage_and_writes_and_calls_nothing() {
    let sb = Sandbox::new();
    let mut env = sb.env();
    env.remove("KNOWLEDGE_SKIP_LLM");
    // a stub that would record any call: the dry run must make none
    sb.stub("systemctl", &format!("echo systemctl \"$@\" >> {}/calls", sb.root.display()));
    sb.stub("curl", &format!("echo curl \"$@\" >> {}/calls; exit 7", sb.root.display()));
    sb.stub("fuser", &format!("echo fuser \"$@\" >> {}/calls; exit 1", sb.root.display()));
    sb.with_stubs_first(&mut env);
    let before = (listing(&sb.root), git_status(&repo_root()));
    let text = sb.ok(&["--dry-run"], &env);
    for h in [
        "stage 1 fetch-build:",
        "stage 2 bulk-sync:",
        "stage 3 roles:",
        "stage 4 llm:",
        "stage 5 map-digest:",
        "stage 6 report:",
    ] {
        assert!(text.contains(h), "{h} in:\n{text}");
    }
    assert!(
        text.contains("DRY RUN") && text.contains("Qwen3.6-27B") && text.contains("llm_roles.pending.jsonl"),
        "{text}"
    );
    assert_eq!(before, (listing(&sb.root), git_status(&repo_root())), "nothing written");
    assert!(!sb.p("calls").exists(), "no curl, fuser or systemctl call");
    assert!(!sb.p("cache").exists(), "no state directory either");
    // one stage only
    let text = sb.ok(&["--dry-run", "--only", "roles"], &env);
    assert!(text.contains("stage 3 roles:") && !text.contains("stage 2 bulk-sync:"), "{text}");
}

#[test]
fn bad_arguments_are_refused_and_help_is_printed() {
    let sb = Sandbox::new();
    let env = sb.env();
    for args in [&["--stage", "9"][..], &["--stage"], &["--only", "nope"], &["--nope"]] {
        let o = sb.run(args, &env);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }
    let o = sb.run(&["--help"], &env);
    let help = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(o.status.success() && help.contains("NEVER AUTOMATED") && help.contains("--dry-run"), "{help}");
    assert!(!sb.p("cache").exists());
}

// ------------------------------------------------------------------------------------------------ the full smoke run

#[test]
fn a_full_offline_run_writes_only_untracked_outputs_a_report_and_exits_zero() {
    let sb = Sandbox::new();
    let env = sb.env();
    let tracked_before = git_status(&repo_root());
    let text = sb.ok(&[], &env);
    let stages = sb.stages("2026-11-22");
    let names: Vec<(&str, &str)> = stages.iter().map(|s| (s.0.as_str(), s.1.as_str())).collect();
    assert_eq!(
        names,
        [("fetch-build", "ok"), ("bulk-sync", "ok"), ("roles", "ok"), ("llm", "skip"), ("map-digest", "ok")],
        "{text}\n{stages:?}"
    );
    assert!(text.contains("report: ") && text.contains("2026-11-22.md"), "{text}");
    // everything the job wrote is a generated file
    let files = listing(&sb.p("out"));
    for want in ["oracle.json", "sync.lock", "roles.json", "map.md", "reports/2026-11-22.md"] {
        assert!(files.iter().any(|f| f == want), "{want} in {files:?}");
    }
    assert!(files.iter().any(|f| f.starts_with("digests/") && f.ends_with(".txt")), "{files:?}");
    let allowed = ["oracle.json", "oracle.prev.json", "sync.lock", "roles.json", "map.md", "roles.toml"];
    for f in &files {
        assert!(
            allowed.contains(&f.as_str()) || f.starts_with("digests/") || f.starts_with("reports/"),
            "unexpected output {f}"
        );
    }
    assert_eq!(tracked_before, git_status(&repo_root()), "the repository's working tree is untouched");
    let report = sb.read("reports/2026-11-22.md");
    assert!(report.starts_with("# Card knowledge: weekly report 2026-11-22"), "{report}");
    assert!(report.contains("first run") && report.contains("| llm | skip | 0 | KNOWLEDGE_SKIP_LLM=1 |"), "{report}");
    assert!(sb.read("oracle.json").contains(BULK_AT));

    // a second run with the same bulk date: nothing new, the previous catalog is not replaced, exit 0
    let (oracle, roles) = (sb.read("oracle.json"), sb.read("roles.json"));
    let text = sb.ok(&[], &env);
    let stages = sb.stages("2026-11-22");
    assert_eq!(stages[1].1, "skip", "{text}\n{stages:?}");
    assert!(stages[1].2.contains("up to date"), "{stages:?}");
    assert_eq!((sb.read("oracle.json"), sb.read("roles.json")), (oracle, roles), "deterministic and untouched");
    let report = sb.read("reports/2026-11-22.md");
    assert!(report.contains("the bulk file did not change this run, so there are no new cards"), "{report}");
    assert!(sb.out("reports/2026-11-22.1.md").is_file(), "the earlier report of the day is kept");
    assert!(!sb.out("oracle.prev.json").exists(), "a skipped sync does not make a previous catalog");

    // a newer bulk date: the old catalog becomes oracle.prev.json and the report compares the two
    let mut env2 = env.clone();
    env2.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    sb.ok(&[], &env2);
    assert!(sb.read("oracle.prev.json").contains(BULK_AT) && sb.read("oracle.json").contains(BULK_AT2));
    let report = sb.read("reports/2026-11-22.md");
    assert!(report.contains(&format!("Bulk file updated_at: {BULK_AT} -> {BULK_AT2}")), "{report}");
    assert!(report.contains("0 new Oracle cards"), "{report}");
    assert_eq!(tracked_before, git_status(&repo_root()));
}

#[test]
fn a_failed_stage_is_recorded_the_previous_outputs_stay_and_the_exit_code_is_zero() {
    let sb = Sandbox::new();
    let env = sb.env();
    sb.ok(&[], &env);
    let before: Vec<(String, Vec<u8>)> = ["oracle.json", "sync.lock", "roles.json", "map.md"]
        .iter()
        .map(|f| (f.to_string(), std::fs::read(sb.out(f)).unwrap()))
        .collect();
    // the next bulk file is unreadable: the sync fails, everything else still runs on the old catalog
    let mut bad = env.clone();
    bad.insert("KNOWLEDGE_BULK_FILE".into(), sb.p("missing.json").display().to_string());
    bad.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    let text = sb.ok(&[], &bad);
    let stages = sb.stages("2026-11-22");
    assert_eq!(stages[1].0, "bulk-sync");
    assert_eq!(stages[1].1, "fail", "{stages:?}\n{text}");
    for (f, bytes) in &before {
        assert_eq!(&std::fs::read(sb.out(f)).unwrap(), bytes, "{f} was kept");
    }
    let report = sb.read("reports/2026-11-22.md");
    let action = report.split("## Run").next().unwrap();
    assert!(action.contains("Stage `bulk-sync` failed: bulk-sync failed"), "{action}");
    assert!(report.contains("the catalog sync failed this run"), "{report}");
    assert!(listing(&sb.p("out")).iter().all(|f| !f.ends_with(".tmp")), "no temp files left");
}

#[test]
fn without_a_cardgen_binary_the_short_report_is_still_written() {
    let sb = Sandbox::new();
    let mut env = sb.env();
    env.insert("KNOWLEDGE_CARDGEN".into(), sb.p("nope/cardgen").display().to_string());
    let text = sb.ok(&[], &env);
    let stages = sb.stages("2026-11-22");
    assert_eq!(stages[0].1, "fail", "{stages:?}\n{text}");
    assert!(stages[1..].iter().all(|s| s.1 == "skip"), "{stages:?}");
    let report = sb.read("reports/2026-11-22.md");
    assert!(report.contains("short form") && report.contains("Stage `fetch-build` failed"), "{report}");
    assert!(report.find("## Action needed").unwrap() < report.find("## Run").unwrap());
    assert!(!sb.out("oracle.json").exists(), "nothing else was written");
}

#[test]
fn a_fetch_failure_is_recorded_and_the_previous_build_is_used() {
    let sb = Sandbox::new();
    let mut env = sb.env();
    // a git that cannot fetch (no network): every other git call goes to the real one
    let real = String::from_utf8(Command::new("which").arg("git").output().unwrap().stdout).unwrap();
    sb.stub("git", &format!("for a in \"$@\"; do [ \"$a\" = fetch ] && exit 1; done; exec {} \"$@\"", real.trim()));
    sb.with_stubs_first(&mut env);
    env.remove("KNOWLEDGE_SKIP_FETCH");
    // a clone that exists already (a .git directory), so no clone is attempted
    let repo = sb.p("clone");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(repo.join("data")).unwrap();
    std::fs::copy(repo_root().join("data/decks.json"), repo.join("data/decks.json")).unwrap();
    std::fs::copy(repo_root().join("data/scryfall_cache.json"), repo.join("data/scryfall_cache.json")).unwrap();
    env.insert("KNOWLEDGE_REPO".into(), repo.display().to_string());
    sb.ok(&[], &env);
    let stages = sb.stages("2026-11-22");
    assert_eq!(stages[0].1, "fail");
    assert!(stages[0].2.contains("git fetch failed"), "{stages:?}");
    // the build in use still ran the other stages
    assert_eq!(stages[1].1, "ok", "{stages:?}");
    let report = sb.read("reports/2026-11-22.md");
    assert!(report.contains("Stage `fetch-build` failed: git fetch failed"), "{report}");
}

#[test]
fn a_second_run_while_one_is_active_exits_zero_and_writes_nothing() {
    let sb = Sandbox::new();
    let env = sb.env();
    std::fs::create_dir_all(sb.p("cache")).unwrap();
    // hold the lock the way a running job does
    let mut holder = Command::new("flock").arg(sb.p("cache/lock")).args(["sleep", "30"]).spawn().unwrap();
    let t = std::time::Instant::now();
    while !sb.p("cache/lock").exists() && t.elapsed().as_secs() < 5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let text = sb.ok(&[], &env);
    let _ = holder.kill();
    let _ = holder.wait();
    assert!(text.contains("another run is active"), "{text}");
    assert_eq!(listing(&sb.p("out")), ["roles.toml"], "nothing was written");
    assert!(!sb.p("cache/runs").exists());
}

#[test]
fn an_interrupted_run_leaves_the_previous_outputs_intact_and_the_next_run_completes() {
    let sb = Sandbox::new();
    let env = sb.env();
    sb.ok(&[], &env);
    let (roles, map) = (sb.read("roles.json"), sb.read("map.md"));
    let report = sb.read("reports/2026-11-22.md");
    // a cardgen that hangs in the roles stage (after the sync of a newer bulk file), then is killed with its group
    let real = env!("CARGO_BIN_EXE_cardgen");
    sb.stub(
        "slow-cardgen",
        &format!(
            "for a in \"$@\"; do [ \"$a\" = roles ] && {{ touch {}/hung; exec sleep 60; }}; done; exec {real} \"$@\"",
            sb.root.display()
        ),
    );
    let mut slow = env.clone();
    slow.insert("KNOWLEDGE_CARDGEN".into(), sb.p("bin/slow-cardgen").display().to_string());
    slow.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    let mut child = Command::new("setsid")
        .arg("bash")
        .arg(script())
        .envs(&slow)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let t = std::time::Instant::now();
    while !sb.p("hung").exists() && t.elapsed().as_secs() < 60 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(sb.p("hung").exists(), "the roles stage started");
    // systemd stops a unit by signalling its whole control group; the group here is the session setsid made
    assert!(Command::new("kill").args(["-s", "TERM", "--", &format!("-{}", child.id())]).status().unwrap().success());
    let _ = child.wait();
    assert_eq!(sb.read("roles.json"), roles, "the role table is the previous one");
    assert_eq!(sb.read("map.md"), map);
    assert_eq!(sb.read("reports/2026-11-22.md"), report, "no half-written report");
    assert!(sb.read("oracle.json").contains(BULK_AT2), "the sync finished before the interruption");
    assert!(sb.read("oracle.prev.json").contains(BULK_AT));
    assert!(listing(&sb.p("out")).iter().all(|f| !f.ends_with(".tmp")), "{:?}", listing(&sb.p("out")));
    // the lock died with the process: the next run is not blocked and completes
    let mut again = env.clone();
    again.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    let text = sb.ok(&[], &again);
    assert!(!text.contains("another run is active"), "{text}");
    assert!(sb.read("reports/2026-11-22.md").contains("## Action needed"));
    assert_eq!(sb.stages("2026-11-22")[2], ("roles".into(), "ok".into(), "rules only (no tracked LLM rows)".into()));
}

// ------------------------------------------------------------------------------------------------ the LLM decisions

/// the script's own `llm_decide` with stubs of curl, ss, fuser and systemctl first in PATH
struct Probe {
    sb: Sandbox,
}

impl Probe {
    fn new() -> Probe {
        let sb = Sandbox::new();
        std::fs::write(sb.p("gpu"), "").unwrap();
        Probe { sb }
    }

    /// curl answers with `models` (or fails when empty); ss prints a listener when `held`; fuser says busy when `busy`;
    /// `unit` is whether systemctl knows the unit
    fn decide(&self, models: &str, held: bool, busy: bool, unit: bool, device: bool) -> String {
        let sb = &self.sb;
        let curl = if models.is_empty() { "exit 7".to_string() } else { format!("echo '{models}'") };
        sb.stub("curl", &curl);
        sb.stub("ss", if held { "echo 'LISTEN 0 512 127.0.0.1:8080 0.0.0.0:*'" } else { "exit 0" });
        sb.stub("fuser", if busy { "exit 0" } else { "exit 1" });
        sb.stub(
            "systemctl",
            &format!(
                "[ \"$2\" = cat ] && exit {}; echo \"systemctl $*\" >> {}/calls; exit 0",
                if unit { 0 } else { 1 },
                sb.root.display()
            ),
        );
        let mut env = sb.env();
        sb.with_stubs_first(&mut env);
        let dev = if device { sb.p("gpu") } else { sb.p("no-such-device") };
        env.insert("KNOWLEDGE_GPU_DEVICE".into(), dev.display().to_string());
        let o = Command::new("bash")
            .arg("-c")
            .arg(format!("source {}; llm_decide", script().display()))
            .envs(&env)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let calls = std::fs::read_to_string(sb.p("calls")).unwrap_or_default();
        assert!(
            !calls.contains("start") && !calls.contains("stop"),
            "a decision never starts or stops anything: {calls}"
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }
}

const MODEL_27B: &str = r#"{"object":"list","data":[{"id":"/home/x/models/Qwen3.6-27B-Q5_K_M.gguf"}]}"#;
const MODEL_CODER: &str = r#"{"object":"list","data":[{"id":"Qwen3-Coder-30B-A3B-Instruct-UD-Q4_K_XL.gguf"}]}"#;

#[test]
fn a_running_27b_server_is_used_another_model_is_not() {
    let p = Probe::new();
    assert_eq!(
        p.decide(MODEL_27B, true, true, true, true),
        "use",
        "in use even though the GPU is busy: it is ours to share"
    );
    assert_eq!(p.decide(MODEL_CODER, true, false, true, true), "skip: other model on the port");
    assert_eq!(p.decide(MODEL_27B, false, false, false, false), "use", "answering is enough");
}

#[test]
fn a_held_port_a_busy_gpu_a_missing_device_and_a_missing_unit_each_skip_with_their_reason() {
    let p = Probe::new();
    assert!(
        p.decide("", true, false, true, true).starts_with("skip: port 8080 is held but /v1/models does not answer")
    );
    assert_eq!(p.decide("", false, true, true, true), "skip: GPU busy");
    assert!(
        p.decide("", false, false, true, false).starts_with("skip: GPU device ")
            && p.decide("", false, false, true, false).ends_with("not found")
    );
    assert!(p.decide("", false, false, false, true).contains("llama-27b.service is not installed"));
    assert_eq!(p.decide("", false, false, true, true), "start");
}

/// the whole LLM stage against stubs: a fake cardgen (dry run counts, the run appends pending rows), a curl that
/// answers once the fake unit is started, a systemctl that logs and starts and stops the fake unit
struct LlmRun {
    sb: Sandbox,
    env: BTreeMap<String, String>,
}

impl LlmRun {
    fn new(todo: u32) -> LlmRun {
        let sb = Sandbox::new();
        let root = sb.root.display().to_string();
        // a real catalog, so the real cardgen can judge the arguments the script passes
        std::fs::copy(manifest().join("tests/fixtures/roles_oracle.json"), sb.out("oracle.json")).unwrap();
        std::fs::create_dir_all(sb.p("repo/data")).unwrap();
        std::fs::write(sb.p("gpu"), "").unwrap();
        // The fake accepts a `roles` call only if the REAL cardgen accepts the same arguments (as a dry run, which
        // makes no request): the grammar is the real one, the behaviour (counts, rows, sleeping, failing) is scripted.
        let real = env!("CARGO_BIN_EXE_cardgen");
        let grammar = [
            "case \" $* \" in *' roles '*)".to_string(),
            "  chk=(\"$@\")".to_string(),
            "  case \" $* \" in *' --dry-run '*) ;; *) chk+=(--dry-run);; esac".to_string(),
            "  case \" $* \" in *' --model '*) ;; *) chk+=(--model Qwen3.6-27B-Q5_K_M.gguf);; esac".to_string(),
            format!(
                "  {real} \"${{chk[@]}}\" >/dev/null 2>\"$TEST_ROOT/grammar.err\" \\
                 || {{ cat \"$TEST_ROOT/grammar.err\" >&2; exit 2; }};;"
            ),
            "esac".to_string(),
        ]
        .join("\n");
        let dry = format!(
            "case \" $* \" in *' --dry-run '*) echo 'DRY RUN: model m; 9 candidate cards selected, 4 cached, \
             {todo} to classify in 1 batches'; exit 0;; esac"
        );
        sb.stub(
            "fake-cardgen",
            &[
                grammar,
                dry,
                format!("echo \"cardgen $*\" >> {root}/cardgen.calls"),
                "if [ -n \"$FAKE_SLEEP\" ]; then exec sleep \"$FAKE_SLEEP\"; fi".into(),
                "[ -n \"$FAKE_FAIL\" ] && { echo 'boom: roles --llm failed' >&2; exit 1; }".into(),
                "printf '{}\\n{}\\n{}\\n' >> \"$CARDGEN_KNOWLEDGE_DIR/llm_roles.pending.jsonl\"".into(),
                "echo 'classified 3 cards'".into(),
            ]
            .join("\n"),
        );
        let chat = r#"echo '{"choices":[{"message":{"content":"OK"}}]}'"#;
        sb.stub(
            "curl",
            &[
                format!(
                    "case \"$*\" in *chat/completions*) [ -n \"$FAKE_SMOKE_FAIL\" ] && exit 22; {chat}; exit 0;; esac"
                ),
                format!("if [ -e {root}/server-up ]; then echo '{MODEL_27B}'; exit 0; fi"),
                "exit 7".into(),
            ]
            .join("\n"),
        );
        sb.stub("ss", "exit 0");
        sb.stub("fuser", "[ -n \"$FAKE_GPU_BUSY\" ] && exit 0; exit 1");
        sb.stub(
            "systemctl",
            &[
                format!("echo \"systemctl $*\" >> {root}/calls"),
                "case \"$2\" in".into(),
                format!(" start) touch {root}/server-up;;"),
                format!(" stop) rm -f {root}/server-up;;"),
                format!(" is-active) [ -e {root}/server-up ] || exit 3;;"),
                "esac".into(),
            ]
            .join("\n"),
        );
        let mut env = sb.env();
        env.remove("KNOWLEDGE_SKIP_LLM");
        env.insert("KNOWLEDGE_CARDGEN".into(), sb.p("bin/fake-cardgen").display().to_string());
        env.insert("TEST_ROOT".into(), root.clone());
        env.insert("KNOWLEDGE_REPO".into(), sb.p("repo").display().to_string());
        env.insert("KNOWLEDGE_GPU_DEVICE".into(), sb.p("gpu").display().to_string());
        env.insert("KNOWLEDGE_LLM_POLL_SECS".into(), "1".into());
        env.insert("KNOWLEDGE_LLM_READY_SECS".into(), "10".into());
        sb.with_stubs_first(&mut env);
        LlmRun { sb, env }
    }

    fn calls(&self) -> String {
        std::fs::read_to_string(self.sb.p("calls")).unwrap_or_default()
    }

    /// the llm stage alone; returns (status, note)
    fn stage(&self) -> (String, String) {
        self.sb.ok(&["--stage", "4"], &self.env);
        let s = self.sb.stages("2026-11-22");
        let llm = s.iter().find(|s| s.0 == "llm").expect("llm stage recorded");
        (llm.1.clone(), llm.2.clone())
    }
}

#[test]
fn the_stage_starts_the_unit_when_the_gpu_is_free_and_stops_it_afterwards_and_writes_only_pending_rows() {
    let r = LlmRun::new(5);
    let (status, note) = r.stage();
    assert_eq!(status, "ok", "{note}");
    assert!(note.contains("3 new pending rows (3 pending in all), 5 were to do"), "{note}");
    let calls = r.calls();
    let (start, stop) =
        (calls.find("systemctl --user start llama-27b.service"), calls.find("systemctl --user stop llama-27b.service"));
    assert!(start.is_some() && stop.is_some() && start < stop, "{calls}");
    // the ownership is looked up after the start and again before the stop; nothing sets a runtime property any more
    assert!(calls.contains("show -p InvocationID") && !calls.contains("set-property"), "{calls}");
    assert!(!r.sb.p("server-up").exists(), "the unit was stopped again");
    assert!(!r.sb.p("cache/owns-llama-27b").exists(), "the ownership marker is gone");
    assert!(r.sb.out("llm_roles.pending.jsonl").is_file());
    assert!(!r.sb.out("llm_roles.jsonl").exists(), "the tracked cache is never written");
    let args = std::fs::read_to_string(r.sb.p("cardgen.calls")).unwrap();
    let want = "roles --llm --routes NoRole,UnknownLine --only-routed --batch 10 --no-think --pending";
    assert!(args.contains(want) && args.contains("--endpoint http://127.0.0.1:8080/v1"), "{args}");
}

#[test]
fn a_server_that_is_already_running_is_used_and_never_started_or_stopped() {
    let r = LlmRun::new(5);
    std::fs::write(r.sb.p("server-up"), "").unwrap();
    let (status, note) = r.stage();
    assert_eq!(status, "ok", "{note}");
    let calls = r.calls();
    assert!(!calls.contains(" start ") && !calls.contains(" stop "), "{calls}");
    assert!(r.sb.p("server-up").exists(), "still running afterwards");
}

#[test]
fn a_busy_gpu_or_another_model_skips_the_stage_without_starting_anything() {
    let mut r = LlmRun::new(5);
    r.env.insert("FAKE_GPU_BUSY".into(), "1".into());
    let (status, note) = r.stage();
    assert_eq!((status.as_str(), note.as_str()), ("skip", "GPU busy"));
    assert!(!r.calls().contains(" start "), "{}", r.calls());
    assert!(!r.sb.out("llm_roles.pending.jsonl").exists());
    // another model on the port: left alone
    let r = LlmRun::new(5);
    r.sb.stub("curl", &format!("echo '{MODEL_CODER}'"));
    let (status, note) = r.stage();
    assert_eq!((status.as_str(), note.as_str()), ("skip", "other model on the port"));
    assert!(!r.calls().contains(" start ") && !r.calls().contains(" stop "), "{}", r.calls());
}

#[test]
fn a_quiet_week_never_wakes_the_gpu() {
    let r = LlmRun::new(0);
    let (status, note) = r.stage();
    assert_eq!(status, "skip");
    assert!(note.contains("nothing to classify"), "{note}");
    assert!(r.calls().is_empty(), "no systemctl call at all: {}", r.calls());
}

#[test]
fn a_failed_smoke_prompt_a_failed_run_and_a_spent_budget_each_stop_the_unit_that_was_started() {
    let mut r = LlmRun::new(5);
    r.env.insert("FAKE_SMOKE_FAIL".into(), "1".into());
    let (status, note) = r.stage();
    assert_eq!((status.as_str(), note.as_str()), ("fail", "the smoke prompt failed"));
    assert!(r.calls().contains(" stop ") && !r.sb.p("server-up").exists(), "{}", r.calls());
    assert!(!r.sb.out("llm_roles.pending.jsonl").exists());

    let mut r = LlmRun::new(5);
    r.env.insert("FAKE_FAIL".into(), "1".into());
    let (status, note) = r.stage();
    assert_eq!(status, "fail");
    assert!(note.contains("roles --llm exited 1") && note.contains("boom"), "{note}");
    assert!(r.calls().contains(" stop ") && !r.sb.p("server-up").exists());

    let mut r = LlmRun::new(5);
    r.env.insert("FAKE_SLEEP".into(), "20".into());
    r.env.insert("KNOWLEDGE_LLM_BUDGET_SECS".into(), "1".into());
    let (status, note) = r.stage();
    assert_eq!(status, "ok", "{note}");
    assert!(note.contains("budget of 1s used up") && note.contains("carries over"), "{note}");
    assert!(r.calls().contains(" stop ") && !r.sb.p("server-up").exists());

    // a server that never comes up
    let mut r = LlmRun::new(5);
    r.env.insert("KNOWLEDGE_LLM_READY_SECS".into(), "2".into());
    r.sb.stub("curl", "exit 7");
    r.sb.stub(
        "systemctl",
        &format!(
            "echo \"systemctl $*\" >> {}/calls; case \"$2\" in is-active) exit 0;; esac; exit 0",
            r.sb.root.display()
        ),
    );
    let (status, note) = r.stage();
    assert_eq!(status, "fail");
    assert!(note.contains("did not become ready") && note.contains("stopped it"), "{note}");
    assert!(r.calls().contains(" stop "), "{}", r.calls());
}

#[test]
fn a_job_killed_while_it_owns_the_server_stops_it() {
    let mut r = LlmRun::new(5);
    r.env.insert("FAKE_SLEEP".into(), "60".into());
    let mut child = Command::new("setsid")
        .arg("bash")
        .arg(script())
        .args(["--stage", "4"])
        .envs(&r.env)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let t = std::time::Instant::now();
    while !r.sb.p("cardgen.calls").exists() && t.elapsed().as_secs() < 60 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(r.sb.p("server-up").exists(), "the job started the unit");
    assert!(Command::new("kill").args(["-s", "TERM", "--", &format!("-{}", child.id())]).status().unwrap().success());
    let _ = child.wait();
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(!r.sb.p("server-up").exists(), "stopped on the way out: {}", r.calls());
    assert!(!r.sb.p("cache/owns-llama-27b").exists(), "the ownership marker is gone");
}

// ------------------------------------------------------------------------------------------------ round 2

/// the catalog of `roles_oracle.json` in the sandbox's output directory, with the real role files
fn real_catalog(sb: &Sandbox) {
    std::fs::copy(manifest().join("tests/fixtures/roles_oracle.json"), sb.out("oracle.json")).unwrap();
}

/// a cache row for one card: the current think/no-think prompt and the dense model the weekly job names
fn row(index: &Index, taxonomy: &Taxonomy, name: &str, role: &str) -> String {
    let card = &index.cards[index.lookup(name)[0] as usize];
    let prompt = llm::prompt_fnv_opts(taxonomy, Variant::Think, true);
    let model = "Qwen3.6-27B-Q5_K_M.gguf";
    let row = Row {
        key: llm::cache_key(&llm::card_text(card), &prompt, model),
        name: name.to_string(),
        roles: vec![role.to_string()],
        primary: Some(role.to_string()),
        evidence: BTreeMap::from([(role.to_string(), 0)]),
        model: model.to_string(),
        taxonomy: taxonomy.hash(),
        prompt,
        why: None,
    };
    format!("{}\n", serde_json::to_string(&row).unwrap())
}

#[test]
fn the_roles_stage_merges_a_populated_tracked_cache_and_never_the_pending_rows() {
    let sb = Sandbox::new();
    real_catalog(&sb);
    let mut env = sb.env();
    env.insert("CARDGEN_LLM_VARIANT".into(), "think".into());
    let index = Index::load(&sb.out("oracle.json")).unwrap();
    let taxonomy = Taxonomy::parse(&std::fs::read_to_string(sb.out("roles.toml")).unwrap()).unwrap();
    // two cards the rules route to the LLM: one answered in the tracked cache, the other only in the pending file
    let rules = roles::generate(&index, "").unwrap();
    let routed: Vec<&str> = index
        .cards
        .iter()
        .filter(|c| c.legal == cardgen::knowledge::Legal::Legal && rules[&c.raw.oracle_id].needs_llm)
        .map(|c| c.raw.name.as_str())
        .take(2)
        .collect();
    assert_eq!(routed.len(), 2);
    std::fs::write(sb.out("llm_roles.jsonl"), row(&index, &taxonomy, routed[0], "removal")).unwrap();
    std::fs::write(sb.out("llm_roles.pending.jsonl"), row(&index, &taxonomy, routed[1], "removal")).unwrap();
    let text = sb.ok(&["--stage", "3"], &env);
    let stages = sb.stages("2026-11-22");
    assert_eq!(stages[0], ("roles".into(), "ok".into(), "rules and the tracked LLM cache merged".into()), "{text}");
    let table = roles::load(&sb.out("roles.json")).unwrap();
    assert!(!table.meta.prompt_fnv.is_empty() && table.meta.model == "Qwen3.6-27B-Q5_K_M.gguf", "{:?}", table.meta);
    let merge: serde_json::Value = serde_json::from_str(&sb.read("reports/merge.json")).unwrap();
    assert_eq!(merge["with_answer"], 1, "only the tracked row is an answer; the pending one stays out: {merge}");
    assert!(std::fs::read_to_string(sb.out("llm_roles.pending.jsonl")).unwrap().contains(routed[1]), "pending kept");
}

#[test]
fn the_default_count_floor_refuses_a_small_bulk_file_and_the_catalog_stays() {
    let sb = Sandbox::new();
    let mut env = sb.env();
    env.remove("KNOWLEDGE_BULK_MIN");
    sb.ok(&["--stage", "2"], &sb.env());
    let before = (sb.read("oracle.json"), sb.read("sync.lock"));
    env.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    sb.ok(&["--stage", "2"], &env);
    let stages = sb.stages("2026-11-22");
    assert_eq!((stages[0].0.as_str(), stages[0].1.as_str()), ("bulk-sync", "fail"), "{stages:?}");
    assert!(stages[0].2.contains("minimum 25000"), "{stages:?}");
    assert_eq!((sb.read("oracle.json"), sb.read("sync.lock")), before);
    assert!(!sb.out("oracle.prev.json").exists());
}

#[test]
fn an_interruption_after_the_roles_writer_has_written_leaves_the_published_directory_untouched() {
    let sb = Sandbox::new();
    let env = sb.env();
    sb.ok(&[], &env);
    let before: Vec<(String, Vec<u8>)> = listing(&sb.p("out"))
        .into_iter()
        .map(|f| {
            let bytes = std::fs::read(sb.out(&f)).unwrap();
            (f, bytes)
        })
        .collect();
    // the real cardgen writes roles.json (into the staging directory), then the process hangs before the stage ends
    let real = env!("CARGO_BIN_EXE_cardgen");
    sb.stub(
        "after-writer",
        &format!(
            "{real} \"$@\" || exit $?; case \" $* \" in \\
             *' --rules '*) touch \"$TEST_ROOT/written\"; exec sleep 60;; esac"
        ),
    );
    let mut slow = env.clone();
    slow.insert("KNOWLEDGE_CARDGEN".into(), sb.p("bin/after-writer").display().to_string());
    slow.insert("TEST_ROOT".into(), sb.root.display().to_string());
    let mut child = Command::new("setsid")
        .arg("bash")
        .arg(script())
        .envs(&slow)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let t = std::time::Instant::now();
    while !sb.p("written").exists() && t.elapsed().as_secs() < 60 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(sb.p("written").exists(), "the roles writer ran");
    let staged: Vec<String> = listing(&sb.p("cache/staging"));
    assert!(staged.iter().any(|f| f.ends_with("roles/roles.json")), "roles.json was written, in staging: {staged:?}");
    assert!(Command::new("kill").args(["-s", "KILL", "--", &format!("-{}", child.id())]).status().unwrap().success());
    let _ = child.wait();
    let after: Vec<(String, Vec<u8>)> = listing(&sb.p("out"))
        .into_iter()
        .map(|f| {
            let bytes = std::fs::read(sb.out(&f)).unwrap();
            (f, bytes)
        })
        .collect();
    assert_eq!(before, after, "byte-identical, and no *.tmp in it");
    // the next run removes the abandoned staging directory
    sb.ok(&[], &env);
    assert!(listing(&sb.p("cache/staging")).is_empty(), "{:?}", listing(&sb.p("cache/staging")));
}

#[test]
fn the_report_is_published_whole_the_earlier_one_of_the_day_is_kept_and_twelve_stay() {
    let sb = Sandbox::new();
    let env = sb.env();
    sb.ok(&[], &env);
    let first = sb.read("reports/2026-11-22.md");
    let mut again = env.clone();
    again.insert("KNOWLEDGE_BULK_UPDATED_AT".into(), BULK_AT2.into());
    sb.ok(&[], &again);
    assert_eq!(sb.read("reports/2026-11-22.1.md"), first, "the earlier report of the day is DATE.1.md");
    assert_ne!(sb.read("reports/2026-11-22.md"), first);
    // ten more runs of the day: archives are numbered upward, numerically, and the newest twelve stay
    for _ in 0..12 {
        sb.ok(&["--stage", "6"], &env);
    }
    let mut names: Vec<String> = listing(&sb.p("out/reports")).into_iter().filter(|f| f.ends_with(".md")).collect();
    names.sort();
    assert_eq!(names.len(), 12, "{names:?}");
    assert!(
        names.contains(&"2026-11-22.md".to_string()) && names.contains(&"2026-11-22.13.md".to_string()),
        "{names:?}"
    );
    assert!(
        !names.contains(&"2026-11-22.1.md".to_string()) && !names.contains(&"2026-11-22.2.md".to_string()),
        "{names:?}"
    );
    // an old report of another day goes first
    assert!(listing(&sb.p("out/reports")).iter().all(|f| !f.starts_with(".")));
}

#[test]
fn cleanup_stops_only_the_invocation_the_jobs_own_marker_names_and_a_second_job_never_does() {
    let r = LlmRun::new(5);
    let sb = &r.sb;
    let root = sb.root.display().to_string();
    // `show` reports the unit's current invocation from a file the test controls
    sb.stub(
        "systemctl",
        &[
            format!("echo \"systemctl $*\" >> {root}/calls"),
            "case \"$2\" in".to_string(),
            format!(" show) cat {root}/invocation 2>/dev/null;;"),
            format!(" stop) rm -f {root}/server-up;;"),
            "esac".to_string(),
        ]
        .join("\n"),
    );
    std::fs::create_dir_all(sb.p("cache")).unwrap();
    let marker = sb.p("cache/owns-llama-27b");
    let stops = || r.calls().matches(" stop ").count();
    // no marker: nothing is stopped, whoever started the unit
    std::fs::write(sb.p("server-up"), "").unwrap();
    sb.ok(&["--cleanup"], &r.env);
    assert_eq!(stops(), 0, "{}", r.calls());
    // the marker names the running invocation: stopped, marker removed
    std::fs::write(sb.p("invocation"), "abc\n").unwrap();
    std::fs::write(&marker, "run=2026-11-20-1\ninvocation=abc\n").unwrap();
    sb.ok(&["--cleanup"], &r.env);
    assert_eq!(stops(), 1, "{}", r.calls());
    assert!(!marker.exists() && !sb.p("server-up").exists());
    // the marker names an older invocation: the newer one is left running
    std::fs::write(sb.p("server-up"), "").unwrap();
    std::fs::write(sb.p("invocation"), "def\n").unwrap();
    std::fs::write(&marker, "run=2026-11-20-1\ninvocation=abc\n").unwrap();
    sb.ok(&["--cleanup"], &r.env);
    assert_eq!(stops(), 1, "{}", r.calls());
    assert!(!marker.exists() && sb.p("server-up").exists());
    // a second job that finds the lock taken exits without touching the first job's marker or server
    std::fs::write(&marker, "run=2026-11-22-4242\ninvocation=def\n").unwrap();
    let mut holder = Command::new("flock").arg(sb.p("cache/lock")).args(["sleep", "30"]).spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let text = sb.ok(&[], &r.env);
    let _ = holder.kill();
    let _ = holder.wait();
    assert!(text.contains("another run is active"), "{text}");
    assert_eq!(stops(), 1, "{}", r.calls());
    assert!(marker.exists() && sb.p("server-up").exists());
}
