//! C4, the data side in sim-core: the `ai` block of decks.json formats. The real file carries Cali's profile and
//! Commander's default; a bad block is refused when the engine loads. (cardgen's reader refuses the same things in
//! `cardgen/tests/cali_c4_ai_format.rs`.) Each case loads the real engine from a scratch data directory.

use serde_json::{Value, json};
use sim_core::format::{GoRules, TableAi};
use sim_core::run::Engine;
use std::path::{Path, PathBuf};

fn real_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data")
}

fn engine_with(name: &str, edit: impl FnOnce(&mut Value)) -> Result<Engine, String> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cali_c4_ai_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for e in std::fs::read_dir(real_data()).unwrap() {
        let e = e.unwrap();
        if e.file_name() != "decks.json" {
            std::os::unix::fs::symlink(e.path(), dir.join(e.file_name())).unwrap();
        }
    }
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(real_data().join("decks.json")).unwrap()).unwrap();
    edit(&mut v);
    std::fs::write(dir.join("decks.json"), serde_json::to_string(&v).unwrap()).unwrap();
    Engine::load(Path::new(&dir))
}

#[test]
fn the_real_data_carries_cali_s_profile_and_commander_s_default() {
    let e = engine_with("real", |_| {}).unwrap();
    assert_eq!(e.formats["commander"].ai, TableAi::COMMANDER);
    // duel is derived from the seats, not stored
    assert_eq!(e.formats["cali"].ai, TableAi { learned_eval: false, go_rules: GoRules::All, duel: true });
    // the stored block holds only the two data fields
    let v: Value = serde_json::from_str(&std::fs::read_to_string(real_data().join("decks.json")).unwrap()).unwrap();
    assert_eq!(v["formats"]["cali"]["ai"], json!({"learned_eval": false, "go_rules": "all"}));
    assert_eq!(v["formats"]["commander"]["ai"], json!({"learned_eval": true, "go_rules": "t6"}));
}

#[test]
fn a_decks_json_without_ai_reads_every_format_as_commander_plus_the_derived_duel() {
    let e = engine_with("no_ai", |v| {
        for f in v["formats"].as_object_mut().unwrap().values_mut() {
            f.as_object_mut().unwrap().remove("ai");
        }
    })
    .unwrap();
    assert_eq!(e.formats["commander"].ai, TableAi::COMMANDER);
    assert_eq!(e.formats["cali"].ai, TableAi { duel: true, ..TableAi::COMMANDER });
}

#[test]
fn a_bad_ai_block_is_refused_at_load() {
    let bad = |name: &str, f: &dyn Fn(&mut Value)| {
        engine_with(name, |v| f(v)).err().unwrap_or_else(|| panic!("{name}: loaded"))
    };
    assert!(bad("unknown", &|v| v["formats"]["cali"]["ai"]["speed"] = json!(1)).contains("unknown field"));
    assert!(bad("duel", &|v| v["formats"]["cali"]["ai"]["duel"] = json!(true)).contains("unknown field"));
    assert!(
        bad("go_word", &|v| v["formats"]["cali"]["ai"]["go_rules"] = json!("everyone")).contains("unknown variant")
    );
    assert!(bad("go_type", &|v| v["formats"]["cali"]["ai"]["go_rules"] = json!(true)).len() > 0);
    assert!(bad("le_type", &|v| v["formats"]["cali"]["ai"]["learned_eval"] = json!("no")).contains("invalid type"));
    assert!(bad("ai_type", &|v| v["formats"]["cali"]["ai"] = json!(1)).len() > 0);
    // [commander] cannot change its AI profile, field by field
    for (k, val) in [("learned_eval", json!(false)), ("go_rules", json!("all"))] {
        let e = bad(&format!("cmd_{k}"), &|v| v["formats"]["commander"]["ai"][k] = val.clone());
        assert!(e.contains("must stay 4 seats, 40 life") && e.contains("AI profile"), "{k}: {e}");
    }
}
