//! The in-memory card index over `data/knowledge/oracle.json` and `cardgen find`.
//!
//! `oracle.json` is loaded with a typed serde struct (not the ordered `J` tree) into an [`Index`]: per card the colour
//! identity bits, cmc, type flags, the lower-case type line and names, the Oracle text of every face, the commander
//! legality, the Game Changer flag, the EDHREC rank and the price. Every query is a linear scan. Nothing is persisted.
//! The command is read-only. See documents/card-knowledge.md.

use crate::data::Project;
use crate::facts::{Facts, status_selected};
use crate::roles::Source;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

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

/// The subtypes that are two words (the catalog has one: `Time Lord`, on 29 legal cards).
pub const MULTI_WORD_SUBTYPES: [&str; 1] = ["Time Lord"];

const COLORS: [char; 5] = ['W', 'U', 'B', 'R', 'G'];

fn is_combining(c: char) -> bool {
    ('\u{300}'..='\u{36F}').contains(&c)
}

/// The search key of a name, not trimmed (what `--name-contains` uses, so `" of "` stays `" of "`): lower-case, accents
/// and other combining marks dropped (NFKD, lower-cased again after it), every apostrophe variant a plain `'`, U+A789
/// (the modifier colon in Ratonhnhaké꞉ton) a `:`, `Æ`/`Œ`/`ß` spelled `ae`/`oe`/`ss`. Hyphens, commas and spaces stay.
pub fn fold_raw(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.to_lowercase().chars() {
        match c {
            '\u{2019}' | '\u{2018}' | '\u{2bc}' | '`' | '\u{b4}' => out.push('\''),
            '\u{a789}' => out.push(':'),
            'æ' => out.push_str("ae"),
            'œ' => out.push_str("oe"),
            'ß' => out.push_str("ss"),
            c => {
                for d in std::iter::once(c).nfkd().filter(|d| !is_combining(*d)) {
                    out.extend(d.to_lowercase());
                }
            }
        }
    }
    out
}

/// [`fold_raw`] without the lower-casing (and without its backtick rule, so markdown code spans survive): accents and
/// combining marks dropped, every other apostrophe variant a plain `'`,
/// `Æ`/`Œ`/`ß` spelled out (`Ae`, `Oe`, `ss`), the case kept. The spelling a case-sensitive text search compares by
/// (`Anduril` in a note finds `Andúril`; `Sol Ring` never finds `sol ring`).
pub fn fold_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\u{2019}' | '\u{2018}' | '\u{2bc}' | '\u{b4}' => out.push('\''),
            '\u{a789}' => out.push(':'),
            'Æ' => out.push_str("Ae"),
            'æ' => out.push_str("ae"),
            'Œ' => out.push_str("Oe"),
            'œ' => out.push_str("oe"),
            'ß' => out.push_str("ss"),
            c => out.extend(std::iter::once(c).nfkd().filter(|d| !is_combining(*d))),
        }
    }
    out
}

/// The search key of a name: [`fold_raw`] with the ends trimmed. `Andúril` and `Anduril`, `Urza’s Saga` and
/// `Urza's Saga`, `Æther Vial` and `Aether Vial` have the same key.
pub fn fold(s: &str) -> String {
    fold_raw(s).trim().to_string()
}

/// Terminal cells a string takes: combining marks 0, East Asian wide and emoji ranges 2, everything else 1.
pub fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| match c as u32 {
            0x300..=0x36F => 0,
            0x1100..=0x115F
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1F64F
            | 0x20000..=0x3FFFD => 2,
            _ => 1,
        })
        .sum()
}

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
    /// the folded full name (see [`fold`]): the name lookups and the sort key
    pub key: String,
    /// the folded full name and every folded face name, one per line (what `--name-contains` searches)
    pub keys: String,
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
        let key = fold(&raw.name);
        let mut keys = key.clone();
        for f in faces {
            keys.push('\n');
            keys.push_str(&fold(&f.name));
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
        let text = Card::parts(&raw).iter().map(|p| p.1).collect::<Vec<_>>().join("\n//\n");
        Card {
            ident,
            cmc: raw.cmc as f32,
            types,
            legal: Legal::parse(raw.legalities.commander.as_deref()),
            usd: raw.usd.as_deref().and_then(|u| u.parse::<f32>().ok()),
            key,
            keys,
            type_lc,
            text,
            raw,
        }
    }

    /// the Oracle text of the card (if it has its own) and of each face, empty and repeated texts left out, with the
    /// face index (`None` for the card's own text)
    fn parts(raw: &Raw) -> Vec<(Option<usize>, &str)> {
        let faces = raw.card_faces.as_deref().unwrap_or(&[]);
        let mut parts: Vec<(Option<usize>, &str)> = vec![];
        let texts = std::iter::once((None, raw.oracle_text.as_deref()))
            .chain(faces.iter().enumerate().map(|(i, f)| (Some(i), f.oracle_text.as_deref())));
        for (i, t) in texts {
            match t {
                Some(t) if !t.is_empty() && !parts.iter().any(|p| p.1 == t) => parts.push((i, t)),
                _ => {}
            }
        }
        parts
    }

    /// The mana cost: the card's own, else the non-empty face costs joined with ` // ` (a transform or modal card has
    /// none of its own).
    pub fn cost(&self) -> String {
        match self.raw.mana_cost.as_deref() {
            Some(c) if !c.is_empty() => c.to_string(),
            _ => {
                let faces = self.raw.card_faces.as_deref().unwrap_or(&[]);
                let costs: Vec<&str> =
                    faces.iter().filter_map(|f| f.mana_cost.as_deref()).filter(|c| !c.is_empty()).collect();
                costs.join(" // ")
            }
        }
    }

    /// The Oracle lines with the face each belongs to (`None`: the card's own text): the numbering [`Card::text`] has
    /// without its `//` joiner lines, so an evidence index and a digest gist cite the same line.
    pub fn lines(&self) -> Vec<(Option<usize>, &str)> {
        Card::parts(&self.raw)
            .into_iter()
            .flat_map(|(i, t)| t.split('\n').filter(|l| !l.is_empty()).map(move |l| (i, l)))
            .collect()
    }

    /// The subtypes: the words after the `—` of the type line of each face, in order, each once. A subtype of two
    /// words (`Time Lord`, see [`MULTI_WORD_SUBTYPES`]) is one subtype.
    pub fn subtypes(&self) -> Vec<&str> {
        let mut out: Vec<&str> = vec![];
        for line in self.raw.type_line.split(" // ") {
            let Some((_, subs)) = line.split_once('—') else { continue };
            let words: Vec<&str> = subs.split_whitespace().collect();
            let mut i = 0;
            while i < words.len() {
                let multi = MULTI_WORD_SUBTYPES.iter().find(|m| {
                    let parts: Vec<&str> = m.split(' ').collect();
                    words[i..].starts_with(&parts)
                });
                let (sub, n) = match multi {
                    Some(m) => (*m, m.split(' ').count()),
                    None => (words[i], 1),
                };
                if !out.contains(&sub) {
                    out.push(sub);
                }
                i += n;
            }
        }
        out
    }

    /// What the card's Oracle text may call itself, for a `~` substitution: the full name, each face name and the short
    /// name before a comma (`Andúril` for `Andúril, Narsil Reforged`), as printed (accents kept), each once.
    pub fn self_names(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![self.raw.name.clone()];
        names.extend(self.raw.card_faces.iter().flatten().map(|f| f.name.clone()));
        let shorts: Vec<String> =
            names.iter().filter_map(|n| n.split_once(',').map(|(a, _)| a.trim().to_string())).collect();
        names.extend(shorts);
        let mut out: Vec<String> = vec![];
        for n in names {
            if !n.is_empty() && !out.contains(&n) {
                out.push(n);
            }
        }
        out
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

pub fn ident_string(bits: u8) -> String {
    COLORS.iter().enumerate().filter(|(i, _)| bits & (1 << i) != 0).map(|(_, c)| *c).collect()
}

/// The whole catalog in memory, with name lookups.
pub struct Index {
    pub cards: Vec<Card>,
    pub bulk_updated_at: String,
    pub oracle_fnv: String,
    pub sync_fnv: Option<String>,
    pub taxonomy_fnv: String,
    pub rules_fnv: String,
    /// the text of roles.toml the guards of LLM evidence are checked against (see `attach_roles`)
    taxonomy_src: String,
    /// lower-case full name -> every card with it (15 names are shared by two or more cards)
    names: HashMap<String, Vec<u32>>,
    /// lower-case face name (when different from the full name) -> every card with such a face
    faces: HashMap<String, Vec<u32>>,
    ids: HashMap<String, u32>,
    roles: Option<Vec<crate::roles::Assignment>>,
}

impl Index {
    /// `knowledge_dir` plus `load`: the catalog `find` would read (see [`knowledge_dir`])
    pub fn open(project: &Project, data_dir: Option<&Path>) -> Result<Index, String> {
        Index::load(&knowledge_dir(project, data_dir)?.join("oracle.json"))
    }

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
        let mut ids: HashMap<&str, usize> = HashMap::new();
        for (n, c) in oracle.cards.iter().enumerate() {
            let bad = |what: String| {
                format!("{}: record {} {what}; run `cardgen bulk-sync` to rebuild it", path.display(), n + 1)
            };
            if c.name.trim().is_empty() {
                return Err(bad(format!("has an empty name (oracle_id '{}')", c.oracle_id)));
            }
            if c.oracle_id.is_empty() {
                return Err(bad(format!("('{}') has no oracle_id", c.name)));
            }
            if let Some(first) = ids.insert(&c.oracle_id, n + 1) {
                return Err(bad(format!("('{}') repeats oracle_id {} of record {first}", c.name, c.oracle_id)));
            }
        }
        drop(ids);
        let sync_fnv = crate::bulk::read_lock(&path.with_file_name("sync.lock")).map(|lock| lock.fnv1a64);
        let oracle_fnv = match &sync_fnv {
            Some(hash) => hash.clone(),
            None => crate::roles::oracle_fingerprint(path)?,
        };
        let taxonomy_bytes = std::fs::read(path.with_file_name("roles.toml"))
            .unwrap_or_else(|_| include_bytes!("../../../../data/knowledge/roles.toml").to_vec());
        let mut idx = Index {
            cards: oracle.cards.into_iter().map(Card::new).collect(),
            bulk_updated_at: oracle.meta.bulk_updated_at,
            oracle_fnv,
            sync_fnv,
            // the same file `roles --rules` reads (one resolver: `roles::overrides_path`)
            rules_fnv: crate::roles::rules_hash(&std::fs::read(crate::roles::overrides_path(path)).unwrap_or_default()),
            taxonomy_fnv: crate::roles::fingerprint(&taxonomy_bytes),
            taxonomy_src: String::from_utf8_lossy(&taxonomy_bytes).into_owned(),
            names: HashMap::new(),
            faces: HashMap::new(),
            ids: HashMap::new(),
            roles: None,
        };
        for (i, c) in idx.cards.iter().enumerate() {
            idx.ids.insert(c.raw.oracle_id.clone(), i as u32);
            idx.names.entry(c.key.clone()).or_default().push(i as u32);
            let mut seen: Vec<String> = vec![];
            for f in c.raw.card_faces.as_deref().unwrap_or(&[]) {
                let n = fold(&f.name);
                if n != c.key && !seen.contains(&n) {
                    idx.faces.entry(n.clone()).or_default().push(i as u32);
                    seen.push(n);
                }
            }
        }
        Ok(idx)
    }

    /// Attach a complete deck-card side table, aligned with card indices. Card and oracle.json stay immutable.
    /// Reject stale/malformed data rather than silently returning incomplete role searches.
    pub fn attach_roles(&mut self, map: &crate::roles::RoleMap) -> Result<(), String> {
        map.check_meta(self)?;
        if map.len() != self.cards.iter().filter(|c| crate::roles::is_deck_card(c)).count() {
            return Err("roles.json: catalog count differs; run `cardgen roles --rules`".into());
        }
        let mut rows = Vec::with_capacity(self.cards.len());
        let mut taxonomy: Option<crate::roles::Taxonomy> = None;
        // A merged table was built under one LLM prompt; another prompt (or a table that claims LLM evidence without
        // naming one) means its LLM roles are not the ones this build would accept.
        let claims_llm = map.values().any(|a| a.evidence.values().any(|e| e.source == Source::Llm));
        if !map.meta.prompt_fnv.is_empty() || claims_llm {
            let tax = crate::roles::Taxonomy::parse(&self.taxonomy_src)?;
            // either prompt variant (CARDGEN_LLM_VARIANT) may have built the table
            let known = [crate::llm::Variant::Plain, crate::llm::Variant::Think];
            let built_here = known.iter().any(|v| {
                [false, true].iter().any(|nt| map.meta.prompt_fnv == crate::llm::prompt_fnv_opts(&tax, *v, *nt))
            });
            if !built_here {
                return Err("roles.json: built under another LLM prompt; run `cardgen roles --merge`".into());
            }
            taxonomy = Some(tax);
        }
        for c in &self.cards {
            if !crate::roles::is_deck_card(c) {
                if map.contains_key(&c.raw.oracle_id) {
                    return Err("roles.json: non-deck card has roles; run `cardgen roles --rules`".into());
                }
                rows.push(crate::roles::Assignment::default());
                continue;
            }
            let bad =
                || format!("roles.json: invalid or missing roles for {}; run `cardgen roles --rules`", c.raw.name);
            let a = map.get(&c.raw.oracle_id).ok_or_else(bad)?;
            if a.roles.len() > 4
                || a.roles.first().map_or("", String::as_str) != a.primary
                || a.roles.iter().collect::<std::collections::BTreeSet<_>>().len() != a.roles.len()
                || a.evidence.len() != a.roles.len()
            {
                return Err(bad());
            }
            let lines = c.lines();
            for role in &a.roles {
                crate::roles::check_role(role)?;
                let e = a.evidence.get(role).ok_or_else(bad)?;
                match e.source {
                    Source::Rule => {
                        let (face, _) = e.line.and_then(|i| lines.get(i)).ok_or_else(bad)?;
                        if *face != e.face || !crate::roles::RULES.iter().any(|r| r.id == e.rule && r.role == role) {
                            return Err(bad());
                        }
                    }
                    Source::Override => {
                        if e.certain || e.line.is_some() || e.face.is_some() {
                            return Err(bad());
                        }
                    }
                    // LLM evidence must cite a real, readable line whose text passes the role's guard.
                    Source::Llm => {
                        let line = e.line.ok_or_else(bad)?;
                        let (face, _) = lines.get(line).ok_or_else(bad)?;
                        if *face != e.face || e.certain || e.rule != "llm" || crate::roles::skipped(c, line) {
                            return Err(bad());
                        }
                        if taxonomy.is_none() {
                            taxonomy = Some(crate::roles::Taxonomy::parse(&self.taxonomy_src)?);
                        }
                        let guard = taxonomy.as_ref().expect("parsed above").guard(role);
                        if !guard.is_some_and(|g| g.is_match(&crate::roles::line_text(c, line)).unwrap_or(false)) {
                            return Err(bad());
                        }
                    }
                }
            }
            rows.push(a.clone());
        }
        self.roles = Some(rows);
        Ok(())
    }

    /// the text of the taxonomy (roles.toml beside the catalog, else the built-in copy): what the guards come from
    pub fn taxonomy_text(&self) -> &str {
        &self.taxonomy_src
    }

    /// Attached roles for a card index; None means no roles file was attached.
    pub fn roles(&self, i: u32) -> Option<&crate::roles::Assignment> {
        self.roles.as_ref()?.get(i as usize)
    }

    /// the index of the card with this `oracle_id`
    pub fn by_oracle_id(&self, id: &str) -> Option<u32> {
        self.ids.get(id).copied()
    }

    /// Every subtype word (see [`Card::subtypes`]) with the number of cards that have it, sorted by word.
    pub fn subtype_vocab(&self) -> Vec<(String, usize)> {
        let mut m: std::collections::BTreeMap<String, usize> = Default::default();
        for c in &self.cards {
            for w in c.subtypes() {
                *m.entry(w.to_string()).or_default() += 1;
            }
        }
        m.into_iter().collect()
    }

    /// Every name key shared by two or more cards, with the cards (for reports and tests).
    pub fn shared_names(&self) -> Vec<(&str, &[u32])> {
        let mut v: Vec<(&str, &[u32])> =
            self.names.iter().filter(|(_, c)| c.len() > 1).map(|(k, c)| (k.as_str(), c.as_slice())).collect();
        v.sort();
        v
    }

    /// The cards a name stands for (compared by [`fold`]: case, accents and apostrophe style do not matter): every
    /// card with that full name; only when there is none, the cards that have a face of that name. An exact full
    /// name therefore beats a face name (`Lightning Bolt` is the instant, not the `prepare` card that has it as a
    /// face).
    pub fn lookup(&self, name: &str) -> &[u32] {
        let n = fold(name);
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
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
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
    pub roles: Vec<String>,
    pub gc: bool,
    pub deck: Option<String>,
    /// `--status LIST`: the engine model statuses to keep, lower-case, `none` = no engine record
    pub status: Option<Vec<String>>,
    /// `--in-deck KEY` / `--not-in-deck KEY`: a deck of `mine` that plays the card / does not
    pub in_deck: Option<String>,
    pub not_in_deck: Option<String>,
    pub sort: Sort,
    pub limit: usize,
    pub offset: usize,
    pub count: bool,
    pub json: bool,
    pub full: bool,
    pub meld: bool,
    pub data_dir: Option<PathBuf>,
}

pub const DEFAULT_LIMIT: usize = 30;

pub fn parse_ident(s: &str) -> Result<u8, String> {
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

impl Default for Query {
    /// every Commander-legal card, EDHREC order, no limit (what code that builds a query without argv starts from)
    fn default() -> Query {
        Query {
            name: None,
            name_contains: None,
            ident: None,
            legal: LegalFilter::Only(Legal::Legal),
            cmc: None,
            types: vec![],
            texts: vec![],
            roles: vec![],
            gc: false,
            deck: None,
            status: None,
            in_deck: None,
            not_in_deck: None,
            sort: Sort::Edhrec,
            limit: usize::MAX,
            offset: 0,
            count: false,
            json: false,
            full: false,
            meld: false,
            data_dir: None,
        }
    }
}

impl Query {
    pub fn parse(args: &[String]) -> Result<Query, FindError> {
        let mut q = Query { limit: DEFAULT_LIMIT, ..Query::default() };
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
                | "--sort" | "--limit" | "--offset" | "--data-dir" | "--role" | "--status" | "--in-deck"
                | "--not-in-deck" => {
                    let Some(v) = args.get(i) else { return usage(format!("{a} needs a value")) };
                    if v.starts_with("--") {
                        return usage(format!("{a} needs a value (got '{v}')"));
                    }
                    i += 1;
                    let repeatable = a == "--type" || a == "--text" || a == "--role";
                    if !repeatable && seen.contains(&a) {
                        return usage(format!("{a} given twice"));
                    }
                    seen.push(a);
                    match a {
                        "--name" => q.name = Some(v.clone()),
                        "--name-contains" => {
                            if v.is_empty() {
                                return usage("--name-contains needs some text, got ''");
                            }
                            q.name_contains = Some(fold_raw(v))
                        }
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
                        "--type" => {
                            if words(v).next().is_none() {
                                return usage(format!("--type needs a word of a type line, got '{v}'"));
                            }
                            q.types.push(v.trim().to_lowercase())
                        }
                        "--text" => q.texts.push(v.clone()),
                        "--role" => {
                            crate::roles::check_role(v).map_err(FindError::Usage)?;
                            q.roles.push(v.clone());
                        }
                        "--deck" => q.deck = Some(v.clone()),
                        "--status" => q.status = Some(crate::facts::parse_status_list(v).map_err(FindError::Usage)?),
                        "--in-deck" => q.in_deck = Some(v.clone()),
                        "--not-in-deck" => q.not_in_deck = Some(v.clone()),
                        "--sort" => {
                            q.sort = match v.as_str() {
                                "edhrec" => Sort::Edhrec,
                                "name" => Sort::Name,
                                "cmc" => Sort::Cmc,
                                "usd" => Sort::Usd,
                                _ => return usage(format!("--sort takes edhrec, name, cmc or usd, got '{v}'")),
                            }
                        }
                        "--limit" if v == "all" => q.limit = usize::MAX,
                        "--limit" => match v.parse() {
                            Ok(0) => return usage("--limit 0 shows nothing; use --count for the number of matches"),
                            Ok(n) => q.limit = n,
                            Err(_) => return usage(format!("--limit needs a number or 'all', got '{v}'")),
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

/// A deck of `data/decks.json`.
#[derive(Deserialize, Clone, Debug)]
pub struct Deck {
    pub key: String,
    #[serde(default)]
    pub ident: String,
    #[serde(default)]
    pub commander: String,
    #[serde(default)]
    pub commander2: Option<String>,
    #[serde(default)]
    pub cards: Vec<String>,
}

/// the colour identity limit of a deck (its `ident`)
pub fn deck_ident(d: &Deck) -> Result<u8, String> {
    parse_ident(&d.ident).map_err(|e| format!("deck {}: {e}", d.key))
}

/// a deck's key in `data/decks.json` (`mine`, then `pool`)
pub fn load_deck(project: &Project, key: &str) -> Result<Deck, FindError> {
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
    /// The cards a deck already plays (its list and its commanders, resolved by [`Index::lookup`]: name or face name,
    /// either spelling) as a mark per card index, and the names that resolve to no card.
    pub fn deck_members(&self, d: &Deck) -> (Vec<bool>, Vec<String>) {
        let mut marks = vec![false; self.cards.len()];
        let mut unresolved: Vec<String> = vec![];
        let names = d.cards.iter().chain(std::iter::once(&d.commander)).chain(d.commander2.iter());
        for n in names.filter(|n| !n.trim().is_empty()) {
            let found = self.lookup(n);
            if found.is_empty() && !unresolved.contains(n) {
                unresolved.push(n.clone());
            }
            for &i in found {
                marks[i as usize] = true;
            }
        }
        (marks, unresolved)
    }

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
        for role in &q.roles {
            crate::roles::check_role(role).map_err(FindError::Usage)?;
        }
        if !q.roles.is_empty() && self.roles.is_none() {
            return usage("--role requires roles.json; run `cardgen roles --rules` first");
        }
        let pool: Vec<u32> = match &q.name {
            Some(n) => self.lookup(n).to_vec(),
            None => (0..self.cards.len() as u32).collect(),
        };
        let mut out = vec![];
        for i in pool {
            let c = &self.cards[i as usize];
            if (c.is_meld() && !q.meld && q.name.is_none())
                || !Self::legal_ok(c, q)
                || ident.is_some_and(|l| c.ident & !l != 0)
                || q.cmc.is_some_and(|(op, n)| !op.test(c.cmc, n))
                || (q.gc && !c.raw.game_changer)
                || exclude.get(i as usize).copied().unwrap_or(false)
                || q.name_contains.as_deref().is_some_and(|s| !c.keys.contains(s))
                || !q.types.iter().all(|t| t.split_whitespace().all(|w| c.has_type_word(w)))
                || !q.roles.iter().all(|r| self.roles(i).is_some_and(|a| a.roles.contains(r)))
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
                    Err(e) => {
                        let shown = re.as_str().strip_prefix("(?i)").unwrap_or(re.as_str());
                        return failed(format!("--text {shown}: {e}"));
                    }
                }
            }
            if ok {
                out.push(i);
            }
        }
        Ok(out)
    }

    /// Sorts card indices as `find` does: by the key (EDHREC rank with unranked last, name, cmc or price), ties by
    /// folded name, then name, then index.
    pub fn sort(&self, ids: &mut [u32], sort: Sort) {
        ids.sort_by(|a, b| self.order(*a, *b, sort, false));
    }

    fn order(&self, a: u32, b: u32, sort: Sort, legal_first: bool) -> Ordering {
        let (ca, cb) = (&self.cards[a as usize], &self.cards[b as usize]);
        let by_name = || ca.key.cmp(&cb.key).then_with(|| ca.raw.name.cmp(&cb.raw.name)).then(a.cmp(&b));
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

/// One face in a JSON row.
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

/// The one JSON row shape of `find --json`. Keys are only ever added, never renamed or removed.
#[derive(Serialize)]
struct OutCard<'a> {
    name: &'a str,
    oracle_id: &'a str,
    layout: &'a str,
    mana_cost: String,
    cmc: f32,
    type_line: &'a str,
    identity: String,
    legal: &'static str,
    gc: bool,
    edhrec_rank: Option<u32>,
    usd: Option<f32>,
    /// `Card::text`: the card's and its faces' text joined by `\n//\n`, what `--text` matches
    oracle_text: &'a str,
    keywords: &'a [String],
    set: &'a str,
    released_at: &'a str,
    rarity: &'a str,
    reserved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    faces: Option<Vec<OutFace<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    toughness: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    loyalty: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    produced_mana: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meld_parts: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary: Option<&'a str>,
    /// the keys of the decks of `mine` that play the card (always present, possibly empty)
    decks: Vec<&'a str>,
    /// the lines of the decks' `FINDINGS.md` files that mention the card (always present, possibly empty)
    findings: Vec<OutFinding<'a>>,
    /// the engine's model status (`cardgen audit`'s words): only when the card has an engine record
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_note: Option<&'a str>,
}

/// One `FINDINGS.md` line in a JSON row.
#[derive(Serialize)]
struct OutFinding<'a> {
    file: &'a str,
    line: usize,
    heading: &'a str,
    text: &'a str,
}

fn out_card(c: &Card) -> OutCard<'_> {
    let r = &c.raw;
    let faces = r.card_faces.as_ref().filter(|f| !f.is_empty()).map(|fs| {
        fs.iter()
            .map(|f| OutFace {
                name: &f.name,
                mana_cost: f.mana_cost.as_deref().unwrap_or(""),
                type_line: &f.type_line,
                oracle_text: f.oracle_text.as_deref().unwrap_or(""),
                power: f.power.as_deref(),
                toughness: f.toughness.as_deref(),
                loyalty: f.loyalty.as_deref(),
            })
            .collect()
    });
    OutCard {
        name: &r.name,
        oracle_id: &r.oracle_id,
        layout: &r.layout,
        mana_cost: c.cost(),
        cmc: c.cmc,
        type_line: &r.type_line,
        identity: c.identity(),
        legal: c.legal.as_str(),
        gc: r.game_changer,
        edhrec_rank: r.edhrec_rank,
        usd: c.usd,
        oracle_text: &c.text,
        keywords: &r.keywords,
        set: &r.set,
        released_at: &r.released_at,
        rarity: &r.rarity,
        reserved: r.reserved,
        faces,
        power: r.power.as_deref(),
        toughness: r.toughness.as_deref(),
        loyalty: r.loyalty.as_deref(),
        produced_mana: r.produced_mana.as_deref(),
        meld_parts: r.meld_parts.as_deref(),
        roles: None,
        primary: None,
        decks: vec![],
        findings: vec![],
        status: None,
        status_note: None,
    }
}

/// a bare array, one object per line
fn json_array(cards: &[&Card], index: &Index, facts: &Facts) -> String {
    let rows: Vec<String> = cards
        .iter()
        .map(|c| {
            let mut row = out_card(c);
            let id = index.by_oracle_id(&c.raw.oracle_id);
            if let Some(a) = id.and_then(|i| index.roles(i)) {
                row.roles = Some(&a.roles);
                row.primary = Some(&a.primary);
            }
            if let Some(i) = id {
                row.decks = facts.decks_of(i);
                row.findings = facts
                    .findings_of(i)
                    .iter()
                    .map(|f| OutFinding { file: &f.file, line: f.line, heading: &f.heading, text: &f.text })
                    .collect();
                if let Some((s, n)) = facts.status_of(i) {
                    row.status = Some(s);
                    row.status_note = Some(n);
                }
            }
            serde_json::to_string(&row).expect("a card serialises")
        })
        .collect();
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
                dash(&c.cost()).to_string(),
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
            w[i] = w[i].max(display_width(cell));
        }
    }
    let mut out = String::new();
    for r in std::iter::once(&head).chain(rows.iter()) {
        let mut line = String::new();
        for (i, cell) in r.iter().enumerate() {
            let pad = w[i] - display_width(cell);
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

fn readable(c: &Card, i: Option<u32>, facts: &Facts) -> String {
    let r = &c.raw;
    let mut s = String::new();
    let _ = writeln!(s, "{}  {}  (cmc {})", r.name, dash(&c.cost()), c.cmc);
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
    if let Some(i) = i {
        match facts.status_of(i) {
            Some((st, note)) => {
                let _ = writeln!(
                    s,
                    "engine status: {st}{}",
                    if note.is_empty() { String::new() } else { format!(" ({note})") }
                );
            }
            None => {
                let _ = writeln!(s, "engine status: none (no record in data/cards.json: the sim cannot evaluate it)");
            }
        }
        let decks = facts.decks_of(i);
        let _ = writeln!(s, "decks: {}", if decks.is_empty() { "none".to_string() } else { decks.join(", ") });
        let found = facts.findings_of(i);
        if found.is_empty() {
            let _ = writeln!(s, "findings: none");
        } else {
            let _ = writeln!(s, "findings ({}):", found.len());
            for f in found {
                let _ = writeln!(s, "  {}:{}  [{}]  {}", f.file, f.line, dash(&f.heading), f.text);
            }
        }
    }
    s
}

// ------------------------------------------------------------------------------------------------ the command

/// What `find` produced: the text for stdout and notes about what the result leaves out (a cut page, a named card the
/// filters removed, deck names that match no card, hidden meld results).
#[derive(Debug)]
pub struct FindOutput {
    pub out: String,
    pub notes: Vec<String>,
    /// JSON and `--count` output is for a program: its notes go to stderr; a table gets them as trailing lines
    pub notes_on_stderr: bool,
    /// Derived-role availability is always a stderr diagnostic, independent of display mode.
    pub role_note: Option<String>,
}

impl FindOutput {
    /// the text for stdout: the output, then the notes when they are not for stderr
    pub fn stdout(&self) -> String {
        let mut s = self.out.clone();
        if !self.notes_on_stderr {
            for n in &self.notes {
                s.push_str(n);
                s.push('\n');
            }
        }
        s
    }

    /// the text for stderr: the notes when they are for stderr
    pub fn stderr(&self) -> String {
        let mut out: String =
            if self.notes_on_stderr { self.notes.iter().map(|n| format!("{n}\n")).collect() } else { String::new() };
        if let Some(note) = &self.role_note {
            out.push_str(note);
            out.push('\n');
        }
        out
    }
}

/// `cardgen find`: the text to print (notes included when they belong on stdout). Read-only.
pub fn find(project: &Project, args: &[String]) -> Result<String, FindError> {
    find_with_notes(project, args).map(|o| o.stdout())
}

/// every reason a card that `--name` found is not in the result
fn why_filtered(
    c: &Card,
    i: u32,
    q: &Query,
    ident: Option<u8>,
    exclude: &[bool],
    deck: Option<&Deck>,
    facts: Option<&Facts>,
) -> Vec<String> {
    let mut why = vec![];
    if let Some(f) = facts {
        if let Some(list) = &q.status
            && !status_selected(list, f.status_of(i).map(|s| s.0))
        {
            why.push(format!("engine status {}", f.status_of(i).map_or("none", |s| s.0).to_lowercase()));
        }
        let bits = f.deck_bits.get(i as usize).copied().unwrap_or(0);
        if let Some(k) = &q.in_deck
            && let Ok(bit) = f.deck_bit(k)
            && bits & bit == 0
        {
            why.push(format!("not in deck {k}"));
        }
        if let Some(k) = &q.not_in_deck
            && let Ok(bit) = f.deck_bit(k)
            && bits & bit != 0
        {
            why.push(format!("in deck {k} (--not-in-deck)"));
        }
    }
    if !Index::legal_ok(c, q) {
        why.push(format!("legality {}", c.legal.as_str()));
    }
    if exclude.get(i as usize).copied().unwrap_or(false) {
        why.push(format!("in deck {}", deck.map_or("", |d| &d.key)));
    }
    if let Some(l) = ident.filter(|l| c.ident & !l != 0) {
        why.push(format!("outside the identity {}", if l == 0 { "C".to_string() } else { ident_string(l) }));
    }
    if why.is_empty() {
        why.push("other filters".into());
    }
    why
}

fn compile_texts(q: &Query) -> Result<Vec<Regex>, FindError> {
    let mut regexes = vec![];
    for t in &q.texts {
        // case-insensitive unless the pattern turns it off with (?-i)
        let bad = |e: fancy_regex::Error| FindError::Failed(format!("bad --text regex '{t}': {e}"));
        Regex::new(t).map_err(bad)?; // check the pattern as typed, so an error position is the user's
        regexes.push(Regex::new(&format!("(?i){t}")).map_err(bad)?);
    }
    Ok(regexes)
}

/// The identity limit a query runs under. A deck's identity is a ceiling: `--ident` may narrow it, never widen it (a
/// usage error), and an unreadable deck identity is an error.
pub fn effective_ident(q: &Query, deck: Option<&Deck>) -> Result<Option<u8>, FindError> {
    let Some(d) = deck else { return Ok(q.ident) };
    let limit = deck_ident(d).map_err(FindError::Failed)?;
    if let Some(given) = q.ident.filter(|g| g & !limit != 0) {
        let name = |b: u8| if b == 0 { "C".to_string() } else { ident_string(b) };
        return usage(format!("--ident {} is outside deck {}'s identity {}", name(given), d.key, name(limit)));
    }
    Ok(Some(q.ident.unwrap_or(limit)))
}

impl Index {
    /// Everything `find` does before it prints, so a program (the digest) cannot re-implement the deck's identity
    /// ceiling: compiles the `--text` patterns, applies the identity limit ([`effective_ident`]), leaves out the deck's
    /// own cards ([`Index::deck_members`]), filters ([`Index::matches`]) and sorts. `deck` is the deck for the query
    /// (the caller loads it; `q.deck` is not read). Returns the card indices in order and the deck names that match no
    /// card. `q.limit` and `q.offset` are not applied.
    pub fn search(&self, q: &Query, deck: Option<&Deck>) -> Result<(Vec<u32>, Vec<String>), FindError> {
        let regexes = compile_texts(q)?;
        let ident = effective_ident(q, deck)?;
        let (exclude, unresolved) = match deck {
            Some(d) => self.deck_members(d),
            None => (vec![false; self.cards.len()], vec![]),
        };
        let mut hits = self.matches(q, ident, &exclude, &regexes)?;
        let name_query = q.name.is_some();
        hits.sort_by(|a, b| self.order(*a, *b, q.sort, name_query));
        Ok((hits, unresolved))
    }
}

/// `cardgen find` with its notes kept apart from the output.
pub fn find_with_notes(project: &Project, args: &[String]) -> Result<FindOutput, FindError> {
    let q = Query::parse(args)?;
    compile_texts(&q)?; // a bad pattern is reported before the catalog is read
    let deck = q.deck.as_deref().map(|k| load_deck(project, k)).transpose()?;
    let ident = effective_ident(&q, deck.as_ref())?;
    let dir = knowledge_dir(project, q.data_dir.as_deref()).map_err(FindError::Failed)?;
    let mut index = Index::load(&dir.join("oracle.json")).map_err(FindError::Failed)?;
    let role_file = dir.join("roles.json");
    let mut notes = vec![];
    let mut role_note = None;
    // Role filters require the table. JSON may enrich its rows when the table is current.
    // Plain text/count searches never depend on the derived role data.
    if !q.roles.is_empty() || q.json || !role_file.exists() {
        let attached = crate::roles::load(&role_file).and_then(|roles| index.attach_roles(&roles));
        if let Err(error) = attached {
            if !q.roles.is_empty() {
                return Err(if role_file.exists() {
                    FindError::Failed(error)
                } else {
                    FindError::Usage(format!("--role needs a current table: {error}"))
                });
            }
            role_note = Some(format!("(roles unavailable: {})", error.lines().next().unwrap_or("invalid table")));
        }
    } else if let Err(error) = crate::roles::check_file(&role_file, &index) {
        role_note = Some(format!("(roles unavailable: {})", error.lines().next().unwrap_or("invalid table")));
    }
    let (hits, unresolved) = index.search(&q, deck.as_ref())?;
    // phase 4: the engine, deck and findings facts, loaded only when the output or a filter needs them
    let need_engine = q.json || q.full || q.status.is_some();
    let need_decks = q.json || q.full || q.in_deck.is_some() || q.not_in_deck.is_some();
    let facts = if need_engine || need_decks {
        Some(Facts::load(project, &index, need_engine, need_decks).map_err(FindError::Failed)?)
    } else {
        None
    };
    let hits = match &facts {
        Some(f) => {
            let in_bit = q.in_deck.as_deref().map(|k| f.deck_bit(k)).transpose()?;
            let not_bit = q.not_in_deck.as_deref().map(|k| f.deck_bit(k)).transpose()?;
            hits.into_iter()
                .filter(|&i| {
                    let bits = f.deck_bits.get(i as usize).copied().unwrap_or(0);
                    q.status.as_ref().is_none_or(|l| status_selected(l, f.status_of(i).map(|s| s.0)))
                        && in_bit.is_none_or(|b| bits & b != 0)
                        && not_bit.is_none_or(|b| bits & b == 0)
                })
                .collect()
        }
        None => hits,
    };
    if let (Some(d), false) = (&deck, unresolved.is_empty()) {
        let shown: Vec<&str> = unresolved.iter().take(10).map(String::as_str).collect();
        let more = if unresolved.len() > 10 { ", ..." } else { "" };
        notes.push(format!(
            "(deck {}: {} name(s) not in the catalog: {}{more})",
            d.key,
            unresolved.len(),
            shown.join(", ")
        ));
    }
    let notes_on_stderr = q.json || q.count;
    let exclude = || match &deck {
        Some(d) => index.deck_members(d).0,
        None => vec![false; index.cards.len()],
    };
    // a name that matched cards the filters removed: say why, for every such card
    let named_but_filtered = |notes: &mut Vec<String>| {
        let Some(n) = &q.name else { return };
        let found = index.lookup(n);
        if found.is_empty() {
            return;
        }
        let exclude = exclude();
        let mut why: Vec<String> = vec![];
        for &i in found {
            for w in why_filtered(&index.cards[i as usize], i, &q, ident, &exclude, deck.as_ref(), facts.as_ref()) {
                if !why.contains(&w) {
                    why.push(w);
                }
            }
        }
        let hint = if why.iter().any(|w| w.starts_with("legality ")) { "; try --legal any" } else { "" };
        notes.push(format!(
            "({} card(s) named {n} exist outside these filters: {}{hint})",
            found.len(),
            why.join(" / ")
        ));
    };
    if q.count {
        if hits.is_empty() {
            named_but_filtered(&mut notes);
        }
        return Ok(FindOutput { out: format!("{}\n", hits.len()), notes, notes_on_stderr, role_note });
    }
    let total = hits.len();
    let shown_ids: Vec<u32> = hits.iter().copied().skip(q.offset).take(q.limit).collect();
    let shown: Vec<&Card> = shown_ids.iter().map(|i| &index.cards[*i as usize]).collect();
    let end = q.offset.saturating_add(shown.len());
    if !shown.is_empty() && end < total {
        notes.push(format!("(showing {}-{end} of {total}; --offset {end} or --limit all)", q.offset + 1));
    }
    if shown.is_empty() && total == 0 {
        named_but_filtered(&mut notes);
        if !q.meld && q.name.is_none() {
            let regexes = compile_texts(&q)?;
            let hidden = index.matches(&Query { meld: true, ..q.clone() }, ident, &exclude(), &regexes)?;
            if !hidden.is_empty() {
                notes.push(format!("({} meld result(s) are hidden; add --meld)", hidden.len()));
            }
        }
    }
    let out = if q.json {
        json_array(&shown, &index, facts.as_ref().expect("facts are loaded for JSON"))
    } else if shown.is_empty() {
        (if total == 0 { "no matches\n" } else { "no matches in this page\n" }).to_string()
    } else if q.full {
        let f = facts.as_ref().expect("facts are loaded for --full");
        shown.iter().zip(&shown_ids).map(|(c, i)| readable(c, Some(*i), f)).collect::<Vec<_>>().join("\n")
    } else {
        table(&shown)
    };
    Ok(FindOutput { out, notes, notes_on_stderr, role_note })
}
