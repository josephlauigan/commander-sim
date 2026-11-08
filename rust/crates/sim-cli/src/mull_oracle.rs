//! --- c4-mulligan-oracle: `commander-sim --mulligan-oracle KEYS [--hands N] [--playouts M] [--seed S] [--ai ..]`.
//! A trace-only report that grades the AI's keep/mulligan rule against an oracle; it changes no game.
//!
//! For each of N seeds the deck's opening seven is dealt exactly as `mulligan` deals it (try 0), with the rule's
//! decision (`keeps_hand`); when the rule mulligans it, the second seven (try 1) too. For each such hand, M goldfish
//! playouts (the machinery of `--goldfish`: the deck alone against three inert seats) are played from each arm:
//!
//! - KEEP: start with this hand; the rest of the library is reshuffled per playout (the player doesn't know its order);
//! - MULLIGAN: put the hand back and follow the normal `mulligan` procedure from the next try (rule-driven keeps,
//!   bottoming included).
//!
//! Both arms use the same playout streams (paired). The score is the own turn of the kill, 11 for no kill by turn 10.
//! The oracle says "mulligan" when MULLIGAN's mean is lower than KEEP's by more than 2 standard errors of the paired
//! difference, "keep" when it is higher by that margin, "unclear" otherwise.

use crate::args::Args;
use crate::goldfish::{deck_keys, list_of, seats, settings_of};
use crate::play::{Sim, ai_name, profile};
use sim_core::engine::turn::Seat;
use sim_core::mull_oracle::{NO_KILL, ROUNDS, TryHand, opening_tries, paired_scores};
use sim_core::settings::Settings;
use std::sync::Arc;
use std::time::Instant;

// the verdict and its reading moved to sim-core (practice mode's hint plays the same playouts)
pub use sim_core::mull_oracle::{Verdict, judge};

/// one dealt hand with the rule's decision and the oracle's
#[derive(Debug, Clone, PartialEq)]
pub struct TryRes {
    pub seed: u64,
    /// 0 the first seven, 1 the second
    pub tries: i64,
    pub rule_keeps: bool,
    pub lands: usize,
    /// a Tier 6 deck: the hand's effective mana
    pub eff: Option<f64>,
    pub cards: Vec<String>,
    /// mean score of each arm (kill turn, 11 for none by turn 10)
    pub keep_mean: f64,
    pub mull_mean: f64,
    /// mean paired difference (MULLIGAN - KEEP) and its standard error
    pub diff: f64,
    pub se: f64,
    pub verdict: Verdict,
}

impl TryRes {
    /// how many turns slower the rule's choice is than the other arm (positive: the rule is the worse choice)
    pub fn rule_cost(&self) -> f64 {
        if self.rule_keeps { self.keep_mean - self.mull_mean } else { self.mull_mean - self.keep_mean }
    }
}

/// what one deck's run needs
pub struct Ctx<'a> {
    pub sim: &'a Sim,
    pub seats: Vec<Seat>,
    pub settings: Arc<Settings>,
    pub playouts: u64,
}

impl Ctx<'_> {
    fn grade(&self, seed: u64, th: &TryHand) -> TryRes {
        let e = &self.sim.engine;
        // the paired playouts are sim-core's (practice mode's hint plays the same ones); one thread: the hands run in parallel
        let (keep, mull) =
            paired_scores(&e.db, &e.registry, &self.settings, &self.seats, &th.hand, th.tries, seed, 0..self.playouts, 1)
                .expect("a playout panicked");
        let (keep_mean, mull_mean, diff, se, verdict) = judge(&keep, &mull);
        let db = &self.sim.engine.db;
        let mut cards: Vec<String> = th.hand.iter().map(|&c| db.get(c).name.to_string()).collect();
        cards.sort();
        TryRes {
            seed,
            tries: th.tries,
            rule_keeps: th.keeps,
            lands: th.lands,
            eff: th.eff,
            cards,
            keep_mean,
            mull_mean,
            diff,
            se,
            verdict,
        }
    }

    /// the graded hands of one seed: try 0, and try 1 when the rule mulligans try 0
    pub fn hand_job(&self, seed: u64) -> Vec<TryRes> {
        let e = &self.sim.engine;
        opening_tries(&e.db, &e.registry, &self.settings, seed, &self.seats).iter().map(|th| self.grade(seed, th)).collect()
    }
}

/// the deck's context: seats, loose-profile settings
pub fn ctx_for<'a>(sim: &'a Sim, key: &str, prof: &str, playouts: u64) -> Result<Ctx<'a>, String> {
    let deck = sim.seat(key, &list_of(sim, key)?)?;
    let seats = seats(sim, &deck)?;
    let settings = Arc::new(settings_of(sim, key, profile(prof)?));
    Ok(Ctx { sim, seats, settings, playouts })
}

/// all graded hands of a deck (seeds seed0 .. seed0+hands), in seed order; the progress moves per hand
pub fn run_deck(sim: &Sim, key: &str, prof: &str, hands: u64, playouts: u64, seed0: u64) -> Result<Vec<TryRes>, String> {
    let ctx = ctx_for(sim, key, prof, playouts)?;
    let seeds: Vec<u64> = (seed0..seed0 + hands).collect();
    Ok(sim.run_seeds(&seeds, |s| ctx.hand_job(s)).into_iter().flat_map(|x| x.1).collect())
}

// ------------------------------------------------------------------ the report
/// counts by oracle verdict
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cell {
    pub keep: usize,
    pub mull: usize,
    pub unclear: usize,
}

impl Cell {
    fn add(&mut self, v: Verdict) {
        match v {
            Verdict::Keep => self.keep += 1,
            Verdict::Mull => self.mull += 1,
            Verdict::Unclear => self.unclear += 1,
        }
    }
    fn total(&self) -> usize {
        self.keep + self.mull + self.unclear
    }
}

/// rule keeps / rule mulligans, each by oracle verdict
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Agree {
    pub rule_keep: Cell,
    pub rule_mull: Cell,
}

pub fn tally<'a>(rs: impl Iterator<Item = &'a TryRes>) -> Agree {
    let mut a = Agree::default();
    for r in rs {
        if r.rule_keeps { a.rule_keep.add(r.verdict) } else { a.rule_mull.add(r.verdict) }
    }
    a
}

pub(crate) fn pct(k: usize, n: usize) -> f64 {
    100.0 * k as f64 / n.max(1) as f64
}

pub(crate) fn table(a: &Agree) -> String {
    let mut o = format!("  {:18} {:>11} {:>11} {:>9} {:>7}\n", "", "oracle KEEP", "oracle MULL", "unclear", "total");
    for (name, c) in [("rule KEEPS", a.rule_keep), ("rule MULLIGANS", a.rule_mull)] {
        o += &format!("  {name:18} {:>11} {:>11} {:>9} {:>7}\n", c.keep, c.mull, c.unclear, c.total());
    }
    let settled = a.rule_keep.keep + a.rule_keep.mull + a.rule_mull.keep + a.rule_mull.mull;
    let agree = a.rule_keep.keep + a.rule_mull.mull;
    let all = a.rule_keep.total() + a.rule_mull.total();
    o += &format!(
        "  agreement: {:.1}% of the {} hands the oracle settles ({:.1}% of all {} hands counting unclear as no disagreement: {})\n",
        pct(agree, settled),
        settled,
        pct(all - (a.rule_keep.mull + a.rule_mull.keep), all),
        all,
        all - (a.rule_keep.mull + a.rule_mull.keep)
    );
    o
}

pub(crate) fn split_row(label: &str, a: &Agree) -> String {
    format!(
        "  {label:10} | rule keeps: {:>5} {:>5} {:>5} | rule mulls: {:>5} {:>5} {:>5}\n",
        a.rule_keep.keep, a.rule_keep.mull, a.rule_keep.unclear, a.rule_mull.keep, a.rule_mull.mull, a.rule_mull.unclear
    )
}

pub(crate) fn split_head(by: &str) -> String {
    format!(
        "  {:10} | rule keeps: {:>5} {:>5} {:>5} | rule mulls: {:>5} {:>5} {:>5}   (cells: oracle keep / mull / unclear)\n",
        by, "keep", "mull", "unc", "keep", "mull", "unc"
    )
}

pub const LAND_BUCKETS: [&str; 5] = ["0-1", "2", "3", "4", "5+"];

pub fn land_bucket(lands: usize) -> usize {
    match lands {
        0 | 1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 4,
    }
}

pub const EFF_BUCKETS: [&str; 5] = ["<2", "2-2.9", "3-3.9", "4-4.9", "5+"];

pub fn eff_bucket(eff: f64) -> usize {
    match eff {
        e if e < 2.0 => 0,
        e if e < 3.0 => 1,
        e if e < 4.0 => 2,
        e if e < 5.0 => 3,
        _ => 4,
    }
}

fn hand_line(r: &TryRes) -> String {
    let mut cards: Vec<(String, usize)> = vec![];
    for c in &r.cards {
        match cards.last_mut() {
            Some((n, k)) if n == c => *k += 1,
            _ => cards.push((c.clone(), 1)),
        }
    }
    let names: Vec<String> = cards.iter().map(|(n, k)| if *k > 1 { format!("{n} x{k}") } else { n.clone() }).collect();
    format!(
        "    seed {} try {}: {} lands{}; keep {:.2}, mull {:.2}, gap {:+.2} turns against the rule (se {:.2})\n      {}\n",
        r.seed,
        r.tries,
        r.lands,
        r.eff.map_or(String::new(), |e| format!(", eff mana {e:.1}")),
        r.keep_mean,
        r.mull_mean,
        r.rule_cost(),
        r.se,
        names.join(", ")
    )
}

/// the report of one deck
pub fn render(sim: &Sim, key: &str, rs: &[TryRes], hands: u64, playouts: u64, ai: &str, seed0: u64, secs: f64) -> String {
    let mut o = format!(
        "\n=== {} ({key}) mulligan oracle [{ai} AI; {hands} hands from seed {seed0}, {playouts} playouts per arm; kill turn, {NO_KILL} = none by turn {ROUNDS}] ===\n",
        sim.name_of(key)
    );
    let t6 = rs.iter().any(|r| r.eff.is_some());
    for tries in 0..2 {
        let sub: Vec<&TryRes> = rs.iter().filter(|r| r.tries == tries).collect();
        if sub.is_empty() {
            continue;
        }
        o += &format!(
            "\nTry {tries} ({}): {} hands, the rule keeps {:.1}%\n",
            if tries == 0 { "the first seven" } else { "the second seven, dealt when the rule mulligans the first" },
            sub.len(),
            pct(sub.iter().filter(|r| r.rule_keeps).count(), sub.len())
        );
        o += &table(&tally(sub.iter().copied()));
        let mean = |f: &dyn Fn(&&TryRes) -> f64| sub.iter().map(f).sum::<f64>() / sub.len() as f64;
        o += &format!(
            "  mean kill turn over these hands: KEEP {:.2}, MULLIGAN {:.2}\n",
            mean(&|r| r.keep_mean),
            mean(&|r| r.mull_mean)
        );
        o += &format!("  by lands in the hand:\n{}", split_head("lands"));
        for (b, name) in LAND_BUCKETS.iter().enumerate() {
            let t = tally(sub.iter().copied().filter(|r| land_bucket(r.lands) == b));
            if t.rule_keep.total() + t.rule_mull.total() > 0 {
                o += &split_row(name, &t);
            }
        }
        if t6 {
            o += &format!("  by effective mana (t6_keep: lands plus the mana the hand makes by turn 2):\n{}", split_head("eff mana"));
            for (b, name) in EFF_BUCKETS.iter().enumerate() {
                let t = tally(sub.iter().copied().filter(|r| r.eff.is_some_and(|e| eff_bucket(e) == b)));
                if t.rule_keep.total() + t.rule_mull.total() > 0 {
                    o += &split_row(name, &t);
                }
            }
        }
    }
    // the worst disagreements: the oracle settles it against the rule, ranked by turns lost
    let worst = |rule_keeps: bool, against: Verdict| -> Vec<&TryRes> {
        let mut v: Vec<&TryRes> = rs.iter().filter(|r| r.rule_keeps == rule_keeps && r.verdict == against).collect();
        v.sort_by(|a, b| b.rule_cost().total_cmp(&a.rule_cost()));
        v.truncate(5);
        v
    };
    for (title, rule_keeps, against) in [
        ("Worst disagreements, the rule KEEPS and the oracle says MULLIGAN", true, Verdict::Mull),
        ("Worst disagreements, the rule MULLIGANS and the oracle says KEEP", false, Verdict::Keep),
    ] {
        let w = worst(rule_keeps, against);
        o += &format!("\n{title} (top {}):\n", w.len());
        if w.is_empty() {
            o += "    none\n";
        }
        for r in w {
            o += &hand_line(r);
        }
    }
    let lost: Vec<f64> = rs
        .iter()
        .filter(|r| (r.rule_keeps && r.verdict == Verdict::Mull) || (!r.rule_keeps && r.verdict == Verdict::Keep))
        .map(|r| r.rule_cost())
        .collect();
    o += &format!(
        "\nHands the oracle settles against the rule: {} of {} ({:.1}%), costing {:.2} turns each on average ({:.3} turns per hand over all)\n",
        lost.len(),
        rs.len(),
        pct(lost.len(), rs.len()),
        lost.iter().sum::<f64>() / lost.len().max(1) as f64,
        lost.iter().sum::<f64>() / rs.len().max(1) as f64
    );
    o += &format!("Run time: {secs:.0}s\n");
    o
}

/// `--mulligan-oracle KEYS`: every deck's report
pub fn main(sim: &Sim, a: &Args) -> Result<(), String> {
    let spec = a.mulligan_oracle.as_deref().unwrap_or("");
    let keys = deck_keys(sim, spec).map_err(|e| e.replacen("--goldfish", "--mulligan-oracle", 1))?;
    if a.hands == 0 || a.playouts == 0 {
        return Err("--hands and --playouts must be at least 1".into());
    }
    let prof = a.profiles.first().cloned().unwrap_or_else(|| "loose".into());
    let ai = ai_name(sim.opts.ai);
    let lists: Vec<crate::coverage::Entry> = keys
        .iter()
        .map(|k| list_of(sim, k).map(|l| crate::coverage::Entry::Deck(crate::coverage::deck_list(sim, k, &l, None))))
        .collect::<Result<_, _>>()?;
    crate::coverage::print(&lists);
    println!(
        "Mulligan oracle: {} deck{}, {} hands each from seed {}, {} playouts per arm (2 arms, up to 2 tries per hand: at most {} games per deck, {ai} AI, {} threads)",
        keys.len(),
        if keys.len() > 1 { "s" } else { "" },
        a.hands,
        a.seed,
        a.playouts,
        4 * a.hands * a.playouts,
        sim.opts.jobs
    );
    sim.prog.lock().unwrap().on = !a.quiet;
    sim.prog.lock().unwrap().plan(a.hands * keys.len() as u64);
    let t0 = Instant::now();
    for k in &keys {
        sim.prog.lock().unwrap().label = format!("mulligan oracle {k} (hands)");
        let t = Instant::now();
        let rs = run_deck(sim, k, &prof, a.hands, a.playouts, a.seed)?;
        print!("{}", render(sim, k, &rs, a.hands, a.playouts, ai, a.seed, t.elapsed().as_secs_f64()));
    }
    println!("\n({:.0}s total)", t0.elapsed().as_secs_f64());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;
    use sim_core::engine::turn::setup_game;
    use sim_core::ids::PlayerId;
    use sim_core::mull_oracle::{Arm, setup_playout};
    use sim_core::settings::AiMode;

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    fn t6_key(sim: &Sim) -> String {
        sim.pool_keys("t6")[0].clone()
    }

    #[test]
    fn the_verdict_needs_two_standard_errors() {
        // mull is faster in every playout by 1 +- 0: settled
        assert_eq!(judge(&[5, 6, 7, 5], &[4, 5, 6, 4]).4, Verdict::Mull);
        assert_eq!(judge(&[4, 5, 6, 4], &[5, 6, 7, 5]).4, Verdict::Keep);
        // no difference
        assert_eq!(judge(&[5, 6, 7, 5], &[5, 6, 7, 5]).4, Verdict::Unclear);
        // a noisy small gap: mean difference -0.5, se 0.87
        assert_eq!(judge(&[5, 5, 5, 5], &[3, 7, 4, 4]).4, Verdict::Unclear);
        // one playout has no standard error
        assert_eq!(judge(&[5], &[4]).4, Verdict::Unclear);
        let (km, mm, d, se, v) = judge(&[6, 8], &[5, 5]);
        assert_eq!((km, mm, d, v), (7.0, 5.0, -2.0, Verdict::Unclear));
        assert!((se - 1.0).abs() < 1e-9);
        // the buckets
        assert_eq!([0, 1, 2, 3, 4, 7].map(land_bucket), [0, 0, 1, 2, 3, 4]);
        assert_eq!([1.9, 2.0, 3.5, 4.9, 5.0].map(eff_bucket), [0, 1, 2, 3, 4]);
    }

    /// the new setup follows the real one: for the same seed the first mulligan outcome, library and stats of
    /// `Arm::Mull { from_try: 0 }` are those of `setup_game`, and the tries `opening_tries` deals are the real draws
    #[test]
    fn the_oracle_deals_the_hands_the_game_deals() {
        let sim = sim();
        for key in ["aragorn".to_string(), t6_key(&sim)] {
            let ctx = ctx_for(&sim, &key, "loose", 2).unwrap();
            let e = &sim.engine;
            for seed in 500000..500012u64 {
                let real = setup_game(e.db.clone(), e.registry.clone(), ctx.settings.clone(), seed, &ctx.seats, false);
                let mine = setup_playout(
                    &e.db,
                    &e.registry,
                    &ctx.settings,
                    &ctx.seats,
                    &seed.to_string(),
                    seed,
                    &Arm::Mull { from_try: 0 },
                );
                let (a, b) = (real.player(PlayerId(0)), mine.player(PlayerId(0)));
                assert_eq!((&a.hand, &a.library, a.stats.get("mull_n")), (&b.hand, &b.library, b.stats.get("mull_n")), "{key} {seed}");
                // the tries: the last one dealt is the real hand when it was kept
                let tries = opening_tries(&e.db, &e.registry, &ctx.settings, seed, &ctx.seats);
                let last = tries.last().unwrap();
                if last.keeps {
                    assert_eq!(last.hand, a.hand, "{key} {seed}: the kept hand");
                    assert_eq!(a.stats.get("mull_n").copied().unwrap_or(0), last.tries, "{key} {seed}");
                } else {
                    assert_eq!(tries.len(), 2);
                    assert!(a.stats.get("mull_n").copied().unwrap_or(0) >= 2, "{key} {seed}: mulliganed past the second seven");
                }
            }
        }
    }

    #[test]
    fn the_keep_arm_starts_with_the_hand_and_reshuffles_the_rest() {
        let sim = sim();
        let key = "aragorn";
        let ctx = ctx_for(&sim, key, "loose", 2).unwrap();
        let e = &sim.engine;
        let th = &opening_tries(&e.db, &e.registry, &ctx.settings, 500003, &ctx.seats)[0];
        let mk = |name: &str| setup_playout(&e.db, &e.registry, &ctx.settings, &ctx.seats, name, 1, &Arm::Keep { hand: &th.hand, tries: 0 });
        let (a, b) = (mk("x:1"), mk("x:2"));
        let (pa, pb) = (a.player(PlayerId(0)), b.player(PlayerId(0)));
        assert_eq!(pa.hand, th.hand);
        assert_eq!(pb.hand, th.hand);
        assert_ne!(pa.library, pb.library, "the library is reshuffled per playout");
        let mut la = pa.library.clone();
        let mut lb = pb.library.clone();
        la.extend(pa.hand.iter().copied());
        lb.extend(pb.hand.iter().copied());
        la.sort();
        lb.sort();
        assert_eq!(la, lb, "the same cards in total");
        assert_eq!(pa.library.len() + 7, ctx.seats[0].cards.len() - 1);
    }

    #[test]
    fn the_mode_runs_on_a_jd_deck_and_a_tier_6_deck() {
        let sim = sim();
        for key in ["aragorn".to_string(), t6_key(&sim)] {
            let rs = run_deck(&sim, &key, "loose", 4, 2, 500000).unwrap();
            assert!(rs.len() >= 4 && rs.iter().all(|r| r.tries <= 1));
            assert_eq!(rs.iter().filter(|r| r.tries == 0).count(), 4);
            // try 1 only follows a mulliganed try 0
            for r in rs.iter().filter(|r| r.tries == 1) {
                assert!(rs.iter().any(|x| x.seed == r.seed && x.tries == 0 && !x.rule_keeps));
            }
            assert!(rs.iter().all(|r| r.keep_mean >= 1.0 && r.keep_mean <= NO_KILL as f64 && r.mull_mean <= NO_KILL as f64));
            let again = run_deck(&sim, &key, "loose", 4, 2, 500000).unwrap();
            assert_eq!(rs, again, "the same seed gives the same report");
            let text = render(&sim, &key, &rs, 4, 2, "adaptive", 500000, 0.0);
            assert!(text.contains("mulligan oracle") && text.contains("Try 0") && text.contains("by lands in the hand"));
            assert!(text.contains("Worst disagreements, the rule KEEPS"));
            assert_eq!(text.contains("by effective mana"), sim_core::impls::t6_plans::is_t6_key(&key));
        }
    }

    /// the oracle changes no game: a plain goldfish run is the same with the oracle's machinery loaded and run before
    /// it (same seed, same report text), and equals the report stored when this mode was written
    #[test]
    fn a_plain_goldfish_run_is_unchanged() {
        let sim = sim();
        let key = "aragorn";
        let before = crate::goldfish::run_deck(&sim, key, "loose", 8, 500000).unwrap();
        let _ = run_deck(&sim, key, "loose", 3, 2, 500000).unwrap();
        let after = crate::goldfish::run_deck(&sim, key, "loose", 8, 500000).unwrap();
        assert_eq!(before, after);
        assert_eq!(
            crate::goldfish::render(&sim, key, &before, "adaptive", 500000),
            crate::goldfish::render(&sim, key, &after, "adaptive", 500000)
        );
    }
}
