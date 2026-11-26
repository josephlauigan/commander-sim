use sim_core::engine::{turn, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{marchesa, sephfs_cards};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const MARCHESA: &str = "Marchesa, the Black Rose";
const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(card(g, name)))
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.contains(&card(g, name))
}

fn commander(g: &mut Game, name: &str) -> PermId {
    set_commander(g, P0, name);
    g.player_mut(P0).cmd_in_zone = false;
    let m = perm(g, P0, name);
    g.perm_mut(m).is_cmd = true;
    m
}

#[test]
fn marchesa_pending_return_leaves_the_commander_in_the_graveyard() {
    let mut g = table(&["sauron", "cloud"]);
    let m = commander(&mut g, MARCHESA);
    g.perm_mut(m).plus = 1;
    zones::die(&mut g, m, "sac").unwrap();
    assert_eq!(g.marchesa_due.len(), 1);
    assert!(!g.player(P0).cmd_in_zone);
    assert!(in_gy(&g, P0, MARCHESA));
    turn::end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, MARCHESA).is_some());
    assert!(!in_gy(&g, P0, MARCHESA));
    assert_eq!(g.player(P0).stats.get("marchesa_returns"), Some(&1));
}

#[test]
fn marchesa_commander_returns_from_the_graveyard_at_the_end_step() {
    let mut g = table(&["sauron", "cloud"]);
    let m = commander(&mut g, MARCHESA);
    g.perm_mut(m).plus = 1;
    zones::die(&mut g, m, "sac").unwrap();
    // Isolate return resolution from the owner's earlier command-zone decision.
    let cd = card(&g, MARCHESA);
    g.player_mut(P0).cmd_in_zone = false;
    if !g.player(P0).gy.contains(&cd) {
        g.player_mut(P0).gy.push(cd);
    }
    turn::end_step(&mut g, P0).unwrap();
    let back = on_bf(&g, P0, MARCHESA).expect("Marchesa returns from the graveyard");
    assert!(g.perm(back).is_cmd);
    assert!(!in_gy(&g, P0, MARCHESA));
    assert!(!g.player(P0).cmd_in_zone);
    assert_eq!(g.player(P0).stats.get("marchesa_returns"), Some(&1));
}

#[test]
fn marchesa_never_returns_a_commander_from_the_command_zone() {
    let mut g = table(&["sauron", "cloud"]);
    set_commander(&mut g, P0, MARCHESA);
    let cd = card(&g, MARCHESA);
    g.marchesa_due.push((P0, cd, P0));
    marchesa::marchesa_return(&mut g).unwrap();
    assert!(g.player(P0).cmd_in_zone);
    assert!(on_bf(&g, P0, MARCHESA).is_none());
    assert_eq!(g.player(P0).stats.get("marchesa_returns"), None);
}

#[test]
fn an_unrelated_commander_still_goes_to_the_command_zone() {
    let mut g = table(&["sauron", "cloud"]);
    let m = commander(&mut g, "Sauron, the Dark Lord");
    zones::die(&mut g, m, "sac").unwrap();
    assert!(g.player(P0).cmd_in_zone);
    assert!(!in_gy(&g, P0, "Sauron, the Dark Lord"));
    assert!(g.marchesa_due.is_empty());
}

#[test]
fn ghostform_returns_a_commander_after_death_or_exile() {
    for exile in [false, true] {
        let mut g = table(&["sephfs", "cloud"]);
        let m = commander(&mut g, SEPH);
        perm(&mut g, P0, "Kaya's Ghostform");
        if exile {
            zones::exile_perm(&mut g, m).unwrap();
        } else {
            zones::die(&mut g, m, "sac").unwrap();
        }
        let back = on_bf(&g, P0, SEPH).expect("Ghostform returns its commander");
        assert!(g.perm(back).is_cmd);
        assert!(!g.player(P0).cmd_in_zone);
        assert!(!in_gy(&g, P0, SEPH));
        assert!(!g.player(P0).exile.contains(&card(&g, SEPH)));
        assert_eq!(g.player(P0).stats.get("sf_ghostform"), Some(&1));
    }
}

#[test]
fn ghostform_never_returns_a_commander_from_the_command_zone() {
    let mut g = table(&["sephfs", "cloud"]);
    let m = commander(&mut g, SEPH);
    zones::die(&mut g, m, "sac").unwrap();
    assert!(g.player(P0).cmd_in_zone);
    // A pending Ghostform trigger cannot follow a card that moved to the command zone.
    g.perm_mut(m).data.set(DataKey::Ghostform, Val::Player(P0));
    assert!(!sephfs_cards::ghostform_return(&mut g, m, P0).unwrap());
    assert!(g.player(P0).cmd_in_zone);
    assert!(on_bf(&g, P0, SEPH).is_none());
    assert_eq!(g.player(P0).stats.get("sf_ghostform"), None);
}
