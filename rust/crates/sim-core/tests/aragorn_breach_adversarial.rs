//! Adversarial tests for the Aragorn Breach rebuild (2026-11-10): Baral's discount in the line, the Thassa's Oracle
//! finish, cantrips cast mid-line, the dry run's determinism, waiting for protection, Cathartic Reunion's discard cost
//! and Baral's loot. Every expectation is from the cards' Oracle text; the two defects these tests found are fixed.

use sim_core::ai::act;
use sim_core::engine::cast::cast_card;
use sim_core::engine::stack::counter_side_effects;
use sim_core::ids::PlayerId;
use sim_core::impls::breach_line;
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// every card the line counts as its own (never plain escape fuel)
const LINE: [&str; 14] = [
    "Brain Freeze",
    "Grapeshot",
    "Lotus Petal",
    "Dark Ritual",
    "Cabal Ritual",
    "Seething Song",
    "Desperate Ritual",
    "Pyretic Ritual",
    "Rite of Flame",
    "Lion's Eye Diamond",
    "Jeska's Will",
    "Strike It Rich",
    "Thassa's Oracle",
    "Underworld Breach",
];

/// P0 (Aragorn) in its main phase with Underworld Breach out, `hand_names` in hand, `n_fuel` plain cards in the
/// graveyard and `lands_n` untapped Volcanic Islands. Each opponent keeps `opp_lib` library cards and `life` life.
/// P0's library holds no line card.
fn position(hand_names: &[&str], n_fuel: usize, lands_n: usize, opp_lib: usize, life: i32) -> Game {
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).turns = 6;
    g.player_mut(P0).hand.clear();
    perm(&mut g, P0, "Underworld Breach");
    lands(&mut g, P0, "Volcanic Island", lands_n, false);
    hand(&mut g, P0, hand_names);
    let lib: Vec<_> = g.player(P0).library.iter().copied().filter(|&c| !LINE.contains(&&*g.db.get(c).name)).collect();
    g.player_mut(P0).library = lib;
    let mut fuel = vec![];
    while fuel.len() < n_fuel {
        fuel.push(g.player_mut(P0).library.pop().unwrap());
    }
    g.player_mut(P0).gy.extend(fuel);
    for q in 1..4 {
        let pl = g.player_mut(PlayerId(q));
        pl.library.truncate(opp_lib);
        pl.life = life;
    }
    g
}

fn table_finished(g: &Game) -> bool {
    g.winner == Some(P0) || g.opps(P0).all(|q| !g.player(q).alive || g.player(q).library.is_empty())
}

fn offered(g: &mut Game) -> bool {
    !breach_line::breach_options(g, P0, false).unwrap().is_empty()
}

fn go(g: &mut Game) -> bool {
    let opts = breach_line::breach_options(g, P0, false).unwrap();
    if opts.is_empty() {
        return false;
    }
    act::perform(g, P0, opts[0].act.as_ref().unwrap()).unwrap();
    true
}

// ------------------------------------------------------------------ Baral in the line

#[test]
fn baral_one_island_pays_for_brain_freeze() {
    // Baral: instants and sorceries cost {1} less, so Brain Freeze ({1}{U}) costs {U}: one Volcanic Island pays for it.
    // Only one opponent still has a library (three cards: one Brain Freeze at it, no storm needed); the others are
    // already milled out
    let mut g = position(&["Brain Freeze"], 3, 1, 3, 40);
    for q in 2..4 {
        g.player_mut(PlayerId(q)).library.clear();
    }
    perm(&mut g, P0, "Baral, Chief of Compliance");
    let bf = card(&g, "Brain Freeze");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, bf).0, 0, "Brain Freeze costs {{U}} with Baral");
    // (found here 2026-11-10: line_step's "short of the generic" check used the printed {1}; it now reads the cost)
    assert!(go(&mut g), "one Island and Baral: Brain Freeze mills the last library");
    assert!(table_finished(&g));
}

#[test]
fn baral_one_island_control_two_islands_work() {
    // the same position with a second Island: the line is offered (so the one-Island failure above is the {1} check)
    let mut g = position(&["Brain Freeze"], 3, 2, 3, 40);
    for q in 2..4 {
        g.player_mut(PlayerId(q)).library.clear();
    }
    assert!(go(&mut g));
    assert!(table_finished(&g));
}

#[test]
fn baral_discount_is_realised_by_the_line() {
    // three Islands, Seething Song and Brain Freeze, three opponents with one card each, 12 cards of fuel. With Baral
    // Song costs {1}{R} and the third Island pays Brain Freeze's {U}; without him Song takes all three Islands and the
    // red mana cannot pay {U}
    let mut with = position(&["Seething Song", "Brain Freeze"], 12, 3, 1, 40);
    perm(&mut with, P0, "Baral, Chief of Compliance");
    assert!(go(&mut with), "with Baral the line finishes");
    assert!(table_finished(&with));
    let mut without = position(&["Seething Song", "Brain Freeze"], 12, 3, 1, 40);
    assert!(!offered(&mut without), "without Baral it cannot (no blue left after Song)");
}

#[test]
fn baral_does_not_discount_the_breach_itself() {
    // Underworld Breach is an enchantment: Baral doesn't touch it ({1}{R})
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P0, "Baral, Chief of Compliance");
    let b = hand(&mut g, P0, &["Underworld Breach"])[0];
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, b).0, 1);
}

// ------------------------------------------------------------------ Thassa's Oracle finish

#[test]
fn oracle_wins_on_an_empty_library() {
    // "If X is greater than or equal to the number of cards in your library, you win the game"
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.player_mut(P0).library.clear();
    lands(&mut g, P0, "Island", 2, false);
    let o = hand(&mut g, P0, &["Thassa's Oracle"])[0];
    sim_core::engine::mana::pay(&mut g, P0, 0, "UU", false).unwrap();
    cast_card(&mut g, P0, o, "hand", Ctx::default()).unwrap();
    assert_eq!(g.winner, Some(P0), "devotion 2 (its own {{U}}{{U}}) >= library 0");
}

#[test]
fn oracle_does_not_win_on_a_library_above_its_devotion() {
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.player_mut(P0).library.truncate(3);
    lands(&mut g, P0, "Island", 2, false);
    let o = hand(&mut g, P0, &["Thassa's Oracle"])[0];
    sim_core::engine::mana::pay(&mut g, P0, 0, "UU", false).unwrap();
    cast_card(&mut g, P0, o, "hand", Ctx::default()).unwrap();
    assert_ne!(g.winner, Some(P0), "devotion 2 < library 3");
}

#[test]
fn oracle_line_wins_with_the_library_at_the_boundary() {
    // library 2 = its own {U}{U}: the Oracle alone wins; Brain Freeze is only there to make the line legal
    let mut g = position(&["Brain Freeze", "Thassa's Oracle"], 3, 4, 99, 40);
    g.player_mut(P0).library.truncate(2);
    assert!(go(&mut g));
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn oracle_line_is_not_offered_when_the_library_stays_too_big() {
    // 30 cards, no fuel, one Brain Freeze mills three: no way to get under the Oracle's devotion
    let mut g = position(&["Brain Freeze", "Thassa's Oracle"], 0, 4, 99, 40);
    g.player_mut(P0).library.truncate(30);
    assert!(!offered(&mut g));
}

#[test]
fn oracle_line_with_a_lab_man_and_oracle_both_in_hand_does_not_lose() {
    // Laboratory Maniac and the Oracle in hand, small library: whichever finish the line picks, P0 must not be
    // decked without a win
    let mut g = position(&["Brain Freeze", "Thassa's Oracle", "Laboratory Maniac"], 3, 4, 99, 40);
    g.player_mut(P0).library.truncate(5);
    if go(&mut g) {
        assert_eq!(g.winner, Some(P0), "a line that mills P0 out must win on the spot");
    }
}

// ------------------------------------------------------------------ cantrips mid-line

#[test]
fn cantrip_is_not_cast_into_a_three_card_library() {
    // Brainstorm draws three: from a three-card library it leaves P0 to draw from an empty one next turn (a loss)
    let mut g = position(&["Brain Freeze", "Seething Song", "Grapeshot", "Brainstorm"], 6, 6, 1, 40);
    g.player_mut(P0).library.truncate(3);
    let bs = card(&g, "Brainstorm");
    assert!(go(&mut g), "the line finishes without the cantrip");
    assert!(table_finished(&g));
    assert!(g.player(P0).library.len() >= 1, "P0's library must not be drawn empty by a cantrip");
    assert!(g.player(P0).hand.contains(&bs), "Brainstorm stays in hand");
}

#[test]
fn strike_it_rich_escaped_from_the_graveyard_costs_its_mana_cost() {
    // Strike It Rich escapes for {R} (not its {2}{R} flashback) and its Treasure pays the next escape: two Islands,
    // Seething Song and Brain Freeze finish three one-card libraries only with it in the graveyard
    let mut without = position(&["Brain Freeze", "Seething Song"], 12, 2, 1, 40);
    assert!(!offered(&mut without), "without it: no");
    let mut g = position(&["Brain Freeze", "Seething Song"], 12, 2, 1, 40);
    let sir = take(&mut g, P0, "Strike It Rich");
    g.player_mut(P0).gy.push(sir);
    assert!(go(&mut g));
    assert!(table_finished(&g));
    assert!(g.player(P0).cast_names.contains(&sir));
}

#[test]
fn cathartic_reunion_mid_line_still_finishes_the_table() {
    // Cathartic Reunion has the `draw` tag, so the line may cast it as a cantrip: its discard cost throws away two
    // cards from hand. Discarded line cards are graveyard cards (Breach escapes them), so the table must still finish
    let hand_a = ["Cathartic Reunion", "Brain Freeze", "Grapeshot", "Seething Song", "Rite of Flame", "Lotus Petal"];
    let mut g = position(&hand_a, 12, 6, 9, 40);
    assert!(go(&mut g));
    assert!(table_finished(&g), "the line finishes the table");
}

// ------------------------------------------------------------------ the dry run

#[test]
fn the_dry_run_leaves_the_real_library_alone() {
    // the determinized copy is shuffled, not the real game: the library order is unchanged by asking
    let mut g = position(&["Brain Freeze", "Seething Song", "Preordain"], 3, 3, 80, 40);
    let before = g.player(P0).library.clone();
    let _ = offered(&mut g);
    let _ = offered(&mut g);
    assert_eq!(g.player(P0).library, before, "asking whether the line works must not shuffle P0's library");
}

#[test]
fn the_dry_run_gives_the_same_answer_twice_and_on_a_fresh_copy() {
    let build = || {
        let mut g = position(&["Brain Freeze", "Seething Song", "Preordain", "Opt"], 6, 4, 9, 40);
        let gs = take(&mut g, P0, "Grapeshot");
        g.player_mut(P0).library.push(gs);
        g
    };
    let mut a = build();
    let first = offered(&mut a);
    a.breach_dry = None; // no cache: recomputed
    let second = offered(&mut a);
    let mut b = build();
    let third = offered(&mut b);
    assert_eq!(first, second, "same position, same answer");
    assert_eq!(first, third, "same position built again, same answer");
}

// ------------------------------------------------------------------ waiting for protection

fn risky_position() -> Game {
    let hand_a = ["Brain Freeze", "Grapeshot", "Seething Song", "Desperate Ritual", "Rite of Flame", "Lotus Petal"];
    let mut g = position(&hand_a, 12, 4, 9, 40);
    for q in 1..4 {
        lands(&mut g, PlayerId(q), "Island", 3, false);
        g.player_mut(PlayerId(q)).hand = vec![card(&g, "Island"); 5];
    }
    g
}

#[test]
fn waiting_ends_after_one_turn_even_if_the_risk_stays() {
    let mut g = risky_position();
    assert!(!offered(&mut g), "turn 6: waits");
    g.player_mut(P0).turns = 7;
    assert!(offered(&mut g), "turn 7: goes");
    // and the turn after the wait it keeps going (the wait is not renewed on the same turn)
    assert!(offered(&mut g));
}

#[test]
fn a_wait_ends_in_the_second_main_phase_when_the_table_taps_out() {
    let mut g = risky_position();
    assert!(!offered(&mut g), "main 1: waits");
    for q in 1..4 {
        let ls = g.player(PlayerId(q)).lands.clone();
        for l in ls {
            g.perm_mut(l).tapped = true;
        }
    }
    g.step = Step::Main2;
    assert!(offered(&mut g), "the table has tapped out: nothing to wait for");
}

#[test]
fn a_look_ahead_copy_does_not_start_a_wait() {
    // in_search copies never record a wait (the real game's decision stands)
    let mut g = risky_position();
    g.in_search = true;
    g.breach_dry = Some((g.turn_stamp(), vec![], true));
    let _ = breach_line::breach_options(&mut g, P0, false).unwrap();
    assert_eq!(g.player(P0).breach_wait, None);
}

// ------------------------------------------------------------------ Cathartic Reunion and Baral's loot

#[test]
fn cathartic_reunion_with_exactly_two_other_cards_discards_both() {
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.player_mut(P0).hand.clear();
    let cr = hand(&mut g, P0, &["Cathartic Reunion", "Forest", "Mountain"])[0];
    lands(&mut g, P0, "Mountain", 2, false);
    assert!(sim_core::engine::cast::additional_cost(&mut g, P0, cr, true).unwrap());
    assert!(sim_core::engine::cast::additional_cost(&mut g, P0, cr, false).unwrap());
    assert_eq!(g.player(P0).hand, vec![cr], "both other cards are the cost; the Reunion stays to be cast");
    assert_eq!(g.player(P0).gy.len(), 2);
}

#[test]
fn baral_loots_once_per_baral() {
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P0, "Baral, Chief of Compliance");
    perm(&mut g, P0, "Baral, Chief of Compliance");
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Island", "Lightning Bolt"]);
    let lib0 = g.player(P0).library.len();
    let ctr = card(&g, "Counterspell");
    counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    assert_eq!(g.player(P0).library.len(), lib0 - 2, "each Baral triggers: two draws");
    assert_eq!(g.player(P0).hand.len(), 2, "two discards");
}

#[test]
fn a_phased_out_baral_does_not_loot() {
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    let b = perm(&mut g, P0, "Baral, Chief of Compliance");
    g.perm_mut(b).phased = true;
    let lib0 = g.player(P0).library.len();
    let ctr = card(&g, "Counterspell");
    counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    assert_eq!(g.player(P0).library.len(), lib0);
}

#[test]
fn baral_does_not_loot_the_opponent_whose_spell_was_countered() {
    // only the controller of the counterspell (q) loots, not the player whose spell was countered
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P1, "Baral, Chief of Compliance");
    let lib1 = g.player(P1).library.len();
    let ctr = card(&g, "Counterspell");
    counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    assert_eq!(g.player(P1).library.len(), lib1);
}

#[test]
fn baral_does_not_draw_the_empty_library() {
    // "you may draw a card": a player with an empty library doesn't (it would lose at the next state check)
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P0, "Baral, Chief of Compliance");
    g.player_mut(P0).library.clear();
    let ctr = card(&g, "Counterspell");
    counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    // (found here 2026-11-10: baral_loot always drew for the AI; it now declines with one card or none left)
    assert!(g.player(P0).alive, "an empty library and a Baral: the loot is optional");
}

#[test]
fn empty_library_without_baral_is_untouched_by_a_counter() {
    // control for the test above: the same position without Baral leaves P0 alive
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.player_mut(P0).library.clear();
    let ctr = card(&g, "Counterspell");
    counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.player(P0).alive);
}
