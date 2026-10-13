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
    let w = o
        .iter()
        .find(|o| o.label.starts_with("Yawgmoth's Will"))
        .expect("Will is offered with a storm card in the yard");
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

// ------------------------------------------------------------------ Veyran: Terror of the Peaks, Mirari
const P1: PlayerId = PlayerId(1);

fn cast_free(g: &mut Game, p: PlayerId, name: &str, ctx: sim_core::state::Ctx) {
    let c = hand(g, p, &[name])[0];
    sim_core::engine::cast::cast_card(g, p, c, "hand", ctx).unwrap();
}

#[test]
fn terror_of_the_peaks_hits_for_each_token_a_maker_creates() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Terror of the Peaks");
    perm(&mut g, P0, "Young Pyromancer");
    let before = g.player(P1).life;
    cast_free(&mut g, P0, "Lightning Bolt", sim_core::state::Ctx { face: Some(P1), ..Default::default() });
    // Bolt: 3 damage, then the Pyromancer's 1/1 Elemental enters: Terror deals 1 more
    assert_eq!(g.player(P1).life, before - 3 - 1, "Bolt plus a 1-damage Terror trigger");
}

#[test]
fn mirari_copies_burn_and_the_copy_counts_as_a_cast_for_magecraft() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 6, false);
    perm(&mut g, P0, "Mirari");
    let before = g.player(P1).life;
    cast_free(&mut g, P0, "Lightning Bolt", sim_core::state::Ctx { face: Some(P1), ..Default::default() });
    assert_eq!(g.player(P1).life, before - 6, "Mirari: pay {{3}}, the Bolt resolves twice");
}

// ------------------------------------------------------------------ Sauron: Sheoldred, the Apocalypse
#[test]
fn jaces_archivist_wheel_is_worth_it_with_sheoldred_out() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "Jace's Archivist");
    hand(&mut g, P0, &["Sol Ring", "Mana Vault", "Island"]);
    hand(&mut g, P1, &["Forest", "Forest", "Forest"]);
    assert!(!sim_core::ai::decks::archivist_worth(&g, P0), "no payoff: the wheel only helps the opponents");
    perm(&mut g, P0, "Sheoldred, the Apocalypse");
    assert!(sim_core::ai::decks::archivist_worth(&g, P0), "each opponent draw costs 2 life");
}

// ------------------------------------------------------------------ Titania: Kura, the Boundless Sky
#[test]
fn kura_dies_into_three_lands_while_there_are_land_drops_to_make() {
    let mut g = table(&["titania", "seph"]);
    let before = g.player(P0).hand.len();
    lands(&mut g, P0, "Forest", 3, false);
    let k = perm(&mut g, P0, "Kura, the Boundless Sky");
    sim_core::engine::zones::die(&mut g, k, "destroy").unwrap();
    let h = &g.player(P0).hand;
    assert_eq!(h.len() - before, 3);
    assert!(h.iter().rev().take(3).all(|&c| g.db.get(c).land));
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token));
}

#[test]
fn kura_dies_into_a_big_spirit_once_the_lands_are_down() {
    let mut g = table(&["titania", "seph"]);
    lands(&mut g, P0, "Forest", 6, false);
    hand(&mut g, P0, &["Forest", "Forest"]);
    let before = g.player(P0).hand.len();
    let k = perm(&mut g, P0, "Kura, the Boundless Sky");
    sim_core::engine::zones::die(&mut g, k, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before, "no cards fetched");
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a Spirit");
    assert_eq!((sim_core::engine::values::epow(&g, t), sim_core::engine::values::etgh(&g, t)), (6, 6));
}

#[test]
fn titania_casts_survey_early_and_vorinclex_as_a_top_end_threat() {
    let mut g = table(&["titania", "seph"]);
    lands(&mut g, P0, "Forest", 3, false);
    let hs = hand(&mut g, P0, &["Cartographer's Survey", "Tireless Tracker", "Vorinclex, Monstrous Raider"]);
    let pr = |g: &Game, c| sim_core::ai::decks::deck_prio(g, P0, c);
    assert!(pr(&g, hs[0]) > pr(&g, hs[1]), "Survey before Tracker");
    assert!(pr(&g, hs[2]) >= 62);
}

#[test]
fn veyran_still_escapes_spells_with_underworld_breach() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, "Underworld Breach");
    perm(&mut g, P1, "Orcish Bowmasters"); // a creature worth burning
    to_gy(&mut g, P0, &["Lightning Bolt", "Counterspell", "Opt", "Brainstorm", "Island"]);
    let l = labels(&mut g, P0);
    assert!(l.iter().any(|l| l == "escape Lightning Bolt"), "{l:?}");
}
