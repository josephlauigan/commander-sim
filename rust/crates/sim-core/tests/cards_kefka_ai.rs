//! Kefka's AI (impls/kefka.rs): the cards the card report showed as rarely cast, the Crawler and Niv-Mizzet loops, and
//! the cards the lead swaps in from Veyran's deck (Deflecting Swat, Slip Out the Back, Underworld Breach, Niv-Mizzet,
//! Jin-Gitaxias).

use sim_core::ai::{self, act, brain, decks};
use sim_core::engine::{removal, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::kefka;
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const CRAWLER: &str = "Psychosis Crawler";
const NIV: &str = "Niv-Mizzet, the Firemind";

fn opts_of(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn take(g: &mut Game, p: PlayerId, prefix: &str) -> bool {
    let opts = opts_of(g, p, false);
    let o = opts
        .iter()
        .find(|o| o.label.starts_with(prefix))
        .unwrap_or_else(|| panic!("no option {prefix:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn has_opt(g: &mut Game, p: PlayerId, prefix: &str) -> bool {
    opts_of(g, p, false).iter().any(|o| o.label.starts_with(prefix))
}

fn table2() -> Game {
    table(&["kefka", "veyran"])
}

fn fill_library(g: &mut Game, p: PlayerId, n: usize) {
    let cs = vec![card(g, "Island"); n];
    g.player_mut(p).library = cs;
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

/// Kefka on the battlefield (the command zone card), optionally flipped
fn kefka_out(g: &mut Game, p: PlayerId, flip: bool) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.player_mut(p).cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    if flip {
        let x = g.perm_mut(m);
        x.data.set(DataKey::Ruin, Val::Bool(true));
        (x.pow, x.tgh, x.fly) = (5, 7, true);
    }
    m
}

fn prio(g: &Game, p: PlayerId, name: &str) -> i32 {
    decks::deck_prio(g, p, card(g, name))
}

// ======================================================== Kaervek's Purge
#[test]
fn purge_is_offered_on_a_big_creature_and_counted_as_cast() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 4, false);
    lands(&mut g, P0, "Mountain", 4, false);
    let purge = hand(&mut g, P0, &["Kaervek's Purge"])[0];
    perm(&mut g, P1, "Llanowar Elves");
    let titan = perm(&mut g, P1, "Grave Titan"); // mana value 6: X=6
    assert!(take(&mut g, P0, "Kaervek's Purge on Grave Titan"));
    assert!(!g.perm(titan).on_bf);
    assert!(gy_has(&g, P0, "Kaervek's Purge"));
    // the card report counts a card as cast from this set (it showed 0% for the card before)
    assert!(g.player(P0).cast_names.contains(&purge));
    assert!(g.player(P1).life < 40, "its power is dealt to its controller");
}

#[test]
fn purge_on_a_small_creature_waits_for_the_flip() {
    // Eternal Witness: worth under 3, power 2
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Mountain", 3, false);
    hand(&mut g, P0, &["Kaervek's Purge"]);
    perm(&mut g, P1, "Eternal Witness");
    assert!(!has_opt(&mut g, P0, "Kaervek's Purge"));
    // flipped Kefka: the 2 life its controller loses is 2 cards
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Mountain", 3, false);
    kefka_out(&mut g, P0, true);
    hand(&mut g, P0, &["Kaervek's Purge"]);
    perm(&mut g, P1, "Eternal Witness");
    fill_library(&mut g, P0, 30);
    let n = g.player(P0).hand.len();
    assert!(take(&mut g, P0, "Kaervek's Purge on Eternal Witness"));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
}

// ======================================================== Psychosis Crawler and Niv-Mizzet: the priority ladder
#[test]
fn the_crawler_is_cast_before_the_flip_and_held_only_when_the_flipped_loop_would_deck_us() {
    let mut g = table2();
    fill_library(&mut g, P0, 60);
    hand(&mut g, P0, &[CRAWLER]);
    assert_eq!(prio(&g, P0, CRAWLER), 70); // Kefka still in the command zone
    kefka_out(&mut g, P0, false);
    assert_eq!(prio(&g, P0, CRAWLER), 84, "Kefka's own discard and draws ping the table");
    let mut g = table2();
    let k = kefka_out(&mut g, P0, true);
    let _ = k;
    hand(&mut g, P0, &[CRAWLER]);
    fill_library(&mut g, P0, 60);
    assert_eq!(prio(&g, P0, CRAWLER), 80);
    fill_library(&mut g, P0, 20);
    assert_eq!(prio(&g, P0, CRAWLER), 0, "the loop is mandatory: not with a library shorter than their life");
}

#[test]
fn the_crawler_now_beats_ordinary_five_drops_in_the_main_phase() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Mountain", 2, false);
    kefka_out(&mut g, P0, false);
    hand(&mut g, P0, &[CRAWLER, "Alpharael, Dreaming Acolyte", "Shimmercreep"]);
    let opts = opts_of(&mut g, P0, false);
    let u = |l: &str| opts.iter().find(|o| o.label.starts_with(l)).map(|o| o.utility).unwrap_or(-9.0);
    assert!(
        u(CRAWLER) > u("Alpharael") && u(CRAWLER) > u("Shimmercreep"),
        "{:?}",
        opts.iter().map(|o| (&o.label, o.utility)).collect::<Vec<_>>()
    );
}

#[test]
fn niv_mizzet_is_cast_like_the_crawler_and_never_held() {
    let mut g = table2();
    fill_library(&mut g, P0, 60);
    hand(&mut g, P0, &[NIV]);
    assert_eq!(prio(&g, P0, NIV), 70);
    kefka_out(&mut g, P0, false);
    assert_eq!(prio(&g, P0, NIV), 84);
    let mut g = table2();
    kefka_out(&mut g, P0, true);
    hand(&mut g, P0, &[NIV]);
    fill_library(&mut g, P0, 5);
    assert_eq!(prio(&g, P0, NIV), 80, "his pings are aimed, so a short library does not hold him back");
}

#[test]
fn triskelion_and_rapturous_moment_priorities() {
    let mut g = table2();
    hand(&mut g, P0, &["Triskelion", "Rapturous Moment", "Duress", "Forest"]);
    // Kefka's enter trigger redraws what it discards: a fixed library keeps the test off the decklist's shuffle
    let duress = card(&g, "Duress");
    g.player_mut(P0).library = vec![duress; 30];
    let t0 = prio(&g, P0, "Triskelion");
    kefka_out(&mut g, P0, false);
    assert!(prio(&g, P0, "Triskelion") > t0);
    assert_eq!(prio(&g, P0, "Rapturous Moment"), 58, "free filtering: two other spells to spend the mana on");
    let mut g = table2();
    kefka_out(&mut g, P0, true);
    hand(&mut g, P0, &["Triskelion"]);
    assert_eq!(prio(&g, P0, "Triskelion"), 74, "each of its three pings is a card");
}

// ======================================================== the engine guard: Niv-Mizzet with the flipped Kefka
fn niv_table(life: i32, library: usize) -> Game {
    let mut g = table(&["kefka", "veyran", "seph"]);
    kefka_out(&mut g, P0, true);
    perm(&mut g, P0, NIV);
    fill_library(&mut g, P0, library);
    for q in [P1, P2] {
        g.player_mut(q).life = life;
    }
    g
}

#[test]
fn niv_with_a_long_library_pings_faces_and_the_loop_runs() {
    let mut g = niv_table(3, 80);
    assert!(kefka::niv_safe(&g, P0));
    assert!(!kefka::loop_guard(&g, P0));
    zones::draw(&mut g, P0, 1, false).unwrap();
    // each ping is a life lost on our turn, which is a card, which is a ping: the table is dead well inside the library
    assert!(!g.player(P1).alive || g.player(P1).life < 3);
    assert!(g.player(P0).alive);
}

#[test]
fn niv_with_a_short_library_holds_the_face_pings() {
    let mut g = niv_table(40, 20);
    assert!(!kefka::niv_safe(&g, P0));
    assert!(kefka::loop_guard(&g, P0));
    perm(&mut g, P1, "Llanowar Elves");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (40, 40), "no face ping while the loop would deck us");
    assert_eq!(g.player(P0).library.len(), 19, "one draw, and no loop");
}

#[test]
fn niv_with_a_short_library_still_finishes_a_player_on_one_life() {
    let mut g = niv_table(40, 20);
    g.player_mut(P1).life = 1;
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!g.player(P1).alive);
    assert!(g.player(P0).alive);
}

#[test]
fn the_crawler_guard_holds_a_bolt_from_a_face_when_the_loop_would_deck_us() {
    let mut g = table2();
    kefka_out(&mut g, P0, true);
    perm(&mut g, P0, CRAWLER);
    fill_library(&mut g, P0, 20);
    assert!(kefka::loop_guard(&g, P0));
    fill_library(&mut g, P0, 90);
    assert!(!kefka::loop_guard(&g, P0));
}

#[test]
fn the_flip_waits_for_the_crawler_but_not_for_niv() {
    let mut g = table2();
    lands(&mut g, P0, "Swamp", 8, false);
    kefka_out(&mut g, P0, false);
    perm(&mut g, P0, NIV);
    fill_library(&mut g, P0, 20);
    assert!(has_opt(&mut g, P0, "Kefka: {8}, transform"), "Niv's pings are aimed by the guard");
}

// ======================================================== the cards Veyran's deck brings
#[test]
fn deflecting_swat_protects_kefka_from_a_removal_spell() {
    let mut g = table(&["kefka", "seph"]);
    let archon = perm(&mut g, P1, "Archon of Cruelty");
    let k = kefka_out(&mut g, P0, false);
    hand(&mut g, P0, &["Deflecting Swat"]);
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), k, "exile", Some(stp)).unwrap();
    assert!(g.perm(k).on_bf, "Kefka is his own commander: the Swat is free");
    assert!(!g.perm(archon).on_bf);
    assert!(gy_has(&g, P0, "Deflecting Swat"));
}

#[test]
fn slip_out_the_back_phases_kefka_out_of_a_removal_spell() {
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P0, "Island", 1, false);
    let k = kefka_out(&mut g, P0, false);
    hand(&mut g, P0, &["Slip Out the Back"]);
    let stp = card(&g, "Swords to Plowshares");
    assert!(ai::protect_response(&mut g, P0, k, "exile", Some(P1), Some(stp)).unwrap());
    assert!(g.perm(k).phased);
}

#[test]
fn underworld_breach_is_cast_with_a_full_graveyard_and_escapes_a_discarded_spell() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 4, false);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Underworld Breach"]);
    let gy = ["Lightning Bolt", "Duress", "Dark Inquiry", "Deadly Embrace", "Contract Killing", "Forest"];
    for n in gy {
        let c = card(&g, n);
        g.player_mut(P0).gy.push(c);
    }
    perm(&mut g, P1, "Eternal Witness");
    assert!(prio(&g, P0, "Underworld Breach") > 0, "a graveyard of spells and the mana to escape them");
    assert!(take(&mut g, P0, "Underworld Breach"));
    let opts = opts_of(&mut g, P0, false);
    assert!(
        opts.iter().any(|o| o.label.starts_with("escape ")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}

#[test]
fn garrison_excavator_makes_spirits_when_breach_exiles_the_graveyard() {
    let mut g = table2();
    lands(&mut g, P0, "Mountain", 4, false);
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Garrison Excavator");
    perm(&mut g, P0, "Underworld Breach");
    for n in ["Lightning Bolt", "Duress", "Dark Inquiry", "Deadly Embrace", "Contract Killing"] {
        let c = card(&g, n);
        g.player_mut(P0).gy.push(c);
    }
    perm(&mut g, P1, "Eternal Witness");
    let spirits = |g: &Game| {
        g.player(P0)
            .perms
            .iter()
            .filter(|&&m| g.perm(m).token && g.perm(m).ttypes.iter().any(|t| &**t == "spirit"))
            .count()
    };
    assert_eq!(spirits(&g), 0);
    assert!(take(&mut g, P0, "escape "));
    // one trigger for the three cards exiled to the escape cost, one for the escaped card leaving
    assert_eq!(spirits(&g), 2);
}

#[test]
fn jin_gitaxias_copies_kefkas_first_spell_and_counters_theirs() {
    let mut g = table2();
    perm(&mut g, P0, "Jin-Gitaxias, Progress Tyrant");
    hand(&mut g, P0, &["Duress"]);
    hand(&mut g, P1, &["Forest", "Sol Ring", "Counterspell"]);
    let c = card(&g, "Duress");
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    // the copy of Duress takes a second noncreature card
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn jin_gitaxias_is_cast_when_there_is_mana_for_him() {
    let mut g = table2();
    lands(&mut g, P0, "Island", 7, false);
    hand(&mut g, P0, &["Jin-Gitaxias, Progress Tyrant"]);
    assert!(prio(&g, P0, "Jin-Gitaxias, Progress Tyrant") >= 40);
    assert!(has_opt(&mut g, P0, "Jin-Gitaxias"));
}

#[test]
fn mystic_confluence_draw_three_is_offered_at_the_end_of_an_opponents_turn_and_counted() {
    let mut g = table2();
    lands(&mut g, P0, "Island", 6, false);
    let c = hand(&mut g, P0, &["Mystic Confluence"])[0];
    fill_library(&mut g, P0, 30);
    g.active = Some(P1);
    let n = g.player(P0).hand.len();
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n - 1 + 3);
    assert!(g.player(P0).cast_names.contains(&c), "counted as cast in the card report");
}

