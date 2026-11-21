//! Gandalf the White ({3}{W}{W}, Flash): "You may cast legendary spells and artifact spells as though they had flash.
//! If a legendary permanent or an artifact entering or leaving the battlefield causes a triggered ability of a permanent
//! you control to trigger, that ability triggers an additional time." Each expectation follows the Oracle text and the
//! 2023-06-16 rulings (a land is never cast; the grant is for casting only).

mod play_common;
use play_common::*;
use sim_core::ai::brain;
use sim_core::engine::{cast, zones};
use sim_core::human::{human, legal};
use sim_core::state::{Ctx, DataKey, Step, Val};

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
    assert!(sim_core::engine::values::is_artifact_perm(&g, wurm));
    let runes = perm(&mut g, P0, "Mother of Runes");
    assert!(!sim_core::engine::values::is_artifact_perm(&g, runes));
    let ring = perm(&mut g, P0, "Sol Ring");
    assert!(sim_core::engine::values::is_artifact_perm(&g, ring));
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

// ------------------------------------------------------------------ leaving: one additional time
const TEYSA: &str = "Teysa Karlov";
const VONDAM: &str = "Syr Vondam, Sunstar Exemplar";
const DELNEY: &str = "Delney, Streetwise Lookout";

/// the triggers of `src` waiting in the queue (nothing resolves while a resolution is held)
fn waiting(g: &Game, src: PermId) -> usize {
    g.trig_queue.iter().filter(|t| t.src == Some(src)).count()
}

fn tokens_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

/// Syr Vondam with Gandalf, then a creature of yours: (counters on Vondam, life you gained) after `leave` takes it away
fn vondam_sees(leave: fn(&mut Game, PermId), name: &str) -> (i32, i32) {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let v = perm(&mut g, P0, VONDAM);
    let x = perm(&mut g, P0, name);
    let life = g.player(P0).life;
    leave(&mut g, x);
    (counters(&g, v), g.player(P0).life - life)
}

#[test]
fn a_legendary_creature_dying_triggers_vondam_twice() {
    let die = |g: &mut Game, m| zones::die(g, m, "destroy").unwrap();
    assert_eq!(vondam_sees(die, "Tymna the Weaver"), (2, 2), "a legendary creature dies: 1 + Gandalf");
    assert_eq!(vondam_sees(die, "Mother of Runes"), (1, 1), "a plain creature dies");
}

#[test]
fn a_legendary_creature_exiled_triggers_vondam_twice() {
    let exile = |g: &mut Game, m| zones::exile_perm(g, m).unwrap();
    assert_eq!(vondam_sees(exile, "Tymna the Weaver"), (2, 2), "put into exile: 1 + Gandalf");
    assert_eq!(vondam_sees(exile, "Mother of Runes"), (1, 1), "a plain creature exiled");
}

/// Skyclave Apparition that has exiled Phyrexian Arena (mana value 3): the Illusions its owner's opponent gets
fn skyclave_after(g: &mut Game, legendary: bool) -> PermId {
    let arena = perm(g, P1, "Phyrexian Arena");
    let sky = perm(g, P0, "Skyclave Apparition");
    assert!(!g.perm(arena).on_bf, "exiled by the Apparition");
    if legendary {
        g.perm_mut(sky).data.set(DataKey::Legendary, Val::Bool(true));
    }
    sky
}

#[test]
fn a_legendary_permanent_that_leaves_without_dying_triggers_its_own_leaves_ability_twice() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let sky = skyclave_after(&mut g, true);
    let before = tokens_of(&g, P1);
    zones::bounce(&mut g, sky).unwrap();
    assert_eq!(tokens_of(&g, P1) - before, 2, "a legendary Apparition returned to hand: 1 + Gandalf");
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let sky = skyclave_after(&mut g, false);
    let before = tokens_of(&g, P1);
    zones::bounce(&mut g, sky).unwrap();
    assert_eq!(tokens_of(&g, P1) - before, 1, "a plain Apparition");
}

#[test]
fn a_creature_that_dies_triggers_its_own_leaves_ability_twice_with_teysa() {
    let mut g = mine();
    perm(&mut g, P0, TEYSA);
    let sky = skyclave_after(&mut g, false);
    let before = tokens_of(&g, P1);
    zones::die(&mut g, sky, "destroy").unwrap();
    assert_eq!(tokens_of(&g, P1) - before, 2, "a leaves trigger of a dying creature is a dies trigger: 1 + Teysa");
}

#[test]
fn dour_port_mage_draws_twice_for_a_legendary_creature_that_leaves_without_dying() {
    for (name, want) in [("Tymna the Weaver", 2), ("Mother of Runes", 1)] {
        let mut g = mine();
        perm(&mut g, P0, GANDALF);
        let pm = perm(&mut g, P0, "Dour Port-Mage");
        let x = perm(&mut g, P0, name);
        g.resolving = 1;
        zones::bounce(&mut g, x).unwrap();
        assert_eq!(waiting(&g, pm), want, "{name} returned to hand");
        g.trig_queue.clear();
        g.resolving = 0;
    }
}

#[test]
fn chromatic_star_put_into_a_graveyard_draws_twice_with_gandalf() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let star = perm(&mut g, P0, "Chromatic Star");
    let hand = g.player(P0).hand.len();
    zones::die(&mut g, star, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len() - hand, 2, "an artifact put into a graveyard from the battlefield: 1 + Gandalf");
}

#[test]
fn nim_deathmantle_triggers_twice_for_a_creature_dying_with_teysa() {
    let mut g = mine();
    perm(&mut g, P0, TEYSA);
    let nim = perm(&mut g, P0, "Nim Deathmantle");
    let x = perm(&mut g, P0, "Mother of Runes");
    g.resolving = 1;
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!(waiting(&g, nim), 2, "a nontoken creature put into your graveyard: 1 + Teysa");
    g.trig_queue.clear();
    g.resolving = 0;
}

#[test]
fn titania_makes_two_elementals_for_a_legendary_land_put_into_a_graveyard_with_gandalf() {
    for (land, want) in [("Gaea's Cradle", 2), ("Plains", 1)] {
        let mut g = mine();
        perm(&mut g, P0, GANDALF);
        perm(&mut g, P0, "Titania, Protector of Argoth");
        let l = lands(&mut g, P0, land, 1, false)[0];
        let before = tokens_of(&g, P0);
        sim_core::impls::fixes::destroy_land(&mut g, P0, l).unwrap();
        assert_eq!(tokens_of(&g, P0) - before, want, "{land} destroyed");
    }
}

#[test]
fn wurmcoil_engine_makes_four_wurms_when_it_dies_with_gandalf_or_teysa() {
    for copier in [GANDALF, TEYSA] {
        let mut g = mine();
        perm(&mut g, P0, copier);
        let w = perm(&mut g, P0, "Wurmcoil Engine");
        let before = tokens_of(&g, P0);
        zones::die(&mut g, w, "destroy").unwrap();
        assert_eq!(tokens_of(&g, P0) - before, 4, "an artifact creature dies: 1 + {copier}");
    }
}

#[test]
fn an_opponents_massacre_wurm_drains_twice_for_a_legendary_creature_with_its_controllers_gandalf() {
    for (name, want) in [("Tymna the Weaver", 4), ("Mother of Runes", 2)] {
        let mut g = mine();
        perm(&mut g, P1, "Massacre Wurm");
        perm(&mut g, P1, GANDALF);
        let x = perm(&mut g, P0, name);
        let life = g.player(P0).life;
        zones::die(&mut g, x, "destroy").unwrap();
        assert_eq!(life - g.player(P0).life, want, "{name} dies");
    }
}

#[test]
fn a_sacrifice_trigger_is_not_doubled_but_one_for_an_artifact_put_into_a_graveyard_is() {
    // Korvold: "Whenever you sacrifice a permanent" is not caused by the permanent leaving
    let mut g = mine();
    let k = perm(&mut g, P0, "Korvold, Fae-Cursed King");
    perm(&mut g, P0, GANDALF);
    let x = perm(&mut g, P0, "Tymna the Weaver");
    zones::die(&mut g, x, "sac").unwrap();
    assert_eq!(counters(&g, k), 1, "one Korvold trigger for a sacrificed legendary creature");
    // Disciple of the Vault: "Whenever an artifact is put into a graveyard from the battlefield"; a Treasure is one
    let drains = |copier: &str| {
        let mut g = mine();
        perm(&mut g, P0, "Disciple of the Vault");
        perm(&mut g, P0, copier);
        let opp = g.player(P1).life + g.player(P2).life;
        sim_core::engine::hooks::fire_trigger(
            &mut g,
            sim_core::hooks::Event::Sacrifice,
            sim_core::hooks::Call::Sacrifice { p: P0, what: sim_core::hooks::Sacrificed::Token("Treasure") },
        )
        .unwrap();
        opp - (g.player(P1).life + g.player(P2).life)
    };
    assert_eq!(drains(GANDALF), 2, "a Treasure sacrificed: 1 + Gandalf");
    assert_eq!(drains(TEYSA), 1, "Teysa answers to a creature dying, not to a Treasure");
}

// ------------------------------------------------------------------ Delney leaving with the cause
#[test]
fn a_source_that_stays_hears_delney_die_as_delney_made_it_trigger_twice() {
    let mut g = mine();
    perm(&mut g, P0, "Elas il-Kor, Sadistic Pilgrim");
    let d = perm(&mut g, P0, DELNEY);
    let life = g.player(P1).life;
    zones::die(&mut g, d, "destroy").unwrap();
    assert_eq!(
        life - g.player(P1).life,
        2,
        "Elas (power 2) triggers for Delney dying, and Delney's ability looks back"
    );
}

#[test]
fn a_source_that_stays_hears_a_creature_die_once_without_delney() {
    let mut g = mine();
    perm(&mut g, P0, "Elas il-Kor, Sadistic Pilgrim");
    let x = perm(&mut g, P0, "Mother of Runes");
    let life = g.player(P1).life;
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!(life - g.player(P1).life, 1, "the control");
}

// ------------------------------------------------------------------ a copier that leaves with the cause
fn indestructible(g: &mut Game, m: PermId) {
    g.perm_mut(m).eot_kw.push("indestructible");
}

fn wipe(g: &mut Game) {
    sim_core::engine::removal::apply_wipe(g, P0, "destroy", None, &Ctx::default()).unwrap();
}

#[test]
fn gandalf_dying_alone_doubles_the_triggers_he_causes() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    indestructible(&mut g, v);
    let gan = perm(&mut g, P0, GANDALF);
    let life = g.player(P0).life;
    zones::die(&mut g, gan, "destroy").unwrap();
    assert!(!g.perm(gan).on_bf);
    assert_eq!(counters(&g, v), 2, "Gandalf is a legendary creature dying, and his ability looks back");
    assert_eq!(g.player(P0).life - life, 2);
}

#[test]
fn gandalf_and_a_legendary_creature_in_one_wipe_each_cause_two_triggers_in_either_order() {
    for gandalf_first in [true, false] {
        let mut g = mine();
        let v = perm(&mut g, P0, VONDAM);
        indestructible(&mut g, v);
        let names = if gandalf_first { [GANDALF, "Tymna the Weaver"] } else { ["Tymna the Weaver", GANDALF] };
        for n in names {
            perm(&mut g, P0, n);
        }
        let life = g.player(P0).life;
        wipe(&mut g);
        assert_eq!(counters(&g, v), 4, "Gandalf first: {gandalf_first}: two creatures, two triggers each");
        assert_eq!(g.player(P0).life - life, 4, "Gandalf first: {gandalf_first}");
    }
}

#[test]
fn teysa_and_a_creature_in_one_wipe_each_cause_two_triggers_in_either_order() {
    for teysa_first in [true, false] {
        let mut g = mine();
        let v = perm(&mut g, P0, VONDAM);
        indestructible(&mut g, v);
        let names = if teysa_first { [TEYSA, "Mother of Runes"] } else { ["Mother of Runes", TEYSA] };
        for n in names {
            perm(&mut g, P0, n);
        }
        wipe(&mut g);
        assert_eq!(counters(&g, v), 4, "Teysa first: {teysa_first}");
    }
}

#[test]
fn a_departed_vondam_hears_a_legendary_creature_die_twice_with_gandalf() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    perm(&mut g, P0, "Tymna the Weaver");
    let gan = perm(&mut g, P0, GANDALF);
    indestructible(&mut g, gan);
    let life = g.player(P0).life;
    wipe(&mut g);
    assert!(!g.perm(v).on_bf && g.perm(gan).on_bf);
    assert_eq!(g.player(P0).life - life, 2, "Vondam left with Tymna: 1 + Gandalf");
}

#[test]
fn a_permanent_that_phases_out_or_a_copier_without_abilities_causes_no_additional_trigger() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let v = perm(&mut g, P0, VONDAM);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).phased = true;
    assert_eq!(counters(&g, v), 0, "phasing out is not leaving the battlefield");
    g.perm_mut(t).phased = false;
    let gan = *g.player(P0).perms.iter().find(|&&m| g.perm(m).name == GANDALF).unwrap();
    g.perm_mut(gan).neutered = true;
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(counters(&g, v), 1, "a Gandalf with abilities off adds nothing");
}

#[test]
fn an_opponents_gandalf_adds_nothing_to_your_leave_triggers() {
    let mut g = mine();
    perm(&mut g, P1, GANDALF);
    let v = perm(&mut g, P0, VONDAM);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(counters(&g, v), 1);
}

// ------------------------------------------------------------------ the other leave paths
#[test]
fn angelic_sleuth_investigates_twice_for_a_legendary_permanent_with_counters_that_leaves() {
    for (name, want) in [(Y, 2), ("Mother of Runes", 1)] {
        let mut g = mine();
        perm(&mut g, P0, GANDALF);
        perm(&mut g, P0, "Angelic Sleuth");
        let x = perm(&mut g, P0, name);
        g.perm_mut(x).plus = 1;
        let clues = g.player(P0).clues;
        zones::bounce(&mut g, x).unwrap();
        assert_eq!(g.player(P0).clues - clues, want, "{name} with a +1/+1 counter returned to hand");
    }
}

#[test]
fn vondam_exiled_with_power_four_destroys_two_permanents_with_gandalf() {
    for gandalf in [true, false] {
        let mut g = mine();
        if gandalf {
            perm(&mut g, P0, GANDALF);
        }
        let v = perm(&mut g, P0, VONDAM);
        g.perm_mut(v).plus = 3;
        for _ in 0..2 {
            perm(&mut g, P1, "Serra Angel");
        }
        zones::exile_perm(&mut g, v).unwrap();
        let angels = g.player(P1).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name == "Serra Angel").count();
        assert_eq!(angels, if gandalf { 0 } else { 1 }, "Gandalf: {gandalf}: one destroy per instance");
    }
}

#[test]
fn a_compiled_dies_trigger_happens_twice_for_a_legendary_creature_with_gandalf() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Elas il-Kor, Sadistic Pilgrim");
    let t = perm(&mut g, P0, "Tymna the Weaver");
    let life = g.player(P1).life;
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(life - g.player(P1).life, 2, "another creature you control dies: 1 + Gandalf");
}

// ------------------------------------------------------------------ more of a copier that leaves with the cause
const DRIVNOD: &str = "Drivnod, Carnage Dominus";

#[test]
fn drivnod_dying_alone_doubles_the_triggers_he_causes() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    indestructible(&mut g, v);
    let d = perm(&mut g, P0, DRIVNOD);
    zones::die(&mut g, d, "destroy").unwrap();
    assert!(!g.perm(d).on_bf);
    assert_eq!(counters(&g, v), 2, "Drivnod is a creature dying, and his ability looks back");
}

#[test]
fn gandalf_exiled_alone_doubles_the_triggers_his_leaving_causes() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    let gan = perm(&mut g, P0, GANDALF);
    zones::exile_perm(&mut g, gan).unwrap();
    assert!(!g.perm(gan).on_bf);
    assert_eq!(counters(&g, v), 2, "a legendary creature put into exile: 1 + Gandalf, who leaves with it");
}

#[test]
fn a_second_gandalf_adds_to_the_triggers_a_dying_gandalf_causes() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    indestructible(&mut g, v);
    let first = perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, GANDALF);
    zones::die(&mut g, first, "destroy").unwrap();
    assert_eq!(counters(&g, v), 3, "1 + the Gandalf that stays + the Gandalf that dies");
}

#[test]
fn a_copier_that_left_earlier_adds_nothing_to_a_later_leaving() {
    let mut g = mine();
    let v = perm(&mut g, P0, VONDAM);
    indestructible(&mut g, v);
    let gan = perm(&mut g, P0, GANDALF);
    zones::die(&mut g, gan, "destroy").unwrap();
    assert_eq!(counters(&g, v), 2);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(counters(&g, v), 3, "Gandalf is gone: one more trigger for Tymna, not two");
}

#[test]
fn a_ring_bearer_that_leaves_is_still_legendary_for_gandalf() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    let v = perm(&mut g, P0, VONDAM);
    let bearer = perm(&mut g, P0, "Mother of Runes");
    g.player_mut(P0).ring_level = 1;
    g.player_mut(P0).ring_bearer = Some(bearer);
    assert!(sim_core::impls::mine::is_legendary(&g, bearer));
    zones::die(&mut g, bearer, "destroy").unwrap();
    assert_eq!(counters(&g, v), 2, "the Ring-bearer was legendary as it died: 1 + Gandalf");
}

#[test]
fn nothing_is_remembered_about_a_leaving_permanent_without_a_copier() {
    let mut g = mine();
    perm(&mut g, P0, VONDAM);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    let x = perm(&mut g, P0, "Mother of Runes");
    zones::die(&mut g, t, "destroy").unwrap();
    wipe(&mut g);
    assert!(!g.perm(x).on_bf);
    for p in g.perms.iter() {
        let d = format!("{:?}", p.data);
        assert!(!d.contains("LkiCopier") && !d.contains("LkiLegArt"), "{}: {d}", p.name);
    }
}

// ------------------------------------------------------------------ a leave hook that is not a trigger is not copied
#[test]
fn a_hook_that_only_keeps_count_of_deaths_is_not_copied_by_teysa() {
    // Mahadi, Emporium Master counts the creatures that died; the Treasures are made at the end step
    let mut g = mine();
    perm(&mut g, P0, TEYSA);
    perm(&mut g, P0, "Mahadi, Emporium Master");
    let x = perm(&mut g, P0, "Mother of Runes");
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!(g.deaths_turn.map(|d| d.1), Some(1), "one creature died: bookkeeping, not a trigger");
}

// ------------------------------------------------------------------ a leave trigger that returns a card returns it once
fn on_battlefield(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name == name).count()
}

#[test]
fn oblivion_ring_that_leaves_twice_over_returns_its_card_once() {
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P1, "Phyrexian Arena");
    let ring = perm(&mut g, P0, "Oblivion Ring");
    assert_eq!(on_battlefield(&g, P1, "Phyrexian Arena"), 0, "exiled by Oblivion Ring");
    g.perm_mut(ring).data.set(DataKey::Legendary, Val::Bool(true));
    zones::bounce(&mut g, ring).unwrap();
    assert_eq!(on_battlefield(&g, P1, "Phyrexian Arena"), 1, "its leaves trigger happened twice: one card, once");
    assert!(g.player(P1).exile.is_empty());
}

#[test]
fn angel_of_serenity_that_dies_twice_over_returns_each_card_once() {
    let mut g = mine();
    perm(&mut g, P0, TEYSA);
    for _ in 0..2 {
        perm(&mut g, P1, "Serra Angel");
    }
    let hand = g.player(P1).hand.len();
    let a = perm(&mut g, P0, "Angel of Serenity");
    assert_eq!(on_battlefield(&g, P1, "Serra Angel"), 0, "exiled by Angel of Serenity");
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(g.player(P1).hand.len() - hand, 2, "returned to their owner's hand once each");
}

#[test]
fn nine_lives_familiar_that_dies_twice_over_returns_once() {
    let mut g = mine();
    perm(&mut g, P0, TEYSA);
    let f = perm(&mut g, P0, "Nine-Lives Familiar");
    g.perm_mut(f).data.set(DataKey::Revival, Val::Int(3));
    zones::die(&mut g, f, "destroy").unwrap();
    sim_core::impls::t45_urza_yawgmoth::revival_returns(&mut g).unwrap();
    assert_eq!(on_battlefield(&g, P0, "Nine-Lives Familiar"), 1, "the second instance finds the card gone");
}

// ------------------------------------------------------------------ landfall carries the land
/// p plays this land card from the hand: the land event and landfall come with the real play
fn play_land(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    sim_core::engine::turn::play_land_card(g, p, c, "plays").unwrap();
    *g.player(p).lands.last().unwrap()
}

const PANHARMONICON: &str = "Panharmonicon";

#[test]
fn a_legendary_land_or_an_artifact_land_makes_landfall_trigger_twice_with_gandalf() {
    for (land, want) in [("Gaea's Cradle", 2), ("Seat of the Synod", 2), ("Plains", 1)] {
        let mut g = mine();
        perm(&mut g, P0, GANDALF);
        perm(&mut g, P0, "Scute Swarm"); // a landfall hook: a 1/1 Insect (fewer than six lands)
        let before = tokens_of(&g, P0);
        play_land(&mut g, P0, land);
        assert_eq!(tokens_of(&g, P0) - before, want, "{land} enters");
    }
}

#[test]
fn a_compiled_landfall_ability_happens_twice_for_a_legendary_land_with_gandalf() {
    for (land, want) in [("Gaea's Cradle", 2), ("Plains", 1)] {
        let mut g = mine();
        perm(&mut g, P0, GANDALF);
        perm(&mut g, P0, "Icetill Explorer"); // compiled: landfall, mill a card
        let gy = g.player(P0).gy.len();
        play_land(&mut g, P0, land);
        assert_eq!(g.player(P0).gy.len() - gy, want, "{land} enters");
    }
}

#[test]
fn panharmonicon_adds_one_to_landfall_for_a_creature_land_and_an_artifact_land() {
    for (land, want) in [("Seat of the Synod", 2), ("Dryad Arbor", 2), ("Forest", 1)] {
        let mut g = mine();
        perm(&mut g, P0, PANHARMONICON);
        perm(&mut g, P0, "Scute Swarm");
        let before = tokens_of(&g, P0);
        play_land(&mut g, P0, land);
        assert_eq!(tokens_of(&g, P0) - before, want, "{land} enters: 1 + Panharmonicon for a creature or an artifact");
    }
}

#[test]
fn every_copier_adds_to_the_landfall_of_a_creature_land() {
    let mut g = mine();
    for n in [PANHARMONICON, "Starfield Vocalist", "Starfield Vocalist", "Ancient Greenwarden", DELNEY] {
        perm(&mut g, P0, n);
    }
    let throne = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(throne).data.set(DataKey::Ctype, Val::Str("insect"));
    perm(&mut g, P0, "Scute Swarm");
    let before = tokens_of(&g, P0);
    play_land(&mut g, P0, "Dryad Arbor");
    assert_eq!(tokens_of(&g, P0) - before, 7, "1 + Panharmonicon + two Vocalists + Greenwarden + Delney + Throne");
}

// ------------------------------------------------------------------ Hushbringer
const HUSH: &str = "Hushbringer";

#[test]
fn hushbringer_blanks_the_landfall_of_a_creature_land_but_not_of_a_basic_land() {
    let mut g = mine();
    for n in [PANHARMONICON, "Starfield Vocalist", "Ancient Greenwarden", DELNEY, HUSH] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, "Scute Swarm");
    let before = tokens_of(&g, P0);
    play_land(&mut g, P0, "Dryad Arbor");
    assert_eq!(tokens_of(&g, P0) - before, 0, "a creature entering causes no ability to trigger, however many copiers");
    let mut g = mine();
    perm(&mut g, P1, HUSH);
    perm(&mut g, P0, "Scute Swarm");
    let before = tokens_of(&g, P0);
    play_land(&mut g, P0, "Forest");
    assert_eq!(tokens_of(&g, P0) - before, 1, "the control: a land that is not a creature");
}

#[test]
fn a_creature_put_onto_the_battlefield_under_hushbringer_still_enters_as_its_card_says() {
    let mut g = mine();
    perm(&mut g, P1, HUSH);
    let f = perm(&mut g, P0, "Freestrider Commando");
    assert_eq!(counters(&g, f), 2, "it enters with two +1/+1 counters unless it was cast: not a trigger");
    let r = perm(&mut g, P0, "Phyrexian Revoker");
    assert!(g.perm(r).data.get(DataKey::Imprint).is_some(), "the name is chosen as it enters");
}

#[test]
fn a_creature_entering_under_hushbringer_sets_off_nothing() {
    let mut g = mine();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P1, HUSH);
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 0, "Yoshimaru gets nothing for a legendary creature entering");
}

#[test]
fn a_creature_that_dies_under_hushbringer_causes_no_leaves_trigger() {
    let mut g = mine();
    let sky = skyclave_after(&mut g, false);
    perm(&mut g, P0, "Angelic Sleuth");
    g.perm_mut(sky).plus = 1;
    perm(&mut g, P1, HUSH);
    let (tokens, clues) = (tokens_of(&g, P1), g.player(P0).clues);
    zones::die(&mut g, sky, "destroy").unwrap();
    assert_eq!(tokens_of(&g, P1) - tokens, 0, "Skyclave Apparition's leaves ability is caused by it dying");
    assert_eq!(g.player(P0).clues - clues, 0, "and so is Angelic Sleuth's, for a creature with a counter");
}

#[test]
fn a_creature_that_leaves_without_dying_under_hushbringer_still_causes_its_leaves_trigger() {
    let mut g = mine();
    let sky = skyclave_after(&mut g, false);
    perm(&mut g, P1, HUSH);
    let before = tokens_of(&g, P1);
    zones::bounce(&mut g, sky).unwrap();
    assert_eq!(tokens_of(&g, P1) - before, 1, "returned to hand: not a death");
}

#[test]
fn creature_tokens_entering_under_hushbringer_give_cathars_crusade_nothing() {
    let mut g = mine();
    perm(&mut g, P0, "Cathars' Crusade");
    let runes = perm(&mut g, P0, "Mother of Runes");
    perm(&mut g, P1, HUSH);
    let before = counters(&g, runes);
    zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap();
    assert_eq!(counters(&g, runes), before, "creature tokens entering cause no ability to trigger");
}

#[test]
fn tishanas_tidebinder_has_nothing_to_counter_under_hushbringer() {
    let mut g = table(&["seph", "cloud", "sauron"]);
    perm(&mut g, P2, HUSH);
    let tide = hand(&mut g, P1, &["Tishana's Tidebinder"])[0];
    lands(&mut g, P1, "Island", 3, false);
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    assert!(!g.perm(archon).neutered, "its enters trigger never triggers, so nothing is countered");
    assert!(g.player(P1).hand.contains(&tide), "Tidebinder stays in the hand");
}

// ------------------------------------------------------------------ the sacrifice routes
#[test]
fn a_treasure_spent_for_mana_is_an_artifact_put_into_a_graveyard_for_gandalf_but_not_for_teysa() {
    use sim_core::cards::Colors;
    use sim_core::engine::mana::{self, Source, Unit};
    let drains = |copier: &str| {
        let mut g = mine();
        perm(&mut g, P0, "Disciple of the Vault");
        perm(&mut g, P0, copier);
        g.player_mut(P0).treasures = 1;
        let opp = g.player(P1).life + g.player(P2).life;
        mana::spend_unit(&mut g, P0, Unit::free(Source::Treasure, Colors::NONE, 1), 1, "").unwrap();
        assert_eq!(g.player(P0).treasures, 0, "the Treasure was sacrificed");
        opp - (g.player(P1).life + g.player(P2).life)
    };
    assert_eq!(drains(GANDALF), 2, "1 + Gandalf");
    assert_eq!(drains(TEYSA), 1, "a Treasure is no creature");
}

#[test]
fn a_hook_that_only_notes_the_wipe_a_creature_dies_in_is_not_copied_by_teysa() {
    // Liesa, Forgotten Archangel's own death only records the wipe, for the creatures that die with her
    let mut g = mine();
    perm(&mut g, P0, "Liesa, Forgotten Archangel");
    perm(&mut g, P0, TEYSA);
    wipe(&mut g);
    assert_eq!(g.t12_batch.len(), 1, "one record of the wipe, not one per additional time");
}

// ------------------------------------------------------------------ the land a leave event is about
#[test]
fn a_leave_event_fired_while_another_is_handled_is_not_about_the_first_land() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static NESTED: AtomicBool = AtomicBool::new(false);
    let mut g = mine();
    perm(&mut g, P0, GANDALF);
    perm(&mut g, P0, "Titania, Protector of Argoth");
    // Titania's hook counts a life point each time it happens, and once, as it happens, fires the event for a basic land
    // card that no permanent left from
    let mut registry = sim_core::hooks::Registry::new(&g.db);
    for d in g.db.iter() {
        let c = card(&g, &d.name);
        if let Some(i) = g.registry.get(c) {
            registry.on(&g.db, &d.name, *i).unwrap();
        }
    }
    registry.card(&g.db, "Titania, Protector of Argoth").unwrap().land_gy = Some(|g, _src, p, _c| {
        g.player_mut(p).life += 1;
        if !NESTED.swap(true, Ordering::SeqCst) {
            let plains = card(g, "Plains");
            sim_core::engine::hooks::fire_trigger(
                g,
                sim_core::hooks::Event::LandGy,
                sim_core::hooks::Call::Cards { p, cards: vec![plains] },
            )?;
        }
        Ok(())
    });
    g.registry = std::sync::Arc::new(registry);
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, false)[0];
    let life = g.player(P0).life;
    sim_core::impls::fixes::destroy_land(&mut g, P0, cradle).unwrap();
    assert_eq!(g.player(P0).life - life, 3, "a legendary land: twice; the nested event is about no permanent: once");
}
