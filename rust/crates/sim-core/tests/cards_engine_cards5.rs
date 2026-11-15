//! Klauth's spell-only kept mana and Rivaz's independent graveyard permission and cast trigger.
mod play_common;
use play_common::*;
use sim_core::engine::pool::{Keep, Key, Kind, Pool, Restr};
use sim_core::engine::{cast, mana, stack, turn, ward, zones};
use sim_core::human::{human, legal, mana as hm};
use sim_core::state::{Ctx, DataKey, Step, Val};
use std::sync::Arc;

const KLAUTH: &str = "Klauth, Unrivaled Ancient";
const RIVAZ: &str = "Rivaz of the Claw";
const GOLD: &str = "Goldspan Dragon";

fn tbl() -> Game {
    let mut g = table(&["urdragon", "urza-mono-blue-artifacts", "kinnan-simic-mana-combo"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    let ur = card(&g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(ur, false);
    g
}

fn passes(g: &mut Game) {
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(|r| {
            if r.kind == "priority" {
                Answer::Action(json!({"do":"pass"}))
            } else if r.prompt.contains("which Rivaz") {
                Answer::Pick(0)
            } else {
                Answer::Decline
            }
        })),
    );
}

fn trigger_klauth(g: &mut Game) -> u32 {
    let k = perm(g, P0, KLAUTH);
    let second = perm(g, P0, GOLD);
    let expected = (sim_core::engine::values::epow(g, k) + sim_core::engine::values::epow(g, second)) as u32;
    let f = g.registry.get(card(g, KLAUTH)).unwrap().attack.unwrap();
    f(g, k, P0, &[k, second], P1).unwrap();
    expected
}

#[test]
fn klauth_pool_keys_preserve_restriction_duration_and_exact_spending() {
    let mut pool = Pool::default();
    let key = Key { restr: Restr::SpellOnly, kind: Kind::Any, keep: Keep::Turn };
    pool.add_kept_restr(Kind::Any, Restr::SpellOnly, 5);
    pool.add(Kind::Any, 2);
    assert_eq!(pool.total(), 7);
    assert_eq!(pool.get(Kind::Any), 2);
    assert_eq!(pool.spell(), 5);
    pool.take_kind(Kind::Any, 1);
    assert_eq!(pool.amount(key), 5);
    pool.take(key, 2);
    assert_eq!(pool.total(), 4);
    pool.end_step();
    assert_eq!(pool.total(), 3);
    pool.clear();
    assert!(pool.is_empty());
}

#[test]
fn klauth_ai_pays_spells_first_but_never_abilities_equip_ward_or_taxes() {
    let mut g = tbl();
    let amount = trigger_klauth(&mut g);
    assert_eq!(g.player(P0).mana.spell(), amount);
    assert!(!mana::can_pay(&g, P0, 1, "", false));
    assert!(!mana::pay(&mut g, P0, 1, "", false).unwrap());
    let victim = perm(&mut g, P1, "Goldspan Dragon");
    // Rith grants ward {2} to the opponent's other Dragon.
    perm(&mut g, P1, "Rith, Liberated Primeval");
    assert!(!ward::pay_ward(&mut g, P0, victim).unwrap());
    let equip = perm(&mut g, P0, "Lightning Greaves");
    assert!(!stack::equip_to(&mut g, P0, equip, equip, 1).unwrap());
    g.player_mut(P0).mana.add(Kind::R, 2);
    g.pay_for = Some(card(&g, "Shivan Dragon"));
    assert!(mana::pay(&mut g, P0, 1, "RR", false).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::R), 2, "restricted mana pays pips and generic first");
    assert_eq!(g.player(P0).mana.spell(), amount - 3);
    g.pay_for = None;
    turn::end_step_pools(&mut g);
    assert_eq!(g.player(P0).mana.spell(), amount - 3);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).mana.spell(), 0);
}

#[test]
fn klauth_person_casts_in_combat_and_main_two_keeps_leftovers_then_loses_them_next_turn() {
    let mut g = tbl();
    passes(&mut g);
    g.step = Step::Combat;
    let amount = trigger_klauth(&mut g);
    let instant = card(&g, "Brainstorm");
    g.player_mut(P0).hand.push(instant);
    assert!(legal::check_cast(&mut g, P0, instant, "hand").is_none());
    assert!(human::cast(&mut g, P0, instant, "hand").unwrap().is_none());
    assert_eq!(g.player(P0).pool.spell, amount - 1);
    assert!(g.player(P0).pool.text().contains("for spells"));
    hm::empty_pools(&mut g);
    turn::end_step_pools(&mut g);
    assert_eq!(g.player(P0).pool.spell, amount - 1);
    g.step = Step::Main2;
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).hand = vec![dragon];
    assert!(legal::check_cast(&mut g, P0, dragon, "hand").is_none());
    assert!(human::cast(&mut g, P0, dragon, "hand").unwrap().is_none());
    assert!(has_perm(&g, P0, "Shivan Dragon"));
    assert_eq!(g.player(P0).pool.spell, amount - 7);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).pool.spell, 0);
}

#[test]
fn klauth_person_does_not_pay_abilities_equip_ward_or_triggered_taxes_even_in_spell_context() {
    let mut g = tbl();
    passes(&mut g);
    let amount = trigger_klauth(&mut g);
    hm::gather_floating(&mut g, P0);
    let dragon = card(&g, GOLD);
    for spending in [None, Some(hm::Spending::Perm(perms_named(&g, P0, GOLD)[0]))] {
        g.player_mut(P0).pool.spending = spending;
        assert!(!hm::pool_can_pay(&g, P0, 1, ""));
        assert!(hm::pay_from_pool(&mut g, P0, 1, "").is_some());
    }
    let equip = perm(&mut g, P0, "Swiftfoot Boots");
    let target = perms_named(&g, P0, GOLD)[0];
    assert!(legal::check_equip(&g, P0, equip, target).unwrap().contains("Can't equip"));
    g.player_mut(P0).pool.spending = Some(hm::Spending::Card(dragon));
    g.pay_for = Some(dragon);
    assert!(!hm::pay_tax_ask(&mut g, P0, 1, "Rhystic Study", None).unwrap());
    let victim = perm(&mut g, P1, GOLD);
    perm(&mut g, P1, "Rith, Liberated Primeval");
    assert!(!ward::pay_ward(&mut g, P0, victim).unwrap());
    assert_eq!(g.player(P0).pool.spell, amount);
}

#[test]
fn klauth_person_spends_restricted_before_plain_and_combines_with_typed_mana() {
    let mut g = tbl();
    passes(&mut g);
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).pool.spell = 2;
    pool(&mut g, P0, &[('R', 3)]);
    g.player_mut(P0).pool.spending = Some(hm::Spending::Card(dragon));
    assert!(hm::pay_from_pool(&mut g, P0, 1, "R").is_none());
    assert_eq!(g.player(P0).pool.m[3], 3);
    assert_eq!(g.player(P0).pool.spell, 0);
    g.player_mut(P0).pool.m = [0; 7];
    g.player_mut(P0).pool.spell = 1;
    g.player_mut(P0).pool.add_restricted('R', 1, "dragon");
    assert!(hm::pay_from_pool(&mut g, P0, 0, "RG").is_none());
    assert_eq!(g.player(P0).pool.total(), 0);
}

#[test]
fn klauth_counts_current_pumped_power_and_excludes_a_phased_attacker_at_resolution() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KLAUTH);
    let gold = perm(&mut g, P0, GOLD);
    g.perm_mut(gold).eot_pt.0 = 3;
    g.perm_mut(k).phased = true; // removed from combat, but its triggered ability still resolves
    let expected = sim_core::engine::values::epow(&g, gold) as u32;
    let f = g.registry.get(card(&g, KLAUTH)).unwrap().attack.unwrap();
    f(&mut g, k, P0, &[k, gold], P1).unwrap();
    assert_eq!(g.player(P0).mana.spell(), expected);
}

#[test]
fn klauth_trigger_uses_power_after_responses_and_survives_source_removal() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KLAUTH);
    let gold = perm(&mut g, P0, GOLD);
    hand(&mut g, P0, &["Blossoming Defense"]);
    pool(&mut g, P0, &[('G', 1)]);
    g.log = Some(vec![]);
    let mut cast_growth = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" && !cast_growth {
            cast_growth = true;
            Answer::Action(json!({"do":"cast","card":0}))
        } else if let Some(k) = r.choices.iter().position(|s| s.contains("Goldspan")) {
            Answer::Pick(k)
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h.clone());
    let f = g.registry.get(card(&g, KLAUTH)).unwrap().attack.unwrap();
    f(&mut g, k, P0, &[k, gold], P1).unwrap();
    assert_eq!(g.player(P0).mana.spell(), 10, "log: {:?}", (g.log.as_ref(), h.told()));
    g.player_mut(P0).mana.clear();
    zones::leave(&mut g, k).unwrap();
    f(&mut g, k, P0, &[k, gold], P1).unwrap();
    assert_eq!(g.player(P0).mana.spell(), 6);
}

fn cast_gy(g: &mut Game, c: sim_core::ids::CardId) -> Option<String> {
    let i = g.player(P0).gy.iter().position(|&x| x == c).unwrap();
    human::apply(g, P0, &json!({"do":"cast","zone":"gy","card":i})).unwrap()
}

#[test]
fn rivaz_person_permission_is_once_per_turn_and_not_consumed_by_failed_payment() {
    let mut g = tbl();
    passes(&mut g);
    let src = perm(&mut g, P0, RIVAZ);
    let first = card(&g, "Shivan Dragon");
    let second = card(&g, GOLD);
    g.player_mut(P0).gy = vec![first, second];
    assert!(matches!(legal::gy_mode(&g, P0, first), Some((legal::GyKind::Yawg, 4, _))));
    assert_eq!(legal::rivaz_source(&g, P0, first), Some(src));
    assert!(cast_gy(&mut g, first).unwrap().contains("mana"));
    assert!(g.perm(src).data.get(DataKey::T45Dragon).is_none());
    pool(&mut g, P0, &[('A', 20)]);
    assert!(cast_gy(&mut g, first).is_none());
    assert!(has_perm(&g, P0, "Shivan Dragon"));
    assert_eq!(g.perm(src).data.get(DataKey::T45Dragon), Some(&Val::Stamp(g.turn_stamp())));
    assert!(cast_gy(&mut g, second).unwrap().contains("already been used"));
    turn::step_start(&mut g, P1).unwrap();
    assert!(legal::gy_mode(&g, P0, second).is_none(), "only during your own turns");
    turn::step_start(&mut g, P0).unwrap();
    g.step = Step::Main1;
    pool(&mut g, P0, &[('A', 10)]);
    assert!(cast_gy(&mut g, second).is_none());
    assert!(has_perm(&g, P0, GOLD));
}

#[test]
fn rivaz_person_cancelled_permission_choice_does_not_consume_a_stamp_or_mana() {
    let mut g = tbl();
    let src = perm(&mut g, P0, RIVAZ);
    let c = card(&g, RIVAZ);
    let copy = zones::enter(
        &mut g,
        P0,
        c,
        zones::Enter {
            token: true,
            exc: Some(sim_core::state::CopyExc { not_legendary: true, ..Default::default() }),
            ..Default::default()
        },
    )
    .unwrap();
    let dragon = card(&g, "Shivan Dragon");
    let non_dragon = card(&g, "Sol Ring");
    g.player_mut(P0).gy = vec![dragon, non_dragon];
    assert!(legal::gy_mode(&g, P0, non_dragon).is_none());
    pool(&mut g, P0, &[('A', 10)]);
    seat(
        &mut g,
        P0,
        vec![Box::new(|r| {
            assert_eq!(r.choices, vec!["Rivaz of the Claw (1 of 2)", "Rivaz of the Claw (2 of 2)", "cancel"]);
            Answer::Decline
        })],
    );
    assert!(cast_gy(&mut g, dragon).is_none());
    assert!(g.player(P0).gy.contains(&dragon));
    assert_eq!(g.player(P0).pool.total(), 10);
    for m in [src, copy] {
        assert!(g.perm(m).data.get(DataKey::T45Dragon).is_none());
    }
    g.step = Step::Combat;
    assert!(legal::check_cast_gy(&g, P0, dragon).is_some(), "normal creature timing");
}

#[test]
fn rivaz_two_copies_give_two_casts_and_each_cast_gets_the_grant() {
    let mut g = tbl();
    passes(&mut g);
    let a = perm(&mut g, P0, RIVAZ);
    let c = card(&g, RIVAZ);
    let b = zones::enter(
        &mut g,
        P0,
        c,
        zones::Enter {
            token: true,
            exc: Some(sim_core::state::CopyExc { not_legendary: true, ..Default::default() }),
            ..Default::default()
        },
    )
    .unwrap();
    let first = card(&g, "Shivan Dragon");
    let second = card(&g, GOLD);
    g.player_mut(P0).gy = vec![first, second];
    pool(&mut g, P0, &[('A', 20)]);
    assert!(cast_gy(&mut g, first).is_none());
    assert!(cast_gy(&mut g, second).is_none());
    for m in [a, b] {
        assert_eq!(g.perm(m).data.get(DataKey::T45Dragon), Some(&Val::Stamp(g.turn_stamp())));
    }
    let m = perms_named(&g, P0, "Shivan Dragon")[0];
    assert_eq!(g.perm(m).data.get(DataKey::RivazExile), Some(&Val::Int(2)));
    zones::die(&mut g, a, "destroy").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).exile.contains(&first));
}

#[test]
fn rivaz_grant_applies_to_another_graveyard_permission_but_not_when_absent_or_phased() {
    for presence in [0, 1, 2] {
        let mut g = tbl();
        passes(&mut g);
        let src = if presence > 0 { Some(perm(&mut g, P0, RIVAZ)) } else { None };
        if presence == 2 {
            g.perm_mut(src.unwrap()).phased = true;
        }
        let dragon = card(&g, "Shivan Dragon");
        g.player_mut(P0).gy.push(dragon);
        g.player_mut(P0).yawg = true;
        g.player_mut(P0).yawg_gy = vec![dragon];
        pool(&mut g, P0, &[('A', 10)]);
        assert!(matches!(legal::gy_mode(&g, P0, dragon), Some((legal::GyKind::Yawg, ..))));
        assert!(cast_gy(&mut g, dragon).is_none());
        let m = perms_named(&g, P0, "Shivan Dragon")[0];
        if let Some(src) = src {
            zones::leave(&mut g, src).unwrap();
        }
        // Disable Yawgmoth's replacement to test specifically the granted dies trigger.
        g.player_mut(P0).yawg = false;
        zones::die(&mut g, m, "destroy").unwrap();
        assert_eq!(g.player(P0).exile.contains(&dragon), presence == 1);
        assert_eq!(g.player(P0).gy.contains(&dragon), presence != 1);
    }
}

#[test]
fn rivaz_person_can_tap_its_two_mana_in_different_colours_and_cast_a_changeling() {
    let mut g = tbl();
    let src = perm(&mut g, P0, RIVAZ);
    g.perm_mut(src).sick = false;
    let dragon = card(&g, "Mothdust Changeling");
    g.player_mut(P0).gy = vec![dragon];
    seat(&mut g, P0, vec![by_text("U")]);
    let source =
        hm::sources(&g, P0).into_iter().find(|s| s.perm.is_some_and(|i| g.player(P0).perms[i] == src)).unwrap();
    assert_eq!(source.amount, 2);
    assert!(!source.colours.contains('C'));
    assert!(hm::tap(&mut g, P0, source.id as i64, Some('R')).unwrap().is_none());
    assert_eq!(g.player(P0).pool.dragon[3], 1);
    assert_eq!(g.player(P0).pool.dragon[1], 1);
    assert!(g.player(P0).pool.text().contains("{U}{R} for Dragon creature spells"));
    passes(&mut g);
    assert!(legal::check_cast_gy(&g, P0, dragon).is_none());
    assert!(cast_gy(&mut g, dragon).is_none());
    assert!(has_perm(&g, P0, "Mothdust Changeling"));
    assert_eq!(g.player(P0).pool.dragon.iter().sum::<u32>(), 1);
    assert!(!hm::pool_can_pay(&g, P0, 1, ""));
}

#[test]
fn rivaz_cast_trigger_obeys_probe_without_state_changes() {
    let mut g = tbl();
    perm(&mut g, P0, RIVAZ);
    let c = card(&g, GOLD);
    let before_rng = g.rng.clone();
    let before_perms = g.perms.clone();
    g.trig_probe = true;
    assert!(matches!(
        sim_core::impls::t45_urdragon::rivaz_cast(&mut g, P0, c, "escape"),
        Err(sim_core::flow::Stop::Probe(_))
    ));
    assert_eq!(g.rng, before_rng);
    assert_eq!(g.perms, before_perms);
}

#[test]
fn rivaz_person_normal_cost_includes_increases_and_reductions() {
    let mut g = tbl();
    passes(&mut g);
    perm(&mut g, P0, RIVAZ);
    perm(&mut g, P0, "Dragonlord's Servant");
    perm(&mut g, P1, "Sphere of Resistance");
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    let cost = mana::cost_of(&g, P0, dragon);
    assert_eq!(cost, (4, "RR".into()), "-1 Dragon reduction and +1 spell increase");
    assert_eq!(legal::gy_mode(&g, P0, dragon).unwrap().1, cost.0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    assert!(cast_gy(&mut g, dragon).is_none());
    assert_eq!(g.player(P0).pool.total(), 0);
    assert!(has_perm(&g, P0, "Shivan Dragon"));
}

#[test]
fn rivaz_countered_cast_consumes_permission_but_grants_no_permanent() {
    let mut g = tbl();
    passes(&mut g);
    let src = perm(&mut g, P0, RIVAZ);
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    pool(&mut g, P0, &[('A', 10)]);
    hand(&mut g, P1, &["Counterspell"]);
    pool(&mut g, P1, &[('U', 2)]);
    let mut countered = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" && r.prompt.contains("casts Shivan Dragon") && !countered {
            countered = true;
            Answer::Action(json!({"do":"cast","card":0}))
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    g.humans.add(P1, h.clone());
    assert!(cast_gy(&mut g, dragon).is_none(), "{:?}", h.told());
    assert!(!has_perm(&g, P0, "Shivan Dragon"));
    assert!(g.player(P0).gy.contains(&dragon), "Rivaz is not flashback: countered cards return to the graveyard");
    assert!(!g.player(P0).exile.contains(&dragon));
    assert_eq!(g.perm(src).data.get(DataKey::T45Dragon), Some(&Val::Stamp(g.turn_stamp())));
}

#[test]
fn rivaz_common_cast_grant_survives_removal_during_its_cast_trigger() {
    let mut g = tbl();
    let src = perm(&mut g, P0, RIVAZ);
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    let mut removed = false;
    g.humans.add(
        P1,
        Arc::new(ScriptHuman::from_fn(move |r| {
            if r.kind == "priority" && r.prompt.contains("grant the Dragon") && !removed {
                removed = true;
                Answer::Action(json!({"do":"cast","card":0}))
            } else if r.kind == "target" {
                Answer::Pick(r.choices.iter().position(|s| s.contains("Rivaz")).unwrap())
            } else if r.kind == "priority" {
                Answer::Action(json!({"do":"pass"}))
            } else {
                Answer::Decline
            }
        })),
    );
    assert!(cast::cast_card(&mut g, P0, dragon, "escape", Ctx::default()).unwrap());
    assert!(!g.perm(src).on_bf);
    let m = perms_named(&g, P0, "Shivan Dragon")[0];
    assert_eq!(g.perm(m).data.get(DataKey::RivazExile), Some(&Val::Int(1)));
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).exile.contains(&dragon));
}

#[test]
fn rivaz_dies_exile_can_be_countered_and_obeys_the_trigger_probe() {
    let mut g = tbl();
    perm(&mut g, P0, RIVAZ);
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    assert!(cast::cast_card(&mut g, P0, dragon, "mgy", Ctx::default()).unwrap());
    let m = perms_named(&g, P0, "Shivan Dragon")[0];
    g.trig_probe = true;
    let before_rng = g.rng.clone();
    assert!(matches!(sim_core::impls::t45_urdragon::rivaz_exile(&mut g, m), Err(sim_core::flow::Stop::Probe(_))));
    assert_eq!(g.rng, before_rng);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    pool(&mut g, P1, &[('U', 3)]);
    let mut stifled = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" && r.prompt.contains("exile it") && !stifled {
            stifled = true;
            Answer::Action(json!({"do":"cast","card":0}))
        } else if r.prompt.contains("counter which ability") {
            Answer::Pick(0)
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    g.humans.add(P1, h.clone());
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).gy.contains(&dragon), "{:?}", h.told());
    assert!(!g.player(P0).exile.contains(&dragon));
}

#[test]
fn klauth_person_reserves_flexible_mana_for_colours_the_plain_pool_lacks() {
    for pips in ["URR", "RUR", "RRU"] {
        let g = tbl();
        let mut pool = hm::ManaPool::default();
        pool.spell = 1;
        pool.spending = Some(hm::Spending::Card(card(&g, GOLD)));
        pool.add('U', 1);
        pool.add('R', 1);
        assert!(pool.pay(&g, P0, 0, pips), "{pips}");
        assert_eq!(pool.total(), 0);
    }
    let g = tbl();
    let mut pool = hm::ManaPool::default();
    pool.spell = 1;
    pool.spending = Some(hm::Spending::Card(card(&g, GOLD)));
    assert!(!pool.can_pay(&g, P0, 0, "C"));
    assert_eq!(pool.missing(0, "C"), "{C}");
}

#[test]
fn rivaz_floated_mana_keeps_its_restriction_beside_a_lands_different_chosen_type() {
    let g = tbl();
    let elf = card(&g, "Llanowar Elves");
    let dragon = card(&g, "Shivan Dragon");
    let mut pool = hm::ManaPool::default();
    pool.dragon[4] = 2;
    pool.add_restricted('G', 1, "elf");
    pool.spending = Some(hm::Spending::Card(elf));
    assert!(!pool.can_pay(&g, P0, 1, "G"), "Rivaz mana cannot be relabelled Elf mana");
    assert!(pool.pay(&g, P0, 0, "G"));
    assert_eq!(pool.dragon[4], 2);
    pool.add_restricted('R', 1, "elf");
    pool.spending = Some(hm::Spending::Card(dragon));
    assert!(!pool.can_pay(&g, P0, 0, "RR"), "land mana cannot be relabelled Dragon mana");
    pool.spending = None;
    assert!(!pool.can_pay(&g, P0, 2, ""));
    pool.empty();
    assert_eq!(pool.total(), 0);
}

#[test]
fn klauth_ai_and_person_can_pay_for_a_counterspell() {
    for person in [false, true] {
        let mut g = tbl();
        g.player_mut(P0).mana.add_kept_restr(Kind::Any, Restr::SpellOnly, 2);
        hand(&mut g, P0, &["Counterspell"]);
        if person {
            passes(&mut g);
        }
        let ctr = card(&g, "Counterspell");
        let spell = card(&g, "Sol Ring");
        assert_eq!(stack::pick_counter(&g, P0, spell), Some(ctr));
        assert!(stack::pay_counter(&mut g, P0, ctr, Some(spell)).unwrap());
        assert_eq!(g.player(P0).mana.spell() + g.player(P0).pool.spell, 0);
        assert!(g.pay_for.is_none());
        assert!(g.player(P0).pool.spending.is_none());
    }
}

#[test]
fn klauth_ai_cannot_use_spell_mana_for_ward_or_draw_taxes_during_a_payment() {
    let mut g = tbl();
    let spell = card(&g, "Shivan Dragon");
    g.pay_for = Some(spell);
    g.player_mut(P0).mana.add_kept_restr(Kind::Any, Restr::SpellOnly, 20);
    let target = perm(&mut g, P1, GOLD);
    perm(&mut g, P1, "Rith, Liberated Primeval");
    assert!(!ward::pay_ward(&mut g, P0, target).unwrap());
    assert!(sim_core::cardcode::tithe_unpaid(&mut g, P0).unwrap());
    assert!(sim_core::cardcode::rhystic_unpaid(&mut g, P0).unwrap());
    assert_eq!(g.pay_for, Some(spell));
    assert_eq!(g.player(P0).mana.spell(), 20);
}

#[test]
fn rivaz_permission_refreshes_on_a_consecutive_extra_turn_without_a_new_round_stamp() {
    let mut g = tbl();
    passes(&mut g);
    let src = perm(&mut g, P0, RIVAZ);
    let a = card(&g, "Shivan Dragon");
    let b = card(&g, GOLD);
    g.player_mut(P0).gy = vec![a, b];
    pool(&mut g, P0, &[('A', 10)]);
    assert!(cast_gy(&mut g, a).is_none());
    let stamp = g.turn_stamp();
    assert!(cast_gy(&mut g, b).is_some());
    turn::step_start(&mut g, P0).unwrap();
    g.step = Step::Main1;
    assert_eq!(g.turn_stamp(), stamp);
    assert!(g.perm(src).data.get(DataKey::T45Dragon).is_none());
    pool(&mut g, P0, &[('A', 10)]);
    assert!(cast_gy(&mut g, b).is_none());
}

#[test]
fn rivaz_with_no_abilities_grants_no_mana_permission_or_dies_trigger() {
    let mut g = tbl();
    passes(&mut g);
    let src = perm(&mut g, P0, RIVAZ);
    g.perm_mut(src).sick = false;
    g.perm_mut(src).neutered = true;
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    assert!(legal::gy_mode(&g, P0, dragon).is_none());
    assert!(hm::sources(&g, P0).iter().all(|s| s.perm.is_none()));
    assert!(cast::cast_card(&mut g, P0, dragon, "escape", Ctx::default()).unwrap());
    let m = perms_named(&g, P0, "Shivan Dragon")[0];
    assert!(g.perm(m).data.get(DataKey::RivazExile).is_none());
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).gy.contains(&dragon));
    assert!(!g.player(P0).exile.contains(&dragon));
}

#[test]
fn klauth_ai_casts_heroic_intervention_in_combat_with_only_its_spell_mana() {
    let mut g = tbl();
    g.step = Step::Combat;
    let amount = trigger_klauth(&mut g);
    hand(&mut g, P0, &["Heroic Intervention"]);
    let target = perm(&mut g, P0, "The Ur-Dragon");
    g.perm_mut(target).is_cmd = true;
    let swords = card(&g, "Swords to Plowshares");
    hand(&mut g, P1, &["Swords to Plowshares"]);
    cast::cast_card(&mut g, P1, swords, "hand", Ctx { target: Some(target), ..Default::default() }).unwrap();
    assert!(g.perm(target).on_bf);
    assert!(g.player(P0).gy.contains(&card(&g, "Heroic Intervention")));
    assert_eq!(g.player(P0).mana.spell(), amount - 2);
    assert!(g.pay_for.is_none());
}

#[test]
fn klauth_ai_main_two_offers_and_casts_twinflame_with_no_other_mana() {
    let mut g = tbl();
    let amount = trigger_klauth(&mut g);
    g.step = Step::Main2;
    perm(&mut g, P0, "Mulldrifter");
    hand(&mut g, P0, &["Twinflame"]);
    let choices = sim_core::ai::brain::main_options(&mut g, P0, true).unwrap();
    let opt = choices.iter().find(|o| o.label.starts_with("Twinflame")).expect("main-two Twinflame option");
    assert!(sim_core::ai::act::perform(&mut g, P0, opt.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).gy.contains(&card(&g, "Twinflame")));
    assert!(g.player(P0).mana.spell() < amount);
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).token && g.perm(m).temp));
}

#[test]
fn klauth_person_casts_twinflame_through_its_hand_override() {
    let mut g = tbl();
    let amount = trigger_klauth(&mut g);
    g.step = Step::Main2;
    hand(&mut g, P0, &["Twinflame"]);
    let mut picked = false;
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r| {
            if r.kind == "target" && !picked {
                picked = true;
                Answer::Pick(r.choices.iter().position(|s| s.contains("Goldspan")).unwrap())
            } else if r.kind == "priority" {
                Answer::Action(json!({"do":"pass"}))
            } else {
                Answer::Decline
            }
        })),
    );
    assert!(human::apply(&mut g, P0, &json!({"do":"cast","card":0})).unwrap().is_none());
    assert_eq!(perms_named(&g, P0, GOLD).len(), 2);
    assert_eq!(g.player(P0).pool.spell, amount - 2);
    assert!(g.player(P0).pool.spending.is_none());
}

#[test]
fn klauth_person_casts_paid_deflecting_swat_in_response() {
    let mut g = tbl();
    g.step = Step::Combat;
    let amount = trigger_klauth(&mut g);
    let target = perms_named(&g, P0, KLAUTH)[0];
    let other = perm(&mut g, P1, "Shivan Dragon");
    hand(&mut g, P0, &["Deflecting Swat"]);
    hand(&mut g, P1, &["Swords to Plowshares"]);
    assert!(!sim_core::engine::values::commander_out(&g, P0));
    let mut swatted = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" && !swatted {
            swatted = true;
            Answer::Action(json!({"do":"cast","card":0}))
        } else if r.kind == "target" {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Shivan Dragon")).unwrap())
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    g.humans.add(P0, h.clone());
    let swords = card(&g, "Swords to Plowshares");
    cast::cast_card(&mut g, P1, swords, "hand", Ctx { target: Some(target), ..Default::default() }).unwrap();
    assert!(g.perm(target).on_bf, "{:?}", h.told());
    assert!(!g.perm(other).on_bf);
    assert!(g.player(P0).gy.contains(&card(&g, "Deflecting Swat")));
    assert_eq!(g.player(P0).pool.spell, amount - 3);
    assert!(g.player(P0).pool.spending.is_none());
}

#[test]
fn mightstone_mana_cannot_pay_heroic_intervention_but_klauth_mana_can() {
    let mut g = tbl();
    let target = perm(&mut g, P0, "The Ur-Dragon");
    g.perm_mut(target).is_cmd = true;
    hand(&mut g, P0, &["Heroic Intervention"]);
    g.player_mut(P0).mana.add_kept_restr(Kind::C, Restr::NotNonartifact, 2);
    g.player_mut(P0).mana.add(Kind::G, 1);
    let swords = card(&g, "Swords to Plowshares");
    assert!(!sim_core::ai::pool::protect(&mut g, P0, target, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).hand.contains(&card(&g, "Heroic Intervention")));
    assert_eq!(g.player(P0).mana.total(), 3);
    g.player_mut(P0).mana.clear();
    g.player_mut(P0).mana.add_kept_restr(Kind::Any, Restr::SpellOnly, 2);
    assert!(sim_core::ai::pool::protect(&mut g, P0, target, "exile", Some(P1), Some(swords)).unwrap());
    assert_eq!(g.player(P0).mana.total(), 0);
}

#[test]
fn spell_helpers_restore_context_and_do_not_enable_ability_payments() {
    let mut g = tbl();
    let spell = card(&g, "Sol Ring"); // no activated ability is involved in casting the spell
    let outer = card(&g, GOLD);
    g.player_mut(P0).mana.add_kept_restr(Kind::Any, Restr::SpellOnly, 2);
    g.pay_for = Some(outer);
    assert!(mana::can_pay_spell(&g, P0, spell, 1, "", false));
    assert!(mana::pay_spell(&mut g, P0, spell, 1, "", false).unwrap());
    assert_eq!(g.pay_for, Some(outer));
    g.player_mut(P0).hand.push(spell);
    assert!(cast::cast_card(&mut g, P0, spell, "hand", Ctx::default()).unwrap());
    assert!(has_perm(&g, P0, "Sol Ring"));
    g.perm_mut(perms_named(&g, P0, "Sol Ring")[0]).tapped = true;
    g.pay_for = None;
    assert!(!mana::can_pay(&g, P0, 1, "", false));
    assert!(!mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).mana.spell(), 1);
}

#[test]
fn rivaz_does_not_offer_graveyard_casts_that_rule_of_law_forbids() {
    let mut g = tbl();
    let src = perm(&mut g, P0, RIVAZ);
    let dragon = card(&g, "Shivan Dragon");
    g.player_mut(P0).gy = vec![dragon];
    g.player_mut(P0).mana.add(Kind::Any, 20);
    perm(&mut g, P1, "Rule of Law");
    let f = g.registry.get(card(&g, RIVAZ)).unwrap().options.unwrap();
    assert!(!f(&mut g, src, P0, Some(true)).unwrap().is_empty());
    let ring = card(&g, "Sol Ring");
    g.player_mut(P0).hand.push(ring);
    assert!(cast::cast_card(&mut g, P0, ring, "hand", Ctx::default()).unwrap());
    assert!(!cast::castable(&g, P0, dragon, "mgy"));
    assert!(f(&mut g, src, P0, Some(true)).unwrap().is_empty());
}

#[test]
fn twilight_diviner_sees_rivaz_and_escape_graveyard_casts() {
    for zone in ["mgy", "escape"] {
        let mut g = tbl();
        perm(&mut g, P0, "Twilight Diviner");
        let src = perm(&mut g, P0, RIVAZ);
        let dragon = card(&g, "Shivan Dragon");
        g.player_mut(P0).gy = vec![dragon];
        sim_core::impls::t12_brago_meren::note_gy(&mut g, dragon);
        let ctx = Ctx { rivaz_permission: (zone == "mgy").then_some(src), ..Default::default() };
        assert!(cast::cast_card(&mut g, P0, dragon, zone, ctx).unwrap());
        let dragons = perms_named(&g, P0, "Shivan Dragon");
        assert_eq!(dragons.len(), 2, "{zone}: original plus Diviner token");
        assert_eq!(dragons.iter().filter(|&&m| g.perm(m).token).count(), 1);
    }
}

struct WardObserver {
    pay: bool,
    pools_at_ward: std::sync::Mutex<Vec<(u32, u32)>>,
}

impl Human for WardObserver {
    fn ask(&self, g: &Game, p: PlayerId, r: Request) -> Answer {
        if r.kind == "choose" && r.prompt.contains("ward {2}") {
            self.pools_at_ward.lock().unwrap().push((g.player(p).pool.m[0], g.player(p).pool.total()));
            Answer::Pick(usize::from(!self.pay))
        } else if r.kind == "target" {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Colossal Skyturtle")).unwrap())
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }
    fn tell(&self, _: PlayerId, _: sim_core::sym::Sym, _: &str) {}
}

#[test]
fn person_swords_pays_white_at_cast_and_ward_two_exactly_once_or_is_countered() {
    for pay in [true, false] {
        let mut g = tbl();
        let target = perm(&mut g, P1, "Colossal Skyturtle");
        hand(&mut g, P0, &["Swords to Plowshares"]);
        pool(&mut g, P0, &[('W', 1), ('C', 2)]);
        let h = Arc::new(WardObserver { pay, pools_at_ward: Default::default() });
        g.humans.add(P0, h.clone());
        assert!(human::apply(&mut g, P0, &json!({"do":"cast","card":0})).unwrap().is_none());
        assert_eq!(*h.pools_at_ward.lock().unwrap(), vec![(0, 2)], "white paid at cast, ward asked once");
        assert_eq!(g.player(P0).pool.total(), if pay { 0 } else { 2 });
        assert_eq!(g.perm(target).on_bf, !pay);
        assert_eq!(g.player(P1).exile.contains(&card(&g, "Colossal Skyturtle")), pay);
        assert!(g.player(P0).gy.contains(&card(&g, "Swords to Plowshares")));
    }
}

#[test]
fn person_counter_removal_and_confluence_cannot_skip_ward() {
    for name in ["Heartless Act", "Eldrazi Confluence"] {
        let mut g = tbl();
        let target = perm(&mut g, P1, "Colossal Skyturtle");
        g.perm_mut(target).plus = 1;
        hand(&mut g, P0, &[name]);
        if name == "Heartless Act" {
            pool(&mut g, P0, &[('B', 1), ('C', 3)]);
        } else {
            pool(&mut g, P0, &[('C', 6)]);
        }
        let h = Arc::new(WardObserver { pay: false, pools_at_ward: Default::default() });
        g.humans.add(P0, h.clone());
        assert!(human::apply(&mut g, P0, &json!({"do":"cast","card":0})).unwrap().is_none(), "{name}");
        assert_eq!(h.pools_at_ward.lock().unwrap().len(), 1, "{name}");
        assert!(g.perm(target).on_bf, "{name}");
        assert_eq!(g.perm(target).plus, 1, "{name}");
        assert_eq!(g.perm(target).eot_pt, (0, 0), "{name}");
        assert_eq!(g.player(P0).pool.total(), 2, "{name}");
    }
}

#[test]
fn rivaz_second_colour_cancel_reports_failure_without_tapping_or_adding_mana() {
    let mut g = tbl();
    let src = perm(&mut g, P0, RIVAZ);
    g.perm_mut(src).sick = false;
    seat(&mut g, P0, vec![cancel()]);
    let source =
        hm::sources(&g, P0).into_iter().find(|s| s.perm.is_some_and(|i| g.player(P0).perms[i] == src)).unwrap();
    let why = hm::tap(&mut g, P0, source.id as i64, Some('R')).unwrap().unwrap();
    assert!(why.contains("Cancelled"));
    assert!(!g.perm(src).tapped);
    assert_eq!(g.player(P0).pool.total(), 0);
}

#[test]
fn rivaz_controlled_by_a_colourless_player_refuses_mana_without_panicking() {
    let mut g = tbl();
    g.player_mut(P0).ident = sim_core::cards::Colors::NONE;
    let src = perm(&mut g, P0, RIVAZ);
    g.perm_mut(src).sick = false;
    passes(&mut g);
    let source =
        hm::sources(&g, P0).into_iter().find(|s| s.perm.is_some_and(|i| g.player(P0).perms[i] == src)).unwrap();
    assert!(hm::tap(&mut g, P0, source.id as i64, None).unwrap().unwrap().contains("coloured mana"));
    assert!(!g.perm(src).tapped);
    assert_eq!(g.player(P0).pool.total(), 0);
}

/// P0 with every land and mana creature tapped but the lands named, so they are the only sources
fn only_sources(names: &[&str]) -> (Game, Vec<PermId>) {
    let mut g = tbl();
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = true;
    }
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).tapped = true;
    }
    let mut ids = vec![];
    for n in names {
        ids.push(lands(&mut g, P0, n, 1, false)[0]);
    }
    (g, ids)
}

#[test]
fn can_pay_spell_sees_the_same_hook_colours_as_pay_spell_for_cavern_of_souls() {
    let (mut g, ids) = only_sources(&["Cavern of Souls"]);
    g.perm_mut(ids[0]).data.set(DataKey::Ctype, Val::Str(sim_core::sym::intern("dragon")));
    let dragon = card(&g, "Goldspan Dragon");
    let other = card(&g, "Mulldrifter");
    assert!(g.pay_for.is_none());
    // the Dragon's {R} pip: Cavern makes any colour only for a Dragon spell
    assert!(mana::can_pay_spell(&g, P0, dragon, 0, "R", false));
    assert!(!mana::can_pay_spell(&g, P0, other, 0, "R", false), "a non-Dragon creature: colourless only");
    assert!(mana::paying(&g).is_none(), "the check puts the peeked spell back");
    // the peeked spell beats a different spell already being paid for
    g.pay_for = Some(other);
    assert!(mana::can_pay_spell(&g, P0, dragon, 0, "R", false));
    assert_eq!(g.pay_for, Some(other));
    g.pay_for = None;
    assert!(mana::pay_spell(&mut g, P0, dragon, 0, "R", false).unwrap(), "the payment agrees with the check");
    assert!(g.pay_for.is_none());
}

#[test]
fn can_pay_spell_sees_the_same_hook_colours_as_pay_spell_for_plaza_of_heroes() {
    let (mut g, ids) = only_sources(&["Plaza of Heroes"]);
    let legendary = card(&g, KLAUTH);
    let plain = card(&g, "Mulldrifter");
    assert!(mana::can_pay_spell(&g, P0, legendary, 0, "R", false), "Plaza: any colour for a legendary spell");
    assert!(!mana::can_pay_spell(&g, P0, plain, 0, "R", false));
    assert!(mana::paying(&g).is_none());
    assert!(mana::pay_spell(&mut g, P0, legendary, 0, "R", false).unwrap());
    assert!(g.perm(ids[0]).tapped);
    assert!(g.pay_for.is_none());
}

#[test]
fn can_pay_spell_sees_the_same_hook_colours_as_pay_spell_for_delighted_halfling() {
    let (mut g, _) = only_sources(&[]);
    let halfling = perm(&mut g, P0, "Delighted Halfling");
    g.perm_mut(halfling).sick = false;
    let legendary = card(&g, KLAUTH);
    let plain = card(&g, "Mulldrifter");
    assert!(mana::can_pay_spell(&g, P0, legendary, 0, "R", false), "Halfling: any colour for a legendary spell");
    assert!(!mana::can_pay_spell(&g, P0, plain, 0, "R", false));
    assert!(mana::pay_spell(&mut g, P0, legendary, 0, "R", false).unwrap());
    assert!(g.pay_for.is_none());
}
