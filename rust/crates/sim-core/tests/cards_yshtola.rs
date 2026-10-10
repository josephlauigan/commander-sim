//! Y'shtola's cards and AI plan (`impls/yshtola.rs`): Python's tests/test_yshtola.py (classes Yshtola, Curiosity,
//! Drain, Stolen, OtherCards, AI) and test_my_cards.PollutedBonds, plus tests of the plan's choices.
//!
//! Not ported here: test_yshtola.Practice (a person at the table: phase 9). Ignored until their card code is merged:
//! the two Sanguine Bond tests (t4.py's gain-life hook, ported with t4) and Zur fetching for this deck (Zur's attack
//! trigger is t5.py / zur.py); `cargo test -- --ignored` runs them.

use sim_core::ai::{self, act, brain, decks};
use sim_core::engine::{cast, combat, hooks, life, mana, removal, turn, values, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{common, yshtola as ysh_mod};
use sim_core::rng::Rng;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn three() -> Game {
    table(&["yshtola", "veyran", "sauron"])
}

fn two() -> Game {
    table(&["yshtola", "veyran"])
}

/// Y'shtola onto p's battlefield as the commander
fn ysh(g: &mut Game, p: PlayerId) -> PermId {
    let cmd = g.player(p).cmd;
    let m = zones::enter(g, p, cmd, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

/// cast a card from hand (already paid for)
fn cast_x(g: &mut Game, p: PlayerId, name: &str, x: i32) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() }).unwrap();
    c
}

fn cast(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    cast_x(g, p, name, 0)
}

/// these cards on top of p's library, the last one on top
fn stack_top(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).library.push(c);
    }
}

fn lives(g: &Game) -> Vec<i32> {
    g.players.iter().map(|q| q.life).collect()
}

fn combat_damage(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) {
    hooks::fire_trigger(g, Event::CombatDamage, Call::CombatDamage { p, a, d, dmg }).unwrap();
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

/// a hand card's own options (hand_options)
fn hand_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn has_perm(g: &Game, p: PlayerId, c: CardId) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).cd == Some(c))
}

// ======================================================== Y'shtola
#[test]
fn a_noncreature_spell_of_mana_value_three_drains_each_opponent() {
    let mut g = three();
    ysh(&mut g, P0);
    cast(&mut g, P0, "Propaganda");
    assert_eq!(lives(&g), vec![42, 38, 38]);
    assert_eq!(g.player(P0).milestone.get("yshtola"), Some(&g.player(P0).turns));
}

#[test]
fn small_spells_and_creatures_dont() {
    let mut g = three();
    ysh(&mut g, P0);
    cast(&mut g, P0, "Arcane Signet"); // mana value 2
    cast(&mut g, P0, "Marauding Blight-Priest"); // a creature
    assert_eq!(lives(&g), vec![40, 40, 40]);
}

#[test]
fn x_counts_toward_the_mana_value() {
    let mut g = three();
    ysh(&mut g, P0);
    cast_x(&mut g, P0, "Exsanguinate", 1); // {1}{B}{B}: mana value 3
    assert_eq!((g.player(P1).life, g.player(P2).life), (37, 37)); // 1 from Exsanguinate, 2 from Y'shtola
    assert_eq!(g.player(P0).life, 44);
}

#[test]
fn opponents_spells_dont() {
    let mut g = three();
    ysh(&mut g, P0);
    cast(&mut g, P1, "Propaganda");
    assert_eq!(lives(&g), vec![40, 40, 40]);
}

#[test]
fn end_step_draw_when_a_player_lost_four_life() {
    let mut g = three();
    ysh(&mut g, P0);
    let n = g.player(P0).hand.len();
    life::lose_life(&mut g, P2, 2, Some(P1), "other", None).unwrap();
    life::lose_life(&mut g, P2, 1, Some(P1), "other", None).unwrap();
    assert_eq!(ysh_mod::lost_this_turn(&g, P2), 3); // counted once each
    turn::end_step(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
    life::lose_life(&mut g, P2, 1, Some(P1), "other", None).unwrap();
    turn::end_step(&mut g, P1).unwrap(); // anyone's end step; any player, you included
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn vigilance() {
    let mut g = two();
    let m = ysh(&mut g, P0);
    assert!(g.perm(m).vig);
}

// ======================================================== Curiosity
#[test]
fn curiosity_on_yshtola_draws_for_each_opponent_her_trigger_hits() {
    let mut g = three();
    let m = ysh(&mut g, P0);
    perm(&mut g, P0, "Curiosity");
    assert!(common::auras_on(&g, m).iter().any(|&a| g.perm(a).name == "Curiosity")); // the AI's host
    let n = g.player(P0).hand.len();
    cast(&mut g, P0, "Propaganda");
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn curiosity_combat_damage_draws() {
    let mut g = two();
    let t = perm(&mut g, P0, "Thief of Sanity");
    g.attach_to = Some(t);
    perm(&mut g, P0, "Curiosity");
    g.attach_to = None;
    let n = g.player(P0).hand.len();
    combat_damage(&mut g, P0, t, P1, 2);
    assert!(g.player(P0).hand.len() >= n + 1);
}

#[test]
fn curiosity_goes_on_your_best_evasive_creature_without_yshtola() {
    let mut g = two();
    perm(&mut g, P0, "Marauding Blight-Priest");
    let t = perm(&mut g, P0, "Thief of Sanity"); // flying
    let a = perm(&mut g, P0, "Curiosity");
    assert_eq!(g.perm(a).attached, Some(t));
}

// ======================================================== the drain package
#[test]
fn blight_priest_and_sanguine_bond() {
    let mut g = three();
    perm(&mut g, P0, "Marauding Blight-Priest");
    perm(&mut g, P0, "Sanguine Bond");
    life::gain(&mut g, P0, 3).unwrap();
    let mut l = vec![g.player(P1).life, g.player(P2).life];
    l.sort();
    assert_eq!(l, vec![36, 39]); // 1 each (Priest), 3 more to one (Bond)
}

#[test]
fn sanguine_bond_takes_the_kill() {
    let mut g = three();
    perm(&mut g, P0, "Sanguine Bond");
    perm(&mut g, P2, "Grave Titan"); // Sauron is the bigger threat
    g.player_mut(P1).life = 3;
    life::gain(&mut g, P0, 3).unwrap();
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P2).life, 40);
}

#[test]
fn blight_priest_drains_when_you_gain_life() {
    let mut g = three();
    perm(&mut g, P0, "Marauding Blight-Priest");
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(lives(&g), vec![43, 39, 39]);
    life::gain(&mut g, P1, 3).unwrap(); // an opponent's gain doesn't
    assert_eq!(lives(&g), vec![43, 42, 39]);
}

#[test]
fn debt_to_the_deathless() {
    let mut g = three();
    cast_x(&mut g, P0, "Debt to the Deathless", 3);
    assert_eq!(lives(&g), vec![52, 34, 34]);
}

#[test]
fn the_ai_casts_an_x_drain_for_the_kill() {
    let mut g = three();
    lands(&mut g, P0, "Swamp", 6, false);
    let c = hand(&mut g, P0, &["Exsanguinate"])[0];
    g.player_mut(P1).life = 4;
    let (u, label) = ysh_mod::x_drain_option(&g, P0, c).unwrap();
    assert!(label.contains("X = 4"), "{label}");
    assert!(u > 8.0);
    let o = hand_options(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, label);
    assert!(perform(&mut g, P0, &o[0]));
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P2).life, 36);
}

#[test]
fn no_small_x_without_a_kill() {
    let mut g = three();
    lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &["Exsanguinate"])[0];
    assert!(ysh_mod::x_drain_option(&g, P0, c).is_none());
}

#[test]
fn ill_gotten_inheritance() {
    let mut g = three();
    let igi = perm(&mut g, P0, "Ill-Gotten Inheritance");
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert_eq!(lives(&g), vec![41, 39, 39]);
    lands(&mut g, P0, "Swamp", 6, false);
    g.player_mut(P1).life = 4;
    let opts = options(&mut g, igi, P0, Some(true));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(!g.player(P1).alive);
    assert!(!g.perm(igi).on_bf);
    assert_eq!(g.player(P0).life, 45);
}

#[test]
fn ill_gotten_inheritance_waits_without_a_kill() {
    // no kill: only late (turn 9 on), at an opponent's end step
    let mut g = three();
    let igi = perm(&mut g, P0, "Ill-Gotten Inheritance");
    lands(&mut g, P0, "Swamp", 6, false);
    g.player_mut(P0).turns = 9;
    assert!(options(&mut g, igi, P0, Some(true)).is_empty());
    assert!(options(&mut g, igi, P0, None).is_empty()); // your own turn
    g.active = Some(P1);
    let o = options(&mut g, igi, P0, None);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].utility, 1.5);
}

#[test]
fn urborg_syphon_mage() {
    let mut g = three();
    let m = perm(&mut g, P0, "Urborg Syphon-Mage");
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Island"]);
    g.player_mut(P1).life = 2;
    let opts = options(&mut g, m, P0, None);
    assert!(!opts.is_empty() && opts[0].utility > 8.0); // a kill
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P2).life, 38);
    assert_eq!(g.player(P0).life, 44);
    assert!(g.perm(m).tapped);
    assert!(g.player(P0).gy.contains(&card(&g, "Island")));
}

#[test]
fn polluted_bonds_opponent_land_drains() {
    let mut g = two();
    perm(&mut g, P0, "Polluted Bonds");
    let c = take(&mut g, P1, "Island");
    g.player_mut(P1).hand.push(c);
    g.active = Some(P1);
    turn::play_land(&mut g, P1).unwrap();
    assert_eq!((g.player(P1).life, g.player(P0).life), (38, 42));
}

#[test]
fn polluted_bonds_own_land_does_nothing() {
    let mut g = two();
    perm(&mut g, P0, "Polluted Bonds");
    let c = take(&mut g, P0, "Plains");
    g.player_mut(P0).hand.push(c);
    turn::play_land(&mut g, P0).unwrap();
    assert_eq!((g.player(P1).life, g.player(P0).life), (40, 40));
}

// ======================================================== cards taken from opponents
#[test]
fn gonti_takes_a_card_castable_with_any_mana() {
    let mut g = two();
    stack_top(&mut g, P1, &["Guttersnipe", "Island", "Island", "Island"]);
    perm(&mut g, P0, "Gonti, Lord of Luxury");
    let c = card(&g, "Guttersnipe");
    assert!(g.player(P0).hand.contains(&c));
    assert_eq!(mana::cost_of(&g, P0, c), (3, String::new())); // {2}{R}: the red paid with any mana
    let island = card(&g, "Island");
    assert_eq!(g.player(P1).library[..3].iter().filter(|&&x| x == island).count(), 3); // the rest on the bottom
    lands(&mut g, P0, "Swamp", 3, false);
    mana::pay(&mut g, P0, 3, "", false).unwrap();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let m = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).cd == Some(c)).unwrap();
    assert_eq!(g.perm(m).orig, P1); // still Veyran's card
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P1).gy.contains(&c));
    assert!(!g.player(P0).gy.contains(&c));
}

#[test]
fn a_taken_spell_goes_to_its_owners_graveyard() {
    let mut g = two();
    let c = card(&g, "Night's Whisper");
    ysh_mod::take_card(&mut g, P0, c, P1, "test");
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P1).gy.contains(&c));
}

#[test]
fn taken_cards_arent_discarded_to_hand_size() {
    let mut g = two();
    let c = card(&g, "Guttersnipe");
    ysh_mod::take_card(&mut g, P0, c, P1, "test");
    hand(&mut g, P0, &["Island", "Plains", "Swamp", "Sol Ring", "Arrest", "Curiosity", "Propaganda"]);
    turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).hand.contains(&c));
    assert_eq!(g.player(P0).hand.len(), 8);
}

#[test]
fn hostage_taker_returns_the_creature_when_it_leaves() {
    let mut g = two();
    let gs = perm(&mut g, P1, "Guttersnipe");
    let t = perm(&mut g, P0, "Hostage Taker");
    let c = card(&g, "Guttersnipe");
    assert!(!g.perm(gs).on_bf);
    assert!(g.player(P0).hand.contains(&c));
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(has_perm(&g, P1, c));
    assert!(!g.player(P0).hand.contains(&c));
    assert!(!g.player(P0).stolen.contains_key(&c));
}

#[test]
fn hostage_taker_cast_it_and_keep_it() {
    let mut g = two();
    perm(&mut g, P1, "Guttersnipe");
    let t = perm(&mut g, P0, "Hostage Taker");
    lands(&mut g, P0, "Swamp", 3, false);
    mana::pay(&mut g, P0, 3, "", false).unwrap();
    let c = card(&g, "Guttersnipe");
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    zones::die(&mut g, t, "destroy").unwrap();
    let m = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).cd == Some(c)).unwrap();
    assert_eq!(g.perm(m).orig, P1);
    assert!(!has_perm(&g, P1, c));
}

#[test]
fn hostage_taker_leaves_tokens_and_small_things_alone() {
    // the AI's target is the best opposing creature or artifact worth 1.5 or more
    let mut g = two();
    let tok = token(&mut g, P1, 1);
    let t = perm(&mut g, P0, "Hostage Taker");
    assert!(g.perm(tok).on_bf);
    assert!(g.perm(t).data.get(sim_core::state::DataKey::Hostage).is_none());
}

#[test]
fn thief_of_sanity() {
    let mut g = two();
    stack_top(&mut g, P1, &["Island", "Guttersnipe", "Island"]);
    let t = perm(&mut g, P0, "Thief of Sanity");
    combat_damage(&mut g, P0, t, P1, 2);
    assert!(g.player(P0).hand.contains(&card(&g, "Guttersnipe")));
    let island = card(&g, "Island");
    assert_eq!(g.player(P1).gy.iter().filter(|&&c| c == island).count(), 2);
}

// ======================================================== the deck's other cards
#[test]
fn jesters_cap() {
    let mut g = two();
    let cap = perm(&mut g, P0, "Jester's Cap");
    let n = g.player(P1).library.len();
    ysh_mod::cap_use(&mut g, P0, cap, P1).unwrap();
    assert_eq!(g.player(P1).library.len(), n - 3);
    assert_eq!(g.player(P1).exile.len(), 3);
    assert!(!g.perm(cap).on_bf);
}

#[test]
fn jesters_cap_waits_for_main_two_or_an_opponents_end_step() {
    let mut g = two();
    let cap = perm(&mut g, P0, "Jester's Cap");
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(options(&mut g, cap, P0, Some(false)).is_empty()); // main 1
    assert!(options(&mut g, cap, P0, None).is_empty()); // your own end step
    let o = options(&mut g, cap, P0, Some(true));
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Jester's Cap on Veyran");
    assert!(perform(&mut g, P0, &o[0]));
    assert_eq!(g.player(P1).exile.len(), 3);
}

#[test]
fn dark_petition_spell_mastery() {
    let mut g = two();
    let n = g.player(P0).hand.len();
    cast(&mut g, P0, "Dark Petition");
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).floating.b, 0); // no instants or sorceries in the graveyard yet
    let (nw, rtb) = (take(&mut g, P0, "Night's Whisper"), take(&mut g, P0, "Read the Bones"));
    g.player_mut(P0).gy.extend([nw, rtb]);
    cast(&mut g, P0, "Diabolic Tutor"); // (a plain tutor, for comparison)
    let dp = card(&g, "Dark Petition");
    let pl = g.player_mut(P0);
    pl.hand.push(dp);
    let i = pl.gy.iter().position(|&c| c == dp).unwrap();
    pl.gy.remove(i);
    cast::cast_card(&mut g, P0, dp, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).floating.b, 3);
}

#[test]
fn plea_for_guidance_finds_two_enchantments() {
    let mut g = two();
    cast(&mut g, P0, "Plea for Guidance");
    let n = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).types.has(sim_core::cards::Types::ENCHANTMENT)).count();
    assert_eq!(n, 2);
}

#[test]
fn take_up_the_shield_saves_yshtola() {
    let mut g = two();
    let m = ysh(&mut g, P0);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Take Up the Shield"]);
    let gftt = card(&g, "Go for the Throat");
    removal::apply_removal(&mut g, Some(P1), m, "destroy", Some(gftt)).unwrap();
    assert!(g.perm(m).on_bf);
    assert_eq!(g.perm(m).plus, 1);
    assert!(g.perm(m).eot_kw.contains(&"lifelink"));
    assert!(g.perm(m).eot_kw.contains(&"indestructible"));
}

#[test]
fn take_up_the_shield_as_a_spell() {
    let mut g = two();
    let m = ysh(&mut g, P0);
    let c = hand(&mut g, P0, &["Take Up the Shield"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(m), ..Ctx::default() }).unwrap();
    assert_eq!(g.perm(m).plus, 1);
    assert!(g.perm(m).eot_kw.contains(&"indestructible"));
}

#[test]
fn zur_in_the_99_fetches_for_this_deck() {
    let mut g = two();
    let y = ysh(&mut g, P0);
    let z = perm(&mut g, P0, "Zur the Enchanter");
    let keep: Vec<CardId> = g
        .player(P0)
        .library
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            !(d.types.has(sim_core::cards::Types::ENCHANTMENT) && d.cmc <= 3)
                || matches!(&*d.name, "Curiosity" | "Propaganda")
        })
        .collect();
    g.player_mut(P0).library = keep;
    combat::attack_triggers(&mut g, P0, &[z], P1).unwrap();
    let got: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(sim_core::cards::Types::ENCHANTMENT)))
        .collect();
    assert_eq!(got.iter().map(|&m| g.perm(m).name).collect::<Vec<_>>(), vec!["Curiosity"]);
    assert_eq!(g.perm(got[0]).attached, Some(y));
}

#[test]
fn zurs_fetch_values_for_this_deck() {
    // what the Zur port asks this deck (yshtola.fetch_value): Curiosity for Y'shtola, Necropotence early
    let mut g = two();
    let z = perm(&mut g, P0, "Zur the Enchanter");
    let cur = card(&g, "Curiosity");
    assert_eq!(ysh_mod::fetch_value(&g, P0, cur, z), 1.5);
    ysh(&mut g, P0);
    assert_eq!(ysh_mod::fetch_value(&g, P0, cur, z), 4.5); // 3 + 1.5 per opponent
    let necro = card(&g, "Necropotence");
    assert!((ysh_mod::fetch_value(&g, P0, necro, z) - 8.2).abs() < 1e-9);
    perm(&mut g, P0, "Propaganda");
    assert_eq!(ysh_mod::fetch_value(&g, P0, card(&g, "Propaganda"), z), 0.0); // already out
}

#[test]
fn yshtola_doesnt_chump_block_unless_it_would_kill_you() {
    let mut g = table(&["veyran", "yshtola"]);
    let m = ysh(&mut g, P1);
    let big = perm(&mut g, P0, "Grave Titan");
    for i in 0..20 {
        // 6 damage at 10 life: other decks chump half the time
        g.rng = Rng::from_u64(i);
        g.player_mut(P1).life = 10;
        combat::resolve_combat(&mut g, P0, &[big], P1, &[]).unwrap();
        assert!(g.perm(m).on_bf);
        assert_eq!(g.player(P1).life, 4);
    }
}

// ======================================================== the AI
#[test]
fn a_spell_that_triggers_yshtola_ranks_higher() {
    let mut g = three();
    let c = card(&g, "Read the Bones");
    let before = ysh_mod::yshtola_prio(&g, P0, c);
    ysh(&mut g, P0);
    assert!(ysh_mod::yshtola_prio(&g, P0, c) > before);
    assert_eq!(ai::deck_prio(&g, P0, c), ysh_mod::yshtola_prio(&g, P0, c)); // the deck's priority is this plan's
}

#[test]
fn tutors_find_sanguine_bond() {
    let mut g = two();
    g.player_mut(P0).turns = 8;
    g.player_mut(P0).life = 20;
    assert_eq!(ai::tutor_pick(&g, P0, "any"), Some(card(&g, "Sanguine Bond")));
    perm(&mut g, P0, "Sanguine Bond");
    assert_ne!(ai::tutor_pick(&g, P0, "ench"), Some(card(&g, "Sanguine Bond")));
}

#[test]
fn idyllic_tutor_uses_the_wish_list() {
    let mut g = two();
    g.player_mut(P0).turns = 8;
    g.player_mut(P0).life = 20;
    cast(&mut g, P0, "Idyllic Tutor");
    assert!(g.player(P0).hand.contains(&card(&g, "Sanguine Bond")));
}

#[test]
fn tutors_find_the_finisher_when_it_kills() {
    let mut g = three();
    lands(&mut g, P0, "Swamp", 9, false);
    g.player_mut(P1).life = 6;
    g.player_mut(P2).life = 6;
    let pick = ai::tutor_pick(&g, P0, "any").unwrap();
    assert!(["Debt to the Deathless", "Exsanguinate"].contains(&&*g.db.get(pick).name));
}

#[test]
fn the_x_drain_is_a_main_phase_play() {
    let mut g = three();
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Exsanguinate"]);
    g.player_mut(P1).life = 4;
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == "Exsanguinate (X = 4)").expect("the X drain");
    assert!(matches!(o.act, Some(Action::Plan { .. })));
    assert!(!opts.iter().any(|o| o.label == "Exsanguinate")); // not cast by priority
}

#[test]
fn a_taken_card_is_cast_by_its_generic_priority() {
    let mut g = two();
    let c = card(&g, "Guttersnipe");
    ysh_mod::take_card(&mut g, P0, c, P1, "test");
    let v = sim_core::ai::pool::generic_prio(&g, P0, c) as f64;
    assert!(v > 0.0);
    assert_eq!(ysh_mod::yshtola_prio(&g, P0, c), v);
}

#[test]
fn hostage_taker_priority_from_its_target() {
    let mut g = two();
    let ht = card(&g, "Hostage Taker");
    assert_eq!(ysh_mod::yshtola_prio(&g, P0, ht), 25.0); // nothing to take
    let gs = perm(&mut g, P1, "Grave Titan");
    let want = 45 + 30.min((6.0 * values::pval(&g, gs)) as i32);
    assert_eq!(ysh_mod::yshtola_prio(&g, P0, ht), want as f64);
}

#[test]
fn champions_helm_goes_on_yshtola() {
    let mut g = two();
    let helm = perm(&mut g, P0, "Champion's Helm");
    let y = ysh(&mut g, P0);
    lands(&mut g, P0, "Island", 1, false);
    assert!(options(&mut g, helm, P0, None).is_empty()); // not at the end-of-turn window
    let o = options(&mut g, helm, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "equip Champion's Helm to Y'shtola, Night's Blessed");
    assert!(perform(&mut g, P0, &o[0]));
    assert_eq!(g.perm(helm).attached, Some(y));
}

#[test]
fn champions_helm_is_yshtolas_alone() {
    let mut g = table(&["sauron", "yshtola"]);
    let helm = perm(&mut g, P0, "Champion's Helm");
    perm(&mut g, P0, "Sauron, the Dark Lord");
    lands(&mut g, P0, "Island", 1, false);
    assert!(options(&mut g, helm, P0, Some(false)).is_empty());
}

#[test]
fn the_wipe_answer_is_zurs() {
    // Y'shtola has no wipe answer in hand: no response; with Clever Concealment her creatures phase out
    let mut g = two();
    let y = ysh(&mut g, P0);
    perm(&mut g, P0, "Thief of Sanity");
    perm(&mut g, P0, "Gonti, Lord of Luxury");
    assert_eq!(decks::wipe_response(&mut g, P0, "destroy", P1).unwrap(), None);
    hand(&mut g, P0, &["Clever Concealment"]);
    lands(&mut g, P0, "Plains", 4, false);
    let loss: f64 = g.player(P0).perms.iter().map(|&m| values::pval(&g, m)).sum();
    assert!(loss >= 6.0, "{loss}");
    assert_eq!(decks::wipe_response(&mut g, P0, "destroy", P1).unwrap(), Some("all"));
    assert!(g.perm(y).phased);
}

// ======================================================== whole games
#[test]
fn yshtola_plays_whole_games() {
    // against every tier: the games finish, and her cast trigger comes up
    let (mut triggers, mut games) = (0, 0);
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let pool = tier(t);
        for k in 0..6usize {
            let g = play(&["yshtola", pool[0], pool[1 + k % 4], pool[(2 + k) % 5]], 5_000 + k as u64, false);
            triggers += g.player(P0).stats.get("ysh_triggers").copied().unwrap_or(0);
            games += 1;
        }
    }
    assert_eq!(games, 30);
    assert!(triggers > 0, "Y'shtola never triggered");
}
