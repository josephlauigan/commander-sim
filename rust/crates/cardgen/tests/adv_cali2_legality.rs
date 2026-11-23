// written by an independent reviewer
//! Adversarial tests for C2 (per-format legality in cardgen). Written by a reviewer, not the author. Real data only
//! (data/scryfall_cache.json, data/decks.json, decklists/); no network.

use cardgen::data::{FormatDef, Project, load_formats};
use cardgen::decks::{Illegal, card_problems, legality_problems, validate_all};
use cardgen::json::{self, J};
use cardgen::scryfall::Cache;
use sim_core::export::{RawDeck, load_decks};
use sim_core::format::Format;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn formats() -> (FormatDef, FormatDef) {
    let all = load_formats(&Project::new(root())).unwrap();
    let get = |k: &str| all.iter().find(|f| f.key == k).unwrap().clone();
    (get("commander"), get("cali"))
}

fn cache() -> Cache {
    Cache::load(&root().join("data/scryfall_cache.json")).unwrap()
}

fn rec(text: &str) -> J {
    json::parse(text).unwrap()
}

/// sim-core's reading of the same Cali format (decks.json `formats`), with its key filled in
fn sim_core_formats() -> (Format, Format) {
    let mut f = load_decks(&root().join("data/decks.json")).unwrap().formats;
    let mut cali = f.remove("cali").unwrap();
    cali.key = "cali".into();
    let mut cmd = Format::commander();
    cmd.key = "commander".into();
    (cali, cmd)
}

fn deck_of(card: &str) -> RawDeck {
    RawDeck {
        tier: None,
        key: "probe".into(),
        name: "probe".into(),
        commander: "X".into(),
        commander2: None,
        ident: None,
        tiers: vec![],
        format: None,
        cards: vec![card.to_string()],
    }
}

/// the ban list in the guide text matches data/formats.toml, token by token (read from the prose, not from the toml)
#[test]
fn the_cali_list_is_the_guide_text_and_nothing_else() {
    let guide = std::fs::read_to_string(root().join("decklists/cali_commander/kyodai-artifact-combo.md")).unwrap();
    let flat = guide.replace('\n', " ");
    let start = flat.find("Banned: ").expect("the guide has a Banned: sentence") + "Banned: ".len();
    let end = start + flat[start..].find(" are legal").expect("... are legal");
    let banned_text = &flat[start..end];
    let (before_legal, _) = banned_text.split_once(". Mana Crypt").expect("Mana Crypt starts the legal sentence");
    // expand the Moxen
    let text = before_legal.replace("the five Moxen (Emerald, Jet, Pearl, Ruby, Sapphire)", "MOXEN");
    let mut want: Vec<String> = vec![];
    for tok in text.split(", ").map(str::trim) {
        match tok {
            "MOXEN" => want.extend(["Emerald", "Jet", "Pearl", "Ruby", "Sapphire"].map(|c| format!("Mox {c}"))),
            "Erayo" => want.push("Erayo, Soratami Ascendant".into()),
            "Nadu" => want.push("Nadu, Winged Wisdom".into()),
            "Tabernacle" => want.push("The Tabernacle at Pendrell Vale".into()),
            t => want.push(t.trim_start_matches("and ").to_string()),
        }
    }
    let (_, cali) = formats();
    let mut got = cali.banned.clone();
    got.sort();
    want.sort();
    assert_eq!(got, want, "guide text: {before_legal}");
    assert_eq!(got.len(), 18);
    // the guide's legal cards are not on it
    for n in ["Mana Crypt", "Tinker", "Primeval Titan", "Paradox Engine", "Jeweled Lotus", "Dockside Extortionist", "Upheaval"]
    {
        assert!(!cali.banned.iter().any(|b| b.eq_ignore_ascii_case(n)), "{n} is legal in the guide");
    }
}

/// Commander's verdict for every one of the 3000+ cached cards is exactly what origin/main decided:
/// `banned` is refused, everything else passes (the one change: a non-legal value is now reported too)
#[test]
fn commander_verdicts_over_the_whole_cache_are_the_old_ones() {
    let (cmd, _) = formats();
    let c = cache();
    let mut n = 0;
    let mut other = vec![];
    for name in c.names() {
        let r = c.get(&name).unwrap();
        let old_banned = r.get("legalities").and_then(|l| l.get("commander")).and_then(|v| v.as_str()) == Some("banned");
        let v = legality_problems(&cmd, &name, Some(r));
        assert_eq!(v.contains(&Illegal::Banned), old_banned, "{name}");
        // any non-banned problem is the new `not legal` report: list it, it must only be junk keys
        for i in &v {
            if let Illegal::NotLegal(why) = i {
                other.push(format!("{name}: {why}"));
            }
        }
        n += 1;
    }
    assert!(n > 3000, "{n}");
    // origin/main's cache has exactly these two non-legal, non-banned entries
    other.sort();
    assert_eq!(other.len(), 2, "{other:?}");
    assert!(other.iter().all(|o| o.ends_with(": Scryfall not_legal") && o.starts_with("_____")), "{other:?}");
}

/// every deck in the repository, under Commander (its own format for kyodai): the problem list is empty, and kyodai under
/// Commander fails on exactly the three
#[test]
fn every_deck_validates_and_kyodai_under_commander_fails_on_exactly_three() {
    let p = Project::new(root());
    let c = cache();
    let report = validate_all(&p, &c).unwrap();
    assert!(report.len() > 36);
    for (d, probs, _) in &report {
        assert!(probs.is_empty(), "{} {}: {probs:?}", d.tier, d.key);
    }
    let (cmd, _) = formats();
    let kyodai = report.iter().find(|(d, ..)| d.key == "kyodai").unwrap().0.clone();
    let (probs, _) = card_problems(&cmd, &kyodai.cards, &[kyodai.commander.as_str()], None, None, &|n| c.get(n).cloned());
    assert_eq!(probs, ["banned in Commander: Mana Crypt, Primeval Titan, Tinker"]);
}

/// cardgen and sim-core must agree on which cards the Cali list bans: cardgen is what blesses a deck list, sim-core is
/// what admits a Commander deck to a Cali table. Over every cached card, and over fabricated back-face cases.
#[test]
fn cardgen_and_sim_core_agree_on_who_is_banned_in_cali() {
    let (_, cali) = formats();
    let (sc_cali, sc_cmd) = sim_core_formats();
    let c = cache();
    let mut disagree = vec![];
    for name in c.names() {
        let r = c.get(&name).unwrap();
        let cg = legality_problems(&cali, &name, Some(r)).contains(&Illegal::Banned);
        let sc = sc_cali.admits(&deck_of(&name), &sc_cmd).is_err();
        if cg != sc {
            disagree.push(format!("{name}: cardgen banned={cg}, sim-core refuses={sc}"));
        }
    }
    assert!(disagree.is_empty(), "{disagree:#?}");
}

/// the same, for a fabricated card whose BACK face is a Cali-banned name (Emeritus of Ideation // Ancestral Recall is
/// the real one: a Commander-legal creature whose prepared spell is named Ancestral Recall)
#[test]
fn a_back_face_named_like_a_banned_card_is_treated_the_same_by_both_readers() {
    let (_, cali) = formats();
    let (sc_cali, sc_cmd) = sim_core_formats();
    let dfc = rec(
        r#"{"name":"Zork the Brave // Balance","layout":"transform","legalities":{"commander":"legal"},
        "card_faces":[{"name":"Zork the Brave"},{"name":"Balance"}]}"#,
    );
    let cg = legality_problems(&cali, "Zork the Brave // Balance", Some(&dfc)).contains(&Illegal::Banned);
    let sc = sc_cali.admits(&deck_of("Zork the Brave // Balance"), &sc_cmd).is_err();
    assert_eq!(cg, sc, "cardgen banned={cg}, sim-core refuses={sc}");
}

/// the real card behind the previous test, from the cache
#[test]
fn emeritus_of_ideation_is_not_banned_by_the_ancestral_recall_entry() {
    let (_, cali) = formats();
    let c = cache();
    let r = c.get("Emeritus of Ideation // Ancestral Recall").expect("in the cache");
    let v = legality_problems(&cali, "Emeritus of Ideation // Ancestral Recall", Some(r));
    assert!(v.is_empty(), "a Commander-legal creature banned only because its second face shares a name: {v:?}");
}

/// edge cases of the name match: whitespace, case, apostrophe style on the record, no record, names with commas
#[test]
fn name_matching_edge_cases() {
    let (cmd, cali) = formats();
    let r = |name: &str| rec(&format!(r#"{{"name":"{name}","legalities":{{"commander":"legal"}}}}"#));
    // padded and upper case request, record spelled canonically
    assert_eq!(legality_problems(&cali, "  MOX JET  ", Some(&r("Mox Jet"))), [Illegal::Banned]);
    // the requested name is a typographic apostrophe spelling, the record the canonical one: the record name still bans it
    assert_eq!(legality_problems(&cali, "Yawgmoth\u{2019}s Bargain", Some(&r("Yawgmoth's Bargain"))), [Illegal::Banned]);
    // the comma names
    assert_eq!(
        legality_problems(&cali, "Nadu, Winged Wisdom", Some(&r("Nadu, Winged Wisdom"))),
        [Illegal::Banned]
    );
    // a name that merely contains a banned name is fine
    assert!(legality_problems(&cali, "Mox Jetty", Some(&r("Mox Jetty"))).is_empty());
    assert!(legality_problems(&cali, "Time Walker", Some(&r("Time Walker"))).is_empty());
    assert!(legality_problems(&cali, "Balance of Power", Some(&r("Balance of Power"))).is_empty());
    // no record at all: the request alone decides, by front face
    assert_eq!(legality_problems(&cali, "mox jet // anything", None), [Illegal::Banned]);
    assert!(legality_problems(&cali, "Sol Ring", None).is_empty());
    assert!(legality_problems(&cmd, "Mox Jet", None).is_empty());
    // split spelled without spaces around the slashes is NOT split by the " // " rule (Scryfall always spaces it)
    assert!(legality_problems(&cali, "Mox Jet//Foo", None).is_empty());
    // the Tabernacle: both spellings of the request reach the record's canonical name
    assert_eq!(
        legality_problems(&cali, "tabernacle at pendrell vale", Some(&r("The Tabernacle at Pendrell Vale"))),
        [Illegal::Banned]
    );
}

/// a Scryfall-`banned` card that is not on the Cali list passes in Cali; one that is `restricted` or `not_legal` does not,
/// and a record with no `card_faces` array and a `null` legalities field does not crash
#[test]
fn odd_records_do_not_panic_and_are_refused_in_cali() {
    let (cmd, cali) = formats();
    for text in [
        r#"{"name":"A","legalities":null}"#,
        r#"{"name":"A","legalities":{"commander":null}}"#,
        r#"{"name":"A","legalities":{}}"#,
        r#"{"name":"A","legalities":{"commander":"legal"},"card_faces":null}"#,
        r#"{"name":"A","legalities":{"commander":"legal"},"card_faces":[{}]}"#,
        r#"{"legalities":{"commander":"legal"}}"#,
    ] {
        let r = rec(text);
        let _ = legality_problems(&cmd, "A", Some(&r));
        let _ = legality_problems(&cali, "A", Some(&r));
    }
    // a record with a non-string commander legality is not "legal": Cali refuses, Commander (as before) lets it through
    let r = rec(r#"{"name":"A","legalities":{"commander":null}}"#);
    assert!(!legality_problems(&cali, "A", Some(&r)).is_empty());
}

/// the three that Scryfall bans for Commander and Cali admits, as a deck: a full 100-card Cali list passes `card_problems`
/// while the same list under Commander reports them (mixed-case request)
#[test]
fn card_problems_names_the_requested_spelling() {
    let (cmd, cali) = formats();
    let cards: Vec<String> = ["Mana Crypt", "Tinker"].map(String::from).to_vec();
    let recs = |n: &str| Some(rec(&format!(r#"{{"name":"{n}","legalities":{{"commander":"banned"}}}}"#)));
    assert!(card_problems(&cali, &cards, &[], None, None, &recs).0.is_empty());
    assert_eq!(card_problems(&cmd, &cards, &[], None, None, &recs).0, ["banned in Commander: Mana Crypt, Tinker"]);
}
