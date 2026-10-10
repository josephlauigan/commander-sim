//! The AI on hand-built positions: Python's tests/test_search.py (game copies, hidden information, position scores,
//! a whole decision), test_rules.Tutors and the AI's half of TeferisProtection, test_game_changers' Mox Diamond
//! priority, and Rust-only checks of the heuristic AI's choices.

use sim_core::ai::{brain, decks, plans, pool, search};
use sim_core::engine::{combat, tutors};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

type Fingerprint = Vec<(String, i32, Vec<u16>, Vec<u16>, Vec<String>, Vec<(u16, bool)>, u32)>;

fn fingerprint(g: &Game) -> Fingerprint {
    g.players
        .iter()
        .map(|p| {
            (
                p.key.to_string(),
                p.life,
                p.hand.iter().map(|c| c.0).collect(),
                p.library.iter().map(|c| c.0).collect(),
                p.perms.iter().map(|&m| g.perm(m).name.to_string()).collect(),
                p.lands.iter().map(|&l| (g.land(l).cd.0, g.land(l).tapped)).collect(),
                p.treasures,
            )
        })
        .collect()
}

// ------------------------------------------------------------------ test_search.Clone
#[test]
fn a_copy_is_independent() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    hand(&mut g, P1, &["Counterspell", "Lightning Bolt"]);
    let before = fingerprint(&g);
    let mut g2 = search::clone(&mut g);
    g2.player_mut(P0).perms.clear();
    g2.player_mut(P1).life = 1;
    g2.player_mut(P1).hand.clear();
    g2.player_mut(P0).library.pop();
    assert_eq!(fingerprint(&g), before);
}

// ------------------------------------------------------------------ test_search.Determinize
#[test]
fn opponents_hands_are_redealt_from_hand_and_library() {
    let mut g = table(&["seph", "veyran"]);
    hand(&mut g, P0, &["Entomb"]);
    hand(&mut g, P1, &["Counterspell", "Lightning Bolt"]);
    let mut g2 = search::clone(&mut g);
    let mut cards: Vec<u16> = g2.player(P1).hand.iter().chain(g2.player(P1).library.iter()).map(|c| c.0).collect();
    cards.sort();
    search::determinize(&mut g2, P0, &mut sim_core::rng::Rng::from_u64(3));
    let mut after: Vec<u16> = g2.player(P1).hand.iter().chain(g2.player(P1).library.iter()).map(|c| c.0).collect();
    after.sort();
    assert_eq!(after, cards); // the same cards
    assert_eq!(g2.player(P1).hand.len(), 2); // the same hand size
    assert_eq!(g2.player(P0).hand, vec![card(&g2, "Entomb")]); // my own hand is known
    assert_ne!(g2.player(P1).hand, g.player(P1).hand); // (seed 3 deals a different hand)
}

// ------------------------------------------------------------------ test_search.Evaluate
#[test]
fn a_win_beats_any_board() {
    let mut g = table(&["sauron", "veyran"]);
    for _ in 0..60 {
        token(&mut g, P0, 5);
    }
    assert!(search::evaluate(&g, P0) < 95.0);
    g.over = true;
    g.winner = Some(P0);
    assert_eq!(search::evaluate(&g, P0), 100.0);
    g.winner = Some(P1);
    assert_eq!(search::evaluate(&g, P0), -100.0);
}

#[test]
fn more_is_better() {
    let mut g = table(&["sauron", "veyran"]);
    let a = search::evaluate(&g, P0);
    perm(&mut g, P0, "Hellkite Tyrant");
    assert!(search::evaluate(&g, P0) > a);
}

// ------------------------------------------------------------------ test_search.Decision
fn decision_table() -> Game {
    let mut g = table_with(&["veyran", "sauron"], 7, 40);
    set_ai(&mut g, AiMode::Lookahead);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &["Guttersnipe", "Think Twice", "Lightning Bolt"]);
    g
}

#[test]
fn choose_picks_an_option_and_leaves_the_game_alone() {
    let mut g = decision_table();
    perm(&mut g, P1, "Jace's Archivist");
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let before = fingerprint(&g);
    let pick = search::choose(&mut g, P0, false, &opts).expect("a pick");
    assert!(pick < opts.len());
    assert_eq!(fingerprint(&g), before);
    assert_eq!(g.search_n, 1); // one decision taken
    assert!(g.search_work > 0); // its playouts' work is booked to the game
}

#[test]
fn decisions_are_reproducible() {
    let picks: Vec<String> = (0..2)
        .map(|_| {
            let mut g = decision_table();
            let opts = brain::main_options(&mut g, P0, false).unwrap();
            opts[search::choose(&mut g, P0, false, &opts).unwrap()].label.clone()
        })
        .collect();
    assert_eq!(picks[0], picks[1]);
}

#[test]
fn copies_never_search() {
    let mut g = decision_table();
    assert!(search::enabled(&mut g, P0));
    let mut g2 = search::clone(&mut g);
    g2.in_search = true;
    assert!(!search::enabled(&mut g2, P0));
    set_ai(&mut g, AiMode::Adaptive);
    assert!(!search::enabled(&mut g, P0));
}

// ------------------------------------------------------------------ test_rules.Tutors
#[test]
fn a_tutor_moves_one_card_from_library_to_hand() {
    let mut g = table(&["seph", "veyran"]);
    let lib = g.player(P0).library.clone();
    tutors::tutor(&mut g, P0, "any").unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
    assert_eq!(g.player(P0).library.len(), lib.len() - 1);
    assert!(lib.contains(&g.player(P0).hand[0]));
}

#[test]
fn sauron_tutors_for_the_missing_half_of_the_combo() {
    // Sword of Feast and Famine out, Aggravated Assault in the library: the tutor finds the Assault
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sword of Feast and Famine");
    let assault = card(&g, "Aggravated Assault");
    assert!(g.player(P0).library.contains(&assault));
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(assault));
}

// ------------------------------------------------------------------ test_rules.TeferisProtection (the AI's half)
#[test]
fn teferis_protection_answers_lethal_combat_damage_only() {
    let mut g = table(&["sauron", "seph"]);
    lands(&mut g, P1, "Plains", 3, false);
    hand(&mut g, P1, &["Teferi's Protection"]);
    let t = perm(&mut g, P0, "Grave Titan");
    combat::resolve_combat(&mut g, P0, &[t], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 34); // 6 damage from 40: not lethal, it waits
    assert!(g.player(P1).hand.contains(&card(&g, "Teferi's Protection")));
    let mut g = table(&["sauron", "seph"]);
    lands(&mut g, P1, "Plains", 3, false);
    hand(&mut g, P1, &["Teferi's Protection"]);
    g.player_mut(P1).life = 5;
    let t = perm(&mut g, P0, "Grave Titan");
    combat::resolve_combat(&mut g, P0, &[t], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 5);
    assert!(g.player(P1).alive);
}

#[test]
fn a_pool_deck_uses_teferis_protection_against_a_wipe() {
    let mut g = table(&["seph", "heliod-mono-white-stax"]);
    lands(&mut g, P1, "Plains", 3, false);
    hand(&mut g, P1, &["Teferi's Protection"]);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P1, "Archon of Cruelty");
    assert_eq!(pool::wipe_response(&mut g, P1, "destroy", P0).unwrap(), Some("all"));
    assert!(g.player(P1).life_locked);
    assert_eq!(g.player(P1).exile, vec![card(&g, "Teferi's Protection")]);
}

// ------------------------------------------------------------------ test_game_changers.Mana
#[test]
fn mox_diamond_waits_for_a_land_to_discard() {
    let mut g = table(&["kinnan-simic-mana-combo", "veyran"]);
    g.player_mut(P0).turns = 6;
    let mox = hand(&mut g, P0, &["Mox Diamond"])[0];
    assert_eq!(pool::generic_prio(&g, P0, mox), 0);
    hand(&mut g, P0, &["Forest", "Island"]);
    assert!(pool::generic_prio(&g, P0, mox) > 0);
}

// ------------------------------------------------------------------ the heuristic AI (Rust-only)
#[test]
fn the_main_phase_lists_castable_cards_and_stop() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Sol Ring", "Aggravated Assault"]); // Assault costs 3: not castable with two lands
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let labels: Vec<&str> = opts.iter().map(|o| o.label.as_str()).collect();
    assert!(labels.contains(&"Sol Ring"));
    assert!(!labels.contains(&"Aggravated Assault"));
    assert!(opts.last().unwrap().act.is_none()); // "stop" comes last
}

#[test]
fn the_main_phase_casts_what_it_can() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Sol Ring"]);
    brain::main(&mut g, P0, false).unwrap();
    assert!(g.player(P0).hand.is_empty());
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).name == "Sol Ring"));
}

#[test]
fn removal_goes_after_the_biggest_threat() {
    let mut g = table(&["seph", "sauron"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    let small = token(&mut g, P1, 1);
    let big = perm(&mut g, P1, "Hellkite Tyrant");
    let s = brain::Situation::new(&g, P0);
    let opts = brain::removal_options(&g, P0, &s);
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.ends_with("Hellkite Tyrant"));
    let _ = (small, big);
}

#[test]
fn farewell_takes_the_modes_that_hurt_opponents_more() {
    let mut g = table(&["seph", "sauron"]);
    perm(&mut g, P1, "Hellkite Tyrant"); // their creature
    perm(&mut g, P0, "Sol Ring"); // our artifact
    let modes = decks::wipe_modes(&g, P0, "farewell");
    assert!(modes.contains(&"cre"));
    assert!(!modes.contains(&"art"));
}

#[test]
fn a_rift_overload_counts_every_opponent() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    token(&mut g, P1, 3);
    token(&mut g, P2, 3);
    let (ol, ml, victim) = decks::wipe_eval(&g, P0, "rift");
    assert!(ol > 0.0 && ml == 0.0 && victim.is_none());
    let one = decks::wipe_eval(&g, P0, "rebuke");
    assert!(one.0 < ol && one.2.is_some());
}

#[test]
fn disruptor_flute_names_the_combo_half() {
    // Sauron's Sword and Assault both out: naming one of them stops the combo
    let mut g = table(&["seph", "sauron"]);
    perm(&mut g, P1, "Sword of Feast and Famine");
    perm(&mut g, P1, "Aggravated Assault");
    let (c, v) = decks::flute_pick(&g, P0).unwrap();
    assert!(v >= 11.0);
    assert!(["Sword of Feast and Famine", "Aggravated Assault"].contains(&&*g.db.get(c).name));
}

#[test]
fn outside_decks_keep_their_plans() {
    assert!(plans::config("gaa-azorius-stax-control").style.unwrap().caution > 0.8);
    assert!(plans::config("no-such-deck").style.is_none());
}

#[test]
fn every_deck_finishes_games_with_the_look_ahead() {
    let t1 = tier("t1");
    let g = play_with(&["sauron", t1[0], t1[1], t1[2]], 600_001, false, AiMode::Lookahead);
    assert!(g.wintype.is_some());
    assert!(g.search_n > 0);
}
