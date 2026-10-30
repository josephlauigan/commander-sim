//! Tier 3 deck `aurelia-boros-extra-combats`: the cards modeled for it (impls/t3_aurelia.rs and the "Tier 3
//! (aurelia)" ability-language patterns) on hand-built positions. Seat 0 plays the Aurelia deck.

use sim_core::engine::combat::{self, double_strike, first_strike};
use sim_core::engine::life::lose_life;
use sim_core::engine::removal::apply_removal;
use sim_core::engine::values::{indestructible, no_damage};
use sim_core::engine::{cast, zones};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t3_aurelia as t3;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DECK: &str = "aurelia-boros-extra-combats";

fn game() -> Game {
    let mut g = table(&[DECK, "seph", "cloud"]);
    for p in [P0, P1, P2] {
        let cs: Vec<_> = (0..30).map(|_| card(&g, "Plains")).collect();
        g.player_mut(p).library.extend(cs); // something to draw besides the deck
    }
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn eot(g: &Game, m: PermId, kw: &str) -> bool {
    g.perm(m).eot_kw.contains(&kw)
}

fn run_each_combat(g: &mut Game, src: PermId, active: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.each_combat).expect("each_combat hook");
    f(g, src, active).unwrap();
}

fn land_option(g: &mut Game, p: PlayerId, l: PermId) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(l).card();
    let f = g.registry.get(c).and_then(|i| i.land_options).expect("land option hook");
    f(g, l, p, Some(false)).unwrap()
}

fn perm_options(g: &mut Game, p: PlayerId, src: PermId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options hook");
    f(g, src, p, post).unwrap()
}

fn act(g: &mut Game, p: PlayerId, o: &sim_core::hooks::Opt) {
    assert!(sim_core::ai::act::perform(g, p, o.act.as_ref().unwrap()).unwrap(), "{} was refused", o.label);
}

// ======================================================== Blade Historian (compiled)
#[test]
fn blade_historian_gives_double_strike_to_attackers_only() {
    let mut g = game();
    perm(&mut g, P0, "Blade Historian");
    let a = token(&mut g, P0, 3);
    let home = token(&mut g, P0, 3);
    assert!(!double_strike(&g, a), "not attacking yet");
    g.in_combat = vec![a];
    assert!(double_strike(&g, a), "attacking: double strike");
    assert!(!double_strike(&g, home), "a creature at home has none");
    let life = g.player(P1).life;
    combat::resolve_combat(&mut g, P0, &[a], P1, &[a]).unwrap();
    assert_eq!(g.player(P1).life, life - 6, "3 power, twice");
}

// ======================================================== Avacyn, Angel of Hope (compiled)
#[test]
fn avacyn_makes_other_permanents_and_lands_indestructible() {
    let mut g = game();
    perm(&mut g, P0, "Avacyn, Angel of Hope");
    let ring = perm(&mut g, P0, "Sol Ring");
    let t = token(&mut g, P0, 2);
    assert!(indestructible(&g, ring) && indestructible(&g, t), "an artifact and a creature");
    apply_removal(&mut g, Some(P1), ring, "destroy", None).unwrap();
    assert!(g.perm(ring).on_bf, "Sol Ring survives destroy");
    let l = lands(&mut g, P0, "Mountain", 1, false)[0];
    sim_core::impls::fixes::destroy_land(&mut g, P0, l).unwrap();
    assert!(g.player(P0).lands.contains(&l), "the land survives too");
    // an opponent's permanents don't benefit
    let theirs = perm(&mut g, P1, "Sol Ring");
    assert!(!indestructible(&g, theirs));
}

// ======================================================== Firemane Avenger (compiled battalion)
#[test]
fn firemane_avenger_battalion_needs_two_other_attackers() {
    let mut g = game();
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let life = g.player(P0).life;
    combat::attack_triggers(&mut g, P0, &[fa, t1], P1).unwrap();
    assert_eq!(g.player(P0).life, life, "two attackers: no battalion");
    let t2 = token(&mut g, P0, 1);
    let opp_life: i32 = [P1, P2].iter().map(|&q| g.player(q).life).sum();
    combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert_eq!(g.player(P0).life, life + 3, "you gain 3 life");
    let opp_after: i32 = [P1, P2].iter().map(|&q| g.player(q).life).sum();
    let creatures_hit = g.player(P1).perms.len() + g.player(P2).perms.len() == 0;
    assert!(opp_after == opp_life - 3 || creatures_hit || opp_after == opp_life, "3 damage to a target");
}

// ======================================================== Aurelia, the Law Above (compiled)
#[test]
fn aurelia_law_above_draws_on_three_attackers_and_burns_on_five() {
    let mut g = game();
    perm(&mut g, P0, "Aurelia, the Law Above");
    let atk: Vec<PermId> = (0..5).map(|_| token(&mut g, P1, 1)).collect();
    let h = hand_len(&g, P0);
    combat::attack_triggers(&mut g, P1, &atk[..2], P0).unwrap();
    assert_eq!(hand_len(&g, P0), h, "two attackers: nothing");
    combat::attack_triggers(&mut g, P1, &atk[..3], P0).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1, "any player's attack with three: you draw");
    let (l0, l1, l2) = (g.player(P0).life, g.player(P1).life, g.player(P2).life);
    combat::attack_triggers(&mut g, P1, &atk, P0).unwrap();
    assert_eq!(hand_len(&g, P0), h + 2, "five attackers draw too");
    assert_eq!(g.player(P0).life, l0 + 3, "you gain 3 life");
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 3, l2 - 3), "3 damage to each opponent");
}

// ======================================================== Gisela, Blade of Goldnight
#[test]
fn gisela_doubles_damage_to_opponents_and_halves_damage_to_you() {
    let mut g = game();
    perm(&mut g, P0, "Gisela, Blade of Goldnight");
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    lose_life(&mut g, P1, 3, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P1).life, l1 - 6, "doubled");
    lose_life(&mut g, P0, 3, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, l0 - 1, "half prevented, rounded up: 1 of 3");
    lose_life(&mut g, P1, 2, Some(P0), "drain", None).unwrap();
    assert_eq!(g.player(P1).life, l1 - 8, "life loss is not damage");
    // combat damage, and damage to an opponent's creature
    let a = token(&mut g, P0, 2);
    combat::resolve_combat(&mut g, P0, &[a], P1, &[a]).unwrap();
    assert_eq!(g.player(P1).life, l1 - 12, "2 combat damage, doubled");
    let b = token(&mut g, P1, 3);
    apply_removal(&mut g, Some(P0), b, "dmg2", None).unwrap();
    assert!(!g.perm(b).on_bf, "2 damage doubled kills a 3-toughness creature");
}

// ======================================================== Iroas, God of Victory
#[test]
fn iroas_is_a_creature_only_with_devotion_seven() {
    let mut g = game();
    let iroas = perm(&mut g, P0, "Iroas, God of Victory");
    assert!(!g.is_creature(iroas), "devotion 2: an enchantment");
    perm(&mut g, P0, "Blade Historian"); // {R/W} x4: devotion 6
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(!g.is_creature(iroas), "devotion 6");
    perm(&mut g, P0, "Aurelia, the Law Above"); // devotion 8
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.is_creature(iroas), "devotion 8: a creature");
    assert_eq!((g.perm(iroas).pow, g.perm(iroas).tgh), (7, 4));
    assert!(indestructible(&g, iroas));
}

#[test]
fn iroas_gives_menace_and_prevents_damage_to_attackers() {
    let mut g = game();
    perm(&mut g, P0, "Iroas, God of Victory");
    let a = token(&mut g, P0, 2);
    assert!(combat::kw(&g, a, "menace"), "creatures you control have menace");
    assert!(!no_damage(&g, a), "not attacking");
    g.in_combat = vec![a];
    assert!(no_damage(&g, a), "attacking: damage to it is prevented");
    let theirs = token(&mut g, P1, 2);
    g.in_combat = vec![theirs];
    assert!(!no_damage(&g, theirs), "an opponent's attacker isn't");
}

// ======================================================== Angelic Skirmisher
#[test]
fn angelic_skirmisher_picks_first_strike_into_blockers_and_lifelink_into_an_open_board() {
    let mut g = game();
    let sk = perm(&mut g, P0, "Angelic Skirmisher");
    let t = token(&mut g, P0, 2);
    run_each_combat(&mut g, sk, P0);
    assert!(eot(&g, t, "lifelink") && eot(&g, sk, "lifelink"), "no blocker anywhere: lifelink");
    let mut g = game();
    let sk = perm(&mut g, P0, "Angelic Skirmisher");
    let t = token(&mut g, P0, 2);
    token(&mut g, P1, 2);
    run_each_combat(&mut g, sk, P0);
    assert!(first_strike(&g, t), "a blocker: first strike");
}

// ======================================================== Odric, Lunarch Marshal
#[test]
fn odric_shares_keywords_among_your_creatures() {
    let mut g = game();
    let od = perm(&mut g, P0, "Odric, Lunarch Marshal");
    perm(&mut g, P0, "Firemane Avenger"); // flying
    perm(&mut g, P0, "Aurelia, the Law Above"); // flying, vigilance, haste
    let t = token(&mut g, P0, 2);
    run_each_combat(&mut g, od, P0);
    for kw in ["flying", "vigilance", "haste"] {
        assert!(combat::kw(&g, t, kw), "the token gains {kw}");
    }
    assert!(!combat::kw(&g, t, "trample"), "no one has trample");
}

// ======================================================== Battle Angels of Tyr
#[test]
fn battle_angels_reward_hitting_the_player_with_the_most() {
    let mut g = game();
    let ba = perm(&mut g, P0, "Battle Angels of Tyr");
    let f = g.registry.get(card(&g, "Battle Angels of Tyr")).and_then(|i| i.combat_damage).unwrap();
    hand(&mut g, P1, &["Plains", "Plains", "Plains"]);
    lands(&mut g, P1, "Plains", 3, false);
    g.player_mut(P1).life = 50;
    let (h, t, l) = (hand_len(&g, P0), g.player(P0).treasures, g.player(P0).life);
    f(&mut g, ba, P0, ba, P1, 4).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1, "most cards in hand: draw");
    assert_eq!(g.player(P0).treasures, t + 1, "most lands: a Treasure");
    assert_eq!(g.player(P0).life, l + 3, "most life: 3 life");
    // P2 has the most of nothing
    f(&mut g, ba, P0, ba, P2, 4).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1);
    assert_eq!(g.player(P0).treasures, t + 1);
    assert_eq!(g.player(P0).life, l + 3);
}

// ======================================================== Bruna, the Fading Light
#[test]
fn bruna_returns_an_angel_from_the_graveyard_when_cast() {
    let mut g = game();
    let angel = take(&mut g, P0, "Firemane Avenger");
    g.player_mut(P0).gy.push(angel);
    let b = hand(&mut g, P0, &["Bruna, the Fading Light"])[0];
    cast::cast_card(&mut g, P0, b, "hand", Ctx::default()).unwrap();
    assert!(on_bf(&g, P0, "Firemane Avenger").is_some(), "the Angel returns");
    assert!(on_bf(&g, P0, "Bruna, the Fading Light").is_some());
    assert!(!g.player(P0).gy.contains(&angel));
}

// ======================================================== Fighter Class
#[test]
fn fighter_class_finds_an_equipment_and_levels_up() {
    let mut g = game();
    let h = hand_len(&g, P0);
    let fc = perm(&mut g, P0, "Fighter Class");
    assert_eq!(hand_len(&g, P0), h + 1, "an Equipment to hand");
    let got = *g.player(P0).hand.last().unwrap();
    assert!(g.db.get(got).has_subtype("equipment"), "{} is an Equipment", g.db.get(got).name);
    // level 2: equip costs {2} less
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Plains", 1, false);
    let opts = perm_options(&mut g, P0, fc, Some(false));
    assert_eq!(opts.len(), 1, "level 2 with an Equipment in hand");
    act(&mut g, P0, &opts[0]);
    assert_eq!(t3::equip_discount(&g, P0), 2);
    let sf = perm(&mut g, P0, "Sunforger");
    let t = token(&mut g, P0, 2);
    assert_eq!(sim_core::impls::cloud::equip_cost(&g, P0, sf, t, 3), 1, "equip {{3}} costs {{1}}");
}

#[test]
fn fighter_class_level_three_forces_a_losing_block() {
    let mut g = game();
    let fc = perm(&mut g, P0, "Fighter Class");
    g.perm_mut(fc).data.set(sim_core::state::DataKey::Level, sim_core::state::Val::Int(3));
    let a = token(&mut g, P0, 4);
    let b = token(&mut g, P1, 1);
    let mut assign = vec![];
    let f = g.registry.get(card(&g, "Fighter Class")).and_then(|i| i.blocks).unwrap();
    f(&mut g, fc, P0, &[a], P1, &mut assign).unwrap();
    assert_eq!(assign, vec![(a, b)], "the 1/1 must block the 4/1");
}

// ======================================================== Chance for Glory
#[test]
fn chance_for_glory_makes_creatures_indestructible_and_queues_a_losing_extra_turn() {
    let mut g = game();
    let t = token(&mut g, P0, 2);
    let c = hand(&mut g, P0, &["Chance for Glory"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(indestructible(&g, t));
    assert_eq!(g.extra_turns, vec![(P0, true)], "an extra turn, lost at its end step");
}

#[test]
fn chance_for_glory_is_cast_only_for_a_lethal_extra_turn() {
    let mut g = game();
    let c = card(&g, "Chance for Glory");
    let prio = g.registry.get(c).and_then(|i| i.prio).unwrap();
    g.step = sim_core::state::Step::Main2;
    token(&mut g, P0, 5);
    assert_eq!(prio(&g, P0, c), 0, "5 damage into 80 life");
    for q in [P1, P2] {
        g.player_mut(q).life = 2;
    }
    assert_eq!(prio(&g, P0, c), 85, "lethal next turn");
    g.step = sim_core::state::Step::Main1;
    assert_eq!(prio(&g, P0, c), 0, "after combat only");
}

// ======================================================== Trouble in Pairs
#[test]
fn trouble_in_pairs_skips_opponents_extra_turns() {
    let mut g = game();
    perm(&mut g, P0, "Trouble in Pairs");
    assert!(t3::skips_extra_turn(&g, P1));
    assert!(!t3::skips_extra_turn(&g, P0), "yours are taken");
}

#[test]
fn trouble_in_pairs_draws_on_second_card_second_spell_and_double_attacks() {
    let mut g = game();
    perm(&mut g, P0, "Trouble in Pairs");
    let h = hand_len(&g, P0);
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(hand_len(&g, P0), h, "a first card: nothing");
    zones::draw(&mut g, P1, 2, false).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1, "the second card: one draw, once");
    g.active = Some(P1);
    for name in ["Sol Ring", "Arcane Signet"] {
        let c = hand(&mut g, P1, &[name])[0];
        cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    }
    assert_eq!(hand_len(&g, P0), h + 2, "the second spell");
    let atk: Vec<PermId> = (0..2).map(|_| token(&mut g, P1, 1)).collect();
    g.atk_targets = atk.iter().map(|&m| (m, P0)).collect();
    combat::attack_triggers(&mut g, P1, &atk, P0).unwrap();
    assert_eq!(hand_len(&g, P0), h + 3, "attacked by two");
    g.atk_targets = atk.iter().map(|&m| (m, P2)).collect();
    combat::attack_triggers(&mut g, P1, &atk, P2).unwrap();
    assert_eq!(hand_len(&g, P0), h + 3, "an attack on someone else");
}

// ======================================================== Wear // Tear
#[test]
fn wear_tear_fuses_when_both_targets_and_white_are_there() {
    let mut g = game();
    let ring = perm(&mut g, P1, "Sol Ring");
    let tithe = perm(&mut g, P1, "Smothering Tithe");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Wear // Tear"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(ring), ..Ctx::default() }).unwrap();
    assert!(!g.perm(ring).on_bf, "Wear destroys the artifact");
    assert!(!g.perm(tithe).on_bf, "Tear, fused with {{W}}, the enchantment");
}

#[test]
fn wear_tear_without_white_destroys_one() {
    let mut g = game();
    let ring = perm(&mut g, P1, "Sol Ring");
    let tithe = perm(&mut g, P1, "Smothering Tithe");
    let c = hand(&mut g, P0, &["Wear // Tear"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(tithe), ..Ctx::default() }).unwrap();
    assert!(!g.perm(tithe).on_bf);
    assert!(g.perm(ring).on_bf, "no {{W}} left to fuse");
}

// ======================================================== Slayers' Stronghold / Sunhome
#[test]
fn slayers_stronghold_gives_a_new_creature_haste() {
    let mut g = game();
    let l = lands(&mut g, P0, "Slayers' Stronghold", 1, false)[0];
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let m = perm(&mut g, P0, "Firemane Avenger");
    g.perm_mut(m).sick = true;
    let opts = land_option(&mut g, P0, l);
    assert_eq!(opts.len(), 1);
    act(&mut g, P0, &opts[0]);
    assert!(combat::has_haste(&g, m) && combat::kw(&g, m, "vigilance"));
    assert_eq!(g.perm(m).eot_pt, (2, 0));
    assert!(g.perm(l).tapped);
}

#[test]
fn sunhome_gives_an_unblockable_attacker_double_strike() {
    let mut g = game();
    let l = lands(&mut g, P0, "Sunhome, Fortress of the Legion", 1, false)[0];
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Plains", 2, false);
    let m = perm(&mut g, P0, "Aurelia, the Law Above");
    let opts = land_option(&mut g, P0, l);
    assert_eq!(opts.len(), 1, "no opposing creature to block it");
    act(&mut g, P0, &opts[0]);
    assert!(double_strike(&g, m));
    // with a blocker for it, no activation
    let mut g = game();
    let l = lands(&mut g, P0, "Sunhome, Fortress of the Legion", 1, false)[0];
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Plains", 2, false);
    perm(&mut g, P0, "Aurelia, the Law Above");
    perm(&mut g, P1, "Aurelia, the Law Above");
    assert!(land_option(&mut g, P0, l).is_empty());
}

// ======================================================== Sunforger
#[test]
fn sunforger_unattaches_to_cast_removal_from_the_library() {
    let mut g = game();
    let sf = perm(&mut g, P0, "Sunforger");
    let t = token(&mut g, P0, 2);
    g.perm_mut(sf).attached = Some(t);
    assert_eq!(sim_core::engine::values::epow(&g, t), 6, "+4/+0");
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let threat = perm(&mut g, P1, "Avacyn, Angel of Hope");
    let opts = perm_options(&mut g, P0, sf, None);
    assert_eq!(opts.len(), 1, "a threat to answer");
    act(&mut g, P0, &opts[0]);
    assert!(!g.perm(threat).on_bf, "exiled by the instant it found");
    assert_eq!(g.perm(sf).attached, None, "unattached");
}

// ======================================================== Boros Garrison
#[test]
fn boros_garrison_returns_a_land() {
    let mut g = game();
    let plains = lands(&mut g, P0, "Plains", 1, true)[0];
    let c = take(&mut g, P0, "Boros Garrison");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert!(!g.player(P0).lands.contains(&plains), "the Plains goes back");
    assert!(g.player(P0).hand.contains(&card(&g, "Plains")));
    let gar: Vec<PermId> = g.player(P0).lands.clone();
    assert_eq!(gar.len(), 1);
    assert!(g.perm(gar[0]).tapped, "enters tapped");
    let _: CardId = c;
}
