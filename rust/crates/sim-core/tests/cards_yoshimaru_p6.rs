//! P6 cards of the Yoshimaru / Tymna blink deck (impls/yoshimaru_p6.rs, t45_heliod_zur.rs, t45_urza_yawgmoth.rs): Sunpearl
//! Kirin, Haliya, Guided by Light and The Meathook Massacre. Each expectation follows the card's Oracle text.

mod play_common;
use play_common::*;
use sim_core::ai::brain;
use sim_core::engine::{cast, hooks, stack, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::hooks::{Call, Event};
use sim_core::state::{Ctx, DataKey};
use sim_core::sym::intern;

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
    perm(&mut g, P0, "Solitude");
    perm(&mut g, P0, "Selfless Spirit");
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P0, "Plains", 2, false);
    stock_library(&mut g, P0, 5);
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
    lands(&mut g, P0, "Plains", 2, false);
    stock_library(&mut g, P0, 5);
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

// ------------------------------------------------------------------ Haliya, Guided by Light
// "Whenever Haliya or another creature or artifact you control enters, you gain 1 life. At the beginning of your end
// step, draw a card if you've gained 3 or more life this turn. Warp {W}"
const HALIYA: &str = "Haliya, Guided by Light";

fn run_plan(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.expect("an action") {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn hook_opts(g: &mut Game, c: sim_core::ids::CardId) -> Vec<Opt> {
    match g.registry.get(c).and_then(|i| i.hand_options) {
        Some(f) => f(g, c, P0, Some(false)).unwrap(),
        None => vec![],
    }
}

fn end_step(g: &mut Game, m: PermId) {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.end_step).expect("end step");
    f(g, m, P0).unwrap();
}

#[test]
fn haliya_gains_1_life_for_each_treasure_food_or_clue_token_you_create() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(g.player(P0).life, l0 + 2, "two Treasure tokens are two artifacts entering");
    zones::make_artifact_tokens(&mut g, P0, "Food", 1).unwrap();
    zones::make_artifact_tokens(&mut g, P0, "Clue", 1).unwrap();
    assert_eq!(g.player(P0).life, l0 + 4, "a Food and a Clue too");
    zones::add_treasure(&mut g, P1, 3).unwrap();
    assert_eq!(g.player(P0).life, l0 + 4, "an opponent's tokens are not yours");
}

#[test]
fn haliya_sees_a_clue_made_by_investigate_wherever_it_is_made() {
    // a compiled enters trigger (Thraben Inspector: a creature and a Clue enter)
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    perm(&mut g, P0, "Thraben Inspector");
    assert_eq!(g.player(P0).life, l0 + 2, "the creature and its Clue");
    // a spell with the investigate tag (Deduce: draw a card, investigate)
    let l1 = g.player(P0).life;
    let c = take(&mut g, P0, "Deduce");
    cast::resolve(&mut g, P0, c, &Ctx::default(), intern("hand")).unwrap();
    assert_eq!(g.player(P0).life, l1 + 1, "the Clue");
    // landfall card code (Tireless Tracker)
    perm(&mut g, P0, "Tireless Tracker");
    let l2 = g.player(P0).life;
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, l2 + 1, "the Clue");
}

#[test]
fn haliya_gains_for_an_artifact_or_creature_entering_and_for_itself() {
    let mut g = tbl();
    let l0 = g.player(P0).life;
    perm(&mut g, P0, HALIYA);
    assert_eq!(g.player(P0).life, l0 + 1, "Haliya itself");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mother of Runes");
    perm(&mut g, P0, "Exquisite Blood");
    assert_eq!(g.player(P0).life, l0 + 3, "an artifact and a creature; not an enchantment");
}

#[test]
fn haliya_draws_at_your_end_step_with_3_life_gained_and_not_with_2() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let h = perm(&mut g, P0, HALIYA);
    perm(&mut g, P0, "Sol Ring");
    end_step(&mut g, h);
    assert!(g.player(P0).hand.is_empty(), "2 life gained: no card");
    perm(&mut g, P0, "Mother of Runes");
    end_step(&mut g, h);
    assert_eq!(g.player(P0).hand.len(), 1, "3 life gained: a card");
}

#[test]
fn haliya_warp_is_cast_for_w_exiled_at_the_end_step_and_still_draws() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    stock_library(&mut g, P0, 5);
    let l0 = g.player(P0).life;
    let c = hand(&mut g, P0, &[HALIYA, "Sol Ring"])[0];
    let mut opts = hook_opts(&mut g, c);
    assert!(!opts.is_empty(), "warp is offered: {{2}}{{W}} is out of reach and a Sol Ring follows");
    assert!(run_plan(&mut g, opts.pop().unwrap(), P0));
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).cd == Some(c)).expect("Haliya is out");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    assert_eq!(g.player(P0).life, l0 + 1, "she gained 1 as she entered");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, l0 + 3);
    end_step(&mut g, m);
    assert!(!g.perm(m).on_bf, "exiled at the beginning of the end step");
    assert!(g.player(P0).exile.contains(&c), "in exile");
    assert_eq!(g.player(P0).library.len(), 4, "her end-step trigger was already on the stack: the card is drawn");
    assert!(sim_core::impls::generic_paths::warped_in_exile(&g, P0).is_empty(), "not on the turn it was exiled");
    g.round += 1;
    assert_eq!(sim_core::impls::generic_paths::warped_in_exile(&g, P0), vec![c], "castable on a later turn");
}

#[test]
fn haliya_warp_is_not_offered_for_nothing() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let c = hand(&mut g, P0, &[HALIYA])[0];
    assert!(g.registry.get(c).is_some_and(|i| i.hand_options.is_some()), "Haliya has a warp hook");
    assert!(hook_opts(&mut g, c).is_empty(), "nothing else would enter: no warp");
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let c = hand(&mut g, P0, &[HALIYA])[0];
    assert!(hook_opts(&mut g, c).is_empty(), "the full cost is payable and nothing else follows: cast her normally");
}

#[test]
fn a_person_can_warp_haliya_for_w() {
    let mut g = tbl();
    g.step = sim_core::state::Step::Main1;
    pool(&mut g, P0, &[('W', 1)]);
    let c = hand(&mut g, P0, &[HALIYA])[0];
    person(&mut g, |_| None);
    let ways = sim_core::human::alt::hand_alts(&mut g, P0, c).unwrap();
    let (label, o) = ways.into_iter().find(|(l, _)| l.contains("warp")).expect("a person is always offered the warp");
    assert!(label.contains("{W}"), "{label}");
    assert!(sim_core::human::alt::run(&mut g, P0, c, &o).unwrap().is_none());
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).cd == Some(c)).expect("Haliya is out");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    assert_eq!(pool_total(&g, P0), 0, "paid {{W}}");
}

// ------------------------------------------------------------------ The Meathook Massacre
// "When The Meathook Massacre enters, each creature gets -X/-X until end of turn. Whenever a creature you control dies,
// each opponent loses 1 life. Whenever a creature an opponent controls dies, you gain 1 life." X is chosen as it is
// cast: it is on the stack with the spell, where the table sees it.
const MEATHOOK: &str = "The Meathook Massacre";

/// answer X with `x`, recording every question asked
fn x_answer(x: u32, asked: std::sync::Arc<std::sync::Mutex<Vec<(String, Vec<String>)>>>) -> F {
    Box::new(move |r: &Request| {
        asked.lock().unwrap().push((r.prompt.to_string(), r.choices.iter().map(|c| c.to_string()).collect()));
        let want = format!("X = {x}");
        Answer::Pick(
            r.choices.iter().position(|c| *c == want).unwrap_or_else(|| panic!("no {want} in {:?}", r.choices)),
        )
    })
}

fn board_for_meathook(g: &mut Game) -> (PermId, PermId, PermId, PermId) {
    let mine = perm(g, P0, "Sheoldred, the Apocalypse");
    let small = perm(g, P1, "Ornithopter");
    let t2 = token(g, P1, 2);
    let big = perm(g, P1, "Sheoldred, the Apocalypse");
    (mine, small, t2, big)
}

#[test]
fn meathook_ai_chooses_x_as_it_casts_and_the_table_sees_it_on_the_stack() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 4, false);
    let (mine, small, t2, big) = board_for_meathook(&mut g);
    let c = hand(&mut g, P0, &[MEATHOOK])[0];
    // an opponent who is a person gets priority with the spell on the stack
    let seen = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let s2 = seen.clone();
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            s2.lock().unwrap().push(r.data.clone());
        }
        Answer::Action(json!({"do": "pass"}))
    }));
    g.humans.add(P1, h);
    let life = g.player(P0).life;
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    let seen = seen.lock().unwrap();
    let spell = seen
        .iter()
        .flat_map(|d| d["stack"].as_array().cloned().unwrap_or_default())
        .find(|it| it["name"] == MEATHOOK && it["kind"] == "spell")
        .expect("the spell was on the stack for the table");
    assert_eq!(spell["x"], 2, "X = 2 is chosen as it is cast and shown with the spell: {spell}");
    assert_eq!(total_mana(&g, P0), 0, "X = 2 and {{B}}{{B}} paid");
    assert!(!g.perm(small).on_bf && !g.perm(t2).on_bf, "-2/-2 kills the 0/2 and the 2/2");
    assert!(g.perm(big).on_bf && g.perm(mine).on_bf);
    assert_eq!(g.player(P0).life, life + 2, "two opposing creatures died: you gain 1 for each");
}

fn total_mana(g: &Game, p: PlayerId) -> u32 {
    sim_core::engine::mana::total_mana(g, p, false)
}

#[test]
fn meathook_person_chooses_x_at_the_cast_and_the_enter_trigger_uses_it() {
    let mut g = tbl();
    let (mine, small, t2, big) = board_for_meathook(&mut g);
    hand(&mut g, P0, &[MEATHOOK]);
    pool(&mut g, P0, &[('B', 2), ('C', 4)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(2, asked.clone())]);
    let a = asked.lock().unwrap();
    assert!(a[0].0.contains("The Meathook Massacre: choose X"), "{a:?}");
    assert_eq!(a.len(), 1, "X is asked once, at the cast: {a:?}");
    assert_eq!(pool_total(&g, P0), 2, "X = 2 and {{B}}{{B}} paid; nothing more is taken as it enters");
    assert!(!g.perm(small).on_bf && !g.perm(t2).on_bf, "each creature got -2/-2");
    assert!(g.perm(big).on_bf && g.perm(mine).on_bf);
}

#[test]
fn meathook_cast_with_x_zero_shrinks_nothing_and_takes_no_more_mana() {
    let mut g = tbl();
    let (_mine, small, t2, _big) = board_for_meathook(&mut g);
    hand(&mut g, P0, &[MEATHOOK]);
    pool(&mut g, P0, &[('B', 2), ('C', 4)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(0, asked.clone())]);
    assert_eq!(pool_total(&g, P0), 4, "X = 0: only {{B}}{{B}} is paid");
    assert!(g.perm(small).on_bf && g.perm(t2).on_bf, "-0/-0");
    assert!(has_perm(&g, P0, MEATHOOK));
}

#[test]
fn meathook_that_enters_without_being_cast_has_x_zero() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 4, false);
    let (_mine, small, t2, _big) = board_for_meathook(&mut g);
    perm(&mut g, P0, MEATHOOK);
    assert!(g.perm(small).on_bf && g.perm(t2).on_bf, "no X was chosen: nothing shrinks");
    assert_eq!(total_mana(&g, P0), 4, "and no mana is paid");
}

#[test]
fn meathook_drains_when_your_creature_dies_and_gains_when_an_opponents_does() {
    let mut g = tbl();
    perm(&mut g, P0, MEATHOOK);
    let mine = perm(&mut g, P0, "Mother of Runes");
    let theirs = perm(&mut g, P1, "Mother of Runes");
    let (l0, l1, l2) = (g.player(P0).life, g.player(P1).life, g.player(P2).life);
    zones::die(&mut g, mine, "destroy").unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 1, l2 - 1), "each opponent loses 1");
    zones::die(&mut g, theirs, "destroy").unwrap();
    assert_eq!(g.player(P0).life, l0 + 1, "you gain 1");
}

// ------------------------------------------------------------------ round 2: triggers that outlive their source
fn flush(g: &mut Game) {
    g.resolving -= 1;
    stack::flush_triggers(g).unwrap();
}

#[test]
fn kirins_trigger_resolves_even_if_kirin_is_gone() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let t = token(&mut g, P0, 1);
    g.resolving += 1;
    let k = perm(&mut g, P0, "Sunpearl Kirin");
    zones::die(&mut g, k, "destroy").unwrap();
    flush(&mut g);
    assert!(!g.perm(t).on_bf, "a queued enters trigger resolves without its source");
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn haliyas_enter_trigger_resolves_even_if_she_is_gone() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let l = g.player(P0).life;
    g.resolving += 1;
    perm(&mut g, P0, "Mother of Runes");
    let h = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == HALIYA).unwrap();
    zones::die(&mut g, h, "destroy").unwrap();
    flush(&mut g);
    assert_eq!(g.player(P0).life, l + 1, "the queued trigger still gains the life");
}

#[test]
fn haliyas_token_triggers_resolve_even_if_she_is_gone() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HALIYA);
    let l = g.player(P0).life;
    g.resolving += 1;
    zones::add_treasure(&mut g, P0, 3).unwrap();
    zones::die(&mut g, h, "destroy").unwrap();
    flush(&mut g);
    assert_eq!(g.player(P0).life - l, 3, "triggers already generated survive her removal");
}

#[test]
fn haliyas_end_step_draw_resolves_even_if_she_is_gone() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let h = perm(&mut g, P0, HALIYA);
    zones::add_treasure(&mut g, P0, 2).unwrap();
    g.resolving += 1;
    hooks::fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    zones::die(&mut g, h, "destroy").unwrap();
    flush(&mut g);
    assert_eq!(g.player(P0).library.len(), 4, "the end-step trigger is on the stack: the card is drawn");
}

#[test]
fn haliyas_end_step_trigger_always_triggers_and_checks_the_life_as_it_resolves() {
    // "draw a card if you've gained 3 or more life this turn" is not an intervening if: with no life gained the trigger
    // still goes on the stack (the probe reaches its window)
    let mut g = tbl();
    let h = perm(&mut g, P0, HALIYA);
    let c = g.perm(h).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.end_step).unwrap();
    g.player_mut(P0).gained_turn = None;
    g.trig_probe = true;
    let r = f(&mut g, h, P0);
    g.trig_probe = false;
    assert!(matches!(r, Err(sim_core::flow::Stop::Probe(_))), "it triggers with no life gained: {r:?}");
}

// ------------------------------------------------------------------ round 2: legality
#[test]
fn kirin_cannot_target_its_controllers_shrouded_permanent() {
    let mut g = tbl();
    let v = perm(&mut g, P0, "Selfless Spirit");
    let boots = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(boots).attached = Some(v);
    perm(&mut g, P0, "Solitude");
    let (_h, asked) = person(&mut g, |r| choice_with(r, "Solitude"));
    perm(&mut g, P0, "Sunpearl Kirin");
    let a = asked.lock().unwrap();
    assert!(
        !a.iter().any(|(_, cs)| cs.iter().any(|c| c.contains("Selfless Spirit"))),
        "shroud stops its controller: {a:?}"
    );
    assert!(a.iter().any(|(_, cs)| cs.iter().any(|c| c.contains("Solitude"))), "the unshrouded one is offered: {a:?}");
}

#[test]
fn kirin_cannot_target_a_permanent_with_protection_from_white() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let v = perm(&mut g, P0, "Skyclave Apparition");
    g.perm_mut(v).eot_kw.push("pro_W");
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(g.perm(v).on_bf, "the AI does not return a permanent Kirin can't target");
}

#[test]
fn warp_is_not_offered_or_cast_under_rule_of_law() {
    let mut g = tbl();
    g.step = sim_core::state::Step::Main1;
    perm(&mut g, P0, "Rule of Law");
    lands(&mut g, P0, "Plains", 2, false);
    pool(&mut g, P0, &[('W', 1)]);
    let c = hand(&mut g, P0, &[HALIYA, "Sol Ring"])[0];
    let sol = card(&g, "Sol Ring");
    cast::cast_card(&mut g, P0, sol, "hand", Ctx::default()).unwrap();
    hand(&mut g, P0, &["Ornithopter", "Memnite"]);
    assert!(!cast::castable(&g, P0, c, "hand"), "the one spell this turn is cast");
    assert!(hook_opts(&mut g, c).is_empty(), "no warp option for the AI");
    // the warp play itself checks it too (a person's menu offers it unchecked; its code must still refuse)
    person(&mut g, |_| None);
    if let Some((_, o)) =
        sim_core::human::alt::hand_alts(&mut g, P0, c).unwrap().into_iter().find(|(l, _)| l.contains("warp"))
    {
        assert!(sim_core::human::alt::run(&mut g, P0, c, &o).unwrap().is_some(), "the warp is refused");
    }
    assert!(!has_perm(&g, P0, HALIYA), "it was not cast");
}

#[test]
fn a_warped_haliya_is_not_recast_from_exile_under_drannith_magistrate() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let c = card(&g, HALIYA);
    g.player_mut(P0).exile.push(c);
    let st = g.turn_stamp();
    g.player_mut(P0).flag_turn.insert(intern("haliya_warped"), st);
    g.round += 1;
    let opts = |g: &mut Game| sim_core::impls::generic_paths::exile_options(g, P0, Some(false)).unwrap();
    assert_eq!(opts(&mut g).len(), 1, "castable without the Magistrate");
    perm(&mut g, P1, "Drannith Magistrate");
    assert!(opts(&mut g).is_empty(), "opponents can't cast spells from anywhere but their hand");
}

// ------------------------------------------------------------------ round 3: the enters dispatcher
// Elesh Norn, Mother of Machines: "If a permanent entering causes a triggered ability of a permanent you control to
// trigger, that ability triggers an additional time. Permanents entering don't cause abilities of permanents your
// opponents control to trigger." Panharmonicon: the same for an artifact or creature entering. Starfield Vocalist: the
// same as Elesh Norn's first sentence. Delney: a triggered ability of a creature you control with power 2 or less.
const ELESH: &str = "Elesh Norn, Mother of Machines";
const PANHARM: &str = "Panharmonicon";
const VOCALIST: &str = "Starfield Vocalist";

#[test]
fn panharmonicon_doubles_haliyas_life_for_an_artifact_token_entering() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, l0 + 2, "a Treasure is an artifact entering: 1 + Panharmonicon");
    let l1 = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, l1 + 2, "and a creature entering: 1 + Panharmonicon");
}

#[test]
fn a_vocalist_doubles_haliyas_life_for_an_artifact_token_entering() {
    let mut g = tbl();
    perm(&mut g, P0, VOCALIST);
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::make_artifact_tokens(&mut g, P0, "Food", 1).unwrap();
    assert_eq!(g.player(P0).life, l0 + 2, "1 + Starfield Vocalist");
}

#[test]
fn your_own_elesh_doubles_haliyas_life_for_each_artifact_token_entering() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(g.player(P0).life, l0 + 4, "each of two Treasures: 1 + Elesh Norn");
    let l1 = g.player(P0).life;
    zones::make_artifact_tokens(&mut g, P0, "Clue", 1).unwrap();
    assert_eq!(g.player(P0).life, l1 + 2, "a Clue too");
}

#[test]
fn panharmonicon_and_your_own_elesh_make_three_for_a_treasure() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, l0 + 3, "1 + Elesh Norn + Panharmonicon");
}

#[test]
fn an_opponents_elesh_blanks_haliyas_life_but_the_tokens_still_enter() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    perm(&mut g, P1, ELESH);
    let l0 = g.player(P0).life;
    let t0 = g.player(P0).treasures;
    zones::add_treasure(&mut g, P0, 2).unwrap();
    zones::make_artifact_tokens(&mut g, P0, "Food", 1).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 2, "the Treasures still enter");
    assert_eq!(g.player(P0).foods, 1, "and the Food");
    assert_eq!(g.player(P0).life, l0, "no entering causes Haliya's trigger");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(g.player(P0).life, l0, "nor does a permanent");
}

#[test]
fn delney_does_not_copy_haliyas_triggers_because_she_has_power_3() {
    let mut g = tbl();
    perm(&mut g, P0, "Delney, Streetwise Lookout");
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, l0 + 1, "the ability is Haliya's, a 3/3");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, l0 + 2, "a Treasure token is not a creature either");
}

#[test]
fn panharmonicon_doubles_haliyas_life_for_each_treasure_of_a_batch() {
    // two Treasures entering together: each is its own trigger, each with its additional time
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, HALIYA);
    let l0 = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(g.player(P0).life, l0 + 4, "two Treasures, each 1 + Panharmonicon");
}

#[test]
fn kirins_enter_trigger_doubles_under_the_event_doublers_and_blanks_under_an_opponents_elesh() {
    // two 1/1 tokens: each trigger returns one of them and draws a card
    let run = |own: Option<&str>, opp_elesh: bool| {
        let mut g = tbl();
        stock_library(&mut g, P0, 5);
        if let Some(n) = own {
            perm(&mut g, P0, n);
        }
        if opp_elesh {
            perm(&mut g, P1, ELESH);
        }
        let a = token(&mut g, P0, 1);
        let b = token(&mut g, P0, 1);
        perm(&mut g, P0, "Sunpearl Kirin");
        (g.player(P0).hand.len(), g.perm(a).on_bf as u32 + g.perm(b).on_bf as u32)
    };
    assert_eq!(run(None, false), (1, 1), "one trigger: one token returned, one card");
    assert_eq!(run(Some(PANHARM), false), (2, 0), "Kirin is a creature: 1 + Panharmonicon");
    assert_eq!(run(Some(VOCALIST), false), (2, 0), "1 + Starfield Vocalist");
    assert_eq!(run(Some(ELESH), false), (2, 0), "1 + Elesh Norn");
    assert_eq!(run(None, true), (0, 2), "an opponent's Elesh Norn: Kirin's trigger does not happen");
}

/// cast The Meathook Massacre with X = x through the person's cast, with `setup` on the table first
fn meathook_with(x: u32, setup: impl Fn(&mut Game)) -> (Game, PermId, PermId) {
    let mut g = tbl();
    setup(&mut g);
    let small = perm(&mut g, P1, "Ornithopter"); // 0/2
    let t2 = token(&mut g, P1, 2);
    hand(&mut g, P0, &[MEATHOOK]);
    pool(&mut g, P0, &[('B', 2), ('C', 4)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), x_answer(x, asked)]);
    assert!(has_perm(&g, P0, MEATHOOK), "it was cast");
    (g, small, t2)
}

#[test]
fn meathooks_enter_trigger_doubles_under_elesh_and_a_vocalist_and_blanks_under_an_opponents_elesh() {
    let (g, small, t2) = meathook_with(1, |_| {});
    assert!(g.perm(small).on_bf && g.perm(t2).on_bf, "X = 1 once: -1/-1 kills neither");
    let (g, small, t2) = meathook_with(1, |g| {
        perm(g, P0, ELESH);
    });
    assert!(!g.perm(small).on_bf && !g.perm(t2).on_bf, "1 + Elesh Norn: -1/-1 twice kills both");
    let (g, small, t2) = meathook_with(1, |g| {
        perm(g, P0, VOCALIST);
    });
    assert!(!g.perm(small).on_bf && !g.perm(t2).on_bf, "1 + Starfield Vocalist");
    let (g, small, t2) = meathook_with(2, |g| {
        perm(g, P1, ELESH);
    });
    assert!(g.perm(small).on_bf && g.perm(t2).on_bf, "an opponent's Elesh Norn: nothing shrinks");
}

#[test]
fn panharmonicon_does_not_double_meathook_because_it_is_an_enchantment() {
    let (g, small, t2) = meathook_with(1, |g| {
        perm(g, P0, PANHARM);
    });
    assert!(g.perm(small).on_bf && g.perm(t2).on_bf, "neither an artifact nor a creature entering: one trigger");
}

// ------------------------------------------------------------------ round 4: a triggered ability keeps its controller
// CR 603.3a: a triggered ability is controlled by whoever controlled its source as it triggered; a change of control of
// the source afterwards does not change it (113.8: it exists independently of its source).
fn steal(g: &mut Game, by: PlayerId, m: PermId) {
    use sim_core::engine::control::{Duration, gain_control};
    assert!(gain_control(g, by, m, Duration::Permanent), "control of the source changed");
}

#[test]
fn kirins_trigger_keeps_its_controller_when_kirin_is_stolen() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    let t = token(&mut g, P0, 1);
    g.resolving += 1;
    let k = perm(&mut g, P0, "Sunpearl Kirin");
    steal(&mut g, P1, k);
    flush(&mut g);
    assert!(!g.perm(t).on_bf, "the trigger is still P0's: it returns P0's token");
    assert_eq!(g.player(P0).library.len(), 4, "and P0 draws");
}

#[test]
fn haliyas_token_triggers_keep_their_controller_when_she_is_stolen() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    g.resolving += 1;
    zones::add_treasure(&mut g, P0, 2).unwrap();
    steal(&mut g, P1, h);
    flush(&mut g);
    assert_eq!(g.player(P0).life - life, 2, "P0's queued token triggers survive the change of control");
}

#[test]
fn haliyas_enter_trigger_keeps_its_controller_when_she_is_stolen() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    let l1 = g.player(P1).life;
    g.resolving += 1;
    perm(&mut g, P0, "Mother of Runes");
    steal(&mut g, P1, h);
    flush(&mut g);
    assert_eq!(g.player(P0).life - life, 1, "the trigger P0 had when the creature entered gains P0's life");
    assert_eq!(g.player(P1).life, l1, "not the thief's");
}

#[test]
fn haliyas_end_step_trigger_keeps_its_controller_when_she_is_stolen() {
    let mut g = tbl();
    stock_library(&mut g, P0, 5);
    stock_library(&mut g, P1, 5);
    let h = perm(&mut g, P0, HALIYA);
    sim_core::engine::life::gain(&mut g, P0, 3).unwrap();
    g.resolving += 1;
    hooks::fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    steal(&mut g, P1, h);
    flush(&mut g);
    assert_eq!(g.player(P0).library.len(), 4, "P0's end-step trigger is still P0's: P0 draws");
    assert_eq!(g.player(P1).library.len(), 5, "the thief draws nothing");
}

#[test]
fn a_stolen_haliya_triggers_for_her_new_controller_only() {
    let mut g = tbl();
    stock_library(&mut g, P1, 5);
    let h = perm(&mut g, P0, HALIYA);
    steal(&mut g, P1, h);
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    perm(&mut g, P0, "Mother of Runes");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, l0, "her old controller's permanents no longer set her off");
    perm(&mut g, P1, "Mother of Runes");
    zones::add_treasure(&mut g, P1, 1).unwrap();
    assert_eq!(g.player(P1).life - l1, 2, "her new controller's do");
    sim_core::engine::life::gain(&mut g, P1, 1).unwrap();
    hooks::fire_trigger(&mut g, Event::EndStep, Call::Player { p: P1 }).unwrap();
    assert_eq!(g.player(P1).library.len(), 4, "and her new controller's end step draws");
}

// ------------------------------------------------------------------ round 4: creature tokens, one trigger per token, lands
// "Whenever Haliya or another creature or artifact you control enters": a creature token entering is a creature entering
// (it triggers her), each artifact entering is its own triggered ability (CR 603.2c), and a land that is also a creature
// or an artifact counts.
#[test]
fn haliya_gains_for_each_ordinary_creature_token() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    zones::make_tokens(&mut g, P0, zones::Tokens::new(3, 1)).unwrap();
    assert_eq!(g.player(P0).life - life, 3, "three creature tokens entering are three triggers");
}

#[test]
fn panharmonicon_doubles_haliyas_life_for_each_creature_token() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(g.player(P0).life - life, 4, "two creature tokens, each 1 + Panharmonicon");
}

#[test]
fn your_own_elesh_doubles_haliyas_life_for_each_creature_token() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(g.player(P0).life - life, 4, "two creature tokens, each 1 + Elesh Norn");
}

#[test]
fn an_opponents_elesh_blanks_haliyas_life_for_creature_tokens_but_they_enter() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    perm(&mut g, P1, ELESH);
    let life = g.player(P0).life;
    let toks = zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|&t| g.perm(t).on_bf), "the tokens are there");
    assert_eq!(g.player(P0).life, life, "no entering causes Haliya's trigger");
}

#[test]
fn an_artifact_creature_token_and_a_treasure_are_two_enters_not_three() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    let spec = zones::Tokens { types: vec!["artifact"], ..zones::Tokens::new(1, 1) };
    zones::make_tokens(&mut g, P0, spec).unwrap();
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life - life, 2, "the artifact creature once, the Treasure once");
}

/// how many stack items a priority question showed at most, whose name contains `who` and text `what`
fn most_items_seen(
    g: &mut Game,
    who: &'static str,
    what: &'static str,
) -> std::sync::Arc<std::sync::Mutex<Vec<usize>>> {
    let counts = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let out = counts.clone();
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            let n = r.data["stack"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|x| {
                    x["name"].as_str().is_some_and(|s| s.contains(who))
                        && x["text"].as_str().is_some_and(|s| s.contains(what))
                })
                .count();
            out.lock().unwrap().push(n);
        }
        Answer::Action(json!({"do": "pass"}))
    }));
    h.set_autopass("stack");
    g.humans.add(P1, h.clone());
    g.humans.add(P0, h);
    counts
}

#[test]
fn each_artifact_token_entering_is_its_own_haliya_trigger_on_the_stack() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let counts = most_items_seen(&mut g, "Haliya", "gain 1");
    zones::add_treasure(&mut g, P0, 3).unwrap();
    let c = counts.lock().unwrap();
    assert_eq!(c.iter().copied().max(), Some(3), "three artifact entries trigger three distinct abilities: {c:?}");
}

#[test]
fn each_artifact_token_entering_is_its_own_fireweaver_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Reckless Fireweaver");
    let counts = most_items_seen(&mut g, "Fireweaver", "1 damage");
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    zones::add_treasure(&mut g, P0, 3).unwrap();
    let c = counts.lock().unwrap();
    assert_eq!(c.iter().copied().max(), Some(3), "three artifacts entering: three triggers of 1 damage: {c:?}");
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 3, l2 - 3), "3 damage in all");
}

#[test]
fn a_copy_of_one_haliya_token_trigger_gains_one_extra() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    pool(&mut g, P0, &[('C', 2)]);
    let idx = g.player(P0).perms.iter().position(|&m| m == cam).unwrap();
    let mut used = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !used && r.prompt.contains("Haliya") && !r.prompt.contains("copy of") {
                used = true;
                return Answer::Action(json!({"do":"use","perm":idx}));
            }
            return Answer::Action(json!({"do":"pass"}));
        }
        Answer::Pick(0)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    let life = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 3).unwrap();
    assert!(g.perm(cam).tapped, "the Camera was actually activated");
    assert_eq!(g.player(P0).life - life, 4, "three original triggers plus one copied trigger");
}

/// p plays this land card from the hand: the land event (and landfall) come with the real play
fn play_land(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    sim_core::engine::turn::play_land_card(g, p, c, "plays").unwrap();
    *g.player(p).lands.last().unwrap()
}

#[test]
fn haliya_gains_for_a_creature_land_and_an_artifact_land_but_not_a_basic() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    play_land(&mut g, P0, "Forest");
    assert_eq!(g.player(P0).life, life, "a basic land is neither a creature nor an artifact");
    play_land(&mut g, P0, "Dryad Arbor");
    assert_eq!(g.player(P0).life - life, 1, "a land creature entering");
    play_land(&mut g, P0, "Darksteel Citadel");
    assert_eq!(g.player(P0).life - life, 2, "an artifact land entering");
}

#[test]
fn panharmonicon_doubles_haliyas_life_for_a_creature_land() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, HALIYA);
    let life = g.player(P0).life;
    play_land(&mut g, P0, "Dryad Arbor");
    assert_eq!(g.player(P0).life - life, 2, "1 + Panharmonicon");
}

#[test]
fn an_opponents_elesh_blanks_haliyas_life_for_a_creature_land() {
    let mut g = tbl();
    perm(&mut g, P0, HALIYA);
    perm(&mut g, P1, ELESH);
    let life = g.player(P0).life;
    play_land(&mut g, P0, "Dryad Arbor");
    assert_eq!(g.player(P0).life, life, "no entering causes her trigger");
}
