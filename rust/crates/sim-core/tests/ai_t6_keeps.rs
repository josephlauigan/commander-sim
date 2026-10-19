//! Tier 6 opening-hand keeps: how often each deck mulligans, and how many lands the kept hand has seen by turn 4
//! (the 7 cards plus the 3 draws of turns 2-4 on the play, 4 on a later seat; counted as 10 cards).
//! Run `cargo test --release --test ai_t6_keeps -- --nocapture rates` for the table.

use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const DECKS: [&str; 7] = [
    "dack-fayden-white-stax",
    "kinnan-bonder-mana-combo",
    "kraum-tymna-blue-breach-midrange",
    "rograkh-silas-turbo-breach",
    "rograkh-thrasios-creature-combo",
    "sisay-legends-toolbox",
    "godo",
];

struct Rates {
    hands: u32,
    mulled: u32,
    triple: u32,
    lands10: f64,
    low10: u32,
    one_land_keeps: u32,
}

fn measure(key: &str, n: u64) -> Rates {
    let mut r = Rates { hands: 0, mulled: 0, triple: 0, lands10: 0.0, low10: 0, one_land_keeps: 0 };
    for seed in 0..n {
        let mut g = table(&[key, "seph"]);
        let mut rng = sim_core::rng::Rng::named(&format!("lib:{seed}:{key}"));
        sim_core::engine::turn::mulligan(&mut g, P0, &mut rng);
        let pl = g.player(P0);
        let mn = pl.stats.get("mull_n").copied().unwrap_or(0);
        let is_land = |c: sim_core::ids::CardId| g.db.get(c).land;
        let hand_lands = pl.hand.iter().filter(|&&c| is_land(c)).count();
        let top3 = pl.library.iter().rev().take(3).filter(|&&c| is_land(c)).count();
        r.hands += 1;
        r.mulled += (mn >= 1) as u32;
        r.triple += (mn >= 3) as u32;
        r.lands10 += (hand_lands + top3) as f64;
        r.low10 += (hand_lands + top3 <= 2) as u32;
        r.one_land_keeps += (hand_lands <= 1 && mn < 3) as u32;
    }
    r
}

#[test]
fn rates() {
    for key in DECKS {
        let r = measure(key, 1500);
        let h = r.hands as f64;
        println!(
            "KEEPS {key:36} mulligan {:5.1}%  triple {:4.1}%  lands by T4 {:.2}  (<=2 lands: {:4.1}%)  1-land keeps {:4.1}%",
            100.0 * r.mulled as f64 / h,
            100.0 * r.triple as f64 / h,
            r.lands10 / h,
            100.0 * r.low10 as f64 / h,
            100.0 * r.one_land_keeps as f64 / h
        );
    }
}

/// the first seven only: by land count, how many hands there are and how many are kept
#[test]
fn first_hand_by_lands() {
    for key in DECKS {
        let mut seen = [0u32; 8];
        let mut kept = [0u32; 8];
        for seed in 0..1500u64 {
            let mut g = table(&[key, "seph"]);
            let mut rng = sim_core::rng::Rng::named(&format!("lib:{seed}:{key}"));
            let pl = g.player_mut(P0);
            rng.shuffle(&mut pl.library);
            for _ in 0..7 {
                let c = pl.library.pop().unwrap();
                pl.hand.push(c);
            }
            let n = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count();
            let k = sim_core::impls::t6_plans::keep(&g, P0).unwrap_or(true);
            seen[n] += 1;
            kept[n] += k as u32;
        }
        let row: Vec<String> = (0..7).map(|n| format!("{n}L {:>4}/{:<4}", kept[n], seen[n])).collect();
        println!("FIRST {key:34} kept/seen: {}", row.join("  "));
    }
}

use sim_core::impls::t6_plans;

fn tbl(key: &str) -> sim_core::state::Game {
    let mut g = table(&[key, "seph", "sauron", "kefka"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

#[test]
fn a_one_lander_with_a_mox_diamond_or_a_chrome_mox_is_shipped() {
    // Mox Diamond needs a second land to discard; Chrome Mox needs spare cards to imprint; Sol Ring needs a land
    for key in [
        "kraum-tymna-blue-breach-midrange",
        "rograkh-silas-turbo-breach",
        "kinnan-bonder-mana-combo",
        "sisay-legends-toolbox",
        "dack-fayden-white-stax",
    ] {
        let mut g = tbl(key);
        hand(&mut g, P0, &["Plains", "Mox Diamond", "Rhystic Study", "Demonic Tutor", "Ad Nauseam", "Dispel", "Silence"]);
        assert_eq!(t6_plans::keep(&g, P0), Some(false), "{key}: land + Mox Diamond is one land");
    }
}

#[test]
fn a_one_lander_with_free_moxen_and_a_sol_ring_is_kept_on_the_first_seven() {
    // land + Sol Ring (2) + Petal + Amber is about 5 mana on turn 2, and the hand has things to do
    for key in ["kraum-tymna-blue-breach-midrange", "rograkh-silas-turbo-breach"] {
        let mut g = tbl(key);
        hand(&mut g, P0, &["Scrubland", "Sol Ring", "Lotus Petal", "Mox Amber", "Demonic Tutor", "Ad Nauseam", "Rhystic Study"]);
        assert_eq!(t6_plans::keep(&g, P0), Some(true), "{key}");
    }
}

#[test]
fn a_later_smaller_hand_is_kept_more_easily() {
    // land + Mox Jet + a payoff and a tutor: shipped as a first seven, kept after two mulligans
    let mut g = tbl("rograkh-silas-turbo-breach");
    hand(&mut g, P0, &["Scrubland", "Mox Amber", "Demonic Tutor", "Ad Nauseam", "Dispel", "Silence", "Swan Song"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
    g.player_mut(P0).stats.insert("mull_try", 2);
    assert_eq!(t6_plans::keep(&g, P0), Some(true));
    // a hand with no land at all is never kept
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Mox Amber", "Mox Diamond", "Lotus Petal", "Demonic Tutor", "Ad Nauseam", "Dispel", "Silence"]);
    assert_eq!(t6_plans::keep(&g, P0), Some(false));
}

#[test]
fn mox_diamond_keeps_the_only_source_of_a_colour_the_hand_needs() {
    let mut g = tbl("kraum-tymna-blue-breach-midrange");
    lands(&mut g, P0, "Island", 1, false);
    let hs = hand(&mut g, P0, &["Island", "Badlands", "Mox Diamond", "Dark Ritual", "Dispel"]);
    // Badlands is the only source of B among battlefield and hand lands, and Dark Ritual needs it
    let lands_in_hand: Vec<_> = vec![hs[0], hs[1]];
    let keys: Vec<f64> = lands_in_hand.iter().map(|&l| sim_core::engine::zones::mox_discard_key(&g, P0, l)).collect();
    assert!(keys[0] < keys[1], "the spare Island goes, not Badlands: {keys:?}");
}

#[test]
fn a_two_land_hand_does_not_cast_mox_diamond_but_a_three_land_one_does() {
    let mut g = tbl("kraum-tymna-blue-breach-midrange");
    g.player_mut(P0).turns = 2;
    let hs = hand(&mut g, P0, &["Island", "Badlands", "Mox Diamond"]);
    assert_eq!(sim_core::ai::pool::generic_prio(&g, P0, hs[2]), 0, "it would leave one land");
    hand(&mut g, P0, &["Scrubland"]);
    assert!(sim_core::ai::pool::generic_prio(&g, P0, hs[2]) > 0);
}
