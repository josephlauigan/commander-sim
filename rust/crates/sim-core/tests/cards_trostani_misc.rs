//! Trostani deck, Batch C (combat and AI-side cards): Wall of Omens, Elder Gargaroth, Sunblast Angel, Cloudshift,
//! Aetherflux Reservoir, Castle Ardenvale, and the cards that only needed a test (Beast Whisperer, Eladamri's Call,
//! Kodama's Reach, Thalia the Survivor, Archon of Absolution, Sphere of Safety). Every expected number is derived from the
//! card's Oracle text (and the rulings in /tmp/handoff/trostani-opus-design.md), not from what the engine printed.
//! Card code: impls/trostani_misc.rs; the defender filter is in engine/combat.rs, Cloudshift's scope in ai/pool.rs.

mod play_common;
use play_common::*;
use sim_core::ai;
use sim_core::ai::act::perform;
use sim_core::engine::cast::{cast_card, castable};
use sim_core::engine::combat::{self, attack_limits, attack_restrictions};
use sim_core::engine::mana::cost_of;
use sim_core::engine::repl;
use sim_core::state::{Ctx, Step};

const DECK: &str = "trostani";
const OPP: &str = "seph";

/// P0 plays the trostani deck, P1 a pool deck; both hands empty, P0's first main phase, 40 life each
fn tbl() -> Game {
    let mut g = table(&[DECK, OPP]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn beasts(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"beast")).collect()
}

fn tokens_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count()
}

fn tap(g: &mut Game, m: PermId) {
    g.perm_mut(m).tapped = true;
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn dead(g: &Game, q: PlayerId) -> bool {
    !g.player(q).alive || g.player(q).life <= 0
}

/// Gargaroth attacks (the attack trigger of the engine: the whole attack event), P1 defending
fn attacks(g: &mut Game, a: PermId) {
    combat::attack_triggers(g, P0, &[a], P1).unwrap();
}

// ================================================================== Wall of Omens
// Defender. When this creature enters, draw a card.

#[test]
fn wall_of_omens_draws_a_card_when_it_enters() {
    let mut g = tbl();
    assert_eq!(hand_len(&g, P0), 0);
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(hand_len(&g, P0), 1, "enters: draw one card");
}

#[test]
fn wall_of_omens_with_five_counters_never_attacks() {
    // CR 702.3b: a creature with defender can't attack, whatever its power. The control: a vanilla 5-power creature
    // attacks the same empty board, so the AI does want to attack with it.
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wall of Omens");
    repl::put_plus(&mut g, w, 5);
    assert_eq!(sim_core::engine::values::epow(&g, w), 5, "0/4 with five +1/+1 counters is 5/9");
    combat::combat(&mut g, P0).unwrap();
    assert!(!g.perm(w).tapped, "the Wall did not attack");
    assert_eq!(g.player(P1).life, 40, "no damage dealt");

    let mut g = tbl();
    let big = token(&mut g, P0, 5);
    combat::combat(&mut g, P0).unwrap();
    assert!(g.perm(big).tapped, "control: the AI attacks an empty board with a 5/5 token");
    assert_eq!(g.player(P1).life, 35);
}

// ================================================================== Elder Gargaroth
// Reach, vigilance, trample. Whenever this creature attacks or blocks, choose one: a 3/3 green Beast creature token; you
// gain 3 life; draw a card. (No enters trigger.)

/// Gargaroth onto P0's battlefield. data/cards.json (as of 4a98f340) still carries the auto-tags `tok=1 tokp=3` from the
/// Scryfall text, which make a 3/3 token as it enters; the override in the handoff removes them. Until the coordinator
/// applies it, this helper exiles any token made as it entered so the attack and block tests read the trigger alone.
fn gargaroth(g: &mut Game) -> PermId {
    let before = g.player(P0).perms.clone();
    let a = perm(g, P0, "Elder Gargaroth");
    let strays: Vec<PermId> =
        g.player(P0).perms.iter().copied().filter(|m| !before.contains(m) && g.perm(*m).token).collect();
    for m in strays {
        sim_core::engine::zones::exile_perm(g, m).unwrap();
    }
    a
}

#[test]
fn gargaroth_has_no_enters_trigger() {
    let mut g = tbl();
    let before = hand_len(&g, P0);
    perm(&mut g, P0, "Elder Gargaroth");
    assert_eq!(tokens_of(&g, P0), 0, "no Beast as it enters");
    assert_eq!(hand_len(&g, P0), before);
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn gargaroth_attacking_gives_exactly_one_mode_a_card_by_default() {
    let mut g = tbl();
    let a = gargaroth(&mut g);
    token(&mut g, P0, 1); // another creature: no blocker is needed
    let before = hand_len(&g, P0);
    attacks(&mut g, a);
    assert_eq!(hand_len(&g, P0), before + 1, "the card mode");
    assert!(beasts(&g, P0).is_empty(), "not the Beast");
    assert_eq!(g.player(P0).life, 40, "not the life");
}

#[test]
fn gargaroth_blocking_gives_exactly_one_mode() {
    // P1 attacks P0 with a 2/2; the AI blocks with the 6/6; the block trigger gives one mode
    let mut g = tbl();
    let a = gargaroth(&mut g);
    token(&mut g, P0, 1);
    let atk = token(&mut g, P1, 2);
    g.active = Some(P1);
    let before = hand_len(&g, P0);
    combat::resolve_combat(&mut g, P1, &[atk], P0, &[]).unwrap();
    assert!(!on_bf(&g, atk), "the 2/2 was blocked and died");
    assert!(on_bf(&g, a));
    assert_eq!(hand_len(&g, P0), before + 1, "one card: one mode, once");
    assert!(beasts(&g, P0).is_empty());
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn gargaroth_makes_a_beast_with_trostani_out() {
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    let a = gargaroth(&mut g);
    token(&mut g, P0, 1);
    let before = hand_len(&g, P0);
    let life0 = g.player(P0).life;
    attacks(&mut g, a);
    let b = beasts(&g, P0);
    assert_eq!(b.len(), 1, "a Beast token (the creature entering feeds Trostani)");
    assert_eq!((sim_core::engine::values::epow(&g, b[0]), sim_core::engine::values::etgh(&g, b[0])), (3, 3));
    assert_eq!(hand_len(&g, P0), before, "not the card");
    // Trostani gains the Beast's toughness (3) as it enters; the life mode would add 3 more (6 in all)
    assert_eq!(g.player(P0).life, life0 + 3, "Trostani's 3 for the Beast, not the life mode on top");
}

#[test]
fn gargaroth_makes_a_beast_when_it_is_the_only_creature() {
    let mut g = tbl();
    let a = gargaroth(&mut g);
    attacks(&mut g, a);
    assert_eq!(beasts(&g, P0).len(), 1, "a blocker for the next turn");
}

#[test]
fn gargaroth_gains_life_at_38_with_felidar_out() {
    // Felidar Sovereign wins at the start of your upkeep with 40 or more life: 38 + 3 = 41 crosses the line
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Sovereign");
    let a = gargaroth(&mut g);
    g.player_mut(P0).life = 38;
    let before = hand_len(&g, P0);
    attacks(&mut g, a);
    assert_eq!(g.player(P0).life, 41, "the life mode");
    assert_eq!(hand_len(&g, P0), before);
    assert!(beasts(&g, P0).is_empty());
}

#[test]
fn gargaroth_does_not_take_life_when_it_does_not_reach_the_line() {
    // 30 + 3 = 33 < 40 with Felidar out: a card; 47 + 3 = 50 with Test of Endurance out: the life
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Sovereign");
    let a = gargaroth(&mut g);
    token(&mut g, P0, 1);
    g.player_mut(P0).life = 30;
    let before = hand_len(&g, P0);
    attacks(&mut g, a);
    assert_eq!(g.player(P0).life, 30);
    assert_eq!(hand_len(&g, P0), before + 1);

    let mut g = tbl();
    perm(&mut g, P0, "Test of Endurance");
    let a = gargaroth(&mut g);
    token(&mut g, P0, 1);
    g.player_mut(P0).life = 47;
    attacks(&mut g, a);
    assert_eq!(g.player(P0).life, 50, "47 + 3 reaches Test of Endurance's 50");
}

#[test]
fn gargaroth_attacks_in_a_real_combat_and_triggers_once() {
    let mut g = tbl();
    let a = gargaroth(&mut g);
    perm(&mut g, P0, "Wall of Omens"); // a defender stays home and is another creature: no blocker is needed, the card mode
    let before = hand_len(&g, P0);
    combat::combat(&mut g, P0).unwrap();
    assert!(g.perm(a).on_bf);
    assert_eq!(g.player(P1).life, 34, "6 trample damage");
    assert_eq!(hand_len(&g, P0), before + 1, "one trigger, one mode");
}

// ================================================================== Sunblast Angel
// Flying. When this creature enters, destroy all tapped creatures.

#[test]
fn sunblast_destroys_every_tapped_creature_yours_included() {
    let mut g = tbl();
    let mine_tapped = token(&mut g, P0, 2);
    let mine_ready = token(&mut g, P0, 2);
    let theirs_tapped = token(&mut g, P1, 3);
    let theirs_ready = token(&mut g, P1, 3);
    tap(&mut g, mine_tapped);
    tap(&mut g, theirs_tapped);
    let angel = perm(&mut g, P0, "Sunblast Angel");
    assert!(!on_bf(&g, mine_tapped), "our own tapped creature is destroyed");
    assert!(!on_bf(&g, theirs_tapped), "their tapped creature is destroyed");
    assert!(on_bf(&g, mine_ready) && on_bf(&g, theirs_ready), "untapped creatures stay");
    assert!(on_bf(&g, angel), "the Angel enters untapped");
}

#[test]
fn sunblast_leaves_an_indestructible_tapped_heliod_creature() {
    // Heliod, Sun-Crowned is indestructible and is a creature only with devotion to white of 5 or more (it is an
    // enchantment otherwise). Devotion with the Angel on the battlefield: Heliod {2}{W} 1, Archon {3}{W} 1, Thalia {3}{W} 1,
    // Sphere {4}{W} 1, the Angel {4}{W}{W} 2 = 6
    let mut g = tbl();
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    perm(&mut g, P0, "Archon of Absolution");
    perm(&mut g, P0, "Thalia, the Survivor");
    perm(&mut g, P0, "Sphere of Safety");
    tap(&mut g, h);
    let thalia = perms_named(&g, P0, "Thalia, the Survivor")[0];
    tap(&mut g, thalia);
    let angel = perm(&mut g, P0, "Sunblast Angel");
    assert!(g.is_creature(h), "devotion 6: Heliod is a creature");
    assert!(on_bf(&g, h), "tapped but indestructible: it survives");
    assert!(!on_bf(&g, thalia), "Thalia, tapped, is destroyed");
    assert!(on_bf(&g, angel));
}

#[test]
fn sunblast_is_not_cast_when_only_our_mana_creatures_would_be_tapped() {
    // six mana from three Plains and three Birds of Paradise: paying taps the Birds, which the Angel then destroys;
    // the opponent has nothing tapped, so the cast is all loss
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    for _ in 0..3 {
        perm(&mut g, P0, "Birds of Paradise");
    }
    let c = hand(&mut g, P0, &["Sunblast Angel"])[0];
    token(&mut g, P1, 3); // untapped
    assert_eq!(ai::pool::generic_prio(&g, P0, c), 0, "the dorks are not free kills");
}

#[test]
fn sunblast_is_cast_when_lands_pay_and_their_creatures_are_tapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    perm(&mut g, P0, "Birds of Paradise"); // not needed to pay: stays untapped
    let c = hand(&mut g, P0, &["Sunblast Angel"])[0];
    let big = token(&mut g, P1, 6);
    tap(&mut g, big); // it attacked last turn and stays tapped
    assert!(ai::pool::generic_prio(&g, P0, c) >= 55, "a clear gain, nothing of ours dies");
}

#[test]
fn sunblast_is_not_cast_when_our_tapped_creature_is_worth_more_than_theirs() {
    // Trostani tapped (her populate ability) dies to our own Angel; a tapped 1/1 of theirs is not worth that
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let t = perm(&mut g, P0, "Trostani, Selesnya's Voice");
    tap(&mut g, t);
    let c = hand(&mut g, P0, &["Sunblast Angel"])[0];
    let small = token(&mut g, P1, 1);
    tap(&mut g, small);
    assert_eq!(ai::pool::generic_prio(&g, P0, c), 0);
}

// ================================================================== Cloudshift
// Exile target creature you control, then return that card to the battlefield under your control.

fn swords_at(g: &mut Game, victim: PermId) -> bool {
    lands(g, P1, "Plains", 1, false);
    let sw = hand(g, P1, &["Swords to Plowshares"])[0];
    g.active = Some(P1);
    cast_card(g, P1, sw, "hand", Ctx { target: Some(victim), ..Ctx::default() }).unwrap()
}

#[test]
fn cloudshift_saves_a_trostani_creature_and_the_removal_fizzles() {
    // Swords to Plowshares would exile the Gargaroth and give its controller 6 life; the AI blinks it in response, so the
    // spell's target is gone: nothing is exiled and no life is gained
    let mut g = tbl();
    let a = gargaroth(&mut g);
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    swords_at(&mut g, a);
    assert!(g.player(P0).gy.contains(&c), "Cloudshift was cast");
    assert!(!on_bf(&g, a), "the old object is gone");
    assert_eq!(perms_named(&g, P0, "Elder Gargaroth").len(), 1, "the Gargaroth came back");
    assert!(!g.player(P0).exile.contains(&card(&g, "Elder Gargaroth")), "not exiled by Swords");
    assert_eq!(g.player(P0).life, 40, "Swords did nothing: no life gained");
}

#[test]
fn cloudshift_blink_runs_the_enters_trigger_again() {
    // the blinked Sunblast Angel is a new object that enters again: "destroy all tapped creatures" happens a second time
    let mut g = tbl();
    let angel = perm(&mut g, P0, "Sunblast Angel");
    repl::put_plus(&mut g, angel, 3); // worth saving to the AI (a value of 4 or more)
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let late = token(&mut g, P1, 3); // tapped after the first enters trigger
    tap(&mut g, late);
    let swords = card(&g, "Swords to Plowshares");
    assert!(ai::protect_response(&mut g, P0, angel, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).gy.contains(&c));
    assert!(!on_bf(&g, late), "the second enters trigger destroyed the tapped creature");
    assert_eq!(perms_named(&g, P0, "Sunblast Angel").len(), 1);
}

#[test]
fn cloudshift_protects_only_in_the_trostani_deck() {
    // the same position in another deck: the AI keeps Cloudshift (for value or a combo, as before this deck)
    let mut g = table(&["sheoldred", OPP]);
    let t = gargaroth(&mut g);
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let swords = card(&g, "Swords to Plowshares");
    assert!(!ai::protect_response(&mut g, P0, t, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).hand.contains(&c), "Cloudshift stays in hand");
    assert!(on_bf(&g, t));
}

#[test]
fn cloudshift_does_not_blink_a_token_away() {
    // a token that leaves the battlefield ceases to exist: blinking it saves nothing
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let swords = card(&g, "Swords to Plowshares");
    assert!(!ai::protect_response(&mut g, P0, t, "exile", Some(P1), Some(swords)).unwrap());
    assert!(g.player(P0).hand.contains(&c));
}

// ================================================================== cards that only needed a test

#[test]
fn beast_whisperer_draws_for_a_creature_spell_and_not_for_another_spell() {
    // Whenever you cast a creature spell, draw a card.
    let mut g = tbl();
    perm(&mut g, P0, "Beast Whisperer");
    lands(&mut g, P0, "Forest", 2, false);
    let bird = hand(&mut g, P0, &["Birds of Paradise"])[0];
    assert_eq!(hand_len(&g, P0), 1);
    assert!(cast_card(&mut g, P0, bird, "hand", Ctx::default()).unwrap());
    assert_eq!(hand_len(&g, P0), 1, "the Birds left the hand and a card was drawn");

    let mut g = tbl();
    perm(&mut g, P0, "Beast Whisperer");
    lands(&mut g, P0, "Forest", 2, false);
    let sig = hand(&mut g, P0, &["Arcane Signet"])[0];
    assert!(cast_card(&mut g, P0, sig, "hand", Ctx::default()).unwrap());
    assert_eq!(hand_len(&g, P0), 0, "an artifact spell draws nothing");
}

#[test]
fn eladamris_call_finds_a_creature_at_instant_speed() {
    // Instant: search your library for a creature card, reveal it, put it into your hand, then shuffle.
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Eladamri's Call"])[0];
    g.active = Some(P1); // an opponent's turn
    g.step = Step::Main1;
    assert!(castable(&g, P0, c, "hand"), "an instant: castable on the opponent's turn");
    let lib = g.player(P0).library.len();
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(g.player(P0).library.len(), lib - 1);
    assert_eq!(hand_len(&g, P0), 1, "the creature card");
    let found = g.player(P0).hand[0];
    assert!(g.db.get(found).creature, "a creature card");
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn kodamas_reach_puts_one_basic_onto_the_battlefield_tapped_and_one_into_hand() {
    // Search your library for up to two basic land cards, reveal them, put one onto the battlefield tapped and the other
    // into your hand, then shuffle.
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Kodama's Reach"])[0];
    let lib = g.player(P0).library.len();
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(g.player(P0).library.len(), lib - 2, "two cards left the library");
    let ls = &g.player(P0).lands;
    assert_eq!(ls.len(), 4, "one new land on the battlefield");
    let new = *ls.last().unwrap();
    assert!(g.perm(new).tapped, "tapped");
    assert!(g.db.get(g.perm(new).card()).basic, "a basic land");
    assert_eq!(hand_len(&g, P0), 1, "one card in hand");
    let h = g.player(P0).hand[0];
    assert!(g.db.get(h).land && g.db.get(h).basic, "a basic land card");
}

#[test]
fn thalia_taxes_an_opponents_noncreature_spell_and_not_their_creature_spell() {
    // Noncreature spells your opponents cast cost {1} more to cast.
    let mut g = tbl();
    let sw = hand(&mut g, P1, &["Swords to Plowshares"])[0];
    let bird = hand(&mut g, P1, &["Birds of Paradise"])[0];
    let mine = hand(&mut g, P0, &["Arcane Signet"])[0];
    assert_eq!(cost_of(&g, P1, sw), (0, "W".to_string()));
    perm(&mut g, P0, "Thalia, the Survivor");
    assert_eq!(cost_of(&g, P1, sw), (1, "W".to_string()), "Swords costs {{1}} more");
    assert_eq!(cost_of(&g, P1, bird), (0, "G".to_string()), "a creature spell costs the same");
    assert_eq!(cost_of(&g, P0, mine).0, 2, "her controller's own spells are not taxed");
}

#[test]
fn thalia_has_lifelink() {
    // Lifelink: damage dealt by Thalia (3) gains her controller that much life
    let mut g = tbl();
    let t = perm(&mut g, P0, "Thalia, the Survivor");
    combat::resolve_combat(&mut g, P0, &[t], P1, &[t]).unwrap();
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P0).life, 43, "3 damage, 3 life");
}

#[test]
fn archon_of_absolution_attack_tax_is_paid_or_the_attack_does_not_happen() {
    // Creatures can't attack you unless their controller pays {1} for each of those creatures.
    let mut g = tbl();
    perm(&mut g, P0, "Archon of Absolution");
    assert_eq!(attack_restrictions(&g, P1, P0).0, 1);
    let atk: Vec<PermId> = (0..2).map(|_| token(&mut g, P1, 2)).collect();
    let none = attack_limits(&mut g, P1, P0, atk.clone()).unwrap();
    assert!(none.is_empty(), "no mana: the attack does not happen");
    let ls = lands(&mut g, P1, "Plains", 2, false);
    let some = attack_limits(&mut g, P1, P0, atk).unwrap();
    assert_eq!(some.len(), 2, "two attackers, {{1}} each");
    assert!(ls.iter().all(|&l| g.perm(l).tapped), "paid");
}

#[test]
fn sphere_of_safety_tax_is_the_number_of_enchantments_you_control() {
    // pays {X} for each attacker, X the number of enchantments the defender controls
    let mut g = tbl();
    perm(&mut g, P0, "Sphere of Safety");
    assert_eq!(attack_restrictions(&g, P1, P0).0, 1, "one enchantment: Sphere itself");
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Cathars' Crusade");
    assert_eq!(attack_restrictions(&g, P1, P0).0, 3, "three enchantments");
    let atk: Vec<PermId> = (0..2).map(|_| token(&mut g, P1, 3)).collect(); // the AI attacks when power covers the tax
    lands(&mut g, P1, "Plains", 4, false);
    let out = attack_limits(&mut g, P1, P0, atk).unwrap();
    assert_eq!(out.len(), 1, "4 mana pays {{3}} for one attacker, not {{6}} for two");
}

// ================================================================== Aetherflux Reservoir
// Pay 50 life: this artifact deals 50 damage to any target. The AI (JD, 2026-11-27) shoots to kill an opponent when it
// stays alive: its life must be 51 or more and the target's 50 or less.

fn aether_option(g: &mut Game) -> Option<sim_core::hooks::Opt> {
    ai::brain::main_options(g, P0, false)
        .unwrap()
        .into_iter()
        .find(|o| o.label.starts_with("Aetherflux Reservoir: pay 50"))
}

#[test]
fn aetherflux_ai_pays_50_at_60_life_to_kill_an_opponent_at_30() {
    let mut g = table_with(&[DECK, OPP], 1, 40);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 60;
    g.player_mut(P1).life = 30;
    let o = aether_option(&mut g).expect("the shot is offered");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(dead(&g, P1), "50 damage to an opponent at 30");
    assert_eq!(g.player(P0).life, 10, "paid 50 life and lives");
}

#[test]
fn aetherflux_ai_does_not_pay_at_50_life() {
    // paying 50 at 50 life leaves the AI dead
    let mut g = table_with(&[DECK, OPP], 1, 40);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 50;
    g.player_mut(P1).life = 30;
    assert!(aether_option(&mut g).is_none());
}

#[test]
fn aetherflux_ai_pays_at_51_life_and_keeps_1() {
    let mut g = table_with(&[DECK, OPP], 1, 40);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 51;
    g.player_mut(P1).life = 50;
    let o = aether_option(&mut g).expect("51 life pays for a kill and stays alive");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(dead(&g, P1));
    assert_eq!(g.player(P0).life, 1);
}

#[test]
fn aetherflux_ai_does_not_shoot_when_it_would_not_kill() {
    let mut g = table_with(&[DECK, OPP], 1, 40);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 90;
    g.player_mut(P1).life = 51;
    assert!(aether_option(&mut g).is_none(), "51 damage needed, 50 dealt");
}

#[test]
fn aetherflux_ai_shoots_the_most_threatening_opponent_it_can_kill() {
    let mut g = table_with(&[DECK, OPP, "cloud"], 1, 40);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 70;
    g.player_mut(P1).life = 20;
    g.player_mut(P2).life = 20;
    perm(&mut g, P2, "Grave Titan"); // the bigger board
    let o = aether_option(&mut g).expect("offered");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    let dead_count = [P1, P2].iter().filter(|&&q| dead(&g, q)).count();
    assert_eq!(dead_count, 1, "one shot, one kill");
    assert!(dead(&g, P2), "the opponent with the board");
}

// ================================================================== Castle Ardenvale
// {2}{W}{W},{T}: create a 1/1 white Human creature token.

#[test]
fn castle_ardenvale_makes_a_human_at_the_end_of_an_opponents_turn_with_the_mana_spare() {
    let mut g = tbl();
    let castle = lands(&mut g, P0, "Castle Ardenvale", 1, false)[0];
    let plains = lands(&mut g, P0, "Plains", 4, false);
    g.active = Some(P1);
    g.step = Step::End;
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.len(), 1, "one option: the Castle's");
    assert!(o[0].utility > 0.0, "a real use, not a negative value");
    assert!(perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let humans: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"human"))
        .collect();
    assert_eq!(humans.len(), 1, "one 1/1 Human");
    assert_eq!(sim_core::engine::values::epow(&g, humans[0]), 1);
    assert!(g.perm(castle).tapped, "the Castle taps for its own cost");
    assert!(plains.iter().all(|&l| g.perm(l).tapped), "{{2}}{{W}}{{W}} paid by the other four lands");
}

#[test]
fn castle_ardenvale_end_of_turn_window_uses_it() {
    let mut g = tbl();
    lands(&mut g, P0, "Castle Ardenvale", 1, false);
    lands(&mut g, P0, "Plains", 4, false);
    g.active = Some(P1);
    g.step = Step::End;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(tokens_of(&g, P0), 1, "the AI made a Human at the end of the opponent's turn");
}

#[test]
fn castle_ardenvale_is_not_used_at_its_own_controllers_end_step() {
    // the AI makes its Human at an opponent's end step only (the card's note), not at its own
    let mut g = tbl();
    lands(&mut g, P0, "Castle Ardenvale", 1, false);
    lands(&mut g, P0, "Plains", 4, false);
    g.active = Some(P0);
    g.step = Step::End;
    assert!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().is_empty());
    g.active = Some(P1);
    assert_eq!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().len(), 1);
}

#[test]
fn castle_ardenvale_is_not_used_without_the_spare_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Castle Ardenvale", 1, false);
    lands(&mut g, P0, "Plains", 3, false); // {2}{W}{W} needs four other sources
    g.active = Some(P1);
    assert!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().is_empty());
}
