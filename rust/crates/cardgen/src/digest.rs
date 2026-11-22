//! `cardgen digest`: the compact per-deck view of the catalog (layer 2 of documents/card-knowledge.md).
//!
//! One line per card, `Name|{cost}|Type—Subtypes|roles|gist|flags` (six `|`-separated fields, at most 160 characters).
//! The cards come from [`Index::search`], so the deck's identity ceiling and exclusions are the ones `find` applies;
//! the digest never re-implements them. The gist is a whole-token cut of the Oracle line cited as evidence for the
//! card's primary role (or its first Oracle line that says something when no line is cited), never written text.
//! Nothing here needs the LLM: the role table is read as it is, rules-only or merged, and a card whose roles are not
//! reviewed yet carries a `?` flag. [`flags`] makes every line's flags.
//!
//! Sections: A the deck's own cards (the commander first), B per primary role the best-ranked cards, C cards that
//! share a subtype with the commander's text, D cards new since the previous sync, E the best-ranked cards with no
//! role. `--role R` pages through every card that has R, `--view roleless|new` through the complete lists. Output is
//! deterministic: no timestamps, a fixed order.

use crate::data::Project;
use crate::knowledge::{self, Card, Deck, FindError, Index, Legal, Query, Sort};
use crate::roles::{self, Assignment};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// the default size of a digest, in estimated tokens
pub const DEFAULT_BUDGET_TOKENS: usize = 42_000;
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
/// section E: the best-ranked cards that have no role
pub const CAP_ROLELESS: usize = 60;
/// section D: the most new cards listed (`--view new` pages through all of them)
pub const CAP_NEW: usize = 100;
/// a gist shorter than this is not worth a line: the subtypes give way first
const MIN_GIST_CHARS: usize = 16;

/// The size estimate every budget uses: bytes / 4, rounded up. It is never below characters / 4, so a digest within
/// the budget is within it by either count.
pub fn tokens(s: &str) -> usize {
    s.len().div_ceil(4)
}

// ------------------------------------------------------------------------------------------------ one line

/// How a card's roles were decided (only an unreviewed card gets a flag, `?`).
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
    /// the flag of a line: `?` for a card not reviewed yet, nothing for the others (the rules decided, the LLM
    /// answered, or there are no roles and nothing to ask)
    pub fn flag(self) -> Option<&'static str> {
        match self {
            Review::Uncertain => Some("?"),
            Review::Rules | Review::Llm | Review::None => None,
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
    review_card(a, merged, true)
}

/// [`review`] for a card of known legality: the merge only asks Commander-legal cards (it clears `llm_failed` on the
/// others), so a routed card that is not legal never has an answer, whatever the table says.
pub fn review_card(a: &Assignment, merged: bool, legal: bool) -> Review {
    if !a.needs_llm {
        if a.roles.is_empty() { Review::None } else { Review::Rules }
    } else if merged && legal && !a.llm_failed {
        Review::Llm
    } else {
        Review::Uncertain
    }
}

/// the abbreviation of one face's type line: `Lg` when legendary, then a letter per card type in the order
/// A E L C P B I S, `O` when it has none of them
fn face_abbrev(type_line: &str) -> String {
    let head = type_line.split('—').next().unwrap_or("").to_lowercase();
    let has = |w: &str| head.split_whitespace().any(|x| x == w);
    let mut s = String::new();
    if has("legendary") {
        s.push_str("Lg");
    }
    let letters: String = [
        ("artifact", 'A'),
        ("enchantment", 'E'),
        ("land", 'L'),
        ("creature", 'C'),
        ("planeswalker", 'P'),
        ("battle", 'B'),
        ("instant", 'I'),
        ("sorcery", 'S'),
    ]
    .iter()
    .filter(|(w, _)| has(w))
    .map(|(_, l)| *l)
    .collect();
    s.push_str(if letters.is_empty() { "O" } else { &letters });
    s
}

/// The type abbreviation: per face, `Lg` for a legendary face, then a letter per card type in the order
/// A E L C P B I S (`AC` is an artifact creature, `LgC` a legendary creature), `O` when it has none of them. The faces
/// of a multi-face card are joined with `/` (`C/L` is an MDFC with a land back, `LgC/LgA` Birgi); when every face
/// has the same abbreviation it is written once (`I` for Fire // Ice).
pub fn type_abbrev(c: &Card) -> String {
    let mut faces: Vec<&str> = c.raw.type_line.split(" // ").collect();
    if faces.len() == 1 {
        let from_faces: Vec<&str> =
            c.raw.card_faces.iter().flatten().map(|f| f.type_line.as_str()).filter(|t| !t.is_empty()).collect();
        if from_faces.len() > 1 {
            faces = from_faces;
        }
    }
    let mut parts: Vec<String> = faces.iter().map(|f| face_abbrev(f)).collect();
    parts.dedup();
    if parts.iter().all(|p| *p == parts[0]) {
        parts.truncate(1);
    }
    parts.join("/")
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

/// whether every comma-separated part of the line starts with one of the card's keywords (`Flying, trample`, `Equip
/// {1}`): such a line says little about what the card does
fn keyword_only(c: &Card, line: &str) -> bool {
    let kws: Vec<String> = c.raw.keywords.iter().map(|k| k.to_lowercase()).collect();
    !kws.is_empty()
        && line.split(',').all(|part| {
            let part = part.trim().to_lowercase();
            // a keyword is a whole word: `Enchant` is not `Enchanted creature gets...`, `Equip` is not `Equipped`
            kws.iter().any(|k| part.strip_prefix(k.as_str()).is_some_and(|rest| !rest.starts_with(char::is_alphabetic)))
        })
}

/// The Oracle line a gist is cut from, before cleaning and cutting: the line cited as evidence for `role` (for the
/// primary role when `role` is `None` or cites no line of its own table entry; the normalised text the rules and
/// guards read). For a card with no cited line (no roles, or a hand override) its first Oracle line with text after
/// reminder text is removed, skipping a line that only lists keywords unless there is nothing else. Empty when the
/// card has no such line.
pub fn gist_source(c: &Card, a: &Assignment, role: Option<&str>) -> String {
    let ev = match role {
        Some(r) if a.evidence.contains_key(r) => a.evidence.get(r),
        _ => a.evidence.get(&a.primary),
    };
    if let Some(line) = ev.and_then(|e| e.line) {
        let t = roles::line_text(c, line);
        if !t.is_empty() {
            return t;
        }
    }
    let prompt = roles::prompt_lines(c);
    if let Some((_, t)) = prompt.iter().find(|(_, t)| !keyword_only(c, t)) {
        return t.clone();
    }
    if let Some((_, t)) = prompt.into_iter().next() {
        return t;
    }
    c.lines().iter().map(|(_, l)| roles::normalize(c, l)).find(|t| !t.is_empty()).unwrap_or_default()
}

/// `s` cut at a word boundary to at most `max` characters; a cut text ends in `…` (counted). The result is always a
/// prefix of `s` (plus the `…`). With `whole_tokens` the cut is also never inside a `{…}` symbol or a quoted
/// ability: the last space at which the braces are closed and the quotes balanced is used, so a gist never ends in
/// an open quote. When no such boundary fits the result is empty (the caller writes `-`).
pub fn cut_to(s: &str, max: usize, whole_tokens: bool) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let (mut depth, mut quotes, mut n) = (0usize, 0usize, 0usize);
    let mut best: Option<usize> = None;
    for (i, ch) in s.char_indices() {
        if n + 1 > max {
            break;
        }
        if ch == ' ' && i > 0 && (!whole_tokens || (depth == 0 && quotes % 2 == 0)) {
            let prefix = s[..i].trim_end();
            if !prefix.is_empty() {
                best = Some(prefix.len());
            }
        }
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            '"' => quotes += 1,
            _ => {}
        }
        n += 1;
    }
    best.map(|b| format!("{}…", &s[..b])).unwrap_or_default()
}

/// [`cut_to`] for a gist: whole tokens only.
pub fn cut(s: &str, max: usize) -> String {
    cut_to(s, max, true)
}

/// What else a line can say about a card, by whoever has it: `crate::facts::Facts::context` fills it (empty by
/// default) and [`flags`] prints it. Everything is resolved to card indices once, when the
/// context is built: nothing is looked up by name at flag time.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// engine model status by card index (`m:X`): only cards that have a `cards.json` record
    pub model_status: HashMap<u32, String>,
    /// per card index (aligned with `Index::cards`, may be shorter): a bit per JD deck that plays the card (`d:`)
    pub deck_bits: Vec<u64>,
    /// the letter of each deck bit, in bit order: `(C, "cloud")`; the digest header prints it
    pub deck_letters: Vec<(char, String)>,
    /// the deck bit that section A leaves out of its own `d:` (the digest's own deck)
    pub current: Option<u64>,
    /// `FINDINGS.md` entries that name the card, by card index (`k:N`, only when N > 0)
    pub findings: HashMap<u32, usize>,
    /// the note printed in the header when the engine status is unavailable (`m:` flags are then left out)
    pub engine_note: Option<String>,
    /// the engine facts are available: a card with no engine record then prints `m:N`
    pub engine_available: bool,
}

impl Context {
    /// The letter table for decks in the order of `decks.json` `mine`: the first letter of the key not used yet
    /// (upper case), else the first unused of A to Z, then a to z. Deterministic; at most 52 decks.
    pub fn letters(keys: &[String]) -> Vec<(char, String)> {
        let mut used: Vec<char> = vec![];
        let mut out = vec![];
        for key in keys.iter().take(52) {
            let pick = key
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .map(|c| c.to_ascii_uppercase())
                .chain('A'..='Z')
                .chain('a'..='z')
                .find(|c| !used.contains(c))
                .unwrap_or('?');
            used.push(pick);
            out.push((pick, key.clone()));
        }
        out
    }

    /// the header line that explains the `d:` letters, when there are any
    pub fn legend(&self) -> Option<String> {
        if self.deck_letters.is_empty() {
            return None;
        }
        let list: Vec<String> = self.deck_letters.iter().map(|(c, k)| format!("{c}={k}")).collect();
        let own = self
            .current
            .filter(|b| b.is_power_of_two())
            .and_then(|b| self.deck_letters.get(b.trailing_zeros() as usize))
            .map(|(c, _)| format!(" (this digest: {c})"))
            .unwrap_or_default();
        Some(format!("# d: letters of JD's decks that also play the card: {}{own}", list.join(" ")))
    }

    /// the header line that explains `m:` and `k:`, when the context holds any status, finding or engine note
    pub fn model_legend(&self) -> Option<String> {
        let k = "k:N = lines of JD's audit/*/FINDINGS.md that name the card";
        if self.engine_available || !self.model_status.is_empty() {
            return Some(format!(
                "# m: the simulator's model status of the card: F full (hand-verified), f full-auto (compiled, not \
                 hand-verified), A approximate, P partial (some text ignored), U unmodeled, N = NO engine record: \
                 status unknown, treat as below Full (adding it needs a cards.json entry, cardgen export and cardgen \
                 audit, then JD); a card below F (f A P U N) goes to JD before it is accepted. {k}"
            ));
        }
        if self.engine_note.is_some() || !self.findings.is_empty() {
            return Some(format!("# m: flags are left out: engine status unavailable. {k}"));
        }
        None
    }
}

/// The flags of card `i`, in the order they are printed: `*` in the deck's list, `BAN` or `NL` when the catalog does
/// not call it Commander-legal, `GC` Game Changer, `#N` EDHREC rank, `$N` price in USD, `m:X` model status (`m:N` =
/// no engine record, printed only when the engine facts are available), `d:XY` the JD decks (other than the current
/// one) that play it, `k:N` findings that name it, `?` role tags not reviewed yet (always last). `m:`, `d:` and `k:`
/// print only what the [`Context`] holds. The one place a line's flags are made.
pub fn flags(i: u32, c: &Card, review: Review, in_deck: bool, ctx: &Context) -> Vec<String> {
    let mut flags: Vec<String> = vec![];
    if in_deck {
        flags.push("*".into());
    }
    match c.legal {
        Legal::Legal => {}
        Legal::Banned => flags.push("BAN".into()),
        _ => flags.push("NL".into()),
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
    if let Some(status) = ctx.model_status.get(&i) {
        flags.push(format!("m:{status}"));
    } else if ctx.engine_available {
        flags.push("m:N".into());
    }
    let bits = ctx.deck_bits.get(i as usize).copied().unwrap_or(0) & !ctx.current.unwrap_or(0);
    let decks: String =
        ctx.deck_letters.iter().enumerate().filter(|(n, _)| bits & (1 << n) != 0).map(|(_, (c, _))| *c).collect();
    if !decks.is_empty() {
        flags.push(format!("d:{decks}"));
    }
    if let Some(n) = ctx.findings.get(&i).filter(|n| **n > 0) {
        flags.push(format!("k:{n}"));
    }
    flags.extend(review.flag().map(String::from));
    flags
}

/// Builds digest lines for one catalog.
pub struct Lines<'a> {
    pub index: &'a Index,
    merged: bool,
    ctx: Context,
}

impl<'a> Lines<'a> {
    pub fn new(index: &'a Index, table: &Table) -> Lines<'a> {
        Lines::with_context(index, table, Context::default())
    }

    pub fn with_context(index: &'a Index, table: &Table, ctx: Context) -> Lines<'a> {
        Lines { index, merged: table.merged, ctx }
    }

    pub fn review(&self, i: u32) -> Review {
        let legal = self.index.cards[i as usize].legal == Legal::Legal;
        self.index.roles(i).map_or(Review::None, |a| review_card(a, self.merged, legal))
    }

    /// The digest line of card `i`; `in_deck` adds the `*` flag.
    pub fn line(&self, i: u32, in_deck: bool) -> String {
        self.line_for(i, in_deck, None)
    }

    /// [`Lines::line`] for a page of one role: the gist cites that role's own evidence line.
    pub fn line_for(&self, i: u32, in_deck: bool, role: Option<&str>) -> String {
        let c = &self.index.cards[i as usize];
        let default = Assignment::default();
        let a = self.index.roles(i).unwrap_or(&default);
        let cost = Some(clean(&c.cost())).filter(|s| !s.is_empty()).unwrap_or_else(|| "-".into());
        let roles = if a.roles.is_empty() { "-".to_string() } else { a.roles.join(",") };
        let flags = flags(i, c, self.review(i), in_deck, &self.ctx);
        let flags = if flags.is_empty() { "-".to_string() } else { flags.join(" ") };
        let src = clean(&gist_source(c, a, role));
        let src_len = src.chars().count();
        let subs = c.subtypes();
        let abbrev = type_abbrev(c);
        let len = |s: &str| s.chars().count();
        // the name gives way only when the fields that cannot shrink leave no room for it and a one-character gist
        let rest = len(&cost) + len(&abbrev) + len(&roles) + len(&flags) + 5 + 1;
        let name_room = MAX_LINE_CHARS.saturating_sub(rest).max(2);
        let mut name = clean(&c.raw.name);
        if len(&name) > name_room {
            let cut_name = cut_to(&name, name_room, false);
            name = if cut_name.is_empty() {
                format!("{}…", name.chars().take(name_room - 1).collect::<String>())
            } else {
                cut_name
            };
        }
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
        let cut_gist = if src.is_empty() { String::new() } else { cut(&src, room.min(GIST_CHARS)) };
        let gist = if cut_gist.is_empty() { "-".to_string() } else { cut_gist };
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
    /// Reads `oracle.prev.json` of a knowledge directory. A file that does not parse, has another `meta.schema` than
    /// this build reads, or has a record without an `oracle_id` is [`Prev::Unreadable`]: taking its missing ids for
    /// "nothing" would mark every card new. A valid file with no cards is a valid, empty previous sync.
    pub fn load(dir: &Path) -> Prev {
        #[derive(serde::Deserialize)]
        struct Meta {
            schema: Option<i64>,
        }
        #[derive(serde::Deserialize)]
        struct Id {
            oracle_id: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct File {
            meta: Option<Meta>,
            cards: Vec<Id>,
        }
        let path = dir.join("oracle.prev.json");
        if !path.is_file() {
            return Prev::Missing;
        }
        let bad = |e: String| Prev::Unreadable(format!("{}: {e}", path.display()));
        let file = match std::fs::File::open(&path)
            .map_err(|e| e.to_string())
            .and_then(|f| serde_json::from_reader::<_, File>(std::io::BufReader::new(f)).map_err(|e| e.to_string()))
        {
            Ok(f) => f,
            Err(e) => return bad(e),
        };
        if let Some(schema) = file.meta.and_then(|m| m.schema).filter(|s| *s != knowledge::SCHEMA) {
            return bad(format!("meta.schema is {schema}, this cardgen reads schema {}", knowledge::SCHEMA));
        }
        let mut ids = HashSet::with_capacity(file.cards.len());
        for (n, c) in file.cards.into_iter().enumerate() {
            match c.oracle_id.filter(|id| !id.is_empty()) {
                Some(id) => ids.insert(id),
                None => return bad(format!("record {} has no oracle_id", n + 1)),
            };
        }
        Prev::Ids(ids)
    }
}

/// A complete list of candidates, paged (`--view`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    /// every candidate with no role
    Roleless,
    /// every candidate new since the previous sync
    New,
}

impl View {
    pub fn parse(s: &str) -> Result<View, String> {
        match s {
            "roleless" => Ok(View::Roleless),
            "new" => Ok(View::New),
            _ => Err(format!("--view takes roleless or new, got '{s}'")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            View::Roleless => "roleless",
            View::New => "new",
        }
    }
}

/// What to digest.
pub struct Spec<'a> {
    /// a deck of `data/decks.json`: its identity is the ceiling, its cards are section A, its commanders drive C
    pub deck: Option<&'a Deck>,
    /// an identity limit: the whole identity without a deck, a narrowing of the deck's with one
    pub ident: Option<u8>,
    /// only the cards that have this role (primary first, then the others; all of them, paged by `limit` and `offset`)
    pub role: Option<&'a str>,
    /// a complete paged list instead of the sections: the candidates with no role, or the new ones
    pub view: Option<View>,
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

/// the card of a name: the Commander-legal one when several share it (all of them when none is legal)
pub fn resolve(index: &Index, name: &str) -> Vec<u32> {
    let found = index.lookup(name);
    let legal: Vec<u32> = found.iter().copied().filter(|&i| index.cards[i as usize].legal == Legal::Legal).collect();
    if legal.is_empty() { found.to_vec() } else { legal }
}

/// the deck's commanders as card indices, in order
pub fn commander_cards(index: &Index, deck: &Deck) -> Vec<u32> {
    let mut out: Vec<u32> = vec![];
    for n in std::iter::once(&deck.commander).chain(deck.commander2.iter()).filter(|n| !n.trim().is_empty()) {
        for i in resolve(index, n) {
            if roles::is_deck_card(&index.cards[i as usize]) && !out.contains(&i) {
                out.push(i);
            }
        }
    }
    out
}

/// the cards of a deck as indices: the commanders first, then its list, each name once (the Commander-legal card of
/// a shared name when there is one), the list sorted as `find` sorts
pub fn deck_cards(index: &Index, deck: &Deck) -> Vec<u32> {
    let commanders = commander_cards(index, deck);
    let mut rest: Vec<u32> = vec![];
    for n in deck.cards.iter().filter(|n| !n.trim().is_empty()) {
        for i in resolve(index, n) {
            if roles::is_deck_card(&index.cards[i as usize]) && !commanders.contains(&i) && !rest.contains(&i) {
                rest.push(i);
            }
        }
    }
    index.sort(&mut rest, Sort::Edhrec);
    commanders.into_iter().chain(rest).collect()
}

/// How many of the cards have each role (in any position), the biggest first, ties in the order of the taxonomy;
/// roles no card has are left out.
pub fn role_counts(index: &Index, cards: &[u32]) -> Vec<(&'static str, usize)> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for &i in cards {
        for r in index.roles(i).iter().flat_map(|a| a.roles.iter()) {
            *count.entry(r.as_str()).or_default() += 1;
        }
    }
    let mut v: Vec<(&'static str, usize)> =
        roles::IDS.iter().copied().filter_map(|r| count.get(r).map(|n| (r, *n))).collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    v
}

/// the basic lands of a deck's list with how many copies it runs (a basic is one line of section A)
pub fn basic_lands(index: &Index, deck: &Deck) -> Vec<(String, usize)> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for n in &deck.cards {
        if let Some(&i) = resolve(index, n).first() {
            let c = &index.cards[i as usize];
            if c.raw.type_line.split_whitespace().next() == Some("Basic") {
                *count.entry(c.raw.name.clone()).or_default() += 1;
            }
        }
    }
    let mut v: Vec<(String, usize)> = count.into_iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    v
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

/// `text` with every `named <card name>` reference (the name resolved in the catalog, the longest that resolves)
/// replaced by `@`: a card called `Spirit Guide` is not Spirit synergy
fn mask_named(index: &Index, text: &str) -> String {
    const NAMED: &str = "named ";
    let (mut out, mut rest) = (String::with_capacity(text.len()), text);
    while let Some(p) = rest.find(NAMED) {
        out.push_str(&rest[..p + NAMED.len()]);
        let after = &rest[p + NAMED.len()..];
        let line = after.split('\n').next().unwrap_or("");
        let words: Vec<&str> = line.split(' ').collect();
        let matched = (1..=words.len().min(8)).rev().find_map(|k| {
            let cand = words[..k].join(" ");
            let cand = cand.trim_end_matches([',', '.', ';', ':', '"', ')']);
            (!cand.is_empty() && !index.lookup(cand).is_empty()).then_some(cand.len())
        });
        match matched {
            Some(n) => {
                out.push('@');
                rest = &after[n..];
            }
            None => rest = after,
        }
    }
    out.push_str(rest);
    out
}

/// the Oracle text with reminder text removed, the card's own name as `~` (the text the rules read) and the names of
/// other cards masked, joined
fn plain_text(index: &Index, c: &Card) -> String {
    let text = c.lines().iter().map(|(_, l)| roles::normalize(c, l)).collect::<Vec<_>>().join("\n");
    mask_named(index, &text)
}

/// Every subtype of the catalog that the commanders' Oracle text names (`Dragon`, `Equipment`; a plural names it too).
pub fn commander_subtypes(index: &Index, deck: &Deck) -> Vec<String> {
    let vocab = index.subtype_vocab();
    let mut found: Vec<String> = vec![];
    let names = std::iter::once(&deck.commander).chain(deck.commander2.iter());
    for n in names {
        for i in resolve(index, n) {
            let text = plain_text(index, &index.cards[i as usize]);
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
fn names_subtype(index: &Index, c: &Card, subs: &[String], forms: &[Vec<String>]) -> bool {
    if c.subtypes().iter().any(|s| subs.iter().any(|t| t == s)) {
        return true;
    }
    // a cheap substring test first, then the whole-word test on the text the rules read
    if !forms.iter().flatten().any(|f| c.text.contains(f.as_str())) {
        return false;
    }
    let text = plain_text(index, c);
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
            "I instant, S sorcery, O other; faces of a multi-face card are joined with / (C/L: creature, land back)"
        )
        .into(),
        concat!(
            "# gist: the Oracle line cited for the primary role, reminder text removed, ~ is the card, ",
            "cut at a word boundary (… = cut)"
        )
        .into(),
        concat!(
            "# flags: * in the deck's list; BAN banned / NL not legal in Commander; GC Game Changer; #N EDHREC rank; ",
            "$N price in USD; ? the role tags are not reviewed yet (about the tags, not the card's quality: routed to ",
            "the LLM, no answer, so they may be incomplete or wrong; no ? means the rules or a hand override decided, ",
            "or the LLM answered); - no flags"
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
    build_with(index, table, spec, &Context::default())
}

/// [`build`] with what phase 4 adds to the flags (model status, JD's decks, findings): the lines carry them and the
/// header explains the `m:`, `k:` and deck letters.
pub fn build_with(index: &Index, table: &Table, spec: &Spec, ctx: &Context) -> Result<Digest, FindError> {
    if spec.deck.is_none() && spec.ident.is_none() {
        return usage("digest needs --deck KEY or --ident LETTERS");
    }
    if let Some(role) = spec.role {
        roles::check_role(role).map_err(FindError::Usage)?;
        if spec.view.is_some() {
            return usage("--role and --view are two different pages: give one");
        }
    } else if spec.view.is_none() && (spec.limit.is_some() || spec.offset > 0) {
        return usage("--limit and --offset page through one role's cards or one view: add --role R or --view V");
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
    let lines = Lines::with_context(index, table, ctx.clone());
    let mut head: Vec<String> = vec![format!("# digest: {}", subject(spec.deck, spec.ident))];
    head.push(format!(
        "# role table: {} (roles.json); catalog: Scryfall bulk {}; {} legal cards in the identity, not in the deck",
        table.kind(),
        if index.bulk_updated_at.is_empty() { "unknown" } else { &index.bulk_updated_at },
        num(hits.len())
    ));
    head.extend(legend());
    head.extend(ctx.engine_note.iter().map(|n| format!("# {n}")));
    head.extend(ctx.model_legend());
    head.extend(ctx.legend());
    if !unresolved.is_empty() {
        let shown: Vec<&str> = unresolved.iter().take(10).map(String::as_str).collect();
        let more = if unresolved.len() > 10 { ", ..." } else { "" };
        head.push(format!("# deck names not in the catalog ({}): {}{more}", unresolved.len(), shown.join(", ")));
    }
    match (spec.role, spec.view) {
        (Some(role), _) => role_page(&lines, spec, &hits, role, head),
        (None, Some(view)) => view_page(&lines, spec, &hits, view, head),
        (None, None) => full(&lines, spec, &hits, head),
    }
}

/// a card whose roles list is empty (the rules found none and, in a merged table, neither did the LLM)
fn roleless(index: &Index, i: u32) -> bool {
    index.roles(i).is_none_or(|a| a.primary.is_empty())
}

/// A page of one list of cards: `title` makes the block header from the range and the tail; `role` selects the
/// evidence line of the gists; `id` is the section letter of the block.
fn page(
    l: &Lines,
    spec: &Spec,
    all: &[u32],
    head: Vec<String>,
    id: char,
    role: Option<&str>,
    title: &dyn Fn(&str, &str) -> String,
) -> Result<Digest, FindError> {
    let cap = spec.budget_tokens.unwrap_or(ROLE_PAGE_TOKENS);
    let limit = spec.limit.unwrap_or(usize::MAX);
    let mut used: usize = head.iter().map(|h| tokens(h) + 1).sum::<usize>() + 60;
    let mut shown: Vec<String> = vec![];
    for &i in all.iter().skip(spec.offset) {
        if shown.len() >= limit {
            break;
        }
        let line = l.line_for(i, false, role);
        used += tokens(&line) + 1;
        if used > cap && !shown.is_empty() {
            break;
        }
        shown.push(line);
    }
    let render = |shown: &[String]| -> (String, Option<usize>) {
        let end = spec.offset.saturating_add(shown.len());
        let next = (end < all.len()).then_some(end);
        let range = if shown.is_empty() {
            format!("no cards at offset {} (there are {})", spec.offset, num(all.len()))
        } else {
            format!("cards {}-{} of {}", spec.offset + 1, end, num(all.len()))
        };
        let tail = match next {
            Some(n) => format!("; next page: --offset {n}"),
            None => "; last page".into(),
        };
        let mut text = head.join("\n");
        text.push('\n');
        text.push_str(&title(&range, &tail));
        text.push('\n');
        for s in shown {
            text.push_str(s);
            text.push('\n');
        }
        (text, next)
    };
    let (mut text, mut next) = render(&shown);
    // the estimate above is rough: drop lines until the real text fits, but never below one card
    while tokens(&text) > cap && shown.len() > 1 {
        shown.pop();
        (text, next) = render(&shown);
    }
    if tokens(&text) > cap && !shown.is_empty() {
        return usage(format!(
            "--budget-tokens {cap} cannot hold the page header and its first card (about {} tokens needed)",
            tokens(&text)
        ));
    }
    Ok(Digest {
        text,
        lines: shown.len(),
        sections: vec![Section { id, role: role.map(String::from), lines: shown.len() }],
        total: Some(all.len()),
        next_offset: next,
        b_caps: None,
        over_budget: false,
    })
}

/// `--role R`: every card that has role R in any position, as one list best first; a page of it
fn role_page(l: &Lines, spec: &Spec, hits: &[u32], role: &str, mut head: Vec<String>) -> Result<Digest, FindError> {
    let index = l.index;
    let all: Vec<u32> =
        hits.iter().copied().filter(|&i| index.roles(i).is_some_and(|a| a.roles.iter().any(|r| r == role))).collect();
    head.push(format!(
        "# page: role {role}{}; {} cards have it, in any position (the roles field shows where)",
        if spec.deck.is_some() { ", the deck's own cards left out" } else { "" },
        num(all.len())
    ));
    let title = |range: &str, tail: &str| {
        format!("## B role {role}: {range}, every card that has {role}, best EDHREC rank first, unranked last{tail}")
    };
    page(l, spec, &all, head, 'B', Some(role), &title)
}

/// `--view roleless` or `--view new`: the complete list of such candidates, a page of it
fn view_page(l: &Lines, spec: &Spec, hits: &[u32], view: View, mut head: Vec<String>) -> Result<Digest, FindError> {
    let index = l.index;
    match view {
        View::Roleless => {
            let all: Vec<u32> = hits.iter().copied().filter(|&i| roleless(index, i)).collect();
            head.push(format!("# page: candidates with no role{}: {}", deck_note(spec), num(all.len())));
            let title = |range: &str, tail: &str| {
                format!("## E candidates with no role: {range}, best EDHREC rank first, unranked last{tail}")
            };
            page(l, spec, &all, head, 'E', None, &title)
        }
        View::New => {
            let prev = match spec.prev {
                Prev::Ids(ids) => ids,
                Prev::Missing => {
                    return Err(FindError::Failed(
                        "--view new needs a previous sync: there is no oracle.prev.json (run cardgen bulk-sync twice)"
                            .into(),
                    ));
                }
                Prev::Unreadable(why) => {
                    return Err(FindError::Failed(format!("--view new needs a previous sync: {why}")));
                }
            };
            let all: Vec<u32> =
                hits.iter().copied().filter(|&i| !prev.contains(&index.cards[i as usize].raw.oracle_id)).collect();
            head.push(format!("# page: candidates new since the previous sync{}: {}", deck_note(spec), num(all.len())));
            let title = |range: &str, tail: &str| {
                format!("## D new since the previous sync: {range}, best EDHREC rank first, unranked last{tail}")
            };
            page(l, spec, &all, head, 'D', None, &title)
        }
    }
}

fn deck_note(spec: &Spec) -> &'static str {
    if spec.deck.is_some() { " (the deck's own cards left out)" } else { "" }
}

/// the header line of a commander: name, cost, type line and its full Oracle text (lines joined with ` / `)
fn commander_line(c: &Card) -> String {
    let cost = c.cost();
    let text = c.lines().iter().map(|(_, l)| *l).collect::<Vec<_>>().join(" / ");
    let mut s = format!("# commander: {}", c.raw.name);
    if !cost.is_empty() {
        s.push_str(&format!(" {cost}"));
    }
    s.push_str(&format!(", {}", c.raw.type_line));
    if !text.is_empty() {
        s.push_str(&format!(": {text}"));
    }
    s
}

/// the default digest: sections A to E within the budget
fn full(l: &Lines, spec: &Spec, hits: &[u32], mut head: Vec<String>) -> Result<Digest, FindError> {
    let index = l.index;
    let budget = spec.budget_tokens.unwrap_or(DEFAULT_BUDGET_TOKENS);
    // A: the deck's own cards, the commanders first
    let own = spec.deck.map(|d| deck_cards(index, d)).unwrap_or_default();
    let counts = role_counts(index, &own);
    // the deck block of the header: what the deck is and does
    let mut block: Vec<String> = vec![];
    if let Some(d) = spec.deck {
        for i in commander_cards(index, d) {
            block.push(commander_line(&index.cards[i as usize]));
        }
        block.push(if counts.is_empty() {
            "# deck roles: none".to_string()
        } else {
            format!(
                "# deck roles (cards that have the role, any position): {}",
                counts.iter().map(|(r, n)| format!("{r} {n}")).collect::<Vec<_>>().join(", ")
            )
        });
        let basics = basic_lands(index, d);
        if !basics.is_empty() {
            block.push(format!(
                "# basic lands in the list (one line each in A): {}",
                basics.iter().map(|(n, c)| format!("{n} x{c}")).collect::<Vec<_>>().join(", ")
            ));
        }
        // a card of the list the catalog does not call Commander-legal is still the deck's own: say so
        let illegal: Vec<&str> = own
            .iter()
            .map(|&i| &index.cards[i as usize])
            .filter(|c| c.legal != Legal::Legal)
            .map(|c| c.raw.name.as_str())
            .collect();
        if !illegal.is_empty() {
            block.push(format!(
                "# deck cards the catalog does not list as Commander-legal ({}): {}",
                illegal.len(),
                illegal.join(", ")
            ));
        }
    }
    block.push(
        "# absent here is not the same as does not exist: check with find or --role before saying there is no option"
            .into(),
    );
    head.splice(2..2, block);
    // the deck's most common roles get the bigger cap; fixing does not in a one-colour deck, where it is lands
    let ident =
        knowledge::effective_ident(&Query { ident: spec.ident, ..Query::default() }, spec.deck)?.unwrap_or(0b11111);
    let common: Vec<&str> = counts
        .iter()
        .map(|(r, _)| *r)
        .filter(|r| !(ident.count_ones() <= 1 && *r == "fixing"))
        .take(TOP_ROLES)
        .collect();
    // C: synergy with the commanders' subtypes, built first so that B can leave its cards out
    let subs = spec.deck.map(|d| commander_subtypes(index, d)).unwrap_or_default();
    let forms: Vec<Vec<String>> = subs.iter().map(|s| plural_forms(s)).collect();
    let synergy: Vec<u32> = if subs.is_empty() {
        vec![]
    } else {
        hits.iter()
            .copied()
            .filter(|&i| names_subtype(index, &index.cards[i as usize], &subs, &forms))
            .take(CAP_SYNERGY)
            .collect()
    };
    let in_c: HashSet<u32> = synergy.iter().copied().collect();
    // D: new since the previous sync, the first CAP_NEW of them
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
    let in_d: HashSet<u32> = new_cards.iter().flatten().take(CAP_NEW).copied().collect();
    // B: the best-ranked cards of each primary role, leaving out those of C and D; E: the best-ranked cards with
    // no role
    let mut ranked: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    let mut no_role: Vec<u32> = vec![];
    for &i in hits.iter().filter(|i| !in_c.contains(i) && !in_d.contains(i)) {
        match index.roles(i).filter(|a| !a.primary.is_empty()) {
            None => no_role.push(i),
            Some(a) => {
                let role =
                    roles::IDS.iter().copied().find(|r| *r == a.primary).expect("a primary role is in the taxonomy");
                if index.cards[i as usize].raw.edhrec_rank.is_some() {
                    ranked.entry(role).or_default().push(i);
                }
            }
        }
    }
    let roleless_total = hits.iter().filter(|&&i| roleless(index, i)).count();

    let a_block = Block {
        id: 'A',
        role: None,
        header: match spec.deck {
            Some(d) => format!("## A deck {}: its own {} cards, the commander first", d.key, own.len()),
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
        header: format!(
            "## D new since the previous sync: {} of {} cards, not repeated from C (--view new lists them all)",
            v.len().min(CAP_NEW),
            num(v.len())
        ),
        lines: render(v, CAP_NEW),
    });
    let e_lines = render(&no_role, CAP_ROLELESS);

    // assemble at a scale (per mille) for the caps of B and E
    let assemble = |sb: usize, over: bool| -> (String, Vec<Section>) {
        let mut blocks: Vec<Block> =
            vec![Block { id: 'A', role: None, header: a_block.header.clone(), lines: a_block.lines.clone() }];
        for (r, cap, total, ls) in &b_lines {
            let n = (cap * sb / 1000).min(ls.len());
            if n > 0 {
                blocks.push(Block {
                    id: 'B',
                    role: Some(r.to_string()),
                    header: format!(
                        "## B role {r}: best {n} of {} ranked cards with this primary role, not repeated from C \
                         (--role {r} lists them all)",
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
        let n = (CAP_ROLELESS * sb / 1000).min(e_lines.len());
        blocks.push(Block {
            id: 'E',
            role: None,
            header: format!(
                "## E best-ranked cards with no role (no role = no rule matched; not a judgement of the card): \
                 {n} of {} (--view roleless lists them all)",
                num(roleless_total)
            ),
            lines: e_lines[..n].to_vec(),
        });
        let secs: Vec<Section> =
            blocks.iter().map(|b| Section { id: b.id, role: b.role.clone(), lines: b.lines.len() }).collect();
        let size = |id: char| secs.iter().filter(|s| s.id == id).map(|s| s.lines).sum::<usize>();
        let b_roles = secs.iter().filter(|s| s.id == 'B').count();
        let mut h = head.clone();
        h.push(format!(
            "# sections: A {} | B {} ({} roles) | C {} | D {} | E {} of {} candidates with no role",
            size('A'),
            size('B'),
            b_roles,
            size('C'),
            if d_block.is_some() { size('D').to_string() } else { "omitted".into() },
            size('E'),
            num(roleless_total)
        ));
        if d_block.is_none() {
            h.push(format!("# D omitted: {d_note}"));
        }
        let top = if common.is_empty() { "none".to_string() } else { common.join(", ") };
        h.push(format!(
            "# caps: B {CAP_ROLE} per role, {CAP_TOP_ROLE} for the deck's most common roles ({top}); C {CAP_SYNERGY}; \
             D {CAP_NEW}; E {CAP_ROLELESS} (before trimming)"
        ));
        if sb < 1000 || over {
            let mut note = format!("# budget: {} tokens (bytes/4)", num(budget));
            if sb < 1000 {
                note.push_str(&format!(
                    "; B caps trimmed to {} per role and {} for the most common roles, E to {}",
                    CAP_ROLE * sb / 1000,
                    CAP_TOP_ROLE * sb / 1000,
                    CAP_ROLELESS * sb / 1000
                ));
            }
            if over {
                note.push_str("; still over the budget: A, C and D are not trimmed");
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
    let fits = |sb: usize| tokens(&assemble(sb, false).0) <= budget;
    // trim B and E together: the largest scale (0..=1000) that fits
    let (mut sb, mut over) = (1000, false);
    if !fits(sb) {
        let (mut lo, mut hi) = (0usize, 1000usize);
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if fits(mid) { lo = mid } else { hi = mid - 1 }
        }
        sb = lo;
        over = !fits(sb);
    }
    let (text, sections) = assemble(sb, over);
    Ok(Digest {
        text,
        lines: sections.iter().map(|s| s.lines).sum(),
        sections,
        total: None,
        next_offset: None,
        b_caps: (sb < 1000).then_some((CAP_ROLE * sb / 1000, CAP_TOP_ROLE * sb / 1000)),
        over_budget: over,
    })
}

// ------------------------------------------------------------------------------------------------ the command

pub const USAGE: &str =
    "usage: cardgen digest (--deck KEY | --ident LETTERS) [--role R | --view roleless|new] [--limit N] [--offset N]
                      [--out PATH] [--budget-tokens N] [--stdout] [--data-dir DIR]
  (the digest is always written to its file; --stdout also prints it, and the one-line summary goes to stderr)";

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

/// Writes a generated file: its directory is created, the text goes to a temporary file of its own in the same
/// directory (`.NAME.PID.N.tmp`, created with `create_new`, so no two writers share one) and is renamed over `path`.
/// The temporary file is removed on every error path.
pub fn write_file(path: &Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "out".into());
    let tmp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .and_then(|mut f| f.write_all(text.as_bytes()).and_then(|_| f.flush()))
        .map_err(|e| format!("{}: {e}", tmp.display()));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

/// Parsed `digest` arguments.
#[derive(Debug, Default)]
pub struct Args {
    pub deck: Option<String>,
    pub ident: Option<u8>,
    pub role: Option<String>,
    pub view: Option<View>,
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
            if ![
                "--deck",
                "--ident",
                "--role",
                "--view",
                "--limit",
                "--offset",
                "--out",
                "--budget-tokens",
                "--data-dir",
            ]
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
                "--view" => a.view = Some(View::parse(v).map_err(FindError::Usage)?),
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
        if a.role.is_some() && a.view.is_some() {
            return usage("--role and --view are two different pages: give one");
        }
        if a.role.is_none() && a.view.is_none() && (a.limit.is_some() || a.offset > 0) {
            return usage("--limit and --offset page through one role's cards or one view: add --role R or --view V");
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
        if let Some(page) = self.role.as_deref().or(self.view.map(View::name)) {
            s.push('.');
            s.push_str(page);
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
        view: a.view,
        limit: a.limit,
        offset: a.offset,
        budget_tokens: a.budget_tokens,
        prev: &prev,
    };
    // phase 4: the engine status of each card, JD's decks that play it and the FINDINGS lines that name it
    let facts = crate::facts::Facts::load(project, &index, true, true).map_err(FindError::Failed)?;
    let ctx = facts.context_checked(deck.as_ref().map(|d| d.key.as_str()))?;
    let digest = build_with(&index, &table, &spec, &ctx)?;
    let path = a
        .out
        .clone()
        .unwrap_or_else(|| out_dir(project, a.data_dir.as_deref()).join("digests").join(format!("{}.txt", a.stem())));
    write_file(&path, &digest.text).map_err(FindError::Failed)?;
    let bytes = digest.text.len();
    Ok(Output { digest, path, bytes })
}
