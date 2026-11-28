//! The Trostani deck's counter cards (impls/trostani_counters.rs, engine/repl.rs): Branching Evolution, Conclave Mentor,
//! Kalonian Hydra, Wildwood Scourge, The Doctor (EMH), Gavony Township, Cathars' Crusade, and the doublers they combine
//! with (Doubling Season, Innkeeper's Talent level 3, Anointed Procession, Hangarback Walker, Walking Ballista, Elspeth,
//! Sun's Champion). Each test names the Oracle text it checks and takes its numbers from it (CR 614.5, 616.1: the
//! controller of the counters' permanent orders the replacement effects, each applies once to one placement; the best
//! order for +1/+1 counters puts every "plus one" before every doubling).

use sim_core::ai::act;
use sim_core::engine::{hooks, life, repl, stack, turn, zones};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::trostani_counters::doctor_resolve;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["trostani", "seph"])
}

/// a 2/2 token of p's, not summoning sick
fn bear(g: &mut Game, p: PlayerId) -> PermId {
    token(g, p, 2)
}

/// +1/+1 counters p's effect puts on m, through every counter effect; how many went on
fn put(g: &mut Game, p: PlayerId, m: PermId, n: i32) -> i32 {
    let k = repl::put_counters(g, p, m, n);
    stack::flush_triggers(g).unwrap(); // the end of the effect: what it triggered goes on the stack
    k
}

fn plus(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

/// Innkeeper's Talent at level 3: "If you would put one or more counters on a permanent or player, put twice that many ..."
fn innkeeper3(g: &mut Game, p: PlayerId) -> PermId {
    let t = perm(g, p, "Innkeeper's Talent");
    g.perm_mut(t).data.set(DataKey::Level, Val::Int(3));
    t
}

/// Wildwood Scourge cast for X ({X}{G}): it enters with X +1/+1 counters
fn scourge(g: &mut Game, p: PlayerId, x: i32) -> PermId {
    let c = take(g, p, "Wildwood Scourge");
    zones::enter(g, p, c, zones::Enter { x, was_cast: true, sick: false, ..Default::default() }).unwrap()
}

/// Elspeth, Sun's Champion's +1 (card code: the AI's option "+1", then its action)
fn plus_one(g: &mut Game, e: PermId) {
    let f = g.registry.get(g.perm(e).cd.unwrap()).and_then(|i| i.options).expect("Elspeth has options");
    let o = f(g, e, P0, Some(false)).unwrap().into_iter().find(|o| o.label.contains("+1")).expect("the +1");
    assert!(act::perform(g, P0, o.act.as_ref().unwrap()).unwrap());
}

fn attack(g: &mut Game, src: PermId) {
    let call = Call::Attack { p: P0, atk: vec![src], d: P1 };
    hooks::run_hook(g, src, Event::Attack, &call).unwrap();
}

// ======================================================== Branching Evolution
// "If one or more +1/+1 counters would be put on a creature you control, twice that many +1/+1 counters are put on that
// creature instead."

#[test]
fn branching_doubles_a_counter_on_your_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 2);
    assert_eq!(plus(&g, a), 2);
    assert_eq!(put(&mut g, P0, a, 3), 6, "twice as many, whatever the number");
    assert_eq!(plus(&g, a), 8);
}

#[test]
fn branching_leaves_an_opponents_creature_alone() {
    // "a creature you control": a +1/+1 counter you put on an opponent's creature is one counter
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let theirs = bear(&mut g, P1);
    assert_eq!(put(&mut g, P0, theirs, 1), 1);
    assert_eq!(plus(&g, theirs), 1);
}

#[test]
fn branching_does_not_double_loyalty() {
    // loyalty counters are not +1/+1 counters: Elspeth, Sun's Champion (loyalty 4) enters with 4
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let e = perm(&mut g, P0, "Elspeth, Sun's Champion");
    assert_eq!(g.perm(e).loyalty, Some(4));
}

#[test]
fn branching_does_not_double_counters_on_a_noncreature() {
    // Heliod, Sun-Crowned without devotion is not a creature: a +1/+1 counter on it is one counter
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    g.perm_mut(h).data.set(DataKey::NotCreature, Val::Bool(true));
    assert!(!g.is_creature(h));
    assert_eq!(put(&mut g, P0, h, 1), 1);
}

#[test]
fn branching_does_not_change_minus_counters() {
    // "+1/+1 counters": -1/-1 counters on your creature are as many as they were
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let a = perm(&mut g, P0, "Serra Angel");
    assert_eq!(put(&mut g, P0, a, -1), -1);
    assert_eq!(plus(&g, a), -1);
}

#[test]
fn two_branchings_double_twice() {
    // each copy is its own replacement effect: 1 -> 2 -> 4
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P0, "Branching Evolution");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 4);
}

#[test]
fn branching_with_doubling_season_makes_four() {
    // two doublings: 1 x 2 x 2 = 4
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P0, "Doubling Season");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 4);
}

#[test]
fn branching_season_and_innkeeper_level_3_make_eight() {
    // three doublings: 1 x 2 x 2 x 2 = 8
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P0, "Doubling Season");
    innkeeper3(&mut g, P0);
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 8);
}

#[test]
fn branching_season_innkeeper_and_mentor_make_sixteen() {
    // the controller orders the effects (616.1): Conclave Mentor's "plus one" first, then three doublings: (1 + 1) x 8
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P0, "Doubling Season");
    innkeeper3(&mut g, P0);
    perm(&mut g, P0, "Conclave Mentor");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 16);
}

// ======================================================== Conclave Mentor
// "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on that
// creature instead. When this creature dies, you gain life equal to its power."

#[test]
fn mentor_adds_one() {
    let mut g = tbl();
    perm(&mut g, P0, "Conclave Mentor");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 2);
    assert_eq!(put(&mut g, P0, a, 3), 4);
}

#[test]
fn mentor_before_doubling_season_makes_four_not_three() {
    // (1 + 1) x 2 = 4: the controller picks the order that gives the most
    let mut g = tbl();
    perm(&mut g, P0, "Conclave Mentor");
    perm(&mut g, P0, "Doubling Season");
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 4);
}

#[test]
fn mentor_does_not_touch_an_opponents_creature_or_minus_counters() {
    let mut g = tbl();
    perm(&mut g, P0, "Conclave Mentor");
    let theirs = bear(&mut g, P1);
    assert_eq!(put(&mut g, P0, theirs, 1), 1);
    let mine = perm(&mut g, P0, "Serra Angel");
    assert_eq!(put(&mut g, P0, mine, -1), -1);
}

#[test]
fn mentor_dying_gains_its_power() {
    // a 2/2 with three +1/+1 counters has power 5: "you gain life equal to its power"
    let mut g = tbl();
    let m = perm(&mut g, P0, "Conclave Mentor");
    g.perm_mut(m).plus = 3;
    g.player_mut(P0).life = 20;
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(!g.perm(m).on_bf);
    assert_eq!(g.player(P0).life, 25);
}

#[test]
fn another_creature_dying_does_not_gain_life() {
    let mut g = tbl();
    perm(&mut g, P0, "Conclave Mentor");
    let a = bear(&mut g, P0);
    g.player_mut(P0).life = 20;
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(g.player(P0).life, 20);
}

// ======================================================== Kalonian Hydra
// "This creature enters with four +1/+1 counters on it. Whenever this creature attacks, double the number of +1/+1 counters
// on each creature you control."

#[test]
fn kalonian_enters_with_four_and_eight_under_doubling_season() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    assert_eq!(plus(&g, k), 4);
    perm(&mut g, P0, "Doubling Season");
    let k2 = perm(&mut g, P0, "Kalonian Hydra");
    assert_eq!(plus(&g, k2), 8, "placed counters: Doubling Season doubles them");
}

#[test]
fn kalonian_attack_doubles_a_creature_with_three_counters() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    attack(&mut g, k);
    assert_eq!(plus(&g, a), 6);
    assert_eq!(plus(&g, k), 8, "the Hydra's own four are doubled too");
}

#[test]
fn kalonian_attack_with_doubling_season_makes_nine() {
    // to double 3 it puts 3 more; Doubling Season makes that 6: 3 + 6 = 9
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    perm(&mut g, P0, "Doubling Season");
    attack(&mut g, k);
    assert_eq!(plus(&g, a), 9);
}

#[test]
fn kalonian_attack_skips_creatures_without_counters_even_with_mentor() {
    // "double zero" is nothing put: Conclave Mentor's "one or more" does not apply
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let none = bear(&mut g, P0);
    perm(&mut g, P0, "Conclave Mentor");
    attack(&mut g, k);
    assert_eq!(plus(&g, none), 0);
}

#[test]
fn kalonian_attack_leaves_an_opponents_creature_alone() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let theirs = bear(&mut g, P1);
    g.perm_mut(theirs).plus = 3;
    attack(&mut g, k);
    assert_eq!(plus(&g, theirs), 3);
}

#[test]
fn kalonian_attack_under_every_doubler() {
    // Doubling Season, Branching Evolution and Innkeeper level 3: each of the 3 put counters is placed as 8: 3 + 24 = 27;
    // with Conclave Mentor the placement is (3 + 1) x 8 = 32: 35
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Branching Evolution");
    innkeeper3(&mut g, P0);
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    attack(&mut g, k);
    assert_eq!(plus(&g, a), 27);
    perm(&mut g, P0, "Conclave Mentor");
    let b = bear(&mut g, P0);
    g.perm_mut(b).plus = 3;
    attack(&mut g, k);
    assert_eq!(plus(&g, b), 35);
}

#[test]
fn kalonian_attack_trigger_survives_its_source_leaving() {
    // CR 113.7a: a triggered ability exists without its source. The trigger is put on hold (a spell is resolving), an
    // opponent removes the Hydra, and the trigger still resolves: a creature with 3 counters ends at 6
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    g.resolving += 1;
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![k], d: P1 }).unwrap();
    zones::exile_perm(&mut g, k).unwrap();
    assert!(!g.perm(k).on_bf);
    g.resolving -= 1;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(plus(&g, a), 6);
}

#[test]
fn another_creature_attacking_does_not_double() {
    let mut g = tbl();
    perm(&mut g, P0, "Kalonian Hydra");
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    attack(&mut g, a);
    assert_eq!(plus(&g, a), 3);
}

// ======================================================== Wildwood Scourge
// "This creature enters with X +1/+1 counters on it. Whenever one or more +1/+1 counters are put on another non-Hydra
// creature you control, put a +1/+1 counter on this creature."

#[test]
fn scourge_enters_with_x_counters_and_twice_that_under_doubling_season() {
    let mut g = tbl();
    let s = scourge(&mut g, P0, 3);
    assert_eq!(plus(&g, s), 3);
    perm(&mut g, P0, "Doubling Season");
    let s2 = scourge(&mut g, P0, 3);
    assert_eq!(plus(&g, s2), 6);
}

#[test]
fn scourge_grows_when_counters_go_on_another_creature() {
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = bear(&mut g, P0);
    put(&mut g, P0, a, 1);
    assert_eq!(plus(&g, s), 3);
}

#[test]
fn scourge_grows_once_for_one_placement_however_many_counters() {
    // "one or more": five counters on a creature in one event are one trigger
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = bear(&mut g, P0);
    put(&mut g, P0, a, 5);
    assert_eq!(plus(&g, s), 3);
    put(&mut g, P0, a, 1);
    assert_eq!(plus(&g, s), 4, "a second event, a second trigger");
}

#[test]
fn scourges_own_counter_goes_through_the_doublers() {
    // Doubling Season: its X = 2 enters as 4; the +1/+1 counter on the Soldier is 2, and the Scourge's one becomes 2
    let mut g = tbl();
    perm(&mut g, P0, "Doubling Season");
    let s = scourge(&mut g, P0, 2);
    assert_eq!(plus(&g, s), 4);
    let a = bear(&mut g, P0);
    assert_eq!(put(&mut g, P0, a, 1), 2);
    assert_eq!(plus(&g, s), 6);
    // Branching and Mentor apply to a Scourge too (a creature you control): Mentor's one more, then Season: (1 + 1) x 2
    perm(&mut g, P0, "Conclave Mentor");
    let before = plus(&g, s);
    put(&mut g, P0, a, 1);
    assert_eq!(plus(&g, s), before + 4);
}

#[test]
fn scourge_ignores_hydras_itself_and_opponents_creatures() {
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let theirs = bear(&mut g, P1);
    let base = plus(&g, s);
    put(&mut g, P0, k, 1);
    assert_eq!(plus(&g, s), base, "counters on a Hydra");
    put(&mut g, P0, s, 1);
    assert_eq!(plus(&g, s), base + 1, "its own counters: no trigger");
    put(&mut g, P0, theirs, 1);
    assert_eq!(plus(&g, s), base + 1, "an opponent's creature");
    // -1/-1 counters are not +1/+1 counters
    let a = bear(&mut g, P0);
    put(&mut g, P0, a, -1);
    assert_eq!(plus(&g, s), base + 1);
}

#[test]
fn two_scourges_both_grow() {
    let mut g = tbl();
    let s1 = scourge(&mut g, P0, 2);
    let s2 = scourge(&mut g, P0, 3);
    let a = bear(&mut g, P0);
    put(&mut g, P0, a, 1);
    assert_eq!((plus(&g, s1), plus(&g, s2)), (3, 4));
}

#[test]
fn scourge_grows_when_a_creature_enters_with_counters() {
    // CR 122.6: a creature that enters with counters had them put on it. Hangarback Walker enters with X = 2
    let mut g = tbl();
    let s = scourge(&mut g, P0, 1);
    let c = take(&mut g, P0, "Hangarback Walker");
    let h =
        zones::enter(&mut g, P0, c, zones::Enter { x: 2, was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert_eq!(plus(&g, h), 2);
    assert_eq!(plus(&g, s), 2);
}

// ======================================================== Cathars' Crusade
// "Whenever a creature you control enters, put a +1/+1 counter on each creature you control."

#[test]
fn crusade_puts_a_counter_on_each_creature_through_the_counter_effects() {
    // Doubling Season: one creature entering is one trigger, a placement of 1 on each creature: 2 each
    let mut g = tbl();
    let a = bear(&mut g, P0);
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Cathars' Crusade");
    let b = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(plus(&g, a), 2);
    assert_eq!(plus(&g, b), 2);
}

#[test]
fn crusade_without_doublers_is_one_counter_each() {
    let mut g = tbl();
    perm(&mut g, P0, "Cathars' Crusade");
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Fyndhorn Elves");
    assert_eq!((plus(&g, a), plus(&g, b)), (2, 1));
}

#[test]
fn crusade_with_mentor_and_three_tokens_at_once_gives_six() {
    // three creatures enter at once: three triggers, each a separate placement on each creature: (1 + 1) x 3 = 6
    let mut g = tbl();
    let e = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Conclave Mentor");
    perm(&mut g, P0, "Cathars' Crusade");
    let before = plus(&g, e);
    zones::make_tokens(&mut g, P0, zones::Tokens::new(3, 1)).unwrap();
    assert_eq!(plus(&g, e), before + 6);
}

#[test]
fn crusade_counters_make_the_scourge_grow_once_for_each_creature() {
    // one trigger of Crusade puts a counter on each creature: a placement on each of the three Elves (a Scourge trigger
    // each), and one on the Scourge itself (a Hydra: no trigger)
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    for n in ["Llanowar Elves", "Fyndhorn Elves"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, "Cathars' Crusade");
    let before = plus(&g, s);
    perm(&mut g, P0, "Elvish Mystic");
    assert_eq!(plus(&g, s) - before, 3 + 1);
}

// ======================================================== The Doctor, EMH
// "When The Doctor enters, put a +1/+1 counter on target creature. Then put a +1/+1 counter on each creature you control
// with a counter on it."

#[test]
fn doctor_counters_the_target_and_each_creature_with_a_counter() {
    let mut g = tbl();
    let angel = perm(&mut g, P0, "Serra Angel");
    let b = bear(&mut g, P0);
    g.perm_mut(b).plus = 1;
    let theirs = bear(&mut g, P1);
    let d = perm(&mut g, P0, "The Doctor, EMH");
    assert_eq!(plus(&g, angel), 2, "the target with no counter: its own, then the second sentence's");
    assert_eq!(plus(&g, b), 2, "had one: one more");
    assert_eq!(plus(&g, d), 0, "no counter when the second sentence looks");
    assert_eq!(plus(&g, theirs), 0);
}

#[test]
fn doctor_under_doubling_season_makes_four_and_three() {
    // each placement is 2: the target 2 + 2 = 4; the creature with one counter 1 + 2 = 3
    let mut g = tbl();
    let angel = perm(&mut g, P0, "Serra Angel");
    let b = bear(&mut g, P0);
    g.perm_mut(b).plus = 1;
    let theirs = bear(&mut g, P1);
    perm(&mut g, P0, "Doubling Season");
    let d = perm(&mut g, P0, "The Doctor, EMH");
    assert_eq!(plus(&g, angel), 4);
    assert_eq!(plus(&g, b), 3);
    assert_eq!((plus(&g, d), plus(&g, theirs)), (0, 0));
}

#[test]
fn the_ai_aims_the_doctor_at_its_own_creature_with_no_counter() {
    // a creature without counters is the better target than a bigger one that has them (it is counted twice), and an
    // opponent's creature is not aimed at while the Doctor's controller has creatures
    let mut g = tbl();
    let none = perm(&mut g, P0, "Serra Angel");
    let big = token(&mut g, P0, 6);
    g.perm_mut(big).plus = 2;
    let theirs = token(&mut g, P1, 9);
    let d = perm(&mut g, P0, "The Doctor, EMH");
    assert_eq!(plus(&g, none), 2, "the target: its own counter, then the second sentence's");
    assert_eq!(plus(&g, big), 3, "one counter from the second sentence only");
    assert_eq!((plus(&g, theirs), plus(&g, d)), (0, 0));
}

#[test]
fn doctor_does_nothing_when_its_target_is_gone() {
    // 608.2b: the only target is illegal as the ability resolves: neither sentence does anything
    let mut g = tbl();
    let a = bear(&mut g, P0);
    let b = bear(&mut g, P0);
    g.perm_mut(b).plus = 1;
    g.skip_etb = true;
    let d = perm(&mut g, P0, "The Doctor, EMH");
    g.skip_etb = false;
    zones::die(&mut g, a, "destroy").unwrap();
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!((plus(&g, b), plus(&g, d)), (1, 0));
}

#[test]
fn doctor_aimed_at_an_opponents_creature_counts_only_creatures_you_control() {
    let mut g = tbl();
    let b = bear(&mut g, P0);
    g.perm_mut(b).plus = 1;
    let theirs = bear(&mut g, P1);
    g.skip_etb = true;
    let d = perm(&mut g, P0, "The Doctor, EMH");
    g.skip_etb = false;
    doctor_resolve(&mut g, d, P0, theirs).unwrap();
    assert_eq!(plus(&g, theirs), 1, "the target, once: it is not a creature you control");
    assert_eq!(plus(&g, b), 2);
}

// ======================================================== Gavony Township
// "{2}{G}{W}, {T}: Put a +1/+1 counter on each creature you control."

/// Gavony and `others` more lands that make {G} or {W}
fn gavony_with(g: &mut Game, others: usize) -> PermId {
    let l = lands(g, P0, "Gavony Township", 1, false)[0];
    lands(g, P0, "Forest", others.div_ceil(2), false);
    lands(g, P0, "Plains", others / 2, false);
    l
}

fn gavony_opts(g: &mut Game, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    sim_core::impls::lands::land_options(g, P0, post)
        .unwrap()
        .into_iter()
        .filter(|o| o.label.contains("Gavony"))
        .collect()
}

#[test]
fn gavony_puts_two_on_each_of_three_creatures_under_doubling_season() {
    let mut g = tbl();
    perm(&mut g, P0, "Doubling Season");
    let l = gavony_with(&mut g, 4);
    let cs = [bear(&mut g, P0), bear(&mut g, P0), bear(&mut g, P0)];
    g.active = Some(P1); // an opponent's end step
    let o = gavony_opts(&mut g, None);
    assert_eq!(o.len(), 1, "the AI uses it at an opponent's end step");
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    for c in cs {
        assert_eq!(plus(&g, c), 2);
    }
    assert!(g.perm(l).tapped, "{{T}} is part of the cost");
    assert!(g.player(P0).lands.iter().all(|&x| g.perm(x).tapped), "{{2}}{{G}}{{W}} came from the other four");
}

#[test]
fn gavony_cannot_pay_its_own_mana_into_its_cost() {
    // the cost taps it, so its {C} can't pay for {2}{G}{W}: three other lands are not enough
    let mut g = tbl();
    gavony_with(&mut g, 3);
    bear(&mut g, P0);
    bear(&mut g, P0);
    g.active = Some(P1);
    assert!(gavony_opts(&mut g, None).is_empty());
}

#[test]
fn gavony_is_used_with_two_creatures_never_with_fewer() {
    let mut g = tbl();
    gavony_with(&mut g, 4);
    g.active = Some(P1);
    assert!(gavony_opts(&mut g, None).is_empty(), "no creature");
    bear(&mut g, P0);
    assert!(gavony_opts(&mut g, None).is_empty(), "one creature");
    bear(&mut g, P0);
    assert_eq!(gavony_opts(&mut g, None).len(), 1, "two creatures");
    g.active = Some(P0);
    assert!(gavony_opts(&mut g, None).is_empty(), "not at your own end step");
}

#[test]
fn gavony_is_worth_more_under_a_doubler() {
    let mut g = tbl();
    gavony_with(&mut g, 4);
    bear(&mut g, P0);
    bear(&mut g, P0);
    g.active = Some(P1);
    let plain = gavony_opts(&mut g, None)[0].utility;
    perm(&mut g, P0, "Doubling Season");
    let doubled = gavony_opts(&mut g, None)[0].utility;
    assert!(doubled > plain, "{doubled} against {plain}");
}

// ======================================================== the doublers with other cards
#[test]
fn hangarback_walker_enters_with_x_doubled_by_branching() {
    // "enters with X +1/+1 counters": a placed counter. X = 2 under Branching Evolution: 4
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    let c = take(&mut g, P0, "Hangarback Walker");
    let h =
        zones::enter(&mut g, P0, c, zones::Enter { x: 2, was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert_eq!(plus(&g, h), 4);
}

#[test]
fn walking_ballista_enters_with_x_doubled_by_branching_and_season() {
    // {X}{X}: X = 2 (four mana paid); Branching and Doubling Season: 2 x 2 x 2 = 8
    let mut g = tbl();
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P0, "Doubling Season");
    g.last_x = 4;
    let c = take(&mut g, P0, "Walking Ballista");
    let b = zones::enter(&mut g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert_eq!(plus(&g, b), 8);
}

#[test]
fn elspeth_plus_one_makes_six_soldiers_with_anointed_procession_and_twelve_with_doubling_season() {
    // "+1: Create three 1/1 white Soldier creature tokens." Anointed Procession: twice as many (6); Doubling Season too (12)
    let mut g = tbl();
    perm(&mut g, P0, "Anointed Procession");
    let e = perm(&mut g, P0, "Elspeth, Sun's Champion");
    let tokens = |g: &Game| g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    plus_one(&mut g, e);
    assert_eq!(tokens(&g), 6);
    let mut g = tbl();
    perm(&mut g, P0, "Anointed Procession");
    perm(&mut g, P0, "Doubling Season");
    let e = perm(&mut g, P0, "Elspeth, Sun's Champion");
    assert_eq!(g.perm(e).loyalty, Some(8), "Doubling Season: it enters with twice the loyalty");
    plus_one(&mut g, e);
    assert_eq!(tokens(&g), 12);
    assert_eq!(g.perm(e).loyalty, Some(9), "the +1 is a cost: Doubling Season does not double it");
}

// ======================================================== Wildwood Scourge on the stack
// "Whenever one or more +1/+1 counters are put on another non-Hydra creature you control, put a +1/+1 counter on this
// creature." A triggered ability (CR 603.3): it goes on the stack the next time a player would get priority, after the spell,
// ability or trigger that put the counters has finished. Each path the deck uses ends with the counter landed and nothing
// left in `trig_queue`.

fn landed(g: &Game) {
    assert!(g.trig_queue.is_empty(), "a Scourge trigger is still queued: {:?}", g.trig_queue);
}

#[test]
fn gavony_activation_ends_with_the_scourges_counters_landed() {
    // each of 3 creatures gets one counter (the Scourge among them: a Hydra, no trigger); the two Bears are non-Hydra:
    // two triggers. 2 + 1 + 2 = 5
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = bear(&mut g, P0);
    let b = bear(&mut g, P0);
    gavony_with(&mut g, 4);
    g.active = Some(P1);
    let o = gavony_opts(&mut g, None);
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!((plus(&g, a), plus(&g, b)), (1, 1));
    assert_eq!(plus(&g, s), 5);
    landed(&g);
}

#[test]
fn heliod_life_gain_counter_ends_with_the_scourges_counter_landed() {
    // "Whenever you gain life, put a +1/+1 counter on target creature or enchantment you control": if it goes on the
    // Bear (a non-Hydra) the Scourge triggers once
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = perm(&mut g, P0, "Serra Angel"); // (the AI aims the counter at its best creature)
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    life::gain(&mut g, P0, 3).unwrap();
    landed(&g);
    assert_eq!(plus(&g, a), 1, "Heliod's counter went on the Bear (scourge {}, heliod {})", plus(&g, s), plus(&g, h));
    assert_eq!(plus(&g, s), 3);
}

#[test]
fn crusade_on_a_creature_land_entry_ends_with_the_scourges_counters_landed() {
    // the land-entry path (a token copy of a land that is a creature) has its own flush: Crusade puts one counter on each
    // creature (the Scourge, then the land creature itself, a non-Hydra: one trigger): 2 + 1 + 1 = 4
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    perm(&mut g, P0, "Cathars' Crusade");
    let exc = sim_core::state::CopyExc {
        add_types: sim_core::cards::Types::CREATURE,
        pt: Some((3, 3)),
        ..Default::default()
    };
    let forest = card(&g, "Forest");
    let l = turn::enter_token_land_copy(&mut g, P0, forest, exc).unwrap().unwrap();
    assert_eq!(plus(&g, l), 1);
    assert_eq!(plus(&g, s), 4);
    landed(&g);
}

#[test]
fn crusade_on_a_spell_entry_and_on_tokens_ends_with_the_scourges_counters_landed() {
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    perm(&mut g, P0, "Cathars' Crusade");
    // an Elf enters: one trigger; counters on the Scourge (own) and the Elf (a Scourge trigger): 2 + 1 + 1
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(plus(&g, s), 4);
    landed(&g);
    // two tokens at once: two Crusade triggers, each a counter on each of the 4 creatures: 2 own placements on the
    // Scourge and a Scourge trigger for each placement on a non-Hydra (3 non-Hydras x 2 = 6): 4 + 2 + 6 = 12
    zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(plus(&g, s), 12);
    landed(&g);
}

#[test]
fn the_doctors_enters_trigger_ends_with_the_scourges_counters_landed() {
    // the Angel (no counters) is the target: two placements on it (two Scourge triggers); the Scourge has counters, so the
    // second sentence puts one on it (a Hydra: no trigger): 2 + 1 + 2 = 5
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let angel = perm(&mut g, P0, "Serra Angel");
    let d = perm(&mut g, P0, "The Doctor, EMH");
    assert_eq!((plus(&g, angel), plus(&g, d)), (2, 0));
    assert_eq!(plus(&g, s), 5);
    landed(&g);
}

#[test]
fn kalonians_attack_trigger_ends_with_the_scourges_counter_landed() {
    // the Bear's 3 counters are doubled (one placement: one Scourge trigger); the Scourge doubles its own 2: 2 + 2 + 1 = 5
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let k = perm(&mut g, P0, "Kalonian Hydra");
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 3;
    attack(&mut g, k);
    assert_eq!(plus(&g, a), 6);
    assert_eq!(plus(&g, s), 5);
    landed(&g);
}

#[test]
fn emiels_counter_ends_with_the_scourges_counter_landed() {
    // Emiel: whenever another creature enters under your control, you may pay {G/W}: a +1/+1 counter on it. A token and a
    // spell's creature each enter with one more counter, and the Scourge grows for each
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    perm(&mut g, P0, "Emiel the Blessed");
    lands(&mut g, P0, "Plains", 2, false);
    let toks = zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 1)).unwrap();
    assert_eq!(plus(&g, toks[0]), 1);
    assert_eq!(plus(&g, s), 3);
    landed(&g);
    let e = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(plus(&g, e), 1);
    assert_eq!(plus(&g, s), 4);
    landed(&g);
}

#[test]
fn a_scourge_that_left_before_its_trigger_resolves_gets_nothing() {
    // "the same permanent": the trigger is on its way, the Scourge is exiled; a new Scourge (another object) gets no counter
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = bear(&mut g, P0);
    g.resolving += 1;
    repl::put_counters(&mut g, P0, a, 1);
    zones::exile_perm(&mut g, s).unwrap();
    let s2 = scourge(&mut g, P0, 1);
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(plus(&g, s2), 1);
    assert!(!g.perm(s).on_bf);
    landed(&g);
}

#[test]
fn trostanis_life_trigger_still_resolves_after_the_scourges_triggers() {
    // Trostani's trigger is late: it resolves after the other triggers of the same batch even when it was queued first.
    // The Scourge enters (Trostani's trigger queued: she gains life equal to its toughness), then a counter goes on a Bear in
    // the same ability (the Scourge's trigger, queued after hers): the Scourge's counter lands first, so she reads 3, not 2
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    let a = bear(&mut g, P0);
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    let s = scourge(&mut g, P0, 2);
    repl::put_counters(&mut g, P0, a, 1);
    assert_eq!(g.trig_queue.len(), 2, "Trostani's trigger and the Scourge's");
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(plus(&g, s), 3);
    assert_eq!(g.player(P0).life, 23);
    landed(&g);
}

#[test]
fn hangarback_walkers_activation_ends_with_the_scourges_counter_landed() {
    // Hangarback Walker: "{1}, {T}: Put a +1/+1 counter on Hangarback Walker." A compiled top-level activation: the counter
    // makes the Scourge trigger, and its counter lands before control returns (not at some later window). 2, +1 for the
    // Walker entering with a counter, +1 for the activation = 4
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let c = take(&mut g, P0, "Hangarback Walker");
    let w =
        zones::enter(&mut g, P0, c, zones::Enter { x: 1, was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert_eq!((plus(&g, w), plus(&g, s)), (1, 3));
    landed(&g);
    lands(&mut g, P0, "Forest", 1, false);
    g.active = Some(P0);
    let o = sim_core::dsl::ability_options(&mut g, P0, true)
        .unwrap()
        .into_iter()
        .find(|o| o.label.contains("Hangarback Walker"))
        .expect("the Walker's {1},{T} is offered");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(plus(&g, w), 2);
    assert_eq!(plus(&g, s), 4, "the Scourge's counter landed at the end of the activation");
    landed(&g);
}

#[test]
fn proliferate_ends_with_the_scourges_counter_landed() {
    // Proliferate: one +1/+1 counter more on each permanent with one. The Scourge (a Hydra) and the Bear (a non-Hydra, one
    // Scourge trigger): 2 + 1 + 1 = 4; the Bear 1 + 1 = 2
    let mut g = tbl();
    let s = scourge(&mut g, P0, 2);
    let a = bear(&mut g, P0);
    g.perm_mut(a).plus = 1;
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(plus(&g, a), 2);
    assert_eq!(plus(&g, s), 4);
    landed(&g);
}

// ======================================================== Heliod + Walking Ballista readiness
// Walking Ballista: "Remove a +1/+1 counter from Walking Ballista: It deals 1 damage to any target." Heliod gives it lifelink
// ({1}{W}), the life gained puts a counter back. With one counter the cost leaves it 0/0: it dies before the ping resolves
// and before Heliod's trigger can target it. With two, the Ballista stays and the loop runs.

#[test]
fn heliod_with_one_counter_on_the_ballista_is_not_a_ready_line() {
    let mut g = tbl();
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    g.last_x = 2; // {X}{X} with X = 1
    let c = take(&mut g, P0, "Walking Ballista");
    let b = zones::enter(&mut g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    let i = sim_core::impls::combos::combo_index("Heliod, Sun-Crowned + Walking Ballista").unwrap();
    assert_eq!(plus(&g, b), 1);
    assert!(!sim_core::impls::combos::ready(&g, P0, i), "one counter is spent as the cost: the Ballista dies");
    g.perm_mut(b).plus = 2;
    assert!(sim_core::impls::combos::ready(&g, P0, i), "two counters: one stays after paying");
}

// ======================================================== Innkeeper's Talent: the combat trigger
// "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control." (CR 113.7a: a triggered
// ability exists apart from its source.)

#[test]
fn innkeepers_combat_trigger_survives_its_source_leaving() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Innkeeper's Talent");
    let a = bear(&mut g, P0);
    g.active = Some(P0);
    g.resolving += 1;
    hooks::fire_trigger(&mut g, Event::EachCombat, Call::Player { p: P0 }).unwrap();
    zones::exile_perm(&mut g, t).unwrap();
    assert!(!g.perm(t).on_bf);
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(plus(&g, a), 1, "the trigger was already on its way: it resolves without the Talent");
}

#[test]
fn innkeepers_combat_trigger_is_for_its_controllers_turn_only() {
    let mut g = tbl();
    perm(&mut g, P0, "Innkeeper's Talent");
    let a = bear(&mut g, P0);
    g.active = Some(P1);
    hooks::fire_trigger(&mut g, Event::EachCombat, Call::Player { p: P1 }).unwrap();
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(plus(&g, a), 0);
}
