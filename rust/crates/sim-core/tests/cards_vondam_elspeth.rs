//! The exile-from-the-battlefield event (`zones::exiled_from_bf`), Syr Vondam, Sunstar Exemplar, and Elspeth,
//! Knight-Errant (+1, +1, the -8 emblem). Each expectation follows the card's Oracle text (data/scryfall_cache.json);
//! the rules quoted are from the Comprehensive Rules (603.10a: leaves-the-battlefield abilities look back in time).

mod play_common;
use play_common::*;
use sim_core::engine::hooks::{fire_trigger, run_hook};
use sim_core::engine::{cast, removal, values, zones};
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
    assert_eq!(
        (g.perm(back).plus, g.player(P0).life),
        (0, life),
        "the new Syr Vondam saw nothing: not another creature"
    );
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

// ------------------------------------------------------------------ Elspeth, Knight-Errant: the -8 emblem
const EMBLEM: &str = "elspeth_ke";

fn indestructible(g: &Game, m: PermId) -> bool {
    sim_core::dsl::has_kw(g, m, "indestructible")
}

#[test]
fn elspeths_emblem_makes_artifacts_creatures_enchantments_and_lands_indestructible() {
    let mut g = tbl();
    g.player_mut(P0).emblems.push(EMBLEM);
    let creature = perm(&mut g, P0, "Serra Angel");
    let soldier = token(&mut g, P0, 2);
    let artifact = perm(&mut g, P0, "Sol Ring");
    let enchantment = perm(&mut g, P0, "Smothering Tithe");
    let land = lands(&mut g, P0, "Plains", 1, false)[0];
    let walker = perm(&mut g, P0, "Jace, the Mind Sculptor");
    let theirs = [perm(&mut g, P1, "Serra Angel"), perm(&mut g, P1, "Sol Ring")];
    for m in [creature, soldier, artifact, enchantment, land] {
        assert!(indestructible(&g, m), "{}: yours, and an artifact, creature, enchantment or land", g.perm(m).name);
        zones::die(&mut g, m, "destroy").unwrap();
        assert!(g.perm(m).on_bf, "{} was not destroyed", g.perm(m).name);
    }
    assert!(!indestructible(&g, walker), "a planeswalker is not on the list");
    zones::die(&mut g, walker, "destroy").unwrap();
    assert!(!g.perm(walker).on_bf, "so it is destroyed");
    for m in theirs {
        assert!(!indestructible(&g, m), "an opponent's {} is not protected", g.perm(m).name);
    }
    // land destruction asks the same question
    sim_core::impls::fixes::destroy_land(&mut g, P0, land).unwrap();
    assert!(g.perm(land).on_bf, "Strip Mine and Wasteland do nothing to it");
}

#[test]
fn elspeths_emblem_stops_a_destroy_wipe_but_not_a_sacrifice_or_toughness_zero() {
    let mut g = tbl();
    g.player_mut(P0).emblems.push(EMBLEM);
    let (a, b, c) = (perm(&mut g, P0, "Serra Angel"), perm(&mut g, P0, "Grave Titan"), token(&mut g, P0, 2));
    let theirs = perm(&mut g, P1, "Serra Angel");
    removal::apply_wipe(&mut g, P1, "destroy", None, &Ctx::default()).unwrap();
    assert!(g.perm(a).on_bf && g.perm(b).on_bf && g.perm(c).on_bf, "Kaya's Wrath and Wrath of God: yours live");
    assert!(!g.perm(theirs).on_bf, "an opponent's creature dies");
    zones::die(&mut g, a, "sac").unwrap();
    assert!(!g.perm(a).on_bf, "a sacrifice is not destruction");
    g.perm_mut(b).eot_pt = (-9, -9);
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.perm(b).on_bf, "toughness 0 or less: a state-based action, which indestructible does not stop");
}

#[test]
fn elspeths_emblem_follows_the_controller() {
    let mut g = tbl();
    g.player_mut(P0).emblems.push(EMBLEM);
    let m = perm(&mut g, P0, "Serra Angel");
    assert!(indestructible(&g, m));
    sim_core::engine::control::gain_control(&mut g, P1, m, sim_core::engine::control::Duration::Permanent);
    assert!(!indestructible(&g, m), "under an opponent's control it is not yours");
}

#[test]
fn elspeth_sun_champions_emblem_unchanged() {
    let mut g = tbl();
    g.player_mut(P0).elspeth_emblem = true;
    let m = perm(&mut g, P0, "Serra Angel");
    assert!(sim_core::dsl::has_kw(&g, m, "flying"));
    assert_eq!(sim_core::engine::values::epow(&g, m), 4 + 2, "+2/+2 and flying");
    assert!(!indestructible(&g, m), "it says nothing about indestructible");
}

// ------------------------------------------------------------------ Elspeth, Knight-Errant: the walker
const ELSPETH: &str = "Elspeth, Knight-Errant";
use sim_core::hooks::{Action, Opt};
use sim_core::human::human::{AbilityKind, abilities_of};

/// the options of the card code of `src` (none when its card has no `options` hook)
fn options(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    match g.registry.get(cd).and_then(|i| i.options) {
        Some(f) => f(g, src, p, Some(false)).unwrap(),
        None => vec![],
    }
}

fn arg_of(o: &Opt) -> i64 {
    match &o.act {
        Some(Action::Ability { arg, .. }) => *arg,
        other => panic!("not a walker ability: {other:?}"),
    }
}

/// the AI uses the loyalty ability number `i` of the list (0: the Soldier, 1: the pump, 2: the emblem)
fn ai_uses(g: &mut Game, src: PermId, i: i64) {
    let o = options(g, src, P0);
    let found = o.iter().find(|o| arg_of(o) == i);
    assert!(found.is_some(), "ability {i} is not offered: {:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert!(sim_core::ai::act::perform(g, P0, found.unwrap().act.as_ref().unwrap()).unwrap());
}

fn white_soldiers(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.token && x.ttypes.iter().any(|t| &**t == "soldier") && x.pow == 1 && x.tgh == 1
        })
        .collect()
}

#[test]
fn the_ai_uses_elspeths_plus_one_soldier() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    assert_eq!(g.perm(e).loyalty, Some(4));
    ai_uses(&mut g, e, 0);
    assert_eq!(white_soldiers(&g, P0).len(), 1, "a 1/1 white Soldier creature token");
    assert_eq!(g.perm(e).loyalty, Some(5));
}

#[test]
fn elspeths_plus_one_pump_targets_an_attacker() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let sick = perm(&mut g, P0, "Grave Titan");
    g.perm_mut(sick).sick = true;
    let ready = token(&mut g, P0, 2);
    ai_uses(&mut g, e, 1);
    assert!(sim_core::dsl::has_kw(&g, ready, "flying"), "the creature that can attack gets flying");
    assert_eq!(sim_core::engine::values::epow(&g, ready), 2 + 3);
    assert_eq!(sim_core::engine::values::epow(&g, sick), 6, "the summoning-sick Titan is left alone");
    assert_eq!(g.perm(e).loyalty, Some(5));
    // with no creature there is nothing to pump: only the Soldier and (loyalty 8) the emblem are offered
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let args: Vec<i64> = options(&mut g, e, P0).iter().map(arg_of).collect();
    assert_eq!(args, [0], "no creature, no pump");
}

#[test]
fn the_ai_uses_elspeths_minus_eight_at_eight_loyalty() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    g.perm_mut(e).loyalty = Some(7);
    assert!(!options(&mut g, e, P0).iter().any(|o| arg_of(o) == 2), "not at 7 loyalty");
    g.perm_mut(e).loyalty = Some(8);
    let mut o = options(&mut g, e, P0);
    o.sort_by(|a, b| b.utility.total_cmp(&a.utility));
    assert!(!o.is_empty(), "Elspeth offers her abilities to the AI");
    assert_eq!(
        arg_of(&o[0]),
        2,
        "the ultimate is the best play: {:?}",
        o.iter().map(|x| (&x.label, x.utility)).collect::<Vec<_>>()
    );
    ai_uses(&mut g, e, 2);
    assert!(g.player(P0).emblems.contains(&"elspeth_ke"), "you get the emblem");
    assert!(!g.perm(e).on_bf, "Elspeth is at 0 loyalty: she is put into the graveyard");
    let m = perm(&mut g, P0, "Serra Angel");
    assert!(sim_core::dsl::has_kw(&g, m, "indestructible"));
}

/// the person uses the ability of `m` whose label contains `word`; the reason it could not, if any
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, P0, m).unwrap();
    let all: Vec<String> = list.iter().map(|x| x.0.clone()).collect();
    let found = list.into_iter().find(|(l, _)| l.contains(word));
    assert!(found.is_some(), "{} lists no ability with {word:?}: {all:?}", g.perm(m).name);
    let AbilityKind::Extra(act) = found.unwrap().1 else { panic!("not an extra ability") };
    act(g, P0, m).unwrap()
}

#[test]
fn a_person_is_offered_elspeths_three_abilities_and_asked_the_pump_target_once() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let mine = perm(&mut g, P0, "Serra Angel");
    token(&mut g, P0, 2);
    let labels: Vec<String> = abilities_of(&mut g, P0, e).unwrap().into_iter().map(|x| x.0).collect();
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(labels[0].contains("Soldier") && labels[1].contains("+3/+3") && labels[2].contains("emblem"), "{labels:?}");
    assert!(labels[0].starts_with("+1") && labels[1].starts_with("+1") && labels[2].starts_with("-8"), "{labels:?}");
    let asked = person_picking(&mut g, "Serra Angel");
    assert_eq!(use_ability(&mut g, e, "+3/+3"), None);
    let q = asked.lock().unwrap().clone();
    assert_eq!(q.iter().filter(|x| x.contains("Elspeth")).count(), 1, "the target is asked once: {q:?}");
    assert_eq!(sim_core::engine::values::epow(&g, mine), 4 + 3);
    assert!(sim_core::dsl::has_kw(&g, mine, "flying"));
    assert_eq!(g.perm(e).loyalty, Some(5));
}

#[test]
fn a_person_makes_a_soldier_with_elspeths_plus_one_and_is_asked_nothing() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let asked = person_picking(&mut g, "nothing matches");
    assert_eq!(use_ability(&mut g, e, "Soldier"), None);
    assert!(asked.lock().unwrap().is_empty(), "no question for the Soldier");
    assert_eq!(white_soldiers(&g, P0).len(), 1);
    assert_eq!(g.perm(e).loyalty, Some(5));
}

#[test]
fn a_person_cannot_aim_elspeths_pump_at_hexproof_or_protected_creatures() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let hexproof = perm(&mut g, P1, "Sylvan Caryatid");
    let protected = perm(&mut g, P1, "Malakir Bloodwitch"); // protection from white
    let fine = perm(&mut g, P1, "Serra Angel");
    let asked = person_picking(&mut g, "Serra Angel");
    assert_eq!(use_ability(&mut g, e, "+3/+3"), None);
    let q = asked.lock().unwrap().clone();
    assert_eq!(q.len(), 1, "{q:?}");
    assert!(q[0].contains("Serra Angel") && !q[0].contains("Caryatid") && !q[0].contains("Bloodwitch"), "{q:?}");
    assert_eq!(sim_core::engine::values::epow(&g, fine), 4 + 3, "an opponent's creature can be the target");
    let _ = (hexproof, protected);
}

/// a person at P0 who passes priority, answers yes to the ward question and picks `word` for any other question
fn person_paying_ward(g: &mut Game, word: &'static str) {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("ward") {
            return Answer::Yes;
        }
        r.choices.iter().position(|c| c.contains(word)).map_or(Answer::Decline, Answer::Pick)
    }));
    g.humans.add(P0, h);
}

#[test]
fn a_persons_pump_pays_the_ward_of_an_opponents_creature_or_is_countered() {
    for pay in [false, true] {
        let mut g = tbl();
        let e = perm(&mut g, P0, ELSPETH);
        let warded = perm(&mut g, P1, "Frost Titan"); // ward {2}, a 6/6
        if pay {
            g.player_mut(P0).pool.add('W', 2);
        }
        person_paying_ward(&mut g, "Frost Titan");
        assert_eq!(use_ability(&mut g, e, "+3/+3"), None);
        let grown = sim_core::engine::values::epow(&g, warded) == 6 + 3;
        assert_eq!(grown, pay, "ward {{2}}: {}", if pay { "paid" } else { "not paid: the ability is countered" });
        assert_eq!(g.perm(e).loyalty, Some(5), "the loyalty is paid either way");
    }
}

#[test]
fn a_persons_pump_rechecks_its_target_on_resolution() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let m = perm(&mut g, P0, "Mother of Runes");
    let giver = perm(&mut g, P0, "Giver of Runes");
    let giver_i = g.player(P0).perms.iter().position(|&x| x == giver).unwrap();
    let mut protected = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !protected {
                protected = true;
                return Answer::Action(json!({"do": "use", "perm": giver_i}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let word = if r.prompt.contains("which ability") {
            "protection"
        } else if r.prompt.contains("protection from what") {
            "white"
        } else {
            "Mother of Runes"
        };
        r.choices.iter().position(|c| c.contains(word)).map_or(Answer::Decline, Answer::Pick)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    assert_eq!(use_ability(&mut g, e, "+3/+3"), None);
    assert!(sim_core::engine::values::protected_from(&g, m, sim_core::cards::Colors::from_letters("W")));
    assert_eq!(sim_core::engine::values::epow(&g, m), 1, "protection from white in response: no longer a legal target");
}

#[test]
fn a_person_gets_the_emblem_from_elspeths_minus_eight() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    g.perm_mut(e).loyalty = Some(8);
    assert_eq!(use_ability(&mut g, e, "emblem"), None);
    assert!(g.player(P0).emblems.contains(&"elspeth_ke"));
    assert!(!g.perm(e).on_bf);
    // not with less loyalty
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let why = use_ability(&mut g, e, "emblem");
    assert!(why.is_some_and(|w| w.contains("loyalty")), "she has 4 loyalty");
}

// ------------------------------------------------------------------ the other exile sites
/// the single-line exile sites that fired nothing: each exiles (or blinks) a creature from the battlefield, witnessed by
/// Soulherder's counter. A table like `every_blink_path_fires_the_exile_event`; the test lists every site that came out
/// wrong. How each case reaches its site is said in its row.
#[test]
fn every_other_exile_site_fires_the_exile_event() {
    type Case = (&'static str, fn(&mut Game, PermId) -> i32);
    let cases: Vec<Case> = vec![
        // the AI answers removal aimed at its creature with Ephemerate from its hand (ai/pool.rs protect_with)
        ("the AI's Ephemerate in response to removal", |g, _| {
            let titan = perm(g, P0, "Grave Titan");
            hand(g, P0, &["Ephemerate"]);
            lands(g, P0, "Plains", 1, false);
            let saved =
                sim_core::ai::pool::protect(g, P0, titan, sim_core::sym::intern("destroy"), Some(P1), None).unwrap();
            assert!(saved && !g.perm(titan).on_bf, "Ephemerate blinked the Titan: {saved}");
            1
        }),
        // Ennis, Debate Moderator enters: exile another creature of yours until the next end step (bulk_white.rs)
        ("Ennis, Debate Moderator", |g, _| {
            let wall = perm(g, P0, "Wall of Omens");
            perm(g, P0, "Ennis, Debate Moderator");
            assert!(!g.perm(wall).on_bf, "Ennis exiled the Wall");
            1
        }),
        // unearth: exile it at the beginning of the end step (alela.rs)
        ("Reconstructed Thopter's unearth exile", |g, _| {
            let m = perm(g, P0, "Reconstructed Thopter");
            g.perm_mut(m).data.set(DataKey::Unearth, Val::Bool(true));
            run_hook(g, m, Event::EndStep, &Call::Player { p: P0 }).unwrap();
            assert!(!g.perm(m).on_bf);
            1
        }),
        // unearth: exile it at the beginning of the end step (partials.rs)
        ("Archfiend of Sorrows' unearth exile", |g, _| {
            let m = perm(g, P0, "Archfiend of Sorrows");
            g.perm_mut(m).data.set(DataKey::Unearth, Val::Bool(true));
            run_hook(g, m, Event::EndStep, &Call::Player { p: P0 }).unwrap();
            assert!(!g.perm(m).on_bf);
            1
        }),
        // {4}{U}{B}{R}: exile Nicol Bolas, then return him transformed (card_models.rs)
        ("Nicol Bolas, the Ravager transforms", |g, _| {
            let m = perm(g, P0, "Nicol Bolas, the Ravager");
            lands(g, P0, "Island", 3, false);
            lands(g, P0, "Swamp", 2, false);
            lands(g, P0, "Mountain", 2, false);
            assert!(sim_core::impls::card_models::bolas_flip(g, m, P0, 0).unwrap());
            assert!(!g.perm(m).on_bf);
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
    assert!(wrong.is_empty(), "sites that did not fire the exile event once per creature: {wrong:#?}");
}

/// Sakashima's Protege "enters as a copy": it is never on the battlefield as itself, so nothing is exiled from it
#[test]
fn a_protege_entering_as_a_copy_exiles_nothing() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let _shiny = perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P0, "Sakashima's Protege");
    assert_eq!(grown(&g, herd), 0, "a replacement effect, not a leave of the battlefield");
}

/// the power and the wipe Syr Vondam left in are kept under their own key, not under the page counters of a Mazemind Tome
#[test]
fn syr_vondams_last_known_information_has_its_own_key() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2; // a 4/4
    zones::die(&mut g, v, "destroy").unwrap();
    assert_eq!(
        g.perm(v).data.get(DataKey::LastKnown),
        Some(&Val::List(vec![Val::Int(4), Val::Int(0)])),
        "last known power 4, in no wipe"
    );
    assert!(g.perm(v).data.get(DataKey::Counters).is_none(), "the counters key is not Syr Vondam's");
}

// ------------------------------------------------------------------ Kaya's Ghostform: the returned object hears nothing
/// Kaya's Ghostform returns an exiled creature at once, as a new object. The new object was not on the battlefield when
/// the old one was exiled (CR 603.10a, 400.7), so it sees nothing of that exile: no counter, no life
#[test]
fn ghostform_returned_vondam_does_not_see_his_own_exile() {
    let mut g = tbl();
    let v = perm(&mut g, P0, VONDAM);
    let aura = perm(&mut g, P0, "Kaya's Ghostform");
    assert_eq!(g.perm(aura).attached, Some(v));
    let life = g.player(P0).life;
    zones::exile_perm(&mut g, v).unwrap();
    let back = perms_named(&g, P0, VONDAM);
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], v);
    assert_eq!((g.perm(back[0]).plus, g.player(P0).life), (0, life), "new Vondam did not exist at exile time");
}

#[test]
fn ghostform_returned_soulherder_does_not_see_his_own_exile() {
    let mut g = tbl();
    let herd = perm(&mut g, P0, "Soulherder");
    let aura = perm(&mut g, P0, "Kaya's Ghostform");
    assert_eq!(g.perm(aura).attached, Some(herd));
    zones::exile_perm(&mut g, herd).unwrap();
    let back = perms_named(&g, P0, "Soulherder");
    assert_eq!(back.len(), 1);
    assert_eq!(grown(&g, back[0]), 0, "new Soulherder did not exist at exile time");
}

/// control: the object that was exiled (a power-4 Syr Vondam under Ghostform) still queues his own exile trigger, and
/// a Soulherder that was already there still hears the exile
#[test]
fn the_exiled_original_still_triggers_under_ghostform() {
    let mut g = tbl();
    let v = perm(&mut g, P0, VONDAM);
    g.perm_mut(v).plus = 2; // a 4/4
    let aura = perm(&mut g, P0, "Kaya's Ghostform");
    assert_eq!(g.perm(aura).attached, Some(v));
    let herd = perm(&mut g, P0, "Soulherder");
    let prize = perm(&mut g, P1, "Serra Angel");
    zones::exile_perm(&mut g, v).unwrap();
    assert!(!g.perm(prize).on_bf, "his own exile trigger destroys the best opposing permanent");
    assert_eq!(grown(&g, herd), 1, "the Soulherder that was there hears a creature exiled");
}

// ------------------------------------------------------------------ the exile sites that fired nothing (token exiles)
/// the creature exile sites that did not fire the exile event, each witnessed by an existing Soulherder's counter: the
/// victim left the battlefield, then Soulherder has exactly one counter. A table; the test lists every row that is wrong.
#[test]
fn the_token_and_oath_exile_sites_fire_the_exile_event() {
    type Case = (&'static str, fn(&mut Game) -> PermId);
    let cases: Vec<Case> = vec![
        // Restoration Angel's enters trigger on a token of yours: exiled for good (t2.rs)
        ("Restoration Angel on a token", |g| {
            let t = token(g, P0, 3);
            g.resto_target = Some(t);
            perm(g, P0, "Restoration Angel");
            g.resto_target = None;
            t
        }),
        // Detention Sphere exiles every token with the target's name (rules2.rs)
        ("Detention Sphere on a token", |g| {
            let t = token(g, P1, 5);
            perm(g, P0, "Detention Sphere");
            t
        }),
        // Eldrazi Displacer's AI option on an opposing token (rules2.rs)
        ("Eldrazi Displacer on a token", |g| {
            let t = token(g, P1, 5);
            let d = perm(g, P0, "Eldrazi Displacer");
            lands(g, P0, "Wastes", 3, false);
            let f = g.registry.get(g.perm(d).cd.unwrap()).unwrap().options.unwrap();
            let act =
                f(g, d, P0, Some(false)).unwrap().into_iter().find(|o| o.label.contains("token")).unwrap().act.unwrap();
            assert!(sim_core::ai::act::perform(g, P0, &act).unwrap());
            t
        }),
        // Rionya's token copies are exiled at the end step (rules2.rs)
        ("Rionya's token copies", |g| {
            let r = perm(g, P0, "Rionya, Fire Dancer");
            let t = token(g, P0, 3);
            g.perm_mut(t).data.set(DataKey::Rionya, Val::Bool(true));
            run_hook(g, r, Event::EndStep, &Call::Player { p: P0 }).unwrap();
            t
        }),
        // Oath of Teferi exiles a permanent of yours until the end step (partials.rs)
        ("Oath of Teferi", |g| {
            let wall = perm(g, P0, "Wall of Omens");
            perm(g, P0, "Oath of Teferi");
            wall
        }),
    ];
    let mut wrong = vec![];
    for (name, run) in cases {
        let mut g = tbl();
        let herd = perm(&mut g, P0, "Soulherder");
        let victim = run(&mut g);
        assert!(!g.perm(victim).on_bf, "{name}: the site must exile its victim");
        if grown(&g, herd) != 1 {
            wrong.push(format!("{name}: Soulherder has {} counters, wanted 1", grown(&g, herd)));
        }
    }
    assert!(wrong.is_empty(), "sites that did not fire the exile event: {wrong:#?}");
}

/// Restoration Angel exiling your Soldier token is a creature of yours put into exile: Syr Vondam hears it
#[test]
fn restoration_angel_exiling_your_token_gives_syr_vondam_a_counter() {
    let (mut g, v) = vondam_table();
    let soldier = token(&mut g, P0, 1);
    let life = g.player(P0).life;
    g.resto_target = Some(soldier);
    perm(&mut g, P0, "Restoration Angel");
    g.resto_target = None;
    assert!(!g.perm(soldier).on_bf, "Restoration Angel exiled the token");
    assert_eq!((g.perm(v).plus, g.player(P0).life), (1, life + 1), "a counter and 1 life");
}

// ------------------------------------------------------------------ Eerie Interlude exiles its creatures together
/// Eerie Interlude exiles your creatures at the same time, so a Syr Vondam it exiles sees the others leave with him (CR
/// 603.10a), in whichever order they are exiled
#[test]
fn interlude_is_simultaneous_for_vondam_in_either_order() {
    for vondam_first in [true, false] {
        let (mut g, v) = vondam_table();
        let ally = token(&mut g, P0, 1);
        let life = g.player(P0).life;
        let ms = if vondam_first { vec![v, ally] } else { vec![ally, v] };
        g.resolving += 1; // Eerie Interlude is resolving
        sim_core::impls::galadriel::interlude(&mut g, P0, &ms).unwrap();
        g.resolving -= 1;
        sim_core::engine::stack::flush_triggers(&mut g).unwrap();
        assert_eq!(g.player(P0).life, life + 1, "Vondam-first={vondam_first}: ally exiled at the same time");
        assert_eq!(g.perm(v).plus, 0, "Vondam-first={vondam_first}: the counter has nowhere to go");
    }
}

/// the same with nothing holding the triggers while the spell resolves: Interlude holds them itself, so the counter is
/// not put on a Syr Vondam that is leaving too
#[test]
fn interlude_holds_its_own_triggers_until_everything_has_left() {
    for vondam_first in [true, false] {
        let (mut g, v) = vondam_table();
        let ally = token(&mut g, P0, 1);
        let life = g.player(P0).life;
        let ms = if vondam_first { vec![v, ally] } else { vec![ally, v] };
        sim_core::impls::galadriel::interlude(&mut g, P0, &ms).unwrap();
        assert!(g.trig_queue.is_empty(), "the triggers were resolved");
        assert_eq!(g.player(P0).life, life + 1, "Vondam-first={vondam_first}: ally exiled at the same time");
        assert_eq!(
            g.perm(v).plus,
            0,
            "Vondam-first={vondam_first}: he left with the ally, so the counter has nowhere to go"
        );
    }
}

/// Interlude cast in response to a wipe (the AI does): a Syr Vondam it exiles left before the wipe resolved, so a token
/// that dies later in that wipe gives him nothing
#[test]
fn interlude_inside_a_wipe_does_not_make_vondam_leave_with_the_wipe() {
    let (mut g, v) = vondam_table();
    let victim = token(&mut g, P0, 1);
    let life = g.player(P0).life;
    g.batch += 1; // a wipe is resolving (as `removal::apply_wipe` sets it)
    g.cur_batch = Some(g.batch);
    g.resolving += 1;
    sim_core::impls::galadriel::interlude(&mut g, P0, &[v]).unwrap();
    assert!(!g.perm(v).on_bf, "Interlude exiled Vondam");
    zones::die(&mut g, victim, "destroy").unwrap(); // dies later in the wipe
    g.cur_batch = None;
    g.resolving -= 1;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(!g.perm(victim).on_bf);
    assert_eq!(g.player(P0).life, life, "he was gone before the wipe resolved");
}

// ------------------------------------------------------------------ last known power in a simultaneous wipe
/// Syr Vondam and the anthem that made him a 4/4 die in the same wipe: he was power 4 just before (CR 603.10a), whichever
/// of them the wipe takes first
#[test]
fn simultaneous_anthem_death_preserves_vondam_power() {
    let mut g = tbl();
    let anthem = perm(&mut g, P0, "Elesh Norn, Grand Cenobite");
    let v = perm(&mut g, P0, VONDAM);
    assert_eq!(values::epow(&g, v), 4);
    let prize = perm(&mut g, P1, "Smothering Tithe");
    removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.perm(anthem).on_bf && !g.perm(v).on_bf);
    assert!(!g.perm(prize).on_bf, "Vondam was power four immediately before the simultaneous wipe");
}

/// control: a Syr Vondam that survives a wipe (the anthem does not) and dies alone later was power 3 as he left, not
/// the 4 he was before that wipe
#[test]
fn a_vondam_that_survived_a_wipe_and_dies_alone_uses_his_power_at_that_moment() {
    let mut g = tbl();
    let anthem = perm(&mut g, P0, "Elesh Norn, Grand Cenobite");
    let v = perm(&mut g, P0, VONDAM);
    assert_eq!(values::epow(&g, v), 4);
    let prize = perm(&mut g, P1, "Smothering Tithe");
    g.perm_mut(v).eot_kw.push("indestructible"); // he survives the wipe
    removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.perm(anthem).on_bf && g.perm(v).on_bf, "the anthem died, he did not");
    assert_eq!(values::epow(&g, v), 3, "2/2 and the counter the anthem's death gave him");
    g.perm_mut(v).eot_kw.clear();
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(!g.perm(v).on_bf);
    assert!(g.perm(prize).on_bf, "he was power 3 as he died alone: no destroy");
}

// ------------------------------------------------------------------ own shroud, own hexproof
/// a person at P0 whose questions are all declined; true once any choice of a question contains `word`
fn offered(g: &mut Game, word: &'static str) -> std::sync::Arc<std::sync::Mutex<bool>> {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(false));
    let log = seen.clone();
    g.humans.add(
        P0,
        std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
            if r.kind == "priority" {
                return Answer::Action(json!({"do": "pass"}));
            }
            *log.lock().unwrap() |= r.choices.iter().any(|s| s.contains(word));
            Answer::Decline
        })),
    );
    seen
}

/// Syr Vondam (power 4) of P0 dies; the person is asked for his target. Was `word` among the choices?
fn offered_as_vondams_target(setup: fn(&mut Game), word: &'static str) -> bool {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    setup(&mut g);
    let seen = offered(&mut g, word);
    zones::die(&mut g, v, "destroy").unwrap();
    let r = *seen.lock().unwrap();
    r
}

/// a creature of P0's that nothing may target: shroud stops its own controller too (CR 702.18a)
#[test]
fn owned_shroud_is_not_offered_as_a_vondam_destroy_target() {
    let shrouded = offered_as_vondams_target(
        |g| {
            let t = perm(g, P0, "Serra Angel");
            g.perm_mut(t).eot_kw.push("shroud");
        },
        "Serra Angel",
    );
    assert!(!shrouded, "shroud forbids targeting even your own permanent");
}

#[test]
fn own_lightning_greaves_and_whispersilk_cloak_stop_vondams_destroy_too() {
    type Setup = fn(&mut Game);
    let rows: [(&str, Setup); 2] = [
        ("Lightning Greaves", |g| {
            let t = perm(g, P0, "Serra Angel");
            let e = perm(g, P0, "Lightning Greaves");
            g.perm_mut(e).attached = Some(t);
        }),
        ("Whispersilk Cloak", |g| {
            let t = perm(g, P0, "Serra Angel");
            let e = perm(g, P0, "Whispersilk Cloak");
            g.perm_mut(e).attached = Some(t);
        }),
    ];
    let offered: Vec<&str> = rows
        .into_iter()
        .filter(|&(_, setup)| offered_as_vondams_target(setup, "Serra Angel"))
        .map(|(e, _)| e)
        .collect();
    assert!(offered.is_empty(), "shroud, so not even yours may target it, but it was offered with: {offered:?}");
}

/// controls: your own hexproof permanent is a legal target (hexproof stops only opponents); an opponent's is not
#[test]
fn hexproof_is_offered_for_your_own_permanent_and_not_for_an_opponents() {
    assert!(
        offered_as_vondams_target(
            |g| {
                perm(g, P0, "Sylvan Caryatid");
            },
            "Sylvan Caryatid",
        ),
        "your own hexproof permanent may be targeted by your ability"
    );
    assert!(
        !offered_as_vondams_target(
            |g| {
                perm(g, P1, "Sylvan Caryatid");
            },
            "Sylvan Caryatid",
        ),
        "an opponent's hexproof permanent may not"
    );
}

/// Elspeth's pump has the same rule: shroud from a keyword, Lightning Greaves or Whispersilk Cloak stops its controller
/// too; hexproof (the keyword, Swiftfoot Boots) stops only opponents
#[test]
fn elspeths_pump_respects_own_shroud_from_every_source() {
    use sim_core::impls::vondam_elspeth::pump_legal;
    type Row = (&'static str, fn(&mut Game) -> PermId, bool);
    let rows: [Row; 6] = [
        (
            "shroud keyword",
            |g| {
                let t = perm(g, P0, "Serra Angel");
                g.perm_mut(t).eot_kw.push("shroud");
                t
            },
            false,
        ),
        (
            "Lightning Greaves",
            |g| {
                let t = perm(g, P0, "Serra Angel");
                let e = perm(g, P0, "Lightning Greaves");
                g.perm_mut(e).attached = Some(t);
                t
            },
            false,
        ),
        (
            "Whispersilk Cloak",
            |g| {
                let t = perm(g, P0, "Serra Angel");
                let e = perm(g, P0, "Whispersilk Cloak");
                g.perm_mut(e).attached = Some(t);
                t
            },
            false,
        ),
        ("own hexproof keyword", |g| perm(g, P0, "Sylvan Caryatid"), true),
        (
            "own Swiftfoot Boots",
            |g| {
                let t = perm(g, P0, "Serra Angel");
                let e = perm(g, P0, "Swiftfoot Boots");
                g.perm_mut(e).attached = Some(t);
                t
            },
            true,
        ),
        ("an opponent's hexproof keyword", |g| perm(g, P1, "Sylvan Caryatid"), false),
    ];
    let mut wrong = vec![];
    for (name, setup, legal) in rows {
        let mut g = tbl();
        let e = perm(&mut g, P0, ELSPETH);
        let t = setup(&mut g);
        if pump_legal(&g, P0, e, t) != legal {
            wrong.push(format!("{name}: pump_legal should be {legal}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// hexproof stops only opponents (CR 702.11b): Syr Vondam's controller may destroy their own Sylvan Caryatid
#[test]
fn own_hexproof_is_a_legal_vondam_destroy_target() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    let target = perm(&mut g, P0, "Sylvan Caryatid");
    person_picking(&mut g, "Sylvan Caryatid");
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(!g.perm(target).on_bf, "hexproof allows your own ability to destroy it");
}

/// control: your own Phantasmal Image that Vondam targets is still sacrificed ("when it becomes the target of a spell or
/// ability"), even when destroy could not kill it
#[test]
fn a_phantasmal_image_that_vondam_targets_is_still_sacrificed() {
    let (mut g, v) = vondam_table();
    g.perm_mut(v).plus = 2;
    perm(&mut g, P1, "Serra Angel");
    let img = perm(&mut g, P0, "Phantasmal Image");
    assert!(sim_core::impls::t6_copies::is_image(&g, img));
    g.perm_mut(img).eot_kw.push("indestructible"); // a destroy would not take it
    g.humans.add(
        P0,
        std::sync::Arc::new(ScriptHuman::from_fn(|r| {
            if r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Pick(0) }
        })),
    );
    zones::die(&mut g, v, "destroy").unwrap();
    assert!(!g.perm(img).on_bf, "the Image became the target of Vondam's ability and was sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Phantasmal Image")));
}

// ------------------------------------------------------------------ the AI's pump commits its target on activation
/// the AI activates Elspeth's +1 aimed at its Serra Angel (the only creature that can attack); in response the person's
/// Giver of Runes gives the Angel protection from white. Elspeth is white, so the Angel is no longer a legal target and
/// the ability does nothing (CR 608.2b): the target was chosen on activation (CR 602.2b), so it does not switch to the
/// Giver as the ability resolves. The loyalty was paid once.
#[test]
fn ai_pump_cannot_switch_targets_after_response() {
    let mut g = tbl();
    let e = perm(&mut g, P0, ELSPETH);
    let angel = perm(&mut g, P0, "Serra Angel");
    let giver = perm(&mut g, P0, "Giver of Runes");
    g.perm_mut(angel).sick = false;
    let giver_i = g.player(P0).perms.iter().position(|&m| m == giver).unwrap();
    let mut responded = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !responded {
                responded = true;
                return Answer::Action(json!({"do": "use", "perm": giver_i}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let word = if r.prompt.contains("which ability") {
            "protection"
        } else if r.prompt.contains("protection from what") {
            "white"
        } else {
            "Serra Angel"
        };
        r.choices.iter().position(|s| s.contains(word)).map_or(Answer::Decline, Answer::Pick)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    ai_uses(&mut g, e, 1);
    assert!(values::protected_from(&g, angel, sim_core::cards::Colors::from_letters("W")), "the response resolved");
    assert_eq!(g.perm(e).loyalty, Some(5), "the loyalty is paid once");
    assert_eq!(g.perm(giver).eot_pt, (0, 0), "the invalidated target fizzles: it does not switch to the Giver");
    assert_eq!(g.perm(angel).eot_pt, (0, 0), "the Angel is protected from white: no longer a legal target");
}
