//! P6 cards of the Yoshimaru / Tymna blink deck (impls/yoshimaru_p6.rs, t45_heliod_zur.rs, t45_urza_yawgmoth.rs): Sunpearl
//! Kirin, Haliya, Guided by Light and The Meathook Massacre. Each expectation follows the card's Oracle text.

mod play_common;
use play_common::*;
use sim_core::engine::zones;

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// a person at P0 who answers by what is asked: `pick` gives the index of the choice to take (None: decline); priority
/// is always passed. Every question is recorded.
fn person(
    g: &mut Game,
    pick: impl Fn(&Request) -> Option<usize> + Send + 'static,
) -> (std::sync::Arc<ScriptHuman>, std::sync::Arc<std::sync::Mutex<Vec<(String, Vec<String>)>>>) {
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let a2 = asked.clone();
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        a2.lock().unwrap().push((r.prompt.to_string(), r.choices.iter().map(|c| c.to_string()).collect()));
        match pick(r) {
            Some(i) => Answer::Pick(i),
            None => Answer::Decline,
        }
    }));
    g.humans.add(P0, h.clone());
    (h, asked)
}

fn choice_with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    has_perm(g, p, name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.contains(&card(g, name))
}

fn stock_library(g: &mut Game, p: PlayerId, n: usize) {
    let filler = card(g, "Plains");
    g.player_mut(p).library = vec![filler; n];
}

// ------------------------------------------------------------------ Sunpearl Kirin
// "Flash, flying. When this creature enters, return up to one other target nonland permanent you control to its owner's
// hand. If it was a token, draw a card."
#[test]
fn kirin_asks_a_person_which_permanent_to_return_and_returns_that_one() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    perm(&mut g, P0, "Solitude");
    perm(&mut g, P0, "Selfless Spirit");
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P0, "Plains", 2, false);
    let (_h, asked) = person(&mut g, |r| choice_with(r, "Selfless Spirit"));
    perm(&mut g, P0, "Sunpearl Kirin");
    let a = asked.lock().unwrap();
    assert_eq!(a.len(), 1, "one question: {a:?}");
    assert!(a[0].0.contains("Sunpearl Kirin"), "{a:?}");
    let c = &a[0].1;
    assert!(c.iter().any(|x| x.contains("Solitude")) && c.iter().any(|x| x.contains("Selfless Spirit")), "{c:?}");
    assert!(!c.iter().any(|x| x.contains("Plains")), "a land is not a nonland permanent: {c:?}");
    assert!(!c.iter().any(|x| x.contains("Kirin")), "'other': not itself: {c:?}");
    assert!(!c.iter().any(|x| x.contains("Sol Ring")), "an opponent's permanent is not offered: {c:?}");
    assert!(!on_bf(&g, P0, "Selfless Spirit") && in_hand(&g, P0, "Selfless Spirit"), "returned to its owner's hand");
    assert!(on_bf(&g, P0, "Solitude") && on_bf(&g, P0, "Sunpearl Kirin"));
    assert_eq!(g.player(P0).library.len(), 5, "it was not a token: no card is drawn");
}

#[test]
fn kirin_lets_a_person_return_nothing() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    perm(&mut g, P0, "Solitude");
    let (_h, asked) = person(&mut g, |_| None);
    perm(&mut g, P0, "Sunpearl Kirin");
    assert_eq!(asked.lock().unwrap().len(), 1, "asked once");
    assert!(on_bf(&g, P0, "Solitude") && g.player(P0).hand.is_empty(), "up to one: nothing returned");
    assert_eq!(g.player(P0).library.len(), 5, "and nothing drawn");
}

#[test]
fn kirin_draws_only_when_the_returned_permanent_was_a_token() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let t = token(&mut g, P0, 2);
    perm(&mut g, P0, "Solitude");
    let (_h, _asked) = person(&mut g, |r| {
        // the token is the choice that is not Solitude
        r.choices.iter().position(|c| !c.contains("Solitude"))
    });
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(!g.perm(t).on_bf, "the token is returned: it ceases to exist");
    assert_eq!(g.player(P0).hand.len(), 1, "and its controller draws a card");
    assert_eq!(g.player(P0).library.len(), 4);
    assert!(on_bf(&g, P0, "Solitude"));
}

#[test]
fn kirin_with_no_other_nonland_permanent_asks_nothing_and_draws_nothing() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    lands(&mut g, P0, "Plains", 2, false);
    let (_h, asked) = person(&mut g, |_| Some(0));
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(asked.lock().unwrap().is_empty(), "no target to choose");
    assert_eq!(g.player(P0).library.len(), 5, "no draw: nothing was a token");
}

#[test]
fn kirin_can_return_a_treasure_token_for_a_card() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    zones::add_treasure(&mut g, P0, 1).unwrap();
    let (_h, asked) = person(&mut g, |r| choice_with(r, "Treasure"));
    perm(&mut g, P0, "Sunpearl Kirin");
    assert_eq!(asked.lock().unwrap().len(), 1, "a Treasure is a nonland permanent: it is offered");
    assert_eq!(g.player(P0).treasures, 0, "the Treasure is returned: it ceases to exist");
    assert_eq!(g.player(P0).hand.len(), 1, "it was a token: a card is drawn");
}

#[test]
fn kirin_ai_returns_the_permanent_with_the_best_enter_effect() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    perm(&mut g, P0, "Selfless Spirit");
    perm(&mut g, P0, "Skyclave Apparition");
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(in_hand(&g, P0, "Skyclave Apparition") && !on_bf(&g, P0, "Skyclave Apparition"), "replayed for its enter");
    assert!(on_bf(&g, P0, "Selfless Spirit"), "the one with no enter effect stays");
    assert_eq!(g.player(P0).library.len(), 5, "not a token: no draw");
}

#[test]
fn kirin_ai_returns_nothing_when_no_enter_effect_is_worth_replaying() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    perm(&mut g, P0, "Selfless Spirit");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Inspiring Overseer");
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(on_bf(&g, P0, "Selfless Spirit") && on_bf(&g, P0, "Sol Ring"), "up to one: none chosen");
    assert!(g.player(P0).hand.is_empty() && g.player(P0).library.len() == 5, "nothing returned, nothing drawn");
    assert!(on_bf(&g, P1, "Inspiring Overseer"), "never an opponent's permanent");
}

#[test]
fn kirin_ai_trades_a_small_token_for_a_card() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let t = token(&mut g, P0, 1);
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(!g.perm(t).on_bf, "a 1/1 token is worth less than the card");
    assert_eq!(g.player(P0).hand.len(), 1);
    // a big token is kept
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let t = token(&mut g, P0, 6);
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(g.perm(t).on_bf && g.player(P0).hand.is_empty(), "a 6/6 token stays");
}

#[test]
fn kirin_is_a_flash_flyer_with_no_unconditional_draw() {
    let g = tbl();
    let d = g.db.get(card(&g, "Sunpearl Kirin"));
    assert!(d.has_kw("flash") && d.has_kw("flying"));
    assert!(!d.tags.has(sim_core::tag::Tag::Draw), "its draw is the token clause, in card code");
}
