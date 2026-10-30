//! The Captain America precon (Tier 0): card code in impls/t0_captain.rs.

use sim_core::ai::act::perform;
use sim_core::ai::brain::{self, Situation};
use sim_core::engine::combat::{attack_triggers, can_block};
use sim_core::engine::mana::cost_of;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::die;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const CAP: &str = "captain-america-avengers-assemble";
const CAP_TL: &str = "Captain America, Team Leader";
const ANT: &str = "Ant-Man, Elusive Avenger";

fn tbl() -> Game {
    table(&[CAP, "kefka"])
}

fn hook_opts(g: &mut Game, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let s = Situation::new(g, p);
    brain::hook_options(g, p, &s, post).unwrap()
}

fn labels(o: &[Opt]) -> Vec<String> {
    o.iter().map(|o| o.label.clone()).collect()
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    sim_core::engine::cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn plus(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

// ------------------------------------------------------------------ Captain America, Team Leader
#[test]
fn team_leader_gives_another_hero_vigilance_haste_and_counters() {
    let mut g = tbl();
    let cap = perm(&mut g, P0, CAP_TL);
    let ant = perm(&mut g, P0, ANT);
    assert_eq!(plus(&g, ant), 1, "counter on the Hero");
    assert_eq!(plus(&g, cap), 1, "counter on Captain America");
    assert!(g.perm(ant).eot_kw.contains(&"vigilance") && g.perm(ant).eot_kw.contains(&"haste"));
    assert!(!g.perm(ant).sick);
    // a non-Hero does nothing
    let jarvis = perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler");
    assert_eq!((plus(&g, jarvis), plus(&g, cap)), (0, 1));
}

// ------------------------------------------------------------------ Captain America, Living Legend
#[test]
fn living_legend_untaps_a_creature_the_first_time_it_becomes_tapped_each_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Captain America, Living Legend");
    let t = token(&mut g, P0, 2);
    g.perm_mut(t).tapped = true;
    sim_core::impls::t0_captain::became_tapped(&mut g, t);
    assert!(!g.perm(t).tapped, "first tap: untapped");
    g.perm_mut(t).tapped = true;
    sim_core::impls::t0_captain::became_tapped(&mut g, t);
    assert!(g.perm(t).tapped, "second tap this turn stays tapped");
}

#[test]
fn living_legend_untaps_attackers_and_ignores_the_opponents_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Captain America, Living Legend");
    let a = perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler");
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    // Jarvis (power 1) attacks only if the AI chooses; whatever it did, nothing of ours stays tapped after attacking
    assert!(!g.perm(a).tapped);
    let mut g = tbl();
    perm(&mut g, P1, "Captain America, Living Legend");
    let t = token(&mut g, P1, 2);
    g.perm_mut(t).tapped = true;
    sim_core::impls::t0_captain::became_tapped(&mut g, t);
    assert!(g.perm(t).tapped, "not its controller's turn");
}

// ------------------------------------------------------------------ Avengers Quinjet
#[test]
fn quinjet_enter_puts_a_hero_from_hand_onto_the_battlefield() {
    let mut g = tbl();
    hand(&mut g, P0, &["Hawkeye, Avenging Archer"]);
    perm(&mut g, P0, "Avengers Quinjet");
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Hawkeye, Avenging Archer"));
    assert!(g.player(P0).hand.iter().all(|&c| &*g.db.get(c).name != "Hawkeye, Avenging Archer"));
}

#[test]
fn quinjet_returns_a_hero_from_the_graveyard_when_the_hand_has_none() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Hercules, Olympian Hero");
    g.player_mut(P0).gy.push(c);
    perm(&mut g, P0, "Avengers Quinjet");
    assert!(g.player(P0).hand.contains(&c));
}

#[test]
fn quinjet_attacks_with_a_trigger_and_crews_with_power_three() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let q = perm(&mut g, P0, "Avengers Quinjet");
    let crew = perm(&mut g, P0, "Hercules, Olympian Hero");
    g.perm_mut(crew).sick = true;
    sim_core::cardcode::turn_start(&mut g, P0).unwrap();
    // beginning of combat: the Crew hook
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Crew,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    assert!(g.is_creature(q), "crewed");
    assert!(g.perm(crew).tapped);
    assert_eq!((g.perm(q).pow, g.perm(q).tgh, g.perm(q).fly), (4, 4, true));
    let c = take(&mut g, P0, "Winter Soldier, Reborn Avenger");
    g.player_mut(P0).gy.push(c);
    attack_triggers(&mut g, P0, &[q], P1).unwrap();
    assert!(g.player(P0).hand.contains(&c), "attack trigger: Hero from the graveyard to hand");
}

// ------------------------------------------------------------------ Folk Hero, Jarvis, Fury, Horn
#[test]
fn folk_hero_draws_once_a_turn_for_a_spell_sharing_a_type_with_the_commander() {
    let mut g = tbl();
    perm(&mut g, P0, "Folk Hero");
    let cap = perm(&mut g, P0, CAP_TL);
    g.perm_mut(cap).is_cmd = true;
    g.perm_mut(cap).orig = P0;
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, ANT);
    assert_eq!(g.player(P0).hand.len(), n + 1, "Hero spell shares Hero with the commander: draw");
    cast_free(&mut g, P0, "Hawkeye, Avenging Archer");
    assert_eq!(g.player(P0).hand.len(), n + 1, "only once each turn");
}

#[test]
fn folk_hero_ignores_spells_sharing_no_type() {
    let mut g = tbl();
    perm(&mut g, P0, "Folk Hero");
    let cap = perm(&mut g, P0, CAP_TL);
    g.perm_mut(cap).is_cmd = true;
    g.perm_mut(cap).orig = P0;
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Sol Ring");
    assert_eq!(g.player(P0).hand.len(), n);
}

#[test]
fn jarvis_draws_when_a_hero_spell_is_cast() {
    let mut g = tbl();
    perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler");
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, ANT);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    cast_free(&mut g, P0, "Sol Ring");
    assert_eq!(g.player(P0).hand.len(), n + 1, "a non-Hero spell draws nothing");
}

#[test]
fn nick_fury_and_herald_horn_make_hero_spells_cheaper() {
    let mut g = tbl();
    let c = card(&g, "Thor, Asgard's Avenger");
    let base = cost_of(&g, P0, c).0 + cost_of(&g, P0, c).1.chars().count() as u32;
    perm(&mut g, P0, "Director Nick Fury");
    let one = cost_of(&g, P0, c).0 + cost_of(&g, P0, c).1.chars().count() as u32;
    assert_eq!(one, base - 1);
    perm(&mut g, P0, "Herald's Horn");
    let two = cost_of(&g, P0, c).0 + cost_of(&g, P0, c).1.chars().count() as u32;
    assert_eq!(two, base - 2);
    // other players' spells aren't cheaper
    let other = cost_of(&g, P1, c).0 + cost_of(&g, P1, c).1.chars().count() as u32;
    assert_eq!(other, base);
}

#[test]
fn nick_fury_attack_finds_a_hero_in_the_top_four() {
    let mut g = tbl();
    let fury = perm(&mut g, P0, "Director Nick Fury");
    let hero = take(&mut g, P0, "Hercules, Olympian Hero");
    g.player_mut(P0).library.push(hero);
    let n = g.player(P0).hand.len();
    let lib = g.player(P0).library.len();
    attack_triggers(&mut g, P0, &[fury], P1).unwrap();
    assert!(g.player(P0).hand.contains(&hero));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).library.len(), lib - 1, "the other three went to the bottom");
}

#[test]
fn herald_horn_takes_a_hero_from_the_top_at_upkeep() {
    let mut g = tbl();
    perm(&mut g, P0, "Herald's Horn");
    let hero = take(&mut g, P0, "Thor, Asgard's Avenger");
    g.player_mut(P0).library.push(hero);
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Upkeep,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    assert!(g.player(P0).hand.contains(&hero));
}

// ------------------------------------------------------------------ Gift of Immortality
#[test]
fn gift_returns_the_creature_then_the_aura_at_the_end_step() {
    let mut g = tbl();
    let host = perm(&mut g, P0, "Hercules, Olympian Hero");
    let hc = g.perm(host).cd.unwrap();
    g.attach_to = Some(host);
    let gift = perm(&mut g, P0, "Gift of Immortality");
    g.attach_to = None;
    assert_eq!(g.perm(gift).attached, Some(host));
    die(&mut g, host, "destroy").unwrap();
    let back = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(hc)).expect("returned");
    assert_ne!(back, host);
    assert!(g.player(P0).perms.iter().all(|&m| &*g.perm(m).name != "Gift of Immortality"), "Aura waits");
    sim_core::cardcode::delayed_end_step(&mut g, P0).unwrap();
    let gift2 =
        g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Gift of Immortality").expect("Aura back");
    assert_eq!(g.perm(gift2).attached, Some(back));
}

// ------------------------------------------------------------------ Love on the Battlefield
#[test]
fn love_on_the_battlefield_with_two_attackers() {
    let mut g = tbl();
    perm(&mut g, P0, "Love on the Battlefield");
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    let n = g.player(P0).hand.len();
    attack_triggers(&mut g, P0, &[a, b], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "draws a card");
    assert!(g.perm(a).eot_kw.contains(&"first strike") && g.perm(b).eot_kw.contains(&"first strike"));
    let c = token(&mut g, P0, 2);
    let n = g.player(P0).hand.len();
    attack_triggers(&mut g, P0, &[a, b, c], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "three attackers: nothing");
}

// ------------------------------------------------------------------ Ant-Man
#[test]
fn ant_man_cant_be_blocked_by_greater_power() {
    let mut g = tbl();
    let ant = perm(&mut g, P0, ANT);
    let big = token(&mut g, P1, 3);
    let small = token(&mut g, P1, 1);
    assert!(!can_block(&g, big, ant));
    assert!(can_block(&g, small, ant));
}

#[test]
fn ant_man_makes_treasures_equal_to_the_damage() {
    let mut g = tbl();
    let ant = perm(&mut g, P0, ANT);
    g.player_mut(P1).life = 40;
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[ant], P1, &[ant]).unwrap();
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).treasures, 1);
}

// ------------------------------------------------------------------ Avenge, Heroic Return
fn generic(g: &Game, p: PlayerId, name: &str) -> u32 {
    cost_of(g, p, card(g, name)).0
}

#[test]
fn avenge_is_cheaper_after_a_player_attacked_you() {
    let mut g = tbl();
    assert_eq!(generic(&g, P0, "Avenge"), 4);
    attack_triggers(&mut g, P1, &[], P0).unwrap();
    assert_eq!(generic(&g, P0, "Avenge"), 2);
    // when P1's next turn begins without an attack it is forgotten
    sim_core::cardcode::turn_start(&mut g, P1).unwrap();
    assert_eq!(generic(&g, P0, "Avenge"), 4);
}

#[test]
fn heroic_return_is_cheaper_while_a_creature_attacks_you() {
    let mut g = tbl();
    g.active = Some(P1);
    g.step = sim_core::state::Step::Combat;
    assert_eq!(generic(&g, P0, "Heroic Return"), 5);
    attack_triggers(&mut g, P1, &[], P0).unwrap();
    assert_eq!(generic(&g, P0, "Heroic Return"), 3);
}

#[test]
fn heroic_return_gives_a_hero_two_counters() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Hercules, Olympian Hero");
    g.player_mut(P0).gy.push(c);
    cast_free(&mut g, P0, "Heroic Return");
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    assert_eq!(plus(&g, m), 2);
}

// ------------------------------------------------------------------ Avengers Tower
#[test]
fn avengers_tower_makes_coloured_mana_for_hero_spells_only() {
    let mut g = tbl();
    lands(&mut g, P0, "Avengers Tower", 1, false);
    let hero = card(&g, ANT); // {1}{U}{R}
    let sol = card(&g, "Hero's Blade"); // {1}{R}... an artifact (not a Hero spell)
    use sim_core::engine::mana::can_pay;
    g.pay_for = Some(hero);
    assert!(can_pay(&g, P0, 0, "U", false), "any colour for a Hero spell");
    g.pay_for = Some(sol);
    assert!(!can_pay(&g, P0, 0, "U", false));
    assert!(can_pay(&g, P0, 1, "", false), "{{C}}");
    g.pay_for = None;
}

#[test]
fn avengers_tower_digs_for_a_hero() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let tower = lands(&mut g, P0, "Avengers Tower", 1, false)[0];
    let hero = take(&mut g, P0, "Thor, Asgard's Avenger");
    g.player_mut(P0).library.push(hero);
    let o = hook_opts(&mut g, P0, None);
    let dig = o.iter().find(|o| o.label.contains("Tower")).expect("offered at the end of turn");
    perform(&mut g, P0, dig.act.as_ref().unwrap()).unwrap();
    assert!(g.player(P0).hand.contains(&hero));
    assert!(g.perm(tower).tapped);
}

// ------------------------------------------------------------------ Dismantling Wave
#[test]
fn dismantling_wave_cycles_into_a_global_artifact_enchantment_wipe() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 8, false);
    hand(&mut g, P0, &["Dismantling Wave"]);
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P1, "Smothering Tithe");
    let mine = perm(&mut g, P0, "Arcane Signet");
    let creature = perm(&mut g, P1, "Hawkeye, Avenging Archer");
    let o = hook_opts(&mut g, P0, Some(true));
    let cyc = o.iter().find(|o| o.label.contains("Dismantling Wave")).expect("cycling offered");
    let n = g.player(P0).hand.len();
    perform(&mut g, P0, cyc.act.as_ref().unwrap()).unwrap();
    assert!(
        g.player(P1)
            .perms
            .iter()
            .all(|&m| !g.perm(m).on_bf || m == creature || &*g.perm(m).name == "Hawkeye, Avenging Archer")
    );
    assert!(!g.perm(mine).on_bf, "its own artifact goes too");
    assert!(g.perm(creature).on_bf);
    assert_eq!(g.player(P0).hand.len(), n, "discarded the card, drew one");
}

// ------------------------------------------------------------------ Hawkeye
#[test]
fn hawkeye_kills_a_one_toughness_creature_and_draws() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hawkeye, Avenging Archer");
    let victim = perm(&mut g, P1, "Black Widow, Agile Avenger");
    g.perm_mut(victim).tgh = 1;
    let o = hook_opts(&mut g, P0, Some(false));
    let o = o.iter().find(|o| o.label.starts_with("Hawkeye")).expect("ping offered");
    let n = g.player(P0).hand.len();
    perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert!(!g.perm(victim).on_bf);
    assert!(g.perm(h).tapped);
    assert_eq!(g.player(P0).hand.len(), n + 1, "draw for the creature Hawkeye damaged");
}

#[test]
fn hawkeye_draws_when_a_creature_he_hurt_in_combat_dies() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hawkeye, Avenging Archer"); // 3/4
    let blocker = token(&mut g, P1, 1);
    g.perm_mut(blocker).tgh = 2;
    let n = g.player(P0).hand.len();
    // Hawkeye blocks / is blocked: a fight in resolve_combat with an assignment is internal; use the hook pair directly
    sim_core::impls::t0_captain::pair_mark(&mut g, h, blocker);
    die(&mut g, blocker, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

// ------------------------------------------------------------------ Hercules
#[test]
fn hercules_attack_counter_and_indestructible() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hercules, Olympian Hero");
    attack_triggers(&mut g, P0, &[h], P1).unwrap();
    assert_eq!(plus(&g, h), 1);
    assert!(sim_core::engine::values::indestructible(&g, h));
    assert_eq!((epow(&g, h), etgh(&g, h)), (4, 4));
}

#[test]
fn hercules_counts_damage_taken_once_a_turn() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Hercules, Olympian Hero");
    let a = token(&mut g, P1, 2);
    sim_core::impls::t0_captain::pair_after(&mut g, a, h);
    assert_eq!(plus(&g, h), 2, "damage of 2: two counters");
    sim_core::impls::t0_captain::pair_after(&mut g, a, h);
    assert_eq!(plus(&g, h), 2, "only the first time each turn");
}

// ------------------------------------------------------------------ flash
#[test]
fn quicksilver_tapped_and_mar_vell_after_an_opponent_cast_give_flash() {
    use sim_core::impls::t0_captain::as_flash;
    let mut g = tbl();
    assert!(!as_flash(&g, P0));
    let q = perm(&mut g, P0, "Quicksilver, Speedster");
    assert!(!as_flash(&g, P0), "untapped");
    g.perm_mut(q).tapped = true;
    assert!(as_flash(&g, P0));
    let mut g = tbl();
    perm(&mut g, P0, "Captain Mar-Vell, Space-Born");
    assert!(!as_flash(&g, P0), "no opposing spell yet");
    cast_free(&mut g, P1, "Sol Ring");
    assert!(as_flash(&g, P0));
}

// ------------------------------------------------------------------ Raise the Palisade
#[test]
fn raise_the_palisade_keeps_our_type_and_bounces_theirs() {
    let mut g = tbl();
    let ours = perm(&mut g, P0, ANT); // Hero, Human, Rogue
    let jarvis = perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler");
    let theirs: Vec<PermId> = (0..3).map(|_| token(&mut g, P1, 4)).collect();
    cast_free(&mut g, P0, "Raise the Palisade");
    assert!(g.perm(ours).on_bf);
    assert!(g.perm(jarvis).on_bf, "Human is shared by both: kept");
    assert!(theirs.iter().all(|&m| !g.perm(m).on_bf), "their creatures are bounced (tokens vanish)");
}

#[test]
fn raise_the_palisade_is_not_cast_into_nothing() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Raise the Palisade"]);
    perm(&mut g, P0, ANT);
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(!labels(&o).iter().any(|l| l.starts_with("Raise the Palisade")));
    for n in ["Thor, Asgard's Avenger", "Professor Hulk", "Iron Man, Armored Avenger"] {
        perm(&mut g, P1, n);
    }
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(labels(&o).iter().any(|l| l.starts_with("Raise the Palisade")), "{:?}", labels(&o));
}

// ------------------------------------------------------------------ Shang-Chi
#[test]
fn shang_chi_grows_on_each_draw_and_the_tenth_counter_pays_off() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Shang-Chi and the Ten Rings");
    sim_core::engine::zones::draw(&mut g, P0, 3, false).unwrap();
    assert_eq!(plus(&g, s), 3);
    g.perm_mut(s).plus = 9;
    let n = g.player(P0).hand.len();
    let life = g.player(P0).life;
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(plus(&g, s), 10 + 5, "the five cards drawn add five more counters");
    assert_eq!(g.player(P0).hand.len(), n + 1 + 5);
    assert_eq!(g.player(P0).life, life + 5);
}

// ------------------------------------------------------------------ She-Hulk
#[test]
fn she_hulk_gives_other_heroes_trample_and_grows_blocked_heroes() {
    let mut g = tbl();
    perm(&mut g, P0, "She-Hulk, Wallbreaker");
    let hero = perm(&mut g, P0, ANT);
    let jarvis = perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler");
    assert!(sim_core::dsl::has_kw(&g, hero, "trample"));
    assert!(!sim_core::dsl::has_kw(&g, jarvis, "trample"));
    let b1 = token(&mut g, P1, 1);
    let b2 = token(&mut g, P1, 1);
    let mut assign = vec![(hero, b1), (hero, b2), (jarvis, b1)];
    sim_core::cardcode::blocks_hooks(&mut g, P0, &[hero, jarvis], P1, &mut assign).unwrap();
    assert_eq!(plus(&g, hero), 2, "two blockers: two counters");
    assert_eq!(plus(&g, jarvis), 0, "not a Hero");
}

// ------------------------------------------------------------------ Winter Soldier
#[test]
fn winter_soldier_returns_a_creature_up_to_his_power() {
    let mut g = tbl();
    let ws = perm(&mut g, P0, "Winter Soldier, Reborn Avenger"); // power 3
    let hero = take(&mut g, P0, "Ant-Man, Elusive Avenger"); // MV 3, Hero
    let big = take(&mut g, P0, "Thor, Asgard's Avenger"); // MV 5: too big
    g.player_mut(P0).gy.extend([hero, big]);
    attack_triggers(&mut g, P0, &[ws], P1).unwrap();
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(hero)).expect("returned");
    assert_eq!(plus(&g, m), 1, "a Hero gets an extra counter");
    assert!(g.player(P0).gy.contains(&big));
}

// ------------------------------------------------------------------ Captain Marvel
#[test]
fn captain_marvel_copies_counters_put_on_another_creature() {
    let mut g = tbl();
    let cm = perm(&mut g, P0, "Captain Marvel, Apex Avenger");
    let t = token(&mut g, P0, 2);
    sim_core::engine::repl::put_plus(&mut g, t, 2);
    assert_eq!((plus(&g, t), plus(&g, cm)), (2, 2));
    let kree = perm(&mut g, P0, "Captain Mar-Vell, Space-Born");
    sim_core::engine::repl::put_plus(&mut g, kree, 1);
    assert_eq!(plus(&g, cm), 2, "a Kree doesn't count");
}

// ------------------------------------------------------------------ Heroic Sacrifice
#[test]
fn heroic_sacrifice_redirects_combat_damage_to_the_chosen_creature() {
    let mut g = tbl();
    g.active = Some(P1);
    perm(&mut g, P0, "Captain Mar-Vell, Space-Born"); // 4/4
    perm(&mut g, P0, "Jarvis, Earth's Mightiest Butler"); // 1/4
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P1, 3)).collect();
    let life = g.player(P0).life;
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Heroic Sacrifice");
    sim_core::engine::combat::resolve_combat(&mut g, P1, &atk, P0, &atk).unwrap();
    assert_eq!(g.player(P0).life, life, "no damage to the player");
    let left: Vec<_> = g.player(P0).perms.iter().filter(|&&m| g.is_creature(m)).collect();
    assert_eq!(left.len(), 1, "one creature absorbed it and died");
    assert_eq!(g.player(P0).hand.len(), n + 1, "and its controller drew a card");
}

#[test]
fn avenge_destroys_all_creatures_and_gains_one_life_for_each() {
    let mut g = tbl();
    for _ in 0..2 {
        token(&mut g, P0, 1);
    }
    for _ in 0..4 {
        token(&mut g, P1, 1);
    }
    let life = g.player(P0).life;
    cast_free(&mut g, P0, "Avenge");
    assert!(g.players.iter().all(|q| q.perms.iter().all(|&m| !g.is_creature(m) || !g.perm(m).on_bf)));
    assert_eq!(g.player(P0).life, life + 6, "six creatures destroyed");
}

#[test]
fn shang_chi_pays_off_at_the_next_draw_when_other_counters_reached_ten() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Shang-Chi and the Ten Rings");
    sim_core::engine::repl::put_plus(&mut g, s, 10);
    let life = g.player(P0).life;
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.player(P0).life, life + 5);
}
