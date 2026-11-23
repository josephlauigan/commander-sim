//! A deck whose format the simulator cannot seat (kyodai plays Cali Commander) is refused by every mode that would
//! seat it, before any game is played: nothing silently plays it as a 4-player Commander game. The tier and key
//! logic is unit-tested in src/poolmode.rs and src/goldfish.rs; this runs the real binary.

use std::process::Command;

fn commander_sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// exit 1, the reason on stderr, and no game on stdout
fn refused(args: &[&str]) {
    let (code, out, err) = commander_sim(args);
    assert_eq!(code, 1, "{args:?}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("Cali Commander"), "{args:?}: {err}");
    assert!(err.contains("kyodai"), "{args:?}: {err}");
    for game in ["Play-by-play", "Winner:", "R1 ", "Round 1"] {
        assert!(!out.contains(game), "{args:?} played a game ({game:?}):\n{out}");
    }
}

#[test]
fn trace_refuses_kyodai() {
    refused(&["--deck", "kyodai", "--pool", "t5", "--trace", "0", "--quiet"]);
    refused(&["--deck", "kyodai", "--pool", "t6", "--trace", "0", "--quiet"]);
    refused(&["--deck", "kyodai", "--pool", "t1", "--any-tier", "--trace", "0", "--quiet"]);
}

#[test]
fn a_pool_run_refuses_kyodai() {
    refused(&["--deck", "kyodai", "--pool", "t5", "--games", "2", "--quiet"]);
    refused(&["--deck", "kyodai", "--pool", "all", "--games", "2", "--quiet", "--any-tier"]);
}

#[test]
fn searchtest_refuses_kyodai() {
    refused(&["searchtest", "kyodai", "t5", "1"]);
}

#[test]
fn goldfish_and_the_mulligan_oracle_refuse_kyodai() {
    refused(&["--goldfish", "kyodai", "--quiet"]);
    refused(&["--goldfish", "kyodai", "--trace", "0", "--quiet"]);
    let (code, _, err) = commander_sim(&["--mulligan-oracle", "kyodai", "--quiet"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("--mulligan-oracle: kyodai") && err.contains("Cali Commander"), "{err}");
    let (code, _, err) = commander_sim(&["--mulligan-oracle", "kyodai", "--vs-tier", "--quiet"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("Cali Commander"), "{err}");
}
