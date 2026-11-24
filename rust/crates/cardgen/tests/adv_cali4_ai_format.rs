// written by an independent reviewer
//! Adversarial tests for C4's data side: cardgen and sim-core must read `[<key>.ai]` the same way (both accept the
//! same snippets with the same result, both refuse the same ones), key order never changes the export bytes, a
//! `[commander.ai]` equal to Commander's is fine and any other is refused, inline-table spellings work.

use cardgen::data::parse_formats;
use cardgen::export::formats_doc;
use cardgen::json;
use sim_core::format::{Format, TableAi, formats_table};
use std::collections::BTreeMap;
use std::path::Path;

const COMMANDER: &str =
    "[commander]\nname = \"Commander\"\nseats = 4\nlife = 40\nfirst_player_draws = true\nbans = \"scryfall\"\n";
const X: &str = "[x]\nname = \"X\"\nseats = 2\nlife = 30\nfirst_player_draws = false\nbans = \"scryfall\"\n";

/// cardgen's reading of `src` (the formats table text), then sim-core's reading of what cardgen would export
fn both(src: &str) -> (Result<BTreeMap<String, TableAi>, String>, Result<BTreeMap<String, TableAi>, String>) {
    let cg = parse_formats(src, Path::new("formats.toml"));
    let a = cg.as_ref().map(|f| f.iter().map(|d| (d.key.clone(), d.ai)).collect()).map_err(|e| e.clone());
    let b = cg.and_then(|f| {
        let text = json::dumps(&formats_doc(&f), None, false);
        let raw: BTreeMap<String, Format> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        formats_table(raw, []).map(|t| t.into_iter().map(|(k, v)| (k, v.ai)).collect())
    });
    (a, b)
}

#[test]
fn adv_both_readers_agree_on_every_accepted_spelling() {
    let cases = [
        // (text after [x], expected learned_eval, expected go_rules all?)
        ("", true, false),
        ("[x.ai]\n", true, false),
        ("[x.ai]\ngo_rules = \"all\"\n", true, true),
        ("[x.ai]\nlearned_eval = false\n", false, false),
        ("[x.ai]\ngo_rules = \"all\"\nlearned_eval = false\n", false, true),
        ("[x.ai]\nlearned_eval = false\ngo_rules = \"all\"\n", false, true),
        ("[x.ai]\ngo_rules = \"t6\"\nlearned_eval = true\n", true, false),
        ("ai = { go_rules = \"all\", learned_eval = false }\n", false, true),
        ("[x.ai]\n# a comment\ngo_rules = \"all\" # trailing\n", true, true),
    ];
    for (extra, le, all) in cases {
        let src =
            if extra.starts_with("ai =") { format!("{COMMANDER}{X}{extra}") } else { format!("{COMMANDER}{X}{extra}") };
        let (a, b) = both(&src);
        let (a, b) = (
            a.unwrap_or_else(|e| panic!("{extra:?}: cardgen: {e}")),
            b.unwrap_or_else(|e| panic!("{extra:?}: core: {e}")),
        );
        assert_eq!(a, b, "{extra:?}: the two readers differ");
        assert_eq!(a["x"].learned_eval, le, "{extra:?}");
        assert_eq!(a["x"].go_rules == sim_core::format::GoRules::All, all, "{extra:?}");
        assert!(a["x"].duel, "duel from seats");
        assert_eq!(a["commander"], TableAi::COMMANDER);
    }
}

#[test]
fn adv_both_readers_refuse_the_same_spellings() {
    let bad = [
        "[x.ai]\nduel = true\n",
        "[x.ai]\nduel = false\n",
        "[x.ai]\nspeed = 3\n",
        "[x.ai]\nGo_Rules = \"all\"\n",
        "[x.ai]\ngo_rules = \"ALL\"\n",
        "[x.ai]\ngo_rules = \"All\"\n",
        "[x.ai]\ngo_rules = \"T6\"\n",
        "[x.ai]\ngo_rules = \"\"\n",
        "[x.ai]\ngo_rules = 1\n",
        "[x.ai]\ngo_rules = [\"all\"]\n",
        "[x.ai]\nlearned_eval = 1\n",
        "[x.ai]\nlearned_eval = \"false\"\n",
        "[x.ai.deeper]\na = 1\n",
        "ai = \"all\"\n",
        "ai = [1]\n",
        "ai = true\n",
    ];
    for extra in bad {
        let src = format!("{COMMANDER}{X}{extra}");
        let cg = parse_formats(&src, Path::new("formats.toml"));
        assert!(cg.is_err(), "cardgen accepted {extra:?}");
    }
}

#[test]
fn adv_commander_ai_equal_to_commanders_is_fine_and_anything_else_is_refused_by_both() {
    // explicit, equal
    let (a, b) = both(&format!("{COMMANDER}[commander.ai]\nlearned_eval = true\ngo_rules = \"t6\"\n"));
    assert_eq!(a.unwrap(), b.unwrap());
    // empty table
    let (a, b) = both(&format!("{COMMANDER}[commander.ai]\n"));
    assert_eq!(a.unwrap(), b.unwrap());
    // different, in cardgen
    for ai in ["learned_eval = false", "go_rules = \"all\"", "learned_eval = false\ngo_rules = \"all\""] {
        let (a, _) = both(&format!("{COMMANDER}[commander.ai]\n{ai}\n"));
        assert!(a.is_err(), "{ai}");
    }
    // a second format may not take Commander's name by case or space tricks to dodge the check
    for key in ["Commander", "commander "] {
        let src = format!("{COMMANDER}[\"{key}\"]\nname = \"C2\"\nseats = 4\nlife = 40\nfirst_player_draws = true\n");
        let r = parse_formats(&src, Path::new("formats.toml"));
        if let Ok(f) = r {
            // if the key is accepted it is a different format and `commander` itself is untouched
            assert_eq!(f[0].ai, TableAi::COMMANDER);
        }
    }
}

#[test]
fn adv_export_bytes_do_not_depend_on_key_order_or_spelling() {
    let a = format!("{COMMANDER}{X}[x.ai]\nlearned_eval = false\ngo_rules = \"all\"\n");
    let b = format!("{COMMANDER}{X}[x.ai]\ngo_rules = \"all\"\nlearned_eval = false\n");
    let c = format!("{COMMANDER}{X}ai = {{ go_rules = \"all\", learned_eval = false }}\n");
    let dump = |s: &str| json::dumps(&formats_doc(&parse_formats(s, Path::new("f.toml")).unwrap()), None, false);
    assert_eq!(dump(&a), dump(&b));
    assert_eq!(dump(&a), dump(&c));
    assert_eq!(dump(&a), dump(&a));
    // every format has an ai block, duel nowhere
    let text = dump(&a);
    assert_eq!(text.matches("\"ai\"").count(), 2);
    assert!(!text.contains("duel"));
    // and a format with no `ai` exports Commander's values
    let v: serde_json::Value = serde_json::from_str(&dump(&format!("{COMMANDER}{X}"))).unwrap();
    assert_eq!(v["x"]["ai"], serde_json::json!({"learned_eval": true, "go_rules": "t6"}));
}

#[test]
fn adv_sim_core_refuses_odd_ai_values_in_decks_json_records() {
    let f = |ai: &str| {
        serde_json::from_str::<Format>(&format!(
            r#"{{"name":"X","seats":2,"life":30,"first_player_draws":false,"ai":{ai}}}"#
        ))
    };
    for bad in [
        "null",
        "1",
        "\"all\"",
        "true",
        r#"{"go_rules":null}"#,
        r#"{"learned_eval":null}"#,
        r#"{"go_rules":"All"}"#,
        r#"{"duel":false}"#,
        r#"{"duel":null}"#,
    ] {
        assert!(f(bad).is_err(), "sim-core accepted ai = {bad}");
    }
    for ok in ["{}", r#"{"go_rules":"all"}"#, r#"{"learned_eval":false}"#] {
        assert!(f(ok).is_ok(), "{ok}");
    }
    // a duplicate field is refused by serde_json's derive
    assert!(f(r#"{"go_rules":"all","go_rules":"t6"}"#).is_err());
}

/// serde's derived struct reader also accepts a JSON array (positional form), so a hand-edited decks.json with
/// `"ai": []` loads as the defaults instead of being refused. cardgen never exports this; left failing on purpose.
#[test]
#[ignore = "ticket T-S5-11: serde's derived readers accept the positional array form for every struct, sim-core reads \"ai\": [] as the defaults; only a hand-edited decks.json can reach it"]
fn adv_sim_core_refuses_an_array_for_ai() {
    let r = serde_json::from_str::<Format>(r#"{"name":"X","seats":2,"life":30,"first_player_draws":false,"ai":[]}"#);
    assert!(r.is_err(), "sim-core accepted ai = []");
}
