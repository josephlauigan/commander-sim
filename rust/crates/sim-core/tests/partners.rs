//! Two commanders per seat (Partner): the Tymna decks. Tymna the Weaver ({1}{W}{B}) and Thrasios, Triton Hero
//! ({G}{U}) sit in the command zone together, each with its own commander tax and its own commander damage.

use sim_core::ai::brain;
use sim_core::engine::{life, mana, values, zones};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// a table whose first seat is the Thrasios deck, with plenty of untapped lands of every colour
fn partner_table() -> sim_core::state::Game {
    let mut g = table(&["thrasios", "veyran"]);
    for n in ["Plains", "Island", "Swamp", "Forest"] {
        lands(&mut g, P0, n, 6, false);
    }
    g
}

fn untap(g: &mut sim_core::state::Game, p: PlayerId) {
    for l in g.player(p).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
}

fn tymna_thrasios(g: &sim_core::state::Game) -> (CardId, CardId) {
    (card(g, "Tymna the Weaver"), card(g, "Thrasios, Triton Hero"))
}

fn perm_of(g: &sim_core::state::Game, c: CardId) -> Option<PermId> {
    g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c))
}

#[test]
fn both_start_in_the_command_zone_and_neither_is_in_the_library() {
    let g = table(&["thrasios", "veyran"]);
    let (a, b) = tymna_thrasios(&g);
    let pl = g.player(P0);
    assert_eq!((pl.cmd, pl.cmd2), (a, Some(b)));
    assert!(pl.cmd_in_zone && pl.cmd2_in_zone);
    assert!(!pl.library.contains(&a) && !pl.library.contains(&b) && !pl.hand.contains(&a));
    assert_eq!(pl.library.len() + 2, pl.deck_names.len());
    assert_eq!(pl.commanders().collect::<Vec<_>>(), vec![a, b]);
    assert_eq!(pl.cmds_in_zone().count(), 2);
}

#[test]
fn both_can_be_cast_from_the_command_zone() {
    let mut g = partner_table();
    let (a, b) = tymna_thrasios(&g);
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert!(brain::do_cast(&mut g, P0, b, None).unwrap());
    let (ma, mb) = (perm_of(&g, a).unwrap(), perm_of(&g, b).unwrap());
    assert!(g.perm(ma).is_cmd && g.perm(mb).is_cmd);
    assert!(!g.player(P0).cmd_in_zone && !g.player(P0).cmd2_in_zone);
    assert_eq!((g.player(P0).tax, g.player(P0).tax2), (2, 2));
    assert!(values::commander_out(&g, P0));
}

#[test]
fn each_has_its_own_commander_tax() {
    let mut g = partner_table();
    let (a, b) = tymna_thrasios(&g);
    assert_eq!(mana::cost_of(&g, P0, a), (1, "WB".to_string()));
    assert_eq!(mana::cost_of(&g, P0, b), (0, "GU".to_string()));
    // cast Tymna twice and Thrasios once: +2/+4 for Tymna, +0/+2 for Thrasios
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert_eq!(mana::cost_of(&g, P0, a), (3, "WB".to_string()));
    assert_eq!(mana::cost_of(&g, P0, b), (0, "GU".to_string()), "Tymna's casts don't tax Thrasios");
    untap(&mut g, P0);
    let m = perm_of(&g, a).unwrap();
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).cmd_in_zone);
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert_eq!(mana::cost_of(&g, P0, a), (5, "WB".to_string()));
    assert!(brain::do_cast(&mut g, P0, b, None).unwrap());
    assert_eq!(mana::cost_of(&g, P0, b), (2, "GU".to_string()));
    assert_eq!((g.player(P0).tax, g.player(P0).tax2), (4, 2));
}

#[test]
fn twenty_one_from_one_partner_eliminates() {
    let mut g = partner_table();
    g.player_mut(P1).cmd2_dmg[0] = 21;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive && g.over);
    assert_eq!(g.player(P1).death_kind, "commander damage");
    let mut g = partner_table();
    g.player_mut(P1).cmd_dmg[0] = 21;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

#[test]
fn twenty_plus_twenty_split_across_both_does_not() {
    let mut g = partner_table();
    g.player_mut(P1).cmd_dmg[0] = 20;
    g.player_mut(P1).cmd2_dmg[0] = 20;
    life::check_state(&mut g).unwrap();
    assert!(g.player(P1).alive && !g.over);
}

#[test]
fn combat_damage_is_counted_per_commander() {
    let mut g = partner_table();
    let (a, b) = tymna_thrasios(&g);
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert!(brain::do_cast(&mut g, P0, b, None).unwrap());
    let (ma, mb) = (perm_of(&g, a).unwrap(), perm_of(&g, b).unwrap());
    values::add_cmd_dmg(&mut g, P1, P0, ma, 12);
    values::add_cmd_dmg(&mut g, P1, P0, mb, 9);
    assert_eq!((g.player(P1).cmd_dmg[0], g.player(P1).cmd2_dmg[0]), (12, 9));
    assert_eq!((values::cmd_dmg_by(&g, P1, P0, ma), values::cmd_dmg_by(&g, P1, P0, mb)), (12, 9));
    life::check_state(&mut g).unwrap();
    assert!(g.player(P1).alive, "12 + 9 from two different commanders is not 21 from one");
    values::add_cmd_dmg(&mut g, P1, P0, mb, 12);
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

#[test]
fn each_goes_back_to_the_command_zone() {
    let mut g = partner_table();
    let (a, b) = tymna_thrasios(&g);
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert!(brain::do_cast(&mut g, P0, b, None).unwrap());
    let m = perm_of(&g, b).unwrap();
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).cmd2_in_zone && !g.player(P0).cmd_in_zone, "only Thrasios went back");
    assert!(!g.player(P0).gy.contains(&b));
    let m = perm_of(&g, a).unwrap();
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).cmd_in_zone && !g.player(P0).gy.contains(&a));
}

#[test]
fn bastion_protector_buffs_both_commanders() {
    let mut g = partner_table();
    let (a, b) = tymna_thrasios(&g);
    assert!(brain::do_cast(&mut g, P0, a, None).unwrap());
    assert!(brain::do_cast(&mut g, P0, b, None).unwrap());
    let (ma, mb) = (perm_of(&g, a).unwrap(), perm_of(&g, b).unwrap());
    let (pa, pb) = (values::epow(&g, ma), values::epow(&g, mb));
    let other = perm(&mut g, P0, "Aven Mindcensor");
    perm(&mut g, P0, "Bastion Protector");
    assert_eq!(values::epow(&g, ma), pa + 2);
    assert_eq!(values::epow(&g, mb), pb + 2);
    assert_eq!(values::epow(&g, other), 2, "a non-commander creature is not buffed");
}

#[test]
fn a_single_commander_seat_is_unchanged() {
    let mut g = table(&["seph", "veyran"]);
    let pl = g.player(P0);
    assert_eq!(pl.cmd2, None);
    assert!(pl.cmd_in_zone && !pl.cmd2_in_zone);
    assert_eq!(pl.commanders().count(), 1);
    assert_eq!(pl.library.len() + 1, pl.deck_names.len());
    let cmd = g.player(P0).cmd;
    assert_eq!(mana::cost_of(&g, P0, cmd), (3, "GWUB".to_string()));
    g.player_mut(P0).tax = 4;
    assert_eq!(mana::cost_of(&g, P0, cmd), (7, "GWUB".to_string()));
    g.player_mut(P1).cmd_dmg[0] = 21;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}
