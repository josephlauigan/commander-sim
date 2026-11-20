//! Person paths of the Yoshimaru / Tymna deck, part B: the enters triggers that a person could not aim (Flickerwisp,
//! Felidar Guardian, Ennis, Palace Jailer, Solitude) and the Emperor's AI half. Each expectation follows the card's
//! Oracle text; every enters branch is also tried under a doubled trigger (Panharmonicon).

mod play_common;
use play_common::*;
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

/// the beginning of an end step: what was exiled until then comes back
fn end_step(g: &mut Game) {
    sim_core::cardcode::delayed_end_step(g, P0).unwrap();
}

fn asks(h: &ScriptHuman, word: &str) -> Vec<Request> {
    h.asked().into_iter().filter(|r| r.prompt.contains(word)).collect()
}

/// the permanent is on the battlefield under p: a land among p's lands (and not among the permanents), anything else
/// among the permanents
fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    if ["Island", "Plains", "Mountain", "Swamp", "Forest"].contains(&name) {
        g.player(p).lands.iter().any(|&l| g.perm(l).on_bf && g.perm(l).name == name)
            && !g.player(p).perms.iter().any(|&m| g.perm(m).name == name)
    } else {
        has_perm(g, p, name)
    }
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
    assert!(
        c.iter().any(|x| x.contains("Sol Ring")) && c.iter().any(|x| x.contains("token") || x.contains("Token")),
        "{c:?}"
    );
    assert!(!c.iter().any(|x| x.starts_with("Flickerwisp")), "'another': not itself: {c:?}");
    assert!(!c.iter().any(|x| x.contains("no target") || x.contains("none")), "the target is mandatory: {c:?}");
    assert!(!on_bf(&g, P1, "Island") && in_exile(&g, P1, "Island"), "the land is exiled");
    assert!(on_bf(&g, P1, "Sol Ring"), "only the chosen one");
    end_step(&mut g);
    assert!(on_bf(&g, P1, "Island") && !in_exile(&g, P1, "Island"), "it returns at the end step under its owner");
}

#[test]
fn flickerwisp_exiles_the_token_a_person_picks_for_good() {
    let mut g = tbl();
    let t = token(&mut g, P1, 3);
    lands(&mut g, P1, "Island", 1, false);
    let h = person(&mut g, &["oken"]);
    perm(&mut g, P0, "Flickerwisp");
    assert_eq!(asks(&h, "Flickerwisp").len(), 1, "the person picks: {:?}", h.asked());
    assert!(!g.perm(t).on_bf, "the token is exiled");
    end_step(&mut g);
    assert!(!g.player(P1).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token), "a token never returns");
    assert!(on_bf(&g, P1, "Island"), "the land stayed");
}

#[test]
fn flickerwisp_returns_a_card_exiled_in_an_end_step_at_the_next_end_step_only() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    g.step = Step::End;
    end_step(&mut g); // the beginning of this end step: nothing waits yet
    let _h = person(&mut g, &["Island"]);
    perm(&mut g, P0, "Flickerwisp");
    assert!(in_exile(&g, P1, "Island"));
    end_step(&mut g); // the next end step
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
    end_step(&mut g);
    assert!(on_bf(&g, P1, "Island") && on_bf(&g, P1, "Mountain"), "both return");
}

#[test]
fn flickerwisp_ai_can_exile_a_land_when_it_is_the_only_other_permanent() {
    let mut g = tbl();
    g.active = Some(P1); // the opponent's turn: its untapped land is mana it can't use
    lands(&mut g, P1, "Island", 1, false);
    perm(&mut g, P0, "Flickerwisp");
    assert!(!on_bf(&g, P1, "Island") && in_exile(&g, P1, "Island"), "{:?}", g.player(P1).lands);
    end_step(&mut g);
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

// ------------------------------------------------------------------ Felidar Guardian
// "When this creature enters, you may exile another target permanent you control, then return that card to the
// battlefield under its owner's control."
#[test]
fn felidar_asks_a_person_for_another_permanent_they_control_and_blinks_it() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    let wall = perm(&mut g, P0, "Wall of Omens");
    lands(&mut g, P0, "Plains", 1, false);
    token(&mut g, P0, 2);
    let hand = g.player(P0).hand.len();
    let h = person(&mut g, &["Wall of Omens"]);
    perm(&mut g, P0, "Felidar Guardian");
    let q = asks(&h, "Felidar Guardian");
    assert_eq!(q.len(), 1, "one question: {:?}", h.asked());
    let c = &q[0].choices;
    assert!(c.iter().any(|x| x.contains("Plains")), "a land you control is a legal target: {c:?}");
    assert!(c.iter().any(|x| x.contains("Wall of Omens")), "{c:?}");
    assert!(!c.iter().any(|x| x.contains("Sol Ring")), "an opponent's permanent is not: {c:?}");
    assert!(!c.iter().any(|x| x.starts_with("Felidar Guardian")), "'another': {c:?}");
    assert!(c.iter().any(|x| x.contains("no") || x.contains("none")), "'you may': a way out: {c:?}");
    assert!(!g.perm(wall).on_bf && on_bf(&g, P0, "Wall of Omens"), "blinked: a new object");
    assert_eq!(g.player(P0).hand.len(), hand + 1, "and it entered again: Wall of Omens draws");
}

#[test]
fn felidar_lets_a_person_blink_nothing() {
    let mut g = tbl();
    let wall = perm(&mut g, P0, "Wall of Omens");
    let hand = g.player(P0).hand.len();
    let _h = person(&mut g, &["no"]);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(g.perm(wall).on_bf, "not exiled");
    assert_eq!(g.player(P0).hand.len(), hand);
}

#[test]
fn felidar_exiles_a_token_a_person_picks_for_good() {
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    let _h = person(&mut g, &["token"]);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(!g.perm(t).on_bf, "the token is exiled");
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token), "and does not come back");
}

#[test]
fn felidar_blinks_a_tapped_land_it_returns_untapped() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Plains", 1, true);
    let _h = person(&mut g, &["Plains"]);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(!g.perm(ls[0]).on_bf, "the old land is gone");
    let back: Vec<_> = g.player(P0).lands.iter().copied().filter(|&l| g.perm(l).on_bf).collect();
    assert_eq!(back.len(), 1, "one land on the battlefield");
    assert!(!g.perm(back[0]).tapped, "it returned untapped");
}

#[test]
fn felidar_doubled_asks_a_person_once_per_instance() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Plains", 2, true);
    perm(&mut g, P0, "Panharmonicon");
    let h = person(&mut g, &["tapped", "tapped"]);
    perm(&mut g, P0, "Felidar Guardian");
    assert_eq!(asks(&h, "Felidar Guardian").len(), 2, "{:?}", h.asked());
    assert!(ls.iter().all(|&l| !g.perm(l).on_bf), "both lands were blinked");
    let back: Vec<_> = g.player(P0).lands.iter().copied().filter(|&l| g.perm(l).on_bf).collect();
    assert_eq!(back.len(), 2);
    assert!(back.iter().all(|&l| !g.perm(l).tapped), "both came back untapped");
}

#[test]
fn felidar_chain_driven_by_a_person_is_not_cut_at_three_and_a_cap_is_logged() {
    let mut g = tbl();
    g.log = Some(vec![]);
    perm(&mut g, P0, "Felidar Guardian");
    // every Felidar that enters blinks the other one: a loop the person drives by choosing it each time
    let h = person(&mut g, &["Felidar Guardian"; 12]);
    perm(&mut g, P0, "Felidar Guardian");
    let n = asks(&h, "Felidar Guardian").len();
    assert!(n > 3, "a person's chain runs past three links, asked {n} times");
    let log = g.log.as_ref().unwrap().join("\n");
    assert!(log.contains("blink chain"), "the cap is not silent: {log}");
    assert_eq!(g.blink_depth, 0);
}
// ------------------------------------------------------------------ Ennis, Debate Moderator
// "When Ennis enters, exile up to one other target creature you control. Return that card to the battlefield under its
// owner's control at the beginning of the next end step."
#[test]
fn ennis_asks_a_person_for_up_to_one_other_creature_they_control() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    let wall = perm(&mut g, P0, "Wall of Omens");
    token(&mut g, P0, 2);
    let h = person(&mut g, &["Wall of Omens"]);
    perm(&mut g, P0, "Ennis, Debate Moderator");
    let q = asks(&h, "Ennis");
    assert_eq!(q.len(), 1, "{:?}", h.asked());
    let c = &q[0].choices;
    assert!(c.iter().any(|x| x.contains("Wall of Omens")) && c.iter().any(|x| x.contains("oken")), "{c:?}");
    assert!(!c.iter().any(|x| x.contains("Baneslayer")), "only your own creatures: {c:?}");
    assert!(!c.iter().any(|x| x.starts_with("Ennis")), "'other': {c:?}");
    assert!(c.iter().any(|x| x.contains("none") || x.contains("no target")), "'up to one': {c:?}");
    assert!(!g.perm(wall).on_bf && in_exile(&g, P0, "Wall of Omens"));
    end_step(&mut g);
    assert!(on_bf(&g, P0, "Wall of Omens"), "it returns at the end step");
}

#[test]
fn ennis_lets_a_person_exile_nothing() {
    let mut g = tbl();
    let wall = perm(&mut g, P0, "Wall of Omens");
    let _h = person(&mut g, &["none"]);
    perm(&mut g, P0, "Ennis, Debate Moderator");
    assert!(g.perm(wall).on_bf);
}

#[test]
fn ennis_exiles_a_token_for_good_and_it_is_no_card_for_the_counter() {
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    let _h = person(&mut g, &["oken"]);
    let e = perm(&mut g, P0, "Ennis, Debate Moderator");
    assert!(!g.perm(t).on_bf);
    end_step(&mut g);
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token));
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(e).plus, 0, "a token is not a card put into exile");
}

#[test]
fn ennis_returns_a_creature_it_exiled_under_its_owners_control() {
    let mut g = tbl();
    let stolen = perm(&mut g, P1, "Wall of Omens");
    sim_core::engine::control::steal(&mut g, P0, stolen, sim_core::engine::control::Duration::Permanent).unwrap();
    let _h = person(&mut g, &["Wall of Omens"]);
    perm(&mut g, P0, "Ennis, Debate Moderator");
    assert!(in_exile(&g, P1, "Wall of Omens"), "owned by P1: it goes to P1's exile");
    end_step(&mut g);
    assert!(on_bf(&g, P1, "Wall of Omens") && !on_bf(&g, P0, "Wall of Omens"), "back under its owner's control");
}

#[test]
fn ennis_doubled_asks_a_person_once_per_instance() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Helitrooper");
    let b = perm(&mut g, P0, "Armory Automaton");
    perm(&mut g, P0, "Panharmonicon");
    let h = person(&mut g, &["Helitrooper", "Armory Automaton"]);
    perm(&mut g, P0, "Ennis, Debate Moderator");
    assert_eq!(asks(&h, "Ennis").len(), 2, "{:?}", h.asked());
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "two creatures exiled");
    end_step(&mut g);
    assert!(on_bf(&g, P0, "Helitrooper") && on_bf(&g, P0, "Armory Automaton"));
}
