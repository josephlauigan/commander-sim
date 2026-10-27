//! --- ai-audit: the goldfish report's loss accounting (`sim_core::gf_audit` records it; this renders it). Each own
//! turn of the deck under test falls in one bucket (first match wins, see `sim_core::gf_audit::classify`); then the
//! passed cards by role, the cleanup discards, the main-phase "stop" choices, the look-ahead's sure-win cap and the
//! crude best case.

use crate::goldfish::GfGame;
use crate::stats::Counter;
use sim_core::gf_audit::{AuditOut, BUCKETS, Bucket, ROLES, Role};

fn pct(k: usize, n: usize) -> f64 {
    100.0 * k as f64 / n.max(1) as f64
}

fn top(c: &Counter<String>, n: usize) -> String {
    let v: Vec<String> = c.most_common().iter().take(n).map(|(k, v)| format!("{k} {v}")).collect();
    if v.is_empty() { "-".into() } else { v.join(", ") }
}

fn median(v: &mut [i64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_unstable();
    let n = v.len();
    Some((v[(n - 1) / 2] + v[n / 2]) as f64 / 2.0)
}

/// bucket counts over all games: (all own turns, own turns 1-4)
pub fn bucket_counts(audits: &[&AuditOut]) -> (Vec<(Bucket, usize)>, Vec<(Bucket, usize)>) {
    let count = |early: bool| {
        BUCKETS
            .iter()
            .map(|&b| {
                let n =
                    audits.iter().flat_map(|a| &a.turns).filter(|t| t.bucket == b && (!early || t.turn <= 4)).count();
                (b, n)
            })
            .collect::<Vec<_>>()
    };
    (count(false), count(true))
}

fn role_line(roles: &[Role]) -> String {
    let n = roles.len();
    let v: Vec<String> = ROLES
        .iter()
        .filter_map(|&r| {
            let k = roles.iter().filter(|&&x| x == r).count();
            (k > 0).then(|| format!("{} {k} ({:.0}%)", r.name(), pct(k, n)))
        })
        .collect();
    if v.is_empty() { "-".into() } else { v.join(", ") }
}

fn util_bin(u: Option<f64>) -> &'static str {
    match u {
        None => "nothing else offered",
        Some(u) if u < 0.0 => "< 0",
        Some(u) if u < 3.0 => "0-3",
        Some(u) if u < 6.0 => "3-6",
        Some(u) if u < 10.0 => "6-10",
        Some(_) => ">= 10",
    }
}

const BINS: [&str; 6] = ["nothing else offered", "< 0", "0-3", "3-6", "6-10", ">= 10"];

/// the loss-accounting section of one deck's goldfish report
pub fn render(games: &[GfGame]) -> String {
    let audits: Vec<&AuditOut> = games.iter().filter_map(|g| g.audit.as_ref()).collect();
    let n = audits.len();
    if n == 0 {
        return String::new();
    }
    let mut o = String::from("Loss accounting [ai-audit] (each own turn in one bucket, the first that matches):\n");
    let turns: usize = audits.iter().map(|a| a.turns.len()).sum();
    let early: usize = audits.iter().flat_map(|a| &a.turns).filter(|t| t.turn <= 4).count();
    o += &format!("  own turns: {turns} ({:.2} a game; {early} of them turns 1-4)\n", turns as f64 / n as f64);
    o += &format!("  {:18} {:>10} {:>11} {:>9}\n", "bucket", "all turns", "turns 1-4", "per game");
    let (all, first4) = bucket_counts(&audits);
    for ((b, k), (_, k4)) in all.iter().zip(&first4) {
        o += &format!(
            "  {:18} {:>9.1}% {:>10.1}% {:>9.2}\n",
            b.name(),
            pct(*k, turns),
            pct(*k4, early),
            *k as f64 / n as f64
        );
    }
    // passed_castable: the cards behind it
    let passed: Vec<(String, Role)> = audits.iter().flat_map(|a| &a.turns).flat_map(|t| t.passed.clone()).collect();
    let roles: Vec<Role> = passed.iter().map(|x| x.1).collect();
    o += &format!("  passed_castable cards (card-turns, a turn can hold several): {}\n", role_line(&roles));
    let mut pc: Counter<String> = Counter::new();
    for (c, _) in &passed {
        pc.add(c.clone(), 1);
    }
    o += &format!("    most passed: {}\n", top(&pc, 12));
    // payable cards never offered (instants, cards a rule holds, a priority of 0)
    let held: Vec<(String, Role)> = audits.iter().flat_map(|a| &a.turns).flat_map(|t| t.held.clone()).collect();
    let hturns = audits.iter().flat_map(|a| &a.turns).filter(|t| !t.held.is_empty()).count();
    let hroles: Vec<Role> = held.iter().map(|x| x.1).collect();
    o += &format!(
        "  payable but never offered as a play (card-turns; in {:.1}% of turns): {}\n",
        pct(hturns, turns),
        role_line(&hroles)
    );
    let mut hc: Counter<String> = Counter::new();
    for (c, _) in &held {
        hc.add(c.clone(), 1);
    }
    o += &format!("    most held: {}\n", top(&hc, 12));
    let mut lines: Counter<String> = Counter::new();
    for t in audits.iter().flat_map(|a| &a.turns) {
        if let Some(l) = &t.line {
            lines.add(l.clone(), 1);
        }
    }
    o += &format!("  ready_not_taken, the line ready: {}\n", top(&lines, 6));
    let mut rc: Counter<String> = Counter::new();
    for a in &audits {
        for c in &a.ready_choices {
            rc.add(c.clone(), 1);
        }
    }
    o += &format!("    chosen at decisions with a line ready (before going for one): {}\n", top(&rc, 10));
    // cleanup discards
    let dis: Vec<&(String, Role, bool)> = audits.iter().flat_map(|a| &a.discards).collect();
    let with = audits.iter().filter(|a| !a.discards.is_empty()).count();
    let pieces = audits.iter().filter(|a| a.discards.iter().any(|d| d.1 == Role::ComboPiece)).count();
    o += &format!(
        "Cleanup discards to 7: {} ({:.2} a game, in {:.1}% of games); combo pieces {} (in {:.1}% of games); {} that ai::discard_keep would have kept\n",
        dis.len(),
        dis.len() as f64 / n as f64,
        pct(with, n),
        dis.iter().filter(|d| d.1 == Role::ComboPiece).count(),
        pct(pieces, n),
        dis.iter().filter(|d| d.2).count()
    );
    let droles: Vec<Role> = dis.iter().map(|d| d.1).collect();
    o += &format!("  by role: {}\n", role_line(&droles));
    let mut dc: Counter<String> = Counter::new();
    for d in &dis {
        dc.add(d.0.clone(), 1);
    }
    o += &format!("  most discarded: {}\n", top(&dc, 12));
    // stops
    o += "Main-phase stops, by who chose and the phase: the best other option by heuristic utility (count by bin)\n";
    for post in [false, true] {
        for search in [true, false] {
            for label in ["stop", "stop (hold mana)"] {
                let ss: Vec<_> = audits
                    .iter()
                    .flat_map(|a| &a.stops)
                    .filter(|s| s.search == search && s.label == label && s.post == post)
                    .collect();
                if ss.is_empty() {
                    continue;
                }
                let bins: Vec<String> = BINS
                    .iter()
                    .map(|b| (b, ss.iter().filter(|s| util_bin(s.alt.as_ref().map(|x| x.1)) == *b).count()))
                    .filter(|x| x.1 > 0)
                    .map(|(b, k)| format!("{b}: {k}"))
                    .collect();
                let early = ss.iter().filter(|s| s.turn <= 4).count();
                let ready = ss.iter().filter(|s| s.ready).count();
                o += &format!(
                    "  {} {} \"{label}\": {} ({:.2} a game; {early} on turns 1-4; {ready} with a line ready) | {}\n",
                    if post { "main 2" } else { "main 1" },
                    if search { "look-ahead" } else { "heuristic " },
                    ss.len(),
                    ss.len() as f64 / n as f64,
                    bins.join(", ")
                );
                let mut alt: Counter<String> = Counter::new();
                for s in &ss {
                    if let Some((l, _)) = &s.alt {
                        alt.add(l.clone(), 1);
                    }
                }
                o += &format!("      best other option: {}\n", top(&alt, 8));
                if search {
                    let gaps: Vec<f64> = ss.iter().filter_map(|s| s.scores.map(|(st, b)| b - st)).collect();
                    if !gaps.is_empty() {
                        let m = gaps.iter().sum::<f64>() / gaps.len() as f64;
                        let close = gaps.iter().filter(|&&x| x > -1.0).count();
                        o += &format!(
                            "      look-ahead score, best other minus stop: mean {m:.2}; within 1 point of the stop {} of {}\n",
                            close,
                            gaps.len()
                        );
                    }
                }
            }
        }
    }
    let cap: u32 = audits.iter().map(|a| a.cap_changed).sum();
    o += &format!(
        "Sure-win cap (d10bbb05): changed the look-ahead's pick {cap} times, in {:.1}% of games\n",
        pct(audits.iter().filter(|a| a.cap_changed > 0).count(), n)
    );
    o += &best_section(games, n, "tutors aside", |a| a.best.as_ref());
    o += &best_section(games, n, "each seen tutor standing in for one missing piece", |a| a.best_tutors.as_ref());
    o
}

/// the crude best case of each game (`pick`: which variant) and the kill turn's distance from it
fn best_section(games: &[GfGame], n: usize, what: &str, pick: impl Fn(&AuditOut) -> Option<&(u32, String)>) -> String {
    let mut o = String::new();
    let best: Vec<i64> = games.iter().filter_map(|g| pick(g.audit.as_ref()?).map(|b| b.0 as i64)).collect();
    let mut bv = best.clone();
    o += &format!(
        "Crude best case (a line's pieces and its mana seen; {what}): in {:.1}% of games, median own turn {}\n",
        pct(best.len(), n),
        median(&mut bv).map_or("-".into(), |m| format!("{m:.1}"))
    );
    let bys: Vec<String> =
        (1..=6).map(|t| format!("T{t} {:.1}%", pct(best.iter().filter(|&&b| b <= t).count(), n))).collect();
    o += &format!("  best case by turn (share of all games): {}\n", bys.join("  "));
    let mut bl: Counter<String> = Counter::new();
    for g in games {
        if let Some((_, l)) = g.audit.as_ref().and_then(&pick) {
            bl.add(l.clone(), 1);
        }
    }
    o += &format!("  lines: {}\n", top(&bl, 4));
    let mut gaps: Vec<i64> =
        games.iter().filter_map(|g| Some(g.kill? as i64 - pick(g.audit.as_ref()?)?.0 as i64)).collect();
    let ng = gaps.len();
    let med = median(&mut gaps);
    let bin = |f: &dyn Fn(i64) -> bool| pct(gaps.iter().filter(|&&x| f(x)).count(), ng);
    let mean = if ng > 0 { gaps.iter().sum::<i64>() as f64 / ng as f64 } else { 0.0 };
    o += &format!(
        "  kill turn - best case ({ng} games with both): median {}, mean {mean:.2}; < 0: {:.1}%, 0: {:.1}%, 1: {:.1}%, 2: {:.1}%, 3: {:.1}%, 4-5: {:.1}%, 6+: {:.1}%\n",
        med.map_or("-".into(), |m| format!("{m:.1}")),
        bin(&|x| x < 0),
        bin(&|x| x == 0),
        bin(&|x| x == 1),
        bin(&|x| x == 2),
        bin(&|x| x == 3),
        bin(&|x| (4..=5).contains(&x)),
        bin(&|x| x >= 6)
    );
    let nokill =
        games.iter().filter(|g| g.kill.is_none() && g.audit.as_ref().is_some_and(|a| pick(a).is_some())).count();
    o += &format!("  games with a best case and no kill by the round cap: {nokill}\n");
    o
}

/// one traced game's audit, turn by turn (`--trace`)
pub fn render_trace(a: &AuditOut) -> String {
    let mut o = String::from("Loss accounting [ai-audit], by own turn:\n");
    for t in &a.turns {
        let mut s = format!("  T{:<2} {}", t.turn, t.bucket.name());
        if let Some(l) = &t.line {
            s += &format!(" ({l})");
        }
        if !t.passed.is_empty() {
            let v: Vec<String> = t.passed.iter().map(|(c, r)| format!("{c} [{}]", r.name())).collect();
            s += &format!(": {}", v.join(", "));
        }
        if !t.held.is_empty() {
            let v: Vec<String> = t.held.iter().map(|(c, _)| c.clone()).collect();
            s += &format!(" | held (payable, not offered): {}", v.join(", "));
        }
        o += &s;
        o += "\n";
    }
    let d: Vec<String> = a
        .discards
        .iter()
        .map(|(c, r, k)| format!("{c} [{}]{}", r.name(), if *k { " (kept by discard_keep)" } else { "" }))
        .collect();
    o += &format!("  cleanup discards: {}\n", if d.is_empty() { "none".into() } else { d.join(", ") });
    let s: Vec<String> = a
        .stops
        .iter()
        .filter(|s| s.alt.is_some())
        .map(|s| {
            format!(
                "T{} {} {} \"{}\" over {}",
                s.turn,
                if s.post { "main 2" } else { "main 1" },
                if s.search { "look-ahead" } else { "heuristic" },
                s.label,
                s.alt.as_ref().map_or("nothing".into(), |(l, u)| format!("{l} ({u:.1})"))
            )
        })
        .collect();
    o += &format!("  stops with another option offered: {}\n", if s.is_empty() { "none".into() } else { s.join("; ") });
    o += &format!(
        "  crude best case: {}\n",
        a.best.as_ref().map_or("none".into(), |(t, l)| format!("own turn {t} ({l})"))
    );
    o += &format!(
        "  crude best case with tutors: {}\n",
        a.best_tutors.as_ref().map_or("none".into(), |(t, l)| format!("own turn {t} ({l})"))
    );
    o
}
