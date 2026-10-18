//! Tier 6 batch `green2` (impls/t6_green2.rs) on hand-built positions. Seat 0 plays the Rograkh & Thrasios deck,
//! seat 1 the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-thrasios-creature-combo", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn squirrels(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).pow == 1).count()
}

// ======================================================== Chatterstorm
#[test]
fn chatterstorm_makes_a_squirrel_and_a_copy_for_each_earlier_spell() {
    let mut g = game();
    cast_spell(&mut g, P0, "Opt");
    cast_spell(&mut g, P0, "Preordain");
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 3, "the Squirrel and two copies' Squirrels");
    assert!(in_gy(&g, P0, "Chatterstorm"));
}

#[test]
fn chatterstorm_alone_makes_one_squirrel() {
    let mut g = game();
    cast_spell(&mut g, P0, "Chatterstorm");
    assert_eq!(squirrels(&g, P0), 1);
}
