//! generic-paths (2026-11-01): the cards whose only play path was a deck's hand-written AI (deleted on 2026-11-01,
//! `audit/phase4/hand-off-design.md`, "Deleted") or a deck-key gate no live deck opens, and the generic path each got
//! (`audit/phase4/generic-paths-2026-11-01.md`). For each card, with the defaults (no `AI_HAND`, no `force_hand`), the
//! generic policy plays it in a hand-built position where it should, and declines where it shouldn't.

use sim_core::ai::{self, pool};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Game, Step};
use sim_core::testkit::{card, hand, lands, perm, table, take, token};

const P: PlayerId = PlayerId(0);
const Q: PlayerId = PlayerId(1);
const OPP: &str = "krenko-mono-red-goblins";

/// deck `key` in seat 0 against a pool deck
fn seated(key: &str) -> Game {
    table(&[key, OPP])
}

/// Q's turn, in the declare attackers step
fn q_attacks(g: &mut Game) {
    g.active = Some(Q);
    g.step = Step::Combat;
}

fn creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && g.perm(m).on_bf).count()
}

/// the card-code options of permanent `src` (P's)
fn perm_options(g: &mut Game, src: PermId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("card options");
    f(g, src, P, post).unwrap()
}

/// the hand options of card c in P's hand
fn hand_options(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, P, post).unwrap()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ answers to an attack (galadriel's, any deck)
#[test]
fn ballista_squad_kills_a_valuable_attacker_and_ignores_a_token() {
    let mut g = seated("galadriel");
    let b = perm(&mut g, P, "Ballista Squad");
    lands(&mut g, P, "Plains", 8, false);
    let a = perm(&mut g, Q, "Grave Titan");
    q_attacks(&mut g);
    assert!(ai::attack_response(&mut g, P, Q, &[a]).unwrap());
    assert!(g.perm(b).tapped, "Ballista Squad used");
    assert!(!g.perm(a).on_bf, "Grave Titan dealt 6 and destroyed");

    let mut g = seated("galadriel");
    let b = perm(&mut g, P, "Ballista Squad");
    lands(&mut g, P, "Plains", 8, false);
    let t = token(&mut g, Q, 1);
    q_attacks(&mut g);
    assert!(!ai::attack_response(&mut g, P, Q, &[t]).unwrap());
    assert!(!g.perm(b).tapped, "not on a 1/1 token");
}

#[test]
fn lawbringer_and_lightbringer_exile_an_attacker_of_their_colour_only() {
    let mut g = seated("galadriel");
    let l = perm(&mut g, P, "Lightbringer");
    let law = perm(&mut g, P, "Lawbringer");
    let a = perm(&mut g, Q, "Grave Titan"); // black
    q_attacks(&mut g);
    assert!(ai::attack_response(&mut g, P, Q, &[a]).unwrap());
    assert!(!g.perm(a).on_bf, "Lightbringer exiles the black attacker");
    assert!(!g.perm(l).on_bf, "Lightbringer sacrificed");
    assert!(g.perm(law).on_bf, "Lawbringer (red only) stays");
}

#[test]
fn the_tappers_tap_the_attacker_before_another_players_combat() {
    let mut g = seated("galadriel");
    let w = perm(&mut g, P, "Whipcorder");
    let d = perm(&mut g, P, "Errant Doomsayers");
    lands(&mut g, P, "Plains", 1, false);
    let a = perm(&mut g, Q, "Grave Titan");
    q_attacks(&mut g);
    sim_core::engine::stack::step_priority(&mut g, "combat", None, &[]).unwrap();
    assert!(g.perm(a).tapped, "Whipcorder taps the Titan");
    assert!(g.perm(w).tapped);
    assert!(!g.perm(d).tapped, "Errant Doomsayers can't: toughness 6");

    // on its own turn nothing is tapped by this (its own taps are the cards' own-turn options)
    let mut g = seated("galadriel");
    let w = perm(&mut g, P, "Whipcorder");
    lands(&mut g, P, "Plains", 1, false);
    perm(&mut g, Q, "Grave Titan");
    g.step = Step::Combat;
    sim_core::engine::stack::step_priority(&mut g, "combat", None, &[]).unwrap();
    assert!(!g.perm(w).tapped);
}

#[test]
fn mishras_factory_animates_to_block_a_dangerous_attack_only() {
    let mut g = seated("karn");
    lands(&mut g, P, "Mishra's Factory", 1, false);
    lands(&mut g, P, "Wastes", 2, false);
    g.player_mut(P).life = 10;
    let a = perm(&mut g, Q, "Grave Titan");
    q_attacks(&mut g);
    let before = creatures(&g, P);
    ai::attack_response(&mut g, P, Q, &[a]).unwrap();
    assert_eq!(creatures(&g, P), before + 1, "the Factory is a 2/2 blocker");

    let mut g = seated("karn");
    lands(&mut g, P, "Mishra's Factory", 1, false);
    lands(&mut g, P, "Wastes", 2, false);
    let t = token(&mut g, Q, 1);
    q_attacks(&mut g);
    let before = creatures(&g, P);
    ai::attack_response(&mut g, P, Q, &[t]).unwrap();
    assert_eq!(creatures(&g, P), before, "a 1/1 at 40 life is no danger");
}

// ------------------------------------------------------------------ protection the deleted hand AIs cast
/// Q's targeted removal (`kind`) at P's m: did P protect it?
fn protect(g: &mut Game, m: PermId, kind: &'static str) -> bool {
    let spell = card(g, "Swords to Plowshares");
    ai::protect_response(g, P, m, kind, Some(Q), Some(spell)).unwrap()
}

#[test]
fn unbreakable_formation_make_a_stand_and_rootborn_save_from_destroy_not_exile() {
    for name in ["Unbreakable Formation", "Make a Stand", "Rootborn Defenses"] {
        let mut g = seated("galadriel");
        let m = perm(&mut g, P, "Grave Titan");
        lands(&mut g, P, "Plains", 3, false);
        hand(&mut g, P, &[name]);
        g.active = Some(Q);
        assert!(!protect(&mut g, m, "exile"), "{name}: indestructible doesn't stop exile");
        assert!(in_hand(&g, P, name), "{name}: kept");
        assert!(protect(&mut g, m, "destroy"), "{name}: saves from destroy");
        assert!(in_gy(&g, P, name), "{name}: cast");
        assert!(g.perm(m).eot_kw.contains(&"indestructible"), "{name}: indestructible until end of turn");
    }
}

#[test]
fn unbreakable_formation_answers_a_destroy_wipe() {
    let mut g = seated("galadriel");
    perm(&mut g, P, "Grave Titan");
    perm(&mut g, P, "Lightbringer");
    lands(&mut g, P, "Plains", 3, false);
    hand(&mut g, P, &["Unbreakable Formation"]);
    g.active = Some(Q);
    assert_eq!(ai::wipe_response(&mut g, P, "destroy", Q).unwrap(), Some("indes"));
    assert!(in_gy(&g, P, "Unbreakable Formation"));
}

#[test]
fn eerie_interlude_exiles_the_target_until_the_end_step() {
    let mut g = seated("galadriel");
    let m = perm(&mut g, P, "Grave Titan");
    lands(&mut g, P, "Plains", 3, false);
    hand(&mut g, P, &["Eerie Interlude"]);
    g.active = Some(Q);
    assert!(protect(&mut g, m, "exile"));
    assert!(!g.perm(m).on_bf, "exiled by Interlude (it returns at the end step)");
    let titan = card(&g, "Grave Titan");
    assert!(g.player(P).exile.contains(&titan));
}

#[test]
fn blossoming_defense_saves_a_valuable_creature_not_a_token() {
    let mut g = seated("thrasios");
    let m = perm(&mut g, P, "Grave Titan");
    lands(&mut g, P, "Forest", 1, false);
    hand(&mut g, P, &["Blossoming Defense"]);
    g.active = Some(Q);
    let t = token(&mut g, P, 1);
    assert!(!protect(&mut g, t, "exile"), "a 1/1 token isn't worth the card");
    assert!(protect(&mut g, m, "exile"), "hexproof stops any targeted removal");
    assert!(in_gy(&g, P, "Blossoming Defense"));
    assert_eq!(g.perm(m).eot_pt, (2, 2));
}

#[test]
fn slip_out_the_back_phases_the_creature_out() {
    let mut g = seated("sauron");
    let m = perm(&mut g, P, "Grave Titan");
    lands(&mut g, P, "Island", 1, false);
    hand(&mut g, P, &["Slip Out the Back"]);
    g.active = Some(Q);
    assert!(protect(&mut g, m, "exile"));
    assert!(g.perm(m).phased);
    assert!(in_gy(&g, P, "Slip Out the Back"));
}

#[test]
fn not_of_this_world_is_free_for_a_seven_power_creature_and_needs_seven_otherwise() {
    let mut g = seated("sauron");
    let m = perm(&mut g, P, "Grave Titan"); // 6 power
    hand(&mut g, P, &["Not of This World"]);
    g.active = Some(Q);
    assert!(!protect(&mut g, m, "exile"), "no mana and power 6: not cast");
    g.perm_mut(m).eot_pt.0 += 1; // 7 power
    assert!(protect(&mut g, m, "exile"), "free against removal aimed at a 7-power creature");
    assert!(in_gy(&g, P, "Not of This World"));
}

#[test]
fn angelic_shield_bounces_the_target_except_from_a_bounce() {
    let mut g = seated("alela");
    let m = perm(&mut g, P, "Grave Titan");
    let s = perm(&mut g, P, "Angelic Shield");
    g.active = Some(Q);
    assert!(!protect(&mut g, m, "bounce"), "nothing to gain against a bounce");
    assert!(g.perm(s).on_bf);
    assert!(protect(&mut g, m, "destroy"));
    assert!(!g.perm(s).on_bf, "the Shield is sacrificed");
    assert!(in_hand(&g, P, "Grave Titan"), "the Titan is back in hand");
}

// ------------------------------------------------------------------ main-phase plays
#[test]
fn jaces_archivist_wheels_with_bowmasters_out_or_an_empty_hand_only() {
    let mut g = seated("sauron");
    let a = perm(&mut g, P, "Jace's Archivist");
    lands(&mut g, P, "Island", 1, false);
    hand(&mut g, P, &["Island", "Swamp", "Mountain", "Plains"]);
    hand(&mut g, Q, &["Island", "Swamp", "Mountain", "Plains"]);
    assert!(perm_options(&mut g, a, Some(false)).is_empty(), "a full hand and no payoff: no wheel");
    perm(&mut g, P, "Orcish Bowmasters");
    let o = perm_options(&mut g, a, Some(false));
    assert_eq!(o.len(), 1, "Bowmasters pings every extra draw");
    let Some(Action::Ability { src, f, arg }) = o[0].act.clone() else { panic!("an ability") };
    assert!(f(&mut g, src, P, arg).unwrap());
    assert!(g.perm(a).tapped);
    assert_eq!(g.player(Q).hand.len(), 4, "everyone discards and draws four");
    assert!(perm_options(&mut g, a, Some(false)).is_empty(), "once a turn ({{T}})");
}

#[test]
fn yawgmoths_will_with_a_storm_card_in_the_yard_not_an_empty_one() {
    let mut g = seated("kefka");
    lands(&mut g, P, "Swamp", 6, false);
    let w = hand(&mut g, P, &["Yawgmoth's Will"])[0];
    assert!(hand_options(&mut g, w, Some(false)).is_empty(), "an empty graveyard");
    let bf = take(&mut g, P, "Brain Freeze");
    g.player_mut(P).gy.push(bf);
    let o = hand_options(&mut g, w, Some(false));
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].utility, 7.0);
    g.active = Some(Q);
    assert!(hand_options(&mut g, w, Some(false)).is_empty(), "a sorcery: p's own turn");
}

#[test]
fn susurian_voidborn_is_cast_from_exile_on_a_later_turn_after_its_warp() {
    let mut g = seated("kefka");
    lands(&mut g, P, "Swamp", 3, false);
    let v = take(&mut g, P, "Susurian Voidborn");
    g.player_mut(P).exile.push(v);
    let opts = |g: &mut Game| sim_core::impls::generic_paths::exile_options(g, P, Some(false)).unwrap();
    assert!(opts(&mut g).is_empty(), "exiled some other way: not castable");
    let st = g.turn_stamp();
    g.player_mut(P).flag_turn.insert(sim_core::sym::intern("voidborn_warped"), st);
    assert!(opts(&mut g).is_empty(), "warped this turn: a later turn only");
    g.round += 1;
    let o = opts(&mut g);
    assert_eq!(o.len(), 1);
    let Some(Action::Plan { f, arg }) = o[0].act.clone() else { panic!("a plan") };
    assert!(f(&mut g, P, arg).unwrap());
    assert!(!g.player(P).exile.contains(&v));
    assert!(g.player(P).perms.iter().any(|&m| g.perm(m).cd == Some(v)), "on the battlefield");
}

#[test]
fn exsanguinate_is_offered_to_sauron_through_its_plan_plays() {
    let mut g = seated("sauron");
    lands(&mut g, P, "Swamp", 12, false);
    g.player_mut(Q).life = 5;
    let c = hand(&mut g, P, &["Exsanguinate"])[0];
    assert!(sim_core::ai::plan_data::plays(&g, P, "Exsanguinate"));
    assert!(!hand_options(&mut g, c, Some(false)).is_empty(), "the X drain for the kill");
    g.active = Some(Q);
    assert!(hand_options(&mut g, c, Some(false)).is_empty(), "a sorcery: not on another player's turn");
}

#[test]
fn underworld_breach_is_cast_only_with_an_escape_to_make() {
    let mut g = seated("kefka");
    lands(&mut g, P, "Mountain", 4, false);
    let b = hand(&mut g, P, &["Underworld Breach"])[0];
    assert_eq!(pool::generic_prio(&g, P, b), 0, "an empty graveyard: nothing to escape");
    for n in ["Lightning Bolt", "Island", "Swamp", "Plains"] {
        let c = take(&mut g, P, n);
        g.player_mut(P).gy.push(c);
    }
    assert_eq!(pool::generic_prio(&g, P, b), 0, "Lightning Bolt with nothing to kill is no escape");
    perm(&mut g, Q, "Krenko, Mob Boss");
    assert!(pool::generic_prio(&g, P, b) > 0, "Lightning Bolt on Krenko can be escaped with the mana left");
}

#[test]
fn equipment_with_a_card_code_equip_is_equipped_by_the_generic_policy() {
    let equip_of = |g: &Game, e: PermId| {
        sim_core::dsl::equip_options(g, P)
            .into_iter()
            .any(|o| matches!(o.act, Some(Action::Equip { src, .. }) if src == e))
    };
    for (key, eq, mana) in [
        ("urdragon", "Sword of Feast and Famine", 2),
        ("thrasios", "Conqueror's Flail", 2),
        ("aurelia-boros-extra-combats", "Sword of the Animist", 2),
        ("ghave", "Nim Deathmantle", 4),
    ] {
        let mut g = seated(key);
        let e = perm(&mut g, P, eq);
        perm(&mut g, P, "Grave Titan");
        lands(&mut g, P, "Wastes", mana - 1, false);
        assert!(!equip_of(&g, e), "{key}: {eq} needs {mana}");
        lands(&mut g, P, "Wastes", 1, false);
        assert!(equip_of(&g, e), "{key}: {eq} equipped");
    }
    // cloud plays its hand AI (on by default), which equips its Sword itself: no generic equip added
    let mut g = seated("cloud");
    let e = perm(&mut g, P, "Sword of Feast and Famine");
    perm(&mut g, P, "Grave Titan");
    lands(&mut g, P, "Wastes", 4, false);
    assert!(!equip_of(&g, e));
}

#[test]
fn lightning_greaves_moves_off_the_creature_daydream_wants() {
    let mut g = seated("alela");
    let m = perm(&mut g, P, "Mulldrifter");
    token(&mut g, P, 2);
    let gr = perm(&mut g, P, "Lightning Greaves");
    g.perm_mut(gr).attached = Some(m);
    lands(&mut g, P, "Plains", 1, false);
    g.player_mut(P).hand.clear(); // Mulldrifter drew two
    assert!(perm_options(&mut g, gr, Some(false)).is_empty(), "no Aura or Daydream for it");
    hand(&mut g, P, &["Daydream"]);
    let o = perm_options(&mut g, gr, Some(false));
    assert_eq!(o.len(), 1, "move the Greaves so Daydream can target Mulldrifter");
    assert!(perm_options(&mut g, gr, None).is_empty(), "before combat on its own turn");
}

#[test]
fn ephemerate_blinks_an_enter_effect_creature_for_value() {
    let mut g = seated("ghave");
    lands(&mut g, P, "Plains", 1, false);
    let c = hand(&mut g, P, &["Ephemerate"])[0];
    token(&mut g, P, 2);
    assert!(hand_options(&mut g, c, Some(false)).is_empty(), "nothing worth blinking");
    perm(&mut g, P, "Grave Titan");
    assert_eq!(hand_options(&mut g, c, Some(false)).len(), 1, "two Zombies a blink");
}

#[test]
fn mizzixs_mastery_is_offered_to_veyran() {
    // the audit's claim: its gate (`is_main(hand_key(..))`) reads Veyran as a generic deck now, so it is reachable
    let mut g = seated("veyran");
    lands(&mut g, P, "Mountain", 8, false);
    let c = hand(&mut g, P, &["Mizzix's Mastery"])[0];
    for n in ["Brainstorm", "Ponder", "Preordain", "Lightning Bolt"] {
        let x = take(&mut g, P, n);
        g.player_mut(P).gy.push(x);
    }
    assert!(!hand_options(&mut g, c, Some(false)).is_empty());
}
