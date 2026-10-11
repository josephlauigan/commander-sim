//! What the reports and checks read from a card's exported record (data/cards.json): its description (--cards),
//! colour identity (--swap checks) and whether it's held for a response (--analyze's card report).

use sim_core::export::{RawCard, TagValue};
use std::collections::BTreeSet;

fn tag<'a>(c: &'a RawCard, k: &str) -> Option<&'a TagValue> {
    c.tags.get(k)
}

fn tag_str(v: &TagValue) -> String {
    match v {
        TagValue::Flag(true) => "True".into(),
        TagValue::Flag(false) => "False".into(),
        TagValue::Value(s) => s.clone(),
    }
}

/// compare.print_analysis's `held`: a reactive card (counterspell, protection, Tidebinder-style), only cast in
/// response
pub fn held(c: &RawCard) -> bool {
    let prot = tag(c, "prot").map(tag_str);
    tag(c, "ctr").is_some()
        || prot.is_some_and(|p| matches!(p.as_str(), "hi" | "phase" | "indes" | "blink" | "notw"))
        || tag(c, "tide").is_some()
}

/// cards.sources.identity: the card's colour identity (the recorded one, else its pips and a land's colours)
pub fn identity(c: &RawCard) -> BTreeSet<char> {
    if let Some(id) = &c.identity {
        return id.chars().collect();
    }
    let mut s: BTreeSet<char> = c.pips.chars().collect();
    if c.derived.land
        && let Some(v) = tag(c, "c")
    {
        let v = tag_str(v);
        if v != "A" && v != "C" {
            s.extend(v.chars());
        }
    }
    s.into_iter().filter(|ch| "WUBRG".contains(*ch)).collect()
}

fn json_str(v: Option<&serde_json::Value>) -> Option<String> {
    match v? {
        serde_json::Value::Null => None,
        serde_json::Value::String(s) if s.is_empty() => None,
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Bool(false) => None,
        other => Some(other.to_string()),
    }
}

/// cards.sources.describe: name, source, types, cost, tags, the compiled abilities and any text not modeled
pub fn describe(c: &RawCard) -> String {
    let tags: Vec<String> = c
        .tags
        .iter()
        .map(|(k, v)| match v {
            TagValue::Flag(true) => k.clone(),
            v => format!("{k}={}", tag_str(v)),
        })
        .collect();
    let tags = if tags.is_empty() { "(no tags)".to_string() } else { tags.join(" ") };
    let generic = if c.generic == 0 { String::new() } else { c.generic.to_string() };
    let mut s = format!("{} [{}] {} {generic}{} :: {tags}", c.name, c.source, c.types, c.pips);
    if let Some(serde_json::Value::Array(abilities)) = &c.dsl {
        for a in abilities {
            let what = json_str(a.get("event"))
                .or_else(|| json_str(a.get("static")))
                .or_else(|| json_str(a.get("replace")))
                .unwrap_or_default();
            let ty = json_str(a.get("type")).unwrap_or_default();
            let effects: Vec<String> = match a.get("effects") {
                Some(serde_json::Value::Array(es)) => {
                    es.iter().map(|e| json_str(e.get("do")).unwrap_or_else(|| "?".into())).collect()
                }
                _ => vec![],
            };
            s += &format!("\n      ability: {ty} {what} -> {}", effects.join(", "));
        }
    }
    for u in &c.unparsed {
        s += &format!("\n      not modeled: {u}");
    }
    s
}
