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

// ------------------------------------------------------------------ D: the Tier 6 must-do plays
/// Turn 2, three Underground Seas, Thassa's Oracle + Demonic Consultation ready (UU + B) and Arcane Signet in hand.
fn oracle_ready_with_a_signet() -> Game {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation", "Arcane Signet", "Lotus Petal"]);
    g.player_mut(P0).turns = 2;
    g
}

/// The old must-do rule forced a cheap rock on turns 1-4 before the look-ahead, and ignored the `combo:` option: the
/// ready line comes first now
#[test]
fn d_a_ready_line_comes_before_a_forced_rock() {
    let mut g = oracle_ready_with_a_signet();
    let opts = ai::brain::main_options(&mut g, P0, false).unwrap();
    let i = ai::t6_table::must_option(&g, P0, &opts).expect("a must-do play");
    assert!(opts[i].label.starts_with("combo: Thassa's Oracle"), "{}", opts[i].label);
}

/// With the line not on offer (say the table could still stop it), Arcane Signet (two in, one out) would leave it a
/// mana short: it is not forced.
/// --- ai-eval: the old assertion that Lotus Petal (free, one out) *is* forced is gone with the cheap-rock rule that
/// forced it (eval-fix.md, Part 3): no rock is forced now, and the look-ahead decides.
#[test]
fn d_a_forced_cast_that_breaks_a_ready_line_is_skipped() {
    let mut g = oracle_ready_with_a_signet();
    let opts: Vec<_> = ai::brain::main_options(&mut g, P0, false)
        .unwrap()
        .into_iter()
        .filter(|o| !o.label.starts_with("combo:"))
        .collect();
    assert!(opts.iter().any(|o| o.label == "Arcane Signet"), "{:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    let pick = ai::t6_table::must_option(&g, P0, &opts).map(|i| opts[i].label.clone());
    assert_ne!(pick.as_deref(), Some("Arcane Signet"));
}

// ------------------------------------------------------------------ E: win now against win next turn
/// Turn 5, five Command Towers, Thassa's Oracle and Demonic Tutor in hand, Demonic Consultation in the library.
/// Demonic Tutor now wins this turn (two for the tutor, three for the line). Casting the commander (five) instead
/// leaves the same win for next turn. Both scored 100, and the tie went to the higher utility.
fn win_now_or_next_turn() -> Game {
    let mut g = scion_goldfish();
    set_ai(&mut g, AiMode::Lookahead);
    let consult = card(&g, "Demonic Consultation");
    g.player_mut(P0).library.insert(0, consult);
    lands(&mut g, P0, "Command Tower", 5, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Tutor"]);
    g.player_mut(P0).turns = 5;
    g.round = 5;
    g
}

#[test]
fn e_a_win_scores_by_when_it_comes() {
    let mut g = scion_goldfish();
    assert_eq!(sim_core::ai::search::win_score(&g, P0), 100.0, "outside the look-ahead");
    g.in_search = true;
    g.search_from = 4;
    g.player_mut(P0).turns = 4;
    assert_eq!(sim_core::ai::search::win_score(&g, P0), 100.0, "this turn");
    g.player_mut(P0).turns = 5;
    assert_eq!(sim_core::ai::search::win_score(&g, P0), 100.0 - sim_core::ai::search::WIN_TURN_COST, "next turn");
    g.player_mut(P0).turns = 9;
    assert_eq!(sim_core::ai::search::win_score(&g, P0), sim_core::ai::search::WIN_FLOOR, "never below the floor");
}

#[test]
fn e_winning_this_turn_beats_a_play_that_wins_next_turn() {
    let mut g = win_now_or_next_turn();
    let opts = ai::brain::main_options(&mut g, P0, false).unwrap();
    let cmd = g.db.get(g.player(P0).cmd).name.to_string();
    let u = |l: &str| opts.iter().find(|o| o.label == l).map(|o| o.utility);
    assert!(
        u(&cmd).is_some() && u("Demonic Tutor").is_some(),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    let pick = ai::search::choose(&mut g, P0, false, &opts).map(|i| opts[i].label.clone());
    let scores = ai::search::LAST_SCORES.with(|l| l.borrow().clone());
    assert_eq!(pick.as_deref(), Some("Demonic Tutor"), "{scores:?}");
    let s = |l: &str| scores.iter().find(|x| x.0 == l).map(|x| x.1).unwrap();
    assert!(s("Demonic Tutor") > s(&cmd), "{scores:?}");
}

// ------------------------------------------------------------------ B: tutor targets
/// Vampiric Tutor put Thassa's Oracle on top: a second tutor fetched it again (it was still "missing"). The known top
/// card is on its way now; the record ends when the card is drawn, or the library shuffled.
#[test]
fn b_a_card_put_on_top_is_not_fetched_again() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 3, false);
    hand(&mut g, P0, &["Demonic Consultation"]);
    let oracle = card(&g, "Thassa's Oracle");
    g.player_mut(P0).library.insert(0, oracle);
    assert_eq!(ai::tutor_pick(&g, P0, "any"), Some(oracle), "the Oracle is the missing piece");
    sim_core::engine::tutors::tutor_to_top(&mut g, P0, "any", 2).unwrap();
    assert_eq!(g.player(P0).library.last(), Some(&oracle));
    assert_eq!(sim_core::engine::tutors::known_top(&g, P0), vec![oracle]);
    assert_ne!(ai::tutor_pick(&g, P0, "any"), Some(oracle), "it is already on top");
    assert!(!sim_core::cardcode::missing_pieces(&g, P0).contains(&oracle));
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.player(P0).hand.contains(&oracle));
    assert!(sim_core::engine::tutors::known_top(&g, P0).is_empty(), "drawn: the record ends");
    // a shuffle ends it too
    let o2 = g.player_mut(P0).hand.pop().unwrap();
    sim_core::engine::tutors::put_known_top(&mut g, P0, o2);
    sim_core::engine::tutors::shuffle_library(&mut g, P0);
    assert!(sim_core::engine::tutors::known_top(&g, P0).is_empty(), "shuffled: the record ends");
}

/// Oracle + Consultation in hand, two lands for a line that needs three: the line was skipped as complete, so a tutor
/// fetched a piece of another line. Now it fetches the mana: Lotus Petal or a land (with the land drop to come) pays
/// for the line this turn and comes first.
#[test]
fn b_a_line_short_only_of_mana_fetches_mana() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    for n in ["Sol Ring", "Lotus Petal", "Underworld Breach", "Brain Freeze"] {
        let c = card(&g, n);
        g.player_mut(P0).library.insert(0, c);
    }
    g.player_mut(P0).land_turn = -1;
    let miss: Vec<String> = names(&g, &sim_core::cardcode::missing_pieces(&g, P0));
    assert!(!miss.is_empty(), "the line wants mana");
    assert!(miss.iter().all(|n| ["Sol Ring", "Lotus Petal", "Island"].contains(&n.as_str())), "{miss:?}");
    let petal = miss.iter().position(|n| n == "Lotus Petal").expect("Lotus Petal pays for it now");
    assert!(miss.iter().position(|n| n == "Sol Ring").is_none_or(|s| petal < s), "{miss:?}");
}

/// Enlightened Tutor (tagged `tut=any`) was scored 72 for finding Thassa's Oracle; it finds an artifact or enchantment
#[test]
fn b_enlightened_tutor_is_scored_by_what_it_can_find() {
    let mut g = scion_goldfish();
    lands(&mut g, P0, "Underground Sea", 2, false);
    hand(&mut g, P0, &["Demonic Consultation", "Enlightened Tutor", "Demonic Tutor"]);
    for n in ["Thassa's Oracle", "Sol Ring"] {
        let c = card(&g, n);
        g.player_mut(P0).library.insert(0, c);
    }
    let et = card(&g, "Enlightened Tutor");
    let dt = card(&g, "Demonic Tutor");
    assert_eq!(sim_core::ai::pool::search_kind(&g, et), Some("ae"));
    let (pe, pd) = (ai::deck_prio(&g, P0, et), ai::deck_prio(&g, P0, dt));
    assert_eq!(pd, 72.0, "Demonic Tutor finds the Oracle");
    assert!(pe < 72.0 && pe > 0.0, "Enlightened Tutor finds only Sol Ring: {pe}");
}

// ------------------------------------------------------------------ I: determinize keeps the known top card
/// Vampiric Tutor put Thassa's Oracle on top; every playout's shuffle of p's own library lost it, so no playout saw
/// the line it completes next turn. The known top card stays in place; the rest is shuffled as before.
#[test]
fn i_determinize_keeps_the_card_put_on_top() {
    let mut g = scion_goldfish();
    let names_lib: Vec<_> = ["Brainstorm", "Ponder", "Sol Ring", "Swan Song"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library.extend(names_lib);
    let oracle = card(&g, "Thassa's Oracle");
    sim_core::engine::tutors::put_known_top(&mut g, P0, oracle);
    let before = g.player(P0).library.clone();
    for seed in 0..8 {
        let mut g2 = g.clone();
        let mut rng = sim_core::rng::Rng::from_u64(seed);
        sim_core::ai::search::determinize(&mut g2, P0, &mut rng);
        assert_eq!(g2.player(P0).library.last(), Some(&oracle), "seed {seed}");
        let mut a = g2.player(P0).library.clone();
        let mut b = before.clone();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "the same cards");
    }
    // with no record, the top card is shuffled with the rest
    g.player_mut(P0).known_top.clear();
    let moved = (0..8).any(|seed| {
        let mut g2 = g.clone();
        sim_core::ai::search::determinize(&mut g2, P0, &mut sim_core::rng::Rng::from_u64(seed));
        g2.player(P0).library.last() != Some(&oracle)
    });
    assert!(moved);
}
