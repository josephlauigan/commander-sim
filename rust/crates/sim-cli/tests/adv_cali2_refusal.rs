// written by an independent reviewer
//! Adversarial tests for C2 (the sim-cli refusal of a deck whose format the simulator cannot seat). Written by a reviewer,
//! not the author. Only paths that exit before any game is played run here.
//!
//! C3 plays kyodai at its own (Cali) table, so the Commander-table refusals these tests pinned now need `--format
//! commander` to be reached (a Cali deck is refused at a Commander table); each call below passes it. The `searchtest`
//! refusals are gone: `searchtest kyodai` plays a Cali game.

use std::process::Command;

fn sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn no_game(out: &str) {
    for g in ["Play-by-play", "Winner:", "R1 ", "Round 1"] {
        assert!(!out.contains(g), "a game was played ({g:?}):\n{out}");
    }
}

fn refused(args: &[&str]) {
    let (code, out, err) = sim(args);
    assert_eq!(code, 1, "{args:?}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(err.contains("Cali Commander") && err.contains("kyodai"), "{args:?}: {err}");
    no_game(&out);
}

/// every `--deck kyodai --pool ...` mode that would seat the deck at a Commander table: swap, analyze, preflight, each
/// tier, with --any-tier
#[test]
fn every_pool_mode_refuses_kyodai_at_a_commander_table() {
    let swap = "Sol Ring=>Swiftfoot Boots";
    for pool in ["t5", "t6", "all", "t3"] {
        let c = ["--format", "commander"];
        refused(&[&c[..], &["--deck", "kyodai", "--pool", pool, "--analyze", "--quiet", "--games", "1"]].concat());
        refused(
            &[&c[..], &["--deck", "kyodai", "--pool", pool, "--swap", swap, "--no-explain", "--quiet", "--games", "1"]]
                .concat(),
        );
        refused(
            &[
                &c[..],
                &[
                    "--deck",
                    "kyodai",
                    "--pool",
                    pool,
                    "--swap",
                    swap,
                    "--no-preflight",
                    "--no-explain",
                    "--quiet",
                    "--games",
                    "1",
                ],
            ]
            .concat(),
        );
        refused(
            &[
                &c[..],
                &["--deck", "kyodai", "--pool", pool, "--swap", swap, "--preflight-only", "--no-explain", "--quiet"],
            ]
            .concat(),
        );
        refused(
            &[&c[..], &["--deck", "kyodai", "--pool", pool, "--any-tier", "--brief", "--quiet", "--games", "1"]]
                .concat(),
        );
        refused(
            &[
                &c[..],
                &["--deck", "kyodai", "--pool", pool, "--profiles", "conservative,loose", "--quiet", "--games", "1"],
            ]
            .concat(),
        );
    }
}

/// `--goldfish` and `--mulligan-oracle` take a list: kyodai anywhere in it refuses the whole run (a Commander table for
/// `--goldfish`; the oracle is 4-seat by construction), with spaces too
#[test]
fn goldfish_and_oracle_lists_refuse_kyodai_anywhere() {
    for spec in ["kyodai", "yoshimaru,kyodai", "kyodai,yoshimaru", "yoshimaru , kyodai", "t6,kyodai", "kyodai,kyodai"] {
        refused(&["--goldfish", spec, "--format", "commander", "--quiet"]);
        let (code, out, err) = sim(&["--mulligan-oracle", spec, "--quiet"]);
        assert_eq!(code, 1, "{spec}\n{out}\n{err}");
        assert!(
            err.contains("Cali Commander") && err.contains("--mulligan-oracle") && err.contains("not available"),
            "{spec}: {err}"
        );
        no_game(&out);
    }
    let (code, _, err) = sim(&["--mulligan-oracle", "yoshimaru,kyodai", "--vs-tier", "--quiet"]);
    assert_eq!(code, 1);
    assert!(err.contains("Cali Commander"), "{err}");
}

/// non-game uses of the deck keep working: the card list, with and without a pool, with a swap
#[test]
fn cards_listing_still_works_for_kyodai() {
    for extra in [
        &[][..],
        &["--pool", "t5"][..],
        &["--pool", "t6", "--any-tier"][..],
        &["--swap", "Sol Ring=>Swiftfoot Boots", "--no-explain"][..],
    ] {
        let mut args = vec!["--deck", "kyodai", "--cards"];
        args.extend_from_slice(extra);
        let (code, out, err) = sim(&args);
        assert_eq!(code, 0, "{args:?}\n{out}\n{err}");
        assert!(out.contains("kyodai:") && out.contains("unique cards"), "{args:?}: {out}");
        assert!(out.contains("Mana Crypt"), "{args:?}: {out}");
    }
    // another deck is unaffected
    let (code, out, _) = sim(&["--deck", "yoshimaru", "--cards"]);
    assert_eq!(code, 0);
    assert!(out.contains("yoshimaru:"));
}

/// `--all-decks` for a tier only kyodai plays: no deck left, the deck is named as left out, nothing is played, exit 0
#[test]
fn all_decks_t6_leaves_kyodai_out_and_plays_nothing() {
    let (code, out, err) = sim(&["--all-decks", "--pool", "t6", "--quiet", "--games", "1"]);
    eprintln!("exit {code}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("leaves out kyodai"), "{out}");
    assert!(!out.contains("leaves out yshtola") && !out.contains("leaves out sheoldred"));
    no_game(&out);
}

/// the other decks and swaptest on a key that is not a pool deck
#[test]
fn swaptest_still_takes_pool_decks_only() {
    let (code, _, err) = sim(&["swaptest", "kyodai", "t5", "1", "Sol Ring>Swiftfoot Boots"]);
    assert_eq!(code, 1);
    assert!(err.contains("no pool deck"), "{err}");
}

/// argument parsing: the deck key must be exact and the refusal must not depend on the order of the flags
#[test]
fn flag_order_and_spelling() {
    let (code, _, err) = sim(&["--deck", "Kyodai", "--pool", "t5", "--quiet"]);
    assert_ne!(code, 0);
    assert!(!err.is_empty());
    refused(&["--quiet", "--games", "1", "--pool", "t5", "--deck", "kyodai", "--format", "commander"]);
    refused(&["--trace", "0", "--pool", "t6", "--deck", "kyodai", "--quiet", "--any-tier", "--format=commander"]);
}
