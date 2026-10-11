//! The jenova candidate deck's cards (impls/jenova.rs), on hand-built positions. Seat 0 plays the jenova deck, seat 1
//! the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::combat::kw;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, hooks, life, turn, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["jenova", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn take_option(g: &mut Game, p: PlayerId, post: bool, label: &str) -> bool {
    let opts = options(g, p, post);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(label))
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn combat_start(g: &mut Game, p: PlayerId) {
    hooks::fire_trigger(g, Event::Crew, Call::Player { p }).unwrap();
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn is_mutant(g: &Game, m: PermId) -> bool {
    g.perm(m).ttypes.contains(&"mutant")
}

fn tokens_named(g: &Game, p: PlayerId, ty: &'static str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).collect()
}

// ------------------------------------------------------------------ Jenova
#[test]
fn jenova_puts_counters_equal_to_her_power_and_makes_a_mutant() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(j).plus = 2; // power 3
    assert_eq!(epow(&g, j), 3);
    combat_start(&mut g, P0);
    assert_eq!(g.perm(elf).plus, 3);
    assert!(is_mutant(&g, elf));
    assert!(!is_mutant(&g, j), "'other target creature'");
    // her power is read as the trigger resolves: it grows with her
    g.perm_mut(j).plus = 5;
    combat_start(&mut g, P0);
    assert_eq!(g.perm(elf).plus, 3 + 6);
}

#[test]
fn jenova_triggers_only_on_her_controllers_turn() {
    let mut g = game();
    let _j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.active = Some(P1);
    combat_start(&mut g, P1);
    assert_eq!(g.perm(elf).plus, 0);
}

#[test]
fn jenova_prefers_an_evasive_creature() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let _bear = perm(&mut g, P0, "Penumbra Spider");
    let tok = zones::make_tokens(&mut g, P0, zones::Tokens { fly: true, sick: false, ..zones::Tokens::new(1, 1) })
        .unwrap()[0];
    combat_start(&mut g, P0);
    assert!(g.perm(tok).plus > 0 || is_mutant(&g, tok) || epow(&g, j) > 0);
    assert!(is_mutant(&g, tok), "the flyer is the better Mutant");
}

#[test]
fn a_mutant_dying_on_your_turn_draws_cards_equal_to_its_power() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(j).plus = 2;
    combat_start(&mut g, P0);
    let power = epow(&g, elf);
    assert_eq!(power, 3);
    let before = g.player(P0).hand.len();
    zones::die(&mut g, elf, "sac").unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 3);
}

#[test]
fn a_mutant_dying_on_another_turn_draws_nothing_and_a_plain_creature_neither() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    g.perm_mut(j).plus = 2;
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Pest Mascot");
    combat_start(&mut g, P0);
    let before = g.player(P0).hand.len();
    let (mutant, plain) = if is_mutant(&g, a) { (a, b) } else { (b, a) };
    zones::die(&mut g, plain, "sac").unwrap();
    assert_eq!(g.player(P0).hand.len(), before);
    g.active = Some(P1);
    zones::die(&mut g, mutant, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before);
}

#[test]
fn a_mutant_is_cheap_to_sacrifice_on_your_turn_only() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let tok = token(&mut g, P0, 1);
    g.perm_mut(j).plus = 3;
    combat_start(&mut g, P0);
    assert!(is_mutant(&g, elf) || is_mutant(&g, tok));
    let mutant = if is_mutant(&g, elf) { elf } else { tok };
    assert!(zones::sac_worth(&g, mutant) < 0.0, "a Mutant of power 4+ on my turn is worth less than nothing");
    g.active = Some(P1);
    assert!(zones::sac_worth(&g, mutant) > 0.5);
}

#[test]
fn jenova_ai_sacrifices_a_big_mutant_to_carrion_feeder_after_combat() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let _feeder = perm(&mut g, P0, "Carrion Feeder");
    let big = perm(&mut g, P0, "Hungry Graffalon");
    g.perm_mut(j).plus = 3;
    combat_start(&mut g, P0);
    assert!(is_mutant(&g, big));
    let before = g.player(P0).hand.len();
    assert!(take_option(&mut g, P0, true, "sacrifice the Mutant"));
    assert!(!g.perm(big).on_bf);
    assert!(g.player(P0).hand.len() >= before + 4, "drew its power");
}

// ------------------------------------------------------------------ Auras
#[test]
fn forced_adaptation_adds_a_counter_each_upkeep_and_goes_on_jenova() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let _elf = perm(&mut g, P0, "Llanowar Elves");
    let a = perm(&mut g, P0, "Forced Adaptation");
    assert_eq!(g.perm(a).attached, Some(j), "the AI enchants her");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.perm(j).plus, 1);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.perm(j).plus, 2);
}

#[test]
fn lion_umbra_needs_a_modified_creature_and_gives_its_bonuses() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let a = perm(&mut g, P0, "Lion Umbra");
    assert!(g.perm(a).attached.is_none(), "an unmodified creature can't be enchanted");
    assert!(in_gy(&g, P0, "Lion Umbra"));
    g.perm_mut(elf).plus = 1;
    let a = perm(&mut g, P0, "Lion Umbra");
    assert_eq!(g.perm(a).attached, Some(elf));
    assert_eq!(epow(&g, elf), 0 + 1 + 3); // Llanowar Elves is 0/1 in the card data
    assert_eq!(etgh(&g, elf), 1 + 1 + 3);
    assert!(kw(&g, elf, "reach"));
    assert!(kw(&g, elf, "vigilance"));
}

#[test]
fn lion_umbra_is_destroyed_instead_of_the_creature() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(elf).plus = 1;
    let a = perm(&mut g, P0, "Lion Umbra");
    zones::die(&mut g, elf, "destroy").unwrap();
    assert!(g.perm(elf).on_bf, "umbra armor");
    assert!(!g.perm(a).on_bf);
    assert!(in_gy(&g, P0, "Lion Umbra"));
}

#[test]
fn umbra_armor_also_saves_from_lethal_combat_damage() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(elf).plus = 1;
    let a = perm(&mut g, P0, "Lion Umbra");
    zones::die(&mut g, elf, "combat").unwrap();
    assert!(g.perm(elf).on_bf);
    assert!(!g.perm(a).on_bf);
}

// ------------------------------------------------------------------ increment
#[test]
fn increment_grows_a_creature_when_the_mana_spent_is_greater_than_its_smaller_stat() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer"); // 1/2
    let graf = perm(&mut g, P0, "Hungry Graffalon"); // 3/4
    cast_spell(&mut g, P0, "Rampant Growth"); // 2 mana
    assert_eq!(g.perm(lec).plus, 1, "2 > 1");
    assert_eq!(g.perm(graf).plus, 0, "2 is not greater than 3");
    cast_spell(&mut g, P0, "Cultivate"); // 3 mana: the Lecturer is now 2/3
    assert_eq!(g.perm(lec).plus, 2, "3 > 2");
    assert_eq!(g.perm(graf).plus, 0, "3 is not greater than 3");
    cast_spell(&mut g, P0, "Skyshroud Claim"); // 4 mana
    assert_eq!(g.perm(graf).plus, 1);
    assert_eq!(g.perm(lec).plus, 3);
}

#[test]
fn increment_ignores_spells_cast_by_opponents() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer");
    let c = hand(&mut g, P1, &["Terror"])[0];
    cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.perm(lec).plus, 0);
}

#[test]
fn topiary_lecturer_taps_for_mana_equal_to_its_power() {
    let mut g = game();
    let lec = perm(&mut g, P0, "Topiary Lecturer");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, lec), Some(1));
    g.perm_mut(lec).plus = 3;
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, lec), Some(4));
}

#[test]
fn ambitious_augmenter_leaves_a_fractal_with_its_counters() {
    let mut g = game();
    let aug = perm(&mut g, P0, "Ambitious Augmenter");
    g.perm_mut(aug).plus = 3;
    zones::die(&mut g, aug, "destroy").unwrap();
    let f = tokens_named(&g, P0, "fractal");
    assert_eq!(f.len(), 1);
    assert_eq!(g.perm(f[0]).plus, 3);
    assert_eq!(epow(&g, f[0]), 3);
    assert_eq!(etgh(&g, f[0]), 3);
}

#[test]
fn ambitious_augmenter_without_counters_leaves_nothing_and_has_no_etb_token() {
    let mut g = game();
    let aug = perm(&mut g, P0, "Ambitious Augmenter");
    assert!(tokens_named(&g, P0, "fractal").is_empty(), "entering makes no token");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 0);
    zones::die(&mut g, aug, "destroy").unwrap();
    assert!(tokens_named(&g, P0, "fractal").is_empty());
}

// ------------------------------------------------------------------ infusion and life
#[test]
fn old_growth_educator_gets_two_counters_only_after_life_gain() {
    let mut g = game();
    let a = perm(&mut g, P0, "Old-Growth Educator");
    assert_eq!(g.perm(a).plus, 0);
    life::gain(&mut g, P0, 1).unwrap();
    let b = perm(&mut g, P0, "Old-Growth Educator");
    assert_eq!(g.perm(b).plus, 2);
}

#[test]
fn efflorescence_puts_two_counters_and_the_infusion_needs_life_gained() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let _elf = perm(&mut g, P0, "Llanowar Elves");
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Efflorescence", "Efflorescence"]);
    assert!(take_option(&mut g, P0, false, "Efflorescence"));
    assert_eq!(g.perm(j).plus, 2, "on Jenova");
    assert!(!kw(&g, j, "trample") && !kw(&g, j, "indestructible"));
    life::gain(&mut g, P0, 1).unwrap();
    lands(&mut g, P0, "Forest", 3, false);
    assert!(take_option(&mut g, P0, false, "Efflorescence"));
    assert_eq!(g.perm(j).plus, 4);
    assert!(kw(&g, j, "trample") && kw(&g, j, "indestructible"));
}

#[test]
fn adventurous_eater_enters_prepared_and_casts_a_copy_of_have_a_bite() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let eater = perm(&mut g, P0, "Adventurous Eater");
    assert!(g.perm(eater).data.truthy(DataKey::Prepared));
    lands(&mut g, P0, "Swamp", 1, false);
    let life0 = g.player(P0).life;
    assert!(take_option(&mut g, P0, false, "Have a Bite"));
    assert_eq!(g.perm(j).plus, 1);
    assert_eq!(g.player(P0).life, life0 + 1);
    assert!(!g.perm(eater).data.truthy(DataKey::Prepared), "casting the copy unprepares it");
    assert_eq!(g.player(P0).spells_this_turn, 1, "a copy is a cast");
}

#[test]
fn essenceknit_scholar_makes_a_pest_that_gains_life_attacking() {
    let mut g = game();
    let _s = perm(&mut g, P0, "Essenceknit Scholar");
    let pests = tokens_named(&g, P0, "pest");
    assert_eq!(pests.len(), 1);
    assert_eq!((g.perm(pests[0]).pow, g.perm(pests[0]).tgh), (1, 1));
    let life0 = g.player(P0).life;
    sim_core::cardcode::keyword_attack(&mut g, P0, &pests, P1).unwrap();
    assert_eq!(g.player(P0).life, life0 + 1);
}

#[test]
fn essenceknit_scholar_draws_at_the_end_step_if_a_creature_died() {
    let mut g = game();
    let _s = perm(&mut g, P0, "Essenceknit Scholar");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let n = g.player(P0).hand.len();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "nothing died");
    zones::die(&mut g, elf, "destroy").unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn essenceknit_scholar_does_not_count_an_opponents_creature() {
    let mut g = game();
    let _s = perm(&mut g, P0, "Essenceknit Scholar");
    let theirs = token(&mut g, P1, 2);
    zones::die(&mut g, theirs, "destroy").unwrap();
    let n = g.player(P0).hand.len();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
}

// ------------------------------------------------------------------ Edgar, Teacher's Pest, Vinelasher Adept, Grove
#[test]
fn edgar_grows_by_two_when_you_cast_nothing() {
    let mut g = game();
    let e = perm(&mut g, P0, "Edgar, Moonlit Sovereign");
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(e).plus, 2);
    g.player_mut(P0).spells_this_turn = 1;
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(e).plus, 2, "a spell was cast");
}

#[test]
fn edgar_activation_adds_a_counter_to_each_creature_with_one() {
    let mut g = game();
    let e = perm(&mut g, P0, "Edgar, Moonlit Sovereign");
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Pest Mascot");
    let c = perm(&mut g, P0, "Penumbra Spider");
    for m in [e, a, b] {
        g.perm_mut(m).plus = 1;
    }
    g.perm_mut(c).plus = 0;
    lands(&mut g, P0, "Forest", 5, false);
    assert!(take_option(&mut g, P0, true, "Edgar"));
    assert_eq!((g.perm(e).plus, g.perm(a).plus, g.perm(b).plus, g.perm(c).plus), (2, 2, 2, 0));
}

#[test]
fn teachers_pest_returns_from_the_graveyard_tapped() {
    let mut g = game();
    let _j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let c = take(&mut g, P0, "Teacher's Pest");
    g.player_mut(P0).gy.push(c);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(take_option(&mut g, P0, true, "Teacher's Pest"));
    assert!(!in_gy(&g, P0, "Teacher's Pest"));
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Teacher's Pest").unwrap();
    assert!(g.perm(m).tapped);
}

#[test]
fn vinelasher_adept_landcycles_for_two() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Vinelasher Adept"]);
    let lands_before = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count();
    assert!(take_option(&mut g, P0, false, "Vinelasher Adept (basic landcycling)"));
    assert!(in_gy(&g, P0, "Vinelasher Adept"));
    let basic = g
        .player(P0)
        .hand
        .iter()
        .filter(|&&c| g.db.get(c).land && ["Forest", "Swamp"].contains(&&*g.db.get(c).name))
        .count();
    assert_eq!(basic, lands_before + 1);
}

#[test]
fn vinelasher_adept_puts_three_counters_on_jenova() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity");
    let _a = perm(&mut g, P0, "Vinelasher Adept");
    assert_eq!(g.perm(j).plus, 3);
}

#[test]
fn dormant_grove_grows_a_creature_and_transforms_at_toughness_six() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jenova, Ancient Calamity"); // 1/5
    let grove = perm(&mut g, P0, "Dormant Grove");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    assert!(!g.is_creature(grove));
    combat_start(&mut g, P0);
    assert_eq!(g.perm(j).plus, 1, "the counter goes on the best creature");
    assert_eq!(etgh(&g, j), 6);
    assert!(g.is_creature(grove), "transformed");
    assert_eq!((epow(&g, grove), etgh(&g, grove)), (3, 6));
    assert!(kw(&g, elf, "vigilance"));
}

#[test]
fn dormant_grove_stays_a_grove_below_toughness_six() {
    let mut g = game();
    let grove = perm(&mut g, P0, "Dormant Grove");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    combat_start(&mut g, P0);
    assert_eq!(g.perm(elf).plus, 1);
    assert!(!g.is_creature(grove));
}

// ------------------------------------------------------------------ whole games
#[test]
fn the_jenova_deck_plays_whole_games() {
    for seed in [1u64, 2, 3] {
        let g = play(&["jenova", "seph", "veyran", "kefka"], seed, false);
        assert!(g.round > 1);
    }
}

#[allow(dead_code)]
fn _unused(_: CardId) {}
