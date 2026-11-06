//! model-gaps-a4: Siege-Gang Commander, Syr Konrad, Professional Face-Breaker, Tinybones and Ezuri have their activated
//! abilities (AI, person, and borrowed by Marvin, Murderous Mimic); a person chooses the targets of what Marvin
//! borrows (Kiki-Jiki, Chatterfang, ...); the Godo line's Helm of the Host equip is a copyable ability on the stack.

mod play_common;
use play_common::*;
use sim_core::control::Answer;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::impls::combos::{self, StepKind};
use sim_core::impls::model_gaps_a4 as mg;
use sim_core::state::Step;
use std::sync::Arc;

const MARVIN: &str = "Marvin, Murderous Mimic";

fn tbl() -> Game {
    table(&["karn", "kefka", "sauron"])
}

/// p0's turn, first main phase
fn main1(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

fn ready(g: &mut Game, m: PermId) {
    let x = g.perm_mut(m);
    x.sick = false;
    x.tapped = false;
}

/// the options hook of permanent `src` (a card's own, or Marvin's)
fn opts(g: &mut Game, src: PermId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, P0, post).unwrap()
}

fn run(g: &mut Game, o: &sim_core::hooks::Opt) -> bool {
    sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

type Rule = (&'static str, Box<dyn Fn(&sim_core::control::Request) -> Answer + Send>);

/// a person who answers by what is asked: each rule is used once, the first whose text is in the prompt
fn scripted(rules: Vec<Rule>) -> ScriptHuman {
    let mut rules: Vec<Option<Rule>> = rules.into_iter().map(Some).collect();
    ScriptHuman::from_fn(move |r| {
        for slot in rules.iter_mut() {
            if slot.as_ref().is_some_and(|(k, _)| r.prompt.contains(k) || &*r.kind == *k) {
                let (_, f) = slot.take().unwrap();
                return f(r);
            }
        }
        if r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    })
}

fn rule(key: &'static str, f: impl Fn(&sim_core::control::Request) -> Answer + Send + 'static) -> Rule {
    (key, Box::new(f))
}

fn first(_: &sim_core::control::Request) -> Answer {
    Answer::Pick(0)
}

fn pick_containing(word: String) -> impl Fn(&sim_core::control::Request) -> Answer + Send + 'static {
    move |r| Answer::Pick(r.choices.iter().position(|c| c.contains(&word)).unwrap_or_else(|| panic!("{word:?} in {:?}", r.choices)))
}

/// a person at P0 with these answers
fn person(g: &mut Game, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let h = Arc::new(scripted(rules));
    g.humans.add(P0, h.clone());
    h
}

/// run the person's ability whose label contains `word` on permanent m (Marvin's included)
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let abil = abilities_of(g, P0, m).unwrap();
    let (_, kind) = abil
        .into_iter()
        .find(|(l, _)| l.contains(word))
        .unwrap_or_else(|| panic!("no ability with {word:?}"));
    match kind {
        AbilityKind::Extra(f) => f(g, P0, m).unwrap(),
        _ => panic!("not an extra ability"),
    }
}

fn dead(g: &Game, q: PlayerId) -> bool {
    g.player(q).life <= 0 || !g.player(q).alive || g.over
}

// ======================================================== Siege-Gang Commander
fn siege_table(life: i32) -> (Game, PermId) {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Mountain", 4, false);
    let sg = perm(&mut g, P0, mg::SIEGE);
    ready(&mut g, sg);
    g.player_mut(P1).life = life;
    (g, sg)
}

#[test]
fn siege_gang_the_ai_pings_a_player_the_mana_and_goblins_reach() {
    let (mut g, sg) = siege_table(4);
    assert_eq!(mg::goblins(&g, P0).len(), 4, "Siege-Gang and its three tokens");
    let o = opts(&mut g, sg, Some(false));
    assert!(o.iter().any(|x| x.label.contains("Siege-Gang")), "{o:?}");
    assert!(run(&mut g, &o[0]));
    assert_eq!(g.player(P1).life, 2, "2 damage");
    assert!(sim_core::engine::values::epow(&g, sg) == 2 && g.perm(sg).on_bf, "a token was sacrificed, not Siege-Gang");
    assert_eq!(mg::goblins(&g, P0).len(), 3);
    let o = opts(&mut g, sg, Some(false));
    assert!(run(&mut g, &o[0]), "the second activation finishes it");
    assert!(dead(&g, P1));
    // out of reach (life 7 with mana for two pings): no ping at a player
    let (mut g, sg) = siege_table(7);
    assert!(opts(&mut g, sg, Some(false)).is_empty(), "nothing worth a Goblin");
}

#[test]
fn siege_gang_the_ai_finishes_a_planeswalker() {
    let (mut g, sg) = siege_table(40);
    let w = perm(&mut g, P1, "Liliana of the Veil");
    g.perm_mut(w).loyalty = Some(2);
    let o = opts(&mut g, sg, Some(false));
    assert!(o.iter().any(|x| x.label.contains("Liliana")), "{o:?}");
    let best = o.iter().max_by(|a, b| a.utility.total_cmp(&b.utility)).unwrap();
    assert!(run(&mut g, best));
    assert!(!g.perm(w).on_bf, "the walker died");
}

#[test]
fn siege_gang_a_person_chooses_the_target_and_the_goblin() {
    let (mut g, sg) = siege_table(20);
    pool(&mut g, P0, &[('R', 1), ('C', 1)]);
    let name = g.player(P1).name.to_string();
    person(&mut g, vec![rule("2 damage to which", pick_containing(name)), rule("which Goblin", first)]);
    let before = mg::goblins(&g, P0).len();
    assert!(use_ability(&mut g, sg, "sacrifice a Goblin").is_none());
    assert_eq!(g.player(P1).life, 18);
    assert_eq!(mg::goblins(&g, P0).len(), before - 1);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn siege_gang_marvin_uses_it_for_the_ai_and_for_a_person() {
    let (mut g, _sg) = siege_table(4);
    let marvin = perm(&mut g, P0, MARVIN);
    ready(&mut g, marvin);
    let o = opts(&mut g, marvin, Some(false));
    let o = o.iter().find(|x| x.label.contains("Siege-Gang")).expect("Marvin offers it");
    assert!(run(&mut g, o));
    assert_eq!(g.player(P1).life, 2);
    // a person, with the target and the Goblin of their choosing, the damage dealt by Marvin
    let (mut g, _sg) = siege_table(20);
    let marvin = perm(&mut g, P0, MARVIN);
    pool(&mut g, P0, &[('R', 1), ('C', 1)]);
    let name = g.player(P1).name.to_string();
    person(&mut g, vec![rule("2 damage to which", pick_containing(name)), rule("which Goblin", first)]);
    assert!(use_ability(&mut g, marvin, "Marvin copies Siege-Gang").is_none());
    assert_eq!(g.player(P1).life, 18);
}

// ======================================================== Syr Konrad, the Grim
fn konrad_table() -> (Game, PermId) {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Swamp", 4, false);
    let k = perm(&mut g, P0, mg::KONRAD);
    ready(&mut g, k);
    // libraries of creature cards: every mill is 1 damage to each opponent
    let bear = card(&g, "Llanowar Elves");
    for q in [P0, P1, P2] {
        g.player_mut(q).library = vec![bear; 30];
    }
    (g, k)
}

#[test]
fn konrad_the_ai_mills_each_player_and_the_creature_cards_hurt() {
    let (mut g, k) = konrad_table();
    let o = opts(&mut g, k, None);
    assert!(!o.is_empty(), "creature-heavy libraries are worth a mill");
    let (l0, l1, life1) = (g.player(P0).library.len(), g.player(P1).library.len(), g.player(P1).life);
    assert!(run(&mut g, &o[0]));
    assert_eq!((g.player(P0).library.len(), g.player(P1).library.len()), (l0 - 1, l1 - 1));
    assert_eq!(g.player(P1).life, life1 - 3, "three players milled a creature card each: 1 damage each");
    // its own library nearly empty: it doesn't
    g.player_mut(P0).library.truncate(5);
    assert!(opts(&mut g, k, None).is_empty());
}

#[test]
fn konrad_a_person_and_marvin() {
    let (mut g, k) = konrad_table();
    pool(&mut g, P0, &[('B', 1), ('C', 1)]);
    person(&mut g, vec![]);
    let l1 = g.player(P1).library.len();
    assert!(use_ability(&mut g, k, "each player mills").is_none());
    assert_eq!(g.player(P1).library.len(), l1 - 1);
    assert_eq!(pool_total(&g, P0), 0);
    // Marvin (a person)
    let (mut g, _k) = konrad_table();
    let marvin = perm(&mut g, P0, MARVIN);
    pool(&mut g, P0, &[('B', 1), ('C', 1)]);
    person(&mut g, vec![]);
    let l1 = g.player(P1).library.len();
    assert!(use_ability(&mut g, marvin, "Marvin copies Syr Konrad").is_none());
    assert_eq!(g.player(P1).library.len(), l1 - 1);
    // Marvin (the AI)
    let (mut g, _k) = konrad_table();
    let marvin = perm(&mut g, P0, MARVIN);
    ready(&mut g, marvin);
    let o = opts(&mut g, marvin, None);
    let o = o.iter().find(|x| x.label.contains("Syr Konrad")).expect("Marvin offers the mill");
    let l1 = g.player(P1).library.len();
    assert!(run(&mut g, o));
    assert_eq!(g.player(P1).library.len(), l1 - 1);
}

// ======================================================== Professional Face-Breaker
fn breaker_table() -> (Game, PermId) {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Mountain", 3, false);
    let b = perm(&mut g, P0, mg::FACE_BREAKER);
    ready(&mut g, b);
    g.player_mut(P0).treasures = 2;
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).lands_played = 0;
    (g, b)
}

#[test]
fn face_breaker_exiles_the_top_card_playable_this_turn() {
    let (mut g, b) = breaker_table();
    let mtn = card(&g, "Mountain");
    g.player_mut(P0).library = vec![mtn; 20];
    let o = opts(&mut g, b, Some(false));
    assert!(!o.is_empty(), "a land with the land drop open is worth a Treasure");
    assert!(run(&mut g, &o[0]));
    assert_eq!(g.player(P0).treasures, 1);
    assert_eq!(g.player(P0).library.len(), 19);
    assert!(g.player(P0).hand.contains(&mtn) && g.player(P0).impulse.contains(&mtn), "playable this turn");
    // no Treasure, no ability
    g.player_mut(P0).treasures = 0;
    assert!(opts(&mut g, b, Some(false)).is_empty());
    // a library of expensive spells and no mana to spare: not worth it
    let (mut g, b) = breaker_table();
    let bomb = card(&g, "Grave Titan");
    g.player_mut(P0).library = vec![bomb; 20];
    g.player_mut(P0).lands_played = 1;
    assert!(opts(&mut g, b, Some(false)).is_empty());
}

#[test]
fn face_breaker_a_person_and_marvin() {
    let (mut g, b) = breaker_table();
    person(&mut g, vec![]);
    let top = *g.player(P0).library.last().unwrap();
    assert!(use_ability(&mut g, b, "sacrifice a Treasure").is_none());
    assert_eq!(g.player(P0).treasures, 1);
    assert!(g.player(P0).impulse.contains(&top));
    let (mut g, _b) = breaker_table();
    let marvin = perm(&mut g, P0, MARVIN);
    person(&mut g, vec![]);
    let top = *g.player(P0).library.last().unwrap();
    assert!(use_ability(&mut g, marvin, "Marvin copies Professional Face-Breaker").is_none());
    assert_eq!(g.player(P0).treasures, 1);
    assert!(g.player(P0).impulse.contains(&top));
    // the AI's Marvin
    let (mut g, _b) = breaker_table();
    let mtn = card(&g, "Mountain");
    g.player_mut(P0).library = vec![mtn; 20];
    let marvin = perm(&mut g, P0, MARVIN);
    ready(&mut g, marvin);
    let o = opts(&mut g, marvin, Some(false));
    let o = o.iter().find(|x| x.label.contains("Face-Breaker")).expect("Marvin offers it");
    assert!(run(&mut g, o));
    assert_eq!(g.player(P0).treasures, 1);
}

// ======================================================== Tinybones, Trinket Thief
fn tiny_table(life: i32) -> (Game, PermId) {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Swamp", 6, false);
    let t = perm(&mut g, P0, mg::TINYBONES);
    ready(&mut g, t);
    g.player_mut(P1).hand.clear();
    g.player_mut(P1).life = life;
    // the others keep cards
    (g, t)
}

#[test]
fn tinybones_the_ai_kills_an_empty_handed_opponent_at_ten() {
    let (mut g, t) = tiny_table(9);
    let o = opts(&mut g, t, Some(false));
    assert!(o.iter().any(|x| x.utility >= 8.0), "{o:?}");
    assert!(run(&mut g, &o[0]));
    assert!(dead(&g, P1));
    // an opponent with a card in hand isn't touched, and with 6 or more mana short there is nothing to do
    let (mut g, t) = tiny_table(9);
    g.player_mut(P1).hand = vec![card(&g, "Island")];
    assert!(opts(&mut g, t, Some(false)).iter().all(|x| x.utility < 8.0) || tinybones_none(&mut g, t));
    let (mut g, t) = tiny_table(40);
    assert!(opts(&mut g, t, Some(false)).is_empty(), "40 life: not close");
}

fn tinybones_none(g: &mut Game, t: PermId) -> bool {
    let o = opts(g, t, Some(false));
    let life = g.player(P1).life;
    if o.is_empty() {
        return true;
    }
    run(g, &o[0]);
    g.player(P1).life == life // the one with a card in hand lost nothing
}

#[test]
fn tinybones_a_person_and_marvin() {
    let (mut g, t) = tiny_table(25);
    pool(&mut g, P0, &[('B', 2), ('C', 4)]);
    person(&mut g, vec![]);
    assert!(use_ability(&mut g, t, "10 life").is_none());
    assert_eq!(g.player(P1).life, 15);
    assert_eq!(pool_total(&g, P0), 0);
    let (mut g, _t) = tiny_table(25);
    let marvin = perm(&mut g, P0, MARVIN);
    pool(&mut g, P0, &[('B', 2), ('C', 4)]);
    person(&mut g, vec![]);
    assert!(use_ability(&mut g, marvin, "Marvin copies Tinybones").is_none());
    assert_eq!(g.player(P1).life, 15);
    let (mut g, _t) = tiny_table(9);
    let marvin = perm(&mut g, P0, MARVIN);
    ready(&mut g, marvin);
    let o = opts(&mut g, marvin, Some(false));
    let o = o.iter().find(|x| x.label.contains("Tinybones")).expect("Marvin offers it");
    assert!(run(&mut g, o));
    assert!(dead(&g, P1));
}

// ======================================================== Ezuri, Renegade Leader
fn ezuri_table(elves: usize, life: i32) -> (Game, PermId, Vec<PermId>) {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Forest", 5, false);
    let e = perm(&mut g, P0, mg::EZURI);
    ready(&mut g, e);
    let es: Vec<PermId> = (0..elves)
        .map(|_| {
            let m = perm(&mut g, P0, "Llanowar Elves");
            ready(&mut g, m);
            g.perm_mut(m).noatk = false; // a mana creature the AI would not send to attack: here it does
            m
        })
        .collect();
    for q in [P1, P2] {
        g.player_mut(q).life = life;
    }
    (g, e, es)
}

#[test]
fn ezuri_the_ai_overruns_only_for_a_lethal_or_near_lethal_attack() {
    // 1 Ezuri (2) + 5 Elves (1 each) = 7; +3 each on 6 Elves = 25 against 20 life: lethal
    let (mut g, e, es) = ezuri_table(5, 20);
    g.player_mut(P2).life = 99;
    let o = opts(&mut g, e, Some(false));
    assert!(o.iter().any(|x| x.utility >= 9.0), "the pump makes the attack lethal: {o:?}");
    assert!(run(&mut g, &o[0]));
    for &m in &es {
        assert_eq!(sim_core::engine::values::epow(&g, m), 4, "+3/+3");
        assert!(sim_core::dsl::has_kw(&g, m, "trample"), "and trample");
    }
    assert_eq!(sim_core::engine::values::epow(&g, e), 5);
    // not before combat's end (main 2) and not when nobody is close
    let (mut g, e, _) = ezuri_table(5, 20);
    g.player_mut(P2).life = 99;
    assert!(opts(&mut g, e, Some(true)).is_empty(), "only before combat");
    let (mut g, e, _) = ezuri_table(5, 80);
    g.player_mut(P2).life = 99;
    assert!(opts(&mut g, e, Some(false)).is_empty(), "80 life: not close");
    // many ready Elves used to be enough on their own: with nothing to gain it stays silent
    let (mut g, e, _) = ezuri_table(8, 5);
    g.player_mut(P2).life = 99;
    assert!(opts(&mut g, e, Some(false)).is_empty(), "already lethal without it");
}

#[test]
fn ezuri_a_person_and_marvin() {
    let (mut g, e, es) = ezuri_table(2, 40);
    pool(&mut g, P0, &[('G', 3), ('C', 2)]);
    person(&mut g, vec![]);
    assert!(use_ability(&mut g, e, "+3/+3").is_none());
    assert_eq!(sim_core::engine::values::epow(&g, es[0]), 4);
    assert_eq!(pool_total(&g, P0), 0);
    let (mut g, _e, es) = ezuri_table(2, 40);
    let marvin = perm(&mut g, P0, MARVIN);
    pool(&mut g, P0, &[('G', 3), ('C', 2)]);
    person(&mut g, vec![]);
    assert!(use_ability(&mut g, marvin, "Marvin copies Ezuri").is_none());
    assert_eq!(sim_core::engine::values::epow(&g, es[0]), 4);
    let (mut g, _e, es) = ezuri_table(5, 20);
    g.player_mut(P2).life = 99;
    let marvin = perm(&mut g, P0, MARVIN);
    ready(&mut g, marvin);
    let o = opts(&mut g, marvin, Some(false));
    let o = o.iter().find(|x| x.label.contains("Ezuri")).expect("Marvin offers the overrun");
    assert!(run(&mut g, o));
    assert_eq!(sim_core::engine::values::epow(&g, es[0]), 4);
}

// ======================================================== a person chooses what Marvin borrows
#[test]
fn a_person_chooses_the_creature_marvin_copies_with_kiki_jiki() {
    let mut g = tbl();
    main1(&mut g);
    let marvin = perm(&mut g, P0, MARVIN);
    let kiki = perm(&mut g, P0, "Kiki-Jiki, Mirror Breaker");
    let angel = perm(&mut g, P0, "Serra Angel");
    let bears = perm(&mut g, P0, "Llanowar Elves");
    for m in [marvin, kiki, angel, bears] {
        ready(&mut g, m);
    }
    // the AI picks the better creature: the person picks the Bears
    let asked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let a2 = asked.clone();
    person(
        &mut g,
        vec![rule("copy which", move |r| {
            a2.store(true, std::sync::atomic::Ordering::SeqCst);
            Answer::Pick(r.choices.iter().position(|c| c.contains("Llanowar")).unwrap())
        })],
    );
    let (bears_n, angels_n) = (perms_named(&g, P0, "Llanowar Elves").len(), perms_named(&g, P0, "Serra Angel").len());
    assert!(use_ability(&mut g, marvin, "Kiki-Jiki").is_none());
    assert!(asked.load(std::sync::atomic::Ordering::SeqCst), "the person was asked");
    assert!(g.perm(marvin).tapped && !g.perm(kiki).tapped, "Marvin paid the {{T}}");
    assert_eq!(perms_named(&g, P0, "Llanowar Elves").len(), bears_n + 1, "a copy of the Bears");
    assert_eq!(perms_named(&g, P0, "Serra Angel").len(), angels_n, "not the Angel the AI would take");
}

#[test]
fn a_person_chooses_the_target_and_x_of_marvins_chatterfang() {
    let mut g = tbl();
    main1(&mut g);
    lands(&mut g, P0, "Swamp", 2, false);
    let marvin = perm(&mut g, P0, MARVIN);
    let fang = perm(&mut g, P0, "Chatterfang, Squirrel General");
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let other = perm(&mut g, P1, "Serra Angel");
    for _ in 0..3 {
        let spec = sim_core::engine::zones::Tokens { types: vec!["squirrel"], ..sim_core::engine::zones::Tokens::new(1, 1) };
        sim_core::engine::zones::make_tokens(&mut g, P0, spec).unwrap();
    }
    let sq = sim_core::impls::rules2::squirrels(&g, P0).len();
    assert!(sq >= 2, "Chatterfang makes Squirrels (got {sq})");
    ready(&mut g, marvin);
    ready(&mut g, fang);
    pool(&mut g, P0, &[('B', 1)]);
    let asked = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let a2 = asked.clone();
    person(
        &mut g,
        vec![
            rule("to which creature", move |r| {
                a2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Answer::Pick(r.choices.iter().position(|c| c.contains("Llanowar")).unwrap())
            }),
            rule("how many Squirrels", pick_containing("X = 2".to_string())),
        ],
    );
    let t0 = sim_core::engine::values::etgh(&g, victim);
    assert!(use_ability(&mut g, marvin, "Chatterfang").is_none());
    assert_eq!(asked.load(std::sync::atomic::Ordering::SeqCst), 1, "the person chose the target");
    assert!(!g.perm(victim).on_bf, "toughness {t0} - 2 kills the Bears");
    assert!(g.perm(other).on_bf, "the Angel is untouched");
    assert_eq!(sim_core::impls::rules2::squirrels(&g, P0).len(), sq - 2, "two Squirrels sacrificed");
    assert!(!g.perm(marvin).tapped, "Chatterfang's ability has no {{T}}");
}

#[test]
fn a_person_uses_marvins_krenko() {
    let mut g = tbl();
    main1(&mut g);
    let marvin = perm(&mut g, P0, MARVIN);
    let krenko = perm(&mut g, P0, "Krenko, Mob Boss");
    perm(&mut g, P0, "Mogg Fanatic");
    ready(&mut g, marvin);
    ready(&mut g, krenko);
    person(&mut g, vec![]);
    let n = |g: &Game| mg::goblins(g, P0).len();
    let before = n(&g);
    let abil = abilities_of(&mut g, P0, marvin).unwrap();
    assert!(abil.iter().any(|(l, _)| l.contains("Krenko")), "{:?}", abil.iter().map(|a| &a.0).collect::<Vec<_>>());
    let (_, kind) = abil.into_iter().find(|(l, _)| l.contains("Krenko")).unwrap();
    match kind {
        AbilityKind::Hook(a) => {
            assert!(sim_core::ai::act::perform(&mut g, P0, &a).unwrap());
        }
        _ => panic!("a hook"),
    }
    assert!(n(&g) > before, "Marvin's Krenko made Goblins");
    assert!(g.perm(marvin).tapped);
}

// ======================================================== the Godo line's equip
#[test]
fn the_godo_helm_equip_steps_are_copyable_abilities_on_the_stack() {
    let equips: Vec<_> = combos::all()
        .iter()
        .flat_map(|c| c.steps.iter().map(move |s| (c.name, s)))
        .filter(|(_, s)| s.label.starts_with("equip Helm of the Host"))
        .collect();
    assert!(equips.len() >= 2, "the {{5}} line and the Kinnan loop's");
    for (name, s) in equips {
        assert_eq!(s.kind, StepKind::Equip, "{name}: {}", s.label);
    }
}

fn attach_helm(g: &mut Game, p: PlayerId) -> sim_core::flow::Res<bool> {
    let h = perms_named(g, p, "Helm of the Host")[0];
    let t = perms_named(g, p, "Godo, Bandit Warlord")[0];
    Ok(sim_core::impls::cloud::attach(g, h, t))
}

fn helm_src(g: &Game, p: PlayerId) -> Option<PermId> {
    perms_named(g, p, "Helm of the Host").first().copied()
}

#[test]
fn an_equip_step_can_be_copied_by_a_camera_and_responded_to() {
    let mut g = tbl();
    main1(&mut g);
    let helm = perm(&mut g, P0, "Helm of the Host");
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(helm).attached = None;
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    pool(&mut g, P0, &[('C', 10)]);
    let ci = g.player(P0).perms.iter().position(|&x| x == cam).unwrap();
    let films = |g: &Game| g.perm(cam).data.int(sim_core::state::DataKey::Counters);
    let before = films(&g);
    // the person has priority with the equip on the stack: the Camera copies it
    let stack_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s2 = stack_seen.clone();
    person(
        &mut g,
        vec![
            rule("You activate", move |r| {
                s2.store(true, std::sync::atomic::Ordering::SeqCst);
                let _ = r;
                Answer::Action(json!({"do": "use", "perm": ci}))
            }),
            rule("which ability", first),
            rule("copy which", first),
        ],
    );
    static STEP: sim_core::impls::combos::Step = sim_core::impls::combos::Step {
        label: "equip Helm of the Host to Godo",
        kind: StepKind::Equip,
        src: helm_src,
        pay: (5, ""),
        run: attach_helm,
        keys: sim_core::impls::combos::no_keys,
    };
    static STEPS: [sim_core::impls::combos::Step; 1] = [STEP];
    let cmb = sim_core::impls::combos::Combo {
        name: "test equip",
        groups: &[],
        ready: |_, _| unreachable!(),
        mana: (0, ""),
        finish: Some(|_, _| Ok(())),
        xcost: &[],
        steps: &STEPS,
    };
    assert!(combos::run_steps(&mut g, P0, &cmb).unwrap());
    assert!(stack_seen.load(std::sync::atomic::Ordering::SeqCst), "the person had priority with it on the stack");
    assert_eq!(g.perm(helm).attached, Some(godo), "equipped");
    assert_eq!(films(&g), before - 1, "the Camera spent a film counter copying it");
    assert!(g.perm(cam).tapped);
}


// ======================================================== item 2: a person chooses what Marvin borrows
use sim_core::human::marvin_hooks::NAMES as HOOK_NAMES;
use sim_core::human::model_gaps_a4::BORROWED;
use std::sync::atomic::{AtomicUsize, Ordering};

/// the creatures whose `options` hook offers no choice (nothing to target, pick, sacrifice or search for): they stay
/// hooks for a person, listed by `abilities_of` as the AI's option
const NO_CHOICE: &[&str] = &[
    "Allosaurus Shepherd",
    "Battle Cry Goblin",
    "Bloodline Keeper // Lord of Lineage",
    "Bristly Bill, Spine Sower",
    "Deadpool, Trading Card",
    "Elvish Clancaller",
    "Elvish Warmaster",
    "Gilded Goose",
    "Incubation Druid",
    "Joraga Treespeaker",
    "Kaza, Roil Chaser",
    "King Darien XLVIII",
    "Krenko, Mob Boss",
    "Lathliss, Dragon Queen",
    "Pteramander",
    "Razia, Boros Archangel",
    "Savvy Hunter",
    "Sawblade Scamp",
    "Shalai, Voice of Plenty",
    "Speaker of the Heavens",
    "Thunderclap Drake",
    "Tolsimir Wolfblood",
    "Twilight Drover",
    "Marvin, Murderous Mimic",
];

#[test]
fn every_creature_with_an_options_hook_is_a_person_path_a_no_choice_hook_or_not_an_ability() {
    let g = tbl();
    let mut bad = vec![];
    for d in g.db.iter().filter(|d| d.creature) {
        let n: &str = &d.name;
        let Some(cd) = g.db.id(n) else { continue };
        if g.registry.get(cd).and_then(|c| c.options).is_none() {
            continue;
        }
        let known = BORROWED.contains(&n)
            || HOOK_NAMES.contains(&n)
            || NO_CHOICE.contains(&n)
            || sim_core::impls::newcards_b::NOT_ACTIVATED.contains(&n);
        // a creature of the pool decks whose person path the Karn / Cloud / Sauron decks keep elsewhere
        let elsewhere = matches!(n, "Walking Ballista" | "Demon of Fate's Design" | "Gingerbrute" | "Thrasios, Triton Hero");
        if !known && !elsewhere {
            bad.push(n.to_string());
        }
    }
    assert!(bad.is_empty(), "creatures with a hook and no classification: {bad:?}");
}

/// a table with everything a borrowed ability could want: lands of every color, a full pool, cards in hands, libraries
/// and graveyards, creatures of the types the costs ask for, opponents' creatures, artifacts, enchantments
fn rich(donor: &str) -> (Game, PermId, PermId) {
    let mut g = tbl();
    main1(&mut g);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, l, 3, true);
    }
    pool(&mut g, P0, &[('W', 6), ('U', 6), ('B', 6), ('R', 6), ('G', 6), ('C', 8)]);
    g.player_mut(P0).treasures = 6;
    g.player_mut(P0).foods = 2;
    g.player_mut(P0).life = 30;
    for _ in 0..2 {
        let _ = perm(&mut g, P0, "Llanowar Elves");
    }
    for n in ["Llanowar Elves"; 12] {
        let e = perm(&mut g, P0, n);
        ready(&mut g, e);
    }
    for n in ["Goblin Guide", "Goblin Chieftain", "Mogg War Marshal", "Vampire Nighthawk", "Stromkirk Noble", "Elesh Norn, Grand Cenobite", "Mother of Runes", "Isamaru, Hound of Konda", "Sol Ring", "Mind Stone", "Relic of Progenitus"] {
        if g.db.id(n).is_some() {
            let x = perm(&mut g, P0, n);
            ready(&mut g, x);
        }
    }
    for n in ["Serra Angel", "Baneslayer Angel", "Grizzly Bears", "Vampire Nighthawk"] {
        if g.db.id(n).is_some() {
            let x = perm(&mut g, P1, n);
            ready(&mut g, x);
        }
    }
    for n in ["Sol Ring", "Pernicious Deed", "Rest in Peace"] {
        if g.db.id(n).is_some() {
            let _ = perm(&mut g, P1, n);
        }
    }
    let ids: Vec<_> = ["Llanowar Elves", "Lightning Bolt", "Sol Ring", "Forest", "Serra Angel", "Swords to Plowshares", "Goblin Welder", "Mind Stone"]
        .iter()
        .filter(|n| g.db.id(n).is_some())
        .map(|n| card(&g, n))
        .collect();
    for q in [P0, P1] {
        g.player_mut(q).gy = ids.clone();
        g.player_mut(q).hand = ids.iter().copied().take(4).collect();
    }
    let tops = vec![card(&g, "Llanowar Elves"); 8];
    g.player_mut(P0).library.extend(tops);
    let d = perm(&mut g, P0, donor);
    ready(&mut g, d);
    let m = perm(&mut g, P0, MARVIN);
    ready(&mut g, m);
    (g, d, m)
}

fn asking_person(g: &mut Game) -> Arc<AtomicUsize> {
    let n = Arc::new(AtomicUsize::new(0));
    let n2 = n.clone();
    let h = ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        n2.fetch_add(1, Ordering::SeqCst);
        if r.choices.is_empty() { Answer::Decline } else { Answer::Pick(0) }
    });
    g.humans.add(P0, Arc::new(h));
    n
}

/// the borrowed abilities with a choice that stay quiet in the rich table, and why
const QUIET: &[(&str, &str)] = &[
    ("Balan, Wandering Knight", "attaches every Equipment: nothing to choose"),
    ("Nicol Bolas, the Ravager", "the flip has nothing to choose"),
    ("Resplendent Angel", "+2/+2 and lifelink: nothing to choose"),
    ("Steel Hellkite", "the +1/+0 and the X: nothing to choose in the pump"),
    ("Steel Overseer", "a counter on each artifact creature: nothing to choose"),
    ("Urza, Lord High Artificer", "play the top card free: nothing to choose"),
    ("Syr Konrad, the Grim", "each player mills: nothing to choose"),
    ("Professional Face-Breaker", "sacrifices a Treasure: the Treasure is not a choice"),
    ("Tinybones, Trinket Thief", "each opponent: nothing to choose"),
    ("Ezuri, Renegade Leader", "all Elves: nothing to choose"),
    ("Blightspeaker", "its search finds no Rebel in the test library"),
    ("Ramosian Sergeant", "its search finds no Rebel in the test library"),
    ("Jace's Archivist", "everyone discards and draws: nothing to choose"),
    ("Scourge of Valkas", "{R}: +1/+0 has nothing to choose"),
    ("Kefka, Court Mage // Kefka, Ruler of Ruin", "the opponents (AIs here) choose what they sacrifice"),
    ("Siege-Gang Commander", "asks the target, the Goblin only when several: see its own test"),
];

#[test]
fn a_person_using_each_borrowed_ability_is_asked_and_never_gets_the_ais_choice() {
    let all: Vec<&str> = BORROWED.iter().chain(HOOK_NAMES.iter()).copied().collect();
    let mut silent = vec![];
    let mut missing = vec![];
    let mut hooks = vec![];
    for name in all {
        if (name.starts_with("Ramosian") && name != "Ramosian Sergeant") || name == "Defiant Vanguard" {
            continue; // the same Rebel searchers are walked through the Sergeant
        }
        let (mut g, _d, m) = rich(name);
        let _ = asking_person(&mut g);
        let want = format!("Marvin copies {name}:");
        let labels: Vec<(String, bool)> = abilities_of(&mut g, P0, m)
            .unwrap()
            .into_iter()
            .filter(|(l, _)| l.starts_with(&want))
            .map(|(l, k)| (l, matches!(k, AbilityKind::Extra(_))))
            .collect();
        if labels.is_empty() {
            missing.push(name);
            continue;
        }
        if labels.iter().any(|(_, extra)| !extra) {
            hooks.push(name);
        }
        for (i, (label, _)) in labels.iter().enumerate() {
            let (mut g, _d, m) = rich(name);
            let asked = asking_person(&mut g);
            let abil = abilities_of(&mut g, P0, m).unwrap();
            let (l2, kind) = abil.into_iter().filter(|(l, _)| l.starts_with(&want)).nth(i).unwrap();
            assert_eq!(&l2, label);
            let AbilityKind::Extra(f) = kind else { continue };
            let why = f(&mut g, P0, m).unwrap_or_else(|e| panic!("{label}: {e:?}"));
            if asked.load(Ordering::SeqCst) == 0 && why.is_none() && !QUIET.iter().any(|q| q.0 == name) {
                silent.push(format!("{label} (done without a question)"));
            }
        }
    }
    assert!(missing.is_empty(), "no borrowed ability listed for a person: {missing:?}");
    assert!(hooks.is_empty(), "borrowed as the AI's hook (the AI picks the target) for a person: {hooks:?}");
    assert!(silent.is_empty(), "a person was not asked anything: {silent:#?}");
}

// ---- spot tests: what a person picks is what happens, with Marvin as the source
#[test]
fn marvin_mogg_fanatic_the_person_picks_the_target_and_marvin_is_sacrificed() {
    let (mut g, _d, m) = rich("Mogg Fanatic");
    let name = g.player(P2).name.to_string();
    person(&mut g, vec![rule("1 damage to which", pick_containing(name))]);
    let before = g.player(P2).life;
    assert!(use_ability(&mut g, m, "Marvin copies Mogg Fanatic").is_none());
    assert_eq!(g.player(P2).life, before - 1, "the person aimed it at P2, not at the AI's pick");
    assert!(!g.perm(m).on_bf, "Marvin was sacrificed (the cost's 'this creature')");
}

#[test]
fn marvin_cankerbloom_the_person_picks_the_mode_and_the_target() {
    let (mut g, _d, m) = rich("Cankerbloom");
    let deed = perms_named(&g, P1, "Pernicious Deed")[0];
    person(&mut g, vec![rule("choose one", pick_containing("destroy target enchantment".into())), rule("destroy which", pick_containing("Pernicious Deed".into()))]);
    assert!(use_ability(&mut g, m, "Marvin copies Cankerbloom").is_none());
    assert!(!g.perm(deed).on_bf, "the person's target died");
    assert!(!g.perm(m).on_bf, "Marvin is the sacrifice");
}

#[test]
fn marvin_chittering_witch_the_person_picks_the_creature_to_sacrifice_and_the_target() {
    let (mut g, _d, m) = rich("Chittering Witch");
    let angel = perms_named(&g, P1, "Serra Angel")[0];
    person(&mut g, vec![rule("-2/-2 on which", pick_containing("(Kefka, 0/1)".into())), rule("sacrifice which", pick_containing("Mogg War Marshal".into()))]);
    let marshals = perms_named(&g, P0, "Mogg War Marshal");
    assert!(use_ability(&mut g, m, "Marvin copies Chittering Witch").is_none());
    assert!(perms_named(&g, P1, "Vampire Nighthawk").is_empty(), "the 0/1 took -2/-2");
    assert!(g.perm(angel).on_bf);
    assert!(marshals.iter().any(|&x| !g.perm(x).on_bf), "the person's choice of sacrifice");
}

#[test]
fn marvin_tekuthal_puts_the_indestructible_counter_on_tekuthal_and_the_person_picks_the_counters() {
    let (mut g, tek, m) = rich("Tekuthal, Inquiry Dominus");
    let elves = perms_named(&g, P0, "Llanowar Elves");
    for &e in elves.iter().take(3) {
        g.perm_mut(e).plus = 1;
    }
    person(&mut g, vec![rule("remove a counter", first)]);
    assert!(use_ability(&mut g, m, "Marvin copies Tekuthal").is_none());
    assert!(g.perm(tek).data.truthy(sim_core::state::DataKey::Indestr), "the text names Tekuthal");
    assert!(!g.perm(m).data.truthy(sim_core::state::DataKey::Indestr));
    assert!(elves.iter().take(3).all(|&e| g.perm(e).plus == 0), "three counters were removed");
}

#[test]
fn marvin_does_not_borrow_what_is_not_an_activated_ability() {
    // Muldrotha casts from the graveyard and Realmwalker from the top of the library: static permissions
    for name in ["Muldrotha, the Gravetide", "Realmwalker", "Conspicuous Snoop"] {
        let (mut g, _d, m) = rich(name);
        let o = opts(&mut g, m, Some(false));
        assert!(!o.iter().any(|x| x.label.contains(name)), "{name}: {o:?}");
    }
}

#[test]
fn a_person_chooses_how_the_opponents_block_with_a_brutal_hordechief_through_marvin() {
    let (mut g, _d, m) = rich("Brutal Hordechief");
    g.step = Step::Main1;
    person(&mut g, vec![rule("pay {R/W}", first)]);
    assert!(use_ability(&mut g, m, "Marvin copies Brutal Hordechief").is_none());
    assert!(matches!(g.perm(m).data.get(sim_core::state::DataKey::Lure), Some(_)), "Marvin carries the lure for this turn");
}
