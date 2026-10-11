//! One progress bar for the whole command (compare.Progress): games done / planned, elapsed, estimated time left.
//! On a terminal it redraws one line; redirected, it prints a line every 10%.

use std::io::{IsTerminal, Write};
use std::time::Instant;

/// CPU seconds per game for the estimate (measured on the Rust engine, 10/10)
pub fn sec_per_game(ai: &str) -> f64 {
    match ai {
        "lookahead" => 3.0,
        _ => 0.05,
    }
}

pub struct Progress {
    pub total: u64,
    pub done: u64,
    pub t0: Option<Instant>,
    pub label: String,
    pub on: bool,
    last: Option<Instant>,
    tty: bool,
    next_pct: f64,
}

impl Default for Progress {
    fn default() -> Self {
        Progress::new()
    }
}

/// 1234567 -> "1,234,567" (Python's {:,})
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn mmss(x: f64) -> String {
    let x = x.max(0.0);
    format!("{}m{:02}s", (x / 60.0).floor() as u64, (x % 60.0).floor() as u64)
}

impl Progress {
    pub fn new() -> Progress {
        Progress {
            total: 0,
            done: 0,
            t0: None,
            label: String::new(),
            on: true,
            last: None,
            tty: std::io::stderr().is_terminal(),
            next_pct: 10.0,
        }
    }

    pub fn plan(&mut self, games: u64) {
        self.total += games;
    }

    /// one line before the run starts: how many games, with what, and roughly how long; then an empty bar
    pub fn announce(&mut self, what: &str, ai: &str, jobs: usize) {
        if !self.on || self.total == 0 {
            return;
        }
        let est = self.total as f64 * sec_per_game(ai) / (jobs as u64).min(self.total).max(1) as f64;
        let fmt = |x: f64| {
            if x >= 5400.0 {
                format!("{:.1} hours", x / 3600.0)
            } else if x >= 90.0 {
                format!("{:.0} minutes", x / 60.0)
            } else {
                format!("{x:.0} seconds")
            }
        };
        eprintln!(
            "{} games ({what}), {ai} AI, {jobs} worker thread{}: about {}",
            thousands(self.total),
            if jobs > 1 { "s" } else { "" },
            fmt(est)
        );
        if ai == "lookahead" && est > 3600.0 {
            eprintln!("  (for a quick read: --ai adaptive is about 60x faster; --games 240 --profile loose also cut it down)");
        }
        self.t0 = Some(Instant::now());
        self.update(0);
    }

    pub fn update(&mut self, k: u64) {
        let now = Instant::now();
        let t0 = *self.t0.get_or_insert(now);
        self.done += k;
        if !self.on || self.total == 0 {
            return;
        }
        let pct = 100.0 * self.done as f64 / self.total as f64;
        let el = (now - t0).as_secs_f64();
        let left = if self.done > 0 { el / self.done as f64 * (self.total - self.done.min(self.total)) as f64 } else { 0.0 };
        let mut err = std::io::stderr().lock();
        if self.tty {
            if let Some(last) = self.last
                && (now - last).as_secs_f64() < 0.25
                && self.done < self.total
            {
                return;
            }
            self.last = Some(now);
            let filled = ((pct / 4.0) as usize).min(25);
            let bar = format!("{}{}", "#".repeat(filled), "-".repeat(25 - filled));
            let label: String = self.label.chars().take(28).collect();
            let _ = write!(
                err,
                "\r[{bar}] {pct:5.1}%  {}/{} games  {} elapsed, ~{} left  {label:28}",
                thousands(self.done),
                thousands(self.total),
                mmss(el),
                if self.done > 0 { mmss(left) } else { "?".into() }
            );
            if self.done >= self.total {
                let _ = writeln!(err);
            }
            let _ = err.flush();
        } else if pct >= self.next_pct || self.done >= self.total {
            self.next_pct = (pct / 10.0).floor() * 10.0 + 10.0;
            let _ = writeln!(err, "  progress {pct:5.1}%  {} elapsed, ~{} left", mmss(el), mmss(left));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separator() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(960), "960");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }

    #[test]
    fn minutes_seconds() {
        assert_eq!(mmss(0.0), "0m00s");
        assert_eq!(mmss(61.9), "1m01s");
        assert_eq!(mmss(3725.0), "62m05s");
    }
}
