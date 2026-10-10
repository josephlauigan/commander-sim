//! Tier 5 card code (impls/t5.rs, Python's cards/impl/t5.py) on hand-built positions: Urza's Construct and blue
//! artifacts, Kinnan, Winota, Yawgmoth on both sides of combat, Solitary Confinement, Copy Enchantment, Zealous
//! Conscripts, Seedborn Muse, Windfall, Karn's and Cursed Totem's locks, Nether Traitor, Zur, Kiki-Jiki and Tezzeret.
//! Each expectation was checked against the Python card code on the same position.

use sim_core::cardcode;
use sim_core::engine::mana::{Source, mana_units, total_mana};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{hooks, life, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const UR: &str = "urza-mono-blue-artifacts";
const KI: &str = "kinnan-simic-mana-combo";
const WI: &str = "winota-boros-humans-cheat";
const YA: &str = "yawgmoth-mono-black-aristocrats";
const ZU: &str = "zur-esper-enchantment-control";
const BR: &str = "brago-azorius-blink-control";

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = g.player(p).perms.iter().map(|&m| g.perm(m).name.to_string()).collect();
    v.sort();
    v
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        _ => panic!("not an ability"),
    }
}

fn labels(os: &[Opt]) -> Vec<(String, f64)> {
    os.iter().map(|o| (o.label.clone(), (o.utility * 10000.0).round() / 10000.0)).collect()
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token).collect()
}

// ------------------------------------------------------------------ Urza, Kinnan
#[test]
fn urza_makes_a_construct_and_artifacts_tap_for_blue() {
    let mut g = table(&[UR, BR]);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    let con = tokens(&g, P0)[0];
    assert_eq!((epow(&g, con), etgh(&g, con)), (3, 3), "a 0/0 with three artifacts");
    let us = mana_units(&g, P0, false);
    let blue: Vec<_> = us.iter().filter(|w| w.cols.has('U')).collect();
    assert_eq!(blue.len(), 1, "the Construct (the rocks already make mana)");
    assert_eq!(blue[0].src, Source::Perm(con));
    assert_eq!(total_mana(&g, P0, false), 4);
    assert!(options(&mut g, u, P0, Some(false)).is_empty(), "{{5}} is out of reach");
}

#[test]
fn kinnan_adds_one_to_nonland_mana_and_digs_five() {
    let mut g = table(&[KI, BR]);
    perm(&mut g, P0, "Sol Ring");
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Island", 2, false);
    assert_eq!(total_mana(&g, P0, false), 7);
    let k = perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    assert_eq!(total_mana(&g, P0, false), 8);
    let o = options(&mut g, k, P0, Some(true));
    assert_eq!(labels(&o), [("Kinnan: dig five".to_string(), 5.0)]);
    let lib = g.player(P0).library.len();
    let before = g.player(P0).perms.len();
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).library.len(), lib - 1, "one creature out, four to the bottom");
    assert_eq!(g.player(P0).perms.len(), before + 1);
}

// ------------------------------------------------------------------ Winota
#[test]
fn winota_finds_an_attacking_indestructible_human() {
    let mut g = table(&[WI, BR]);
    perm(&mut g, P0, "Winota, Joiner of Forces");
    let tk = token(&mut g, P0, 1);
    let lib = g.player(P0).library.len();
    g.new_attackers.clear();
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![tk], d: P1 }).unwrap();
    assert_eq!(g.new_attackers.len(), 1);
    let h = g.new_attackers[0];
    let x = g.perm(h);
    assert!(g.db.get(x.cd.unwrap()).has_subtype("human"));
    assert!(x.tapped && !x.sick && x.eot_kw.contains(&"indestructible"));
    assert_eq!(g.player(P0).library.len(), lib - 1);
}

// ------------------------------------------------------------------ Yawgmoth
#[test]
fn yawgmoth_sacrifices_fodder_for_a_card() {
    let mut g = table(&[YA, BR]);
    lands(&mut g, P0, "Swamp", 3, false);
    let y = perm(&mut g, P0, "Yawgmoth, Thran Physician");
    let tk = token(&mut g, P0, 1);
    let wall = perm(&mut g, P1, "Wall of Omens");
    hand(&mut g, P0, &["Nether Traitor"]);
    assert!(options(&mut g, y, P0, None).is_empty());
    let o = options(&mut g, y, P0, Some(false));
    assert_eq!(labels(&o), [("Yawgmoth: sacrifice Token".to_string(), 1.5)]);
    let n = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).life, 39);
    assert!(!g.perm(tk).on_bf);
    assert_eq!(g.perm(wall).plus, 0, "a 0/4: no counter");
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn yawgmoth_kills_unblocked_x1_attackers() {
    let mut g = table(&[YA, BR]);
    perm(&mut g, P0, "Yawgmoth, Thran Physician");
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    let mut atk = vec![];
    for (pw, tg) in [(1, 1), (2, 1), (3, 3)] {
        let spec = zones::Tokens { sick: false, tgh: Some(tg), ..zones::Tokens::new(1, pw) };
        atk.extend(zones::make_tokens(&mut g, P1, spec).unwrap());
    }
    let mut assign = vec![];
    let mut a = atk.clone();
    cardcode::defend_hooks(&mut g, P1, &mut a, P0, &mut assign).unwrap();
    assert_eq!(g.player(P0).life, 38);
    assert_eq!(atk.iter().map(|&m| g.perm(m).on_bf).collect::<Vec<_>>(), [false, false, true]);
    assert!(tokens(&g, P0).is_empty());
}

#[test]
fn nether_traitor_returns_when_a_creature_card_dies() {
    let mut g = table(&[YA, BR]);
    lands(&mut g, P0, "Swamp", 1, false);
    let c = take(&mut g, P0, "Nether Traitor");
    g.player_mut(P0).gy.push(c);
    let tk = token(&mut g, P0, 1);
    zones::die(&mut g, tk, "destroy").unwrap();
    assert!(names(&g, P0).is_empty(), "a token dying doesn't count");
    let w = perm(&mut g, P0, "Wall of Omens");
    zones::die(&mut g, w, "destroy").unwrap();
    assert_eq!(names(&g, P0), ["Nether Traitor"]);
    let l = g.player(P0).lands[0];
    assert!(g.land(l).tapped);
}

// ------------------------------------------------------------------ Zur's deck
#[test]
fn solitary_confinement_skips_draws_prevents_damage_and_costs_a_card() {
    let mut g = table(&[ZU, BR]);
    let sc = perm(&mut g, P0, "Solitary Confinement");
    hand(&mut g, P0, &["Propaganda", "Swords to Plowshares"]);
    assert!(cardcode::skip_draw(&g, P0) && !cardcode::skip_draw(&g, P1));
    assert!(life::prevents_damage(&g, P0, Some(P1)) && !life::prevents_damage(&g, P1, Some(P0)));
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
    assert!(g.perm(sc).on_bf);
    g.player_mut(P0).hand.clear();
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(!g.perm(sc).on_bf, "no card to discard: sacrificed");
}

#[test]
fn copy_enchantment_copies_the_best_enchantment() {
    let mut g = table(&[ZU, BR]);
    perm(&mut g, P1, "Propaganda");
    perm(&mut g, P0, "Ghostly Prison");
    perm(&mut g, P0, "Copy Enchantment");
    assert_eq!(names(&g, P0), ["Ghostly Prison", "Ghostly Prison"]);
    let copy = *g.player(P0).perms.last().unwrap();
    assert_eq!(g.perm(copy).phys, Some(card(&g, "Copy Enchantment")));
}

#[test]
fn zur_fetches_necropotence_first() {
    let mut g = table(&[ZU, BR]);
    let z = perm(&mut g, P0, "Zur the Enchanter");
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![z], d: P1 }).unwrap();
    assert_eq!(names(&g, P0), ["Necropotence", "Zur the Enchanter"]);
}

// ------------------------------------------------------------------ Winota's deck
#[test]
fn zealous_conscripts_steals_the_best_creature() {
    let mut g = table(&[WI, BR]);
    let t = perm(&mut g, P1, "Restoration Angel");
    perm(&mut g, P0, "Zealous Conscripts");
    assert!(g.player(P0).perms.contains(&t));
    assert_eq!(g.perm(t).owner, P0);
    assert_eq!(g.player(P0).borrowed, vec![t]);
}

#[test]
fn kiki_jiki_copies_your_best_enter_effect() {
    let mut g = table(&[WI, BR]);
    let ki = perm(&mut g, P0, "Kiki-Jiki, Mirror Breaker");
    perm(&mut g, P0, "Coiling Oracle");
    assert!(options(&mut g, ki, P0, Some(true)).is_empty(), "before combat only");
    let o = options(&mut g, ki, P0, Some(false));
    assert_eq!(labels(&o), [("Kiki-Jiki copies Coiling Oracle".to_string(), 3.3)]);
    assert!(run(&mut g, P0, &o[0]));
    let t = tokens(&g, P0);
    assert_eq!(t.len(), 1);
    let x = g.perm(t[0]);
    assert!(x.name == "Coiling Oracle" && !x.sick && x.temp);
}

// ------------------------------------------------------------------ Kinnan's and Urza's decks
#[test]
fn seedborn_muse_untaps_in_other_players_untap_steps() {
    let mut g = table(&[KI, BR]);
    perm(&mut g, P0, "Seedborn Muse");
    let ls = lands(&mut g, P0, "Forest", 2, true);
    let r = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(r).tapped = true;
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(ls.iter().all(|&l| g.land(l).tapped) && g.perm(r).tapped);
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P1 }).unwrap();
    assert!(ls.iter().all(|&l| !g.land(l).tapped) && !g.perm(r).tapped);
}

#[test]
fn windfall_draws_the_largest_hand_less_one_for_its_caster() {
    let mut g = table(&[UR, BR]);
    let c = hand(&mut g, P0, &["Windfall", "Sol Ring"])[0];
    hand(&mut g, P1, &["Counterspell", "Brainstorm", "Ponder", "Swords to Plowshares"]);
    g.player_mut(P0).hand.retain(|&x| x != c);
    let f = g.registry.get(c).and_then(|i| i.resolve).unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    let (a, b) = (g.player(P0), g.player(P1));
    assert_eq!((a.hand.len(), b.hand.len(), a.gy.len(), b.gy.len()), (4, 4, 1, 4));
}

#[test]
fn karn_and_cursed_totem_shut_off_mana() {
    let mut g = table(&[UR, BR]);
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P1, "Island", 1, false);
    g.player_mut(P1).treasures = 1;
    assert_eq!(total_mana(&g, P1, false), 4);
    perm(&mut g, P0, "Karn, the Great Creator");
    assert_eq!(total_mana(&g, P1, false), 1, "opponents' artifacts (Treasures too) make nothing");
    assert_eq!(total_mana(&g, P0, false), 0);

    let mut g = table(&[UR, KI]);
    perm(&mut g, P1, "Llanowar Elves");
    lands(&mut g, P1, "Forest", 1, false);
    assert_eq!(total_mana(&g, P1, false), 2);
    perm(&mut g, P0, "Cursed Totem");
    assert_eq!(total_mana(&g, P1, false), 1);
}

#[test]
fn tezzeret_puts_an_artifact_onto_the_battlefield() {
    let mut g = table(&[UR, BR]);
    let tz = perm(&mut g, P0, "Tezzeret the Seeker");
    let o = options(&mut g, tz, P0, Some(false));
    assert_eq!(g.perm(tz).loyalty, Some(4));
    assert_eq!(labels(&o), [("Tezzeret the Seeker +1".to_string(), 2.15), ("Tezzeret the Seeker -2".to_string(), 2.7)]);
    assert!(run(&mut g, P0, &o[1]));
    assert_eq!(names(&g, P0), ["Grim Monolith", "Tezzeret the Seeker"], "a combo artifact with mana value 2 or less");
    assert_eq!(g.perm(tz).loyalty, Some(2));
}
