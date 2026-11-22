//! The local-LLM stage of the role classifier: the fixed prompt, the OpenAI-compatible client, the cache and the pure
//! merge policy. The rule stage (`roles.rs`) is not reopened here; this module only reads its assignments.
//!
//! The model sees a card's type line and its numbered, normalised Oracle lines (`roles::prompt_lines`), never the
//! rule roles. Every role it proposes must cite a real line and pass that role's guard regex on the cited text
//! (`roles::line_text`); anything else is retried, then dropped, and a card that cannot be classified keeps its rule
//! roles with `llm_failed` set.
use crate::knowledge::{Card, Index, Legal};
use crate::roles::{self, Assignment, Evidence, RoleMap, RoleMeta, RouteReason, Source, Taxonomy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8080/v1";
pub const DEFAULT_BATCH: usize = 10;
const MAX_ROLES: usize = 4;
/// Consecutive transport failures (no answer at all) after which a run saves what it has and stops.
const MAX_TRANSPORT_FAILURES: usize = 2;
/// Consecutive batches in which not one card got a valid answer (a server rejecting every request) after which it stops.
const MAX_EMPTY_BATCHES: usize = 3;
/// A run that leaves more than this share of the cards it asked about `llm_failed` exits non-zero.
const MAX_FAILED_SHARE: f64 = 0.20;
/// Seconds one request may take (a batch of ten answers in about ten).
const REQUEST_TIMEOUT_SECS: u64 = 180;

// ---------------------------------------------------------------------------------------------------- the prompt

const INTRO: &str = concat!(
    "You label Magic: The Gathering cards with deck-building roles. You see only a card's type line and its ",
    "numbered Oracle lines (the card's own name is written ~, reminder text is removed). You know nothing else ",
    "about the card and must not guess from its name or from deck context."
);
/// Which prompt and schema a run uses. `Think` (the default) adds a short `why` sentence BEFORE the roles, so the model
/// names the lines it considered before it commits to roles; `Plain` answers `{"id","roles"}`. Chosen by
/// `CARDGEN_LLM_VARIANT` (`think` or `plain`); the two have different prompt fingerprints, so their cache rows never mix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    Plain,
    #[default]
    Think,
}
impl Variant {
    pub fn parse(v: &str) -> Result<Variant, String> {
        match v.trim() {
            "" | "think" => Ok(Variant::Think),
            "plain" => Ok(Variant::Plain),
            other => Err(format!("CARDGEN_LLM_VARIANT must be 'plain' or 'think', not '{other}'")),
        }
    }
    /// The variant named by the environment (an error for an unknown name).
    pub fn from_env() -> Result<Variant, String> {
        Variant::parse(&std::env::var("CARDGEN_LLM_VARIANT").unwrap_or_default())
    }
    /// The variant the convenience functions use: the environment's, `Plain` when unset or unknown.
    pub fn current() -> Variant {
        Variant::from_env().unwrap_or_default()
    }
    pub fn name(self) -> &'static str {
        match self {
            Variant::Plain => "plain",
            Variant::Think => "think",
        }
    }
}
/// Longest `why` the schema allows, in characters.
pub const WHY_MAX: usize = 160;

fn answer_rules(v: Variant) -> String {
    let entry = match v {
        Variant::Plain => "{\"id\": the card's id, \"roles\": [...]}".to_string(),
        Variant::Think => format!(
            "{{\"id\": the card's id, \"why\": one short sentence (at most {WHY_MAX} characters) naming the lines you \
             considered and what each does, \"roles\": [...]}}"
        ),
    };
    format!(
        concat!(
            "HOW TO ANSWER\n",
            "Work through each card line by line: for each Oracle line decide which roles it fulfils, collect every role ",
            "from every line, then keep at most 4, most prominent first. A card with several abilities usually has ",
            "several roles: do not stop after the first one. Check a role's NO list before you give it. ",
            "Reply with one JSON object {{\"cards\":[...]}} with exactly one entry per card, in the order given: {entry}. ",
            "Each element of roles is {{\"role\": a role id, \"line\": the number before the line where that effect is ",
            "written}}. The first role is the card's primary role. If no role applies, roles is []. Riders, costs, ",
            "drawbacks, printed keywords and conditions that merely mention an effect give no role, except as GENERAL ",
            "RULES says. Most plain bodies and minor-rider cards have no role."
        ),
        entry = entry
    )
}

/// One worked example: a card as the model sees it and the correct answer. None of these cards is in a gold set.
pub struct Example {
    pub name: &'static str,
    /// the `why` sentence of the `think` variant
    pub why: &'static str,
    pub type_line: &'static str,
    pub lines: &'static [(usize, &'static str)],
    pub roles: &'static [&'static str],
    pub evidence: &'static [(&'static str, usize)],
}
/// A rider on a removal spell, a cost-side counter, a "would draw" replacement, a permanent steal, a one-turn Fog, a body
/// with no roles, two cards with two roles, one with three, and a fetchland (so the format, the order conventions and
/// the completeness habit are all seen). The answers follow the written conventions.
pub const EXAMPLES: [Example; 10] = [
    Example {
        name: "Chastise",
        why: "Line 0 destroys one creature (removal); the life gain is a minor rider.",
        type_line: "Instant",
        lines: &[(0, "Destroy target attacking creature. You gain life equal to its power.")],
        roles: &["removal"],
        evidence: &[("removal", 0)],
    },
    Example {
        name: "Ancient Hydra",
        why: "Line 1 pings any target, paying with a fade counter: repeatable damage is removal; the counter is a cost.",
        type_line: "Creature — Hydra",
        lines: &[
            (0, "Fading 5"),
            (1, "{1}, Remove a fade counter from this creature: It deals 1 damage to any target."),
        ],
        roles: &["removal"],
        evidence: &[("removal", 1)],
    },
    Example {
        name: "Obstinate Familiar",
        why: "Line 0 only mentions drawing as a condition and lets you skip it: no role.",
        type_line: "Creature — Lizard",
        lines: &[(0, "If you would draw a card, you may skip that draw instead.")],
        roles: &[],
        evidence: &[],
    },
    Example {
        name: "Mind Control",
        why: "Line 1 gains control of the enchanted creature for good: removal.",
        type_line: "Enchantment — Aura",
        lines: &[(0, "Enchant creature"), (1, "You control enchanted creature.")],
        roles: &["removal"],
        evidence: &[("removal", 1)],
    },
    Example {
        name: "Holy Day",
        why: "Line 0 prevents all combat damage this turn, a Fog: protection.",
        type_line: "Instant",
        lines: &[(0, "Prevent all combat damage that would be dealt this turn.")],
        roles: &["protection"],
        evidence: &[("protection", 0)],
    },
    Example {
        name: "Kird Ape",
        why: "Line 0 only pumps this creature: no role.",
        type_line: "Creature — Ape",
        lines: &[(0, "This creature gets +1/+2 as long as you control a Forest.")],
        roles: &[],
        evidence: &[],
    },
    Example {
        name: "Pitiless Plunderer",
        why: "Line 0 makes a Treasure each time a creature dies: tokens, and a repeatable Treasure maker is also ramp.",
        type_line: "Creature — Human Pirate",
        lines: &[(0, "Whenever another creature you control dies, create a Treasure token.")],
        roles: &["tokens", "ramp"],
        evidence: &[("tokens", 0), ("ramp", 0)],
    },
    Example {
        name: "Cackling Counterpart",
        why: "Line 0 makes a token copy of a creature: copy first, then tokens; flashback on line 1 is no role.",
        type_line: "Instant",
        lines: &[(0, "Create a token that's a copy of target creature you control."), (1, "Flashback {5}{U}{U}")],
        roles: &["copy", "tokens"],
        evidence: &[("copy", 0), ("tokens", 0)],
    },
    Example {
        name: "Tireless Tracker",
        why: "Line 0 investigates on every landfall: tokens and draw; line 1 grows it for each Clue sacrificed: counters.",
        type_line: "Creature — Human Scout",
        lines: &[
            (0, "Landfall — Whenever a land you control enters, investigate."),
            (1, "Whenever you sacrifice a Clue, put a +1/+1 counter on this creature."),
        ],
        roles: &["tokens", "draw", "counters"],
        evidence: &[("tokens", 0), ("draw", 0), ("counters", 1)],
    },
    Example {
        name: "Wooded Foothills",
        why: "Line 0 sacrifices the land to fetch a Mountain or Forest: a fetchland is fixing, not ramp.",
        type_line: "Land",
        lines: &[(
            0,
            "{T}, Pay 1 life, Sacrifice this land: Search your library for a Mountain or Forest card, put it onto the \
             battlefield, then shuffle.",
        )],
        roles: &["fixing"],
        evidence: &[("fixing", 0)],
    },
];

/// Batch id of the `i`th card of a batch: a letter and a digit (`a0` ... `a9`, `b0`, ...).
pub fn batch_id(i: usize) -> String {
    format!("{}{}", (b'a' + (i / 10) as u8) as char, i % 10)
}

/// One card of a batch as the model sees it.
pub struct Shown<'a> {
    pub id: String,
    pub type_line: &'a str,
    pub lines: &'a [(usize, String)],
    /// unnumbered headers printed before the line with that index (`roles::prompt_headers`)
    pub headers: &'a BTreeMap<usize, String>,
}
/// The cards of a batch as the user message shows them, with each later face announced by its header.
pub fn user_message_cards(cards: &[Shown]) -> String {
    let mut out = format!("Classify these {} cards.\n", cards.len());
    for c in cards {
        let _ = writeln!(out, "[{}] {}", c.id, c.type_line);
        for (i, text) in c.lines {
            if let Some(h) = c.headers.get(i) {
                let _ = writeln!(out, "{h}");
            }
            let _ = writeln!(out, "{i}: {text}");
        }
    }
    out
}
/// The cards of a batch as the user message shows them (no face headers).
pub fn user_message(cards: &[(String, &str, &[(usize, String)])]) -> String {
    let none = BTreeMap::new();
    let shown: Vec<Shown> =
        cards.iter().map(|(id, type_line, lines)| Shown { id: id.clone(), type_line, lines, headers: &none }).collect();
    user_message_cards(&shown)
}

/// A JSON string literal.
fn quoted(s: &str) -> String {
    serde_json::to_string(s).expect("string")
}
/// An answer entry in the schema's property order: the card's id, then its roles most prominent first, each with the
/// line that supports it (`evidence` pairs a role with its line; the first role is the primary).
pub fn answer_entry(id: &str, roles: &[&str], evidence: &[(&str, usize)]) -> String {
    answer_entry_v(Variant::Plain, id, "", roles, evidence)
}
/// [`answer_entry`] for a variant: `Think` writes `why` between the id and the roles.
pub fn answer_entry_v(v: Variant, id: &str, why: &str, roles: &[&str], evidence: &[(&str, usize)]) -> String {
    let list = roles
        .iter()
        .map(|r| {
            let line = evidence.iter().find(|(e, _)| e == r).map_or(0, |(_, n)| *n);
            format!("{{\"role\":{},\"line\":{line}}}", quoted(r))
        })
        .collect::<Vec<_>>()
        .join(",");
    match v {
        Variant::Plain => format!("{{\"id\":{},\"roles\":[{list}]}}", quoted(id)),
        Variant::Think => format!("{{\"id\":{},\"why\":{},\"roles\":[{list}]}}", quoted(id), quoted(why)),
    }
}

/// The worked examples as a real conversation turn: the user message that lists the example cards and the assistant's
/// answer. Sent between the system prompt and the real user message, so the static prefix still caches.
pub fn example_turn() -> (String, String) {
    example_turn_v(Variant::current())
}
pub fn example_turn_v(v: Variant) -> (String, String) {
    let lines: Vec<Vec<(usize, String)>> =
        EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let cards: Vec<(String, &str, &[(usize, String)])> =
        EXAMPLES.iter().zip(&lines).enumerate().map(|(i, (e, l))| (batch_id(i), e.type_line, l.as_slice())).collect();
    let entries: Vec<String> =
        EXAMPLES.iter().enumerate().map(|(i, e)| answer_entry_v(v, &batch_id(i), e.why, e.roles, e.evidence)).collect();
    (user_message(&cards), format!("{{\"cards\":[{}]}}", entries.join(",")))
}

/// The system prompt: the taxonomy and the fixed answer rules. (The worked examples follow as a user/assistant turn.)
pub fn system_prompt(t: &Taxonomy) -> String {
    system_prompt_v(t, Variant::current())
}
pub fn system_prompt_v(t: &Taxonomy, v: Variant) -> String {
    format!("{INTRO}\n\n{}\n\n{}\n", t.prompt_text(), answer_rules(v))
}
/// Everything that precedes the real user message: the system prompt, the example user message and its answer.
pub fn preface_text(t: &Taxonomy) -> String {
    preface_text_v(t, Variant::current())
}
pub fn preface_text_v(t: &Taxonomy, v: Variant) -> String {
    let (user, answer) = example_turn_v(v);
    format!("{}\n--- example user message ---\n{user}--- example answer ---\n{answer}\n", system_prompt_v(t, v))
}
/// The prompt fingerprint that, with the model id and the card text, keys the cache: any change to what the model is
/// told (definitions, clauses, answer rules, examples, the variant) invalidates; guards, `keep_rule` and `llm_add` are
/// not part of it.
pub fn prompt_fnv(t: &Taxonomy) -> String {
    prompt_fnv_v(t, Variant::current())
}
pub fn prompt_fnv_v(t: &Taxonomy, v: Variant) -> String {
    prompt_fnv_opts(t, v, false)
}
/// [`prompt_fnv_v`] for a run that sends `enable_thinking: false` (`--no-think`): a marker in the fingerprint keeps
/// the rows of thinking and non-thinking runs apart.
pub fn prompt_fnv_opts(t: &Taxonomy, v: Variant, no_think: bool) -> String {
    let mut text = preface_text_v(t, v);
    if no_think {
        text.push_str("\n--- chat_template_kwargs: enable_thinking=false ---\n");
    }
    roles::fingerprint(text.as_bytes())
}
/// Rough token count used for budgeting: characters / 4.
pub fn estimate_tokens(s: &str) -> usize {
    s.chars().count().div_ceil(4)
}

/// The JSON schema (llama.cpp grammar-constrains the output to it) for a batch of `n` cards, as text so that the
/// property order is the one written here. One array carries both the roles and their lines, so a grammar that only
/// allows optional keys in declared order cannot block a legal answer; `id` is an enum of the batch's ids. If a server
/// rejects `minimum` (HTTP 400), drop `,"minimum":0` below: the validator refuses a negative line anyway.
pub fn response_schema_text(t: &Taxonomy, n: usize) -> String {
    response_schema_text_v(t, n, Variant::current())
}
/// [`response_schema_text`] for a variant: `Think` adds a required `why` string (at most [`WHY_MAX`] characters)
/// between `id` and `roles`, so the grammar makes the model write it first.
pub fn response_schema_text_v(t: &Taxonomy, n: usize, v: Variant) -> String {
    let names = t.role.iter().map(|r| quoted(&r.id)).collect::<Vec<_>>().join(",");
    let ids = (0..n).map(|i| quoted(&batch_id(i))).collect::<Vec<_>>().join(",");
    let (why, required) = match v {
        Variant::Plain => (String::new(), "[\"id\",\"roles\"]"),
        Variant::Think => {
            (format!("\"why\":{{\"type\":\"string\",\"maxLength\":{WHY_MAX}}},"), "[\"id\",\"why\",\"roles\"]")
        }
    };
    format!(
        concat!(
            "{{\"type\":\"object\",\"properties\":{{\"cards\":{{\"type\":\"array\",\"minItems\":{n},\"maxItems\":{n},",
            "\"items\":{{\"type\":\"object\",\"properties\":{{\"id\":{{\"enum\":[{ids}]}},{why}",
            "\"roles\":{{\"type\":\"array\",\"maxItems\":{max},\"items\":{{\"type\":\"object\",\"properties\":{{",
            "\"role\":{{\"enum\":[{names}]}},\"line\":{{\"type\":\"integer\",\"minimum\":0}}}},",
            "\"required\":[\"role\",\"line\"],\"additionalProperties\":false}}}}}},",
            "\"required\":{required},\"additionalProperties\":false}}}}}},",
            "\"required\":[\"cards\"],\"additionalProperties\":false}}"
        ),
        n = n,
        max = MAX_ROLES,
        names = names,
        ids = ids,
        why = why,
        required = required,
    )
}
/// The same schema as a value (key order not preserved), for inspection.
pub fn response_schema(t: &Taxonomy, n: usize) -> Value {
    serde_json::from_str(&response_schema_text(t, n)).expect("the schema text is JSON")
}

// ----------------------------------------------------------------------------------------------- the model's answer

/// One card's answer: roles in the model's order, the primary and the cited line of each role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LlmCard {
    pub roles: Vec<String>,
    pub primary: Option<String>,
    pub evidence: BTreeMap<String, usize>,
    /// the model's one-sentence reasoning (`think` variant only)
    #[serde(default)]
    pub why: Option<String>,
}

/// Check a model reply against the batch it answers, card by card. `cards` are the batch ids with the numbered lines
/// shown (the only citable ones). `Err` means the document as a whole is unusable (not JSON, no `cards` array). Each
/// card then gets its own verdict: `Err` for a structural fault (its id missing or repeated, a role outside the
/// vocabulary or repeated, more than four roles, a line that is missing, out of range or was not shown), `Ok` for an
/// answer. A role whose guard fails on its cited line is NOT a fault: the raw answer is kept and the merge drops
/// that role as a `GuardDrop`.
pub fn validate_output(
    content: &str,
    cards: &[(String, &[(usize, String)])],
    taxonomy: &Taxonomy,
) -> Result<Vec<Result<LlmCard, String>>, String> {
    validate_output_v(content, cards, taxonomy, Variant::current())
}
/// [`validate_output`] for a variant: in `Think` every entry must carry a `why` string of at most [`WHY_MAX`] characters.
pub fn validate_output_v(
    content: &str,
    cards: &[(String, &[(usize, String)])],
    taxonomy: &Taxonomy,
    variant: Variant,
) -> Result<Vec<Result<LlmCard, String>>, String> {
    let _ = taxonomy;
    let doc: Value = serde_json::from_str(content).map_err(|e| format!("not JSON: {e}"))?;
    let entries = doc.get("cards").and_then(Value::as_array).ok_or("no cards array")?;
    let mut by_id: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
    for e in entries {
        if let Some(id) = e.get("id").and_then(Value::as_str) {
            by_id.entry(id).or_default().push(e);
        }
    }
    Ok(cards
        .iter()
        .map(|(id, lines)| match by_id.get(id.as_str()).map(Vec::as_slice) {
            None | Some([]) => Err(format!("{id}: missing id")),
            Some([e]) => check_card(id, e, lines, variant),
            Some(_) => Err(format!("{id}: repeated id")),
        })
        .collect())
}

fn check_card(id: &str, e: &Value, lines: &[(usize, String)], variant: Variant) -> Result<LlmCard, String> {
    let bad = |what: &str| format!("{id}: {what}");
    let why = match variant {
        Variant::Plain => None,
        Variant::Think => {
            let w = e.get("why").and_then(Value::as_str).ok_or_else(|| bad("no why sentence"))?;
            if w.chars().count() > WHY_MAX {
                return Err(bad("why is longer than the limit"));
            }
            Some(w.to_string())
        }
    };
    let items = e.get("roles").and_then(Value::as_array).ok_or_else(|| bad("roles is not an array"))?;
    if items.len() > MAX_ROLES {
        return Err(bad("more than 4 roles"));
    }
    let mut roles = vec![];
    let mut evidence = BTreeMap::new();
    for item in items {
        let role = item.get("role").and_then(Value::as_str).ok_or_else(|| bad("a role entry has no role id"))?;
        if !roles::IDS.contains(&role) {
            return Err(bad(&format!("role {role} is not in the vocabulary")));
        }
        if roles.iter().any(|r| r == role) {
            return Err(bad(&format!("repeated role {role}")));
        }
        let n =
            item.get("line").and_then(Value::as_u64).ok_or_else(|| bad(&format!("no line number cited for {role}")))?
                as usize;
        if !lines.iter().any(|(i, _)| *i == n) {
            return Err(bad(&format!("{role} cites line {n}, which is out of range or skipped")));
        }
        roles.push(role.to_string());
        evidence.insert(role.to_string(), n);
    }
    Ok(LlmCard { primary: roles.first().cloned(), roles, evidence, why })
}

// ------------------------------------------------------------------------------------------------------- the cache

/// What the model is asked about a card: the type line and the numbered lines, exactly as shown to it.
pub fn card_text(card: &Card) -> String {
    text_of(&card.raw.type_line, &roles::prompt_lines(card), &roles::prompt_headers(card))
}
fn text_of(type_line: &str, lines: &[(usize, String)], headers: &BTreeMap<usize, String>) -> String {
    let mut s = type_line.to_string();
    for (i, text) in lines {
        if let Some(h) = headers.get(i) {
            let _ = write!(s, "\n{h}");
        }
        let _ = write!(s, "\n{i}: {text}");
    }
    s
}
/// The cache key: the normalised card text, the prompt fingerprint and the model id. Never the rules fingerprint: a
/// code edit must not discard hours of GPU work.
pub fn cache_key(card_text: &str, prompt: &str, model: &str) -> String {
    roles::fingerprint(format!("{card_text}\u{1}{prompt}\u{1}{model}").as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub key: String,
    pub name: String,
    pub roles: Vec<String>,
    pub primary: Option<String>,
    pub evidence: BTreeMap<String, usize>,
    pub model: String,
    pub taxonomy: String,
    pub prompt: String,
    /// the model's one-sentence reasoning (`think` variant rows only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}
impl Row {
    pub fn card(&self) -> LlmCard {
        LlmCard {
            roles: self.roles.clone(),
            primary: self.primary.clone(),
            evidence: self.evidence.clone(),
            why: self.why.clone(),
        }
    }
}

pub fn cache_path(dir: &Path) -> PathBuf {
    dir.join("llm_roles.jsonl")
}
pub fn pending_path(dir: &Path) -> PathBuf {
    dir.join("llm_roles.pending.jsonl")
}
pub fn reports_dir(dir: &Path) -> PathBuf {
    dir.join("reports")
}

/// Rows of one JSON Lines file keyed by `key`; a missing file is empty.
pub fn read_rows(path: &Path) -> Result<BTreeMap<String, Row>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let mut rows = BTreeMap::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let row: Row = serde_json::from_str(line).map_err(|e| format!("{} line {}: {e}", path.display(), n + 1))?;
        if let Some(r) = row.roles.iter().chain(row.evidence.keys()).find(|r| !roles::IDS.contains(&r.as_str())) {
            return Err(format!("{} line {}: role '{r}' is not in the vocabulary", path.display(), n + 1));
        }
        rows.insert(row.key.clone(), row);
    }
    Ok(rows)
}
/// Sorted by key, one row per line; deterministic.
pub fn rows_bytes(rows: &BTreeMap<String, Row>) -> Vec<u8> {
    let mut out = String::new();
    for row in rows.values() {
        out.push_str(&serde_json::to_string(row).expect("row"));
        out.push('\n');
    }
    out.into_bytes()
}
/// Write a file by temp name and rename, so a crash never leaves half a file.
pub fn write_atomic(path: &Path, data: &[u8]) -> Result<(), String> {
    let tmp = path.with_file_name(format!(
        "{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("out"),
        std::process::id()
    ));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&tmp, data).map_err(|e| format!("{}: {e}", tmp.display()))?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("{}: {e}", path.display()));
    }
    Ok(())
}

/// The tracked cache only: the scheduled job's pending rows are unreviewed and are read only when asked for
/// ([`load_cache_with`], `roles --merge --pending`).
pub fn load_cache(dir: &Path) -> Result<BTreeMap<String, Row>, String> {
    load_cache_with(dir, false)
}
/// The tracked cache, and the pending file after it when `pending` is true (a pending row wins on the same key).
pub fn load_cache_with(dir: &Path, pending: bool) -> Result<BTreeMap<String, Row>, String> {
    let mut rows = read_rows(&cache_path(dir))?;
    if pending {
        rows.extend(read_rows(&pending_path(dir))?);
    }
    Ok(rows)
}

// ------------------------------------------------------------------------------------------------------ the client

enum Fail {
    /// no answer at all: the connection failed, was cut or timed out
    Transport(String),
    /// the server answered with an error status (the reply body says why, e.g. a rejected grammar)
    Status(u16, String),
    /// the answer arrived but its document is unusable
    Invalid(String),
}
impl Fail {
    fn why(&self) -> String {
        match self {
            Fail::Transport(e) => format!("transport: {e}"),
            Fail::Status(code, body) => format!("HTTP {code}: {body}"),
            Fail::Invalid(e) => format!("invalid answer: {e}"),
        }
    }
}
/// The start of an error reply's body, on one line.
fn snippet(body: &str) -> String {
    let one: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    one.chars().take(300).collect()
}

/// The model id the cache is keyed on: a server reports the path it was started with (`-m /models/x.gguf`), so only the
/// file name counts.
pub fn model_key(id: &str) -> String {
    id.rsplit(['/', '\\']).next().unwrap_or(id).to_string()
}

pub struct Client {
    agent: ureq::Agent,
    base: String,
}
impl Client {
    pub fn new(endpoint: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build();
        Self { agent, base: endpoint.trim_end_matches('/').to_string() }
    }
    /// The id of the loaded model, from GET /models (its file name, see [`model_key`]).
    pub fn model(&self) -> Result<String, String> {
        let url = format!("{}/models", self.base);
        let unreachable = |why: String| format!("LLM unreachable: {url}: {why}");
        let body = match self.agent.get(&url).call() {
            Ok(r) => r.into_string().map_err(|e| unreachable(e.to_string()))?,
            Err(ureq::Error::Status(code, r)) => {
                return Err(unreachable(format!("HTTP {code}: {}", snippet(&r.into_string().unwrap_or_default()))));
            }
            Err(e) => return Err(unreachable(e.to_string())),
        };
        let v: Value = serde_json::from_str(&body).map_err(|e| unreachable(format!("bad /models reply: {e}")))?;
        let id = v["data"][0]["id"].as_str().or_else(|| v["models"][0]["model"].as_str()).map(model_key);
        id.filter(|m| !m.trim().is_empty()).ok_or_else(|| unreachable("/models names no model".into()))
    }
    fn chat(&self, body: &str) -> Result<String, Fail> {
        let resp = match self
            .agent
            .post(&format!("{}/chat/completions", self.base))
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                return Err(Fail::Status(code, snippet(&r.into_string().unwrap_or_default())));
            }
            Err(e) => return Err(Fail::Transport(e.to_string())),
        };
        let text = resp.into_string().map_err(|e| Fail::Transport(e.to_string()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| Fail::Invalid(format!("reply is not JSON: {e}")))?;
        let choice = v["choices"].get(0).ok_or_else(|| Fail::Invalid("reply has no choices".into()))?;
        if choice["finish_reason"].as_str() == Some("length") {
            return Err(Fail::Invalid("reply was truncated".into()));
        }
        choice["message"]["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| Fail::Invalid("reply has no content".into()))
    }
}

/// The request body of one chat call (text, so the schema keeps its property order): the system prompt, the worked
/// example as a user/assistant turn, then the real user message.
pub fn chat_body(model: &str, system: &str, user: &str, schema: &str, n: usize) -> String {
    chat_body_v(model, system, user, schema, n, Variant::current())
}
pub fn chat_body_v(model: &str, system: &str, user: &str, schema: &str, n: usize, v: Variant) -> String {
    chat_body_opts(model, system, user, schema, n, v, false)
}
/// [`chat_body_v`] with `chat_template_kwargs: {"enable_thinking": false}` when `no_think` is set (a reasoning model
/// would otherwise spend its token budget thinking and truncate).
pub fn chat_body_opts(
    model: &str,
    system: &str,
    user: &str,
    schema: &str,
    n: usize,
    v: Variant,
    no_think: bool,
) -> String {
    let (example_user, example_answer) = example_turn_v(v);
    let kwargs = if no_think { ",\"chat_template_kwargs\":{\"enable_thinking\":false}" } else { "" };
    // a `why` sentence is about 40 tokens more per card
    let per_card = if v == Variant::Think { 210 } else { 160 };
    format!(
        concat!(
            "{{\"model\":{},\"messages\":[{{\"role\":\"system\",\"content\":{}}},",
            "{{\"role\":\"user\",\"content\":{}}},{{\"role\":\"assistant\",\"content\":{}}},",
            "{{\"role\":\"user\",\"content\":{}}}],\"temperature\":0,\"seed\":1,\"cache_prompt\":true,",
            "\"max_tokens\":{},\"response_format\":{{\"type\":\"json_schema\",",
            "\"json_schema\":{{\"name\":\"role_labels\",\"strict\":true,\"schema\":{}}}}}{}}}"
        ),
        quoted(model),
        quoted(system),
        quoted(&example_user),
        quoted(&example_answer),
        quoted(user),
        per_card * n + 64,
        schema,
        kwargs
    )
}

// ---------------------------------------------------------------------------------------------------- the merge

/// What the merge policy needs from outside: `keep` says whether a rule's non-certain role survives an LLM omission
/// (the per-role `keep_rule` of roles.toml), `check` says whether the role's guard passes on the cited line and, if
/// so, gives the line's face index (`None` outside `Some` means the line is not citable or the guard fails).
pub struct MergeEnv<'a> {
    pub keep: &'a dyn Fn(&str) -> bool,
    pub check: &'a dyn Fn(&str, usize) -> Option<Option<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    /// a non-certain rule role the LLM did not confirm was dropped
    Veto,
    /// an LLM role whose guard failed on its cited line was dropped
    GuardDrop,
    /// a certain rule role the LLM omitted was kept
    CertainOmitted,
    /// a role was cut to stay within four
    Overflow,
    /// a rules-final card (never changed) whose role set the LLM would have given differently
    Shadow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Disagreement {
    pub card: String,
    pub kind: Kind,
    pub role: String,
    pub route: Vec<RouteReason>,
    pub rule: Vec<String>,
    pub llm: Vec<String>,
    pub merged: Vec<String>,
}

/// The merge policy, a pure function of one card's rule assignment, its LLM answer (`None` when it failed or has not
/// run), its override and the environment:
///
/// * an override is used exactly;
/// * rule-`certain` roles are always kept;
/// * `needs_llm == false`: the rule roles are used, the LLM never changes them;
/// * `needs_llm == true` with an answer: `certain` plus the LLM roles whose guard passes on the cited line plus the
///   non-certain rule roles the LLM also proposed or that `keep` protects; order is the LLM primary, the LLM order,
///   the certain roles, then the kept rule roles by prominence; at most four (never cutting a certain role);
/// * `needs_llm == true` without an answer: the rule roles with `llm_failed` set.
///
/// Every veto, guard drop, omitted certain role and overflow comes back as a [`Disagreement`] (the card name is left
/// empty for the caller to fill in).
pub fn merge(
    rule: &Assignment,
    llm: Option<&LlmCard>,
    ovr: Option<&Assignment>,
    env: &MergeEnv,
) -> (Assignment, Vec<Disagreement>) {
    if let Some(o) = ovr {
        return (o.clone(), vec![]);
    }
    let note = |kind: Kind, role: &str, llm: &[String], merged: &[String]| Disagreement {
        card: String::new(),
        kind,
        role: role.to_string(),
        route: rule.route.clone(),
        rule: rule.roles.clone(),
        llm: llm.to_vec(),
        merged: merged.to_vec(),
    };
    let certain = |r: &str| rule.evidence.get(r).is_some_and(|e| e.certain);
    if !rule.needs_llm {
        let mut dis = vec![];
        if let Some(l) = llm {
            for r in rule.roles.iter().filter(|r| !l.roles.contains(r)) {
                dis.push(note(Kind::Shadow, r, &l.roles, &rule.roles));
            }
            for r in l.roles.iter().filter(|r| !rule.roles.contains(r)) {
                dis.push(note(Kind::Shadow, r, &l.roles, &rule.roles));
            }
        }
        return (rule.clone(), dis);
    }
    let Some(l) = llm else {
        let mut a = rule.clone();
        a.llm_failed = true;
        return (a, vec![]);
    };
    let mut dis = vec![];
    // The LLM's roles that pass their guard, with the face of the cited line.
    let mut guarded: Vec<(&String, usize, Option<usize>)> = vec![];
    let mut dropped: Vec<&String> = vec![];
    for r in &l.roles {
        match l.evidence.get(r).and_then(|&line| (env.check)(r, line).map(|face| (line, face))) {
            Some((line, face)) => guarded.push((r, line, face)),
            None => dropped.push(r),
        }
    }
    let kept_rule = |r: &String| rule.roles.contains(r) && !certain(r) && (l.roles.contains(r) || (env.keep)(r));
    let in_final = |r: &String| certain(r) || guarded.iter().any(|(g, _, _)| *g == r) || kept_rule(r);
    let mut order: Vec<String> = vec![];
    let push = |r: &String, order: &mut Vec<String>| {
        if !order.contains(r) {
            order.push(r.clone());
        }
    };
    if let Some(p) = l.primary.as_ref().filter(|p| in_final(p)) {
        push(p, &mut order);
    }
    for r in l.roles.iter().filter(|r| in_final(r)) {
        push(r, &mut order);
    }
    for r in rule.roles.iter().filter(|r| certain(r)) {
        push(r, &mut order);
    }
    for r in rule.roles.iter().filter(|r| kept_rule(r)) {
        push(r, &mut order);
    }
    for r in rule.roles.iter().filter(|r| !certain(r) && !in_final(r)) {
        dis.push(note(Kind::Veto, r, &l.roles, &order));
    }
    for r in dropped.iter().filter(|r| !order.contains(r)) {
        dis.push(note(Kind::GuardDrop, r, &l.roles, &order));
    }
    for r in rule.roles.iter().filter(|r| certain(r) && !l.roles.contains(r)) {
        dis.push(note(Kind::CertainOmitted, r, &l.roles, &order));
    }
    // At most four; a certain role is never the one cut.
    let mut cut = vec![];
    while order.len() > MAX_ROLES {
        match order.iter().rposition(|r| !certain(r)) {
            Some(i) => cut.push(order.remove(i)),
            None => break,
        }
    }
    for r in &cut {
        dis.push(note(Kind::Overflow, r, &l.roles, &order));
    }
    let evidence: BTreeMap<String, Evidence> = order
        .iter()
        .map(|r| {
            let e = match (rule.evidence.get(r), guarded.iter().find(|(g, _, _)| *g == r)) {
                (Some(e), _) => e.clone(),
                (None, Some((_, line, face))) => {
                    Evidence { source: Source::Llm, certain: false, rule: "llm".into(), face: *face, line: Some(*line) }
                }
                (None, None) => unreachable!("a final role has rule or guarded LLM evidence"),
            };
            (r.clone(), e)
        })
        .collect();
    let merged = Assignment {
        primary: order.first().cloned().unwrap_or_default(),
        roles: order,
        evidence,
        needs_llm: true,
        route: rule.route.clone(),
        llm_failed: false,
    };
    (merged, dis)
}

// --------------------------------------------------------------------------------------------------------- runs

/// Counts of one `roles --llm` run, written to `reports/llm-run.json`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunStats {
    pub model: String,
    /// the run sent `enable_thinking: false` (`--no-think`)
    #[serde(default)]
    pub no_think: bool,
    /// the prompt variant (`think` or `plain`)
    #[serde(default)]
    pub variant: String,
    pub candidates: usize,
    pub cached: usize,
    pub classified: usize,
    pub failed: usize,
    pub batch_retries: usize,
    pub card_retries: usize,
    pub chat_requests: usize,
}
/// What the merge saw, written to `reports/merge.json`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeStats {
    /// Commander-legal routed cards the overrides do not fix
    pub routed: usize,
    pub with_answer: usize,
    /// routed legal cards without an answer: `failed` plus `not_run`
    pub llm_failed: usize,
    /// of those, cards the last run asked about and could not get a valid answer for (`reports/llm-failed.txt`)
    #[serde(default)]
    pub failed: usize,
    /// of those, cards no run has answered yet
    #[serde(default)]
    pub not_run: usize,
    /// routed cards that are not Commander-legal: never sent, never flagged
    #[serde(default)]
    pub not_legal: usize,
    /// per role switched off with `llm_add = false`: how many LLM proposals (on routed cards) the merge dropped for it
    #[serde(default)]
    pub llm_add_dropped: BTreeMap<String, usize>,
    /// route label -> disagreement kind -> count
    pub disagreements: BTreeMap<String, BTreeMap<String, usize>>,
}
/// The LLM numbers a validation report prints.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct LlmStats {
    pub run: Option<RunStats>,
    pub merge: Option<MergeStats>,
}
pub fn read_stats(dir: &Path) -> LlmStats {
    let read = |name: &str| std::fs::read(reports_dir(dir).join(name)).ok();
    LlmStats {
        run: read("llm-run.json").and_then(|b| serde_json::from_slice(&b).ok()),
        merge: read("merge.json").and_then(|b| serde_json::from_slice(&b).ok()),
    }
}

#[derive(Debug, Clone)]
pub struct LlmOptions {
    pub limit: Option<usize>,
    pub ids: Option<PathBuf>,
    pub only_routed: bool,
    pub batch: usize,
    pub dry_run: bool,
    pub endpoint: String,
    pub model: Option<String>,
    pub pending: bool,
    /// only cards whose rules-stage route contains any of these reasons (empty: no such filter)
    pub routes: Vec<RouteReason>,
    /// send `chat_template_kwargs: {"enable_thinking": false}` with every request
    pub no_think: bool,
    /// also send cards that are not Commander-legal (default: legal cards only)
    pub include_not_legal: bool,
}
impl Default for LlmOptions {
    fn default() -> Self {
        Self {
            limit: None,
            ids: None,
            only_routed: false,
            batch: DEFAULT_BATCH,
            dry_run: false,
            endpoint: DEFAULT_ENDPOINT.into(),
            model: None,
            pending: false,
            routes: vec![],
            no_think: false,
            include_not_legal: false,
        }
    }
}

fn card_label(c: &Card) -> String {
    format!("{} ({})", c.raw.name, c.raw.oracle_id)
}

/// The cards the ids file names (oracle ids or exact names, `#` comments allowed).
fn read_ids(index: &Index, path: &Path) -> Result<BTreeSet<u32>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = BTreeSet::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        if let Some(i) = index.by_oracle_id(line) {
            out.insert(i);
        } else if let Some(&i) = index.lookup(line).first() {
            out.insert(i);
        } else {
            return Err(format!("{}: no card with id or name '{line}'", path.display()));
        }
    }
    Ok(out)
}

/// The lock file of one cache target: created exclusively for the length of a `roles --llm` run, so two runs never
/// write one cache at the same moment. A second run waits for the first (up to `CARDGEN_LLM_LOCK_WAIT_SECS`, default
/// 900) and then names the file in its error, which is also how a stale lock from a crashed run is found.
struct Lock(PathBuf);
impl Lock {
    fn acquire(path: PathBuf) -> Result<Lock, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let wait = std::env::var("CARDGEN_LLM_LOCK_WAIT_SECS").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(900);
        let deadline = std::time::Instant::now() + Duration::from_secs(wait);
        loop {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut f) => {
                    use std::io::Write as _;
                    let _ = writeln!(f, "{}", std::process::id());
                    return Ok(Lock(path));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // a run that was killed leaves its lock behind: when the pid inside is gone, take the lock over
                    if Self::owner_is_gone(&path) && std::fs::remove_file(&path).is_ok() {
                        continue;
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(format!(
                            "{} exists: another `roles --llm` is writing this cache (or a crashed run left the file \
                             behind: remove it when no run is active)",
                            path.display()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => return Err(format!("{}: {e}", path.display())),
            }
        }
    }
}
impl Lock {
    /// The process named in the lock file no longer exists (Linux: no `/proc/<pid>`; elsewhere assume it does).
    fn owner_is_gone(path: &Path) -> bool {
        let Ok(text) = std::fs::read_to_string(path) else { return false };
        let Ok(pid) = text.trim().parse::<u32>() else { return false };
        Path::new("/proc/self").exists() && !Path::new(&format!("/proc/{pid}")).exists()
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// `roles --llm`: classify the chosen cards through the local server and extend the cache. Reads the model id first, so
/// an unreachable server stops the run before any file is touched. `rules` gives the rule map and the Oracle ids the
/// hand overrides fix (those cards are never sent).
///
/// Ladder, per batch of cards: one request; after a transport error (no answer) the identical request once more;
/// every card the answer did not cover validly, or all of them after an error status or an unusable document, is then
/// asked once on its own (a card already asked alone is not asked again: the request would be identical). A card
/// that still has no valid answer is `llm_failed`. A guard miss is not a fault (see [`validate_output`]).
pub fn run(
    index: &Index,
    taxonomy: &Taxonomy,
    rules: &dyn Fn() -> Result<(RoleMap, BTreeSet<String>), String>,
    dir: &Path,
    opts: &LlmOptions,
) -> Result<String, String> {
    if opts.batch == 0 || opts.batch > 260 {
        return Err("--batch must be between 1 and 260".into());
    }
    if opts.model.is_some() && !opts.dry_run {
        return Err(
            "--model only names the model for a --dry-run; a real run asks the server which model is loaded".into()
        );
    }
    let client = Client::new(&opts.endpoint);
    let model = match &opts.model {
        Some(m) => Some(model_key(m)).filter(|m| !m.trim().is_empty()).ok_or("--model is empty")?,
        None => client.model()?,
    };
    let (rules, fixed) = rules()?;
    let variant = Variant::from_env()?;
    let prompt = prompt_fnv_opts(taxonomy, variant, opts.no_think);
    let system = system_prompt_v(taxonomy, variant);
    let tax_hash = taxonomy.hash();
    let known = load_cache_with(dir, opts.pending)?;
    let wanted = opts.ids.as_deref().map(|p| read_ids(index, p)).transpose()?;
    let mut stats = RunStats {
        model: model.clone(),
        no_think: opts.no_think,
        variant: variant.name().to_string(),
        ..RunStats::default()
    };
    struct Todo<'a> {
        card: &'a Card,
        key: String,
        lines: Vec<(usize, String)>,
        headers: BTreeMap<usize, String>,
    }
    let mut todo: Vec<Todo> = vec![];
    let mut order: Vec<&Card> = index
        .cards
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            roles::is_deck_card(c)
                && (opts.include_not_legal || c.legal == Legal::Legal)
                && wanted.as_ref().is_none_or(|w| w.contains(&(*i as u32)))
                && !fixed.contains(&c.raw.oracle_id)
                && rules.get(&c.raw.oracle_id).is_some_and(|a| {
                    (!opts.only_routed || a.needs_llm)
                        && (opts.routes.is_empty() || a.route.iter().any(|r| opts.routes.contains(r)))
                })
        })
        .map(|(_, c)| c)
        .collect();
    order.sort_by(|a, b| a.raw.oracle_id.cmp(&b.raw.oracle_id));
    for card in order {
        let lines = roles::prompt_lines(card);
        if lines.is_empty() {
            continue; // nothing to read or cite; the merge treats it as an empty answer
        }
        stats.candidates += 1;
        let headers = roles::prompt_headers(card);
        let key = cache_key(&text_of(&card.raw.type_line, &lines, &headers), &prompt, &model);
        if known.contains_key(&key) {
            stats.cached += 1;
        } else {
            todo.push(Todo { card, key, lines, headers });
        }
    }
    // Several catalog cards can share one normalised text (and so one key): ask once.
    let mut seen = BTreeSet::new();
    todo.retain(|t| seen.insert(t.key.clone()));
    if let Some(n) = opts.limit {
        todo.truncate(n);
    }
    let batches = todo.len().div_ceil(opts.batch);
    if opts.dry_run {
        return Ok(format!(
            "DRY RUN: model {model} (prompt variant {}{}); {} candidate cards selected, {} cached, {} to classify in \
             {batches} batches of up to {}; system prompt and example about {} tokens; 0 chat requests made\n",
            variant.name(),
            if opts.no_think { ", no thinking" } else { "" },
            stats.candidates,
            stats.cached,
            todo.len(),
            opts.batch,
            estimate_tokens(&preface_text_v(taxonomy, variant)),
        ));
    }
    let target = if opts.pending { pending_path(dir) } else { cache_path(dir) };
    // Nothing is classified until the target can be read and written: the lock file is created beside it.
    let mut rows_now = read_rows(&target)?.len();
    let _lock = Lock::acquire(target.with_extension("lock"))?;
    let mut new_rows: BTreeMap<String, Row> = BTreeMap::new();
    let mut failed: Vec<String> = vec![];
    let mut aborted: Option<String> = None;
    let mut empty_batches = 0usize;

    let transport = std::cell::Cell::new(0usize);
    let attempt = |items: &[&Todo], stats: &mut RunStats| -> Result<Vec<Result<LlmCard, String>>, Fail> {
        let ids: Vec<String> = (0..items.len()).map(batch_id).collect();
        let shown: Vec<Shown> = items
            .iter()
            .zip(&ids)
            .map(|(t, id)| Shown {
                id: id.clone(),
                type_line: t.card.raw.type_line.as_str(),
                lines: t.lines.as_slice(),
                headers: &t.headers,
            })
            .collect();
        let checked: Vec<(String, &[(usize, String)])> =
            items.iter().zip(&ids).map(|(t, id)| (id.clone(), t.lines.as_slice())).collect();
        let body = chat_body_opts(
            &model,
            &system,
            &user_message_cards(&shown),
            &response_schema_text_v(taxonomy, items.len(), variant),
            items.len(),
            variant,
            opts.no_think,
        );
        stats.chat_requests += 1;
        let reply = client.chat(&body);
        // Any answer, valid or not, shows the server is up; only consecutive transport failures stop the run.
        match &reply {
            Err(Fail::Transport(_)) => transport.set(transport.get() + 1),
            _ => transport.set(0),
        }
        validate_output_v(&reply?, &checked, taxonomy, variant).map_err(Fail::Invalid)
    };
    let down = || transport.get() >= MAX_TRANSPORT_FAILURES;

    'batches: for (batch_no, chunk) in todo.chunks(opts.batch).enumerate() {
        let batch_started = std::time::Instant::now();
        let items: Vec<&Todo> = chunk.iter().collect();
        let mut results: Vec<Option<Result<LlmCard, String>>> = vec![None; items.len()];
        let mut again: Vec<(usize, String)> = vec![];
        let mut first = attempt(&items, &mut stats);
        if matches!(first, Err(Fail::Transport(_))) && !down() {
            stats.batch_retries += 1;
            first = attempt(&items, &mut stats);
        }
        let mut last_why = String::new();
        match first {
            Ok(per_card) => {
                for (i, r) in per_card.into_iter().enumerate() {
                    match r {
                        Ok(c) => results[i] = Some(Ok(c)),
                        Err(why) => again.push((i, why)),
                    }
                }
            }
            Err(f) if down() => aborted = Some(format!("LLM unreachable mid-run: {}", f.why())),
            Err(f) => {
                last_why = f.why();
                again.extend((0..items.len()).map(|i| (i, f.why())));
            }
        }
        for (i, why) in again {
            if aborted.is_some() {
                break;
            }
            if items.len() == 1 {
                results[i] = Some(Err(why)); // asked alone already: the same request would give the same answer
                continue;
            }
            stats.card_retries += 1;
            let one = [items[i]];
            let mut r = attempt(&one, &mut stats);
            if matches!(r, Err(Fail::Transport(_))) && !down() {
                r = attempt(&one, &mut stats);
            }
            match r {
                Ok(mut v) => results[i] = v.pop(),
                Err(f) if down() => aborted = Some(format!("LLM unreachable mid-run: {}", f.why())),
                Err(f) => results[i] = Some(Err(f.why())),
            }
        }
        // The cards answered so far are kept even when the run is stopping.
        let mut answered = 0usize;
        for (item, result) in items.iter().zip(results) {
            match result {
                Some(Ok(c)) => {
                    answered += 1;
                    stats.classified += 1;
                    new_rows.insert(
                        item.key.clone(),
                        Row {
                            key: item.key.clone(),
                            name: item.card.raw.name.clone(),
                            roles: c.roles,
                            primary: c.primary,
                            evidence: c.evidence,
                            model: model.clone(),
                            taxonomy: tax_hash.clone(),
                            prompt: prompt.clone(),
                            why: c.why,
                        },
                    );
                }
                Some(Err(why)) => {
                    stats.failed += 1;
                    failed.push(format!("{}: {why}", card_label(item.card)));
                    last_why = why;
                }
                None => {}
            }
        }
        if answered > 0 {
            // re-read before writing: another process may have added rows since this run started
            let mut current = read_rows(&target)?;
            current.extend(new_rows.clone());
            write_atomic(&target, &rows_bytes(&current))?;
            rows_now = current.len();
        }
        eprintln!(
            "batch {}/{batches}: {} cards, {:.1} s, cache rows {rows_now}",
            batch_no + 1,
            chunk.len(),
            batch_started.elapsed().as_secs_f64()
        );
        empty_batches = if answered == 0 { empty_batches + 1 } else { 0 };
        if aborted.is_none() && empty_batches >= MAX_EMPTY_BATCHES {
            aborted = Some(format!("no valid answer for {MAX_EMPTY_BATCHES} batches in a row (last: {last_why})"));
        }
        if aborted.is_some() {
            break 'batches;
        }
    }
    let reports = reports_dir(dir);
    let text = |s: &RunStats| format!("{}\n", serde_json::to_string_pretty(s).unwrap());
    write_atomic(&reports.join("llm-run-last.json"), text(&stats).as_bytes())?;
    if stats.chat_requests > 0 {
        // a warm rerun must not overwrite the report of the last real run
        write_atomic(&reports.join("llm-run.json"), text(&stats).as_bytes())?;
        let mut failed_text = String::new();
        for f in &failed {
            let _ = writeln!(failed_text, "{f}");
        }
        write_atomic(&reports.join("llm-failed.txt"), failed_text.as_bytes())?;
    }
    if let Some(e) = aborted {
        return Err(format!(
            "{e}; saved {} classified cards to {}, rerun to continue",
            stats.classified,
            target.display()
        ));
    }
    let summary = format!(
        "LLM stage: model {model} (prompt variant {}); {} candidate cards, {} cached, {} classified, {} failed \
         ({} batch retries, {} per-card retries); {} chat requests (plus 1 model query)\nwrote {}\n",
        variant.name(),
        stats.candidates,
        stats.cached,
        stats.classified,
        stats.failed,
        stats.batch_retries,
        stats.card_retries,
        stats.chat_requests,
        target.display()
    );
    let asked = stats.classified + stats.failed;
    if asked > 0 && stats.failed as f64 > MAX_FAILED_SHARE * asked as f64 {
        return Err(format!(
            "{} of {asked} cards ended llm_failed, more than {:.0}%: {}see reports/llm-failed.txt",
            stats.failed,
            MAX_FAILED_SHARE * 100.0,
            summary.replace('\n', "; ")
        ));
    }
    Ok(summary)
}

// ------------------------------------------------------------------------------------------------ merging a table

/// The model the cache holds answers from for this prompt: the one `--model` names (it must have rows), else the only
/// one present. A cache with rows but none under this prompt is an error; only an empty cache merges with no model.
fn merge_model(rows: &BTreeMap<String, Row>, prompt: &str, wanted: Option<&str>) -> Result<String, String> {
    let models: BTreeSet<&str> = rows.values().filter(|r| r.prompt == prompt).map(|r| r.model.as_str()).collect();
    if let Some(m) = wanted {
        let m = model_key(m);
        if !models.contains(m.as_str()) {
            return Err(format!(
                "no cached answers from model '{m}' for the current prompt (the cache has: {})",
                if models.is_empty() { "none".to_string() } else { models.into_iter().collect::<Vec<_>>().join(", ") }
            ));
        }
        return Ok(m);
    }
    match models.len() {
        0 if rows.is_empty() => Ok(String::new()),
        0 => {
            // rows exist, but none under this prompt: a merge would silently answer nothing (a rules-only table)
            let held: BTreeSet<&str> = rows.values().map(|r| r.model.as_str()).collect();
            Err(format!(
                "the cache holds {} rows (models: {}) but none for the current prompt, so the merge would use no LLM \
                 answer at all; pass --model <id> with the same --no-think flag and CARDGEN_LLM_VARIANT the run used \
                 (an empty --model is never a silent rules-only merge), or run `roles --rules` for a rules-only table",
                rows.len(),
                held.into_iter().collect::<Vec<_>>().join(", ")
            ))
        }
        1 => Ok(models.into_iter().next().unwrap().to_string()),
        _ => Err(format!(
            "the cache holds answers from several models for this prompt ({}); pass --model ID",
            models.into_iter().collect::<Vec<_>>().join(", ")
        )),
    }
}

/// Rule roles + cache + overrides through [`merge`]: the merged table and the disagreement report text.
pub fn merge_table(
    index: &Index,
    taxonomy: &Taxonomy,
    rules: &RoleMap,
    overrides: &BTreeMap<String, Assignment>,
    cache: &BTreeMap<String, Row>,
    model: Option<&str>,
    failed: &BTreeSet<String>,
    no_think: bool,
) -> Result<(RoleMap, String, MergeStats), String> {
    let prompt = prompt_fnv_opts(taxonomy, Variant::current(), no_think);
    let model = merge_model(cache, &prompt, model)?;
    let keep = |r: &str| taxonomy.keep_rule(r);
    let mut meta: RoleMeta = rules.meta.clone();
    meta.prompt_fnv = prompt.clone();
    meta.model = model.clone();
    let mut merged = RoleMap { meta, cards: BTreeMap::new() };
    let mut all: Vec<Disagreement> = vec![];
    let mut stats = MergeStats::default();
    let inert = taxonomy.llm_inert();
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let id = &c.raw.oracle_id;
        let lines = roles::prompt_lines(c);
        let check = |role: &str, line: usize| -> Option<Option<usize>> {
            let (_, text) = lines.iter().find(|(i, _)| *i == line)?;
            // a role switched off with `llm_add = false` is refused here like a failed guard
            (taxonomy.llm_add(role) && taxonomy.guard(role)?.is_match(text).unwrap_or(false)).then(|| c.lines()[line].0)
        };
        let env = MergeEnv { keep: &keep, check: &check };
        let rule = &rules[id];
        let llm = if lines.is_empty() {
            Some(LlmCard::default())
        } else {
            cache.get(&cache_key(&card_text(c), &prompt, &model)).map(Row::card)
        };
        let legal = c.legal == Legal::Legal;
        if rule.needs_llm && !overrides.contains_key(id) {
            if !legal {
                stats.not_legal += usize::from(llm.is_none());
            } else {
                stats.routed += 1;
                stats.with_answer += usize::from(llm.is_some());
                stats.llm_failed += usize::from(llm.is_none());
                stats.failed += usize::from(llm.is_none() && failed.contains(id));
                stats.not_run += usize::from(llm.is_none() && !failed.contains(id));
            }
        }
        if rule.needs_llm && !overrides.contains_key(id) {
            for r in llm.iter().flat_map(|l| &l.roles).filter(|r| !taxonomy.llm_add(r)) {
                *stats.llm_add_dropped.entry(r.clone()).or_default() += 1;
            }
        }
        let (mut a, mut dis) = merge(rule, llm.as_ref(), overrides.get(id), &env);
        if inert && !overrides.contains_key(id) {
            // nothing from the LLM is used: the rules' assignment exactly (order, primary), and a routed card stays
            // unreviewed
            a = rule.clone();
            a.llm_failed = rule.needs_llm;
        }
        // only Commander-legal cards are ever sent, so only they can be waiting for an answer
        a.llm_failed &= legal;
        for d in &mut dis {
            d.card = c.raw.name.clone();
        }
        all.extend(dis);
        merged.cards.insert(id.clone(), a);
    }
    for d in &all {
        *stats.disagreements.entry(route_label(&d.route)).or_default().entry(format!("{:?}", d.kind)).or_default() += 1;
    }
    Ok((merged, disagreement_report(&all, &stats), stats))
}

/// The roles the cached answer for `card` proposes that pass their guard on the cited line, with no rules and no
/// merge (the LLM alone); `None` when nothing is cached for this prompt and model.
pub fn alone_roles(
    card: &Card,
    taxonomy: &Taxonomy,
    cache: &BTreeMap<String, Row>,
    prompt: &str,
    model: &str,
) -> Option<Vec<String>> {
    let lines = roles::prompt_lines(card);
    if lines.is_empty() {
        return Some(vec![]);
    }
    let row = cache.get(&cache_key(&card_text(card), prompt, model))?;
    Some(
        row.roles
            .iter()
            .filter(|r| {
                taxonomy.llm_add(r)
                    && row.evidence.get(*r).is_some_and(|n| {
                        lines.iter().find(|(i, _)| i == n).is_some_and(|(_, text)| {
                            taxonomy.guard(r).is_some_and(|g| g.is_match(text).unwrap_or(false))
                        })
                    })
            })
            .cloned()
            .collect(),
    )
}

fn route_label(route: &[RouteReason]) -> String {
    if route.is_empty() {
        "RulesFinal".into()
    } else {
        route.iter().map(|r| format!("{r:?}")).collect::<Vec<_>>().join("+")
    }
}

/// The text report: counts by route reason and kind, then every disagreement with both role sets.
pub fn disagreement_report(all: &[Disagreement], stats: &MergeStats) -> String {
    let mut out = String::from("ROLE DISAGREEMENTS (rules versus the local LLM, one line each)\n");
    let _ = writeln!(
        out,
        "routed legal cards {}: {} with an LLM answer, {} without (rule roles kept, llm_failed set: {} failed, {} not run \
         yet); {} routed cards are not legal (never sent)",
        stats.routed, stats.with_answer, stats.llm_failed, stats.failed, stats.not_run, stats.not_legal
    );
    for (r, n) in &stats.llm_add_dropped {
        let _ = writeln!(out, "LLM proposals dropped because llm_add = false for {r}: {n}");
    }
    let _ = writeln!(out, "\nBY ROUTE REASON");
    for (route, kinds) in &stats.disagreements {
        let _ = writeln!(
            out,
            "  {route}: {}",
            kinds.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")
        );
    }
    let mut sorted: Vec<&Disagreement> = all.iter().collect();
    sorted.sort_by(|a, b| {
        (route_label(&a.route), &a.card, a.kind, &a.role).cmp(&(route_label(&b.route), &b.card, b.kind, &b.role))
    });
    let _ = writeln!(out, "\nDETAILS (route | kind | card | role | rules | llm | merged)");
    for d in sorted {
        let _ = writeln!(
            out,
            "{} | {:?} | {} | {} | [{}] | [{}] | [{}]",
            route_label(&d.route),
            d.kind,
            d.card,
            d.role,
            d.rule.join(", "),
            d.llm.join(", "),
            d.merged.join(", ")
        );
    }
    out
}

/// The Oracle ids the last run could not get a valid answer for (`reports/llm-failed.txt`, best effort).
fn failed_ids(dir: &Path) -> BTreeSet<String> {
    let text = std::fs::read_to_string(reports_dir(dir).join("llm-failed.txt")).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let open = l.find(" (")? + 2;
            Some(l[open..open + l[open..].find(')')?].to_string())
        })
        .collect()
}

/// `roles --merge`: write the merged table and the reports beside the catalog.
pub fn run_merge(
    index: &Index,
    taxonomy: &Taxonomy,
    rules: &RoleMap,
    overrides: &BTreeMap<String, Assignment>,
    dir: &Path,
    model: Option<&str>,
    pending: bool,
    no_think: bool,
) -> Result<String, String> {
    let variant = Variant::from_env()?;
    let cache = load_cache_with(dir, pending)?;
    let (merged, report, stats) =
        merge_table(index, taxonomy, rules, overrides, &cache, model, &failed_ids(dir), no_think)?;
    write_atomic(&dir.join("roles.json"), &roles::bytes(&merged)?)?;
    let reports = reports_dir(dir);
    write_atomic(&reports.join("roles-disagreements.txt"), report.as_bytes())?;
    write_atomic(
        &reports.join("merge.json"),
        format!("{}\n", serde_json::to_string_pretty(&stats).unwrap()).as_bytes(),
    )?;
    let total: usize = stats.disagreements.values().flat_map(|k| k.values()).sum();
    let mut dropped = String::new();
    for (r, n) in &stats.llm_add_dropped {
        let _ = write!(dropped, "\nllm_add = false for {r}: {n} LLM proposals dropped");
    }
    Ok(format!(
        "merged {} cards (model '{}', prompt variant {}): {} routed legal cards, {} with an LLM answer, {} without ({} failed, {} not run); \
         {} routed cards not legal; {total} disagreements (reports/roles-disagreements.txt){dropped}\nwrote {}\n",
        merged.len(),
        merged.meta.model,
        variant.name(),
        stats.routed,
        stats.with_answer,
        stats.llm_failed,
        stats.failed,
        stats.not_run,
        stats.not_legal,
        dir.join("roles.json").display()
    ))
}
