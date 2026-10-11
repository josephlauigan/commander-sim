//! Tier 2 card code (impls/t2.rs, Python's cards/impl/t2.py) on hand-built positions: the blink engines, Palace
//! Jailer and the monarch, the any-target damage heuristic, Kaalia, Angel of Serenity, The Gitrog Monster, Meren,
//! evoke, Lord Windgrace, Restoration Angel, Master of Cruelties, Glorybringer and Survival of the Fittest. Each
//! expectation was checked against the Python card code on the same position.

use sim_core::cardcode;
use sim_core::engine::{hooks, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t2;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const BR: &str = "brago-azorius-blink-control";
const KA: &str = "kaalia-mardu-creature-cheat";
const WG: &str = "lord-windgrace-jund-lands";
const ME: &str = "meren-golgari-recursion";
const SY: &str = "sythis-selesnya-enchantress";

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).name == name)
}

fn names(g: &Game, cs: &[CardId]) -> Vec<String> {
    let mut v: Vec<String> = cs.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    v.sort();
    v
}

fn fire(g: &mut Game, e: Event, p: PlayerId) {
    hooks::fire_trigger(g, e, Call::Player { p }).unwrap();
}

/// the options a permanent's `options` hook offers p
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).unwrap();
    f(g, src, p, post).unwrap()
}

/// run an option's action
fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        _ => panic!("not a card action"),
    }
}

// ------------------------------------------------------------------ blink engines
#[test]
fn soulherder_blinks_at_your_end_step_and_grows() {
    let mut g = table(&[BR, KA]);
    let herd = perm(&mut g, P0, "Soulherder");
    let seeker = perm(&mut g, P0, "Wall of Omens");
    assert_eq!(t2::blink_value(&g, P0, seeker), 3); // its compiled enters draw
    assert_eq!(t2::blink_value(&g, P0, herd), 0);
    fire(&mut g, Event::EndStep, P1); // an opponent's end step: nothing
    assert!(g.perm(seeker).on_bf);
    assert_eq!(g.perm(herd).plus, 0);
    let n = g.player(P0).hand.len();
    fire(&mut g, Event::EndStep, P0);
    assert!(!g.perm(seeker).on_bf, "blinked");
    assert!(named(&g, P0, "Wall of Omens").is_some());
    assert_eq!(g.perm(herd).plus, 1, "a creature was exiled from the battlefield");
    assert_eq!(g.player(P0).hand.len(), n + 1, "Wall of Omens drew again");
    assert_eq!(g.player(P0).stats.get("flicker Soulherder"), Some(&1));
}

#[test]
fn teleportation_circle_untaps_a_rock_with_no_creature_worth_it() {
    let mut g = table(&[BR, KA]);
    perm(&mut g, P0, "Teleportation Circle");
    let rock = perm(&mut g, P0, "Mind Stone");
    g.perm_mut(rock).tapped = true;
    fire(&mut g, Event::EndStep, P0);
    assert!(!g.perm(rock).on_bf);
    let back = named(&g, P0, "Mind Stone").unwrap();
    assert!(!g.perm(back).tapped, "back untapped for the opponents' turns");
}

#[test]
fn blinks_nest_three_deep_at_most() {
    let mut g = table(&[BR, KA]);
    let seeker = perm(&mut g, P0, "Wall of Omens");
    g.blink_depth = 3;
    assert_eq!(t2::blink(&mut g, P0, seeker).unwrap(), None);
    assert!(g.perm(seeker).on_bf);
    g.blink_depth = 0;
    let n = t2::blink(&mut g, P0, seeker).unwrap().unwrap();
    assert_ne!(n, seeker);
    assert_eq!(g.blink_depth, 0);
}

// ------------------------------------------------------------------ Palace Jailer and the monarch
#[test]
fn palace_jailer_exiles_until_an_opponent_becomes_the_monarch() {
    let mut g = table(&[BR, KA]);
    let t = perm(&mut g, P1, "Baneslayer Angel");
    let j = perm(&mut g, P0, "Palace Jailer");
    assert_eq!(g.monarch, Some(P0));
    assert!(!g.perm(t).on_bf);
    assert_eq!(names(&g, &g.player(P1).exile.clone()), ["Baneslayer Angel"]);
    cardcode::become_monarch(&mut g, P1).unwrap();
    assert_eq!(g.monarch, Some(P1));
    assert!(named(&g, P1, "Baneslayer Angel").is_some(), "it returns");
    assert_eq!(g.perm(j).data.get(DataKey::Oring), Some(&Val::List(vec![])));
}

// ------------------------------------------------------------------ any target
#[test]
fn any_target_damage_finishes_a_player_then_kills_a_creature_then_hits_a_face() {
    let mut g = table(&[KA, BR, SY]);
    g.player_mut(P1).life = 3;
    t2::best_target_any(&mut g, P0, 3).unwrap();
    assert!(g.player(P1).life <= 0);
    assert_eq!(g.player(P2).life, 40);

    let mut g = table(&[KA, BR, SY]);
    let m = perm(&mut g, P1, "Restoration Angel");
    t2::best_target_any(&mut g, P0, 4).unwrap();
    assert!(!g.perm(m).on_bf, "a 3/4 worth 5.5 dies to 4 damage");
    assert_eq!((g.player(P1).life, g.player(P2).life), (40, 40));

    let mut g = table(&[KA, BR, SY]);
    let s = perm(&mut g, P2, "Sythis, Harvest's Hand");
    t2::best_target_any(&mut g, P0, 2).unwrap();
    assert!(!g.perm(s).on_bf);
    assert_eq!((g.player(P1).life, g.player(P2).life), (40, 40));
}

// ------------------------------------------------------------------ Kaalia
#[test]
fn kaalia_puts_the_best_angel_demon_or_dragon_in_attacking() {
    let mut g = table(&[KA, BR]);
    let k = perm(&mut g, P0, "Kaalia of the Vast");
    hand(&mut g, P0, &["Baneslayer Angel", "Glorybringer", "Sign in Blood"]);
    g.new_attackers.clear();
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![k], d: P1 }).unwrap();
    let b = named(&g, P0, "Baneslayer Angel").unwrap();
    assert!(g.perm(b).tapped && !g.perm(b).sick);
    assert_eq!(g.new_attackers, vec![b]);
    assert_eq!(names(&g, &g.player(P0).hand.clone()), ["Glorybringer", "Sign in Blood"]);
    let drak = card(&g, "Drakuseth, Maw of Flames");
    assert_eq!(t2::kaalia_prio(&g, P0, drak), Some(0), "held for Kaalia");
    assert_eq!(t2::kaalia_prio(&g, P0, card(&g, "Sign in Blood")), None);
}

// ------------------------------------------------------------------ Angel of Serenity
#[test]
fn angel_of_serenity_exiles_three_and_returns_them_to_hand() {
    let mut g = table(&[KA, BR]);
    for n in ["Restoration Angel", "Deadeye Navigator", "Baneslayer Angel"] {
        perm(&mut g, P1, n);
    }
    let s = perm(&mut g, P0, "Angel of Serenity");
    assert!(g.player(P1).perms.is_empty());
    assert_eq!(names(&g, &g.player(P1).exile.clone()), ["Baneslayer Angel", "Deadeye Navigator", "Restoration Angel"]);
    let before = g.player(P1).hand.len();
    zones::die(&mut g, s, "destroy").unwrap();
    assert!(g.player(P1).exile.is_empty());
    assert_eq!(g.player(P1).hand.len(), before + 3);
}

// ------------------------------------------------------------------ The Gitrog Monster
#[test]
fn gitrog_sacrifices_a_tapped_land_and_draws_or_itself() {
    let mut g = table(&[WG, BR]);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Swamp", 1, true);
    lands(&mut g, P0, "Mountain", 1, false);
    let gm = perm(&mut g, P0, "The Gitrog Monster");
    let n = g.player(P0).hand.len();
    fire(&mut g, Event::Upkeep, P0);
    let left: Vec<String> = g.player(P0).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect();
    assert_eq!(left, ["Forest", "Forest", "Mountain"]);
    assert_eq!(names(&g, &g.player(P0).gy.clone()), ["Swamp"]);
    assert_eq!(g.player(P0).hand.len(), n + 1, "a land went to the graveyard: draw");
    assert!(g.perm(gm).on_bf);
    fire(&mut g, Event::Upkeep, P0);
    assert!(!g.perm(gm).on_bf, "three lands: it sacrifices itself");
}

// ------------------------------------------------------------------ Meren
#[test]
fn meren_returns_by_experience() {
    let mut g = table(&[ME, BR]);
    perm(&mut g, P0, "Meren of Clan Nel Toth");
    let slime = perm(&mut g, P0, "Acidic Slime");
    let seer = perm(&mut g, P0, "Viscera Seer");
    zones::die(&mut g, seer, "sac").unwrap();
    assert_eq!(g.player(P0).experience, 1);
    fire(&mut g, Event::EndStep, P0);
    assert!(named(&g, P0, "Viscera Seer").is_some(), "mana value 1 fits one experience: battlefield");
    zones::die(&mut g, slime, "destroy").unwrap();
    g.player_mut(P0).experience = 1;
    fire(&mut g, Event::EndStep, P0);
    assert!(named(&g, P0, "Acidic Slime").is_none());
    assert_eq!(names(&g, &g.player(P0).hand.clone()), ["Acidic Slime"], "too big: hand");
}

// ------------------------------------------------------------------ evoke
#[test]
fn shriekmaw_is_evoked_when_its_full_cost_is_out_of_reach() {
    let mut g = table(&[ME, BR]);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Shriekmaw"]);
    let r = perm(&mut g, P1, "Restoration Angel");
    assert!(t2::evoke_options(&mut g, P0, None).unwrap().is_empty());
    let o = t2::evoke_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "evoke Shriekmaw");
    assert_eq!(o[0].utility, 3.0);
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(r).on_bf, "its enters trigger destroyed the Angel");
    assert_eq!(names(&g, &g.player(P0).gy.clone()), ["Shriekmaw"]);
}

// ------------------------------------------------------------------ Lord Windgrace
#[test]
fn windgrace_minus_three_returns_the_two_best_lands() {
    let mut g = table(&[WG, BR]);
    let wg = perm(&mut g, P0, "Lord Windgrace");
    for n in ["Raging Ravine", "Forest", "Bojuka Bog"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let opts = options(&mut g, wg, P0, Some(false));
    let got: Vec<(String, f64)> =
        opts.iter().map(|o| (o.label.clone(), (o.utility * 1000.0).round() / 1000.0)).collect();
    assert_eq!(got, [("Lord Windgrace +2".to_string(), 2.8), ("Lord Windgrace -3".to_string(), 4.05)]);
    assert!(run(&mut g, P0, &opts[1]));
    let ls: Vec<String> = g.player(P0).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect();
    assert_eq!(ls, ["Raging Ravine", "Forest"]);
    assert_eq!(names(&g, &g.player(P0).gy.clone()), ["Bojuka Bog"]);
    assert_eq!(g.perm(wg).loyalty, Some(2));
}

// ------------------------------------------------------------------ Restoration Angel
#[test]
fn restoration_angel_blinks_the_best_enter_effect() {
    let mut g = table(&[BR, KA]);
    let s = perm(&mut g, P0, "Wall of Omens");
    let r = perm(&mut g, P0, "Mulldrifter");
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Restoration Angel");
    assert!(!g.perm(s).on_bf, "Wall of Omens (3) over Mulldrifter (2)");
    assert!(g.perm(r).on_bf);
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn restoration_angel_is_cast_at_the_end_of_an_opponents_turn() {
    let mut g = table(&[BR, KA]);
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &["Restoration Angel"])[0];
    let s = perm(&mut g, P0, "Wall of Omens");
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    assert!(f(&mut g, c, P0, None).unwrap().is_empty(), "not on your own turn");
    g.active = Some(P1);
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "only at the end-of-turn window");
    let o = f(&mut g, c, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!((o[0].label.as_str(), o[0].utility), ("Restoration Angel (end of turn)", 4.5));
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(s).on_bf, "blinked");
    assert!(named(&g, P0, "Restoration Angel").is_some() && named(&g, P0, "Wall of Omens").is_some());
    assert_eq!(g.resto_target, None);
}

// ------------------------------------------------------------------ Master of Cruelties, Glorybringer
#[test]
fn master_of_cruelties_unblocked_alone_sets_life_to_one() {
    let mut g = table(&[KA, BR]);
    let moc = perm(&mut g, P0, "Master of Cruelties");
    let mut assign = vec![];
    cardcode::blocks_hooks(&mut g, P0, &[moc], P1, &mut assign).unwrap();
    assert_eq!(g.player(P1).life, 1);
    assert_eq!(g.perm(moc).eot_pt, (-1, 0), "it deals no combat damage");
}

#[test]
fn glorybringer_exerts_every_other_turn() {
    let mut g = table(&[KA, BR]);
    let gb = perm(&mut g, P0, "Glorybringer");
    let t = perm(&mut g, P1, "Coiling Oracle");
    let attack = |g: &mut Game| {
        hooks::fire_trigger(g, Event::Attack, Call::Attack { p: P0, atk: vec![gb], d: P1 }).unwrap();
    };
    attack(&mut g);
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.perm(gb).data.get(DataKey::Exerted), Some(&Val::Int(1)));
    let t = perm(&mut g, P1, "Welcoming Vampire");
    g.player_mut(P0).turns += 1;
    attack(&mut g);
    assert!(g.perm(t).on_bf, "exerted last turn");
    g.player_mut(P0).turns += 1;
    attack(&mut g);
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.perm(gb).data.get(DataKey::Exerted), Some(&Val::Int(3)));
}

// ------------------------------------------------------------------ Survival of the Fittest
#[test]
fn survival_discards_a_creature_for_the_best_one() {
    let mut g = table(&[ME, BR]);
    lands(&mut g, P0, "Forest", 3, false);
    let sv = perm(&mut g, P0, "Survival of the Fittest");
    hand(&mut g, P0, &["Viscera Seer"]);
    assert!(options(&mut g, sv, P0, None).is_empty());
    let o = options(&mut g, sv, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Survival of the Fittest");
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(names(&g, &g.player(P0).gy.clone()), ["Viscera Seer"]);
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(g.db.get(g.player(P0).hand[0]).creature);
}
