//! Rograkh & Thrasios plan slots (impls/t6_plan_rograkh_thrasios.rs), from the SPEC.md decision scenarios 10, 11, 12.

use sim_core::ai::decks::deck_prio_f;
use sim_core::ids::PlayerId;
use sim_core::impls::t6_plans;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const RT: &str = "rograkh-thrasios-creature-combo";
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";

fn tbl() -> Game {
    let mut g = table(&[RT, KRAUM]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn set_hand(g: &mut Game, names: &[&str]) {
    g.player_mut(P0).hand.clear();
    hand(g, P0, names);
}

#[test]
fn scenario_10_one_land_with_two_fast_mana_pieces_is_a_keep() {
    let mut g = tbl();
    set_hand(&mut g, &["Forest", "Sol Ring", "Mox Amber", "Chatterstorm", "Gamble", "Snap", "Spellseeker"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
}

#[test]
fn scenario_11_two_lands_and_four_reactive_spells_is_a_ship() {
    let mut g = tbl();
    set_hand(
        &mut g,
        &["Forest", "Island", "Force of Will", "Pact of Negation", "Fierce Guardianship", "Mental Misstep", "Snap"],
    );
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
}

#[test]
fn two_lands_and_a_piece_of_acceleration_is_a_keep_and_no_land_is_a_ship() {
    let mut g = tbl();
    set_hand(&mut g, &["Forest", "Island", "Birds of Paradise", "Chatterstorm", "Snap", "Gamble", "Rhystic Study"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
    set_hand(
        &mut g,
        &["Sol Ring", "Mox Amber", "Birds of Paradise", "Chatterstorm", "Snap", "Gamble", "Rhystic Study"],
    );
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
}

#[test]
fn scenario_12_oboro_and_the_finishers_stay_in_hand_and_rograkh_comes_first() {
    let mut g = tbl();
    set_hand(
        &mut g,
        &["Oboro Breezecaller", "Finale of Devastation", "Eternal Witness", "Chatterstorm", "Rograkh, Son of Rohgahh"],
    );
    let by = |g: &Game, n: &str| deck_prio_f(g, P0, card(g, n));
    assert!(by(&g, "Oboro Breezecaller") <= 0.0);
    assert!(by(&g, "Finale of Devastation") <= 0.0);
    assert!(by(&g, "Eternal Witness") <= 0.0);
    assert!(by(&g, "Chatterstorm") > 50.0);
    assert!(by(&g, "Rograkh, Son of Rohgahh") > 90.0);
}

#[test]
fn tutors_fetch_cradle_first_and_the_gate_is_tier_6_only() {
    let g = tbl();
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(&*g.db.get(w[0]).name, "Gaea's Cradle");
    let mut h = table(&["seph", KRAUM]);
    h.active = Some(P0);
    assert_eq!(t6_plans::keep(&h, P0), None);
    assert_eq!(t6_plans::wish(&h, P0), None);
}

#[test]
fn line_a_goes_with_cover_in_hand() {
    let mut g = tbl();
    set_hand(&mut g, &["Veil of Summer"]);
    assert_eq!(t6_plans::go_now(&g, P0, "Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara"), Some(true));
    assert_eq!(t6_plans::go_now(&g, P0, "Kinnan + Basalt Monolith"), None);
}

#[test]
fn opening_hands_are_kept_like_a_cedh_deck_keeps() {
    // 7-card hands from shuffled libraries: the plan keeps about 3 in 5 (1.9 lands a hand: 40% are 0-1 landers, and a
    // 1-lander now needs 3 mana from its own hand: Mox Diamond, Chrome Mox and a lone Sol Ring do not count; t6-ai-keeps)
    let (mut kept, mut n) = (0, 0);
    for seed in 0..300u64 {
        let mut g = table_with(&[RT, KRAUM], seed, 40);
        g.player_mut(P0).hand.clear();
        for _ in 0..7 {
            let c = g.player_mut(P0).library.pop().unwrap();
            g.player_mut(P0).hand.push(c);
        }
        n += 1;
        if t6_plans::keep(&g, P0) == Some(true) {
            kept += 1;
        }
    }
    eprintln!("kept {kept} of {n}");
    assert!(kept * 100 >= n * 55, "kept only {kept} of {n}");
}

#[test]
fn ancient_tomb_with_six_fast_mana_and_reactive_cards_does_not_overflow() {
    // Ancient Tomb counts as a land and as acceleration; the "other cards" count used to go below zero (a panic in
    // Tier 6 game 59)
    let mut g = tbl();
    set_hand(
        &mut g,
        &["Ancient Tomb", "Sol Ring", "Mana Vault", "Mox Diamond", "Force of Will", "Force of Negation", "Swan Song"],
    );
    assert!(t6_plans::keep(&g, P0).is_some());
}

#[test]
fn sylvan_scrying_finds_gaeas_cradle_not_a_basic() {
    // "search for a land card" took a basic only (dsl::search), so a land tutor never found Cradle
    let mut g = tbl();
    g.active = Some(P0);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Sylvan Scrying"]);
    let lib: Vec<_> =
        ["Forest", "Island", "Gaea's Cradle", "Talon Gates of Madara"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    let c = card(&g, "Sylvan Scrying");
    assert!(sim_core::ai::brain::do_cast(&mut g, P0, c, None).unwrap());
    let cradle = card(&g, "Gaea's Cradle");
    assert!(g.player(P0).hand.contains(&cradle), "the wish list's first land");
}

#[test]
fn growing_rites_transforms_at_four_creatures_and_taps_like_cradle() {
    const RITES: &str = "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun";
    let mut g = tbl();
    perm(&mut g, P0, RITES);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).lands.is_empty(), "three creatures: no transform");
    token(&mut g, P0, 1);
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).lands.len(), 1, "four creatures: Itlimoc");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "GGGG", false), "{{G}} for each of the four creatures");
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "GGGGG", false));
}

// --- t6 rograkh-thrasios: tutors finish the closer line
/// Cradle out, Eternal Witness on the battlefield, no Oboro or Talon Gates held: line B is one card away (Snap), line A
/// two, so the tutors want Snap before Gates and Oboro
#[test]
fn tutors_want_snap_first_when_witness_is_out_and_line_a_is_two_cards_away() {
    let mut g = tbl();
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lands(&mut g, P0, "Forest", 3, false);
    perm(&mut g, P0, "Eternal Witness");
    let w = t6_plans::wish(&g, P0).unwrap();
    let name = |i: usize| g.db.get(w[i]).name.to_string();
    assert_eq!(name(0), "Snap", "{:?}", (0..4).map(name).collect::<Vec<_>>());
    // with Oboro in hand line A is one card away too: Gates keeps its place first
    hand(&mut g, P0, &["Oboro Breezecaller"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(g.db.get(w[0]).name.to_string(), "Talon Gates of Madara");
}

// --- t6 quiet progress: the lookahead's `combo_progress` reads the quiet (stepped) lines
/// Oboro and Talon Gates in hand with Thrasios in the command zone are three of line A's four groups (a quiet line): the
/// look-ahead scores (3/4)^2; Gaea's Cradle on the battlefield (a land) completes it
#[test]
fn a_quiet_line_mostly_held_scores_combo_progress() {
    use sim_core::cardcode::combo_progress;
    let mut g = tbl();
    assert!(combo_progress(&g, P0) < 0.07, "only the commanders: {}", combo_progress(&g, P0));
    hand(&mut g, P0, &["Oboro Breezecaller", "Talon Gates of Madara"]);
    assert!((combo_progress(&g, P0) - 0.5625).abs() < 1e-9, "{}", combo_progress(&g, P0));
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    assert!((combo_progress(&g, P0) - 1.0).abs() < 1e-9, "{}", combo_progress(&g, P0));
}

/// Dack Fayden's quiet line is the commander alone: always "held" from the command zone, so it is no progress (it
/// would add `combo_w` to Dack's strength in every evaluation and make Dack every seat's target)
#[test]
fn a_quiet_line_of_the_commander_alone_is_no_progress() {
    use sim_core::cardcode::combo_progress;
    let mut g = table(&["dack-fayden-white-stax", KRAUM]);
    g.player_mut(P0).hand.clear();
    let x = combo_progress(&g, P0);
    assert!(x.is_finite() && x < 0.99, "{x}");
}
