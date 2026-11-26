//! Haste lets a summoning-sick creature use its {T} abilities (CR 302.6), however the haste is granted: a static ability
//! (The Fire Crystal), Equipment (Lightning Greaves, Swiftfoot Boots), or "as though it had haste" (Shang-Chi, Thousand-Year
//! Elixir). `combat::tap_sick` is the one question every {T} check asks; it reads, and never clears, `sick`.
//! The sites it replaced: the AI's mana now (`mana::mana_units`) and next turn (`mana::next_turn_perm_units`, which must
//! agree), the compiled {T} abilities (`dsl::activate`), the AI's Giver of Runes (`ai::protect_response`), Twitching Doll's
//! sacrifice, and a person's {T} abilities (`human::cards::abilities`).

mod play_common;
use play_common::*;
use sim_core::engine::combat;
use sim_core::engine::mana::{next_turn_perm_units, total_mana};
use sim_core::engine::zones::{self, Enter};
use sim_core::state::{DataKey, Step, Val};

const CRYSTAL: &str = "The Fire Crystal";
const ELVES: &str = "Llanowar Elves";

fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

/// a creature of p's that came this turn (summoning sick)
fn sick(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    zones::enter(g, p, c, Enter { sick: true, ..Default::default() }).unwrap()
}

fn mana_now(g: &Game) -> u32 {
    total_mana(g, P0, false)
}

fn equip(g: &mut Game, gear: &str, m: PermId) -> PermId {
    let e = perm(g, P0, gear);
    g.perm_mut(e).attached = Some(m);
    e
}

// ======================================================== the predicate
#[test]
fn a_sick_creature_is_tap_sick_unless_something_gives_it_haste() {
    let mut g = tbl();
    let m = sick(&mut g, P0, ELVES);
    assert!(g.perm(m).sick && combat::tap_sick(&g, m));
    let ready = perm(&mut g, P0, ELVES);
    assert!(!g.perm(ready).sick && !combat::tap_sick(&g, ready), "not sick: nothing to ask");
    perm(&mut g, P0, CRYSTAL);
    assert!(!combat::tap_sick(&g, m), "the Crystal's haste");
    assert!(g.perm(m).sick, "and the flag is never cleared");
}

#[test]
fn a_noncreature_permanent_is_never_tap_sick() {
    let mut g = tbl();
    let ring = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(ring).sick = true;
    assert!(!combat::tap_sick(&g, ring));
    assert_eq!(mana_now(&g), 2, "it taps for mana the turn it came");
}

// ======================================================== mana now
#[test]
fn a_sick_dork_under_the_crystal_adds_mana_and_stops_when_the_crystal_leaves() {
    let mut g = tbl();
    let elves = sick(&mut g, P0, ELVES);
    assert_eq!(mana_now(&g), 0, "control: summoning sick, no mana");
    let c = perm(&mut g, P0, CRYSTAL);
    assert_eq!(mana_now(&g), 1, "the Crystal's haste lets it tap");
    zones::die(&mut g, c, "destroy").unwrap();
    assert_eq!(mana_now(&g), 0, "the Crystal left: sick again");
    assert!(g.perm(elves).sick, "its flag never changed");
}

#[test]
fn lightning_greaves_and_swiftfoot_boots_let_a_sick_dork_tap() {
    for gear in ["Lightning Greaves", "Swiftfoot Boots"] {
        let mut g = tbl();
        let birds = sick(&mut g, P0, "Birds of Paradise");
        assert_eq!(mana_now(&g), 0, "{gear}: control");
        equip(&mut g, gear, birds);
        assert_eq!(mana_now(&g), 1, "{gear}: equipped, so it taps the turn it came");
    }
}

#[test]
fn thousand_year_elixir_lets_a_sick_dork_tap() {
    let mut g = tbl();
    sick(&mut g, P0, ELVES);
    assert_eq!(mana_now(&g), 0);
    perm(&mut g, P0, "Thousand-Year Elixir");
    assert_eq!(mana_now(&g), 1, "'as though those creatures had haste'");
}

#[test]
fn a_crystal_token_copy_of_a_dork_taps_the_turn_it_is_made() {
    let mut g = tbl();
    let elves = perm(&mut g, P0, ELVES);
    assert_eq!(mana_now(&g), 1);
    zones::token_copy_of(&mut g, P0, elves, &[]).unwrap();
    assert_eq!(mana_now(&g), 1, "control: the sick token copy can't tap without haste");
    perm(&mut g, P0, CRYSTAL);
    assert_eq!(mana_now(&g), 2, "with the Crystal both tap");
}

// ======================================================== mana next turn agrees
#[test]
fn next_turn_lists_exactly_the_dorks_mana_now_leaves_out() {
    let mut g = tbl();
    sick(&mut g, P0, ELVES);
    assert_eq!((mana_now(&g), next_turn_perm_units(&g, P0).len()), (0, 1), "sick, no haste: next turn only");
    perm(&mut g, P0, CRYSTAL);
    assert_eq!((mana_now(&g), next_turn_perm_units(&g, P0).len()), (1, 0), "sick with haste: now, not again");
}

// ======================================================== compiled {T} abilities
/// a sick Hangarback Walker with a +1/+1 counter and an untapped Forest: "{1}, {T}: put a +1/+1 counter on it" (compiled)
fn walker(g: &mut Game) -> (PermId, PermId) {
    let c = take(g, P0, "Hangarback Walker");
    let m = zones::enter(g, P0, c, Enter { sick: true, plus: 1, ..Default::default() }).unwrap();
    (m, lands(g, P0, "Forest", 1, false)[0])
}

#[test]
fn a_sick_creatures_compiled_tap_ability_needs_haste() {
    let mut g = tbl();
    let (w, forest) = walker(&mut g);
    assert!(!sim_core::dsl::activate(&mut g, P0, w, 1).unwrap(), "control: summoning sick");
    assert!(!g.perm(w).tapped && !g.perm(forest).tapped, "nothing was spent");
    perm(&mut g, P0, CRYSTAL);
    assert!(sim_core::dsl::activate(&mut g, P0, w, 1).unwrap(), "the Crystal's haste");
    assert!(g.perm(w).tapped && g.perm(forest).tapped);
}

#[test]
fn greaves_on_a_sick_hangarback_walker_lets_it_tap() {
    let mut g = tbl();
    let (w, _) = walker(&mut g);
    equip(&mut g, "Lightning Greaves", w);
    assert!(sim_core::dsl::activate(&mut g, P0, w, 1).unwrap());
    assert!(g.perm(w).tapped);
}

#[test]
fn shang_chi_lets_a_sick_creatures_compiled_ability_tap() {
    let mut g = tbl();
    let (w, _) = walker(&mut g);
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    assert!(sim_core::dsl::activate(&mut g, P0, w, 1).unwrap(), "'as though it had haste' for abilities");
}

// ======================================================== Karn's deck: Twitching Doll
#[test]
fn a_sick_twitching_doll_under_the_crystal_may_be_sacrificed_and_taps_for_mana() {
    let mut g = tbl();
    let d = sick(&mut g, P0, "Twitching Doll");
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(3));
    let opts = |g: &mut Game| {
        let c = g.perm(d).cd.unwrap();
        let f = g.registry.get(c).unwrap().options.unwrap();
        f(g, d, P0, Some(false)).unwrap()
    };
    assert!(opts(&mut g).is_empty(), "control: summoning sick, no {{T}} ability");
    assert_eq!(mana_now(&g), 0);
    perm(&mut g, P0, CRYSTAL);
    let o = opts(&mut g);
    assert_eq!(o.len(), 1, "the sacrifice is listed");
    assert_eq!(mana_now(&g), 1, "and its mana is counted");
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(d).on_bf, "sacrificed");
    assert_eq!(perms_named(&g, P0, "Token").len(), 3, "three Spiders");
}

// ======================================================== Giver of Runes
#[test]
fn the_ais_giver_of_runes_protects_the_turn_it_came_under_haste_only() {
    let protect = |g: &mut Game, m: PermId| {
        g.active = Some(P1);
        let s = card(g, "Swords to Plowshares");
        sim_core::ai::protect_response(g, P0, m, "exile", Some(P1), Some(s)).unwrap()
    };
    let mut g = tbl();
    let m = perm(&mut g, P0, "Grave Titan");
    let giver = sick(&mut g, P0, "Giver of Runes");
    assert!(!protect(&mut g, m), "control: a sick Giver can't tap");
    assert!(!g.perm(giver).tapped);
    let mut g = tbl();
    let m = perm(&mut g, P0, "Grave Titan");
    let giver = sick(&mut g, P0, "Giver of Runes");
    perm(&mut g, P0, CRYSTAL);
    assert!(protect(&mut g, m), "the Crystal's haste: Giver is not refused");
    assert!(g.perm(giver).tapped);
}

#[test]
fn a_person_activates_a_sick_giver_of_runes_under_the_crystal() {
    let try_it = |with_crystal: bool| -> Option<String> {
        let mut g = tbl();
        perm(&mut g, P0, "Grave Titan");
        let giver = sick(&mut g, P0, "Giver of Runes");
        if with_crystal {
            perm(&mut g, P0, CRYSTAL);
        }
        let h = std::sync::Arc::new(ScriptHuman::from_fn(|_| Answer::Pick(0)));
        g.humans.add(P0, h);
        let a = sim_core::human::cloud::perm_abilities(&g, P0, giver);
        assert_eq!(a.len(), 1, "Giver has a person's ability");
        (a[0].act)(&mut g, P0, giver).unwrap()
    };
    let why = try_it(false).expect("control: refused");
    assert!(why.contains("summoning sickness"), "{why}");
    assert_eq!(try_it(true), None, "the Crystal's haste: it works");
}
