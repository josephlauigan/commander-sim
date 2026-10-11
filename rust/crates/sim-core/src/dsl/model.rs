//! The card ability language's data, typed: what `cards/dsl_parse.py` compiles Oracle text into. Parsed once when the
//! card database loads; a field this doesn't know is a load error, so a new construct in the compiler can't be
//! silently ignored here.

use serde::Deserialize;

/// A number in card text: a count, or a named quantity ('X', 'type_you:elf', 'creatures_you_control' ...).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Num {
    Int(i32),
    Name(String),
}

/// Which permanents or cards an effect applies to.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// 'you' or 'opp'
    pub controller: Option<String>,
    pub other: Option<bool>,
    pub keyword: Option<String>,
    /// (compiled for some cards, not read by the interpreter)
    pub kw: Option<String>,
    pub basic: Option<bool>,
    pub subtype: Option<String>,
    pub min_mv: Option<u32>,
    pub max_mv: Option<u32>,
    pub legendary: Option<bool>,
    pub nonlegendary: Option<bool>,
    pub max_pow: Option<i32>,
    pub min_pow: Option<i32>,
    pub nontoken: Option<bool>,
    /// (compiled for some cards, not read by the interpreter)
    pub color: Option<String>,
}

/// A selection: 'self', 'all' matching the filter, a 'target', 'any_target', a 'player', the 'event' permanent.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sel {
    pub sel: Option<String>,
    pub filter: Option<Filter>,
    pub upto: Option<bool>,
    pub who: Option<String>,
}

/// `to`: where damage goes (a selection), or where a found card goes ('hand', 'battlefield', 'top' ...)
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum To {
    Sel(Box<Sel>),
    Zone(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    #[serde(rename = "do")]
    pub kind: String,
    pub n: Option<Num>,
    pub who: Option<String>,
    pub what: Option<Sel>,
    pub to: Option<To>,
    pub filter: Option<Filter>,
    pub pow: Option<Num>,
    pub tgh: Option<Num>,
    pub keyword: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub attacking: Option<bool>,
    pub warrior: Option<bool>,
    pub types: Option<Vec<String>>,
    pub tapped: Option<bool>,
    pub from: Option<String>,
    pub per_opponent: Option<bool>,
    pub choose: Option<u32>,
    #[serde(default)]
    pub modes: Vec<Vec<Effect>>,
    pub unless: Option<u32>,
    pub optional: Option<bool>,
    pub random: Option<bool>,
    pub mult: Option<i32>,
}

/// An activated ability's cost.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    pub mana: Option<String>,
    pub tap: Option<bool>,
    /// 'self', or a permanent kind to sacrifice
    pub sac: Option<String>,
    pub life: Option<i32>,
    pub discard: Option<u32>,
    pub other: Option<String>,
    pub tap_other: Option<serde_json::Value>,
    pub energy: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Ability {
    /// an instant's or sorcery's effects
    Spell {
        effects: Vec<Effect>,
    },
    Triggered {
        event: String,
        source: Option<String>,
        who: Option<String>,
        spell: Option<String>,
        nth: Option<u32>,
        other: Option<bool>,
        if_cast: Option<bool>,
        min_pow: Option<i32>,
        subtype: Option<String>,
        effects: Vec<Effect>,
    },
    Activated {
        #[serde(default)]
        cost: Cost,
        effects: Vec<Effect>,
        sorcery: Option<bool>,
        once: Option<bool>,
        ai_min: Option<f64>,
    },
    Loyalty {
        loyalty: i32,
        effects: Vec<Effect>,
    },
    Static {
        #[serde(rename = "static")]
        kind: String,
        pow: Option<i32>,
        tgh: Option<i32>,
        filter: Option<Filter>,
        mana: Option<u32>,
        keyword: Option<String>,
        text: Option<String>,
        per: Option<Num>,
        base0: Option<bool>,
        who: Option<String>,
        spell: Option<String>,
        amount: Option<i32>,
        colors: Option<String>,
    },
    AdditionalCost {
        text: String,
    },
    Replacement {
        replace: String,
        multiplier: Option<i32>,
    },
}

impl Ability {
    /// the static or replacement kind ('anthem', 'tokens' ...), for indexing static abilities
    pub fn static_kind(&self) -> Option<&str> {
        match self {
            Ability::Static { kind, .. } => Some(kind),
            Ability::Replacement { replace, .. } => Some(replace),
            _ => None,
        }
    }
}

/// Parse a card's compiled abilities (the export's `dsl` list).
pub fn parse(v: &serde_json::Value) -> Result<Vec<Ability>, String> {
    serde_json::from_value(v.clone()).map_err(|e| e.to_string())
}
