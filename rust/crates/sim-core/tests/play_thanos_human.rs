//! Practice mode: JD's Thanos deck played by a person. Thassa's Oracle with Demonic Consultation / Tainted Pact (cast in
//! response to the Oracle's trigger, or first), Lion's Eye Diamond, Brain Freeze and Grapeshot with their storm
//! targets, the Underworld Breach loop (by hand, and as the table shortcut) and Ad Nauseam.

mod play_common;
use play_common::*;
use sim_core::human::human::human_main;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn tbl() -> Game {
    let mut g = table(&["scion", "sauron"]);
    g.log = Some(vec![]);
    g
}

fn prompt_has(h: &ScriptHuman, word: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(word))
}

/// seat a person who answers by the request: `f` returns the answer; a main-phase `priority` request with nothing
/// else to do is answered by passing
fn person(g: &mut Game, f: impl FnMut(&Request) -> Answer + Send + 'static) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(f));
    g.humans.add(P0, h.clone());
    h
}

fn pass_a() -> Answer {
    Answer::Action(json!({"do": "pass"}))
}

fn cast_a(i: usize) -> Answer {
    Answer::Action(json!({"do": "cast", "card": i}))
}

fn is_priority(r: &Request) -> bool {
    r.kind == "priority"
}

fn lib_to(g: &mut Game, p: PlayerId, top: &[&str]) {
    // `top` listed from the top of the library down: those cards are moved to the top in that order
    let ids: Vec<_> = top.iter().map(|n| card(g, n)).collect();
    let mut once = std::collections::HashSet::new();
    for &c in &ids {
        if once.insert(c) {
            take(g, p, &g.db.get(c).name.to_string()); // the deck's own copy leaves the library (only once per name)
        }
    }
    for &c in ids.iter().rev() {
        g.player_mut(p).library.push(c);
    }
}

#[test]
fn primeval_titan_person_can_find_zero_one_or_two_lands_on_enter_and_attack() {
    for attack in [false, true] {
        for count in 0..=2 {
            let mut g = table(&["thanos", "urza-mono-blue-artifacts"]);
            let titan = perm(&mut g, P0, "Primeval Titan");
            let forest = card(&g, "Forest");
            let tower = card(&g, "Command Tower");
            let spell = card(&g, "Sol Ring");
            // Repeated basics remain legal for the second pick; the shared multi-card search helper
            // deliberately restricts searches to distinct names, so Titan uses the single-choice helper.
            g.player_mut(P0).library = vec![forest, forest, tower, spell];
            let before = g.player(P0).lands.len();
            let mut picked = 0;
            let h = person(&mut g, move |r| {
                if r.kind == "search" {
                    assert!(r.prompt.starts_with("Primeval Titan:"));
                    assert!(!r.choices.iter().any(|c| c.contains("Sol Ring")));
                    if picked < count {
                        picked += 1;
                        Answer::Pick(r.choices.iter().position(|c| c.starts_with("Forest")).unwrap())
                    } else {
                        Answer::Decline
                    }
                } else if is_priority(r) {
                    pass_a()
                } else {
                    Answer::Decline
                }
            });
            let imp = g.registry.get(card(&g, "Primeval Titan")).unwrap();
            let (etb, atk) = (imp.etb.unwrap(), imp.attack.unwrap());
            if attack {
                atk(&mut g, titan, P0, &[titan], P1).unwrap();
            } else {
                etb(&mut g, titan, P0, titan).unwrap();
            }
            assert_eq!(g.player(P0).lands.len(), before + count);
            assert_eq!(g.player(P0).library.iter().filter(|&&c| c == forest).count(), 2 - count);
            assert!(g.player(P0).library.contains(&tower) && g.player(P0).library.contains(&spell));
            for &l in &g.player(P0).lands[before..] {
                assert_eq!(g.perm(l).card(), forest);
                assert!(g.perm(l).tapped);
            }
            assert_eq!(h.asked().iter().filter(|r| r.kind == "search").count(), if count == 2 { 2 } else { count + 1 });
        }
    }
}

#[test]
fn primeval_titan_probe_does_not_ask_search_shuffle_or_put_lands() {
    let mut g = table(&["thanos", "urza-mono-blue-artifacts"]);
    let titan = perm(&mut g, P0, "Primeval Titan");
    let forest = card(&g, "Forest");
    g.player_mut(P0).library = vec![forest, forest];
    let before = g.clone();
    let h = person(&mut g, |_| panic!("no question before the trigger window"));
    let imp = g.registry.get(card(&g, "Primeval Titan")).unwrap();
    let (etb, atk) = (imp.etb.unwrap(), imp.attack.unwrap());
    g.trig_probe = true;
    assert!(matches!(etb(&mut g, titan, P0, titan), Err(sim_core::flow::Stop::Probe(_))));
    g.trig_probe = true;
    assert!(matches!(atk(&mut g, titan, P0, &[titan], P1), Err(sim_core::flow::Stop::Probe(_))));
    assert!(h.asked().is_empty());
    assert_eq!(g.rng, before.rng);
    assert_eq!(g.player(P0).library, before.player(P0).library);
    assert_eq!(g.player(P0).lands, before.player(P0).lands);
}

#[test]
fn sneak_attack_person_may_put_any_creature_or_none_even_on_an_opponents_turn() {
    for put in [false, true] {
        let mut g = tbl();
        let src = perm(&mut g, P0, "Sneak Attack");
        hand(&mut g, P0, &["Llanowar Elves"]);
        pool(&mut g, P0, &[('R', 1)]);
        g.active = Some(P1);
        g.step = sim_core::state::Step::End;
        let h = person(&mut g, move |r| {
            if r.prompt.ends_with(": which ability?") {
                Answer::Pick(0)
            } else if r.prompt.starts_with("Sneak Attack: put which") {
                Answer::Pick(if put { 0 } else { r.choices.len() - 1 })
            } else if is_priority(r) {
                pass_a()
            } else {
                Answer::Decline
            }
        });
        let abilities = sim_core::human::cards::abilities(&g, P0, src).unwrap();
        assert!(abilities.iter().any(|a| a.label.contains("{R}")));
        let i = index_of_sneak(&g, src);
        sim_core::human::human::apply(&mut g, P0, &json!({"do":"use", "perm":i})).unwrap();
        assert!(prompt_has(&h, "Sneak Attack: put which"));
        assert_eq!(pool_total(&g, P0), 0, "paid even when choosing none");
        let elves = perms_named(&g, P0, "Llanowar Elves");
        assert_eq!(elves.len(), usize::from(put));
        if let Some(&m) = elves.first() {
            assert!(sim_core::engine::combat::has_haste(&g, m));
            sim_core::engine::turn::end_step(&mut g, P1).unwrap();
            assert!(!g.perm(m).on_bf);
        }
    }
}

fn index_of_sneak(g: &Game, src: PermId) -> usize {
    g.player(P0).perms.iter().position(|&m| m == src).unwrap()
}

#[test]
fn sneak_attack_with_an_empty_hand_is_a_legal_activation() {
    let mut g = tbl();
    let src = perm(&mut g, P0, "Sneak Attack");
    g.player_mut(P0).hand.clear();
    pool(&mut g, P0, &[('R', 1)]);
    person(&mut g, |r| {
        if is_priority(r) {
            pass_a()
        } else if r.prompt.ends_with(": which ability?") {
            Answer::Pick(0)
        } else {
            Answer::Decline
        }
    });
    let i = index_of_sneak(&g, src);
    sim_core::human::human::apply(&mut g, P0, &json!({"do":"use", "perm":i})).unwrap();
    assert_eq!(pool_total(&g, P0), 0);
    assert!(g.player(P0).hand.is_empty());
}

#[test]
fn rings_copies_sneak_attack_and_each_resolution_chooses_a_creature() {
    let mut g = tbl();
    let src = perm(&mut g, P0, "Sneak Attack");
    perm(&mut g, P0, "Rings of Brighthearth");
    hand(&mut g, P0, &["Llanowar Elves", "Birds of Paradise"]);
    pool(&mut g, P0, &[('R', 1), ('C', 2)]);
    let h = person(&mut g, |r| {
        if is_priority(r) {
            pass_a()
        } else if r.prompt.starts_with("Rings of Brighthearth: pay") {
            Answer::Yes
        } else if r.prompt.starts_with("Sneak Attack:") {
            Answer::Pick(0)
        } else {
            Answer::Decline
        }
    });
    let i = index_of_sneak(&g, src);
    assert_eq!(sim_core::human::human::apply(&mut g, P0, &json!({"do":"use", "perm":i})).unwrap(), None);
    assert!(has_perm(&g, P0, "Llanowar Elves") && has_perm(&g, P0, "Birds of Paradise"));
    assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Sneak Attack: put which")).count(), 2);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn sneak_attack_destroyed_in_response_still_puts_in_the_creature() {
    let mut g = tbl();
    let src = perm(&mut g, P0, "Sneak Attack");
    hand(&mut g, P0, &["Disenchant", "Llanowar Elves"]);
    pool(&mut g, P0, &[('R', 1), ('W', 2)]);
    let mut responded = false;
    let h = person(&mut g, move |r| {
        if is_priority(r) {
            if !responded && r.prompt.contains("Sneak Attack") {
                responded = true;
                cast_a(0)
            } else {
                pass_a()
            }
        } else if r.prompt.ends_with(": which ability?") || r.prompt.starts_with("Sneak Attack: put which") {
            Answer::Pick(0)
        } else if r.choices.iter().any(|s| s.contains("Sneak Attack")) {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Sneak Attack")).unwrap())
        } else {
            Answer::Decline
        }
    });
    h.set_autopass("all");
    let i = index_of_sneak(&g, src);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do":"use", "perm":i})).unwrap();
    assert_eq!(why, None);
    assert!(!g.perm(src).on_bf, "Disenchant resolved during the activation: {:?}", h.asked());
    let elves = perms_named(&g, P0, "Llanowar Elves");
    assert_eq!(elves.len(), 1);
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(!g.perm(elves[0]).on_bf, "the delayed effect survives its source");
}

#[test]
fn sneaking_during_an_end_step_trigger_waits_for_the_next_end_step() {
    let mut g = tbl();
    let src = perm(&mut g, P0, "Sneak Attack");
    perm(&mut g, P0, "Teleportation Circle");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Llanowar Elves"]);
    pool(&mut g, P0, &[('R', 1)]);
    let i = index_of_sneak(&g, src);
    let mut used = false;
    let h = person(&mut g, move |r| {
        if is_priority(r) {
            if !used && r.prompt.contains("Teleportation Circle") {
                used = true;
                Answer::Action(json!({"do":"use", "perm":i}))
            } else {
                pass_a()
            }
        } else if r.prompt.ends_with(": which ability?") || r.prompt.starts_with("Sneak Attack: put which") {
            Answer::Pick(0)
        } else {
            Answer::Decline
        }
    });
    h.set_autopass("all");
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    let elves = perms_named(&g, P0, "Llanowar Elves");
    assert_eq!(elves.len(), 1, "the creature put in after this step began survives");
    sim_core::engine::turn::end_step(&mut g, P1).unwrap();
    assert!(!g.perm(elves[0]).on_bf);
}

// ------------------------------------------------------------------ Thassa's Oracle + Consultation / Pact
#[test]
fn oracle_then_consultation_in_response_to_its_trigger_wins() {
    let mut g = tbl();
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    pool(&mut g, P0, &[('U', 2), ('B', 1)]);
    let (cast_oracle, cast_cons) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let h = person(&mut g, move |r| {
        if r.prompt.contains("name a card") {
            Answer::Pick(0) // a card that is not in the library
        } else if is_priority(r) && r.prompt.contains("Thassa's Oracle") {
            // the trigger is on the stack and it's mine: Consultation (the only card left in hand)
            if cast_cons.fetch_add(1, Ordering::SeqCst) == 0 { cast_a(0) } else { pass_a() }
        } else if is_priority(r) {
            if cast_oracle.fetch_add(1, Ordering::SeqCst) == 0 { cast_a(0) } else { pass_a() }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "Your trigger: Thassa's Oracle"), "you got priority on the Oracle's trigger");
    assert!(g.player(P0).library.is_empty(), "Consultation exiled the whole library");
    assert!(g.player(P0).alive && !g.player(P1).alive, "and the trigger resolved: you win");
}

#[test]
fn consultation_first_then_oracle_wins() {
    let mut g = tbl();
    hand(&mut g, P0, &["Demonic Consultation", "Thassa's Oracle"]);
    pool(&mut g, P0, &[('U', 2), ('B', 1)]);
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.contains("name a card") {
            Answer::Pick(0)
        } else if is_priority(r) {
            match n.fetch_add(1, Ordering::SeqCst) {
                0 => cast_a(0), // Consultation
                1 => cast_a(0), // then the Oracle (now the only card)
                _ => pass_a(),
            }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(!prompt_has(&h, "Your trigger"), "no Consultation left in hand: the trigger just resolves");
    assert!(g.player(P0).library.is_empty());
    assert!(g.player(P0).alive && !g.player(P1).alive, "Oracle on an empty library wins");
}

#[test]
fn oracle_with_a_full_library_asks_which_card_goes_on_top() {
    let mut g = tbl();
    lib_to(&mut g, P0, &["Sol Ring", "Mana Vault", "Chrome Mox"]);
    let h = person(&mut g, |r| {
        if r.prompt.starts_with("Thassa's Oracle: the top 2") {
            let i = r.choices.iter().position(|c| c.starts_with("Mana Vault")).expect("Mana Vault among the top two");
            Answer::Pick(i)
        } else {
            Answer::Pick(0)
        }
    });
    let o = perm(&mut g, P0, "Thassa's Oracle");
    assert!(g.perm(o).on_bf);
    let asked = h.asked();
    let q = asked.iter().find(|r| r.prompt.starts_with("Thassa's Oracle: the top 2")).expect("you were asked");
    assert_eq!(q.choices.len(), 3, "the two cards and 'none of them on top'");
    assert!(q.choices[0].starts_with("Sol Ring") && q.choices[1].starts_with("Mana Vault"));
    let lib = &g.player(P0).library;
    assert_eq!(lib.last().map(|&c| &*g.db.get(c).name), Some("Mana Vault"), "your pick is on top");
    assert_eq!(&*g.db.get(lib[0]).name, "Sol Ring", "and the other went to the bottom");
    assert!(g.player(P0).alive && g.player(P1).alive, "no win: the library is not empty");
}

#[test]
fn oracle_may_put_none_of_them_on_top() {
    let mut g = tbl();
    lib_to(&mut g, P0, &["Sol Ring", "Mana Vault", "Chrome Mox"]);
    let len = g.player(P0).library.len() - 1; // the Oracle itself leaves the library
    person(&mut g, |r| Answer::Pick(r.choices.len() - 1)); // the last choice: none of them
    perm(&mut g, P0, "Thassa's Oracle");
    let lib = &g.player(P0).library;
    assert_eq!(lib.len(), len);
    assert!(lib.iter().rev().take(1).all(|&c| &*g.db.get(c).name != "Sol Ring" && &*g.db.get(c).name != "Mana Vault"));
    let bottom: Vec<&str> = lib[..2].iter().map(|&c| &*g.db.get(c).name).collect();
    assert!(bottom.contains(&"Sol Ring") && bottom.contains(&"Mana Vault"), "both went to the bottom: {bottom:?}");
}

#[test]
fn consultation_names_a_card_exiles_six_and_takes_it() {
    let mut g = tbl();
    // the named card is the tenth from the top: six exiled, three more revealed and exiled, then it
    lib_to(
        &mut g,
        P0,
        &["Island", "Island", "Island", "Island", "Island", "Island", "Island", "Island", "Island", "Sol Ring"],
    );
    hand(&mut g, P0, &["Demonic Consultation"]);
    let len = g.player(P0).library.len();
    pool(&mut g, P0, &[('B', 1)]);
    let n = AtomicUsize::new(0);
    let h2 = person(&mut g, move |r| {
        if r.prompt.contains("name a card") {
            Answer::Pick(r.choices.iter().position(|c| c == "Sol Ring").expect("Sol Ring is named in the list"))
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            cast_a(0)
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(h2.asked().iter().any(|r| r.prompt.starts_with("Demonic Consultation: name a card")));
    let pl = g.player(P0);
    assert!(pl.hand.iter().any(|&c| &*g.db.get(c).name == "Sol Ring"), "the named card is in hand\n{}", dump(&g));
    assert_eq!(pl.library.len(), len - 10, "the top six and the three after them were exiled, and Sol Ring taken");
    assert_eq!(pl.exile.len(), 9);
}

#[test]
fn consultation_naming_a_card_in_the_top_six_exiles_it_and_the_whole_library() {
    let mut g = tbl();
    lib_to(&mut g, P0, &["Sol Ring"]);
    hand(&mut g, P0, &["Demonic Consultation"]);
    pool(&mut g, P0, &[('B', 1)]);
    let n = AtomicUsize::new(0);
    person(&mut g, move |r| {
        if r.prompt.contains("name a card") {
            Answer::Pick(r.choices.iter().position(|c| c == "Sol Ring").unwrap())
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            cast_a(0)
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(g.player(P0).library.is_empty(), "no later copy: everything is exiled");
    assert!(!g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Sol Ring"), "the exiled six are not found");
}

#[test]
fn tainted_pact_take_the_third_card() {
    let mut g = tbl();
    lib_to(&mut g, P0, &["Sol Ring", "Mana Vault", "Chrome Mox"]);
    hand(&mut g, P0, &["Tainted Pact"]);
    let len = g.player(P0).library.len();
    pool(&mut g, P0, &[('B', 1), ('U', 1)]);
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Tainted Pact:") {
            // leave the first two, take the third
            if r.prompt.contains("Chrome Mox") { Answer::Pick(0) } else { Answer::Pick(1) }
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            cast_a(0)
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Tainted Pact:")).count(), 3);
    let pl = g.player(P0);
    assert!(pl.hand.iter().any(|&c| &*g.db.get(c).name == "Chrome Mox"), "the third card is in hand");
    assert_eq!(pl.exile.len(), 2, "the first two stay exiled");
    assert_eq!(pl.library.len(), len - 3);
}

#[test]
fn tainted_pact_declining_every_card_stops_on_a_duplicate_name() {
    let mut g = tbl();
    // the third card from the top has the second card's name: the Pact ends there, library left below
    lib_to(&mut g, P0, &["Sol Ring", "Island", "Island"]);
    hand(&mut g, P0, &["Tainted Pact"]);
    let len = g.player(P0).library.len();
    pool(&mut g, P0, &[('B', 1), ('U', 1)]);
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Tainted Pact:") {
            Answer::Pick(2) // leave it and every card after it exiled
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            cast_a(0)
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Tainted Pact:")).count(), 1, "asked once");
    let pl = g.player(P0);
    assert_eq!(pl.library.len(), len - 3, "Sol Ring, Island, Island: the second Island ends it\n{}", dump(&g));
    assert!(pl.hand.is_empty());
}

#[test]
fn tainted_pact_in_response_to_the_oracle_trigger_wins() {
    let mut g = tbl();
    // a library of singletons only (no two cards share a name), so declining every card exiles all of it
    let mut seen = std::collections::HashSet::new();
    let lib: Vec<_> = g.player(P0).library.clone();
    let keep: Vec<_> = lib.into_iter().filter(|&c| seen.insert(g.db.get(c).name.to_string())).collect();
    g.player_mut(P0).library = keep;
    hand(&mut g, P0, &["Thassa's Oracle", "Tainted Pact"]);
    pool(&mut g, P0, &[('U', 2), ('B', 1), ('R', 1)]);
    let (a, b) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Tainted Pact:") {
            Answer::Pick(2)
        } else if is_priority(r) && r.prompt.contains("Thassa's Oracle") {
            if b.fetch_add(1, Ordering::SeqCst) == 0 { cast_a(0) } else { pass_a() }
        } else if is_priority(r) {
            if a.fetch_add(1, Ordering::SeqCst) == 0 { cast_a(0) } else { pass_a() }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "Your trigger: Thassa's Oracle"));
    assert!(g.player(P0).library.is_empty());
    assert!(!g.player(P1).alive, "the trigger saw an empty library: you win");
}

// ------------------------------------------------------------------ Ad Nauseam
#[test]
fn ad_nauseam_asks_after_every_card() {
    let mut g = tbl();
    lib_to(&mut g, P0, &["Sol Ring", "Sylvan Library", "Island", "Mana Vault"]);
    hand(&mut g, P0, &["Ad Nauseam"]);
    pool(&mut g, P0, &[('B', 2), ('C', 3)]);
    let life0 = g.player(P0).life;
    let (cast, revealed) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Ad Nauseam:") {
            // reveal three cards, then stop
            if revealed.fetch_add(1, Ordering::SeqCst) < 3 { Answer::Pick(0) } else { Answer::Pick(1) }
        } else if is_priority(r) && cast.fetch_add(1, Ordering::SeqCst) == 0 {
            cast_a(0)
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    let asks: Vec<_> = h.asked().into_iter().filter(|r| r.prompt.starts_with("Ad Nauseam:")).collect();
    assert_eq!(asks.len(), 4, "asked before each card and once more to stop");
    assert!(asks[0].prompt.contains("40 life") && asks[0].prompt.contains("nothing yet"));
    assert!(asks[2].prompt.contains("Sol Ring (1)") && asks[2].prompt.contains("Sylvan Library (2)"));
    assert_eq!(g.player(P0).life, life0 - 3, "Sol Ring 1 + Sylvan Library 2 + Island 0");
    let names: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert!(names.contains(&"Island".to_string()) && !names.contains(&"Mana Vault".to_string()), "{names:?}");
}

// ------------------------------------------------------------------ Lion's Eye Diamond
#[test]
fn lion_s_eye_diamond_discards_the_hand_and_adds_three_of_one_colour() {
    let mut g = tbl();
    let led = perm(&mut g, P0, "Lion's Eye Diamond");
    hand(&mut g, P0, &["Brain Freeze", "Sol Ring"]);
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.contains("which ability") {
            Answer::Pick(0)
        } else if r.prompt.starts_with("Lion's Eye Diamond:") {
            Answer::Pick(1) // blue
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            let i = 0; // the permanents list: Lion's Eye Diamond is the first
            Answer::Action(json!({"do": "use", "perm": i}))
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "Lion's Eye Diamond: discard your hand and sacrifice it. Add which three mana?"));
    assert!(!g.perm(led).on_bf, "sacrificed");
    assert!(g.player(P0).hand.is_empty(), "hand discarded");
    let gy: Vec<String> = g.player(P0).gy.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert!(gy.contains(&"Brain Freeze".to_string()) && gy.contains(&"Lion's Eye Diamond".to_string()), "{gy:?}");
    assert!(pool_text(&g, P0).contains("{U}{U}{U}"), "UUU: {}", pool_text(&g, P0));
}

// ------------------------------------------------------------------ storm: Brain Freeze, Grapeshot
fn storm_game() -> (Game, Arc<AtomicUsize>) {
    let mut g = tbl();
    hand(&mut g, P0, &["Dark Ritual", "Brain Freeze"]);
    pool(&mut g, P0, &[('B', 1), ('U', 1), ('C', 1)]);
    (g, Arc::new(AtomicUsize::new(0)))
}

#[test]
fn brain_freeze_copies_all_at_the_same_target_by_choice() {
    let (mut g, n) = storm_game();
    let (me0, opp0) = (g.player(P0).library.len(), g.player(P1).library.len());
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Brain Freeze: choose a target") {
            Answer::Pick(1) // the other player
        } else if r.prompt.starts_with("Brain Freeze: storm made") {
            Answer::Pick(0) // all copies at the same target
        } else if is_priority(r) {
            match n.fetch_add(1, Ordering::SeqCst) {
                0 => cast_a(0), // Dark Ritual: one spell cast before Brain Freeze
                1 => cast_a(0), // Brain Freeze (now the only card)
                _ => pass_a(),
            }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "Storm: 1 copy will be made"));
    assert_eq!(g.player(P1).library.len(), opp0 - 6, "the spell and its copy: 3 each, all at the opponent");
    assert_eq!(g.player(P0).library.len(), me0, "you milled nothing");
}

#[test]
fn brain_freeze_each_copy_may_have_its_own_target() {
    let (mut g, n) = storm_game();
    let (me0, opp0) = (g.player(P0).library.len(), g.player(P1).library.len());
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Brain Freeze: choose a target") {
            Answer::Pick(0) // yourself
        } else if r.prompt.starts_with("Brain Freeze: storm made") {
            Answer::Pick(1) // a target for each copy
        } else if r.prompt.starts_with("Brain Freeze: target for copy 1 of 1") {
            Answer::Pick(1) // the opponent
        } else if is_priority(r) {
            match n.fetch_add(1, Ordering::SeqCst) {
                0 | 1 => cast_a(0),
                _ => pass_a(),
            }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "target for copy 1 of 1"));
    assert_eq!(g.player(P0).library.len(), me0 - 3, "the spell milled you");
    assert_eq!(g.player(P1).library.len(), opp0 - 3, "its copy milled the opponent");
}

#[test]
fn grapeshot_copies_ask_for_targets() {
    let mut g = tbl();
    hand(&mut g, P0, &["Dark Ritual", "Grapeshot"]);
    pool(&mut g, P0, &[('B', 1), ('R', 1), ('C', 1)]);
    let (life0, me0) = (g.player(P1).life, g.player(P0).life);
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.starts_with("Grapeshot: choose a target") {
            Answer::Pick(r.choices.iter().position(|c| c.contains("sauron") || c.contains("Sauron")).unwrap_or(1))
        } else if r.prompt.starts_with("Grapeshot: storm made") {
            Answer::Pick(1)
        } else if r.prompt.starts_with("Grapeshot: target for copy") {
            Answer::Pick(0) // yourself
        } else if is_priority(r) {
            match n.fetch_add(1, Ordering::SeqCst) {
                0 | 1 => cast_a(0),
                _ => pass_a(),
            }
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(prompt_has(&h, "Grapeshot: target for copy 1 of 1"));
    assert_eq!(g.player(P1).life, life0 - 1, "the spell hit the opponent");
    assert_eq!(g.player(P0).life, me0 - 1, "the copy hit you");
}

// ------------------------------------------------------------------ Underworld Breach + LED + Brain Freeze + Oracle
/// Breach and LED on the battlefield, Brain Freeze and Oracle in hand, fuel in the graveyard
fn breach_game() -> Game {
    let mut g = tbl();
    perm(&mut g, P0, "Underworld Breach");
    perm(&mut g, P0, "Lion's Eye Diamond");
    hand(&mut g, P0, &["Brain Freeze", "Thassa's Oracle"]);
    for n in ["Dark Ritual", "Cabal Ritual", "Rite of Flame", "Opt"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    // Breach and LED were cast this turn (storm count 2), as in the real line
    let cast = vec![card(&g, "Underworld Breach"), card(&g, "Lion's Eye Diamond")];
    g.player_mut(P0).turn_casts = Some((g.turn_stamp(), cast));
    g
}

#[test]
fn the_breach_loop_shortcut_is_offered_and_wins_with_the_oracle() {
    let mut g = breach_game();
    let breach = g.player(P0).perms.iter().position(|&m| g.perm(m).name == "Underworld Breach").unwrap();
    let n = AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.contains("which ability") {
            let i =
                r.choices.iter().position(|c| c.starts_with("loop Underworld Breach")).expect("the shortcut is listed");
            Answer::Pick(i)
        } else if is_priority(r) && n.fetch_add(1, Ordering::SeqCst) == 0 {
            Answer::Action(json!({"do": "use", "perm": breach}))
        } else if is_priority(r) {
            pass_a()
        } else {
            Answer::Pick(0)
        }
    });
    human_main(&mut g, P0, false).unwrap();
    assert!(g.player(P0).library.is_empty(), "the loop milled the library\n{}\n{}", dump(&g), zones_text(&g, P0));
    assert!(g.player(P0).alive && !g.player(P1).alive, "and the escaped Oracle won");
    assert!(
        h.asked().len() <= 4,
        "two answers started it: {:?}",
        h.asked().iter().map(|r| &r.prompt).collect::<Vec<_>>()
    );
}

#[test]
fn the_breach_loop_shortcut_is_not_offered_without_the_pieces() {
    let mut g = breach_game();
    // no Thassa's Oracle in hand or graveyard
    let o = g.player(P0).hand.iter().position(|&c| &*g.db.get(c).name == "Thassa's Oracle").unwrap();
    g.player_mut(P0).hand.remove(o);
    let breach = g.player(P0).perms.iter().position(|&m| g.perm(m).name == "Underworld Breach").unwrap();
    let m = g.player(P0).perms[breach];
    let abil = sim_core::human::human::abilities_of(&mut g, P0, m).unwrap();
    assert!(abil.iter().all(|a| !a.0.starts_with("loop Underworld Breach")), "not assembled: no shortcut");
}

/// the play-by-play so far, for assertion messages
fn dump(g: &Game) -> String {
    g.log.as_ref().map(|l| l.join("\n")).unwrap_or_default()
}

fn zones_text(g: &Game, p: PlayerId) -> String {
    let names =
        |v: &[sim_core::ids::CardId]| v.iter().map(|&c| g.db.get(c).name.to_string()).collect::<Vec<_>>().join(", ");
    let pl = g.player(p);
    format!(
        "hand: {}\ngy: {}\nexile: {}\nlibrary {}",
        names(&pl.hand),
        names(&pl.gy),
        names(&pl.exile),
        pl.library.len()
    )
}

#[test]
fn the_breach_loop_shortcut_is_not_offered_when_the_graveyard_would_run_dry() {
    // nothing cast this turn (storm 0) and only four other cards: the first Brain Freeze mills three, then the loop
    // would have to exile Thassa's Oracle to go on
    let mut g = breach_game();
    g.player_mut(P0).turn_casts = None;
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Underworld Breach").unwrap();
    let abil = sim_core::human::human::abilities_of(&mut g, P0, m).unwrap();
    assert!(abil.iter().all(|a| !a.0.starts_with("loop Underworld Breach")), "it would stall: no shortcut");
}

// ------------------------------------------------------------------ the whole line by hand
/// A person who plays the Breach loop with real actions only (no shortcut), counting every answer they give.
struct Bot {
    answers: AtomicUsize,
}

impl sim_core::control::Human for Bot {
    fn ask(&self, g: &Game, _seat: PlayerId, req: Request) -> Answer {
        if self.answers.fetch_add(1, Ordering::SeqCst) > 600 {
            return Answer::Cancel; // a runaway ends the game
        }
        by_hand(g, &req)
    }
    fn tell(&self, _seat: PlayerId, _kind: sim_core::sym::Sym, _text: &str) {}
}

fn gy_at(g: &Game, name: &str) -> Option<usize> {
    g.player(P0).gy.iter().position(|&c| &*g.db.get(c).name == name)
}

fn by_hand(g: &Game, r: &Request) -> Answer {
    let pl = g.player(P0);
    if r.prompt.contains("which ability") {
        return Answer::Pick(0);
    }
    if r.prompt.starts_with("Lion's Eye Diamond:") {
        return Answer::Pick(1); // three blue
    }
    if r.prompt.starts_with("Brain Freeze: choose a target") || r.prompt.starts_with("Brain Freeze: storm made") {
        return Answer::Pick(0); // yourself; every copy at the same target
    }
    if r.prompt.starts_with("Escape ") {
        // exile a card that isn't part of the line
        let i = r
            .choices
            .iter()
            .position(|c| !["Lion's Eye", "Brain Freeze", "Thassa's Oracle"].iter().any(|n| c.starts_with(n)));
        return Answer::Pick(i.unwrap_or(0));
    }
    if is_priority(r) {
        if let Some(i) = pl.perms.iter().position(|&m| g.perm(m).name == "Lion's Eye Diamond") {
            return Answer::Action(json!({"do": "use", "perm": i})); // crack it
        }
        let gy = |name: &str| gy_at(g, name).map(|i| Answer::Action(json!({"do": "cast", "zone": "gy", "card": i})));
        if pl.library.is_empty() && pl.pool.total() >= 2 {
            if let Some(a) = gy("Thassa's Oracle") {
                return a;
            }
        } else if pl.pool.total() >= 2 {
            if let Some(a) = gy("Brain Freeze") {
                return a;
            }
        }
        if let Some(a) = gy("Lion's Eye Diamond") {
            return a;
        }
        return pass_a();
    }
    Answer::Pick(0)
}

#[test]
fn the_whole_breach_line_can_be_looped_by_hand() {
    let mut g = breach_game();
    let bot = Arc::new(Bot { answers: AtomicUsize::new(0) });
    g.humans.add(P0, bot.clone());
    let lib0 = g.player(P0).library.len();
    let r = human_main(&mut g, P0, false);
    let clicks = bot.answers.load(Ordering::SeqCst);
    eprintln!("by hand: {clicks} answers to mill a library of {lib0} and win with the escaped Oracle");
    assert!(r.is_ok(), "{r:?}\n{}", dump(&g));
    assert!(g.player(P0).library.is_empty(), "{}\n{}", dump(&g), zones_text(&g, P0));
    assert!(g.player(P0).alive && !g.player(P1).alive, "the escaped Oracle won\n{}", dump(&g));
    assert!(clicks < 150, "{clicks} answers");
}
