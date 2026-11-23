//! JD's `urdragon` deck: the cards modeled for it (Kindred Dominance in impls/sweepers.rs, Kaldra Compleat's living
//! weapon in impls/karn_engines.rs and its compiled bonus), and the seat's plan file.

use sim_core::engine::removal::apply_wipe;
use sim_core::engine::values::{epow, etgh, indestructible};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"])
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

const SILUMGAR: &str = "Dragonlord Silumgar";

#[test]
fn silumgar_steals_the_best_creature_until_it_leaves() {
    let mut g = tbl();
    let small = perm(&mut g, P1, "Llanowar Elves");
    let big = perm(&mut g, P1, "Blightsteel Colossus");
    let src = perm(&mut g, P0, SILUMGAR);
    assert_eq!(g.perm(big).owner, P0);
    assert_eq!(g.perm(small).owner, P1);
    assert!(g.perm(big).sick, "changing control causes summoning sickness");
    assert!(sim_core::dsl::has_kw(&g, src, "flying"));
    assert!(sim_core::dsl::has_kw(&g, src, "deathtouch"));
    sim_core::engine::zones::die(&mut g, src, "destroy").unwrap();
    assert_eq!(g.perm(big).owner, P1);
    assert!(g.control_links.is_empty());
}

#[test]
fn silumgar_changing_hands_ends_the_steal_for_good() {
    use sim_core::engine::control::{Duration, gain_control, return_borrowed};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    gain_control(&mut g, P1, src, Duration::EndOfTurn);
    assert_eq!(g.perm(victim).owner, P1);
    return_borrowed(&mut g, P1);
    assert_eq!(g.perm(src).owner, P0);
    assert_eq!(g.perm(victim).owner, P1);
    assert!(g.control_links.is_empty());
}

#[test]
fn miiryms_silumgar_copy_anchors_its_own_control_effect() {
    let mut g = tbl();
    perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Blightsteel Colossus");
    let src = perm(&mut g, P0, SILUMGAR);
    assert_eq!(g.control_links.len(), 2);
    let original = *g.control_links.iter().find(|link| link.anchor == src).unwrap();
    let copy = *g.control_links.iter().find(|link| link.anchor != src).unwrap();
    assert!(g.perm(copy.anchor).token);
    sim_core::engine::zones::die(&mut g, src, "destroy").unwrap();
    assert_eq!(g.perm(original.stolen).owner, P1);
    assert_eq!(g.perm(copy.stolen).owner, P0);
    sim_core::engine::zones::die(&mut g, copy.anchor, "destroy").unwrap();
    assert_eq!(g.perm(copy.stolen).owner, P1);
}

#[test]
fn silumgar_blink_creates_a_new_anchor_and_phasing_ends_control() {
    use sim_core::engine::{control, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    g.perm_mut(src).phased = true;
    control::end_links(&mut g);
    assert_eq!(g.perm(victim).owner, P1, "phasing out the anchor ends its duration");
    assert!(g.control_links.is_empty());
    g.perm_mut(src).phased = false;
    assert_eq!(g.perm(victim).owner, P1, "phasing in does not restart the effect");
    zones::leave(&mut g, src).unwrap();
    assert_eq!(g.perm(victim).owner, P1);
    let new = perm(&mut g, P0, SILUMGAR);
    assert_ne!(src, new);
    assert_eq!(g.perm(victim).owner, P0);
    zones::leave(&mut g, new).unwrap();
    assert_eq!(g.perm(victim).owner, P1);
}

#[test]
fn teferis_protection_ends_silumgars_steal_even_when_both_phase_out() {
    use sim_core::engine::{turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    g.perm_mut(victim).tapped = true;
    zones::teferis_protection(&mut g, P0);
    assert!(g.perm(src).phased && g.perm(victim).phased);
    assert_eq!(g.perm(victim).owner, P1);
    assert!(g.player(P1).perms.contains(&victim) && !g.player(P0).perms.contains(&victim));
    assert!(g.control_links.is_empty());
    assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)));
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(victim).phased && g.perm(src).phased);
    assert!(g.perm(victim).tapped, "a phased-out permanent does not untap");
    turn::step_start(&mut g, P0).unwrap();
    assert!(!g.perm(src).phased && !g.perm(victim).phased);
    assert!(g.perm(victim).tapped, "phasing in during another player's untap does not untap it");
    assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
    assert_eq!(g.perm(victim).owner, P1, "the control effect does not restart");
    turn::step_start(&mut g, P1).unwrap();
    assert!(!g.perm(victim).tapped);
}

#[test]
fn phasing_only_silumgars_victim_keeps_the_control_effect() {
    use sim_core::engine::{control, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    g.perm_mut(victim).phased = true;
    control::end_links(&mut g);
    assert_eq!(g.perm(victim).owner, P0);
    assert_eq!(g.control_links.len(), 1);
    zones::leave(&mut g, src).unwrap();
    assert_eq!(g.perm(victim).owner, P1, "the effect can expire while its victim is phased out");
}

#[test]
fn phased_silumgar_victim_returns_when_anchor_leaves_but_phases_in_with_holder() {
    use sim_core::engine::{control, turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    g.perm_mut(victim).phased = true;
    control::end_links(&mut g);
    assert_eq!(g.control_links.len(), 1);
    assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
    zones::leave(&mut g, src).unwrap();
    assert_eq!(g.perm(victim).owner, P1);
    assert!(g.player(P1).perms.contains(&victim) && !g.player(P0).perms.contains(&victim));
    assert!(g.control_links.is_empty());
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(victim).phased);
    turn::step_start(&mut g, P0).unwrap();
    assert!(!g.perm(victim).phased);
    assert_eq!(g.perm(victim).owner, P1);
    assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
}

#[test]
fn phasing_without_a_control_link_still_uses_the_controllers_untap() {
    use sim_core::engine::{turn, zones};
    let mut g = tbl();
    let m = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(m).tapped = true;
    zones::teferis_protection(&mut g, P0);
    assert!(g.control_links.is_empty());
    assert!(g.perm(m).data.get(DataKey::PhaseInWith).is_none());
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(m).phased && g.perm(m).tapped);
    turn::step_start(&mut g, P0).unwrap();
    assert!(!g.perm(m).phased && !g.perm(m).tapped);
    assert!(g.perm(m).data.get(DataKey::PhaseInWith).is_none());
}

#[test]
fn returned_phased_victim_uses_current_controller_if_former_holder_is_eliminated() {
    use sim_core::engine::{life, turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, SILUMGAR);
    zones::teferis_protection(&mut g, P0);
    life::eliminate(&mut g, P0);
    assert!(live(&g, victim) && g.perm(victim).phased);
    assert_eq!(g.perm(victim).owner, P1);
    turn::step_start(&mut g, P1).unwrap();
    assert!(!g.perm(victim).phased);
    assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
}

#[test]
fn returned_phased_victims_forget_phase_in_timing_when_they_leave() {
    use sim_core::engine::{life, zones};
    for eliminated in [false, true] {
        let mut g = tbl();
        let victim = perm(&mut g, P1, "Llanowar Elves");
        perm(&mut g, P0, SILUMGAR);
        zones::teferis_protection(&mut g, P0);
        assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)));
        if eliminated {
            life::eliminate(&mut g, P1);
        } else {
            zones::leave(&mut g, victim).unwrap();
        }
        assert!(!live(&g, victim));
        assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
    }
}

#[test]
fn spark_double_copying_silumgar_runs_the_copied_enter_trigger() {
    use sim_core::control::Answer;
    use sim_core::human::script::ScriptHuman;
    use std::sync::Arc;
    let mut g = tbl();
    let src = perm(&mut g, P0, SILUMGAR);
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let human = Arc::new(ScriptHuman::from_fn(|r| {
        let wanted = if r.prompt.starts_with("Spark Double: enter as a copy") {
            SILUMGAR
        } else if r.prompt.starts_with("Dragonlord Silumgar: gain control of which") {
            "Llanowar Elves"
        } else {
            return Answer::Decline;
        };
        Answer::Pick(r.choices.iter().position(|s| s.contains(wanted)).unwrap())
    }));
    g.humans.add(P0, human.clone());
    let copy = perm(&mut g, P0, "Spark Double");
    assert_eq!(g.perm(copy).cd, Some(card(&g, SILUMGAR)));
    assert_eq!(g.perm(copy).phys, Some(card(&g, "Spark Double")));
    assert!(g.perm(copy).exc.not_legendary && g.perm(copy).plus == 1);
    assert_eq!(g.perm(victim).owner, P0);
    assert!(g.control_links.iter().any(|l| l.anchor == copy && l.stolen == victim));
    assert_eq!(
        human.asked().iter().filter(|r| r.prompt.starts_with("Dragonlord Silumgar: gain control of which")).count(),
        1
    );
    sim_core::engine::zones::leave(&mut g, src).unwrap();
    assert_eq!(g.perm(victim).owner, P0, "the copy has its own anchor");
    sim_core::engine::zones::leave(&mut g, copy).unwrap();
    assert_eq!(g.perm(victim).owner, P1);
}

#[test]
fn two_silumgar_effects_on_one_creature_hold_it_until_both_anchors_leave() {
    use sim_core::control::Answer;
    use sim_core::engine::zones;
    use sim_core::human::script::ScriptHuman;
    use std::sync::Arc;
    for order in 0..3 {
        let mut g = tbl();
        let victim = perm(&mut g, P1, "Llanowar Elves");
        let src = perm(&mut g, P0, SILUMGAR);
        g.humans.add(
            P0,
            Arc::new(ScriptHuman::from_fn(|r| {
                let wanted = if r.prompt.starts_with("Spark Double: enter as a copy") {
                    SILUMGAR
                } else if r.prompt.starts_with("Dragonlord Silumgar: gain control of which") {
                    "Llanowar Elves"
                } else {
                    return Answer::Decline;
                };
                Answer::Pick(r.choices.iter().position(|s| s.contains(wanted)).unwrap())
            })),
        );
        let copy = perm(&mut g, P0, "Spark Double");
        assert_eq!(g.control_links.len(), 2);
        if order == 2 {
            zones::teferis_protection(&mut g, P0);
        } else {
            let (first, last) = if order == 0 { (src, copy) } else { (copy, src) };
            zones::leave(&mut g, first).unwrap();
            assert_eq!(g.perm(victim).owner, P0, "the other Silumgar still holds the creature");
            assert_eq!(g.control_links.len(), 1);
            zones::leave(&mut g, last).unwrap();
        }
        assert_eq!(g.perm(victim).owner, P1);
        assert!(g.control_links.is_empty());
    }
}

#[test]
fn silumgar_can_take_a_planeswalker_and_control_effect_ends_on_elimination() {
    let mut g = tbl();
    let walker = perm(&mut g, P1, "Jace, the Mind Sculptor");
    perm(&mut g, P0, SILUMGAR);
    assert_eq!(g.perm(walker).owner, P0);
    assert!(g.perm(walker).loyalty.is_some());
    sim_core::engine::life::eliminate(&mut g, P0);
    assert_eq!(g.perm(walker).owner, P1);
    assert!(live(&g, walker) && g.player(P1).perms.contains(&walker));
}

#[test]
fn silumgar_does_nothing_if_its_duration_has_already_ended() {
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let src = perm(&mut g, P0, SILUMGAR);
    sim_core::engine::zones::leave(&mut g, src).unwrap();
    let f = g.registry.get(card(&g, SILUMGAR)).unwrap().etb.unwrap();
    f(&mut g, src, P0, src).unwrap();
    assert_eq!(g.perm(victim).owner, P1, "rule 611.2b");
}

#[test]
fn silumgar_pays_ward_or_fails_to_steal() {
    for paid in [false, true] {
        let mut g = tbl();
        let victim = perm(&mut g, P1, "Bronze Guardian"); // ward {2}
        if paid {
            lands(&mut g, P0, "Island", 2, false);
        }
        perm(&mut g, P0, SILUMGAR);
        assert_eq!(g.perm(victim).owner, if paid { P0 } else { P1 });
        if paid {
            assert!(!sim_core::engine::mana::can_pay(&g, P0, 1, "", false));
        }
    }
}

#[test]
fn silumgar_links_drop_when_the_victim_leaves_changes_hands_or_its_owner_leaves() {
    use sim_core::engine::{
        control::{Duration, gain_control},
        life, zones,
    };
    for mode in 0..3 {
        let mut g = tbl();
        let victim = perm(&mut g, P1, "Llanowar Elves");
        let src = perm(&mut g, P0, SILUMGAR);
        match mode {
            0 => {
                zones::die(&mut g, victim, "destroy").unwrap();
            }
            1 => {
                gain_control(&mut g, PlayerId(2), victim, Duration::Permanent);
            }
            _ => {
                life::eliminate(&mut g, P1);
            }
        }
        assert!(g.control_links.is_empty());
        zones::leave(&mut g, src).unwrap();
        assert_eq!(g.perm(victim).on_bf, mode == 1);
        if mode == 1 {
            assert_eq!(g.perm(victim).owner, PlayerId(2));
        }
    }
}

fn dominance(g: &mut Game) {
    let c = card(g, "Kindred Dominance");
    apply_wipe(g, P0, "dominance", Some(c), &Ctx::default()).unwrap();
}

#[test]
fn kindred_dominance_names_the_type_that_keeps_the_casters_creatures() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord"); // Human Barbarian
    let titan = perm(&mut g, P1, "Grave Titan"); // Zombie Giant
    let tok = token(&mut g, P1, 1);
    dominance(&mut g);
    assert!(live(&g, godo), "Godo's type is named");
    assert!(!live(&g, titan) && !live(&g, tok), "every creature not of that type is destroyed");
}

#[test]
fn kindred_dominance_with_maskwood_nexus_spares_every_creature_of_ours() {
    let mut g = tbl();
    perm(&mut g, P0, "Maskwood Nexus");
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let celebrant = perm(&mut g, P0, "Combat Celebrant");
    let titan = perm(&mut g, P1, "Grave Titan");
    dominance(&mut g);
    assert!(live(&g, godo) && live(&g, celebrant), "Maskwood Nexus: ours are every type");
    assert!(!live(&g, titan));
}

#[test]
fn kaldra_compleat_comes_with_a_germ_it_makes_a_five_five() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kaldra Compleat");
    assert!(indestructible(&g, k), "Kaldra itself is indestructible");
    let germ = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).expect("a Germ");
    assert_eq!(g.perm(k).attached, Some(germ), "living weapon: attached to the Germ");
    assert_eq!((epow(&g, germ), etgh(&g, germ)), (5, 5));
    for kw in ["first strike", "trample", "indestructible", "haste"] {
        assert!(sim_core::dsl::has_kw(&g, germ, kw), "{kw}");
    }
}

#[test]
fn the_thanos_seat_plays_from_its_plan_file() {
    assert!(sim_core::ai::plan_data::get("kyodai").is_some());
    assert!(sim_core::ai::plans::plays_plan("kyodai"));
    assert!(!sim_core::ai::is_main("kyodai"), "a plan-file deck, not a hand-written main deck");
    assert!(
        !sim_core::t6stats::is_t6_key("kyodai"),
        "the artifacts-and-Dragons list plays Tier 5 on the ordinary rules"
    );
    assert!(sim_core::t6stats::is_t6_key("thrasios"), "the Doomsday list took the Tier 6 key");
    let s = sim_core::ai::base_style("kyodai");
    assert_eq!((s.aggression, s.caution), (0.6, 0.5), "the 11/05 plan style");
}

/// P0 takes P1's creature until end of turn and it phases out under P0 (Talon Gates). At the end step it goes back
/// to P1, and it still phases in at P0's untap step, the controller it phased out under.
#[test]
fn a_threatened_creature_phased_out_under_the_thief_phases_in_at_the_thiefs_untap() {
    use sim_core::engine::{control, turn};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    control::gain_control(&mut g, P0, victim, control::Duration::EndOfTurn);
    assert_eq!(g.perm(victim).owner, P0);
    g.perm_mut(victim).phased = true;
    control::return_borrowed(&mut g, P0);
    assert_eq!(g.perm(victim).owner, P1);
    assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)));
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(victim).phased, "P1's untap step is not the one it phased out under");
    turn::step_start(&mut g, P0).unwrap();
    assert!(!g.perm(victim).phased);
    assert_eq!(g.perm(victim).owner, P1);
    assert!(g.perm(victim).data.get(DataKey::PhaseInWith).is_none());
}

/// a phased-out permanent that changes hands keeps the player it phased out under
#[test]
fn gaining_control_of_a_phased_out_permanent_keeps_the_old_controllers_untap() {
    use sim_core::engine::{control, turn};
    let mut g = tbl();
    let m = perm(&mut g, P1, "Llanowar Elves");
    g.perm_mut(m).phased = true;
    control::gain_control(&mut g, P0, m, control::Duration::Permanent);
    assert_eq!(g.perm(m).data.get(DataKey::PhaseInWith), Some(&Val::Player(P1)));
    turn::step_start(&mut g, P0).unwrap();
    assert!(g.perm(m).phased, "it phased out under P1");
    turn::step_start(&mut g, P1).unwrap();
    assert!(!g.perm(m).phased);
    assert_eq!(g.perm(m).owner, P0);
    // an earlier record is kept: a second change of hands does not overwrite it
    let mut g = tbl();
    let m = perm(&mut g, P1, "Llanowar Elves");
    g.perm_mut(m).phased = true;
    control::gain_control(&mut g, P0, m, control::Duration::Permanent);
    control::gain_control(&mut g, PlayerId(2), m, control::Duration::Permanent);
    assert_eq!(g.perm(m).data.get(DataKey::PhaseInWith), Some(&Val::Player(P1)));
}

/// the helper alone: it records the controller being left, keeps an earlier record, and ignores phased-in permanents
#[test]
fn note_phase_holder_records_only_the_first_controller_of_a_phased_out_permanent() {
    use sim_core::engine::control;
    let mut g = tbl();
    let m = perm(&mut g, P1, "Llanowar Elves");
    control::gain_control(&mut g, P0, m, control::Duration::Permanent);
    g.perm_mut(m).phased = true;
    control::note_phase_holder(&mut g, m, P0);
    assert_eq!(g.perm(m).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)));
    control::note_phase_holder(&mut g, m, P1);
    assert_eq!(g.perm(m).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)), "the first record stays");
    let plain = perm(&mut g, P1, "Llanowar Elves");
    control::note_phase_holder(&mut g, plain, P1);
    assert!(g.perm(plain).data.get(DataKey::PhaseInWith).is_none(), "a phased-in permanent gets no record");
}

/// P2 takes the creature from P0's effect (which ends for good), it phases out under P2, and P2's effect ends: it goes
/// to P0 and still phases in at P2's untap step, the player it phased out under.
#[test]
fn a_creature_taken_from_a_control_effect_phases_in_for_the_player_it_phased_out_under() {
    use sim_core::engine::{control, turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let (a, b) = (perm(&mut g, P0, "Llanowar Elves"), perm(&mut g, PlayerId(2), "Llanowar Elves"));
    control::gain_control(&mut g, P0, victim, control::Duration::WhileControls(a));
    control::gain_control(&mut g, PlayerId(2), victim, control::Duration::WhileControls(b));
    assert_eq!(g.perm(victim).owner, PlayerId(2));
    g.perm_mut(victim).phased = true;
    zones::leave(&mut g, b).unwrap();
    assert_eq!(g.perm(victim).owner, P0, "it goes back to the player it was taken from");
    assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(PlayerId(2))));
    zones::leave(&mut g, a).unwrap(); // P0's own effect was dropped when P2 took the creature: nothing changes
    assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(PlayerId(2))));
    turn::step_start(&mut g, P0).unwrap();
    assert!(g.perm(victim).phased);
    turn::step_start(&mut g, PlayerId(2)).unwrap();
    assert!(!g.perm(victim).phased);
    assert_eq!(g.perm(victim).owner, P0);
}

/// phasing in does not interrupt control: a creature returned while phased out is not summoning sick at the untap
/// step of the player it came back to
#[test]
fn a_returned_phased_creature_is_not_summoning_sick_at_its_owners_untap() {
    use sim_core::engine::{turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, SILUMGAR);
    zones::teferis_protection(&mut g, P0);
    assert!(g.perm(victim).sick, "regaining control makes it sick for the moment");
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(victim).phased);
    assert!(!g.perm(victim).sick, "P1 has controlled it since its own turn began");
}

#[test]
fn phase_in_bumps_the_battlefield_version_once_something_phased_in() {
    use sim_core::engine::turn;
    let mut g = tbl();
    let m = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(m).phased = true;
    let before = g.bf_ver;
    turn::phase_in_for(&mut g, P0);
    assert!(!g.perm(m).phased && g.bf_ver > before);
    let after = g.bf_ver;
    turn::phase_in_for(&mut g, P0);
    assert_eq!(g.bf_ver, after, "nothing phased in");
}

/// the AI planner's copy of the next turn follows the same phase-in rule as the real untap step
#[test]
fn the_planners_next_turn_leaves_a_creature_phased_out_under_another_player() {
    use sim_core::ai::planner::next_turn;
    use sim_core::engine::zones;
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, SILUMGAR);
    zones::teferis_protection(&mut g, P0);
    g.perm_mut(victim).tapped = true;
    let mut copy = g.clone();
    next_turn(&mut copy, P1).unwrap();
    assert!(copy.perm(victim).phased && copy.perm(victim).tapped, "P1's next turn leaves it phased out");
    let mut copy = g.clone();
    next_turn(&mut copy, P0).unwrap();
    assert!(!copy.perm(victim).phased, "P0's next turn phases it in");
    assert!(copy.perm(victim).tapped, "and phasing in does not untap it");
}

/// Abduction (a card path that hands the creature back by its own code) keeps the controller it phased out under
#[test]
fn abduction_leaving_returns_a_phased_creature_for_the_thiefs_untap() {
    use sim_core::engine::{turn, zones};
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Llanowar Elves");
    let aura = perm(&mut g, P0, "Abduction");
    assert_eq!(g.perm(victim).owner, P0);
    g.perm_mut(victim).phased = true;
    zones::leave(&mut g, aura).unwrap();
    assert_eq!(g.perm(victim).owner, P1);
    assert_eq!(g.perm(victim).data.get(DataKey::PhaseInWith), Some(&Val::Player(P0)));
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(victim).phased);
    turn::step_start(&mut g, P0).unwrap();
    assert!(!g.perm(victim).phased);
}
