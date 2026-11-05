//! Practice mode: triggers and abilities the AI used to decide for the person (trigger-fixes task). Each test seats a
//! person at P0, shows they are asked, and checks that their answer is what happens.

mod play_common;
use play_common::*;
use std::sync::Arc;

/// a person who answers with `pick` (None: the first choice, or yes); at a priority, pass
fn person(pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

/// the index of the first choice containing `word`
fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

fn count(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name.contains(name)).count()
}

// ------------------------------------------------------------------ Felidar Retreat
#[test]
fn felidar_retreat_person_picks_the_counters_mode() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Felidar Retreat");
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let h = person(|r| if r.prompt.contains("Felidar Retreat") { with(r, "+1/+1") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    assert!(asked(&h, "Felidar Retreat: choose a mode"), "the person was asked");
    assert_eq!(count(&g, P0, "Cat"), 0, "no Cat token: the person chose the counters");
    assert_eq!(g.perm(bear).plus, 1, "a +1/+1 counter on the creature");
    assert!(g.perm(bear).eot_kw.contains(&"vigilance"), "and vigilance until end of turn");
}

#[test]
fn felidar_retreat_person_picks_the_cat() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Felidar Retreat");
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let h = person(|r| if r.prompt.contains("Felidar Retreat") { with(r, "Cat") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    assert!(asked(&h, "Felidar Retreat: choose a mode"));
    assert_eq!(
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).pow == 2).count(),
        1,
        "the 2/2 Cat token"
    );
    assert_eq!(g.perm(bear).plus, 0);
}

// ------------------------------------------------------------------ Sun Titan
#[test]
fn sun_titan_returns_the_card_the_person_picks() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let sol = take(&mut g, P0, "Sol Ring");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, elves]);
    let h = person(|r| if r.prompt.contains("Sun Titan") { with(r, "Sol Ring") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(asked(&h, "Sun Titan: return which"), "the person was asked");
    assert!(has_perm(&g, P0, "Sol Ring"), "the picked card returned");
    assert!(!has_perm(&g, P0, "Llanowar Elves"), "the AI's pick (the creature) stays");
    assert!(g.player(P0).gy.contains(&elves));
}

#[test]
fn sun_titan_person_may_return_nothing() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.push(elves);
    let h = person(|r| if r.prompt.contains("Sun Titan") { with(r, "none") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(asked(&h, "Sun Titan: return which"));
    assert!(!has_perm(&g, P0, "Llanowar Elves"), "the person chose none");
    assert!(g.player(P0).gy.contains(&elves));
}

#[test]
fn sun_titan_ai_still_takes_the_best() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let sol = take(&mut g, P0, "Sol Ring");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, elves]);
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(has_perm(&g, P0, "Llanowar Elves"), "the AI's pick is the creature");
}

// ------------------------------------------------------------------ Mirrorworks
#[test]
fn mirrorworks_asks_the_person_before_paying() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("Mirrorworks") { Answer::Pick(1) } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Sol Ring");
    assert!(asked(&h, "Mirrorworks: pay {2}"), "the person was asked");
    assert_eq!(count(&g, P0, "Sol Ring"), 1, "they said no: no copy");
    assert!(g.player(P0).lands.iter().all(|&l| !g.perm(l).tapped), "nothing was paid");
}

#[test]
fn mirrorworks_pays_and_copies_when_the_person_says_yes() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("Mirrorworks") { Answer::Pick(0) } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Sol Ring");
    assert!(asked(&h, "Mirrorworks: pay {2}"));
    assert_eq!(count(&g, P0, "Sol Ring"), 2, "the card and its token copy");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{2}} was paid");
}

// ------------------------------------------------------------------ Garth One-Eye ("you may cast the copy")
/// a person at seat 0 who uses Garth (the perm at index `i`), names `card`, and answers the rest with `pick`
fn garth_person(
    i: usize,
    card: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = std::sync::Mutex::new(true);
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "use", "perm": i}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.contains("choose a card name") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains(card)).unwrap());
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

#[test]
fn garths_copy_is_not_cast_when_the_person_declines() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = garth_person(i, "Shivan", |r| {
        if r.prompt.contains("cast the copy") { with(r, "cast Shivan").map(|_| r.choices.len() - 1) } else { None }
    });
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "Garth One-Eye: cast the copy of Shivan Dragon"), "the person was asked");
    assert!(!has_perm(&g, P0, "Shivan Dragon"), "declined: no Shivan Dragon");
    assert_eq!(pool_total(&g, P0), 6, "nothing was paid");
    assert!(sim_core::impls::garth::chosen(&g, garth, 3), "the name stays used");
    assert!(g.perm(garth).tapped);
}

#[test]
fn garths_copy_is_cast_when_the_person_accepts() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = garth_person(i, "Shivan", |_| None);
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "Garth One-Eye: cast the copy of Shivan Dragon"));
    assert!(has_perm(&g, P0, "Shivan Dragon"), "accepted: cast");
}

#[test]
fn garths_braingeyser_x_counts_untapped_lands_and_taps_them() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    let ls = lands(&mut g, P0, "Island", 5, false); // {X}{U}{U} with X = 3: five untapped Islands, an empty pool
    let them = sim_core::human::legal::describe_target(&g, P0, sim_core::human::Target::Player(P1));
    let h = garth_person(i, "Braingeyser", move |r| {
        if r.prompt.contains("choose X") {
            assert_eq!(r.choices.last().map(String::as_str), Some("X = 3"), "X tops out at 3: {:?}", r.choices);
            r.choices.iter().position(|c| c == "X = 3")
        } else if r.prompt.contains("which player draws") {
            r.choices.iter().position(|c| *c == them)
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    let theirs = g.player(P1).hand.len();
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "choose X"), "{:?}", h.told());
    assert_eq!(g.player(P1).hand.len(), theirs + 3, "the opponent drew X = 3");
    assert!(ls.iter().all(|&l| g.perm(l).tapped), "the Islands were tapped to pay");
    assert_eq!(pool_total(&g, P0), 0);
}
