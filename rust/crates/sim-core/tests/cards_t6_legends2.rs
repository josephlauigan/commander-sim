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
    // that opponent drew on our turn: Scions' Secretary made a tapped Treasure
    assert_eq!((g.player(P0).treasures, g.player(P0).tapped_treasures), (1, 1), "a tapped Treasure");
    assert_eq!(g.player(P0).live_treasures(), 0, "no mana from it yet");
}

#[test]
fn scions_secretary_once_a_turn_and_not_on_their_own_turn() {
    let mut g = tbl();
    // every library fixed: what Tataru's enter trigger draws must not depend on the decks' shuffles (with Ertai
    // Resurrected drawn into its owner's hand, the Treasure never comes: tataru_with_ertai_in_hand below)
    for i in 0..4 {
        set_library(&mut g, PlayerId(i), &["Plains"; 6]);
    }
    let t = perm(&mut g, P0, "Tataru Taru");
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    let base = g.player(P0).treasures;
    set_library(&mut g, P1, &["Mountain", "Mountain", "Mountain", "Mountain"]);
    g.active = Some(P1);
    sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).treasures, base, "their own turn");
    g.active = Some(P0);
    g.round = 2; // (a new turn: the Treasure from the enter-trigger draw used the first turn's)
    let base = g.player(P0).treasures;
    for _ in 0..3 {
        sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
        sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    }
    assert_eq!(g.player(P0).treasures, base + 1, "once each turn");
    assert_eq!(g.player(P0).tapped_treasures, g.player(P0).treasures, "all tapped");
    let _ = t;
}

/// With Ertai Resurrected in its owner's hand, Tataru Taru still makes its Treasure when an opponent draws off their
/// turn. (Fixed 2026-10-26: the trigger probe that runs when anyone holds an ability answer used up `once_per_turn`.)
#[test]
fn tataru_with_ertai_in_hand() {
    let mut g = tbl();
    for i in 0..4 {
        set_library(&mut g, PlayerId(i), &["Plains"; 6]);
    }
    set_library(&mut g, P0, &["Plains", "Plains", "Ertai Resurrected"]); // the top is last: Tataru's draw takes Ertai
    perm(&mut g, P0, "Tataru Taru");
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Ertai Resurrected"));
    assert_eq!(g.player(P0).treasures, 1, "the opponent's draw on our turn makes a Treasure");
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

// ======================================================== Spider-Woman, Stunning Savior
#[test]
fn spider_woman_makes_opposing_artifacts_and_creatures_enter_tapped() {
    let mut g = tbl();
    perm(&mut g, P0, "Spider-Woman, Stunning Savior");
    let ring = perm(&mut g, P1, "Sol Ring");
    assert!(g.perm(ring).tapped, "an opposing artifact enters tapped");
    let tok = token(&mut g, P1, 2);
    assert!(g.perm(tok).tapped, "an opposing creature token enters tapped");
    let creature = perm(&mut g, P1, "Goblin Guide");
    assert!(g.perm(creature).tapped, "an opposing creature enters tapped");
    let mine = perm(&mut g, P0, "Sol Ring");
    assert!(!g.perm(mine).tapped, "our own artifact is untouched");
    let own_tok = token(&mut g, P0, 2);
    assert!(!g.perm(own_tok).tapped);
    sim_core::engine::zones::add_treasure(&mut g, P1, 1).unwrap();
    assert_eq!(g.player(P1).tapped_treasures, 1, "an opposing Treasure enters tapped");
    let enchant = perm(&mut g, P1, "Rhystic Study");
    assert!(!g.perm(enchant).tapped, "an enchantment is not an artifact or creature");
}

#[test]
fn spider_woman_gone_means_untapped_again() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Spider-Woman, Stunning Savior");
    sim_core::engine::zones::die(&mut g, s, "destroy").unwrap();
    let ring = perm(&mut g, P1, "Sol Ring");
    assert!(!g.perm(ring).tapped);
}

// ======================================================== Ertai Resurrected
use sim_core::state::{Ctx, StackItem, StackKind};

fn on_stack(g: &mut Game, p: PlayerId, name: &str, kind: StackKind) -> u32 {
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    let c = if kind == StackKind::Spell { Some(card(g, name)) } else { None };
    g.stack.push(StackItem {
        id,
        controller: p,
        card: c,
        ctx: Ctx::default(),
        zone: "hand",
        imp: 7.0,
        aff: vec![],
        generic: true,
        kind,
        name: name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    id
}

#[test]
fn ertai_enters_and_destroys_their_best_creature_its_controller_draws() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Lyra Dawnbringer");
    let h = g.player(P1).hand.len();
    perm(&mut g, P0, "Ertai Resurrected");
    assert!(!g.perm(big).on_bf, "destroyed");
    assert_eq!(g.player(P1).hand.len(), h + 1, "its controller drew a card");
}

#[test]
fn ertai_with_nothing_worth_destroying_chooses_no_mode() {
    let mut g = tbl();
    let small = token(&mut g, P1, 1);
    let h = g.player(P1).hand.len();
    perm(&mut g, P0, "Ertai Resurrected");
    assert!(g.perm(small).on_bf, "up to one mode: none chosen");
    assert_eq!(g.player(P1).hand.len(), h);
}

#[test]
fn ertai_flashed_in_against_a_spell_counters_it_and_its_controller_draws() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let ertai = hand(&mut g, P0, &["Ertai Resurrected"])[0];
    let id = on_stack(&mut g, P1, "Grave Titan", StackKind::Spell);
    let h = g.player(P1).hand.len();
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ertai, id).unwrap());
    assert!(g.stack.iter().find(|x| x.id == id).unwrap().countered);
    assert!(has_perm(&g, P0, "Ertai Resurrected"), "it entered");
    assert_eq!(g.player(P1).hand.len(), h + 1, "the spell's controller drew a card (and no creature was destroyed)");
}

#[test]
fn ertai_answers_an_ability_when_the_engine_asks() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Ertai Resurrected"]);
    let bystander = perm(&mut g, P1, "Lyra Dawnbringer");
    let id = on_stack(&mut g, P1, "Godo, Bandit Warlord: fetch Helm of the Host", StackKind::Trigger);
    let h = g.player(P1).hand.len();
    assert!(sim_core::impls::t6_legends2::ertai_answer(&mut g, P0, id).unwrap());
    assert!(g.stack.iter().find(|x| x.id == id).unwrap().countered, "the trigger is countered");
    assert!(has_perm(&g, P0, "Ertai Resurrected"));
    assert_eq!(g.player(P1).hand.len(), h + 1, "its controller drew a card");
    assert!(g.perm(bystander).on_bf, "only one mode");
    // no Ertai, no mana, or a spell on the stack: nothing happens
    let id2 = on_stack(&mut g, P1, "x", StackKind::Trigger);
    assert!(!sim_core::impls::t6_legends2::ertai_answer(&mut g, P0, id2).unwrap());
}

// ======================================================== Legolas's Quick Reflexes
#[test]
fn legolas_untaps_and_gives_reach_and_hexproof_until_end_of_turn() {
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    g.perm_mut(t).tapped = true;
    let c = hand(&mut g, P0, &["Legolas's Quick Reflexes"])[0];
    sim_core::engine::cast::resolve(&mut g, P0, c, &Ctx { target: Some(t), ..Ctx::default() }, "hand").unwrap();
    assert!(!g.perm(t).tapped, "untapped");
    assert!(g.perm(t).eot_kw.iter().any(|k| &**k == "reach"));
    assert!(sim_core::engine::values::untargetable(&g, t), "hexproof: opponents can't target it");
    assert!(g.player(P0).gy.contains(&c), "the instant goes to the graveyard");
}

#[test]
fn legolas_cast_as_a_response_costs_g() {
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    let c = hand(&mut g, P0, &["Legolas's Quick Reflexes"])[0];
    assert!(!sim_core::impls::t6_legends2::legolas_cast(&mut g, P0, c, t).unwrap(), "no {{G}}");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(sim_core::impls::t6_legends2::legolas_cast(&mut g, P0, c, t).unwrap());
    assert!(sim_core::engine::values::untargetable(&g, t));
    assert!(g.player(P0).gy.contains(&c));
}

// ======================================================== Colossal Skyturtle
fn turtle_options(g: &mut Game, c: CardId, p: PlayerId) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, Some(false)).unwrap()
}

fn run_plan(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

#[test]
fn skyturtle_channel_bounces_their_best_creature_for_1u() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Colossal Skyturtle"])[0];
    assert!(turtle_options(&mut g, c, P0).is_empty(), "nothing worth bouncing");
    let big = perm(&mut g, P1, "Lyra Dawnbringer");
    let o = turtle_options(&mut g, c, P0);
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("bounce"));
    assert!(run_plan(&mut g, P0, &o[0]));
    assert!(!g.perm(big).on_bf, "returned to its owner's hand");
    assert!(g.player(P1).hand.contains(&card(&g, "Lyra Dawnbringer")));
    assert!(g.player(P0).gy.contains(&c), "the turtle was discarded as the cost");
    assert!(!g.player(P0).hand.contains(&c));
}

#[test]
fn skyturtle_channel_returns_a_graveyard_card_for_2g() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Colossal Skyturtle"])[0];
    let sol = card(&g, "Sol Ring");
    g.player_mut(P0).gy.push(sol);
    assert!(sim_core::impls::t6_legends2::turtle_regrow_go(&mut g, P0, sol.0 as i64).unwrap());
    assert!(g.player(P0).hand.contains(&sol), "Sol Ring is back in hand");
    assert!(!g.player(P0).gy.contains(&sol));
    assert!(g.player(P0).gy.contains(&c));
    // too little mana: nothing happens
    let mut g = tbl();
    let c = hand(&mut g, P0, &["Colossal Skyturtle"])[0];
    g.player_mut(P0).gy.push(sol);
    assert!(!sim_core::impls::t6_legends2::turtle_regrow_go(&mut g, P0, sol.0 as i64).unwrap());
    assert!(g.player(P0).hand.contains(&c));
}

// ======================================================== Wan Shi Tong, Librarian
#[test]
fn wan_shi_tong_x_counters_and_draws_half_x() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 8, false);
    let c = hand(&mut g, P0, &["Wan Shi Tong, Librarian"])[0];
    let o = {
        let f = g.registry.get(c).unwrap().hand_options.unwrap();
        assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "flash: held for the end-of-turn window");
        f(&mut g, c, P0, None).unwrap()
    };
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Wan Shi Tong X=6");
    let h = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    let w = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Wan Shi Tong, Librarian").unwrap();
    assert_eq!((epow(&g, w), sim_core::engine::values::etgh(&g, w)), (7, 7), "1/1 plus X = 6 counters");
    assert_eq!(g.player(P0).hand.len(), h - 1 + 3, "left the hand, drew half of X");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "all eight mana spent");
}

#[test]
fn wan_shi_tong_is_not_cast_for_nothing() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Wan Shi Tong, Librarian"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    assert!(f(&mut g, c, P0, None).unwrap().is_empty(), "X would be 1");
    assert_eq!(g.registry.get(c).unwrap().prio.unwrap()(&g, P0, c), 0, "the AI never casts it for X = 0");
}

#[test]
fn wan_shi_tong_grows_and_draws_when_an_opponent_searches() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wan Shi Tong, Librarian");
    let h = g.player(P0).hand.len();
    let p = epow(&g, w);
    sim_core::impls::t6_legends2::wan_shi_tong_searched(&mut g, P1).unwrap();
    assert_eq!(epow(&g, w), p + 1);
    assert_eq!(g.player(P0).hand.len(), h + 1);
    sim_core::impls::t6_legends2::wan_shi_tong_searched(&mut g, P0).unwrap(); // our own search: nothing
    assert_eq!(epow(&g, w), p + 1);
}
