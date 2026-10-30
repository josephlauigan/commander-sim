//! The Tier 1 lathril-golgari-elves cards (impls/t12_lathril.rs and the compiled patterns under
//! `// --- Tiers 1/2 (lathril)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::dsl::has_kw;
use sim_core::engine::cast;
use sim_core::engine::mana::total_mana;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const LATHRIL: &str = "lathril-golgari-elves";

fn elves() -> Game {
    table(&[LATHRIL, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn tokens_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options");
    f(g, src, p, post).unwrap()
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx::default(), "hand").unwrap();
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn my_turn(g: &mut Game, p: PlayerId, step: Step) {
    g.active = Some(p);
    g.step = step;
}

// ------------------------------------------------------------------ blight: High Perfect Morcant
#[test]
fn morcant_makes_each_opponent_blight_when_an_elf_enters() {
    let mut g = elves();
    let a = token(&mut g, P1, 3);
    let b = token(&mut g, P2, 2);
    perm(&mut g, P0, "High Perfect Morcant");
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (-1, -1), "Morcant itself entering");
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (-2, -2), "another Elf entering");
    let t = token(&mut g, P0, 1); // a non-Elf creature of mine: nothing
    assert_eq!(g.perm(t).plus, 0);
    assert_eq!(g.perm(a).plus, -2);
}

#[test]
fn morcant_taps_three_elves_to_proliferate_after_combat() {
    let mut g = elves();
    let m = perm(&mut g, P0, "High Perfect Morcant");
    let es: Vec<PermId> =
        ["Llanowar Elves", "Elvish Mystic", "Fyndhorn Elves"].iter().map(|n| perm(&mut g, P0, n)).collect();
    let o = token(&mut g, P1, 3);
    g.perm_mut(o).plus = -1;
    for &e in &es {
        g.perm_mut(e).plus = 1;
    }
    my_turn(&mut g, P0, Step::Main1);
    assert!(opts(&mut g, m, P0, Some(false)).is_empty(), "not before combat");
    g.step = Step::Main2;
    let o1 = opts(&mut g, m, P0, Some(true)).pop().expect("the proliferate option");
    assert!(run(&mut g, o1, P0));
    let tapped = g.player(P0).perms.iter().filter(|&&x| g.perm(x).tapped).count();
    assert_eq!(tapped, 3, "three Elves tapped");
    assert_eq!(g.perm(o).plus, -2, "their -1/-1 counter");
    assert_eq!(g.perm(es[0]).plus, 2, "my +1/+1 counter");
}

// ------------------------------------------------------------------ Banner of Kinship
#[test]
fn banner_counts_elves_as_it_enters_and_pumps_them() {
    let mut g = elves();
    let a = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Elvish Mystic");
    perm(&mut g, P0, "Fyndhorn Elves");
    let t = token(&mut g, P0, 2); // not an Elf
    let (p0, t0) = (epow(&g, a), etgh(&g, a));
    let b = perm(&mut g, P0, "Banner of Kinship");
    assert_eq!(g.perm(b).plus, 3, "a fellowship counter per Elf");
    assert_eq!((epow(&g, a), etgh(&g, a)), (p0 + 3, t0 + 3));
    assert_eq!(epow(&g, t), 2, "a non-Elf gets nothing");
    let o = perm(&mut g, P1, "Llanowar Elves");
    assert_eq!(epow(&g, o), 1, "only creatures you control");
}

// ------------------------------------------------------------------ Bounty of Skemfar
#[test]
fn bounty_puts_a_land_onto_the_battlefield_tapped_and_an_elf_into_hand() {
    let mut g = elves();
    let top: Vec<CardId> = ["Sol Ring", "Forest", "Elvish Mystic", "Cultivate", "Putrefy", "Wood Elves"]
        .iter()
        .map(|n| take(&mut g, P0, n))
        .collect();
    g.player_mut(P0).library.extend(&top);
    let b = take(&mut g, P0, "Bounty of Skemfar");
    let lands = g.player(P0).lands.len();
    let hand = g.player(P0).hand.len();
    let lib = g.player(P0).library.len();
    cast::resolve(&mut g, P0, b, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P0).lands.len(), lands + 1);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.land(l).tapped && &*g.db.get(g.land(l).card()).name == "Forest");
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    let elf = *g.player(P0).hand.last().unwrap();
    assert!(["Elvish Mystic", "Wood Elves"].contains(&&*g.db.get(elf).name));
    let lib_now = &g.player(P0).library;
    assert_eq!(lib_now.len(), lib - 2);
    for c in &top {
        if *c != elf && g.db.get(*c).name.as_ref() != "Forest" {
            let i = lib_now.iter().position(|x| x == c).unwrap();
            assert!(i < 4, "the rest on the bottom");
        }
    }
}

// ------------------------------------------------------------------ Crown of Skemfar
#[test]
fn crown_gives_plus_one_per_elf_and_reach_and_returns_from_the_graveyard() {
    let mut g = elves();
    let a = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Elvish Mystic");
    let base = epow(&g, a);
    let cr = perm(&mut g, P0, "Crown of Skemfar");
    let host = g.perm(cr).attached.expect("it enchants a creature");
    let host_base = if host == a { base } else { 1 };
    assert_eq!(epow(&g, host), host_base + 2, "+1/+1 for each of two Elves");
    assert!(has_kw(&g, host, "reach"));
    // from the graveyard: {2}{G}
    let c = to_gy(&mut g, P0, "Crown of Skemfar");
    lands(&mut g, P0, "Forest", 3, false);
    let f = g.registry.get(c).and_then(|i| i.gy_options).unwrap();
    let o = f(&mut g, c, P0, None).unwrap().pop().expect("the return option");
    assert!(run(&mut g, o, P0));
    assert!(g.player(P0).hand.contains(&c) && !g.player(P0).gy.contains(&c));
}

// ------------------------------------------------------------------ Pact of the Serpent
#[test]
fn pact_draws_and_loses_life_for_each_elf() {
    let mut g = elves();
    for n in ["Llanowar Elves", "Elvish Mystic", "Fyndhorn Elves", "Wood Elves"] {
        perm(&mut g, P0, n);
    }
    let (hand, life) = (g.player(P0).hand.len(), g.player(P0).life);
    resolve(&mut g, P0, "Pact of the Serpent");
    assert_eq!(g.player(P0).hand.len(), hand + 4);
    assert_eq!(g.player(P0).life, life - 4);
}

#[test]
fn pact_kills_an_opponent_the_life_loss_finishes() {
    let mut g = elves();
    perm(&mut g, P0, "Llanowar Elves");
    for _ in 0..3 {
        let t = token(&mut g, P1, 1);
        g.perm_mut(t).ttypes = vec!["zombie"];
    }
    g.player_mut(P1).life = 3;
    resolve(&mut g, P0, "Pact of the Serpent");
    assert!(!g.player(P1).alive || g.player(P1).life <= 0, "three Zombies: three life");
}

// ------------------------------------------------------------------ Champions of the Perfect
#[test]
fn champions_beholds_an_elf_and_returns_it_when_it_leaves() {
    let mut g = elves();
    let c = take(&mut g, P0, "Champions of the Perfect");
    assert!(!cast::additional_cost(&mut g, P0, c, true).unwrap(), "no Elf to behold");
    let mystic = hand(&mut g, P0, &["Elvish Mystic"])[0];
    assert!(cast::additional_cost(&mut g, P0, c, true).unwrap());
    cast::additional_cost(&mut g, P0, c, false).unwrap();
    assert!(g.player(P0).exile.contains(&mystic) && !g.player(P0).hand.contains(&mystic));
    let m = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).hand.contains(&mystic) && !g.player(P0).exile.contains(&mystic), "back to hand");
}

// ------------------------------------------------------------------ Abomination of Llanowar
#[test]
fn abomination_counts_elves_and_elf_cards_in_the_graveyard() {
    let mut g = elves();
    perm(&mut g, P0, "Llanowar Elves");
    to_gy(&mut g, P0, "Elvish Mystic");
    to_gy(&mut g, P0, "Cultivate");
    let a = perm(&mut g, P0, "Abomination of Llanowar");
    assert_eq!((epow(&g, a), etgh(&g, a)), (3, 3), "two Elves (itself too) and one Elf card");
}

// ------------------------------------------------------------------ Elderfang Venom
#[test]
fn elderfang_venom_drains_when_an_elf_dies_and_attacking_elves_have_deathtouch() {
    let mut g = elves();
    perm(&mut g, P0, "Elderfang Venom");
    let e = perm(&mut g, P0, "Llanowar Elves");
    let t = token(&mut g, P0, 2);
    assert!(!has_kw(&g, e, "deathtouch"), "not while it isn't attacking");
    g.in_combat = vec![e, t];
    assert!(has_kw(&g, e, "deathtouch") && !has_kw(&g, t, "deathtouch"), "attacking Elves only");
    g.in_combat.clear();
    let (l0, l1, l2) = (g.player(P0).life, g.player(P1).life, g.player(P2).life);
    zones::die(&mut g, e, "destroy").unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life, g.player(P2).life), (l0 + 1, l1 - 1, l2 - 1));
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.player(P1).life, l1 - 1, "a non-Elf dying: nothing");
}

// ------------------------------------------------------------------ Elven Chorus
#[test]
fn elven_chorus_casts_creatures_from_the_top() {
    let mut g = elves();
    let ch = perm(&mut g, P0, "Elven Chorus");
    lands(&mut g, P0, "Forest", 4, false);
    let w = take(&mut g, P0, "Wood Elves");
    g.player_mut(P0).library.push(w);
    my_turn(&mut g, P0, Step::Main1);
    let o = opts(&mut g, ch, P0, Some(false));
    assert!(o.iter().any(|x| x.label.contains("Wood Elves")), "cast it from the top");
    let s = take(&mut g, P0, "Putrefy");
    g.player_mut(P0).library.push(s);
    assert!(opts(&mut g, ch, P0, Some(false)).is_empty(), "not a noncreature spell");
}

// ------------------------------------------------------------------ Miara, Thorn of the Glade
#[test]
fn miara_pays_one_and_a_life_to_draw_when_an_elf_dies() {
    let mut g = elves();
    lands(&mut g, P0, "Swamp", 2, false);
    let mi = perm(&mut g, P0, "Miara, Thorn of the Glade");
    let e = perm(&mut g, P0, "Llanowar Elves");
    let (hand, life) = (g.player(P0).hand.len(), g.player(P0).life);
    zones::die(&mut g, e, "destroy").unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (hand + 1, life - 1));
    zones::die(&mut g, mi, "destroy").unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (hand + 2, life - 2), "Miara itself");
}

// ------------------------------------------------------------------ Nullmage Shepherd
#[test]
fn nullmage_shepherd_taps_four_creatures_to_destroy_an_artifact_or_enchantment() {
    let mut g = elves();
    let s = perm(&mut g, P0, "Nullmage Shepherd");
    let tgt = perm(&mut g, P1, "Rhystic Study");
    for _ in 0..2 {
        token(&mut g, P0, 1);
    }
    assert!(opts(&mut g, s, P0, None).is_empty(), "three creatures only");
    token(&mut g, P0, 1);
    let o = opts(&mut g, s, P0, None).pop().expect("the destroy option");
    assert!(run(&mut g, o, P0));
    assert!(!g.perm(tgt).on_bf, "Rhystic Study destroyed");
    let tapped = g.player(P0).perms.iter().filter(|&&x| g.perm(x).tapped).count();
    assert_eq!(tapped, 4);
}

// ------------------------------------------------------------------ Return Upon the Tide
#[test]
fn return_upon_the_tide_returns_a_creature_and_two_elves_for_an_elf() {
    let mut g = elves();
    to_gy(&mut g, P0, "Wood Elves");
    resolve(&mut g, P0, "Return Upon the Tide");
    assert!(on_bf(&g, P0, "Wood Elves").is_some());
    assert_eq!(tokens_of(&g, P0), 2, "two Elf Warriors");
}

// ------------------------------------------------------------------ Rhys the Exiled
#[test]
fn rhys_sacrifices_an_elf_to_regenerate() {
    let mut g = elves();
    lands(&mut g, P0, "Swamp", 1, false);
    let r = perm(&mut g, P0, "Rhys the Exiled");
    zones::die(&mut g, r, "destroy").unwrap();
    assert!(!g.perm(r).on_bf, "no other Elf: it dies");
    let mut g = elves();
    lands(&mut g, P0, "Swamp", 1, false);
    let r = perm(&mut g, P0, "Rhys the Exiled");
    let e = perm(&mut g, P0, "Elvish Mystic");
    zones::die(&mut g, r, "destroy").unwrap();
    assert!(g.perm(r).on_bf && g.perm(r).tapped, "regenerated");
    assert!(!g.perm(e).on_bf, "the Elf was sacrificed");
}

// ------------------------------------------------------------------ Ruthless Winnower
#[test]
fn ruthless_winnower_makes_each_player_sacrifice_a_non_elf_at_their_upkeep() {
    let mut g = elves();
    let w = perm(&mut g, P0, "Ruthless Winnower");
    let e = perm(&mut g, P0, "Llanowar Elves");
    let o = token(&mut g, P1, 2);
    let c = g.perm(w).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.upkeep).unwrap();
    f(&mut g, w, P1).unwrap();
    assert!(!g.perm(o).on_bf, "their non-Elf");
    let mine = token(&mut g, P0, 1);
    f(&mut g, w, P0).unwrap();
    assert!(g.perm(e).on_bf && g.perm(w).on_bf, "my Elves stay (the Winnower is an Elf too)");
    assert!(!g.perm(mine).on_bf, "my non-Elf goes");
}

// ------------------------------------------------------------------ Tyvar Kell
#[test]
fn tyvar_kell_elves_tap_for_black_and_its_abilities() {
    let mut g = elves();
    let e = perm(&mut g, P0, "Llanowar Elves"); // a mana Elf: taps for its own G only
    let w = perm(&mut g, P0, "Wood Elves");
    my_turn(&mut g, P0, Step::Main2);
    let before = total_mana(&g, P0, false);
    let t = perm(&mut g, P0, "Tyvar Kell");
    assert_eq!(total_mana(&g, P0, false), before + 1, "Wood Elves taps for B");
    let c = g.perm(t).cd.unwrap();
    let ab = g.registry.get(c).and_then(|i| i.walker).unwrap();
    g.perm_mut(e).tapped = true;
    (ab[0].eff)(&mut g, P0, t).unwrap();
    assert!(!g.perm(e).tapped && g.perm(e).plus == 1 && has_kw(&g, e, "deathtouch"), "+1 on the tapped mana Elf");
    let toks = tokens_of(&g, P0);
    (ab[1].eff)(&mut g, P0, t).unwrap();
    assert_eq!(tokens_of(&g, P0), toks + 1, "0: an Elf Warrior");
    (ab[2].eff)(&mut g, P0, t).unwrap();
    let hand = g.player(P0).hand.len();
    let m = take(&mut g, P0, "Elvish Mystic");
    cast::on_cast(&mut g, P0, m).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 2, "the emblem: an Elf spell draws two");
    let mm = zones::enter(&mut g, P0, m, zones::Enter::default()).unwrap();
    assert!(sim_core::engine::combat::has_haste(&g, mm), "and it has haste");
    let _ = w;
}

// ------------------------------------------------------------------ Wolverine Riders
#[test]
fn wolverine_riders_gain_life_equal_to_an_entering_elfs_toughness() {
    let mut g = elves();
    perm(&mut g, P0, "Wolverine Riders");
    let life = g.player(P0).life;
    perm(&mut g, P0, "Wood Elves");
    assert_eq!(g.player(P0).life, life + 1, "Wood Elves is a 1/1");
    perm(&mut g, P0, "Nullmage Shepherd");
    assert_eq!(g.player(P0).life, life + 5, "an Elf Shaman 2/4");
    token(&mut g, P0, 3);
    assert_eq!(g.player(P0).life, life + 5, "not a non-Elf");
}
