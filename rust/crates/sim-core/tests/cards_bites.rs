//! The bite and fight spells and the green creatures with them (impls/bites.rs), on hand-built positions. Seat 0 plays the
//! titania deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::mana::cost_of;
use sim_core::engine::{cast, combat, removal};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["omnath", "seph"])
}

fn options(g: &mut Game) -> Vec<Opt> {
    brain::main_options(g, P0, false).unwrap()
}

fn has_option(g: &mut Game, prefix: &str) -> bool {
    options(g).iter().any(|o| o.label.starts_with(prefix))
}

fn take_option(g: &mut Game, prefix: &str) -> bool {
    let opts = options(g);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(prefix))
        .unwrap_or_else(|| panic!("no option {prefix:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

/// cast a spell from hand with these choices (nothing is paid)
fn cast_with(g: &mut Game, name: &str, ctx: Ctx) -> CardId {
    let c = hand(g, P0, &[name])[0];
    cast::cast_card(g, P0, c, "hand", ctx).unwrap();
    c
}

fn bite(host: PermId, target: PermId) -> Ctx {
    Ctx { host: Some(host), target: Some(target), ..Ctx::default() }
}

fn alive(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ bite
#[test]
fn bite_down_with_deathtouch_kills_anything() {
    let mut g = game();
    let newt = perm(&mut g, P0, "Noxious Newt");
    let big = token(&mut g, P1, 9);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Bite Down"]);
    assert!(take_option(&mut g, "Bite Down"));
    assert!(!alive(&g, big));
    assert!(alive(&g, newt));
    assert!(in_gy(&g, P0, "Bite Down"));
}

#[test]
fn bite_down_without_deathtouch_needs_the_power() {
    let mut g = game();
    perm(&mut g, P0, "Llanowar Elves");
    token(&mut g, P1, 5);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Bite Down"]);
    assert!(!has_option(&mut g, "Bite Down"));
    let fat = token(&mut g, P0, 6);
    assert!(has_option(&mut g, "Bite Down"));
    assert!(take_option(&mut g, "Bite Down"));
    assert!(alive(&g, fat));
    assert_eq!(g.player(P1).perms.iter().filter(|&&m| g.is_creature(m)).count(), 0);
}

#[test]
fn bite_uses_the_deathtouch_creature_and_hits_the_biggest_threat() {
    let mut g = game();
    let fat = token(&mut g, P0, 6);
    let newt = perm(&mut g, P0, "Noxious Newt");
    let (small, big) = (token(&mut g, P1, 2), token(&mut g, P1, 8));
    lands(&mut g, P0, "Forest", 2, false);
    let c = hand(&mut g, P0, &["Bite Down"])[0];
    assert!(take_option(&mut g, "Bite Down"));
    assert!(!alive(&g, big));
    assert!(alive(&g, small) && alive(&g, fat) && alive(&g, newt));
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn hard_hitting_question_is_a_sorcery_for_one_mana() {
    let mut g = game();
    perm(&mut g, P0, "Mabel, Bitter Recluse");
    let big = token(&mut g, P1, 7);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Hard-Hitting Question"]);
    assert!(take_option(&mut g, "Hard-Hitting Question"));
    assert!(!alive(&g, big));
}

#[test]
fn bite_hits_a_planeswalker_for_its_loyalty() {
    let mut g = game();
    let fat = token(&mut g, P0, 5);
    let w = perm(&mut g, P1, "Chandra, Torch of Defiance");
    let l = g.perm(w).loyalty.unwrap();
    assert!(l > 0);
    g.perm_mut(w).loyalty = Some(5);
    cast_with(&mut g, "Bite Down", bite(fat, w));
    assert!(!alive(&g, w));
}

#[test]
fn compel_brutality_bites_a_creature() {
    let mut g = game();
    let newt = perm(&mut g, P0, "Noxious Newt");
    let big = token(&mut g, P1, 9);
    cast_with(&mut g, "Compel Brutality", bite(newt, big));
    assert!(!alive(&g, big));
}

#[test]
fn compel_brutality_second_mode_uses_loyalty() {
    let mut g = game();
    let w = perm(&mut g, P0, "Chandra, Torch of Defiance");
    g.perm_mut(w).loyalty = Some(4);
    let victim = token(&mut g, P1, 4);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Compel Brutality"]);
    assert!(take_option(&mut g, "Compel Brutality"));
    assert!(!alive(&g, victim));
}

#[test]
fn diplomatic_relations_pumps_first() {
    let mut g = game();
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 4);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Diplomatic Relations"]);
    assert!(take_option(&mut g, "Diplomatic Relations"));
    assert!(!alive(&g, theirs));
    assert_eq!(sim_core::engine::values::epow(&g, mine), 4);
    assert!(sim_core::dsl::has_kw(&g, mine, "vigilance"));
}

#[test]
fn knockout_maneuver_puts_the_counter_on_first() {
    let mut g = game();
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 4);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Knockout Maneuver"]);
    assert!(take_option(&mut g, "Knockout Maneuver"));
    assert!(!alive(&g, theirs));
    assert_eq!(g.perm(mine).plus, 1);
}

#[test]
fn chelonian_tackle_fights_and_the_victim_hits_back() {
    let mut g = game();
    let mine = token(&mut g, P0, 5);
    let theirs = token(&mut g, P1, 5);
    cast_with(&mut g, "Chelonian Tackle", bite(mine, theirs));
    assert!(!alive(&g, theirs));
    assert!(alive(&g, mine), "+0/+10 keeps it alive");
    assert_eq!(sim_core::engine::values::etgh(&g, mine), 15);
}

#[test]
fn chelonian_tackle_leaves_deathtouch_blockers_alone() {
    let mut g = game();
    token(&mut g, P0, 6);
    let theirs = token(&mut g, P1, 3);
    g.perm_mut(theirs).dt = true;
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Chelonian Tackle"]);
    assert!(!has_option(&mut g, "Chelonian Tackle"), "a deathtouch victim would kill the fighter");
    let mine = g.player(P0).perms[0];
    cast_with(&mut g, "Chelonian Tackle", bite(mine, theirs));
    assert!(!alive(&g, mine), "the fight is lethal for the fighter anyway");
}

// ------------------------------------------------------------------ the spells

#[test]
fn track_down_draws_a_creature_or_land_on_top() {
    let mut g = game();
    let land = take(&mut g, P0, "Forest");
    g.player_mut(P0).library.push(land);
    let n = g.player(P0).hand.len();
    cast_with(&mut g, "Track Down", Ctx::default());
    assert!(g.player(P0).hand.len() >= n + 1);
}

// ------------------------------------------------------------------ creatures

#[test]
fn hardbristle_bandit_untaps_on_a_crime_once_a_turn() {
    let mut g = game();
    let b = perm(&mut g, P0, "Hardbristle Bandit");
    g.perm_mut(b).tapped = true;
    let theirs = token(&mut g, P1, 2);
    cast_with(&mut g, "Terror", Ctx { target: Some(theirs), ..Ctx::default() });
    assert!(!g.perm(b).tapped, "targeting an opponent's creature is a crime");
    g.perm_mut(b).tapped = true;
    let t2 = token(&mut g, P1, 2);
    cast_with(&mut g, "Last Gasp", Ctx { target: Some(t2), ..Ctx::default() });
    assert!(g.perm(b).tapped, "only once each turn");
}

#[test]
fn hardbristle_bandit_ignores_spells_aimed_at_your_own_creatures() {
    let mut g = game();
    let b = perm(&mut g, P0, "Hardbristle Bandit");
    g.perm_mut(b).tapped = true;
    let mine = token(&mut g, P0, 2);
    cast_with(&mut g, "Cost of Brilliance", Ctx { face: Some(P0), host: Some(mine), ..Ctx::default() });
    assert!(g.perm(b).tapped);
}

#[test]
fn lashwhip_predator_costs_two_less_against_three_creatures() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Lashwhip Predator"])[0];
    assert_eq!(cost_of(&g, P0, c).0, 4);
    for _ in 0..2 {
        token(&mut g, P1, 1);
    }
    assert_eq!(cost_of(&g, P0, c).0, 4);
    token(&mut g, P1, 1);
    assert_eq!(cost_of(&g, P0, c).0, 2);
}

#[test]
fn paladin_of_predation_poisons_and_dodges_small_blockers() {
    let mut g = game();
    let pal = perm(&mut g, P0, "Paladin of Predation");
    let small = token(&mut g, P1, 2);
    let big = token(&mut g, P1, 3);
    assert!(!combat::can_block(&g, small, pal), "power 2 or less can't block it");
    assert!(combat::can_block(&g, big, pal));
    let f = g.registry.get(card(&g, "Paladin of Predation")).unwrap().combat_damage.unwrap();
    f(&mut g, pal, P0, pal, P1, 6).unwrap();
    assert_eq!(g.player(P1).poison, 6);
    f(&mut g, pal, P0, pal, P1, 6).unwrap();
    assert_eq!(g.player(P1).poison, 12);
}

#[test]
fn tenured_concocter_draws_when_targeted_and_grows_with_life() {
    let mut g = game();
    let m = perm(&mut g, P0, "Tenured Concocter");
    assert_eq!(sim_core::engine::values::epow(&g, m), 4);
    sim_core::engine::life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, m), 6, "infusion");
    let n = g.player(P0).hand.len();
    removal::apply_removal(&mut g, Some(P1), m, "destroy", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(!alive(&g, m));
    // targeted by its own controller: nothing
    let m2 = perm(&mut g, P0, "Tenured Concocter");
    let n = g.player(P0).hand.len();
    removal::apply_removal(&mut g, Some(P0), m2, "destroy", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
}

// ------------------------------------------------------------------ the AI plan

#[test]
fn a_bite_is_not_wasted_on_a_trivial_creature() {
    let mut g = game();
    perm(&mut g, P0, "Noxious Newt");
    token(&mut g, P1, 1);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Bite Down"]);
    assert!(!has_option(&mut g, "Bite Down"), "a 1/1 token is worth less than a spell");
    token(&mut g, P1, 4);
    assert!(has_option(&mut g, "Bite Down"));
}

#[test]
fn bugenhagen_draws_at_upkeep_only_with_a_creature_of_power_seven() {
    let mut g = game();
    let bug = perm(&mut g, P0, "Bugenhagen, Wise Elder");
    let fb = g.registry.get(card(&g, "Bugenhagen, Wise Elder")).unwrap().upkeep.unwrap();
    let n = g.player(P0).hand.len();
    token(&mut g, P0, 6);
    fb(&mut g, bug, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "no creature with power 7");
    token(&mut g, P0, 7);
    fb(&mut g, bug, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}
