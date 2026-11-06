//! doomsday-led (rust/agents/doomsday-led.md): Lion's Eye Diamond as Doomsday mana in JD's Tymna/Thrasios deck.
//!
//! LED: "{0} artifact. Discard your hand, Sacrifice Lion's Eye Diamond: Add three mana of any one color. Activate only as
//! an instant." It is a mana ability, so it can be activated while a spell is being cast, with the spell already on the
//! stack: the spell is not discarded.
//!
//! 1. LED pays Doomsday's {B}{B}{B} when the lands have no black; the discarded hand is in the graveyard, where Doomsday
//!    (library and graveyard) takes it into the pile. Under Necropotence the discards are exiled, so it cannot.
//! 2. LED cracked inside the pile pays a spell the lands can not, as late as possible, and the line still wins; it is
//!    never cracked when the hand it would discard holds a card the line needs.
//! 3. The AI casts LED from hand with Doomsday in hand (plan file), and holds it without.
//! 4. A control: with the black mana there and the hand worth keeping, the plain cast is taken and LED stays.

mod play_common;
use play_common::*;
use sim_core::ai::brain::end_of_turn_window;
use sim_core::impls::combos;
use sim_core::impls::doomsday::{self, NAME};
use sim_core::state::Step;

const LED: &str = "Lion's Eye Diamond";

fn ids(g: &Game, names: &[&str]) -> Vec<sim_core::ids::CardId> {
    names.iter().map(|n| card(g, n)).collect()
}

/// `thrasios` at P0 and Sauron at P1 (a hand of three cards, three Islands), P0's own main phase, 40 life; P0 has
/// `islands` Islands and `plains` Plains (no black), the diamond on the battlefield, and exactly this hand, library (top
/// first) and graveyard
fn setup(islands: usize, plains: usize, hand_names: &[&str], lib_top_first: &[&str], gy: &[&str]) -> Game {
    let mut g = table_with(&["thrasios", "sauron"], 1, 40);
    g.log = Some(vec![]);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", islands, false);
    lands(&mut g, P0, "Plains", plains, false);
    perm(&mut g, P0, LED);
    let (h, l, y) = (ids(&g, hand_names), ids(&g, lib_top_first), ids(&g, gy));
    let pl = g.player_mut(P0);
    pl.hand = h;
    pl.library = l.into_iter().rev().collect();
    pl.gy = y;
    lands(&mut g, P1, "Island", 3, false);
    hand(&mut g, P1, &["Sol Ring", "Mana Vault", "Grim Monolith"]);
    g
}

fn logged(g: &Game, text: &str) -> bool {
    g.log.as_ref().is_some_and(|l| l.iter().any(|x| x.contains(text)))
}

fn log_pos(g: &Game, text: &str) -> Option<usize> {
    g.log.as_ref().and_then(|l| l.iter().position(|x| x.contains(text)))
}

fn won(g: &Game) -> bool {
    g.over && g.winner == Some(P0)
}

fn names(g: &Game, cards: &[sim_core::ids::CardId]) -> Vec<String> {
    cards.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

fn on_bf(g: &Game, name: &str) -> bool {
    g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
}

fn doomsday_option(g: &mut Game, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let c = card(g, "Doomsday");
    doomsday::doomsday_options(g, c, P0, post).unwrap()
}

/// the cast of Doomsday as the AI makes it: the option, performed
fn cast_doomsday(g: &mut Game, post: Option<bool>) {
    let o = doomsday_option(g, post).remove(0);
    sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap();
}

// ------------------------------------------------------------------ 1. LED pays Doomsday
/// no black mana at all (four Islands, a Plains), Teferi's +1 in force: the Oracle and a Preordain in hand are discarded by
/// the diamond and Doomsday takes them from the graveyard, because the library holds neither
fn no_black_position() -> Game {
    let mut g = setup(
        4,
        1,
        &["Doomsday", "Thassa's Oracle", "Preordain"],
        &["Gitaxian Probe", "Ponder", "Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith", "Sol Ring"],
        &[],
    );
    let turns = g.player(P0).turns;
    g.player_mut(P0).sorc_flash = Some(turns);
    g
}

#[test]
fn led_pays_doomsday_with_no_other_black_and_the_pile_takes_the_discarded_oracle() {
    let mut g = no_black_position();
    g.active = Some(P1);
    g.step = Step::End;
    let (oracle, preordain) = (card(&g, "Thassa's Oracle"), card(&g, "Preordain"));
    let c = card(&g, "Doomsday");
    let (built, _) =
        doomsday::plan_cast(&g, P0, c, doomsday::Scn::EndStep).expect("the diamond makes the cast possible");
    assert!(built.led, "no {{B}} but the diamond's: it pays");
    assert!(built.pile.contains(&oracle), "the Oracle from the graveyard is in the pile: {:?}", names(&g, &built.pile));
    assert!(built.pile.contains(&preordain), "so is the Preordain: {:?}", names(&g, &built.pile));
    end_of_turn_window(&mut g, P0).unwrap();
    assert!(logged(&g, "casts Doomsday"), "{:?}", g.log);
    assert!(
        log_pos(&g, "cracks Lion's Eye Diamond for BBB to pay for Doomsday").unwrap() > log_pos(&g, "casts Doomsday").unwrap(),
        "the crack comes while the spell is being cast: {:?}",
        g.log
    );
    assert!(!on_bf(&g, LED), "the diamond was sacrificed");
    assert!(g.player(P0).hand.is_empty(), "the hand was discarded");
    assert_eq!(g.player(P0).life, 20, "Doomsday was on the stack when the hand went: it was not discarded, and it resolved");
    let lib = names(&g, &g.player(P0).library.iter().rev().copied().collect::<Vec<_>>());
    assert!(lib.iter().any(|n| n == "Thassa's Oracle"), "the Oracle that was in the hand is on top of the library: {lib:?}");
    // P0's next turn: untap, draw, play the pile
    g.round += 1;
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    let i = combos::combo_index(NAME).expect("the pile line is in the catalogue");
    assert!(combos::ready(&g, P0, i), "the pile line is ready");
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert!(won(&g), "Thassa's Oracle wins with the pile that came out of the graveyard");
}

#[test]
fn in_the_main_phase_the_diamond_does_not_pay_because_an_empty_hand_cannot_start_the_pile() {
    // the same cards on P0's own turn: the hand is the pile's start (there is no draw before the Oracle), the diamond throws
    // it away, and the lands have no {B}: no cast
    let mut g = no_black_position();
    assert_eq!(doomsday_option(&mut g, Some(false)).len(), 0);
}

#[test]
fn without_the_diamond_the_same_hand_has_no_cast() {
    let mut g = no_black_position();
    g.active = Some(P1);
    assert_eq!(doomsday_option(&mut g, None).len(), 1, "with the diamond: the cast");
    let m = g.player(P0).perms.clone().into_iter().find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == LED)).unwrap();
    sim_core::engine::zones::leave(&mut g, m).unwrap();
    assert_eq!(doomsday_option(&mut g, None).len(), 0, "no black source: Doomsday can not be paid");
}

#[test]
fn necropotence_exiles_the_discards_so_the_pile_cannot_take_them() {
    let mut g = no_black_position();
    g.active = Some(P1);
    perm(&mut g, P0, "Necropotence");
    assert_eq!(
        doomsday_option(&mut g, None).len(),
        0,
        "the Oracle would be exiled by the discard (the guide's mistake 6): there is no pile"
    );
}

// ------------------------------------------------------------------ 2. LED inside the pile
/// a pile in play: Doomsday resolved, the hand and library (top first) as given, the diamond on the battlefield
fn pile_in_play(islands: usize, plains: usize, hand_names: &[&str], lib_top_first: &[&str]) -> Game {
    let mut g = setup(islands, plains, hand_names, lib_top_first, &[]);
    g.player_mut(P0).stats.insert("doomsday", 1);
    g
}

#[test]
fn led_cracked_inside_the_pile_pays_the_oracle_and_the_line_wins() {
    // one Island and one Plains: Silence ({W}) is paid by the Plains first, and the Oracle's {U}{U} by the diamond, cracked
    // with the Oracle on the stack (nothing else is in the hand to lose)
    let mut g = pile_in_play(1, 1, &["Thassa's Oracle", "Silence"], &["Chrome Mox", "Lotus Petal"]);
    doomsday::run_pile(&mut g, P0).unwrap();
    assert!(won(&g), "the Oracle won: {:?}", g.log);
    let crack = log_pos(&g, "cracks Lion's Eye Diamond for UUU to pay for Thassa's Oracle").expect("the diamond paid the Oracle");
    let silence = log_pos(&g, "plays the pile: Silence").expect("Silence was cast");
    assert!(silence < crack, "the diamond is cracked as late as it can be: Silence first, with the hand still whole");
    assert!(!on_bf(&g, LED));
}

#[test]
fn led_is_not_cracked_when_it_would_discard_the_oracle_the_line_needs() {
    // no lands at all, the Oracle and a Preordain in hand, three cards in the library (the Oracle's devotion of 2 allows two):
    // the line needs the Preordain's draw first, and the only way to pay for it is a crack that throws the Oracle away
    let mut g = pile_in_play(0, 0, &["Thassa's Oracle", "Preordain"], &["Chrome Mox", "Lotus Petal", "Arcane Signet"]);
    doomsday::run_pile(&mut g, P0).unwrap();
    assert!(!won(&g));
    assert!(on_bf(&g, LED), "the diamond is still on the battlefield");
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Thassa's Oracle"), "the Oracle is still in hand");
    assert!(!logged(&g, "cracks Lion's Eye Diamond"), "{:?}", g.log);
    // with one card fewer in the library the Oracle itself is the cast the diamond pays for (the Preordain it discards is not
    // needed)
    let mut g = pile_in_play(0, 0, &["Thassa's Oracle", "Preordain"], &["Chrome Mox", "Lotus Petal"]);
    doomsday::run_pile(&mut g, P0).unwrap();
    assert!(won(&g), "{:?}", g.log);
    assert!(logged(&g, "cracks Lion's Eye Diamond for UUU to pay for Thassa's Oracle"));
}

#[test]
fn led_pays_a_cantrip_in_the_middle_of_a_line_when_nothing_in_hand_is_lost() {
    // no lands, a Preordain alone in hand, the Oracle on top of a library of three: the Preordain is paid by the diamond
    // (nothing else in hand to lose) and draws the Oracle
    let mut g = pile_in_play(0, 0, &["Preordain"], &["Thassa's Oracle", "Chrome Mox", "Lotus Petal"]);
    // (U: three from the diamond for the Preordain; the Oracle's {U}{U} needs the other two of the three: both are paid from
    // the one crack, a running total)
    doomsday::run_pile(&mut g, P0).unwrap();
    assert!(won(&g), "{:?}", g.log);
    assert_eq!(log_count(&g, "cracks Lion's Eye Diamond"), 1, "one crack pays for both spells: {:?}", g.log);
}

fn log_count(g: &Game, text: &str) -> usize {
    g.log.as_ref().map_or(0, |l| l.iter().filter(|x| x.contains(text)).count())
}

// ------------------------------------------------------------------ 3. the cast of LED
const KINNAN: &str = "kinnan-bonder-mana-combo";
const DACK: &str = "dack-fayden-white-stax";
const SILAS: &str = "rograkh-silas-turbo-breach";

/// thrasios at P0 on its own turn with an empty hand; the other seats are goldfish with no cards in hand
fn solo(turn: u32) -> Game {
    let mut g = table(&["thrasios", KINNAN, DACK, SILAS]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = turn;
    g.round = turn;
    g.active = Some(P0);
    g.step = Step::Main1;
    g.log = Some(vec![]);
    g
}

#[test]
fn the_ai_casts_led_from_hand_with_doomsday_in_hand_and_holds_it_without() {
    let prio = |g: &Game| sim_core::impls::t6_plans::prio(g, P0, card(g, LED));
    for (cards, want) in [
        (vec![LED, "Doomsday"], 90.0),
        (vec![LED, "Doomsday", "Island"], 90.0),
        (vec![LED, "Demonic Tutor"], 90.0), // a tutor that can fetch it
        (vec![LED, "Mystic Remora"], 0.0),
        (vec![LED, "Thassa's Oracle", "Gitaxian Probe"], 0.0),
    ] {
        let mut g = solo(2);
        hand(&mut g, P0, &cards);
        assert_eq!(prio(&g), Some(want), "{cards:?}");
    }
    // the main phase offers the cast with Doomsday in hand, and not without
    let mut g = solo(2);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &[LED, "Doomsday", "Mystic Remora"]);
    let o = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let led = o.iter().find(|x| x.label == LED).expect("LED is offered with Doomsday in hand");
    assert!(led.utility > 0.0, "{}", led.utility);
    let mut g = solo(2);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &[LED, "Mystic Remora", "Rhystic Study"]);
    let o = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(o.iter().all(|x| x.label != LED), "nothing to fuel: it waits in hand");
}

#[test]
fn the_held_statistic_counts_doomsday_as_a_use_of_the_diamond() {
    use sim_core::sym::intern;
    // a hand with LED, Doomsday, the Oracle and an enabler, no black land: the diamond is held mana (T1 hand stat)
    let held = |cards: &[&str]| {
        let mut g = solo(1);
        lands(&mut g, P0, "Island", 3, false);
        lands(&mut g, P0, "Plains", 1, false);
        hand(&mut g, P0, cards);
        let lib: Vec<_> = ["Gitaxian Probe", "Preordain", "Ponder", "Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"]
            .iter()
            .map(|n| card(&g, n))
            .collect();
        g.player_mut(P0).library = lib;
        sim_core::t6stats::mana_held(&mut g, P0);
        *g.player(P0).stats.get(&intern("mn|heldr|T1|Lion's Eye Diamond")).unwrap_or(&0)
    };
    assert_eq!(held(&[LED, "Doomsday", "Thassa's Oracle", "Borne Upon a Wind"]), 1, "a Doomsday it pays for (at the end step, behind the Borne)");
    assert_eq!(held(&[LED, "Mystic Remora", "Gamble"]), 0, "no use");
}

// ------------------------------------------------------------------ 4. the control: the plain cast is better
#[test]
fn led_stays_when_the_plain_cast_is_as_good() {
    // the same Pile C position, with the black mana there: the hand keeps its Oracle and Probe, and the diamond stays
    let mut g = setup(
        0,
        0,
        &["Doomsday", "Thassa's Oracle", "Gitaxian Probe"],
        &["Preordain", "Ponder", "Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"],
        &[],
    );
    lands(&mut g, P0, "Underground Sea", 8, false);
    lands(&mut g, P0, "Scrubland", 2, false);
    let c = card(&g, "Doomsday");
    let (built, _) = doomsday::plan_cast(&g, P0, c, doomsday::Scn::Main).expect("a pile wins");
    assert!(!built.led, "nothing is gained by throwing the hand away");
    cast_doomsday(&mut g, Some(false));
    assert!(won(&g));
    assert!(on_bf(&g, LED), "the diamond was never cracked");
    assert!(!logged(&g, "cracks Lion's Eye Diamond"));
}
