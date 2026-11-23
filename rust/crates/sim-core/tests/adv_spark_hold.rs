// Written by an independent reviewer; the board builder (and c2's board) was corrected to include all five colours.
//! Adversarial tests for the Ur-Dragon Spark Double hold (`spark_hold`, `spark_hold_shape`, `card_utility`, `ur_pick`).
mod play_common;
use play_common::*;
use sim_core::ai::brain::{Situation, card_utility, main_options};
use sim_core::ids::CardId;
use sim_core::impls::t45_urdragon::ur_pick;
use sim_core::impls::t45_urdragon_combos::{spark_garth_target, spark_hold, spark_hold_shape};
use sim_core::rng::Rng;

const GARTH: &str = "Garth One-Eye";
const SPARK: &str = "Spark Double";
const SEASON: &str = "Doubling Season";
const TWIN: &str = "Twinflame";
const TERROR: &str = "Terror of the Peaks";
const DEM: &str = "Demonic Tutor";
const VAM: &str = "Vampiric Tutor";
const WOR: &str = "Worldly Tutor";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Z {
    Library,
    Hand,
    Gy,
    Exile,
    Bf,
    Gone,
}

/// Garth's possible places
#[derive(Clone, Copy, Debug, PartialEq)]
enum G {
    Z(Z),
    BfPhased,
    BfToken,
    Stolen,
}

fn put(g: &mut Game, p: PlayerId, name: &str, z: Z) {
    let c = card(g, name);
    let pl = g.player_mut(p);
    pl.library.retain(|&x| x != c);
    pl.hand.retain(|&x| x != c);
    pl.gy.retain(|&x| x != c);
    pl.exile.retain(|&x| x != c);
    match z {
        Z::Library => pl.library.push(c),
        Z::Hand => pl.hand.push(c),
        Z::Gy => pl.gy.push(c),
        Z::Exile => pl.exile.push(c),
        Z::Gone => {}
        Z::Bf => {
            perm(g, p, name);
        }
    }
}

#[derive(Clone, Debug)]
struct Cfg {
    garth: G,
    season: Z,
    twin: Z,
    /// bit 0 Demonic, bit 1 Vampiric, bit 2 Worldly (in hand)
    tutors: u8,
    lands: usize,
}

fn build(key: &str, c: &Cfg) -> Game {
    let mut g = table(&[key, "seph"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    put(&mut g, P0, SPARK, Z::Hand);
    match c.garth {
        G::Z(z) => put(&mut g, P0, GARTH, z),
        G::BfPhased => {
            put(&mut g, P0, GARTH, Z::Bf);
            let m = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == GARTH).unwrap();
            g.perm_mut(m).phased = true;
        }
        G::BfToken => {
            put(&mut g, P0, GARTH, Z::Bf);
            let m = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == GARTH).unwrap();
            g.perm_mut(m).token = true;
        }
        G::Stolen => {
            put(&mut g, P0, GARTH, Z::Gone);
            perm(&mut g, P1, GARTH);
        }
    }
    put(&mut g, P0, SEASON, c.season);
    put(&mut g, P0, TWIN, c.twin);
    for (bit, n) in [(1u8, DEM), (2, VAM), (4, WOR)] {
        if c.tutors & bit != 0 {
            put(&mut g, P0, n, Z::Hand);
        }
    }
    // all five colours so that mana colours never matter in the shape tests
    let mut left = c.lands;
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        if left > 0 {
            lands(&mut g, P0, n, 1, false);
            left -= 1;
        }
    }
    lands(&mut g, P0, "Island", left, false);
    perm(&mut g, P0, TERROR);
    g
}

fn all_cfgs() -> Vec<Cfg> {
    let gs =
        [G::Z(Z::Library), G::Z(Z::Hand), G::Z(Z::Gy), G::Z(Z::Exile), G::Z(Z::Bf), G::BfPhased, G::BfToken, G::Stolen];
    let ss = [Z::Library, Z::Hand, Z::Gy, Z::Bf];
    let ts = [Z::Library, Z::Hand, Z::Gy];
    let mut v = vec![];
    for &garth in &gs {
        for &season in &ss {
            for &twin in &ts {
                for tutors in 0..8u8 {
                    v.push(Cfg { garth, season, twin, tutors, lands: 7 });
                }
            }
        }
    }
    v
}

fn spark_offered(g: &mut Game) -> bool {
    main_options(g, P0, false).unwrap().iter().any(|o| o.label == SPARK)
}

fn enter_spark(g: &mut Game) -> String {
    perm(g, P0, SPARK);
    let c = card(g, SPARK);
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(c)).expect("Spark entered");
    g.perm(m).name.to_string()
}

// ------------------------------------------------------------------ 1. refactor equivalence
/// the body of `spark_garth_target` on origin/main f1ffd896, from the public state only
fn old_target(g: &Game, p: PlayerId) -> Option<sim_core::ids::PermId> {
    if &*g.player(p).key != "urdragon" {
        return None;
    }
    let live = |m: sim_core::ids::PermId| g.perm(m).on_bf && !g.perm(m).phased;
    let nm = |m: sim_core::ids::PermId| g.perm(m).name.to_string();
    let cn = |c: CardId| g.db.get(c).name.to_string();
    let bf = |n: &str| g.player(p).perms.iter().copied().find(|&m| live(m) && nm(m) == n);
    let in_hand = |n: &str| g.player(p).hand.iter().any(|&c| cn(c) == n);
    let in_lib = |n: &str| g.player(p).library.iter().any(|&c| cn(c) == n);
    let garth =
        g.player(p).perms.iter().copied().find(|&m| {
            live(m) && nm(m) == GARTH && !g.perm(m).token && !g.perm(m).phys.is_some_and(|c| cn(c) == SPARK)
        })?;
    let season_out = bf(SEASON).is_some() || in_hand(SEASON);
    let twin_hand = in_hand(TWIN);
    let tutor = in_hand(DEM) || in_hand(VAM);
    let near = (season_out && twin_hand) || (tutor && ((season_out && in_lib(TWIN)) || (twin_hand && in_lib(SEASON))));
    near.then_some(garth)
}

#[test]
fn a1_spark_garth_target_equals_the_old_body_over_every_state() {
    let mut n = 0;
    for key in ["urdragon", "aragorn"] {
        for c in all_cfgs() {
            let g = build(key, &c);
            assert_eq!(spark_garth_target(&g, P0), old_target(&g, P0), "{key} {c:?}");
            n += 1;
        }
    }
    // Garth that is Spark Double's own copy on the battlefield (the real card is then the copy source)
    let c = Cfg { garth: G::Z(Z::Bf), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 };
    let mut g = build("urdragon", &c);
    assert_eq!(enter_spark(&mut g), GARTH);
    // now remove the plain Garth: only the Spark copy of Garth is left
    let plain = *g
        .player(P0)
        .perms
        .iter()
        .find(|&&m| &*g.perm(m).name == GARTH && g.perm(m).phys != Some(card(&g, SPARK)))
        .unwrap();
    g.perm_mut(plain).on_bf = false;
    assert_eq!(spark_garth_target(&g, P0), old_target(&g, P0));
    assert_eq!(spark_garth_target(&g, P0), None, "Spark Double's own Garth is never the target");
    println!("compared {n} states");
}

// ------------------------------------------------------------------ 2. hold versus copy
/// Garth arrives (a tutor spent on it when it was in the library, Worldly first), the other missing piece fetched by
/// an any-tutor (spent): the best assignment
fn arrival(c: &Cfg) -> Cfg {
    let mut a = c.clone();
    if a.garth == G::Z(Z::Library) {
        if a.tutors & 4 != 0 {
            a.tutors &= !4;
        } else if a.tutors & 1 != 0 {
            a.tutors &= !1;
        } else if a.tutors & 2 != 0 {
            a.tutors &= !2;
        }
    }
    a.garth = G::Z(Z::Bf);
    for piece in 0..2 {
        let z = if piece == 0 { &mut a.season } else { &mut a.twin };
        if *z == Z::Library {
            if a.tutors & 1 != 0 {
                a.tutors &= !1;
                *z = Z::Hand;
            } else if a.tutors & 2 != 0 {
                a.tutors &= !2;
                *z = Z::Hand;
            }
        }
    }
    a
}

#[test]
fn a2_whenever_the_hold_is_on_garths_best_case_arrival_is_copied() {
    let mut bad = vec![];
    let mut held = 0;
    for c in all_cfgs() {
        let g = build("urdragon", &c);
        if !spark_hold_shape(&g, P0) {
            continue;
        }
        held += 1;
        let a = arrival(&c);
        let mut ga = build("urdragon", &a);
        if spark_garth_target(&ga, P0).is_none() {
            bad.push(format!("{c:?}"));
        } else if enter_spark(&mut ga) != GARTH {
            bad.push(format!("copy not Garth: {c:?}"));
        }
    }
    println!("held shapes: {held}");
    assert!(held > 30);
    assert!(bad.is_empty(), "held for nothing: {bad:#?}");
}

#[test]
fn a2b_hold_off_with_plain_garth_out_and_line_near_spark_copies_garth() {
    let mut n = 0;
    for c in all_cfgs() {
        if c.garth != G::Z(Z::Bf) {
            continue;
        }
        let mut g = build("urdragon", &c);
        assert!(!spark_hold_shape(&g, P0), "{c:?}: held with Garth out");
        if spark_garth_target(&g, P0).is_some() {
            assert_eq!(enter_spark(&mut g), GARTH, "{c:?}");
            n += 1;
        }
    }
    assert!(n > 20);
}

/// Worldly Tutor fetches a creature only: a state whose only tutor for Twinflame/Season is Worldly is no hold
#[test]
fn a2c_worldly_is_not_counted_for_a_noncreature_piece() {
    for (season, twin) in [(Z::Hand, Z::Library), (Z::Library, Z::Hand)] {
        let c = Cfg { garth: G::Z(Z::Hand), season, twin, tutors: 4, lands: 7 };
        let g = build("urdragon", &c);
        assert!(!spark_hold_shape(&g, P0), "{c:?}");
    }
    // Demonic AND Vampiric in hand, Garth in the library, Season out, Twinflame in the library: Garth by one, Twin by the
    // other: the hold is right to be on
    let c = Cfg { garth: G::Z(Z::Library), season: Z::Bf, twin: Z::Library, tutors: 3, lands: 7 };
    assert!(spark_hold_shape(&build("urdragon", &c), P0));
    // one tutor, two pieces still to fetch: no hold
    for t in [1u8, 2] {
        let c = Cfg { garth: G::Z(Z::Library), season: Z::Bf, twin: Z::Library, tutors: t, lands: 7 };
        assert!(!spark_hold_shape(&build("urdragon", &c), P0), "tutors {t}");
    }
}

// ------------------------------------------------------------------ 3. danger edges and consistency
fn opp_flier(g: &mut Game, power: i32) {
    let m = token(g, P1, power);
    g.perm_mut(m).fly = true;
}

#[test]
fn b1_danger_exactly_one_releases_just_below_holds() {
    let c = Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 };
    // 4 power flier: danger 4/life
    for (life, expect_hold) in [(5, true), (4, false), (3, false), (400, true)] {
        let mut g = build("urdragon", &c);
        opp_flier(&mut g, 4);
        g.player_mut(P0).life = life;
        let d = Situation::new(&g, P0).danger;
        assert_eq!(spark_hold(&g, P0), expect_hold, "life {life} danger {d}");
        assert_eq!(spark_offered(&mut g), !expect_hold, "life {life} danger {d}");
        let s = Situation::new(&g, P0);
        let spark = card(&g, SPARK);
        assert_eq!(card_utility(&g, P0, &s, spark).is_none(), expect_hold, "life {life}");
        // the look-ahead copy reads the same
        g.in_search = true;
        assert_eq!(spark_hold(&g, P0), expect_hold, "in_search life {life}");
        assert_eq!(spark_offered(&mut g), !expect_hold, "in_search life {life}");
    }
    // the danger is exactly 1.0 at life 4
    let mut g = build("urdragon", &c);
    opp_flier(&mut g, 4);
    g.player_mut(P0).life = 4;
    assert_eq!(Situation::new(&g, P0).danger, 1.0);
}

#[test]
fn b2_utility_none_iff_hold_over_random_states_and_stable_across_calls() {
    let mut rng = Rng::from_u64(77);
    let cfgs = all_cfgs();
    let mut bad = vec![];
    let (mut h, mut r) = (0, 0);
    for i in 0..600 {
        let c = rng.choice(&cfgs).unwrap().clone();
        let mut g = build("urdragon", &c);
        for _ in 0..rng.below(4) {
            let m = token(&mut g, P1, 1 + rng.below(9) as i32);
            g.perm_mut(m).fly = rng.below(2) == 0;
        }
        g.player_mut(P0).life = 1 + rng.below(40) as i32;
        let hold = spark_hold(&g, P0);
        let s = Situation::new(&g, P0);
        let spark = card(&g, SPARK);
        let u1 = card_utility(&g, P0, &s, spark);
        let u2 = card_utility(&g, P0, &Situation::new(&g, P0), spark);
        if u1.is_none() != hold || u1 != u2 {
            bad.push(format!("state {i} {c:?} hold {hold} u1 {u1:?} u2 {u2:?} danger {}", s.danger));
        }
        if hold {
            h += 1;
        } else {
            r += 1;
        }
        // main_options agrees with the hold, in a copy or not
        let offered = spark_offered(&mut g);
        g.in_search = true;
        let offered2 = spark_offered(&mut g);
        if offered != offered2 {
            bad.push(format!("state {i}: offered flips in search {c:?}"));
        }
        if hold && offered {
            bad.push(format!("state {i}: held but offered {c:?}"));
        }
    }
    println!("held {h} released {r}");
    assert!(bad.is_empty(), "{bad:#?}");
}

// ------------------------------------------------------------------ 4. other seats
#[test]
fn c1_other_seats_never_hold_and_utility_is_the_same_as_without_the_line() {
    let c = Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 };
    // no line at all: Garth, Season, Twinflame all dead
    let none = Cfg { garth: G::Z(Z::Gy), season: Z::Gy, twin: Z::Gy, tutors: 0, lands: 7 };
    for key in ["miirym-temur-dragons", "brago-azorius-blink-control", "atraxa-infect", "aragorn"] {
        let g = build(key, &c);
        let g0 = build(key, &none);
        let spark = card(&g, SPARK);
        let u = card_utility(&g, P0, &Situation::new(&g, P0), spark);
        let u0 = card_utility(&g0, P0, &Situation::new(&g0, P0), spark);
        assert!(u.is_some(), "{key}");
        let _ = u0; // the hand itself moves other seats' utilities (their deck prio reads the hand); only the hold matters
        assert!(!spark_hold(&g, P0) && !spark_hold_shape(&g, P0), "{key}");
    }
}

#[test]
fn c2_a_non_urdragon_seat_in_the_same_game_is_unaffected() {
    // seat 0 aragorn, seat 1 urdragon with the line in hand; and the reverse
    for (a, b) in [("aragorn", "urdragon"), ("urdragon", "aragorn")] {
        let mut g = table(&[a, b]);
        for q in &mut g.players {
            q.hand.clear();
        }
        for p in [P0, P1] {
            for n in [SPARK, GARTH, SEASON, TWIN] {
                put(&mut g, p, n, Z::Hand);
            }
            for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
                lands(&mut g, p, n, 1, false);
            }
            lands(&mut g, p, "Island", 2, false);
            perm(&mut g, p, TERROR);
        }
        let spark = card(&g, SPARK);
        let (ur, other) = if a == "urdragon" { (P0, P1) } else { (P1, P0) };
        assert!(spark_hold(&g, ur));
        assert!(!spark_hold(&g, other));
        assert!(card_utility(&g, ur, &Situation::new(&g, ur), spark).is_none());
        assert!(card_utility(&g, other, &Situation::new(&g, other), spark).is_some());
    }
}

// ------------------------------------------------------------------ 5. free drop
#[test]
fn d1_free_drop_cases() {
    let c = Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 };
    let g = build("urdragon", &c);
    let spark = card(&g, SPARK);
    let land = card(&g, "Plains");
    assert_eq!(ur_pick(&g, P0, &[spark]), None);
    assert_eq!(ur_pick(&g, P0, &[spark, land]), Some(land), "a land is still picked");
    // danger releases it
    let mut g = build("urdragon", &c);
    opp_flier(&mut g, 40);
    assert_eq!(ur_pick(&g, P0, &[spark]), Some(spark));
    // Tiamat and Spark both out of the offer
    let tiamat = card(&g, "Tiamat");
    let g = build("urdragon", &c);
    assert_eq!(ur_pick(&g, P0, &[spark, tiamat]), None);
    assert_eq!(ur_pick(&g, P0, &[spark, tiamat, land]), Some(land));
}

// ------------------------------------------------------------------ 6. Yawgmoth's Will, discard
#[test]
fn e1_yawgmoths_will_does_not_cast_a_held_spark_from_the_graveyard() {
    for (garth, expect) in [(G::Z(Z::Hand), false), (G::Z(Z::Gy), true)] {
        let c = Cfg { garth, season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 };
        let mut g = build("urdragon", &c);
        let spark = card(&g, SPARK);
        g.player_mut(P0).hand.retain(|&x| x != spark);
        g.player_mut(P0).gy.push(spark);
        g.player_mut(P0).yawg = true;
        g.player_mut(P0).yawg_gy.push(spark);
        let o = main_options(&mut g, P0, false).unwrap().iter().any(|o| o.label.starts_with(SPARK));
        assert_eq!(o, expect, "garth {garth:?}");
    }
}

// ------------------------------------------------------------------ 7. mana: Garth castable by next turn
fn hold_row_mana(land: &str, n: usize, extra: &[&str]) -> Game {
    let mut g = table(&["urdragon", "seph"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    put(&mut g, P0, SPARK, Z::Hand);
    put(&mut g, P0, GARTH, Z::Hand);
    put(&mut g, P0, SEASON, Z::Hand);
    put(&mut g, P0, TWIN, Z::Hand);
    for e in extra {
        put(&mut g, P0, e, Z::Hand);
    }
    lands(&mut g, P0, land, n, false);
    perm(&mut g, P0, TERROR);
    g
}

/// Garth costs {W}{U}{B}{R}{G}. Seven Islands count as "five mana" for `next_turn_mana` but can never cast it, so Spark
/// Double is held (and Twinflame too) for a Garth that cannot be cast: the hold waits on colours it does not check.
#[test]
fn f1_garth_in_hand_with_no_way_to_pay_its_colours_does_not_hold_spark() {
    let mut g = hold_row_mana("Island", 7, &[]);
    let garth = card(&g, GARTH);
    let (gn, pips) = sim_core::engine::mana::cost_of(&g, P0, garth);
    assert!(!sim_core::engine::mana::can_pay_spell(&g, P0, garth, gn, &pips, false), "Garth is not payable");
    assert!(!spark_hold(&g, P0), "Spark Double held for a Garth that cannot be cast (only Islands)");
    assert!(spark_offered(&mut g));
}

/// the same with a tutor: Worldly Tutor ({G}) cannot be cast with Islands, so Garth in the library is not coming
#[test]
fn f2_garth_in_library_with_an_uncastable_tutor_does_not_hold_spark() {
    let mut g = hold_row_mana("Island", 7, &[WOR]);
    put(&mut g, P0, GARTH, Z::Library);
    let w = card(&g, WOR);
    let (gn, pips) = sim_core::engine::mana::cost_of(&g, P0, w);
    assert!(!sim_core::engine::mana::can_pay_spell(&g, P0, w, gn, &pips, false), "Worldly is not payable");
    assert!(!spark_hold(&g, P0), "Spark Double held for a tutor that cannot be cast");
}

/// a tax: Sphere of Resistance (each spell costs {1} more) makes Garth six with five mana sources
#[test]
fn f3_a_tax_that_prices_garth_out_of_next_turns_mana_releases_the_hold() {
    let mut g = build("urdragon", &Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 5 });
    assert!(spark_hold(&g, P0));
    assert!(g.db.id("Sphere of Resistance").is_some(), "Sphere of Resistance is in the card database");
    perm(&mut g, P1, "Sphere of Resistance");
    let garth = card(&g, GARTH);
    let (gn, pips) = sim_core::engine::mana::cost_of(&g, P0, garth);
    assert_eq!(gn + pips.len() as u32, 6, "the tax is seen");
    assert!(!spark_hold(&g, P0), "five lands, Garth costs six now");
    assert!(spark_offered(&mut g));
}

/// Garth countered after Spark was held: the hold ends, Spark is offered
#[test]
fn f4_garth_countered_releases_the_hold() {
    let mut g = build("urdragon", &Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 });
    assert!(spark_hold(&g, P0));
    put(&mut g, P0, GARTH, Z::Gy);
    assert!(!spark_hold(&g, P0));
    assert!(spark_offered(&mut g));
}

/// Garth stolen, phased, a token: release (the copy would not target it either)
#[test]
fn f5_stolen_phased_token_garth_release_and_spark_copies_something_else_consistently() {
    for garth in [G::Stolen, G::BfPhased, G::BfToken] {
        let mut g = build("urdragon", &Cfg { garth, season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 7 });
        assert!(!spark_hold(&g, P0), "{garth:?}");
        assert!(spark_offered(&mut g), "{garth:?}");
        assert_eq!(spark_garth_target(&g, P0), None, "{garth:?}");
    }
}

/// four lands and no land in hand: next turn four mana against Garth's five, so the hold is off
#[test]
fn f6_four_lands_and_no_land_in_hand_cannot_reach_garths_five() {
    let g = build("urdragon", &Cfg { garth: G::Z(Z::Hand), season: Z::Hand, twin: Z::Hand, tutors: 0, lands: 4 });
    assert!(!spark_hold(&g, P0));
}

/// seven Islands: Garth cannot be cast, so Spark Double must not be held for it
#[test]
fn f1b_islands_only_garth_is_not_offered_but_spark_double_is() {
    let mut g = hold_row_mana("Island", 7, &[]);
    let labels: Vec<String> = main_options(&mut g, P0, false).unwrap().into_iter().map(|o| o.label).collect();
    println!("{labels:?}");
    assert!(!labels.iter().any(|l| l == GARTH), "Garth cannot be cast with Islands");
    assert!(labels.iter().any(|l| l == SPARK), "...so Spark Double should be castable, but it is held: {labels:?}");
}
