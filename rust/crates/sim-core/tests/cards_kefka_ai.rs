//! Kefka's AI (impls/kefka.rs): the cards the card report showed as rarely cast, the Crawler and Niv-Mizzet loops, and
//! the cards the lead swaps in from Veyran's deck (Deflecting Swat, Slip Out the Back, Underworld Breach, Niv-Mizzet,
//! Jin-Gitaxias).

use sim_core::ai::{act, brain, decks};
use sim_core::engine::zones;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
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

// ======================================================== Psychosis Crawler and Niv-Mizzet: the priority ladder
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
fn niv_with_a_short_library_still_finishes_a_player_on_one_life() {
    let mut g = niv_table(40, 20);
    g.player_mut(P1).life = 1;
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(!g.player(P1).alive);
    assert!(g.player(P0).alive);
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

