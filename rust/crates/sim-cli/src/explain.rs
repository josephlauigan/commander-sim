//! The paired breakdown printed after every `--swap` run (`--no-explain` turns it off): the model status of each
//! swapped-in card before the run, and after it a side-by-side look at what the swapped cards did and how the game
//! changed shape. It reads the analysis both lists collected during the paired run, so it costs no extra games and
//! never touches the RNG.

use crate::report::{Analysis, plan_text};
use std::path::Path;

/// One card's line in the audit: (card, status, reason)
pub type Verdict = (String, String, String);

/// `cardgen audit`'s verdict for each card, as one of your decks plays it
pub fn model_status(root: &Path, cards: &[String]) -> Result<Vec<Verdict>, String> {
    let project = cardgen::data::Project::new(root);
    let inp = cardgen::export::Inputs::load(&project)?;
    let built = inp.build();
    let au = cardgen::audit::Audit::new(&inp, &built.cards)?;
    Ok(cards
        .iter()
        .map(|c| {
            let (s, n) = au.card_status(c, true);
            (c.clone(), s, n)
        })
        .collect())
}

/// the pre-flight lines for the swapped-in cards; Partial and Unmodeled get a loud WARNING
pub fn preflight_lines(v: &[Verdict]) -> Vec<String> {
    let mut out = vec!["Model status of the swapped-in cards (cardgen audit):".to_string()];
    for (c, s, n) in v {
        let warn = matches!(s.as_str(), "Partial" | "Unmodeled");
        out.push(format!("  {}{c}: {s} ({n})", if warn { "WARNING " } else { "" }));
        if warn {
            out.push(format!(
                "  WARNING {c} is {s}: the sim plays it differently from the real card; read the result with that in mind"
            ));
        }
    }
    out
}

fn pct(a: i64, b: i64) -> f64 {
    if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 }
}

fn avg(a: &Analysis, k: &str) -> Option<f64> {
    a.snap.get(k).filter(|x| x.1 > 0).map(|x| x.0 as f64 / x.1 as f64)
}

/// one card's numbers in one list's analysis
struct CardRow {
    drawn: f64,
    cast_of_drawn: Option<f64>,
    round: Option<f64>,
    stuck: Option<f64>,
    win_cast: Option<f64>,
    lost: f64,
}

fn card_row(a: &Analysis, name: &str) -> Option<CardRow> {
    let key = name.to_string();
    let seen = a.seen.get(&key);
    let cast = a.cast.get(&key);
    if seen == 0 && cast == 0 {
        return None;
    }
    let drawn_base = seen.max(cast);
    Some(CardRow {
        drawn: pct(seen, a.n),
        cast_of_drawn: Some(pct(cast, drawn_base)),
        round: (cast > 0).then(|| a.cast_round.get(&key) as f64 / cast as f64),
        stuck: Some(pct(drawn_base - cast, drawn_base)),
        win_cast: (cast > 0).then(|| pct(a.win_cast.get(&key), cast)),
        lost: a.lost.get(&key) as f64 / a.n.max(1) as f64,
    })
}

fn opt(x: Option<f64>, w: usize, d: usize) -> String {
    match x {
        Some(v) => format!("{v:>w$.d$}"),
        None => format!("{:>w$}", "-"),
    }
}

fn card_table(title: &str, names: &[String], a: &Analysis) {
    println!("  {title}");
    println!(
        "    {:34} {:>7} {:>9} {:>11} {:>8} {:>10} {:>9}",
        "card", "drawn%", "cast%*", "avg round", "stuck%", "win%cast", "lost/game"
    );
    for n in names {
        let t: String = n.chars().take(34).collect();
        match card_row(a, n) {
            Some(r) => println!(
                "    {t:34} {:>7.1} {} {} {} {} {:>9.2}",
                r.drawn,
                opt(r.cast_of_drawn, 9, 1),
                opt(r.round, 11, 1),
                opt(r.stuck, 8, 1),
                opt(r.win_cast, 10, 1),
                r.lost
            ),
            None => println!("    {t:34} never drawn in this list's games"),
        }
    }
}

fn row(label: &str, b: Option<f64>, v: Option<f64>, d: usize) {
    let f = |x: Option<f64>| match x {
        Some(x) => format!("{x:>9.d$}"),
        None => format!("{:>9}", "-"),
    };
    let diff = match (b, v) {
        (Some(b), Some(v)) => format!("{:>+8.d$}", v - b),
        _ => format!("{:>8}", ""),
    };
    println!("    {label:44} {} {} {diff}", f(b), f(v));
}

/// by-turn cumulative share of games with the plan online / the commander out
fn by_turn(c: &crate::stats::Counter<u32>, t: u32, n: i64) -> f64 {
    (2..=t).map(|x| c.get(&x)).sum::<i64>() as f64 * 100.0 / n.max(1) as f64
}

fn shares(c: &crate::stats::Counter<String>, total: i64, take: usize) -> String {
    c.most_common()
        .into_iter()
        .take(take)
        .map(|(k, v)| format!("{k} {:.0}%", pct(v, total)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// the breakdown. `base` is the current list's analysis, `var` the swapped one's; both were played on the same seeds.
pub fn print_explain(
    deck: &str,
    commanders: &[String],
    swaps: &[(String, String)],
    base: &Analysis,
    var: &Analysis,
    profile: &str,
    name_of: &dyn Fn(&str) -> String,
) {
    println!("\n-- Why: paired breakdown [{profile} profile, {} games per list, same seeds] --", base.n);
    let outs: Vec<String> = swaps.iter().map(|s| s.0.clone()).collect();
    let ins: Vec<String> = swaps.iter().map(|s| s.1.clone()).collect();
    card_table("Swapped IN (from the swapped list)", &ins, var);
    card_table("Swapped OUT (from the current list, what they were doing)", &outs, base);
    println!("    (*cast% = share of the games the card was drawn in; avg round = own-turn count of the first cast;");
    println!("     lost/game = times opponents removed it. A land is never 'cast'. Correlation, not causation.)");

    println!("\n  Game shape{:36} {:>9} {:>9} {:>8}", "", "current", "swapped", "change");
    let (bn, vn) = (base.n.max(1) as f64, var.n.max(1) as f64);
    row("win rate %", Some(pct(base.win, base.n)), Some(pct(var.win, var.n)), 1);
    row("rounds survived", Some(base.survived as f64 / bn), Some(var.survived as f64 / vn), 2);
    row("eliminated, % of games", Some(pct(base.death.total(), base.n)), Some(pct(var.death.total(), var.n)), 1);
    row(
        "opponents it eliminated, per game",
        Some(base.kills.total() as f64 / bn),
        Some(var.kills.total() as f64 / vn),
        2,
    );
    row("damage dealt, per game", Some(base.dmg.total() as f64 / bn), Some(var.dmg.total() as f64 / vn), 1);
    if !plan_text(deck).is_empty() {
        println!("    plan: {}", plan_text(deck));
        for t in [4u32, 6, 8, 12] {
            row(
                &format!("  plan online by turn {t}, %"),
                Some(by_turn(&base.plan, t, base.n)),
                Some(by_turn(&var.plan, t, var.n)),
                1,
            );
        }
        row("  plan online at all, %", Some(pct(base.plan.total(), base.n)), Some(pct(var.plan.total(), var.n)), 1);
    }
    for t in [4u32, 6] {
        row(
            &format!("commander resolved by turn {t}, %"),
            Some(by_turn(&base.cmd, t, base.n)),
            Some(by_turn(&var.cmd, t, var.n)),
            1,
        );
    }
    let removed = |a: &Analysis| commanders.iter().map(|c| a.lost.get(c)).sum::<i64>() as f64 / a.n.max(1) as f64;
    row("commander removals, per game", Some(removed(base)), Some(removed(var)), 2);
    for (label, k) in [
        ("lands on the battlefield, turn 4", "lands_T4"),
        ("lands on the battlefield, turn 8", "lands_T8"),
        ("mana available, turn 4", "mana_T4"),
        ("mana available, turn 8", "mana_T8"),
        ("cards in hand after the land drop, turn 4", "hand_T4"),
        ("cards in hand after the land drop, turn 8", "hand_T8"),
    ] {
        row(label, avg(base, k), avg(var, k), 2);
    }
    println!("    wins by (current):  {}", shares(&base.wintype, base.win, 4));
    println!("    wins by (swapped):  {}", shares(&var.wintype, var.win, 4));
    let deaths = |a: &Analysis| {
        a.death
            .most_common()
            .into_iter()
            .take(4)
            .map(|((who, how), v)| {
                let who = if crate::decks::MINE.contains(&who.as_str()) || who == "self" { who } else { name_of(&who) };
                format!("{who} via {how} {:.1}%", pct(v, a.n))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("    eliminated by (current): {}", deaths(base));
    println!("    eliminated by (swapped): {}", deaths(var));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_warns_on_partial_and_unmodeled_only() {
        let v = vec![
            ("A".to_string(), "Full".to_string(), "hand-verified".to_string()),
            ("B".to_string(), "Partial".to_string(), "no cost reduction".to_string()),
            ("C".to_string(), "Unmodeled".to_string(), "does nothing".to_string()),
            ("D".to_string(), "Approximate".to_string(), "regex tags".to_string()),
        ];
        let lines = preflight_lines(&v);
        let warns: Vec<&String> = lines.iter().filter(|l| l.contains("WARNING")).collect();
        assert_eq!(warns.len(), 4, "{lines:?}"); // two lines for each of B and C
        assert!(!lines.iter().any(|l| l.contains("WARNING A") || l.contains("WARNING D")));
        assert!(lines[0].contains("Model status"));
    }

    #[test]
    fn the_breakdown_names_the_swapped_cards_and_their_numbers() {
        let mut base = Analysis::default();
        let mut var = Analysis::default();
        for (a, card) in [(&mut base, "Mind Stone"), (&mut var, "Welcoming Vampire")] {
            a.n = 10;
            a.win = 3;
            a.seen.add(card.to_string(), 8);
            a.cast.add(card.to_string(), 6);
            a.cast_round.add(card.to_string(), 18);
            a.win_cast.add(card.to_string(), 3);
        }
        let swaps = vec![("Mind Stone".to_string(), "Welcoming Vampire".to_string())];
        let ins = card_row(&var, "Welcoming Vampire").unwrap();
        assert!((ins.drawn - 80.0).abs() < 1e-9);
        assert!((ins.cast_of_drawn.unwrap() - 75.0).abs() < 1e-9);
        assert!((ins.round.unwrap() - 3.0).abs() < 1e-9);
        assert!((ins.stuck.unwrap() - 25.0).abs() < 1e-9);
        assert!(card_row(&base, "Welcoming Vampire").is_none());
        // the printer must not panic on a sparse analysis
        print_explain("cloud", &["Cloud".to_string()], &swaps, &base, &var, "loose", &|k| k.to_string());
    }
}
