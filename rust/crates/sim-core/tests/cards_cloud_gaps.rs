//! Judgment of Alexander and Forge Anew (impls/cloud.rs, "Judgment of Alexander" and "Forge Anew"), on hand-built
//! positions. Seat 0 plays the cloud deck, seat 1 the seph deck.

use sim_core::dsl;
use sim_core::engine::life::lose_life;
use sim_core::engine::removal::apply_removal;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::{Tokens, make_tokens};
use sim_core::engine::{cast, combat, mana};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::cloud;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const CLOUD: &str = "Cloud, Midgar Mercenary";
const JUDGMENT: &str = "Judgment of Alexander";
const FORGE: &str = "Forge Anew";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P1); // the opponent's turn
    g
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

/// a token with this power and toughness, not summoning sick
fn body(g: &mut Game, p: PlayerId, pow: i32, tgh: i32) -> PermId {
    make_tokens(g, p, Tokens { sick: false, tgh: Some(tgh), ..Tokens::new(1, pow) }).unwrap()[0]
}

/// P0 holds Judgment with {2}{W} open; the shield is cast (not by the AI)
fn shielded() -> Game {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let c = hand(&mut g, P0, &[JUDGMENT])[0];
    assert!(cloud::cast_judgment(&mut g, P0, c).unwrap());
    assert!(in_gy(&g, P0, JUDGMENT));
    g
}

/// Cloud out and tapped (so he doesn't block), a commander creature
fn tapped_cloud(g: &mut Game) -> PermId {
    let cl = perm(g, P0, CLOUD);
    g.perm_mut(cl).tapped = true;
    g.perm_mut(cl).is_cmd = true; // (entered by the test, not cast from the command zone)
    cl
}

// ------------------------------------------------------------------ Judgment of Alexander: the shield
#[test]
fn judgment_prevents_combat_and_noncombat_damage_from_opponents_sources() {
    let mut g = shielded();
    let life = g.player(P0).life;
    lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life, "burn from an opponent is prevented");
    lose_life(&mut g, P0, 4, Some(P1), "triggers", None).unwrap();
    assert_eq!(g.player(P0).life, life, "so is a trigger's damage");
    let a = body(&mut g, P1, 7, 7);
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, life, "combat damage is prevented");
    // life loss is not damage
    lose_life(&mut g, P0, 3, Some(P1), "drain", None).unwrap();
    assert_eq!(g.player(P0).life, life - 3, "life loss still happens");
}

#[test]
fn judgment_shields_only_its_caster_and_not_their_permanents() {
    let mut g = shielded();
    let life1 = g.player(P1).life;
    lose_life(&mut g, P1, 5, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P1).life, life1 - 5, "the opponent is not shielded");
    let m = body(&mut g, P0, 2, 2);
    apply_removal(&mut g, Some(P1), m, "dmg3", None).unwrap();
    assert!(!g.perm(m).on_bf, "damage to the caster's creature is not prevented");
}

#[test]
fn judgment_does_not_stop_the_casters_own_damage_and_ends_with_the_turn() {
    let mut g = shielded();
    let life = g.player(P0).life;
    lose_life(&mut g, P0, 2, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life - 2, "damage from the caster's own source is dealt");
    assert!(cloud::judgment_up(&g, P0));
    g.round += 1; // the next turn
    assert!(!cloud::judgment_up(&g, P0), "the shield is gone");
    let life = g.player(P0).life;
    lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life - 5);
}

// ------------------------------------------------------------------ Judgment of Alexander: the strike back
#[test]
fn judgment_strike_back_deals_cloud_power_to_each_prevented_attacker() {
    let mut g = shielded();
    let cl = tapped_cloud(&mut g);
    let pow = epow(&g, cl);
    assert!(pow >= 2);
    let small = body(&mut g, P1, 4, pow); // toughness = Cloud's power: dies
    let sturdy = body(&mut g, P1, 4, pow + 1); // survives, with damage
    let life = g.player(P0).life;
    combat::resolve_combat(&mut g, P1, &[small, sturdy], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, life, "no damage to the player");
    assert!(!g.perm(small).on_bf, "{} damage kills the {}-toughness attacker", pow, etgh(&g, small));
    assert!(g.perm(sturdy).on_bf, "one more toughness survives it");
}

#[test]
fn judgment_strike_back_does_nothing_without_a_commander_creature() {
    let mut g = shielded();
    let a = body(&mut g, P1, 3, 1);
    let life = g.player(P0).life;
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, life, "the damage is still prevented");
    assert!(g.perm(a).on_bf, "no commander on the battlefield: nothing strikes back");
}

#[test]
fn judgment_prevents_burn_with_a_commander_out_and_nothing_to_strike() {
    let mut g = shielded();
    tapped_cloud(&mut g);
    // burn from a spell: prevented, and no creature to answer it (a ping has no source creature here)
    let life = g.player(P0).life;
    lose_life(&mut g, P0, 3, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life);
}

// ------------------------------------------------------------------ Judgment of Alexander: the AI
#[test]
fn the_ai_casts_judgment_against_a_lethal_attack() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &[JUDGMENT]);
    g.player_mut(P0).life = 6;
    let a = body(&mut g, P1, 7, 7);
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, 6, "survived");
    assert!(in_gy(&g, P0, JUDGMENT), "Judgment of Alexander was cast");
}

#[test]
fn the_ai_casts_judgment_when_the_strike_back_kills_a_worthwhile_attacker() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false); // (Thalia taxes the spell)
    hand(&mut g, P0, &[JUDGMENT]);
    let cl = tapped_cloud(&mut g);
    // a worthwhile attacker within Cloud's power, hitting for little: three damage in all
    let thalia = perm(&mut g, P1, "Thalia, Guardian of Thraben");
    assert!(etgh(&g, thalia) <= epow(&g, cl));
    let pawn = body(&mut g, P1, 1, 5);
    combat::resolve_combat(&mut g, P1, &[thalia, pawn], P0, &[]).unwrap();
    assert!(in_gy(&g, P0, JUDGMENT), "cast: the strike back kills Thalia");
    assert!(!g.perm(thalia).on_bf);
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn the_ai_holds_judgment_against_a_trivial_attack() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &[JUDGMENT]);
    let a = body(&mut g, P1, 2, 2);
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, 38, "two damage taken");
    assert!(in_hand(&g, P0, JUDGMENT), "held");
    // and without the mana it is not cast either
    let mut g = game();
    hand(&mut g, P0, &[JUDGMENT]);
    g.player_mut(P0).life = 6;
    let a = body(&mut g, P1, 7, 7);
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert!(in_hand(&g, P0, JUDGMENT));
}

#[test]
fn the_ai_answers_a_big_burn_spell_aimed_at_it_but_not_a_small_one() {
    for (spell, cast_it) in [("Heated Argument", true), ("Lightning Bolt", false)] {
        let mut g = game();
        lands(&mut g, P0, "Plains", 3, false);
        hand(&mut g, P0, &[JUDGMENT]);
        lands(&mut g, P1, "Mountain", 5, false);
        let c = hand(&mut g, P1, &[spell])[0];
        let (gn, pips) = mana::cost_of(&g, P1, c);
        assert!(mana::pay(&mut g, P1, gn, &pips, false).unwrap());
        let life = g.player(P0).life;
        cast::cast_card(&mut g, P1, c, "hand", Ctx { face: Some(P0), ..Ctx::default() }).unwrap();
        assert_eq!(in_gy(&g, P0, JUDGMENT), cast_it, "{spell}");
        if cast_it {
            assert_eq!(g.player(P0).life, life, "the burn spell was prevented");
        } else {
            assert_eq!(g.player(P0).life, life - 3);
        }
    }
}

// ------------------------------------------------------------------ Forge Anew
/// P0's turn with Forge Anew out, Cloud and two Equipment (a Sword {2}, the Hammer {8}) loose
fn forge_table() -> (Game, PermId, PermId, PermId) {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, FORGE);
    let cl = perm(&mut g, P0, CLOUD);
    let sword = perm(&mut g, P0, "Sword of Fire and Ice");
    let hammer = perm(&mut g, P0, "Colossus Hammer");
    for e in [sword, hammer] {
        g.perm_mut(e).attached = None;
    }
    (g, cl, sword, hammer)
}

#[test]
fn forge_anew_makes_the_first_equip_each_turn_free_and_the_second_full_price() {
    let (mut g, cl, sword, hammer) = forge_table();
    lands(&mut g, P0, "Plains", 6, false);
    assert_eq!(cloud::equip_cost(&g, P0, sword, cl, 2), 0, "the first equip is {{0}}");
    assert_eq!(cloud::equip_cost(&g, P0, hammer, cl, 8), 0);
    // the AI offers both at 0 now
    let free: Vec<_> = dsl::equip_options(&g, P0);
    assert!(free.iter().all(|o| matches!(o.act, Some(sim_core::hooks::Action::Equip { n: 0, .. }))), "{free:?}");
    // ... and spends the free one on the dearest equip
    let top = free.iter().max_by(|a, b| a.utility.total_cmp(&b.utility)).unwrap();
    assert!(
        matches!(top.act, Some(sim_core::hooks::Action::Equip { src, .. }) if src == hammer),
        "the Hammer, not the Sword: {top:?}"
    );
    let before = mana::total_mana(&g, P0, false);
    assert!(dsl::equip(&mut g, P0, hammer, cl, 0).unwrap());
    assert_eq!(g.perm(hammer).attached, Some(cl));
    assert_eq!(mana::total_mana(&g, P0, false), before, "no mana spent");
    assert_eq!(cloud::equip_cost(&g, P0, sword, cl, 2), 2, "the second equip pays in full");
    // a new turn: free again
    g.round += 1;
    assert_eq!(cloud::equip_cost(&g, P0, sword, cl, 2), 0);
}

#[test]
fn forge_anew_cost_applies_on_the_controllers_turn_only() {
    let (mut g, cl, sword, _) = forge_table();
    g.active = Some(P1);
    assert_eq!(cloud::equip_cost(&g, P0, sword, cl, 2), 2, "an opponent's turn: full price");
    assert!(!cloud::forge_instant(&g, P0));
    g.active = Some(P0);
    assert!(cloud::forge_instant(&g, P0));
}

#[test]
fn forge_anew_equips_in_combat_on_its_controllers_turn_but_not_on_an_opponents() {
    // after blockers, an unblocked attacker takes the Equipment
    let (mut g, cl, sword, hammer) = forge_table();
    g.perm_mut(cl).sick = false;
    let life = g.player(P1).life;
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    let worn = [sword, hammer].iter().filter(|&&e| g.perm(e).attached == Some(cl)).count();
    assert_eq!(worn, 1, "the one free equip was made, mid-combat");
    assert!(g.player(P1).life < life);
    // not on an opponent's turn
    let (mut g, cl, sword, hammer) = forge_table();
    g.active = Some(P1);
    g.perm_mut(cl).sick = false;
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    assert!(g.perm(sword).attached.is_none() && g.perm(hammer).attached.is_none(), "no instant-speed equip");
}

#[test]
fn forge_anew_leaves_equipment_on_an_attacker_alone() {
    let (mut g, cl, sword, hammer) = forge_table();
    g.perm_mut(cl).sick = false;
    g.perm_mut(sword).attached = Some(cl);
    g.perm_mut(hammer).attached = Some(cl);
    combat::resolve_combat(&mut g, P0, &[cl], P1, &[cl]).unwrap();
    assert_eq!(g.perm(sword).attached, Some(cl));
    assert_eq!(g.perm(hammer).attached, Some(cl));
}

#[test]
fn forge_anew_enters_and_returns_an_equipment_not_another_card() {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P0);
    let ids: Vec<CardId> =
        ["Swords to Plowshares", "Sword of Fire and Ice", "Brass Squire"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).gy.extend(ids);
    perm(&mut g, P0, FORGE);
    assert!(on_bf(&g, P0, "Sword of Fire and Ice").is_some(), "the Equipment returned");
    assert!(in_gy(&g, P0, "Swords to Plowshares") && in_gy(&g, P0, "Brass Squire"), "nothing else did");
    // no Equipment in the graveyard: nothing happens
    let mut g = table(&["cloud", "seph"]);
    let c = card(&g, "Swords to Plowshares");
    g.player_mut(P0).gy.push(c);
    perm(&mut g, P0, FORGE);
    assert!(in_gy(&g, P0, "Swords to Plowshares"));
}
