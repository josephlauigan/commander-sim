//! tymna-fix: Witherbloom Charm's modes (card rules). The AI fixes of the first thrasios run (Lagrella, the protection
//! spells) went with the thrasios seat's hand AI on 2026-11-01.

use sim_core::hooks::{Action, Opt};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl(key: &str) -> Game {
    table(&[key, "veyran", "seph", "galadriel"])
}

fn on(g: &Game, m: sim_core::ids::PermId) -> bool {
    g.perm(m).on_bf
}

fn cast_charm(g: &mut Game, mode: i32, target: Option<sim_core::ids::PermId>) {
    let c = hand(g, P0, &["Witherbloom Charm"])[0];
    sim_core::engine::cast::cast_card(g, P0, c, "hand", Ctx { x: mode, target, ..Default::default() }).unwrap();
}

#[test]
fn witherbloom_charm_has_its_three_modes() {
    let mut g = tbl("thrasios");
    // mode 2: gain 5 life
    let life = g.player(P0).life;
    cast_charm(&mut g, 2, None);
    assert_eq!(g.player(P0).life, life + 5);
    // mode 3: destroy a nonland permanent with mana value 2 or less, not a bigger one
    let small = perm(&mut g, P1, "Birds of Paradise");
    let big = perm(&mut g, P1, "Lyra Dawnbringer");
    cast_charm(&mut g, 3, Some(big));
    assert!(on(&g, big), "mana value 5 is not a legal target");
    cast_charm(&mut g, 3, Some(small));
    assert!(!on(&g, small));
    // mode 1: sacrifice a token, draw two
    let t = token(&mut g, P0, 1);
    let hand_before = g.player(P0).hand.len();
    cast_charm(&mut g, 1, None);
    assert!(!on(&g, t), "the token was sacrificed");
    assert_eq!(g.player(P0).hand.len(), hand_before + 2);
}

#[test]
fn witherbloom_charm_is_offered_as_a_hand_play() {
    let mut g = tbl("thrasios");
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    token(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Witherbloom Charm"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o: Vec<Opt> = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(o.iter().any(|x| x.label.contains("draw two")), "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert!(o.iter().all(|x| matches!(x.act, Some(Action::Plan { .. }))));
}

