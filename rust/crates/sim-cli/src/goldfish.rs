//! Goldfish mode (`--goldfish KEYS`): each deck plays alone against three inert seats (`sim_core::goldfish`: 40 life,
//! 99 Wastes, they never cast, activate, attack, block or respond; their turns still pass, so a round is real). The
//! deck under test is always seat 1, so its own turn number is the round. Same seeds and seed streams as pool mode
//! (`lib:{seed}:{key}` for the shuffles), so the same seed gives the same game.
//!
//! Per deck: the kill turn (median and mean, the share of games won by turn 1 ... 8, no kill by the round cap), how
//! it won, the mulligans, the turn a line was first ready, the protection held on the kill turn, the lines it went
//! for, and when the commander was cast (with Scion of the Ur-Dragon's activations by Dragon).

use crate::args::Args;
use crate::decks::MINE;
use crate::play::{GameOut, Sim, ai_name, profile};
use crate::stats::Counter;
use sim_core::engine::turn::Seat;
use sim_core::ids::PlayerId;
use std::collections::BTreeMap;
use std::time::Instant;

/// the round cap: games stop after this many rounds (the deck's 20th turn)
pub const ROUNDS: u32 = 20;

/// the decks `--goldfish` takes: JD's deck keys and the six Tier 6 pool keys ("t6" names all six)
pub fn deck_keys(sim: &Sim, spec: &str) -> Result<Vec<String>, String> {
    let t6 = sim.pool_keys("t6");
    let mut out: Vec<String> = vec![];
    for k in spec.split(',').map(str::trim).filter(|k| !k.is_empty()) {
        let add: Vec<String> = if k == "t6" {
            t6.clone()
        } else if MINE.contains(&k) || t6.iter().any(|x| x == k) {
            vec![k.to_string()]
        } else {
            return Err(format!(
                "--goldfish: {k:?} is not one of your decks ({}) or a Tier 6 pool deck ({}, or t6 for all six)",
                MINE.join(", "),
                t6.join(", ")
            ));
        };
        for a in add {
            if !out.contains(&a) {
                out.push(a);
            }
        }
    }
    if out.is_empty() {
        return Err("--goldfish needs at least one deck key".into());
    }
    Ok(out)
}

/// one goldfish game, as the report reads it
#[derive(Debug, Clone, PartialEq)]
pub struct GfGame {
    /// the deck's own turn of the kill (None: no kill by the round cap, or the deck lost)
    pub kill: Option<u32>,
    /// how it won: a combo line's name, "damage", "commander damage", "poison", "other"
    pub route: Option<String>,
    /// the deck itself lost (Final Fortune's turn, an empty library, its own life)
    pub lost: bool,
    /// mulligans taken (0-3): 0 and 1 keep seven cards (the first mulligan is free), 2 keeps six, 3 keeps five
    pub mulls: i64,
    /// the round (= own turn) a line was first ready (Tier 6 decks), or the deck's plan came online (the others)
    pub ready: Option<u32>,
    /// on a kill: protection held when the deck went for the win
    pub prot: Option<bool>,
    /// the lines it went for this game (`go|<line>`)
    pub go: Vec<String>,
    /// the own turn the commander first resolved
    pub cmd_turn: Option<u32>,
    /// deck-specific counters (`scion|...`: Scion of the Ur-Dragon's activations by Dragon)
    pub extra: Vec<(String, i64)>,
    /// --- ai-audit: own turns taken, and the loss accounting (gf_audit)
    pub own_turns: u32,
    pub audit: Option<sim_core::gf_audit::AuditOut>,
    /// --- ai-flow: the deck's flow counters (`fl|...`, read by `flow::FlowTally`)
    pub flow: Vec<(String, i64)>,
}

impl GfGame {
    pub fn read(g: &sim_core::state::Game) -> GfGame {
        let me = PlayerId(0);
        let out = GameOut::from_game(g, false);
        let p = &out.players[0];
        let won = out.winner == Some(0) && out.wintype != Some("timeout");
        let kill = won.then_some(p.turns);
        let route = if won { crate::t6report::win_route(&out, 0) } else { None };
        let stat = |k: &str| p.stats.get(k).copied();
        let prot = won.then(|| match (stat("gf|prot"), stat("gf|go_turn")) {
            // a combo win: what the deck held as it went for the line; a damage win: what it holds as it lands
            (Some(v), Some(t)) if t == p.turns as i64 && route.as_deref().is_some_and(|r| r != "damage") => v != 0,
            _ => sim_core::goldfish::protected(g, me),
        });
        let ready = match stat("sp|ready") {
            Some(r) => Some(r as u32),
            None => crate::report::plan_turn(p, p.key),
        };
        let mut go: Vec<String> = p.stats.keys().filter_map(|k| k.strip_prefix("go|")).map(|s| s.to_string()).collect();
        go.sort();
        let mut extra: Vec<(String, i64)> =
            // --- ai-planner: and the planner's counters (`plan|...`)
            p.stats
                .iter()
                .filter(|(k, _)| k.starts_with("scion|") || k.starts_with("plan|"))
                .map(|(k, v)| (k.to_string(), *v))
                .collect();
        extra.sort();
        // --- ai-flow: in key order (the stats map has none), so the same seed gives the same game record
        let mut flow: Vec<(String, i64)> =
            p.stats.iter().filter(|(k, _)| k.starts_with("fl|")).map(|(k, v)| (k.to_string(), *v)).collect();
        flow.sort();
        GfGame {
            kill,
            route,
            lost: !p.alive,
            mulls: stat("mull_n").unwrap_or(0),
            ready,
            prot,
            go,
            cmd_turn: p.milestone.get("cmd_cast").copied(),
            extra,
            own_turns: p.turns, // --- ai-audit
            audit: sim_core::gf_audit::report(g, won), // --- ai-audit
            // --- ai-flow
            flow,
        }
    }
}

/// the seats of a goldfish game: the deck, then the three inert seats
pub fn seats(sim: &Sim, deck: &Seat) -> Result<Vec<Seat>, String> {
    let mut v = vec![deck.clone()];
    v.extend(sim_core::goldfish::inert_seats(&sim.engine.db)?);
    Ok(v)
}

/// a deck's list: yours from decklists/<owner>/, a pool deck's from its file
fn list_of(sim: &Sim, key: &str) -> Result<Vec<String>, String> {
    if let Some(l) = sim.mine.get(key) {
        return Ok(l.clone());
    }
    sim.pool.iter().find(|d| d.key == key).map(|d| d.cards.clone()).ok_or_else(|| format!("no deck {key:?}"))
}

/// play n goldfish games of `key` from seed0
pub fn run_deck(sim: &Sim, key: &str, prof: &str, n: u64, seed0: u64) -> Result<Vec<GfGame>, String> {
    let deck = sim.seat(key, &list_of(sim, key)?)?;
    let seats = seats(sim, &deck)?;
    let st = sim.settings(profile(prof)?);
    let seeds: Vec<u64> = (seed0..seed0 + n).collect();
    let games = sim.run_seeds(&seeds, |s| GfGame::read(&sim.play(s, &seats, &st, false)));
    Ok(games.into_iter().map(|x| x.1).collect())
}

fn pct(k: usize, n: usize) -> f64 {
    100.0 * k as f64 / n.max(1) as f64
}

/// the median of a list where None (no kill) sorts after every turn; None when half or more are None
fn median_turn(xs: &[Option<u32>]) -> Option<f64> {
    let mut v: Vec<u32> = xs.iter().map(|x| x.unwrap_or(u32::MAX)).collect();
    v.sort_unstable();
    if v.is_empty() {
        return None;
    }
    let n = v.len();
    let (a, b) = (v[(n - 1) / 2], v[n / 2]);
    if a == u32::MAX || b == u32::MAX {
        return None;
    }
    Some((a as f64 + b as f64) / 2.0)
}

fn mean(xs: impl Iterator<Item = u32>) -> Option<f64> {
    let v: Vec<u32> = xs.collect();
    (!v.is_empty()).then(|| v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64)
}

fn fmt_opt(x: Option<f64>, none: &str) -> String {
    x.map_or(none.to_string(), |v| format!("{v:.1}"))
}

/// one deck's summary numbers (the closing table and the tests)
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub n: usize,
    pub median: Option<f64>,
    pub mean: Option<f64>,
    /// cumulative share of games with a kill by own turn 1 ... 8
    pub by: [f64; 8],
    pub no_kill: f64,
    /// share of kills with protection held
    pub prot: Option<f64>,
}

pub fn summarize(games: &[GfGame]) -> Summary {
    let n = games.len();
    let kills: Vec<Option<u32>> = games.iter().map(|g| g.kill).collect();
    let mut by = [0.0; 8];
    for (t, b) in by.iter_mut().enumerate() {
        *b = pct(kills.iter().filter(|k| k.is_some_and(|k| k <= t as u32 + 1)).count(), n);
    }
    let won: Vec<&GfGame> = games.iter().filter(|g| g.kill.is_some()).collect();
    Summary {
        n,
        median: median_turn(&kills),
        mean: mean(kills.iter().flatten().copied()),
        by,
        no_kill: pct(n - won.len(), n),
        prot: (!won.is_empty()).then(|| pct(won.iter().filter(|g| g.prot == Some(true)).count(), won.len())),
    }
}

/// the cards of this deck that count as protection on the kill turn (for the report)
fn protection_cards(sim: &Sim, key: &str) -> Vec<String> {
    let mut out: Vec<String> = list_of(sim, key)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| sim.engine.db.id(c).is_some_and(|id| sim_core::goldfish::protection_card(&sim.engine.db, id)))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// the report of one deck
pub fn render(sim: &Sim, key: &str, games: &[GfGame], ai: &str, seed0: u64) -> String {
    let n = games.len();
    let s = summarize(games);
    let mut o = String::new();
    o += &format!(
        "\n=== {} ({key}) goldfish [{ai} AI, {n} games from seed {seed0}; three inert seats at 40 life; round cap {ROUNDS}] ===\n",
        sim.name_of(key)
    );
    o += &format!(
        "Kill turn (own turn): median {}, mean {} (over the {} kills)\n",
        fmt_opt(s.median, "none (half the games have no kill)"),
        fmt_opt(s.mean, "n/a"),
        n - games.iter().filter(|g| g.kill.is_none()).count()
    );
    let by: Vec<String> = s.by.iter().enumerate().map(|(t, b)| format!("T{} {b:.1}%", t + 1)).collect();
    o += &format!("  won by turn: {}\n", by.join("  "));
    let lost = games.iter().filter(|g| g.lost).count();
    o += &format!(
        "  no kill by round {ROUNDS}: {:.1}% (of which the deck itself lost: {:.1}% of games)\n",
        s.no_kill,
        pct(lost, n)
    );
    let mut routes: Counter<String> = Counter::new();
    for g in games {
        if let Some(r) = &g.route {
            routes.add(r.clone(), 1);
        }
    }
    let wins = routes.total().max(1) as usize;
    let parts: Vec<String> =
        routes.most_common().iter().map(|(k, v)| format!("{k} {:.1}%", pct(*v as usize, wins))).collect();
    o += &format!("How it won (share of kills): {}\n", if parts.is_empty() { "-".into() } else { parts.join(", ") });
    let mull = |f: &dyn Fn(i64) -> bool| pct(games.iter().filter(|g| f(g.mulls)).count(), n);
    o += &format!(
        "Mulligans: kept 7 cards {:.1}% (no mulligan {:.1}%, the free mulligan {:.1}%), 6 cards {:.1}%, 5 or fewer {:.1}%\n",
        mull(&|m| m <= 1),
        mull(&|m| m == 0),
        mull(&|m| m == 1),
        mull(&|m| m == 2),
        mull(&|m| m >= 3)
    );
    let ready: Vec<Option<u32>> = games.iter().map(|g| g.ready).collect();
    let t6 = sim_core::impls::t6_plans::is_t6_key(key);
    o += &format!(
        "Ready turn ({}): median {}, mean {}, reached in {:.1}% of games\n",
        if t6 { "a line first ready, as the Tier 6 speed counters measure it" } else { "the plan came online" },
        fmt_opt(median_turn(&ready), "none"),
        fmt_opt(mean(ready.iter().flatten().copied()), "n/a"),
        pct(ready.iter().flatten().count(), n)
    );
    o += &format!(
        "Protection on the kill turn: {} of kills held a protection card or had a cast lock in force\n",
        s.prot.map_or("n/a".to_string(), |p| format!("{p:.1}%"))
    );
    let pc = protection_cards(sim, key);
    o += &format!(
        "  counted ({}): {}\n",
        pc.len(),
        if pc.is_empty() { "none in this list".to_string() } else { pc.join(", ") }
    );
    let mut go: Counter<String> = Counter::new();
    for g in games {
        for l in &g.go {
            go.add(l.clone(), 1);
        }
    }
    if !go.is_empty() {
        let parts: Vec<String> =
            go.most_common().iter().map(|(k, v)| format!("{k} {:.1}%", pct(*v as usize, n))).collect();
        o += &format!("Went for (share of games): {}\n", parts.join(", "));
        let mut won_by: Counter<String> = Counter::new();
        for g in games {
            if let Some(r) = &g.route {
                won_by.add(r.clone(), 1);
            }
        }
        let parts: Vec<String> = won_by
            .most_common()
            .iter()
            .filter(|(k, _)| go.get(k) > 0)
            .map(|(k, v)| format!("{k} {:.1}% of tries", pct(*v as usize, go.get(k) as usize)))
            .collect();
        if !parts.is_empty() {
            o += &format!("  converted: {}\n", parts.join(", "));
        }
    }
    let cmd: Vec<Option<u32>> = games.iter().filter(|g| g.cmd_turn.is_some()).map(|g| g.cmd_turn).collect();
    o += &format!(
        "Commander cast: {:.1}% of games, median own turn {} (of the games it was cast)\n",
        pct(cmd.len(), n),
        fmt_opt(median_turn(&cmd), "n/a")
    );
    let mut extra: BTreeMap<String, (usize, i64)> = BTreeMap::new();
    for g in games {
        for (k, v) in &g.extra {
            let e = extra.entry(k.clone()).or_default();
            e.0 += 1;
            e.1 += v;
        }
    }
    for (k, (games_with, total)) in &extra {
        o += &format!("  {k}: in {:.1}% of games ({total} in all)\n", pct(*games_with, n));
    }
    o += &crate::gf_audit::render(games); // --- ai-audit
    // --- ai-flow: the card and mana flow, own turns 1-8
    let mut fl = crate::flow::FlowTally::default();
    for g in games {
        fl.add(key, g.flow.iter().map(|(k, v)| (k.as_str(), *v)));
    }
    if !fl.is_empty() {
        o += &fl.render(&|k| sim.name_of(k));
    }
    o
}

/// the closing table: one row per deck
pub fn table(rows: &[(String, Summary)]) -> String {
    let mut o = String::from("\n=== Goldfish summary (own turn of the kill; no opponents interacting) ===\n");
    o += &format!(
        "  {:34} {:>7} {:>6} {:>7} {:>7} {:>7} {:>8} {:>11}\n",
        "deck", "median", "mean", "by T1", "by T2", "by T3", "no kill", "protection"
    );
    for (k, s) in rows {
        o += &format!(
            "  {:34} {:>7} {:>6} {:>6.1}% {:>6.1}% {:>6.1}% {:>7.1}% {:>11}\n",
            k,
            fmt_opt(s.median, "-"),
            fmt_opt(s.mean, "-"),
            s.by[0],
            s.by[1],
            s.by[2],
            s.no_kill,
            s.prot.map_or("-".to_string(), |p| format!("{p:.1}%"))
        );
    }
    o
}

/// `--goldfish KEYS`: every deck's report, then the summary table (or `--trace GAME`: one game's play-by-play)
pub fn main(sim: &Sim, a: &Args) -> Result<(), String> {
    let spec = a.goldfish.as_deref().unwrap_or("");
    let keys = deck_keys(sim, spec)?;
    let prof = a.profiles.first().cloned().unwrap_or_else(|| "loose".into());
    let ai = ai_name(sim.opts.ai);
    if let Some(game) = a.trace {
        let key = &keys[0];
        let seed = a.seed + game;
        let deck = sim.seat(key, &list_of(sim, key)?)?;
        let seats = seats(sim, &deck)?;
        let st = sim.settings(profile(&prof)?);
        let g = crate::play::catch_game(|| sim.play(seed, &seats, &st, true))?;
        println!("Play-by-play of goldfish game {game} (seed {seed}): {} alone, {ai} AI", sim.name_of(key));
        println!("{}", g.log.clone().unwrap_or_default().join("\n"));
        let gf = GfGame::read(&g);
        println!(
            "Kill: {}",
            gf.kill.map_or("none".to_string(), |t| format!("own turn {t} ({})", gf.route.clone().unwrap_or_default()))
        );
        if let Some(au) = &gf.audit {
            print!("{}", crate::gf_audit::render_trace(au)); // --- ai-audit
        }
        return Ok(());
    }
    sim.prog.lock().unwrap().on = !a.quiet;
    {
        let mut pr = sim.prog.lock().unwrap();
        pr.plan(a.games * keys.len() as u64);
        pr.announce(
            &format!("goldfish, {} deck{}", keys.len(), if keys.len() > 1 { "s" } else { "" }),
            ai,
            sim.opts.jobs,
        );
    }
    let t0 = Instant::now();
    let mut rows = vec![];
    for k in &keys {
        sim.prog.lock().unwrap().label = format!("goldfish {k}");
        let games = run_deck(sim, k, &prof, a.games, a.seed)?;
        print!("{}", render(sim, k, &games, ai, a.seed));
        rows.push((k.clone(), summarize(&games)));
    }
    print!("{}", table(&rows));
    println!("\n({:.0}s total)", t0.elapsed().as_secs_f64());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;
    use sim_core::settings::AiMode;

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    /// the log lines in which an inert seat is the actor (casts, plays, activates, attacks, blocks, responds ...):
    /// every log line about a player starts with its name after the round and the indent
    fn inert_actions(log: &[String]) -> Vec<String> {
        log.iter()
            .filter(|l| {
                // "R12    Goldfish 2 casts ...": the round, then the indent
                let t = l.trim_start_matches('R').trim_start_matches(|c: char| c.is_ascii_digit()).trim_start();
                // passive lines: what happens to the seat (Wishclaw Talisman handed over, a mill, life lost)
                t.starts_with("Goldfish ")
                    && ![" draws", " loses", " mills", " gains control of"].iter().any(|x| t.contains(x))
            })
            .cloned()
            .collect()
    }

    #[test]
    fn the_inert_seats_never_act() {
        let sim = sim();
        let st = sim.settings(profile("loose").unwrap());
        for key in ["kraum-tymna-blue-breach-midrange", "rograkh-silas-turbo-breach"] {
            let deck = sim.seat(key, &list_of(&sim, key).unwrap()).unwrap();
            let seats = seats(&sim, &deck).unwrap();
            for seed in 500000..500006 {
                let g = sim.play(seed, &seats, &st, true);
                let log = g.log.clone().unwrap();
                assert!(log.iter().any(|l| l.contains("--- Goldfish 2 turn")), "the inert seats take turns");
                let acts = inert_actions(&log);
                assert!(acts.is_empty(), "{key} seed {seed}: an inert seat acted: {acts:?}");
                for q in 1..4 {
                    let pl = g.player(PlayerId(q));
                    assert!(pl.lands.is_empty() && pl.cast_names.is_empty(), "seat {} played nothing", q + 1);
                    assert_eq!(pl.stats.get("mull_n"), None, "seat {} kept its seven", q + 1);
                }
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_report() {
        let sim = sim();
        let key = "rograkh-silas-turbo-breach";
        let a = run_deck(&sim, key, "loose", 12, 500000).unwrap();
        let b = run_deck(&sim, key, "loose", 12, 500000).unwrap();
        assert_eq!(a, b);
        assert_eq!(render(&sim, key, &a, "adaptive", 500000), render(&sim, key, &b, "adaptive", 500000));
        // the games are the pool games' seeds: a different seed gives other games
        let c = run_deck(&sim, key, "loose", 12, 600000).unwrap();
        assert_ne!(a, c);
    }

    // --- ai-audit
    /// every own turn of the deck falls in exactly one bucket: the bucket counts sum to the own turns, turn by turn
    /// in order, and a kill's last turn (only it) is `won`
    #[test]
    fn the_loss_buckets_sum_to_the_own_turns() {
        use sim_core::gf_audit::Bucket;
        let sim = sim();
        for key in ["scion", "kinnan-bonder-mana-combo"] {
            let games = run_deck(&sim, key, "loose", 16, 500000).unwrap();
            let audits: Vec<&sim_core::gf_audit::AuditOut> = games.iter().map(|g| g.audit.as_ref().unwrap()).collect();
            let (all, early) = crate::gf_audit::bucket_counts(&audits);
            let own: u32 = games.iter().map(|g| g.own_turns).sum();
            assert_eq!(all.iter().map(|x| x.1).sum::<usize>() as u32, own, "{key}: buckets vs own turns");
            let own4: u32 = games.iter().map(|g| g.own_turns.min(4)).sum();
            assert_eq!(early.iter().map(|x| x.1).sum::<usize>() as u32, own4, "{key}: turns 1-4");
            for g in &games {
                let a = g.audit.as_ref().unwrap();
                let turns: Vec<u32> = a.turns.iter().map(|t| t.turn).collect();
                assert_eq!(turns, (1..=g.own_turns).collect::<Vec<_>>(), "{key}: one record per own turn");
                let won = a.turns.iter().filter(|t| t.bucket == Bucket::Won).count();
                assert_eq!(won, g.kill.is_some() as usize, "{key}: one won turn per kill");
                if let Some(k) = g.kill {
                    assert_eq!(a.turns.last().unwrap().bucket, Bucket::Won);
                    assert_eq!(k, g.own_turns);
                }
            }
            assert!(crate::gf_audit::render(&games).contains("Loss accounting"));
        }
    }

    /// the accounting is stats only: the same games with it switched off play out the same
    #[test]
    fn the_loss_accounting_changes_no_play() {
        let sim = sim();
        let key = "scion";
        let deck = sim.seat(key, &list_of(&sim, key).unwrap()).unwrap();
        let seats = seats(&sim, &deck).unwrap();
        let st = sim.settings(profile("loose").unwrap());
        for seed in 500000..500008 {
            let on = sim.play(seed, &seats, &st, true);
            let mut g = sim_core::engine::turn::setup_game(
                sim.engine.db.clone(),
                sim.engine.registry.clone(),
                std::sync::Arc::new(st.clone()),
                seed,
                &seats,
                true,
            );
            assert!(g.audit.is_some());
            g.audit = None;
            sim_core::engine::turn::run_rounds(&mut g, ROUNDS);
            let strip = |l: &Vec<String>| l.iter().filter(|x| !x.contains("[audit]")).cloned().collect::<Vec<_>>();
            assert_eq!(strip(on.log.as_ref().unwrap()), strip(g.log.as_ref().unwrap()), "seed {seed}");
        }
    }

    #[test]
    fn the_deck_keys_are_yours_and_the_tier_6_pool() {
        let sim = sim();
        assert_eq!(deck_keys(&sim, "t6").unwrap().len(), 6);
        assert_eq!(deck_keys(&sim, "seph,t6,seph").unwrap().len(), 7);
        assert!(deck_keys(&sim, "krenko-mono-red-goblins").is_err());
        assert!(deck_keys(&sim, "").is_err());
    }

    #[test]
    fn the_summary_counts_turns_and_the_median_needs_half_the_games() {
        let mk = |k: Option<u32>, prot: bool| GfGame {
            kill: k,
            route: k.map(|_| "damage".to_string()),
            lost: false,
            mulls: 0,
            ready: None,
            prot: k.map(|_| prot),
            go: vec![],
            cmd_turn: None,
            extra: vec![],
            own_turns: 0, // --- ai-audit
            audit: None,
            flow: vec![], // --- ai-flow
        };
        let gs = vec![mk(Some(2), true), mk(Some(3), false), mk(Some(3), true), mk(None, false)];
        let s = summarize(&gs);
        assert_eq!(s.median, Some(3.0));
        assert_eq!((s.by[0], s.by[1], s.by[2]), (0.0, 25.0, 75.0));
        assert_eq!(s.no_kill, 25.0);
        assert!((s.prot.unwrap() - 66.666).abs() < 0.01);
        let s = summarize(&[mk(Some(2), true), mk(None, false)]);
        assert_eq!(s.median, None, "half the games have no kill");
    }
}
