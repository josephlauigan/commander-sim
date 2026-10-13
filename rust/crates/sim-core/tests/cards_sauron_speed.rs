//! Sauron's speed metrics: the `army_hit` plan milestone, the Army power snapshots and the commander timing.

use sim_core::engine::{combat, turn, values};
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

#[test]
fn army_hit_is_set_on_the_turn_an_army_first_connects() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sauron, the Dark Lord");
    let t = token(&mut g, P0, 3); // not an Army
    g.player_mut(P0).turns = 3;
    combat::resolve_combat(&mut g, P0, &[t], P1, &[t]).unwrap();
    assert!(!g.player(P0).milestone.contains_key("army_hit"), "a non-Army connecting is not the milestone");
    let a = sim_core::engine::zones::amass(&mut g, P0, 2).map(|_| values::army_of(&g, P0).unwrap()).unwrap();
    g.player_mut(P0).turns = 5;
    combat::resolve_combat(&mut g, P0, &[a], P1, &[a]).unwrap();
    assert_eq!(g.player(P0).milestone.get("army_hit"), Some(&5));
    g.player_mut(P0).turns = 7;
    combat::resolve_combat(&mut g, P0, &[a], P1, &[a]).unwrap();
    assert_eq!(g.player(P0).milestone.get("army_hit"), Some(&5), "set once");
}

#[test]
fn army_power_is_recorded_at_the_start_of_turns_6_8_and_10() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sauron, the Dark Lord");
    sim_core::engine::zones::amass(&mut g, P0, 4).unwrap();
    g.player_mut(P0).turns = 4;
    turn::step_start(&mut g, P0).unwrap(); // turn 5: nothing recorded
    assert!(!g.player(P0).milestone.keys().any(|k| k.starts_with("army_p")));
    turn::step_start(&mut g, P0).unwrap(); // turn 6
    let a = values::army_of(&g, P0).unwrap();
    assert_eq!(g.player(P0).milestone.get("army_p6").copied(), Some(values::epow(&g, a) as u32));
    assert!(values::epow(&g, a) >= 4);
    assert!(!g.player(P0).milestone.contains_key("army_p8"));
}

#[test]
fn the_commander_resolving_sets_cmd_cast() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    lands(&mut g, P0, "Mountain", 2, false);
    g.player_mut(P0).turns = 4;
    assert!(!g.player(P0).milestone.contains_key("cmd_cast"));
    let cmd = g.player(P0).cmd;
    sim_core::engine::cast::cast_card(&mut g, P0, cmd, "cmd", Default::default()).unwrap();
    assert_eq!(g.player(P0).milestone.get("cmd_cast"), Some(&4));
}
