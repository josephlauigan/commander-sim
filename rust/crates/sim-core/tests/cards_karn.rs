//! The karn deck (impls/karn.rs): Karn, Legacy Reforged and the artifact-only mana, the Tron lands, Urza's Workshop,
//! Mishra's Factory, The Mightstone and Weakstone. Seat 0 plays the karn deck, seat 1 the seph deck.

use sim_core::engine::mana::{can_pay, mana_units, pay};
use sim_core::engine::turn::step_start;
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::{Call, Event};
use sim_core::ids::PlayerId;
use sim_core::impls::karn;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["karn", "seph"]);
    g.selfpt = true;
    g
}

fn upkeep(g: &mut Game, p: PlayerId) {
    sim_core::engine::hooks::fire_trigger(g, Event::Upkeep, Call::Player { p }).unwrap();
}

#[test]
fn karn_adds_one_colourless_per_artifact_at_upkeep() {
    let mut g = game();
    perm(&mut g, P0, "Karn, Legacy Reforged");
    for n in ["Sol Ring", "Mind Stone", "Mana Vault"] {
        perm(&mut g, P0, n);
    }
    assert_eq!(karn::artifacts(&g, P0), 4);
    upkeep(&mut g, P0);
    assert_eq!(g.player(P0).mana.art(), 4);
    // the opponent's upkeep adds nothing for Karn
    let before = g.player(P0).mana.art();
    upkeep(&mut g, P1);
    assert_eq!(g.player(P0).mana.art(), before);
    assert_eq!(g.player(P0).milestone.get("karn").copied(), Some(g.player(P0).turns));
}

#[test]
fn karn_is_as_big_as_his_biggest_artifact() {
    let mut g = game();
    let k = perm(&mut g, P0, "Karn, Legacy Reforged");
    assert_eq!((epow(&g, k), etgh(&g, k)), (5, 5), "he is his own biggest artifact");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!((epow(&g, k), etgh(&g, k)), (5, 5));
    perm(&mut g, P0, "Myr Battlesphere"); // mana value 7
    assert_eq!((epow(&g, k), etgh(&g, k)), (7, 7));
    // an artifact of the opponent doesn't count
    perm(&mut g, P1, "Wurmcoil Engine");
    assert_eq!(epow(&g, k), 7);
}

#[test]
fn karns_mana_pays_artifact_spells_and_abilities_not_other_spells() {
    let mut g = game();
    g.player_mut(P0).mana.set_art(6);
    let wurmcoil = card(&g, "Wurmcoil Engine");
    let ugin = card(&g, "Ugin, the Spirit Dragon");
    let dust = card(&g, "All Is Dust");
    g.pay_for = Some(wurmcoil);
    assert!(can_pay(&g, P0, 6, "", false), "an artifact creature spell");
    g.pay_for = Some(ugin);
    assert!(!can_pay(&g, P0, 6, "", false), "not a planeswalker");
    g.pay_for = Some(dust);
    assert!(!can_pay(&g, P0, 1, "", false), "not All Is Dust");
    g.pay_for = None;
    assert!(can_pay(&g, P0, 4, "", false), "an ability cost");
    assert!(pay(&mut g, P0, 4, "", false).unwrap());
    assert_eq!(g.player(P0).mana.art(), 2);
    // it is used before any land
    lands(&mut g, P0, "Wastes", 2, false);
    g.pay_for = Some(wurmcoil);
    assert!(pay(&mut g, P0, 2, "", false).unwrap());
    assert_eq!(g.player(P0).mana.art(), 0);
    assert!(g.player(P0).lands.iter().all(|&l| !g.perm(l).tapped), "Karn's mana paid, not the lands");
}

#[test]
fn karns_mana_lasts_the_turn_and_is_gone_when_the_next_begins() {
    let mut g = game();
    perm(&mut g, P0, "Karn, Legacy Reforged");
    upkeep(&mut g, P0);
    assert_eq!(g.player(P0).mana.art(), 1);
    // steps and phases do not empty it (the engine empties floating mana only when a turn starts)
    sim_core::human::mana::empty_pools(&mut g);
    assert_eq!(g.player(P0).mana.art(), 1);
    // the next turn begins (the opponent's): gone
    step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).mana.art(), 0);
}

#[test]
fn karns_mana_in_the_pool_of_a_person() {
    use sim_core::human::mana::{ManaPool, Spending};
    let mut g = game();
    let wurmcoil = card(&g, "Wurmcoil Engine");
    let ugin = card(&g, "Ugin, the Spirit Dragon");
    let mut pool = ManaPool::default();
    pool.art = 4;
    pool.spending = Some(Spending::Card(wurmcoil));
    assert!(pool.can_pay(&g, P0, 4, ""));
    pool.spending = Some(Spending::Card(ugin));
    assert!(!pool.can_pay(&g, P0, 1, ""));
    pool.spending = None;
    assert!(pool.can_pay(&g, P0, 4, ""), "an ability");
    pool.spending = Some(Spending::Card(wurmcoil));
    pool.add('C', 1);
    assert!(pool.pay(&g, P0, 5, ""));
    assert_eq!((pool.art, pool.total()), (0, 0));
    // a step ends: the pool empties, Karn's mana stays
    pool.art = 2;
    pool.add('C', 1);
    pool.empty();
    assert_eq!((pool.art, pool.m), (2, [0; 7]));
    g.player_mut(P0).pool = pool;
    sim_core::human::mana::empty_pools(&mut g);
    assert_eq!(g.player(P0).pool.art, 2);
    // the player's own turn starting empties it
    step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).pool.art, 0);
}

#[test]
fn the_mightstone_and_weakstone_makes_two_for_artifacts_only() {
    let mut g = game();
    let m = perm(&mut g, P0, "The Mightstone and Weakstone");
    g.perm_mut(m).sick = false;
    let u = |g: &Game| mana_units(g, P0, false).iter().map(|w| w.amt).sum::<u32>();
    assert_eq!(u(&g), 2, "{{T}}: {{C}}{{C}}");
    g.pay_for = Some(card(&g, "Ugin, the Spirit Dragon"));
    assert!(!can_pay(&g, P0, 1, "", false), "not for a nonartifact spell");
    g.pay_for = Some(card(&g, "Wurmcoil Engine"));
    assert!(can_pay(&g, P0, 2, "", false), "{{C}}{{C}} for an artifact spell");
    assert!(!can_pay(&g, P0, 3, "", false));
    g.pay_for = None;
    assert!(can_pay(&g, P0, 2, "", false), "and for an ability");
}

#[test]
fn mishras_workshop_is_for_artifact_spells_only() {
    let mut g = game();
    lands(&mut g, P0, "Mishra's Workshop", 1, false);
    g.pay_for = Some(card(&g, "Wurmcoil Engine"));
    assert!(can_pay(&g, P0, 3, "", false));
    g.pay_for = Some(card(&g, "Ugin, the Spirit Dragon"));
    assert!(!can_pay(&g, P0, 1, "", false));
    g.pay_for = None;
    assert!(!can_pay(&g, P0, 1, "", false), "not an ability");
}

#[test]
fn tron_makes_seven_with_all_three_and_one_each_without() {
    let mut g = game();
    lands(&mut g, P0, "Urza's Mine", 1, false);
    lands(&mut g, P0, "Urza's Power Plant", 1, false);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2);
    lands(&mut g, P0, "Urza's Tower", 1, false);
    assert!(karn::tron_on(&g, P0));
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 7);
}

#[test]
fn urza_workshop_counts_urza_lands_with_three_artifacts() {
    let mut g = game();
    lands(&mut g, P0, "Urza's Workshop", 1, false);
    lands(&mut g, P0, "Urza's Mine", 1, false);
    lands(&mut g, P0, "Urza's Factory", 1, false);
    lands(&mut g, P0, "Wastes", 1, false);
    // without the artifacts: {C} each
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 4);
    for n in ["Sol Ring", "Mind Stone", "Mana Vault"] {
        perm(&mut g, P0, n);
    }
    // three artifacts: the Workshop makes {C} for each of the three Urza's lands (itself, the Mine, the Factory)
    let ws = mana_units(&g, P0, false).iter().filter(|w| w.amt == 3).count();
    assert!(ws >= 1, "the Workshop makes three");
}

#[test]
fn mishras_factory_animates_as_an_artifact_creature() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    lands(&mut g, P0, "Mishra's Factory", 1, false);
    lands(&mut g, P0, "Wastes", 2, false);
    let l = g.player(P0).lands[0];
    let n_before = karn::artifacts(&g, P0);
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    let _ = opts; // (the animate option depends on an open attack; the block response below is the direct path)
    let atk = token(&mut g, P1, 6);
    g.player_mut(P0).life = 10;
    assert!(karn::block_response(&mut g, P0, P1, &[atk]).unwrap());
    assert!(!g.player(P0).lands.contains(&l), "the land is a creature now");
    assert_eq!(karn::artifacts(&g, P0), n_before + 1, "an artifact while animated (Karn would count it)");
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Mishra's Factory").unwrap();
    assert_eq!((epow(&g, m), etgh(&g, m)), (2, 2));
}

#[test]
fn karn_prio_casts_him_first_and_holds_the_sweepers() {
    let mut g = game();
    g.player_mut(P0).turns = 2;
    let cmd = g.player(P0).cmd;
    hand(&mut g, P0, &["Oblivion Stone", "All Is Dust", "Sol Ring", "Lightning Greaves"]);
    let p = |g: &Game, n: &str| karn::karn_prio(g, P0, card(g, n));
    assert!(karn::karn_prio(&g, P0, cmd) >= 100);
    assert!(p(&g, "Sol Ring") >= 74);
    assert_eq!(p(&g, "Oblivion Stone"), 0, "not while we're not behind");
    assert_eq!(p(&g, "All Is Dust"), 0, "no coloured board to sweep");
    assert!(p(&g, "Lightning Greaves") < 50, "Greaves wait for Karn");
    perm(&mut g, P0, "Karn, Legacy Reforged");
    assert!(p(&g, "Lightning Greaves") > 80);
}

#[test]
fn the_land_searches_take_the_missing_tron_piece_then_wastes() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    lands(&mut g, P0, "Urza's Mine", 1, false);
    let names = ["Wastes", "Urza's Tower", "Urza's Power Plant", "Mishra's Workshop"];
    let cands: Vec<_> = names.iter().map(|n| card(&g, n)).collect();
    let pick = |g: &Game, c: &[sim_core::ids::CardId]| g.db.get(karn::search_pick(g, P0, c).unwrap()).name.to_string();
    assert!(["Urza's Tower", "Urza's Power Plant"].contains(&pick(&g, &cands).as_str()), "a Tron piece first");
    lands(&mut g, P0, "Urza's Tower", 1, false);
    assert_eq!(pick(&g, &cands), "Urza's Power Plant");
    lands(&mut g, P0, "Urza's Power Plant", 1, false);
    assert_eq!(pick(&g, &cands), "Mishra's Workshop", "Tron whole: a land that does something");
    assert_eq!(pick(&g, &cands[..1]), "Wastes");
    // the other decks' searches are left alone
    assert!(karn::search_pick(&table(&["seph", "karn"]), P0, &cands).is_none());
}

#[test]
fn the_baubles_stay_out_while_karn_is_coming() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    hand(&mut g, P0, &["Sol Ring", "Mind Stone"]);
    assert!(karn::hold_ability(&g, P0, "Mishra's Bauble"), "Karn is in the command zone");
    assert!(!karn::hold_ability(&g, P0, "Sol Ring"));
    g.player_mut(P0).hand.clear();
    assert!(!karn::hold_ability(&g, P0, "Mishra's Bauble"), "an empty hand digs");
}
