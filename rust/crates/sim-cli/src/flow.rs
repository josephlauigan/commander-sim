//! --- ai-flow: the card and mana flow report of the Tier 6 decks, per own turn 1-8, from the `fl|...` counters
//! (`sim_core::t6stats`, the flow section). Stats only: it reads finished games. Printed after the `--realism --pool
//! t6` scorecard (every deck at the table) and in each `--goldfish` deck report (the deck under test).
//!
//! Per-turn rows are averages over the player-turns that began (`fl|n|T<t>`); the end-of-turn rows (cards seen, land
//! drops) over those that reached their end step (`fl|ne|T<t>`). Draws are counted on the own turn the card could
//! first be used: a Rhystic Study draw on an opponent's turn counts toward the drawer's next turn.

use std::collections::{BTreeMap, HashMap};

pub const TURNS: u32 = 8;

/// the draw-source classes, in report order (`t6stats::flow_class`)
pub const CLASSES: [(&str, &str); 6] = [
    ("step", "draw step"),
    ("cantrip", "cantrips (spells)"),
    ("engine", "engines"),
    ("wheel", "wheels"),
    ("cmdr", "commanders"),
    ("other", "other permanents"),
];

/// the summed counters of one deck (keys without the `fl|` prefix)
#[derive(Default, Clone, Debug)]
pub struct DeckFlow {
    pub sums: HashMap<String, i64>,
}

impl DeckFlow {
    fn get(&self, k: &str) -> i64 {
        self.sums.get(k).copied().unwrap_or(0)
    }

    /// the average of `what|T<t>` over the turns that began (`n`) or reached their end step (`ne`)
    pub fn avg(&self, what: &str, t: u32, end: bool) -> f64 {
        let d = self.get(&format!("{}|T{t}", if end { "ne" } else { "n" }));
        self.get(&format!("{what}|T{t}")) as f64 / d.max(1) as f64
    }

    /// cards drawn on turn t, every source together
    pub fn drawn(&self, t: u32) -> f64 {
        CLASSES.iter().map(|(c, _)| self.avg(&format!("d|{c}"), t, false)).sum()
    }

    /// games seated (turn 1 began)
    pub fn games(&self) -> i64 {
        self.get("n|T1")
    }
}

/// the flow counters of every deck
#[derive(Default, Clone, Debug)]
pub struct FlowTally {
    pub decks: BTreeMap<String, DeckFlow>,
}

impl FlowTally {
    /// add one player-game's counters (any key; only the `fl|` ones are read, the transient `fl|_` ones skipped)
    pub fn add<'a>(&mut self, deck: &str, stats: impl Iterator<Item = (&'a str, i64)>) {
        for (k, v) in stats {
            let Some(r) = k.strip_prefix("fl|") else { continue };
            if r.starts_with('_') {
                continue;
            }
            *self.decks.entry(deck.to_string()).or_default().sums.entry(r.to_string()).or_default() += v;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.decks.values().all(|d| d.games() == 0)
    }

    pub fn render(&self, name_of: &dyn Fn(&str) -> String) -> String {
        let mut o = String::new();
        for (key, d) in &self.decks {
            if d.games() > 0 {
                o += &render_deck(&name_of(key), d);
            }
        }
        o
    }
}

fn row(label: &str, f: &dyn Fn(u32) -> f64, dec: usize) -> String {
    let cells: Vec<String> = (1..=TURNS).map(|t| format!("{:>6.*}", dec, f(t))).collect();
    format!("  {label:34}{}\n", cells.join(""))
}

/// one deck's flow table
pub fn render_deck(name: &str, d: &DeckFlow) -> String {
    let mut o = format!("\n--- Card and mana flow: {name} ({} games; own turns 1-8) ---\n", d.games());
    let head: Vec<String> = (1..=TURNS).map(|t| format!("{:>6}", format!("T{t}"))).collect();
    o += &format!("  {:34}{}\n", "own turn", head.join(""));
    o += &row("turns that began (%)", &|t| 100.0 * d.get(&format!("n|T{t}")) as f64 / d.games().max(1) as f64, 0);
    o += &row("cards seen by the turn's end (cum.)", &|t| d.avg("seen", t, true), 1);
    o += &row("cards drawn", &|t| d.drawn(t), 2);
    for (c, label) in CLASSES {
        o += &row(&format!("  {label}"), &|t| d.avg(&format!("d|{c}"), t, false), 2);
    }
    o += &row("cards to hand by tutors and others", &|t| d.avg("tut", t, true), 2);
    o += &row("cards in hand (main phase)", &|t| d.avg("hand", t, false), 2);
    o += &row("spells cast, own turn", &|t| d.avg("cast", t, false), 2);
    o += &row("spells cast, others' turns after it", &|t| d.avg("casto", t, false), 2);
    o += &row("spells countered (of those)", &|t| d.avg("ctrd", t, false), 2);
    o += &row("mana available (main phase)", &|t| d.avg("mana", t, false), 2);
    o += &row("mana spent, own turn", &|t| d.avg("spent", t, false), 2);
    o += &row("mana spent, others' turns after it", &|t| d.avg("spento", t, false), 2);
    let life = |t: u32| -> f64 {
        let tot: i64 = d
            .sums
            .iter()
            .filter(|(k, _)| k.starts_with("life|") && k.ends_with(&format!("|T{t}")))
            .map(|(_, v)| *v)
            .sum();
        tot as f64 / d.get(&format!("n|T{t}")).max(1) as f64
    };
    o += &row("life paid for cards", &life, 2);
    o += &row("land drop made (%)", &|t| 100.0 * d.avg("land", t, true), 0);
    o += &row("missed, a land in hand (%)", &|t| 100.0 * d.avg("noldh", t, true), 0);
    o += &row("missed, no land in hand (%)", &|t| 100.0 * (d.avg("nold", t, true) - d.avg("noldh", t, true)), 0);
    // the cards drawn by source card, turns 1-8, per game
    let mut by: Vec<(&str, i64)> =
        d.sums.iter().filter_map(|(k, v)| k.strip_prefix("dc|").map(|c| (c, *v))).filter(|x| x.1 != 0).collect();
    by.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let g = d.games().max(1) as f64;
    let parts: Vec<String> = by.iter().map(|(c, v)| format!("{c} {:.2}", *v as f64 / g)).collect();
    o += &format!(
        "  cards drawn by source card, turns 1-8, per game: {}\n",
        if parts.is_empty() { "-".into() } else { parts.join(", ") }
    );
    // the engines while out: turn cycles out (turns 2-8), the cards each cycle, the tax triggers and how many were paid
    let mut outs: Vec<(&str, i64)> =
        d.sums.iter().filter_map(|(k, v)| k.strip_prefix("out|").map(|c| (c, *v))).filter(|x| x.1 > 0).collect();
    outs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (c, n) in outs {
        let cards = d.get(&format!("dc|{c}"));
        let trig = d.get(&format!("tax|{c}|trig"));
        let paid = d.get(&format!("tax|{c}|paid"));
        let tax = if trig > 0 {
            format!(
                "; {:.2} tax triggers a cycle, {:.0}% paid",
                trig as f64 / n as f64,
                100.0 * paid as f64 / trig as f64
            )
        } else {
            String::new()
        };
        o += &format!(
            "  {c} out: {:.2} cycles a game, {:.2} cards a cycle out{tax}\n",
            n as f64 / g,
            cards as f64 / n as f64
        );
    }
    // the engine funnel: the share of the turns that ended with the engine seen / cast / out, by own turn
    let mut fun: Vec<&str> = d
        .sums
        .keys()
        .filter_map(|k| k.strip_prefix("ef|").and_then(|r| r.split('|').next()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    fun.sort();
    if !fun.is_empty() {
        o += "  engine funnel, % of turns ended (seen / cast / out):\n";
        for c in fun {
            let cells: Vec<String> = [2, 3, 4, 6, 8]
                .iter()
                .map(|&t| {
                    let f = |w: &str| 100.0 * d.avg(&format!("ef|{c}|{w}"), t, true);
                    format!("T{t} {:.0}/{:.0}/{:.0}", f("seen"), f("cast"), f("out"))
                })
                .collect();
            let ctrd = d.get(&format!("ctrdc|{c}"));
            o += &format!("    {c:24} {}; countered {:.3} a game\n", cells.join("  "), ctrd as f64 / g);
        }
    }
    let mut fates: Vec<(&str, i64)> =
        d.sums.iter().filter_map(|(k, v)| k.strip_prefix("ez|").map(|c| (c, *v))).filter(|x| x.1 > 0).collect();
    fates.sort();
    if !fates.is_empty() {
        let parts: Vec<String> = fates.iter().map(|(k, v)| format!("{k} {v}")).collect();
        o += &format!("  seen, never cast, at the end of turn 4 (where it is, games): {}\n", parts.join(", "));
    }
    let mut lifes: BTreeMap<&str, i64> = BTreeMap::new();
    for (k, v) in &d.sums {
        if let Some(rest) = k.strip_prefix("life|")
            && let Some((card, _)) = rest.rsplit_once('|')
        {
            *lifes.entry(card).or_default() += v;
        }
    }
    let parts: Vec<String> = lifes.iter().map(|(c, v)| format!("{c} {:.2}", *v as f64 / g)).collect();
    o += &format!(
        "  life paid by card, turns 1-8, per game: {}\n",
        if parts.is_empty() { "-".into() } else { parts.join(", ") }
    );
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_divide_by_the_turns_that_began_and_skip_transient_keys() {
        let mut t = FlowTally::default();
        let a = [("fl|n|T1", 1), ("fl|ne|T1", 1), ("fl|d|step|T1", 1), ("fl|seen|T1", 8), ("fl|_seen", 8)];
        let b = [("fl|n|T1", 1), ("fl|d|step|T1", 1), ("fl|d|engine|T1", 2), ("mull_n", 1)];
        t.add("x", a.iter().copied());
        t.add("x", b.iter().copied());
        let d = &t.decks["x"];
        assert_eq!(d.games(), 2);
        assert_eq!(d.drawn(1), 2.0);
        assert_eq!(d.avg("seen", 1, true), 8.0);
        assert!(!d.sums.contains_key("_seen") && !d.sums.contains_key("mull_n"));
        assert!(render_deck("X", d).contains("cards drawn"));
    }
}
