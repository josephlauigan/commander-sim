//! The two-step preflight of every `--swap` run (`--no-preflight` skips it, `--preflight-only` stops after it): it
//! catches a swapped-in card that is fully modeled but that the deck's own AI never casts (it ranks it 0), before the
//! long look-ahead run spends hours on it.
//!
//! 1. the AI-rule check (instant): the deck's cast priority (`ai::decks::deck_prio`) for each swapped-in card on
//!    mid-game boards of the swapped list, and whether the card can still be cast as a response (an instant, or tagged
//!    ctr / rem / wipe / prot).
//! 2. a quick dry run (about a minute): the swapped list on the adaptive AI, 200 games per profile on the first tier;
//!    each swapped-in card's drawn and cast numbers. A card drawn in 20+ games and cast in under 5% of them stops the
//!    run with exit code 3.

use crate::args::Args;
use crate::play::{Sim, profile};
use crate::poolmode::{Ctx, tier_plan};
use crate::report::Analysis;
use sim_core::ai::{decks::deck_prio, pool::generic_prio};
use sim_core::engine::turn::{run_rounds, setup_game};
use sim_core::ids::PlayerId;
use sim_core::settings::{AiMode, Settings};
use sim_core::tag::Tag;
use std::sync::Arc;

/// games per profile in the dry run
pub const DRY_GAMES: u64 = 200;
/// a card drawn in at least this many dry-run games is judged ...
pub const MIN_DRAWN: i64 = 20;
/// ... and warned about when cast in fewer than this share of them
pub const MAX_CAST_PCT: f64 = 5.0;
/// the exit code of a stop
pub const EXIT_STOP: i32 = 3;

/// one swapped-in card's AI-rule result
#[derive(Debug, Clone, PartialEq)]
pub struct AiRule {
    pub card: String,
    /// the best priority over the sampled boards
    pub prio: i32,
    /// the deck's own rule gave it (the priority differs from the generic one)
    pub deck_rule: bool,
    /// an instant or a ctr / rem / wipe / prot card: cast by the response and removal paths
    pub response: bool,
    pub land: bool,
}

impl AiRule {
    pub fn warns(&self) -> bool {
        !self.land && self.prio <= 0 && !self.response
    }

    pub fn line(&self) -> String {
        if self.land {
            return format!("AI: {}: land (played, never cast)", self.card);
        }
        if self.warns() {
            return format!(
                "WARNING AI: {}: priority 0 and no response path: the AI may never cast it",
                self.card
            );
        }
        let src = if self.deck_rule { "deck rule" } else { "generic" };
        let resp = if self.response && self.prio <= 0 { ", cast as a response / removal only" } else { "" };
        format!("AI: {}: priority {} ({src}){resp}", self.card, self.prio)
    }
}

/// The deck's priority for each swapped-in card on mid-game boards of the swapped list (rounds 4 and 7 of two games
/// against the first tier's decks, adaptive AI): the best value seen, since a rule may wait for a graveyard or mana.
pub fn ai_rules(sim: &Sim, deck: &str, var: &[String], ins: &[String], opp_keys: &[String], seed0: u64) -> Result<Vec<AiRule>, String> {
    let vseat = sim.seat(deck, var)?;
    let settings = Arc::new(Settings::new(profile("loose")?, AiMode::Adaptive, sim.opts.temp));
    let mut best: Vec<i32> = vec![i32::MIN; ins.len()];
    let mut generic: Vec<i32> = vec![i32::MIN; ins.len()];
    let mut deck_differs = vec![false; ins.len()];
    for seed in [seed0, seed0 + 1] {
        for rounds in [4u32, 7] {
            let seats = sim.seats_for(seed, Some(deck), Some(&vseat), opp_keys, 3);
            let mut g = setup_game(sim.engine.db.clone(), sim.engine.registry.clone(), settings.clone(), seed, &seats, false);
            run_rounds(&mut g, rounds);
            let Some(me) = g.players.iter().position(|p| p.key == deck) else { continue };
            let me = PlayerId(me as u8);
            for (i, name) in ins.iter().enumerate() {
                let Some(c) = g.db.id(name) else { continue };
                let (d, gp) = (deck_prio(&g, me, c), generic_prio(&g, me, c));
                best[i] = best[i].max(d);
                generic[i] = generic[i].max(gp);
                deck_differs[i] |= d != gp;
            }
        }
    }
    let db = &sim.engine.db;
    ins.iter()
        .enumerate()
        .map(|(i, name)| {
            let c = db.id(name).ok_or_else(|| format!("{name} is not in data/cards.json"))?;
            let d = db.get(c);
            Ok(AiRule {
                card: name.clone(),
                prio: best[i].max(0),
                deck_rule: deck_differs[i],
                response: d.instant || d.tag(Tag::Ctr) || d.tag(Tag::Rem) || d.tag(Tag::Wipe) || d.tag(Tag::Prot),
                land: d.land,
            })
        })
        .collect()
}

/// one swapped-in card's dry-run numbers, summed over the profiles
#[derive(Debug, Clone, PartialEq)]
pub struct DryRow {
    pub card: String,
    pub games: i64,
    pub drawn: i64,
    pub cast: i64,
}

impl DryRow {
    pub fn drawn_pct(&self) -> f64 {
        100.0 * self.drawn as f64 / self.games.max(1) as f64
    }

    pub fn cast_pct(&self) -> f64 {
        if self.drawn == 0 { 0.0 } else { 100.0 * self.cast as f64 / self.drawn as f64 }
    }

    pub fn warns(&self) -> bool {
        self.drawn >= MIN_DRAWN && self.cast_pct() < MAX_CAST_PCT
    }

    pub fn line(&self) -> String {
        if self.warns() {
            format!(
                "WARNING: {} drawn {}, cast {} ({:.1}%): the deck's AI rarely casts it",
                self.card,
                self.drawn,
                self.cast,
                self.cast_pct()
            )
        } else {
            format!(
                "dry run: {}: drawn in {:.1}% of games ({}), cast in {:.1}% of those ({})",
                self.card,
                self.drawn_pct(),
                self.drawn,
                self.cast_pct(),
                self.cast
            )
        }
    }
}

/// the drawn / cast numbers of each card (a land counts as drawn when seen, and is never cast, so lands are left to
/// the caller to skip)
pub fn dry_rows(analyses: &[Analysis], cards: &[String]) -> Vec<DryRow> {
    let games: i64 = analyses.iter().map(|a| a.n).sum();
    cards
        .iter()
        .map(|c| {
            let (mut drawn, mut cast) = (0, 0);
            for a in analyses {
                let (s, k) = (a.seen.get(c), a.cast.get(c));
                drawn += s.max(k);
                cast += k;
            }
            DryRow { card: c.clone(), games, drawn, cast }
        })
        .collect()
}

pub enum Outcome {
    Continue,
    /// stop before the main run, with this exit code
    Stop(i32),
}

/// Both steps for a `--swap` run; prints their lines. `var` is the swapped list.
pub fn run(sim: &Sim, a: &Args, swaps: &[(String, String)], var: &[String]) -> Result<Outcome, String> {
    let Some(deck) = a.deck.as_deref() else { return Ok(Outcome::Continue) };
    let plan = tier_plan(sim, a)?;
    let Some(tier) = plan.tiers.first().cloned() else { return Ok(Outcome::Continue) };
    let keys = sim.pool_keys(&tier);
    let ins: Vec<String> = swaps.iter().map(|s| s.1.clone()).collect();
    println!("Preflight (--no-preflight skips it): is each swapped-in card ever cast by {deck}'s AI?");
    let rules = ai_rules(sim, deck, var, &ins, &keys, a.seed)?;
    let mut bad = false;
    for r in &rules {
        println!("  {}", r.line());
        bad |= r.warns();
    }
    sim.prog.lock().unwrap().on = !a.quiet;
    let ctx = Ctx { sim, verbose: false };
    let seat = sim.seat(deck, var)?;
    let mut analyses = vec![];
    {
        let mut pr = sim.prog.lock().unwrap();
        pr.plan(DRY_GAMES * a.profiles.len() as u64);
        pr.announce(&format!("preflight dry run: swapped list, {DRY_GAMES} per profile on {tier}"), "adaptive", sim.opts.jobs);
    }
    for prof in &a.profiles {
        sim.prog.lock().unwrap().label = format!("preflight {tier} / {prof}");
        let st = Settings::new(profile(prof)?, AiMode::Adaptive, sim.opts.temp);
        analyses.push(ctx.analyze_with(deck, &seat, &keys, st, DRY_GAMES, a.seed));
    }
    println!(
        "  dry run: {DRY_GAMES} adaptive-AI games per profile ({}) of the swapped list vs {}",
        a.profiles.join(", "),
        crate::decks::tier_label(&tier)
    );
    let mut stop = false;
    for (row, r) in dry_rows(&analyses, &ins).iter().zip(&rules) {
        if r.land {
            continue;
        }
        println!("  {}", row.line());
        stop |= row.warns();
    }
    if stop {
        println!(
            "Preflight stopped the run: the AI above rarely casts a swapped-in card, so the result would measure a dead \
             card. Fix the deck's priority (ai::decks::deck_prio / its impls/<deck>.rs prio) or rerun with --no-preflight."
        );
        return Ok(Outcome::Stop(EXIT_STOP));
    }
    if bad {
        println!("  (the AI-rule warning above did not show up in the dry run; check the card's drawn / cast numbers after the main run)");
    }
    if a.preflight_only {
        return Ok(Outcome::Stop(0));
    }
    println!();
    Ok(Outcome::Continue)
}

/// whether a run does the preflight: every plain `--swap` run, unless `--no-preflight`
pub fn wanted(a: &Args, swaps: &[(String, String)]) -> bool {
    !swaps.is_empty() && !a.no_preflight && !a.analyze && !a.all_decks && a.calibrate.is_none() && a.trace.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    fn rules(sim: &Sim, deck: &str, out: &str, inn: &str, tier: &str) -> AiRule {
        let swaps = vec![(out.to_string(), inn.to_string())];
        let var = crate::swap_in_place(&sim.mine[deck], &swaps);
        ai_rules(sim, deck, &var, &[inn.to_string()], &sim.pool_keys(tier), 500000).unwrap().remove(0)
    }

    fn args(extra: &[&str]) -> Args {
        let mut v: Vec<String> = ["--deck", "kefka", "--pool", "t3", "--swap", "A=>B"].iter().map(|s| s.to_string()).collect();
        v.extend(extra.iter().map(|s| s.to_string()));
        crate::args::parse(&v).unwrap()
    }

    /// a Kefka card whose priority is 0 on a plain board and that is no instant: the same shape as Bribery before the
    /// audit gave it an arm in kefka_prio
    #[test]
    fn a_card_the_deck_rule_ranks_zero_gets_the_warning() {
        let s = sim();
        let out = s.mine["kefka"].iter().find(|c| s.engine.db.id(c).is_some_and(|i| !s.engine.db.get(i).land)).unwrap().clone();
        let r = rules(&s, "kefka", &out, "Underworld Breach", "t3");
        assert_eq!(r.prio, 0, "{r:?}");
        assert!(r.warns() && !r.response, "{r:?}");
        assert_eq!(r.line(), "WARNING AI: Underworld Breach: priority 0 and no response path: the AI may never cast it");
    }

    #[test]
    fn a_card_the_deck_ranks_positive_does_not() {
        let s = sim();
        let out = s.mine["kefka"].iter().find(|c| s.engine.db.id(c).is_some_and(|i| !s.engine.db.get(i).land)).unwrap().clone();
        // Psychosis Crawler: Kefka's own rule ranks it (kefka_prio), so no warning
        let crawler = rules(&s, "kefka", &out, "Psychosis Crawler", "t3");
        assert!(!crawler.warns() && crawler.prio > 0 && crawler.deck_rule, "{crawler:?}");
        assert!(crawler.line().ends_with("(deck rule)"), "{}", crawler.line());
        let ring = rules(&s, "kefka", &out, "Sol Ring", "t3");
        assert!(!ring.warns() && ring.prio > 0 && !ring.deck_rule, "{ring:?}");
        assert!(ring.line().ends_with("(generic)"));
    }

    #[test]
    fn a_response_card_is_not_warned_about() {
        let r = AiRule { card: "Counterspell".into(), prio: 0, deck_rule: false, response: true, land: false };
        assert!(!r.warns());
        assert!(r.line().contains("response"));
    }

    #[test]
    fn the_dry_run_stops_on_a_card_drawn_often_and_cast_rarely() {
        let mut a = Analysis::default();
        a.n = 200;
        a.seen.add("Dead".to_string(), 30);
        a.cast.add("Dead".to_string(), 1);
        a.seen.add("Fine".to_string(), 30);
        a.cast.add("Fine".to_string(), 20);
        a.seen.add("Rare".to_string(), 10); // drawn under 20 times: too little to judge
        let rows = dry_rows(&[a], &["Dead".to_string(), "Fine".to_string(), "Rare".to_string()]);
        assert!(rows[0].warns() && !rows[1].warns() && !rows[2].warns());
        assert_eq!(rows[0].line(), "WARNING: Dead drawn 30, cast 1 (3.3%): the deck's AI rarely casts it");
        assert!(rows[1].line().starts_with("dry run: Fine"));
    }

    #[test]
    fn no_preflight_skips_it_and_only_plain_swap_runs_get_it() {
        let swaps = vec![("A".to_string(), "B".to_string())];
        assert!(wanted(&args(&[]), &swaps));
        assert!(wanted(&args(&["--preflight-only"]), &swaps));
        assert!(!wanted(&args(&["--no-preflight"]), &swaps));
        assert!(!wanted(&args(&[]), &[]));
        assert!(!wanted(&args(&["--analyze"]), &swaps));
    }
}
