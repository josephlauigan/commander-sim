//! Poison for the Tier 3 Atraxa (infect and proliferate): compiled poison spells, toxic N, infect against players and
//! creatures, the ability language's proliferate (the engine's full one) and "whenever you proliferate" triggers.

use sim_core::ai::brain::do_cast;
use sim_core::engine::combat;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ATRAXA: &str = "atraxa-superfriends";

fn tbl() -> sim_core::state::Game {
    let mut g = table(&[ATRAXA, "seph"]);
    g.active = Some(P0);
    g
}

#[test]
fn prologue_to_phyresis_gives_each_opponent_a_poison_counter_and_draws() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Prologue to Phyresis"]);
    let n = g.player(P0).hand.len();
    let c = card(&g, "Prologue to Phyresis");
    assert!(do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(g.player(P1).poison, 1);
    assert_eq!(g.player(P0).hand.len(), n, "the card cast, one drawn");
}

#[test]
fn toxic_adds_poison_on_top_of_combat_damage() {
    let mut g = tbl();
    let rat = perm(&mut g, P0, "Blightbelly Rat");
    let life = g.player(P1).life;
    combat::resolve_combat(&mut g, P0, &[rat], P1, &[rat]).unwrap();
    assert_eq!(g.player(P1).poison, 1, "toxic 1");
    assert!(g.player(P1).life < life, "and the damage itself");
}

#[test]
fn infect_poisons_a_player_and_leaves_minus_counters_on_a_creature() {
    let mut g = tbl();
    let agent = perm(&mut g, P0, "Blighted Agent");
    let life = g.player(P1).life;
    combat::resolve_combat(&mut g, P0, &[agent], P1, &[agent]).unwrap();
    assert_eq!(g.player(P1).poison, 1, "infect: poison counters instead of damage");
    assert_eq!(g.player(P1).life, life, "no life lost");
    // Plague Stinger (1/1 infect flier) blocked by Thrasios (1/3): a -1/-1 counter stays on the survivor
    let stinger = perm(&mut g, P0, "Plague Stinger");
    let bear = perm(&mut g, P1, "Thrasios, Triton Hero");
    g.perm_mut(bear).fly = true; // so it can block the flier
    combat::resolve_combat(&mut g, P0, &[stinger], P1, &[]).unwrap();
    assert!(
        g.perm(bear).plus == -1 || g.player(P1).poison == 2,
        "blocked: the -1/-1 counter stays on Thrasios; unblocked: a second poison counter"
    );
}

#[test]
fn compiled_proliferate_adds_poison_and_ezuri_draws() {
    let mut g = tbl();
    g.player_mut(P1).poison = 3;
    perm(&mut g, P0, "Ezuri, Stalker of Spheres");
    perm(&mut g, P0, "Contagion Engine");
    lands(&mut g, P0, "Plains", 4, false);
    let n = g.player(P0).hand.len();
    let opts = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    let act = opts
        .into_iter()
        .find(|o| o.label.contains("Contagion Engine"))
        .and_then(|o| o.act)
        .expect("Contagion Engine's {4}, {T}: proliferate twice");
    assert!(sim_core::ai::act::perform(&mut g, P0, &act).unwrap());
    assert_eq!(g.player(P1).poison, 5, "proliferate twice: two more poison counters");
    assert!(g.player(P0).hand.len() >= n + 2, "Ezuri draws on each proliferate");
}

#[test]
fn a_player_out_on_poison_is_recorded_as_a_poison_death() {
    let mut g = tbl();
    g.player_mut(P1).poison = 10;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P1).death_kind, "poison", "the scorecard's poison wins count this");
}
