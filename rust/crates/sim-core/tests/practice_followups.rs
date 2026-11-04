//! Practice-mode follow-ups (2026-11-04, after the gap fixes #242-#244): Urza's Saga's chapter III response, Garth's
//! copies (the Regrowth and Braingeyser targets), Cavern of Souls' creature types, Giver of Runes' protection from
//! colorless, Emry's permission, Mirage Mirror's and Dauthi Voidwalker's choices, Natural Order's sacrifice and the
//! "(automatic)" notice. Each test plays the person's side and checks the answer is what happens.

mod play_common;
use play_common::*;
use sim_core::state::DataKey;
use std::sync::{Arc, Mutex};

/// a person who answers by text: for a question whose prompt contains the key, the choice containing the word (the
/// first choice when no rule fits); at a priority, the scripted actions in order, then pass
fn person(actions: Vec<Value>, rules: Vec<(&'static str, &'static str)>) -> Arc<ScriptHuman> {
    let acts = Mutex::new(actions.into_iter());
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(acts.lock().unwrap().next().unwrap_or(json!({"do": "pass"})));
        }
        for (key, word) in &rules {
            if r.prompt.contains(key)
                && let Some(i) = r.choices.iter().position(|c| c.contains(word))
            {
                return Answer::Pick(i);
            }
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }))
}

fn constructs(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).data.truthy(DataKey::Construct)).count()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Urza's Saga, chapter III
#[test]
fn urzas_saga_chapter_three_can_be_answered_with_a_construct() {
    let mut g = table(&["karn", "cloud"]);
    let l = lands(&mut g, P0, "Urza's Saga", 1, false)[0];
    g.player_mut(P0).sagas.push((l, 2)); // chapter II was last turn: this lore counter is chapter III
    pool(&mut g, P0, &[('C', 2)]);
    let i = g.player(P0).lands.iter().position(|&x| x == l).unwrap();
    // in response to chapter III: the Saga's own ability ({2}, {T}); then the search finds Sol Ring
    let h = person(vec![json!({"do": "use", "land": i})], vec![("search", "Sol Ring")]);
    g.humans.add(P0, h.clone());
    sim_core::impls::common::saga_step(&mut g, P0).unwrap();
    assert_eq!(constructs(&g, P0), 1, "the Construct made in response: {:?}", h.told());
    assert!(on_bf(&g, P0, "Sol Ring"), "chapter III found Sol Ring");
    assert!(in_gy(&g, P0, "Urza's Saga") && !g.player(P0).lands.contains(&l), "then it was sacrificed");
}

#[test]
fn urzas_saga_chapter_three_without_an_answer_still_searches_and_is_sacrificed() {
    let mut g = table(&["karn", "cloud"]);
    let l = lands(&mut g, P0, "Urza's Saga", 1, false)[0];
    g.player_mut(P0).sagas.push((l, 2));
    let h = person(vec![], vec![("search", "Sol Ring")]);
    g.humans.add(P0, h.clone());
    sim_core::impls::common::saga_step(&mut g, P0).unwrap();
    assert_eq!(constructs(&g, P0), 0, "no Construct: the person passed");
    assert!(on_bf(&g, P0, "Sol Ring") && in_gy(&g, P0, "Urza's Saga"));
}

// ------------------------------------------------------------------ Garth One-Eye's copies
/// a person at seat 0 who uses Garth (the perm at index `i`), names `card`, and answers the rest with `pick`
fn garth_person(i: usize, card: &'static str, pick: impl Fn(&Request) -> Option<usize> + Send + 'static) -> Arc<ScriptHuman> {
    let first = Mutex::new(true);
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
fn garths_braingeyser_draws_for_the_player_the_person_names() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    pool(&mut g, P0, &[('U', 2), ('C', 2)]); // {U}{U} and X = 2
    let them = sim_core::human::legal::describe_target(&g, P0, sim_core::human::Target::Player(P1));
    let h = garth_person(i, "Braingeyser", move |r| {
        if r.prompt.contains("choose X") {
            r.choices.iter().position(|c| c == "X = 2")
        } else if r.prompt.contains("which player draws") {
            r.choices.iter().position(|c| *c == them)
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    let (mine, theirs) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(h.asked().iter().any(|r| r.prompt.contains("which player draws")), "{:?}", h.told());
    assert_eq!(g.player(P1).hand.len(), theirs + 2, "the opponent drew X = 2");
    assert_eq!(g.player(P0).hand.len(), mine, "the person drew nothing");
}

#[test]
fn garths_regrowth_copy_returns_the_card_the_person_picks() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    for n in ["Grave Titan", "Sol Ring"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    pool(&mut g, P0, &[('G', 1), ('C', 1)]);
    let h = garth_person(i, "Regrowth", |r| {
        if r.prompt.contains("graveyard") { r.choices.iter().position(|c| c.contains("Sol Ring")) } else { None }
    });
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    let names: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert!(names.iter().any(|n| n == "Sol Ring"), "the person's pick came back: {names:?} / {:?}", h.told());
    assert!(in_gy(&g, P0, "Grave Titan"), "not the AI's pick (the bigger card)");
}

// ------------------------------------------------------------------ Giver of Runes: protection from colorless
#[test]
fn giver_of_runes_protection_from_colorless_stops_colorless_sources() {
    use sim_core::cards::Colors;
    use sim_core::engine::values::protected_from;
    let mut g = table(&["cloud", "seph"]);
    let giver = perm(&mut g, P0, "Giver of Runes");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let i = g.player(P0).perms.iter().position(|&m| m == giver).unwrap();
    assert!(!protected_from(&g, elf, Colors::NONE), "nothing yet");
    let h = person(
        vec![json!({"do": "use", "perm": i})],
        vec![("which other creature", "Llanowar"), ("protection from what", "colorless")],
    );
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(protected_from(&g, elf, Colors::NONE), "a colorless source (an artifact, a land): {:?}", h.told());
    assert!(!protected_from(&g, elf, Colors::from_letters("W")), "a coloured source is not stopped");
    assert!(!protected_from(&g, giver, Colors::NONE), "only the target gained it");
}

// ------------------------------------------------------------------ Cavern of Souls: any creature type
#[test]
fn a_person_can_name_any_creature_type_not_only_the_decks_twelve() {
    let mut g = table(&["urdragon", "seph"]);
    let h = person(
        vec![],
        vec![("choose a creature type", "another creature type"), ("choose any creature type", "Goblin")],
    );
    g.humans.add(P0, h.clone());
    let t = sim_core::impls::galadriel::choose_type(&g, P0, "Cavern of Souls").unwrap();
    assert_eq!(&*t, "goblin", "{:?}", h.asked().iter().map(|r| r.prompt.clone()).collect::<Vec<_>>());
    // the deck's own types are still offered first
    let first = &h.asked()[0];
    assert!(first.choices[0].contains("Dragon"), "{:?}", first.choices);
    assert!(first.choices.last().unwrap().contains("another creature type"));
}

/// the person takes `action` at the first priority, answers by text, then passes; the main phase runs
fn act_main(g: &mut Game, action: Value, rules: Vec<(&'static str, &'static str)>) -> Arc<ScriptHuman> {
    let h = person(vec![action], rules);
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(g, P0, false).unwrap();
    h
}

fn perm_index(g: &Game, m: PermId) -> usize {
    g.player(P0).perms.iter().position(|&x| x == m).unwrap()
}

// ------------------------------------------------------------------ Natural Order: back out before paying
#[test]
fn natural_orders_sacrifice_can_be_cancelled_with_nothing_paid() {
    let mut g = table(&["thrasios", "seph"]);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    hand(&mut g, P0, &["Natural Order"]);
    pool(&mut g, P0, &[('G', 4)]);
    let h = Arc::new(ScriptHuman::from_fn({
        let first = Mutex::new(true);
        move |r: &Request| {
            if &*r.kind == "priority" {
                let mut f = first.lock().unwrap();
                let a = if *f { json!({"do": "cast", "card": 0}) } else { json!({"do": "pass"}) };
                *f = false;
                return Answer::Action(a);
            }
            if r.prompt.contains("sacrifice which") {
                return Answer::Decline; // cancel
            }
            Answer::Pick(0)
        }
    }));
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(h.asked().iter().any(|r| r.prompt.contains("sacrifice which")), "{:?}", h.told());
    assert_eq!(pool_total(&g, P0), 4, "the mana is still in the pool");
    assert!(g.player(P0).perms.contains(&elf), "nothing sacrificed");
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Natural Order"), "still in hand");
}

// ------------------------------------------------------------------ Mirage Mirror and the legend rule
#[test]
fn mirage_mirror_can_copy_your_own_legend_and_you_choose_which_to_keep() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let emry = perm(&mut g, P0, "Emry, Lurker of the Loch");
    pool(&mut g, P0, &[('C', 2)]);
    let i = perm_index(&g, mm);
    let h = act_main(
        &mut g,
        json!({"do": "use", "perm": i}),
        vec![("become a copy of which", "Emry"), ("legend rule", "Emry")],
    );
    assert!(h.asked().iter().any(|r| r.prompt.contains("legend rule")), "the person keeps one: {:?}", h.told());
    let emrys = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Emry, Lurker of the Loch").count();
    assert_eq!(emrys, 1, "one Emry left");
    assert!(g.perm(emry).on_bf != g.perm(mm).on_bf, "the other went to the graveyard");
}

// ------------------------------------------------------------------ Dauthi Voidwalker: an exiled land
#[test]
fn dauthi_voidwalker_can_play_an_exiled_land_as_the_land_drop() {
    use sim_core::state::Val;
    let mut g = table(&["thrasios", "seph"]);
    let d = perm(&mut g, P0, "Dauthi Voidwalker");
    let land = take(&mut g, P1, "Forest");
    g.player_mut(P1).exile.push(land);
    g.perm_mut(d).data.set(DataKey::Void, Val::List(vec![Val::List(vec![Val::Card(land), Val::Player(P1)])]));
    let lands = g.player(P0).lands.len();
    let i = perm_index(&g, d);
    let h = act_main(&mut g, json!({"do": "use", "perm": i}), vec![("play which", "Forest")]);
    assert_eq!(g.player(P0).lands.len(), lands + 1, "the Forest was played: {:?}", h.told());
    assert_eq!(g.player(P0).lands_played, 1, "as the land drop");
    assert!(!g.player(P1).exile.cards().contains(&land));
    // with the land drop used, a land is no longer offered
    let mut g2 = table(&["thrasios", "seph"]);
    let d2 = perm(&mut g2, P0, "Dauthi Voidwalker");
    let l2 = take(&mut g2, P1, "Forest");
    g2.player_mut(P1).exile.push(l2);
    g2.perm_mut(d2).data.set(DataKey::Void, Val::List(vec![Val::List(vec![Val::Card(l2), Val::Player(P1)])]));
    g2.player_mut(P0).lands_played = 1;
    let i2 = perm_index(&g2, d2);
    let h2 = act_main(&mut g2, json!({"do": "use", "perm": i2}), vec![]);
    assert!(h2.told_kind("invalid").iter().any(|t| t.contains("can't play a land now")), "{:?}", h2.told());
    assert!(g2.perm(d2).on_bf, "not sacrificed");
}
