// written by an independent reviewer
//! Adversarial tests for C2 (sim-core `Format`, `Engine.formats`, decks.json reading). Written by a reviewer, not the author.
//! Each case loads the real engine from a scratch data directory: every file of data/ symlinked, decks.json rewritten.

use serde_json::{Value, json};
use sim_core::export::{RawDeck, load_decks};
use sim_core::format::{Format, Rules};
use sim_core::run::Engine;
use std::path::{Path, PathBuf};

fn real_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data")
}

/// a data dir whose decks.json is `edit(real decks.json)`
fn engine_with(name: &str, edit: impl FnOnce(&mut Value)) -> Result<Engine, String> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("adv_cali2_{name}"));
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

fn deck_mut<'a>(v: &'a mut Value, group: &str, key: &str) -> &'a mut Value {
    v[group].as_array_mut().unwrap().iter_mut().find(|d| d["key"] == key).unwrap()
}

#[test]
fn the_real_data_loads_with_both_formats() {
    let e = engine_with("real", |_| {}).unwrap();
    assert_eq!(e.formats.keys().collect::<Vec<_>>(), ["cali", "commander"]);
    assert_eq!(e.format_of("arzakon").key, "cali");
    assert_eq!(e.format_of("arzakon").rules(), Rules { life: 30, first_player_draws: false });
    assert_eq!(e.format_of("yshtola").rules(), Rules::COMMANDER);
    assert_eq!(e.format_of("no-such-deck").key, "commander");
    assert!(e.admits("arzakon", "commander").is_err());
    assert!(e.admits("arzakon", "cali").is_ok());
    assert!(e.admits("yshtola", "commander").is_ok());
    // a key that is no deck is admitted; a table that is no format is not
    assert!(e.admits("nobody", "cali").is_ok());
    assert!(e.admits("yshtola", "nowhere").is_err());
    assert!(e.admits("nobody", "nowhere").is_err());
}

#[test]
fn an_old_decks_json_without_formats_or_format_reads_as_commander() {
    let e = engine_with("old", |v| {
        v.as_object_mut().unwrap().remove("formats");
        deck_mut(v, "mine", "arzakon").as_object_mut().unwrap().remove("format");
    })
    .unwrap();
    assert_eq!(e.formats.keys().collect::<Vec<_>>(), ["commander"]);
    assert_eq!(e.format_of("arzakon").key, "commander");
    assert!(e.admits("arzakon", "commander").is_ok());
}

#[test]
fn a_deck_naming_a_format_the_file_lacks_is_a_load_error() {
    let e = engine_with("lacks", |v| {
        v.as_object_mut().unwrap().remove("formats");
    });
    assert!(e.err().unwrap().contains("unknown format \"cali\""));
    let e = engine_with("typo", |v| {
        deck_mut(v, "mine", "arzakon")["format"] = json!("calli");
    });
    assert!(e.err().unwrap().contains("unknown format \"calli\""));
    // `"format": null` is the same as absent
    let e = engine_with("null", |v| deck_mut(v, "mine", "arzakon")["format"] = Value::Null).unwrap();
    assert_eq!(e.format_of("arzakon").key, "commander");
}

#[test]
fn a_pool_deck_with_a_format_is_read_and_refused_at_a_commander_table() {
    let e = engine_with("pool_format", |v| deck_mut(v, "pool", "dack-fayden-white-stax")["format"] = json!("cali"))
        .unwrap();
    assert_eq!(e.format_of("dack-fayden-white-stax").key, "cali");
    assert!(e.admits("dack-fayden-white-stax", "commander").is_err());
}

#[test]
fn bad_format_records_are_refused_at_load() {
    let bad = |name: &str, f: &dyn Fn(&mut Value)| {
        engine_with(name, |v| f(&mut v["formats"])).err().unwrap_or_else(|| panic!("{name} loaded"))
    };
    assert!(bad("seats3", &|f| f["cali"]["seats"] = json!(3)).contains("seats must be 2 or 4"));
    let _ = bad("seats_neg", &|f| f["cali"]["seats"] = json!(-2));
    assert!(bad("life0", &|f| f["cali"]["life"] = json!(0)).contains("life must be at least 1"));
    let _ = bad("life_float", &|f| f["cali"]["life"] = json!(30.5));
    assert!(bad("empty_list", &|f| f["cali"]["banned"] = json!([])).contains("non-empty banned list"));
    assert!(
        bad("no_list", &|f| {
            f["cali"].as_object_mut().unwrap().remove("banned");
        })
        .contains("non-empty banned list")
    );
    assert!(
        bad("dup", &|f| f["cali"]["banned"].as_array_mut().unwrap().push(json!("balance"))).contains("listed twice")
    );
    assert!(
        bad("slashes", &|f| f["cali"]["banned"].as_array_mut().unwrap().push(json!("A // B"))).contains("front-face")
    );
    assert!(bad("blank", &|f| f["cali"]["banned"].as_array_mut().unwrap().push(json!("  "))).contains("front-face"));
    assert!(bad("scry_list", &|f| f["cali"]["bans"] = json!("scryfall")).contains("needs bans"));
    assert!(bad("bans_word", &|f| f["cali"]["bans"] = json!("everything")).contains("unknown variant"));
    assert!(bad("unknown_field", &|f| f["cali"]["ai"] = json!(1)).contains("unknown field"));
    assert!(bad("cmd_life", &|f| f["commander"]["life"] = json!(30)).contains("must stay"));
    assert!(bad("cmd_seats", &|f| f["commander"]["seats"] = json!(2)).contains("must stay"));
    assert!(bad("cmd_draw", &|f| f["commander"]["first_player_draws"] = json!(false)).contains("must stay"));
    assert!(
        bad("cmd_list", &|f| {
            f["commander"]["bans"] = json!("list");
            f["commander"]["banned"] = json!(["Sol Ring"]);
        })
        .contains("must stay")
    );
    // formats as an array, a string
    assert!(engine_with("arr", |v| v["formats"] = json!([])).is_err());
    assert!(engine_with("str", |v| v["formats"] = json!("x")).is_err());
}

/// a Commander deck at a Cali table: names compare by front face and lower case; a back-face name is not looked at here
#[test]
fn admits_compares_front_faces_only() {
    let f = load_decks(&real_data().join("decks.json")).unwrap();
    let mut cali = f.formats["cali"].clone();
    cali.key = "cali".into();
    let mut cmd = Format::commander();
    cmd.key = "commander".into();
    let deck = |cards: &[&str]| RawDeck {
        tier: None,
        key: "d".into(),
        name: "d".into(),
        commander: "X".into(),
        commander2: None,
        ident: None,
        tiers: vec![],
        format: None,
        cards: cards.iter().map(|c| c.to_string()).collect(),
    };
    assert!(cali.admits(&deck(&["Sol Ring", "Mox Jet // x"]), &cmd).is_err());
    assert!(cali.admits(&deck(&["MOX JET"]), &cmd).is_err());
    assert!(cali.admits(&deck(&["  Mox Jet  "]), &cmd).is_err());
    let e = cali.admits(&deck(&["Mox Jet", "Mox Jet", "Balance"]), &cmd).unwrap_err();
    assert_eq!(e.len(), 1);
    assert!(e[0].contains("Mox Jet, Balance"), "{e:?}");
    assert!(cali.admits(&deck(&["Mox Jetty", "Balance of Power"]), &cmd).is_ok());
    // an empty deck, and a deck of the Cali format holding banned cards (its own table admits it: cardgen checks the list)
    assert!(cali.admits(&deck(&[]), &cmd).is_ok());
    let mut own = deck(&["Mox Jet"]);
    own.format = Some("cali".into());
    assert!(cali.admits(&own, &cali).is_ok());
}
