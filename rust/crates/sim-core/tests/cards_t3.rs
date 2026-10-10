//! Card code of t3.py (impls/t3.rs: the Tier 3 decks' cards and planeswalkers) on hand-built positions, each card's
//! clause from its rules text, and the two Rust-only fixes in rules.py and rules2.py cards the Tier 3 decks play
//! (Bloodchief's Thirst's kicker, Legion's Landing's Adanto).

use sim_core::ai::act::perform;
use sim_core::engine::{cast, combat, stack, zones};
use sim_core::hooks::{Action, Opt, Sacrificed};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{rules, t3};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, post).unwrap()
}

fn option_with(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>, word: &str) -> Action {
    options(g, src, p, post).into_iter().find(|o| o.label.contains(word)).and_then(|o| o.act).expect("the option")
}

fn resolve(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> &'static str {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, p, c, &ctx).unwrap()
}

fn tokens_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token).collect()
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

fn count_in(v: &[CardId], c: CardId) -> usize {
    v.iter().filter(|&&x| x == c).count()
}

// ------------------------------------------------------------------ fix: Bloodchief's Thirst's kicker
#[test]
fn bloodchiefs_thirst_kicked_destroys_a_big_creature() {
    // Python reads p.thirst_kicked, which nothing set: the AI paid {2}{B}{B} (its {B} and the kicker {2}{B}) at a
    // creature with mana value 3 or more, and nothing happened
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    lands(&mut g, P0, "Swamp", 4, false);
    let thirst = hand(&mut g, P0, &["Bloodchief's Thirst"])[0];
    let titan = perm(&mut g, P1, "Grave Titan");
    assert!(sim_core::ai::brain::cast_removal(&mut g, P0, thirst, &[titan], 0, None).unwrap());
    stack::settle_stack(&mut g).unwrap();
    assert!(g.player(P0).thirst_kicked);
    assert!(!g.perm(titan).on_bf);
    assert!(g.player(P0).gy.contains(&thirst));
}

#[test]
fn bloodchiefs_thirst_unkicked_still_destroys_a_small_creature() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    lands(&mut g, P0, "Swamp", 4, false);
    let thirst = hand(&mut g, P0, &["Bloodchief's Thirst"])[0];
    let elves = perm(&mut g, P1, "Llanowar Elves");
    assert!(sim_core::ai::brain::cast_removal(&mut g, P0, thirst, &[elves], 0, None).unwrap());
    stack::settle_stack(&mut g).unwrap();
    assert!(!g.player(P0).thirst_kicked);
    assert!(!g.perm(elves).on_bf);
}

#[test]
fn bloodchiefs_thirst_spares_a_big_creature_when_the_kick_was_not_recorded() {
    // the resolve itself still needs the kick for mana value 3 or more
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let titan = perm(&mut g, P1, "Grave Titan");
    g.player_mut(P0).thirst_kicked = false;
    resolve(&mut g, P0, "Bloodchief's Thirst", Ctx { target: Some(titan), ..Ctx::default() });
    assert!(g.perm(titan).on_bf);
}

// ------------------------------------------------------------------ fix: Legion's Landing's Adanto
#[test]
fn legions_landing_flips_into_adanto_which_makes_lifelink_tokens() {
    // Python's Adanto option was registered on the enchantment's name and read p.adanto, which nothing set
    let mut g = table(&["aurelia-boros-extra-combats", "sauron"]);
    perm(&mut g, P0, "Legion's Landing // Adanto, the First Fort");
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P0, 2)).collect();
    combat::attack_triggers(&mut g, P0, &atk, P1).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert!(named(&g, P0, "Legion's Landing // Adanto, the First Fort").is_empty());
    let adanto =
        *g.player(P0).lands.iter().find(|&&l| &*g.db.get(g.land(l).cd).name == "Adanto, the First Fort").unwrap();
    lands(&mut g, P0, "Plains", 3, false);
    assert!(
        sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap().iter().all(|o| o.label != "Adanto token")
    );
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    let a = o.into_iter().find(|o| o.label == "Adanto token").and_then(|o| o.act).expect("the Adanto option");
    let before = tokens_of(&g, P0).len();
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(g.land(adanto).tapped);
    let toks = tokens_of(&g, P0);
    assert_eq!(toks.len(), before + 1);
    let v = *toks.last().unwrap();
    assert!(g.perm(v).lifelink && g.perm(v).ttypes.contains(&"vampire"));
    assert_eq!((g.perm(v).pow, g.perm(v).tgh), (1, 1));
}

// ------------------------------------------------------------------ Korvold
#[test]
fn korvold_grows_and_draws_on_each_sacrifice() {
    let mut g = table(&["korvold-jund-sacrifice", "sauron"]);
    let k = perm(&mut g, P0, "Korvold, Fae-Cursed King");
    stack::settle_stack(&mut g).unwrap();
    // it entered: sacrificed the least valuable other permanent (none: a land)
    let plus = g.perm(k).plus;
    let hand0 = g.player(P0).hand.len();
    g.player_mut(P0).foods = 1;
    let f = g.registry.get(card(&g, "Korvold, Fae-Cursed King")).unwrap().sacrifice.unwrap();
    f(&mut g, k, P0, Sacrificed::Token("Food")).unwrap();
    assert_eq!(g.perm(k).plus, plus + 1);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    f(&mut g, k, P0, Sacrificed::Perm(k)).unwrap(); // not itself
    assert_eq!(g.perm(k).plus, plus + 1);
}

#[test]
fn sac_worst_permanent_spends_food_first_then_the_cheapest() {
    let mut g = table(&["korvold-jund-sacrifice", "sauron"]);
    g.player_mut(P0).foods = 1;
    g.player_mut(P0).treasures = 1;
    let elf = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(t3::sac_worst_permanent(&mut g, P0, None).unwrap(), Some(Sacrificed::Token("Food")));
    assert_eq!(t3::sac_worst_permanent(&mut g, P0, None).unwrap(), Some(Sacrificed::Token("Treasure")));
    assert_eq!(t3::sac_worst_permanent(&mut g, P0, None).unwrap(), Some(Sacrificed::Perm(elf)));
    assert!(!g.perm(elf).on_bf);
}

#[test]
fn mayhem_devil_pings_on_any_sacrifice() {
    let mut g = table(&["korvold-jund-sacrifice", "sauron"]);
    perm(&mut g, P0, "Mayhem Devil");
    g.player_mut(P1).treasures = 1;
    g.player_mut(P1).life = 1;
    t3::sac_worst_permanent(&mut g, P1, None).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P1).treasures, 0);
    assert!(g.player(P1).life <= 0); // 1 damage at the player it kills
}

#[test]
fn fable_goes_through_its_chapters() {
    let mut g = table(&["korvold-jund-sacrifice", "sauron"]);
    let f = perm(&mut g, P0, "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki");
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.perm(f).data.get(DataKey::Lore), Some(&Val::Int(1)));
    let gob = tokens_of(&g, P0);
    assert_eq!(gob.len(), 1);
    assert!(g.perm(gob[0]).data.truthy(DataKey::FableGoblin));
    hand(&mut g, P0, &["Forest", "Mountain"]);
    let up =
        g.registry.get(card(&g, "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki")).unwrap().upkeep.unwrap();
    up(&mut g, f, P0).unwrap(); // chapter II: rummage two
    assert_eq!(g.perm(f).data.get(DataKey::Lore), Some(&Val::Int(2)));
    assert_eq!(g.player(P0).gy.len(), 2);
    assert_eq!(g.player(P0).hand.len(), 2);
    up(&mut g, f, P0).unwrap(); // chapter III: Reflection
    assert!(g.perm(f).data.truthy(DataKey::Reflection));
    assert!(g.perm(f).data.get(DataKey::Lore).is_none());
}

#[test]
fn witchs_oven_cooks_a_spare_creature_when_deaths_pay() {
    let mut g = table(&["korvold-jund-sacrifice", "sauron"]);
    let oven = perm(&mut g, P0, "Witch's Oven");
    perm(&mut g, P0, "Grave Pact");
    perm(&mut g, P1, "Grave Titan");
    let t = token(&mut g, P0, 1);
    let a = option_with(&mut g, oven, P0, Some(false), "Witch's Oven");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(!g.perm(t).on_bf && g.perm(oven).tapped);
    assert_eq!(g.player(P0).foods, 1);
}

// ------------------------------------------------------------------ Marwyn
#[test]
fn craterhoof_pumps_every_creature_and_gives_trample() {
    let mut g = table(&["marwyn-mono-green-elves", "sauron"]);
    let elves: Vec<PermId> = (0..3).map(|_| token(&mut g, P0, 1)).collect();
    let hoof = perm(&mut g, P0, "Craterhoof Behemoth");
    stack::settle_stack(&mut g).unwrap();
    for &m in elves.iter().chain([&hoof]) {
        assert_eq!(g.perm(m).eot_pt, (4, 4));
        assert!(g.perm(m).eot_kw.contains(&"trample"));
    }
    assert!(g.player(P0).trample);
}

#[test]
fn natural_order_wants_craterhoof_only_on_a_wide_board() {
    let mut g = table(&["marwyn-mono-green-elves", "sauron"]);
    let no = card(&g, "Natural Order");
    let prio = g.registry.get(no).unwrap().prio.unwrap();
    perm(&mut g, P0, "Llanowar Elves");
    assert!(g.player(P0).library.contains(&card(&g, "Craterhoof Behemoth")));
    let narrow = prio(&g, P0, no);
    assert_ne!(narrow, 85);
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    assert_eq!(prio(&g, P0, no), 85);
}

#[test]
fn marwyn_grows_with_each_elf() {
    let mut g = table(&["marwyn-mono-green-elves", "sauron"]);
    let m = perm(&mut g, P0, "Marwyn, the Nurturer");
    perm(&mut g, P0, "Llanowar Elves");
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.perm(m).plus, 1);
    token(&mut g, P0, 1); // not an Elf
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.perm(m).plus, 1);
}

// ------------------------------------------------------------------ Aurelia
#[test]
fn combat_celebrant_exerts_for_an_extra_combat_not_two_turns_running() {
    let mut g = table(&["aurelia-boros-extra-combats", "sauron"]);
    let c = perm(&mut g, P0, "Combat Celebrant");
    let other = token(&mut g, P0, 2);
    g.perm_mut(other).tapped = true;
    combat::attack_triggers(&mut g, P0, &[c, other], P1).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1);
    assert!(!g.perm(other).tapped);
    g.player_mut(P0).turns += 1; // the next turn: still exerted
    combat::attack_triggers(&mut g, P0, &[c], P1).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1);
}

#[test]
fn helm_of_the_host_makes_a_hasty_attacker() {
    let mut g = table(&["aurelia-boros-extra-combats", "sauron"]);
    let helm = perm(&mut g, P0, "Helm of the Host");
    let a = perm(&mut g, P0, "Goblin Rabblemaster");
    g.perm_mut(helm).attached = Some(a);
    let new = sim_core::cardcode::combat_start(&mut g, P0).unwrap();
    assert_eq!(new.len(), 1);
    let t = new[0];
    assert!(g.perm(t).token && !g.perm(t).sick);
    assert_eq!(g.perm(t).name, "Goblin Rabblemaster");
}

#[test]
fn world_at_war_rebounds_for_another_combat() {
    let mut g = table(&["aurelia-boros-extra-combats", "sauron"]);
    let c = card(&g, "World at War");
    assert_eq!(resolve(&mut g, P0, "World at War", Ctx::default()), "exile");
    assert_eq!(g.player(P0).extra_combats, 1);
    assert_eq!(g.player(P0).rebound, [c]);
    g.player_mut(P0).exile.push(c);
    let reb = g.registry.get(c).unwrap().rebound.unwrap();
    reb(&mut g, P0, c).unwrap();
    assert_eq!(g.player(P0).extra_combats, 2);
    assert!(g.player(P0).gy.contains(&c) && !g.player(P0).exile.contains(&c));
}

// ------------------------------------------------------------------ Atraxa
#[test]
fn proliferate_adds_counters_and_loyalty_twice_with_tekuthal() {
    let mut g = table(&["atraxa-superfriends", "sauron"]);
    let m = token(&mut g, P0, 2);
    g.perm_mut(m).plus = 1;
    let w = perm(&mut g, P0, "Karn Liberated");
    g.perm_mut(w).loyalty = Some(6);
    let o = token(&mut g, P1, 5);
    g.perm_mut(o).plus = -1;
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!((g.perm(m).plus, g.perm(w).loyalty, g.perm(o).plus), (2, Some(7), -2));
    perm(&mut g, P0, "Tekuthal, Inquiry Dominus");
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!((g.perm(m).plus, g.perm(w).loyalty, g.perm(o).plus), (4, Some(9), -4));
}

#[test]
fn karn_liberated_ultimate_ends_the_game() {
    let mut g = table(&["atraxa-superfriends", "sauron", "veyran"]);
    let k = perm(&mut g, P0, "Karn Liberated");
    (rules::KARN_ULT.eff)(&mut g, P0, k).unwrap();
    assert!(g.over && g.winner == Some(P0));
}

#[test]
fn walkers_list_their_ultimates() {
    let mut g = table(&["atraxa-superfriends", "sauron"]);
    let w = perm(&mut g, P0, "Jace, the Mind Sculptor");
    g.perm_mut(w).loyalty = Some(13);
    let labels: Vec<String> = options(&mut g, w, P0, Some(false)).into_iter().map(|o| o.label).collect();
    assert!(labels.contains(&"Jace, the Mind Sculptor -12".to_string()), "{labels:?}");
    assert!(labels.contains(&"Jace, the Mind Sculptor +0".to_string()));
}

#[test]
fn teferi_time_raveler_keeps_opponents_at_sorcery_speed() {
    let mut g = table(&["atraxa-superfriends", "sauron"]);
    perm(&mut g, P0, "Teferi, Time Raveler");
    let bolt = card(&g, "Lightning Bolt");
    assert!(sim_core::engine::hooks::allowed(&g, P0, bolt, "hand"));
    assert!(!sim_core::engine::hooks::allowed(&g, P1, bolt, "hand"));
    g.active = Some(P1);
    assert!(sim_core::engine::hooks::allowed(&g, P1, bolt, "hand"));
}

#[test]
fn ichormoon_gauntlet_proliferates_on_noncreature_spells() {
    let mut g = table(&["atraxa-superfriends", "sauron"]);
    perm(&mut g, P0, "Ichormoon Gauntlet");
    let m = token(&mut g, P0, 2);
    g.perm_mut(m).plus = 1;
    let f = g.registry.get(card(&g, "Ichormoon Gauntlet")).unwrap().cast.unwrap();
    let src = named(&g, P0, "Ichormoon Gauntlet")[0];
    let (ring, elves) = (card(&g, "Sol Ring"), card(&g, "Llanowar Elves"));
    f(&mut g, src, P0, ring).unwrap();
    assert_eq!(g.perm(m).plus, 2);
    f(&mut g, src, P0, elves).unwrap(); // a creature spell: no
    assert_eq!(g.perm(m).plus, 2);
}

// ------------------------------------------------------------------ Tergrid
#[test]
fn liliana_of_the_veil_ultimate_keeps_the_better_pile() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let l = perm(&mut g, P0, "Liliana of the Veil");
    let big = perm(&mut g, P1, "Grave Titan");
    let n = g.player(P1).perms.len();
    (rules::LOTV_ULT.eff)(&mut g, P0, l).unwrap();
    assert!(g.perm(big).on_bf, "the pile with the best permanent is kept");
    assert_eq!(g.player(P1).perms.len(), n.div_ceil(2));
}

#[test]
fn liliana_dreadhorde_general_ultimate_keeps_one_of_each() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let l = perm(&mut g, P0, "Liliana, Dreadhorde General");
    for _ in 0..3 {
        token(&mut g, P1, 2);
    }
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Arcane Signet");
    lands(&mut g, P1, "Island", 3, false);
    (rules::LDG_ULT.eff)(&mut g, P0, l).unwrap();
    assert_eq!(g.player(P1).perms.len(), 2); // a creature and an artifact
    assert_eq!(g.player(P1).lands.len(), 1);
}

#[test]
fn hymn_discards_two_at_random_from_the_fullest_hand() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron", "veyran"]);
    hand(&mut g, P1, &["Island", "Swamp", "Counterspell", "Sol Ring"]);
    hand(&mut g, P2, &["Mountain"]);
    resolve(&mut g, P0, "Hymn to Tourach", Ctx::default());
    assert_eq!(g.player(P1).hand.len(), 2);
    assert_eq!(g.player(P1).gy.len(), 2);
    assert_eq!(g.player(P2).hand.len(), 1);
}

#[test]
fn waste_not_rewards_each_kind_of_discard() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    perm(&mut g, P0, "Waste Not");
    let cs = hand(&mut g, P1, &["Swamp", "Grave Titan", "Counterspell"]);
    let hand0 = g.player(P0).hand.len();
    for c in cs {
        let i = g.player(P1).hand.iter().position(|&x| x == c).unwrap();
        zones::discard_index(&mut g, P1, i).unwrap();
        stack::settle_stack(&mut g).unwrap();
    }
    assert_eq!(g.player(P0).floating.any, 2);
    assert_eq!(tokens_of(&g, P0).len(), 1);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
}

#[test]
fn archfiend_of_depravity_leaves_two_creatures() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let a = perm(&mut g, P0, "Archfiend of Depravity");
    for p in 1..=4 {
        token(&mut g, P1, p);
    }
    let f = g.registry.get(card(&g, "Archfiend of Depravity")).unwrap().end_step.unwrap();
    f(&mut g, a, P1).unwrap();
    let left: Vec<i32> = tokens_of(&g, P1).iter().map(|&m| g.perm(m).pow).collect();
    assert_eq!(left, [3, 4]);
}

#[test]
fn phyrexian_obliterator_makes_the_attacker_sacrifice() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let o = perm(&mut g, P0, "Phyrexian Obliterator");
    let a = token(&mut g, P1, 3);
    for _ in 0..4 {
        token(&mut g, P1, 1);
    }
    let mut assign = vec![(a, o)];
    let f = g.registry.get(card(&g, "Phyrexian Obliterator")).unwrap().blocks.unwrap();
    f(&mut g, o, P1, &[a], P0, &mut assign).unwrap();
    assert_eq!(g.player(P1).perms.len(), 2); // five, less three
}

#[test]
fn tinybones_plays_a_cheap_permanent_after_a_discard() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    let t = perm(&mut g, P0, "Tinybones, Trinket Thief");
    let ring = card(&g, "Sol Ring");
    g.player_mut(P1).gy.push(ring);
    let f = g.registry.get(card(&g, "Tinybones, Trinket Thief")).unwrap().end_step.unwrap();
    f(&mut g, t, P0).unwrap();
    assert!(named(&g, P0, "Sol Ring").is_empty(), "no discard this turn");
    g.player_mut(P1).discarded_turn = Some(g.turn_stamp());
    f(&mut g, t, P0).unwrap();
    assert_eq!(named(&g, P0, "Sol Ring").len(), 1);
    assert_eq!(count_in(&g.player(P1).gy, ring), 0);
}

#[test]
fn painful_quandary_takes_junk_or_life() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron"]);
    perm(&mut g, P0, "Painful Quandary");
    let bolt = card(&g, "Lightning Bolt");
    cast::on_cast(&mut g, P1, bolt).unwrap();
    stack::settle_stack(&mut g).unwrap();
    assert_eq!(g.player(P1).life, 35); // nothing to discard
}

// ------------------------------------------------------------------ whole games
/// Tier 3 decks against each other (their card code in play): games finish without a panic. STRESS_GAMES=n plays n
/// games a tier over every pool tier instead.
#[test]
fn tier_three_games_play_out() {
    let n: u64 = std::env::var("STRESS_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let tiers: &[&str] = if n > 0 { &["t1", "t2", "t3", "t4", "t5"] } else { &["t3"] };
    let n = n.max(10);
    for t in tiers {
        let d = tier(t);
        for i in 0..n {
            let k = i as usize % d.len();
            let keys = [d[k], d[(k + 1) % d.len()], d[(k + 2) % d.len()], d[(k + 3) % d.len()]];
            let g = play(&keys, 330_000 + i, false);
            assert!(g.round >= 1, "{t} {keys:?} seed {}", 330_000 + i);
        }
    }
}
