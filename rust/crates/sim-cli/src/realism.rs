//! `commander-sim --realism --pool tN`: the tier's own decks played against each other (the games of
//! `--calibrate within`), each scorecard statistic computed from those games and set against the real-game targets in
//! `data/realism/tN.toml` (sim value, its 95% interval, the target, the band the sim must land in, PASS or FAIL).
//!
//! A targets file holds `tier = "tN"` and one table per statistic it has a target for:
//!
//! ```toml
//! [seat1_win]
//! value = 30.5            # the real value, in the statistic's unit
//! lo = 27.5               # the band the sim must land in
//! hi = 33.5
//! n = 387427              # the real sample size
//! source = "EDHTop16 monthly seat win rates, Apr-Sep 2026"
//! date = "2026-10-22"     # when the source was read
//! note = "optional"
//! scored = false         # optional: a reference value, printed with its band but not scored (default true)
//! ```
//!
//! A statistic with no table has no target yet (printed, not scored). A table with `scored = false` is a reference: its
//! value and band are printed (result `reference`) but it is left out of the pass count and of `--fit`. Tier 6 (cEDH)
//! keeps its win types as references: their only source is casual logs (JD, 2026-10-24). Every table needs all six fields: a file
//! missing one, or naming an unknown statistic or field, is refused before any game is played.

use crate::args::Args;
use crate::decks::{self, TIERS, tier_label};
use crate::play::{GameOut, Sim};
use crate::poolmode::Ctx;
use crate::stats::wilson;
use crate::t6report::{T6Tally, win_route};
use std::collections::BTreeMap;
use std::path::Path;

/// a scorecard statistic: its key in the targets file, what it measures and its unit
pub struct Stat {
    pub key: &'static str,
    pub what: &'static str,
    pub unit: &'static str,
}

const fn stat(key: &'static str, what: &'static str, unit: &'static str) -> Stat {
    Stat { key, what, unit }
}

/// the scorecard, in print order; "decisive" = a game with a winner that was not the round-cap tiebreak
pub const STATS: [Stat; 12] = [
    stat("seat1_win", "seat 1 share of decisive games", "%"),
    stat("seat2_win", "seat 2 share of decisive games", "%"),
    stat("seat3_win", "seat 3 share of decisive games", "%"),
    stat("seat4_win", "seat 4 share of decisive games", "%"),
    stat("win_round_median", "median round of a decisive win", "rounds"),
    stat("win_round_spread", "win round, 75th minus 25th percentile", "rounds"),
    stat("win_combo_alt", "wins by a combo or an alternate win", "%"),
    stat("win_damage", "wins by damage (combat or other)", "%"),
    stat("win_commander_damage", "wins by commander damage", "%"),
    stat("win_poison", "wins by poison", "%"),
    stat("deck_win_spread", "deck win spread, best minus worst deck", "pts"),
    stat("games_over_15", "games over 15 rounds (round cap 20)", "%"),
];

const FIELDS: [&str; 8] = ["value", "lo", "hi", "n", "source", "date", "note", "scored"];

/// one target: the real value, the band the sim must land in, the real sample size, where it came from and when
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub value: f64,
    pub lo: f64,
    pub hi: f64,
    pub n: u64,
    pub source: String,
    pub date: String,
    pub note: Option<String>,
    /// false: a reference value (printed with its band, not scored or fitted)
    pub scored: bool,
}

/// a tier's targets file, checked: statistic key -> target
pub type Targets = BTreeMap<String, Target>;

/// data/realism/<tier>.toml
pub fn targets_path(root: &Path, tier: &str) -> std::path::PathBuf {
    root.join("data").join("realism").join(format!("{tier}.toml"))
}

/// Read and check a targets file's text (`file` names it in errors). Refused: bad TOML, a missing or wrong `tier`, an
/// unknown statistic or field, a missing field, a number of the wrong kind, a value outside its band.
pub fn parse_targets(text: &str, tier: &str, file: &str) -> Result<Targets, String> {
    let t: toml::Table = text.parse().map_err(|e| format!("{file}: not valid TOML: {e}"))?;
    match t.get("tier").and_then(|v| v.as_str()) {
        Some(x) if x == tier => {}
        Some(x) => return Err(format!("{file}: tier = \"{x}\", but this is the file for {tier}")),
        None => return Err(format!("{file}: missing `tier = \"{tier}\"`")),
    }
    let mut out = Targets::new();
    for (key, v) in &t {
        if key == "tier" {
            continue;
        }
        if !STATS.iter().any(|s| s.key == key) {
            let known: Vec<&str> = STATS.iter().map(|s| s.key).collect();
            return Err(format!("{file}: unknown statistic [{key}] (known: {})", known.join(", ")));
        }
        let tb = v.as_table().ok_or_else(|| format!("{file}: `{key}` must be a table ([{key}])"))?;
        if let Some(f) = tb.keys().find(|f| !FIELDS.contains(&f.as_str())) {
            return Err(format!("{file}: [{key}] has an unknown field `{f}` (fields: {})", FIELDS.join(", ")));
        }
        let missing = |f: &str| {
            format!("{file}: [{key}] is missing `{f}` (every statistic needs value, lo, hi, n, source and date)")
        };
        let num = |f: &str| -> Result<f64, String> {
            match tb.get(f) {
                None => Err(missing(f)),
                Some(toml::Value::Float(x)) => Ok(*x),
                Some(toml::Value::Integer(x)) => Ok(*x as f64),
                Some(_) => Err(format!("{file}: [{key}] `{f}` must be a number")),
            }
        };
        let text = |f: &str| -> Result<String, String> {
            match tb.get(f) {
                None => Err(missing(f)),
                Some(toml::Value::String(s)) if !s.trim().is_empty() => Ok(s.clone()),
                Some(_) => Err(format!("{file}: [{key}] `{f}` must be a non-empty string")),
            }
        };
        let (value, lo, hi) = (num("value")?, num("lo")?, num("hi")?);
        let n = match tb.get("n") {
            None => return Err(missing("n")),
            Some(toml::Value::Integer(x)) if *x > 0 => *x as u64,
            Some(_) => return Err(format!("{file}: [{key}] `n` must be a whole number above 0")),
        };
        let (source, date) = (text("source")?, text("date")?);
        if !(lo <= value && value <= hi) {
            return Err(format!("{file}: [{key}] value {value} is outside its band {lo}-{hi}"));
        }
        let note = tb.get("note").and_then(|v| v.as_str()).map(str::to_string);
        let scored = match tb.get("scored") {
            None => true,
            Some(toml::Value::Boolean(b)) => *b,
            Some(_) => return Err(format!("{file}: [{key}] `scored` must be true or false")),
        };
        out.insert(key.clone(), Target { value, lo, hi, n, source, date, note, scored });
    }
    Ok(out)
}

pub fn load_targets(root: &Path, tier: &str) -> Result<Targets, String> {
    let p = targets_path(root, tier);
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e} (no targets file for {tier})", p.display()))?;
    parse_targets(&text, tier, &p.display().to_string())
}

/// the sim's value of a statistic and, where one is cheap, its 95% interval
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured {
    pub value: f64,
    pub ci: Option<(f64, f64)>,
}

/// what the games show besides the scorecard: game and decisive counts, timeouts, each deck's decisive share
pub struct Shape {
    pub games: u64,
    pub decisive: u64,
    pub timeouts: u64,
    pub decks: Vec<(String, u64, u64)>,
}

fn pct_ci(k: u64, n: u64) -> Measured {
    let (lo, hi) = wilson(k, n);
    Measured { value: 100.0 * k as f64 / n.max(1) as f64, ci: Some(((100.0 * lo).max(0.0), (100.0 * hi).min(100.0))) }
}

/// the q-th quantile of sorted v (nearest rank)
fn quantile(v: &[u32], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let i = ((q * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1;
    v[i] as f64
}

/// every scorecard statistic from the games; keys are the tier's deck keys (for the per-deck spread)
pub fn measure(games: &[GameOut], keys: &[String]) -> (BTreeMap<&'static str, Measured>, Shape) {
    let mut t = T6Tally::default();
    let mut rounds = vec![];
    let mut routes = [0u64; 4]; // combo or alternate, damage, commander damage, poison
    let mut over15 = 0;
    let mut decks: BTreeMap<&str, (u64, u64)> = keys.iter().map(|k| (k.as_str(), (0, 0))).collect();
    for g in games {
        t.add(g);
        over15 += (g.round > 15) as u64;
        let Some(w) = g.winner.filter(|_| g.wintype != Some("timeout")) else { continue };
        rounds.push(g.round);
        let r = match win_route(g, w).as_deref() {
            Some("damage") => 1,
            Some("commander damage") => 2,
            Some("poison") => 3,
            _ => 0, // a combo line, or "other" (an alternate win, mill)
        };
        routes[r] += 1;
        for (i, p) in g.players.iter().enumerate() {
            if let Some(d) = decks.get_mut(p.key) {
                d.0 += (i == w) as u64;
                d.1 += 1;
            }
        }
    }
    rounds.sort();
    let dec = t.decisive();
    let mut m = BTreeMap::new();
    for s in 0..4 {
        m.insert(STATS[s].key, pct_ci(t.seat_wins[s], dec));
    }
    // the median's interval from the order statistics around it (binomial, 95%)
    let nr = rounds.len();
    let med_ci = (nr > 0).then(|| {
        let h = 0.98 * (nr as f64).sqrt();
        let lo = ((nr as f64 / 2.0 - h).floor() as usize).min(nr - 1);
        let hi = ((nr as f64 / 2.0 + h).ceil() as usize).min(nr - 1);
        (rounds[lo] as f64, rounds[hi] as f64)
    });
    m.insert("win_round_median", Measured { value: quantile(&rounds, 0.5), ci: med_ci });
    m.insert("win_round_spread", Measured { value: quantile(&rounds, 0.75) - quantile(&rounds, 0.25), ci: None });
    for (i, k) in ["win_combo_alt", "win_damage", "win_commander_damage", "win_poison"].iter().enumerate() {
        m.insert(*k, pct_ci(routes[i], dec));
    }
    let shares: Vec<f64> = decks.values().filter(|d| d.1 > 0).map(|&(w, n)| 100.0 * w as f64 / n as f64).collect();
    let spread = match shares.iter().cloned().reduce(f64::max).zip(shares.iter().cloned().reduce(f64::min)) {
        Some((hi, lo)) => hi - lo,
        None => 0.0,
    };
    m.insert("deck_win_spread", Measured { value: spread, ci: None });
    m.insert("games_over_15", pct_ci(over15, games.len() as u64));
    let shape = Shape {
        games: games.len() as u64,
        decisive: dec,
        timeouts: t.timeouts,
        decks: decks.into_iter().map(|(k, (w, n))| (k.to_string(), w, n)).collect(),
    };
    (m, shape)
}

fn num(x: f64, unit: &str) -> String {
    match unit {
        "%" => format!("{x:.1}%"),
        _ => format!("{x:.1}"),
    }
}

/// the scorecard table: one line per statistic, then the counts; returns (passed, scored)
pub fn render(m: &BTreeMap<&'static str, Measured>, targets: &Targets) -> (String, usize, usize) {
    let mut s =
        format!("  {:52} {:>7} {:>13} {:>7} {:>13}  result\n", "statistic", "sim", "sim 95% CI", "target", "band");
    let (mut pass, mut scored) = (0, 0);
    for st in &STATS {
        let v = m[&st.key];
        let ci = v.ci.map_or("-".to_string(), |(a, b)| format!("{:.1}-{:.1}", a, b));
        let label = format!("{} ({})", st.what, st.unit);
        match targets.get(st.key) {
            Some(t) => {
                let ok = t.lo <= v.value && v.value <= t.hi;
                if t.scored {
                    scored += 1;
                    pass += ok as usize;
                }
                s += &format!(
                    "  {label:52} {:>7} {ci:>13} {:>7} {:>13}  {}\n",
                    num(v.value, st.unit),
                    num(t.value, st.unit),
                    format!("{}-{}", t.lo, t.hi),
                    if !t.scored { "reference" } else if ok { "PASS" } else { "FAIL" }
                );
            }
            None => {
                s += &format!("  {label:52} {:>7} {ci:>13} {:>7} {:>13}  no target\n", num(v.value, st.unit), "-", "-")
            }
        }
    }
    (s, pass, scored)
}

/// --realism: refuse a bad command or targets file before playing, then play, score and print
pub fn run(sim: &Sim, a: &Args) -> Result<(), String> {
    let tier = match a.pool.as_deref() {
        Some(t) if TIERS.contains(&t) => t,
        _ => return Err("--realism needs --pool with one tier (t0..t6): it plays that tier's own decks".into()),
    };
    if a.deck.is_some() || a.calibrate.is_some() || a.all_decks || !a.swap.is_empty() {
        return Err("--realism plays the tier's own decks: drop --deck, --swap, --all-decks and --calibrate".into());
    }
    let root = decks::root()?;
    let targets = load_targets(&root, tier)?;
    let ctx = Ctx { sim, verbose: !a.brief };
    let keys = sim.pool_keys(tier);
    let (n, seed0) = (a.games, a.seed);
    {
        let mut pr = sim.prog.lock().unwrap();
        pr.on = !a.quiet;
        pr.plan(n * a.profiles.len() as u64);
        pr.announce(
            &format!("realism: {tier}, {n} games, profile {}", a.profiles.join(" and ")),
            ctx.ai(),
            sim.opts.jobs,
        );
    }
    for prof in &a.profiles {
        crate::play::profile(prof)?;
        ctx.label(format!("realism {tier} / {prof}"));
        let games: Vec<GameOut> = ctx.tier_games(None, &keys, 4, prof, n, seed0).into_iter().map(|(_, g)| g).collect();
        let (m, shape) = measure(&games, &keys);
        println!(
            "\n=== Realism: {} [{prof}, {} AI, {} games, seeds {seed0}-{}; {} decks, 4 a pod] ===",
            tier_label(tier),
            ctx.ai(),
            shape.games,
            seed0 + n.max(1) - 1,
            keys.len()
        );
        println!("targets: {}", targets_path(Path::new("."), tier).display().to_string().trim_start_matches("./"));
        let (table, pass, scored) = render(&m, &targets);
        print!("{table}");
        let refs = targets.values().filter(|t| !t.scored).count();
        let refs_note = if refs > 0 { format!("; {refs} reference values, not scored") } else { String::new() };
        println!(
            "  {pass} of {scored} targets pass ({} statistics have no target yet{refs_note})",
            STATS.len() - scored - refs
        );
        println!(
            "Games: {}, decisive {} ({:.1}%), round-cap timeouts {:.1}%",
            shape.games,
            shape.decisive,
            100.0 * shape.decisive as f64 / shape.games.max(1) as f64,
            100.0 * shape.timeouts as f64 / shape.games.max(1) as f64
        );
        let per: Vec<String> = shape
            .decks
            .iter()
            .map(|(k, w, d)| format!("{} {:.1}% of {d}", sim.name_of(k), 100.0 * *w as f64 / (*d).max(1) as f64))
            .collect();
        println!("Each deck's share of the decisive games it sat in: {}", per.join("; "));
        if !targets.is_empty() {
            println!("Target sources:");
            for st in &STATS {
                if let Some(t) = targets.get(st.key) {
                    let r = if t.scored { "" } else { " [reference, not scored]" };
                    println!("  {}: {} (n={}, read {}){r}", st.key, t.source, t.n, t.date);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
tier = "t6"
[seat1_win]
value = 30.5
lo = 27.5
hi = 33.5
n = 387427
source = "EDHTop16"
date = "2026-10-22"
"#;

    #[test]
    fn a_complete_file_is_read() {
        let t = parse_targets(GOOD, "t6", "t6.toml").unwrap();
        assert_eq!(t["seat1_win"].n, 387427);
        assert_eq!(t["seat1_win"].lo, 27.5);
    }

    #[test]
    fn a_missing_field_is_refused_with_its_name() {
        let e = parse_targets(&GOOD.replace("lo = 27.5\n", ""), "t6", "t6.toml").unwrap_err();
        assert!(e.contains("[seat1_win] is missing `lo`"), "{e}");
        let e = parse_targets(&GOOD.replace("source = \"EDHTop16\"\n", ""), "t6", "t6.toml").unwrap_err();
        assert!(e.contains("[seat1_win] is missing `source`"), "{e}");
    }

    #[test]
    fn unknown_statistics_fields_and_bad_values_are_refused() {
        let e = parse_targets(&GOOD.replace("[seat1_win]", "[seat_one]"), "t6", "t6.toml").unwrap_err();
        assert!(e.contains("unknown statistic [seat_one]"), "{e}");
        let e = parse_targets(&GOOD.replace("n = 387427", "n = 387427\nsize = 3"), "t6", "t6.toml").unwrap_err();
        assert!(e.contains("unknown field `size`"), "{e}");
        let e = parse_targets(&GOOD.replace("value = 30.5", "value = 40"), "t6", "t6.toml").unwrap_err();
        assert!(e.contains("outside its band"), "{e}");
        let e = parse_targets(GOOD, "t5", "t5.toml").unwrap_err();
        assert!(e.contains("file for t5"), "{e}");
    }

    /// every tier has a targets file in the repository, and each one passes the checks
    #[test]
    fn the_repository_targets_files_are_valid() {
        let root = crate::decks::root().unwrap();
        for t in TIERS {
            load_targets(&root, t).unwrap_or_else(|e| panic!("{e}"));
        }
        assert!(load_targets(&root, "t6").unwrap().contains_key("seat1_win"));
    }

    #[test]
    fn a_reference_value_is_printed_but_not_scored() {
        let t = parse_targets(&GOOD.replace("date = \"2026-10-22\"", "date = \"2026-10-22\"\nscored = false"), "t6", "t6.toml")
            .unwrap();
        assert!(!t["seat1_win"].scored);
        assert!(parse_targets(GOOD, "t6", "t6.toml").unwrap()["seat1_win"].scored, "scored by default");
        let e = parse_targets(&GOOD.replace("date = \"2026-10-22\"", "date = \"2026-10-22\"\nscored = 0"), "t6", "t6.toml")
            .unwrap_err();
        assert!(e.contains("`scored` must be true or false"), "{e}");
        let m: BTreeMap<&'static str, Measured> =
            STATS.iter().map(|s| (s.key, Measured { value: 30.0, ci: None })).collect();
        let (table, pass, scored) = render(&m, &t);
        assert_eq!((pass, scored), (0, 0), "a reference is not counted");
        assert!(table.contains("reference"), "{table}");
    }

    /// Tier 6 keeps its win types (casual logs) as references, out of the score (JD, 2026-10-24)
    #[test]
    fn tier_6_win_types_are_references() {
        let t = load_targets(&crate::decks::root().unwrap(), "t6").unwrap();
        for k in ["win_combo_alt", "win_damage", "win_commander_damage", "win_poison"] {
            assert!(!t[k].scored, "{k}");
        }
        assert!(t["seat1_win"].scored && t["win_round_median"].scored && t["deck_win_spread"].scored);
    }

    #[test]
    fn quantiles_take_the_nearest_rank() {
        let v = [5, 6, 7, 8, 9, 10, 11, 12];
        assert_eq!(quantile(&v, 0.5), 8.0);
        assert_eq!(quantile(&v, 0.25), 6.0);
        assert_eq!(quantile(&v, 0.75), 10.0);
    }
}
