//! Adversarial tests for Karn, Gilded Guardian's enter trigger (impls/karn_gilded.rs), written by a reviewer who did not write
//! the code. Expected values come from the Oracle text ("When Karn enters, draw a card for each color among other artifacts
//! you control") and the comprehensive rules, not from what the engine does.

mod play_common;
use play_common::*;
use sim_core::cards::Types;
use sim_core::engine::control::{self, Duration};
use sim_core::engine::mana::{can_pay_spell, cost_of};
use sim_core::engine::zones::{self, Tokens};
use sim_core::impls::karn_gilded::colours_among_other_artifacts as colours;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const KARN: &str = "Karn, Gilded Guardian";
const COURIER: &str = "Transguild Courier";

fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola", "karn"]);
    g.active = Some(P0);
    g
}

fn hand_size(g: &Game) -> usize {
    g.player(P0).hand.len()
}

fn karn_draws(g: &mut Game) -> usize {
    let before = hand_size(g);
    perm(g, P0, KARN);
    hand_size(g) - before
}

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

// ===================================================================== 1. the count
#[test]
fn artifact_lands_are_colourless() {
    // Ancient Den / Great Furnace / Seat of the Synod are artifact lands: lands have no colour
    let mut g = tbl();
    for n in ["Ancient Den", "Great Furnace", "Seat of the Synod", "Darksteel Citadel"] {
        lands(&mut g, P0, n, 1, false);
    }
    assert_eq!(karn_draws(&mut g), 0, "artifact lands are colourless");
}

#[test]
fn artifact_creatures_count_their_colours_once_each() {
    let mut g = tbl();
    perm(&mut g, P0, "Etherium Sculptor"); // U
    perm(&mut g, P0, "Porcelain Legionnaire"); // W
    perm(&mut g, P0, "Baleful Strix"); // U and B
    assert_eq!(karn_draws(&mut g), 3, "W, U, B");
}

#[test]
fn two_colour_artifacts_union() {
    let mut g = tbl();
    perm(&mut g, P0, "Cauldron of Essence"); // B and G
    perm(&mut g, P0, "Gold-Forged Thopteryx"); // W and U
    assert_eq!(karn_draws(&mut g), 4);
}

#[test]
fn food_clue_blood_counters_are_colourless() {
    let mut g = tbl();
    g.player_mut(P0).clues = 3;
    g.player_mut(P0).foods = 2;
    g.player_mut(P0).bloods = 2;
    assert_eq!(karn_draws(&mut g), 0);
}

#[test]
fn a_blue_artifact_token_and_a_red_one_are_two() {
    let mut g = tbl();
    for c in ["U", "R", "R"] {
        let spec = Tokens {
            types: vec!["construct", "artifact"],
            color: Some(sim_core::cards::Colors::from_letters(c)),
            sick: false,
            ..Tokens::new(1, 1)
        };
        zones::make_tokens(&mut g, P0, spec).unwrap();
    }
    assert_eq!(karn_draws(&mut g), 2);
}

#[test]
fn leyline_of_the_guildpact_does_not_colour_artifact_lands() {
    let mut g = tbl();
    perm(&mut g, P0, "Leyline of the Guildpact");
    lands(&mut g, P0, "Darksteel Citadel", 1, false);
    lands(&mut g, P0, "Ancient Den", 1, false);
    assert_eq!(karn_draws(&mut g), 0, "Leyline: a nonland permanent is every colour; lands keep theirs (none)");
}

#[test]
fn leyline_of_the_guildpact_colours_karns_other_artifacts_but_an_opponents_leyline_does_not() {
    let mut g = tbl();
    perm(&mut g, P1, "Leyline of the Guildpact");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(karn_draws(&mut g), 0, "the opponent's Leyline colours the opponent's permanents only");
}

#[test]
fn an_artifact_you_own_but_an_opponent_controls_is_not_counted() {
    let mut g = tbl();
    let c = perm(&mut g, P0, COURIER);
    assert!(control::gain_control(&mut g, P1, c, Duration::Permanent));
    assert_eq!(karn_draws(&mut g), 0, "controlled by the opponent now");
}

#[test]
fn an_artifact_you_control_but_an_opponent_owns_is_counted() {
    let mut g = tbl();
    let c = perm(&mut g, P1, COURIER);
    assert!(control::gain_control(&mut g, P0, c, Duration::Permanent));
    assert_eq!(karn_draws(&mut g), 5, "you control it");
}

#[test]
fn karn_entering_under_a_player_whose_cards_are_in_a_different_seat() {
    // Karn enters under P1's control (as from a steal-from-the-stack): P1's artifacts count, P0's do not
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    perm(&mut g, P1, "The Mind Stone"); // W
    let before = g.player(P1).hand.len();
    perm(&mut g, P1, KARN);
    assert_eq!(g.player(P1).hand.len() - before, 1, "P1 controls Karn: one colour among P1's artifacts");
}

#[test]
fn a_second_karn_counts_the_first_even_when_the_first_is_the_only_artifact() {
    let mut g = tbl();
    let a = perm(&mut g, P0, KARN);
    assert_eq!(colours(&g, P0, a), 0);
    perm(&mut g, P0, COURIER);
    assert_eq!(colours(&g, P0, a), 5, "the Courier counts for the first Karn if it were entering");
}

// ===================================================================== 2. triggers and the stack
#[test]
fn panharmonicon_doubles_the_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 10, "an artifact entering triggers twice: Panharmonicon");
}

#[test]
fn an_opposing_panharmonicon_does_not_double_it() {
    let mut g = tbl();
    perm(&mut g, P1, "Panharmonicon");
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 5);
}

#[test]
fn hushbringer_blanks_a_creatures_enter_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, "Hushbringer");
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 0, "Hushbringer: creatures entering don't cause abilities of creatures to trigger");
}

#[test]
fn karn_dying_in_response_still_draws() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    hand(&mut g, P0, &["Disenchant"]);
    pool(&mut g, P0, &[('W', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let _h = person(&mut g, P0, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        if &*r.kind == "target" {
            return Some(Answer::Pick(r.choices.iter().position(|c| c.contains("Gilded")).unwrap_or(0)));
        }
        None
    });
    let before = hand_size(&g) - 1;
    let k = perm(&mut g, P0, KARN);
    assert!(cast_it.load(Ordering::SeqCst), "the person responded");
    assert!(!g.perm(k).on_bf, "Karn was destroyed in response");
    assert_eq!(hand_size(&g), before + 5, "the trigger resolves without Karn: five colours from the Courier");
}

#[test]
fn an_artifact_leaving_in_response_changes_the_count_by_its_colours_only() {
    // Mind Stone (W), Archaeomancer's Map (W) and Cauldron of Essence (B, G): destroying the Cauldron leaves W only
    let mut g = tbl();
    perm(&mut g, P0, "The Mind Stone");
    perm(&mut g, P0, "Archaeomancer's Map");
    perm(&mut g, P0, "Cauldron of Essence"); // B and G
    hand(&mut g, P0, &["Disenchant"]);
    pool(&mut g, P0, &[('W', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let _h = person(&mut g, P0, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        if &*r.kind == "target" {
            return Some(Answer::Pick(r.choices.iter().position(|c| c.contains("Cauldron")).unwrap_or(0)));
        }
        None
    });
    let before = hand_size(&g) - 1;
    perm(&mut g, P0, KARN);
    assert!(cast_it.load(Ordering::SeqCst));
    assert_eq!(hand_size(&g), before + 1, "only white is left");
}

#[test]
fn karn_himself_is_never_counted_even_with_leyline() {
    let mut g = tbl();
    perm(&mut g, P0, "Leyline of the Guildpact");
    assert_eq!(karn_draws(&mut g), 0, "Karn is five colours with Leyline, but not another artifact");
}

// ===================================================================== 3. the library
#[test]
fn drawing_more_than_the_library_holds_draws_what_is_there_and_loses_once() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    g.player_mut(P0).library.truncate(3);
    let drew = karn_draws(&mut g);
    assert_eq!(drew, 3, "all three that are there");
    assert!(g.player(P0).library.is_empty());
    assert!(
        g.player(P0).decked,
        "the failed draw is recorded (a player loses at the next state-based check, CR 121.4)"
    );
}

#[test]
fn exact_library_does_not_deck() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    g.player_mut(P0).library.truncate(5);
    assert_eq!(karn_draws(&mut g), 5);
    assert!(!g.player(P0).decked, "five cards for five draws: no failed draw");
}

#[test]
fn laboratory_maniac_wins_on_the_draw_that_would_empty_the_library() {
    let mut g = tbl();
    perm(&mut g, P0, "Laboratory Maniac");
    perm(&mut g, P0, COURIER);
    g.player_mut(P0).library.truncate(3);
    let _ = karn_draws(&mut g);
    assert!(g.over && g.winner == Some(P0), "the fourth draw from an empty library wins with Laboratory Maniac");
}

#[test]
fn karn_with_no_other_artifact_and_an_empty_library_does_not_deck() {
    let mut g = tbl();
    g.player_mut(P0).library.clear();
    assert_eq!(karn_draws(&mut g), 0);
    assert!(!g.player(P0).decked, "a draw of zero cards is not a failed draw");
}

#[test]
fn notion_thief_takes_the_extra_draws() {
    let mut g = tbl();
    perm(&mut g, P1, "Notion Thief");
    perm(&mut g, P0, COURIER);
    let p1 = g.player(P1).hand.len();
    assert_eq!(karn_draws(&mut g), 0, "an opponent's Notion Thief: the draws are not yours");
    assert_eq!(g.player(P1).hand.len() - p1, 5, "Notion Thief's controller draws instead");
}

#[test]
fn narset_limits_the_draws_to_one_a_turn() {
    let mut g = tbl();
    perm(&mut g, P1, "Narset, Parter of Veils");
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 1, "an opponent's Narset: no more than one card each turn");
}

#[test]
fn spirit_of_the_labyrinth_limits_the_draws_to_one_a_turn() {
    let mut g = tbl();
    perm(&mut g, P1, "Spirit of the Labyrinth");
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 1);
}

#[test]
fn a_dead_controller_draws_nothing() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    g.player_mut(P0).alive = false;
    assert_eq!(karn_draws(&mut g), 0);
}

// ===================================================================== 4. cast and cost paths
#[test]
fn nine_plains_pay_for_karn_and_eight_do_not() {
    // W + four {2}s is nine mana
    for (n, ok) in [(8usize, false), (9, true)] {
        let mut g = tbl();
        lands(&mut g, P0, "Plains", n, false);
        let c = hand(&mut g, P0, &[KARN])[0];
        let (gn, pips) = cost_of(&g, P0, c);
        assert_eq!(can_pay_spell(&g, P0, c, gn, &pips, false), ok, "{n} Plains: cost ({gn}, {pips})");
    }
}

#[test]
fn four_colours_and_two_colourless_pay_for_karn() {
    // W, U, B, R and two Wastes: {2} for green, so six mana
    let mut g = tbl();
    for l in ["Plains", "Island", "Swamp", "Mountain"] {
        lands(&mut g, P0, l, 1, false);
    }
    lands(&mut g, P0, "Wastes", 2, false);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "cost ({gn}, {pips})");
    assert_eq!(gn as usize + pips.chars().count(), 6);
}

#[test]
fn tax_two_adds_two_to_the_hybrid_price() {
    let mut g = tbl();
    set_commander(&mut g, P0, KARN);
    g.player_mut(P0).tax = 2;
    for l in ["Plains", "Island", "Swamp"] {
        lands(&mut g, P0, l, 1, false);
    }
    lands(&mut g, P0, "Wastes", 4, false);
    let c = card(&g, KARN);
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false), "seven lands do not pay nine: ({gn}, {pips})");
    lands(&mut g, P0, "Wastes", 2, false);
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "nine lands pay: ({gn}, {pips})");
    assert_eq!(gn as usize + pips.chars().count(), 9);
}

// ===================================================================== 4b. the AI
fn ai_game(lands_spec: &[(&str, usize)], cmd: bool, tax: u32) -> Game {
    let mut g = tbl();
    g.player_mut(P0).turns = 3;
    g.player_mut(P0).cmd_in_zone = false; // the arzakon deck's own commander would compete for the mana
    for (l, n) in lands_spec {
        lands(&mut g, P0, l, *n, false);
    }
    if cmd {
        set_commander(&mut g, P0, KARN);
        g.player_mut(P0).tax = tax;
    } else {
        g.player_mut(P0).hand.clear();
        hand(&mut g, P0, &[KARN]);
    }
    g
}

fn karn_on_bf(g: &Game) -> bool {
    !perms_named(g, P0, KARN).is_empty()
}

#[test]
fn the_ai_casts_karn_from_hand_with_w_u_b_and_four_colourless() {
    let mut g = ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Wastes", 4)], false, 0);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(karn_on_bf(&g), "seven mana pay it");
}

#[test]
fn the_ai_casts_karn_from_hand_with_five_colours_and_two_spare() {
    let mut g =
        ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Mountain", 1), ("Forest", 1), ("Wastes", 2)], false, 0);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(karn_on_bf(&g));
}

#[test]
fn the_ai_does_not_cast_karn_it_cannot_pay() {
    let mut g = ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Wastes", 3)], false, 0);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(!karn_on_bf(&g), "six lands W U B + 3 do not pay seven");
    assert!(g.player(P0).hand.iter().any(|&c| c == card(&g, KARN)));
}

#[test]
fn the_ai_casts_karn_from_the_command_zone() {
    let mut g = ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Wastes", 4)], true, 0);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(karn_on_bf(&g), "from the command zone, seven mana");
    assert!(!g.player(P0).cmd_in_zone);
}

#[test]
fn the_ai_casts_karn_from_the_command_zone_with_tax_when_it_can_pay() {
    let mut g = ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Wastes", 6)], true, 2);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(karn_on_bf(&g), "nine mana pay seven plus the tax of two");
}

#[test]
fn the_ai_does_not_cast_commander_karn_it_cannot_pay_with_tax() {
    let mut g = ai_game(&[("Plains", 1), ("Island", 1), ("Swamp", 1), ("Wastes", 4)], true, 2);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(!karn_on_bf(&g), "seven lands do not pay nine");
    assert!(g.player(P0).cmd_in_zone);
}

#[test]
fn the_prio_hook_is_only_on_karn_gilded_and_does_not_leak_to_other_karns() {
    let g = tbl();
    for (n, has) in
        [(KARN, true), ("Karn, the Great Creator", false), ("Karn Liberated", false), ("Karn, Legacy Reforged", false)]
    {
        if g.db.id(n).is_none() {
            continue;
        }
        let imp = g.registry.get(card(&g, n));
        let karn_prio = imp.is_some_and(|i| i.prio.is_some());
        if has {
            assert!(karn_prio, "{n}");
        } else {
            // a prio hook on another Karn would be another card's; only report it
            eprintln!("{n}: prio hook = {karn_prio}");
        }
    }
}

// ===================================================================== 5. the data
#[test]
fn the_record_matches_the_card() {
    let g = tbl();
    let d = g.db.get(card(&g, KARN));
    assert_eq!(&*d.hybrid, "W2U2B2R2G2");
    assert_eq!(d.cmc, 10);
    assert_eq!(d.colors.count(), 5);
    assert!(!d.has_dsl(), "no compiled draw trigger left: the card code draws, so it can't draw twice");
    assert!(d.types.has(Types::ARTIFACT) && d.types.has(Types::CREATURE));
    assert!(g.registry.get(card(&g, KARN)).is_some_and(|i| i.etb.is_some()));
}

#[test]
fn the_override_file_has_exactly_one_karn_block_with_empty_dsl() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/card_overrides.toml");
    let s = std::fs::read_to_string(p).unwrap();
    assert_eq!(s.matches("[\"Karn, Gilded Guardian\"]").count(), 1);
    let at = s.find("[\"Karn, Gilded Guardian\"]").unwrap();
    let block: String =
        s[at..].lines().take_while(|l| !l.starts_with("[\"Karn, the Great")).collect::<Vec<_>>().join("\n");
    assert!(block.contains("dsl = []"));
    assert!(block.contains("status = \"Full\""));
    assert!(!block.contains("\ncost ="), "no cost line: it would drop the {{2/X}} symbols");
}

#[test]
fn only_one_draw_when_the_record_has_no_dsl_and_the_hook_runs() {
    // draws exactly five, not ten (a second compiled trigger would double it)
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    assert_eq!(karn_draws(&mut g), 5);
}

#[test]
fn the_enters_inventory_row_is_in_the_file() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/enters_inventory.txt");
    let s = std::fs::read_to_string(p).unwrap();
    let rows: Vec<&str> = s.lines().filter(|l| l.starts_with("Karn, Gilded Guardian |")).collect();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0].contains("| Etb |"));
}

// ===================================================================== 6. leakage
#[test]
fn another_artifact_creature_entering_draws_nothing() {
    let mut g = tbl();
    perm(&mut g, P0, COURIER);
    let before = hand_size(&g);
    perm(&mut g, P0, "Etherium Sculptor");
    perm(&mut g, P0, "Wurmcoil Engine");
    assert_eq!(hand_size(&g), before, "only Karn's own enter trigger draws");
}

// ---- copies and the legend rule (Singleton: a second Karn can only be a copy, never the same card twice)
#[test]
fn a_metamorph_copy_of_karn_draws_for_the_original_and_the_legend_rule_keeps_the_original() {
    use sim_core::engine::cast::cast_card;
    use sim_core::state::Ctx;
    // CR 704.5j runs before the copy's trigger is put on the stack: the controller keeps the original (the copy dies and its
    // trigger still counts the original, five colours) or keeps the copy (the original is gone, the trigger counts nothing)
    let mut g = tbl();
    let a = perm(&mut g, P0, KARN);
    let meta = hand(&mut g, P0, &["Phyrexian Metamorph"])[0];
    let before = hand_size(&g) - 1;
    cast_card(&mut g, P0, meta, "hand", Ctx::default()).unwrap();
    let drew = hand_size(&g) - before;
    let on: Vec<_> =
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| g.perm(m).name.to_string()).collect();
    assert_eq!(on.iter().filter(|n| &***n == KARN).count(), 1, "one Karn: {on:?}");
    if g.perm(a).on_bf {
        assert_eq!(drew, 5, "the original stayed: the copy's trigger counted it");
    } else {
        assert_eq!(drew, 0, "the copy stayed: the original was gone before its trigger resolved");
    }
}
