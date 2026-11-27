//! Wasteland and Umezawa's Jitte (the karn_reforged deck's two cards with no record; impls/kr_model.rs). Every expectation
//! follows the Oracle text:
//!
//! Wasteland: {T}: Add {C}. / {T}, Sacrifice this land: Destroy target nonbasic land.
//!
//! Umezawa's Jitte: Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte. / Remove a
//! charge counter from Umezawa's Jitte: Choose one: equipped creature gets +2/+2 until end of turn; target creature gets
//! -1/-1 until end of turn; you gain 2 life. / Equip {2}
//!
//! Hand-built boards; the combat tests call `combat::resolve_blocks` with the blocks written out.

mod play_common;
use play_common::*;
use sim_core::ai::act as ai_act;
use sim_core::engine::mana;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::{self, Tokens};
use sim_core::engine::{combat, life};
use sim_core::hooks::{Action, Opt};
use sim_core::human::human::abilities_of;
use sim_core::impls::kr_model::{JITTE, WASTELAND, counters, pair_steps};
use sim_core::state::{DataKey, Step, Val};

fn tbl() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn creature(g: &mut Game, p: PlayerId, pow: i32, tgh: i32) -> PermId {
    let spec = Tokens { sick: false, tgh: Some(tgh), ..Tokens::new(1, pow) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn give(g: &mut Game, m: PermId, kw: &'static str) {
    g.perm_mut(m).eot_kw.push(kw);
}

/// a Jitte of p's on the battlefield, worn by c, with n charge counters
fn jitte(g: &mut Game, p: PlayerId, c: PermId, n: i64) -> PermId {
    let j = perm(g, p, JITTE);
    g.perm_mut(j).attached = Some(c);
    g.perm_mut(j).data.set(DataKey::Charge, Val::Int(n));
    j
}

fn fight(g: &mut Game, atk: Vec<PermId>, assign: Vec<(PermId, PermId)>) {
    let mut tot = 0;
    combat::resolve_blocks(g, P0, atk, P1, assign, &mut tot).unwrap();
}

/// the (perm index, ability index) of the first ability of m with `word` in its label, as the person at p sees them
fn label_index(g: &mut Game, p: PlayerId, m: PermId, word: &str) -> (usize, usize) {
    seat(g, p, vec![]); // a person's list: the hook's options for a person, not the AI's
    let i = g.player(p).perms.iter().position(|&x| x == m).unwrap();
    let ab = abilities_of(g, p, m).unwrap();
    let k = ab
        .iter()
        .position(|a| a.0.contains(word))
        .unwrap_or_else(|| panic!("no ability with {word:?} in {:?}", ab.iter().map(|a| &a.0).collect::<Vec<_>>()));
    (i, k)
}

fn options_of(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.expect("the card has options");
    f(g, src, p, Some(false)).unwrap()
}

// ================================================================== Wasteland
#[test]
fn wasteland_taps_for_colourless_mana() {
    let mut g = tbl();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    assert!(mana::can_pay(&g, P0, 1, "", false));
    assert!(mana::can_pay(&g, P0, 0, "C", false), "{{C}}, not a coloured mana");
    assert!(!mana::can_pay(&g, P0, 0, "B", false));
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert!(g.perm(w).tapped);
    assert!(g.player(P0).lands.contains(&w), "tapping for mana does not sacrifice it");
}

#[test]
fn the_ai_sacrifices_wasteland_to_destroy_a_key_nonbasic_land() {
    let mut g = tbl();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    lands(&mut g, P0, "Swamp", 3, false);
    let tomb = lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false)[0];
    let isle = lands(&mut g, P1, "Island", 2, false);
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    let o = opts.iter().find(|o| o.label.starts_with("Wasteland -> ")).expect("Wasteland's option");
    assert_eq!(o.label, "Wasteland -> Urborg, Tomb of Yawgmoth");
    assert!(ai_act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&w), "sacrificed as the cost");
    assert!(g.player(P0).gy.contains(&card(&g, WASTELAND)));
    assert!(!g.player(P1).lands.contains(&tomb), "the nonbasic land is destroyed");
    assert!(g.player(P1).gy.contains(&card(&g, "Urborg, Tomb of Yawgmoth")));
    assert!(isle.iter().all(|i| g.player(P1).lands.contains(i)), "nothing else is touched");
}

#[test]
fn the_ai_never_aims_wasteland_at_basic_lands() {
    let mut g = tbl();
    lands(&mut g, P0, WASTELAND, 1, false);
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P1, "Island", 3, false);
    lands(&mut g, P1, "Forest", 2, false);
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert!(!opts.iter().any(|o| o.label.starts_with("Wasteland")), "only basics to target: no option");
}

#[test]
fn a_tapped_wasteland_cannot_be_used() {
    let mut g = tbl();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
    g.perm_mut(w).tapped = true;
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert!(!opts.iter().any(|o| o.label.starts_with("Wasteland")), "the {{T}} cost can't be paid");
}

#[test]
fn a_person_picks_a_nonbasic_land_and_basics_are_not_offered() {
    let mut g = tbl();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let island = lands(&mut g, P1, "Island", 2, false);
    let grove = lands(&mut g, P1, "Sunpetal Grove", 1, false)[0];
    let target: F = Box::new(|r| {
        assert!(r.prompt.starts_with("Wasteland: destroy which"), "{}", r.prompt);
        assert!(!r.choices.iter().any(|c| c.contains("Island")), "a basic land is no target: {:?}", r.choices);
        Answer::Pick(r.choices.iter().position(|c| c.contains("Sunpetal Grove")).expect("the nonbasic land"))
    });
    main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), target]);
    assert!(!g.player(P0).lands.contains(&w), "Wasteland was sacrificed");
    assert!(!g.player(P1).lands.contains(&grove), "Sunpetal Grove was destroyed");
    assert!(island.iter().all(|i| g.player(P1).lands.contains(i)));
}

#[test]
fn a_person_with_only_basics_to_target_is_offered_nothing() {
    let mut g = tbl();
    lands(&mut g, P0, WASTELAND, 1, false);
    lands(&mut g, P1, "Island", 3, false);
    let h = main(&mut g, P0, vec![act(json!({"do": "use", "land": 0}))]);
    assert!(
        h.told().iter().any(|t| t.1.contains("no ability to activate")),
        "nothing to destroy, nothing to activate: {:?}",
        h.told()
    );
    assert!(g.player(P0).lands.len() == 1, "Wasteland stays");
}

#[test]
fn an_indestructible_land_survives_wasteland_but_the_land_is_still_sacrificed() {
    let mut g = tbl();
    let w = lands(&mut g, P0, WASTELAND, 1, false)[0];
    let citadel = lands(&mut g, P1, "Darksteel Citadel", 1, false)[0];
    let target: F = Box::new(|r| Answer::Pick(r.choices.iter().position(|c| c.contains("Darksteel")).unwrap()));
    main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), target]);
    assert!(g.player(P1).lands.contains(&citadel), "indestructible: not destroyed");
    assert!(!g.player(P0).lands.contains(&w), "the sacrifice was a cost");
}

#[test]
fn the_ai_leaves_an_indestructible_land_alone() {
    let mut g = tbl();
    lands(&mut g, P0, WASTELAND, 1, false);
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P1, "Darksteel Citadel", 1, false);
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert!(!opts.iter().any(|o| o.label.starts_with("Wasteland")));
}

// ================================================================== Umezawa's Jitte: equip
#[test]
fn equip_costs_two_for_the_ai_and_a_person() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = perm(&mut g, P0, JITTE);
    assert_eq!(sim_core::human::legal::equip_cost(&g, j), Some(2));
    lands(&mut g, P0, "Plains", 1, false);
    assert!(
        !sim_core::dsl::equip_options(&g, P0)
            .iter()
            .any(|o| matches!(o.act, Some(Action::Equip { src, .. }) if src == j)),
        "one land is not enough"
    );
    lands(&mut g, P0, "Plains", 1, false);
    let eq: Vec<Action> = sim_core::dsl::equip_options(&g, P0)
        .into_iter()
        .filter_map(|o| o.act)
        .filter(|a| matches!(a, Action::Equip { src, .. } if *src == j))
        .collect();
    assert_eq!(eq.len(), 1, "the AI offers the equip with two lands");
    let Action::Equip { target, n, .. } = &eq[0] else { unreachable!() };
    assert_eq!((*target, *n), (c, 2));
    assert!(ai_act::perform(&mut g, P0, &eq[0]).unwrap());
    assert_eq!(g.perm(j).attached, Some(c));
}

#[test]
fn a_person_equips_it_for_two() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = perm(&mut g, P0, JITTE);
    let (i, k) = label_index(&mut g, P0, j, "Equip");
    pool(&mut g, P0, &[('C', 2)]);
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pick(0)]);
    assert_eq!(g.perm(j).attached, Some(c));
    assert_eq!(pool_total(&g, P0), 0, "{{2}} paid");
}

// ================================================================== the counters
#[test]
fn two_counters_for_combat_damage_to_a_player() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 0);
    let life = g.player(P1).life;
    fight(&mut g, vec![c], vec![]);
    assert_eq!(g.player(P1).life, life - 3);
    assert_eq!(counters(&g, j), 2);
}

#[test]
fn two_counters_for_combat_damage_to_a_blocker_without_damage_to_the_player() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 0);
    let b = creature(&mut g, P1, 1, 9);
    let life = g.player(P1).life;
    fight(&mut g, vec![c], vec![(c, b)]);
    assert_eq!(g.player(P1).life, life, "blocked: nothing to the player");
    assert_eq!(counters(&g, j), 2, "it dealt combat damage to the blocker");
}

#[test]
fn an_equipped_blocker_gets_counters_for_the_damage_it_deals() {
    let mut g = tbl();
    let a = creature(&mut g, P0, 3, 5);
    let b = creature(&mut g, P1, 2, 9);
    let j = jitte(&mut g, P1, b, 0);
    fight(&mut g, vec![a], vec![(a, b)]);
    assert_eq!(counters(&g, j), 2, "the blocker dealt combat damage to the attacker");
}

#[test]
fn counters_are_put_even_if_the_equipped_creature_dies_in_that_damage() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 0);
    let b = creature(&mut g, P1, 5, 5);
    fight(&mut g, vec![c], vec![(c, b)]);
    assert!(!g.perm(c).on_bf, "the 2/2 died");
    assert!(g.perm(j).on_bf);
    assert_eq!(counters(&g, j), 2, "it dealt its damage first: the trigger still happens");
}

#[test]
fn no_counters_when_it_deals_no_damage() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 0, 3);
    let j = jitte(&mut g, P0, c, 0);
    fight(&mut g, vec![c], vec![]);
    assert_eq!(counters(&g, j), 0, "a 0-power creature deals no damage");
    // damage that is all prevented is not dealt either
    let d = creature(&mut g, P0, 3, 3);
    let j2 = jitte(&mut g, P0, d, 0);
    g.fog = Some(g.turn_stamp());
    fight(&mut g, vec![d], vec![]);
    assert_eq!(counters(&g, j2), 0, "a fog stops it");
}

#[test]
fn noncombat_damage_by_the_equipped_creature_adds_no_counters() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 0);
    let victim = creature(&mut g, P1, 1, 5);
    life::removal_damage(&mut g, P0, Some(c), victim, 2, None).unwrap();
    assert_eq!(counters(&g, j), 0, "damage from a spell or ability is not combat damage");
}

#[test]
fn a_creature_that_is_not_equipped_gives_no_counters() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 3, 3);
    let other = creature(&mut g, P0, 3, 3);
    let j = jitte(&mut g, P0, c, 0);
    fight(&mut g, vec![other], vec![]);
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn double_strike_triggers_once_for_each_damage_step() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    give(&mut g, c, "double strike");
    let j = jitte(&mut g, P0, c, 0);
    fight(&mut g, vec![c], vec![]);
    assert_eq!(counters(&g, j), 4, "two damage steps: two triggers");
    // blocked by a creature that survives both steps: two triggers too
    let d = creature(&mut g, P0, 2, 2);
    give(&mut g, d, "double strike");
    let j2 = jitte(&mut g, P0, d, 0);
    let b = creature(&mut g, P1, 0, 9);
    fight(&mut g, vec![d], vec![(d, b)]);
    assert_eq!(counters(&g, j2), 4);
}

#[test]
fn a_first_striker_that_kills_its_blocker_triggers_once_and_the_dead_blocker_deals_nothing() {
    let mut g = tbl();
    let a = creature(&mut g, P0, 3, 3);
    give(&mut g, a, "first strike");
    let ja = jitte(&mut g, P0, a, 0);
    let b = creature(&mut g, P1, 5, 3);
    let jb = jitte(&mut g, P1, b, 0);
    fight(&mut g, vec![a], vec![(a, b)]);
    assert!(g.perm(a).on_bf && !g.perm(b).on_bf, "first strike kills the blocker before it deals damage");
    assert_eq!(counters(&g, ja), 2);
    assert_eq!(counters(&g, jb), 0, "the blocker never dealt combat damage");
}

#[test]
fn trample_damage_to_the_blocker_and_the_player_in_one_step_is_one_trigger() {
    let mut g = tbl();
    let a = creature(&mut g, P0, 6, 6);
    give(&mut g, a, "trample");
    let j = jitte(&mut g, P0, a, 0);
    let b = creature(&mut g, P1, 0, 2);
    let life = g.player(P1).life;
    fight(&mut g, vec![a], vec![(a, b)]);
    assert_eq!(g.player(P1).life, life - 4, "4 tramples over the 2/2");
    assert_eq!(counters(&g, j), 2, "one damage step, one trigger");
}

#[test]
fn combat_damage_to_a_planeswalker_triggers_it() {
    // the person's attacker sent at a planeswalker: no damage to the player, two counters
    let mut g = tbl();
    let a = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, a, 0);
    let w = perm(&mut g, P1, "Karn Liberated");
    let loyalty = g.perm(w).loyalty.unwrap();
    seat(&mut g, P0, vec![]);
    g.human_walker_atk = vec![(a, w)];
    let life = g.player(P1).life;
    fight(&mut g, vec![a], vec![]);
    assert_eq!(g.perm(w).loyalty, Some(loyalty - 2), "the damage went to the planeswalker");
    assert_eq!(g.player(P1).life, life);
    assert_eq!(counters(&g, j), 2);
}

#[test]
fn pair_steps_follow_the_damage_steps() {
    // (hits, kills, first strike, double strike) of each creature
    let plain = (true, false, false, false);
    let lethal = (true, true, false, false);
    // two ordinary creatures trade: one regular step each
    assert_eq!(pair_steps(lethal, lethal), (1, 1));
    // a first striker that kills: the other never deals damage
    assert_eq!(pair_steps((true, true, true, false), lethal), (1, 0));
    // both first strike: both deal, both once (first strikers deal no second-step damage)
    assert_eq!(pair_steps((true, true, true, false), (true, true, true, false)), (1, 1));
    // a double striker against something it does not kill and that does not kill it: two steps; the other, one
    assert_eq!(pair_steps((true, false, true, true), plain), (2, 1));
    // a double striker whose first hit kills: nothing left for its second step
    assert_eq!(pair_steps((true, true, true, true), plain), (1, 0));
    // a creature that deals no damage triggers nothing
    assert_eq!(pair_steps((false, false, false, false), plain), (0, 1));
    // damage that is prevented is not dealt
    assert_eq!(pair_steps(plain, (false, false, false, false)), (1, 0));
}

#[test]
fn the_counters_stay_when_the_equipment_moves_and_when_the_creature_dies() {
    let mut g = tbl();
    let c1 = creature(&mut g, P0, 3, 3);
    let c2 = creature(&mut g, P0, 1, 1);
    let j = jitte(&mut g, P0, c1, 2);
    lands(&mut g, P0, "Plains", 2, false);
    assert!(sim_core::dsl::equip(&mut g, P0, j, c2, 2).unwrap());
    assert_eq!(g.perm(j).attached, Some(c2));
    assert_eq!(counters(&g, j), 2, "moving it keeps the counters");
    zones::die(&mut g, c2, "destroy").unwrap();
    assert!(g.perm(j).on_bf, "the Equipment stays when its creature dies");
    assert_eq!(counters(&g, j), 2, "and so do the counters");
    // it still collects counters once it is worn again
    assert_eq!(g.perm(j).attached.filter(|&m| g.perm(m).on_bf), None);
    g.perm_mut(j).attached = Some(c1);
    fight(&mut g, vec![c1], vec![]);
    assert_eq!(counters(&g, j), 4);
}

// ================================================================== the three modes (a person)
fn use_mode(g: &mut Game, j: PermId, word: &str, then: Vec<F>) {
    let (i, k) = label_index(g, P0, j, word);
    let mut a = vec![act(json!({"do": "use", "perm": i})), pick(k)];
    a.extend(then);
    main(g, P0, a);
}

#[test]
fn a_person_pumps_the_equipped_creature_plus_two_plus_two() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 2);
    use_mode(&mut g, j, "+2/+2", vec![]);
    assert_eq!((epow(&g, c), etgh(&g, c)), (4, 4));
    assert_eq!(counters(&g, j), 1, "one counter removed");
}

#[test]
fn a_person_shrinks_a_creature_and_a_one_toughness_creature_dies() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 2);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    let big = creature(&mut g, P1, 3, 3);
    let pick_elf: F = Box::new(|r| Answer::Pick(r.choices.iter().position(|x| x.contains("Llanowar")).unwrap()));
    use_mode(&mut g, j, "-1/-1", vec![pick_elf]);
    assert!(!g.perm(elf).on_bf, "a 1/1 with -1/-1 dies");
    assert_eq!(counters(&g, j), 1);
    // a bigger creature just shrinks until end of turn
    let pick_big: F = Box::new(|r| Answer::Pick(r.choices.iter().position(|x| x.contains("3/3")).expect("the 3/3")));
    use_mode(&mut g, j, "-1/-1", vec![pick_big]);
    assert!(g.perm(big).on_bf);
    assert_eq!((epow(&g, big), etgh(&g, big)), (2, 2));
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn a_person_gains_two_life() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 1);
    let life = g.player(P0).life;
    use_mode(&mut g, j, "gain 2 life", vec![]);
    assert_eq!(g.player(P0).life, life + 2);
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn with_no_counters_a_person_is_offered_no_mode() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 0);
    seat(&mut g, P0, vec![]);
    let labels: Vec<String> = abilities_of(&mut g, P0, j).unwrap().into_iter().map(|a| a.0).collect();
    assert!(labels.iter().all(|l| l.starts_with("Equip")), "only Equip: {labels:?}");
    g.perm_mut(j).data.set(DataKey::Charge, Val::Int(1));
    let labels: Vec<String> = abilities_of(&mut g, P0, j).unwrap().into_iter().map(|a| a.0).collect();
    assert_eq!(labels.len(), 4, "Equip and the three modes with a counter: {labels:?}");
}

// ================================================================== the AI
#[test]
fn the_ai_kills_a_one_toughness_creature_with_a_counter() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 1);
    let birds = perm(&mut g, P1, "Thalia, Guardian of Thraben");
    let opts = options_of(&mut g, j, P0);
    let o = opts.iter().find(|o| o.label == "Umezawa's Jitte -1/-1").unwrap_or_else(|| {
        panic!(
            "no kill offered: {:?} (worth {})",
            opts.iter().map(|o| &o.label).collect::<Vec<_>>(),
            sim_core::engine::values::pval(&g, birds)
        )
    });
    assert!(ai_act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(birds).on_bf);
    assert_eq!(counters(&g, j), 0);
}

#[test]
fn the_ai_gains_life_only_when_it_is_low() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 2);
    assert!(options_of(&mut g, j, P0).is_empty(), "nothing to kill and life is high");
    g.player_mut(P0).life = 6;
    let opts = options_of(&mut g, j, P0);
    let o = opts.iter().find(|o| o.label.contains("gain 2")).expect("life gain at 6");
    assert!(ai_act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).life, 8);
    assert_eq!(counters(&g, j), 1);
}

#[test]
fn the_ai_pumps_its_equipped_attacker_to_win_the_fight() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 2, 2);
    let j = jitte(&mut g, P0, c, 2);
    let b = creature(&mut g, P1, 3, 3);
    fight(&mut g, vec![c], vec![(c, b)]);
    assert!(g.perm(c).on_bf, "pumped to 4/4: it survives 3 damage");
    assert!(!g.perm(b).on_bf, "and kills the 3/3");
    assert_eq!(counters(&g, j), 2 - 1 + 2, "one counter spent, two more for the damage it dealt");
}

#[test]
fn the_ai_pumps_its_equipped_blocker_too() {
    let mut g = tbl();
    let a = creature(&mut g, P0, 3, 3);
    let b = creature(&mut g, P1, 2, 2);
    let j = jitte(&mut g, P1, b, 2);
    fight(&mut g, vec![a], vec![(a, b)]);
    assert!(g.perm(b).on_bf && !g.perm(a).on_bf, "the blocker pumps to 4/4 and wins");
    assert_eq!(counters(&g, j), 3);
}

#[test]
fn the_ai_spends_nothing_when_the_fight_is_already_won() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 5, 5);
    let j = jitte(&mut g, P0, c, 2);
    let b = creature(&mut g, P1, 1, 1);
    fight(&mut g, vec![c], vec![(c, b)]);
    assert_eq!(counters(&g, j), 4, "no counter spent; two gained");
}

#[test]
fn the_ai_pumps_an_unblocked_attacker_for_lethal() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 1, 1);
    jitte(&mut g, P0, c, 1);
    g.player_mut(P1).life = 3;
    fight(&mut g, vec![c], vec![]);
    assert!(g.player(P1).life <= 0, "1 + 2 = 3: lethal (life {})", g.player(P1).life);
}

#[test]
fn the_ai_keeps_its_counters_when_the_pump_is_not_lethal() {
    let mut g = tbl();
    let c = creature(&mut g, P0, 1, 1);
    let j = jitte(&mut g, P0, c, 1);
    g.player_mut(P1).life = 20;
    fight(&mut g, vec![c], vec![]);
    assert_eq!(g.player(P1).life, 19);
    assert_eq!(counters(&g, j), 3, "kept the counter and gained two");
}
