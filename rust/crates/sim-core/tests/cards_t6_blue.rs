//! Tier 6 batch `blue` (impls/t6_blue.rs) on hand-built positions. Seat 0 plays the Rograkh & Silas deck, seat 1
//! the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::t6_blue as t6;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-silas-turbo-breach", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ======================================================== Borne Upon a Wind
#[test]
fn borne_upon_a_wind_draws_and_grants_flash_this_turn() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Borne Upon a Wind");
    assert_eq!(g.player(P0).hand.len(), before + 1, "the card left the hand, one was drawn");
    assert!(in_gy(&g, P0, "Borne Upon a Wind"));
    assert!(t6::flash_turn_grant(&g, P0));
    assert!(!t6::flash_turn_grant(&g, P1), "only the caster");
}

// ======================================================== Mnemonic Betrayal
fn put_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

#[test]
fn mnemonic_betrayal_takes_opposing_graveyards_for_the_turn() {
    let mut g = game();
    let bolt = put_gy(&mut g, P1, "Lightning Bolt");
    let land = put_gy(&mut g, P1, "Island");
    let mine = put_gy(&mut g, P0, "Opt");
    cast_free(&mut g, P0, "Mnemonic Betrayal");
    assert!(g.player(P1).gy.is_empty(), "the opposing graveyard is exiled");
    assert!(g.player(P1).exile.contains(&land), "a land can't be cast: it stays in exile");
    assert!(g.player(P0).hand.contains(&bolt), "the spell can be cast from among them");
    assert!(g.player(P0).agent_ids.contains(&bolt), "with mana of any type");
    assert_eq!(g.player(P0).stolen.get(&bolt), Some(&P1), "it still belongs to its owner");
    assert!(g.player(P0).impulse.contains(&bolt), "only this turn");
    assert!(g.player(P0).gy.contains(&mine), "own graveyard untouched");
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Mnemonic Betrayal"), "Betrayal is exiled");
}

// ======================================================== Timetwister
#[test]
fn timetwister_wheels_every_player_for_seven() {
    let mut g = game();
    let a = put_gy(&mut g, P0, "Opt");
    let b = put_gy(&mut g, P1, "Lightning Bolt");
    hand(&mut g, P1, &["Island", "Island"]);
    let total: usize = [P0, P1].iter().map(|&q| g.player(q).library.len() + g.player(q).hand.len() + g.player(q).gy.len()).sum();
    cast_free(&mut g, P0, "Timetwister");
    assert_eq!(g.player(P0).hand.len(), 7);
    assert_eq!(g.player(P1).hand.len(), 7);
    assert_eq!(g.player(P0).gy.len(), 1, "only Timetwister itself is in the graveyard");
    assert!(in_gy(&g, P0, "Timetwister"));
    assert!(g.player(P1).gy.is_empty());
    let now: usize = [P0, P1].iter().map(|&q| g.player(q).library.len() + g.player(q).hand.len() + g.player(q).gy.len()).sum();
    assert_eq!(total, now, "no card is lost");
    let all = |g: &Game, q: PlayerId, c: CardId| g.player(q).library.contains(&c) || g.player(q).hand.contains(&c);
    assert!(all(&g, P0, a) && all(&g, P1, b), "the graveyard cards were shuffled in");
}

// ======================================================== Demonic Counsel
fn lib_with(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
    c
}

#[test]
fn demonic_counsel_finds_a_demon_without_delirium() {
    let mut g = game();
    let demon = lib_with(&mut g, P0, "Doom Whisperer");
    let other = lib_with(&mut g, P0, "Grave Titan");
    cast_free(&mut g, P0, "Demonic Counsel");
    assert!(g.player(P0).hand.contains(&demon));
    assert!(!g.player(P0).hand.contains(&other), "only a Demon without delirium");
    assert!(in_gy(&g, P0, "Demonic Counsel"));
}

#[test]
fn demonic_counsel_without_a_demon_finds_nothing() {
    let mut g = game();
    lib_with(&mut g, P0, "Grave Titan");
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Demonic Counsel");
    assert_eq!(g.player(P0).hand.len(), n, "the card left, nothing came");
}

#[test]
fn demonic_counsel_with_delirium_finds_any_card() {
    let mut g = game();
    for n in ["Island", "Grave Titan", "Lightning Bolt", "Sol Ring"] {
        put_gy(&mut g, P0, n);
    }
    assert!(sim_core::impls::t6_green2::delirium(&g, P0));
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Demonic Counsel");
    assert_eq!(g.player(P0).hand.len(), n + 1, "one card was tutored");
}

// ======================================================== Rain of Filth
#[test]
fn rain_of_filth_sets_the_turn_flag_for_the_caster() {
    let mut g = game();
    assert!(!t6::rain_of_filth_active(&g, P0));
    cast_free(&mut g, P0, "Rain of Filth");
    assert!(t6::rain_of_filth_active(&g, P0));
    assert!(!t6::rain_of_filth_active(&g, P1));
    assert!(in_gy(&g, P0, "Rain of Filth"));
}
