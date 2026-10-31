//! Card models, part 3 (10-31): the pool's last Partial and Unmodeled cards. Each test checks the card's Oracle
//! behaviour (data/scryfall_cache.json), and the AI's use of it where that matters.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::card_models3 as cm3;
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn t6(first: &str) -> Game {
    let others: Vec<&str> = [
        "rograkh-thrasios-creature-combo",
        "sisay-legends-toolbox",
        "kinnan-bonder-mana-combo",
        "dack-fayden-white-stax",
    ]
    .into_iter()
    .filter(|&k| k != first)
    .take(2)
    .collect();
    let mut g = table(&[first, others[0], others[1]]);
    g.active = Some(P0);
    g
}

fn cast(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> bool {
    let c = hand(g, p, &[name])[0];
    sim_core::engine::cast::cast_card(g, p, c, "hand", ctx).unwrap()
}

// ======================================================== Hidden Strings: cipher
const STRINGS: &str = "Hidden Strings";

#[test]
fn hidden_strings_is_encoded_on_a_creature_and_cast_again_on_combat_damage() {
    let mut g = t6("rograkh-thrasios-creature-combo");
    let small = token(&mut g, P0, 2);
    let big = token(&mut g, P0, 4);
    let ls = lands(&mut g, P0, "Forest", 3, true);
    assert!(cast(&mut g, P0, STRINGS, Ctx::default()));
    let c = card(&g, STRINGS);
    assert_eq!(ls.iter().filter(|&&l| !g.perm(l).tapped).count(), 2, "it untaps two of the tapped lands");
    assert!(g.player(P0).exile.contains(&c) && !g.player(P0).gy.contains(&c), "encoded: in exile");
    assert!(g.perm(big).data.get(DataKey::Cipher).is_some(), "on the creature likeliest to hit (power 4)");
    assert!(g.perm(small).data.get(DataKey::Cipher).is_none());
    // the encoded creature deals combat damage to a player: a free copy
    for &l in &ls {
        g.perm_mut(l).tapped = true;
    }
    let spells = g.player(P0).spells_this_turn;
    sim_core::cardcode::combat_damage_cards(&mut g, P0, big, P1, 4).unwrap();
    assert_eq!(ls.iter().filter(|&&l| !g.perm(l).tapped).count(), 2, "the copy untaps two lands");
    assert_eq!(g.player(P0).spells_this_turn, spells + 1, "the copy is cast");
    assert_eq!(g.player(P0).exile.iter().filter(|&&x| x == c).count(), 1, "the card stays encoded, once");
    assert!(!g.player(P0).gy.contains(&c));
    // another creature's damage casts nothing
    for &l in &ls {
        g.perm_mut(l).tapped = true;
    }
    sim_core::cardcode::combat_damage_cards(&mut g, P0, small, P1, 2).unwrap();
    assert!(ls.iter().all(|&l| g.perm(l).tapped));
}

#[test]
fn hidden_strings_with_no_creature_goes_to_the_graveyard() {
    let mut g = t6("rograkh-thrasios-creature-combo");
    lands(&mut g, P0, "Forest", 3, true);
    assert!(cast(&mut g, P0, STRINGS, Ctx::default()));
    let c = card(&g, STRINGS);
    assert!(g.player(P0).gy.contains(&c) && !g.player(P0).exile.contains(&c));
}

// ======================================================== Invasion of Ikoria: a battle
const IKORIA: &str = "Invasion of Ikoria // Zilortha, Apex of Ikoria";

fn battle_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && cm3::defense(g, m) > 0)
}

#[test]
fn invasion_of_ikoria_stays_as_a_battle_that_an_opponent_protects() {
    let mut g = t6("kinnan-bonder-mana-combo");
    // P1 has two creatures, P2 none: P2 protects it
    token(&mut g, P1, 2);
    token(&mut g, P1, 2);
    assert!(cast(&mut g, P0, IKORIA, Ctx { x: 2, ..Ctx::default() }));
    let c = card(&g, IKORIA);
    let w = battle_of(&g, P0).expect("the Siege is on the battlefield");
    assert_eq!(g.perm(w).cd, Some(c));
    assert_eq!(cm3::defense(&g, w), 6);
    assert!(!g.is_creature(w));
    assert_eq!(cm3::protector(&g, w), Some(PlayerId(2)), "the opponent with the fewest blockers");
    assert!(!g.player(P0).gy.contains(&c));
    assert!(cm3::battles_for(&g, P0, PlayerId(2)) == vec![w] && cm3::battles_for(&g, P0, P1).is_empty());
}

#[test]
fn attacking_the_battle_removes_defense_and_defeating_it_casts_zilortha() {
    let mut g = t6("kinnan-bonder-mana-combo");
    token(&mut g, P1, 2);
    assert!(cast(&mut g, P0, IKORIA, Ctx { x: 0, ..Ctx::default() }));
    let w = battle_of(&g, P0).unwrap();
    let d = cm3::protector(&g, w).unwrap();
    let a = token(&mut g, P0, 4);
    let life = g.player(d).life;
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(&mut g, P0, vec![a], d, vec![], &mut tot).unwrap();
    assert_eq!(cm3::defense(&g, w), 2, "4 damage to the battle");
    assert_eq!(g.player(d).life, life, "none to its protector");
    let b = token(&mut g, P0, 3);
    sim_core::engine::combat::resolve_blocks(&mut g, P0, vec![b], d, vec![], &mut tot).unwrap();
    assert!(!g.perm(w).on_bf, "defeated");
    let z = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && &*g.perm(m).name == cm3::ZILORTHA)
        .expect("Zilortha is cast transformed");
    assert!(g.is_creature(z));
    assert_eq!((g.perm(z).pow, g.perm(z).tgh), (8, 8));
    assert!(sim_core::dsl::has_kw(&g, z, "reach"));
    assert!(!g.player(P0).exile.contains(&card(&g, IKORIA)));
    assert_eq!(g.player(d).life, life, "the damage past the last counter is lost, not dealt to the player");
}

#[test]
fn zilortha_lets_non_humans_deal_their_damage_as_though_unblocked() {
    let mut g = t6("kinnan-bonder-mana-combo");
    token(&mut g, P1, 2);
    assert!(cast(&mut g, P0, IKORIA, Ctx { x: 0, ..Ctx::default() }));
    let w = battle_of(&g, P0).unwrap();
    g.perm_mut(w).data.set(DataKey::Defense, sim_core::state::Val::Int(1));
    let a = token(&mut g, P0, 1);
    let d = cm3::protector(&g, w).unwrap();
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(&mut g, P0, vec![a], d, vec![], &mut tot).unwrap();
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == cm3::ZILORTHA));
    // a 3/3 blocked by a 0/4 wall-like token: all 3 to the player
    let atk = token(&mut g, P0, 3);
    let blk = token(&mut g, d, 1);
    g.perm_mut(blk).tgh = 4;
    let life = g.player(d).life;
    sim_core::engine::combat::resolve_blocks(&mut g, P0, vec![atk], d, vec![(atk, blk)], &mut tot).unwrap();
    assert_eq!(g.player(d).life, life - 3);
    assert!(g.perm(blk).on_bf);
}

// ======================================================== Magda, Brazen Outlaw: becomes tapped
const MAGDA: &str = "Magda, Brazen Outlaw";

#[test]
fn magda_makes_a_treasure_whenever_a_dwarf_becomes_tapped_by_any_means() {
    let mut g = t6("sisay-legends-toolbox");
    let magda = perm(&mut g, P0, MAGDA);
    let tataru = perm(&mut g, P0, "Tataru Taru");
    sim_core::engine::life::check_state(&mut g).unwrap();
    let t0 = g.player(P0).treasures;
    g.perm_mut(tataru).tapped = true; // tapped by an effect (an opponent's tapper, a cost)
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1, "Tataru became tapped");
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1, "once: it stays tapped");
    g.perm_mut(tataru).tapped = false;
    sim_core::engine::life::check_state(&mut g).unwrap();
    g.perm_mut(tataru).tapped = true;
    g.perm_mut(magda).tapped = true;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 3, "again after untapping, and Magda herself is a Dwarf");
}

#[test]
fn magda_ignores_a_dwarf_that_entered_tapped_and_non_dwarves() {
    let mut g = t6("sisay-legends-toolbox");
    perm(&mut g, P0, MAGDA);
    let other = token(&mut g, P0, 2);
    let c = take(&mut g, P0, "Tataru Taru");
    let how = sim_core::engine::zones::Enter { sick: false, ..Default::default() };
    let tataru = sim_core::engine::zones::enter(&mut g, P0, c, how).unwrap();
    g.perm_mut(tataru).tapped = true; // as if it entered tapped: no state check saw it untapped
    let t0 = g.player(P0).treasures;
    g.perm_mut(other).tapped = true;
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, t0);
}

#[test]
fn magdas_attack_trigger_is_not_counted_twice_by_the_state_check() {
    let mut g = t6("sisay-legends-toolbox");
    let magda = perm(&mut g, P0, MAGDA);
    sim_core::engine::life::check_state(&mut g).unwrap();
    let t0 = g.player(P0).treasures;
    g.perm_mut(magda).tapped = true;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[magda], P1).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
}
