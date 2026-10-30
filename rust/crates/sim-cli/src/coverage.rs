//! Model coverage, printed before the results of every run that plays decks (Phase 2 of documents/realism.md): for
//! each deck, how many of its 99 + commander are Full / Full-auto / Approximate / Partial / Unmodeled (the statuses of
//! `cardgen audit`, the same ones `--swap`'s model status prints), and the Unmodeled and Partial cards that matter
//! most: the commander first, then the deck's engine cards (data/audit.toml), then nonlands, then lands. Computed
//! once per run from the card data (no game is played and the RNG is never touched), so it changes no result.

use crate::args::Args;
use crate::decks::{TIERS, tier_label};
use crate::play::Sim;
use cardgen::audit::{Audit, STATUSES, rank};
use std::collections::{BTreeMap, BTreeSet};

/// the Unmodeled / Partial names shown per deck
const SHOWN: usize = 5;

/// one deck as a run plays it
pub struct DeckList {
    pub key: String,
    pub label: String,
    /// the deck list (or the variant list), the commander included
    pub cards: Vec<String>,
    pub commanders: Vec<String>,
    /// one of your decks: your decks' AI acts on the hand tags (`cardgen audit`'s `mine`)
    pub mine: bool,
}

pub enum Entry {
    /// a line per deck
    Deck(DeckList),
    /// one line for an opponent tier: its decks' counts summed, and the deck with the most Unmodeled cards
    Tier(String, Vec<DeckList>),
}

pub fn deck_list(sim: &Sim, key: &str, cards: &[String], label: Option<String>) -> DeckList {
    let commanders: Vec<String> = match sim.engine.decks.get(key) {
        Some(d) => std::iter::once(d.commander.clone()).chain(d.commander2.clone()).collect(),
        None => sim.commander(key).into_iter().collect(),
    };
    DeckList {
        key: key.to_string(),
        label: label.unwrap_or_else(|| key.to_string()),
        cards: cards.to_vec(),
        commanders,
        mine: sim.mine.contains_key(key),
    }
}

fn pool_list(sim: &Sim, key: &str) -> Option<DeckList> {
    let d = sim.pool.iter().find(|d| d.key == key)?;
    Some(deck_list(sim, key, &d.cards, Some(sim.name_of(key))))
}

/// a deck's statuses: (count by status over the 100 card slots, the not-Full names worst first, in the order above)
struct Counted {
    n: usize,
    cnt: BTreeMap<&'static str, usize>,
    worst: Vec<(String, &'static str)>,
}

fn count(au: &Audit, d: &DeckList) -> Counted {
    let engines: BTreeSet<&str> = au
        .data
        .engines
        .iter()
        .filter(|(k, _)| *k == d.key)
        .flat_map(|(_, es)| es.iter().flat_map(|e| e.cards.iter().map(String::as_str)))
        .collect();
    let mut cnt: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut worst = vec![];
    // the lists hold the commander already; a commander missing from one is added
    let slots: Vec<&String> = d.commanders.iter().filter(|c| !d.cards.contains(c)).chain(d.cards.iter()).collect();
    for name in &slots {
        let (s, _) = au.card_status(name, d.mine);
        let s = STATUSES.iter().copied().find(|x| *x == s).unwrap_or("Unmodeled");
        *cnt.entry(s).or_default() += 1;
        if matches!(s, "Partial" | "Unmodeled") && seen.insert(name.as_str()) {
            let order = if d.commanders.contains(name) {
                0
            } else if engines.contains(name.as_str()) {
                1
            } else if au.cards.get(name.as_str()).is_some_and(|c| c.land()) {
                3
            } else {
                2
            };
            worst.push((rank(s), order, name.to_string(), s));
        }
    }
    worst.sort();
    Counted { n: slots.len(), cnt, worst: worst.into_iter().map(|(_, _, n, s)| (n, s)).collect() }
}

fn counts_text(n: usize, cnt: &BTreeMap<&'static str, usize>) -> String {
    let parts: Vec<String> = STATUSES.iter().map(|s| format!("{s} {}", cnt.get(s).copied().unwrap_or(0))).collect();
    format!("{n} cards: {}", parts.join(", "))
}

fn names_text(worst: &[(String, &'static str)]) -> String {
    if worst.is_empty() {
        return String::new();
    }
    let mut by: Vec<String> = vec![];
    for s in ["Unmodeled", "Partial"] {
        let v: Vec<&str> = worst.iter().take(SHOWN).filter(|(_, x)| *x == s).map(|(n, _)| n.as_str()).collect();
        if !v.is_empty() {
            by.push(format!("{s}: {}", v.join(", ")));
        }
    }
    let more = worst.len().saturating_sub(SHOWN);
    format!(" | {}{}", by.join("; "), if more > 0 { format!(" (+{more} more)") } else { String::new() })
}

/// the block's lines (header first)
pub fn lines(entries: &[Entry]) -> Result<Vec<String>, String> {
    let root = crate::decks::root()?;
    let project = cardgen::data::Project::new(&root);
    let inp = cardgen::export::Inputs::load(&project)?;
    let built = inp.build();
    let au = Audit::new(&inp, &built.cards)?;
    let mut out = vec![format!(
        "Model coverage (cardgen audit statuses of the 99 + commander; names: Unmodeled, then Partial, commander and \
         engine cards first, at most {SHOWN}):"
    )];
    let w = entries
        .iter()
        .map(|e| match e {
            Entry::Deck(d) => d.label.chars().count(),
            Entry::Tier(t, _) => format!("opponents {t}").len(),
        })
        .max()
        .unwrap_or(0);
    for e in entries {
        match e {
            Entry::Deck(d) => {
                let c = count(&au, d);
                out.push(format!("  {:w$}  {}{}", d.label, counts_text(c.n, &c.cnt), names_text(&c.worst)));
            }
            Entry::Tier(t, ds) => {
                let mut cnt: BTreeMap<&'static str, usize> = BTreeMap::new();
                let mut n = 0;
                let mut most: Option<(usize, &str)> = None;
                for d in ds {
                    let c = count(&au, d);
                    n += c.n;
                    for (s, k) in &c.cnt {
                        *cnt.entry(s).or_default() += k;
                    }
                    let u = c.cnt.get("Unmodeled").copied().unwrap_or(0);
                    if most.is_none_or(|(m, _)| u > m) {
                        most = Some((u, d.label.as_str()));
                    }
                }
                let most = most.filter(|(u, _)| *u > 0).map_or(String::new(), |(u, l)| format!(" | most Unmodeled: {l} ({u})"));
                out.push(format!(
                    "  {:w$}  {} decks, {}{most}",
                    format!("opponents {t}"),
                    ds.len(),
                    counts_text(n, &cnt)
                ));
            }
        }
    }
    Ok(out)
}

/// print the block, or why it is missing (never fails the run)
pub fn print(entries: &[Entry]) {
    match lines(entries) {
        Ok(l) => println!("{}\n", l.join("\n")),
        Err(e) => println!("(model coverage unavailable: {e})\n"),
    }
}

/// each tier's own decks, a line per deck under a tier heading (--realism, --calibrate)
pub fn tier_blocks(sim: &Sim, tiers: &[&str]) -> Vec<Entry> {
    let mut out = vec![];
    for t in tiers {
        let keys = sim.pool_keys(t);
        for k in &keys {
            if let Some(mut d) = pool_list(sim, k) {
                d.label = format!("{t} {}", d.label);
                out.push(Entry::Deck(d));
            }
        }
    }
    out
}

/// what a pool-mode run plays (--deck, --all-decks, --calibrate, --analyze, --swap): your decks a line each (a
/// swapped list a second line), the opponent tiers a line each; --calibrate, every deck of its tiers
pub fn pool_run(sim: &Sim, a: &Args, swaps: &[(String, String)], base: &[String], var: &[String]) -> Result<Vec<Entry>, String> {
    if a.calibrate.is_some() {
        let tiers: Vec<&str> =
            TIERS.iter().copied().filter(|t| a.pool.as_deref().is_none_or(|p| p == "all" || p == *t)).collect();
        return Ok(tier_blocks(sim, &tiers));
    }
    let plan = crate::poolmode::tier_plan(sim, a)?;
    let mut out = vec![];
    for (d, _) in &plan.per_deck {
        let list = if a.deck.as_deref() == Some(d.as_str()) && !base.is_empty() { base.to_vec() } else { sim.mine[d].clone() };
        out.push(Entry::Deck(deck_list(sim, d, &list, None)));
        if !swaps.is_empty() && a.deck.as_deref() == Some(d.as_str()) {
            out.push(Entry::Deck(deck_list(sim, d, var, Some(format!("{d} (swapped)")))));
        }
    }
    for t in &plan.tiers {
        let ds = sim.pool_keys(t).iter().filter_map(|k| pool_list(sim, k)).collect();
        out.push(Entry::Tier(tier_label(t).to_string(), ds));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;
    use sim_core::settings::AiMode;

    #[test]
    fn the_names_are_capped_unmodeled_first() {
        let w: Vec<(String, &'static str)> = (0..7)
            .map(|i| (format!("C{i}"), if i < 2 { "Unmodeled" } else { "Partial" }))
            .collect();
        assert_eq!(names_text(&w), " | Unmodeled: C0, C1; Partial: C2, C3, C4 (+2 more)");
        assert_eq!(names_text(&[]), "");
    }

    #[test]
    fn every_deck_line_counts_its_99_and_commander() {
        let sim = Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap();
        let mut entries: Vec<Entry> =
            sim.mine.iter().map(|(k, l)| Entry::Deck(deck_list(&sim, k, l, None))).collect();
        entries.extend(tier_blocks(&sim, &["t3"]));
        let n = entries.len();
        let l = lines(&entries).unwrap();
        assert_eq!(l.len(), n + 1, "{l:#?}");
        for line in &l[1..] {
            assert!(line.contains(" 100 cards: Full "), "{line}");
            assert!(line.contains("Partial ") && line.contains("Unmodeled "), "{line}");
        }
        // an opponent tier: one summed line
        let ds = sim.pool_keys("t3").iter().filter_map(|k| pool_list(&sim, k)).collect::<Vec<_>>();
        let k = ds.len();
        let l = lines(&[Entry::Tier("t3".into(), ds)]).unwrap();
        assert!(l[1].contains(&format!("{k} decks, {} cards:", 100 * k)), "{}", l[1]);
    }
}
