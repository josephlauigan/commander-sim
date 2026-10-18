//! Engine feature `stack` (non-counter responses): redirecting removal with Redirect Lightning / Deflecting Swat
//! (`ai::decks::redirect_response`), Misdirection on the stack, and stealing a spell (Commandeer, Sudden Substitution:
//! `cast_card` resolves under the stack item's controller). Seat 0 holds the responses; seat 1 casts.

use sim_core::engine::cast;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game(a: &str) -> Game {
    let mut g = table(&[a, "seph"]);
    g.active = Some(P1);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// P1 casts Swords to Plowshares at `t`
fn swords_at(g: &mut Game, t: PermId) {
    let sw = hand(g, P1, &["Swords to Plowshares"])[0];
    lands(g, P1, "Plains", 1, false);
    cast::cast_card(g, P1, sw, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
}

// ======================================================== redirect (the removal-response hook)
#[test]
fn any_deck_with_redirect_lightning_turns_removal_on_the_caster() {
    // Godo's deck is not one of the decks that called Deflecting Swat by name
    let mut g = game("godo");
    lands(&mut g, P0, "Mountain", 3, false);
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(godo).is_cmd = true; // the commander
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Redirect Lightning"]);
    swords_at(&mut g, godo);
    assert!(g.perm(godo).on_bf, "Godo survives");
    assert!(!g.perm(theirs).on_bf, "the opponent's creature takes the Swords");
    assert!(in_gy(&g, P0, "Redirect Lightning"));
}

#[test]
fn any_deck_with_deflecting_swat_redirects_too() {
    let mut g = game("godo");
    lands(&mut g, P0, "Mountain", 3, false);
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(godo).is_cmd = true; // the commander
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Deflecting Swat"]);
    swords_at(&mut g, godo);
    assert!(g.perm(godo).on_bf && !g.perm(theirs).on_bf);
    assert!(in_gy(&g, P0, "Deflecting Swat"));
}

#[test]
fn without_a_redirect_card_the_removal_lands() {
    let mut g = game("godo");
    lands(&mut g, P0, "Mountain", 3, false);
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    g.perm_mut(godo).is_cmd = true; // the commander
    let theirs = perm(&mut g, P1, "Grave Titan");
    swords_at(&mut g, godo);
    assert!(!g.perm(godo).on_bf || g.perm(theirs).on_bf);
}

// ======================================================== Misdirection (a response to a spell on the stack)
#[test]
fn misdirection_aims_the_spell_at_the_casters_own_permanent() {
    let mut g = game("sheoldred");
    lands(&mut g, P0, "Island", 5, false);
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Misdirection"]);
    swords_at(&mut g, mine);
    assert!(g.perm(mine).on_bf, "ours survives");
    assert!(!g.perm(theirs).on_bf, "the spell resolved on the caster's creature");
    assert!(in_gy(&g, P0, "Misdirection"));
}

// ======================================================== steal a spell
#[test]
fn commandeer_makes_the_artifact_ours() {
    let mut g = game("sheoldred");
    lands(&mut g, P0, "Island", 7, false);
    hand(&mut g, P0, &["Commandeer"]);
    let sw = hand(&mut g, P1, &["Sword of Fire and Ice"])[0];
    lands(&mut g, P1, "Plains", 3, false);
    cast::cast_card(&mut g, P1, sw, "hand", Ctx::default()).unwrap();
    assert!(on_bf(&g, P0, "Sword of Fire and Ice").is_some(), "it entered under our control");
    assert!(on_bf(&g, P1, "Sword of Fire and Ice").is_none());
    assert!(in_gy(&g, P0, "Commandeer"));
}

#[test]
fn sudden_substitution_trades_a_creature_for_the_spell() {
    let mut g = game("sheoldred");
    lands(&mut g, P0, "Island", 4, false);
    let tok = token(&mut g, P0, 1);
    hand(&mut g, P0, &["Sudden Substitution"]);
    let sw = hand(&mut g, P1, &["Batterskull"])[0];
    lands(&mut g, P1, "Plains", 3, false);
    cast::cast_card(&mut g, P1, sw, "hand", Ctx::default()).unwrap();
    assert!(on_bf(&g, P0, "Batterskull").is_some(), "the Sword is ours");
    assert_eq!(g.perm(tok).owner, P1, "the exchanged creature changed hands");
    assert!(g.player(P1).perms.contains(&tok), "the creature went to the caster");
}
