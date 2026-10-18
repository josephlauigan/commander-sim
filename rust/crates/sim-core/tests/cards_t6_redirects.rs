//! Tier 6 batch `redirects` (impls/t6_redirects.rs) on hand-built positions. Seat 0 plays a stand-in deck (the cards
//! are keyed by name), seat 1 the seph deck.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t6_redirects as t6;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P1);
    g
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn untapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

// ======================================================== Redirect Lightning
#[test]
fn redirect_lightning_sends_removal_back_at_the_caster_paying_mana() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    let life = g.player(P0).life;
    let done = t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap();
    assert!(done);
    assert!(g.perm(mine).on_bf, "our creature survives");
    assert!(!g.perm(theirs).on_bf, "the caster's creature takes the removal");
    assert!(in_gy(&g, P0, "Redirect Lightning"));
    assert_eq!(untapped(&g, P0), 0, "{{R}} and {{2}} were paid");
    assert_eq!(g.player(P0).life, life);
}

#[test]
fn redirect_lightning_pays_five_life_when_short_of_mana() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    let life = g.player(P0).life;
    assert!(t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(g.perm(mine).on_bf && !g.perm(theirs).on_bf);
    assert_eq!(g.player(P0).life, life - 5);
}

#[test]
fn redirect_lightning_needs_a_target_and_a_cost_and_ignores_own_spells() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    // the caster controls nothing to aim it at
    assert!(!t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(in_hand(&g, P0, "Redirect Lightning"));
    // low on life and mana: no way to pay
    let _t = perm(&mut g, P1, "Grave Titan");
    g.player_mut(P0).life = 8;
    assert!(!t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(on_bf(&g, P0, "Grave Titan").is_some());
}

// ======================================================== Ricochet Trap
#[test]
fn ricochet_trap_costs_one_red_after_an_opposing_blue_spell() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Ricochet Trap"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    // no blue spell yet: {3}{R} is out of reach
    assert!(!t6::blue_spell_this_turn(&g, P0));
    assert!(!t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    // the opponent cast Counterspell this turn
    let blue = card(&g, "Counterspell");
    let st = g.turn_stamp();
    g.player_mut(P1).turn_casts = Some((st, vec![blue]));
    assert!(t6::blue_spell_this_turn(&g, P0));
    assert!(t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(g.perm(mine).on_bf && !g.perm(theirs).on_bf);
    assert!(in_gy(&g, P0, "Ricochet Trap"));
    assert_eq!(untapped(&g, P0), 0);
}

#[test]
fn ricochet_trap_hard_cast_costs_four_without_a_blue_spell() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 4, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Ricochet Trap"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    assert!(t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(!g.perm(theirs).on_bf);
    assert_eq!(untapped(&g, P0), 0);
}

// ======================================================== Untimely Malfunction
fn cast_free(g: &mut Game, p: PlayerId, name: &str, ctx: sim_core::state::Ctx) {
    let c = hand(g, p, &[name])[0];
    sim_core::engine::cast::cast_card(g, p, c, "hand", ctx).unwrap();
}

#[test]
fn untimely_malfunction_destroys_an_artifact() {
    let mut g = game();
    g.active = Some(P0);
    let ring = perm(&mut g, P1, "Sol Ring");
    cast_free(&mut g, P0, "Untimely Malfunction", sim_core::state::Ctx { target: Some(ring), ..Default::default() });
    assert!(!g.perm(ring).on_bf);
    assert!(in_gy(&g, P0, "Untimely Malfunction"));
}

#[test]
fn untimely_malfunction_without_an_artifact_stops_the_two_best_blockers() {
    let mut g = game();
    g.active = Some(P0);
    let big = perm(&mut g, P1, "Grave Titan");
    let mid = token(&mut g, P1, 3);
    let small = token(&mut g, P1, 1);
    cast_free(&mut g, P0, "Untimely Malfunction", sim_core::state::Ctx::default());
    assert!(g.perm(big).eot_kw.contains(&"cant_block"));
    assert!(g.perm(mid).eot_kw.contains(&"cant_block"));
    assert!(!g.perm(small).eot_kw.contains(&"cant_block"));
    assert!(in_gy(&g, P0, "Untimely Malfunction"));
}

#[test]
fn untimely_malfunction_redirects_removal_like_the_other_redirects() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 2, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Untimely Malfunction"]);
    let swords = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    assert!(t6::redirect_removal(&mut g, P0, mine, intern("destroy"), Some(P1), Some(swords)).unwrap());
    assert!(g.perm(mine).on_bf && !g.perm(theirs).on_bf);
    assert!(in_gy(&g, P0, "Untimely Malfunction"));
}

// ======================================================== Pyrokinesis
fn toughness(g: &Game, m: PermId) -> i32 {
    sim_core::engine::values::etgh(g, m)
}

#[test]
fn pyrokinesis_divides_four_damage_among_creatures() {
    let mut g = game();
    g.active = Some(P0);
    let a = token(&mut g, P1, 2);
    let b = token(&mut g, P1, 2);
    let big = perm(&mut g, P1, "Grave Titan");
    assert!(toughness(&g, a) + toughness(&g, b) <= 4);
    cast_free(&mut g, P0, "Pyrokinesis", sim_core::state::Ctx::default());
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "both small creatures die");
    assert!(g.perm(big).on_bf, "4 damage cannot kill a 6/6");
    assert!(in_gy(&g, P0, "Pyrokinesis"));
}

#[test]
fn pyrokinesis_never_hits_its_own_creatures() {
    let mut g = game();
    g.active = Some(P0);
    let mine = token(&mut g, P0, 2);
    let theirs = token(&mut g, P1, 2);
    cast_free(&mut g, P0, "Pyrokinesis", sim_core::state::Ctx::default());
    assert!(g.perm(mine).on_bf && !g.perm(theirs).on_bf);
}

#[test]
fn pyrokinesis_alternative_cost_exiles_a_red_card() {
    let mut g = game();
    hand(&mut g, P0, &["Pyrokinesis", "Lightning Bolt", "Island"]);
    let c = card(&g, "Pyrokinesis");
    assert_eq!(t6::red_pitch(&g, P0, c).len(), 1, "Pyrokinesis itself is not a pitch");
    assert!(t6::pay_pyro_pitch(&mut g, P0, c).unwrap());
    assert!(g.player(P0).exile.iter().any(|&x| &*g.db.get(x).name == "Lightning Bolt"));
    assert!(in_hand(&g, P0, "Pyrokinesis") && in_hand(&g, P0, "Island"));
    assert!(!t6::pay_pyro_pitch(&mut g, P0, c).unwrap(), "no red card left");
}
