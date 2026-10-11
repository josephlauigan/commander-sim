//! What the engine reads from a card's Scryfall record besides its tags: Phyrexian mana colours, a land's enters
//! effects and the condition it enters untapped under, its basic land types (Python's `engine.phyrexian`,
//! `engine.land_etb_fx`, `ais.enters_rule`, `ais.land_types`).

use crate::cd::Cd;
use crate::json::J;
use crate::re::{findall, rmatch, search};
use crate::scryfall::Cache;

pub const BASIC_TYPES: [&str; 5] = ["plains", "island", "swamp", "mountain", "forest"];

fn rec<'a>(cache: &'a Cache, name: &str) -> Option<&'a J> {
    cache.get(name)
}

/// the colours of the Phyrexian mana symbols in the printed cost ('B' for {B/P})
pub fn phyrexian(cache: &Cache, cd: &Cd) -> String {
    let cost = rec(cache, &cd.name).and_then(|r| r.s("mana_cost")).unwrap_or("");
    let v: String = findall(r"\{([WUBRG])/P\}", cost).concat();
    if !v.is_empty() {
        v
    } else if cd.tags.contains("phyU") {
        "U".into()
    } else {
        String::new()
    }
}

/// a land's "when this land enters" effects: ("scry", n) / ("gain", n)
pub fn land_etb_fx(cache: &Cache, cd: &Cd) -> Vec<(String, i64)> {
    let txt = rec(cache, &cd.name).and_then(|r| r.s("oracle_text")).unwrap_or("").to_lowercase();
    let mut fx = vec![];
    if let Some(m) = search(r"when this land enters, scry (\d+)", &txt) {
        fx.push(("scry".to_string(), m.gs(1).parse().unwrap()));
    }
    if let Some(m) = search(r"when this land enters, you gain (\d+) life", &txt) {
        fx.push(("gain".to_string(), m.gs(1).parse().unwrap()));
    }
    fx
}

/// (basic land types, is basic) from the type line ('Land — Island Swamp')
pub fn land_types(cache: &Cache, name: &str) -> (Vec<String>, bool) {
    let tl = rec(cache, name).and_then(|r| r.s("type_line")).unwrap_or("").to_lowercase();
    let sub = tl.split_once('—').map(|x| x.1).unwrap_or("");
    let mut ts: Vec<String> =
        sub.split_whitespace().filter(|w| BASIC_TYPES.contains(w)).map(|w| w.to_string()).collect();
    ts.sort();
    ts.dedup();
    (ts, tl.starts_with("basic"))
}

fn two(s: &str) -> i64 {
    if s == "two" { 2 } else { 3 }
}

/// the condition under which land cd enters untapped, read from its Oracle text, as JSON: `[kind, args...]`
/// ('control' types, 'fewer' n, 'more' n, 'basics' n, 'opponents' n, 'types_more' type n, 'reveal' types,
/// 'legendary', 'planeswalker', 'pay' life), or None
pub fn enters_rule(cache: &Cache, cd: &Cd) -> Option<J> {
    let txt = rec(cache, &cd.name).and_then(|r| r.s("oracle_text")).unwrap_or("").to_lowercase();
    let s = |x: &str| J::from(x);
    let sorted = |ts: Vec<String>| {
        let mut ts = ts;
        ts.sort();
        ts.dedup();
        J::Arr(ts.iter().map(|t| J::from(t.as_str())).collect())
    };
    if let Some(m) = search(r"enters tapped unless (.*?)\.", &txt) {
        let cond = m.gs(1);
        if let Some(k) = rmatch(r"you control (two|three) or fewer other lands", cond) {
            return Some(J::Arr(vec![s("fewer"), two(k.gs(1)).into()]));
        } else if let Some(k) = rmatch(r"you control (two|three) or more other lands", cond) {
            return Some(J::Arr(vec![s("more"), two(k.gs(1)).into()]));
        } else if let Some(k) = rmatch(r"you control (two|three) or more basic lands", cond) {
            return Some(J::Arr(vec![s("basics"), two(k.gs(1)).into()]));
        } else if let Some(k) = rmatch(r"you have (two|three) or more opponents", cond) {
            return Some(J::Arr(vec![s("opponents"), two(k.gs(1)).into()]));
        } else if let Some(k) =
            rmatch(r"you control (two|three) or more other (\w+?)s\b", cond).filter(|k| BASIC_TYPES.contains(&k.gs(2)))
        {
            return Some(J::Arr(vec![s("types_more"), s(k.gs(2)), two(k.gs(1)).into()]));
        } else if cond.contains("legendary creature") {
            return Some(J::Arr(vec![s("legendary")]));
        } else if cond.contains("planeswalker") {
            return Some(J::Arr(vec![s("planeswalker")]));
        } else {
            let ts: Vec<String> =
                findall(r"an? (\w+)", cond).into_iter().filter(|w| BASIC_TYPES.contains(&w.as_str())).collect();
            if cond.starts_with("you control") && !ts.is_empty() {
                return Some(J::Arr(vec![s("control"), sorted(ts)]));
            }
        }
        None
    } else if let Some(m) = search(r"you may pay (\d+) life\. if you don.t, it enters tapped", &txt) {
        Some(J::Arr(vec![s("pay"), m.gs(1).parse::<i64>().unwrap().into()]))
    } else if let Some(m) =
        search(r"you may reveal an? (\w+) or (\w+) card from your hand\. if you don.t, [^.]*enters tapped", &txt)
    {
        let ts: Vec<String> =
            m.groups().into_iter().flatten().filter(|w| BASIC_TYPES.contains(w)).map(|w| w.to_string()).collect();
        Some(J::Arr(vec![s("reveal"), sorted(ts)]))
    } else {
        None
    }
}
