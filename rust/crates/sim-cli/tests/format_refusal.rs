//! What the real binary refuses before any game is played. kyodai plays Cali Commander (1v1, 30 life): it is refused at
//! a Commander table (`--format commander`), and the five modes that read four seats (`--calibrate`, `--realism`,
//! `--fit`, `--mulligan-oracle`, `--vs-tier`) are refused for it by name. Each exits 1 with the reason on stderr and no
//! game on stdout. The tier, key and table logic is unit-tested in src/poolmode.rs, src/duel.rs and src/goldfish.rs;
//! this runs the binary only on paths that exit before a game.

use std::process::Command;

fn commander_sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn no_game(args: &[&str], out: &str) {
    for game in ["Play-by-play", "Winner:", "R1 ", "Round 1", "Win rate", "goldfish ["] {
        assert!(!out.contains(game), "{args:?} played a game ({game:?}):\n{out}");
    }
}

/// exit 1, the reason on stderr (naming `must`), and no game on stdout
fn refused(args: &[&str], must: &[&str]) -> String {
    let (code, out, err) = commander_sim(args);
    assert_eq!(code, 1, "{args:?}\nstdout:\n{out}\nstderr:\n{err}");
    for m in must {
        assert!(err.contains(m), "{args:?}: {m:?} missing from\n{err}");
    }
    no_game(args, &out);
    err
}

/// a Cali deck is refused at a Commander table, by every mode that seats it
#[test]
fn a_commander_table_refuses_kyodai() {
    let why = ["Cali Commander", "kyodai", "Commander table"];
    refused(&["--deck", "kyodai", "--pool", "t5", "--trace", "0", "--quiet", "--format", "commander"], &why);
    refused(&["--deck", "kyodai", "--pool", "t6", "--trace", "0", "--quiet", "--format", "commander"], &why);
    refused(
        &["--deck", "kyodai", "--pool", "t1", "--any-tier", "--trace", "0", "--quiet", "--format", "commander"],
        &why,
    );
    refused(&["--deck", "kyodai", "--pool", "t5", "--games", "2", "--quiet", "--format", "commander"], &why);
    refused(
        &["--deck", "kyodai", "--pool", "all", "--games", "2", "--quiet", "--any-tier", "--format", "commander"],
        &why,
    );
    refused(&["--goldfish", "kyodai", "--quiet", "--format", "commander"], &["--goldfish: kyodai", "Cali Commander"]);
    refused(
        &["--goldfish", "kyodai", "--trace", "0", "--quiet", "--format", "commander"],
        &["--goldfish: kyodai", "Cali Commander"],
    );
    refused(
        &["--mulligan-oracle", "kyodai", "--quiet", "--format", "commander"],
        &["--mulligan-oracle: kyodai", "Cali Commander"],
    );
}

/// each of the five modes that read four seats is refused for kyodai, by name, with no game. `--realism` and `--fit` take
/// no deck (they refuse `--deck` themselves, as before), so for them it is `--format cali` that selects the table.
#[test]
fn the_four_seat_modes_are_refused_for_a_cali_deck() {
    let cases: [(&[&str], &str); 5] = [
        (&["--deck", "kyodai", "--calibrate", "within", "--quiet"], "--calibrate"),
        (&["--mulligan-oracle", "kyodai", "--quiet"], "--mulligan-oracle"),
        (&["--mulligan-oracle", "yoshimaru,kyodai", "--quiet"], "--mulligan-oracle"),
        (&["--mulligan-oracle", "kyodai", "--vs-tier", "--quiet"], "--vs-tier"),
        (&["--mulligan-oracle", "t6,kyodai", "--vs-tier", "--quiet"], "--vs-tier"),
    ];
    for (args, mode) in cases {
        let err = refused(args, &[mode, "kyodai", "Cali Commander", "is not available for Cali Commander yet"]);
        assert!(err.contains("1v1"), "{args:?}: {err}");
    }
    for mode in ["--realism", "--fit"] {
        let err = refused(&["--deck", "kyodai", mode, "--pool", "t5", "--quiet"], &["drop --deck"]);
        assert!(!err.contains("not available for"), "{err}");
    }
}

/// the same modes for `--format cali` (a Commander deck, or no deck at all) are refused too, `--realism` and `--fit`
/// included; they stay available for a Commander table
#[test]
fn the_four_seat_modes_are_refused_at_a_cali_table() {
    let cases: [(&[&str], &str); 5] = [
        (&["--calibrate", "within", "--format", "cali", "--quiet"], "--calibrate"),
        (&["--realism", "--pool", "t5", "--format", "cali", "--quiet"], "--realism"),
        (&["--fit", "--pool", "t5", "--format", "cali", "--quiet"], "--fit"),
        (&["--deck", "yshtola", "--calibrate", "within", "--format", "cali", "--quiet"], "--calibrate"),
        (&["--mulligan-oracle", "yshtola", "--format", "cali", "--quiet"], "--mulligan-oracle"),
    ];
    for (args, mode) in cases {
        refused(args, &[mode, "Cali Commander", "is not available for Cali Commander yet"]);
    }
}

/// `searchtest` measures the AI, and the 2-seat AI is the 4-player one until C4: refused for kyodai, no game
#[test]
fn searchtest_is_refused_for_a_cali_deck() {
    for tier in ["t5", "t6"] {
        refused(
            &["searchtest", "kyodai", tier, "1"],
            &["searchtest", "kyodai", "Cali Commander", "is not available for Cali Commander yet"],
        );
    }
}

/// the deck-less `--trace` is a Commander table: `--format cali` is refused with it. `--pool tN --trace` alone already needs
/// `--deck`; the one deck-less trace is `--calibrate within --pool tN --trace`, which `check_modes` refuses first (the
/// guard in `poolmode::trace_pool` is the second line of defence).
#[test]
fn a_deckless_trace_refuses_a_cali_format() {
    refused(
        &["--format", "cali", "--calibrate", "within", "--pool", "t5", "--trace", "0", "--quiet"],
        &["--calibrate", "Cali Commander", "is not available for Cali Commander yet"],
    );
    let err = refused(&["--format", "cali", "--pool", "t5", "--trace", "0", "--quiet"], &["--deck is required"]);
    assert!(!err.contains("Play-by-play"), "{err}");
}

/// `--format` takes a format of data/decks.json, and not with `--all-decks`
#[test]
fn the_format_flag_is_checked() {
    refused(
        &["--deck", "yshtola", "--pool", "t5", "--format", "nowhere", "--quiet"],
        &["--format nowhere", "cali, commander"],
    );
    refused(&["--all-decks", "--pool", "t5", "--format", "cali", "--quiet"], &["--format", "--all-decks"]);
}

/// a Commander deck at a Cali table is admitted unless a card of it is banned there (none of JD's decks): the pool
/// deck with The Tabernacle at Pendrell Vale is refused at a Cali goldfish, before any game
#[test]
fn a_pool_deck_with_a_cali_banned_card_is_refused_at_a_cali_table() {
    let err = refused(&["--goldfish", "dack-fayden-white-stax", "--format", "cali", "--quiet"], &["Cali Commander"]);
    assert!(err.contains("Tabernacle"), "{err}");
}

/// the card list and the pool decks of swaptest are unaffected
#[test]
fn cards_and_swaptest_are_unchanged() {
    let (code, out, err) = commander_sim(&["--deck", "kyodai", "--cards"]);
    assert_eq!(code, 0, "{out}\n{err}");
    assert!(out.contains("kyodai:") && out.contains("Mana Crypt"), "{out}");
    let (code, _, err) = commander_sim(&["swaptest", "kyodai", "t5", "1", "Sol Ring>Swiftfoot Boots"]);
    assert_eq!(code, 1);
    assert!(err.contains("no pool deck"), "{err}");
}
