//! --- t5-hold: the held-mana report of `--realism`. Reads the `hd|...` counters (`sim_core::holdstats`, which says what each
//! one counts); stats only, it reads finished games. Printed for the whole table (all seats pooled), then per deck, for the
//! holder's own turns 5-10 and for every own turn.

use std::collections::{BTreeMap, HashMap};

/// the groups of a snapshot, in report order, with what they are
const GROUPS: [(&str, &str); 6] = [
    ("ctr", "counter held"),
    ("prot", "protection held"),
    ("rem", "instant removal held"),
    ("adn", "Ad Nauseam held"),
    ("oth", "other card held"),
    ("idle", "nothing held, 2+ idle"),
];
const HELD: [&str; 5] = ["ctr", "prot", "rem", "adn", "oth"];
const OUTCOMES: [&str; 4] = ["used", "castnh", "else", "unused"];
const CLASSES: [&str; 7] = ["combo", "cmdr", "creature", "engine", "removal", "spell", "ability"];

/// the summed `hd|` counters of every deck
#[derive(Default, Clone, Debug)]
pub struct HoldTally {
    pub decks: BTreeMap<String, HashMap<String, i64>>,
}

fn get(m: &HashMap<String, i64>, k: &str) -> i64 {
    m.get(k).copied().unwrap_or(0)
}

fn pct(k: i64, n: i64) -> f64 {
    100.0 * k as f64 / n.max(1) as f64
}

impl HoldTally {
    /// add one player-game's counters (the `hd|` ones; any other key is skipped)
    pub fn add<'a>(&mut self, deck: &str, stats: impl Iterator<Item = (&'a str, i64)>) {
        for (k, v) in stats {
            if let Some(r) = k.strip_prefix("hd|") {
                *self.decks.entry(deck.to_string()).or_default().entry(r.to_string()).or_default() += v;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.decks.values().all(|d| get(d, "all|turns") == 0)
    }

    fn pooled(&self) -> HashMap<String, i64> {
        let mut o: HashMap<String, i64> = HashMap::new();
        for d in self.decks.values() {
            for (k, v) in d {
                *o.entry(k.clone()).or_default() += v;
            }
        }
        o
    }

    pub fn render(&self, name_of: &dyn Fn(&str) -> String) -> String {
        let mut o = String::new();
        let pooled = self.pooled();
        for (b, what) in [("mid", "own turns 5-10"), ("all", "every own turn")] {
            o += &render_one(&format!("all seats pooled, {what}"), &pooled, b, true);
        }
        for (b, what) in [("mid", "own turns 5-10"), ("all", "every own turn")] {
            for (key, d) in &self.decks {
                o += &render_one(&format!("{}, {what}", name_of(key)), d, b, false);
            }
        }
        o
    }
}

/// one block: the turns of one deck (or the table) in one range
fn render_one(label: &str, m: &HashMap<String, i64>, b: &str, detail: bool) -> String {
    let k = |s: &str| get(m, &format!("{b}|{s}"));
    let turns = k("turns");
    if turns == 0 {
        return String::new();
    }
    let held_n: i64 = HELD.iter().map(|g| k(&format!("{g}|n"))).sum();
    let mut o = format!("\n--- Held mana at the end of the turn: {label} ({turns} turns) ---\n");
    o += &format!(
        "  2+ mana unused and a castable card in hand: {:.1}% of turns; of those the castable card is not a held kind on {:.1}% (an answer held: {:.1}%, nothing held: {:.1}%)\n",
        pct(k("u2any"), turns),
        pct(k("u2c|held") + k("u2c|idle"), turns),
        pct(k("u2c|held"), turns),
        pct(k("u2c|idle"), turns),
    );
    o += &format!(
        "  turns ending with mana held for an answer (hold_of named a card): {:.1}% ({held_n}); idle turns (2+ unused, castable, nothing held): {:.1}% ({})\n",
        pct(held_n, turns),
        pct(k("idle|n"), turns),
        k("idle|n"),
    );
    o += &format!(
        "  {:24}{:>7}{:>7}{:>8}{:>6} | by the holder's next turn: {:>6}{:>9}{:>9}{:>8}{:>7} | {:>9}{:>7}\n",
        "group",
        "turns",
        "% turns",
        "unused",
        "hold",
        "used",
        "cast,miss",
        "elsewhere",
        "unused",
        "ended",
        "spent/avg",
        "oinst"
    );
    let mut rows: Vec<(String, Vec<String>)> =
        GROUPS.iter().map(|(g, l)| (l.to_string(), vec![g.to_string()])).collect();
    rows.insert(5, ("all held together".into(), HELD.iter().map(|g| g.to_string()).collect()));
    for (l, gs) in &rows {
        let s = |name: &str| -> i64 { gs.iter().map(|g| k(&format!("{g}|{name}"))).sum() };
        let n = s("n");
        if n == 0 {
            continue;
        }
        let done: i64 = OUTCOMES.iter().map(|x| s(x)).sum();
        o += &format!(
            "  {l:24}{n:>7}{:>7.1}{:>8.2}{:>6.2} | {:>38.1}%{:>8.1}%{:>8.1}%{:>7.1}%{:>6.1}% | {:>9.2}{:>7.2}\n",
            pct(n, turns),
            s("mana") as f64 / n as f64,
            s("hv") as f64 / 100.0 / n as f64,
            pct(s("used"), n),
            pct(s("castnh"), n),
            pct(s("else"), n),
            pct(s("unused"), n),
            pct(n - done, n),
            s("spent") as f64 / done.max(1) as f64,
            s("oinst") as f64 / done.max(1) as f64,
        );
    }
    if detail {
        o += "  what the card that was used hit (counter: the spell countered; removal: the permanent removed; protection: the permanent saved):\n";
        for (g, l) in GROUPS.iter().take(3) {
            let tot: i64 = CLASSES.iter().map(|c| k(&format!("{g}|hit|{c}"))).sum();
            if tot == 0 {
                continue;
            }
            let parts: Vec<String> = CLASSES
                .iter()
                .filter(|c| k(&format!("{g}|hit|{c}")) > 0)
                .map(|c| format!("{c} {:.0}% ({})", pct(k(&format!("{g}|hit|{c}")), tot), k(&format!("{g}|hit|{c}"))))
                .collect();
            o += &format!("    {l:24}{tot:>5} hits: {}\n", parts.join(", "));
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blocks_read_the_counters() {
        let mut t = HoldTally::default();
        let st = [
            ("hd|all|turns", 10),
            ("hd|all|ctr|n", 4),
            ("hd|all|ctr|used", 1),
            ("hd|all|ctr|unused", 3),
            ("hd|all|ctr|hit|combo", 1),
            ("fl|n|T1", 5),
        ];
        t.add("zur", st.iter().map(|(k, v)| (*k, *v)));
        assert!(!t.is_empty());
        let r = t.render(&|k| k.to_string());
        assert!(r.contains("counter held"), "{r}");
        assert!(r.contains("combo 100% (1)"), "{r}");
        assert!(!r.contains("fl|"), "{r}");
    }
}
