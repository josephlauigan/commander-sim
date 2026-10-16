//! commander-sim: the run harness (Python's compare.py, poolmode.py, pools.py, decks.py and the experiment tools)
//! without Python. Measure one of your decks against the opponent pools (decklists/pool/), or compare a list
//! change with paired runs.
//!
//!     commander-sim --deck seph --pool t3 --games 1500
//!     commander-sim --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor"
//!     commander-sim --all-decks --pool all --profile loose
//!
//! Baseline = the deck list in decklists/<owner>/. Same seeds are used for baseline and variant, so the comparison
//! is paired.

mod args;
mod cardinfo;
mod decks;
mod explain;
mod play;
mod poolmode;
mod progress;
mod pyrand;
mod report;
mod stats;
mod tools;

use args::Args;
use play::{RunOpts, Sim};
use sim_core::settings::AiMode;
use std::process::ExitCode;

fn main() -> ExitCode {
    play::install_panic_hook();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let code = match argv.first().map(String::as_str) {
        Some("swaptest") => tools::swaptest(&argv[1..]),
        Some("searchtest") => tools::searchtest(&argv[1..]),
        Some("pools") => tools::pools(&argv[1..]),
        _ => compare(&argv),
    };
    ExitCode::from(code as u8)
}

pub fn ai_mode(name: &str) -> AiMode {
    if name == "adaptive" { AiMode::Adaptive } else { AiMode::Lookahead }
}

/// compare.cli: parse, run, then list any games that hit an error
fn compare(argv: &[String]) -> i32 {
    let a = match args::parse(argv) {
        Ok(a) => a,
        Err(e) => {
            if e.code == 0 {
                print!("{}", e.msg);
            } else {
                eprintln!("{}", e.msg);
            }
            return e.code;
        }
    };
    let opts = RunOpts { ai: ai_mode(&a.ai), temp: a.temp, jobs: a.jobs };
    let sim = match Sim::load(opts) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    let r = run(&sim, a);
    report_errors(&sim);
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

/// compare.parse_swaps: "Out Card=>In Card" pairs
fn parse_swaps(raw: &[String]) -> Result<Vec<(String, String)>, String> {
    raw.iter()
        .map(|s| match s.split_once("=>") {
            Some((a, b)) => Ok((a.trim().to_string(), b.trim().to_string())),
            None => Err(format!("swap must look like 'Out Card=>In Card': {s}")),
        })
        .collect()
}

/// compare.apply_swaps: validates the swaps (each card out must be in the list); the variant appends the new cards
fn apply_swaps(list: &[String], deck: &str, swaps: &[(String, String)]) -> Result<Vec<String>, String> {
    let mut d = list.to_vec();
    for (out, inn) in swaps {
        let i = d.iter().position(|c| c == out).ok_or_else(|| format!("'{out}' is not in the current {deck} list"))?;
        d.remove(i);
        d.push(inn.clone());
    }
    Ok(d)
}

/// poolmode.swap_in_place: each card swapped at its list position, so both versions of the deck get the same library
/// permutation from the same seed
fn swap_in_place(list: &[String], swaps: &[(String, String)]) -> Vec<String> {
    let mut out = list.to_vec();
    for (o, i) in swaps {
        if let Some(j) = out.iter().position(|c| c == o) {
            out[j] = i.clone();
        }
    }
    out
}

/// compare.main
fn run(sim: &Sim, mut a: Args) -> Result<(), String> {
    let mut swaps = vec![];
    let (mut base, mut var) = (vec![], vec![]);
    if let Some(deck) = a.deck.clone()
        && !a.all_decks
    {
        swaps = parse_swaps(&a.swap)?;
        let missing: Vec<&str> =
            swaps.iter().map(|(_, b)| b.as_str()).filter(|b| sim.raw_card(b).is_none()).collect();
        if !missing.is_empty() {
            return Err(format!(
                "Not in data/cards.json (check the spelling; a card no list has played yet needs a card-data export \
                 first): {}",
                missing.join(", ")
            ));
        }
        // compare.check_identity
        let ident = sim.engine.decks.get(&deck).and_then(|d| d.ident.clone()).unwrap_or_default();
        let bad: Vec<&str> = swaps
            .iter()
            .map(|(_, b)| b.as_str())
            .filter(|b| !cardinfo::identity(sim.raw_card(b).unwrap()).iter().all(|c| ident.contains(*c)))
            .collect();
        if !bad.is_empty() {
            return Err(format!("Outside {deck}'s color identity ({ident}): {}", bad.join(", ")));
        }
        // compare.show_auto
        let auto: Vec<_> =
            swaps.iter().filter_map(|(_, b)| sim.raw_card(b)).filter(|c| c.source != "manual").collect();
        if !auto.is_empty() {
            println!("Cards modeled from Scryfall text (check the abilities below):");
            for c in auto {
                println!("   {}", cardinfo::describe(c));
            }
        }
        if !a.no_explain && !swaps.is_empty() {
            let ins: Vec<String> = swaps.iter().map(|(_, b)| b.clone()).collect();
            match decks::root().and_then(|r| explain::model_status(&r, &ins)) {
                Ok(v) => println!("{}\n", explain::preflight_lines(&v).join("\n")),
                Err(e) => println!("(model status unavailable: {e})"),
            }
        }
        base = sim.mine[&deck].clone();
        var = apply_swaps(&base, &deck, &swaps)?;
    }

    if a.cards {
        let deck = a.deck.as_deref().ok_or("--cards needs --deck")?;
        print_cards(sim, deck, if swaps.is_empty() { &base } else { &var });
        return Ok(());
    }

    if a.pool.is_none() && a.calibrate.is_none() && !a.all_decks {
        return Err("--pool t1..t5|all is required (or --all-decks / --calibrate). The four-deck head-to-head mode was \
                    removed; decks are measured against the opponent pools."
            .into());
    }
    if a.all_decks && a.pool.is_none() {
        a.pool = Some("all".into());
    }
    if a.deck.is_none() && a.calibrate.is_none() && !a.all_decks {
        return Err("--deck is required (or use --all-decks)".into());
    }
    if !swaps.is_empty() {
        var = swap_in_place(&base, &swaps);
    }
    if a.trace.is_some() {
        return poolmode::trace(sim, &a, if swaps.is_empty() { &base } else { &var });
    }
    poolmode::main(sim, &a, &swaps, &base, &var)
}

/// --cards: every card in the deck with its tags, source and unmodeled text, and its Game Changers
fn print_cards(sim: &Sim, deck: &str, list: &[String]) {
    let mut names: Vec<&String> = list.iter().collect();
    names.sort();
    names.dedup();
    let auto = names.iter().filter(|n| sim.raw_card(n).is_none_or(|c| c.source != "manual")).count();
    println!(
        "{deck}: {} unique cards, {} hand-tagged, {auto} from Scryfall / ability data",
        names.len(),
        names.len() - auto
    );
    for n in &names {
        match sim.raw_card(n) {
            Some(c) => println!("  {}", cardinfo::describe(c)),
            None => println!("  {n} [not in data/cards.json]"),
        }
    }
    // cards.game_changers, from the exported Scryfall data
    let known: Vec<_> = names.iter().filter_map(|n| sim.raw_card(n)).filter(|c| c.game_changer.is_some()).collect();
    if (known.len() as f64) < names.len() as f64 * 0.9 {
        println!("\n(Game Changer check unavailable: data/cards.json has no Game Changer flag for every card)");
    } else {
        let gcs: Vec<&str> = known.iter().filter(|c| c.game_changer == Some(true)).map(|c| c.name.as_str()).collect();
        println!("\nGame Changers per Scryfall ({}): {}", gcs.len(), if gcs.is_empty() { "none".into() } else { gcs.join(", ") });
    }
}

/// compare.report_errors: games that panicked, left out of the results
fn report_errors(sim: &Sim) {
    let errs = sim.errors.lock().unwrap();
    if errs.is_empty() {
        return;
    }
    println!(
        "\nWARNING: {} game(s) hit an internal simulator error and were skipped (results use the rest).",
        errs.len()
    );
    println!(
        "  Replay one to see what happened:  commander-sim --deck <deck> [--swap ...] --pool <tier> --trace <seed - \
         first seed>"
    );
    for e in errs.iter().take(5) {
        let what: String = e.what.chars().take(200).collect();
        println!("   seed {}  {what}", e.seed);
    }
    println!("  Please send me that output so I can fix the cause.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: &[&str]) -> Vec<String> {
        x.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn swaps_in_place_keep_positions() {
        let list = v(&["A", "B", "C", "D"]);
        let swaps = parse_swaps(&v(&["B => X", "D=>Y"])).unwrap();
        assert_eq!(swaps, [("B".to_string(), "X".to_string()), ("D".to_string(), "Y".to_string())]);
        assert_eq!(swap_in_place(&list, &swaps), ["A", "X", "C", "Y"]);
        assert_eq!(apply_swaps(&list, "d", &swaps).unwrap(), ["A", "C", "X", "Y"]);
        assert!(apply_swaps(&list, "d", &[("Z".into(), "Q".into())]).is_err());
        assert!(parse_swaps(&v(&["B->X"])).is_err());
    }
}
