//! Tier 6 "godo" batch (impls/t6_godo.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

fn tbl() -> Game {
    table(&["godo", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

#[allow(dead_code)]
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn on(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
}

fn attack(g: &mut Game, src: PermId, p: PlayerId, atk: &[PermId], d: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().attack.unwrap();
    f(g, src, p, atk, d).unwrap();
}

// ======================================================== Godo, Bandit Warlord
#[test]
fn godo_enters_and_fetches_helm_onto_the_battlefield() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Helm of the Host"), "Equipment put onto the battlefield, not the hand");
    assert!(!g.player(P0).hand.contains(&card(&g, "Helm of the Host")));
    assert!(!g.player(P0).library.contains(&card(&g, "Helm of the Host")));
    let _ = godo;
}

#[test]
fn godo_fetches_hammer_first_with_a_copier_in_hand() {
    let mut g = tbl();
    hand(&mut g, P0, &["Twinflame"]);
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Hammer of Nazahn"));
    assert!(!on(&g, P0, "Helm of the Host"));
}

#[test]
fn godo_then_fetches_hammer_once_helm_is_out() {
    let mut g = tbl();
    perm(&mut g, P0, "Helm of the Host");
    perm(&mut g, P0, "Godo, Bandit Warlord");
    assert!(on(&g, P0, "Hammer of Nazahn"), "Helm already out: the next Equipment is Hammer");
}

#[test]
fn godo_first_attack_untaps_and_adds_one_combat_once_a_turn() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(godo).tapped = true; // attacking tapped it
    assert_eq!(g.player(P0).extra_combats, 0);
    attack(&mut g, godo, P0, &[godo], P1);
    assert!(!g.perm(godo).tapped, "untaps");
    assert_eq!(g.player(P0).extra_combats, 1, "one additional combat phase");
    g.perm_mut(godo).tapped = true;
    attack(&mut g, godo, P0, &[godo], P1);
    assert_eq!(g.player(P0).extra_combats, 1, "only the first attack each turn");
    assert!(g.perm(godo).tapped);
}

#[test]
fn godo_attack_trigger_needs_godo_to_attack() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let other = token(&mut g, P0, 2);
    attack(&mut g, godo, P0, &[other], P1);
    assert_eq!(g.player(P0).extra_combats, 0);
}

// ======================================================== Hammer of Nazahn
fn attached_to(g: &Game, e: PermId) -> Option<PermId> {
    g.perm(e).attached
}

#[test]
fn hammer_attaches_itself_when_it_enters() {
    let mut g = tbl();
    let t = token(&mut g, P0, 2);
    let h = perm(&mut g, P0, "Hammer of Nazahn");
    assert_eq!(attached_to(&g, h), Some(t), "Hammer attaches itself to a creature you control");
    assert_eq!(sim_core::engine::values::epow(&g, t), 4, "+2/+0");
}

#[test]
fn hammer_attaches_another_equipment_that_enters_to_godo() {
    let mut g = tbl();
    let other = token(&mut g, P0, 4);
    perm(&mut g, P0, "Hammer of Nazahn");
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord"); // its fetch finds Helm, which Hammer attaches
    let helm = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Helm of the Host")).unwrap();
    assert_eq!(attached_to(&g, helm), Some(godo), "Godo is preferred over a bigger creature ({other:?})");
}

#[test]
fn hammer_ignores_opponents_equipment_and_noncreature_boards() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hammer of Nazahn");
    assert_eq!(attached_to(&g, h), None, "no creature to attach to");
    let e = perm(&mut g, P1, "Sword of Fire and Ice");
    let t = token(&mut g, P0, 2);
    assert_eq!(attached_to(&g, e), None);
    assert_eq!(attached_to(&g, h), None, "an opponent's Equipment entering is not 'you control'");
    let _ = t;
}

// ======================================================== Magnetic Theft
fn cast_resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, p, c, &sim_core::state::Ctx::default()).unwrap();
}

fn prio(g: &Game, p: PlayerId, name: &str) -> i32 {
    let c = card(g, name);
    g.registry.get(c).unwrap().prio.unwrap()(g, p, c)
}

#[test]
fn magnetic_theft_attaches_a_loose_equipment_to_the_creature() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord"); // Helm comes out unattached (no Hammer)
    let helm = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Helm of the Host")).unwrap();
    assert_eq!(attached_to(&g, helm), None);
    assert!(prio(&g, P0, "Magnetic Theft") > 0, "worth casting");
    cast_resolve(&mut g, P0, "Magnetic Theft");
    assert_eq!(attached_to(&g, helm), Some(godo));
    assert_eq!(prio(&g, P0, "Magnetic Theft"), 0, "nothing left to move");
}

#[test]
fn magnetic_theft_needs_a_creature_and_an_equipment() {
    let mut g = tbl();
    assert_eq!(prio(&g, P0, "Magnetic Theft"), 0);
    perm(&mut g, P0, "Hammer of Nazahn");
    assert_eq!(prio(&g, P0, "Magnetic Theft"), 0, "no creature");
    cast_resolve(&mut g, P0, "Magnetic Theft"); // fizzles harmlessly
}

// ======================================================== Sting, the Glinting Dagger
fn each_combat(g: &mut Game, src: PermId, p: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    g.registry.get(c).unwrap().each_combat.unwrap()(g, src, p).unwrap()
}

#[test]
fn sting_untaps_the_equipped_creature_at_the_beginning_of_every_combat() {
    let mut g = tbl();
    let t = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Sting, the Glinting Dagger");
    g.perm_mut(s).attached = Some(t);
    assert_eq!(sim_core::engine::values::epow(&g, t), 3, "+1/+1");
    g.perm_mut(t).tapped = true;
    each_combat(&mut g, s, P1);
    assert!(!g.perm(t).tapped, "untapped at an opponent's combat too");
    g.perm_mut(t).tapped = true;
    each_combat(&mut g, s, P0);
    assert!(!g.perm(t).tapped, "untapped at its controller's combat");
}

#[test]
fn sting_unattached_does_nothing() {
    let mut g = tbl();
    let t = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Sting, the Glinting Dagger");
    g.perm_mut(t).tapped = true;
    each_combat(&mut g, s, P0);
    assert!(g.perm(t).tapped);
}

// ======================================================== Goblin Welder
fn welder_swap(g: &mut Game, w: PermId) -> bool {
    let o = options(g, w, P0, Some(false));
    if o.is_empty() {
        return false;
    }
    sim_core::ai::act::perform(g, P0, o[0].act.as_ref().unwrap()).unwrap()
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
}

#[test]
fn welder_swaps_a_spent_rock_for_helm_in_the_graveyard() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Goblin Welder");
    let rock = perm(&mut g, P0, "Sol Ring");
    to_gy(&mut g, P0, "Helm of the Host");
    assert!(welder_swap(&mut g, w));
    assert!(g.perm(w).tapped, "{{T}}");
    assert!(!g.perm(rock).on_bf, "the artifact was sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Sol Ring")));
    assert!(on(&g, P0, "Helm of the Host"), "returned to the battlefield");
    assert!(!g.player(P0).gy.contains(&card(&g, "Helm of the Host")));
}

#[test]
fn welder_wont_trade_down_or_when_summoning_sick() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Goblin Welder");
    perm(&mut g, P0, "Helm of the Host");
    to_gy(&mut g, P0, "Sol Ring");
    assert!(options(&mut g, w, P0, Some(false)).is_empty(), "Sol Ring is not worth Helm");
    let mut g = tbl();
    let c = take(&mut g, P0, "Goblin Welder");
    let sick = sim_core::engine::zones::enter(&mut g, P0, c, Default::default()).unwrap();
    perm(&mut g, P0, "Sol Ring");
    to_gy(&mut g, P0, "Helm of the Host");
    assert!(options(&mut g, sick, P0, Some(false)).is_empty(), "summoning sick: {{T}} is a cost");
}

// ======================================================== Goblin Engineer
#[test]
fn engineer_puts_helm_in_the_graveyard_when_a_welder_is_out() {
    let mut g = tbl();
    perm(&mut g, P0, "Goblin Welder");
    perm(&mut g, P0, "Goblin Engineer");
    assert!(g.player(P0).gy.contains(&card(&g, "Helm of the Host")), "searched into the graveyard");
    assert!(!on(&g, P0, "Helm of the Host"));
    assert!(!g.player(P0).library.contains(&card(&g, "Helm of the Host")));
}

#[test]
fn engineer_without_a_welder_takes_cursed_mirror() {
    let mut g = tbl();
    perm(&mut g, P0, "Goblin Engineer");
    assert!(g.player(P0).gy.contains(&card(&g, "Cursed Mirror")));
    assert_eq!(g.player(P0).gy.len(), 1);
}

#[test]
fn engineer_sacrifices_a_rock_to_return_a_cheap_artifact_for_r() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let e = perm(&mut g, P0, "Goblin Engineer"); // finds Cursed Mirror
    let rock = perm(&mut g, P0, "Mox Amber");
    g.perm_mut(rock).tapped = true;
    let o = options(&mut g, e, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(on(&g, P0, "Cursed Mirror"), "returned to the battlefield");
    assert!(!g.perm(rock).on_bf, "sacrificed as a cost");
    assert!(g.perm(e).tapped);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "paid {{R}}: one Mountain left, plus Cursed Mirror taps for {{R}}");
}

#[test]
fn engineer_ability_returns_only_mana_value_three_or_less() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let e = perm(&mut g, P0, "Goblin Engineer");
    g.player_mut(P0).gy.clear();
    to_gy(&mut g, P0, "Helm of the Host"); // mana value 4
    perm(&mut g, P0, "Sol Ring");
    assert!(options(&mut g, e, P0, Some(false)).is_empty());
}

// ======================================================== Tezzeret, Cruel Captain
fn walker_opts(g: &mut Game, src: PermId) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, P0, Some(false)).unwrap()
}

#[test]
fn tezzeret_enters_with_four_and_gains_loyalty_per_artifact() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    assert_eq!(g.perm(t).loyalty, Some(4));
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(g.perm(t).loyalty, Some(5), "an artifact you control entered");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, "Goblin Welder");
    assert_eq!(g.perm(t).loyalty, Some(5), "an opponent's artifact or a non-artifact does not count");
}

#[test]
fn tezzeret_zero_untaps_a_spent_rock() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    let rock = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(rock).tapped = true;
    let o = walker_opts(&mut g, t);
    let zero = o.iter().find(|o| o.label.contains("+0")).expect("a 0 ability");
    assert!(sim_core::ai::act::perform(&mut g, P0, zero.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(rock).tapped, "untapped");
    assert_eq!(g.perm(t).loyalty, Some(5), "loyalty unchanged by 0 (4, plus 1 for Sol Ring entering)");
}

#[test]
fn tezzeret_minus_three_tutors_a_cheap_artifact_to_hand() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    let o = walker_opts(&mut g, t);
    let m3 = o.iter().find(|o| o.label.contains("-3")).expect("-3 available at 4 loyalty");
    let before = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, m3.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), before + 1);
    let c = *g.player(P0).hand.last().unwrap();
    assert!(g.db.get(c).types.has(sim_core::cards::Types::ARTIFACT) && g.db.get(c).cmc <= 1, "{}", g.db.get(c).name);
    assert_eq!(g.perm(t).loyalty, Some(1));
}

#[test]
fn tezzeret_has_no_minus_seven_yet() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    g.perm_mut(t).loyalty = Some(9);
    assert!(walker_opts(&mut g, t).iter().all(|o| !o.label.contains("-7")), "the emblem is an engine need");
}

// ======================================================== Party Thrasher
fn main1(g: &mut Game, src: PermId, p: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    g.registry.get(c).unwrap().main1.unwrap()(g, src, p).unwrap();
}

#[test]
fn thrasher_discards_a_spare_card_and_lets_you_play_one_of_the_top_two() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Party Thrasher");
    hand(&mut g, P0, &["Mountain", "Mountain", "Sol Ring"]);
    let (hand_before, lib_before, gy_before) = (g.player(P0).hand.len(), g.player(P0).library.len(), g.player(P0).gy.len());
    main1(&mut g, t, P0);
    assert_eq!(g.player(P0).gy.len(), gy_before + 1, "discarded a card");
    assert_eq!(g.player(P0).library.len(), lib_before - 2, "two cards exiled");
    assert_eq!(g.player(P0).hand.len(), hand_before - 1 + 1, "one discarded, one chosen");
    assert_eq!(g.player(P0).impulse.len(), 1, "playable this turn only");
    assert_eq!(g.player(P0).exile.len(), 1, "the other stays exiled");
    let c = g.player(P0).impulse[0];
    assert!(g.player(P0).hand.contains(&c));
}

#[test]
fn thrasher_does_nothing_without_a_spare_card_or_on_an_opponents_turn() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Party Thrasher");
    hand(&mut g, P0, &["Sol Ring"]);
    let lib = g.player(P0).library.len();
    main1(&mut g, t, P0);
    assert_eq!(g.player(P0).library.len(), lib, "may discard: the AI declines without a spare card");
    hand(&mut g, P0, &["Mountain", "Mountain"]);
    let lib = g.player(P0).library.len();
    main1(&mut g, t, P1);
    assert_eq!(g.player(P0).library.len(), lib, "only on its controller's first main phase");
}

// ======================================================== Vexing Bauble
fn may_cast(g: &Game, p: PlayerId, name: &str) -> bool {
    sim_core::engine::hooks::allowed(g, p, card(g, name), "hand")
}

#[test]
fn bauble_stops_spells_that_cost_no_mana_for_every_player() {
    let mut g = tbl();
    assert!(may_cast(&g, P0, "Mox Amber") && may_cast(&g, P1, "Mox Amber"));
    perm(&mut g, P0, "Vexing Bauble");
    assert!(!may_cast(&g, P0, "Mox Amber"), "its controller's free spells are countered too");
    assert!(!may_cast(&g, P1, "Lotus Petal"), "an opponent's too");
    assert!(may_cast(&g, P0, "Sol Ring"), "a spell that costs mana is fine");
    assert!(may_cast(&g, P1, "Mountain"), "lands are not spells");
}

#[test]
fn bauble_draws_for_one_when_sacrificed() {
    // the compiled {1},{T}, sacrifice: draw a card is still on the card
    let g = tbl();
    let c = card(&g, "Vexing Bauble");
    assert!(g.db.get(c).has_dsl());
}

// ======================================================== Hexing Squelcher
#[test]
fn squelcher_makes_its_controllers_spells_uncounterable() {
    let mut g = tbl();
    let c = card(&g, "Godo, Bandit Warlord");
    assert!(!sim_core::engine::hooks::any_uncounterable(&g, P0, c));
    perm(&mut g, P0, "Hexing Squelcher");
    assert!(sim_core::engine::hooks::any_uncounterable(&g, P0, c), "spells you control can't be countered");
    assert!(!sim_core::engine::hooks::any_uncounterable(&g, P1, card(&g, "Sol Ring")), "an opponent's spells can");
}

#[test]
fn squelcher_itself_cant_be_countered() {
    let g = tbl();
    assert!(g.db.get(card(&g, "Hexing Squelcher")).tag(sim_core::tag::Tag::Unc));
}

// ======================================================== Blood Moon
fn units(g: &Game, p: PlayerId) -> Vec<(String, u32)> {
    sim_core::engine::mana::mana_units(g, p, false).iter().map(|u| (format!("{:?}", u.cols), u.amt)).collect()
}

#[test]
fn blood_moon_makes_nonbasic_lands_tap_for_one_red() {
    let mut g = tbl();
    lands(&mut g, P1, "Ancient Tomb", 1, false);
    lands(&mut g, P1, "Island", 1, false);
    let before = sim_core::engine::mana::total_mana(&g, P1, false);
    assert_eq!(before, 3, "Tomb {{C}}{{C}} and an Island");
    perm(&mut g, P0, "Blood Moon");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P1, false), 2, "Tomb makes one mana, the Island is basic");
    assert!(sim_core::engine::mana::can_pay(&g, P1, 0, "R", false), "the Tomb taps for R");
    assert!(sim_core::engine::mana::can_pay(&g, P1, 0, "U", false), "a basic Island still taps for U");
    let _ = units;
}

#[test]
fn blood_moon_is_cast_against_nonbasic_heavy_tables_only() {
    let mut g = tbl();
    assert_eq!(prio(&g, P0, "Blood Moon"), 0);
    lands(&mut g, P1, "Ancient Tomb", 1, false);
    lands(&mut g, P2, "City of Traitors", 1, false);
    lands(&mut g, P3, "Cavern of Souls", 1, false);
    assert!(prio(&g, P0, "Blood Moon") > 0);
}

// ======================================================== Defense Grid
#[test]
fn defense_grid_taxes_spells_cast_outside_their_casters_turn() {
    let mut g = tbl();
    let sol = card(&g, "Sol Ring");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P1, sol).0, 1);
    perm(&mut g, P0, "Defense Grid");
    assert_eq!(g.active, Some(P0));
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, sol).0, 1, "on its caster's own turn: no tax");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P1, sol).0, 4, "an opponent's spell on P0's turn costs {{3}} more");
    g.active = Some(P1);
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, sol).0, 4, "the Grid's own controller pays too on another turn");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P1, sol).0, 1);
}

// ======================================================== Final Fortune / Last Chance
fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    let dest = f(g, p, c, &sim_core::state::Ctx::default()).unwrap();
    assert_eq!(&*dest, "gy");
}

#[test]
fn final_fortune_queues_an_extra_turn_and_the_caster_loses_at_its_end_step() {
    let mut g = tbl();
    resolve(&mut g, P0, "Final Fortune");
    assert_eq!(g.extra_turns, vec![(P0, true)]);
    assert!(g.player(P0).alive, "nothing is lost until the extra turn ends");
    let turns = g.player(P0).turns;
    sim_core::engine::turn::run_extra_turns(&mut g).unwrap();
    assert_eq!(g.player(P0).turns, turns + 1, "one extra turn was played");
    assert!(!g.player(P0).alive, "P0 loses at the extra turn's end step");
    assert!(g.extra_turns.is_empty());
    assert!(g.player(P1).alive);
}

#[test]
fn last_chance_does_the_same() {
    let mut g = tbl();
    resolve(&mut g, P0, "Last Chance");
    assert_eq!(g.extra_turns, vec![(P0, true)]);
    sim_core::engine::turn::run_extra_turns(&mut g).unwrap();
    assert!(!g.player(P0).alive);
}

#[test]
fn a_plain_extra_turn_is_a_normal_turn_with_no_loss_and_the_last_queued_goes_first() {
    let mut g = tbl();
    sim_core::engine::turn::add_extra_turn(&mut g, P0, false);
    sim_core::engine::turn::add_extra_turn(&mut g, P1, false);
    let (t0, t1) = (g.player(P0).turns, g.player(P1).turns);
    sim_core::engine::turn::run_extra_turns(&mut g).unwrap();
    assert_eq!((g.player(P0).turns, g.player(P1).turns), (t0 + 1, t1 + 1));
    assert!(g.player(P0).alive && g.player(P1).alive);
    assert_eq!(g.active, Some(P0), "P1's turn (queued last) came first, then P0's");
    assert!(g.doomed_turn.is_none());
}

#[test]
fn an_extra_turn_in_the_round_loop_follows_the_current_turn() {
    let mut g = tbl();
    sim_core::engine::turn::add_extra_turn(&mut g, P1, false);
    let t1 = g.player(P1).turns;
    sim_core::engine::turn::play_turn(&mut g, P0).unwrap();
    assert_eq!(g.active, Some(P1), "after P0's turn, P1 took the queued extra turn");
    assert_eq!(g.player(P1).turns, t1 + 1);
}

#[test]
fn the_ai_casts_the_extra_turn_spells_only_when_the_extra_turn_wins() {
    let mut g = tbl();
    assert_eq!(prio(&g, P0, "Final Fortune"), 0, "empty table: the extra turn wins nothing");
    // Godo in the command zone, eight lands all tapped: not castable now, castable after the untap
    lands(&mut g, P0, "Mountain", 8, true);
    assert_eq!(prio(&g, P0, "Final Fortune"), 85);
    assert_eq!(prio(&g, P0, "Last Chance"), 85);
    g.doomed_turn = Some(P0);
    assert_eq!(prio(&g, P0, "Final Fortune"), 0, "never on a turn that already ends in a loss");
    g.doomed_turn = None;
    sim_core::engine::turn::add_extra_turn(&mut g, P0, true);
    assert_eq!(prio(&g, P0, "Final Fortune"), 0, "one is already queued");
}

#[test]
fn the_ai_does_not_cast_it_when_godo_can_win_this_turn() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 12, false);
    assert_eq!(prio(&g, P0, "Final Fortune"), 0, "plenty of mana now: Godo goes off this turn");
}

#[test]
fn the_ai_casts_it_when_godo_and_helm_are_out_but_summoning_sick() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    perm(&mut g, P0, "Helm of the Host");
    g.perm_mut(godo).sick = true;
    assert_eq!(prio(&g, P0, "Final Fortune"), 85);
    g.perm_mut(godo).sick = false;
    assert_eq!(prio(&g, P0, "Final Fortune"), 0, "Godo can attack now");
}

// ======================================================== enters tapped (engine mechanism)
#[test]
fn helm_token_copy_of_godo_enters_tapped_under_an_opponents_blind_obedience_and_cannot_attack() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let helm = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Helm of the Host"))
        .unwrap();
    g.perm_mut(helm).attached = Some(godo);
    // without the enchantment the copy is a new, hasty attacker
    let before = sim_core::cardcode::combat_start(&mut g, P0).unwrap();
    assert_eq!(before.len(), 1, "an untapped hasty copy attacks");
    assert!(!g.perm(before[0]).tapped);
    perm(&mut g, P1, "Blind Obedience");
    let after = sim_core::cardcode::combat_start(&mut g, P0).unwrap();
    assert!(after.is_empty(), "the copy entered tapped: it can't attack");
    let tapped_copies = g
        .player(P0)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).token && g.perm(m).on_bf && g.perm(m).tapped && g.perm(m).name.contains("Godo"))
        .count();
    assert_eq!(tapped_copies, 1, "the token copy of Godo is on the battlefield, tapped");
}
