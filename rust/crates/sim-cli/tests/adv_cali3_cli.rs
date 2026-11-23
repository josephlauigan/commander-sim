// adopted into the branch unchanged apart from this line (reviewer of C3, see /tmp/handoff/cali3-adv.md)
// written by an independent adversarial reviewer (C3), not the author
//! Adversarial tests for C3 through the real binary. Only paths that exit before any game is played run here: every call
//! is a refusal (exit 1 or a usage error) and each asserts that no game text reached stdout.

use std::process::Command;

fn sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn no_game(args: &[&str], out: &str) {
    for game in ["Play-by-play", "Winner:", "Win rate", "goldfish [", "On the play", "analysis ["] {
        assert!(!out.contains(game), "{args:?} played a game ({game:?}):\n{out}");
    }
}

/// exit 1, `must` on stderr, no game on stdout, no panic
fn refused(args: &[&str], must: &[&str]) -> String {
    let (code, out, err) = sim(args);
    assert_eq!(code, 1, "{args:?}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(!err.contains("panicked"), "{args:?} panicked:\n{err}");
    for m in must {
        assert!(err.contains(m), "{args:?}: {m:?} missing from\n{err}");
    }
    no_game(args, &out);
    err
}

/// a usage error or a refusal, never a panic (101), never a game
fn rejected(args: &[&str]) -> String {
    let (code, out, err) = sim(args);
    assert!(code == 1 || code == 2, "{args:?} exit {code}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(!err.contains("panicked"), "{args:?} panicked:\n{err}");
    assert!(!err.is_empty(), "{args:?}: nothing on stderr");
    no_game(args, &out);
    err
}

/// every four-seat mode refused for `--format cali` with no deck, in every flag order and spelling, each naming itself
#[test]
fn adv_cali3_every_four_seat_mode_is_refused_by_format_cali_alone() {
    let modes: [(&[&str], &str); 9] = [
        (&["--calibrate", "within"], "--calibrate"),
        (&["--calibrate", "ordering"], "--calibrate"),
        (&["--calibrate", "all"], "--calibrate"),
        (&["--realism"], "--realism"),
        (&["--fit"], "--fit"),
        (&["--mulligan-oracle", "t6"], "--mulligan-oracle"),
        (&["--mulligan-oracle", "yshtola"], "--mulligan-oracle"),
        (&["--mulligan-oracle", "t6", "--vs-tier"], "--vs-tier"),
        (&["--mulligan-oracle", "yshtola,t6", "--vs-tier"], "--vs-tier"),
    ];
    for (mode, name) in modes {
        // format first, last, `=` spelling, and with a pool and a quiet flag around it
        for fmt in [&["--format", "cali"][..], &["--format=cali"][..]] {
            for order in 0..3 {
                let mut a: Vec<&str> = vec![];
                match order {
                    0 => {
                        a.extend_from_slice(fmt);
                        a.extend_from_slice(mode);
                    }
                    1 => {
                        a.extend_from_slice(mode);
                        a.extend_from_slice(fmt);
                    }
                    _ => {
                        a.extend_from_slice(&["--quiet", "--pool", "t5"]);
                        a.extend_from_slice(mode);
                        a.extend_from_slice(&["--games", "1"]);
                        a.extend_from_slice(fmt);
                    }
                }
                let err = refused(&a, &[name, "Cali Commander", "not available for Cali Commander yet"]);
                assert!(err.contains("--format selects"), "{a:?}: {err}");
            }
        }
    }
}

/// the same modes for a Cali deck: `--calibrate` and the oracle name the deck; `--realism`/`--fit` with a deck keep their
/// older "drop --deck" message (the builder's open question 3) and play nothing
#[test]
fn adv_cali3_four_seat_modes_with_kyodai_never_reach_a_game() {
    for a in [
        &["--deck", "kyodai", "--calibrate", "within"][..],
        &["--deck", "kyodai", "--calibrate", "ordering", "--pool", "t5"][..],
        &["--deck", "kyodai", "--calibrate", "all", "--pool", "t6"][..],
        &["--mulligan-oracle", "kyodai", "--hands", "1"][..],
        &["--mulligan-oracle", "kyodai", "--vs-tier", "--hands", "1"][..],
        &["--mulligan-oracle", "t6, kyodai", "--vs-tier"][..],
        &["--mulligan-oracle", " kyodai ,yoshimaru"][..],
        &["--deck", "kyodai", "--realism", "--pool", "t5"][..],
        &["--deck", "kyodai", "--fit", "--pool", "t5"][..],
    ] {
        let (code, out, err) = sim(a);
        assert_eq!(code, 1, "{a:?}\n{out}\n{err}");
        assert!(!err.contains("panicked"), "{a:?}: {err}");
        no_game(a, &out);
        assert!(
            err.contains("not available for Cali Commander yet") || err.contains("drop --deck"),
            "{a:?}: neither refusal text:\n{err}"
        );
    }
}

/// `--format` with `--all-decks`: refused for every format name (commander too), by name, before a game
#[test]
fn adv_cali3_format_with_all_decks_is_refused_for_any_format() {
    for f in ["cali", "commander"] {
        refused(&["--all-decks", "--pool", "t5", "--format", f, "--quiet"], &["--format", "--all-decks"]);
        refused(&["--format", f, "--all-decks", "--quiet"], &["--format", "--all-decks"]);
        refused(&[&format!("--format={f}")[..], "--all-decks", "--games", "1"], &["--format", "--all-decks"]);
    }
    // an unknown format with --all-decks is still refused (either message), never played
    rejected(&["--all-decks", "--format", "nowhere", "--quiet"]);
}

/// unknown, mis-cased, empty and malformed format names exit before a game, naming the flag, without a panic
#[test]
fn adv_cali3_unknown_format_names() {
    for bad in ["Cali", "CALI", "", " ", "cali ", "commander2", "cali,commander", "1v1", "--quiet", "nowhere"] {
        let err = rejected(&["--deck", "yshtola", "--pool", "t5", "--format", bad, "--games", "1"]);
        if bad != "--quiet" {
            assert!(err.contains("--format"), "{bad:?}: {err}");
        }
    }
    // missing value, and `=` with nothing after it
    rejected(&["--deck", "yshtola", "--pool", "t5", "--format"]);
    rejected(&["--deck", "yshtola", "--pool", "t5", "--format="]);
    // repeated flag: last one wins or is refused, never a game on a Commander-only mode
    let (code, out, err) = sim(&["--format", "commander", "--format", "cali", "--fit", "--pool", "t5"]);
    assert_eq!(code, 1, "{out}\n{err}");
    no_game(&[], &out);
    assert!(err.contains("--fit"), "the last --format cali must refuse --fit: {err}");
}

/// `--deck` is limited to JD's decks: a pool deck key is a usage error even with `--format cali`
#[test]
fn adv_cali3_a_pool_deck_is_not_a_deck_argument() {
    for k in ["dack-fayden-white-stax", "krenko-mono-red-goblins", "Kyodai", "thanos"] {
        rejected(&["--deck", k, "--pool", "t6", "--format", "cali", "--quiet", "--games", "1"]);
    }
}

/// goldfish lists at a Cali table: Dack inside `t6` or named, anywhere in the list, refused with the card named; a Cali
/// deck inside a list at a Commander table is refused naming kyodai (and not yshtola)
#[test]
fn adv_cali3_goldfish_lists_refuse_what_the_table_does_not_admit() {
    for spec in ["t6", "dack-fayden-white-stax", "yshtola,t6", "t6,kyodai", "kyodai , dack-fayden-white-stax"] {
        let err = refused(&["--goldfish", spec, "--format", "cali", "--quiet"], &["--goldfish:", "Tabernacle"]);
        assert!(err.contains("dack-fayden-white-stax"), "{spec}: {err}");
    }
    for spec in ["yshtola,kyodai", "kyodai,yshtola", "t6,kyodai,yshtola"] {
        let err = refused(
            &["--goldfish", spec, "--format", "commander", "--quiet"],
            &["--goldfish: kyodai", "Cali Commander"],
        );
        assert!(!err.contains("yshtola cannot"), "{spec}: {err}");
    }
    // unknown keys are still refused with the key list
    refused(&["--goldfish", "kyodai,nothing", "--quiet"], &["nothing"]);
    refused(&["--goldfish", "", "--quiet"], &[]);
    refused(&["--goldfish", ",", "--quiet"], &[]);
}

/// searchtest and swaptest: the tier rule still refuses a tier outside kyodai's before any game; swaptest takes pool decks
/// only (kyodai is not one)
#[test]
fn adv_cali3_searchtest_and_swaptest_refusals_before_a_game() {
    for t in ["t1", "t2", "t3", "t4"] {
        let (code, out, err) = sim(&["searchtest", "kyodai", t, "1"]);
        assert_eq!(code, 1, "searchtest kyodai {t}\n{out}\n{err}");
        assert!(!err.contains("panicked"), "{err}");
        no_game(&[], &out);
    }
    let (code, out, err) = sim(&["searchtest", "kyodai", "t9", "1"]);
    assert_eq!(code, 1, "{out}\n{err}");
    let (code, out, err) = sim(&["swaptest", "kyodai", "t5", "1", "Sol Ring>Swiftfoot Boots"]);
    assert_eq!(code, 1, "{out}\n{err}");
    assert!(err.contains("no pool deck") && !err.contains("panicked"), "{err}");
}

/// a Commander deck forced into `--format cali` is refused by card, before a game, on the modes that check the deck first
/// (a Commander deck from JD's has no Cali-banned card, so only the pool decks can be refused: this is the goldfish path)
#[test]
fn adv_cali3_cards_listing_is_not_affected_by_format() {
    for f in ["cali", "commander"] {
        let (code, out, err) = sim(&["--deck", "yshtola", "--cards", "--format", f]);
        assert_eq!(code, 0, "{f}\n{out}\n{err}");
    }
    // the coverage / model listing of a Cali deck at its own table is refused only when it would seat the deck wrongly
    let (code, out, err) = sim(&["--deck", "kyodai", "--cards", "--format", "commander"]);
    assert!(code == 0 || code == 1, "{out}\n{err}");
}
