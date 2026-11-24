//! Two-seat tables (Cali Commander, 1v1): which opponent a seed seats the deck against, and who is on the play.
//!
//! A 4-seat table is drawn from a random stream (`pyrand::draw_seats`). A 2-seat table reads no random stream at all:
//! the opponents are the format's admitted pool decks of one tier in sorted key order, each met on two consecutive
//! seeds (`seed / 2`), and the deck is on the play (seat 0, which skips its first draw under Cali rules) on the even
//! seed and on the draw on the odd one. Over 2 x n consecutive seeds from an even seed, each of n opponents is met once
//! on the play and once on the draw. Why `seed / 2` and not `seed % n`: with an even number of opponents, `seed % n`
//! and the seed's parity would meet each opponent only on the play or only on the draw.
//!
//! The opponents are 4-player pool lists playing 1v1 (a Cali pool is a later decision), and until the format AI profile
//! exists (C4) the AI is the 4-player AI: every Cali report says so (`ai_label`).

use crate::args::Args;
use crate::play::Sim;
use sim_core::format::Format;

/// The index (into the sorted opponent keys) of the opponent seed `seed` meets: the opponent is `sorted_keys[(seed / 2) % n]`,
/// so adding or removing a pool deck in a tier re-deals the opponents of nearly every seed: Cali numbers from before and
/// after a pool change are not comparable seed for seed (paired comparisons with the same list in both arms are fine).
pub fn opponent_index(seed: u64, opponents: usize) -> usize {
    ((seed / 2) % opponents as u64) as usize
}

/// is the deck on the play (seat 0, the starting player) at this seed?
pub fn on_the_play(seed: u64) -> bool {
    seed % 2 == 0
}

/// The seat keys of the 1v1 game of `seed`: the deck and one of `opp_keys`, in seat order. Reads no random stream;
/// depends only on the seed and the key lists, so two versions of a deck (a `--swap`) meet the same opponent in the
/// same order.
pub fn duel_seats(seed: u64, me: &str, opp_keys: &[String]) -> Vec<String> {
    assert!(!opp_keys.is_empty(), "a 1v1 table needs at least one opponent");
    let mut sorted = opp_keys.to_vec();
    sorted.sort();
    let opp = sorted[opponent_index(seed, sorted.len())].clone();
    if on_the_play(seed) { vec![me.to_string(), opp] } else { vec![opp, me.to_string()] }
}

/// Games on the play and on the draw against each of `opponents` opponents (sorted key order) over the seeds
/// `seed0 .. seed0 + n`
pub fn play_draw_counts(seed0: u64, n: u64, opponents: usize) -> Vec<(u64, u64)> {
    let mut v = vec![(0u64, 0u64); opponents];
    for s in seed0..seed0.saturating_add(n) {
        let e = &mut v[opponent_index(s, opponents)];
        if on_the_play(s) {
            e.0 += 1;
        } else {
            e.1 += 1;
        }
    }
    v
}

/// does every opponent meet the deck equally often on the play and on the draw over these seeds?
pub fn is_balanced(seed0: u64, n: u64, opponents: usize) -> bool {
    opponents > 0 && play_draw_counts(seed0, n, opponents).iter().all(|(p, d)| p == d)
}

/// the note a run prints when its seeds do not balance play and draw (None: balanced, or not a 2-seat table)
pub fn balance_note(seed0: u64, n: u64, opponents: usize) -> Option<String> {
    if is_balanced(seed0, n, opponents) {
        return None;
    }
    Some(format!(
        "note: seeds {seed0}..{} do not put the deck on the play and on the draw equally often against every opponent \
         ({opponents} opponents: start from an even seed and play a multiple of {} games for an exact balance)",
        seed0.saturating_add(n.saturating_sub(1)),
        2 * opponents.max(1)
    ))
}

/// the even share of a table of this many seats, in percent: what a deck wins by chance (25 for 4 seats, 50 for 2)
pub fn even_share_pct(seats: usize) -> usize {
    100 / seats.max(1)
}

/// "4-player baseline 25%" / "2-player baseline 50%"
pub fn baseline_text(seats: usize) -> String {
    format!("{seats}-player baseline {}%", even_share_pct(seats))
}

/// Is this format's table 2 seats? A 2-seat run needs the labels, the play / draw split and the seed balance; a
/// 4-seat run prints exactly what it always printed.
pub fn is_duel(fmt: &Format) -> bool {
    fmt.seats == 2
}

/// What every report of a 2-seat run says about its AI: until the format AI profile exists, the 4-player AI plays 1v1.
/// None for a 4-seat format.
pub fn ai_label(fmt: &Format) -> Option<String> {
    is_duel(fmt).then(|| format!("{}, 4-player AI, no format AI profile yet (C4)", fmt.name))
}

/// The line a 2-seat run prints before it plays: the format, its rules, and what the opponents are. None for a 4-seat
/// format.
pub fn describe(fmt: &Format) -> Option<String> {
    is_duel(fmt).then(|| {
        format!(
            "Format: {}, 1v1, {} life{}; the opponents are 4-player pool lists playing 1v1 ({})",
            fmt.name,
            fmt.life,
            if fmt.first_player_draws { "" } else { ", the starting player skips its first draw" },
            ai_label(fmt).unwrap_or_default()
        )
    })
}

/// The refusal of a mode that is 4-seat by construction, for a deck of a 2-seat format: the seat shares of `--calibrate`,
/// `--realism` and `--fit`, the Tier 6 reports, and the keep / mulligan oracle's real-game arm all read four seats.
pub fn refuse(mode: &str, deck: Option<&str>, fmt: &Format) -> String {
    let who =
        deck.map(|d| format!("{d} plays {}", fmt.name)).unwrap_or_else(|| format!("--format selects {}", fmt.name));
    format!("{mode}: {who} (1v1, {} seats), and {mode} is not available for {} yet", fmt.seats, fmt.name)
}

/// The five modes that read four seats are refused, each with its name, when the run's table is a 2-seat format: a
/// deck of that format (`--deck`, or a key of `--mulligan-oracle`) or `--format` naming one. Exit 1; nothing is played.
pub fn check_modes(sim: &Sim, a: &Args) -> Result<(), String> {
    let deck_fmt = |d: &str| sim.format_for(d);
    let pool_fmt = || sim.requested_format();
    let check = |mode: &str, fmt: Option<&Format>, deck: Option<&str>| match fmt {
        Some(f) if is_duel(f) => Err(refuse(mode, deck, f)),
        _ => Ok(()),
    };
    // `--realism` and `--fit` take no deck (they refuse `--deck` themselves); `--calibrate` ignores one, so its deck counts
    for (mode, on) in [("--calibrate", a.calibrate.is_some()), ("--realism", a.realism), ("--fit", a.fit)] {
        if on {
            match a.deck.as_deref().filter(|_| mode == "--calibrate") {
                Some(d) => check(mode, Some(deck_fmt(d)), Some(d))?,
                None => check(mode, pool_fmt(), None)?,
            }
        }
    }
    if let Some(spec) = a.mulligan_oracle.as_deref() {
        let mode = if a.vs_tier { "--vs-tier" } else { "--mulligan-oracle" };
        check(mode, pool_fmt(), None)?;
        for k in spec.split(',').map(str::trim).filter(|k| !k.is_empty() && *k != "t6") {
            if sim.engine.decks.contains_key(k) {
                check(mode, Some(deck_fmt(k)), Some(k))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("deck-{}", (b'a' + i as u8) as char)).collect()
    }

    /// over 2 x n consecutive seeds from an even seed, each opponent meets the deck once on the play and once on the
    /// draw, for an even and an odd number of opponents, and at the default seed
    #[test]
    fn duel_seats_balance_play_and_draw_over_two_n_seeds() {
        for n in 1..=8usize {
            let opp = keys(n);
            for seed0 in [0u64, 2, 500000, 123456] {
                let mut play: Vec<(String, u32)> = vec![];
                let mut draw: Vec<(String, u32)> = vec![];
                for s in seed0..seed0 + 2 * n as u64 {
                    let seats = duel_seats(s, "me", &opp);
                    assert_eq!(seats.len(), 2);
                    let (side, other) = if seats[0] == "me" { (&mut play, &seats[1]) } else { (&mut draw, &seats[0]) };
                    assert_ne!(other, "me");
                    match side.iter_mut().find(|(k, _)| k == other) {
                        Some(e) => e.1 += 1,
                        None => side.push((other.clone(), 1)),
                    }
                }
                let mut want: Vec<(String, u32)> = opp.iter().map(|k| (k.clone(), 1)).collect();
                play.sort();
                draw.sort();
                want.sort();
                assert_eq!(play, want, "{n} opponents from seed {seed0}: on the play");
                assert_eq!(draw, want, "{n} opponents from seed {seed0}: on the draw");
                assert!(is_balanced(seed0, 2 * n as u64, n));
            }
        }
    }

    /// each opponent holds the seat on two consecutive seeds, the deck is on the play on the even one (seat 0 skips its
    /// first draw), and the order of the opponent list does not matter
    #[test]
    fn a_pair_of_seeds_is_one_opponent_both_ways() {
        let opp = keys(5);
        let mut shuffled = opp.clone();
        shuffled.reverse();
        for s in (500000u64..500020).step_by(2) {
            let a = duel_seats(s, "me", &opp);
            let b = duel_seats(s + 1, "me", &opp);
            assert_eq!((a[0].as_str(), b[1].as_str()), ("me", "me"));
            assert_eq!(a[1], b[0], "seed {s}: the same opponent on both seeds");
            assert_eq!(duel_seats(s, "me", &shuffled), a, "seed {s}: the key order is not a matter");
        }
    }

    #[test]
    fn the_default_run_balances_and_other_runs_say_so() {
        // 1500 games from 500000 against 5 opponents (the default run at t5 or at t6 without Dack)
        assert!(is_balanced(500000, 1500, 5));
        assert!(balance_note(500000, 1500, 5).is_none());
        // 1500 = 125 x 12 also balances against 6 opponents
        assert!(is_balanced(500000, 1500, 6));
        // a count that is not a multiple of 2 x n leaves some opponent met more often on one side
        let note = balance_note(500000, 1499, 5).unwrap();
        assert!(note.contains("500000") && note.contains("multiple of 10"), "{note}");
        assert!(balance_note(500000, 7, 5).is_some() && balance_note(500001, 7, 5).is_some());
        // an odd first seed starts on a draw and ends on a play, so a whole number of 2 x n still balances
        assert!(is_balanced(500001, 1500, 5));
        assert_eq!(play_draw_counts(500000, 20, 5), vec![(2, 2); 5]);
    }

    // ---- against the real decks (data/decks.json, decklists/pool): the tables, the opponents, the rules

    use crate::decks::MINE;
    use crate::play::RunOpts;
    use sim_core::engine::turn::Seat;
    use sim_core::format::Rules;
    use sim_core::settings::{AiMode, Profile};

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    /// a seat list as the games read it: key, name, commander and the card list
    fn id(s: &[Seat]) -> Vec<(String, String, String, Vec<String>)> {
        s.iter()
            .map(|x| {
                (
                    x.key.to_string(),
                    x.name.to_string(),
                    format!("{:?}", x.commander),
                    x.cards.iter().map(|c| format!("{c:?}")).collect(),
                )
            })
            .collect()
    }

    /// Commander is byte-identical: `table_for` with the Commander format is `seats_for(.., 3)` (or 4 pool decks without a
    /// deck), seat for seat, for 200 seeds, every deck of yours against every tier it could meet
    #[test]
    fn table_for_commander_is_seats_for_three_for_200_seeds() {
        let sim = sim();
        let cmd = &sim.engine.formats["commander"];
        for me in ["yshtola", "sheoldred", "urdragon", "yoshimaru", "arzakon"] {
            for tier in ["t3", "t5", "t6"] {
                let keys = sim.opponents(tier, cmd);
                assert_eq!(keys, sim.pool_keys(tier), "a Commander table admits every pool deck");
                for seed in 500000..500200u64 {
                    let want = sim.seats_for(seed, Some(me), None, &keys, 3);
                    let got = sim.table_for(seed, Some(me), None, &keys, cmd);
                    assert_eq!(id(&got), id(&want), "{me} {tier} seed {seed}");
                    assert_eq!(got.len(), 4);
                }
            }
        }
        // no deck: four pool decks, as `--pool tN --trace` draws them
        for seed in 500000..500200u64 {
            let keys = sim.pool_keys("t4");
            assert_eq!(
                id(&sim.table_for(seed, None, None, &keys, cmd)),
                id(&sim.seats_for(seed, None, None, &keys, 4))
            );
        }
        // with a variant list: the deck's own seat is the variant on both paths
        let var = sim.seat("yshtola", &sim.mine["yshtola"][1..].to_vec()).unwrap();
        let keys = sim.pool_keys("t5");
        for seed in 500000..500050u64 {
            assert_eq!(
                id(&sim.table_for(seed, Some("yshtola"), Some(&var), &keys, cmd)),
                id(&sim.seats_for(seed, Some("yshtola"), Some(&var), &keys, 3))
            );
        }
    }

    /// a Cali table is two seats under the Cali rules; the settings carry them and Commander's stay Commander's
    #[test]
    fn a_cali_table_has_two_seats_and_the_cali_rules() {
        let sim = sim();
        let (cali, cmd) = (&sim.engine.formats["cali"], &sim.engine.formats["commander"]);
        let keys = sim.opponents("t5", cali);
        for seed in 500000..500020u64 {
            let seats = sim.table_for(seed, Some("arzakon"), None, &keys, cali);
            assert_eq!(seats.len(), 2);
            assert_eq!(seats.iter().filter(|x| &*x.key == "arzakon").count(), 1);
            assert_eq!(
                &*seats[usize::from(seed % 2 == 1)].key,
                "arzakon",
                "seed {seed}: the deck is on the play on an even seed"
            );
        }
        assert_eq!(
            sim.settings_vs_for(Profile::Loose, &keys, cali).rules,
            Rules { life: 30, first_player_draws: false }
        );
        assert_eq!(sim.settings_vs_for(Profile::Loose, &keys, cmd).rules, Rules::COMMANDER);
        assert_eq!(sim.settings_in_for(Profile::Loose, None, cali).rules.life, 30);
        // the default settings are untouched
        assert_eq!(sim.settings_vs(Profile::Loose, &keys).rules, Rules::COMMANDER);
        // a deck plays its own format unless `--format` says another
        assert_eq!(sim.format_for("arzakon").key, "cali");
        assert_eq!(sim.format_for("yoshimaru").key, "commander");
        assert_eq!(sim.format_for("dack-fayden-white-stax").key, "commander");
        assert!(describe(cali).unwrap().contains("30 life") && describe(cmd).is_none());
        assert_eq!(ai_label(cali).unwrap(), "Cali Commander, 4-player AI, no format AI profile yet (C4)");
        assert!(ai_label(cmd).is_none() && !is_duel(cmd));
    }

    /// the opponents of a Cali table are exactly the admitted pool decks of the tier: t5 whole, t6 without the Tabernacle
    /// deck (D4), five each; every Commander tier is whole
    #[test]
    fn the_opponent_list_per_tier_is_exactly_the_admitted_decks() {
        let sim = sim();
        let (cali, cmd) = (&sim.engine.formats["cali"], &sim.engine.formats["commander"]);
        let t5 = sim.opponents("t5", cali);
        let t6 = sim.opponents("t6", cali);
        assert_eq!(t5, sim.pool_keys("t5"));
        assert_eq!((t5.len(), t6.len()), (5, 5));
        assert_eq!(sim.pool_keys("t6").len(), 6);
        assert!(!t6.iter().any(|k| k == "dack-fayden-white-stax"));
        let mut want = sim.pool_keys("t6");
        want.retain(|k| k != "dack-fayden-white-stax");
        assert_eq!(t6, want);
        for t in crate::decks::TIERS {
            assert_eq!(sim.opponents(t, cmd), sim.pool_keys(t), "{t}");
            assert!(sim.opponents(t, cali).iter().all(|k| sim.engine.admits(k, "cali").is_ok()));
        }
    }

    /// over 2 x n seeds from the default seed, arzakon meets each admitted deck of a tier once on the play and once on the
    /// draw, and the tiers are never mixed in one table
    #[test]
    fn arzakon_meets_every_opponent_once_on_each_side() {
        let sim = sim();
        let cali = &sim.engine.formats["cali"];
        for tier in ["t5", "t6"] {
            let keys = sim.opponents(tier, cali);
            let mut play: Vec<String> = vec![];
            let mut draw: Vec<String> = vec![];
            for seed in 500000..500000 + 2 * keys.len() as u64 {
                let seats = sim.table_for(seed, Some("arzakon"), None, &keys, cali);
                let (side, other) =
                    if &*seats[0].key == "arzakon" { (&mut play, &seats[1]) } else { (&mut draw, &seats[0]) };
                assert!(keys.iter().any(|k| other.key == k.as_str()), "{tier}: {} is from the tier", other.key);
                side.push(other.key.to_string());
            }
            let mut want = keys.clone();
            want.sort();
            play.sort();
            draw.sort();
            assert_eq!((&play, &draw), (&want, &want), "{tier}");
        }
    }

    /// `--format`: the key must exist, the format must admit the deck, and a Cali deck is refused at a Commander table
    #[test]
    fn the_format_of_a_run_is_checked() {
        let mut sim = sim();
        assert_eq!(sim.check_format("arzakon").unwrap().key, "cali");
        assert!(sim.set_format("nowhere").unwrap_err().contains("--format nowhere"));
        sim.set_format("commander").unwrap();
        let e = sim.check_format("arzakon").unwrap_err();
        assert!(e.contains("arzakon") && e.contains("Cali Commander") && e.contains("Commander table"), "{e}");
        assert_eq!(sim.check_format("yoshimaru").unwrap().key, "commander");
        // a Commander deck at a Cali table: fine unless a card of it is banned there
        sim.set_format("cali").unwrap();
        for k in MINE {
            assert!(sim.check_format(k).is_ok(), "{k}");
        }
        assert!(sim.check_format("dack-fayden-white-stax").unwrap_err().contains("Tabernacle"));
    }

    /// seeds near u64::MAX do not overflow the balance check (the release profile has overflow checks)
    #[test]
    fn seeds_near_the_top_of_u64_do_not_panic() {
        let top = u64::MAX - 3;
        let c = play_draw_counts(top, 10, 5);
        assert_eq!(c.iter().map(|(p, d)| p + d).sum::<u64>(), 3, "only the seeds that exist are counted");
        assert!(balance_note(top, 10, 5).is_some());
        assert!(balance_note(u64::MAX - 1, 5, 5).unwrap().contains(&u64::MAX.to_string()));
        assert!(balance_note(u64::MAX, u64::MAX, 5).is_none(), "no seeds, nothing unbalanced");
        let _ = is_balanced(top, 1, 3);
    }

    #[test]
    fn even_share_is_one_over_the_seats() {
        assert_eq!(even_share_pct(4), 25);
        assert_eq!(even_share_pct(2), 50);
        assert_eq!(baseline_text(4), "4-player baseline 25%");
        assert_eq!(baseline_text(2), "2-player baseline 50%");
    }
}
