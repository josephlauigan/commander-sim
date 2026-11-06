//! Veil of Summer as its Oracle text: "Draw a card if an opponent has cast a blue or black spell this turn. Spells you
//! control can't be countered this turn. You and permanents you control gain hexproof from blue and from black until
//! end of turn." Seat 0 is a stand-in deck that is not on the never-answers list (`thrasios`), seat 1 the seph deck.

mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::cast::cast_card;
use sim_core::engine::removal::legal_targets;
use sim_core::engine::values::{player_hexproof_from, veil_shield};
use sim_core::engine::{stack, turn};
use sim_core::ids::CardId;
use sim_core::state::{Ctx, StackItem, StackKind};
use std::sync::Arc;

const VEIL: &str = "Veil of Summer";

fn game() -> Game {
    let mut g = table(&["thrasios", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g.player_mut(P1).hand.clear();
    g
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a spell of p's on the stack (cast: out of the hand)
fn on_stack(g: &mut Game, p: PlayerId, name: &str) -> (u32, CardId) {
    let c = hand(g, p, &[name])[0];
    let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(p).hand.remove(i);
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    g.stack.push(StackItem {
        id,
        controller: p,
        card: Some(c),
        ctx: Ctx::default(),
        zone: "hand",
        imp: 6.0,
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

fn countered(g: &Game, id: u32) -> bool {
    g.stack.iter().find(|x| x.id == id).unwrap().countered
}

/// p casts Veil of Summer from hand with a Forest (nothing on the stack)
fn cast_veil(g: &mut Game, p: PlayerId) {
    lands(g, p, "Forest", 1, false);
    let v = hand(g, p, &[VEIL])[0];
    assert!(cast_card(g, p, v, "hand", Ctx::default()).unwrap());
}

fn drawn_by_veil(g: &mut Game) -> usize {
    cast_veil(g, P0);
    g.player(P0).hand.len()
}

// ------------------------------------------------------------------ every counter that turn fails
#[test]
fn two_counters_in_a_turn_both_fail_after_one_veil() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P1, "Island", 4, false);
    let veil = hand(&mut g, P0, &[VEIL])[0];
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    let c1 = hand(&mut g, P1, &["Counterspell"])[0];
    let c2 = hand(&mut g, P1, &["Counterspell"])[0];
    // the first counterspell is cast; P0 answers it with Veil of Summer, then it resolves and does nothing
    assert!(stack::cast_counter_spell(&mut g, P1, c1, id).unwrap());
    assert!(in_gy(&g, P0, VEIL), "P0 cast Veil of Summer in response");
    assert!(g.player(P0).gy.contains(&veil));
    assert!(veil_shield(&g, P0));
    assert!(!countered(&g, id), "the first counterspell does nothing");
    assert_eq!(g.player(P0).hand.len(), 1, "Counterspell is blue: Veil drew a card");
    // the second one is just as useless, and nothing answers it this time
    assert!(stack::cast_counter_spell(&mut g, P1, c2, id).unwrap());
    assert!(!countered(&g, id), "the second counterspell does nothing either");
    assert!(in_gy(&g, P1, "Counterspell"), "both were spent");
}

#[test]
fn a_red_blast_counter_mode_fails_too() {
    let mut g = game();
    lands(&mut g, P1, "Mountain", 1, false);
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    let blast = hand(&mut g, P1, &["Pyroblast"])[0];
    // without Veil a Pyroblast counters a blue spell ...
    let mut ctrl = g.clone();
    assert!(stack::cast_counter_spell(&mut ctrl, P1, blast, id).unwrap());
    assert!(countered(&ctrl, id), "the control: Pyroblast counters Brainstorm");
    // ... after Veil it does not
    g.stack.clear();
    cast_veil(&mut g, P0);
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    assert!(stack::cast_counter_spell(&mut g, P1, blast, id).unwrap());
    assert!(!countered(&g, id), "Pyroblast's counter mode does nothing");
}

#[test]
fn mindbreak_trap_still_exiles_a_spell_under_veil() {
    let mut g = game();
    lands(&mut g, P1, "Island", 4, false);
    cast_veil(&mut g, P0);
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    let trap = hand(&mut g, P1, &["Mindbreak Trap"])[0];
    let it = g.stack.iter().find(|x| x.id == id).unwrap();
    assert!(!stack::counterable(&g, it), "Veil: it can't be countered");
    assert!(stack::counterable_but_veil(&g, it), "but nothing else stops it");
    assert_eq!(stack::pick_counter_only(&g, P1, g.stack[0].card.unwrap(), Some("Mindbreak Trap")), Some(trap));
    assert!(stack::cast_counter_spell(&mut g, P1, trap, id).unwrap());
    let it = g.stack.iter().find(|x| x.id == id).unwrap();
    assert!(it.countered, "Mindbreak Trap exiles it: Veil of Summer does not stop that");
    assert_eq!(it.countered_by, Some(g.db.id("Mindbreak Trap").unwrap()));
}

#[test]
fn without_veil_a_counterspell_counters() {
    let mut g = game();
    lands(&mut g, P1, "Island", 2, false);
    let (id, _) = on_stack(&mut g, P0, "Brainstorm"); // P0 holds no Veil
    let c = hand(&mut g, P1, &["Counterspell"])[0];
    assert!(stack::cast_counter_spell(&mut g, P1, c, id).unwrap());
    assert!(countered(&g, id));
}

// ------------------------------------------------------------------ the draw
#[test]
fn veil_draws_only_after_an_opponents_blue_or_black_spell() {
    let st = |g: &Game| g.turn_stamp();
    for (spell, draws) in
        [(None, false), (Some("Lightning Bolt"), false), (Some("Counterspell"), true), (Some("Murder"), true)]
    {
        let mut g = game();
        library_cards(&mut g, P0, 3);
        if let Some(s) = spell {
            let c = card(&g, s);
            let stamp = st(&g);
            g.player_mut(P1).turn_casts = Some((stamp, vec![c]));
        }
        let before = g.player(P0).hand.len();
        let after = drawn_by_veil(&mut g);
        assert_eq!(after, before + usize::from(draws), "{spell:?}: draws {draws}");
    }
    // the Veil player's own blue spell does not count, and neither does a blue spell from an earlier turn
    let mut g = game();
    library_cards(&mut g, P0, 3);
    let c = card(&g, "Brainstorm");
    let stamp = g.turn_stamp();
    g.player_mut(P0).turn_casts = Some((stamp, vec![c]));
    assert_eq!(drawn_by_veil(&mut g), 0, "own blue spell");
    let mut g = game();
    library_cards(&mut g, P0, 3);
    let mut old = g.turn_stamp();
    old.round = 0;
    g.player_mut(P1).turn_casts = Some((old, vec![card(&g, "Counterspell")]));
    assert_eq!(drawn_by_veil(&mut g), 0, "last turn's blue spell");
}

fn library_cards(g: &mut Game, p: PlayerId, n: usize) {
    assert!(g.player(p).library.len() >= n);
}

// ------------------------------------------------------------------ hexproof from blue and from black
#[test]
fn blue_and_black_targeted_removal_cannot_target_the_veil_players_creatures_but_red_can() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Mulldrifter");
    let land = lands(&mut g, P0, "Forest", 1, false)[0];
    let murder = card(&g, "Murder");
    let bolt = card(&g, "Lightning Bolt");
    let unsummon = card(&g, "Unsummon");
    assert!(legal_targets(&g, P1, "destroy", "c", false, Some(murder)).contains(&mine), "before Veil");
    cast_veil(&mut g, P0);
    let late = perm(&mut g, P0, "Grave Titan"); // enters after Veil resolved: no hexproof
    let black = legal_targets(&g, P1, "destroy", "c", false, Some(murder));
    assert!(!black.contains(&mine), "hexproof from black");
    assert!(black.contains(&late), "a permanent that enters later this turn has none");
    let red = legal_targets(&g, P1, "dmg3", "c", false, Some(bolt));
    assert!(red.contains(&mine), "red damage can still target it");
    let blue = legal_targets(&g, P1, "bounce", "c", false, Some(unsummon));
    assert!(!blue.contains(&mine), "hexproof from blue");
    assert!(sim_core::impls::tymna_cards::hexproof_from(&g, land, Colors::from_letters("U")), "lands too");
    // the player: blue and black spells can't target them, red ones can
    assert!(player_hexproof_from(&g, P0, g.db.get(murder).colors));
    assert!(player_hexproof_from(&g, P0, g.db.get(unsummon).colors));
    assert!(!player_hexproof_from(&g, P0, g.db.get(bolt).colors));
    assert!(!player_hexproof_from(&g, P1, g.db.get(murder).colors), "the opponents have no hexproof");
}

#[test]
fn a_burn_spell_to_the_face_needs_to_be_red_under_veil() {
    let mut g = game();
    cast_veil(&mut g, P0);
    let c = card(&g, "Lightning Bolt");
    let ctx = sim_core::engine::cast::spell_targets(&g, P1, c, None);
    assert_eq!(ctx.face, Some(P0), "red burn can still go to the face");
    let murder = card(&g, "Murder");
    let ctx = sim_core::engine::cast::spell_targets(&g, P1, murder, None);
    assert_eq!(ctx.face, None);
}

// ------------------------------------------------------------------ until end of turn
#[test]
fn the_effect_ends_at_the_end_of_the_turn() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Mulldrifter");
    cast_veil(&mut g, P0);
    let murder = card(&g, "Murder");
    assert!(veil_shield(&g, P0));
    assert!(sim_core::impls::tymna_cards::hexproof_from(&g, mine, Colors::from_letters("B")));
    // P1's turn begins
    g.active = Some(P1);
    turn::step_start(&mut g, P1).unwrap();
    assert!(!veil_shield(&g, P0), "spells can be countered again");
    assert!(!sim_core::impls::tymna_cards::hexproof_from(&g, mine, Colors::from_letters("B")), "no hexproof");
    assert!(legal_targets(&g, P1, "destroy", "c", false, Some(murder)).contains(&mine));
    assert!(!player_hexproof_from(&g, P0, g.db.get(murder).colors));
    lands(&mut g, P1, "Island", 2, false);
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    let c = hand(&mut g, P1, &["Counterspell"])[0];
    assert!(stack::cast_counter_spell(&mut g, P1, c, id).unwrap());
    assert!(countered(&g, id), "a counterspell works again");
}

// ------------------------------------------------------------------ the person
#[test]
fn a_person_casts_their_own_veil_and_the_state_applies_the_same() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Mulldrifter");
    lands(&mut g, P1, "Island", 2, false);
    pool(&mut g, P0, &[('G', 1)]);
    g.player_mut(P0).hand.clear(); // (Mulldrifter drew two)
    hand(&mut g, P0, &[VEIL]);
    // at priority: cast the card in hand (Veil, hand index 0), then pass
    let first = std::sync::Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "cast", "card": 0}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    let (id, _) = on_stack(&mut g, P0, "Brainstorm");
    let c = hand(&mut g, P1, &["Counterspell"])[0];
    assert!(stack::cast_counter_spell(&mut g, P1, c, id).unwrap());
    if !in_gy(&g, P0, VEIL) {
        for r in h.asked() {
            eprintln!("ASK {}: {} {:?}", r.kind, r.prompt, r.choices);
        }
        for t in h.told() {
            eprintln!("TOLD {}: {}", t.0, t.1);
        }
    }
    assert!(in_gy(&g, P0, VEIL), "the person cast Veil of Summer in response");
    assert!(veil_shield(&g, P0));
    assert!(!countered(&g, id), "the counterspell does nothing");
    assert!(sim_core::impls::tymna_cards::hexproof_from(&g, mine, Colors::from_letters("U")));
    assert!(in_exile(&g, P1, "Counterspell") || in_gy(&g, P1, "Counterspell"));
}
