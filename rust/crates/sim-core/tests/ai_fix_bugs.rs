//! ai-fix (AI foundations step 1): the decision bugs of `audit/ai-foundations/REPORT.md`, one test (or more) per bug,
//! each a game state where the old code chose wrong.

use sim_core::ai;
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// the scion seat against three inert goldfish seats (they never cast), its hand emptied, a library of Islands
fn scion_goldfish() -> Game {
    let mut g =
        table(&["scion", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P0).library = lib;
    g
}

fn names(g: &Game, cs: &[sim_core::ids::CardId]) -> Vec<String> {
    cs.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

// ------------------------------------------------------------------ G: the cleanup discard
/// Eight cards at the end step, Thassa's Oracle the only two-drop: the old rule discarded the dearest card (the
/// Oracle). It is a piece of the deck's own line, so it stays and a spare one-drop goes.
#[test]
fn g_the_cleanup_keeps_thassas_oracle() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(
        &mut g,
        P0,
        &[
            "Thassa's Oracle",
            "Island",
            "Brainstorm",
            "Ponder",
            "Swan Song",
            "Mystical Tutor",
            "Vampiric Tutor",
            "Dark Ritual",
        ],
    );
    let h = g.player(P0).hand.clone();
    let c = ai::cleanup_discard(&g, P0, &h);
    assert_ne!(&*g.db.get(c).name, "Thassa's Oracle");
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    let left = names(&g, &g.player(P0).hand);
    assert_eq!(left.len(), 7, "{left:?}");
    assert!(left.iter().any(|n| n == "Thassa's Oracle"), "{left:?}");
    assert!(left.iter().any(|n| n == "Island"), "the one land in hand is kept for the next land drop: {left:?}");
}

/// Protection is kept up to two cards (free ones first); a third counter goes before a card it can cast soon
#[test]
fn g_the_cleanup_keeps_two_protection_cards_and_spares_the_rest() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 4, false);
    hand(
        &mut g,
        P0,
        &[
            "Thassa's Oracle",
            "Demonic Consultation",
            "Force of Will",
            "Pact of Negation",
            "Counterspell",
            "Demonic Tutor",
            "Island",
            "Brainstorm",
        ],
    );
    let h = g.player(P0).hand.clone();
    let c = ai::cleanup_discard(&g, P0, &h);
    let n = g.db.get(c).name.to_string();
    let kept = ["Thassa's Oracle", "Demonic Consultation", "Force of Will", "Pact of Negation", "Island"];
    assert!(!kept.contains(&n.as_str()), "{n}: the pieces, the two free counters and the one land stay");
}

/// Spare lands go first: five lands on the battlefield, three in hand and nothing dearer than three mana
#[test]
fn g_the_cleanup_discards_a_spare_land_first() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 5, false);
    hand(
        &mut g,
        P0,
        &[
            "Thassa's Oracle",
            "Demonic Consultation",
            "Island",
            "Island",
            "Island",
            "Force of Will",
            "Ponder",
            "Swan Song",
        ],
    );
    let h = g.player(P0).hand.clone();
    let c = ai::cleanup_discard(&g, P0, &h);
    assert_eq!(&*g.db.get(c).name, "Island");
    let _ = AiMode::Lookahead;
}
