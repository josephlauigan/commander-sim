//! The Tier 2 kaalia-mardu-creature-cheat and sythis-selesnya-enchantress cards (impls/t12_kaalia_sythis.rs and the
//! compiled patterns under `// --- Tiers 1/2 (kaalia-sythis)`) on hand-built positions. Each expectation follows the
//! card's Oracle text.

use sim_core::cardcode;
use sim_core::engine::combat::{attack_restrictions, can_block};
use sim_core::engine::hooks::{fire_sba, run_hook, total_count};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, life, mana, removal, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t12_kaalia_sythis as t12;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KAALIA: &str = "kaalia-mardu-creature-cheat";
const SYTHIS: &str = "sythis-selesnya-enchantress";

fn kaalia() -> Game {
    table(&[KAALIA, "seph", "sauron"])
}

fn sythis() -> Game {
    table(&[SYTHIS, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn run(g: &mut Game, o: &Opt, p: PlayerId) -> bool {
    match o.act.clone().unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, p, post).unwrap()
}

/// a creature card of p's on the battlefield (its card goes to p's graveyard when it dies)
fn creature(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    perm(g, p, name)
}

// ------------------------------------------------------------------ Angelic Arbiter
#[test]
fn arbiter_stops_casters_attacking_and_attackers_casting() {
    let mut g = kaalia();
    perm(&mut g, P0, "Angelic Arbiter");
    assert_eq!(attack_restrictions(&g, P1, P0).1, None, "no spell cast yet: free to attack");
    let bolt = card(&g, "Lightning Bolt");
    g.player_mut(P1).turn_casts = Some((g.turn_stamp(), vec![bolt]));
    assert_eq!(attack_restrictions(&g, P1, P0).1, Some(0), "an opponent who cast a spell can't attack");
    assert_eq!(attack_restrictions(&g, P0, P1).1, None, "its controller can");
    let c = hand(&mut g, P2, &["Night's Whisper"])[0];
    assert!(cast::castable(&g, P2, c, "hand"), "P2 hasn't attacked");
    g.player_mut(P2).attacking = Some(g.turn_stamp());
    assert!(!cast::castable(&g, P2, c, "hand"), "an opponent who attacked can't cast spells");
    let mine = hand(&mut g, P0, &["Night's Whisper"])[0];
    g.player_mut(P0).attacking = Some(g.turn_stamp());
    assert!(cast::castable(&g, P0, mine, "hand"), "its controller can");
}

// ------------------------------------------------------------------ Avacyn, Angel of Horror
#[test]
fn avacyn_returns_herself_and_your_nontoken_creatures_at_the_end_step() {
    let mut g = kaalia();
    let av = perm(&mut g, P0, "Avacyn, Angel of Horror");
    let lyra = creature(&mut g, P0, "Lyra Dawnbringer");
    let tok = token(&mut g, P0, 2);
    let theirs = creature(&mut g, P1, "Llanowar Elves");
    for m in [lyra, tok, theirs] {
        zones::die(&mut g, m, "destroy").unwrap();
    }
    assert!(g.player(P0).gy.contains(&card(&g, "Lyra Dawnbringer")));
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Lyra Dawnbringer").is_some(), "the nontoken creature returns");
    assert!(on_bf(&g, P0, "Llanowar Elves").is_none() && on_bf(&g, P1, "Llanowar Elves").is_none(), "not theirs");
    zones::die(&mut g, av, "destroy").unwrap();
    assert!(on_bf(&g, P0, "Avacyn, Angel of Horror").is_none());
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Avacyn, Angel of Horror").is_some(), "Avacyn returns herself");
}

#[test]
fn avacyn_returns_the_creatures_dying_after_her_in_the_same_wipe() {
    let mut g = kaalia();
    let av = perm(&mut g, P0, "Avacyn, Angel of Horror");
    let lyra = creature(&mut g, P0, "Lyra Dawnbringer");
    g.cur_batch = Some(9);
    zones::die(&mut g, av, "destroy").unwrap();
    zones::die(&mut g, lyra, "destroy").unwrap();
    g.cur_batch = None;
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Avacyn, Angel of Horror").is_some());
    assert!(on_bf(&g, P0, "Lyra Dawnbringer").is_some(), "her ability looks back in time");
}

// ------------------------------------------------------------------ Liesa, Forgotten Archangel
#[test]
fn liesa_returns_yours_to_hand_and_exiles_theirs() {
    let mut g = kaalia();
    let liesa = perm(&mut g, P0, "Liesa, Forgotten Archangel");
    let lyra = creature(&mut g, P0, "Lyra Dawnbringer");
    let elf = creature(&mut g, P1, "Llanowar Elves");
    zones::die(&mut g, lyra, "destroy").unwrap();
    zones::die(&mut g, elf, "destroy").unwrap();
    let elves = card(&g, "Llanowar Elves");
    assert!(g.player(P1).exile.contains(&elves) && !g.player(P1).gy.contains(&elves), "theirs is exiled instead");
    let lc = card(&g, "Lyra Dawnbringer");
    assert!(g.player(P0).gy.contains(&lc) && !g.player(P0).hand.contains(&lc));
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).hand.contains(&lc), "back to its owner's hand at the end step");
    zones::die(&mut g, liesa, "destroy").unwrap();
    cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(!g.player(P0).hand.contains(&card(&g, "Liesa, Forgotten Archangel")), "another creature only");
}

// ------------------------------------------------------------------ Drana and Linvala
#[test]
fn drana_and_linvala_shuts_off_opposing_creature_abilities() {
    let mut g = kaalia();
    let birds = creature(&mut g, P1, "Birds of Paradise");
    let mine = creature(&mut g, P0, "Llanowar Elves");
    assert!(!cardcode::ability_locked(&g, birds, P1));
    perm(&mut g, P0, "Drana and Linvala");
    assert!(cardcode::ability_locked(&g, birds, P1), "an opponent's creature");
    assert!(!cardcode::ability_locked(&g, mine, P0), "not yours");
    assert_eq!(total_count(&g, Event::NoCreatureMana, P1), 1, "their creatures' mana abilities too");
    assert_eq!(total_count(&g, Event::NoCreatureMana, P0), 0);
}

// ------------------------------------------------------------------ Kaalia, Zenith Seeker
#[test]
fn zenith_seeker_takes_an_angel_a_demon_and_a_dragon() {
    let mut g = kaalia();
    let names = ["Plains", "Sol Ring", "Terror of the Peaks", "Rune-Scarred Demon", "Lyra Dawnbringer", "Swamp"];
    let top: Vec<CardId> = names.iter().map(|n| take(&mut g, P0, n)).collect();
    let deep = take(&mut g, P0, "Mountain");
    g.player_mut(P0).library = vec![deep];
    g.player_mut(P0).library.extend(top.iter().rev()); // "Plains" on top
    g.player_mut(P0).hand.clear();
    perm(&mut g, P0, "Kaalia, Zenith Seeker");
    let h = &g.player(P0).hand;
    for n in ["Terror of the Peaks", "Rune-Scarred Demon", "Lyra Dawnbringer"] {
        assert!(h.contains(&card(&g, n)), "{n} to hand");
    }
    assert_eq!(h.len(), 3);
    let lib = &g.player(P0).library;
    assert_eq!(lib.len(), 4);
    assert_eq!(*lib.last().unwrap(), deep, "the rest on the bottom");
}

// ------------------------------------------------------------------ Lord of the Void
#[test]
fn lord_of_the_void_exiles_seven_and_takes_a_creature() {
    let mut g = kaalia();
    let lord = perm(&mut g, P0, "Lord of the Void");
    let names = ["Island", "Island", "Lyra Dawnbringer", "Sol Ring", "Island", "Island", "Island", "Swamp"];
    let cs: Vec<CardId> = names.iter().map(|n| take(&mut g, P1, n)).collect();
    g.player_mut(P1).library = cs;
    run_hook(&mut g, lord, Event::CombatDamage, &Call::CombatDamage { p: P0, a: lord, d: P1, dmg: 7 }).unwrap();
    assert_eq!(g.player(P1).library.len(), 1, "seven exiled");
    assert_eq!(g.player(P1).exile.len(), 6);
    let m = on_bf(&g, P0, "Lyra Dawnbringer").expect("the creature card enters under my control");
    assert_eq!(g.perm(m).orig, P1, "still theirs");
}

// ------------------------------------------------------------------ Rakdos, Patron of Chaos
#[test]
fn rakdos_takes_two_cheap_permanents_or_draws_two() {
    let mut g = kaalia();
    let r = perm(&mut g, P0, "Rakdos, Patron of Chaos");
    let n0 = g.player(P0).hand.len();
    run_hook(&mut g, r, Event::EndStep, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n0 + 2, "no permanents to give: I draw two");
    for q in [P1, P2] {
        perm(&mut g, q, "Sol Ring");
        perm(&mut g, q, "Arcane Signet");
    }
    let n0 = g.player(P0).hand.len();
    run_hook(&mut g, r, Event::EndStep, &Call::Player { p: P0 }).unwrap();
    let left = [P1, P2].iter().map(|&q| g.player(q).perms.iter().filter(|&&m| g.perm(m).on_bf).count()).sum::<usize>();
    assert_eq!(left, 2, "the targeted opponent sacrificed its two rocks");
    assert_eq!(g.player(P0).hand.len(), n0, "so I draw nothing");
    run_hook(&mut g, r, Event::EndStep, &Call::Player { p: P1 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n0, "your end step only");
}

// ------------------------------------------------------------------ Serra's Emissary
#[test]
fn serras_emissary_gives_protection_from_creatures() {
    let mut g = kaalia();
    let a = creature(&mut g, P0, "Llanowar Elves");
    let b = creature(&mut g, P1, "Llanowar Elves");
    assert!(can_block(&g, b, a));
    assert!(!t12::emissary_shields(&g, P0));
    perm(&mut g, P0, "Serra's Emissary");
    assert!(!can_block(&g, b, a), "creatures can't block my creatures");
    assert!(can_block(&g, a, b), "theirs can still be blocked");
    assert!(t12::emissary_shields(&g, P0) && !t12::emissary_shields(&g, P1), "I take no combat damage");
}

// ------------------------------------------------------------------ Valgavoth, Terror Eater
#[test]
fn valgavoth_ward_takes_three_nonland_permanents() {
    let mut g = kaalia();
    let v = perm(&mut g, P0, "Valgavoth, Terror Eater");
    let stp = card(&g, "Swords to Plowshares");
    token(&mut g, P1, 1);
    token(&mut g, P1, 1);
    assert!(t12::ward_blocks(&g, P1, v), "two permanents: it can't pay the ward");
    removal::apply_removal(&mut g, Some(P1), v, "exile", Some(stp)).unwrap();
    assert!(g.perm(v).on_bf, "unpaid ward: countered");
    token(&mut g, P1, 1);
    assert!(!t12::ward_blocks(&g, P1, v));
    removal::apply_removal(&mut g, Some(P1), v, "exile", Some(stp)).unwrap();
    assert!(!g.perm(v).on_bf, "three tokens sacrificed: it resolves");
    assert_eq!(g.player(P1).perms.iter().filter(|&&m| g.perm(m).on_bf).count(), 0);
}

#[test]
fn valgavoth_exiles_what_goes_to_their_graveyards_and_casts_it_for_life() {
    let mut g = kaalia();
    let old = to_gy(&mut g, P1, "Lightning Bolt");
    let v = perm(&mut g, P0, "Valgavoth, Terror Eater");
    fire_sba(&mut g).unwrap(); // its first look: what is there stays
    assert!(g.player(P1).gy.contains(&old));
    let elf = creature(&mut g, P1, "Llanowar Elves");
    zones::die(&mut g, elf, "destroy").unwrap();
    let ec = card(&g, "Llanowar Elves");
    assert!(g.player(P1).exile.contains(&ec), "their creature is exiled instead");
    let w = to_gy(&mut g, P2, "Night's Whisper");
    fire_sba(&mut g).unwrap();
    assert!(g.player(P2).exile.contains(&w) && !g.player(P2).gy.contains(&w), "a card put into their graveyard");
    assert!(g.player(P1).gy.contains(&old), "not what was there before");
    let mine = to_gy(&mut g, P0, "Divination");
    fire_sba(&mut g).unwrap();
    assert!(g.player(P0).gy.contains(&mine), "not my graveyard");
    // my turn: cast the best one for life
    let plains = card(&g, "Plains");
    g.player_mut(P0).library.extend([plains; 3]);
    let life = g.player(P0).life;
    let o = opts(&mut g, v, P0, Some(false));
    assert_eq!(o.len(), 1, "{o:?}");
    assert!(run(&mut g, &o[0], P0));
    let spent = life - g.player(P0).life;
    assert!(spent == 1 || spent == 2, "life equal to its mana value: {spent}");
    assert!(opts(&mut g, v, P0, None).is_empty(), "during your turn only");
}

// ------------------------------------------------------------------ Vilis, Broker of Blood
#[test]
fn vilis_draws_for_life_lost_and_shrinks_for_two_life() {
    let mut g = kaalia();
    let v = perm(&mut g, P0, "Vilis, Broker of Blood");
    let plains = card(&g, "Plains");
    g.player_mut(P0).library.extend([plains; 12]);
    let n = g.player(P0).hand.len();
    life::lose_life(&mut g, P0, 3, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 3, "whenever you lose life, draw that many");
    lands(&mut g, P0, "Swamp", 1, false);
    let t = token(&mut g, P1, 1);
    let (life0, n) = (g.player(P0).life, g.player(P0).hand.len());
    let o = opts(&mut g, v, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0], P0));
    assert!(!g.perm(t).on_bf, "-1/-1 kills the 1/1");
    assert_eq!(g.player(P0).life, life0 - 2, "pay 2 life");
    assert_eq!(g.player(P0).hand.len(), n + 2, "and draw two for it");
    assert!(!mana::can_pay(&g, P0, 0, "B", false), "{{B}} paid");
}

// ------------------------------------------------------------------ Strength of the Harvest // Haven of the Harvest
#[test]
fn strength_of_the_harvest_counts_creatures_and_enchantments() {
    let mut g = sythis();
    let m = creature(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Ghostly Prison");
    let (p0, t0) = (epow(&g, m), etgh(&g, m));
    perm(&mut g, P0, "Strength of the Harvest // Haven of the Harvest");
    // the elf, Ghostly Prison and the Aura itself
    assert_eq!((epow(&g, m), etgh(&g, m)), (p0 + 3, t0 + 3));
}

#[test]
fn haven_of_the_harvest_enters_tapped_for_g_or_w() {
    let mut g = sythis();
    let c = hand(&mut g, P0, &["Strength of the Harvest // Haven of the Harvest"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "no other land in hand: the land face");
    assert!(run(&mut g, &o[0], P0));
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.land(l).tapped, "enters tapped");
    let cols = mana::land_colors_now(&g, P0, l);
    assert!(cols.has('G') && cols.has('W'));
}

// ------------------------------------------------------------------ Branchloft Pathway
#[test]
fn branchloft_pathway_is_played_as_the_face_the_hand_needs() {
    let mut g = sythis();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Ghostly Prison"]); // {2}{W}
    let name = "Branchloft Pathway // Boulderloft Pathway";
    let l = lands(&mut g, P0, name, 1, false)[0];
    (g.registry.get(card(&g, name)).unwrap().land_etb.unwrap())(&mut g, P0, l).unwrap();
    let cols = mana::land_colors_now(&g, P0, l);
    assert!(cols.has('W') && !cols.has('G'), "white needed: Boulderloft");
}

// ------------------------------------------------------------------ Calix, Guided by Fate
#[test]
fn calix_counters_on_constellation_and_copies_an_enchantment_once_a_turn() {
    let mut g = sythis();
    let calix = perm(&mut g, P0, "Calix, Guided by Fate");
    assert_eq!(g.perm(calix).plus, 1, "Calix entering is an enchantment entering");
    perm(&mut g, P0, "Ghostly Prison");
    assert_eq!(g.perm(calix).plus, 2, "another enchantment");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(g.perm(calix).plus, 2, "not an enchantment");
    let dmg = Call::CombatDamage { p: P0, a: calix, d: P1, dmg: 3 };
    run_hook(&mut g, calix, Event::CombatDamage, &dmg).unwrap();
    let prisons = |g: &Game| {
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Ghostly Prison").count()
    };
    assert_eq!(prisons(&g), 2, "a token copy of Ghostly Prison");
    run_hook(&mut g, calix, Event::CombatDamage, &dmg).unwrap();
    assert_eq!(prisons(&g), 2, "only once each turn");
}

// ------------------------------------------------------------------ Resurgent Belief
#[test]
fn resurgent_belief_is_suspended_and_returns_every_enchantment() {
    let mut g = sythis();
    lands(&mut g, P0, "Plains", 2, false);
    let c = hand(&mut g, P0, &["Resurgent Belief"])[0];
    assert!(hand_opts(&mut g, c, P0, Some(false)).is_empty(), "nothing to return yet");
    to_gy(&mut g, P0, "Ghostly Prison");
    to_gy(&mut g, P0, "Sphere of Safety");
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0], P0));
    assert!(g.player(P0).exile.contains(&c) && g.player(P0).suspended.contains(&(c, 2)), "suspend 2");
    cardcode::suspend_upkeep(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Ghostly Prison").is_none(), "one time counter left");
    cardcode::suspend_upkeep(&mut g, P0).unwrap();
    assert!(on_bf(&g, P0, "Ghostly Prison").is_some() && on_bf(&g, P0, "Sphere of Safety").is_some(), "all of them");
}

#[test]
fn return_all_land_cards_returns_every_land_tapped() {
    let mut g = sythis();
    for n in ["Forest", "Plains", "Forest"] {
        to_gy(&mut g, P0, n);
    }
    to_gy(&mut g, P0, "Ghostly Prison");
    let before = g.player(P0).lands.len();
    let c = take(&mut g, P0, "Splendid Reclamation");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P0).lands.len(), before + 3, "every land card");
    assert!(g.player(P0).lands[before..].iter().all(|&l| g.land(l).tapped), "tapped");
    assert!(g.player(P0).gy.contains(&card(&g, "Ghostly Prison")), "lands only");
}

// ------------------------------------------------------------------ Weaver of Harmony
#[test]
fn weaver_of_harmony_copies_an_enchantress_draw() {
    let mut g = sythis();
    let w = perm(&mut g, P0, "Weaver of Harmony");
    lands(&mut g, P0, "Forest", 1, false);
    let plains = card(&g, "Plains");
    g.player_mut(P0).library.extend([plains; 5]);
    let prison = card(&g, "Ghostly Prison");
    let n = g.player(P0).hand.len();
    run_hook(&mut g, w, Event::Cast, &Call::Cast { caster: P0, c: prison }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "no enchantment source that draws: nothing to copy");
    perm(&mut g, P0, "Enchantress's Presence");
    run_hook(&mut g, w, Event::Cast, &Call::Cast { caster: P0, c: prison }).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "the copied draw");
    assert!(g.perm(w).tapped, "{{G}}, {{T}}");
}
