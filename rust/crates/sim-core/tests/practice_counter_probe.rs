//! Practice-mode counterspell probe (2026-11-04): the choice probe can't cast a counterspell (nothing on the stack), so
//! this one does. For every counterspell in the owned decks (an instant whose text says "counter target"), an
//! opponent casts a spell it can answer (a creature, a sorcery, an instant or an enchantment, tried in turn); the person
//! has the counterspell in hand with plenty of mana and casts it in the response window, answering every question with
//! the first choice. It must counter the spell. Each card's questions are printed (`--nocapture`) so its choices
//! (alternative costs, modes, X, which spell) can be reviewed against the Oracle text.

mod play_common;
use play_common::*;
use sim_core::state::Ctx;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

fn json_file(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

/// the opponent's spells to answer, in the order tried: one each of the kinds counterspells name
const SPELLS: &[&str] = &["Grave Titan", "Demonic Tutor", "Brainstorm", "Rhystic Study", "Sol Ring", "Swords to Plowshares"];

/// (countered, the person's questions, what they were told) for counterspell `name` against `spell`
fn try_counter(key: &str, name: &str, spell: &str) -> (bool, Vec<String>, Vec<String>) {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P1); // the opponent's turn: their sorcery-speed spell is legal
    let c = take(&mut g, P0, name);
    g.player_mut(P0).hand.push(c);
    // something to pitch or reveal for alternative costs (Force of Will, Force of Negation, Mindbreak Trap ...)
    hand(&mut g, P0, &["Brainstorm", "Mystical Tutor"]); // blue cards to pitch (not a second copy of a probed card)
    pool(&mut g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6), ('C', 6)]);
    let elf = perm(&mut g, P0, "Llanowar Elves"); // a permanent of yours: Swords to Plowshares aims at it (Not of This World)
    let first = Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "cast", "card": 0}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    let s = hand(&mut g, P1, &[spell])[0];
    // cast_card: true when the spell resolved, false when it was countered
    let resolved = sim_core::engine::cast::cast_card(&mut g, P1, s, "hand", Ctx { target: (spell == "Swords to Plowshares").then_some(elf), ..Ctx::default() }).unwrap_or(true);
    let countered = !resolved;
    if std::env::var("PROBE_DUMP").is_ok_and(|v| name.contains(v.as_str())) {
        for r in h.asked() {
            eprintln!("  ASK {}: {} {:?}", r.kind, r.prompt, r.choices);
        }
        for t in h.told() {
            eprintln!("  TOLD {}: {}", t.0, t.1);
        }
    }
    let asked: Vec<String> =
        h.asked().iter().filter(|r| r.kind != "priority").map(|r| format!("{}: {}", r.kind, r.prompt)).collect();
    let told: Vec<String> = h.told().into_iter().map(|t| t.1).collect();
    (countered, asked, told)
}

#[test]
fn every_counterspell_of_the_owned_decks_works_for_a_person() {
    let cache = json_file("scryfall_cache.json");
    let decks = json_file("decks.json");
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut failed: Vec<String> = vec![];
    let mut n = 0;
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        for name in d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str()) {
            let Some(e) = cache.get(name.to_lowercase()) else { continue };
            let text = e["oracle_text"].as_str().unwrap_or("").to_lowercase();
            let instant = e["type_line"].as_str().unwrap_or("").contains("Instant");
            if !instant || !text.contains("counter target") || !seen.insert(name.to_string()) {
                continue;
            }
            n += 1;
            let mut result = None;
            for spell in SPELLS {
                std::panic::set_hook(Box::new(|_| {}));
                let r = std::panic::catch_unwind(|| try_counter(key, name, spell));
                let _ = std::panic::take_hook();
                match r {
                    Ok((true, asked, _)) => {
                        result = Some(format!("countered {spell}; asked: {}", if asked.is_empty() { "nothing".into() } else { asked.join(" || ") }));
                        break;
                    }
                    Ok((false, _, told)) => result = result.or(Some(format!("NOT countered (last told: {})", told.last().cloned().unwrap_or_default()))),
                    Err(_) => result = result.or(Some("panicked".into())),
                }
            }
            let r = result.unwrap_or_default();
            eprintln!("{key}: {name}: {r}");
            if !r.starts_with("countered") {
                failed.push(format!("{key}: {name}: {r}"));
            }
        }
    }
    eprintln!("{n} counterspells probed, {} failed", failed.len());
    for f in &failed {
        eprintln!("!! {f}");
    }
    assert!(failed.is_empty(), "{} counterspell(s) failed for a person: see above", failed.len());
}

/// Protection cards (the AI's `ai::pool` protection table) a person holds: an opponent casts Infernal Grasp at the
/// person's creature, the person answers with the card, and the creature must survive (indestructible, phased out,
/// blinked, hexproof, the spell countered or redirected). The cards whose effect lives only in the AI's table did
/// nothing for a person.
#[test]
fn every_protection_card_of_the_owned_decks_saves_a_persons_creature() {
    let decks = json_file("decks.json");
    let mut cards: BTreeSet<(String, String)> = BTreeSet::new();
    let list = [
        "Deflecting Swat",
        "Eerie Interlude",
        "Ephemerate",
        "Make a Stand",
        "Not of This World",
        "Rootborn Defenses",
        "Slip Out the Back",
        "Teferi's Protection",
        "Unbreakable Formation",
    ];
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        for name in d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str()) {
            if list.contains(&name) && !cards.iter().any(|(_, n)| n == name) {
                cards.insert((key.to_string(), name.to_string()));
            }
        }
    }
    let mut failed = vec![];
    for (key, name) in &cards {
        let mut g = table(&[key.as_str(), "seph"]);
        g.active = Some(P1);
        let elf = perm(&mut g, P0, "Llanowar Elves");
        perm(&mut g, P1, "Birds of Paradise"); // somewhere else to send the removal (Deflecting Swat)
        let c = take(&mut g, P0, name);
        g.player_mut(P0).hand.push(c);
        pool(&mut g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6), ('C', 8)]);
        let first = Mutex::new(true);
        let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            if &*r.kind == "priority" {
                let mut f = first.lock().unwrap();
                let a = if *f { json!({"do": "cast", "card": 0}) } else { json!({"do": "pass"}) };
                *f = false;
                return Answer::Action(a);
            }
            // a redirect (Deflecting Swat): any new target but the Elves
            if r.prompt.to_lowercase().contains("new target")
                && let Some(i) = r.choices.iter().position(|c| !c.contains("Llanowar"))
            {
                return Answer::Pick(i);
            }
            if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
        }));
        g.humans.add(P0, h.clone());
        let grasp = hand(&mut g, P1, &["Infernal Grasp"])[0];
        let _ = sim_core::engine::cast::cast_card(&mut g, P1, grasp, "hand", Ctx { target: Some(elf), ..Ctx::default() });
        // the Elves (or the creature blinked back) are still the person's
        // (or exiled to come back at the end step: Eerie Interlude)
        let alive = g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Llanowar Elves")
            || g.player(P0).exile.cards().iter().any(|&x| &*g.db.get(x).name == "Llanowar Elves");
        let used = !g.player(P0).hand.contains(&c);
        let asked: Vec<String> =
            h.asked().iter().filter(|r| r.kind != "priority").map(|r| format!("{}: {}", r.kind, r.prompt)).collect();
        eprintln!("{key}: {name}: survived {alive}, cast {used}; asked: {}", asked.join(" || "));
        if !alive {
            failed.push(format!("{key}: {name} (cast {used}; told {:?})", h.told().last()));
        }
    }
    for f in &failed {
        eprintln!("!! {f}");
    }
    assert!(failed.is_empty(), "{} protection card(s) did not save the creature: see above", failed.len());
}
