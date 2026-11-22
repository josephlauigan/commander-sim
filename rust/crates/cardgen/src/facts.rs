//! What the repository knows about a catalog card besides its Oracle text (phase 4 of documents/card-knowledge.md):
//! the engine's model status, which of JD's decks play it, and what the `audit/<deck>/FINDINGS.md` files say about it.
//! `find` (JSON rows, `--full`, the `--status` and `--in-deck` filters), the digest (`m:`, `d:`, `k:` flags) and the
//! map read the same [`Facts`], so they cannot disagree.
//!
//! Every fact is resolved to a catalog card index once, when the facts are loaded: never by name at flag time.
//!
//! * **Model status** comes from the code `cardgen audit` runs ([`Audit::card_status`]) for every card that has a
//!   record in `data/cards.json` (the cards `cardgen export` builds from the same inputs). A card of any deck of
//!   `mine` is audited as one of JD's cards (`mine = true`, as `audit --mine` does), every other card as an outside
//!   deck plays it. A card with no engine record has no status: it is never guessed, and never `Full`.
//! * **Deck usage**: a card is in a deck of `data/decks.json` `mine` when its list or its commanders name it (by
//!   folded name or face name, [`Index::lookup`]; basic lands included). One bit per deck.
//! * **Findings**: a line of `audit/<key>/FINDINGS.md` of a deck of `mine` mentions a card when the card's full name or
//!   any face name occurs word-bounded and case-sensitive (the catalog spelling or its accent-folded spelling). A name
//!   of 4 characters or fewer (`Opt`) only counts inside `**...**`, in a backtick span or on a `=>` swap line, so
//!   `Option` never mentions `Opt`.

use crate::audit::{Audit, rank};
use crate::data::Project;
use crate::digest::{self, Context};
use crate::export::Inputs;
use crate::knowledge::{FindError, Index, fold, fold_case};
use crate::map::load_mine;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// the longest `text` and `heading` of a finding, in characters
pub const TEXT_CHARS: usize = 200;
/// a name this short (in characters) only counts as a mention inside bold, a code span or a `=>` swap line
pub const SHORT_NAME: usize = 4;

/// the statuses `--status` takes besides `none` (the engine's own, lower-case)
pub const STATUS_WORDS: [&str; 6] = ["full", "full-auto", "approximate", "partial", "unmodeled", "none"];

/// One line of a `FINDINGS.md` that mentions a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// the file, relative to the repository: `audit/<deck key>/FINDINGS.md`
    pub file: String,
    /// the line number, from 1
    pub line: usize,
    /// the nearest heading line (a line that starts with `#`) before this line; empty when there is none
    pub heading: String,
    /// the line, cut to [`TEXT_CHARS`] characters
    pub text: String,
}

/// The one-letter digest code of a status: `F` Full, `f` Full-auto, `A` Approximate, `P` Partial, `U` Unmodeled.
pub fn status_code(status: &str) -> &'static str {
    match status {
        "Full" => "F",
        "Full-auto" => "f",
        "Approximate" => "A",
        "Partial" => "P",
        "Unmodeled" => "U",
        _ => "-",
    }
}

/// The engine side: status and note per catalog card, and what could not be matched.
#[derive(Clone, Debug, Default)]
pub struct Engine {
    /// whether the project has engine data at all (`data/carddb.txt`); a project without it has no records
    pub present: bool,
    /// (status, note) by catalog card index: only cards that have an engine record
    pub status: HashMap<u32, (String, String)>,
    /// engine card names (cards.json records) that resolve to no catalog card
    pub unresolved: Vec<String>,
    /// catalog cards that two engine records resolve to: `card: kept record (status), other record (status)`
    pub conflicts: Vec<String>,
    /// how many engine records there are
    pub records: usize,
    /// why the engine facts cannot be used (`data/cards.json` missing, unreadable, malformed or stale): the project has
    /// engine data but no status is shown, and no read command regenerates the export
    pub unavailable: Option<String>,
}

impl Engine {
    /// the one-line note a command prints when the engine facts are unavailable
    pub fn note(&self) -> Option<String> {
        self.unavailable.as_ref().map(|why| format!("(engine status unavailable: {why}; run cardgen export)"))
    }
}

/// the most decks of `mine` the facts hold (one bit each)
pub const MAX_DECKS: usize = 64;
/// the most decks the digest's `d:` letters can name (`A`-`Z`, then `a`-`z`)
pub const MAX_LETTERS: usize = 52;

/// Everything phase 4 joins to a catalog card.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    pub engine: Engine,
    /// the keys of the decks of `mine`, in the order of `decks.json`; the bit of a deck is its position
    pub keys: Vec<String>,
    /// the letter of each deck (`Context::letters`), for the first [`MAX_LETTERS`] decks
    pub letters: Vec<(char, String)>,
    /// per card index, a bit per deck of `mine` that plays the card
    pub deck_bits: Vec<u64>,
    /// per card index, the lines of the decks' `FINDINGS.md` files that mention it (file order, then line order)
    pub findings: HashMap<u32, Vec<Finding>>,
    /// the `FINDINGS.md` files that were read
    pub files: Vec<String>,
}

impl Facts {
    /// Loads the facts for `index`: the engine (which builds every card of the simulator and audits it, about 0.2 s)
    /// with `engine`, the deck bits and the findings with `decks`.
    pub fn load(project: &Project, index: &Index, engine: bool, decks: bool) -> Result<Facts, String> {
        let mut f = if decks { Facts::decks_and_findings(project, index)? } else { Facts::default() };
        if engine {
            f.engine = engine_facts(project, index)?;
        }
        Ok(f)
    }

    /// The deck bits and the findings (no engine).
    pub fn decks_and_findings(project: &Project, index: &Index) -> Result<Facts, String> {
        let path = project.data("decks.json");
        let decks = if path.is_file() { load_mine(&path)? } else { vec![] };
        if decks.len() > MAX_DECKS {
            return Err(format!(
                "{}: {} decks in mine; at most {MAX_DECKS} are supported (one bit per deck)",
                path.display(),
                decks.len()
            ));
        }
        let keys: Vec<String> = decks.iter().map(|d| d.deck.key.clone()).collect();
        let mut deck_bits = vec![0u64; index.cards.len()];
        for (bit, d) in decks.iter().enumerate() {
            let names = d.deck.cards.iter().chain(std::iter::once(&d.deck.commander)).chain(d.deck.commander2.iter());
            for n in names.filter(|n| !n.trim().is_empty()) {
                for i in digest::resolve(index, n) {
                    deck_bits[i as usize] |= 1 << bit;
                }
            }
        }
        let (findings, files) = read_findings(&project.root, &keys, index)?;
        Ok(Facts { engine: Engine::default(), letters: Context::letters(&keys), keys, deck_bits, findings, files })
    }

    /// the bit of a deck of `mine`; an unknown key is a usage error that lists the keys
    pub fn deck_bit(&self, key: &str) -> Result<u64, FindError> {
        let want = key.to_lowercase();
        match self.keys.iter().position(|k| k.to_lowercase() == want) {
            Some(n) => Ok(1 << n),
            None => Err(FindError::Usage(format!(
                "unknown deck '{key}' for --in-deck/--not-in-deck; the decks of mine are: {}",
                if self.keys.is_empty() { "(none)".to_string() } else { self.keys.join(", ") }
            ))),
        }
    }

    /// [`Facts::context`], but a usage error when the digest's `d:` letters cannot name every deck of `mine`
    pub fn context_checked(&self, current: Option<&str>) -> Result<Context, FindError> {
        if self.keys.len() > MAX_LETTERS {
            return Err(FindError::Failed(format!(
                "{} decks in mine: the digest's d: letters (A-Z, a-z) name at most {MAX_LETTERS}",
                self.keys.len()
            )));
        }
        Ok(self.context(current))
    }

    /// the keys of the decks of `mine` that play card `i`
    pub fn decks_of(&self, i: u32) -> Vec<&str> {
        let bits = self.deck_bits.get(i as usize).copied().unwrap_or(0);
        self.keys.iter().enumerate().filter(|(n, _)| bits & (1 << n) != 0).map(|(_, k)| k.as_str()).collect()
    }

    pub fn findings_of(&self, i: u32) -> &[Finding] {
        self.findings.get(&i).map(Vec::as_slice).unwrap_or(&[])
    }

    /// (status, note) of card `i`, when it has an engine record
    pub fn status_of(&self, i: u32) -> Option<(&str, &str)> {
        self.engine.status.get(&i).map(|(s, n)| (s.as_str(), n.as_str()))
    }

    /// The digest [`Context`]: the status letters, the deck bits and letters, the findings counts, and `current` (the
    /// digest's own deck, left out of its `d:` flag) when it is a deck of `mine`.
    pub fn context(&self, current: Option<&str>) -> Context {
        Context {
            model_status: self
                .engine
                .status
                .iter()
                .filter(|(_, (s, _))| status_code(s) != "-")
                .map(|(i, (s, _))| (*i, status_code(s).to_string()))
                .collect(),
            deck_bits: self.deck_bits.clone(),
            deck_letters: self.letters.clone(),
            current: current.and_then(|k| self.keys.iter().position(|x| x == k)).map(|n| 1u64 << n),
            findings: self.findings.iter().map(|(i, v)| (*i, v.len())).collect(),
            engine_note: self.engine.note(),
        }
    }
}

// ------------------------------------------------------------------------------------------------ engine

/// the card names of an exported `data/cards.json` (`{"version": .., "cards": [{"name": ..}, ..]}`), or why it cannot
/// be used
fn read_export(path: &std::path::Path) -> Result<BTreeSet<String>, String> {
    #[derive(serde::Deserialize)]
    struct Rec {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct File {
        #[allow(dead_code)]
        version: serde_json::Value,
        cards: Vec<Rec>,
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err("data/cards.json is missing".into()),
        Err(e) => return Err(format!("data/cards.json cannot be read ({e})")),
    };
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("data/cards.json is not valid JSON ({e})"))?;
    let file: File = serde_json::from_value(value)
        .map_err(|e| format!("data/cards.json is not {{\"version\": .., \"cards\": [..]}} ({e})"))?;
    Ok(file.cards.into_iter().map(|r| r.name).collect())
}

/// The engine records of the project, resolved to catalog cards. A project without `data/carddb.txt` has none.
pub fn engine_facts(project: &Project, index: &Index) -> Result<Engine, String> {
    if !project.data("carddb.txt").is_file() {
        return Ok(Engine::default());
    }
    let unavailable = |why: String| Ok(Engine { present: true, unavailable: Some(why), ..Engine::default() });
    // the exported `data/cards.json` is the engine's record set; a missing, malformed or stale export makes the facts
    // unavailable (a read command never regenerates it)
    let exported = match read_export(&project.data("cards.json")) {
        Ok(names) => names,
        Err(why) => return unavailable(why),
    };
    let inp = Inputs::load(project)?;
    let built = inp.build();
    let rebuilt: BTreeSet<&str> = built.cards.keys().map(String::as_str).collect();
    let differ: Vec<&str> = exported
        .iter()
        .map(String::as_str)
        .filter(|n| !rebuilt.contains(n))
        .chain(rebuilt.iter().copied().filter(|n| !exported.contains(*n)))
        .collect();
    if !differ.is_empty() {
        let shown: Vec<&str> = differ.iter().take(3).copied().collect();
        return unavailable(format!(
            "data/cards.json is stale: {} card name(s) differ from the data files, e.g. {}",
            differ.len(),
            shown.join(", ")
        ));
    }
    let audit = Audit::new(&inp, &built.cards)?;
    // a card of any deck of `mine` is audited as JD's decks play it, as `cardgen audit --mine` does
    let mut mine: BTreeSet<&str> = inp.my_cards.iter().flat_map(|(_, c)| c.iter().map(String::as_str)).collect();
    for d in &inp.my {
        mine.insert(&d.commander);
        mine.extend(d.commander2.as_deref());
    }
    let mut e = Engine { present: true, records: exported.len(), ..Engine::default() };
    let mut from: HashMap<u32, &str> = HashMap::new();
    for name in built.cards.keys() {
        let (s, n) = audit.card_status(name, mine.contains(name.as_str()));
        let cards = digest::resolve(index, name);
        if cards.is_empty() {
            e.unresolved.push(name.clone());
        }
        for i in cards {
            let Some(other) = from.get(&i).copied() else {
                from.insert(i, name);
                e.status.insert(i, (s.clone(), n.clone()));
                continue;
            };
            // two engine names for one catalog card (`A` and `A // B`): the lower status wins, so a card is never
            // shown above what any of its records says; on a tie the record named like the catalog card
            let (old, _) = &e.status[&i];
            let key = &index.cards[i as usize].key;
            let keep_new = match rank(&s).cmp(&rank(old)) {
                std::cmp::Ordering::Less => true,
                std::cmp::Ordering::Greater => false,
                std::cmp::Ordering::Equal => fold(name) == *key && fold(other) != *key,
            };
            let (kept, kept_status, dropped, dropped_status) =
                if keep_new { (name.as_str(), &s, other, old.clone()) } else { (other, old, name.as_str(), s.clone()) };
            e.conflicts.push(format!(
                "{}: kept {kept} ({kept_status}), other {dropped} ({dropped_status})",
                index.cards[i as usize].raw.name
            ));
            if keep_new {
                from.insert(i, name);
                e.status.insert(i, (s.clone(), n.clone()));
            }
        }
    }
    Ok(e)
}

// ------------------------------------------------------------------------------------------------ findings

struct Needle {
    text: String,
    short: bool,
    ends_word: bool,
    cards: Vec<u32>,
}

/// every name of the catalog (full names and face names, as spelled and accent-folded) with the cards it stands for,
/// found by the first word the name starts with
struct Needles {
    all: Vec<Needle>,
    by_token: HashMap<String, Vec<usize>>,
    odd: Vec<usize>,
}

fn word(c: char) -> bool {
    c.is_alphanumeric()
}

impl Needles {
    fn build(index: &Index) -> Needles {
        let mut names: HashMap<&str, Vec<u32>> = HashMap::new();
        for c in &index.cards {
            for n in
                std::iter::once(c.raw.name.as_str()).chain(c.raw.card_faces.iter().flatten().map(|f| f.name.as_str()))
            {
                if !n.trim().is_empty() {
                    names.entry(n).or_insert_with(|| digest::resolve(index, n));
                }
            }
        }
        let mut sorted: Vec<(&str, Vec<u32>)> = names.into_iter().collect();
        sorted.sort();
        let mut s = Needles { all: vec![], by_token: HashMap::new(), odd: vec![] };
        for (name, cards) in sorted {
            if cards.is_empty() {
                continue;
            }
            let folded = fold_case(name);
            let short = name.chars().count() <= SHORT_NAME;
            for text in std::iter::once(name.to_string()).chain((folded != name).then_some(folded)) {
                let ends_word = text.chars().next_back().is_some_and(word);
                let n = s.all.len();
                match text.chars().next() {
                    Some(c) if word(c) => {
                        let end = text.find(|c: char| !word(c)).unwrap_or(text.len());
                        s.by_token.entry(text[..end].to_string()).or_default().push(n);
                    }
                    _ => s.odd.push(n),
                }
                s.all.push(Needle { text, short, ends_word, cards: cards.clone() });
            }
        }
        s
    }

    /// the cards one line mentions (`line` as written, and accent-folded)
    fn mentions(&self, line: &str, out: &mut BTreeSet<u32>) {
        let swap = line.contains("=>");
        self.scan(line, swap, out);
        let folded = fold_case(line);
        if folded != line {
            self.scan(&folded, swap, out);
        }
    }

    fn scan(&self, s: &str, swap: bool, out: &mut BTreeSet<u32>) {
        let spans = spans(s);
        let mut prev_word = false;
        for (p, ch) in s.char_indices() {
            let starts = !prev_word;
            prev_word = word(ch);
            if !starts {
                continue;
            }
            let rest = &s[p..];
            let token = if word(ch) {
                let end = rest.find(|c: char| !word(c)).unwrap_or(rest.len());
                self.by_token.get(&rest[..end])
            } else {
                None
            };
            for &n in token.into_iter().flatten().chain(self.odd.iter()) {
                let nd = &self.all[n];
                if !rest.starts_with(&nd.text) {
                    continue;
                }
                let end = p + nd.text.len();
                if nd.ends_word && s[end..].chars().next().is_some_and(word) {
                    continue;
                }
                if nd.short && !swap && !spans.iter().any(|&(a, b)| a <= p && end <= b) {
                    continue;
                }
                out.extend(&nd.cards);
            }
        }
    }
}

/// the byte ranges inside `**...**` pairs and inside backtick pairs
fn spans(s: &str) -> Vec<(usize, usize)> {
    let mut out = vec![];
    let bold: Vec<usize> = s.match_indices("**").map(|(i, _)| i).collect();
    for pair in bold.chunks_exact(2) {
        out.push((pair[0] + 2, pair[1]));
    }
    // code spans (CommonMark): a run of backticks closes at the next run of the same length
    let mut runs: Vec<(usize, usize)> = vec![];
    let mut at = 0;
    while let Some(i) = s[at..].find('`') {
        let start = at + i;
        let len = s[start..].bytes().take_while(|b| *b == b'`').count();
        runs.push((start, len));
        at = start + len;
    }
    let mut n = 0;
    while n < runs.len() {
        let (start, len) = runs[n];
        match runs[n + 1..].iter().position(|r| r.1 == len) {
            Some(k) => {
                let close = n + 1 + k;
                out.push((start + len, runs[close].0));
                n = close + 1;
            }
            None => n += 1,
        }
    }
    out
}

/// the first [`TEXT_CHARS`] characters of the physical line as written (trailing spaces can be a Markdown line break)
fn cut(s: &str) -> String {
    s.chars().take(TEXT_CHARS).collect()
}

/// reads `audit/<key>/FINDINGS.md` of each deck key that has one
fn read_findings(
    root: &std::path::Path,
    keys: &[String],
    index: &Index,
) -> Result<(HashMap<u32, Vec<Finding>>, Vec<String>), String> {
    let mut texts: Vec<(String, String)> = vec![];
    for k in keys {
        let rel = format!("audit/{k}/FINDINGS.md");
        let path = root.join(&rel);
        match std::fs::read_to_string(&path) {
            Ok(t) => texts.push((rel, t)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", path.display())),
        }
    }
    let files: Vec<String> = texts.iter().map(|(f, _)| f.clone()).collect();
    if texts.iter().all(|(_, t)| t.trim().is_empty()) {
        return Ok((HashMap::new(), files));
    }
    let needles = Needles::build(index);
    let mut found: BTreeMap<u32, Vec<Finding>> = BTreeMap::new();
    for (file, text) in &texts {
        let mut heading = String::new();
        // one leading byte order mark is not part of the first line
        for (n, line) in text.strip_prefix('\u{feff}').unwrap_or(text).lines().enumerate() {
            let mut hit = BTreeSet::new();
            needles.mentions(line, &mut hit);
            for i in hit {
                found.entry(i).or_default().push(Finding {
                    file: file.clone(),
                    line: n + 1,
                    heading: heading.clone(),
                    text: cut(line),
                });
            }
            if line.starts_with('#') {
                heading = cut(line.trim_end());
            }
        }
    }
    Ok((found.into_iter().collect(), files))
}

/// the `--status` words of a list: lower-case, checked against [`STATUS_WORDS`]
pub fn parse_status_list(list: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = vec![];
    for w in list.split(',') {
        let w = w.trim().to_lowercase();
        if !STATUS_WORDS.contains(&w.as_str()) {
            return Err(format!("--status takes a comma-separated list of {}, got '{}'", STATUS_WORDS.join(", "), w));
        }
        if !out.contains(&w) {
            out.push(w);
        }
    }
    Ok(out)
}

/// whether a card with this engine status (`None` = no record) is in a `--status` list
pub fn status_selected(list: &[String], status: Option<&str>) -> bool {
    let word = status.map_or("none".to_string(), |s| s.to_lowercase());
    list.contains(&word)
}
