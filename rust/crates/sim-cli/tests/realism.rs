//! `commander-sim --realism --pool tN` end to end, on a handful of games (the targets-file checks are unit tests in
//! src/realism.rs).

use std::process::Command;

fn commander_sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn realism_scores_a_small_t6_run() {
    let (code, out, err) =
        commander_sim(&["--realism", "--pool", "t6", "--games", "8", "--ai", "adaptive", "--quiet", "--jobs", "2"]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("=== Realism: Tier 6"), "{out}");
    assert!(out.contains("[loose, adaptive AI, 8 games"), "{out}");
    for line in ["seat 1 share of decisive games", "median round of a decisive win", "games over 15 rounds"] {
        assert!(out.contains(line), "missing {line:?}:\n{out}");
    }
    assert!(out.contains("PASS") || out.contains("FAIL"), "t6 has targets, so something is scored:\n{out}");
    assert!(out.contains("no target"), "games_over_15 has no t6 target:\n{out}");
    assert!(out.contains("targets pass"), "{out}");
}

#[test]
fn realism_scores_nothing_for_a_tier_without_targets() {
    let (code, out, err) =
        commander_sim(&["--realism", "--pool", "t0", "--games", "4", "--ai", "adaptive", "--quiet", "--jobs", "2"]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("0 of 0 targets pass"), "{out}");
}

#[test]
fn realism_needs_one_tier_and_no_deck() {
    let (code, _, err) = commander_sim(&["--realism", "--pool", "all", "--games", "4", "--quiet"]);
    assert_eq!(code, 1);
    assert!(err.contains("--realism needs --pool with one tier"), "{err}");
    let (code, _, err) = commander_sim(&["--realism", "--quiet"]);
    assert_eq!(code, 1);
    assert!(err.contains("--realism needs --pool with one tier"), "{err}");
    let (code, _, err) = commander_sim(&["--realism", "--pool", "t5", "--deck", "karn", "--quiet"]);
    assert_eq!(code, 1);
    assert!(err.contains("drop --deck"), "{err}");
}
