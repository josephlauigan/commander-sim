//! The command line, parsed the way compare.py's argparse parser read it: `--opt value` or `--opt=value`, a unique
//! prefix of an option's name also works (`--gam 960`), --swap repeats, and a bad value is an error naming the
//! option (exit code 2).

pub const DECK_KEYS: [&str; 18] = crate::decks::MINE;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub deck: Option<String>,
    pub swap: Vec<String>,
    pub pool: Option<String>,
    pub all_decks: bool,
    pub calibrate: Option<String>,
    pub games: u64,
    pub seed: u64,
    pub profiles: Vec<String>,
    pub analyze: bool,
    pub trace: Option<u64>,
    pub cards: bool,
    pub brief: bool,
    pub ai: String,
    pub temp: f64,
    pub quiet: bool,
    pub jobs: usize,
}

struct Opt {
    name: &'static str,
    flag: bool,
    choices: &'static [&'static str],
    metavar: &'static str,
    help: &'static str,
}

const fn opt(name: &'static str, flag: bool, choices: &'static [&'static str], metavar: &'static str, help: &'static str) -> Opt {
    Opt { name, flag, choices, metavar, help }
}

const OPTS: [Opt; 20] = [
    opt("--deck", false, &DECK_KEYS, "", "the deck to measure"),
    opt("--swap", false, &[], "\"OUT=>IN\"", "\"Card Out=>Card In\" (repeat for several): paired A/B run"),
    opt(
        "--pool",
        false,
        &["t1", "t2", "t3", "t4", "t5", "all"],
        "",
        "play --deck against three outside decks drawn from this tier (see decklists/pool/)",
    ),
    opt("--all-decks", true, &[], "", "run each of your decks (deck x tier matrix)"),
    opt("--calibrate", false, &["within", "ordering", "all"], "", "pool balance checks (no --deck needed)"),
    opt("--games", false, &[], "N", "games per run (default 1500)"),
    opt("--n", false, &[], "N", "same as --games"),
    opt("--seed", false, &[], "SEED", "first seed (games use seed .. seed+n-1; default 500000)"),
    opt("--profiles", false, &[], "LIST", "AI interaction profiles to run (default conservative,loose)"),
    opt("--profile", false, &[], "NAME", "run one profile only (conservative or loose)"),
    opt("--analyze", true, &[], "", "deep report: how the deck wins and loses, plan timing, card report"),
    opt("--trace", false, &[], "GAME", "print a play-by-play log of one game (seed offset) and exit"),
    opt("--cards", true, &[], "", "list every card in --deck with its tags, source and unmodeled text"),
    opt("--brief", true, &[], "", "skip the per-axis breakdown"),
    opt(
        "--ai",
        false,
        &["lookahead", "adaptive"],
        "",
        "AI decision model: lookahead (default; the adaptive AI plus look-ahead search) or adaptive (heuristic only, \
         about 60x faster)",
    ),
    opt("--temp", false, &[], "T", "adaptive AI randomness multiplier (lower = sharper play; default 1.0)"),
    opt("--quiet", true, &[], "", "no progress bar"),
    opt("--jobs", false, &[], "N", "parallel worker threads (default: every CPU core)"),
    opt("--engine", false, &["rust"], "", "accepted for old command lines; the games are always played in Rust"),
    opt("--dsl-all", true, &[], "", "not available: it needs the Python card compiler (see the README)"),
];

pub fn usage() -> &'static str {
    "usage: commander-sim [-h] [--deck KEY] [--swap \"OUT=>IN\"] [--pool TIER] [--all-decks] [--calibrate WHICH]\n\
     \x20                    [--games N] [--seed SEED] [--profiles LIST | --profile NAME] [--analyze] [--trace GAME]\n\
     \x20                    [--cards] [--brief] [--ai {lookahead,adaptive}] [--temp T] [--quiet] [--jobs N]\n\
     \x20      commander-sim swaptest | searchtest | pools ...   (commander-sim swaptest --help)"
}

pub fn help() -> String {
    let mut s = String::new();
    s += usage();
    s += "\n\nCommander pod simulator: one of your decks against three outside decks drawn from a tier of \
          decklists/pool/ (pool mode)\n\noptions:\n  -h, --help            show this help and exit\n";
    for o in &OPTS {
        let mut left = o.name.to_string();
        if !o.flag {
            left += " ";
            left += &if o.choices.is_empty() { o.metavar.to_string() } else { format!("{{{}}}", o.choices.join(",")) };
        }
        if left.len() <= 20 {
            s += &format!("  {left:20}  {}\n", o.help);
        } else {
            s += &format!("  {left}\n  {:20}  {}\n", "", o.help);
        }
    }
    s += "\nExamples\n  commander-sim --deck sauron --pool t1 --profile loose --games 960\n  commander-sim --deck seph \
          --pool t3 --swap \"Blood Artist=>Grim Tutor\"\n  commander-sim --all-decks --pool all --profile loose\n";
    s
}

/// a command-line error: the message and the exit code (2 for a usage error, as argparse; 0 for --help)
#[derive(Debug, PartialEq)]
pub struct Exit {
    pub code: i32,
    pub msg: String,
}

fn usage_err(msg: String) -> Exit {
    Exit { code: 2, msg: format!("{}\ncommander-sim: error: {msg}", usage()) }
}

fn resolve(name: &str) -> Result<&'static Opt, Exit> {
    if let Some(o) = OPTS.iter().find(|o| o.name == name) {
        return Ok(o);
    }
    let hits: Vec<&Opt> = OPTS.iter().filter(|o| o.name.starts_with(name)).collect();
    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(usage_err(format!("unrecognized arguments: {name}"))),
        _ => Err(usage_err(format!(
            "ambiguous option: {name} could match {}",
            hits.iter().map(|o| o.name).collect::<Vec<_>>().join(", ")
        ))),
    }
}

fn int<T: std::str::FromStr>(name: &str, v: &str) -> Result<T, Exit> {
    v.parse().map_err(|_| usage_err(format!("argument {name}: invalid int value: '{v}'")))
}

pub fn parse(argv: &[String]) -> Result<Args, Exit> {
    let mut a = Args {
        deck: None,
        swap: vec![],
        pool: None,
        all_decks: false,
        calibrate: None,
        games: 0,
        seed: 500000,
        profiles: vec![],
        analyze: false,
        trace: None,
        cards: false,
        brief: false,
        ai: "lookahead".into(),
        temp: 1.0,
        quiet: false,
        jobs: 0,
    };
    let (mut games, mut n, mut profiles, mut profile, mut jobs) = (None, None, None, None, None);
    let mut i = 0;
    while i < argv.len() {
        let tok = &argv[i];
        i += 1;
        if tok == "-h" || tok == "--help" {
            return Err(Exit { code: 0, msg: help() });
        }
        if !tok.starts_with("--") {
            return Err(usage_err(format!("unrecognized arguments: {tok}")));
        }
        let (name, inline) = match tok.split_once('=') {
            Some((n, v)) => (n, Some(v.to_string())),
            None => (tok.as_str(), None),
        };
        let o = resolve(name)?;
        let val = if o.flag {
            if inline.is_some() {
                return Err(usage_err(format!("argument {}: ignored explicit argument '{}'", o.name, inline.unwrap())));
            }
            String::new()
        } else if let Some(v) = inline {
            v
        } else if i < argv.len() {
            i += 1;
            argv[i - 1].clone()
        } else {
            return Err(usage_err(format!("argument {}: expected one argument", o.name)));
        };
        if !o.choices.is_empty() && !o.choices.contains(&val.as_str()) {
            let choices = o.choices.iter().map(|c| format!("'{c}'")).collect::<Vec<_>>().join(", ");
            return Err(usage_err(format!("argument {}: invalid choice: '{val}' (choose from {choices})", o.name)));
        }
        match o.name {
            "--deck" => a.deck = Some(val),
            "--swap" => a.swap.push(val),
            "--pool" => a.pool = Some(val),
            "--all-decks" => a.all_decks = true,
            "--calibrate" => a.calibrate = Some(val),
            "--games" => games = Some(int::<u64>(o.name, &val)?),
            "--n" => n = Some(int::<u64>(o.name, &val)?),
            "--seed" => a.seed = int(o.name, &val)?,
            "--profiles" => profiles = Some(val),
            "--profile" => profile = Some(val),
            "--analyze" => a.analyze = true,
            "--trace" => a.trace = Some(int(o.name, &val)?),
            "--cards" => a.cards = true,
            "--brief" => a.brief = true,
            "--ai" => a.ai = val,
            "--temp" => {
                a.temp = val.parse().map_err(|_| usage_err(format!("argument --temp: invalid float value: '{val}'")))?
            }
            "--quiet" => a.quiet = true,
            "--jobs" => jobs = Some(int::<usize>(o.name, &val)?),
            "--engine" => {}
            "--dsl-all" => {
                return Err(Exit {
                    code: 1,
                    msg: "--dsl-all is not available: it ran every card through the Python card compiler (Oracle text \
                          -> abilities), which the Rust program doesn't have. The games use the abilities in \
                          data/cards.json."
                        .into(),
                });
            }
            _ => unreachable!(),
        }
    }
    // a.games = a.games or a.n or 1500 (0 counts as unset, as in Python)
    a.games = games.filter(|&g| g != 0).or(n.filter(|&g| g != 0)).unwrap_or(1500);
    let profs = match profile {
        Some(p) => vec![p],
        None => profiles.unwrap_or_else(|| "conservative,loose".into()).split(',').map(|s| s.to_string()).collect(),
    };
    a.profiles = profs;
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    a.jobs = jobs.filter(|&j| j != 0).unwrap_or(cores).max(1);
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Result<Args, Exit> {
        parse(&s.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn defaults_and_forms() {
        let a = p("--deck sauron --pool=t1 --profile loose --gam 960 --jobs 22").unwrap();
        assert_eq!(a.deck.as_deref(), Some("sauron"));
        assert_eq!(a.pool.as_deref(), Some("t1"));
        assert_eq!(a.profiles, ["loose"]);
        assert_eq!((a.games, a.seed, a.jobs, a.ai.as_str(), a.temp), (960, 500000, 22, "lookahead", 1.0));
        let a = p("--deck seph --pool t3 --n 200").unwrap();
        assert_eq!(a.games, 200);
        assert_eq!(a.profiles, ["conservative", "loose"]);
        let a = p("--calibrate within").unwrap();
        assert_eq!(a.games, 1500);
    }

    #[test]
    fn swaps_repeat_and_values_may_contain_spaces() {
        let argv: Vec<String> =
            ["--deck", "sauron", "--swap", "Spell Pierce=>Negate", "--swap", "A=>B"].iter().map(|s| s.to_string()).collect();
        assert_eq!(parse(&argv).unwrap().swap, ["Spell Pierce=>Negate", "A=>B"]);
    }

    #[test]
    fn errors() {
        assert_eq!(p("--pool t9").unwrap_err().code, 2);
        assert_eq!(p("--deck najeela").unwrap_err().code, 2);
        assert_eq!(p("--ai rigid").unwrap_err().code, 2);
        assert_eq!(p("--games x").unwrap_err().code, 2);
        assert_eq!(p("--bogus").unwrap_err().code, 2);
        assert_eq!(p("--games").unwrap_err().code, 2);
        assert_eq!(p("--dsl-all").unwrap_err().code, 1);
        assert_eq!(p("--help").unwrap_err().code, 0);
        // --prof is ambiguous: --profiles, --profile
        assert!(p("--prof loose").unwrap_err().msg.contains("ambiguous"));
    }
}
