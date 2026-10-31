//! The `thrasios` seat's play: Blossoming Defense and Dark Endurance (protection, held mana, the combat trick), the
//! Walking Ballista and combo-piece priorities, the tutor picking the missing combo piece, and Thrasios's {4} sink.

use sim_core::ai::decks::deck_prio;
use sim_core::engine::removal::apply_removal;
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{tymna_cards, tymna_partners};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const TYMNA: &str = "Tymna the Weaver";

fn tbl() -> Game {
    let mut g = table(&["thrasios", "seph", "veyran"]);
    g.active = Some(P0);
    g
}

fn tymna(g: &mut Game) -> PermId {
    let t = perm(g, P0, TYMNA);
    g.perm_mut(t).is_cmd = true;
    t
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ---------------------------------------------------------------- Blossoming Defense
#[test]
fn blossoming_defense_saves_tymna_from_targeted_removal() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    let t = tymna(&mut g);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    apply_removal(&mut g, Some(P1), t, "exile", None).unwrap();
    assert!(g.perm(t).on_bf, "hexproof in response: the exile has no target");
    assert!(in_gy(&g, P0, "Blossoming Defense"));
    assert_eq!((epow(&g, t), etgh(&g, t)), (4, 4), "+2/+2 until end of turn");
}

#[test]
fn blossoming_defense_needs_its_green_mana_and_is_not_cast_against_an_edict() {
    let mut g = tbl();
    let t = tymna(&mut g);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    apply_removal(&mut g, Some(P1), t, "exile", None).unwrap();
    assert!(!g.perm(t).on_bf, "no green mana: nothing to cast");
    let mut g = tbl();
    let t = tymna(&mut g);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    assert!(!sim_core::ai::protect_response(&mut g, P0, t, "edict".into(), Some(P1), None).unwrap());
    assert!(!in_gy(&g, P0, "Blossoming Defense"), "an edict targets no creature");
}

#[test]
fn the_thrasios_seat_casts_blossoming_defense_to_save_a_commander() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = table(&["thrasios", "seph", "veyran"]);
    let t = perm(&mut g, P0, TYMNA);
    g.perm_mut(t).is_cmd = true;
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    apply_removal(&mut g, Some(P1), t, "exile", None).unwrap();
    assert!(g.perm(t).on_bf, "Thrasios's list has Blossoming Defense: the commander is saved");
}

#[test]
fn mana_is_held_for_the_protection_spells_while_a_commander_is_out() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    let ids = hand(&mut g, P0, &["Blossoming Defense", "Dark Endurance", "Swords to Plowshares"]);
    assert!(!tymna_partners::protection_held(&g, P0, ids[0]), "nothing to protect yet");
    tymna(&mut g);
    assert!(tymna_partners::protection_held(&g, P0, ids[0]));
    assert!(tymna_partners::protection_held(&g, P0, ids[1]));
    assert!(!tymna_partners::protection_held(&g, P0, ids[2]));
}

// ---------------------------------------------------------------- the combat trick
fn blocked_tymna(g: &mut Game, blocker_power: i32) -> (PermId, PermId) {
    let t = tymna(g);
    let b = token(g, P1, blocker_power);
    (t, b)
}

#[test]
fn a_blocked_tymna_that_would_die_gets_blossoming_defense() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    let (t, b) = blocked_tymna(&mut g, 3); // 3 damage kills a 2/2
    tymna_partners::combat_trick(&mut g, P0, &[t], &[(t, b)]).unwrap();
    assert!(in_gy(&g, P0, "Blossoming Defense"));
    assert_eq!((epow(&g, t), etgh(&g, t)), (4, 4), "survives the 3 damage");
}

#[test]
fn a_blocked_tymna_gets_dark_endurance_when_the_blocker_is_too_big_for_blossoming_defense() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Blossoming Defense", "Dark Endurance"]);
    let (t, b) = blocked_tymna(&mut g, 5); // 5 damage kills even a 4/4
    tymna_partners::combat_trick(&mut g, P0, &[t], &[(t, b)]).unwrap();
    assert!(in_gy(&g, P0, "Dark Endurance"));
    assert!(!in_gy(&g, P0, "Blossoming Defense"), "kept: it would not have saved her");
    assert!(sim_core::engine::values::indestructible(&g, t));
}

#[test]
fn no_trick_when_the_attacker_survives_or_is_not_worth_it() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    let (t, b) = blocked_tymna(&mut g, 1); // she takes 1 and kills the 1/2... nothing to save
    tymna_partners::combat_trick(&mut g, P0, &[t], &[(t, b)]).unwrap();
    assert!(!in_gy(&g, P0, "Blossoming Defense"));
}

// ---------------------------------------------------------------- the combo cards
fn prio(g: &Game, name: &str) -> i32 {
    deck_prio(g, P0, card(g, name))
}

#[test]
fn walking_ballista_is_cast_for_value_with_spare_mana_but_held_for_the_combo() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Walking Ballista"]);
    assert_eq!(prio(&g, "Walking Ballista"), 0, "X would be 1 with three mana");
    lands(&mut g, P0, "Forest", 2, false);
    assert!(prio(&g, "Walking Ballista") >= 45, "five mana: X = 2, even with Rings still in the library");
    // Basalt and Rings both in reach: hold the kill
    hand(&mut g, P0, &["Basalt Monolith", "Rings of Brighthearth"]);
    assert_eq!(prio(&g, "Walking Ballista"), 0);
    // ... unless the mana is huge
    lands(&mut g, P0, "Forest", 5, false);
    assert!(prio(&g, "Walking Ballista") > 0);
}

#[test]
fn basalt_is_always_ramp_and_rings_waits_for_a_basalt() {
    let mut g = tbl();
    g.player_mut(P0).turns = 9;
    hand(&mut g, P0, &["Basalt Monolith", "Rings of Brighthearth"]);
    assert!(prio(&g, "Basalt Monolith") >= 70, "cast on any turn");
    let with_basalt = prio(&g, "Rings of Brighthearth");
    let mut g2 = tbl();
    hand(&mut g2, P0, &["Rings of Brighthearth"]);
    assert!(with_basalt >= 70 && with_basalt > prio(&g2, "Rings of Brighthearth"));
}

fn lib_ids(g: &Game) -> Vec<CardId> {
    g.player(P0).library.clone()
}

#[test]
fn a_tutor_fetches_the_missing_third_piece_of_the_combo() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    perm(&mut g, P0, "Basalt Monolith");
    hand(&mut g, P0, &["Rings of Brighthearth"]);
    let ok = lib_ids(&g);
    let c = tymna_cards::tutor_pick(&g, P0, "any", &ok).expect("two pieces: fetch the outlet");
    assert_eq!(&*g.db.get(c).name, "Walking Ballista");
    // Basalt + the Ballista in hand: Rings
    let mut g = tbl();
    perm(&mut g, P0, "Basalt Monolith");
    hand(&mut g, P0, &["Walking Ballista"]);
    let ok = lib_ids(&g);
    let c = tymna_cards::tutor_pick(&g, P0, "any", &ok).unwrap();
    assert_eq!(&*g.db.get(c).name, "Rings of Brighthearth");
    // Rings + Thrasios out: Basalt
    let mut g = tbl();
    perm(&mut g, P0, "Rings of Brighthearth");
    perm(&mut g, P0, "Thrasios, Triton Hero");
    let ok = lib_ids(&g);
    let c = tymna_cards::tutor_pick(&g, P0, "any", &ok).unwrap();
    assert_eq!(&*g.db.get(c).name, "Basalt Monolith");
}

// ---------------------------------------------------------------- Thrasios's sink
#[test]
fn thrasios_ability_is_found_among_the_options_and_left_out_of_the_search() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let g = tbl();
    let opts = vec![
        Opt { utility: 1.0, label: "Swords to Plowshares".into(), act: None },
        Opt { utility: -2.0, label: "Thrasios: scry 1, a land or a card".into(), act: Some(Action::Clue) },
        Opt { utility: -3.0, label: "stop".into(), act: None },
    ];
    assert_eq!(tymna_partners::thrasios_sink(&g, P0, &opts), Some(1));
    assert_eq!(tymna_partners::thrasios_sink(&g, P1, &opts), None, "only the thrasios and kraum seats");
}

// ---------------------------------------------------------------- Dark Endurance against a sweeper
#[test]
fn dark_endurance_keeps_the_commander_out_of_a_destroy_sweeper() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = tbl();
    let t = tymna(&mut g);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Dark Endurance"]);
    tymna_cards::darien_wipe(&mut g, P0, "destroy".into(), P1).unwrap();
    assert!(in_gy(&g, P0, "Dark Endurance"));
    assert!(sim_core::engine::values::indestructible(&g, t));
    // a Vandalblast-style sweeper (artifacts only) is no reason
    let mut g = tbl();
    tymna(&mut g);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Dark Endurance"]);
    tymna_cards::darien_wipe(&mut g, P0, "vandal".into(), P1).unwrap();
    assert!(!in_gy(&g, P0, "Dark Endurance"));
}
