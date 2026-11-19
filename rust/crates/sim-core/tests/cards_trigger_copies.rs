//! "Triggers an additional time" effects add up: with n such effects a trigger happens 1 + n times (CR 603.2d, and the
//! Panharmonicon ruling: two Panharmonicons give three triggers, not four). The event-based sources (Panharmonicon,
//! Starfield Vocalist, Ancient Greenwarden, Teysa Karlov, Isshin) and the controller-based ones (Cloud, Delney, Roaming
//! Throne) are counted together: the old engine multiplied the second kind into the first.

use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{combat, turn, zones};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const DELNEY: &str = "Delney, Streetwise Lookout";
const PANHARM: &str = "Panharmonicon";
const VOCALIST: &str = "Starfield Vocalist";
const Y: &str = "Yoshimaru, Ever Faithful";

fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "sauron"])
}

fn brago() -> Game {
    table(&["brago-azorius-blink-control", "seph", "sauron"])
}

fn land(g: &mut Game, name: &str) -> PermId {
    let c = take(g, P0, name);
    turn::play_land_card(g, P0, c, "plays").unwrap();
    *g.player(P0).lands.last().unwrap()
}

fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

/// the triggers of `src` waiting in the queue
fn waiting(g: &Game, src: PermId) -> usize {
    g.trig_queue.iter().filter(|t| t.src == Some(src)).count()
}

/// Cloud with an Equipment attached (it has no stat boost, so Cloud stays a 2/1)
fn equipped_cloud(g: &mut Game) -> PermId {
    let cl = perm(g, P0, "Cloud, Midgar Mercenary");
    let mask = perm(g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(cl);
    cl
}

/// Cloud's enters trigger as if he had just entered, counted in the queue (his entry itself already happened)
fn cloud_enters_again(g: &mut Game, cl: PermId) -> usize {
    g.resolving = 1;
    fire_trigger(g, Event::Etb, Call::Etb { p: P0, m: cl }).unwrap();
    let n = waiting(g, cl);
    g.trig_queue.clear();
    g.resolving = 0;
    n
}

// ------------------------------------------------------------------ Delney with the event-based sources
#[test]
fn delney_and_panharmonicon_make_three_triggers_not_four() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, PANHARM);
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Tymna the Weaver"); // a legendary creature enters
    assert_eq!(g.perm(y).plus, 3, "1 + Panharmonicon + Delney");
}

#[test]
fn delney_and_a_vocalist_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, VOCALIST);
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(g.perm(y).plus, 3, "1 + Vocalist + Delney");
}

#[test]
fn delney_and_a_vocalist_on_a_legendary_land_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, VOCALIST);
    let y = perm(&mut g, P0, Y);
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 3, "the land-enters path: 1 + Vocalist + Delney");
}

#[test]
fn delney_and_two_vocalists_make_four_triggers_not_six() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, VOCALIST);
    perm(&mut g, P0, VOCALIST);
    let y = perm(&mut g, P0, Y);
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 4, "1 + two Vocalists + Delney");
}

#[test]
fn delney_and_panharmonicon_on_a_compiled_enters_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, "Soul Warden"); // compiled: another creature enters, gain 1 life
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 3, "1 + Panharmonicon + Delney");
}

#[test]
fn delney_and_a_vocalist_on_an_engine_enters_trigger() {
    let mut g = brago();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, VOCALIST);
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Mulldrifter"); // the engine's enters tag: draw two, once per trigger
    assert_eq!(g.player(P0).hand.len(), hand + 6, "1 + Vocalist + Delney = 3 triggers of 2 cards");
}

// ------------------------------------------------------------------ Roaming Throne
#[test]
fn roaming_throne_and_panharmonicon_make_three_triggers() {
    let mut g = table(&["miirym-temur-dragons", "urza-mono-blue-artifacts", "kinnan-simic-mana-combo", "seph"]);
    g.player_mut(P0).hand.clear();
    let t = perm(&mut g, P0, "Roaming Throne");
    assert_eq!(g.perm(t).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, "Lathliss, Dragon Queen"); // another nontoken Dragon enters: a 5/5 Dragon token
    let before = tokens(&g, P0);
    perm(&mut g, P0, "Ancient Gold Dragon");
    assert_eq!(tokens(&g, P0) - before, 3, "1 + Panharmonicon + Throne");
}

#[test]
fn delney_and_roaming_throne_make_three_triggers() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(t).data.set(DataKey::Ctype, Val::Str("human"));
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Soul Warden"); // a 1/1 Human
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 3, "1 + Throne + Delney");
}

// ------------------------------------------------------------------ Cloud
#[test]
fn cloud_and_panharmonicon_make_three_triggers() {
    let mut g = tbl();
    let cl = equipped_cloud(&mut g);
    assert_eq!(cloud_enters_again(&mut g, cl), 2, "1 + Cloud");
    perm(&mut g, P0, PANHARM);
    assert_eq!(cloud_enters_again(&mut g, cl), 3, "1 + Panharmonicon + Cloud");
}

#[test]
fn delney_and_cloud_make_three_triggers() {
    let mut g = tbl();
    let cl = equipped_cloud(&mut g);
    perm(&mut g, P0, DELNEY);
    assert_eq!(cloud_enters_again(&mut g, cl), 3, "1 + Cloud + Delney");
}

#[test]
fn cloud_delney_and_throne_make_four_triggers_not_eight() {
    let mut g = tbl();
    let cl = equipped_cloud(&mut g);
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(t).data.set(DataKey::Ctype, Val::Str("mercenary"));
    assert_eq!(cloud_enters_again(&mut g, cl), 4, "1 + Cloud + Delney + Throne");
}

// ------------------------------------------------------------------ the loops that fire an event once per copy
#[test]
fn delney_and_isshin_make_three_attack_triggers_for_a_hook() {
    let mut g = table(&["isshin-mardu-attack-triggers", "seph", "cloud"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    perm(&mut g, P0, DELNEY);
    let h = perm(&mut g, P0, "Hanweir Garrison"); // 2/3: two attacking 1/1 Humans
    let before = tokens(&g, P0);
    combat::attack_triggers(&mut g, P0, &[h], P1).unwrap();
    assert_eq!(tokens(&g, P0) - before, 6, "1 + Isshin + Delney = 3 triggers of 2 tokens");
}

#[test]
fn delney_and_isshin_make_three_attack_triggers_for_a_compiled_ability() {
    let mut g = table(&["isshin-mardu-attack-triggers", "seph", "cloud"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    perm(&mut g, P0, DELNEY);
    let p = perm(&mut g, P0, "Teacher's Pest"); // compiled: whenever it attacks, gain 1 life
    let life = g.player(P0).life;
    combat::attack_triggers(&mut g, P0, &[p], P1).unwrap();
    assert_eq!(g.player(P0).life, life + 3, "1 + Isshin + Delney");
}

#[test]
fn delney_and_greenwarden_make_three_landfall_triggers_for_a_hook() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Ancient Greenwarden");
    perm(&mut g, P0, "Scute Swarm"); // an Event::Landfall hook: a 1/1 Insect
    let before = tokens(&g, P0);
    land(&mut g, "Plains");
    assert_eq!(tokens(&g, P0) - before, 3, "1 + Greenwarden + Delney");
}

#[test]
fn delney_and_greenwarden_make_three_landfall_triggers_for_a_compiled_ability() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Ancient Greenwarden");
    perm(&mut g, P0, "Icetill Explorer"); // compiled: landfall, mill a card
    let c = take(&mut g, P0, "Plains");
    let lib = g.player(P0).library.len();
    turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert_eq!(lib - g.player(P0).library.len(), 3, "1 + Greenwarden + Delney");
}

#[test]
fn delney_and_greenwarden_on_a_legendary_land_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Ancient Greenwarden");
    let y = perm(&mut g, P0, Y);
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 3, "the land-enters path: 1 + Greenwarden + Delney");
}

// ------------------------------------------------------------------ Teysa
#[test]
fn teysa_and_delney_make_three_dies_triggers_not_four() {
    let mut g = tbl();
    perm(&mut g, P0, "Teysa Karlov");
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Cruel Celebrant"); // 1/2: whenever it or another creature of yours dies, each opponent loses 1
    let t = token(&mut g, P0, 1);
    let life = g.player(P1).life;
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(life - g.player(P1).life, 3, "1 + Teysa + Delney");
}

#[test]
fn teysa_makes_a_creatures_own_dies_trigger_twice() {
    let mut g = tbl();
    perm(&mut g, P0, "Teysa Karlov");
    let c = perm(&mut g, P0, "Carrier Thrall"); // when it dies: an Eldrazi Scion
    let before = tokens(&g, P0);
    zones::die(&mut g, c, "destroy").unwrap();
    assert_eq!(tokens(&g, P0) - before, 2, "1 + Teysa");
}

// ------------------------------------------------------------------ Syr Vondam (power 2 or less)
#[test]
fn delney_doubles_syr_vondams_counter_trigger_while_his_power_is_two_or_less() {
    let mut g = brago();
    perm(&mut g, P0, DELNEY);
    let v = perm(&mut g, P0, "Syr Vondam, Sunstar Exemplar"); // 2/2
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.perm(v).plus, 2, "power 2: his trigger happens twice");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.perm(v).plus, 3, "power 4 now: once");
}

// ------------------------------------------------------------------ copies on the stack are not triggers
/// Gray Merchant of Asphodel enters (a 2/2, so Delney applies), with a Resonator ready to copy its trigger: how much
/// the opponent loses
fn gray_merchant_drain(delney: bool, resonator: bool) -> i32 {
    let mut g = brago();
    if resonator {
        perm(&mut g, P0, "Strionic Resonator");
        lands(&mut g, P0, "Island", 2, false);
    }
    if delney {
        perm(&mut g, P0, DELNEY);
    }
    g.active = Some(P0);
    g.step = Step::Main1;
    let life = g.player(P1).life;
    perm(&mut g, P0, "Gray Merchant of Asphodel");
    life - g.player(P1).life
}

#[test]
fn a_strionic_resonator_copy_adds_exactly_one_resolution_to_a_doubled_trigger() {
    let one = gray_merchant_drain(false, false);
    assert!(one > 0);
    assert_eq!(gray_merchant_drain(true, false), 2 * one, "Delney: two triggers");
    assert_eq!(gray_merchant_drain(true, true), 3 * one, "Delney's two triggers and one copy of one of them");
}

// ------------------------------------------------------------------ Harmonic Prodigy adds like the others
#[test]
fn isshin_and_harmonic_prodigy_make_three_triggers_not_four() {
    let mut g = tbl();
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    perm(&mut g, P0, "Harmonic Prodigy");
    let w = perm(&mut g, P0, "World Shaper"); // a Shaman: whenever it attacks, mill three cards
    let lib = g.player(P0).library.len();
    combat::attack_triggers(&mut g, P0, &[w], P1).unwrap();
    assert_eq!(lib - g.player(P0).library.len(), 9, "1 + Isshin + Prodigy: three triggers milling three");
}
