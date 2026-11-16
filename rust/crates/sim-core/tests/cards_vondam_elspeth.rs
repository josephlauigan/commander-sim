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
