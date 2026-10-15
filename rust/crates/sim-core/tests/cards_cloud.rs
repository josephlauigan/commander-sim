//! The cloud deck's cards (impls/cloud.rs) on hand-built positions. Seat 0 plays the cloud deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, combat, hooks, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::cloud;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["cloud", "seph"])
}

/// the library is exactly these cards (so the tests don't depend on the Cloud deck list)
fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn equipment_in_hand(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p)
        .hand
        .iter()
        .filter(|&&c| g.db.get(c).has_subtype("equipment"))
        .map(|&c| g.db.get(c).name.to_string())
        .collect()
}

/// Cloud on the battlefield, not sick, with these Equipment attached
fn cloud_with(g: &mut Game, eq: &[&str]) -> (PermId, Vec<PermId>) {
    let cl = perm(g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    let es: Vec<PermId> = eq.iter().map(|n| perm(g, P0, n)).collect();
    for &e in &es {
        g.perm_mut(e).attached = Some(cl);
    }
    (cl, es)
}

fn attack_triggers(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) {
    combat::attack_triggers(g, p, atk, d).unwrap();
}

fn fire_attack(g: &mut Game, atk: &[PermId]) {
    hooks::fire_trigger(g, Event::Attack, Call::Attack { p: P0, atk: atk.to_vec(), d: P1 }).unwrap();
}

// ------------------------------------------------------------------ Cloud, Midgar Mercenary
#[test]
fn cloud_finds_a_cheap_sword_when_no_equipment_is_out() {
    // (the tutor now ranks by what the mana allows: cast + equip by next turn; the cheap Swords come first)
    let mut g = game();
    set_library(
        &mut g,
        P0,
        &[
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Sword of Feast and Famine",
            "Champion's Helm",
            "Colossus Hammer",
            "Shield of Kaldra",
            "Wrecking Ball Arm",
        ],
    );
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(equipment_in_hand(&g, P0), vec!["Sword of Fire and Ice"]);
}

#[test]
fn cloud_finds_the_next_piece_with_one_out() {
    let mut g = game();
    set_library(
        &mut g,
        P0,
        &[
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Sword of Feast and Famine",
            "Champion's Helm",
            "Colossus Hammer",
            "Shield of Kaldra",
            "Wrecking Ball Arm",
        ],
    );
    lands(&mut g, P0, "Plains", 7, false);
    hand(&mut g, P0, &["Sword of Fire and Ice"]);
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert!(
        equipment_in_hand(&g, P0).contains(&"Sword of Feast and Famine".to_string()),
        "one Sword held: the next Sword is Feast and Famine (cloud-aifix3 F4; it was Hearth and Home)"
    );
}

#[test]
fn cloud_finds_the_third_kaldra_piece() {
    let mut g = game();
    set_library(&mut g, P0, &["Shield of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"]);
    perm(&mut g, P0, "Sword of Kaldra");
    perm(&mut g, P0, "Helm of Kaldra");
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(equipment_in_hand(&g, P0), vec!["Shield of Kaldra"]);
}

#[test]
fn cloud_s_own_enter_trigger_is_not_doubled() {
    let mut g = game();
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(equipment_in_hand(&g, P0).len(), 1, "an unequipped Cloud finds one Equipment");
}

#[test]
fn an_equipped_cloud_doubles_mask_of_memory() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Mask of Memory"]);
    let before = g.player(P0).hand.len();
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[]).unwrap();
    // draw two, discard one, twice
    assert_eq!(g.player(P0).hand.len(), before + 2, "Mask of Memory triggers twice: 2 net cards");
}

#[test]
fn another_creature_wearing_mask_of_memory_is_not_doubled() {
    let mut g = game();
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let a = perm(&mut g, P0, "Thalia, the Survivor");
    let mask = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(a);
    let before = g.player(P0).hand.len();
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1, "once: 1 net card");
}

#[test]
fn an_unequipped_cloud_has_nothing_to_double() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert!(!cloud::doubles(&g, cl));
    let (cl2, es) = cloud_with(&mut g, &["Medic's Kitesail"]);
    assert!(cloud::doubles(&g, cl2) && cloud::doubles(&g, es[0]));
}

#[test]
fn conformer_shuriken_taps_two_blockers_on_cloud() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Conformer Shuriken"]);
    let a = perm(&mut g, P1, "Grave Titan");
    let b = perm(&mut g, P1, "Lyra Dawnbringer");
    attack_triggers(&mut g, P0, &[cl], P1);
    assert!(g.perm(a).tapped && g.perm(b).tapped, "two taps");
    assert!(g.perm(cl).plus > 0, "counters for the power difference");
}

#[test]
fn conformer_shuriken_taps_one_on_another_creature() {
    let mut g = game();
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let w = perm(&mut g, P0, "Thalia, the Survivor");
    let s = perm(&mut g, P0, "Conformer Shuriken");
    g.perm_mut(s).attached = Some(w);
    let a = perm(&mut g, P1, "Grave Titan");
    let b = perm(&mut g, P1, "Lyra Dawnbringer");
    attack_triggers(&mut g, P0, &[w], P1);
    assert_eq!([a, b].iter().filter(|&&m| g.perm(m).tapped).count(), 1);
}

#[test]
fn explorer_s_scope_doubles_on_cloud_and_puts_lands_in_play() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Explorer's Scope"]);
    let land = take(&mut g, P0, "Plains");
    let land2 = take(&mut g, P0, "Plains");
    g.player_mut(P0).library.push(land);
    g.player_mut(P0).library.push(land2);
    let n = g.player(P0).lands.len();
    attack_triggers(&mut g, P0, &[cl], P1);
    assert_eq!(g.player(P0).lands.len(), n + 2, "two looks, two lands");
    assert!(g.player(P0).lands.iter().rev().take(2).all(|&l| g.land(l).tapped));
}

#[test]
fn explorer_s_scope_leaves_a_nonland_on_top() {
    let mut g = game();
    let w = perm(&mut g, P0, "Thalia, the Survivor");
    let s = perm(&mut g, P0, "Explorer's Scope");
    g.perm_mut(s).attached = Some(w);
    let top = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).library.push(top);
    let n = g.player(P0).lands.len();
    attack_triggers(&mut g, P0, &[w], P1);
    assert_eq!(g.player(P0).lands.len(), n);
    assert_eq!(g.player(P0).library.last(), Some(&top));
}

#[test]
fn medic_s_kitesail_gains_life_twice_on_cloud() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Medic's Kitesail"]);
    let life = g.player(P0).life;
    attack_triggers(&mut g, P0, &[cl], P1);
    assert_eq!(g.player(P0).life, life + 2);
}

#[test]
fn the_kaldra_pieces_work_on_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let (cl, es) = cloud_with(&mut g, &["Sword of Kaldra", "Shield of Kaldra"]);
    assert_eq!(epow(&g, cl), 7, "Sword of Kaldra: 2/1 to 7/6");
    let helm = perm(&mut g, P0, "Helm of Kaldra");
    let o = options(&mut g, helm, P0, Some(false));
    assert!(!o.is_empty());
    assert!(run(&mut g, P0, &o[0]));
    let k = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).token && g.perm(m).data.truthy(sim_core::state::DataKey::Kaldra))
        .expect("the Kaldra token");
    assert_eq!(g.perm(es[0]).attached, Some(k), "the Sword moved onto Kaldra");
    assert!(g.perm(cl).on_bf, "Cloud is untouched");
}

// ------------------------------------------------------------------ Hero's Blade, Armory Automaton, Beatrix
#[test]
fn hero_s_blade_attaches_to_an_entering_legend() {
    let mut g = game();
    let blade = perm(&mut g, P0, "Hero's Blade");
    assert_eq!(g.perm(blade).attached, None);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(g.perm(blade).attached, Some(cl));
    assert_eq!((epow(&g, cl), etgh(&g, cl)), (5, 3));
}

#[test]
fn hero_s_blade_ignores_a_nonlegendary_creature() {
    let mut g = game();
    let blade = perm(&mut g, P0, "Hero's Blade");
    perm(&mut g, P0, "Bastion Protector");
    assert_eq!(g.perm(blade).attached, None);
}

#[test]
fn armory_automaton_takes_loose_equipment_on_entering_and_attacking() {
    let mut g = game();
    let a = perm(&mut g, P0, "Mask of Memory");
    let b = perm(&mut g, P0, "Medic's Kitesail");
    let am = perm(&mut g, P0, "Armory Automaton");
    assert_eq!((g.perm(a).attached, g.perm(b).attached), (Some(am), Some(am)));
    let c = perm(&mut g, P0, "Explorer's Scope");
    attack_triggers(&mut g, P0, &[am], P1);
    assert_eq!(g.perm(c).attached, Some(am), "attacking attaches the new one");
}

#[test]
fn armory_automaton_leaves_cloud_s_equipment_alone() {
    let mut g = game();
    let (cl, es) = cloud_with(&mut g, &["Mask of Memory"]);
    let am = perm(&mut g, P0, "Armory Automaton");
    assert_eq!(g.perm(es[0]).attached, Some(cl));
    let _ = am;
}

#[test]
fn beatrix_attaches_everything_to_the_best_wearer_at_combat() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    let bx = perm(&mut g, P0, "Beatrix, Loyal General");
    let a = perm(&mut g, P0, "Mask of Memory");
    let b = perm(&mut g, P0, "Explorer's Scope");
    hooks::fire_trigger(&mut g, Event::CombatStart, Call::Player { p: P0 }).unwrap();
    assert_eq!((g.perm(a).attached, g.perm(b).attached), (Some(cl), Some(cl)));
    let _ = bx;
}

// ------------------------------------------------------------------ Puresteel Paladin, equip costs
#[test]
fn puresteel_paladin_draws_for_each_equipment() {
    let mut g = game();
    perm(&mut g, P0, "Puresteel Paladin");
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Mask of Memory");
    assert_eq!(g.player(P0).hand.len(), n + 1);
    perm(&mut g, P0, "Bastion Protector");
    assert_eq!(g.player(P0).hand.len(), n + 1, "a creature isn't an Equipment");
}

#[test]
fn equip_costs_follow_the_cards() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let plain = perm(&mut g, P0, "Thalia, the Survivor");
    let wba = perm(&mut g, P0, "Wrecking Ball Arm");
    assert_eq!(cloud::equip_cost(&g, P0, wba, cl, 7), 3, "legend {{3}}");
    assert_eq!(cloud::equip_cost(&g, P0, wba, plain, 7), 3, "Thalia is legendary too");
    let tok = token(&mut g, P0, 1);
    assert_eq!(cloud::equip_cost(&g, P0, wba, tok, 7), 7);
    let hulk = perm(&mut g, P0, "Hulkbuster Armor");
    assert_eq!(cloud::equip_cost(&g, P0, hulk, tok, 6), 6);
    g.perm_mut(tok).ttypes.push("hero");
    assert_eq!(cloud::equip_cost(&g, P0, hulk, tok, 6), 3, "Equip Hero {{3}}");
    let heli = perm(&mut g, P0, "Helitrooper");
    assert_eq!(cloud::equip_cost(&g, P0, hulk, heli, 6), 4, "Helitrooper: {{2}} less");
    let pb = perm(&mut g, P0, "Veteran's Powerblade");
    assert_eq!(cloud::equip_cost(&g, P0, pb, cl, 2), 1, "Cloud is a Soldier: {{W}}");
}

#[test]
fn puresteel_paladin_makes_equip_free_with_three_artifacts() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    perm(&mut g, P0, "Puresteel Paladin");
    let hammer = perm(&mut g, P0, "Colossus Hammer");
    assert_eq!(cloud::equip_cost(&g, P0, hammer, cl, 8), 8, "two artifacts: no metalcraft");
    perm(&mut g, P0, "Mask of Memory");
    perm(&mut g, P0, "Medic's Kitesail");
    assert_eq!(cloud::equip_cost(&g, P0, hammer, cl, 8), 0, "three artifacts: equip {{0}}");
}

#[test]
fn the_equip_option_uses_the_changed_cost_and_picks_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    let other = perm(&mut g, P0, "Thalia, the Survivor");
    g.perm_mut(other).sick = false;
    let wba = perm(&mut g, P0, "Wrecking Ball Arm");
    let o = brain::main_options(&mut g, P0, false).unwrap();
    let eq = o.iter().find(|x| x.label == "equip Wrecking Ball Arm").expect("equip {3} for a legend is affordable");
    assert!(run(&mut g, P0, eq));
    assert_eq!(g.perm(wba).attached, Some(cl));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "paid {{3}}");
}

// ------------------------------------------------------------------ the sizing and evasion Equipment
#[test]
fn wrecking_ball_arm_sets_base_7_7_and_stacks_with_a_sword() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Wrecking Ball Arm"]);
    assert_eq!((epow(&g, cl), etgh(&g, cl)), (7, 7));
    let sw = perm(&mut g, P0, "Sword of Kaldra");
    g.perm_mut(sw).attached = Some(cl);
    assert_eq!((epow(&g, cl), etgh(&g, cl)), (12, 12));
}

#[test]
fn wrecking_ball_arm_can_t_be_blocked_by_power_2_or_less() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Wrecking Ball Arm"]);
    let small = token(&mut g, P1, 2);
    let big = token(&mut g, P1, 3);
    assert!(!combat::can_block(&g, small, cl));
    assert!(combat::can_block(&g, big, cl));
}

#[test]
fn hulkbuster_armor_is_9_9_and_flies() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Hulkbuster Armor"]);
    assert_eq!((epow(&g, cl), etgh(&g, cl)), (9, 9));
    assert!(sim_core::dsl::has_kw(&g, cl, "flying"));
    let ground = token(&mut g, P1, 3);
    assert!(!combat::can_block(&g, ground, cl));
}

#[test]
fn colossus_hammer_adds_10_and_removes_flying() {
    let mut g = game();
    let heli = perm(&mut g, P0, "Helitrooper");
    assert!(sim_core::dsl::has_kw(&g, heli, "flying"));
    let h = perm(&mut g, P0, "Colossus Hammer");
    g.perm_mut(h).attached = Some(heli);
    assert_eq!((epow(&g, heli), etgh(&g, heli)), (11, 12));
    assert!(!sim_core::dsl::has_kw(&g, heli, "flying"), "loses flying");
    let ground = token(&mut g, P1, 3);
    assert!(combat::can_block(&g, ground, heli), "a ground creature can block it now");
}

#[test]
fn trailblazer_s_boots_need_a_nonbasic_land_on_the_other_side() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["Trailblazer's Boots"]);
    let b = token(&mut g, P1, 5);
    lands(&mut g, P1, "Plains", 1, false);
    assert!(combat::can_block(&g, b, cl), "only basic lands: blockable");
    lands(&mut g, P1, "Command Tower", 1, false);
    assert!(!combat::can_block(&g, b, cl), "a nonbasic land: unblockable");
}

// ------------------------------------------------------------------ the Equipment that attaches itself
#[test]
fn magitek_scythe_attaches_and_the_wearer_must_be_blocked() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let s = perm(&mut g, P0, "Magitek Scythe");
    assert_eq!(g.perm(s).attached, Some(cl));
    assert!(sim_core::dsl::has_kw(&g, cl, "first strike"));
    let b = token(&mut g, P1, 1);
    let mut assign = vec![];
    cloud::force_blocks(&g, P1, &[cl], &[], &mut assign);
    assert_eq!(assign, vec![(cl, b)]);
}

#[test]
fn squire_s_lightblade_attaches_for_first_strike() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let s = perm(&mut g, P0, "Squire's Lightblade");
    assert_eq!(g.perm(s).attached, Some(cl));
    assert!(sim_core::dsl::has_kw(&g, cl, "first strike"));
    assert_eq!(epow(&g, cl), 3);
}

#[test]
fn white_mage_s_staff_makes_a_hero_wearing_it() {
    let mut g = game();
    let s = perm(&mut g, P0, "White Mage's Staff");
    let hero = g.perm(s).attached.expect("the Hero token wears it");
    assert!(g.perm(hero).token);
    assert!(sim_core::engine::values::has_type(&g, hero, "hero"));
    assert_eq!((epow(&g, hero), etgh(&g, hero)), (2, 2));
}

// ------------------------------------------------------------------ the creatures
#[test]
fn cid_makes_equipment_cheaper_and_flies_on_your_turn() {
    let mut g = game();
    let cid = perm(&mut g, P0, "Cid, Freeflier Pilot");
    let sw = card(&g, "Sword of Kaldra");
    let (generic, _) = sim_core::engine::mana::cost_of(&g, P0, sw);
    assert_eq!(generic, 3, "Sword of Kaldra costs {{1}} less");
    assert!(sim_core::dsl::has_kw(&g, cid, "flying"), "my turn");
    g.active = Some(P1);
    assert!(!sim_core::dsl::has_kw(&g, cid, "flying"), "not on the other turn");
}

#[test]
fn cid_returns_an_equipment_from_the_graveyard() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let cid = perm(&mut g, P0, "Cid, Freeflier Pilot");
    g.perm_mut(cid).sick = false;
    let c = take(&mut g, P0, "Mask of Memory");
    g.player_mut(P0).gy.push(c);
    let o = options(&mut g, cid, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(in_hand(&g, P0, "Mask of Memory") && !in_gy(&g, P0, "Mask of Memory"));
    assert!(g.perm(cid).tapped);
}

#[test]
fn helitrooper_gives_another_attacker_flying() {
    let mut g = game();
    let h = perm(&mut g, P0, "Helitrooper");
    let t = perm(&mut g, P0, "Thalia, the Survivor");
    fire_attack(&mut g, &[h, t]);
    assert!(sim_core::dsl::has_kw(&g, t, "flying"));
}

#[test]
fn magnetic_snuffler_returns_an_equipment_wearing_itself() {
    let mut g = game();
    let c = take(&mut g, P0, "Mask of Memory");
    g.player_mut(P0).gy.push(c);
    let s = perm(&mut g, P0, "Magnetic Snuffler");
    let e = on_bf(&g, P0, "Mask of Memory").expect("returned to the battlefield");
    assert_eq!(g.perm(e).attached, Some(s));
}

#[test]
fn magnetic_snuffler_grows_when_you_sacrifice_an_artifact() {
    let mut g = game();
    let s = perm(&mut g, P0, "Magnetic Snuffler");
    let before = epow(&g, s);
    let a = perm(&mut g, P0, "Mask of Memory");
    zones::die(&mut g, a, "sac").unwrap();
    assert_eq!(epow(&g, s), before + 1);
}

#[test]
fn jocasta_grows_when_the_commander_connects() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jocasta, Automaton Avenger");
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).is_cmd = true;
    g.perm_mut(cl).sick = false;
    let before = epow(&g, j);
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[]).unwrap();
    assert_eq!(epow(&g, j), before + 1);
}

#[test]
fn jocasta_returns_attacking_from_the_graveyard() {
    let mut g = game();
    let c = take(&mut g, P0, "Jocasta, Automaton Avenger");
    g.player_mut(P0).gy.push(c);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).is_cmd = true;
    g.perm_mut(cl).sick = false;
    let new = combat::attack_triggers(&mut g, P0, &[cl], P1).unwrap();
    let j = on_bf(&g, P0, "Jocasta, Automaton Avenger").expect("back on the battlefield");
    assert!(new.contains(&j) && g.perm(j).tapped);
}

#[test]
fn zack_fair_enters_with_a_counter_and_passes_everything_on() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, "Zack Fair");
    assert_eq!((epow(&g, z), etgh(&g, z)), (1, 2), "a +1/+1 counter");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = Some(z);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert!(cloud::zack_sacrifice(&mut g, z, P0, cl.0 as i64).unwrap());
    assert!(!g.perm(z).on_bf);
    assert_eq!(g.perm(e).attached, Some(cl), "the Equipment moves");
    assert_eq!(g.perm(cl).plus, 1, "the counter moves");
    assert!(sim_core::engine::values::indestructible(&g, cl));
}

#[test]
fn unfinished_business_returns_cloud_with_two_equipment() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    for n in ["Cloud, Midgar Mercenary", "Mask of Memory", "Explorer's Scope", "Sword of Kaldra"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let c: CardId = hand(&mut g, P0, &["Unfinished Business"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let cl = on_bf(&g, P0, "Cloud, Midgar Mercenary").expect("Cloud is back");
    assert_eq!(cloud::equipment_on(&g, cl).len(), 2, "up to two Equipment");
    assert!(in_gy(&g, P0, "Unfinished Business"));
}

#[test]
fn the_deck_plays_a_game_to_the_end() {
    let g = play(&["cloud", "seph"], 3, false);
    assert!(g.over || g.round > 0);
}

// ------------------------------------------------------------------ White Mage's Staff, Veteran's Powerblade
#[test]
fn white_mage_s_staff_gains_life_when_the_wearer_attacks_twice_on_cloud() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &["White Mage's Staff"]);
    // Cloud wears the Staff too (the Hero token arrived with it): two copies of the attack trigger
    let staff = on_bf(&g, P0, "White Mage's Staff").unwrap();
    g.perm_mut(staff).attached = Some(cl);
    let life = g.player(P0).life;
    attack_triggers(&mut g, P0, &[cl], P1);
    assert_eq!(g.player(P0).life, life + 2);
    assert_eq!((epow(&g, cl), etgh(&g, cl)), (3, 2), "+1/+1");
}

// ------------------------------------------------------------------ the plan
#[test]
fn cloud_is_the_play_on_turn_two() {
    let mut g = game();
    g.player_mut(P0).turns = 2;
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Thalia, the Survivor", "Helitrooper", "Squire's Lightblade"]);
    let o = brain::main_options(&mut g, P0, false).unwrap();
    let cloud =
        o.iter().find(|x| x.label == "Cloud, Midgar Mercenary").expect("Cloud is castable from the command zone");
    let best = o.iter().map(|x| x.utility).fold(f64::MIN, f64::max);
    assert!((cloud.utility - best).abs() < 1e-9, "Cloud has the highest utility");
    assert!(o.iter().all(|x| x.utility < cloud.utility - 3.0 || x.label == cloud.label), "by a wide margin");
}

#[test]
fn an_equipment_that_attaches_on_entering_waits_for_a_creature() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    hand(&mut g, P0, &["Squire's Lightblade", "Magitek Scythe"]);
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(!o.iter().any(|x| x.label == "Squire's Lightblade" || x.label == "Magitek Scythe"), "no body, no cast");
    perm(&mut g, P0, "Thalia, the Survivor");
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(o.iter().any(|x| x.label == "Squire's Lightblade"));
}

#[test]
fn an_unequipped_cloud_stays_home_when_he_can_be_blocked_and_goes_when_equipped() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    token(&mut g, P1, 1);
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert!(out.is_empty(), "a bare 2/1 against a blocker stays home");
    let e = perm(&mut g, P0, "Medic's Kitesail");
    g.perm_mut(e).attached = Some(cl);
    let out = cloud::attack_filter(&mut g, P0, vec![cl], P1);
    assert_eq!(out, vec![cl]);
}

#[test]
fn an_unequipped_cloud_attacks_into_an_empty_board() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cl).sick = false;
    assert_eq!(cloud::attack_filter(&mut g, P0, vec![cl], P1), vec![cl]);
}

#[test]
fn entering_again_is_worth_more_to_a_bare_cloud_than_a_laden_one() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let bare = cloud::reenter_value(&g, P0, cl);
    assert!(bare > 0.0, "his search finds an Equipment");
    for n in ["Sword of Kaldra", "Mask of Memory", "Colossus Hammer"] {
        let e = perm(&mut g, P0, n);
        g.perm_mut(e).attached = Some(cl);
    }
    assert!(cloud::reenter_value(&g, P0, cl) < bare, "he lets go of what he wears");
    let other = perm(&mut g, P0, "Helitrooper");
    assert_eq!(cloud::reenter_value(&g, P0, other), 0.0, "only Cloud");
}

// ------------------------------------------------------------------ protecting Cloud
fn destroy(g: &mut Game, m: PermId) {
    sim_core::engine::removal::apply_removal(g, Some(P1), m, "destroy", None).unwrap();
}

#[test]
fn take_up_the_shield_saves_cloud_from_a_destroy_spell() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    hand(&mut g, P0, &["Take Up the Shield"]);
    destroy(&mut g, cl);
    assert!(g.perm(cl).on_bf, "indestructible");
    assert_eq!(g.perm(cl).plus, 1, "and a +1/+1 counter");
    assert!(in_gy(&g, P0, "Take Up the Shield"));
}

#[test]
fn zack_fair_saves_cloud_and_gives_him_the_equipment_and_counters() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let z = perm(&mut g, P0, "Zack Fair");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = Some(z);
    destroy(&mut g, cl);
    assert!(g.perm(cl).on_bf && !g.perm(z).on_bf);
    assert_eq!(g.perm(e).attached, Some(cl));
}

#[test]
fn a_creature_that_is_not_worth_it_is_left_to_die() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, "Thalia, the Survivor");
    hand(&mut g, P0, &["Take Up the Shield"]);
    destroy(&mut g, t);
    assert!(!g.perm(t).on_bf);
    assert!(in_hand(&g, P0, "Take Up the Shield"));
}

#[test]
fn unbreakable_formation_answers_a_destroy_spell_when_nothing_cheaper_is_there() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    hand(&mut g, P0, &["Unbreakable Formation"]);
    destroy(&mut g, cl);
    assert!(g.perm(cl).on_bf);
}

#[test]
fn an_edict_cannot_be_answered() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    hand(&mut g, P0, &["Take Up the Shield"]);
    assert!(!cloud::cloud_protect(&mut g, P0, cl, "edict", Some(P1), None).unwrap());
}

// ------------------------------------------------------------------ the plan (cloud-plan)
#[test]
fn bounced_cloud_stays_in_hand_without_tax() {
    let mut g = game();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.player_mut(P0).cmd_in_zone = false;
    zones::bounce(&mut g, cl).unwrap();
    assert!(in_hand(&g, P0, "Cloud, Midgar Mercenary"), "903.9b: he stays in hand");
    assert!(!g.player(P0).cmd_in_zone);
    assert_eq!(g.player(P0).tax, 0);
}

#[test]
fn search_bonus_favours_casting_cloud_only() {
    use sim_core::hooks::Action;
    let mut g = game();
    let c = g.player(P0).cmd;
    let cast =
        Opt { utility: 5.0, label: "Cloud, Midgar Mercenary".into(), act: Some(Action::Cast { card: c, zone: None }) };
    let stop = Opt { utility: 0.0, label: "stop".into(), act: None };
    assert!(cloud::search_bonus(&g, P0, &cast) > 0.0);
    assert_eq!(cloud::search_bonus(&g, P0, &stop), 0.0);
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    assert_eq!(cloud::search_bonus(&g, P0, &cast), 0.0, "no bonus once he is out");
}

#[test]
fn plan_milestone_needs_an_equipped_cloud() {
    let mut g = game();
    let (cl, _) = cloud_with(&mut g, &[]);
    g.player_mut(P0).turns = 4;
    cloud::plan_milestone(&mut g, P0, &[cl]);
    assert!(g.player(P0).milestone.get("cloud").is_none(), "bare Cloud: not yet");
    let e = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(e).attached = Some(cl);
    cloud::plan_milestone(&mut g, P0, &[cl]);
    assert_eq!(g.player(P0).milestone.get("cloud"), Some(&4));
}
