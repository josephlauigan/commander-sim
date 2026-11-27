// written by an independent reviewer
//! C5a adversarial: the seating functions. The OLD `exact_seats` / `choose_seats` / `sample` are copied verbatim from
//! 08f0c483 (`git show 08f0c483:rust/crates/practice/src/session/setup.rs`); the new `choose_seats_in` at the
//! Commander format (and `choose_seats` for a Commander deck) must return exactly what they return, for every call
//! shape: default, partner, seat pick, fixed opponents, exact seats, and the error texts.

#[path = "session_common/mod.rs"]
mod sess;

use practice::session::MY_DECKS;
use practice::session::setup::{SeatArgs, choose_seats, choose_seats_in, pool_keys, pool_keys_in};
use sim_core::format::DEFAULT_FORMAT;
use sim_core::rng::Rng;
use sim_core::run::Engine;

const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

// ---------------------------------------------------------------------------------- OLD code, copied verbatim

/// k keys drawn without replacement (Python's `random.sample`)
fn sample(r: &mut Rng, keys: &[String], k: usize) -> Vec<String> {
    let mut v = keys.to_vec();
    r.shuffle(&mut v);
    v.truncate(k);
    v
}

/// a seating given in full (playing the same seed again)
fn exact_seats(e: &Engine, a: &SeatArgs, seats: &[String]) -> Result<Vec<String>, String> {
    let keys = pool_keys(e, a.tier);
    let humans: Vec<&str> = std::iter::once(a.deck).chain(a.partner).collect();
    let n = 4 - humans.len();
    let bad = seats.len() != 4
        || humans.iter().any(|h| seats.iter().filter(|s| s == h).count() != 1)
        || (0..4).any(|i| (0..i).any(|j| seats[i] == seats[j]))
        || seats.iter().any(|k| !humans.contains(&k.as_str()) && !keys.contains(k));
    if bad {
        return Err(format!(
            "seats: {} and {n} of {}, in turn order",
            if a.partner.is_some() { "your decks" } else { "your deck" },
            keys.join(", ")
        ));
    }
    Ok(seats.to_vec())
}

/// the seat order for a new game
fn old_choose_seats(e: &Engine, a: &SeatArgs) -> Result<Vec<String>, String> {
    if let Some(s) = a.seats {
        return exact_seats(e, a, s);
    }
    let keys = pool_keys(e, a.tier);
    if let Some(partner) = a.partner {
        // two people: two opponents, everyone seated at random
        if let Some(o) = a.opponents
            && (o.len() != 2 || o.iter().any(|k| !keys.contains(k)))
        {
            return Err(format!("pick two of: {}", keys.join(", ")));
        }
        if a.seat.is_some() {
            return Err("with two players the seats are drawn at random".into());
        }
        let mut seats = match a.opponents {
            Some(o) => o.to_vec(),
            None => sample(&mut Rng::named(&format!("pool:{}", a.seed)), &keys, 2),
        };
        seats.push(a.deck.to_string());
        seats.push(partner.to_string());
        Rng::named(&format!("seats:{}", a.seed)).shuffle(&mut seats);
        return Ok(seats);
    }
    let mut seats = if let Some(o) = a.opponents {
        if o.len() != 3 || o.iter().any(|k| !keys.contains(k)) {
            return Err(format!("pick three of: {}", keys.join(", ")));
        }
        let mut s = o.to_vec();
        s.push(a.deck.to_string());
        Rng::named(&format!("seats:{}", a.seed)).shuffle(&mut s);
        s
    } else {
        // pools.draw_seats: three opponents and the deck, in a random order
        let mut r = Rng::named(&format!("pool:{}", a.seed));
        let mut s = sample(&mut r, &keys, 3);
        s.push(a.deck.to_string());
        r.shuffle(&mut s);
        s
    };
    if let Some(n) = a.seat {
        if !(1..=4).contains(&n) {
            return Err("seat is 1 to 4".into());
        }
        let i = seats.iter().position(|s| s == a.deck).unwrap();
        let d = seats.remove(i);
        seats.insert(n - 1, d);
    }
    Ok(seats)
}

// ---------------------------------------------------------------------------------- the comparison

struct Tally {
    calls: usize,
    errors: usize,
}

/// old == new for one set of arguments (the Commander format, and `choose_seats` too when the deck plays Commander)
fn same(e: &Engine, a: &SeatArgs, what: &str, own_commander: bool, t: &mut Tally) {
    let old = old_choose_seats(e, a);
    let new = choose_seats_in(e, &e.formats[DEFAULT_FORMAT], a);
    assert_eq!(old, new, "choose_seats_in(commander) differs from the old choose_seats: {what}");
    if own_commander {
        assert_eq!(old, choose_seats(e, a), "choose_seats differs from the old one: {what}");
    }
    t.calls += 1;
    t.errors += usize::from(old.is_err());
}

fn s(x: &[&str]) -> Vec<String> {
    x.iter().map(|k| k.to_string()).collect()
}

/// Every deck of MY_DECKS (arzakon too, at the Commander format: the table of 4 it used to get) x t0..t6 x seeds
/// 1..=40 x every call shape, errors included.
#[test]
fn old_and_new_seating_agree_for_every_call_shape() {
    let e = sess::engine();
    let mut t = Tally { calls: 0, errors: 0 };
    let mut small_tiers = vec![];
    for tier in TIERS {
        let keys = pool_keys(&e, tier);
        // the new code is the same as the old one for a Commander table when it has the decks to fill it
        if keys.len() < 3 {
            small_tiers.push((tier, keys.len()));
        }
    }
    for deck in MY_DECKS {
        let own_commander = e.format_of(deck).key == DEFAULT_FORMAT;
        for tier in TIERS {
            let keys = pool_keys(&e, tier);
            if keys.len() < 3 {
                continue; // see `a_tier_with_too_few_decks`
            }
            for seed in 1..=40u64 {
                let base = SeatArgs { deck, tier, partner: None, seed, seat: None, opponents: None, seats: None };
                let w = |x: &str| format!("{deck} {tier} seed {seed}: {x}");
                same(&e, &base, &w("default"), own_commander, &mut t);
                // seat pick, in range and out of it
                for pick in 0..=5usize {
                    let a = SeatArgs { seat: Some(pick), ..base };
                    same(&e, &a, &w(&format!("seat {pick}")), own_commander, &mut t);
                }
                // fixed opponents: three valid ones (rotated by the seed), then the wrong shapes
                let at = |i: usize| keys[(seed as usize + i) % keys.len()].clone();
                let three = vec![at(0), at(1), at(2)];
                let good = [three.clone(), vec![at(0), at(0), at(0)]];
                for (i, o) in good.iter().enumerate() {
                    same(
                        &e,
                        &SeatArgs { opponents: Some(o), ..base },
                        &w(&format!("opponents ok {i}")),
                        own_commander,
                        &mut t,
                    );
                    for pick in [1usize, 3, 4, 5] {
                        let a = SeatArgs { opponents: Some(o), seat: Some(pick), ..base };
                        same(&e, &a, &w(&format!("opponents {i} seat {pick}")), own_commander, &mut t);
                    }
                }
                let bad: [Vec<String>; 5] = [
                    three[..2].to_vec(),
                    vec![at(0), at(1), at(2), at(3)],
                    s(&["nope", "x", "y"]),
                    vec![at(0), at(1), "nope".into()],
                    vec![],
                ];
                for (i, o) in bad.iter().enumerate() {
                    same(
                        &e,
                        &SeatArgs { opponents: Some(o), ..base },
                        &w(&format!("opponents bad {i}")),
                        own_commander,
                        &mut t,
                    );
                }
                // exact seats: what the default gives, a rotation, wrong lengths, a duplicate, an unknown key, no deck
                let natural = old_choose_seats(&e, &base).unwrap();
                let mut rot = natural.clone();
                rot.rotate_left(1);
                let mut dup = natural.clone();
                dup[0] = dup[1].clone();
                let mut unknown = natural.clone();
                let i = unknown.iter().position(|k| k != deck).unwrap();
                unknown[i] = "nope".into();
                let mut nodeck = natural.clone();
                let i = nodeck.iter().position(|k| k == deck).unwrap();
                nodeck[i] = at(0);
                let cases: [(&str, Vec<String>); 8] = [
                    ("natural", natural.clone()),
                    ("rotated", rot),
                    ("three", natural[..3].to_vec()),
                    ("five", [natural.clone(), vec![at(0)]].concat()),
                    ("dup", dup),
                    ("unknown", unknown),
                    ("no deck", nodeck),
                    ("empty", vec![]),
                ];
                for (n, sv) in &cases {
                    same(&e, &SeatArgs { seats: Some(sv), ..base }, &w(&format!("seats {n}")), own_commander, &mut t);
                    // exact seats win over seat and opponents
                    let a = SeatArgs { seats: Some(sv), seat: Some(2), opponents: Some(&three), ..base };
                    same(&e, &a, &w(&format!("seats {n} + seat + opponents")), own_commander, &mut t);
                }
                // two people: every other deck of mine as the partner
                for partner in MY_DECKS.iter().copied().filter(|p| *p != deck) {
                    let p = SeatArgs { partner: Some(partner), ..base };
                    let w = |x: &str| format!("{deck}+{partner} {tier} seed {seed}: {x}");
                    same(&e, &p, &w("default"), own_commander, &mut t);
                    for pick in [0usize, 1, 2, 4] {
                        same(
                            &e,
                            &SeatArgs { seat: Some(pick), ..p },
                            &w(&format!("seat {pick}")),
                            own_commander,
                            &mut t,
                        );
                    }
                    let two = vec![at(0), at(1)];
                    same(&e, &SeatArgs { opponents: Some(&two), ..p }, &w("opponents two"), own_commander, &mut t);
                    same(
                        &e,
                        &SeatArgs { opponents: Some(&two), seat: Some(1), ..p },
                        &w("opponents two + seat"),
                        own_commander,
                        &mut t,
                    );
                    for o in [three.clone(), vec![at(0)], s(&["nope", "x"]), vec![at(0), "nope".into()], vec![]] {
                        same(
                            &e,
                            &SeatArgs { opponents: Some(&o), ..p },
                            &w(&format!("opponents bad {o:?}")),
                            own_commander,
                            &mut t,
                        );
                    }
                    let natural = old_choose_seats(&e, &p).unwrap();
                    let mut rot = natural.clone();
                    rot.rotate_left(1);
                    let mut nopartner = natural.clone();
                    let i = nopartner.iter().position(|k| k == partner).unwrap();
                    nopartner[i] = at(0);
                    for (n, sv) in [
                        ("natural", natural.clone()),
                        ("rotated", rot),
                        ("no partner", nopartner),
                        ("three", natural[..3].to_vec()),
                    ] {
                        same(&e, &SeatArgs { seats: Some(&sv), ..p }, &w(&format!("seats {n}")), own_commander, &mut t);
                    }
                    // the seating for ONE person given to a TWO-person call (the deck counts, the partner is missing)
                    let one = old_choose_seats(&e, &base).unwrap();
                    same(
                        &e,
                        &SeatArgs { seats: Some(&one), ..p },
                        &w("one-person seats, two-person call"),
                        own_commander,
                        &mut t,
                    );
                }
            }
        }
    }
    eprintln!("compared {} calls ({} were errors); tiers with fewer than 3 decks: {small_tiers:?}", t.calls, t.errors);
    // 100_000 with four Commander decks of yours; three since the yshtola and yoshimaru keys left (2026-11-24)
    assert!(t.calls > 75_000, "the comparison did not run: {}", t.calls);
    assert!(t.errors > 10_000, "the error paths were not exercised: {}", t.errors);
}

/// A Commander table admits every pool deck, so the opponent pool is the same list the old code sorted.
#[test]
fn a_commander_table_admits_every_pool_deck() {
    let e = sess::engine();
    for tier in TIERS {
        assert_eq!(pool_keys_in(&e, tier, &e.formats[DEFAULT_FORMAT]), pool_keys(&e, tier), "{tier}");
    }
}

/// A tier with fewer than 3 pool decks (t0 may be empty): the old code seated a short table (fewer than 4 seats, a
/// game that cannot be played); the new code says so. Recorded, not a failure: the old behaviour was a bug.
#[test]
fn a_tier_with_too_few_decks_is_an_error_not_a_short_table() {
    let e = sess::engine();
    for tier in TIERS {
        let keys = pool_keys(&e, tier);
        if keys.len() >= 3 {
            continue;
        }
        let a =
            SeatArgs { deck: "karn_gilded", tier, partner: None, seed: 1, seat: None, opponents: None, seats: None };
        let old = old_choose_seats(&e, &a).unwrap();
        let new = choose_seats(&e, &a);
        eprintln!("{tier}: {} decks; old seated {} ; new: {new:?}", keys.len(), old.len());
        assert!(old.len() < 4);
        assert!(new.is_err(), "{tier}: {new:?}");
    }
}

/// Every caller of `choose_seats` (grep over rust/crates): `Session::new` (now `choose_seats_in` with the chosen
/// format), `tests/play_bot_games.rs` (arzakon at t5) and `tests/commander_practice_fixture.rs` (Commander decks only).
/// `choose_seats` is "the table of the deck's own format": identical to the old function for each Commander deck of
/// MY_DECKS, and a 2-seat table for the deck whose format has 2 seats.
#[test]
fn choose_seats_without_a_format_is_the_decks_own_table() {
    let e = sess::engine();
    let mut cali = vec![];
    for deck in MY_DECKS {
        let fmt = e.format_of(deck);
        for tier in TIERS {
            if pool_keys(&e, tier).len() < 3 {
                continue;
            }
            for seed in 1..=10u64 {
                let a = SeatArgs { deck, tier, partner: None, seed, seat: None, opponents: None, seats: None };
                let new = choose_seats(&e, &a).unwrap();
                if fmt.key == DEFAULT_FORMAT {
                    assert_eq!(new, old_choose_seats(&e, &a).unwrap(), "{deck} {tier} {seed}");
                } else {
                    assert_eq!(new.len(), fmt.seats, "{deck} {tier} {seed}: {new:?}");
                    assert!(new.contains(&deck.to_string()));
                }
            }
        }
        if fmt.key != DEFAULT_FORMAT {
            cali.push((deck, fmt.key.clone(), fmt.seats));
        }
    }
    eprintln!("MY_DECKS whose own format is not Commander: {cali:?}");
    assert!(cali.iter().all(|(_, k, n)| k == "cali" && *n == 2), "{cali:?}");
    assert!(!cali.is_empty());
}

/// A seating the old code produced for a Commander deck is one the new code accepts back as exact seats, and the
/// 2-person seats too (the save path: a loaded game passes its seats back in).
#[test]
fn old_seatings_are_accepted_back_as_exact_seats() {
    let e = sess::engine();
    for deck in ["karn_gilded", "sheoldred", "marchesa", "urdragon"] {
        for tier in ["t1", "t3", "t5", "t6"] {
            for seed in 1..=15u64 {
                let base = SeatArgs { deck, tier, partner: None, seed, seat: None, opponents: None, seats: None };
                let old = old_choose_seats(&e, &base).unwrap();
                let back = SeatArgs { seats: Some(&old), ..base };
                assert_eq!(choose_seats(&e, &back).unwrap(), old);
                let p = SeatArgs { partner: Some("arzakon"), ..base };
                let old = old_choose_seats(&e, &p).unwrap();
                assert_eq!(choose_seats(&e, &SeatArgs { seats: Some(&old), ..p }).unwrap(), old);
            }
        }
    }
}
