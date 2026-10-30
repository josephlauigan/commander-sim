//! How long finding and paying mana takes (Phase 3, the mana pool): `mana_units`, `can_pay` and `pay` for one player of
//! the `clone_cost` mid-game table with a fixed mana base (basics, Command Tower, Ancient Tomb, Sol Ring, Arcane Signet,
//! The Mightstone and Weakstone, Llanowar Elves), first with an empty pool, then with floating mana in it (two {R},
//! three of any colour, two of Karn's artifact-only colourless). `pay` runs on a copy of the game; the copy's own time
//! is measured alone and taken off. Not a sim: one fixed board, asked again and again.
//!     cargo run --release -p sim-core --example mana_cost

use sim_core::engine::mana::{can_pay, mana_units, pay};
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit;
use std::hint::black_box;
use std::time::Instant;

const P0: PlayerId = PlayerId(0);

fn board() -> Game {
    let t4: Vec<&str> =
        testkit::decks().iter().filter(|d| d.tier.as_deref() == Some("t4")).take(4).map(|d| &*d.key).collect();
    let mut g = testkit::table(&t4);
    for i in 1..4 {
        let p = PlayerId(i);
        for _ in 0..8 {
            let c = g.player_mut(p).library.pop().unwrap();
            g.add_land(p, c, false);
        }
    }
    for name in ["Forest", "Forest", "Forest", "Island", "Island", "Mountain", "Command Tower", "Ancient Tomb"] {
        let c = g.db.id(name).unwrap_or_else(|| panic!("{name}"));
        g.add_land(P0, c, false);
    }
    for name in ["Sol Ring", "Arcane Signet", "The Mightstone and Weakstone", "Llanowar Elves"] {
        let c = g.db.id(name).unwrap_or_else(|| panic!("{name}"));
        let m = g.new_perm(P0, Some(c), intern("Token"), 1, 1);
        g.perm_mut(m).on_bf = true;
        g.perm_mut(m).sick = false;
        g.player_mut(P0).perms.push(m);
    }
    g
}

/// floating mana: {R}{R}, three of any colour, two of Karn's colourless
fn float(g: &mut Game) {
    use sim_core::engine::pool::Kind;
    let pl = g.player_mut(P0);
    pl.mana.add(Kind::R, 2);
    pl.mana.add(Kind::Any, 3);
    pl.mana.add_art(2);
}

fn run(g: &Game, label: &str) {
    let n = 200_000;
    let time = |what: &str, f: &dyn Fn() -> i64| -> f64 {
        let t = Instant::now();
        let mut keep = 0i64;
        for _ in 0..n {
            keep += f();
        }
        let per = t.elapsed().as_secs_f64() / n as f64 * 1e9;
        println!("{label:6} {what:22} {per:8.1} ns ({keep})");
        per
    };
    time("mana_units", &|| mana_units(black_box(g), P0, false).len() as i64);
    time("can_pay {3}{G}", &|| can_pay(black_box(g), P0, 3, "G", false) as i64);
    time("can_pay {6}{U}{R}", &|| can_pay(black_box(g), P0, 6, "UR", false) as i64);
    let copy = time("copy (taken off)", &|| black_box(g.clone()).players.len() as i64);
    let paid = time("copy + pay {3}{G}", &|| {
        let mut c = black_box(g.clone());
        pay(&mut c, P0, 3, "G", false).unwrap() as i64
    });
    println!("{label:6} {:22} {:8.1} ns", "pay {3}{G}", paid - copy);
}

fn main() {
    let mut g = board();
    run(&g, "empty");
    float(&mut g);
    run(&g, "float");
}
