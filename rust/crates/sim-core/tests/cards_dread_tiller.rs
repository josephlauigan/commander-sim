//! Dread Tiller ({1}{B}{G} artifact creature, 2/4; impls/karn_gilded_cards.rs `tiller_etb`, `tiller_dies`), in the Karn,
//! Gilded Guardian deck. One test for each part of each Oracle line, on a hand-built position; the expectation is the
//! Oracle text, not the code:
//!   - "When this creature enters, put a -1/-1 counter on target creature."
//!   - "Whenever a creature with a -1/-1 counter on it dies, you may put a land card from your hand or graveyard onto the
//!     battlefield tapped."
//!
//! The AI's choices (the target, the land) are heuristics, written down in the card's note in data/card_overrides.toml;
//! the tests that pin them say so.

mod play_common;
use play_common::*;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{repl, zones};
use sim_core::state::Step;
use std::sync::Arc;

const TILLER: &str = "Dread Tiller";
const REAPER: &str = "The Reaper, King No More";
const NORN: &str = "Elesh Norn, Mother of Machines";
const ELVES: &str = "Llanowar Elves"; // a 1/1: the counter kills it
const BEARS: &str = "Ainok Strike Leader"; // a 2/2: the counter shrinks it
const DRAGON: &str = "Glorybringer"; // a 4/4, worth the most of these
const WARDED: &str = "Unflinching Hortimancer"; // a 2/1 with ward {1}

/// P0 (five colours) and P1: P0's turn, main phase one, an empty hand
fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn gy_land(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
}

/// the names of p's lands, in the order they came
fn land_names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.perm(l).card()).name.to_string()).collect()
}

fn n_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.len()
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.contains(&card(g, name))
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.contains(&card(g, name))
}

/// P0 has one land of each colour on the battlefield, so no land adds a colour
fn all_colours(g: &mut Game) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, P0, b, 1, false);
    }
}

/// a person at P0's seat: "which creature?" is answered with the first choice that contains `target`, the land question
/// with the first choice that contains `land` (None: decline), every other request (a window to respond) with a pass
fn person(g: &mut Game, target: &'static str, land: Option<&'static str>) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains("on which creature") {
            Answer::Pick(r.choices.iter().position(|c| c.contains(target)).unwrap_or(0))
        } else if r.prompt.contains("put a land from your hand or graveyard") {
            match land {
                Some(w) => Answer::Pick(r.choices.iter().position(|c| c.contains(w)).expect("the land is offered")),
                None => Answer::Decline,
            }
        } else {
            Answer::Action(json!({"do": "pass"}))
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

// ======================================================== "When this creature enters, put a -1/-1 counter on target creature."
#[test]
fn enters_puts_a_counter_on_the_opposing_creature_worth_most() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    let small = perm(&mut g, P1, BEARS);
    let own = perm(&mut g, P0, "Vampire Nighthawk");
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, -1, "a -1/-1 counter");
    assert_eq!((epow(&g, big), etgh(&g, big)), (3, 3), "4/4 becomes 3/3");
    assert_eq!(g.perm(small).plus, 0);
    assert_eq!(g.perm(own).plus, 0, "never our own while an opposing creature can take it");
    assert_eq!(g.perm(t).plus, 0);
    assert_eq!(n_lands(&g, P0), 0, "nothing died: no land");
}

#[test]
fn enters_target_is_any_creature_so_alone_it_shrinks_itself() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(t).plus, -1, "the only creature is the Tiller: a creature, not 'another creature'");
    assert_eq!((epow(&g, t), etgh(&g, t)), (1, 3), "2/4 becomes 1/3");
    assert!(on(&g, t));
}

#[test]
fn enters_prefers_the_tiller_to_another_creature_of_ours() {
    let mut g = tbl();
    let own = perm(&mut g, P0, "Vampire Nighthawk");
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(t).plus, -1, "the Tiller takes it: the least harm");
    assert_eq!(g.perm(own).plus, 0);
}

#[test]
fn enters_never_kills_a_creature_of_ours_while_the_tiller_can_take_it() {
    let mut g = tbl();
    let own = perm(&mut g, P0, ELVES);
    let t = perm(&mut g, P0, TILLER);
    assert!(on(&g, own), "a 1/1 of ours is not the target");
    assert_eq!(g.perm(t).plus, -1);
}

#[test]
fn enters_a_creature_that_cannot_have_minus_counters_is_a_legal_target_with_no_counter() {
    let mut g = tbl();
    perm(&mut g, P1, "Melira, Sylvok Outcast"); // "Creatures you control can't have -1/-1 counters put on them"
    let big = perm(&mut g, P1, DRAGON);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, 0, "P1's creatures can't have -1/-1 counters");
    assert_eq!(g.perm(t).plus, 0, "the target was one of P1's: the Tiller took none");
    assert_eq!(n_lands(&g, P0), 0);
}

// ======================================================== the second line: a land from the hand or the graveyard
#[test]
fn enters_kills_a_one_toughness_opposing_creature_and_its_death_puts_a_land_tapped() {
    let mut g = tbl();
    let elves = perm(&mut g, P1, ELVES);
    let big = perm(&mut g, P1, DRAGON);
    hand(&mut g, P0, &["Swamp"]);
    let played = g.player(P0).lands_played;
    perm(&mut g, P0, TILLER);
    assert!(!on(&g, elves), "the 1/1 dies to the counter, though the 4/4 is worth more");
    assert!(in_gy(&g, P1, ELVES), "in its owner's graveyard");
    assert_eq!(g.perm(big).plus, 0);
    assert_eq!(land_names(&g, P0), vec!["Swamp"], "the creature died with a -1/-1 counter on it: a land");
    assert!(g.perm(g.player(P0).lands[0]).tapped, "onto the battlefield tapped");
    assert!(g.player(P0).hand.is_empty(), "it came from the hand");
    assert_eq!(g.player(P0).lands_played, played, "not the land drop");
}

#[test]
fn a_land_from_the_graveyard() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    gy_land(&mut g, P0, "Plains");
    perm(&mut g, P0, TILLER);
    assert_eq!(land_names(&g, P0), vec!["Plains"]);
    assert!(g.perm(g.player(P0).lands[0]).tapped);
    assert!(!in_gy(&g, P0, "Plains"), "it left the graveyard");
}

#[test]
fn the_land_enters_with_its_landfall_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Retreat"); // landfall: a 2/2 Cat Beast token
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 1);
    let cats = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    assert_eq!(cats, 1, "landfall: one 2/2 token");
}

#[test]
fn no_land_in_either_zone_nothing_happens() {
    let mut g = tbl();
    let elves = perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Lightning Bolt"]);
    perm(&mut g, P0, TILLER);
    assert!(!on(&g, elves), "it still dies");
    assert_eq!(n_lands(&g, P0), 0);
    assert!(in_hand(&g, P0, "Lightning Bolt"));
}

#[test]
fn a_land_of_someone_elses_graveyard_is_not_offered() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    gy_land(&mut g, P1, "Plains");
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 0, "your hand or your graveyard");
    assert!(in_gy(&g, P1, "Plains"));
}

#[test]
fn the_ai_prefers_a_land_that_adds_a_missing_colour_to_the_zone() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false); // blue is made; white, black, red and green are not
    hand(&mut g, P0, &["Swamp"]);
    gy_land(&mut g, P0, "Island");
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER);
    assert_eq!(
        land_names(&g, P0),
        vec!["Island", "Island", "Swamp"],
        "black is new: the hand copy, not the graveyard Island"
    );
    assert!(in_gy(&g, P0, "Island"));
}

#[test]
fn the_ai_prefers_the_utility_lands_in_order_when_no_colour_is_missing() {
    let order = [
        "Cabal Coffers",
        "Urborg, Tomb of Yawgmoth",
        "Reliquary Tower",
        "Mistrise Village",
        "Path of Ancestry",
        "Command Tower",
    ];
    for i in 0..order.len() {
        let mut g = tbl();
        all_colours(&mut g);
        hand(&mut g, P0, &["Plains"]);
        for n in &order[i..] {
            gy_land(&mut g, P0, n);
        }
        perm(&mut g, P1, ELVES);
        perm(&mut g, P0, TILLER);
        let names = land_names(&g, P0);
        assert_eq!(names.last().unwrap(), order[i], "{order:?} from {i}: {names:?}");
        assert!(in_hand(&g, P0, "Plains"), "a plain Plains is the last choice");
    }
}

#[test]
fn equal_lands_the_graveyard_copy_goes_first_and_the_hand_keeps_its_own() {
    let mut g = tbl();
    all_colours(&mut g);
    hand(&mut g, P0, &["Plains"]);
    gy_land(&mut g, P0, "Plains");
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 6);
    assert!(in_hand(&g, P0, "Plains"), "the hand copy stays");
    assert!(!in_gy(&g, P0, "Plains"));
}

#[test]
fn equal_lands_a_hand_at_the_limit_gives_up_its_copy() {
    let mut g = tbl();
    all_colours(&mut g);
    hand(
        &mut g,
        P0,
        &[
            "Plains",
            "Lightning Bolt",
            "Lightning Bolt",
            "Lightning Bolt",
            "Lightning Bolt",
            "Lightning Bolt",
            "Lightning Bolt",
        ],
    );
    gy_land(&mut g, P0, "Plains");
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 6);
    assert_eq!(g.player(P0).hand.len(), 6, "seven cards: the one that would be discarded is the land");
    assert!(!in_hand(&g, P0, "Plains"));
    assert!(in_gy(&g, P0, "Plains"), "the graveyard keeps its copy");
}

#[test]
fn the_ai_does_not_put_mana_confluence_at_ten_life_or_less() {
    let mut g = tbl();
    all_colours(&mut g);
    gy_land(&mut g, P0, "Mana Confluence");
    g.player_mut(P0).life = 10;
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 5, "10 life: it costs 1 life a tap");
    assert!(in_gy(&g, P0, "Mana Confluence"));
    let mut g = tbl();
    all_colours(&mut g);
    gy_land(&mut g, P0, "Mana Confluence");
    g.player_mut(P0).life = 11;
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER);
    assert_eq!(land_names(&g, P0).last().unwrap(), "Mana Confluence", "11 life: it does");
}

// ======================================================== any creature with a -1/-1 counter on it that dies
#[test]
fn a_creature_that_dies_to_something_else_with_a_counter_puts_a_land() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 0);
    gy_land(&mut g, P0, "Forest");
    let a = perm(&mut g, P1, BEARS);
    repl::put_counters(&mut g, P1, a, -1);
    assert_eq!(g.perm(a).plus, -1);
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(!on(&g, a));
    assert_eq!(land_names(&g, P0), vec!["Forest"]);
    assert!(on(&g, t));
}

#[test]
fn a_creature_of_ours_that_dies_with_a_counter_also_puts_a_land() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let own = perm(&mut g, P0, BEARS);
    repl::put_counters(&mut g, P0, own, -1);
    zones::die(&mut g, own, "sac").unwrap();
    assert_eq!(land_names(&g, P0), vec!["Forest"], "any creature, whoever controls it");
}

#[test]
fn a_token_with_a_counter_that_dies_puts_a_land() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let tok = token(&mut g, P1, 2);
    repl::put_counters(&mut g, P1, tok, -1);
    zones::die(&mut g, tok, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1, "a token is a creature that died");
}

#[test]
fn a_creature_without_a_counter_that_dies_puts_nothing() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let a = perm(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0, "no -1/-1 counter on it");
    assert!(in_gy(&g, P0, "Forest"));
}

#[test]
fn a_noncreature_permanent_that_dies_puts_nothing() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let rock = perm(&mut g, P1, "Sol Ring");
    g.perm_mut(rock).plus = -1; // (a stray number: the rule is about a creature)
    zones::die(&mut g, rock, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0);
}

#[test]
fn the_tiller_dying_with_its_own_counter_puts_a_land() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER); // alone: its own counter, 1/3
    assert_eq!(g.perm(t).plus, -1);
    gy_land(&mut g, P0, "Forest");
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(!on(&g, t));
    assert_eq!(land_names(&g, P0), vec!["Forest"], "it sees its own death, with the counter it carried");
}

#[test]
fn the_tiller_dying_without_a_counter_puts_nothing() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    g.perm_mut(t).plus = 0; // (it was put on a creature of an opponent's, say)
    gy_land(&mut g, P0, "Forest");
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0);
}

#[test]
fn two_tillers_two_lands() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER); // alone: its counter goes on itself
    hand(&mut g, P0, &["Swamp", "Plains"]);
    perm(&mut g, P1, ELVES);
    perm(&mut g, P0, TILLER); // kills the Elves
    assert_eq!(n_lands(&g, P0), 2, "each Tiller triggers on its own");
    assert!(g.player(P0).hand.is_empty());
}

#[test]
fn counters_cancel_one_plus_one_and_one_minus_one_is_no_counter() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let a = perm(&mut g, P1, BEARS);
    repl::put_counters(&mut g, P1, a, 1);
    repl::put_counters(&mut g, P1, a, -1);
    assert_eq!(g.perm(a).plus, 0, "the engine nets the counters, as 704.5q does");
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0, "no -1/-1 counter is left on it");
}

#[test]
fn counters_one_plus_one_and_two_minus_one_leave_a_minus_one_counter() {
    let mut g = tbl();
    perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    let a = perm(&mut g, P1, BEARS);
    repl::put_counters(&mut g, P1, a, 1);
    repl::put_counters(&mut g, P1, a, -2);
    assert_eq!(g.perm(a).plus, -1);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1, "one -1/-1 counter is left on it");
}

// ======================================================== ward
#[test]
fn the_ai_skips_a_creature_whose_ward_it_cannot_pay() {
    let mut g = tbl();
    let w = perm(&mut g, P1, WARDED);
    let big = perm(&mut g, P1, DRAGON);
    hand(&mut g, P0, &["Swamp"]);
    perm(&mut g, P0, TILLER); // no mana: ward {1} can't be paid
    assert!(on(&g, w), "the 2/1 the counter would kill is left alone");
    assert_eq!(g.perm(w).plus, 0);
    assert_eq!(g.perm(big).plus, -1, "the next best target");
    assert_eq!(n_lands(&g, P0), 0);
}

#[test]
fn the_ai_pays_a_ward_it_can_afford_to_kill() {
    let mut g = tbl();
    let w = perm(&mut g, P1, WARDED);
    perm(&mut g, P1, DRAGON);
    let m = lands(&mut g, P0, "Mountain", 1, false)[0];
    hand(&mut g, P0, &["Swamp"]);
    perm(&mut g, P0, TILLER);
    assert!(g.perm(m).tapped, "ward {{1}} paid");
    assert!(!on(&g, w), "killed: it is worth more than the 4/4's shrink");
    assert_eq!(n_lands(&g, P0), 2, "the Mountain and the Swamp that came with the death");
}

#[test]
fn a_person_who_does_not_pay_the_ward_loses_the_trigger() {
    let mut g = tbl();
    let w = perm(&mut g, P1, WARDED);
    gy_land(&mut g, P0, "Forest");
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains("on which creature") {
            Answer::Pick(r.choices.iter().position(|c| c.contains(WARDED)).unwrap_or(0))
        } else if r.prompt.contains("ward") {
            Answer::No
        } else {
            Answer::Action(json!({"do": "pass"}))
        }
    }));
    g.humans.add(P0, h);
    perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(w).plus, 0, "ward countered the trigger: no counter");
    assert!(on(&g, w));
    assert_eq!(n_lands(&g, P0), 0);
}

// ======================================================== a person's choices
#[test]
fn a_person_chooses_any_creature_and_cannot_skip() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    let elves = perm(&mut g, P1, ELVES);
    let h = person(&mut g, DRAGON, None);
    let t = perm(&mut g, P0, TILLER);
    let asked: Vec<_> = h.asked_kind("target").into_iter().filter(|r| r.prompt.starts_with("Dread Tiller")).collect();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].choices.len(), 3, "both of P1's and the Tiller itself, no way to skip: {:?}", asked[0].choices);
    assert_eq!(g.perm(big).plus, -1, "the 4/4, though the 1/1 would die");
    assert!(on(&g, elves));
    assert_eq!(g.perm(t).plus, 0);
}

#[test]
fn a_person_may_target_a_creature_of_their_own() {
    let mut g = tbl();
    perm(&mut g, P1, DRAGON);
    let own = perm(&mut g, P0, BEARS);
    person(&mut g, BEARS, None);
    perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(own).plus, -1);
}

#[test]
fn a_person_picks_the_land_from_either_zone() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp", "Lightning Bolt"]);
    gy_land(&mut g, P0, "Forest");
    let h = person(&mut g, ELVES, Some("Swamp"));
    perm(&mut g, P0, TILLER);
    let qs: Vec<_> =
        h.asked().into_iter().filter(|r| r.prompt.contains("put a land from your hand or graveyard")).collect();
    assert_eq!(qs.len(), 1, "asked once");
    let q = &qs[0];
    assert_eq!(q.choices, vec!["Swamp (hand)", "Forest (graveyard)", "decline"], "both zones, labelled, and a way out");
    assert_eq!(land_names(&g, P0), vec!["Swamp"]);
    assert!(in_gy(&g, P0, "Forest"), "the other stays");
    assert!(in_hand(&g, P0, "Lightning Bolt"));
}

#[test]
fn a_person_picks_the_graveyard_land() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    gy_land(&mut g, P0, "Forest");
    person(&mut g, ELVES, Some("Forest"));
    perm(&mut g, P0, TILLER);
    assert_eq!(land_names(&g, P0), vec!["Forest"]);
    assert!(in_hand(&g, P0, "Swamp"));
    assert!(!in_gy(&g, P0, "Forest"));
    assert!(g.perm(g.player(P0).lands[0]).tapped);
}

#[test]
fn a_person_may_decline_the_land() {
    let mut g = tbl();
    let elves = perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    gy_land(&mut g, P0, "Forest");
    person(&mut g, ELVES, None);
    perm(&mut g, P0, TILLER);
    assert!(!on(&g, elves), "the creature still died");
    assert_eq!(n_lands(&g, P0), 0, "'you may': declined");
    assert!(in_hand(&g, P0, "Swamp"));
    assert!(in_gy(&g, P0, "Forest"));
}

#[test]
fn a_person_with_no_land_is_not_asked() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    let h = person(&mut g, ELVES, None);
    perm(&mut g, P0, TILLER);
    assert!(
        h.asked().iter().all(|r| !r.prompt.contains("put a land from your hand or graveyard")),
        "no land card in either zone"
    );
}

// ======================================================== Elesh Norn, Mother of Machines
#[test]
fn our_elesh_norn_makes_the_enters_trigger_happen_twice() {
    let mut g = tbl();
    perm(&mut g, P0, NORN);
    let big = perm(&mut g, P1, DRAGON);
    perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, -2, "two counters");
    assert_eq!((epow(&g, big), etgh(&g, big)), (2, 2));
}

#[test]
fn our_elesh_norn_gives_two_chances_the_second_after_the_first_death() {
    let mut g = tbl();
    perm(&mut g, P0, NORN);
    let elves = perm(&mut g, P1, ELVES);
    let big = perm(&mut g, P1, DRAGON);
    hand(&mut g, P0, &["Swamp", "Plains"]);
    perm(&mut g, P0, TILLER);
    assert!(!on(&g, elves), "the first trigger kills the 1/1");
    assert_eq!(g.perm(big).plus, -1, "the second shrinks the 4/4");
    assert_eq!(n_lands(&g, P0), 1, "one death with a counter: one land");
}

#[test]
fn an_opposing_elesh_norn_stops_the_enters_trigger() {
    let mut g = tbl();
    let norn = perm(&mut g, P1, NORN);
    let big = perm(&mut g, P1, DRAGON);
    let elves = perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, 0);
    assert_eq!(g.perm(norn).plus, 0);
    assert!(on(&g, elves));
    assert_eq!(g.perm(t).plus, 0, "the trigger did not happen");
    assert_eq!(n_lands(&g, P0), 0);
}

// ======================================================== The Reaper, King No More
#[test]
fn the_reaper_takes_the_creature_the_tiller_kills_and_the_tiller_puts_a_land() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER); // no creature of an opponent's yet: nothing to shrink
    let elves = perm(&mut g, P1, ELVES);
    gy_land(&mut g, P0, "Forest");
    perm(&mut g, P0, TILLER);
    assert!(!on(&g, elves));
    assert!(has_perm(&g, P0, ELVES), "the Reaper puts it onto the battlefield under P0's control");
    assert!(!in_gy(&g, P1, ELVES), "out of its owner's graveyard");
    assert_eq!(land_names(&g, P0), vec!["Forest"], "and the Tiller's land, from the same death");
    assert!(g.perm(g.player(P0).lands[0]).tapped);
}

// ======================================================== blinking
#[test]
fn a_blink_happens_the_enters_trigger_again() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, -1);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    assert!(sim_core::impls::mine::ephemerate_cast(&mut g, P0, t, "value").unwrap());
    assert_eq!(perms_named(&g, P0, TILLER).len(), 1, "it came back");
    assert_eq!(g.perm(big).plus, -2, "a second counter from the new Tiller");
}

// ======================================================== a Tiller that has left
#[test]
fn a_creature_that_dies_after_the_tiller_has_left_is_not_seen() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    gy_land(&mut g, P0, "Forest");
    gy_land(&mut g, P0, "Plains");
    let a = perm(&mut g, P1, BEARS);
    repl::put_counters(&mut g, P1, a, -1);
    zones::die(&mut g, t, "destroy").unwrap(); // its own death, with its own counter: a land
    assert_eq!(n_lands(&g, P0), 1);
    zones::die(&mut g, a, "destroy").unwrap(); // dies later: the Tiller is gone
    assert_eq!(n_lands(&g, P0), 1, "an ability of a permanent that is no longer on the battlefield");
}
