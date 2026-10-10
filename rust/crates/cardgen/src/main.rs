use cardgen::data::Project;
use std::path::PathBuf;

const USAGE: &str = "usage:
  cardgen export [--out DIR] [--offline]        write data/cards.json and data/decks.json
  cardgen audit [--deck KEY | --mine | --md FILE | --hooks [--all]]
                                                how faithfully the cards are modeled; what is left to port
  cardgen validate                              check every pool deck (size, singleton, colours, bans, Game Changers)
  cardgen deck KEY LIST|- [--dry-run] [--log TEXT] [--offline]
                                                replace a deck's list (KEY: your deck key, a pool deck key, or a path)
  (--root DIR names the repository; by default the one containing the current directory)";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("cardgen: {e}");
        std::process::exit(1);
    }
}

fn flag(args: &mut Vec<String>, name: &str) -> bool {
    match args.iter().position(|a| a == name) {
        Some(i) => {
            args.remove(i);
            true
        }
        None => false,
    }
}

fn opt(args: &mut Vec<String>, name: &str) -> Result<Option<String>, String> {
    match args.iter().position(|a| a == name) {
        Some(i) => {
            args.remove(i);
            if i < args.len() { Ok(Some(args.remove(i))) } else { Err(format!("{name} needs a value")) }
        }
        None => Ok(None),
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mut args = args.to_vec();
    let root = opt(&mut args, "--root")?;
    let project = match root {
        Some(r) => Project::new(r),
        None => Project::find()?,
    };
    let cmd = if args.is_empty() { String::new() } else { args.remove(0) };
    match cmd.as_str() {
        "export" => {
            let offline = flag(&mut args, "--offline");
            let out = opt(&mut args, "--out")?.map(PathBuf::from).unwrap_or_else(|| project.root.join("data"));
            let (c, d) = cardgen::export::run(&project, &out, offline)?;
            println!("{c} cards, {d} decks -> {}", out.display());
            Ok(())
        }
        "audit" => audit(&project, &mut args),
        "validate" => validate(&project, &args),
        "deck" => deck(&project, &mut args),
        _ => Err(USAGE.into()),
    }
}

fn audit(project: &Project, args: &mut Vec<String>) -> Result<(), String> {
    use cardgen::audit as a;
    let inp = cardgen::export::Inputs::load(project)?;
    let built = inp.build();
    let md = opt(args, "--md")?;
    let deck = opt(args, "--deck")?;
    let mine = flag(args, "--mine");
    let hooks = flag(args, "--hooks");
    let all = flag(args, "--all");
    if hooks {
        print!("{}", a::hooks_report(&inp, &built.cards, all)?);
        return Ok(());
    }
    let au = a::Audit::new(&inp, &built.cards)?;
    if mine {
        print!("{}", a::mine_report(&au));
        return Ok(());
    }
    let rows = a::audit_pool(&au, &inp.pool);
    if let Some(key) = deck {
        print!("{}", a::deck_report(&rows, &key));
        return Ok(());
    }
    if let Some(f) = md {
        std::fs::write(&f, a::markdown(&rows) + "\n").map_err(|e| format!("{f}: {e}"))?;
    }
    print!("{}", a::summary(&rows));
    Ok(())
}

fn validate(project: &Project, _args: &[String]) -> Result<(), String> {
    let cache = cardgen::scryfall::Cache::load(&project.data("scryfall_cache.json"))?;
    let report = cardgen::decks::validate_all(project, &cache)?;
    let mut bad = 0;
    for (d, probs, info) in &report {
        bad += !probs.is_empty() as usize;
        println!(
            "{} {} {:<40} {} [{}] GCs={}",
            if probs.is_empty() { "OK " } else { "ERR" },
            d.tier,
            d.key,
            d.commander,
            info.identity,
            info.gcs.len()
        );
        for p in probs {
            println!("      - {p}");
        }
    }
    let n = report.len();
    if bad > 0 {
        println!("\n{} of {n} decks passed", n - bad);
        std::process::exit(1);
    }
    println!("\nall decks passed");
    Ok(())
}

fn deck(project: &Project, args: &mut Vec<String>) -> Result<(), String> {
    let dry = flag(args, "--dry-run");
    let offline = flag(args, "--offline");
    let log = opt(args, "--log")?;
    if args.len() != 2 {
        return Err(USAGE.into());
    }
    let list = if args[1] == "-" {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).map_err(|e| e.to_string())?;
        s
    } else {
        std::fs::read_to_string(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?
    };
    let o = cardgen::update_deck::run(project, &args[0], &list, &cardgen::update_deck::Options { dry, log, offline })?;
    print!("{}", o.text);
    if !o.ok {
        std::process::exit(1);
    }
    Ok(())
}
