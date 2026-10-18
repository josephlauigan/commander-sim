//! Tier 6 "mana" batch (impls/t6_mana.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["rograkh-silas-turbo-breach", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.player(p).perms.contains(&m)
}

// ======================================================== Chromatic Star
#[test]
fn chromatic_star_is_not_a_free_rock_and_cracks_for_a_card() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let s = perm(&mut g, P0, "Chromatic Star");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "Star adds no mana by itself");
    assert!(options(&mut g, s, P0, Some(false)).is_empty(), "not before combat");
    let o = options(&mut g, s, P0, Some(true));
    assert_eq!(o.len(), 1);
    let before = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert!(!on(&g, P0, s), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Chromatic Star")));
    assert_eq!(g.player(P0).hand.len(), before + 1, "drew a card");
    assert_eq!(g.player(P0).floating.any, 1, "one mana of any colour");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2 - 1 + 1, "paid {{1}}, got one mana back");
}

#[test]
fn chromatic_star_destroyed_still_draws() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Chromatic Star");
    let before = g.player(P0).hand.len();
    sim_core::engine::zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1);
    let _ = P1;
}

// ======================================================== Desperate Ritual
#[test]
fn desperate_ritual_adds_rrr_for_two_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let c = hand(&mut g, P0, &["Desperate Ritual"])[0];
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "R", false).unwrap(), "{{1}}{{R}}");
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap());
    assert_eq!(g.player(P0).floating.r, 3, "RRR");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 3, "both Mountains were tapped for the cost");
    assert!(g.player(P0).gy.contains(&c), "an instant goes to the graveyard");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "RRR", false));
}

// ======================================================== Rite of Flame
#[test]
fn rite_of_flame_adds_rr_and_one_more_per_copy_in_a_graveyard() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let c = hand(&mut g, P0, &["Rite of Flame"])[0];
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "R", false).unwrap());
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap());
    assert_eq!(g.player(P0).floating.r, 2, "RR with no other copy");
    // two more copies (other players' graveyards count too) make RRRR
    let again = card(&g, "Rite of Flame");
    g.player_mut(P1).gy.push(again);
    g.player_mut(P0).gy.push(again);
    g.player_mut(P0).floating.r = 0;
    let c2 = card(&g, "Rite of Flame");
    let mut ctx = sim_core::state::Ctx::default();
    ctx.owner = None;
    let f = g.registry.get(c2).unwrap().resolve.unwrap();
    let dest = f(&mut g, P0, c2, &ctx).unwrap();
    assert_eq!(&*dest, "gy");
    assert_eq!(g.player(P0).floating.r, 2 + 3, "the copy cast first plus two more in graveyards");
}

// ======================================================== Manifold Key
#[test]
fn manifold_key_untaps_a_sol_ring_for_one() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let ring = perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Rite of Flame"]);
    g.perm_mut(ring).tapped = true;
    let o = options(&mut g, key, P0, Some(false));
    assert_eq!(o.len(), 1, "untap offer only; no creature to make unblockable");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(ring).tapped, "Sol Ring untapped");
    assert!(g.perm(key).tapped, "{{T}} is part of the cost");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "Mountain paid {{1}}; Sol Ring makes {{C}}{{C}}");
    assert!(options(&mut g, key, P0, Some(false)).is_empty(), "the Key is tapped");
}

#[test]
fn manifold_key_does_not_untap_a_one_mana_rock() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let rock = perm(&mut g, P0, "Arcane Signet");
    hand(&mut g, P0, &["Rite of Flame"]);
    g.perm_mut(rock).tapped = true;
    assert!(options(&mut g, key, P0, Some(false)).is_empty(), "{{1}} for {{1}} is no gain");
}

#[test]
fn manifold_key_makes_the_best_attacker_unblockable_for_three() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 3, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let big = token(&mut g, P0, 5);
    token(&mut g, P1, 2); // a blocker
    let o = options(&mut g, key, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    let st = g.turn_stamp();
    assert_eq!(g.perm(big).data.get(sim_core::state::DataKey::Unbl), Some(&sim_core::state::Val::Stamp(st)));
    assert!(g.perm(key).tapped);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "paid {{3}}");
}

// ======================================================== Honor-Worn Shaku
#[test]
fn shaku_taps_a_legendary_permanent_to_untap_for_more_mana() {
    use sim_core::engine::mana::{can_pay, pay, total_mana};
    let mut g = tbl();
    let shaku = perm(&mut g, P0, "Honor-Worn Shaku");
    assert_eq!(total_mana(&g, P0, false), 1, "just {{C}} with no legendary permanent");
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    assert_eq!(total_mana(&g, P0, false), 2, "tap the legendary Apple to untap Shaku: {{C}}{{C}}");
    assert!(can_pay(&g, P0, 2, "", false));
    assert!(pay(&mut g, P0, 2, "", false).unwrap());
    assert!(g.perm(shaku).tapped && g.perm(apple).tapped, "both tapped");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn shaku_uses_one_mana_when_only_one_is_needed() {
    use sim_core::engine::mana::{pay, total_mana};
    let mut g = tbl();
    let shaku = perm(&mut g, P0, "Honor-Worn Shaku");
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    assert!(g.perm(shaku).tapped);
    assert!(!g.perm(apple).tapped, "the Apple is tapped only for the extra mana");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn shaku_does_not_tap_a_creature_that_can_attack() {
    use sim_core::engine::mana::total_mana;
    let mut g = tbl();
    perm(&mut g, P0, "Honor-Worn Shaku");
    perm(&mut g, P0, "Godo, Bandit Warlord"); // legendary, not summoning sick: a blocker and an attacker
    assert_eq!(total_mana(&g, P0, false), 1);
}

// ======================================================== Metalworker
#[test]
fn metalworker_makes_cc_for_each_artifact_card_in_hand() {
    use sim_core::engine::mana::{pay, total_mana};
    let mut g = tbl();
    let mw = perm(&mut g, P0, "Metalworker");
    assert_eq!(total_mana(&g, P0, false), 0, "no artifact card revealed, no mana");
    hand(&mut g, P0, &["Sol Ring", "Manifold Key", "Rite of Flame"]);
    assert_eq!(total_mana(&g, P0, false), 4, "two artifact cards: {{C}}{{C}} each");
    assert!(pay(&mut g, P0, 4, "", false).unwrap());
    assert!(g.perm(mw).tapped);
}

#[test]
fn metalworker_is_summoning_sick_like_any_creature() {
    use sim_core::engine::mana::total_mana;
    let mut g = tbl();
    let c = take(&mut g, P0, "Metalworker");
    let m = sim_core::engine::zones::enter(&mut g, P0, c, Default::default()).unwrap();
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(g.perm(m).sick);
    assert_eq!(total_mana(&g, P0, false), 0);
}

// ======================================================== Treasonous Ogre
#[test]
fn treasonous_ogre_pays_three_life_per_red_mana_after_lands() {
    use sim_core::engine::mana::{can_pay, pay, total_mana};
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let ogre = perm(&mut g, P0, "Treasonous Ogre");
    assert_eq!(g.player(P0).life, 40);
    assert_eq!(total_mana(&g, P0, false), 2 + 10, "two lands and ten payments of 3 life (40 down to 10)");
    // a small cost ({1}{R}) is paid by the two lands: no life lost
    assert!(pay(&mut g, P0, 1, "R", false).unwrap());
    assert_eq!(g.player(P0).life, 40);
    // both Mountains are tapped now ({1}{R}): RRR is all the Ogre
    assert!(can_pay(&g, P0, 0, "RRR", false));
    assert!(pay(&mut g, P0, 0, "RRR", false).unwrap());
    assert_eq!(g.player(P0).life, 40 - 3 * 3, "three mana from the Ogre");
    assert!(!g.perm(ogre).tapped, "paying life does not tap the Ogre");
}

#[test]
fn treasonous_ogre_stops_paying_near_the_life_floor_and_gives_no_colour_but_red() {
    use sim_core::engine::mana::{can_pay, total_mana};
    let mut g = tbl();
    perm(&mut g, P0, "Treasonous Ogre");
    g.player_mut(P0).life = 12;
    assert_eq!(total_mana(&g, P0, false), 0, "12 life is below the floor of 10 plus one payment");
    g.player_mut(P0).life = 19;
    assert_eq!(total_mana(&g, P0, false), 3);
    assert!(can_pay(&g, P0, 0, "RRR", false));
    assert!(!can_pay(&g, P0, 0, "U", false), "only red");
}

#[test]
fn treasonous_ogre_has_no_proactive_floating_mana_ability() {
    use sim_core::dsl;
    let mut g = tbl();
    perm(&mut g, P0, "Treasonous Ogre");
    let o = dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(o.iter().all(|x| !x.label.contains("Treasonous")), "no pay-3-life option for the AI");
}
