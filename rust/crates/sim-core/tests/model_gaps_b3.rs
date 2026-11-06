//! model-gaps-b3: Soul-Guide Lantern's enter effect is a trigger (Camera copies it, Rings does not); Staff of Domination's
//! untap is covered by the run's one Rings answer; a person activates Clue, Food and Blood tokens; Lathliss's pump
//! counts only the activations the mana can pay ({1}{R} each).

mod play_common;
use play_common::*;
use sim_core::hooks::Action;
use sim_core::state::Step;
use std::sync::Arc;

const RINGS: &str = "Rings of Brighthearth";

fn asked(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| r.prompt.clone()).collect()
}

fn idx(g: &Game, p: PlayerId, m: PermId) -> usize {
    g.player(p).perms.iter().position(|&x| x == m).unwrap()
}

fn apply(g: &mut Game, p: PlayerId, v: Value) -> Option<String> {
    sim_core::human::human::apply(g, p, &v).unwrap()
}

fn token(g: &mut Game, p: PlayerId, kind: &str) -> Option<String> {
    apply(g, p, json!({"do": "use", "token": kind}))
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn to_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

/// a seat that answers by rule: (a part of the prompt, a part of the choice to pick); a priority is a pass unless
/// `on_priority` returns an action for it; anything else the first choice
fn smart(
    g: &mut Game,
    p: PlayerId,
    rules: Vec<(&'static str, &'static str)>,
    mut on_priority: impl FnMut(&Request) -> Option<Value> + Send + 'static,
) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            return Answer::Action(on_priority(r).unwrap_or(json!({"do": "pass"})));
        }
        for (pk, ck) in &rules {
            if r.prompt.contains(pk)
                && let Some(i) = r.choices.iter().position(|c| c.contains(ck))
            {
                return Answer::Pick(i);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(p, h.clone());
    h
}

fn cast_lantern(g: &mut Game) -> Option<String> {
    let c = hand(g, P0, &["Soul-Guide Lantern"])[0];
    let i = g.player(P0).hand.iter().position(|&x| x == c).unwrap();
    apply(g, P0, json!({"do": "cast", "card": i}))
}

// ================================================================== 1. Soul-Guide Lantern's enter effect is a trigger
#[test]
fn lanterns_enter_trigger_exiles_the_card_the_person_picks_and_rings_offers_nothing() {
    let mut g = table(&["karn", "cloud"]);
    perm(&mut g, P0, RINGS);
    to_gy(&mut g, P0, &["Island"]);
    to_gy(&mut g, P1, &["Grave Titan", "Sol Ring"]);
    pool(&mut g, P0, &[('C', 5)]);
    let h = smart(&mut g, P0, vec![("exile which card", "Grave Titan")], |_| None);
    assert_eq!(cast_lantern(&mut g), None);
    assert!(in_exile(&g, P1, "Grave Titan") && in_gy(&g, P0, "Island") && in_gy(&g, P1, "Sol Ring"));
    let a = asked(&h);
    assert!(a.iter().any(|p| p.contains("exile which card")), "{a:?}");
    assert!(
        !a.iter().any(|p| p.starts_with("Rings of Brighthearth: pay")),
        "Rings copies activated abilities only: {a:?}"
    );
    assert_eq!(pool_total(&g, P0), 4, "only the {{1}} for the Lantern was spent");
}

#[test]
fn a_cameras_copy_of_lanterns_trigger_picks_a_new_target() {
    let mut g = table(&["karn", "cloud"]);
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    to_gy(&mut g, P0, &["Island"]);
    to_gy(&mut g, P1, &["Grave Titan", "Sol Ring"]);
    pool(&mut g, P0, &[('C', 5)]);
    let ci = idx(&g, P0, cam);
    let mut used = false;
    let h = smart(
        &mut g,
        P0,
        // the copy resolves first and asks first: it takes Sol Ring; the original then takes Grave Titan
        vec![("copy which ability", "Lantern"), ("exile which card", "Grave Titan")],
        move |r| {
            if !used && r.prompt.contains("Soul-Guide Lantern") {
                used = true;
                return Some(json!({"do": "use", "perm": ci}));
            }
            None
        },
    );
    assert_eq!(cast_lantern(&mut g), None);
    let a = asked(&h);
    assert!(a.iter().any(|p| p.contains("copy which ability")), "the Camera copied the trigger: {a:?}");
    assert_eq!(a.iter().filter(|p| p.contains("Soul-Guide Lantern: exile which card")).count(), 2, "{a:?}");
    assert!(in_exile(&g, P1, "Grave Titan"), "one target");
    assert_eq!(g.player(P0).gy.len() + g.player(P1).gy.len(), 1, "two cards exiled, one left");
}

// ================================================================== 2. Staff of Domination: the run's Rings answer covers the untap
fn staff_run(rings_yes: bool) -> (Game, Arc<ScriptHuman>, usize) {
    let mut g = table(&["karn", "cloud"]);
    let staff = perm(&mut g, P0, "Staff of Domination");
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 999)]);
    let (hand0, _lib) = (g.player(P0).hand.len(), g.player(P0).library.len());
    let si = idx(&g, P0, staff);
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.starts_with("Rings of Brighthearth: pay {2} to copy each draw") {
            return Answer::Pick(if rings_yes { 0 } else { 1 });
        }
        if r.prompt.ends_with("which ability?") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("draw N cards")).unwrap());
        }
        if r.prompt.starts_with("Staff of Domination: draw how many") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("draw 4 cards")).unwrap());
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    assert_eq!(apply(&mut g, P0, json!({"do": "use", "perm": si})), None);
    (g, h, hand0)
}

#[test]
fn staff_with_rings_yes_copies_the_untap_and_the_draw() {
    let (g, h, hand0) = staff_run(true);
    assert_eq!(g.player(P0).hand.len(), hand0 + 4, "four cards: two draws, each copied");
    // {5}+{2} the first draw; {1}+{2} the untap and {5}+{2} the second draw
    assert_eq!(pool_total(&g, P0), 999 - 7 - 3 - 7, "the untap was copied too");
    let a = asked(&h);
    assert_eq!(
        a.iter().filter(|p| p.starts_with("Rings of Brighthearth: pay")).count(),
        1,
        "one answer for the run: {a:?}"
    );
}

#[test]
fn staff_with_rings_no_copies_neither() {
    let (g, h, hand0) = staff_run(false);
    assert_eq!(g.player(P0).hand.len(), hand0 + 4);
    assert_eq!(pool_total(&g, P0), 999 - 5 - 3 * (1 + 5), "no copy of the untap or the draw");
    let a = asked(&h);
    assert_eq!(a.iter().filter(|p| p.starts_with("Rings of Brighthearth: pay")).count(), 1, "{a:?}");
}

// ================================================================== 3. Clue, Food and Blood tokens
#[test]
fn a_person_cracks_a_clue() {
    let mut g = table(&["karn", "cloud"]);
    g.player_mut(P0).clues = 2;
    let hand0 = g.player(P0).hand.len();
    // the refusals
    assert_eq!(token(&mut g, P0, "Clue").unwrap().contains("Can't activate a Clue token"), true);
    assert_eq!(token(&mut g, P0, "Food"), Some("You have no Food token.".into()));
    pool(&mut g, P0, &[('C', 3)]);
    let h = smart(&mut g, P0, vec![], |_| None);
    assert_eq!(token(&mut g, P0, "Clue"), None);
    assert_eq!((g.player(P0).clues, g.player(P0).hand.len(), pool_total(&g, P0)), (1, hand0 + 1, 1));
    assert!(asked(&h).is_empty());
    assert!(token(&mut g, P0, "Clue").is_some(), "{{2}} again: only {{1}} left");
}

#[test]
fn a_person_eats_a_food() {
    let mut g = table(&["karn", "cloud"]);
    g.player_mut(P0).foods = 1;
    let life = g.player(P0).life;
    assert!(token(&mut g, P0, "Food").unwrap().contains("Can't activate a Food token"));
    pool(&mut g, P0, &[('C', 2)]);
    smart(&mut g, P0, vec![], |_| None);
    assert_eq!(token(&mut g, P0, "Food"), None);
    assert_eq!((g.player(P0).foods, g.player(P0).life, pool_total(&g, P0)), (0, life + 3, 0));
}

#[test]
fn sacrificing_a_token_is_a_cost_and_fires_sacrifice_triggers() {
    // Trail of Crumbs: whenever you sacrifice a Food, you may pay {1}: look at the top two cards
    let mut g = table(&["karn", "cloud"]);
    perm(&mut g, P0, "Trail of Crumbs");
    g.player_mut(P0).foods = 1;
    pool(&mut g, P0, &[('C', 3)]);
    let h = smart(&mut g, P0, vec![], |_| None);
    assert_eq!(token(&mut g, P0, "Food"), None);
    assert_eq!(pool_total(&g, P0), 0, "{{2}} for the Food and {{1}} for the trigger: {:?}", asked(&h));
}

#[test]
fn a_person_uses_a_blood_token_and_chooses_the_discard() {
    let mut g = table(&["karn", "cloud"]);
    g.player_mut(P0).bloods = 1;
    g.player_mut(P0).hand.clear();
    pool(&mut g, P0, &[('C', 1)]);
    assert!(token(&mut g, P0, "Blood").unwrap().contains("no card in hand"));
    hand(&mut g, P0, &["Island", "Sol Ring"]);
    pool(&mut g, P0, &[('C', 1)]);
    smart(&mut g, P0, vec![("Blood token: discard which", "Sol Ring")], |_| None);
    pool(&mut g, P0, &[]);
    assert_eq!(token(&mut g, P0, "Blood"), None);
    assert!(in_gy(&g, P0, "Sol Ring"), "the person's choice");
    let hand = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>();
    assert!(hand.contains(&"Island".to_string()) && !hand.contains(&"Sol Ring".to_string()), "{hand:?}");
    assert_eq!(
        (g.player(P0).bloods, hand.len(), pool_total(&g, P0)),
        (0, 2, 1),
        "discarded one, drew one; {{1}} left of 2"
    );
    let _ = Action::Clue;
}

#[test]
fn a_blood_token_needs_mana() {
    let mut g = table(&["karn", "cloud"]);
    g.player_mut(P0).bloods = 1;
    hand(&mut g, P0, &["Island"]);
    let why = token(&mut g, P0, "Blood").unwrap();
    assert!(why.contains("Can't activate a Blood token"), "{why}");
    assert_eq!(g.player(P0).bloods, 1);
}

#[test]
fn rings_copies_a_clue_activation() {
    let mut g = table(&["karn", "cloud"]);
    perm(&mut g, P0, RINGS);
    g.player_mut(P0).clues = 1;
    pool(&mut g, P0, &[('C', 4)]);
    let hand0 = g.player(P0).hand.len();
    let h = smart(&mut g, P0, vec![], |_| None);
    assert_eq!(token(&mut g, P0, "Clue"), None);
    let a = asked(&h);
    assert!(a.iter().any(|p| p.starts_with("Rings of Brighthearth: pay {2} to copy") && p.contains("Clue")), "{a:?}");
    assert_eq!(g.player(P0).hand.len(), hand0 + 2, "the ability and its copy each draw a card");
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn the_text_client_and_the_view_know_the_tokens() {
    let mut g = table(&["karn", "cloud"]);
    g.player_mut(P0).clues = 2;
    assert_eq!(sim_core::human::tokens::count(&g, P0, "Clue"), 2);
    assert_eq!(sim_core::human::tokens::count(&g, P0, "Food"), 0);
}

// ================================================================== 4. Lathliss: the pump counts what can be paid
fn urd() -> Game {
    let mut g = table(&[
        "ur-dragon-five-color-dragons",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ]);
    g.player_mut(P0).hand.clear();
    g
}

fn lathliss_option(g: &mut Game, l: PermId) -> Option<sim_core::hooks::Opt> {
    g.active = Some(P0);
    g.step = Step::Main1;
    let f = (*g.registry.get(card(g, "Lathliss, Dragon Queen")).unwrap()).options.unwrap();
    f(g, l, P0, Some(false)).unwrap().pop()
}

#[test]
fn the_ai_pump_counts_the_red_it_has() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    perm(&mut g, P0, "Goldspan Dragon");
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Island", 5, false);
    // six mana keeps two: two activations by the old count, but only one {R}
    let o = lathliss_option(&mut g, l).expect("one activation can be paid");
    assert!(o.label.contains("+1/+0"), "{}", o.label);
    let Some(Action::Ability { src, f, arg }) = o.act else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap(), "no refusal");
    assert_eq!(sim_core::engine::values::epow(&g, l), 6 + 1);
}

#[test]
fn the_ai_pump_is_not_offered_with_no_red() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    perm(&mut g, P0, "Goldspan Dragon");
    lands(&mut g, P0, "Island", 6, false);
    assert!(lathliss_option(&mut g, l).is_none());
}

#[test]
fn the_ai_pump_with_plenty_of_red_is_as_before() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    lands(&mut g, P0, "Mountain", 6, false);
    let o = lathliss_option(&mut g, l).expect("two activations");
    assert!(o.label.contains("+2/+0"), "{}", o.label);
    let Some(Action::Ability { src, f, arg }) = o.act else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(sim_core::engine::values::epow(&g, gold), 4 + 2);
}

fn lathliss_use(g: &mut Game, l: PermId) -> (Option<String>, Vec<Vec<String>>, Arc<ScriptHuman>) {
    let h = smart(g, P0, vec![("which ability?", "{1}{R}")], |_| None);
    let i = idx(g, P0, l);
    let r = apply(g, P0, json!({"do": "use", "perm": i}));
    let how_many =
        h.asked().iter().filter(|r| r.prompt.contains("how many times")).map(|r| r.choices.clone()).collect();
    (r, how_many, h)
}

#[test]
fn a_persons_pump_is_what_their_pool_pays() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    perm(&mut g, P0, "Goldspan Dragon");
    pool(&mut g, P0, &[('R', 1), ('C', 3)]); // four mana but one red: one activation
    let (r, how_many, _) = lathliss_use(&mut g, l);
    assert_eq!(r, None);
    assert_eq!(how_many.len(), 1);
    assert_eq!(how_many[0].len(), 2, "only 1 time is offered (and cancel): {:?}", how_many[0]);
    assert_eq!(sim_core::engine::values::epow(&g, l), 6 + 1);
    assert_eq!(pool_total(&g, P0), 2);
}

#[test]
fn a_persons_pump_offers_up_to_what_the_pool_pays_and_pays_each_time() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    pool(&mut g, P0, &[('R', 3), ('C', 3)]); // three activations
    let h = smart(&mut g, P0, vec![("which ability?", "{1}{R}"), ("how many times", "3 times")], |_| None);
    let i = idx(&g, P0, l);
    assert_eq!(apply(&mut g, P0, json!({"do": "use", "perm": i})), None);
    let many = h.asked().into_iter().find(|r| r.prompt.contains("how many times")).unwrap();
    assert_eq!(many.choices.len(), 4, "{:?}", many.choices);
    assert_eq!(sim_core::engine::values::epow(&g, gold), 4 + 3);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn a_person_with_no_red_in_the_pool_is_refused_with_the_reason() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    perm(&mut g, P0, "Goldspan Dragon");
    pool(&mut g, P0, &[('C', 6)]);
    let (r, how_many, _) = lathliss_use(&mut g, l);
    assert!(r.unwrap().contains("Can't activate Lathliss"), "a reason, not 'that can't be done right now'");
    assert!(how_many.is_empty());
    assert_eq!(pool_total(&g, P0), 6);
}

#[test]
fn rings_copies_each_pump_with_one_answer_for_the_run() {
    let mut g = urd();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('R', 2), ('C', 6)]);
    let h = smart(
        &mut g,
        P0,
        vec![("which ability?", "{1}{R}"), ("how many times", "2 times"), ("Rings of Brighthearth: pay", "yes")],
        |_| None,
    );
    let i = idx(&g, P0, l);
    assert_eq!(apply(&mut g, P0, json!({"do": "use", "perm": i})), None);
    let rings = asked(&h).iter().filter(|p| p.starts_with("Rings of Brighthearth: pay")).count();
    assert_eq!(rings, 1, "{:?}", asked(&h));
    assert_eq!(sim_core::engine::values::epow(&g, gold), 4 + 4, "two activations, each copied");
    assert_eq!(pool_total(&g, P0), 0);
}
