//! The Tier 4 heliod-mono-white-stax and Tier 5 zur-esper-enchantment-control cards (impls/t45_heliod_zur.rs and the
//! compiled patterns under `// --- Tiers 4/5 (heliod-zur)`) on hand-built positions. Each expectation follows the
//! card's Oracle text.

use sim_core::cardcode;
use sim_core::dsl::has_kw;
use sim_core::engine::combat::can_block;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh, melira, untargetable};
use sim_core::engine::{cast, life, tutors};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const HELIOD: &str = "heliod-mono-white-stax";
const ZUR: &str = "zur-esper-enchantment-control";

fn heliod() -> Game {
    table(&[HELIOD, "seph", "sauron"])
}

fn zur() -> Game {
    table(&[ZUR, "seph", "sauron"])
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn land_opts(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(l).card()).unwrap().land_options.unwrap();
    f(g, l, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

/// the option whose label ends so
fn pick(o: &[Opt], end: &str) -> Opt {
    o.iter()
        .find(|x| x.label.ends_with(end))
        .unwrap_or_else(|| panic!("{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>()))
        .clone()
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn end_step(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

// ------------------------------------------------------------------ life gain: replacements and triggers
#[test]
fn cleric_class_adds_one_then_the_wind_crystal_doubles() {
    let mut g = heliod();
    perm(&mut g, P0, "Cleric Class");
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, 42, "1 plus 1");
    perm(&mut g, P0, "The Wind Crystal");
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, 46, "(1 + 1) x 2");
    life::gain(&mut g, P1, 1).unwrap();
    assert_eq!(g.player(P1).life, 41, "only its controller's life gain");
}

#[test]
fn whenever_you_gain_life_triggers_fire_and_voice_of_the_blessed_flies_at_four() {
    let mut g = heliod();
    let pm = perm(&mut g, P0, "Ajani's Pridemate");
    let v = perm(&mut g, P0, "Voice of the Blessed");
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!((g.perm(pm).plus, g.perm(v).plus), (1, 1), "one counter each per life-gain event");
    life::gain(&mut g, P1, 3).unwrap();
    assert_eq!(g.perm(pm).plus, 1, "not on an opponent's gain");
    assert!(!has_kw(&g, v, "flying"));
    g.perm_mut(v).plus = 4;
    assert!(has_kw(&g, v, "flying") && has_kw(&g, v, "vigilance") && !has_kw(&g, v, "indestructible"));
    g.perm_mut(v).plus = 10;
    assert!(has_kw(&g, v, "indestructible"));
}

#[test]
fn cleric_class_levels_two_and_three() {
    let mut g = heliod();
    let cc = perm(&mut g, P0, "Cleric Class");
    let a = perm(&mut g, P0, "Mother of Runes");
    perm(&mut g, P0, "Giver of Runes");
    lands(&mut g, P0, "Plains", 9, false);
    g.active = Some(P0);
    g.step = Step::Main1;
    let o = opts(&mut g, cc, P0, Some(false));
    assert_eq!(o.len(), 1, "level 2 on offer");
    assert!(run(&mut g, P0, &o[0]));
    let before: i32 = g.player(P0).perms.iter().map(|&m| g.perm(m).plus).sum();
    life::gain(&mut g, P0, 1).unwrap();
    let after: i32 = g.player(P0).perms.iter().map(|&m| g.perm(m).plus).sum();
    assert_eq!(after, before + 1, "level 2: a counter when you gain life");
    let _ = a;
    // level 3: the best creature card returns and you gain its toughness (plus 1)
    let t = to_gy(&mut g, P0, "Sun Titan");
    let o = opts(&mut g, cc, P0, Some(false));
    assert_eq!(o.len(), 1, "level 3 on offer");
    let life0 = g.player(P0).life;
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.player(P0).gy.contains(&t) && !named(&g, P0, "Sun Titan").is_empty(), "Sun Titan returned");
    assert!(g.player(P0).life >= life0 + 7, "6 toughness plus 1");
}

#[test]
fn well_of_lost_dreams_pays_x_to_draw_x() {
    let mut g = heliod();
    perm(&mut g, P0, "Well of Lost Dreams");
    lands(&mut g, P0, "Plains", 5, false);
    g.active = Some(P1); // an opponent's turn: nothing kept back
    let hand0 = g.player(P0).hand.len();
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0 + 3, "X = 3, the life gained");
    let untapped = g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count();
    assert_eq!(untapped, 2);
}

#[test]
fn cosmos_elixir_gains_two_or_draws_above_forty() {
    let mut g = heliod();
    perm(&mut g, P0, "Cosmos Elixir");
    end_step(&mut g, P0);
    assert_eq!(g.player(P0).life, 42, "at 40: gain 2");
    let hand0 = g.player(P0).hand.len();
    end_step(&mut g, P0);
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (42, hand0 + 1), "above 40: draw");
    end_step(&mut g, P1);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "only your end step");
}

#[test]
fn felidar_sovereign_wins_at_forty_life() {
    let mut g = heliod();
    perm(&mut g, P0, "Felidar Sovereign");
    g.player_mut(P0).life = 39;
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(!g.over, "39 life");
    g.player_mut(P0).life = 40;
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(g.over && g.winner == Some(P0));
}

#[test]
fn serra_ascendant_and_daxos_scale() {
    let mut g = heliod();
    let s = perm(&mut g, P0, "Serra Ascendant");
    g.player_mut(P0).life = 29;
    assert_eq!(epow(&g, s), 1);
    assert!(!has_kw(&g, s, "flying"));
    g.player_mut(P0).life = 30;
    assert_eq!((epow(&g, s), etgh(&g, s)), (6, 6));
    assert!(has_kw(&g, s, "flying"));
    let d = perm(&mut g, P0, "Daxos, Blessed by the Sun");
    assert_eq!(etgh(&g, d), 3, "devotion to white: WW + W");
    let life0 = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(etgh(&g, d), 4);
    assert_eq!(g.player(P0).life, life0 + 1, "another creature entered: gain 1");
}

#[test]
fn altar_of_the_pantheon_adds_devotion_and_life() {
    let mut g = heliod();
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'W'), 0);
    let a = perm(&mut g, P0, "Altar of the Pantheon");
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'W'), 1);
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'U'), 1);
    let tap = g.registry.get(g.perm(a).cd.unwrap()).unwrap().on_tap_perm.unwrap();
    tap(&mut g, P0, a, 1).unwrap();
    assert_eq!(g.player(P0).life, 40, "no God or legendary enchantment");
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    tap(&mut g, P0, a, 1).unwrap();
    assert_eq!(g.player(P0).life, 41, "Heliod: gain 1");
}

#[test]
fn haliya_gains_on_entering_and_draws_after_three() {
    let mut g = heliod();
    perm(&mut g, P0, "Haliya, Guided by Light");
    assert_eq!(g.player(P0).life, 41, "Haliya itself");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(g.player(P0).life, 42, "an artifact");
    let hand0 = g.player(P0).hand.len();
    end_step(&mut g, P0);
    assert_eq!(g.player(P0).hand.len(), hand0, "only 2 gained");
    life::gain(&mut g, P0, 1).unwrap();
    end_step(&mut g, P0);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "3 gained this turn");
}

#[test]
fn linden_gains_for_each_white_attacker() {
    let mut g = heliod();
    let l = perm(&mut g, P0, "Linden, the Steadfast Queen");
    let m = perm(&mut g, P0, "Mother of Runes");
    let w = perm(&mut g, P0, "Walking Ballista"); // colourless
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![l, m, w], d: P1 }).unwrap();
    assert_eq!(g.player(P0).life, 42);
}

#[test]
fn ajani_gains_makes_a_pridemate_and_exiles_at_fifty_five() {
    let mut g = heliod();
    let a = perm(&mut g, P0, "Ajani, Strength of the Pride");
    perm(&mut g, P0, "Mother of Runes");
    let o = opts(&mut g, a, P0, Some(false));
    let plus = pick(&o, "+1");
    assert!(o.iter().all(|x| !x.label.ends_with("+0")), "0 needs 55 life");
    assert!(run(&mut g, P0, &plus));
    assert_eq!(g.player(P0).life, 42, "a creature and a planeswalker");
    // -2 on a later turn
    g.round += 1;
    let o = opts(&mut g, a, P0, Some(false));
    let minus = pick(&o, "-2");
    assert!(run(&mut g, P0, &minus));
    let pm = named(&g, P0, "Ajani's Pridemate");
    assert_eq!(pm.len(), 1);
    assert!(g.perm(pm[0]).token && epow(&g, pm[0]) == 2);
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(pm[0]).plus, 1, "the token grows on life gain");
    // 0 at 55 life: Ajani and the opponents' artifacts and creatures go
    g.round += 1;
    g.player_mut(P0).life = 60;
    let t = perm(&mut g, P1, "Sun Titan");
    let r = perm(&mut g, P1, "Sol Ring");
    let o = opts(&mut g, a, P0, Some(false));
    let zero = pick(&o, "+0");
    assert!(run(&mut g, P0, &zero));
    assert!(!g.perm(a).on_bf && !g.perm(t).on_bf && !g.perm(r).on_bf);
    assert!(!named(&g, P0, "Mother of Runes").is_empty(), "yours stay");
}

#[test]
fn heliods_intervention_destroys_x_or_gains_twice_x() {
    let mut g = heliod();
    let c = take(&mut g, P0, "Heliod's Intervention");
    g.player_mut(P0).hand.push(c);
    lands(&mut g, P0, "Plains", 6, false);
    let a = perm(&mut g, P1, "Rhystic Study");
    let b = perm(&mut g, P1, "Smothering Tithe");
    let o = hand_opts(&mut g, c, P0, Some(false));
    let d = o.iter().find(|x| x.label.contains("destroy 2")).expect("destroy two").clone();
    assert!(run(&mut g, P0, &d));
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
    // the life mode, toward Felidar Sovereign
    let mut g = heliod();
    let c = take(&mut g, P0, "Heliod's Intervention");
    g.player_mut(P0).hand.push(c);
    lands(&mut g, P0, "Plains", 6, false);
    perm(&mut g, P0, "Felidar Sovereign");
    g.player_mut(P0).life = 35;
    g.active = Some(P1);
    let o = hand_opts(&mut g, c, P0, None);
    let l = o.iter().find(|x| x.label.contains("gain")).expect("the life mode").clone();
    assert!(run(&mut g, P0, &l));
    assert!(g.player(P0).life >= 40, "{}", g.player(P0).life);
}

#[test]
fn archivist_of_oghma_answers_an_opponents_search() {
    let mut g = heliod();
    perm(&mut g, P1, "Archivist of Oghma");
    let hand0 = g.player(P1).hand.len();
    tutors::shuffle_library(&mut g, P0);
    life::check_state(&mut g).unwrap();
    assert_eq!((g.player(P1).life, g.player(P1).hand.len()), (41, hand0 + 1));
    tutors::shuffle_library(&mut g, P1);
    life::check_state(&mut g).unwrap();
    assert_eq!(g.player(P1).hand.len(), hand0 + 1, "not its controller's own search");
}

#[test]
fn authority_taps_opposing_creatures_and_suture_priest_drains_them() {
    let mut g = heliod();
    perm(&mut g, P0, "Authority of the Consuls");
    perm(&mut g, P0, "Suture Priest");
    let m = perm(&mut g, P1, "Sun Titan");
    assert!(g.perm(m).tapped, "enters tapped");
    assert_eq!(g.player(P0).life, 41, "Authority: gain 1");
    assert_eq!(g.player(P1).life, 39, "Suture Priest: that player loses 1");
    let mine = perm(&mut g, P0, "Mother of Runes");
    assert!(!g.perm(mine).tapped, "only opponents' creatures");
}

#[test]
fn oketras_monument_discounts_white_creatures_only() {
    let mut g = heliod();
    perm(&mut g, P0, "Oketra's Monument");
    assert_eq!(sim_core::dsl::cost_delta(&g, P0, card(&g, "Mother of Runes")), -1);
    assert_eq!(sim_core::dsl::cost_delta(&g, P0, card(&g, "Walking Ballista")), 0, "colourless");
    assert_eq!(sim_core::dsl::cost_delta(&g, P0, card(&g, "Swords to Plowshares")), 0, "not a creature");
    assert_eq!(sim_core::dsl::cost_delta(&g, P1, card(&g, "Mother of Runes")), 0, "only yours");
}

#[test]
fn buried_ruin_and_hall_return_cards() {
    let mut g = heliod();
    let r = lands(&mut g, P0, "Buried Ruin", 1, false)[0];
    lands(&mut g, P0, "Plains", 3, false);
    let ring = to_gy(&mut g, P0, "The One Ring");
    let o = land_opts(&mut g, r, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.player(P0).hand.contains(&ring) && !g.player(P0).lands.contains(&r), "to hand, the Ruin sacrificed");
    let mut g = zur();
    let h = lands(&mut g, P0, "Hall of Heliod's Generosity", 1, false)[0];
    lands(&mut g, P0, "Plains", 2, false);
    let rs = to_gy(&mut g, P0, "Rhystic Study");
    let o = land_opts(&mut g, h, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).library.last(), Some(&rs), "on top of the library");
}

#[test]
fn heliod_and_triskelion_is_a_combo() {
    let mut g = heliod();
    let cmb = sim_core::impls::combos::all().iter().find(|c| c.name == "Heliod, Sun-Crowned + Triskelion").unwrap();
    assert!(!(cmb.ready)(&g, P0).0);
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    let t = perm(&mut g, P0, "Triskelion");
    assert!(g.perm(t).plus > 0, "enters with counters");
    assert!((cmb.ready)(&g, P0).0);
}

// ------------------------------------------------------------------ Zur
#[test]
fn solemnity_stops_counters_and_poison() {
    let mut g = zur();
    let pm = perm(&mut g, P0, "Walking Ballista");
    perm(&mut g, P0, "Solemnity");
    cardcode::put_plus(&mut g, pm, 2);
    assert_eq!(g.perm(pm).plus, 0);
    assert!(melira(&g, P0) && melira(&g, P1), "no player gets poison counters");
}

#[test]
fn steel_of_the_godhead_and_diplomatic_immunity() {
    let mut g = zur();
    let z = perm(&mut g, P0, "Zur the Enchanter"); // white, blue and black: 1/4
    perm(&mut g, P0, "Steel of the Godhead");
    assert_eq!((epow(&g, z), etgh(&g, z)), (3, 6), "+1/+1 for white, +1/+1 for blue");
    assert!(has_kw(&g, z, "lifelink") && has_kw(&g, z, "unblockable"));
    assert!(!untargetable(&g, z));
    perm(&mut g, P0, "Diplomatic Immunity");
    assert!(untargetable(&g, z), "shroud");
}

#[test]
fn estrids_invocation_copies_your_best_enchantment() {
    let mut g = zur();
    perm(&mut g, P0, "Rhystic Study");
    perm(&mut g, P0, "Estrid's Invocation");
    assert_eq!(named(&g, P0, "Rhystic Study").len(), 2);
}

#[test]
fn delney_evasion_and_extra_triggers() {
    let mut g = zur();
    perm(&mut g, P0, "Delney, Streetwise Lookout");
    let w = perm(&mut g, P0, "Esper Sentinel"); // 1/1
    let big = perm(&mut g, P1, "Sun Titan");
    let small = perm(&mut g, P1, "Esper Sentinel");
    assert!(!can_block(&g, big, w), "power 3 or greater can't block it");
    assert!(can_block(&g, small, w));
    // Soul Warden (power 1): its trigger triggers an additional time
    let mut g = heliod();
    perm(&mut g, P0, "Delney, Streetwise Lookout");
    perm(&mut g, P0, "Soul Warden");
    let life0 = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life0 + 2);
}

#[test]
fn eidolon_of_rhetoric_allows_one_spell_a_turn() {
    let mut g = zur();
    perm(&mut g, P0, "Eidolon of Rhetoric");
    lands(&mut g, P1, "Island", 3, false);
    let c = take(&mut g, P1, "Counterspell");
    g.player_mut(P1).hand.push(c);
    assert!(cast::castable(&g, P1, c, "hand"));
    let st = g.turn_stamp();
    g.player_mut(P1).turn_casts = Some((st, vec![card(&g, "Sol Ring")]));
    assert!(!cast::castable(&g, P1, c, "hand"), "a second spell this turn");
}

#[test]
fn imprisoned_in_the_moon_makes_a_creature_a_wastes() {
    let mut g = zur();
    let t = perm(&mut g, P1, "Sun Titan");
    let lands0 = g.player(P1).lands.len();
    perm(&mut g, P0, "Imprisoned in the Moon");
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).lands.len(), lands0 + 1);
    let l = *g.player(P1).lands.last().unwrap();
    assert_eq!(&*g.db.get(g.perm(l).card()).name, "Wastes");
}
