//! Trigger counts, life gain, and phasing for creature abilities.
use sim_core::engine::{removal, zones};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);
const PAN: &str = "Panharmonicon";
const D: &str = "Delney, Streetwise Lookout";

fn tbl() -> Game {
    table(&["miirym-temur-dragons", "seph", "sauron", "cloud"])
}

#[test]
fn prodigy_copies_departed_shamans_death_trigger() {
    let mut g = tbl();
    perm(&mut g, A, "Harmonic Prodigy");
    let s = perm(&mut g, A, "World Shaper");
    g.resolving += 1;
    zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(g.trig_queue.iter().filter(|t| t.src == Some(s)).count(), 2);
}

#[test]
fn prodigy_and_shaman_leaving_together_trigger_twice() {
    let mut counts = vec![];
    for first in [true, false] {
        let mut g = tbl();
        if first {
            perm(&mut g, A, "Harmonic Prodigy");
        }
        let s = perm(&mut g, A, "World Shaper");
        if !first {
            perm(&mut g, A, "Harmonic Prodigy");
        }
        g.resolving += 1;
        removal::apply_wipe(&mut g, A, "destroy", None, &Ctx::default()).unwrap();
        counts.push(g.trig_queue.iter().filter(|t| t.src == Some(s)).count());
    }
    assert_eq!(counts, vec![2, 2], "Prodigy before / after World Shaper");
}

#[test]
fn thundermaw_does_not_damage_or_tap_phased_out_fliers() {
    let mut g = tbl();
    let t = token(&mut g, B, 4);
    g.perm_mut(t).fly = true;
    g.perm_mut(t).phased = true;
    perm(&mut g, A, "Thundermaw Hellkite");
    assert_eq!((sim_core::human::combos::marked(&g, t), g.perm(t).tapped), (0, false));
}

#[test]
fn wispdrinker_gains_one_life_per_instance_in_multiplayer() {
    let mut g = tbl();
    perm(&mut g, A, D);
    perm(&mut g, A, PAN);
    perm(&mut g, A, "Wispdrinker Vampire");
    perm(&mut g, A, "Wispdrinker Vampire");
    let life = g.player(A).life;
    let opposing_life = g.player(B).life;
    token(&mut g, A, 1);
    assert_eq!(opposing_life - g.player(B).life, 6);
    // Two Vampires, original + Panharmonicon + Delney each. Oracle says gain 1, irrespective of opponents.
    assert_eq!(g.player(A).life - life, 6);
}

#[test]
fn wispdrinker_nontoken_gain_is_not_multiplied_by_opponent_count() {
    let mut g = tbl();
    perm(&mut g, A, "Wispdrinker Vampire");
    let before = g.player(A).life;
    perm(&mut g, A, "Llanowar Elves");
    assert_eq!(g.player(A).life - before, 1);
}

#[test]
fn wispdrinker_sees_another_wispdrinker_enter() {
    let mut g = tbl();
    perm(&mut g, A, "Wispdrinker Vampire");
    let life = g.player(B).life;
    perm(&mut g, A, "Wispdrinker Vampire");
    assert_eq!(life - g.player(B).life, 1);
}
