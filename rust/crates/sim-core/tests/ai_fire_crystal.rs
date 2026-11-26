//! The AI plays The Fire Crystal ({4}{R}{R}, {T}: create a token that's a copy of target creature you control; sacrifice it
//! at the beginning of the next end step), in the Karn, Gilded Guardian deck. The expected results come from the card's
//! Oracle text and the design (/tmp/handoff/opus-fire-design-out.md, D3), on hand-built boards:
//!   - the AI copies in the end-of-turn window just before its own turn (the mana and the Crystal are free there), the best
//!     target by `karn_gilded_cards::copy_value`; in a main phase it copies only to win with Thassa's Oracle;
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
use sim_core::impls::karn_gilded_cards::copy_value;
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
    assert_eq!(with, before + sim_core::impls::t12_brago_meren::wisp_value(&g, P0, wisp));
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

// ======================================================== 13. the switch
#[test]
fn ai_crystal_off_gives_no_options() {
    table_fixes::force_crystal(Some(false));
    let (mut g, _, _) = karn_board();
    assert!(fc_opts(&mut g, None).is_empty());
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
