//! Independent Oracle-based probes for kr_model; deliberately retain real failing regressions.
mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::values::{epow, etgh, untargetable};
use sim_core::engine::zones::{self, Tokens};
use sim_core::engine::{combat, life, turn};
use sim_core::hooks::{Action, Opt};
use sim_core::human::{human::abilities_of, legal};
use sim_core::impls::kr_model::{JITTE, WASTELAND, counters};
use sim_core::settings::Profile;
use sim_core::state::{DataKey, Step, Val};
use std::sync::Arc;

fn board() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    Arc::make_mut(&mut g.settings).profile = Profile::Loose;
    g.log = Some(vec![]);
    g
}

// Snow-Covered Island is absent from the committed export. Add its known basic-land
// definition only to this test's in-memory database; never rewrite the card data.
fn snow_board() -> Game {
    let mut g = board();
    let mut raw = sim_core::export::load_cards(&data_dir().join("cards.json")).unwrap();
    let mut snow = raw.iter().find(|c| c.name == "Island").unwrap().clone();
    snow.name = "Snow-Covered Island".into();
    assert!(snow.basic);
    raw.push(snow);
    g.db = Arc::new(sim_core::cards::CardDb::from_raw(&raw).unwrap());
    g.registry = Arc::new(sim_core::impls::registry(&g.db).unwrap());
    g
}

fn body(g: &mut Game, p: PlayerId, power: i32, toughness: i32) -> PermId {
    zones::make_tokens(g, p, Tokens { sick: false, tgh: Some(toughness), ..Tokens::new(1, power) }).unwrap()[0]
}

fn kw(g: &mut Game, c: PermId, keyword: &'static str) {
    g.perm_mut(c).eot_kw.push(keyword);
}

fn jitte(g: &mut Game, p: PlayerId, c: PermId, n: i64) -> PermId {
    let j = perm(g, p, JITTE);
    g.perm_mut(j).attached = Some(c);
    g.perm_mut(j).data.set(DataKey::Charge, Val::Int(n));
    j
}

fn fight(g: &mut Game, a: PermId, blocks: &[PermId]) -> i32 {
    let mut total = 0;
    combat::resolve_blocks(g, P0, vec![a], P1, blocks.iter().map(|&b| (a, b)).collect(), &mut total).unwrap();
    total
}

fn options(g: &mut Game, j: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(j).card()).unwrap().options.unwrap();
    f(g, j, P0, Some(false)).unwrap()
}

fn mode(g: &mut Game, j: PermId, word: &str) -> Action {
    let opts = options(g, j);
    opts.iter()
        .find(|o| o.label.contains(word))
        .unwrap_or_else(|| {
            panic!(
                "no mode {word}: source live={} counters={} options={:?}",
                g.perm(j).on_bf,
                counters(g, j),
                opts.iter().map(|o| &o.label).collect::<Vec<_>>()
            )
        })
        .act
        .clone()
        .unwrap()
}

fn use_mode(g: &mut Game, j: PermId, word: &str, answers: Vec<F>) {
    let mut answers = std::collections::VecDeque::from(answers);
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            answers.pop_front().map_or(Answer::Decline, |mut f| f(r))
        }
    }));
    g.humans.add(P0, h);
    let a = mode(g, j, word);
    assert!(perform(g, P0, &a).unwrap());
}

fn wasteland_action(g: &mut Game, w: PermId) -> Action {
    let f = g.registry.get(g.perm(w).card()).unwrap().land_options.unwrap();
    f(g, w, P0, Some(false)).unwrap().into_iter().next().expect("Wasteland activation").act.unwrap()
}

#[test]
fn multiple_blockers_each_deal_combat_damage_and_trigger_their_own_jitte() {
    let mut g = board();
    let a = body(&mut g, P0, 4, 10);
    let b1 = body(&mut g, P1, 1, 10);
    let b2 = body(&mut g, P1, 1, 10);
    let ja = jitte(&mut g, P0, a, 0);
    let jb1 = jitte(&mut g, P1, b1, 0);
    // A copied, nonlegendary second Jitte avoids the legend rule on the test board.
    let jb2 = jitte(&mut g, P1, b2, 0);
    g.perm_mut(jb2).exc.not_legendary = true;
    fight(&mut g, a, &[b1, b2]);
    assert_eq!(counters(&g, ja), 2, "one combat damage event despite two recipients");
    assert_eq!(counters(&g, jb1), 2);
    assert_eq!(counters(&g, jb2), 2, "the second blocker also dealt combat damage");
}

#[test]
fn double_strike_trample_over_a_first_step_kill_triggers_in_both_steps() {
    let mut g = board();
    let a = body(&mut g, P0, 4, 4);
    kw(&mut g, a, "double strike");
    kw(&mut g, a, "trample");
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 2);
    fight(&mut g, a, &[b]);
    assert_eq!(counters(&g, j), 4, "first step hits blocker/player, regular step hits player");
}

#[test]
fn first_strike_trample_is_one_trigger_even_when_it_hits_two_recipients() {
    let mut g = board();
    let a = body(&mut g, P0, 4, 4);
    kw(&mut g, a, "first strike");
    kw(&mut g, a, "trample");
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 2);
    fight(&mut g, a, &[b]);
    assert_eq!(counters(&g, j), 2);
    assert_eq!(g.player(P1).life, 38);
}

#[test]
fn double_striking_blocker_surviving_both_steps_gets_four_counters() {
    let mut g = board();
    let a = body(&mut g, P0, 1, 10);
    let b = body(&mut g, P1, 2, 10);
    kw(&mut g, b, "double strike");
    let j = jitte(&mut g, P1, b, 0);
    fight(&mut g, a, &[b]);
    assert_eq!(counters(&g, j), 4);
}

#[test]
fn first_strike_blocker_kills_before_the_equipped_attacker_deals_damage() {
    let mut g = board();
    let a = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 3, 4);
    kw(&mut g, b, "first strike");
    fight(&mut g, a, &[b]);
    assert!(!g.perm(a).on_bf);
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn simultaneous_trade_triggers_even_when_the_equipped_creature_dies() {
    let mut g = board();
    let a = body(&mut g, P0, 1, 1);
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 1, 1);
    fight(&mut g, a, &[b]);
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
    assert_eq!(counters(&g, j), 2);
}

#[test]
fn combat_damage_need_not_kill_anything() {
    let mut g = board();
    let a = body(&mut g, P0, 1, 10);
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 1, 10);
    fight(&mut g, a, &[b]);
    assert!(g.perm(a).on_bf && g.perm(b).on_bf);
    assert_eq!(counters(&g, j), 2);
}

#[test]
fn zero_power_and_fog_generate_no_counters() {
    for fog in [false, true] {
        let mut g = board();
        let a = body(&mut g, P0, if fog { 3 } else { 0 }, 5);
        let j = jitte(&mut g, P0, a, 0);
        let b = body(&mut g, P1, 0, 10);
        if fog {
            g.fog = Some(g.turn_stamp());
        }
        fight(&mut g, a, &[b]);
        assert_eq!(counters(&g, j), 0);
    }
}

#[test]
fn attacker_removed_before_damage_never_triggers() {
    let mut g = board();
    let a = body(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, a, 0);
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    seat(&mut g, P1, vec![act(json!({"do":"cast", "card":0})), pick(0), pass()]);
    fight(&mut g, a, &[]);
    assert!(!g.perm(a).on_bf, "exiled during blockers priority");
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn noncombat_damage_does_not_trigger() {
    let mut g = board();
    let a = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 10);
    life::removal_damage(&mut g, P0, Some(a), b, 2, None).unwrap();
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn redirected_combat_damage_still_triggers() {
    let mut g = board();
    let a = body(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 10);
    let aura = perm(&mut g, P1, "Pariah");
    g.perm_mut(aura).attached = Some(b);
    fight(&mut g, a, &[]);
    assert_eq!(g.player(P1).life, 40, "the damage was redirected to a creature");
    assert_eq!(counters(&g, j), 2, "redirected combat damage is still combat damage");
}

fn attack_walker(trample: bool, double: bool) -> (Game, PermId) {
    let mut g = board();
    let a = body(&mut g, P0, 3, 5);
    let j = jitte(&mut g, P0, a, 0);
    if trample {
        kw(&mut g, a, "trample");
    }
    if double {
        kw(&mut g, a, "double strike");
    }
    let w = perm(&mut g, P1, "Karn Liberated");
    g.perm_mut(w).loyalty = Some(20);
    seat(&mut g, P0, vec![]);
    g.human_walker_atk = vec![(a, w)];
    let blocks = if trample { vec![body(&mut g, P1, 0, 1)] } else { vec![] };
    fight(&mut g, a, &blocks);
    assert_eq!(g.player(P1).life, 40);
    (g, j)
}

#[test]
fn unblocked_double_striker_hitting_a_planeswalker_gets_four_counters() {
    let (g, j) = attack_walker(false, true);
    assert_eq!(counters(&g, j), 4);
}

#[test]
fn trampling_into_a_planeswalker_does_not_double_count_the_damage_step() {
    let (g, j) = attack_walker(true, false);
    assert_eq!(counters(&g, j), 2, "blocker and planeswalker are hit in the same step");
}

#[test]
fn trampling_into_a_battle_does_not_double_count_the_damage_step() {
    let mut g = board();
    let a = body(&mut g, P0, 3, 5);
    let j = jitte(&mut g, P0, a, 0);
    kw(&mut g, a, "trample");
    let battle = perm(&mut g, P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    g.perm_mut(battle).data.set(DataKey::Defense, Val::Int(10));
    g.perm_mut(battle).data.set(DataKey::Protector, Val::Player(P1));
    seat(&mut g, P0, vec![]);
    g.human_walker_atk = vec![(a, battle)];
    let b = body(&mut g, P1, 0, 1);
    fight(&mut g, a, &[b]);
    assert_eq!(sim_core::impls::card_models3::defense(&g, battle), 8);
    assert_eq!(counters(&g, j), 2, "blocker and battle are hit in one step");
}

#[test]
fn lifelink_happens_before_the_jitte_trigger_resolves() {
    let mut g = board();
    // Observe the real game during lifelink's static checks, before queued triggers resolve.
    fn observe(g: &Game, src: PermId, p: PlayerId) -> bool {
        if p == P0 {
            assert_eq!(counters(g, src), 0, "Jitte counters cannot resolve before lifelink damage results");
        }
        false
    }
    let mut registry = sim_core::impls::registry(&g.db).unwrap();
    registry.card(&g.db, JITTE).unwrap().no_lifegain = Some(observe);
    g.registry = Arc::new(registry);
    let a = body(&mut g, P0, 3, 5);
    g.perm_mut(a).lifelink = true;
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 10);
    fight(&mut g, a, &[b]);
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(counters(&g, j), 2);
}

#[test]
fn shrink_kills_through_indestructible_and_regeneration() {
    for keyword in ["indestructible", ""] {
        let mut g = board();
        let c = body(&mut g, P0, 2, 2);
        let j = jitte(&mut g, P0, c, 1);
        let victim = body(&mut g, P1, 1, 1);
        if !keyword.is_empty() {
            kw(&mut g, victim, keyword);
        }
        g.player_mut(P1).regen_turn = Some(g.turn_stamp());
        use_mode(&mut g, j, "-1/-1", vec![by_text("1/1")]);
        assert!(!g.perm(victim).on_bf, "zero toughness is not destruction");
        assert_eq!(counters(&g, j), 0);
    }
}

#[test]
fn shrink_excludes_opposing_hexproof_and_shroud_but_allows_own_hexproof() {
    let mut g = board();
    let own = body(&mut g, P0, 2, 5);
    kw(&mut g, own, "hexproof");
    let j = jitte(&mut g, P0, own, 1);
    let hex = body(&mut g, P1, 3, 3);
    let shroud = body(&mut g, P1, 4, 4);
    kw(&mut g, hex, "hexproof");
    kw(&mut g, shroud, "shroud");
    use_mode(
        &mut g,
        j,
        "-1/-1",
        vec![Box::new(|r| {
            assert!(!r.choices.iter().any(|s| s.contains("3/3") || s.contains("4/4")), "{:?}", r.choices);
            Answer::Pick(r.choices.iter().position(|s| s.contains("2/5")).unwrap())
        })],
    );
    assert_eq!(etgh(&g, own), 4);
}

#[test]
fn ward_counters_shrink_without_refunding_the_charge_counter() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 1);
    let victim = perm(&mut g, P1, "Esper Sentinel");
    // Construct its already-attached state without replaying its separate ETB exile.
    let cd = card(&g, "Sheltered by Ghosts");
    let aura = g.new_perm(P1, Some(cd), "", 0, 0);
    g.perm_mut(aura).on_bf = true;
    g.player_mut(P1).perms.push(aura);
    g.auras.push(aura);
    g.perm_mut(aura).attached = Some(victim);
    assert!(sim_core::cardcode::aura_ward(&g, victim) > 0);
    use_mode(&mut g, j, "-1/-1", vec![by_text("Esper Sentinel")]);
    assert!(g.perm(victim).on_bf);
    assert_eq!(etgh(&g, victim), 1, "the aura grants +1/+0, not toughness; ward stops the shrink");
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn shrink_target_exiled_in_response_is_not_replaced_and_the_counter_stays_paid() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 1);
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let other = body(&mut g, P1, 0, 5);
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    let h = seat(&mut g, P1, vec![act(json!({"do":"cast", "card":0})), by_text("Llanowar"), pass()]);
    use_mode(&mut g, j, "-1/-1", vec![by_text("Llanowar")]);
    assert!(h.told_kind("invalid").is_empty(), "{:?}", h.told());
    assert!(!g.perm(victim).on_bf);
    assert_eq!(etgh(&g, other), 5);
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn no_charge_counter_means_no_modes_for_human_or_ai() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 0);
    g.player_mut(P0).life = 1;
    assert!(options(&mut g, j).is_empty());
    seat(&mut g, P0, vec![]);
    assert!(options(&mut g, j).is_empty());
    assert!(abilities_of(&mut g, P0, j).unwrap().iter().all(|x| x.0.starts_with("Equip")));
}

#[test]
fn pump_affects_only_the_equipped_creature_and_expires_next_turn() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 5);
    let other = body(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 1);
    use_mode(&mut g, j, "+2/+2", vec![]);
    assert_eq!((epow(&g, c), etgh(&g, c)), (4, 7));
    assert_eq!((epow(&g, other), etgh(&g, other)), (3, 3));
    turn::end_step(&mut g, P0).unwrap();
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!((epow(&g, c), etgh(&g, c)), (2, 5));
}

#[test]
fn equip_moves_for_two_and_is_rejected_outside_sorcery_timing() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 2);
    let other = body(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 3);
    seat(&mut g, P0, vec![]);
    pool(&mut g, P0, &[('C', 2)]);
    assert_eq!(legal::equip_cost(&g, j), Some(2));
    g.step = Step::Combat;
    assert!(legal::check_equip(&g, P0, j, other).is_some());
    g.step = Step::Main1;
    g.active = Some(P1);
    assert!(legal::check_equip(&g, P0, j, other).is_some());
    g.active = Some(P0);
    assert!(legal::check_equip(&g, P0, j, other).is_none());
    let i = g.player(P0).perms.iter().position(|&m| m == j).unwrap();
    let k = abilities_of(&mut g, P0, j).unwrap().iter().position(|a| a.0.starts_with("Equip")).unwrap();
    main(&mut g, P0, vec![act(json!({"do":"use", "perm":i})), pick(k), by_text("3/3")]);
    assert_eq!(g.perm(j).attached, Some(other));
    assert_eq!(pool_total(&g, P0), 0);
    assert_eq!(counters(&g, j), 3);
}

#[test]
fn equipment_becomes_unattached_when_its_creature_dies() {
    let mut g = board();
    let c = body(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 3);
    zones::die(&mut g, c, "destroy").unwrap();
    life::check_state(&mut g).unwrap();
    assert!(g.perm(j).on_bf);
    assert_eq!(counters(&g, j), 3);
    assert_eq!(g.perm(j).attached, None, "CR 301.5c: nonexistent equipped creature means unattached");
}

#[test]
fn without_jitte_the_hooks_preserve_damage_totals_and_rng_state() {
    for double in [false, true] {
        let mut g = board();
        let a = body(&mut g, P0, 4, 10);
        if double {
            kw(&mut g, a, "double strike");
        }
        kw(&mut g, a, "trample");
        let b = body(&mut g, P1, 0, 2);
        let before = format!("{:?}", g.rng);
        sim_core::impls::kr_model::jitte_combat(&mut g, P0, &[a], P1, &[(a, b)]).unwrap();
        assert_eq!(sim_core::impls::kr_model::jitte_pair(&mut g, a, b).unwrap(), 0);
        sim_core::impls::kr_model::jitte_hits(&mut g, a, 255).unwrap();
        assert_eq!(format!("{:?}", g.rng), before, "no Jitte: hooks consume no RNG");
        let expected = if double { 6 } else { 2 };
        assert_eq!(fight(&mut g, a, &[b]), expected);
        assert_eq!(g.player(P1).life, 40 - expected);
    }
    // The lethal-pump hook also runs for unblocked attackers in every deck.
    let mut g = board();
    let a = body(&mut g, P0, sim_core::engine::chars::POWER_CAP, 10);
    kw(&mut g, a, "double strike");
    let before = g.rng.clone();
    sim_core::impls::kr_model::jitte_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!(g.rng, before);
    assert_eq!(g.player(P1).life, 40, "the no-Jitte hook changes no game state");
}

#[test]
fn snow_basics_are_excluded_but_another_wasteland_is_a_legal_target() {
    let mut g = snow_board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let snow = lands(&mut g, P1, "Snow-Covered Island", 1, false)[0];
    assert!(g.db.get(g.perm(snow).card()).basic);
    let other = lands(&mut g, P1, WASTELAND, 1, false)[0];
    seat(
        &mut g,
        P0,
        vec![Box::new(|r| {
            assert!(!r.choices.iter().any(|s| s.contains("Snow-Covered")), "{:?}", r.choices);
            Answer::Pick(r.choices.iter().position(|s| s.starts_with("Wasteland (") && !s.contains("yours")).unwrap())
        })],
    );
    let a = wasteland_action(&mut g, w);
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(!g.player(P0).lands.contains(&w));
    assert!(!g.player(P1).lands.contains(&other));
    assert!(g.player(P1).lands.contains(&snow));
}

#[test]
fn wasteland_may_target_itself_then_fizzles_after_sacrificing_itself() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    assert!(!g.db.get(g.perm(w).card()).basic);
    seat(&mut g, P0, vec![pick(0)]);
    let f = g.registry.get(g.perm(w).card()).unwrap().land_options.unwrap();
    let opts = f(&mut g, w, P0, Some(false)).unwrap();
    assert!(!opts.is_empty(), "target chosen before costs: Wasteland itself is a legal nonbasic target");
    assert!(perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&w));
}

fn protected_land_menu(keyword: &'static str) {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let protected = lands(&mut g, P1, "Ancient Tomb", 1, false)[0];
    kw(&mut g, protected, keyword);
    assert!(untargetable(&g, protected));
    lands(&mut g, P1, "Sunpetal Grove", 1, false);
    seat(
        &mut g,
        P0,
        vec![Box::new(|r| {
            assert!(
                !r.choices.iter().any(|s| s.contains("Ancient Tomb")),
                "illegal land target offered: {:?}",
                r.choices
            );
            Answer::Pick(r.choices.iter().position(|s| s.contains("Sunpetal Grove")).unwrap())
        })],
    );
    let a = wasteland_action(&mut g, w);
    assert!(perform(&mut g, P0, &a).unwrap());
}

#[test]
fn wasteland_does_not_offer_opposing_hexproof_lands() {
    protected_land_menu("hexproof");
}

#[test]
fn wasteland_does_not_offer_shroud_lands() {
    protected_land_menu("shroud");
}

#[test]
fn ai_wasteland_ignores_basics_and_indestructible_lands() {
    let mut g = snow_board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P1, "Island", 2, false);
    lands(&mut g, P1, "Snow-Covered Island", 1, false);
    lands(&mut g, P1, "Darksteel Citadel", 1, false);
    let f = g.registry.get(g.perm(w).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, w, P0, Some(false)).unwrap().is_empty());
}

#[test]
fn wasteland_is_sacrificed_even_when_indestructible_target_survives() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let target = lands(&mut g, P1, "Darksteel Citadel", 1, false)[0];
    seat(&mut g, P0, vec![by_text("Darksteel Citadel")]);
    let a = wasteland_action(&mut g, w);
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(!g.player(P0).lands.contains(&w));
    assert!(g.player(P1).lands.contains(&target));
}

#[test]
fn rings_copy_can_retarget_wasteland_without_paying_the_sacrifice_twice() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let first = lands(&mut g, P1, "Ancient Tomb", 1, false)[0];
    let second = lands(&mut g, P1, "Sunpetal Grove", 1, false)[0];
    perm(&mut g, P0, "Rings of Brighthearth");
    pool(&mut g, P0, &[('C', 2)]);
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        if r.prompt.starts_with("Wasteland: destroy which") {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Ancient Tomb")).unwrap())
        } else if r.prompt.starts_with("Rings of Brighthearth: pay") {
            Answer::Pick(0)
        } else if r.prompt.starts_with("The copy of Wasteland") {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Sunpetal Grove")).unwrap())
        } else {
            Answer::Action(json!({"do":"pass"}))
        }
    }));
    g.humans.add(P0, h.clone());
    let a = wasteland_action(&mut g, w);
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("The copy of Wasteland")));
    assert!(!g.player(P0).lands.contains(&w));
    assert!(!g.player(P1).lands.contains(&first) && !g.player(P1).lands.contains(&second));
    assert_eq!(pool_total(&g, P0), 0, "only Rings' {{2}} is paid");
    assert_eq!(g.player(P0).gy.iter().filter(|&&c| c == card(&g, WASTELAND)).count(), 1);
}

#[test]
fn wasteland_cost_stays_paid_when_target_sacrifices_itself_in_response() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    let target = lands(&mut g, P1, "Strip Mine", 1, false)[0];
    let h = seat(&mut g, P1, vec![act(json!({"do":"use", "land":0})), pick(0), by_text("Island"), pass()]);
    h.set_autopass("stack");
    seat(&mut g, P0, vec![by_text("Strip Mine"), pass()]);
    let a = wasteland_action(&mut g, w);
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(h.told_kind("invalid").is_empty(), "{:?}", h.told());
    assert!(!g.player(P1).lands.contains(&target), "Strip Mine sacrificed itself in response");
    assert!(!g.player(P0).lands.contains(&island), "the response destroyed Island");
    assert!(!g.player(P0).lands.contains(&w), "Wasteland's cost is not refunded");
}

#[test]
fn ai_wasteland_never_chooses_an_opponents_hexproof_land() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    lands(&mut g, P0, "Swamp", 3, false);
    let target = lands(&mut g, P1, "Ancient Tomb", 1, false)[0];
    kw(&mut g, target, "hexproof");
    let f = g.registry.get(g.perm(w).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, w, P0, Some(false)).unwrap().is_empty(), "hexproof is not a legal AI target");
}

#[test]
fn jitte_combat_trigger_gives_the_opponent_priority() {
    let mut g = board();
    let a = body(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, a, 0);
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Action(json!({"do":"pass"}))));
    h.set_autopass("all");
    g.humans.add(P1, h.clone());
    fight(&mut g, a, &[]);
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(counters(&g, j), 2);
    assert!(
        h.asked().iter().any(|r| r.prompt.contains("Jitte")),
        "the combat damage trigger must use the stack and give priority: {:?}",
        h.asked().iter().map(|r| &r.prompt).collect::<Vec<_>>()
    );
}

#[test]
fn damage_reduced_to_zero_by_gisela_cannot_trigger_jitte() {
    let mut g = board();
    let a = body(&mut g, P0, 1, 10);
    g.perm_mut(a).lifelink = true;
    let j = jitte(&mut g, P0, a, 0);
    let b = body(&mut g, P1, 0, 10);
    perm(&mut g, P1, "Gisela, Blade of Goldnight");
    fight(&mut g, a, &[b]);
    assert_eq!(g.player(P0).life, 40, "Gisela prevented all one damage: no lifelink");
    assert_eq!(counters(&g, j), 0, "prevented damage was not dealt");
}

#[test]
fn practice_wasteland_menu_activates_the_hook_and_preserves_basics() {
    let mut g = board();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let basic = lands(&mut g, P1, "Island", 1, false)[0];
    let target = lands(&mut g, P1, "Sunpetal Grove", 1, false)[0];
    let h = main(&mut g, P0, vec![act(json!({"do":"use", "land":0})), pick(0), by_text("Sunpetal Grove")]);
    assert!(h.told_kind("invalid").is_empty(), "{:?}", h.told());
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Wasteland: which ability?")));
    assert!(!g.player(P0).lands.contains(&w));
    assert!(!g.player(P1).lands.contains(&target));
    assert!(g.player(P1).lands.contains(&basic));
}
