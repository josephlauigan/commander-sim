//! The in-memory card index over `data/knowledge/oracle.json` and `cardgen find`.
//!
//! `oracle.json` is loaded with a typed serde struct (not the ordered `J` tree) into an [`Index`]: per card the colour
//! identity bits, cmc, type flags, the lower-case type line and names, the Oracle text of every face, the commander
//! legality, the Game Changer flag, the EDHREC rank and the price. Every query is a linear scan. Nothing is persisted.
//! The command is read-only. See documents/card-knowledge.md.

use crate::data::Project;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// the `oracle.json` schema this code reads (the one `bulk-sync` writes)
pub const SCHEMA: i64 = crate::bulk::SCHEMA;

/// Why a `find` stopped: bad arguments (the caller adds the usage text) or a failure with its own message.
#[derive(Debug)]
pub enum FindError {
    Usage(String),
    Failed(String),
}

impl std::fmt::Display for FindError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FindError::Usage(m) | FindError::Failed(m) => f.write_str(m),
        }
    }
}

fn usage<T>(msg: impl Into<String>) -> Result<T, FindError> {
    Err(FindError::Usage(msg.into()))
}

fn failed<T>(msg: impl Into<String>) -> Result<T, FindError> {
    Err(FindError::Failed(msg.into()))
}

// ------------------------------------------------------------------------------------------------ loading

#[derive(Deserialize)]
struct Oracle {
    meta: RawMeta,
    cards: Vec<Raw>,
}

#[derive(Deserialize)]
struct RawMeta {
    schema: i64,
    #[serde(default)]
    bulk_updated_at: String,
    #[serde(default)]
    count: usize,
}

/// `legalities` of a slim record: only `commander` is kept
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Legalities {
    pub commander: Option<String>,
}

/// One face of a multi-face card. Fields are absent (not null) when they do not apply.
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Face {
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: String,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
}

/// A slim record of `oracle.json`. Unknown fields are ignored.
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Raw {
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    pub type_line: String,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub color_identity: Vec<String>,
    pub keywords: Vec<String>,
    pub produced_mana: Option<Vec<String>>,
    pub layout: String,
    pub game_changer: bool,
    pub legalities: Legalities,
    pub card_faces: Option<Vec<Face>>,
    pub oracle_id: String,
    pub edhrec_rank: Option<u32>,
    pub usd: Option<String>,
    pub released_at: String,
    pub set: String,
    pub rarity: String,
    pub reserved: bool,
    pub all_parts_meld: Option<bool>,
    pub meld_parts: Option<Vec<String>>,
}

/// Commander legality as Scryfall states it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Legal {
    Legal,
    Banned,
    NotLegal,
    Restricted,
    Unknown,
}

impl Legal {
    fn parse(s: Option<&str>) -> Legal {
        match s {
            Some("legal") => Legal::Legal,
            Some("banned") => Legal::Banned,
            Some("not_legal") => Legal::NotLegal,
            Some("restricted") => Legal::Restricted,
            _ => Legal::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Legal::Legal => "legal",
            Legal::Banned => "banned",
            Legal::NotLegal => "not_legal",
            Legal::Restricted => "restricted",
            Legal::Unknown => "unknown",
        }
    }
}

pub const T_LAND: u16 = 1;
pub const T_CREATURE: u16 = 1 << 1;
pub const T_INSTANT: u16 = 1 << 2;
pub const T_SORCERY: u16 = 1 << 3;
pub const T_ARTIFACT: u16 = 1 << 4;
pub const T_ENCHANTMENT: u16 = 1 << 5;
pub const T_PLANESWALKER: u16 = 1 << 6;
pub const T_BATTLE: u16 = 1 << 7;
pub const T_LEGENDARY: u16 = 1 << 8;

const TYPE_WORDS: [(&str, u16); 9] = [
    ("land", T_LAND),
    ("creature", T_CREATURE),
    ("instant", T_INSTANT),
    ("sorcery", T_SORCERY),
    ("artifact", T_ARTIFACT),
    ("enchantment", T_ENCHANTMENT),
    ("planeswalker", T_PLANESWALKER),
    ("battle", T_BATTLE),
    ("legendary", T_LEGENDARY),
];

const COLORS: [char; 5] = ['W', 'U', 'B', 'R', 'G'];

/// words of a type line: split on anything but letters, digits, apostrophes and hyphens (`Creature — Elf // Instant`)
fn words(line: &str) -> impl Iterator<Item = &str> {
    line.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-')).filter(|w| !w.is_empty())
}

/// One card of the index: the record plus what the queries use.
#[derive(Debug)]
pub struct Card {
    pub raw: Raw,
    /// WUBRG bits (W = 1 ... G = 16); colourless = 0
    pub ident: u8,
    pub cmc: f32,
    /// `T_*` flags from the type line of the card and of every face
    pub types: u16,
    pub legal: Legal,
    pub usd: Option<f32>,
    /// the lower-case full name
    pub name_lc: String,
    /// the lower-case full name and every face name, one per line
    pub names_lc: String,
    /// the lower-case type line of the card and of every face
    pub type_lc: String,
    /// the Oracle text of the card and of every face, joined with `\n//\n`
    pub text: String,
}

impl Card {
    fn new(raw: Raw) -> Card {
        let mut ident = 0u8;
        for c in &raw.color_identity {
            if let Some(i) = COLORS.iter().position(|k| c.chars().next() == Some(*k)) {
                ident |= 1 << i;
            }
        }
        let faces = raw.card_faces.as_deref().unwrap_or(&[]);
        let name_lc = raw.name.to_lowercase();
        let mut names_lc = name_lc.clone();
        for f in faces {
            names_lc.push('\n');
            names_lc.push_str(&f.name.to_lowercase());
        }
        let mut type_lc = raw.type_line.to_lowercase();
        for f in faces {
            let t = f.type_line.to_lowercase();
            if !t.is_empty() && !type_lc.contains(&t) {
                type_lc.push_str(" // ");
                type_lc.push_str(&t);
            }
        }
        let mut types = 0;
        for w in words(&type_lc) {
            for (name, bit) in TYPE_WORDS {
                if w == name {
                    types |= bit;
                }
            }
        }
        let mut parts: Vec<&str> = vec![];
        for t in std::iter::once(raw.oracle_text.as_deref()).chain(faces.iter().map(|f| f.oracle_text.as_deref())) {
            match t {
                Some(t) if !t.is_empty() && !parts.contains(&t) => parts.push(t),
                _ => {}
            }
        }
        let text = parts.join("\n//\n");
        Card {
            ident,
            cmc: raw.cmc as f32,
            types,
            legal: Legal::parse(raw.legalities.commander.as_deref()),
            usd: raw.usd.as_deref().and_then(|u| u.parse::<f32>().ok()),
            name_lc,
            names_lc,
            type_lc,
            text,
            raw,
        }
    }

    /// the identity as letters in WUBRG order (empty = colourless)
    pub fn identity(&self) -> String {
        ident_string(self.ident)
    }

    pub fn is_meld(&self) -> bool {
        self.raw.all_parts_meld == Some(true)
    }

    fn has_type_word(&self, word: &str) -> bool {
        match TYPE_WORDS.iter().find(|(n, _)| *n == word) {
            Some((_, bit)) => self.types & bit != 0,
            None => words(&self.type_lc).any(|w| w == word),
        }
    }
}

fn ident_string(bits: u8) -> String {
    COLORS.iter().enumerate().filter(|(i, _)| bits & (1 << i) != 0).map(|(_, c)| *c).collect()
}

/// The whole catalog in memory, with name lookups.
pub struct Index {
    pub cards: Vec<Card>,
    pub bulk_updated_at: String,
    /// lower-case full name -> every card with it (15 names are shared by two or more cards)
    names: HashMap<String, Vec<u32>>,
    /// lower-case face name (when different from the full name) -> every card with such a face
    faces: HashMap<String, Vec<u32>>,
}

impl Index {
    /// Reads `oracle.json`; a missing file, bad JSON or another `meta.schema` is an error.
    pub fn load(path: &Path) -> Result<Index, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let oracle: Oracle =
            serde_json::from_reader(std::io::BufReader::with_capacity(1 << 20, file)).map_err(|e| {
                format!(
                    "{}: cannot read it ({e}); is it from a different cardgen? run `cardgen bulk-sync` to rebuild it",
                    path.display()
                )
            })?;
        if oracle.meta.schema != SCHEMA {
            return Err(format!(
                "{}: meta.schema is {}, this cardgen reads schema {SCHEMA}; run `cardgen bulk-sync` to rebuild it",
                path.display(),
                oracle.meta.schema
            ));
        }
        if oracle.meta.count != 0 && oracle.meta.count != oracle.cards.len() {
            return Err(format!(
                "{}: meta.count is {} but the file holds {} cards; run `cardgen bulk-sync` to rebuild it",
                path.display(),
                oracle.meta.count,
                oracle.cards.len()
            ));
        }
        let mut idx = Index {
            cards: oracle.cards.into_iter().map(Card::new).collect(),
            bulk_updated_at: oracle.meta.bulk_updated_at,
            names: HashMap::new(),
            faces: HashMap::new(),
        };
        for (i, c) in idx.cards.iter().enumerate() {
            idx.names.entry(c.name_lc.clone()).or_default().push(i as u32);
            let mut seen: Vec<String> = vec![];
            for f in c.raw.card_faces.as_deref().unwrap_or(&[]) {
                let n = f.name.to_lowercase();
                if n != c.name_lc && !seen.contains(&n) {
                    idx.faces.entry(n.clone()).or_default().push(i as u32);
                    seen.push(n);
                }
            }
        }
        Ok(idx)
    }

    /// The cards a name stands for (case-insensitive): every card with that full name; only when there is none, the
    /// cards that have a face of that name. An exact full name therefore beats a face name (`Lightning Bolt` is the
    /// instant, not the `prepare` card that has it as a face).
    pub fn lookup(&self, name: &str) -> &[u32] {
        let n = name.trim().to_lowercase();
        match self.names.get(&n) {
            Some(v) => v,
            None => self.faces.get(&n).map(Vec::as_slice).unwrap_or(&[]),
        }
    }
}

/// where `oracle.json` lives: `--data-dir`, else `CARDGEN_KNOWLEDGE_DIR`, else `<project>/data/knowledge`, and when
/// that has no `oracle.json` the main checkout's (`<git-common-dir>/../data/knowledge`), so worktrees work
pub fn knowledge_dir(project: &Project, data_dir: Option<&Path>) -> Result<PathBuf, String> {
    let explicit = match data_dir {
        Some(d) => Some(d.to_path_buf()),
        None => std::env::var_os("CARDGEN_KNOWLEDGE_DIR").filter(|d| !d.is_empty()).map(PathBuf::from),
    };
    let missing = |dir: &Path| {
        format!("no card catalog at {}: run `cardgen bulk-sync` first (it writes oracle.json there)", dir.display())
    };
    if let Some(d) = explicit {
        return if d.join("oracle.json").is_file() { Ok(d) } else { Err(missing(&d)) };
    }
    let own = project.data("knowledge");
    if own.join("oracle.json").is_file() {
        return Ok(own);
    }
    if let Some(main) = main_checkout(&project.root) {
        let d = main.join("data").join("knowledge");
        if d.join("oracle.json").is_file() {
            return Ok(d);
        }
    }
    Err(missing(&own))
}

/// the main checkout of the repository `root` belongs to (the parent of the git common dir), when it can be found
fn main_checkout(root: &Path) -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-common-dir"])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let common = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let common = if common.is_absolute() { common } else { root.join(common) };
    let common = common.canonicalize().ok()?;
    common.parent().map(Path::to_path_buf)
}

// ------------------------------------------------------------------------------------------------ the query

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmp {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}

impl Cmp {
    fn test(self, a: f32, b: f32) -> bool {
        match self {
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Eq => a == b,
            Cmp::Ge => a >= b,
            Cmp::Gt => a > b,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sort {
    Edhrec,
    Name,
    Cmc,
    Usd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegalFilter {
    Only(Legal),
    Any,
}

/// The parsed `find` arguments.
#[derive(Clone, Debug)]
pub struct Query {
    pub name: Option<String>,
    pub name_contains: Option<String>,
    pub ident: Option<u8>,
    pub legal: LegalFilter,
    pub cmc: Option<(Cmp, f32)>,
    pub types: Vec<String>,
    pub texts: Vec<String>,
    pub gc: bool,
    pub deck: Option<String>,
    pub sort: Sort,
    pub limit: Option<usize>,
    pub offset: usize,
    pub count: bool,
    pub json: bool,
    pub full: bool,
    pub meld: bool,
    pub data_dir: Option<PathBuf>,
}

pub const DEFAULT_LIMIT: usize = 30;

fn parse_ident(s: &str) -> Result<u8, String> {
    let mut bits = 0;
    for ch in s.chars() {
        match ch.to_ascii_uppercase() {
            'C' => {}
            c => match COLORS.iter().position(|k| *k == c) {
                Some(i) => bits |= 1 << i,
                None => {
                    return Err(format!(
                        "bad colour identity '{s}': use letters from WUBRG, or C or empty for colourless"
                    ));
                }
            },
        }
    }
    Ok(bits)
}

fn parse_cmp(s: &str) -> Option<(Cmp, f32)> {
    let s = s.trim();
    let (op, rest) = [("<=", Cmp::Le), (">=", Cmp::Ge), ("<", Cmp::Lt), (">", Cmp::Gt), ("=", Cmp::Eq)]
        .iter()
        .find_map(|(p, op)| s.strip_prefix(p).map(|r| (*op, r)))
        .unwrap_or((Cmp::Eq, s));
    let n: f32 = rest.trim().parse().ok()?;
    n.is_finite().then_some((op, n))
}

impl Query {
    pub fn parse(args: &[String]) -> Result<Query, FindError> {
        let mut q = Query {
            name: None,
            name_contains: None,
            ident: None,
            legal: LegalFilter::Only(Legal::Legal),
            cmc: None,
            types: vec![],
            texts: vec![],
            gc: false,
            deck: None,
            sort: Sort::Edhrec,
            limit: None,
            offset: 0,
            count: false,
            json: false,
            full: false,
            meld: false,
            data_dir: None,
        };
        let mut seen: Vec<&str> = vec![];
        let mut i = 0;
        while i < args.len() {
            let a = args[i].as_str();
            i += 1;
            match a {
                "--gc" => q.gc = true,
                "--count" => q.count = true,
                "--json" => q.json = true,
                "--full" => q.full = true,
                "--meld" => q.meld = true,
                "--name" | "--name-contains" | "--ident" | "--legal" | "--cmc" | "--type" | "--text" | "--deck"
                | "--sort" | "--limit" | "--offset" | "--data-dir" => {
                    let Some(v) = args.get(i) else { return usage(format!("{a} needs a value")) };
                    i += 1;
                    let repeatable = a == "--type" || a == "--text";
                    if !repeatable && seen.contains(&a) {
                        return usage(format!("{a} given twice"));
                    }
                    seen.push(a);
                    match a {
                        "--name" => q.name = Some(v.clone()),
                        "--name-contains" => q.name_contains = Some(v.to_lowercase()),
                        "--ident" => q.ident = Some(parse_ident(v).or_else(usage)?),
                        "--legal" => {
                            q.legal = match v.as_str() {
                                "any" => LegalFilter::Any,
                                "legal" => LegalFilter::Only(Legal::Legal),
                                "banned" => LegalFilter::Only(Legal::Banned),
                                "not_legal" => LegalFilter::Only(Legal::NotLegal),
                                "restricted" => LegalFilter::Only(Legal::Restricted),
                                _ => return usage(format!("--legal takes legal, banned, not_legal or any, got '{v}'")),
                            }
                        }
                        "--cmc" => match parse_cmp(v) {
                            Some(c) => q.cmc = Some(c),
                            None => {
                                return usage(format!(
                                    "--cmc takes a number with an optional <, <=, =, >= or >, got '{v}'"
                                ));
                            }
                        },
                        "--type" => q.types.push(v.trim().to_lowercase()),
                        "--text" => q.texts.push(v.clone()),
                        "--deck" => q.deck = Some(v.clone()),
                        "--sort" => {
                            q.sort = match v.as_str() {
                                "edhrec" => Sort::Edhrec,
                                "name" => Sort::Name,
                                "cmc" => Sort::Cmc,
                                "usd" => Sort::Usd,
                                _ => return usage(format!("--sort takes edhrec, name, cmc or usd, got '{v}'")),
                            }
                        }
                        "--limit" => match v.parse() {
                            Ok(n) => q.limit = Some(n),
                            Err(_) => return usage(format!("--limit needs a number, got '{v}'")),
                        },
                        "--offset" => match v.parse() {
                            Ok(n) => q.offset = n,
                            Err(_) => return usage(format!("--offset needs a number, got '{v}'")),
                        },
                        _ => q.data_dir = Some(PathBuf::from(v)),
                    }
                }
                _ => return usage(format!("unknown argument '{a}'")),
            }
        }
        Ok(q)
    }
}

#[derive(Deserialize)]
struct DecksFile {
    #[serde(default)]
    mine: Vec<Deck>,
    #[serde(default)]
    pool: Vec<Deck>,
}

#[derive(Deserialize)]
struct Deck {
    key: String,
    #[serde(default)]
    ident: String,
    #[serde(default)]
    commander: String,
    #[serde(default)]
    commander2: Option<String>,
    #[serde(default)]
    cards: Vec<String>,
}

/// a deck's key in `data/decks.json` (`mine`, then `pool`)
fn load_deck(project: &Project, key: &str) -> Result<Deck, FindError> {
    let path = project.data("decks.json");
    let text = std::fs::read_to_string(&path).map_err(|e| FindError::Failed(format!("{}: {e}", path.display())))?;
    let file: DecksFile =
        serde_json::from_str(&text).map_err(|e| FindError::Failed(format!("{}: {e}", path.display())))?;
    let want = key.to_lowercase();
    let keys = |d: &[Deck]| d.iter().map(|d| d.key.clone()).collect::<Vec<_>>().join(", ");
    let (mine_keys, pool_keys) = (keys(&file.mine), keys(&file.pool));
    match file.mine.into_iter().chain(file.pool).find(|d| d.key.to_lowercase() == want) {
        Some(d) => Ok(d),
        None => failed(format!("unknown deck '{key}'; valid keys: {mine_keys} (yours); {pool_keys} (pool)")),
    }
}

impl Index {
    fn legal_ok(c: &Card, q: &Query) -> bool {
        match q.legal {
            LegalFilter::Any => true,
            LegalFilter::Only(l) => c.legal == l,
        }
    }

    /// Indices of the matching cards, in the order of the query's sort. `ident` is the identity limit, `exclude` marks
    /// the cards to leave out (a deck's own), `regexes` are the compiled `--text` patterns.
    pub fn matches(
        &self,
        q: &Query,
        ident: Option<u8>,
        exclude: &[bool],
        regexes: &[Regex],
    ) -> Result<Vec<u32>, FindError> {
        let pool: Vec<u32> = match &q.name {
            Some(n) => self.lookup(n).to_vec(),
            None => (0..self.cards.len() as u32).collect(),
        };
        let mut out = vec![];
        for i in pool {
            let c = &self.cards[i as usize];
            if (c.is_meld() && !q.meld)
                || !Self::legal_ok(c, q)
                || ident.is_some_and(|l| c.ident & !l != 0)
                || q.cmc.is_some_and(|(op, n)| !op.test(c.cmc, n))
                || (q.gc && !c.raw.game_changer)
                || exclude.get(i as usize).copied().unwrap_or(false)
                || q.name_contains.as_deref().is_some_and(|s| !c.names_lc.contains(s))
                || !q.types.iter().all(|t| t.split_whitespace().all(|w| c.has_type_word(w)))
            {
                continue;
            }
            let mut ok = true;
            for re in regexes {
                match re.is_match(&c.text) {
                    Ok(true) => {}
                    Ok(false) => {
                        ok = false;
                        break;
                    }
                    Err(e) => return failed(format!("--text {}: {e}", re.as_str())),
                }
            }
            if ok {
                out.push(i);
            }
        }
        Ok(out)
    }

    fn order(&self, a: u32, b: u32, sort: Sort, legal_first: bool) -> Ordering {
        let (ca, cb) = (&self.cards[a as usize], &self.cards[b as usize]);
        let by_name = || ca.name_lc.cmp(&cb.name_lc).then_with(|| ca.raw.name.cmp(&cb.raw.name)).then(a.cmp(&b));
        // a name with several cards: the Commander-legal one first
        let legal =
            if legal_first { (ca.legal != Legal::Legal).cmp(&(cb.legal != Legal::Legal)) } else { Ordering::Equal };
        legal
            .then_with(|| match sort {
                Sort::Name => Ordering::Equal,
                Sort::Cmc => ca.cmc.total_cmp(&cb.cmc),
                // ranked first, ascending; unranked last
                Sort::Edhrec => match (ca.raw.edhrec_rank, cb.raw.edhrec_rank) {
                    (Some(x), Some(y)) => x.cmp(&y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                },
                Sort::Usd => match (ca.usd, cb.usd) {
                    (Some(x), Some(y)) => x.total_cmp(&y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                },
            })
            .then_with(by_name)
    }
}

// ------------------------------------------------------------------------------------------------ output

#[derive(Serialize)]
struct OutFace<'a> {
    name: &'a str,
    mana_cost: &'a str,
    type_line: &'a str,
    oracle_text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    toughness: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    loyalty: Option<&'a str>,
}

#[derive(Serialize)]
struct OutCard<'a> {
    name: &'a str,
    mana_cost: &'a str,
    cmc: f32,
    type_line: &'a str,
    identity: String,
    legal: &'static str,
    gc: bool,
    edhrec_rank: Option<u32>,
    usd: Option<f32>,
    oracle_text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    faces: Option<Vec<OutFace<'a>>>,
    set: &'a str,
    released_at: &'a str,
    rarity: &'a str,
    // only with --full
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    produced_mana: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    toughness: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    loyalty: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oracle_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reserved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meld_parts: Option<&'a [String]>,
}

fn out_card(c: &Card, full: bool) -> OutCard<'_> {
    let r = &c.raw;
    let faces = r.card_faces.as_ref().filter(|f| !f.is_empty()).map(|fs| {
        fs.iter()
            .map(|f| OutFace {
                name: &f.name,
                mana_cost: f.mana_cost.as_deref().unwrap_or(""),
                type_line: &f.type_line,
                oracle_text: f.oracle_text.as_deref().unwrap_or(""),
                power: f.power.as_deref().filter(|_| full),
                toughness: f.toughness.as_deref().filter(|_| full),
                loyalty: f.loyalty.as_deref().filter(|_| full),
            })
            .collect()
    });
    OutCard {
        name: &r.name,
        mana_cost: r.mana_cost.as_deref().unwrap_or(""),
        cmc: c.cmc,
        type_line: &r.type_line,
        identity: c.identity(),
        legal: c.legal.as_str(),
        gc: r.game_changer,
        edhrec_rank: r.edhrec_rank,
        usd: c.usd,
        oracle_text: r.oracle_text.as_deref().unwrap_or(""),
        faces,
        set: &r.set,
        released_at: &r.released_at,
        rarity: &r.rarity,
        keywords: Some(&r.keywords[..]).filter(|_| full),
        produced_mana: r.produced_mana.as_deref().filter(|_| full),
        power: r.power.as_deref().filter(|_| full),
        toughness: r.toughness.as_deref().filter(|_| full),
        loyalty: r.loyalty.as_deref().filter(|_| full),
        oracle_id: Some(&r.oracle_id[..]).filter(|_| full),
        reserved: Some(r.reserved).filter(|_| full),
        meld_parts: r.meld_parts.as_deref().filter(|_| full),
    }
}

fn json_array(cards: &[&Card], full: bool) -> String {
    let rows: Vec<String> =
        cards.iter().map(|c| serde_json::to_string(&out_card(c, full)).expect("a card serialises")).collect();
    if rows.is_empty() { "[]\n".into() } else { format!("[\n{}\n]\n", rows.join(",\n")) }
}

fn dash(s: &str) -> &str {
    if s.is_empty() { "-" } else { s }
}

fn table(cards: &[&Card]) -> String {
    let rows: Vec<[String; 6]> = cards
        .iter()
        .map(|c| {
            [
                c.raw.name.clone(),
                dash(c.raw.mana_cost.as_deref().unwrap_or("")).to_string(),
                c.raw.type_line.clone(),
                c.raw.edhrec_rank.map_or("-".into(), |r| r.to_string()),
                c.raw.usd.clone().unwrap_or_else(|| "-".into()),
                c.legal.as_str().to_string(),
            ]
        })
        .collect();
    let head = ["name", "cost", "type", "edhrec", "usd", "legal"].map(String::from);
    let mut w = [0usize; 6];
    for r in std::iter::once(&head).chain(rows.iter()) {
        for (i, cell) in r.iter().enumerate() {
            w[i] = w[i].max(cell.chars().count());
        }
    }
    let mut out = String::new();
    for r in std::iter::once(&head).chain(rows.iter()) {
        let mut line = String::new();
        for (i, cell) in r.iter().enumerate() {
            let pad = w[i] - cell.chars().count();
            line.push_str(cell);
            if i + 1 < r.len() {
                line.push_str(&" ".repeat(pad + 2));
            }
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn readable(c: &Card) -> String {
    let r = &c.raw;
    let mut s = String::new();
    let _ = writeln!(s, "{}  {}  (cmc {})", r.name, dash(r.mana_cost.as_deref().unwrap_or("")), c.cmc);
    let _ = writeln!(s, "{}", r.type_line);
    let pt = match (&r.power, &r.toughness, &r.loyalty) {
        (Some(p), Some(t), _) => format!("  P/T {p}/{t}"),
        (_, _, Some(l)) => format!("  loyalty {l}"),
        _ => String::new(),
    };
    let _ = writeln!(
        s,
        "identity {}  legal {}  GC {}  EDHREC {}  USD {}{pt}",
        dash(&c.identity()),
        c.legal.as_str(),
        if r.game_changer { "yes" } else { "no" },
        r.edhrec_rank.map_or("-".into(), |n| n.to_string()),
        r.usd.as_deref().unwrap_or("-"),
    );
    if let Some(t) = r.oracle_text.as_deref().filter(|t| !t.is_empty()) {
        let _ = writeln!(s, "{t}");
    }
    for f in r.card_faces.as_deref().unwrap_or(&[]) {
        let _ = writeln!(s, "-- {}  {}  {}", f.name, dash(f.mana_cost.as_deref().unwrap_or("")), f.type_line);
        match (&f.power, &f.toughness, &f.loyalty) {
            (Some(p), Some(t), _) => {
                let _ = writeln!(s, "   P/T {p}/{t}");
            }
            (_, _, Some(l)) => {
                let _ = writeln!(s, "   loyalty {l}");
            }
            _ => {}
        }
        if let Some(t) = f.oracle_text.as_deref().filter(|t| !t.is_empty()) {
            let _ = writeln!(s, "{t}");
        }
    }
    if !r.keywords.is_empty() {
        let _ = writeln!(s, "keywords: {}", r.keywords.join(", "));
    }
    if let Some(m) = &r.produced_mana {
        let _ = writeln!(s, "produces: {}", m.join(" "));
    }
    if let Some(m) = &r.meld_parts {
        let _ = writeln!(s, "melds from: {}", m.join(" + "));
    }
    let _ = writeln!(
        s,
        "oracle_id {}  set {} ({}, {})  reserved {}",
        r.oracle_id,
        r.set,
        r.rarity,
        r.released_at,
        if r.reserved { "yes" } else { "no" }
    );
    s
}

// ------------------------------------------------------------------------------------------------ the command

/// `cardgen find`: returns the text to print. Read-only.
pub fn find(project: &Project, args: &[String]) -> Result<String, FindError> {
    let q = Query::parse(args)?;
    let mut regexes = vec![];
    for t in &q.texts {
        // case-insensitive unless the pattern turns it off with (?-i)
        let bad = |e: fancy_regex::Error| FindError::Failed(format!("bad --text regex '{t}': {e}"));
        Regex::new(t).map_err(bad)?; // check the pattern as typed, so an error position is the user's
        regexes.push(Regex::new(&format!("(?i){t}")).map_err(bad)?);
    }
    let deck = q.deck.as_deref().map(|k| load_deck(project, k)).transpose()?;
    let ident = match (&q.ident, &deck) {
        (Some(i), _) => Some(*i),
        (None, Some(d)) => Some(parse_ident(&d.ident).map_err(|e| FindError::Failed(format!("deck {}: {e}", d.key)))?),
        (None, None) => None,
    };
    let dir = knowledge_dir(project, q.data_dir.as_deref()).map_err(FindError::Failed)?;
    let index = Index::load(&dir.join("oracle.json")).map_err(FindError::Failed)?;
    let mut exclude = vec![false; index.cards.len()];
    if let Some(d) = &deck {
        for n in d.cards.iter().chain(std::iter::once(&d.commander)).chain(d.commander2.iter()) {
            for &i in index.lookup(n) {
                exclude[i as usize] = true;
            }
        }
    }
    let mut hits = index.matches(&q, ident, &exclude, &regexes)?;
    if q.count {
        return Ok(format!("{}\n", hits.len()));
    }
    let name_query = q.name.is_some();
    hits.sort_by(|a, b| index.order(*a, *b, q.sort, name_query));
    let total = hits.len();
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT);
    let shown: Vec<&Card> = hits.iter().skip(q.offset).take(limit).map(|i| &index.cards[*i as usize]).collect();
    if q.json {
        return Ok(json_array(&shown, q.full));
    }
    let mut out = String::new();
    if shown.is_empty() {
        out.push_str(if total == 0 { "no matches\n" } else { "no matches in this page\n" });
        if total == 0 && !q.meld {
            let hidden = index.matches(&Query { meld: true, ..q.clone() }, ident, &exclude, &regexes)?;
            if !hidden.is_empty() {
                let _ = writeln!(out, "({} meld result(s) are hidden; add --meld)", hidden.len());
            }
        }
        return Ok(out);
    }
    if q.full {
        out = shown.iter().map(|c| readable(c)).collect::<Vec<_>>().join("\n");
    } else {
        out = table(&shown);
    }
    let end = q.offset + shown.len();
    if end < total {
        let _ = writeln!(out, "(showing {}-{} of {total}; --offset {end} for the next page)", q.offset + 1, end);
    }
    Ok(out)
}
