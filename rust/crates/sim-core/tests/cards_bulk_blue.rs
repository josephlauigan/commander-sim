//! The elrond deck's blue spells (impls/bulk_blue.rs) on hand-built positions. Seat 0 plays the elrond deck, seat 1
//! the seph deck.

use sim_core::ai::act;
use sim_core::engine::{cast, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::engine::hooks::fire_trigger;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["elrond", "seph"])
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    f(g, m, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn jace_token(g: &Game, p: PlayerId) -> Option<PermId> {
    sim_core::impls::alela::jace_token(g, p)
}

// ======================================================== Jace, Reality Sculptor
#[test]
fn jace_plus_empowers_by_the_islands_you_control() {
    let mut g = game();
    lands(&mut g, P0, "Island", 5, false);
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(5);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let plus = opts.iter().find(|o| o.label.ends_with("+1")).expect("the +1 is offered");
    assert!(perform(&mut g, P0, plus));
    assert_eq!(g.perm(j).loyalty, Some(6));
    let t = jace_token(&g, P0).expect("the Jace token was made");
    assert_eq!(g.perm(t).loyalty, Some(5), "X = five Islands");
}

#[test]
fn jace_minus_three_shrinks_attackers_until_your_next_turn() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(5);
    let a = token(&mut g, P1, 6);
    let b = token(&mut g, P1, 6);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let minus = opts.iter().find(|o| o.label.ends_with("-3")).expect("the -3 is offered facing a big board");
    assert!(perform(&mut g, P0, minus));
    assert_eq!(g.perm(j).loyalty, Some(2));
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a, b], d: P0 }).unwrap();
    assert_eq!(values::epow(&g, a), 1, "6 - 5");
    // an attack on someone else is not shrunk
    let mut g = game();
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).data.set(sim_core::state::DataKey::Used, sim_core::state::Val::Bool(true));
    let a = token(&mut g, P1, 6);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a], d: PlayerId(1) }).unwrap();
    assert_eq!(values::epow(&g, a), 6);
    // the effect ends at the Jace's controller's next upkeep
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a], d: P0 }).unwrap();
    assert_eq!(values::epow(&g, a), 6);
}

#[test]
fn jace_zero_needs_twenty_five_loyalty_among_your_jaces() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(10);
    let n = g.player(P1).library.len();
    assert!(n > 1);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    assert!(!opts.iter().any(|o| o.label.ends_with("+0")), "10 loyalty: no 0 ability");
    sim_core::impls::alela::empower(&mut g, P0, 15);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let zero = opts.iter().find(|o| o.label.ends_with("+0")).expect("10 + 15 = 25");
    assert!(perform(&mut g, P0, zero));
    assert_eq!(g.player(P1).library.len(), 1);
    assert_eq!(g.player(P1).exile.len(), n - 1);
}
