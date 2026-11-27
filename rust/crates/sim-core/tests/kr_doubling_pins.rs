//! Karn, Legacy Reforged deck: the final-state pins behind the `enters_census_double_allow.txt` rows for four cards.
//! With each enter-trigger doubler (Elesh Norn, Mother of Machines; Starfield Vocalist; Panharmonicon) on the table, the
//! trigger of each of these cards runs exactly one extra time, no more and no fewer:
//! - Batterskull: two Germs are made (the second takes the Equipment, the first dies as a 0/0), so one Germ survives;
//! - Hammer of Nazahn: the Equipment is attached once and stays on that creature: the game log has exactly one attach
//!   line and the final holder is the same creature as without a doubler (Serra Angel), except under Elesh Norn, where
//!   `attach_target` picks Yoshimaru from the first instance on (observed, not a second move: still one attach line);
//! - Portal to Phyrexia: each opponent sacrifices 3 creatures (3 without a doubler); the extra instance takes 3 more, but
//!   only 2 are left on a five-creature seat, so 5 in all;
//! - Spine of Ish Sah: one permanent is destroyed without a doubler, two in total with one (across both opponents).
//! Deterministic: the same table every run, no random draws; the fixture puts five creatures on every seat.

use sim_core::engine::{cast, life};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game, Step};
mod play_common;
use play_common::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const DOUBLERS: [&str; 3] = ["Elesh Norn, Mother of Machines", "Starfield Vocalist", "Panharmonicon"];

/// the table (five creatures per seat; the doubler, if any, on seat 0), the card cast from the hand, state checked
fn cast_with(doubler: Option<&str>, name: &str) -> (Game, sim_core::ids::CardId) {
    let mut g = table(&["karn_reforged", "seph", "sauron"]);
    g.log = Some(vec![]);
    g.active = Some(P0);
    g.step = Step::Main1;
    for p in [P0, P1, P2] {
        for n in ["Mother of Runes", "Serra Angel", "Viscera Seer", "Yoshimaru, Ever Faithful", "Thraben Inspector"] {
            perm(&mut g, p, n);
        }
    }
    if let Some(n) = doubler {
        perm(&mut g, P0, n);
    }
    g.log = Some(vec![]); // the cast alone
    let c = card(&g, name);
    g.player_mut(P0).hand.push(c);
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap(), "{name} resolves");
    life::check_state(&mut g).unwrap();
    (g, c)
}

/// the Germ tokens made (all, alive)
fn germs(g: &Game) -> (usize, usize) {
    let all = g.perms.iter().filter(|m| m.token && m.ttypes.contains(&"germ")).count();
    let live = g.perms.iter().filter(|m| m.token && m.ttypes.contains(&"germ") && m.on_bf).count();
    (all, live)
}

/// the name of the permanent Hammer of Nazahn is attached to
fn hammer_target(g: &Game, c: sim_core::ids::CardId) -> String {
    let h: PermId =
        g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("Hammer on the battlefield");
    let t = g.perm(h).attached.expect("Hammer is attached");
    assert!(g.is_creature(t), "Hammer is attached to a creature");
    g.perm(t).name.to_string()
}

#[test]
fn batterskull_makes_exactly_one_extra_germ_and_one_survives() {
    let (g, _) = cast_with(None, "Batterskull");
    assert_eq!(germs(&g), (1, 1), "no doubler: one Germ");
    for d in DOUBLERS {
        let (g, _) = cast_with(Some(d), "Batterskull");
        assert_eq!(germs(&g), (2, 1), "{d}: two Germs made, the first dies without the Equipment");
    }
}

/// how many times the log says Hammer of Nazahn attached to a creature
fn hammer_attaches(g: &Game) -> usize {
    g.log.as_ref().unwrap().iter().filter(|l| l.contains("Hammer of Nazahn attaches to")).count()
}

#[test]
fn hammer_of_nazahn_is_attached_once_and_stays() {
    let (g, c) = cast_with(None, "Hammer of Nazahn");
    assert_eq!(hammer_attaches(&g), 1);
    let base = hammer_target(&g, c);
    assert_eq!(base, "Serra Angel");
    for d in DOUBLERS {
        let (g, c) = cast_with(Some(d), "Hammer of Nazahn");
        assert_eq!(hammer_attaches(&g), 1, "{d}: the extra instance does not move it");
        let want = if d.starts_with("Elesh") { "Yoshimaru, Ever Faithful" } else { base.as_str() };
        assert_eq!(hammer_target(&g, c), want, "{d}: the final holder");
    }
}

#[test]
fn portal_to_phyrexia_sacrifices_three_then_five_per_opponent() {
    let (g, _) = cast_with(None, "Portal to Phyrexia");
    for p in [P1, P2] {
        assert_eq!(g.player(p).gy.len(), 3, "no doubler: {p:?} sacrifices 3");
    }
    for d in DOUBLERS {
        let (g, _) = cast_with(Some(d), "Portal to Phyrexia");
        for p in [P1, P2] {
            assert_eq!(g.player(p).gy.len(), 5, "{d}: {p:?} sacrifices 3 then the remaining 2");
        }
    }
}

#[test]
fn spine_of_ish_sah_destroys_one_then_two_permanents_in_total() {
    let dead = |g: &Game| [P1, P2].map(|p| g.player(p).gy.len());
    let (g, _) = cast_with(None, "Spine of Ish Sah");
    assert_eq!(dead(&g).iter().sum::<usize>(), 1, "no doubler: one permanent");
    for d in DOUBLERS {
        let (g, _) = cast_with(Some(d), "Spine of Ish Sah");
        assert_eq!(dead(&g).iter().sum::<usize>(), 2, "{d}: two permanents in total, {:?}", dead(&g));
    }
}
