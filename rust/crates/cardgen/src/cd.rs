//! A card definition (Python's `engine.CD`) and its record in `data/cards.json`.

use crate::autotag::tagstr;
use crate::json::{J, Obj};
use crate::re::rmatch;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Cd {
    pub name: String,
    pub types: String,
    pub generic: i64,
    pub pips: String,
    pub cmc: i64,
    /// tag -> true, or its value as a string; insertion order kept
    pub tags: Obj,
    pub pow: i64,
    pub tgh: i64,
    pub bomb: i64,
    pub dsl: Option<Vec<Obj>>,
    pub start_loyalty: Option<String>,
    pub kws: BTreeSet<String>,
    pub subtypes: BTreeSet<String>,
    pub protfrom: String,
    pub ward: i64,
    pub identity: Option<String>,
    pub game_changer: Option<bool>,
    pub source: String,
    pub unparsed: Vec<String>,
    pub pool_override: bool,
    pub types_filled: bool,
}

fn tag_int(tags: &Obj, k: &str) -> Option<i64> {
    tags.get(k).map(|v| match v {
        J::Str(s) => s.trim().parse::<i64>().unwrap_or_else(|_| panic!("tag {k}={s}: not a number")),
        J::Int(i) => *i,
        _ => panic!("tag {k} has no number"),
    })
}

impl Cd {
    /// `CD(name, types, cost, tags)`: cost is the generic number then coloured pips ("2UU"; "-" or "" for none)
    pub fn new(name: &str, types: &str, cost: &str, tags: &str) -> Cd {
        let (generic, pips) = if cost == "-" || cost.is_empty() {
            (0, String::new())
        } else {
            let m = rmatch(r"^(\d*)(.*)$", cost).expect("cost");
            (m.gs(1).parse::<i64>().unwrap_or(0), m.gs(2).to_string())
        };
        let mut t = Obj::new();
        for tok in tags.split_whitespace() {
            match tok.split_once('=') {
                Some((k, v)) => t.set(k, v),
                None => t.set(tok, true),
            }
        }
        let creature = types.contains('C');
        let pow = tag_int(&t, "pow").unwrap_or(0);
        let tgh = if creature { tag_int(&t, "tgh").unwrap_or(pow.max(1)) } else { 0 };
        let bomb = tag_int(&t, "bomb").unwrap_or(0);
        Cd {
            name: name.to_string(),
            types: types.to_string(),
            generic,
            cmc: generic + pips.chars().count() as i64,
            pips,
            tags: t,
            pow,
            tgh,
            bomb,
            dsl: None,
            start_loyalty: None,
            kws: BTreeSet::new(),
            subtypes: BTreeSet::new(),
            protfrom: String::new(),
            ward: 0,
            identity: None,
            game_changer: None,
            source: "manual".into(),
            unparsed: vec![],
            pool_override: false,
            types_filled: false,
        }
    }

    pub fn land(&self) -> bool {
        self.types.contains('L')
    }
    pub fn creature(&self) -> bool {
        self.types.contains('C')
    }
    pub fn instant(&self) -> bool {
        self.types.contains('I')
    }
    pub fn sorcery(&self) -> bool {
        self.types.contains('S')
    }
    pub fn perm(&self) -> bool {
        !(self.instant() || self.sorcery() || self.land())
    }

    /// the tag string (`pow=3 fly leg`)
    pub fn tag_string(&self) -> String {
        tagstr(&self.tags)
    }

    /// the cost as `CD()` reads it back: the generic number (if any) then the pips, "0" when there is none
    pub fn cost_string(&self) -> String {
        let s = format!("{}{}", if self.generic != 0 { self.generic.to_string() } else { String::new() }, self.pips);
        if s.is_empty() { "0".into() } else { s }
    }

    /// set a tag from `k` or `k=v` as `cd.tags[k] = v if v else True` does
    pub fn set_tag(&mut self, kv: &str) {
        let (k, v) = match kv.split_once('=') {
            Some((k, v)) => (k, v),
            None => (kv, ""),
        };
        if v.is_empty() {
            self.tags.set(k, true);
        } else {
            self.tags.set(k, v);
        }
    }

    pub fn remove_tag(&mut self, k: &str) {
        self.tags.0.retain(|(x, _)| x != k);
    }
}

pub fn strs(set: &BTreeSet<String>) -> J {
    J::Arr(set.iter().map(|s| J::from(s.as_str())).collect())
}
