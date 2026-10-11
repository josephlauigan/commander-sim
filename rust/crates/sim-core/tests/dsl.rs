//! The card ability language's interpreter on real cards: Python's tests/test_dsl.py (Interpreter) and one test per
//! kind of ability (triggers, statics, spells, activated abilities). The compiler stays in Python.

use sim_core::dsl;
use sim_core::engine::{mana, values, zones};
use sim_core::ids::PlayerId;
use sim_core::state::Ctx;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn niv_mizzet_pings_when_you_draw() {
    // compiled from Scryfall: "Whenever you draw a card, Niv-Mizzet deals 1 damage to any target."
    let mut g = table(&["veyran", "seph"]);
    assert!(g.db.by_name("Niv-Mizzet, the Firemind").unwrap().has_dsl());
    perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn niv_mizzets_tap_draws_and_pings() {
    // "{T}: Draw a card." as an option for the AI, then its draw trigger
    let mut g = table(&["veyran", "seph"]);
    let niv = perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    let opts = dsl::ability_options(&mut g, P0, true).unwrap();
    let (src, idx) = opts
        .iter()
        .find_map(|o| match o.act {
            Some(sim_core::hooks::Action::Dsl { src, idx }) => Some((src, idx)),
            _ => None,
        })
        .expect("its tap ability is offered");
    assert_eq!(src, niv);
    let hand = g.player(P0).hand.len();
    assert!(dsl::activate(&mut g, P0, src, idx).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert_eq!(g.player(P1).life, 39);
    assert!(g.perm(niv).tapped);
}

#[test]
fn dictate_of_erebos_makes_each_opponent_sacrifice() {
    // "Whenever a creature you control dies, each opponent sacrifices a creature."
    let mut g = table(&["seph", "veyran", "sauron"]);
    perm(&mut g, P0, "Dictate of Erebos");
    let mine = token(&mut g, P0, 2);
    let a = token(&mut g, P1, 1);
    let b = token(&mut g, PlayerId(2), 1);
    zones::die(&mut g, mine, "destroy").unwrap();
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
}

#[test]
fn lyra_pumps_other_angels_and_gives_angels_lifelink() {
    // "Other Angels you control get +1/+1 and have lifelink."
    let mut g = table(&["seph", "veyran"]);
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    let angel = perm(&mut g, P0, "Admonition Angel");
    let printed = g.perm(angel).pow;
    assert_eq!(values::epow(&g, angel), printed + 1);
    assert_eq!(values::epow(&g, lyra), g.perm(lyra).pow); // other Angels only
    assert!(dsl::has_kw(&g, angel, "lifelink") && dsl::has_kw(&g, lyra, "lifelink"));
    let theirs = perm(&mut g, P1, "Admonition Angel");
    assert_eq!(values::epow(&g, theirs), printed); // you control
}

#[test]
fn goblin_electromancer_makes_instants_and_sorceries_cheaper() {
    // "Instant and sorcery spells you cast cost {1} less to cast."
    let mut g = table(&["veyran", "seph"]);
    let div = card(&g, "Divination");
    assert_eq!(mana::cost_of(&g, P0, div), (2, "U".into()));
    perm(&mut g, P0, "Goblin Electromancer");
    assert_eq!(mana::cost_of(&g, P0, div), (1, "U".into()));
    assert_eq!(mana::cost_of(&g, P1, div), (2, "U".into())); // only yours
}

#[test]
fn dread_drone_makes_two_spawn_that_make_mana() {
    // "When Dread Drone enters, create two 0/1 colorless Eldrazi Spawn creature tokens with 'Sacrifice this creature:
    // Add {C}.'"
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Dread Drone");
    let spawn: Vec<_> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(spawn.len(), 2);
    assert!(spawn.iter().all(|&m| g.perm(m).ttypes.contains(&"spawn") && g.perm(m).tgh == 1));
    assert_eq!(mana::total_mana(&g, P0, false), 2);
}

#[test]
fn divination_draws_two() {
    let mut g = table(&["veyran", "seph"]);
    let div = card(&g, "Divination");
    let hand = g.player(P0).hand.len();
    dsl::resolve_spell(&mut g, P0, div, &Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 2);
}
