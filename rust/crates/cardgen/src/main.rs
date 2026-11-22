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
  cardgen bulk-sync [--from-file PATH [--updated-at STAMP]] [--data-dir DIR] [--min-count N]
                                                download Scryfall's Oracle Cards bulk file and write
                                                data/knowledge/oracle.json (+ oracle.prev.json, sync.lock, which
                                                is untracked); prints \"up to date\" when the bulk file has not
                                                changed. --from-file parses a local bulk file (no network, no
                                                up-to-date check); with --updated-at the file stands in for the
                                                remote at that bulk date (up-to-date check, no network); the
                                                directory can also be set with CARDGEN_KNOWLEDGE_DIR
  cardgen roles --rules [--data-dir DIR]        classify the catalog and write roles.json (offline)
  cardgen roles --llm [--limit N] [--ids FILE] [--only-routed] [--batch 10] [--dry-run] [--endpoint URL] [--model ID]
                       [--pending] [--data-dir DIR]
                                                classify routed cards through the local LLM server (llama.cpp,
                                                http://127.0.0.1:8080/v1) into llm_roles.jsonl; needs `llama up`
  cardgen roles --merge [--model ID] [--pending] [--data-dir DIR]
                                                rules + LLM cache + overrides -> roles.json, reports/ (offline;
                                                --pending also reads llm_roles.pending.jsonl)
  cardgen roles --validate [--gold PATH] [--json] [--stage rules|merged] [--data-dir DIR]
  cardgen roles --audit [--seed N] [--n 80] [--rule ID] [--certain] [--data-dir DIR]
  cardgen roles --show NAME | --stats | --lint [--data-dir DIR]
  cardgen find [--name N] [--name-contains S] [--ident WU] [--legal legal|banned|not_legal|any] [--cmc '<=2']
               [--type WORD]... [--text REGEX]... [--role R]... [--gc] [--deck KEY] [--meld]
               [--status LIST] [--in-deck KEY] [--not-in-deck KEY]
               [--sort edhrec|name|cmc|usd] [--limit N|all] [--offset N] [--count] [--json] [--full] [--data-dir DIR]
                                                search the whole Commander catalog (data/knowledge/oracle.json, see
                                                bulk-sync); filters combine with AND. --name is exact (also a face
                                                name, accents and apostrophe style ignored); --ident is a limit
                                                (WU = any card inside W and U, C or empty = colourless only);
                                                --legal defaults to legal; --type and --text repeat (all must match;
                                                --text is a case-insensitive regex over the Oracle text of every
                                                face); --deck KEY limits to that deck's colours (--ident may
                                                only narrow them) and leaves out its cards; a meld result needs
                                                --meld unless --name is given; --sort defaults to edhrec, --limit to
                                                30 (all = no limit); --count prints only the number of matches;
                                                notes about a cut page or a filtered-out name go to stderr
                                                --json rows and --full also show the engine model status, the decks
                                                of mine that play the card and the FINDINGS.md lines that name it;
                                                --status LIST (full, full-auto, approximate, partial, unmodeled,
                                                none = no engine record) and --in-deck / --not-in-deck KEY filter
                                                on them
  cardgen weekly-report [--date YYYY-MM-DD] [--commit TEXT] [--stages FILE] [--roles-before FILE] [--llm-log FILE]
                        [--out PATH] [--keep N] [--stdout] [--data-dir DIR]
                                                the weekly job's report (reports/DATE.md): action needed for the decks
                                                in data/decks.json, the run, new cards, ban and Game Changer changes
                                                (oracle.prev.json against oracle.json), role changes; offline,
                                                writes only the report
  cardgen map [--out PATH] [--stdout] [--data-dir DIR]
                                                write the whole-catalog summary (about 3k tokens, under 32 KB) to
                                                data/knowledge/map.md; needs oracle.json and roles.json; --stdout
                                                also prints it
  cardgen digest (--deck KEY | --ident LETTERS) [--role R | --view roleless|new] [--limit N] [--offset N]
                 [--out PATH] [--budget-tokens N] [--stdout] [--data-dir DIR]
                                                one compact line per card for a deck's colours (or any identity),
                                                always written to data/knowledge/digests/KEY.txt (a page to
                                                KEY.ROLE.txt or KEY.VIEW.txt): sections A the deck's cards (the
                                                commander first), B best per role, C synergy with the commander's
                                                subtypes, D new since the previous sync, E the best-ranked cards
                                                with no role; --role R pages through every card that has R in
                                                any position, --view roleless|new through the complete lists;
                                                --budget-tokens defaults to 42000 (estimated as bytes/4);
                                                --stdout also prints the text
  cardgen ownership --check [--file PATH] [--data-dir DIR] [--json] [--strict]
                                                validate data/ownership.toml (JD's record of which cards of his
                                                decks he has, plays as a proxy or is missing) against the deck
                                                lists of data/decks.json and the catalog; prints every problem
                                                with its deck and card, exits 1 on any ERROR; without an
                                                oracle.json names are compared by spelling (--strict: an error)
  (--root DIR names the repository; by default the one containing the current directory)";

fn main() {
    let mut args: Vec<String> = vec![];
    for (n, arg) in std::env::args_os().skip(1).enumerate() {
        match arg.into_string() {
            Ok(a) => args.push(a),
            Err(bad) => {
                eprintln!("cardgen: argument {} is not valid Unicode ({bad:?})\n{USAGE}", n + 1);
                std::process::exit(1);
            }
        }
    }
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
        "map" => map(&project, &args),
        "digest" => digest(&project, &args),
        "weekly-report" => weekly_report(&project, &args),
        "ownership" => {
            let (out, ok) = cardgen::ownership::command(&project, &args)?;
            print_text(&out)?;
            if !ok {
                std::process::exit(1);
            }
            Ok(())
        }
        "roles" => {
            let (out, passed) = cardgen::roles::command(&project, &args)?;
            print!("{out}");
            if passed { Ok(()) } else { Err("role validation thresholds missed".into()) }
        }
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
    let o = match cardgen::knowledge::find_with_notes(project, args) {
        Ok(o) => o,
        Err(FindError::Usage(m)) => return Err(format!("{m}\n{USAGE}")),
        Err(FindError::Failed(m)) => return Err(m),
    };
    let wrote = std::io::stdout().write_all(o.stdout().as_bytes()).and_then(|_| std::io::stdout().flush());
    match wrote {
        Ok(()) => {}
        // a closed pipe (`| head`) is not an error
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => return Ok(()),
        Err(e) => return Err(format!("cannot write the result: {e}")),
    }
    let _ = std::io::stderr().write_all(o.stderr().as_bytes());
    Ok(())
}

/// the text of a generated view to stdout (a closed pipe is not an error)
fn print_text(text: &str) -> Result<(), String> {
    use std::io::Write;
    match std::io::stdout().write_all(text.as_bytes()).and_then(|_| std::io::stdout().flush()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(format!("cannot write the result: {e}")),
    }
}

fn map(project: &Project, args: &[String]) -> Result<(), String> {
    let (o, stdout) = cardgen::map::command(project, args)?;
    if stdout {
        print_text(&o.text)?;
        eprintln!("{}", o.summary());
    } else {
        println!("{}", o.summary());
    }
    Ok(())
}

fn digest(project: &Project, args: &[String]) -> Result<(), String> {
    use cardgen::knowledge::FindError;
    let wants_text = args.iter().any(|a| a == "--stdout");
    let o = match cardgen::digest::command(project, args) {
        Ok(o) => o,
        Err(FindError::Usage(m)) => return Err(format!("{m}\n{}", cardgen::digest::USAGE)),
        Err(FindError::Failed(m)) => return Err(m),
    };
    if wants_text {
        print_text(&o.digest.text)?;
        eprintln!("{}", o.summary());
    } else {
        println!("{}", o.summary());
    }
    Ok(())
}

fn bulk_sync(project: &Project, args: &mut Vec<String>) -> Result<(), String> {
    use cardgen::bulk;
    let from_file = opt(args, "--from-file")?.map(PathBuf::from);
    let updated_at = opt(args, "--updated-at")?;
    if updated_at.is_some() && from_file.is_none() {
        return Err("--updated-at needs --from-file".into());
    }
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
    if from_file.is_some() && updated_at.is_none() && !explicit_dir {
        eprintln!(
            "warning: --from-file writes bulk_updated_at=from-file into {}/sync.lock; pass --data-dir for a scratch \
             directory",
            dir.display()
        );
    }
    let synced = bulk::now_utc();
    let out = match (from_file, updated_at) {
        (Some(path), Some(updated_at)) => {
            let opts = bulk::Options { dir, from_file: None, min_count, synced };
            bulk::run(&opts, &mut bulk::LocalRemote { updated_at, path })?
        }
        (from_file, _) => bulk::run(&bulk::Options { dir, from_file, min_count, synced }, &mut bulk::Http)?,
    };
    print!("{}", out.text);
    Ok(())
}

fn weekly_report(project: &Project, args: &[String]) -> Result<(), String> {
    let o = cardgen::weekly::command(project, args)?;
    if o.stdout {
        print_text(&o.text)?;
    }
    println!("{}", o.summary());
    Ok(())
}
