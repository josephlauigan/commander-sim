//! Gandalf the White ({3}{W}{W}, Flash): "You may cast legendary spells and artifact spells as though they had flash.
//! If a legendary permanent or an artifact entering or leaving the battlefield causes a triggered ability of a permanent
//! you control to trigger, that ability triggers an additional time." Each expectation follows the Oracle text and the
//! 2023-06-16 rulings (a land is never cast; the grant is for casting only).

mod play_common;
use play_common::*;
use sim_core::ai::brain;
use sim_core::engine::cast;
use sim_core::human::{human, legal};
use sim_core::state::Step;

const GANDALF: &str = "Gandalf the White";

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P1);
    g.step = Step::Main1;
    g
}

fn tap_all(g: &mut Game, p: PlayerId) {
    while let Some(s) = sim_core::human::mana::sources(g, p).first() {
        let id = s.id as i64;
        sim_core::human::mana::tap(g, p, id, None).unwrap();
    }
}

/// n lands that make {W} and {B}, tapped for mana into the pool as an instant-speed cast would draw on it
fn mana(g: &mut Game, p: PlayerId, n: usize) {
    lands(g, p, "Plains", n, false);
    lands(g, p, "Swamp", n, false);
}

// ------------------------------------------------------------------ the flash grant (person)
#[test]
fn a_person_casts_an_artifact_spell_on_an_opponents_turn_with_gandalf() {
    let mut g = tbl();
    mana(&mut g, P0, 2);
    let ring = hand(&mut g, P0, &["Sol Ring"])[0];
    assert_eq!(
        legal::check_cast(&mut g, P0, ring, "hand").as_deref(),
        Some("It isn't your turn."),
        "without Gandalf an artifact is a sorcery-speed spell"
    );
    perm(&mut g, P0, GANDALF);
    tap_all(&mut g, P0);
    assert_eq!(legal::check_cast(&mut g, P0, ring, "hand"), None);
}

#[test]
fn a_person_casts_a_legendary_spell_on_an_opponents_turn_with_gandalf() {
    let mut g = tbl();
    mana(&mut g, P0, 2);
    let syr = hand(&mut g, P0, &["Syr Vondam, Sunstar Exemplar"])[0];
    assert_eq!(legal::check_cast(&mut g, P0, syr, "hand").as_deref(), Some("It isn't your turn."));
    perm(&mut g, P0, GANDALF);
    tap_all(&mut g, P0);
    assert_eq!(legal::check_cast(&mut g, P0, syr, "hand"), None);
}

#[test]
fn a_person_casts_a_commander_from_the_command_zone_at_instant_speed_with_gandalf() {
    let mut g = tbl();
    let y = card(&g, "Yoshimaru, Ever Faithful");
    g.player_mut(P0).cmd = y;
    g.player_mut(P0).cmd_in_zone = true;
    mana(&mut g, P0, 1);
    assert_eq!(legal::check_cast(&mut g, P0, y, "cmd").as_deref(), Some("It isn't your turn."));
    perm(&mut g, P0, GANDALF);
    tap_all(&mut g, P0);
    assert_eq!(legal::check_cast(&mut g, P0, y, "cmd"), None);
}

#[test]
fn gandalfs_grant_is_for_legendary_and_artifact_spells_only() {
    let mut g = tbl();
    mana(&mut g, P0, 2);
    perm(&mut g, P0, GANDALF);
    let ranger = hand(&mut g, P0, &["Ranger-Captain of Eos"])[0];
    assert_eq!(
        legal::check_cast(&mut g, P0, ranger, "hand").as_deref(),
        Some("It isn't your turn."),
        "a creature that is neither legendary nor an artifact"
    );
    assert!(!cast::grants_flash(&g, P0, ranger));
    for land in ["Gaea's Cradle", "Seat of the Synod"] {
        let c = card(&g, land);
        assert!(!cast::grants_flash(&g, P0, c), "{land}: a land is never cast");
    }
    let ring = card(&g, "Sol Ring");
    assert!(cast::grants_flash(&g, P0, ring), "the control: an artifact does");
}

#[test]
fn a_phased_out_a_neutered_and_an_opponents_gandalf_grant_nothing() {
    let mut g = tbl();
    let ring = card(&g, "Sol Ring");
    let syr = card(&g, "Syr Vondam, Sunstar Exemplar");
    let theirs = perm(&mut g, P1, GANDALF);
    assert!(!cast::grants_flash(&g, P0, ring) && !cast::grants_flash(&g, P0, syr), "an opponent's Gandalf");
    g.perm_mut(theirs).phased = true;
    let mine = perm(&mut g, P0, GANDALF);
    assert!(cast::grants_flash(&g, P0, ring) && cast::grants_flash(&g, P0, syr), "the control: live");
    g.perm_mut(mine).phased = true;
    assert!(!cast::grants_flash(&g, P0, ring) && !cast::grants_flash(&g, P0, syr), "phased out");
    g.perm_mut(mine).phased = false;
    g.perm_mut(mine).neutered = true;
    assert!(!cast::grants_flash(&g, P0, ring) && !cast::grants_flash(&g, P0, syr), "neutered");
}

#[test]
fn a_person_with_only_a_commander_castable_through_gandalf_may_respond() {
    let mut g = tbl();
    let y = card(&g, "Yoshimaru, Ever Faithful");
    g.player_mut(P0).cmd = y;
    g.player_mut(P0).cmd_in_zone = true;
    g.player_mut(P0).hand.clear();
    mana(&mut g, P0, 1);
    assert!(!human::can_respond(&g, P0), "a commander is a sorcery-speed spell without Gandalf");
    perm(&mut g, P0, GANDALF);
    assert!(human::can_respond(&g, P0), "with Gandalf the commander can be cast in response");
}

// ------------------------------------------------------------------ the flash grant (AI)
fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    has_perm(g, p, name)
}

#[test]
fn the_ai_casts_an_artifact_at_the_end_of_an_opponents_turn_with_gandalf() {
    let build = |gandalf: bool| {
        let mut g = tbl();
        mana(&mut g, P0, 3);
        if gandalf {
            perm(&mut g, P0, GANDALF);
        }
        hand(&mut g, P0, &["Mind Stone"]);
        g.step = Step::End;
        brain::end_of_turn_window(&mut g, P0).unwrap();
        g
    };
    let with = build(true);
    assert!(on_bf(&with, P0, "Mind Stone"), "cast in the end-of-turn window");
    let without = build(false);
    assert!(!on_bf(&without, P0, "Mind Stone"), "no grant: it stays in hand");
}
