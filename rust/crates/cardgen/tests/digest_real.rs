//! `cardgen digest` and `cardgen map` against the real catalog (`data/knowledge/oracle.json` from `cardgen bulk-sync`
//! and `roles.json` from `cardgen roles --rules` or `--merge`): the phase 3b acceptance checks. Run with
//! `cargo test --release -p cardgen --test digest_real -- --ignored`. Whichever role table is present is used.

use cardgen::data::Project;
use cardgen::digest::{self, Prev, Spec, Table};
use cardgen::knowledge::{self, Index, Legal, Query};
use cardgen::map;
use cardgen::roles;
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::time::Instant;

const NEEDS: &str = "needs data/knowledge/oracle.json (run cardgen bulk-sync) and roles.json";
const MINE: [&str; 6] = ["cloud", "sheoldred", "thanos", "thrasios", "urdragon", "yshtola"];

fn project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

fn open() -> (Index, Table) {
    digest::open(&project(), None).unwrap_or_else(|e| panic!("{NEEDS}: {e}"))
}

fn lines(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect()
}

fn build(
    idx: &Index,
    table: &Table,
    deck: &knowledge::Deck,
    role: Option<&str>,
    limit: Option<usize>,
    offset: usize,
) -> digest::Digest {
    let spec = Spec {
        deck: Some(deck),
        ident: None,
        role,
        view: None,
        limit,
        offset,
        budget_tokens: None,
        prev: &Prev::Missing,
    };
    digest::build(idx, table, &spec).unwrap()
}

/// every digest of JD's six decks: line shape, the identity ceiling, section A, the budget, determinism
#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync) and roles.json"]
fn the_six_decks_of_mine_have_clean_digests_within_budget() {
    let (idx, table) = open();
    for key in MINE {
        let deck = knowledge::load_deck(&project(), key).unwrap();
        let limit = knowledge::deck_ident(&deck).unwrap();
        let t = Instant::now();
        let d = build(&idx, &table, &deck, None, None, 0);
        let took = t.elapsed();
        assert!(took.as_secs() < 5, "{key}: {took:?}");
        let ls = lines(&d.text);
        assert_eq!(ls.len(), d.lines);
        assert!(ls.len() > 500, "{key}: {} lines", ls.len());
        assert!(digest::tokens(&d.text) <= digest::DEFAULT_BUDGET_TOKENS, "{key}: {} tokens", digest::tokens(&d.text));
        assert!(d.text.len() < 200_000, "{key}: {} bytes", d.text.len());
        // section A is exactly the deck's own cards (legal ones; a name shared with an un-card is the legal card)
        let mut want: BTreeSet<String> = BTreeSet::new();
        for n in deck.cards.iter().chain(std::iter::once(&deck.commander)).chain(deck.commander2.iter()) {
            let found = idx.lookup(n);
            assert!(!found.is_empty(), "{key}: {n} is not in the catalog");
            let legal: Vec<u32> =
                found.iter().copied().filter(|&i| idx.cards[i as usize].legal == Legal::Legal).collect();
            for i in if legal.is_empty() { found.to_vec() } else { legal } {
                want.insert(idx.cards[i as usize].raw.name.clone());
            }
        }
        let a: BTreeSet<String> = d
            .text
            .lines()
            .skip_while(|l| !l.starts_with("## A "))
            .skip(1)
            .take_while(|l| !l.starts_with('#'))
            .map(|l| l.split('|').next().unwrap().to_string())
            .collect();
        assert_eq!(a, want, "{key}: section A");
        let mut seen_b: HashSet<&str> = HashSet::new();
        let mut section = ' ';
        for l in d.text.lines() {
            if let Some(h) = l.strip_prefix("## ") {
                section = h.chars().next().unwrap();
                continue;
            }
            if l.starts_with('#') || l.is_empty() {
                continue;
            }
            let fields: Vec<&str> = l.split('|').collect();
            assert_eq!(fields.len(), 6, "{key}: {l}");
            assert!(l.chars().count() <= 160, "{key}: {} chars: {l}", l.chars().count());
            let name = fields[0];
            let found = idx.lookup(name);
            let c = &idx.cards[found
                .iter()
                .copied()
                .find(|&i| idx.cards[i as usize].legal == Legal::Legal)
                .unwrap_or(found[0]) as usize];
            // the deck's own cards are listed whatever their legality (JD's list may name a banned card)
            assert!(section == 'A' || c.legal == Legal::Legal, "{key}: {name}");
            assert!(roles::is_deck_card(c) && !c.is_meld(), "{key}: {name}");
            assert_eq!(c.ident & !limit, 0, "{key}: {name} is outside the deck's identity");
            assert_eq!(fields[5].starts_with('*'), section == 'A', "{key}: {l}");
            if section == 'B' {
                assert!(seen_b.insert(name), "{key}: {name} twice in B");
            }
        }
        // two runs are byte-identical
        assert_eq!(d.text, build(&idx, &table, &deck, None, None, 0).text, "{key}");
        println!("{key}: {} lines, {} bytes, ~{} tokens, {took:?}", ls.len(), d.text.len(), digest::tokens(&d.text));
    }
}

/// a role's pages chain without gaps or overlaps and each stays within the page budget
#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync) and roles.json"]
fn role_pages_of_a_real_deck_chain_without_gaps_or_overlaps() {
    let (idx, table) = open();
    let deck = knowledge::load_deck(&project(), "cloud").unwrap();
    // one page with a huge budget holds the whole role (every card that has it, primary first)
    let spec = Spec {
        deck: Some(&deck),
        ident: None,
        role: Some("removal"),
        view: None,
        limit: None,
        offset: 0,
        budget_tokens: Some(10_000_000),
        prev: &Prev::Missing,
    };
    let all = digest::build(&idx, &table, &spec).unwrap();
    let total = all.total.unwrap();
    assert!(total > 500, "{total}");
    let want: Vec<&str> = lines(&all.text).iter().map(|l| l.split('|').next().unwrap()).collect();
    assert_eq!(want.len(), total);
    let (mut got, mut offset, mut pages) = (Vec::<String>::new(), 0, 0);
    loop {
        let p = build(&idx, &table, &deck, Some("removal"), None, offset);
        assert!(
            digest::tokens(&p.text) <= digest::ROLE_PAGE_TOKENS + 100,
            "page {pages}: {} tokens",
            digest::tokens(&p.text)
        );
        got.extend(lines(&p.text).iter().map(|l| l.split('|').next().unwrap().to_string()));
        pages += 1;
        match p.next_offset {
            Some(n) => {
                assert_eq!(n, offset + p.lines);
                offset = n;
            }
            None => break,
        }
    }
    assert_eq!(got, want);
    println!("cloud removal: {total} cards in {pages} page(s)");
    // every role of every deck has a first page within the budget
    for role in roles::IDS {
        let p = build(&idx, &table, &deck, Some(role), None, 0);
        assert!(digest::tokens(&p.text) <= digest::ROLE_PAGE_TOKENS + 100, "{role}");
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync) and roles.json"]
fn an_identity_digest_without_a_deck_stays_inside_its_colours() {
    let (idx, table) = open();
    let spec = Spec {
        deck: None,
        ident: Some(knowledge::parse_ident("WUB").unwrap()),
        role: None,
        view: None,
        limit: None,
        offset: 0,
        budget_tokens: None,
        prev: &Prev::Missing,
    };
    let d = digest::build(&idx, &table, &spec).unwrap();
    assert!(d.lines > 500);
    for l in lines(&d.text) {
        let c = &idx.cards[idx.lookup(l.split('|').next().unwrap())[0] as usize];
        assert_eq!(c.ident & !0b00111, 0, "{l}");
    }
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync) and roles.json"]
fn the_map_is_under_32_kb_complete_and_deterministic() {
    let (idx, table) = open();
    let spell = project().data("spellbook");
    let t = Instant::now();
    let text = map::build(&idx, &table, Some(&spell)).unwrap();
    println!("map: {} bytes, ~{} tokens, {:?}", text.len(), digest::tokens(&text), t.elapsed());
    assert!(text.len() < map::MAX_BYTES);
    assert!(digest::tokens(&text) < 9_000, "{}", digest::tokens(&text));
    assert_eq!(text, map::build(&idx, &table, Some(&spell)).unwrap());
    assert!(text.contains("32 that occur"), "all 32 colour identities occur");
    let ids = text
        .lines()
        .skip_while(|l| !l.starts_with("## Colour identities"))
        .skip(4)
        .take_while(|l| l.starts_with('|'))
        .count();
    assert_eq!(ids, 32);
    for r in roles::IDS {
        assert!(text.contains(&format!("| {r} | ")), "{r}");
    }
    // the covered count is what a default search returns
    let (hits, _) = idx.search(&Query::default(), None).unwrap();
    let covered = hits.iter().filter(|&&i| roles::is_deck_card(&idx.cards[i as usize])).count();
    assert!(text.contains(&format!("cover {} legal cards", num(covered))), "{}", &text[..600]);
    assert!(text.contains("## Game Changers (") && text.contains("## Banned in Commander ("));
    assert!(text.contains("- thanos (fetched") && text.contains("| thanos | Thanos, the Mad Titan | WUBRG | t5 |"));
}

fn num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}
