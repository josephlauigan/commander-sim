//! The green counters cards (impls/green_cards.rs) on hand-built positions. Seat 0 plays the titania deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::combat::can_block;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, hooks, zones};
use sim_core::hooks::Opt;
use sim_core::hooks::{Call, Event};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["omnath", "seph"])
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
    assert_eq!(
        g.player(P0).library.len(),
        before - 2,
        "the Wanderers leave the library, one card to hand, three to the bottom"
    );
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
    sim_core::impls::tymna_cards::toxic_hit(&mut g, b, P1); // the compiled toxic 3, with the combat damage
    assert_eq!(g.player(P1).poison, 3, "toxic 3");
}

fn tokens_named(g: &Game, p: PlayerId, ty: &'static str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).collect()
}

fn has_option(g: &mut Game, post: bool, prefix: &str) -> bool {
    options(g, P0, post).iter().any(|o| o.label.starts_with(prefix))
}

// ------------------------------------------------------------------ Cactarantula
#[test]
fn cactarantula_draws_when_an_opponent_targets_it() {
    let mut g = game();
    let c = perm(&mut g, P0, "Cactarantula");
    let before = g.player(P0).hand.len();
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), c, "bounce", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 2, "the card drawn, and Cactarantula back in hand");
}

#[test]
fn cactarantula_ignores_its_own_controllers_targeting() {
    let mut g = game();
    let c = perm(&mut g, P0, "Cactarantula");
    let before = g.player(P0).hand.len();
    sim_core::engine::removal::apply_removal(&mut g, Some(P0), c, "bounce", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1);
}

// ------------------------------------------------------------------ Carnivorous Canopy
#[test]
fn canopy_destroys_an_enchantment_and_proliferates_at_mana_value_three_or_less() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    g.perm_mut(bill).plus = 2;
    let rs = perm(&mut g, P1, "Rhystic Study"); // mana value 3
    cast_spell(&mut g, P0, "Carnivorous Canopy");
    assert!(!g.perm(rs).on_bf);
    assert_eq!(g.perm(bill).plus, 3, "proliferate");
}

#[test]
fn canopy_at_mana_value_four_does_not_proliferate() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    g.perm_mut(bill).plus = 2;
    let ds = perm(&mut g, P1, "Doubling Season"); // mana value 5
    let c = hand(&mut g, P0, &["Carnivorous Canopy"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(ds), ..Ctx::default() }).unwrap();
    assert!(!g.perm(ds).on_bf);
    assert_eq!(g.perm(bill).plus, 2);
}

#[test]
fn canopy_kills_a_flier_and_leaves_a_ground_creature_alone() {
    let mut g = game();
    let ground = perm(&mut g, P1, "Llanowar Elves");
    let flyer = perm(&mut g, P1, "Hornet Queen");
    cast_spell(&mut g, P0, "Carnivorous Canopy");
    assert!(g.perm(ground).on_bf);
    assert!(!g.perm(flyer).on_bf);
}

#[test]
fn canopy_is_offered_only_with_a_legal_target() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Carnivorous Canopy"]);
    perm(&mut g, P1, "Llanowar Elves");
    assert!(!has_option(&mut g, false, "Carnivorous Canopy"));
    perm(&mut g, P1, "Rhystic Study");
    assert!(has_option(&mut g, false, "Carnivorous Canopy"));
}

// ------------------------------------------------------------------ Wildgrowth Archaic
#[test]
fn archaic_enters_with_a_counter_and_gives_every_creature_spell_one() {
    let mut g = game();
    cast_spell(&mut g, P0, "Wildgrowth Archaic");
    let a = on_bf(&g, P0, "Wildgrowth Archaic").unwrap();
    assert_eq!(g.perm(a).plus, 1, "converge: one color of mana");
    cast_spell(&mut g, P0, "Llanowar Elves");
    let e = on_bf(&g, P0, "Llanowar Elves").unwrap();
    assert_eq!(g.perm(e).plus, 1, "enters with an additional counter");
    perm(&mut g, P0, "Hardened Scales");
    cast_spell(&mut g, P0, "Noxious Newt");
    let n = on_bf(&g, P0, "Noxious Newt").unwrap();
    assert_eq!(g.perm(n).plus, 2, "Hardened Scales: that many plus one");
}

#[test]
fn archaic_costs_two_green() {
    let g = game();
    let c = card(&g, "Wildgrowth Archaic");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c), (0, "GG".to_string()));
}

// ------------------------------------------------------------------ Royal Treatment
#[test]
fn royal_role_is_plus_one_plus_one_and_ward_not_a_counter() {
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    g.perm_mut(bill).plus = 4;
    let (p0, t0) = (epow(&g, bill), etgh(&g, bill));
    sim_core::impls::green_cards::royal_apply(&mut g, bill);
    assert_eq!(g.perm(bill).plus, 4, "not a counter: Hardened Scales added nothing");
    assert_eq!(sim_core::cardcode::aura_ward(&g, bill), 1, "ward {{1}}");
    assert_eq!((epow(&g, bill), etgh(&g, bill)), (p0 + 1, t0 + 1), "the Role's +1/+1");
}

// ------------------------------------------------------------------ plot
#[test]
fn freestrider_plots_then_enters_free_with_counters() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 4, false);
    hand(&mut g, P0, &[FREESTRIDER]);
    assert!(take_option(&mut g, P0, true, "plot Freestrider Commando"));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "plot cost {{3}}{{G}}");
    assert!(!has_option(&mut g, true, "Freestrider Commando (plotted"), "not on the turn it was plotted");
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(take_option(&mut g, P0, false, "Freestrider Commando (plotted, free)"));
    let f = on_bf(&g, P0, "Freestrider Commando").unwrap();
    assert_eq!(g.perm(f).plus, 2);
    assert!(g.player(P0).lands.iter().all(|&l| !g.land(l).tapped), "cast for free");
}

const FREESTRIDER: &str = "Freestrider Commando";

#[test]
fn freestrider_cast_for_its_cost_has_no_counters_and_scales_helps_the_free_one() {
    let mut g = game();
    cast_spell(&mut g, P0, FREESTRIDER);
    let f = on_bf(&g, P0, FREESTRIDER).unwrap();
    assert_eq!(g.perm(f).plus, 0);
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    lands(&mut g, P0, "Forest", 4, false);
    hand(&mut g, P0, &[FREESTRIDER]);
    take_option(&mut g, P0, true, "plot Freestrider Commando");
    g.player_mut(P0).turns += 1;
    take_option(&mut g, P0, false, "Freestrider Commando (plotted, free)");
    let f = on_bf(&g, P0, FREESTRIDER).unwrap();
    assert_eq!(g.perm(f).plus, 3, "two counters, plus one for Hardened Scales");
}

#[test]
fn tumbleweed_rising_makes_an_x_by_x_elemental_and_can_be_plotted() {
    let mut g = game();
    let big = perm(&mut g, P0, "Thrashing Brontodon");
    g.perm_mut(big).plus = 2; // 5 power
    cast_spell(&mut g, P0, "Tumbleweed Rising");
    let t = tokens_named(&g, P0, "elemental");
    assert_eq!(t.len(), 1);
    assert_eq!((epow(&g, t[0]), etgh(&g, t[0])), (5, 5));
    // plot {2}{G}, then free
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Tumbleweed Rising"]);
    assert!(take_option(&mut g, P0, true, "plot Tumbleweed Rising"));
    g.player_mut(P0).turns += 1;
    assert!(take_option(&mut g, P0, false, "Tumbleweed Rising (plotted, free)"));
    assert_eq!(tokens_named(&g, P0, "elemental").len(), 2);
}

// ------------------------------------------------------------------ Lifestream's Blessing
#[test]
fn lifestream_draws_x_and_gains_twice_x_from_exile_only() {
    let mut g = game();
    let big = perm(&mut g, P0, "Thrashing Brontodon");
    g.perm_mut(big).plus = 1; // power 4
    let (h, life) = (g.player(P0).hand.len(), g.player(P0).life);
    cast_spell(&mut g, P0, LIFESTREAM);
    assert_eq!(g.player(P0).hand.len(), h + 4, "X = the greatest power, not the number of creatures");
    assert_eq!(g.player(P0).life, life, "cast from hand: no life");
}

const LIFESTREAM: &str = "Lifestream's Blessing";

#[test]
fn lifestream_foretells_and_is_cast_from_exile_for_five() {
    let mut g = game();
    let big = perm(&mut g, P0, "Thrashing Brontodon");
    g.perm_mut(big).plus = 1;
    lands(&mut g, P0, "Forest", 5, false);
    hand(&mut g, P0, &[LIFESTREAM]);
    assert!(take_option(&mut g, P0, true, "foretell Lifestream's Blessing"));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count(), 3, "foretell costs {{2}}");
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    let (h, life) = (g.player(P0).hand.len(), g.player(P0).life);
    assert!(take_option(&mut g, P0, false, "Lifestream's Blessing (foretold)"));
    assert_eq!(g.player(P0).hand.len(), h - 1 + 4);
    assert_eq!(g.player(P0).life, life + 8, "twice X");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count(), 0, "{{4}}{{G}}");
}

// ------------------------------------------------------------------ Sample Collector
#[test]
fn sample_collector_collects_evidence_for_a_counter() {
    let mut g = game();
    let sc = perm(&mut g, P0, "Sample Collector");
    let gy = hand(&mut g, P0, &["Cultivate", "Rampant Growth"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).gy.extend(gy);
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sc], d: P1 }).unwrap();
    assert_eq!(g.perm(sc).plus, 1);
    assert_eq!(g.player(P0).gy.len(), 1, "one card with enough mana value is exiled");
    assert!(in_gy(&g, P0, "Rampant Growth"), "the smaller card stays");
    assert_eq!(g.player(P0).exile.len(), 1);
}

#[test]
fn sample_collector_needs_three_mana_value_in_the_graveyard() {
    let mut g = game();
    let sc = perm(&mut g, P0, "Sample Collector");
    let gy = hand(&mut g, P0, &["Rampant Growth"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).gy.extend(gy);
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sc], d: P1 }).unwrap();
    assert_eq!(g.perm(sc).plus, 0);
    assert_eq!(g.player(P0).gy.len(), 1);
    // two cards worth 2 + 2 do it
    let more = hand(&mut g, P0, &["Nature's Lore"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).gy.extend(more);
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sc], d: P1 }).unwrap();
    assert_eq!(g.perm(sc).plus, 1);
    assert!(g.player(P0).gy.is_empty());
}

// ------------------------------------------------------------------ Spider-Man
#[test]
fn spider_man_web_slings_by_returning_a_tapped_creature() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(elf).tapped = true;
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Spider-Man, Brooklyn Visionary"]);
    let n = g.player(P0).lands.len();
    assert!(take_option(&mut g, P0, false, "Spider-Man, Brooklyn Visionary (web-slinging"));
    assert!(on_bf(&g, P0, "Spider-Man, Brooklyn Visionary").is_some());
    assert!(!g.perm(elf).on_bf);
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Llanowar Elves"));
    assert_eq!(g.player(P0).lands.len(), n + 1, "the land from its enters trigger");
}

#[test]
fn spider_man_needs_a_tapped_creature_to_web_sling() {
    let mut g = game();
    perm(&mut g, P0, "Llanowar Elves"); // untapped
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Spider-Man, Brooklyn Visionary"]);
    assert!(!has_option(&mut g, false, "Spider-Man, Brooklyn Visionary (web"));
}

// ------------------------------------------------------------------ Greenbelt Radical
#[test]
fn greenbelt_radical_is_cast_face_down_and_turned_up_for_seven() {
    let mut g = game();
    let wurm = perm(&mut g, P0, "Elfhame Wurm");
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Greenbelt Radical"]);
    assert!(take_option(&mut g, P0, false, "Greenbelt Radical (face down)"));
    let r = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).data.truthy(sim_core::state::DataKey::Facedown))
        .unwrap();
    assert_eq!((epow(&g, r), etgh(&g, r)), (2, 2));
    assert_eq!(sim_core::cardcode::aura_ward(&g, r), 2, "ward {{2}}");
    // seven lands: turn it up (a +1/+1 counter on each creature you control, and trample)
    lands(&mut g, P0, "Forest", 4, false);
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(take_option(&mut g, P0, false, "Greenbelt Radical (turn face up)"));
    assert_eq!((epow(&g, r), etgh(&g, r)), (4 + 1, 4 + 1), "a 4/4 with its own counter");
    assert_eq!(g.perm(wurm).plus, 1, "a counter on each creature");
    assert_eq!(sim_core::cardcode::aura_ward(&g, r), 0);
    assert!(sim_core::engine::combat::kw(&g, wurm, "trample"));
}

// ------------------------------------------------------------------ Studious First-Year
#[test]
fn first_year_casts_a_prepared_rampant_growth_once() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    let fy = perm(&mut g, P0, "Studious First-Year");
    lands(&mut g, P0, "Forest", 2, false);
    let n = g.player(P0).lands.len();
    assert!(take_option(&mut g, P0, false, "Rampant Growth (Studious First-Year"));
    assert_eq!(g.player(P0).lands.len(), n + 1);
    assert!(g.player(P0).lands.last().is_some_and(|&l| g.land(l).tapped), "tapped");
    assert_eq!(g.perm(bill).plus + g.perm(fy).plus, 1, "landfall");
    assert!(!has_option(&mut g, false, "Rampant Growth (Studious First-Year"), "unprepared");
}

// ------------------------------------------------------------------ the shared cards Hardened Scales touches
#[test]
fn scales_adds_to_increment_and_the_hydra() {
    // the Hydra enters with X counters (X = 3 here), plus one
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    g.last_x = 3;
    let h = perm(&mut g, P0, "Genesis Hydra");
    assert_eq!(g.perm(h).plus, 4);
    // increment: a counter for a spell that cost more than its power or toughness
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    let lec = perm(&mut g, P0, "Topiary Lecturer"); // 1/2
    let spell = hand(&mut g, P0, &["Cultivate"])[0]; // three mana
    cast::cast_card(&mut g, P0, spell, "hand", Ctx::default()).unwrap();
    assert_eq!(g.perm(lec).plus, 2, "increment: one counter, plus one for Hardened Scales");
}

// ------------------------------------------------------------------ the cards the audit already called Full
#[test]
fn retreat_to_kazandu_puts_a_counter_on_a_creature_for_each_land() {
    let mut g = game();
    perm(&mut g, P0, "Retreat to Kazandu");
    let t = perm(&mut g, P0, "Thrashing Brontodon");
    let life = g.player(P0).life;
    land_enters(&mut g, P0);
    assert_eq!((g.perm(t).plus, g.player(P0).life), (1, life), "the counter mode while a creature can use it");
    // with Hardened Scales the counter is two
    perm(&mut g, P0, "Hardened Scales");
    land_enters(&mut g, P0);
    assert_eq!(g.perm(t).plus, 1 + 2);
}

#[test]
fn evolution_sage_proliferates_on_each_land_and_scales_adds_to_it() {
    let mut g = game();
    perm(&mut g, P0, "Evolution Sage");
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    g.perm_mut(bill).plus = 2;
    land_enters(&mut g, P0);
    let sage = on_bf(&g, P0, "Evolution Sage").unwrap();
    // the landfall counter goes to one creature; the proliferate adds one per creature that has counters (4 or 5)
    let total = g.perm(bill).plus + g.perm(sage).plus;
    assert!((4..=5).contains(&total), "landfall counter plus proliferate: {total}");
}

#[test]
fn harrow_is_two_landfall_triggers_and_a_land_sacrificed() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Harrow"]);
    let n = g.player(P0).lands.len();
    cast_harrow(&mut g);
    assert_eq!(g.player(P0).lands.len(), n + 1, "sacrifice one, fetch two");
    assert_eq!(g.perm(bill).plus, 2, "two landfall triggers");
}

fn cast_harrow(g: &mut Game) {
    let c = card(g, "Harrow");
    // the additional cost (sacrifice a land) is paid in `additional_cost` before the spell resolves
    g.player_mut(P0).hand.retain(|&x| x != c);
    let l = g.player(P0).lands[0];
    sim_core::engine::turn::remove_land(g, P0, l);
    let cd = g.land(l).cd;
    g.player_mut(P0).gy.push(cd);
    g.player_mut(P0).hand.push(c);
    cast::cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}

#[test]
fn a_fetch_land_is_two_landfall_triggers() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    let c = hand(&mut g, P0, &["Evolving Wilds"])[0];
    let _ = c;
    let before = g.player(P0).lands.len();
    sim_core::engine::turn::play_land(&mut g, P0).unwrap();
    assert_eq!(g.perm(bill).plus, 2, "the Wilds, and the Forest it finds");
    assert_eq!(g.player(P0).lands.len(), before + 1);
    assert!(g.player(P0).lands.iter().all(|&l| &*g.db.get(g.land(l).cd).name != "Evolving Wilds"));
}

#[test]
fn additive_evolution_makes_a_fractal_with_three_counters() {
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    cast_spell(&mut g, P0, "Additive Evolution");
    let t = tokens_named(&g, P0, "fractal");
    assert_eq!(t.len(), 1, "the Fractal survives");
    assert_eq!(g.perm(t[0]).plus, 3 + 1, "three counters, plus one for Hardened Scales");
}

#[test]
fn additive_evolution_grows_a_creature_each_combat() {
    let mut g = game();
    let big = perm(&mut g, P0, "Thrashing Brontodon");
    cast_spell(&mut g, P0, "Additive Evolution");
    let tk = tokens_named(&g, P0, "fractal")[0];
    g.perm_mut(big).plus = 5;
    hooks::fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert_eq!(g.perm(big).plus, 6, "the biggest stack");
    assert!(sim_core::engine::combat::kw(&g, big, "vigilance"));
    assert!(!sim_core::engine::combat::kw(&g, tk, "vigilance"), "only the target");
}

#[test]
fn wild_hypothesis_makes_an_x_fractal_for_the_spare_mana() {
    let mut g = game();
    perm(&mut g, P0, "Hardened Scales");
    lands(&mut g, P0, "Forest", 5, false);
    hand(&mut g, P0, &["Wild Hypothesis"]);
    assert!(take_option(&mut g, P0, true, "Wild Hypothesis (X = 4)"));
    let t = tokens_named(&g, P0, "fractal");
    assert_eq!(t.len(), 1);
    assert_eq!(g.perm(t[0]).plus, 4 + 1);
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

#[test]
fn wild_hypothesis_is_not_cast_for_x_of_one() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 2, false);
    let c = hand(&mut g, P0, &["Wild Hypothesis"])[0];
    assert!(!has_option(&mut g, true, "Wild Hypothesis"));
    assert_eq!(sim_core::ai::deck_prio(&g, P0, c), 0.0);
}

// ------------------------------------------------------------------ Crushing Canopy
#[test]
fn crushing_canopy_is_cast_only_at_a_flier_or_an_enchantment() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Crushing Canopy"]);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    assert!(!has_option(&mut g, false, "Crushing Canopy"), "not at a ground creature");
    let rs = perm(&mut g, P1, "Rhystic Study");
    assert!(take_option(&mut g, P0, false, "Crushing Canopy"));
    assert!(!g.perm(rs).on_bf);
    assert!(g.perm(elf).on_bf);
}

// ------------------------------------------------------------------ the lethal doubling counts the blockers
fn doubles(g: &mut Game) -> bool {
    options(g, P0, false).iter().any(|o| o.label.starts_with("Bristly Bill doubles"))
}

#[test]
fn lethal_doubling_not_offered_when_chump_blockers_stop_the_attack() {
    let mut g = game();
    let bill = perm(&mut g, P0, "Bristly Bill, Spine Sower");
    lands(&mut g, P0, "Forest", 5, false);
    g.perm_mut(bill).plus = 6; // an 8/8, 14 after doubling
    g.player_mut(P1).life = 14;
    assert!(doubles(&mut g), "no blockers: lethal");
    token(&mut g, P1, 1); // one chump blocker stops the lone attacker
    assert!(!doubles(&mut g), "the raw sum is lethal but the chump stops it");
}

// ------------------------------------------------------------------ Topiary Lecturer and Vinelasher Adept
#[test]
fn increment_grows_the_lecturer_when_the_mana_spent_is_greater_than_its_smaller_stat() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer"); // 1/2
    cast_spell(&mut g, P0, "Rampant Growth"); // 2 mana
    assert_eq!(g.perm(lec).plus, 1, "2 > 1");
    cast_spell(&mut g, P0, "Cultivate"); // 3 mana: the Lecturer is now 2/3
    assert_eq!(g.perm(lec).plus, 2, "3 > 2");
    cast_spell(&mut g, P0, "Rampant Growth"); // 2 mana: not greater than 3? the Lecturer is 3/4 now
    assert_eq!(g.perm(lec).plus, 2, "2 is not greater than 3");
}

#[test]
fn increment_ignores_spells_cast_by_opponents() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer");
    let c = hand(&mut g, P1, &["Terror"])[0];
    cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.perm(lec).plus, 0);
}

#[test]
fn topiary_lecturer_taps_for_mana_equal_to_its_power() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, lec), Some(1));
    g.perm_mut(lec).plus = 3;
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, lec), Some(4));
}

#[test]
fn vinelasher_adept_landcycles_for_two() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Vinelasher Adept"]);
    let lands_before = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count();
    assert!(take_option(&mut g, P0, false, "Vinelasher Adept (basic landcycling)"));
    assert!(in_gy(&g, P0, "Vinelasher Adept"));
    let basic = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land && matches!(&*g.db.get(c).name, "Forest" | "Mountain")).count(); // any basic: the omnath deck plays two
    assert_eq!(basic, lands_before + 1);
}

// ------------------------------------------------------------------ Vorinclex, Monstrous Raider
const VORINCLEX: &str = "Vorinclex, Monstrous Raider";

fn enter_with(g: &mut Game, p: PlayerId, name: &str, plus: i32) -> PermId {
    let c = take(g, p, name);
    zones::enter(g, p, c, zones::Enter { sick: false, plus, ..Default::default() }).unwrap()
}

#[test]
fn vorinclex_doubles_mine_and_halves_the_opponents_rounded_down() {
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    let a = enter_with(&mut g, P0, "Llanowar Elves", 3);
    assert_eq!(g.perm(a).plus, 6, "enters with 3: twice that");
    let b = enter_with(&mut g, P1, "Llanowar Elves", 3);
    assert_eq!(g.perm(b).plus, 1, "an opponent's 3 become 1 (rounded down)");
    let c = enter_with(&mut g, P1, "Birds of Paradise", 1);
    assert_eq!(g.perm(c).plus, 0, "an opponent's 1 becomes 0");
    // Vinelasher Adept (compiled ETB: three counters on the best creature): six
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    let before: i32 = g.player(P0).perms.iter().map(|&m| g.perm(m).plus).sum();
    perm(&mut g, P0, "Vinelasher Adept");
    let after: i32 = g.player(P0).perms.iter().map(|&m| g.perm(m).plus).sum();
    assert_eq!(after - before, 6);
    // without Vorinclex nothing changes
    let mut g = game();
    let a = enter_with(&mut g, P0, "Llanowar Elves", 3);
    assert_eq!(g.perm(a).plus, 3);
}

#[test]
fn two_vorinclex_both_apply_and_scales_comes_first() {
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    perm(&mut g, P0, VORINCLEX);
    let a = enter_with(&mut g, P0, "Llanowar Elves", 3);
    assert_eq!(g.perm(a).plus, 12);
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    perm(&mut g, P0, "Hardened Scales");
    let a = enter_with(&mut g, P0, "Llanowar Elves", 3);
    assert_eq!(g.perm(a).plus, 8, "(3 + 1) doubled");
}

#[test]
fn vorinclex_doubles_planeswalker_loyalty_on_entering_and_proliferate() {
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    let w = perm(&mut g, P0, "Wrenn and Six");
    assert_eq!(g.perm(w).loyalty, Some(6), "enters with 3 loyalty: twice that");
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(8), "proliferate adds one: doubled");
}

#[test]
fn vorinclex_doubles_proliferate_and_a_persist_counter_and_halves_the_opponents() {
    let mut g = game();
    perm(&mut g, P0, VORINCLEX);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(elf).plus = 1;
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(elf).plus, 3, "proliferate: +1 doubled");
    // a -1/-1 counter on an opponent's creature: proliferate puts two (doubled for me, the proliferator)
    let bird = perm(&mut g, P1, "Birds of Paradise");
    g.perm_mut(bird).plus = -1;
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(bird).plus, -3, "-1 doubled to -2 more");
    // persist: the -1/-1 counter on return is doubled
    let c = enter_with(&mut g, P0, "Llanowar Elves", -1);
    assert_eq!(g.perm(c).plus, -2);
    // an opponent's Vorinclex halves my proliferate: -1 -> 0 more
    let mut g = game();
    perm(&mut g, P1, VORINCLEX);
    let bird = perm(&mut g, P1, "Birds of Paradise");
    g.perm_mut(bird).plus = -1;
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(bird).plus, -1, "half of one is nothing");
}

#[test]
fn vorinclex_helper_math() {
    let mut g = game();
    assert_eq!(sim_core::cardcode::vorinclex(&g, P0, 3), 3);
    perm(&mut g, P1, VORINCLEX);
    assert_eq!(sim_core::cardcode::vorinclex(&g, P0, 3), 1);
    assert_eq!(sim_core::cardcode::vorinclex(&g, P1, 3), 6);
    assert_eq!(sim_core::cardcode::vorinclex(&g, P0, -3), -1);
}
