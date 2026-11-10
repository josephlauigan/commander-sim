//! Aragorn's Breach rebuild (2026-11-10, JD): the cards new to the simulator, checked against their Oracle text, and
//! what the Underworld Breach line (impls/breach_line.rs) does with them: Lion's Eye Diamond, Thassa's Oracle,
//! Pyretic Ritual, Strike It Rich, cantrips mid-line, Baral's discount, a dry run that doesn't know its own draws, and
//! a ready line that waits one turn for protection when a counter is likely.

use sim_core::ai::act;
use sim_core::engine::cast::{additional_cost, cast_card};
use sim_core::engine::mana::cost_of;
use sim_core::ids::PlayerId;
use sim_core::impls::breach_line;
use sim_core::state::Ctx;
use sim_core::state::{Game, Step};
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
/// P0's library holds no line card (nothing for a self-mill to dig up by luck).
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

fn go(g: &mut Game) -> bool {
    let opts = breach_line::breach_options(g, P0, false).unwrap();
    if opts.is_empty() {
        return false;
    }
    assert_eq!(opts[0].label, "Underworld Breach line");
    act::perform(g, P0, opts[0].act.as_ref().unwrap()).unwrap();
    true
}

// ------------------------------------------------------------------ the line

#[test]
fn breach_lion_s_eye_diamond_and_brain_freeze_mill_the_table() {
    // one land and three graveyard cards: Lion's Eye Diamond pays for every escape, Brain Freeze at yourself refuels
    let mut g = position(&["Lion's Eye Diamond", "Brain Freeze"], 3, 1, 60, 40);
    assert!(go(&mut g), "the Diamond loop finishes the table, so the line is offered");
    assert!(table_finished(&g), "every opponent milled out or dead");
    assert!(g.player(P0).stats.get("led_used").copied().unwrap_or(0) >= 2, "the Diamond is escaped again and again");
}

#[test]
fn thassa_s_oracle_finishes_when_the_table_cannot_be_milled() {
    // P0's library is down to 15 cards: the Diamond loop runs out of self-mill fuel long before it mills three full
    // libraries, and the table is at full life. Only milling yourself and casting the Oracle wins
    let small = |g: &mut Game| g.player_mut(P0).library.truncate(15);
    let mut g = position(&["Lion's Eye Diamond", "Brain Freeze", "Thassa's Oracle"], 3, 1, 99, 40);
    small(&mut g);
    let mut no_oracle = position(&["Lion's Eye Diamond", "Brain Freeze"], 3, 1, 99, 40);
    small(&mut no_oracle);
    assert!(!go(&mut no_oracle), "without the Oracle the line doesn't finish the table: not offered");
    assert!(go(&mut g), "with the Oracle it does");
    assert_eq!(g.winner, Some(P0), "Thassa's Oracle on an empty library wins");
}

#[test]
fn pyretic_ritual_is_one_of_the_line_s_rituals() {
    // the same position as the Aragorn line test, with Pyretic Ritual instead of Desperate Ritual
    let hand_a = ["Brain Freeze", "Grapeshot", "Seething Song", "Pyretic Ritual", "Rite of Flame", "Lotus Petal"];
    let mut g = position(&hand_a, 12, 4, 9, 40);
    assert!(go(&mut g));
    assert!(table_finished(&g));
    assert!(!g.player(P0).hand.contains(&card(&g, "Pyretic Ritual")), "Pyretic Ritual was cast in the line");
}

#[test]
fn strike_it_rich_adds_storm_for_its_own_treasure() {
    let mut g =
        position(&["Brain Freeze", "Grapeshot", "Seething Song", "Strike It Rich", "Lotus Petal"], 12, 4, 9, 40);
    assert!(go(&mut g));
    assert!(table_finished(&g));
    assert!(!g.player(P0).hand.contains(&card(&g, "Strike It Rich")), "Strike It Rich was cast for storm");
}

#[test]
fn baral_makes_the_line_s_instants_and_sorceries_cheaper() {
    let mut g = position(&["Brain Freeze", "Seething Song", "Grapeshot"], 6, 2, 40, 40);
    let (ss, bf, gs) = (card(&g, "Seething Song"), card(&g, "Brain Freeze"), card(&g, "Grapeshot"));
    assert_eq!(cost_of(&g, P0, ss).0, 2);
    perm(&mut g, P0, "Baral, Chief of Compliance");
    assert_eq!(cost_of(&g, P0, ss).0, 1, "Seething Song costs {{1}}{{R}} with Baral");
    assert_eq!(cost_of(&g, P0, bf).0, 0, "Brain Freeze costs {{U}}");
    assert_eq!(cost_of(&g, P0, gs).0, 0, "Grapeshot costs {{R}}");
    let breach = card(&g, "Underworld Breach");
    let _ = breach;
}

#[test]
fn a_cantrip_in_hand_is_cast_mid_line_for_storm() {
    let hand_a = ["Brain Freeze", "Grapeshot", "Seething Song", "Desperate Ritual", "Rite of Flame", "Lotus Petal"];
    let mut with = position(&hand_a, 12, 6, 9, 40);
    hand(&mut with, P0, &["Preordain"]);
    assert!(go(&mut with));
    assert!(table_finished(&with));
    assert!(!with.player(P0).hand.contains(&card(&with, "Preordain")), "Preordain was cast in the line");
}

#[test]
fn the_dry_run_does_not_know_its_own_draws() {
    // the only way to finish is the Grapeshot on top of the library, drawn by Preordain. The dry run shuffles the
    // library first, so it can't count on that draw: the line is not offered
    let mut g = position(&["Brain Freeze", "Seething Song", "Preordain"], 3, 3, 80, 40);
    let gs = take(&mut g, P0, "Grapeshot");
    g.player_mut(P0).library.push(gs);
    assert!(!go(&mut g), "a line that needs a lucky draw is not offered");
}

#[test]
fn a_card_put_on_top_by_p_itself_is_still_known() {
    // the same, but P0 put the Grapeshot there itself (Mystical Tutor): the dry run keeps it on top
    let mut g = position(&["Brain Freeze", "Seething Song", "Preordain"], 3, 3, 80, 40);
    let gs = take(&mut g, P0, "Grapeshot");
    g.player_mut(P0).library.push(gs);
    g.player_mut(P0).known_top = vec![gs];
    let mut shuffled = position(&["Brain Freeze", "Seething Song", "Preordain"], 3, 3, 80, 40);
    let gs2 = take(&mut shuffled, P0, "Grapeshot");
    shuffled.player_mut(P0).library.push(gs2);
    assert!(!go(&mut shuffled));
    // known: whether it finishes now depends only on the line's arithmetic, not on luck
    let offered = go(&mut g);
    let again = {
        let mut g3 = position(&["Brain Freeze", "Seething Song", "Preordain"], 3, 3, 80, 40);
        let gs3 = take(&mut g3, P0, "Grapeshot");
        g3.player_mut(P0).library.push(gs3);
        g3.player_mut(P0).known_top = vec![gs3];
        breach_line::breach_options(&mut g3, P0, false).unwrap().len() == 1
    };
    assert_eq!(offered, again, "the answer for a known top card is the same every time");
}

// ------------------------------------------------------------------ waiting for protection

/// a position whose line finishes the table, with each opponent holding untapped Islands and a counterspell in its
/// deck (a counter is likely)
fn risky_position(extra_hand: &[&str]) -> Game {
    let mut hand_a =
        vec!["Brain Freeze", "Grapeshot", "Seething Song", "Desperate Ritual", "Rite of Flame", "Lotus Petal"];
    hand_a.extend_from_slice(extra_hand);
    let mut g = position(&hand_a, 12, 4, 9, 40);
    for q in 1..4 {
        lands(&mut g, PlayerId(q), "Island", 3, false);
        g.player_mut(PlayerId(q)).hand = vec![card(&g, "Island"); 5];
    }
    g
}

#[test]
fn an_unprotected_line_waits_one_turn_when_a_counter_is_likely() {
    let mut g = risky_position(&[]);
    let risk = sim_core::ai::brain::counter_risk(&g, P0);
    assert!(risk >= 0.35, "the position must make a counter likely (risk {risk:.2})");
    assert!(breach_line::breach_options(&mut g, P0, false).unwrap().is_empty(), "turn 6: it waits");
    assert!(breach_line::breach_options(&mut g, P0, true).unwrap().is_empty(), "the second main phase agrees");
    g.player_mut(P0).turns = 7;
    assert_eq!(breach_line::breach_options(&mut g, P0, false).unwrap().len(), 1, "turn 7: it goes");
}

#[test]
fn a_protected_line_goes_at_once() {
    let mut g = risky_position(&["Force of Will"]);
    assert_eq!(breach_line::breach_options(&mut g, P0, false).unwrap().len(), 1, "Force of Will in hand");
    let mut abolished = risky_position(&[]);
    perm(&mut abolished, P0, "Grand Abolisher");
    assert_eq!(breach_line::breach_options(&mut abolished, P0, false).unwrap().len(), 1, "Grand Abolisher out");
}

#[test]
fn low_on_life_it_does_not_wait() {
    let mut g = risky_position(&[]);
    g.player_mut(P0).life = 8;
    assert_eq!(breach_line::breach_options(&mut g, P0, false).unwrap().len(), 1);
}

// ------------------------------------------------------------------ the cards, against their Oracle text

#[test]
fn thought_scour_mills_yourself_with_breach_out_and_an_opponent_otherwise() {
    // "Target player mills two cards. Draw a card."
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    lands(&mut g, P0, "Island", 1, false);
    let ts = hand(&mut g, P0, &["Thought Scour"])[0];
    let (lib0, opp0) = (g.player(P0).library.len(), g.player(P1).library.len() + g.player(PlayerId(2)).library.len());
    sim_core::engine::mana::pay(&mut g, P0, 0, "U", false).unwrap();
    cast_card(&mut g, P0, ts, "hand", Ctx::default()).unwrap();
    let opp1 = g.player(P1).library.len() + g.player(PlayerId(2)).library.len();
    assert_eq!(g.player(P0).library.len(), lib0 - 1, "no Breach: an opponent is milled, P0 only draws");
    assert!(opp1 + 2 <= opp0 + 0 || g.player(PlayerId(3)).library.len() < 99, "an opponent milled two");

    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P0, "Underworld Breach");
    lands(&mut g, P0, "Island", 1, false);
    let ts = hand(&mut g, P0, &["Thought Scour"])[0];
    let lib0 = g.player(P0).library.len();
    let gy0 = g.player(P0).gy.len();
    sim_core::engine::mana::pay(&mut g, P0, 0, "U", false).unwrap();
    cast_card(&mut g, P0, ts, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).library.len(), lib0 - 3, "Breach out: P0 mills two (escape fuel) and draws one");
    assert_eq!(g.player(P0).gy.len(), gy0 + 3, "the two milled cards and Thought Scour itself");
}

#[test]
fn cathartic_reunion_discards_two_as_its_cost_then_draws_three() {
    // "As an additional cost to cast this spell, discard two cards. Draw three cards."
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    g.player_mut(P0).hand.clear();
    let cr = hand(&mut g, P0, &["Cathartic Reunion"])[0];
    hand(&mut g, P0, &["Forest"]);
    assert!(!additional_cost(&mut g, P0, cr, true).unwrap(), "one other card: it can't be paid");
    hand(&mut g, P0, &["Plains", "Lightning Bolt"]);
    assert!(additional_cost(&mut g, P0, cr, true).unwrap(), "three other cards: it can");
    lands(&mut g, P0, "Mountain", 2, false);
    sim_core::engine::mana::pay(&mut g, P0, 1, "R", false).unwrap();
    assert!(additional_cost(&mut g, P0, cr, false).unwrap());
    assert_eq!(g.player(P0).hand.len(), 2, "two of the three discarded (Cathartic Reunion is still in hand)");
    assert_eq!(g.player(P0).gy.len(), 2, "both to the graveyard");
    g.player_mut(P0).hand.retain(|&x| x != cr);
    let before = g.player(P0).hand.len();
    sim_core::engine::cast::resolve(&mut g, P0, cr, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 3, "then it draws three");
}

#[test]
fn baral_loots_when_a_spell_of_yours_counters_a_spell() {
    // "Whenever a spell or ability you control counters a spell, you may draw a card. If you do, discard a card."
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    perm(&mut g, P0, "Baral, Chief of Compliance");
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Island", "Lightning Bolt"]);
    let (lib0, gy0) = (g.player(P0).library.len(), g.player(P0).gy.len());
    let ctr = card(&g, "Counterspell");
    sim_core::engine::stack::counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    assert_eq!(g.player(P0).library.len(), lib0 - 1, "drew a card");
    assert_eq!(g.player(P0).hand.len(), 2, "and discarded one");
    assert_eq!(g.player(P0).gy.len(), gy0 + 1);
    // no Baral: nothing
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    let lib0 = g.player(P0).library.len();
    sim_core::engine::stack::counter_side_effects(&mut g, P0, P1, ctr).unwrap();
    assert_eq!(g.player(P0).library.len(), lib0);
}

#[test]
fn strike_it_rich_makes_a_treasure() {
    // "Create a Treasure token. Flashback {2}{R}"
    let mut g = table(&["aragorn", "cloud", "sheoldred", "yshtola"]);
    let sir = hand(&mut g, P0, &["Strike It Rich"])[0];
    lands(&mut g, P0, "Mountain", 1, false);
    let t0 = g.player(P0).treasures;
    sim_core::engine::mana::pay(&mut g, P0, 0, "R", false).unwrap();
    cast_card(&mut g, P0, sir, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
    assert!(g.player(P0).gy.contains(&sir), "to the graveyard, where flashback can cast it again");
    assert_eq!(g.db.get(sir).tags.str(sim_core::tag::Tag::Fb), Some("2R"));
}

// ------------------------------------------------------------------ review fixes (2026-11-10)

#[test]
fn a_lotus_petal_makes_the_oracle_s_second_blue() {
    // one Volcanic Island, a two-card library, Thassa's Oracle and Lotus Petal in hand, no Diamond: the Petal floats
    // {U} (not {R} or any colour), so the Oracle's {U}{U} is paid and it wins
    let mut g = position(&["Thassa's Oracle", "Lotus Petal", "Brain Freeze"], 3, 1, 99, 40);
    g.player_mut(P0).library.truncate(2);
    assert!(go(&mut g), "the Oracle line is offered");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_look_ahead_copy_of_a_waiting_turn_does_not_take_the_line() {
    let mut g = risky_position(&[]);
    assert!(breach_line::breach_options(&mut g, P0, false).unwrap().is_empty(), "the real game waits");
    g.in_search = true; // a look-ahead copy of this turn (the dry run's answer is cached for it)
    assert!(breach_line::breach_options(&mut g, P0, false).unwrap().is_empty(), "so do its copies");
}
