//! The mulligan hint names the rule that decided (the plan file's keep-table row), the cards it counted, the colours,
//! the speed and, for Tymna/Thrasios, the oracle's numbers; and it never changes the verdict (`keeps_hand`).

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::advisor;
use serde_json::{Value, json};
use sim_core::control::Request;
use sim_core::engine::turn::keeps_hand;
use sim_core::ids::PlayerId;
use sim_core::run::settings;
use sim_core::state::Game;

/// a game whose seat 0 is `deck`, with exactly this hand (taken from the library; the rest stays shuffled)
fn game_with(deck: &str, hand: &[&str]) -> (Game, PlayerId) {
    let e = engine();
    let others = ["aragorn", "sheoldred", "cloud", "thanos", "yshtola"];
    let mut seats: Vec<String> = vec![deck.to_string()];
    seats.extend(others.iter().filter(|k| **k != deck).take(3).map(|s| s.to_string()));
    let (mut g, _) =
        practice::session::setup::build_game(&e, settings("loose", "adaptive", 1.0).unwrap(), 77, &seats, &[]).unwrap();
    let p = PlayerId(0);
    let back = std::mem::take(&mut g.player_mut(p).hand);
    g.player_mut(p).library.extend(back);
    for n in hand {
        let i = g
            .player(p)
            .library
            .iter()
            .position(|&c| &*g.db.get(c).name == *n)
            .unwrap_or_else(|| panic!("{deck}: no {n} in the library"));
        let c = g.player_mut(p).library.remove(i);
        g.player_mut(p).hand.push(c);
    }
    (g, p)
}

fn req(prompt: &str) -> Request {
    Request {
        kind: sim_core::sym::intern("mulligan"),
        prompt: prompt.into(),
        choices: vec!["keep".into(), "mulligan".into()],
        data: json!({}),
    }
}

fn hint(g: &mut Game, p: PlayerId) -> Value {
    advisor::hint(g, p, &req("Your opening hand (7 to keep)."))
}

fn lines(h: &Value) -> Vec<String> {
    h["detail"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
}

fn has(h: &Value, needle: &str) -> bool {
    lines(h).iter().any(|l| l.contains(needle)) || h["text"].as_str().unwrap().contains(needle)
}

/// the verdict in the hint is the one `keeps_hand` gives
fn verdict_unchanged(g: &mut Game, p: PlayerId, h: &Value) {
    let k = keeps_hand(g, p);
    assert_eq!(h["choice"], if k { "keep" } else { "mulligan" }, "{h}");
    assert!(h["text"].as_str().unwrap().starts_with(if k { "Keep" } else { "Mulligan" }), "{h}");
}

#[test]
fn thrasios_no_lands_with_fast_mana_and_a_tutor_is_kept_by_row_2() {
    let (mut g, p) = game_with(
        "thrasios",
        &[
            "Lotus Petal",
            "Sol Ring",
            "Chrome Mox",
            "Vampiric Tutor",
            "Demonic Consultation",
            "Thassa's Oracle",
            "Brainstorm",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "keep", "{h}");
    let rule = &lines(&h)[0];
    assert!(rule.contains("row 2 of"), "{rule}");
    assert!(rule.contains("Lands: none"), "{rule}");
    assert!(
        rule.contains("Fast mana") && rule.contains("3 or more") && rule.contains("Chrome Mox, Lotus Petal, Sol Ring"),
        "{rule}"
    );
    assert!(rule.contains("Tutors: at least 1") && rule.contains("Vampiric Tutor"), "{rule}");
    assert!(!has(&h, "keep a hand with 2 to 5 lands"), "the generic rule must not be described: {h}");
    // mana, key cards, colours and the line
    assert!(has(&h, "Mana on turn 2:"), "{h}");
    assert!(has(&h, "Key cards:") && has(&h, "win-line pieces"), "{h}");
    assert!(has(&h, "Black:"), "{h}");
    assert!(has(&h, "the hand holds all of \"Thassa's Oracle + Demonic Consultation / Tainted Pact\""), "{h}");
}

/// turn-2 mana is what you can have then (two land drops at most), not the keep rule's count of every land in hand
#[test]
fn three_lands_and_sol_ring_make_four_mana_on_turn_2_not_five() {
    let (mut g, p) = game_with(
        "thrasios",
        &["Tundra", "Watery Grave", "Underground Sea", "Sol Ring", "Vampiric Tutor", "Thassa's Oracle", "Brainstorm"],
    );
    let h = hint(&mut g, p);
    let d = lines(&h);
    assert!(d.iter().any(|l| l.starts_with("Mana on turn 2: 4 = 2 lands + Sol Ring 2 (cast turn 1)")), "{h}");
    assert!(!d.iter().any(|l| l.contains("Mana on turn 2: 5")), "{h}");
    assert!(d.iter().any(|l| l.starts_with("The keep rule counts this hand as 5 mana")), "{h}");
    // the speed line never has more lands than the turn number: Tymna (cost 3) needs turn 2 here (2 lands + Sol Ring)
    assert!(d.iter().any(|l| l.contains("Tymna the Weaver (cost 3) turn 2")), "{h}");
    // no rocks: a four-drop commander needs four turns, one land a turn
    let (mut g, p) =
        game_with("aragorn", &["Plains", "Island", "Forest", "Tundra", "Taiga", "Savannah", "Counterspell"]);
    let h = hint(&mut g, p);
    assert!(lines(&h).iter().any(|l| l.contains("Aragorn, the Uniter (cost 4) turn 4")), "{h}");
    assert!(lines(&h).iter().any(|l| l.starts_with("Mana on turn 2: 2 = 2 lands.")), "{h}");
}

#[test]
fn thrasios_six_lands_are_sent_back_by_row_1() {
    let (mut g, p) = game_with(
        "thrasios",
        &["Watery Grave", "Underground Sea", "Bayou", "Scrubland", "Overgrown Tomb", "Tundra", "Vampiric Tutor"],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "mulligan", "{h}");
    let rule = &lines(&h)[0];
    assert!(rule.contains("row 1 of") && rule.contains("Lands: 6 or more"), "{rule}");
}

#[test]
fn thrasios_colours_say_what_doomsday_and_ad_nauseam_need() {
    let (mut g, p) = game_with(
        "thrasios",
        &["Tundra", "Hallowed Fountain", "Watery Grave", "Doomsday", "Ad Nauseam", "Thassa's Oracle", "Brainstorm"],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    let black = lines(&h).into_iter().find(|l| l.starts_with("Black:")).unwrap_or_else(|| panic!("{h}"));
    assert!(black.contains("Doomsday {B}{B}{B}") && black.contains("Ad Nauseam {B}{B}"), "{black}");
    assert!(black.contains("1 black source") && black.contains("short"), "{black}");
}

#[test]
fn aragorn_one_lander_without_fast_mana_goes_back_by_its_row() {
    let (mut g, p) = game_with(
        "aragorn",
        &["Plains", "Counterspell", "Mana Drain", "Cyclonic Rift", "Rhystic Study", "Brain Freeze", "Grapeshot"],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "mulligan", "{h}");
    let rule = &lines(&h)[0];
    assert!(
        rule.contains("row 3 of 4")
            && rule.contains("Lands: exactly 1")
            && rule.contains("Fast mana")
            && rule.contains("fewer than 2"),
        "{rule}"
    );
    assert!(rule.contains("Plains"), "{rule}");
    assert!(!has(&h, "2 to 5 lands"), "{h}");
    assert!(has(&h, "Line:"), "{h}");
}

#[test]
fn thanos_one_lander_with_too_little_fast_mana_goes_back() {
    let (mut g, p) = game_with(
        "thanos",
        &[
            "Forest",
            "Sol Ring",
            "Mind Stone",
            "Vampiric Tutor",
            "Basalt Monolith",
            "Rings of Brighthearth",
            "Walking Ballista",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "mulligan", "{h}");
    let rule = &lines(&h)[0];
    assert!(
        rule.contains("row 3 of 5") && rule.contains("Lands: exactly 1") && rule.contains("fewer than 3"),
        "{rule}"
    );
    assert!(rule.contains("Sol Ring"), "the counted fast-mana piece is named: {rule}");
    assert!(has(&h, "Mind Stone (needs two lands to cast it)"), "{h}");
    assert!(has(&h, "Line:") && has(&h, "Basalt Monolith"), "{h}");
}

#[test]
fn cloud_three_lands_are_kept_by_the_land_row() {
    let (mut g, p) = game_with(
        "cloud",
        &[
            "Plains",
            "Plains",
            "Plains",
            "Sword of Fire and Ice",
            "Swords to Plowshares",
            "Stoneforge Mystic",
            "Sol Ring",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "keep", "{h}");
    let rule = &lines(&h)[0];
    assert!(rule.contains("row 1 of 2") && rule.contains("Lands: 3 to 5") && rule.contains("you have 3"), "{rule}");
    assert!(has(&h, "Colours the plan needs") && has(&h, "White:"), "{h}");
    assert!(has(&h, "commander Cloud, Midgar Mercenary"), "{h}");
}

#[test]
fn cloud_two_lands_need_a_cheap_play_and_say_what_would_have_kept_it() {
    let (mut g, p) = game_with(
        "cloud",
        &[
            "Plains",
            "Plains",
            "Wrath of God",
            "Fumigate",
            "Austere Command",
            "Elspeth, Sun's Champion",
            "Open the Armory",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "mulligan", "{h}");
    let rule = &lines(&h)[0];
    assert!(
        rule.contains("kinds of hand Cloud")
            && rule.contains("so mulligan it")
            && rule.contains("The table keeps:")
            && rule.contains("Lands: 3 to 5"),
        "{rule}"
    );
}

#[test]
fn yshtola_and_sheoldred_have_no_table_and_say_so() {
    let (mut g, p) = game_with(
        "yshtola",
        &["Swamp", "Rhystic Study", "Counterspell", "Farewell", "Exsanguinate", "Sanguine Bond", "Torment of Hailfire"],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "mulligan", "{h}");
    assert!(lines(&h)[0].contains("no keep table of its own") && lines(&h)[0].contains("2 to 5 lands"), "{h}");
    let (mut g, p) = game_with(
        "sheoldred",
        &["Swamp", "Swamp", "Demonic Tutor", "Dark Ritual", "Grave Titan", "Reanimate", "Entomb"],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "keep", "{h}");
    assert!(lines(&h)[0].contains("no keep table of its own"), "{h}");
    assert!(has(&h, "tutors - Demonic Tutor"), "{h}");
}

/// the explanation never changes the verdict: over many dealt hands of every deck the hint's choice is `keeps_hand`
#[test]
fn the_verdict_is_keeps_hand_over_many_hands() {
    let e = engine();
    for deck in ["thrasios", "aragorn", "thanos", "cloud", "yshtola", "sheoldred"] {
        let others = ["aragorn", "sheoldred", "cloud", "thanos", "yshtola"];
        let mut seats: Vec<String> = vec![deck.to_string()];
        seats.extend(others.iter().filter(|k| **k != deck).take(3).map(|s| s.to_string()));
        for seed in 0..12u64 {
            let (mut g, _) = practice::session::setup::build_game(
                &e,
                settings("loose", "adaptive", 1.0).unwrap(),
                1000 + seed,
                &seats,
                &[],
            )
            .unwrap();
            let p = PlayerId(0);
            let pl = g.player_mut(p);
            let hand = std::mem::take(&mut pl.hand);
            pl.library.extend(hand);
            for _ in 0..7 {
                let c = pl.library.pop().unwrap();
                pl.hand.push(c);
            }
            // the plan file's reason and the table's verdict agree (all but the oracle, which is Thrasios's and slow)
            let why = sim_core::ai::plan_data::keeps_hand_why(&g, p);
            if let Some(w) = &why {
                assert_eq!(Some(w.keep), sim_core::ai::plan_data::keeps_hand(&g, p), "{deck} seed {seed}");
            }
            if deck == "thrasios" {
                continue; // the hint runs the oracle: covered by its own test
            }
            let h = hint(&mut g, p);
            verdict_unchanged(&mut g, p, &h);
            assert!(lines(&h).len() >= 4, "{h}");
        }
    }
}

#[test]
fn the_thrasios_hint_carries_the_oracles_numbers_and_they_do_not_move() {
    let hand = [
        "Tundra",
        "Watery Grave",
        "Underground Sea",
        "Sol Ring",
        "Vampiric Tutor",
        "Thassa's Oracle",
        "Demonic Consultation",
    ];
    let (mut g, p) = game_with("thrasios", &hand);
    let a = hint(&mut g, p);
    let o = lines(&a)
        .into_iter()
        .find(|l| l.starts_with("Oracle, 40 goldfish games each way"))
        .unwrap_or_else(|| panic!("{a}"));
    assert!(o.contains("kept, this hand kills on turn") && o.contains("mulliganed, on turn"), "{o}");
    assert!(o.contains("The oracle says"), "{o}");
    assert!(has(&a, "goldfish games have no opponents"), "{a}");
    // asked again (cached) and asked fresh (uncached, a new game with the same hand): the same numbers
    let b = hint(&mut g, p);
    assert_eq!(a, b);
    let u1 = advisor::mulligan::oracle_uncached(&g, p, 0).unwrap();
    let (g2, p2) = game_with("thrasios", &hand);
    let u2 = advisor::mulligan::oracle_uncached(&g2, p2, 0).unwrap();
    assert_eq!(u1, u2, "the same hand gives the same numbers");
    assert_eq!(u1[0], o);
    verdict_unchanged(&mut g, p, &a);
}

#[test]
fn other_decks_get_no_oracle_line() {
    let (mut g, p) = game_with(
        "aragorn",
        &["Plains", "Island", "Forest", "Sol Ring", "Mystical Tutor", "Rhystic Study", "Counterspell"],
    );
    let h = hint(&mut g, p);
    assert!(!has(&h, "goldfish"), "{h}");
}

#[test]
fn the_bottom_choice_says_why() {
    let (mut g, p) = game_with(
        "cloud",
        &[
            "Plains",
            "Plains",
            "Plains",
            "Sword of Fire and Ice",
            "Swords to Plowshares",
            "Wrath of God",
            "Elspeth, Sun's Champion",
        ],
    );
    let r = req("Your opening hand (6 to keep). If you keep, you put 1 card(s) on the bottom.");
    let h = advisor::hint(&mut g, p, &r);
    let d = lines(&h);
    let line = d.iter().find(|l| l.contains("on the bottom")).unwrap_or_else(|| panic!("{h}"));
    assert!(line.contains("Why: it is the spell worth least") || line.contains("most expensive"), "{line}");
    let (mut g, p) = game_with(
        "thrasios",
        &["Tundra", "Watery Grave", "Underground Sea", "Bayou", "Sol Ring", "Vampiric Tutor", "Thassa's Oracle"],
    );
    let r = req("Your opening hand (6 to keep). If you keep, you put 1 card(s) on the bottom.");
    let h = advisor::hint(&mut g, p, &r);
    let line = lines(&h).into_iter().find(|l| l.contains("on the bottom")).unwrap_or_else(|| panic!("{h}"));
    assert!(line.contains("Why: it is the card this plan needs least"), "{line}");
    assert!(!has(&h, "Oracle, 40 goldfish"), "no oracle after a mulligan: {h}");
}

/// prints three example hints (run with `--ignored --nocapture`)
#[test]
#[ignore]
fn print_example_hints() {
    let cases: [(&str, &[&str]); 3] = [
        (
            "thrasios",
            &[
                "Tundra",
                "Watery Grave",
                "Underground Sea",
                "Sol Ring",
                "Vampiric Tutor",
                "Thassa's Oracle",
                "Demonic Consultation",
            ],
        ),
        (
            "aragorn",
            &["Plains", "Counterspell", "Mana Drain", "Cyclonic Rift", "Rhystic Study", "Brain Freeze", "Grapeshot"],
        ),
        (
            "cloud",
            &[
                "Plains",
                "Plains",
                "Plains",
                "Sword of Fire and Ice",
                "Swords to Plowshares",
                "Stoneforge Mystic",
                "Sol Ring",
            ],
        ),
    ];
    for (deck, hand) in cases {
        let (mut g, p) = game_with(deck, hand);
        let t = std::time::Instant::now();
        let h = hint(&mut g, p);
        println!("=== {deck} ({:.2}s)\n{}", t.elapsed().as_secs_f64(), h["text"].as_str().unwrap());
        for l in lines(&h) {
            println!("  - {l}");
        }
    }
}

/// a hand no row fits: the hint says what the rows are (reasons to mulligan, hands to keep, or both) and what that
/// means for the hand, not "the default decides"
#[test]
fn a_hand_no_row_fits_says_why_it_is_kept() {
    // Aragorn's rows are all reasons to mulligan: passing them all keeps the hand
    let (mut g, p) = game_with(
        "aragorn",
        &[
            "Steam Vents",
            "Glacial Fortress",
            "Yavimaya Coast",
            "Counterspell",
            "Mana Drain",
            "Rhystic Study",
            "Grapeshot",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "keep", "{h}");
    let rule = &lines(&h)[0];
    assert!(rule.contains("reasons to mulligan fits this hand, so keep it"), "{rule}");
    assert!(!rule.contains("default"), "{rule}");
    // Tymna/Thrasios's rows both keep and mulligan: the hint counts them
    let (mut g, p) = game_with(
        "thrasios",
        &[
            "Tundra",
            "Underground Sea",
            "Watery Grave",
            "Sol Ring",
            "Demonic Consultation",
            "Vampiric Tutor",
            "Thassa's Oracle",
        ],
    );
    let h = hint(&mut g, p);
    verdict_unchanged(&mut g, p, &h);
    assert_eq!(h["choice"], "keep", "{h}");
    let rule = &lines(&h)[0];
    assert!(rule.contains("keep a hand,") && rule.contains("send one back") && rule.contains("so keep it"), "{rule}");
}

/// Mox Jasper makes mana only while you control a Dragon: with a Dragon the hand cannot cast (Scion of Draco, 9) it
/// makes nothing, so turn 2 has 2 lands and Mind Stone (cost 2) is cast on turn 2, not turn 1
#[test]
fn mox_jasper_makes_no_mana_without_a_dragon_the_hand_can_cast() {
    let (mut g, p) = game_with(
        "thanos",
        &["Plateau", "Taiga", "Mox Jasper", "Mind Stone", "Scion of Draco", "Vampiric Tutor", "Swords to Plowshares"],
    );
    let h = hint(&mut g, p);
    let d = lines(&h);
    assert!(d.iter().any(|l| l.starts_with("Mana on turn 2: 2 = 2 lands")), "{h}");
    assert!(has(&h, "Mox Jasper (needs a Dragon you can cast first)"), "{h}");
    assert!(has(&h, "Mind Stone (cast turn 2, makes mana from turn 3)"), "{h}");
    assert!(!d.iter().any(|l| l.contains("Mind Stone 1 (cast turn 1)")), "{h}");
}

/// ... and counts as a mana when the hand can cast a Dragon (2 lands + Sol Ring + Mana Vault pay for Thundermaw Hellkite, cost 5)
#[test]
fn mox_jasper_counts_when_the_hand_can_cast_a_dragon() {
    let (mut g, p) = game_with(
        "thanos",
        &["Plateau", "Taiga", "Mox Jasper", "Sol Ring", "Mana Vault", "Thundermaw Hellkite", "Vampiric Tutor"],
    );
    let h = hint(&mut g, p);
    assert!(!has(&h, "Mox Jasper (needs a Dragon"), "{h}");
    assert!(lines(&h).iter().any(|l| l.contains("Mox Jasper 1")), "{h}");
}
