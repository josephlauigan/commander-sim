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
// ------------------------------------------------------------------ Titania: Kura, the Boundless Sky
#[test]
fn kura_dies_into_three_lands_while_there_are_land_drops_to_make() {
    let mut g = table(&["omnath", "seph"]);
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
    let mut g = table(&["omnath", "seph"]);
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
fn kura_late_game_makes_the_spirit_even_with_an_empty_hand() {
    let mut g = table(&["omnath", "seph"]);
    lands(&mut g, P0, "Forest", 9, false);
    let before = g.player(P0).hand.len();
    let k = perm(&mut g, P0, "Kura, the Boundless Sky");
    sim_core::engine::zones::die(&mut g, k, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before, "no lands fetched with nine on the battlefield");
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a Spirit");
    assert_eq!(sim_core::engine::values::epow(&g, t), 9);
}

#[test]
fn titania_casts_survey_early_and_vorinclex_as_a_top_end_threat() {
    let mut g = table(&["omnath", "seph"]);
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
