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
        panic!("{e:?}; asked: {:#?}", h.asked().iter().map(|r| (r.prompt.clone(), r.choices.len())).collect::<Vec<_>>());
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
            pick(0),                          // Rings of Brighthearth: pay {2} to copy it? yes
            act(json!({"do": "tap", "source": 0})), // the copy untapped it: tap it again before the original resolves
            pass(),                           // ... then let the original resolve (it untaps it again)
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
            by_text("X = 999"),
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
    assert_eq!(colourless(&g), 999 - 6 * 5 + 1, "{{5}} to draw and {{1}} to untap, five times over (no untap at first)");
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
