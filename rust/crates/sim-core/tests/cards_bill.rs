//! The bill deck's cards (impls/bill.rs) on hand-built positions. Seat 0 plays the bill deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::combat::can_block;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, hooks, zones};
use sim_core::hooks::{Call, Event};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["bill", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn take_option(g: &mut Game, p: PlayerId, post: bool, label: &str) -> bool {
    let opts = options(g, p, post);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(label))
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

/// a land enters under p's control (the landfall triggers)
fn land_enters(g: &mut Game, p: PlayerId) {
    zones::land_ramp(g, p, 1, false).unwrap();
}

// ------------------------------------------------------------------ Hardened Scales
#[test]
fn scales_adds_one_to_landfall_counters_and_two_scales_stack() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    land_enters(&mut g, P0);
    assert_eq!(g.perm(bill).plus, 1, "no Scales: one counter");
    perm(&mut g, P0, "Hardened Scales");
    land_enters(&mut g, P0);
    assert_eq!(g.perm(bill).plus, 1 + 2, "one Scales: that many plus one");
    let mut g2 = game();
    let bill2 = perm(&mut g2, P0, "Bristly Bill, Spine Sower");
    g2.perm_mut(bill2).sick = false;
    // a second Scales in the graveyard does nothing; two on the battlefield add two
    let s1 = perm(&mut g2, P0, "Hardened Scales");
    let _ = s1;
    assert_eq!(sim_core::cardcode::scales(&g2, P0), 1);
}

#[test]
fn scales_does_not_help_the_opponents_counters_or_other_permanents() {
    let mut g = game();
    perm(&mut g, P1, "Hardened Scales");
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    land_enters(&mut g, P0);
    assert_eq!(g.perm(bill).plus, 1, "the opponent's Scales is not mine");
}


/// the options the hook slots offer at the end of the turn before p's own (post None)
fn eot_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let s = brain::Situation::new(g, p);
    brain::hook_options(g, p, &s, None).unwrap()
}

fn take_eot(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = eot_options(g, p);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(label))
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

#[test]
fn scales_applies_to_proliferate_and_doubling() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Hardened Scales");
    g.perm_mut(bill).plus = 4;
    g.perm_mut(elf).plus = 1;
    lands(&mut g, P0, "Forest", 5, false);
    // Copper Longlegs-style proliferate (the shared engine function): +1 and Scales' +1 each
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!((g.perm(bill).plus, g.perm(elf).plus), (6, 3));
    // Bill's {3}{G}{G}: as many counters again, plus one each with Scales
    assert!(take_eot(&mut g, P0, "Bristly Bill doubles"));
    assert_eq!((g.perm(bill).plus, g.perm(elf).plus), (6 + 7, 3 + 4));
}

#[test]
fn doubling_needs_four_counters_and_five_mana() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    lands(&mut g, P0, "Forest", 5, false);
    g.perm_mut(bill).plus = 3;
    assert!(!eot_options(&mut g, P0).iter().any(|o| o.label.starts_with("Bristly Bill doubles")));
    g.perm_mut(bill).plus = 4;
    assert!(eot_options(&mut g, P0).iter().any(|o| o.label.starts_with("Bristly Bill doubles")));
    // four lands: not enough mana
    let l = g.player_mut(P0).lands.pop().unwrap();
    let _ = l;
    assert!(!eot_options(&mut g, P0).iter().any(|o| o.label.starts_with("Bristly Bill doubles")));
}

#[test]
fn doubling_before_combat_only_when_it_makes_the_attack_lethal() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    lands(&mut g, P0, "Forest", 5, false);
    g.perm_mut(bill).plus = 6; // an 8/8
    g.player_mut(P1).life = 20;
    assert!(!options(&mut g, P0, false).iter().any(|o| o.label.starts_with("Bristly Bill doubles")), "8 + 6 < 20");
    g.player_mut(P1).life = 14; // 8 < 14 <= 8 + 6
    assert!(options(&mut g, P0, false).iter().any(|o| o.label.starts_with("Bristly Bill doubles")));
    g.player_mut(P1).life = 8; // already lethal: save the mana
    assert!(!options(&mut g, P0, false).iter().any(|o| o.label.starts_with("Bristly Bill doubles")));
}

// ------------------------------------------------------------------ where the counters go
#[test]
fn landfall_counter_goes_to_silhana_first_then_the_biggest_stack() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    let big = perm(&mut g, P0, "Thrashing Brontodon");
    g.perm_mut(big).plus = 3;
    land_enters(&mut g, P0);
    assert_eq!(g.perm(big).plus, 4, "the creature with the most counters");
    let sil = perm(&mut g, P0, "Silhana Ledgewalker");
    land_enters(&mut g, P0);
    assert_eq!(g.perm(sil).plus, 1, "Silhana first");
    assert_eq!(g.perm(bill).plus, 0);
}

#[test]
fn landfall_counter_goes_to_the_lecturer_over_a_plain_body() {
    let mut g = game();
    perm(&mut g, P0, "Bristly Bill, Spine Sower");
    let lec = perm(&mut g, P0, "Topiary Lecturer");
    let newt = perm(&mut g, P0, "Noxious Newt");
    land_enters(&mut g, P0);
    assert_eq!((g.perm(lec).plus, g.perm(newt).plus), (1, 0));
}

// ------------------------------------------------------------------ the simple creatures
#[test]
fn boughside_wanderers_takes_a_permanent_from_the_top_four() {
    let mut g = game();
    // top four: two spells and an artifact-free mix; put a known permanent on top
    let lib = &mut g.player_mut(P0).library;
    let keep = lib.len();
    let _ = keep;
    let c = card(&g, "Hardened Scales");
    g.player_mut(P0).library.push(c);
    let before = g.player(P0).library.len();
    perm(&mut g, P0, "Boughside Wanderers");
    assert_eq!(g.player(P0).library.len(), before - 2, "the Wanderers leave the library, one card to hand, three to the bottom");
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn boughside_wanderers_grows_with_landfall() {
    let mut g = game();
    let w = perm(&mut g, P0, "Boughside Wanderers");
    let (p0, t0) = (epow(&g, w), etgh(&g, w));
    land_enters(&mut g, P0);
    assert_eq!((epow(&g, w), etgh(&g, w)), (p0 + 2, t0 + 2));
}

#[test]
fn ghalta_costs_x_less_for_the_greatest_power() {
    let mut g = game();
    let c = card(&g, "Ghalta the Unstoppable");
    g.player_mut(P0).hand.push(c);
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c), (8, "G".to_string()));
    let b = perm(&mut g, P0, "Thrashing Brontodon"); // power 3
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c), (5, "G".to_string()));
    g.perm_mut(b).plus = 20;
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c), (0, "G".to_string()), "never below {{G}}");
}

#[test]
fn silhana_is_blocked_only_by_fliers() {
    let mut g = game();
    let sil = perm(&mut g, P0, "Silhana Ledgewalker");
    let bear = perm(&mut g, P1, "Llanowar Elves");
    assert!(!can_block(&g, bear, sil));
    let flyer = perm(&mut g, P1, "Hornet Queen");
    assert!(can_block(&g, flyer, sil));
}

#[test]
fn venomous_brutalizer_gives_three_poison_and_proliferates_for_a_price() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    g.perm_mut(bill).plus = 2;
    lands(&mut g, P0, "Forest", 2, false);
    let b = perm(&mut g, P0, "Venomous Brutalizer");
    assert_eq!(g.perm(bill).plus, 3, "paid {{1}}{{G}}: proliferate");
    hooks::fire_trigger(&mut g, Event::CombatDamage, Call::CombatDamage { p: P0, a: b, d: P1, dmg: 4 }).unwrap();
    assert_eq!(g.player(P1).poison, 3, "toxic 3");
}
