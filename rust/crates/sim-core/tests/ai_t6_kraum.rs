//! Kraum & Tymna (blue midrange) plan slots: keep rule, cast priority, wish list, hold-up, go/wait
//! (audit/t6/kraum-tymna-blue-breach-midrange/SPEC.md scenarios 1, 2, 4-6, 13, 17).

use sim_core::ids::PlayerId;
use sim_core::impls::{t6_plan_kraum as plan, t6_plans};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const KRAUM: &str = "kraum-tymna-blue-breach-midrange";
const ORACLE_LINE: &str = "Thassa's Oracle + Demonic Consultation / Tainted Pact";

fn tbl() -> Game {
    let mut g = table(&[KRAUM, "seph", "sauron", "kefka"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn name(g: &Game, c: sim_core::ids::CardId) -> String {
    g.db.get(c).name.to_string()
}

#[test]
fn keeps_fast_mana_and_engine_hands_with_one_or_two_lands() {
    // scenario 1: two lands + Mox Diamond, Study, Sentinel, Force of Will
    let mut g = tbl();
    hand(
        &mut g,
        P0,
        &["Scrubland", "Island", "Mox Diamond", "Rhystic Study", "Esper Sentinel", "Force of Will", "Swan Song"],
    );
    assert_eq!(plan::keep(&g, P0), Some(true));
    // one land, two fast mana, a tutor
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Tundra", "Sol Ring", "Mana Vault", "Demonic Tutor", "Force of Will", "Dispel", "Swan Song"]);
    assert_eq!(plan::keep(&g, P0), Some(true));
}

#[test]
fn ships_lands_and_spells_and_a_lone_engine() {
    // three lands and only counters / a commander-style top end: no fast mana, no payoff, no tutor
    let mut g = tbl();
    hand(&mut g, P0, &["Scrubland", "Island", "Tundra", "Dispel", "Swan Song", "Chain of Vapor", "Snapback"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
    // all lands
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Scrubland", "Island", "Tundra", "Plateau", "Badlands", "Underground Sea", "Volcanic Island"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
    // one land with a single engine and nothing else
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Tundra", "Mystic Remora", "Dispel", "Swan Song", "Chain of Vapor", "Snapback", "Flusterstorm"]);
    assert_eq!(plan::keep(&g, P0), Some(false));
}

#[test]
fn casts_engines_and_mox_first_and_holds_counters_and_the_win_pieces() {
    let mut g = tbl();
    let cs =
        hand(&mut g, P0, &["Esper Sentinel", "Sol Ring", "Force of Will", "Thassa's Oracle", "Silence", "Dark Ritual"]);
    assert!(plan::prio(&g, P0, cs[0]).unwrap() >= 88.0);
    assert!(plan::prio(&g, P0, cs[1]).unwrap() > 90.0);
    for i in 2..5 {
        assert_eq!(plan::prio(&g, P0, cs[i]), Some(0.0), "{}", name(&g, cs[i]));
    }
}

#[test]
fn wish_list_finishes_the_oracle_pair_then_asks_for_cover() {
    let mut g = tbl();
    let v: Vec<_> = ["Mox Diamond", "Thassa's Oracle", "Tainted Pact", "Force of Will", "Island"]
        .iter()
        .map(|n| card(&g, n))
        .collect();
    g.player_mut(P0).library = v;
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(name(&g, w[0]), "Thassa's Oracle");
    // Oracle and Pact in hand, no cover: Force of Will first
    hand(&mut g, P0, &["Thassa's Oracle", "Tainted Pact"]);
    let w = t6_plans::wish(&g, P0).unwrap();
    assert_eq!(name(&g, w[0]), "Force of Will");
}

#[test]
fn develops_early_then_holds_counters_up() {
    let mut g = tbl();
    hand(&mut g, P0, &["Force of Will", "Swan Song"]);
    g.player_mut(P0).turns = 1;
    assert!(plan::hold_up(&g, P0).unwrap() < 0.0);
    g.player_mut(P0).turns = 5;
    assert!(plan::hold_up(&g, P0).unwrap() > 0.0);
    // nothing to hold: no opinion
    g.player_mut(P0).hand.clear();
    assert_eq!(plan::hold_up(&g, P0), None);
}

#[test]
fn waits_on_oracle_into_open_blue_mana_without_cover_and_goes_with_silence() {
    // scenarios 4, 5: the opponent holds up Islands and a counter
    let mut g = table(&[KRAUM, "sisay-legends-toolbox", "sauron", "kefka"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g.player_mut(P1).hand.clear();
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    lands(&mut g, P1, "Island", 3, false);
    hand(&mut g, P1, &["Force of Will", "Swan Song", "Mental Misstep"]);
    assert_eq!(plan::go_now(&g, P0, ORACLE_LINE), Some(false));
    // Silence in hand: the shared rule decides
    hand(&mut g, P0, &["Silence"]);
    assert_eq!(plan::go_now(&g, P0, ORACLE_LINE), None);
}

#[test]
fn goes_when_the_table_is_tapped_out_and_never_with_kraum_out_against_a_drawer() {
    // scenario 2 reason: nothing open, nothing to answer with
    let mut g = tbl();
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    assert_eq!(plan::go_now(&g, P0, ORACLE_LINE), None);
    // scenario 6: Kraum on my board draws on an opponent's second spell. One card and one land can't cast two
    perm(&mut g, P0, "Kraum, Ludevic's Opus");
    lands(&mut g, P1, "Island", 1, false);
    hand(&mut g, P1, &["Brainstorm"]);
    assert_eq!(plan::go_now(&g, P0, ORACLE_LINE), None, "one spell can't make Kraum draw");
    // two cards and two lands up can
    lands(&mut g, P1, "Island", 1, false);
    hand(&mut g, P1, &["Opt"]);
    assert_eq!(plan::go_now(&g, P0, ORACLE_LINE), Some(false));
}

#[test]
fn ad_nauseam_needs_life() {
    let mut g = tbl();
    g.player_mut(P0).life = 20;
    assert_eq!(plan::go_now(&g, P0, "Ad Nauseam into a win"), Some(false));
    g.player_mut(P0).life = 40;
    assert_eq!(plan::go_now(&g, P0, "Ad Nauseam into a win"), None);
}

#[test]
fn other_decks_do_not_reach_the_kraum_plan() {
    let mut g = table(&["seph", KRAUM, "sauron", "kefka"]);
    g.player_mut(P0).hand.clear();
    assert_eq!(t6_plans::keep(&g, P0), None);
}
