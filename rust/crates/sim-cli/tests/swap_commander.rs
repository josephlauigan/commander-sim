//! `--swap-commander "OLD=>NEW"` through the real binary: the refusals before any game, and one very small paired run
//! (arzakon, Arzakon -> The Ur-Dragon, 8 games on t5) whose baseline must equal a plain run on the same seeds. The
//! only game-playing here; the checks of the pairs themselves are unit tests in src/swapcmd.rs.

use std::process::Command;

const ARZAKON: &str = "Arzakon"; // the deck's commander
const KYODAI: &str = "Kyodai, Soul of Kamigawa"; // a card of the 99
const URDRAGON: &str = "The Ur-Dragon"; // five colours, a legendary creature, Commander-legal, in no arzakon list

fn sim(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_commander-sim")).args(args).output().expect("run commander-sim");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// exit 1, `must` on stderr, no game on stdout, no panic
fn refused(args: &[&str], must: &[&str]) {
    let (code, out, err) = sim(args);
    assert_eq!(code, 1, "{args:?}\nstdout:\n{out}\nstderr:\n{err}");
    assert!(!err.contains("panicked"), "{args:?} panicked:\n{err}");
    for m in must {
        assert!(err.contains(m), "{args:?}: {m:?} missing from\n{err}");
    }
    for game in ["Win rate", "baseline vs variant", "paired breakdown", "Preflight"] {
        assert!(!out.contains(game), "{args:?} played a game ({game:?}):\n{out}");
    }
}

fn swap_cmd(pair: &str) -> Vec<String> {
    ["--deck", "arzakon", "--pool", "t5", "--games", "2", "--quiet", "--swap-commander", pair]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn refused_pair(pair: &str, extra: &[&str], must: &[&str]) {
    let mut v = swap_cmd(pair);
    v.extend(extra.iter().map(|s| s.to_string()));
    let v: Vec<&str> = v.iter().map(String::as_str).collect();
    refused(&v, must);
}

#[test]
fn a_bad_commander_change_is_refused_before_any_game() {
    // the old commander is not the deck's
    refused_pair(&format!("Sheoldred, the Apocalypse=>{URDRAGON}"), &[], &["is not arzakon's commander", ARZAKON]);
    // a malformed pair, and the same commander
    refused_pair("just a name", &[], &["--swap-commander must look like"]);
    refused_pair(&format!("{ARZAKON}=>{ARZAKON}"), &[], &["is already the commander"]);
    // a card the card data does not have
    refused_pair(&format!("{ARZAKON}=>No Such Card"), &[], &["Not in data/cards.json", "No Such Card"]);
    // not a legendary creature; outside the deck's colours (Sheoldred is mono-black, the list is five colours)
    refused_pair(&format!("{ARZAKON}=>Swiftfoot Boots"), &[], &["is not a legendary creature"]);
    refused_pair(&format!("{ARZAKON}=>Atraxa, Praetors' Voice"), &[], &["outside colour identity"]);
    // a card of the 99 as the new commander
    refused_pair(&format!("{ARZAKON}=>Counterspell"), &[], &["Counterspell"]);
    // a --swap whose card out is not in the list, and one that names the commander as a card out
    refused_pair(&format!("{ARZAKON}=>{URDRAGON}"), &["--swap", "No Such Card=>Sol Ring"], &["No Such Card"]);
    refused_pair(
        &format!("{ARZAKON}=>{URDRAGON}"),
        &["--swap", &format!("{ARZAKON}=>Sol Ring")],
        &["commander being replaced"],
    );
    // the option needs a deck and a paired pool run
    refused(
        &["--pool", "t5", "--swap-commander", &format!("{ARZAKON}=>{URDRAGON}")],
        &["--swap-commander needs --deck"],
    );
    refused(
        &["--all-decks", "--pool", "t5", "--swap-commander", &format!("{ARZAKON}=>{URDRAGON}")],
        &["--swap-commander needs --deck"],
    );
    // a second --swap-commander is a usage error
    let (code, _, err) = sim(&["--deck", "arzakon", "--swap-commander", "A=>B", "--swap-commander", "C=>D"]);
    assert_eq!(code, 2, "{err}");
}

/// a plain `--swap` may not name the deck's commander, out or in; the message points at `--swap-commander`
#[test]
fn a_plain_swap_naming_the_commander_is_refused_before_any_game() {
    let base = ["--deck", "arzakon", "--pool", "t5", "--games", "2", "--quiet", "--swap"];
    for pair in [format!("{ARZAKON}=>Sol Ring"), format!("{KYODAI}=>{ARZAKON}")] {
        let args: Vec<&str> = base.iter().copied().chain([pair.as_str()]).collect();
        refused(&args, &[&format!("'{ARZAKON}' is the deck's commander"), "--swap-commander"]);
    }
}

/// the value after `label` up to the next space
fn field<'a>(text: &'a str, label: &str) -> &'a str {
    let i = text.find(label).unwrap_or_else(|| panic!("{label:?} missing from\n{text}")) + label.len();
    text[i..].split_whitespace().next().unwrap()
}

#[test]
fn a_paired_commander_run_plays_both_lists_explains_the_commander_and_keeps_the_baseline() {
    let common = ["--deck", "arzakon", "--pool", "t5", "--games", "8", "--ai", "adaptive", "--quiet", "--jobs", "2"];
    let (code, plain, err) = sim(&common);
    assert_eq!(code, 0, "{err}\n{plain}");
    let (code, out, err) =
        sim(&[&common[..], &["--no-preflight", "--swap-commander", &format!("{ARZAKON}=>{URDRAGON}")]].concat());
    assert_eq!(code, 0, "{err}\n{out}");
    assert!(!err.contains("panicked"), "{err}");

    // the model status of the new commander before the run, and the commander named in the swaps line
    assert!(out.contains("Model status of the swapped-in cards") && out.contains(URDRAGON), "{out}");
    assert!(out.contains(&format!("Swaps: commander {ARZAKON} -> {URDRAGON}")), "{out}");
    // the variant's coverage line counts the new commander, not the old one
    assert!(out.contains("arzakon (swapped)"), "{out}");

    // both lists were played: the paired line "win a% -> b%" has both, n games each
    assert!(out.contains("baseline vs variant (paired"), "{out}");
    let pair = out.lines().find(|l| l.contains("win ") && l.contains(" -> ") && l.contains("n=8")).expect(&out);
    let base_win = field(pair, "win ");
    let plain_win = field(&plain, "Win rate");
    assert_eq!(base_win, plain_win, "baseline win rate differs from a plain run on the same seeds\n{pair}\n{plain}");
    // the play / draw split of the baseline is the plain run's: "On the play 50.0% (2/4), on the draw ..."
    let plain_sides = plain.lines().find(|l| l.starts_with("On the play")).expect(&plain);
    let sides: Vec<&str> = out.lines().filter(|l| l.contains("on the play") || l.contains("on the draw")).collect();
    assert_eq!(sides.len(), 2, "{out}");
    for (word, plain_word) in [("on the play", "On the play"), ("on the draw", "on the draw")] {
        let want = &plain_sides[plain_sides.find(plain_word).unwrap() + plain_word.len()..];
        let want = want.split_whitespace().next().unwrap();
        let got = sides.iter().find(|l| l.contains(word)).unwrap();
        assert_eq!(field(got, &format!("{word} win ")), want.trim_end_matches(','), "{word}\n{got}\n{plain_sides}");
    }

    // the explain section: the new commander in the IN table (variant), the old one in the OUT table (baseline), both
    // with the command-zone reading, and the game shape of both lists
    let why = &out[out.find("-- Why: paired breakdown").expect(&out)..];
    let (ins, outs) = why.split_once("Swapped OUT").expect(why);
    assert!(ins.contains(&format!("{URDRAGON} (cmdr)")), "{why}");
    assert!(outs.contains(&format!("{ARZAKON} (cmdr)")), "{why}");
    assert!(why.contains("starts in the command zone") && why.contains("commander removals, per game"), "{why}");
    assert!(why.contains("current") && why.contains("swapped") && why.contains("win rate %"), "{why}");
}

/// `--no-explain` keeps the paired line and drops the breakdown, as for --swap; a --swap rides along on the 99 and the
/// old commander can come in as a card (here the pair of the real Arzakon / Kyodai change: the new commander is the 99
/// card the same run swaps out)
#[test]
fn no_explain_and_a_combined_swap_work() {
    let (code, out, err) = sim(&[
        "--deck",
        "arzakon",
        "--pool",
        "t5",
        "--games",
        "4",
        "--ai",
        "adaptive",
        "--quiet",
        "--jobs",
        "2",
        "--no-preflight",
        "--no-explain",
        "--swap-commander",
        &format!("{ARZAKON}=>{KYODAI}"),
        "--swap",
        &format!("{KYODAI}=>{ARZAKON}"),
    ]);
    assert_eq!(code, 0, "{err}\n{out}");
    assert!(out.contains("baseline vs variant (paired"), "{out}");
    assert!(!out.contains("paired breakdown") && !out.contains("Model status of the swapped-in cards"), "{out}");
    assert!(out.contains(&format!("Swaps: commander {ARZAKON} -> {KYODAI}; {KYODAI} -> {ARZAKON}")), "{out}");
}

/// the preflight reads the new commander like a swapped-in card: its AI rule and its dry-run cast share
#[test]
fn the_preflight_reads_the_new_commander() {
    let (code, out, err) = sim(&[
        "--deck",
        "arzakon",
        "--pool",
        "t5",
        "--quiet",
        "--jobs",
        "2",
        "--preflight-only",
        "--no-explain",
        "--swap-commander",
        &format!("{ARZAKON}=>{URDRAGON}"),
    ]);
    assert!(code == 0 || code == 3, "{err}\n{out}");
    assert!(out.contains(&format!("AI: {URDRAGON}: priority")), "{out}");
    assert!(out.contains(&format!("commander {URDRAGON}")), "{out}");
}
