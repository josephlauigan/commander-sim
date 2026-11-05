//! Practice-mode trigger probe (2026-11-04): every card in the owned decks (`data/decks.json` "mine") that is a
//! permanent with a triggered line ("When", "Whenever", "At the beginning", a Saga chapter) other than "when ... enters"
//! (the choice probe covers that), whose text implies a choice ("search", "choose", "target", "you may", "sacrifice a",
//! ...), is put onto the battlefield for a seat played by a person, on a table with two of everything. The moment the
//! trigger waits for is then driven through the engine's own entry points (`turn::step_start`, `combat::combat`,
//! `turn::continue_turn`, `zones::die`, `life::gain`, `zones::draw`, the person's main phase with a land and two spells ...),
//! so the Tag-driven engine triggers are covered too.
//!
//! A card passes a moment if its trigger did not fire there, or fired and the person was asked something. It is flagged
//! when the trigger fired (a "Your trigger: <card>" priority window, or the card's name in the game log) and the person was
//! asked nothing: the AI's code chose for the person. Cards whose trigger never fired in any driven moment are listed as
//! unprobed with the reason. The activated abilities' resolution is the ability probe's and the counter probe's business.
//!
//! Limits: a card with two triggers in the same driven moment, one asking and one silent, is not flagged; the person is
//! always the first seat; every question is answered with the first choice (or yes).

mod play_common;
use play_common::*;
use sim_core::control::Human;
use sim_core::engine::{combat, life, turn, zones};
use sim_core::state::Step;
use sim_core::sym::intern;
use std::collections::{BTreeMap, BTreeSet};
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

fn is_roman(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | ',' | ' '))
}

/// a triggered line that implies a choice: the whole line and its condition (the words before the first comma: "whenever
/// you cast a spell"; a Saga chapter's is "chapter"), both lowercased, quoted text taken out of the condition
struct TLine {
    full: String,
    cond: String,
}

/// the triggered lines of a card that imply a choice, with an ability word ("Landfall — ") or a Saga's chapter marker
/// ("I, II — ") taken off the front
fn trigger_lines(text: &str) -> Vec<TLine> {
    text.lines()
        .map(str::trim)
        .filter_map(|l| {
            let mut l = l.to_string();
            let mut chapter = false;
            if let Some((pre, rest)) = l.split_once(" — ") {
                if is_roman(pre) {
                    chapter = true;
                    l = rest.to_string();
                } else if pre.len() < 28 && !pre.contains(',') && !pre.contains('{') {
                    l = rest.to_string(); // an ability word
                }
            }
            let lc = l.to_lowercase();
            let trig = chapter || lc.starts_with("when") || lc.starts_with("at the") || lc.starts_with("whenever");
            if !trig {
                return None;
            }
            // the condition: up to the first comma, outside quotes ("gains \"when this creature dies, ...\"" is not it)
            let mut cond = String::new();
            let mut quoted = false;
            for ch in lc.chars() {
                match ch {
                    '"' => quoted = !quoted,
                    ',' if !quoted => break,
                    _ if !quoted => cond.push(ch),
                    _ => {}
                }
            }
            let cond = if chapter { "chapter".to_string() } else { cond };
            let self_etb = cond.starts_with("when ")
                && cond.contains(" enters")
                && !["attack", "dies", "leaves", "whenever", " cast", " draw"].iter().any(|w| cond.contains(w));
            if self_etb {
                return None;
            }
            let permission =
                ["you may cast", "you may play", "choose new targets", "without paying"].iter().any(|w| lc.contains(w));
            if !WORDS.iter().any(|w| lc.contains(w) && !(permission && *w == "you may")) {
                return None;
            }
            Some(TLine { full: if chapter { format!("chapter {lc}") } else { lc }, cond })
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Sc {
    /// the person uses the card's k-th activated ability (the resolution, which the ability probe does not do)
    Activate(usize),
    Start,
    Main1,
    Combat,
    Main2End,
    OppTurn,
    DieSelf,
    DieOwn,
    DieOpp,
    SacOwn,
    GainLife,
    LoseLife,
    OppLife,
    Discard,
    Draw,
    OppDraw,
    Token,
    EnterOwn,
    EnterOpp,
    SelfEnter,
    Ring,
}

/// the moments to drive for these trigger conditions, and the lines no moment fits
fn scenarios(lines: &[TLine]) -> (BTreeSet<Sc>, Vec<String>) {
    let mut out = BTreeSet::new();
    let mut none = vec![];
    for t in lines {
        let lc = &t.cond;
        let n0 = out.len();
        let has = |w: &str| lc.contains(w);
        // another player's turn: "each opponent's upkeep", "an opponent casts", "attacks you"
        let opp = [
            "each opponent",
            "an opponent",
            "each player",
            "each upkeep",
            "each end step",
            "each combat",
            "attacks you",
            "opponent casts",
            "opponent draws",
        ]
        .iter()
        .any(|w| has(w));
        if lc == "chapter" || has("upkeep") || has("draw step") || has("precombat main") {
            out.insert(Sc::Start);
            if opp {
                out.insert(Sc::OppTurn);
            }
        }
        if has("postcombat main") || has("second main") || has("end step") {
            out.insert(Sc::Main2End);
            if opp {
                out.insert(Sc::OppTurn);
            }
        }
        if has("combat") || has("attack") || has("deals damage") || has("blocks") || has("becomes blocked") {
            out.insert(Sc::Combat);
            if opp {
                out.insert(Sc::OppTurn);
            }
        }
        if has("dies") || has("graveyard from the battlefield") || has("leaves the battlefield") || has("is destroyed")
        {
            out.extend([Sc::DieSelf, Sc::DieOwn, Sc::DieOpp]);
        }
        if has(" cast") || has("landfall") || has("play a land") || (has("land") && has("enter")) || has("you play") {
            out.insert(Sc::Main1);
            if opp {
                out.insert(Sc::OppTurn);
            }
        }
        if has("draw") && !lc.starts_with("at the") {
            out.extend([Sc::Draw, Sc::Start]);
            if has("opponent") {
                out.extend([Sc::OppDraw, Sc::OppTurn]);
            }
        }
        if has("life") {
            out.extend([Sc::GainLife, Sc::LoseLife, Sc::OppLife]);
        }
        if has("damage") && !has("combat damage") {
            out.extend([Sc::LoseLife, Sc::OppLife]);
        }
        if has("discard") {
            out.insert(Sc::Discard);
        }
        if has("sacrifice") {
            out.insert(Sc::SacOwn);
        }
        if has("ring-bearer") || has("ring tempts") {
            out.insert(Sc::Ring);
        }
        if has("token") {
            out.insert(Sc::Token);
        }
        if has("enter") {
            let selfish = (has(" enters or ") || has(" enters and ")) && !has("another") && !has("one or more");
            if !selfish {
                out.extend([Sc::EnterOwn, Sc::EnterOpp]);
            }
            out.insert(Sc::SelfEnter);
        }
        if out.len() == n0 {
            none.push(t.full.clone());
        }
    }
    (out, none)
}

struct Rec {
    kind: String,
    prompt: String,
    choices: Vec<String>,
}

/// the person: records everything, answers with the first choice, passes priority; in `script` mode its main phase
/// plays a land and casts two spells
struct Probe {
    script: bool,
    act: Mutex<Option<Value>>,
    want: String,
    n_abil: Mutex<usize>,
    n_main: Mutex<usize>,
    reqs: Mutex<Vec<Rec>>,
    told: Mutex<Vec<(String, String)>>,
}

impl Human for Probe {
    fn ask(&self, _g: &Game, _seat: PlayerId, req: Request) -> Answer {
        let n = {
            let mut r = self.reqs.lock().unwrap();
            r.push(Rec { kind: req.kind.to_string(), prompt: req.prompt.clone(), choices: req.choices.clone() });
            r.len()
        };
        if n > 3000 {
            return Answer::Cancel;
        }
        match &*req.kind {
            "priority" => {
                if req.prompt.starts_with("Main phase") || req.prompt.starts_with("Your turn: you have priority") {
                    if let Some(a) = self.act.lock().unwrap().take() {
                        return Answer::Action(a);
                    }
                    let mut k = self.n_main.lock().unwrap();
                    let i = *k;
                    *k += 1;
                    if self.script && i == 0 {
                        return Answer::Action(json!({"do": "land", "card": 0}));
                    }
                    if self.script && (1..=4).contains(&i) {
                        return Answer::Action(json!({"do": "cast", "card": 0}));
                    }
                }
                Answer::Action(json!({"do": "pass"}))
            }
            "attack" => Answer::Picks((0..req.choices.len()).collect()),
            "block" => Answer::Picks(vec![]),
            _ if req.prompt.ends_with("which ability?") => {
                let mut n = self.n_abil.lock().unwrap();
                *n = (*n).max(req.choices.len());
                Answer::Pick(pick_ability(&req.choices, &self.want))
            }
            _ if req.choices.is_empty() => Answer::Yes,
            _ => Answer::Pick(0),
        }
    }
    fn tell(&self, _seat: PlayerId, kind: sim_core::sym::Sym, text: &str) {
        self.told.lock().unwrap().push((kind.to_string(), text.to_string()));
    }
    fn autopass(&self) -> Option<&'static str> {
        Some("all")
    }
}

struct Outcome {
    n_abil: usize,
    fired: bool,
    sig: Vec<i64>,
    asked: Vec<String>,
    autos: usize,
    error: Option<String>,
}

fn fixture<'a>(name: &str, a: &'a str, b: &'a str) -> &'a str {
    if name == a { b } else { a }
}

/// what is on the table afterwards, compared with a run where `put` (a blank permanent) stands in for the card
fn signature(g: &Game, ignore_life: bool) -> Vec<i64> {
    let mut v = vec![];
    for p in [P0, P1] {
        let q = g.player(p);
        v.extend([
            if ignore_life { 0 } else { q.life as i64 },
            q.hand.len() as i64,
            q.library.len() as i64,
            q.gy.len() as i64,
            q.exile.len() as i64,
            q.perms.len() as i64,
            q.lands.len() as i64,
        ]);
        v.push(q.perms.iter().map(|&m| g.perm(m).plus as i64).sum());
    }
    v
}

fn run(key: &str, name: &str, put: &str, land: bool, saga: bool, equip: bool, sc: Sc, want: &str) -> Outcome {
    let mut g: Game = table(&[key, "seph", "thanos"]);
    g.log = Some(vec![]);
    g.active = Some(P0);
    g.step = Step::Main1;
    let elves = fixture(name, "Llanowar Elves", "Shivan Dragon");
    let birds = fixture(name, "Birds of Paradise", "Shivan Dragon");
    let e0 = perm(&mut g, P0, elves);
    let b0 = perm(&mut g, P0, birds);
    perm(&mut g, P0, "Bonesplitter");
    lands(&mut g, P0, "Island", 3, false);
    let e1 = perm(&mut g, P1, elves);
    let b1 = perm(&mut g, P1, birds);
    for n in ["Sol Ring", "Arcane Signet", "Doubling Season"] {
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
        (P0, "Llanowar Elves"),
        (P0, "Birds of Paradise"),
        (P1, "Grave Titan"),
        (P1, "Sol Ring"),
    ] {
        let c = take(&mut g, p, n);
        g.player_mut(p).gy.push(c);
    }
    let creature = fixture(name, "Llanowar Elves", "Birds of Paradise");
    let _ = hand(
        &mut g,
        P0,
        &["Forest", creature, "Lightning Bolt", "Brainstorm", "Mind Stone", "Plains", "Island", "Swamp"],
    );
    hand(&mut g, P1, &["Mind Stone", "Sakura-Tribe Elder"]);
    // equipment on top of the library (Armored Skyhunter, Gilgamesh look at the top six)
    let equipment_on_top = |g: &mut Game| {
        for n in ["Bonesplitter", "Lightning Greaves", "Swiftfoot Boots"] {
            let c = take(g, P0, n);
            g.player_mut(P0).library.push(c);
        }
    };
    if put != name {
        take(&mut g, P0, name); // the library is the same with the blank in the card's place
    }
    let script = sc == Sc::Main1;
    let h = Arc::new(Probe {
        want: want.to_string(),
        n_abil: Mutex::new(0),
        act: Mutex::new(None),
        script,
        n_main: Mutex::new(0),
        reqs: Mutex::new(vec![]),
        told: Mutex::new(vec![]),
    });
    if sc == Sc::SelfEnter {
        // the card's own enter trigger ("whenever this creature enters or attacks"): the person is seated as it enters
        equipment_on_top(&mut g);
        g.humans.add(P0, h.clone());
    }
    let m = if land { lands(&mut g, P0, put, 1, false)[0] } else { perm(&mut g, P0, put) };
    if equip {
        g.perm_mut(m).attached = Some(e0); // "equipped creature" / "enchanted creature"
    }
    if sc == Sc::Start && !g.is_creature(m) {
        g.perm_mut(m).tapped = true; // Mana Vault: "if it is tapped"
    }
    let on = |g: &Game| g.perm(m).on_bf;
    if !on(&g) {
        return Outcome {
            n_abil: 1,
            fired: false,
            sig: vec![],
            asked: vec![],
            autos: 0,
            error: Some("left the battlefield on entering".into()),
        };
    }
    if sc != Sc::SelfEnter {
        equipment_on_top(&mut g);
        g.humans.add(P0, h.clone());
        g.log = Some(vec![]);
    }
    if let Sc::Activate(_) = sc {
        let i = if land {
            g.player(P0).lands.iter().position(|&x| x == m)
        } else {
            g.player(P0).perms.iter().position(|&x| x == m)
        };
        *h.act.lock().unwrap() = Some(match (land, i) {
            (true, Some(i)) => json!({"do": "use", "land": i}),
            (false, Some(i)) => json!({"do": "use", "perm": i}),
            _ => json!({"do": "pass"}),
        });
    }
    if script || sc == Sc::EnterOwn || matches!(sc, Sc::Activate(_)) {
        pool(&mut g, P0, &[('W', 10), ('U', 10), ('B', 10), ('R', 10), ('G', 10), ('C', 10)]);
    }
    let before = signature(&g, sc == Sc::Combat);
    let destroy = intern("destroy");
    let r: sim_core::flow::Res = (|| {
        match sc {
            Sc::Start => {
                turn::step_start(&mut g, P0)?;
                if saga {
                    turn::step_start(&mut g, P0)?;
                }
            }
            Sc::Main1 | Sc::Activate(_) => {
                g.step = Step::Main1;
                sim_core::human::human::human_main(&mut g, P0, false)?;
            }
            Sc::Combat => {
                for t in [e1, b1] {
                    g.perm_mut(t).tapped = true; // nothing blocks: combat damage reaches the player
                }
                g.step = Step::Combat;
                combat::combat(&mut g, P0)?;
            }
            Sc::Main2End => turn::continue_turn(&mut g, P0, Step::Main2)?,
            Sc::OppTurn => {
                g.active = Some(P1);
                turn::continue_turn(&mut g, P1, Step::Start)?;
            }
            Sc::DieSelf => zones::die(&mut g, m, destroy)?,
            Sc::DieOwn => zones::die(&mut g, e0, destroy)?,
            Sc::DieOpp => zones::die(&mut g, e1, destroy)?,
            Sc::SacOwn => zones::die(&mut g, b0, intern("sac"))?,
            Sc::GainLife => life::gain(&mut g, P0, 3)?,
            Sc::LoseLife => life::lose_life(&mut g, P0, 2, Some(P1), intern("other"), None)?,
            Sc::OppLife => {
                life::gain(&mut g, P1, 3)?;
                life::lose_life(&mut g, P1, 2, Some(P0), intern("other"), None)?;
            }
            Sc::Discard => {
                let n = g.player(P0).hand.len();
                zones::discard_index(&mut g, P0, n - 1)?
            }
            Sc::Draw => zones::draw(&mut g, P0, 1, false)?,
            Sc::OppDraw => zones::draw(&mut g, P1, 1, false)?,
            Sc::Token => {
                zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 1))?;
            }
            Sc::EnterOwn => {
                perm(&mut g, P0, "Mind Stone");
                perm(&mut g, P0, "Lightning Greaves");
                perm(&mut g, P0, "Shivan Dragon");
                perm(&mut g, P0, "Sakura-Tribe Elder");
            }
            Sc::SelfEnter => {}
            Sc::Ring => sim_core::cardcode::ring_tempt(&mut g, P0)?,
            Sc::EnterOpp => {
                perm(&mut g, P1, "Mind Stone");
                perm(&mut g, P1, "Shivan Dragon");
            }
        }
        Ok(())
    })();
    let error = r.err().map(|e| format!("{e:?}"));
    let reqs = h.reqs.lock().unwrap();
    if std::env::var("PROBE_ALL").is_ok_and(|v| v == name) {
        for q in reqs.iter() {
            eprintln!("  REQ {sc:?} {} | {} | {:?}", q.kind, q.prompt, q.choices);
        }
    }
    let window = format!("Your trigger: {name}:");
    let fired = reqs.iter().any(|q| {
        (q.kind == "priority"
            && (q.prompt.contains(&window) || (q.prompt.starts_with("You activate") && q.prompt.contains(name))))
            || (q.kind != "priority" && q.prompt.starts_with(name))
    });
    // only the probed card's questions: not the structure of the probe's own plays, nor a prompt headed by another card
    let other =
        |p: &str| p.split_once(':').is_some_and(|(hd, _)| hd != name && !name.starts_with(hd) && g.db.id(hd).is_some());
    let asked: Vec<String> = reqs
        .iter()
        .filter(|q| {
            !matches!(&*q.kind, "priority" | "attack" | "block")
                && !q.prompt.starts_with("Attack which")
                && !q.prompt.starts_with("attacking ")
                && !q.prompt.ends_with("which ability?")
                && !other(&q.prompt)
        })
        .map(|q| {
            if std::env::var("PROBE_CHOICES").is_ok() {
                format!("{}: {} {:?}", q.kind, q.prompt, q.choices)
            } else {
                format!("{}: {}", q.kind, q.prompt)
            }
        })
        .collect();
    let autos = h.told.lock().unwrap().iter().filter(|t| t.0 == "auto").count();
    let ignore_life = sc == Sc::Combat;
    let sig: Vec<i64> = signature(&g, ignore_life).iter().zip(&before).map(|(a, b)| a - b).collect();
    let n_abil = (*h.n_abil.lock().unwrap()).max(1);
    Outcome { n_abil, fired, sig, asked, autos, error }
}

#[test]
fn every_trigger_choice_of_the_owned_decks_is_asked_of_a_person() {
    let cache = json("scryfall_cache.json");
    let decks = json("decks.json");
    let db = db();
    let mut silent: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut unprobed: Vec<String> = vec![];
    let mut unprobed_keys: BTreeSet<String> = BTreeSet::new();
    let (mut pairs, mut probed, mut asked_ok) = (0, 0, 0);
    let mut twice: Vec<String> = vec![];
    let mut cards: BTreeSet<String> = BTreeSet::new();
    for d in decks["mine"].as_array().unwrap() {
        let key = d["key"].as_str().unwrap();
        let mut names: Vec<String> =
            d["cards"].as_array().unwrap().iter().filter_map(|c| c.as_str().map(String::from)).collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(c) = db.id(&name) else { continue };
            let def = db.get(c);
            if !(def.perm || def.land) {
                continue;
            }
            let Some(text) = oracle(&cache, &name) else { continue };
            let lines = trigger_lines(&text);
            let acts = if std::env::var("PROBE_ACTIVATE").is_ok() { activated_lines(&text) } else { vec![] };
            if lines.is_empty() && acts.is_empty() {
                continue;
            }
            pairs += 1;
            cards.insert(name.clone());
            let (mut scs, none) = scenarios(&lines);
            let texts: Vec<String> = lines.iter().map(|t| t.full.clone()).chain(acts.iter().cloned()).collect();
            scs.extend((0..acts.len()).map(Sc::Activate));
            if scs.is_empty() {
                unprobed_keys.insert(format!("{key}: {name}"));
                unprobed.push(format!("{key}: {name}: no moment driven for: {}", texts.join(" / ")));
                continue;
            }
            let saga = def.subtypes.iter().any(|s| &**s == "saga");
            let mut fired_any = false;
            let mut gap: Vec<String> = vec![];
            let mut errs: Vec<String> = vec![];
            let mut n_abil = 1;
            for sc in scs {
                let want = if let Sc::Activate(k) = sc { acts[k].clone() } else { String::new() };
                if sc == Sc::SelfEnter && def.land {
                    continue;
                }
                std::panic::set_hook(Box::new(|_| {}));
                let equip = def.subtypes.iter().any(|s| matches!(&**s, "equipment" | "aura"));
                let blank = if def.land {
                    "Forest"
                } else if name == "Shivan Dragon" {
                    "Llanowar Elves"
                } else {
                    "Shivan Dragon"
                };
                let r = catch_unwind(AssertUnwindSafe(|| {
                    (
                        run(key, &name, &name, def.land, saga, equip, sc, &want),
                        run(key, &name, blank, def.land, saga, equip, sc, &want),
                    )
                }));
                let _ = std::panic::take_hook();
                match r {
                    Err(e) => {
                        let e = e
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_default();
                        errs.push(format!("{sc:?} panicked: {e}"));
                    }
                    Ok((o, base)) => {
                        if let Sc::Activate(_) = sc {
                            n_abil = n_abil.max(o.n_abil);
                        }
                        let by_state =
                            !o.fired && o.error.is_none() && o.sig != base.sig && !matches!(sc, Sc::Activate(_)); // an activation changes the state by its cost
                        if by_state && std::env::var("PROBE_SIG").is_ok() {
                            eprintln!("SIG {key}: {name}: {sc:?}: {:?} vs blank {:?}", o.sig, base.sig);
                        }
                        let o = Outcome { fired: o.fired || by_state, ..o };
                        if let Some(e) = &o.error {
                            errs.push(format!("{sc:?}: {e}"));
                        }
                        if matches!(
                            sc,
                            Sc::DieSelf
                                | Sc::DieOwn
                                | Sc::DieOpp
                                | Sc::SacOwn
                                | Sc::GainLife
                                | Sc::LoseLife
                                | Sc::Discard
                                | Sc::Draw
                                | Sc::OppDraw
                                | Sc::Token
                        ) && o.asked.windows(2).any(|w| w[0] == w[1])
                        {
                            eprintln!("TWICE {key}: {name}: {sc:?}: {}", o.asked.join(" || "));
                            twice.push(format!("{key}: {name}"));
                        }
                        if std::env::var("PROBE_DUMP").is_ok() {
                            eprintln!(
                                "{key}: {name}: {sc:?}: fired={} state={by_state} autos={} asked: {}",
                                o.fired,
                                o.autos,
                                o.asked.join(" || ")
                            );
                        }
                        if o.fired {
                            fired_any = true;
                            if o.asked.is_empty() {
                                gap.push(format!(
                                    "{sc:?}{}{}",
                                    if by_state { " (state only)" } else { "" },
                                    if o.autos > 0 { " (announced)" } else { "" }
                                ));
                            } else {
                                asked_ok += 1;
                            }
                        }
                    }
                }
            }
            if !gap.is_empty() {
                silent.entry(key.to_string()).or_default().push(format!(
                    "{name} ({})  |  {}",
                    gap.join(", "),
                    texts.join("  /  ")
                ));
            } else if !fired_any {
                probed += 0;
                unprobed_keys.insert(format!("{key}: {name}"));
                unprobed.push(format!(
                    "{key}: {name}: did not fire in the driven moments{}: {}",
                    if errs.is_empty() { String::new() } else { format!(" ({})", errs.join("; ")) },
                    texts.join(" / ")
                ));
            } else {
                probed += 1;
            }
            if !none.is_empty() {
                unprobed_keys.insert(format!("{key}: {name}"));
                unprobed.push(format!("{key}: {name}: line with no moment: {}", none.join(" / ")));
            }
        }
    }
    let flagged: usize = silent.values().map(Vec::len).sum();
    eprintln!(
        "{pairs} (deck, card) pairs / {} cards with a triggered choice; {probed} fired and asked, {flagged} flagged, {} unprobed ({asked_ok} moments asked)",
        cards.len(),
        unprobed.len()
    );
    for (k, v) in &silent {
        eprintln!("== {k}: {} card(s) whose trigger resolved without asking", v.len());
        for x in v {
            eprintln!("   {x}");
        }
    }
    for u in &unprobed {
        eprintln!("?? {u}");
    }
    assert!(
        twice.is_empty() || twice.iter().all(|t| TWICE_OK.iter().any(|(n, _)| n == t)),
        "asked the same question twice: {twice:?}"
    );
    check_unprobed(&unprobed_keys);
    check(&silent);
}

/// asked twice in a row on purpose, each with why
const TWICE_OK: &[(&str, &str)] = &[];

/// gaps found on 2026-11-04 and not yet fixed: the AI chooses for the person. Fixing one means taking it off this list;
/// a new gap fails the test. (The key ends with the moments the trigger was silent in.)
const TO_FIX: &[&str] = &[
];

/// listed by the probe but not a gap, each with why
const FALSE_ALARMS: &[(&str, &str)] = &[
    (
        "alela: Stirring Hopesinger (Main1)",
        "no choice: it puts a counter on each creature; the word \"target\" is in the condition (a spell that targets a creature)",
    ),
    (
        "thanos: Necropotence (Discard (state only))",
        "no choice: the discarded card is exiled; the line only matched \"discard a\"",
    ),
    (
        "yshtola: Necropotence (Discard (state only))",
        "no choice: the discarded card is exiled; the line only matched \"discard a\"",
    ),
    (
        "urdragon: Goldspan Dragon (Combat)",
        "no choice: it makes a Treasure; the word \"target\" is in the condition (becomes the target of a spell)",
    ),
    (
        "urdragon: Korvold, Fae-Cursed King (SacOwn (state only))",
        "no choice: the counter and the card are automatic; its \"sacrifice another permanent\" is asked on entering and attacking",
    ),
];

/// cards with a triggered choice the probe could not drive (or whose trigger never fired), each with why; a new one
/// fails the test, and so does one that now fires
const UNPROBED: &[(&str, &str)] = &[
    (
        "alela: Karmic Justice",
        "needs a spell or ability an opponent controls to destroy the person's noncreature permanent: no driver",
    ),
    (
        "cloud: Jocasta, Automaton Avenger",
        "triggers from the graveyard when the commander attacks: the probe puts cards on the battlefield only",
    ),
    (
        "karn: Rings of Brighthearth",
        "triggers on activating an ability: the probe does not activate abilities (the ability probe does, without resolving)",
    ),
    (
        "thrasios: Rings of Brighthearth",
        "triggers on activating an ability: the probe does not activate abilities (the ability probe does, without resolving)",
    ),
    (
        "karn: Urza's Saga",
        "a land Saga: the probe puts it on the battlefield without lore counters, so its chapters never come; chapter III's search is asked in practice_binder.rs",
    ),
    ("sauron: The Ozolith", "needs counters on the Ozolith (a creature with counters must leave first)"),
    (
        "sauron: Vision, Synthezoid Avenger",
        "triggers when an opponent casts a spell on their own turn: the probe's AI opponent casts nothing on the person's turn",
    ),
    (
        "thrasios: Tymna the Weaver",
        "needs an opponent dealt combat damage this turn before the second main phase: the probe drives the phases apart",
    ),
];

fn check(found: &BTreeMap<String, Vec<String>>) {
    let found: BTreeSet<String> = found
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
    assert!(
        new.is_empty() && fixed.is_empty(),
        "{} new gap(s), {} list entr(ies) to remove: see above",
        new.len(),
        fixed.len()
    );
}

/// the unprobed cards against `UNPROBED`: a new one or one that is now probed fails
fn check_unprobed(found: &BTreeSet<String>) {
    let new: Vec<&String> = found.iter().filter(|s| !UNPROBED.iter().any(|(n, _)| n == s)).collect();
    let gone: Vec<&str> = UNPROBED.iter().map(|(n, _)| *n).filter(|n| !found.contains(*n)).collect();
    for s in &new {
        eprintln!("NEW UNPROBED: {s}");
    }
    for s in &gone {
        eprintln!("NOW PROBED (take it off UNPROBED): {s}");
    }
    assert!(
        new.is_empty() && gone.is_empty(),
        "{} new unprobed card(s), {} to remove from UNPROBED: see above",
        new.len(),
        gone.len()
    );
}

/// the activated abilities used on the battlefield whose effect implies a choice (the ability probe's selection, with
/// the effect checked against WORDS), lowercased; mana abilities, Equip and abilities used from other zones are left out
fn activated_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|l| {
            let (cost, effect) = l.split_once(':')?;
            let cost = cost.rsplit('—').next().unwrap_or(cost).trim();
            let effect = effect.trim();
            let loyalty = cost.starts_with(['+', '−', '-', '0']) && cost.len() <= 4;
            let costy = cost.contains('{')
                || ["Sacrifice", "Exile", "Pay", "Remove", "Discard", "Tap", "Return"]
                    .iter()
                    .any(|w| cost.starts_with(w));
            if !(loyalty || costy) {
                return None;
            }
            let (lc, le) = (cost.to_lowercase(), effect.to_lowercase());
            if cost.starts_with("Equip")
                || le.starts_with("add ")
                || le.contains(", add ")
                || le.contains("add one mana")
                || le.contains("only your opponents may activate")
                || lc.contains("discard this card")
                || lc.contains("from your graveyard")
                || lc.contains("from your hand")
                || lc.contains("exile this card")
            {
                return None;
            }
            let permission =
                ["you may cast", "you may play", "choose new targets", "without paying"].iter().any(|w| le.contains(w));
            if !WORDS.iter().any(|w| le.contains(w) && !(permission && *w == "you may")) {
                return None;
            }
            Some(l.to_lowercase())
        })
        .collect()
}

/// the activated ability a "which ability?" prompt lists that is the Oracle line `want`: a planeswalker's by its loyalty
/// cost, any other by the words of its effect (the last choice is "cancel")
fn pick_ability(choices: &[String], want: &str) -> usize {
    let (cost, effect) = want.split_once(':').unwrap_or(("", want));
    let cost = cost.replace('−', "-");
    let cost = cost.trim();
    let words: Vec<&str> = effect.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 3).collect();
    let n = choices.len().saturating_sub(1).max(1).min(choices.len());
    (0..n)
        .max_by_key(|&i| {
            let l = choices[i].to_lowercase().replace('−', "-");
            let loyalty = cost.starts_with(['+', '-', '0']) && l.ends_with(cost);
            (loyalty as usize * 100 + words.iter().filter(|w| l.contains(**w)).count(), std::cmp::Reverse(i))
        })
        .unwrap_or(0)
}
