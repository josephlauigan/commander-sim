//! Person paths of the Yoshimaru / Tymna deck, part A: the cards a person could not play (or could not play fully)
//! though the AI could. Each test seats a person at P0, asks for the action and checks what the person was asked and
//! what happened. Oracle text rules every expectation.

mod play_common;
use play_common::*;
use sim_core::human::human::human_main;
use sim_core::human::{Target, legal};
use std::sync::{Arc, Mutex};

type Rule = (String, String);

fn ans_word(key: &str, word: &str) -> Rule {
    (key.to_string(), word.to_string())
}

fn ans_target(g: &Game, key: &str, x: Target) -> Rule {
    (key.to_string(), legal::describe_target(g, P0, x))
}

/// the person takes `action` at the first priority, answers by `rules` (the first that fits; the first choice when
/// none does) and passes at every other priority
fn play(g: &mut Game, action: Value, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let first = Mutex::new(Some(action));
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return match first.lock().unwrap().take() {
                Some(a) => Answer::Action(a),
                None => Answer::Action(json!({"do": "pass"})),
            };
        }
        for (key, word) in &rules {
            if r.prompt.contains(key.as_str())
                && let Some(i) = r.choices.iter().position(|c| c.contains(word.as_str()))
            {
                return Answer::Pick(i);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    human_main(g, P0, false).unwrap();
    h
}

fn use_perm(g: &mut Game, m: PermId, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let i = g.player(P0).perms.iter().position(|&x| x == m).unwrap();
    play(g, json!({"do": "use", "perm": i}), rules)
}

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

fn asked(h: &ScriptHuman, word: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(word))
}

fn has_kw(g: &Game, m: PermId, kw: &str) -> bool {
    g.perm(m).eot_kw.iter().any(|k| *k == kw)
}

// ------------------------------------------------------------------ Mother of Runes
#[test]
fn mother_of_runes_gives_another_creature_the_chosen_protection() {
    let mut g = tbl();
    let mom = perm(&mut g, P0, "Mother of Runes");
    let other = perm(&mut g, P0, "Llanowar Elves");
    let tg = ans_target(&g, "Mother of Runes", Target::Perm(other));
    let h = use_perm(&mut g, mom, vec![tg, ans_word("protection from what", "red")]);
    assert!(asked(&h, "protection from what"));
    assert!(has_kw(&g, other, "pro_R"), "{:?}", g.perm(other).eot_kw);
    assert!(g.perm(mom).tapped);
}

#[test]
fn mother_of_runes_may_target_herself_and_has_five_colours_only() {
    let mut g = tbl();
    let mom = perm(&mut g, P0, "Mother of Runes");
    let tg = ans_target(&g, "Mother of Runes", Target::Perm(mom));
    let h = use_perm(&mut g, mom, vec![tg, ans_word("protection from what", "blue")]);
    assert!(has_kw(&g, mom, "pro_U"), "{:?}", g.perm(mom).eot_kw);
    let q = h.asked().into_iter().find(|r| r.prompt.contains("protection from what")).expect("asks the colour");
    let colours = q.choices.iter().filter(|c| c.starts_with("protection from")).count();
    assert_eq!(colours, 5, "{:?}", q.choices);
    assert!(!q.choices.iter().any(|c| c.contains("colorless")), "{:?}", q.choices);
}

#[test]
fn mother_of_runes_is_refused_while_summoning_sick() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Mother of Runes");
    let mom = sim_core::engine::zones::enter(
        &mut g,
        P0,
        c,
        sim_core::engine::zones::Enter { sick: true, ..Default::default() },
    )
    .unwrap();
    let h = use_perm(&mut g, mom, vec![]);
    assert!(!g.perm(mom).tapped);
    assert!(h.told().iter().any(|t| t.1.contains("summoning sickness")), "{:?}", h.told());
}
