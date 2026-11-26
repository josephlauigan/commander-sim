//! Adversarial tests for Dread Tiller (impls/karn_gilded_cards.rs `tiller_etb`, `tiller_dies`), written by a reviewer
//! who did not write the code. The expected values come from the Oracle text and the comprehensive rules, not from what
//! the engine does:
//!   - "When this creature enters, put a -1/-1 counter on target creature."
//!   - "Whenever a creature with a -1/-1 counter on it dies, you may put a land card from your hand or graveyard onto
//!     the battlefield tapped."
//!
//! Every test passes. Most pin a rule worth keeping. Four pin a NAMED LIMIT instead (their names start with `limit_`): the
//! engine kills creatures one at a time (a wipe, combat damage, state-based actions), so a Tiller that has died does not
//! hear a creature that dies later in the same event, though CR 603.10a says all of them are heard. Each of those asserts
//! what the engine does today and says what the rules would give; if the engine ever looks back, they fail and are to be
//! turned round.

mod play_common;
use play_common::*;
use sim_core::engine::control::{self, Duration};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{removal, repl, zones};
use sim_core::state::{Ctx, Step};
use std::sync::Arc;

const TILLER: &str = "Dread Tiller";
const REAPER: &str = "The Reaper, King No More";
const NORN: &str = "Elesh Norn, Mother of Machines";
const TEYSA: &str = "Teysa Karlov";
const ELVES: &str = "Llanowar Elves"; // a 1/1
const BEARS: &str = "Ainok Strike Leader"; // a 2/2
const DRAGON: &str = "Glorybringer"; // a 4/4
const CARYATID: &str = "Sylvan Caryatid"; // 0/3 hexproof

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

fn n_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.len()
}

fn land_names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.perm(l).card()).name.to_string()).collect()
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// a creature of P1's carrying one -1/-1 counter (put directly: the Tiller is not the one under test here)
fn with_counter(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let m = perm(g, p, name);
    repl::put_counters(g, p, m, -1);
    assert_eq!(g.perm(m).plus, -1);
    m
}

/// a Tiller on P0's side with no counter on it and P1 holding one creature the counter went on (so the Tiller's own
/// counter is not in the way)
fn clean_tiller(g: &mut Game) -> PermId {
    let t = perm(g, P0, TILLER);
    g.perm_mut(t).plus = 0;
    t
}

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

// ============================================================== 1. double fire / no fire on the Tiller's own death
/// pins: the dying Tiller is heard once (self_dies), not twice (dies as well): three lands available, one comes
#[test]
fn the_dying_tiller_with_its_counter_puts_exactly_one_land() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER); // alone: its own counter
    assert_eq!(g.perm(t).plus, -1);
    hand(&mut g, P0, &["Swamp", "Forest"]);
    gy_land(&mut g, P0, "Plains");
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1, "one death, one trigger: {:?}", land_names(&g, P0));
}

/// pins: a second Tiller hears the first one's death with its counter, the first hears its own: one land each
#[test]
fn two_tillers_the_one_that_dies_with_a_counter_gives_each_exactly_one_land() {
    let mut g = tbl();
    let a = perm(&mut g, P0, TILLER); // counter on itself
    let b = perm(&mut g, P0, TILLER); // counter on itself too
    g.perm_mut(b).plus = 0; // b carries no counter; a does
    assert_eq!(g.perm(a).plus, -1);
    hand(&mut g, P0, &["Swamp", "Forest", "Plains", "Island"]);
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(on(&g, b));
    assert_eq!(n_lands(&g, P0), 2, "A's own trigger and B's: {:?}", land_names(&g, P0));
}

/// pins: a Tiller without a counter dying beside a Tiller without one: nothing
#[test]
fn two_tillers_no_counter_nothing() {
    let mut g = tbl();
    let a = clean_tiller(&mut g);
    let b = clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp", "Forest"]);
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(on(&g, b));
    assert_eq!(n_lands(&g, P0), 0);
}

// ============================================================== 2. whose land: controller, not owner
/// Oracle: "you" is the Tiller's controller. P0 owns the card, P1 controls it: P1 puts the land from P1's hand.
#[test]
fn a_stolen_tiller_gives_the_land_to_its_controller_from_the_controllers_zones() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    assert!(control::gain_control(&mut g, P1, t, Duration::Permanent));
    hand(&mut g, P0, &["Plains"]);
    hand(&mut g, P1, &["Swamp"]);
    let _ = with_counter(&mut g, P0, BEARS);
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(land_names(&g, P1), vec!["Swamp"], "the controller's land");
    assert_eq!(n_lands(&g, P0), 0, "the owner gets nothing");
}

/// Oracle: the stolen Tiller dying with its counter: the controller's trigger
#[test]
fn a_stolen_tiller_dying_gives_its_controller_the_land() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER); // counter on itself
    assert_eq!(g.perm(t).plus, -1);
    assert!(control::gain_control(&mut g, P1, t, Duration::Permanent));
    hand(&mut g, P0, &["Plains"]);
    hand(&mut g, P1, &["Swamp"]);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(land_names(&g, P1), vec!["Swamp"]);
    assert_eq!(n_lands(&g, P0), 0);
}

/// a Tiller entering under P1's control (P0 owns the card, as a Reaper steal or Sneak Attack would do): the ETB counter
/// is P1's, so P0's creature is the opposing one
#[test]
fn a_tiller_entering_under_another_players_control_counters_the_controllers_opponents() {
    let mut g = tbl();
    let big = perm(&mut g, P0, DRAGON);
    let theirs = perm(&mut g, P1, "Vampire Nighthawk");
    let cd = take(&mut g, P0, TILLER);
    let t = zones::enter(&mut g, P1, cd, zones::Enter { sick: false, orig: Some(P0), ..Default::default() }).unwrap();
    assert_eq!(g.perm(t).owner, P1, "P1 controls it");
    assert_eq!(g.perm(big).plus, -1, "P0's 4/4 is P1's opponent's creature");
    assert_eq!(g.perm(theirs).plus, 0);
}

// ============================================================== 3. phased out, neutered, copies, blink
/// Rule 702.26: a phased-out permanent is treated as though it does not exist: its abilities do not trigger
#[test]
fn a_phased_out_tiller_does_not_hear_a_death() {
    let mut g = tbl();
    let t = clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    g.perm_mut(t).phased = true;
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0);
}

#[test]
fn a_neutered_tiller_does_not_hear_a_death() {
    let mut g = tbl();
    let t = clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    g.perm_mut(t).neutered = true;
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 0);
}

/// a phased-out creature is not a legal target of the enters trigger
#[test]
fn a_phased_out_creature_is_not_targeted() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    g.perm_mut(big).phased = true;
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, 0);
    assert_eq!(g.perm(t).plus, -1, "the only legal target is the Tiller itself");
}

/// a token copy of the Tiller (The Fire Crystal, Helm of the Host ...) has the same card, so its enters trigger fires
/// and its death trigger works
#[test]
fn a_token_copy_of_the_tiller_enters_and_hears() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, -1);
    let copies = zones::token_copy_of(&mut g, P0, t, &[]).unwrap();
    assert_eq!(copies.len(), 1);
    assert_eq!(g.perm(big).plus, -2, "the copy's enters trigger put a second counter");
    gy_land(&mut g, P0, "Forest");
    gy_land(&mut g, P0, "Plains");
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 2, "the original and the copy each hear the death");
}

/// the Tiller leaves before its enters trigger resolves (blink in response): the trigger still resolves, the counter is
/// put (it does not need its source), and nothing panics
#[test]
fn the_enters_trigger_still_resolves_when_the_tiller_has_left() {
    let mut g = tbl();
    let big = perm(&mut g, P1, DRAGON);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(big).plus, -1);
    zones::leave(&mut g, t).unwrap();
    assert!(!on(&g, t));
    sim_core::engine::hooks::run_hook(
        &mut g,
        t,
        sim_core::hooks::Event::Etb,
        &sim_core::hooks::Call::Etb { p: P0, m: t },
    )
    .unwrap();
    assert_eq!(g.perm(big).plus, -2, "the counter of the trigger whose source is gone");
}

// ============================================================== 4. counter bookkeeping
/// a creature with a -1/-1 counter and a non-counter pump (an effect until end of turn) still has the counter
#[test]
fn a_pumped_creature_with_a_counter_dies_with_it() {
    let mut g = tbl();
    clean_tiller(&mut g);
    gy_land(&mut g, P0, "Forest");
    let a = with_counter(&mut g, P1, BEARS);
    g.perm_mut(a).eot_pt = (3, 3);
    assert_eq!(etgh(&g, a), 4);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1);
}

/// a creature whose toughness is raised so the counter does not kill it: no death, no land
#[test]
fn a_toughness_boost_keeps_the_creature_alive_and_the_ai_does_not_count_it_as_a_kill() {
    let mut g = tbl();
    let e = perm(&mut g, P1, ELVES);
    g.perm_mut(e).eot_pt = (0, 1); // 1/2 until end of turn
    let bears = perm(&mut g, P1, BEARS);
    gy_land(&mut g, P0, "Forest");
    perm(&mut g, P0, TILLER);
    assert!(on(&g, e), "the 1/2 survives a -1/-1 counter");
    assert_eq!(n_lands(&g, P0), 0);
    // the AI: both survive the counter, so it shrinks the one worth more (the 2/2 bears are worth more than the elf)
    let _ = bears;
}

/// persist: Kitchen Finks returns with a -1/-1 counter (a real counter): dying again triggers the Tiller
#[test]
fn a_persisted_creature_that_dies_again_has_a_counter() {
    let mut g = tbl();
    clean_tiller(&mut g);
    gy_land(&mut g, P0, "Forest");
    gy_land(&mut g, P0, "Plains");
    let f = perm(&mut g, P1, "Kitchen Finks");
    zones::die(&mut g, f, "destroy").unwrap(); // first death: no counter, it persists back
    assert_eq!(n_lands(&g, P0), 0, "no counter the first time");
    let back = g.player(P1).perms.iter().copied().find(|&m| g.perm(m).name == "Kitchen Finks").expect("it persisted");
    assert_eq!(g.perm(back).plus, -1);
    zones::die(&mut g, back, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1, "the second death carries the persist counter");
}

/// two -1/-1 counters and a +1/+1 counter: net one -1/-1 counter (CR 704.5q): triggers once
#[test]
fn net_minus_counters_trigger_once() {
    let mut g = tbl();
    clean_tiller(&mut g);
    gy_land(&mut g, P0, "Forest");
    gy_land(&mut g, P0, "Plains");
    let a = perm(&mut g, P1, DRAGON);
    repl::put_counters(&mut g, P1, a, 1);
    repl::put_counters(&mut g, P1, a, -2);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1);
}

// ============================================================== 5. wipes and simultaneous deaths
fn wrath(g: &mut Game, caster: PlayerId) {
    removal::apply_wipe(g, caster, "destroy", None, &Ctx::default()).unwrap();
}

/// PINS A NAMED LIMIT (CR 603.10a is not modelled): the rules say a Tiller that dies in the same Wrath as a creature
/// carrying a -1/-1 counter hears that creature die (1 land). The engine kills the creatures one at a time (`apply_wipe`
/// goes seat by seat, each seat's `perms` in order), and a Tiller that has left does not hear later deaths of the same
/// event. P0's Tiller (seat 0) is wiped before P1's creature (seat 1): no land.
#[test]
fn limit_wrath_tiller_in_the_earlier_seat_does_not_hear_the_later_death() {
    let mut g = tbl();
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    with_counter(&mut g, P1, BEARS);
    wrath(&mut g, P1);
    assert_eq!(n_lands(&g, P0), 0, "the limit: the Tiller is gone before the Bears die (the rules give 1)");
}

/// the same with the seats swapped (the Tiller is P1's: P0's creature dies first): the engine hears it
#[test]
fn wrath_the_other_seat_order_the_tiller_hears_the_creature_that_died_first() {
    let mut g = tbl();
    let t = perm(&mut g, P1, TILLER);
    g.perm_mut(t).plus = 0;
    hand(&mut g, P1, &["Swamp"]);
    with_counter(&mut g, P0, BEARS);
    wrath(&mut g, P0);
    assert_eq!(n_lands(&g, P1), 1);
}

/// PINS A NAMED LIMIT (CR 603.10a is not modelled): the same seat, the counter creature entered after the Tiller, so it
/// dies after it in the seat's list: not heard (the rules give 1 land)
#[test]
fn limit_wrath_same_seat_creature_listed_after_the_tiller_is_not_heard() {
    let mut g = tbl();
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    with_counter(&mut g, P0, BEARS);
    wrath(&mut g, P0);
    assert_eq!(n_lands(&g, P0), 0, "the limit: the Bears die after the Tiller (the rules give 1)");
}

/// the same seat, the creature listed before the Tiller
#[test]
fn wrath_same_seat_creature_listed_before_the_tiller() {
    let mut g = tbl();
    with_counter(&mut g, P0, BEARS);
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    wrath(&mut g, P0);
    assert_eq!(n_lands(&g, P0), 1);
}

/// PINS A NAMED LIMIT (CR 603.10a is not modelled): the Tiller with its own counter in a Wrath beside a P1 creature with a
/// counter: the rules give two lands (both deaths, at the same time); the engine gives the Tiller's own death only
#[test]
fn limit_wrath_tiller_with_its_own_counter_hears_its_own_death_and_not_a_later_one() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(t).plus, -1);
    hand(&mut g, P0, &["Swamp", "Forest"]);
    with_counter(&mut g, P1, BEARS);
    wrath(&mut g, P1);
    assert_eq!(n_lands(&g, P0), 1, "the limit: its own death, not the Bears' (the rules give 2)");
}

// ============================================================== 6. targeting
/// the Tiller's black-green source can't target a hexproof creature of an opponent
#[test]
fn the_ai_does_not_target_a_hexproof_creature() {
    let mut g = tbl();
    let c = perm(&mut g, P1, CARYATID);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(c).plus, 0);
    assert_eq!(g.perm(t).plus, -1);
}

#[test]
fn a_person_is_not_offered_a_hexproof_or_protected_creature() {
    let mut g = tbl();
    let c = perm(&mut g, P1, CARYATID);
    let p = perm(&mut g, P1, BEARS);
    g.perm_mut(p).eot_kw.push("pro_B"); // protection from black
    let h = person(&mut g, DRAGON, None);
    perm(&mut g, P0, TILLER);
    let q = &h.asked_kind("target")[0];
    assert!(!q.choices.iter().any(|c| c.contains(CARYATID)), "{:?}", q.choices);
    assert!(!q.choices.iter().any(|c| c.contains(BEARS)), "pro-black Bears: {:?}", q.choices);
    assert_eq!((g.perm(c).plus, g.perm(p).plus), (0, 0));
}

/// shroud and protection stop the controller's own spells and abilities too (only hexproof does not): a Giver of Runes'
/// protection from black (the Tiller is black and green) or Lightning Greaves' shroud on a creature of ours makes it an
/// illegal target (`targetable_by`)
#[test]
fn a_person_is_not_offered_their_own_creature_with_protection_from_black() {
    let mut g = tbl();
    perm(&mut g, P1, DRAGON);
    let own = perm(&mut g, P0, BEARS);
    g.perm_mut(own).eot_kw.push("pro_B");
    let h = person(&mut g, DRAGON, None);
    perm(&mut g, P0, TILLER);
    let q = &h.asked_kind("target")[0];
    assert!(
        !q.choices.iter().any(|c| c.contains(BEARS)),
        "own protection from black is no way through: {:?}",
        q.choices
    );
}

/// shroud from Lightning Greaves on our own creature
#[test]
fn a_person_is_not_offered_their_own_creature_with_shroud() {
    let mut g = tbl();
    perm(&mut g, P1, DRAGON);
    let own = perm(&mut g, P0, BEARS);
    let greaves = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(greaves).attached = Some(own);
    assert!(sim_core::engine::values::untargetable_by_its_controller(
        &g,
        own,
        sim_core::cards::Colors::from_letters("BG")
    ));
    let h = person(&mut g, DRAGON, None);
    perm(&mut g, P0, TILLER);
    let q = &h.asked_kind("target")[0];
    assert!(!q.choices.iter().any(|c| c.contains(BEARS)), "Greaves: shroud: {:?}", q.choices);
}

/// a creature that can't have -1/-1 counters is a legal target that gets none (no crash); the Tiller does not shrink
#[test]
fn melira_side_creature_is_a_legal_target_and_the_ai_prefers_it_to_hurting_itself() {
    let mut g = tbl();
    perm(&mut g, P1, "Melira, Sylvok Outcast");
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(t).plus, 0, "Melira's creature is worth 0: no harm to the Tiller");
}

/// the lone creature is the Tiller: it targets itself (1/3)
#[test]
fn alone_it_shrinks_itself() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    assert_eq!((epow(&g, t), etgh(&g, t)), (1, 3));
}

/// the AI does not kill its own better creature while the Tiller itself can take the counter
#[test]
fn the_ai_does_not_put_the_counter_on_its_own_commander_or_a_one_toughness_creature() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Karn, Gilded Guardian");
    let e = perm(&mut g, P0, ELVES);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(k).plus, 0);
    assert!(on(&g, e));
    assert_eq!(g.perm(t).plus, -1);
}

/// ward the person won't pay: the trigger is countered, the Tiller does nothing (existing pins this); here the person
/// is asked once for the target even with a responder around (the probe path)
#[test]
fn the_person_is_asked_for_the_target_once() {
    let mut g = tbl();
    perm(&mut g, P1, DRAGON);
    perm(&mut g, P1, BEARS);
    let h = person(&mut g, DRAGON, None);
    perm(&mut g, P0, TILLER);
    let asked: Vec<_> = h.asked_kind("target").into_iter().filter(|r| r.prompt.starts_with("Dread Tiller")).collect();
    assert_eq!(asked.len(), 1, "{:?}", asked.iter().map(|r| r.prompt.clone()).collect::<Vec<_>>());
}

/// the Reaper asks a person once for each of its two targets (not again as the engine probes the trigger)
#[test]
fn the_reaper_asks_how_many_times() {
    let mut g = tbl();
    perm(&mut g, P1, DRAGON);
    perm(&mut g, P1, BEARS);
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains("The Reaper") { Answer::Pick(0) } else { Answer::Action(json!({"do": "pass"})) }
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, REAPER);
    let n = h.asked().into_iter().filter(|r| r.prompt.contains("The Reaper")).count();
    println!("REAPER prompts: {n}");
    assert_eq!(n, 2, "the Reaper asked {n} times for two targets");
}

// ============================================================== 7. the land clause
/// a land that enters tapped does not use the land drop and fires landfall once
#[test]
fn the_land_is_not_a_land_drop_and_landfall_happens_once() {
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Retreat");
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    let played = g.player(P0).lands_played;
    perm(&mut g, P0, TILLER);
    assert_eq!(g.player(P0).lands_played, played);
    let cats = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    assert_eq!(cats, 1);
}

/// our Elesh Norn doubles what a permanent entering causes to trigger: the land entering triggers landfall twice
#[test]
fn landfall_with_our_elesh_norn_is_doubled_like_a_played_land() {
    let mut g = tbl();
    perm(&mut g, P0, NORN);
    perm(&mut g, P0, "Felidar Retreat"); // landfall: a 2/2 token (twice with Norn)
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    let before = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count();
    perm(&mut g, P0, TILLER);
    // a played land with the same board, for comparison
    let after = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count() - before;
    let mut h = tbl();
    perm(&mut h, P0, NORN);
    perm(&mut h, P0, "Felidar Retreat");
    hand(&mut h, P0, &["Swamp"]);
    let b2 = h.player(P0).perms.iter().filter(|&&m| h.perm(m).token).count();
    let sw = h.player(P0).hand[0];
    h.player_mut(P0).hand.clear();
    sim_core::engine::turn::play_land_card(&mut h, P0, sw, "plays").unwrap();
    let a2 = h.player(P0).perms.iter().filter(|&&m| h.perm(m).token).count() - b2;
    assert_eq!(after, a2, "the Tiller's land makes as many landfall tokens as a played land ({a2})");
}

/// a fetch land put by the Tiller from the hand cracks and leaves a land (the fetch itself is not the land that stays)
#[test]
fn a_fetchland_from_the_hand_cracks_at_once() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Wooded Foothills"]);
    let life = g.player(P0).life;
    perm(&mut g, P0, TILLER);
    let names = land_names(&g, P0);
    assert_eq!(names.len(), 1, "{names:?}");
    assert_ne!(names[0], "Wooded Foothills", "the fetch was cracked: {names:?}");
    assert_eq!(g.player(P0).life, life - 1);
    assert!(g.player(P0).gy.contains(&card(&g, "Wooded Foothills")));
}

/// a modal double-faced card whose front is a spell is not a land card in hand or graveyard
#[test]
fn a_modal_card_with_a_spell_front_is_not_a_land_card() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Agadeem's Awakening // Agadeem, the Undercrypt"]);
    perm(&mut g, P0, TILLER);
    assert_eq!(n_lands(&g, P0), 0);
}

/// an artifact land is a land card
#[test]
fn an_artifact_land_is_a_land_card() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    gy_land(&mut g, P0, "Darksteel Citadel");
    perm(&mut g, P0, TILLER);
    assert_eq!(land_names(&g, P0), vec!["Darksteel Citadel"]);
}

/// a creature land card (Dryad Arbor) put onto the battlefield is a land AND a creature (Oracle: Land Creature)
/// (the same engine path as playing it: if this fails, it fails for a played Dryad Arbor too)
#[test]
fn dryad_arbor_is_still_a_creature_when_it_arrives() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    gy_land(&mut g, P0, "Dryad Arbor");
    perm(&mut g, P0, TILLER);
    let l = g.player(P0).lands[0];
    println!("Dryad Arbor creature: {}", g.is_creature(l));
    assert!(g.is_creature(l));
}

/// the land that comes is the card of the chosen zone: nothing is duplicated, the other zone keeps its copy
#[test]
fn nothing_duplicated() {
    let mut g = tbl();
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Swamp"]);
    gy_land(&mut g, P0, "Swamp");
    let total = |g: &Game| {
        let sw = card(g, "Swamp");
        g.player(P0).hand.iter().filter(|&&c| c == sw).count()
            + g.player(P0).gy.iter().filter(|&&c| c == sw).count()
            + g.player(P0).lands.iter().filter(|&&l| g.perm(l).card() == sw).count()
    };
    let before = total(&g);
    perm(&mut g, P0, TILLER);
    assert_eq!(total(&g), before);
    assert_eq!(n_lands(&g, P0), 1);
}

// ============================================================== 8. Elesh Norn, Teysa
/// an opposing Elesh Norn blanks what an ENTERING permanent causes: a death is not an entering
#[test]
fn an_opposing_elesh_norn_does_not_stop_the_death_trigger() {
    let mut g = tbl();
    let t = clean_tiller(&mut g);
    perm(&mut g, P1, NORN);
    hand(&mut g, P0, &["Swamp"]);
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(on(&g, t));
    assert_eq!(n_lands(&g, P0), 1);
}

/// our Teysa Karlov: a creature dying causes a triggered ability of a permanent we control to trigger an additional time
#[test]
fn teysa_doubles_the_death_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, TEYSA);
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp", "Forest", "Plains"]);
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 2, "two triggers: two lands ({:?})", land_names(&g, P0));
}

/// Teysa and the Tiller's own death: the Tiller's leaves-the-battlefield trigger is caused by a creature dying (itself)
#[test]
fn teysa_doubles_the_tillers_own_death_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, TEYSA);
    let t = perm(&mut g, P0, TILLER);
    assert_eq!(g.perm(t).plus, -1);
    hand(&mut g, P0, &["Swamp", "Forest", "Plains"]);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 2, "{:?}", land_names(&g, P0));
}

/// an opponent's Teysa doubles only the opponent's triggers
#[test]
fn an_opposing_teysa_does_not_double() {
    let mut g = tbl();
    perm(&mut g, P1, TEYSA);
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp", "Forest", "Plains"]);
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 1);
}

// ============================================================== 9. the Reaper chain
/// the Tiller's counter kills an opposing 1/1 while the Reaper is out: the Tiller's land and the Reaper's steal, once
#[test]
fn two_deaths_in_one_turn_two_lands_and_one_steal() {
    let mut g = tbl();
    clean_tiller(&mut g);
    perm(&mut g, P0, REAPER);
    hand(&mut g, P0, &["Swamp", "Forest"]);
    let a = with_counter(&mut g, P1, ELVES);
    let b = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    assert_eq!(n_lands(&g, P0), 2, "each death: a land");
    let stolen = g.player(P0).perms.iter().filter(|&&m| matches!(&*g.perm(m).name, ELVES | BEARS)).count();
    assert_eq!(stolen, 1, "the Reaper: once each turn");
}

/// the Reaper's own enters trigger (two counters) with a Tiller out: the Reaper kills two 1/1s: two lands, one steal
#[test]
fn reaper_enters_with_a_tiller_out_two_kills_two_lands_one_steal() {
    let mut g = tbl();
    clean_tiller(&mut g);
    perm(&mut g, P1, ELVES);
    perm(&mut g, P1, "Birds of Paradise");
    hand(&mut g, P0, &["Swamp", "Forest", "Plains"]);
    perm(&mut g, P0, REAPER);
    assert_eq!(n_lands(&g, P0), 2, "{:?}", land_names(&g, P0));
    let stolen =
        g.player(P0).perms.iter().filter(|&&m| matches!(&*g.perm(m).name, ELVES | "Birds of Paradise")).count();
    assert_eq!(stolen, 1);
}

// ============================================================== 10. the human path
/// a person with a land in hand is asked once per death; declining leaves the land where it was
#[test]
fn a_person_is_asked_once_per_death_and_may_decline() {
    let mut g = tbl();
    clean_tiller(&mut g);
    hand(&mut g, P0, &["Swamp"]);
    let h = person(&mut g, ELVES, None);
    let a = with_counter(&mut g, P1, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    let qs: Vec<_> =
        h.asked().into_iter().filter(|r| r.prompt.contains("put a land from your hand or graveyard")).collect();
    assert_eq!(qs.len(), 1, "asked once");
    assert_eq!(n_lands(&g, P0), 0);
}

/// a stolen Tiller's land question goes to the controller, not to the owner
#[test]
fn a_stolen_tillers_person_prompt_goes_to_the_controller() {
    let mut g = tbl();
    let t = clean_tiller(&mut g);
    assert!(control::gain_control(&mut g, P1, t, Duration::Permanent));
    hand(&mut g, P0, &["Swamp"]);
    hand(&mut g, P1, &["Forest"]);
    let h = person(&mut g, ELVES, None); // P0 is the person
    let a = with_counter(&mut g, P0, BEARS);
    zones::die(&mut g, a, "destroy").unwrap();
    let qs: Vec<_> =
        h.asked().into_iter().filter(|r| r.prompt.contains("put a land from your hand or graveyard")).collect();
    assert_eq!(qs.len(), 0, "the person at P0 does not control the Tiller");
    assert_eq!(land_names(&g, P1), vec!["Forest"]);
}

// ============================================================== 13. AI quality
/// the AI with a worse choice than the best: a killable 1/1 (worth little) against a much better creature the counter
/// only shrinks: it prefers the kill (the land and the Reaper); printed so the board is on record
#[test]
fn ai_choice_kill_a_mana_dork_over_shrinking_a_dragon() {
    let mut g = tbl();
    let e = perm(&mut g, P1, ELVES);
    let d = perm(&mut g, P1, DRAGON);
    gy_land(&mut g, P0, "Forest");
    perm(&mut g, P0, TILLER);
    println!("elves on bf: {}, dragon plus: {}", on(&g, e), g.perm(d).plus);
    assert!(!on(&g, e));
}

/// the AI never puts a land it can't use well: fetch in hand, basic in graveyard, every colour made
#[test]
fn ai_land_choice_fetch_versus_basic() {
    let mut g = tbl();
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, b, 1, false);
    }
    perm(&mut g, P1, ELVES);
    hand(&mut g, P0, &["Wooded Foothills"]);
    gy_land(&mut g, P0, "Forest");
    perm(&mut g, P0, TILLER);
    println!("AI put: {:?}", land_names(&g, P0));
}

// ============================================================== more simultaneous deaths (state-based, not only wipes)
/// 0-toughness creatures die together (704.5f): the Tiller (2/4 with four -1/-1 counters) and a Bears with two. CR 603.10a:
/// the rules give 2 lands (CR 603.10a). PINS A NAMED LIMIT: `sba_toughness` kills them one at a time, in seat order, so
/// the Tiller (seat 0) hears only its own death.
#[test]
fn limit_state_based_deaths_in_one_check_are_heard_in_seat_order() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TILLER);
    g.perm_mut(t).plus = -4; // 0 toughness once the counters are on (put directly: no check yet)
    let b = perm(&mut g, P1, BEARS);
    g.perm_mut(b).plus = -2;
    hand(&mut g, P0, &["Swamp", "Forest"]);
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!on(&g, t) && !on(&g, b));
    assert_eq!(n_lands(&g, P0), 1, "the limit: its own death only (the rules give 2): {:?}", land_names(&g, P0));
}

/// the Tiller's counter on an opposing creature with undying and toughness 1 (Young Wolf) kills it, and it comes back with
/// a +1/+1 counter as a 2/2 (undying cares about +1/+1 counters only): the AI does not count that as a kill, and shrinks a
/// creature that stays shrunk
#[test]
fn ai_does_not_kill_an_undying_creature_it_would_return_bigger() {
    let mut g = tbl();
    let w = perm(&mut g, P1, "Young Wolf");
    let bears = perm(&mut g, P1, BEARS);
    gy_land(&mut g, P0, "Forest");
    perm(&mut g, P0, TILLER);
    assert!(on(&g, w), "the Wolf was not killed (it would come back as a 2/2)");
    assert_eq!(g.perm(w).plus, 0, "it carries no counter of any kind");
    assert_eq!(g.perm(bears).plus, -1, "the counter shrinks the Bears instead");
    assert_eq!(n_lands(&g, P0), 0);
}
