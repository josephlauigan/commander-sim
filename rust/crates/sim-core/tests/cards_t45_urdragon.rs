//! The Tier 5 ur-dragon-five-color-dragons cards (impls/t45_urdragon.rs, the compiled "Dragon spells cost less" and
//! "Dragons you control have" lines, and the hooks marked `// --- Tiers 4/5 (urdragon)`) on hand-built positions. Each
//! expectation follows the card's Oracle text.

use sim_core::engine::mana::cost_of;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t45_urdragon as u;
use sim_core::state::{DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&[
        "ur-dragon-five-color-dragons",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ]);
    g.player_mut(P0).hand.clear();
    g
}

/// a table whose first deck is mostly Dragons (the Ur-Dragon list retired 2026-10-25: Miirym's now), for the cards that
/// name the deck's most common creature type
fn tbl_dragons() -> Game {
    let mut g = table(&[
        "miirym-temur-dragons",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ]);
    g.player_mut(P0).hand.clear();
    g
}

/// the Ur-Dragon out of the command zone (its eminence would change every cost)
fn no_eminence(g: &mut Game) {
    let c = card(g, u::UR_DRAGON);
    g.player_mut(P0).set_cmd_zone(c, false);
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn imp(g: &Game, name: &str) -> sim_core::hooks::CardImpl {
    *g.registry.get(card(g, name)).unwrap()
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

fn on(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

/// set p's library, top card last
fn library(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs.clone();
    cs
}

// ------------------------------------------------------------------ cost reductions
#[test]
fn dragon_spells_cost_less_with_the_compiled_reducers() {
    let mut g = tbl();
    no_eminence(&mut g);
    let gold = card(&g, "Goldspan Dragon"); // {3}{R}{R}
    let sol = card(&g, "Sol Ring");
    assert_eq!(cost_of(&g, P0, gold).0, 3);
    perm(&mut g, P0, "Dragonlord's Servant");
    assert_eq!(cost_of(&g, P0, gold).0, 2, "Dragon spells cost {{1}} less");
    perm(&mut g, P0, "Dragonspeaker Shaman");
    assert_eq!(cost_of(&g, P0, gold).0, 0, "and {{2}} less more: only the generic part");
    assert_eq!(cost_of(&g, P0, gold).1, "RR");
    assert_eq!(cost_of(&g, P0, sol).0, 1, "not a Dragon");
    assert_eq!(cost_of(&g, P1, gold).0, 3, "only your spells");
    perm(&mut g, P0, "Sarkhan, Soul Aflame");
    let ur = card(&g, "Atarka, World Render"); // {5}{R}{R}{G}
    assert_eq!(cost_of(&g, P0, ur).0, 1, "Servant, Shaman and Sarkhan: {{4}} less");
}

#[test]
fn the_ur_dragon_eminence_works_from_the_command_zone_and_the_battlefield() {
    let mut g = tbl();
    let gold = card(&g, "Goldspan Dragon");
    let ur = card(&g, u::UR_DRAGON);
    assert!(g.player(P0).cmd_zone_of(ur));
    assert_eq!(cost_of(&g, P0, gold).0, 2, "in the command zone: other Dragon spells cost {{1}} less");
    assert_eq!(cost_of(&g, P0, card(&g, "Sol Ring")).0, 1);
    assert_eq!(cost_of(&g, P1, gold).0, 3, "not an opponent's");
    assert_eq!(cost_of(&g, P0, ur).0, 4, "not The Ur-Dragon itself");
    no_eminence(&mut g);
    assert_eq!(cost_of(&g, P0, gold).0, 3);
    perm(&mut g, P0, u::UR_DRAGON);
    assert_eq!(cost_of(&g, P0, gold).0, 2, "on the battlefield too");
}

#[test]
fn dracogenesis_casts_dragons_without_mana() {
    let mut g = tbl();
    no_eminence(&mut g);
    perm(&mut g, P0, "Dracogenesis");
    assert_eq!(cost_of(&g, P0, card(&g, "Goldspan Dragon")), (0, String::new()));
    assert_eq!(cost_of(&g, P0, card(&g, "Sol Ring")).0, 1, "only Dragon spells");
    assert_eq!(cost_of(&g, P1, card(&g, "Goldspan Dragon")).0, 3);
}

#[test]
fn urzas_incubator_and_morophon_reduce_the_chosen_type() {
    let mut g = tbl_dragons();
    no_eminence(&mut g);
    let gold = card(&g, "Goldspan Dragon");
    let inc = perm(&mut g, P0, "Urza's Incubator");
    assert_eq!(g.perm(inc).data.get(DataKey::Ctype), Some(&Val::Str("dragon")), "the deck's most common type");
    assert_eq!(cost_of(&g, P0, gold).0, 1, "creature spells of the chosen type cost {{2}} less");
    assert_eq!(cost_of(&g, P0, card(&g, "Birds of Paradise")), (0, "G".into()));
    let mor = perm(&mut g, P0, "Morophon, the Boundless");
    assert_eq!(g.perm(mor).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
    assert_eq!(cost_of(&g, P0, gold), (1, "R".into()), "{{W}}{{U}}{{B}}{{R}}{{G}} less: one of the two red pips");
    let ur = card(&g, u::UR_DRAGON);
    assert_eq!(cost_of(&g, P0, ur), (2, String::new()), "five colours off, {{2}} off the Incubator, tax 0");
    // other creatures of the chosen type get +1/+1
    let d = perm(&mut g, P0, "Goldspan Dragon");
    let b = perm(&mut g, P0, "Birds of Paradise");
    assert_eq!(sim_core::engine::values::epow(&g, d), 5);
    assert_eq!(sim_core::engine::values::epow(&g, b), 0);
    assert_eq!(sim_core::engine::values::epow(&g, mor), 6, "not Morophon itself");
}

// ------------------------------------------------------------------ The Ur-Dragon's attack
#[test]
fn the_ur_dragon_draws_per_attacking_dragon_and_puts_a_permanent_in() {
    let mut g = tbl();
    no_eminence(&mut g);
    g.step = Step::Combat;
    let ur = perm(&mut g, P0, u::UR_DRAGON);
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    library(&mut g, P0, &["Plains", "Island", "Forest"]);
    hand(&mut g, P0, &["Ancient Copper Dragon"]);
    let f = imp(&g, u::UR_DRAGON).attack.unwrap();
    f(&mut g, ur, P0, &[ur, gold, bird], P1).unwrap();
    let h: Vec<&str> = g.player(P0).hand.iter().map(|&c| &*g.db.get(c).name).collect();
    assert_eq!(h, vec!["Forest", "Island"], "two Dragons attacked: draw two");
    assert!(on(&g, P0, "Ancient Copper Dragon").is_some(), "the permanent card goes onto the battlefield");
    // no Dragon attacking: nothing
    f(&mut g, ur, P0, &[bird], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
}

// ------------------------------------------------------------------ the Ancient Dragons
#[test]
fn ancient_copper_gold_and_silver_roll_a_d20() {
    let mut g = tbl();
    g.step = Step::Combat;
    let copper = perm(&mut g, P0, "Ancient Copper Dragon");
    let t0 = g.player(P0).live_treasures();
    (imp(&g, "Ancient Copper Dragon").combat_damage.unwrap())(&mut g, copper, P0, copper, P1, 6).unwrap();
    let n = g.player(P0).live_treasures() - t0;
    assert!((1..=20).contains(&n), "a d20's worth of Treasures: {n}");

    let gold = perm(&mut g, P0, "Ancient Gold Dragon");
    let k0 = tokens(&g, P0).len();
    (imp(&g, "Ancient Gold Dragon").combat_damage.unwrap())(&mut g, gold, P0, gold, P1, 7).unwrap();
    let new: Vec<PermId> = tokens(&g, P0).into_iter().skip(k0).collect();
    assert!((1..=20).contains(&new.len()));
    assert!(new.iter().all(|&t| g.perm(t).fly && sim_core::engine::values::has_type(&g, t, "dragon")));
    assert!(new.iter().all(|&t| sim_core::engine::values::colors_of(&g, t).has('U')), "blue Faerie Dragons");

    let silver = perm(&mut g, P0, "Ancient Silver Dragon");
    library(&mut g, P0, &["Island"; 30]);
    (imp(&g, "Ancient Silver Dragon").combat_damage.unwrap())(&mut g, silver, P0, silver, P1, 8).unwrap();
    assert!((1..=20).contains(&g.player(P0).hand.len()), "draw a d20's worth");
    assert!(g.player(P0).t0s.nomax, "no maximum hand size");
    // another creature's damage does nothing
    let h = g.player(P0).hand.len();
    (imp(&g, "Ancient Silver Dragon").combat_damage.unwrap())(&mut g, silver, P0, copper, P1, 6).unwrap();
    assert_eq!(g.player(P0).hand.len(), h);
}

#[test]
fn ancient_brass_dragon_reanimates_from_any_graveyard() {
    let mut g = tbl();
    g.step = Step::Combat;
    let brass = perm(&mut g, P0, "Ancient Brass Dragon");
    let bird = card(&g, "Birds of Paradise");
    g.player_mut(P1).gy.push(bird);
    let picks = u::brass_pick(&g, P0, 1);
    assert_eq!(picks, vec![(bird, P1)], "mana value 1 fits a roll of 1");
    assert!(u::brass_pick(&g, P0, 0).is_empty());
    (imp(&g, "Ancient Brass Dragon").combat_damage.unwrap())(&mut g, brass, P0, brass, P1, 7).unwrap();
    let b = on(&g, P0, "Birds of Paradise").expect("under your control");
    assert_eq!(g.perm(b).orig, P1, "still the opponent's card");
    assert!(!g.player(P1).gy.contains(&bird));
}

#[test]
fn brass_picks_the_most_worth_within_the_roll() {
    let mut g = tbl();
    let big = card(&g, "Atarka, World Render"); // 8
    let mid = card(&g, "Goldspan Dragon"); // 5
    let small = card(&g, "Birds of Paradise"); // 1
    g.player_mut(P1).gy.extend([big, mid, small]);
    let mut got: Vec<CardId> = u::brass_pick(&g, P0, 9).into_iter().map(|x| x.0).collect();
    got.sort();
    let mut want = vec![big, small];
    want.sort();
    assert_eq!(got, want, "8 + 1 is the most that fits 9");
    assert_eq!(u::brass_pick(&g, P0, 6).len(), 2, "Goldspan and Birds");
}

// ------------------------------------------------------------------ Call the Spirit Dragons
#[test]
fn call_the_spirit_dragons_wins_with_five_dragons_of_different_colours() {
    let mut g = tbl();
    let call = perm(&mut g, P0, "Call the Spirit Dragons");
    let ds: Vec<PermId> = ["Ancient Gold Dragon", "Ancient Silver Dragon", "Ancient Brass Dragon", "Goldspan Dragon"]
        .iter()
        .map(|n| perm(&mut g, P0, n))
        .collect();
    assert!(sim_core::engine::values::indestructible(&g, ds[0]), "Dragons you control have indestructible");
    let f = imp(&g, "Call the Spirit Dragons").upkeep.unwrap();
    f(&mut g, call, P0).unwrap();
    assert!(ds.iter().all(|&d| g.perm(d).plus == 1), "one counter each on four Dragons (no green one)");
    assert!(g.player(P1).alive, "four Dragons: no win");
    let gnaw = perm(&mut g, P0, "Old Gnawbone");
    f(&mut g, call, P0).unwrap();
    assert_eq!(g.perm(gnaw).plus, 1);
    assert!(!g.player(P1).alive && g.player(P0).alive, "five Dragons got counters: you win");
}

#[test]
fn call_assign_spreads_counters_over_different_dragons() {
    let mut g = tbl();
    // Atarka is red-green, Goldspan red: green must go to Atarka for both to get one
    let atarka = perm(&mut g, P0, "Atarka, World Render");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let a = u::call_assign(&g, P0);
    assert_eq!(a.len(), 2, "red and green");
    assert!(a.contains(&('G', atarka)) && a.contains(&('R', gold)));
}

// ------------------------------------------------------------------ Dragon's Hoard
#[test]
fn dragons_hoard_counts_dragons_and_draws() {
    let mut g = tbl();
    let hoard = perm(&mut g, P0, "Dragon's Hoard");
    perm(&mut g, P0, "Goldspan Dragon");
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P1, "Goldspan Dragon");
    assert_eq!(g.perm(hoard).data.get(DataKey::T45Dragon), Some(&Val::Int(1)), "only your Dragons");
    library(&mut g, P0, &["Island", "Island"]);
    let f = imp(&g, "Dragon's Hoard").options.unwrap();
    assert!(f(&mut g, hoard, P0, Some(false)).unwrap().is_empty(), "drawn at the end-of-turn window");
    let o = f(&mut g, hoard, P0, None).unwrap().pop().unwrap();
    assert!(run(&mut g, o, P0));
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(g.perm(hoard).tapped);
    assert_eq!(g.perm(hoard).data.get(DataKey::T45Dragon), Some(&Val::Int(0)));
    g.perm_mut(hoard).tapped = false;
    assert!(f(&mut g, hoard, P0, None).unwrap().is_empty(), "no gold counter left");
}

// ------------------------------------------------------------------ Lathliss, Miirym
#[test]
fn lathliss_makes_a_dragon_when_another_nontoken_dragon_enters() {
    let mut g = tbl();
    perm(&mut g, P0, "Lathliss, Dragon Queen");
    assert!(tokens(&g, P0).is_empty(), "no token for Lathliss herself");
    perm(&mut g, P0, "Goldspan Dragon");
    let t = tokens(&g, P0);
    assert_eq!(t.len(), 1);
    assert_eq!(sim_core::engine::values::epow(&g, t[0]), 5);
    assert!(g.perm(t[0]).fly && sim_core::engine::values::has_type(&g, t[0], "dragon"));
    perm(&mut g, P0, "Birds of Paradise");
    assert_eq!(tokens(&g, P0).len(), 1, "not for a non-Dragon (nor for the 5/5 token itself)");
}

#[test]
fn miirym_copies_another_nontoken_dragon() {
    let mut g = tbl();
    perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    perm(&mut g, P0, "Klauth, Unrivaled Ancient");
    let t = tokens(&g, P0);
    assert_eq!(t.len(), 1, "one token copy (the copy is a token: no copy of it)");
    assert_eq!(&*g.perm(t[0]).name, "Klauth, Unrivaled Ancient");
    assert!(g.perm(t[0]).exc.not_legendary, "it isn't legendary");
    assert!(on(&g, P0, "Klauth, Unrivaled Ancient").is_some_and(|m| !g.perm(m).token));
}

// ------------------------------------------------------------------ Hellkite Courser
#[test]
fn hellkite_courser_puts_the_commander_in_until_the_end_step() {
    let mut g = tbl();
    let ur = card(&g, u::UR_DRAGON);
    perm(&mut g, P0, "Hellkite Courser");
    let m = on(&g, P0, u::UR_DRAGON).expect("the commander is on the battlefield");
    assert!(!g.player(P0).cmd_zone_of(ur) && g.perm(m).is_cmd);
    assert!(sim_core::engine::combat::has_haste(&g, m), "it gains haste");
    u::courser_return(&mut g).unwrap();
    assert!(!g.perm(m).on_bf && g.player(P0).cmd_zone_of(ur), "back to the command zone");
    assert!(!g.player(P0).library.contains(&ur) && !g.player(P0).gy.contains(&ur));
}

// ------------------------------------------------------------------ Klauth
#[test]
fn klauth_adds_mana_equal_to_the_attackers_power() {
    let mut g = tbl();
    g.step = Step::Combat;
    let k = perm(&mut g, P0, "Klauth, Unrivaled Ancient");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let f = imp(&g, "Klauth, Unrivaled Ancient").attack.unwrap();
    f(&mut g, k, P0, &[gold], P1).unwrap();
    assert_eq!(g.player(P0).floating.any, 0, "Klauth itself must attack");
    f(&mut g, k, P0, &[k, gold], P1).unwrap();
    assert_eq!(g.player(P0).floating.any, 4 + 4);
}

// ------------------------------------------------------------------ Rith, Twinflame Tyrant, Scion of Draco
#[test]
fn rith_gives_other_dragons_ward() {
    let mut g = tbl();
    let rith = perm(&mut g, P0, "Rith, Liberated Primeval");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let opp = perm(&mut g, P1, "Goldspan Dragon");
    assert_eq!(sim_core::cardcode::aura_ward(&g, gold), 2);
    assert_eq!(sim_core::cardcode::aura_ward(&g, rith), 0, "other Dragons (Rith has its own printed ward)");
    assert_eq!(sim_core::cardcode::aura_ward(&g, bird), 0);
    assert_eq!(sim_core::cardcode::aura_ward(&g, opp), 0, "only yours");
    assert!(tokens(&g, P0).is_empty(), "no 4/4 as it enters");
}

#[test]
fn twinflame_tyrant_doubles_damage_to_opponents() {
    let mut g = tbl();
    assert_eq!(sim_core::engine::combat::fiery(&g, P0), 1);
    perm(&mut g, P0, "Twinflame Tyrant");
    assert_eq!(sim_core::engine::combat::fiery(&g, P0), 2);
    let life = g.player(P1).life;
    sim_core::engine::life::lose_life(&mut g, P1, 3, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P1).life, life - 6);
    let mine = g.player(P0).life;
    sim_core::engine::life::lose_life(&mut g, P0, 3, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, mine - 3, "not damage to yourself");
}

#[test]
fn scion_of_draco_has_domain_and_grants_keywords_by_colour() {
    let mut g = tbl();
    no_eminence(&mut g);
    let draco = card(&g, "Scion of Draco");
    assert_eq!(cost_of(&g, P0, draco).0, 12);
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Ketria Triome", 1, false); // Forest Island Mountain
    assert_eq!(cost_of(&g, P0, draco).0, 4, "four basic land types: {{8}} less");
    perm(&mut g, P0, "Scion of Draco");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let gilded = perm(&mut g, P0, "Ancient Gold Dragon");
    let kw = |g: &Game, m, k| sim_core::dsl::has_kw(g, m, k);
    assert!(kw(&g, gold, "first strike") && !kw(&g, gold, "vigilance"), "red: first strike");
    assert!(kw(&g, gilded, "vigilance") && !kw(&g, gilded, "trample"), "white: vigilance");
    let o = perm(&mut g, P1, "Goldspan Dragon");
    assert!(!kw(&g, o, "first strike"), "only your creatures");
}

// ------------------------------------------------------------------ Tiamat, Ureni
#[test]
fn tiamat_finds_five_dragons_with_different_names_if_cast() {
    let mut g = tbl();
    library(
        &mut g,
        P0,
        &[
            "Island",
            "Goldspan Dragon",
            "Goldspan Dragon",
            "Tiamat",
            "Ancient Gold Dragon",
            "Ancient Silver Dragon",
            "Ancient Brass Dragon",
            "Old Gnawbone",
            "Klauth, Unrivaled Ancient",
        ],
    );
    let t = perm(&mut g, P0, "Tiamat");
    assert!(g.player(P0).hand.is_empty(), "put onto the battlefield, not cast: nothing");
    g.last_cast_etb = true;
    (imp(&g, "Tiamat").etb.unwrap())(&mut g, t, P0, t).unwrap();
    g.last_cast_etb = false;
    let mut names: Vec<&str> = g.player(P0).hand.iter().map(|&c| &*g.db.get(c).name).collect();
    assert_eq!(names.len(), 5);
    names.dedup();
    assert_eq!(names.len(), 5, "different names");
    assert!(!names.contains(&"Tiamat") && !names.contains(&"Island"));
}

#[test]
fn ureni_puts_a_dragon_from_the_top_eight_onto_the_battlefield() {
    let mut g = tbl();
    // nine cards: the Dragon at the bottom is not among the top eight
    library(
        &mut g,
        P0,
        &[
            "Atarka, World Render",
            "Island",
            "Island",
            "Goldspan Dragon",
            "Island",
            "Island",
            "Island",
            "Island",
            "Sol Ring",
        ],
    );
    perm(&mut g, P0, "Ureni of the Unwritten");
    assert!(on(&g, P0, "Goldspan Dragon").is_some());
    assert!(on(&g, P0, "Atarka, World Render").is_none(), "only the top eight");
    let lib = &g.player(P0).library;
    assert_eq!(lib.len(), 8);
    assert_eq!(lib.last(), Some(&card(&g, "Atarka, World Render")), "the rest went under it");
}

// ------------------------------------------------------------------ Rivaz of the Claw, Haven
#[test]
fn rivaz_casts_a_dragon_from_the_graveyard_once_each_turn() {
    let mut g = tbl();
    no_eminence(&mut g);
    let rivaz = perm(&mut g, P0, "Rivaz of the Claw");
    lands(&mut g, P0, "Mountain", 5, false);
    let gold = card(&g, "Goldspan Dragon");
    let copper = card(&g, "Ancient Copper Dragon");
    g.player_mut(P0).gy.extend([gold, copper]);
    let f = imp(&g, "Rivaz of the Claw").options.unwrap();
    let o = f(&mut g, rivaz, P0, Some(false)).unwrap().pop().expect("a Dragon to cast");
    assert!(o.label.contains("Ancient Copper Dragon"), "the best one it can pay for: {}", o.label);
    assert!(run(&mut g, o, P0));
    let m = on(&g, P0, "Ancient Copper Dragon").expect("cast from the graveyard");
    assert!(f(&mut g, rivaz, P0, Some(false)).unwrap().is_empty(), "once each turn");
    // it dies: exiled
    sim_core::engine::zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).exile.contains(&copper) && !g.player(P0).gy.contains(&copper));
}

#[test]
fn rivaz_and_haven_mana_is_for_dragon_creature_spells() {
    let mut g = tbl();
    no_eminence(&mut g);
    perm(&mut g, P0, "Rivaz of the Claw");
    lands(&mut g, P0, "Haven of the Spirit Dragon", 1, false);
    let gold = card(&g, "Goldspan Dragon");
    let can = |g: &mut Game, c: CardId, gn: u32, pips: &str| {
        g.pay_for = Some(c);
        let ok = sim_core::engine::mana::can_pay(g, P0, gn, pips, false);
        g.pay_for = None;
        ok
    };
    assert!(can(&mut g, gold, 1, "RR"), "Rivaz's two and Haven's one for a Dragon");
    let sol = card(&g, "Sol Ring");
    assert!(!can(&mut g, sol, 0, "R"), "Haven gives {{C}} otherwise, Rivaz nothing");
    assert!(can(&mut g, sol, 1, ""));
}

// ------------------------------------------------------------------ Roaming Throne
#[test]
fn roaming_throne_doubles_triggers_of_the_chosen_type() {
    let mut g = tbl_dragons();
    let t = perm(&mut g, P0, "Roaming Throne");
    assert_eq!(g.perm(t).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
    assert!(sim_core::engine::values::has_type(&g, t, "dragon"), "it is the chosen type too");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    assert_eq!(u::throne_extra(&g, gold), 1);
    assert_eq!(u::throne_extra(&g, bird), 0);
    assert_eq!(u::throne_extra(&g, t), 0, "another creature");
    // Lathliss's trigger twice: two 5/5s
    perm(&mut g, P0, "Lathliss, Dragon Queen");
    perm(&mut g, P0, "Ancient Gold Dragon");
    assert_eq!(tokens(&g, P0).len(), 2);
}

#[test]
fn lathliss_pumps_dragons_before_an_open_attack() {
    let mut g = tbl();
    let l = perm(&mut g, P0, "Lathliss, Dragon Queen");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    lands(&mut g, P0, "Mountain", 6, false);
    let f = imp(&g, "Lathliss, Dragon Queen").options.unwrap();
    assert!(f(&mut g, l, P0, None).unwrap().is_empty(), "before combat only");
    hand(&mut g, P0, &["Ancient Copper Dragon"]);
    assert!(f(&mut g, l, P0, Some(false)).unwrap().is_empty(), "a Dragon to cast first");
    g.player_mut(P0).hand.clear();
    let o = f(&mut g, l, P0, Some(false)).unwrap().pop().expect("the spare mana pumps");
    assert!(run(&mut g, o, P0));
    assert_eq!(sim_core::engine::values::epow(&g, gold), 4 + 2, "{{1}}{{R}} twice, two lands kept");
    assert_eq!(sim_core::engine::values::epow(&g, l), 6 + 2);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2);
}

// ------------------------------------------------------------------ Crux of Fate (the compiled Dragon modes)
#[test]
fn crux_of_fate_spares_your_dragons() {
    let mut g = tbl();
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let theirs = perm(&mut g, P1, "Birds of Paradise");
    let crux = card(&g, "Crux of Fate");
    sim_core::engine::cast::resolve(&mut g, P0, crux, &sim_core::state::Ctx::default(), "hand").unwrap();
    assert!(g.perm(gold).on_bf, "the non-Dragon mode: the Dragon lives");
    assert!(!g.perm(bird).on_bf && !g.perm(theirs).on_bf, "every non-Dragon creature is destroyed");
}
