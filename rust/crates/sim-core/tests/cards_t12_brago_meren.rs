//! The Tier 2 brago-azorius-blink-control and meren-golgari-recursion cards (impls/t12_brago_meren.rs and the compiled
//! pattern under `// --- Tiers 1/2 (brago-meren)`) and their new combos, on hand-built positions. Each expectation
//! follows the card's Oracle text.

use sim_core::engine::cast;
use sim_core::engine::hooks::run_hook;
use sim_core::engine::stack::flush_triggers;
use sim_core::engine::values::epow;
use sim_core::engine::zones;
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::impls::galadriel::eot_returns;
use sim_core::state::{Ctx, DataKey, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const BRAGO: &str = "brago-azorius-blink-control";
const MEREN: &str = "meren-golgari-recursion";

fn brago() -> Game {
    table(&[BRAGO, "seph", "sauron"])
}

fn meren() -> Game {
    table(&[MEREN, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx::default(), "hand").unwrap();
    c
}

fn my_turn(g: &mut Game, p: PlayerId, step: Step) {
    g.active = Some(p);
    g.step = step;
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

// ------------------------------------------------------------------ Agent of Treachery
#[test]
fn agent_of_treachery_takes_the_best_opposing_permanent_and_draws_with_three_stolen() {
    let mut g = brago();
    let small = token(&mut g, P1, 1);
    let big = token(&mut g, P2, 6);
    let agent = perm(&mut g, P0, "Agent of Treachery");
    assert_eq!(g.perm(big).owner, P0, "the 6/6 is taken for good");
    assert_eq!(g.perm(small).owner, P1);
    let hand = g.player(P0).hand.len();
    run_hook(&mut g, agent, Event::EndStep, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand, "one permanent it doesn't own: no draw");
    let a = perm(&mut g, P1, "Sol Ring");
    let b = token(&mut g, P2, 2);
    for m in [a, b] {
        sim_core::engine::control::gain_control(&mut g, P0, m, sim_core::engine::control::Duration::Permanent);
    }
    run_hook(&mut g, agent, Event::EndStep, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 3, "three permanents it doesn't own: draw three");
}

// ------------------------------------------------------------------ Cryogen Relic
#[test]
fn cryogen_relic_draws_on_entering_and_leaving_and_stuns_a_tapped_creature() {
    let mut g = brago();
    let hand = g.player(P0).hand.len();
    let relic = perm(&mut g, P0, "Cryogen Relic");
    assert_eq!(g.player(P0).hand.len(), hand + 1, "enters: draw a card");
    let t = token(&mut g, P1, 5);
    g.perm_mut(t).tapped = true;
    lands(&mut g, P0, "Island", 2, false);
    g.player_mut(P0).hand.clear();
    my_turn(&mut g, P1, Step::End);
    let f = g.registry.get(card(&g, "Cryogen Relic")).and_then(|i| i.options).unwrap();
    let opts = f(&mut g, relic, P0, None).unwrap();
    assert!(!opts.is_empty(), "the end of an opponent's turn: stun their tapped creature");
    assert!(run(&mut g, opts.into_iter().next().unwrap(), P0));
    assert!(!g.perm(relic).on_bf, "sacrificed");
    assert_eq!(g.perm(t).data.int(DataKey::Stun), 1, "a stun counter on the tapped creature");
    assert_eq!(g.player(P0).hand.len(), 1, "leaves: draw a card");
}

// ------------------------------------------------------------------ Dour Port-Mage
#[test]
fn dour_port_mage_draws_once_per_batch_of_creatures_leaving_without_dying() {
    let mut g = brago();
    perm(&mut g, P0, "Dour Port-Mage");
    let (a, b, c) = (token(&mut g, P0, 2), token(&mut g, P0, 2), token(&mut g, P0, 2));
    let hand = g.player(P0).hand.len();
    zones::bounce(&mut g, a).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1, "a creature returned to hand: draw");
    zones::die(&mut g, b, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1, "a creature died: no draw");
    let (d, e) = (token(&mut g, P0, 2), token(&mut g, P0, 2));
    g.resolving += 1;
    zones::exile_perm(&mut g, c).unwrap();
    zones::exile_perm(&mut g, d).unwrap();
    g.resolving -= 1;
    flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 2, "two leave together: one card");
    let x = token(&mut g, P1, 2);
    zones::bounce(&mut g, x).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 2, "an opponent's creature: nothing");
    assert!(g.perm(e).on_bf);
}

// ------------------------------------------------------------------ Flickerwisp
#[test]
fn flickerwisp_exiles_an_opposing_token_for_good() {
    let mut g = brago();
    let t = token(&mut g, P1, 4);
    perm(&mut g, P0, "Flickerwisp");
    assert!(!g.perm(t).on_bf, "the token is exiled");
    eot_returns(&mut g).unwrap();
    assert!(!g.player(P1).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).token), "a token never returns");
}

#[test]
fn flickerwisp_returns_its_card_at_the_beginning_of_the_next_end_step() {
    let mut g = brago();
    let wall = perm(&mut g, P0, "Wall of Omens");
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Flickerwisp");
    assert!(!g.perm(wall).on_bf, "its own enters creature is exiled");
    assert!(on_bf(&g, P0, "Wall of Omens").is_empty());
    eot_returns(&mut g).unwrap();
    assert_eq!(on_bf(&g, P0, "Wall of Omens").len(), 1, "it returns at the end step");
    assert_eq!(g.player(P0).hand.len(), hand + 1, "and enters again: Wall of Omens draws");
}

// ------------------------------------------------------------------ Ghostly Flicker
#[test]
fn ghostly_flicker_blinks_the_two_best_enters_permanents() {
    let mut g = brago();
    let walls = [perm(&mut g, P0, "Wall of Omens"), perm(&mut g, P0, "Mulldrifter")];
    let hand = g.player(P0).hand.len();
    resolve(&mut g, P0, "Ghostly Flicker");
    assert!(walls.iter().all(|&m| !g.perm(m).on_bf), "both exiled");
    assert_eq!(on_bf(&g, P0, "Wall of Omens").len() + on_bf(&g, P0, "Mulldrifter").len(), 2, "and returned");
    assert_eq!(g.player(P0).hand.len(), hand + 3, "Wall of Omens draws one, Mulldrifter two");
}

#[test]
fn ghostly_flicker_is_offered_for_value_with_two_good_targets() {
    let mut g = brago();
    perm(&mut g, P0, "Mulldrifter");
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Ghostly Flicker"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    my_turn(&mut g, P0, Step::Main1);
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "one target is not enough");
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(f(&mut g, c, P0, Some(false)).unwrap().len(), 1);
}

// ------------------------------------------------------------------ Reality Acid
#[test]
fn reality_acid_vanishes_after_three_upkeeps_and_takes_its_permanent() {
    let mut g = brago();
    let ring = perm(&mut g, P1, "Sol Ring");
    let acid = perm(&mut g, P0, "Reality Acid");
    assert_eq!(g.perm(acid).attached, Some(ring), "it enchants the opposing permanent");
    for k in 0..2 {
        run_hook(&mut g, acid, Event::Upkeep, &Call::Player { p: P0 }).unwrap();
        assert!(g.perm(acid).on_bf && g.perm(ring).on_bf, "upkeep {k}: a time counter is removed");
    }
    run_hook(&mut g, acid, Event::Upkeep, &Call::Player { p: P0 }).unwrap();
    assert!(!g.perm(acid).on_bf, "the last time counter: sacrificed");
    assert!(!g.perm(ring).on_bf, "and its permanent's controller sacrifices it");
}

#[test]
fn reality_acid_leaving_early_still_takes_its_permanent() {
    let mut g = brago();
    let t = token(&mut g, P2, 5);
    let acid = perm(&mut g, P0, "Reality Acid");
    assert_eq!(g.perm(acid).attached, Some(t));
    zones::bounce(&mut g, acid).unwrap();
    assert!(!g.perm(t).on_bf, "Acid left the battlefield: the enchanted permanent is sacrificed");
}

// ------------------------------------------------------------------ Riptide Gearhulk
#[test]
fn riptide_gearhulk_puts_each_opponents_best_permanent_third_from_the_top() {
    let mut g = brago();
    let ring = perm(&mut g, P1, "Sol Ring");
    let t = token(&mut g, P2, 5);
    let ring_card = g.perm(ring).cd.unwrap();
    perm(&mut g, P0, "Riptide Gearhulk");
    assert!(!g.perm(ring).on_bf && !g.perm(t).on_bf, "one permanent of each opponent");
    let lib = &g.player(P1).library;
    assert_eq!(lib[lib.len() - 3], ring_card, "third from the top (the top is the end)");
}

// ------------------------------------------------------------------ Starfield Vocalist
#[test]
fn starfield_vocalist_makes_enters_triggers_trigger_again() {
    let mut g = brago();
    perm(&mut g, P0, "Starfield Vocalist");
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(g.player(P0).hand.len(), hand + 2, "Wall of Omens draws twice");
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Mulldrifter");
    assert_eq!(g.player(P0).hand.len(), hand + 4, "the engine's enters effects too");
}

// ------------------------------------------------------------------ Strionic Resonator
#[test]
fn strionic_resonator_copies_an_important_trigger_of_yours() {
    let mut g = brago();
    let res = perm(&mut g, P0, "Strionic Resonator");
    lands(&mut g, P0, "Island", 2, false);
    assert!(sim_core::impls::t12_brago_meren::resonator_ready(&g, P0));
    let (a, b) = (token(&mut g, P1, 6), token(&mut g, P2, 5));
    my_turn(&mut g, P0, Step::Main1);
    perm(&mut g, P0, "Agent of Treachery");
    assert!(g.perm(res).tapped, "the Resonator was used");
    assert_eq!((g.perm(a).owner, g.perm(b).owner), (P0, P0), "the copy takes a second permanent");
}

// ------------------------------------------------------------------ Time Wipe
#[test]
fn time_wipe_returns_the_commander_then_destroys_all_creatures() {
    let mut g = brago();
    let b = perm(&mut g, P0, "Brago, King Eternal");
    g.perm_mut(b).is_cmd = true;
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 4);
    let bc = g.perm(b).cd.unwrap();
    resolve(&mut g, P0, "Time Wipe");
    assert!(!g.perm(b).on_bf && !g.player(P0).gy.contains(&bc), "Brago went back, not to the graveyard");
    assert!(!g.perm(mine).on_bf && !g.perm(theirs).on_bf, "every other creature is destroyed");
}

// ------------------------------------------------------------------ Yorion, Sky Nomad
#[test]
fn yorion_exiles_your_enters_permanents_until_the_end_step() {
    let mut g = brago();
    let wall = perm(&mut g, P0, "Wall of Omens");
    let bear = token(&mut g, P0, 2);
    perm(&mut g, P0, "Yorion, Sky Nomad");
    assert!(!g.perm(wall).on_bf, "Wall of Omens is exiled");
    assert!(g.perm(bear).on_bf, "a token is not");
    let hand = g.player(P0).hand.len();
    eot_returns(&mut g).unwrap();
    assert_eq!(on_bf(&g, P0, "Wall of Omens").len(), 1, "it returns at the end step");
    assert_eq!(g.player(P0).hand.len(), hand + 1);
}

// ------------------------------------------------------------------ Buried Alive
#[test]
fn buried_alive_puts_three_creature_cards_into_the_graveyard() {
    let mut g = meren();
    let creatures = |g: &Game| g.player(P0).gy.iter().filter(|&&c| g.db.get(c).creature).count();
    let c = take(&mut g, P0, "Buried Alive");
    let (before, lib) = (creatures(&g), g.player(P0).library.len());
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!(creatures(&g), before + 3, "three creature cards");
    assert_eq!(g.player(P0).library.len(), lib - 3, "out of the library");
    assert!(g.player(P0).gy.contains(&card(&g, "Buried Alive")));
    let gy: Vec<CardId> = g.player(P0).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    assert!(gy.iter().all(|c| g.t12_gy.contains(c)), "Twilight Diviner's record sees them");
}

// ------------------------------------------------------------------ Journey to Eternity
#[test]
fn journey_to_eternity_returns_its_creature_and_becomes_atzal() {
    let mut g = meren();
    let host = perm(&mut g, P0, "Elvish Mystic");
    let j = perm(&mut g, P0, "Journey to Eternity // Atzal, Cave of Eternity");
    assert_eq!(g.perm(j).attached, Some(host));
    zones::die(&mut g, host, "destroy").unwrap();
    assert_eq!(on_bf(&g, P0, "Elvish Mystic").len(), 1, "the creature returns");
    let jc = card(&g, "Journey to Eternity // Atzal, Cave of Eternity");
    let atzal = g.player(P0).lands.iter().copied().find(|&l| g.land(l).cd == jc).expect("Atzal is a land now");
    assert!(sim_core::engine::mana::land_colors_now(&g, P0, atzal).count() >= 2, "any colour");
    // {3}{B}{G}, {T}: return a creature card from the graveyard
    to_gy(&mut g, P0, "Massacre Wurm");
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Forest", 2, false);
    let f = g.registry.get(jc).and_then(|i| i.land_options).unwrap();
    let opts = f(&mut g, atzal, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(run(&mut g, opts.into_iter().next().unwrap(), P0));
    assert_eq!(on_bf(&g, P0, "Massacre Wurm").len(), 1);
    assert!(g.land(atzal).tapped);
}

// ------------------------------------------------------------------ Twilight Diviner
#[test]
fn twilight_diviner_copies_a_creature_that_came_back_from_the_graveyard_once_a_turn() {
    let mut g = meren();
    perm(&mut g, P0, "Twilight Diviner");
    let elf = perm(&mut g, P0, "Elvish Mystic");
    assert_eq!(on_bf(&g, P0, "Elvish Mystic").len(), 1, "cast or put from the library: no copy");
    zones::die(&mut g, elf, "destroy").unwrap();
    let c = card(&g, "Elvish Mystic");
    let gy = &mut g.player_mut(P0).gy;
    let i = gy.iter().position(|&x| x == c).unwrap();
    gy.remove(i);
    zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    let elves = on_bf(&g, P0, "Elvish Mystic");
    assert_eq!(elves.len(), 2, "it came back from the graveyard: a token copy");
    assert_eq!(elves.iter().filter(|&&m| g.perm(m).token).count(), 1);
    let wood = perm(&mut g, P0, "Llanowar Elves");
    zones::die(&mut g, wood, "destroy").unwrap();
    let c = card(&g, "Llanowar Elves");
    let gy = &mut g.player_mut(P0).gy;
    let i = gy.iter().position(|&x| x == c).unwrap();
    gy.remove(i);
    zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(on_bf(&g, P0, "Llanowar Elves").len(), 1, "only once each turn");
}

// ------------------------------------------------------------------ Wight of the Reliquary (compiled)
#[test]
fn wight_of_the_reliquary_grows_with_creature_cards_in_your_graveyard() {
    let mut g = meren();
    let w = perm(&mut g, P0, "Wight of the Reliquary");
    let base = epow(&g, w);
    to_gy(&mut g, P0, "Elvish Mystic");
    to_gy(&mut g, P0, "Massacre Wurm");
    to_gy(&mut g, P0, "Dark Ritual");
    assert_eq!(epow(&g, w), base + 2, "+1/+1 for each creature card, not the sorcery");
}

// ------------------------------------------------------------------ combos
fn combo_ready(g: &Game, p: PlayerId, part: &str) -> bool {
    combos::ready(g, p, combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part:?}")))
}

#[test]
fn peregrine_drake_and_deadeye_navigator_is_a_combo_with_agent_of_treachery() {
    let mut g = brago();
    let part = "Peregrine Drake + Deadeye Navigator";
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Deadeye Navigator");
    assert!(!combo_ready(&g, P0, part), "unbounded mana alone does not win");
    perm(&mut g, P0, "Agent of Treachery");
    assert!(combo_ready(&g, P0, part));
}

#[test]
fn peregrine_drake_archaeomancer_and_ghostly_flicker_is_a_combo() {
    let mut g = brago();
    let part = "Peregrine Drake + Archaeomancer + Ghostly Flicker";
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Island", 4, false); // --- casual combos (brago): three lands only pay for Flicker
    assert!(!combo_ready(&g, P0, part), "Ghostly Flicker is needed in hand");
    hand(&mut g, P0, &["Ghostly Flicker"]);
    assert!(combo_ready(&g, P0, part));
}
