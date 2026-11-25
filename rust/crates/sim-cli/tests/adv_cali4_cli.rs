// written by an independent reviewer
//! Adversarial tests for C4 through the real binary. Only a bad `TABLE_AI` is run: it must stop before any game is
//! played (the settings are built before the games). No simulation is run.

use std::process::Command;

fn run(args: &[&str], table_ai: Option<&str>) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_commander-sim"));
    c.args(args).env_remove("TABLE_AI");
    if let Some(v) = table_ai {
        c.env("TABLE_AI", v);
    }
    let out = c.output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn no_game(out: &str) {
    // (the usage text itself says "win rate", so only the markers a played game prints are checked)
    for game in ["Play-by-play", "Winner:", "On the play", "analysis ["] {
        assert!(!out.contains(game), "a game was played ({game:?}):\n{out}");
    }
}

/// every spelling that `TableAi::turned_off` refuses stops the run with a message naming TABLE_AI, before any game, for a
/// Commander deck and for a Cali deck alike
#[test]
fn adv_a_bad_table_ai_stops_the_run_before_any_game() {
    for deck in ["karn_gilded", "arzakon"] {
        for bad in ["bogus", "Commander", "go_rules,", "commander,go_rules", "all"] {
            let (code, out, err) = run(
                &["--deck", deck, "--pool", "t5", "--games", "1", "--no-preflight", "--quiet", "--jobs", "1"],
                Some(bad),
            );
            no_game(&out);
            assert!(err.contains("bad TABLE_AI"), "{deck} {bad:?}: exit {code}\nstdout:\n{out}\nstderr:\n{err}");
            assert_ne!(code, 0, "{deck} {bad:?}");
        }
    }
}

/// an empty TABLE_AI is the same as unset: `--help` (no game) still works with it set
#[test]
fn adv_table_ai_does_not_break_usage_paths() {
    for v in [None, Some(""), Some("commander"), Some("bogus")] {
        let (code, out, _err) = run(&["--help"], v);
        assert_eq!(code, 0, "{v:?}");
        no_game(&out);
    }
}
