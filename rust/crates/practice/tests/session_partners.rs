//! Partner decks in practice mode. No deck of JD's in `data/my_decks.toml` has two commanders since the `yoshimaru` key
//! left on 2026-11-24 (the earlier partner decks, `thrasios` and `yoshimaru`, are retired as keys), so the tests that
//! seated a person on a Partner deck (the catalog entry with both commanders, and a person casting either commander from the
//! command zone and paying its own tax) are gone with them; what stays is the deck file line that names two commanders and
//! the list of decks a person can play.

use serde_json::Value;

#[test]
fn the_deck_file_line_with_two_commanders() {
    assert_eq!(
        practice::catalog::display_name("x\n**Commanders (2).** Tymna the Weaver, Thrasios, Triton Hero\n").unwrap(),
        "Tymna the Weaver and Thrasios"
    );
}

#[test]
fn the_list_of_playable_decks_is_every_deck_of_decks_json() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let d: Value = serde_json::from_str(&std::fs::read_to_string(data.join("decks.json")).unwrap()).unwrap();
    let mut mine: Vec<&str> = d["mine"].as_array().unwrap().iter().map(|x| x["key"].as_str().unwrap()).collect();
    mine.sort();
    let mut listed = practice::session::MY_DECKS.to_vec();
    listed.sort();
    assert_eq!(listed, mine);
    let mut cat = practice::catalog::MY_DECKS.to_vec();
    cat.sort();
    assert_eq!(cat, mine);
}

#[test]
fn the_yoshimaru_guide_line_names_both_commanders() {
    // two commanders separated by a semicolon, a comma inside the first one's name
    assert_eq!(
        practice::catalog::display_name("x\n**Commanders (2).** Yoshimaru, Ever Faithful; Tymna the Weaver\n").unwrap(),
        "Yoshimaru and Tymna the Weaver"
    );
}
