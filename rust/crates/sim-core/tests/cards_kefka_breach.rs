//! The Underworld Breach line outside Sauron: Kefka holding Breach, Brain Freeze and rituals mills the table out; with no
//! storm card Kefka uses Breach as before; its discards keep the pieces the other half is waiting for.

use sim_core::ai::{act, brain, decks};
use sim_core::ids::PlayerId;
use sim_core::impls::{kefka, mine};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Kefka against one Veyran whose library holds `opp_library` cards (the rest milled, no hand); `fuel` Islands in
/// Kefka's graveyard; Kefka's hand empty
fn board(islands: usize, swamps: usize, opp_library: usize, fuel: usize) -> Game {
    let mut g = table(&["kefka", "veyran"]);
    let pl = g.player_mut(P1);
    pl.hand.clear();
    let rest = pl.library.split_off(opp_library.min(pl.library.len()));
    pl.gy.extend(rest);
    let island = card(&g, "Island");
    let k = g.player_mut(P0);
    k.hand.clear();
    k.gy = vec![island; fuel];
    lands(&mut g, P0, "Island", islands, false);
    lands(&mut g, P0, "Swamp", swamps, false);
    g
}

fn prio(g: &Game, name: &str) -> i32 {
    decks::deck_prio(g, P0, card(g, name))
}

fn has_line(g: &mut Game) -> bool {
    brain::main_options(g, P0, false).unwrap().iter().any(|o| o.label == "Underworld Breach line")
}

#[test]
fn kefka_runs_the_line_and_mills_the_table_out() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = board(5, 4, 10, 8);
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Dark Ritual", "Lotus Petal"]);

    assert!(mine::breach_line(&mut g, P0, false).unwrap(), "the dry run finishes the table");
    assert!(has_line(&mut g), "offered as a main-phase option");
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == "Underworld Breach line").unwrap();
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert!(g.player(P1).library.is_empty(), "{}", g.player(P1).library.len());
    assert!(g.player(P0).stats.get("breach_line").copied().unwrap_or(0) >= 1);
}

#[test]
fn kefka_without_enough_mana_does_not_offer_the_line() {
    let mut g = board(1, 1, 30, 8);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap());
    assert!(!has_line(&mut g));
}

#[test]
fn kefka_holds_the_line_cards_while_breach_is_reachable() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = board(5, 4, 30, 8);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Dark Ritual", "Lotus Petal"]);
    for n in ["Underworld Breach", "Brain Freeze", "Dark Ritual", "Lotus Petal"] {
        assert_eq!(prio(&g, n), 0, "{n} is held for the line");
    }
    // Brain Freeze with no Breach anywhere is not held by the line (it keeps its usual priority)
    let mut g = board(5, 4, 30, 8);
    let b = card(&g, "Underworld Breach");
    g.player_mut(P0).library.retain(|&c| c != b);
    hand(&mut g, P0, &["Brain Freeze", "Dark Ritual"]);
    assert!(decks::breach_hold_prio(&g, P0, card(&g, "Brain Freeze")).is_none());
    assert!(decks::breach_hold_prio(&g, P0, card(&g, "Dark Ritual")).is_none());
}

#[test]
fn kefka_without_a_storm_card_uses_breach_as_before() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = board(4, 4, 30, 0);
    hand(&mut g, P0, &["Underworld Breach", "Dark Ritual"]);
    perm(&mut g, P1, "Eternal Witness");
    for n in ["Lightning Bolt", "Duress", "Dark Inquiry", "Deadly Embrace", "Contract Killing", "Forest"] {
        let c = card(&g, n);
        g.player_mut(P0).gy.push(c);
    }
    assert!(!has_line(&mut g));
    assert!(prio(&g, "Underworld Breach") > 0, "a graveyard of spells and the mana to escape them");
    assert!(prio(&g, "Dark Ritual") > 0 || decks::breach_hold_prio(&g, P0, card(&g, "Dark Ritual")).is_none());
}

#[test]
fn kefka_does_not_discard_the_pieces_the_line_waits_for() {
    let mut g = board(5, 4, 30, 0);
    let h = hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Forest", "Forest"]);
    let pick = kefka::discard_choice(&g, P0, &h, 0).unwrap();
    assert!(&*g.db.get(pick).name == "Forest", "discarded {}", g.db.get(pick).name);
    // Brain Freeze alone (no Breach out) is fine to bin: it is escaped from the graveyard later
    let mut g = board(5, 4, 30, 0);
    let h = hand(&mut g, P0, &["Brain Freeze", "Island", "Swamp"]);
    let _ = kefka::discard_choice(&g, P0, &h, 0).unwrap();
}

#[test]
fn diabolic_tutor_fetches_the_missing_half_of_the_line() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = board(5, 4, 30, 0);
    let bf = card(&g, "Brain Freeze");
    let ub = card(&g, "Underworld Breach");
    g.player_mut(P0).library.extend([bf, ub]);
    let mut dn = g.player(P0).deck_names.to_vec();
    dn.push(bf);
    g.player_mut(P0).deck_names = dn.into();
    let pick = decks::tutor_pick(&g, P0, "any").unwrap();
    assert_eq!(&*g.db.get(pick).name, "Underworld Breach", "Breach first");
    hand(&mut g, P0, &["Underworld Breach"]);
    g.player_mut(P0).library.retain(|&c| c != ub);
    let pick = decks::tutor_pick(&g, P0, "any").unwrap();
    assert_eq!(&*g.db.get(pick).name, "Brain Freeze");
}

#[test]
fn storm_cards_wait_while_breach_is_in_the_library() {
    let mut g = board(5, 4, 30, 8);
    hand(&mut g, P0, &["Brain Freeze"]);
    let b = card(&g, "Underworld Breach");
    g.player_mut(P0).library.push(b);
    assert_eq!(prio(&g, "Brain Freeze"), 0, "Breach can still be drawn or tutored");
    // Breach gone from the library, hand and battlefield: Brain Freeze is just a spell again
    g.player_mut(P0).library.retain(|&c| c != b);
    assert!(decks::breach_hold_prio(&g, P0, card(&g, "Brain Freeze")).is_none());
}

#[test]
fn storm_card_is_not_held_when_it_kills_now() {
    // the opponent has 3 cards left: one plain Brain Freeze copy mills them out
    let mut g = board(5, 4, 3, 8);
    hand(&mut g, P0, &["Brain Freeze"]);
    let b = card(&g, "Underworld Breach");
    g.player_mut(P0).library.push(b);
    assert!(decks::breach_hold_prio(&g, P0, card(&g, "Brain Freeze")).is_none());
}

#[test]
fn sauron_already_holds_storm_cards_without_breach_out() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Brain Freeze", "Grapeshot"]);
    for n in ["Brain Freeze", "Grapeshot"] {
        assert_eq!(decks::sauron_prio(&g, P0, card(&g, n)), 0, "{n} is never cast plainly by Sauron");
    }
}
