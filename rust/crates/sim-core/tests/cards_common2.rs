//! Phase 6: the rest of common.py, partials.py and fixes.py (impls/common.rs, partials.rs, fixes.rs and Karn's
//! Bastion in lands.rs), on hand-built positions: the four bug fixes made in the Rust only (Druid Class's level 2,
//! Mardu Charm's discard mode, Angelic Destiny's return, proliferate's counters), then each card's clause as the
//! Python models it.

use sim_core::ai::act::perform;
use sim_core::cardcode;
use sim_core::engine::{hooks, mana, removal, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{common, partials};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const TATYOVA: &str = "tatyova-simic-landfall";
const ISSHIN: &str = "isshin-mardu-attack-triggers";
const LP: &str = "light-paws-aura-voltron";
const HELIOD: &str = "heliod-mono-white-stax";
const URZA: &str = "urza-mono-blue-artifacts";

/// src's activated-ability options (its card's `options` hook)
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, post).unwrap()
}

fn option_with(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>, word: &str) -> Action {
    options(g, src, p, post).into_iter().find(|o| o.label.contains(word)).and_then(|o| o.act).expect("the option")
}

/// a card in hand's own plays (its `hand_options` hook)
fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, p, post).unwrap()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

// ================================================================== the four bug fixes
#[test]
fn fix_druid_class_levels_up_from_one_to_three() {
    // Python's partials hook replaced t1's level-2 option and only offered 2 -> 3: Druid Class stayed at level 1
    let mut g = table(&[TATYOVA, "seph"]);
    lands(&mut g, P0, "Forest", 8, false);
    let d = perm(&mut g, P0, "Druid Class");
    assert_eq!(cardcode::extra_lands(&g, P0), 0);
    assert!(options(&mut g, d, P0, Some(false)).iter().all(|o| !o.label.contains("level 3")));
    let a = option_with(&mut g, d, P0, Some(false), "level 2");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.perm(d).data.get(DataKey::Level), Some(&Val::Int(2)));
    assert_eq!(cardcode::extra_lands(&g, P0), 1); // level 2: an extra land each turn
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3); // {2}{G}
    // level 2 offers level 3 (seven lands, {4}{G} left)
    let opts = options(&mut g, d, P0, Some(false));
    assert!(opts.iter().all(|o| !o.label.contains("level 2")));
    assert!(opts.iter().any(|o| o.label.contains("level 3")));
}

#[test]
fn fix_mardu_charm_chooses_its_discard_mode() {
    // Python's resolve read ctx['mode'], which nothing set: the discard mode never happened
    let mut g = table(&[ISSHIN, "seph"]);
    for l in ["Mountain", "Plains", "Swamp"] {
        lands(&mut g, P0, l, 1, false);
    }
    let c = hand(&mut g, P0, &["Mardu Charm"])[0];
    hand(&mut g, P1, &["Counterspell", "Grave Titan", "Island", "Demonic Tutor", "Swamp"]);
    let opts = hand_opts(&mut g, c, P0, None);
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.contains("discard"), "{}", opts[0].label);
    assert!(opts[0].utility > 1.2);
    assert!(perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    // the best noncreature, nonland card goes; no Warriors
    assert_eq!(g.player(P1).hand.len(), 4);
    assert!(in_gy(&g, P1, "Counterspell") || in_gy(&g, P1, "Demonic Tutor"));
    assert!(in_hand(&g, P1, "Grave Titan") && in_hand(&g, P1, "Island"));
    assert!(named(&g, P0, "Warrior").is_empty() && g.player(P0).perms.iter().all(|&m| !g.perm(m).token));
    assert!(in_gy(&g, P0, "Mardu Charm"));
}

#[test]
fn mardu_charm_makes_warriors_against_a_small_hand() {
    let mut g = table(&[ISSHIN, "seph"]);
    for l in ["Mountain", "Plains", "Swamp"] {
        lands(&mut g, P0, l, 1, false);
    }
    let c = hand(&mut g, P0, &["Mardu Charm"])[0];
    hand(&mut g, P1, &["Counterspell", "Island"]);
    let opts = hand_opts(&mut g, c, P0, None);
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.contains("Warriors"));
    assert!(perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 2);
    assert_eq!(g.player(P1).hand.len(), 2);
}

#[test]
fn fix_angelic_destiny_returns_only_when_its_creature_dies() {
    // Python's died check ran after the host had left, so the Aura came back however it left
    let mut g = table(&[LP, "seph"]);
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Angelic Destiny");
    removal::apply_removal(&mut g, Some(P1), host, "bounce", None).unwrap();
    assert!(in_hand(&g, P0, "Esper Sentinel"));
    assert!(!in_hand(&g, P0, "Angelic Destiny"));
    assert!(in_gy(&g, P0, "Angelic Destiny"));
    // exiled: to the graveyard too
    let host = perm(&mut g, P0, "Mother of Runes");
    let a = take(&mut g, P0, "Angelic Destiny");
    g.player_mut(P0).gy.retain(|&c| c != a);
    zones::enter(&mut g, P0, a, zones::Enter::default()).unwrap();
    removal::apply_removal(&mut g, Some(P1), host, "exile", None).unwrap();
    assert!(in_gy(&g, P0, "Angelic Destiny"));
    assert!(!in_hand(&g, P0, "Angelic Destiny"));
}

#[test]
fn angelic_destiny_returns_when_its_creature_dies_under_hushbringer() {
    let mut g = table(&[LP, "seph"]);
    perm(&mut g, P1, "Hushbringer"); // creatures dying trigger nothing, but they still die
    let host = perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P0, "Angelic Destiny");
    zones::die(&mut g, host, "destroy").unwrap();
    assert!(in_hand(&g, P0, "Angelic Destiny"));
    assert!(!in_gy(&g, P0, "Angelic Destiny"));
}

#[test]
fn fix_proliferate_adds_the_counters_the_engine_keeps() {
    // Python's proliferate added to m.data['counters'], which nothing writes: poison, charge counters, the Incubator's
    // and The Ozolith's counters were never proliferated
    let mut g = table(&["atraxa-superfriends", "seph"]);
    let m = token(&mut g, P0, 2);
    g.perm_mut(m).plus = 1;
    let w = perm(&mut g, P0, "Narset, Parter of Veils");
    g.perm_mut(w).loyalty = Some(5);
    let v = lands(&mut g, P0, "Vivid Creek", 1, false)[0]; // two charge counters as it enters
    g.player_mut(P0).incubator = 2;
    g.player_mut(P1).poison = 3;
    let o = token(&mut g, P1, 3);
    g.perm_mut(o).plus = -1;
    common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(m).plus, 2);
    assert_eq!(g.perm(w).loyalty, Some(6));
    assert_eq!(g.land(v).data.get(DataKey::Charge), Some(&Val::Int(3)));
    assert_eq!(g.player(P0).incubator, 3);
    assert_eq!(g.player(P1).poison, 4);
    assert_eq!(g.perm(o).plus, -2);
    // no counters, nothing added: your poison and a player with none stay
    assert_eq!(g.player(P0).poison, 0);
}

#[test]
fn proliferate_poison_can_finish_a_player() {
    let mut g = table(&["atraxa-superfriends", "seph"]);
    g.player_mut(P1).poison = 9;
    common::proliferate(&mut g, P0, 1).unwrap();
    assert!(!g.player(P1).alive);
}

// ================================================================== common.py
#[test]
fn pillowfort_taxes_and_caps() {
    let mut g = table(&["sythis-selesnya-enchantress", "seph"]);
    let gp = perm(&mut g, P0, "Ghostly Prison");
    let tax = g.registry.get(card(&g, "Ghostly Prison")).unwrap().attack_tax.unwrap();
    assert_eq!(tax(&g, gp, P1, P0), Some(2));
    assert_eq!(tax(&g, gp, P1, PlayerId(2)), Some(0)); // only attacks at its controller
    let s = perm(&mut g, P0, "Sphere of Safety");
    perm(&mut g, P0, "Oblivion Ring");
    let tax = g.registry.get(card(&g, "Sphere of Safety")).unwrap().attack_tax.unwrap();
    assert_eq!(tax(&g, s, P1, P0), Some(3)); // three enchantments
    let c = perm(&mut g, P0, "Crawlspace");
    let cap = g.registry.get(card(&g, "Crawlspace")).unwrap().attack_cap.unwrap();
    assert_eq!(cap(&g, c, P1, P0), Some(2));
    assert_eq!(cap(&g, c, P1, PlayerId(2)), None);
}

#[test]
fn stax_cost_taxes_and_spell_limits() {
    let mut g = table(&[HELIOD, "seph"]);
    let bolt = card(&g, "Counterspell");
    let titan = card(&g, "Grave Titan");
    perm(&mut g, P0, "Thalia, Guardian of Thraben");
    assert_eq!(hooks::total_cost(&g, P1, bolt), 1);
    assert_eq!(hooks::total_cost(&g, P0, bolt), 1); // its controller's too
    assert_eq!(hooks::total_cost(&g, P1, titan), 0);
    perm(&mut g, P0, "Rule of Law");
    assert!(hooks::allowed(&g, P1, titan, "hand"));
    let st = g.turn_stamp();
    g.player_mut(P1).turn_casts = Some((st, vec![bolt]));
    assert!(!hooks::allowed(&g, P1, titan, "hand")); // one spell a turn
    perm(&mut g, P0, "Drannith Magistrate");
    g.player_mut(P1).turn_casts = None;
    assert!(!hooks::allowed(&g, P1, titan, "cmd"));
    assert!(hooks::allowed(&g, P1, titan, "hand"));
    assert!(hooks::allowed(&g, P0, titan, "cmd"));
}

#[test]
fn trinisphere_and_the_grand_arbiter() {
    let mut g = table(&[URZA, "seph"]);
    let sol = card(&g, "Sol Ring");
    let (gn0, _) = mana::cost_of(&g, P1, sol);
    perm(&mut g, P0, "Trinisphere");
    let (gn, pips) = mana::cost_of(&g, P1, sol);
    assert_eq!(gn as usize + pips.len(), 3);
    assert!(gn > gn0);
}

#[test]
fn walking_ballista_gets_half_of_x_and_pings_for_lethal() {
    let mut g = table(&[URZA, "seph"]);
    // as in the Python, a 0/0 entering dies to the toughness check before its enter hook runs (a Python bug, ported)
    g.last_x = 4;
    let b = perm(&mut g, P0, "Walking Ballista");
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.last_x, 4);
    // entering with a counter, it survives and gets X/2 more
    let c = take(&mut g, P0, "Walking Ballista");
    g.player_mut(P0).gy.clear();
    let b = zones::enter(&mut g, P0, c, zones::Enter { plus: 1, sick: false, ..zones::Enter::default() }).unwrap();
    assert_eq!(g.perm(b).plus, 3);
    assert_eq!(g.last_x, 0);
    g.perm_mut(b).plus = 2;
    g.player_mut(P1).life = 2;
    let opts = options(&mut g, b, P0, Some(false));
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].utility, 8.0);
    assert!(perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P1).life, 1);
    assert_eq!(g.perm(b).plus, 1);
}

#[test]
fn magus_of_the_moon_is_a_blood_moon() {
    let mut g = table(&["winota-boros-humans-cheat", "seph"]);
    assert!(!cardcode::blood_moon(&g));
    perm(&mut g, P0, "Magus of the Moon");
    assert!(cardcode::blood_moon(&g));
    assert!(common::blood_moon_active(&g));
}

#[test]
fn elspeth_makes_soldiers() {
    let mut g = table(&["brago-azorius-blink-control", "seph"]);
    let e = perm(&mut g, P0, "Elspeth, Sun's Champion");
    let a = option_with(&mut g, e, P0, Some(false), "+1");
    let l0 = g.perm(e).loyalty.unwrap();
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.perm(e).loyalty, Some(l0 + 1));
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 3);
    // once a turn
    assert!(options(&mut g, e, P0, Some(false)).is_empty());
}

#[test]
fn narset_digs_a_noncreature_card() {
    let mut g = table(&["atraxa-superfriends", "seph"]);
    let n = perm(&mut g, P0, "Narset, Parter of Veils");
    let lib = &mut g.player_mut(P0).library;
    lib.clear();
    let cs: Vec<CardId> =
        ["Forest", "Grave Titan", "Island", "Counterspell", "Plains"].iter().map(|x| card(&g, x)).collect();
    g.player_mut(P0).library = cs;
    let a = option_with(&mut g, n, P0, Some(false), "-2");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert!(in_hand(&g, P0, "Counterspell"));
    assert_eq!(g.player(P0).library.len(), 4);
    assert_eq!(*g.player(P0).library.last().unwrap(), card(&g, "Forest")); // the rest went to the bottom
}

#[test]
fn spark_double_copies_the_best_creature_with_a_counter() {
    let mut g = table(&["brago-azorius-blink-control", "seph"]);
    perm(&mut g, P0, "Grave Titan");
    // as in the Python, the 0/0 dies to the toughness check before its enter hook copies (a Python bug, ported)
    let s = perm(&mut g, P0, "Spark Double");
    assert!(!g.perm(s).on_bf);
    assert!(in_gy(&g, P0, "Spark Double"));
    assert_eq!(named(&g, P0, "Grave Titan").len(), 1);
    // entering with a counter, it survives and copies
    let c = take(&mut g, P0, "Spark Double");
    g.player_mut(P0).gy.clear();
    zones::enter(&mut g, P0, c, zones::Enter { plus: 1, ..zones::Enter::default() }).unwrap();
    let copies = named(&g, P0, "Grave Titan");
    assert_eq!(copies.len(), 2);
    let c = copies[1];
    assert_eq!(g.perm(c).plus, 1);
    assert_eq!(g.perm(c).phys, Some(card(&g, "Spark Double")));
    assert!(g.perm(c).data.truthy(DataKey::Copied));
}

#[test]
fn reflector_mage_bounces_and_locks_the_name() {
    let mut g = table(&["brago-azorius-blink-control", "seph"]);
    let t = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Reflector Mage");
    assert!(!g.perm(t).on_bf);
    let titan = card(&g, "Grave Titan");
    assert!(!hooks::allowed(&g, P1, titan, "hand"));
    g.player_mut(P0).turns += 1; // the Mage's controller's next turn
    assert!(hooks::allowed(&g, P1, titan, "hand"));
}

#[test]
fn tormods_crypt_answers_a_reanimation() {
    let mut g = table(&["seph", URZA]);
    perm(&mut g, P1, "Tormod's Crypt");
    let titan = card(&g, "Grave Titan");
    g.player_mut(P0).gy.push(titan);
    assert!(!cardcode::gy_response(&mut g, P0, 3.0, P0).unwrap()); // not worth it
    assert!(cardcode::gy_response(&mut g, P0, 8.0, P0).unwrap());
    assert!(g.player(P0).gy.is_empty());
    assert!(g.player(P0).exile.contains(&titan));
    assert!(in_gy(&g, P1, "Tormod's Crypt"));
}

#[test]
fn bojuka_bog_exiles_the_worst_graveyard() {
    let mut g = table(&["meren-golgari-recursion", "seph", URZA]);
    let titan = card(&g, "Grave Titan");
    g.player_mut(P1).gy.push(titan); // 6 power: worth exiling
    let sol = card(&g, "Sol Ring");
    g.player_mut(PlayerId(2)).gy.push(sol);
    let bog = lands(&mut g, P0, "Bojuka Bog", 1, true)[0];
    cardcode::land_etb(&mut g, P0, bog).unwrap();
    assert!(g.player(P1).gy.is_empty());
    assert_eq!(g.player(PlayerId(2)).gy, [sol]);
    assert!(common::gy_worth(&g, P0, P0) < 0.0);
}

#[test]
fn static_net_makes_a_tapped_powerstone() {
    let mut g = table(&["yshtola", "seph"]);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Static Net");
    assert_eq!(g.player(P0).life, 42);
    let ps = named(&g, P0, "Powerstone");
    assert_eq!(ps.len(), 1);
    assert!(g.perm(ps[0]).token && g.perm(ps[0]).tapped);
}

#[test]
fn urza_s_saga_makes_a_construct_and_fetches_an_artifact() {
    let mut g = table(&[URZA, "seph"]);
    lands(&mut g, P0, "Island", 3, false);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    let saga = lands(&mut g, P0, "Urza's Saga", 1, false)[0];
    cardcode::land_etb(&mut g, P0, saga).unwrap();
    assert_eq!(g.player(P0).sagas, [(saga, 1)]);
    common::saga_step(&mut g, P0).unwrap(); // lore 2: a Construct ({2} from the rocks)
    assert_eq!(g.player(P0).sagas, [(saga, 2)]);
    assert!(g.land(saga).tapped);
    g.land_mut(saga).tapped = false;
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).tapped = false;
    }
    let before: Vec<PermId> = g.player(P0).perms.clone();
    common::saga_step(&mut g, P0).unwrap(); // lore 3: a 0/1-cost artifact, then it's sacrificed
    assert!(g.player(P0).sagas.is_empty());
    assert!(!g.player(P0).lands.contains(&saga));
    assert!(in_gy(&g, P0, "Urza's Saga"));
    let fetched: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|m| !before.contains(m) && g.perm(*m).cd.is_some_and(|c| g.db.get(c).cmc <= 1))
        .collect();
    assert_eq!(fetched.len(), 1);
    assert!(g.db.get(g.perm(fetched[0]).cd.unwrap()).types.has(sim_core::cards::Types::ARTIFACT));
}

#[test]
fn karns_bastion_proliferates() {
    let mut g = table(&["atraxa-superfriends", "seph"]);
    lands(&mut g, P0, "Plains", 4, false);
    lands(&mut g, P0, "Karn's Bastion", 1, false);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    g.perm_mut(a).plus = 1;
    g.perm_mut(b).plus = 1;
    let opts = cardcode::land_options(&mut g, P0, None).unwrap();
    let o = opts.into_iter().find(|o| o.label == "Karn's Bastion").expect("Karn's Bastion");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (2, 2));
}

#[test]
fn draco_costs_less_per_basic_type_and_delve() {
    let mut g = table(&["yuriko-dimir-ninjas", "seph"]);
    let draco = card(&g, "Draco");
    let (gn0, _) = mana::cost_of(&g, P0, draco);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Swamp", 2, false);
    assert_eq!(partials::domain(&g, P0), 2);
    let (gn, _) = mana::cost_of(&g, P0, draco);
    assert_eq!(gn, gn0 - 4);
    let cruise = card(&g, "Treasure Cruise");
    let junk: Vec<CardId> = ["Island", "Swamp", "Plains"].iter().map(|x| card(&g, x)).collect();
    g.player_mut(P0).gy.extend(junk);
    assert_eq!(partials::delve_count(&g, P0, cruise), 3);
    let (gn, _) = mana::cost_of(&g, P0, cruise);
    assert_eq!(gn, g.db.get(cruise).generic - 3);
}

#[test]
fn coat_of_arms_and_lord_of_lineage() {
    let mut g = table(&["krenko-mono-red-goblins", "tergrid-mono-black-disruption"]);
    let a = perm(&mut g, P0, "Goblin Matron");
    let b = perm(&mut g, P0, "Mogg Fanatic");
    let (p0, t0) = (values::epow(&g, a), values::etgh(&g, a));
    perm(&mut g, P0, "Coat of Arms");
    // Goblin Matron and Mogg Fanatic share Goblin: +1/+1 each
    assert_eq!((values::epow(&g, a), values::etgh(&g, a)), (p0 + 1, t0 + 1));
    assert_eq!(partials::coat_bonus(&g, b), 1);
    let k = perm(&mut g, P1, "Bloodline Keeper // Lord of Lineage");
    for _ in 0..4 {
        let o = option_with(&mut g, k, P1, Some(false), "token");
        g.perm_mut(k).tapped = false;
        assert!(perform(&mut g, P1, &o).unwrap());
        g.perm_mut(k).tapped = false;
    }
    lands(&mut g, P1, "Swamp", 1, false);
    let v = *g.player(P1).perms.iter().find(|&&m| g.perm(m).token).unwrap();
    assert_eq!(partials::lineage_bonus(&g, v), 0);
    let o = option_with(&mut g, k, P1, Some(false), "transform");
    assert!(perform(&mut g, P1, &o).unwrap());
    assert_eq!(partials::lineage_bonus(&g, v), 2); // other Vampires +2/+2
    assert_eq!(partials::lineage_bonus(&g, k), 0);
}

#[test]
fn golgari_charm_regenerates_against_a_wipe() {
    let mut g = table(&["meren-golgari-recursion", "seph"]);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Golgari Charm"]);
    perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P0, "Sheoldred, Whispering One");
    assert!(cardcode::regen_wipe(&mut g, P0).unwrap());
    assert_eq!(g.player(P0).regen_turn, Some(g.turn_stamp()));
    assert!(in_gy(&g, P0, "Golgari Charm"));
    let t = named(&g, P0, "Grave Titan")[0];
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(g.perm(t).on_bf); // regenerated
}

#[test]
fn hunters_insight_draws_for_the_damage() {
    let mut g = table(&["lathril-golgari-elves", "seph"]);
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Hunter's Insight"])[0];
    let t = perm(&mut g, P0, "Grave Titan");
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).insight, Some((g.turn_stamp(), t)));
    let n = g.player(P0).hand.len();
    partials::insight_draw(&mut g, P0, t, 6).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 6);
}

#[test]
fn quirion_ranger_untaps_a_mana_creature() {
    let mut g = table(&["marwyn-mono-green-elves", "seph"]);
    lands(&mut g, P0, "Forest", 1, true);
    g.player_mut(P0).land_turn = -1;
    perm(&mut g, P0, "Quirion Ranger");
    let pt = perm(&mut g, P0, "Priest of Titania");
    perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(pt).tapped = true;
    let units = mana::mana_units(&g, P0, false);
    let u = units.iter().find(|u| matches!(u.src, mana::Source::Quirion(..))).expect("a Quirion unit");
    assert!(u.amt >= 2);
    mana::spend_unit(&mut g, P0, *u, u.amt, "").unwrap();
    assert!(g.player(P0).lands.is_empty());
    assert!(in_hand(&g, P0, "Forest"));
    // once a turn
    assert!(!mana::mana_units(&g, P0, false).iter().any(|u| matches!(u.src, mana::Source::Quirion(..))));
}

#[test]
fn the_one_ring_draws_more_each_time() {
    let mut g = table(&[URZA, "seph"]);
    let r = perm(&mut g, P0, "The One Ring");
    let n = g.player(P0).hand.len();
    let a = option_with(&mut g, r, P0, Some(false), "The One Ring");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.player(P0).hand.len(), n + 1);
    g.perm_mut(r).tapped = false;
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.player(P0).hand.len(), n + 3);
    assert_eq!(g.perm(r).data.get(DataKey::Burden), Some(&Val::Int(2)));
    let up = g.registry.get(card(&g, "The One Ring")).unwrap().upkeep.unwrap();
    up(&mut g, r, P0).unwrap();
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn acidic_slime_and_decimate() {
    let mut g = table(&["meren-golgari-recursion", "seph"]);
    let sol = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, "Acidic Slime");
    assert!(!g.perm(sol).on_bf);
    // Decimate needs all four targets
    let dec = card(&g, "Decimate");
    let prio = g.registry.get(dec).unwrap().prio.unwrap();
    assert_eq!(prio(&g, P0, dec), 0);
}

#[test]
fn mana_rocks_that_enter_tapped() {
    let mut g = table(&["tergrid-mono-black-disruption", "seph"]);
    let d = perm(&mut g, P0, "Charcoal Diamond");
    assert!(g.perm(d).tapped);
}

#[test]
fn rishkar_puts_counters_and_makes_mana() {
    let mut g = table(&["marwyn-mono-green-elves", "seph"]);
    let a = perm(&mut g, P0, "Grave Titan");
    let b = perm(&mut g, P0, "Elvish Visionary");
    let r = perm(&mut g, P0, "Rishkar, Peema Renegade");
    // as the Python sorts them, (it is Rishkar, -value) from the top: Rishkar, then the least valuable creature (the
    // first of Grave Titan's two Zombies), not the best two as the card intends
    let zombie = *g.player(P0).perms.iter().find(|&&m| g.perm(m).token).unwrap();
    assert_eq!(g.perm(r).plus, 1);
    assert_eq!(g.perm(zombie).plus, 1);
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 0));
    g.perm_mut(zombie).sick = false;
    let units = mana::mana_units(&g, P0, false);
    assert!(units.iter().any(|u| u.src == mana::Source::Perm(zombie)));
    assert!(!units.iter().any(|u| u.src == mana::Source::Perm(b)));
}
