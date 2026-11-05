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
