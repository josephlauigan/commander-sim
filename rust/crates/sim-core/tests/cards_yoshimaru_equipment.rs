//! Adversarial interaction tests for the Yoshimaru / Tymna Equipment package (JD's `yoshimaru` deck): Mithril Coat,
//! Yoshimaru, Puresteel Paladin, Cloud, Delney, Court of Ardenvale, Forge Anew, Zack Fair, Sword of Hearth and Home,
//! Stoneforge Mystic, Kor Outfitter, Ardenn, Wayfarer's Bauble, Steelshaper's Gift, Sting, Wrath of God and Stroke of
//! Midnight. Each expectation follows the card's Oracle text in data/scryfall_cache.json, quoted above the test; the
//! tests are written to break the engine, so a failing one that shows a real defect is `#[ignore = "UNRESOLVED: ..."]`
//! (it keeps the assertion and says what the engine did).

mod play_common;
use play_common::*;
use sim_core::ai::act;
use sim_core::cards::Colors;
use sim_core::engine::values::{colors_of, epow, etgh, has_type, indestructible, protected_from};
use sim_core::engine::{cast, combat, turn, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::human::legal::check_equip;
use sim_core::impls::{cloud, t2};
use sim_core::state::{Ctx, Step};
use std::sync::{Arc, Mutex};

const YOSHI: &str = "Yoshimaru, Ever Faithful";
const TYMNA: &str = "Tymna the Weaver";
const CLOUD: &str = "Cloud, Midgar Mercenary";
const DELNEY: &str = "Delney, Streetwise Lookout";
const ARDENN: &str = "Ardenn, Intrepid Archaeologist";
const COAT: &str = "Mithril Coat";
const STING: &str = "Sting, the Glinting Dagger";
const SOFI: &str = "Sword of Fire and Ice";
const SOFF: &str = "Sword of Feast and Famine";
const SOHH: &str = "Sword of Hearth and Home";
const BUSTER: &str = "Buster Sword";
const BOOTS: &str = "Swiftfoot Boots";
const FORGE: &str = "Forge Anew";
const COURT: &str = "Court of Ardenvale";
const PURESTEEL: &str = "Puresteel Paladin";
const SFM: &str = "Stoneforge Mystic";
const OUTFITTER: &str = "Kor Outfitter";
const ZACK: &str = "Zack Fair";
const BAUBLE: &str = "Wayfarer's Bauble";

/// The Yoshimaru list now runs Equipment, so a Stoneforge Mystic, Cloud or Steelshaper's Gift entering in a test would search a
/// real Equipment card out of the library into the hand. These tests build their own boards and hands (`hand` conjures a
/// card the library lacks), so P0 starts with no Equipment in its library, as if the list had none.
fn no_library_equipment(g: &mut Game, p: PlayerId) {
    let db = g.db.clone();
    g.player_mut(p).library.retain(|&c| !db.get(c).has_subtype("equipment"));
}

/// P0 is the Yoshimaru/Tymna seat; P1 and P2 are other decks. P0's turn, first main phase.
fn tbl() -> Game {
    let mut g = table(&["yoshimaru", "seph", "sauron"]);
    no_library_equipment(&mut g, P0);
    g.active = Some(P0);
    g
}

/// two seats, so that "any target" has one opponent
fn tbl2() -> Game {
    let mut g = table(&["yoshimaru", "seph"]);
    no_library_equipment(&mut g, P0);
    g.active = Some(P0);
    g
}

fn on(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn count_on(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name == name).count()
}

fn tokens_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).on_bf).collect()
}

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn lib_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).library.len()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn equipment_in_hand(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| g.db.get(c).has_subtype("equipment")).count()
}

/// the library is exactly these cards, the first listed on the bottom and the last on top
fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn gy_add(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

/// cast `name` from p's hand, already paid for
fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap()
}

fn cast_on(g: &mut Game, p: PlayerId, name: &str, t: PermId) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap()
}

fn wear(g: &mut Game, e: PermId, m: PermId) {
    g.perm_mut(e).attached = Some(m);
}

/// blink m (exile it, return it as a new object); the permanent it came back as
fn blink(g: &mut Game, m: PermId) -> PermId {
    t2::blink(g, P0, m).unwrap().expect("blinked")
}

fn plus(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

fn play_land(g: &mut Game, name: &str) -> PermId {
    let c = take(g, P0, name);
    let n = g.player(P0).lands.len();
    turn::play_land_card(g, P0, c, "plays").unwrap();
    assert_eq!(g.player(P0).lands.len(), n + 1);
    *g.player(P0).lands.last().unwrap()
}

/// P0's creature a attacks P1 unblocked
fn hit(g: &mut Game, a: PermId) {
    combat::resolve_combat(g, P0, &[a], P1, &[]).unwrap();
}

fn fire(g: &mut Game, e: Event, p: PlayerId) {
    sim_core::engine::hooks::fire_trigger(g, e, Call::Player { p }).unwrap();
}

fn options_of(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.expect("the card has options");
    f(g, src, p, Some(false)).unwrap()
}

fn tapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| g.perm(l).tapped).count()
}

/// a person at P0: a question whose prompt contains `prompt` takes the choice containing the paired word; every other
/// question takes the first choice and priority is passed
fn person_rules(g: &mut Game, rules: &[(&str, &str)]) -> Arc<ScriptHuman> {
    let rules: Vec<(String, String)> = rules.iter().map(|(p, c)| (p.to_string(), c.to_string())).collect();
    let used = Mutex::new(vec![false; rules.len()]);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        let mut used = used.lock().unwrap();
        for (i, (p, c)) in rules.iter().enumerate() {
            if used[i] {
                continue;
            }
            if r.prompt.contains(p.as_str())
                && let Some(k) = r.choices.iter().position(|x| x.contains(c.as_str()))
            {
                used[i] = true;
                return Answer::Pick(k);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the beginning of an end step: what was exiled until then comes back
fn end_step(g: &mut Game) {
    sim_core::cardcode::delayed_end_step(g, P0).unwrap();
}

// =================================================================================================================
// 1. Mithril Coat
// "Flash. Indestructible. When Mithril Coat enters, attach it to target legendary creature you control. Equipped
// creature has indestructible. Equip {3}"
// =================================================================================================================
#[test]
fn coat_put_onto_the_battlefield_attaches_to_a_legendary_creature_we_control() {
    // "When Mithril Coat enters, attach it to target legendary creature you control."
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(y));
    assert!(indestructible(&g, y), "the equipped creature has indestructible");
    assert!(indestructible(&g, coat), "and so does the Coat itself");
}

#[test]
fn coat_cast_from_hand_attaches_through_its_enter_trigger() {
    // "When Mithril Coat enters, attach it to target legendary creature you control." (cast, not put onto the field)
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    assert!(cast_free(&mut g, P0, COAT));
    let coat = on(&g, P0, COAT).expect("the Coat resolved");
    assert_eq!(g.perm(coat).attached, Some(t));
    assert!(indestructible(&g, t));
}

#[test]
fn coat_without_a_legendary_creature_stays_unattached() {
    // "attach it to target legendary creature you control": a non-legendary creature and a legendary non-creature are no targets
    let mut g = tbl();
    let t = token(&mut g, P0, 2);
    perm(&mut g, P0, STING); // legendary, but not a creature
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, None);
    assert!(!indestructible(&g, t), "the token is not indestructible");
}

#[test]
fn coat_never_attaches_to_an_opponents_legendary_creature() {
    // "target legendary creature you control"
    let mut g = tbl();
    let theirs = perm(&mut g, P1, TYMNA);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, None);
    assert!(!indestructible(&g, theirs));
}

#[test]
fn a_blinked_coat_comes_back_unattached_and_stays_loose_without_a_legendary_creature() {
    // "When Mithril Coat enters, attach it ..." is the only way it attaches itself: a later legendary creature does not pull it in
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(y));
    zones::exile_perm(&mut g, y).unwrap(); // the only legendary creature goes
    let n = blink(&mut g, coat);
    assert!(!g.perm(coat).on_bf, "a new object");
    assert_eq!(g.perm(n).attached, None, "no legendary creature to attach to");
    perm(&mut g, P0, TYMNA);
    assert_eq!(g.perm(n).attached, None, "the enter trigger has passed: Tymna arriving later does not attach it");
}

#[test]
fn a_blinked_coat_attaches_again_through_its_own_enter_trigger() {
    // "When Mithril Coat enters, attach it to target legendary creature you control."
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    let n = blink(&mut g, coat);
    assert!(!g.perm(coat).on_bf);
    assert_eq!(g.perm(n).attached, Some(y), "re-entered, then attached again");
    assert!(indestructible(&g, y));
}

#[test]
fn felidar_guardian_blinks_the_coat_and_it_attaches_again() {
    // Felidar Guardian: "When this creature enters, you may exile another target permanent you control, then return
    // that card to the battlefield under its owner's control."
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    person_rules(&mut g, &[("Felidar Guardian", COAT)]);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(!g.perm(coat).on_bf, "the Coat was exiled");
    let n = on(&g, P0, COAT).expect("and came back");
    assert_eq!(g.perm(n).attached, Some(y), "through its own enter trigger");
}

#[test]
fn flickerwisp_returns_the_coat_at_the_end_step_and_it_attaches_if_a_legend_is_there() {
    // Flickerwisp: "When this creature enters, exile another target permanent. Return that card to the battlefield
    // under its owner's control at the beginning of the next end step."
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    person_rules(&mut g, &[("Flickerwisp", COAT)]);
    perm(&mut g, P0, "Flickerwisp");
    assert!(!g.perm(coat).on_bf && in_exile(&g, P0, COAT));
    assert!(!indestructible(&g, y), "while the Coat is away Yoshimaru is not indestructible");
    end_step(&mut g);
    let n = on(&g, P0, COAT).expect("back at the end step");
    assert_eq!(g.perm(n).attached, Some(y));
}

#[test]
fn flickerwisp_returns_the_coat_unattached_when_the_legend_has_gone_meanwhile() {
    // "attach it to target legendary creature you control": none then, so none
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    perm(&mut g, P0, COAT);
    person_rules(&mut g, &[("Flickerwisp", COAT)]);
    perm(&mut g, P0, "Flickerwisp");
    zones::exile_perm(&mut g, y).unwrap();
    end_step(&mut g);
    let n = on(&g, P0, COAT).expect("back at the end step");
    assert_eq!(g.perm(n).attached, None);
}

// =================================================================================================================
// 2. Yoshimaru, Ever Faithful
// "Whenever another legendary permanent you control enters, put a +1/+1 counter on Yoshimaru."
// =================================================================================================================
#[test]
fn legendary_equipment_entering_gives_a_counter_each() {
    // "another legendary permanent you control enters": Mithril Coat, Sting and Andúril are legendary Equipment
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    for (i, name) in [COAT, STING, "Andúril, Narsil Reforged"].iter().enumerate() {
        perm(&mut g, P0, name);
        assert_eq!(plus(&g, y), i as i32 + 1, "{name}");
    }
}

#[test]
fn legendary_creatures_and_a_legendary_land_each_give_a_counter() {
    // Cloud, Ardenn, Zack Fair and Tymna are legendary creatures, Minas Tirith a legendary land
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    for (i, name) in [CLOUD, ARDENN, ZACK, TYMNA].iter().enumerate() {
        perm(&mut g, P0, name);
        assert_eq!(plus(&g, y), i as i32 + 1, "{name}");
    }
    play_land(&mut g, "Minas Tirith");
    assert_eq!(plus(&g, y), 5, "Minas Tirith");
}

#[test]
fn nonlegendary_equipment_gives_nothing() {
    // the Swords, Buster Sword and Swiftfoot Boots are not legendary
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    for name in [SOFI, SOFF, SOHH, BUSTER, BOOTS] {
        perm(&mut g, P0, name);
    }
    assert_eq!(plus(&g, y), 0);
}

#[test]
fn a_legendary_equipment_that_leaves_and_comes_back_gives_a_counter_again() {
    // a blinked Sting is a new object entering: "Whenever another legendary permanent you control enters"
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let sting = perm(&mut g, P0, STING);
    assert_eq!(plus(&g, y), 1);
    blink(&mut g, sting);
    assert_eq!(plus(&g, y), 2, "the blink is a second entry");
    // and a nonlegendary Sword blinked is not
    let sword = perm(&mut g, P0, SOFI);
    blink(&mut g, sword);
    assert_eq!(plus(&g, y), 2);
}

#[test]
fn a_blinked_coat_gives_a_counter_on_its_second_entry() {
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(plus(&g, y), 1);
    blink(&mut g, coat);
    assert_eq!(plus(&g, y), 2);
}

#[test]
fn yoshimarus_own_entry_gives_none_and_a_blinked_yoshimaru_starts_over() {
    // "another legendary permanent": not himself; the new object has no counters
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    assert_eq!(plus(&g, y), 0, "his own entry gives nothing");
    perm(&mut g, P0, TYMNA);
    assert_eq!(plus(&g, y), 1, "Tymna entering after him gives one");
    let n = blink(&mut g, y);
    assert_eq!(plus(&g, n), 0, "a new object with no counters, and his own entry again gives none");
}

#[test]
fn an_opponents_legendary_permanents_give_nothing() {
    // "you control"
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    perm(&mut g, P1, STING);
    perm(&mut g, P1, TYMNA);
    perm(&mut g, P2, COAT);
    assert_eq!(plus(&g, y), 0);
}

#[test]
fn delney_doubles_yoshimaru_while_his_power_is_two_or_less_and_stops_when_he_grows() {
    // Delney: "If a triggered ability of a creature you control with power 2 or less triggers, that ability triggers
    // an additional time." Yoshimaru is 1/1: Tymna's entry gives two counters (3/3), then Sting's only one.
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let y = perm(&mut g, P0, YOSHI);
    assert_eq!(plus(&g, y), 0, "Delney herself was there before him: not counted");
    let before = plus(&g, y);
    perm(&mut g, P0, TYMNA);
    assert_eq!(plus(&g, y) - before, 2, "doubled at power 1");
    let now = plus(&g, y);
    assert!(epow(&g, y) >= 3);
    perm(&mut g, P0, STING);
    assert_eq!(plus(&g, y) - now, 1, "power 3 or more: no longer doubled");
}

// =================================================================================================================
// 3. Puresteel Paladin
// "Whenever an Equipment you control enters, you may draw a card. Metalcraft — Equipment you control have equip {0}
// as long as you control three or more artifacts."
// =================================================================================================================
fn paladin_table() -> Game {
    let mut g = tbl();
    perm(&mut g, P0, PURESTEEL);
    g
}

#[test]
fn puresteel_draws_when_an_equipment_is_cast_and_not_for_a_non_equipment_artifact() {
    // "Whenever an Equipment you control enters, you may draw a card."
    let mut g = paladin_table();
    hand(&mut g, P0, &[SOFI, "Sol Ring"]);
    let lib = lib_len(&g, P0);
    let c = card(&g, SOFI);
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert!(on(&g, P0, SOFI).is_some());
    assert_eq!(lib_len(&g, P0), lib - 1, "one card drawn");
    let lib = lib_len(&g, P0);
    let c = card(&g, "Sol Ring");
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(on(&g, P0, "Sol Ring").is_some());
    assert_eq!(lib_len(&g, P0), lib, "Sol Ring is not an Equipment");
}

#[test]
fn puresteel_draws_for_stoneforge_mystics_ability_which_is_an_enter_not_a_cast() {
    // Stoneforge Mystic: "{1}{W}, {T}: You may put an Equipment card from your hand onto the battlefield."
    let mut g = paladin_table();
    lands(&mut g, P0, "Plains", 2, false);
    let s = perm(&mut g, P0, SFM);
    hand(&mut g, P0, &[SOFI]);
    let (lib, spells) = (lib_len(&g, P0), g.player(P0).spells_this_turn);
    let o = options_of(&mut g, s, P0);
    assert_eq!(o.len(), 1, "the activation is offered");
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(on(&g, P0, SOFI).is_some());
    assert_eq!(g.player(P0).spells_this_turn, spells, "never cast");
    assert_eq!(lib_len(&g, P0), lib - 1, "Puresteel drew for the enter");
}

#[test]
fn puresteel_draws_for_forge_anews_return_from_the_graveyard() {
    // Forge Anew: "When this enchantment enters, return target Equipment card from your graveyard to the battlefield."
    let mut g = paladin_table();
    gy_add(&mut g, P0, &[BUSTER]);
    sim_core::testkit::take(&mut g, P0, FORGE); // the list runs Forge Anew: `perm` would take it from the library, a card we do not count
    let lib = lib_len(&g, P0);
    perm(&mut g, P0, FORGE);
    assert!(on(&g, P0, BUSTER).is_some(), "returned");
    assert_eq!(lib_len(&g, P0), lib - 1, "Puresteel drew for the returned Equipment");
}

#[test]
fn puresteel_draws_for_a_blinked_equipment() {
    let mut g = paladin_table();
    let sword = perm(&mut g, P0, SOFI);
    let lib = lib_len(&g, P0);
    blink(&mut g, sword);
    assert_eq!(lib_len(&g, P0), lib - 1);
}

#[test]
fn puresteel_ignores_an_opponents_equipment() {
    // "an Equipment you control"
    let mut g = paladin_table();
    let lib = lib_len(&g, P0);
    perm(&mut g, P1, SOFI);
    assert_eq!(lib_len(&g, P0), lib);
}

#[test]
fn moving_equipment_never_draws_a_card() {
    // equip, Zack Fair's sacrifice and Ardenn's attach move Equipment: nothing enters
    let mut g = paladin_table();
    lands(&mut g, P0, "Plains", 3, false);
    let sword = perm(&mut g, P0, SOFI);
    let a = perm(&mut g, P0, TYMNA);
    let b = perm(&mut g, P0, "Kor Duelist");
    let z = perm(&mut g, P0, ZACK);
    sim_core::testkit::take(&mut g, P0, ARDENN); // the list runs Ardenn: `perm` would take it from the library, a card we do not count
    let lib = lib_len(&g, P0);
    assert!(sim_core::dsl::equip(&mut g, P0, sword, a, 2).unwrap(), "equip");
    assert_eq!(g.perm(sword).attached, Some(a));
    wear(&mut g, sword, z);
    assert!(cloud::zack_sacrifice(&mut g, z, P0, b.0 as i64).unwrap());
    assert_eq!(g.perm(sword).attached, Some(b), "Zack's sacrifice moved it");
    perm(&mut g, P0, ARDENN);
    assert_eq!(lib_len(&g, P0), lib, "entering Ardenn is not an Equipment entering");
    fire(&mut g, Event::CombatStart, P0);
    assert_eq!(lib_len(&g, P0), lib, "no card drawn by any move");
}

#[test]
fn metalcraft_needs_three_artifacts_we_control_to_make_equip_free() {
    // "Metalcraft — Equipment you control have equip {0} as long as you control three or more artifacts."
    let mut g = paladin_table();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 2, "two artifacts: the printed {{2}}");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mind Stone");
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 2, "an opponent's artifacts do not count");
    perm(&mut g, P0, BAUBLE);
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 0, "three artifacts: equip {{0}}");
    // the equip option the AI offers costs nothing
    let free = sim_core::dsl::equip_options(&g, P0);
    assert!(!free.is_empty() && free.iter().all(|o| matches!(o.act, Some(Action::Equip { n: 0, .. }))), "{free:?}");
}

#[test]
fn metalcraft_ends_when_an_artifact_leaves_or_the_paladin_does() {
    let mut g = paladin_table();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    let bauble = perm(&mut g, P0, BAUBLE);
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 0);
    zones::die(&mut g, bauble, "destroy").unwrap();
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 2, "two artifacts left");
    perm(&mut g, P0, BAUBLE);
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 0);
    let p = on(&g, P0, PURESTEEL).unwrap();
    g.perm_mut(p).phased = true;
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 2, "no Paladin in play, no metalcraft");
}

#[test]
fn metalcraft_equip_is_free_to_activate_for_a_person_too() {
    // the practice-mode check (human::legal) with no mana at all
    let mut g = paladin_table();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    perm(&mut g, P0, BAUBLE);
    assert!(check_equip(&g, P0, sword, t).is_some(), "two artifacts and no mana: can't pay {{2}}");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(check_equip(&g, P0, sword, t), None, "three artifacts: free");
}

// =================================================================================================================
// 4. Cloud, Midgar Mercenary
// "When Cloud enters, search your library for an Equipment card, reveal it, put it into your hand, then shuffle. As
// long as Cloud is equipped, if a triggered ability of Cloud or an Equipment attached to it triggers, that ability
// triggers an additional time."
// =================================================================================================================
fn cloud_wearing(g: &mut Game, eq: &[&str]) -> PermId {
    let c = perm(g, P0, CLOUD);
    for n in eq {
        let e = perm(g, P0, n);
        wear(g, e, c);
    }
    c
}

#[test]
fn sword_of_fire_and_ice_on_an_equipped_cloud_triggers_twice() {
    // SoFI: "Whenever equipped creature deals combat damage to a player, this Equipment deals 2 damage to any target
    // and you draw a card."
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    let c = cloud_wearing(&mut g, &[SOFI]);
    assert_eq!(epow(&g, c), 4);
    let (life, h) = (g.player(P1).life, hand_len(&g, P0));
    hit(&mut g, c);
    assert_eq!(hand_len(&g, P0), h + 2, "two draws");
    assert_eq!(g.player(P1).life, life - 4 - 2 * 2, "4 combat damage and two 2-damage triggers");
}

#[test]
fn sword_of_fire_and_ice_on_another_creature_triggers_once_even_with_cloud_out() {
    // Cloud only doubles triggers of Cloud and of Equipment attached to Cloud
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    perm(&mut g, P0, CLOUD); // unequipped
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    wear(&mut g, sword, t);
    let (life, h) = (g.player(P1).life, hand_len(&g, P0));
    hit(&mut g, t);
    assert_eq!(hand_len(&g, P0), h + 1, "one draw");
    assert_eq!(g.player(P1).life, life - 4 - 2, "4 combat damage and one 2-damage trigger");
}

#[test]
fn an_unequipped_cloud_with_a_loose_sword_doubles_nothing() {
    // "As long as Cloud is equipped": the Sword is on Tymna, Cloud wears nothing
    let mut g = tbl2();
    let c = perm(&mut g, P0, CLOUD);
    assert!(!cloud::doubles(&g, c));
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    wear(&mut g, sword, t);
    assert!(!cloud::doubles(&g, sword));
}

#[test]
fn sword_of_hearth_and_home_on_an_equipped_cloud_makes_two_lands_and_two_blinks() {
    // SoHH: "Whenever equipped creature deals combat damage to a player, exile up to one target creature you own, then
    // search your library for a basic land card. Put both cards onto the battlefield under your control, then shuffle."
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains", "Plains", "Plains"]);
    let c = cloud_wearing(&mut g, &[SOHH]);
    perm(&mut g, P0, "Wall of Omens"); // "enters, draw a card": the witness of a blink
    let (lands, lib) = (g.player(P0).lands.len(), lib_len(&g, P0));
    hit(&mut g, c);
    assert_eq!(g.player(P0).lands.len(), lands + 2, "two lands, one per trigger");
    assert!(g.player(P0).lands.iter().all(|&l| !g.perm(l).tapped), "untapped");
    // Wall of Omens drew twice if it was the one blinked each time: 2 lands + 2 draws
    assert_eq!(lib_len(&g, P0), lib - 4, "two lands fetched and two Wall of Omens draws");
}

#[test]
fn buster_sword_on_an_equipped_cloud_draws_twice() {
    // Buster Sword: "Whenever equipped creature deals combat damage to a player, draw a card, then you may cast a spell
    // from your hand with mana value less than or equal to that damage without paying its mana cost."
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    let c = cloud_wearing(&mut g, &[BUSTER]);
    assert_eq!(epow(&g, c), 5);
    let h = hand_len(&g, P0);
    hit(&mut g, c);
    assert_eq!(hand_len(&g, P0), h + 2, "two draws (lands are not spells to cast)");
}

#[test]
fn cloud_enters_unequipped_so_his_search_is_not_doubled() {
    // "As long as Cloud is equipped": entering, he is not
    let mut g = tbl();
    set_library(&mut g, P0, &[SOFI, SOFF, SOHH, BUSTER, "Plains"]);
    perm(&mut g, P0, CLOUD);
    assert_eq!(equipment_in_hand(&g, P0), 1);
}

#[test]
fn delney_doubles_clouds_enter_search_while_his_power_is_two_or_less() {
    // Delney: "If a triggered ability of a creature you control with power 2 or less triggers, that ability triggers
    // an additional time." Cloud is a 2/1.
    let mut g = tbl();
    set_library(&mut g, P0, &[SOFI, SOFF, SOHH, BUSTER, "Plains"]);
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, CLOUD);
    assert_eq!(equipment_in_hand(&g, P0), 2, "one search per instance");
}

#[test]
fn delney_does_not_double_an_equipments_trigger_so_cloud_plus_delney_is_two_not_three() {
    // Delney doubles triggered abilities of creatures, not of Equipment. Cloud's own doubling applies to the Sword's
    // trigger: twice, not three times.
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    perm(&mut g, P0, DELNEY);
    let c = cloud_wearing(&mut g, &[SOFI]);
    let h = hand_len(&g, P0);
    hit(&mut g, c);
    assert_eq!(hand_len(&g, P0), h + 2);
}

// =================================================================================================================
// 5. Delney, Streetwise Lookout
// "Creatures you control with power 2 or less can't be blocked by creatures with power 3 or greater. If a triggered
// ability of a creature you control with power 2 or less triggers, that ability triggers an additional time."
// =================================================================================================================
/// Tymna attacks P1, then her postcombat main phase begins: (cards drawn, life paid after the combat's lifelink)
fn tymna_turn(g: &mut Game, t: PermId) -> (usize, i32) {
    hit(g, t);
    let (h, life) = (hand_len(g, P0), g.player(P0).life);
    fire(g, Event::Main2, P0);
    (hand_len(g, P0) - h, life - g.player(P0).life)
}

#[test]
fn delney_makes_tymnas_postcombat_draw_trigger_twice() {
    // Tymna: "At the beginning of each of your postcombat main phases, you may pay X life, where X is the number of
    // opponents that were dealt combat damage this turn. If you do, draw X cards."
    let mut g = tbl2();
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, TYMNA);
    assert_eq!(tymna_turn(&mut g, t), (2, 2), "two instances: pay 1 life and draw 1, twice");
    // without Delney: once
    let mut g = tbl2();
    let t = perm(&mut g, P0, TYMNA);
    assert_eq!(tymna_turn(&mut g, t), (1, 1));
}

#[test]
fn boots_leave_delneys_doubling_on_tymna_and_a_sword_turns_it_off() {
    // Tymna 2/2: Swiftfoot Boots change nothing; Sword of Feast and Famine (+2/+2) makes her 4/4
    let mut g = tbl2();
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, TYMNA);
    let boots = perm(&mut g, P0, BOOTS);
    wear(&mut g, boots, t);
    assert_eq!(epow(&g, t), 2);
    assert_eq!(tymna_turn(&mut g, t), (2, 2), "Boots: still power 2, still doubled");
    let mut g = tbl2();
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFF); // no draw of its own
    wear(&mut g, sword, t);
    assert_eq!(epow(&g, t), 4);
    hit(&mut g, t);
    let h = hand_len(&g, P0);
    fire(&mut g, Event::Main2, P0);
    assert_eq!(hand_len(&g, P0), h + 1, "a Sword: power 4, Tymna triggers once");
}

#[test]
fn sting_makes_tymna_power_three_and_ends_the_doubling() {
    // Sting: "Equipped creature gets +1/+1 and has haste."
    let mut g = tbl2();
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, TYMNA);
    let sting = perm(&mut g, P0, STING);
    wear(&mut g, sting, t);
    assert_eq!(epow(&g, t), 3);
    hit(&mut g, t);
    let h = hand_len(&g, P0);
    fire(&mut g, Event::Main2, P0);
    assert_eq!(hand_len(&g, P0), h + 1);
}

#[test]
fn delney_doubles_enter_triggers_of_small_creatures_and_not_of_big_ones_or_opponents() {
    // Wall of Omens (0/4): "When this creature enters, draw a card." Grave Titan (6/6): "Whenever Grave Titan enters
    // or attacks, create two 2/2 black Zombie creature tokens."
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let lib = lib_len(&g, P0);
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(lib_len(&g, P0), lib - 2, "power 0: two draws");
    perm(&mut g, P0, "Grave Titan");
    assert_eq!(tokens_of(&g, P0).len(), 2, "power 6: one trigger, two Zombies");
    let lib1 = lib_len(&g, P1);
    perm(&mut g, P1, "Wall of Omens");
    assert_eq!(lib_len(&g, P1), lib1 - 1, "an opponent's creature is not ours: no extra trigger");
}

#[test]
fn delney_doubles_stoneforge_mystics_search() {
    // Stoneforge Mystic (1/2): "When this creature enters, you may search your library for an Equipment card"
    let mut g = tbl();
    set_library(&mut g, P0, &[SOFI, SOFF, SOHH, BUSTER, "Plains"]);
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, SFM);
    assert_eq!(equipment_in_hand(&g, P0), 2);
}

// =================================================================================================================
// 6. Court of Ardenvale
// "When this enchantment enters, you become the monarch. At the beginning of your upkeep, return target permanent card
// with mana value 3 or less from your graveyard to your hand. If you're the monarch, return that permanent card to the
// battlefield instead."
// =================================================================================================================
#[test]
fn court_entering_makes_us_the_monarch() {
    let mut g = tbl();
    assert_eq!(g.monarch, None);
    perm(&mut g, P0, COURT);
    assert_eq!(g.monarch, Some(P0));
}

#[test]
fn court_at_our_upkeep_returns_a_cheap_permanent_to_the_battlefield_while_we_are_the_monarch() {
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Mind Stone"]);
    fire(&mut g, Event::Upkeep, P0);
    assert!(on(&g, P0, "Mind Stone").is_some());
    assert!(!in_gy(&g, P0, "Mind Stone") && !in_hand(&g, P0, "Mind Stone"));
}

#[test]
fn court_returns_to_hand_when_we_are_not_the_monarch() {
    // "If you're the monarch, return that permanent card to the battlefield instead."
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    g.monarch = Some(P1);
    gy_add(&mut g, P0, &["Mind Stone"]);
    fire(&mut g, Event::Upkeep, P0);
    assert!(in_hand(&g, P0, "Mind Stone"));
    assert!(on(&g, P0, "Mind Stone").is_none());
}

#[test]
fn court_ignores_cards_that_cost_four_or_more_and_nonpermanent_cards() {
    // "permanent card with mana value 3 or less"
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Wrath of God", "Grave Titan", "Swords to Plowshares", "Sword of Kaldra"]);
    let n = g.player(P0).gy.len();
    fire(&mut g, Event::Upkeep, P0);
    assert_eq!(g.player(P0).gy.len(), n, "a sorcery, an instant and cards of mana value 4+ stay");
}

#[test]
fn court_does_nothing_at_an_opponents_upkeep_and_never_touches_their_graveyard() {
    // "At the beginning of your upkeep, ... from your graveyard"
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Mind Stone"]);
    gy_add(&mut g, P1, &["Sol Ring"]);
    g.active = Some(P1);
    fire(&mut g, Event::Upkeep, P1);
    assert!(in_gy(&g, P0, "Mind Stone") && in_gy(&g, P1, "Sol Ring"), "their upkeep: nothing");
    g.active = Some(P0);
    fire(&mut g, Event::Upkeep, P0);
    assert!(on(&g, P0, "Mind Stone").is_some());
    assert!(in_gy(&g, P1, "Sol Ring"), "only our graveyard");
}

#[test]
fn an_opponents_combat_damage_takes_the_crown_and_court_then_returns_to_hand() {
    // "the monarch": whenever a creature deals combat damage to the monarch, its controller becomes the monarch
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    assert_eq!(g.monarch, Some(P0));
    let a = perm(&mut g, P1, "Serra Angel");
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(g.monarch, Some(P1), "the opponent's combat damage to us flips the flag");
    gy_add(&mut g, P0, &["Mind Stone"]);
    fire(&mut g, Event::Upkeep, P0);
    assert!(in_hand(&g, P0, "Mind Stone"), "no longer the monarch: to hand");
    // and we take it back by hitting them (their Angel, which has vigilance, is gone so it can't block)
    zones::die(&mut g, a, "destroy").unwrap();
    let t = perm(&mut g, P0, TYMNA);
    hit(&mut g, t);
    assert_eq!(g.monarch, Some(P0));
}

#[test]
fn walking_ballista_returned_by_court_enters_with_x_zero_and_dies() {
    // Walking Ballista: "This creature enters with X +1/+1 counters on it." Mana value 0 in the graveyard (X is 0).
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Walking Ballista"]);
    fire(&mut g, Event::Upkeep, P0);
    assert!(on(&g, P0, "Walking Ballista").is_none(), "a 0/0 does not survive");
    assert!(in_gy(&g, P0, "Walking Ballista"), "back in the graveyard");
}

#[test]
fn blinking_court_makes_us_the_monarch_again() {
    // "When this enchantment enters, you become the monarch."
    let mut g = tbl();
    let court = perm(&mut g, P0, COURT);
    g.monarch = Some(P1);
    blink(&mut g, court);
    assert_eq!(g.monarch, Some(P0));
    assert_eq!(count_on(&g, P0, COURT), 1);
}

#[test]
fn court_returns_a_land_card_too() {
    // "return target permanent card with mana value 3 or less": a land is a permanent card
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Minas Tirith"]);
    fire(&mut g, Event::Upkeep, P0);
    let lands: Vec<_> = g.player(P0).lands.iter().map(|&l| g.perm(l).name).collect();
    assert!(lands.iter().any(|n| *n == "Minas Tirith"), "returned to the battlefield as the monarch: {lands:?}");
}

#[test]
fn two_courts_each_return_a_card() {
    // each Court's own trigger: two instances, two cards
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &["Mind Stone", "Sol Ring", "Arcane Signet"]);
    fire(&mut g, Event::Upkeep, P0);
    let back = ["Mind Stone", "Sol Ring", "Arcane Signet"].iter().filter(|n| on(&g, P0, n).is_some()).count();
    assert_eq!(back, 2);
}

// =================================================================================================================
// 7. Forge Anew
// "When this enchantment enters, return target Equipment card from your graveyard to the battlefield. During your turn,
// you may activate equip abilities any time you could cast an instant. You may pay {0} rather than pay the equip cost
// of the first equip ability you activate during each of your turns."
// =================================================================================================================
#[test]
fn forge_anew_returns_one_equipment_card_from_our_graveyard_only() {
    let mut g = tbl();
    gy_add(&mut g, P0, &[SOFI, BUSTER, "Swords to Plowshares"]);
    gy_add(&mut g, P1, &[SOFF]);
    perm(&mut g, P0, FORGE);
    let back = [SOFI, BUSTER].iter().filter(|n| on(&g, P0, n).is_some()).count();
    assert_eq!(back, 1, "target Equipment card: one");
    assert!(on(&g, P0, SOFF).is_none() && on(&g, P1, SOFF).is_none() && in_gy(&g, P1, SOFF), "not the opponent's");
    assert!(in_gy(&g, P0, "Swords to Plowshares"));
}

#[test]
fn forge_anew_with_no_equipment_in_the_graveyard_does_nothing() {
    let mut g = tbl();
    gy_add(&mut g, P0, &["Mind Stone"]);
    perm(&mut g, P0, FORGE);
    assert!(on(&g, P0, FORGE).is_some());
    assert!(in_gy(&g, P0, "Mind Stone") && on(&g, P0, "Mind Stone").is_none());
    // and with an empty graveyard
    let mut g = tbl();
    g.player_mut(P0).gy.clear();
    perm(&mut g, P0, FORGE);
    assert!(on(&g, P0, FORGE).is_some());
}

#[test]
fn a_legendary_equipment_returned_by_forge_anew_gives_yoshimaru_a_counter() {
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    gy_add(&mut g, P0, &[STING]);
    perm(&mut g, P0, FORGE);
    assert!(on(&g, P0, STING).is_some());
    assert_eq!(plus(&g, y), 1, "Sting entered under our control");
}

#[test]
fn a_mithril_coat_returned_by_forge_anew_attaches_itself() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    gy_add(&mut g, P0, &[COAT]);
    perm(&mut g, P0, FORGE);
    let coat = on(&g, P0, COAT).expect("returned");
    assert_eq!(g.perm(coat).attached, Some(t));
}

#[test]
fn blinking_forge_anew_returns_another_equipment() {
    let mut g = tbl();
    gy_add(&mut g, P0, &[SOFI]);
    let forge = perm(&mut g, P0, FORGE);
    assert!(on(&g, P0, SOFI).is_some());
    gy_add(&mut g, P0, &[BUSTER]);
    blink(&mut g, forge);
    assert!(on(&g, P0, BUSTER).is_some(), "the second entry returned the new one");
    assert!(on(&g, P0, SOFI).is_some());
}

/// P0's turn with Forge Anew out, Tymna, two loose Equipment
fn forge_table() -> (Game, PermId, PermId, PermId) {
    let mut g = tbl();
    perm(&mut g, P0, FORGE);
    let t = perm(&mut g, P0, TYMNA);
    let a = perm(&mut g, P0, SOFI);
    let b = perm(&mut g, P0, BUSTER);
    (g, t, a, b)
}

#[test]
fn the_first_equip_each_turn_costs_zero_and_the_second_pays_full() {
    let (mut g, t, a, b) = forge_table();
    lands(&mut g, P0, "Plains", 3, false);
    assert_eq!(cloud::equip_cost(&g, P0, a, t, 2), 0);
    let first = sim_core::dsl::equip_options(&g, P0);
    assert!(first.iter().all(|o| matches!(o.act, Some(Action::Equip { n: 0, .. }))), "{first:?}");
    assert!(act::perform(&mut g, P0, first[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(tapped_lands(&g, P0), 0, "the first equip cost nothing");
    let worn = [a, b].iter().filter(|&&e| g.perm(e).attached == Some(t)).count();
    assert_eq!(worn, 1);
    // the second one: the full {2}
    let second = sim_core::dsl::equip_options(&g, P0);
    assert!(
        !second.is_empty() && second.iter().all(|o| matches!(o.act, Some(Action::Equip { n: 2, .. }))),
        "{second:?}"
    );
    assert!(act::perform(&mut g, P0, second[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(tapped_lands(&g, P0), 2, "paid {{2}}");
    assert!([a, b].iter().all(|&e| g.perm(e).attached == Some(t)));
}

#[test]
fn the_second_equip_is_not_offered_without_the_mana() {
    let (mut g, t, a, b) = forge_table();
    lands(&mut g, P0, "Plains", 1, false);
    let first = sim_core::dsl::equip_options(&g, P0);
    assert!(act::perform(&mut g, P0, first[0].act.as_ref().unwrap()).unwrap());
    assert!(sim_core::dsl::equip_options(&g, P0).is_empty(), "{{2}} is more than the one land");
    let worn = [a, b].iter().filter(|&&e| g.perm(e).attached == Some(t)).count();
    assert_eq!(worn, 1);
}

#[test]
fn the_free_equip_comes_back_on_the_next_turn() {
    let (mut g, t, a, _) = forge_table();
    sim_core::dsl::equip(&mut g, P0, a, t, 0).unwrap();
    assert_eq!(cloud::equip_cost(&g, P0, a, t, 2), 2, "spent");
    g.round += 1;
    assert_eq!(cloud::equip_cost(&g, P0, a, t, 2), 0, "a new turn of ours");
}

#[test]
fn forge_anews_equip_discount_and_timing_are_for_our_own_turn_only() {
    // "During your turn" ... "during each of your turns"
    let (mut g, t, a, _) = forge_table();
    g.active = Some(P1);
    assert_eq!(cloud::equip_cost(&g, P0, a, t, 2), 2, "an opponent's turn: the full {{2}}");
    assert!(!cloud::forge_instant(&g, P0));
}

#[test]
fn a_person_may_equip_at_instant_speed_only_on_our_own_turn_with_forge_anew() {
    // "you may activate equip abilities any time you could cast an instant" (the combat step)
    let (mut g, t, a, _) = forge_table();
    g.step = Step::Combat;
    assert_eq!(check_equip(&g, P0, a, t), None, "in combat, with Forge Anew, free");
    g.step = Step::Main1;
    assert_eq!(check_equip(&g, P0, a, t), None);
    // an opponent's turn: sorcery timing again
    g.active = Some(P1);
    g.step = Step::Combat;
    assert!(check_equip(&g, P0, a, t).is_some(), "not our turn");
    // no Forge Anew: not in combat
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, TYMNA);
    let a = perm(&mut g, P0, SOFI);
    g.step = Step::Combat;
    assert!(check_equip(&g, P0, a, t).is_some(), "without Forge Anew, equip is a sorcery-speed ability");
}

#[test]
fn forge_anews_free_equip_is_free_for_a_costly_equipment_too() {
    // "You may pay {0} rather than pay the equip cost of the first equip ability": Mithril Coat's {3}
    let mut g = tbl();
    perm(&mut g, P0, FORGE);
    let t = perm(&mut g, P0, TYMNA);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(t));
    let y = perm(&mut g, P0, YOSHI);
    assert_eq!(cloud::equip_cost(&g, P0, coat, y, 3), 0);
    assert_eq!(check_equip(&g, P0, coat, y), None);
}

// =================================================================================================================
// 8. Zack Fair
// "Zack Fair enters with a +1/+1 counter on it. {1}, Sacrifice Zack Fair: Target creature you control gains
// indestructible until end of turn. Put Zack Fair's counters on that creature and attach an Equipment that was
// attached to Zack Fair to that creature."
// =================================================================================================================
#[test]
fn zack_fair_enters_with_a_counter() {
    let mut g = tbl();
    let z = perm(&mut g, P0, ZACK);
    assert_eq!(plus(&g, z), 1);
    assert_eq!((epow(&g, z), etgh(&g, z)), (1, 2));
}

#[test]
fn zack_fairs_sacrifice_moves_counters_and_equipment_and_grants_indestructible() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    g.perm_mut(z).plus = 3; // more counters than he entered with
    let sword = perm(&mut g, P0, SOFI);
    wear(&mut g, sword, z);
    let t = perm(&mut g, P0, TYMNA);
    assert!(!indestructible(&g, t));
    assert!(cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap());
    assert!(!g.perm(z).on_bf && in_gy(&g, P0, ZACK), "sacrificed");
    assert_eq!(plus(&g, t), 3, "all of Zack's counters");
    assert_eq!(g.perm(sword).attached, Some(t), "the Equipment follows");
    assert!(indestructible(&g, t));
    assert_eq!(tapped_lands(&g, P0), 1, "paid {{1}}");
}

#[test]
fn zack_fair_leaves_equipment_that_was_on_another_creature_alone() {
    // "an Equipment that was attached to Zack Fair"
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let t = perm(&mut g, P0, TYMNA);
    let other = perm(&mut g, P0, "Kor Duelist");
    let sword = perm(&mut g, P0, SOFI);
    wear(&mut g, sword, other);
    assert!(cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap());
    assert_eq!(g.perm(sword).attached, Some(other), "not Zack's, so not moved");
}

#[test]
fn a_mithril_coat_on_a_sacrificed_zack_moves_too() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(z), "the Coat's own trigger put it on Zack");
    let y = perm(&mut g, P0, YOSHI);
    assert!(cloud::zack_sacrifice(&mut g, z, P0, y.0 as i64).unwrap());
    assert_eq!(g.perm(coat).attached, Some(y));
    assert!(indestructible(&g, y));
}

#[test]
fn zack_fair_attaches_only_one_of_two_equipment() {
    // "attach an Equipment that was attached to Zack Fair to that creature": one Equipment
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let (a, b) = (perm(&mut g, P0, SOFI), perm(&mut g, P0, BUSTER));
    wear(&mut g, a, z);
    wear(&mut g, b, z);
    let t = perm(&mut g, P0, TYMNA);
    assert!(cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap());
    let moved = [a, b].iter().filter(|&&e| g.perm(e).attached == Some(t)).count();
    assert_eq!(moved, 1, "exactly one Equipment is attached to the new creature");
}

#[test]
fn zack_fair_cannot_be_activated_with_no_other_creature_or_no_mana() {
    // "Target creature you control": he is sacrificed as a cost, so he can't be it
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    assert!(!cloud::zack_sacrifice(&mut g, z, P0, z.0 as i64).unwrap());
    assert!(g.perm(z).on_bf && tapped_lands(&g, P0) == 0, "no creature to target: nothing paid, nothing sacrificed");
    // no mana
    let mut g = tbl();
    let z = perm(&mut g, P0, ZACK);
    let t = perm(&mut g, P0, TYMNA);
    assert!(!cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap());
    assert!(g.perm(z).on_bf, "no {{1}}: not sacrificed");
}

#[test]
fn zack_fair_cannot_target_an_opponents_creature() {
    // "Target creature you control"
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let theirs = perm(&mut g, P1, "Serra Angel");
    assert!(!cloud::zack_sacrifice(&mut g, z, P0, theirs.0 as i64).unwrap());
    assert!(g.perm(z).on_bf);
}

#[test]
fn zack_fair_cannot_target_a_creature_with_protection_from_white() {
    // Sword of Hearth and Home: "Equipped creature gets +2/+2 and has protection from green and from white."
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    assert!(protected_from(&g, t, Colors::from_letters("W")));
    assert!(!cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap(), "a white ability can't target it");
    assert!(g.perm(z).on_bf);
}

// =================================================================================================================
// 9. Sword of Hearth and Home
// "Equipped creature gets +2/+2 and has protection from green and from white. Whenever equipped creature deals combat
// damage to a player, exile up to one target creature you own, then search your library for a basic land card. Put
// both cards onto the battlefield under your control, then shuffle."
// =================================================================================================================
#[test]
fn sword_of_hearth_and_home_gives_plus_two_and_protection_from_green_and_white_only() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    let (p, tg) = (epow(&g, t), etgh(&g, t));
    wear(&mut g, sword, t);
    assert_eq!((epow(&g, t), etgh(&g, t)), (p + 2, tg + 2));
    for (c, want) in [("W", true), ("G", true), ("B", false), ("R", false), ("U", false)] {
        assert_eq!(protected_from(&g, t, Colors::from_letters(c)), want, "{c}");
    }
}

#[test]
fn creatures_that_are_white_or_green_cannot_block_the_carrier_but_black_ones_can() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    let white = perm(&mut g, P1, "Serra Angel");
    let green = perm(&mut g, P1, "Llanowar Elves");
    let black = perm(&mut g, P1, "Grave Titan");
    assert!(!combat::can_block(&g, white, t), "protection from white");
    assert!(!combat::can_block(&g, green, t), "protection from green");
    assert!(combat::can_block(&g, black, t));
}

#[test]
fn sword_of_hearth_and_homes_trigger_puts_a_land_onto_the_battlefield_untapped_and_blinks_a_creature() {
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    let wall = perm(&mut g, P0, "Wall of Omens");
    let (lands_before, lib) = (g.player(P0).lands.len(), lib_len(&g, P0));
    hit(&mut g, t);
    assert_eq!(g.player(P0).lands.len(), lands_before + 1, "one basic land");
    let l = *g.player(P0).lands.last().unwrap();
    assert!(!g.perm(l).tapped, "the land comes in untapped");
    let blinked_creature = !g.perm(wall).on_bf || !g.perm(t).on_bf;
    assert!(blinked_creature, "a creature we own was exiled and returned");
    assert!(g.player(P0).exile.is_empty(), "nothing is left in exile: the exiled card was put onto the battlefield");
    assert!(lib_len(&g, P0) < lib, "the land came from the library");
}

#[test]
fn the_exiled_creatures_enter_triggers_run_again() {
    // "Put both cards onto the battlefield": Wall of Omens draws as it returns
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    let wall = perm(&mut g, P0, "Wall of Omens");
    let lib = lib_len(&g, P0);
    hit(&mut g, t);
    // whichever creature was chosen, a land was fetched; if the Wall was chosen it drew a card as well
    let drew = lib - lib_len(&g, P0);
    assert!(drew == 1 || drew == 2, "land fetched, and the Wall's draw when it was the blinked creature: {drew}");
    if !g.perm(wall).on_bf {
        assert_eq!(drew, 2, "Wall of Omens re-entered: it drew");
        assert!(on(&g, P0, "Wall of Omens").is_some());
    }
}

#[test]
fn protection_from_white_does_not_stop_mithril_coats_attach_trigger() {
    // Mithril Coat is colourless: its enter trigger may target a creature with protection from white
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(t));
}

#[test]
fn protection_from_white_does_not_stop_a_colourless_equipments_equip_ability() {
    // equip abilities of colourless Equipment may target the carrier
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let t = perm(&mut g, P0, TYMNA);
    let sohh = perm(&mut g, P0, SOHH);
    wear(&mut g, sohh, t);
    let sofi = perm(&mut g, P0, SOFI);
    pool(&mut g, P0, &[('W', 2)]);
    assert_eq!(check_equip(&g, P0, sofi, t), None);
    assert!(sim_core::dsl::equip(&mut g, P0, sofi, t, 2).unwrap());
    assert_eq!(g.perm(sofi).attached, Some(t));
}

#[test]
fn kor_outfitter_cannot_target_the_carrier_with_protection_from_white() {
    // Kor Outfitter: "When this creature enters, you may attach target Equipment you control to target creature you
    // control." Kor Outfitter is white; Tymna wearing the Sword has protection from white.
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sohh = perm(&mut g, P0, SOHH);
    wear(&mut g, sohh, t);
    let boots = perm(&mut g, P0, BOOTS);
    perm(&mut g, P0, OUTFITTER);
    assert_ne!(g.perm(boots).attached, Some(t), "the Outfitter's trigger can't target Tymna");
}

#[test]
fn ardenn_cannot_attach_to_the_carrier_with_protection_from_white() {
    // Ardenn: "attach any number of Auras and Equipment you control to target permanent or player." Ardenn is white.
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sohh = perm(&mut g, P0, SOHH);
    wear(&mut g, sohh, t);
    let boots = perm(&mut g, P0, BOOTS);
    perm(&mut g, P0, ARDENN);
    fire(&mut g, Event::CombatStart, P0);
    assert_ne!(g.perm(boots).attached, Some(t), "Ardenn can't target Tymna");
}

#[test]
fn restoration_angel_cannot_blink_the_carrier_with_protection_from_white() {
    // Restoration Angel: "When this creature enters, you may exile target non-Angel creature you control, then return
    // that card to the battlefield under your control." Control run first: the same board without the Sword blinks.
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Restoration Angel");
    assert!(!g.perm(c).on_bf, "control: without the Sword the AI blinks Cloud (an enters creature)");
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    let sohh = perm(&mut g, P0, SOHH);
    wear(&mut g, sohh, c);
    perm(&mut g, P0, "Restoration Angel");
    assert!(g.perm(c).on_bf, "protection from white: the Angel's trigger can't target Cloud");
}

#[test]
fn felidar_guardian_cannot_blink_the_carrier_with_protection_from_white() {
    // Felidar Guardian: "you may exile another target permanent you control". Control run first.
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(!g.perm(c).on_bf, "control: without the Sword the AI blinks Cloud");
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    let sohh = perm(&mut g, P0, SOHH);
    wear(&mut g, sohh, c);
    perm(&mut g, P0, "Felidar Guardian");
    assert!(g.perm(c).on_bf, "protection from white: Felidar's trigger can't target Cloud");
}

#[test]
fn ephemerate_is_not_offered_on_the_carrier_with_protection_from_white() {
    // Ephemerate: "Exile target creature you control, then return it to the battlefield under its owner's control."
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let s = perm(&mut g, P0, SFM);
    let c = hand(&mut g, P0, &["Ephemerate"])[0];
    let f = g.registry.get(c).unwrap().hand_options.expect("Ephemerate has a hand option");
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(o.iter().any(|x| x.label.contains("Ephemerate")), "control: offered on a plain Stoneforge Mystic");
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, s);
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(!o.iter().any(|x| x.label.contains("Ephemerate")), "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
}

// =================================================================================================================
// 10. Stoneforge Mystic, Kor Outfitter, Ardenn
// =================================================================================================================
#[test]
fn stoneforge_mystic_enters_and_fetches_an_equipment() {
    // "When this creature enters, you may search your library for an Equipment card, reveal it, put it into your hand"
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Sol Ring", SOFI, "Mind Stone"]);
    perm(&mut g, P0, SFM);
    assert_eq!(equipment_in_hand(&g, P0), 1);
    assert!(in_hand(&g, P0, SOFI));
}

#[test]
fn stoneforge_mystics_ability_puts_a_sword_onto_the_battlefield_unattached_and_pays_one_white() {
    // "{1}{W}, {T}: You may put an Equipment card from your hand onto the battlefield." Sword of Fire and Ice has no
    // attach trigger: it enters unattached.
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, TYMNA);
    let s = perm(&mut g, P0, SFM);
    hand(&mut g, P0, &[SOFI]);
    let o = options_of(&mut g, s, P0);
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let sword = on(&g, P0, SOFI).expect("on the battlefield");
    assert_eq!(g.perm(sword).attached, None, "it enters unattached");
    assert!(g.perm(s).tapped, "{{T}}");
    assert_eq!(tapped_lands(&g, P0), 2, "{{1}}{{W}}");
    assert!(!in_hand(&g, P0, SOFI));
    assert!(!g.perm(t).tapped);
}

#[test]
fn stoneforge_mystic_puts_mithril_coat_which_attaches_itself() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let t = perm(&mut g, P0, TYMNA);
    let s = perm(&mut g, P0, SFM);
    hand(&mut g, P0, &[COAT]);
    let o = options_of(&mut g, s, P0);
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let coat = on(&g, P0, COAT).expect("on the battlefield");
    assert_eq!(g.perm(coat).attached, Some(t), "its own enter trigger attached it");
}

#[test]
fn stoneforge_mystics_ability_needs_an_untapped_unsick_mystic_and_mana() {
    // "{1}{W}, {T}": a summoning-sick or tapped Mystic can't pay {T}; without {1}{W} it can't either
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let s = perm(&mut g, P0, SFM);
    hand(&mut g, P0, &[SOFI]);
    g.perm_mut(s).sick = true;
    assert!(options_of(&mut g, s, P0).is_empty(), "summoning sick");
    g.perm_mut(s).sick = false;
    g.perm_mut(s).tapped = true;
    assert!(options_of(&mut g, s, P0).is_empty(), "tapped");
    g.perm_mut(s).tapped = false;
    assert_eq!(options_of(&mut g, s, P0).len(), 1);
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let s = perm(&mut g, P0, SFM);
    hand(&mut g, P0, &[SOFI]);
    assert!(options_of(&mut g, s, P0).is_empty(), "one land is not {{1}}{{W}}");
}

#[test]
fn kor_outfitter_attaches_a_loose_equipment_when_it_enters_and_a_blinked_equipment_does_not_use_it() {
    // "When this creature enters, you may attach target Equipment you control to target creature you control."
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    assert_eq!(g.perm(sword).attached, None);
    perm(&mut g, P0, OUTFITTER);
    // attached by the enter trigger; the yoshimaru seat scores the holder (`cloud::yoshimaru_attach`), and a Sword is not
    // for Tymna (it turns off Delney's doubling for her), so it goes on the Outfitter itself
    assert!(g.perm(sword).attached.is_some_and(|h| h != t), "attached by the enter trigger, not to Tymna");
    // blinking the Equipment does not trigger the Outfitter again: only the Outfitter entering does
    let n = blink(&mut g, sword);
    assert_eq!(g.perm(n).attached, None, "a new object, unattached, and the Outfitter on the battlefield does nothing");
}

#[test]
fn blinking_kor_outfitter_attaches_again() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let o = perm(&mut g, P0, OUTFITTER);
    let sword = perm(&mut g, P0, SOFI);
    assert_eq!(g.perm(sword).attached, None, "the Outfitter was there first");
    blink(&mut g, o);
    // (the yoshimaru seat's holder scoring keeps the Sword off Tymna: it goes on the Outfitter that just came back)
    assert!(g.perm(sword).attached.is_some_and(|h| h != t));
}

#[test]
fn ardenn_attaches_any_number_of_equipment_at_the_beginning_of_our_combat() {
    // "At the beginning of combat on your turn, you may attach any number of Auras and Equipment you control to target
    // permanent or player."
    let mut g = tbl();
    perm(&mut g, P0, ARDENN);
    let (s1, s2) = (perm(&mut g, P0, SOFI), perm(&mut g, P0, BUSTER));
    fire(&mut g, Event::CombatStart, P0);
    let h1 = g.perm(s1).attached.expect("attached");
    assert_eq!(g.perm(s2).attached, Some(h1), "both on one creature");
    assert!(g.is_creature(h1) && g.perm(h1).owner == P0);
}

#[test]
fn ardenn_works_on_the_turn_she_enters_and_only_on_our_own_turn() {
    let mut g = tbl();
    let s1 = perm(&mut g, P0, SOFI);
    perm(&mut g, P0, TYMNA);
    perm(&mut g, P0, ARDENN); // just entered
    g.active = Some(P1);
    fire(&mut g, Event::CombatStart, P1);
    assert_eq!(g.perm(s1).attached, None, "an opponent's combat: nothing");
    g.active = Some(P0);
    fire(&mut g, Event::CombatStart, P0);
    assert!(g.perm(s1).attached.is_some(), "our combat, the turn she entered");
}

// =================================================================================================================
// 11. Wayfarer's Bauble, Steelshaper's Gift, Sting
// =================================================================================================================
#[test]
fn wayfarers_bauble_fetches_a_basic_land_tapped_and_is_sacrificed() {
    // "{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put that card onto the battlefield tapped, then shuffle."
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let b = perm(&mut g, P0, BAUBLE);
    set_library(&mut g, P0, &["Command Tower", "Plains", "Island", "Sol Ring"]);
    assert!(sim_core::dsl::activate(&mut g, P0, b, 0).unwrap());
    assert!(in_gy(&g, P0, BAUBLE) && !g.perm(b).on_bf, "sacrificed");
    assert_eq!(g.player(P0).lands.len(), 3);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.perm(l).tapped, "the land enters tapped");
    let n = g.perm(l).name;
    assert!(n == "Plains" || n == "Island" || n == "Swamp", "a basic land, not Command Tower: {n}");
    assert_eq!(tapped_lands(&g, P0), 3, "{{2}} paid, and the new land is tapped");
}

#[test]
fn wayfarers_bauble_cannot_be_activated_without_two_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let b = perm(&mut g, P0, BAUBLE);
    assert!(!sim_core::dsl::activate(&mut g, P0, b, 0).unwrap());
    assert!(g.perm(b).on_bf);
}

#[test]
fn steelshapers_gift_finds_only_an_equipment_card() {
    // "Search your library for an Equipment card, reveal that card, put it into your hand, then shuffle."
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Grave Titan", "Sol Ring", SOHH, "Brainstorm"]);
    let h = hand_len(&g, P0);
    cast_free(&mut g, P0, "Steelshaper's Gift");
    let names: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert!(in_hand(&g, P0, SOHH), "the Equipment; the hand is {names:?}");
    assert_eq!(hand_len(&g, P0), h + 1);
    // no Equipment in the library: it finds nothing
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Grave Titan", "Sol Ring", "Brainstorm"]);
    let h = hand_len(&g, P0);
    cast_free(&mut g, P0, "Steelshaper's Gift");
    assert_eq!(hand_len(&g, P0), h, "no Equipment card: nothing found, and not a card of another kind");
}

#[test]
fn sting_gives_plus_one_plus_one_and_haste() {
    // "Equipped creature gets +1/+1 and has haste."
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sting = perm(&mut g, P0, STING);
    wear(&mut g, sting, t);
    assert_eq!((epow(&g, t), etgh(&g, t)), (3, 3));
    assert!(sim_core::dsl::has_kw(&g, t, "haste"));
}

#[test]
fn sting_untaps_the_equipped_creature_at_the_beginning_of_each_combat() {
    // "At the beginning of each combat, untap equipped creature." (each: also an opponent's)
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sting = perm(&mut g, P0, STING);
    wear(&mut g, sting, t);
    g.perm_mut(t).tapped = true;
    fire(&mut g, Event::EachCombat, P0);
    assert!(!g.perm(t).tapped, "our combat");
    g.perm_mut(t).tapped = true;
    g.active = Some(P1);
    fire(&mut g, Event::EachCombat, P1);
    assert!(!g.perm(t).tapped, "an opponent's combat too");
    // a creature without Sting stays tapped
    let u = perm(&mut g, P0, "Kor Duelist");
    g.perm_mut(u).tapped = true;
    fire(&mut g, Event::EachCombat, P1);
    assert!(g.perm(u).tapped);
}

// =================================================================================================================
// Cross-card interactions
// =================================================================================================================
#[test]
fn delney_doubles_puresteel_paladins_draw_for_each_equipment() {
    // Puresteel Paladin is a 2/2: Delney, "If a triggered ability of a creature you control with power 2 or less
    // triggers, that ability triggers an additional time": two draws for one Equipment entering
    let mut g = paladin_table();
    perm(&mut g, P0, DELNEY);
    hand(&mut g, P0, &[SOFI]);
    let lib = lib_len(&g, P0);
    let c = card(&g, SOFI);
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(lib_len(&g, P0), lib - 2, "two instances of the draw");
}

#[test]
fn court_returning_a_legendary_creature_gives_yoshimaru_a_counter() {
    // Court of Ardenvale returns Tymna (mana value 3) to the battlefield as the monarch: a legendary permanent entering
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    perm(&mut g, P0, COURT);
    gy_add(&mut g, P0, &[TYMNA]);
    fire(&mut g, Event::Upkeep, P0);
    assert!(on(&g, P0, TYMNA).is_some());
    assert_eq!(plus(&g, y), 1);
    // Zack Fair (mana value 1): comes back with his counter and gives one too
    gy_add(&mut g, P0, &[ZACK]);
    fire(&mut g, Event::Upkeep, P0);
    let z = on(&g, P0, ZACK).expect("Zack returned");
    assert_eq!(plus(&g, z), 1, "he enters with a +1/+1 counter");
    assert_eq!(plus(&g, y), 2);
}

#[test]
fn the_monarch_draws_at_the_beginning_of_their_own_end_step_only() {
    // monarch: "At the beginning of your end step, draw a card."
    let mut g = tbl();
    perm(&mut g, P0, COURT);
    let h = hand_len(&g, P0);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(hand_len(&g, P0), h + 1, "the monarch draws");
    g.monarch = Some(P1);
    let h = hand_len(&g, P0);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(hand_len(&g, P0), h, "not the monarch: no card");
}

#[test]
fn mithril_coat_attaches_to_a_legend_that_has_hexproof_from_swiftfoot_boots() {
    // hexproof stops only an opponent's spells and abilities: our own Coat may target a creature wearing Boots
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let boots = perm(&mut g, P0, BOOTS);
    wear(&mut g, boots, t);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(t));
}

#[test]
fn metalcraft_counts_treasure_tokens_as_artifacts() {
    // "three or more artifacts": a Treasure token is an artifact
    let mut g = paladin_table();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    sim_core::engine::zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(cloud::equip_cost(&g, P0, sword, t, 2), 0, "the Sword and two Treasures are three artifacts");
}

#[test]
fn a_person_blinks_wall_of_omens_with_sword_of_hearth_and_home_and_it_draws_again() {
    // the exiled creature is put onto the battlefield, so its enter trigger runs again; the land is untapped
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains"]);
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, t);
    let wall = perm(&mut g, P0, "Wall of Omens");
    person_rules(&mut g, &[("", "Wall of Omens")]);
    let (lib, lands_before) = (lib_len(&g, P0), g.player(P0).lands.len());
    hit(&mut g, t);
    assert!(!g.perm(wall).on_bf, "the Wall was exiled");
    assert!(on(&g, P0, "Wall of Omens").is_some(), "and put back onto the battlefield");
    assert_eq!(lib - lib_len(&g, P0), 2, "one card for the land and one for the Wall's enter trigger");
    assert_eq!(g.player(P0).lands.len(), lands_before + 1);
    let l = *g.player(P0).lands.last().unwrap();
    assert!(!g.perm(l).tapped, "untapped");
    assert!(g.perm(sword).attached == Some(t), "the carrier stayed");
}

#[test]
fn a_person_makes_cloud_use_sword_of_hearth_and_home_twice_on_the_wall() {
    let mut g = tbl2();
    set_library(&mut g, P0, &["Plains", "Plains", "Plains", "Plains", "Plains", "Plains"]);
    let c = cloud_wearing(&mut g, &[SOHH]);
    perm(&mut g, P0, "Wall of Omens");
    person_rules(&mut g, &[("", "Wall of Omens"), ("", "Wall of Omens")]);
    let (lib, lands_before) = (lib_len(&g, P0), g.player(P0).lands.len());
    hit(&mut g, c);
    assert_eq!(g.player(P0).lands.len(), lands_before + 2);
    assert_eq!(lib - lib_len(&g, P0), 4, "two lands and two draws of the re-entered Wall");
    assert_eq!(count_on(&g, P0, "Wall of Omens"), 1);
}

// =================================================================================================================
// 12. Wrath of God, Stroke of Midnight
// =================================================================================================================
#[test]
fn wrath_of_god_destroys_every_creature_and_leaves_equipment_unattached() {
    // "Destroy all creatures. They can't be regenerated."
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sword = perm(&mut g, P0, SOFI);
    wear(&mut g, sword, t);
    let mine = perm(&mut g, P0, "Kor Duelist");
    let theirs = perm(&mut g, P1, "Serra Angel");
    let tok = token(&mut g, P2, 2);
    let rock = perm(&mut g, P1, "Sol Ring");
    cast_free(&mut g, P0, "Wrath of God");
    for m in [t, mine, theirs, tok] {
        assert!(!g.perm(m).on_bf);
    }
    assert!(g.perm(sword).on_bf, "Equipment stays");
    assert_eq!(g.perm(sword).attached, None, "unattached");
    assert!(g.perm(rock).on_bf, "not a creature");
    assert!(in_gy(&g, P0, "Wrath of God"));
}

#[test]
fn wrath_of_god_cannot_be_regenerated_against() {
    // "They can't be regenerated."
    let mut g = tbl();
    let theirs = perm(&mut g, P1, "Serra Angel");
    g.player_mut(P1).regen_turn = Some(g.turn_stamp()); // a regeneration shield (Golgari Charm) is up
    cast_free(&mut g, P0, "Wrath of God");
    assert!(!g.perm(theirs).on_bf, "regeneration does not save it from Wrath of God");
}

#[test]
fn wrath_of_god_leaves_indestructible_creatures_a_mithril_coat_carrier_among_them() {
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    let coat = perm(&mut g, P0, COAT);
    assert_eq!(g.perm(coat).attached, Some(y));
    let other = perm(&mut g, P0, "Kor Duelist");
    cast_free(&mut g, P0, "Wrath of God");
    assert!(g.perm(y).on_bf, "indestructible by the Coat");
    assert!(!g.perm(other).on_bf);
    assert_eq!(g.perm(coat).attached, Some(y), "the Coat is still on it");
}

#[test]
fn wrath_of_god_leaves_a_creature_that_gained_indestructible_from_zack_fair() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    let z = perm(&mut g, P0, ZACK);
    let t = perm(&mut g, P0, "Kor Duelist");
    let u = perm(&mut g, P0, "Serra Angel");
    assert!(cloud::zack_sacrifice(&mut g, z, P0, t.0 as i64).unwrap());
    cast_free(&mut g, P0, "Wrath of God");
    assert!(g.perm(t).on_bf, "indestructible until end of turn");
    assert!(!g.perm(u).on_bf);
}

#[test]
fn flawless_maneuver_cast_first_saves_our_creatures_from_wrath_of_god() {
    // Flawless Maneuver: "Creatures you control gain indestructible until end of turn."
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let theirs = perm(&mut g, P1, "Serra Angel");
    cast_free(&mut g, P0, "Flawless Maneuver");
    assert!(indestructible(&g, t));
    cast_free(&mut g, P0, "Wrath of God");
    assert!(g.perm(t).on_bf && !g.perm(theirs).on_bf);
}

#[test]
fn stroke_of_midnight_destroys_a_nonland_permanent_and_its_controller_gets_a_token() {
    // "Destroy target nonland permanent. Its controller creates a 1/1 white Human creature token."
    let mut g = tbl();
    lands(&mut g, P1, "Island", 1, false);
    let a = perm(&mut g, P1, "Serra Angel");
    assert!(cast_on(&mut g, P0, "Stroke of Midnight", a));
    assert!(!g.perm(a).on_bf);
    let toks = tokens_of(&g, P1);
    assert_eq!(toks.len(), 1, "its controller gets the token");
    assert!(tokens_of(&g, P0).is_empty(), "not us");
    assert_eq!((epow(&g, toks[0]), etgh(&g, toks[0])), (1, 1));
    assert!(g.is_creature(toks[0]));
    assert_eq!(g.player(P1).lands.len(), 1, "the land is untouched");
}

#[test]
fn the_token_from_stroke_of_midnight_is_a_white_human() {
    let mut g = tbl();
    let a = perm(&mut g, P1, "Serra Angel");
    assert!(cast_on(&mut g, P0, "Stroke of Midnight", a));
    let tok = tokens_of(&g, P1)[0];
    assert_eq!(colors_of(&g, tok), Colors::from_letters("W"), "white");
    assert!(has_type(&g, tok, "human"), "Human");
}

#[test]
fn stroke_of_midnight_on_our_own_indestructible_creature_still_makes_us_a_token() {
    // "Its controller creates a 1/1 ... token": the token does not depend on the permanent being destroyed
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    perm(&mut g, P0, COAT);
    assert!(tokens_of(&g, P0).is_empty());
    assert!(cast_on(&mut g, P0, "Stroke of Midnight", y));
    assert!(g.perm(y).on_bf, "indestructible: it survives");
    assert_eq!(tokens_of(&g, P0).len(), 1, "we are its controller: we get the token");
}

#[test]
fn stroke_of_midnight_on_an_opponents_indestructible_permanent_gives_them_a_token_anyway() {
    let mut g = tbl();
    let coat = perm(&mut g, P1, COAT); // loose, but indestructible itself
    assert!(cast_on(&mut g, P0, "Stroke of Midnight", coat));
    assert!(g.perm(coat).on_bf, "an indestructible Equipment survives");
    assert_eq!(tokens_of(&g, P1).len(), 1);
}

// =================================================================================================================
// 12. Other white sources and a carrier with protection from white (a Sword of Hearth and Home carrier)
// Protection from white: "can't be targeted by white sources". Each white source below must skip the carrier, while the
// ability of a colourless Equipment (equip) still targets it.
// =================================================================================================================
/// a creature wearing a Sword of Hearth and Home ("protection from green and from white")
fn pro_white_carrier(g: &mut Game, name: &str) -> PermId {
    let c = perm(g, P0, name);
    let sword = perm(g, P0, SOHH);
    wear(g, sword, c);
    assert!(protected_from(g, c, Colors::from_letters("W")), "{name} has protection from white");
    c
}

/// the equip ability of a colourless Equipment (Sword of Fire and Ice) may still target `carrier`
fn colourless_equip_still_targets(g: &mut Game, carrier: PermId) {
    g.active = Some(P0); // equip at sorcery speed, on our own turn
    let sofi = perm(g, P0, SOFI);
    lands(g, P0, "Plains", 2, false);
    pool(g, P0, &[('W', 2)]);
    assert_eq!(check_equip(g, P0, sofi, carrier), None);
    assert!(sim_core::dsl::equip(g, P0, sofi, carrier, 2).unwrap());
    assert_eq!(g.perm(sofi).attached, Some(carrier), "an Equipment is colourless: protection from white is no bar");
}

#[test]
fn heliods_lifelink_ability_cannot_target_the_carrier_with_protection_from_white() {
    // Heliod, Sun-Crowned: "{1}{W}: Another target creature gains lifelink until end of turn." Heliod is white.
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    perm(&mut g, P0, "Serra Angel");
    let o = options_of(&mut g, h, P0);
    assert!(o.iter().any(|x| x.label.contains("Serra Angel gains lifelink")), "control: offered, {o:?}");
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    let c = pro_white_carrier(&mut g, "Serra Angel");
    let o = options_of(&mut g, h, P0);
    assert!(
        !o.iter().any(|x| x.label.contains("lifelink")),
        "no target: {:?}",
        o.iter().map(|x| &x.label).collect::<Vec<_>>()
    );
    assert!(!sim_core::impls::t4::heliod_grant_legal(&g, P0, h, c));
    colourless_equip_still_targets(&mut g, c);
}

#[test]
fn giver_of_runes_cannot_target_the_carrier_with_protection_from_white() {
    // Giver of Runes: "{T}: Another target creature you control gains protection from colorless or from the color of
    // your choice until end of turn." Giver is white. The AI uses it in response to removal aimed at the creature.
    let spell = |g: &Game| card(g, "Swords to Plowshares");
    let mut g = tbl();
    let m = perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P0, "Giver of Runes");
    g.active = Some(P1);
    let s = spell(&g);
    assert!(
        sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap(),
        "control: Giver saves it"
    );
    let mut g = tbl();
    let giver = perm(&mut g, P0, "Giver of Runes");
    let m = pro_white_carrier(&mut g, "Grave Titan");
    g.active = Some(P1);
    let s = spell(&g);
    assert!(
        !sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap(),
        "a white Giver can't target the creature with protection from white"
    );
    assert!(!g.perm(giver).tapped, "Giver was not used");
    colourless_equip_still_targets(&mut g, m);
}

#[test]
fn sunpearl_kirin_cannot_return_the_carrier_with_protection_from_white() {
    // Sunpearl Kirin: "When this creature enters, return up to one other target nonland permanent you control to its
    // owner's hand." Kirin is white. Control first.
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(!g.perm(c).on_bf, "control: without the Sword the AI returns Cloud (an enters creature)");
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, c);
    perm(&mut g, P0, "Sunpearl Kirin");
    assert!(g.perm(c).on_bf, "protection from white: Kirin's trigger can't target Cloud");
    colourless_equip_still_targets(&mut g, c);
}

#[test]
fn eerie_interlude_cannot_target_the_carrier_with_protection_from_white() {
    // Eerie Interlude: "Exile any number of target creatures you control. Return those cards to the battlefield under
    // their owner's control at the beginning of the next end step." It is white. The AI casts it in response to removal.
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let m = perm(&mut g, P0, "Grave Titan");
    hand(&mut g, P0, &["Eerie Interlude"]);
    g.active = Some(P1);
    let s = card(&g, "Swords to Plowshares");
    assert!(sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap(), "control: cast");
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let m = pro_white_carrier(&mut g, "Grave Titan");
    hand(&mut g, P0, &["Eerie Interlude"]);
    g.active = Some(P1);
    let s = card(&g, "Swords to Plowshares");
    assert!(
        !sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap(),
        "a white spell can't target the creature with protection from white"
    );
    assert!(in_hand(&g, P0, "Eerie Interlude"), "kept");
    assert!(g.perm(m).on_bf);
    colourless_equip_still_targets(&mut g, m);
}

#[test]
fn teleportation_circle_cannot_blink_the_carrier_with_protection_from_white() {
    // Teleportation Circle: "At the beginning of your end step, exile up to one target artifact or creature you
    // control, then return that card to the battlefield under your control." It is white. Control first.
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Teleportation Circle");
    fire(&mut g, Event::EndStep, P0);
    assert!(g.perm(c).cd.is_some());
    assert!(!g.perm(c).on_bf, "control: without the Sword the AI blinks Cloud");
    let mut g = tbl();
    let c = perm(&mut g, P0, CLOUD);
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, c);
    perm(&mut g, P0, "Teleportation Circle");
    fire(&mut g, Event::EndStep, P0);
    assert!(g.perm(c).on_bf, "protection from white: the Circle's trigger can't target Cloud");
    assert_eq!(g.perm(sword).attached, Some(c));
    colourless_equip_still_targets(&mut g, c);
}

// =================================================================================================================
// 13. Round 2: protectors that target the removal spell, and Court returning lands through the real land-entry path
// =================================================================================================================
#[test]
fn protectors_that_target_the_spell_still_save_a_carrier_with_protection_from_red_and_blue() {
    // Sword of Fire and Ice: "Equipped creature gets +2/+2 and has protection from red and from blue." Siren Stormtamer
    // ("{U}, Sacrifice: Counter target spell or ability that targets you or a creature you control") and Deflecting
    // Swat ("change the target of target spell or ability with a single target") target the removal, not the creature.
    let spell = |g: &Game| card(g, "Swords to Plowshares");
    for (protector, land, in_hand_card) in
        [("Siren Stormtamer", "Island", false), ("Deflecting Swat", "Mountain", true)]
    {
        let mut g = tbl();
        lands(&mut g, P0, land, 3, false);
        let m = perm(&mut g, P0, "Grave Titan");
        let sword = perm(&mut g, P0, SOFI);
        wear(&mut g, sword, m);
        assert!(protected_from(&g, m, Colors::from_letters("R")) && protected_from(&g, m, Colors::from_letters("U")));
        if in_hand_card {
            hand(&mut g, P0, &[protector]);
        } else {
            perm(&mut g, P0, protector);
        }
        g.active = Some(P1);
        let s = spell(&g);
        assert!(
            sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap(),
            "{protector} targets the spell, so protection on the creature does not stop it"
        );
    }
}

#[test]
fn not_of_this_world_still_counters_removal_aimed_at_a_carrier_with_protection_from_white() {
    // "Counter target spell or ability that targets a permanent you control." Colourless, free for power 7 or more.
    let mut g = tbl();
    let m = pro_white_carrier(&mut g, "Grave Titan"); // 6 + 2 power
    hand(&mut g, P0, &["Not of This World"]);
    g.active = Some(P1);
    let s = card(&g, "Swords to Plowshares");
    assert!(sim_core::ai::protect_response(&mut g, P0, m, "exile", Some(P1), Some(s)).unwrap());
    assert!(in_gy(&g, P0, "Not of This World"));
}

#[test]
fn a_white_protector_that_targets_the_creature_still_cannot_use_a_pro_white_carrier() {
    // Valorous Stance (white, "Target creature gains indestructible") targets the creature: the check stays
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let m = pro_white_carrier(&mut g, "Grave Titan");
    hand(&mut g, P0, &["Valorous Stance"]);
    g.active = Some(P1);
    let s = card(&g, "Swords to Plowshares");
    assert!(!sim_core::ai::protect_response(&mut g, P0, m, "destroy", Some(P1), Some(s)).unwrap());
    assert!(in_hand(&g, P0, "Valorous Stance"), "kept");
}

fn court_returning(g: &mut Game, land: &str) {
    perm(g, P0, COURT);
    assert_eq!(g.monarch, Some(P0));
    gy_add(g, P0, &[land]);
    fire(g, Event::Upkeep, P0);
}

#[test]
fn court_returns_isolated_chapel_tapped_without_a_plains_or_swamp() {
    // Isolated Chapel: "enters tapped unless you control a Plains or a Swamp."
    let mut g = tbl();
    court_returning(&mut g, "Isolated Chapel");
    let l = *g.player(P0).lands.last().expect("it came back");
    assert_eq!(g.perm(l).name, "Isolated Chapel");
    assert!(g.perm(l).tapped, "no Plains or Swamp: it enters tapped");
    // with a Plains it enters untapped
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 1, false);
    court_returning(&mut g, "Isolated Chapel");
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.perm(l).name, "Isolated Chapel");
    assert!(!g.perm(l).tapped, "a Plains: untapped");
}

#[test]
fn court_returns_a_legendary_land_and_yoshimaru_gets_his_counter() {
    // Takenuma, Abandoned Mire is a legendary land: "another legendary permanent you control enters"
    let mut g = tbl();
    let y = perm(&mut g, P0, YOSHI);
    court_returning(&mut g, "Takenuma, Abandoned Mire");
    assert!(g.player(P0).lands.iter().any(|&l| g.perm(l).name == "Takenuma, Abandoned Mire"));
    assert_eq!(plus(&g, y), 1, "the land entering counts");
}

#[test]
fn court_returns_a_basic_land_untapped_and_it_triggers_landfall() {
    // Rampaging Baloths: "Landfall - Whenever a land you control enters, you may create a 4/4 green Beast token."
    let mut g = tbl();
    perm(&mut g, P0, "Rampaging Baloths");
    court_returning(&mut g, "Plains");
    let l = *g.player(P0).lands.last().expect("it came back");
    assert_eq!(g.perm(l).name, "Plains");
    assert!(!g.perm(l).tapped, "a basic land enters untapped");
    assert_eq!(tokens_of(&g, P0).len(), 1, "landfall");
}

#[test]
fn steelshapers_gift_a_person_picks_among_the_equipment_cards_only() {
    // practice mode: "Search your library for an Equipment card": the offered cards are Equipment cards, and the
    // person's choice goes to the hand
    let mut g = tbl();
    set_library(&mut g, P0, &["Plains", "Sol Ring", SOHH, BUSTER, "Brainstorm"]);
    let h = person_rules(&mut g, &[("Search your library", BUSTER)]);
    let n = hand_len(&g, P0);
    cast_free(&mut g, P0, "Steelshaper's Gift");
    assert!(in_hand(&g, P0, BUSTER), "the person's pick");
    assert_eq!(hand_len(&g, P0), n + 1);
    assert!(!in_hand(&g, P0, SOHH) && !in_hand(&g, P0, "Sol Ring"));
    let offered: Vec<String> = h.asked().into_iter().flat_map(|r| r.choices.clone()).collect();
    assert!(
        !offered.iter().any(|c| c.contains("Sol Ring") || c.contains("Brainstorm") || c.contains("Plains")),
        "{offered:?}"
    );
    assert!(offered.iter().any(|c| c.contains(SOHH)), "{offered:?}");
}

// =================================================================================================================
// 14. Round 3: Momentary Blink, practice-mode targets, Secure the Scene, Zack Fair's choice of Equipment
// =================================================================================================================
/// a person at P0 who takes `action` at the first priority, answers a question whose prompt contains the first word of
/// a rule with the choice containing the second (else the first choice) and passes at every other priority
fn person_does(g: &mut Game, action: Value, rules: &[(&str, &str)]) -> Arc<ScriptHuman> {
    let rules: Vec<(String, String)> = rules.iter().map(|(p, c)| (p.to_string(), c.to_string())).collect();
    let first = Mutex::new(Some(action));
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(first.lock().unwrap().take().unwrap_or_else(|| json!({"do": "pass"})));
        }
        for (p, c) in &rules {
            if r.prompt.contains(p.as_str())
                && let Some(k) = r.choices.iter().position(|x| x.contains(c.as_str()))
            {
                return Answer::Pick(k);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    h
}

fn offered(h: &ScriptHuman, prompt: &str) -> Vec<String> {
    h.asked().into_iter().filter(|r| r.prompt.contains(prompt)).flat_map(|r| r.choices.clone()).collect()
}

#[test]
fn momentary_blink_is_not_offered_on_the_carrier_with_protection_from_white() {
    // Momentary Blink: "Exile target creature you control, then return it to the battlefield under its owner's
    // control." It is white (from the hand or by flashback).
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let s = perm(&mut g, P0, SFM);
    let c = hand(&mut g, P0, &["Momentary Blink"])[0];
    let f = g.registry.get(c).unwrap().hand_options.expect("Momentary Blink has a hand option");
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(o.iter().any(|x| x.label.contains("Momentary Blink")), "control: offered on a plain Stoneforge Mystic");
    let sword = perm(&mut g, P0, SOHH);
    wear(&mut g, sword, s);
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(
        !o.iter().any(|x| x.label.contains("Momentary Blink")),
        "{:?}",
        o.iter().map(|x| &x.label).collect::<Vec<_>>()
    );
    // and a direct cast (protection or value path) is refused, nothing paid
    assert!(!sim_core::impls::mine::momentary_blink_cast(&mut g, P0, s, "protection").unwrap());
    assert!(in_hand(&g, P0, "Momentary Blink") && g.perm(s).on_bf);
    colourless_equip_still_targets(&mut g, s);
}

#[test]
fn ephemerate_for_a_person_does_not_offer_the_carrier_with_protection_from_white() {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    let c = perm(&mut g, P0, "Kor Duelist");
    let t = pro_white_carrier(&mut g, TYMNA);
    hand(&mut g, P0, &["Ephemerate"]);
    pool(&mut g, P0, &[('W', 1)]);
    let h = person_does(&mut g, json!({"do": "cast", "card": 0}), &[]);
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    let o = offered(&h, "Ephemerate: exile and return which creature?");
    assert!(!o.is_empty() && o.iter().any(|x| x.contains("Kor Duelist")), "{o:?}");
    assert!(!o.iter().any(|x| x.contains("Tymna")), "the carrier is no target: {o:?}");
    assert!(g.perm(t).on_bf && c != t);
}

#[test]
fn kor_outfitter_for_a_person_does_not_offer_the_carrier_with_protection_from_white() {
    let mut g = tbl();
    let t = pro_white_carrier(&mut g, TYMNA);
    perm(&mut g, P0, "Serra Angel");
    perm(&mut g, P0, BOOTS);
    let h = person_rules(&mut g, &[]);
    perm(&mut g, P0, OUTFITTER);
    let o = offered(&h, "Kor Outfitter: attach it to which creature");
    assert!(o.iter().any(|x| x.contains("Serra Angel")), "{o:?}");
    assert!(!o.iter().any(|x| x.contains("Tymna")), "{o:?}");
    colourless_equip_still_targets(&mut g, t);
}

#[test]
fn ardenn_for_a_person_does_not_offer_the_carrier_with_protection_from_white() {
    let mut g = tbl();
    let t = pro_white_carrier(&mut g, TYMNA);
    perm(&mut g, P0, "Serra Angel");
    perm(&mut g, P0, BOOTS);
    perm(&mut g, P0, ARDENN);
    let h = person_rules(&mut g, &[]);
    fire(&mut g, Event::CombatStart, P0);
    let o = offered(&h, "Ardenn (beginning of combat): attach Equipment you control to which creature?");
    assert!(o.iter().any(|x| x.contains("Serra Angel")), "{o:?}");
    assert!(!o.iter().any(|x| x.contains("Tymna")), "{o:?}");
    colourless_equip_still_targets(&mut g, t);
}

#[test]
fn secure_the_scene_gives_the_controller_a_white_soldier() {
    // "Exile target nonland permanent. Its controller creates a 1/1 white Soldier creature token."
    let mut g = tbl();
    let a = perm(&mut g, P1, "Serra Angel");
    assert!(cast_on(&mut g, P0, "Secure the Scene", a));
    assert!(!g.perm(a).on_bf);
    let toks = tokens_of(&g, P1);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(colors_of(&g, tok), Colors::from_letters("W"), "white");
    assert!(has_type(&g, tok, "soldier"), "Soldier");
    assert_eq!((epow(&g, tok), etgh(&g, tok)), (1, 1));
    assert!(tokens_of(&g, P0).is_empty());
}

#[test]
fn beast_within_and_cavalier_of_dawn_tokens_are_unchanged() {
    // Beast Within: "Its controller creates a 3/3 green Beast creature token" (the engine's plain token: no type or
    // colour of its own); the override for Stroke and Secure the Scene changes only those cards
    let mut g = tbl();
    let a = perm(&mut g, P1, "Serra Angel");
    assert!(cast_on(&mut g, P0, "Beast Within", a));
    let toks = tokens_of(&g, P1);
    assert_eq!(toks.len(), 1);
    assert_eq!((epow(&g, toks[0]), etgh(&g, toks[0])), (3, 3));
    assert!(!has_type(&g, toks[0], "soldier") && !has_type(&g, toks[0], "human"));
}

fn zack_table() -> (Game, PermId, PermId, PermId, PermId) {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    let z = perm(&mut g, P0, ZACK);
    let (a, b) = (perm(&mut g, P0, SOFI), perm(&mut g, P0, BUSTER));
    wear(&mut g, a, z);
    wear(&mut g, b, z);
    let t = perm(&mut g, P0, TYMNA);
    pool(&mut g, P0, &[('C', 1)]);
    (g, z, a, b, t)
}

#[test]
fn zack_fair_for_a_person_with_two_equipment_asks_which_one_moves() {
    let (mut g, z, a, b, t) = zack_table();
    let (lesser, other) =
        if sim_core::engine::values::pval(&g, a) <= sim_core::engine::values::pval(&g, b) { (a, b) } else { (b, a) };
    let name = g.perm(lesser).name.to_string();
    let i = g.player(P0).perms.iter().position(|&x| x == z).unwrap();
    let h = person_does(
        &mut g,
        json!({"do": "use", "perm": i}),
        &[("Zack Fair: which creature", "Tymna"), ("attach which Equipment", &name)],
    );
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    let o = offered(&h, "Zack Fair: attach which Equipment");
    assert_eq!(o.len(), 2, "both Equipment are offered: {o:?}");
    assert!(!g.perm(z).on_bf, "sacrificed");
    assert_eq!(g.perm(lesser).attached, Some(t), "the one the person chose moved");
    assert_ne!(g.perm(other).attached, Some(t), "and only that one");
}

#[test]
fn zack_fair_for_a_person_with_one_equipment_asks_nothing() {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    let z = perm(&mut g, P0, ZACK);
    let a = perm(&mut g, P0, SOFI);
    wear(&mut g, a, z);
    let t = perm(&mut g, P0, TYMNA);
    pool(&mut g, P0, &[('C', 1)]);
    let i = g.player(P0).perms.iter().position(|&x| x == z).unwrap();
    let h = person_does(&mut g, json!({"do": "use", "perm": i}), &[("Zack Fair: which creature", "Tymna")]);
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(offered(&h, "attach which Equipment").is_empty());
    assert_eq!(g.perm(a).attached, Some(t));
}
