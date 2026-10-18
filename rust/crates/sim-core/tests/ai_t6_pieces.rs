//! Tier 6 piece care (task `pieces`): tutors and free casts do not bin or misplay the pieces of the deck's own lines,
//! and self-damage engines are priced against life. Tiers 0-5 are untouched.

use sim_core::ids::PlayerId;
use sim_core::impls::{fixes, partials, rules2, t6_green2, topdeck};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const RT: &str = "rograkh-thrasios-creature-combo";
const SISAY: &str = "sisay-legends-toolbox";

fn tbl(keys: &[&str]) -> Game {
    let mut g = table_with(keys, 1, 40);
    g.active = Some(P0);
    g
}

fn in_exile(g: &Game, name: &str) -> bool {
    g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn gamble_is_held_with_a_line_piece_in_hand_unless_the_find_is_cast_at_once() {
    let mut g = tbl(&[KRAUM, SISAY]);
    lands(&mut g, P0, "Mountain", 1, false);
    let gam = hand(&mut g, P0, &["Gamble", "Thassa's Oracle"])[0];
    g.player_mut(P0).library = vec![card(&g, "Demonic Consultation"), card(&g, "Brainstorm")];
    // one land: Gamble uses it, Demonic Consultation could not be cast after it
    assert!(!fixes::gamble_allowed(&g, P0, gam));
    // three lands: the find (Consultation completes the Oracle line) is cast at once
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(fixes::gamble_allowed(&g, P0, gam));
    // the library holds nothing that completes a line
    g.player_mut(P0).library = vec![card(&g, "Brainstorm")];
    assert!(!fixes::gamble_allowed(&g, P0, gam));
}

#[test]
fn gamble_is_unchanged_without_a_piece_in_hand_and_outside_tier_6() {
    let mut g = tbl(&[KRAUM, SISAY]);
    lands(&mut g, P0, "Mountain", 1, false);
    let gam = hand(&mut g, P0, &["Gamble", "Brainstorm"])[0];
    assert!(fixes::gamble_allowed(&g, P0, gam));
    let mut g = tbl(&["seph", "sauron"]);
    lands(&mut g, P0, "Mountain", 1, false);
    let gam = hand(&mut g, P0, &["Gamble", "Thassa's Oracle"])[0];
    assert!(fixes::gamble_allowed(&g, P0, gam));
}

#[test]
fn beseech_casts_oracle_free_only_with_the_other_half_in_hand() {
    let mut g = tbl(&[KRAUM, SISAY]);
    let oracle = card(&g, "Thassa's Oracle");
    assert!(!rules2::beseech_free_ok(&g, P0, oracle), "a lone Oracle stays in hand");
    hand(&mut g, P0, &["Demonic Consultation"]);
    assert!(rules2::beseech_free_ok(&g, P0, oracle));
    // a card that is no piece is cast free as before
    assert!(rules2::beseech_free_ok(&g, P0, card(&g, "Brainstorm")));
    // other tiers: no restriction
    let g = tbl(&["seph", "sauron"]);
    assert!(rules2::beseech_free_ok(&g, P0, card(&g, "Thassa's Oracle")));
}

#[test]
fn chrome_mox_never_imprints_a_line_piece() {
    let mut g = tbl(&[KRAUM, SISAY]);
    let m = perm(&mut g, P0, "Chrome Mox");
    hand(&mut g, P0, &["Demonic Consultation"]);
    sim_core::engine::tutors::chrome_imprint(&mut g, P0, m).unwrap();
    assert!(!in_exile(&g, "Demonic Consultation"));
    hand(&mut g, P0, &["Brainstorm"]);
    sim_core::engine::tutors::chrome_imprint(&mut g, P0, m).unwrap();
    assert!(in_exile(&g, "Brainstorm"));
    assert!(!in_exile(&g, "Demonic Consultation"));
}

#[test]
fn wishclaw_is_activated_only_when_the_tutored_card_finishes_a_line_and_is_cast_at_once() {
    let mut g = tbl(&[KRAUM, SISAY]);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle"]);
    g.player_mut(P0).library = vec![card(&g, "Demonic Consultation"), card(&g, "Brainstorm")];
    assert!(rules2::wishclaw_ok(&g, P0));
    g.player_mut(P0).library = vec![card(&g, "Brainstorm")];
    assert!(!rules2::wishclaw_ok(&g, P0), "no win to tutor for: do not hand the opponents a tutor");
    let g = tbl(&["seph", "sauron"]);
    assert!(rules2::wishclaw_ok(&g, P0));
}

#[test]
fn invasion_of_ikoria_fetches_the_missing_piece_of_the_line() {
    let mut g = tbl(&[RT, SISAY]);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lands(&mut g, P0, "Talon Gates of Madara", 1, false);
    let oboro = card(&g, "Oboro Breezecaller");
    let mast = card(&g, "Faerie Mastermind");
    g.player_mut(P0).library = vec![mast, oboro];
    let (c, lib) = t6_green2::ikoria_pick(&g, P0, 14).unwrap();
    assert_eq!(c, oboro);
    assert!(lib);
}

#[test]
fn the_one_ring_is_not_tapped_down_to_the_grave() {
    let mut g = tbl(&[KRAUM, SISAY]);
    g.player_mut(P0).life = 3;
    assert!(!partials::ring_life_ok(&g, P0, 1));
    g.player_mut(P0).life = 40;
    assert!(partials::ring_life_ok(&g, P0, 2));
    g.player_mut(P0).life = 20;
    assert!(!partials::ring_life_ok(&g, P0, 4), "four counters would cost 5 a turn");
    let mut g = tbl(&["seph", "sauron"]);
    g.player_mut(P0).life = 3;
    assert!(partials::ring_life_ok(&g, P0, 1), "other tiers unchanged");
}

#[test]
fn sylvan_library_pays_four_life_for_one_strong_card_when_life_is_high() {
    let mut g = tbl(&[KRAUM, SISAY]);
    g.player_mut(P0).life = 40;
    assert!(topdeck::sylvan_pays(&g, P0, 70.0, 0));
    assert!(!topdeck::sylvan_pays(&g, P0, 70.0, 1), "one card a turn");
    assert!(!topdeck::sylvan_pays(&g, P0, 58.0, 0), "only a strong card");
    g.player_mut(P0).life = 26;
    assert!(!topdeck::sylvan_pays(&g, P0, 90.0, 0));
    let mut g = tbl(&["seph", "sauron"]);
    g.player_mut(P0).life = 26;
    assert!(topdeck::sylvan_pays(&g, P0, 58.0, 1), "other tiers unchanged");
}
