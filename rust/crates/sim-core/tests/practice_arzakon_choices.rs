//! Practice mode (2026-11-23): the choices of cards the Arzakon change brought into the Dragons deck, asked of a person:
//! Hellkite Courser ("you may" put a commander onto the battlefield, and which one), Deflecting Swat (the new target,
//! and no cast with nothing on the stack), Maelstrom of the Spirit Dragon (which Dragon to find).

mod play_common;
use play_common::*;
use std::sync::{Arc, Mutex};

/// a person at seat 0 whose first action is `action` and who picks the ability whose label starts with `ab`; `pick`
/// answers every other question (None: the first choice, or yes). Runs their main phase
fn person(
    g: &mut Game,
    action: Value,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { action.clone() } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.ends_with(": which ability?") {
            return Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.starts_with(ab))
                    .unwrap_or_else(|| panic!("no ability {ab:?} in {:?}", r.choices)),
            );
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }));
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(g, P0, false).unwrap();
    h
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

fn prompts(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| r.prompt.clone()).collect()
}

// ------------------------------------------------------------------ Hellkite Courser
fn courser_table() -> Game {
    let mut g = table(&["urdragon", "seph"]);
    g.active = Some(P0);
    assert!(g.player(P0).cmds_in_zone().count() >= 1, "the Ur-Dragon waits in the command zone");
    g
}

/// a person who passes every priority and answers the Courser question with `answer` (a position, or the choice
/// holding `text`)
fn courser_person(g: &mut Game, answer: Result<usize, &'static str>) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.starts_with("Hellkite Courser: put") {
            return Answer::Pick(match answer {
                Ok(i) => i,
                Err(t) => r.choices.iter().position(|c| c.contains(t)).expect("the commander is a choice"),
            });
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    h
}

#[test]
fn hellkite_courser_asks_the_person_whether_to_put_the_commander_onto_the_battlefield() {
    for (answer, onto) in [(0usize, true), (1, false)] {
        let mut g = courser_table();
        let h = courser_person(&mut g, Ok(answer));
        perm(&mut g, P0, "Hellkite Courser");
        assert!(asked(&h, "Hellkite Courser: put The Ur-Dragon onto the battlefield?"), "{:?}", prompts(&h));
        assert_eq!(has_perm(&g, P0, "The Ur-Dragon"), onto);
        assert_eq!(g.player(P0).cmds_in_zone().count(), if onto { 0 } else { 1 });
    }
}

#[test]
fn hellkite_courser_with_no_commander_in_the_command_zone_asks_nothing() {
    let mut g = courser_table();
    let cmd = g.player(P0).cmds_in_zone().next().unwrap();
    g.player_mut(P0).set_cmd_zone(cmd, false);
    let h = courser_person(&mut g, Ok(0));
    perm(&mut g, P0, "Hellkite Courser");
    assert!(!asked(&h, "Hellkite Courser: put"), "{:?}", prompts(&h));
}

#[test]
fn hellkite_courser_asks_which_commander_when_two_wait() {
    let mut g = courser_table();
    // a second commander (partners): any other card will do as the second commander seat
    let second = card(&g, "Sol Ring");
    g.player_mut(P0).cmd2 = Some(second);
    g.player_mut(P0).set_cmd_zone(second, true);
    let h = courser_person(&mut g, Err("Sol Ring"));
    perm(&mut g, P0, "Hellkite Courser");
    assert!(asked(&h, "Hellkite Courser: put which commander onto the battlefield?"), "{:?}", prompts(&h));
    assert!(has_perm(&g, P0, "Sol Ring"), "the commander the person named came down");
    assert!(!has_perm(&g, P0, "The Ur-Dragon"));
}

// ------------------------------------------------------------------ Deflecting Swat
#[test]
fn deflecting_swat_asks_the_person_for_the_new_target() {
    let mut g = table(&["urdragon", "seph"]);
    g.active = Some(P1);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let swat = hand(&mut g, P0, &["Deflecting Swat"])[0];
    pool(&mut g, P0, &[('R', 1), ('C', 2)]);
    let first = Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "cast", "card": 0}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.contains("new target") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("Birds")).expect("Birds is a choice"));
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    let grasp = hand(&mut g, P1, &["Infernal Grasp"])[0];
    let _ = sim_core::engine::cast::cast_card(
        &mut g,
        P1,
        grasp,
        "hand",
        sim_core::state::Ctx { target: Some(elf), ..Default::default() },
    );
    assert!(asked(&h, "Deflecting Swat: choose a new target"), "{:?}", prompts(&h));
    assert!(!g.player(P0).hand.contains(&swat), "Swat was cast");
    assert!(g.perm(elf).on_bf, "the Elves live: the removal was sent to the person's pick");
    assert!(!g.perm(birds).on_bf, "the new target took it");
}

#[test]
fn deflecting_swat_cannot_be_cast_with_nothing_on_the_stack() {
    let mut g = table(&["urdragon", "seph"]);
    g.active = Some(P0);
    let swat = hand(&mut g, P0, &["Deflecting Swat"])[0];
    pool(&mut g, P0, &[('R', 1), ('C', 2)]);
    let h = person(&mut g, json!({"do": "cast", "card": 0}), "", |_| None);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("Deflecting Swat")), "{:?}", h.told());
    assert!(g.player(P0).hand.contains(&swat), "refused: the card stays in hand");
    assert_eq!(pool_total(&g, P0), 3, "nothing was paid");
}

// ------------------------------------------------------------------ Maelstrom of the Spirit Dragon
#[test]
fn maelstrom_asks_the_person_which_dragon_to_find() {
    let mut g = table(&["urdragon", "seph"]);
    let l = lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false)[0];
    pool(&mut g, P0, &[('C', 4)]);
    let i = g.player(P0).lands.iter().position(|&x| x == l).unwrap();
    let h = person(&mut g, json!({"do": "use", "land": i}), "{4}, {T}", |r| {
        if r.prompt.contains("search for a Dragon card") {
            r.choices.iter().position(|c| c.starts_with("Dragonlord Dromoka"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Maelstrom of the Spirit Dragon: search for a Dragon card"), "{:?}", prompts(&h));
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Dragonlord Dromoka"));
}
