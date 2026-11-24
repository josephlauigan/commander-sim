//! C4, the data side in cardgen: `[<format>.ai]` in data/formats.toml. The parser is strict (an unknown key, `duel`
//! included, a bad `go_rules` or a bad type is an error), `[commander.ai]` cannot differ from Commander's profile, the
//! export is deterministic, and it reproduces the committed data/decks.json and cards.json byte for byte. (sim-core's
//! reader refuses the same things: `sim-core/tests/cali_c4_ai_data.rs`.)

use cardgen::data::{FormatDef, Project, parse_formats};
use cardgen::export::{Inputs, export, formats_doc};
use cardgen::json::{self, J};
use sim_core::format::{GoRules, TableAi};
use std::path::{Path, PathBuf};

const COMMANDER: &str =
    "[commander]\nname = \"Commander\"\nseats = 4\nlife = 40\nfirst_player_draws = true\nbans = \"scryfall\"\n";
const CALI: &str =
    "[x]\nname = \"X\"\nseats = 2\nlife = 30\nfirst_player_draws = false\nbans = \"list\"\nbanned = [\"A\"]\n";

fn parse(extra: &str) -> Result<Vec<FormatDef>, String> {
    parse_formats(&format!("{COMMANDER}{extra}"), Path::new("formats.toml"))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let t = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

fn scratch(name: &str) -> Project {
    let real = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cali_c4_{name}"));
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&real.join("data"), &root.join("data"));
    copy_dir(&real.join("decklists"), &root.join("decklists"));
    copy_dir(&real.join("tests/fixtures"), &root.join("tests/fixtures"));
    Project::new(root)
}

#[test]
fn an_ai_table_is_read_strictly_and_a_missing_key_keeps_commander_s_value() {
    let f = parse(&format!("{CALI}[x.ai]\nlearned_eval = false\ngo_rules = \"all\"\n")).unwrap();
    assert_eq!(f[1].ai, TableAi { learned_eval: false, go_rules: GoRules::All, duel: true });
    let f = parse(&format!("{CALI}[x.ai]\ngo_rules = \"all\"\n")).unwrap();
    assert_eq!(f[1].ai, TableAi { go_rules: GoRules::All, duel: true, ..TableAi::COMMANDER });
    // no table: Commander's values (duel from the seats)
    assert_eq!(parse(CALI).unwrap()[1].ai, TableAi { duel: true, ..TableAi::COMMANDER });
    for (extra, why) in [
        ("[x.ai]\nspeed = 1\n", "unknown key \"speed\""),
        ("[x.ai]\nduel = true\n", "unknown key \"duel\""),
        ("[x.ai]\ngo_rules = \"everyone\"\n", "go_rules must be"),
        ("[x.ai]\ngo_rules = true\n", "go_rules must be"),
        ("[x.ai]\nlearned_eval = \"no\"\n", "learned_eval must be true or false"),
    ] {
        let e = parse(&format!("{CALI}{extra}")).unwrap_err();
        assert!(e.contains(why), "{extra}: {e}");
    }
    // `ai = 1` on the format itself
    let e = parse(&format!("{}ai = 1\n", CALI)).unwrap_err();
    assert!(e.contains("must be a table"), "{e}");
}

#[test]
fn commander_s_ai_profile_cannot_change() {
    let ok = parse_formats(
        &format!("{COMMANDER}[commander.ai]\nlearned_eval = true\ngo_rules = \"t6\"\n"),
        Path::new("formats.toml"),
    )
    .unwrap();
    assert_eq!(ok[0].ai, TableAi::COMMANDER);
    for ai in ["learned_eval = false", "go_rules = \"all\""] {
        let e = parse_formats(&format!("{COMMANDER}[commander.ai]\n{ai}\n"), Path::new("formats.toml")).unwrap_err();
        assert!(e.contains("must stay 4 seats, 40 life") && e.contains("AI profile"), "{ai}: {e}");
    }
}

#[test]
fn every_format_exports_its_ai_block_without_duel() {
    let f = parse(&format!("{CALI}[x.ai]\nlearned_eval = false\ngo_rules = \"all\"\n")).unwrap();
    let doc = formats_doc(&f);
    let text = json::dumps(&doc, None, false);
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["commander"]["ai"], serde_json::json!({"learned_eval": true, "go_rules": "t6"}));
    assert_eq!(v["x"]["ai"], serde_json::json!({"learned_eval": false, "go_rules": "all"}));
}

#[test]
fn the_real_formats_toml_has_cali_s_profile() {
    let p = scratch("real_formats");
    let f = cardgen::data::load_formats(&p).unwrap();
    let cali = f.iter().find(|f| f.key == "cali").unwrap();
    assert_eq!(cali.ai, TableAi { learned_eval: false, go_rules: GoRules::All, duel: true });
    assert_eq!(f.iter().find(|f| f.key == "commander").unwrap().ai, TableAi::COMMANDER);
}

#[test]
fn exporting_twice_gives_identical_bytes_and_reproduces_the_committed_files() {
    let p = scratch("export");
    let a = export(&Inputs::load(&p).unwrap()).unwrap();
    let b = export(&Inputs::load(&p).unwrap()).unwrap();
    assert_eq!(a.decks_json, b.decks_json);
    assert_eq!(a.cards_json, b.cards_json);
    let real = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    assert!(
        a.decks_json == std::fs::read_to_string(real.join("decks.json")).unwrap(),
        "data/decks.json is stale: run cardgen export"
    );
    assert!(
        a.cards_json == std::fs::read_to_string(real.join("cards.json")).unwrap(),
        "data/cards.json is stale: run cardgen export"
    );
    // only `formats` carries the ai blocks
    let v = json::parse(&a.decks_json).unwrap();
    let text = json::dumps(&v, None, false);
    assert_eq!(text.matches("\"go_rules\"").count(), 2, "one ai block per format, none elsewhere");
    let _: Option<&J> = v.get("formats");
}
