//! The Theorist, Jace Beleren against its Oracle text (data/scryfall_cache.json): "At the beginning of each opponent's
//! draw step, you draw a card. +1: Create a 1/1 blue Illusion creature token. -2: For each opponent, return up to one
//! target artifact or creature that player controls to its owner's hand. -6: Draw three cards. Then put X +1/+1
//! counters on each creature you control, where X is the number of cards in your hand." Starting loyalty 3.

mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::cards::Colors;
use sim_core::engine::{turn, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::{abilities_of, human_main};

const THEORIST: &str = "The Theorist, Jace Beleren";
const SHEO: &str = "Sheoldred, the Apocalypse";

/// P0 (the Theorist's seat), P1 and P2
fn tbl() -> Game {
    table(&["yshtola", "karn", "cloud"])
}

fn options(g: &mut Game, w: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(w).cd.unwrap()).and_then(|i| i.options).expect("the Theorist has options");
    f(g, w, P0, Some(false)).unwrap()
}

fn labels(g: &mut Game, w: PermId) -> Vec<String> {
    options(g, w).into_iter().map(|o| o.label).collect()
}

fn use_ability(g: &mut Game, w: PermId, word: &str) -> bool {
    let act = options(g, w).into_iter().find(|o| o.label.ends_with(word)).and_then(|o| o.act).expect("the option");
    perform(g, P0, &act).unwrap()
}

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    has_perm(g, p, name)
}

// ------------------------------------------------------------------ the draw trigger
#[test]
fn it_enters_with_three_loyalty() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    assert_eq!(g.perm(w).loyalty, Some(3));
}

#[test]
fn its_controller_draws_in_each_opponents_draw_step_but_not_their_own() {
    let mut g = tbl();
    perm(&mut g, P0, THEORIST);
    for opp in [P1, P2] {
        g.active = Some(opp);
        let (mine, theirs) = (hand_len(&g, P0), hand_len(&g, opp));
        turn::step_start(&mut g, opp).unwrap();
        assert_eq!(hand_len(&g, P0), mine + 1, "an opponent's draw step: the Theorist draws a card");
        assert_eq!(hand_len(&g, opp), theirs + 1, "the opponent's own draw is unchanged");
    }
    // the Theorist's own controller's draw step: just the normal draw
    let mut g = tbl();
    perm(&mut g, P0, THEORIST);
    turn::step_start(&mut g, P0).unwrap();
    // the turn's land drop follows the draw step: a land drawn from the top of the library is already on the battlefield
    assert_eq!(hand_len(&g, P0) + g.player(P0).lands.len(), 1, "your own draw step: no extra card");
}

#[test]
fn two_theorists_both_trigger() {
    // legendary: one per controller, so P0 and P1 each have one; P2's draw step
    let mut g = tbl();
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, THEORIST);
    g.active = Some(P2);
    let (a, b, c) = (hand_len(&g, P0), hand_len(&g, P1), hand_len(&g, P2));
    turn::step_start(&mut g, P2).unwrap();
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1), hand_len(&g, P2)), (a + 1, b + 1, c + 1));
    // P1's draw step: only P0 (P1's own Theorist is not an opponent's)
    g.active = Some(P1);
    let (a, b) = (hand_len(&g, P0), hand_len(&g, P1));
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1)), (a + 1, b + 1));
}

#[test]
fn a_phased_out_or_removed_theorist_does_not_trigger() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).phased = true;
    g.active = Some(P1);
    let mine = hand_len(&g, P0);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(hand_len(&g, P0), mine, "phased out");
    g.perm_mut(w).phased = false;
    zones::die(&mut g, w, "destroy").unwrap();
    g.active = Some(P2);
    turn::step_start(&mut g, P2).unwrap();
    assert_eq!(hand_len(&g, P0), mine, "destroyed");
}

#[test]
fn the_draw_is_a_normal_draw_sheoldred_punishes_it() {
    let mut g = table(&["yshtola", "sheoldred", "cloud"]);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, SHEO);
    g.active = Some(P2);
    let life = g.player(P0).life;
    turn::step_start(&mut g, P2).unwrap();
    assert_eq!(g.player(P0).life, life - 2, "the Theorist's draw: Sheoldred's opponent loses 2");
}

#[test]
fn no_trigger_when_the_active_player_skips_the_step_and_an_empty_library_loses_without_a_crash() {
    let mut g = tbl();
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Necropotence"); // skip your draw step: no beginning-of-draw-step trigger either
    g.active = Some(P1);
    let (mine, theirs) = (hand_len(&g, P0), hand_len(&g, P1));
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!((hand_len(&g, P0), hand_len(&g, P1)), (mine, theirs), "the step is skipped, so nothing triggers");
    // an empty library: the controller has drawn from an empty library and loses at the next state check
    let mut g = tbl();
    perm(&mut g, P0, THEORIST);
    g.player_mut(P0).library.clear();
    g.active = Some(P1);
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.player(P0).decked && !g.player(P0).alive, "drew from an empty library");
}

// ------------------------------------------------------------------ +1
#[test]
fn plus_one_makes_a_one_one_blue_illusion() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    assert!(use_ability(&mut g, w, "+1"));
    assert_eq!(g.perm(w).loyalty, Some(4));
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 1);
    let t = toks[0];
    assert!(g.is_creature(t));
    assert_eq!((sim_core::engine::values::epow(&g, t), sim_core::engine::values::etgh(&g, t)), (1, 1));
    assert!(sim_core::engine::values::has_type(&g, t, "illusion"));
    assert_eq!(sim_core::engine::values::colors_of(&g, t), Colors::from_letters("U"));
}

#[test]
fn a_walker_acts_once_a_turn() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    assert!(use_ability(&mut g, w, "+1"));
    assert!(labels(&mut g, w).is_empty(), "one loyalty ability a turn");
}

// ------------------------------------------------------------------ -2
#[test]
fn minus_two_returns_one_artifact_or_creature_of_each_opponent_and_never_your_own() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(5);
    let mine = perm(&mut g, P0, "Serra Angel");
    let (angel, ring) = (perm(&mut g, P1, "Serra Angel"), perm(&mut g, P1, "Sol Ring"));
    let (titan, bird) = (perm(&mut g, P2, "Grave Titan"), perm(&mut g, P2, "Birds of Paradise"));
    assert!(labels(&mut g, w).iter().any(|l| l.ends_with("-2")), "{:?}", labels(&mut g, w));
    assert!(use_ability(&mut g, w, "-2"));
    assert_eq!(g.perm(w).loyalty, Some(3), "-2 paid");
    assert!(!g.perm(angel).on_bf && g.perm(ring).on_bf, "P1: one permanent, the best");
    assert!(!g.perm(titan).on_bf && g.perm(bird).on_bf, "P2: one permanent, the best");
    assert!(g.perm(mine).on_bf, "never your own");
    assert!(g.player(P1).hand.contains(&g.perm(angel).cd.unwrap()), "back to its owner's hand");
    assert!(g.player(P2).hand.contains(&g.perm(titan).cd.unwrap()));
}

#[test]
fn minus_two_with_an_opponent_that_controls_nothing_is_a_no_op_for_that_opponent() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(4);
    let angel = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P2, "Island", 2, false); // lands are neither artifacts nor creatures
    assert!(use_ability(&mut g, w, "-2"));
    assert!(!g.perm(angel).on_bf);
    assert_eq!(g.player(P2).lands.len(), 2);
    assert_eq!(g.perm(w).loyalty, Some(2));
}

#[test]
fn minus_two_is_not_offered_when_nothing_is_worth_bouncing_or_loyalty_is_short() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(4);
    perm(&mut g, P1, "Grave Titan");
    g.perm_mut(w).loyalty = Some(1);
    assert!(!labels(&mut g, w).iter().any(|l| l.ends_with("-2")), "1 loyalty can't pay 2");
    g.perm_mut(w).loyalty = Some(2);
    assert!(labels(&mut g, w).iter().any(|l| l.ends_with("-2")), "2 loyalty can: the walker goes to 0");
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Llanowar Elves");
    assert!(!labels(&mut g, w).iter().any(|l| l.ends_with("-2")), "a mana dork is not worth a loyalty");
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    assert!(!labels(&mut g, w).iter().any(|l| l.ends_with("-2")), "no opposing artifact or creature");
    assert!(labels(&mut g, w).iter().any(|l| l.ends_with("+1")));
}

#[test]
fn minus_two_kills_a_token_and_skips_a_hexproof_creature() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(5);
    let t = token(&mut g, P1, 8); // P1's only creature
    let hex = perm(&mut g, P2, "Sigarda, Host of Herons"); // hexproof: can't be chosen
    let other = perm(&mut g, P2, "Grave Titan");
    assert!(sim_core::engine::values::untargetable(&g, hex));
    assert!(use_ability(&mut g, w, "-2"));
    assert!(!g.perm(t).on_bf, "a token bounced ceases to exist");
    assert!(g.perm(hex).on_bf && !g.perm(other).on_bf, "the hexproof one can't be chosen");
}

// ------------------------------------------------------------------ -6
#[test]
fn minus_six_draws_three_then_puts_cards_in_hand_counters_on_each_creature() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(6);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Brainstorm", "Counterspell"]);
    let (a, b) = (perm(&mut g, P0, "Serra Angel"), token(&mut g, P0, 1));
    let theirs = perm(&mut g, P1, "Grave Titan");
    let before = sim_core::engine::values::epow(&g, a);
    assert!(use_ability(&mut g, w, "-6"));
    assert_eq!(hand_len(&g, P0), 5, "two cards and three drawn");
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (5, 5), "X is the hand after drawing: 5");
    assert_eq!(sim_core::engine::values::epow(&g, a), before + 5);
    assert_eq!(g.perm(theirs).plus, 0, "only your creatures");
    assert!(!g.perm(w).on_bf, "6 loyalty paid: the walker is gone");
}

#[test]
fn minus_six_needs_six_loyalty_and_a_board_worth_pumping() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    perm(&mut g, P0, "Serra Angel");
    token(&mut g, P0, 1);
    g.perm_mut(w).loyalty = Some(5);
    assert!(!labels(&mut g, w).iter().any(|l| l.ends_with("-6")), "5 loyalty can't pay 6");
    g.perm_mut(w).loyalty = Some(6);
    assert!(labels(&mut g, w).iter().any(|l| l.ends_with("-6")));
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(6);
    assert!(!labels(&mut g, w).iter().any(|l| l.ends_with("-6")), "no creatures: not worth it");
}

// ------------------------------------------------------------------ practice mode
fn use_index(g: &mut Game, w: PermId, sign_cost: &str) -> (usize, usize) {
    let i = g.player(P0).perms.iter().position(|&x| x == w).unwrap();
    let ab = abilities_of(g, P0, w).unwrap();
    let k = ab
        .iter()
        .position(|a| a.0.starts_with(sign_cost))
        .unwrap_or_else(|| panic!("{:?}", ab.iter().map(|a| &a.0).collect::<Vec<_>>()));
    (i, k)
}

#[test]
fn a_person_uses_all_three_abilities() {
    // +1
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    let (i, k) = use_index(&mut g, w, "+1");
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(4));
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 1, "the Illusion");
    // -2: one question per opponent that controls an artifact or creature; the person may decline one
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(5);
    let (angel, ring) = (perm(&mut g, P1, "Serra Angel"), perm(&mut g, P1, "Sol Ring"));
    let titan = perm(&mut g, P2, "Grave Titan");
    let (i, k) = use_index(&mut g, w, "-2");
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), by_text("Sol Ring"), cancel(), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(3));
    assert!(g.perm(angel).on_bf && !g.perm(ring).on_bf, "the person's pick, not the AI's");
    assert!(g.perm(titan).on_bf, "up to one: declined");
    // -6
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(6);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Brainstorm"]);
    let a = perm(&mut g, P0, "Serra Angel");
    let (i, k) = use_index(&mut g, w, "-6");
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!((hand_len(&g, P0), g.perm(a).plus), (4, 4));
    assert!(!on_bf(&g, P0, THEORIST));
}

#[test]
fn a_person_cannot_pay_more_loyalty_than_the_walker_has() {
    let mut g = tbl();
    let w = perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Serra Angel");
    let (i, k) = use_index(&mut g, w, "-6");
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(3), "nothing paid");
    assert!(h.told().iter().any(|t| t.1.contains("loyalty")), "{:?}", h.told());
}
