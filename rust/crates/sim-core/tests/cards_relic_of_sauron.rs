//! Relic of Sauron ({4} artifact): `{T}: Add two mana in any combination of {U}, {B} and/or {R}.`
//! `{3}, {T}: Draw two cards, then discard a card.` In JD's `urdragon` deck.

use sim_core::engine::mana;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    let mut g =
        table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
    g.active = Some(P0);
    g
}

#[test]
fn the_card_data_reads_two_mana_in_blue_black_red() {
    let g = tbl();
    let c = card(&g, "Relic of Sauron");
    assert_eq!(
        g.db.get(c).tags.str(sim_core::tag::Tag::Rock),
        Some("2:UBR"),
        "two mana, not the three symbols in the text"
    );
}

#[test]
fn it_taps_for_two_mana_in_any_combination_of_blue_black_and_red_only() {
    let mut g = tbl();
    perm(&mut g, P0, "Relic of Sauron");
    assert_eq!(mana::total_mana(&g, P0, false), 2);
    for pips in ["UB", "UR", "BR", "UU", "BB", "RR", "R", "B"] {
        assert!(mana::can_pay(&g, P0, 0, pips, false), "{pips}");
    }
    assert!(mana::can_pay(&g, P0, 2, "", false), "two generic");
    assert!(!mana::can_pay(&g, P0, 0, "W", false), "no white");
    assert!(!mana::can_pay(&g, P0, 0, "G", false), "no green");
    assert!(!mana::can_pay(&g, P0, 1, "UB", false), "only two mana");
}

#[test]
fn the_ai_loots_when_three_other_mana_pay_for_it_and_not_before() {
    use sim_core::dsl::{ability_options, activate};
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    lands(&mut g, P0, "Mountain", 2, false);
    let opts = ability_options(&mut g, P0, true).unwrap();
    assert!(
        !opts.iter().any(|o| o.label == "Relic of Sauron ability"),
        "two lands cannot pay {{3}}: its own mana does not count"
    );
    assert!(!activate(&mut g, P0, relic, 0).unwrap(), "and an activation is refused too");
    assert!(!g.perm(relic).tapped);
    lands(&mut g, P0, "Mountain", 1, false);
    let opts = ability_options(&mut g, P0, true).unwrap();
    let o = opts.iter().find(|o| o.label == "Relic of Sauron ability").expect("the loot is on offer");
    assert!(o.utility > 0.0);
    let (hand0, gy0) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    assert!(activate(&mut g, P0, relic, 0).unwrap());
    assert!(g.perm(relic).tapped);
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "all three lands paid the {{3}}");
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "two drawn, one discarded");
    assert_eq!(g.player(P0).gy.len(), gy0 + 1);
}

#[test]
fn a_person_sees_the_loot_ability() {
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    let ab = sim_core::dsl::human_abilities(&g, P0, relic);
    assert_eq!(ab.len(), 1);
    assert!(ab[0].0.contains("draw") && ab[0].0.contains("discard"), "{}", ab[0].0);
}

/// a person's pool: the source asked for the first colour, then the person picks the other
fn person(g: &mut Game, second: &'static str) {
    use sim_core::control::Answer;
    use sim_core::human::script::ScriptHuman;
    use std::sync::Arc;
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r| {
            if r.prompt.contains("choose the colour of the next mana") {
                return match r.choices.iter().position(|c| c == second) {
                    Some(i) => Answer::Pick(i),
                    None => Answer::Decline,
                };
            }
            Answer::Pick(0)
        })),
    );
}

fn relic_id(g: &Game) -> i64 {
    sim_core::human::mana::sources(g, P0).into_iter().find(|x| x.name == "Relic of Sauron").expect("a source").id as i64
}

#[test]
fn a_person_can_tap_it_for_a_mix_of_colours() {
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    person(&mut g, "B");
    let id = relic_id(&g);
    assert_eq!(sim_core::human::mana::tap(&mut g, P0, id, Some('U')).unwrap(), None);
    assert!(g.perm(relic).tapped);
    let pool = g.player(P0).pool.clone();
    assert!(pool.can_pay(&g, P0, 0, "UB"), "{{U}}{{B}}");
    assert!(!pool.can_pay(&g, P0, 0, "UU"), "not {{U}}{{U}}");
    // and the same colour twice when the person picks it
    let mut g = tbl();
    perm(&mut g, P0, "Relic of Sauron");
    person(&mut g, "R");
    let id = relic_id(&g);
    sim_core::human::mana::tap(&mut g, P0, id, Some('R')).unwrap();
    assert!(g.player(P0).pool.clone().can_pay(&g, P0, 0, "RR"));
}

#[test]
fn cancelling_the_second_colour_taps_nothing() {
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    person(&mut g, "W"); // not offered: the script declines
    let id = relic_id(&g);
    let why = sim_core::human::mana::tap(&mut g, P0, id, Some('U')).unwrap();
    assert!(why.is_some_and(|w| w.starts_with("Cancelled")));
    assert!(!g.perm(relic).tapped);
}

#[test]
fn a_person_loots_with_three_mana_in_the_pool_and_not_after_tapping_it_for_mana() {
    use sim_core::dsl::activate;
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    person(&mut g, "B");
    g.player_mut(P0).pool.add('R', 3);
    let (hand0, gy0) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    assert!(activate(&mut g, P0, relic, 0).unwrap());
    assert!(g.perm(relic).tapped);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "two drawn, one discarded");
    assert_eq!(g.player(P0).gy.len(), gy0 + 1);
    // tapped for mana first, it cannot also pay the {T} cost
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    person(&mut g, "B");
    let id = relic_id(&g);
    sim_core::human::mana::tap(&mut g, P0, id, Some('U')).unwrap();
    g.player_mut(P0).pool.add('R', 3);
    assert!(!activate(&mut g, P0, relic, 0).unwrap());
}

/// the loot ability as a person plays it: they tap three lands for {R}{R}{R}, then use the Relic: `{3}, {T}: Draw two
/// cards, then discard a card.` The person is asked what to discard, from the hand as it is after the draw (the two
/// new cards included)
#[test]
fn a_person_loots_with_three_lands_draws_two_and_is_asked_what_to_discard() {
    use sim_core::control::{Answer, Request};
    use sim_core::human::script::ScriptHuman;
    use std::sync::{Arc, Mutex};
    let mut g = tbl();
    let relic = perm(&mut g, P0, "Relic of Sauron");
    let lands = lands(&mut g, P0, "Mountain", 3, false);
    hand(&mut g, P0, &["Sol Ring", "Brainstorm"]);
    // tap the three Mountains for mana (not the Relic)
    // (the source ids shift as sources are used: look each one up again)
    while let Some(s) = sim_core::human::mana::sources(&g, P0).into_iter().find(|s| s.name == "Mountain") {
        assert_eq!(sim_core::human::mana::tap(&mut g, P0, s.id as i64, None).unwrap(), None);
    }
    assert!(lands.iter().all(|&l| g.perm(l).tapped) && !g.perm(relic).tapped);
    assert_eq!(g.player(P0).pool.total(), 3);
    let at = g.player(P0).perms.iter().position(|&m| m == relic).unwrap();
    let asked: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(vec![]));
    let (a2, mut used) = (asked.clone(), false);
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r| {
            a2.lock().unwrap().push(r.clone());
            if &*r.kind == "priority" {
                let first = !std::mem::replace(&mut used, true);
                return Answer::Action(if first {
                    serde_json::json!({"do": "use", "perm": at})
                } else {
                    serde_json::json!({"do": "pass"})
                });
            }
            Answer::Pick(0)
        })),
    );
    let lib0 = g.player(P0).library.len();
    let gy0 = g.player(P0).gy.len();
    let _ = sim_core::human::human::human_main(&mut g, P0, false);
    assert!(g.perm(relic).tapped, "{{T}} paid");
    assert_eq!(g.player(P0).pool.total(), 0, "the {{3}} was paid from the three mana");
    assert_eq!(g.player(P0).library.len(), lib0 - 2, "two cards drawn");
    let asked = asked.lock().unwrap();
    let discards: Vec<&Request> = asked.iter().filter(|r| r.prompt.contains("Discard a card")).collect();
    assert_eq!(discards.len(), 1, "asked once what to discard: {asked:?}");
    // the hand after the draw: the two it held and the two drawn: four cards to choose from
    assert_eq!(discards[0].choices.len(), 4, "{:?}", discards[0].choices);
    assert!(discards[0].choices.iter().any(|c| c.contains("Sol Ring")));
    assert!(discards[0].choices.iter().any(|c| c.contains("Brainstorm")));
    assert_eq!(
        (g.player(P0).hand.len(), g.player(P0).gy.len()),
        (3, gy0 + 1),
        "four after the draw, one discarded: three left"
    );
}

/// Oracle: "Add two mana in any combination of {U}, {B} and/or {R}": the source is listed as two mana of U, B or R,
/// and the second mana's colour is offered as exactly those three, nothing else
#[test]
fn a_person_is_offered_exactly_blue_black_and_red_for_each_mana() {
    use sim_core::control::{Answer, Request};
    use sim_core::human::script::ScriptHuman;
    use std::sync::{Arc, Mutex};
    let mut g = tbl();
    perm(&mut g, P0, "Relic of Sauron");
    let src = sim_core::human::mana::sources(&g, P0).into_iter().find(|x| x.name == "Relic of Sauron").unwrap();
    assert_eq!((src.amount, src.colours.as_str()), (2, "UBR"), "two mana, each blue, black or red");
    let asked: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(vec![]));
    let a2 = asked.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r| {
            a2.lock().unwrap().push(r.clone());
            Answer::Pick(0)
        })),
    );
    sim_core::human::mana::tap(&mut g, P0, src.id as i64, Some('R')).unwrap();
    let asked = asked.lock().unwrap();
    assert_eq!(asked.len(), 1, "the first mana's colour came with the tap; one question for the second: {asked:?}");
    assert_eq!(asked[0].choices, vec!["U", "B", "R", "cancel"]);
}
