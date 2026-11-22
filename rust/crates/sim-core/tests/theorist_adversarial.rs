//! Oracle-based adversarial checks. Confirmed defects remain executable with `--ignored`.
mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::cards::Colors;
use sim_core::engine::{combat, control, life, turn, values, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::impls::common::theorist_draws;
use sim_core::state::{DataKey, Step, Val};
use std::sync::Arc;

const THEORIST: &str = "The Theorist, Jace Beleren";
const P3: PlayerId = PlayerId(3);

fn position(n: usize) -> Game {
    let mut g = table(&["yshtola", "karn", "cloud", "sheoldred"][..n]);
    let c = card(&g, "Counterspell");
    for q in &mut g.players {
        // Known nonlands prevent step_start's automatic land play from obscuring draw counts.
        q.library = vec![c; 20];
        q.hand.clear();
    }
    g
}

fn options(g: &mut Game, w: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(w).cd.unwrap()).unwrap().options.unwrap();
    f(g, w, P0, Some(false)).unwrap()
}

fn ai_use(g: &mut Game, w: PermId, delta: &str) {
    let a = options(g, w)
        .into_iter()
        .find(|o| o.label.ends_with(delta))
        .unwrap_or_else(|| panic!("expected an AI {delta} option"))
        .act
        .unwrap();
    assert!(perform(g, P0, &a).unwrap());
}

fn human_use(g: &mut Game, w: PermId, delta: &str) -> Option<String> {
    let (_, ability) = abilities_of(g, P0, w).unwrap().into_iter().find(|a| a.0.starts_with(delta)).unwrap();
    match ability {
        AbilityKind::Extra(f) => f(g, P0, w).unwrap(),
        _ => panic!("expected the practice-mode card implementation"),
    }
}

#[test]
fn controller_is_active_has_only_the_normal_draw_at_two_and_four_seats() {
    for n in [2, 4] {
        let mut g = position(n);
        perm(&mut g, P0, THEORIST);
        turn::step_start(&mut g, P0).unwrap();
        assert_eq!(g.player(P0).hand.len(), 1);
    }
}

#[test]
fn each_opponent_including_consecutive_extra_turns_triggers_at_two_and_four_seats() {
    for n in [2, 4] {
        let mut g = position(n);
        perm(&mut g, P0, THEORIST);
        let turns: Vec<_> = (1..n).map(|i| PlayerId(i as u8)).chain([P1, P1]).collect();
        for (i, p) in turns.into_iter().enumerate() {
            turn::step_start(&mut g, p).unwrap();
            assert_eq!(g.player(P0).hand.len(), i + 1);
        }
    }
}

#[test]
fn two_copies_one_phased_and_one_stolen_draw_for_the_current_controller() {
    let mut g = position(4);
    let phased = perm(&mut g, P0, THEORIST);
    g.perm_mut(phased).phased = true;
    let stolen = perm(&mut g, P1, THEORIST);
    assert!(control::gain_control(&mut g, P2, stolen, control::Duration::Permanent));
    turn::step_start(&mut g, P3).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(P2).hand.len()), (0, 0, 1));
}

#[test]
fn dead_controller_and_dead_seats_do_not_draw_or_panic() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P2, THEORIST);
    life::eliminate(&mut g, P0);
    life::eliminate(&mut g, P1);
    turn::step_start(&mut g, P3).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P2).hand.len()), (0, 1));
}

#[test]
fn controller_eliminated_by_draw_punisher_does_not_block_other_controllers() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, THEORIST);
    perm(&mut g, P2, "Sheoldred, the Apocalypse");
    g.player_mut(P0).life = 2;
    turn::step_start(&mut g, P3).unwrap();
    assert!(!g.player(P0).alive);
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn controller_empty_library_loses_and_other_trigger_still_resolves() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, THEORIST);
    g.player_mut(P0).library.clear();
    turn::step_start(&mut g, P3).unwrap();
    assert!(g.player(P0).decked && !g.player(P0).alive);
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn necropotence_skips_the_step_so_theorist_does_not_trigger() {
    let mut g = position(3);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Necropotence");
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P1).hand.len(), 0);
    assert_eq!(g.player(P0).hand.len(), 0, "CR 614.10a: a skipped step has no beginning");
}

#[test]
fn a_replaced_active_draw_does_not_suppress_theorist() {
    let mut g = position(3);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Uba Mask");
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).exile.len(), 1, "Theorist still triggers; Uba Mask replaces that draw too");
    assert_eq!(g.player(P1).impulse.len(), 1, "active-player exiled cards are represented as impulse cards");
    assert_eq!(values::hand_count(&g, P1), 0);
}

#[test]
fn active_player_decks_before_theorist_can_resolve_in_two_player_game() {
    let mut g = position(2);
    perm(&mut g, P0, THEORIST);
    g.player_mut(P1).library.clear();
    turn::step_start(&mut g, P1).unwrap();
    assert!(!g.player(P1).alive && g.over);
    assert_eq!(g.player(P0).hand.len(), 0, "CR 504.1: normal draw and state checks precede trigger resolution");
}

#[test]
fn notion_thief_replaces_theorist_draw_but_not_the_active_normal_draw() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P2, "Notion Thief");
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(P2).hand.len()), (0, 1, 1));
}

#[test]
fn narset_denies_a_second_theorist_draw_in_the_same_turn() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P2, "Narset, Parter of Veils");
    g.active = Some(P1);
    zones::draw(&mut g, P0, 1, false).unwrap();
    theorist_draws(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
    assert_eq!(g.player(P0).stats.get("narset_denied"), Some(&1));
}

#[test]
fn howling_mine_and_theorist_draw_normally_and_rhystic_does_not_trigger() {
    let mut g = position(4);
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P2, "Howling Mine");
    perm(&mut g, P3, "Rhystic Study");
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(P3).hand.len()), (1, 2, 0));
}

#[test]
fn normal_draw_precedes_theorist_draw_stolen_by_the_active_player() {
    let mut g = position(3);
    // Bowmasters must enter before the 1-toughness Thief, or its enters trigger kills the Thief.
    perm(&mut g, P2, "Orcish Bowmasters");
    perm(&mut g, P0, THEORIST);
    perm(&mut g, P0, "Narset, Parter of Veils");
    let thief = perm(&mut g, P1, "Notion Thief");
    assert!(g.perm(thief).on_bf);
    let life = g.player(P1).life;
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P1).hand.len(), 1);
    assert_eq!(g.player(P1).life, life, "normal draw is first and exempt from Bowmasters; stolen extra is denied");
}

#[test]
fn minus_two_at_exactly_two_loyalty_resolves_after_source_dies() {
    for human in [false, true] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(2);
        let target = perm(&mut g, P1, "Grave Titan");
        if human {
            seat(&mut g, P0, vec![by_text("Grave Titan")]);
            assert_eq!(human_use(&mut g, w, "-2"), None);
        } else {
            ai_use(&mut g, w, "-2");
        }
        assert!(!g.perm(w).on_bf && !g.perm(target).on_bf);
        assert!(g.player(P0).gy.contains(&card(&g, THEORIST)));
        assert!(g.player(P1).hand.contains(&card(&g, "Grave Titan")));
    }
}

#[test]
fn minus_two_cannot_target_protection_from_blue_ai() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let protected = perm(&mut g, P1, "Serra Angel");
    g.perm_mut(protected).eot_kw.push("pro_U".into());
    let other = perm(&mut g, P2, "Grave Titan");
    assert!(values::protected_from(&g, protected, Colors::from_letters("U")));
    ai_use(&mut g, w, "-2");
    assert!(!g.perm(other).on_bf);
    assert!(g.perm(protected).on_bf, "a blue source cannot target protection from blue");
}

#[test]
fn minus_two_cannot_target_protection_from_blue_human() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let protected = perm(&mut g, P1, "Serra Angel");
    g.perm_mut(protected).eot_kw.push("pro_U".into());
    seat(&mut g, P0, vec![by_text("Serra Angel")]);
    human_use(&mut g, w, "-2");
    assert!(g.perm(protected).on_bf, "practice mode must not offer this illegal target");
}

#[test]
fn minus_two_skips_hexproof_shroud_and_phased_targets() {
    for keyword in ["hexproof", "shroud", "phased"] {
        for human in [false, true] {
            let mut g = position(3);
            let w = perm(&mut g, P0, THEORIST);
            let protected = perm(&mut g, P1, "Serra Angel");
            if keyword == "phased" {
                g.perm_mut(protected).phased = true;
            } else {
                g.perm_mut(protected).eot_kw.push(keyword.into());
            }
            let target = perm(&mut g, P2, "Grave Titan");
            assert!(values::untargetable(&g, protected));
            if human {
                seat(&mut g, P0, vec![by_text("Grave Titan")]);
                human_use(&mut g, w, "-2");
            } else {
                ai_use(&mut g, w, "-2");
            }
            assert!(g.perm(protected).on_bf);
            assert!(!g.perm(target).on_bf);
        }
    }
}

#[test]
#[ignore = "ENGINE LIMIT: ward is paid per target inside engine/removal.rs apply_removal (an unpaid ward skips only that target and returns success), and the AI picks the warded permanent although 'up to one' would let it skip it; shared removal/ward code, not the Theorist"]
fn unpaid_ward_counters_entire_multi_target_ability_ai() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let warded = perm(&mut g, P1, "Sauron, the Dark Lord");
    let other = perm(&mut g, P2, "Grave Titan");
    // The only legend P0 controls is a noncreature, nonartifact planeswalker: it cannot pay Sauron's ward.
    ai_use(&mut g, w, "-2");
    assert!(g.perm(warded).on_bf);
    assert!(g.perm(other).on_bf, "ward counters the whole ability, including its other targets");
    assert_eq!(g.perm(w).loyalty, Some(1));
}

#[test]
#[ignore = "ENGINE LIMIT: ward is paid inside engine/removal.rs apply_removal per target, which returns success after an unpaid ward, so the practice -2 (human/cards.rs, a per-target loop) cannot counter the whole ability; shared removal/ward code, not the Theorist"]
fn unpaid_ward_counters_entire_multi_target_ability_human() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let warded = perm(&mut g, P1, "Sauron, the Dark Lord");
    let other = perm(&mut g, P2, "Grave Titan");
    seat(&mut g, P0, vec![by_text("Sauron"), by_text("Grave Titan")]);
    human_use(&mut g, w, "-2");
    assert!(g.perm(warded).on_bf);
    assert!(g.perm(other).on_bf, "practice mode must counter the whole ability too");
}

#[test]
#[ignore = "ENGINE LIMIT: ward is paid per target inside engine/removal.rs apply_removal (an unpaid ward skips only that target and returns success), and the AI picks the warded permanent although 'up to one' would let it skip it; shared removal/ward code, not the Theorist"]
fn unpaid_ward_on_later_target_prevents_earlier_bounce() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let first = perm(&mut g, P1, "Grave Titan");
    let warded = perm(&mut g, P2, "Sauron, the Dark Lord");
    ai_use(&mut g, w, "-2");
    assert!(g.perm(warded).on_bf);
    assert!(g.perm(first).on_bf, "ward resolves before any part of the targeted ability");
}

#[test]
fn minus_two_bounces_indestructible_artifact_creature_once_and_token_ceases() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let artifact = perm(&mut g, P1, "Steel Hellkite");
    g.perm_mut(artifact).eot_kw.push("indestructible".into());
    let t = token(&mut g, P2, 8);
    assert!(values::indestructible(&g, artifact));
    ai_use(&mut g, w, "-2");
    assert!(!g.perm(artifact).on_bf && !g.perm(t).on_bf);
    assert_eq!(g.player(P1).hand.iter().filter(|&&c| c == card(&g, "Steel Hellkite")).count(), 1);
    assert!(g.player(P2).hand.is_empty());
}

#[test]
fn human_minus_two_can_choose_an_artifact_land() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let land = lands(&mut g, P1, "Darksteel Citadel", 1, false)[0];
    assert!(values::is_artifact_perm(&g, land));
    seat(&mut g, P0, vec![by_text("Darksteel Citadel")]);
    human_use(&mut g, w, "-2");
    assert!(!g.perm(land).on_bf, "artifact lands are artifacts and legal targets");
    assert!(g.player(P1).hand.contains(&card(&g, "Darksteel Citadel")));
}

#[test]
fn bounced_stolen_permanent_goes_to_its_owner_and_commander_can_use_command_zone() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let stolen = perm(&mut g, P0, "Grave Titan");
    control::gain_control(&mut g, P2, stolen, control::Duration::Permanent);
    // Karn uses the command-zone replacement; Cloud deliberately prefers keeping its commander in hand.
    let cmd = g.player(P1).cmd;
    g.player_mut(P1).set_cmd_zone(cmd, false);
    let commander = zones::enter(&mut g, P1, cmd, zones::Enter::default()).unwrap();
    ai_use(&mut g, w, "-2");
    assert!(!g.perm(stolen).on_bf && !g.perm(commander).on_bf);
    assert!(g.player(P0).hand.contains(&card(&g, "Grave Titan")));
    assert!(!g.player(P2).hand.contains(&card(&g, "Grave Titan")));
    assert!(g.player(P1).cmd_zone_of(cmd));
    assert!(!g.player(P1).hand.contains(&cmd));
}

#[test]
fn human_minus_two_may_choose_zero_targets_and_still_pay() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    perm(&mut g, P1, "Serra Angel");
    seat(&mut g, P0, vec![cancel()]);
    assert_eq!(human_use(&mut g, w, "-2"), None);
    assert_eq!(g.perm(w).loyalty, Some(1));
    assert!(has_perm(&g, P1, "Serra Angel"));
}

#[test]
fn ultimate_with_zero_or_one_creature_is_legal_in_practice() {
    for n in [0, 1] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(7);
        let c = (n == 1).then(|| token(&mut g, P0, 1));
        hand(&mut g, P0, &["Brainstorm", "Counterspell"]);
        assert_eq!(human_use(&mut g, w, "-6"), None);
        assert_eq!(g.player(P0).hand.len(), 5);
        assert_eq!(g.perm(w).loyalty, Some(1));
        if let Some(c) = c {
            assert_eq!(g.perm(c).plus, 5);
        }
    }
}

#[test]
fn ultimate_ai_and_human_match_post_draw_x_phasing_and_counter_replacements() {
    for human in [false, true] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(6);
        let a = token(&mut g, P0, 1);
        let b = token(&mut g, P0, 1);
        let phased = token(&mut g, P0, 1);
        g.perm_mut(phased).phased = true;
        let other = token(&mut g, P1, 1);
        perm(&mut g, P0, "Hardened Scales");
        perm(&mut g, P0, "Doubling Season");
        hand(&mut g, P0, &["Brainstorm", "Counterspell"]);
        if human {
            human_use(&mut g, w, "-6");
        } else {
            ai_use(&mut g, w, "-6");
        }
        assert_eq!(g.player(P0).hand.len(), 5);
        assert_eq!((g.perm(a).plus, g.perm(b).plus), (12, 12), "(post-draw hand 5 + Scales 1) doubled");
        assert_eq!((g.perm(phased).plus, g.perm(other).plus), (0, 0));
        assert!(!g.perm(w).on_bf);
    }
}

#[test]
fn ultimate_empty_library_mid_draw_uses_cards_actually_drawn_then_state_checks() {
    for human in [false, true] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(7);
        let a = token(&mut g, P0, 1);
        token(&mut g, P0, 1);
        hand(&mut g, P0, &["Brainstorm"]);
        g.player_mut(P0).library.truncate(1);
        if human {
            human_use(&mut g, w, "-6");
        } else {
            ai_use(&mut g, w, "-6");
        }
        assert_eq!(g.perm(a).plus, 2, "finish the resolving ability before deck-out state actions");
        life::check_state(&mut g).unwrap();
        assert!(g.player(P0).decked && !g.player(P0).alive);
    }
}

fn check_ultimate_draw_triggers(human: bool) {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(7);
    let a = token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    perm(&mut g, P1, "Sheoldred, the Apocalypse");
    g.player_mut(P0).life = 2;
    if human {
        human_use(&mut g, w, "-6");
    } else {
        ai_use(&mut g, w, "-6");
    }
    assert_eq!(g.perm(a).plus, 3, "draw triggers wait until the ultimate finishes drawing and placing counters");
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive);
}

#[test]
fn ultimate_finishes_before_sheoldred_draw_triggers_ai() {
    check_ultimate_draw_triggers(false);
}

#[test]
fn ultimate_finishes_before_sheoldred_draw_triggers_human() {
    check_ultimate_draw_triggers(true);
}

#[test]
fn ultimate_x_uses_actual_hand_after_notion_thief_replaces_all_draws() {
    for human in [false, true] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(7);
        let a = token(&mut g, P0, 1);
        token(&mut g, P0, 1);
        perm(&mut g, P1, "Notion Thief");
        hand(&mut g, P0, &["Brainstorm"]);
        if human {
            human_use(&mut g, w, "-6");
        } else {
            ai_use(&mut g, w, "-6");
        }
        assert_eq!(g.player(P0).hand.len(), 1);
        assert_eq!(g.player(P1).hand.len(), 3);
        assert_eq!(g.perm(a).plus, 1);
    }
}

#[test]
fn plus_one_doubles_tokens_but_doubling_season_does_not_double_loyalty_cost() {
    for human in [false, true] {
        let mut g = position(3);
        perm(&mut g, P0, "Doubling Season");
        let w = perm(&mut g, P0, THEORIST);
        assert_eq!(g.perm(w).loyalty, Some(6));
        if human {
            human_use(&mut g, w, "+1");
        } else {
            ai_use(&mut g, w, "+1");
        }
        assert_eq!(g.perm(w).loyalty, Some(7));
        let ts: Vec<_> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
        assert_eq!(ts.len(), 2);
        for t in ts {
            assert_eq!(values::colors_of(&g, t), Colors::from_letters("U"));
            assert!(values::has_type(&g, t, "illusion"));
            assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (1, 1));
        }
    }
}

#[test]
fn ai_never_offers_unaffordable_minus_abilities() {
    for loyalty in 1..=7 {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        g.perm_mut(w).loyalty = Some(loyalty);
        token(&mut g, P0, 1);
        token(&mut g, P0, 1);
        perm(&mut g, P1, "Grave Titan");
        for o in options(&mut g, w) {
            let delta: i32 = o.label.split_whitespace().last().unwrap().parse().unwrap();
            assert!(loyalty + delta >= 0, "{} at {loyalty} loyalty", o.label);
        }
    }
}

#[test]
fn once_per_turn_without_oath_agrees_for_ai_and_human() {
    for human in [false, true] {
        let mut g = position(3);
        let w = perm(&mut g, P0, THEORIST);
        if human {
            assert_eq!(human_use(&mut g, w, "+1"), None);
        } else {
            ai_use(&mut g, w, "+1");
        }
        assert_eq!(g.perm(w).loyalty, Some(4));
        if human {
            assert!(human_use(&mut g, w, "+1").is_some());
        } else {
            assert!(options(&mut g, w).is_empty());
        }
    }
}

#[test]
fn oath_allows_exactly_two_ai_activations() {
    let mut g = position(3);
    perm(&mut g, P0, "Oath of Teferi");
    let w = perm(&mut g, P0, THEORIST);
    ai_use(&mut g, w, "+1");
    ai_use(&mut g, w, "+1");
    assert_eq!(g.perm(w).loyalty, Some(5));
    assert!(options(&mut g, w).is_empty());
}

#[test]
#[ignore = "ENGINE LIMIT: practice loyalty uses are the binary pw_use stamp (human/cards.rs, impls/mine.rs), not the Oath-aware impls/common.rs uses/allowed count; shared practice walker code, not the Theorist"]
fn oath_allows_exactly_two_human_activations() {
    let mut g = position(3);
    perm(&mut g, P0, "Oath of Teferi");
    let w = perm(&mut g, P0, THEORIST);
    assert_eq!(human_use(&mut g, w, "+1"), None);
    assert_eq!(human_use(&mut g, w, "+1"), None);
    assert_eq!(g.perm(w).loyalty, Some(5));
    assert!(human_use(&mut g, w, "+1").is_some());
}

fn check_extra_turn_loyalty(human: bool) {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    if human {
        assert_eq!(human_use(&mut g, w, "+1"), None);
    } else {
        ai_use(&mut g, w, "+1");
    }
    // Extra turns run step_start without incrementing the enclosing round (turn::run_extra_turns).
    turn::step_start(&mut g, P0).unwrap();
    g.step = Step::Main1;
    if human {
        assert_eq!(human_use(&mut g, w, "+1"), None, "a new turn permits a new activation");
    } else {
        assert!(options(&mut g, w).iter().any(|o| o.label.ends_with("+1")), "new turn, same round");
    }
}

#[test]
#[ignore = "ENGINE LIMIT: loyalty uses are stamped by round and seat (impls/common.rs round_stamp/uses), so a consecutive extra turn of the same seat is not a new turn; shared walker code, not the Theorist"]
fn loyalty_ability_refreshes_on_consecutive_extra_turns_ai() {
    check_extra_turn_loyalty(false);
}

#[test]
#[ignore = "ENGINE LIMIT: loyalty uses are stamped by round and seat (impls/mine.rs pw_use stamp), so a consecutive extra turn of the same seat is not a new turn; shared walker code, not the Theorist"]
fn loyalty_ability_refreshes_on_consecutive_extra_turns_human() {
    check_extra_turn_loyalty(true);
}

#[test]
fn tidebinder_removing_abilities_suppresses_future_draw_trigger() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(7);
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    lands(&mut g, P1, "Island", 3, false);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    g.perm_mut(w).neutered = true; // Tishana's Tidebinder countered an ability and removed all of its abilities
    assert!(g.perm(w).on_bf && g.perm(w).neutered);
    assert!(g.player(P0).hand.is_empty());
    turn::step_start(&mut g, P2).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0, "a permanent with no abilities has no draw trigger");
}

#[test]
fn draw_trigger_offers_priority_on_the_stack() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Action(json!({"do": "pass"}))));
    h.set_autopass("all");
    g.humans.add(P2, h.clone());
    // Positive control: this observer sees the same card's activated ability on the stack.
    ai_use(&mut g, w, "+1");
    assert!(h.asked().iter().any(|r| r.prompt.contains(THEORIST)));
    let before = h.asked().len();
    turn::step_start(&mut g, P1).unwrap();
    assert!(
        h.asked()[before..].iter().any(|r| r.prompt.contains(THEORIST)),
        "draw trigger must offer a response window"
    );
}

fn check_animated_land_counters(human: bool) {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    g.perm_mut(w).loyalty = Some(7);
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    let land = lands(&mut g, P0, "Darksteel Citadel", 1, false)[0];
    // Layer-4 animation, the same representation used by artifact animation effects.
    let x = g.perm_mut(land);
    x.data.set(DataKey::Anim, Val::Bool(true));
    (x.pow, x.tgh) = (3, 3);
    assert!(g.is_creature(land));
    if human {
        human_use(&mut g, w, "-6");
    } else {
        ai_use(&mut g, w, "-6");
    }
    assert_eq!(g.perm(land).plus, 3, "each creature includes lands that are creatures");
}

#[test]
fn ultimate_puts_counters_on_animated_lands_ai() {
    check_animated_land_counters(false);
}

#[test]
fn ultimate_puts_counters_on_animated_lands_human() {
    check_animated_land_counters(true);
}

fn check_fixed_bounce_targets(human: bool) {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let primary = perm(&mut g, P1, "Grave Titan");
    let backup = perm(&mut g, P1, "Serra Angel");
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    let h = seat(&mut g, P1, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan"), pass()]);
    if human {
        seat(&mut g, P0, vec![by_text("Grave Titan"), pass()]);
        human_use(&mut g, w, "-2");
    } else {
        ai_use(&mut g, w, "-2");
    }
    assert!(h.told_kind("invalid").is_empty(), "{:?}", h.told());
    assert!(g.player(P1).exile.contains(&card(&g, "Grave Titan")), "the response removed the original target");
    assert!(!g.perm(primary).on_bf);
    assert!(g.perm(backup).on_bf, "targets are fixed before responses; cannot choose a replacement at resolution");
}

#[test]
fn ai_minus_two_does_not_retarget_after_a_response_removes_its_target() {
    check_fixed_bounce_targets(false);
}

#[test]
fn human_minus_two_does_not_retarget_after_a_response_removes_its_target() {
    check_fixed_bounce_targets(true);
}

#[test]
fn combat_damage_removes_loyalty_and_kills_theorist_without_hurting_controller() {
    let mut g = position(3);
    let w = perm(&mut g, P0, THEORIST);
    let attacker = token(&mut g, P1, 3);
    g.active = Some(P1);
    let life = g.player(P0).life;
    combat::resolve_combat(&mut g, P1, &[attacker], P0, &[]).unwrap();
    assert!(!g.perm(w).on_bf, "three unblocked damage kills a three-loyalty walker");
    assert_eq!(g.player(P0).life, life);
    assert!(g.player(P0).gy.contains(&card(&g, THEORIST)));
}
