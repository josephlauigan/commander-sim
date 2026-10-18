//! Tier 6 batch `legends2` (impls/t6_legends2.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::engine::values::epow;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["sisay-legends-toolbox", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn has_perm(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

// ======================================================== Magda, Brazen Outlaw
#[test]
fn magda_pumps_other_dwarves_only() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Magda, Brazen Outlaw");
    let t = perm(&mut g, P0, "Tataru Taru");
    assert_eq!(epow(&g, m), 2, "Magda does not pump herself");
    assert_eq!(epow(&g, t), 1, "another Dwarf gets +1/+0 (0/3 -> 1/3)");
    let o = perm(&mut g, P1, "Tataru Taru");
    assert_eq!(epow(&g, o), 0, "an opponent's Dwarf gets nothing");
}

#[test]
fn magda_makes_a_treasure_when_a_dwarf_taps() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Magda, Brazen Outlaw");
    let t = perm(&mut g, P0, "Tataru Taru");
    let human = perm(&mut g, P0, "The Queen of Dale");
    let before = g.player(P0).treasures;
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().mana_tapped.unwrap();
    f(&mut g, m, P0, t, 1).unwrap();
    assert_eq!(g.player(P0).treasures, before + 1, "a Dwarf tapped for mana");
    f(&mut g, m, P0, human, 1).unwrap();
    assert_eq!(g.player(P0).treasures, before + 1, "a Human did not");
    // attacking: Magda and Tataru tap
    g.perm_mut(m).tapped = true;
    g.perm_mut(t).tapped = true;
    let a = g.registry.get(c).unwrap().attack.unwrap();
    a(&mut g, m, P0, &[m, t, human], P1).unwrap();
    assert_eq!(g.player(P0).treasures, before + 3, "two attacking Dwarves, two Treasures");
}

#[test]
fn magda_sacrifices_five_treasures_for_an_artifact_or_dragon() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Magda, Brazen Outlaw");
    set_library(&mut g, P0, &["Lightning Bolt", "Sol Ring", "Balefire Dragon"]);
    g.player_mut(P0).treasures = 4;
    assert!(options(&mut g, m, P0, Some(false)).is_empty(), "four Treasures are not enough");
    g.player_mut(P0).treasures = 6;
    let o = options(&mut g, m, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).treasures, 1, "five sacrificed");
    let got = has_perm(&g, P0, "Sol Ring") || has_perm(&g, P0, "Balefire Dragon");
    assert!(got, "an artifact or Dragon is on the battlefield");
    assert!(!has_perm(&g, P0, "Lightning Bolt"));
}

fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

// ======================================================== The Queen of Dale
#[test]
fn queen_of_dale_recruits_on_the_first_noncreature_spell_only() {
    let mut g = tbl();
    g.active = Some(P1);
    perm(&mut g, P0, "The Queen of Dale");
    hand(&mut g, P0, &["Sol Ring"]);
    set_library(&mut g, P0, &["Plains", "Plains", "Plains"]);
    let bolt = hand(&mut g, P1, &["Lightning Bolt", "Counterspell", "Goblin Guide"]);
    let hand0 = g.player(P0).hand.len();
    sim_core::engine::cast::on_cast(&mut g, P1, bolt[2]).unwrap(); // a creature spell: no trigger
    assert_eq!(g.player(P0).hand.len(), hand0);
    sim_core::engine::cast::on_cast(&mut g, P1, bolt[0]).unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0, "drew one, discarded one");
    assert_eq!(tokens(&g, P0), 1, "a nonland card was discarded: a 1/1 Human Soldier");
    assert!(g.player(P0).gy.contains(&card(&g, "Sol Ring")));
    sim_core::engine::cast::on_cast(&mut g, P1, bolt[1]).unwrap(); // the second noncreature spell
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(tokens(&g, P0), 1, "only the first noncreature spell each turn");
}

#[test]
fn queen_of_dale_ignores_its_own_controllers_spells() {
    let mut g = tbl();
    perm(&mut g, P0, "The Queen of Dale");
    set_library(&mut g, P0, &["Plains", "Plains"]);
    let b = hand(&mut g, P0, &["Sol Ring"]);
    sim_core::engine::cast::on_cast(&mut g, P0, b[0]).unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(tokens(&g, P0), 0);
}
