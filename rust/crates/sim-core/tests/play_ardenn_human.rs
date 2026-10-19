//! Practice mode: beginning-of-combat triggers for the human seat come before attackers are declared, and Ardenn,
//! Intrepid Archaeologist asks what to attach ("At the beginning of combat on your turn, you may attach any number of
//! Auras and Equipment you control to target permanent or player.")

mod play_common;
use play_common::*;
use std::sync::{Arc, Mutex};

const ARDENN: &str = "Ardenn, Intrepid Archaeologist";

/// answers Ardenn's questions (the creature whose name contains `to`, then every Equipment, then done) and declines
/// everything else (no attack); records the kinds and prompts asked, in order
fn script(to: &'static str, asked: Arc<Mutex<Vec<String>>>) -> ScriptHuman {
    ScriptHuman::from_fn(move |r| {
        asked.lock().unwrap().push(format!("{}: {}", r.kind, r.prompt));
        if r.prompt.contains("to which creature") {
            Answer::Pick(r.choices.iter().position(|c| c.contains(to)).unwrap())
        } else if r.prompt.starts_with("Ardenn: attach which") {
            Answer::Pick(0) // the first Equipment; once only "done" is left, that
        } else {
            Answer::Decline
        }
    })
}

#[test]
fn ardenn_asks_you_at_the_beginning_of_combat_before_you_declare_attackers() {
    let mut g = table(&["cloud", "seph"]);
    let asked = Arc::new(Mutex::new(vec![]));
    g.humans.add(P0, Arc::new(script("Esper Sentinel", asked.clone())));
    perm(&mut g, P0, ARDENN);
    let holder = perm(&mut g, P0, "Esper Sentinel");
    g.perm_mut(holder).sick = false;
    let boots = perm(&mut g, P0, "Lightning Greaves");
    let bone = perm(&mut g, P0, "Bonesplitter");
    g.active = Some(P0);
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.perm(bone).attached, Some(holder), "Bonesplitter moved to the Sentinel");
    assert_eq!(g.perm(boots).attached, Some(holder), "and the Greaves");
    let asked = asked.lock().unwrap();
    let ardenn = asked.iter().position(|a| a.contains("Ardenn")).expect("Ardenn asked you");
    let attack = asked.iter().position(|a| a.starts_with("attack:")).expect("then the attack");
    assert!(ardenn < attack, "the beginning of combat comes first: {asked:?}");
}

#[test]
fn you_may_attach_nothing() {
    let mut g = table(&["cloud", "seph"]);
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Decline)); // every question declined: attach nothing, no attack
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, ARDENN);
    let bone = perm(&mut g, P0, "Bonesplitter");
    g.active = Some(P0);
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.perm(bone).attached, None);
    assert!(h.asked().iter().any(|r| r.prompt.contains("Ardenn")));
}
