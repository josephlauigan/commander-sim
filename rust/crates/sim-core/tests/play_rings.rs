//! Practice mode: Rings of Brighthearth for the person (JD, 11/03: Basalt Monolith + Rings + Walking Ballista could not
//! combo), the table shortcut for the Basalt loop, and the outlets that spend a huge pool (Walking Ballista, Staff of
//! Domination, Thrasios).

mod play_common;
use play_common::*;
use sim_core::human::mana;

const BASALT: &str = "Basalt Monolith";
const RINGS: &str = "Rings of Brighthearth";

fn idx(g: &Game, p: PlayerId, m: PermId) -> usize {
    g.player(p).perms.iter().position(|&x| x == m).unwrap()
}

fn use_perm(g: &Game, m: PermId) -> F {
    act(json!({"do": "use", "perm": idx(g, P0, m)}))
}

/// `main`, but a script that runs dry says what was asked
fn run(g: &mut Game, p: PlayerId, mut answers: Vec<F>) -> std::sync::Arc<ScriptHuman> {
    answers.push(pass());
    let h = seat(g, p, answers);
    if let Err(e) = sim_core::human::human::human_main(g, p, false) {
        panic!(
            "{e:?}; asked: {:#?}",
            h.asked().iter().map(|r| (r.prompt.clone(), r.choices.len())).collect::<Vec<_>>()
        );
    }
    h
}

fn colourless(g: &Game) -> u32 {
    g.player(P0).pool.m[5]
}

#[test]
fn rings_copies_basalts_untap_and_each_loop_nets_one() {
    let mut g = table(&["thrasios", "cloud"]);
    let basalt = perm(&mut g, P0, BASALT);
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 5)]);
    g.perm_mut(basalt).tapped = true;
    let u = use_perm(&g, basalt);
    let h = run(
        &mut g,
        P0,
        vec![
            u,
            by_text("{3}: untap"),
            pick(0),                                // Rings of Brighthearth: pay {2} to copy it? yes
            act(json!({"do": "tap", "source": 0})), // the copy untapped it: tap it again before the original resolves
            pass(),                                 // ... then let the original resolve (it untaps it again)
            act(json!({"do": "tap", "source": 0})),
        ],
    );
    assert!(
        h.asked().iter().any(|r| r.prompt.starts_with("Rings of Brighthearth: pay {2} to copy")),
        "asked: {:?}",
        h.asked().iter().map(|r| r.prompt.clone()).collect::<Vec<_>>()
    );
    assert!(h.told().is_empty(), "{:?}", h.told());
    // {5} in, {6} out: Basalt tapped for {C}{C}{C} twice
    assert_eq!(colourless(&g), 6, "pool {}", pool_text(&g, P0));
    assert!(g.perm(basalt).tapped);
}

#[test]
fn declining_rings_costs_nothing_and_the_untap_still_works() {
    let mut g = table(&["thrasios", "cloud"]);
    let basalt = perm(&mut g, P0, BASALT);
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 5)]);
    g.perm_mut(basalt).tapped = true;
    let u = use_perm(&g, basalt);
    run(&mut g, P0, vec![u, by_text("{3}: untap"), pick(1)]); // no
    assert_eq!(colourless(&g), 2);
    assert!(!g.perm(basalt).tapped);
}

#[test]
fn the_loop_shortcut_pays_five_and_adds_the_mana_asked_for() {
    let mut g = table(&["thrasios", "cloud"]);
    let basalt = perm(&mut g, P0, BASALT);
    let rings = perm(&mut g, P0, RINGS);
    // Basalt untapped (its {C}{C}{C}) plus {2} more is {5} in all
    pool(&mut g, P0, &[('C', 2)]);
    assert_eq!(mana::available(&g, P0), 5);
    let u = use_perm(&g, rings);
    let h = run(&mut g, P0, vec![u, by_text("start the loop"), by_text("999")]);
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert_eq!(colourless(&g), 999, "pool {}", pool_text(&g, P0));
    assert!(g.perm(basalt).tapped, "Basalt ends tapped, as the real loop leaves it");
}

#[test]
fn the_loop_needs_five_mana_in_all() {
    let mut g = table(&["thrasios", "cloud"]);
    let basalt = perm(&mut g, P0, BASALT);
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 1)]); // 3 + 1: not enough
    let u = use_perm(&g, basalt);
    let h = run(&mut g, P0, vec![u, cancel()]);
    let asked = h.asked();
    let which = asked.iter().find(|r| r.prompt.contains("which ability")).expect("asked which ability");
    assert!(!which.choices.iter().any(|c| c.contains("start the loop")), "{:?}", which.choices);
    assert!(which.choices.iter().any(|c| c.contains("{3}: untap")));
}

#[test]
fn ballista_cast_for_a_big_x_pings_the_table_dead() {
    let mut g = table(&["thrasios", "cloud"]);
    perm(&mut g, P0, BASALT);
    perm(&mut g, P0, RINGS);
    let hc = hand(&mut g, P0, &["Walking Ballista"]);
    assert_eq!(hc.len(), 1);
    pool(&mut g, P0, &[('C', 999)]);
    let name1 = g.player(P1).name.to_string();
    let life = g.player(P1).life;
    let n = life.max(1);
    let hand_i = g.player(P0).hand.iter().position(|&c| c == hc[0]).unwrap();
    let bal_label = format!("{n} counter"); // "40 counters: 40 damage" (or "1 counter" for a 1-life opponent)
    let h = run(
        &mut g,
        P0,
        vec![
            act(json!({"do": "cast", "card": hand_i})),
            // {X}{X}: X is the real X now, 2 mana each (2026-11-07; before, the question asked for the mana paid, 999)
            by_text("X = 499"),
            // the permanent's index is only known once it is out: use the last permanent
            Box::new(|_| Answer::Action(json!({"do": "use", "perm": 2}))),
            by_text("remove a +1/+1"),
            Box::new(move |r| {
                Answer::Pick(r.choices.iter().position(|c| c.contains(&name1)).expect("the opponent is a target"))
            }),
            Box::new(move |r| {
                Answer::Pick(
                    r.choices
                        .iter()
                        .position(|c| c.starts_with(&bal_label))
                        .unwrap_or_else(|| panic!("no {bal_label:?} in {:?}", r.choices)),
                )
            }),
            pick(1), // Rings of Brighthearth: no copies for the whole run
        ],
    );
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive, "life {}", g.player(P1).life);
    let bal = perms_named(&g, P0, "Walking Ballista");
    assert_eq!(bal.len(), 1, "Ballista is still there");
    assert_eq!(g.perm(bal[0]).plus, 499 - n, "X = 999 mana makes 499 counters; {n} were removed");
}

#[test]
fn karn_seat_the_loop_uses_the_artifact_mana_too() {
    let mut g = table(&["karn", "cloud"]);
    let basalt = perm(&mut g, P0, BASALT);
    let rings = perm(&mut g, P0, RINGS);
    g.player_mut(P0).pool.art = 2; // Karn's mana: pays for abilities (the loop's start is one)
    assert_eq!(mana::available(&g, P0), 5);
    let u = use_perm(&g, rings);
    let h = run(&mut g, P0, vec![u, by_text("start the loop"), by_text("999")]);
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert_eq!(colourless(&g), 999);
    assert_eq!(g.player(P0).pool.art, 0, "the {{5}} came out of the artifact mana first");
    assert!(g.perm(basalt).tapped);
}

#[test]
fn karn_seat_staff_of_domination_draws_n_cards_with_the_big_pool() {
    let mut g = table(&["karn", "cloud"]);
    let staff = perm(&mut g, P0, "Staff of Domination");
    pool(&mut g, P0, &[('C', 999)]);
    let (lib, hand0) = (g.player(P0).library.len(), g.player(P0).hand.len());
    let u = use_perm(&g, staff);
    let h = run(&mut g, P0, vec![u, by_text("draw N cards"), by_text("draw 5 cards")]);
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert_eq!(g.player(P0).hand.len(), hand0 + 5);
    assert_eq!(g.player(P0).library.len(), lib - 5);
    assert_eq!(
        colourless(&g),
        999 - 6 * 5 + 1,
        "{{5}} to draw and {{1}} to untap, five times over (no untap at first)"
    );
}

#[test]
fn thrasios_can_be_activated_again_and_again() {
    let mut g = table(&["thrasios", "cloud"]);
    let th = perm(&mut g, P0, "Thrasios, Triton Hero");
    pool(&mut g, P0, &[('C', 40)]);
    let (lib, hand0, bf0) = (g.player(P0).library.len(), g.player(P0).hand.len(), g.player(P0).lands.len());
    let u = use_perm(&g, th);
    // the scry each time: leave it on top (pick the first choice)
    let mut answers = vec![u, by_text("activate it N times"), by_text("5 times")];
    answers.extend((0..5).map(|_| pick(0))); // the scry each time: leave it on top
    run(&mut g, P0, answers);
    let gained = (g.player(P0).hand.len() - hand0) + (g.player(P0).lands.len() - bf0);
    assert_eq!(gained, 5, "five reveals: each a land onto the battlefield or a card in hand");
    assert!(g.player(P0).library.len() <= lib - 5, "a fetch land revealed may crack and take one more");
    assert_eq!(pool_total(&g, P0), 40 - 20, "{{4}} each");
}

// ================================================================== model-gaps-b2: Rings copies every ability a person has
// (the sweep over every ability of the owned decks is `practice_rings_probe.rs`; these check the copies' effects)

use std::sync::Arc;

/// a seat that answers every question by rule: (a part of the prompt, a part of the choice to pick); a priority is a
/// pass; anything else the first choice. Rings' question is a "yes" unless a rule says otherwise.
fn smart(g: &mut Game, p: PlayerId, rules: Vec<(&'static str, &'static str)>) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        for (pk, ck) in &rules {
            if r.prompt.contains(pk)
                && let Some(i) = r.choices.iter().position(|c| c.contains(ck))
            {
                return Answer::Pick(i);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(p, h.clone());
    h
}

fn use_it(g: &mut Game, p: PlayerId, m: PermId) -> Option<String> {
    let i = idx(g, p, m);
    sim_core::human::human::apply(g, p, &json!({"do": "use", "perm": i})).unwrap()
}

fn asked(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| r.prompt.clone()).collect()
}

fn names_in_hand(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

fn gy_push(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

#[test]
fn cids_copy_returns_a_second_card_chosen_anew() {
    let mut g = table(&["cloud", "karn"]);
    let cid = perm(&mut g, P0, "Cid, Freeflier Pilot");
    perm(&mut g, P0, RINGS);
    gy_push(&mut g, P0, &["Skullclamp", "Batterskull"]);
    pool(&mut g, P0, &[('C', 6)]);
    let h = smart(
        &mut g,
        P0,
        vec![("Cid: return which", "Skullclamp"), ("The copy of Cid's ability", "Batterskull"), ("Rings", "yes")],
    );
    assert_eq!(use_it(&mut g, P0, cid), None);
    let a = asked(&h);
    assert!(a.iter().any(|p| p.starts_with("Rings of Brighthearth: pay {2} to copy")), "{a:?}");
    assert!(a.iter().any(|p| p.starts_with("The copy of Cid's ability")), "the copy asks for a new target: {a:?}");
    let hand = names_in_hand(&g, P0);
    assert!(hand.contains(&"Skullclamp".to_string()) && hand.contains(&"Batterskull".to_string()), "{hand:?}");
    assert_eq!(pool_total(&g, P0), 6 - 2 - 2, "{{2}} for Cid and {{2}} for Rings");
}

#[test]
fn mind_stones_copy_draws_two_cards() {
    let mut g = table(&["karn", "cloud"]);
    let ms = perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 3)]);
    let hand0 = g.player(P0).hand.len();
    smart(&mut g, P0, vec![]);
    assert_eq!(use_it(&mut g, P0, ms), None);
    assert_eq!(g.player(P0).hand.len(), hand0 + 2, "the ability and its copy each draw a card");
}

#[test]
fn a_fetch_lands_copy_fetches_a_second_land() {
    let mut g = table(&["thrasios", "cloud"]);
    let strand = lands(&mut g, P0, "Flooded Strand", 1, false)[0];
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 3)]);
    let n0 = g.player(P0).lands.len();
    let h = smart(&mut g, P0, vec![]);
    let i = g.player(P0).lands.iter().position(|&x| x == strand).unwrap();
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "land": i})).unwrap();
    assert_eq!(r, None);
    let searches = h.asked().iter().filter(|r| r.prompt == "Flooded Strand: search for a land card").count();
    assert_eq!(searches, 2, "{:?}", asked(&h));
    assert_eq!(g.player(P0).lands.len(), n0 - 1 + 2, "the fetch land is sacrificed; two lands come back");
}

#[test]
fn a_karn_lands_ability_urzas_saga_is_copied() {
    let mut g = table(&["karn", "cloud"]);
    let saga = lands(&mut g, P0, "Urza's Saga", 1, false)[0];
    g.player_mut(P0).sagas.retain(|x| x.0 != saga);
    g.player_mut(P0).sagas.push((saga, 2));
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 6)]);
    let n0 = g.player(P0).perms.len();
    let h = smart(&mut g, P0, vec![]);
    let i = g.player(P0).lands.iter().position(|&x| x == saga).unwrap();
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "land": i})).unwrap();
    assert_eq!(r, None);
    assert!(asked(&h).iter().any(|p| p.starts_with("Rings of Brighthearth: pay {2} to copy")), "{:?}", asked(&h));
    assert_eq!(g.player(P0).perms.len(), n0 + 2, "two Constructs: the ability and its copy");
}

#[test]
fn a_targeted_abilitys_copy_chooses_a_new_target() {
    // Triskelion: 1 damage to any target; the original hits the first opponent, the copy the second
    let mut g = table(&["karn", "cloud", "sauron"]);
    let t = perm(&mut g, P0, "Triskelion");
    perm(&mut g, P0, RINGS);
    g.perm_mut(t).plus = 3;
    pool(&mut g, P0, &[('C', 3)]);
    let (n1, n2) = (g.player(P1).name.to_string(), g.player(P2).name.to_string());
    let (n1, n2): (&'static str, &'static str) = (Box::leak(n1.into_boxed_str()), Box::leak(n2.into_boxed_str()));
    let h = smart(
        &mut g,
        P0,
        vec![("Triskelion: 1 damage", n1), ("The copy of Triskelion's ability", n2), ("Rings", "yes")],
    );
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    assert_eq!(use_it(&mut g, P0, t), None);
    assert!(asked(&h).iter().any(|p| p.starts_with("The copy of Triskelion's ability")), "{:?}", asked(&h));
    assert_eq!(g.player(P1).life, l1 - 1, "the original hit the first opponent");
    assert_eq!(g.player(P2).life, l2 - 1, "the copy hit the new target");
}

/// P1 (a person too) holds an Azorius Guildmage and counters the one ability of P0's whose name does (not) start with
/// "copy of"
fn guildmage_seat(g: &mut Game, counter_copy: bool) -> Arc<ScriptHuman> {
    let gm = perm(g, P1, "Azorius Guildmage");
    pool(g, P1, &[('U', 2), ('C', 2)]);
    let gi = idx(g, P1, gm);
    let mut used = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            if !used && r.prompt.contains("activates") {
                used = true;
                return Answer::Action(json!({"do": "use", "perm": gi}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let pick_named = |want_copy: bool| {
            r.choices.iter().position(|c| c.contains("copy of") == want_copy && c.contains("draw")).unwrap_or(0)
        };
        if r.prompt.contains("counter which ability") {
            return Answer::Pick(pick_named(counter_copy));
        }
        if r.prompt.ends_with("which ability?") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("counter target")).unwrap_or(0));
        }
        Answer::Pick(0)
    }));
    g.humans.add(P1, h.clone());
    h
}

#[test]
fn the_original_countered_the_copy_still_resolves() {
    let mut g = table(&["karn", "cloud"]);
    let ms = perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 3)]);
    let hand0 = g.player(P0).hand.len();
    smart(&mut g, P0, vec![]);
    let h1 = guildmage_seat(&mut g, false);
    assert_eq!(use_it(&mut g, P0, ms), None);
    assert!(
        h1.asked().iter().any(|r| r.prompt.contains("counter which ability")),
        "P1 chose what to counter: {:?}",
        asked(&h1)
    );
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "the original was countered; its copy drew the card");
}

#[test]
fn the_copy_countered_the_original_still_resolves() {
    let mut g = table(&["karn", "cloud"]);
    let ms = perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, RINGS);
    pool(&mut g, P0, &[('C', 3)]);
    let hand0 = g.player(P0).hand.len();
    smart(&mut g, P0, vec![]);
    let h1 = guildmage_seat(&mut g, true);
    assert_eq!(use_it(&mut g, P0, ms), None);
    assert!(h1.asked().iter().any(|r| r.prompt.contains("counter which ability")), "{:?}", asked(&h1));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "the copy was countered; the original drew the card");
}

#[test]
fn a_peter_parkers_camera_copy_of_a_converted_ability_chooses_a_new_target() {
    let mut g = table(&["sauron", "karn"]);
    let cid = perm(&mut g, P0, "Cid, Freeflier Pilot");
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    gy_push(&mut g, P0, &["Skullclamp", "Batterskull"]);
    pool(&mut g, P0, &[('C', 8)]);
    let ci = idx(&g, P0, cam);
    let mut used = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            if !used && r.prompt.contains("You activate") {
                used = true;
                return Answer::Action(json!({"do": "use", "perm": ci}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let pick = |w: &str| r.choices.iter().position(|c| c.contains(w));
        if r.prompt.contains("Cid: return which") {
            return Answer::Pick(pick("Skullclamp").unwrap_or(0));
        }
        if r.prompt.contains("The copy of Cid's ability") {
            return Answer::Pick(pick("Batterskull").unwrap_or(0));
        }
        if r.prompt.ends_with("which ability?") && r.prompt.starts_with("Peter Parker") {
            return Answer::Pick(0);
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    assert_eq!(use_it(&mut g, P0, cid), None);
    let a = asked(&h);
    assert!(a.iter().any(|p| p.contains("copy which ability")), "the Camera was used on Cid's ability: {a:?}");
    assert!(a.iter().any(|p| p.starts_with("The copy of Cid's ability")), "{a:?}");
    let hand = names_in_hand(&g, P0);
    assert!(hand.contains(&"Skullclamp".to_string()) && hand.contains(&"Batterskull".to_string()), "{hand:?}");
}

#[test]
fn a_loyalty_abilitys_copy_pays_no_loyalty_and_asks_for_its_target_again() {
    let mut g = table(&["karn", "cloud"]);
    let karn = perm(&mut g, P0, "Karn Liberated");
    perm(&mut g, P0, RINGS);
    let a = perm(&mut g, P1, "Serra Angel");
    let b = perm(&mut g, P1, "Vampire Nighthawk");
    pool(&mut g, P0, &[('C', 3)]);
    let loyalty0 = g.perm(karn).loyalty.unwrap();
    let mut nth = 0;
    let (na, nb) = (g.perm(a).name.to_string(), g.perm(b).name.to_string());
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.ends_with("which ability?") {
            return Answer::Pick(r.choices.iter().position(|c| c.starts_with("-3")).unwrap());
        }
        if r.prompt.starts_with("Karn Liberated -3: exile which permanent?") {
            nth += 1;
            let want = if nth == 1 { &na } else { &nb };
            return Answer::Pick(r.choices.iter().position(|c| c.starts_with(want.as_str())).unwrap());
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    assert_eq!(use_it(&mut g, P0, karn), None);
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "the original exiled one, the copy the other");
    assert_eq!(g.perm(karn).loyalty.unwrap(), loyalty0 - 3, "loyalty is paid once, for the original");
    assert_eq!(asked(&h).iter().filter(|p| p.starts_with("Karn Liberated -3: exile which")).count(), 2);
}
