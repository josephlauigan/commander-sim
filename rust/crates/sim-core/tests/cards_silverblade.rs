//! Silverblade Paladin: soulbond pairing and double strike for both.

use sim_core::engine::combat;
use sim_core::engine::zones::leave;
use sim_core::ids::PlayerId;
use sim_core::impls::bulk_white::soulbond_partner;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn setup() -> Game {
    let mut g: Game = table(&["cloud", "seph"]);
    for p in [P0, P1] {
        let cs: Vec<_> = (0..20).map(|_| card(&g, "Plains")).collect();
        g.player_mut(p).library = cs;
    }
    g.player_mut(P1).life = 100;
    g
}

#[test]
fn pairs_on_its_own_entry_and_both_have_double_strike() {
    let mut g = setup();
    let ally = perm(&mut g, P0, "Serra Angel");
    let sp = perm(&mut g, P0, "Silverblade Paladin");
    assert_eq!(soulbond_partner(&g, sp), Some(ally));
    assert_eq!(soulbond_partner(&g, ally), Some(sp));
    assert!(combat::double_strike(&g, sp) && combat::double_strike(&g, ally));
}

#[test]
fn pairs_with_a_later_creature_and_unpairs_when_one_leaves() {
    let mut g = setup();
    let sp = perm(&mut g, P0, "Silverblade Paladin");
    assert!(soulbond_partner(&g, sp).is_none());
    assert!(!combat::double_strike(&g, sp));
    let late = perm(&mut g, P0, "Serra Angel");
    assert_eq!(soulbond_partner(&g, sp), Some(late));
    assert!(combat::double_strike(&g, late));
    leave(&mut g, late).unwrap();
    assert!(soulbond_partner(&g, sp).is_none());
    assert!(!combat::double_strike(&g, sp));
    // an unpaired Paladin pairs with the next creature
    let next = perm(&mut g, P0, "Serra Angel");
    assert_eq!(soulbond_partner(&g, sp), Some(next));
}

#[test]
fn equipped_cloud_paired_gets_four_sword_triggers() {
    let mut g = setup();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).is_cmd = true;
    let sw = perm(&mut g, P0, "Sword of Fire and Ice");
    g.perm_mut(sw).attached = Some(cl);
    let sp = perm(&mut g, P0, "Silverblade Paladin");
    assert_eq!(soulbond_partner(&g, sp), Some(cl));
    assert!(combat::double_strike(&g, cl));
    let lib0 = g.player(P0).library.len();
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    assert_eq!(lib0 - g.player(P0).library.len(), 4, "four Sword draws");
}

#[test]
fn cloud_entering_pairs_with_the_unpaired_paladin() {
    let mut g = setup();
    let sp = perm(&mut g, P0, "Silverblade Paladin");
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(soulbond_partner(&g, sp), Some(cl));
}
