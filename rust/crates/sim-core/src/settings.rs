//! Run-wide settings. In Python these are module globals set once per run and read everywhere: the interaction
//! profile's counter thresholds (`engine.CTHRESH`, `CTHRESH_DEFAULT`, `instant_extra`), the AI mode
//! (`engine.AI_MODE`), the heuristic AI's temperature scale (`brain.TEMP_SCALE`) and the look-ahead's knobs
//! (`search.KEYS`, `ROLLOUTS`, `TOP_K`, `HORIZON`, the step caps). Here they are one value a `Game` shares through
//! an `Arc`, so copying a game doesn't copy them and two runs with different settings can share a process.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// counter only big threats (importance >= 7), hold instant removal for emergencies
    Conservative,
    /// counter at importance >= 6, use instant removal as freely as sorcery removal
    Loose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiMode {
    /// the heuristic AI alone (Python: `--ai adaptive`)
    Adaptive,
    /// the heuristic AI with look-ahead at main phases, attacks and counters (the default)
    Lookahead,
}

/// the default ramp weight (see `Search::ramp_w`): off. At 1.0 the AI cast ramp early but won no more
/// (audit/ramp/2026-10-15)
pub const RAMP_W: f64 = 0.0;

// --- ai-eval
/// the default margin (see `Search::margin`): one standard error of the paired difference
pub const MARGIN: f64 = 1.0;

/// the default hold weight (see `Search::hold_w`)
pub const HOLD_W: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Search {
    pub rollouts: u32,
    pub top_k: usize,
    pub horizon: u32,
    pub combo_w: f64,
    /// the look-ahead's bonus for casting early ramp, per mana per own turn left (`search::ramp_bonus`); 0: no
    /// bonus (the old search). Env SEARCH_RAMP_W overrides the default for A/B runs
    pub ramp_w: f64,
    /// the look-ahead's bonus for holding mana up (the "stop (hold mana)" line), times the heuristic's hold value
    /// (`search::hold_bonus`); 0: no bonus (the old search). Env SEARCH_HOLD_W overrides the default for A/B runs
    pub hold_w: f64,
    /// --- ai-eval: `search::strength` values each permanent for what it will produce (`search::output_value`). Env
    /// AI_EVAL_OUTPUT=0 turns it off (the old evaluation), for A/B runs
    pub output: bool,
    /// --- ai-eval: the hold bonus goes to each playout that kept the held card's mana up through p's turn
    /// (`search::choose`), not to the "stop (hold mana)" label. Env AI_EVAL_HOLD=0: the old rule, for A/B runs
    pub hold_kept: bool,
    /// --- ai-eval: the look-ahead departs from the heuristic's top play only when its pick wins by more than `margin`
    /// standard errors of the paired difference (`search::margin_holds`); 0: the bare argmax (the old search). Env
    /// AI_EVAL_MARGIN overrides the default for A/B runs
    pub margin: f64,
    pub playout_work: u64,
    pub game_decisions: u32,
    pub board_limit: usize,
    pub game_search_work: u64,
}

impl Default for Search {
    /// search.py's values
    fn default() -> Search {
        Search {
            rollouts: 6,
            top_k: 5,
            horizon: 1,
            combo_w: 12.0,
            ramp_w: std::env::var("SEARCH_RAMP_W")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|v| v.parse().unwrap_or_else(|_| panic!("bad SEARCH_RAMP_W {v:?}")))
                .unwrap_or(RAMP_W),
            hold_w: std::env::var("SEARCH_HOLD_W")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|v| v.parse().unwrap_or_else(|_| panic!("bad SEARCH_HOLD_W {v:?}")))
                .unwrap_or(HOLD_W),
            output: planner_from(std::env::var("AI_EVAL_OUTPUT").ok().as_deref()), // --- ai-eval
            hold_kept: planner_from(std::env::var("AI_EVAL_HOLD").ok().as_deref()), // --- ai-eval
            margin: std::env::var("AI_EVAL_MARGIN")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|v| v.parse().unwrap_or_else(|_| panic!("bad AI_EVAL_MARGIN {v:?}")))
                .unwrap_or(MARGIN), // --- ai-eval
            playout_work: 2_000,
            game_decisions: 1000,
            board_limit: 150,
            game_search_work: 3_000_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub profile: Profile,
    pub ai: AiMode,
    /// multiplies every deck's temperature (`--temp`)
    pub temp_scale: f64,
    /// counter threshold per deck key (your decks); outside decks use `cthresh_default`
    pub cthresh: HashMap<String, i32>,
    pub cthresh_default: i32,
    /// extra importance instant removal needs before it's used proactively (conservative: 2, loose: 0)
    pub instant_extra: i32,
    pub search: Search,
    /// engine steps one game may take (Python's `E.GAME_WORK`)
    pub game_work: u64,
    /// fitted or hand-set style per deck key (`--knobs FILE`, `--fit`), read before the built-in tables
    /// (`ai::style`); empty: every deck plays its built-in style
    pub styles: HashMap<String, crate::ai::Style>,
    /// Tier 6: the lands (hand plus battlefield) a deck needs before it casts Mox Diamond, counting the one it
    /// discards (`ai::pool`); 3 keeps two after the discard
    pub t6_moxd_lands: usize,
    /// --- ai-planner: the line planner for the Tier 6 decks (`ai::planner`); env PLANNER=0 or a knobs file's
    /// `[ai] planner = false` turns it off (the step 1 AI)
    pub planner: bool,
    /// --- ai-planner: when no play leaves a line ready next turn, the planner still casts a tutor for the piece that
    /// brings the closest line nearest. Off unless env PLANNER_PROGRESS=1 (an A/B switch: it measured slower, see
    /// audit/ai-foundations/step2-planner.md)
    pub planner_progress: bool,
    /// --- ai-eval: step 3's fixes A and B (`t6_table::engine_first`: a Tier 6 engine, commander or mana permanent
    /// the heuristic rates best is cast before the look-ahead). On unless env AI_FLOW_MUST=0 (an A/B switch: Part 1
    /// of the ai-eval brief measures the look-ahead without them)
    pub flow_must: bool,
}

impl Settings {
    /// engine.PROFILES
    pub fn new(profile: Profile, ai: AiMode, temp_scale: f64) -> Settings {
        let (mine, default, instant_extra) = match profile {
            Profile::Conservative => (7, 7, 2),
            Profile::Loose => (6, 6, 0),
        };
        let mut cthresh: HashMap<String, i32> =
            ["veyran", "sauron", "galadriel", "yshtola", "alela", "kefka", "omnath", "sheoldred", "cloud", "karn"]
                .iter()
                .map(|k| (k.to_string(), mine))
                .collect();
        cthresh.insert("seph".into(), 6);
        Settings {
            profile,
            ai,
            temp_scale,
            cthresh,
            cthresh_default: default,
            instant_extra,
            search: Search::default(),
            game_work: 20_000,
            styles: HashMap::new(),
            t6_moxd_lands: 3,
            planner: planner_default(), // --- ai-planner
            planner_progress: matches!(std::env::var("PLANNER_PROGRESS").ok().as_deref(), Some("1")), // --- ai-planner
            flow_must: planner_from(std::env::var("AI_FLOW_MUST").ok().as_deref()), // --- ai-eval
        }
    }

    pub fn counter_threshold(&self, deck: &str) -> Option<i32> {
        self.cthresh.get(deck).copied()
    }
}

// --- ai-planner
/// the line planner is on unless env PLANNER is `0` (or `false`, `off`)
pub fn planner_default() -> bool {
    planner_from(std::env::var("PLANNER").ok().as_deref())
}

/// the planner setting an env value gives (unset or anything else: on)
pub fn planner_from(v: Option<&str>) -> bool {
    !matches!(v.map(str::trim), Some("0" | "false" | "off"))
}
