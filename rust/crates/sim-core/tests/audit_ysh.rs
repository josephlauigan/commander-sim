//! AI audit phase 2 (suffix "ysh"): the kraum seat's fixes. Dark Endurance, Underworld Breach, Baird, the Diabolic Tutor
//! pick, Firebird and Lightning.

use sim_core::engine::{removal, values, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn kraum_tbl() -> Game {
    table(&["thrasios", "galadriel", "cloud", "seph"])
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

// ---------------------------------------------------------------- Dark Endurance
#[test]
fn dark_endurance_is_not_cast_against_exile_or_for_a_small_creature() {
    let mut g = kraum_tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Dark Endurance"]);
    let swords = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), t, "exile", Some(swords)).unwrap();
    assert!(!on(&g, t), "exile isn't stopped by indestructible");
    assert_eq!(g.player(P0).hand.len(), 1, "the card was kept");
    let tok = token(&mut g, P0, 1);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), tok, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, tok));
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn dark_endurance_is_only_for_the_tymna_partner_seats() {
    let mut g = table(&["galadriel", "cloud", "seph", "sauron"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).is_cmd = true;
    hand(&mut g, P0, &["Dark Endurance"]);
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), t, "destroy", Some(doom)).unwrap();
    assert!(!on(&g, t));
}

// ---------------------------------------------------------------- Underworld Breach
// (dropped 2026-10-19 with the Kraum deck: breach_candidates' wipe rule is keyed to the `kraum` seat, which no longer exists)

// ---------------------------------------------------------------- Baird
// ---------------------------------------------------------------- Diabolic Tutor
// ---------------------------------------------------------------- Firebird, Lightning
#[test]
fn firebird_pumps_the_other_attackers() {
    let mut g = kraum_tbl();
    g.active = Some(P0);
    let f = perm(&mut g, P0, "Firebird, Blazing Ranger");
    let a = perm(&mut g, P0, "Aven Mindcensor");
    let before = values::epow(&g, a);
    let imp = g.registry.get(g.perm(f).cd.unwrap()).unwrap().attack.unwrap();
    imp(&mut g, f, P0, &[f, a], P1).unwrap();
    assert_eq!(values::epow(&g, a), before + values::epow(&g, f));
    assert_eq!(values::epow(&g, f), 1, "Firebird itself isn't pumped");
}

#[test]
fn lightning_exiles_a_card_playable_while_it_stays_and_loses_them_when_it_leaves() {
    let mut g = kraum_tbl();
    g.active = Some(P0);
    let l = perm(&mut g, P0, "Lightning, Security Sergeant");
    let c = card(&g, "Sol Ring");
    g.player_mut(P0).library.push(c);
    let imp = g.registry.get(g.perm(l).cd.unwrap()).unwrap().combat_damage.unwrap();
    imp(&mut g, l, P0, l, P1, 2).unwrap();
    assert!(g.player(P0).hand.contains(&c), "waits in the hand");
    assert!(!g.player(P0).library.contains(&c));
    // survives end-of-turn cleanup of impulse cards
    sim_core::engine::turn::end_step(&mut g, P0).ok();
    assert!(g.player(P0).hand.contains(&c), "still playable on later turns");
    zones::die(&mut g, l, "destroy").unwrap();
    assert!(!g.player(P0).hand.contains(&c));
    assert!(g.player(P0).exile.contains(&c), "exiled for good");
}
