//! Kuja, Genome Sorcerer // Trance Kuja, Fate Defied (Kefka): the Wizard token, its pings, the transform at four
//! Wizards and Flare Star's doubling (impls/kuja.rs).

use sim_core::engine::{cast, combat, hooks, values};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KUJA: &str = "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied";
const KEFKA: &str = "Kefka, Court Mage // Kefka, Ruler of Ruin";
const NIV: &str = "Niv-Mizzet, the Firemind";

fn tbl() -> Game {
    let mut g = table(&["kefka", "seph", "veyran"]);
    g.active = Some(P0);
    g
}

fn end_step(g: &mut Game, p: PlayerId) {
    hooks::fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

fn kuja_tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).data.truthy(DataKey::KujaWiz)).collect()
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn trance(g: &mut Game, k: PermId) {
    g.perm_mut(k).data.set(DataKey::Flipped, sim_core::state::Val::Bool(true));
    g.perm_mut(k).pow = 4;
    g.perm_mut(k).tgh = 6;
}

#[test]
fn the_end_step_makes_a_tapped_black_zero_one_wizard() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    assert_eq!((values::epow(&g, k), values::etgh(&g, k)), (3, 4));
    end_step(&mut g, P0);
    let toks = kuja_tokens(&g, P0);
    assert_eq!(toks.len(), 1);
    let t = toks[0];
    assert!(g.perm(t).tapped, "the token enters tapped");
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (0, 1));
    assert!(values::has_type(&g, t, "wizard"));
    assert!(values::colors_of(&g, t).has('B'));
    end_step(&mut g, P1); // an opponent's end step: nothing
    assert_eq!(kuja_tokens(&g, P0).len(), 1);
    end_step(&mut g, P0);
    assert_eq!(kuja_tokens(&g, P0).len(), 2, "one more each of its controller's end steps");
}

#[test]
fn each_token_pings_every_opponent_for_a_noncreature_spell_only() {
    let mut g = tbl();
    perm(&mut g, P0, KUJA);
    end_step(&mut g, P0);
    end_step(&mut g, P0);
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    cast_free(&mut g, P0, "Night's Whisper");
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 2, l2 - 2), "two tokens, 1 damage to each opponent each");
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    cast_free(&mut g, P0, "Marvin, Murderous Mimic");
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1, l2), "a creature spell doesn't trigger them");
    // the opponent's own noncreature spell doesn't trigger P0's tokens
    cast_free(&mut g, P1, "Night's Whisper");
    assert_eq!(g.player(P1).life, l1);
}

#[test]
fn the_tokens_keep_pinging_without_kuja() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    end_step(&mut g, P0);
    sim_core::engine::zones::die(&mut g, k, "destroy").unwrap();
    let l1 = g.player(P1).life;
    cast_free(&mut g, P0, "Night's Whisper");
    assert_eq!(g.player(P1).life, l1 - 1);
}

#[test]
fn kuja_transforms_at_four_wizards_counting_kefka_and_niv() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    perm(&mut g, P0, KEFKA);
    end_step(&mut g, P0); // Kuja + Kefka + token = 3 Wizards
    assert!(!sim_core::impls::kuja::trance(&g, k));
    assert_eq!((values::epow(&g, k), values::etgh(&g, k)), (3, 4));
    perm(&mut g, P0, NIV);
    end_step(&mut g, P0); // + Niv + a second token = 5
    assert!(sim_core::impls::kuja::trance(&g, k));
    assert_eq!((values::epow(&g, k), values::etgh(&g, k)), (4, 6));
}

#[test]
fn kuja_alone_transforms_at_its_third_end_step() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    for _ in 0..2 {
        end_step(&mut g, P0);
        assert!(!sim_core::impls::kuja::trance(&g, k));
    }
    end_step(&mut g, P0); // Kuja + three tokens = four Wizards
    assert!(sim_core::impls::kuja::trance(&g, k));
    end_step(&mut g, P0);
    assert_eq!(kuja_tokens(&g, P0).len(), 3, "Trance Kuja makes no more tokens");
}

#[test]
fn flare_star_doubles_the_pings_of_wizards_you_control() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    end_step(&mut g, P0);
    end_step(&mut g, P0);
    trance(&mut g, k);
    let l1 = g.player(P1).life;
    cast_free(&mut g, P0, "Night's Whisper");
    assert_eq!(g.player(P1).life, l1 - 4, "two tokens, 2 damage each");
}

#[test]
fn flare_star_doubles_wizard_combat_damage_and_nothing_else() {
    // a Wizard attacker
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    trance(&mut g, k);
    g.perm_mut(k).sick = true; // Kuja stays home
    let w = perm(&mut g, P0, KEFKA);
    let base = g.player(P1).life + g.player(P2).life;
    combat::combat(&mut g, P0).unwrap();
    let lost = base - (g.player(P1).life + g.player(P2).life);
    assert!(lost >= 8 && lost % 8 == 0, "Kefka's 4 flying damage is doubled: lost {lost}");
    assert!(values::has_type(&g, w, "wizard"));
    // a non-Wizard attacker is not doubled
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    trance(&mut g, k);
    g.perm_mut(k).sick = true;
    token(&mut g, P0, 4);
    let base = g.player(P1).life + g.player(P2).life;
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(base - (g.player(P1).life + g.player(P2).life), 4, "a 4/4 token deals its 4");
}

#[test]
fn flare_star_doubles_niv_mizzets_draw_trigger_but_only_for_its_controller() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    trance(&mut g, k);
    perm(&mut g, P0, NIV);
    let before = g.player(P1).life + g.player(P2).life;
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    let lost = before - (g.player(P1).life + g.player(P2).life);
    assert_eq!(lost, 2, "Niv's 1 damage, doubled");
    // Niv under another player's control isn't doubled by P0's Kuja
    let mut g = tbl();
    let k = perm(&mut g, P0, KUJA);
    trance(&mut g, k);
    perm(&mut g, P1, NIV);
    let before = g.player(P0).life + g.player(P2).life;
    sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(before - (g.player(P0).life + g.player(P2).life), 1);
}

#[test]
fn kuja_is_cast_by_the_kefka_seat() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &[KUJA])[0];
    assert!(sim_core::ai::decks::deck_prio(&g, P0, c) > 0);
}
