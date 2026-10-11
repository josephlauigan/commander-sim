//! impls/zada.rs: Zada, Hedron Grinder's copy ability, the Zada spells, the spell payoffs and the deck's AI plan.

use sim_core::dsl;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, zones};
use sim_core::hooks::{Action, Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const ME: PlayerId = PlayerId(0);
const OPP: PlayerId = PlayerId(1);

/// Zada and `n` 1/1 tokens, not summoning sick, and plenty of red mana
fn board(n: usize) -> (Game, PermId, Vec<PermId>) {
    let mut g = table(&["zada", "seph"]);
    let z = perm(&mut g, ME, "Zada, Hedron Grinder");
    let toks: Vec<PermId> = (0..n).map(|_| token(&mut g, ME, 1)).collect();
    lands(&mut g, ME, "Mountain", 8, false);
    (g, z, toks)
}

/// cast a card from hand (paid for) at a target
fn cast_at(g: &mut Game, name: &str, target: Option<PermId>) {
    let c = hand(g, ME, &[name])[0];
    cast::cast_card(g, ME, c, "hand", Ctx { target, ..Ctx::default() }).unwrap();
}

fn trample(g: &Game, m: PermId) -> bool {
    dsl::has_kw(g, m, "trample")
}

#[test]
fn ancestral_anger_on_zada_draws_for_every_creature() {
    let (mut g, z, toks) = board(3);
    let lib = g.player(ME).library.len();
    cast_at(&mut g, "Ancestral Anger", Some(z));
    // the original and three copies each draw a card
    assert_eq!(g.player(ME).hand.len(), 4);
    assert_eq!(g.player(ME).library.len(), lib - 1 - 4);
    assert_eq!(epow(&g, z), 3 + 1, "+X/+0 with X = 1 (no Angers in the graveyard)");
    for m in toks {
        assert_eq!(epow(&g, m), 2);
        assert!(trample(&g, m));
    }
    assert_eq!(g.player(ME).spells_this_turn, 1, "copies are not cast");
    assert_eq!(g.player(ME).gy.len(), 1, "one card goes to the graveyard, not one per copy");
}

#[test]
fn ancestral_anger_counts_angers_in_the_graveyard() {
    let (mut g, z, _) = board(1);
    let extra = hand(&mut g, ME, &["Ancestral Anger"])[0];
    g.player_mut(ME).hand.clear();
    g.player_mut(ME).gy.push(extra);
    cast_at(&mut g, "Ancestral Anger", Some(z));
    assert_eq!(epow(&g, z), 3 + 2, "X = 1 plus the Anger already in the graveyard");
}

#[test]
fn a_spell_aimed_at_another_creature_is_not_copied() {
    let (mut g, _z, toks) = board(3);
    cast_at(&mut g, "Ancestral Anger", Some(toks[0]));
    assert_eq!(g.player(ME).hand.len(), 1);
    assert_eq!(epow(&g, toks[0]), 2);
    assert_eq!(epow(&g, toks[1]), 1);
}

#[test]
fn monstrous_rage_gives_every_creature_a_role() {
    let (mut g, z, toks) = board(3);
    cast_at(&mut g, "Monstrous Rage", Some(z));
    for m in toks.iter().chain([&z]) {
        assert!(g.perm(*m).data.truthy(DataKey::Role));
        assert!(trample(&g, *m));
    }
    assert_eq!(epow(&g, toks[0]), 1 + 2 + 1, "+2/+0 this turn and the Role's +1/+1");
    assert_eq!(etgh(&g, toks[0]), 1 + 1);
    // at end of turn the +2/+0 goes, the Role stays
    g.perm_mut(toks[0]).eot_pt = (0, 0);
    assert_eq!(epow(&g, toks[0]), 2);
    assert!(trample(&g, toks[0]));
}

#[test]
fn a_second_monster_role_replaces_the_first() {
    let (mut g, _z, toks) = board(1);
    hand(&mut g, ME, &["Monstrous Rage"]);
    cast_at(&mut g, "Monstrous Rage", Some(toks[0]));
    cast_at(&mut g, "Monstrous Rage", Some(toks[0]));
    assert_eq!(etgh(&g, toks[0]), 2, "one Role, not two");
}

#[test]
fn bulk_up_doubles_each_creatures_power() {
    let (mut g, z, toks) = board(2);
    g.perm_mut(toks[1]).pow = 3;
    cast_at(&mut g, "Bulk Up", Some(z));
    assert_eq!(epow(&g, z), 6);
    assert_eq!(epow(&g, toks[0]), 2);
    assert_eq!(epow(&g, toks[1]), 6);
}

#[test]
fn act_of_treason_on_zada_untaps_and_hastes_the_team() {
    let (mut g, z, toks) = board(3);
    for &m in toks.iter().chain([&z]) {
        let x = g.perm_mut(m);
        x.tapped = true;
        x.sick = true;
    }
    cast_at(&mut g, "Act of Treason", Some(z));
    for m in toks.iter().chain([&z]) {
        assert!(!g.perm(*m).tapped);
        assert!(sim_core::engine::combat::has_haste(&g, *m));
    }
}

#[test]
fn act_of_treason_steals_an_opposing_creature_until_end_of_turn() {
    let (mut g, _z, _) = board(1);
    let t = token(&mut g, OPP, 4);
    cast_at(&mut g, "Act of Treason", Some(t));
    assert_eq!(g.perm(t).owner, ME);
    assert!(sim_core::engine::combat::has_haste(&g, t));
}

#[test]
fn into_the_fray_forces_each_creature_to_attack() {
    let (mut g, z, toks) = board(2);
    cast_at(&mut g, "Into the Fray", Some(z));
    let now = sim_core::state::Val::Stamp(g.turn_stamp());
    for m in toks.iter().chain([&z]) {
        assert_eq!(g.perm(*m).data.get(DataKey::MustAttack), Some(&now));
    }
}

#[test]
fn blazing_crescendo_exiles_a_card_per_copy_to_play_later() {
    let (mut g, z, toks) = board(3);
    let lib = g.player(ME).library.len();
    cast_at(&mut g, "Blazing Crescendo", Some(z));
    assert_eq!(g.player(ME).library.len(), lib - 1 - 4);
    assert_eq!(g.player(ME).impulse_long.len(), 4);
    assert_eq!(g.player(ME).hand.len(), 4, "playable cards wait in hand until the end of next turn");
    assert_eq!(epow(&g, toks[0]), 1 + 3);
    assert_eq!(etgh(&g, toks[0]), 1 + 1);
}

#[test]
fn the_compiled_pump_spells_take_each_copys_target() {
    for (name, kw) in [("Sure Strike", "first strike"), ("Furious Bellow", "first strike"), ("Smaug's Fury", "reach")] {
        let (mut g, z, toks) = board(3);
        cast_at(&mut g, name, Some(z));
        for m in toks.iter().chain([&z]) {
            assert_eq!(epow(&g, *m), if *m == z { 6 } else { 4 }, "{name}");
            assert!(dsl::has_kw(&g, *m, kw), "{name}: {kw}");
        }
    }
    let (mut g, z, toks) = board(2);
    cast_at(&mut g, "Crowd's Favor", Some(z));
    assert_eq!(epow(&g, toks[0]), 2);
    assert!(dsl::has_kw(&g, toks[1], "first strike"));
}

#[test]
fn crowds_favor_can_be_paid_by_convoke() {
    let mut g = table(&["zada", "seph"]);
    let z = perm(&mut g, ME, "Zada, Hedron Grinder");
    token(&mut g, ME, 1);
    let c = hand(&mut g, ME, &["Crowd's Favor"])[0];
    let paid =
        sim_core::ai::brain::do_cast_with(&mut g, ME, c, None, Ctx { target: Some(z), ..Ctx::default() }).unwrap();
    assert!(paid, "no lands: a creature is tapped for the {{R}}");
    assert!(g.perm(z).tapped || g.player(ME).perms.iter().any(|&m| g.perm(m).tapped));
}

#[test]
fn twinflame_on_zada_copies_every_creature_with_haste() {
    let (mut g, z, toks) = board(2);
    let marshal = perm(&mut g, ME, "Mogg War Marshal");
    let before = g.player(ME).perms.len();
    cast_at(&mut g, "Twinflame", Some(z));
    let temps: Vec<PermId> = g.player(ME).perms.iter().copied().filter(|&m| g.perm(m).temp).collect();
    // a copy of each token, of the Marshal (and the Goblins its copy brings), none of Zada
    assert!(temps.len() >= 3, "{} hasty copies", temps.len());
    assert!(g.player(ME).perms.len() >= before + 3);
    assert!(temps.iter().all(|&m| !g.perm(m).sick));
    assert_eq!(
        g.player(ME).perms.iter().filter(|&&m| g.perm(m).cd == g.perm(z).cd).count(),
        1,
        "the legend rule: no second Zada"
    );
    let _ = (marshal, toks);
}

#[test]
fn return_the_favor_doubles_zadas_copies() {
    let (mut g, z, _) = board(3);
    hand(&mut g, ME, &["Return the Favor"]);
    let lib = g.player(ME).library.len();
    cast_at(&mut g, "Ancestral Anger", Some(z));
    // the original, plus three copies, twice
    assert_eq!(g.player(ME).library.len(), lib - 1 - 7);
    assert!(g.player(ME).gy.iter().any(|&c| &*g.db.get(c).name == "Return the Favor"));
    assert_eq!(g.player(ME).spells_this_turn, 2, "Return the Favor is cast; the copies are not");
}

#[test]
fn empty_the_warrens_storms() {
    let (mut g, _z, _) = board(1);
    for _ in 0..2 {
        cast_at(&mut g, "Sure Strike", None);
    }
    let before = g.player(ME).perms.iter().filter(|&&m| g.perm(m).token).count();
    cast_at(&mut g, "Empty the Warrens", None);
    let after = g.player(ME).perms.iter().filter(|&&m| g.perm(m).token).count();
    assert_eq!(after - before, 2 * 3, "storm 2: the spell and two copies, two Goblins each");
}

#[test]
fn zadas_copies_add_no_storm() {
    let (mut g, z, _) = board(3);
    cast_at(&mut g, "Sure Strike", Some(z));
    let before = g.player(ME).perms.iter().filter(|&&m| g.perm(m).token).count();
    cast_at(&mut g, "Empty the Warrens", None);
    let after = g.player(ME).perms.iter().filter(|&&m| g.perm(m).token).count();
    assert_eq!(after - before, 2 * 2, "one spell was cast before it, whatever Zada copied");
}

#[test]
fn expressive_firedancer_gets_double_strike_from_five_mana() {
    let (mut g, _z, _) = board(1);
    let f = perm(&mut g, ME, "Expressive Firedancer");
    cast_at(&mut g, "Sure Strike", None);
    assert_eq!(epow(&g, f), 2 + 1);
    assert!(!dsl::has_kw(&g, f, "double strike"), "two mana spent");
    // Bulk Up from the graveyard for {4}{R}{R}
    let c = hand(&mut g, ME, &["Bulk Up"])[0];
    g.player_mut(ME).hand.retain(|&x| x != c);
    g.player_mut(ME).gy.push(c);
    cast::cast_card(&mut g, ME, c, "gy", Ctx::default()).unwrap();
    assert!(dsl::has_kw(&g, f, "double strike"), "six mana spent");
}

#[test]
fn burning_prophet_pumps_and_scries() {
    let (mut g, _z, _) = board(1);
    let bp = perm(&mut g, ME, "Burning Prophet");
    cast_at(&mut g, "Sure Strike", None);
    assert_eq!(epow(&g, bp), 2);
    // a creature spell does nothing
    let b = epow(&g, bp);
    cast_at(&mut g, "Mogg War Marshal", None);
    assert_eq!(epow(&g, bp), b);
}

#[test]
fn speed_makes_a_hasty_creature_blockable_only_by_haste() {
    let mut g = table(&["zada", "seph"]);
    let speed = perm(&mut g, ME, "Speed, Young Avenger");
    perm(&mut g, ME, "Burning Prophet");
    lands(&mut g, ME, "Mountain", 4, false);
    let blocker = token(&mut g, OPP, 2);
    g.active = Some(ME);
    cast_at(&mut g, "Sure Strike", Some(speed));
    assert!(!sim_core::engine::combat::can_block(&g, blocker, speed), "a creature without haste can't block it");
    g.perm_mut(blocker).eot_kw.push("haste");
    assert!(sim_core::engine::combat::can_block(&g, blocker, speed), "a creature with haste can");
}

#[test]
fn dreadhorde_arcanist_recasts_a_spell_for_free_and_exiles_it() {
    let (mut g, z, toks) = board(3);
    let arc = perm(&mut g, ME, "Dreadhorde Arcanist");
    let anger = hand(&mut g, ME, &["Ancestral Anger"])[0];
    g.player_mut(ME).hand.retain(|&x| x != anger);
    g.player_mut(ME).gy.push(anger);
    let lib = g.player(ME).library.len();
    let mana = sim_core::engine::mana::total_mana(&g, ME, false);
    g.perm_mut(arc).tapped = true;
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: ME, atk: vec![arc], d: OPP }).unwrap();
    assert_eq!(sim_core::engine::mana::total_mana(&g, ME, false), mana, "free");
    assert!(g.player(ME).exile.contains(&anger), "exiled, not in the graveyard");
    assert!(!g.player(ME).gy.contains(&anger));
    // cast with Zada as its target: a card for him and each of the four others
    assert_eq!(g.player(ME).library.len(), lib - 5);
    assert!(trample(&g, z) && trample(&g, toks[0]) && trample(&g, arc));
}

#[test]
fn firebird_pumps_the_other_attackers() {
    let (mut g, _z, toks) = board(2);
    let fb = perm(&mut g, ME, "Firebird, Blazing Ranger");
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: ME, atk: vec![fb, toks[0], toks[1]], d: OPP }).unwrap();
    assert_eq!(epow(&g, toks[0]), 1 + 1);
    assert_eq!(epow(&g, toks[1]), 1 + 1);
    assert_eq!(epow(&g, fb), 1);
}

#[test]
fn the_ai_holds_zada_spells_until_zada_has_company() {
    let mut g = table(&["zada", "seph"]);
    perm(&mut g, ME, "Zada, Hedron Grinder");
    token(&mut g, ME, 1);
    lands(&mut g, ME, "Mountain", 6, false);
    hand(&mut g, ME, &["Ancestral Anger", "Monstrous Rage", "Mogg War Marshal"]);
    let labels = |g: &Game| -> Vec<String> {
        sim_core::impls::zada::zada_options(g, ME, false).into_iter().map(|o| o.label).collect()
    };
    assert!(labels(&g).is_empty(), "Zada and one creature: hold the spells ({:?})", labels(&g));
    token(&mut g, ME, 1);
    let l = labels(&g);
    assert!(l.iter().any(|x| x.starts_with("Ancestral Anger on Zada")), "{l:?}");
    assert!(l.iter().any(|x| x.starts_with("Monstrous Rage on Zada")), "{l:?}");
}

#[test]
fn the_ai_casts_a_zada_spell_on_zada_and_the_pump_when_attacking() {
    let (mut g, z, toks) = board(3);
    hand(&mut g, ME, &["Ancestral Anger", "Sure Strike"]);
    let opts = sim_core::impls::zada::zada_options(&g, ME, false);
    assert_eq!(
        opts.len(),
        1,
        "only the draw spell waits for main 1: {:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    let Some(Action::Plan { f, arg }) = opts[0].act.clone() else { panic!("a plan") };
    assert!(f(&mut g, ME, arg).unwrap());
    assert_eq!(g.player(ME).hand.len(), 1 + 4, "Sure Strike and four draws");
    // as the attack is declared, Sure Strike goes on Zada and is copied
    let mut atk = toks.clone();
    atk.push(z);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: ME, atk, d: OPP }).unwrap();
    assert!(dsl::has_kw(&g, toks[2], "first strike"));
    assert_eq!(epow(&g, toks[2]), 1 + 1 + 3, "Anger's +1 and Sure Strike's +3");
}

#[test]
fn a_cantrip_is_cast_on_a_creature_when_nothing_better_is_in_hand() {
    let mut g = table(&["zada", "seph"]);
    token(&mut g, ME, 1);
    lands(&mut g, ME, "Mountain", 2, false);
    hand(&mut g, ME, &["Ancestral Anger"]);
    let opts = sim_core::impls::zada::zada_options(&g, ME, false);
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.contains("cantrip"));
}

#[test]
fn zada_decks_play_whole_games() {
    for seed in 0..3 {
        let g = play(&["zada", "seph", "veyran", "sauron"], 700 + seed, false);
        assert!(g.round > 1);
    }
    let _ = zones::TOKEN_CAP;
}
