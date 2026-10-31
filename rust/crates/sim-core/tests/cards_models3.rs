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

// ======================================================== Astral Cornucopia (atraxa-infect, t3)
const CORNUCOPIA: &str = "Astral Cornucopia";

fn atraxa_table() -> Game {
    let t3: Vec<&str> = tier("t3").into_iter().filter(|&k| k != "atraxa-infect").take(2).collect();
    let mut g = table(&["atraxa-infect", t3[0], t3[1]]);
    g.active = Some(P0);
    g
}

#[test]
fn astral_cornucopia_is_cast_for_the_largest_x_and_taps_for_x_of_one_colour() {
    let mut g = atraxa_table();
    let ls = lands(&mut g, P0, "Forest", 7, false);
    let c = hand(&mut g, P0, &[CORNUCOPIA])[0];
    assert!(cm3::cornucopia_cast(&mut g, P0, c.0 as i64).unwrap());
    assert_eq!(ls.iter().filter(|&&l| g.perm(l).tapped).count(), 6, "X = 2 costs six");
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    assert_eq!(g.perm(m).data.int(DataKey::Charge), 2);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1 + 2, "a Forest and two from the Cornucopia");
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(m).data.int(DataKey::Charge), 3, "proliferate adds a charge counter");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1 + 3);
}

#[test]
fn astral_cornucopia_needs_three_mana_and_is_never_cast_for_x_zero() {
    let mut g = atraxa_table();
    lands(&mut g, P0, "Forest", 2, false);
    let c = hand(&mut g, P0, &[CORNUCOPIA])[0];
    assert!(!cm3::cornucopia_cast(&mut g, P0, c.0 as i64).unwrap());
    assert!(g.player(P0).hand.contains(&c));
    assert_eq!(sim_core::ai::pool::generic_prio(&g, P0, c), 0, "the generic path leaves it to the hand option");
}

// ======================================================== Angel's Grace (dack-fayden, t6)
#[test]
fn angels_grace_saves_a_lethal_hit_and_keeps_life_at_one_this_turn() {
    let mut g = t6("dack-fayden-white-stax");
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Angel's Grace"])[0];
    g.player_mut(P0).life = 3;
    sim_core::engine::life::lose_life(&mut g, P0, 5, Some(P1), "combat", None).unwrap();
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.player(P0).alive, "cast as it would lose");
    assert_eq!(g.player(P0).life, 1);
    assert!(g.player(P0).gy.contains(&c));
    sim_core::engine::life::lose_life(&mut g, P0, 10, Some(P1), "combat", None).unwrap();
    assert_eq!(g.player(P0).life, 1, "damage can't take it below 1");
    sim_core::ai::win(&mut g, P1, "combo", None).unwrap();
    assert!(g.player(P0).alive, "it can't lose this turn: a drain win does not kill it");
}

// ======================================================== Paladin Class (dack-fayden, t6)
fn run(g: &mut Game, o: sim_core::hooks::Opt, p: PlayerId) -> bool {
    use sim_core::hooks::Action;
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn options_of(g: &mut Game, p: PlayerId, src: PermId) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("an options slot");
    f(g, src, p, Some(true)).unwrap()
}

#[test]
fn paladin_class_level_two_gives_your_creatures_plus_one_plus_one() {
    use sim_core::engine::values::{epow, etgh};
    let mut g = t6("dack-fayden-white-stax");
    let pc = perm(&mut g, P0, "Paladin Class");
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 1);
    let theirs = token(&mut g, P1, 2);
    let (pa, ta, pb, pt) = (epow(&g, a), etgh(&g, a), epow(&g, b), epow(&g, theirs));
    assert_eq!(pb, 1, "level 1: no anthem yet");
    lands(&mut g, P0, "Plains", 3, false);
    let o = options_of(&mut g, P0, pc).into_iter().find(|o| o.label.contains("level 2")).expect("level 2 offered");
    assert!(run(&mut g, o, P0));
    assert_eq!((epow(&g, a), etgh(&g, a)), (pa + 1, ta + 1));
    assert_eq!(epow(&g, b), pb + 1);
    assert_eq!(epow(&g, theirs), pt, "only your creatures");
}

// ======================================================== Mandate of Peace (dack-fayden, t6)
#[test]
fn mandate_of_peace_is_cast_only_during_combat_and_ends_a_lethal_attack() {
    let mut g = table(&["rograkh-thrasios-creature-combo", "dack-fayden-white-stax", "sisay-legends-toolbox"]);
    let d = P1;
    lands(&mut g, d, "Plains", 2, false);
    let c = hand(&mut g, d, &["Mandate of Peace"])[0];
    assert!(!sim_core::engine::cast::castable(&g, d, c, "hand"), "not outside combat");
    g.player_mut(d).life = 6;
    let atk = vec![token(&mut g, P0, 3), token(&mut g, P0, 3)];
    g.in_combat = atk.clone();
    assert!(sim_core::engine::cast::castable(&g, d, c, "hand"), "during combat");
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(&mut g, P0, atk, d, vec![], &mut tot).unwrap();
    assert_eq!(g.player(d).life, 6, "the combat phase ended: no damage");
    assert!(g.player(d).gy.contains(&c));
    let opt = take(&mut g, P0, "Opt");
    assert!(!sim_core::engine::cast::castable(&g, P0, opt, "hand"), "its opponents can't cast spells this turn");
}

// ======================================================== Innkeeper's Talent (atraxa-infect, t3)
#[test]
fn innkeepers_talent_levels_counter_ward_and_doubling() {
    use sim_core::engine::repl::{Ctr, On, Path, counters};
    let mut g = atraxa_table();
    let inn = perm(&mut g, P0, "Innkeeper's Talent");
    let t = token(&mut g, P0, 2);
    let c = g.perm(inn).cd.unwrap();
    let combat = g.registry.get(c).and_then(|i| i.each_combat).unwrap();
    combat(&mut g, inn, P0).unwrap();
    assert_eq!(g.perm(t).plus, 1, "level 1: a +1/+1 counter at the beginning of combat on your turn");
    g.active = Some(P1);
    combat(&mut g, inn, P1).unwrap();
    assert_eq!(g.perm(t).plus, 1, "not on an opponent's turn");
    g.active = Some(P0);
    assert_eq!(sim_core::cardcode::aura_ward(&g, t), 0);
    lands(&mut g, P0, "Forest", 5, false);
    let o = options_of(&mut g, P0, inn).into_iter().next().expect("level 2 offered");
    assert!(run(&mut g, o, P0));
    assert_eq!(sim_core::cardcode::aura_ward(&g, t), 1, "level 2: ward {{1}} on a permanent with counters");
    let bare = token(&mut g, P0, 2);
    assert_eq!(sim_core::cardcode::aura_ward(&g, bare), 0);
    assert_eq!(counters(&g, P0, On::Player(P1), Ctr::Poison, 1, Path::POISON), 1);
    let o = options_of(&mut g, P0, inn).into_iter().next().expect("level 3 offered");
    assert!(run(&mut g, o, P0));
    assert_eq!(counters(&g, P0, On::Player(P1), Ctr::Poison, 1, Path::POISON), 2, "level 3: twice the poison");
    assert_eq!(counters(&g, P0, On::Perm(t), Ctr::Plus, 1, Path::ALL), 2);
    assert_eq!(counters(&g, P1, On::Perm(t), Ctr::Plus, 1, Path::ALL), 1, "only the counters you put");
}

// ======================================================== Venerated Rotpriest (atraxa-infect, t3)
#[test]
fn venerated_rotpriest_poisons_an_opponent_when_your_creature_is_targeted_by_a_spell() {
    let mut g = atraxa_table();
    perm(&mut g, P0, "Venerated Rotpriest");
    let t = token(&mut g, P0, 2);
    let theirs = token(&mut g, P1, 2);
    g.player_mut(PlayerId(2)).poison = 3;
    sim_core::impls::card_models3::spell_targeted(&mut g, &Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(PlayerId(2)).poison, 4, "the opponent closest to ten");
    sim_core::impls::card_models3::spell_targeted(&mut g, &Ctx { target: Some(theirs), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(PlayerId(2)).poison + g.player(P1).poison, 4, "not for an opponent's creature");
    // through a real cast: an opponent's removal spell aimed at our creature
    lands(&mut g, P1, "Plains", 1, false);
    let stp = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    sim_core::engine::cast::cast_card(&mut g, P1, stp, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(PlayerId(2)).poison, 5);
}

// ======================================================== Ajani, Sleeper Agent (atraxa-infect, t3)
fn walker_uses(g: &mut Game, p: PlayerId, w: PermId) -> Vec<sim_core::hooks::Opt> {
    options_of(g, p, w)
}

#[test]
fn ajani_sleeper_agent_reveals_distributes_and_his_emblem_poisons() {
    let mut g = atraxa_table();
    let ajani = perm(&mut g, P0, "Ajani, Sleeper Agent");
    assert_eq!(g.perm(ajani).loyalty, Some(4));
    // +1 with a creature on top: to hand
    let top = take(&mut g, P0, "Venerated Rotpriest");
    g.player_mut(P0).library.insert(0, top);
    let o = walker_uses(&mut g, P0, ajani).into_iter().find(|o| o.label.ends_with("+1")).expect("+1");
    assert!(run(&mut g, o, P0));
    assert!(g.player(P0).hand.contains(&top));
    assert_eq!(g.perm(ajani).loyalty, Some(5));
    // the emblem: a creature spell gives the opponent closest to ten two poison counters
    g.player_mut(P0).emblems.push("ajani_sleeper");
    g.player_mut(P1).poison = 2;
    sim_core::impls::rules::emblem_cast(&mut g, P0, top).unwrap();
    assert_eq!(g.player(P1).poison, 4);
    let opt = take(&mut g, P0, "Swords to Plowshares");
    sim_core::impls::rules::emblem_cast(&mut g, P0, opt).unwrap();
    assert_eq!(g.player(P1).poison, 4, "not for an instant");
}

#[test]
fn ajani_minus_three_puts_all_three_counters_on_an_infect_creature() {
    let mut g = atraxa_table();
    let ajani = perm(&mut g, P0, "Ajani, Sleeper Agent");
    let big = token(&mut g, P0, 5);
    let inf = perm(&mut g, P0, "Venerated Rotpriest");
    g.active = Some(P0);
    let o = walker_uses(&mut g, P0, ajani).into_iter().find(|o| o.label.ends_with("-3")).expect("-3");
    assert!(run(&mut g, o, P0));
    assert_eq!((g.perm(inf).plus, g.perm(big).plus), (3, 0), "toxic: each counter is a poison counter");
    assert!(g.perm(inf).eot_kw.contains(&"vigilance"));
    assert_eq!(g.perm(ajani).loyalty, Some(1));
}

// ======================================================== Ixhel, Scion of Atraxa (atraxa-infect, t3)
#[test]
fn ixhel_takes_the_top_card_of_each_corrupted_opponent_at_your_end_step() {
    let mut g = atraxa_table();
    let ix = perm(&mut g, P0, "Ixhel, Scion of Atraxa");
    g.player_mut(P1).poison = 3;
    g.player_mut(PlayerId(2)).poison = 2;
    let (top1, top2) = (g.player(P1).library[0], g.player(PlayerId(2)).library[0]);
    let c = g.perm(ix).cd.unwrap();
    let end = g.registry.get(c).and_then(|i| i.end_step).unwrap();
    end(&mut g, ix, P1).unwrap();
    assert!(!g.player(P0).hand.contains(&top1), "only at your end step");
    end(&mut g, ix, P0).unwrap();
    assert!(g.player(P0).hand.contains(&top1) && g.player(P0).agent_ids.contains(&top1), "playable with any mana");
    assert_eq!(g.player(P0).stolen.get(&top1), Some(&P1));
    assert!(!g.player(P0).hand.contains(&top2), "two poison: not corrupted");
    assert_eq!(g.player(PlayerId(2)).library[0], top2);
}
