//! JD's card packages (Kefka, Veyran, Sauron, Titania): do the decks' AIs use the new cards?

use sim_core::ai::brain;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn labels(g: &mut Game, p: PlayerId) -> Vec<String> {
    brain::main_options(g, p, false).unwrap().iter().map(|o| o.label.clone()).collect()
}

fn to_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

// ------------------------------------------------------------------ Kefka: Yawgmoth's Will
#[test]
fn kefka_holds_yawgmoths_will_into_an_empty_or_thin_graveyard() {
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P0, "Swamp", 6, false);
    hand(&mut g, P0, &["Yawgmoth's Will"]);
    assert!(!labels(&mut g, P0).iter().any(|l| l.starts_with("Yawgmoth's Will")), "empty yard");
    to_gy(&mut g, P0, &["Dark Ritual"]);
    assert!(!labels(&mut g, P0).iter().any(|l| l.starts_with("Yawgmoth's Will")), "one ritual is not a turn");
}

#[test]
fn kefka_casts_yawgmoths_will_for_a_storm_card_and_replays_the_line() {
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Yawgmoth's Will"]);
    to_gy(&mut g, P0, &["Brain Freeze", "Dark Ritual", "Lotus Petal"]);
    let o = brain::main_options(&mut g, P0, false).unwrap();
    let w = o.iter().find(|o| o.label.starts_with("Yawgmoth's Will")).expect("Will is offered with a storm card in the yard");
    sim_core::ai::act::perform(&mut g, P0, w.act.as_ref().unwrap()).unwrap();
    assert!(g.player(P0).yawg, "Will resolved");
    let l = labels(&mut g, P0);
    assert!(l.iter().any(|l| l.starts_with("Dark Ritual (Yawgmoth's Will)")), "{l:?}");
}

#[test]
fn kefka_casts_yawgmoths_will_for_three_cheap_spells() {
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P0, "Swamp", 7, false);
    hand(&mut g, P0, &["Yawgmoth's Will"]);
    to_gy(&mut g, P0, &["Duress", "Dark Ritual", "Lotus Petal"]);
    assert!(labels(&mut g, P0).iter().any(|l| l.starts_with("Yawgmoth's Will")));
}

#[test]
fn kefkas_mystical_tutor_finds_the_storm_card_for_a_waiting_breach() {
    use sim_core::ai::decks::tutor_pick_named;
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Underworld Breach");
    let bf = g_card_ids(&mut g, &["Brain Freeze", "Grapeshot"]);
    g.player_mut(P0).library.extend(bf);
    let c = tutor_pick_named(&g, P0, "is").expect("a pick");
    assert!(matches!(&*g.db.get(c).name, "Brain Freeze" | "Grapeshot"));
}

fn g_card_ids(g: &mut Game, names: &[&str]) -> Vec<sim_core::ids::CardId> {
    names.iter().map(|n| card(g, n)).collect()
}
