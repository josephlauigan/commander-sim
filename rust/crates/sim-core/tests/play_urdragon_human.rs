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

/// the Godo package moved to Sauron in the 11/03 Ur-Dragon rebuild: the tests whose searches need Godo's Equipment in
/// the library (Helm of the Host, Hammer of Nazahn, Whispersilk Cloak) sit at the Sauron seat
fn sauron_table() -> Game {
    table(&["sauron", "kinnan-simic-mana-combo", "urza-mono-blue-artifacts"])
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

#[test]
fn brass_person_selects_any_number_from_all_graveyards_within_the_roll() {
    for (chosen, prompts) in [
        (vec![], 1),
        (vec!["Birds of Paradise"], 2),
        (vec!["Shivan Dragon", "Ornithopter"], 2),
        (vec!["Birds of Paradise", "Llanowar Elves", "Ornithopter"], 3),
    ] {
        let mut g = urdragon_table();
        g.log = Some(vec![]);
        let brass = perm(&mut g, P0, "Ancient Brass Dragon");
        let bird = card(&g, "Birds of Paradise");
        let elf = card(&g, "Llanowar Elves");
        let zero = card(&g, "Ornithopter");
        let dragon = card(&g, "Shivan Dragon");
        let big = card(&g, "Blightsteel Colossus");
        let rock = card(&g, "Sol Ring");
        g.player_mut(P0).gy = vec![elf, zero, rock];
        g.player_mut(P1).gy = vec![bird, dragon, big];
        g.rng = (0..).map(sim_core::rng::Rng::from_u64).find(|r| r.clone().below(20) == 5).unwrap();
        let mut expected_rng = g.rng.clone();
        assert_eq!(expected_rng.below(20) + 1, 6);
        let expected = chosen.clone();
        let mut next = 0;
        let h = Arc::new(ScriptHuman::from_fn(move |r| {
            if r.prompt.starts_with("Ancient Brass Dragon: rolled") {
                assert!(r.prompt.contains("rolled 6;"));
                assert!(!r.choices.iter().any(|s| s.contains("Blightsteel") || s.contains("Sol Ring")));
                if next > 0 {
                    assert!(
                        !r.choices.iter().any(|s| s.contains("Shivan Dragon")),
                        "remaining budget excludes six mana"
                    );
                }
                if let Some(name) = chosen.get(next) {
                    next += 1;
                    Answer::Pick(r.choices.iter().position(|s| s.starts_with(name)).unwrap())
                } else {
                    Answer::Decline
                }
            } else if r.kind == "priority" {
                Answer::Action(json!({"do":"pass"}))
            } else {
                Answer::Decline
            }
        }));
        g.humans.add(P0, h.clone());
        let f = g.registry.get(card(&g, "Ancient Brass Dragon")).unwrap().combat_damage.unwrap();
        f(&mut g, brass, P0, brass, P1, 7).unwrap();
        assert_eq!(g.rng, expected_rng, "exactly one d20 roll");
        assert!(g.log.as_ref().unwrap().iter().any(|s| s.contains("Ancient Brass Dragon rolls 6")));
        for (c, q) in [(elf, P0), (zero, P0), (bird, P1), (dragon, P1)] {
            let name = &*g.db.get(c).name;
            let picked = expected.contains(&name);
            assert_eq!(has_perm(&g, P0, name), picked);
            assert_eq!(g.player(q).gy.contains(&c), !picked);
            if picked && q != P0 {
                let m = perms_named(&g, P0, name)[0];
                assert_eq!(g.perm(m).orig, q);
            }
        }
        assert!(g.player(P1).gy.contains(&big) && g.player(P0).gy.contains(&rock));
        assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Ancient Brass Dragon: rolled")).count(), prompts);
    }
}

#[test]
fn brass_person_targets_are_not_replaced_by_ai_picks_after_graveyard_hate() {
    let mut g = urdragon_table();
    let brass = perm(&mut g, P0, "Ancient Brass Dragon");
    let crypt = perm(&mut g, P2, "Tormod's Crypt");
    let dragon = card(&g, "Shivan Dragon");
    let bird = card(&g, "Birds of Paradise");
    g.player_mut(P0).gy = vec![dragon];
    g.player_mut(P1).gy = vec![bird];
    g.rng = (0..).map(sim_core::rng::Rng::from_u64).find(|r| r.clone().below(20) == 5).unwrap();
    let log = seat_script(&mut g, vec![rule("Ancient Brass Dragon: rolled", text("Shivan Dragon"))]);
    let f = g.registry.get(card(&g, "Ancient Brass Dragon")).unwrap().combat_damage.unwrap();
    f(&mut g, brass, P0, brass, P1, 7).unwrap();
    assert!(asked_has(&log, "Ancient Brass Dragon: rolled 6"));
    assert!(!g.perm(crypt).on_bf);
    assert!(g.player(P0).exile.contains(&dragon));
    assert!(g.player(P1).gy.contains(&bird));
    assert!(!has_perm(&g, P0, "Shivan Dragon") && !has_perm(&g, P0, "Birds of Paradise"));
}

#[test]
fn brass_interleaved_person_picks_offer_graveyard_hate_once_per_owner() {
    let mut g = urdragon_table();
    let brass = perm(&mut g, P0, "Ancient Brass Dragon");
    let crypts: Vec<_> = (0..3).map(|_| perm(&mut g, P2, "Tormod's Crypt")).collect();
    let dragon = card(&g, "Shivan Dragon");
    let elf = card(&g, "Llanowar Elves");
    let zero = card(&g, "Ornithopter");
    g.player_mut(P1).gy = vec![dragon, elf];
    g.player_mut(P2).gy = vec![zero];
    g.rng = (0..).map(sim_core::rng::Rng::from_u64).find(|r| r.clone().below(20) == 6).unwrap();
    let log = seat_script(
        &mut g,
        vec![
            rule("Ancient Brass Dragon: rolled", text("Shivan Dragon")), // P1
            rule("Ancient Brass Dragon: rolled", text("Ornithopter")),   // P2
            rule("Ancient Brass Dragon: rolled", text("Llanowar Elves")), // P1 again
        ],
    );
    let f = g.registry.get(card(&g, "Ancient Brass Dragon")).unwrap().combat_damage.unwrap();
    f(&mut g, brass, P0, brass, P1, 7).unwrap();
    assert!(asked_has(&log, "Ancient Brass Dragon: rolled 7"));
    assert_eq!(
        crypts.iter().filter(|&&m| g.perm(m).on_bf).count(),
        1,
        "two distinct graveyards get one response each; P1 must not consume the third Crypt"
    );
    assert!(g.player(P1).exile.contains(&dragon) && g.player(P1).exile.contains(&elf));
    assert!(g.player(P2).exile.contains(&zero));
}

#[test]
fn brass_probe_does_not_ask_roll_or_change_graveyards_and_rolls_even_without_targets() {
    let mut g = urdragon_table();
    let brass = perm(&mut g, P0, "Ancient Brass Dragon");
    let bird = card(&g, "Birds of Paradise");
    g.player_mut(P1).gy = vec![bird];
    let h = Arc::new(ScriptHuman::from_fn(|_| panic!("no question before the trigger window")));
    g.humans.add(P0, h.clone());
    let f = g.registry.get(card(&g, "Ancient Brass Dragon")).unwrap().combat_damage.unwrap();
    let before = g.clone();
    g.trig_probe = true;
    assert!(matches!(f(&mut g, brass, P0, brass, P1, 7), Err(sim_core::flow::Stop::Probe(_))));
    assert_eq!(g.rng, before.rng);
    assert_eq!(g.player(P1).gy, before.player(P1).gy);
    assert_eq!(g.player(P0).perms, before.player(P0).perms);
    assert!(h.asked().is_empty());
    for q in [P0, P1, P2] {
        g.player_mut(q).gy.clear();
    }
    g.humans = Default::default();
    let mut expected_rng = g.rng.clone();
    expected_rng.below(20);
    f(&mut g, brass, P0, brass, P1, 7).unwrap();
    assert_eq!(g.rng, expected_rng, "Oracle rolls even with empty graveyards");
}

#[test]
fn silumgar_person_is_asked_once_and_can_choose_a_planeswalker() {
    let mut g = urdragon_table();
    perm(&mut g, P1, "Blightsteel Colossus");
    let walker = perm(&mut g, P1, "Jace, the Mind Sculptor");
    let log = seat_script(&mut g, vec![rule("Dragonlord Silumgar: gain control of which", text("Jace"))]);
    let src = perm(&mut g, P0, "Dragonlord Silumgar");
    assert_eq!(g.perm(walker).owner, P0);
    assert_eq!(
        log.lock().unwrap().iter().filter(|s| s.contains("Dragonlord Silumgar: gain control of which")).count(),
        1
    );
    sim_core::engine::zones::leave(&mut g, src).unwrap();
    assert_eq!(g.perm(walker).owner, P1);
}

#[test]
fn silumgar_target_choices_respect_shroud_hexproof_protection_and_phasing() {
    let mut g = urdragon_table();
    let own = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(own).eot_kw.push("hexproof");
    for (name, kw) in [("Birds of Paradise", "shroud"), ("Noble Hierarch", "hexproof"), ("Shivan Dragon", "pro_U")] {
        let m = perm(&mut g, P1, name);
        g.perm_mut(m).eot_kw.push(kw);
    }
    let own_shroud = perm(&mut g, P0, "Fyndhorn Elves");
    g.perm_mut(own_shroud).eot_kw.push("shroud");
    let phased = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(phased).phased = true;
    let log = seat_script(
        &mut g,
        vec![rule("Dragonlord Silumgar: gain control of which", |r| {
            assert!(r.choices.iter().any(|s| s.contains("Llanowar Elves")), "own hexproof is legal");
            assert!(r.choices.iter().any(|s| s.contains("Dragonlord Silumgar")), "may target itself");
            for name in ["Birds of Paradise", "Noble Hierarch", "Shivan Dragon", "Fyndhorn Elves", "Grave Titan"] {
                assert!(!r.choices.iter().any(|s| s.contains(name)), "illegal target {name}");
            }
            Answer::Pick(r.choices.iter().position(|s| s.contains("Llanowar Elves")).unwrap())
        })],
    );
    perm(&mut g, P0, "Dragonlord Silumgar");
    assert!(asked_has(&log, "Dragonlord Silumgar: gain control of which"));
}

#[test]
fn tiamat_person_searches_up_to_five_distinct_names_and_reveals_each() {
    let mut g = urdragon_table();
    g.log = Some(vec![]);
    let gold = card(&g, "Goldspan Dragon");
    let tiamat = card(&g, "Tiamat");
    let copper = card(&g, "Ancient Copper Dragon");
    g.player_mut(P0).library = vec![gold, gold, tiamat, copper];
    let log = seat_script(
        &mut g,
        vec![rule("Tiamat: search", text("Goldspan")), rule("Tiamat: search", text("find nothing"))],
    );
    sim_core::engine::zones::enter(
        &mut g,
        P0,
        tiamat,
        sim_core::engine::zones::Enter { was_cast: true, ..Default::default() },
    )
    .unwrap();
    assert_eq!(g.player(P0).hand.iter().filter(|&&c| c == gold).count(), 1);
    assert!(!g.player(P0).hand.contains(&tiamat) && !g.player(P0).hand.contains(&copper));
    assert!(asked_has(&log, "Tiamat: search"));
    assert!(g.log.as_ref().unwrap().iter().any(|s| s.contains("reveals Goldspan Dragon")));
}

#[test]
fn hellkite_charger_person_can_pay_for_an_extra_combat_with_only_six_power_attacking() {
    let mut g = urdragon_table();
    let src = perm(&mut g, P0, "Hellkite Charger");
    g.perm_mut(src).tapped = true;
    pool(&mut g, P0, &[('R', 7)]);
    let log = seat_script(&mut g, vec![rule("Hellkite Charger: pay", |_| Answer::Yes)]);
    let f = g.registry.get(card(&g, "Hellkite Charger")).unwrap().attack.unwrap();
    f(&mut g, src, P0, &[src], P1).unwrap();
    assert!(asked_has(&log, "Hellkite Charger: pay"));
    assert_eq!(pool_total(&g, P0), 0);
    assert_eq!(g.player(P0).extra_combats, 1);
    assert!(!g.perm(src).tapped);
    f(&mut g, src, P0, &[src], P1).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1, "once per combat");
}

#[test]
fn hellkite_charger_failed_red_payment_keeps_sources_and_combat_stamp() {
    for (mountains, floating) in [(0, 0), (1, 0), (0, 6), (1, 6)] {
        let mut g = urdragon_table();
        let src = perm(&mut g, P0, "Hellkite Charger");
        g.perm_mut(src).tapped = true;
        let red = lands(&mut g, P0, "Mountain", mountains, false);
        let blue = lands(&mut g, P0, "Island", 7 - mountains - floating, false);
        pool(&mut g, P0, &[('U', floating as u32)]);
        let before = g.player(P0).pool.clone();
        let log = seat_script(&mut g, vec![rule("Hellkite Charger: pay", |_| Answer::Yes)]);
        let f = g.registry.get(card(&g, "Hellkite Charger")).unwrap().attack.unwrap();
        f(&mut g, src, P0, &[src], P1).unwrap();
        assert!(asked_has(&log, "Hellkite Charger: pay"));
        assert_eq!(g.player(P0).extra_combats, 0);
        assert!(g.perm(src).tapped);
        assert!(red.iter().chain(&blue).all(|&m| !g.perm(m).tapped), "failed payment must not tap lands");
        assert_eq!(g.player(P0).pool, before);
        // The same combat can still pay once the missing red becomes available.
        pool(&mut g, P0, &[('R', 2)]);
        seat_script(&mut g, vec![rule("Hellkite Charger: pay", |_| Answer::Yes)]);
        f(&mut g, src, P0, &[src], P1).unwrap();
        assert_eq!(g.player(P0).extra_combats, 1, "failed payment must not consume the combat stamp");
        assert!(!g.perm(src).tapped);
    }
}

#[test]
fn hellkite_charger_person_can_automatically_tap_lands_with_two_red_sources() {
    let mut g = urdragon_table();
    let src = perm(&mut g, P0, "Hellkite Charger");
    let red = lands(&mut g, P0, "Mountain", 2, false);
    let blue = lands(&mut g, P0, "Island", 5, false);
    seat_script(&mut g, vec![rule("Hellkite Charger: pay", |_| Answer::Yes)]);
    let f = g.registry.get(card(&g, "Hellkite Charger")).unwrap().attack.unwrap();
    f(&mut g, src, P0, &[src], P1).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1);
    assert!(red.iter().chain(&blue).all(|&m| g.perm(m).tapped));
    assert_eq!(pool_total(&g, P0), 0);
}

// ------------------------------------------------------------------ Godo's search
#[test]
fn godo_lets_you_search_for_any_equipment() {
    let mut g = sauron_table();
    let log = seat_script(
        &mut g,
        vec![
            rule("search your library for an Equipment", first),
            rule("search for an Equipment", text("Whispersilk Cloak")),
        ],
    );
    perm(&mut g, P0, GODO);
    assert!(
        has_perm(&g, P0, "Whispersilk Cloak"),
        "any Equipment, not only the one the AI wants: {:?}",
        log.lock().unwrap()
    );
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
    let mut g = sauron_table();
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
        vec![
            rule("which ability", first),
            rule("Jaxis: copy which", text("Birds")),
            rule("Jaxis: discard", text("Rampant")),
        ],
    );
    let i = index_of(&g, P0, jaxis);
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert_eq!(why, None);
    assert!(
        g.player(P0).gy.contains(&junk[1]) && g.player(P0).hand.contains(&junk[0]),
        "the chosen card was discarded"
    );
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
    seat_script(
        &mut g,
        vec![rule("Twinflame: target creature #1", first), rule("Twinflame: target creature #2", |_| Answer::Decline)],
    );
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
    assert!(
        g.over && g.winner == Some(P0),
        "the table is dead: {:?}",
        g.players.iter().map(|q| (q.alive, q.life)).collect::<Vec<_>>()
    );
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

// ------------------------------------------------------------------ the rest of the deck's Equipment engine
#[test]
fn gilgamesh_asks_which_equipment_goes_on_which_samurai() {
    let mut g = urdragon_table();
    // the top six: Lightning Greaves and a Sword; both come out, and you pick the one to attach
    for n in ["Lightning Greaves", "Sword of Fire and Ice"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).library.push(c);
    }
    let log = seat_script(
        &mut g,
        vec![
            rule("look at the top six", first),
            // both, one at a time (each rule answers once; then nothing is left to pick)
            rule("Gilgamesh: put which Equipment", first),
            rule("Gilgamesh: put which Equipment", first),
            rule("Gilgamesh: attach which Equipment", text("Sword of Fire")),
        ],
    );
    let gil = perm(&mut g, P0, "Gilgamesh, Master-at-Arms");
    let sword = perms_named(&g, P0, "Sword of Fire and Ice");
    assert_eq!(sword.len(), 1, "{:?}", log.lock().unwrap());
    assert_eq!(g.perm(sword[0]).attached, Some(gil), "the Sword went on the only Samurai");
    assert!(has_perm(&g, P0, "Lightning Greaves"), "and the Greaves came out too");
    let greaves = perms_named(&g, P0, "Lightning Greaves")[0];
    assert_eq!(g.perm(greaves).attached, None, "but only the one you chose was attached");
}

#[test]
fn the_ur_dragon_lets_you_choose_the_permanent_to_put_onto_the_battlefield() {
    let mut g = urdragon_table();
    let ur = perm(&mut g, P0, "The Ur-Dragon");
    g.perm_mut(ur).sick = false;
    g.active = Some(P0);
    let cards = hand(&mut g, P0, &["Sol Ring", "Mana Vault"]);
    let log = seat_script(
        &mut g,
        vec![
            rule("Declare attackers", |r| Answer::Picks((0..r.choices.len()).collect())),
            rule("Attack which player", first),
            rule("The Ur-Dragon: put a permanent", text("Mana Vault")),
        ],
    );
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(has_perm(&g, P0, "Mana Vault"), "{:?}", log.lock().unwrap());
    assert!(g.player(P0).hand.contains(&cards[0]), "the Sol Ring stayed in hand: you chose");
}

#[test]
fn embercleave_asks_which_creature_it_attaches_to() {
    let mut g = urdragon_table();
    perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    seat_script(&mut g, vec![rule("Embercleave: attach", text("Birds"))]);
    let e = perm(&mut g, P0, "Embercleave");
    assert_eq!(g.perm(e).attached, Some(b));
}
