//! The modeled combos win only with a way to turn the loop into a kill: unbounded mana needs a sink (Walking
//! Ballista, Urza), a death loop needs a payoff that triggers on deaths, Krenko's Goblins need an outlet that makes
//! mana or deals damage, and Kiki-Jiki's hasty copies need a combat still to come.

use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KINNAN: &str = "kinnan-simic-mana-combo";

fn ready(g: &Game, part: &str) -> bool {
    combos::ready(g, P0, combos::combo_index(part).unwrap())
}

#[test]
fn kinnan_and_basalt_monolith_need_a_mana_sink() {
    // the practice game: Kinnan + Basalt Monolith with no outlet "won" with everyone at 36-41 life
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    assert!(!ready(&g, "Kinnan + Basalt"), "unbounded colourless alone kills no one");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(ready(&g, "Kinnan + Basalt"));
}

#[test]
fn a_tapped_basalt_monolith_needs_three_to_start() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    let b = perm(&mut g, P0, "Basalt Monolith");
    hand(&mut g, P0, &["Walking Ballista"]);
    g.perm_mut(b).tapped = true;
    assert!(!ready(&g, "Kinnan + Basalt"));
    lands(&mut g, P0, "Island", 3, false);
    assert!(ready(&g, "Kinnan + Basalt"));
}

#[test]
fn kinnan_is_no_sink_for_power_artifact() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Power Artifact");
    perm(&mut g, P0, "Grim Monolith");
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    assert!(!ready(&g, "Power Artifact"), "Kinnan's dig needs green and blue each time");
    perm(&mut g, P0, "Urza, Lord High Artificer");
    assert!(ready(&g, "Power Artifact"));
}

#[test]
fn thassas_oracle_and_spell_tutors_are_no_mana_sinks() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Power Artifact");
    perm(&mut g, P0, "Basalt Monolith");
    hand(&mut g, P0, &["Thassa's Oracle", "Mystical Tutor"]);
    assert!(g.player(P0).library.contains(&card(&g, "Walking Ballista")));
    assert!(!ready(&g, "Power Artifact"), "mana doesn't empty the library; Mystical Tutor can't find a creature");
    hand(&mut g, P0, &["Demonic Tutor"]);
    assert!(ready(&g, "Power Artifact"), "Demonic Tutor finds the Ballista");
}

#[test]
fn freed_from_the_real_needs_a_dork_that_makes_blue() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Freed from the Real");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(!ready(&g, "Freed from the Real"), "no mana creature");
    perm(&mut g, P0, "Elvish Mystic");
    assert!(!ready(&g, "Freed from the Real"), "two green can't pay the {{U}} untap");
    perm(&mut g, P0, "Birds of Paradise");
    assert!(ready(&g, "Freed from the Real"));
    g.player_mut(P0).hand.clear();
    assert!(!ready(&g, "Freed from the Real"), "and a sink");
}

#[test]
fn goblin_bombardment_is_no_payoff_for_the_gravecrawler_loop() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Gravecrawler");
    perm(&mut g, P0, "Geralf's Messenger");
    perm(&mut g, P0, "Goblin Bombardment");
    assert!(!ready(&g, "Gravecrawler"), "the loop sacrifices to the Altar, so the Bombardment never fires");
    perm(&mut g, P0, "Blood Artist");
    assert!(ready(&g, "Gravecrawler"));
}

#[test]
fn gravecrawler_in_the_graveyard_needs_black_to_start() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Phyrexian Altar");
    perm(&mut g, P0, "Geralf's Messenger");
    perm(&mut g, P0, "Blood Artist");
    let gc = take(&mut g, P0, "Gravecrawler");
    g.player_mut(P0).gy.push(gc);
    assert!(!ready(&g, "Gravecrawler"));
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(ready(&g, "Gravecrawler"));
}

#[test]
fn krenko_and_thornbite_staff_need_an_outlet_that_kills() {
    let mut g = table(&[KINNAN, "seph"]);
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    let st = perm(&mut g, P0, "Thornbite Staff");
    g.perm_mut(st).attached = Some(k);
    perm(&mut g, P0, "Viscera Seer");
    assert!(!ready(&g, "Krenko"), "summoning-sick Goblins and scries kill no one");
    perm(&mut g, P0, "Blood Artist");
    assert!(ready(&g, "Krenko"), "each Goblin's death drains");
    let mut g = table(&[KINNAN, "seph"]);
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    let st = perm(&mut g, P0, "Thornbite Staff");
    g.perm_mut(st).attached = Some(k);
    perm(&mut g, P0, "Skirk Prospector");
    assert!(ready(&g, "Krenko"), "the Prospector's mana pays for the Staff's pings");
}

#[test]
fn kiki_jiki_goes_off_only_before_combat() {
    let mut g = table(&[KINNAN, "seph"]);
    perm(&mut g, P0, "Kiki-Jiki, Mirror Breaker");
    perm(&mut g, P0, "Pestermite");
    assert!(ready(&g, "Kiki-Jiki"));
    g.step = Step::Main2;
    assert!(!ready(&g, "Kiki-Jiki"), "the hasty copies have no combat left");
}
