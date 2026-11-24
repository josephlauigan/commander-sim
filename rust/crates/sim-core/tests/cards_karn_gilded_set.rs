//! The other cards of the karn_gilded deck (impls/karn_gilded_cards.rs, the compiled Shorikai, the {2/X}-free ones of E5):
//! Infinite Guideline Station, Breya, Etherium Shaper, Gold-Forged Thopteryx, Shorikai, Genesis Engine, The Reaper, King
//! No More, Herd Heirloom, Twitching Doll and Transguild Courier. One test for each part of each Oracle text, on a
//! hand-built position; the expectation is the Oracle text, not the code.

mod play_common;
use play_common::*;
use sim_core::ai::act;
use sim_core::cardcode;
use sim_core::dsl;
use sim_core::engine::mana::{self, can_pay, can_pay_spell, cost_of};
use sim_core::engine::{chars, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::state::{DataKey, Step, Val};

const KARN: &str = "Karn, Gilded Guardian";
const STATION: &str = "Infinite Guideline Station";
const BREYA: &str = "Breya, Etherium Shaper";
const THOPTERYX: &str = "Gold-Forged Thopteryx";
const SHORIKAI: &str = "Shorikai, Genesis Engine";
const REAPER: &str = "The Reaper, King No More";
const HEIRLOOM: &str = "Herd Heirloom";
const DOLL: &str = "Twitching Doll";
const COURIER: &str = "Transguild Courier";

/// P0 is a five-colour deck (any colour = any of the five), P1 an opponent
fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn ready(g: &mut Game, m: PermId) {
    let x = g.perm_mut(m);
    x.sick = false;
    x.tapped = false;
}

fn types_of(g: &Game, m: PermId) -> sim_core::cards::Types {
    values::ptypes(g, m)
}

fn colour_letters(g: &Game, m: PermId) -> String {
    "WUBRG".chars().filter(|&c| chars::colors(g, m).has(c)).collect()
}

fn named(g: &Game, p: PlayerId, n: &str) -> Vec<PermId> {
    perms_named(g, p, n)
}

// ======================================================== Transguild Courier
#[test]
fn courier_is_every_colour_so_it_counts_as_multicoloured() {
    let mut g = tbl();
    let c = perm(&mut g, P0, COURIER);
    assert_eq!(colour_letters(&g, c), "WUBRG", "Transguild Courier is all colors");
    assert_eq!(sim_core::impls::karn_gilded_cards::multicoloured(&g, P0), 1);
}

// ======================================================== Infinite Guideline Station
#[test]
fn station_makes_robots_per_multicoloured() {
    let mut g = tbl();
    perm(&mut g, P0, KARN); // five colours
    perm(&mut g, P0, "Llanowar Elves"); // one colour: not counted
    let thopter = perm(&mut g, P0, "Sol Ring"); // colourless: not counted
    let st = perm(&mut g, P0, STATION);
    // multicoloured: Karn and the Station itself
    let robots = named(&g, P0, "Token");
    assert_eq!(robots.len(), 2, "one tapped Robot for each of Karn and the Station");
    for &r in &robots {
        assert!(g.perm(r).tapped, "the Robots enter tapped");
        assert_eq!(values::epow(&g, r), 2);
        assert_eq!(values::etgh(&g, r), 2);
        assert!(chars::colors(&g, r).is_empty(), "colourless");
        assert!(sim_core::impls::karn_cards::is_art(&g, r), "an artifact creature");
        assert!(g.is_creature(r));
        assert!(g.perm(r).ttypes.contains(&"robot"));
    }
    assert!(on(&g, st) && on(&g, thopter));
    // the Courier is every colour: one more is counted the next time
    perm(&mut g, P0, COURIER);
    assert_eq!(sim_core::impls::karn_gilded_cards::multicoloured(&g, P0), 3);
}

#[test]
fn station_counts_when_the_trigger_resolves_with_itself_included() {
    let mut g = tbl();
    perm(&mut g, P0, STATION);
    assert_eq!(named(&g, P0, "Token").len(), 1, "alone it is its own multicoloured permanent");
}

#[test]
fn station_is_a_noncreature_until_twelve_charge_counters_then_a_flying_7_15() {
    let mut g = tbl();
    let st = perm(&mut g, P0, STATION);
    assert!(!g.is_creature(st), "a Spacecraft is not a creature below 12 counters");
    let a = token(&mut g, P0, 6);
    let b = token(&mut g, P0, 6);
    g.player_mut(P0).hand.clear();
    let os = options(&mut g, st, P0, Some(false));
    assert_eq!(os.len(), 1, "the AI stations with creatures that can't attack: the summoning-sick ones");
    assert!(run(&mut g, P0, &os[0]));
    assert!(g.perm(a).tapped || g.perm(b).tapped);
    assert_eq!(g.perm(st).data.int(DataKey::Charge), 6, "charge counters equal to the tapped creature's power");
    assert!(!g.is_creature(st));
    let os = options(&mut g, st, P0, Some(false));
    assert!(run(&mut g, P0, &os[0]));
    assert_eq!(g.perm(st).data.int(DataKey::Charge), 12);
    assert!(g.is_creature(st), "an artifact creature at 12+");
    assert!(types_of(&g, st).has(sim_core::cards::Types::ARTIFACT));
    assert_eq!((values::epow(&g, st), values::etgh(&g, st)), (7, 15));
    assert!(chars::has_keyword(&g, st, "flying"), "12+ | Flying");
    assert!(options(&mut g, st, P0, Some(false)).is_empty(), "the AI stops at 12");
    // it ends with the counters
    g.perm_mut(st).data.set(DataKey::Charge, Val::Int(11));
    assert!(!g.is_creature(st));
    assert!(!chars::has_keyword(&g, st, "flying"));
}

#[test]
fn station_only_as_a_sorcery() {
    let mut g = tbl();
    let st = perm(&mut g, P0, STATION);
    token(&mut g, P0, 3);
    g.step = Step::Combat;
    assert!(options(&mut g, st, P0, Some(false)).is_empty(), "not in combat");
    g.step = Step::Main1;
    g.active = Some(P1);
    assert!(options(&mut g, st, P0, Some(false)).is_empty(), "not on another player's turn");
}

#[test]
fn station_attack_draws_for_each_multicoloured_permanent() {
    let mut g = tbl();
    perm(&mut g, P0, KARN);
    let st = perm(&mut g, P0, STATION);
    g.perm_mut(st).data.set(DataKey::Charge, Val::Int(12));
    ready(&mut g, st);
    let before = g.player(P0).hand.len();
    let c = g.perm(st).cd.unwrap();
    let f = g.registry.get(c).unwrap().attack.unwrap();
    f(&mut g, st, P0, &[st], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 2, "Karn and the Station");
    // a different attacker: nothing
    let other = token(&mut g, P0, 2);
    f(&mut g, st, P0, &[other], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 2);
}

// ======================================================== Breya, Etherium Shaper
#[test]
fn breya_thopters_are_blue_artifacts() {
    let mut g = tbl();
    perm(&mut g, P0, BREYA);
    let ts: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(ts.len(), 2);
    for &t in &ts {
        assert_eq!(colour_letters(&g, t), "U", "blue");
        assert!(sim_core::impls::karn_cards::is_art(&g, t), "artifact creature tokens");
        assert!(g.is_creature(t));
        assert!(g.perm(t).ttypes.contains(&"thopter"));
        assert!(g.perm(t).fly);
        assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (1, 1));
    }
    // Karn counts artifacts: Breya and two Thopters
    assert_eq!(sim_core::impls::karn::artifacts(&g, P0), 3);
}

fn breya_set(life_p1: i32) -> (Game, PermId) {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let b = perm(&mut g, P0, BREYA);
    ready(&mut g, b);
    g.player_mut(P1).life = life_p1;
    (g, b)
}

#[test]
fn breya_deals_three_to_a_player_for_two_artifacts_and_two_mana() {
    let (mut g, b) = breya_set(3);
    let os = options(&mut g, b, P0, Some(false));
    assert_eq!(os.len(), 1);
    assert!(run(&mut g, P0, &os[0]));
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive, "3 damage finishes a player on 3");
    assert!(on(&g, b), "Breya herself is never the sacrifice");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 0, "the two Thopters were sacrificed");
    assert_eq!(mana::total_mana(&g, P0, false), 0, "{{2}} was paid");
}

#[test]
fn breya_minus_four_kills_a_creature_with_four_toughness_or_less() {
    let (mut g, b) = breya_set(40);
    let big = perm(&mut g, P1, "Glorybringer");
    let os = options(&mut g, b, P0, Some(false));
    assert_eq!(os.len(), 1, "a 4/4 worth killing");
    assert!(run(&mut g, P0, &os[0]));
    assert!(!on(&g, big), "-4/-4 kills a 4/4");
    assert_eq!(g.player(P1).life, 40, "no damage mode");
}

#[test]
fn breya_gains_five_life_when_low() {
    let (mut g, b) = breya_set(40);
    g.player_mut(P0).life = 6;
    let os = options(&mut g, b, P0, Some(false));
    assert_eq!(os.len(), 1);
    assert!(run(&mut g, P0, &os[0]));
    assert_eq!(g.player(P0).life, 11);
}

#[test]
fn breya_is_not_offered_with_nothing_to_gain_and_never_sacrifices_treasure_counters() {
    let (mut g, b) = breya_set(40);
    assert!(options(&mut g, b, P0, Some(false)).is_empty(), "40 life, no creature to kill, a healthy 40-life us");
    // only Breya herself is an artifact permanent: Treasure tokens are counters on the player, not sacrificed to it
    for t in named(&g, P0, "Token") {
        zones::die(&mut g, t, "sac").unwrap();
    }
    zones::add_treasure(&mut g, P0, 3).unwrap();
    g.player_mut(P0).life = 3;
    g.player_mut(P1).life = 3;
    assert!(options(&mut g, b, P0, Some(false)).is_empty(), "two artifacts needed: Treasures are not permanents");
    assert_eq!(g.player(P0).treasures, 3);
}

// ======================================================== Gold-Forged Thopteryx
#[test]
fn thopteryx_gives_karn_ward_2() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    assert_eq!(cardcode::aura_ward(&g, karn), 0, "no Thopteryx yet");
    let t = perm(&mut g, P0, THOPTERYX);
    assert_eq!(cardcode::aura_ward(&g, karn), 2, "each legendary permanent has ward {{2}}");
    let station = perm(&mut g, P0, STATION);
    assert_eq!(cardcode::aura_ward(&g, station), 2, "any legendary permanent, noncreatures too");
    assert_eq!(cardcode::aura_ward(&g, t), 0, "it is not legendary itself");
    let other = perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(cardcode::aura_ward(&g, other), 0, "nonlegendary");
    // two of them: two wards of {2}
    perm(&mut g, P0, THOPTERYX);
    assert_eq!(cardcode::aura_ward(&g, karn), 4);
    // an opponent's legendary permanent is not covered
    let theirs = perm(&mut g, P1, KARN);
    assert_eq!(cardcode::aura_ward(&g, theirs), 0);
    // the card itself has no printed ward
    assert_eq!(g.db.get(g.perm(t).cd.unwrap()).ward, 0);
}

#[test]
fn thopteryx_ward_counters_an_opponents_effect_it_cannot_pay_for() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    assert!(sim_core::engine::ward::ward_affordable(&g, P1, karn, 0, ""), "no ward, no payment");
    perm(&mut g, P0, THOPTERYX);
    assert!(!sim_core::engine::ward::ward_affordable(&g, P1, karn, 0, ""), "P1 has no mana for ward {{2}}");
    lands(&mut g, P1, "Wastes", 2, false);
    assert!(sim_core::engine::ward::ward_affordable(&g, P1, karn, 0, ""), "two lands pay it");
    assert!(sim_core::engine::ward::pay_ward(&mut g, P1, karn).unwrap());
    assert_eq!(g.player(P1).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0, "it cost {{2}}");
    assert!(!sim_core::engine::ward::pay_ward(&mut g, P1, karn).unwrap(), "the second try can't pay");
}

#[test]
fn thopteryx_is_a_flying_lifelinking_artifact_creature() {
    let mut g = tbl();
    let t = perm(&mut g, P0, THOPTERYX);
    assert!(g.is_creature(t));
    assert!(types_of(&g, t).has(sim_core::cards::Types::ARTIFACT));
    assert!(chars::has_keyword(&g, t, "flying") && chars::has_keyword(&g, t, "lifelink"));
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (1, 3));
}

// ======================================================== Shorikai, Genesis Engine
#[test]
fn shorikai_draws_two_discards_one_and_makes_a_pilot() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 1, false);
    let s = perm(&mut g, P0, SHORIKAI);
    let os: Vec<Opt> = dsl::ability_options(&mut g, P0, true)
        .unwrap()
        .into_iter()
        .filter(|o| matches!(o.act, Some(Action::Dsl { src, .. }) if src == s))
        .collect();
    assert_eq!(os.len(), 1);
    let hand = g.player(P0).hand.len();
    let Some(Action::Dsl { src, idx }) = os[0].act.clone() else { panic!() };
    assert!(dsl::activate(&mut g, P0, src, idx).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand + 1, "two drawn, one discarded");
    assert!(g.perm(s).tapped, "{{T}}");
    assert_eq!(mana::total_mana(&g, P0, false), 0, "{{1}} was paid");
    let pilots: Vec<PermId> =
        named(&g, P0, "Token").into_iter().filter(|&m| g.perm(m).ttypes.contains(&"pilot")).collect();
    assert_eq!(pilots.len(), 1, "a Pilot token");
    assert!(chars::colors(&g, pilots[0]).is_empty(), "colourless");
    assert_eq!((values::epow(&g, pilots[0]), values::etgh(&g, pilots[0])), (1, 1));
    assert!(g.is_creature(pilots[0]));
    assert!(!g.is_creature(s), "a Vehicle is no creature until crewed (Crew 8 is not modelled)");
}

// ======================================================== The Reaper, King No More
#[test]
fn reaper_puts_a_counter_on_up_to_two_opposing_creatures() {
    let mut g = tbl();
    let a = perm(&mut g, P1, "Glorybringer");
    let b = perm(&mut g, P1, "Grave Titan");
    let c = perm(&mut g, P1, "Ainok Strike Leader");
    let own = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, REAPER);
    let marked: Vec<PermId> = [a, b, c].into_iter().filter(|&m| g.perm(m).plus == -1).collect();
    assert_eq!(marked.len(), 2, "up to two creatures");
    assert!(g.perm(c).plus == 0, "the two worth most");
    assert_eq!(g.perm(own).plus, 0, "never our own");
}

#[test]
fn reaper_takes_dying_countered_creature_once_per_turn() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER);
    let a = perm(&mut g, P1, "Ainok Strike Leader");
    let b = perm(&mut g, P1, "Amrou Seekers");
    let plain = perm(&mut g, P1, "Ballista Squad");
    g.perm_mut(a).plus = -1;
    g.perm_mut(b).plus = -1;
    zones::die(&mut g, plain, "destroy").unwrap();
    assert!(!has_perm(&g, P0, "Ballista Squad"), "no -1/-1 counter on it: nothing");
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(has_perm(&g, P0, "Ainok Strike Leader"), "it comes back under the Reaper's controller");
    let m = perms_named(&g, P0, "Ainok Strike Leader")[0];
    assert_eq!(g.perm(m).owner, P0);
    assert_eq!(g.perm(m).orig, P1, "it is still P1's card");
    assert!(!g.player(P1).gy.contains(&card(&g, "Ainok Strike Leader")));
    zones::die(&mut g, b, "destroy").unwrap();
    assert!(!has_perm(&g, P0, "Amrou Seekers"), "only once each turn");
    assert!(g.player(P1).gy.contains(&card(&g, "Amrou Seekers")));
    // the next turn it works again
    let d = perm(&mut g, P1, "Amrou Seekers");
    g.perm_mut(d).plus = -1;
    g.round += 1;
    zones::die(&mut g, d, "destroy").unwrap();
    assert!(has_perm(&g, P0, "Amrou Seekers"), "the next turn: once more");
}

#[test]
fn reaper_ignores_its_own_controllers_creatures_and_tokens() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER);
    let mine = perm(&mut g, P0, "Ainok Strike Leader");
    g.perm_mut(mine).plus = -1;
    zones::die(&mut g, mine, "destroy").unwrap();
    assert!(!has_perm(&g, P0, "Ainok Strike Leader"), "an opponent's creature only");
    let t = token(&mut g, P1, 2);
    g.perm_mut(t).plus = -1;
    let before = g.player(P0).perms.len();
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.player(P0).perms.len(), before, "a token has no card to put onto the battlefield");
}

#[test]
fn reaper_etb_kills_x_one_creatures_and_takes_one() {
    let mut g = tbl();
    let a = perm(&mut g, P1, "Llanowar Elves");
    let b = perm(&mut g, P1, "Elvish Mystic");
    perm(&mut g, P0, REAPER);
    let mine = perms_named(&g, P0, "Llanowar Elves").len() + perms_named(&g, P0, "Elvish Mystic").len();
    assert!(!on(&g, a) || !on(&g, b), "a -1/-1 counter on each kills a 1/1");
    // both died at once: one comes back under the Reaper's controller, the other stays in the graveyard (once each turn)
    assert_eq!(mine, 1);
}

// ======================================================== Herd Heirloom
fn karn_in_hand(g: &mut Game) -> (sim_core::ids::CardId, sim_core::ids::CardId) {
    let k = hand(g, P0, &[KARN])[0];
    let s = hand(g, P0, &[STATION])[0];
    (k, s)
}

#[test]
fn heirloom_mana_pays_karn_not_station() {
    let mut g = tbl();
    let (karn, station) = karn_in_hand(&mut g);
    // Island, Swamp, Mountain, Forest: the four colours; the fifth, white, only the Heirloom can make
    for b in ["Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, b, 1, false);
    }
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let (kg, kp) = cost_of(&g, P0, karn);
    assert!(
        can_pay_spell(&g, P0, karn, kg, &kp, false),
        "Karn (a creature spell): the Heirloom's one mana is the white pip, the lands the other four"
    );
    let (sg, sp) = cost_of(&g, P0, station);
    assert_eq!((sg, sp.len()), (0, 5));
    assert!(
        !can_pay_spell(&g, P0, station, sg, &sp, false),
        "the Station is an artifact spell, not a creature spell: the Heirloom's mana can't pay it"
    );
    // the same lands with a Plains in its place pay for the Station
    g.perm_mut(h).tapped = true;
    lands(&mut g, P0, "Plains", 1, false);
    assert!(can_pay_spell(&g, P0, station, sg, &sp, false));
}

#[test]
fn heirloom_mana_is_for_creature_spells_only_not_for_abilities() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    assert!(!can_pay(&g, P0, 1, "", false), "an ability or a tax: not a spell");
    assert_eq!(mana::total_mana(&g, P0, false), 0);
    let bears = hand(&mut g, P0, &["Llanowar Elves"])[0];
    assert!(can_pay_spell(&g, P0, bears, 0, "G", false), "a creature spell: any colour");
    let ring = hand(&mut g, P0, &["Rhystic Study"])[0];
    assert!(!can_pay_spell(&g, P0, ring, 0, "U", false), "an enchantment spell: no");
}

#[test]
fn heirloom_gives_trample_and_a_card_on_combat_damage() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let small = token(&mut g, P0, 3);
    let big = token(&mut g, P0, 5);
    ready(&mut g, small);
    ready(&mut g, big);
    let os = options(&mut g, h, P0, Some(false));
    assert_eq!(os.len(), 1);
    assert!(run(&mut g, P0, &os[0]));
    assert!(g.perm(h).tapped, "{{T}}");
    assert!(chars::has_keyword(&g, big, "trample"), "power 4 or more: the 5/5");
    assert!(!chars::has_keyword(&g, small, "trample"), "not the 3/3");
    let before = g.player(P0).hand.len();
    cardcode::combat_damage_cards(&mut g, P0, small, P1, 3).unwrap();
    assert_eq!(g.player(P0).hand.len(), before, "only the creature it targeted draws");
    cardcode::combat_damage_cards(&mut g, P0, big, P1, 5).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1, "a card for combat damage to a player");
    // untapped Heirloom gone: the grant stays on the creature
    zones::die(&mut g, h, "destroy").unwrap();
    cardcode::combat_damage_cards(&mut g, P0, big, P1, 5).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 2, "the grant is the creature's, not the Heirloom's");
}

#[test]
fn heirloom_needs_a_creature_with_power_four() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let small = token(&mut g, P0, 3);
    ready(&mut g, small);
    assert!(options(&mut g, h, P0, Some(false)).is_empty(), "a 3/3 is no target");
}

// ======================================================== Twitching Doll
fn doll(g: &mut Game) -> PermId {
    let d = perm(g, P0, DOLL);
    ready(g, d);
    d
}

#[test]
fn doll_taps_for_mana_and_gets_a_nest_counter() {
    let mut g = tbl();
    let d = doll(&mut g);
    let sparks = hand(&mut g, P0, &["Llanowar Elves"])[0];
    assert!(can_pay_spell(&g, P0, sparks, 0, "G", false), "one mana of any colour");
    assert!(mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert!(g.perm(d).tapped);
    assert_eq!(g.perm(d).data.int(DataKey::Counters), 1, "a nest counter");
    // no Spider token from the compiled enters tag any more
    assert!(named(&g, P0, "Token").is_empty());
}

#[test]
fn doll_sacrifices_for_a_spider_per_counter_at_sorcery_speed() {
    let mut g = tbl();
    let d = doll(&mut g);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(3));
    let os = options(&mut g, d, P0, Some(false));
    assert_eq!(os.len(), 1, "three counters: worth it");
    assert!(run(&mut g, P0, &os[0]));
    assert!(!on(&g, d), "sacrificed");
    let spiders: Vec<PermId> = named(&g, P0, "Token");
    assert_eq!(spiders.len(), 3);
    for &s in &spiders {
        assert_eq!((values::epow(&g, s), values::etgh(&g, s)), (2, 2));
        assert_eq!(colour_letters(&g, s), "G");
        assert!(chars::has_keyword(&g, s, "reach"));
        assert!(g.perm(s).ttypes.contains(&"spider"));
    }
}

#[test]
fn doll_cannot_sacrifice_when_tapped_sick_or_out_of_a_main_phase() {
    let mut g = tbl();
    let d = doll(&mut g);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(4));
    g.perm_mut(d).tapped = true;
    assert!(options(&mut g, d, P0, Some(false)).is_empty(), "tapped");
    g.perm_mut(d).tapped = false;
    g.perm_mut(d).sick = true;
    assert!(options(&mut g, d, P0, Some(false)).is_empty(), "summoning sick: no {{T}} ability");
    g.perm_mut(d).sick = false;
    g.step = Step::Combat;
    assert!(options(&mut g, d, P0, Some(false)).is_empty(), "sorcery speed only");
    g.step = Step::Main1;
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(1));
    assert!(options(&mut g, d, P0, Some(false)).is_empty(), "the AI waits for three counters");
}

// ======================================================== Herd Heirloom: a person's pool
#[test]
fn a_person_taps_the_heirloom_for_mana_that_pays_only_creature_spells() {
    use sim_core::human::mana::{Spending, sources, tap};
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let src = sources(&g, P0).into_iter().find(|s| s.name == HEIRLOOM).expect("the Heirloom is listed as a source");
    assert_eq!(src.restricted, Some("creature"));
    assert!(tap(&mut g, P0, src.id as i64, Some('G')).unwrap().is_none());
    assert!(g.perm(h).tapped);
    assert!(pool_text(&g, P0).contains("for creature spells"), "{}", pool_text(&g, P0));
    let elves = card(&g, "Llanowar Elves");
    let study = card(&g, "Rhystic Study");
    let mut pool = std::mem::take(&mut g.player_mut(P0).pool);
    pool.spending = Some(Spending::Card(study));
    assert!(!pool.can_pay(&g, P0, 1, ""), "an enchantment spell can't use it");
    assert!(!pool.can_pay(&g, P0, 0, "G"));
    pool.spending = None;
    assert!(!pool.can_pay(&g, P0, 1, ""), "an ability can't use it");
    pool.spending = Some(Spending::Card(elves));
    assert!(pool.can_pay(&g, P0, 0, "G"), "a creature spell can");
    assert!(pool.pay(&g, P0, 0, "G"));
    assert_eq!(pool.total(), 0);
    g.player_mut(P0).pool = pool;
}

#[test]
fn creature_spell_restriction_is_used_by_the_heirloom_alone() {
    use sim_core::engine::pool::Restr;
    let g = tbl();
    let users: Vec<String> = (0..g.db.len())
        .map(|i| sim_core::ids::CardId(i as u16))
        .filter(|&c| g.db.get(c).mana_restr == Restr::CreatureSpell)
        .map(|c| g.db.get(c).name.to_string())
        .collect();
    assert_eq!(users, vec![HEIRLOOM.to_string()]);
}
