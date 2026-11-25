//! Karn, Gilded Guardian ({2/W}{2/U}{2/B}{2/R}{2/G}, Legendary Artifact Creature - Golem 5/5, vigilance, trample): "When Karn
//! enters, draw a card for each color among other artifacts you control." The expected results come from that text and the
//! comprehensive rules (the count is of colours, not of artifacts, and is taken as the trigger resolves, CR 608.2h), not from
//! what the engine did before. Card code: impls/karn_gilded.rs. The {2/X} cost is tested in hybrid_mono.rs too; the record
//! and cost tests here prove the override block's `dsl = []` keeps the {2/X} symbols.

mod play_common;
use play_common::*;
use sim_core::ai::decks::deck_prio_f;
use sim_core::engine::mana::{can_pay_spell, cost_of, pay_spell};
use sim_core::engine::zones::{self, Tokens};
use sim_core::impls::karn_gilded::colours_among_other_artifacts as colours;
use sim_core::state::Ctx;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const KARN: &str = "Karn, Gilded Guardian";
const COURIER: &str = "Transguild Courier";
const GANDALF: &str = "Gandalf the White";
const ELESH: &str = "Elesh Norn, Mother of Machines";

/// P0 plays the arzakon deck (a 99-card library to draw from), P1..P2 have empty boards
fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola", "karn"]);
    g.active = Some(P0);
    g
}

fn hand_size(g: &Game) -> usize {
    g.player(P0).hand.len()
}

/// cards drawn by putting Karn onto the battlefield for P0 (his enter trigger resolves inside `perm`)
fn karn_draws(g: &mut Game) -> usize {
    let before = hand_size(g);
    perm(g, P0, KARN);
    hand_size(g) - before
}

// ------------------------------------------------------------------ the card record
#[test]
fn karn_record_keeps_its_two_hybrid_symbols_under_the_override_block() {
    let g = tbl();
    let d = g.db.get(card(&g, KARN));
    assert_eq!(&*d.hybrid, "W2U2B2R2G2", "the override block states no cost, so the {{2/X}} symbols stay");
    assert_eq!(d.cmc, 10, "five {{2/X}} symbols: mana value 2 each");
    assert_eq!(d.colors.count(), 5);
    assert!(d.types.has(sim_core::cards::Types::ARTIFACT) && d.types.has(sim_core::cards::Types::CREATURE));
    assert_eq!((d.pow, d.tgh), (5, 5));
    assert!(d.kws.iter().any(|k| &**k == "vigilance") && d.kws.iter().any(|k| &**k == "trample"));
    assert!(d.tag(sim_core::tag::Tag::Leg), "legendary");
    // the compiled "draw X" listener is gone: the card code draws, so no second draw comes from data
    assert!(!d.has_dsl(), "dsl = [] in the override block");
}

// ------------------------------------------------------------------ the cost
#[test]
fn castable_from_the_command_zone_with_w_u_b_and_four_colourless_lands() {
    let mut g = tbl();
    set_commander(&mut g, P0, KARN);
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Wastes", 4, false);
    let c = card(&g, KARN);
    let (gn, pips) = cost_of(&g, P0, c);
    let mut p: Vec<char> = pips.chars().collect();
    p.sort();
    assert_eq!(
        (gn, p.into_iter().collect::<String>()),
        (4, "BUW".to_string()),
        "{{2}} for red and for green: seven mana"
    );
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap(), "seven lands pay it");
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "cmd", Ctx::default()).unwrap());
    assert_eq!(perms_named(&g, P0, KARN).len(), 1, "Karn is on the battlefield");
    assert!(!g.player(P0).cmd_in_zone, "he left the command zone");
}

#[test]
fn costs_five_coloured_with_all_five_colours() {
    let mut g = tbl();
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, l, 1, false);
    }
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    let mut p: Vec<char> = pips.chars().collect();
    p.sort();
    assert_eq!((gn, p.into_iter().collect::<String>()), (0, "BGRUW".to_string()));
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false));
}

// ------------------------------------------------------------------ what is counted
#[test]
fn karn_alone_draws_zero() {
    let mut g = tbl();
    assert_eq!(karn_draws(&mut g), 0, "no other artifact: nothing to draw, and it is not an error");
    assert!(!g.over);
}

#[test]
fn colourless_rocks_draw_zero() {
    let mut g = tbl();
    for n in ["Sol Ring", "Arcane Signet", "Thought Vessel"] {
        perm(&mut g, P0, n);
    }
    zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(karn_draws(&mut g), 0, "colourless artifacts have no colour");
}

#[test]
fn courier_alone_draws_five() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 5, "Transguild Courier is all colors");
}

#[test]
fn mind_stone_and_soul_stone_draw_two() {
    let mut g = tbl();
    perm(&mut g, P0, "The Mind Stone"); // {W}
    perm(&mut g, P0, "The Soul Stone"); // {B}
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(karn_draws(&mut g), 2, "white and black");
}

#[test]
fn colours_not_artifacts_are_counted() {
    let mut g = tbl();
    perm(&mut g, P0, "The Mind Stone");
    perm(&mut g, P0, "Archaeomancer's Map"); // another white artifact
    perm(&mut g, P0, "Dragoon's Lance"); // and another
    assert_eq!(karn_draws(&mut g), 1, "three white artifacts are one colour");
}

#[test]
fn a_non_artifact_creature_or_enchantment_is_not_counted() {
    let mut g = tbl();
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Rhystic Study");
    perm(&mut g, P0, "Kitchen Finks");
    assert_eq!(karn_draws(&mut g), 0);
}

#[test]
fn breya_and_thopters_draw_four() {
    // Breya's card code is not on this branch's base (E5), and her compiled enter trigger makes no Thopters when she is put
    // onto the battlefield by the test kit: build the board by hand, as she leaves it (a W/U/B/R artifact creature and her two
    // blue Thopter artifact tokens)
    let mut g = tbl();
    perm(&mut g, P0, "Breya, Etherium Shaper");
    let spec = Tokens {
        types: vec!["thopter", "artifact"],
        color: Some(sim_core::cards::Colors::from_letters("U")),
        fly: true,
        sick: false,
        ..Tokens::new(2, 1)
    };
    let thopters = zones::make_tokens(&mut g, P0, spec).unwrap();
    assert_eq!(
        thopters.len() + g.player(P0).perms.iter().filter(|&&m| &*g.perm(m).name == "Breya, Etherium Shaper").count(),
        3
    );
    assert_eq!(karn_draws(&mut g), 4, "Breya is white, blue, black and red; the blue Thopters add nothing new");
}

#[test]
fn an_opponents_artifacts_are_not_counted() {
    let mut g = tbl();
    perm(&mut g, P1, COURIER);
    assert_eq!(karn_draws(&mut g), 0);
}

#[test]
fn an_artifact_token_counts_by_its_token_colour() {
    let mut g = tbl();
    let spec = Tokens { types: vec!["golem", "artifact"], sick: false, ..Tokens::new(1, 1) };
    zones::make_tokens(&mut g, P0, spec).unwrap();
    assert_eq!(karn_draws(&mut g), 0, "a colourless artifact token");
    let mut g = tbl();
    let spec = Tokens {
        types: vec!["thopter", "artifact"],
        color: Some(sim_core::cards::Colors::from_letters("U")),
        sick: false,
        ..Tokens::new(1, 1)
    };
    zones::make_tokens(&mut g, P0, spec).unwrap();
    assert_eq!(karn_draws(&mut g), 1, "a blue artifact token");
}

#[test]
fn a_leyline_of_the_guildpact_makes_every_artifact_five_colours() {
    let mut g = tbl();
    perm(&mut g, P0, "Leyline of the Guildpact");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(karn_draws(&mut g), 5, "Sol Ring is every colour with Leyline of the Guildpact");
}

#[test]
fn karn_himself_is_not_counted_but_another_karn_is() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KARN);
    assert_eq!(colours(&g, P0, k), 0, "he does not count himself");
    assert_eq!(karn_draws(&mut g), 5, "a second Karn: the first is another artifact of five colours");
}

#[test]
fn a_phased_out_artifact_is_not_counted() {
    let mut g = tbl();
    let c = perm(&mut g, P0, COURIER);
    g.perm_mut(c).phased = true;
    assert_eq!(karn_draws(&mut g), 0);
}

#[test]
fn a_face_down_artifact_is_not_counted() {
    let mut g = tbl();
    let c = perm(&mut g, P0, COURIER);
    g.perm_mut(c).data.set(sim_core::state::DataKey::Facedown, sim_core::state::Val::Bool(true));
    assert_eq!(karn_draws(&mut g), 0, "a face-down permanent is a 2/2 with no types and no colour");
}

// ------------------------------------------------------------------ the trigger uses the stack
#[test]
fn gandalf_and_norn_give_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    perm(&mut g, P0, GANDALF); // a legendary or artifact permanent entering: one more trigger
    perm(&mut g, P0, ELESH); // your own Elesh Norn: one more
    assert!(g.player(P0).library.len() >= 15);
    assert_eq!(karn_draws(&mut g), 15, "three triggers, five colours each");
}

#[test]
fn gandalf_alone_gives_two_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    perm(&mut g, P0, GANDALF);
    assert_eq!(karn_draws(&mut g), 10);
}

#[test]
fn opposing_norn_blanks_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    perm(&mut g, P1, ELESH);
    assert_eq!(karn_draws(&mut g), 0, "an opponent's Elesh Norn: the enter trigger does not trigger");
}

/// a person at `p` that passes priority (or lets `f` answer first); a person gets priority on their own trigger only
/// with autopass "all"
fn person(g: &mut Game, p: PlayerId, f: impl FnMut(&Request) -> Option<Answer> + Send + 'static) -> Arc<ScriptHuman> {
    let mut f = f;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if let Some(a) = f(r) {
            return a;
        }
        if &*r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    }));
    g.humans.add(p, h.clone());
    h.set_autopass("all");
    h
}

#[test]
fn counted_at_resolution() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    let disenchant = hand(&mut g, P0, &["Disenchant"])[0];
    pool(&mut g, P0, &[('W', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let _h = person(&mut g, P0, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        if &*r.kind == "target" {
            return Some(Answer::Pick(r.choices.iter().position(|c| c.contains("Transguild")).unwrap_or(0)));
        }
        None
    });
    let before = hand_size(&g) - 1; // Disenchant leaves the hand
    perm(&mut g, P0, KARN);
    assert!(cast_it.load(Ordering::SeqCst), "the person responded to the trigger");
    assert!(g.player(P0).gy.contains(&disenchant), "Disenchant resolved first");
    assert!(perms_named(&g, P0, COURIER).is_empty(), "the Courier was destroyed in response");
    assert_eq!(hand_size(&g), before, "the trigger counted as it resolved: no artifact left, no draw");
}

#[test]
fn an_opponents_artifact_made_in_response_is_not_counted() {
    let mut g = tbl();
    hand(&mut g, P1, &["Deduce"]);
    pool(&mut g, P1, &[('U', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let _h = person(&mut g, P1, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        None
    });
    // an opponent's Clue is not Karn's controller's artifact: still 0
    assert_eq!(karn_draws(&mut g), 0);
    assert!(cast_it.load(Ordering::SeqCst));
    assert_eq!(g.player(P1).clues, 1);
}

// ------------------------------------------------------------------ copies
#[test]
fn a_clone_of_karn_counts_the_original() {
    // Cursed Mirror cannot copy a creature of its own controller's that is legendary in this engine (the AI skips it, and a
    // person is not offered it), so the copy of the original Karn is a Phyrexian Metamorph ("enters as a copy of an artifact or
    // creature you choose", the engine takes the best one)
    use sim_core::engine::cast::cast_card;
    let mut g = tbl();
    perm(&mut g, P0, KARN);
    let meta = hand(&mut g, P0, &["Phyrexian Metamorph"])[0];
    let before = hand_size(&g) - 1; // the Metamorph leaves the hand
    cast_card(&mut g, P0, meta, "hand", Ctx::default()).unwrap();
    assert_eq!(hand_size(&g), before + 5, "the copy draws for the original Karn: five colours");
    let karns = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == KARN).count();
    assert_eq!(karns, 1, "the legend rule keeps one of the two");
}

#[test]
fn cursed_mirror_copy_of_an_opponents_karn_counts_only_its_own_controllers_artifacts() {
    use sim_core::engine::cast::cast_card;
    let mut g = tbl();
    perm(&mut g, P1, KARN);
    perm(&mut g, P0, COURIER);
    let mirror = hand(&mut g, P0, &["Cursed Mirror"])[0];
    let before = hand_size(&g) - 1;
    cast_card(&mut g, P0, mirror, "hand", Ctx::default()).unwrap();
    let copy = g
        .player(P0)
        .perms
        .iter()
        .any(|&m| g.perm(m).on_bf && g.perm(m).phys == Some(mirror) && &*g.perm(m).name == KARN);
    assert!(copy, "the Mirror became a copy of the opponent's Karn");
    assert_eq!(hand_size(&g), before + 5, "the Courier is five colours; the opponent's Karn is not your artifact");
    assert_eq!(perms_named(&g, P1, KARN).len(), 1, "the opponent keeps his: the legend rule is per controller");
}

// ------------------------------------------------------------------ Ringsight
#[test]
fn ringsight_with_karn_finds_courier() {
    // Karn is the legendary creature you control: his colours are all five, and the Courier is all five
    let mut g = tbl();
    perm(&mut g, P0, KARN);
    g.player_mut(P0).library = vec![card(&g, "Sol Ring"), card(&g, COURIER)];
    let c = take(&mut g, P0, "Ringsight");
    sim_core::engine::cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert!(g.player(P0).hand.contains(&card(&g, COURIER)), "the Courier shares a colour with Karn");
    assert!(!g.player(P0).hand.contains(&card(&g, "Sol Ring")), "a colourless card shares none");
}

// ------------------------------------------------------------------ the AI
#[test]
fn the_ai_values_him_more_with_more_colours_among_its_artifacts() {
    let mut g = tbl();
    g.active = Some(P0);
    g.player_mut(P0).turns = 3;
    let c = hand(&mut g, P0, &[KARN])[0];
    let none = deck_prio_f(&g, P0, c);
    assert!(none > 0.0, "a 5/5 vigilance trample body is worth casting");
    perm(&mut g, P0, "The Mind Stone");
    let one = deck_prio_f(&g, P0, c);
    perm(&mut g, P0, "The Soul Stone");
    let two = deck_prio_f(&g, P0, c);
    perm(&mut g, P0, COURIER);
    let five = deck_prio_f(&g, P0, c);
    assert!(none < one && one < two && two < five, "{none} {one} {two} {five}");
}
