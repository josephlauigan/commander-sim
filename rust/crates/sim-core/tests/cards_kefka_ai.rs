//! Kefka's AI (impls/kefka.rs): the cards the card report showed as rarely cast, and the swap-in cards.

use sim_core::ai::{act, brain};
use sim_core::hooks::Opt;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn opts_of(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn labels(g: &mut Game, p: PlayerId, post: bool) -> Vec<String> {
    opts_of(g, p, post).iter().map(|o| format!("{} [{:.2}]", o.label, o.utility)).collect()
}

fn take(g: &mut Game, p: PlayerId, post: bool, prefix: &str) -> bool {
    let opts = opts_of(g, p, post);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(prefix))
        .unwrap_or_else(|| panic!("no option {prefix:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn table2() -> Game {
    table(&["kefka", "veyran"])
}

#[test]
fn probe_purge() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Mountain", 3, false);
    hand(&mut g, P0, &["Kaervek's Purge"]);
    perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Eternal Witness"); perm(&mut g, P1, "Craterhoof Behemoth"); perm(&mut g, P1, "Grave Titan");
    for post in [false, true] {
        println!("post={post}: {:?}", labels(&mut g, P0, post));
    }
    panic!();
}
