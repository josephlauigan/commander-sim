//! The Tier 5 urza-mono-blue-artifacts and yawgmoth-mono-black-aristocrats cards (impls/t45_urza_yawgmoth.rs and the
//! compiled pattern under `// --- Tiers 4/5 (urza-yawgmoth)`) on hand-built positions. Each expectation follows the
//! card's Oracle text.

use sim_core::cardcode;
use sim_core::dsl::has_kw;
use sim_core::engine::hooks::run_hook;
use sim_core::engine::mana::{cost_of, total_mana};
use sim_core::engine::values::{epow, etgh, untargetable};
use sim_core::engine::{cast, turn, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{combos, t45_urza_yawgmoth as t45};
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const URZA: &str = "urza-mono-blue-artifacts";
const YAWG: &str = "yawgmoth-mono-black-aristocrats";

fn urza() -> Game {
    let mut g = table(&[URZA, "cloud"]);
    g.player_mut(P0).cmd_in_zone = false;
    g
}

fn yawg() -> Game {
    let mut g = table(&[YAWG, "cloud"]);
    g.player_mut(P0).cmd_in_zone = false;
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn tokens(g: &Game, p: PlayerId, ty: &str) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).ttypes.contains(&ty))
        .collect()
}

fn untapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options");
    f(g, src, p, post).unwrap()
}

fn cast_enter(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    zones::enter(g, p, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap()
}

/// p's untap step alone: its library holds no land and its hand is empty, so nothing is played after it
fn untap(g: &mut Game, p: PlayerId) {
    g.player_mut(p).hand.clear();
    let filler = card(g, "Counterspell");
    g.player_mut(p).library = vec![filler; 20];
    turn::step_start(g, p).unwrap();
}

// ------------------------------------------------------------------ Winter Orb, Static Orb, Back to Basics, Clock
#[test]
fn winter_orb_untaps_one_land_and_back_to_basics_keeps_nonbasics_tapped() {
    let mut g = urza();
    perm(&mut g, P0, "Winter Orb");
    lands(&mut g, P1, "Plains", 3, true);
    untap(&mut g, P1);
    assert_eq!(untapped_lands(&g, P1), 1, "Winter Orb: one land");
    // a tapped Orb doesn't hold the next untap step
    let mut g = urza();
    let o = perm(&mut g, P0, "Winter Orb");
    g.perm_mut(o).tapped = true;
    lands(&mut g, P1, "Plains", 3, true);
    untap(&mut g, P1);
    assert_eq!(untapped_lands(&g, P1), 3);
    // Back to Basics: only the basics untap
    let mut g = urza();
    perm(&mut g, P0, "Back to Basics");
    lands(&mut g, P1, "Plains", 2, true);
    lands(&mut g, P1, "Ancient Tomb", 1, true);
    untap(&mut g, P1);
    assert_eq!(untapped_lands(&g, P1), 2, "the Tomb stays tapped");
}

#[test]
fn static_orb_untaps_two_permanents_and_unwinding_clock_untaps_artifacts_on_other_turns() {
    let mut g = urza();
    perm(&mut g, P0, "Static Orb");
    lands(&mut g, P1, "Plains", 3, true);
    let sol = perm(&mut g, P1, "Sol Ring");
    g.perm_mut(sol).tapped = true;
    untap(&mut g, P1);
    assert!(!g.perm(sol).tapped, "Sol Ring first");
    assert_eq!(untapped_lands(&g, P1), 1, "two permanents in all");
    // Unwinding Clock: P0's artifacts untap during P1's untap step, the Orb's limit notwithstanding
    let mut g = urza();
    perm(&mut g, P0, "Static Orb");
    perm(&mut g, P0, "Unwinding Clock");
    let vault = perm(&mut g, P0, "Mana Vault");
    let sig = perm(&mut g, P0, "Arcane Signet");
    g.perm_mut(vault).tapped = true;
    g.perm_mut(sig).tapped = true;
    let isl = lands(&mut g, P0, "Island", 1, true);
    untap(&mut g, P1);
    assert!(!g.perm(vault).tapped && !g.perm(sig).tapped, "the Clock untaps artifacts");
    assert!(g.perm(isl[0]).tapped, "not lands");
    // Urza taps no Orb for mana
    let mut g = urza();
    perm(&mut g, P0, "Urza, Lord High Artificer");
    perm(&mut g, P0, "Winter Orb");
    perm(&mut g, P0, "Static Orb");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(total_mana(&g, P0, false), 3, "Sol Ring's two and the Construct's {{U}}, no Orb");
}

// ------------------------------------------------------------------ Welding Jar
#[test]
fn welding_jar_regenerates_a_destroyed_artifact() {
    let mut g = urza();
    let jar = perm(&mut g, P0, "Welding Jar");
    let sc = perm(&mut g, P0, "Isochron Scepter");
    zones::die(&mut g, sc, "destroy").unwrap();
    assert!(g.perm(sc).on_bf && g.perm(sc).tapped, "regenerated: tapped, still there");
    assert!(!g.perm(jar).on_bf, "the Jar was sacrificed");
    zones::die(&mut g, sc, "destroy").unwrap();
    assert!(!g.perm(sc).on_bf, "no Jar left");
}

// ------------------------------------------------------------------ Forensic Gadgeteer
#[test]
fn forensic_gadgeteer_makes_clues_cheaper_and_combos_with_basalt_monolith() {
    let mut g = urza();
    assert_eq!(t45::clue_cost(&g, P0), 2);
    perm(&mut g, P0, "Forensic Gadgeteer");
    assert_eq!(t45::clue_cost(&g, P0), 1, "{{1}} less");
    assert_eq!(t45::artifact_cost(&g, P0, 1, 0), 1, "not below one mana");
    assert_eq!(t45::artifact_cost(&g, P0, 3, 0), 2, "Basalt Monolith's untap");
    // the investigate trigger is compiled: an artifact spell makes a Clue
    let i = combos::combo_index("Forensic Gadgeteer").unwrap();
    perm(&mut g, P0, "Basalt Monolith");
    assert!(!combos::ready(&g, P0, i), "no mana sink");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(combos::ready(&g, P0, i));
}

// ------------------------------------------------------------------ Padeem
#[test]
fn padeem_gives_artifacts_hexproof_and_draws_with_the_biggest_artifact() {
    let mut g = urza();
    let pd = perm(&mut g, P0, "Padeem, Consul of Innovation");
    let sol = perm(&mut g, P0, "Sol Ring");
    let theirs = perm(&mut g, P1, "Mox Opal");
    assert!(untargetable(&g, sol) && has_kw(&g, sol, "hexproof"));
    assert!(!untargetable(&g, pd), "Padeem is no artifact");
    assert!(!untargetable(&g, theirs), "only yours");
    let n = g.player(P0).hand.len();
    run_hook(&mut g, pd, Event::Upkeep, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "Sol Ring (1) is the biggest artifact");
    perm(&mut g, P1, "The One Ring");
    run_hook(&mut g, pd, Event::Upkeep, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "the opponent's One Ring is bigger");
}

// ------------------------------------------------------------------ The Reality Chip
#[test]
fn reality_chip_reconfigures_then_plays_from_the_top() {
    let mut g = urza();
    lands(&mut g, P0, "Island", 6, false);
    let chip = perm(&mut g, P0, "The Reality Chip");
    let host = perm(&mut g, P0, "Etherium Sculptor");
    g.active = Some(P0);
    g.step = Step::Main1;
    let o = options(&mut g, chip, P0, Some(false));
    assert_eq!(o.len(), 1, "reconfigure");
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert_eq!(g.perm(chip).attached, Some(host));
    // lands from the top
    let f = g.registry.get(card(&g, "The Reality Chip")).unwrap().lands_from_top.unwrap();
    assert_eq!(f(&g, chip, P0), 1);
    // a spell from the top
    let top = take(&mut g, P0, "Thought Vessel");
    g.player_mut(P0).library.push(top);
    let o = options(&mut g, chip, P0, Some(false));
    assert!(o.iter().any(|x| x.label.contains("Thought Vessel")));
    let o = o.into_iter().find(|x| x.label.contains("Thought Vessel")).unwrap();
    assert!(run(&mut g, o, P0));
    assert!(on_bf(&g, P0, "Thought Vessel").is_some());
    // the host leaves: unattached
    zones::die(&mut g, host, "destroy").unwrap();
    assert_eq!(f(&g, chip, P0), 0);
}

// ------------------------------------------------------------------ affinity
#[test]
fn thought_monitor_and_thoughtcast_cost_one_less_per_artifact() {
    let mut g = urza();
    let tm = card(&g, "Thought Monitor");
    let tc = card(&g, "Thoughtcast");
    assert_eq!(cost_of(&g, P0, tm).0, 6);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Ornithopter");
    perm(&mut g, P0, "Mox Opal");
    assert_eq!(cost_of(&g, P0, tm).0, 3);
    assert_eq!(cost_of(&g, P0, tc).0, 1);
}

// ------------------------------------------------------------------ Uthros, Titanic Godcore
#[test]
fn uthros_stations_with_a_creature_and_taps_for_each_artifact_at_twelve() {
    let mut g = urza();
    let l = lands(&mut g, P0, "Uthros, Titanic Godcore", 1, false)[0];
    let f = g.registry.get(card(&g, "Uthros, Titanic Godcore")).copied().unwrap();
    assert_eq!((f.dyn_mana_land.unwrap())(&g, P0, l), 1);
    let big = token(&mut g, P0, 12);
    g.active = Some(P0);
    g.step = Step::Main2;
    let o = (f.land_options.unwrap())(&mut g, l, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert!(g.perm(big).tapped);
    assert_eq!(g.perm(l).data.int(DataKey::Charge), 12);
    for n in ["Sol Ring", "Ornithopter", "Mox Opal", "Arcane Signet"] {
        perm(&mut g, P0, n);
    }
    assert_eq!((f.dyn_mana_land.unwrap())(&g, P0, l), 3, "four artifacts, less the {{U}} it costs");
}

// ------------------------------------------------------------------ Entomb
#[test]
fn entomb_bins_the_best_reanimation_target_or_a_recursive_creature() {
    let mut g = yawg();
    lands(&mut g, P0, "Swamp", 2, false);
    let lib = ["Bloodghast", "Sheoldred, the Apocalypse", "Swamp", "Blood Artist"];
    g.player_mut(P0).library = lib.iter().map(|n| card(&g, n)).collect();
    let e = hand(&mut g, P0, &["Entomb"])[0];
    let f = g.registry.get(e).and_then(|i| i.hand_options).unwrap();
    let o = f(&mut g, e, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("Bloodghast"), "no reanimation in hand: a creature that returns by itself");
    hand(&mut g, P0, &["Reanimate"]);
    let o = f(&mut g, e, P0, None).unwrap();
    assert!(o[0].label.contains("Sheoldred"), "{}", o[0].label);
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert!(g.player(P0).gy.contains(&card(&g, "Sheoldred, the Apocalypse")));
    assert!(g.player(P0).gy.contains(&e));
}

// ------------------------------------------------------------------ Genesis Chamber
#[test]
fn genesis_chamber_gives_each_nontoken_creature_a_myr() {
    let mut g = yawg();
    let gc = perm(&mut g, P0, "Genesis Chamber");
    perm(&mut g, P1, "Blood Artist");
    assert_eq!(tokens(&g, P1, "myr").len(), 1, "its controller's Myr");
    token(&mut g, P0, 1);
    assert!(tokens(&g, P0, "myr").is_empty(), "not for tokens");
    g.perm_mut(gc).tapped = true;
    perm(&mut g, P0, "Gravecrawler");
    assert!(tokens(&g, P0, "myr").is_empty(), "tapped: nothing");
}

// ------------------------------------------------------------------ Ayara (compiled)
#[test]
fn ayara_drains_one_as_she_or_another_black_creature_enters() {
    let mut g = yawg();
    let life = g.player(P0).life;
    perm(&mut g, P0, "Ayara, First of Locthwain");
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, life + 1, "one life in all");
    perm(&mut g, P0, "Gravecrawler");
    assert_eq!(g.player(P1).life, 38);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Ornithopter");
    assert_eq!(g.player(P1).life, 38, "a colourless creature doesn't count");
}

// ------------------------------------------------------------------ Blight Mound
#[test]
fn blight_mound_makes_pests_that_gain_life_and_grow_when_attacking() {
    let mut g = yawg();
    perm(&mut g, P0, "Blight Mound");
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token), "no enter token");
    let gc = perm(&mut g, P0, "Gravecrawler");
    zones::die(&mut g, gc, "sac").unwrap();
    let pests = tokens(&g, P0, "pest");
    assert_eq!(pests.len(), 1);
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sac").unwrap();
    assert_eq!(tokens(&g, P0, "pest").len(), 1, "tokens dying make none");
    let life = g.player(P0).life;
    let f = g.registry.get(card(&g, "Blight Mound")).and_then(|i| i.attack).unwrap();
    let mound = on_bf(&g, P0, "Blight Mound").unwrap();
    f(&mut g, mound, P0, &pests, P1).unwrap();
    assert_eq!(epow(&g, pests[0]), 2);
    assert!(has_kw(&g, pests[0], "menace"));
    zones::die(&mut g, pests[0], "sac").unwrap();
    assert_eq!(g.player(P0).life, life + 1, "the Pest's death");
}

// ------------------------------------------------------------------ Forsaken Miner
#[test]
fn forsaken_miner_returns_for_b_on_a_crime() {
    let mut g = yawg();
    lands(&mut g, P0, "Swamp", 1, false);
    let m = take(&mut g, P0, "Forsaken Miner");
    g.player_mut(P0).gy.push(m);
    sim_core::impls::bites::crime(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Forsaken Miner").is_some());
    assert!(!g.player(P0).gy.contains(&m));
    assert_eq!(total_mana(&g, P0, false), 0, "paid {{B}}");
}

// ------------------------------------------------------------------ Hancock
#[test]
fn hancock_pumps_other_zombies_by_its_counters() {
    let mut g = yawg();
    let h = perm(&mut g, P0, "Hancock, Ghoulish Mayor");
    let z = perm(&mut g, P0, "Butcher Ghoul");
    let other = perm(&mut g, P0, "Blood Artist");
    assert_eq!(epow(&g, z), 1, "no counters yet");
    zones::die(&mut g, h, "sac").unwrap(); // undying: back with a +1/+1 counter
    let h2 = on_bf(&g, P0, "Hancock, Ghoulish Mayor").expect("undying");
    assert_eq!(g.perm(h2).plus, 1);
    assert_eq!((epow(&g, z), etgh(&g, z)), (2, 2));
    assert_eq!(epow(&g, other), 0, "not a Zombie");
    assert_eq!(epow(&g, h2), 3, "not itself: 2/1 with its counter");
}

// ------------------------------------------------------------------ Necroskitter
#[test]
fn necroskitter_takes_opposing_creatures_dying_with_minus_counters() {
    let mut g = yawg();
    perm(&mut g, P0, "Necroskitter");
    let a = perm(&mut g, P1, "Blood Artist");
    sim_core::engine::repl::put_plus(&mut g, a, -1);
    zones::die(&mut g, a, "sba").unwrap();
    assert!(on_bf(&g, P0, "Blood Artist").is_some(), "it returns under my control");
    let b = perm(&mut g, P1, "Zulaport Cutthroat");
    zones::die(&mut g, b, "destroy").unwrap();
    assert!(on_bf(&g, P0, "Zulaport Cutthroat").is_none(), "no counter: no return");
}

// ------------------------------------------------------------------ Nine-Lives Familiar
#[test]
fn nine_lives_familiar_returns_at_the_end_step_with_one_fewer_counter() {
    let mut g = yawg();
    let m = cast_enter(&mut g, P0, "Nine-Lives Familiar");
    assert_eq!(g.perm(m).data.int(DataKey::Revival), 8);
    zones::die(&mut g, m, "sac").unwrap();
    assert!(on_bf(&g, P0, "Nine-Lives Familiar").is_none());
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    let n = on_bf(&g, P0, "Nine-Lives Familiar").expect("back");
    assert_eq!(g.perm(n).data.get(DataKey::Revival), Some(&Val::Int(7)));
    // not cast: no counters, so it stays dead
    let mut g = yawg();
    let m = perm(&mut g, P0, "Nine-Lives Familiar");
    zones::die(&mut g, m, "sac").unwrap();
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Nine-Lives Familiar").is_none());
}

// ------------------------------------------------------------------ The Meathook Massacre
#[test]
fn meathook_massacre_pays_x_and_shrinks_every_creature() {
    let mut g = yawg();
    lands(&mut g, P0, "Swamp", 2, false);
    let mine = perm(&mut g, P0, "Sheoldred, the Apocalypse");
    let small = perm(&mut g, P1, "Ornithopter");
    let t2 = token(&mut g, P1, 2);
    let big = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    let life = g.player(P0).life;
    cast_enter(&mut g, P0, "The Meathook Massacre");
    assert_eq!(total_mana(&g, P0, false), 0, "X = 2 paid");
    assert!(!g.perm(small).on_bf && !g.perm(t2).on_bf, "-2/-2 kills the 0/2 and the 2/2");
    assert!(g.perm(big).on_bf && g.perm(mine).on_bf && etgh(&g, mine) == 3);
    assert_eq!(g.player(P0).life, life + 2, "an opposing creature dying: you gain 1");
}

// ------------------------------------------------------------------ combos
#[test]
fn hullbreaker_horror_loops_free_artifacts_with_urza() {
    let mut g = urza();
    let i = combos::combo_index("Hullbreaker Horror + Urza").unwrap();
    perm(&mut g, P0, "Hullbreaker Horror");
    perm(&mut g, P0, "Ornithopter");
    hand(&mut g, P0, &["Mox Opal"]);
    assert!(!combos::ready(&g, P0, i), "no Urza, no Sol Ring");
    perm(&mut g, P0, "Urza, Lord High Artificer");
    assert!(combos::ready(&g, P0, i));
}

#[test]
fn skeleton_plunderer_and_an_altar_loop_into_a_death_payoff() {
    let mut g = yawg();
    let i = combos::combo_index("Pitiless Plunderer").unwrap();
    perm(&mut g, P0, "Pitiless Plunderer");
    perm(&mut g, P0, "Ashnod's Altar");
    perm(&mut g, P0, "Reassembling Skeleton");
    assert!(!combos::ready(&g, P0, i), "no payoff");
    perm(&mut g, P0, "The Meathook Massacre");
    assert!(combos::ready(&g, P0, i), "Meathook is a payoff");
}

#[test]
fn entomb_resolves_through_the_hand_hook_without_casting_twice() {
    // the card's resolve hook puts one card in the graveyard and Entomb itself goes there after
    let mut g = yawg();
    let lib = ["Bloodghast", "Swamp"];
    g.player_mut(P0).library = lib.iter().map(|n| card(&g, n)).collect();
    let e: CardId = take(&mut g, P0, "Entomb");
    cast::resolve(&mut g, P0, e, &Ctx::default(), sim_core::sym::intern("hand")).unwrap();
    assert_eq!(g.player(P0).gy.iter().filter(|&&c| c == card(&g, "Bloodghast")).count(), 1);
    assert_eq!(g.player(P0).library.len(), 1);
}
