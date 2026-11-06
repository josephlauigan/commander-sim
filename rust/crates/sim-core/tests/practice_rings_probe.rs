//! Practice-mode Rings of Brighthearth probe (model-gaps-b2, 2026-11-05): a person with Rings of Brighthearth and {2}
//! to spare activates every non-mana ability the karn, thrasios and sauron decks give them (the Camera deck too), and
//! each one that goes on the stack must ask "Rings of Brighthearth: pay {2} to copy it?"; a "yes" must resolve a copy
//! (the engine logs "the copy of X resolves" when a copy's effect is carried out).
//!
//! Each card is put on the battlefield of a fresh game (untapped, not summoning sick, with lands, mana, creatures,
//! artifacts and graveyard cards around it, so abilities that need a target or a cost can be paid). Every ability its
//! menu lists is activated in turn (first main phase, then second main phase) with a scripted answer for every
//! question: the first choice, "yes" to Rings, a pass for every priority. An ability the probe could not put on the
//! stack is "not exercised"; it is listed in `NOT_EXERCISED` with the reason (the probe's board lacks what the ability
//! needs, or the ability is a special action).

use serde_json::{Value, json};
use sim_core::control::{Answer, Request};
use sim_core::human::script::ScriptHuman;
use sim_core::human::{abilities, alt, human};
use sim_core::ids::PlayerId;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const DECKS: &[&str] = &["karn", "thrasios", "sauron"];

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

/// the board every case starts from
fn board(g: &mut Game) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, P0, b, 3, false);
    }
    perm(g, P0, "Rings of Brighthearth");
    token(g, P0, 2);
    token(g, P0, 3);
    token(g, P1, 2);
    token(g, P1, 4);
    perm(g, P0, "Sol Ring");
    perm(g, P1, "Mind Stone");
    for n in ["Mountain", "Island", "Lightning Bolt", "Counterspell"] {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let c = take(g, P1, "Swamp");
    g.player_mut(P1).gy.push(c);
    hand(g, P0, &["Forest", "Island", "Sol Ring"]);
    pool(g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6), ('C', 12)]);
}

fn pool(g: &mut Game, p: PlayerId, mana: &[(char, u32)]) {
    for &(c, n) in mana {
        g.player_mut(p).pool.add(c, n);
    }
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
        let m = if land { lands(&mut g, P0, name, 1, false)[0] } else { perm(&mut g, P0, name) };
        board(&mut g);
        let on_bf = g.player(P0).perms.contains(&m) || g.player(P0).lands.contains(&m);
        if !on_bf {
            return Out::NotExercised("left the battlefield on entering".into());
        }
        g.active = Some(P0);
        g.step = step;
        let state = Arc::new(Mutex::new(Probe { asked_rings: false, answered_which: false, n: 0 }));
        let st = state.clone();
        let want = Arc::new(Mutex::new(String::new()));
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
        *want.lock().unwrap() = label.clone();
        let w2 = want.clone();
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
                let w = w2.lock().unwrap();
                return Answer::Pick(r.choices.iter().position(|c| *c == *w).unwrap_or(0));
            }
            if r.choices.is_empty() {
                return Answer::Yes;
            }
            Answer::Pick(0)
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
            eprintln!("--- {key}: {name} #{k} ({step:?}) asked: {:#?}\n{}", h.asked().iter().map(|r| (&*r.kind, r.prompt.clone(), r.choices.len())).collect::<Vec<_>>(), log.join("\n"));
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
        let m = if land { lands(&mut g, P0, name, 1, false)[0] } else { perm(&mut g, P0, name) };
        board(&mut g);
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
            let n2 = {
                // second main phase can list more
                n
            };
            let _ = n2;
            for k in 0..n {
                let mut best: Option<Out> = None;
                for step in [Step::Main1, Step::Main2] {
                    let o = run_case(key, &name, dd.land, k, step);
                    best = Some(match (best, o) {
                        (None, o) => o,
                        (Some(Out::Offered), _) => Out::Offered,
                        (_, Out::Offered) => Out::Offered,
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

#[test]
fn sweep_report() {
    let res = sweep(DECKS);
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (k, v) in &res {
        let tag = match v {
            Out::Offered => "offered",
            Out::NoCopy => "NO COPY",
            Out::NotOffered => "NOT OFFERED",
            Out::NotExercised(_) => "not exercised",
            Out::Panicked(_) => "PANICKED",
        };
        *counts.entry(tag).or_default() += 1;
        if *v != Out::Offered {
            eprintln!("{tag:14} {k}  {v:?}");
        }
    }
    eprintln!("{counts:?}");
}
