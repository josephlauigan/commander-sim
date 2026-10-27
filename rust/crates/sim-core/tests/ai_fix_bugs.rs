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

// ------------------------------------------------------------------ C: rituals
fn labels(g: &mut Game) -> Vec<(String, f64)> {
    ai::brain::main_options(g, P0, false).unwrap().into_iter().map(|o| (o.label, o.utility)).collect()
}

/// Two lands and Dark Ritual pay for Thassa's Oracle + Demonic Consultation (UU + B): the line counts the ritual's net
/// mana, so it is offered and won this turn. The old check saw two mana and never offered it.
#[test]
fn c_a_ritual_in_hand_makes_the_line_affordable() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Dark Ritual"]);
    let opts = labels(&mut g);
    assert!(opts.iter().any(|o| o.0.starts_with("combo: Thassa's Oracle")), "{opts:?}");
    ai::brain::main(&mut g, P0, false).unwrap();
    assert!(g.over && g.winner == Some(P0), "the line wins with the ritual's mana");
}

/// One land pays Dark Ritual, whose three mana (any colour) pay UU + B exactly; with no land the ritual itself can't be
/// cast, so the line is not offered
#[test]
fn c_the_ritual_must_itself_be_castable() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 1, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Dark Ritual"]);
    let opts = labels(&mut g);
    assert!(opts.iter().any(|o| o.0.starts_with("combo: Thassa's Oracle")), "one land and Dark Ritual: {opts:?}");
    let mut g = scion_goldfish();
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Dark Ritual"]);
    let opts = labels(&mut g);
    assert!(!opts.iter().any(|o| o.0.starts_with("combo:")), "no land: the ritual can't be cast: {opts:?}");
}

/// Cabal Ritual had priority 0 and was never cast outside a line. With four lands it makes the commander castable
/// this turn (WUBRG from two Underground Seas and the ritual's three), so it is worth the commander less a little.
#[test]
fn c_cabal_ritual_is_cast_for_the_commander() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 4, false);
    hand(&mut g, P0, &["Cabal Ritual"]);
    g.player_mut(P0).turns = 3; // the commander waits for turn 2 (`cmd_turn`)
    let s = ai::brain::Situation::new(&g, P0);
    let c = g.player(P0).cmd;
    let u = ai::brain::card_utility(&g, P0, &s, c).expect("the commander has a utility");
    let opts = labels(&mut g);
    let cmd = g.db.get(g.player(P0).cmd).name.to_string();
    assert!(!opts.iter().any(|o| o.0 == cmd), "four Seas can't pay WUBRG: {opts:?}");
    let r = opts.iter().find(|o| o.0 == "Cabal Ritual").unwrap_or_else(|| panic!("{opts:?}"));
    assert!((r.1 - (u - ai::brain::RITUAL_DISCOUNT)).abs() < 1e-9, "{} against {u}", r.1);
}

/// Dark Ritual scored 1.5 from its ability value and was cast for nothing. With nothing it enables it is not offered;
/// with Demonic Tutor one mana short it is worth the tutor less a little.
#[test]
fn c_dark_ritual_is_worth_the_tutor_it_enables_and_nothing_alone() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 1, false);
    hand(&mut g, P0, &["Dark Ritual", "Brainstorm"]);
    let opts = labels(&mut g);
    assert!(!opts.iter().any(|o| o.0 == "Dark Ritual"), "Brainstorm is payable without it: {opts:?}");
    hand(&mut g, P0, &["Demonic Tutor", "Demonic Consultation"]);
    let oracle = card(&g, "Thassa's Oracle");
    g.player_mut(P0).library.insert(0, oracle); // the tutor has the line's missing piece to find
    let opts = labels(&mut g);
    let r = opts.iter().find(|o| o.0 == "Dark Ritual").unwrap_or_else(|| panic!("{opts:?}"));
    assert!(r.1 > 3.0, "worth the tutor: {opts:?}");
}

/// each ritual counts once, and one the line itself uses (`skip`) not at all
#[test]
fn c_each_ritual_counts_once_and_a_skipped_one_not_at_all() {
    use sim_core::engine::mana::can_pay_with_rituals;
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 1, false);
    let r = hand(&mut g, P0, &["Dark Ritual"])[0];
    assert!(can_pay_with_rituals(&g, P0, 0, "BBB", &[]), "Sea -> Dark Ritual: BBB");
    assert!(!can_pay_with_rituals(&g, P0, 1, "BBB", &[]), "one ritual is three mana, not five");
    assert!(!can_pay_with_rituals(&g, P0, 0, "BBB", &[r]), "a ritual the line uses is no mana for it");
    hand(&mut g, P0, &["Dark Ritual"]);
    assert!(can_pay_with_rituals(&g, P0, 2, "BBB", &[]), "two rituals: one B in, five out");
}
