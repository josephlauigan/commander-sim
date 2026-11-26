//! Iron Man, Armored Avenger ({3}{U}, in the Karn, Gilded Guardian deck): "Whenever you draw a card, put a +1/+1 counter on
//! target creature." The counter is put as each card is drawn (`zones::draw`, the AI's pick; `human::triggers_t1`, a
//! person's). The target must be legal: our own creature with shroud or protection from Iron Man's colour (blue) is not
//! (hexproof is), an opposing creature must be targetable and not protected, and its ward is paid or it is not picked.
//! (The older tests of the AI's pick and a person's pick are in ai_logic_fixes_l.rs and practice_triggers_t1.rs.)

mod play_common;
use play_common::*;
use sim_core::engine::zones;
use sim_core::state::Step;
use std::sync::Arc;

const IRON_MAN: &str = "Iron Man, Armored Avenger";
const HOOF: &str = "Craterhoof Behemoth"; // worth far more than a token
const WARDED: &str = "Unflinching Hortimancer"; // a 2/1 with ward {1}

fn tbl() -> Game {
    let mut g = table(&["sauron", "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

/// the AI draws a card with Iron Man out, a Craterhoof (the creature it values most) and a token on the table
fn board(g: &mut Game) -> (PermId, PermId) {
    perm(g, P0, IRON_MAN);
    let hoof = perm(g, P0, HOOF);
    let tok = token(g, P0, 1);
    (hoof, tok)
}

/// Iron Man himself (the AI may put the counter on him too)
fn iron(g: &Game) -> PermId {
    perms_named(g, P0, IRON_MAN)[0]
}

#[test]
fn the_ai_puts_the_counter_on_the_creature_it_values_most_when_it_is_legal() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((g.perm(hoof).plus, g.perm(tok).plus), (1, 0));
}

#[test]
fn the_ai_skips_its_own_creature_with_protection_from_blue() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    g.perm_mut(hoof).eot_kw.push("pro_U"); // Giver of Runes: protection from blue, Iron Man is blue
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(hoof).plus, 0, "protection from blue: Iron Man can't target it");
    assert_eq!(g.perm(iron(&g)).plus + g.perm(tok).plus, 1, "the next best creature takes it");
}

#[test]
fn protection_from_another_colour_does_not_stop_it() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    g.perm_mut(hoof).eot_kw.push("pro_R");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((g.perm(hoof).plus, g.perm(tok).plus), (1, 0));
}

#[test]
fn the_ai_skips_its_own_creature_with_shroud_from_lightning_greaves() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    let greaves = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(greaves).attached = Some(hoof);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(hoof).plus, 0, "shroud");
    assert_eq!(g.perm(iron(&g)).plus + g.perm(tok).plus, 1, "the next best creature takes it");
}

#[test]
fn the_ai_may_target_its_own_hexproof_creature() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    g.perm_mut(hoof).eot_kw.push("hexproof");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((g.perm(hoof).plus, g.perm(tok).plus), (1, 0), "hexproof stops only opponents");
}

#[test]
fn the_ai_counter_never_goes_on_an_opposing_creature() {
    let mut g = tbl();
    let big = perm(&mut g, P1, HOOF);
    perm(&mut g, P0, IRON_MAN);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(big).plus, 0);
}

/// a person at P0's seat: "which creature?" is answered with the first choice that contains `target`, a ward with `pay`
fn person(g: &mut Game, target: &'static str, pay: bool) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.starts_with("Iron Man") {
            Answer::Pick(r.choices.iter().position(|c| c.contains(target)).unwrap_or(0))
        } else if r.prompt.contains("ward") {
            Answer::Pick(if pay { 0 } else { 1 }) // "yes", "no"
        } else {
            Answer::Action(json!({"do": "pass"}))
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

fn offered(h: &ScriptHuman) -> Vec<String> {
    h.asked().into_iter().find(|r| r.prompt.starts_with("Iron Man")).map(|r| r.choices).unwrap_or_default()
}

#[test]
fn a_person_is_not_offered_their_creature_with_protection_from_blue_or_shroud() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    g.perm_mut(hoof).eot_kw.push("pro_U");
    let other = perm(&mut g, P0, "Vampire Nighthawk");
    let greaves = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(greaves).attached = Some(other);
    let h = person(&mut g, "1/1", false);
    zones::draw(&mut g, P0, 1, false).unwrap();
    let list = offered(&h);
    assert!(!list.iter().any(|c| c.contains(HOOF) || c.contains("Vampire Nighthawk")), "{list:?}");
    assert!(list.iter().any(|c| c.contains("1/1")), "the token is offered: {list:?}");
    assert_eq!(g.perm(tok).plus, 1);
    assert_eq!((g.perm(hoof).plus, g.perm(other).plus), (0, 0));
}

#[test]
fn a_person_is_offered_their_own_hexproof_creature() {
    let mut g = tbl();
    let (hoof, _) = board(&mut g);
    g.perm_mut(hoof).eot_kw.push("hexproof");
    let h = person(&mut g, HOOF, false);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(offered(&h).iter().any(|c| c.contains(HOOF)), "{:?}", offered(&h));
    assert_eq!(g.perm(hoof).plus, 1);
}

#[test]
fn a_person_is_not_offered_an_opposing_creature_with_protection_from_blue() {
    let mut g = tbl();
    board(&mut g);
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    g.perm_mut(theirs).eot_kw.push("pro_U");
    let h = person(&mut g, "Llanowar", false);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!offered(&h).iter().any(|c| c.contains("Llanowar")), "{:?}", offered(&h));
    assert_eq!(g.perm(theirs).plus, 0);
}

#[test]
fn a_person_is_not_offered_a_warded_opposing_creature_they_cannot_pay_for() {
    let mut g = tbl();
    board(&mut g);
    let w = perm(&mut g, P1, WARDED);
    let h = person(&mut g, "Hortimancer", false);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!offered(&h).iter().any(|c| c.contains("Hortimancer")), "no mana for ward {{1}}: {:?}", offered(&h));
    assert_eq!(g.perm(w).plus, 0);
}

#[test]
fn a_person_pays_a_ward_to_aim_the_counter_at_an_opposing_creature() {
    let mut g = tbl();
    board(&mut g);
    let w = perm(&mut g, P1, WARDED);
    let m = lands(&mut g, P0, "Mountain", 1, false)[0];
    let h = person(&mut g, "Hortimancer", true);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(offered(&h).iter().any(|c| c.contains("Hortimancer")), "{:?}", offered(&h));
    assert!(g.perm(m).tapped, "ward {{1}} paid");
    assert_eq!(g.perm(w).plus, 1, "the counter went on");
}

#[test]
fn a_person_who_does_not_pay_the_ward_loses_the_counter() {
    let mut g = tbl();
    let (hoof, tok) = board(&mut g);
    let w = perm(&mut g, P1, WARDED);
    lands(&mut g, P0, "Mountain", 1, false);
    let h = person(&mut g, "Hortimancer", false);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(offered(&h).iter().any(|c| c.contains("Hortimancer")), "it is offered: {:?}", offered(&h));
    assert_eq!(g.perm(w).plus, 0);
    let iron = g.perm(iron(&g)).plus;
    assert_eq!((iron, g.perm(hoof).plus, g.perm(tok).plus), (0, 0, 0), "countered by ward: no counter anywhere");
}

#[test]
fn a_person_whose_only_source_is_restricted_mana_is_not_offered_the_warded_creature() {
    let mut g = tbl();
    board(&mut g);
    let w = perm(&mut g, P1, WARDED);
    perm(&mut g, P0, "Herd Heirloom"); // {T}: one mana, only to cast a creature spell: it can't pay a ward
    let h = person(&mut g, "Hortimancer", true);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!offered(&h).iter().any(|c| c.contains("Hortimancer")), "{:?}", offered(&h));
    assert_eq!(g.perm(w).plus, 0);
}

/// every creature of the AI's has protection from blue (Akroma's Will): the trigger must still target
fn all_protected(g: &mut Game) -> Vec<PermId> {
    let (hoof, tok) = board(g);
    let iron = iron(g);
    for m in [hoof, tok, iron] {
        g.perm_mut(m).eot_kw.push("pro_U");
    }
    vec![hoof, tok, iron]
}

#[test]
fn the_ai_with_no_legal_creature_of_its_own_targets_an_opposing_one() {
    let mut g = tbl();
    let own = all_protected(&mut g);
    let big = perm(&mut g, P1, HOOF);
    let small = perm(&mut g, P1, "Llanowar Elves");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(own.iter().all(|&m| g.perm(m).plus == 0));
    assert_eq!((g.perm(small).plus, g.perm(big).plus), (1, 0), "the opposing creature worth least");
}

#[test]
fn the_ai_with_no_legal_creature_of_its_own_ignores_an_opposing_creature_with_protection_from_blue() {
    let mut g = tbl();
    all_protected(&mut g);
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    g.perm_mut(theirs).eot_kw.push("pro_U");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(theirs).plus, 0);
}

#[test]
fn the_ai_with_no_legal_creature_of_its_own_and_no_mana_for_the_ward_puts_no_counter() {
    let mut g = tbl();
    let own = all_protected(&mut g);
    let w = perm(&mut g, P1, WARDED);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(w).plus, 0, "ward {{1}} not paid: countered");
    assert!(own.iter().all(|&m| g.perm(m).plus == 0));
}

#[test]
fn the_ai_with_no_legal_creature_of_its_own_pays_the_ward_and_puts_the_counter() {
    let mut g = tbl();
    all_protected(&mut g);
    let w = perm(&mut g, P1, WARDED);
    let m = lands(&mut g, P0, "Mountain", 1, false)[0];
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.perm(m).tapped, "ward {{1}} paid");
    assert_eq!(g.perm(w).plus, 1);
}

#[test]
fn the_ai_prefers_an_opposing_creature_it_can_ward_free_to_one_whose_ward_it_cannot_pay() {
    let mut g = tbl();
    all_protected(&mut g);
    let w = perm(&mut g, P1, WARDED); // worth less, but its ward can't be paid
    let free = perm(&mut g, P1, HOOF);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((g.perm(w).plus, g.perm(free).plus), (0, 1));
}
