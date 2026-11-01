//! --- phase4-values: the derived card values (`ai/values.rs`) on named cards, checked against their Oracle text.

use sim_core::ai::values::{self, BOMB};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P: PlayerId = PlayerId(0);

fn game() -> Game {
    let mut g = table(&tier("t1")[0..4]);
    g.round = 3;
    g.player_mut(P).turns = 3;
    lands(&mut g, P, "Plains", 2, false);
    lands(&mut g, P, "Forest", 1, false);
    lands(&mut g, P, "Swamp", 1, false);
    g
}

/// Arcane Signet: "{T}: Add one mana of any color in your commander's color identity." A rock: one mana a turn,
/// worth more early than late, and a small threat.
#[test]
fn a_rock_makes_one_mana_a_turn_and_matters_early() {
    let mut g = game();
    let c = card(&g, "Arcane Signet");
    let pr = values::profile(&g, c);
    assert_eq!(pr.mana, 1.0);
    assert_eq!(pr.cards + pr.cards_rep + pr.damage + pr.tokens, 0.0);
    let early = values::worth(&g, P, c, false);
    g.player_mut(P).turns = 9;
    g.round = 9;
    let late = values::worth(&g, P, c, false);
    assert!(early > 2.0 * late, "early {early} late {late}");
    assert!(values::threat(&g, c) < BOMB);
    // Sol Ring ({T}: Add {C}{C}) makes two
    assert_eq!(values::profile(&g, card(&g, "Sol Ring")).mana, 2.0);
}

/// Phyrexian Arena: "At the beginning of your upkeep, you draw a card and you lose 1 life." A card a turn.
#[test]
fn a_draw_engine_draws_a_card_a_turn() {
    let g = game();
    let c = card(&g, "Phyrexian Arena");
    let pr = values::profile(&g, c);
    assert!(pr.cards_rep >= 1.0, "{pr:?}");
    assert!(pr.engine >= 1.0);
    // worth more than a one-shot draw of two (Night's Whisper: "You draw two cards and you lose 2 life.")
    let once = values::worth(&g, P, card(&g, "Night's Whisper"), false);
    assert!(values::worth(&g, P, c, false) > once, "{} vs {once}", values::worth(&g, P, c, false));
}

/// Wrath of God: "Destroy all creatures. They can't be regenerated." A sweeper: held (no proactive cast value), worth
/// more with the opponents' creatures out than with only yours.
#[test]
fn a_wipe_is_held_and_worth_the_board_it_takes() {
    let mut g = game();
    let c = card(&g, "Wrath of God");
    assert!(values::profile(&g, c).sweep);
    assert_eq!(values::worth(&g, P, c, false), 0.0, "a sorcery sweeper is cast by the wipe logic, not the cast order");
    let mine = perm(&mut g, P, "Llanowar Elves");
    let _ = mine;
    let alone = values::worth(&g, P, c, true);
    for q in [PlayerId(1), PlayerId(2), PlayerId(3)] {
        perm(&mut g, q, "Archon of Cruelty");
    }
    let theirs = values::worth(&g, P, c, true);
    assert!(theirs > alone + 1.0, "with their Archons {theirs}, without {alone}");
}

/// Archon of Cruelty: a 6/6 flier whose enters and attack trigger takes a permanent, a card and 3 life from an
/// opponent. A bomb by its abilities (the tag said 9); Llanowar Elves, a 1/1 mana dork, is not.
#[test]
fn a_bomb_reads_as_a_bomb() {
    let g = game();
    let archon = values::threat(&g, card(&g, "Archon of Cruelty"));
    assert!(archon >= 7.0, "{archon}");
    let atraxa = values::threat(&g, card(&g, "Atraxa, Grand Unifier"));
    assert!(atraxa >= 7.0, "{atraxa}");
    assert!(values::threat(&g, card(&g, "Llanowar Elves")) < BOMB);
    // a spell is no board threat
    assert_eq!(values::threat(&g, card(&g, "Wrath of God")), 0.0);
}

/// Exquisite Blood ("Whenever an opponent loses life, you gain that much life") with Sanguine Bond ("Whenever you
/// gain life, target opponent loses that much life") out: it completes a catalogued combo, its role dominates.
#[test]
fn a_combo_piece_is_worth_its_line() {
    let mut g = game();
    let c = card(&g, "Exquisite Blood");
    let alone = values::worth(&g, P, c, false);
    perm(&mut g, P, "Sanguine Bond");
    let with = values::worth(&g, P, c, false);
    assert!(with >= alone + 4.0, "with Sanguine Bond {with}, alone {alone}");
}

#[test]
fn the_override_table_loads_and_refuses_rows_without_a_reason() {
    let g = game();
    for name in values::overrides().keys() {
        assert!(g.db.id(name).is_some(), "values_overrides.toml: {name:?} isn't in data/cards.json");
    }
    assert!(values::parse_overrides("[[card]]\nname = \"Sol Ring\"\nworth = 9.0\nreason = \"\"\n").is_err());
    assert!(values::parse_overrides("[[card]]\nname = \"Sol Ring\"\nreason = \"why\"\n").is_err());
    assert!(
        values::parse_overrides("[[card]]\nname = \"Sol Ring\"\nworth = 9.0\nreason = \"why\"\nbomb = 1\n").is_err()
    );
    assert_eq!(
        values::parse_overrides("[[card]]\nname = \"Sol Ring\"\nthreat = 3.0\nreason = \"why\"\n").unwrap().len(),
        1
    );
}

/// JD's decks with their hand AI and Tier 6 never read derived values (their own priority code stays)
#[test]
fn jd_decks_and_tier_6_are_never_on_derived_values() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    // the decks that keep their hand AI (veyran, sauron, karn and the rest of the deleted ones are generic decks)
    let g = table(&["sheoldred", "yshtola", "cloud", "omnath"]);
    for i in 0..4 {
        assert!(!values::generic_deck(&g, PlayerId(i)));
    }
    let g = table(&tier("t6")[0..4]);
    for i in 0..4 {
        assert!(!values::generic_deck(&g, PlayerId(i)));
    }
    let g = table(&tier("t2")[0..4]);
    assert!((0..4).all(|i| values::generic_deck(&g, PlayerId(i))));
}
