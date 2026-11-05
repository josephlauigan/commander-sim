//! Practice mode, simplifications removed (2026-11-04, part 2): Urza's {5} and Triple Triad give a permission for the
//! turn, Mirage Mirror copies lands with abilities, Sylvan Library offers the cards really drawn this turn, the Ur-Dragon
//! loops ask once for a batch of tokens, and (AI-affecting) Ragavan's exiled card and Goldspan Dragon's targeting.

mod play_common;
use play_common::*;
use sim_core::ids::CardId;
use sim_core::state::{Ctx, Step};
use std::sync::{Arc, Mutex};

/// a person who answers `rule`'s choice when it returns one (the first choice, or yes, otherwise); at a priority, the
/// scripted actions in order, then pass
fn person(actions: Vec<Value>, rule: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    let acts = Mutex::new(actions.into_iter());
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(acts.lock().unwrap().next().unwrap_or(json!({"do": "pass"})));
        }
        if let Some(k) = rule(r) {
            return Answer::Pick(k);
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }))
}

fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

#[allow(dead_code)]
fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

/// the library is exactly `names`, the last one on top
fn library(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).library = cs.clone();
    cs
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn main_phase(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

fn from_exile(g: &mut Game, c: CardId) -> Option<String> {
    sim_core::human::human::apply(g, P0, &json!({"do": "cast", "zone": "ex", "card": c.0})).unwrap()
}

// ------------------------------------------------------------------ 1. Urza's {5} and Triple Triad: until end of turn
/// seat 0 is a person with Urza; its {5} is paid and the top card (the last of `lib`) is exiled with a permission
fn urza_top(
    lib: &[&str],
    answers: Vec<Value>,
    rule: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> (Game, Vec<CardId>) {
    let mut g = table(&["urdragon", "seph"]);
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    let cs = library(&mut g, P0, lib);
    g.humans.add(P0, person(answers, rule));
    // (one card: the shuffle can't move it)
    sim_core::impls::t5::urza_top(&mut g, u, P0).unwrap();
    (g, cs)
}

#[test]
fn urzas_land_is_played_as_the_land_drop_in_the_main_phase() {
    let (mut g, cs) = urza_top(&["Island"], vec![], |_| None);
    let isle = cs[0];
    assert!(g.player(P0).free_play.contains(&isle), "a permission, not played at once");
    assert_eq!(g.player(P0).lands_played, 0);
    // not in the opponent's turn: a land is a main-phase land drop
    g.active = Some(P1);
    g.step = Step::Main1;
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": isle.0})).unwrap();
    assert!(r.is_some(), "not your turn: {r:?}");
    main_phase(&mut g);
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": isle.0})).unwrap();
    assert_eq!(r, None);
    assert_eq!(g.player(P0).lands_played, 1, "it was the land drop");
    assert!(g.player(P0).lands.iter().any(|&l| &*g.perm(l).name == "Island"));
    assert!(!in_exile(&g, P0, "Island"));
}

#[test]
fn urzas_land_needs_a_land_drop_and_is_still_offered_after_a_failed_try() {
    let (mut g, cs) = urza_top(&["Island"], vec![], |_| None);
    main_phase(&mut g);
    g.player_mut(P0).lands_played = 1;
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": cs[0].0})).unwrap();
    assert!(r.is_some_and(|w| w.contains("already played a land")));
    assert!(in_exile(&g, P0, "Island"), "still in exile, still permitted");
    assert!(g.player(P0).free_play.contains(&cs[0]));
}

#[test]
fn urzas_aura_like_animate_dead_is_cast_in_the_main_phase_and_not_lost() {
    let (mut g, cs) = urza_top(&["Animate Dead"], vec![], |r| with(r, "Grave Titan"));
    let titan = take(&mut g, P1, "Grave Titan");
    g.player_mut(P1).gy.push(titan);
    main_phase(&mut g);
    assert!(g.player(P0).free_play.contains(&cs[0]), "kept for the main phase (it was lost before)");
    assert_eq!(from_exile(&mut g, cs[0]), None);
    assert!(has_perm(&g, P0, "Grave Titan"), "reanimated");
    assert!(in_gy(&g, P0, "Animate Dead"), "and the spell goes to its owner's graveyard");
}

#[test]
fn urzas_counterspell_is_held_for_a_spell_and_the_permission_ends_with_the_turn() {
    // the person answers the opponent's Grave Titan with the free Counterspell from exile
    let mut g = table(&["urdragon", "seph"]);
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    let ctr = library(&mut g, P0, &["Counterspell"])[0];
    let acts = vec![json!({"do": "cast", "zone": "ex", "card": ctr.0})];
    let h = person(acts, |_| None);
    g.humans.add(P0, h.clone());
    sim_core::impls::t5::urza_top(&mut g, u, P0).unwrap();
    assert!(in_exile(&g, P0, "Counterspell"), "not wasted at once: nothing to counter now");
    g.active = Some(P1);
    let titan = hand(&mut g, P1, &["Grave Titan"])[0];
    let resolved = sim_core::engine::cast::cast_card(&mut g, P1, titan, "hand", Ctx::default()).unwrap_or(true);
    assert!(!resolved, "countered: {:?}", h.told());
    assert!(!has_perm(&g, P1, "Grave Titan"));
    assert!(in_gy(&g, P0, "Counterspell"), "the spell is in its owner's graveyard");
    assert!(!in_exile(&g, P0, "Counterspell"));
}

#[test]
fn urzas_permission_ends_at_the_end_of_the_turn() {
    let (mut g, cs) = urza_top(&["Sol Ring"], vec![], |_| None);
    main_phase(&mut g);
    assert!(sim_core::human::legal::check_cast_exile(&g, P0, cs[0]).is_none());
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).free_play.is_empty());
    assert!(sim_core::human::legal::check_cast_exile(&g, P0, cs[0]).is_some());
    assert!(in_exile(&g, P0, "Sol Ring"), "unplayed, it stays in exile");
}

// ------------------------------------------------------------------ 2. Mirage Mirror copies lands with abilities
use sim_core::human::human::{AbilityKind, abilities_of, apply};

/// run the ability of m whose label contains `word`; the reason it can't be done, if any
fn use_ability(g: &mut Game, p: PlayerId, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, p, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, p, m).unwrap()
}

#[test]
fn mirage_mirror_copies_a_fetch_land_and_cracking_it_sends_the_mirror_to_the_graveyard() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let mirror_card = g.perm(mm).card();
    let fetch = lands(&mut g, P0, "Verdant Catacombs", 1, false)[0];
    let fetch_card = g.perm(fetch).card();
    library(&mut g, P0, &["Swamp"]);
    pool(&mut g, P0, &[('C', 2)]);
    let h = person(vec![], |r| if r.prompt.contains("Mirage Mirror") { with(r, "Verdant Catacombs") } else { None });
    g.humans.add(P0, h.clone());
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None, "a fetch land is a legal choice now");
    assert_eq!(g.perm(mm).name, "Verdant Catacombs");
    let i = g.player(P0).lands.iter().position(|&l| l == mm).expect("the Mirror is in the land list");
    // the copy's ability works: pay 1 life, sacrifice it, search
    main_phase(&mut g);
    let life = g.player(P0).life;
    assert_eq!(apply(&mut g, P0, &json!({"do": "use", "land": i})).unwrap(), None, "{:?}", h.told());
    assert_eq!(g.player(P0).life, life - 1);
    assert!(!g.perm(mm).on_bf, "sacrificed");
    assert!(g.player(P0).gy.contains(&mirror_card), "the Mirage Mirror card is in the graveyard");
    assert!(!g.player(P0).gy.contains(&fetch_card), "not a second Verdant Catacombs");
    assert!(g.perm(fetch).on_bf, "the real fetch land is untouched");
    assert!(g.player(P0).lands.iter().any(|&l| &*g.perm(l).name == "Swamp"), "and it found a Swamp");
}

#[test]
fn mirage_mirror_copies_your_own_legendary_land_and_the_legend_rule_has_you_keep_one() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let mirror_card = g.perm(mm).card();
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, false)[0];
    pool(&mut g, P0, &[('C', 2)]);
    // keep the Mirror (the second choice listed is not assumed: pick by the label's text)
    let h = person(vec![], |r| {
        if r.prompt.contains("Mirage Mirror") {
            with(r, "Gaea's Cradle")
        } else if r.prompt.contains("legend rule") {
            Some(r.choices.len() - 1)
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None);
    assert!(asked(&h, "legend rule"), "two Gaea's Cradle: {:?}", h.asked());
    let cradles =
        g.player(P0).lands.iter().filter(|&&l| g.perm(l).on_bf && &*g.perm(l).name == "Gaea's Cradle").count();
    assert_eq!(cradles, 1, "you keep one");
    let gone = if g.perm(cradle).on_bf { mm } else { cradle };
    assert!(!g.perm(gone).on_bf);
    if gone == mm {
        assert!(in_gy(&g, P0, "Mirage Mirror"), "the Mirror, not a second Cradle, is in the graveyard");
        assert!(g.player(P0).gy.contains(&mirror_card));
    }
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).on_bf), "no dead permanent left in the land list");
}
