//! Practice-mode Rings of Brighthearth probe (model-gaps-b2, 2026-11-05): a person with Rings of Brighthearth and {2}
//! to spare activates every non-mana ability the owned decks give them, and each one that goes on the stack must ask
//! "Rings of Brighthearth: pay {2} to copy it?"; a "yes" must resolve a copy (the engine logs "the copy of X resolves"
//! when it carries a copy's effect out, and no copy may be left waiting in `trig_copies`).
//!
//! Each card is put on the battlefield of a fresh game (untapped, not summoning sick, with lands, mana, creatures,
//! artifacts and graveyard cards around it, so abilities that need a target or a cost can be paid; `prep` adds what one
//! card needs). Every ability its menu lists is activated in turn (first main phase, then second main phase) with a
//! scripted answer for every question: the first choice (never Rings itself), "yes" to Rings, a pass for every
//! priority. A new card whose ability is not offered, or is offered and not carried out, fails the test; an exception
//! needs a reason (`EXCEPTIONS`) and is JD's to accept.

use serde_json::{Value, json};
use sim_core::control::{Answer, Request};
use sim_core::engine::zones::{self, Enter};
use sim_core::human::script::ScriptHuman;
use sim_core::human::{abilities, alt, human};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game, Step, Val};
use sim_core::testkit::*;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// the decks with Rings of Brighthearth (karn, thrasios) or Peter Parker's Camera (sauron) first, then the rest: a card
/// added to any of them later is covered too
const DECKS: &[&str] =
    &["karn", "thrasios", "sauron", "alela", "cloud", "galadriel", "kefka", "sheoldred", "thanos", "urdragon", "yshtola"];

#[derive(Debug, Clone, PartialEq)]
enum Out {
    /// Rings asked and the copy resolved
    Offered,
    /// the ability went on the stack and Rings asked, but no copy resolved after a yes
    NoCopy,
    /// the ability went on the stack and Rings did not ask
    NotOffered,
    /// nothing went on the stack (what the probe tried is in the note)
    NotExercised(String),
    /// the probe panicked
    Panicked(String),
}

fn json_file(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(data_dir().join(file)).unwrap()).unwrap()
}

fn pool(g: &mut Game, p: PlayerId, mana: &[(char, u32)]) {
    for &(c, n) in mana {
        g.player_mut(p).pool.add(c, n);
    }
}

/// the board every case starts from
fn board(g: &mut Game) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, P0, b, 3, false);
    }
    perm(g, P0, "Rings of Brighthearth");
    token(g, P0, 2);
    let big = token(g, P0, 4);
    g.perm_mut(big).ttypes.push("assembly-worker");
    g.in_combat.push(big); // an attacking creature (Foundry, Angelic Page ...)
    token(g, P1, 2);
    perm(g, P1, "Serra Angel");
    perm(g, P0, "Sol Ring");
    perm(g, P0, "Mind Stone");
    perm(g, P0, "Arcane Signet");
    perm(g, P1, "Mind Stone");
    for n in ["Mountain", "Island", "Lightning Bolt", "Counterspell", "Sol Ring", "Llanowar Elves"] {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let c = take(g, P1, "Swamp");
    g.player_mut(P1).gy.push(c);
    // exiled: an artifact card (Karn, the Great Creator) and a card with a silver counter (Karn, Scion of Urza)
    for n in ["Sol Ring", "Mind Stone"] {
        let c = take(g, P0, n);
        g.player_mut(P0).exile.push(c);
    }
    g.player_mut(P0).exile.last_mut().unwrap().silver = true;
    hand(g, P0, &["Forest", "Island", "Sol Ring", "Llanowar Elves"]);
    g.died_turn = Some(g.turn_stamp()); // a creature died this turn (Barad-dûr)
    pool(g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6), ('C', 12)]);
}

/// what one card needs besides the board
fn prep(g: &mut Game, name: &str, m: PermId) {
    if g.perm(m).loyalty.is_some() {
        g.perm_mut(m).loyalty = Some(20); // every loyalty ability, the ultimates too
    }
    match name {
        "Grim Monolith" | "Basalt Monolith" => g.perm_mut(m).tapped = true, // their untap needs it tapped
        "Steel Hellkite" => {
            let st = g.turn_stamp();
            g.perm_mut(m).data.set(DataKey::Hit, Val::List(vec![Val::Stamp(st), Val::Player(P1)]));
        }
        "Urza's Saga" => {
            g.player_mut(P0).sagas.retain(|x| x.0 != m);
            g.player_mut(P0).sagas.push((m, 2));
        }
        "Dauthi Voidwalker" => {
            let c = take(g, P1, "Lightning Bolt");
            g.player_mut(P1).exile.push(c);
            g.perm_mut(m).data.set(DataKey::Void, Val::List(vec![Val::List(vec![Val::Card(c), Val::Player(P1)])]));
        }
        _ => {}
    }
}

/// put the card on P0's battlefield (the X creatures with counters)
fn place(g: &mut Game, name: &str, land: bool) -> PermId {
    if land {
        return lands(g, P0, name, 1, false)[0];
    }
    if matches!(name, "Walking Ballista" | "Hangarback Walker") {
        let c = take(g, P0, name);
        return zones::enter(g, P0, c, Enter { sick: false, plus: 3, ..Default::default() }).unwrap();
    }
    perm(g, P0, name)
}

struct Probe {
    asked_rings: bool,
    answered_which: bool,
    n: usize,
}

/// one ability of one card, activated once
fn run_case(key: &str, name: &str, land: bool, k: usize, step: Step) -> Out {
    let r = catch_unwind(AssertUnwindSafe(|| -> Out {
        let mut g: Game = table(&[key, "seph"]);
        g.log = Some(vec![]);
        let m = place(&mut g, name, land);
        board(&mut g);
        prep(&mut g, name, m);
        let on_bf = g.player(P0).perms.contains(&m) || g.player(P0).lands.contains(&m);
        if !on_bf {
            return Out::NotExercised("left the battlefield on entering".into());
        }
        g.active = Some(P0);
        g.step = step;
        let state = Arc::new(Mutex::new(Probe { asked_rings: false, answered_which: false, n: 0 }));
        let st = state.clone();
        // the label of ability k: fetched with the person seated (the menus read the seat)
        g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
        let labels: Vec<String> = if land {
            let mut v: Vec<String> = abilities::land_abilities(&g, P0, m).into_iter().map(|a| a.label).collect();
            v.extend(alt::land_alts(&mut g, P0, m).unwrap_or_default().into_iter().map(|a| a.0));
            v
        } else {
            human::abilities_of(&mut g, P0, m).map(|v| v.into_iter().map(|a| a.0).collect()).unwrap_or_default()
        };
        let Some(label) = labels.get(k).cloned() else { return Out::NotExercised("no such ability".into()) };
        let want = label.clone();
        let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| -> Answer {
            let mut s = st.lock().unwrap();
            s.n += 1;
            if s.n > 300 {
                return Answer::Decline;
            }
            if &*r.kind == "priority" {
                return Answer::Action(json!({"do": "pass"}));
            }
            if r.prompt.starts_with("Rings of Brighthearth: pay {2} to copy") {
                s.asked_rings = true;
                return Answer::Pick(0);
            }
            if r.prompt.ends_with("which ability?") && !s.answered_which {
                s.answered_which = true;
                return Answer::Pick(r.choices.iter().position(|c| *c == want).unwrap_or(0));
            }
            if r.choices.is_empty() {
                return Answer::Yes;
            }
            // the first choice that is not Rings itself (a sacrifice or a target must not remove the copier)
            Answer::Pick(r.choices.iter().position(|c| !c.contains("Rings of Brighthearth")).unwrap_or(0))
        }));
        g.humans.add(P0, h.clone());
        let idx = if land {
            g.player(P0).lands.iter().position(|&x| x == m).unwrap()
        } else {
            g.player(P0).perms.iter().position(|&x| x == m).unwrap()
        };
        let before = g.stack_pushes;
        let act = if land { json!({"do": "use", "land": idx}) } else { json!({"do": "use", "perm": idx}) };
        let res = human::apply(&mut g, P0, &act);
        let why = match &res {
            Ok(Some(w)) => w.clone(),
            _ => String::new(),
        };
        let pushed = g.stack_pushes > before;
        let s = state.lock().unwrap();
        let log = g.log.clone().unwrap_or_default();
        if std::env::var("PROBE_SHOW").is_ok_and(|v| format!("{key}: {name} #{k}").contains(&v)) {
            eprintln!(
                "--- {key}: {name} #{k} ({step:?}) asked: {:#?}\n{}",
                h.asked().iter().map(|r| (&*r.kind, r.prompt.clone(), r.choices.len())).collect::<Vec<_>>(),
                log.join("\n")
            );
        }
        let copied = log.iter().any(|l| l.contains("the copy of") && l.contains("resolves"));
        if let Err(e) = res {
            return Out::Panicked(format!("{e:?}"));
        }
        if !g.trig_copies.is_empty() {
            return Out::NoCopy;
        }
        if s.asked_rings {
            if copied { Out::Offered } else { Out::NoCopy }
        } else if pushed {
            Out::NotOffered
        } else {
            let told: Vec<String> = h.told().into_iter().map(|t| t.1).collect();
            Out::NotExercised(format!("{label}: {}{}", told.first().cloned().unwrap_or_default(), why))
        }
    }));
    r.unwrap_or_else(|e| {
        Out::Panicked(
            e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default(),
        )
    })
}

/// how many abilities does the card's menu list (first main phase)
fn menu_len(key: &str, name: &str, land: bool) -> usize {
    let r = catch_unwind(AssertUnwindSafe(|| -> usize {
        let mut g: Game = table(&[key, "seph"]);
        let m = place(&mut g, name, land);
        board(&mut g);
        prep(&mut g, name, m);
        g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
        g.active = Some(P0);
        g.step = Step::Main1;
        if land {
            abilities::land_abilities(&g, P0, m).len() + alt::land_alts(&mut g, P0, m).map(|v| v.len()).unwrap_or(0)
        } else {
            human::abilities_of(&mut g, P0, m).map(|v| v.len()).unwrap_or(0)
        }
    }));
    r.unwrap_or(0)
}

/// every ability of every card in `decks`, activated: "deck: card #k" -> what happened (the best of the two phases)
fn sweep(decks: &[&str]) -> BTreeMap<String, Out> {
    let all = json_file("decks.json");
    let db = db();
    let mut out = BTreeMap::new();
    for d in all["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        if !decks.contains(&key) {
            continue;
        }
        let mut names: Vec<String> = d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(c) = db.id(&name) else { continue };
            let dd = db.get(c);
            if !(dd.perm || dd.land) {
                continue;
            }
            std::panic::set_hook(Box::new(|_| {}));
            let n = menu_len(key, &name, dd.land);
            for k in 0..n {
                let mut best: Option<Out> = None;
                for step in [Step::Main1, Step::Main2] {
                    let o = run_case(key, &name, dd.land, k, step);
                    best = Some(match (best, o) {
                        (None, o) => o,
                        (Some(Out::Offered), _) | (_, Out::Offered) => Out::Offered,
                        (Some(Out::NotExercised(_)), o) => o,
                        (Some(b), Out::NotExercised(_)) => b,
                        (Some(b), _) => b,
                    });
                    if best == Some(Out::Offered) {
                        break;
                    }
                }
                out.insert(format!("{key}: {name} #{k}"), best.unwrap());
            }
            let _ = std::panic::take_hook();
        }
    }
    out
}

/// abilities that are not offered Rings' copy, or not exercised by the probe, each with why. These are decisions for
/// JD: converting one means the ability's effect becomes re-runnable (see `rust/agents/report-model-gaps-b2.md`).
/// A key here that is now offered (or no longer found) fails the test: take it off the list.
const EXCEPTIONS: &[(&str, &str)] = &[];

#[test]
fn every_ability_a_person_can_use_offers_the_rings_copy() {
    let res = sweep(DECKS);
    let mut bad: Vec<String> = vec![];
    let mut stale: Vec<String> = vec![];
    let mut offered = 0;
    for (k, v) in &res {
        let listed = EXCEPTIONS.iter().find(|e| k.starts_with(e.0));
        match (v, listed) {
            (Out::Offered, None) => offered += 1,
            (Out::Offered, Some(e)) => stale.push(format!("{k}: now offered, take {:?} off EXCEPTIONS", e.0)),
            (v, None) => bad.push(format!("{k}: {v:?}")),
            (_, Some(_)) => {}
        }
    }
    for e in EXCEPTIONS {
        if !res.keys().any(|k| k.starts_with(e.0)) {
            stale.push(format!("{:?} is not an ability any more: take it off EXCEPTIONS", e.0));
        }
    }
    eprintln!("{offered} abilities offered Rings' copy; {} exception(s)", EXCEPTIONS.len());
    for b in &bad {
        eprintln!("NOT OK  {b}");
    }
    for s in &stale {
        eprintln!("STALE   {s}");
    }
    assert!(bad.is_empty() && stale.is_empty(), "{} ability(ies) without a copy, {} stale exception(s): see above", bad.len(), stale.len());
}
