//! Tier 6 batch `counters` (impls/t6_counters.rs) on hand-built positions. Seat 0 plays a stand-in deck (the cards are
//! keyed by name), seat 1 the seph deck.

use sim_core::engine::cast;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t6_counters as t6;
use sim_core::state::{Ctx, Game, StackItem, StackKind};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
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

/// a spell of p's on the stack (cast: out of the hand)
fn on_stack(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> (u32, CardId) {
    let c = hand(g, p, &[name])[0];
    let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(p).hand.remove(i);
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    g.stack.push(StackItem {
        id,
        controller: p,
        card: Some(c),
        ctx,
        zone: "hand",
        imp: 5.0,
        aff: vec![],
        generic: true,
        kind: StackKind::Spell,
        name: name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    (id, c)
}

fn item(g: &Game, id: u32) -> &StackItem {
    g.stack.iter().find(|x| x.id == id).unwrap()
}

// ======================================================== Dizzy Spell
#[test]
fn dizzy_spell_gives_minus_three_power_until_end_of_turn() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Dizzy Spell"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert_eq!(g.perm(t).eot_pt, (-3, 0));
    assert!(in_gy(&g, P0, "Dizzy Spell"));
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

#[test]
fn dizzy_spell_transmutes_into_a_one_drop_at_sorcery_speed() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Dizzy Spell"])[0];
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    // not at the end-of-turn window
    assert!(f(&mut g, c, P0, None).unwrap().is_empty());
    // not on another player's turn
    g.active = Some(P1);
    assert!(f(&mut g, c, P0, Some(true)).unwrap().is_empty());
    g.active = Some(P0);
    // the library must hold a mana value 1 card
    let one = take(&mut g, P0, "Brainstorm");
    g.player_mut(P0).library.push(one);
    let opts = f(&mut g, c, P0, Some(true)).unwrap();
    assert_eq!(opts.len(), 1);
    let before = g.player(P0).hand.len();
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(in_gy(&g, P0, "Dizzy Spell"));
    assert_eq!(g.player(P0).hand.len(), before, "Dizzy Spell out, one card in");
    assert!(g.player(P0).hand.iter().any(|&x| g.db.get(x).cmc == 1 && !g.db.get(x).land));
}

#[test]
fn dizzy_spell_transmute_needs_the_mana() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Dizzy Spell"])[0];
    let one = take(&mut g, P0, "Brainstorm");
    g.player_mut(P0).library.push(one);
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    assert!(f(&mut g, c, P0, Some(true)).unwrap().is_empty());
}

// ======================================================== Into the Flood Maw
#[test]
fn flood_maw_bounces_a_creature_without_a_gift() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let before = g.player(P1).perms.len();
    let c = hand(&mut g, P0, &["Into the Flood Maw"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert!(!g.perm(t).on_bf);
    assert!(in_hand(&g, P1, "Grave Titan"));
    assert!(g.player(P1).perms.len() < before, "no Fish: the creature simply left");
    assert!(on_bf(&g, P1, "Fish").is_none());
}

#[test]
fn flood_maw_gift_mode_bounces_a_nonland_permanent_and_gives_a_tapped_fish() {
    let mut g = game();
    let t = perm(&mut g, P1, "Sol Ring");
    let c = hand(&mut g, P0, &["Into the Flood Maw"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert!(!g.perm(t).on_bf);
    assert!(in_hand(&g, P1, "Sol Ring"));
    let fish = g.player(P1).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).expect("a Fish");
    assert!(g.perm(fish).tapped);
    assert_eq!(sim_core::engine::values::epow(&g, fish), 1);
}

// ======================================================== Abeyance
#[test]
fn abeyance_draws_and_flags_the_opponent() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    let c = hand(&mut g, P0, &["Abeyance"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1, "Abeyance out, a card drawn");
    assert!(t6::abeyance_flag(&g, P1));
    assert!(!t6::abeyance_flag(&g, P0));
    g.active = Some(P1);
    assert!(!t6::abeyance_flag(&g, P1), "only this turn");
}

// ======================================================== alternative costs
#[test]
fn daze_returns_an_island() {
    let mut g = game();
    let d = hand(&mut g, P0, &["Daze"])[0];
    assert!(!t6::alt_cost_ok(&g, P0, d, None));
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(!t6::alt_cost_ok(&g, P0, d, None), "a Swamp is no Island");
    lands(&mut g, P0, "Island", 2, false);
    assert!(t6::alt_cost_ok(&g, P0, d, None));
    assert!(t6::pay_alt_cost(&mut g, P0, d, None).unwrap());
    assert_eq!(t6::islands(&g, P0).len(), 1);
    assert!(in_hand(&g, P0, "Island"));
}

#[test]
fn flare_of_denial_sacrifices_a_nontoken_blue_creature() {
    let mut g = game();
    let f = hand(&mut g, P0, &["Flare of Denial"])[0];
    token(&mut g, P0, 2);
    assert!(!t6::alt_cost_ok(&g, P0, f, None), "a token (and not blue) doesn't count");
    perm(&mut g, P0, "Grave Titan");
    assert!(!t6::alt_cost_ok(&g, P0, f, None), "a black creature doesn't count");
    let m = perm(&mut g, P0, "Mulldrifter");
    assert!(t6::alt_cost_ok(&g, P0, f, None));
    assert!(t6::pay_alt_cost(&mut g, P0, f, None).unwrap());
    assert!(!g.perm(m).on_bf);
    assert!(in_gy(&g, P0, "Mulldrifter"));
}

#[test]
fn blue_pitch_cards_exile_the_least_useful_blue_card() {
    let mut g = game();
    let s = hand(&mut g, P0, &["Snapback"])[0];
    assert!(!t6::alt_cost_ok(&g, P0, s, None), "the Snapback itself is not the blue card");
    hand(&mut g, P0, &["Lightning Bolt"]);
    assert!(!t6::alt_cost_ok(&g, P0, s, None), "a red card is not blue");
    hand(&mut g, P0, &["Counterspell"]);
    assert!(t6::alt_cost_ok(&g, P0, s, None));
    assert!(t6::pay_alt_cost(&mut g, P0, s, None).unwrap());
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Counterspell"));
    assert!(in_hand(&g, P0, "Lightning Bolt"));
}

#[test]
fn commandeer_and_misdirection_pitch_costs() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Commandeer"])[0];
    let m = hand(&mut g, P0, &["Misdirection"])[0];
    hand(&mut g, P0, &["Counterspell"]);
    assert!(t6::alt_cost_ok(&g, P0, m, None), "Counterspell pitches to Misdirection");
    // Commandeer: two blue cards besides itself (Misdirection and Counterspell)
    assert!(t6::alt_cost_ok(&g, P0, c, None));
    assert!(t6::pay_alt_cost(&mut g, P0, c, None).unwrap());
    assert_eq!(g.player(P0).exile.len(), 2);
    assert!(!t6::alt_cost_ok(&g, P0, c, None), "nothing left to pitch");
}

#[test]
fn disrupting_shoal_pitches_a_blue_card_with_the_spells_mana_value() {
    let mut g = game();
    let s = hand(&mut g, P0, &["Disrupting Shoal"])[0];
    hand(&mut g, P0, &["Counterspell"]); // mana value 2
    let (_, bolt) = on_stack(&mut g, P1, "Lightning Bolt", Ctx::default()); // mana value 1
    let (_, golem) = on_stack(&mut g, P1, "Phantasmal Image", Ctx::default()); // mana value 2
    assert!(!t6::alt_cost_ok(&g, P0, s, Some(bolt)));
    assert!(t6::alt_cost_ok(&g, P0, s, Some(golem)));
    assert!(t6::shoal_counters(&g, golem, 2));
    assert!(!t6::shoal_counters(&g, bolt, 2));
    assert!(t6::pay_alt_cost(&mut g, P0, s, Some(golem)).unwrap());
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Counterspell"));
}

#[test]
fn mindbreak_trap_is_free_after_three_opposing_spells() {
    let mut g = game();
    let t = hand(&mut g, P0, &["Mindbreak Trap"])[0];
    assert!(!t6::alt_cost_ok(&g, P0, t, None));
    let cs = vec![card(&g, "Brainstorm"), card(&g, "Brainstorm"), card(&g, "Counterspell")];
    let st = g.turn_stamp();
    g.player_mut(P1).turn_casts = Some((st, cs[..2].to_vec()));
    assert!(!t6::alt_cost_ok(&g, P0, t, None), "two spells are not enough");
    g.player_mut(P1).turn_casts = Some((st, cs));
    assert!(t6::alt_cost_ok(&g, P0, t, None));
    assert!(t6::pay_alt_cost(&mut g, P0, t, None).unwrap());
}

// ======================================================== spells on the stack
#[test]
fn misdirection_sends_a_bolt_back_at_the_caster() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Grave Titan");
    let theirs = perm(&mut g, P1, "Mulldrifter");
    let (id, _) = on_stack(&mut g, P1, "Lightning Bolt", Ctx { target: Some(mine), ..Ctx::default() });
    assert!(t6::misdirect(&mut g, P0, id).unwrap());
    assert_eq!(item(&g, id).ctx.target, Some(theirs));
    // aimed at someone else's permanent: nothing to do
    assert!(!t6::misdirect(&mut g, P0, id).unwrap());
}

#[test]
fn misdirection_redirects_a_spell_aimed_at_our_face_and_ignores_untargeted_spells() {
    let mut g = game();
    let (id, _) = on_stack(&mut g, P1, "Lightning Bolt", Ctx { face: Some(P0), ..Ctx::default() });
    assert!(t6::misdirect(&mut g, P0, id).unwrap());
    assert_eq!(item(&g, id).ctx.face, Some(P1));
    let (id2, _) = on_stack(&mut g, P1, "Brainstorm", Ctx::default());
    assert!(!t6::misdirect(&mut g, P0, id2).unwrap(), "no target");
}

#[test]
fn commandeer_takes_a_noncreature_spell_but_not_a_creature_spell() {
    let mut g = game();
    let (id, _) = on_stack(&mut g, P1, "Rhystic Study", Ctx::default());
    assert!(t6::commandeer(&mut g, P0, id).unwrap());
    assert_eq!(item(&g, id).controller, P0);
    assert_eq!(item(&g, id).ctx.owner, Some(P1), "the card still goes back to its owner");
    let (id2, _) = on_stack(&mut g, P1, "Grave Titan", Ctx::default());
    assert!(!t6::commandeer(&mut g, P0, id2).unwrap());
    assert_eq!(item(&g, id2).controller, P1);
}

#[test]
fn sudden_substitution_exchanges_a_spell_and_a_creature() {
    let mut g = game();
    let titan = perm(&mut g, P0, "Grave Titan");
    let (id, _) = on_stack(&mut g, P1, "Rhystic Study", Ctx::default());
    assert!(t6::substitute(&mut g, id, titan).unwrap());
    assert_eq!(item(&g, id).controller, P0, "the creature's controller gets the spell");
    assert_eq!(g.perm(titan).owner, P1, "the spell's controller gets the creature");
    assert!(g.player(P1).perms.contains(&titan));
    assert!(!g.player(P0).perms.contains(&titan));
}

#[test]
fn bilbo_gambit_returns_a_spell_to_hand_and_the_gift_locks_spells() {
    let mut g = game();
    let (id, _) = on_stack(&mut g, P1, "Brainstorm", Ctx::default());
    assert!(t6::bilbo_bounce(&mut g, P0, id, false).unwrap());
    assert!(item(&g, id).countered);
    assert!(in_hand(&g, P1, "Brainstorm"));
    assert!(!t6::lock_flag(&g, P0), "no gift, no lock");
    let (id2, _) = on_stack(&mut g, P1, "Counterspell", Ctx::default());
    assert!(t6::bilbo_bounce(&mut g, P0, id2, true).unwrap());
    assert!(t6::lock_flag(&g, P0));
}

#[test]
fn mindbreak_trap_exiles_every_opposing_spell_on_the_stack() {
    let mut g = game();
    let (a, _) = on_stack(&mut g, P1, "Brainstorm", Ctx::default());
    let (b, _) = on_stack(&mut g, P1, "Counterspell", Ctx::default());
    let (mine, _) = on_stack(&mut g, P0, "Lightning Bolt", Ctx::default());
    assert_eq!(t6::mindbreak_exile(&mut g, P0).unwrap(), 2);
    for id in [a, b] {
        assert!(item(&g, id).countered && item(&g, id).ctx.exile_after);
    }
    assert!(!item(&g, mine).countered);
}

// ======================================================== Tibalt's Trickery
#[test]
fn tibalts_trickery_mills_then_casts_the_first_nonland_card_free() {
    let mut g = game();
    let swamp = card(&g, "Swamp");
    let ring = take(&mut g, P1, "Sol Ring");
    let titan = take(&mut g, P1, "Grave Titan");
    // top is last: four Swamps, then Sol Ring, then Grave Titan
    g.player_mut(P1).library = vec![titan, ring, swamp, swamp, swamp, swamp];
    let spell = card(&g, "Counterspell");
    let cast = t6::tibalt_after(&mut g, P0, P1, spell).unwrap();
    assert_eq!(cast, Some(ring));
    assert!(on_bf(&g, P1, "Sol Ring").is_some());
    // the milled cards are in the graveyard, the exiled lands went to the bottom with Grave Titan still on top
    assert!((1..=3).contains(&g.player(P1).gy.len()));
    assert_eq!(*g.player(P1).library.last().unwrap(), titan);
}

#[test]
fn tibalts_trickery_skips_a_card_named_like_the_countered_spell() {
    let mut g = game();
    let swamp = card(&g, "Swamp");
    let ring = take(&mut g, P1, "Sol Ring");
    let bolt = take(&mut g, P1, "Lightning Bolt");
    g.player_mut(P1).library = vec![bolt, ring, ring, swamp, swamp, swamp];
    // Sol Ring was countered: the other Sol Ring is skipped, the first different nonland card is cast
    let cast = t6::tibalt_after(&mut g, P0, P1, ring).unwrap();
    assert_eq!(cast, Some(bolt));
}
