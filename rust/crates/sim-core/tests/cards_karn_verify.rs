//! Karn deck: the cards that were already in the engine, checked against their Oracle text (impls/karn_engines.rs holds
//! the fixes). Each on a hand-built position.

use sim_core::engine::mana::{can_pay, pay, total_mana};
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    g
}

fn name(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

fn lands_named(g: &Game, p: PlayerId, n: &str) -> usize {
    g.player(p).lands.iter().filter(|&&l| &*g.db.get(g.land(l).card()).name == n).count()
}

/// every option the AI has for p's permanents, lands and hooks, main phase two
fn all_opts(g: &mut Game) -> Vec<Opt> {
    let s = sim_core::ai::brain::Situation::new(g, P0);
    let mut o = sim_core::dsl::ability_options(g, P0, true).unwrap();
    o.extend(sim_core::ai::brain::hook_options(g, P0, &s, Some(true)).unwrap());
    o.extend(sim_core::ai::brain::hook_options(g, P0, &s, Some(false)).unwrap()); // the first main phase too
    o
}

fn run(g: &mut Game, word: &str) -> bool {
    let o = all_opts(g);
    let best = o.iter().filter(|o| o.label.contains(word)).max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap());
    let Some(x) = best else { panic!("no option {word:?} in {:?}", o.iter().map(|o| &o.label).collect::<Vec<_>>()) };
    sim_core::ai::act::perform(g, P0, x.act.as_ref().unwrap()).unwrap()
}

fn has_opt(g: &mut Game, word: &str) -> bool {
    all_opts(g).iter().any(|o| o.label.contains(word))
}

fn cast(g: &mut Game, p: PlayerId, n: &str) {
    let c = hand(g, p, &[n])[0];
    assert!(sim_core::ai::brain::do_cast(g, p, c, None).unwrap(), "{n} cast");
    sim_core::engine::stack::settle_stack(g).unwrap();
}

// ======================================================== mana
#[test]
fn ancient_tomb_taps_for_two_colourless_and_hurts() {
    let mut g = tbl();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    assert_eq!(total_mana(&g, P0, false), 2);
    let life = g.player(P0).life;
    assert!(pay(&mut g, P0, 2, "", false).unwrap());
    assert_eq!(g.player(P0).life, life - 2, "2 damage");
}

#[test]
fn sol_ring_and_hedron_archive_make_two() {
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(total_mana(&g, P0, false), 2);
    perm(&mut g, P0, "Hedron Archive");
    assert_eq!(total_mana(&g, P0, false), 4);
}

#[test]
fn hedron_archive_sacrifices_for_two_cards() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let h = perm(&mut g, P0, "Hedron Archive");
    let hand0 = g.player(P0).hand.len();
    g.player_mut(P0).hand.clear();
    assert!(has_opt(&mut g, "Hedron Archive"));
    assert!(run(&mut g, "Hedron Archive"));
    assert!(!g.perm(h).on_bf);
    assert_eq!(g.player(P0).hand.len(), 2, "drew two (hand was {hand0})");
}

#[test]
fn thought_vessel_taps_for_one_and_removes_the_hand_limit() {
    let mut g = tbl();
    perm(&mut g, P0, "Thought Vessel");
    assert_eq!(total_mana(&g, P0, false), 1);
    let c = card(&g, "Thought Vessel");
    assert!(g.db.get(c).tag(sim_core::tag::Tag::Nomax), "no maximum hand size");
}

#[test]
fn mana_vault_makes_three_does_not_untap_and_pays_four_to_untap() {
    let mut g = tbl();
    let v = perm(&mut g, P0, "Mana Vault");
    assert_eq!(total_mana(&g, P0, false), 3);
    assert!(pay(&mut g, P0, 3, "", false).unwrap());
    assert!(g.perm(v).tapped);
    let c = card(&g, "Mana Vault");
    assert!(g.db.get(c).tag(sim_core::tag::Tag::Vaultping) && g.db.get(c).tag(sim_core::tag::Tag::Nountap));
    lands(&mut g, P0, "Wastes", 5, false);
    g.player_mut(P0).life = 12;
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Upkeep,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    assert!(!g.perm(v).tapped, "paid {{4}} at upkeep to untap it");
}

#[test]
fn liquimetal_torque_taps_for_one() {
    let mut g = tbl();
    perm(&mut g, P0, "Liquimetal Torque");
    assert_eq!(total_mana(&g, P0, false), 1);
}

// ======================================================== land fetchers
#[test]
fn wayfarers_bauble_fetches_a_wastes_tapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let b = perm(&mut g, P0, "Wayfarer's Bauble");
    assert!(run(&mut g, "Wayfarer's Bauble"));
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.player(P0).lands.len(), 3);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.land(l).tapped);
    assert_eq!(name(&g, g.land(l).card()), "Wastes");
}

#[test]
fn burnished_hart_fetches_two_wastes_tapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let h = perm(&mut g, P0, "Burnished Hart");
    assert!(run(&mut g, "Burnished Hart"));
    assert!(!g.perm(h).on_bf);
    assert_eq!(g.player(P0).lands.len(), 5);
    assert!(g.player(P0).lands[3..].iter().all(|&l| g.land(l).tapped));
}

#[test]
fn solemn_simulacrum_fetches_a_wastes_and_draws_when_it_dies() {
    let mut g = tbl();
    let n = g.player(P0).lands.len();
    cast_free_creature(&mut g, "Solemn Simulacrum");
    assert_eq!(g.player(P0).lands.len(), n + 1);
    let s = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Solemn Simulacrum"))
        .unwrap();
    let hand0 = g.player(P0).hand.len();
    sim_core::engine::zones::die(&mut g, s, "destroy").unwrap();
    sim_core::engine::stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "it drew a card");
}

fn cast_free_creature(g: &mut Game, n: &str) {
    let c = hand(g, P0, &[n])[0];
    sim_core::engine::cast::cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
    sim_core::engine::stack::settle_stack(g).unwrap();
}

#[test]
fn expedition_map_finds_the_missing_tron_piece_not_a_basic() {
    let mut g = tbl();
    lands(&mut g, P0, "Urza's Mine", 1, false);
    lands(&mut g, P0, "Urza's Power Plant", 1, false);
    lands(&mut g, P0, "Wastes", 1, false);
    let m = perm(&mut g, P0, "Expedition Map");
    let hand0 = g.player(P0).hand.len();
    assert!(run(&mut g, "Expedition Map"));
    assert!(!g.perm(m).on_bf);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert_eq!(name(&g, *g.player(P0).hand.last().unwrap()), "Urza's Tower");
}

#[test]
fn urzas_cave_fetches_the_tron_piece_onto_the_battlefield_tapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Urza's Mine", 1, false);
    lands(&mut g, P0, "Urza's Power Plant", 1, false);
    lands(&mut g, P0, "Wastes", 3, false);
    let cave = lands(&mut g, P0, "Urza's Cave", 1, false)[0];
    assert!(has_opt(&mut g, "Urza's Cave"));
    assert!(run(&mut g, "Urza's Cave"));
    assert!(!g.player(P0).lands.contains(&cave), "sacrificed");
    assert_eq!(lands_named(&g, P0, "Urza's Tower"), 1);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.land(l).tapped);
}

#[test]
fn urzas_cave_taps_for_colourless() {
    let mut g = tbl();
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert_eq!(total_mana(&g, P0, false), 1);
    assert!(can_pay(&g, P0, 1, "", false));
}

// ======================================================== other lands
#[test]
fn endless_atlas_needs_three_lands_with_the_same_name() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    lands(&mut g, P0, "Urza's Mine", 1, false);
    perm(&mut g, P0, "Endless Atlas");
    assert!(!has_opt(&mut g, "Endless Atlas"), "two Wastes and a Mine");
    lands(&mut g, P0, "Wastes", 1, false);
    assert!(has_opt(&mut g, "Endless Atlas"));
    let hand0 = g.player(P0).hand.len();
    assert!(run(&mut g, "Endless Atlas"));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
}

#[test]
fn inventors_fair_gains_life_at_upkeep_with_three_artifacts() {
    let mut g = tbl();
    lands(&mut g, P0, "Inventors' Fair", 1, false);
    let life = g.player(P0).life;
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::LandUpkeep,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    assert_eq!(g.player(P0).life, life, "no artifacts");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, "Mox Opal");
    sim_core::impls::lands::land_upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, life + 1);
}

#[test]
fn inventors_fair_sacrifices_to_tutor_an_artifact() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 4, false);
    let fair = lands(&mut g, P0, "Inventors' Fair", 1, false)[0];
    assert!(!has_opt(&mut g, "Inventors' Fair"), "needs three artifacts");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, "Mox Opal");
    let hand0 = g.player(P0).hand.len();
    assert!(run(&mut g, "Inventors' Fair"));
    assert!(!g.player(P0).lands.contains(&fair));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    let c = *g.player(P0).hand.last().unwrap();
    assert!(g.db.get(c).types.has(sim_core::cards::Types::ARTIFACT), "{}", name(&g, c));
}

#[test]
fn rogues_passage_makes_a_creature_unblockable_for_four() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 4, false);
    lands(&mut g, P0, "Rogue's Passage", 1, false);
    let t = token(&mut g, P0, 4);
    assert!(!has_opt(&mut g, "Rogue's Passage"), "nobody to block");
    token(&mut g, P1, 2);
    assert!(has_opt(&mut g, "Rogue's Passage"));
    assert!(run(&mut g, "Rogue's Passage"));
    assert_eq!(g.perm(t).data.get(DataKey::Unbl), Some(&sim_core::state::Val::Stamp(g.turn_stamp())));
}

// ======================================================== artifacts with abilities
#[test]
fn staff_of_domination_draws_for_five_and_untaps_for_one() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 6, false);
    let s = perm(&mut g, P0, "Staff of Domination");
    let hand0 = g.player(P0).hand.len();
    assert!(run(&mut g, "Staff of Domination"));
    assert!(g.perm(s).tapped);
    assert!(g.player(P0).hand.len() > hand0 || g.player(P0).life > 40, "drew or gained");
}

#[test]
fn liquimetal_torque_makes_a_permanent_an_artifact_until_end_of_turn() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Liquimetal Torque");
    let c = token(&mut g, P0, 2);
    assert!(!sim_core::impls::t5::is_artifact(&g, c));
    assert!(has_opt(&mut g, "Liquimetal Torque"));
    assert!(run(&mut g, "Liquimetal Torque"));
    assert!(g.perm(t).tapped);
    assert!(sim_core::impls::t5::is_artifact(&g, c), "the target is an artifact");
}

// ======================================================== creatures
#[test]
fn meteor_golem_destroys_a_nonland_permanent_of_an_opponent() {
    let mut g = tbl();
    let target = perm(&mut g, P1, "Sol Ring");
    cast_free_creature(&mut g, "Meteor Golem");
    assert!(!g.perm(target).on_bf, "destroyed");
}

#[test]
fn wurmcoil_engine_leaves_a_deathtouch_wurm_and_a_lifelink_wurm() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wurmcoil Engine");
    assert_eq!((epow(&g, w), etgh(&g, w)), (6, 6));
    sim_core::engine::zones::die(&mut g, w, "destroy").unwrap();
    sim_core::engine::stack::settle_stack(&mut g).unwrap();
    let wurms: Vec<PermId> =
        g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).on_bf).collect();
    assert_eq!(wurms.len(), 2);
    assert!(wurms.iter().all(|&m| epow(&g, m) == 3 && etgh(&g, m) == 3));
    assert!(wurms.iter().any(|&m| g.perm(m).dt) && wurms.iter().any(|&m| g.perm(m).lifelink));
    assert!(wurms.iter().all(|&m| sim_core::impls::t5::is_artifact(&g, m)), "artifact creature tokens");
}

#[test]
fn hangarback_walker_enters_with_x_counters_and_leaves_thopters() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 6, false);
    cast(&mut g, P0, "Hangarback Walker");
    let h = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Hangarback Walker"))
        .expect("on the battlefield");
    let x = g.perm(h).plus;
    assert!(x >= 1, "X counters, {x}");
    assert_eq!(epow(&g, h), x);
    sim_core::engine::zones::die(&mut g, h, "destroy").unwrap();
    sim_core::engine::stack::settle_stack(&mut g).unwrap();
    let thopters: Vec<PermId> =
        g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).on_bf).collect();
    assert_eq!(thopters.len() as i32, x, "one Thopter per counter");
    assert!(thopters.iter().all(|&m| g.perm(m).fly && epow(&g, m) == 1 && etgh(&g, m) == 1));
}

#[test]
fn hangarback_walker_grows_for_one_and_tap() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 7, false);
    cast(&mut g, P0, "Hangarback Walker");
    let h = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Hangarback Walker"))
        .unwrap();
    g.perm_mut(h).sick = false;
    let x = g.perm(h).plus;
    assert!(run(&mut g, "Hangarback Walker"));
    assert_eq!(g.perm(h).plus, x + 1);
}

// ======================================================== Batterskull
#[test]
fn batterskull_makes_a_germ_that_it_pumps_and_returns_for_three() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 8, false);
    cast(&mut g, P0, "Batterskull");
    let b = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Batterskull"))
        .unwrap();
    let germ = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).expect("a Germ");
    assert_eq!(g.perm(b).attached, Some(germ), "attached");
    assert_eq!((epow(&g, germ), etgh(&g, germ)), (4, 4));
    assert!(sim_core::dsl::has_kw(&g, germ, "lifelink") || g.perm(germ).lifelink);
    assert!(!has_opt(&mut g, "Batterskull: return"), "carried by the Germ");
    sim_core::engine::zones::die(&mut g, germ, "destroy").unwrap();
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    let o = all_opts(&mut g);
    assert!(
        o.iter().any(|o| o.label.contains("Batterskull") && o.label.to_lowercase().contains("return")),
        "{{3}}: return it: {:?}",
        o.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}

// ======================================================== Pithing Needle
#[test]
fn pithing_needle_stops_the_named_cards_abilities() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 4, false);
    perm(&mut g, P0, "Hedron Archive");
    assert!(has_opt(&mut g, "Hedron Archive"));
    let n = perm(&mut g, P1, "Pithing Needle");
    let c = card(&g, "Hedron Archive");
    g.perm_mut(n).data.set(DataKey::Imprint, sim_core::state::Val::Card(c));
    assert!(!has_opt(&mut g, "Hedron Archive"), "the Needle names it");
    // mana abilities are not stopped
    assert_eq!(total_mana(&g, P0, false), 4 + 2);
}

#[test]
fn pithing_needle_stops_a_named_land_too() {
    let mut g = tbl();
    lands(&mut g, P0, "Urza's Mine", 1, false);
    lands(&mut g, P0, "Urza's Power Plant", 1, false);
    lands(&mut g, P0, "Wastes", 3, false);
    lands(&mut g, P0, "Urza's Cave", 1, false);
    assert!(has_opt(&mut g, "Urza's Cave"));
    let n = perm(&mut g, P1, "Pithing Needle");
    let c = card(&g, "Urza's Cave");
    g.perm_mut(n).data.set(DataKey::Imprint, sim_core::state::Val::Card(c));
    assert!(!has_opt(&mut g, "Urza's Cave"));
}

fn staff_table() -> Game {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 6, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    perm(&mut g, P0, "Staff of Domination");
    g
}

#[test]
fn staff_of_domination_with_infinite_mana_draws_down_but_never_decks() {
    let mut g = staff_table();
    let b = card(&g, "Walking Ballista");
    g.player_mut(P0).library.retain(|&c| c != b); // no Ballista to find: it stops at the safe size
    assert!(sim_core::impls::karn_engines::infinite_mana(&g, P0));
    assert!(run(&mut g, "draw with infinite mana"));
    assert_eq!(g.player(P0).library.len(), sim_core::impls::karn_engines::SAFE_LIBRARY);
    assert!(g.player(P0).alive);
    assert!(!has_opt(&mut g, "draw with infinite mana"), "nothing more to draw");
}

#[test]
fn staff_of_domination_with_infinite_mana_stops_when_it_has_the_ballista() {
    let mut g = staff_table();
    let b = take(&mut g, P0, "Walking Ballista");
    let lib = &mut g.player_mut(P0).library;
    let at = lib.len() - 5; // the sixth card from the top: the sixth draw
    lib.insert(at, b);
    let before = g.player(P0).library.len();
    assert!(run(&mut g, "draw with infinite mana"));
    assert!(g.player(P0).hand.contains(&b));
    assert_eq!(g.player(P0).library.len(), before - 6, "stopped at the Ballista");
}

#[test]
fn no_staff_draw_without_the_infinite_mana_pair() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 6, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Staff of Domination");
    assert!(!sim_core::impls::karn_engines::infinite_mana(&g, P0));
    assert!(!has_opt(&mut g, "draw with infinite mana"));
}
