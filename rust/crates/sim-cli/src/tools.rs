//! The experiment tools (commander_sim/tools/swaptest.py, searchtest.py) and the pool listing (pools.py).

use crate::cardinfo;
use crate::decks::{self, PoolDeck, structural_problems};
use crate::play::{GameOut, RunOpts, Sim};
use sim_core::ai::search;
use sim_core::settings::{AiMode, Profile};

fn load(ai: AiMode, jobs: usize) -> Result<Sim, String> {
    Sim::load(RunOpts { ai, temp: 1.0, jobs })
}

fn cores() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

fn finish(r: Result<(), String>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn arg<T: std::str::FromStr>(v: &[String], i: usize, what: &str) -> Result<T, String> {
    v.get(i).ok_or_else(|| format!("missing {what}"))?.parse().map_err(|_| format!("bad {what}: {:?}", v[i]))
}

fn is_help(argv: &[String]) -> bool {
    argv.iter().any(|a| a == "-h" || a == "--help")
}

fn note_errors(sim: &Sim) {
    let errs = sim.errors.lock().unwrap();
    for e in errs.iter() {
        println!("  seed {} left out, error: {}", e.seed, e.what);
    }
}

// ------------------------------------------------------------------ swaptest
const SWAPTEST: &str = "Test proposed list changes for an opponent-pool deck without editing the list.

    commander-sim swaptest <deck key> <tier the deck plays against> <games> \"Out Card>In Card; Out>In; ...\" [jobs]

Runs the deck alone against three decks of that tier, paired (same seeds, seats and opponents' draws), once with
the list as written and once with the swaps, and reports both win rates (adaptive AI, conservative profile). The
variant list is checked as the real lists are: 100 cards, singleton, every card in data/cards.json, colour identity
and the tier's Game Changer limit (Scryfall's legality data isn't exported, so bans aren't checked).";

/// swaptest.variant: the swaps made in place (each out card must be in the list, each in card not)
fn variant(deck: &PoolDeck, swaps: &[(String, String)]) -> Result<Vec<String>, String> {
    let mut cards = deck.cards.clone();
    for (out, inn) in swaps {
        let i = cards.iter().position(|c| c == out).ok_or_else(|| format!("'{out}' is not in {}", deck.key))?;
        if cards.contains(inn) {
            return Err(format!("'{inn}' is already in {}", deck.key));
        }
        cards[i] = inn.clone();
    }
    Ok(cards)
}

/// pools.GC_RULES: Game Changers allowed per tier
fn gc_rule(tier: &str) -> Option<(usize, usize)> {
    match tier {
        "t1" => Some((0, 0)),
        "t2" => Some((1, 2)),
        "t3" => Some((3, 3)),
        _ => None,
    }
}

/// swaptest.check: pools.structural_problems and the card checks that the exported card data can answer
fn check(sim: &Sim, deck: &PoolDeck, cards: &[String]) -> Vec<String> {
    let mut probs = structural_problems(cards, &deck.commander, 100);
    let mut names: Vec<&String> = cards.iter().collect();
    names.sort();
    names.dedup();
    let missing: Vec<&str> = names.iter().filter(|n| sim.raw_card(n).is_none()).map(|n| n.as_str()).collect();
    if !missing.is_empty() {
        probs.push(format!("not in data/cards.json: {}", missing.join(", ")));
    }
    if let Some(cmd) = sim.raw_card(&deck.commander) {
        let ident = cardinfo::identity(cmd);
        let off: Vec<&str> = names
            .iter()
            .filter_map(|n| sim.raw_card(n))
            .filter(|c| !cardinfo::identity(c).is_subset(&ident))
            .map(|c| c.name.as_str())
            .collect();
        if !off.is_empty() {
            let id: String = ident.iter().collect(); // sorted, as Python's ''.join(sorted(ident))
            probs.push(format!(
                "outside colour identity {}: {}",
                if id.is_empty() { "C" } else { &id },
                off.join(", ")
            ));
        }
    }
    let gcs = names.iter().filter_map(|n| sim.raw_card(n)).filter(|c| c.game_changer == Some(true)).count();
    if let Some((lo, hi)) = gc_rule(&deck.tier)
        && !(lo..=hi).contains(&gcs)
    {
        probs.push(format!("{gcs} Game Changers, tier {} allows {lo}-{hi}", deck.tier));
    }
    probs
}

pub fn swaptest(argv: &[String]) -> i32 {
    if argv.len() < 4 || is_help(argv) {
        println!("{SWAPTEST}");
        return if is_help(argv) { 0 } else { 2 };
    }
    finish((|| {
        let (me, tier) = (argv[0].as_str(), argv[1].as_str());
        let n: u64 = arg(argv, 2, "number of games")?;
        let swaps: Vec<(String, String)> = argv[3]
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|s| match s.split_once('>') {
                Some((a, b)) => Ok((a.trim().to_string(), b.trim().to_string())),
                None => Err(format!("a swap looks like 'Out Card>In Card': {s:?}")),
            })
            .collect::<Result<_, _>>()?;
        let jobs = if argv.len() > 4 { arg(argv, 4, "jobs")? } else { cores() };
        let sim = load(AiMode::Adaptive, jobs)?;
        let deck = sim.pool.iter().find(|d| d.key == me).ok_or_else(|| format!("no pool deck {me:?}"))?.clone();
        let opp = sim.pool_keys(tier);
        if opp.is_empty() {
            return Err(format!("no tier {tier:?}"));
        }
        let cards = variant(&deck, &swaps)?;
        let probs = check(&sim, &deck, &cards);
        if !probs.is_empty() {
            return Err(format!("variant list fails validation: {}", probs.join("; ")));
        }
        let vseat = sim.seat(me, &cards)?;
        let st = sim.settings(Profile::Conservative);
        sim.prog.lock().unwrap().on = false;
        let seeds: Vec<u64> = (500000..500000 + n).collect();
        let res = sim.run_seeds(&seeds, |s| {
            let won = |seat: Option<&sim_core::engine::turn::Seat>| {
                let seats = sim.seats_for(s, Some(me), seat, &opp, 3);
                let g = GameOut::from_game(&sim.play(s, &seats, &st, false), false);
                g.winner.is_some_and(|w| g.players[w].key == me)
            };
            (won(None), won(Some(&vseat)))
        });
        let m = res.len() as f64;
        let b = res.iter().filter(|r| r.1.0).count() as f64 / m;
        let v = res.iter().filter(|r| r.1.1).count() as f64 / m;
        println!(
            "{me} alone vs {tier}: as written {:.1}%, with {} swaps {:.1}% ({:+.1} pts, n={})",
            100.0 * b,
            swaps.len(),
            100.0 * v,
            100.0 * (v - b),
            res.len()
        );
        note_errors(&sim);
        Ok(())
    })())
}

// ------------------------------------------------------------------ searchtest
const SEARCHTEST: &str =
    "Paired test of the look-ahead AI: a pool deck alone against three decks of a tier, the same seeds
played with the heuristic AI and with search.

    commander-sim searchtest <deck key> <tier> <games> [rollouts] [top_k] [jobs] [--any-tier]

One of your decks is only run against its own tiers (data/my_decks.toml); --any-tier overrides that.

In the Rust engine every deck at the table searches in the second run (Python's 'all' option): the look-ahead has
no per-deck switch. Environment: SEARCH_HORIZON (own turns to look ahead), SEARCH_COMBO_W (the combo weight).";

pub fn searchtest(argv: &[String]) -> i32 {
    let any_tier = argv.iter().any(|x| x == "--any-tier");
    let argv: &[String] = &argv.iter().filter(|x| *x != "--any-tier").cloned().collect::<Vec<_>>();
    if argv.len() < 3 || is_help(argv) {
        println!("{SEARCHTEST}");
        return if is_help(argv) { 0 } else { 2 };
    }
    finish((|| {
        let (me, tier) = (argv[0].as_str(), argv[1].as_str());
        let n: u64 = arg(argv, 2, "number of games")?;
        let jobs = if argv.len() > 5 { arg(argv, 5, "jobs")? } else { cores() };
        let sim = load(AiMode::Adaptive, jobs)?;
        if !sim.pool.iter().any(|d| d.key == me) && !sim.mine.contains_key(me) {
            return Err(format!("no deck {me:?}"));
        }
        if sim.mine.contains_key(me) {
            crate::poolmode::select_tiers(me, &crate::poolmode::deck_tiers(&sim, me), tier, any_tier)?;
        }
        let opp = sim.pool_keys(tier);
        if opp.is_empty() {
            return Err(format!("no tier {tier:?}"));
        }
        let heur = sim.settings(Profile::Conservative);
        let mut srch = sim_core::settings::Settings::new(Profile::Conservative, AiMode::Lookahead, 1.0);
        if argv.len() > 3 {
            srch.search.rollouts = arg(argv, 3, "rollouts")?;
        }
        if argv.len() > 4 {
            srch.search.top_k = arg(argv, 4, "top_k")?;
        }
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        if let Some(h) = env("SEARCH_HORIZON") {
            srch.search.horizon = h.parse().map_err(|_| format!("bad SEARCH_HORIZON {h:?}"))?;
        }
        if let Some(w) = env("SEARCH_COMBO_W") {
            srch.search.combo_w = w.parse().map_err(|_| format!("bad SEARCH_COMBO_W {w:?}"))?;
        }
        {
            let mut p = sim.prog.lock().unwrap();
            p.plan(n);
            p.announce(&format!("{me} vs {tier}, heuristic and search"), "lookahead", jobs);
        }
        let seeds: Vec<u64> = (500000..500000 + n).collect();
        let res = sim.run_seeds(&seeds, |s| {
            let seats = sim.seats_for(s, Some(me), None, &opp, 3);
            let won = |g: &sim_core::state::Game| g.winner.is_some_and(|w| g.player(w).key == me);
            let g0 = sim.play(s, &seats, &heur, false);
            let before = search::stats();
            let g1 = sim.play(s, &seats, &srch, false);
            let after = search::stats();
            (won(&g0), won(&g1), g0.stopped || g1.stopped, before, after)
        });
        let m = res.len().max(1) as f64;
        let b = res.iter().filter(|r| r.1.0).count() as f64 / m;
        let v = res.iter().filter(|r| r.1.1).count() as f64 / m;
        let differ = res.iter().filter(|r| r.1.0 != r.1.1).count();
        let mut st = search::Stats::default();
        for (_, (_, _, _, a, z)) in &res {
            st.decisions += z.decisions - a.decisions;
            st.playouts += z.playouts - a.playouts;
            st.changed += z.changed - a.changed;
            st.cut += z.cut - a.cut;
            st.capped += z.capped - a.capped;
            st.big_board += z.big_board - a.big_board;
            st.max_work = st.max_work.max(z.max_work);
        }
        println!(
            "[horizon {}, combo_w {}] {me} alone vs {tier}: heuristic {:.1}%, search {:.1}% ({:+.1} pts, n={}, \
             {differ} games differ in outcome)  {{'decisions': {}, 'playouts': {}, 'changed': {}, 'cut': {}, \
             'max_work': {}, 'capped': {}, 'big_board': {}}}",
            srch.search.horizon,
            srch.search.combo_w,
            100.0 * b,
            100.0 * v,
            100.0 * (v - b),
            res.len(),
            st.decisions,
            st.playouts,
            st.changed,
            st.cut,
            st.max_work,
            st.capped,
            st.big_board
        );
        for (s, r) in &res {
            if r.2 {
                println!("  game stopped (step budget): seed {s}");
            }
        }
        note_errors(&sim);
        Ok(())
    })())
}

// ------------------------------------------------------------------ pools
/// pools.main: list the pool decks
pub fn pools(argv: &[String]) -> i32 {
    if is_help(argv) {
        println!("commander-sim pools: list the opponent-pool decks (tier, key, commander)");
        return 0;
    }
    finish((|| {
        let root = decks::root()?;
        for d in decks::load_pool(&root)? {
            println!("{} {:40} {}", d.tier, d.key, d.commander);
        }
        Ok(())
    })())
}
