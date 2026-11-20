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

// ------------------------------------------------------------------ Palace Jailer and the monarch
// "When this creature enters, you become the monarch. When this creature enters, exile target creature an opponent
// controls until an opponent becomes the monarch."
fn exiled_creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).exile.iter().count()
}

/// a 0/1 token: worth nothing to the AI's value floor
fn weak_token(g: &mut Game, p: PlayerId) -> PermId {
    let spec =
        sim_core::engine::zones::Tokens { tgh: Some(1), sick: false, ..sim_core::engine::zones::Tokens::new(1, 0) };
    sim_core::engine::zones::make_tokens(g, p, spec).unwrap()[0]
}

#[test]
fn palace_jailer_asks_a_person_for_a_creature_an_opponent_controls_and_it_is_mandatory() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    weak_token(&mut g, P1);
    perm(&mut g, P0, "Wall of Omens");
    let h = person(&mut g, &["oken"]);
    perm(&mut g, P0, "Palace Jailer");
    let q = asks(&h, "Palace Jailer");
    assert_eq!(q.len(), 1, "one question: {:?}", h.asked());
    let c = &q[0].choices;
    assert!(c.iter().any(|x| x.contains("Baneslayer")) && c.iter().any(|x| x.contains("oken")), "{c:?}");
    assert!(!c.iter().any(|x| x.contains("Wall of Omens")), "only an opponent's creature: {c:?}");
    assert!(!c.iter().any(|x| x.contains("no target") || x.contains("none")), "the target is mandatory: {c:?}");
    assert_eq!(g.monarch, Some(P0), "you become the monarch");
    assert!(on_bf(&g, P1, "Baneslayer Angel"), "the one the person did not pick stays");
    assert!(!g.player(P1).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token), "the token was picked");
}

#[test]
fn palace_jailer_exiles_a_creature_a_person_must_pick_even_when_the_ai_would_not_bother() {
    let mut g = tbl();
    let t = weak_token(&mut g, P1);
    assert!(sim_core::engine::values::pval(&g, t) < 1.0, "the AI's value floor would skip it");
    let _h = person(&mut g, &["oken"]);
    perm(&mut g, P0, "Palace Jailer");
    assert!(!g.perm(t).on_bf, "a legal target must be taken");
}

#[test]
fn palace_jailer_is_two_triggers_that_use_the_stack_one_after_the_other() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    hand(&mut g, P1, &["Counterspell"]); // something to respond with, so that the opponent is given priority
    pool(&mut g, P1, &[('U', 2)]);
    let seen = Arc::new(Mutex::new(vec![]));
    let s2 = seen.clone();
    let opp = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            s2.lock().unwrap().push(r.data["stack"].to_string());
        }
        Answer::Action(json!({"do": "pass"}))
    }));
    g.humans.add(P1, opp);
    perm(&mut g, P0, "Palace Jailer");
    let seen = seen.lock().unwrap();
    let monarch = seen.iter().position(|s| s.contains("monarch"));
    let exile = seen.iter().position(|s| s.contains("exile"));
    assert!(monarch.is_some() && exile.is_some(), "each trigger gets a window of its own: {seen:?}");
    assert!(monarch < exile, "{seen:?}");
}

#[test]
fn palace_jailer_creature_returns_when_an_opponent_takes_the_monarchy_even_after_the_jailer_died() {
    let mut g = tbl();
    let t = perm(&mut g, P1, "Baneslayer Angel");
    let j = perm(&mut g, P0, "Palace Jailer");
    assert!(!g.perm(t).on_bf && in_exile(&g, P1, "Baneslayer Angel"));
    sim_core::engine::zones::die(&mut g, j, "sba").unwrap();
    assert!(in_exile(&g, P1, "Baneslayer Angel"), "the Jailer leaving does not return it");
    let a = token(&mut g, P1, 2);
    sim_core::engine::combat::resolve_combat(&mut g, P1, &[a], P0, &[a]).unwrap(); // hits the monarch
    assert_eq!(g.monarch, Some(P1));
    assert!(on_bf(&g, P1, "Baneslayer Angel") && !in_exile(&g, P1, "Baneslayer Angel"), "it returns");
}

#[test]
fn palace_jailer_creature_returns_when_the_monarch_is_eliminated_and_an_opponent_takes_over() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P0, "Palace Jailer");
    assert_eq!(g.monarch, Some(P0));
    g.active = Some(P1);
    // the Jailer's controller is the monarch and leaves the game: the player whose turn it is becomes the monarch,
    // and its opponents' becoming the monarch ends the exile (CR 800.4i: its opponents are still its opponents)
    sim_core::engine::life::lose_life(&mut g, P0, 40, Some(P2), "burn", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive);
    assert_eq!(g.monarch, Some(P1), "the active player becomes the monarch");
    assert!(on_bf(&g, P1, "Baneslayer Angel"), "and the creature returns under its owner");
}

#[test]
fn the_monarch_leaving_the_game_passes_the_crown_to_the_active_player_or_the_next_in_turn_order() {
    let mut g = tbl();
    g.monarch = Some(P2);
    g.active = Some(P0);
    sim_core::engine::life::lose_life(&mut g, P2, 40, Some(P0), "burn", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.monarch, Some(P0), "the player whose turn it is");

    let mut g = tbl();
    g.monarch = Some(P1);
    g.active = Some(P1);
    sim_core::engine::life::lose_life(&mut g, P1, 40, Some(P0), "burn", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.monarch, Some(P2), "the active player left: the next in turn order");
}

#[test]
fn palace_jailer_doubled_asks_a_person_once_per_instance_and_exiles_two_creatures() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P1, "Wall of Omens");
    perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, "Panharmonicon");
    let h = person(&mut g, &["Baneslayer", "Wall"]);
    perm(&mut g, P0, "Palace Jailer");
    assert_eq!(asks(&h, "Palace Jailer").len(), 2, "{:?}", h.asked());
    assert_eq!(exiled_creatures(&g, P1), 2, "two are exiled, the one nobody picked stays");
    assert!(on_bf(&g, P1, "Llanowar Elves"));
    assert_eq!(g.monarch, Some(P0));
}
#[test]
fn palace_jailer_creature_stays_when_its_controller_takes_the_monarchy_back() {
    let mut g = tbl();
    perm(&mut g, P0, "Palace Jailer"); // nothing to exile yet; P0 is the monarch
    let j = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Palace Jailer").unwrap();
    g.monarch = Some(P1); // the monarch trigger did nothing (it was countered): an opponent holds the crown
    let t = perm(&mut g, P1, "Baneslayer Angel");
    sim_core::impls::t2::jailer_exile(&mut g, j, P0).unwrap();
    assert!(in_exile(&g, P1, "Baneslayer Angel"), "exiled though an opponent is the monarch");
    sim_core::cardcode::become_monarch(&mut g, P0).unwrap(); // the Jailer's controller takes it
    assert!(in_exile(&g, P1, "Baneslayer Angel"), "nothing returns when its controller becomes the monarch");
    assert!(!g.perm(t).on_bf);
    sim_core::cardcode::become_monarch(&mut g, P2).unwrap(); // any other opponent
    assert!(on_bf(&g, P1, "Baneslayer Angel"), "any opponent becoming the monarch returns it");
}

// ------------------------------------------------------------------ Solitude
// "Flash, lifelink. When this creature enters, exile up to one other target creature. That creature's controller gains
// life equal to its power. Evoke: exile a white card from your hand."
const SOL: &str = "Solitude";

#[test]
fn solitude_asks_a_person_for_up_to_one_other_creature_and_its_controller_gains_its_power() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P0, "Wall of Omens");
    let life = g.player(P1).life;
    let h = person(&mut g, &["Baneslayer"]);
    perm(&mut g, P0, SOL);
    let q = asks(&h, "Solitude");
    assert_eq!(q.len(), 1, "one question: {:?}", h.asked());
    let c = &q[0].choices;
    assert!(c.iter().any(|x| x.contains("Baneslayer")) && c.iter().any(|x| x.contains("Wall of Omens")), "{c:?}");
    assert!(!c.iter().any(|x| x.starts_with(SOL)), "'other': {c:?}");
    assert!(c.iter().any(|x| x.contains("none") || x.contains("no target")), "'up to one': {c:?}");
    assert!(in_exile(&g, P1, "Baneslayer Angel"));
    assert_eq!(g.player(P1).life, life + 5, "its controller gains life equal to its power");
}

#[test]
fn solitude_may_exile_a_creature_of_the_persons_own_and_they_gain_its_power() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P0, "Selfless Spirit");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    let _h = person(&mut g, &["Selfless Spirit"]);
    perm(&mut g, P0, SOL);
    assert!(in_exile(&g, P0, "Selfless Spirit"));
    assert_eq!(g.player(P0).life, mine + 2);
    assert_eq!(g.player(P1).life, theirs, "the opponent gains nothing");
    assert!(on_bf(&g, P1, "Baneslayer Angel"));
}

#[test]
fn solitude_lets_a_person_exile_nothing() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    let life = g.player(P1).life;
    let h = person(&mut g, &["none"]);
    perm(&mut g, P0, SOL);
    assert_eq!(asks(&h, "Solitude").len(), 1, "{:?}", h.asked());
    assert!(on_bf(&g, P1, "Baneslayer Angel"));
    assert_eq!(g.player(P1).life, life);
}

#[test]
fn solitude_doubled_asks_a_person_once_per_instance() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P1, "Wall of Omens");
    perm(&mut g, P0, "Panharmonicon");
    let life = g.player(P1).life;
    let h = person(&mut g, &["Baneslayer", "Grave Titan"]);
    perm(&mut g, P0, SOL);
    assert_eq!(asks(&h, "Solitude").len(), 2, "{:?}", h.asked());
    assert!(in_exile(&g, P1, "Baneslayer Angel") && in_exile(&g, P1, "Grave Titan"));
    assert!(on_bf(&g, P1, "Wall of Omens"));
    assert_eq!(g.player(P1).life, life + 5 + 6, "each exile gains its own creature's power");
}

#[test]
fn an_evoked_solitude_exiles_what_the_person_picks_and_is_then_sacrificed() {
    let mut g = tbl();
    perm(&mut g, P1, "Baneslayer Angel");
    hand(&mut g, P0, &[SOL, "Swords to Plowshares"]);
    let life = g.player(P1).life;
    let words = ["evoke", "Baneslayer"].map(String::from);
    let first = Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return if std::mem::take(&mut *first.lock().unwrap()) {
                Answer::Action(json!({"do": "cast", "card": 0}))
            } else {
                Answer::Action(json!({"do": "pass"}))
            };
        }
        for w in &words {
            if let Some(k) = r.choices.iter().position(|c| c.contains(w.as_str())) {
                return Answer::Pick(k);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(!asks(&h, "Solitude: exile up to one").is_empty(), "the person picks the target: {:?}", h.asked());
    assert!(in_exile(&g, P1, "Baneslayer Angel"));
    assert_eq!(g.player(P1).life, life + 5);
    assert!(!has_perm(&g, P0, SOL), "the evoke sacrifice comes after the exile");
    assert!(g.player(P0).gy.contains(&card(&g, SOL)));
}
