//! The Tier 6 speed measure's tutors (`combos::tutor_for_piece`, `one_tutor_away`, `useful_tutors`): a card that
//! searches nothing is no tutor. Walking Ballista carries the `xtutor` tag (an X spell), and before the fix it made
//! every Tymna/Thrasios hand with one piece of a line "one tutor away" and topped the "tutors held" list.

use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// Tymna/Thrasios in seat 0, measured as a Tier 6 deck: an empty hand (the dealt cards back in the library), three
/// Underground Seas, Thassa's Oracle in hand; Demonic Consultation and Tainted Pact stay in the library
fn oracle_held() -> Game {
    let mut g = table(&["thrasios", "seph"]);
    g.active = Some(P0);
    let dealt: Vec<_> = std::mem::take(&mut g.player_mut(P0).hand);
    g.player_mut(P0).library.extend(dealt);
    g.player_mut(P0).t6 = true;
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(&mut g, P0, &["Thassa's Oracle"]);
    let lib = &g.player(P0).library;
    assert!(lib.contains(&card(&g, "Demonic Consultation")) && lib.contains(&card(&g, "Tainted Pact")));
    g
}

fn reach(g: &mut Game) -> bool {
    combos::combo_options(g, P0, 0.0, Some(true)).unwrap();
    let st = &g.player(P0).stats;
    assert!(!st.contains_key(&intern("sp|ready")), "no line is ready: the exile half is in the library");
    st.contains_key(&intern("sp|reach"))
}

fn names(g: &Game, cs: &[sim_core::ids::CardId]) -> Vec<String> {
    cs.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

#[test]
fn walking_ballista_is_no_tutor_for_the_measure() {
    let mut g = oracle_held();
    let b = card(&g, "Walking Ballista");
    assert!(g.db.get(b).tags.has(sim_core::tag::Tag::Xtutor), "the case: Ballista carries the xtutor tag");
    hand(&mut g, P0, &["Walking Ballista"]);
    let piece = vec![card(&g, "Demonic Consultation")];
    assert!(!combos::tutor_for_piece(&g, P0, b, &piece));
    assert!(combos::useful_tutors(&g, P0).is_empty(), "{:?}", names(&g, &combos::useful_tutors(&g, P0)));
    assert!(!reach(&mut g), "Oracle and Ballista in hand is not one tutor away");
}

#[test]
fn demonic_tutor_with_the_oracle_is_one_tutor_away() {
    let mut g = oracle_held();
    hand(&mut g, P0, &["Walking Ballista", "Demonic Tutor"]);
    assert_eq!(names(&g, &combos::useful_tutors(&g, P0)), vec!["Demonic Tutor".to_string()]);
    assert!(reach(&mut g), "Demonic Tutor ({{1}}{{B}}) and Consultation ({{B}}) on three Underground Seas");
}

#[test]
fn a_tutor_that_cannot_take_the_piece_is_no_tutor_for_the_line() {
    let mut g = oracle_held();
    let wt = hand(&mut g, P0, &["Worldly Tutor"])[0];
    let piece = vec![card(&g, "Demonic Consultation"), card(&g, "Tainted Pact")];
    assert!(!combos::tutor_for_piece(&g, P0, wt, &piece), "Worldly Tutor finds creatures only");
    assert!(!reach(&mut g));
}
