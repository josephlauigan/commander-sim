//! Independent Oracle-based adversarial checks for Batch A. Production code is intentionally unchanged.
use sim_core::ai::act;
use sim_core::engine::{control, hooks, repl, stack, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{common, galadriel, lands as utility_lands, trostani_counters::doctor_resolve};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["trostani", "seph"])
}

fn scourge(g: &mut Game) -> PermId {
    let c = take(g, P0, "Wildwood Scourge");
    zones::enter(g, P0, c, zones::Enter { x: 2, was_cast: true, ..Default::default() }).unwrap()
}

/// the end of the effect that put the counters: what it triggered goes on the stack (CR 603.3)
fn flush(g: &mut Game) {
    stack::flush_triggers(g).unwrap();
}

fn doctor(g: &mut Game) -> PermId {
    g.skip_etb = true;
    let d = perm(g, P0, "The Doctor, EMH");
    g.skip_etb = false;
    d
}

fn attack(g: &mut Game, h: PermId) {
    hooks::run_hook(g, h, Event::Attack, &Call::Attack { p: P0, atk: vec![h], d: P1 }).unwrap();
}

fn gavony(g: &mut Game, others: usize) -> PermId {
    let l = lands(g, P0, "Gavony Township", 1, false)[0];
    lands(g, P0, "Forest", others.div_ceil(2), false);
    lands(g, P0, "Plains", others / 2, false);
    g.active = Some(P1);
    l
}

fn gavony_options(g: &mut Game) -> Vec<Opt> {
    utility_lands::land_options(g, P0, None).unwrap().into_iter().filter(|o| o.label.contains("Gavony")).collect()
}

// Branching cares about the creature's controller, even when an opponent puts the counter on it.
#[test]
fn branching_applies_to_opponents_effect_on_your_creature() {
    let mut g = game();
    perm(&mut g, P0, "Branching Evolution");
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P1, a, 1), 2);
}

// An opponent's Branching cannot double counters on our creatures.
#[test]
fn opponents_branching_does_not_apply() {
    let mut g = game();
    perm(&mut g, P1, "Branching Evolution");
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 3), 3);
}

// A phased-out source is treated as absent (CR 702.26b).
#[test]
fn phased_branching_does_not_apply() {
    let mut g = game();
    let b = perm(&mut g, P0, "Branching Evolution");
    g.perm_mut(b).phased = true;
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 1);
}

// Removing a source's abilities removes its counter replacement.
#[test]
fn disabled_branching_does_not_apply() {
    let mut g = game();
    let b = perm(&mut g, P0, "Branching Evolution");
    g.perm_mut(b).neutered = true;
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 1);
}

// Branching checks "a creature" when counters are placed, not when the permanent entered.
#[test]
fn branching_checks_animation_at_placement_time() {
    let mut g = game();
    perm(&mut g, P0, "Branching Evolution");
    let a = perm(&mut g, P0, "Sol Ring");
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 1);
    g.perm_mut(a).data.set(DataKey::Anim, Val::Bool(true));
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 2);
    assert_eq!(g.perm(a).plus, 3);
}

// Both Mentor and Scales say "one or more". Once Vorinclex rounds a placement down to zero, neither
// can revive it. The best legal order adds both first, then halves: floor((1+1+1)/2) = 1.
#[test]
#[ignore = "T1: pending JD"]
fn replacements_cannot_revive_a_zero_counter_placement() {
    let mut g = game();
    for name in ["Conclave Mentor", "Hardened Scales"] {
        perm(&mut g, P0, name);
    }
    perm(&mut g, P1, "Vorinclex, Monstrous Raider");
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 1);
}

// Mentor says "that many plus one" for EACH copy. CR 616.1 still lets the affected player order seven effects:
// an opponent places 2; five Mentors make 7; that opponent's Vorinclex doubles before ours halves: 7*2/2 = 7.
#[test]
#[ignore = "T2: pending JD"]
fn seven_replacements_still_choose_best_order() {
    let mut g = game();
    for _ in 0..5 {
        perm(&mut g, P0, "Conclave Mentor");
    }
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    perm(&mut g, P1, "Vorinclex, Monstrous Raider");
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P1, a, 2), 7);
}

// "One or more" cannot replace a zero placement, even under Mentor and Branching.
#[test]
fn zero_counter_placement_does_not_grow_scourge() {
    let mut g = game();
    let s = scourge(&mut g);
    perm(&mut g, P0, "Conclave Mentor");
    perm(&mut g, P0, "Branching Evolution");
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 0), 0);
    assert_eq!(g.perm(s).plus, 2);
}

// Solemnity prohibits placement; no replacement may overcome "can't", and Scourge sees no placement.
#[test]
fn solemnity_prevents_crusade_and_scourge_growth() {
    let mut g = game();
    let s = scourge(&mut g);
    perm(&mut g, P0, "Conclave Mentor");
    perm(&mut g, P0, "Branching Evolution");
    perm(&mut g, P1, "Solemnity");
    perm(&mut g, P0, "Cathars' Crusade");
    let a = perm(&mut g, P0, "Serra Angel");
    assert_eq!((g.perm(a).plus, g.perm(s).plus), (0, 2));
}

// Melira prevents -1/-1 counters, but does not prevent +1/+1 counters replaced by Branching.
#[test]
fn melira_and_branching_keep_counter_kinds_separate() {
    let mut g = game();
    perm(&mut g, P0, "Melira, Sylvok Outcast");
    perm(&mut g, P0, "Branching Evolution");
    let a = token(&mut g, P0, 3);
    assert_eq!(repl::put_counters(&mut g, P1, a, -2), 0);
    assert_eq!(repl::put_counters(&mut g, P1, a, 2), 4);
}

// Scourge: "one or more" is one trigger per creature/placement, not per counter.
#[test]
fn scourge_one_trigger_for_large_placement() {
    let mut g = game();
    let s = scourge(&mut g);
    let a = token(&mut g, P0, 2);
    repl::put_counters(&mut g, P0, a, 100);
    flush(&mut g);
    assert_eq!(g.perm(s).plus, 3);
}

// Panharmonicon only duplicates triggers caused by entering; an ordinary counter placement is not entering.
#[test]
fn panharmonicon_does_not_copy_ordinary_scourge_trigger() {
    let mut g = game();
    let s = scourge(&mut g);
    perm(&mut g, P0, "Panharmonicon");
    let a = token(&mut g, P0, 2);
    repl::put_counters(&mut g, P0, a, 1);
    flush(&mut g);
    assert_eq!(g.perm(s).plus, 3);
}

// CR 122.6: entering with counters counts as putting them on. That entry causes Scourge to trigger, so
// Panharmonicon's "an artifact or creature entering causes a triggered ability ..." adds a second trigger.
#[test]
#[ignore = "A7: rules question, pending JD"]
fn panharmonicon_copies_scourge_for_entering_with_counters() {
    let mut g = game();
    let s = scourge(&mut g);
    perm(&mut g, P0, "Panharmonicon");
    let c = take(&mut g, P0, "Hangarback Walker");
    zones::enter(&mut g, P0, c, zones::Enter { x: 1, was_cast: true, ..Default::default() }).unwrap();
    assert_eq!(g.perm(s).plus, 4);
}

// Hushbringer: "Creatures entering ... don't cause abilities to trigger", including Scourge observing
// counters that a creature enters with (CR 122.6). Existing counters on Scourge stay at two.
#[test]
#[ignore = "A5: rules question, pending JD"]
fn hushbringer_stops_scourge_for_entering_with_counters() {
    let mut g = game();
    let s = scourge(&mut g);
    perm(&mut g, P1, "Hushbringer");
    let c = take(&mut g, P0, "Hangarback Walker");
    zones::enter(&mut g, P0, c, zones::Enter { x: 1, was_cast: true, ..Default::default() }).unwrap();
    assert_eq!(g.perm(s).plus, 2);
}

// Undying's +1/+1 counter is also a counter the non-Hydra creature enters with (CR 122.6).
#[test]
fn scourge_sees_undying_entry_counter() {
    let mut g = game();
    let s = scourge(&mut g);
    let c = take(&mut g, P0, "Serra Angel");
    let a = zones::enter(&mut g, P0, c, zones::Enter { undying: true, ..Default::default() }).unwrap();
    assert_eq!(g.perm(a).plus, 1);
    assert_eq!(g.perm(s).plus, 3);
}

// Changeling means EVERY creature type, including Hydra. Scourge's "non-Hydra" excludes it.
#[test]
#[ignore = "T3: pending JD"]
fn scourge_does_not_trigger_for_changeling() {
    let mut g = game();
    let s = scourge(&mut g);
    let a = perm(&mut g, P0, "Changeling Outcast");
    repl::put_counters(&mut g, P0, a, 1);
    assert_eq!(g.perm(s).plus, 2);
}

// A face-down creature has no creature types (CR 708.2), even when its front face is a Hydra.
#[test]
#[ignore = "T4: pending JD"]
fn scourge_triggers_for_face_down_hydra() {
    let mut g = game();
    let s = scourge(&mut g);
    let c = take(&mut g, P0, "Kalonian Hydra");
    let a = galadriel::enter_face_down(&mut g, P0, c).unwrap();
    repl::put_counters(&mut g, P0, a, 1);
    assert_eq!(g.perm(s).plus, 3);
}

// A phased-out Scourge is absent and cannot trigger.
#[test]
fn phased_scourge_does_not_trigger() {
    let mut g = game();
    let s = scourge(&mut g);
    g.perm_mut(s).phased = true;
    let a = token(&mut g, P0, 2);
    repl::put_counters(&mut g, P0, a, 1);
    flush(&mut g);
    assert_eq!(g.perm(s).plus, 2);
}

// Scourge is a "Whenever" triggered ability, not a replacement (CR 603.3, 608.2): while another ability
// resolves, it must wait. This uses the engine's real resolving-depth and trigger-flush mechanism.
#[test]
fn scourge_trigger_waits_until_current_ability_finishes() {
    let mut g = game();
    let s = scourge(&mut g);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    g.resolving += 1;
    repl::put_counters(&mut g, P0, a, 1);
    assert_eq!(g.perm(s).plus, 2, "Scourge's trigger has not resolved yet");
    assert_eq!(g.trig_queue.len(), 1);
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.perm(s).plus, 3);
    // two placement events in one ability: two triggers, both after it
    g.resolving += 1;
    repl::put_counters(&mut g, P0, a, 1);
    repl::put_counters(&mut g, P0, b, 1);
    assert_eq!((g.perm(s).plus, g.trig_queue.len()), (3, 2));
    g.resolving -= 1;
    stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.perm(s).plus, 5);
    assert!(g.trig_queue.is_empty());
}

// Each Scourge is a Hydra, so the two triggers do not recursively trigger one another.
#[test]
fn two_scourges_and_branching_do_not_recurse() {
    let mut g = game();
    let s1 = scourge(&mut g);
    let s2 = scourge(&mut g);
    perm(&mut g, P0, "Branching Evolution");
    let a = token(&mut g, P0, 2);
    repl::put_counters(&mut g, P0, a, 1);
    flush(&mut g);
    assert_eq!((g.perm(s1).plus, g.perm(s2).plus), (4, 4));
}

// Proliferating an existing counter is a placement; each other non-Hydra creature grows Scourge once.
#[test]
fn proliferate_grows_scourge_for_each_non_hydra() {
    let mut g = game();
    let s = scourge(&mut g);
    for _ in 0..2 {
        let a = token(&mut g, P0, 2);
        g.perm_mut(a).plus = 1;
    }
    common::proliferate(&mut g, P0, 1).unwrap();
    flush(&mut g);
    assert_eq!(g.perm(s).plus, 5, "one proliferated counter and two Scourge triggers");
}

// Kalonian doubles +1/+1 counters, not -1/-1 counters, and not counters on phased-out creatures.
#[test]
fn kalonian_excludes_minus_counters_and_phased_creatures() {
    let mut g = game();
    let h = perm(&mut g, P0, "Kalonian Hydra");
    let a = token(&mut g, P0, 5);
    let b = token(&mut g, P0, 2);
    g.perm_mut(a).plus = -2;
    g.perm_mut(b).plus = 3;
    g.perm_mut(b).phased = true;
    attack(&mut g, h);
    assert_eq!((g.perm(a).plus, g.perm(b).plus, g.perm(h).plus), (-2, 3, 8));
}

// Kalonian's trigger reads counters on resolution, after a response has removed them.
#[test]
fn kalonian_reads_current_counters_not_entering_count() {
    let mut g = game();
    let h = perm(&mut g, P0, "Kalonian Hydra");
    g.perm_mut(h).plus = 1; // model three counters removed before the attack trigger resolves
    attack(&mut g, h);
    assert_eq!(g.perm(h).plus, 2);
}

// Mentor's "power" is last known power, including Equipment that detaches when it dies (CR 608.2h).
#[test]
#[ignore = "T5: pending JD"]
fn mentor_dies_uses_equipped_last_known_power() {
    let mut g = game();
    let m = perm(&mut g, P0, "Conclave Mentor");
    let e = perm(&mut g, P0, "Colossus Hammer");
    g.perm_mut(e).attached = Some(m);
    assert_eq!(values::epow(&g, m), 12);
    let before = g.player(P0).life;
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(g.player(P0).life - before, 12);
}

// The Doctor can target himself and then qualifies for the second sentence: two counters.
#[test]
fn doctor_as_only_creature_gets_two() {
    let mut g = game();
    let d = perm(&mut g, P0, "The Doctor, EMH");
    assert_eq!(g.perm(d).plus, 2);
}

// All targets illegal on resolution => neither sentence resolves (CR 608.2b).
#[test]
fn doctor_target_gaining_hexproof_fizzles_everything() {
    let mut g = game();
    let a = token(&mut g, P1, 2);
    let b = token(&mut g, P0, 2);
    g.perm_mut(b).plus = 1;
    let d = doctor(&mut g);
    g.perm_mut(a).eot_kw.push("hexproof");
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 1));
}

// Ward counters the entire targeted ability when unpaid, including the Doctor's second sentence.
#[test]
fn doctor_unpaid_ward_counters_entire_ability() {
    let mut g = game();
    let a = perm(&mut g, P1, "Bronze Guardian");
    let b = token(&mut g, P0, 2);
    g.perm_mut(b).plus = 1;
    let d = doctor(&mut g);
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 1));
}

// Paying the opposing target's ward allows both sentences to resolve, once each.
#[test]
fn doctor_paid_ward_resolves_both_sentences() {
    let mut g = game();
    let a = perm(&mut g, P1, "Bronze Guardian");
    let b = token(&mut g, P0, 2);
    g.perm_mut(b).plus = 1;
    let d = doctor(&mut g);
    lands(&mut g, P0, "Forest", 2, false);
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (1, 2));
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped));
}

// "With a counter" includes a charge counter; the second sentence is not targeted, so shroud is irrelevant.
#[test]
fn doctor_second_sentence_counts_charge_and_ignores_shroud() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    g.perm_mut(b).data.set(DataKey::Charge, Val::Int(1));
    g.perm_mut(b).eot_kw.push("shroud");
    let d = doctor(&mut g);
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (2, 1));
}

// CR 704.7/704.5q: opposite counters cancel as an SBA, not between the Doctor's sentences.
// Starting with a -1/-1 counter, the target still has counters after the first placement; the second adds
// another +1/+1, and only THEN the pair cancels, leaving one +1/+1 counter.
#[test]
fn doctor_target_with_minus_counter_still_gets_second_counter() {
    let mut g = game();
    let a = token(&mut g, P0, 3);
    g.perm_mut(a).plus = -1;
    let d = doctor(&mut g);
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!(g.perm(a).plus, 1);
}

// Once triggered, the ability remains controlled by P0 even if P1 gains the source (CR 113.8).
// P0's hexproof creature is still a legal target of P0's ability, so it receives two counters.
#[test]
fn doctor_source_control_change_does_not_change_target_legality() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    g.perm_mut(a).eot_kw.push("hexproof");
    let d = doctor(&mut g);
    assert!(control::gain_control(&mut g, P1, d, control::Duration::Permanent));
    doctor_resolve(&mut g, d, P0, a).unwrap();
    assert_eq!(g.perm(a).plus, 2);
}

// Gavony's ability does not tap the recipients, so summoning sickness and tapped state do not exclude them.
#[test]
fn gavony_grows_sick_and_tapped_creatures_with_enemy_vorinclex() {
    let mut g = game();
    let l = gavony(&mut g, 4);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    g.perm_mut(a).sick = true;
    g.perm_mut(b).tapped = true;
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P1, "Vorinclex, Monstrous Raider");
    let options = gavony_options(&mut g);
    assert_eq!(options.len(), 1);
    assert!(act::perform(&mut g, P0, options[0].act.as_ref().unwrap()).unwrap());
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (1, 1));
    assert!(g.perm(l).tapped);
}

// A single untapped Gavony cannot supply both {2}{G}{W} and its own tap cost.
#[test]
fn gavony_alone_cannot_activate() {
    let mut g = game();
    gavony(&mut g, 0);
    token(&mut g, P0, 2);
    token(&mut g, P0, 2);
    assert!(gavony_options(&mut g).is_empty());
}

// A phased-out creature is absent from "each creature you control" (CR 702.26b).
#[test]
fn gavony_does_not_grow_phased_creature() {
    let mut g = game();
    gavony(&mut g, 4);
    token(&mut g, P0, 2);
    token(&mut g, P0, 2);
    let a = token(&mut g, P0, 2);
    g.perm_mut(a).phased = true;
    let options = gavony_options(&mut g);
    assert!(act::perform(&mut g, P0, options[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.perm(a).plus, 0);
}

// Crusade's individual placement is halved to zero by an opponent's Vorinclex.
#[test]
fn crusade_with_enemy_vorinclex_places_zero() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    perm(&mut g, P1, "Vorinclex, Monstrous Raider");
    perm(&mut g, P0, "Cathars' Crusade");
    let b = perm(&mut g, P0, "Serra Angel");
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 0));
}

// Crusade has no 999-counter limit: one additional entering creature takes an existing 999 to 1000.
#[test]
fn crusade_can_cross_999_counters() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    g.perm_mut(a).plus = 999;
    perm(&mut g, P0, "Cathars' Crusade");
    perm(&mut g, P0, "Serra Angel");
    assert_eq!(g.perm(a).plus, 1000);
}

// Two requested tokens * Procession * Season = 8 entries; Panharmonicon = 16 triggers;
// each placement under Season * Branching is 4, so EVERY creature gets 64 counters.
#[test]
fn crusade_token_and_trigger_doublers_compose() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    for name in ["Anointed Procession", "Doubling Season", "Branching Evolution", "Panharmonicon", "Cathars' Crusade"] {
        perm(&mut g, P0, name);
    }
    let made = zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(made.len(), 8);
    assert_eq!(g.perm(a).plus, 64);
    for t in made {
        assert_eq!(g.perm(t).plus, 64);
    }
}

// Crusade's "each creature you control" cannot include phased-out creatures (CR 702.26b).
#[test]
fn crusade_nontoken_entry_excludes_phased_creature() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    g.perm_mut(a).phased = true;
    perm(&mut g, P0, "Cathars' Crusade");
    perm(&mut g, P0, "Serra Angel");
    assert_eq!(g.perm(a).plus, 0);
}

// The token-entry Crusade path must also ignore phased-out creatures (CR 702.26b).
#[test]
fn crusade_token_entry_excludes_phased_creature() {
    let mut g = game();
    let a = token(&mut g, P0, 2);
    g.perm_mut(a).phased = true;
    perm(&mut g, P0, "Cathars' Crusade");
    zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 1)).unwrap();
    assert_eq!(g.perm(a).plus, 0);
}

// Oracle's "twice that many" has no numerical cap: 17 independent doublings of ONE counter yield
// 131072, below the per-event resource cap. This deliberately exposes the model's multiplier-cap
// approximation (COUNTER_MULT_CAP=65536); distinguish it from ordinary-board rules bugs in the report.
#[test]
#[ignore = "A14: the 2^16 cap, pending JD"]
fn seventeen_branchings_each_apply_once() {
    let mut g = game();
    for _ in 0..17 {
        perm(&mut g, P0, "Branching Evolution");
    }
    let a = token(&mut g, P0, 2);
    assert_eq!(repl::put_counters(&mut g, P0, a, 1), 131_072);
}
