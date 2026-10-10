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

#[derive(Debug, Clone)]
pub struct Search {
    pub rollouts: u32,
    pub top_k: usize,
    pub horizon: u32,
    pub combo_w: f64,
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
}

impl Settings {
    /// engine.PROFILES
    pub fn new(profile: Profile, ai: AiMode, temp_scale: f64) -> Settings {
        let (mine, default, instant_extra) = match profile {
            Profile::Conservative => (7, 7, 2),
            Profile::Loose => (6, 6, 0),
        };
        let mut cthresh: HashMap<String, i32> = ["veyran", "sauron", "galadriel", "yshtola", "alela", "jodah"]
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
        }
    }

    pub fn counter_threshold(&self, deck: &str) -> Option<i32> {
        self.cthresh.get(deck).copied()
    }
}
