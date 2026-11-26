//! The AI plays The Fire Crystal ({4}{R}{R}, {T}: create a token that's a copy of target creature you control; sacrifice it
//! at the beginning of the next end step), in the Karn, Gilded Guardian deck. The expected results come from the card's
//! Oracle text and the design (/tmp/handoff/opus-fire-design-out.md, D3), on hand-built boards:
//!   - the AI copies in the end-of-turn window just before its own turn (the mana and the Crystal are free there), the best
//!     target by `karn_gilded_cards::copy_value`; in a main phase it copies only to win with Thassa's Oracle;
//!   - the hold rule (`AI_CRYSTAL_HOLD`) keeps {4}{R}{R} up in the main phase when the copy is worth three cards or more;
//!   - a person's ability is untouched (`human::abilities_of` returns the Crystal's own arm), and `AI_CRYSTAL=0` removes
//!     the AI's option.
//! P0 is the AI seat (Karn's deck); P1 is Krenko's deck and gives opposing creatures. Every constant is PROVISIONAL, so a
//! test pins what the AI does on a board, not the number's exact value.

mod play_common;
use play_common::*;
use sim_core::ai::{self, act, brain, table_fixes};
use sim_core::cards::Colors;
use sim_core::engine::life::check_state;
use sim_core::engine::zones;
use sim_core::hooks::Opt;
use sim_core::impls::karn_gilded::KARN_GILDED;
use sim_core::impls::karn_gilded_cards::{FC_HOLD_PEN, FC_HOLD_SHARE, copy_value};
use sim_core::state::{DataKey, Step, Val};
use std::sync::Arc;

const CRYSTAL: &str = "The Fire Crystal";
const KARN: &str = KARN_GILDED;
const OPP: &str = "krenko-mono-red-goblins";
const ORACLE: &str = "Thassa's Oracle";

/// P0 (Karn's deck, an AI) and P1: the end-of-turn window before P0's turn (P1's end step), an empty hand
fn tbl() -> Game {
    let mut g = table(&["karn_gilded", OPP]);
    g.active = Some(P1);
    g.step = Step::End;
    g.player_mut(P0).hand.clear();
    g
}

/// a main phase of P0's own turn
fn main_phase(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

/// six untapped lands with two red sources: {4}{R}{R}
fn six_lands(g: &mut Game) {
    lands(g, P0, "Mountain", 2, false);
    lands(g, P0, "Island", 4, false);
}

/// Karn on the battlefield as P0's commander (not in the command zone). Put him down before any artifact: his trigger
/// draws nothing yet
fn karn_on_the_table(g: &mut Game) -> PermId {
    set_commander(g, P0, KARN);
    let m = perm(g, P0, KARN);
    g.player_mut(P0).cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    m
}

/// a board with Karn (the best copy: five cards), the Crystal and {4}{R}{R}
fn karn_board() -> (Game, PermId, PermId) {
    let mut g = tbl();
    let karn = karn_on_the_table(&mut g);
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    assert!(g.player(P0).library.len() >= 20, "a library to draw from");
    (g, karn, c)
}

/// the Crystal's options in a window (`post`: None the end-of-turn window, Some(false) main one, Some(true) main two), as
/// the AI asks for them
fn fc_opts(g: &mut Game, post: Option<bool>) -> Vec<Opt> {
    let s = brain::Situation::new(g, P0);
    brain::hook_options(g, P0, &s, post)
        .unwrap()
        .into_iter()
        .filter(|o| o.label.starts_with("The Fire Crystal"))
        .collect()
}

fn do_opt(g: &mut Game, o: &Opt) -> bool {
    act::perform(g, P0, o.act.as_ref().expect("an action")).unwrap()
}

fn named(g: &Game, name: &str) -> Vec<PermId> {
    perms_named(g, P0, name)
}

fn tokens_named(g: &Game, name: &str) -> Vec<PermId> {
    named(g, name).into_iter().filter(|&m| g.perm(m).token).collect()
}

fn value(g: &Game, c: PermId, m: PermId, at_eot: bool) -> f64 {
    copy_value(g, P0, c, m, at_eot)
}

// ======================================================== 1. Karn at the end-of-turn window
#[test]
fn karn_is_copied_in_the_end_of_turn_window_and_the_real_karn_stays() {
    let (mut g, karn, c) = karn_board();
    let opts = fc_opts(&mut g, None);
    assert_eq!(opts.len(), 1, "{:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    assert_eq!(opts[0].label, "The Fire Crystal: copy Karn, Gilded Guardian");
    assert!(opts[0].utility >= 8.0, "five cards: {}", opts[0].utility);
    let hand = g.player(P0).hand.len();
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 5, "the copy's trigger: a card for each of the five colours");
    assert!(g.perm(c).tapped, "the Crystal is tapped");
    assert_eq!(named(&g, KARN).len(), 1, "exactly one Karn");
    assert!(g.perm(karn).on_bf && g.perm(karn).is_cmd && !g.perm(karn).token, "the real one, the commander");
    assert!(tokens_named(&g, KARN).is_empty(), "the token went to the legend rule");
    let me = g.player(P0);
    assert_eq!(me.stats.get("fc_acts"), Some(&1));
    assert_eq!(me.stats.get("fc_acts_eot"), Some(&1));
    assert_eq!(me.stats.get("fc_eot_ready"), Some(&1), "the window is counted once");
    assert_eq!(me.stats.get("fc_copy:Karn, Gilded Guardian"), Some(&1));
}

// ======================================================== 2. no main-phase activation without a win
#[test]
fn there_is_no_main_phase_activation_without_a_win() {
    let (mut g, _, _) = karn_board();
    main_phase(&mut g);
    assert!(fc_opts(&mut g, Some(false)).is_empty(), "main phase one");
    assert!(fc_opts(&mut g, Some(true)).is_empty(), "main phase two");
    assert_eq!(fc_opts(&mut g, None).len(), 1, "the same board in the end-of-turn window");
}

// ======================================================== 3. the Oracle wins
#[test]
fn a_copy_of_thassas_oracle_that_empties_the_library_wins_in_a_main_phase() {
    let mut g = tbl();
    main_phase(&mut g);
    perm(&mut g, P0, ORACLE);
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    g.player_mut(P0).library.truncate(4); // devotion UU plus the copy's UU
    let opts = fc_opts(&mut g, Some(false));
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].utility, 9.5);
    assert!(value(&g, c, named(&g, ORACLE)[0], false) >= 50.0);
    assert!(do_opt(&mut g, &opts[0]));
    assert!(g.over, "the copy's trigger looked at the whole library");
    assert_eq!(g.winner, Some(P0));
    assert_eq!(g.player(P0).stats.get("fc_acts_win"), Some(&1));
}

#[test]
fn a_copy_of_the_oracle_that_does_not_win_is_not_made() {
    let mut g = tbl();
    main_phase(&mut g);
    perm(&mut g, P0, ORACLE);
    perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    g.player_mut(P0).library.truncate(5); // one card too many
    assert!(fc_opts(&mut g, Some(false)).is_empty(), "no win, no main-phase copy");
}

// ======================================================== 4. nothing worth copying
#[test]
fn a_creature_worth_nothing_gives_no_option() {
    let mut g = tbl();
    let sentinel = perm(&mut g, P0, "Esper Sentinel");
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let v = value(&g, c, sentinel, true);
    assert!((v - 0.15).abs() < 1e-9, "a 1/1 body and no enters trigger: {v}");
    assert!(fc_opts(&mut g, None).is_empty());
}

// ======================================================== 5. the decking guard
#[test]
fn a_copy_of_karn_that_would_deck_the_ai_is_never_made() {
    let (mut g, karn, c) = karn_board();
    g.player_mut(P0).library.truncate(6); // five draws leave one: under the floor of three
    assert_eq!(value(&g, c, karn, true), sim_core::impls::karn_gilded::DECKING);
    assert!(fc_opts(&mut g, None).is_empty());
    // the edge: eight cards (8 <= 5 + 3) is a bust, nine leaves four, over the floor
    let (mut g, _, _) = karn_board();
    g.player_mut(P0).library.truncate(8);
    assert!(fc_opts(&mut g, None).is_empty(), "five draws leave three: the floor");
    let (mut g, _, _) = karn_board();
    g.player_mut(P0).library.truncate(9);
    assert_eq!(fc_opts(&mut g, None).len(), 1, "five draws leave four");
}

// ======================================================== 6. an opposing Elesh Norn
#[test]
fn an_opposing_elesh_norn_blanks_the_copys_trigger_so_a_copy_of_karn_is_worth_nothing() {
    let (mut g, karn, c) = karn_board();
    let before = value(&g, c, karn, true);
    assert!(before >= 7.0);
    perm(&mut g, P1, "Elesh Norn, Mother of Machines");
    assert_eq!(value(&g, c, karn, true), 0.0, "no enters trigger, and the legendary body is the real card's");
    let bat = perm(&mut g, P0, "Vampire Nighthawk");
    let v = value(&g, c, bat, true);
    assert!((v - 0.45).abs() < 1e-9, "a 2/3 flyer is a 0.45 body and no trigger: {v}");
    assert!(fc_opts(&mut g, None).is_empty(), "under the minimum of 0.5");
}

// ======================================================== 7. cost and state
#[test]
fn the_cost_the_crystals_state_and_a_pithing_needle_gate_the_option() {
    // one red source: {4}{R}{R} is unpayable
    let mut g = tbl();
    karn_on_the_table(&mut g);
    perm(&mut g, P0, CRYSTAL);
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Island", 5, false);
    assert!(fc_opts(&mut g, None).is_empty(), "one red source");
    lands(&mut g, P0, "Mountain", 1, false);
    assert_eq!(fc_opts(&mut g, None).len(), 1, "a second red source");

    // a tapped Crystal
    let (mut g, _, c) = karn_board();
    g.perm_mut(c).tapped = true;
    assert!(fc_opts(&mut g, None).is_empty(), "tapped");

    // a Pithing Needle naming the Crystal stops it (hook_options asks `ability_locked`)
    let (mut g, _, _) = karn_board();
    assert_eq!(fc_opts(&mut g, None).len(), 1);
    let needle = perm(&mut g, P1, "Pithing Needle");
    let card = card(&g, CRYSTAL);
    g.perm_mut(needle).data.set(DataKey::Imprint, Val::Card(card));
    assert!(fc_opts(&mut g, None).is_empty(), "the Needle names the Crystal");

    // a person at P0 gets none, and the person's own ability is still the Crystal's
    let (mut g, _, c) = karn_board();
    g.humans.add(P0, Arc::new(ScriptHuman::from_fn(|_| Answer::Action(json!({"do": "pass"})))));
    assert!(fc_opts(&mut g, None).is_empty(), "a person is never offered the AI's option");
    let own = sim_core::human::cards::abilities(&mut g, P0, c).expect("the person's ability of the Crystal");
    assert_eq!(own.len(), 1);
}

// ======================================================== 8. choice by value
#[test]
fn solitude_beats_strix_when_an_opposing_creature_is_worth_exiling() {
    let mut g = tbl();
    let solitude = perm(&mut g, P0, "Solitude"); // first: its trigger finds nothing worth exiling
    let strix = perm(&mut g, P0, "Baleful Strix");
    perm(&mut g, P1, "Terror of the Peaks"); // pval 7, placed last: it triggers on later creatures only
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let (vs, vx) = (value(&g, c, solitude, true), value(&g, c, strix, true));
    assert!(vs > vx + 3.0, "Solitude {vs}, Baleful Strix {vx}");
    let opts = fc_opts(&mut g, None);
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].label, "The Fire Crystal: copy Solitude");
}

#[test]
fn karn_beats_breya() {
    let mut g = tbl();
    let karn = karn_on_the_table(&mut g);
    let breya = perm(&mut g, P0, "Breya, Etherium Shaper");
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let (vk, vb) = (value(&g, c, karn, true), value(&g, c, breya, true));
    assert!(vb > 1.0, "two Thopters: {vb}");
    assert!(vk > vb + 3.0, "Karn {vk}, Breya {vb}");
    assert_eq!(fc_opts(&mut g, None)[0].label, "The Fire Crystal: copy Karn, Gilded Guardian");
}

#[test]
fn restoration_angel_beats_karn_when_it_can_blink_him_for_his_five_colours() {
    // the Angel's trigger blinks Karn (his draw again, for the colours among his other artifacts) and the copy is a 3/4
    // flyer: five coloured artifacts make the blink worth what the copy of Karn is, so the body decides
    let mut g = tbl();
    let angel = perm(&mut g, P0, "Restoration Angel"); // first: nothing to blink yet
    let karn = karn_on_the_table(&mut g);
    for ch in "WUBRG".chars() {
        let spec = zones::Tokens {
            types: vec!["thopter", "artifact"],
            color: Some(Colors::from_letters(&ch.to_string())),
            sick: false,
            ..zones::Tokens::new(1, 1)
        };
        zones::make_tokens(&mut g, P0, spec).unwrap();
    }
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let (va, vk) = (value(&g, c, angel, true), value(&g, c, karn, true));
    assert!(va > vk, "Restoration Angel {va}, Karn {vk}");
    assert_eq!(fc_opts(&mut g, None)[0].label, "The Fire Crystal: copy Restoration Angel");
}

#[test]
fn restoration_angel_with_nothing_worth_blinking_is_only_a_body() {
    let mut g = tbl();
    let angel = perm(&mut g, P0, "Restoration Angel");
    karn_on_the_table(&mut g); // blinking him draws only for the Crystal's red
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let v = value(&g, c, angel, true);
    assert!((v - 0.675).abs() < 1e-9, "a 3/4 flyer and no blink worth 3: {v}");
    assert_eq!(fc_opts(&mut g, None)[0].label, "The Fire Crystal: copy Karn, Gilded Guardian");
}

// ======================================================== the other enters triggers
#[test]
fn the_reaper_is_worth_the_creatures_its_counters_kill() {
    let mut g = tbl();
    let reaper = perm(&mut g, P0, "The Reaper, King No More"); // first: no opposing creature yet
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    assert_eq!(value(&g, c, reaper, true), 0.0, "nothing to shrink; legendary, so no body");
    let small = token(&mut g, P1, 1); // a 1/1: the -1/-1 counter kills it
    let v = value(&g, c, reaper, true);
    assert!((v - sim_core::engine::values::pval(&g, small)).abs() < 1e-9, "its pval: {v}");
    g.perm_mut(small).on_bf = false;
    token(&mut g, P1, 3); // a 3/3 survives: only the counter's 0.4
    assert!((value(&g, c, reaper, true) - 0.4).abs() < 1e-9);
}

#[test]
fn skyclave_apparition_is_worth_the_exile_less_the_illusion_its_owner_gets() {
    let mut g = tbl();
    let sky = perm(&mut g, P0, "Skyclave Apparition"); // first: nothing to exile yet
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let body = value(&g, c, sky, true);
    assert!(body < 0.5, "a body alone is under the minimum: {body}");
    let t = perm(&mut g, P1, "Goblin Chieftain"); // mana value 3, pval 3
    let pv = sim_core::engine::values::pval(&g, t);
    let v = value(&g, c, sky, true);
    assert!((v - body - (0.8 * pv - (0.5 + 0.25 * 3.0))).abs() < 1e-9, "0.8 x pval less the Illusion: {v}");
    assert!(v > 0.5, "worth copying: {v}");
}

#[test]
fn flickerwisp_is_worth_its_best_target() {
    let mut g = tbl();
    let wisp = perm(&mut g, P0, "Flickerwisp"); // first: an empty board
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let bare = value(&g, c, wisp, true);
    assert!((bare - 0.675).abs() < 1e-9, "a 3/1 flyer's body and no target: {bare}");
    let before = bare;
    lands(&mut g, P1, "Mountain", 1, false); // an untapped land of the player whose turn it is: mana lost to them
    let with = value(&g, c, wisp, true);
    assert!(with > before, "an opposing land to exile until the end step: {with} against {before}");
    assert!((with - before - 1.0).abs() < 1e-9, "an untapped land of the active player is worth 1.0 to exile: {with}");
}

// ======================================================== 9. the legend rule
#[test]
fn the_ai_copies_karn_while_the_real_karn_is_tapped_and_the_real_karn_stays() {
    let (mut g, karn, _) = karn_board();
    g.perm_mut(karn).tapped = true; // an untapped token would otherwise win the legend rule's tie-break
    let hand = g.player(P0).hand.len();
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert!(g.perm(karn).on_bf && g.perm(karn).is_cmd, "the real Karn is kept");
    assert!(g.perm(karn).tapped, "and he is still tapped");
    assert_eq!(named(&g, KARN).len(), 1);
    assert_eq!(g.player(P0).hand.len(), hand + 5);
}

// ======================================================== 10. Doomsday
#[test]
fn a_planned_doomsday_pile_blocks_every_copy_but_a_win() {
    let (mut g, karn, c) = karn_board();
    g.player_mut(P0).stat("doomsday", 1); // the pile is in play: `doomsday::active`
    assert_eq!(value(&g, c, karn, true), f64::NEG_INFINITY);
    assert!(fc_opts(&mut g, None).is_empty());
    // the Oracle's win comes before the Doomsday check
    let mut g = tbl();
    main_phase(&mut g);
    let oracle = perm(&mut g, P0, ORACLE);
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    g.player_mut(P0).library.truncate(4);
    g.player_mut(P0).stat("doomsday", 1);
    assert!(value(&g, c, oracle, false) >= 50.0);
    assert_eq!(fc_opts(&mut g, Some(false)).len(), 1);
}

// ======================================================== 11. purity
#[test]
fn copy_value_draws_no_random_numbers_and_changes_nothing() {
    let (mut g, karn, c) = karn_board();
    perm(&mut g, P0, "Solitude");
    let mut witness = g.clone();
    for m in g.player(P0).perms.clone() {
        if g.is_creature(m) {
            let _ = value(&g, c, m, true);
            let _ = value(&g, c, m, false);
        }
    }
    let _ = value(&g, c, karn, true);
    let _ = fc_opts(&mut g, None);
    assert_eq!(g.rng.next_u64(), witness.rng.next_u64(), "no draw from the game's generator");
    assert_eq!(g.player(P0).hand, witness.player(P0).hand);
    assert_eq!(g.player(P0).library.len(), witness.player(P0).library.len());
}

// ======================================================== 12. the hold rule (C6)
/// P0's main phase with Karn and the Crystal out and `lands` lands (two Mountains, two Swamps, Islands), and Vampire
/// Nighthawk ({1}{B}{B}) in hand: with seven mana casting it leaves four, which cannot pay {4}{R}{R}
fn hold_board(lands_n: usize) -> Game {
    let mut g = tbl();
    main_phase(&mut g);
    karn_on_the_table(&mut g);
    perm(&mut g, P0, CRYSTAL);
    self::lands(&mut g, P0, "Mountain", 2, false);
    self::lands(&mut g, P0, "Swamp", 2, false);
    self::lands(&mut g, P0, "Island", lands_n - 4, false);
    hand(&mut g, P0, &["Vampire Nighthawk"]);
    g
}

fn utility(opts: &[Opt], label: &str) -> f64 {
    opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("{label} not offered")).utility
}

fn stop(opts: &[Opt]) -> f64 {
    opts.iter().find(|o| o.act.is_none()).expect("a stop").utility
}

#[test]
fn the_hold_rule_cuts_a_cast_that_breaks_the_reserve_and_raises_stop() {
    table_fixes::force_crystal_hold(Some(false));
    let mut g = hold_board(7);
    let off = brain::main_options(&mut g, P0, false).unwrap();
    let off_cast = utility(&off, "Vampire Nighthawk");
    let off_stop = stop(&off);

    table_fixes::force_crystal_hold(Some(true));
    let hold = ai::table_fixes::crystal_hold_on();
    assert!(hold);
    let mut g = hold_board(7);
    let h = sim_core::impls::karn_gilded_cards::crystal_hold(&g, P0, 0.0).expect("a hold for the Karn copy");
    assert!((h - FC_HOLD_SHARE * 8.5).abs() < 1e-9, "0.6 of the copy's utility (1 + 7.5): {h}");
    let on = brain::main_options(&mut g, P0, false).unwrap();
    assert!(
        (utility(&on, "Vampire Nighthawk") - (off_cast - FC_HOLD_PEN * h)).abs() < 1e-9,
        "the cast is cut by 0.7 x hold"
    );
    assert!(stop(&on) >= h, "stop is worth at least the hold: {}", stop(&on));
    assert!(stop(&on) >= off_stop);
    assert_eq!(g.player(P0).stats.get("fc_hold"), Some(&1));

    // nine mana: the cast keeps {4}{R}{R}, it is not cut and the stop is not raised
    let mut g = hold_board(9);
    let nine = brain::main_options(&mut g, P0, false).unwrap();
    table_fixes::force_crystal_hold(Some(false));
    let mut g = hold_board(9);
    let nine_off = brain::main_options(&mut g, P0, false).unwrap();
    assert_eq!(utility(&nine, "Vampire Nighthawk"), utility(&nine_off, "Vampire Nighthawk"), "not cut");
    assert_eq!(stop(&nine), stop(&nine_off), "stop not raised");

    // the switch off with the Crystal ready equals a tapped Crystal (nothing to hold for)
    let mut g = hold_board(7);
    let c = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == CRYSTAL).unwrap();
    g.perm_mut(c).tapped = true;
    table_fixes::force_crystal_hold(Some(true));
    let tapped = brain::main_options(&mut g, P0, false).unwrap();
    assert_eq!(utility(&tapped, "Vampire Nighthawk"), off_cast, "a tapped Crystal holds nothing");
    assert_eq!(stop(&tapped), off_stop);
    table_fixes::force_crystal_hold(None);
}

#[test]
fn the_hold_rule_exempts_mana_sources_and_a_near_win() {
    table_fixes::force_crystal_hold(Some(true));
    let mut g = hold_board(7);
    let rock = hand(&mut g, P0, &["Thought Vessel"])[0];
    let night = card(&g, "Vampire Nighthawk");
    assert!(!sim_core::impls::karn_gilded_cards::crystal_reserve_broken(&g, P0, rock), "a mana rock is exempt");
    assert!(sim_core::impls::karn_gilded_cards::crystal_reserve_broken(&g, P0, night));
    assert!(sim_core::impls::karn_gilded_cards::crystal_hold(&g, P0, 0.8).is_none(), "an opponent near a win: no hold");
    table_fixes::force_crystal_hold(None);
}

// ======================================================== 13. the switch
#[test]
fn ai_crystal_off_gives_no_options_and_no_hold() {
    table_fixes::force_crystal(Some(false));
    let (mut g, _, _) = karn_board();
    assert!(fc_opts(&mut g, None).is_empty());
    assert!(sim_core::impls::karn_gilded_cards::crystal_hold(&g, P0, 0.0).is_none());
    table_fixes::force_crystal(None);
    assert_eq!(fc_opts(&mut g, None).len(), 1);
}

// ======================================================== the activation itself
#[test]
fn the_activation_pays_six_taps_the_crystal_and_marks_the_token_for_the_end_step() {
    let mut g = tbl();
    perm(&mut g, P0, "Orcish Bowmasters"); // a nonlegendary copy: both stay until the end step
    let c = perm(&mut g, P0, CRYSTAL);
    six_lands(&mut g);
    let opts = fc_opts(&mut g, None);
    assert_eq!(opts.len(), 1, "Bowmasters' trigger is worth 2.5");
    assert!(do_opt(&mut g, &opts[0]));
    assert!(g.perm(c).tapped);
    let toks = tokens_named(&g, "Orcish Bowmasters");
    assert_eq!(toks.len(), 1);
    assert_eq!(
        g.perm(toks[0]).data.get(DataKey::CopySac),
        Some(&Val::Player(P0)),
        "the controller is told to sacrifice it"
    );
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "exactly the six lands were spent");
    check_state(&mut g).unwrap();
}

// ======================================================== C6b: locks, the hand, the starvation bound, tutors
use sim_core::impls::karn_gilded::{DECKING, PER_CARD, PER_DISCARD};
use sim_core::impls::karn_gilded_cards::{crystal_hold, crystal_reserve_broken};

fn fill_hand(g: &mut Game, n: usize) {
    let c = card(g, "Island");
    g.player_mut(P0).hand.extend(std::iter::repeat_n(c, n));
}

fn untap_all(g: &mut Game) {
    for m in g.player(P0).perms.clone().into_iter().chain(g.player(P0).lands.clone()) {
        g.perm_mut(m).tapped = false;
    }
}

fn crystal_perm(g: &Game) -> PermId {
    named(g, CRYSTAL)[0]
}

/// the main-phase utilities of the hold board with the Crystal tapped: what the AI does with nothing to hold for
fn baseline_opts(lands_n: usize) -> Vec<Opt> {
    let mut g = hold_board(lands_n);
    let c = crystal_perm(&g);
    g.perm_mut(c).tapped = true;
    brain::main_options(&mut g, P0, false).unwrap()
}

/// the AI casts as if there were no Crystal: the cast keeps its utility and stop is not raised
fn assert_no_hold(g: &mut Game, why: &str) {
    assert!(crystal_hold(g, P0, 0.0).is_none(), "{why}: no hold");
    let base = baseline_opts(7);
    let now = brain::main_options(g, P0, false).unwrap();
    assert_eq!(utility(&now, "Vampire Nighthawk"), utility(&base, "Vampire Nighthawk"), "{why}: the cast is not cut");
    assert_eq!(stop(&now), stop(&base), "{why}: stop is not raised");
}

#[test]
fn a_lock_on_the_crystal_means_no_hold_and_the_ai_casts_normally() {
    table_fixes::force_crystal_hold(Some(true));
    // a Pithing Needle naming the Crystal
    let mut g = hold_board(7);
    assert!(crystal_hold(&g, P0, 0.0).is_some(), "without the Needle it holds");
    let needle = perm(&mut g, P1, "Pithing Needle");
    let c = card(&g, CRYSTAL);
    g.perm_mut(needle).data.set(DataKey::Imprint, Val::Card(c));
    assert_no_hold(&mut g, "Pithing Needle");
    // Collector Ouphe stops activated abilities of artifacts
    let mut g = hold_board(7);
    perm(&mut g, P1, "Collector Ouphe");
    assert_no_hold(&mut g, "Collector Ouphe");
    // Grand Abolisher of the seat before P0 stops it at the window (P1 is the seat before P0 at this table)
    let mut g = hold_board(7);
    perm(&mut g, P1, "Grand Abolisher");
    assert_no_hold(&mut g, "Grand Abolisher at the window");
    table_fixes::force_crystal_hold(None);
}

#[test]
fn the_held_mana_becomes_the_copy_end_to_end() {
    table_fixes::force_crystal_hold(Some(true));
    let mut g = hold_board(7);
    let hand0 = g.player(P0).hand.len();
    ai::main(&mut g, P0, false).unwrap();
    assert!(g.player(P0).hand.contains(&card(&g, "Vampire Nighthawk")), "the AI held the Nighthawk back");
    assert_eq!(g.player(P0).hand.len(), hand0);
    assert_eq!(g.player(P0).stats.get("fc_hold"), Some(&1), "a hold is counted once");
    // the opponents' turns, then the window before P0's turn
    g.round += 1;
    g.active = Some(P1);
    g.step = Step::End;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).stats.get("fc_acts_eot"), Some(&1), "the window copied");
    assert_eq!(g.player(P0).hand.len(), hand0 + 5, "Karn's five cards");
    table_fixes::force_crystal_hold(None);
}

#[test]
fn a_copy_of_karn_is_worth_the_cards_p_can_keep() {
    let value_with = |hand: usize, nomax: bool| {
        let (mut g, karn, c) = karn_board();
        fill_hand(&mut g, hand);
        if nomax {
            let t = g.player(P0).turns;
            g.player_mut(P0).nomax_turn = Some(t);
        }
        value(&g, c, karn, true)
    };
    let all = 5.0 * PER_CARD;
    assert_eq!(value_with(0, false), all, "an empty hand keeps all five");
    assert_eq!(value_with(4, false), all, "seven - 4 + 2 = 5 places");
    assert_eq!(value_with(5, false), 4.0 * PER_CARD + PER_DISCARD, "four places");
    assert_eq!(value_with(7, false), 2.0 * PER_CARD + 3.0 * PER_DISCARD, "a full hand keeps two");
    assert_eq!(value_with(7, true), all, "no maximum hand size: nothing is discarded");
    // the library guard still wins over the hand
    let (mut g, karn, c) = karn_board();
    g.player_mut(P0).library.truncate(6);
    assert_eq!(value(&g, c, karn, true), DECKING);
}

#[test]
fn a_karn_copy_with_a_full_hand_is_still_made_in_the_window() {
    let (mut g, _, _) = karn_board();
    fill_hand(&mut g, 7);
    let opts = fc_opts(&mut g, None);
    assert_eq!(opts.len(), 1);
    assert!((opts[0].utility - (1.0 + 2.0 * PER_CARD + 3.0 * PER_DISCARD)).abs() < 1e-9, "{}", opts[0].utility);
    assert!(do_opt(&mut g, &opts[0]));
    assert_eq!(g.player(P0).hand.len(), 7 + 5, "the cards arrive; the cleanup discards them later");
}

#[test]
fn a_hand_of_seven_is_cast_from_not_held_against() {
    table_fixes::force_crystal_hold(Some(true));
    let mut g = hold_board(7);
    fill_hand(&mut g, 5); // Nighthawk and five Islands: six cards, room for 3 new ones: worth 5.0, over the minimum
    assert!(crystal_hold(&g, P0, 0.0).is_some(), "six cards: still held for");
    fill_hand(&mut g, 1);
    assert!(crystal_hold(&g, P0, 0.0).is_none(), "seven cards: cast something instead");
    // with no maximum hand size the copy is worth all five cards, and the guard alone says no
    let t = g.player(P0).turns;
    g.player_mut(P0).nomax_turn = Some(t);
    let c = crystal_perm(&g);
    let karn = named(&g, KARN)[0];
    assert_eq!(value(&g, c, karn, true), 5.0 * PER_CARD);
    assert!(crystal_hold(&g, P0, 0.0).is_none(), "seven cards: the guard, not the value");
    table_fixes::force_crystal_hold(None);
}

#[test]
fn a_hold_whose_window_did_not_copy_is_not_repeated_the_next_turn() {
    table_fixes::force_crystal_hold(Some(true));
    let mut g = hold_board(7);
    ai::main(&mut g, P0, false).unwrap(); // holds: round r
    assert_eq!(g.player(P0).stats.get("fc_hold"), Some(&1));
    // the window before the next turn: the Crystal is tapped (or locked, or Karn is gone), so nothing is copied
    g.round += 1;
    g.active = Some(P1);
    g.step = Step::End;
    let c = crystal_perm(&g);
    g.perm_mut(c).tapped = true;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).stats.get("fc_acts"), None, "no copy in the window");
    // P0's untap step, then its main phase (round r + 1): the reason to hold is back, but it held last turn for nothing
    untap_all(&mut g);
    main_phase(&mut g);
    assert!(crystal_hold(&g, P0, 0.0).is_none(), "no second hold in a row");
    // the turn after that, it may hold again
    g.round += 1;
    assert!(crystal_hold(&g, P0, 0.0).is_some(), "the bound lasts one turn");
    table_fixes::force_crystal_hold(None);
}

#[test]
fn a_hold_whose_window_copied_may_be_repeated() {
    table_fixes::force_crystal_hold(Some(true));
    let mut g = hold_board(7);
    ai::main(&mut g, P0, false).unwrap();
    g.round += 1;
    g.active = Some(P1);
    g.step = Step::End;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).stats.get("fc_acts"), Some(&1));
    untap_all(&mut g);
    main_phase(&mut g);
    assert!(crystal_hold(&g, P0, 0.0).is_some(), "the hold paid off: hold again");
    table_fixes::force_crystal_hold(None);
}

#[test]
fn a_game_without_a_crystal_or_a_hold_is_never_bound() {
    let mut g = tbl();
    main_phase(&mut g);
    assert!(crystal_hold(&g, P0, 0.0).is_none(), "no Crystal: nothing to hold");
    g.round += 5;
    assert!(g.player(P0).flag_turn.is_empty(), "no flag was set");
}

#[test]
fn a_cheap_tutor_is_exempt_from_the_reserve_and_a_dear_one_is_not() {
    let mut g = hold_board(7);
    let ids = hand(&mut g, P0, &["Demonic Tutor", "Vampiric Tutor", "Mystical Tutor", "Ringsight"]);
    for &c in &ids[..3] {
        assert!(!crystal_reserve_broken(&g, P0, c), "{}", g.db.get(c).name);
    }
    assert!(crystal_reserve_broken(&g, P0, ids[3]), "a three-mana tutor is an ordinary cast");
}

#[test]
fn a_stale_option_spends_nothing() {
    let stale = |what: &str, spoil: &dyn Fn(&mut Game, PermId, PermId)| {
        let (mut g, karn, c) = karn_board();
        let opts = fc_opts(&mut g, None);
        assert_eq!(opts.len(), 1);
        let mana = sim_core::engine::mana::total_mana(&g, P0, false);
        let hand = g.player(P0).hand.len();
        spoil(&mut g, karn, c);
        let before = sim_core::engine::mana::total_mana(&g, P0, false);
        assert!(!do_opt(&mut g, &opts[0]), "{what}: refused");
        assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), before, "{what}: nothing was spent");
        assert_eq!(g.player(P0).hand.len(), hand, "{what}: nothing drawn");
        assert!(before <= mana);
        assert_eq!(g.player(P0).stats.get("fc_acts"), None, "{what}: no activation counted");
    };
    stale("the Crystal is tapped", &|g, _, c| g.perm_mut(c).tapped = true);
    stale("the target is gone", &|g, karn, _| g.perm_mut(karn).on_bf = false);
    stale("the cost is no longer payable", &|g, _, _| {
        for l in g.player(P0).lands.clone().into_iter().take(2) {
            g.perm_mut(l).tapped = true;
        }
    });
    stale("a lock appeared", &|g, _, c| {
        let needle = perm(g, P1, "Pithing Needle");
        let card = card(g, CRYSTAL);
        g.perm_mut(needle).data.set(DataKey::Imprint, Val::Card(card));
        let _ = c;
    });
}
