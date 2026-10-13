//! Regression: Conspicuous Snoop with a Conspicuous Snoop on top of the library (a second copy, as when the one on
//! the battlefield is a copy) called its own `options` hook on the top card forever (stack overflow, Sauron vs T4,
//! seed 500211).

use sim_core::ids::PlayerId;
use sim_core::testkit::*;

#[test]
fn snoop_with_snoop_on_top_does_not_recurse() {
    let mut g = table(&["sauron", "veyran"]);
    let p = PlayerId(0);
    let snoop = take(&mut g, p, "Conspicuous Snoop");
    let src = perm(&mut g, p, "Conspicuous Snoop");
    g.player_mut(p).library.push(snoop);
    let f = g.registry.get(g.perm(src).cd.unwrap()).and_then(|i| i.options).expect("options");
    let o = f(&mut g, src, p, Some(false)).unwrap();
    // it may cast the Goblin on top, but must not loop
    assert!(o.len() <= 1);
}
