//! --- c5-oracle-vs-tier: `commander-sim --mulligan-oracle KEYS --vs-tier [--pool tN|all] [--hands N] [--playouts M]
//! [--recheck K] [--out FILE]`. The real-game arm of the mulligan oracle (`mull_oracle.rs` judges by goldfish kill
//! turn). Trace only: it changes no game.
//!
//! For each of N seeds and each of the deck's own tiers (`data/my_decks.toml`; `--pool all` is all of them, a tier
//! outside them is refused, `--any-tier` is not supported), the opening seven is dealt exactly as the goldfish oracle
//! deals it (`opening_tries`: try 0, and try 1 when the rule mulligans try 0). For each such hand M real pod games are
//! played from each arm, the deck against three opponents drawn from the tier and seated as pool mode seats them
//! (`Sim::seats_for`, a fresh table per playout):
//!
//! - KEEP: start with this hand, the rest of the library reshuffled;
//! - MULLIGAN: put the hand back and follow the normal rule from the next draw.
//!
//! The opponents mulligan and play normally. Both arms of one playout share the table, the opponents' libraries and
//! hands and the play stream (paired). The score is the win (the deck is `GameOut.winner`, as pool mode counts wins):
//! 1 or 0. The verdict is "keep" or "mulligan" when the paired difference is more than 2 standard errors, otherwise
//! "unclear" (the goldfish oracle's `judge`, fed the loss of each playout).

use crate::args::Args;
use crate::decks::MINE;
use crate::goldfish::list_of;
use crate::mull_oracle::{
    EFF_BUCKETS, LAND_BUCKETS, TryRes, Verdict, eff_bucket, judge, land_bucket, pct, split_head, split_row, table,
    tally,
};
use crate::play::{Sim, ai_name, profile};
use crate::poolmode::{deck_tiers, select_tiers, tier_range};
use sim_core::engine::turn::{Seat, run_rounds};
use sim_core::mull_oracle::{Arm, TryHand, opening_tries};
use sim_core::mull_oracle_tier::{HandFeat, POD_ROUNDS, hand_features, setup_pod_playout, won};
use sim_core::rng::Rng;
use sim_core::settings::{AiMode, Settings};
use std::sync::Arc;
use std::time::Instant;

/// the pod games of one playout are numbered from this seed (see `pod_seed`): at most this many playouts per try
pub const MAX_PLAYOUTS: u64 = 999;

/// one dealt hand against one tier
#[derive(Debug, Clone, PartialEq)]
pub struct TierRes {
    pub tier: String,
    pub seed: u64,
    /// 0 the first seven, 1 the second
    pub tries: i64,
    pub rule_keeps: bool,
    pub feat: HandFeat,
    /// the hand's cards, sorted
    pub cards: Vec<String>,
    pub playouts: u64,
    pub keep_wins: u64,
    pub mull_wins: u64,
    /// win rates of the two arms
    pub keep_wr: f64,
    pub mull_wr: f64,
    /// MULLIGAN - KEEP win rate, and its paired standard error
    pub diff: f64,
    pub se: f64,
    pub verdict: Verdict,
}

impl TierRes {
    /// win points (0-100) the rule's choice loses against the other arm (positive: the rule is the worse choice)
    pub fn rule_cost(&self) -> f64 {
        100.0 * if self.rule_keeps { self.mull_wr - self.keep_wr } else { self.keep_wr - self.mull_wr }
    }

    /// the goldfish oracle's record of it (its means are win rates here), for its agreement tables
    fn as_try(&self) -> TryRes {
        TryRes {
            seed: self.seed,
            tries: self.tries,
            rule_keeps: self.rule_keeps,
            lands: self.feat.lands,
            eff: Some(self.feat.eff),
            cards: self.cards.clone(),
            keep_mean: self.keep_wr,
            mull_mean: self.mull_wr,
            diff: self.diff,
            se: self.se,
            verdict: self.verdict,
        }
    }
}

/// the pod seed of playout `j` of try `tries` of hand `seed`: it names the table (`draw_seats`) and every stream of the
/// game (`setup_pod_playout`), so a playout is one fixed table whatever arm is played on it
pub fn pod_seed(seed: u64, tries: i64, j: u64) -> u64 {
    seed.wrapping_mul(1_000_000).wrapping_add(tries as u64 * 1000 + j)
}

/// what one deck against one tier needs
pub struct TierCtx<'a> {
    sim: &'a Sim,
    key: String,
    seat: Seat,
    pub tier: String,
    /// the tier's five decks, the pool the opponents are drawn from
    opp: Vec<String>,
    settings: Arc<Settings>,
    playouts: u64,
}

impl<'a> TierCtx<'a> {
    /// `lookahead`: the look-ahead AI whatever `--ai` says (the recheck)
    pub fn new(
        sim: &'a Sim,
        key: &str,
        tier: &str,
        prof: &str,
        playouts: u64,
        lookahead: bool,
    ) -> Result<Self, String> {
        let seat = sim.seat(key, &list_of(sim, key)?)?;
        let opp = sim.pool_keys(tier);
        if opp.len() < 3 {
            return Err(format!("tier {tier} has {} decks, a table needs three", opp.len()));
        }
        // the settings a pool run of this tier plays with (`poolmode::Ctx::run`)
        let mut st = sim.settings_vs(profile(prof)?, &opp);
        if lookahead {
            st.ai = AiMode::Lookahead;
        }
        Ok(TierCtx { sim, key: key.to_string(), seat, tier: tier.to_string(), opp, settings: Arc::new(st), playouts })
    }

    /// the table of pod seed `pod`: pool mode's seats, and the deck's seat index
    fn pod(&self, pod: u64) -> (Vec<Seat>, usize) {
        let seats = self.sim.seats_for(pod, Some(&self.key), Some(&self.seat), &self.opp, 3);
        let me = seats.iter().position(|s| s.key == self.key.as_str()).expect("the deck is seated");
        (seats, me)
    }

    /// the hands of one seed: try 0, and try 1 when the rule mulligans try 0 (the deck's hand does not depend on the
    /// table, but the rule is asked with this tier's settings and the seed's own table)
    pub fn tries(&self, seed: u64) -> Vec<TryHand> {
        let (mut seats, me) = self.pod(seed);
        seats.swap(0, me); // `opening_tries` deals seat 0
        let e = &self.sim.engine;
        opening_tries(&e.db, &e.registry, &self.settings, seed, &seats)
    }

    fn play(&self, seats: &[Seat], me: usize, pod: u64, arm: &Arm) -> bool {
        let e = &self.sim.engine;
        let mut g = setup_pod_playout(&e.db, &e.registry, &self.settings, seats, me, &pod.to_string(), pod, arm);
        run_rounds(&mut g, POD_ROUNDS);
        sim_core::ai::learned::game_end(&g, pod);
        won(&g, me)
    }

    /// the M paired playouts of one dealt hand and the verdict
    pub fn grade(&self, seed: u64, th: &TryHand) -> TierRes {
        let (mut keep, mut mull) = (vec![], vec![]);
        for j in 0..self.playouts {
            let pod = pod_seed(seed, th.tries, j);
            let (seats, me) = self.pod(pod);
            let k = self.play(&seats, me, pod, &Arm::Keep { hand: &th.hand, tries: th.tries });
            let m = self.play(&seats, me, pod, &Arm::Mull { from_try: th.tries + 1 });
            // the oracle's `judge` reads lower scores as better: a loss scores 1, a win 0
            keep.push(!k as u32);
            mull.push(!m as u32);
        }
        let (keep_loss, mull_loss, dm, se, verdict) = judge(&keep, &mull);
        let e = &self.sim.engine;
        let mut cards: Vec<String> = th.hand.iter().map(|&c| e.db.get(c).name.to_string()).collect();
        cards.sort();
        let wins = |v: &[u32]| v.iter().filter(|&&x| x == 0).count() as u64;
        TierRes {
            tier: self.tier.clone(),
            seed,
            tries: th.tries,
            rule_keeps: th.keeps,
            feat: hand_features(&e.db, &e.registry, &self.settings, &self.seat, &th.hand),
            cards,
            playouts: self.playouts,
            keep_wins: wins(&keep),
            mull_wins: wins(&mull),
            keep_wr: 1.0 - keep_loss,
            mull_wr: 1.0 - mull_loss,
            diff: -dm,
            se,
            verdict,
        }
    }

    /// the graded hands of one seed
    pub fn hand_job(&self, seed: u64) -> Vec<TierRes> {
        self.tries(seed).iter().map(|th| self.grade(seed, th)).collect()
    }
}

/// all graded hands of a deck against `tiers`: seeds seed0 .. seed0+hands, in (seed, tier) order; the progress moves
/// per (seed, tier)
pub fn run_deck(
    sim: &Sim,
    key: &str,
    tiers: &[String],
    prof: &str,
    hands: u64,
    playouts: u64,
    seed0: u64,
) -> Result<Vec<TierRes>, String> {
    let ctxs: Vec<TierCtx> =
        tiers.iter().map(|t| TierCtx::new(sim, key, t, prof, playouts, false)).collect::<Result<_, _>>()?;
    let nt = ctxs.len() as u64;
    let jobs: Vec<u64> = (0..hands * nt).collect();
    let out = sim.run_seeds(&jobs, |i| ctxs[(i % nt) as usize].hand_job(seed0 + i / nt));
    Ok(out.into_iter().flat_map(|x| x.1).collect())
}

// ------------------------------------------------------------------ the CSV
pub const CSV_COLUMNS: [&str; 20] = [
    "deck",
    "tier",
    "seed",
    "try",
    "rule_call",
    "lands",
    "eff_mana",
    "fast_mana",
    "tutors",
    "payoffs",
    "cards",
    "playouts",
    "keep_wins",
    "mull_wins",
    "keep_wr",
    "mull_wr",
    "diff",
    "se",
    "verdict",
    "rule_cost_pp",
];

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() }
}

fn verdict_name(v: Verdict) -> &'static str {
    match v {
        Verdict::Keep => "keep",
        Verdict::Mull => "mulligan",
        Verdict::Unclear => "unclear",
    }
}

pub fn csv_header() -> String {
    CSV_COLUMNS.join(",") + "\n"
}

/// one row per hand x tier. `cards` joins the sorted names with " | ". Win rates are fractions; `diff` is
/// MULLIGAN - KEEP; `rule_cost_pp` is the win points the rule's call loses (positive: the other arm is better)
pub fn csv_rows(key: &str, rs: &[TierRes]) -> String {
    let mut o = String::new();
    for r in rs {
        let f = &r.feat;
        let cells = [
            key.to_string(),
            r.tier.clone(),
            r.seed.to_string(),
            r.tries.to_string(),
            (if r.rule_keeps { "keep" } else { "mulligan" }).to_string(),
            f.lands.to_string(),
            format!("{:.2}", f.eff),
            f.fast_mana.to_string(),
            f.tutors.to_string(),
            f.payoffs.to_string(),
            csv_field(&r.cards.join(" | ")),
            r.playouts.to_string(),
            r.keep_wins.to_string(),
            r.mull_wins.to_string(),
            format!("{:.4}", r.keep_wr),
            format!("{:.4}", r.mull_wr),
            format!("{:.4}", r.diff + 0.0),
            format!("{:.4}", r.se),
            verdict_name(r.verdict).to_string(),
            format!("{:.2}", r.rule_cost() + 0.0),
        ];
        o += &cells.join(",");
        o.push('\n');
    }
    o
}

// ------------------------------------------------------------------ the report
fn mean(rs: &[&TierRes], f: impl Fn(&TierRes) -> f64) -> f64 {
    rs.iter().map(|r| f(r)).sum::<f64>() / rs.len().max(1) as f64
}

pub const FAST_BUCKETS: [&str; 4] = ["0", "1", "2", "3+"];

pub fn fast_bucket(n: usize) -> usize {
    n.min(3)
}

/// KEEP and MULLIGAN win rate by a bucket of the hand
fn wr_split(by: &str, labels: &[&str], rs: &[&TierRes], bucket: &dyn Fn(&TierRes) -> usize) -> String {
    let mut o = format!(
        "  {:10} {:>6} {:>11} {:>9} {:>9} {:>11}\n",
        by, "hands", "rule keeps", "KEEP win", "MULL win", "MULL-KEEP"
    );
    for (b, name) in labels.iter().enumerate() {
        let sub: Vec<&TierRes> = rs.iter().copied().filter(|r| bucket(r) == b).collect();
        if sub.is_empty() {
            continue;
        }
        o += &format!(
            "  {:10} {:>6} {:>10.1}% {:>8.1}% {:>8.1}% {:>+10.1}\n",
            name,
            sub.len(),
            pct(sub.iter().filter(|r| r.rule_keeps).count(), sub.len()),
            100.0 * mean(&sub, |r| r.keep_wr),
            100.0 * mean(&sub, |r| r.mull_wr),
            100.0 * mean(&sub, |r| r.mull_wr - r.keep_wr) + 0.0
        );
    }
    o
}

fn hand_line(r: &TierRes) -> String {
    let mut cards: Vec<(String, usize)> = vec![];
    for c in &r.cards {
        match cards.last_mut() {
            Some((n, k)) if n == c => *k += 1,
            _ => cards.push((c.clone(), 1)),
        }
    }
    let names: Vec<String> = cards.iter().map(|(n, k)| if *k > 1 { format!("{n} x{k}") } else { n.clone() }).collect();
    let f = &r.feat;
    format!(
        "    {} seed {} try {}: {} lands, eff mana {:.1}, {} fast mana, {} tutors, {} payoffs; keep {:.0}%, mull {:.0}%, \
         {:+.1} win points against the rule (se {:.1})\n      {}\n",
        r.tier,
        r.seed,
        r.tries,
        f.lands,
        f.eff,
        f.fast_mana,
        f.tutors,
        f.payoffs,
        100.0 * r.keep_wr,
        100.0 * r.mull_wr,
        r.rule_cost(),
        100.0 * r.se,
        names.join(", ")
    )
}

/// the report of one set of hands: one tier, or all of a deck's tiers together (`full` adds the worst disagreements)
fn render_block(title: &str, rs: &[&TierRes], full: bool) -> String {
    let mut o = format!("\n=== {title} ===\n");
    for tries in 0..2 {
        let sub: Vec<&TierRes> = rs.iter().copied().filter(|r| r.tries == tries).collect();
        if sub.is_empty() {
            continue;
        }
        o += &format!(
            "\nTry {tries} ({}): {} hands, the rule keeps {:.1}%\n",
            if tries == 0 { "the first seven" } else { "the second seven, dealt when the rule mulligans the first" },
            sub.len(),
            pct(sub.iter().filter(|r| r.rule_keeps).count(), sub.len())
        );
        let tr: Vec<TryRes> = sub.iter().map(|r| r.as_try()).collect();
        o += &table(&tally(tr.iter()));
        o += &format!(
            "  mean win rate over these hands: KEEP {:.1}%, MULLIGAN {:.1}%\n",
            100.0 * mean(&sub, |r| r.keep_wr),
            100.0 * mean(&sub, |r| r.mull_wr)
        );
        o += &format!("  by lands in the hand:\n{}", split_head("lands"));
        for (b, name) in LAND_BUCKETS.iter().enumerate() {
            let t = tally(tr.iter().zip(&sub).filter(|(_, r)| land_bucket(r.feat.lands) == b).map(|x| x.0));
            if t.rule_keep.keep
                + t.rule_keep.mull
                + t.rule_keep.unclear
                + t.rule_mull.keep
                + t.rule_mull.mull
                + t.rule_mull.unclear
                > 0
            {
                o += &split_row(name, &t);
            }
        }
        o +=
            &format!("  by effective mana (lands plus the mana the hand makes by turn 2):\n{}", split_head("eff mana"));
        for (b, name) in EFF_BUCKETS.iter().enumerate() {
            let t = tally(tr.iter().zip(&sub).filter(|(_, r)| eff_bucket(r.feat.eff) == b).map(|x| x.0));
            if t.rule_keep.keep
                + t.rule_keep.mull
                + t.rule_keep.unclear
                + t.rule_mull.keep
                + t.rule_mull.mull
                + t.rule_mull.unclear
                > 0
            {
                o += &split_row(name, &t);
            }
        }
    }
    let kept: Vec<&TierRes> = rs.iter().copied().filter(|r| r.rule_keeps).collect();
    o += "\nWin rate by hand, every dealt hand (KEEP = play it, MULL = ship it; the rule's own choice is the arm it takes):\n";
    o += &wr_split("lands", &LAND_BUCKETS, rs, &|r| land_bucket(r.feat.lands));
    o += &wr_split("fast mana", &FAST_BUCKETS, rs, &|r| fast_bucket(r.feat.fast_mana));
    o += &format!("Win rate by hand, the {} hands the rule keeps:\n", kept.len());
    o += &wr_split("lands", &LAND_BUCKETS, &kept, &|r| land_bucket(r.feat.lands));
    o += &wr_split("fast mana", &FAST_BUCKETS, &kept, &|r| fast_bucket(r.feat.fast_mana));
    if full {
        let worst = |rule_keeps: bool, against: Verdict| -> Vec<&TierRes> {
            let mut v: Vec<&TierRes> =
                rs.iter().copied().filter(|r| r.rule_keeps == rule_keeps && r.verdict == against).collect();
            v.sort_by(|a, b| b.rule_cost().total_cmp(&a.rule_cost()));
            v.truncate(5);
            v
        };
        for (t, rule_keeps, against) in [
            ("Worst disagreements, the rule KEEPS and the oracle says MULLIGAN", true, Verdict::Mull),
            ("Worst disagreements, the rule MULLIGANS and the oracle says KEEP", false, Verdict::Keep),
        ] {
            let w = worst(rule_keeps, against);
            o += &format!("\n{t} (top {}):\n", w.len());
            if w.is_empty() {
                o += "    none\n";
            }
            for r in w {
                o += &hand_line(r);
            }
        }
    }
    let lost: Vec<f64> = rs
        .iter()
        .filter(|r| (r.rule_keeps && r.verdict == Verdict::Mull) || (!r.rule_keeps && r.verdict == Verdict::Keep))
        .map(|r| r.rule_cost())
        .collect();
    o += &format!(
        "\nHands the oracle settles against the rule: {} of {} ({:.1}%), costing {:.1} win points each on average ({:.2} win points per hand over all)\n",
        lost.len(),
        rs.len(),
        pct(lost.len(), rs.len()),
        lost.iter().sum::<f64>() / lost.len().max(1) as f64 + 0.0, // + 0.0: no "-0.0"
        lost.iter().sum::<f64>() / rs.len().max(1) as f64 + 0.0
    );
    o
}

/// the report of one deck: a full block per tier, then the combined block over its tiers
pub fn render(
    sim: &Sim,
    key: &str,
    tiers: &[String],
    rs: &[TierRes],
    hands: u64,
    playouts: u64,
    ai: &str,
    seed0: u64,
    secs: f64,
) -> String {
    let name = sim.name_of(key);
    let mut o = String::new();
    for t in tiers {
        let sub: Vec<&TierRes> = rs.iter().filter(|r| &r.tier == t).collect();
        let opp = sim.pool_keys(t).iter().map(|k| sim.name_of(k)).collect::<Vec<_>>().join(", ");
        o += &render_block(
            &format!(
                "{name} ({key}) mulligan oracle vs {t} [{ai} AI; {hands} hands from seed {seed0}, {playouts} paired playouts per arm; win rate against three of {opp}]"
            ),
            &sub,
            true,
        );
    }
    let all: Vec<&TierRes> = rs.iter().collect();
    let label = format!("over {}", tier_range(tiers));
    o += &render_block(
        &format!("{name} ({key}) mulligan oracle {label} ({} hand x tier rows, combined)", all.len()),
        &all,
        false,
    );
    let settled_against = all
        .iter()
        .filter(|r| (r.rule_keeps && r.verdict == Verdict::Mull) || (!r.rule_keeps && r.verdict == Verdict::Keep))
        .count();
    o += &format!(
        "\nSUMMARY {key} {label}: the rule keeps {:.1}% of first sevens; mean win rate KEEP {:.1}%, MULLIGAN {:.1}%; settled against the rule {:.1}% of rows; {:.2} win points lost per hand\nRun time: {secs:.0}s\n",
        pct(all.iter().filter(|r| r.tries == 0 && r.rule_keeps).count(), all.iter().filter(|r| r.tries == 0).count()),
        100.0 * mean(&all, |r| r.keep_wr),
        100.0 * mean(&all, |r| r.mull_wr),
        pct(settled_against, all.len()),
        mean(&all, |r| {
            if (r.rule_keeps && r.verdict == Verdict::Mull) || (!r.rule_keeps && r.verdict == Verdict::Keep) {
                r.rule_cost()
            } else {
                0.0
            }
        })
    );
    o
}

// ------------------------------------------------------------------ the recheck
/// rerun K random graded hands with the look-ahead AI; how often do the verdicts agree
pub fn recheck(
    sim: &Sim,
    key: &str,
    rs: &[TierRes],
    k: usize,
    prof: &str,
    playouts: u64,
    seed0: u64,
) -> Result<String, String> {
    let mut idx: Vec<usize> = (0..rs.len()).collect();
    Rng::named(&format!("recheck:{key}:{seed0}")).shuffle(&mut idx);
    idx.truncate(k);
    let mut tiers: Vec<String> = vec![];
    for &i in &idx {
        if !tiers.contains(&rs[i].tier) {
            tiers.push(rs[i].tier.clone());
        }
    }
    let ctxs: Vec<TierCtx> =
        tiers.iter().map(|t| TierCtx::new(sim, key, t, prof, playouts, true)).collect::<Result<_, _>>()?;
    let jobs: Vec<u64> = (0..idx.len() as u64).collect();
    let out = sim.run_seeds(&jobs, |j| {
        let r = &rs[idx[j as usize]];
        let ctx = &ctxs[tiers.iter().position(|t| t == &r.tier).unwrap()];
        let th = ctx.tries(r.seed).into_iter().find(|th| th.tries == r.tries).expect("the hand is dealt again");
        assert_eq!(th.hand.len(), 7, "a dealt seven");
        ctx.grade(r.seed, &th)
    });
    let mut o = format!(
        "\nRecheck with the look-ahead AI: {} random hands (of {}), {playouts} paired playouts per arm\n",
        out.len(),
        rs.len()
    );
    let (mut same, mut half, mut opposite) = (0, 0, 0);
    for (j, la) in &out {
        let a = &rs[idx[*j as usize]];
        match (a.verdict, la.verdict) {
            (x, y) if x == y => same += 1,
            (Verdict::Unclear, _) | (_, Verdict::Unclear) => half += 1,
            _ => opposite += 1,
        }
        o += &format!(
            "  {} seed {} try {}: adaptive keep {:.0}% mull {:.0}% ({}); look-ahead keep {:.0}% mull {:.0}% ({})\n",
            a.tier,
            a.seed,
            a.tries,
            100.0 * a.keep_wr,
            100.0 * a.mull_wr,
            verdict_name(a.verdict),
            100.0 * la.keep_wr,
            100.0 * la.mull_wr,
            verdict_name(la.verdict)
        );
    }
    let n = out.len();
    o += &format!(
        "Verdicts agree on {same} of {n} ({:.0}%); settled in one AI and unclear in the other on {half}; opposite verdicts on {opposite}\n",
        pct(same, n)
    );
    Ok(o)
}

// ------------------------------------------------------------------ the command
/// the decks and tiers a `--vs-tier` run covers; refuses what the tier rules refuse
pub fn check_args(sim: &Sim, a: &Args) -> Result<Vec<(String, Vec<String>)>, String> {
    let spec = a.mulligan_oracle.as_deref().unwrap_or("");
    if a.any_tier {
        return Err("--mulligan-oracle --vs-tier: --any-tier is not supported (a deck only meets its own tiers)".into());
    }
    if a.profiles != ["loose"] {
        return Err("--mulligan-oracle --vs-tier: the loose profile only".into());
    }
    if a.hands == 0 || a.playouts == 0 {
        return Err("--hands and --playouts must be at least 1".into());
    }
    if a.playouts > MAX_PLAYOUTS {
        return Err(format!("--playouts is at most {MAX_PLAYOUTS} with --vs-tier"));
    }
    let pool = a.pool.as_deref().unwrap_or("all");
    let mut out: Vec<(String, Vec<String>)> = vec![];
    for k in spec.split(',').map(str::trim).filter(|k| !k.is_empty()) {
        if !MINE.contains(&k) {
            return Err(format!("--mulligan-oracle --vs-tier: {k:?} is not one of your decks ({})", MINE.join(", ")));
        }
        if !out.iter().any(|(d, _)| d == k) {
            out.push((k.to_string(), select_tiers(k, &deck_tiers(sim, k), pool, false)?));
        }
    }
    if out.is_empty() {
        return Err("--mulligan-oracle needs at least one deck key".into());
    }
    Ok(out)
}

/// `--mulligan-oracle KEYS --vs-tier`: every deck's report
pub fn main(sim: &Sim, a: &Args) -> Result<(), String> {
    let plan = check_args(sim, a)?;
    let prof = "loose";
    let ai = ai_name(sim.opts.ai);
    let lists: Vec<crate::coverage::Entry> = plan
        .iter()
        .map(|(k, _)| {
            list_of(sim, k).map(|l| crate::coverage::Entry::Deck(crate::coverage::deck_list(sim, k, &l, None)))
        })
        .collect::<Result<_, _>>()?;
    crate::coverage::print(&lists);
    let rows_total: u64 = plan.iter().map(|(_, t)| a.hands * t.len() as u64).sum();
    for (k, tiers) in &plan {
        println!(
            "Mulligan oracle vs tier: {k} against {} ({} hands from seed {}, {} paired playouts per arm: at most {} real games, {ai} AI, {} threads)",
            tiers.join(", "),
            a.hands,
            a.seed,
            a.playouts,
            4 * a.hands * a.playouts * tiers.len() as u64,
            sim.opts.jobs
        );
    }
    if let Some(path) = &a.out {
        std::fs::write(path, csv_header()).map_err(|e| format!("--out {path}: {e}"))?;
    }
    sim.prog.lock().unwrap().on = !a.quiet;
    sim.prog.lock().unwrap().plan(rows_total + a.recheck.min(rows_total));
    let t0 = Instant::now();
    for (k, tiers) in &plan {
        sim.prog.lock().unwrap().label = format!("mulligan oracle vs tier {k} (hand x tier)");
        let t = Instant::now();
        let rs = run_deck(sim, k, tiers, prof, a.hands, a.playouts, a.seed)?;
        if let Some(path) = &a.out {
            use std::io::Write;
            let mut f =
                std::fs::OpenOptions::new().append(true).open(path).map_err(|e| format!("--out {path}: {e}"))?;
            f.write_all(csv_rows(k, &rs).as_bytes()).map_err(|e| format!("--out {path}: {e}"))?;
        }
        print!("{}", render(sim, k, tiers, &rs, a.hands, a.playouts, ai, a.seed, t.elapsed().as_secs_f64()));
        if a.recheck > 0 {
            sim.prog.lock().unwrap().label = format!("recheck {k} (look-ahead)");
            print!("{}", recheck(sim, k, &rs, a.recheck as usize, prof, a.playouts, a.seed)?);
        }
    }
    println!("\n({:.0}s total)", t0.elapsed().as_secs_f64());
    let errs = sim.errors.lock().unwrap();
    if !errs.is_empty() {
        let first: Vec<String> = errs.iter().take(5).map(|e| format!("job {}: {}", e.seed, e.what)).collect();
        return Err(format!("{} jobs failed and are missing from the report ({})", errs.len(), first.join("; ")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;
    use sim_core::engine::turn::setup_game;
    use sim_core::ids::PlayerId;

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    fn args(s: &str) -> Args {
        crate::args::parse(&s.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn only_the_decks_own_tiers_are_played() {
        let sim = sim();
        // cloud is t1-t3, aragorn t4-t5, thrasios t6
        let p = check_args(&sim, &args("--mulligan-oracle cloud,aragorn --vs-tier")).unwrap();
        assert_eq!(p[0].1, ["t1", "t2", "t3"]);
        assert_eq!(p[1].1, ["t4", "t5"]);
        assert_eq!(check_args(&sim, &args("--mulligan-oracle cloud --vs-tier --pool t2")).unwrap()[0].1, ["t2"]);
        // a tier outside the deck's own is refused, as `--deck X --pool tN` is
        let e = check_args(&sim, &args("--mulligan-oracle cloud --vs-tier --pool t5")).unwrap_err();
        assert!(e.contains("only simulated against") && e.contains("t1, t2, t3"), "{e}");
        assert!(
            check_args(&sim, &args("--mulligan-oracle aragorn,cloud --vs-tier --pool t4")).is_err(),
            "cloud has no t4"
        );
        // no --any-tier, no conservative profile, your decks only
        assert!(
            check_args(&sim, &args("--mulligan-oracle cloud --vs-tier --any-tier")).unwrap_err().contains("--any-tier")
        );
        assert!(check_args(&sim, &args("--mulligan-oracle cloud --vs-tier --profiles loose,conservative")).is_err());
        assert!(check_args(&sim, &args("--mulligan-oracle t6 --vs-tier")).is_err());
        assert!(check_args(&sim, &args("--mulligan-oracle kinnan --vs-tier")).is_err());
        // the defaults
        let a = args("--mulligan-oracle cloud --vs-tier");
        assert_eq!((a.hands, a.playouts, a.ai.as_str(), a.recheck), (100, 40, "adaptive", 0));
        assert_eq!(args("--mulligan-oracle cloud --vs-tier --hands 7 --recheck 3 --out x.csv").hands, 7);
        assert_eq!(args("--mulligan-oracle cloud --hands 5").hands, 5);
        assert_eq!(args("--mulligan-oracle cloud").hands, 1000);
    }

    /// both arms of a playout have the same table, the same opponents' libraries and hands; the deck's own seat is
    /// the only difference; the MULLIGAN arm from draw 0 is `setup_game` itself
    #[test]
    fn the_arms_are_paired() {
        let sim = sim();
        let e = &sim.engine;
        for key in ["aragorn", "thrasios"] {
            let tier = &deck_tiers(&sim, key)[0];
            let ctx = TierCtx::new(&sim, key, tier, "loose", 2, false).unwrap();
            for seed in 500000..500006u64 {
                let th = &ctx.tries(seed)[0];
                let pod = pod_seed(seed, th.tries, 0);
                let (seats, me) = ctx.pod(pod);
                let (seats2, me2) = ctx.pod(pod);
                assert_eq!(me, me2);
                assert_eq!(
                    seats.iter().map(|s| s.key).collect::<Vec<_>>(),
                    seats2.iter().map(|s| s.key).collect::<Vec<_>>()
                );
                let name = pod.to_string();
                let mk = |arm: &Arm| setup_pod_playout(&e.db, &e.registry, &ctx.settings, &seats, me, &name, pod, arm);
                let keep = mk(&Arm::Keep { hand: &th.hand, tries: th.tries });
                let mull = mk(&Arm::Mull { from_try: th.tries + 1 });
                for i in (0..4).filter(|&i| i != me) {
                    let (a, b) = (keep.player(PlayerId(i as u8)), mull.player(PlayerId(i as u8)));
                    assert_eq!(
                        (&a.hand, &a.library, a.stats.get("mull_n")),
                        (&b.hand, &b.library, b.stats.get("mull_n")),
                        "{key} {seed} seat {i}"
                    );
                }
                assert_eq!(keep.player(PlayerId(me as u8)).hand, th.hand);
                // the same play stream: a fresh generator of that name draws what the games started with
                assert_eq!(keep.rng, mull.rng);
                assert_eq!(keep.players.len(), 4);
                // three opponents from the tier
                let opp: Vec<_> = seats.iter().filter(|s| s.key != key).map(|s| s.key.to_string()).collect();
                assert_eq!(opp.len(), 3);
                assert!(opp.iter().all(|k| sim.pool_keys(tier).contains(k)));
                // MULLIGAN from draw 0 is the real setup
                let real = setup_game(e.db.clone(), e.registry.clone(), ctx.settings.clone(), pod, &seats, false);
                let m0 = mk(&Arm::Mull { from_try: 0 });
                for i in 0..4u8 {
                    let (a, b) = (real.player(PlayerId(i)), m0.player(PlayerId(i)));
                    assert_eq!(
                        (&a.hand, &a.library, a.stats.get("mull_n")),
                        (&b.hand, &b.library, b.stats.get("mull_n")),
                        "{key} {seed} seat {i}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_small_run_is_deterministic_and_the_csv_has_its_columns() {
        let sim = sim();
        let tiers = vec!["t4".to_string(), "t5".to_string()];
        let rs = run_deck(&sim, "aragorn", &tiers, "loose", 2, 2, 500000).unwrap();
        assert!(rs.len() >= 4 && rs.iter().all(|r| r.tries <= 1 && tiers.contains(&r.tier)));
        assert_eq!(rs.iter().filter(|r| r.tries == 0).count(), 4, "two hands x two tiers");
        for r in &rs {
            assert_eq!(r.playouts, 2);
            assert!(
                (r.keep_wr - r.keep_wins as f64 / 2.0).abs() < 1e-9
                    && (r.mull_wr - r.mull_wins as f64 / 2.0).abs() < 1e-9
            );
            assert_eq!(r.cards.len(), 7);
            assert!((r.diff - (r.mull_wr - r.keep_wr)).abs() < 1e-9);
        }
        let again = run_deck(&sim, "aragorn", &tiers, "loose", 2, 2, 500000).unwrap();
        assert_eq!(rs, again, "the same seed gives the same report");
        let text = render(&sim, "aragorn", &tiers, &rs, 2, 2, "adaptive", 500000, 0.0);
        assert!(
            text.contains("mulligan oracle vs t4") && text.contains("vs t5") && text.contains("over t4, t5"),
            "{text}"
        );
        assert!(text.contains("Worst disagreements, the rule KEEPS") && text.contains("Win rate by hand"));
        assert!(text.contains("SUMMARY aragorn over t4, t5"));
        // the CSV
        let csv = csv_header() + &csv_rows("aragorn", &rs);
        let mut lines = csv.lines();
        let head: Vec<&str> = lines.next().unwrap().split(',').collect();
        assert_eq!(head, CSV_COLUMNS);
        assert_eq!(csv.lines().count(), rs.len() + 1);
        // fields: the cards cell is quoted (names hold commas), so parse by a quote-aware split
        for (line, r) in lines.zip(&rs) {
            let mut cells = vec![];
            let (mut cur, mut q) = (String::new(), false);
            for ch in line.chars() {
                match ch {
                    '"' => q = !q,
                    ',' if !q => cells.push(std::mem::take(&mut cur)),
                    c => cur.push(c),
                }
            }
            cells.push(cur);
            assert_eq!(cells.len(), CSV_COLUMNS.len(), "{line}");
            assert_eq!((cells[0].as_str(), cells[1].as_str()), ("aragorn", r.tier.as_str()));
            assert_eq!((cells[2].parse::<u64>().unwrap(), cells[3].parse::<i64>().unwrap()), (r.seed, r.tries));
            assert_eq!(cells[4], if r.rule_keeps { "keep" } else { "mulligan" });
            assert_eq!(cells[10].split(" | ").count(), 7);
            assert_eq!(cells[18], verdict_name(r.verdict));
        }
        assert_eq!(csv_field("a, b"), "\"a, b\"");
    }

    /// the oracle changes no game: a plain pool run is the same before and after the real-game arm ran, and goldfish too
    #[test]
    fn a_plain_pool_run_is_unchanged() {
        let sim = sim();
        let ctx = crate::poolmode::Ctx { sim: &sim, verbose: false };
        let opp = sim.pool_keys("t4");
        let run = || {
            let t = ctx.run("aragorn", None, &opp, "loose", 6, 500000);
            let mut by: Vec<(u64, bool)> = t.by_seed.iter().map(|(k, v)| (*k, *v)).collect();
            by.sort();
            (t.n, t.win, t.win_turns, t.loss_turns, by)
        };
        let before = run();
        let gf = crate::goldfish::run_deck(&sim, "aragorn", "loose", 4, 500000).unwrap();
        let _ = run_deck(&sim, "aragorn", &["t4".to_string()], "loose", 2, 2, 500000).unwrap();
        assert_eq!(before, run());
        assert_eq!(gf, crate::goldfish::run_deck(&sim, "aragorn", "loose", 4, 500000).unwrap());
    }

    #[test]
    fn hand_features_count_what_the_hand_holds() {
        let sim = sim();
        let ctx = TierCtx::new(&sim, "thrasios", "t6", "loose", 2, false).unwrap();
        for seed in 500000..500020u64 {
            for th in ctx.tries(seed) {
                let f = hand_features(&sim.engine.db, &sim.engine.registry, &ctx.settings, &ctx.seat, &th.hand);
                assert_eq!(f.lands, th.lands);
                assert_eq!(Some(f.eff), th.eff);
                assert!(f.fast_mana <= 7 - f.lands && f.tutors <= 7 - f.lands && f.payoffs <= 7 - f.lands);
            }
        }
    }
}
