//! Adversarial Oracle-text checks for Damnable Pact and Enduring Tenacity.
//! Existing happy-path tests live in cards_damnable_pact.rs and cards_enduring_tenacity.rs.

use sim_core::cards::Types;
use sim_core::control::Answer;
use sim_core::engine::{cast, control, life, mana, removal, values, zones};
use sim_core::hooks::{Action, Event, Opt};
use sim_core::human::script::ScriptHuman;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{combos, common, karn_cards, t2};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PACT: &str = "Damnable Pact";
const TEN: &str = "Enduring Tenacity";
const SHEO: &str = "Sheoldred, the Apocalypse";

fn game() -> Game {
    table(&["sheoldred", "seph", "cloud"])
}

fn pact(g: &mut Game, target: PlayerId, x: i32) {
    let c = hand(g, P0, &[PACT])[0];
    cast::cast_card(g, P0, c, "hand", Ctx { face: Some(target), x, ..Ctx::default() }).unwrap();
}

fn options(g: &mut Game) -> Vec<Opt> {
    let c = card(g, PACT);
    if !g.player(P0).hand.contains(&c) {
        hand(g, P0, &[PACT]);
    }
    g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P0, Some(false)).unwrap()
}

fn play(g: &mut Game, o: &Opt) -> bool {
    let Some(Action::Plan { f, arg }) = o.act else { panic!("expected Pact plan") };
    f(g, P0, arg).unwrap()
}

fn tenacities(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == TEN).collect()
}

fn returned(g: &mut Game) -> PermId {
    let old = perm(g, P0, TEN);
    zones::die(g, old, "destroy").unwrap();
    let back = tenacities(g, P0);
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], old);
    assert!(!g.is_creature(back[0]));
    back[0]
}

fn zone_count(g: &Game, c: CardId) -> usize {
    g.players
        .iter()
        .map(|p| {
            p.hand.iter().chain(p.library.iter()).chain(p.gy.iter()).chain(p.exile.iter()).filter(|&&x| x == c).count()
                + p.perms.iter().filter(|&&m| g.perm(m).on_bf && !g.perm(m).token && g.perm(m).cd == Some(c)).count()
        })
        .sum()
}

#[test]
fn pact_zero_does_not_attempt_a_draw_from_an_empty_library() {
    let mut g = game();
    g.player_mut(P1).library.clear();
    pact(&mut g, P1, 0);
    assert!(!g.player(P1).decked);
    assert!(g.player(P1).alive);
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn pact_overdraw_marks_draw_loss_and_still_applies_its_life_loss() {
    let mut g = game();
    g.player_mut(P1).library.truncate(2);
    pact(&mut g, P1, 5);
    assert_eq!(g.player(P1).hand.len(), 2);
    assert!(g.player(P1).decked);
    assert_eq!(g.player(P1).life, 35, "finish resolving before checking draw-loss SBAs");
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn pact_exact_library_size_is_not_a_failed_draw() {
    let mut g = game();
    g.player_mut(P1).library.truncate(2);
    pact(&mut g, P1, 2);
    assert!(g.player(P1).library.is_empty());
    assert!(!g.player(P1).decked);
    assert!(g.player(P1).alive);
}

#[test]
fn pact_illegal_sole_target_does_not_redirect_to_its_caster() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P1).life = 6;
    let o = options(&mut g);
    assert!(o[0].label.contains(&g.player(P1).name.to_string()));
    // A target can leave after it was chosen. Reject the stale plan or fizzle; never choose a different player.
    g.player_mut(P1).alive = false;
    play(&mut g, &o[0]);
    // BUG: pact_resolve replaces an eliminated target with the caster instead of letting the spell fizzle.
    assert_eq!(g.player(P0).life, 40);
    assert_eq!(g.player(P0).draw_n, 0);
}

#[test]
fn pact_notion_thief_replaces_each_draw_but_not_the_target_life_loss() {
    let mut g = game();
    perm(&mut g, P0, "Notion Thief");
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, TEN);
    pact(&mut g, P1, 3);
    assert_eq!(g.player(P0).hand.len(), 3);
    assert!(g.player(P1).hand.is_empty());
    assert_eq!(g.player(P0).life, 46);
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 3 - 6);
}

#[test]
fn pact_draws_finish_and_spell_life_loss_applies_before_lethal_draw_triggers() {
    let mut g = game();
    perm(&mut g, P0, "Underworld Dreams");
    g.player_mut(P1).life = 1;
    pact(&mut g, P1, 3);
    assert_eq!(g.player(P1).hand.len(), 3, "the draw instruction must finish");
    assert!(g.player(P1).life <= -2, "Pact itself loses 3 life before the draw triggers resolve");
    assert!(!g.player(P1).alive);
}

#[test]
fn pact_self_draw_triggers_each_of_three_gain_drains_once_per_card() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, TEN);
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    perm(&mut g, P0, "Sanguine Bond");
    pact(&mut g, P0, 3);
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 3 * 2 * 3);
    assert_eq!(g.bond_depth, 0);
}

#[test]
fn pact_jet_medallion_reduces_the_generic_x_payment() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    lands(&mut g, P0, "Swamp", 4, false);
    let o = options(&mut g);
    assert_eq!(o.len(), 1);
    assert!(play(&mut g, &o[0]));
    // BUG: cost_of clamps the discount at X=0 before pact_max_x adds X, losing Jet Medallion's reduction.
    assert_eq!(g.player(P0).hand.len(), 3, "{{3}}{{B}}{{B}} minus {{1}} costs four mana");
    assert_eq!(g.player(P0).life, 37);
}

#[test]
fn pact_krrik_black_pips_paid_with_life_are_actually_charged() {
    let mut g = game();
    let k = perm(&mut g, P0, "K'rrik, Son of Yawgmoth");
    lands(&mut g, P0, "Wastes", 4, false);
    let o = options(&mut g);
    assert_eq!(o.len(), 1);
    assert!(play(&mut g, &o[0]));
    assert_eq!(g.player(P0).hand.len(), 4, "four colourless mana pay X, life pays both black pips");
    assert_eq!(g.perm(k).plus, 1, "casting a black spell still grows K'rrik");
    // BUG: pact_plan calls pay_spell/cast_card directly and omits the four life owed for K'rrik's black pips.
    assert_eq!(g.player(P0).life, 32, "4 life for {{B}}{{B}}, then 4 life for Pact");
}

#[test]
fn pact_stale_plan_rejects_a_missing_hand_card_without_spending_mana() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 7, false);
    let o = options(&mut g);
    let c = card(&g, PACT);
    g.player_mut(P0).hand.retain(|&x| x != c);
    g.player_mut(P0).gy.push(c);
    assert!(!play(&mut g, &o[0]));
    assert!(ls.iter().all(|&m| !g.perm(m).tapped));
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn pact_stale_plan_rejects_spent_mana_without_casting() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 7, false);
    let o = options(&mut g);
    for m in ls {
        g.perm_mut(m).tapped = true;
    }
    assert!(!play(&mut g, &o[0]));
    assert!(g.player(P0).hand.contains(&card(&g, PACT)));
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn pact_plan_survives_sheoldred_removal_without_phantom_drain() {
    let mut g = game();
    let s = perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P1).life = 6;
    let o = options(&mut g);
    zones::exile_perm(&mut g, s).unwrap();
    assert!(play(&mut g, &o[0]));
    assert_eq!((g.player(P1).hand.len(), g.player(P1).life), (2, 4));
    assert!(g.player(P1).alive);
}

#[test]
fn pact_human_seat_has_no_ai_hand_option() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 7, false);
    g.humans.add(P0, std::sync::Arc::new(ScriptHuman::from_fn(|_| Answer::Decline)));
    assert!(options(&mut g).is_empty());
}

#[test]
fn pact_picks_lowest_living_opponent_at_a_four_player_table() {
    let mut g = table(&["sheoldred", "seph", "cloud", "karn"]);
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P1).life = 1;
    g.player_mut(P1).alive = false;
    g.player_mut(P2).life = 6;
    let o = options(&mut g);
    assert!(play(&mut g, &o[0]));
    assert!(!g.player(P2).alive);
    assert_eq!(g.player(PlayerId(3)).life, 40);
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn pact_ai_does_not_offer_a_drain_on_a_locked_life_total() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P1).life = 12;
    g.player_mut(P1).life_locked = true;
    // Also make the other opponent unavailable under the documented library filter.
    g.player_mut(P2).library.truncate(2);
    let o = options(&mut g);
    assert_eq!(o.len(), 1);
    assert!(play(&mut g, &o[0]));
    assert_eq!(g.player(P1).life, 12);
    // BUG: pact_options disables the kill bonus for life_locked but still scores 12 imaginary life loss as a drain.
    assert!(g.player(P1).hand.is_empty(), "the locked target should not receive five free cards");
}

#[test]
fn pact_ai_does_not_score_platinum_angel_as_a_kill() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P1, "Platinum Angel");
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P1).life = 3;
    let o = options(&mut g);
    assert_eq!(o.len(), 1);
    assert!(play(&mut g, &o[0]));
    assert!(g.player(P1).alive, "Platinum Angel prevents losing at zero life");
    // BUG: pact_options uses only life_locked to exclude kills, ignoring the supported can't-lose effect.
    assert!(o[0].utility < 8.0, "a surviving opponent was incorrectly awarded the +7 kill bonus: {}", o[0].utility);
}

#[test]
fn tenacity_disk_return_survives_the_same_wipe_and_occupies_one_zone() {
    let mut g = game();
    let c = take(&mut g, P0, TEN);
    let old = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    perm(&mut g, P1, "Sol Ring");
    karn_cards::run(&mut g, P1, karn_cards::Wipe::Disk, None).unwrap();
    assert!(!g.perm(old).on_bf);
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert!(!g.is_creature(back[0]));
    assert_eq!(zone_count(&g, c), 1);
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, 77);
}

#[test]
fn tenacity_return_is_not_a_creature_entering_for_soul_warden() {
    let mut g = game();
    perm(&mut g, P1, "Soul Warden");
    let old = perm(&mut g, P0, TEN);
    assert_eq!(g.player(P1).life, 41, "the original creature enters");
    zones::die(&mut g, old, "destroy").unwrap();
    assert_eq!(tenacities(&g, P0).len(), 1);
    // BUG: tenacity_dies sets NotCreature only after enter has already fired creature-enter triggers.
    assert_eq!(g.player(P1).life, 41, "the returned object enters as an enchantment, not as a creature");
}

#[test]
fn tenacity_zero_toughness_death_returns_without_old_modifiers() {
    let mut g = game();
    let old = perm(&mut g, P0, TEN);
    g.perm_mut(old).eot_pt = (-4, -4);
    life::check_state(&mut g).unwrap();
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], old);
    assert_eq!(g.perm(back[0]).eot_pt, (0, 0));
    assert!(!g.is_creature(back[0]));
}

#[test]
fn tenacity_actual_theft_returns_to_original_owner_and_drains_for_that_owner() {
    let mut g = game();
    let old = perm(&mut g, P0, TEN);
    assert!(control::steal(&mut g, P1, old, control::Duration::Permanent).unwrap());
    assert_eq!(g.perm(old).owner, P1, "engine owner field denotes current controller");
    assert_eq!(g.perm(old).orig, P0);
    zones::die(&mut g, old, "sac").unwrap();
    assert!(tenacities(&g, P1).is_empty());
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert!(!g.is_creature(back[0]));
    life::gain(&mut g, P1, 3).unwrap();
    assert_eq!(g.player(P0).life, 40);
    life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, 81);
    assert_eq!(zone_count(&g, card(&g, TEN)), 1);
}

#[test]
fn tenacity_rest_in_peace_exiles_instead_of_returning() {
    let mut g = game();
    perm(&mut g, P1, "Rest in Peace");
    let old = perm(&mut g, P0, TEN);
    zones::die(&mut g, old, "destroy").unwrap();
    assert!(tenacities(&g, P0).is_empty());
    assert!(g.player(P0).gy.is_empty());
    assert!(g.player(P0).exile.contains(&card(&g, TEN)));
}

#[test]
fn tenacity_token_copy_dies_without_returning_a_card() {
    let mut g = game();
    let c = card(&g, TEN);
    let t = zones::enter_token_copy(&mut g, P0, c).unwrap().unwrap();
    assert!(g.perm(t).token);
    zones::die(&mut g, t, "sac").unwrap();
    assert!(tenacities(&g, P0).is_empty());
    assert!(!g.player(P0).gy.contains(&c));
}

#[test]
fn tenacity_no_owner_card_self_death_hook_is_a_noop() {
    let mut g = game();
    let t = token(&mut g, P0, 4);
    // Exercise the missing-card guard separately from the token guard.
    g.perm_mut(t).token = false;
    assert!(g.perm(t).cd.is_none());
    let f = g.registry.get(card(&g, TEN)).unwrap().self_dies.unwrap();
    f(&mut g, t, t, "sac").unwrap();
    assert!(tenacities(&g, P0).is_empty());
    assert!(g.perm(t).on_bf);
}

#[test]
fn tenacity_doubled_dies_trigger_returns_only_once() {
    let mut g = game();
    perm(&mut g, P0, "Teysa Karlov");
    returned(&mut g);
    assert_eq!(zone_count(&g, card(&g, TEN)), 1);
    assert!(g.trig_queue.is_empty());
}

#[test]
fn tenacity_blink_resets_the_enchantment_to_a_creature_then_the_cycle_restarts() {
    let mut g = game();
    let back = returned(&mut g);
    let fresh = t2::blink(&mut g, P0, back).unwrap().unwrap();
    assert!(!g.perm(back).on_bf);
    assert!(g.is_creature(fresh));
    zones::die(&mut g, fresh, "destroy").unwrap();
    let again = tenacities(&g, P0);
    assert_eq!(again.len(), 1);
    assert!(!g.is_creature(again[0]));
    assert_eq!(zone_count(&g, card(&g, TEN)), 1);
}

#[test]
fn tenacity_bounce_and_tuck_never_trigger_a_return() {
    for tuck in [false, true] {
        let mut g = game();
        let t = returned(&mut g);
        if tuck {
            zones::tuck(&mut g, t).unwrap();
        } else {
            zones::bounce(&mut g, t).unwrap();
        }
        assert!(tenacities(&g, P0).is_empty());
        let c = card(&g, TEN);
        assert!(!g.player(P0).gy.contains(&c));
        assert_eq!(zone_count(&g, c), 1);
        if tuck {
            assert!(g.player(P0).library.contains(&c));
        } else {
            assert!(g.player(P0).hand.contains(&c));
        }
    }
}

#[test]
fn tenacity_reanimation_starts_a_new_creature_cycle() {
    let mut g = game();
    let back = returned(&mut g);
    zones::die(&mut g, back, "destroy").unwrap();
    assert!(tenacities(&g, P0).is_empty());
    let c = hand(&mut g, P0, &["Reanimate"])[0];
    let ten = card(&g, TEN);
    cast::cast_card(&mut g, P0, c, "hand", Ctx { rean_target: Some(ten), ..Ctx::default() }).unwrap();
    let fresh = tenacities(&g, P0);
    assert_eq!(fresh.len(), 1);
    assert!(g.is_creature(fresh[0]));
    assert_eq!(g.player(P0).life, 36);
    zones::die(&mut g, fresh[0], "sac").unwrap();
    assert!(!g.is_creature(tenacities(&g, P0)[0]));
    assert_eq!(zone_count(&g, ten), 1);
}

#[test]
fn tenacity_enchantment_death_does_not_gain_mikaeus_undying() {
    let mut g = game();
    let back = returned(&mut g);
    perm(&mut g, P0, "Mikaeus, the Unhallowed");
    assert!(!g.is_creature(back));
    zones::die(&mut g, back, "destroy").unwrap();
    // BUG: die_rest grants Mikaeus's undying using the printed creature flag even for returned noncreature Tenacity.
    assert!(tenacities(&g, P0).is_empty(), "Mikaeus gives undying only to other non-Human creatures");
    assert!(g.player(P0).gy.contains(&card(&g, TEN)));
}

#[test]
fn tenacity_enchantment_survives_creature_edicts_and_keeps_black_devotion() {
    let mut g = game();
    let back = returned(&mut g);
    zones::edict(&mut g, P0, false).unwrap();
    assert!(g.perm(back).on_bf);
    assert!(values::ptypes(&g, back).has(Types::ENCHANTMENT));
    assert!(!removal::legal_targets(&g, P1, "destroy", "c", false, None).contains(&back));
    let c = hand(&mut g, P0, &["Gray Merchant of Asphodel"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).life, 48, "two black pips on each permanent; two opponents drain four each");
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 8 - 8);
}

#[test]
fn tenacity_two_copies_drain_independently_without_recursion_leaks() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    perm(&mut g, P0, TEN);
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(g.player(P1).life + g.player(P2).life, 74);
    assert_eq!(g.bond_depth, 0);
}

#[test]
fn tenacity_leave_inventory_classifies_its_self_death_as_a_trigger() {
    let g = game();
    let imp = g.registry.get(card(&g, TEN)).unwrap();
    assert!(sim_core::engine::hooks::leave_hook_is_trigger(imp, Event::SelfDies));
    let rows: Vec<_> =
        include_str!("leaves_inventory.txt").lines().filter(|s| s.starts_with("Enduring Tenacity |")).collect();
    assert_eq!(rows.len(), 1);
    let fields: Vec<_> = rows[0].split('|').map(str::trim).collect();
    assert_eq!((&*fields[1], &*fields[2]), ("SelfDies", "T"));
}

#[test]
fn pact_cost_estimate_and_payment_have_no_unspent_mana_without_reducers() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 6, false);
    let o = options(&mut g);
    let c = card(&g, PACT);
    assert_eq!(mana::cost_of(&g, P0, c), (0, "BB".into()));
    assert!(play(&mut g, &o[0]));
    assert_eq!(g.player(P0).hand.len(), 4);
    assert!(ls.iter().all(|&m| g.perm(m).tapped));
}

#[test]
fn pact_stale_plan_rechecks_a_new_silence_before_spending_mana() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 7, false);
    let o = options(&mut g);
    let s = hand(&mut g, P1, &["Silence"])[0];
    cast::cast_card(&mut g, P1, s, "hand", Ctx::default()).unwrap();
    assert!(!cast::castable(&g, P0, card(&g, PACT), "hand"), "Silence forbids this cast");
    let did_play = play(&mut g, &o[0]);
    // BUG: pact_plan never rechecks castable, so it casts and spends mana after Silence resolves.
    assert!(!did_play, "opponents can't cast spells this turn");
    assert!(g.player(P0).hand.contains(&card(&g, PACT)));
    assert!(ls.iter().all(|&m| !g.perm(m).tapped));
}

#[test]
fn pact_target_gaining_hexproof_before_resolution_is_untouched() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    lands(&mut g, P0, "Swamp", 5, false);
    g.player_mut(P1).life = 6;
    let o = options(&mut g);
    // Shalai grants her controller hexproof after the AI has selected a legal opponent.
    perm(&mut g, P1, "Shalai, Voice of Plenty");
    assert!(values::player_hexproof_from(&g, P1, g.db.get(card(&g, PACT)).colors));
    play(&mut g, &o[0]);
    // BUG: pact_resolve checks alive but never checks whether the targeted player gained hexproof.
    assert_eq!((g.player(P1).hand.len(), g.player(P1).life), (0, 6));
}

#[test]
fn tenacity_lethal_damage_returns_an_undamaged_enchantment() {
    let mut g = game();
    let old = perm(&mut g, P0, TEN);
    life::removal_damage(&mut g, P1, None, old, 3, None).unwrap();
    assert!(!g.perm(old).on_bf);
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert!(!g.is_creature(back[0]));
    assert_eq!(zone_count(&g, card(&g, TEN)), 1);
}

#[test]
fn tenacity_ashnods_altar_sacrifice_returns_and_adds_two_mana() {
    let mut g = game();
    let altar = perm(&mut g, P0, "Ashnod's Altar");
    let old = perm(&mut g, P0, TEN);
    common::sac_through(&mut g, P0, altar, old).unwrap();
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert!(!g.is_creature(back[0]));
    assert_eq!(mana::total_mana(&g, P0, false), 2);
    assert_eq!(zone_count(&g, card(&g, TEN)), 1);
}

#[test]
fn tenacity_grave_pact_sees_only_the_creature_death() {
    let mut g = game();
    perm(&mut g, P0, "Grave Pact");
    let victim1 = token(&mut g, P1, 2);
    let victim2 = token(&mut g, P2, 2);
    let back = returned(&mut g);
    assert!(!g.perm(victim1).on_bf && !g.perm(victim2).on_bf);
    let survivor1 = token(&mut g, P1, 2);
    let survivor2 = token(&mut g, P2, 2);
    zones::die(&mut g, back, "destroy").unwrap();
    assert!(g.perm(survivor1).on_bf && g.perm(survivor2).on_bf);
}

#[test]
fn tenacity_enchantment_drains_the_total_exsanguinate_gain_once() {
    let mut g = game();
    returned(&mut g);
    let c = hand(&mut g, P0, &["Exsanguinate"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 3, ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).life, 46);
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 6 - 6);
    assert_eq!(g.bond_depth, 0);
}

#[test]
fn tenacity_returned_enchantment_exquisite_blood_combo_finishes_instead_of_stalling() {
    for keys in [&["sheoldred", "seph"][..], &["sheoldred", "seph", "cloud"][..]] {
        let mut g = table(keys);
        returned(&mut g);
        perm(&mut g, P0, "Exquisite Blood");
        let i = combos::combo_index("Bloodthirsty Conqueror").unwrap();
        assert!(combos::ready_to_go(&g, P0, i));
        assert!(combos::attempt(&mut g, P0, i as i64).unwrap());
        assert!(g.over);
        assert_eq!(g.winner, Some(P0));
        assert_eq!(g.bond_depth, 0);
    }
}

#[test]
fn tenacity_phased_out_does_not_trigger_life_gain_and_survives_disk() {
    let mut g = game();
    let t = perm(&mut g, P0, TEN);
    g.perm_mut(t).phased = true;
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, 80);
    karn_cards::run(&mut g, P1, karn_cards::Wipe::Disk, None).unwrap();
    assert!(g.perm(t).on_bf && g.perm(t).phased);
    assert_eq!(tenacities(&g, P0), vec![t]);
}

// ------------------------------------------------------------------ a clone of Enduring Tenacity is not the card
#[test]
fn a_clone_of_tenacity_dying_does_not_return_the_real_card_from_the_graveyard() {
    let mut g = game();
    perm(&mut g, P1, TEN);
    let image = perm(&mut g, P0, "Phantasmal Image");
    assert_eq!(&*g.perm(image).name, TEN, "the Image is a copy of Enduring Tenacity");
    assert!(g.perm(image).phys.is_some(), "its own card is the physical card");
    // the real Enduring Tenacity sits in P0's graveyard (as it would after being milled or discarded)
    let real = card(&g, TEN);
    g.player_mut(P0).gy.push(real);
    zones::die(&mut g, image, "destroy").unwrap();
    assert!(tenacities(&g, P0).is_empty(), "the clone dying brings nothing back");
    assert!(g.player(P0).gy.contains(&real), "the real card stays where it was");
    assert!(g.player(P0).gy.contains(&card(&g, "Phantasmal Image")), "the clone card goes to the graveyard");
}

#[test]
fn a_token_copy_of_tenacity_dying_returns_nothing() {
    let mut g = game();
    let old = perm(&mut g, P0, TEN);
    let real = card(&g, TEN);
    g.player_mut(P0).gy.push(real);
    g.perm_mut(old).token = true;
    zones::die(&mut g, old, "destroy").unwrap();
    assert!(tenacities(&g, P0).is_empty());
    assert!(g.player(P0).gy.contains(&real));
}

#[test]
fn the_real_tenacity_still_returns_with_a_clone_of_it_in_play() {
    let mut g = game();
    perm(&mut g, P1, TEN);
    let image = perm(&mut g, P0, "Phantasmal Image");
    assert!(g.perm(image).phys.is_some());
    let real = perm(&mut g, P0, TEN);
    assert!(g.perm(real).phys.is_none());
    zones::die(&mut g, real, "destroy").unwrap();
    let back: Vec<PermId> = tenacities(&g, P0).into_iter().filter(|&m| m != image).collect();
    assert_eq!(back.len(), 1, "the physical card comes back as an enchantment");
    assert!(!g.is_creature(back[0]));
}
