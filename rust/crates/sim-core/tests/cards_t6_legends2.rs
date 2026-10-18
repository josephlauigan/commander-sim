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

// ======================================================== Tataru Taru
#[test]
fn tataru_enters_you_draw_and_an_opponent_may_draw() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Plains"]);
    set_library(&mut g, P1, &["Mountain", "Mountain"]);
    set_library(&mut g, PlayerId(2), &["Plains"]);
    set_library(&mut g, PlayerId(3), &["Plains"]);
    let (h0, h1) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    let (h2, h3) = (g.player(PlayerId(2)).hand.len(), g.player(PlayerId(3)).hand.len());
    perm(&mut g, P0, "Tataru Taru");
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), h0 + 1, "you draw a card");
    let drawn =
        g.player(P1).hand.len() - h1 + g.player(PlayerId(2)).hand.len() - h2 + g.player(PlayerId(3)).hand.len() - h3;
    assert_eq!(drawn, 1, "exactly one opponent drew");
    // that opponent drew on our turn: Scions' Secretary made a tapped Treasure, which arrives at our next upkeep
    assert_eq!(g.player(P0).treasures, 0, "tapped: not usable yet");
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Upkeep,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, 1);
}

#[test]
fn scions_secretary_once_a_turn_and_not_on_their_own_turn() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tataru Taru");
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    g.perm_mut(t).data.set(sim_core::state::DataKey::Charge, sim_core::state::Val::Int(0));
    set_library(&mut g, P1, &["Mountain", "Mountain", "Mountain", "Mountain"]);
    g.active = Some(P1);
    sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.perm(t).data.get(sim_core::state::DataKey::Charge).map_or(0, |v| v.int()), 0, "their own turn");
    g.active = Some(P0);
    g.round = 2; // (a new turn: the Treasure from the enter-trigger draw used the first turn's)
    for _ in 0..3 {
        sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
        sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    }
    assert_eq!(g.perm(t).data.get(sim_core::state::DataKey::Charge).map_or(0, |v| v.int()), 1, "once each turn");
    sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, 1, "Tataru left: the Treasure appears");
}

// ======================================================== Teferi, Who Slows the Sunset
fn walker_eff(g: &mut Game, t: PermId, p: PlayerId, i: usize) {
    let c = g.perm(t).cd.unwrap();
    let ab = g.registry.get(c).unwrap().walker.unwrap()[i];
    (ab.eff)(g, p, t).unwrap();
}

#[test]
fn teferi_plus_untaps_ours_taps_theirs_and_gains_two() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Teferi, Who Slows the Sunset");
    let rock = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(rock).tapped = true;
    let mine = token(&mut g, P0, 2);
    g.perm_mut(mine).tapped = true;
    let l0 = lands(&mut g, P0, "Plains", 1, true)[0];
    let l1 = lands(&mut g, P1, "Mountain", 2, false);
    let life = g.player(P0).life;
    walker_eff(&mut g, t, P0, 0);
    assert!(!g.perm(rock).tapped, "our artifact untapped");
    assert!(!g.perm(mine).tapped, "our creature untapped");
    assert!(!g.land(l0).tapped, "our land untapped");
    assert!(!g.land(l1[0]).tapped, "(only one land is chosen, ours)");
    assert_eq!(g.player(P0).life, life + 2);
    let _ = l1;
}

#[test]
fn teferi_plus_taps_their_untapped_when_ours_are_untapped() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Teferi, Who Slows the Sunset");
    let theirs = token(&mut g, P1, 3);
    let l1 = lands(&mut g, P1, "Mountain", 1, false)[0];
    walker_eff(&mut g, t, P0, 0);
    assert!(g.perm(theirs).tapped);
    assert!(g.land(l1).tapped, "their land tapped");
}

#[test]
fn teferi_minus_two_takes_one_of_three_and_bottoms_the_rest() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Teferi, Who Slows the Sunset");
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Sol Ring", "Island", "Swamp"]);
    let (h, n) = (g.player(P0).hand.len(), g.player(P0).library.len());
    walker_eff(&mut g, t, P0, 1);
    assert_eq!(g.player(P0).hand.len(), h + 1, "one into the hand");
    assert_eq!(g.player(P0).library.len(), n - 1, "the other two went back (to the bottom)");
    let lib = g.player(P0).library.clone();
    let taken = *g.player(P0).hand.last().unwrap();
    let three = [card(&g, "Sol Ring"), card(&g, "Island"), card(&g, "Swamp")];
    assert!(three.contains(&taken), "the one taken is from the top three");
    assert!(lib[..2].iter().all(|c| three.contains(c) && *c != taken), "the other two are at the bottom");
    assert!(lib[2..].iter().all(|&c| c == card(&g, "Plains")), "the rest is untouched");
}

#[test]
fn teferi_ultimate_records_the_emblem_but_the_ai_does_not_pick_it() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Teferi, Who Slows the Sunset");
    g.perm_mut(t).loyalty = Some(8);
    let o = options(&mut g, t, P0, Some(false));
    assert!(o.iter().all(|x| !x.label.contains("-7")), "the unmodeled ultimate is never offered");
    walker_eff(&mut g, t, P0, 2);
    assert!(g.player(P0).emblems.contains(&"teferi_sunset"));
}
