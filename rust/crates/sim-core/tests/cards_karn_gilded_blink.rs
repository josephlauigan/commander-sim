//! Blinking Karn, Gilded Guardian (E4): the AI blinks him for his enter trigger ("draw a card for each color among other
//! artifacts you control") with the seven blinkers of the Karn deck, he stays the commander through each blink, and the
//! decking and hand-size guards hold. The expected results come from the card's Oracle text, the comprehensive rules
//! (903.9a: a commander in exile may stay there; 400.7: a blinked card is a new object) and the design consult
//! (`/tmp/handoff/opus-e4-design.md`), not from what the engine did before. Card code: impls/karn_gilded.rs
//! (`reenter_bonus`); the choosers it feeds are driven here the way the AI drives them (hand options, end-step triggers,
//! the protection response), not only called directly.

mod play_common;
use play_common::*;
use sim_core::ai;
use sim_core::cards::Colors;
use sim_core::engine::cast::cast_card;
use sim_core::engine::zones::{self, Tokens};
use sim_core::engine::{hooks, turn};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::CardId;
use sim_core::impls::karn_gilded::{DECKING, STOLEN, reenter_bonus, reenter_now};
use sim_core::impls::{galadriel, mine, t2};
use sim_core::state::{Ctx, Step};

const KARN: &str = "Karn, Gilded Guardian";
const OPP: &str = "krenko-mono-red-goblins";
const TAX: u32 = 4;

/// P0 plays a generic deck (a 99-card library), P1 a red deck with an empty board; it is P0's turn, an empty hand
fn tbl() -> Game {
    let mut g = table(&["sheoldred", OPP]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

/// Karn on the battlefield as P0's commander (the tax is TAX, he is not in the command zone). Put him down before any
/// artifact: his trigger draws nothing yet
fn karn(g: &mut Game) -> PermId {
    set_commander(g, P0, KARN);
    let m = perm(g, P0, KARN);
    let pl = g.player_mut(P0);
    pl.cmd_in_zone = false;
    pl.tax = TAX;
    g.perm_mut(m).is_cmd = true;
    m
}

/// one coloured artifact creature token of P0's for each letter of `cols`: that many colours among his other artifacts
fn colours(g: &mut Game, cols: &str) {
    for ch in cols.chars() {
        let spec = Tokens {
            types: vec!["thopter", "artifact"],
            color: Some(Colors::from_letters(&ch.to_string())),
            sick: false,
            ..Tokens::new(1, 1)
        };
        zones::make_tokens(g, P0, spec).unwrap();
    }
}

fn fill_hand(g: &mut Game, n: usize) {
    let c = card(g, "Island");
    g.player_mut(P0).hand.extend(std::iter::repeat_n(c, n));
}

fn library(g: &mut Game, n: usize) {
    g.player_mut(P0).library.truncate(n);
}

fn end_step(g: &mut Game) {
    g.step = Step::End;
    g.active = Some(P0);
}

fn fire(g: &mut Game, e: Event, p: PlayerId) {
    hooks::fire_trigger(g, e, Call::Player { p }).unwrap();
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        _ => panic!("not a card action"),
    }
}

fn hand_options(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, P0, post).unwrap()
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// Karn was blinked: the old permanent is gone, one new Karn is on the battlefield, still the commander, the tax is
/// unchanged, he never sat in the command zone; `drawn` more cards than `hand_before` are in hand
fn assert_blinked(g: &Game, old: PermId, hand_before: usize, drawn: usize) -> PermId {
    assert!(!on_bf(g, old), "the old permanent is gone");
    let now = perms_named(g, P0, KARN);
    assert_eq!(now.len(), 1, "one Karn on the battlefield");
    assert_ne!(now[0], old, "a new object");
    assert!(g.perm(now[0]).is_cmd, "still the commander");
    assert_eq!(g.player(P0).tax, TAX, "the tax is unchanged");
    assert!(!g.player(P0).cmd_in_zone, "he never went to the command zone");
    assert_eq!(g.player(P0).hand.len(), hand_before + drawn, "his trigger drew");
    now[0]
}

fn assert_stays(g: &Game, k: PermId) {
    assert!(on_bf(g, k), "Karn was not blinked");
}

// ------------------------------------------------------------------ the blinkers
#[test]
fn ephemerate_blinks_karn() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Ephemerate"])[0];
    let o = hand_options(&mut g, c, Some(false));
    assert_eq!(o.len(), 1, "two colours: worth the spell");
    assert!(o[0].label.contains(KARN), "{}", o[0].label);
    let before = g.player(P0).hand.len() - 1; // the spell leaves the hand
    assert!(run(&mut g, P0, &o[0]));
    assert_blinked(&g, k, before, 2);
    assert!(g.player(P0).rebound.contains(&c), "rebound is set");
}

#[test]
fn ephemerate_rebound_blinks_karn() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    let c = take(&mut g, P0, "Ephemerate");
    g.player_mut(P0).exile.push(c);
    g.player_mut(P0).rebound.push(c);
    let before = g.player(P0).hand.len();
    sim_core::cardcode::turn_start(&mut g, P0).unwrap();
    assert_blinked(&g, k, before, 2);
    assert!(g.player(P0).gy.contains(&c), "the rebound cast went to the graveyard");
}

#[test]
fn cloudshift_blinks_karn_and_keeps_is_cmd() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let before = g.player(P0).hand.len() - 1;
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_blinked(&g, k, before, 2);
}

#[test]
fn resto_blinks_karn_at_opponent_end_step() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &["Restoration Angel"])[0];
    g.active = Some(P1); // the end of the turn before yours
    let o = hand_options(&mut g, c, None);
    assert_eq!(o.len(), 1, "the end-of-turn window offers it");
    assert_eq!(o[0].label, "Restoration Angel (end of turn)");
    let before = g.player(P0).hand.len() - 1;
    assert!(run(&mut g, P0, &o[0]));
    assert_blinked(&g, k, before, 2);
    assert!(perms_named(&g, P0, "Restoration Angel").len() == 1, "the Angel is on the battlefield");
}

#[test]
fn felidar_blinks_karn() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    let before = g.player(P0).hand.len();
    perm(&mut g, P0, "Felidar Guardian");
    assert_blinked(&g, k, before, 2);
}

#[test]
fn circle_blinks_karn_at_end_step() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_blinked(&g, k, before, 2);
}

#[test]
fn harnessed_mind_stone_blinks_karn() {
    let mut g = tbl();
    let k = karn(&mut g);
    let stone = perm(&mut g, P0, "The Mind Stone"); // white
    perm(&mut g, P0, "The Soul Stone"); // black: two colours
    lands(&mut g, P0, "Plains", 8, false);
    let c = g.perm(stone).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options");
    let o = f(&mut g, stone, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1, "the harness is offered with Karn at two colours");
    assert_eq!(o[0].label, "harness The Mind Stone");
    assert!(run(&mut g, P0, &o[0]));
    end_step(&mut g);
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_blinked(&g, k, before, 2);
}

#[test]
fn flickerwisp_karn_waits_in_exile_and_returns() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    let before = g.player(P0).hand.len();
    perm(&mut g, P0, "Flickerwisp");
    let kc = card(&g, KARN);
    assert!(!on_bf(&g, k));
    assert!(g.player(P0).exile.contains(&kc), "he waits in exile");
    assert!(!g.player(P0).cmd_in_zone, "not in the command zone (CR 903.9a: the owner leaves him there)");
    assert_eq!(g.player(P0).tax, TAX);
    assert_eq!(g.player(P0).hand.len(), before, "no draw while he is away");
    end_step(&mut g);
    galadriel::eot_returns(&mut g).unwrap();
    assert!(!g.player(P0).exile.contains(&kc));
    assert_blinked(&g, k, before, 2);
}

// ------------------------------------------------------------------ protection
#[test]
fn ephemerate_saves_karn_from_swords() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Ephemerate"])[0];
    let swords = card(&g, "Swords to Plowshares");
    let before = g.player(P0).hand.len() - 1;
    assert!(ai::protect_response(&mut g, P0, k, "exile", Some(P1), Some(swords)).unwrap());
    assert_blinked(&g, k, before, 2);
    assert!(g.player(P0).rebound.contains(&c));
}

#[test]
fn cloudshift_saves_karn_from_removal() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let swords = card(&g, "Swords to Plowshares");
    let before = g.player(P0).hand.len() - 1;
    assert!(ai::protect_response(&mut g, P0, k, "exile", Some(P1), Some(swords)).unwrap());
    assert_blinked(&g, k, before, 2);
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn cloudshift_does_not_protect_other_creatures() {
    // Cloudshift is a protector for Karn only: every other deck keeps holding it for value or a combo
    let mut g = tbl();
    let t = perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let swords = card(&g, "Swords to Plowshares");
    assert!(!ai::protect_response(&mut g, P0, t, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).hand.contains(&c), "Cloudshift stays in hand");
}

#[test]
fn no_blink_protection_when_it_would_deck() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Ephemerate", "Cloudshift"])[0];
    library(&mut g, 5); // two draws leave three: the floor
    let swords = card(&g, "Swords to Plowshares");
    assert!(!ai::protect_response(&mut g, P0, k, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).hand.contains(&c), "Ephemerate was not cast");
    assert_stays(&g, k);
}

// ------------------------------------------------------------------ the decking guard
#[test]
fn no_blink_when_library_low() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    library(&mut g, 5); // n = 2: three would be left
    assert!(reenter_now(&g, P0, k).unwrap() < 0.0);
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(DECKING));
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k);
    // no other chooser picks him either
    assert!(t2::blink_value(&g, P0, k) < 0);
    assert!(mine::blink_worth(&g, P0, k) < 0.0);
}

#[test]
fn library_n_plus_4_is_fine() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    library(&mut g, 6);
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(3.0));
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_blinked(&g, k, before, 2);
}

#[test]
fn gandalf_doubles_the_count_in_the_guard() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUBRG");
    library(&mut g, 12);
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(7.5), "five draws, 12 left minus 5 is above the floor");
    perm(&mut g, P0, "Gandalf the White");
    // two triggers: ten draws from twelve cards
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(DECKING));
}

#[test]
fn opposing_norn_makes_him_worthless() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    perm(&mut g, P1, "Elesh Norn, Mother of Machines");
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(0.0));
    let wall = perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k);
    assert!(!on_bf(&g, wall), "the Circle picked the Wall of Omens instead");
}

#[test]
fn doomsday_pile_never_blinks() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    g.player_mut(P0).stat("doomsday", 1);
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(0.0), "the pile is planned");
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k);
}

/// library exactly 5, five colours, Thassa's Oracle in hand and UU open
fn oracle_board() -> (Game, PermId) {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUBRG");
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle"]);
    library(&mut g, 5);
    (g, k)
}

#[test]
fn oracle_line_lifts_the_guard_in_main_phase() {
    let (g, k) = oracle_board();
    assert_eq!(reenter_now(&g, P0, k), Some(7.5), "the draws empty the library; the Oracle wins before the next draw");
}

#[test]
fn oracle_line_not_at_end_step() {
    let (mut g, k) = oracle_board();
    end_step(&mut g);
    assert_eq!(reenter_now(&g, P0, k), Some(DECKING));
}

#[test]
fn oracle_line_never_overdraws() {
    let (mut g, k) = oracle_board();
    library(&mut g, 4); // n = 5: the fifth draw is from an empty library, a loss at the next state check
    assert_eq!(reenter_now(&g, P0, k), Some(DECKING));
}

// ------------------------------------------------------------------ the hand-size guard
#[test]
fn circle_skips_karn_at_full_hand_without_nomax() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUB");
    fill_hand(&mut g, 7);
    end_step(&mut g);
    assert_eq!(reenter_now(&g, P0, k), Some(0.75), "three cards, all discarded at the cleanup step");
    let wall = perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, "Teleportation Circle");
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k);
    assert!(!on_bf(&g, wall), "the Circle blinked the Wall of Omens instead");
}

#[test]
fn circle_picks_karn_with_reliquary_tower() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUB");
    fill_hand(&mut g, 7);
    lands(&mut g, P0, "Reliquary Tower", 1, false);
    perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    assert_eq!(reenter_now(&g, P0, k), Some(4.5), "no maximum hand size: all three count");
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_blinked(&g, k, before, 3);
}

#[test]
fn circle_picks_karn_with_thought_vessel() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUB");
    fill_hand(&mut g, 7);
    perm(&mut g, P0, "Thought Vessel");
    perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    assert_eq!(reenter_now(&g, P0, k), Some(4.5));
    let before = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_blinked(&g, k, before, 3);
}

#[test]
fn cap_counts_room() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WUBRG");
    fill_hand(&mut g, 4);
    assert_eq!(
        reenter_bonus(&g, P0, k, true),
        Some(1.5 * 3.0 + 0.25 * 2.0),
        "room for three of the five at the end step"
    );
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(7.5), "not at an end step: no cleanup discard follows");
}

// ------------------------------------------------------------------ ownership and repeats
#[test]
fn stolen_karn_is_never_blinked() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    g.perm_mut(k).orig = P1; // P0 controls him, P1 owns him: he returns to P1, who draws
    assert_eq!(reenter_bonus(&g, P0, k, false), Some(STOLEN));
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k);
}

#[test]
fn two_blinks_in_one_turn_reread_the_guard() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    lands(&mut g, P0, "Plains", 1, false);
    library(&mut g, 6); // n = 2: four are left after one blink, but three are not enough after a second
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let before = g.player(P0).hand.len() - 1;
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    let k2 = assert_blinked(&g, k, before, 2);
    assert_eq!(g.player(P0).library.len(), 4);
    perm(&mut g, P0, "Teleportation Circle");
    end_step(&mut g);
    assert_eq!(reenter_now(&g, P0, k2), Some(DECKING), "the second blink would leave two cards");
    let hand_now = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert_stays(&g, k2);
    assert_eq!(g.player(P0).hand.len(), hand_now);
}

#[test]
fn opponent_swords_on_karn_still_goes_to_command_zone() {
    let mut g = tbl();
    let k = karn(&mut g);
    colours(&mut g, "WB");
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), k, "exile", None).unwrap();
    assert!(!on_bf(&g, k));
    assert!(g.player(P0).cmd_in_zone, "the engine moves a commander to the command zone");
    assert!(!g.player(P0).exile.contains(&card(&g, KARN)));
}

#[test]
fn opponent_flickerwisp_on_karn_goes_to_command_zone() {
    // a kept limit: the opponent's Flickerwisp path is left alone (the owner is not asked)
    let mut g = tbl();
    let k = karn(&mut g);
    perm(&mut g, P1, "Flickerwisp");
    assert!(!on_bf(&g, k));
    assert!(g.player(P0).cmd_in_zone, "he is in the command zone");
    assert!(!g.player(P0).exile.contains(&card(&g, KARN)));
}

// ------------------------------------------------------------------ regression: everything else is unchanged
#[test]
fn reenter_bonus_is_none_for_everything_else() {
    let mut g = tbl();
    for n in ["Yoshimaru, Ever Faithful", "Tymna the Weaver", "Thassa's Oracle", "Wall of Omens"] {
        let m = perm(&mut g, P0, n);
        assert_eq!(reenter_bonus(&g, P0, m, false), None, "{n}");
        assert_eq!(reenter_now(&g, P0, m), None, "{n}");
    }
    // an opponent's Karn is not P0's
    let theirs = perm(&mut g, P1, KARN);
    assert_eq!(reenter_bonus(&g, P0, theirs, false), None);
}

#[test]
fn a_copy_of_karn_is_not_karn() {
    // keyed on the physical card (the Cursed Mirror), so a copy keeps today's value
    let mut g = tbl();
    perm(&mut g, P1, KARN);
    let mirror = hand(&mut g, P0, &["Cursed Mirror"])[0];
    cast_card(&mut g, P0, mirror, "hand", Ctx::default()).unwrap();
    let copy = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(mirror)).expect("the copy");
    assert_eq!(&*g.perm(copy).name, KARN);
    assert_eq!(reenter_bonus(&g, P0, copy, false), None, "a copy of Karn is not Karn");
}

#[test]
fn blink_value_of_other_commanders_still_zero() {
    let mut g = tbl();
    set_commander(&mut g, P0, "Yoshimaru, Ever Faithful");
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    g.perm_mut(y).is_cmd = true;
    assert_eq!(t2::blink_value(&g, P0, y), 0, "a commander is never blinked by the generic value");
    let w = perm(&mut g, P0, "Wall of Omens");
    assert_eq!(t2::blink_value(&g, P0, w), 3, "an ordinary creature keeps its value");
}

#[test]
fn no_max_hand_refactor_matches() {
    // the cleanup step's discard to seven reads `no_max_hand`: three cases, each checked through the step itself
    let mut g = tbl();
    assert!(!turn::no_max_hand(&g, P0));
    fill_hand(&mut g, 9);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 7, "neither: discards to seven");

    let mut g = tbl();
    lands(&mut g, P0, "Reliquary Tower", 1, false);
    assert!(turn::no_max_hand(&g, P0), "Reliquary Tower");
    fill_hand(&mut g, 9);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 9);

    let mut g = tbl();
    perm(&mut g, P0, "Thought Vessel");
    assert!(turn::no_max_hand(&g, P0), "Thought Vessel");
    fill_hand(&mut g, 9);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 9);

    let mut g = tbl();
    g.player_mut(P0).nomax_turn = Some(g.player(P0).turns);
    assert!(turn::no_max_hand(&g, P0), "a turn-long effect");
    let mut g = tbl();
    g.player_mut(P0).t0s.nomax = true;
    assert!(turn::no_max_hand(&g, P0), "the Stella flag");
}
