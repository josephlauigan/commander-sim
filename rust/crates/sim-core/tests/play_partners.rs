//! Practice mode with a Partner deck (kraum: Tymna the Weaver + Kraum, Ludevic's Opus): the person casts either
//! commander from the command zone, each with its own tax, and each goes back to the command zone when it dies.
//! (`{"do": "cast", "zone": "cmd", "cmd": i}`: i is the index among the seat's commanders, or the commander's name.)

mod play_common;
use play_common::*;
use sim_core::engine::{mana as emana, zones};
use sim_core::human::{human::human_main, mana};

/// more lands of every colour, all tapped for mana into the pool
fn fund(g: &mut Game, p: PlayerId, each: usize) {
    for n in ["Plains", "Swamp", "Island", "Mountain"] {
        lands(g, p, n, each, false);
    }
    while let Some(s) = mana::sources(g, p).first() {
        let id = s.id as i64;
        mana::tap(g, p, id, None).unwrap();
    }
}

fn on_battlefield(g: &Game, c: sim_core::ids::CardId) -> Option<PermId> {
    g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c))
}

fn run(g: &mut Game, answers: Vec<F>) -> std::sync::Arc<ScriptHuman> {
    let h = seat(g, P0, answers);
    human_main(g, P0, false).unwrap();
    h
}

#[test]
fn either_commander_is_cast_from_the_command_zone_with_its_own_tax() {
    let mut g = table(&["kraum", "veyran"]);
    let (tymna, kraum) = (card(&g, "Tymna the Weaver"), card(&g, "Kraum, Ludevic's Opus"));
    assert_eq!(g.player(P0).cmds_in_zone().collect::<Vec<_>>(), vec![tymna, kraum]);
    fund(&mut g, P0, 6);
    // the second commander first, by index; the first one is still in the zone
    let h = run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd", "cmd": 1})), pass()]);
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert!(on_battlefield(&g, kraum).is_some() && on_battlefield(&g, tymna).is_none());
    assert_eq!((g.player(P0).cmd_in_zone, g.player(P0).cmd2_in_zone), (true, false));
    assert_eq!((g.player(P0).tax, g.player(P0).tax2), (0, 2));
    // casting the one that is out of the zone is refused with the reason
    let h = run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd", "cmd": 1})), pass()]);
    assert_eq!(h.told()[0].0, "invalid");
    assert!(h.told()[0].1.contains("not in the command zone"), "{}", h.told()[0].1);
    // the first one by name (the default with no "cmd" is the first one in the zone)
    let h = run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd", "cmd": "Tymna the Weaver"})), pass()]);
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert!(on_battlefield(&g, tymna).is_some());
    assert_eq!((g.player(P0).tax, g.player(P0).tax2), (2, 2));
}

#[test]
fn a_recast_pays_that_commanders_tax_and_each_returns_to_the_zone() {
    let mut g = table(&["kraum", "veyran"]);
    let (tymna, kraum) = (card(&g, "Tymna the Weaver"), card(&g, "Kraum, Ludevic's Opus"));
    fund(&mut g, P0, 6);
    run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd"})), pass()]); // no "cmd": the first in the zone, Tymna
    assert!(on_battlefield(&g, tymna).is_some() && on_battlefield(&g, kraum).is_none());
    run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd"})), pass()]); // now the only one left in the zone
    assert!(on_battlefield(&g, kraum).is_some());
    // both die: each goes back to the command zone, the engine's usual answer to a commander leaving
    for c in [tymna, kraum] {
        let m = on_battlefield(&g, c).unwrap();
        zones::die(&mut g, m, "destroy").unwrap();
    }
    assert!(g.player(P0).cmd_in_zone && g.player(P0).cmd2_in_zone);
    assert!(!g.player(P0).gy.contains(&tymna) && !g.player(P0).gy.contains(&kraum));
    // the recasts cost the base plus 2 for each earlier cast of that commander only
    let (t0, k0) = (emana::cost_of(&g, P0, tymna), emana::cost_of(&g, P0, kraum));
    assert_eq!(t0.0, 3, "Tymna {{1}}{{W}}{{B}} + 2");
    assert_eq!(k0.0, 5, "Kraum {{3}}{{U}}{{R}} + 2 (Kraum's own cast is its first: tax 2)");
    // pay for Kraum's recast; Tymna stays in the zone and keeps her own tax
    fund(&mut g, P0, 6);
    run(&mut g, vec![act(json!({"do": "cast", "zone": "cmd", "cmd": 1})), pass()]);
    assert!(on_battlefield(&g, kraum).is_some() && g.player(P0).cmd_zone_of(tymna));
    assert_eq!((g.player(P0).tax, g.player(P0).tax2), (2, 4));
}
