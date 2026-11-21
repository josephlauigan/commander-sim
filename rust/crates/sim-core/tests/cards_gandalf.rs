//! Gandalf the White ({3}{W}{W}, Flash): "You may cast legendary spells and artifact spells as though they had flash.
//! If a legendary permanent or an artifact entering or leaving the battlefield causes a triggered ability of a permanent
//! you control to trigger, that ability triggers an additional time." Each expectation follows the Oracle text and the
//! 2023-06-16 rulings (a land is never cast; the grant is for casting only).

mod play_common;
use play_common::*;
use sim_core::ai::brain;
use sim_core::engine::{cast, zones};
use sim_core::human::{human, legal};
use sim_core::state::{Ctx, Step};

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

// ------------------------------------------------------------------ entering: one additional time
const Y: &str = "Yoshimaru, Ever Faithful";
const ELESH: &str = "Elesh Norn, Mother of Machines";

fn mine() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn counters(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

#[test]
fn a_legendary_permanent_entering_triggers_yoshimaru_twice_and_a_plain_creature_not_at_all() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Syr Vondam, Sunstar Exemplar");
    assert_eq!(counters(&g, y), 2, "a legendary creature enters: 1 + Gandalf");
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(counters(&g, y), 2, "a nonlegendary creature causes nothing");
}

#[test]
fn gandalf_entering_is_doubled_with_his_own_copy_ability() {
    let mut g = mine();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, GANDALF);
    assert_eq!(counters(&g, y), 2, "Gandalf is legendary and sees himself enter");
}

#[test]
fn an_artifact_creature_entering_triggers_a_compiled_listener_twice() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Elas il-Kor, Sadistic Pilgrim");
    let life = g.player(P0).life;
    perm(&mut g, P0, "Solemn Simulacrum");
    assert_eq!(g.player(P0).life, life + 2, "an artifact creature: 1 + Gandalf");
    let life = g.player(P0).life;
    perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.player(P0).life, life + 1, "neither legendary nor an artifact");
}

#[test]
fn a_legendary_artifact_with_panharmonicon_triggers_three_times() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Panharmonicon");
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Mox Amber");
    assert_eq!(counters(&g, y), 3, "1 + Panharmonicon + Gandalf");
}

#[test]
fn gandalf_and_delney_make_three_triggers() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Delney, Streetwise Lookout");
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 3, "1 + Gandalf + Delney");
}

#[test]
fn gandalf_and_your_own_elesh_make_three_triggers() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, ELESH);
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 3, "1 + Gandalf + Elesh Norn");
}

#[test]
fn haliya_gains_two_life_for_a_treasure_with_gandalf_and_three_with_elesh_too() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Haliya, Guided by Light");
    let life = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, life + 2, "a Treasure is an artifact: 1 + Gandalf");
    perm(&mut g, P0, ELESH);
    let life = g.player(P0).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, life + 3, "1 + Gandalf + Elesh Norn");
}

#[test]
fn gandalf_adds_nothing_to_a_cast_trigger() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Lotho, Corrupt Shirriff");
    let t0 = g.player(P0).treasures;
    g.active = Some(P1);
    for n in ["Opt", "Brainstorm"] {
        let c = hand(&mut g, P1, &[n])[0];
        cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    }
    assert_eq!(g.player(P0).treasures, t0 + 1, "a cast is not an entering");
}

#[test]
fn an_opponents_gandalf_adds_nothing_to_your_triggers() {
    let mut g = mine();
    perm(&mut g, P1, GANDALF);
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Syr Vondam, Sunstar Exemplar");
    assert_eq!(counters(&g, y), 1, "the ability is the controller's own");
}

// ------------------------------------------------------------------ the copy rule itself
use sim_core::engine::hooks::total_trigger_copies;

#[test]
fn gandalfs_copies_follow_what_entered_or_left() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let legend = perm(&mut g, P0, "Tymna the Weaver");
    let ring = perm(&mut g, P0, "Sol Ring");
    let plain = perm(&mut g, P0, "Mother of Runes");
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, false)[0];
    let basic = lands(&mut g, P0, "Plains", 1, false)[0];
    let n = |g: &Game, p: PlayerId, kind: &'static str, x: Option<PermId>| total_trigger_copies(g, p, kind, x);
    for kind in ["etb", "dies", "leaves"] {
        assert_eq!(n(&g, P0, kind, Some(legend)), 1, "{kind}: a legendary permanent");
        assert_eq!(n(&g, P0, kind, Some(ring)), 1, "{kind}: an artifact");
        assert_eq!(n(&g, P0, kind, Some(plain)), 0, "{kind}: neither");
        assert_eq!(n(&g, P1, kind, Some(legend)), 0, "{kind}: only the controller's own abilities");
    }
    assert_eq!(n(&g, P0, "landfall", Some(cradle)), 1, "a legendary land");
    assert_eq!(n(&g, P0, "landfall", Some(basic)), 0, "a basic land");
    assert_eq!(n(&g, P0, "etb", None), 1, "an artifact token that is a counter on the player (Treasure)");
    assert_eq!(n(&g, P0, "dies", None), 1, "one of those put into the graveyard from the battlefield");
    for kind in ["cast", "attack", "landfall"] {
        assert_eq!(n(&g, P0, kind, None), 0, "{kind}: not an entering or leaving");
    }
    assert_eq!(n(&g, P0, "cast", Some(legend)), 0, "a cast is neither");
}

#[test]
fn a_token_that_is_an_artifact_counts_for_gandalf() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let wurm = token(&mut g, P0, 3);
    assert_eq!(total_trigger_copies(&g, P0, "etb", Some(wurm)), 0, "a plain creature token");
    g.perm_mut(wurm).data.set(sim_core::state::DataKey::Artifact, sim_core::state::Val::Bool(true));
    assert_eq!(total_trigger_copies(&g, P0, "etb", Some(wurm)), 1, "an artifact creature token (Wurmcoil's Wurm)");
}

#[test]
fn teysa_and_drivnod_read_none_as_not_a_creature() {
    let mut g = mine();
    perm(&mut g, P0, "Teysa Karlov");
    perm(&mut g, P0, "Drivnod, Carnage Dominus");
    let tok = token(&mut g, P0, 1);
    let ring = perm(&mut g, P0, "Sol Ring");
    // Teysa and Drivnod each add one for a creature dying
    assert_eq!(total_trigger_copies(&g, P0, "dies", Some(tok)), 2, "a creature token dying");
    assert_eq!(total_trigger_copies(&g, P0, "dies", Some(ring)), 0, "an artifact that is not a creature");
    assert_eq!(total_trigger_copies(&g, P0, "dies", None), 0, "an artifact token put into a graveyard (a Treasure)");
}
