//! The command line, parsed the way compare.py's argparse parser read it: `--opt value` or `--opt=value`, a unique
//! prefix of an option's name also works (`--gam 960`), --swap repeats, and a bad value is an error naming the
//! option (exit code 2).

pub const DECK_KEYS: [&str; 14] = crate::decks::MINE;

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub deck: Option<String>,
    pub swap: Vec<String>,
    pub pool: Option<String>,
    pub all_decks: bool,
    pub calibrate: Option<String>,
    pub realism: bool,
    pub knobs: Option<String>,
    pub fit: bool,
    pub fit_scope: String,
    pub fit_gens: u32,
    pub fit_out: Option<String>,
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
    pub any_tier: bool,
    pub no_explain: bool,
    pub no_preflight: bool,
    pub preflight_only: bool,
}

struct Opt {
    name: &'static str,
    flag: bool,
    choices: &'static [&'static str],
    metavar: &'static str,
    help: &'static str,
}

const fn opt(
    name: &'static str,
    flag: bool,
    choices: &'static [&'static str],
    metavar: &'static str,
    help: &'static str,
) -> Opt {
    Opt { name, flag, choices, metavar, help }
}

const OPTS: [Opt; 30] = [
    opt("--deck", false, &DECK_KEYS, "", "the deck to measure"),
    opt("--swap", false, &[], "\"OUT=>IN\"", "\"Card Out=>Card In\" (repeat for several): paired A/B run"),
    opt(
        "--pool",
        false,
        &["t0", "t1", "t2", "t3", "t4", "t5", "t6", "all"],
        "",
        "play --deck against three outside decks drawn from this tier (see decklists/pool/); all = the deck's own tiers 
         (data/my_decks.toml: e.g. sephfs t1-t3, yshtola t4-t5). A tier outside them is refused unless --any-tier",
    ),
    opt("--all-decks", true, &[], "", "run each of your decks, each only against its own tiers (deck x tier matrix)"),
    opt("--calibrate", false, &["within", "ordering", "all"], "", "pool balance checks (no --deck needed)"),
    opt(
        "--realism",
        true,
        &[],
        "",
        "with --pool tN: play the tier's own decks and score the games against data/realism/tN.toml (no --deck needed)",
    ),
    opt(
        "--fit",
        true,
        &[],
        "",
        "with --pool tN: search the AI knobs (sep-CMA-ES, same seeds for every candidate) for the values closest to \
         data/realism/tN.toml, and write them as a knobs file",
    ),
    opt("--fit-scope", false, &["shared", "deck"], "", "with --fit: one style for the whole tier (shared, the default) or one per deck"),
    opt("--fit-gens", false, &[], "N", "with --fit: generations to run (default 15)"),
    opt("--fit-out", false, &[], "FILE", "with --fit: where to write the knobs (default audit/realism/fit/<tier>-<scope>.toml)"),
    opt(
        "--knobs",
        false,
        &[],
        "FILE",
        "AI knobs (deck styles, counter thresholds, look-ahead weights) from a TOML file, over the built-in values: \
         see rust/crates/sim-cli/src/knobs.rs; --fit writes one",
    ),
    opt(
        "--any-tier",
        true,
        &[],
        "",
        "let --deck run against a tier outside its own (data/my_decks.toml); --pool all then means t0-t5. Only when asked.",
    ),
    opt("--games", false, &[], "N", "games per run (default 1500)"),
    opt("--n", false, &[], "N", "same as --games"),
    opt("--seed", false, &[], "SEED", "first seed (games use seed .. seed+n-1; default 500000)"),
    opt("--profiles", false, &[], "LIST", "AI interaction profiles to run (default loose; conservative,loose for both)"),
    opt("--profile", false, &[], "NAME", "run one profile only (conservative or loose)"),
    opt("--analyze", true, &[], "", "deep report: how the deck wins and loses, plan timing, card report"),
    opt(
        "--trace",
        false,
        &[],
        "GAME",
        "print a play-by-play log of one game (seed offset) and exit; with --calibrate within --pool tN, a game of that tier",
    ),
    opt("--cards", true, &[], "", "list every card in --deck with its tags, source and unmodeled text"),
    opt("--brief", true, &[], "", "skip the per-axis breakdown"),
    opt(
        "--no-explain",
        true,
        &[],
        "",
        "with --swap: skip the paired breakdown (the swapped cards' model status before the run; their drawn / cast / stuck numbers and the game shape of both lists after it). It is on by default and costs no extra games",
    ),
    opt(
        "--no-preflight",
        true,
        &[],
        "",
        "with --swap: skip the two-step preflight that runs before the long run: (1) the AI-rule check (does the deck's priority function rank each swapped-in card above 0, or is it an instant or removal the AI casts as a response) and (2) a quick adaptive dry run (200 games per profile on the first tier) that stops with exit code 3 when a swapped-in card was drawn in 20+ games and cast in under 5% of them.",
    ),
    opt(
        "--preflight-only",
        true,
        &[],
        "",
        "with --swap: run the preflight (AI-rule check and dry run), print it and exit without the main run",
    ),
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
    "usage: commander-sim [-h] [--deck KEY] [--swap \"OUT=>IN\"] [--pool TIER] [--all-decks] [--calibrate WHICH] [--realism]\n\
     \x20                    [--games N] [--seed SEED] [--profiles LIST | --profile NAME] [--analyze] [--trace GAME]\n\
     \x20                    [--cards] [--brief] [--no-explain] [--no-preflight | --preflight-only] [--ai {lookahead,adaptive}]\n\
     \x20                    [--temp T] [--quiet] [--jobs N]\n\
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
    s += "\nExamples\n  commander-sim --deck sauron --pool t2 --profile loose --games 960\n  commander-sim --deck seph \
          --pool t4 --swap \"Blood Artist=>Grim Tutor\"\n  commander-sim --all-decks --pool all --profile loose\n  commander-sim --deck yshtola --pool t1 --any-tier   (only when asked: t1 is outside Y'shtola's tiers)\n";
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
        realism: false,
        knobs: None,
        fit: false,
        fit_scope: "shared".into(),
        fit_gens: 15,
        fit_out: None,
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
        any_tier: false,
        no_explain: false,
        no_preflight: false,
        preflight_only: false,
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
            "--any-tier" => a.any_tier = true,
            "--no-explain" => a.no_explain = true,
            "--no-preflight" => a.no_preflight = true,
            "--preflight-only" => a.preflight_only = true,
            "--calibrate" => a.calibrate = Some(val),
            "--realism" => a.realism = true,
            "--knobs" => a.knobs = Some(val),
            "--fit" => a.fit = true,
            "--fit-scope" => a.fit_scope = val,
            "--fit-gens" => a.fit_gens = int(o.name, &val)?,
            "--fit-out" => a.fit_out = Some(val),
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
        // loose only by default (JD 2026-10-19); --profiles conservative,loose runs both
        None => profiles.unwrap_or_else(|| "loose".into()).split(',').map(|s| s.to_string()).collect(),
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
        assert_eq!(a.profiles, ["loose"]); // the default (JD 2026-10-19)
        let a = p("--deck seph --pool t3 --profiles conservative,loose").unwrap();
        assert_eq!(a.profiles, ["conservative", "loose"]);
        let a = p("--calibrate within").unwrap();
        assert_eq!(a.games, 1500);
    }

    #[test]
    fn swaps_repeat_and_values_may_contain_spaces() {
        let argv: Vec<String> = ["--deck", "sauron", "--swap", "Spell Pierce=>Negate", "--swap", "A=>B"]
            .iter()
            .map(|s| s.to_string())
            .collect();
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

#[cfg(test)]
mod any_tier_tests {
    use super::*;

    fn p(s: &str) -> Result<Args, Exit> {
        parse(&s.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn any_tier_flag_parses() {
        assert!(!p("--deck sauron --pool t3").unwrap().any_tier);
        assert!(p("--deck sauron --pool t1 --any-tier").unwrap().any_tier);
        assert!(p("--deck sauron --pool t1 --any").unwrap().any_tier);
        assert!(p("--any-tier=x").is_err());
    }
}

#[cfg(test)]
mod calibrate_pool_tests {
    use super::*;

    #[test]
    fn calibrate_takes_a_tier_filter() {
        let argv: Vec<String> = "--calibrate within --pool t2".split_whitespace().map(|x| x.to_string()).collect();
        let a = parse(&argv).unwrap();
        assert_eq!((a.calibrate.as_deref(), a.pool.as_deref()), (Some("within"), Some("t2")));
    }
}

#[cfg(test)]
mod explain_flag_tests {
    use super::*;

    fn p(x: &[&str]) -> Result<Args, Exit> {
        parse(&x.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn no_explain_is_a_flag_and_off_by_default() {
        assert!(!p(&["--deck", "cloud", "--swap", "A=>B"]).unwrap().no_explain);
        assert!(p(&["--deck", "cloud", "--swap", "A=>B", "--no-explain"]).unwrap().no_explain);
        // --n still means --games, and a prefix of --no-explain works
        assert_eq!(p(&["--n", "50"]).unwrap().games, 50);
        assert!(p(&["--no-ex"]).unwrap().no_explain);
    }
}

#[cfg(test)]
mod preflight_flag_tests {
    use super::*;

    fn p(a: &[&str]) -> Result<Args, Exit> {
        parse(&a.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn preflight_is_on_by_default_and_the_flags_turn_it_off_or_stop_after_it() {
        let a = p(&["--deck", "kefka", "--swap", "A=>B"]).unwrap();
        assert!(!a.no_preflight && !a.preflight_only);
        assert!(p(&["--deck", "kefka", "--swap", "A=>B", "--no-preflight"]).unwrap().no_preflight);
        assert!(p(&["--deck", "kefka", "--swap", "A=>B", "--preflight-only"]).unwrap().preflight_only);
        assert!(help().contains("--no-preflight") && help().contains("--preflight-only"));
    }
}
