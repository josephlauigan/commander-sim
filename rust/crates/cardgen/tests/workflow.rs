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
    cards
        .get("cards")
        .unwrap()
        .as_arr()
        .unwrap()
        .iter()
        .find(|c| c.s_or("name") == name)
        .unwrap_or_else(|| panic!("no card {name}"))
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
    let deck = inp.my.iter().find(|d| d.key == "urdragon").unwrap().clone();
    let path = cardgen::decks::deck_path(&p, &deck);
    let mut list = cardgen::decks::load(&path).unwrap();
    let out = list.iter().position(|c| c == "Arcane Signet").unwrap();
    list[out] = "Prismatic Lens".to_string();
    let text = list.join("\n");
    // dry run: reports, writes nothing
    let before = std::fs::read_to_string(&path).unwrap();
    let o = run(&p, "urdragon", &text, &Options { dry: true, log: None, offline: true }).unwrap();
    assert!(o.ok);
    assert!(o.text.contains("1 out, 1 in"), "{}", o.text);
    assert!(o.text.contains("out: Arcane Signet") && o.text.contains("in:  Prismatic Lens"), "{}", o.text);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    // for real, with a log line
    let o = run(&p, "urdragon", &text, &Options { dry: false, log: Some("Test.".into()), offline: true }).unwrap();
    assert!(o.ok, "{}", o.text);
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("1 Prismatic Lens\n") && !after.contains("1 Arcane Signet\n"));
    // the log line goes to the changelog, under the deck's title, not into the deck file
    assert!(!after.contains("Out: Arcane Signet. In: Prismatic Lens. Test."));
    let log = std::fs::read_to_string(p.decklists().join("CHANGELOG.md")).unwrap();
    let title = before.lines().find_map(|l| l.strip_prefix("# ")).unwrap();
    let section =
        log.split(&format!("## {title}\n")).nth(1).expect("the deck's section").split("\n## ").next().unwrap();
    let changes = section.split("### Changes\n\n").nth(1).expect("a Changes list");
    assert!(
        changes.starts_with("- **")
            && changes.lines().next().unwrap().ends_with("Out: Arcane Signet. In: Prismatic Lens. Test."),
        "{section}"
    );
    let new_list = cardgen::decks::load(&path).unwrap();
    assert_eq!(new_list.len(), 100);
    let fixture = std::fs::read_to_string(p.root.join("tests/fixtures/my_decks_parsed.json")).unwrap();
    assert!(fixture.contains("\"Prismatic Lens\""));
    // a bad list stops before any file is touched
    let o =
        run(&p, "urdragon", &format!("{text}\n1 Prismatic Lens"), &Options { dry: false, log: None, offline: true })
            .unwrap();
    assert!(!o.ok);
    assert!(
        o.text.contains("101 cards, expected 100") && o.text.contains("not singleton: Prismatic Lens"),
        "{}",
        o.text
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
}

#[test]
fn the_audit_runs_on_the_scratch_data() {
    let p = scratch("audit");
    let inp = Inputs::load(&p).unwrap();
    let built = inp.build();
    let au = cardgen::audit::Audit::new(&inp, &built.cards).unwrap();
    let rows = cardgen::audit::audit_pool(&au, &inp.pool);
    assert_eq!(rows.len(), 36);
    assert_eq!(au.card_status("Island", false).0, "Full");
    let (s, _) = au.card_status("Sol Ring", true);
    assert_eq!(s, "Full");
    let s = cardgen::audit::summary(&rows);
    assert!(s.starts_with("deck "), "{s}");
    assert!(s.contains("all decks (card-slots, unique per deck): Full "));
    let hooks = cardgen::audit::hooks_report(&inp, &built.cards, false).unwrap();
    assert!(hooks.starts_with("Rust card code vs the Python hooks"), "{hooks}");
}

#[test]
fn a_pool_deck_with_two_commanders_carries_both() {
    let real = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let d = cardgen::decks::load_pool_deck(&real.join("decklists/pool/t6-cedh/kraum-tymna-blue-breach-midrange.md"))
        .unwrap();
    assert_eq!(d.tier, "t6");
    assert_eq!(d.commander, "Kraum, Ludevic's Opus");
    assert_eq!(d.commander2.as_deref(), Some("Tymna the Weaver"));
    // a single commander, and a double-faced one, stay whole
    let d = cardgen::decks::load_pool_deck(&real.join("decklists/pool/t6-cedh/sisay-legends-toolbox.md")).unwrap();
    assert_eq!((d.commander.as_str(), d.commander2), ("Sisay, Weatherlight Captain", None));
    // the exported decks.json lists both for the pair, and the seat's colours are their union
    let decks: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(real.join("data/decks.json")).unwrap()).unwrap();
    let k = decks["pool"].as_array().unwrap().iter().find(|x| x["key"] == "kraum-tymna-blue-breach-midrange").unwrap();
    assert_eq!(k["commander2"], "Tymna the Weaver");
    assert_eq!(k["ident"], "WUBR");
}

#[test]
fn audit_never_overwrites_a_decklist() {
    let p = scratch("audit_md");
    let doc = std::fs::read_dir(p.root.join("decklists/JD"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|f| f.extension().is_some_and(|x| x == "md"))
        .expect("a deck doc in the scratch data");
    let before = std::fs::read_to_string(&doc).unwrap();
    let run = |extra: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_cardgen"))
            .arg("--root")
            .arg(&p.root)
            .arg("audit")
            .args(extra)
            .output()
            .unwrap()
    };
    let out = run(&["--md", doc.to_str().unwrap()]);
    assert!(!out.status.success(), "the decklist path must be refused");
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), before, "the deck doc is untouched");
    // a plain file that is not an audit table is refused too; a new file or an old table is written
    let other = p.root.join("notes.md");
    std::fs::write(&other, "# my notes\n").unwrap();
    assert!(!run(&["--md", other.to_str().unwrap()]).status.success());
    let table = p.root.join("table.md");
    assert!(run(&["--md", table.to_str().unwrap()]).status.success());
    assert!(std::fs::read_to_string(&table).unwrap().starts_with("| Tier | Deck |"));
    assert!(run(&["--md", table.to_str().unwrap()]).status.success(), "an old table can be rewritten");
    // --deck and --mine print to stdout and touch no file
    let key = "urdragon";
    assert!(run(&["--deck", key]).status.success());
    assert!(run(&["--mine"]).status.success());
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), before);
}

/// the cards of one of your decks, as `cardgen deck` takes them
fn list_of(p: &Project, key: &str) -> (PathBuf, Vec<String>) {
    let inp = Inputs::load(p).unwrap();
    let d = inp.my.iter().find(|d| d.key == key).unwrap().clone();
    let path = cardgen::decks::deck_path(p, &d);
    let cards = cardgen::decks::load(&path).unwrap();
    (path, cards)
}

fn dry(p: &Project, which: &str, cards: &[String]) -> cardgen::update_deck::Outcome {
    run(p, which, &cards.join("\n"), &Options { dry: true, log: None, offline: true }).unwrap()
}

#[test]
fn cardgen_deck_checks_a_deck_under_its_own_format() {
    let p = scratch("deck_format");
    // arzakon plays Cali Commander: Mana Crypt, Primeval Titan and Tinker are legal there
    let (path, cards) = list_of(&p, "arzakon");
    for c in ["Mana Crypt", "Primeval Titan", "Tinker"] {
        assert!(cards.iter().any(|x| x == c), "{c}");
    }
    let o = dry(&p, "arzakon", &cards);
    assert!(o.ok, "{}", o.text);
    // the same deck named by its file path takes the same format
    let o = dry(&p, path.to_str().unwrap(), &cards);
    assert!(o.ok, "a path to a deck of yours takes its format: {}", o.text);
    // a card on the Cali list is refused for it: the Tabernacle in for Tinker
    let mut bad = cards.clone();
    let i = bad.iter().position(|c| c == "Tinker").unwrap();
    bad[i] = "The Tabernacle at Pendrell Vale".into();
    let o = dry(&p, "arzakon", &bad);
    assert!(!o.ok);
    assert!(o.text.contains("banned in Cali Commander: The Tabernacle at Pendrell Vale"), "{}", o.text);
    // under Commander the same list fails on exactly the three
    let toml = std::fs::read_to_string(p.data("my_decks.toml")).unwrap();
    std::fs::write(p.data("my_decks.toml"), toml.replace("format = \"cali\"\n", "")).unwrap();
    let o = dry(&p, "arzakon", &cards);
    assert!(!o.ok);
    assert!(
        o.text.contains("banned in Commander: Dockside Extortionist, Mana Crypt, Primeval Titan, Tinker"),
        "{}",
        o.text
    );
}

#[test]
fn a_commander_deck_with_mana_crypt_swapped_in_is_refused() {
    let p = scratch("deck_commander_ban");
    let (_, mut cards) = list_of(&p, "urdragon");
    let o = dry(&p, "urdragon", &cards);
    assert!(o.ok, "{}", o.text);
    let i = cards.iter().position(|c| c == "Sol Ring").expect("Sol Ring in the list");
    cards[i] = "Mana Crypt".into();
    let o = dry(&p, "urdragon", &cards);
    assert!(!o.ok);
    assert!(o.text.contains("banned in Commander: Mana Crypt"), "{}", o.text);
}

#[test]
fn validate_covers_each_of_your_decks_under_its_own_format() {
    let real = Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."));
    let cache = cardgen::scryfall::Cache::load(&real.data("scryfall_cache.json")).unwrap();
    let report = cardgen::decks::validate_all(&real, &cache).unwrap();
    for (d, probs, _) in &report {
        assert!(probs.is_empty(), "{} {}: {probs:?}", d.tier, d.key);
    }
    let mut mine: Vec<(&str, &str)> =
        report.iter().filter(|(d, ..)| d.tier == "mine").map(|(d, ..)| (d.key.as_str(), d.format.as_str())).collect();
    mine.sort();
    assert_eq!(
        mine,
        [
            ("arzakon", "cali"),
            ("karn_gilded", "commander"),
            ("karn_reforged", "commander"),
            ("marchesa", "commander"),
            ("sheoldred", "commander"),
            ("trostani", "commander"),
            ("urdragon", "commander")
        ]
    );
}

#[test]
fn a_typo_or_an_unknown_format_in_my_decks_is_an_error() {
    let p = scratch("my_decks_format");
    let toml = std::fs::read_to_string(p.data("my_decks.toml")).unwrap();
    std::fs::write(p.data("my_decks.toml"), toml.replace("format = \"cali\"", "fromat = \"cali\"")).unwrap();
    let e = cardgen::data::load_my_decks(&p).unwrap_err();
    assert!(e.contains("[arzakon]") && e.contains("unknown key \"fromat\""), "{e}");
    std::fs::write(p.data("my_decks.toml"), toml.replace("format = \"cali\"", "format = \"calli\"")).unwrap();
    let e = cardgen::data::load_my_decks(&p).unwrap_err();
    assert!(e.contains("[arzakon]") && e.contains("unknown format \"calli\""), "{e}");
    std::fs::write(p.data("my_decks.toml"), &toml).unwrap();
    let decks = cardgen::data::load_my_decks(&p).unwrap();
    let fmt: Vec<(&str, &str)> = decks.iter().map(|d| (d.key.as_str(), d.format.as_str())).collect();
    assert!(fmt.contains(&("arzakon", "cali")) && fmt.contains(&("urdragon", "commander")), "{fmt:?}");
}

#[test]
fn export_writes_the_formats_and_only_arzakon_carries_one() {
    let p = scratch("export_formats");
    let e = export(&Inputs::load(&p).unwrap()).unwrap();
    let doc = json::parse(&e.decks_json).unwrap();
    let fmts = doc.get("formats").and_then(|f| f.as_obj()).expect("a formats object");
    assert_eq!(fmts.keys().collect::<Vec<_>>(), ["commander", "cali"]);
    let cali = fmts.get("cali").unwrap();
    assert_eq!(cali.s("name"), Some("Cali Commander"));
    assert_eq!(cali.s("bans"), Some("list"));
    assert_eq!(cali.get("banned").unwrap().as_arr().unwrap().len(), 18);
    assert!(fmts.get("commander").unwrap().get("banned").is_none());
    for m in doc.get("mine").unwrap().as_arr().unwrap() {
        let key = m.s_or("key");
        assert_eq!(m.s("format"), (key == "arzakon").then_some("cali"), "{key}");
    }
    assert!(doc.get("pool").unwrap().as_arr().unwrap().iter().all(|d| d.get("format").is_none()));
}
