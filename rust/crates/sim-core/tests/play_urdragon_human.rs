//! Practice mode: the Ur-Dragon deck's combos for a person at the seat (`human/urdragon.rs`): Godo's search, equip
//! discounts, choosing what Hammer / Sigarda's Aid / Brass Squire attach to, Peter Parker's Camera, Jaxis, Twinflame
//! with Dualcaster Mage (and the loop), and the repeat mode of the Helm loop (Godo, Combat Celebrant).

mod play_common;
use play_common::*;
use sim_core::state::Step;
use std::sync::{Arc, Mutex};

const GODO: &str = "Godo, Bandit Warlord";
const HELM: &str = "Helm of the Host";
const HAMMER: &str = "Hammer of Nazahn";

fn urdragon_table() -> Game {
    table(&["urdragon", "kinnan-simic-mana-combo", "urza-mono-blue-artifacts"])
}

type Rule = (&'static str, Box<dyn Fn(&Request) -> Answer + Send>);

/// a person who answers by what is asked: each rule is used once, the first whose text is in the prompt (or the
/// kind); anything else is passed or declined. Everything asked is recorded as "kind: prompt".
fn scripted(rules: Vec<Rule>, log: Arc<Mutex<Vec<String>>>) -> ScriptHuman {
    let mut rules: Vec<Option<Rule>> = rules.into_iter().map(Some).collect();
    ScriptHuman::from_fn(move |r| {
        log.lock().unwrap().push(format!("{}: {}", r.kind, r.prompt));
        for slot in rules.iter_mut() {
            if slot.as_ref().is_some_and(|(k, _)| r.prompt.contains(k) || &*r.kind == *k) {
                let (_, f) = slot.take().unwrap();
                return f(r);
            }
        }
        if r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    })
}

fn rule(key: &'static str, f: impl Fn(&Request) -> Answer + Send + 'static) -> Rule {
    (key, Box::new(f))
}

/// pick the first choice containing `word`
fn text(word: &'static str) -> impl Fn(&Request) -> Answer + Send + 'static {
    move |r| {
        Answer::Pick(
            r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
        )
    }
}

fn first(_: &Request) -> Answer {
    Answer::Pick(0)
}

fn index_of(g: &Game, p: PlayerId, m: PermId) -> usize {
    g.player(p).perms.iter().position(|&x| x == m).unwrap()
}

fn asked_has(log: &Arc<Mutex<Vec<String>>>, what: &str) -> bool {
    log.lock().unwrap().iter().any(|l| l.contains(what))
}

fn seat_script(g: &mut Game, rules: Vec<Rule>) -> Arc<Mutex<Vec<String>>> {
    let log = Arc::new(Mutex::new(vec![]));
    g.humans.add(P0, Arc::new(scripted(rules, log.clone())));
    log
}

// ------------------------------------------------------------------ Godo's search
#[test]
fn godo_lets_you_search_for_any_equipment() {
    let mut g = urdragon_table();
    let log = seat_script(
        &mut g,
        vec![rule("search your library for an Equipment", first), rule("search for an Equipment", text("Embercleave"))],
    );
    perm(&mut g, P0, GODO);
    assert!(has_perm(&g, P0, "Embercleave"), "any Equipment, not only the one the AI wants: {:?}", log.lock().unwrap());
    assert!(!has_perm(&g, P0, HELM));
}

#[test]
fn godos_search_is_a_may() {
    let mut g = urdragon_table();
    let log = seat_script(&mut g, vec![rule("search your library for an Equipment", |_| Answer::Pick(1))]);
    perm(&mut g, P0, GODO);
    assert!(!has_perm(&g, P0, HELM) && !has_perm(&g, P0, HAMMER) && !has_perm(&g, P0, "Embercleave"));
    assert!(!asked_has(&log, "search for an Equipment card to put onto"), "no search after declining");
}

// ------------------------------------------------------------------ equip discounts
#[test]
fn puresteel_paladin_makes_your_equip_free() {
    let mut g = urdragon_table();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Puresteel Paladin");
    for n in ["Sol Ring", "Mana Vault", "Grim Monolith"] {
        perm(&mut g, P0, n); // metalcraft
    }
    let helm = perm(&mut g, P0, HELM);
    seat_script(&mut g, vec![rule("which ability", first), rule("Equip", text("Llanowar"))]);
    let i = index_of(&g, P0, helm);
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert_eq!(g.perm(helm).attached, Some(elves), "equipped for {{0}} with Puresteel Paladin and metalcraft");
}

#[test]
fn without_metalcraft_equip_costs_the_printed_cost() {
    let mut g = urdragon_table();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Puresteel Paladin");
    let helm = perm(&mut g, P0, HELM);
    seat_script(&mut g, vec![rule("which ability", first), rule("Equip", text("Llanowar"))]);
    let i = index_of(&g, P0, helm);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert!(why.is_some_and(|w| w.contains("Can't equip")), "{{5}} is still due");
    assert_eq!(g.perm(helm).attached, None);
    let _ = elves;
}

// ------------------------------------------------------------------ what to attach
#[test]
fn hammer_asks_which_creature_gets_the_equipment() {
    let mut g = urdragon_table();
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    seat_script(
        &mut g,
        vec![
            rule("Hammer of Nazahn: attach Hammer", |r| Answer::Pick(r.choices.len() - 1)),
            rule("Hammer of Nazahn: attach Lightning Greaves", text("Birds")),
        ],
    );
    perm(&mut g, P0, HAMMER);
    let e = perm(&mut g, P0, "Lightning Greaves");
    assert_eq!(g.perm(e).attached, Some(b), "the Greaves went where the person said");
    assert_ne!(g.perm(e).attached, Some(a));
}

#[test]
fn hammer_may_attach_nothing() {
    let mut g = urdragon_table();
    perm(&mut g, P0, "Llanowar Elves");
    let log = seat_script(
        &mut g,
        vec![
            rule("Hammer of Nazahn: attach Hammer", |r| Answer::Pick(r.choices.len() - 1)),
            rule("Hammer of Nazahn: attach Lightning Greaves", |r| Answer::Pick(r.choices.len() - 1)),
        ],
    );
    perm(&mut g, P0, HAMMER);
    let e = perm(&mut g, P0, "Lightning Greaves");
    assert!(asked_has(&log, "attach Lightning Greaves"), "you were asked");
    assert_eq!(g.perm(e).attached, None);
}

#[test]
fn sigardas_aid_asks_which_creature() {
    let mut g = urdragon_table();
    perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Sigarda's Aid");
    seat_script(&mut g, vec![rule("Sigarda's Aid: attach", text("Birds"))]);
    let e = perm(&mut g, P0, "Lightning Greaves");
    assert_eq!(g.perm(e).attached, Some(b));
}

#[test]
fn brass_squire_attaches_at_instant_speed_where_you_choose() {
    let mut g = urdragon_table();
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    let squire = perm(&mut g, P0, "Brass Squire");
    let e = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(e).attached = Some(a);
    // in the opponent's turn, with something on the stack, and in your second main phase: not main phase 1 only
    g.step = Step::Main2;
    seat_script(
        &mut g,
        vec![rule("which ability", first), rule("which Equipment", first), rule("to which creature", text("Birds"))],
    );
    let i = index_of(&g, P0, squire);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert_eq!(why, None);
    assert_eq!(g.perm(e).attached, Some(b), "moved to the creature the person chose");
    assert!(g.perm(squire).tapped);
}

// ------------------------------------------------------------------ Peter Parker's Camera
#[test]
fn the_camera_copies_godos_enter_trigger() {
    let mut g = urdragon_table();
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    pool(&mut g, P0, &[('C', 2)]);
    let ci = index_of(&g, P0, cam);
    let used = Arc::new(Mutex::new(false));
    let used2 = used.clone();
    let log = seat_script(
        &mut g,
        vec![
            // Godo's trigger is on the stack: you have priority; use the Camera on it
            rule("Your trigger: Godo", move |_| {
                *used2.lock().unwrap() = true;
                Answer::Action(json!({"do": "use", "perm": ci}))
            }),
            rule("Camera: which ability", first),
            rule("Peter Parker's Camera: copy which", first),
            // the copy first (Hammer), then the original (Helm)
            rule("search your library for an Equipment", first),
            rule("search for an Equipment card", text("Hammer")),
            rule("Hammer of Nazahn: attach", text("Godo")),
            rule("search your library for an Equipment", first),
            rule("search for an Equipment card", text("Helm")),
            rule("Hammer of Nazahn: attach", text("Godo")),
        ],
    );
    let godo = perm(&mut g, P0, GODO);
    let log = log.lock().unwrap().clone();
    assert!(*used.lock().unwrap(), "{log:?}");
    assert!(has_perm(&g, P0, HAMMER) && has_perm(&g, P0, HELM), "both fetched: {log:?}");
    let helm = perms_named(&g, P0, HELM)[0];
    assert_eq!(g.perm(helm).attached, Some(godo), "Hammer attached the Helm");
    assert!(g.perm(cam).tapped);
    assert_eq!(g.perm(cam).data.int(sim_core::state::DataKey::Counters), 2, "a film counter was removed");
    assert_eq!(pool_total(&g, P0), 0, "{{2}} paid");
}

#[test]
fn the_camera_needs_one_of_your_abilities_on_the_stack() {
    let mut g = urdragon_table();
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    pool(&mut g, P0, &[('C', 2)]);
    let i = index_of(&g, P0, cam);
    seat_script(&mut g, vec![rule("which ability", first)]);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert!(why.is_some_and(|w| w.contains("on the stack")));
    assert!(!g.perm(cam).tapped);
}

// ------------------------------------------------------------------ Jaxis, the Troublemaker
#[test]
fn jaxis_copies_the_creature_you_choose_and_you_choose_the_discard() {
    let mut g = urdragon_table();
    let birds = perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Llanowar Elves");
    let jaxis = perm(&mut g, P0, "Jaxis, the Troublemaker");
    let junk = hand(&mut g, P0, &["Swords to Plowshares", "Rampant Growth"]);
    pool(&mut g, P0, &[('R', 1)]);
    seat_script(
        &mut g,
        vec![rule("which ability", first), rule("Jaxis: copy which", text("Birds")), rule("Jaxis: discard", text("Rampant"))],
    );
    let i = index_of(&g, P0, jaxis);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert_eq!(why, None);
    assert!(g.player(P0).gy.contains(&junk[1]) && g.player(P0).hand.contains(&junk[0]), "the chosen card was discarded");
    let toks: Vec<PermId> = perms_named(&g, P0, "Birds of Paradise").into_iter().filter(|&m| m != birds).collect();
    assert_eq!(toks.len(), 1, "a token copy of Birds");
    assert!(g.perm(toks[0]).token && !g.perm(toks[0]).sick, "with haste");
    assert!(g.perm(jaxis).tapped);
}

// ------------------------------------------------------------------ Twinflame and Dualcaster Mage
#[test]
fn twinflame_makes_hasty_token_copies_that_leave_at_the_end() {
    let mut g = urdragon_table();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let c = hand(&mut g, P0, &["Twinflame"])[0];
    pool(&mut g, P0, &[('R', 2)]);
    seat_script(&mut g, vec![rule("Twinflame: target creature #1", first), rule("Twinflame: target creature #2", |_| Answer::Decline)]);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "cast", "card": 0})).unwrap();
    assert_eq!(why, None);
    let toks: Vec<PermId> = perms_named(&g, P0, "Llanowar Elves").into_iter().filter(|&m| m != elves).collect();
    assert_eq!(toks.len(), 1);
    assert!(g.perm(toks[0]).token && !g.perm(toks[0]).sick && g.perm(toks[0]).temp, "hasty, exiled at the end step");
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn dualcaster_flashed_in_copies_twinflame_and_the_token_copies_it_again() {
    let mut g = urdragon_table();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let cards = hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    pool(&mut g, P0, &[('R', 6)]);
    let dm_idx = 0usize; // Twinflame has left the hand: Dualcaster Mage is the first card
    let log = seat_script(
        &mut g,
        vec![
            rule("Twinflame: target creature #1", first),
            rule("Twinflame: target creature #2", |_| Answer::Decline),
            // Twinflame is on the stack: you may flash in Dualcaster Mage in response (a window for your own spell)
            rule("You cast Twinflame", move |_| Answer::Action(json!({"do": "cast", "card": dm_idx}))),
            // its enter trigger copies Twinflame
            rule("Dualcaster Mage: copy Twinflame", first),
            // the copy: aim it at the Dualcaster Mage
            rule("The copy of Twinflame: target creature #1", text("Dualcaster")),
        ],
    );
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "cast", "card": 0})).unwrap();
    assert_eq!(why, None, "{:?}", log.lock().unwrap());
    let log = log.lock().unwrap().clone();
    assert!(log.iter().any(|l| l.contains("You cast Twinflame")), "a window to respond to your own spell: {log:?}");
    assert!(log.iter().any(|l| l.contains("Dualcaster Mage: copy Twinflame")), "the enter trigger asked: {log:?}");
    assert!(g.player(P0).gy.contains(&cards[0]), "Twinflame ended in the graveyard");
    let dms = perms_named(&g, P0, "Dualcaster Mage");
    assert!(dms.len() >= 2, "the flashed Dualcaster Mage and the hasty token copy of it: {dms:?}");
    let tok = dms.iter().copied().find(|&m| g.perm(m).token).expect("a token Dualcaster Mage");
    assert!(!g.perm(tok).sick && g.perm(tok).temp);
    // the original Twinflame (aimed at Llanowar Elves) also resolved
    assert!(perms_named(&g, P0, "Llanowar Elves").len() >= 2, "{:?}", elves);
}

#[test]
fn the_dualcaster_loop_makes_enough_hasty_dualcasters_to_attack() {
    let mut g = urdragon_table();
    perm(&mut g, P0, "Llanowar Elves");
    hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    pool(&mut g, P0, &[('R', 6)]);
    let log = seat_script(
        &mut g,
        vec![
            rule("Twinflame: target creature #1", first),
            rule("Twinflame: target creature #2", |_| Answer::Decline),
            rule("You cast Twinflame", |_| Answer::Action(json!({"do": "cast", "card": 0}))),
            rule("Dualcaster Mage: copy Twinflame", first),
            rule("The copy of Twinflame: target creature #1", text("Dualcaster")),
            // the token's trigger offers the loop (the second choice)
            rule("Dualcaster Mage: copy Twinflame", |r| {
                assert!(r.choices.iter().any(|c| c.starts_with("the loop")), "{:?}", r.choices);
                Answer::Pick(1)
            }),
        ],
    );
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "cast", "card": 0})).unwrap();
    assert_eq!(why, None, "{:?}", log.lock().unwrap());
    let dms = perms_named(&g, P0, "Dualcaster Mage");
    let tokens = dms.iter().filter(|&&m| g.perm(m).token && !g.perm(m).sick && g.perm(m).temp).count();
    let need: i32 = [P1, P2].iter().map(|&q| (g.player(q).life + 1) / 2).sum();
    assert!(tokens as i32 >= need, "{tokens} hasty Dualcaster Mages for {need} unblocked hits");
}

#[test]
fn the_loop_is_not_offered_after_combat() {
    let mut g = urdragon_table();
    perm(&mut g, P0, "Llanowar Elves");
    hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    g.step = Step::Main2;
    pool(&mut g, P0, &[('R', 6)]);
    let log = seat_script(
        &mut g,
        vec![
            rule("Twinflame: target creature #1", first),
            rule("Twinflame: target creature #2", |_| Answer::Decline),
            rule("You cast Twinflame", |_| Answer::Action(json!({"do": "cast", "card": 0}))),
            rule("Dualcaster Mage: copy Twinflame", |r| {
                assert!(!r.choices.iter().any(|c| c.starts_with("the loop")), "{:?}", r.choices);
                Answer::Pick(0)
            }),
        ],
    );
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "cast", "card": 0})).unwrap();
    assert!(asked_has(&log, "Dualcaster Mage: copy Twinflame"));
}

// ------------------------------------------------------------------ the Helm loop
/// Godo wearing Helm of the Host; the opponents have no creatures
fn godo_helm(g: &mut Game) -> PermId {
    let godo = perm(g, P0, GODO);
    let helm = perm(g, P0, HELM);
    g.perm_mut(helm).attached = Some(godo);
    g.active = Some(P0);
    godo
}

fn attack_rules(repeat: &'static str) -> Vec<Rule> {
    vec![
        rule("Declare attackers", |r| Answer::Picks((0..r.choices.len()).collect())),
        rule("Attack which player", text("Kinnan")),
        rule("Helm loop: repeat", text(repeat)),
    ]
}

#[test]
fn the_helm_loop_repeats_your_attack_until_the_table_is_dead() {
    let mut g = urdragon_table();
    // no search prompts matter in repeat mode: Godo's token triggers skip theirs
    let log = seat_script(&mut g, {
        let mut r = vec![rule("search your library for an Equipment", |_| Answer::Pick(1))];
        r.extend(attack_rules("yes"));
        r
    });
    let godo = godo_helm(&mut g);
    g.perm_mut(godo).sick = false;
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    let log = log.lock().unwrap().clone();
    let attacks = log.iter().filter(|l| l.starts_with("attack:")).count();
    assert_eq!(attacks, 1, "only the first combat asked for attackers: {attacks} in {log:?}");
    assert!(log.iter().any(|l| l.contains("Helm loop: repeat")), "{log:?}");
    assert!(g.over && g.winner == Some(P0), "the table is dead: {:?}", g.players.iter().map(|q| (q.alive, q.life)).collect::<Vec<_>>());
}

#[test]
fn without_repeat_every_combat_asks_again() {
    let mut g = urdragon_table();
    // decline the repeat: then each combat prompts for attackers; stop after a few
    let n = Arc::new(Mutex::new(0usize));
    let n2 = n.clone();
    let log = Arc::new(Mutex::new(vec![]));
    let log2 = log.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r| {
            log2.lock().unwrap().push(format!("{}: {}", r.kind, r.prompt));
            if r.prompt.starts_with("Declare attackers") {
                let mut k = n2.lock().unwrap();
                *k += 1;
                return if *k >= 3 { Answer::Decline } else { Answer::Picks((0..r.choices.len()).collect()) };
            }
            if r.prompt.contains("Attack which player") {
                return Answer::Pick(0);
            }
            if r.prompt.contains("Helm loop: repeat") {
                return Answer::Pick(1); // no
            }
            if r.prompt.contains("search your library") {
                return Answer::Pick(1);
            }
            Answer::Decline
        })),
    );
    let godo = godo_helm(&mut g);
    g.perm_mut(godo).sick = false;
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(*n.lock().unwrap() >= 3, "asked for attackers in each combat");
}

#[test]
fn the_repeat_is_not_offered_without_the_loop() {
    let mut g = urdragon_table();
    let log = seat_script(
        &mut g,
        vec![
            rule("Declare attackers", |r| Answer::Picks((0..r.choices.len()).collect())),
            rule("Attack which player", first),
        ],
    );
    let godo = perm(&mut g, P0, GODO); // Godo, no Helm
    g.perm_mut(godo).sick = false;
    g.active = Some(P0);
    sim_core::engine::combat::combat(&mut g, P0).ok();
    assert!(!asked_has(&log, "Helm loop"));
}

#[test]
fn combat_celebrant_with_helm_loops_too() {
    let mut g = urdragon_table();
    let log = seat_script(&mut g, {
        let mut r = vec![rule("Combat Celebrant: exert", first)];
        r.extend(attack_rules("yes"));
        r
    });
    let c = perm(&mut g, P0, "Combat Celebrant");
    let helm = perm(&mut g, P0, HELM);
    g.perm_mut(helm).attached = Some(c);
    g.active = Some(P0);
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    let log = log.lock().unwrap().clone();
    assert_eq!(log.iter().filter(|l| l.starts_with("attack:")).count(), 1, "{log:?}");
    assert!(g.over && g.winner == Some(P0), "{:?}", g.players.iter().map(|q| (q.alive, q.life)).collect::<Vec<_>>());
}
