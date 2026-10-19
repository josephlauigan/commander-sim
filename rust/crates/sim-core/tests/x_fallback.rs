//! x-fallback: compiled "X" amounts (a counting phrase, or "that many") used to run as the stand-in 3 on cards that
//! never choose an X. Each card here must use its real number; the last test plays whole games and asserts that no
//! compiled amount fell back to 3 on a JD deck's cards.

use sim_core::dsl;
use sim_core::engine::cast;
use sim_core::engine::values::{epow, etgh};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn fill_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

fn attach_aura(g: &mut Game, host: PermId, name: &str) {
    let a = perm(g, P0, name);
    g.perm_mut(a).attached = Some(host);
    if !g.auras.contains(&a) {
        g.auras.push(a);
    }
}

#[test]
fn adelbert_steiner_gets_plus_one_for_each_equipment_you_control() {
    let mut g = table(&["cloud", "kefka"]);
    let a = perm(&mut g, P0, "Adelbert Steiner");
    let (p0, t0) = (epow(&g, a), etgh(&g, a));
    perm(&mut g, P0, "Colossus Hammer");
    perm(&mut g, P0, "Sol Ring"); // not an Equipment
    assert_eq!((epow(&g, a), etgh(&g, a)), (p0 + 1, t0 + 1));
    perm(&mut g, P1, "Colossus Hammer"); // an opponent's does not count
    assert_eq!((epow(&g, a), etgh(&g, a)), (p0 + 1, t0 + 1));
    assert_eq!((p0, t0), (2, 1), "Adelbert is a printed 2/1");
}

#[test]
fn kor_spiritdancer_gets_plus_two_for_each_aura_on_it_and_no_more() {
    let mut g = table(&["light-paws-aura-voltron", "seph"]);
    let k = perm(&mut g, P0, "Kor Spiritdancer");
    assert_eq!((epow(&g, k), etgh(&g, k)), (0, 2), "0/2 with no Aura");
    attach_aura(&mut g, k, "Flickering Ward");
    assert_eq!((epow(&g, k), etgh(&g, k)), (2, 4), "+2/+2 for the Aura on it");
}

#[test]
fn haughty_djinn_power_counts_instants_and_sorceries_in_your_graveyard_toughness_stays() {
    let mut g = table(&["stella-lee-quick-draw", "seph"]);
    let d = perm(&mut g, P0, "Haughty Djinn");
    let t0 = etgh(&g, d);
    assert_eq!(epow(&g, d), 0);
    fill_gy(&mut g, P0, &["Opt", "Ponder", "Island"]);
    assert_eq!((epow(&g, d), etgh(&g, d)), (2, t0), "two instants/sorceries; toughness is printed 4");
    assert_eq!(t0, 4);
}

#[test]
fn deadly_embrace_draws_for_each_creature_that_died_this_turn() {
    let mut g = table(&["kefka", "seph"]);
    token(&mut g, P1, 1);
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Deadly Embrace");
    assert_eq!(g.player(P0).hand.len(), n + 1, "only the destroyed creature died");
    let mut g = table(&["kefka", "seph"]);
    token(&mut g, P1, 1);
    let mine = token(&mut g, P0, 1);
    sim_core::engine::zones::die(&mut g, mine, "destroy").unwrap();
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Deadly Embrace");
    assert_eq!(g.player(P0).hand.len(), n + 2, "an earlier death counts too");
    let mut g = table(&["kefka", "seph"]);
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Deadly Embrace");
    assert_eq!(g.player(P0).hand.len(), n, "no creature died: draws nothing");
}

#[test]
fn fumigate_gains_one_life_for_each_creature_destroyed() {
    let mut g = table(&["kefka", "seph"]);
    for _ in 0..2 {
        token(&mut g, P0, 1);
    }
    for _ in 0..3 {
        token(&mut g, P1, 1);
    }
    let life = g.player(P0).life;
    cast_spell(&mut g, P0, "Fumigate");
    assert_eq!(g.player(P0).life, life + 5);
    let mut g = table(&["kefka", "seph"]);
    let life = g.player(P0).life;
    cast_spell(&mut g, P0, "Fumigate");
    assert_eq!(g.player(P0).life, life, "nothing destroyed, nothing gained");
}

#[test]
fn gandalfs_sanction_deals_damage_equal_to_the_instants_and_sorceries_in_your_graveyard() {
    let mut g = table(&["kefka", "seph"]);
    let t = token(&mut g, P1, 1);
    fill_gy(&mut g, P0, &["Lightning Bolt", "Duress"]);
    let c = hand(&mut g, P0, &["Gandalf's Sanction"])[0];
    let ctx = Ctx { target: Some(t), ..Ctx::default() };
    let _ = cast::cast_card(&mut g, P0, c, "hand", ctx);
    assert!(!g.perm(t).on_bf, "2 damage kills a 1/1");
    let mut g = table(&["kefka", "seph"]);
    let big = token(&mut g, P1, 4);
    fill_gy(&mut g, P0, &["Lightning Bolt", "Duress"]);
    let c = hand(&mut g, P0, &["Gandalf's Sanction"])[0];
    let ctx = Ctx { target: Some(big), ..Ctx::default() };
    let _ = cast::cast_card(&mut g, P0, c, "hand", ctx);
    assert!(g.perm(big).on_bf, "2 damage does not kill a 4/4 (it used to deal 3)");
}

#[test]
fn shimmercreep_drains_for_each_color_among_your_permanents() {
    let mut g = table(&["kefka", "seph"]);
    let (me, opp) = (g.player(P0).life, g.player(P1).life);
    perm(&mut g, P0, "Shimmercreep"); // black only, so one colour
    assert_eq!((g.player(P0).life - me, opp - g.player(P1).life), (1, 1));
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Sol Ring"); // colourless adds none
    perm(&mut g, P0, "Llanowar Elves"); // green
    let (me, opp) = (g.player(P0).life, g.player(P1).life);
    perm(&mut g, P0, "Shimmercreep");
    assert_eq!((g.player(P0).life - me, opp - g.player(P1).life), (2, 2), "black + green");
}

#[test]
fn demon_of_fates_design_pumps_by_the_sacrificed_enchantments_mana_value() {
    let mut g = table(&["anikthea-enduring-enchantments", "seph"]);
    let d = perm(&mut g, P0, "Demon of Fate's Design");
    perm(&mut g, P0, "Font of Fertility"); // mana value 1
    lands(&mut g, P0, "Swamp", 3, false);
    let p0 = epow(&g, d);
    let idx = {
        let cd = g.perm(d).cd.unwrap();
        g.db.get(cd)
            .abilities
            .iter()
            .position(|a| matches!(a, sim_core::dsl::model::Ability::Activated { .. }))
            .unwrap()
    };
    assert!(dsl::activate(&mut g, P0, d, idx as u16).unwrap());
    assert_eq!(epow(&g, d), p0 + 1, "+1/+0: the sacrificed Font of Fertility has mana value 1 (it used to be +3)");
}

/// The hole: nothing in a whole game of the JD decks reads a compiled "X" with no X chosen (it would run as 3).
#[test]
fn no_compiled_x_falls_back_to_three_in_the_jd_decks_games() {
    dsl::x_fallbacks();
    for (keys, seed) in [(["kefka", "cloud", "sephfs", "omnath"], 11u64), (["cloud", "kefka", "veyran", "seph"], 12)] {
        for s in 0..3 {
            let _ = play(&keys, seed + s, false);
        }
    }
    assert_eq!(dsl::x_fallbacks(), 0, "a compiled \"X\" amount ran as the stand-in 3");
}

/// The counter itself works: Grand Crescendo (a Tier 0 card still waiting for the Zurgo agent's code) makes its X
/// tokens from the stand-in 3.
#[test]
fn the_fallback_counter_sees_a_card_that_still_runs_on_the_stand_in() {
    let mut g = table(&["zurgo-mardu-surge", "kefka"]);
    dsl::x_fallbacks();
    cast_spell(&mut g, P0, "Grand Crescendo");
    assert!(dsl::x_fallbacks() > 0);
}
