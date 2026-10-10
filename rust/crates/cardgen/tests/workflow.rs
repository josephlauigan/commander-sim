//! The things the README tells you to do, done on a scratch copy of the repository's data and deck lists: add a card by
//! hand, add an override, change a deck, audit. Nothing here touches the network (`--offline` paths only) or the real
//! files.

use cardgen::data::Project;
use cardgen::export::{Inputs, export};
use cardgen::json::{self, J};
use cardgen::update_deck::{Options, run};
use std::path::{Path, PathBuf};

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

/// a scratch repository: data/, decklists/ and tests/fixtures/
fn scratch(name: &str) -> Project {
    let real = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&real.join("data"), &root.join("data"));
    copy_dir(&real.join("decklists"), &root.join("decklists"));
    copy_dir(&real.join("tests/fixtures"), &root.join("tests/fixtures"));
    Project::new(root)
}

fn card<'a>(cards: &'a J, name: &str) -> &'a J {
    cards.get("cards").unwrap().as_arr().unwrap().iter().find(|c| c.s_or("name") == name).unwrap_or_else(|| panic!("no card {name}"))
}

fn append(p: &Project, file: &str, text: &str) {
    let path = p.data(file);
    let mut s = std::fs::read_to_string(&path).unwrap();
    s.push_str(text);
    std::fs::write(path, s).unwrap();
}

#[test]
fn a_hand_tagged_card_appears_in_cards_json() {
    let p = scratch("hand_card");
    append(&p, "carddb.txt", "Test Widget|A|2U|rock=2:C sac\n");
    let e = export(&Inputs::load(&p).unwrap()).unwrap();
    let cards = json::parse(&e.cards_json).unwrap();
    let c = card(&cards, "Test Widget");
    assert_eq!(c.s("source"), Some("manual"));
    assert_eq!(c.get("generic"), Some(&J::Int(2)));
    assert_eq!(c.s("pips"), Some("U"));
    assert_eq!(c.get("tags").unwrap().s("rock"), Some("2:C"));
    assert_eq!(c.get("tags").unwrap().get("sac"), Some(&J::Bool(true)));
}

#[test]
fn an_override_replaces_a_cards_abilities_and_adds_hints() {
    let p = scratch("override");
    // Cancel is built from its Oracle text (Lightning Bolt would be hand-tagged)
    let before = json::parse(&export(&Inputs::load(&p).unwrap()).unwrap().cards_json).unwrap();
    assert_eq!(card(&before, "Cancel").s("source"), Some("scryfall+dsl"));
    append(
        &p,
        "card_overrides.toml",
        "\n[\"Cancel\"]\ntags = \"burn\"\ndsl = [{type = \"spell\", effects = [{do = \"damage\", n = 2, to = {sel = \"any_target\"}}]}]\nstatus = \"Full\"\nnote = \"test\"\n",
    );
    let after = json::parse(&export(&Inputs::load(&p).unwrap()).unwrap().cards_json).unwrap();
    let c = card(&after, "Cancel");
    assert_eq!(c.s("source"), Some("pool"));
    let tags = c.get("tags").unwrap();
    assert_eq!(tags.get("burn"), Some(&J::Bool(true)));
    assert_eq!(tags.s("rem"), Some("dmg2")); // hint from the ability
    assert_eq!(tags.get("face"), Some(&J::Bool(true)));
    assert_eq!(c.get("dsl").unwrap().as_arr().unwrap().len(), 1);
}

#[test]
fn a_card_without_scryfall_data_is_an_error_offline() {
    let p = scratch("missing");
    let path = p.root.join("decklists/pool/t1-high-b2-low-b3");
    let f = std::fs::read_dir(&path).unwrap().next().unwrap().unwrap().path();
    let text = std::fs::read_to_string(&f).unwrap().replacen("```\n", "```\n1 Zzz Not A Card\n", 1);
    std::fs::write(&f, text).unwrap();
    let inp = Inputs::load(&p).unwrap();
    assert_eq!(inp.uncached(), vec!["Zzz Not A Card".to_string()]);
    let err = export(&inp).err().expect("an error");
    assert!(err.contains("Zzz Not A Card"), "{err}");
}

#[test]
fn updating_a_deck_rewrites_its_file_and_the_guard_fixture() {
    let p = scratch("update");
    let inp = Inputs::load(&p).unwrap();
    let jodah = inp.my.iter().find(|d| d.key == "jodah").unwrap().clone();
    let path = cardgen::decks::deck_path(&p, &jodah);
    let mut list = cardgen::decks::load(&path).unwrap();
    let out = list.iter().position(|c| c == "Arcane Signet").unwrap();
    list[out] = "Sol Ring".to_string();
    let text = list.join("\n");
    // dry run: reports, writes nothing
    let before = std::fs::read_to_string(&path).unwrap();
    let o = run(&p, "jodah", &text, &Options { dry: true, log: None, offline: true }).unwrap();
    assert!(o.ok);
    assert!(o.text.contains("1 out, 1 in"), "{}", o.text);
    assert!(o.text.contains("out: Arcane Signet") && o.text.contains("in:  Sol Ring"), "{}", o.text);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    // for real, with a log line
    let o = run(&p, "jodah", &text, &Options { dry: false, log: Some("Test.".into()), offline: true }).unwrap();
    assert!(o.ok, "{}", o.text);
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("1 Sol Ring\n") && !after.contains("1 Arcane Signet\n"));
    assert!(after.contains("Out: Arcane Signet. In: Sol Ring. Test."));
    let new_list = cardgen::decks::load(&path).unwrap();
    assert_eq!(new_list.len(), 100);
    let fixture = std::fs::read_to_string(p.root.join("tests/fixtures/my_decks_parsed.json")).unwrap();
    assert!(fixture.contains("\"Sol Ring\""));
    // a bad list stops before any file is touched
    let o = run(&p, "jodah", &format!("{text}\n1 Sol Ring"), &Options { dry: false, log: None, offline: true }).unwrap();
    assert!(!o.ok);
    assert!(o.text.contains("101 cards, expected 100") && o.text.contains("not singleton: Sol Ring"), "{}", o.text);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
}

#[test]
fn the_audit_runs_on_the_scratch_data() {
    let p = scratch("audit");
    let inp = Inputs::load(&p).unwrap();
    let built = inp.build();
    let au = cardgen::audit::Audit::new(&inp, &built.cards).unwrap();
    let rows = cardgen::audit::audit_pool(&au, &inp.pool);
    assert_eq!(rows.len(), 25);
    assert_eq!(au.card_status("Island", false).0, "Full");
    let (s, _) = au.card_status("Sol Ring", true);
    assert_eq!(s, "Full");
    let s = cardgen::audit::summary(&rows);
    assert!(s.starts_with("deck "), "{s}");
    assert!(s.contains("all decks (card-slots, unique per deck): Full "));
    let hooks = cardgen::audit::hooks_report(&inp, &built.cards, false).unwrap();
    assert!(hooks.starts_with("Rust card code vs the Python hooks"), "{hooks}");
}
