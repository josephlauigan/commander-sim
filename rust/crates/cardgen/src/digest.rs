//! `cardgen digest`: the compact per-deck view of the catalog (layer 2 of documents/card-knowledge.md).
//!
//! One line per card, `Name|{cost}|Type—Subtypes|roles|gist|flags` (six `|`-separated fields, at most 160 characters).
//! The cards come from [`Index::search`], so the deck's identity ceiling and exclusions are the ones `find` applies;
//! the digest never re-implements them. The gist is a word-boundary cut of the Oracle line cited as evidence for the
//! card's primary role (or its first Oracle line when no line is cited), never written text. Nothing here needs the
//! LLM: the role table is read as it is, rules-only or merged, and each line says how its roles were decided (`r:`).
//!
//! Sections: A the deck's own cards, B per primary role the best-ranked cards, C cards that share a subtype with the
//! commander's text, D cards new since the previous sync, E unranked cards per role. `--role R` prints one role's cards
//! and pages through them. Output is deterministic: no timestamps, a fixed order.

use crate::data::Project;
use crate::knowledge::{
    self, Card, Deck, FindError, Index, Legal, Query, Sort, T_ARTIFACT, T_BATTLE, T_CREATURE, T_ENCHANTMENT, T_INSTANT,
    T_LAND, T_LEGENDARY, T_PLANESWALKER, T_SORCERY,
};
use crate::roles::{self, Assignment};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// the default size of a digest, in estimated tokens
pub const DEFAULT_BUDGET_TOKENS: usize = 50_000;
/// the size of one `--role` page, in estimated tokens
pub const ROLE_PAGE_TOKENS: usize = 30_000;
/// the longest line, in characters
pub const MAX_LINE_CHARS: usize = 160;
/// the longest gist, in characters (the line limit may leave less room)
pub const GIST_CHARS: usize = 110;
/// section B: cards per role, and per each of the deck's `TOP_ROLES` most common roles
pub const CAP_ROLE: usize = 60;
pub const CAP_TOP_ROLE: usize = 150;
pub const TOP_ROLES: usize = 5;
/// section C: the most cards listed
pub const CAP_SYNERGY: usize = 300;
/// section E: cards per role
pub const CAP_UNRANKED: usize = 40;
/// a gist shorter than this is not worth a line: the subtypes give way first
const MIN_GIST_CHARS: usize = 16;

/// The size estimate every budget uses: bytes / 4, rounded up. It is never below characters / 4, so a digest within
/// the budget is within it by either count.
pub fn tokens(s: &str) -> usize {
    s.len().div_ceil(4)
}

// ------------------------------------------------------------------------------------------------ one line

/// How a card's roles were decided (the `r:` flag).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Review {
    /// the rules (or a hand override) decided alone: the LLM was never needed
    Rules,
    /// the card was routed to the LLM and its answer is merged
    Llm,
    /// the card was routed to the LLM and has no answer yet: its roles may be incomplete or wrong
    Uncertain,
    /// no roles, and nothing to ask the LLM
    None,
}

impl Review {
    pub fn flag(self) -> &'static str {
        match self {
            Review::Rules => "r:rules",
            Review::Llm => "r:llm",
            Review::Uncertain => "r:uncertain",
            Review::None => "r:none",
        }
    }
}

/// What kind of role table is attached (`roles.json`'s metadata): rules alone, or merged with LLM answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// built under an LLM prompt and model (a merged table) rather than by the rules alone
    pub merged: bool,
    pub model: String,
}

impl Table {
    pub fn of(meta: &roles::RoleMeta) -> Table {
        Table { merged: !meta.model.is_empty() || !meta.prompt_fnv.is_empty(), model: meta.model.clone() }
    }

    /// the words the header uses: `rules-only` or `merged (model M)`
    pub fn kind(&self) -> String {
        match (self.merged, self.model.is_empty()) {
            (false, _) => "rules-only".into(),
            (true, true) => "merged".into(),
            (true, false) => format!("merged (model {})", self.model),
        }
    }
}

/// Opens the catalog `find` would read with its role table attached (`roles.json` beside `oracle.json`): what the map
/// and the digest need. A missing or stale table is an error that names the command to run.
pub fn open(project: &Project, data_dir: Option<&Path>) -> Result<(Index, Table), String> {
    let dir = knowledge::knowledge_dir(project, data_dir)?;
    let mut index = Index::load(&dir.join("oracle.json"))?;
    let path = dir.join("roles.json");
    if !path.is_file() {
        return Err(format!("no role table at {}: run `cardgen roles --rules` first", path.display()));
    }
    let map = roles::load(&path)?;
    index.attach_roles(&map)?;
    Ok((index, Table::of(&map.meta)))
}

/// The review status of one assignment. A routed card of a merged table has an answer unless the merge marked it
/// `llm_failed`; a routed card of a rules-only table has none.
pub fn review(a: &Assignment, merged: bool) -> Review {
    if !a.needs_llm {
        if a.roles.is_empty() { Review::None } else { Review::Rules }
    } else if merged && !a.llm_failed {
        Review::Llm
    } else {
        Review::Uncertain
    }
}

const ABBREV: [(u16, char); 8] = [
    (T_ARTIFACT, 'A'),
    (T_ENCHANTMENT, 'E'),
    (T_LAND, 'L'),
    (T_CREATURE, 'C'),
    (T_PLANESWALKER, 'P'),
    (T_BATTLE, 'B'),
    (T_INSTANT, 'I'),
    (T_SORCERY, 'S'),
];

/// The type abbreviation: `Lg` for a legendary card, then a letter per card type of any face in the order
/// A E L C P B I S (`AC` is an artifact creature, `LgC` a legendary creature), `O` when it has none of them.
pub fn type_abbrev(c: &Card) -> String {
    let mut s = String::new();
    if c.types & T_LEGENDARY != 0 {
        s.push_str("Lg");
    }
    let letters: String = ABBREV.iter().filter(|(bit, _)| c.types & bit != 0).map(|(_, l)| *l).collect();
    s.push_str(if letters.is_empty() { "O" } else { &letters });
    s
}

/// a field of a line: `|` becomes `/`, white space other than a space becomes a space
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '|' => '/',
            '\n' | '\r' | '\t' => ' ',
            c => c,
        })
        .collect()
}

/// The Oracle line a gist is cut from, before cleaning and cutting: the line cited as evidence for the primary role
/// (the normalised text the rules and guards read); for a card with no cited line (no roles, or a hand override) its
/// first Oracle line with text after reminder text is removed. Empty when the card has no such line.
pub fn gist_source(c: &Card, a: &Assignment) -> String {
    if let Some(line) = a.evidence.get(&a.primary).and_then(|e| e.line) {
        let t = roles::line_text(c, line);
        if !t.is_empty() {
            return t;
        }
    }
    if let Some((_, t)) = roles::prompt_lines(c).into_iter().next() {
        return t;
    }
    c.lines().iter().map(|(_, l)| roles::normalize(c, l)).find(|t| !t.is_empty()).unwrap_or_default()
}

/// `s` cut at a word boundary to at most `max` characters; a cut text ends in `…` (counted). The result is always a
/// prefix of `s` (plus the `…`).
pub fn cut(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max < 2 {
        return String::new();
    }
    let end = s.char_indices().nth(max - 1).map_or(s.len(), |(i, _)| i);
    let at = if s[end..].starts_with(' ') { end } else { s[..end].rfind(' ').filter(|&i| i > 0).unwrap_or(end) };
    format!("{}…", s[..at].trim_end())
}

/// Builds digest lines for one catalog.
pub struct Lines<'a> {
    pub index: &'a Index,
    merged: bool,
}

impl<'a> Lines<'a> {
    pub fn new(index: &'a Index, table: &Table) -> Lines<'a> {
        Lines { index, merged: table.merged }
    }

    pub fn review(&self, i: u32) -> Review {
        self.index.roles(i).map_or(Review::None, |a| review(a, self.merged))
    }

    /// The digest line of card `i`; `in_deck` adds the `*` flag.
    pub fn line(&self, i: u32, in_deck: bool) -> String {
        let c = &self.index.cards[i as usize];
        let default = Assignment::default();
        let a = self.index.roles(i).unwrap_or(&default);
        let name = clean(&c.raw.name);
        let cost = Some(clean(&c.cost())).filter(|s| !s.is_empty()).unwrap_or_else(|| "-".into());
        let roles = if a.roles.is_empty() { "-".to_string() } else { a.roles.join(",") };
        let mut flags: Vec<String> = vec![];
        if in_deck {
            flags.push("*".into());
        }
        if c.raw.game_changer {
            flags.push("GC".into());
        }
        if let Some(rank) = c.raw.edhrec_rank {
            flags.push(format!("#{rank}"));
        }
        if let Some(usd) = c.usd {
            flags.push(format!("${}", usd.round() as i64));
        }
        flags.push(self.review(i).flag().into());
        let flags = flags.join(" ");
        let src = clean(&gist_source(c, a));
        let src_len = src.chars().count();
        let subs = c.subtypes();
        let abbrev = type_abbrev(c);
        let len = |s: &str| s.chars().count();
        // the subtypes give way first when the other fields leave too little room for the gist
        let mut keep = subs.len();
        let (types, room) = loop {
            let types =
                if keep == 0 { abbrev.clone() } else { clean(&format!("{abbrev}—{}", subs[..keep].join(" "))) };
            let used = len(&name) + len(&cost) + len(&types) + len(&roles) + len(&flags) + 5;
            let room = MAX_LINE_CHARS.saturating_sub(used);
            if keep == 0 || room >= src_len.min(GIST_CHARS).min(MIN_GIST_CHARS) {
                break (types, room);
            }
            keep -= 1;
        };
        let gist = if src.is_empty() || room == 0 { "-".to_string() } else { cut(&src, room.min(GIST_CHARS)) };
        format!("{name}|{cost}|{types}|{roles}|{gist}|{flags}")
    }
}

// ------------------------------------------------------------------------------------------------ the digest

/// What the previous sync held (`oracle.prev.json`), for section D.
pub enum Prev {
    /// no previous sync: section D is left out with a note
    Missing,
    /// the `oracle_id`s of the previous sync
    Ids(HashSet<String>),
    /// the file could not be read: section D is left out with this reason
    Unreadable(String),
}

impl Prev {
    /// Reads `oracle.prev.json` of a knowledge directory.
    pub fn load(dir: &Path) -> Prev {
        #[derive(serde::Deserialize)]
        struct Id {
            #[serde(default)]
            oracle_id: String,
        }
        #[derive(serde::Deserialize)]
        struct File {
            cards: Vec<Id>,
        }
        let path = dir.join("oracle.prev.json");
        if !path.is_file() {
            return Prev::Missing;
        }
        let read = std::fs::File::open(&path)
            .map_err(|e| e.to_string())
            .and_then(|f| serde_json::from_reader::<_, File>(std::io::BufReader::new(f)).map_err(|e| e.to_string()));
        match read {
            Ok(f) => Prev::Ids(f.cards.into_iter().map(|c| c.oracle_id).collect()),
            Err(e) => Prev::Unreadable(format!("{}: {e}", path.display())),
        }
    }
}

/// What to digest.
pub struct Spec<'a> {
    /// a deck of `data/decks.json`: its identity is the ceiling, its cards are section A, its commanders drive C
    pub deck: Option<&'a Deck>,
    /// an identity limit: the whole identity without a deck, a narrowing of the deck's with one
    pub ident: Option<u8>,
    /// only this role's cards (section B for it, all of them, paged by `limit` and `offset`)
    pub role: Option<&'a str>,
    pub limit: Option<usize>,
    pub offset: usize,
    /// the size limit in estimated tokens; `None` is [`DEFAULT_BUDGET_TOKENS`] (or [`ROLE_PAGE_TOKENS`] for a page)
    pub budget_tokens: Option<usize>,
    pub prev: &'a Prev,
}

/// One block of lines under one `## ` header.
#[derive(Debug, Clone)]
pub struct Section {
    /// `A`..`E`
    pub id: char,
    /// the role of a per-role block of B or E
    pub role: Option<String>,
    pub lines: usize,
}

#[derive(Debug)]
pub struct Digest {
    pub text: String,
    /// card lines (not comment or header lines)
    pub lines: usize,
    pub sections: Vec<Section>,
    /// `--role` pages: the cards with that primary role, and the offset of the next page when there is one
    pub total: Option<usize>,
    pub next_offset: Option<usize>,
    /// section B's caps (role, top role) after trimming, when they were cut to meet the budget
    pub b_caps: Option<(usize, usize)>,
    pub e_cap: Option<usize>,
    pub over_budget: bool,
}

struct Block {
    id: char,
    role: Option<String>,
    header: String,
    lines: Vec<String>,
}

fn usage<T>(m: impl Into<String>) -> Result<T, FindError> {
    Err(FindError::Usage(m.into()))
}

fn ident_label(bits: u8) -> String {
    if bits == 0 { "C".into() } else { knowledge::ident_string(bits) }
}

/// the cards of a deck as indices: its list and commanders, each name once (the Commander-legal card of a shared
/// name when there is one), sorted as `find` sorts
fn deck_cards(index: &Index, deck: &Deck) -> Vec<u32> {
    let mut out: Vec<u32> = vec![];
    let names = deck.cards.iter().chain(std::iter::once(&deck.commander)).chain(deck.commander2.iter());
    for n in names.filter(|n| !n.trim().is_empty()) {
        let found = index.lookup(n);
        let legal: Vec<u32> =
            found.iter().copied().filter(|&i| index.cards[i as usize].legal == Legal::Legal).collect();
        for i in if legal.is_empty() { found.to_vec() } else { legal } {
            if roles::is_deck_card(&index.cards[i as usize]) && !out.contains(&i) {
                out.push(i);
            }
        }
    }
    index.sort(&mut out, Sort::Edhrec);
    out
}

/// the plural spellings of a subtype as Oracle text writes them (`Elf`: Elves)
fn plural_forms(s: &str) -> Vec<String> {
    let mut v = vec![s.to_string(), format!("{s}s"), format!("{s}es")];
    if let Some(stem) = s.strip_suffix("fe").or_else(|| s.strip_suffix('f')) {
        v.push(format!("{stem}ves"));
    }
    if let Some(stem) = s.strip_suffix('y') {
        if !stem.ends_with(['a', 'e', 'i', 'o', 'u']) {
            v.push(format!("{stem}ies"));
        }
    }
    match s {
        "Mouse" => v.push("Mice".into()),
        "Fungus" => v.push("Fungi".into()),
        "Octopus" => v.push("Octopi".into()),
        _ => {}
    }
    v
}

/// whether `text` has `word` as a whole word (a letter or digit on either side is not a boundary)
fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + word.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// the Oracle text with reminder text removed and the card's own name as `~` (the text the rules read), joined
fn plain_text(c: &Card) -> String {
    c.lines().iter().map(|(_, l)| roles::normalize(c, l)).collect::<Vec<_>>().join("\n")
}

/// Every subtype of the catalog that the commanders' Oracle text names (`Dragon`, `Equipment`; a plural names it too).
pub fn commander_subtypes(index: &Index, deck: &Deck) -> Vec<String> {
    let vocab = index.subtype_vocab();
    let mut found: Vec<String> = vec![];
    let names = std::iter::once(&deck.commander).chain(deck.commander2.iter());
    for n in names {
        let cards = index.lookup(n);
        let legal: Vec<u32> =
            cards.iter().copied().filter(|&i| index.cards[i as usize].legal == Legal::Legal).collect();
        for i in if legal.is_empty() { cards.to_vec() } else { legal } {
            let text = plain_text(&index.cards[i as usize]);
            // a subtype is a capitalised word (the lower-case `of` of a joke card's type line is not one)
            for (sub, _) in vocab.iter().filter(|(s, _)| s.chars().next().is_some_and(char::is_uppercase)) {
                if !found.contains(sub) && plural_forms(sub).iter().any(|f| has_word(&text, f)) {
                    found.push(sub.clone());
                }
            }
        }
    }
    found.sort();
    found
}

/// whether the card's type line has one of the subtypes or its text names one
fn names_subtype(c: &Card, subs: &[String], forms: &[Vec<String>]) -> bool {
    if c.subtypes().iter().any(|s| subs.iter().any(|t| t == s)) {
        return true;
    }
    // a cheap substring test first, then the whole-word test on the text the rules read
    if !forms.iter().flatten().any(|f| c.text.contains(f.as_str())) {
        return false;
    }
    let text = plain_text(c);
    forms.iter().flatten().any(|f| has_word(&text, f))
}

/// the deck's header words: `deck cloud (Cloud, Midgar Mercenary), identity W`
fn subject(deck: Option<&Deck>, ident: Option<u8>) -> String {
    match (deck, ident) {
        (Some(d), _) => {
            let cmd = match &d.commander2 {
                Some(c2) => format!("{} + {c2}", d.commander),
                None => d.commander.clone(),
            };
            let limit = knowledge::deck_ident(d).map(ident_label).unwrap_or_else(|_| d.ident.clone());
            let narrowed = ident.filter(|&i| knowledge::deck_ident(d).is_ok_and(|l| l != i));
            match narrowed {
                Some(n) => format!("deck {} ({cmd}), identity {limit} narrowed to {}", d.key, ident_label(n)),
                None => format!("deck {} ({cmd}), identity {limit}", d.key),
            }
        }
        (None, Some(i)) => format!("identity {} (no deck)", ident_label(i)),
        (None, None) => String::new(),
    }
}

/// the legend lines every digest starts with
fn legend() -> Vec<String> {
    vec![
        "# line: Name|{cost}|Type—Subtypes|roles, primary first|gist|flags (- = none)".into(),
        concat!(
            "# types: Lg legendary; A artifact, E enchantment, L land, C creature, P planeswalker, B battle, ",
            "I instant, S sorcery, O other"
        )
        .into(),
        concat!(
            "# gist: the Oracle line cited for the primary role, reminder text removed, ~ is the card, ",
            "cut at a word boundary (… = cut)"
        )
        .into(),
        concat!(
            "# flags: * in the deck's list; GC Game Changer; #N EDHREC rank; $N price in USD; r: how the roles were ",
            "decided: rules (the rules alone), llm (an LLM answer is merged), uncertain (not reviewed yet: roles may ",
            "be incomplete or wrong), none (no roles)"
        )
        .into(),
    ]
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

/// Builds the digest. `Index::search` supplies the cards (legal, in the identity, not in the deck, no meld results);
/// Stickers and Attractions, which have no roles, are left out. See the module documentation for the sections.
pub fn build(index: &Index, table: &Table, spec: &Spec) -> Result<Digest, FindError> {
    if spec.deck.is_none() && spec.ident.is_none() {
        return usage("digest needs --deck KEY or --ident LETTERS");
    }
    if let Some(role) = spec.role {
        roles::check_role(role).map_err(FindError::Usage)?;
    } else if spec.limit.is_some() || spec.offset > 0 {
        return usage("--limit and --offset page through one role's cards: add --role R");
    }
    if spec.limit == Some(0) {
        return usage("--limit 0 shows nothing");
    }
    if spec.budget_tokens == Some(0) {
        return usage("--budget-tokens needs a positive number");
    }
    if index.roles(0).is_none() {
        return Err(FindError::Failed("the catalog has no role table attached; run `cardgen roles --rules`".into()));
    }
    let q = Query { ident: spec.ident, ..Query::default() };
    let (hits, unresolved) = index.search(&q, spec.deck)?;
    let hits: Vec<u32> = hits.into_iter().filter(|&i| roles::is_deck_card(&index.cards[i as usize])).collect();
    let lines = Lines::new(index, table);
    let mut head: Vec<String> = vec![format!("# digest: {}", subject(spec.deck, spec.ident))];
    head.push(format!(
        "# role table: {} (roles.json); catalog: Scryfall bulk {}; {} legal cards in the identity, not in the deck",
        table.kind(),
        if index.bulk_updated_at.is_empty() { "unknown" } else { &index.bulk_updated_at },
        num(hits.len())
    ));
    head.extend(legend());
    if !unresolved.is_empty() {
        let shown: Vec<&str> = unresolved.iter().take(10).map(String::as_str).collect();
        let more = if unresolved.len() > 10 { ", ..." } else { "" };
        head.push(format!("# deck names not in the catalog ({}): {}{more}", unresolved.len(), shown.join(", ")));
    }
    match spec.role {
        Some(role) => role_page(&lines, spec, &hits, role, head),
        None => full(&lines, spec, &hits, head),
    }
}

/// `--role R`: the cards whose primary role is R, best first, a page of them
fn role_page(l: &Lines, spec: &Spec, hits: &[u32], role: &str, mut head: Vec<String>) -> Result<Digest, FindError> {
    let index = l.index;
    let all: Vec<u32> = hits.iter().copied().filter(|&i| index.roles(i).is_some_and(|a| a.primary == role)).collect();
    let cap = spec.budget_tokens.unwrap_or(ROLE_PAGE_TOKENS);
    let limit = spec.limit.unwrap_or(usize::MAX);
    head.push(format!(
        "# page: role {role}{}; {} cards have it as their primary role",
        if spec.deck.is_some() { ", the deck's own cards left out" } else { "" },
        num(all.len())
    ));
    let mut used: usize = head.iter().map(|h| tokens(h) + 1).sum::<usize>() + 80;
    let mut shown: Vec<String> = vec![];
    for &i in all.iter().skip(spec.offset) {
        let line = l.line(i, false);
        used += tokens(&line) + 1;
        if shown.len() >= limit || (used > cap && !shown.is_empty()) {
            break;
        }
        shown.push(line);
    }
    let end = spec.offset + shown.len();
    let next = (end < all.len()).then_some(end);
    let range = if shown.is_empty() {
        format!("no cards at offset {} (the role has {})", spec.offset, num(all.len()))
    } else {
        format!("cards {}-{} of {}", spec.offset + 1, end, num(all.len()))
    };
    let tail = match next {
        Some(n) => format!("; next page: --offset {n}"),
        None => "; last page".into(),
    };
    let mut text = head.join("\n");
    text.push_str(&format!("\n## B role {role}: {range}, best EDHREC rank first, unranked last{tail}\n"));
    for s in &shown {
        text.push_str(s);
        text.push('\n');
    }
    Ok(Digest {
        text,
        lines: shown.len(),
        sections: vec![Section { id: 'B', role: Some(role.to_string()), lines: shown.len() }],
        total: Some(all.len()),
        next_offset: next,
        b_caps: None,
        e_cap: None,
        over_budget: false,
    })
}

/// the default digest: sections A to E within the budget
fn full(l: &Lines, spec: &Spec, hits: &[u32], mut head: Vec<String>) -> Result<Digest, FindError> {
    let index = l.index;
    let budget = spec.budget_tokens.unwrap_or(DEFAULT_BUDGET_TOKENS);
    // A: the deck's own cards
    let own = spec.deck.map(|d| deck_cards(index, d)).unwrap_or_default();
    // a card of the list the catalog does not call Commander-legal is still the deck's own: say so
    let illegal: Vec<&str> = own
        .iter()
        .map(|&i| &index.cards[i as usize])
        .filter(|c| c.legal != Legal::Legal)
        .map(|c| c.raw.name.as_str())
        .collect();
    if !illegal.is_empty() {
        head.push(format!(
            "# deck cards the catalog does not list as Commander-legal ({}): {}",
            illegal.len(),
            illegal.join(", ")
        ));
    }
    // the deck's most common roles (a card counts once for each of its roles; ties in the order of the taxonomy)
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for &i in &own {
        for r in index.roles(i).iter().flat_map(|a| a.roles.iter()) {
            *count.entry(r.as_str()).or_default() += 1;
        }
    }
    let mut common: Vec<&str> = roles::IDS.iter().copied().filter(|r| count.contains_key(r)).collect();
    common.sort_by_key(|r| std::cmp::Reverse(count[r]));
    common.truncate(TOP_ROLES);
    // the candidates by primary role: ranked ones for B, unranked ones for E
    let mut ranked: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    let mut unranked: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    let mut roleless = 0usize;
    for &i in hits {
        let a = index.roles(i).filter(|a| !a.primary.is_empty());
        let Some(a) = a else {
            roleless += 1;
            continue;
        };
        let role = roles::IDS.iter().copied().find(|r| *r == a.primary).expect("a primary role is in the taxonomy");
        let map = if index.cards[i as usize].raw.edhrec_rank.is_some() { &mut ranked } else { &mut unranked };
        map.entry(role).or_default().push(i);
    }
    // C: synergy with the commanders' subtypes
    let subs = spec.deck.map(|d| commander_subtypes(index, d)).unwrap_or_default();
    let forms: Vec<Vec<String>> = subs.iter().map(|s| plural_forms(s)).collect();
    let synergy: Vec<u32> = if subs.is_empty() {
        vec![]
    } else {
        hits.iter()
            .copied()
            .filter(|&i| names_subtype(&index.cards[i as usize], &subs, &forms))
            .take(CAP_SYNERGY)
            .collect()
    };
    let in_c: HashSet<u32> = synergy.iter().copied().collect();
    // D: new since the previous sync
    let (new_cards, d_note): (Option<Vec<u32>>, String) = match spec.prev {
        Prev::Ids(prev) => {
            let v: Vec<u32> = hits
                .iter()
                .copied()
                .filter(|i| !in_c.contains(i) && !prev.contains(&index.cards[*i as usize].raw.oracle_id))
                .collect();
            (Some(v), String::new())
        }
        Prev::Missing => (None, "no oracle.prev.json, so there is no previous sync to compare with".into()),
        Prev::Unreadable(why) => (None, format!("oracle.prev.json is unreadable ({why})")),
    };
    let in_d: HashSet<u32> = new_cards.iter().flatten().copied().collect();

    let a_block = Block {
        id: 'A',
        role: None,
        header: match spec.deck {
            Some(d) => format!("## A deck {}: its own {} cards", d.key, own.len()),
            None => "## A deck cards: none (no deck given)".into(),
        },
        lines: own.iter().map(|&i| l.line(i, true)).collect(),
    };
    let render = |v: &[u32], n: usize| v.iter().take(n).map(|&i| l.line(i, false)).collect::<Vec<String>>();
    let b_lines: Vec<(&str, usize, usize, Vec<String>)> = roles::IDS
        .iter()
        .filter_map(|&r| {
            let v = ranked.get(r)?;
            let cap = if common.contains(&r) { CAP_TOP_ROLE } else { CAP_ROLE };
            Some((r, cap, v.len(), render(v, cap)))
        })
        .collect();
    let c_block = Block {
        id: 'C',
        role: None,
        header: if subs.is_empty() {
            "## C synergy: none (the commander's text names no subtype)".into()
        } else {
            format!(
                "## C synergy: {} cards sharing a subtype the commander's text names ({})",
                synergy.len(),
                subs.join(", ")
            )
        },
        lines: render(&synergy, usize::MAX),
    };
    let d_block = new_cards.as_ref().map(|v| Block {
        id: 'D',
        role: None,
        header: format!("## D new since the previous sync: {} cards (not repeated from C)", v.len()),
        lines: render(v, usize::MAX),
    });
    let e_lines: Vec<(&str, usize, Vec<String>)> = roles::IDS
        .iter()
        .filter_map(|&r| {
            let v: Vec<u32> =
                unranked.get(r)?.iter().copied().filter(|i| !in_c.contains(i) && !in_d.contains(i)).collect();
            (!v.is_empty()).then(|| (r, v.len(), render(&v, CAP_UNRANKED)))
        })
        .collect();

    // assemble at a scale (per mille) for the caps of B and E
    let assemble = |sb: usize, se: usize, over: bool| -> (String, Vec<Section>) {
        let mut blocks: Vec<Block> =
            vec![Block { id: 'A', role: None, header: a_block.header.clone(), lines: a_block.lines.clone() }];
        for (r, cap, total, ls) in &b_lines {
            let n = (cap * sb / 1000).min(ls.len());
            if n > 0 {
                blocks.push(Block {
                    id: 'B',
                    role: Some(r.to_string()),
                    header: format!(
                        "## B role {r}: best {n} of {} ranked cards with this primary role (--role {r} lists them all)",
                        num(*total)
                    ),
                    lines: ls[..n].to_vec(),
                });
            }
        }
        blocks.push(Block { id: 'C', role: None, header: c_block.header.clone(), lines: c_block.lines.clone() });
        if let Some(d) = &d_block {
            blocks.push(Block { id: 'D', role: None, header: d.header.clone(), lines: d.lines.clone() });
        }
        for (r, total, ls) in &e_lines {
            let n = (CAP_UNRANKED * se / 1000).min(ls.len());
            if n > 0 {
                blocks.push(Block {
                    id: 'E',
                    role: Some(r.to_string()),
                    header: format!(
                        "## E role {r}: {n} of {} cards with no EDHREC rank and this primary role",
                        num(*total)
                    ),
                    lines: ls[..n].to_vec(),
                });
            }
        }
        let secs: Vec<Section> =
            blocks.iter().map(|b| Section { id: b.id, role: b.role.clone(), lines: b.lines.len() }).collect();
        let size = |id: char| secs.iter().filter(|s| s.id == id).map(|s| s.lines).sum::<usize>();
        let b_roles = secs.iter().filter(|s| s.id == 'B').count();
        let mut h = head.clone();
        h.push(format!(
            "# sections: A {} | B {} ({} roles) | C {} | D {} | E {}; {} cards with no role are listed only in A, C, D",
            size('A'),
            size('B'),
            b_roles,
            size('C'),
            if d_block.is_some() { size('D').to_string() } else { "omitted".into() },
            size('E'),
            num(roleless)
        ));
        if d_block.is_none() {
            h.push(format!("# D omitted: {d_note}"));
        }
        let top = if common.is_empty() { "none".to_string() } else { common.join(", ") };
        h.push(format!(
            "# B caps: {CAP_ROLE} per role, {CAP_TOP_ROLE} for the deck's most common roles ({top}); \
             C at most {CAP_SYNERGY}; E {CAP_UNRANKED} per role (before trimming)"
        ));
        if sb < 1000 || se < 1000 || over {
            let mut note = format!("# budget: {} tokens (bytes/4)", num(budget));
            if sb < 1000 {
                note.push_str(&format!(
                    "; B caps trimmed to {} per role and {} for the most common roles",
                    CAP_ROLE * sb / 1000,
                    CAP_TOP_ROLE * sb / 1000
                ));
            }
            if se < 1000 {
                note.push_str(&format!("; E caps trimmed to {} per role", CAP_UNRANKED * se / 1000));
            }
            if over {
                note.push_str("; still over the budget: A, C and D are not capped");
            }
            h.push(note);
        }
        let mut text = h.join("\n");
        text.push('\n');
        for b in &blocks {
            text.push_str(&b.header);
            text.push('\n');
            for line in &b.lines {
                text.push_str(line);
                text.push('\n');
            }
        }
        (text, secs)
    };
    let fits = |sb: usize, se: usize| tokens(&assemble(sb, se, false).0) <= budget;
    // the largest scale (0..=1000) for which `ok` holds, 0 when none does
    let largest = |ok: &dyn Fn(usize) -> bool| -> usize {
        let (mut lo, mut hi) = (0usize, 1000usize);
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if ok(mid) { lo = mid } else { hi = mid - 1 }
        }
        lo
    };
    // trim B first, then E when B is already gone
    let (mut sb, mut se) = (1000, 1000);
    let mut over = false;
    if !fits(sb, se) {
        sb = largest(&|s| fits(s, 1000));
        if !fits(sb, se) {
            se = largest(&|s| fits(0, s));
            over = !fits(0, se);
        }
    }
    let (text, sections) = assemble(sb, se, over);
    Ok(Digest {
        text,
        lines: sections.iter().map(|s| s.lines).sum(),
        sections,
        total: None,
        next_offset: None,
        b_caps: (sb < 1000).then_some((CAP_ROLE * sb / 1000, CAP_TOP_ROLE * sb / 1000)),
        e_cap: (se < 1000).then_some(CAP_UNRANKED * se / 1000),
        over_budget: over,
    })
}

// ------------------------------------------------------------------------------------------------ the command

pub const USAGE: &str =
    "usage: cardgen digest (--deck KEY | --ident LETTERS) [--role R [--limit N] [--offset N]] [--out PATH]
                      [--budget-tokens N] [--stdout] [--data-dir DIR]";

/// What the command did.
#[derive(Debug)]
pub struct Output {
    pub digest: Digest,
    pub path: PathBuf,
    pub bytes: usize,
}

impl Output {
    /// the one-line report
    pub fn summary(&self) -> String {
        format!(
            "digest: {} lines, {} bytes, ~{} tokens -> {}{}",
            self.digest.lines,
            self.bytes,
            tokens(&self.digest.text),
            self.path.display(),
            match self.digest.next_offset {
                Some(n) => format!(" (next page: --offset {n})"),
                None => String::new(),
            }
        )
    }
}

/// the directory the generated views are written under: `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else
/// `<project>/data/knowledge` (never the main checkout's store a worktree reads the catalog from)
pub fn out_dir(project: &Project, data_dir: Option<&Path>) -> PathBuf {
    match data_dir {
        Some(d) => d.to_path_buf(),
        None => std::env::var_os("CARDGEN_KNOWLEDGE_DIR")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| project.data("knowledge")),
    }
}

/// Writes a generated file: its directory is created, the text goes to `<path>.tmp` and is renamed over `path`.
pub fn write_file(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parsed `digest` arguments.
#[derive(Debug, Default)]
pub struct Args {
    pub deck: Option<String>,
    pub ident: Option<u8>,
    pub role: Option<String>,
    pub limit: Option<usize>,
    pub offset: usize,
    pub out: Option<PathBuf>,
    pub budget_tokens: Option<usize>,
    pub stdout: bool,
    pub data_dir: Option<PathBuf>,
}

impl Args {
    pub fn parse(args: &[String]) -> Result<Args, FindError> {
        let mut a = Args::default();
        let mut seen: Vec<&str> = vec![];
        let mut i = 0;
        while i < args.len() {
            let flag = args[i].as_str();
            i += 1;
            if flag == "--stdout" {
                a.stdout = true;
                continue;
            }
            if !["--deck", "--ident", "--role", "--limit", "--offset", "--out", "--budget-tokens", "--data-dir"]
                .contains(&flag)
            {
                return usage(format!("unknown argument '{flag}'"));
            }
            let Some(v) = args.get(i).filter(|v| !v.starts_with("--")) else {
                return usage(format!("{flag} needs a value"));
            };
            i += 1;
            if seen.contains(&flag) {
                return usage(format!("{flag} given twice"));
            }
            seen.push(flag);
            let number = |what: &str| {
                v.parse::<usize>().map_err(|_| FindError::Usage(format!("{what} needs a number, got '{v}'")))
            };
            match flag {
                "--deck" => a.deck = Some(v.clone()),
                "--ident" => a.ident = Some(knowledge::parse_ident(v).map_err(FindError::Usage)?),
                "--role" => {
                    roles::check_role(v).map_err(FindError::Usage)?;
                    a.role = Some(v.clone());
                }
                "--limit" => a.limit = Some(number("--limit")?),
                "--offset" => a.offset = number("--offset")?,
                "--budget-tokens" => a.budget_tokens = Some(number("--budget-tokens")?),
                "--out" => a.out = Some(PathBuf::from(v)),
                _ => a.data_dir = Some(PathBuf::from(v)),
            }
        }
        if a.deck.is_none() && a.ident.is_none() {
            return usage("digest needs --deck KEY or --ident LETTERS");
        }
        if a.role.is_none() && (a.limit.is_some() || a.offset > 0) {
            return usage("--limit and --offset page through one role's cards: add --role R");
        }
        Ok(a)
    }

    /// the default file name stem: the deck key (or `ident-LETTERS`), then `.role` and `.offset` for a page
    fn stem(&self) -> String {
        let mut s = match (&self.deck, self.ident) {
            (Some(d), None) => d.clone(),
            (Some(d), Some(i)) => format!("{d}-{}", ident_label(i)),
            (None, i) => format!("ident-{}", ident_label(i.unwrap_or(0))),
        };
        if let Some(r) = &self.role {
            s.push('.');
            s.push_str(r);
            if self.offset > 0 {
                s.push_str(&format!(".{}", self.offset));
            }
        }
        s
    }
}

/// `cardgen digest`: builds the digest and writes it (default `<knowledge>/digests/<deck>.txt`). Read-only except for
/// that file.
pub fn command(project: &Project, args: &[String]) -> Result<Output, FindError> {
    let a = Args::parse(args)?;
    let deck = a.deck.as_deref().map(|k| knowledge::load_deck(project, k)).transpose()?;
    let (index, table) = open(project, a.data_dir.as_deref()).map_err(FindError::Failed)?;
    let dir = knowledge::knowledge_dir(project, a.data_dir.as_deref()).map_err(FindError::Failed)?;
    let prev = Prev::load(&dir);
    let spec = Spec {
        deck: deck.as_ref(),
        ident: a.ident,
        role: a.role.as_deref(),
        limit: a.limit,
        offset: a.offset,
        budget_tokens: a.budget_tokens,
        prev: &prev,
    };
    let digest = build(&index, &table, &spec)?;
    let path = a
        .out
        .clone()
        .unwrap_or_else(|| out_dir(project, a.data_dir.as_deref()).join("digests").join(format!("{}.txt", a.stem())));
    write_file(&path, &digest.text).map_err(FindError::Failed)?;
    let bytes = digest.text.len();
    Ok(Output { digest, path, bytes })
}
