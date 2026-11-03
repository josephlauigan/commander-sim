//! Practice mode: the Mikaeus, the Unhallowed loops of the Sheoldred deck by hand and as table shortcuts (Triskelion's
//! pings at itself, Mikaeus's undying counters, Yawgmoth's sacrifice with a target of the person's choosing), and
//! Niv-Mizzet's standing ping target (Kefka).

mod play_common;
use play_common::*;
use sim_core::engine::zones;
use sim_core::human::human::{AbilityKind, abilities_of};

const MIKAEUS: &str = "Mikaeus, the Unhallowed";
const YAWG: &str = "Yawgmoth, Thran Physician";

/// a person who declines whatever is asked (a "cancel" at the end of a list), answering `f` first
fn person(g: &mut Game, f: impl FnMut(&Request) -> Option<Answer> + Send + 'static) -> std::sync::Arc<ScriptHuman> {
    let mut f = f;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| f(r).unwrap_or(Answer::Decline)));
    g.humans.add(P0, h.clone());
    h
}

fn pick_text(r: &Request, word: &str) -> Option<Answer> {
    r.choices.iter().position(|c| c.contains(word)).map(Answer::Pick)
}

/// run the ability of m whose label contains `word`; the reason it can't be done, if any
fn use_ability(g: &mut Game, p: PlayerId, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, p, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, p, m).unwrap()
}

fn labels(g: &mut Game, p: PlayerId, m: PermId) -> Vec<String> {
    abilities_of(g, p, m).unwrap().into_iter().map(|a| a.0).collect()
}

fn find_new(g: &Game, p: PlayerId, name: &str) -> PermId {
    perms_named(g, p, name).into_iter().find(|&m| g.perm(m).on_bf).unwrap_or_else(|| panic!("no {name} on the table"))
}

// ------------------------------------------------------------------ Mikaeus + Triskelion
#[test]
fn undying_brings_triskelion_back_with_all_its_counters() {
    let mut g = table(&["sheoldred", "cloud"]);
    person(&mut g, |_| None);
    perm(&mut g, P0, MIKAEUS);
    let t = perm(&mut g, P0, "Triskelion");
    assert_eq!(g.perm(t).plus, 3, "it enters with three +1/+1 counters");
    g.perm_mut(t).plus = 0; // its counters are gone
    zones::die(&mut g, t, "destroy").unwrap();
    let t2 = find_new(&g, P0, "Triskelion");
    assert_ne!(t, t2, "it came back");
    assert_eq!(g.perm(t2).plus, 4, "three from entering and the undying one");
    // and it can go again: undying only asks that it has no +1/+1 counter at that moment
    g.perm_mut(t2).plus = 0;
    zones::die(&mut g, t2, "destroy").unwrap();
    assert_eq!(g.perm(find_new(&g, P0, "Triskelion")).plus, 4);
}

#[test]
fn triskelion_pinging_itself_dies_by_hand_and_comes_back() {
    let mut g = table(&["sheoldred", "cloud"]);
    perm(&mut g, P0, MIKAEUS);
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).plus = 4; // back from undying
    // each ping: itself, itself, the opponent, itself
    let aim =
        std::sync::Arc::new(std::sync::Mutex::new(vec!["Triskelion", "Triskelion", "OPP", "Triskelion"].into_iter()));
    let a2 = aim.clone();
    let h = person(&mut g, move |r| {
        if r.prompt.contains("1 damage to which target") {
            let w = a2.lock().unwrap().next().unwrap();
            if w == "OPP" {
                return r.choices.iter().position(|c| c.contains("(player,") && !c.contains("(you)")).map(Answer::Pick);
            }
            return pick_text(r, w);
        }
        None
    });
    let life = g.player(P1).life;
    for i in 0..4 {
        assert!(g.perm(t).on_bf, "ping {i}: still there");
        let why = use_ability(&mut g, P0, t, "remove a +1/+1 counter");
        assert_eq!(why, None);
        if i == 1 {
            assert!(g.perm(t).on_bf, "two pings at itself: 2 damage marked, toughness 4");
        }
    }
    assert!(!g.perm(t).on_bf, "marked damage 3 against toughness 2: it died");
    assert_eq!(g.player(P1).life, life - 1, "the third ping went to the opponent");
    let back = find_new(&g, P0, "Triskelion");
    assert_eq!(g.perm(back).plus, 4, "undying brought it back with 4 counters (no +1/+1 counter was left on it)");
    assert!(h.asked().iter().filter(|r| r.prompt.contains("1 damage")).count() == 4);
}

#[test]
fn start_the_triskelion_loop_kills_the_table() {
    for key in ["sheoldred", "seph"] {
        let mut g = table(&[key, "cloud", "kefka"]);
        let h = person(&mut g, |r| pick_text(r, "every opponent"));
        perm(&mut g, P0, MIKAEUS);
        let t = perm(&mut g, P0, "Triskelion");
        g.perm_mut(t).plus = 4;
        let l = labels(&mut g, P0, t);
        assert!(l.iter().any(|x| x.starts_with("start the loop: Mikaeus + Triskelion")), "{key}: {l:?}");
        assert_eq!(use_ability(&mut g, P0, t, "start the loop"), None);
        assert!(g.over && g.winner == Some(P0), "{key}: both opponents dead");
        let ask = h.asked();
        assert!(
            ask.iter().any(|r| r.prompt.contains("Triskelion deals as much damage")),
            "{key}: you pick what it hits"
        );
        assert!(ask.len() <= 3, "{key}: one question, not one per ping: {}", ask.len());
    }
}

#[test]
fn the_triskelion_loop_hits_one_opponent_or_a_creature() {
    let mut g = table(&["sheoldred", "cloud", "kefka"]);
    let target = perm(&mut g, P1, "Sol Ring"); // not a creature: only players and creatures are listed
    let _ = target;
    let bear = token(&mut g, P2, 2);
    let mut step = 0;
    let h = person(&mut g, move |r| {
        step += 1;
        match step {
            1 => pick_text(r, "destroy"),
            2 => pick_text(r, "their life to 0 ("),
            _ => None,
        }
    });
    perm(&mut g, P0, MIKAEUS);
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).plus = 4;
    assert_eq!(use_ability(&mut g, P0, t, "start the loop"), None);
    assert!(!g.perm(bear).on_bf, "the creature is destroyed");
    assert!(!g.over || g.winner == Some(P0));
    let _ = h;
}

// ------------------------------------------------------------------ Yawgmoth
#[test]
fn yawgmoth_you_pick_the_fodder_and_the_target_and_undying_works_again() {
    let mut g = table(&["sheoldred", "cloud"]);
    perm(&mut g, P0, MIKAEUS);
    let yawg = perm(&mut g, P0, YAWG);
    let seer = perm(&mut g, P0, "Viscera Seer");
    let frog = perm(&mut g, P0, "Burrog Banemaker");
    // (sacrifice, target) of each use
    let plan = std::sync::Arc::new(std::sync::Mutex::new(
        vec![("Viscera Seer", None), ("Burrog Banemaker", Some("Viscera Seer"))].into_iter(),
    ));
    let cur: std::sync::Arc<std::sync::Mutex<Option<(&str, Option<&str>)>>> = Default::default();
    let (p2, c2) = (plan.clone(), cur.clone());
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Yawgmoth: sacrifice which") {
            let step = p2.lock().unwrap().next().unwrap();
            *c2.lock().unwrap() = Some(step);
            return pick_text(r, step.0);
        }
        if r.prompt.starts_with("Yawgmoth: put a -1/-1 counter") {
            return match c2.lock().unwrap().unwrap().1 {
                Some(w) => pick_text(r, w),
                None => Some(Answer::Decline),
            };
        }
        None
    });
    let (life, hand) = (g.player(P0).life, g.player(P0).hand.len());
    assert_eq!(use_ability(&mut g, P0, yawg, "sacrifice another creature"), None);
    // the Seer had no counter: it died, undying brought it back with one
    let seer2 = find_new(&g, P0, "Viscera Seer");
    assert_ne!(seer2, seer);
    assert_eq!(g.perm(seer2).plus, 1);
    assert_eq!(use_ability(&mut g, P0, yawg, "sacrifice another creature"), None);
    // the frog died and returned with a counter; the -1/-1 counter took the Seer's +1/+1 counter off
    let frog2 = find_new(&g, P0, "Burrog Banemaker");
    assert_ne!(frog2, frog);
    assert_eq!(g.perm(frog2).plus, 1);
    assert_eq!(g.perm(seer2).plus, 0, "+1/+1 and -1/-1 cancel: it can die and return again");
    assert_eq!(g.player(P0).life, life - 2, "1 life each");
    assert_eq!(g.player(P0).hand.len(), hand + 2, "a card each");
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Yawgmoth: put a -1/-1 counter")));
}

#[test]
fn yawgmoth_loop_drains_the_table_with_a_payoff() {
    let mut g = table(&["sheoldred", "cloud", "kefka"]);
    perm(&mut g, P0, MIKAEUS);
    let yawg = perm(&mut g, P0, YAWG);
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Burrog Banemaker");
    perm(&mut g, P0, "Zulaport Cutthroat"); // a Human: no undying, so not one of the two
    for q in [P1, P2] {
        g.player_mut(q).life = 12;
    }
    let l = labels(&mut g, P0, yawg);
    assert!(l.iter().any(|x| x == "start the loop: Mikaeus + Yawgmoth + two undying creatures"), "{l:?}");
    let h = person(&mut g, |r| pick_text(r, "as many times"));
    let (lib, life) = (g.player(P0).library.len(), g.player(P0).life);
    assert_eq!(use_ability(&mut g, P0, yawg, "start the loop"), None);
    assert!(g.over && g.winner == Some(P0), "each trip drains every opponent for 1: twelve trips");
    assert_eq!(g.player(P0).library.len(), lib - 12, "and draws a card each");
    assert_eq!(g.player(P0).life, life, "1 life paid, 1 gained from Zulaport, every trip");
    assert!(h.asked().len() <= 2, "one question: {:?}", h.asked().iter().map(|r| r.prompt.clone()).collect::<Vec<_>>());
}

#[test]
fn yawgmoth_loop_stops_before_the_library_runs_out() {
    let mut g = table(&["sheoldred", "cloud"]);
    perm(&mut g, P0, MIKAEUS);
    let yawg = perm(&mut g, P0, YAWG);
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Burrog Banemaker");
    perm(&mut g, P0, "Zulaport Cutthroat");
    g.player_mut(P1).life = 1000;
    g.player_mut(P0).library.truncate(8);
    person(&mut g, |r| pick_text(r, "as many times"));
    assert_eq!(use_ability(&mut g, P0, yawg, "start the loop"), None);
    assert!(!g.over);
    assert_eq!(g.player(P0).library.len(), 1, "it stops with one card left");
}

// ------------------------------------------------------------------ Kefka: Niv-Mizzet's pings
#[test]
fn niv_mizzet_standing_target_asks_once() {
    let mut g = table(&["kefka", "cloud", "sheoldred"]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("deal 1 damage to which target") {
            return r.choices.iter().position(|c| c.starts_with("every")).map(|i| Answer::Pick(i)); // the first opponent
        }
        None
    });
    let niv = perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    let life = g.player(P1).life;
    zones::draw(&mut g, P0, 4, false).unwrap();
    let asked = h.asked().iter().filter(|r| r.prompt.contains("deal 1 damage")).count();
    assert_eq!(asked, 1, "one question for four draws");
    assert_eq!(g.player(P1).life, life - 4, "all four pings went to the standing target");
    // the ability changes it, or goes back to asking
    assert!(labels(&mut g, P0, niv).iter().any(|l| l.contains("choose where all of its pings go")));
}

// ------------------------------------------------------------------ Kefka: Psychosis Crawler + the flipped Kefka
fn flipped_kefka(g: &mut Game, p: PlayerId) -> PermId {
    use sim_core::state::{DataKey, Val};
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.player_mut(p).cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    let x = g.perm_mut(m);
    x.data.set(DataKey::Ruin, Val::Bool(true));
    (x.pow, x.tgh, x.fly) = (5, 7, true);
    m
}

/// Psychosis Crawler + the flipped Kefka by hand: one draw starts it; with a 45-card library the table (40 life each)
/// is dead before the library is, with no question asked on the way
#[test]
fn crawler_and_flipped_kefka_loop_with_a_45_card_library_asks_nothing() {
    let mut g = table(&["kefka", "cloud", "sheoldred"]);
    let h = person(&mut g, |_| None);
    flipped_kefka(&mut g, P0);
    hand(&mut g, P0, &["Island"]); // the Crawler is */* (cards in hand): with an empty hand it dies
    perm(&mut g, P0, "Psychosis Crawler");
    g.player_mut(P0).library.truncate(45);
    g.active = Some(P0);
    zones::draw(&mut g, P0, 1, false).unwrap();
    let asked: Vec<String> = h.asked().iter().map(|r| format!("{}: {}", r.kind, r.prompt)).collect();
    assert!(g.over && g.winner == Some(P0), "the table is dead (library {} left, lives {} {}, hand {}, asked {asked:?})", g.player(P0).library.len(), g.player(P1).life, g.player(P2).life, g.player(P0).hand.len());
    assert!(asked.len() <= 2, "{} questions: {asked:?}", asked.len());
}
