//! model-gaps-a4, item 6: a person chooses what they proliferate (rule 701.27: any number of permanents and/or players
//! that have a counter; each gets one more of each kind it already has). One shared path (`human::proliferate`, reached
//! from `common::proliferate` and `mine::proliferate_all`) serves every source; the AI's proliferate is unchanged.

mod play_common;
use play_common::*;
use sim_core::control::Answer;
use sim_core::engine::zones;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::impls::common::proliferate;
use sim_core::state::Step;
use std::sync::{Arc, Mutex};

fn tbl() -> Game {
    let mut g = table(&["karn", "kefka", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

type Shown = Arc<Mutex<Vec<Vec<String>>>>;

/// a person at P0 who, asked to proliferate, picks the entries containing these words, one per question, in order,
/// and is done when none is left; any other choice: the "proliferate" one if there is one, else the first; the lists
/// shown by each proliferate question are recorded
fn person(g: &mut Game, mut words: Vec<String>) -> Shown {
    let shown: Shown = Arc::new(Mutex::new(vec![]));
    let s2 = shown.clone();
    let h = ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.starts_with("Proliferate:") {
            s2.lock().unwrap().push(r.choices.clone());
            if words.is_empty() {
                return Answer::Decline;
            }
            let w = words.remove(0);
            return Answer::Pick(
                r.choices.iter().position(|c| c.contains(&w)).unwrap_or_else(|| panic!("{w:?} in {:?}", r.choices)),
            );
        }
        if r.choices.is_empty() {
            return Answer::Decline;
        }
        Answer::Pick(r.choices.iter().position(|c| c.contains("proliferate")).unwrap_or(0))
    });
    g.humans.add(P0, Arc::new(h));
    shown
}

struct Board {
    a: PermId,
    b: PermId,
    w: PermId,
    foe: PermId,
    foe_plus: PermId,
}

/// P0: two creatures with +1/+1 counters and a planeswalker; P1: poison, a creature with -1/-1 counters and one with +1/+1
/// counters; P2: poison
fn board(g: &mut Game) -> Board {
    let a = perm(g, P0, "Llanowar Elves");
    g.perm_mut(a).plus = 2;
    let b = perm(g, P0, "Birds of Paradise");
    g.perm_mut(b).plus = 1;
    let w = perm(g, P0, "Karn Liberated");
    g.perm_mut(w).loyalty = Some(4);
    let foe = perm(g, P1, "Serra Angel");
    g.perm_mut(foe).plus = -2;
    let foe_plus = perm(g, P1, "Vampire Nighthawk");
    g.perm_mut(foe_plus).plus = 1;
    g.player_mut(P1).poison = 3;
    g.player_mut(P2).poison = 2;
    Board { a, b, w, foe, foe_plus }
}

fn p1_poison(g: &Game) -> String {
    format!("{} (player)", g.player(P1).name)
}
fn p2_poison(g: &Game) -> String {
    format!("{} (player)", g.player(P2).name)
}

/// the person's ability whose label contains `word` on permanent m (Marvin's included)
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let abil = abilities_of(g, P0, m).unwrap();
    let (_, kind) =
        abil.into_iter().find(|(l, _)| l.contains(word)).unwrap_or_else(|| panic!("no ability with {word:?}"));
    match kind {
        AbilityKind::Extra(f) => f(g, P0, m).unwrap(),
        _ => panic!("not an extra ability"),
    }
}

#[test]
fn a_person_proliferates_a_chosen_subset_and_the_rest_is_left_alone() {
    let mut g = tbl();
    let bd = board(&mut g);
    let n1 = p1_poison(&g);
    let shown = person(&mut g, vec!["Llanowar Elves".into(), n1]);
    proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(bd.a).plus, 3, "chosen");
    assert_eq!(g.perm(bd.b).plus, 1, "not chosen");
    assert_eq!(g.perm(bd.w).loyalty, Some(4), "not chosen");
    assert_eq!(g.player(P1).poison, 4, "an opponent's poison, chosen");
    assert_eq!(g.player(P2).poison, 2, "an opponent's poison, not chosen");
    assert_eq!(g.perm(bd.foe).plus, -2, "-1/-1 counters, not chosen");
    assert_eq!(g.perm(bd.foe_plus).plus, 1, "an opponent's +1/+1 counter, not chosen");
    assert_eq!(shown.lock().unwrap()[0].len(), 7 + 1, "everything with a counter was listed, and the way out");
}

#[test]
fn the_list_has_everything_with_a_counter_and_the_ais_picks_first() {
    let mut g = tbl();
    let _bd = board(&mut g);
    let shown = person(&mut g, vec![]);
    proliferate(&mut g, P0, 1).unwrap();
    let l = shown.lock().unwrap()[0].clone();
    // Elves, Birds, Karn, Serra Angel (-1/-1), Vampire Nighthawk (+1/+1), two players with poison
    assert_eq!(l.len(), 7 + 1, "{l:?}");
    let marked: Vec<bool> = l.iter().map(|c| c.contains("what the AI would pick")).collect();
    let n = marked.iter().filter(|&&x| x).count();
    assert_eq!(n, 6, "{l:?}"); // Elves, Birds, Karn, Serra Angel, two poisoned opponents
    assert!(marked[..n].iter().all(|&x| x) && marked[n..].iter().all(|&x| !x), "marked ones first: {l:?}");
    assert!(l[n].contains("Vampire Nighthawk"), "an opponent's +1/+1 counter is not what the AI would pick: {l:?}");
    // nothing was chosen: nothing changed
}

#[test]
fn a_person_may_choose_more_of_the_opponents_minus_counters_and_their_plus_counters() {
    let mut g = tbl();
    let bd = board(&mut g);
    person(&mut g, vec!["Serra Angel".into(), "Vampire Nighthawk".into(), "Karn Liberated".into()]);
    proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(bd.foe).plus, -3);
    assert_eq!(g.perm(bd.foe_plus).plus, 2, "a person may choose an opponent's +1/+1 counters too");
    assert_eq!(g.perm(bd.w).loyalty, Some(5));
    assert_eq!((g.perm(bd.a).plus, g.perm(bd.b).plus), (2, 1));
    assert_eq!((g.player(P1).poison, g.player(P2).poison), (3, 2), "poison not chosen");
}

#[test]
fn a_person_may_choose_their_own_minus_counters_and_poison() {
    let mut g = tbl();
    let bd = board(&mut g);
    g.perm_mut(bd.b).plus = -1; // their own -1/-1 counter
    g.player_mut(P0).poison = 1;
    let me = format!("{} (player)", g.player(P0).name);
    person(&mut g, vec![me, "Birds of Paradise".into()]);
    proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).poison, 2);
    assert_eq!(g.perm(bd.b).plus, -2);
}

#[test]
fn a_person_who_chooses_nothing_proliferates_nothing() {
    let mut g = tbl();
    let bd = board(&mut g);
    person(&mut g, vec![]);
    proliferate(&mut g, P0, 1).unwrap();
    assert_eq!((g.perm(bd.a).plus, g.perm(bd.b).plus, g.perm(bd.w).loyalty), (2, 1, Some(4)));
    assert_eq!((g.player(P1).poison, g.player(P2).poison, g.perm(bd.foe).plus), (3, 2, -2));
}

#[test]
fn tekuthal_makes_a_person_proliferate_twice_and_choose_each_time() {
    let mut g = tbl();
    let bd = board(&mut g);
    perm(&mut g, P0, "Tekuthal, Inquiry Dominus");
    let shown = person(&mut g, vec!["Llanowar Elves".into(), "Birds of Paradise".into()]);
    proliferate(&mut g, P0, 1).unwrap();
    // each proliferate asks until the person is done: Elves in the first, Birds in the second; then one "done" each
    assert!(shown.lock().unwrap().len() >= 3);
    assert_eq!(g.perm(bd.a).plus, 3);
    assert_eq!(g.perm(bd.b).plus, 2);
}

#[test]
fn the_engines_other_counters_are_there_to_choose_too() {
    let mut g = tbl();
    let _bd = board(&mut g);
    // charge counters on a Vivid land, a land's kept counters, a player's energy
    let vivid = lands(&mut g, P0, "Vivid Creek", 1, false)[0];
    g.perm_mut(vivid).data.set(sim_core::state::DataKey::Charge, sim_core::state::Val::Int(1));
    g.player_mut(P0).energy = 2;
    let me = format!("{} (player)", g.player(P0).name);
    let shown = person(&mut g, vec!["Vivid Creek".into(), me]);
    proliferate(&mut g, P0, 1).unwrap();
    assert!(shown.lock().unwrap()[0].iter().any(|c| c.contains("Vivid Creek") && c.contains("1 charge")));
    assert_eq!(g.perm(vivid).data.int(sim_core::state::DataKey::Charge), 2);
    assert_eq!(g.player(P0).energy, 3);
}

// ------------------------------------------------------------------ one test per source a person controls
fn assert_subset(g: &Game, bd: &Board, who: &str) {
    assert_eq!(g.perm(bd.a).plus, 3, "{who}: chosen");
    assert_eq!(g.perm(bd.b).plus, 1, "{who}: not chosen");
    assert_eq!(g.perm(bd.w).loyalty, Some(4), "{who}: not chosen");
    assert_eq!(g.player(P1).poison, 3, "{who}: an opponent's poison, not chosen");
    assert_eq!(g.player(P2).poison, 3, "{who}: an opponent's poison, chosen");
    assert_eq!(g.perm(bd.foe).plus, -2, "{who}: not chosen");
}

fn choose_a_and_p2(g: &mut Game) -> Shown {
    let n2 = p2_poison(g);
    person(g, vec!["Llanowar Elves".into(), n2])
}

#[test]
fn source_karns_bastion() {
    let mut g = tbl();
    let bd = board(&mut g);
    let l = lands(&mut g, P0, "Karn's Bastion", 1, false)[0];
    pool(&mut g, P0, &[('C', 4)]);
    choose_a_and_p2(&mut g);
    let alts = sim_core::human::alt::land_alts(&mut g, P0, l).unwrap();
    let (_, o) = alts.into_iter().find(|(t, _)| t.contains("proliferate")).expect("Karn's Bastion's ability");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.perm(l).tapped);
    assert_subset(&g, &bd, "Karn's Bastion");
}

#[test]
fn source_vraska_betrayals_sting_0() {
    let mut g = tbl();
    let bd = board(&mut g);
    let v = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    g.perm_mut(v).loyalty = Some(6);
    choose_a_and_p2(&mut g);
    let why = use_ability(&mut g, v, "proliferate");
    assert!(why.is_none(), "{why:?}");
    assert_subset(&g, &bd, "Vraska");
}

#[test]
fn source_cankerbloom_and_marvin_using_it() {
    for marvin in [false, true] {
        let mut g = tbl();
        let bd = board(&mut g);
        let c = perm(&mut g, P0, "Cankerbloom");
        let user = if marvin { perm(&mut g, P0, "Marvin, Murderous Mimic") } else { c };
        pool(&mut g, P0, &[('C', 3), ('G', 3)]);
        choose_a_and_p2(&mut g);
        let why = use_ability(&mut g, user, if marvin { "Marvin copies Cankerbloom" } else { "proliferate" });
        assert!(why.is_none(), "{why:?}");
        assert_subset(&g, &bd, if marvin { "Marvin using Cankerbloom" } else { "Cankerbloom" });
        assert!(!g.perm(user).on_bf, "sacrificed");
    }
}

#[test]
fn source_high_perfect_morcant_and_marvin_using_it() {
    for marvin in [false, true] {
        let mut g = tbl();
        let c = perm(&mut g, P0, "High Perfect Morcant");
        let user = if marvin { perm(&mut g, P0, "Marvin, Murderous Mimic") } else { c };
        for _ in 0..4 {
            let e = perm(&mut g, P0, "Llanowar Elves");
            g.perm_mut(e).plus = 0;
        }
        let _bd = board(&mut g);
        choose_a_and_p2(&mut g);
        let why = use_ability(&mut g, user, if marvin { "Marvin copies High Perfect Morcant" } else { "proliferate" });
        assert!(why.is_none(), "{why:?}");
        assert_eq!(g.player(P2).poison, 3, "marvin={marvin}: chosen");
        assert_eq!(g.player(P1).poison, 3, "marvin={marvin}: not chosen");
    }
}

#[test]
fn source_a_compiled_effect_evolution_sage() {
    let mut g = tbl();
    let bd = board(&mut g);
    perm(&mut g, P0, "Evolution Sage");
    choose_a_and_p2(&mut g);
    zones::land_ramp(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(bd.b).plus, 1, "not chosen");
    assert_eq!(g.perm(bd.w).loyalty, Some(4), "not chosen");
    assert_eq!(g.player(P2).poison, 3, "chosen");
    assert_eq!(g.player(P1).poison, 3, "not chosen");
    assert!(g.perm(bd.a).plus >= 3, "chosen (and maybe the landfall counter)");
}

#[test]
fn source_mines_proliferate_all_for_a_person() {
    let mut g = tbl();
    let bd = board(&mut g);
    choose_a_and_p2(&mut g);
    sim_core::cardcode::proliferate_all(&mut g, P0).unwrap();
    assert_subset(&g, &bd, "proliferate_all");
}

// ------------------------------------------------------------------ the AI is unchanged
#[test]
fn the_ais_proliferate_still_takes_everything_that_helps_it_and_asks_nobody() {
    let mut g = tbl();
    let bd = board(&mut g);
    proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(bd.a).plus, 3);
    assert_eq!(g.perm(bd.b).plus, 2);
    assert_eq!(g.perm(bd.w).loyalty, Some(5));
    assert_eq!((g.player(P1).poison, g.player(P2).poison), (4, 3));
    assert_eq!(g.perm(bd.foe).plus, -3);
    assert_eq!(g.perm(bd.foe_plus).plus, 1, "never its opponents' +1/+1 counters");
}
