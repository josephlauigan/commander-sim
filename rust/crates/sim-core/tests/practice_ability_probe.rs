//! Practice-mode ability probe (2026-11-04): every card in the owned decks (`data/decks.json` "mine") whose Oracle text
//! has a non-mana activated ability is put onto the battlefield for a seat played by a person (untapped, not summoning
//! sick, with lands of every colour) and must list at least one ability the person can use (`human::abilities_of`,
//! or for a land `abilities::land_abilities` and `alt::land_alts`). A card that lists nothing is one the person can't
//! activate in practice mode (Garth One-Eye was the first found: its option hook returns nothing for a person).
//!
//! The Oracle text is read from `data/scryfall_cache.json`. Mana abilities ("Add ..."), Equip (its own path) and
//! abilities used from the hand or graveyard (channel, cycling, unearth ...) are left out. Each seat is probed in its
//! first and second main phase and on an opponent's turn, and a card passes if any of them lists something.

use serde_json::Value;
use sim_core::human::script::ScriptHuman;
use sim_core::human::{abilities, alt, human};
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn json(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

/// the Oracle text of a card (every face), reminder text removed
fn oracle(cache: &Value, name: &str) -> Option<String> {
    let e = cache.get(name.to_lowercase())?;
    let mut t = e["oracle_text"].as_str().unwrap_or("").to_string();
    if let Some(fs) = e["card_faces"].as_array() {
        for f in fs {
            t.push('\n');
            t.push_str(f["oracle_text"].as_str().unwrap_or(""));
        }
    }
    let mut out = String::new();
    let mut depth = 0;
    for ch in t.chars() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    Some(out)
}

/// the Oracle lines that are activated abilities used on the battlefield, other than mana abilities and Equip
fn activated(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|l| {
            let (cost, effect) = l.split_once(':')?;
            // "Channel — {1}{G}, Discard this card": the cost is after an ability word's dash
            let cost = cost.rsplit('—').next().unwrap_or(cost).trim();
            let effect = effect.trim();
            let loyalty = cost.starts_with(['+', '−', '-', '0']) && cost.len() <= 4;
            let costy = cost.contains('{')
                || ["Sacrifice", "Exile", "Pay", "Remove", "Discard", "Tap", "Return"].iter().any(|w| cost.starts_with(w));
            if !(loyalty || costy) {
                return None;
            }
            let lc = cost.to_lowercase();
            let le = effect.to_lowercase();
            if cost.starts_with("Equip")
                || le.starts_with("add ")
                || le.contains(", add ")
                || le.contains("add one mana") // Faeburrow Elder: a mana ability too
                || le.contains("only your opponents may activate") // Knight of the Holy Nimbus
                || l.starts_with("Grandeur") // Page, Loose Leaf: needs a second copy, never in a singleton deck
                || lc.contains("discard this card")
                || lc.contains("from your graveyard")
                || lc.contains("from your hand")
                || lc.contains("exile this card")
            {
                return None;
            }
            Some(l.to_string())
        })
        .collect()
}

/// what a person at seat 0 is offered on the card now, or why the probe couldn't set it up
fn offered(key: &str, name: &str, land: bool, active: PlayerId, step: Step) -> Result<usize, String> {
    let r = catch_unwind(AssertUnwindSafe(|| -> usize {
        let mut g: Game = table(&[key, "seph"]);
        for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            lands(&mut g, P0, b, 3, false);
        }
        let m = if land { lands(&mut g, P0, name, 1, false)[0] } else { perm(&mut g, P0, name) };
        // the person sits down after the card entered: its enter prompts (Stoneforge Mystic, Inferno Titan) are not the probe's
        g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
        g.active = Some(active);
        g.step = step;
        if !g.player(P0).perms.contains(&m) && !g.player(P0).lands.contains(&m) {
            return usize::MAX; // left the battlefield on entering (a sacrifice-on-enter card): not probed
        }
        let mut n = human::abilities_of(&mut g, P0, m).map(|v| v.len()).unwrap_or(0);
        if g.player(P0).lands.contains(&m) {
            n += abilities::land_abilities(&g, P0, m).len();
            n += alt::land_alts(&mut g, P0, m).map(|v| v.len()).unwrap_or(0);
        }
        n
    }));
    r.map_err(|e| {
        e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
    })
}

#[test]
fn every_activated_ability_of_the_owned_decks_is_offered_to_a_person() {
    let cache = json("scryfall_cache.json");
    let decks = json("decks.json");
    let db = db();
    let mut gaps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut errors: Vec<String> = vec![];
    let mut probed = 0;
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let mut names: Vec<String> = d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(text) = oracle(&cache, &name) else { continue };
            let lines = activated(&text);
            if lines.is_empty() {
                continue;
            }
            let Some(c) = db.id(&name) else {
                errors.push(format!("{key}: {name}: not in cards.json"));
                continue;
            };
            let land = db.get(c).land;
            probed += 1;
            let mut best = 0usize;
            let mut err = None;
            for (active, step) in [(P0, Step::Main1), (P0, Step::Main2), (P1, Step::Main1)] {
                std::panic::set_hook(Box::new(|_| {}));
                let r = offered(key, &name, land, active, step);
                let _ = std::panic::take_hook();
                match r {
                    Ok(n) => best = best.max(n),
                    Err(e) => err = Some(e),
                }
            }
            if best == 0 {
                match err {
                    Some(e) => errors.push(format!("{key}: {name}: probe panicked: {e}")),
                    None => gaps.entry(key.to_string()).or_default().push(format!("{name}  |  {}", lines.join("  /  "))),
                }
            }
        }
    }
    eprintln!("probed {probed} cards with activated abilities");
    for (k, v) in &gaps {
        eprintln!("== {k}: {} card(s) with nothing to activate", v.len());
        for x in v {
            eprintln!("   {x}");
        }
    }
    for e in &errors {
        eprintln!("!! {e}");
    }
    assert!(errors.is_empty(), "{} probe error(s): see above", errors.len());
    check(&gaps);
}

/// gaps found on 2026-11-04 and not yet fixed: the person cannot activate these in practice mode (or only when the AI's
/// own conditions say so). Fixing one means taking it off this list; a new gap fails the test.
const TO_FIX: &[&str] = &[
];

/// listed by the probe but not a gap, each with why
const FALSE_ALARMS: &[(&str, &str)] = &[(
    "kefka: Birgi, God of Storytelling",
    "the one activated ability (Harnfel, Horn of Bounty: discard a card, exile the top two) is the back face of the modal double-faced card, which the engine never plays as a permanent; Birgi on the battlefield has no activated ability",
)];

/// the gaps found against the two lists: a gap on neither list is new (fails), a TO_FIX entry no longer found is
/// fixed and must come off the list (fails), and a FALSE_ALARMS entry no longer found must come off too (fails)
fn check(found: &BTreeMap<String, Vec<String>>) {
    let found: std::collections::BTreeSet<String> = found
        .iter()
        .flat_map(|(k, v)| v.iter().map(move |x| format!("{k}: {}", x.split("  |  ").next().unwrap_or(x))))
        .collect();
    let listed = |s: &str| TO_FIX.contains(&s) || FALSE_ALARMS.iter().any(|(n, _)| *n == s);
    let new: Vec<&String> = found.iter().filter(|s| !listed(s)).collect();
    let fixed: Vec<&str> =
        TO_FIX.iter().chain(FALSE_ALARMS.iter().map(|(n, _)| n)).copied().filter(|s| !found.contains(*s)).collect();
    for s in &new {
        eprintln!("NEW GAP: {s}");
    }
    for s in &fixed {
        eprintln!("NO LONGER FOUND (take it off the list): {s}");
    }
    eprintln!("{} found: {} to fix, {} false alarms", found.len(), TO_FIX.len(), FALSE_ALARMS.len());
    assert!(new.is_empty() && fixed.is_empty(), "{} new gap(s), {} list entr(ies) to remove: see above", new.len(), fixed.len());
}
