//! Practice mode for the four Binder cards that joined JD's lists on 2026-11-04 (#257): the person, not the AI, picks
//! Sauron, the Necromancer's graveyard creature, which Triple Triad cards to play, whether to play Urza's free card,
//! and which Equipment Gilgamesh puts onto the battlefield (and whether to attach one).

mod play_common;
use play_common::*;
use sim_core::ids::CardId;
use std::sync::{Arc, Mutex};

/// a person who answers with `pick` (None: the first choice, or yes); at a priority, pass
fn person(pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

/// the index of the first choice containing `word`
fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

/// the library is exactly `names`, the last one on top
fn library(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).library = cs.clone();
    cs
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Sauron, the Necromancer
#[test]
fn sauron_the_necromancer_copies_the_graveyard_creature_the_person_picks() {
    let mut g = table(&["kefka", "seph"]);
    let s = perm(&mut g, P0, "Sauron, the Necromancer");
    let elves = take(&mut g, P0, "Llanowar Elves");
    let archon = take(&mut g, P0, "Archon of Cruelty");
    g.player_mut(P0).gy.extend([elves, archon]);
    let h = person(|r| if r.prompt.contains("Sauron, the Necromancer: exile") { with(r, "Llanowar") } else { None });
    g.humans.add(P0, h.clone());
    let made = sim_core::impls::mine::necromancer_attack(&mut g, P0, s).unwrap();
    assert!(asked(&h, "Sauron, the Necromancer: exile"), "the person was asked");
    assert_eq!(made.len(), 1);
    assert!(g.perm(made[0]).name.contains("Llanowar Elves"), "copied {}", g.perm(made[0]).name);
    assert!(g.player(P0).gy.contains(&archon), "the Archon (the AI's pick) stays in the graveyard");
    assert!(g.player(P0).exile.contains(&elves));
}

// ------------------------------------------------------------------ Triple Triad
#[test]
fn triple_triad_plays_only_the_cards_the_person_picks() {
    let mut g = table(&["kefka", "seph"]);
    let t = perm(&mut g, P0, "Triple Triad");
    library(&mut g, P0, &["Island", "Thran Dynamo"]); // MV 4 on top: the opponent's MV 1 card is playable too
    library(&mut g, P1, &["Island", "Sol Ring"]);
    // play the opponent's Sol Ring, then stop: Thran Dynamo stays in exile
    let h = person(|r| {
        if r.prompt.contains("Triple Triad: play which card") {
            with(r, "Sol Ring").or_else(|| with(r, "play no more"))
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    sim_core::impls::mine::triad_upkeep(&mut g, t, P0).unwrap();
    assert!(asked(&h, "Triple Triad: play which card"), "the person was asked");
    assert!(has_perm(&g, P0, "Sol Ring"), "the picked card was cast free");
    assert!(!has_perm(&g, P0, "Thran Dynamo"), "the card not picked wasn't played");
    assert!(in_exile(&g, P0, "Thran Dynamo"), "and stays in exile");
}

// ------------------------------------------------------------------ Urza, Lord High Artificer
fn urza_with_top(answer: &'static str) -> (Game, Arc<ScriptHuman>) {
    let mut g = table(&["urdragon", "seph"]);
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    library(&mut g, P0, &["Sol Ring"]); // one card: the shuffle can't move it
    let h = person(move |r| if r.prompt.contains("Urza: cast Sol Ring") { with(r, answer) } else { None });
    g.humans.add(P0, h.clone());
    sim_core::impls::t5::urza_top(&mut g, u, P0).unwrap();
    assert!(asked(&h, "Urza: cast Sol Ring"), "the person was asked");
    (g, h)
}

#[test]
fn urzas_free_card_is_cast_when_the_person_says_so() {
    let (g, _) = urza_with_top("cast Sol Ring free");
    assert!(has_perm(&g, P0, "Sol Ring"));
}

#[test]
fn urzas_free_card_stays_in_exile_when_the_person_declines() {
    let (g, _) = urza_with_top("don't cast it");
    assert!(!has_perm(&g, P0, "Sol Ring"));
    assert!(in_exile(&g, P0, "Sol Ring"));
}

// ------------------------------------------------------------------ Gilgamesh, Master-at-Arms
#[test]
fn gilgamesh_puts_only_the_equipment_the_person_picks_and_attaching_is_optional() {
    let mut g = table(&["sauron", "seph"]);
    library(&mut g, P0, &["Island", "Swiftfoot Boots", "Lightning Greaves"]);
    let picks = Mutex::new(0);
    let h = person(move |r| {
        if r.prompt.contains("Gilgamesh: put which Equipment") {
            let mut n = picks.lock().unwrap();
            *n += 1;
            return if *n == 1 { with(r, "Lightning Greaves") } else { with(r, "done") };
        }
        if r.prompt.contains("Gilgamesh: attach") {
            return with(r, "don't attach");
        }
        None
    });
    g.humans.add(P0, h.clone());
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms"); // his enter trigger
    assert!(asked(&h, "Gilgamesh: put which Equipment"), "the person picked the Equipment");
    assert!(asked(&h, "Gilgamesh: attach"), "and was asked about attaching");
    assert!(has_perm(&g, P0, "Lightning Greaves"), "the picked Equipment came in");
    assert!(!has_perm(&g, P0, "Swiftfoot Boots"), "the other one didn't");
    let boots = card(&g, "Swiftfoot Boots");
    assert!(g.player(P0).library.contains(&boots), "it went to the bottom");
    let greaves = perms_named(&g, P0, "Lightning Greaves")[0];
    assert!(g.perm(greaves).attached.is_none(), "not attached: the person said no ({:?})", g.perm(gil).name);
}
