//! Practice-mode trigger audit, Agent T1 (rust/agents/practice-followups.md): triggers later in a game whose choice the
//! AI's card code made for the person (Cityscape Leveler, Iron Man, Korvold, Kura, Mishra's Self-Replicator, Myr
//! Battlesphere, Portal to Phyrexia, Scarlet Witch, Susurian Voidborn, Sylvan Library, The Ozolith, Vision), and a few
//! cards found to ask already (Solemn Simulacrum's dies trigger, Niv-Mizzet). Each test seats a scripted person, fires
//! the trigger the way the engine does, and checks that the person was asked and that the answer is what happened.

mod play_common;
use play_common::*;
use sim_core::engine::{cast, combat, hooks, zones};
use sim_core::hooks::{Call, Event};
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

fn attack(g: &mut Game, a: PermId, d: PlayerId) {
    hooks::fire_trigger(g, Event::Attack, Call::Attack { p: P0, atk: vec![a], d }).unwrap();
}

// ------------------------------------------------------------------ Cityscape Leveler
#[test]
fn leveler_attack_trigger_destroys_the_permanent_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let lev = perm(&mut g, P0, "Cityscape Leveler");
    let ring = perm(&mut g, P1, "Sol Ring");
    let tok = token(&mut g, P1, 1);
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Cityscape Leveler") { Some(text(r, "Sol Ring")) } else { None });
    attack(&mut g, lev, P1);
    assert!(asked(&log, "Cityscape Leveler (attack)"), "{}", dump(&log));
    assert!(!g.perm(ring).on_bf, "the person's pick was destroyed: {}", dump(&log));
    assert!(g.perm(tok).on_bf);
    assert!(g.perm(lev).on_bf);
    // its controller got a tapped Powerstone
    assert!(g.player(P1).perms.iter().any(|&m| g.perm(m).name == "Powerstone"), "a Powerstone token");
}

#[test]
fn leveler_attack_trigger_may_target_nothing() {
    let mut g = table(&["karn", "seph"]);
    let lev = perm(&mut g, P0, "Cityscape Leveler");
    let ring = perm(&mut g, P1, "Sol Ring");
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Cityscape Leveler") { Some(text(r, "no target")) } else { None });
    attack(&mut g, lev, P1);
    assert!(asked(&log, "Cityscape Leveler (attack)"), "{}", dump(&log));
    assert!(g.perm(ring).on_bf, "up to one: none chosen");
}

#[test]
fn leveler_cast_trigger_asks_for_its_target() {
    let mut g = table(&["karn", "seph"]);
    let ring = perm(&mut g, P1, "Sol Ring");
    let tok = token(&mut g, P1, 1);
    let c = take(&mut g, P0, "Cityscape Leveler");
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Cityscape Leveler") { Some(text(r, "Sol Ring")) } else { None });
    cast::on_cast(&mut g, P0, c).unwrap();
    assert!(asked(&log, "Cityscape Leveler (cast)"), "{}", dump(&log));
    assert!(!g.perm(ring).on_bf && g.perm(tok).on_bf, "{}", dump(&log));
}

// ------------------------------------------------------------------ Iron Man, Armored Avenger
#[test]
fn iron_man_puts_the_counter_on_the_creature_the_person_picks() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "Iron Man, Armored Avenger");
    let tok = token(&mut g, P0, 1);
    let log = person(&mut g, |r| if r.prompt.starts_with("Iron Man") { Some(text(r, "1/1")) } else { None });
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(asked(&log, "Iron Man: put a +1/+1 counter"), "{}", dump(&log));
    assert_eq!(g.perm(tok).plus, 1, "{}", dump(&log));
}

// ------------------------------------------------------------------ Korvold, Fae-Cursed King
#[test]
fn korvold_enter_trigger_sacrifices_what_the_person_picks() {
    let mut g = table(&["urdragon", "seph"]);
    let ring = perm(&mut g, P0, "Sol Ring");
    g.player_mut(P0).treasures = 1;
    let log = person(&mut g, |r| if r.prompt.starts_with("Korvold") { Some(text(r, "Sol Ring")) } else { None });
    let k = perm(&mut g, P0, "Korvold, Fae-Cursed King");
    assert!(asked(&log, "Korvold: sacrifice which other permanent"), "{}", dump(&log));
    assert!(!g.perm(ring).on_bf, "Sol Ring was sacrificed, not the Treasure the AI would have picked: {}", dump(&log));
    assert_eq!(g.player(P0).treasures, 1);
    assert_eq!(g.perm(k).plus, 1, "Korvold's own trigger: a +1/+1 counter");
}

#[test]
fn korvold_attack_trigger_can_sacrifice_a_treasure_or_a_land() {
    let mut g = table(&["urdragon", "seph"]);
    let k = perm(&mut g, P0, "Korvold, Fae-Cursed King");
    g.player_mut(P0).treasures = 1;
    let forest = lands(&mut g, P0, "Forest", 1, false)[0];
    let log = person(&mut g, |r| if r.prompt.starts_with("Korvold") { Some(text(r, "Forest")) } else { None });
    attack(&mut g, k, P1);
    assert!(asked(&log, "Korvold: sacrifice which other permanent"), "{}", dump(&log));
    assert!(!g.perm(forest).on_bf && !g.player(P0).lands.contains(&forest), "{}", dump(&log));
    assert_eq!(g.player(P0).treasures, 1, "the Treasure stayed");
    let log = person(&mut g, |r| if r.prompt.starts_with("Korvold") { Some(text(r, "Treasure")) } else { None });
    attack(&mut g, k, P1);
    assert!(asked(&log, "Korvold"), "{}", dump(&log));
    assert_eq!(g.player(P0).treasures, 0);
}

// ------------------------------------------------------------------ Kura, the Boundless Sky
#[test]
fn kura_dies_the_person_chooses_the_mode_and_the_lands() {
    let mut g = table(&["urdragon", "seph"]);
    let kura = perm(&mut g, P0, "Kura, the Boundless Sky");
    let lands_in_lib = |g: &Game| g.player(P0).library.iter().filter(|&&c| g.db.get(c).land).count();
    let hand_lands = |g: &Game| g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count();
    let (lib0, hand0) = (lands_in_lib(&g), hand_lands(&g));
    assert!(lib0 >= 3);
    let log = person(&mut g, |r| {
        if r.prompt.contains("choose one") {
            Some(text(r, "search your library"))
        } else if r.prompt.contains("search for a land card (2 of 3)") {
            Some(text(r, "find nothing more")) // stop after one
        } else {
            None
        }
    });
    zones::die(&mut g, kura, "destroy").unwrap();
    assert!(asked(&log, "Kura, the Boundless Sky: choose one"), "{}", dump(&log));
    assert_eq!(hand_lands(&g), hand0 + 1, "up to three: the person took one: {}", dump(&log));
    assert_eq!(lands_in_lib(&g), lib0 - 1);
}

#[test]
fn kura_dies_the_person_may_take_the_spirit_when_the_ai_would_fetch_lands() {
    let mut g = table(&["urdragon", "seph"]);
    let kura = perm(&mut g, P0, "Kura, the Boundless Sky");
    lands(&mut g, P0, "Forest", 3, false);
    let log = person(&mut g, |r| if r.prompt.contains("choose one") { Some(text(r, "Spirit")) } else { None });
    zones::die(&mut g, kura, "destroy").unwrap();
    assert!(asked(&log, "Kura, the Boundless Sky: choose one"), "{}", dump(&log));
    let spirit = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a Spirit token");
    assert_eq!(sim_core::engine::values::epow(&g, spirit), 3, "X = the three lands");
}

// ------------------------------------------------------------------ Mishra's Self-Replicator
#[test]
fn self_replicator_asks_before_paying_one_for_a_copy() {
    for say in [true, false] {
        let mut g = table(&["karn", "seph"]);
        perm(&mut g, P0, "Mishra's Self-Replicator");
        lands(&mut g, P0, "Mountain", 2, false);
        let c = take(&mut g, P0, "Sol Ring");
        let log = person(&mut g, move |r| {
            if r.prompt.starts_with("Mishra's Self-Replicator") {
                Some(Answer::Pick(if say { 0 } else { 1 }))
            } else {
                None
            }
        });
        cast::on_cast(&mut g, P0, c).unwrap();
        assert!(asked(&log, "Mishra's Self-Replicator: pay {1}"), "{}", dump(&log));
        let copies = perms_named(&g, P0, "Mishra's Self-Replicator").len();
        assert_eq!(copies, 1 + usize::from(say), "say yes = {say}: {}", dump(&log));
    }
}

// ------------------------------------------------------------------ Myr Battlesphere
#[test]
fn battlesphere_taps_the_number_of_myr_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let bs = perm(&mut g, P0, "Myr Battlesphere");
    let myr: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| m != bs).collect();
    assert_eq!(myr.len(), 4, "the four Myr tokens");
    let log = person(&mut g, |r| if r.prompt.starts_with("Myr Battlesphere") { Some(text(r, "X = 2")) } else { None });
    let life = g.player(P1).life;
    attack(&mut g, bs, P1);
    assert!(asked(&log, "Myr Battlesphere: tap how many"), "{}", dump(&log));
    assert_eq!(myr.iter().filter(|&&m| g.perm(m).tapped).count(), 2, "{}", dump(&log));
    assert_eq!(g.player(P1).life, life - 2);
}

#[test]
fn battlesphere_x_zero_taps_nothing() {
    let mut g = table(&["karn", "seph"]);
    let bs = perm(&mut g, P0, "Myr Battlesphere");
    let log = person(&mut g, |r| if r.prompt.starts_with("Myr Battlesphere") { Some(text(r, "X = 0")) } else { None });
    let life = g.player(P1).life;
    attack(&mut g, bs, P1);
    assert!(asked(&log, "Myr Battlesphere"), "{}", dump(&log));
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).tapped));
    assert_eq!(g.player(P1).life, life);
}

// ------------------------------------------------------------------ Portal to Phyrexia
#[test]
fn portal_upkeep_returns_the_creature_card_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    perm(&mut g, P0, "Portal to Phyrexia");
    let mine = take(&mut g, P0, "Llanowar Elves");
    let theirs = take(&mut g, P1, "Esper Sentinel");
    g.player_mut(P0).gy.push(mine);
    g.player_mut(P1).gy.push(theirs);
    let log =
        person(
            &mut g,
            |r| {
                if r.prompt.starts_with("Portal to Phyrexia") { Some(text(r, "Esper Sentinel")) } else { None }
            },
        );
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(asked(&log, "Portal to Phyrexia: put which creature card"), "{}", dump(&log));
    assert!(has_perm(&g, P0, "Esper Sentinel"), "the person's pick, from an opponent's graveyard: {}", dump(&log));
    assert!(g.player(P0).gy.contains(&mine));
}

// ------------------------------------------------------------------ Scarlet Witch
#[test]
fn scarlet_witch_free_spell_is_aimed_where_the_person_says() {
    let mut g = table(&["kefka", "seph"]);
    let witch = perm(&mut g, P0, "Scarlet Witch, Chaotic Avenger");
    let big = perm(&mut g, P1, "Birds of Paradise");
    let small = perm(&mut g, P1, "Llanowar Elves");
    let terror = take(&mut g, P0, "Terror");
    let island = take(&mut g, P0, "Island");
    g.player_mut(P0).library.extend([island, terror]);
    let log = person(&mut g, |r| {
        if r.prompt.starts_with("Terror: choose a target") {
            Some(text(r, "Llanowar Elves"))
        } else if r.prompt.starts_with("Scarlet Witch") {
            Some(text(r, "Terror"))
        } else {
            None
        }
    });
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a: witch, d: P1, dmg: 3 }).unwrap();
    assert!(asked(&log, "Terror: choose a target"), "{}", dump(&log));
    assert!(!g.perm(small).on_bf, "the person's target died: {}", dump(&log));
    assert!(g.perm(big).on_bf, "not the AI's pick: {}", dump(&log));
}

// ------------------------------------------------------------------ Susurian Voidborn
#[test]
fn voidborn_drains_the_opponent_the_person_picks() {
    let mut g = table(&["kefka", "seph", "cloud"]);
    perm(&mut g, P0, "Susurian Voidborn");
    let fodder = token(&mut g, P0, 1);
    g.player_mut(P1).life = 20; // the AI would pick the lowest life
    let log = person(&mut g, |r| if r.prompt.starts_with("Susurian Voidborn") { Some(Answer::Pick(1)) } else { None });
    zones::die(&mut g, fodder, "destroy").unwrap();
    assert!(asked(&log, "Susurian Voidborn: target which opponent"), "{}", dump(&log));
    assert_eq!((g.player(P1).life, g.player(P2).life), (20, 39), "{}", dump(&log));
    assert_eq!(g.player(P0).life, 41);
}

// ------------------------------------------------------------------ Sylvan Library
#[test]
fn sylvan_library_the_person_chooses_two_cards_and_pays_or_puts_back() {
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Sylvan Library");
    g.active = Some(P0);
    let hand0 = g.player(P0).hand.len();
    let log = person(&mut g, |r| {
        if r.prompt.contains("pay 4 life or put") {
            Some(Answer::Pick(0))
        } else if r.prompt.starts_with("Sylvan Library:") && r.choices.iter().any(|c| c.contains("pay 4 life")) {
            Some(text(r, "pay 4 life"))
        } else {
            None
        }
    });
    zones::draw(&mut g, P0, 1, true).unwrap();
    assert!(asked(&log, "Sylvan Library: draw two additional cards"), "{}", dump(&log));
    assert!(asked(&log, "choose two cards in your hand drawn this turn"), "{}", dump(&log));
    // both chosen cards were paid for (every per-card answer is "pay 4 life"): 8 life, all three drawn cards kept
    assert_eq!(g.player(P0).life, 32, "{}", dump(&log));
    assert_eq!(g.player(P0).hand.len(), hand0 + 3);
}

#[test]
fn sylvan_library_put_back_goes_on_top_and_declining_draws_nothing_extra() {
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Sylvan Library");
    g.active = Some(P0);
    let hand0 = g.player(P0).hand.len();
    let log = person(&mut g, |r| {
        if r.prompt.starts_with("Sylvan Library:") && r.choices.iter().any(|c| c.contains("put it on top")) {
            Some(text(r, "put it on top"))
        } else {
            None
        }
    });
    zones::draw(&mut g, P0, 1, true).unwrap();
    assert_eq!(g.player(P0).life, 40, "{}", dump(&log));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "two of the three cards went back: {}", dump(&log));
    // a second turn's draw with "no" to the question draws just the one
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Sylvan Library");
    g.active = Some(P0);
    let hand0 = g.player(P0).hand.len();
    let log =
        person(&mut g, |r| if r.prompt.starts_with("Sylvan Library: draw two") { Some(text(r, "no")) } else { None });
    zones::draw(&mut g, P0, 1, true).unwrap();
    assert!(asked(&log, "Sylvan Library: draw two additional cards"), "{}", dump(&log));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
}

// ------------------------------------------------------------------ The Ozolith
#[test]
fn ozolith_moves_its_counters_onto_the_creature_the_person_picks() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "The Ozolith");
    let a = token(&mut g, P0, 1);
    let b = token(&mut g, P0, 2);
    g.player_mut(P0).ozolith_counters = 3;
    let log = person(&mut g, |r| if r.prompt.starts_with("The Ozolith") { Some(text(r, "2/2")) } else { None });
    combat::combat(&mut g, P0).unwrap();
    assert!(asked(&log, "The Ozolith: move its 3 counter"), "{}", dump(&log));
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 3), "{}", dump(&log));
    assert_eq!(g.player(P0).ozolith_counters, 0);
}

#[test]
fn ozolith_may_keep_its_counters() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "The Ozolith");
    let a = token(&mut g, P0, 1);
    g.player_mut(P0).ozolith_counters = 3;
    let log = person(&mut g, |r| if r.prompt.starts_with("The Ozolith") { Some(text(r, "don't move")) } else { None });
    combat::combat(&mut g, P0).unwrap();
    assert!(asked(&log, "The Ozolith"), "{}", dump(&log));
    assert_eq!(g.perm(a).plus, 0);
    assert_eq!(g.player(P0).ozolith_counters, 3);
}

// ------------------------------------------------------------------ Vision, Synthezoid Avenger
#[test]
fn vision_mode_is_the_persons_choice() {
    for (word, phased, plus) in [("phases out", true, 0), ("+1/+1 counter", false, 1)] {
        let mut g = table(&["sauron", "seph"]);
        let v = perm(&mut g, P0, "Vision, Synthezoid Avenger");
        let c = take(&mut g, P1, "Sol Ring");
        let log =
            person(
                &mut g,
                move |r| {
                    if r.prompt.starts_with("Vision, Synthezoid Avenger") { Some(text(r, word)) } else { None }
                },
            );
        cast::on_cast(&mut g, P1, c).unwrap(); // an opponent's spell on seat 0's turn
        assert!(asked(&log, "Vision, Synthezoid Avenger: choose one"), "{}", dump(&log));
        assert_eq!((g.perm(v).phased, g.perm(v).plus), (phased, plus), "{word}: {}", dump(&log));
    }
}

// ------------------------------------------------------------------ cards found to ask already
#[test]
fn solemn_simulacrum_dies_trigger_asks_before_drawing() {
    for say in [true, false] {
        let mut g = table(&["cloud", "seph"]);
        let s = perm(&mut g, P0, "Solemn Simulacrum");
        let hand0 = g.player(P0).hand.len();
        let log = person(&mut g, move |r| {
            if r.prompt.starts_with("Solemn Simulacrum") { Some(Answer::Pick(if say { 0 } else { 1 })) } else { None }
        });
        zones::die(&mut g, s, "destroy").unwrap();
        assert!(asked(&log, "Solemn Simulacrum: draw"), "{}", dump(&log));
        assert_eq!(g.player(P0).hand.len(), hand0 + usize::from(say), "{}", dump(&log));
    }
}

#[test]
fn niv_mizzet_draw_trigger_asks_for_its_target() {
    for card in ["Niv-Mizzet, the Firemind", "Niv-Mizzet, Parun"] {
        let mut g = table(&["kefka", "seph"]);
        perm(&mut g, P0, card);
        let log = person(&mut g, |r| if &*r.kind == "target" { Some(Answer::Pick(0)) } else { None });
        zones::draw(&mut g, P0, 1, false).unwrap();
        assert!(log.lock().unwrap().iter().any(|l| l.starts_with("target:")), "{card}: {}", dump(&log));
    }
}
