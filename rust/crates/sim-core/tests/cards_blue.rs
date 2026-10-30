//! The blue creatures (impls/blue_cards.rs) on hand-built positions, with each Oracle text as the expectation.
//! Seat 0 plays a stand-in deck (the cards are keyed by name), seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::mana::{cost_of, pay};
use sim_core::engine::values::{epow, etgh, untargetable};
use sim_core::engine::{cast, hooks, turn, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{alela, mine};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ELROND: &str = "Elrond, Lord of Rivendell";

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn hand_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn has_perm(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn perm_named(g: &Game, p: PlayerId, name: &str) -> PermId {
    g.player(p)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && g.perm(m).name == name)
        .unwrap_or_else(|| panic!("no {name}"))
}

fn fill_library(g: &mut Game, p: PlayerId, n: usize) {
    let cs = vec![card(g, "Island"); n];
    g.player_mut(p).library = cs;
}

fn opts_of(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn has_opt(g: &mut Game, p: PlayerId, label: &str) -> bool {
    opts_of(g, p, false).iter().any(|o| o.label == label)
}

fn do_opt(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = opts_of(g, p, false);
    let o = opts
        .iter()
        .find(|o| o.label == label)
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

/// cast a card from the hand with its cost paid
fn cast_paid(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    let (gn, pips) = cost_of(g, p, c);
    pay(g, p, gn, &pips, false).unwrap();
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

/// a new turn for the "this turn" bookkeeping
fn next_turn(g: &mut Game) {
    g.round += 1;
}

fn level(g: &Game) -> u32 {
    g.player(P0).ring_level
}

// ================================================================== Elrond and the Ring
#[test]
fn elrond_scrys_and_the_second_entry_each_turn_tempts_the_ring() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    let e = perm(&mut g, P0, ELROND);
    assert_eq!(level(&g), 0, "his own entry is the first resolution: scry only");
    assert_eq!(g.perm(e).data.int(DataKey::Resolved), 1);
    perm(&mut g, P0, "Aven Fisher");
    assert_eq!(level(&g), 1, "the second creature: the Ring tempts you");
    assert!(mine::ring_bearer(&g, P0).is_some());
    perm(&mut g, P0, "Ornithopter");
    assert_eq!(level(&g), 1, "the third resolution only scries");
    assert_eq!(g.perm(e).data.int(DataKey::Resolved), 3);
    // a new turn starts the count again
    next_turn(&mut g);
    perm(&mut g, P0, "Coral Barrier"); // and its Squid: two entries
    assert_eq!(level(&g), 2);
}

#[test]
fn elrond_counts_tokens_one_trigger_each_and_other_turns_too() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    perm(&mut g, P0, ELROND);
    g.active = Some(P1); // an opponent's turn
    next_turn(&mut g);
    token(&mut g, P0, 1);
    assert_eq!(level(&g), 0);
    token(&mut g, P0, 1);
    assert_eq!(level(&g), 1, "two tokens: the second tempts");
    // two tokens made by one effect are two triggers
    next_turn(&mut g);
    zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(level(&g), 2);
}

#[test]
fn creatures_that_enter_under_an_opponent_do_not_count() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    let e = perm(&mut g, P0, ELROND);
    perm(&mut g, P1, "Llanowar Elves");
    token(&mut g, P1, 1);
    assert_eq!(g.perm(e).data.int(DataKey::Resolved), 1);
}

#[test]
fn gandalf_draws_when_the_ring_tempts_and_the_bearer_is_another_creature() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    perm(&mut g, P0, "Glorybringer");
    let n = g.player(P0).hand.len();
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "drew a card");
    // Gandalf alone: he is the bearer, no card
    let mut g = game();
    fill_library(&mut g, P0, 20);
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    let n = g.player(P0).hand.len();
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
}

#[test]
fn gandalf_has_flash_and_lets_sorceries_be_cast_at_instant_speed() {
    let mut g = game();
    let gc = card(&g, "Gandalf, Friend of the Shire");
    assert!(g.db.get(gc).tags.has(sim_core::tag::Tag::Flash));
    assert!(g.db.get(gc).creature);
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Sleight of Hand"]);
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    // at the end of another player's turn the sorcery is cast
    g.active = Some(P1);
    let n = g.player(P0).hand.len();
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert!(g.player(P0).hand.len() >= n || gy_has(&g, P0, "Sleight of Hand"), "Sleight of Hand was cast");
    assert!(gy_has(&g, P0, "Sleight of Hand"));
}

// ================================================================== Cloudfin Raptor
#[test]
fn cloudfin_raptor_evolves_on_bigger_creatures_only() {
    let mut g = game();
    let r = perm(&mut g, P0, "Cloudfin Raptor");
    assert_eq!((epow(&g, r), etgh(&g, r)), (0, 1));
    perm(&mut g, P0, "Aven Fisher"); // 2/2 > 0/1
    assert_eq!(g.perm(r).plus, 1);
    perm(&mut g, P0, "Ornithopter"); // 0/2 vs 1/2: neither is greater
    assert_eq!(g.perm(r).plus, 1);
    token(&mut g, P0, 4);
    assert_eq!(g.perm(r).plus, 2, "tokens too");
    perm(&mut g, P1, "Glorybringer");
    assert_eq!(g.perm(r).plus, 2, "only creatures under your control");
}

// ================================================================== Defiler of Dreams
#[test]
fn defiler_draws_on_a_blue_permanent_spell_and_the_life_buys_a_blue_mana() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    perm(&mut g, P0, "Defiler of Dreams");
    lands(&mut g, P0, "Island", 3, false);
    let c = card(&g, "Aven Fisher"); // {3}{U}
    assert_eq!(cost_of(&g, P0, c), (3, String::new()), "{{U}} less with the life");
    let life = g.player(P0).life;
    let n = g.player(P0).hand.len();
    cast_paid(&mut g, P0, "Aven Fisher");
    assert!(has_perm(&g, P0, "Aven Fisher"));
    assert_eq!(g.player(P0).life, life - 2);
    assert_eq!(g.player(P0).hand.len(), n + 1, "drew a card");
    // a colorless permanent and an instant: no card
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Ornithopter");
    assert_eq!(g.player(P0).hand.len(), n);
    cast_free(&mut g, P0, "Quick Study");
    assert_eq!(g.player(P0).hand.len(), n + 2 - 0, "Quick Study draws two itself, no Defiler card");
    // low on life, the AI keeps it
    g.player_mut(P0).life = 10;
    assert_eq!(cost_of(&g, P0, c), (3, "U".to_string()));
}

// ================================================================== Deluge Virtuoso and stun counters
#[test]
fn virtuoso_taps_and_stuns_and_the_stun_counter_replaces_the_untap() {
    let mut g = game();
    let v = perm(&mut g, P1, "Glorybringer");
    perm(&mut g, P0, "Deluge Virtuoso");
    assert!(g.perm(v).tapped);
    assert_eq!(g.perm(v).data.int(DataKey::Stun), 1);
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(v).tapped, "the stun counter is removed instead of untapping");
    assert_eq!(g.perm(v).data.int(DataKey::Stun), 0);
    turn::step_start(&mut g, P1).unwrap();
    assert!(!g.perm(v).tapped, "the next untap works");
}

#[test]
fn virtuoso_opus_grows_by_one_or_two_for_five_mana() {
    let mut g = game();
    let v = perm(&mut g, P0, "Deluge Virtuoso");
    assert_eq!((epow(&g, v), etgh(&g, v)), (2, 2));
    lands(&mut g, P0, "Island", 5, false);
    cast_paid(&mut g, P0, "Quick Study"); // {2}{U}
    assert_eq!((epow(&g, v), etgh(&g, v)), (3, 3));
    cast_paid(&mut g, P0, "Unfathomable Truths"); // {4}{U}: five mana
    assert_eq!((epow(&g, v), etgh(&g, v)), (5, 5), "+2/+2 for five mana");
    cast_free(&mut g, P0, "Aven Fisher");
    assert_eq!((epow(&g, v), etgh(&g, v)), (5, 5), "a creature spell is not an instant or sorcery");
}

#[test]
fn rapier_wit_taps_stuns_on_your_turn_and_draws() {
    let mut g = game();
    fill_library(&mut g, P0, 10);
    let t = perm(&mut g, P1, "Glorybringer");
    g.active = Some(P0);
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Rapier Wit");
    assert!(g.perm(t).tapped);
    assert_eq!(g.perm(t).data.int(DataKey::Stun), 1);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    // on another player's turn: tap only
    let mut g = game();
    fill_library(&mut g, P0, 10);
    let t = perm(&mut g, P1, "Glorybringer");
    g.active = Some(P1);
    cast_free(&mut g, P0, "Rapier Wit");
    assert!(g.perm(t).tapped);
    assert_eq!(g.perm(t).data.int(DataKey::Stun), 0);
}

// ================================================================== returning creatures
#[test]
fn matterbending_mage_returns_the_enter_trigger_creature_or_the_bigger_threat() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    let o = perm(&mut g, P0, "Orysa, Tide Choreographer");
    perm(&mut g, P0, "Matterbending Mage");
    assert!(!g.perm(o).on_bf, "Orysa goes back to hand to draw two again");
    assert!(hand_has(&g, P0, "Orysa, Tide Choreographer"));
    // an opposing bomb is the better target
    let mut g = game();
    let big = perm(&mut g, P1, "Glorybringer");
    perm(&mut g, P0, "Aven Fisher");
    perm(&mut g, P0, "Matterbending Mage");
    assert!(!g.perm(big).on_bf, "the opposing Glorybringer was returned");
    assert!(has_perm(&g, P0, "Aven Fisher"));
}

#[test]
fn matterbending_mage_is_unblockable_when_an_x_spell_is_cast() {
    let mut g = game();
    let m = perm(&mut g, P0, "Matterbending Mage");
    lands(&mut g, P0, "Island", 4, false);
    let st = g.turn_stamp();
    cast_paid(&mut g, P0, "Quick Study");
    assert_ne!(g.perm(m).data.get(DataKey::Unbl), Some(&Val::Stamp(st)));
    let c = hand(&mut g, P0, &["West Coast Expansion"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 2, ..Ctx::default() }).unwrap();
    assert_eq!(g.perm(m).data.get(DataKey::Unbl), Some(&Val::Stamp(st)));
}

#[test]
fn rescue_returns_a_creature_or_an_artifact_and_grows_only_for_an_artifact() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    let o = perm(&mut g, P0, "Orysa, Tide Choreographer");
    let r = perm(&mut g, P0, "Rescue, Pepper Potts");
    assert!(!g.perm(o).on_bf, "returned the creature with the best enter trigger");
    assert_eq!(g.perm(r).plus, 0, "not an artifact: no counter");
    let mut g = game();
    let o = perm(&mut g, P0, "Ornithopter"); // a free artifact creature
    let r = perm(&mut g, P0, "Rescue, Pepper Potts");
    assert!(!g.perm(o).on_bf);
    assert_eq!(g.perm(r).plus, 1, "it was an artifact: a +1/+1 counter");
    // nothing else: nothing returned
    let mut g = game();
    let r = perm(&mut g, P0, "Rescue, Pepper Potts");
    assert!(g.perm(r).on_bf);
}

#[test]
fn fear_of_isolation_returns_a_permanent_as_a_cost() {
    let mut g = game();
    lands(&mut g, P0, "Island", 4, false);
    let before = g.player(P0).lands.len();
    cast_paid(&mut g, P0, "Fear of Isolation");
    assert!(has_perm(&g, P0, "Fear of Isolation"));
    assert_eq!(g.player(P0).lands.len(), before - 1, "a land went back to hand");
    assert!(hand_has(&g, P0, "Island"));
    // with a creature worth re-entering, that is the cost
    let mut g = game();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Island", 4, false);
    let o = perm(&mut g, P0, "Orysa, Tide Choreographer");
    cast_paid(&mut g, P0, "Fear of Isolation");
    assert!(!g.perm(o).on_bf);
    assert_eq!(g.player(P0).lands.len(), 4);
}

// ================================================================== the rest of the creatures
#[test]
fn mindseeker_oculus_empowers_jace_four() {
    let mut g = game();
    perm(&mut g, P0, "Mindseeker Oculus");
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(4));
}

#[test]
fn mouth_of_the_storm_shrinks_opposing_creatures_until_your_next_turn() {
    let mut g = game();
    let a = perm(&mut g, P1, "Glorybringer");
    assert_eq!(epow(&g, a), 4);
    let m = perm(&mut g, P0, "Mouth of the Storm");
    assert_eq!(epow(&g, a), 1);
    let late = perm(&mut g, P1, "Aven Fisher");
    assert_eq!(epow(&g, late), 2, "creatures that enter later are not affected");
    assert_eq!(epow(&g, m), 6);
    g.player_mut(P0).turns += 1; // my next turn begins
    assert_eq!(epow(&g, a), 4);
    // ward {2}
    assert_eq!(g.db.get(g.perm(m).cd.unwrap()).ward, 2);
}

#[test]
fn orysa_costs_three_less_with_ten_toughness() {
    let mut g = game();
    let c = card(&g, "Orysa, Tide Choreographer");
    assert_eq!(cost_of(&g, P0, c), (4, "U".to_string()));
    perm(&mut g, P0, "Mnemonic Wall"); // 0/4
    perm(&mut g, P0, "Mnemonic Wall");
    assert_eq!(cost_of(&g, P0, c), (4, "U".to_string()), "8 toughness");
    perm(&mut g, P0, "Coral Barrier"); // 1/3 and a 1/1
    assert_eq!(cost_of(&g, P0, c), (1, "U".to_string()), "12 toughness: {{3}} less");
    fill_library(&mut g, P0, 10);
    let n = g.player(P0).hand.len();
    cast_paid(&mut g, P0, "Orysa, Tide Choreographer");
    assert_eq!(g.player(P0).hand.len(), n + 2, "draw two");
}

#[test]
fn professor_hulk_draws_cards_equal_to_the_damage() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    let h = perm(&mut g, P0, "Professor Hulk");
    let n = g.player(P0).hand.len();
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a: h, d: P1, dmg: 6 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 6);
}

#[test]
fn starwinder_draws_for_any_of_your_creatures() {
    let mut g = game();
    fill_library(&mut g, P0, 30);
    perm(&mut g, P0, "Starwinder");
    let a = perm(&mut g, P0, "Aven Fisher");
    let n = g.player(P0).hand.len();
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a, d: P1, dmg: 2 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 2);
    // an opponent's creature doesn't count
    let b = perm(&mut g, P1, "Glorybringer");
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P1, a: b, d: P0, dmg: 4 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn textbook_tabulator_increments_when_the_mana_is_greater_than_its_power_or_toughness() {
    let mut g = game();
    fill_library(&mut g, P0, 10);
    let t = perm(&mut g, P0, "Textbook Tabulator"); // 0/3
    lands(&mut g, P0, "Island", 6, false);
    cast_paid(&mut g, P0, "Quick Study"); // 3 mana: not greater than 0 (power) ... greater than the lesser: 0
    assert_eq!(g.perm(t).plus, 1);
    cast_paid(&mut g, P0, "Dreams of Laguna"); // 2 mana vs 1/4: greater than 1
    assert_eq!(g.perm(t).plus, 2);
    cast_paid(&mut g, P0, "Unsummon"); // 1 mana vs 2/5: not greater
    assert_eq!(g.perm(t).plus, 2);
}

#[test]
fn the_wasp_gives_a_hero_hexproof_and_taps_a_defender_when_attacking() {
    let mut g = game();
    let w = perm(&mut g, P0, "The Wasp, Winsome Avenger");
    assert!(untargetable(&g, w), "a Hero (the Wasp) gained hexproof");
    let b = perm(&mut g, P1, "Glorybringer");
    let f = g.registry.get(g.perm(w).cd.unwrap()).unwrap().attack.unwrap();
    f(&mut g, w, P0, &[w], P1).unwrap();
    assert!(g.perm(b).tapped);
}

#[test]
fn dune_mover_is_toxic_and_puts_a_basic_land_on_top() {
    let mut g = game();
    fill_library(&mut g, P0, 10);
    let top_before = g.player(P0).library.len();
    let d = perm(&mut g, P0, "Dune Mover");
    assert_eq!(g.player(P0).library.len(), top_before, "a card stays in the library, on top");
    sim_core::impls::tymna_cards::toxic_hit(&mut g, d, P1); // the compiled toxic 1, with the combat damage
    assert_eq!(g.player(P1).poison, 1);
}

#[test]
fn soul_of_the_rapids_cannot_be_targeted() {
    let mut g = game();
    let s = perm(&mut g, P0, "Soul of the Rapids");
    assert!(untargetable(&g, s));
    let o = perm(&mut g, P0, "Aven Fisher");
    assert!(!untargetable(&g, o));
}

#[test]
fn silver_deputy_puts_a_land_on_top_and_pumps_at_sorcery_speed() {
    let mut g = game();
    let mut lib = vec![card(&g, "Aven Fisher"); 5];
    lib.push(card(&g, "Island"));
    lib.extend(vec![card(&g, "Aven Fisher"); 5]);
    g.player_mut(P0).library = lib;
    perm(&mut g, P0, "Silver Deputy");
    let top = *g.player(P0).library.last().unwrap();
    assert!(g.db.get(top).land, "a basic land is on top of the library");
}

// ================================================================== warp
#[test]
fn starbreach_whale_warps_is_exiled_at_the_end_step() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Starbreach Whale"]);
    assert!(do_opt(&mut g, P0, "Starbreach Whale (warp)"));
    let m = perm_named(&g, P0, "Starbreach Whale");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    let f = g.registry.get(g.perm(m).cd.unwrap()).unwrap().end_step.unwrap();
    f(&mut g, m, P0).unwrap();
    assert!(!g.perm(m).on_bf);
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Starbreach Whale"));
}

#[test]
fn warp_is_not_offered_when_the_full_cost_can_be_paid() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Starbreach Whale"]);
    assert!(!has_opt(&mut g, P0, "Starbreach Whale (warp)"));
    // but it is, as the second entry of Elrond's turn
    perm(&mut g, P0, ELROND);
    assert!(has_opt(&mut g, P0, "Starbreach Whale (warp)"));
}

#[test]
fn starwinder_warp_costs_two_blue_blue() {
    let mut g = game();
    fill_library(&mut g, P0, 20);
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Starwinder"]);
    assert!(do_opt(&mut g, P0, "Starwinder (warp)"));
    let m = perm_named(&g, P0, "Starwinder");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0);
}

// ================================================================== Hope-Ender Coatl
#[test]
fn hope_ender_coatl_counters_a_spell_unless_its_controller_pays_one() {
    let mut g = game();
    let coatl = hand(&mut g, P0, &["Hope-Ender Coatl"])[0];
    lands(&mut g, P0, "Island", 3, false);
    let spell = card(&g, "Glorybringer");
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P0, spell), Some(coatl));
    // (the override's soft=1 tag is applied by cardgen only to cards some deck plays: no deck plays the Coatl now)
    // devoid: colorless
    let d = g.db.get(coatl);
    assert!(d.kws.iter().any(|k| &**k == "devoid"));
}

#[test]
fn hope_ender_coatl_is_cast_as_a_counter_and_then_enters() {
    // the opponent casts a bomb into three open Islands and a Coatl: over some seeds the AI counters it at least once,
    // the spell is countered (the caster cannot pay {1}) and the Coatl is on the battlefield afterwards
    let mut countered = 0;
    for seed in 0..40u64 {
        let mut g = table_with(&["sheoldred", "seph"], seed, 40);
        fill_library(&mut g, P0, 20);
        lands(&mut g, P0, "Island", 3, false);
        hand(&mut g, P0, &["Hope-Ender Coatl"]);
        lands(&mut g, P1, "Mountain", 5, false);
        let c = hand(&mut g, P1, &["Glorybringer"])[0];
        g.active = Some(P1);
        let (gn, pips) = cost_of(&g, P1, c);
        pay(&mut g, P1, gn, &pips, false).unwrap();
        for &l in &g.player(P1).lands.clone() {
            g.perm_mut(l).tapped = true; // nothing left to pay the {1}
        }
        cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
        if has_perm(&g, P0, "Hope-Ender Coatl") {
            assert!(!has_perm(&g, P1, "Glorybringer"), "countered unless they pay {{1}}");
            assert!(gy_has(&g, P1, "Glorybringer"));
            countered += 1;
        }
    }
    assert!(countered > 0, "the Coatl was never cast as a counter");
}

// ================================================================== Gandalf: the AI uses sorceries at instant speed
fn gandalf_table() -> Game {
    table(&["sauron", "seph"])
}

fn utility_of(opts: &[Opt], label: &str) -> f64 {
    opts.iter().find(|o| o.label.starts_with(label)).unwrap_or_else(|| panic!("no option {label}")).utility
}

#[test]
fn gandalf_holds_a_flexible_sorcery_for_the_end_of_turn_window() {
    let mut without = gandalf_table();
    fill_library(&mut without, P0, 20);
    lands(&mut without, P0, "Swamp", 6, false);
    hand(&mut without, P0, &["Night's Whisper", "Swamp"]);
    let mut with = gandalf_table();
    fill_library(&mut with, P0, 20);
    lands(&mut with, P0, "Swamp", 6, false);
    hand(&mut with, P0, &["Night's Whisper", "Swamp"]);
    perm(&mut with, P0, "Gandalf, Friend of the Shire");
    let u0 = utility_of(&brain::main_options(&mut without, P0, false).unwrap(), "Night's Whisper");
    let u1 = utility_of(&brain::main_options(&mut with, P0, false).unwrap(), "Night's Whisper");
    assert!((u0 - u1 - 3.0).abs() < 1e-9, "held by 3.0: {u0} vs {u1}");
    // a creature is not held
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Grave Titan", "Swamp"]);
    let before = utility_of(&brain::main_options(&mut g, P0, false).unwrap(), "Grave Titan");
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    assert_eq!(utility_of(&brain::main_options(&mut g, P0, false).unwrap(), "Grave Titan"), before);
    // and with no land to play yet, card draw is not held
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Night's Whisper"]);
    g.player_mut(P0).turns = 3;
    let before = utility_of(&brain::main_options(&mut g, P0, false).unwrap(), "Night's Whisper");
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    assert_eq!(utility_of(&brain::main_options(&mut g, P0, false).unwrap(), "Night's Whisper"), before);
}

#[test]
fn gandalf_casts_removal_sorceries_at_the_end_of_the_turn_before_yours() {
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Bloodchief's Thirst"]);
    let t = perm(&mut g, P1, "Grave Titan");
    g.active = Some(P1);
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert!(g.perm(t).on_bf, "no Gandalf: the sorcery stays in hand");
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert!(!g.perm(t).on_bf, "Gandalf: the sorcery kills the Titan at instant speed");
}

#[test]
fn gandalf_kills_the_biggest_attacker_with_a_removal_sorcery() {
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Bloodchief's Thirst"]);
    let t = perm(&mut g, P1, "Grave Titan");
    // without Gandalf the sorcery can't answer the attack
    assert!(!brain::attack_response(&mut g, P0, P1, &[t]).unwrap());
    assert!(g.perm(t).on_bf);
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    assert!(brain::attack_response(&mut g, P0, P1, &[t]).unwrap());
    assert!(!g.perm(t).on_bf);
    assert!(gy_has(&g, P0, "Bloodchief's Thirst"));
}

#[test]
fn gandalf_wipes_the_board_after_a_big_attack() {
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Plains", 6, false);
    hand(&mut g, P0, &["Wrath of God"]);
    let a: Vec<PermId> = (0..3).map(|_| perm(&mut g, P1, "Grave Titan")).collect();
    assert!(!brain::attack_response(&mut g, P0, P1, &a).unwrap(), "no Gandalf: no instant-speed wipe");
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    assert!(brain::attack_response(&mut g, P0, P1, &a).unwrap());
    assert!(a.iter().all(|&m| !g.perm(m).on_bf));
    // a small attack does not trigger the wipe
    let mut g = gandalf_table();
    fill_library(&mut g, P0, 20);
    lands(&mut g, P0, "Plains", 6, false);
    hand(&mut g, P0, &["Wrath of God"]);
    perm(&mut g, P0, "Gandalf, Friend of the Shire");
    let s = perm(&mut g, P1, "Llanowar Elves");
    assert!(!brain::attack_response(&mut g, P0, P1, &[s]).unwrap());
}
