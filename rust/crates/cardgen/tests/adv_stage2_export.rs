//! Metadata fallback, reproducibility and the precise generated-data scope of stage2-fixes.
use cardgen::cd::Cd;
use cardgen::data::Project;
use cardgen::export::{Inputs, card_record, export};
use cardgen::json::{self, J};
use cardgen::scryfall::{Cache, key};
use std::path::PathBuf;

fn project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

#[test]
fn fallback_handles_case_split_and_adventure_names_and_preserves_explicit_metadata() {
    let mut cache = Cache::default();
    for (name, ident) in [("Front", "B"), ("Fire // Ice", "RU"), ("Beanstalk Giant // Fertile Footsteps", "G")] {
        cache.insert(
            key(name),
            json::parse(&format!(r#"{{"game_changer":true,"color_identity":["{ident}"]}}"#)).unwrap(),
        );
    }
    for (name, ident) in [("fRoNt // Back", "B"), ("FIRE // ICE", "RU"), ("Beanstalk Giant // Fertile Footsteps", "G")]
    {
        let cd = Cd::new(name, "C", "1", "");
        let r = card_record(&cd, &cache, None);
        assert_eq!(r.get("identity"), Some(&J::from(ident)), "{name}");
        assert_eq!(r.get("game_changer"), Some(&J::from(true)), "{name}");
    }
    let mut cd = Cd::new("FRONT", "C", "1", "");
    cd.identity = Some("W".into());
    cd.game_changer = Some(false);
    let r = card_record(&cd, &cache, None);
    assert_eq!(r.get("identity"), Some(&J::from("W")));
    assert_eq!(r.get("game_changer"), Some(&J::from(false)));
    let unknown = card_record(&Cd::new("Unknown fixture", "C", "1", ""), &cache, None);
    assert_eq!(unknown.get("identity"), Some(&J::Null));
    assert_eq!(unknown.get("game_changer"), Some(&J::Null));
}

#[test]
fn export_is_byte_identical_twice_and_matches_committed_files_without_writing_them() {
    let p = project();
    let inp = Inputs::load(&p).unwrap();
    let a = export(&inp).unwrap();
    let b = export(&inp).unwrap();
    assert_eq!(a.cards_json, b.cards_json);
    assert_eq!(a.decks_json, b.decks_json);
    assert_eq!(a.cards_json, std::fs::read_to_string(p.data("cards.json")).unwrap());
    assert_eq!(a.decks_json, std::fs::read_to_string(p.data("decks.json")).unwrap());
}

#[test]
fn generated_json_changes_only_metadata_and_the_three_intended_abilities() {
    use serde_json::Value;
    let p = project();
    let old = std::process::Command::new("git")
        .arg("-C")
        .arg(&p.root)
        .args(["show", "origin/main:data/cards.json"])
        .output()
        .unwrap();
    assert!(old.status.success());
    let old: Value = serde_json::from_slice(&old.stdout).unwrap();
    let new: Value = serde_json::from_str(&std::fs::read_to_string(p.data("cards.json")).unwrap()).unwrap();
    assert_eq!(old["cards"].as_array().unwrap().len(), new["cards"].as_array().unwrap().len());
    let mut metadata = 0;
    let mut dsl = vec![];
    for (a, b) in old["cards"].as_array().unwrap().iter().zip(new["cards"].as_array().unwrap()) {
        assert_eq!(a["name"], b["name"]);
        assert_eq!(
            a.as_object().unwrap().keys().collect::<Vec<_>>(),
            b.as_object().unwrap().keys().collect::<Vec<_>>()
        );
        let mut meta_changed = false;
        for (field, v) in a.as_object().unwrap() {
            if v == &b[field] {
                continue;
            }
            match field.as_str() {
                "identity" | "game_changer" => meta_changed = true,
                "dsl" => dsl.push(a["name"].as_str().unwrap()),
                _ => panic!("unexpected generated change: {} .{field}", a["name"]),
            }
        }
        metadata += usize::from(meta_changed);
    }
    assert_eq!(metadata, 353);
    assert_eq!(dsl, ["Atarka, World Render", "Morcant's Loyalist", "Seasoned Hallowblade"]);
    eprintln!("353 metadata records, exactly three DSL changes; no other card fields changed");
}
