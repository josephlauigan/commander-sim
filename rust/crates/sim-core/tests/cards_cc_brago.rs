//! Casual combos (brago): the unbounded-mana lines of brago-azorius-blink-control (impls/cc_brago.rs), on hand-built
//! positions. Seat 0 plays Brago; the opponents hold nothing unless a test gives them an answer.

use sim_core::cardcode;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const BRAGO: &str = "brago-azorius-blink-control";
const FLICKER: &str = "Peregrine Drake + Archaeomancer + Ghostly Flicker";
const KITTEN: &str = "Displacer Kitten + Archaeomancer";

fn brago() -> Game {
    let mut g = table(&[BRAGO, "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn ready(g: &Game, part: &str) -> bool {
    combos::ready(g, P0, combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part:?}")))
}

fn go(g: &mut Game, part: &str) {
    let i = combos::combo_index(part).unwrap();
    assert!(ready(g, part), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap();
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn offered(g: &mut Game, part: &str) -> bool {
    cardcode::combo_options(g, P0, 0.0, Some(false)).unwrap().iter().any(|o| o.label.contains(part))
}

// ------------------------------------------------------------------ Peregrine Drake + Archaeomancer + Ghostly Flicker
#[test]
fn the_flicker_loop_pays_for_agent_of_treachery_from_hand() {
    // Ghostly Flicker costs three; Agent's seven come from the loop, not from the lands up front
    let mut g = brago();
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Archaeomancer");
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Ghostly Flicker", "Agent of Treachery"]);
    assert!(ready(&g, FLICKER));
    assert!(offered(&mut g, FLICKER), "five lands are enough to go for it");
    go(&mut g, FLICKER);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Agent of Treachery").is_some(), "the Agent was cast");
}

#[test]
fn the_flicker_loop_needs_four_lands() {
    // three lands untapped pay for Flicker and no more: no mana is gained
    let mut g = brago();
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Island", 3, false);
    hand(&mut g, P0, &["Ghostly Flicker"]);
    assert!(!ready(&g, FLICKER));
    lands(&mut g, P0, "Island", 1, false);
    assert!(ready(&g, FLICKER));
}

#[test]
fn the_flicker_loop_digs_for_agent_with_a_drawer() {
    let mut g = brago();
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Archaeomancer");
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Ghostly Flicker"]);
    assert!(g.player(P0).library.iter().any(|&c| &*g.db.get(c).name == "Agent of Treachery"));
    assert!(!ready(&g, FLICKER), "unbounded mana alone does not win");
    perm(&mut g, P0, "Mulldrifter");
    assert!(ready(&g, FLICKER), "Mulldrifter blinked with the loop draws to the Agent");
    go(&mut g, FLICKER);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_counter_on_agent_of_treachery_stops_the_line() {
    let mut g = brago();
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Archaeomancer");
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Ghostly Flicker", "Agent of Treachery"]);
    lands(&mut g, P1, "Island", 4, false);
    hand(&mut g, P1, &["Counterspell", "Counterspell"]);
    go(&mut g, FLICKER);
    assert_eq!(g.winner, None, "Flicker or the Agent is countered");
    assert!(on_bf(&g, P0, "Agent of Treachery").is_none());
}

// ------------------------------------------------------------------ Displacer Kitten + Archaeomancer
#[test]
fn kitten_archaeomancer_drake_and_cloudshift_win_with_agent() {
    let mut g = brago();
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Plains", 2, false);
    assert!(!ready(&g, KITTEN), "a flicker spell is needed");
    hand(&mut g, P0, &["Cloudshift"]);
    assert!(ready(&g, KITTEN));
    assert!(offered(&mut g, KITTEN));
    go(&mut g, KITTEN);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn kitten_needs_archaeomancer_to_return_the_spell() {
    let mut g = brago();
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Cloudshift"]);
    assert!(!ready(&g, KITTEN));
}

#[test]
fn ephemerate_is_exiled_by_rebound_and_does_not_loop() {
    let mut g = brago();
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Peregrine Drake");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Ephemerate"]);
    assert!(!ready(&g, KITTEN));
}

#[test]
fn cloud_of_faeries_loops_with_cloudshift_but_not_momentary_blink() {
    // Cloud untaps two lands: Cloudshift ({W}) nets one, Momentary Blink ({1}{W}) nets nothing
    let mut g = brago();
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Cloud of Faeries");
    perm(&mut g, P0, "Agent of Treachery");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Momentary Blink", "Ghostly Flicker"]);
    assert!(!ready(&g, KITTEN), "Blink nets nothing, Flicker loses one (Kitten already blinked the Cloud)");
    hand(&mut g, P0, &["Cloudshift"]);
    assert!(ready(&g, KITTEN));
}

#[test]
fn kitten_line_casts_agent_from_hand_with_the_loops_mana() {
    let mut g = brago();
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Archaeomancer");
    perm(&mut g, P0, "Peregrine Drake");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Momentary Blink", "Agent of Treachery"]);
    assert!(ready(&g, KITTEN));
    go(&mut g, KITTEN);
    assert_eq!(g.winner, Some(P0));
}
