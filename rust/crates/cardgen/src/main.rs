use cardgen::data::Project;
use std::path::PathBuf;

const USAGE: &str = "usage:
  cardgen export [--out DIR] [--offline]        write data/cards.json and data/decks.json; refuses (writes
                                                nothing) a card that runs through two paths, see audit --paths
  cardgen audit [--deck KEY | --cards FILE | --mine | --md FILE | --hooks [--all]]
  cardgen audit --paths [--deck KEY] [--md FILE]
                                                how faithfully the cards are modeled; what is left to port;
                                                --paths: each card's paths into the sim and its doubles
  cardgen validate                              check every pool deck (size, singleton, colours, bans, Game Changers)
  cardgen deck KEY LIST|- [--dry-run] [--log TEXT] [--offline]
                                                replace a deck's list (KEY: your deck key, a pool deck key, or a path)
  cardgen bulk-sync [--from-file PATH] [--data-dir DIR] [--min-count N]
                                                download Scryfall's Oracle Cards bulk file and write
                                                data/knowledge/oracle.json (+ oracle.prev.json, sync.lock); prints
                                                \"up to date\" when the bulk file has not changed. --from-file
                                                parses a local bulk file (no network); the directory can also be
                                                set with CARDGEN_KNOWLEDGE_DIR
  cardgen find [--name N] [--name-contains S] [--ident WU] [--legal legal|banned|not_legal|any] [--cmc '<=2']
               [--type WORD]... [--text REGEX]... [--gc] [--deck KEY] [--meld]
               [--sort edhrec|name|cmc|usd] [--limit N] [--offset N] [--count] [--json] [--full] [--data-dir DIR]
                                                search the whole Commander catalog (data/knowledge/oracle.json, see
                                                bulk-sync); filters combine with AND. --name is exact (also a face
                                                name); --ident is a limit (WU = any card inside W and U, C or empty =
                                                colourless only); --legal defaults to legal; --type and --text repeat
                                                (all must match; --text is a case-insensitive regex over the Oracle
                                                text of every face); --deck KEY limits to that deck's colours and
                                                leaves out its cards; a meld result needs --meld; --sort defaults to
                                                edhrec, --limit to 30; --count prints only the number of matches
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
        "bulk-sync" => bulk_sync(&project, &mut args),
        "find" => find(&project, &args),
        _ => Err(USAGE.into()),
    }
}

fn audit(project: &Project, args: &mut Vec<String>) -> Result<(), String> {
    use cardgen::audit as a;
    let mut inp = cardgen::export::Inputs::load(project)?;
    let md = opt(args, "--md")?;
    let paths = flag(args, "--paths");
    if let Some(f) = &md {
        // before anything else: a refused path writes nothing
        if paths { a::check_md_target_for(f, cardgen::paths::MD_HEADER)? } else { a::check_md_target(f)? }
    }
    let deck = opt(args, "--deck")?;
    let cards_file = opt(args, "--cards")?;
    let fetch = flag(args, "--fetch");
    let mine = flag(args, "--mine");
    if let (Some(f), true) = (&cards_file, fetch) {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        let names: Vec<String> =
            text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(String::from).collect();
        let lost = cardgen::scryfall::fetch(&mut inp.cache, &names, true);
        inp.cache.save()?;
        if !lost.is_empty() {
            eprintln!("not found on Scryfall: {}", lost.join(", "));
        }
    }
    let built = inp.build();
    let hooks = flag(args, "--hooks");
    let all = flag(args, "--all");
    if hooks {
        print!("{}", a::hooks_report(&inp, &built.cards, all)?);
        return Ok(());
    }
    let au = a::Audit::new(&inp, &built.cards)?;
    if paths {
        let (db, reg) = a::engine_db(&inp)?;
        let all = cardgen::paths::all_paths(&db, &reg, &au, &inp, &cardgen::paths::paths_ok(&inp));
        let played = inp.wanted().played;
        let d = deck.as_deref().map(|k| cardgen::paths::deck_cards(&inp, k)).transpose()?;
        let lits = cardgen::paths::name_literals(&inp, &db)?;
        if let Some(f) = &md {
            let text = cardgen::paths::report(&all, &played, d.as_ref(), &lits, true);
            std::fs::write(f, text).map_err(|e| format!("{f}: {e}"))?;
        }
        print!("{}", cardgen::paths::report(&all, &played, d.as_ref(), &lits, false));
        return Ok(());
    }
    if let Some(f) = cards_file {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{f}: {e}"))?;
        for name in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (s, n) = au.card_status(name, true);
            println!("{s} | {name} | {n}");
        }
        return Ok(());
    }
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
            match &d.commander2 {
                Some(c2) => format!("{} + {c2}", d.commander),
                None => d.commander.clone(),
            },
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

fn find(project: &Project, args: &[String]) -> Result<(), String> {
    use cardgen::knowledge::FindError;
    use std::io::Write;
    match cardgen::knowledge::find(project, args) {
        // a closed pipe (`| head`) is not an error
        Ok(out) => {
            let _ = std::io::stdout().write_all(out.as_bytes());
            Ok(())
        }
        Err(FindError::Usage(m)) => Err(format!("{m}\n{USAGE}")),
        Err(FindError::Failed(m)) => Err(m),
    }
}

fn bulk_sync(project: &Project, args: &mut Vec<String>) -> Result<(), String> {
    use cardgen::bulk;
    let from_file = opt(args, "--from-file")?.map(PathBuf::from);
    let (dir, explicit_dir) = match opt(args, "--data-dir")? {
        Some(d) => (PathBuf::from(d), true),
        None => match std::env::var_os("CARDGEN_KNOWLEDGE_DIR") {
            Some(d) if !d.is_empty() => (PathBuf::from(d), true),
            _ => (project.data("knowledge"), false),
        },
    };
    let min_count = match opt(args, "--min-count")? {
        Some(n) => n.parse::<usize>().map_err(|_| format!("--min-count needs a number, got {n}"))?,
        None => bulk::MIN_COUNT,
    };
    if !args.is_empty() {
        return Err(USAGE.into());
    }
    if from_file.is_some() && !explicit_dir {
        eprintln!(
            "warning: --from-file writes bulk_updated_at=from-file into {}/sync.lock; pass --data-dir for a scratch \
             directory",
            dir.display()
        );
    }
    let opts = bulk::Options { dir, from_file, min_count, synced: bulk::now_utc() };
    print!("{}", bulk::run(&opts, &mut bulk::Http)?.text);
    Ok(())
}
