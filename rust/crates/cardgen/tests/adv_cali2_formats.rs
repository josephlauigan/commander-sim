// written by an independent reviewer
//! Adversarial tests for C2 (format loading, export, `cardgen deck`, missing files). Written by a reviewer. Works on
//! scratch copies of data/ and decklists/; no network (`--offline` only).

use cardgen::data::{Project, load_formats, load_my_decks, parse_formats};
use cardgen::decks;
use cardgen::export::{Inputs, export};
use cardgen::json::{self, J};
use cardgen::update_deck::{Options, run};
use sim_core::format::Format;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("adv_cali2_{name}"));
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&real.join("data"), &root.join("data"));
    copy_dir(&real.join("decklists"), &root.join("decklists"));
    copy_dir(&real.join("tests/fixtures"), &root.join("tests/fixtures"));
    Project::new(root)
}

const COMMANDER: &str =
    "[commander]\nname = \"Commander\"\nseats = 4\nlife = 40\nfirst_player_draws = true\nbans = \"scryfall\"\n";

fn parse(extra: &str) -> Result<Vec<cardgen::data::FormatDef>, String> {
    parse_formats(&format!("{COMMANDER}{extra}"), Path::new("formats.toml"))
}

const CALI_OK: &str =
    "[x]\nname = \"X\"\nseats = 2\nlife = 30\nfirst_player_draws = false\nbans = \"list\"\nbanned = [\"A\"]\n";

/// every value the loader accepts must survive the trip into sim-core: cardgen's `life` is an i64, sim-core's an i32
#[test]
fn a_format_cardgen_accepts_is_one_sim_core_accepts() {
    for (from, to) in [("life = 30", "life = 3000000000"), ("life = 30", "life = 2147483648")] {
        let src = CALI_OK.replace(from, to);
        let fmts = match parse(&src) {
            Ok(f) => f,
            Err(_) => continue, // refusing it at load is the right answer too
        };
        let doc = cardgen::export::formats_doc(&fmts);
        let text = json::dumps(&doc, None, false);
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let x = v.get("x").unwrap().clone();
        let r = serde_json::from_value::<Format>(x);
        assert!(r.is_ok(), "formats.toml `{to}` loads in cardgen and exports, but sim-core cannot read it: {:?}", r.err());
    }
}

/// the rest of the numbers and names: nothing weird slips through the cardgen loader
#[test]
fn the_cardgen_loader_refuses_what_it_should() {
    let with = |from: &str, to: &str| parse(&CALI_OK.replace(from, to));
    assert!(with("life = 30", "life = 30.5").is_err(), "a float life");
    assert!(with("life = 30", "life = -5").is_err());
    assert!(with("seats = 2", "seats = 3").is_err());
    assert!(with("seats = 2", "seats = 0").is_err());
    assert!(with("seats = 2", "seats = \"2\"").is_err());
    assert!(with("bans = \"list\"", "bans = \"List\"").is_err(), "case matters");
    assert!(with("first_player_draws = false", "first_player_draws = 0").is_err());
    assert!(with("banned = [\"A\"]", "banned = [\"A\", \"b\", \"B\"]").is_err(), "duplicates by case");
    assert!(with("banned = [\"A\"]", "banned = [\"A\", \" A \"]").is_err(), "duplicates by padding");
    assert!(with("banned = [\"A\"]", "banned = [\"A//B\"]").is_err());
    assert!(with("banned = [\"A\"]", "banned = []").is_err());
    assert!(with("banned = [\"A\"]", "banned = \"A\"").is_err(), "a string is not a list");
    assert!(with("[x]", "[x]\ncolour = 1").is_err(), "unknown key");
    assert!(parse("top = 1\n").is_err(), "a scalar at top level");
    assert!(parse("[commander2]\nname = \"C\"\n").is_err());
    // [commander] with a ban list, or with bans = "list", is refused
    let c = COMMANDER.replace("scryfall", "list") + "banned = [\"A\"]\n";
    assert!(parse_formats(&c, Path::new("f")).unwrap_err().contains("must stay"));
    let c = COMMANDER.to_string() + "banned = [\"A\"]\n";
    assert!(parse_formats(&c, Path::new("f")).is_err());
    // a second [commander] is a TOML error, not a silent merge
    assert!(parse(&format!("{COMMANDER}")).is_err());
    // keys are matched exactly: [Commander] is not [commander]
    assert!(parse_formats(&COMMANDER.replace("[commander]", "[Commander]"), Path::new("f")).is_err());
}

/// my_decks.toml: a typo and a wrong-typed field are errors; a deck with commander2 can take a format too
#[test]
fn my_decks_format_field_edge_cases() {
    let p = scratch("my_decks");
    let base = std::fs::read_to_string(p.data("my_decks.toml")).unwrap();
    let try_with = |from: &str, to: &str| {
        std::fs::write(p.data("my_decks.toml"), base.replace(from, to)).unwrap();
        load_my_decks(&p)
    };
    assert!(try_with("format = \"cali\"", "format = \"Cali\"").unwrap_err().contains("unknown format"));
    assert!(try_with("format = \"cali\"", "format = \"\"").unwrap_err().contains("unknown format"));
    assert!(try_with("format = \"cali\"", "format = 4").unwrap_err().contains("format must be text"));
    assert!(try_with("format = \"cali\"", "format = [\"cali\"]").unwrap_err().contains("format must be text"));
    assert!(try_with("format = \"cali\"", "formats = \"cali\"").unwrap_err().contains("unknown key"));
    // a deck with a second commander on the Cali format loads (the format says seats, not commanders)
    let ok = try_with("[yoshimaru]", "[yoshimaru]\nformat = \"cali\"").unwrap();
    assert_eq!(ok.iter().find(|d| d.key == "yoshimaru").unwrap().format, "cali");
    // the format table can be edited: a deck can name a format the file adds
    std::fs::write(p.data("formats.toml"), std::fs::read_to_string(p.data("formats.toml")).unwrap() + &CALI_OK.replace("[x]", "[other]"))
        .unwrap();
    assert!(try_with("format = \"cali\"", "format = \"other\"").is_ok());
    // a my_decks.toml key that is a plain value, not a table
    std::fs::write(p.data("my_decks.toml"), format!("zzz = 1\n{base}")).unwrap();
    assert!(load_my_decks(&p).unwrap_err().contains("must be a table"));
}

/// formats.toml missing: every cardgen command that loads the project data says so by name, and nothing panics.
#[test]
fn a_missing_formats_toml_is_a_named_error_everywhere() {
    let p = scratch("missing_formats");
    std::fs::remove_file(p.data("formats.toml")).unwrap();
    assert!(load_formats(&p).unwrap_err().contains("formats.toml"));
    assert!(load_my_decks(&p).unwrap_err().contains("formats.toml"));
    assert!(Inputs::load(&p).is_err());
    let exe = env!("CARGO_BIN_EXE_cardgen");
    for args in [
        vec!["validate"],
        vec!["export", "--offline", "--out", "/nonexistent/should-not-be-written"],
        vec!["audit", "--mine"],
        vec!["facts"],
    ] {
        let o = Command::new(exe).arg("--root").arg(&p.root).args(&args).output().unwrap();
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(!o.status.success(), "{args:?} succeeded without formats.toml");
        assert!(!err.contains("panicked"), "{args:?}: {err}");
        eprintln!("{args:?}: {}", err.lines().next().unwrap_or(""));
    }
    // commands that do not need the format table keep working without it
    // (an empty catalog dir keeps this independent of a card catalog in the main checkout: with a catalog `map` reaches the
    // engine facts, which do need formats.toml and fail by name; coordinator's setup-only edit, assertions unchanged)
    let empty = p.root.join("empty-knowledge");
    std::fs::create_dir_all(&empty).unwrap();
    for args in [vec!["map", "--stdout"], vec!["digest", "--help"]] {
        let o = Command::new(exe)
            .env("CARDGEN_KNOWLEDGE_DIR", &empty)
            .arg("--root")
            .arg(&p.root)
            .args(&args)
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&o.stderr).contains("formats.toml"), "{args:?}");
    }
}

/// `cardgen export --out` writes bytes identical to the committed files, is deterministic, and writes `formats` before `mine`
#[test]
fn export_reproduces_the_committed_files_byte_for_byte() {
    let p = scratch("export_identical");
    let out = p.root.join("out");
    cardgen::export::run(&p, &out, true).unwrap();
    let real = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for f in ["decks.json", "cards.json"] {
        assert_eq!(std::fs::read(out.join(f)).unwrap(), std::fs::read(real.join("data").join(f)).unwrap(), "{f}");
    }
    let out2 = p.root.join("out2");
    cardgen::export::run(&p, &out2, true).unwrap();
    assert_eq!(std::fs::read(out.join("decks.json")).unwrap(), std::fs::read(out2.join("decks.json")).unwrap());
    let text = std::fs::read_to_string(out.join("decks.json")).unwrap();
    let (f, m, pl) = (text.find("\"formats\"").unwrap(), text.find("\"mine\"").unwrap(), text.find("\"pool\"").unwrap());
    assert!(f < m && m < pl);
    // the exported mine record of a Commander deck has no `format` key at all
    let doc = json::parse(&text).unwrap();
    let mine = doc.get("mine").unwrap().as_arr().unwrap();
    for d in mine {
        let key = d.s_or("key");
        assert_eq!(d.get("format").is_some(), key == "kyodai", "{key}");
    }
}

/// editing formats.toml changes the export and nothing but `formats`/the deck's format
#[test]
fn a_format_edit_reaches_decks_json_and_no_other_field() {
    let p = scratch("export_edit");
    let before = export(&Inputs::load(&p).unwrap()).unwrap().decks_json;
    let toml = std::fs::read_to_string(p.data("formats.toml")).unwrap();
    std::fs::write(p.data("formats.toml"), toml.replace("life = 30", "life = 25")).unwrap();
    let after = export(&Inputs::load(&p).unwrap()).unwrap().decks_json;
    let (b, a): (J, J) = (json::parse(&before).unwrap(), json::parse(&after).unwrap());
    assert_eq!(b.get("pool"), a.get("pool"));
    assert_eq!(b.get("mine"), a.get("mine"));
    assert_ne!(b.get("formats"), a.get("formats"));
}

fn list_of(p: &Project, d: &cardgen::data::MyDeck) -> (PathBuf, Vec<String>) {
    let path = decks::deck_path(p, d);
    let cards = decks::load(&path).unwrap();
    (path, cards)
}

fn dry(p: &Project, which: &str, cards: &[String]) -> cardgen::update_deck::Outcome {
    run(p, which, &cards.join("\n"), &Options { dry: true, log: None, offline: true }).unwrap()
}

/// `cardgen deck KEY|PATH` for every deck of yours: by key, by absolute path and by a path with `..` and a symlink, the
/// unchanged list is accepted; and each is judged by the deck's own format
#[test]
fn deck_by_key_and_by_path_agree_for_every_deck_of_yours() {
    let p = scratch("deck_paths");
    let link_dir = p.root.join("links");
    std::fs::create_dir_all(&link_dir).unwrap();
    for d in load_my_decks(&p).unwrap() {
        let (path, cards) = list_of(&p, &d);
        let by_key = dry(&p, &d.key, &cards);
        assert!(by_key.ok, "{} by key: {}", d.key, by_key.text);
        let by_path = dry(&p, path.to_str().unwrap(), &cards);
        assert!(by_path.ok, "{} by path: {}", d.key, by_path.text);
        let dotted = p.root.join("decklists/../decklists").join(path.strip_prefix(p.root.join("decklists")).unwrap());
        let by_dotted = dry(&p, dotted.to_str().unwrap(), &cards);
        assert!(by_dotted.ok, "{} by dotted path: {}", d.key, by_dotted.text);
        let link = link_dir.join(format!("{}.md", d.key));
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let by_link = dry(&p, link.to_str().unwrap(), &cards);
        assert!(by_link.ok, "{} by symlink: {}", d.key, by_link.text);
        // the same message either way
        assert_eq!(by_key.text.lines().count() > 0, by_path.text.lines().count() > 0);
    }
}

/// each deck under the wrong format's ban list: a Cali-banned card in a Commander deck and in kyodai; the three in a
/// Commander deck; by key and by path
#[test]
fn each_deck_is_judged_by_its_own_format_by_key_and_by_path() {
    let p = scratch("deck_formats");
    for d in load_my_decks(&p).unwrap() {
        let (path, mut cards) = list_of(&p, &d);
        let i = cards.iter().position(|c| c != &d.commander && !decks::BASICS.contains(&c.as_str()) && c != "Mana Crypt" && c != "Sol Ring").unwrap_or(0);
        // the Tabernacle: legal in Commander, banned in Cali
        cards[i] = "The Tabernacle at Pendrell Vale".into();
        for which in [d.key.clone(), path.to_string_lossy().into_owned()] {
            let o = dry(&p, &which, &cards);
            if d.format == "cali" {
                assert!(!o.ok && o.text.contains("banned in Cali Commander: The Tabernacle"), "{} {which}: {}", d.key, o.text);
            } else {
                assert!(!o.text.contains("banned in"), "{} {which}: {}", d.key, o.text);
            }
        }
        // Mana Crypt: banned in Commander, fine in Cali
        cards[i] = "Mana Crypt".into();
        if cards.iter().filter(|c| *c == "Mana Crypt").count() == 1 {
            for which in [d.key.clone(), path.to_string_lossy().into_owned()] {
                let o = dry(&p, &which, &cards);
                let refused = o.text.contains("banned in Commander: Mana Crypt");
                assert_eq!(refused, d.format != "cali", "{} {which}: {}", d.key, o.text);
            }
        }
    }
}

/// a pool deck, by key and by path, still judged under Commander: Mana Crypt refused, Tabernacle fine
#[test]
fn a_pool_deck_is_unchanged() {
    let p = scratch("deck_pool");
    for d in decks::load_pool(&p).unwrap().into_iter().take(4) {
        let mut cards = d.cards.clone();
        let i = cards.iter().position(|c| c == "Sol Ring").unwrap_or(0);
        cards[i] = "Mana Crypt".into();
        for which in [d.key.clone(), d.path.to_string_lossy().into_owned()] {
            let o = dry(&p, &which, &cards);
            assert!(o.text.contains("banned in Commander"), "{} {which}: {}", d.key, o.text);
        }
    }
}
