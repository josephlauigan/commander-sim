//! Practice-mode Rings of Brighthearth probe (model-gaps-b2, 2026-11-05): a person with Rings of Brighthearth and {2}
//! to spare activates every non-mana ability the owned decks give them, and each one that goes on the stack must ask
//! "Rings of Brighthearth: pay {2} to copy it?"; a "yes" must resolve a copy (the engine logs "the copy of X resolves"
//! when it carries a copy's effect out, and no copy may be left waiting in `trig_copies`).
//!
//! Each card is put on the battlefield (or into the hand or graveyard, for the abilities it has there: transmute,
//! unearth, channel ...) of a fresh game, untapped and not summoning sick, with lands, mana, creatures, artifacts and
//! graveyard cards around it, so abilities that need a target or a cost can be paid (`prep` adds what one card needs).
//! Every ability its menu lists is activated in turn (first main phase, then second main phase) with a scripted answer
//! for every question: the first choice (never Rings itself), "yes" to Rings, a pass for every priority.
//!
//! A new card whose ability is not offered, or is offered and not carried out, fails the test. A card in `EXCEPTIONS`
//! is not an activated ability Rings can copy (a spell cast another way, a setting, a mana ability, a table shortcut)
//! or needs a position the probe cannot set up; each has its reason, and JD decides whether it stays.

use serde_json::{Value, json};
use sim_core::control::{Answer, Request};
use sim_core::engine::zones::{self, Enter};
use sim_core::hooks::Opt;
use sim_core::human::script::ScriptHuman;
use sim_core::human::{abilities, alt, human};
use sim_core::ids::{CardId, PermId, PlayerId};
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
    &["karn", "thrasios", "alela", "cloud", "galadriel", "aragorn", "sheoldred", "thanos", "urdragon", "yshtola"];

/// where the ability is used from
#[derive(Debug, Clone, Copy, PartialEq)]
enum Spot {
    /// a permanent (the flag: a land)
    Battlefield(bool),
    /// the hand: transmute, channel ...
    Hand,
    /// the graveyard: unearth ...
    Graveyard,
}

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

thread_local! {
    /// the second board: an empty hand, no spare permanents and little mana, where the AI's own conditions for using an
    /// ability (a small hand, mana to spare ...) differ from the first board's
    static LEAN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// the board every case starts from
fn board(g: &mut Game) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, P0, b, 3, false);
    }
    perm(g, P0, "Rings of Brighthearth");
    if LEAN.with(|l| l.get()) {
        token(g, P0, 2);
        token(g, P1, 2);
        pool(g, P0, &[('W', 2), ('U', 2), ('B', 2), ('R', 14), ('G', 2), ('C', 4)]);
        return;
    }
    token(g, P0, 2);
    let big = token(g, P0, 4);
    g.perm_mut(big).ttypes.push("assembly-worker");
    g.in_combat.push(big); // an attacking creature (Foundry, Angelic Page ...)
    for _ in 0..6 {
        token(g, P0, 1); // ten nonland permanents to sacrifice (Bolas's Citadel)
    }
    token(g, P1, 2);
    perm(g, P1, "Serra Angel"); // white
    perm(g, P1, "Mogg Fanatic"); // red
    perm(g, P1, "Vampire Nighthawk"); // black
    perm(g, P0, "Thrasios, Triton Hero"); // a legendary creature (Plaza of Heroes)
    perm(g, P0, "Sol Ring");
    perm(g, P0, "Mind Stone");
    perm(g, P0, "Arcane Signet");
    perm(g, P1, "Mind Stone");
    for n in [
        "Mountain",
        "Island",
        "Lightning Bolt",
        "Counterspell",
        "Sol Ring",
        "Llanowar Elves",
        "Skullclamp",              // an Equipment card (Cid)
        "Lin Sivvi, Defiant Hero", // Rebels (Galadriel's deck)
        "Ramosian Revivalist",
    ] {
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

/// what one permanent needs besides the board
fn prep(g: &mut Game, name: &str, m: PermId) {
    if g.perm(m).loyalty.is_some() {
        g.perm_mut(m).loyalty = Some(20); // every loyalty ability, the ultimates too
    }
    match name {
        "Nevinyrral's Disk" => g.perm_mut(m).tapped = false,
        "Grim Monolith" | "Basalt Monolith" => g.perm_mut(m).tapped = true, // their untap needs it tapped
        "Steel Hellkite" => {
            let st = g.turn_stamp();
            g.perm_mut(m).data.set(DataKey::Hit, Val::List(vec![Val::Stamp(st), Val::Player(P1)]));
        }
        "Urza's Saga" => {
            g.player_mut(P0).sagas.retain(|x| x.0 != m);
            g.player_mut(P0).sagas.push((m, 2));
        }
        "Otherworld Atlas" => g.perm_mut(m).data.set(DataKey::Charge, Val::Int(2)),
        "Dragon's Hoard" => g.perm_mut(m).data.set(DataKey::T45Dragon, Val::Int(2)),
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

/// an ability from the hand or graveyard (not a way to cast the card: transmute, channel, unearth, cycling ...)
fn activated_way(label: &str) -> bool {
    let l = label.to_lowercase();
    [
        "unearth",
        "eternalize",
        "embalm",
        "encore",
        "scavenge",
        "cycling",
        "channel",
        "transmute",
        "ninjutsu",
        "bloodrush",
    ]
    .iter()
    .any(|k| l.contains(k))
}

/// the card c put in P0's hand or graveyard, with the activated ways its hooks give a person
fn ways(g: &mut Game, name: &str, spot: Spot) -> (CardId, Vec<(String, Opt)>) {
    let c = take(g, P0, name);
    let v = if spot == Spot::Hand {
        g.player_mut(P0).hand.push(c);
        alt::hand_alts(g, P0, c)
    } else {
        g.player_mut(P0).gy.push(c);
        alt::gy_alts(g, P0, c)
    };
    (c, v.unwrap_or_default().into_iter().filter(|w| activated_way(&w.0)).collect())
}

struct Probe {
    asked_rings: bool,
    answered_which: bool,
    n: usize,
}

/// one ability of one card, activated once
fn run_case(key: &str, name: &str, spot: Spot, k: usize, step: Step) -> Out {
    let r = catch_unwind(AssertUnwindSafe(|| -> Out {
        let mut g: Game = table(&[key, "seph"]);
        g.log = Some(vec![]);
        let land = spot == Spot::Battlefield(true);
        let m = if matches!(spot, Spot::Battlefield(_)) { Some(place(&mut g, name, land)) } else { None };
        board(&mut g);
        if let Some(m) = m {
            prep(&mut g, name, m);
            let on_bf = g.player(P0).perms.contains(&m) || g.player(P0).lands.contains(&m);
            if !on_bf {
                return Out::NotExercised("left the battlefield on entering".into());
            }
        }
        g.active = Some(P0);
        g.step = step;
        let state = Arc::new(Mutex::new(Probe { asked_rings: false, answered_which: false, n: 0 }));
        let st = state.clone();
        // the label of ability k: fetched with the person seated (the menus read the seat)
        g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
        let (label, way) = match m {
            Some(m) => {
                let labels: Vec<String> = if land {
                    let mut v: Vec<String> =
                        abilities::land_abilities(&g, P0, m).into_iter().map(|a| a.label).collect();
                    v.extend(alt::land_alts(&mut g, P0, m).unwrap_or_default().into_iter().map(|a| a.0));
                    v
                } else {
                    human::abilities_of(&mut g, P0, m).map(|v| v.into_iter().map(|a| a.0).collect()).unwrap_or_default()
                };
                let Some(label) = labels.get(k).cloned() else {
                    return Out::NotExercised(format!("no such ability (the menu: {labels:?})"));
                };
                (label, None)
            }
            None => {
                let (c, mut v) = ways(&mut g, name, spot);
                if k >= v.len() {
                    return Out::NotExercised("no such way".into());
                }
                let (label, opt) = v.remove(k);
                (label, Some((c, opt)))
            }
        };
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
            // an Equipment goes on the last creature (not the Germ it came with); otherwise the first choice that is not
            // Rings itself (a sacrifice or a target must not remove the copier)
            if r.prompt.starts_with("Equip ") {
                return Answer::Pick(r.choices.len().saturating_sub(2));
            }
            Answer::Pick(r.choices.iter().position(|c| !c.contains("Rings of Brighthearth")).unwrap_or(0))
        }));
        g.humans.add(P0, h.clone());
        let before = g.stack_pushes;
        let res = match (m, way) {
            (Some(m), _) => {
                let idx = if land {
                    g.player(P0).lands.iter().position(|&x| x == m).unwrap()
                } else {
                    g.player(P0).perms.iter().position(|&x| x == m).unwrap()
                };
                let act = if land { json!({"do": "use", "land": idx}) } else { json!({"do": "use", "perm": idx}) };
                human::apply(&mut g, P0, &act)
            }
            (None, Some((c, opt))) => alt::run(&mut g, P0, c, &opt),
            (None, None) => unreachable!(),
        };
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
            e.downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default(),
        )
    })
}

/// how many abilities does the card's menu list at `spot` (first main phase)
fn menu_len(key: &str, name: &str, spot: Spot) -> usize {
    let r = catch_unwind(AssertUnwindSafe(|| -> usize {
        let mut g: Game = table(&[key, "seph"]);
        let land = spot == Spot::Battlefield(true);
        let m = if matches!(spot, Spot::Battlefield(_)) { Some(place(&mut g, name, land)) } else { None };
        board(&mut g);
        g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
        g.active = Some(P0);
        g.step = Step::Main1;
        match m {
            Some(m) => {
                prep(&mut g, name, m);
                if land {
                    abilities::land_abilities(&g, P0, m).len()
                        + alt::land_alts(&mut g, P0, m).map(|v| v.len()).unwrap_or(0)
                } else {
                    human::abilities_of(&mut g, P0, m).map(|v| v.len()).unwrap_or(0)
                }
            }
            None => ways(&mut g, name, spot).1.len(),
        }
    }));
    r.unwrap_or(0)
}

/// every ability of every card in `decks`, activated: "deck: card #k" (or "... [hand]" / "... [graveyard]") -> what
/// happened (the best of the two phases)
fn sweep(decks: &[&str]) -> BTreeMap<String, Out> {
    let all = json_file("decks.json");
    let db = db();
    let mut out = BTreeMap::new();
    for d in all["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        if !decks.contains(&key) {
            continue;
        }
        let mut names: Vec<String> =
            d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(c) = db.id(&name) else { continue };
            let dd = db.get(c);
            let mut spots = vec![Spot::Hand, Spot::Graveyard];
            if dd.perm || dd.land {
                spots.push(Spot::Battlefield(dd.land));
            }
            for spot in spots {
                std::panic::set_hook(Box::new(|_| {}));
                let n = menu_len(key, &name, spot);
                for k in 0..n {
                    let mut best: Option<Out> = None;
                    for step in [Step::Main1, Step::Main2] {
                        let o = run_case(key, &name, spot, k, step);
                        best = Some(match (best, o) {
                            (None, o) => o,
                            (Some(Out::Offered), _) | (_, Out::Offered) => Out::Offered,
                            (Some(Out::NotExercised(a)), Out::NotExercised(b)) => {
                                Out::NotExercised(if a.starts_with("no such") { b } else { a })
                            }
                            (Some(Out::NotExercised(_)), o) => o,
                            (Some(b), Out::NotExercised(_)) => b,
                            (Some(b), _) => b,
                        });
                        if best == Some(Out::Offered) {
                            break;
                        }
                    }
                    let tag = match spot {
                        Spot::Battlefield(_) => String::new(),
                        Spot::Hand => " [hand]".into(),
                        Spot::Graveyard => " [graveyard]".into(),
                    };
                    out.insert(format!("{key}: {name}{tag} #{k}"), best.unwrap());
                }
                let _ = std::panic::take_hook();
            }
        }
    }
    out
}

/// abilities that are not offered Rings' copy, each with why. They are decisions for JD (see
/// `rust/agents/report-model-gaps-b2.md`): not one of them is an activated ability Rings could copy, or the probe cannot
/// set the position up. An entry that is now offered (or no longer found) fails the test: take it off the list.
const EXCEPTIONS: &[(&str, &str)] = &[
    (
        "thrasios: Basalt Monolith #0",
        "\"start the loop\": a table shortcut standing for the whole Basalt Monolith + Rings loop (each pass of it is a real activation Rings copies, see play_rings.rs); not an ability itself",
    ),
    (
        "thanos: Basalt Monolith #0",
        "\"start the loop\": a table shortcut standing for the whole Basalt Monolith + Rings loop (each pass of it is a real activation Rings copies, see play_rings.rs); not an ability itself",
    ),
    ("aragorn: Niv-Mizzet, Parun #0", "a setting (where its pings go), not an ability"),
    (
        "sheoldred: Bolas's Citadel #0",
        "playing the top card of the library (Citadel's static permission): a spell cast, not an ability",
    ),
    ("sheoldred: Phyrexian Tower #0", "a mana ability (rule 605): Rings does not copy it"),
    ("thrasios: Lion's Eye Diamond #0", "a mana ability (rule 605): Rings does not copy it"),
];

/// both boards: a key is bad if either board shows it unanswered, offered if either board offers it
fn both_boards() -> BTreeMap<String, Out> {
    let mut res = sweep(DECKS);
    LEAN.with(|l| l.set(true));
    let lean = sweep(DECKS);
    LEAN.with(|l| l.set(false));
    for (k, v) in lean {
        let merged = match (res.remove(&k), v) {
            (None, v) => v,
            (Some(a), b) => {
                let rank = |o: &Out| match o {
                    Out::Panicked(_) => 4,
                    Out::NotOffered => 3,
                    Out::NoCopy => 2,
                    Out::Offered => 1,
                    Out::NotExercised(_) => 0,
                };
                // the worse of the two shows, except that a board where nothing could be exercised says nothing
                if matches!(a, Out::NotExercised(_)) {
                    b
                } else if matches!(b, Out::NotExercised(_)) {
                    a
                } else if rank(&a) >= rank(&b) {
                    a
                } else {
                    b
                }
            }
        };
        res.insert(k, merged);
    }
    res
}

#[test]
fn every_ability_a_person_can_use_offers_the_rings_copy() {
    let res = both_boards();
    let mut bad: Vec<String> = vec![];
    let mut stale: Vec<String> = vec![];
    let mut offered = 0;
    for (k, v) in &res {
        let listed = EXCEPTIONS.iter().find(|e| k == e.0);
        match (v, listed) {
            (Out::Offered, None) => offered += 1,
            (Out::Offered, Some(e)) => stale.push(format!("{k}: now offered, take {:?} off EXCEPTIONS", e.0)),
            (v, None) => bad.push(format!("{k}: {v:?}")),
            (_, Some(_)) => {}
        }
    }
    for e in EXCEPTIONS {
        if !res.contains_key(e.0) {
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
    assert!(
        bad.is_empty() && stale.is_empty(),
        "{} ability(ies) without a copy, {} stale exception(s): see above",
        bad.len(),
        stale.len()
    );
}

/// the karn and thrasios decks (the ones with Rings of Brighthearth) are swept by the test above; this one says so
/// outright, so removing them from `DECKS` fails
#[test]
fn the_sweep_covers_the_decks_with_rings() {
    assert!(DECKS.contains(&"karn") && DECKS.contains(&"thrasios"));
}

/// (scan) the permanents of the owned decks whose abilities reach a person through the AI's `options` hook (no person
/// path of their own in `cards::abilities`, karn.rs or cloud.rs): their `ability_window` calls are the AI's too
#[test]
#[ignore]
fn scan_option_hooks() {
    let all = json_file("decks.json");
    let db = db();
    for d in all["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let mut names: Vec<String> =
            d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(c) = db.id(&name) else { continue };
            let dd = db.get(c);
            if !(dd.perm || dd.land) {
                continue;
            }
            let r = catch_unwind(AssertUnwindSafe(|| -> String {
                let mut g: Game = table(&[key, "seph"]);
                g.humans.add(P0, Arc::new(ScriptHuman::new(vec![])));
                let m = place(&mut g, &name, dd.land);
                let imp = g
                    .registry
                    .get(c)
                    .map(|i| (i.options.is_some(), i.land_options.is_some()))
                    .unwrap_or((false, false));
                let own = if dd.land {
                    sim_core::human::karn::owns_land(&g, m)
                } else {
                    sim_core::human::cards::abilities(&g, P0, m).is_some()
                        || sim_core::human::karn::owns_perm(&g, m)
                        || sim_core::human::cloud::owns_perm(&g, m)
                };
                format!("own={own} options={} land_options={}", imp.0, imp.1)
            }));
            if let Ok(s) = r {
                if s.contains("options=true") || s.contains("land_options=true") {
                    eprintln!("HOOK {key}: {name} {s}");
                }
            }
        }
    }
}
