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
