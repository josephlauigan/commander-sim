//! The glissa candidate deck's cards (impls/glissa.rs): the bites and the fight, repartee, crime and the other
//! creatures, on hand-built positions. Seat 0 plays the glissa deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::mana::cost_of;
use sim_core::engine::{cast, combat, removal, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["glissa", "seph"])
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

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| &*g.perm(m).name == name)
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
fn burrog_barrage_pumps_only_after_another_spell() {
    let mut g = game();
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 4);
    cast_with(&mut g, "Burrog Barrage", bite(mine, theirs));
    assert!(alive(&g, theirs), "3 damage does not kill a 4/4");
    assert_eq!(sim_core::engine::values::epow(&g, mine), 3);
    let theirs2 = token(&mut g, P1, 4);
    let one = token(&mut g, P1, 1);
    cast_with(&mut g, "Bite Down", bite(mine, one));
    cast_with(&mut g, "Burrog Barrage", bite(mine, theirs2));
    assert!(!alive(&g, theirs2), "+1/+0 after another instant");
    assert_eq!(sim_core::engine::values::epow(&g, mine), 4);
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
fn cost_of_brilliance_draws_two_and_a_counter() {
    let mut g = game();
    let mine = token(&mut g, P0, 2);
    let n = g.player(P0).hand.len();
    cast_with(&mut g, "Cost of Brilliance", Ctx { face: Some(P0), host: Some(mine), ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), n + 2);
    assert_eq!(g.player(P0).life, 38);
    assert_eq!(g.perm(mine).plus, 1);
}

#[test]
fn foolish_fate_infusion_drains_three() {
    let mut g = game();
    let t = token(&mut g, P1, 3);
    cast_with(&mut g, "Foolish Fate", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!alive(&g, t));
    assert_eq!(g.player(P1).life, 40);
    let t2 = token(&mut g, P1, 3);
    sim_core::engine::life::gain(&mut g, P0, 1).unwrap();
    cast_with(&mut g, "Foolish Fate", Ctx { target: Some(t2), ..Ctx::default() });
    assert!(!alive(&g, t2));
    assert_eq!(g.player(P1).life, 37);
}

#[test]
fn witherbloom_charm_modes() {
    let mut g = game();
    // destroy: a nonland permanent with mana value 2 or less
    let tok = token(&mut g, P1, 3);
    cast_with(&mut g, "Witherbloom Charm", Ctx { target: Some(tok), mode: Some("destroy"), ..Ctx::default() });
    assert!(!alive(&g, tok));
    // gain 5
    cast_with(&mut g, "Witherbloom Charm", Ctx { mode: Some("life"), ..Ctx::default() });
    assert_eq!(g.player(P0).life, 45);
    // sacrifice a permanent (a Food), draw two
    g.player_mut(P0).foods = 1;
    let n = g.player(P0).hand.len();
    cast_with(&mut g, "Witherbloom Charm", Ctx { mode: Some("draw"), ..Ctx::default() });
    assert_eq!(g.player(P0).foods, 0);
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn witherbloom_charm_options_pick_a_mode() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Witherbloom Charm"]);
    token(&mut g, P1, 3);
    assert!(has_option(&mut g, "Witherbloom Charm (destroy"), "a token has mana value 0");
    assert!(!has_option(&mut g, "Witherbloom Charm (sacrifice"), "nothing to sacrifice");
}

#[test]
fn culling_ritual_destroys_cheap_permanents_and_makes_mana() {
    let mut g = game();
    let (a, b) = (token(&mut g, P1, 2), token(&mut g, P1, 3));
    let mine = token(&mut g, P0, 1);
    let big = perm(&mut g, P0, "Canyon Crawler");
    g.player_mut(P1).treasures = 1;
    cast_with(&mut g, "Culling Ritual", Ctx::default());
    assert!(!alive(&g, a) && !alive(&g, b) && !alive(&g, mine));
    assert!(alive(&g, big), "mana value 6 stays");
    assert_eq!(g.player(P1).treasures, 0);
    let f = g.player(P0).floating;
    assert_eq!(f.b + f.g, 5, "one mana for each: three tokens, the Treasure and the Crawler's Food");
}

#[test]
fn track_down_draws_a_creature_or_land_on_top() {
    let mut g = game();
    let land = take(&mut g, P0, "Forest");
    g.player_mut(P0).library.push(land);
    let n = g.player(P0).hand.len();
    cast_with(&mut g, "Track Down", Ctx::default());
    assert!(g.player(P0).hand.len() >= n + 1);
}

// ------------------------------------------------------------------ Ferocity of the Hunt
#[test]
fn ferocity_gives_deathtouch_and_returns_the_creature_tapped() {
    let mut g = game();
    let m = perm(&mut g, P0, "Llanowar Elves");
    let base = sim_core::engine::values::epow(&g, m);
    let c = hand(&mut g, P0, &["Ferocity of the Hunt"])[0];
    g.attach_to = Some(m);
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(sim_core::dsl::has_kw(&g, m, "deathtouch"));
    assert_eq!(sim_core::engine::values::epow(&g, m), base + 1);
    zones::die(&mut g, m, "destroy".into()).unwrap();
    assert!(on_bf(&g, P0, "Llanowar Elves"), "its card returned");
    let back = g.player(P0).perms.iter().copied().find(|&x| &*g.perm(x).name == "Llanowar Elves").unwrap();
    assert!(g.perm(back).tapped);
    assert!(in_gy(&g, P0, "Ferocity of the Hunt"));
    assert!(!g.perm(back).data.truthy(DataKey::Ferocity));
}

// ------------------------------------------------------------------ creatures
#[test]
fn armored_scrapgorger_exiles_cards_and_grows() {
    let mut g = game();
    let m = perm(&mut g, P0, "Armored Scrapgorger");
    for _ in 0..3 {
        let c = take(&mut g, P1, "Canyon Crawler");
        g.player_mut(P1).gy.push(c);
    }
    assert_eq!(sim_core::engine::values::epow(&g, m), 0);
    for _ in 0..3 {
        let f = g.registry.get(card(&g, "Armored Scrapgorger")).unwrap().on_tap_perm.unwrap();
        f(&mut g, P0, m, 1).unwrap();
    }
    assert_eq!(g.perm(m).data.int(DataKey::Oil), 3);
    assert_eq!(g.player(P1).gy.len(), 0);
    assert_eq!(g.player(P1).exile.len(), 3);
    assert_eq!(sim_core::engine::values::epow(&g, m), 3);
}

#[test]
fn armored_scrapgorger_triggers_when_tapped_for_mana() {
    let mut g = game();
    let m = perm(&mut g, P0, "Armored Scrapgorger");
    let c = take(&mut g, P1, "Canyon Crawler");
    g.player_mut(P1).gy.push(c);
    hand(&mut g, P0, &["Llanowar Elves"]);
    let elves = card(&g, "Llanowar Elves");
    let ok = brain::do_cast(&mut g, P0, elves, None).unwrap();
    assert!(ok);
    assert!(g.perm(m).tapped);
    assert_eq!(g.perm(m).data.int(DataKey::Oil), 1);
    assert_eq!(g.player(P1).exile.len(), 1);
}

#[test]
fn canyon_crawler_makes_a_food_and_swampcycles() {
    let mut g = game();
    perm(&mut g, P0, "Canyon Crawler");
    assert_eq!(g.player(P0).foods, 1);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Canyon Crawler"]);
    assert!(take_option(&mut g, "Canyon Crawler (swampcycling)"));
    assert!(in_gy(&mut g, P0, "Canyon Crawler"));
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Swamp"));
}

#[test]
fn forum_necroscribe_reanimates_on_repartee_and_wards_with_a_discard() {
    let mut g = game();
    perm(&mut g, P0, "Forum Necroscribe");
    let dead = take(&mut g, P0, "Canyon Crawler");
    g.player_mut(P0).gy.push(dead);
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 2);
    cast_with(&mut g, "Bite Down", bite(mine, theirs));
    assert!(on_bf(&g, P0, "Canyon Crawler"), "repartee returned it");
    // ward: an opponent's removal costs them a card
    let necro = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Forum Necroscribe").unwrap();
    hand(&mut g, P1, &["Swords to Plowshares"]);
    let before = g.player(P1).hand.len();
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), necro, "exile", Some(stp)).unwrap();
    assert_eq!(g.player(P1).hand.len(), before - 1);
    // with no cards in hand the removal is countered
    g.player_mut(P1).hand.clear();
    let again = perm(&mut g, P0, "Forum Necroscribe");
    removal::apply_removal(&mut g, Some(P1), again, "exile", Some(stp)).unwrap();
    assert!(alive(&g, again));
}

#[test]
fn repartee_creatures_trigger_on_spells_that_target_a_creature() {
    let mut g = game();
    let scorn = perm(&mut g, P0, "Lecturing Scornmage");
    perm(&mut g, P0, "Melancholic Poet");
    let mine = token(&mut g, P0, 3);
    let theirs = token(&mut g, P1, 2);
    cast_with(&mut g, "Bite Down", bite(mine, theirs));
    assert_eq!(g.perm(scorn).plus, 1);
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41);
    // a spell that targets no creature does nothing
    cast_with(&mut g, "Track Down", Ctx::default());
    assert_eq!(g.perm(scorn).plus, 1);
}

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
fn hunters_axe_pumps_and_gives_deathtouch_on_attack() {
    let mut g = game();
    let m = token(&mut g, P0, 2);
    let axe = perm(&mut g, P0, "Hunter's Axe");
    g.perm_mut(axe).attached = Some(m);
    assert_eq!(sim_core::engine::values::epow(&g, m), 4);
    let f = g.registry.get(card(&g, "Hunter's Axe")).unwrap().attack.unwrap();
    f(&mut g, axe, P0, &[m], P1).unwrap();
    assert!(sim_core::dsl::has_kw(&g, m, "deathtouch"));
    assert!(!sim_core::dsl::has_kw(&g, m, "trample"));
    g.perm_mut(m).eot_kw.clear();
    g.perm_mut(m).dt = true;
    f(&mut g, axe, P0, &[m], P1).unwrap();
    assert!(sim_core::dsl::has_kw(&g, m, "trample"));
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
fn writhing_necromass_costs_less_for_each_creature_card_in_the_graveyard() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Writhing Necromass"])[0];
    assert_eq!(cost_of(&g, P0, c).0, 6);
    for n in ["Llanowar Elves", "Canyon Crawler"] {
        let x = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(x);
    }
    let x = take(&mut g, P0, "Track Down");
    g.player_mut(P0).gy.push(x);
    assert_eq!(cost_of(&g, P0, c).0, 4);
}

#[test]
fn mabel_removes_counters_from_the_best_opposing_creature() {
    let mut g = game();
    let theirs = token(&mut g, P1, 2);
    g.perm_mut(theirs).plus = 4;
    perm(&mut g, P0, "Mabel, Bitter Recluse");
    assert_eq!(g.perm(theirs).plus, 1);
    // -1/-1 counters on my creature go too
    let mine = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(mine).plus = -2;
    let before = g.perm(theirs).plus;
    g.perm_mut(theirs).plus = 0;
    zones::die(&mut g, mine, "sac".into()).unwrap();
    assert_eq!(before, 1);
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

#[test]
fn vault_plunderer_draws_and_loses_life() {
    let mut g = game();
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Vault Plunderer");
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).life, 39);
}

#[test]
fn seymour_flux_and_bugenhagen_upkeep_triggers() {
    let mut g = game();
    let s = perm(&mut g, P0, "Seymour Flux");
    let bug = perm(&mut g, P0, "Bugenhagen, Wise Elder");
    let n = g.player(P0).hand.len();
    let f = g.registry.get(card(&g, "Seymour Flux")).unwrap().upkeep.unwrap();
    f(&mut g, s, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).life, 39);
    assert_eq!(g.perm(s).plus, 1);
    // Bugenhagen: a creature with power 7 or more
    let fb = g.registry.get(card(&g, "Bugenhagen, Wise Elder")).unwrap().upkeep.unwrap();
    let n = g.player(P0).hand.len();
    fb(&mut g, bug, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "Seymour is 6/6 now");
    g.perm_mut(s).plus = 2;
    fb(&mut g, bug, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn noxious_newt_is_a_deathtouch_mana_dork() {
    let mut g = game();
    let newt = perm(&mut g, P0, "Noxious Newt");
    assert!(sim_core::dsl::has_kw(&g, newt, "deathtouch"));
    hand(&mut g, P0, &["Llanowar Elves"]);
    let elves = card(&g, "Llanowar Elves");
    assert!(brain::do_cast(&mut g, P0, elves, None).unwrap(), "the Newt pays G");
    assert!(g.perm(newt).tapped);
}
