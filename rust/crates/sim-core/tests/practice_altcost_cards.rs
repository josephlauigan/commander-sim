//! Practice mode (2026-11-05): every alternative cost and graveyard cost in JD's decks, paid by the person.
//! Counterspells (Force of Will, Force of Negation, Fierce Guardianship, Pact of Negation, Mental Misstep, Mindbreak
//! Trap) are cast in the response window to an opponent's spell; Snapback, Baleful Mastery and Deadly Rollick from the
//! hand in the main phase; Think Twice, Daydream, Nibelheim Aflame and Reconstructed Thopter from the graveyard;
//! Stinkweed Imp dredges in the draw step. Each is checked against its Oracle text: the cost paid, the effect, and the
//! zone the card ends in.

mod play_common;
use play_common::*;
use sim_core::ids::CardId;
use sim_core::state::{Ctx, DataKey};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// a person who, at their first priority, does `first`, then passes; other questions take `answers` in order (then
/// the first choice, or yes)
fn responder(g: &mut Game, p: PlayerId, first: Value, answers: Vec<F>) -> Arc<ScriptHuman> {
    let first = Mutex::new(Some(first));
    let rest = Mutex::new(answers.into_iter().collect::<VecDeque<F>>());
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(first.lock().unwrap().take().unwrap_or_else(|| json!({"do": "pass"})));
        }
        if let Some(mut f) = rest.lock().unwrap().pop_front() {
            return f(r);
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(p, h.clone());
    h
}

/// P1 casts `spell` from hand; true if it was countered
fn opponent_casts(g: &mut Game, spell: &str) -> bool {
    let s = hand(g, P1, &[spell])[0];
    let resolved = sim_core::engine::cast::cast_card(g, P1, s, "hand", Ctx::default()).unwrap_or(true);
    !resolved
}

fn named(g: &Game, cs: &[CardId], name: &str) -> bool {
    cs.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    named(g, &g.player(p).exile, name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    named(g, &g.player(p).hand, name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    named(g, &g.player(p).gy, name)
}

fn asked(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| format!("{}: {} {:?}", r.kind, r.prompt, r.choices)).collect()
}

fn told(h: &ScriptHuman) -> Vec<String> {
    h.told().into_iter().map(|t| t.1).collect()
}

/// the opponent's turn, the person (P0) holding `cards` (the counterspell first) with no mana
fn counter_table(key: &str, cards: &[&str]) -> Game {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P1);
    hand(&mut g, P0, cards);
    g
}

fn commander_out(g: &mut Game, p: PlayerId, name: &str) {
    let m = perm(g, p, name);
    g.perm_mut(m).is_cmd = true;
}

// ------------------------------------------------------------------ Force of Will

#[test]
fn force_of_will_with_no_mana_pitches_a_blue_card_and_one_life() {
    let mut g = counter_table("thanos", &["Force of Will", "Brainstorm"]);
    let life = g.player(P0).life;
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Brainstorm"), "Brainstorm exiled");
    assert!(in_gy(&g, P0, "Force of Will"));
    assert_eq!(g.player(P0).life, life - 1, "1 life paid");
}

#[test]
fn force_of_will_with_mana_open_you_may_still_pitch() {
    let mut g = counter_table("thanos", &["Force of Will", "Brainstorm"]);
    pool(&mut g, P0, &[('U', 5)]);
    let life = g.player(P0).life;
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![by_text("exile a blue card")]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(asked(&h).iter().any(|a| a.contains("how do you pay")), "asked how to pay: {:?}", asked(&h));
    assert!(in_exile(&g, P0, "Brainstorm"));
    assert_eq!(g.player(P0).life, life - 1);
    assert_eq!(pool_total(&g, P0), 5, "no mana spent");
}

#[test]
fn force_of_will_with_mana_open_you_may_pay_the_mana() {
    let mut g = counter_table("thanos", &["Force of Will", "Brainstorm"]);
    pool(&mut g, P0, &[('U', 5)]);
    let life = g.player(P0).life;
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![by_text("pay {3}{U}{U}")]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_hand(&g, P0, "Brainstorm"), "nothing pitched");
    assert_eq!(g.player(P0).life, life);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn force_of_will_may_pitch_a_phyrexian_blue_card() {
    // Gitaxian Probe ({U/P}) is a blue card
    let mut g = counter_table("thanos", &["Force of Will", "Gitaxian Probe"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Gitaxian Probe"));
}

#[test]
fn force_of_will_asks_which_blue_card_to_pitch() {
    let mut g = counter_table("thanos", &["Force of Will", "Brainstorm", "Mystical Tutor"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![by_text("Mystical Tutor")]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Mystical Tutor"), "the card you chose");
    assert!(in_hand(&g, P0, "Brainstorm"));
}

#[test]
fn force_of_will_with_no_blue_card_and_no_mana_is_refused() {
    let mut g = counter_table("thanos", &["Force of Will", "Sol Ring"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(!opponent_casts(&mut g, "Demonic Tutor"));
    assert!(in_hand(&g, P0, "Force of Will"));
    assert!(told(&h).iter().any(|t| t.contains("Can't cast Force of Will")), "told {:?}", told(&h));
}

// ------------------------------------------------------------------ Force of Negation

#[test]
fn force_of_negation_pitches_on_an_opponents_turn() {
    let mut g = counter_table("thanos", &["Force of Negation", "Brainstorm"]);
    let life = g.player(P0).life;
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Brainstorm"));
    assert!(in_exile(&g, P1, "Demonic Tutor"), "the countered spell is exiled");
    assert_eq!(g.player(P0).life, life, "no life paid");
}

#[test]
fn force_of_negation_cant_pitch_on_your_own_turn() {
    let mut g = counter_table("thanos", &["Force of Negation", "Brainstorm"]);
    g.active = Some(P0);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(!opponent_casts(&mut g, "Brainstorm"), "not countered");
    assert!(in_hand(&g, P0, "Force of Negation"));
    assert!(in_hand(&g, P0, "Brainstorm"), "nothing pitched; told {:?}", told(&h));
}

// ------------------------------------------------------------------ Fierce Guardianship, Pact, Misstep, Mindbreak Trap

#[test]
fn fierce_guardianship_is_free_with_your_commander_out() {
    let mut g = counter_table("thanos", &["Fierce Guardianship"]);
    commander_out(&mut g, P0, "Thanos, the Mad Titan");
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
}

#[test]
fn fierce_guardianship_without_your_commander_needs_its_mana() {
    let mut g = counter_table("thanos", &["Fierce Guardianship"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(!opponent_casts(&mut g, "Demonic Tutor"));
    assert!(in_hand(&g, P0, "Fierce Guardianship"), "told {:?}", told(&h));
}

#[test]
fn pact_of_negation_is_free_now_and_owed_at_upkeep() {
    let mut g = counter_table("thanos", &["Pact of Negation"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert_eq!(g.player(P0).pacts, 1, "a pact owed");
}

#[test]
fn mental_misstep_for_two_life_at_low_life() {
    // Oracle: "({U/P} can be paid with either {U} or 2 life.)": no life floor for the person
    let mut g = counter_table("thanos", &["Mental Misstep"]);
    g.player_mut(P0).life = 8;
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Brainstorm"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert_eq!(g.player(P0).life, 6);
}

#[test]
fn mindbreak_trap_is_free_after_three_opponent_spells() {
    let mut g = table(&["thanos", "seph"]);
    g.active = Some(P1);
    for s in ["Brainstorm", "Opt"] {
        opponent_casts(&mut g, s);
    }
    hand(&mut g, P0, &["Mindbreak Trap"]);
    let h = responder(&mut g, P0, json!({"do": "cast", "card": 0}), vec![]);
    assert!(opponent_casts(&mut g, "Demonic Tutor"), "countered; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P1, "Demonic Tutor"), "exiled");
}

// ------------------------------------------------------------------ non-counter spells from hand

#[test]
fn snapback_pitches_a_blue_card() {
    let mut g = table(&["thanos", "seph"]);
    hand(&mut g, P0, &["Snapback", "Brainstorm"]);
    perm(&mut g, P1, "Grave Titan");
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan")]);
    assert!(in_hand(&g, P1, "Grave Titan"), "bounced; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Brainstorm"));
    assert!(in_gy(&g, P0, "Snapback"));
}

#[test]
fn baleful_mastery_for_one_and_black_and_an_opponent_draws() {
    let mut g = table(&["sauron", "seph"]);
    hand(&mut g, P0, &["Baleful Mastery"]);
    perm(&mut g, P1, "Grave Titan");
    pool(&mut g, P0, &[('B', 2)]);
    let opp_hand = g.player(P1).hand.len();
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan")]);
    assert!(in_exile(&g, P1, "Grave Titan"), "exiled; asked {:?} told {:?}", asked(&h), told(&h));
    assert_eq!(g.player(P1).hand.len(), opp_hand + 1, "the opponent drew");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn baleful_mastery_you_choose_which_opponent_draws() {
    let mut g = table(&["sauron", "seph", "kefka"]);
    hand(&mut g, P0, &["Baleful Mastery"]);
    perm(&mut g, P1, "Grave Titan");
    pool(&mut g, P0, &[('B', 2)]);
    let (h1, h2) = (g.player(P1).hand.len(), g.player(P2).hand.len());
    let who = g.player(P2).name.to_string();
    let h = main(
        &mut g,
        P0,
        vec![
            act(json!({"do": "cast", "card": 0})),
            by_text("Grave Titan"),
            Box::new(move |r: &Request| {
                Answer::Pick(r.choices.iter().position(|x| *x == who).expect("the opponents listed"))
            }),
        ],
    );
    assert!(asked(&h).iter().any(|a| a.contains("which opponent draws")), "asked {:?}", asked(&h));
    assert_eq!((g.player(P1).hand.len(), g.player(P2).hand.len()), (h1, h2 + 1), "the one you chose drew");
}

#[test]
fn baleful_mastery_full_cost_no_one_draws() {
    let mut g = table(&["sauron", "seph"]);
    hand(&mut g, P0, &["Baleful Mastery"]);
    perm(&mut g, P1, "Grave Titan");
    pool(&mut g, P0, &[('B', 4)]);
    let opp_hand = g.player(P1).hand.len();
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan"), by_text("pay {3}{B}")]);
    assert!(in_exile(&g, P1, "Grave Titan"));
    assert_eq!(g.player(P1).hand.len(), opp_hand, "no draw");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn deadly_rollick_is_free_with_your_commander_out() {
    let mut g = table(&["thanos", "seph"]);
    hand(&mut g, P0, &["Deadly Rollick"]);
    commander_out(&mut g, P0, "Thanos, the Mad Titan");
    perm(&mut g, P1, "Grave Titan");
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan")]);
    assert!(in_exile(&g, P1, "Grave Titan"), "exiled; asked {:?} told {:?}", asked(&h), told(&h));
}

#[test]
fn deadly_rollick_without_your_commander_needs_its_mana() {
    let mut g = table(&["thanos", "seph"]);
    hand(&mut g, P0, &["Deadly Rollick"]);
    perm(&mut g, P1, "Grave Titan");
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(has_perm(&g, P1, "Grave Titan"));
    assert!(in_hand(&g, P0, "Deadly Rollick"), "told {:?}", told(&h));
}

// ------------------------------------------------------------------ from the graveyard

fn gy_cast() -> F {
    act(json!({"do": "cast", "zone": "gy", "card": 0}))
}

#[test]
fn think_twice_flashback_draws_and_is_exiled() {
    let mut g = table(&["kefka", "seph"]);
    let c = take(&mut g, P0, "Think Twice");
    g.player_mut(P0).gy = vec![c];
    pool(&mut g, P0, &[('U', 3)]);
    let n = g.player(P0).hand.len();
    let h = main(&mut g, P0, vec![gy_cast()]);
    assert_eq!(g.player(P0).hand.len(), n + 1, "drew; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(in_exile(&g, P0, "Think Twice"), "exiled after flashback");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn twinned_vision_flashback_discards_draws_two_and_is_exiled() {
    let mut g = table(&["kefka", "seph"]);
    let tv = take(&mut g, P0, "Twinned Vision");
    g.player_mut(P0).gy = vec![tv];
    pool(&mut g, P0, &[('U', 3)]);
    hand(&mut g, P0, &["Sol Ring"]);
    let n = g.player(P0).hand.len();
    let h = main(&mut g, P0, vec![gy_cast(), pick(0)]);
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2, "discarded one, drew two; asked {:?}", asked(&h));
    assert!(in_gy(&g, P0, "Sol Ring"), "the discard");
    assert!(in_exile(&g, P0, "Twinned Vision"), "exiled after flashback");
}

#[test]
fn daydream_flashback_blinks_with_a_counter_and_is_exiled() {
    let mut g = table(&["alela", "seph"]);
    let c = take(&mut g, P0, "Daydream");
    g.player_mut(P0).gy = vec![c];
    perm(&mut g, P0, "Grave Titan");
    pool(&mut g, P0, &[('W', 3)]);
    let h = main(&mut g, P0, vec![gy_cast(), by_text("Grave Titan")]);
    let m = perms_named(&g, P0, "Grave Titan");
    assert_eq!(m.len(), 1, "asked {:?} told {:?}", asked(&h), told(&h));
    assert_eq!(g.perm(m[0]).plus, 1, "a +1/+1 counter");
    assert!(in_exile(&g, P0, "Daydream"), "exiled after flashback");
}

#[test]
fn nibelheim_aflame_flashback_discards_your_hand() {
    let mut g = table(&["sauron", "seph"]);
    let c = take(&mut g, P0, "Nibelheim Aflame");
    g.player_mut(P0).gy = vec![c];
    perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P1, "Llanowar Elves");
    hand(&mut g, P0, &["Sol Ring", "Mind Stone"]);
    pool(&mut g, P0, &[('R', 7)]);
    let h = main(&mut g, P0, vec![gy_cast(), by_text("Grave Titan")]);
    assert!(
        !has_perm(&g, P1, "Llanowar Elves"),
        "6 damage to each other creature; asked {:?} told {:?}",
        asked(&h),
        told(&h)
    );
    assert!(in_gy(&g, P0, "Sol Ring") && in_gy(&g, P0, "Mind Stone"), "discarded your hand");
    assert!(in_exile(&g, P0, "Nibelheim Aflame"), "exiled after flashback");
}

#[test]
fn reconstructed_thopter_unearths_with_haste() {
    let mut g = table(&["alela", "seph"]);
    let c = take(&mut g, P0, "Reconstructed Thopter");
    g.player_mut(P0).gy = vec![c];
    pool(&mut g, P0, &[('C', 2)]);
    let h = main(&mut g, P0, vec![gy_cast()]);
    let m = perms_named(&g, P0, "Reconstructed Thopter");
    assert_eq!(m.len(), 1, "unearthed; asked {:?} told {:?}", asked(&h), told(&h));
    assert!(!g.perm(m[0]).sick, "haste");
    assert!(g.perm(m[0]).data.truthy(DataKey::Unearth), "exiled at the next end step");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn stinkweed_imp_dredges_instead_of_drawing() {
    let mut g = table(&["kefka", "seph"]);
    let c = take(&mut g, P0, "Stinkweed Imp");
    g.player_mut(P0).gy = vec![c];
    seat(&mut g, P0, vec![by_text("yes")]);
    assert!(sim_core::human::cards::dredge(&mut g, P0).unwrap());
    assert!(in_hand(&g, P0, "Stinkweed Imp"));
    assert_eq!(g.player(P0).gy.len(), 5, "milled five");
}
