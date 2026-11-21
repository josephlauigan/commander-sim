//! `cardgen find` on a hand-built 41-card catalog (`tests/fixtures/find_oracle.json`: a shared name, a `prepare` card
//! whose face is another card's name, an adventure, a split card, an MDFC with a land back, a meld result, a banned
//! card, Game Changers, ties and unranked cards) and `tests/fixtures/find_decks.json`. Nothing uses the network.

use cardgen::data::Project;
use cardgen::knowledge::{self, FindError, Index, Legal, Query};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// a temporary project root with `data/knowledge/oracle.json` and `data/decks.json`, removed on drop
struct Env(PathBuf);

impl Env {
    fn new() -> Env {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p =
            std::env::temp_dir().join(format!("find-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(p.join("data/knowledge")).unwrap();
        std::fs::copy(fixture("find_oracle.json"), p.join("data/knowledge/oracle.json")).unwrap();
        std::fs::copy(fixture("find_decks.json"), p.join("data/decks.json")).unwrap();
        Env(p)
    }

    fn dir(&self) -> PathBuf {
        self.0.join("data/knowledge")
    }

    /// `find` with the catalog named by --data-dir (so a CARDGEN_KNOWLEDGE_DIR in the environment cannot interfere)
    fn find(&self, args: &[&str]) -> Result<String, FindError> {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.push("--data-dir".into());
        a.push(self.dir().display().to_string());
        knowledge::find(&Project::new(&self.0), &a)
    }

    fn ok(&self, args: &[&str]) -> String {
        self.find(args).unwrap_or_else(|e| panic!("find {args:?} failed: {e}"))
    }

    fn json(&self, args: &[&str]) -> Vec<Value> {
        let mut a = args.to_vec();
        a.push("--json");
        match serde_json::from_str::<Value>(&self.ok(&a)).expect("--json is valid JSON") {
            Value::Array(v) => v,
            v => panic!("--json is not an array: {v}"),
        }
    }

    fn names(&self, args: &[&str]) -> Vec<String> {
        self.json(args).iter().map(|c| c["name"].as_str().unwrap().to_string()).collect()
    }

    fn count(&self, args: &[&str]) -> usize {
        let mut a = args.to_vec();
        a.push("--count");
        self.ok(&a).trim().parse().unwrap()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn usage_err(e: Result<String, FindError>) -> String {
    match e {
        Err(FindError::Usage(m)) => m,
        other => panic!("expected a usage error, got {other:?}"),
    }
}

fn failed_err(e: Result<String, FindError>) -> String {
    match e {
        Err(FindError::Failed(m)) => m,
        other => panic!("expected a failure, got {other:?}"),
    }
}

// ------------------------------------------------------------------------------------------------ filters

#[test]
fn name_is_exact_and_case_insensitive() {
    let e = Env::new();
    assert_eq!(e.names(&["--name", "Sol Ring"]), ["Sol Ring"]);
    assert_eq!(e.names(&["--name", "sOL rING"]), ["Sol Ring"]);
    assert_eq!(e.names(&["--name", "Sol Rin"]), Vec::<String>::new());
    assert_eq!(e.names(&["--name", "Opt"]), ["Opt"], "Option Paralysis is not Opt");
}

#[test]
fn name_contains_searches_full_and_face_names() {
    let e = Env::new();
    assert_eq!(e.names(&["--name-contains", "CHARM"]), ["Azorius Charm", "Esper Charm"]);
    assert_eq!(e.names(&["--name-contains", "petty th"]), ["Brazen Borrower // Petty Theft"]);
    assert_eq!(e.names(&["--name-contains", "opt"]), ["Opt", "Option Paralysis"]);
}

#[test]
fn ident_is_a_subset_limit() {
    let e = Env::new();
    let wu = e.json(&["--ident", "WU", "--limit", "500"]);
    let idents: Vec<&str> = wu.iter().map(|c| c["identity"].as_str().unwrap()).collect();
    assert!(idents.iter().all(|i| ["", "W", "U", "WU"].contains(i)), "{idents:?}");
    for want in ["Sol Ring", "Island", "Plains", "Counterspell", "Absorb", "Azorius Charm", "Fire // Ice"] {
        let ok = want != "Fire // Ice"; // Fire // Ice is red and blue
        assert_eq!(wu.iter().any(|c| c["name"] == want), ok, "{want}");
    }
    // colourless only: C, c and empty all mean it
    for v in ["C", "c", ""] {
        let n = e.json(&["--ident", v, "--limit", "500"]);
        assert!(n.iter().all(|c| c["identity"] == ""), "--ident {v:?}");
        assert!(n.iter().any(|c| c["name"] == "Sol Ring") && n.iter().any(|c| c["name"] == "Command Tower"));
        assert!(!n.iter().any(|c| c["name"] == "Island"));
    }
    // mono
    let w = e.names(&["--ident", "W", "--type", "creature"]);
    assert_eq!(w, ["Isamaru, Hound of Konda"]);
    // multi: an extra colour admits more, a missing one removes the card
    assert!(e.names(&["--ident", "WUB", "--name", "Esper Charm"]).len() == 1);
    assert!(e.names(&["--ident", "WU", "--name", "Esper Charm"]).is_empty());
    assert!(e.names(&["--ident", "U", "--name", "Fire // Ice"]).is_empty());
    assert_eq!(e.names(&["--ident", "ru", "--name", "Fire // Ice"]), ["Fire // Ice"], "letters in any order and case");
    // WUBRG admits every identity
    assert_eq!(e.count(&["--ident", "WUBRG"]), e.count(&[]));
}

#[test]
fn legality_defaults_to_legal_and_any_shows_all() {
    let e = Env::new();
    assert!(e.names(&["--name", "Primeval Titan"]).is_empty());
    let any = e.json(&["--legal", "any", "--name", "Primeval Titan"]);
    assert_eq!(any.len(), 1);
    assert_eq!(any[0]["legal"], "banned");
    assert_eq!(e.names(&["--legal", "banned"]), ["Primeval Titan"]);
    assert_eq!(e.names(&["--legal", "not_legal"]), ["Pick Your Poison", "Little Girl"]);
    // 41 cards: the meld result is left out of every search unless --meld is given
    assert_eq!(e.count(&["--legal", "any"]), 40);
    assert_eq!(e.count(&["--legal", "any", "--meld"]), 41);
    assert_eq!(e.count(&[]), 37);
}

#[test]
fn cmc_operators() {
    let e = Env::new();
    let n = |op: &str| e.names(&["--cmc", op, "--type", "artifact", "--sort", "name"]);
    assert_eq!(n("<=1"), ["Bargain Rock", "Cheap Rock", "Option Paralysis", "Sol Ring"]);
    assert_eq!(n("<1"), Vec::<String>::new());
    assert_eq!(n("=2"), ["Alpha Card", "Beta Card"]);
    assert_eq!(n("2"), ["Alpha Card", "Beta Card"], "a plain number means =");
    assert_eq!(n(">=3"), ["Able Unranked", "Elixir of Vitality", "Zed Unranked"]);
    assert_eq!(n(">2"), ["Able Unranked", "Elixir of Vitality", "Zed Unranked"]);
    assert_eq!(
        n("< 2"),
        ["Bargain Rock", "Cheap Rock", "Option Paralysis", "Sol Ring"],
        "spaces after the operator are fine"
    );
    assert_eq!(e.names(&["--legal", "any", "--cmc", "0.5"]), ["Little Girl"], "half mana");
    assert_eq!(e.names(&["--cmc", "=0", "--type", "land", "--sort", "name"]), ["Command Tower", "Island", "Plains"]);
}

#[test]
fn type_is_a_word_of_the_type_line_of_any_face() {
    let e = Env::new();
    assert!(
        e.names(&["--type", "land", "--sort", "name"])
            .contains(&"Agadeem's Awakening // Agadeem, the Undercrypt".into())
    );
    assert_eq!(e.names(&["--type", "ELF"]), ["Llanowar Elves"]);
    assert_eq!(e.names(&["--type", "dru"]), Vec::<String>::new(), "a word, not a substring");
    assert_eq!(e.names(&["--type", "druid"]), ["Llanowar Elves", "Noble Hierarch"]);
    assert_eq!(e.names(&["--type", "adventure"]), ["Brazen Borrower // Petty Theft"]);
    // repeatable = AND
    assert_eq!(
        e.names(&["--type", "creature", "--type", "instant"]),
        ["Brazen Borrower // Petty Theft", "Emeritus of Conflict // Lightning Bolt"]
    );
    assert_eq!(e.names(&["--type", "legendary", "--type", "planeswalker"]), ["Jace, the Mind Sculptor"]);
    assert_eq!(e.names(&["--type", "battle"]), ["Invasion of Zendikar"]);
    assert_eq!(e.names(&["--type", "basic", "--type", "land", "--sort", "name"]), ["Island", "Plains"]);
}

#[test]
fn text_is_a_case_insensitive_regex_over_every_face() {
    let e = Env::new();
    assert_eq!(
        e.names(&["--text", "COUNTER TARGET SPELL", "--cmc", "<=2"]),
        ["Counterspell", "Mana Leak", "Memory Lapse"]
    );
    // fancy-regex features: look-ahead
    assert_eq!(e.names(&["--text", "counter target (?!spell)", "--cmc", "<=2"]), ["Negate"]);
    // (?-i) turns case-insensitivity off
    assert!(e.names(&["--text", "(?-i)COUNTER"]).is_empty());
    assert!(!e.names(&["--text", "(?-i)Counter"]).is_empty());
    // a face's text is searched (Ice's text; the card has no top-level text)
    assert_eq!(e.names(&["--text", "tap target permanent"]), ["Fire // Ice"]);
    assert_eq!(
        e.names(&["--text", "damage divided", "--text", "lifelink"]),
        Vec::<String>::new(),
        "AND: no card has both"
    );
    assert_eq!(e.names(&["--text", "damage divided", "--text", "tap target"]), ["Fire // Ice"], "AND across two faces");
    // the name and the type line are not searched
    assert!(e.names(&["--text", "Sorcery"]).is_empty());
    assert!(e.names(&["--text", "^Wrath of God$"]).is_empty());
    // anchors apply to the whole text
    assert_eq!(e.names(&["--text", "^Flash"]), ["Brazen Borrower // Petty Theft"]);
}

#[test]
fn gc_flag() {
    let e = Env::new();
    assert_eq!(e.names(&["--gc"]), ["Rhystic Study", "Cyclonic Rift"]);
    let j = e.json(&["--gc"]);
    assert!(j.iter().all(|c| c["gc"] == true));
    assert!(e.json(&["--name", "Sol Ring"])[0]["gc"] == false);
}

// ------------------------------------------------------------------------------------------------ sorting

#[test]
fn sort_edhrec_ranks_ascending_unranked_last_ties_by_name() {
    let e = Env::new();
    assert_eq!(
        e.names(&["--type", "artifact"]),
        [
            "Sol Ring",
            "Alpha Card",
            "Beta Card",
            "Cheap Rock",
            "Bargain Rock",
            "Elixir of Vitality",
            "Option Paralysis",
            "Able Unranked",
            "Zed Unranked"
        ]
    );
    // a rank of 77 twice: Alpha before Beta however the file is ordered
    let ranks: Vec<Value> = e.json(&["--type", "artifact"]).iter().map(|c| c["edhrec_rank"].clone()).collect();
    assert_eq!(ranks[1], 77);
    assert_eq!(ranks[7], Value::Null);
}

#[test]
fn sort_name_cmc_usd() {
    let e = Env::new();
    assert_eq!(
        e.names(&["--type", "artifact", "--sort", "name"]),
        [
            "Able Unranked",
            "Alpha Card",
            "Bargain Rock",
            "Beta Card",
            "Cheap Rock",
            "Elixir of Vitality",
            "Option Paralysis",
            "Sol Ring",
            "Zed Unranked"
        ]
    );
    // cmc ascending, ties by name
    assert_eq!(
        e.names(&["--type", "artifact", "--sort", "cmc"]),
        [
            "Bargain Rock",
            "Cheap Rock",
            "Option Paralysis",
            "Sol Ring",
            "Alpha Card",
            "Beta Card",
            "Able Unranked",
            "Elixir of Vitality",
            "Zed Unranked"
        ]
    );
    // price ascending, no price last (then by name), equal prices by name
    assert_eq!(
        e.names(&["--type", "artifact", "--sort", "usd"]),
        [
            "Elixir of Vitality",
            "Cheap Rock",
            "Option Paralysis",
            "Sol Ring",
            "Alpha Card",
            "Beta Card",
            "Able Unranked",
            "Bargain Rock",
            "Zed Unranked"
        ]
    );
}

#[test]
fn offset_and_limit_page_through_the_sorted_list() {
    let e = Env::new();
    let all = e.names(&["--type", "artifact", "--limit", "100"]);
    assert_eq!(all.len(), 9);
    assert_eq!(e.names(&["--type", "artifact", "--limit", "2"]), &all[..2]);
    assert_eq!(e.names(&["--type", "artifact", "--limit", "2", "--offset", "1"]), &all[1..3]);
    assert_eq!(e.names(&["--type", "artifact", "--offset", "7"]), &all[7..]);
    assert!(e.names(&["--type", "artifact", "--offset", "50"]).is_empty());
    assert!(usage_err(e.find(&["--type", "artifact", "--limit", "0"])).contains("--limit 0"));
    // the default limit is 30
    assert_eq!(e.names(&[]).len(), 30);
    // a table says there is more
    let t = e.ok(&["--type", "artifact", "--limit", "2", "--offset", "1"]);
    assert!(t.contains("showing 2-3 of 9") && t.contains("--offset 3"), "{t}");
    assert!(!e.ok(&["--type", "artifact", "--limit", "100"]).contains("showing"));
    assert!(e.ok(&["--type", "artifact", "--offset", "50"]).contains("no matches"));
}

// ------------------------------------------------------------------------------------------------ names

#[test]
fn a_shared_name_prefers_the_legal_card() {
    let e = Env::new();
    // the index keeps both cards under one name
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    let both = idx.lookup("pick your poison");
    assert_eq!(both.len(), 2);
    assert_eq!(e.names(&["--name", "Pick Your Poison"]).len(), 1);
    assert_eq!(e.json(&["--name", "Pick Your Poison"])[0]["legal"], "legal");
    let any = e.json(&["--legal", "any", "--name", "Pick Your Poison"]);
    assert_eq!(any.len(), 2);
    assert_eq!(any[0]["legal"], "legal", "the legal card first, whatever the sort");
    assert_eq!(any[1]["legal"], "not_legal");
    let by_name = e.json(&["--legal", "any", "--name", "Pick Your Poison", "--sort", "cmc"]);
    assert_eq!(by_name[0]["legal"], "legal");
    assert_ne!(any[0]["oracle_text"], any[1]["oracle_text"]);
}

#[test]
fn an_exact_name_beats_a_face_name() {
    let e = Env::new();
    // "Lightning Bolt" is also a face of the prepare card; only the instant comes back
    let bolt = e.json(&["--name", "Lightning Bolt", "--legal", "any"]);
    assert_eq!(bolt.len(), 1);
    assert_eq!(bolt[0]["name"], "Lightning Bolt");
    assert_eq!(bolt[0]["type_line"], "Instant");
    // the other card is still found by its own name and by its other face
    assert_eq!(
        e.names(&["--name", "Emeritus of Conflict // Lightning Bolt"]),
        ["Emeritus of Conflict // Lightning Bolt"]
    );
    assert_eq!(e.names(&["--name", "Emeritus of Conflict"]), ["Emeritus of Conflict // Lightning Bolt"]);
    // and a text or type filter still sees both
    assert_eq!(
        e.names(&["--text", "deals 3 damage to any target"]),
        ["Lightning Bolt", "Emeritus of Conflict // Lightning Bolt"]
    );
}

#[test]
fn face_names_resolve() {
    let e = Env::new();
    assert_eq!(e.names(&["--name", "Petty Theft"]), ["Brazen Borrower // Petty Theft"]);
    assert_eq!(e.names(&["--name", "brazen borrower"]), ["Brazen Borrower // Petty Theft"]);
    assert_eq!(e.names(&["--name", "Fire"]), ["Fire // Ice"]);
    assert_eq!(e.names(&["--name", "ice"]), ["Fire // Ice"]);
    assert_eq!(e.names(&["--name", "Agadeem, the Undercrypt"]), ["Agadeem's Awakening // Agadeem, the Undercrypt"]);
    assert_eq!(e.names(&["--name", "Fire // Ice"]), ["Fire // Ice"]);
}

// ------------------------------------------------------------------------------------------------ meld

#[test]
fn a_meld_result_needs_meld_unless_named() {
    let e = Env::new();
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    assert_eq!(idx.lookup("Brisela, Voice of Nightmares").len(), 1, "it is indexed");
    // an exact name shows it, with or without --meld
    assert_eq!(e.names(&["--name", "Brisela, Voice of Nightmares"]), ["Brisela, Voice of Nightmares"]);
    assert_eq!(e.names(&["--name", "Brisela, Voice of Nightmares", "--meld"]), ["Brisela, Voice of Nightmares"]);
    // any other search leaves it out and says so when nothing else matched
    let t = e.ok(&["--text", "first strike, lifelink"]);
    assert!(t.contains("no matches") && t.contains("--meld"), "a hint that a meld result is hidden: {t}");
    assert!(!e.names(&["--text", "lifelink", "--limit", "500"]).contains(&"Brisela, Voice of Nightmares".to_string()));
    assert!(e.names(&["--text", "lifelink", "--meld"]).contains(&"Brisela, Voice of Nightmares".to_string()));
    assert_eq!(e.count(&["--meld"]), e.count(&[]) + 1);
    let full = e.json(&["--name", "Brisela, Voice of Nightmares", "--full"]);
    assert_eq!(full[0]["meld_parts"], serde_json::json!(["Bruna, the Fading Light", "Gisela, the Broken Blade"]));
}

// ------------------------------------------------------------------------------------------------ --deck

#[test]
fn deck_limits_identity_and_leaves_out_the_decks_cards() {
    let e = Env::new();
    let all = e.json(&["--deck", "azorius", "--limit", "5000"]);
    assert!(!all.is_empty());
    let names: Vec<&str> = all.iter().map(|c| c["name"].as_str().unwrap()).collect();
    for c in &all {
        assert!(["", "W", "U", "WU"].contains(&c["identity"].as_str().unwrap()), "{c}");
    }
    // by name, by face name (Petty Theft is Brazen Borrower), by a split card's full name, the commander, basics
    for gone in [
        "Sol Ring",
        "Brazen Borrower // Petty Theft",
        "Fire // Ice",
        "Counterspell",
        "Island",
        "Isamaru, Hound of Konda",
    ] {
        assert!(!names.contains(&gone), "{gone} is in the deck");
    }
    assert!(names.contains(&"Plains") && names.contains(&"Absorb") && names.contains(&"Command Tower"));
    // the same count as --count, with the default legality
    assert_eq!(e.count(&["--deck", "azorius"]), all.len());
    // --ident overrides the deck's identity but the exclusion stays
    let w = e.json(&["--deck", "azorius", "--ident", "W", "--limit", "5000"]);
    assert!(w.iter().all(|c| ["", "W"].contains(&c["identity"].as_str().unwrap())));
    assert!(w.iter().all(|c| c["name"] != "Sol Ring"));
    assert!(w.len() < all.len());
    // keys match in any case; the pool is searched after your decks; the commanders and `commander2` are the deck's
    let p = e.names(&["--deck", "POOLDECK", "--limit", "5000"]);
    for gone in ["Birds of Paradise", "Llanowar Elves", "Noble Hierarch"] {
        assert!(!p.contains(&gone.to_string()), "{gone}");
    }
    assert!(p.contains(&"Primeval Titan".to_string()) == false, "banned stays out by default");
    assert!(p.contains(&"Pick Your Poison".to_string()));
    // a deck's identity "C" means colourless only
    let c = e.json(&["--deck", "colorless", "--limit", "5000"]);
    assert!(!c.is_empty() && c.iter().all(|c| c["identity"] == ""));
    assert!(c.iter().all(|c| c["name"] != "Sol Ring"));
    // a card in the deck does not hide a card that merely has its name as a face
    let red = e.names(&["--deck", "red", "--limit", "5000"]);
    assert!(!red.contains(&"Lightning Bolt".to_string()));
    assert!(red.contains(&"Emeritus of Conflict // Lightning Bolt".to_string()));
}

#[test]
fn an_unknown_deck_lists_the_valid_keys() {
    let e = Env::new();
    let m = failed_err(e.find(&["--deck", "nope"]));
    for k in ["azorius", "mono-white", "colorless", "red", "pooldeck", "'nope'"] {
        assert!(m.contains(k), "{m}");
    }
}

// ------------------------------------------------------------------------------------------------ output

#[test]
fn count_prints_only_the_number_of_matches() {
    let e = Env::new();
    assert_eq!(e.ok(&["--type", "artifact", "--count"]), "9\n");
    assert_eq!(
        e.ok(&["--type", "artifact", "--count", "--limit", "2", "--offset", "5", "--json"]),
        "9\n",
        "paging is ignored"
    );
    assert_eq!(e.ok(&["--name", "nothing", "--count"]), "0\n");
}

#[test]
fn json_has_the_documented_shape() {
    let e = Env::new();
    let sol = &e.json(&["--name", "Sol Ring"])[0];
    let keys: Vec<&str> = sol.as_object().unwrap().keys().map(String::as_str).collect();
    // serde_json sorts keys unless preserve_order is on in the workspace; compare as sets
    let mut want = vec![
        "name",
        "oracle_id",
        "layout",
        "mana_cost",
        "cmc",
        "type_line",
        "identity",
        "legal",
        "gc",
        "edhrec_rank",
        "usd",
        "oracle_text",
        "keywords",
        "produced_mana",
        "set",
        "released_at",
        "rarity",
        "reserved",
    ];
    let mut got = keys.clone();
    want.sort();
    got.sort();
    assert_eq!(got, want);
    assert_eq!(sol["mana_cost"], "{1}");
    assert_eq!(sol["cmc"], 1.0);
    assert_eq!(sol["identity"], "");
    assert_eq!(sol["legal"], "legal");
    assert_eq!(sol["edhrec_rank"], 1);
    assert_eq!(sol["usd"], 1.14);
    assert_eq!(sol["oracle_text"], "{T}: Add {C}{C}.");
    // multi-face cards carry their faces; absent text is an empty string
    let bb = &e.json(&["--name", "Petty Theft"])[0];
    let faces = bb["faces"].as_array().unwrap();
    assert_eq!(faces.len(), 2);
    assert_eq!(faces[1]["name"], "Petty Theft");
    let fk: Vec<&str> = faces[1].as_object().unwrap().keys().map(String::as_str).collect();
    let mut fk = fk;
    fk.sort();
    assert_eq!(fk, ["mana_cost", "name", "oracle_text", "type_line"]);
    // the text is the faces' texts joined by a line holding only `//` (what --text matches), not ""
    let text = bb["oracle_text"].as_str().unwrap();
    assert!(text.starts_with("Flash\nFlying") && text.contains("\n//\nReturn target nonland permanent"), "{text}");
    assert_eq!(bb["identity"], "U");
    assert_eq!(bb["layout"], "adventure");
    assert_eq!(bb["keywords"], serde_json::json!(["Flash", "Flying"]));
    // absent fields do not break a card: Command Tower has no cost, Zed Unranked no rank or price
    assert_eq!(e.json(&["--name", "Command Tower"])[0]["mana_cost"], "");
    let z = &e.json(&["--name", "Zed Unranked"])[0];
    assert!(z["edhrec_rank"].is_null() && z["usd"].is_null());
    // the identity of a multicolour card is in WUBRG order
    assert_eq!(e.json(&["--name", "Noble Hierarch"])[0]["identity"], "WUG");
    // nothing found: an empty array
    assert_eq!(e.ok(&["--name", "zzz", "--json"]), "[]\n");
}

#[test]
fn full_adds_the_rest_of_the_record() {
    let e = Env::new();
    // --full changes only the text output: the JSON row is the same shape with or without it
    assert_eq!(e.json(&["--name", "Sol Ring", "--full"]), e.json(&["--name", "Sol Ring"]));
    assert_eq!(e.json(&["--name", "Petty Theft", "--full"]), e.json(&["--name", "Petty Theft"]));
    let sol = &e.json(&["--name", "Sol Ring", "--full"])[0];
    assert_eq!(sol["produced_mana"], serde_json::json!(["C"]));
    assert_eq!(sol["oracle_id"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(sol["reserved"], false);
    assert_eq!(sol["keywords"], serde_json::json!([]));
    assert!(sol.get("power").is_none());
    let jace = &e.json(&["--name", "Jace, the Mind Sculptor", "--full"])[0];
    assert_eq!(jace["loyalty"], "3");
    let llanowar = &e.json(&["--name", "Llanowar Elves", "--full"])[0];
    assert_eq!((&llanowar["power"], &llanowar["toughness"]), (&Value::from("1"), &Value::from("1")));
    // readable text: every face and the facts
    let t = e.ok(&["--name", "Petty Theft", "--full"]);
    for want in [
        "Brazen Borrower // Petty Theft",
        "-- Petty Theft",
        "-- Brazen Borrower",
        "Return target nonland permanent",
        "keywords: Flash, Flying",
        "legal legal",
        "oracle_id",
    ] {
        assert!(t.contains(want), "missing {want:?} in\n{t}");
    }
}

#[test]
fn the_table_has_a_header_and_one_row_per_card() {
    let e = Env::new();
    let t = e.ok(&["--name", "Sol Ring"]);
    let lines: Vec<&str> = t.lines().collect();
    assert_eq!(lines.len(), 2, "{t}");
    for h in ["name", "cost", "type", "edhrec", "usd", "legal"] {
        assert!(lines[0].contains(h), "{t}");
    }
    let cells: Vec<&str> = lines[1].split_whitespace().collect();
    assert_eq!(cells, ["Sol", "Ring", "{1}", "Artifact", "1", "1.14", "legal"]);
    // no rank, no price, no cost: dashes
    let z = e.ok(&["--name", "Island", "--legal", "any"]);
    assert!(z.lines().nth(1).unwrap().contains(" -  "), "{z}");
    assert_eq!(e.ok(&["--name", "nothing"]), "no matches\n");
}

// ------------------------------------------------------------------------------------------------ bad input

#[test]
fn a_malformed_regex_fails_with_the_pattern_in_the_message() {
    let e = Env::new();
    let m = failed_err(e.find(&["--text", "("]));
    assert!(m.contains("bad --text regex '('"), "{m}");
    let m = failed_err(e.find(&["--text", "fine", "--text", "[a-"]));
    assert!(m.contains("[a-"), "{m}");
}

#[test]
fn bad_flags_and_numbers_are_usage_errors() {
    let e = Env::new();
    let cases: &[&[&str]] = &[
        &["--role", "ramp"],
        &["--digest"],
        &["--map"],
        &["--bogus"],
        &["Sol Ring"],
        &["--name"],
        &["--limit"],
        &["--limit", "ten"],
        &["--limit", "-1"],
        &["--offset", "1.5"],
        &["--cmc", "two"],
        &["--cmc", "<="],
        &["--cmc", "=="],
        &["--cmc", "nan"],
        &["--cmc", "inf"],
        &["--legal", "maybe"],
        &["--sort", "price"],
        &["--ident", "WX"],
        &["--name", "a", "--name", "b"],
        &["--deck", "azorius", "--deck", "red"],
    ];
    for args in cases {
        let m = usage_err(e.find(args));
        assert!(!m.is_empty(), "{args:?}");
    }
    assert!(usage_err(e.find(&["--role", "ramp"])).contains("--role"));
    assert!(usage_err(e.find(&["--limit", "ten"])).contains("--limit needs a number"));
    // a bad number does not hide behind a catalog problem: it is reported before the catalog is read
    let r = knowledge::find(&Project::new("/nonexistent"), &["--limit".to_string(), "x".to_string()]);
    assert!(matches!(r, Err(FindError::Usage(_))));
}

// ------------------------------------------------------------------------------------------------ the index

#[test]
fn a_bad_catalog_is_an_error() {
    let e = Env::new();
    let path = e.dir().join("oracle.json");
    let good = std::fs::read_to_string(&path).unwrap();
    let write = |s: &str| std::fs::write(&path, s).unwrap();
    // schema
    write(&good.replacen("\"schema\": 1", "\"schema\": 2", 1));
    let m = failed_err(e.find(&[]));
    assert!(m.contains("schema") && m.contains("bulk-sync"), "{m}");
    write(&good.replacen("\"schema\": 1,", "", 1));
    assert!(failed_err(e.find(&[])).contains("oracle.json"));
    // truncated, not JSON, wrong count
    write(&good[..good.len() / 2]);
    assert!(failed_err(e.find(&[])).contains("cannot read"));
    write("not json");
    assert!(failed_err(e.find(&[])).contains("bulk-sync"));
    write(&good.replacen("\"count\": 41", "\"count\": 40", 1));
    assert!(failed_err(e.find(&[])).contains("meta.count"));
    // unknown fields are ignored
    write(
        &good
            .replacen("\"cards\"", "\"extra\": [1], \"cards\"", 1)
            .replace("\"rarity\"", "\"future_field\": {\"a\": null}, \"rarity\""),
    );
    assert_eq!(e.count(&["--name", "Sol Ring"]), 1);
    // an empty catalog is a catalog
    write("{\"meta\": {\"schema\": 1, \"count\": 0}, \"cards\": []}");
    assert_eq!(e.ok(&["--count"]), "0\n");
}

#[test]
fn the_index_derives_identity_types_and_legality() {
    let e = Env::new();
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    let get = |n: &str| &idx.cards[idx.lookup(n)[0] as usize];
    assert_eq!(get("Noble Hierarch").identity(), "WUG");
    assert_eq!(get("Sol Ring").ident, 0);
    assert_eq!(get("Fire // Ice").ident, 0b01010, "U (2) and R (8)");
    assert_eq!(get("Fire // Ice").identity(), "UR");
    assert_ne!(get("Jace, the Mind Sculptor").types & knowledge::T_PLANESWALKER, 0);
    assert_ne!(get("Jace, the Mind Sculptor").types & knowledge::T_LEGENDARY, 0);
    assert_ne!(get("Brazen Borrower").types & (knowledge::T_CREATURE | knowledge::T_INSTANT), 0);
    assert_ne!(get("Agadeem's Awakening").types & knowledge::T_LAND, 0, "the back face is a land");
    assert_ne!(get("Invasion of Zendikar").types & knowledge::T_BATTLE, 0);
    assert_eq!(get("Primeval Titan").legal, Legal::Banned);
    assert_eq!(get("Sol Ring").usd, Some(1.14));
    assert_eq!(get("Zed Unranked").usd, None);
    assert!(
        get("Fire // Ice").text.contains("Fire deals 2 damage")
            && get("Fire // Ice").text.contains("Tap target permanent")
    );
}

// ------------------------------------------------------------------------------------------------ the binary

fn bin() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cardgen"));
    c.env_remove("CARDGEN_KNOWLEDGE_DIR");
    c
}

fn listing(dir: &Path) -> Vec<(String, u64)> {
    let mut v = vec![];
    fn walk(d: &Path, v: &mut Vec<(String, u64)>) {
        for e in std::fs::read_dir(d).unwrap() {
            let e = e.unwrap();
            let m = e.metadata().unwrap();
            if m.is_dir() {
                walk(&e.path(), v);
            } else {
                v.push((e.path().display().to_string(), m.len()));
            }
        }
    }
    walk(dir, &mut v);
    v.sort();
    v
}

#[test]
fn the_binary_exits_non_zero_with_usage_and_writes_nothing() {
    let e = Env::new();
    let before = listing(&e.0);
    for args in [vec!["--role", "ramp"], vec!["--limit", "x"], vec!["--name"], vec!["--map"]] {
        let out = bin().arg("--root").arg(&e.0).arg("find").args(&args).output().unwrap();
        assert!(!out.status.success(), "{args:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("usage:") && err.contains("cardgen find"), "{args:?}: {err}");
        assert!(out.stdout.is_empty());
    }
    let out = bin().arg("--root").arg(&e.0).args(["find", "--text", "("]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("bad --text regex") && !err.contains("usage:"), "{err}");
    let out = bin().arg("--root").arg(&e.0).args(["find", "--deck", "nope"]).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("azorius"));
    assert_eq!(listing(&e.0), before, "find is read-only");
}

#[test]
fn the_binary_finds_a_card() {
    let e = Env::new();
    let out = bin().arg("--root").arg(&e.0).args(["find", "--name", "Sol Ring", "--json"]).output().unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
}

#[test]
fn the_knowledge_dir_comes_from_the_flag_then_the_environment_then_the_project() {
    let e = Env::new();
    // a missing catalog names bulk-sync
    let empty = e.0.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let out = bin().arg("--root").arg(&e.0).args(["find", "--data-dir"]).arg(&empty).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("bulk-sync") && err.contains("empty"), "{err}");
    // the environment variable
    let alt = e.0.join("alt");
    std::fs::create_dir_all(&alt).unwrap();
    std::fs::write(
        alt.join("oracle.json"),
        concat!(
            r#"{"meta": {"schema": 1, "count": 1}, "cards": [{"name": "Only Card", "cmc": 0.0, "type_line": "Land","#,
            r#" "color_identity": [], "oracle_id": "only", "legalities": {"commander": "legal"}}]}"#
        ),
    )
    .unwrap();
    let out =
        bin().env("CARDGEN_KNOWLEDGE_DIR", &alt).arg("--root").arg(&e.0).args(["find", "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v[0]["name"], "Only Card");
    // --data-dir beats the environment variable
    let out = bin()
        .env("CARDGEN_KNOWLEDGE_DIR", &alt)
        .arg("--root")
        .arg(&e.0)
        .args(["find", "--count", "--data-dir"])
        .arg(e.dir())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "37");
    // the project's own data/knowledge by default
    let out = bin().arg("--root").arg(&e.0).args(["find", "--count"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "37");
    // a set but wrong environment variable is an error, not a silent fallback
    let out =
        bin().env("CARDGEN_KNOWLEDGE_DIR", &empty).arg("--root").arg(&e.0).args(["find", "--count"]).output().unwrap();
    assert!(!out.status.success());
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid", "-c", "commit.gpgsign=false"])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// a worktree has `data/knowledge/` (the tracked sync.lock) but no oracle.json: it falls back to the main checkout's
#[test]
fn a_worktree_falls_back_to_the_main_checkouts_catalog() {
    let e = Env::new();
    let main = e.0.join("main");
    std::fs::create_dir_all(main.join("data/knowledge")).unwrap();
    std::fs::write(main.join("data/knowledge/sync.lock"), "schema=1\n").unwrap();
    std::fs::copy(fixture("find_decks.json"), main.join("data/decks.json")).unwrap();
    git(&main, &["init", "-q"]);
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "x"]);
    // the main checkout has no catalog yet: a clear error
    let out = bin().arg("--root").arg(&main).args(["find", "--count"]).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("bulk-sync"));
    std::fs::copy(fixture("find_oracle.json"), main.join("data/knowledge/oracle.json")).unwrap();
    let wt = e.0.join("wt");
    git(&main, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    assert!(wt.join("data/knowledge/sync.lock").is_file() && !wt.join("data/knowledge/oracle.json").exists());
    for root in [&wt, &main] {
        let out = bin().arg("--root").arg(root).args(["find", "--name", "Sol Ring", "--json"]).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(serde_json::from_slice::<Value>(&out.stdout).unwrap().as_array().unwrap().len(), 1);
    }
    // a worktree with its own catalog uses it
    let mut own: String = std::fs::read_to_string(fixture("find_oracle.json")).unwrap();
    own = own.replace("\"Sol Ring\"", "\"Sol Ring Two\"");
    std::fs::write(wt.join("data/knowledge/oracle.json"), own).unwrap();
    let out = bin().arg("--root").arg(&wt).args(["find", "--name", "Sol Ring Two", "--count"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
    let _ = Command::new("git").arg("-C").arg(&main).args(["worktree", "remove", "--force"]).arg(&wt).output();
}

/// the catalog `bulk-sync` writes (here from its 51-card fixture, no network) loads and is searchable
#[test]
fn a_bulk_sync_catalog_loads() {
    use cardgen::bulk::{self, Entry, Options, Remote};
    struct None_;
    impl Remote for None_ {
        fn latest(&mut self) -> Result<Entry, String> {
            Err("no network in tests".into())
        }
        fn download(&mut self, _: &str, _: &Path) -> Result<(), String> {
            Err("no network in tests".into())
        }
    }
    let e = Env::new();
    let dir = e.0.join("synced");
    let opts = Options {
        dir: dir.clone(),
        from_file: Some(fixture("bulk_fixture.json")),
        min_count: 1,
        synced: "2026-11-21T00:00:00Z".into(),
    };
    let out = bulk::run(&opts, &mut None_).unwrap();
    assert!(!out.up_to_date);
    let idx = Index::load(&dir.join("oracle.json")).unwrap();
    assert!(idx.cards.len() > 20);
    assert!(idx.cards.iter().all(|c| !c.raw.oracle_id.is_empty()));
    let a: Vec<String> = ["--data-dir", dir.to_str().unwrap(), "--legal", "any", "--count"].map(String::from).to_vec();
    let n: usize = knowledge::find(&Project::new(&e.0), &a).unwrap().trim().parse().unwrap();
    assert!(n >= idx.cards.iter().filter(|c| !c.is_meld()).count());
}

// ------------------------------------------------------------------------------------------------ round 2 fixes

/// a minimal legal record; `id` is its oracle_id
fn rec(name: &str, id: &str) -> Value {
    serde_json::json!({"name": name, "cmc": 1.0, "type_line": "Artifact", "color_identity": [],
        "legalities": {"commander": "legal"}, "oracle_id": id, "mana_cost": "{1}"})
}

impl Env {
    /// replace the catalog with these records (and the decks file with `decks`)
    fn with_cards(cards: Vec<Value>, decks: Value) -> Env {
        let e = Env::new();
        let meta = serde_json::json!({"schema": 1, "count": cards.len()});
        let text = serde_json::to_string(&serde_json::json!({"meta": meta, "cards": cards})).unwrap();
        std::fs::write(e.dir().join("oracle.json"), text).unwrap();
        std::fs::write(e.0.join("data/decks.json"), decks.to_string()).unwrap();
        e
    }
}

fn accent_catalog() -> Vec<Value> {
    [
        "Andúril, Narsil Reforged",
        "Barad-dûr",
        "Gríma Wormtongue",
        "Lim-Dûl's Vault",
        "Jötun Grunt",
        "Séance",
        "Urza’s Tower",
        "Aether Vial",
        "Æsop's Fable",
        "Sol Ring",
    ]
    .iter()
    .enumerate()
    .map(|(i, n)| rec(n, &format!("id-{i}")))
    .collect()
}

#[test]
fn fold_is_the_shared_search_key() {
    use knowledge::fold;
    assert_eq!(fold("Andúril, Narsil Reforged"), "anduril, narsil reforged");
    assert_eq!(fold("Barad-dûr"), "barad-dur", "hyphens stay");
    assert_eq!(fold("Gríma Wormtongue"), "grima wormtongue");
    assert_eq!(fold("Lim-Dûl’s Vault"), "lim-dul's vault");
    assert_eq!(fold("Jötun Grunt"), "jotun grunt");
    assert_eq!(fold("Séance"), "seance");
    for a in ["’", "‘", "ʼ", "`", "´"] {
        assert_eq!(fold(&format!("Urza{a}s Saga")), "urza's saga", "{a}");
    }
    assert_eq!(fold("Æther Vial"), "aether vial");
    assert_eq!(fold("ÆTHER"), "aether");
    assert_eq!(fold("Œuvre"), "oeuvre");
    assert_eq!(fold("Straße"), "strasse");
    assert_eq!(fold("  Sol Ring \n"), "sol ring");
    assert_eq!(fold("Fire // Ice"), "fire // ice", "punctuation and inner spaces stay");
    assert_eq!(fold("Sol  Ring"), "sol  ring");
    assert_eq!(fold(""), "");
}

#[test]
fn names_match_across_spellings() {
    let decks = serde_json::json!({"mine": [], "pool": []});
    let e = Env::with_cards(accent_catalog(), decks);
    let cases = [
        ("Anduril, Narsil Reforged", "Andúril, Narsil Reforged"),
        ("Andúril, Narsil Reforged", "Andúril, Narsil Reforged"),
        ("barad-dur", "Barad-dûr"),
        ("Grima Wormtongue", "Gríma Wormtongue"),
        ("Lim-Dul's Vault", "Lim-Dûl's Vault"),
        ("Lim-Dul’s Vault", "Lim-Dûl's Vault"),
        ("Jotun Grunt", "Jötun Grunt"),
        ("Seance", "Séance"),
        ("Urza's Tower", "Urza’s Tower"),
        ("Urza’s Tower", "Urza’s Tower"),
        ("Æther Vial", "Aether Vial"),
        ("Aether Vial", "Aether Vial"),
        ("Aesop's Fable", "Æsop's Fable"),
    ];
    for (q, canon) in cases {
        assert_eq!(e.names(&["--name", q]), [canon], "--name {q}");
    }
    let subs = [
        ("anduril", "Andúril, Narsil Reforged"),
        ("barad-dur", "Barad-dûr"),
        ("grima", "Gríma Wormtongue"),
        ("lim-dul", "Lim-Dûl's Vault"),
        ("jotun", "Jötun Grunt"),
        ("seance", "Séance"),
        ("urza's", "Urza’s Tower"),
        ("urza’s", "Urza’s Tower"),
        ("Æther", "Aether Vial"),
        ("aether", "Aether Vial"),
    ];
    for (q, canon) in subs {
        assert_eq!(e.names(&["--name-contains", q]), [canon], "--name-contains {q}");
    }
    // punctuation is not stripped: a missing hyphen is a different name
    assert!(e.names(&["--name", "Barad dur"]).is_empty());
    // the output keeps the canonical spelling
    assert_eq!(e.json(&["--name", "Seance"])[0]["name"], "Séance");
}

#[test]
fn deck_exclusion_matches_either_spelling() {
    let ascii: Vec<&str> =
        vec!["Anduril, Narsil Reforged", "Barad-dur", "Grima Wormtongue", "Lim-Dul's Vault", "Jotun Grunt", "Seance"];
    let accented: Vec<&str> = vec!["Urza's Tower", "Æther Vial", "Aesop's Fable"];
    let decks = serde_json::json!({"mine": [
        {"key": "a", "ident": "WUBRG", "commander": "Sol Ring", "cards": ascii},
        {"key": "b", "ident": "WUBRG", "commander": "Sol Ring", "cards": accented}], "pool": []});
    let e = Env::with_cards(accent_catalog(), decks);
    // the catalog spells the first six with accents and the deck without; the last three the other way
    assert_eq!(
        e.names(&["--deck", "a", "--limit", "all", "--sort", "name"]),
        ["Æsop's Fable", "Aether Vial", "Urza’s Tower"]
    );
    assert_eq!(
        e.names(&["--deck", "b", "--limit", "all", "--sort", "name"]),
        ["Andúril, Narsil Reforged", "Barad-dûr", "Gríma Wormtongue", "Jötun Grunt", "Lim-Dûl's Vault", "Séance"]
    );
}

#[test]
fn limit_all_is_unlimited() {
    let e = Env::new();
    assert_eq!(e.names(&["--limit", "all"]).len(), 37);
    assert_eq!(e.names(&[]).len(), 30, "the default stays 30");
    assert_eq!(e.names(&["--limit", "all", "--offset", "5"]).len(), 32);
    assert!(!e.ok(&["--limit", "all"]).contains("showing"));
    for bad in ["ALL", "alll", ""] {
        assert!(usage_err(e.find(&["--limit", bad])).contains("--limit needs a number or 'all'"), "{bad:?}");
    }
}

#[test]
fn a_value_that_looks_like_a_flag_is_a_missing_value() {
    let e = Env::new();
    for flag in ["--name", "--name-contains", "--type", "--text", "--deck", "--data-dir", "--legal", "--limit"] {
        let m = usage_err(e.find(&[flag, "--json"]));
        assert!(m.contains(&format!("{flag} needs a value (got '--json')")), "{m}");
    }
    assert!(usage_err(e.find(&["--name", "--role"])).contains("--name needs a value (got '--role')"));
    // negative numbers are values
    assert!(Query::parse(&["--cmc".to_string(), "-1".to_string()]).is_ok());
    // and the binary exits non-zero with the usage text
    let e2 = Env::new();
    let out = bin().arg("--root").arg(&e2.0).args(["find", "--name", "--role"]).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage:"));
}

#[test]
fn a_failed_stdout_write_is_an_error() {
    let Ok(full) = std::fs::OpenOptions::new().write(true).open("/dev/full") else { return };
    let e = Env::new();
    let out = bin()
        .arg("--root")
        .arg(&e.0)
        .args(["find", "--json", "--limit", "all"])
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!out.status.success(), "a full disk is not success");
    assert!(String::from_utf8_lossy(&out.stderr).contains("cardgen: cannot write the result"));
}

#[test]
fn an_empty_name_or_a_repeated_oracle_id_is_a_bad_catalog() {
    let decks = serde_json::json!({"mine": [], "pool": []});
    for name in ["", "   "] {
        let e = Env::with_cards(vec![rec("Ok", "a"), rec(name, "b")], decks.clone());
        let m = failed_err(e.find(&[]));
        assert!(m.contains("record 2") && m.contains("empty name") && m.contains("bulk-sync"), "{m}");
        assert!(m.contains("oracle.json"), "{m}");
    }
    let e = Env::with_cards(vec![rec("One", "same"), rec("Two", "x"), rec("Three", "same")], decks.clone());
    let m = failed_err(e.find(&[]));
    assert!(
        m.contains("record 3") && m.contains("repeats oracle_id same of record 1") && m.contains("bulk-sync"),
        "{m}"
    );
    // a shared name with different ids is fine, and records without an id are not compared
    let e = Env::with_cards(vec![rec("Twin", "a"), rec("Twin", "b"), rec("Three", "c")], decks.clone());
    assert_eq!(e.count(&["--name", "Twin"]), 2);
    assert_eq!(e.count(&[]), 3, "a shared name is two cards");
    // an empty oracle_id is rejected too (roles are keyed on it)
    let e = Env::with_cards(vec![rec("Ok", "a"), rec("No Id", "")], decks);
    let m = failed_err(e.find(&[]));
    assert!(m.contains("record 2") && m.contains("no oracle_id") && m.contains("bulk-sync"), "{m}");
}

#[test]
fn display_width_counts_cells() {
    use knowledge::display_width;
    assert_eq!(display_width(""), 0);
    assert_eq!(display_width("Sol Ring"), 8);
    assert_eq!(display_width("Séance"), 6, "precomposed");
    assert_eq!(display_width("Se\u{301}ance"), 6, "a combining mark takes no cell");
    assert_eq!(display_width("長い名前"), 8, "CJK and kana are two cells each");
    assert_eq!(display_width("한글"), 4);
    assert_eq!(display_width("ＡＢ"), 4, "full-width forms");
    assert_eq!(display_width("🌀"), 2);
    assert_eq!(display_width("—"), 1);
}

#[test]
fn table_columns_line_up_for_wide_and_combining_names() {
    let decks = serde_json::json!({"mine": [], "pool": []});
    let e = Env::with_cards(vec![rec("Se\u{301}ance", "a"), rec("長い名前", "b"), rec("Plain", "c")], decks);
    let t = e.ok(&["--sort", "name"]);
    let cells: Vec<usize> =
        t.lines().map(|l| knowledge::display_width(&l[..l.find("{1}").or_else(|| l.find("cost")).unwrap()])).collect();
    assert!(cells.windows(2).all(|w| w[0] == w[1]), "the cost column starts at one cell: {cells:?}\n{t}");
}

// ------------------------------------------------------------------------------------------------ round 3 fixes

impl Env {
    fn notes(&self, args: &[&str]) -> knowledge::FindOutput {
        let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        a.push("--data-dir".into());
        a.push(self.dir().display().to_string());
        knowledge::find_with_notes(&Project::new(&self.0), &a).unwrap_or_else(|e| panic!("find {args:?}: {e}"))
    }
}

#[test]
fn the_cost_of_a_card_without_its_own_is_its_face_costs() {
    let mut mdfc = rec("Front // Back", "m");
    mdfc.as_object_mut().unwrap().remove("mana_cost");
    mdfc["type_line"] = serde_json::json!("Creature — Human // Sorcery");
    mdfc["card_faces"] = serde_json::json!([
        {"name": "Front", "mana_cost": "{2}{W}", "type_line": "Creature — Human", "oracle_text": "Front line one.\nFront line two."},
        {"name": "Back", "mana_cost": "{1}{R}", "type_line": "Sorcery", "oracle_text": "Back line."}]);
    let mut land_back = rec("Spell // Land", "l");
    land_back.as_object_mut().unwrap().remove("mana_cost");
    land_back["card_faces"] = serde_json::json!([
        {"name": "Spell", "mana_cost": "{G}", "type_line": "Sorcery"},
        {"name": "Land", "mana_cost": "", "type_line": "Land"}]);
    let e = Env::with_cards(vec![mdfc, land_back, rec("Plain", "p")], serde_json::json!({"mine": [], "pool": []}));
    let rows = e.json(&["--sort", "name", "--limit", "all"]);
    assert_eq!(rows[0]["mana_cost"], "{2}{W} // {1}{R}");
    assert_eq!(rows[1]["mana_cost"], "{1}", "its own");
    assert_eq!(rows[2]["mana_cost"], "{G}", "empty face costs are left out");
    assert_eq!(rows[0]["oracle_text"], "Front line one.\nFront line two.\n//\nBack line.");
    let t = e.ok(&["--sort", "name"]);
    assert!(t.contains("{2}{W} // {1}{R}"), "{t}");
    assert!(e.ok(&["--name", "Front", "--full"]).contains("{2}{W} // {1}{R}"));
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    let c = &idx.cards[idx.lookup("front // back")[0] as usize];
    assert_eq!(c.cost(), "{2}{W} // {1}{R}");
    // the canonical line numbering has no joiner line, and says which face a line is on
    assert_eq!(c.lines(), [(Some(0), "Front line one."), (Some(0), "Front line two."), (Some(1), "Back line.")]);
    assert_eq!(c.subtypes(), ["Human"]);
}

#[test]
fn card_helpers_for_later_phases() {
    let e = Env::new();
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    let get = |n: &str| &idx.cards[idx.lookup(n)[0] as usize];
    assert_eq!(get("Sol Ring").lines(), [(None, "{T}: Add {C}{C}.")]);
    assert_eq!(get("Sol Ring").cost(), "{1}");
    assert_eq!(get("Command Tower").cost(), "");
    assert_eq!(get("Brazen Borrower").subtypes(), ["Faerie", "Rogue", "Adventure"]);
    assert_eq!(get("Llanowar Elves").subtypes(), ["Elf", "Druid"]);
    assert!(get("Sol Ring").subtypes().is_empty());
}

#[test]
fn self_names_are_the_printed_names() {
    let e = Env::with_cards(accent_catalog(), serde_json::json!({"mine": [], "pool": []}));
    let idx = Index::load(&e.dir().join("oracle.json")).unwrap();
    let c = &idx.cards[idx.lookup("anduril, narsil reforged")[0] as usize];
    assert_eq!(c.self_names(), ["Andúril, Narsil Reforged", "Andúril"]);
    let c = &idx.cards[idx.lookup("sol ring")[0] as usize];
    assert_eq!(c.self_names(), ["Sol Ring"]);
    let f = Env::new();
    let idx = Index::load(&f.dir().join("oracle.json")).unwrap();
    let c = &idx.cards[idx.lookup("fire // ice")[0] as usize];
    assert_eq!(c.self_names(), ["Fire // Ice", "Fire", "Ice"]);
    let c = &idx.cards[idx.lookup("jace, the mind sculptor")[0] as usize];
    assert_eq!(c.self_names(), ["Jace, the Mind Sculptor", "Jace"]);
}

#[test]
fn the_index_api_for_later_phases() {
    let e = Env::new();
    let project = Project::new(&e.0);
    // open = knowledge_dir + load
    let idx = Index::open(&project, Some(&e.dir())).unwrap();
    assert_eq!(idx.cards.len(), 41);
    assert!(Index::open(&project, Some(&e.0.join("nowhere"))).err().unwrap().contains("bulk-sync"));
    // by_oracle_id
    let sol = idx.by_oracle_id("00000000-0000-0000-0000-000000000001").unwrap();
    assert_eq!(idx.cards[sol as usize].raw.name, "Sol Ring");
    assert_eq!(idx.by_oracle_id("nope"), None);
    // subtype vocabulary: each word once per card, sorted by word
    let v = idx.subtype_vocab();
    assert!(v.windows(2).all(|w| w[0].0 < w[1].0));
    assert_eq!(v.iter().find(|(w, _)| w == "Druid").unwrap().1, 2);
    assert_eq!(v.iter().find(|(w, _)| w == "Island").unwrap().1, 1);
    // Index::sort is find's order: EDHREC with unranked last, ties by name
    let mut ids: Vec<u32> = (0..idx.cards.len() as u32).collect();
    idx.sort(&mut ids, knowledge::Sort::Edhrec);
    let by_find = e.names(&["--legal", "any", "--meld", "--limit", "all"]);
    let sorted: Vec<&str> = ids.iter().map(|i| idx.cards[*i as usize].raw.name.as_str()).collect();
    assert_eq!(sorted, by_find);
    // Query::default: legal only, EDHREC, no limit
    let q = Query::default();
    assert_eq!((q.sort, q.limit, q.offset), (knowledge::Sort::Edhrec, Some(usize::MAX), 0));
    let hits = idx.matches(&q, None, &[], &[]).unwrap();
    assert_eq!(hits.len(), 37);
    // ident helpers
    assert_eq!(knowledge::parse_ident("ub").unwrap(), 0b00110);
    assert_eq!(knowledge::ident_string(0b00110), "UB");
    assert_eq!(knowledge::ident_string(0), "");
    // decks
    let d = knowledge::load_deck(&project, "azorius").unwrap();
    assert_eq!(knowledge::deck_ident(&d).unwrap(), 0b00011);
    let (marks, unresolved) = idx.deck_members(&d);
    assert_eq!(unresolved, ["Not In Catalog"]);
    let names: Vec<&str> =
        marks.iter().enumerate().filter(|(_, m)| **m).map(|(i, _)| idx.cards[i].raw.name.as_str()).collect();
    for n in [
        "Sol Ring",
        "Brazen Borrower // Petty Theft",
        "Fire // Ice",
        "Counterspell",
        "Island",
        "Isamaru, Hound of Konda",
    ] {
        assert!(names.contains(&n), "{n}");
    }
    assert_eq!(names.len(), 6);
    let mut bad = d.clone();
    bad.ident = "X".into();
    assert!(knowledge::deck_ident(&bad).unwrap_err().contains("azorius"));
}

#[test]
fn a_page_cut_says_so_even_in_json() {
    let e = Env::new();
    let o = e.notes(&["--type", "artifact", "--limit", "2", "--offset", "1", "--json"]);
    assert_eq!(o.notes, ["(showing 2-3 of 9; --offset 3 or --limit all)"]);
    assert!(o.notes_on_stderr);
    assert!(!o.stdout().contains("showing"), "stdout stays a bare JSON array");
    assert!(serde_json::from_str::<Value>(&o.stdout()).unwrap().is_array());
    assert_eq!(o.stderr(), "(showing 2-3 of 9; --offset 3 or --limit all)\n");
    // a table gets it as a trailing line on stdout
    let t = e.notes(&["--type", "artifact", "--limit", "2"]);
    assert!(!t.notes_on_stderr && t.stdout().ends_with("(showing 1-2 of 9; --offset 2 or --limit all)\n"));
    assert_eq!(t.stderr(), "");
    // no note when everything is shown, with --limit all, or for --count
    assert!(e.notes(&["--type", "artifact", "--json"]).notes.is_empty());
    assert!(e.notes(&["--type", "artifact", "--limit", "all", "--json"]).notes.is_empty());
    assert_eq!(e.ok(&["--type", "artifact", "--count"]), "9\n");
}

#[test]
fn a_named_card_the_filters_removed_says_why() {
    let e = Env::new();
    // banned
    let t = e.notes(&["--name", "Primeval Titan", "--json"]);
    assert_eq!(t.out, "[]\n");
    assert_eq!(t.notes, ["(1 card(s) named Primeval Titan exist outside these filters: banned; try --legal any)"]);
    assert!(e.ok(&["--name", "Primeval Titan"]).ends_with("try --legal any)\n"));
    // a deck's own card, and the identity
    let t = e.notes(&["--name", "Sol Ring", "--deck", "azorius", "--json"]);
    assert!(
        t.notes.iter().any(|n| n.contains("1 card(s) named Sol Ring") && n.contains("in deck azorius")),
        "{:?}",
        t.notes
    );
    let t = e.notes(&["--name", "Lightning Bolt", "--ident", "U", "--json"]);
    assert!(t.notes[0].contains("outside the identity") && !t.notes[0].contains("--legal any"), "{:?}", t.notes);
    // another filter
    let t = e.notes(&["--name", "Sol Ring", "--cmc", ">5", "--count"]);
    assert_eq!(t.out, "0\n");
    assert!(t.notes[0].contains("other filters"));
    // two cards of one name: both counted
    let t = e.notes(&["--name", "Pick Your Poison", "--legal", "banned", "--json"]);
    assert!(t.notes[0].starts_with("(2 card(s) named Pick Your Poison"), "{:?}", t.notes);
    // a name that is not in the catalog at all, and a name that matched, say nothing
    assert!(e.notes(&["--name", "zzz", "--json"]).notes.is_empty());
    assert!(e.notes(&["--name", "Sol Ring", "--json"]).notes.is_empty());
}

#[test]
fn deck_names_that_match_no_card_are_reported() {
    let e = Env::new();
    let t = e.notes(&["--deck", "azorius", "--count"]);
    assert_eq!(t.notes, ["(deck azorius: 1 name(s) not in the catalog: Not In Catalog)"]);
    let t = e.notes(&["--deck", "mono-white", "--json", "--limit", "1"]);
    assert!(t.notes.iter().all(|n| !n.contains("not in the catalog")), "{:?}", t.notes);
    // the list is cut at ten
    let names: Vec<String> = (0..12).map(|i| format!("Ghost {i:02}")).collect();
    let decks =
        serde_json::json!({"mine": [{"key": "g", "ident": "C", "commander": "Plain", "cards": names}], "pool": []});
    let g = Env::with_cards(vec![rec("Plain", "p"), rec("Other", "o")], decks);
    let n = &g.notes(&["--deck", "g", "--count"]).notes[0];
    assert!(n.starts_with("(deck g: 12 name(s) not in the catalog: Ghost 00,") && n.ends_with(", ...)"), "{n}");
}

#[test]
fn the_meld_hint_is_a_note() {
    let e = Env::new();
    let o = e.notes(&["--text", "first strike, lifelink"]);
    assert_eq!(o.out, "no matches\n");
    assert_eq!(o.notes, ["(1 meld result(s) are hidden; add --meld)"]);
    let j = e.notes(&["--text", "first strike, lifelink", "--json"]);
    assert_eq!(j.out, "[]\n");
    assert_eq!(j.stderr(), "(1 meld result(s) are hidden; add --meld)\n");
}

#[test]
fn ident_under_a_deck_narrows_and_never_widens() {
    let e = Env::new();
    let m = usage_err(e.find(&["--deck", "mono-white", "--ident", "U", "--count"]));
    assert_eq!(m, "--ident U is outside deck mono-white's identity W");
    assert!(
        usage_err(e.find(&["--deck", "mono-white", "--ident", "WU", "--count"])).contains("outside deck mono-white")
    );
    assert!(usage_err(e.find(&["--deck", "colorless", "--ident", "W"])).contains("identity C"));
    assert!(usage_err(e.find(&["--deck", "azorius", "--ident", "WUBRG"])).contains("--ident WUBRG is outside"));
    // narrowing works: W within WU; C within anything
    let w = e.json(&["--deck", "azorius", "--ident", "W", "--limit", "all"]);
    assert!(!w.is_empty() && w.iter().all(|c| ["", "W"].contains(&c["identity"].as_str().unwrap())));
    assert!(e.count(&["--deck", "azorius", "--ident", "C"]) < e.count(&["--deck", "azorius", "--ident", "W"]));
    assert_eq!(e.count(&["--deck", "azorius", "--ident", "WU"]), e.count(&["--deck", "azorius"]));
    // the error is a usage error, so the binary prints the usage text
    let out = bin()
        .arg("--root")
        .arg(&e.0)
        .args(["find", "--deck", "mono-white", "--ident", "U", "--count"])
        .output()
        .unwrap();
    assert!(!out.status.success() && String::from_utf8_lossy(&out.stderr).contains("usage:"));
}

#[test]
fn small_parser_rules() {
    let e = Env::new();
    // --type needs a word
    for v in ["", "  ", "—", "//"] {
        assert!(usage_err(e.find(&["--type", v])).contains("--type needs a word"), "{v:?}");
    }
    // --limit 0 is an error, --limit 1 is not
    assert!(usage_err(e.find(&["--limit", "0"])).contains("use --count"));
    assert_eq!(e.names(&["--limit", "1"]).len(), 1);
    // --name-contains does not trim: " of " keeps its spaces
    let decks = serde_json::json!({"mine": [], "pool": []});
    let c = Env::with_cards(vec![rec("Wrath of God", "a"), rec("Profit", "b"), rec("Sword of Fire", "c")], decks);
    assert_eq!(c.names(&["--name-contains", " of ", "--sort", "name"]), ["Sword of Fire", "Wrath of God"]);
    assert_eq!(c.names(&["--name-contains", "of", "--sort", "name"]), ["Profit", "Sword of Fire", "Wrath of God"]);
    // a regex error shows the user's pattern, not the (?i) prefix
    let m = failed_err(e.find(&["--text", "["]));
    assert!(m.contains("'['") && !m.contains("(?i)"), "{m}");
    // U+A789 folds to a colon; lower-casing happens after NFKD too
    assert_eq!(knowledge::fold("Ratonhnhaké\u{a789}ton"), "ratonhnhake:ton");
    assert_eq!(knowledge::fold("Foo\u{2122}"), "footm");
    assert_eq!(knowledge::fold_raw(" Of "), " of ");
}

#[test]
fn git_environment_does_not_redirect_the_worktree_fallback() {
    // a worktree-style fallback that must not follow a caller's GIT_DIR
    let e = Env::new();
    let main = e.0.join("m2");
    std::fs::create_dir_all(main.join("data/knowledge")).unwrap();
    std::fs::write(main.join("data/knowledge/sync.lock"), "schema=1\n").unwrap();
    git(&main, &["init", "-q"]);
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "x"]);
    std::fs::copy(fixture("find_oracle.json"), main.join("data/knowledge/oracle.json")).unwrap();
    let wt = e.0.join("wt2");
    git(&main, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    let out = bin()
        .env("GIT_DIR", e.0.join("elsewhere"))
        .env("GIT_COMMON_DIR", e.0.join("elsewhere"))
        .arg("--root")
        .arg(&wt)
        .args(["find", "--name", "Sol Ring", "--count"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1", "{}", String::from_utf8_lossy(&out.stderr));
    let _ = Command::new("git").arg("-C").arg(&main).args(["worktree", "remove", "--force"]).arg(&wt).output();
}
