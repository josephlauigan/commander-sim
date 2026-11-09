//! Tests for the cloud-aifix3 task: one section per fix.

use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P1).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

// ---- F2: Stormfront Riders returns Cloud like any creature (the deck-gated spare was deleted with the hand AI)
#[test]
fn riders_still_returns_a_bare_cloud_and_other_decks_creatures() {
    let mut g = game();
    let cl = perm(&mut g, P0, CLOUD);
    perm(&mut g, P0, "Stormfront Riders");
    assert!(!g.perm(cl).on_bf, "a bare Cloud may be returned (rebuy of the tutor)");
}

// ---- F8: Unfinished Business is cast
fn labels(g: &mut Game) -> Vec<String> {
    sim_core::ai::brain::main_options(g, P0, false).unwrap().iter().map(|o| o.label.clone()).collect()
}

fn to_gy(g: &mut Game, names: &[&str]) {
    for n in names {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
}

#[test]
fn recommission_is_cast_on_a_cheap_graveyard_creature() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    to_gy(&mut g, &["Kor Duelist"]);
    hand(&mut g, P0, &["Recommission"]);
    g.player_mut(P0).turns = 5;
    let l = labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Recommission")), "{l:?}");
}

// ---- ai-audit cv: Open the Armory
#[test]
fn open_the_armory_fetches_only_aura_or_equipment() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    g.player_mut(P0).library = ["Sol Ring", "Coalition Relic", "Cid, Freeflier Pilot", "Sword of Fire and Ice"]
        .iter()
        .map(|n| card(&g, n))
        .collect();
    let c = hand(&mut g, P0, &["Open the Armory"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", sim_core::state::Ctx::default()).unwrap();
    let names: Vec<String> = g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert_eq!(names, vec!["Sword of Fire and Ice"], "Sol Ring, Relic and Cid are not Auras or Equipment");
}

#[test]
fn open_the_armory_with_no_equipment_in_the_library_fetches_nothing() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    g.player_mut(P0).library = ["Sol Ring", "Cid, Freeflier Pilot"].iter().map(|n| card(&g, n)).collect();
    let c = hand(&mut g, P0, &["Open the Armory"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", sim_core::state::Ctx::default()).unwrap();
    assert!(g.player(P0).hand.is_empty());
}
