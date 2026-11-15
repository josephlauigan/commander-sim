//! Cards added to JD's `urdragon` deck on 11-14 whose model is compiled from the Oracle text: Niv-Mizzet, the Firemind
//! and Utvara Hellkite. Each test checks the card's Oracle behaviour.

use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    let mut g =
        table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
    g.active = Some(P0);
    g
}

fn opp_life(g: &Game) -> i32 {
    g.players.iter().enumerate().filter(|(i, _)| *i != 0).map(|(_, q)| q.life).sum()
}

fn dragon_tokens(g: &Game) -> Vec<PermId> {
    g.player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).ttypes.contains(&"dragon"))
        .collect()
}

#[test]
fn niv_mizzet_pings_an_opponent_for_each_card_you_draw() {
    let mut g = tbl();
    perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    let (life, hand) = (opp_life(&g), g.player(P0).hand.len());
    sim_core::engine::zones::draw(&mut g, P0, 3, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 3);
    assert_eq!(life - opp_life(&g), 3, "one damage for each card drawn");
}

#[test]
fn niv_mizzet_taps_to_draw_and_that_draw_pings_too() {
    use sim_core::dsl::activate;
    let mut g = tbl();
    let niv = perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    let (life, hand) = (opp_life(&g), g.player(P0).hand.len());
    assert!(activate(&mut g, P0, niv, 1).unwrap());
    assert!(g.perm(niv).tapped);
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert_eq!(life - opp_life(&g), 1);
    assert!(!activate(&mut g, P0, niv, 1).unwrap(), "it is tapped");
}

#[test]
fn niv_mizzet_does_not_ping_for_an_opponents_draw() {
    let mut g = tbl();
    perm(&mut g, P0, "Niv-Mizzet, the Firemind");
    let life = opp_life(&g);
    sim_core::engine::zones::draw(&mut g, PlayerId(1), 2, false).unwrap();
    assert_eq!(opp_life(&g), life);
}

#[test]
fn utvara_hellkite_makes_a_6_6_flying_dragon_for_each_attacking_dragon() {
    let mut g = tbl();
    perm(&mut g, P0, "Utvara Hellkite");
    perm(&mut g, P0, "Smaug the Magnificent");
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    let attackers = g.player(P0).perms.iter().filter(|&&m| g.perm(m).tapped && !g.perm(m).token).count();
    assert!(attackers >= 1, "a Dragon attacked");
    let toks = dragon_tokens(&g);
    assert_eq!(toks.len(), attackers, "a token for each attacking Dragon, Utvara included");
    for t in toks {
        assert_eq!((g.perm(t).pow, g.perm(t).tgh), (6, 6));
        assert!(!g.perm(t).tapped, "the tokens are not attacking");
    }
}

#[test]
fn utvara_hellkite_ignores_an_attacking_creature_that_is_not_a_dragon() {
    let mut g = tbl();
    let u = perm(&mut g, P0, "Utvara Hellkite");
    g.perm_mut(u).sick = true; // it stays home
    let plain = token(&mut g, P0, 5); // a plain creature attacks
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(g.perm(plain).tapped, "the plain creature attacked");
    assert!(dragon_tokens(&g).is_empty());
}

#[test]
fn utvara_hellkite_counts_only_the_dragons_in_a_mixed_attack() {
    let mut g = tbl();
    let u = perm(&mut g, P0, "Utvara Hellkite");
    g.perm_mut(u).sick = true; // it stays home
    let smaug = perm(&mut g, P0, "Smaug the Magnificent");
    let plain = token(&mut g, P0, 5);
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(g.perm(smaug).tapped && g.perm(plain).tapped, "both attacked");
    assert_eq!(dragon_tokens(&g).len(), 1, "one token: for Smaug, not for the plain creature");
}

// ---- Nature's Lore and Three Visits (the AI's search; a person's is in practice_gaps_a.rs)
fn cast_ramp(g: &mut Game, name: &str) {
    let c = hand(g, P0, &[name])[0];
    sim_core::engine::cast::cast_card(g, P0, c, "hand", sim_core::state::Ctx::default()).unwrap();
}

#[test]
fn the_ai_fetches_a_forest_card_and_leaves_other_lands() {
    for name in ["Nature's Lore", "Three Visits"] {
        let mut g = tbl();
        let lib = ["Command Tower", "Swamp", "Bayou", "Forest"].map(|n| card(&g, n)).to_vec();
        g.player_mut(P0).library = lib;
        cast_ramp(&mut g, name);
        assert_eq!(g.player(P0).lands.len(), 1, "{name}");
        let l = g.player(P0).lands[0];
        assert_eq!(&*g.perm(l).name, "Bayou", "{name}: the dual with the most colours");
        assert_eq!(g.player(P0).library.len(), 3, "{name}: Command Tower and Swamp stay");
    }
}

#[test]
fn a_fetched_land_follows_its_own_enters_rule() {
    let mut g = tbl();
    g.player_mut(P0).library = vec![card(&g, "Ketria Triome")];
    cast_ramp(&mut g, "Three Visits");
    let l = g.player(P0).lands[0];
    assert!(g.perm(l).tapped, "a Triome enters tapped");
    // a shock land: the AI pays 2 life early with life to spare
    let mut g = tbl();
    g.player_mut(P0).library = vec![card(&g, "Overgrown Tomb")];
    let life = g.player(P0).life;
    cast_ramp(&mut g, "Nature's Lore");
    let l = g.player(P0).lands[0];
    assert!(!g.perm(l).tapped);
    assert_eq!(g.player(P0).life, life - 2);
}

#[test]
fn opposition_agent_takes_the_ais_fetched_land() {
    let mut g = tbl();
    perm(&mut g, PlayerId(1), "Opposition Agent");
    g.player_mut(P0).library = vec![card(&g, "Forest")];
    cast_ramp(&mut g, "Three Visits");
    assert!(g.player(P0).lands.is_empty(), "the land is the Agent's controller's to exile");
    assert!(g.player(P0).library.is_empty());
    let q = g.player(PlayerId(1));
    assert!(q.hand.iter().any(|&c| &*g.db.get(c).name == "Forest"), "the Forest went to the Agent's controller");
}
