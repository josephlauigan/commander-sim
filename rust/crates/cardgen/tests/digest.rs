//! `cardgen digest` and `cardgen map` on the 364 real-Oracle-text cards of `tests/fixtures/roles_oracle.json` plus a
//! few hand-made records (an Attraction, a Sticker, a banned and a not-legal card, a meld result, unranked cards, a
//! card with a `|` in its text and one whose fields leave little room), with a rules-only role table and a merged one.
//! Nothing uses the network or the LLM.

use cardgen::data::Project;
use cardgen::digest::{self, Digest, Prev, Review, Spec, Table};
use cardgen::knowledge::{self, FindError, Index, Legal};
use cardgen::llm;
use cardgen::map;
use cardgen::roles::{self, Assignment, Evidence, Source, Taxonomy};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const TAXONOMY: &str = include_str!("../../../../data/knowledge/roles.toml");
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/roles_oracle.json");
const LONG_NAME: &str = "Test Card With An Extraordinarily Long Name, Eater Of Many Worlds And Keeper Of Small Secrets";

fn card(name: &str, type_line: &str, text: &str, ident: &[&str], legal: &str, rank: Option<u32>) -> Value {
    let mut c = json!({
        "name": name, "mana_cost": "{1}{R}", "cmc": 2.0, "type_line": type_line, "oracle_text": text,
        "color_identity": ident, "keywords": [], "layout": "normal", "game_changer": false,
        "legalities": {"commander": legal}, "oracle_id": format!("test-{}", name.replace(' ', "-").to_lowercase()),
        "released_at": "2026-01-01", "set": "tst", "rarity": "common", "reserved": false
    });
    if let Some(r) = rank {
        c["edhrec_rank"] = json!(r);
    }
    c
}

/// the fixture records plus the hand-made ones
fn records() -> Vec<Value> {
    let mut v: Vec<Value> = serde_json::from_str::<Value>(&std::fs::read_to_string(FIXTURE).unwrap()).unwrap()["cards"]
        .as_array()
        .unwrap()
        .clone();
    let mut meld = card("Test Meld Result", "Legendary Creature — Eldrazi", "Flying", &["R"], "legal", Some(7));
    meld["all_parts_meld"] = json!(true);
    meld["meld_parts"] = json!(["Test A", "Test B"]);
    v.extend([
        card("Test Attraction", "Artifact — Attraction", "Visit — Draw a card.", &[], "legal", Some(5)),
        card("Test Sticker", "Stickers", "", &[], "legal", Some(6)),
        card("Test Banned Draw", "Instant", "Draw three cards.", &["U"], "banned", Some(1)),
        card("Test Not Legal Draw", "Instant", "Draw three cards.", &["U"], "not_legal", Some(2)),
        meld,
        card("Test Unranked Draw", "Sorcery", "Draw two cards.", &["R"], "legal", None),
        card("Test Unranked Removal", "Instant", "Destroy target creature.", &[], "legal", None),
        card("Test Pipe", "Instant", "Draw a card. Choose one | Gain 2 life.", &["R"], "legal", Some(99_000)),
        card("Test Vanilla", "Creature — Goblin", "", &["R"], "legal", Some(98_000)),
        card(
            LONG_NAME,
            "Legendary Creature — Elf Druid Warrior Wizard Shaman Cleric Knight Rogue Soldier Human Dragon",
            "Whenever another nontoken Dragon you control enters, draw two cards and create a 1/1 white Spirit \
             creature token with flying that is a copy of target creature you control, then destroy target creature.",
            &["R"],
            "legal",
            Some(97_000),
        ),
    ]);
    v
}

fn decks() -> Value {
    json!({"version": 1, "mine": [
        {"key": "dragons", "name": "Dragons", "commander": "Lathliss, Dragon Queen", "ident": "R", "tiers": ["t1"],
         "cards": ["Lightning Bolt", "Mogg Fanatic", "Thundermaw Hellkite", "Chromatic Lantern", "Goblin Bombardment",
                   "Mogg Fanatic", "Not In Catalog"]},
        {"key": "thanos", "name": "Thanos", "commander": "Thanos, the Mad Titan", "ident": "WUBRG", "tiers": ["t5"],
         "cards": ["Counterspell", "Demonic Tutor", "Cyclonic Rift", "Smothering Tithe"]},
        {"key": "duo", "name": "Duo", "commander": "Lathliss, Dragon Queen", "commander2": "Dragonlord Dromoka",
         "ident": "WUBRG", "tiers": ["t1"], "cards": ["Lightning Bolt", "Test Banned Draw"]},
    ], "pool": []})
}

struct Env {
    root: PathBuf,
    index: Index,
    table: Table,
}

impl Env {
    fn new() -> Env {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "digest-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(root.join("data/spellbook")).unwrap();
        let cards = records();
        let meta = json!({"schema": 1, "bulk_updated_at": "fixture-2026-11-21", "download_uri": "x", "synced": "x",
            "count": cards.len()});
        let doc = json!({"meta": meta, "cards": cards});
        std::fs::write(dir.join("oracle.json"), serde_json::to_string(&doc).unwrap()).unwrap();
        std::fs::write(root.join("data/decks.json"), decks().to_string()).unwrap();
        std::fs::write(
            root.join("data/spellbook/dragons.json"),
            json!({"deck": "dragons", "identity": "R", "fetched": "2026-11-04", "included": [{}, {}],
                   "included_by_changing_commanders": [{}], "almost_included": [{}, {}, {}]})
            .to_string(),
        )
        .unwrap();
        let mut env =
            Env { root, index: Index::load(&dir.join("oracle.json")).unwrap(), table: Table::of(&rules_only_meta()) };
        env.rules_only();
        env
    }

    fn dir(&self) -> PathBuf {
        self.root.join("data/knowledge")
    }

    fn project(&self) -> Project {
        Project::new(&self.root)
    }

    fn reload(&mut self) {
        self.index = Index::load(&self.dir().join("oracle.json")).unwrap();
    }

    /// the rules-only table, written to roles.json too
    fn rules_only(&mut self) -> roles::RoleMap {
        self.reload();
        let overrides = roles::read_overrides(&self.dir().join("oracle.json")).unwrap();
        let map = roles::generate(&self.index, &overrides).unwrap();
        std::fs::write(self.dir().join("roles.json"), roles::bytes(&map).unwrap()).unwrap();
        self.index.attach_roles(&map).unwrap();
        self.table = Table::of(&map.meta);
        map
    }

    fn id(&self, name: &str) -> String {
        self.index.cards[self.index.lookup(name)[0] as usize].raw.oracle_id.clone()
    }

    /// A merged table: `Skywhaler's Shot` answered by the LLM (`selection`, cited line 0), `Counterspell` routed and
    /// failed (so unreviewed), the rest as the rules left them.
    fn merged(&mut self) -> roles::RoleMap {
        let mut map = self.rules_only();
        map.meta.prompt_fnv = llm::prompt_fnv(&Taxonomy::parse(TAXONOMY).unwrap());
        map.meta.model = "test-model".into();
        let shot = self.id("Skywhaler's Shot");
        let a = map.get_mut(&shot).unwrap();
        a.roles = vec!["selection".into()];
        a.primary = "selection".into();
        a.evidence = [(
            "selection".to_string(),
            Evidence { source: Source::Llm, certain: false, rule: "llm".into(), face: None, line: Some(0) },
        )]
        .into();
        a.needs_llm = true;
        a.llm_failed = false;
        // the merge marks a routed card that has no answer `llm_failed`
        let counter = self.id("Counterspell");
        map.get_mut(&counter).unwrap().needs_llm = true;
        for (id, a) in map.iter_mut() {
            if a.needs_llm && *id != shot {
                a.llm_failed = true;
            }
        }
        self.index.attach_roles(&map).unwrap();
        self.table = Table::of(&map.meta);
        map
    }

    fn deck(&self, key: &str) -> knowledge::Deck {
        knowledge::load_deck(&self.project(), key).unwrap()
    }

    fn spec<'a>(&self, deck: Option<&'a knowledge::Deck>, prev: &'a Prev) -> Spec<'a> {
        Spec { deck, ident: None, role: None, limit: None, offset: 0, budget_tokens: None, prev }
    }

    fn digest(&self, key: &str) -> Digest {
        let d = self.deck(key);
        digest::build(&self.index, &self.table, &self.spec(Some(&d), &Prev::Missing)).unwrap()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn rules_only_meta() -> roles::RoleMeta {
    roles::RoleMeta {
        taxonomy: String::new(),
        rules_fnv: String::new(),
        oracle_fnv: String::new(),
        count: 0,
        prompt_fnv: String::new(),
        model: String::new(),
        table_fnv: String::new(),
    }
}

/// `(header line, card lines)` per `## ` block
fn blocks(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = vec![];
    for l in text.lines() {
        if l.starts_with("## ") {
            out.push((l.to_string(), vec![]));
        } else if !l.starts_with('#') && !l.is_empty() {
            out.last_mut().expect("a card line follows a header").1.push(l.to_string());
        }
    }
    out
}

fn card_lines(text: &str) -> Vec<String> {
    text.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).map(String::from).collect()
}

fn field(line: &str, n: usize) -> String {
    line.split('|').nth(n).unwrap().to_string()
}

fn names(lines: &[String]) -> Vec<String> {
    lines.iter().map(|l| field(l, 0)).collect()
}

fn section<'a>(text: &'a str, id: char) -> Vec<String> {
    blocks(text).into_iter().filter(|(h, _)| h.starts_with(&format!("## {id} "))).flat_map(|(_, l)| l).collect()
}

fn ident_of(env: &Env, name: &str) -> u8 {
    let found = env.index.lookup(name);
    assert_eq!(found.len(), 1, "{name}");
    env.index.cards[found[0] as usize].ident
}

fn usage(e: FindError) -> String {
    match e {
        FindError::Usage(m) => m,
        other => panic!("expected a usage error, got {other:?}"),
    }
}

// ------------------------------------------------------------------------------------------------ the line

#[test]
fn every_line_has_six_fields_at_most_160_characters_and_no_stray_bar() {
    let env = Env::new();
    for key in ["dragons", "thanos", "duo"] {
        let d = env.digest(key);
        let lines = card_lines(&d.text);
        assert!(lines.len() > 50, "{key}: {}", lines.len());
        assert_eq!(lines.len(), d.lines);
        for l in &lines {
            assert_eq!(l.split('|').count(), 6, "{l}");
            assert!(l.chars().count() <= 160, "{} chars: {l}", l.chars().count());
            // the flags are the known ones, in order: * GC #rank $usd r:status
            let flags = field(l, 5);
            let mut parts = flags.split(' ').peekable();
            let mut rest: Vec<&str> = parts.by_ref().collect();
            let status = rest.pop().unwrap();
            assert!(["r:rules", "r:llm", "r:uncertain", "r:none"].contains(&status), "{l}");
            for (i, f) in rest.iter().enumerate() {
                let ok = *f == "*" && i == 0
                    || *f == "GC"
                    || f.strip_prefix('#').is_some_and(|n| n.parse::<u32>().is_ok())
                    || f.strip_prefix('$').is_some_and(|n| n.parse::<u32>().is_ok());
                assert!(ok, "bad flag {f} in {l}");
            }
        }
    }
}

#[test]
fn a_long_name_with_many_subtypes_still_fits_and_keeps_its_fields() {
    let env = Env::new();
    // the card is a Dragon, so the Dragons deck lists it (section C or B); find its line wherever it is
    let d = env.digest("dragons");
    let all = card_lines(&env.digest("dragons").text);
    let mut with_role = None;
    // a role page lists every card of a role: use the one the card has
    let id = env.index.by_oracle_id(&env.id(LONG_NAME)).unwrap();
    let a = env.index.roles(id).unwrap().clone();
    if !a.primary.is_empty() {
        let deck = env.deck("dragons");
        let spec = Spec { role: Some(a.primary.as_str()), ..env.spec(Some(&deck), &Prev::Missing) };
        let page = digest::build(&env.index, &env.table, &spec).unwrap();
        with_role = card_lines(&page.text).into_iter().find(|l| l.starts_with(LONG_NAME));
    }
    let line = all.iter().find(|l| l.starts_with(LONG_NAME)).cloned().or(with_role);
    let line = line.unwrap_or_else(|| panic!("the long card is listed nowhere: {} lines", d.lines));
    assert_eq!(line.split('|').count(), 6);
    assert!(line.chars().count() <= 160, "{}: {line}", line.chars().count());
    // the subtypes gave way before the gist or the name did, and some gist is left
    assert_eq!(field(&line, 2), "LgC", "{line}");
    assert!(field(&line, 4).starts_with("Whenever another"), "{line}");
}

#[test]
fn type_abbreviations() {
    let env = Env::new();
    let abbrev = |n: &str| digest::type_abbrev(&env.index.cards[env.index.lookup(n)[0] as usize]);
    assert_eq!(abbrev("Chromatic Lantern"), "A");
    assert_eq!(abbrev("Lightning Bolt"), "I");
    assert_eq!(abbrev("Lathliss, Dragon Queen"), "LgC");
    assert_eq!(abbrev("Moltensteel Dragon"), "AC");
    assert_eq!(abbrev("Rhystic Study"), "E");
    assert_eq!(abbrev("Ash Barrens"), "L");
    assert_eq!(abbrev("Time Warp"), "S");
    assert_eq!(abbrev("Liliana of the Veil"), "LgP");
    // a card of several faces has the types of every face
    assert_eq!(abbrev("Agadeem's Awakening // Agadeem, the Undercrypt"), "LS");
}

#[test]
fn cut_is_a_word_boundary_prefix_and_never_longer_than_asked() {
    let s = "Whenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature token with flying.";
    for max in 2..s.len() + 3 {
        let c = digest::cut(s, max);
        assert!(c.chars().count() <= max.max(1), "{max}: {c}");
        let body = c.strip_suffix('…').unwrap_or(&c);
        assert!(s.starts_with(body), "{max}: {c}");
        if c.ends_with('…') {
            // cut at a word boundary: the next character of the source is a space, or the cut is at a space
            let rest = &s[body.len()..];
            assert!(rest.starts_with(' ') || rest.is_empty() || !body.contains(' '), "{max}: {c}");
        } else {
            assert_eq!(c, s);
        }
    }
    assert_eq!(digest::cut("short", 110), "short");
}

#[test]
fn a_gist_is_a_cut_of_a_real_oracle_line_and_a_bar_in_the_text_becomes_a_slash() {
    let env = Env::new();
    let d = env.digest("thanos");
    let mut seen = 0;
    for l in card_lines(&d.text) {
        let gist = field(&l, 4);
        if gist == "-" {
            continue;
        }
        let name = field(&l, 0);
        let found = env.index.lookup(&name);
        let c = &env.index.cards[found[0] as usize];
        let body = gist.strip_suffix('…').unwrap_or(&gist);
        let ok = c.lines().iter().any(|(_, line)| roles::normalize(c, line).replace('|', "/").starts_with(body));
        assert!(ok, "gist of {name} is no cut of an Oracle line: {gist}");
        seen += 1;
    }
    assert!(seen > 100, "{seen}");
    // the pipe card: its line still has six fields, the bar of the text is a slash
    let pipe = digest::Lines::new(&env.index, &env.table).line(env.index.lookup("Test Pipe")[0], false);
    assert_eq!(pipe.split('|').count(), 6, "{pipe}");
    assert!(field(&pipe, 4).contains("Choose one / Gain 2 life"), "{pipe}");
}

#[test]
fn the_gist_is_the_line_cited_for_the_primary_role_and_the_first_line_when_none_is_cited() {
    let mut env = Env::new();
    // a hand override has no cited line: the first Oracle line stands in
    std::fs::write(
        env.dir().join("roles_overrides.toml"),
        "[[card]]\nname = 'Counterspell'\nroles = ['draw']\nnote = 'n'\n",
    )
    .unwrap();
    env.rules_only();
    let id = env.index.lookup("Counterspell")[0];
    let c = &env.index.cards[id as usize];
    let a = env.index.roles(id).unwrap();
    assert_eq!(a.primary, "draw");
    assert!(a.evidence["draw"].line.is_none());
    let lines = digest::Lines::new(&env.index, &env.table);
    let l = lines.line(id, false);
    assert_eq!(field(&l, 4), "Counter target spell.");
    assert_eq!(field(&l, 3), "draw");
    assert!(field(&l, 5).ends_with("r:rules"), "{l}");
    // a rule role cites its line: Force of Will's counter role cites the second line, not the first (alternate cost)
    let fow = env.index.lookup("Force of Will");
    if let Some(&i) = fow.first() {
        let a = env.index.roles(i).unwrap();
        let line = a.evidence[&a.primary].line.unwrap();
        let l = lines.line(i, false);
        let want = digest::cut(&roles::line_text(&env.index.cards[i as usize], line).replace('|', "/"), 110);
        assert!(field(&l, 4).starts_with(want.trim_end_matches('…')), "{l}");
    }
    let _ = c;
    // a card with no Oracle text has no gist
    let vanilla = env.index.lookup("Test Vanilla")[0];
    assert_eq!(field(&lines.line(vanilla, false), 4), "-");
}

// ------------------------------------------------------------------------------------------------ review flags

#[test]
fn the_review_flag_of_each_kind_of_assignment() {
    let a = |needs_llm: bool, failed: bool, roles: &[&str]| Assignment {
        roles: roles.iter().map(|r| r.to_string()).collect(),
        primary: roles.first().map(|r| r.to_string()).unwrap_or_default(),
        needs_llm,
        llm_failed: failed,
        ..Assignment::default()
    };
    // rules alone, rules-only table and merged table alike
    assert_eq!(digest::review(&a(false, false, &["ramp"]), false), Review::Rules);
    assert_eq!(digest::review(&a(false, false, &["ramp"]), true), Review::Rules);
    // no roles and not routed
    assert_eq!(digest::review(&a(false, false, &[]), false), Review::None);
    assert_eq!(digest::review(&a(false, false, &[]), true), Review::None);
    // routed: no answer in a rules-only table, an answer in a merged one unless the merge marked it failed
    assert_eq!(digest::review(&a(true, false, &["ramp"]), false), Review::Uncertain);
    assert_eq!(digest::review(&a(true, false, &[]), false), Review::Uncertain);
    assert_eq!(digest::review(&a(true, false, &["ramp"]), true), Review::Llm);
    assert_eq!(digest::review(&a(true, false, &[]), true), Review::Llm);
    assert_eq!(digest::review(&a(true, true, &["ramp"]), true), Review::Uncertain);
    assert_eq!(Review::Rules.flag(), "r:rules");
    assert_eq!(Review::Llm.flag(), "r:llm");
    assert_eq!(Review::Uncertain.flag(), "r:uncertain");
    assert_eq!(Review::None.flag(), "r:none");
}

#[test]
fn flags_on_real_lines_in_a_rules_only_table_and_in_a_merged_one() {
    let mut env = Env::new();
    let flag_of = |env: &Env, name: &str| {
        let i = env.index.lookup(name)[0];
        let l = digest::Lines::new(&env.index, &env.table).line(i, false);
        field(&l, 5).split(' ').next_back().unwrap().to_string()
    };
    // rules-only: every routed card is uncertain, the others rules or none
    let (mut rules, mut uncertain, mut none) = (0, 0, 0);
    for (i, c) in env.index.cards.iter().enumerate() {
        if c.legal != Legal::Legal || !roles::is_deck_card(c) {
            continue;
        }
        let want = match env.index.roles(i as u32).unwrap() {
            a if a.needs_llm => Review::Uncertain,
            a if a.roles.is_empty() => Review::None,
            _ => Review::Rules,
        };
        let l = digest::Lines::new(&env.index, &env.table).line(i as u32, false);
        assert!(field(&l, 5).ends_with(want.flag()), "{l}");
        match want {
            Review::Rules => rules += 1,
            Review::Uncertain => uncertain += 1,
            _ => none += 1,
        }
    }
    assert!(rules > 10 && uncertain > 10, "{rules} {uncertain} {none}");
    assert_eq!(flag_of(&env, "Skywhaler's Shot"), "r:uncertain");
    assert_eq!(env.table.kind(), "rules-only");
    // merged: the answered card is r:llm, the failed one stays uncertain, a rules card stays r:rules
    env.merged();
    assert_eq!(env.table.kind(), "merged (model test-model)");
    assert_eq!(flag_of(&env, "Skywhaler's Shot"), "r:llm");
    assert_eq!(flag_of(&env, "Counterspell"), "r:uncertain");
    let decided = env
        .index
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| {
            c.legal == Legal::Legal
                && !env.index.roles(*i as u32).unwrap().needs_llm
                && !env.index.roles(*i as u32).unwrap().roles.is_empty()
        })
        .map(|(_, c)| c.raw.name.clone())
        .expect("a card the rules decided alone");
    assert_eq!(flag_of(&env, &decided), "r:rules");
    // the header names the table
    let text = env.digest("dragons").text;
    assert!(text.contains("role table: merged (model test-model)"), "{}", &text[..300]);
}

// ------------------------------------------------------------------------------------------------ sections

#[test]
fn section_a_is_exactly_the_decks_own_cards_once_each_and_marked() {
    let env = Env::new();
    let d = env.digest("dragons");
    let a = section(&d.text, 'A');
    let got: BTreeSet<String> = names(&a).into_iter().collect();
    let want: BTreeSet<String> = [
        "Lightning Bolt",
        "Mogg Fanatic",
        "Thundermaw Hellkite",
        "Chromatic Lantern",
        "Goblin Bombardment",
        "Lathliss, Dragon Queen",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(got, want, "a repeated card once, the commander included, an unknown name not at all");
    assert_eq!(a.len(), want.len());
    for l in &a {
        assert!(field(l, 5).starts_with('*'), "{l}");
    }
    // nothing else carries the mark, and none of the deck's cards is listed again
    let others: Vec<String> = ['B', 'C', 'D', 'E'].iter().flat_map(|&s| section(&d.text, s)).collect();
    assert!(others.iter().all(|l| !field(l, 5).starts_with('*')));
    assert!(names(&others).iter().all(|n| !want.contains(n)), "a deck card is listed outside A");
    // the name the catalog lacks is reported
    assert!(d.text.contains("deck names not in the catalog (1): Not In Catalog"), "{}", &d.text[..800]);
    // sorted by EDHREC rank
    let ranks: Vec<u32> =
        a.iter().map(|l| field(l, 5).split(' ').find_map(|f| f.strip_prefix('#')?.parse().ok()).unwrap()).collect();
    assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{ranks:?}");
}

#[test]
fn only_legal_deck_cards_of_the_identity_never_a_sticker_an_attraction_or_a_meld_result() {
    let env = Env::new();
    for (key, limit) in [("dragons", 0b01000u8), ("thanos", 0b11111), ("duo", 0b11111)] {
        let d = env.digest(key);
        for l in card_lines(&d.text) {
            let name = field(&l, 0);
            let found = env.index.lookup(&name);
            let c = &env.index.cards[found[0] as usize];
            // only the deck's own list (section A) may name a card that is not Commander-legal
            assert!(c.legal == Legal::Legal || field(&l, 5).starts_with('*'), "{name}");
            assert!(roles::is_deck_card(c), "{name}");
            assert!(!c.is_meld(), "{name}");
            assert_eq!(c.ident & !limit, 0, "{name} is outside the identity of {key}");
        }
        for banned in ["Test Attraction", "Test Sticker", "Test Not Legal Draw", "Test Meld Result"] {
            assert!(!d.text.contains(&format!("\n{banned}|")), "{banned} is in the {key} digest");
        }
    }
    // an off-identity card of the fixture really is absent from the red digest
    let red = env.digest("dragons").text;
    assert!(!red.contains("\nCounterspell|") && !red.contains("\nRhystic Study|"));
    assert!(ident_of(&env, "Counterspell") != 0);
}

#[test]
fn a_deck_card_that_is_not_commander_legal_is_still_section_a_and_the_header_says_so() {
    let env = Env::new();
    let d = env.digest("duo");
    let a = section(&d.text, 'A');
    let banned: Vec<&String> = a.iter().filter(|l| l.starts_with("Test Banned Draw|")).collect();
    assert_eq!(banned.len(), 1, "{a:?}");
    assert!(d.text.contains("# deck cards the catalog does not list as Commander-legal (1): Test Banned Draw"));
    // nowhere else, and a legal deck has no such line
    assert_eq!(card_lines(&d.text).iter().filter(|l| l.starts_with("Test Banned Draw|")).count(), 1);
    assert!(!env.digest("dragons").text.contains("does not list as Commander-legal"));
}

#[test]
fn section_b_lists_a_card_once_under_its_primary_role_best_ranked_first_and_never_a_deck_card() {
    let env = Env::new();
    let d = env.digest("thanos");
    let mut seen: HashSet<String> = HashSet::new();
    let mut roles_seen = 0;
    for (h, lines) in blocks(&d.text).into_iter().filter(|(h, _)| h.starts_with("## B ")) {
        let role = h.split_whitespace().nth(3).unwrap().trim_end_matches(':').to_string();
        roles_seen += 1;
        let mut ranks = vec![];
        for l in &lines {
            let name = field(l, 0);
            assert!(seen.insert(name.clone()), "{name} is listed twice in B");
            let i = env.index.lookup(&name)[0];
            assert_eq!(env.index.roles(i).unwrap().primary, role, "{name} under {role}");
            assert!(field(l, 3).starts_with(&role), "primary role first: {l}");
            ranks.push(env.index.cards[i as usize].raw.edhrec_rank.expect("B is ranked cards"));
        }
        assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{role}: {ranks:?}");
        assert!(lines.len() <= 150);
    }
    assert!(roles_seen >= 10, "{roles_seen}");
    let deck = env.deck("thanos");
    for n in &deck.cards {
        assert!(!seen.contains(n));
    }
}

#[test]
fn section_c_lists_the_cards_that_share_a_subtype_the_commanders_text_names() {
    let env = Env::new();
    let deck = env.deck("dragons");
    assert_eq!(digest::commander_subtypes(&env.index, &deck), ["Dragon"]);
    let d = env.digest("dragons");
    let c = section(&d.text, 'C');
    assert!(!c.is_empty());
    let want: Vec<&str> = ["Moltensteel Dragon", "Dragon Tempest", "Thundermaw Hellkite", LONG_NAME].to_vec();
    let listed = names(&c);
    // Thundermaw Hellkite is in the deck, the others are red or colourless Dragon cards or name Dragons
    for n in &want[..2] {
        assert!(listed.contains(&n.to_string()), "{n} missing from C: {listed:?}");
    }
    assert!(!listed.contains(&"Thundermaw Hellkite".to_string()), "a deck card is not section C");
    assert!(listed.contains(&LONG_NAME.to_string()), "a card whose text names Dragon is synergy too");
    // every C card has the subtype in its type line or names it in its text
    for l in &c {
        let i = env.index.lookup(&field(l, 0))[0];
        let card = &env.index.cards[i as usize];
        let text: String = card.lines().iter().map(|(_, l)| roles::normalize(card, l)).collect::<Vec<_>>().join(" ");
        assert!(card.subtypes().contains(&"Dragon") || text.contains("Dragon"), "{l}");
    }
    // Thanos's text names no subtype
    let t = env.digest("thanos");
    assert!(t.text.contains("## C synergy: none"), "no subtype named");
    assert!(section(&t.text, 'C').is_empty());
    // two commanders: the subtypes of both texts
    let duo = env.deck("duo");
    let subs = digest::commander_subtypes(&env.index, &duo);
    assert!(subs.contains(&"Dragon".to_string()), "{subs:?}");
    // a plural names it too, `Time Lord` is one subtype
    assert!(digest::commander_subtypes(&env.index, &env.deck("thanos")).is_empty());
}

#[test]
fn section_d_is_the_set_difference_with_the_previous_sync_and_is_omitted_without_one() {
    let env = Env::new();
    let deck = env.deck("thanos");
    let missing = digest::build(&env.index, &env.table, &env.spec(Some(&deck), &Prev::Missing)).unwrap();
    assert!(missing.text.contains("# D omitted: no oracle.prev.json"), "{}", &missing.text[..600]);
    assert!(!missing.text.contains("\n## D "));
    // the previous sync lacked these three
    let new = ["Rhystic Study", "Cyclonic Rift", "Mana Drain"];
    let prev: HashSet<String> = env
        .index
        .cards
        .iter()
        .filter(|c| !new.contains(&c.raw.name.as_str()))
        .map(|c| c.raw.oracle_id.clone())
        .collect();
    let with = digest::build(&env.index, &env.table, &env.spec(Some(&deck), &Prev::Ids(prev))).unwrap();
    let d = section(&with.text, 'D');
    // Cyclonic Rift is in the Thanos deck: left out; the other two are new, in identity and not in the deck
    let got: BTreeSet<String> = names(&d).into_iter().collect();
    assert_eq!(got, ["Rhystic Study", "Mana Drain"].map(String::from).into_iter().collect::<BTreeSet<_>>());
    let ranks: Vec<u32> =
        names(&d).iter().map(|n| env.index.cards[env.index.lookup(n)[0] as usize].raw.edhrec_rank.unwrap()).collect();
    assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "{ranks:?}");
    assert!(with.text.contains("## D new since the previous sync: 2 cards"));
    // an unreadable previous sync is a note, not a failure
    let bad =
        digest::build(&env.index, &env.table, &env.spec(Some(&deck), &Prev::Unreadable("x: broken".into()))).unwrap();
    assert!(bad.text.contains("# D omitted: oracle.prev.json is unreadable (x: broken)"));
    // `Prev::load` reads a file of the catalog's shape and skips what it does not need
    let dir = env.dir();
    assert!(matches!(Prev::load(&dir), Prev::Missing));
    std::fs::write(
        dir.join("oracle.prev.json"),
        r#"{"meta":{},"cards":[{"oracle_id":"a","name":"A"},{"oracle_id":"b"}]}"#,
    )
    .unwrap();
    match Prev::load(&dir) {
        Prev::Ids(ids) => assert_eq!(ids.len(), 2),
        _ => panic!("a readable file"),
    }
    std::fs::write(dir.join("oracle.prev.json"), "not json").unwrap();
    assert!(matches!(Prev::load(&dir), Prev::Unreadable(_)));
}

#[test]
fn section_e_is_the_unranked_cards_per_role_and_a_card_in_c_or_d_is_not_repeated() {
    let env = Env::new();
    let d = env.digest("dragons");
    let e = section(&d.text, 'E');
    let listed = names(&e);
    assert!(listed.contains(&"Test Unranked Draw".to_string()), "{listed:?}");
    assert!(listed.contains(&"Test Unranked Removal".to_string()), "colourless fits every deck: {listed:?}");
    for l in &e {
        let i = env.index.lookup(&field(l, 0))[0];
        assert!(env.index.cards[i as usize].raw.edhrec_rank.is_none(), "{l}");
        assert!(!field(l, 5).contains('#'), "no rank flag on an unranked card: {l}");
    }
    let c: HashSet<String> = names(&section(&d.text, 'C')).into_iter().collect();
    assert!(listed.iter().all(|n| !c.contains(n)));
}

#[test]
fn an_identity_without_a_deck_has_no_a_or_c_and_may_narrow_a_deck_but_not_widen_it() {
    let env = Env::new();
    let spec = Spec { ident: Some(0b00011), ..env.spec(None, &Prev::Missing) }; // WU
    let d = digest::build(&env.index, &env.table, &spec).unwrap();
    assert!(d.text.contains("# digest: identity WU (no deck)"));
    assert!(section(&d.text, 'A').is_empty() && section(&d.text, 'C').is_empty());
    for l in card_lines(&d.text) {
        let c = &env.index.cards[env.index.lookup(&field(&l, 0))[0] as usize];
        assert_eq!(c.ident & !0b00011, 0, "{l}");
    }
    assert!(d.lines > 50);
    // narrowing: the Thanos deck under U
    let deck = env.deck("thanos");
    let narrow =
        digest::build(&env.index, &env.table, &Spec { ident: Some(0b00010), ..env.spec(Some(&deck), &Prev::Missing) })
            .unwrap();
    assert!(narrow.text.contains("identity WUBRG narrowed to U"), "{}", &narrow.text[..200]);
    for l in card_lines(&narrow.text).iter().filter(|l| !field(l, 5).starts_with('*')) {
        let c = &env.index.cards[env.index.lookup(&field(l, 0))[0] as usize];
        assert_eq!(c.ident & !0b00010, 0, "{l}");
    }
    // widening is a usage error
    let red = env.deck("dragons");
    let e =
        digest::build(&env.index, &env.table, &Spec { ident: Some(0b00010), ..env.spec(Some(&red), &Prev::Missing) });
    assert!(usage(e.unwrap_err()).contains("outside deck dragons"));
}

// ------------------------------------------------------------------------------------------------ budget and pages

#[test]
fn a_small_budget_trims_section_b_proportionally_and_says_so() {
    let env = Env::new();
    let deck = env.deck("thanos");
    let full = digest::build(&env.index, &env.table, &env.spec(Some(&deck), &Prev::Missing)).unwrap();
    assert!(full.b_caps.is_none(), "the fixture is small: nothing to trim");
    let full_tokens = digest::tokens(&full.text);
    let budget = full_tokens * 6 / 10;
    let spec = Spec { budget_tokens: Some(budget), ..env.spec(Some(&deck), &Prev::Missing) };
    let small = digest::build(&env.index, &env.table, &spec).unwrap();
    assert!(digest::tokens(&small.text) <= budget, "{} > {budget}", digest::tokens(&small.text));
    let (per_role, top) = small.b_caps.expect("trimmed");
    assert!(per_role < 60 && top < 150 && top >= per_role, "{per_role} {top}");
    assert!(small.text.contains(&format!("B caps trimmed to {per_role} per role and {top} for the most common roles")));
    assert!(small.text.contains("# budget: "));
    // A and C are not cut; B is
    assert_eq!(section(&small.text, 'A').len(), section(&full.text, 'A').len());
    assert!(section(&small.text, 'B').len() < section(&full.text, 'B').len());
    // the trim is the largest one that fits: one more step would not
    let bigger = digest::build(
        &env.index,
        &env.table,
        &Spec { budget_tokens: Some(budget + 40), ..env.spec(Some(&deck), &Prev::Missing) },
    )
    .unwrap();
    assert!(section(&bigger.text, 'B').len() >= section(&small.text, 'B').len());
    // an impossible budget keeps A, C and D whole and says so
    let tiny = digest::build(
        &env.index,
        &env.table,
        &Spec { budget_tokens: Some(50), ..env.spec(Some(&deck), &Prev::Missing) },
    )
    .unwrap();
    assert!(tiny.over_budget && tiny.text.contains("still over the budget"));
    assert!(section(&tiny.text, 'B').is_empty());
    assert_eq!(section(&tiny.text, 'A').len(), section(&full.text, 'A').len());
}

#[test]
fn a_role_page_lists_that_roles_cards_and_pages_have_no_gaps_or_overlaps() {
    let env = Env::new();
    let deck = env.deck("thanos");
    let spec = |role, limit, offset, budget| Spec {
        role: Some(role),
        limit,
        offset,
        budget_tokens: budget,
        ..env.spec(Some(&deck), &Prev::Missing)
    };
    let all = digest::build(&env.index, &env.table, &spec("removal", None, 0, None)).unwrap();
    let total = all.total.unwrap();
    assert!(total > 20);
    assert_eq!(all.lines, total, "one page holds them all");
    assert!(all.next_offset.is_none());
    let want = names(&card_lines(&all.text));
    // exactly the non-deck cards with this primary role, in the order of the sort
    for n in &want {
        let i = env.index.lookup(n)[0];
        assert_eq!(env.index.roles(i).unwrap().primary, "removal");
        assert!(!deck.cards.contains(n));
    }
    assert!(blocks(&all.text).len() == 1 && blocks(&all.text)[0].0.starts_with("## B role removal: cards 1-"));
    // pages of 7 by limit
    let mut got: Vec<String> = vec![];
    let mut offset = 0;
    loop {
        let p = digest::build(&env.index, &env.table, &spec("removal", Some(7), offset, None)).unwrap();
        got.extend(names(&card_lines(&p.text)));
        match p.next_offset {
            Some(n) => {
                assert_eq!(n, offset + 7);
                assert!(p.text.contains(&format!("next page: --offset {n}")));
                offset = n;
            }
            None => break,
        }
    }
    assert_eq!(got, want);
    // pages cut by tokens (a small budget) chain the same way
    let mut got: Vec<String> = vec![];
    let mut offset = 0;
    let mut pages = 0;
    loop {
        let p = digest::build(&env.index, &env.table, &spec("removal", None, offset, Some(700))).unwrap();
        assert!(p.lines >= 1);
        pages += 1;
        got.extend(names(&card_lines(&p.text)));
        match p.next_offset {
            Some(n) => offset = n,
            None => break,
        }
    }
    assert!(pages > 1, "a 700-token page cannot hold {total} cards");
    assert_eq!(got, want);
    // past the end: an empty page, not an error
    let end = digest::build(&env.index, &env.table, &spec("removal", None, total + 5, None)).unwrap();
    assert_eq!(end.lines, 0);
    assert!(end.text.contains("no cards at offset"));
    // a page is at most 30k tokens by default
    assert!(digest::tokens(&all.text) <= digest::ROLE_PAGE_TOKENS);
    // an unknown role and paging without a role are usage errors
    assert!(
        usage(digest::build(&env.index, &env.table, &spec("bogus", None, 0, None)).unwrap_err())
            .contains("unknown role")
    );
    let e = digest::build(&env.index, &env.table, &Spec { limit: Some(5), ..env.spec(Some(&deck), &Prev::Missing) });
    assert!(usage(e.unwrap_err()).contains("--role"));
}

#[test]
fn digests_are_deterministic() {
    let env = Env::new();
    for key in ["dragons", "thanos", "duo"] {
        assert_eq!(env.digest(key).text, env.digest(key).text, "{key}");
    }
    // a second process-independent build from a fresh index
    let again = Env::new();
    assert_eq!(env.digest("thanos").text, again.digest("thanos").text);
}

// ------------------------------------------------------------------------------------------------ the map

#[test]
fn the_map_has_every_section_fits_32_kb_and_is_deterministic() {
    let env = Env::new();
    let spell = env.root.join("data/spellbook");
    let text = map::build(&env.index, &env.table, Some(&spell)).unwrap();
    assert!(text.len() < map::MAX_BYTES, "{}", text.len());
    assert_eq!(text, map::build(&env.index, &env.table, Some(&spell)).unwrap());
    for h in [
        "# Card catalog map",
        "## Colour identities",
        "## Roles by colour",
        "## Role coverage and review status",
        "## Creature types",
        "## Subtypes of every kind",
        "## Keywords",
        "## Game Changers",
        "## Banned in Commander (1)",
        "## Combos known per deck",
        "## How to use",
    ] {
        assert!(text.contains(h), "missing {h}");
    }
    assert!(text.contains("fixture-2026-11-21"), "the bulk date, no clock");
    assert!(text.contains("Test Banned Draw"));
    assert!(text.contains("| dragons | R | 2026-11-04 | 2 | 1 | 3 |"), "{text}");
    for cmd in ["cardgen digest --deck KEY", "--role removal", "cardgen find --name \"Sol Ring\" --full"] {
        assert!(text.contains(cmd), "footer lacks {cmd}");
    }
    // counts: legal, banned, not legal, and what find/digest cover
    let legal = env.index.cards.iter().filter(|c| c.legal == Legal::Legal).count();
    let all = env.index.cards.len();
    assert!(
        text.contains(&format!("{all} cards: {legal} Commander-legal, 1 banned, 1 not legal;")),
        "{}",
        &text[..300]
    );
    // the meld result, Attractions and Stickers are legal but not covered
    let covered =
        env.index.cards.iter().filter(|c| c.legal == Legal::Legal && roles::is_deck_card(c) && !c.is_meld()).count();
    assert!(covered + 3 <= legal);
    assert!(text.contains(&format!("cover {covered} legal cards")), "{}", &text[..500]);
    // the identity table sums to the covered cards
    let sum: usize = text
        .lines()
        .skip_while(|l| !l.starts_with("## Colour identities"))
        .skip(4)
        .take_while(|l| l.starts_with('|'))
        .map(|l| l.split('|').nth(2).unwrap().trim().replace(',', "").parse::<usize>().unwrap())
        .sum();
    assert_eq!(sum, covered);
}

#[test]
fn map_coverage_counts_match_the_table_for_a_rules_only_and_a_merged_role_table() {
    let mut env = Env::new();
    let covered = |c: &knowledge::Card| c.legal == Legal::Legal && roles::is_deck_card(c) && !c.is_meld();
    let routed = |env: &Env| {
        env.index
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| covered(c))
            .filter(|(i, _)| env.index.roles(*i as u32).unwrap().needs_llm)
            .count()
    };
    let n = routed(&env);
    let text = map::build(&env.index, &env.table, None).unwrap();
    assert!(text.contains("Role table: rules-only."));
    assert!(
        text.contains(&format!(
            "{n} cards need the LLM (`needs_llm`): 0 have an LLM answer merged (`r:llm`), {n} are unreviewed"
        )),
        "{text}"
    );
    assert!(text.contains("No data/spellbook/ directory."));
    // merged: one card answered, one failed
    env.merged();
    let n = routed(&env);
    let text = map::build(&env.index, &env.table, None).unwrap();
    assert!(text.contains("Role table: merged (model test-model)."));
    assert!(
        text.contains(&format!(
            "{n} cards need the LLM (`needs_llm`): 1 have an LLM answer merged (`r:llm`), {} are unreviewed",
            n - 1
        )),
        "{text}"
    );
    // the roles table: a card with the role counts once per colour column, and the no-role row balances
    let covered_n = env.index.cards.iter().filter(|c| covered(c)).count();
    let none: usize = text
        .lines()
        .find(|l| l.starts_with("| (no role)"))
        .map(|l| l.split('|').nth(9).unwrap().trim().parse().unwrap())
        .unwrap();
    let with_role = covered_n - none;
    assert!(
        text.contains(&format!("{with_role} of {covered_n} cards have at least one role; {none} have none")),
        "{text}"
    );
}

#[test]
fn the_top_lists_count_legal_cards_and_time_lord_is_one_subtype() {
    let env = Env::new();
    let text = map::build(&env.index, &env.table, None).unwrap();
    let line_after = |h: &str| text.lines().skip_while(|l| !l.starts_with(h)).nth(2).unwrap().to_string();
    let types = line_after("## Creature types");
    assert!(types.starts_with("Human ") || types.starts_with("Goblin ") || types.contains(", "), "{types}");
    let counts: Vec<usize> = types.split(", ").map(|e| e.rsplit(' ').next().unwrap().parse().unwrap()).collect();
    assert!(counts.windows(2).all(|w| w[0] >= w[1]), "biggest first: {types}");
    assert!(counts.len() <= 40);
    let kw = line_after("## Keywords");
    assert!(kw.contains("Flying "), "{kw}");
    assert!(!kw.is_empty());
}

// ------------------------------------------------------------------------------------------------ the commands

#[test]
fn clean_errors_name_the_command_to_run() {
    let env = Env::new();
    let project = env.project();
    let dir = env.dir();
    let args = |a: &[&str]| -> Vec<String> {
        a.iter().map(|s| s.to_string()).chain(["--data-dir".to_string(), dir.display().to_string()]).collect()
    };
    // a missing role table
    std::fs::remove_file(dir.join("roles.json")).unwrap();
    match digest::command(&project, &args(&["--deck", "dragons"])) {
        Err(FindError::Failed(m)) => assert!(m.contains("cardgen roles --rules"), "{m}"),
        other => panic!("{other:?}"),
    }
    let e = map::command(&project, &args(&[])).unwrap_err();
    assert!(e.contains("cardgen roles --rules"), "{e}");
    // a missing catalog
    let empty = env.root.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let a: Vec<String> = ["--deck", "dragons", "--data-dir", empty.to_str().unwrap()].map(String::from).to_vec();
    match digest::command(&project, &a) {
        Err(FindError::Failed(m)) => assert!(m.contains("cardgen bulk-sync"), "{m}"),
        other => panic!("{other:?}"),
    }
    let a: Vec<String> = ["--data-dir", empty.to_str().unwrap()].map(String::from).to_vec();
    assert!(map::command(&project, &a).unwrap_err().contains("cardgen bulk-sync"));
    // a stale table
    let mut env = env;
    env.rules_only();
    std::fs::write(
        env.dir().join("roles_overrides.toml"),
        "[[card]]\nname = 'Counterspell'\nroles = ['draw']\nnote = 'n'\n",
    )
    .unwrap();
    match digest::command(&env.project(), &args(&["--deck", "dragons"])) {
        Err(FindError::Failed(m)) => assert!(m.contains("cardgen roles --rules"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn usage_errors() {
    let env = Env::new();
    let project = env.project();
    let dir = env.dir();
    let run = |a: &[&str]| {
        let mut v: Vec<String> = a.iter().map(|s| s.to_string()).collect();
        v.extend(["--data-dir".into(), dir.display().to_string()]);
        digest::command(&project, &v)
    };
    for (args, want) in [
        (vec![], "needs --deck KEY or --ident"),
        (vec!["--deck", "dragons", "--limit", "5"], "add --role R"),
        (vec!["--deck", "dragons", "--offset", "5"], "add --role R"),
        (vec!["--deck", "dragons", "--role", "bogus"], "unknown role"),
        (vec!["--deck", "dragons", "--bogus"], "unknown argument"),
        (vec!["--deck", "dragons", "--limit", "x", "--role", "ramp"], "needs a number"),
        (vec!["--deck", "dragons", "--deck", "thanos"], "given twice"),
        (vec!["--deck"], "needs a value"),
        (vec!["--ident", "XYZ"], "bad colour identity"),
        (vec!["--deck", "nosuchdeck"], "unknown deck"),
        (vec!["--deck", "dragons", "--ident", "U"], "outside deck dragons"),
    ] {
        let e = run(&args).unwrap_err();
        let m = match e {
            FindError::Usage(m) | FindError::Failed(m) => m,
        };
        assert!(m.contains(want), "{args:?}: {m}");
    }
    let m = map::command(&project, &["--bogus".to_string()]).unwrap_err();
    assert!(m.contains("unknown argument"), "{m}");
}

#[test]
fn the_commands_write_their_files_and_report_them() {
    let env = Env::new();
    let project = env.project();
    let dir = env.dir();
    let dd = ["--data-dir".to_string(), dir.display().to_string()];
    let mut a: Vec<String> = ["--deck", "dragons"].map(String::from).to_vec();
    a.extend(dd.clone());
    let o = digest::command(&project, &a).unwrap();
    assert_eq!(o.path, dir.join("digests/dragons.txt"));
    assert_eq!(std::fs::read_to_string(&o.path).unwrap(), o.digest.text);
    assert!(o.summary().contains("digests/dragons.txt"));
    assert!(!dir.join("digests/dragons.txt.tmp").exists());
    // a page goes to its own file, --out wins
    let mut a: Vec<String> =
        ["--deck", "dragons", "--role", "removal", "--limit", "3", "--offset", "2"].map(String::from).to_vec();
    a.extend(dd.clone());
    let o = digest::command(&project, &a).unwrap();
    assert_eq!(o.path, dir.join("digests/dragons.removal.2.txt"));
    assert_eq!(o.digest.lines, 3);
    let out = env.root.join("x/y/page.txt");
    let mut a: Vec<String> = ["--ident", "WU", "--out", out.to_str().unwrap()].map(String::from).to_vec();
    a.extend(dd.clone());
    digest::command(&project, &a).unwrap();
    assert!(out.is_file());
    // the map
    let (m, _) = map::command(&project, &dd).unwrap();
    assert_eq!(m.path, dir.join("map.md"));
    assert_eq!(std::fs::read_to_string(&m.path).unwrap(), m.text);
    // spellbook comes from the project, not the knowledge dir
    assert!(m.text.contains("| dragons | R |"));
    // the budget flag is honoured
    let mut a: Vec<String> = ["--deck", "thanos", "--budget-tokens", "1500"].map(String::from).to_vec();
    a.extend(dd);
    let o = digest::command(&project, &a).unwrap();
    assert!(digest::tokens(&o.digest.text) <= 1500 || o.digest.over_budget);
    // nothing but the generated files changed in the catalog directory
    let mut files: Vec<String> =
        std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    files.sort();
    assert_eq!(files, ["digests", "map.md", "oracle.json", "roles.json"]);
}

#[test]
fn the_binary_prints_a_summary_or_the_text_and_exits_non_zero_on_an_error() {
    let env = Env::new();
    let exe = env!("CARGO_BIN_EXE_cardgen");
    let dir = env.dir();
    let run = |args: &[&str]| {
        std::process::Command::new(exe)
            .args(args)
            .args(["--root", env.root.to_str().unwrap(), "--data-dir", dir.to_str().unwrap()])
            .env_remove("CARDGEN_KNOWLEDGE_DIR")
            .output()
            .unwrap()
    };
    let o = run(&["digest", "--deck", "dragons"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.starts_with("digest: ") && out.contains("digests/dragons.txt"), "{out}");
    let o = run(&["digest", "--deck", "dragons", "--stdout"]);
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.starts_with("# digest: deck dragons"), "{}", &out[..80]);
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("digest: "));
    let o = run(&["map", "--stdout"]);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).starts_with("# Card catalog map"));
    let o = run(&["digest"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("needs --deck KEY or --ident"));
    let o = run(&["digest", "--deck", "dragons", "--role", "bogus"]);
    assert!(!o.status.success());
    std::fs::remove_file(dir.join("roles.json")).unwrap();
    let o = run(&["map"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("cardgen roles --rules"));
}

#[test]
fn the_deck_cards_map_counts_roles_through_the_shared_search() {
    // a guard on the design: the digest takes its candidates from Index::search, so for a deck the number of cards in
    // B, C, D, E candidates can never include a card `find --deck` would not return
    let env = Env::new();
    let deck = env.deck("thanos");
    let (hits, _) = env.index.search(&knowledge::Query::default(), Some(&deck)).unwrap();
    let allowed: BTreeSet<String> = hits.iter().map(|&i| env.index.cards[i as usize].raw.name.clone()).collect();
    let d = env.digest("thanos");
    for s in ['B', 'C', 'D', 'E'] {
        for n in names(&section(&d.text, s)) {
            assert!(allowed.contains(&n), "{n} in {s} is not a search hit");
        }
    }
}
