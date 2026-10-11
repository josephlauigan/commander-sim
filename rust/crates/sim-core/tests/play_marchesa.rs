//! Practice mode, cards first written for the Marchesa deck (removed 2026-10-07) that other lists still run, and the
//! sacrifice outlets your decks share: Accursed Marauder's sacrifice, Mystic Remora's upkeep, dredge, Coalition
//! Relic, and free sacrifice outlets as abilities (Python's tests/test_play_marchesa.py).

mod play_common;
use play_common::*;
use sim_core::human::cards;
use sim_core::state::DataKey;

// ------------------------------------------------------------------ Spells
#[test]
fn accursed_marauder_you_choose_your_sacrifice() {
    let mut g = table(&["jodah", "veyran"]);
    hand(&mut g, P0, &["Accursed Marauder"]);
    let keep = perm(&mut g, P0, "Hellkite Tyrant");
    perm(&mut g, P1, "Guttersnipe");
    pool(&mut g, P0, &[('B', 1), ('C', 1)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Accursed")]);
    assert!(g.player(P0).perms.contains(&keep));
    assert!(!has_perm(&g, P1, "Guttersnipe"));
}

// ------------------------------------------------------------------ Triggers
#[test]
fn mystic_remora_upkeep_declined() {
    let mut g = table(&["yshtola", "veyran"]);
    let rm = perm(&mut g, P0, "Mystic Remora");
    pool(&mut g, P0, &[('C', 1)]);
    seat(&mut g, P0, vec![pick(1)]); // "pay {1}?" no
    sim_core::impls::rules::remora_age(&mut g, rm, P0).unwrap();
    assert!(!g.player(P0).perms.contains(&rm));
}

#[test]
fn dredge_or_draw() {
    let mut g = table(&["seph", "veyran"]);
    let imp = card(&g, "Stinkweed Imp");
    g.player_mut(P0).gy.push(imp);
    seat(&mut g, P0, vec![pick(0)]); // yes
    assert!(cards::dredge(&mut g, P0).unwrap());
    assert!(g.player(P0).hand.contains(&imp));
    assert_eq!(g.player(P0).gy.len(), 5);
}

// ------------------------------------------------------------------ Outlets
#[test]
fn carrion_feeder() {
    let mut g = table(&["seph", "veyran"]);
    let f = perm(&mut g, P0, "Carrion Feeder");
    let r = perm(&mut g, P0, "Burglar Rat");
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": 0})), pick(0), by_text("Burglar")]);
    assert!(!g.player(P0).perms.contains(&r));
    assert_eq!(g.perm(f).plus, 1);
}

#[test]
fn ashnods_altar_makes_colourless() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Ashnod's Altar");
    perm(&mut g, P0, "Grave Titan");
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": 0})), pick(0), by_text("Grave Titan")]);
    assert_eq!(pool_total(&g, P0), 2);
    let titan = card(&g, "Grave Titan");
    assert!(g.player(P0).gy.contains(&titan));
}

#[test]
fn coalition_relic_charge() {
    let mut g = table(&["jodah", "veyran"]);
    let cr = perm(&mut g, P0, "Coalition Relic");
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": 0})), pick(0)]);
    assert_eq!((g.perm(cr).data.int(DataKey::Charge), g.perm(cr).tapped), (1, true));
}
