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
/// Consecutive transport failures after which a run saves what it has and stops.
const MAX_TRANSPORT_FAILURES: usize = 3;
const FLUSH_EVERY: usize = 25;

// ---------------------------------------------------------------------------------------------------- the prompt

const INTRO: &str = concat!(
    "You label Magic: The Gathering cards with deck-building roles. You see only a card's type line and its ",
    "numbered Oracle lines (the card's own name is written ~, reminder text is removed). You know nothing else ",
    "about the card and must not guess from its name or from deck context."
);
const ANSWER_RULES: &str = concat!(
    "HOW TO ANSWER\n",
    "Reply with one JSON object {\"cards\":[...]}: exactly one entry per card, each {\"id\",\"roles\",\"primary\",",
    "\"evidence\"} with the card's id. roles: at most 4 role ids, most prominent first. primary: the first role, or ",
    "null when roles is empty. evidence: for every role, the number of the line that supports it (the number before ",
    "the line text); cite the line where that effect is written. If no role applies return roles [], primary null and ",
    "evidence {}. Count main or repeatable effects only: riders, costs, drawbacks, conditions that merely mention an ",
    "effect and keywords that affect only the card itself give no role. Most plain bodies and minor-rider cards have ",
    "no role."
);

/// One worked example: a card as the model sees it and the correct answer. None of these cards is in a gold set.
pub struct Example {
    pub name: &'static str,
    pub type_line: &'static str,
    pub lines: &'static [(usize, &'static str)],
    pub roles: &'static [&'static str],
    pub evidence: &'static [(&'static str, usize)],
}
/// A rider on a removal spell, a cost-side counter, a "would draw" replacement, a permanent steal, a one-turn Fog,
/// and a body with no roles.
pub const EXAMPLES: [Example; 6] = [
    Example {
        name: "Chastise",
        type_line: "Instant",
        lines: &[(0, "Destroy target attacking creature. You gain life equal to its power.")],
        roles: &["removal"],
        evidence: &[("removal", 0)],
    },
    Example {
        name: "Ancient Hydra",
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
        type_line: "Creature — Lizard",
        lines: &[(0, "If you would draw a card, you may skip that draw instead.")],
        roles: &[],
        evidence: &[],
    },
    Example {
        name: "Mind Control",
        type_line: "Enchantment — Aura",
        lines: &[(0, "Enchant creature"), (1, "You control enchanted creature.")],
        roles: &["removal"],
        evidence: &[("removal", 1)],
    },
    Example {
        name: "Holy Day",
        type_line: "Instant",
        lines: &[(0, "Prevent all combat damage that would be dealt this turn.")],
        roles: &["protection"],
        evidence: &[("protection", 0)],
    },
    Example {
        name: "Kird Ape",
        type_line: "Creature — Ape",
        lines: &[(0, "This creature gets +1/+2 as long as you control a Forest.")],
        roles: &[],
        evidence: &[],
    },
];

/// Batch id of the `i`th card of a batch: a letter and a digit (`a0` ... `a9`, `b0`, ...).
pub fn batch_id(i: usize) -> String {
    format!("{}{}", (b'a' + (i / 10) as u8) as char, i % 10)
}

/// The cards of a batch as the user message shows them.
pub fn user_message(cards: &[(String, &str, &[(usize, String)])]) -> String {
    let mut out = format!("Classify these {} cards.\n", cards.len());
    for (id, type_line, lines) in cards {
        let _ = writeln!(out, "[{id}] {type_line}");
        for (i, text) in lines.iter() {
            let _ = writeln!(out, "{i}: {text}");
        }
    }
    out
}

/// A JSON string literal.
fn quoted(s: &str) -> String {
    serde_json::to_string(s).expect("string")
}
/// An answer entry written in the schema's property order (id, roles, primary, evidence), the order the grammar
/// makes the model write it in: the roles are decided before the lines that support them are cited.
pub fn answer_entry(id: &str, roles: &[&str], evidence: &[(&str, usize)]) -> String {
    let list = roles.iter().map(|r| quoted(r)).collect::<Vec<_>>().join(",");
    let primary = roles.first().map_or("null".to_string(), |r| quoted(r));
    let cites = evidence.iter().map(|(r, n)| format!("{}:{n}", quoted(r))).collect::<Vec<_>>().join(",");
    format!("{{\"id\":{},\"roles\":[{list}],\"primary\":{primary},\"evidence\":{{{cites}}}}}", quoted(id))
}

fn examples_text() -> String {
    let lines: Vec<Vec<(usize, String)>> =
        EXAMPLES.iter().map(|e| e.lines.iter().map(|(i, t)| (*i, t.to_string())).collect()).collect();
    let cards: Vec<(String, &str, &[(usize, String)])> =
        EXAMPLES.iter().zip(&lines).enumerate().map(|(i, (e, l))| (batch_id(i), e.type_line, l.as_slice())).collect();
    let entries: Vec<String> =
        EXAMPLES.iter().enumerate().map(|(i, e)| answer_entry(&batch_id(i), e.roles, e.evidence)).collect();
    format!("WORKED EXAMPLE\nUser:\n{}Answer:\n{{\"cards\":[{}]}}", user_message(&cards), entries.join(","))
}

/// The whole system prompt: the taxonomy, the fixed answer rules and the worked examples.
pub fn system_prompt(t: &Taxonomy) -> String {
    format!("{INTRO}\n\n{}\n\n{ANSWER_RULES}\n\n{}\n", t.prompt_text(), examples_text())
}
/// The prompt fingerprint that, with the model id and the card text, keys the cache: any change to what the model is
/// told (definitions, clauses, answer rules, examples) invalidates; guards and `keep_rule` are not part of it.
pub fn prompt_fnv(t: &Taxonomy) -> String {
    roles::fingerprint(system_prompt(t).as_bytes())
}
/// Rough token count used for budgeting: characters / 4.
pub fn estimate_tokens(s: &str) -> usize {
    s.chars().count().div_ceil(4)
}

/// The JSON schema (llama.cpp grammar-constrains the output to it) for a batch of `n` cards, as text so that the
/// property order is the one written here: `id`, `roles`, `primary`, `evidence`.
pub fn response_schema_text(t: &Taxonomy, n: usize) -> String {
    let ids: Vec<String> = t.role.iter().map(|r| quoted(&r.id)).collect();
    let names = ids.join(",");
    let evidence = ids.iter().map(|i| format!("{i}:{{\"type\":\"integer\"}}")).collect::<Vec<_>>().join(",");
    format!(
        concat!(
            "{{\"type\":\"object\",\"properties\":{{\"cards\":{{\"type\":\"array\",\"minItems\":{n},\"maxItems\":{n},",
            "\"items\":{{\"type\":\"object\",\"properties\":{{",
            "\"id\":{{\"type\":\"string\",\"pattern\":\"^[a-z][0-9]$\"}},",
            "\"roles\":{{\"type\":\"array\",\"maxItems\":{max},\"items\":{{\"type\":\"string\",\"enum\":[{names}]}}}},",
            "\"primary\":{{\"enum\":[{names},null]}},",
            "\"evidence\":{{\"type\":\"object\",\"properties\":{{{evidence}}},\"additionalProperties\":false}}}},",
            "\"required\":[\"id\",\"roles\",\"primary\",\"evidence\"],\"additionalProperties\":false}}}}}},",
            "\"required\":[\"cards\"],\"additionalProperties\":false}}"
        ),
        n = n,
        max = MAX_ROLES,
        names = names,
        evidence = evidence,
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
}

/// Check a model reply against the batch it answers. `cards` are the batch ids with the numbered lines shown (the only
/// citable ones). Any deviation is an error: wrong id set, unknown role, more than four roles, a primary outside the
/// roles, a missing, out-of-range or skipped citation, or a role whose guard fails on its cited line.
pub fn validate_output(
    content: &str,
    cards: &[(String, &[(usize, String)])],
    taxonomy: &Taxonomy,
) -> Result<Vec<LlmCard>, String> {
    let doc: Value = serde_json::from_str(content).map_err(|e| format!("not JSON: {e}"))?;
    let entries = doc.get("cards").and_then(Value::as_array).ok_or("no cards array")?;
    if entries.len() != cards.len() {
        return Err(format!("{} entries for {} cards", entries.len(), cards.len()));
    }
    let mut by_id: BTreeMap<&str, &Value> = BTreeMap::new();
    for e in entries {
        let id = e.get("id").and_then(Value::as_str).ok_or("entry without an id")?;
        if by_id.insert(id, e).is_some() {
            return Err(format!("repeated id {id}"));
        }
    }
    let mut out = Vec::with_capacity(cards.len());
    for (id, lines) in cards {
        let e = by_id.remove(id.as_str()).ok_or_else(|| format!("missing id {id}"))?;
        let bad = |what: &str| format!("{id}: {what}");
        let roles: Vec<String> = e
            .get("roles")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("roles is not an array"))?
            .iter()
            .map(|r| r.as_str().map(str::to_string).ok_or_else(|| bad("a role is not a string")))
            .collect::<Result<_, _>>()?;
        if roles.len() > MAX_ROLES {
            return Err(bad("more than 4 roles"));
        }
        if roles.iter().collect::<BTreeSet<_>>().len() != roles.len() {
            return Err(bad("repeated role"));
        }
        if let Some(r) = roles.iter().find(|r| !roles::IDS.contains(&r.as_str())) {
            return Err(bad(&format!("role {r} is not in the vocabulary")));
        }
        let primary = match e.get("primary") {
            Some(Value::Null) => None,
            Some(Value::String(p)) => Some(p.clone()),
            _ => return Err(bad("primary is neither a role nor null")),
        };
        match (&primary, roles.is_empty()) {
            (None, true) => {}
            (Some(p), false) if roles.contains(p) => {}
            _ => return Err(bad("primary must be one of the roles, or null when there are none")),
        }
        let ev = e.get("evidence").and_then(Value::as_object).ok_or_else(|| bad("evidence is not an object"))?;
        if let Some(k) = ev.keys().find(|k| !roles.contains(k)) {
            return Err(bad(&format!("evidence for {k}, which is not a proposed role")));
        }
        let mut evidence = BTreeMap::new();
        for role in &roles {
            let n =
                ev.get(role).and_then(Value::as_u64).ok_or_else(|| bad(&format!("no line number cited for {role}")))?
                    as usize;
            let (_, text) = lines
                .iter()
                .find(|(i, _)| *i == n)
                .ok_or_else(|| bad(&format!("{role} cites line {n}, which is out of range or skipped")))?;
            if !taxonomy.guard(role).is_match(text).unwrap_or(false) {
                return Err(bad(&format!("{role} fails its guard on line {n}")));
            }
            evidence.insert(role.clone(), n);
        }
        out.push(LlmCard { roles, primary, evidence });
    }
    Ok(out)
}

// ------------------------------------------------------------------------------------------------------- the cache

/// What the model is asked about a card: the type line and the numbered lines, exactly as shown to it.
pub fn card_text(card: &Card) -> String {
    text_of(&card.raw.type_line, &roles::prompt_lines(card))
}
fn text_of(type_line: &str, lines: &[(usize, String)]) -> String {
    let mut s = type_line.to_string();
    for (i, text) in lines {
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
}
impl Row {
    pub fn card(&self) -> LlmCard {
        LlmCard { roles: self.roles.clone(), primary: self.primary.clone(), evidence: self.evidence.clone() }
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

/// The tracked cache with the scheduled job's pending rows read after it (a pending row wins on the same key).
pub fn load_cache(dir: &Path) -> Result<BTreeMap<String, Row>, String> {
    let mut rows = read_rows(&cache_path(dir))?;
    rows.extend(read_rows(&pending_path(dir))?);
    Ok(rows)
}

// ------------------------------------------------------------------------------------------------------ the client

enum Fail {
    /// the server could not be reached or answered with an error status
    Transport(String),
    /// the answer arrived but is not a valid classification
    Invalid(String),
}
impl Fail {
    fn why(&self) -> String {
        match self {
            Fail::Transport(e) => format!("transport: {e}"),
            Fail::Invalid(e) => format!("invalid answer: {e}"),
        }
    }
}

pub struct Client {
    agent: ureq::Agent,
    base: String,
}
impl Client {
    pub fn new(endpoint: &str) -> Self {
        let agent =
            ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout(Duration::from_secs(900)).build();
        Self { agent, base: endpoint.trim_end_matches('/').to_string() }
    }
    /// The id of the loaded model, from GET /models.
    pub fn model(&self) -> Result<String, String> {
        let body = self
            .agent
            .get(&format!("{}/models", self.base))
            .call()
            .map_err(|e| format!("LLM unreachable: {}: {e}", self.base))?
            .into_string()
            .map_err(|e| format!("LLM unreachable: {}: {e}", self.base))?;
        let v: Value = serde_json::from_str(&body).map_err(|e| format!("LLM unreachable: bad /models reply: {e}"))?;
        v["data"][0]["id"]
            .as_str()
            .or_else(|| v["models"][0]["model"].as_str())
            .map(str::to_string)
            .ok_or_else(|| "LLM unreachable: /models names no model".to_string())
    }
    fn chat(&self, body: &str) -> Result<String, Fail> {
        let resp = self
            .agent
            .post(&format!("{}/chat/completions", self.base))
            .set("Content-Type", "application/json")
            .send_string(body)
            .map_err(|e| Fail::Transport(e.to_string()))?;
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

/// The request body of one chat call (text, so the schema keeps its property order).
pub fn chat_body(model: &str, system: &str, user: &str, schema: &str, n: usize) -> String {
    format!(
        concat!(
            "{{\"model\":{},\"messages\":[{{\"role\":\"system\",\"content\":{}}},",
            "{{\"role\":\"user\",\"content\":{}}}],\"temperature\":0,\"seed\":1,\"cache_prompt\":true,",
            "\"max_tokens\":{},\"response_format\":{{\"type\":\"json_schema\",",
            "\"json_schema\":{{\"name\":\"role_labels\",\"strict\":true,\"schema\":{}}}}}}}"
        ),
        quoted(model),
        quoted(system),
        quoted(user),
        160 * n + 64,
        schema
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
    pub routed: usize,
    pub with_answer: usize,
    pub llm_failed: usize,
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

/// `roles --llm`: classify the chosen cards through the local server and extend the cache. Reads the model id first, so
/// an unreachable server stops the run before any file is touched. `rules` gives the rule map and the Oracle ids the
/// hand overrides fix (those cards are never sent).
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
    let client = Client::new(&opts.endpoint);
    let model = match &opts.model {
        Some(m) if opts.dry_run => m.clone(),
        _ => client.model()?,
    };
    let (rules, fixed) = rules()?;
    let prompt = prompt_fnv(taxonomy);
    let system = system_prompt(taxonomy);
    let tax_hash = taxonomy.hash();
    let known = load_cache(dir)?;
    let wanted = opts.ids.as_deref().map(|p| read_ids(index, p)).transpose()?;
    let mut stats = RunStats { model: model.clone(), ..RunStats::default() };
    struct Todo<'a> {
        card: &'a Card,
        key: String,
        lines: Vec<(usize, String)>,
    }
    let mut todo: Vec<Todo> = vec![];
    let mut order: Vec<&Card> = index
        .cards
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            roles::is_deck_card(c)
                && c.legal == Legal::Legal
                && wanted.as_ref().is_none_or(|w| w.contains(&(*i as u32)))
                && !fixed.contains(&c.raw.oracle_id)
                && rules.get(&c.raw.oracle_id).is_some_and(|a| !opts.only_routed || a.needs_llm)
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
        let key = cache_key(&text_of(&card.raw.type_line, &lines), &prompt, &model);
        if known.contains_key(&key) {
            stats.cached += 1;
        } else {
            todo.push(Todo { card, key, lines });
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
            "DRY RUN: model {model}; {} candidate cards, {} cached, {} to classify in {batches} batches of up to {}; \
             system prompt about {} tokens; 0 chat requests made\n",
            stats.candidates,
            stats.cached,
            todo.len(),
            opts.batch,
            estimate_tokens(&system),
        ));
    }
    let target = if opts.pending { pending_path(dir) } else { cache_path(dir) };
    let mut file_rows = read_rows(&target)?;
    let mut failed: Vec<String> = vec![];
    let mut aborted: Option<String> = None;
    let mut done_batches = 0usize;
    let mut dirty = false;

    let transport = std::cell::Cell::new(0usize);
    let attempt = |items: &[&Todo], stats: &mut RunStats| -> Result<Vec<LlmCard>, Fail> {
        let ids: Vec<String> = (0..items.len()).map(batch_id).collect();
        let shown: Vec<(String, &str, &[(usize, String)])> = items
            .iter()
            .zip(&ids)
            .map(|(t, id)| (id.clone(), t.card.raw.type_line.as_str(), t.lines.as_slice()))
            .collect();
        let checked: Vec<(String, &[(usize, String)])> =
            items.iter().zip(&ids).map(|(t, id)| (id.clone(), t.lines.as_slice())).collect();
        let body = chat_body(
            &model,
            &system,
            &user_message(&shown),
            &response_schema_text(taxonomy, items.len()),
            items.len(),
        );
        stats.chat_requests += 1;
        let reply = client.chat(&body);
        // Any answer, valid or not, shows the server is up; only consecutive transport failures stop the run.
        match &reply {
            Err(Fail::Transport(_)) => transport.set(transport.get() + 1),
            _ => transport.set(0),
        }
        validate_output(&reply?, &checked, taxonomy).map_err(Fail::Invalid)
    };
    let down = || transport.get() >= MAX_TRANSPORT_FAILURES;

    'batches: for chunk in todo.chunks(opts.batch) {
        let items: Vec<&Todo> = chunk.iter().collect();
        let mut results: Vec<Result<LlmCard, String>> = vec![Err("not answered".into()); items.len()];
        let mut outcome = attempt(&items, &mut stats);
        if outcome.is_err() && !down() {
            stats.batch_retries += 1;
            outcome = attempt(&items, &mut stats);
        }
        match outcome {
            Ok(cards) => results = cards.into_iter().map(Ok).collect(),
            Err(first) => {
                if down() {
                    aborted = Some(format!("LLM unreachable mid-run: {}", first.why()));
                    break 'batches;
                }
                for (i, item) in items.iter().enumerate() {
                    let one = [*item];
                    let mut r = attempt(&one, &mut stats);
                    if r.is_err() && !down() {
                        stats.card_retries += 1;
                        r = attempt(&one, &mut stats);
                    }
                    match r {
                        Ok(mut c) => results[i] = Ok(c.pop().expect("one card asked, one answered")),
                        Err(e) if down() => {
                            aborted = Some(format!("LLM unreachable mid-run: {}", e.why()));
                            break 'batches;
                        }
                        Err(e) => results[i] = Err(e.why()),
                    }
                }
            }
        }
        for (item, result) in items.iter().zip(results) {
            match result {
                Ok(c) => {
                    stats.classified += 1;
                    file_rows.insert(
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
                        },
                    );
                    dirty = true;
                }
                Err(why) => {
                    stats.failed += 1;
                    failed.push(format!("{}: {why}", card_label(item.card)));
                }
            }
        }
        done_batches += 1;
        if dirty && done_batches % FLUSH_EVERY == 0 {
            write_atomic(&target, &rows_bytes(&file_rows))?;
            dirty = false;
        }
    }
    if dirty {
        write_atomic(&target, &rows_bytes(&file_rows))?;
    }
    let reports = reports_dir(dir);
    write_atomic(
        &reports.join("llm-run.json"),
        format!("{}\n", serde_json::to_string_pretty(&stats).unwrap()).as_bytes(),
    )?;
    let mut failed_text = String::new();
    for f in &failed {
        let _ = writeln!(failed_text, "{f}");
    }
    write_atomic(&reports.join("llm-failed.txt"), failed_text.as_bytes())?;
    if let Some(e) = aborted {
        return Err(format!(
            "{e}; saved {} classified cards to {}, rerun to continue",
            stats.classified,
            target.display()
        ));
    }
    Ok(format!(
        "LLM stage: model {model}; {} candidate cards, {} cached, {} classified, {} failed \
         ({} batch retries, {} per-card retries); {} chat requests (plus 1 model query)\nwrote {}\n",
        stats.candidates,
        stats.cached,
        stats.classified,
        stats.failed,
        stats.batch_retries,
        stats.card_retries,
        stats.chat_requests,
        target.display()
    ))
}

// ------------------------------------------------------------------------------------------------ merging a table

/// The model the cache holds answers from for this prompt: the one `--model` names, else the only one present.
fn merge_model(rows: &BTreeMap<String, Row>, prompt: &str, wanted: Option<&str>) -> Result<String, String> {
    if let Some(m) = wanted {
        return Ok(m.to_string());
    }
    let models: BTreeSet<&str> = rows.values().filter(|r| r.prompt == prompt).map(|r| r.model.as_str()).collect();
    match models.len() {
        0 => Ok(String::new()),
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
) -> Result<(RoleMap, String, MergeStats), String> {
    let prompt = prompt_fnv(taxonomy);
    let model = merge_model(cache, &prompt, model)?;
    let keep = |r: &str| taxonomy.keep_rule(r);
    let mut meta: RoleMeta = rules.meta.clone();
    meta.prompt_fnv = prompt.clone();
    meta.model = model.clone();
    let mut merged = RoleMap { meta, cards: BTreeMap::new() };
    let mut all: Vec<Disagreement> = vec![];
    let mut stats = MergeStats::default();
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let id = &c.raw.oracle_id;
        let lines = roles::prompt_lines(c);
        let check = |role: &str, line: usize| -> Option<Option<usize>> {
            let (_, text) = lines.iter().find(|(i, _)| *i == line)?;
            taxonomy.guard(role).is_match(text).unwrap_or(false).then(|| c.lines()[line].0)
        };
        let env = MergeEnv { keep: &keep, check: &check };
        let rule = &rules[id];
        let llm = if lines.is_empty() {
            Some(LlmCard::default())
        } else {
            cache.get(&cache_key(&card_text(c), &prompt, &model)).map(Row::card)
        };
        if rule.needs_llm && !overrides.contains_key(id) {
            stats.routed += 1;
            stats.with_answer += usize::from(llm.is_some());
            stats.llm_failed += usize::from(llm.is_none());
        }
        let (a, mut dis) = merge(rule, llm.as_ref(), overrides.get(id), &env);
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
        "routed cards {}: {} with an LLM answer, {} without (rule roles kept, llm_failed set)",
        stats.routed, stats.with_answer, stats.llm_failed
    );
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

/// `roles --merge`: write the merged table and the reports beside the catalog.
pub fn run_merge(
    index: &Index,
    taxonomy: &Taxonomy,
    rules: &RoleMap,
    overrides: &BTreeMap<String, Assignment>,
    dir: &Path,
    model: Option<&str>,
) -> Result<String, String> {
    let cache = load_cache(dir)?;
    let (merged, report, stats) = merge_table(index, taxonomy, rules, overrides, &cache, model)?;
    write_atomic(&dir.join("roles.json"), &roles::bytes(&merged)?)?;
    let reports = reports_dir(dir);
    write_atomic(&reports.join("roles-disagreements.txt"), report.as_bytes())?;
    write_atomic(
        &reports.join("merge.json"),
        format!("{}\n", serde_json::to_string_pretty(&stats).unwrap()).as_bytes(),
    )?;
    let total: usize = stats.disagreements.values().flat_map(|k| k.values()).sum();
    Ok(format!(
        "merged {} cards (model '{}'): {} routed, {} with an LLM answer, {} without; {total} disagreements \
         (reports/roles-disagreements.txt)\nwrote {}\n",
        merged.len(),
        merged.meta.model,
        stats.routed,
        stats.with_answer,
        stats.llm_failed,
        dir.join("roles.json").display()
    ))
}
