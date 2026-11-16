//! The exile-from-the-battlefield event (`zones::exiled_from_bf`), Syr Vondam, Sunstar Exemplar, and Elspeth,
//! Knight-Errant (+1, +1, the -8 emblem). Each expectation follows the card's Oracle text (data/scryfall_cache.json);
//! the rules quoted are from the Comprehensive Rules (603.10a: leaves-the-battlefield abilities look back in time).

mod play_common;
use play_common::*;
use sim_core::engine::hooks::{fire_trigger, run_hook};
use sim_core::engine::{cast, removal, zones};
use sim_core::hooks::{Call, Event};
use sim_core::state::{Ctx, DataKey, Step, Val};

const BRAGO: &str = "brago-azorius-blink-control";

fn tbl() -> Game {
    let mut g = table(&[BRAGO, "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

/// the +1/+1 counters on Soulherder ("whenever a creature is exiled from the battlefield")
fn grown(g: &Game, herd: PermId) -> i32 {
    g.perm(herd).plus
}

fn end_step(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

// ------------------------------------------------------------------ the engine event
#[test]
fn soulherder_grows_when_swords_exiles_a_creature() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let big = perm(&mut g, P1, "Serra Angel");
    let swords = take(&mut g, P0, "Swords to Plowshares");
    cast::resolve(&mut g, P0, swords, &Ctx { target: Some(big), ..Ctx::default() }, "hand").unwrap();
    assert!(!g.perm(big).on_bf, "Swords exiled the Angel");
    assert_eq!(grown(&g, herd), 1, "a creature was exiled from the battlefield");
    // a token is exiled from the battlefield too (it then ceases to exist)
    let t = token(&mut g, P2, 2);
    removal::apply_removal(&mut g, Some(P0), t, "exile", None).unwrap();
    assert_eq!(grown(&g, herd), 2, "an exiled token counts");
}

#[test]
fn soulherder_ignores_a_noncreature_permanent_exiled() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let rock = perm(&mut g, P1, "Sol Ring");
    zones::exile_perm(&mut g, rock).unwrap();
    assert!(!g.perm(rock).on_bf);
    assert_eq!(grown(&g, herd), 0, "an artifact that is not a creature");
}

/// every blink path of the engine fires the event, witnessed by Soulherder's counter. A table: each case builds its own
/// game, runs its path once on a Wall of Omens (or the case's own subject) and says how many counters Soulherder
/// should have; the test lists every case that came out wrong.
#[test]
fn every_blink_path_fires_the_exile_event() {
    type Case = (&'static str, fn(&mut Game, PermId) -> i32);
    let cases: Vec<Case> = vec![
        ("Ephemerate (the AI's cast)", |g, _| {
            hand(g, P0, &["Ephemerate"]);
            lands(g, P0, "Plains", 1, false);
            let wall = perm(g, P0, "Wall of Omens");
            assert!(sim_core::impls::mine::ephemerate_cast(g, P0, wall, "value").unwrap());
            1
        }),
        ("Ephemerate's rebound", |g, _| {
            perm(g, P0, "Wall of Omens");
            let c = take(g, P0, "Ephemerate");
            g.player_mut(P0).exile.push(c);
            let f = g.registry.get(c).unwrap().rebound.unwrap();
            f(g, P0, c).unwrap();
            1
        }),
        ("Restoration Angel", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Restoration Angel");
            1
        }),
        ("Felidar Guardian", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Felidar Guardian");
            1
        }),
        ("Charming Prince", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Charming Prince");
            1
        }),
        ("Flickerwisp", |g, _| {
            let t = token(g, P1, 4);
            perm(g, P0, "Flickerwisp");
            assert!(!g.perm(t).on_bf);
            1
        }),
        ("Conjurer's Closet", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Conjurer's Closet");
            end_step(g, P0);
            2 // Soulherder's own end step blinks too
        }),
        ("Teleportation Circle", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Teleportation Circle");
            end_step(g, P0);
            2 // Soulherder's own end step blinks too
        }),
        ("The Mind Stone", |g, _| {
            perm(g, P0, "Wall of Omens");
            let ms = perm(g, P0, "The Mind Stone");
            g.perm_mut(ms).data.set(DataKey::Harnessed, Val::Bool(true));
            end_step(g, P0);
            2 // Soulherder's own end step blinks too
        }),
        ("Eerie Interlude", |g, _| {
            perm(g, P0, "Wall of Omens");
            let c = take(g, P0, "Eerie Interlude");
            cast::resolve(g, P0, c, &Ctx::default(), "hand").unwrap();
            1
        }),
        ("Cloudshift (the compiled blink)", |g, _| {
            let wall = perm(g, P0, "Wall of Omens");
            let c = take(g, P0, "Cloudshift");
            cast::resolve(g, P0, c, &Ctx { target: Some(wall), ..Ctx::default() }, "hand").unwrap();
            1
        }),
        ("Ghostly Flicker", |g, _| {
            perm(g, P0, "Wall of Omens");
            perm(g, P0, "Mulldrifter");
            let c = take(g, P0, "Ghostly Flicker");
            cast::resolve(g, P0, c, &Ctx::default(), "hand").unwrap();
            2
        }),
        ("Flicker (Galadriel's deck)", |g, _| {
            let wall = perm(g, P0, "Wall of Omens");
            let c = take(g, P0, "Flicker");
            cast::resolve(g, P0, c, &Ctx { target: Some(wall), ..Ctx::default() }, "hand").unwrap();
            1
        }),
        ("Displacer Kitten (the AI)", |g, _| {
            perm(g, P0, "Mulldrifter");
            let kitten = perm(g, P0, "Displacer Kitten");
            let spell = take(g, P0, "Swords to Plowshares");
            run_hook(g, kitten, Event::Cast, &Call::Cast { caster: P0, c: spell }).unwrap();
            1
        }),
        ("Displacer Kitten (a person)", |g, _| {
            perm(g, P0, "Mulldrifter");
            let kitten = perm(g, P0, "Displacer Kitten");
            let spell = take(g, P0, "Swords to Plowshares");
            let h = std::sync::Arc::new(ScriptHuman::from_fn(|r| {
                if r.kind == "priority" {
                    return Answer::Action(json!({"do": "pass"}));
                }
                match r.choices.iter().position(|c| c.contains("Mulldrifter")) {
                    Some(i) => Answer::Pick(i),
                    None => Answer::Yes,
                }
            }));
            g.humans.add(P0, h);
            run_hook(g, kitten, Event::Cast, &Call::Cast { caster: P0, c: spell }).unwrap();
            1
        }),
        ("a token blinked by a person (exiled for good)", |g, _| {
            let t = token(g, P0, 2);
            sim_core::human::cards::blink(g, P0, t).unwrap();
            assert!(!g.perm(t).on_bf);
            1
        }),
    ];
    let mut wrong = vec![];
    for (name, run) in cases {
        let mut g = tbl();
        let herd = perm(&mut g, P0, "Soulherder");
        let want = run(&mut g, herd);
        let got = grown(&g, herd);
        if got != want {
            wrong.push(format!("{name}: Soulherder has {got} counters, wanted {want}"));
        }
    }
    assert!(wrong.is_empty(), "paths that did not fire the exile event once per creature: {wrong:#?}");
}

#[test]
fn rest_in_peace_exile_instead_fires_the_event() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    perm(&mut g, P1, "Rest in Peace");
    let victim = perm(&mut g, P1, "Serra Angel");
    zones::die(&mut g, victim, "destroy").unwrap();
    assert!(!g.perm(victim).on_bf);
    assert!(g.player(P1).exile.contains(&card(&g, "Serra Angel")), "exiled instead of dying");
    assert!(g.player(P1).gy.is_empty());
    assert_eq!(grown(&g, herd), 1, "it was put into exile from the battlefield");
}

#[test]
fn a_creature_that_dies_normally_is_not_exiled_for_soulherder() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let victim = perm(&mut g, P1, "Serra Angel");
    zones::die(&mut g, victim, "destroy").unwrap();
    assert!(g.player(P1).gy.contains(&card(&g, "Serra Angel")));
    assert_eq!(grown(&g, herd), 0, "it died: not exiled");
}

#[test]
fn a_blinked_soulherder_does_not_hear_its_own_exile() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let back = sim_core::impls::t2::blink(&mut g, P0, herd).unwrap().unwrap();
    assert!(!g.perm(herd).on_bf && g.perm(back).on_bf);
    assert_eq!(grown(&g, back), 0, "the new Soulherder was not on the battlefield when the old one was exiled");
}

#[test]
fn the_exile_trigger_waits_behind_the_enters_trigger_of_the_returned_creature() {
    use sim_core::hooks::TrigAct;
    let mut g = tbl();
    perm(&mut g, P0, "Soulherder");
    let wall = perm(&mut g, P0, "Wall of Omens");
    g.resolving += 1; // hold the triggers: they are queued, not resolved
    sim_core::impls::t2::blink(&mut g, P0, wall).unwrap();
    g.resolving -= 1;
    let order: Vec<&str> = g
        .trig_queue
        .iter()
        .map(|t| match &t.act {
            TrigAct::Hook { event: Event::ExiledFromBf, .. } => "exiled",
            _ => "enters",
        })
        .collect();
    assert_eq!(order, ["enters", "exiled"], "the creature's enters trigger is queued first, as it always was");
}

// ------------------------------------------------------------------ Syr Vondam, Sunstar Exemplar
const VONDAM: &str = "Syr Vondam, Sunstar Exemplar";

fn vondam_table() -> (Game, PermId) {
    let mut g = tbl();
    let v = perm(&mut g, P0, VONDAM);
    (g, v)
}

/// a person at P0 who answers a question by the first choice that contains `word` (decline otherwise) and passes
/// priority; the questions asked (not priority) are collected
fn person_picking(g: &mut Game, word: &'static str) -> std::sync::Arc<std::sync::Mutex<Vec<String>>> {
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let log = asked.clone();
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        log.lock().unwrap().push(format!("{}: {:?}", r.prompt, r.choices));
        match r.choices.iter().position(|c| c.contains(word)) {
            Some(i) => Answer::Pick(i),
            None => Answer::Decline,
        }
    }));
    g.humans.add(P0, h);
    asked
}

#[test]
fn syr_vondam_grows_and_gains_when_another_creature_of_yours_dies_or_is_exiled() {
    let (mut g, v) = vondam_table();
    let life = g.player(P0).life;
    // dies: a token
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!((g.perm(v).plus, g.player(P0).life), (1, life + 1), "a token of yours died");
    // exiled: Swords on your own creature
    let own = perm(&mut g, P0, "Serra Angel");
    let swords = take(&mut g, P0, "Swords to Plowshares");
    let before = g.player(P0).life;
    cast::resolve(&mut g, P0, swords, &Ctx { target: Some(own), ..Ctx::default() }, "hand").unwrap();
    assert_eq!(g.perm(v).plus, 2, "an exiled creature of yours");
    assert_eq!(g.player(P0).life, before + 4 + 1, "Swords gives its controller 4 (its power), Vondam 1");
    // blinked: put into exile too
    let wall = perm(&mut g, P0, "Wall of Omens");
    sim_core::impls::t2::blink(&mut g, P0, wall).unwrap();
    assert_eq!(g.perm(v).plus, 3, "a blink exiles it first");
    // one trigger per creature: two die one after the other
    let (a, b) = (token(&mut g, P0, 1), token(&mut g, P0, 1));
    let before = g.player(P0).life;
    zones::die(&mut g, a, "destroy").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    assert_eq!((g.perm(v).plus, g.player(P0).life), (5, before + 2));
}

#[test]
fn an_opponents_creature_exiled_gives_vondam_nothing() {
    let (mut g, v) = vondam_table();
    let life = g.player(P0).life;
    let theirs = perm(&mut g, P1, "Serra Angel");
    zones::exile_perm(&mut g, theirs).unwrap();
    let dead = token(&mut g, P1, 2);
    zones::die(&mut g, dead, "destroy").unwrap();
    let rock = perm(&mut g, P0, "Sol Ring"); // not a creature
    zones::die(&mut g, rock, "destroy").unwrap();
    assert_eq!((g.perm(v).plus, g.player(P0).life), (0, life), "nothing: not a creature of yours");
}

#[test]
fn a_creature_that_dies_stolen_counts_for_whoever_controlled_it() {
    let (mut g, v) = vondam_table();
    let life = g.player(P0).life;
    let stolen = perm(&mut g, P1, "Serra Angel");
    sim_core::engine::control::gain_control(&mut g, P0, stolen, sim_core::engine::control::Duration::Permanent);
    assert_eq!(g.perm(stolen).owner, P0);
    zones::die(&mut g, stolen, "destroy").unwrap();
    assert_eq!((g.perm(v).plus, g.player(P0).life), (1, life + 1), "it was a creature you controlled as it died");
}

#[test]
fn vondam_does_not_trigger_for_himself_dying_or_blinked() {
    let (mut g, v) = vondam_table();
    let life = g.player(P0).life;
    let back = sim_core::impls::t2::blink(&mut g, P0, v).unwrap().unwrap();
    assert_eq!((g.perm(back).plus, g.player(P0).life), (0, life), "the new Syr Vondam saw nothing: not another creature");
}

#[test]
fn syr_vondam_at_power_four_destroys_when_it_dies_or_is_exiled() {
    for how in ["dies", "exiled", "rest in peace", "blinked"] {
        let (mut g, v) = vondam_table();
        g.perm_mut(v).plus = 2; // a 4/4
        let prize = perm(&mut g, P1, "Serra Angel");
        let land = lands(&mut g, P1, "Plains", 1, false)[0];
        let mut want = prize;
        match how {
            "dies" => zones::die(&mut g, v, "destroy").unwrap(),
            "exiled" => zones::exile_perm(&mut g, v).unwrap(),
            "rest in peace" => {
                want = perm(&mut g, P1, "Rest in Peace"); // worth more to the AI than the Angel
                zones::die(&mut g, v, "destroy").unwrap();
            }
            _ => {
                sim_core::impls::t2::blink(&mut g, P0, v).unwrap();
            }
        }
        assert!(!g.perm(want).on_bf, "{how}: the best opposing nonland permanent is destroyed");
        assert!(g.perm(land).on_bf, "{how}: a land is not a legal target");
    }
}

#[test]
fn syr_vondam_at_power_three_does_nothing_and_the_power_is_the_last_known_one() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 1; // 3/3
    let prize = perm(&mut g, P1, "Serra Angel");
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(g.perm(prize).on_bf, "power 3: no trigger");
    // an Equipment gives power as he leaves; it falls off with him, but his last known power counts
    let (mut g, v) = vondam_table();
    let sword = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(sword).attached = Some(v);
    assert_eq!(sim_core::engine::values::epow(&g, v), 4, "2/2 with +2/+2");
    let prize = perm(&mut g, P1, "Serra Angel");
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(!g.perm(prize).on_bf, "power 4 as he left (the Equipment was still on him)");
}

#[test]
fn syr_vondams_destroy_takes_no_target_when_there_is_none_worth_it() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    let own = perm(&mut g, P0, "Serra Angel");
    let l = lands(&mut g, P1, "Plains", 1, false)[0];
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(g.perm(own).on_bf && g.perm(l).on_bf, "up to one: it never destroys your own permanent or a land");
}

#[test]
fn syr_vondam_gains_for_creatures_in_the_same_wipe() {
    // a destroy wipe (Kaya's Wrath, Toxic Deluge: they die) and an exile wipe (Farewell): both look back in time
    for (kind, with_destroy) in [("destroy", false), ("destroy", true), ("exile", false), ("exile", true)] {
        let mut g = tbl();
        let first = token(&mut g, P0, 1);
        let v = perm(&mut g, P0, VONDAM);
        let (second, third) = (token(&mut g, P0, 1), token(&mut g, P0, 1));
        g.perm_mut(v).plus = if with_destroy { 2 } else { 0 };
        let prize = perm(&mut g, P1, "Smothering Tithe");
        let life = g.player(P0).life;
        let theirs = token(&mut g, P1, 3);
        removal::apply_wipe(&mut g, P0, kind, None, &Ctx::default()).unwrap();
        for m in [first, v, second, third, theirs] {
            assert!(!g.perm(m).on_bf, "{kind}: the wipe took every creature");
        }
        assert_eq!(g.player(P0).life, life + 3, "{kind}: one trigger for each of the three others, whoever left first");
        assert_eq!(g.perm(v).plus, if with_destroy { 2 } else { 0 }, "he was gone: the counters have nowhere to go");
        assert_eq!(g.perm(prize).on_bf, !with_destroy, "{kind}: at power 4 or more his destroy takes the Tithe");
    }
}

#[test]
fn a_syr_vondam_that_lost_his_abilities_triggers_nothing() {
    let mut g = tbl();
    let v = perm(&mut g, P0, VONDAM);
    g.perm_mut(v).plus = 2;
    g.perm_mut(v).neutered = true;
    let other = token(&mut g, P0, 1);
    let prize = perm(&mut g, P1, "Smothering Tithe");
    let life = g.player(P0).life;
    removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.perm(v).on_bf && !g.perm(other).on_bf);
    assert_eq!(g.player(P0).life, life, "no life for the creature that left with him");
    assert!(g.perm(prize).on_bf, "and no destroy of his own");
}

#[test]
fn vondams_wipe_bookkeeping_is_cleared_with_the_wipe() {
    let mut g = tbl();
    let v = perm(&mut g, P0, VONDAM);
    removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.perm(v).on_bf);
    let life = g.player(P0).life;
    let later = token(&mut g, P0, 1);
    removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.perm(later).on_bf);
    assert_eq!(g.player(P0).life, life, "a dead Syr Vondam hears nothing from a later wipe");
    let alone = token(&mut g, P0, 1);
    zones::die(&mut g, alone, "destroy").unwrap();
    assert_eq!(g.player(P0).life, life, "nor from a creature dying alone");
}

#[test]
fn vondams_trigger_waits_behind_the_enters_trigger_of_a_blinked_creature() {
    use sim_core::hooks::TrigAct;
    let (mut g, v) = vondam_table();
    let wall = perm(&mut g, P0, "Wall of Omens");
    g.resolving += 1;
    sim_core::impls::t2::blink(&mut g, P0, wall).unwrap();
    g.resolving -= 1;
    let order: Vec<&str> = g
        .trig_queue
        .iter()
        .map(|t| match &t.act {
            TrigAct::Hook { event: Event::ExiledFromBf, .. } if t.src == Some(v) => "vondam",
            _ => "enters",
        })
        .collect();
    assert_eq!(order, ["enters", "vondam"]);
}

#[test]
fn a_person_is_asked_syr_vondams_destroy_target() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    let prize = perm(&mut g, P1, "Serra Angel");
    let lesser = perm(&mut g, P2, "Sol Ring");
    let asked = person_picking(&mut g, "Sol Ring"); // the person picks the lesser one
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(g.perm(prize).on_bf && !g.perm(lesser).on_bf, "the person's choice, not the AI's");
    let q = asked.lock().unwrap().clone();
    assert_eq!(q.len(), 1, "asked once: {q:?}");
    assert!(q[0].contains("Serra Angel") && q[0].contains("no target"), "{q:?}");
    // declining takes no target
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    let prize = perm(&mut g, P1, "Serra Angel");
    person_picking(&mut g, "nothing matches");
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(g.perm(prize).on_bf, "up to one: a person may choose none");
}
