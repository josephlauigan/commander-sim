//! Pool mode (poolmode.py): one deck against a fixed field of outside decks (decklists/pool/).
//!
//! Each game seats the deck under test plus three opponents drawn without replacement from one tier's five decks,
//! in a random seat order (pools.draw_seats). Opponent draws, seat order, every seat's opening shuffle and
//! mulligans come from random streams that depend only on the seed and the deck keys, so two versions of a deck
//! (a --swap) face identical opponents and identical opponent draws: paired, common-random-number runs.

use crate::args::Args;
use crate::decks::{MINE, TIERS, tier_label};
use crate::duel;
use crate::play::{GameOut, Sim, ai_name, profile};
use crate::report::{self, Analysis, Axes, CardInfo};
use crate::stats::{Counter, paired_delta, wilson};
use sim_core::engine::turn::Seat;
use sim_core::format::DEFAULT_FORMAT;
use sim_core::settings::{AiMode, Settings};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

fn is_mine(key: &str) -> bool {
    MINE.contains(&key)
}

/// Why this deck of yours is not in the `--all-decks` matrix: it plays a 2-seat format, and the matrix (its even share,
/// its average over the tiers) is built for 4-seat Commander tables. None for a deck that plays a 4-seat format.
pub fn not_in_matrix(sim: &Sim, deck: &str) -> Option<String> {
    let own = sim.engine.format_of(deck);
    crate::duel::is_duel(own).then(|| {
        format!("{deck} plays {} (1v1), which the 4-seat matrix does not run: use --deck {deck} --pool all", own.name)
    })
}

/// the opponent tiers a deck of yours is simulated against (data/my_decks.toml via data/decks.json); every tier if
/// the deck record carries none. A deck is refused at a table its format does not admit: a Cali deck at a Commander
/// table (`--format commander`).
pub fn deck_tiers(sim: &Sim, deck: &str) -> Result<Vec<String>, String> {
    sim.check_format(deck)?;
    Ok(match sim.engine.decks.get(deck) {
        Some(d) if !d.tiers.is_empty() => d.tiers.clone(),
        _ => TIERS.iter().map(|t| t.to_string()).collect(),
    })
}

/// "t1-t3" for consecutive tiers, "t4, t5" style otherwise
pub fn tier_range(tiers: &[String]) -> String {
    let nums: Vec<u32> = tiers.iter().filter_map(|t| t.strip_prefix('t')?.parse().ok()).collect();
    if nums.len() == tiers.len() && nums.len() > 2 && nums.windows(2).all(|w| w[1] == w[0] + 1) {
        format!("{}-{}", tiers[0], tiers[tiers.len() - 1])
    } else {
        tiers.join(", ")
    }
}

/// The tiers `deck` (with its own tiers `own`) runs against for `--pool pool`: `all` is its own tiers, one tier must
/// be among them. `--any-tier` lifts both limits (`all` then means every tier, t0-t5).
pub fn select_tiers(deck: &str, own: &[String], pool: &str, any_tier: bool) -> Result<Vec<String>, String> {
    if pool == "all" {
        return Ok(if any_tier { TIERS.iter().map(|t| t.to_string()).collect() } else { own.to_vec() });
    }
    if any_tier || own.iter().any(|t| t == pool) {
        return Ok(vec![pool.to_string()]);
    }
    Err(format!(
        "{deck} is only simulated against {} (data/my_decks.toml), not {pool}. Pass --any-tier to run it there \
         anyway.",
        own.join(", ")
    ))
}

/// what a run covers: the tiers in play (for the plan line and the matrix columns) and each deck with its tiers
pub struct TierPlan {
    pub tiers: Vec<String>,
    pub per_deck: Vec<(String, Vec<String>)>,
    /// `--all-decks` leaves out a deck the matrix does not run (a 2-seat format): (deck, why), one line each in the run
    pub skipped: Vec<(String, String)>,
}

/// poolmode's tier rule: every deck of yours only against its own tiers. Calibration has no deck and is unchanged.
pub fn tier_plan(sim: &Sim, a: &Args) -> Result<TierPlan, String> {
    let (Some(pool), None) = (a.pool.as_deref(), a.calibrate.as_ref()) else {
        return Ok(TierPlan { tiers: vec![], per_deck: vec![], skipped: vec![] });
    };
    let mut per_deck: Vec<(String, Vec<String>)> = vec![];
    let mut skipped: Vec<(String, String)> = vec![];
    if a.all_decks {
        for d in MINE {
            if let Some(why) = not_in_matrix(sim, d) {
                skipped.push((d.to_string(), why));
                continue;
            }
            let own = deck_tiers(sim, d)?;
            // one tier given: only the decks that play it (all of them with --any-tier)
            let t = if pool == "all" || a.any_tier || own.iter().any(|t| t == pool) {
                select_tiers(d, &own, pool, a.any_tier)?
            } else {
                continue;
            };
            per_deck.push((d.to_string(), t));
        }
    } else if let Some(d) = a.deck.as_deref() {
        per_deck.push((d.to_string(), select_tiers(d, &deck_tiers(sim, d)?, pool, a.any_tier)?));
    }
    let tiers = TIERS
        .iter()
        .filter(|t| per_deck.iter().any(|(_, dt)| dt.iter().any(|x| x == *t)))
        .map(|t| t.to_string())
        .collect();
    Ok(TierPlan { tiers, per_deck, skipped })
}

/// poolmode.run's tallies for one deck against one tier under one profile
pub struct RunTally {
    pub profile: String,
    pub n: u64,
    pub win: u64,
    pub win_turns: u64,
    pub loss_turns: u64,
    pub plan: u64,
    pub tempts: u64,
    pub seated: Counter<String>,
    pub opp_won: Counter<String>,
    pub killed_me: Counter<String>,
    pub by_seed: HashMap<u64, bool>,
    /// games the deck started in seat 0 (on the play) and won, and the rest (on the draw): read by a 2-seat run's report
    pub play: (u64, u64),
    pub draw: (u64, u64),
    /// the seeds on which the deck was on the play
    pub play_seeds: HashSet<u64>,
    pub axes: Axes,
    pub wintype: Counter<String>,
    /// Tier 6 report counters, filled only when `me` is a Tier 6 deck (the per-deck block of `--deck X --pool t6`)
    pub t6: crate::t6report::T6Tally,
}

impl RunTally {
    pub fn new(profile: &str) -> RunTally {
        RunTally {
            profile: profile.to_string(),
            n: 0,
            win: 0,
            win_turns: 0,
            loss_turns: 0,
            plan: 0,
            tempts: 0,
            seated: Counter::new(),
            opp_won: Counter::new(),
            killed_me: Counter::new(),
            by_seed: HashMap::new(),
            play: (0, 0),
            draw: (0, 0),
            play_seeds: HashSet::new(),
            axes: Axes::new(),
            wintype: Counter::new(),
            t6: Default::default(),
        }
    }

    /// poolmode._chunk's tally of one game
    pub fn add(&mut self, seed: u64, g: &GameOut, me: &str) {
        let Some(pi) = g.player(me) else { return };
        let p = &g.players[pi];
        let won = g.winner == Some(pi);
        if sim_core::impls::t6_plans::is_t6_key(me) {
            self.t6.add(g);
        }
        self.n += 1;
        self.win += won as u64;
        self.by_seed.insert(seed, won);
        let side = if pi == 0 {
            self.play_seeds.insert(seed);
            &mut self.play
        } else {
            &mut self.draw
        };
        side.0 += 1;
        side.1 += won as u64;
        if won {
            self.win_turns += p.turns as u64;
            self.wintype.add(g.wintype_or_damage().to_string(), 1);
        } else {
            self.loss_turns += p.turns as u64;
        }
        for (qi, q) in g.players.iter().enumerate() {
            if qi == pi {
                continue;
            }
            self.seated.add(q.key.to_string(), 1);
            self.opp_won.add(q.key.to_string(), (g.winner == Some(qi)) as i64);
        }
        if !p.alive
            && let Some(k) = p.killer
            && k != pi
        {
            self.killed_me.add(g.players[k].key.to_string(), 1);
        }
        if is_mine(me) {
            for (k, x) in report::axis_values(g, pi, me) {
                self.axes.entry(k).or_default().add(x);
            }
            self.tempts += p.stat("ring_tempts") as u64;
            if report::plan_turn(p, me).is_some() {
                self.plan += 1;
            }
        }
    }
}

/// the deck's results by seed on the play (seat 0, the starting player) or on the draw
fn side_of(r: &RunTally, play: bool) -> HashMap<u64, bool> {
    r.by_seed.iter().filter(|(s, _)| r.play_seeds.contains(s) == play).map(|(s, w)| (*s, *w)).collect()
}

pub struct Ctx<'a> {
    pub sim: &'a Sim,
    pub verbose: bool,
}

impl Ctx<'_> {
    pub(crate) fn ai(&self) -> &'static str {
        ai_name(self.sim.opts.ai)
    }

    pub(crate) fn label(&self, s: String) {
        self.sim.prog.lock().unwrap().label = s;
    }

    fn seeds(n: u64, seed0: u64) -> Vec<u64> {
        (seed0..seed0 + n).collect()
    }

    /// poolmode.run: `me` (with my_seat's cards, or its list) against opp_keys, n games from seed0
    pub fn run(
        &self,
        me: &str,
        my_seat: Option<&Seat>,
        opp_keys: &[String],
        prof: &str,
        n: u64,
        seed0: u64,
    ) -> RunTally {
        let sim = self.sim;
        let fmt = sim.format_for(me);
        let st = sim.settings_vs_for(profile(prof).unwrap(), opp_keys, fmt);
        let games = sim.run_seeds(&Self::seeds(n, seed0), |s| {
            let seats = sim.table_for(s, Some(me), my_seat, opp_keys, fmt);
            GameOut::from_game(&sim.play(s, &seats, &st, false), false)
        });
        let mut r = RunTally::new(prof);
        for (s, g) in &games {
            r.add(*s, g, me);
        }
        r
    }

    /// the paired run with the analysis collected alongside: the same games as `run` (the tally is identical), plus
    /// the card and game-shape numbers `--swap` explains with. Reads finished games only, so it never touches the RNG.
    pub fn run_explained(
        &self,
        me: &str,
        my_seat: &Seat,
        opp_keys: &[String],
        prof: &str,
        n: u64,
        seed0: u64,
    ) -> (RunTally, Analysis) {
        let sim = self.sim;
        let fmt = sim.format_for(me);
        let st = sim.settings_vs_for(profile(prof).unwrap(), opp_keys, fmt);
        let games = sim.run_seeds(&Self::seeds(n, seed0), |s| {
            let seats = sim.table_for(s, Some(me), Some(my_seat), opp_keys, fmt);
            let mut g = GameOut::from_game(&sim.play(s, &seats, &st, false), true);
            for p in g.players.iter_mut().filter(|p| p.key != me) {
                p.seen.clear();
                p.cast.clear();
                p.lost.clear();
                p.cast_turns.clear();
            }
            g
        });
        let mut r = RunTally::new(prof);
        let mut a = Analysis::default();
        for (s, g) in &games {
            r.add(*s, g, me);
            a.add(g, me);
        }
        (r, a)
    }

    /// poolmode.analyze
    pub fn analyze(&self, me: &str, my_seat: &Seat, opp_keys: &[String], prof: &str, n: u64, seed0: u64) -> Analysis {
        let st = self.sim.settings_vs_for(profile(prof).unwrap(), opp_keys, self.sim.format_for(me));
        self.analyze_with(me, my_seat, opp_keys, st, n, seed0)
    }

    /// analyze with these settings (the preflight dry run plays the adaptive AI whatever --ai says); the caller sets
    /// the settings' rules to the deck's format
    pub fn analyze_with(
        &self,
        me: &str,
        my_seat: &Seat,
        opp_keys: &[String],
        st: Settings,
        n: u64,
        seed0: u64,
    ) -> Analysis {
        let sim = self.sim;
        let fmt = sim.format_for(me);
        let games = sim.run_seeds(&Self::seeds(n, seed0), |s| {
            let seats = sim.table_for(s, Some(me), Some(my_seat), opp_keys, fmt);
            let g = GameOut::from_game(&sim.play(s, &seats, &st, false), true);
            // only my seat's card lists are needed: drop the others' early
            let mut g = g;
            for p in g.players.iter_mut().filter(|p| p.key != me) {
                p.seen.clear();
                p.cast.clear();
                p.lost.clear();
                p.cast_turns.clear();
            }
            g
        });
        let mut r = Analysis::default();
        for (_, g) in &games {
            r.add(g, me);
        }
        r
    }

    /// the games --calibrate and --realism score: me (or nobody) + k decks drawn from opp_keys, seeds seed0..seed0+n
    pub fn tier_games(
        &self,
        me: Option<&str>,
        opp_keys: &[String],
        k: usize,
        prof: &str,
        n: u64,
        seed0: u64,
    ) -> Vec<(u64, GameOut)> {
        let st = self.sim.settings_vs(profile(prof).unwrap(), opp_keys);
        self.tier_games_with(&st, me, opp_keys, k, n, seed0)
    }

    /// `tier_games` with these settings (`--fit` plays every candidate's knobs on the same seeds). Commander tables
    /// only: `--calibrate`, `--realism` and `--fit` are refused for a 2-seat format (`duel::check_modes`), so these
    /// seats keep `seats_for` with the caller's k (4 pool decks, or me and 3).
    pub fn tier_games_with(
        &self,
        st: &Settings,
        me: Option<&str>,
        opp_keys: &[String],
        k: usize,
        n: u64,
        seed0: u64,
    ) -> Vec<(u64, GameOut)> {
        let sim = self.sim;
        sim.run_seeds(&Self::seeds(n, seed0), |s| {
            let seats = sim.seats_for(s, me, None, opp_keys, k);
            GameOut::from_game(&sim.play(s, &seats, st, false), false)
        })
    }

    /// poolmode.calibrate: me (or nobody) + k decks from opp_keys; every seat's wins and seatings
    pub fn calibrate(&self, me: Option<&str>, opp_keys: &[String], k: usize, prof: &str, n: u64, seed0: u64) -> Calib {
        let games = self.tier_games(me, opp_keys, k, prof, n, seed0);
        let mut c = Calib {
            seated: Counter::new(),
            won: Counter::new(),
            rounds: 0,
            timeouts: 0,
            timeout_won: Counter::new(),
            n,
            t6: Default::default(),
        };
        for (_, g) in &games {
            c.t6.add(g);
            c.rounds += g.round as u64;
            c.timeouts += (g.wintype == Some("timeout")) as u64;
            if let (Some(w), Some("timeout")) = (g.winner, g.wintype) {
                c.timeout_won.add(g.players[w].key.to_string(), 1);
            }
            for (qi, q) in g.players.iter().enumerate() {
                c.seated.add(q.key.to_string(), 1);
                c.won.add(q.key.to_string(), (g.winner == Some(qi)) as i64);
            }
        }
        c
    }

    // -------------------------------------------------------------- reports
    pub fn print_run(&self, me: &str, tier: &str, r: &RunTally) {
        let sim = self.sim;
        let (n, w) = (r.n, r.win);
        let (lo, hi) = wilson(w, n);
        let nf = n as f64;
        let fmt = sim.format_for(me);
        let duel = duel::is_duel(fmt);
        let label = duel::ai_label(fmt).map(|l| format!("; {l}")).unwrap_or_default();
        println!(
            "\n=== {} vs {} [{}, {} AI, {n} games{label}] ===",
            sim.name_of(me),
            tier_label(tier),
            r.profile,
            self.ai()
        );
        println!(
            "Win rate {:5.1}%   95% CI {:.1}-{:.1}%   (even share {}%: {:+.1} pts)",
            (100 * w) as f64 / nf,
            100.0 * lo,
            100.0 * hi,
            duel::even_share_pct(fmt.seats),
            100.0 * (w as f64 / nf - 1.0 / fmt.seats as f64)
        );
        if duel {
            let side = |(g, won): (u64, u64)| {
                if g == 0 { "n/a".to_string() } else { format!("{:.1}% ({won}/{g})", (100 * won) as f64 / g as f64) }
            };
            println!("On the play {}, on the draw {}", side(r.play), side(r.draw));
        }
        let wt = if w > 0 { format!("{:.1}", r.win_turns as f64 / w as f64) } else { "n/a".into() };
        let lt = if n > w { format!("{:.1}", r.loss_turns as f64 / (n - w) as f64) } else { "n/a".into() };
        println!("Average turn (your own turn count) of wins {wt}, of losses {lt}");
        if is_mine(me) {
            println!("Plan online ({}): {:.1}% of games", report::plan_text(me), (100 * r.plan) as f64 / nf);
        }
        if !r.wintype.is_empty() {
            let parts: Vec<String> = r
                .wintype
                .most_common()
                .iter()
                .map(|(k, v)| format!("{k} {:.0}%", (100 * v) as f64 / w as f64))
                .collect();
            println!("Wins by: {}", parts.join(", "));
        }
        println!(
            "  {:42} {:>7} {:>16} {:>15}",
            "Opponent",
            if duel { "games" } else { "seated" },
            "won when seated",
            "eliminated you"
        );
        for k in r.seated.keys_by_count() {
            let s = r.seated.get(&k);
            // a 2-seat table seats one opponent per game: the games against it, not a share of three seats
            let seated = if duel { format!("{s:7}") } else { format!("{:6.1}%", (100 * s) as f64 / nf) };
            println!(
                "  {:42.42} {seated} {:15.1}% {:14.1}%",
                format!("{} ({k})", sim.name_of(&k)),
                (100 * r.opp_won.get(&k)) as f64 / s as f64,
                (100 * r.killed_me.get(&k)) as f64 / nf
            );
        }
        println!("  ('eliminated you' is the share of all games in which that opponent dealt the finishing blow)");
        // the Tier 6 report reads 4-seat tables (seat shares, who holds the stops)
        if sim_core::impls::t6_plans::is_t6_key(me) && tier == "t6" && !duel {
            print!("{}", crate::t6report::render(&r.t6, &[me.to_string()], &|k| sim.name_of(k)));
        }
    }

    /// poolmode.print_ab: baseline vs variant per profile, the verdict and the per-axis table
    pub fn print_ab(&self, me: &str, tier: &str, pairs: &[(RunTally, RunTally)]) {
        let sim = self.sim;
        let fmt = sim.format_for(me);
        let label = duel::ai_label(fmt).map(|l| format!("; {l}")).unwrap_or_default();
        println!(
            "\n=== {} vs {}: baseline vs variant (paired, same opponents and draws{label}) ===",
            sim.name_of(me),
            tier_label(tier)
        );
        let (mut deltas, mut ses) = (vec![], vec![]);
        for (b, v) in pairs {
            let (d, se, n) = paired_delta(&b.by_seed, &v.by_seed);
            deltas.push(d);
            ses.push(se);
            println!(
                "[{:12}] win {:5.1}% -> {:5.1}%  (delta {:+.1} pts, noise ±{:.1}, paired)   n={n}",
                b.profile,
                (100 * b.win) as f64 / b.n as f64,
                (100 * v.win) as f64 / v.n as f64,
                100.0 * d,
                200.0 * se
            );
            let opp: Vec<String> = b
                .seated
                .keys_by_count()
                .iter()
                .map(|k| format!("{} {:.1}%", sim.name_of(k), (100 * v.killed_me.get(k)) as f64 / v.n as f64))
                .collect();
            println!("{:15}eliminated by (variant): {}", "", opp.join("  "));
            if duel::is_duel(fmt) {
                // the same seed gives the same opponent in the same order for both lists
                for (side, play) in [("on the play", true), ("on the draw", false)] {
                    let (bs, vs) = (side_of(b, play), side_of(v, play));
                    let (d, se, n) = paired_delta(&bs, &vs);
                    let pct = |m: &HashMap<u64, bool>| {
                        (100 * m.values().filter(|x| **x).count()) as f64 / m.len().max(1) as f64
                    };
                    println!(
                        "{:15}{side:11} win {:5.1}% -> {:5.1}%  (delta {:+.1} pts, noise ±{:.1}, paired)   n={n}",
                        "",
                        pct(&bs),
                        pct(&vs),
                        100.0 * d,
                        200.0 * se
                    );
                }
            }
        }
        let profs: Vec<String> = pairs.iter().map(|(b, _)| b.profile.clone()).collect();
        println!("VERDICT: {}", report::verdict(&deltas, &ses, &profs));
        if self.verbose {
            let ax: Vec<(String, Axes, Axes)> =
                pairs.iter().map(|(b, v)| (b.profile.clone(), b.axes.clone(), v.axes.clone())).collect();
            report::axis_report(me, &ax, self.sim.opts.ai == AiMode::Lookahead);
        }
    }

    fn print_opponent_names(&self, keys: &[String]) {
        let parts: Vec<String> = keys.iter().map(|k| format!("{k} = {}", self.sim.name_of(k))).collect();
        println!("  Opponent keys: {}", parts.join(", "));
    }

    /// a 2-seat run whose seeds do not seat the deck on the play and on the draw equally against every opponent says so
    fn note_balance(&self, me: &str, opp_keys: &[String], n: u64, seed0: u64) {
        if duel::is_duel(self.sim.format_for(me))
            && let Some(note) = duel::balance_note(seed0, n, opp_keys.len())
        {
            println!("{note}");
        }
    }

    /// poolmode.matrix: every deck against its own tiers, one table per profile (a dash where the deck doesn't play
    /// that tier), with each deck's average over the tiers it played
    pub fn matrix(&self, decks: &[(String, Vec<String>)], tiers: &[&str], profiles: &[String], n: u64, seed0: u64) {
        let mut cells: HashMap<(String, String, String), RunTally> = HashMap::new();
        for prof in profiles {
            for (d, dt) in decks {
                for tier in dt {
                    self.label(format!("{d} {tier} / {prof}"));
                    let r = self.run(d, None, &self.sim.opponents(tier, self.sim.format_for(d)), prof, n, seed0);
                    if self.sim.prog.lock().unwrap().on {
                        let (lo, hi) = wilson(r.win, r.n);
                        eprintln!(
                            "  cell {d} vs {tier} [{prof}]: {:5.1}% ({:.1}-{:.1})",
                            (100 * r.win) as f64 / r.n as f64,
                            100.0 * lo,
                            100.0 * hi
                        );
                    }
                    cells.insert((prof.clone(), d.to_string(), tier.to_string()), r);
                }
            }
        }
        // a 4-seat table: `--all-decks` leaves out a deck of a 2-seat format, and a 2-seat deck's tiers are reported apart
        let even = duel::even_share_pct(self.sim.engine.formats[DEFAULT_FORMAT].seats);
        for prof in profiles {
            println!(
                "\n=== Win rate by deck and tier [{prof} profile, {} AI, n={n} per cell; even share {even}%; each deck only \
                 against its own tiers] ===",
                self.ai()
            );
            let heads: Vec<String> = tiers.iter().map(|t| format!("{t:>18}")).collect();
            println!("  {:10} {} {:>26}", "deck", heads.join(" "), "average over its tiers");
            for (d, dt) in decks {
                let row: Vec<String> = tiers
                    .iter()
                    .map(|t| match cells.get(&(prof.clone(), d.to_string(), t.to_string())) {
                        Some(r) => {
                            let (lo, hi) = wilson(r.win, r.n);
                            let x = format!(
                                "{:5.1}% ({:4.1}-{:4.1})",
                                (100 * r.win) as f64 / r.n as f64,
                                100.0 * lo,
                                100.0 * hi
                            );
                            format!("{x:>18}")
                        }
                        None => format!("{:>18}", "-"),
                    })
                    .collect();
                let (w, g) = dt.iter().fold((0u64, 0u64), |(w, g), t| {
                    let r = &cells[&(prof.clone(), d.to_string(), t.to_string())];
                    (w + r.win, g + r.n)
                });
                let avg = format!("{:5.1}% over {}", (100 * w) as f64 / g.max(1) as f64, tier_range(dt));
                println!("  {d:10} {} {avg:>26}", row.join(" "));
            }
        }
    }

    /// poolmode.run_calibration: within-tier balance and tier ordering
    pub fn run_calibration(&self, which: &str, pool: Option<&str>, profiles: &[String], n: u64, seed0: u64) {
        let keep = |t: &str| pool.is_none_or(|p| p == "all" || p == t);
        let sim = self.sim;
        if which == "within" || which == "all" {
            for prof in profiles {
                println!("\n=== Within-tier balance [{prof}, n={n} games per tier; each deck is seated in ~80%] ===");
                for tier in TIERS.iter().filter(|t| keep(t)) {
                    self.label(format!("within {tier} / {prof}"));
                    let keys = sim.pool_keys(tier);
                    let r = self.calibrate(None, &keys, 4, prof, n, seed0);
                    println!(
                        "  {}: avg game {:.1} rounds, timeouts {:.1}%",
                        tier_label(tier),
                        r.rounds as f64 / r.n.max(1) as f64,
                        (100 * r.timeouts) as f64 / r.n.max(1) as f64
                    );
                    for k in &keys {
                        let (w, s) = (r.won.get(k) as u64, r.seated.get(k) as u64);
                        let (lo, hi) = wilson(w, s);
                        let rate = w as f64 / s as f64;
                        let flag = if rate > 0.35 {
                            "  HIGH"
                        } else if rate < 0.15 {
                            "  LOW"
                        } else {
                            ""
                        };
                        println!(
                            "    {:28} {:5.1}% ({:.1}-{:.1}){flag}  [{:.1} pts from round-cap wins]",
                            sim.name_of(k),
                            (100 * w) as f64 / s as f64,
                            100.0 * lo,
                            100.0 * hi,
                            100.0 * r.timeout_won.get(k) as f64 / s as f64
                        );
                    }
                    if *tier == "t6" {
                        print!("{}", crate::t6report::render(&r.t6, &keys, &|k| sim.name_of(k)));
                    }
                }
            }
        }
        if which == "ordering" || which == "all" {
            for prof in profiles {
                println!(
                    "\n=== Tier ordering: each deck alone against three decks of the tier below [{prof}, n={n}] ==="
                );
                for (ti, tier) in TIERS.iter().enumerate().skip(1).filter(|(_, t)| keep(t)) {
                    self.label(format!("ordering {tier} / {prof}"));
                    let below = TIERS[ti - 1];
                    let mut res = vec![];
                    for d in sim.pool_keys(tier) {
                        let r = self.calibrate(Some(&d), &sim.pool_keys(below), 3, prof, n, seed0);
                        res.push((d.clone(), r.won.get(&d) as u64, r.seated.get(&d) as u64));
                    }
                    let tot_w: u64 = res.iter().map(|x| x.1).sum();
                    let tot_s: u64 = res.iter().map(|x| x.2).sum();
                    println!(
                        "  {} into {}: {:.1}% overall",
                        tier_label(tier),
                        tier_label(below),
                        (100 * tot_w) as f64 / tot_s as f64
                    );
                    for (k, w, s) in &res {
                        let (lo, hi) = wilson(*w, *s);
                        let even = 1.0 / self.sim.engine.formats[DEFAULT_FORMAT].seats as f64;
                        let flag = if hi < even || *w as f64 / *s as f64 <= even {
                            format!(
                                "  (not above {}%)",
                                duel::even_share_pct(self.sim.engine.formats[DEFAULT_FORMAT].seats)
                            )
                        } else {
                            String::new()
                        };
                        println!(
                            "    {:28} {:5.1}% ({:.1}-{:.1}){flag}",
                            sim.name_of(k),
                            (100 * w) as f64 / *s as f64,
                            100.0 * lo,
                            100.0 * hi
                        );
                    }
                }
            }
        }
    }
}

/// poolmode.calibrate's tallies
pub struct Calib {
    pub seated: Counter<String>,
    pub won: Counter<String>,
    pub rounds: u64,
    pub timeouts: u64,
    /// wins by round-cap tiebreak (life + 2 x board power), per deck
    pub timeout_won: Counter<String>,
    /// the Tier 6 counters (win routes, combo tries and stops, mulligans, interaction, seat shares)
    pub t6: crate::t6report::T6Tally,
    /// games planned (errors included, as Python counted len(seeds))
    pub n: u64,
}

/// poolmode.main: called when --pool, --all-decks or --calibrate is given
pub fn main(sim: &Sim, a: &Args, swaps: &[(String, String)], base: &[String], var: &[String]) -> Result<(), String> {
    let ctx = Ctx { sim, verbose: !a.brief };
    sim.prog.lock().unwrap().on = !a.quiet;
    let profiles = &a.profiles;
    for p in profiles {
        profile(p)?;
    }
    let n = a.games;
    let seed0 = a.seed;
    // each of your decks runs only against its own tiers (data/my_decks.toml), unless --any-tier
    let plan = tier_plan(sim, a)?;
    for (d, why) in &plan.skipped {
        println!("--all-decks leaves out {d}: {why}.");
    }
    let tiers: Vec<&str> = plan.tiers.iter().map(|t| t.as_str()).collect();
    if let Some(line) = a.deck.as_deref().filter(|_| !a.all_decks).and_then(|d| duel::describe(sim.format_for(d))) {
        println!("{line}");
    }
    let t0 = Instant::now();
    let k = n * profiles.len() as u64;
    let plural = |x: usize, w: &str| format!("{x} {w}{}", if x > 1 { "s" } else { "" });
    // a 2-seat deck's tiers are reported apart, one report each: never the matrix, whose average pools its tiers
    let duel_deck = !a.all_decks && a.deck.as_deref().is_some_and(|d| duel::is_duel(sim.format_for(d)));
    let matrix_mode = a.all_decks || (tiers.len() > 1 && swaps.is_empty() && !a.analyze && !duel_deck);
    let (total, mut what) = if let Some(c) = &a.calibrate {
        let nt = TIERS.iter().filter(|t| a.pool.as_deref().is_none_or(|p| p == "all" || p == **t)).count() as u64;
        let w = if c == "within" || c == "all" { nt } else { 0 };
        let o = if c == "ordering" || c == "all" { 5 * nt.min(TIERS.len() as u64 - 1) } else { 0 };
        (k * (w + o), format!("calibration: {c}"))
    } else if matrix_mode {
        let cells: usize = plan.per_deck.iter().map(|(_, t)| t.len()).sum();
        (
            k * cells as u64,
            format!(
                "{} x {} (each deck in its own tiers)",
                plural(plan.per_deck.len(), "deck"),
                plural(tiers.len(), "tier")
            ),
        )
    } else if !swaps.is_empty() {
        (k * tiers.len() as u64 * 2, "current list and the swapped one".to_string())
    } else {
        (k * tiers.len() as u64, if a.analyze { "analysis".to_string() } else { plural(tiers.len(), "tier") })
    };
    what += &format!(
        ", {} per list per tier, profile{} {}",
        crate::progress::thousands(n),
        if profiles.len() > 1 { "s" } else { "" },
        profiles.join(" and ")
    );
    {
        let mut pr = sim.prog.lock().unwrap();
        pr.plan(total);
        pr.announce(&what, ctx.ai(), sim.opts.jobs);
    }
    if let Some(c) = &a.calibrate {
        ctx.run_calibration(c, a.pool.as_deref(), profiles, n, seed0);
    } else if matrix_mode {
        ctx.matrix(&plan.per_deck, &tiers, profiles, n, seed0);
    } else if a.analyze {
        let deck = a.deck.as_deref().unwrap();
        let cards = if swaps.is_empty() { base } else { var };
        let seat = sim.seat(deck, cards)?;
        if !swaps.is_empty() {
            println!("Analyzing the variant list: {}", swap_text(swaps));
        }
        let infos = card_infos(sim, cards);
        let fmt = sim.format_for(deck);
        for tier in &tiers {
            let keys = sim.opponents(tier, fmt);
            ctx.note_balance(deck, &keys, n, seed0);
            for prof in profiles {
                ctx.label(format!("analyze {tier} / {prof}"));
                let r = ctx.analyze(deck, &seat, &keys, prof, n, seed0);
                println!("\n######## vs {} ########", tier_label(tier));
                report::print_analysis(deck, &infos, &r, prof, ctx.ai(), &|k| sim.name_of(k), fmt);
                ctx.print_opponent_names(&keys);
            }
        }
    } else if !swaps.is_empty() {
        let deck = a.deck.as_deref().unwrap();
        let bseat = sim.seat(deck, base)?;
        let vseat = sim.seat(deck, var)?;
        let explain = !a.no_explain;
        let commanders: Vec<String> = sim
            .engine
            .decks
            .get(deck)
            .map(|d| std::iter::once(d.commander.clone()).chain(d.commander2.clone()).collect())
            .unwrap_or_default();
        let fmt = sim.format_for(deck);
        for tier in &tiers {
            let mut pairs = vec![];
            let mut analyses = vec![];
            let keys = sim.opponents(tier, fmt);
            ctx.note_balance(deck, &keys, n, seed0);
            for prof in profiles {
                ctx.label(format!("{tier} current / {prof}"));
                let (b, v);
                if explain {
                    let (bt, ba) = ctx.run_explained(deck, &bseat, &keys, prof, n, seed0);
                    ctx.label(format!("{tier} swaps / {prof}"));
                    let (vt, va) = ctx.run_explained(deck, &vseat, &keys, prof, n, seed0);
                    analyses.push((prof.clone(), ba, va));
                    (b, v) = (bt, vt);
                } else {
                    b = ctx.run(deck, Some(&bseat), &keys, prof, n, seed0);
                    ctx.label(format!("{tier} swaps / {prof}"));
                    v = ctx.run(deck, Some(&vseat), &keys, prof, n, seed0);
                }
                pairs.push((b, v));
            }
            println!("Swaps: {}", swap_text(swaps));
            ctx.print_ab(deck, tier, &pairs);
            for (prof, ba, va) in &analyses {
                crate::explain::print_explain(deck, &commanders, swaps, ba, va, prof, &|k| sim.name_of(k));
            }
        }
    } else {
        let deck = a.deck.as_deref().unwrap();
        let seat = sim.seat(deck, base)?;
        let fmt = sim.format_for(deck);
        for tier in &tiers {
            let keys = sim.opponents(tier, fmt);
            ctx.note_balance(deck, &keys, n, seed0);
            for prof in profiles {
                ctx.label(format!("{tier} / {prof}"));
                let r = ctx.run(deck, Some(&seat), &keys, prof, n, seed0);
                ctx.print_run(deck, tier, &r);
            }
        }
    }
    println!("\n({:.0}s total)", t0.elapsed().as_secs_f64());
    Ok(())
}

pub fn swap_text(swaps: &[(String, String)]) -> String {
    swaps.iter().map(|(o, i)| format!("{o} -> {i}")).collect::<Vec<_>>().join("; ")
}

/// the card report's view of a deck's cards: unique names in list order, land or not, reactive or not
fn card_infos(sim: &Sim, cards: &[String]) -> Vec<CardInfo> {
    let mut seen = std::collections::HashSet::new();
    cards
        .iter()
        .filter(|c| seen.insert(c.as_str()))
        .map(|c| {
            let raw = sim.raw_card(c);
            CardInfo {
                name: c.clone(),
                land: raw.is_some_and(|r| r.derived.land),
                held: raw.is_some_and(crate::cardinfo::held),
            }
        })
        .collect()
}

/// poolmode.trace: a play-by-play of one pool game (seed = --seed + GAME); without --deck, a game of four decks of
/// the --pool tier seated as `--calibrate within` seats them
pub fn trace(sim: &Sim, a: &Args, cards: &[String]) -> Result<(), String> {
    let Some(deck) = a.deck.as_deref() else { return trace_pool(sim, a) };
    let prof = a.profiles[0].clone();
    // no tier (or all): the deck's first tier; a tier outside the deck's own needs --any-tier
    let own = deck_tiers(sim, deck)?;
    let tier = match a.pool.as_deref() {
        Some("all") | None => own[0].clone(),
        Some(t) => select_tiers(deck, &own, t, a.any_tier)?.remove(0),
    };
    let tier = tier.as_str();
    let fmt = sim.format_for(deck);
    let st = sim.settings_in_for(profile(&prof)?, Some(tier), fmt); // the table is the tier's: its default [table] knobs
    let game = a.trace.unwrap();
    let seed = a.seed + game;
    let seat = sim.seat(deck, cards)?;
    let seats = sim.table_for(seed, Some(deck), Some(&seat), &sim.opponents(tier, fmt), fmt);
    let g = crate::play::catch_game(|| GameOut::from_game(&sim.play(seed, &seats, &st, true), false))?;
    println!(
        "Play-by-play of pool game {game} (seed {seed}): {} vs {}, {prof} profile, {} AI",
        sim.name_of(deck),
        tier_label(tier),
        ai_name(sim.opts.ai)
    );
    if let Some(line) = duel::describe(fmt) {
        println!("{line}");
    }
    println!("{}", g.log.clone().unwrap_or_default().join("\n"));
    let winner = g.winner.map_or("none".to_string(), |w| sim.name_of(g.players[w].key));
    println!("Winner: {winner} ({})", g.wintype.unwrap_or("None"));
    Ok(())
}

/// `--pool tN --trace GAME` without --deck: one within-tier game (the seats `--calibrate within` draws for that seed)
fn trace_pool(sim: &Sim, a: &Args) -> Result<(), String> {
    let tier = match a.pool.as_deref() {
        Some("all") | None => return Err("--trace needs --deck, or --pool with one tier".into()),
        Some(t) => t,
    };
    // the deck-less trace draws four pool decks at a Commander table
    if let Some(f) = sim.requested_format().filter(|f| duel::is_duel(f)) {
        return Err(duel::refuse("--trace without --deck", None, f));
    }
    let prof = a.profiles[0].clone();
    let st = sim.settings_in(profile(&prof)?, Some(tier));
    let game = a.trace.unwrap();
    let seed = a.seed + game;
    // TRACE_ORDER=<deck key> (scratch): the `--calibrate ordering` game of that deck against the tier below
    let seats = match (std::env::var("TRACE_ORDER").ok(), TIERS.iter().position(|t| *t == tier)) {
        (Some(k), Some(ti)) if ti > 0 => sim.seats_for(seed, Some(&k), None, &sim.pool_keys(TIERS[ti - 1]), 3),
        _ => sim.seats_for(seed, None, None, &sim.pool_keys(tier), 4),
    };
    let g = crate::play::catch_game(|| GameOut::from_game(&sim.play(seed, &seats, &st, true), false))?;
    println!(
        "Play-by-play of within-tier game {game} (seed {seed}): {}, {prof} profile, {} AI",
        tier_label(tier),
        ai_name(sim.opts.ai)
    );
    println!("{}", g.log.clone().unwrap_or_default().join("\n"));
    let winner = g.winner.map_or("none".to_string(), |w| sim.name_of(g.players[w].key));
    println!("Winner: {winner} ({})", g.wintype.unwrap_or("None"));
    Ok(())
}
#[cfg(test)]
mod tier_tests {
    use super::*;

    fn v(x: &[&str]) -> Vec<String> {
        x.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn all_means_the_decks_own_tiers() {
        let own = v(&["t1", "t2", "t3"]);
        assert_eq!(select_tiers("cloud", &own, "all", false).unwrap(), own);
        assert_eq!(select_tiers("cloud", &own, "t2", false).unwrap(), v(&["t2"]));
    }

    #[test]
    fn a_tier_outside_is_refused_unless_any_tier() {
        let own = v(&["t4", "t5"]);
        let e = select_tiers("yshtola", &own, "t1", false).unwrap_err();
        assert!(e.contains("yshtola") && e.contains("t4, t5") && e.contains("--any-tier"), "{e}");
        assert_eq!(select_tiers("yshtola", &own, "t1", true).unwrap(), v(&["t1"]));
        // --any-tier with all: every tier
        assert_eq!(select_tiers("yshtola", &own, "all", true).unwrap().len(), 7);
    }

    #[test]
    fn tier_ranges_read_naturally() {
        assert_eq!(tier_range(&v(&["t1", "t2", "t3"])), "t1-t3");
        assert_eq!(tier_range(&v(&["t4", "t5"])), "t4, t5");
        assert_eq!(tier_range(&v(&["t1"])), "t1");
    }

    /// the matched tiers JD approved on 2026-10-15 (audit/pool/2026-10-14-full), as data/decks.json carries them
    #[test]
    fn the_repository_decks_have_their_tiers() {
        let root = crate::decks::root().unwrap();
        let f = sim_core::export::load_decks(&crate::decks::data_dir(&root).join("decks.json")).unwrap();
        let want: [(&str, &[&str], Option<&str>); 5] = [
            ("yshtola", &["t5"], None), // JD 10/18 bands: 1 t5, 2 t3-t4, 3 t3, 4 t1-t3
            ("sheoldred", &["t3", "t4"], None),
            ("kyodai", &["t5", "t6"], Some("cali")), // JD 11/23: Cali Commander, matched tiers t5 and t6
            ("urdragon", &["t5"], None),
            ("yoshimaru", &["t4", "t5"], None),
        ];
        assert_eq!(f.mine.len(), want.len());
        for (k, t, fmt) in want {
            let d = f.mine.iter().find(|d| d.key == k).unwrap_or_else(|| panic!("{k}"));
            assert_eq!(d.tiers, v(t), "{k}");
            assert_eq!(d.format.as_deref(), fmt, "{k}");
        }
        for k in MINE {
            assert!(f.mine.iter().any(|d| d.key == k), "{k}");
        }
        assert!(f.pool.iter().all(|d| d.tiers.is_empty()));
    }

    fn sim() -> Sim {
        Sim::load(crate::play::RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    fn args(s: &str) -> Args {
        crate::args::parse(&s.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>()).unwrap()
    }

    /// a Cali deck plays its own format at every tier it is given (its own tiers, or any tier with `--any-tier`), and is
    /// refused only at a table of another format that does not admit it (`--format commander`), `--any-tier` or not
    #[test]
    fn a_cali_deck_is_played_at_its_own_format_and_refused_at_a_commander_table() {
        let mut sim = sim();
        assert_eq!(
            tier_plan(&sim, &args("--deck kyodai --pool all")).unwrap().per_deck,
            [("kyodai".to_string(), v(&["t5", "t6"]))]
        );
        for pool in ["t5", "t6"] {
            let plan = tier_plan(&sim, &args(&format!("--deck kyodai --pool {pool}"))).unwrap();
            assert_eq!(plan.per_deck, [("kyodai".to_string(), v(&[pool]))]);
        }
        assert!(tier_plan(&sim, &args("--deck kyodai --pool t1")).is_err(), "a tier outside its own needs --any-tier");
        assert!(tier_plan(&sim, &args("--deck kyodai --pool t1 --any-tier")).is_ok());
        sim.set_format("commander").unwrap();
        for pool in ["t5", "t6", "all", "t1"] {
            for any in ["", " --any-tier"] {
                let e = tier_plan(&sim, &args(&format!("--deck kyodai --pool {pool}{any}"))).err().unwrap();
                assert!(
                    e.contains("Cali Commander") && e.contains("kyodai") && e.contains("(--format)"),
                    "{pool}{any}: {e}"
                );
            }
        }
        let e = deck_tiers(&sim, "kyodai").unwrap_err();
        assert!(e.contains("kyodai cannot be played at a Commander table") && e.contains("Cali Commander"), "{e}");
        assert_eq!(deck_tiers(&sim, "yoshimaru").unwrap(), v(&["t4", "t5"]));
        assert!(select_tiers("kyodai", &v(&["t5", "t6"]), "t6", false).is_ok(), "the tier rule alone is unchanged");
    }

    /// `--all-decks` leaves the deck out with a note and plays the other four with their tiers unchanged
    #[test]
    fn all_decks_skips_the_deck_and_keeps_the_others_tiers() {
        let sim = sim();
        let want = vec![
            ("yshtola".to_string(), v(&["t5"])),
            ("sheoldred".to_string(), v(&["t3", "t4"])),
            ("urdragon".to_string(), v(&["t5"])),
            ("yoshimaru".to_string(), v(&["t4", "t5"])),
        ];
        let plan = tier_plan(&sim, &args("--all-decks --pool all")).unwrap();
        assert_eq!(plan.per_deck, want);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].0, "kyodai");
        assert!(plan.skipped[0].1.contains("Cali Commander"), "{:?}", plan.skipped);
        assert_eq!(plan.tiers, v(&["t3", "t4", "t5"]));
        // one tier: the decks that play it, kyodai still out (t6 has no deck of yours left)
        let plan = tier_plan(&sim, &args("--all-decks --pool t5")).unwrap();
        assert_eq!(
            plan.per_deck.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>(),
            ["yshtola", "urdragon", "yoshimaru"]
        );
        assert!(tier_plan(&sim, &args("--all-decks --pool t6")).unwrap().per_deck.is_empty());
        // --any-tier lifts the tier limit, not the format one
        let plan = tier_plan(&sim, &args("--all-decks --pool all --any-tier")).unwrap();
        assert_eq!(
            plan.per_deck.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>(),
            ["yshtola", "sheoldred", "urdragon", "yoshimaru"]
        );
        assert!(plan.per_deck.iter().all(|(_, t)| t.len() == 7));
        assert_eq!(plan.skipped.len(), 1);
    }

    /// the 2-seat decks are the ones the matrix leaves out, and the Commander table refuses exactly the decks whose record
    /// has a format other than commander: both follow data/decks.json
    #[test]
    fn the_matrix_and_the_commander_table_follow_the_decks_format() {
        let mut sim = sim();
        for key in MINE {
            let own = sim.engine.decks[key].format.as_deref().unwrap_or("commander");
            assert!(deck_tiers(&sim, key).is_ok(), "{key}: its own format admits it");
            assert_eq!(not_in_matrix(&sim, key).is_some(), own == "cali", "{key} ({own})");
        }
        sim.set_format("commander").unwrap();
        assert_eq!(MINE.iter().filter(|k| deck_tiers(&sim, k).is_err()).collect::<Vec<_>>(), [&"kyodai"]);
        // a pool deck is a Commander deck
        assert!(sim.pool.iter().all(|d| deck_tiers(&sim, &d.key).is_ok()));
    }

    /// which decks a Cali Commander table admits: its own, and Commander decks with no card on its list (among the
    /// pool, only the Tier 6 stax deck with The Tabernacle at Pendrell Vale is kept out)
    #[test]
    fn a_cali_table_admits_every_deck_of_yours_and_all_but_one_pool_deck() {
        let sim = sim();
        for key in MINE {
            assert!(sim.engine.admits(key, "cali").is_ok(), "{key}");
        }
        let kept_out: Vec<&str> =
            sim.pool.iter().filter(|d| sim.engine.admits(&d.key, "cali").is_err()).map(|d| d.key.as_str()).collect();
        assert_eq!(kept_out, ["dack-fayden-white-stax"]);
        assert!(sim.engine.admits("kyodai", "commander").is_err());
        assert!(sim.engine.admits("yoshimaru", "nowhere").is_err());
    }
}
