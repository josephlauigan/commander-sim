//! Casual combos for the Tier 1 pool deck `teysa-orzhov-aristocrats` (rust/agents/cc-teysa.md): the unbounded death
//! loops its list holds that the catalogue had not modeled. The Plunderer + Skeleton + altar loop is already in the
//! catalogue (t45_urza_yawgmoth.rs); these are the other two.
//!
//! - Teysa Karlov or Drivnod, Carnage Dominus + Pitiless Plunderer + Reassembling Skeleton + a free sacrifice outlet
//!   (Viscera Seer, Carrion Feeder, Yahenni): with the death trigger doubled, each sacrifice of the Skeleton makes two
//!   Treasures, which pay its {1}{B} return. Unbounded deaths; a death payoff wins.
//! - Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog or Sifter of Skulls: sacrificing the Skeleton to the
//!   Altar makes one mana and an Eldrazi token, sacrificing the token makes another, and the two pay {1}{B} for the
//!   Skeleton. Unbounded deaths; a death payoff wins. Only for this deck: sephfs runs the Pawn loop through its own
//!   plan (sephfs.rs), and yawgmoth keeps its own lines.
//!
//! Neither is a two-card combo (Tier 1 allows none): each needs three pieces and a payoff.

use crate::engine::mana::can_pay;
use crate::ids::{PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, loop_payoff};
use crate::state::Game;

pub const DECK: &str = "teysa-orzhov-aristocrats";
const TEYSA: &str = "Teysa Karlov";
const DRIVNOD: &str = "Drivnod, Carnage Dominus";
const PLUNDERER: &str = "Pitiless Plunderer";
const SKELETON: &str = "Reassembling Skeleton";
const ALTAR: &str = "Phyrexian Altar";
/// a dies trigger once more for each death
const DOUBLERS: &[&str] = &[TEYSA, DRIVNOD];
/// sacrifice another creature at no cost, any number of times
const FREE_OUTLETS: &[&str] = &["Viscera Seer", "Carrion Feeder", "Yahenni, Undying Partisan"];
/// a nontoken creature of yours dies: a token that sacrifices for mana
const SPAWNERS: &[&str] = &["Pawn of Ulamog", "Sifter of Skulls"];

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| &*g.db.get(c).name == name)
    })
}

fn any_bf(g: &Game, p: PlayerId, names: &[&str]) -> Option<PermId> {
    names.iter().find_map(|n| on_bf(g, p, n))
}

/// Reassembling Skeleton on the battlefield, or in the graveyard with {1}{B} open to start the loop
fn skeleton(g: &Game, p: PlayerId) -> bool {
    on_bf(g, p, SKELETON).is_some()
        || (g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == SKELETON) && can_pay(g, p, 1, "B", false))
}

/// the loops need the Skeleton to come back from the graveyard and the death triggers to work
fn blocked(g: &Game, p: PlayerId) -> bool {
    g.player(p).key != DECK || super::mine::loops_blocked(g, p)
}

fn doubled_plunderer(g: &Game, p: PlayerId) -> Ready {
    if blocked(g, p) {
        return NOT_READY();
    }
    let (Some(dbl), Some(pl), Some(outlet)) =
        (any_bf(g, p, DOUBLERS), on_bf(g, p, PLUNDERER), any_bf(g, p, FREE_OUTLETS))
    else {
        return NOT_READY();
    };
    if !skeleton(g, p) {
        return NOT_READY();
    }
    let Some(payoff) = loop_payoff(g, p) else { return NOT_READY() };
    (true, vec![dbl, pl, outlet, payoff], vec![])
}

fn altar_spawner(g: &Game, p: PlayerId) -> Ready {
    if blocked(g, p) {
        return NOT_READY();
    }
    let (Some(altar), Some(spawner)) = (on_bf(g, p, ALTAR), any_bf(g, p, SPAWNERS)) else {
        return NOT_READY();
    };
    if !skeleton(g, p) {
        return NOT_READY();
    }
    let Some(payoff) = loop_payoff(g, p) else { return NOT_READY() };
    (true, vec![altar, spawner, payoff], vec![])
}

pub static COMBOS: [Combo; 2] = [
    Combo {
        name: "Teysa Karlov / Drivnod + Pitiless Plunderer + Reassembling Skeleton + a free outlet + a death payoff",
        groups: &[DOUBLERS, &[PLUNDERER], &[SKELETON], FREE_OUTLETS, &crate::ai::plans::DRAIN_PAYOFFS],
        ready: doubled_plunderer,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Phyrexian Altar + Reassembling Skeleton + Pawn of Ulamog / Sifter of Skulls + a death payoff",
        groups: &[&[ALTAR], &[SKELETON], SPAWNERS, &crate::ai::plans::DRAIN_PAYOFFS],
        ready: altar_spawner,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
];
