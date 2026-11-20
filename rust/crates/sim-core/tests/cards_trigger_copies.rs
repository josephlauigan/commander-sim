//! "Triggers an additional time" effects add up: with n such effects a trigger happens 1 + n times (CR 603.2d, and the
//! Panharmonicon ruling: two Panharmonicons give three triggers, not four). The event-based sources (Panharmonicon,
//! Starfield Vocalist, Ancient Greenwarden, Teysa Karlov, Isshin) and the controller-based ones (Cloud, Delney, Roaming
//! Throne) are counted together: the old engine multiplied the second kind into the first.

use sim_core::dsl::{self, Fired};
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{combat, stack, turn, zones};
use sim_core::hooks::{Call, Event, TrigAct, Trigger};
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

fn queued(g: &Game, src: PermId) -> Vec<&Trigger> {
    g.trig_queue.iter().filter(|t| t.src == Some(src)).collect()
}

fn flush(g: &mut Game) {
    g.resolving = 0;
    stack::flush_triggers(g).unwrap();
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

// ------------------------------------------------------------------ a doubler that has lost its abilities adds nothing
#[test]
fn a_neutered_cloud_does_not_double_his_equipment() {
    let mut g = tbl();
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    let mask = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(cl);
    let fire = |g: &mut Game| {
        g.resolving = 1;
        g.trig_queue.clear();
        let hit = Fired { attacker: Some(cl), defender: Some(P1), dmg: Some(2), ..Default::default() };
        dsl::fire(g, "combat_damage", hit).unwrap();
        waiting(g, mask)
    };
    assert_eq!(fire(&mut g), 2, "control: Cloud doubles the Mask's trigger");
    g.perm_mut(cl).neutered = true;
    assert_eq!(fire(&mut g), 1, "Cloud has lost all abilities");
}

#[test]
fn a_neutered_roaming_throne_does_not_add_a_trigger() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(t).data.set(DataKey::Ctype, Val::Str("human"));
    perm(&mut g, P0, "Soul Warden");
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 2, "control: 1 + Throne");
    g.perm_mut(t).neutered = true;
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 1, "Throne has lost its abilities");
}

#[test]
fn a_neutered_delney_does_not_add_a_trigger() {
    let mut g = tbl();
    let d = perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Soul Warden");
    g.perm_mut(d).neutered = true;
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 1, "Delney has lost his abilities");
}

// ------------------------------------------------------------------ the dies loops keep the original of each grant
#[test]
fn selfdies_and_rivaz_two_grants_keep_the_original_of_each_grant() {
    let mut g = tbl();
    for name in ["Teysa Karlov", "Drivnod, Carnage Dominus"] {
        perm(&mut g, P0, name);
    }
    let c = perm(&mut g, P0, "Carrier Thrall");
    g.perm_mut(c).data.set(DataKey::RivazExile, Val::Int(2));
    let cd = g.perm(c).card();
    g.resolving = 1;
    zones::die(&mut g, c, "destroy").unwrap();
    let selfdies: Vec<_> =
        queued(&g, c).into_iter().filter(|t| matches!(t.act, TrigAct::Hook { event: Event::SelfDies, .. })).collect();
    assert_eq!(selfdies.iter().map(|t| t.repeat).collect::<Vec<_>>(), [false, true, true]);
    let rivaz: Vec<_> = queued(&g, c).into_iter().filter(|t| matches!(t.act, TrigAct::RivazExile { .. })).collect();
    assert_eq!(rivaz.iter().map(|t| t.repeat).collect::<Vec<_>>(), [false, true, true, false, true, true]);
    let before = tokens(&g, P0);
    flush(&mut g);
    assert_eq!(tokens(&g, P0) - before, 3, "1 + Teysa + Drivnod Scions");
    assert!(!g.player(P0).gy.contains(&cd));
    assert!(g.player(P0).exile.contains(&cd), "Rivaz's grants exile the card");
    assert!(g.trig_queue.is_empty());
}

// ------------------------------------------------------------------ one landfall event: every listener is the event's own
const GREENWARDEN: &str = "Ancient Greenwarden";
const SCUTE: &str = "Scute Swarm";

fn scutes(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == SCUTE).count()
}

/// six lands in play, so a Scute Swarm's landfall makes a copy of itself
fn six_lands(g: &mut Game) {
    lands(g, P0, "Forest", 6, false);
}

#[test]
fn greenwarden_and_one_scute_make_two_copies_not_three() {
    let mut g = tbl();
    six_lands(&mut g);
    perm(&mut g, P0, GREENWARDEN);
    perm(&mut g, P0, SCUTE);
    land(&mut g, "Plains");
    assert_eq!(scutes(&g), 3, "the Scute's landfall happens twice; the copies it made are not listeners of this event");
}

#[test]
fn greenwarden_and_two_scutes_make_four_copies_not_six() {
    let mut g = tbl();
    six_lands(&mut g);
    perm(&mut g, P0, GREENWARDEN);
    perm(&mut g, P0, SCUTE);
    perm(&mut g, P0, SCUTE);
    land(&mut g, "Plains");
    assert_eq!(scutes(&g), 6, "two Scutes, two triggers each");
}

#[test]
fn a_scute_copy_does_not_hear_the_landfall_that_made_it() {
    let mut g = tbl();
    six_lands(&mut g);
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, GREENWARDEN);
    perm(&mut g, P0, SCUTE);
    land(&mut g, "Plains");
    assert_eq!(scutes(&g) - 1, 3, "1 + Greenwarden + Delney, from the one Scute that was there");
}

#[test]
fn a_scute_alone_with_six_lands_makes_one_copy() {
    let mut g = tbl();
    six_lands(&mut g);
    perm(&mut g, P0, SCUTE);
    land(&mut g, "Plains");
    assert_eq!(scutes(&g), 2);
}

#[test]
fn a_scute_under_a_held_resolution_makes_three_copies() {
    let mut g = tbl();
    six_lands(&mut g);
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, GREENWARDEN);
    perm(&mut g, P0, SCUTE);
    let c = take(&mut g, P0, "Plains");
    g.resolving = 1;
    g.add_land_entering(P0, c, false);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.trig_queue.iter().filter(|t| &*g.perm(t.src.unwrap()).name == SCUTE).count(), 3);
    flush(&mut g);
    assert_eq!(scutes(&g) - 1, 3);
}

#[test]
fn without_a_copier_the_landfall_triggers_queue_in_the_same_order_and_nothing_is_held() {
    let mut g = tbl();
    six_lands(&mut g);
    let explorer = perm(&mut g, P0, "Icetill Explorer"); // compiled: landfall, mill a card
    let scute = perm(&mut g, P0, SCUTE);
    let c = take(&mut g, P0, "Plains");
    g.resolving = 1;
    g.add_land_entering(P0, c, false);
    zones::landfall(&mut g, P0).unwrap();
    let order: Vec<_> = g.trig_queue.iter().map(|t| t.src).collect();
    assert_eq!(order, [Some(explorer), Some(scute)], "the compiled abilities, then the card code's hooks");
    assert_eq!(g.resolving, 1, "the caller's hold is untouched");
    flush(&mut g);
    // not held: a land played with no copier resolves its triggers inside the landfall
    let before = scutes(&g);
    land(&mut g, "Plains");
    assert_eq!(scutes(&g) - before, before, "each Scute makes one copy");
    assert!(g.trig_queue.is_empty());
    assert_eq!(g.resolving, 0);
}

// A guard, not a red test: Bloodghast leaves the graveyard on the first pass, so a second pass would find nothing. The
// `!repeat` gate on `gy_landfall` is rules-correct (a graveyard card's ability is no permanent's, so no copier applies)
// and has no observable effect with today's pool (Bloodghast is the only `gy_landfall` card).
#[test]
fn greenwarden_returns_a_graveyard_bloodghast_once() {
    let mut g = tbl();
    perm(&mut g, P0, GREENWARDEN);
    let b = take(&mut g, P0, "Bloodghast");
    g.player_mut(P0).gy.push(b);
    land(&mut g, "Plains");
    let on_bf = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Bloodghast").count();
    assert_eq!(on_bf, 1);
    assert!(!g.player(P0).gy.contains(&b));
}

// Bloodghast is milled by Icetill Explorer's trigger of this landfall. At base pass 1 resolves that mill before pass 1's
// graveyard landfall, so Bloodghast returns; under the hold nothing of the event resolves first, so it stays milled.
// (Without a copier the old order still returns it: a separate existing difference.)
#[test]
fn bloodghast_milled_by_the_landfall_does_not_hear_that_landfall_under_a_copier() {
    let mut g = tbl();
    perm(&mut g, P0, GREENWARDEN);
    perm(&mut g, P0, "Icetill Explorer");
    let b = take(&mut g, P0, "Bloodghast");
    g.player_mut(P0).library.push(b);
    land(&mut g, "Plains");
    assert!(g.player(P0).gy.contains(&b), "milled by this landfall, so not a listener of it");
    let on_bf = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Bloodghast").count();
    assert_eq!(on_bf, 0);
}

#[test]
fn isshin_and_kaalia_do_not_let_the_angel_kaalia_put_in_hear_the_attack() {
    let mut g = table(&["isshin-mardu-attack-triggers", "seph", "cloud"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    let k = perm(&mut g, P0, "Kaalia of the Vast");
    let a = token(&mut g, P0, 1);
    let b = token(&mut g, P0, 1);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Aurelia, the Law Above"]);
    let new = combat::attack_triggers(&mut g, P0, &[k, a, b], P1).unwrap();
    let aurelia = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Aurelia, the Law Above");
    assert!(aurelia.is_some_and(|m| g.perm(m).on_bf), "Kaalia put Aurelia onto the battlefield attacking");
    assert_eq!(new, [aurelia.unwrap()], "and she joins the attack");
    assert_eq!(g.player(P0).hand.len(), 0, "Aurelia was not there when the three creatures attacked: no draw");
}

#[test]
fn isshin_and_adeline_attack_with_the_tokens_of_both_passes() {
    let mut g = table(&["isshin-mardu-attack-triggers", "seph", "cloud", "sauron"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    let a = perm(&mut g, P0, "Adeline, Resplendent Cathar");
    let new = combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(new.len(), 6, "a Human token for each of 3 opponents, twice");
    assert!(new.iter().all(|&m| g.perm(m).token && g.perm(m).tapped));
}
