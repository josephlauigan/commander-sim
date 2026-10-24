//! `commander-sim --fit --pool tN`: search the AI's knobs (`knobs.rs`) for the values that bring a tier's games
//! closest to its realism targets (`realism.rs`, data/realism/tN.toml). Every candidate plays the same seeds, so two
//! candidates differ by their knobs and not by luck.
//!
//! The search is sep-CMA-ES (Ros and Hansen 2008: CMA-ES with a diagonal covariance, which suits a few dozen
//! parameters and a few hundred evaluations). It works on each knob scaled to [0, 1] between its bounds and starts
//! from the built-in values, which are scored first as the baseline. The loss is the sum over the tier's scored
//! targets of ((sim - target) / half the band)^2: a statistic on the edge of its band costs 1.
//!
//! `--fit-scope shared` (the default) fits one style for every deck of the tier plus the table's knobs; `deck` fits
//! each deck's own style (JD, 2026-10-23: each deck may get its own fitted settings, to close the deck spread).
//! The result is a knobs file (`--knobs` reads it) and a table of the best candidate against the baseline.

use crate::args::Args;
use crate::decks::{self, TIERS, tier_label};
use crate::knobs::{Knobs, StyleKnob};
use crate::play::{Sim, profile};
use crate::poolmode::Ctx;
use crate::pyrand::PyRandom;
use crate::realism::{self, Measured, STATS, Targets};
use std::collections::BTreeMap;

// ------------------------------------------------------------------ sep-CMA-ES
/// sep-CMA-ES state (minimising), on [0, 1]^n
pub struct Cma {
    pub n: usize,
    pub lam: usize,
    mu: usize,
    w: Vec<f64>,
    mueff: f64,
    cs: f64,
    ds: f64,
    cc: f64,
    c1: f64,
    cmu: f64,
    chin: f64,
    pub m: Vec<f64>,
    pub sigma: f64,
    c: Vec<f64>,
    ps: Vec<f64>,
    pc: Vec<f64>,
    generation: u32,
}

impl Cma {
    pub fn new(m: Vec<f64>, sigma: f64) -> Cma {
        let n = m.len();
        let nf = n as f64;
        let lam = 4 + (3.0 * nf.ln()).floor() as usize;
        let mu = lam / 2;
        let raw: Vec<f64> = (1..=mu).map(|i| (mu as f64 + 0.5).ln() - (i as f64).ln()).collect();
        let s: f64 = raw.iter().sum();
        let w: Vec<f64> = raw.iter().map(|x| x / s).collect();
        let mueff = 1.0 / w.iter().map(|x| x * x).sum::<f64>();
        let cs = (mueff + 2.0) / (nf + mueff + 5.0);
        let ds = 1.0 + 2.0 * (((mueff - 1.0) / (nf + 1.0)).sqrt() - 1.0).max(0.0) + cs;
        let cc = (4.0 + mueff / nf) / (nf + 4.0 + 2.0 * mueff / nf);
        // the separable variant learns the diagonal faster: c1 and cmu times (n + 2) / 3
        let sep = (nf + 2.0) / 3.0;
        let c1 = (sep * 2.0 / ((nf + 1.3).powi(2) + mueff)).min(1.0);
        let cmu = (sep * 2.0 * (mueff - 2.0 + 1.0 / mueff) / ((nf + 2.0).powi(2) + mueff)).min(1.0 - c1);
        let chin = nf.sqrt() * (1.0 - 1.0 / (4.0 * nf) + 1.0 / (21.0 * nf * nf));
        Cma { n, lam, mu, w, mueff, cs, ds, cc, c1, cmu, chin, m, sigma, c: vec![1.0; n], ps: vec![0.0; n], pc: vec![0.0; n], generation: 0 }
    }

    /// this generation's candidates (not clamped: `clamp` them to evaluate)
    pub fn ask(&self, rng: &mut Normal) -> Vec<Vec<f64>> {
        (0..self.lam)
            .map(|_| (0..self.n).map(|i| self.m[i] + self.sigma * self.c[i].sqrt() * rng.next()).collect())
            .collect()
    }

    /// the candidates and their losses (lower is better)
    pub fn tell(&mut self, xs: &[Vec<f64>], losses: &[f64]) {
        let mut idx: Vec<usize> = (0..xs.len()).collect();
        idx.sort_by(|&a, &b| losses[a].total_cmp(&losses[b]));
        let old = self.m.clone();
        let mut m = vec![0.0; self.n];
        for (k, &j) in idx.iter().take(self.mu).enumerate() {
            for (i, mi) in m.iter_mut().enumerate() {
                *mi += self.w[k] * xs[j][i];
            }
        }
        let yw: Vec<f64> = (0..self.n).map(|i| (m[i] - old[i]) / self.sigma).collect();
        let a = (self.cs * (2.0 - self.cs) * self.mueff).sqrt();
        for i in 0..self.n {
            self.ps[i] = (1.0 - self.cs) * self.ps[i] + a * yw[i] / self.c[i].sqrt();
        }
        let ps_norm = self.ps.iter().map(|x| x * x).sum::<f64>().sqrt();
        self.generation += 1;
        let hs_den = (1.0 - (1.0 - self.cs).powi(2 * self.generation as i32)).sqrt();
        let hs = ps_norm / hs_den < (1.4 + 2.0 / (self.n as f64 + 1.0)) * self.chin;
        let b = (self.cc * (2.0 - self.cc) * self.mueff).sqrt();
        for i in 0..self.n {
            self.pc[i] = (1.0 - self.cc) * self.pc[i] + if hs { b * yw[i] } else { 0.0 };
        }
        for i in 0..self.n {
            let rank_mu: f64 = idx
                .iter()
                .take(self.mu)
                .enumerate()
                .map(|(k, &j)| self.w[k] * ((xs[j][i] - old[i]) / self.sigma).powi(2))
                .sum();
            let lost = if hs { 0.0 } else { self.cc * (2.0 - self.cc) * self.c[i] };
            self.c[i] = (1.0 - self.c1 - self.cmu) * self.c[i] + self.c1 * (self.pc[i].powi(2) + lost) + self.cmu * rank_mu;
        }
        self.sigma *= ((self.cs / self.ds) * (ps_norm / self.chin - 1.0)).exp();
        self.m = m;
    }
}

/// a point clamped into [0, 1]^n and how far it was outside (for a penalty that keeps the search inside)
pub fn clamp(x: &[f64]) -> (Vec<f64>, f64) {
    let c: Vec<f64> = x.iter().map(|v| v.clamp(0.0, 1.0)).collect();
    let out: f64 = x.iter().zip(&c).map(|(a, b)| (a - b).powi(2)).sum();
    (c, out)
}

/// standard normal draws from the harness's seeded generator (Box-Muller), so a fit repeats exactly
pub struct Normal {
    r: PyRandom,
    spare: Option<f64>,
}

impl Normal {
    pub fn new(seed: &str) -> Normal {
        Normal { r: PyRandom::from_str(seed), spare: None }
    }

    fn unit(&mut self) -> f64 {
        (self.r.genrand_u32() as f64 + 0.5) / 4294967296.0
    }

    pub fn next(&mut self) -> f64 {
        if let Some(s) = self.spare.take() {
            return s;
        }
        let (u, v) = (self.unit(), self.unit());
        let r = (-2.0 * u.ln()).sqrt();
        let t = 2.0 * std::f64::consts::PI * v;
        self.spare = Some(r * t.sin());
        r * t.cos()
    }
}

// ------------------------------------------------------------------ the knobs being fitted
#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Temp,
    Aggression,
    Caution,
    Cthresh,
    InstantExtra,
    MoxdLands,
}

/// one fitted knob: what it sets (one deck's style, or every listed deck's), its bounds and its built-in value
#[derive(Debug, Clone)]
struct Dim {
    name: String,
    field: Field,
    decks: Vec<String>,
    lo: f64,
    hi: f64,
    int: bool,
    base: f64,
}

impl Dim {
    fn to_unit(&self, v: f64) -> f64 {
        ((v - self.lo) / (self.hi - self.lo)).clamp(0.0, 1.0)
    }

    fn from_unit(&self, u: f64) -> f64 {
        let v = self.lo + u.clamp(0.0, 1.0) * (self.hi - self.lo);
        if self.int { v.round() } else { v }
    }
}

fn dims(tier: &str, keys: &[String], scope: &str, st: &sim_core::settings::Settings) -> Vec<Dim> {
    let mut out = vec![];
    let style = |name: String, field, decks: Vec<String>, lo, hi, base| Dim { name, field, decks, lo, hi, int: false, base };
    let mean = |f: &dyn Fn(sim_core::ai::Style) -> f64| {
        keys.iter().map(|k| f(sim_core::ai::base_style(k))).sum::<f64>() / keys.len().max(1) as f64
    };
    if scope == "deck" {
        for k in keys {
            let b = sim_core::ai::base_style(k);
            out.push(style(format!("{k}.temp"), Field::Temp, vec![k.clone()], 0.2, 2.0, b.temp));
            out.push(style(format!("{k}.aggression"), Field::Aggression, vec![k.clone()], 0.0, 1.0, b.aggression));
            out.push(style(format!("{k}.caution"), Field::Caution, vec![k.clone()], 0.0, 1.0, b.caution));
        }
    } else {
        out.push(style("temp".into(), Field::Temp, keys.to_vec(), 0.2, 2.0, mean(&|s| s.temp)));
        out.push(style("aggression".into(), Field::Aggression, keys.to_vec(), 0.0, 1.0, mean(&|s| s.aggression)));
        out.push(style("caution".into(), Field::Caution, keys.to_vec(), 0.0, 1.0, mean(&|s| s.caution)));
    }
    let int = |name: &str, field, lo, hi, base| Dim { name: name.into(), field, decks: vec![], lo, hi, int: true, base };
    out.push(int("cthresh_default", Field::Cthresh, 3.0, 10.0, st.cthresh_default as f64));
    out.push(int("instant_extra", Field::InstantExtra, 0.0, 4.0, st.instant_extra as f64));
    if tier == "t6" {
        out.push(int("t6_moxd_lands", Field::MoxdLands, 1.0, 3.0, st.t6_moxd_lands as f64));
    }
    out
}

/// the knobs a point in [0, 1]^n stands for
fn knobs_of(tier: &str, ds: &[Dim], u: &[f64]) -> Knobs {
    let mut k = Knobs { tier: Some(tier.to_string()), ..Default::default() };
    for (d, &x) in ds.iter().zip(u) {
        let v = d.from_unit(x);
        match d.field {
            Field::Temp | Field::Aggression | Field::Caution => {
                for deck in &d.decks {
                    let s = k.style.entry(deck.clone()).or_insert_with(StyleKnob::default);
                    match d.field {
                        Field::Temp => s.temp = Some(v),
                        Field::Aggression => s.aggression = Some(v),
                        _ => s.caution = Some(v),
                    }
                }
            }
            Field::Cthresh => k.cthresh_default = Some(v as i32),
            Field::InstantExtra => k.instant_extra = Some(v as i32),
            Field::MoxdLands => k.t6_moxd_lands = Some(v as usize),
        }
    }
    k
}

/// ((sim - target) / half the band)^2 summed over the scored statistics
pub fn loss(m: &BTreeMap<&'static str, Measured>, targets: &Targets) -> f64 {
    targets
        .iter()
        .filter_map(|(key, t)| {
            let v = m.get(key.as_str())?.value;
            let half = ((t.hi - t.lo) / 2.0).max(0.5);
            Some(((v - t.value) / half).powi(2))
        })
        .sum()
}

pub fn run(sim: &Sim, a: &Args) -> Result<(), String> {
    let tier = match a.pool.as_deref() {
        Some(t) if TIERS.contains(&t) => t,
        _ => return Err("--fit needs --pool with one tier (t0..t6): it fits that tier's own decks".into()),
    };
    if a.deck.is_some() || a.calibrate.is_some() || a.all_decks || !a.swap.is_empty() || a.realism {
        return Err("--fit plays the tier's own decks: drop --deck, --swap, --all-decks, --calibrate and --realism".into());
    }
    if !["shared", "deck"].contains(&a.fit_scope.as_str()) {
        return Err(format!("--fit-scope must be shared or deck, not {}", a.fit_scope));
    }
    let root = decks::root()?;
    let targets = realism::load_targets(&root, tier)?;
    if targets.is_empty() {
        return Err(format!("{tier} has no targets (data/realism/{tier}.toml): nothing to fit"));
    }
    let prof = a.profiles.first().cloned().unwrap_or_else(|| "loose".into());
    let base_st = sim.settings(profile(&prof)?);
    let ctx = Ctx { sim, verbose: false };
    let keys = sim.pool_keys(tier);
    let ds = dims(tier, &keys, &a.fit_scope, &base_st);
    let (n, seed0) = (a.games, a.seed);
    let mut cma = Cma::new(ds.iter().map(|d| d.to_unit(d.base)).collect(), 0.3);
    let mut rng = Normal::new(&format!("fit:{tier}:{seed0}"));
    let evals = 1 + a.fit_gens as u64 * cma.lam as u64;
    {
        let mut pr = sim.prog.lock().unwrap();
        pr.on = !a.quiet;
        pr.plan(n * evals);
        pr.announce(
            &format!("fit: {tier}, {} knobs ({} scope), {} generations of {}, {n} games each", ds.len(), a.fit_scope, a.fit_gens, cma.lam),
            ctx.ai(),
            sim.opts.jobs,
        );
    }
    let eval = |u: &[f64]| -> (f64, BTreeMap<&'static str, Measured>) {
        let mut st = base_st.clone();
        knobs_of(tier, &ds, u).apply(&mut st);
        let games: Vec<_> = ctx.tier_games_with(&st, None, &keys, 4, n, seed0).into_iter().map(|(_, g)| g).collect();
        let (m, _) = realism::measure(&games, &keys);
        (loss(&m, &targets), m)
    };
    let base_u: Vec<f64> = cma.m.clone();
    let (base_loss, base_m) = eval(&base_u);
    println!("\n=== Fit: {} [{prof}, {} AI, {n} games a candidate, seeds {seed0}-{}] ===", tier_label(tier), ctx.ai(), seed0 + n.max(1) - 1);
    println!("knobs ({}): {}", ds.len(), ds.iter().map(|d| d.name.as_str()).collect::<Vec<_>>().join(", "));
    println!("built-in values: loss {base_loss:.2}");
    let (mut best_u, mut best_loss, mut best_m) = (base_u.clone(), base_loss, base_m.clone());
    for g in 1..=a.fit_gens {
        let xs = cma.ask(&mut rng);
        let mut losses = vec![];
        for x in &xs {
            let (u, out) = clamp(x);
            let (l, m) = eval(&u);
            if l < best_loss {
                (best_u, best_loss, best_m) = (u.clone(), l, m);
            }
            losses.push(l + 10.0 * out);
        }
        cma.tell(&xs, &losses);
        let mean = losses.iter().sum::<f64>() / losses.len() as f64;
        println!("  generation {g:>2}: best so far {best_loss:.2}, this generation {:.2} (mean {mean:.2}), step {:.3}", losses.iter().cloned().fold(f64::INFINITY, f64::min), cma.sigma);
    }
    let best = knobs_of(tier, &ds, &best_u);
    println!("\n  statistic                                          built-in        best      target       band");
    for s in &STATS {
        let Some(t) = targets.get(s.key) else { continue };
        let (b, v) = (base_m.get(s.key).map_or(f64::NAN, |x| x.value), best_m.get(s.key).map_or(f64::NAN, |x| x.value));
        println!("  {:<48} {b:>10.1} {v:>11.1} {:>11.1} {:>5.1}-{:<5.1}", s.what, t.value, t.lo, t.hi);
    }
    println!("  loss: built-in {base_loss:.2}, best {best_loss:.2}");
    println!("\n  best knobs:");
    for (d, &u) in ds.iter().zip(&best_u) {
        println!("    {:<44} {:>7.3}  (built-in {:.3})", d.name, d.from_unit(u), d.base);
    }
    let out = a.fit_out.clone().unwrap_or_else(|| format!("audit/realism/fit/{tier}-{}.toml", a.fit_scope));
    let path = if std::path::Path::new(&out).is_absolute() { std::path::PathBuf::from(&out) } else { root.join(&out) };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let header = vec![
        format!("Knobs fitted by commander-sim --fit --pool {tier} --fit-scope {} ({} AI, {prof} profile)", a.fit_scope, ctx.ai()),
        format!("{} generations of {}, {n} games a candidate on seeds {seed0}-{}", a.fit_gens, cma.lam, seed0 + n.max(1) - 1),
        "Loss: sum over the scored targets of ((sim - target) / half the band)^2".into(),
    ];
    let fit = vec![
        ("loss".into(), format!("{best_loss:.3}")),
        ("loss_builtin".into(), format!("{base_loss:.3}")),
        ("games".into(), n.to_string()),
        ("generations".into(), a.fit_gens.to_string()),
        ("scope".into(), format!("\"{}\"", a.fit_scope)),
    ];
    std::fs::write(&path, best.to_toml(&header, &fit)).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("\nwrote {} (run it with --knobs; confirm with --realism and more games)", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sep_cma_finds_the_minimum_of_a_bowl() {
        let target: Vec<f64> = (0..6).map(|i| 0.1 + 0.13 * i as f64).collect();
        let f = |x: &[f64]| x.iter().zip(&target).map(|(a, b)| (a - b).powi(2)).sum::<f64>();
        let mut cma = Cma::new(vec![0.5; 6], 0.3);
        let mut rng = Normal::new("test");
        for _ in 0..120 {
            let xs = cma.ask(&mut rng);
            let ls: Vec<f64> = xs.iter().map(|x| f(x)).collect();
            cma.tell(&xs, &ls);
        }
        assert!(f(&cma.m) < 1e-6, "mean {:?}, loss {}", cma.m, f(&cma.m));
    }

    #[test]
    fn a_fit_repeats_exactly_from_its_seed() {
        let a: Vec<f64> = { let mut r = Normal::new("fit:t6:500000"); (0..5).map(|_| r.next()).collect() };
        let b: Vec<f64> = { let mut r = Normal::new("fit:t6:500000"); (0..5).map(|_| r.next()).collect() };
        assert_eq!(a, b);
        assert!(a.iter().all(|x| x.is_finite() && x.abs() < 6.0));
    }

    #[test]
    fn clamping_reports_how_far_outside() {
        let (c, out) = clamp(&[-0.5, 0.5, 1.25]);
        assert_eq!(c, vec![0.0, 0.5, 1.0]);
        assert!((out - (0.25 + 0.0625)).abs() < 1e-12);
    }

    #[test]
    fn loss_is_one_on_the_edge_of_a_band() {
        let t = realism::parse_targets(
            "tier = \"t6\"\n[seat1_win]\nvalue = 32.5\nlo = 31.5\nhi = 33.5\nn = 1\nsource = \"s\"\ndate = \"d\"\n",
            "t6",
            "t.toml",
        )
        .unwrap();
        let mut m = BTreeMap::new();
        m.insert("seat1_win", Measured { value: 33.5, ci: None });
        assert!((loss(&m, &t) - 1.0).abs() < 1e-9);
        m.insert("seat1_win", Measured { value: 32.5, ci: None });
        assert_eq!(loss(&m, &t), 0.0);
    }
}
