//! Practice-mode choice probe (2026-11-04): every card in the owned decks (`data/decks.json` "mine") whose Oracle text
//! implies a choice when it is played, cast or enters ("search", "choose", "target", "you may", "sacrifice a",
//! "discard a", "return a" ...) is played through a seat with a person: a land as the land drop, anything else cast
//! from the hand with plenty of mana, on a table with targets (creatures, an artifact, an enchantment, cards in both
//! graveyards). The person answers every question with the first choice and is recorded. A card that resolves without
//! asking anything is one whose choice the AI makes for the person (fetch lands were the first found: `crack_fetch`).
//!
//! It sees the choices made while the card is played and resolves (its spell text, its enter trigger, a land's enter
//! effects), not triggers later in a game (attack, upkeep, an opponent's spell). Lines that are activated abilities
//! ("cost: effect") are left out except a land's (a fetch land is cracked on entering), and so are triggers other than
//! "when ... enters". The commanders are cast from the hand like any card.

mod play_common;
use play_common::*;
use sim_core::state::Step;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

fn json(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

fn strip_reminders(t: &str) -> String {
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
    out
}

/// the front face's Oracle text (what is cast or played), reminder text removed
fn oracle(cache: &Value, name: &str) -> Option<String> {
    let e = cache.get(name.to_lowercase())?;
    let t = match e["card_faces"].as_array() {
        Some(fs) if e["oracle_text"].is_null() => fs.first()?["oracle_text"].as_str().unwrap_or("").to_string(),
        _ => e["oracle_text"].as_str().unwrap_or("").to_string(),
    };
    Some(strip_reminders(&t))
}

const WORDS: &[&str] = &[
    "search your library",
    "choose",
    "you may",
    "target",
    "sacrifice a",
    "sacrifice an",
    "discard a",
    "return a",
    "return up to",
    "exile up to",
];

/// the lines that make the person choose as the card is played, cast or enters
fn choice_lines(text: &str, land: bool, spell: bool) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| {
            let lc = l.to_lowercase();
            // "you may cast spells as though they had flash", "you may choose new targets": a permission, not a choice
            let permission = ["you may cast", "you may play", "choose new targets", "without paying"].iter().any(|w| lc.contains(w));
            if !WORDS.iter().any(|w| lc.contains(w) && !(permission && *w == "you may")) {
                return false;
            }
            let activated = l.split_once(':').is_some_and(|(c, _)| c.contains('{') || c.starts_with("Sacrifice") || c.starts_with("Pay"));
            if activated {
                return land && lc.contains("search your library"); // a fetch land, cracked as it enters
            }
            let trigger = lc.starts_with("when") || lc.starts_with("at the") || lc.starts_with("whenever");
            if trigger {
                return lc.starts_with("when") && !lc.starts_with("whenever") && lc.contains(" enters");
            }
            if lc.starts_with("as ") && lc.contains(" enters") {
                return true; // "As this enters, choose a color"
            }
            spell // an instant's or sorcery's text resolves; a permanent's static text doesn't ask
        })
        .map(String::from)
        .collect()
}

/// one card played through the person's seat: (the questions asked, what it was told it couldn't do)
fn play_it(key: &str, name: &str, land: bool) -> (Vec<String>, Vec<String>, bool) {
    let mut g: Game = table(&[key, "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    // a table with at least two of everything to choose from (one legal choice may be taken without asking), more
    // lands on the opponent's side ("if an opponent controls more lands") and three Islands (Mystic Sanctuary)
    for n in ["Llanowar Elves", "Birds of Paradise"] {
        perm(&mut g, P0, n);
    }
    lands(&mut g, P0, "Island", 3, false);
    for n in ["Llanowar Elves", "Birds of Paradise", "Sol Ring", "Arcane Signet", "Propaganda", "Doubling Season"] {
        perm(&mut g, P1, n);
    }
    lands(&mut g, P1, "Forest", 6, false);
    for (p, n) in [
        (P0, "Grave Titan"),
        (P0, "Sakura-Tribe Elder"),
        (P0, "Counterspell"),
        (P0, "Demonic Tutor"),
        (P0, "Sol Ring"),
        (P0, "Forest"),
        (P1, "Grave Titan"),
        (P1, "Sol Ring"),
    ] {
        let c = take(&mut g, p, n);
        g.player_mut(p).gy.push(c);
    }
    // cards whose choice needs Equipment on the table (Kor Outfitter attaches one; Forge Anew returns one)
    match name {
        "Forge Anew" => {
            let c = take(&mut g, P0, "Bonesplitter");
            g.player_mut(P0).gy.push(c);
        }
        "Kor Outfitter" => {
            perm(&mut g, P0, "Bonesplitter");
        }
        _ => {}
    }
    hand(&mut g, P0, &["Plains", "Island", "Swamp"]); // something to discard or reveal (Foreboding Ruins)
    let c = take(&mut g, P0, name);
    g.player_mut(P0).hand.push(c);
    if !land || sim_core::human::fetch::person_fetch(name) {
        pool(&mut g, P0, &[('W', 10), ('U', 10), ('B', 10), ('R', 10), ('G', 10), ('C', 10)]);
    }
    let first = Mutex::new(true);
    // a fetch land stays on the battlefield for a person: the next thing they do is activate it (the land just played is
    // the fourth: three Islands are already there)
    let fetch = land && sim_core::human::fetch::person_fetch(name);
    let second = Mutex::new(fetch);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        match &*r.kind {
            "priority" => {
                let mut f = first.lock().unwrap();
                if *f {
                    *f = false;
                    // the probed card is the last in hand
                    return Answer::Action(json!({"do": if land { "land" } else { "cast" }, "card": 3}));
                }
                let mut s = second.lock().unwrap();
                if *s {
                    *s = false;
                    return Answer::Action(json!({"do": "use", "land": 3}));
                }
                Answer::Action(json!({"do": "pass"}))
            }
            "attack" | "block" => Answer::Picks(vec![]),
            _ if r.choices.is_empty() => Answer::Yes,
            _ => Answer::Pick(0),
        }
    }));
    g.humans.add(P0, h.clone());
    let _ = sim_core::human::human::human_main(&mut g, P0, false);
    // only this card's questions: a prompt headed by another card's name ("Ad Nauseam: ...", a trigger elsewhere) is not it
    let other = |p: &str| p.split_once(':').is_some_and(|(h, _)| h != name && !name.starts_with(h) && g.db.id(h).is_some());
    let asked: Vec<String> = h
        .asked()
        .iter()
        .filter(|r| r.kind != "priority" && !other(&r.prompt) && !(fetch && r.prompt.ends_with("which ability?")))
        .map(|r| if std::env::var("PROBE_CHOICES").is_ok() { format!("{}: {} {:?}", r.kind, r.prompt, r.choices) } else { format!("{}: {}", r.kind, r.prompt) })
        .collect();
    let refused = h.told_kind("invalid");
    let left_hand = !g.player(P0).hand.contains(&c);
    (asked, refused, left_hand)
}

#[test]
fn every_choice_of_the_owned_decks_is_asked_of_a_person() {
    let cache = json("scryfall_cache.json");
    let decks = json("decks.json");
    let db = db();
    let mut silent: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut unplayed: Vec<String> = vec![];
    let mut probed = 0;
    let mut asked_ok = 0;
    let mut twice: Vec<String> = vec![];
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let mut names: Vec<String> =
            d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(c) = db.id(&name) else { continue };
            let def = db.get(c);
            let spell = def.instant || def.sorcery;
            let Some(text) = oracle(&cache, &name) else { continue };
            let lines = choice_lines(&text, def.land, spell);
            if lines.is_empty() {
                continue;
            }
            probed += 1;
            std::panic::set_hook(Box::new(|_| {}));
            let r = catch_unwind(AssertUnwindSafe(|| play_it(key, &name, def.land)));
            let _ = std::panic::take_hook();
            match r {
                Err(e) => {
                    let e = e
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_default();
                    unplayed.push(format!("{key}: {name}: panicked: {e}"));
                }
                Ok((_, refused, false)) => {
                    unplayed.push(format!("{key}: {name}: not played ({})", refused.join("; ")));
                }
                Ok((asked, _, true)) if asked.is_empty() => {
                    silent.entry(key.to_string()).or_default().push(format!("{name}  |  {}", lines.join("  /  ")));
                }
                Ok((asked, _, _)) => {
                    asked_ok += 1;
                    // the same question twice in a row: asked before a trigger's first window, so the engine's probe run of
                    // the trigger asked it once and the real run again (`stack::resolve_batch`)
                    if asked.windows(2).any(|w| w[0] == w[1]) {
                        eprintln!("TWICE {key}: {name}: {}", asked.join(" || "));
                        twice.push(format!("{key}: {name}"));
                    }
                    if std::env::var("PROBE_DUMP").is_ok() {
                        eprintln!("ASKED {key}: {name}: {}", asked.join(" || "));
                    }
                }
            }
        }
    }
    eprintln!("probed {probed} cards with a choice as they are played; {asked_ok} asked the person something");
    for (k, v) in &silent {
        eprintln!("== {k}: {} card(s) resolved without asking", v.len());
        for x in v {
            eprintln!("   {x}");
        }
    }
    for e in &unplayed {
        eprintln!("!! {e}");
    }
    assert!(twice.is_empty(), "asked the same question twice (before a trigger's first window?): {twice:?}");
    check(&silent);
}

/// gaps found on 2026-11-04 and not yet fixed: the AI chooses for the person. Fixing one means taking it off this list;
/// a new gap fails the test.
/// (2026-11-05) Doomsday has no card code yet (model-gaps PR 2, `doomsday` branch, models
/// it); Ponder resolves without asking the person to order the top three or to shuffle (fixed in the same branch).
const TO_FIX: &[&str] = &[];

/// listed by the probe but not a gap, each with why
const FALSE_ALARMS: &[(&str, &str)] = &[
    ("yshtola: Urza's Cave", "an activated ability, not used as it enters (its activation is driven by practice_trigger_probe's Activate pass, which asks the search)"),
    ("sauron: Urza's Cave", "an activated ability, not used as it enters (its activation is driven by practice_trigger_probe's Activate pass, which asks the search)"),
    ("sauron: Deflecting Swat", "new targets for a spell on the stack: the probe has none"),
    ("thanos: Deflecting Swat", "new targets for a spell on the stack: the probe has none"),
    ("sauron: Not of This World", "counters a spell targeting your permanent: the probe has none"),
    ("sheoldred: Invoke Despair", "the opponent chooses what to sacrifice, not the person"),
    ("sheoldred: Rooftop Assassin", "no creature was dealt damage this turn: no legal target"),
    ("cloud: Loyal Warhound", "a basic Plains: every choice is the same card"),
    ("alela: Airlift Chaplain", "already asks; the probe's three milled cards held no Plains or cheap creature, so there was nothing to choose (practice_gaps_c.rs)"),
    ("kefka: Twinned Vision", "cast from the hand it only draws a card, nothing to choose; the flashback's discard is the person's (practice_gaps_a: twinned_vision_flashback_discards_the_card_the_person_names)"),
];

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
