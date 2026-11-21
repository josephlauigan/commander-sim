//! Gandalf the White and Delney, Streetwise Lookout: what a sacrifice, a simultaneous return to hand and a face-down
//! permanent do to the triggers they add. A face-down permanent has no supertype and is not an artifact (CR 708.2).

mod play_common;
use play_common::*;
use sim_core::engine::{removal, zones};
use sim_core::hooks::{Call, TrigAct};
use sim_core::state::{Ctx, DataKey, Step, Val};

const GANDALF: &str = "Gandalf the White";
const DELNEY: &str = "Delney, Streetwise Lookout";
const VONDAM: &str = "Syr Vondam, Sunstar Exemplar";

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn tokens_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

#[test]
fn juri_hears_delney_sacrificed_alone_twice() {
    let mut g = tbl();
    let juri = perm(&mut g, P0, "Juri, Master of the Revue");
    let delney = perm(&mut g, P0, DELNEY);
    zones::die(&mut g, delney, "sac").unwrap();
    assert_eq!(g.perm(juri).plus, 2, "Juri (power 1) and Delney existed just before the sacrifice (CR 603.10a)");
}

#[test]
fn juri_hears_a_creature_sacrificed_twice_while_delney_stays() {
    let mut g = tbl();
    let juri = perm(&mut g, P0, "Juri, Master of the Revue");
    perm(&mut g, P0, DELNEY);
    let x = perm(&mut g, P0, "Mother of Runes");
    zones::die(&mut g, x, "sac").unwrap();
    assert_eq!(g.perm(juri).plus, 2, "Juri has power 1 and Delney is on the battlefield: twice, as for any trigger");
}

#[test]
fn juri_hears_delney_sacrificed_twice_with_teysa_too() {
    let mut g = tbl();
    let juri = perm(&mut g, P0, "Juri, Master of the Revue");
    let delney = perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Teysa Karlov");
    zones::die(&mut g, delney, "sac").unwrap();
    assert_eq!(g.perm(juri).plus, 2, "Delney adds one; Teysa's copies are for the dying and not for a sacrifice");
}

/// Gandalf and Delney with the abilities they keep as noncreature permanents (they survive Evacuation), a plain and a
/// legendary creature, and a Dour Port-Mage: the Port-Mage triggers waiting for the creatures that left together
fn port_mage_waiting(first: &str, second: &str) -> usize {
    let mut g = tbl();
    for name in [GANDALF, DELNEY] {
        let m = perm(&mut g, P0, name);
        g.perm_mut(m).data.set(DataKey::NotCreature, Val::Bool(true));
        g.perm_mut(m).exc.add_types = sim_core::cards::Types::ENCHANTMENT;
    }
    perm(&mut g, P0, first);
    perm(&mut g, P0, second);
    let mage = perm(&mut g, P0, "Dour Port-Mage");
    g.resolving = 1;
    removal::apply_wipe(&mut g, P0, "evac", None, &Ctx::default()).unwrap();
    g.trig_queue
        .iter()
        .filter(|t| {
            t.src == Some(mage) && matches!(&t.act, TrigAct::Hook { call: Call::Leaves { m }, .. } if *m != mage)
        })
        .count()
}

#[test]
fn dour_port_mage_triggers_three_times_for_a_batch_with_a_legendary_creature_with_gandalf_and_delney() {
    assert_eq!(port_mage_waiting("Mother of Runes", "Tymna the Weaver"), 3, "one + Delney + Gandalf");
    assert_eq!(port_mage_waiting("Tymna the Weaver", "Mother of Runes"), 3, "in either order");
}

#[test]
fn dour_port_mage_triggers_twice_for_a_batch_of_plain_creatures_with_delney() {
    assert_eq!(port_mage_waiting("Mother of Runes", "Blood Artist"), 2, "one + Delney");
}

#[test]
fn stormfront_riders_returning_two_legendary_creatures_make_a_doubled_soldier_for_each() {
    let mut g = tbl();
    let gandalf = perm(&mut g, P0, GANDALF);
    g.perm_mut(gandalf).plus = 100; // the AI keeps Gandalf when it picks the two creatures to return
    perm(&mut g, P0, "Tymna the Weaver");
    perm(&mut g, P0, VONDAM);
    let before = tokens_of(&g, P0);
    perm(&mut g, P0, "Stormfront Riders");
    assert!(g.perm(gandalf).on_bf, "Gandalf stays");
    assert_eq!(tokens_of(&g, P0) - before, 4, "each of the two returns: one + Gandalf");
}

#[test]
fn stormfront_riders_returning_a_plain_and_a_legendary_creature_make_three_soldiers() {
    let mut g = tbl();
    let gandalf = perm(&mut g, P0, GANDALF);
    g.perm_mut(gandalf).plus = 100;
    perm(&mut g, P0, "Tymna the Weaver");
    perm(&mut g, P0, "Mother of Runes");
    let before = tokens_of(&g, P0);
    perm(&mut g, P0, "Stormfront Riders");
    assert!(g.perm(gandalf).on_bf, "Gandalf stays");
    assert_eq!(tokens_of(&g, P0) - before, 3, "one for the plain creature, two for the legendary one");
}

/// a face-down 2/2 as the engine keeps it (`enter_face_down`), sacrificed with Gandalf and Syr Vondam on the table
fn vondam_after_a_face_down_sacrifice(name: &str) -> i32 {
    let mut g = tbl();
    perm(&mut g, P0, GANDALF);
    let vondam = perm(&mut g, P0, VONDAM);
    let x = perm(&mut g, P0, name);
    g.perm_mut(x).data.set(DataKey::Facedown, Val::List(vec![Val::Int(2), Val::Int(2)]));
    g.perm_mut(x).neutered = true;
    g.perm_mut(x).pow = 2;
    g.perm_mut(x).tgh = 2;
    zones::die(&mut g, x, "sac").unwrap();
    g.perm(vondam).plus
}

#[test]
fn a_face_down_artifact_creature_is_not_an_artifact_for_gandalf() {
    assert_eq!(vondam_after_a_face_down_sacrifice("Solemn Simulacrum"), 1, "a face-down 2/2 creature");
}

#[test]
fn a_face_down_legendary_creature_is_not_legendary_for_gandalf() {
    assert_eq!(vondam_after_a_face_down_sacrifice("Tymna the Weaver"), 1, "a face-down 2/2 has no supertype");
}

#[test]
fn a_face_up_legendary_or_artifact_creature_dying_triggers_vondam_twice() {
    for name in ["Tymna the Weaver", "Solemn Simulacrum"] {
        let mut g = tbl();
        perm(&mut g, P0, GANDALF);
        let vondam = perm(&mut g, P0, VONDAM);
        let x = perm(&mut g, P0, name);
        zones::die(&mut g, x, "sac").unwrap();
        assert_eq!(g.perm(vondam).plus, 2, "{name}");
    }
}
