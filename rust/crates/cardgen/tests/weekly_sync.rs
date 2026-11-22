//! What the weekly job needs from `bulk-sync` without the network: `sync.lock` is untracked (it may be missing, and a
//! missing one must not make the next run replace `oracle.prev.json` with a copy of the current catalog), and a local
//! file can stand in for Scryfall's bulk entry (`--from-file PATH --updated-at STAMP`) so the job's smoke test runs
//! offline.

use cardgen::bulk::{self, LocalRemote, Options};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn bulk_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/bulk_fixture.json")
}

struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Tmp {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "weekly-sync-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
    fn join(&self, f: &str) -> PathBuf {
        self.0.join(f)
    }
    fn read(&self, f: &str) -> String {
        std::fs::read_to_string(self.join(f)).unwrap()
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sync(dir: &Path, updated_at: &str) -> bulk::Outcome {
    let opts = Options { dir: dir.to_path_buf(), from_file: None, min_count: 1, synced: "2026-11-22T00:00:00Z".into() };
    bulk::run(&opts, &mut LocalRemote { updated_at: updated_at.into(), path: bulk_fixture() }).unwrap()
}

fn files(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    v.sort();
    v
}

#[test]
fn a_local_file_can_stand_in_for_the_remote_and_the_second_run_is_up_to_date() {
    let t = Tmp::new();
    let first = sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    assert!(!first.up_to_date && first.text.contains("wrote"), "{}", first.text);
    assert_eq!(files(&t.0), ["oracle.json", "sync.lock"], "no previous sync yet, no stray temp files");
    let again = sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    assert!(again.up_to_date && again.text.contains("up to date"), "{}", again.text);
    assert_eq!(files(&t.0), ["oracle.json", "sync.lock"]);
    // a newer bulk date syncs and keeps the old catalog as oracle.prev.json
    let newer = sync(&t.0, "2026-11-21T10:00:00.000+00:00");
    assert!(!newer.up_to_date);
    assert_eq!(files(&t.0), ["oracle.json", "oracle.prev.json", "sync.lock"]);
    assert!(t.read("oracle.prev.json").contains("2026-11-14T10:00:00.000+00:00"));
    assert!(t.read("oracle.json").contains("2026-11-21T10:00:00.000+00:00"));
    assert!(t.read("sync.lock").contains("bulk_updated_at=2026-11-21T10:00:00.000+00:00"));
}

#[test]
fn a_missing_sync_lock_is_rebuilt_and_does_not_cost_the_previous_sync() {
    let t = Tmp::new();
    sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    sync(&t.0, "2026-11-21T10:00:00.000+00:00");
    let (oracle, prev, lock) = (t.read("oracle.json"), t.read("oracle.prev.json"), t.read("sync.lock"));
    // the merge that untracked sync.lock deletes it from a checkout, or a fresh clone never had it
    std::fs::remove_file(t.join("sync.lock")).unwrap();
    let out = sync(&t.0, "2026-11-21T10:00:00.000+00:00");
    assert!(out.up_to_date, "{}", out.text);
    assert_eq!(t.read("oracle.json"), oracle);
    assert_eq!(t.read("oracle.prev.json"), prev, "the previous sync is not replaced by a copy of the current one");
    assert_eq!(t.read("sync.lock"), lock, "the lock is written again from the catalog");
    assert_eq!(files(&t.0), ["oracle.json", "oracle.prev.json", "sync.lock"]);
}

#[test]
fn without_a_lock_a_damaged_catalog_is_synced_again_not_trusted() {
    let t = Tmp::new();
    sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    let whole = t.read("oracle.json");
    std::fs::remove_file(t.join("sync.lock")).unwrap();
    // truncated in the middle of the cards: the first line (the meta) still says the bulk date
    let cut = whole.len() / 2;
    std::fs::write(t.join("oracle.json"), &whole[..cut]).unwrap();
    let out = sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    assert!(!out.up_to_date, "{}", out.text);
    assert_eq!(t.read("oracle.json"), whole);
    // a catalog whose meta.count lies is not whole either
    std::fs::remove_file(t.join("sync.lock")).unwrap();
    let lying = whole.replacen("\"count\": 41", "\"count\": 99", 1);
    if lying != whole {
        std::fs::write(t.join("oracle.json"), &lying).unwrap();
        assert!(!sync(&t.0, "2026-11-14T10:00:00.000+00:00").up_to_date);
    }
}

#[test]
fn a_lock_that_is_present_but_wrong_still_resyncs() {
    let t = Tmp::new();
    sync(&t.0, "2026-11-14T10:00:00.000+00:00");
    std::fs::write(
        t.join("sync.lock"),
        "schema=1\nbulk_updated_at=2026-11-14T10:00:00.000+00:00\ncount=1\nfnv1a64=0000000000000000\n",
    )
    .unwrap();
    assert!(!sync(&t.0, "2026-11-14T10:00:00.000+00:00").up_to_date);
    std::fs::write(t.join("sync.lock"), "garbage").unwrap();
    assert!(!sync(&t.0, "2026-11-14T10:00:00.000+00:00").up_to_date);
}

fn cardgen() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cardgen"))
}

#[test]
fn the_command_line_takes_updated_at_only_with_from_file() {
    let t = Tmp::new();
    let dir = t.join("k");
    let run = |args: &[&str]| {
        cardgen()
            .args([
                "--root",
                t.0.to_str().unwrap(),
                "bulk-sync",
                "--min-count",
                "1",
                "--data-dir",
                dir.to_str().unwrap(),
            ])
            .args(args)
            .env_remove("CARDGEN_KNOWLEDGE_DIR")
            .output()
            .unwrap()
    };
    let f = bulk_fixture();
    let f = f.to_str().unwrap();
    let out = run(&["--updated-at", "2026-11-14"]);
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("--updated-at needs --from-file"));
    let out = run(&["--from-file", f, "--updated-at", "2026-11-14"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("bulk updated_at: 2026-11-14"));
    let out = run(&["--from-file", f, "--updated-at", "2026-11-14"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("up to date"), "{}", String::from_utf8_lossy(&out.stdout));
    // plain --from-file is unchanged: no up-to-date check, the date is "from-file"
    let out = run(&["--from-file", f]);
    assert!(out.status.success() && String::from_utf8_lossy(&out.stdout).contains("bulk updated_at: from-file"));
}
