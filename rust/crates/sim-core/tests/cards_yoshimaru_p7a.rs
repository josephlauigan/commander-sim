//! Person paths of the Yoshimaru / Tymna deck, part A: the cards a person could not play (or could not play fully)
//! though the AI could. Each test seats a person at P0, asks for the action and checks what the person was asked and
//! what happened. Oracle text rules every expectation.

mod play_common;
use play_common::*;
use sim_core::human::human::{human_main, respond};
use sim_core::human::{Target, legal};
use sim_core::state::Step;
use sim_core::state::{DataKey, Val};
use std::collections::VecDeque;
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

/// the person has priority in a window on someone else's turn (`respond`): `actions` are taken one at a time at the
/// priority questions (then it passes), the other questions are answered by `rules`
fn respond_play(g: &mut Game, actions: Vec<Value>, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let queue = Mutex::new(VecDeque::from(actions));
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return match queue.lock().unwrap().pop_front() {
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
    respond(g, P0, "P1's declare attackers step", None, None).unwrap();
    h
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

// ------------------------------------------------------------------ The Wandering Emperor
const EMP: &str = "The Wandering Emperor";

/// P1's turn, its combat: a tapped attacker of P1's
fn opp_attack(g: &mut Game) -> PermId {
    g.active = Some(P1);
    g.step = Step::Combat;
    let atk = perm(g, P1, "Llanowar Elves");
    g.perm_mut(atk).tapped = true;
    atk
}

/// the loyalty The Wandering Emperor enters with
fn emp_start(g: &Game) -> i32 {
    g.db.get(card(g, EMP)).start_loyalty.expect("a planeswalker")
}

fn use_emp(i: usize) -> Value {
    json!({"do": "use", "perm": i})
}

fn exiled(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn emperor_flashed_in_on_the_attack_exiles_a_tapped_attacker() {
    let mut g = tbl();
    let atk = opp_attack(&mut g);
    hand(&mut g, P0, &[EMP]);
    pool(&mut g, P0, &[('W', 2), ('C', 2)]);
    let tg = ans_target(&g, "Wandering Emperor", Target::Perm(atk));
    let h = respond_play(
        &mut g,
        vec![json!({"do": "cast", "card": 0}), use_emp(0)],
        vec![ans_word("which ability", "-2"), tg],
    );
    assert!(!g.perm(atk).on_bf && exiled(&g, P1, "Llanowar Elves"), "{:?}", h.told());
    let e = perms_named(&g, P0, EMP)[0];
    assert_eq!(g.perm(e).loyalty, Some(emp_start(&g) - 2));
    assert_eq!(g.player(P0).life, 42, "gains 2");
}

#[test]
fn emperor_that_did_not_enter_this_turn_is_sorcery_speed_on_an_opponents_turn() {
    let mut g = tbl();
    let e = perm(&mut g, P0, EMP);
    g.turn_enter = g.enter_no; // she entered on an earlier turn
    let atk = opp_attack(&mut g);
    let tg = ans_target(&g, "Wandering Emperor", Target::Perm(atk));
    let h = respond_play(&mut g, vec![use_emp(0)], vec![ans_word("which ability", "-2"), tg]);
    assert!(g.perm(atk).on_bf, "not exiled");
    assert_eq!(g.perm(e).loyalty, Some(emp_start(&g)), "no loyalty paid");
    assert!(told_last(&h).contains("isn't your turn"), "{:?}", h.told());
}

#[test]
fn emperor_activates_once_per_turn_even_in_the_flash_window() {
    let mut g = tbl();
    let atk = opp_attack(&mut g);
    hand(&mut g, P0, &[EMP]);
    pool(&mut g, P0, &[('W', 2), ('C', 2)]);
    let tg = ans_target(&g, "Wandering Emperor", Target::Perm(atk));
    let h = respond_play(
        &mut g,
        vec![json!({"do": "cast", "card": 0}), use_emp(0), use_emp(0)],
        vec![ans_word("which ability", "-1"), ans_word("which ability", "-2"), tg],
    );
    let e = perms_named(&g, P0, EMP)[0];
    assert_eq!(g.perm(e).loyalty, Some(emp_start(&g) - 1), "the first ability (-1) was paid once: {:?}", h.told());
    assert!(told_last(&h).contains("already used a loyalty ability"), "{:?}", h.told());
    assert!(g.perm(atk).on_bf);
}

#[test]
fn emperor_plus_one_gives_an_opponents_creature_a_counter_and_first_strike() {
    let mut g = tbl();
    let e = perm(&mut g, P0, EMP);
    g.active = Some(P1);
    g.step = Step::Combat;
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    let tg = ans_target(&g, "Wandering Emperor", Target::Perm(theirs));
    respond_play(&mut g, vec![use_emp(0)], vec![ans_word("which ability", "+1"), tg]);
    assert_eq!(g.perm(theirs).plus, 1);
    assert!(has_kw(&g, theirs, "first strike"), "{:?}", g.perm(theirs).eot_kw);
    assert_eq!(g.perm(e).loyalty, Some(emp_start(&g) + 1));
}

#[test]
fn emperor_plus_one_may_have_no_target_and_still_gains_loyalty() {
    let mut g = tbl();
    let e = perm(&mut g, P0, EMP);
    let mine = perm(&mut g, P0, "Llanowar Elves");
    let h = use_perm(&mut g, e, vec![ans_word("which ability", "+1"), ans_word("up to one", "no target")]);
    assert!(asked(&h, "up to one") || asked(&h, "Wandering Emperor +1"));
    assert_eq!(g.perm(e).loyalty, Some(emp_start(&g) + 1));
    assert_eq!(g.perm(mine).plus, 0);
}

#[test]
fn emperor_minus_one_makes_a_white_samurai_with_vigilance() {
    let mut g = tbl();
    let e = perm(&mut g, P0, EMP);
    use_perm(&mut g, e, vec![ans_word("which ability", "-1")]);
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a token");
    assert!(g.perm(t).vig, "vigilance");
    assert_eq!((g.perm(t).pow, g.perm(t).tgh), (2, 2));
    assert!(g.perm(t).ttypes.iter().any(|ty| *ty == "samurai"));
    assert!(sim_core::engine::values::colors_of(&g, t).has('W'));
}

#[test]
fn emperor_minus_two_offers_only_tapped_creatures_of_any_controller() {
    let mut g = tbl();
    let e = perm(&mut g, P0, EMP);
    let own_tapped = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(own_tapped).tapped = true;
    let _untapped = perm(&mut g, P1, "Llanowar Elves");
    let tg = ans_target(&g, "Wandering Emperor", Target::Perm(own_tapped));
    let h = use_perm(&mut g, e, vec![ans_word("which ability", "-2"), tg]);
    let q = h.asked().into_iter().find(|r| r.prompt.contains("which tapped creature")).expect("asks the target");
    let creatures = q.choices.iter().filter(|c| c.contains("Llanowar")).count();
    assert_eq!(creatures, 1, "only the tapped one: {:?}", q.choices);
    assert!(!g.perm(own_tapped).on_bf, "your own tapped creature is a legal target");
    assert_eq!(g.player(P0).life, 42);
}

// ------------------------------------------------------------------ Eiganjo, Seat of the Empire
const EIG: &str = "Eiganjo, Seat of the Empire";

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a Pithing Needle of P1's that names `name`
fn needle(g: &mut Game, name: &str) {
    let n = perm(g, P1, "Pithing Needle");
    let c = card(g, name);
    g.perm_mut(n).data.set(DataKey::Imprint, Val::Card(c));
}

/// P1 attacks with a tapped Llanowar Elves
fn eig_combat(g: &mut Game) -> PermId {
    g.active = Some(P1);
    g.step = Step::Combat;
    let atk = perm(g, P1, "Llanowar Elves");
    g.perm_mut(atk).tapped = true;
    g.in_combat = vec![atk];
    atk
}

fn channel() -> Vec<Value> {
    vec![json!({"do": "cast", "card": 0})]
}

#[test]
fn eiganjo_channel_deals_4_damage_to_an_attacker_at_instant_speed() {
    let mut g = tbl();
    let atk = eig_combat(&mut g);
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let tg = ans_target(&g, "Eiganjo", Target::Perm(atk));
    let h = respond_play(&mut g, channel(), vec![tg]);
    assert!(!g.perm(atk).on_bf, "{:?}", h.told());
    assert!(in_gy(&g, P0, EIG), "discarded as a cost");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn eiganjo_costs_one_less_for_each_legendary_creature_you_control() {
    let mut g = tbl();
    perm(&mut g, P0, "Tymna the Weaver");
    perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    let atk = eig_combat(&mut g);
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1)]); // {2}{W} less two: {W}
    respond_play(&mut g, channel(), vec![]);
    assert!(!g.perm(atk).on_bf && in_gy(&g, P0, EIG));
}

#[test]
fn eiganjo_without_the_mana_is_refused() {
    let mut g = tbl();
    let atk = eig_combat(&mut g);
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1), ('C', 1)]); // {2}{W}: one short
    let h = respond_play(&mut g, channel(), vec![]);
    assert!(g.perm(atk).on_bf && !in_gy(&g, P0, EIG));
    assert!(told_last(&h).contains("channel Eiganjo"), "{:?}", h.told());
}

#[test]
fn eiganjo_targets_only_attacking_and_blocking_creatures_of_any_controller() {
    let mut g = tbl();
    let atk = eig_combat(&mut g);
    let blocker = perm(&mut g, P0, "Mother of Runes");
    g.blocking = vec![blocker];
    perm(&mut g, P1, "Birds of Paradise"); // in neither
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let h = respond_play(&mut g, channel(), vec![]);
    let q = h.asked().into_iter().find(|r| r.prompt.contains("Eiganjo")).expect("asks the target");
    assert_eq!(q.choices.iter().filter(|c| c.starts_with("Llanowar") || c.starts_with("Mother")).count(), 2);
    assert!(!q.choices.iter().any(|c| c.contains("Birds of Paradise")), "{:?}", q.choices);
    assert!(!g.perm(atk).on_bf, "the first choice (the attacker) took the damage");
}

#[test]
fn eiganjo_needs_an_attacking_or_blocking_creature() {
    let mut g = tbl();
    perm(&mut g, P1, "Llanowar Elves");
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let h = respond_play(&mut g, channel(), vec![]);
    assert!(!in_gy(&g, P0, EIG));
    assert!(told_last(&h).contains("attacking or blocking"), "{:?}", h.told());
}

#[test]
fn pithing_needle_naming_eiganjo_stops_the_channel() {
    let mut g = tbl();
    let atk = eig_combat(&mut g);
    needle(&mut g, EIG);
    hand(&mut g, P0, &[EIG]);
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let h = respond_play(&mut g, channel(), vec![]);
    assert!(g.perm(atk).on_bf && !in_gy(&g, P0, EIG));
    assert!(told_last(&h).contains("Pithing Needle"), "{:?}", h.told());
}
