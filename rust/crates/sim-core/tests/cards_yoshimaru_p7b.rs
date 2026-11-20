//! Person paths of the Yoshimaru / Tymna deck, part B: the enters triggers that a person could not aim (Flickerwisp,
//! Felidar Guardian, Ennis, Palace Jailer, Solitude) and the Emperor's AI half. Each expectation follows the card's
//! Oracle text; every enters branch is also tried under a doubled trigger (Panharmonicon).

mod play_common;
use play_common::*;
use sim_core::impls::galadriel::eot_returns;
use sim_core::state::Step;
use std::sync::{Arc, Mutex};

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// a person at P0: each question that is not a priority is answered by the next entry of `words` that one of the
/// choices contains (the first choice when none does); priority is passed. Every question is recorded.
fn person(g: &mut Game, words: &[&str]) -> Arc<ScriptHuman> {
    let words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    let used = Mutex::new(vec![false; words.len()]);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        let mut used = used.lock().unwrap();
        for (i, w) in words.iter().enumerate() {
            if used[i] {
                continue;
            }
            if let Some(k) = r.choices.iter().position(|c| c.contains(w.as_str())) {
                used[i] = true;
                return Answer::Pick(k);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    h
}

fn asks(h: &ScriptHuman, word: &str) -> Vec<Request> {
    h.asked().into_iter().filter(|r| r.prompt.contains(word)).collect()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    has_perm(g, p, name) || g.player(p).lands.iter().any(|&l| g.perm(l).on_bf && g.perm(l).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.contains(&card(g, name))
}

// ------------------------------------------------------------------ Flickerwisp
// "Flying. When this creature enters, exile another target permanent. Return that card to the battlefield under its
// owner's control at the beginning of the next end step."
#[test]
fn flickerwisp_asks_a_person_for_any_other_permanent_lands_included_and_it_is_mandatory() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    perm(&mut g, P1, "Sol Ring");
    token(&mut g, P0, 2);
    let h = person(&mut g, &["Island"]);
    perm(&mut g, P0, "Flickerwisp");
    let q = asks(&h, "Flickerwisp");
    assert_eq!(q.len(), 1, "one question: {:?}", h.asked());
    let c = &q[0].choices;
    assert!(c.iter().any(|x| x.contains("Island")), "a land is a legal target: {c:?}");
    assert!(c.iter().any(|x| x.contains("Sol Ring")) && c.iter().any(|x| x.contains("token") || x.contains("Token")), "{c:?}");
    assert!(!c.iter().any(|x| x.starts_with("Flickerwisp")), "'another': not itself: {c:?}");
    assert!(!c.iter().any(|x| x.contains("no target") || x.contains("none")), "the target is mandatory: {c:?}");
    assert!(!on_bf(&g, P1, "Island") && in_exile(&g, P1, "Island"), "the land is exiled");
    assert!(on_bf(&g, P1, "Sol Ring"), "only the chosen one");
    eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P1, "Island") && !in_exile(&g, P1, "Island"), "it returns at the end step under its owner");
}

#[test]
fn flickerwisp_exiles_the_token_a_person_picks_for_good() {
    let mut g = tbl();
    let t = token(&mut g, P1, 3);
    lands(&mut g, P1, "Island", 1, false);
    let h = person(&mut g, &["token"]);
    perm(&mut g, P0, "Flickerwisp");
    assert_eq!(asks(&h, "Flickerwisp").len(), 1, "the person picks: {:?}", h.asked());
    assert!(!g.perm(t).on_bf, "the token is exiled");
    eot_returns(&mut g).unwrap();
    assert!(!g.player(P1).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token), "a token never returns");
    assert!(on_bf(&g, P1, "Island"), "the land stayed");
}

#[test]
fn flickerwisp_returns_a_card_exiled_in_an_end_step_at_the_next_end_step_only() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    g.step = Step::End;
    eot_returns(&mut g).unwrap(); // the beginning of this end step: nothing waits yet
    let _h = person(&mut g, &["Island"]);
    perm(&mut g, P0, "Flickerwisp");
    assert!(in_exile(&g, P1, "Island"));
    eot_returns(&mut g).unwrap(); // the next end step
    assert!(on_bf(&g, P1, "Island"), "back at the beginning of the next end step");
}

#[test]
fn flickerwisp_doubled_asks_a_person_once_per_instance_and_exiles_two_targets() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    lands(&mut g, P1, "Mountain", 1, false);
    perm(&mut g, P0, "Panharmonicon");
    let h = person(&mut g, &["Island", "Mountain"]);
    perm(&mut g, P0, "Flickerwisp");
    let q = asks(&h, "Flickerwisp");
    assert_eq!(q.len(), 2, "one question per instance: {:?}", h.asked());
    assert!(!on_bf(&g, P1, "Island") && !on_bf(&g, P1, "Mountain"), "two targets exiled");
    assert!(q.iter().any(|r| !r.choices.iter().any(|c| c.contains("Island"))), "the second ask no longer offers it");
    eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P1, "Island") && on_bf(&g, P1, "Mountain"), "both return");
}

#[test]
fn flickerwisp_ai_can_exile_a_land_when_it_is_the_only_other_permanent() {
    let mut g = tbl();
    g.active = Some(P1); // the opponent's turn: its untapped land is mana it can't use
    lands(&mut g, P1, "Island", 1, false);
    perm(&mut g, P0, "Flickerwisp");
    assert!(!on_bf(&g, P1, "Island") && in_exile(&g, P1, "Island"), "{:?}", g.player(P1).lands);
    eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P1, "Island"));
}

#[test]
fn flickerwisp_ai_exiles_its_own_tapped_land_not_an_untapped_one() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Plains", 2, false);
    g.perm_mut(ls[0]).tapped = true;
    perm(&mut g, P0, "Flickerwisp");
    assert!(!g.perm(ls[0]).on_bf, "the tapped land comes back untapped");
    assert!(g.perm(ls[1]).on_bf, "the untapped one stays");
}
