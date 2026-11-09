//! Practice-mode limits (2026-11-04, brief section P): five cards whose rules text a person could not play out in full.
//! Scarlet Witch's X, Jin-Gitaxias keeping the original's targets, Cityscape Leveler's unearth, Dauthi Voidwalker's
//! "you may play it this turn", and Mirage Mirror copying a land. The AI's play of each is unchanged.

mod play_common;
use play_common::*;
use sim_core::engine::{cast, hooks};
use sim_core::hooks::{Call, Event};
use sim_core::human::human::{AbilityKind, abilities_of, apply};
use sim_core::state::{DataKey, Step, Val};
use std::sync::{Arc, Mutex};

type Log = Arc<Mutex<Vec<String>>>;

/// seat 0 as a person: every request is recorded as "kind: prompt [choices]"; `f` answers it (None: the first choice;
/// priority is passed, attacks and blocks are none)
fn person(g: &mut Game, f: impl Fn(&Request) -> Option<Answer> + Send + 'static) -> Log {
    let log: Log = Arc::new(Mutex::new(vec![]));
    let l2 = log.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push(format!("{}: {} {:?}", r.kind, r.prompt, r.choices));
            if &*r.kind == "priority" {
                return Answer::Action(json!({"do": "pass"}));
            }
            if let Some(a) = f(r) {
                return a;
            }
            if &*r.kind == "attack" || &*r.kind == "block" { Answer::Picks(vec![]) } else { Answer::Pick(0) }
        })),
    );
    log
}

fn asked(log: &Log, word: &str) -> bool {
    log.lock().unwrap().iter().any(|l| l.contains(word))
}

fn dump(log: &Log) -> String {
    log.lock().unwrap().join("\n")
}

/// the choice containing `word`
fn text(r: &Request, word: &str) -> Answer {
    Answer::Pick(
        r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
    )
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

fn on_table(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    perms_named(g, p, name).into_iter().find(|&m| g.perm(m).on_bf)
}

fn main_phase(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

// ------------------------------------------------------------------ 1. Scarlet Witch: X is 0
// "without paying its mana cost": X is 0 by the rules (107.3b), even with mana floating, so no X question
#[test]
fn scarlet_witch_casts_an_x_spell_for_x_zero_even_with_mana_floating() {
    let mut g = table(&["kefka", "seph"]);
    let witch = perm(&mut g, P0, "Scarlet Witch, Chaotic Avenger");
    let drain = take(&mut g, P0, "Exsanguinate");
    let island = take(&mut g, P0, "Island");
    g.player_mut(P0).library.extend([island, drain]);
    pool(&mut g, P0, &[('B', 4)]);
    let before = g.player(P1).life;
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Scarlet Witch") { Some(text(r, "Exsanguinate")) } else { None });
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a: witch, d: P1, dmg: 3 }).unwrap();
    assert!(!asked(&log, "choose X"), "no X question: {}", dump(&log));
    assert_eq!(g.player(P1).life, before, "X = 0 drains nothing: {}", dump(&log));
    assert_eq!(pool_total(&g, P0), 4, "nothing paid");
}

#[test]
fn scarlet_witch_x_is_zero_with_an_empty_pool_and_no_question() {
    let mut g = table(&["kefka", "seph"]);
    let witch = perm(&mut g, P0, "Scarlet Witch, Chaotic Avenger");
    let drain = take(&mut g, P0, "Exsanguinate");
    let island = take(&mut g, P0, "Island");
    g.player_mut(P0).library.extend([island, drain]);
    let before = g.player(P1).life;
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Scarlet Witch") { Some(text(r, "Exsanguinate")) } else { None });
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a: witch, d: P1, dmg: 3 }).unwrap();
    assert!(!asked(&log, "choose X"), "nothing to pay X with: {}", dump(&log));
    assert_eq!(g.player(P1).life, before, "X = 0 drains nothing");
}

// ------------------------------------------------------------------ 2. Jin-Gitaxias keeps the targets
#[test]
fn jin_gitaxias_copy_for_a_person_offers_to_keep_the_original_target() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Jin-Gitaxias, Progress Tyrant");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let terror = take(&mut g, P0, "Terror");
    g.player_mut(P0).hand.push(terror);
    main_phase(&mut g);
    let log =
        person(&mut g, |r| if r.prompt.starts_with("The copy of Terror") { Some(text(r, "Birds")) } else { None });
    let ctx = sim_core::state::Ctx { target: Some(elves), ..Default::default() };
    cast::cast_card(&mut g, P0, terror, "hand".into(), ctx).unwrap();
    let l = dump(&log);
    let q = l.lines().find(|x| x.contains("The copy of Terror")).unwrap_or_else(|| panic!("no copy question: {l}"));
    assert!(q.contains("keep the same target (Llanowar Elves"), "the original's target is offered: {q}");
    assert_eq!(l.matches("The copy of Terror").count(), 1, "asked once: {l}");
    assert!(!g.perm(birds).on_bf, "the copy took the new target: {l}");
    assert!(!g.perm(elves).on_bf, "and the original its own: {l}");
}

#[test]
fn jin_gitaxias_copy_for_a_person_can_keep_the_same_target() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Jin-Gitaxias, Progress Tyrant");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let terror = take(&mut g, P0, "Terror");
    g.player_mut(P0).hand.push(terror);
    main_phase(&mut g);
    // Decline at the copy's question is the "keep the same target" choice
    person(&mut g, |r| if r.prompt.starts_with("The copy of Terror") { Some(Answer::Decline) } else { None });
    let ctx = sim_core::state::Ctx { target: Some(elves), ..Default::default() };
    cast::cast_card(&mut g, P0, terror, "hand".into(), ctx).unwrap();
    assert!(!g.perm(elves).on_bf, "the copy kept the target");
    assert!(g.perm(birds).on_bf, "the other creature was never chosen");
}

// ------------------------------------------------------------------ 3. Cityscape Leveler: unearth
#[test]
fn leveler_unearth_is_offered_to_a_person_and_pays_eight_from_the_pool() {
    let mut g = table(&["karn", "seph"]);
    let lev = card(&g, "Cityscape Leveler");
    g.player_mut(P0).gy.push(lev);
    main_phase(&mut g);
    person(&mut g, |_| None);
    assert!(sim_core::human::alt::gy_alts(&mut g, P0, lev).unwrap().is_empty(), "no mana in the pool: not offered");
    pool(&mut g, P0, &[('C', 8)]);
    let alts = sim_core::human::alt::gy_alts(&mut g, P0, lev).unwrap();
    assert_eq!(alts.len(), 1, "{:?}", alts.iter().map(|a| a.0.clone()).collect::<Vec<_>>());
    assert!(alts[0].0.contains("unearth"), "{}", alts[0].0);
    let i = g.player(P0).gy.iter().position(|&c| c == lev).unwrap();
    assert_eq!(apply(&mut g, P0, &json!({"do": "cast", "zone": "gy", "card": i})).unwrap(), None);
    let back = on_table(&g, P0, "Cityscape Leveler").expect("unearthed");
    assert!(!g.perm(back).sick, "haste");
    assert!(g.perm(back).data.truthy(DataKey::Unearth), "it is exiled at the end step");
    assert_eq!(pool_total(&g, P0), 0, "{{8}} paid");
    assert!(!g.player(P0).gy.contains(&lev));
}

#[test]
fn leveler_unearth_is_sorcery_speed_for_a_person() {
    let mut g = table(&["karn", "seph"]);
    let lev = card(&g, "Cityscape Leveler");
    g.player_mut(P0).gy.push(lev);
    pool(&mut g, P0, &[('C', 8)]);
    g.active = Some(P1);
    g.step = Step::Main1;
    person(&mut g, |_| None);
    assert!(sim_core::human::alt::gy_alts(&mut g, P0, lev).unwrap().is_empty(), "not on an opponent's turn");
}

// ------------------------------------------------------------------ 4. Dauthi Voidwalker: play it this turn
fn dauthi_table() -> (Game, PermId, sim_core::ids::CardId, sim_core::ids::CardId) {
    let mut g = table(&["thrasios", "seph"]);
    let d = perm(&mut g, P0, "Dauthi Voidwalker");
    let (ring, sig) = (card(&g, "Sol Ring"), card(&g, "Arcane Signet"));
    g.player_mut(P1).exile.extend([ring, sig]);
    let void = Val::List([ring, sig].iter().map(|&c| Val::List(vec![Val::Card(c), Val::Player(P1)])).collect());
    g.perm_mut(d).data.set(DataKey::Void, void);
    (g, d, ring, sig)
}

#[test]
fn dauthi_voidwalker_gives_a_permission_for_the_turn_not_an_instant_cast() {
    use sim_core::human::gaps_c::{dauthi_permits, dauthi_playable};
    use sim_core::human::legal::check_cast_exile;
    let (mut g, d, ring, sig) = dauthi_table();
    let h = person(&mut g, |r| r.prompt.contains("Dauthi").then(|| text(r, "Arcane Signet")));
    assert_eq!(use_ability(&mut g, P0, d, "void counter"), None);
    assert!(asked(&h, "play which exiled card"));
    assert!(!g.perm(d).on_bf, "sacrificed");
    assert!(on_table(&g, P0, "Arcane Signet").is_none(), "not cast yet: {}", dump(&h));
    assert!(g.player(P1).exile.contains(&sig), "still in exile until it is cast");
    assert_eq!(dauthi_permits(&g, P0, sig), Some(P1));
    assert_eq!(dauthi_permits(&g, P0, ring), None, "only the chosen card");
    assert_eq!(dauthi_playable(&g, P0), vec![(sig, P1)]);
    main_phase(&mut g);
    assert_eq!(check_cast_exile(&g, P0, sig), None);
    assert!(check_cast_exile(&g, P0, ring).is_some());
    // cast it, for no mana, from exile
    assert_eq!(pool_total(&g, P0), 0);
    assert_eq!(apply(&mut g, P0, &json!({"do": "cast", "zone": "ex", "card": sig.0})).unwrap(), None);
    assert!(on_table(&g, P0, "Arcane Signet").is_some(), "{}", dump(&h));
    assert!(!g.player(P1).exile.contains(&sig));
    assert!(g.player(P1).exile.contains(&ring), "the other stays exiled");
    assert_eq!(dauthi_permits(&g, P0, sig), None, "cast: the permission is used up");
}

#[test]
fn dauthi_voidwalker_permission_ends_with_the_turn_and_follows_sorcery_timing() {
    use sim_core::human::legal::check_cast_exile;
    let (mut g, d, _ring, sig) = dauthi_table();
    person(&mut g, |r| r.prompt.contains("Dauthi").then(|| text(r, "Arcane Signet")));
    assert_eq!(use_ability(&mut g, P0, d, "void counter"), None);
    g.active = Some(P1);
    g.step = Step::Main1;
    assert!(check_cast_exile(&g, P0, sig).is_some_and(|w| w.contains("Arcane Signet")), "an artifact: sorcery speed");
    main_phase(&mut g);
    g.round += 1;
    assert!(check_cast_exile(&g, P0, sig).is_some(), "a new turn: the permission is gone");
    assert!(g.player(P1).exile.contains(&sig));
}

// ------------------------------------------------------------------ 5. Mirage Mirror copies a land
#[test]
fn mirage_mirror_can_become_a_copy_of_a_land_and_goes_back_at_end_of_turn() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let isle = lands(&mut g, P1, "Island", 1, false)[0];
    pool(&mut g, P0, &[('C', 2)]);
    let log = person(&mut g, |r| r.prompt.contains("Mirage Mirror").then(|| text(r, "Island")));
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None);
    assert!(asked(&log, "become a copy of which"));
    assert_eq!(g.perm(mm).name, "Island", "{}", dump(&log));
    assert!(g.db.get(g.perm(mm).card()).land);
    assert!(g.player(P0).lands.contains(&mm), "listed with the lands");
    assert!(!g.player(P0).perms.contains(&mm), "and no longer with the permanents");
    assert!(g.perm(isle).on_bf, "the Island itself is untouched");
    sim_core::impls::alela::mirror_revert(&mut g).unwrap();
    assert_eq!(g.perm(mm).name, "Mirage Mirror", "it ends with the turn");
    assert!(g.player(P0).perms.contains(&mm) && !g.player(P0).lands.contains(&mm), "back with the permanents");
    assert!(g.perm(mm).phys.is_none());
}

#[test]
fn mirage_mirror_offers_a_land_with_abilities_of_its_own_too() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    lands(&mut g, P1, "Island", 1, false);
    lands(&mut g, P1, "Flooded Strand", 1, false);
    perm(&mut g, P1, "Sol Ring");
    pool(&mut g, P0, &[('C', 2)]);
    let log = person(&mut g, |r| r.prompt.contains("Mirage Mirror").then(|| text(r, "Island")));
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None);
    let l = dump(&log);
    let q = l.lines().find(|x| x.contains("become a copy of which")).unwrap();
    assert!(q.contains("Island") && q.contains("Sol Ring"), "{q}");
    assert!(
        q.contains("Flooded Strand"),
        "a fetch land is offered too (tests/practice_simplifications.rs cracks one): {q}"
    );
}
